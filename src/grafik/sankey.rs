//! Sankey serisi — `echarts/src/chart/sankey` aktarımı.
//!
//! DAG yerleşimi `sankeyLayout.ts`, renk eşleme `sankeyVisual.ts`, şerit ve
//! etiket geometrisi `SankeyView.ts` ile aynı aşama sırasını izler. Çözülen
//! [`SankeyYerleşimi`] rasterdan bağımsız yapısal kanıtın da kaynağıdır.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::sahne::yuvarlak_dikdörtgen_yolu;
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::pasta::zengin_etiketi_hizalı_yaz;
use crate::koordinat::Dikdörtgen;
use crate::model::Uzunluk;
use crate::model::sankey::{
    SankeyBağı, SankeyDurumu, SankeyDüğümHizası, SankeyDüğümü, SankeyKenarBoyası, SankeySerisi,
    SankeySırası, SankeyVurguOdağı, SankeyYönü, SankeyÇizgiStili, SankeyÖğeStili,
};
use crate::model::stil::{
    Etiket, EtiketDöndürme, EtiketKonumu, EtiketYaması, YazıDikeyHizası, YazıStili,
    YazıYatayHizası, ÇizgiTürü,
};
use crate::renk::{Dolgu, Renk, RenkDurağı};
use crate::tema;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SankeyHatası(pub String);

impl fmt::Display for SankeyHatası {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SankeyHatası {}

/// Yerleşimi ve bütün normal stil kalıtımı çözülmüş Sankey düğümü.
#[derive(Clone, Debug)]
pub struct SankeyYerleşikDüğüm {
    pub veri_sırası: usize,
    pub ad: String,
    pub değer: f64,
    pub derinlik: usize,
    pub alan: Dikdörtgen,
    pub renk: Dolgu,
    pub öğe_stili: SankeyÖğeStili,
    pub etiket: Etiket,
    pub etiket_metni: String,
    pub etiket_konumu: (f32, f32),
    pub etiket_yatay_hizası: YatayHiza,
    pub etiket_dikey_hizası: DikeyHiza,
    pub etiket_dönüşü: f32,
    pub vurgu: SankeyDurumu,
    pub bulanık: SankeyDurumu,
    pub seçili: SankeyDurumu,
    pub gelen_bağlar: Vec<usize>,
    pub giden_bağlar: Vec<usize>,
}

/// Yerleşimi ve normal stil kalıtımı çözülmüş Sankey bağı.
#[derive(Clone, Debug)]
pub struct SankeyYerleşikBağ {
    pub veri_sırası: usize,
    pub kaynak_sırası: usize,
    pub hedef_sırası: usize,
    pub kaynak: String,
    pub hedef: String,
    pub değer: f64,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub cpx1: f32,
    pub cpy1: f32,
    pub cpx2: f32,
    pub cpy2: f32,
    pub kalınlık: f32,
    pub yön: SankeyYönü,
    pub dolgu: Dolgu,
    pub çizgi_stili: SankeyÇizgiStili,
    pub kenar_etiketi: Etiket,
    pub etiket_metni: String,
    pub etiket_konumu: (f32, f32),
    pub vurgu: SankeyDurumu,
    pub bulanık: SankeyDurumu,
    pub seçili: SankeyDurumu,
}

#[derive(Clone, Debug)]
pub struct SankeyYerleşimi {
    pub alan: Dikdörtgen,
    pub düğümler: Vec<SankeyYerleşikDüğüm>,
    pub bağlar: Vec<SankeyYerleşikBağ>,
}

#[derive(Clone, Debug)]
struct ÇalışmaDüğümü {
    kaynak: SankeyDüğümü,
    değer: f64,
    derinlik: usize,
    ters_yükseklik: usize,
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    gelen: Vec<usize>,
    giden: Vec<usize>,
}

#[derive(Clone, Debug)]
struct ÇalışmaBağı {
    kaynak: SankeyBağı,
    kaynak_sırası: usize,
    hedef_sırası: usize,
    dy: f64,
    sy: f64,
    ty: f64,
}

fn öğe_stili_yama_uygula(taban: &SankeyÖğeStili, yama: &SankeyÖğeStili) -> SankeyÖğeStili {
    let mut sonuç = taban.clone();
    macro_rules! alan {
        ($ad:ident) => {
            if yama.$ad.is_some() {
                sonuç.$ad.clone_from(&yama.$ad);
            }
        };
    }
    alan!(renk);
    alan!(kenarlık_rengi);
    alan!(kenarlık_kalınlığı);
    alan!(kenarlık_türü);
    alan!(kenarlık_yarıçapı);
    alan!(opaklık);
    alan!(gölge_bulanıklığı);
    alan!(gölge_rengi);
    alan!(gölge_kayması);
    sonuç
}

fn çizgi_stili_yama_uygula(
    taban: &SankeyÇizgiStili,
    yama: &SankeyÇizgiStili,
) -> SankeyÇizgiStili {
    let mut sonuç = taban.clone();
    macro_rules! alan {
        ($ad:ident) => {
            if yama.$ad.is_some() {
                sonuç.$ad.clone_from(&yama.$ad);
            }
        };
    }
    alan!(renk);
    alan!(opaklık);
    alan!(eğrilik);
    alan!(kalınlık);
    alan!(tür);
    alan!(gölge_bulanıklığı);
    alan!(gölge_rengi);
    alan!(gölge_kayması);
    sonuç
}

fn durum_yama_uygula(taban: &SankeyDurumu, yama: &SankeyDurumu) -> SankeyDurumu {
    let mut sonuç = taban.clone();
    if let Some(stil) = &yama.öğe_stili {
        sonuç.öğe_stili = Some(match &sonuç.öğe_stili {
            Some(taban) => öğe_stili_yama_uygula(taban, stil),
            None => stil.clone(),
        });
    }
    if let Some(stil) = &yama.çizgi_stili {
        sonuç.çizgi_stili = Some(match &sonuç.çizgi_stili {
            Some(taban) => çizgi_stili_yama_uygula(taban, stil),
            None => stil.clone(),
        });
    }
    if yama.etiket.is_some() {
        sonuç.etiket.clone_from(&yama.etiket);
    }
    if yama.kenar_etiketi.is_some() {
        sonuç.kenar_etiketi.clone_from(&yama.kenar_etiketi);
    }
    if yama.odak.is_some() {
        sonuç.odak = yama.odak;
    }
    if yama.devre_dışı.is_some() {
        sonuç.devre_dışı = yama.devre_dışı;
    }
    sonuç
}

/// ECharts box-layout alanını, verilmiş koordinat bileşeninin içerik kutusu
/// içinde çözer.
pub fn sankey_alanı(seri: &SankeySerisi, tuval: Dikdörtgen) -> Dikdörtgen {
    let sol = seri.sol.çöz(tuval.genişlik);
    let üst = seri.üst.çöz(tuval.yükseklik);
    let genişlik = seri.genişlik.map_or_else(
        || {
            seri.sağ.map_or(tuval.genişlik - sol, |sağ| {
                tuval.genişlik - sol - sağ.çöz(tuval.genişlik)
            })
        },
        |genişlik| genişlik.çöz(tuval.genişlik),
    );
    let yükseklik = seri.yükseklik.map_or_else(
        || {
            seri.alt.map_or(tuval.yükseklik - üst, |alt| {
                tuval.yükseklik - üst - alt.çöz(tuval.yükseklik)
            })
        },
        |yükseklik| yükseklik.çöz(tuval.yükseklik),
    );
    Dikdörtgen::yeni(
        tuval.x + sol,
        tuval.y + üst,
        genişlik.max(0.0),
        yükseklik.max(0.0),
    )
}

fn grafiği_kur(
    seri: &SankeySerisi,
) -> Result<(Vec<ÇalışmaDüğümü>, Vec<ÇalışmaBağı>), SankeyHatası> {
    let mut kaynaklar = seri.düğümler.clone();
    let mut anahtarlar = HashSet::new();
    for düğüm in &kaynaklar {
        anahtarlar.insert(düğüm.ad.clone());
        if let Some(kimlik) = &düğüm.kimlik {
            anahtarlar.insert(kimlik.clone());
        }
    }
    // Eski yalnız-bağ kurucusunun geriye uyumu. Resmî fixture'lar açık
    // `data` taşıdığı için bu dal onların veriIndex sırasını değiştirmez.
    for bağ in &seri.bağlar {
        for ad in [&bağ.kaynak, &bağ.hedef] {
            if anahtarlar.insert(ad.clone()) {
                kaynaklar.push(SankeyDüğümü::yeni(ad));
            }
        }
    }
    if kaynaklar.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut sıra_bul = HashMap::new();
    for (sıra, düğüm) in kaynaklar.iter().enumerate() {
        sıra_bul.entry(düğüm.ad.clone()).or_insert(sıra);
        if let Some(kimlik) = &düğüm.kimlik {
            sıra_bul.entry(kimlik.clone()).or_insert(sıra);
        }
    }
    let mut düğümler = kaynaklar
        .into_iter()
        .map(|kaynak| ÇalışmaDüğümü {
            kaynak,
            değer: 0.0,
            derinlik: 0,
            ters_yükseklik: 0,
            x: 0.0,
            y: 0.0,
            dx: 0.0,
            dy: 0.0,
            gelen: Vec::new(),
            giden: Vec::new(),
        })
        .collect::<Vec<_>>();
    let mut bağlar = Vec::with_capacity(seri.bağlar.len());
    for bağ in &seri.bağlar {
        if !bağ.değer.is_finite() || bağ.değer < 0.0 {
            return Err(SankeyHatası(format!(
                "Sankey bağı sonlu ve negatif olmayan değer taşımalı: {} -> {} = {}",
                bağ.kaynak, bağ.hedef, bağ.değer
            )));
        }
        let kaynak_sırası = sıra_bul.get(&bağ.kaynak).copied().ok_or_else(|| {
            SankeyHatası(format!("Sankey kaynak düğümü bulunamadı: {}", bağ.kaynak))
        })?;
        let hedef_sırası = sıra_bul.get(&bağ.hedef).copied().ok_or_else(|| {
            SankeyHatası(format!("Sankey hedef düğümü bulunamadı: {}", bağ.hedef))
        })?;
        let sıra = bağlar.len();
        if let Some(düğüm) = düğümler.get_mut(kaynak_sırası) {
            düğüm.giden.push(sıra);
        }
        if let Some(düğüm) = düğümler.get_mut(hedef_sırası) {
            düğüm.gelen.push(sıra);
        }
        bağlar.push(ÇalışmaBağı {
            kaynak: bağ.clone(),
            kaynak_sırası,
            hedef_sırası,
            dy: 0.0,
            sy: 0.0,
            ty: 0.0,
        });
    }
    for düğüm in &mut düğümler {
        let giden = düğüm
            .giden
            .iter()
            .filter_map(|sıra| bağlar.get(*sıra))
            .map(|bağ| bağ.kaynak.değer)
            .sum::<f64>();
        let gelen = düğüm
            .gelen
            .iter()
            .filter_map(|sıra| bağlar.get(*sıra))
            .map(|bağ| bağ.kaynak.değer)
            .sum::<f64>();
        düğüm.değer = giden
            .max(gelen)
            .max(düğüm.kaynak.değer.unwrap_or(0.0).max(0.0));
    }
    Ok((düğümler, bağlar))
}

fn düğüm_katmanlarını_hesapla(
    düğümler: &mut [ÇalışmaDüğümü],
    bağlar: &[ÇalışmaBağı],
    düğüm_genişliği: f64,
    genişlik: f64,
    yükseklik: f64,
    yön: SankeyYönü,
    hiza: SankeyDüğümHizası,
) -> Result<(), SankeyHatası> {
    let mut kalan_bağ = vec![true; bağlar.len()];
    let mut gelen_sayısı = düğümler
        .iter()
        .map(|düğüm| düğüm.gelen.len())
        .collect::<Vec<_>>();
    let mut sıfırlar = gelen_sayısı
        .iter()
        .enumerate()
        .filter_map(|(sıra, derece)| (*derece == 0).then_some(sıra))
        .collect::<Vec<_>>();
    let mut x = 0usize;
    let mut en_büyük_açık_derinlik = None;
    while !sıfırlar.is_empty() {
        let mut sonraki = Vec::new();
        for sıra in sıfırlar {
            let açık = düğümler.get(sıra).and_then(|düğüm| düğüm.kaynak.derinlik);
            let derinlik = açık.unwrap_or(x);
            if açık.is_some() {
                en_büyük_açık_derinlik =
                    Some(en_büyük_açık_derinlik.map_or(derinlik, |eski: usize| eski.max(derinlik)));
            }
            if let Some(düğüm) = düğümler.get_mut(sıra) {
                düğüm.derinlik = derinlik;
                if yön == SankeyYönü::Dikey {
                    düğüm.dy = düğüm_genişliği;
                } else {
                    düğüm.dx = düğüm_genişliği;
                }
            }
            let giden = düğümler
                .get(sıra)
                .map(|düğüm| düğüm.giden.clone())
                .unwrap_or_default();
            for bağ_sırası in giden {
                if let Some(kalan) = kalan_bağ.get_mut(bağ_sırası) {
                    *kalan = false;
                }
                let Some(hedef) = bağlar.get(bağ_sırası).map(|bağ| bağ.hedef_sırası) else {
                    continue;
                };
                if let Some(derece) = gelen_sayısı.get_mut(hedef) {
                    *derece = derece.saturating_sub(1);
                    if *derece == 0 && !sonraki.contains(&hedef) {
                        sonraki.push(hedef);
                    }
                }
            }
        }
        x = x.saturating_add(1);
        sıfırlar = sonraki;
    }
    if kalan_bağ.iter().any(|kalan| *kalan) {
        return Err(SankeyHatası(
            "Sankey bir DAG olmalı; özgün veri döngü içeriyor".to_owned(),
        ));
    }
    let en_büyük_derinlik = en_büyük_açık_derinlik.unwrap_or(0).max(x.saturating_sub(1));
    match hiza {
        SankeyDüğümHizası::Sol => {}
        SankeyDüğümHizası::İkiYana => {
            for düğüm in düğümler.iter_mut() {
                if düğüm.kaynak.derinlik.is_none() && düğüm.giden.is_empty() {
                    düğüm.derinlik = en_büyük_derinlik;
                }
            }
        }
        SankeyDüğümHizası::Sağ => {
            // Resmî uygulama tüm düğümlerden geriye doğru katman katman
            // yürür; bir kaynak son atandığı sink uzaklığıyla hizalanır.
            let mut kalan = (0..düğümler.len()).collect::<Vec<_>>();
            let mut yükseklik_sırası = 0usize;
            while !kalan.is_empty() {
                let mut sonraki = Vec::new();
                for sıra in kalan {
                    if let Some(düğüm) = düğümler.get_mut(sıra) {
                        düğüm.ters_yükseklik = yükseklik_sırası;
                    }
                    let gelen = düğümler
                        .get(sıra)
                        .map(|düğüm| düğüm.gelen.clone())
                        .unwrap_or_default();
                    for bağ_sırası in gelen {
                        if let Some(kaynak) = bağlar.get(bağ_sırası).map(|bağ| bağ.kaynak_sırası)
                            && !sonraki.contains(&kaynak)
                        {
                            sonraki.push(kaynak);
                        }
                    }
                }
                kalan = sonraki;
                yükseklik_sırası = yükseklik_sırası.saturating_add(1);
            }
            for düğüm in düğümler.iter_mut() {
                if düğüm.kaynak.derinlik.is_none() {
                    düğüm.derinlik = en_büyük_derinlik.saturating_sub(düğüm.ters_yükseklik);
                }
            }
        }
    }
    let kx = if en_büyük_derinlik == 0 {
        0.0
    } else if yön == SankeyYönü::Dikey {
        (yükseklik - düğüm_genişliği) / en_büyük_derinlik as f64
    } else {
        (genişlik - düğüm_genişliği) / en_büyük_derinlik as f64
    };
    for düğüm in düğümler {
        let konum = düğüm.derinlik as f64 * kx;
        if yön == SankeyYönü::Dikey {
            düğüm.y = konum;
        } else {
            düğüm.x = konum;
        }
    }
    Ok(())
}

fn katman_grupları(düğümler: &[ÇalışmaDüğümü]) -> Vec<Vec<usize>> {
    let mut gruplar = HashMap::<usize, Vec<usize>>::new();
    for (sıra, düğüm) in düğümler.iter().enumerate() {
        gruplar.entry(düğüm.derinlik).or_default().push(sıra);
    }
    let mut anahtarlar = gruplar.keys().copied().collect::<Vec<_>>();
    anahtarlar.sort_unstable();
    anahtarlar
        .into_iter()
        .filter_map(|anahtar| gruplar.remove(&anahtar))
        .collect()
}

fn düğüm_merkezi(düğüm: &ÇalışmaDüğümü, yön: SankeyYönü) -> f64 {
    if yön == SankeyYönü::Dikey {
        düğüm.x + düğüm.dx / 2.0
    } else {
        düğüm.y + düğüm.dy / 2.0
    }
}

fn düğüm_çapraz_konumu(düğüm: &ÇalışmaDüğümü, yön: SankeyYönü) -> f64 {
    if yön == SankeyYönü::Dikey {
        düğüm.x
    } else {
        düğüm.y
    }
}

fn düğüm_çapraz_boyutu(düğüm: &ÇalışmaDüğümü, yön: SankeyYönü) -> f64 {
    if yön == SankeyYönü::Dikey {
        düğüm.dx
    } else {
        düğüm.dy
    }
}

fn düğüm_çapraz_ayarla(düğüm: &mut ÇalışmaDüğümü, yön: SankeyYönü, konum: f64) {
    if yön == SankeyYönü::Dikey {
        düğüm.x = konum;
    } else {
        düğüm.y = konum;
    }
}

fn çakışmaları_çöz(
    gruplar: &mut [Vec<usize>],
    düğümler: &mut [ÇalışmaDüğümü],
    boşluk: f64,
    genişlik: f64,
    yükseklik: f64,
    yön: SankeyYönü,
    sıralama: SankeySırası,
) {
    let sınır = if yön == SankeyYönü::Dikey {
        genişlik
    } else {
        yükseklik
    };
    for grup in gruplar {
        if sıralama != SankeySırası::Veri {
            grup.sort_by(|a, b| {
                let a = düğümler
                    .get(*a)
                    .map_or(0.0, |d| düğüm_çapraz_konumu(d, yön));
                let b = düğümler
                    .get(*b)
                    .map_or(0.0, |d| düğüm_çapraz_konumu(d, yön));
                a.partial_cmp(&b).unwrap_or(Ordering::Equal)
            });
        }
        let mut y0 = 0.0;
        for sıra in grup.iter().copied() {
            let Some(düğüm) = düğümler.get_mut(sıra) else {
                continue;
            };
            let konum = düğüm_çapraz_konumu(düğüm, yön).max(y0);
            düğüm_çapraz_ayarla(düğüm, yön, konum);
            y0 = konum + düğüm_çapraz_boyutu(düğüm, yön) + boşluk;
        }
        let taşma = y0 - boşluk - sınır;
        if taşma > 0.0 {
            let Some(son_sıra) = grup.last().copied() else {
                continue;
            };
            if let Some(son) = düğümler.get_mut(son_sıra) {
                let konum = düğüm_çapraz_konumu(son, yön) - taşma;
                düğüm_çapraz_ayarla(son, yön, konum);
            }
            let mut y0 = düğümler
                .get(son_sıra)
                .map_or(0.0, |düğüm| düğüm_çapraz_konumu(düğüm, yön));
            for sıra in grup.iter().rev().skip(1).copied() {
                let Some(düğüm) = düğümler.get_mut(sıra) else {
                    continue;
                };
                let konum = düğüm_çapraz_konumu(düğüm, yön);
                let çakışma = konum + düğüm_çapraz_boyutu(düğüm, yön) + boşluk - y0;
                if çakışma > 0.0 {
                    düğüm_çapraz_ayarla(düğüm, yön, konum - çakışma);
                }
                y0 = düğüm_çapraz_konumu(düğüm, yön);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn düğüm_derinliklerini_hesapla(
    düğümler: &mut [ÇalışmaDüğümü],
    bağlar: &mut [ÇalışmaBağı],
    boşluk: f64,
    genişlik: f64,
    yükseklik: f64,
    yineleme: usize,
    yön: SankeyYönü,
    sıralama: SankeySırası,
) {
    let mut gruplar = katman_grupları(düğümler);
    let mut en_küçük_ky = f64::INFINITY;
    for grup in &gruplar {
        let toplam = grup
            .iter()
            .filter_map(|sıra| düğümler.get(*sıra))
            .map(|düğüm| düğüm.değer)
            .sum::<f64>();
        if toplam <= 0.0 {
            continue;
        }
        let çapraz = if yön == SankeyYönü::Dikey {
            genişlik
        } else {
            yükseklik
        };
        let ky = (çapraz - grup.len().saturating_sub(1) as f64 * boşluk) / toplam;
        en_küçük_ky = en_küçük_ky.min(ky);
    }
    if !en_küçük_ky.is_finite() {
        en_küçük_ky = 0.0;
    }
    for grup in &gruplar {
        for (grup_sırası, sıra) in grup.iter().copied().enumerate() {
            if let Some(düğüm) = düğümler.get_mut(sıra) {
                let boyut = düğüm.değer * en_küçük_ky;
                if yön == SankeyYönü::Dikey {
                    düğüm.x = grup_sırası as f64;
                    düğüm.dx = boyut;
                } else {
                    düğüm.y = grup_sırası as f64;
                    düğüm.dy = boyut;
                }
            }
        }
    }
    for bağ in bağlar.iter_mut() {
        bağ.dy = bağ.kaynak.değer * en_küçük_ky;
    }
    çakışmaları_çöz(
        &mut gruplar,
        düğümler,
        boşluk,
        genişlik,
        yükseklik,
        yön,
        sıralama,
    );
    let mut alfa = 1.0;
    for _ in 0..yineleme {
        alfa *= 0.99;
        for grup in gruplar.iter().rev() {
            for sıra in grup.iter().copied() {
                let Some(düğüm) = düğümler.get(sıra) else {
                    continue;
                };
                if düğüm.giden.is_empty() {
                    continue;
                }
                let ağırlık = düğüm
                    .giden
                    .iter()
                    .filter_map(|bağ_sırası| bağlar.get(*bağ_sırası))
                    .map(|bağ| bağ.kaynak.değer)
                    .sum::<f64>();
                let hedef = if ağırlık > 0.0 {
                    düğüm
                        .giden
                        .iter()
                        .filter_map(|bağ_sırası| bağlar.get(*bağ_sırası))
                        .map(|bağ| {
                            düğümler
                                .get(bağ.hedef_sırası)
                                .map_or(0.0, |hedef| düğüm_merkezi(hedef, yön))
                                * bağ.kaynak.değer
                        })
                        .sum::<f64>()
                        / ağırlık
                } else {
                    let sayısı = düğüm.giden.len().max(1) as f64;
                    düğüm
                        .giden
                        .iter()
                        .filter_map(|bağ_sırası| bağlar.get(*bağ_sırası))
                        .map(|bağ| {
                            düğümler
                                .get(bağ.hedef_sırası)
                                .map_or(0.0, |hedef| düğüm_merkezi(hedef, yön))
                        })
                        .sum::<f64>()
                        / sayısı
                };
                let merkez = düğüm_merkezi(düğüm, yön);
                let yeni = düğüm_çapraz_konumu(düğüm, yön) + (hedef - merkez) * alfa;
                if let Some(düğüm) = düğümler.get_mut(sıra) {
                    düğüm_çapraz_ayarla(düğüm, yön, yeni);
                }
            }
        }
        çakışmaları_çöz(
            &mut gruplar,
            düğümler,
            boşluk,
            genişlik,
            yükseklik,
            yön,
            sıralama,
        );
        for grup in &gruplar {
            for sıra in grup.iter().copied() {
                let Some(düğüm) = düğümler.get(sıra) else {
                    continue;
                };
                if düğüm.gelen.is_empty() {
                    continue;
                }
                let ağırlık = düğüm
                    .gelen
                    .iter()
                    .filter_map(|bağ_sırası| bağlar.get(*bağ_sırası))
                    .map(|bağ| bağ.kaynak.değer)
                    .sum::<f64>();
                let kaynak = if ağırlık > 0.0 {
                    düğüm
                        .gelen
                        .iter()
                        .filter_map(|bağ_sırası| bağlar.get(*bağ_sırası))
                        .map(|bağ| {
                            düğümler
                                .get(bağ.kaynak_sırası)
                                .map_or(0.0, |kaynak| düğüm_merkezi(kaynak, yön))
                                * bağ.kaynak.değer
                        })
                        .sum::<f64>()
                        / ağırlık
                } else {
                    let sayısı = düğüm.gelen.len().max(1) as f64;
                    düğüm
                        .gelen
                        .iter()
                        .filter_map(|bağ_sırası| bağlar.get(*bağ_sırası))
                        .map(|bağ| {
                            düğümler
                                .get(bağ.kaynak_sırası)
                                .map_or(0.0, |kaynak| düğüm_merkezi(kaynak, yön))
                        })
                        .sum::<f64>()
                        / sayısı
                };
                let merkez = düğüm_merkezi(düğüm, yön);
                let yeni = düğüm_çapraz_konumu(düğüm, yön) + (kaynak - merkez) * alfa;
                if let Some(düğüm) = düğümler.get_mut(sıra) {
                    düğüm_çapraz_ayarla(düğüm, yön, yeni);
                }
            }
        }
        çakışmaları_çöz(
            &mut gruplar,
            düğümler,
            boşluk,
            genişlik,
            yükseklik,
            yön,
            sıralama,
        );
    }
}

fn bağ_derinliklerini_hesapla(
    düğümler: &mut [ÇalışmaDüğümü],
    bağlar: &mut [ÇalışmaBağı],
    yön: SankeyYönü,
) {
    // Sıralama sırasında aynı dilimi hem mutlu hem de değişmez ödünç almamak
    // için karşı eksen konumlarını ECharts'ın bu aşamadaki sabit görünümü gibi
    // tek seferde donduruyoruz.
    let çapraz_konumlar = düğümler
        .iter()
        .map(|düğüm| düğüm_çapraz_konumu(düğüm, yön))
        .collect::<Vec<_>>();
    for düğüm in düğümler.iter_mut() {
        düğüm.giden.sort_by(|a, b| {
            let a = bağlar
                .get(*a)
                .and_then(|bağ| çapraz_konumlar.get(bağ.hedef_sırası))
                .copied()
                .unwrap_or(0.0);
            let b = bağlar
                .get(*b)
                .and_then(|bağ| çapraz_konumlar.get(bağ.hedef_sırası))
                .copied()
                .unwrap_or(0.0);
            a.partial_cmp(&b).unwrap_or(Ordering::Equal)
        });
        düğüm.gelen.sort_by(|a, b| {
            let a = bağlar
                .get(*a)
                .and_then(|bağ| çapraz_konumlar.get(bağ.kaynak_sırası))
                .copied()
                .unwrap_or(0.0);
            let b = bağlar
                .get(*b)
                .and_then(|bağ| çapraz_konumlar.get(bağ.kaynak_sırası))
                .copied()
                .unwrap_or(0.0);
            a.partial_cmp(&b).unwrap_or(Ordering::Equal)
        });
    }
    for düğüm in düğümler {
        let mut sy = 0.0;
        for sıra in &düğüm.giden {
            if let Some(bağ) = bağlar.get_mut(*sıra) {
                bağ.sy = sy;
                sy += bağ.dy;
            }
        }
        let mut ty = 0.0;
        for sıra in &düğüm.gelen {
            if let Some(bağ) = bağlar.get_mut(*sıra) {
                bağ.ty = ty;
                ty += bağ.dy;
            }
        }
    }
}

fn renk_eşle(değer: f64, en_az: f64, en_çok: f64, renkler: &[Renk]) -> Renk {
    let Some(ilk) = renkler.first().copied() else {
        return tema::palet_rengi(0);
    };
    if renkler.len() == 1 {
        return ilk;
    }
    let oran = if (en_çok - en_az).abs() <= f64::EPSILON {
        0.5
    } else {
        ((değer - en_az) / (en_çok - en_az)).clamp(0.0, 1.0)
    };
    let konum = oran * (renkler.len() - 1) as f64;
    let sıra = (konum.floor() as usize).min(renkler.len() - 1);
    let sonraki = (sıra + 1).min(renkler.len() - 1);
    let sol = renkler.get(sıra).copied().unwrap_or(ilk);
    let sağ = renkler.get(sonraki).copied().unwrap_or(sol);
    sol.karıştır(sağ, (konum - sıra as f64) as f32)
}

fn etiket_metni(etiket: &Etiket, değer: f64, ad: &str, seri_adı: Option<&str>) -> String {
    etiket.biçimleyici.as_ref().map_or_else(
        || ad.to_owned(),
        |biçimleyici| biçimleyici.uygula_bağlamla(değer, ad, seri_adı.unwrap_or_default(), ad),
    )
}

fn sayı_metni(değer: f64) -> String {
    if değer.fract().abs() <= f64::EPSILON {
        format!("{değer:.0}")
    } else {
        değer.to_string()
    }
}

fn etiket_hizalarını_çöz(
    etiket: &Etiket,
    doğal_yatay: YatayHiza,
    doğal_dikey: DikeyHiza,
) -> (YatayHiza, DikeyHiza) {
    let yatay = etiket.yatay_hiza.map_or(doğal_yatay, |hiza| match hiza {
        YazıYatayHizası::Sol => YatayHiza::Sol,
        YazıYatayHizası::Orta => YatayHiza::Orta,
        YazıYatayHizası::Sağ => YatayHiza::Sağ,
    });
    let dikey = etiket.dikey_hiza.map_or(doğal_dikey, |hiza| match hiza {
        YazıDikeyHizası::Üst => DikeyHiza::Üst,
        YazıDikeyHizası::Orta => DikeyHiza::Orta,
        YazıDikeyHizası::Alt => DikeyHiza::Alt,
    });
    (yatay, dikey)
}

fn düğüm_etiket_geometrisi(
    alan: Dikdörtgen,
    etiket: &Etiket,
    kenarlık_kalınlığı: f32,
) -> ((f32, f32), YatayHiza, DikeyHiza, f32) {
    let d = etiket.uzaklık;
    // zrender bağlı metin konumunu, şeklin vuruşu dahil dünya sınırından
    // çözer. Sağ etikette 1 px vuruş bu nedenle şeklin sağına 0,5 px daha
    // ekler. Negatif yüksekliğe sahip dikdörtgenler de önce pozitif sınıra
    // normalleştirilir (resmî `sankey-itemstyle` örneğinde bu durum oluşur).
    let boya_alanı = pozitif_alan(alan);
    let kalınlık = kenarlık_kalınlığı.max(0.0);
    let yarım_kenarlık = kalınlık / 2.0;
    let sınır = Dikdörtgen::yeni(
        boya_alanı.x - yarım_kenarlık,
        boya_alanı.y - yarım_kenarlık,
        boya_alanı.genişlik + kalınlık,
        boya_alanı.yükseklik + kalınlık,
    );
    let (konum, yatay, dikey) = match etiket.konum {
        EtiketKonumu::Sağ | EtiketKonumu::Dış => (
            (sınır.sağ() + d, sınır.merkez().1),
            YatayHiza::Sol,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::Sol => (
            (sınır.x - d, sınır.merkez().1),
            YatayHiza::Sağ,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::Üst => (
            (sınır.merkez().0, sınır.y - d),
            YatayHiza::Orta,
            DikeyHiza::Alt,
        ),
        EtiketKonumu::Alt => (
            (sınır.merkez().0, sınır.alt() + d),
            YatayHiza::Orta,
            DikeyHiza::Üst,
        ),
        EtiketKonumu::İçSol | EtiketKonumu::İçBaşlangıç => (
            (sınır.x + d, sınır.merkez().1),
            YatayHiza::Sol,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::İçSağ | EtiketKonumu::İçBitiş => (
            (sınır.sağ() - d, sınır.merkez().1),
            YatayHiza::Sağ,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::İçÜst => (
            (sınır.merkez().0, sınır.y + d),
            YatayHiza::Orta,
            DikeyHiza::Üst,
        ),
        EtiketKonumu::İçAlt => (
            (sınır.merkez().0, sınır.alt() - d),
            YatayHiza::Orta,
            DikeyHiza::Alt,
        ),
        EtiketKonumu::İçSolÜst | EtiketKonumu::SolÜst => {
            ((sınır.x + d, sınır.y + d), YatayHiza::Sol, DikeyHiza::Üst)
        }
        EtiketKonumu::İçSağÜst | EtiketKonumu::SağÜst => (
            (sınır.sağ() - d, sınır.y + d),
            YatayHiza::Sağ,
            DikeyHiza::Üst,
        ),
        EtiketKonumu::İçSolAlt | EtiketKonumu::SolAlt => (
            (sınır.x + d, sınır.alt() - d),
            YatayHiza::Sol,
            DikeyHiza::Alt,
        ),
        EtiketKonumu::İçSağAlt | EtiketKonumu::SağAlt => (
            (sınır.sağ() - d, sınır.alt() - d),
            YatayHiza::Sağ,
            DikeyHiza::Alt,
        ),
        _ => (sınır.merkez(), YatayHiza::Orta, DikeyHiza::Orta),
    };
    let (yatay, dikey) = etiket_hizalarını_çöz(etiket, yatay, dikey);
    let dönüş = match etiket.döndürme {
        EtiketDöndürme::Derece(derece) => derece.to_radians(),
        _ => 0.0,
    };
    // `zengin_etiketi_hizalı_yaz` label.offset'i tek kez uygular.
    (konum, yatay, dikey, dönüş)
}

fn bağ_yolu(bağ: &SankeyYerleşikBağ) -> Yol {
    let mut yol = Yol::yeni();
    yol.taşı((bağ.x1, bağ.y1));
    yol.kübik((bağ.cpx1, bağ.cpy1), (bağ.cpx2, bağ.cpy2), (bağ.x2, bağ.y2));
    if bağ.yön == SankeyYönü::Dikey {
        yol.çiz((bağ.x2 + bağ.kalınlık, bağ.y2));
        yol.kübik(
            (bağ.cpx2 + bağ.kalınlık, bağ.cpy2),
            (bağ.cpx1 + bağ.kalınlık, bağ.cpy1),
            (bağ.x1 + bağ.kalınlık, bağ.y1),
        );
    } else {
        yol.çiz((bağ.x2, bağ.y2 + bağ.kalınlık));
        yol.kübik(
            (bağ.cpx2, bağ.cpy2 + bağ.kalınlık),
            (bağ.cpx1, bağ.cpy1 + bağ.kalınlık),
            (bağ.x1, bağ.y1 + bağ.kalınlık),
        );
    }
    yol.kapat();
    yol
}

fn bezier(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * a + 3.0 * u * u * t * b + 3.0 * u * t * t * c + t * t * t * d
}

fn bağ_orta_çizgisi(bağ: &SankeyYerleşikBağ) -> Vec<(f32, f32)> {
    (0..=24)
        .map(|sıra| {
            let t = sıra as f32 / 24.0;
            let mut x = bezier(bağ.x1, bağ.cpx1, bağ.cpx2, bağ.x2, t);
            let mut y = bezier(bağ.y1, bağ.cpy1, bağ.cpy2, bağ.y2, t);
            if bağ.yön == SankeyYönü::Dikey {
                x += bağ.kalınlık / 2.0;
            } else {
                y += bağ.kalınlık / 2.0;
            }
            (x, y)
        })
        .collect()
}

fn bağ_dolgusu(
    stil: &SankeyÇizgiStili, yön: SankeyYönü, kaynak: &Dolgu, hedef: &Dolgu
) -> Dolgu {
    let dolgu = match stil.renk.as_ref() {
        Some(SankeyKenarBoyası::Dolgu(dolgu)) => dolgu.clone(),
        Some(SankeyKenarBoyası::Kaynak) => kaynak.clone(),
        Some(SankeyKenarBoyası::Hedef) => hedef.clone(),
        Some(SankeyKenarBoyası::Gradyan) => Dolgu::doğrusal(
            0.0,
            0.0,
            (yön == SankeyYönü::Yatay) as u8 as f32,
            (yön == SankeyYönü::Dikey) as u8 as f32,
            vec![
                RenkDurağı::yeni(0.0, kaynak.temsilî()),
                RenkDurağı::yeni(1.0, hedef.temsilî()),
            ],
        ),
        None => Dolgu::Düz(tema::nötr_50()),
    };
    dolgu.opaklık(stil.opaklık.unwrap_or(1.0))
}

fn piksel_uzunluğunu_ölçekle(uzunluk: &mut Option<Uzunluk>, ölçek: f32) {
    if let Some(Uzunluk::Piksel(değer)) = uzunluk {
        *değer *= ölçek;
    }
}

fn yazı_stilini_ölçekle(yazı: &mut YazıStili, ölçek: f32) {
    if let Some(boyut) = &mut yazı.boyut {
        *boyut *= ölçek;
    }
    if let Some(satır_yüksekliği) = &mut yazı.satır_yüksekliği {
        *satır_yüksekliği *= ölçek;
    }
    if let Some(kalınlık) = &mut yazı.kenarlık_kalınlığı {
        *kalınlık *= ölçek;
    }
    if let Some(bulanıklık) = &mut yazı.metin_gölge_bulanıklığı {
        *bulanıklık *= ölçek;
    }
    if let Some((x, y)) = &mut yazı.metin_gölge_kayması {
        *x *= ölçek;
        *y *= ölçek;
    }
    if let Some(yarıçaplar) = &mut yazı.kenarlık_yarıçapları {
        for yarıçap in yarıçaplar {
            *yarıçap *= ölçek;
        }
    }
    if let Some(boşluklar) = &mut yazı.iç_boşluk {
        for boşluk in boşluklar {
            *boşluk *= ölçek;
        }
    }
    piksel_uzunluğunu_ölçekle(&mut yazı.genişlik, ölçek);
    if let Some(yükseklik) = &mut yazı.yükseklik {
        *yükseklik *= ölçek;
    }
}

fn etiketi_ölçekle(etiket: &mut Etiket, ölçek: f32) {
    etiket.kayma.0 *= ölçek;
    etiket.kayma.1 *= ölçek;
    etiket.uzaklık *= ölçek;
    if let Uzunluk::Piksel(değer) = &mut etiket.kenar_uzaklığı {
        *değer *= ölçek;
    }
    if let Some(pay) = &mut etiket.taşma_payını {
        *pay *= ölçek;
    }
    etiket.çizgi_uzaklığı *= ölçek;
    etiket.kenar_boşluğu *= ölçek;
    etiket.en_küçük_boşluk *= ölçek;
    yazı_stilini_ölçekle(&mut etiket.yazı, ölçek);
    for yazı in etiket.zengin.values_mut() {
        yazı_stilini_ölçekle(yazı, ölçek);
    }
}

fn etiket_yamasını_ölçekle(yama: &mut EtiketYaması, ölçek: f32) {
    if let Some((x, y)) = &mut yama.kayma {
        *x *= ölçek;
        *y *= ölçek;
    }
    if let Some(uzaklık) = &mut yama.uzaklık {
        *uzaklık *= ölçek;
    }
    if let Some(Uzunluk::Piksel(değer)) = &mut yama.kenar_uzaklığı {
        *değer *= ölçek;
    }
    for değer in [
        &mut yama.taşma_payını,
        &mut yama.çizgi_uzaklığı,
        &mut yama.kenar_boşluğu,
        &mut yama.en_küçük_boşluk,
    ]
    .into_iter()
    .flatten()
    {
        *değer *= ölçek;
    }
    if let Some(yazı) = &mut yama.yazı {
        yazı_stilini_ölçekle(yazı, ölçek);
    }
    if let Some(zengin) = &mut yama.zengin {
        for yazı in zengin.values_mut() {
            yazı_stilini_ölçekle(yazı, ölçek);
        }
    }
}

fn öğe_stilini_ölçekle(stil: &mut SankeyÖğeStili, ölçek: f32) {
    if let Some(kalınlık) = &mut stil.kenarlık_kalınlığı {
        *kalınlık *= ölçek;
    }
    if let Some(yarıçaplar) = &mut stil.kenarlık_yarıçapı {
        for yarıçap in yarıçaplar {
            *yarıçap *= ölçek;
        }
    }
    if let Some(bulanıklık) = &mut stil.gölge_bulanıklığı {
        *bulanıklık *= ölçek;
    }
    if let Some((x, y)) = &mut stil.gölge_kayması {
        *x *= ölçek;
        *y *= ölçek;
    }
}

fn çizgi_stilini_ölçekle(stil: &mut SankeyÇizgiStili, ölçek: f32) {
    if let Some(kalınlık) = &mut stil.kalınlık {
        *kalınlık *= ölçek;
    }
    if let Some(bulanıklık) = &mut stil.gölge_bulanıklığı {
        *bulanıklık *= ölçek;
    }
    if let Some((x, y)) = &mut stil.gölge_kayması {
        *x *= ölçek;
        *y *= ölçek;
    }
}

fn durumu_ölçekle(durum: &mut SankeyDurumu, ölçek: f32) {
    if let Some(stil) = &mut durum.öğe_stili {
        öğe_stilini_ölçekle(stil, ölçek);
    }
    if let Some(stil) = &mut durum.çizgi_stili {
        çizgi_stilini_ölçekle(stil, ölçek);
    }
    if let Some(etiket) = &mut durum.etiket {
        etiket_yamasını_ölçekle(etiket, ölçek);
    }
    if let Some(etiket) = &mut durum.kenar_etiketi {
        etiket_yamasını_ölçekle(etiket, ölçek);
    }
}

fn yerleşimi_dönüştür(
    yerleşim: &mut SankeyYerleşimi, dönüşüm: AfinMatris, ölçek: f32
) {
    if !dönüşüm.sonlu_mu() || !ölçek.is_finite() || ölçek <= 0.0 {
        return;
    }
    for düğüm in &mut yerleşim.düğümler {
        let (x, y) = dönüşüm.noktayı_dönüştür((düğüm.alan.x, düğüm.alan.y));
        düğüm.alan = Dikdörtgen::yeni(
            x,
            y,
            düğüm.alan.genişlik * ölçek,
            düğüm.alan.yükseklik * ölçek,
        );
        düğüm.etiket_konumu = dönüşüm.noktayı_dönüştür(düğüm.etiket_konumu);
        öğe_stilini_ölçekle(&mut düğüm.öğe_stili, ölçek);
        etiketi_ölçekle(&mut düğüm.etiket, ölçek);
        durumu_ölçekle(&mut düğüm.vurgu, ölçek);
        durumu_ölçekle(&mut düğüm.bulanık, ölçek);
        durumu_ölçekle(&mut düğüm.seçili, ölçek);
    }
    for bağ in &mut yerleşim.bağlar {
        (bağ.x1, bağ.y1) = dönüşüm.noktayı_dönüştür((bağ.x1, bağ.y1));
        (bağ.x2, bağ.y2) = dönüşüm.noktayı_dönüştür((bağ.x2, bağ.y2));
        (bağ.cpx1, bağ.cpy1) = dönüşüm.noktayı_dönüştür((bağ.cpx1, bağ.cpy1));
        (bağ.cpx2, bağ.cpy2) = dönüşüm.noktayı_dönüştür((bağ.cpx2, bağ.cpy2));
        bağ.etiket_konumu = dönüşüm.noktayı_dönüştür(bağ.etiket_konumu);
        bağ.kalınlık *= ölçek;
        çizgi_stilini_ölçekle(&mut bağ.çizgi_stili, ölçek);
        etiketi_ölçekle(&mut bağ.kenar_etiketi, ölçek);
        durumu_ölçekle(&mut bağ.vurgu, ölçek);
        durumu_ölçekle(&mut bağ.bulanık, ölçek);
        durumu_ölçekle(&mut bağ.seçili, ölçek);
    }
}

/// Model `center`/`zoom` dönüşümünün üstüne pencere etkileşiminden gelen
/// geçici pan/zoom görünümünü uygular. ECharts'ın View koordinat sistemi gibi
/// ölçek, seri alanının merkezinde tutulur; kayma ekran pikselidir.
pub fn sankey_geçici_görünümünü_uygula(
    yerleşim: &mut SankeyYerleşimi,
    görünüm: (f32, f32, f32),
    ölçek_sınırı: (f32, f32),
) {
    let en_küçük = ölçek_sınırı.0.max(0.01);
    let en_büyük = ölçek_sınırı.1.max(en_küçük);
    let ölçek = if görünüm.2.is_finite() {
        görünüm.2.clamp(en_küçük, en_büyük)
    } else {
        1.0_f32.clamp(en_küçük, en_büyük)
    };
    let dx = if görünüm.0.is_finite() {
        görünüm.0
    } else {
        0.0
    };
    let dy = if görünüm.1.is_finite() {
        görünüm.1
    } else {
        0.0
    };
    if dx == 0.0 && dy == 0.0 && (ölçek - 1.0).abs() <= f32::EPSILON {
        return;
    }
    let merkez = yerleşim.alan.merkez();
    let dönüşüm = AfinMatris::ötele(merkez.0 + dx, merkez.1 + dy)
        .çarp(AfinMatris::ölçekle(ölçek, ölçek))
        .çarp(AfinMatris::ötele(-merkez.0, -merkez.1));
    yerleşimi_dönüştür(yerleşim, dönüşüm, ölçek);
}

/// Resmî Sankey yerleşimini ve normal görünüm stilini üretir.
pub fn sankey_yerleşimi(
    seri: &SankeySerisi,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
) -> Result<SankeyYerleşimi, SankeyHatası> {
    let alan = sankey_alanı(seri, tuval);
    let (mut düğümler, mut bağlar) = grafiği_kur(seri)?;
    if düğümler.is_empty() {
        return Ok(SankeyYerleşimi {
            alan,
            düğümler: Vec::new(),
            bağlar: Vec::new(),
        });
    }
    düğüm_katmanlarını_hesapla(
        &mut düğümler,
        &bağlar,
        f64::from(seri.düğüm_genişliği),
        f64::from(alan.genişlik),
        f64::from(alan.yükseklik),
        seri.yön,
        seri.düğüm_hizası,
    )?;
    let sıfır_var = düğümler.iter().any(|düğüm| düğüm.değer == 0.0);
    düğüm_derinliklerini_hesapla(
        &mut düğümler,
        &mut bağlar,
        f64::from(seri.düğüm_boşluğu),
        f64::from(alan.genişlik),
        f64::from(alan.yükseklik),
        if sıfır_var {
            0
        } else {
            seri.yerleşim_yinelemesi
        },
        seri.yön,
        seri.sıralama,
    );
    bağ_derinliklerini_hesapla(&mut düğümler, &mut bağlar, seri.yön);

    let renkler = if seri.renkler.is_empty() {
        (0..9).map(palet).collect::<Vec<_>>()
    } else {
        seri.renkler.clone()
    };
    let en_az = düğümler
        .iter()
        .map(|düğüm| düğüm.değer)
        .fold(f64::INFINITY, f64::min);
    let en_çok = düğümler
        .iter()
        .map(|düğüm| düğüm.değer)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut yerleşik_düğümler = Vec::with_capacity(düğümler.len());
    for (sıra, düğüm) in düğümler.iter().enumerate() {
        let seviye = seri
            .seviyeler
            .iter()
            .find(|seviye| seviye.derinlik == düğüm.derinlik);
        let mut öğe_stili = seri.öğe_stili.clone();
        if let Some(yama) = seviye.and_then(|seviye| seviye.öğe_stili.as_ref()) {
            öğe_stili = öğe_stili_yama_uygula(&öğe_stili, yama);
        }
        if let Some(yama) = &düğüm.kaynak.öğe_stili {
            öğe_stili = öğe_stili_yama_uygula(&öğe_stili, yama);
        }
        let eşlenen = renk_eşle(düğüm.değer, en_az, en_çok, &renkler);
        let renk = öğe_stili.renk.clone().unwrap_or(Dolgu::Düz(eşlenen));
        let mut etiket = seri.etiket.clone();
        if let Some(yama) = seviye.and_then(|seviye| seviye.etiket.as_ref()) {
            etiket = yama.uygula(&etiket);
        }
        if let Some(yama) = &düğüm.kaynak.etiket {
            etiket = yama.uygula(&etiket);
        }
        let yerel_x = düğüm
            .kaynak
            .yerel_x
            .map_or(düğüm.x as f32, |x| x * alan.genişlik);
        let yerel_y = düğüm
            .kaynak
            .yerel_y
            .map_or(düğüm.y as f32, |y| y * alan.yükseklik);
        let düğüm_alanı = Dikdörtgen::yeni(
            alan.x + yerel_x,
            alan.y + yerel_y,
            düğüm.dx as f32,
            düğüm.dy as f32,
        );
        let (etiket_konumu, etiket_yatay_hizası, etiket_dikey_hizası, etiket_dönüşü) =
            düğüm_etiket_geometrisi(
                düğüm_alanı,
                &etiket,
                if öğe_stili.kenarlık_rengi.is_some() {
                    öğe_stili.kenarlık_kalınlığı.unwrap_or(1.0)
                } else {
                    0.0
                },
            );
        let vurgu = durum_yama_uygula(&seri.vurgu, &düğüm.kaynak.vurgu);
        let bulanık = durum_yama_uygula(&seri.bulanık, &düğüm.kaynak.bulanık);
        let seçili = durum_yama_uygula(&seri.seçili, &düğüm.kaynak.seçili);
        yerleşik_düğümler.push(SankeyYerleşikDüğüm {
            veri_sırası: sıra,
            ad: düğüm.kaynak.ad.clone(),
            değer: düğüm.değer,
            derinlik: düğüm.derinlik,
            alan: düğüm_alanı,
            renk,
            öğe_stili,
            etiket_metni: etiket_metni(&etiket, düğüm.değer, &düğüm.kaynak.ad, seri.ad.as_deref()),
            etiket,
            etiket_konumu,
            etiket_yatay_hizası,
            etiket_dikey_hizası,
            etiket_dönüşü,
            vurgu,
            bulanık,
            seçili,
            gelen_bağlar: düğüm.gelen.clone(),
            giden_bağlar: düğüm.giden.clone(),
        });
    }

    let mut yerleşik_bağlar = Vec::with_capacity(bağlar.len());
    for (sıra, bağ) in bağlar.iter().enumerate() {
        let Some(kaynak) = düğümler.get(bağ.kaynak_sırası) else {
            continue;
        };
        let Some(hedef) = düğümler.get(bağ.hedef_sırası) else {
            continue;
        };
        let seviye = seri
            .seviyeler
            .iter()
            .find(|seviye| seviye.derinlik == kaynak.derinlik);
        let mut çizgi_stili = seri.çizgi_stili.clone();
        if let Some(yama) = seviye.and_then(|seviye| seviye.çizgi_stili.as_ref()) {
            çizgi_stili = çizgi_stili_yama_uygula(&çizgi_stili, yama);
        }
        if let Some(yama) = &bağ.kaynak.çizgi_stili {
            çizgi_stili = çizgi_stili_yama_uygula(&çizgi_stili, yama);
        }
        let eğrilik = çizgi_stili.eğrilik.unwrap_or(0.5);
        let kaynak_x = kaynak
            .kaynak
            .yerel_x
            .map_or(kaynak.x as f32, |x| x * alan.genişlik);
        let kaynak_y = kaynak
            .kaynak
            .yerel_y
            .map_or(kaynak.y as f32, |y| y * alan.yükseklik);
        let hedef_x = hedef
            .kaynak
            .yerel_x
            .map_or(hedef.x as f32, |x| x * alan.genişlik);
        let hedef_y = hedef
            .kaynak
            .yerel_y
            .map_or(hedef.y as f32, |y| y * alan.yükseklik);
        let (x1, y1, x2, y2, cpx1, cpy1, cpx2, cpy2) = if seri.yön == SankeyYönü::Dikey {
            let x1 = kaynak_x + bağ.sy as f32;
            let y1 = kaynak_y + kaynak.dy as f32;
            let x2 = hedef_x + bağ.ty as f32;
            let y2 = hedef_y;
            (
                x1,
                y1,
                x2,
                y2,
                x1,
                y1 * (1.0 - eğrilik) + y2 * eğrilik,
                x2,
                y1 * eğrilik + y2 * (1.0 - eğrilik),
            )
        } else {
            let x1 = kaynak_x + kaynak.dx as f32;
            let y1 = kaynak_y + bağ.sy as f32;
            let x2 = hedef_x;
            let y2 = hedef_y + bağ.ty as f32;
            (
                x1,
                y1,
                x2,
                y2,
                x1 * (1.0 - eğrilik) + x2 * eğrilik,
                y1,
                x1 * eğrilik + x2 * (1.0 - eğrilik),
                y2,
            )
        };
        let x1 = x1 + alan.x;
        let y1 = y1 + alan.y;
        let x2 = x2 + alan.x;
        let y2 = y2 + alan.y;
        let cpx1 = cpx1 + alan.x;
        let cpy1 = cpy1 + alan.y;
        let cpx2 = cpx2 + alan.x;
        let cpy2 = cpy2 + alan.y;
        let kalınlık = (bağ.dy as f32).max(1.0);
        let kaynak_rengi = yerleşik_düğümler
            .get(bağ.kaynak_sırası)
            .map(|düğüm| &düğüm.renk)
            .ok_or_else(|| {
                SankeyHatası(format!(
                    "Sankey bağı için yerleşik kaynak düğümü yok: {}",
                    bağ.kaynak_sırası
                ))
            })?;
        let hedef_rengi = yerleşik_düğümler
            .get(bağ.hedef_sırası)
            .map(|düğüm| &düğüm.renk)
            .ok_or_else(|| {
                SankeyHatası(format!(
                    "Sankey bağı için yerleşik hedef düğümü yok: {}",
                    bağ.hedef_sırası
                ))
            })?;
        let dolgu = bağ_dolgusu(&çizgi_stili, seri.yön, kaynak_rengi, hedef_rengi);
        let mut kenar_etiketi = seri.kenar_etiketi.clone();
        if let Some(yama) = &bağ.kaynak.kenar_etiketi {
            kenar_etiketi = yama.uygula(&kenar_etiketi);
        }
        let etiket_metni = kenar_etiketi.biçimleyici.as_ref().map_or_else(
            || sayı_metni(bağ.kaynak.değer),
            |biçimleyici| {
                biçimleyici.uygula_bağlamla(
                    bağ.kaynak.değer,
                    &sayı_metni(bağ.kaynak.değer),
                    seri.ad.as_deref().unwrap_or_default(),
                    &format!("{} -- {}", bağ.kaynak.kaynak, bağ.kaynak.hedef),
                )
            },
        );
        let mut etiket_x = bezier(x1, cpx1, cpx2, x2, 0.5);
        let mut etiket_y = bezier(y1, cpy1, cpy2, y2, 0.5);
        if seri.yön == SankeyYönü::Dikey {
            etiket_x += kalınlık / 2.0;
        } else {
            etiket_y += kalınlık / 2.0;
        }
        let vurgu = durum_yama_uygula(&seri.vurgu, &bağ.kaynak.vurgu);
        let bulanık = durum_yama_uygula(&seri.bulanık, &bağ.kaynak.bulanık);
        let seçili = durum_yama_uygula(&seri.seçili, &bağ.kaynak.seçili);
        yerleşik_bağlar.push(SankeyYerleşikBağ {
            veri_sırası: sıra,
            kaynak_sırası: bağ.kaynak_sırası,
            hedef_sırası: bağ.hedef_sırası,
            kaynak: bağ.kaynak.kaynak.clone(),
            hedef: bağ.kaynak.hedef.clone(),
            değer: bağ.kaynak.değer,
            x1,
            y1,
            x2,
            y2,
            cpx1,
            cpy1,
            cpx2,
            cpy2,
            kalınlık,
            yön: seri.yön,
            dolgu,
            çizgi_stili,
            kenar_etiketi,
            etiket_metni,
            etiket_konumu: (etiket_x, etiket_y),
            vurgu,
            bulanık,
            seçili,
        });
    }
    let mut yerleşim = SankeyYerleşimi {
        alan,
        düğümler: yerleşik_düğümler,
        bağlar: yerleşik_bağlar,
    };
    let en_küçük = seri.en_küçük_ölçek.max(0.01);
    let en_büyük = seri.en_büyük_ölçek.max(en_küçük);
    let ölçek = if seri.yakınlaştırma.is_finite() {
        seri.yakınlaştırma.clamp(en_küçük, en_büyük)
    } else {
        1.0_f32.clamp(en_küçük, en_büyük)
    };
    if seri.merkez.is_some() || (ölçek - 1.0).abs() > f32::EPSILON {
        let kaynak_merkezi = seri
            .merkez
            .map(|(x, y)| {
                (
                    alan.x + x.çöz(alan.genişlik),
                    alan.y + y.çöz(alan.yükseklik),
                )
            })
            .unwrap_or_else(|| alan.merkez());
        let hedef_merkezi = alan.merkez();
        let dönüşüm = AfinMatris::ötele(hedef_merkezi.0, hedef_merkezi.1)
            .çarp(AfinMatris::ölçekle(ölçek, ölçek))
            .çarp(AfinMatris::ötele(-kaynak_merkezi.0, -kaynak_merkezi.1));
        yerleşimi_dönüştür(&mut yerleşim, dönüşüm, ölçek);
    }
    Ok(yerleşim)
}

#[derive(Clone, Copy)]
enum Vurgulu {
    Düğüm(usize),
    Bağ(usize),
}

fn vurgulu_öğe(yerleşim: &SankeyYerleşimi, fare: Option<(f32, f32)>) -> Option<Vurgulu> {
    let fare = fare?;
    if let Some(düğüm) = yerleşim
        .düğümler
        .iter()
        .rev()
        .find(|düğüm| pozitif_alan(düğüm.alan).içeriyor_mu(fare))
    {
        return Some(Vurgulu::Düğüm(düğüm.veri_sırası));
    }
    yerleşim.bağlar.iter().rev().find_map(|bağ| {
        İsabetGeometrisi::ÇokluÇizgi {
            noktalar: bağ_orta_çizgisi(bağ),
            tolerans: (bağ.kalınlık / 2.0).max(2.0),
        }
        .içeriyor_mu(fare)
        .then_some(Vurgulu::Bağ(bağ.veri_sırası))
    })
}

/// zrender `Rect` negatif width/height değerlerini karşı köşeye doğru
/// boyar. Çok kalabalık resmi Sankey katmanlarında `nodeGap` toplam alanı
/// aşabildiği için layout bilinçli olarak negatif düğüm kalınlığı üretir.
fn pozitif_alan(alan: Dikdörtgen) -> Dikdörtgen {
    Dikdörtgen::yeni(
        alan.x + alan.genişlik.min(0.0),
        alan.y + alan.yükseklik.min(0.0),
        alan.genişlik.abs(),
        alan.yükseklik.abs(),
    )
}

fn odak_kümeleri(
    yerleşim: &SankeyYerleşimi,
    vurgulu: Vurgulu,
    odak: SankeyVurguOdağı,
) -> (HashSet<usize>, HashSet<usize>) {
    let mut düğümler = HashSet::new();
    let mut bağlar = HashSet::new();
    match vurgulu {
        Vurgulu::Düğüm(sıra) => {
            düğümler.insert(sıra);
            if matches!(odak, SankeyVurguOdağı::Komşuluk | SankeyVurguOdağı::Yörünge) {
                let mut kuyruk = VecDeque::from([sıra]);
                while let Some(düğüm_sırası) = kuyruk.pop_front() {
                    let Some(düğüm) = yerleşim.düğümler.get(düğüm_sırası) else {
                        continue;
                    };
                    for bağ_sırası in düğüm.gelen_bağlar.iter().chain(&düğüm.giden_bağlar)
                    {
                        let ilk = bağlar.insert(*bağ_sırası);
                        let Some(bağ) = yerleşim.bağlar.get(*bağ_sırası) else {
                            continue;
                        };
                        for komşu in [bağ.kaynak_sırası, bağ.hedef_sırası] {
                            if düğümler.insert(komşu) && odak == SankeyVurguOdağı::Yörünge && ilk
                            {
                                kuyruk.push_back(komşu);
                            }
                        }
                    }
                    if odak == SankeyVurguOdağı::Komşuluk {
                        break;
                    }
                }
            }
        }
        Vurgulu::Bağ(sıra) => {
            bağlar.insert(sıra);
            if let Some(bağ) = yerleşim.bağlar.get(sıra) {
                düğümler.insert(bağ.kaynak_sırası);
                düğümler.insert(bağ.hedef_sırası);
            }
        }
    }
    if odak == SankeyVurguOdağı::Seri {
        düğümler.extend(0..yerleşim.düğümler.len());
        bağlar.extend(0..yerleşim.bağlar.len());
    }
    (düğümler, bağlar)
}

#[allow(clippy::too_many_arguments)]
fn etiketi_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    metin: &str,
    etiket: &Etiket,
    konum: (f32, f32),
    yatay: YatayHiza,
    dikey: DikeyHiza,
    dönüş: f32,
    varsayılan_renk: Renk,
    opaklık: f32,
) {
    if !etiket.göster || metin.is_empty() {
        return;
    }
    let mut etiket = etiket.clone();
    etiket.yazı.opaklık = Some(etiket.yazı.opaklık.unwrap_or(1.0) * opaklık);
    zengin_etiketi_hizalı_yaz(
        çizici,
        metin,
        &etiket,
        konum,
        yatay,
        dikey,
        varsayılan_renk.opaklık(opaklık),
        -dönüş,
    );
}

/// Sankey'i çizer. Yerleşim hatası döngü/geçersiz bağ gibi tipli model
/// hatalarını sessizce atlamak yerine çağırana taşır.
#[allow(clippy::too_many_arguments)]
pub fn sankey_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &SankeySerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    görünüm: (f32, f32, f32),
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> Result<(), SankeyHatası> {
    let mut yerleşim = sankey_yerleşimi(seri, tuval, palet)?;
    sankey_geçici_görünümünü_uygula(
        &mut yerleşim,
        görünüm,
        (
            seri.en_küçük_ölçek / seri.yakınlaştırma.max(0.01),
            seri.en_büyük_ölçek / seri.yakınlaştırma.max(0.01),
        ),
    );
    let vurgulu = vurgulu_öğe(&yerleşim, fare);
    let odak = vurgulu.map_or(SankeyVurguOdağı::Yok, |öğe| match öğe {
        Vurgulu::Düğüm(sıra) => yerleşim
            .düğümler
            .get(sıra)
            .and_then(|düğüm| düğüm.vurgu.odak)
            .unwrap_or(SankeyVurguOdağı::Yok),
        Vurgulu::Bağ(sıra) => yerleşim
            .bağlar
            .get(sıra)
            .and_then(|bağ| bağ.vurgu.odak)
            .unwrap_or(SankeyVurguOdağı::Yok),
    });
    let (odak_düğümleri, odak_bağları) = vurgulu.map_or_else(
        || (HashSet::new(), HashSet::new()),
        |öğe| odak_kümeleri(&yerleşim, öğe, odak),
    );
    let ilerleme = ilerleme.clamp(0.0, 1.0);
    let kırpma = Dikdörtgen::yeni(
        yerleşim.alan.x - 10.0,
        yerleşim.alan.y - 10.0,
        (yerleşim.alan.genişlik + 20.0) * ilerleme,
        yerleşim.alan.yükseklik + 20.0,
    );
    let mut boya = |yüzey: &mut dyn ÇizimYüzeyi| {
        // ECharts grubu önce bütün bağları, sonra bütün düğümleri ekler.
        for bağ in &yerleşim.bağlar {
            let vurgulu_bu = matches!(vurgulu, Some(Vurgulu::Bağ(sıra)) if sıra == bağ.veri_sırası);
            let bulanık = odak != SankeyVurguOdağı::Yok && !odak_bağları.contains(&bağ.veri_sırası);
            let durum = if vurgulu_bu {
                Some(&bağ.vurgu)
            } else if bulanık {
                Some(&bağ.bulanık)
            } else {
                None
            };
            let stil = durum
                .and_then(|durum| durum.çizgi_stili.as_ref())
                .map_or_else(
                    || bağ.çizgi_stili.clone(),
                    |yama| çizgi_stili_yama_uygula(&bağ.çizgi_stili, yama),
                );
            let mut dolgu = bağ.dolgu.clone();
            if (stil.renk != bağ.çizgi_stili.renk || stil.opaklık != bağ.çizgi_stili.opaklık)
                && let (Some(kaynak), Some(hedef)) = (
                    yerleşim.düğümler.get(bağ.kaynak_sırası),
                    yerleşim.düğümler.get(bağ.hedef_sırası),
                )
            {
                dolgu = bağ_dolgusu(&stil, bağ.yön, &kaynak.renk, &hedef.renk);
            }
            if bulanık && durum.and_then(|durum| durum.çizgi_stili.as_ref()).is_none() {
                dolgu = dolgu.opaklık(0.1);
            }
            let yol = bağ_yolu(bağ);
            if let (Some(bulanıklık), Some(renk)) = (stil.gölge_bulanıklığı, stil.gölge_rengi)
                && bulanıklık > 0.0
            {
                yüzey.yol_gölgesi(
                    &yol,
                    renk.opaklık(stil.opaklık.unwrap_or(1.0)),
                    bulanıklık,
                    stil.gölge_kayması.unwrap_or((0.0, 0.0)),
                );
            }
            yüzey.yol_doldur(&yol, &dolgu);
            if stil.kalınlık.unwrap_or(0.0) > 0.0 {
                yüzey.yol_çiz(
                    &yol,
                    stil.kalınlık.unwrap_or(0.0),
                    dolgu.temsilî(),
                    stil.tür.unwrap_or(ÇizgiTürü::Düz),
                );
            }
            let etiket = durum
                .and_then(|durum| durum.kenar_etiketi.as_ref())
                .map_or_else(
                    || bağ.kenar_etiketi.clone(),
                    |yama| yama.uygula(&bağ.kenar_etiketi),
                );
            etiketi_çiz(
                yüzey,
                &bağ.etiket_metni,
                &etiket,
                bağ.etiket_konumu,
                YatayHiza::Orta,
                DikeyHiza::Orta,
                0.0,
                tema::birincil_metin(),
                if bulanık { 0.1 } else { 1.0 },
            );
            if !seri.sessiz {
                isabetler.push(İsabetBölgesi {
                    seri_sırası: genel_sıra,
                    veri_sırası: bağ.veri_sırası,
                    seri_adı: seri.ad.clone(),
                    ad: Some(format!("{} -- {}", bağ.kaynak, bağ.hedef)),
                    değer: Some(bağ.değer),
                    geometri: İsabetGeometrisi::ÇokluÇizgi {
                        noktalar: bağ_orta_çizgisi(bağ),
                        tolerans: (bağ.kalınlık / 2.0).max(2.0),
                    },
                });
            }
        }
        for düğüm in &yerleşim.düğümler {
            let vurgulu_bu =
                matches!(vurgulu, Some(Vurgulu::Düğüm(sıra)) if sıra == düğüm.veri_sırası);
            let bulanık =
                odak != SankeyVurguOdağı::Yok && !odak_düğümleri.contains(&düğüm.veri_sırası);
            let durum = if vurgulu_bu {
                Some(&düğüm.vurgu)
            } else if bulanık {
                Some(&düğüm.bulanık)
            } else {
                None
            };
            let stil = durum
                .and_then(|durum| durum.öğe_stili.as_ref())
                .map_or_else(
                    || düğüm.öğe_stili.clone(),
                    |yama| öğe_stili_yama_uygula(&düğüm.öğe_stili, yama),
                );
            let opaklık = stil.opaklık.unwrap_or(1.0)
                * if bulanık && durum.and_then(|durum| durum.öğe_stili.as_ref()).is_none() {
                    0.1
                } else {
                    1.0
                };
            let dolgu = stil.renk.as_ref().unwrap_or(&düğüm.renk).opaklık(opaklık);
            let yarıçap = stil.kenarlık_yarıçapı.unwrap_or([0.0; 4]);
            let boya_alanı = pozitif_alan(düğüm.alan);
            if let (Some(bulanıklık), Some(renk)) = (stil.gölge_bulanıklığı, stil.gölge_rengi)
                && bulanıklık > 0.0
            {
                let yol = yuvarlak_dikdörtgen_yolu(boya_alanı, yarıçap);
                yüzey.yol_gölgesi(
                    &yol,
                    renk.opaklık(opaklık),
                    bulanıklık,
                    stil.gölge_kayması.unwrap_or((0.0, 0.0)),
                );
            }
            let kenarlık = stil
                .kenarlık_rengi
                .map(|renk| {
                    (
                        stil.kenarlık_kalınlığı.unwrap_or(1.0),
                        renk.opaklık(opaklık),
                    )
                })
                .filter(|(kalınlık, _)| *kalınlık > 0.0);
            if stil.kenarlık_türü.unwrap_or(ÇizgiTürü::Düz) == ÇizgiTürü::Düz {
                yüzey.dikdörtgen(boya_alanı, &dolgu, yarıçap, kenarlık);
            } else {
                let yol = yuvarlak_dikdörtgen_yolu(boya_alanı, yarıçap);
                yüzey.yol_doldur(&yol, &dolgu);
                if let Some((kalınlık, renk)) = kenarlık {
                    yüzey.yol_çiz(
                        &yol,
                        kalınlık,
                        renk,
                        stil.kenarlık_türü.unwrap_or(ÇizgiTürü::Düz),
                    );
                }
            }
            let etiket = durum
                .and_then(|durum| durum.etiket.as_ref())
                .map_or_else(|| düğüm.etiket.clone(), |yama| yama.uygula(&düğüm.etiket));
            let varsayılan_renk = if matches!(
                etiket.konum,
                EtiketKonumu::İç
                    | EtiketKonumu::İçÜst
                    | EtiketKonumu::İçAlt
                    | EtiketKonumu::İçSol
                    | EtiketKonumu::İçSağ
                    | EtiketKonumu::İçSolÜst
                    | EtiketKonumu::İçSağÜst
                    | EtiketKonumu::İçSolAlt
                    | EtiketKonumu::İçSağAlt
            ) {
                dolgu.temsilî().zrender_iç_etiket_stili(tema::koyu_mu()).0
            } else {
                tema::birincil_metin()
            };
            etiketi_çiz(
                yüzey,
                &düğüm.etiket_metni,
                &etiket,
                düğüm.etiket_konumu,
                düğüm.etiket_yatay_hizası,
                düğüm.etiket_dikey_hizası,
                düğüm.etiket_dönüşü,
                varsayılan_renk,
                if bulanık { 0.1 } else { 1.0 },
            );
            if !seri.sessiz {
                isabetler.push(İsabetBölgesi {
                    seri_sırası: genel_sıra,
                    veri_sırası: düğüm.veri_sırası,
                    seri_adı: seri.ad.clone(),
                    ad: Some(düğüm.ad.clone()),
                    değer: Some(düğüm.değer),
                    geometri: İsabetGeometrisi::Dikdörtgen(boya_alanı),
                });
            }
        }
    };
    if ilerleme < 1.0 {
        çizici.kırpılı(kırpma, &mut boya);
    } else {
        boya(çizici);
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing)]
mod testler {
    use super::*;

    fn basit() -> SankeySerisi {
        SankeySerisi::yeni()
            .düğümler(["a", "b", "a1", "a2", "b1", "c"])
            .bağlar([
                ("a", "a1", 5.0),
                ("a", "a2", 3.0),
                ("b", "b1", 8.0),
                ("a", "b1", 3.0),
                ("b1", "a1", 1.0),
                ("b1", "c", 2.0),
            ])
    }

    #[test]
    fn resmi_dag_deger_katman_ve_bag_kalinligini_korur() {
        let yerleşim = sankey_yerleşimi(
            &basit(),
            Dikdörtgen::yeni(0.0, 0.0, 600.0, 450.0),
            &tema::palet_rengi,
        )
        .expect("basit Sankey yerleşmeli");
        assert_eq!(yerleşim.düğümler.len(), 6);
        assert_eq!(yerleşim.bağlar.len(), 6);
        assert_eq!(yerleşim.düğümler[0].değer, 11.0);
        // b1'in gelen toplamı b(8) + a(3) = 11'dir; Sankey düğüm değeri
        // max(gelen, giden, açık değer) kuralıyla çözülür.
        assert_eq!(yerleşim.düğümler[4].değer, 11.0);
        assert_eq!(yerleşim.düğümler[5].derinlik, 2);
        assert!(yerleşim.bağlar[0].kalınlık > yerleşim.bağlar[1].kalınlık);
    }

    #[test]
    fn dikey_yon_ve_sag_hiza_eksenleri_degistirir() {
        let seri = basit()
            .yön(SankeyYönü::Dikey)
            .düğüm_hizası(SankeyDüğümHizası::Sağ);
        let yerleşim = sankey_yerleşimi(
            &seri,
            Dikdörtgen::yeni(0.0, 0.0, 600.0, 450.0),
            &tema::palet_rengi,
        )
        .expect("dikey Sankey yerleşmeli");
        assert!(yerleşim.düğümler[0].alan.genişlik > yerleşim.düğümler[0].alan.yükseklik);
        assert_eq!(yerleşim.bağlar[0].yön, SankeyYönü::Dikey);
    }

    #[test]
    fn model_ve_gecici_gorunum_donusumleri_ayni_merkezi_korur() {
        let tuval = Dikdörtgen::yeni(0.0, 0.0, 600.0, 450.0);
        let taban =
            sankey_yerleşimi(&basit(), tuval, &tema::palet_rengi).expect("taban Sankey yerleşmeli");
        let yakınlaştırılmış_seri = basit()
            .merkez(Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(50.0))
            .yakınlaştırma(2.0);
        let mut yakınlaştırılmış =
            sankey_yerleşimi(&yakınlaştırılmış_seri, tuval, &tema::palet_rengi)
                .expect("yakınlaştırılmış Sankey yerleşmeli");
        let merkez = taban.alan.merkez();
        let ilk = &taban.düğümler[0];
        let model = &yakınlaştırılmış.düğümler[0];
        assert!((model.alan.x - (merkez.0 + 2.0 * (ilk.alan.x - merkez.0))).abs() < 1e-3);
        assert!((model.alan.y - (merkez.1 + 2.0 * (ilk.alan.y - merkez.1))).abs() < 1e-3);
        assert!((model.alan.genişlik - ilk.alan.genişlik * 2.0).abs() < 1e-3);

        let model_x = model.alan.x;
        let model_y = model.alan.y;
        let model_g = model.alan.genişlik;
        sankey_geçici_görünümünü_uygula(&mut yakınlaştırılmış, (10.0, -5.0, 1.5), (0.1, 4.0));
        let geçici = &yakınlaştırılmış.düğümler[0];
        assert!((geçici.alan.x - (merkez.0 + 10.0 + 1.5 * (model_x - merkez.0))).abs() < 1e-3);
        assert!((geçici.alan.y - (merkez.1 - 5.0 + 1.5 * (model_y - merkez.1))).abs() < 1e-3);
        assert!((geçici.alan.genişlik - model_g * 1.5).abs() < 1e-3);
    }

    #[test]
    fn etiket_tabani_negatif_alani_ve_yarim_kenarligi_hesaba_katar() {
        let etiket = Etiket::yeni().konum(EtiketKonumu::Sağ).uzaklık(5.0);
        let (konum, yatay, dikey, _) =
            düğüm_etiket_geometrisi(Dikdörtgen::yeni(10.0, 20.0, 20.0, -2.0), &etiket, 1.0);
        assert!((konum.0 - 35.5).abs() < 1e-6);
        assert!((konum.1 - 19.0).abs() < 1e-6);
        assert_eq!(yatay, YatayHiza::Sol);
        assert_eq!(dikey, DikeyHiza::Orta);
    }

    #[test]
    fn dongu_tipli_hata_verir() {
        let seri = SankeySerisi::yeni().bağlar([("a", "b", 1.0), ("b", "a", 1.0)]);
        let hata = sankey_yerleşimi(
            &seri,
            Dikdörtgen::yeni(0.0, 0.0, 600.0, 450.0),
            &tema::palet_rengi,
        )
        .expect_err("döngü reddedilmeli");
        assert!(hata.0.contains("DAG"));
    }
}
