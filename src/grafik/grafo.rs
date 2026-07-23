#![allow(clippy::indexing_slicing)]
// Düğüm/bağ indisleri `grafı_kur` içinde bir kez doğrulanır; yerleşim ve
// boyama boyunca aynı kapalı indis uzayı kullanılır.

//! Graph/Grafo yerleşimi ve görünümü.
//!
//! Kuvvet çekirdeği kilitli ECharts `forceHelper.ts`; düz ve dairesel
//! yerleşimler `simpleLayoutHelper.ts` ile `circularLayoutHelper.ts`
//! hesap sırasının Rust karşılığıdır. Rasterdan bağımsız kanıt için düğüm,
//! bağ, uç sembolü ve etiket geometrisi dışa açılır.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::sembol_stilli_dönüşümlü_çiz;
use crate::koordinat::{Dikdörtgen, MatrisYerleşimi, TakvimYerleşimi};
use crate::model::grafo::{
    GrafoBağı, GrafoDurumu, GrafoDüğümü, GrafoEnBoyDikeyHizası, GrafoEnBoyKoruma,
    GrafoEnBoyYatayHizası, GrafoKategoriSeçimi, GrafoKenarBoyası, GrafoOtomatikEğrilik,
    GrafoSerisi, GrafoUcu, GrafoYerleşimi, GrafoÇizgiStili, GrafoÖğeStili,
};
use crate::model::seri::Sembol;
use crate::model::stil::{
    Etiket, EtiketDöndürme, EtiketKonumu, YazıDikeyHizası, YazıYatayHizası, ÇizgiTürü,
};
use crate::renk::{Dolgu, Renk};
use crate::tema;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GrafoHatası(pub String);

impl fmt::Display for GrafoHatası {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for GrafoHatası {}

#[derive(Clone, Debug)]
pub struct GrafoYerleşikDüğüm {
    pub veri_sırası: usize,
    pub kimlik: String,
    pub ad: String,
    pub değer: Option<f64>,
    pub konum: (f32, f32),
    pub ham_konum: (f32, f32),
    pub sembol: Sembol,
    pub boyut: f32,
    pub renk: Dolgu,
    pub öğe_stili: GrafoÖğeStili,
    pub etiket: Etiket,
    pub etiket_metni: String,
    pub etiket_konumu: (f32, f32),
    pub etiket_dönüşü: f32,
    pub etiket_yatay_hizası: YatayHiza,
    pub etiket_dikey_hizası: DikeyHiza,
    pub etiket_gizli: bool,
    pub kategori_sırası: Option<usize>,
    pub sabit: bool,
    pub sürüklenebilir: bool,
    pub vurgu: GrafoDurumu,
    pub bulanık: GrafoDurumu,
    pub seçili: GrafoDurumu,
    pub başlangıçta_seçili: bool,
    pub komşu_düğümler: Vec<usize>,
    pub komşu_bağlar: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct GrafoYerleşikBağ {
    pub veri_sırası: usize,
    pub kaynak_sırası: usize,
    pub hedef_sırası: usize,
    pub kaynak: String,
    pub hedef: String,
    pub değer: Option<f64>,
    pub başlangıç: (f32, f32),
    pub bitiş: (f32, f32),
    pub kontrol: Option<(f32, f32)>,
    pub kaynak_sembolü: Sembol,
    pub hedef_sembolü: Sembol,
    pub kaynak_sembol_boyutu: f32,
    pub hedef_sembol_boyutu: f32,
    pub çizgi_stili: GrafoÇizgiStili,
    pub renk: Renk,
    pub etiket: Etiket,
    pub etiket_metni: String,
    pub etiket_konumu: (f32, f32),
    pub etiket_dönüşü: f32,
    pub vurgu: GrafoDurumu,
    pub bulanık: GrafoDurumu,
    pub seçili: GrafoDurumu,
    pub kuvvet_yerleşimini_yoksay: bool,
}

#[derive(Clone, Debug)]
pub struct GrafoYerleşimSonucu {
    pub veri_alanı: Dikdörtgen,
    pub görünüm_alanı: Dikdörtgen,
    pub düğüm_ölçeği: f32,
    pub düğümler: Vec<GrafoYerleşikDüğüm>,
    pub bağlar: Vec<GrafoYerleşikBağ>,
}

type ÇözülmüşBağ = (GrafoBağı, usize, usize);
type KırpılmışBağ = ((f32, f32), (f32, f32), Option<(f32, f32)>);

/// Haricî koordinat sistemlerinin Graph düğümü veri→piksel köprüsü.
pub type GrafoKoordinatHaritası<'a> = dyn Fn(usize, &GrafoDüğümü) -> Option<(f32, f32)> + 'a;

fn öğe_stili_yama_uygula(taban: &GrafoÖğeStili, yama: &GrafoÖğeStili) -> GrafoÖğeStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk.clone_from(&yama.renk);
    }
    if yama.kenarlık_rengi.is_some() {
        sonuç.kenarlık_rengi = yama.kenarlık_rengi;
    }
    if yama.kenarlık_kalınlığı.is_some() {
        sonuç.kenarlık_kalınlığı = yama.kenarlık_kalınlığı;
    }
    if yama.kenarlık_türü.is_some() {
        sonuç.kenarlık_türü = yama.kenarlık_türü;
    }
    if yama.opaklık.is_some() {
        sonuç.opaklık = yama.opaklık;
    }
    if yama.gölge_bulanıklığı.is_some() {
        sonuç.gölge_bulanıklığı = yama.gölge_bulanıklığı;
    }
    if yama.gölge_rengi.is_some() {
        sonuç.gölge_rengi = yama.gölge_rengi;
    }
    if yama.gölge_kayması.is_some() {
        sonuç.gölge_kayması = yama.gölge_kayması;
    }
    sonuç
}

fn çizgi_stili_yama_uygula(taban: &GrafoÇizgiStili, yama: &GrafoÇizgiStili) -> GrafoÇizgiStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk.clone_from(&yama.renk);
    }
    if yama.kalınlık.is_some() {
        sonuç.kalınlık = yama.kalınlık;
    }
    if yama.tür.is_some() {
        sonuç.tür = yama.tür;
    }
    if yama.opaklık.is_some() {
        sonuç.opaklık = yama.opaklık;
    }
    if yama.eğrilik.is_some() {
        sonuç.eğrilik = yama.eğrilik;
    }
    if yama.gölge_bulanıklığı.is_some() {
        sonuç.gölge_bulanıklığı = yama.gölge_bulanıklığı;
    }
    if yama.gölge_rengi.is_some() {
        sonuç.gölge_rengi = yama.gölge_rengi;
    }
    if yama.gölge_kayması.is_some() {
        sonuç.gölge_kayması = yama.gölge_kayması;
    }
    sonuç
}

fn durum_yama_uygula(taban: &GrafoDurumu, yama: &GrafoDurumu) -> GrafoDurumu {
    let mut sonuç = taban.clone();
    if let Some(stil) = &yama.öğe_stili {
        sonuç.öğe_stili = Some(
            sonuç
                .öğe_stili
                .as_ref()
                .map_or_else(|| stil.clone(), |taban| öğe_stili_yama_uygula(taban, stil)),
        );
    }
    if let Some(stil) = &yama.çizgi_stili {
        sonuç.çizgi_stili = Some(sonuç.çizgi_stili.as_ref().map_or_else(
            || stil.clone(),
            |taban| çizgi_stili_yama_uygula(taban, stil),
        ));
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
    if yama.ölçek.is_some() {
        sonuç.ölçek = yama.ölçek;
    }
    if yama.devre_dışı.is_some() {
        sonuç.devre_dışı = yama.devre_dışı;
    }
    sonuç
}

fn grafı_kur(seri: &GrafoSerisi) -> Result<(Vec<usize>, Vec<ÇözülmüşBağ>), GrafoHatası> {
    let mut anahtarlar = HashMap::<String, usize>::new();
    for (sıra, düğüm) in seri.düğümler.iter().enumerate() {
        let kimlik = düğüm.kimlik.clone().unwrap_or_else(|| {
            if düğüm.ad.is_empty() {
                sıra.to_string()
            } else {
                düğüm.ad.clone()
            }
        });
        for anahtar in [
            Some(kimlik),
            (!düğüm.ad.is_empty()).then(|| düğüm.ad.clone()),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(eski) = anahtarlar.insert(anahtar.clone(), sıra)
                && eski != sıra
            {
                return Err(GrafoHatası(format!(
                    "yinelenmiş Graph düğüm anahtarı: {anahtar}"
                )));
            }
        }
    }
    let ham_bağlar = if seri.ayrıntılı_bağlar.is_empty() {
        seri.bağlar
            .iter()
            .map(|(kaynak, hedef)| GrafoBağı::yeni(kaynak.clone(), hedef.clone()))
            .collect::<Vec<_>>()
    } else {
        seri.ayrıntılı_bağlar.clone()
    };
    let uç_çöz = |uç: &GrafoUcu| -> Option<usize> {
        match uç {
            GrafoUcu::Sıra(sıra) => (*sıra < seri.düğümler.len()).then_some(*sıra),
            GrafoUcu::Kimlik(kimlik) => anahtarlar.get(kimlik).copied(),
        }
    };
    let mut bağlar = Vec::with_capacity(ham_bağlar.len());
    for (sıra, bağ) in ham_bağlar.into_iter().enumerate() {
        let kaynak = uç_çöz(&bağ.kaynak)
            .ok_or_else(|| GrafoHatası(format!("{sıra}. Graph bağının kaynağı bulunamadı")))?;
        let hedef = uç_çöz(&bağ.hedef)
            .ok_or_else(|| GrafoHatası(format!("{sıra}. Graph bağının hedefi bulunamadı")))?;
        bağlar.push((bağ, kaynak, hedef));
    }
    Ok(((0..seri.düğümler.len()).collect(), bağlar))
}

fn kategori_sırası(seri: &GrafoSerisi, düğüm: &GrafoDüğümü) -> Option<usize> {
    match düğüm.kategori_seçimi.as_ref() {
        Some(GrafoKategoriSeçimi::Sıra(sıra)) => Some(*sıra),
        Some(GrafoKategoriSeçimi::Ad(ad)) => seri
            .kategoriler
            .iter()
            .position(|kategori| &kategori.ad == ad),
        None => düğüm.kategori,
    }
}

fn uzunluk_çöz(değer: Option<crate::model::Uzunluk>, taban: f32) -> Option<f32> {
    değer.map(|değer| değer.çöz(taban))
}

/// ECharts `getLayoutRect({aspect})` karşılığı Graph görünüm kutusu.
pub fn grafo_görünüm_alanı(
    seri: &GrafoSerisi,
    tuval: Dikdörtgen,
    veri_en_boy_oranı: Option<f32>,
) -> Dikdörtgen {
    let mut genişlik = uzunluk_çöz(seri.genişlik, tuval.genişlik);
    let mut yükseklik = uzunluk_çöz(seri.yükseklik, tuval.yükseklik);
    let sol = uzunluk_çöz(seri.sol, tuval.genişlik);
    let üst = uzunluk_çöz(seri.üst, tuval.yükseklik);
    let sağ = uzunluk_çöz(seri.sağ, tuval.genişlik);
    let alt = uzunluk_çöz(seri.alt, tuval.yükseklik);

    if genişlik.is_none() && seri.sağ.is_some() {
        genişlik = Some((tuval.genişlik - sol.unwrap_or(0.0) - sağ.unwrap_or(0.0)).max(0.0));
    }
    if yükseklik.is_none() && seri.alt.is_some() {
        yükseklik = Some((tuval.yükseklik - üst.unwrap_or(0.0) - alt.unwrap_or(0.0)).max(0.0));
    }
    if let Some(en_boy) = veri_en_boy_oranı.filter(|değer| değer.is_finite() && *değer > 0.0) {
        match (genişlik, yükseklik) {
            (None, None) => {
                if en_boy > tuval.genişlik / tuval.yükseklik.max(f32::EPSILON) {
                    genişlik = Some(tuval.genişlik * 0.8);
                    yükseklik = genişlik.map(|değer| değer / en_boy);
                } else {
                    yükseklik = Some(tuval.yükseklik * 0.8);
                    genişlik = yükseklik.map(|değer| değer * en_boy);
                }
            }
            (Some(g), None) => yükseklik = Some(g / en_boy),
            (None, Some(y)) => genişlik = Some(y * en_boy),
            (Some(_), Some(_)) => {}
        }
    }
    let genişlik = genişlik.unwrap_or(tuval.genişlik);
    let yükseklik = yükseklik.unwrap_or(tuval.yükseklik);
    let x = if seri.genişlik.is_none() && seri.sağ.is_none() {
        // Resmî Graph varsayılanı `left: 'center'`.
        tuval.x + (tuval.genişlik - genişlik) / 2.0
    } else if let Some(sol) = sol {
        tuval.x + sol
    } else {
        tuval.sağ() - sağ.unwrap_or(0.0) - genişlik
    };
    let y = if seri.yükseklik.is_none() && seri.alt.is_none() {
        // Resmî Graph varsayılanı `top: 'center'`.
        tuval.y + (tuval.yükseklik - yükseklik) / 2.0
    } else if let Some(üst) = üst {
        tuval.y + üst
    } else {
        tuval.alt() - alt.unwrap_or(0.0) - yükseklik
    };
    let mut alan = Dikdörtgen::yeni(x, y, genişlik.max(0.0), yükseklik.max(0.0));
    if seri.en_boy_koruma != GrafoEnBoyKoruma::Kapalı
        && let Some(en_boy) = veri_en_boy_oranı.filter(|değer| değer.is_finite() && *değer > 0.0)
        && alan.genişlik > 0.0
        && alan.yükseklik > 0.0
    {
        let gerçek = alan.genişlik / alan.yükseklik;
        let kapla = seri.en_boy_koruma == GrafoEnBoyKoruma::Kapla;
        if (gerçek > en_boy && !kapla) || (gerçek < en_boy && kapla) {
            let yeni = alan.yükseklik * en_boy;
            alan.x += match seri.en_boy_yatay_hizası {
                GrafoEnBoyYatayHizası::Sol => 0.0,
                GrafoEnBoyYatayHizası::Orta => (alan.genişlik - yeni) / 2.0,
                GrafoEnBoyYatayHizası::Sağ => alan.genişlik - yeni,
            };
            alan.genişlik = yeni;
        } else if (gerçek - en_boy).abs() > 1e-9 {
            let yeni = alan.genişlik / en_boy;
            alan.y += match seri.en_boy_dikey_hizası {
                GrafoEnBoyDikeyHizası::Üst => 0.0,
                GrafoEnBoyDikeyHizası::Orta => (alan.yükseklik - yeni) / 2.0,
                GrafoEnBoyDikeyHizası::Alt => alan.yükseklik - yeni,
            };
            alan.yükseklik = yeni;
        }
    }
    alan
}

fn sonlu_veri_alanı(düğümler: &[GrafoDüğümü]) -> Option<Dikdörtgen> {
    let mut en_az_x = f32::INFINITY;
    let mut en_çok_x = f32::NEG_INFINITY;
    let mut en_az_y = f32::INFINITY;
    let mut en_çok_y = f32::NEG_INFINITY;
    for düğüm in düğümler {
        let (Some(x), Some(y)) = (düğüm.x, düğüm.y) else {
            continue;
        };
        if x.is_finite() && y.is_finite() {
            en_az_x = en_az_x.min(x);
            en_çok_x = en_çok_x.max(x);
            en_az_y = en_az_y.min(y);
            en_çok_y = en_çok_y.max(y);
        }
    }
    if !en_az_x.is_finite() {
        return None;
    }
    if en_çok_x - en_az_x == 0.0 {
        en_az_x -= 1.0;
        en_çok_x += 1.0;
    }
    if en_çok_y - en_az_y == 0.0 {
        en_az_y -= 1.0;
        en_çok_y += 1.0;
    }
    Some(Dikdörtgen::yeni(
        en_az_x,
        en_az_y,
        en_çok_x - en_az_x,
        en_çok_y - en_az_y,
    ))
}

/// View tabanlı Graph'ın işaretçi/roam kutusunu option ve ham veri
/// en-boy oranıyla çözer.
pub fn grafo_etkileşim_alanı(seri: &GrafoSerisi, tuval: Dikdörtgen) -> Dikdörtgen {
    let en_boy = sonlu_veri_alanı(&seri.düğümler)
        .map(|alan| alan.genişlik / alan.yükseklik.max(f32::EPSILON));
    grafo_görünüm_alanı(seri, tuval, en_boy)
}

fn doğrusal_eşle(değer: f64, alan: [f64; 2], hedef: [f64; 2]) -> f64 {
    let alan_farkı = alan[1] - alan[0];
    let hedef_farkı = hedef[1] - hedef[0];
    if alan_farkı == 0.0 {
        return if hedef_farkı == 0.0 {
            hedef[0]
        } else {
            (hedef[0] + hedef[1]) / 2.0
        };
    }
    if değer == alan[0] {
        return hedef[0];
    }
    if değer == alan[1] {
        return hedef[1];
    }
    (değer - alan[0]) / alan_farkı * hedef_farkı + hedef[0]
}

#[derive(Clone, Copy)]
struct Mulberry32(u32);

impl Mulberry32 {
    fn yeni(tohum: u32) -> Self {
        Self(tohum)
    }

    fn sonraki(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6D2B_79F5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(1 | t);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        ((t ^ (t >> 14)) as f64) / 4_294_967_296.0
    }
}

#[derive(Clone, Debug)]
struct KuvvetDüğümü {
    p: [f64; 2],
    pp: [f64; 2],
    w: f64,
    rep: f64,
    sabit: bool,
}

#[derive(Clone, Copy, Debug)]
struct KuvvetBağı {
    kaynak: usize,
    hedef: usize,
    uzunluk: f64,
    yoksay: bool,
}

fn değer_kapsamı(değerler: impl Iterator<Item = Option<f64>>) -> [f64; 2] {
    let mut en_az = f64::INFINITY;
    let mut en_çok = f64::NEG_INFINITY;
    for değer in değerler.flatten().filter(|değer| değer.is_finite()) {
        en_az = en_az.min(değer);
        en_çok = en_çok.max(değer);
    }
    [en_az, en_çok]
}

fn vektör_uzunluğu(v: [f64; 2]) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

fn kuvvet_yerleşimi(
    seri: &GrafoSerisi,
    veri_alanı: Dikdörtgen,
    bağlar: &[ÇözülmüşBağ],
    konumlar: &mut [(f32, f32)],
) {
    if konumlar.is_empty() {
        return;
    }
    let düğüm_kapsamı = değer_kapsamı(seri.düğümler.iter().map(|düğüm| düğüm.değer));
    let bağ_kapsamı = değer_kapsamı(bağlar.iter().map(|(bağ, _, _)| bağ.değer));
    let itme = [seri.kuvvet.itme.0 as f64, seri.kuvvet.itme.1 as f64];
    // Büyük bağ değeri daha kısa kenardır.
    let kenar_uzunluğu = [
        seri.kuvvet.kenar_uzunluğu.1 as f64,
        seri.kuvvet.kenar_uzunluğu.0 as f64,
    ];
    let mut rastgele = Mulberry32::yeni(seri.rastgele_tohumu);
    let merkez = [
        (veri_alanı.x + veri_alanı.genişlik / 2.0) as f64,
        (veri_alanı.y + veri_alanı.yükseklik / 2.0) as f64,
    ];
    let mut düğümler = seri
        .düğümler
        .iter()
        .enumerate()
        .map(|(sıra, düğüm)| {
            let değer = düğüm.değer.unwrap_or(f64::NAN);
            let mut rep = doğrusal_eşle(değer, düğüm_kapsamı, itme);
            if rep.is_nan() {
                rep = (itme[0] + itme[1]) / 2.0;
            }
            let ham = konumlar[sıra];
            let p = if ham.0.is_finite() && ham.1.is_finite() {
                [ham.0 as f64, ham.1 as f64]
            } else {
                [
                    veri_alanı.genişlik as f64 * (rastgele.sonraki() - 0.5) + merkez[0],
                    veri_alanı.yükseklik as f64 * (rastgele.sonraki() - 0.5) + merkez[1],
                ]
            };
            KuvvetDüğümü {
                p,
                pp: p,
                w: rep,
                rep,
                sabit: düğüm.sabit,
            }
        })
        .collect::<Vec<_>>();
    let kuvvet_bağları = bağlar
        .iter()
        .map(|(bağ, kaynak, hedef)| {
            let değer = bağ.değer.unwrap_or(f64::NAN);
            let mut uzunluk = doğrusal_eşle(değer, bağ_kapsamı, kenar_uzunluğu);
            if uzunluk.is_nan() {
                uzunluk = (kenar_uzunluğu[0] + kenar_uzunluğu[1]) / 2.0;
            }
            KuvvetBağı {
                kaynak: *kaynak,
                hedef: *hedef,
                uzunluk,
                yoksay: bağ.kuvvet_yerleşimini_yoksay,
            }
        })
        .collect::<Vec<_>>();
    let başlangıç_konumları = düğümler.iter().map(|düğüm| düğüm.p).collect::<Vec<_>>();
    let mut sürtünme = seri.kuvvet.sürtünme as f64;
    let yerçekimi = seri.kuvvet.yerçekimi as f64;
    // `forceLayoutStage` bir ilk adım, GraphView ise durana dek devam eder.
    loop {
        for bağ in &kuvvet_bağları {
            if bağ.yoksay {
                continue;
            }
            let (kaynak, hedef) = (bağ.kaynak, bağ.hedef);
            let v = [
                düğümler[hedef].p[0] - düğümler[kaynak].p[0],
                düğümler[hedef].p[1] - düğümler[kaynak].p[1],
            ];
            let uzunluk = vektör_uzunluğu(v);
            let d = uzunluk - bağ.uzunluk;
            let toplam = düğümler[kaynak].w + düğümler[hedef].w;
            let mut ağırlık = düğümler[hedef].w / toplam;
            if ağırlık.is_nan() {
                ağırlık = 0.0;
            }
            let normal = if uzunluk > 0.0 {
                [v[0] / uzunluk, v[1] / uzunluk]
            } else {
                [0.0, 0.0]
            };
            if !düğümler[kaynak].sabit {
                düğümler[kaynak].p[0] += normal[0] * ağırlık * d * sürtünme;
                düğümler[kaynak].p[1] += normal[1] * ağırlık * d * sürtünme;
            }
            if !düğümler[hedef].sabit {
                düğümler[hedef].p[0] -= normal[0] * (1.0 - ağırlık) * d * sürtünme;
                düğümler[hedef].p[1] -= normal[1] * (1.0 - ağırlık) * d * sürtünme;
            }
        }
        for düğüm in &mut düğümler {
            if !düğüm.sabit {
                düğüm.p[0] += (merkez[0] - düğüm.p[0]) * yerçekimi * sürtünme;
                düğüm.p[1] += (merkez[1] - düğüm.p[1]) * yerçekimi * sürtünme;
            }
        }
        for i in 0..düğümler.len() {
            for j in (i + 1)..düğümler.len() {
                let mut v = [
                    düğümler[j].p[0] - düğümler[i].p[0],
                    düğümler[j].p[1] - düğümler[i].p[1],
                ];
                let mut uzaklık = vektör_uzunluğu(v);
                if uzaklık == 0.0 {
                    v = [rastgele.sonraki() - 0.5, rastgele.sonraki() - 0.5];
                    uzaklık = 1.0;
                }
                let çarpan = (düğümler[i].rep + düğümler[j].rep) / uzaklık / uzaklık;
                if !düğümler[i].sabit {
                    düğümler[i].pp[0] += v[0] * çarpan;
                    düğümler[i].pp[1] += v[1] * çarpan;
                }
                if !düğümler[j].sabit {
                    düğümler[j].pp[0] -= v[0] * çarpan;
                    düğümler[j].pp[1] -= v[1] * çarpan;
                }
            }
        }
        for (sıra, düğüm) in düğümler.iter_mut().enumerate() {
            if !düğüm.sabit {
                let v = [düğüm.p[0] - düğüm.pp[0], düğüm.p[1] - düğüm.pp[1]];
                düğüm.p[0] += v[0] * sürtünme;
                düğüm.p[1] += v[1] * sürtünme;
                if !düğüm.p[0].is_finite()
                    || !düğüm.p[1].is_finite()
                    || !(düğüm.p[0] as f32).is_finite()
                    || !(düğüm.p[1] as f32).is_finite()
                {
                    // Aşırı dar başlangıç kutusunda yüksek repulsion çift
                    // hassasiyetini taşırabilir. ECharts döngüsünün aşama
                    // sırasını koruyup yalnız geçersiz sayıyı aynı kararlı
                    // başlangıca geri al; dışarı NaN sızdırma.
                    düğüm.p = başlangıç_konumları[sıra];
                }
                düğüm.pp = düğüm.p;
            }
        }
        sürtünme *= 0.992;
        if sürtünme < 0.01 {
            break;
        }
    }
    for (hedef, düğüm) in konumlar.iter_mut().zip(düğümler) {
        *hedef = (düğüm.p[0] as f32, düğüm.p[1] as f32);
    }
}

fn dairesel_yerleşim(seri: &GrafoSerisi, veri_alanı: Dikdörtgen, konumlar: &mut [(f32, f32)]) {
    if konumlar.is_empty() {
        return;
    }
    let merkez = veri_alanı.merkez();
    let yarıçap = veri_alanı.genişlik.min(veri_alanı.yükseklik) / 2.0;
    let mut yarım_açılar = Vec::with_capacity(seri.düğümler.len());
    let mut toplam = 0.0f32;
    for düğüm in &seri.düğümler {
        let boyut = düğüm
            .boyut_çifti
            .map_or(düğüm.boyut, |[x, y]| (x + y) / 2.0)
            .max(0.0);
        let oran = boyut / 2.0 / yarıçap.max(f32::EPSILON);
        let yarım = if oran.abs() <= 1.0 {
            oran.asin()
        } else {
            std::f32::consts::FRAC_PI_2
        };
        yarım_açılar.push(yarım);
        toplam += yarım * 2.0;
    }
    let kalan_yarım = (std::f32::consts::TAU - toplam) / konumlar.len() as f32 / 2.0;
    let mut açı = 0.0;
    for (sıra, konum) in konumlar.iter_mut().enumerate() {
        let yarım = kalan_yarım + yarım_açılar[sıra];
        açı += yarım;
        *konum = (
            merkez.0 + yarıçap * açı.cos(),
            merkez.1 + yarıçap * açı.sin(),
        );
        açı += yarım;
    }
}

fn otomatik_eğrilik_listesi(ayar: &GrafoOtomatikEğrilik, en_az: usize) -> Vec<f32> {
    if let GrafoOtomatikEğrilik::Değerler(değerler) = ayar {
        return değerler.clone();
    }
    let mut uzunluk = match ayar {
        GrafoOtomatikEğrilik::Uzunluk(uzunluk) => *uzunluk,
        GrafoOtomatikEğrilik::Kapalı => 0,
        // Üstte erken dönülür; yine de yeni bir enum dalı/akış değişikliği
        // çalışma zamanında paniğe dönüşmesin.
        GrafoOtomatikEğrilik::Değerler(değerler) => değerler.len(),
    };
    uzunluk = uzunluk.max(en_az);
    let uzunluk = if uzunluk % 2 == 1 {
        uzunluk + 2
    } else {
        uzunluk + 3
    };
    (0..uzunluk)
        .map(|sıra| {
            let değer = if sıra % 2 == 1 { sıra + 1 } else { sıra };
            değer as f32 / 10.0 * if sıra % 2 == 1 { -1.0 } else { 1.0 }
        })
        .collect()
}

fn otomatik_eğrilikler(
    ayar: &GrafoOtomatikEğrilik, bağlar: &[ÇözülmüşBağ]
) -> Vec<Option<f32>> {
    if matches!(ayar, GrafoOtomatikEğrilik::Kapalı) {
        return vec![None; bağlar.len()];
    }
    let mut gruplar = HashMap::<(usize, usize), Vec<usize>>::new();
    for (sıra, (_, kaynak, hedef)) in bağlar.iter().enumerate() {
        gruplar.entry((*kaynak, *hedef)).or_default().push(sıra);
    }
    let en_çok = gruplar
        .iter()
        .map(|(&(kaynak, hedef), bağlar)| {
            bağlar.len() + gruplar.get(&(hedef, kaynak)).map_or(0, Vec::len)
        })
        .max()
        .unwrap_or(0);
    let liste = otomatik_eğrilik_listesi(ayar, en_çok);
    let dizi_mi = matches!(ayar, GrafoOtomatikEğrilik::Değerler(_));
    let mut sonuç = vec![None; bağlar.len()];
    for (&(kaynak, hedef), aynı_yön) in &gruplar {
        let ters = gruplar.get(&(hedef, kaynak)).cloned().unwrap_or_default();
        let toplam = aynı_yön.len() + ters.len();
        let düzeltme = usize::from(!dizi_mi && toplam % 2 == 0);
        let ileri = !ters.is_empty() && kaynak < hedef;
        for (yerel, &sıra) in aynı_yön.iter().enumerate() {
            let ön = if ileri { 0 } else { ters.len() };
            let liste_sırası = yerel + ön + düzeltme;
            if !liste.is_empty() {
                sonuç[sıra] = liste.get(liste_sırası % liste.len()).copied();
            }
        }
    }
    sonuç
}

fn kuadratik_nokta(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
    let u = 1.0 - t;
    (
        u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
        u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
    )
}

fn kuadratik_teğet(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
    (
        2.0 * ((1.0 - t) * (p1.0 - p0.0) + t * (p2.0 - p1.0)),
        2.0 * ((1.0 - t) * (p1.1 - p0.1) + t * (p2.1 - p1.1)),
    )
}

fn kuadratik_kesişim_t(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    merkez: (f32, f32),
    yarıçap: f32,
    ters: bool,
) -> f32 {
    let mut en_iyi = if ters { 0.9 } else { 0.1 };
    let mut fark = f32::INFINITY;
    for sıra in 1..=9 {
        let t = sıra as f32 / 10.0;
        let p = kuadratik_nokta(p0, p1, p2, t);
        let yeni = ((p.0 - merkez.0).powi(2) + (p.1 - merkez.1).powi(2) - yarıçap.powi(2)).abs();
        if yeni < fark {
            fark = yeni;
            en_iyi = t;
        }
    }
    let mut alt = if ters { 0.0 } else { en_iyi - 0.1 };
    let mut üst = if ters { en_iyi + 0.1 } else { 1.0 };
    alt = alt.clamp(0.0, 1.0);
    üst = üst.clamp(0.0, 1.0);
    for _ in 0..32 {
        let orta = (alt + üst) / 2.0;
        let p = kuadratik_nokta(p0, p1, p2, orta);
        let içeride = (p.0 - merkez.0).powi(2) + (p.1 - merkez.1).powi(2) < yarıçap.powi(2);
        if ters {
            if içeride { üst = orta } else { alt = orta }
        } else if içeride {
            alt = orta;
        } else {
            üst = orta;
        }
    }
    (alt + üst) / 2.0
}

fn bağı_kırp(
    başlangıç: (f32, f32),
    bitiş: (f32, f32),
    kontrol: Option<(f32, f32)>,
    kaynak_yarıçapı: Option<f32>,
    hedef_yarıçapı: Option<f32>,
) -> KırpılmışBağ {
    if let Some(kontrol) = kontrol {
        let mut t0 = 0.0;
        let mut t1 = 1.0;
        if let Some(yarıçap) = kaynak_yarıçapı {
            t0 = kuadratik_kesişim_t(başlangıç, kontrol, bitiş, başlangıç, yarıçap, false);
        }
        if let Some(yarıçap) = hedef_yarıçapı {
            t1 = kuadratik_kesişim_t(başlangıç, kontrol, bitiş, bitiş, yarıçap, true);
        }
        let yeni_baş = kuadratik_nokta(başlangıç, kontrol, bitiş, t0);
        let yeni_bitiş = kuadratik_nokta(başlangıç, kontrol, bitiş, t1);
        let orta_t = (t0 + t1) / 2.0;
        let orta = kuadratik_nokta(başlangıç, kontrol, bitiş, orta_t);
        // Aynı alt eğriyi tek kuadratikle tam olarak yeniden kur.
        let yeni_kontrol = (
            2.0 * orta.0 - (yeni_baş.0 + yeni_bitiş.0) / 2.0,
            2.0 * orta.1 - (yeni_baş.1 + yeni_bitiş.1) / 2.0,
        );
        (yeni_baş, yeni_bitiş, Some(yeni_kontrol))
    } else {
        let dx = bitiş.0 - başlangıç.0;
        let dy = bitiş.1 - başlangıç.1;
        let uzunluk = (dx * dx + dy * dy).sqrt();
        if uzunluk <= f32::EPSILON {
            return (başlangıç, bitiş, None);
        }
        let birim = (dx / uzunluk, dy / uzunluk);
        let başlangıç = kaynak_yarıçapı.map_or(başlangıç, |r| {
            (başlangıç.0 + birim.0 * r, başlangıç.1 + birim.1 * r)
        });
        let bitiş =
            hedef_yarıçapı.map_or(bitiş, |r| (bitiş.0 - birim.0 * r, bitiş.1 - birim.1 * r));
        (başlangıç, bitiş, None)
    }
}

fn etiket_metni(etiket: &Etiket, değer: Option<f64>, ad: &str, seri_adı: Option<&str>) -> String {
    etiket
        .biçimleyici
        .as_ref()
        .map(|biçimleyici| {
            biçimleyici.uygula_bağlamla(
                değer.unwrap_or_default(),
                ad,
                seri_adı.unwrap_or_default(),
                ad,
            )
        })
        .unwrap_or_else(|| ad.to_string())
}

fn grafo_ucu_metni(uç: &GrafoUcu) -> String {
    match uç {
        GrafoUcu::Sıra(sıra) => sıra.to_string(),
        GrafoUcu::Kimlik(kimlik) => kimlik.clone(),
    }
}

fn etiket_geometrisi(
    etiket: &Etiket,
    merkez: (f32, f32),
    boyut: f32,
    dairesel_dönüş: Option<(f32, f32)>,
) -> ((f32, f32), f32, YatayHiza, DikeyHiza) {
    if let Some((cx, cy)) = dairesel_dönüş {
        let mut radyan = (merkez.1 - cy).atan2(merkez.0 - cx);
        if radyan < 0.0 {
            radyan += std::f32::consts::TAU;
        }
        let sol = merkez.0 < cx;
        if sol {
            radyan -= std::f32::consts::PI;
        }
        let uzaklık = boyut / 2.0 + etiket.uzaklık;
        // GraphView `rotateNodeLabel`, etiketi sembol merkezinin çevresinde
        // döndürür. Dolayısıyla dünya çapası yatay bir ofset değil, düğümün
        // daire merkezinden dışarı uzanan yarıçap üzerindedir.
        let dx = merkez.0 - cx;
        let dy = merkez.1 - cy;
        let uzunluk = dx.hypot(dy).max(f32::EPSILON);
        return (
            (
                merkez.0 + dx / uzunluk * uzaklık,
                merkez.1 + dy / uzunluk * uzaklık,
            ),
            radyan,
            if sol { YatayHiza::Sağ } else { YatayHiza::Sol },
            DikeyHiza::Orta,
        );
    }
    let r = boyut / 2.0;
    let d = etiket.uzaklık;
    let (mut konum, doğal_yatay, doğal_dikey) = match etiket.konum {
        EtiketKonumu::Üst | EtiketKonumu::Dış => (
            (merkez.0, merkez.1 - r - d),
            YatayHiza::Orta,
            DikeyHiza::Alt,
        ),
        EtiketKonumu::Alt => (
            (merkez.0, merkez.1 + r + d),
            YatayHiza::Orta,
            DikeyHiza::Üst,
        ),
        EtiketKonumu::Sol => (
            (merkez.0 - r - d, merkez.1),
            YatayHiza::Sağ,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::Sağ => (
            (merkez.0 + r + d, merkez.1),
            YatayHiza::Sol,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::İçÜst => (
            (merkez.0, merkez.1 - r / 2.0),
            YatayHiza::Orta,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::İçAlt => (
            (merkez.0, merkez.1 + r / 2.0),
            YatayHiza::Orta,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::İçSol => (
            (merkez.0 - r / 2.0, merkez.1),
            YatayHiza::Orta,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::İçSağ => (
            (merkez.0 + r / 2.0, merkez.1),
            YatayHiza::Orta,
            DikeyHiza::Orta,
        ),
        _ => (merkez, YatayHiza::Orta, DikeyHiza::Orta),
    };
    konum.0 += etiket.kayma.0;
    konum.1 += etiket.kayma.1;
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
    let dönüş = match etiket.döndürme {
        EtiketDöndürme::Derece(değer) => -değer.to_radians(),
        _ => 0.0,
    };
    (konum, dönüş, yatay, dikey)
}

fn etiket_kutusu(düğüm: &GrafoYerleşikDüğüm) -> Dikdörtgen {
    let boyut = düğüm.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    // Canvas2D'nin varsayılan `12px sans-serif` yüzü Chromium/macOS'ta
    // Arial metriklerine çözülür. LabelManager `hideOverlap`, gerçek metin
    // sınırını kullandığından eş genişlikli `karakter * 0.56em` yaklaşımı
    // özellikle dar `i/l/t` harfli adlarda gereksiz gizlemeye yol açar.
    let genişlik = düğüm
        .etiket_metni
        .chars()
        .map(|karakter| {
            let on_iki_piksel = match karakter {
                'A' | 'B' | 'K' | 'R' | 'S' | 'X' | 'Y' => 8.0,
                'C' | 'D' | 'H' | 'N' | 'U' => 8.67,
                'E' => 8.0,
                'F' | 'T' | 'Z' => 7.33,
                'G' | 'O' | 'Q' => 9.33,
                'I' => 3.33,
                'J' | 'L' => 6.67,
                'M' => 10.0,
                'P' | 'V' => 8.0,
                'W' => 11.33,
                'a' | 'b' | 'd' | 'e' | 'g' | 'h' | 'n' | 'o' | 'p' | 'q' | 'u' | 'v' | 'x'
                | 'y' => 6.67,
                'c' | 's' | 'z' => 6.0,
                'f' | 't' => 3.33,
                'i' | 'j' | 'l' => 2.67,
                'k' => 6.0,
                'm' => 10.0,
                'r' => 4.0,
                'w' => 8.67,
                ' ' => 3.33,
                _ => 7.2,
            };
            on_iki_piksel * boyut / 12.0
        })
        .sum();
    let x = match düğüm.etiket_yatay_hizası {
        YatayHiza::Sol => düğüm.etiket_konumu.0,
        YatayHiza::Orta => düğüm.etiket_konumu.0 - genişlik / 2.0,
        YatayHiza::Sağ => düğüm.etiket_konumu.0 - genişlik,
    };
    let y = match düğüm.etiket_dikey_hizası {
        DikeyHiza::Üst => düğüm.etiket_konumu.1,
        DikeyHiza::Orta => düğüm.etiket_konumu.1 - boyut / 2.0,
        DikeyHiza::Alt => düğüm.etiket_konumu.1 - boyut,
    };
    Dikdörtgen::yeni(x, y, genişlik, boyut)
}

fn kutular_kesişir(a: Dikdörtgen, b: Dikdörtgen) -> bool {
    a.x < b.sağ() && a.sağ() > b.x && a.y < b.alt() && a.alt() > b.y
}

fn kategori_görünür(
    seri: &GrafoSerisi,
    sıra: Option<usize>,
    kapalı_kategoriler: &HashSet<String>,
) -> bool {
    sıra
        .and_then(|sıra| seri.kategoriler.get(sıra))
        .is_none_or(|kategori| !kapalı_kategoriler.contains(&kategori.ad))
}

/// Graph düğüm/bağ geometrisini kurar. `koordinat_haritası`, external
/// cartesian/polar/single/calendar/matrix veri→piksel dönüşümüdür.
#[allow(clippy::too_many_arguments)]
pub fn grafo_yerleşimi_kur(
    seri: &GrafoSerisi,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    görünüm: (f32, f32, f32),
    kaymalar: &[(usize, f32, f32)],
    koordinat_haritası: Option<&GrafoKoordinatHaritası<'_>>,
    kapalı_kategoriler: &HashSet<String>,
) -> Result<GrafoYerleşimSonucu, GrafoHatası> {
    let (_, bütün_bağlar) = grafı_kur(seri)?;
    let görünür_sıralar = seri
        .düğümler
        .iter()
        .enumerate()
        .filter_map(|(sıra, düğüm)| {
            kategori_görünür(seri, kategori_sırası(seri, düğüm), kapalı_kategoriler).then_some(sıra)
        })
        .collect::<Vec<_>>();
    let görünür_küme = görünür_sıralar.iter().copied().collect::<HashSet<_>>();
    let bağlar = bütün_bağlar
        .into_iter()
        .filter(|(_, kaynak, hedef)| görünür_küme.contains(kaynak) && görünür_küme.contains(hedef))
        .collect::<Vec<_>>();

    let açık_veri_alanı = sonlu_veri_alanı(&seri.düğümler);
    let en_boy = açık_veri_alanı.map(|alan| alan.genişlik / alan.yükseklik.max(f32::EPSILON));
    let görünüm_alanı = grafo_görünüm_alanı(seri, tuval, en_boy);
    let veri_alanı = açık_veri_alanı.unwrap_or(görünüm_alanı);
    let dış_koordinat = koordinat_haritası.is_some();
    let mut ham_konumlar = seri
        .düğümler
        .iter()
        .enumerate()
        .map(|(sıra, düğüm)| {
            koordinat_haritası
                .and_then(|harita| harita(sıra, düğüm))
                .or_else(|| düğüm.x.zip(düğüm.y))
                .unwrap_or((f32::NAN, f32::NAN))
        })
        .collect::<Vec<_>>();

    if seri.korunmuş_noktalar.is_none() && !dış_koordinat {
        match seri.yerleşim {
            GrafoYerleşimi::Yok => {}
            GrafoYerleşimi::Dairesel => dairesel_yerleşim(seri, veri_alanı, &mut ham_konumlar),
            GrafoYerleşimi::Kuvvet => {
                if matches!(
                    seri.kuvvet.başlangıç_yerleşimi,
                    Some(crate::model::grafo::GrafoKuvvetBaşlangıcı::Dairesel)
                ) {
                    dairesel_yerleşim(seri, veri_alanı, &mut ham_konumlar);
                }
                kuvvet_yerleşimi(seri, veri_alanı, &bağlar, &mut ham_konumlar);
            }
        }
    }

    let hamdan_tuvale = |p: (f32, f32)| -> (f32, f32) {
        if dış_koordinat {
            return p;
        }
        let sx = görünüm_alanı.genişlik / veri_alanı.genişlik.max(f32::EPSILON);
        let sy = görünüm_alanı.yükseklik / veri_alanı.yükseklik.max(f32::EPSILON);
        (
            görünüm_alanı.x + (p.0 - veri_alanı.x) * sx,
            görünüm_alanı.y + (p.1 - veri_alanı.y) * sy,
        )
    };
    let (kayma_x, kayma_y, etkileşim_ölçeği) = görünüm;
    let etkileşim_ölçeği = if etkileşim_ölçeği.is_finite() && etkileşim_ölçeği > 0.01 {
        etkileşim_ölçeği
    } else {
        1.0
    };
    let toplam_ölçek = (seri.yakınlaştırma * etkileşim_ölçeği).clamp(
        seri.en_küçük_yakınlaştırma.unwrap_or(0.000_001),
        seri.en_büyük_yakınlaştırma.unwrap_or(f32::MAX),
    );
    let görünüm_merkezi = görünüm_alanı.merkez();
    let gezinme_merkezi = seri.merkez.map_or(görünüm_merkezi, |(x, y)| {
        let veri_merkezi = (
            match x {
                crate::model::Uzunluk::Piksel(değer) => değer,
                crate::model::Uzunluk::Yüzde(yüzde) => {
                    veri_alanı.x + veri_alanı.genişlik * yüzde / 100.0
                }
            },
            match y {
                crate::model::Uzunluk::Piksel(değer) => değer,
                crate::model::Uzunluk::Yüzde(yüzde) => {
                    veri_alanı.y + veri_alanı.yükseklik * yüzde / 100.0
                }
            },
        );
        hamdan_tuvale(veri_merkezi)
    });
    let mut tuval_konumları = if let Some(korunmuş) = &seri.korunmuş_noktalar {
        (0..seri.düğümler.len())
            .map(|sıra| korunmuş.get(sıra).copied().unwrap_or((f32::NAN, f32::NAN)))
            .collect::<Vec<_>>()
    } else {
        ham_konumlar.iter().copied().map(hamdan_tuvale).collect()
    };
    if !dış_koordinat {
        for konum in &mut tuval_konumları {
            konum.0 = görünüm_merkezi.0 + (konum.0 - gezinme_merkezi.0) * toplam_ölçek + kayma_x;
            konum.1 = görünüm_merkezi.1 + (konum.1 - gezinme_merkezi.1) * toplam_ölçek + kayma_y;
        }
    }
    for &(sıra, dx, dy) in kaymalar {
        if let Some(konum) = tuval_konumları.get_mut(sıra) {
            konum.0 += dx;
            konum.1 += dy;
            if !dış_koordinat && seri.yerleşim == GrafoYerleşimi::Dairesel {
                let yarıçap = ((konum.0 - dx - görünüm_merkezi.0).powi(2)
                    + (konum.1 - dy - görünüm_merkezi.1).powi(2))
                .sqrt();
                let yön = (konum.0 - görünüm_merkezi.0, konum.1 - görünüm_merkezi.1);
                let uzunluk = (yön.0 * yön.0 + yön.1 * yön.1).sqrt();
                if uzunluk > f32::EPSILON {
                    konum.0 = görünüm_merkezi.0 + yön.0 / uzunluk * yarıçap;
                    konum.1 = görünüm_merkezi.1 + yön.1 / uzunluk * yarıçap;
                }
            }
        }
    }
    let düğüm_ölçeği = if dış_koordinat {
        1.0
    } else {
        ((toplam_ölçek - 1.0) * seri.düğüm_ölçek_oranı + 1.0).max(0.0)
    };

    let mut düğümler = Vec::with_capacity(görünür_sıralar.len());
    let mut eski_yeni = HashMap::new();
    for veri_sırası in görünür_sıralar {
        let kaynak = &seri.düğümler[veri_sırası];
        let kategori_sırası = kategori_sırası(seri, kaynak);
        let kategori = kategori_sırası.and_then(|sıra| seri.kategoriler.get(sıra));
        let mut stil = seri.grafo_öğe_stili.clone();
        if let Some(yama) = kategori.and_then(|kategori| kategori.öğe_stili.as_ref()) {
            stil = öğe_stili_yama_uygula(&stil, yama);
        }
        if let Some(yama) = &kaynak.öğe_stili {
            stil = öğe_stili_yama_uygula(&stil, yama);
        }
        let palet_rengi = palet(kategori_sırası.unwrap_or(0));
        let renk = stil
            .renk
            .clone()
            .unwrap_or(Dolgu::Düz(palet_rengi))
            .opaklık(stil.opaklık.unwrap_or(1.0));
        let sembol = kaynak
            .sembol
            .clone()
            .or_else(|| kategori.and_then(|kategori| kategori.sembol.clone()))
            .unwrap_or_else(|| seri.sembol.clone());
        let kalıtılan_boyut = kategori
            .and_then(|kategori| kategori.boyut)
            .unwrap_or(seri.sembol_boyutu);
        let boyut = kaynak
            .boyut_çifti
            .map(|[x, y]| (x + y) / 2.0)
            .or_else(|| kaynak.boyut_açık.then_some(kaynak.boyut))
            .unwrap_or(kalıtılan_boyut)
            .max(0.0)
            * düğüm_ölçeği;
        let mut etiket = kategori
            .and_then(|kategori| kategori.etiket.as_ref())
            .map_or_else(|| seri.etiket.clone(), |yama| yama.uygula(&seri.etiket));
        if let Some(yama) = &kaynak.etiket {
            etiket = yama.uygula(&etiket);
        }
        let konum = tuval_konumları[veri_sırası];
        let dairesel_dönüş = (seri.yerleşim == GrafoYerleşimi::Dairesel
            && seri.dairesel.etiketi_döndür)
            .then_some(görünüm_merkezi);
        // ZRender bağlı etiket konumunda Path sınırını kullanır; vuruşun
        // yarısı her iki yana taştığı için toplam sembol çapına borderWidth
        // eklenir. Özellikle `position: 'top'` etiketlerinde yarım piksel
        // farkı böyle kapanır.
        let etiket_boyutu = boyut + stil.kenarlık_kalınlığı.unwrap_or(0.0).max(0.0);
        let (etiket_konumu, etiket_dönüşü, yatay, dikey) =
            etiket_geometrisi(&etiket, konum, etiket_boyutu, dairesel_dönüş);
        let ad = if kaynak.ad.is_empty() {
            kaynak
                .kimlik
                .clone()
                .unwrap_or_else(|| veri_sırası.to_string())
        } else {
            kaynak.ad.clone()
        };
        let düğüm = GrafoYerleşikDüğüm {
            veri_sırası,
            kimlik: kaynak.kimlik.clone().unwrap_or_else(|| ad.clone()),
            ad: ad.clone(),
            değer: kaynak.değer,
            konum,
            ham_konum: ham_konumlar[veri_sırası],
            sembol,
            boyut,
            renk,
            öğe_stili: stil,
            etiket_metni: etiket_metni(&etiket, kaynak.değer, &ad, seri.ad.as_deref()),
            etiket,
            etiket_konumu,
            etiket_dönüşü,
            etiket_yatay_hizası: yatay,
            etiket_dikey_hizası: dikey,
            etiket_gizli: false,
            kategori_sırası,
            sabit: kaynak.sabit,
            sürüklenebilir: kaynak.sürüklenebilir.unwrap_or(seri.sürüklenebilir),
            vurgu: durum_yama_uygula(
                &durum_yama_uygula(
                    &seri.vurgu,
                    &kategori.map_or_else(GrafoDurumu::default, |k| k.vurgu.clone()),
                ),
                &kaynak.vurgu,
            ),
            bulanık: durum_yama_uygula(
                &durum_yama_uygula(
                    &seri.bulanık,
                    &kategori.map_or_else(GrafoDurumu::default, |k| k.bulanık.clone()),
                ),
                &kaynak.bulanık,
            ),
            seçili: durum_yama_uygula(
                &durum_yama_uygula(
                    &seri.seçili,
                    &kategori.map_or_else(GrafoDurumu::default, |k| k.seçili.clone()),
                ),
                &kaynak.seçili,
            ),
            başlangıçta_seçili: kaynak.başlangıçta_seçili,
            komşu_düğümler: Vec::new(),
            komşu_bağlar: Vec::new(),
        };
        eski_yeni.insert(veri_sırası, düğümler.len());
        düğümler.push(düğüm);
    }

    let otomatik = otomatik_eğrilikler(&seri.otomatik_eğrilik, &bağlar);
    let mut yerleşik_bağlar = Vec::with_capacity(bağlar.len());
    for (veri_sırası, (kaynak_bağ, kaynak, hedef)) in bağlar.into_iter().enumerate() {
        let (Some(&kaynak_yeni), Some(&hedef_yeni)) =
            (eski_yeni.get(&kaynak), eski_yeni.get(&hedef))
        else {
            continue;
        };
        let mut stil = seri.grafo_çizgi_stili.clone();
        if let Some(yama) = &kaynak_bağ.çizgi_stili {
            stil = çizgi_stili_yama_uygula(&stil, yama);
        }
        if kaynak_bağ
            .çizgi_stili
            .as_ref()
            .and_then(|stil| stil.eğrilik)
            .is_none()
            && let Some(eğrilik) = otomatik.get(veri_sırası).copied().flatten()
        {
            stil.eğrilik = Some(-eğrilik);
        }
        let p1 = düğümler[kaynak_yeni].konum;
        let p2 = düğümler[hedef_yeni].konum;
        let eğrilik = stil.eğrilik.unwrap_or(0.0);
        let mut kontrol = (eğrilik != 0.0).then(|| {
            if seri.yerleşim == GrafoYerleşimi::Dairesel {
                let eğrilik = eğrilik * 3.0;
                let orta = ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0);
                (
                    görünüm_merkezi.0 * eğrilik + orta.0 * (1.0 - eğrilik),
                    görünüm_merkezi.1 * eğrilik + orta.1 * (1.0 - eğrilik),
                )
            } else {
                (
                    (p1.0 + p2.0) / 2.0 - (p1.1 - p2.1) * eğrilik,
                    (p1.1 + p2.1) / 2.0 - (p2.0 - p1.0) * eğrilik,
                )
            }
        });
        let semboller = kaynak_bağ
            .semboller
            .clone()
            .unwrap_or_else(|| seri.kenar_sembolleri.clone());
        let sembol_boyutları = kaynak_bağ
            .sembol_boyutları
            .unwrap_or(seri.kenar_sembol_boyutları);
        let kaynak_yarıçapı =
            (!matches!(semboller[0], Sembol::Yok)).then_some(düğümler[kaynak_yeni].boyut / 2.0);
        let hedef_yarıçapı =
            (!matches!(semboller[1], Sembol::Yok)).then_some(düğümler[hedef_yeni].boyut / 2.0);
        let (başlangıç, bitiş, yeni_kontrol) =
            bağı_kırp(p1, p2, kontrol, kaynak_yarıçapı, hedef_yarıçapı);
        kontrol = yeni_kontrol;
        let renk = match stil.renk.as_ref() {
            Some(GrafoKenarBoyası::Renk(renk)) => *renk,
            Some(GrafoKenarBoyası::Kaynak) => düğümler[kaynak_yeni].renk.temsilî(),
            Some(GrafoKenarBoyası::Hedef) => düğümler[hedef_yeni].renk.temsilî(),
            None => tema::nötr_50(),
        };
        let etiket = kaynak_bağ.etiket.as_ref().map_or_else(
            || seri.kenar_etiketi.clone(),
            |yama| yama.uygula(&seri.kenar_etiketi),
        );
        let mut etiket_konumu = kontrol.map_or_else(
            || ((başlangıç.0 + bitiş.0) / 2.0, (başlangıç.1 + bitiş.1) / 2.0),
            |kontrol| kuadratik_nokta(başlangıç, kontrol, bitiş, 0.5),
        );
        let teğet = kontrol.map_or(
            (bitiş.0 - başlangıç.0, bitiş.1 - başlangıç.1),
            |kontrol| kuadratik_teğet(başlangıç, kontrol, bitiş, 0.5),
        );
        // ECharts `Line` bağlı metni orta konumda yolun okunabilir üst
        // normaline `distance` kadar taşır. Yönü ters bağlarda hem normal
        // hem yazı açısı çevrilir; etiket baş aşağı kalmaz.
        let teğet_uzunluğu = (teğet.0 * teğet.0 + teğet.1 * teğet.1).sqrt();
        let etiket_ölçeği = if dış_koordinat {
            1.0
        } else {
            (görünüm_alanı.genişlik / veri_alanı.genişlik.max(f32::EPSILON))
                .abs()
                .max(f32::EPSILON)
        };
        let etiket_uzaklığı = etiket.uzaklık / etiket_ölçeği;
        if etiket_uzaklığı > 0.0 && teğet_uzunluğu > f32::EPSILON {
            let mut normal = (teğet.1 / teğet_uzunluğu, -teğet.0 / teğet_uzunluğu);
            if normal.1 > 0.0 {
                normal = (-normal.0, -normal.1);
            }
            etiket_konumu.0 += normal.0 * etiket_uzaklığı;
            etiket_konumu.1 += normal.1 * etiket_uzaklığı;
        }
        let mut etiket_dönüşü = match etiket.döndürme {
            EtiketDöndürme::Derece(değer) => -değer.to_radians(),
            _ => teğet.1.atan2(teğet.0),
        };
        if !matches!(etiket.döndürme, EtiketDöndürme::Derece(_)) {
            if etiket_dönüşü > std::f32::consts::FRAC_PI_2 {
                etiket_dönüşü -= std::f32::consts::PI;
            } else if etiket_dönüşü < -std::f32::consts::FRAC_PI_2 {
                etiket_dönüşü += std::f32::consts::PI;
            }
        }
        // LineDraw'ın edge `name` alanı, çözülmüş düğüm etiketlerinden
        // değil ham `source`/`target` uçlarından üretilir. Örneğin adı
        // "Node 1" olan 0. düğüme sayıyla bağlanan kenarın etiketi
        // `0 > 1` olur.
        let etiket_adı = format!(
            "{} > {}",
            grafo_ucu_metni(&kaynak_bağ.kaynak),
            grafo_ucu_metni(&kaynak_bağ.hedef)
        );
        let yerleşik_sıra = yerleşik_bağlar.len();
        düğümler[kaynak_yeni].komşu_bağlar.push(yerleşik_sıra);
        düğümler[hedef_yeni].komşu_bağlar.push(yerleşik_sıra);
        if !düğümler[kaynak_yeni].komşu_düğümler.contains(&hedef_yeni) {
            düğümler[kaynak_yeni].komşu_düğümler.push(hedef_yeni);
        }
        if !düğümler[hedef_yeni].komşu_düğümler.contains(&kaynak_yeni) {
            düğümler[hedef_yeni].komşu_düğümler.push(kaynak_yeni);
        }
        yerleşik_bağlar.push(GrafoYerleşikBağ {
            veri_sırası,
            kaynak_sırası: kaynak_yeni,
            hedef_sırası: hedef_yeni,
            kaynak: düğümler[kaynak_yeni].kimlik.clone(),
            hedef: düğümler[hedef_yeni].kimlik.clone(),
            değer: kaynak_bağ.değer,
            başlangıç,
            bitiş,
            kontrol,
            kaynak_sembolü: semboller[0].clone(),
            hedef_sembolü: semboller[1].clone(),
            kaynak_sembol_boyutu: sembol_boyutları[0] * düğüm_ölçeği,
            hedef_sembol_boyutu: sembol_boyutları[1] * düğüm_ölçeği,
            çizgi_stili: stil,
            renk,
            etiket_metni: etiket_metni(&etiket, kaynak_bağ.değer, &etiket_adı, seri.ad.as_deref()),
            etiket,
            etiket_konumu,
            etiket_dönüşü,
            vurgu: durum_yama_uygula(&seri.vurgu, &kaynak_bağ.vurgu),
            bulanık: durum_yama_uygula(&seri.bulanık, &kaynak_bağ.bulanık),
            seçili: durum_yama_uygula(&seri.seçili, &kaynak_bağ.seçili),
            kuvvet_yerleşimini_yoksay: kaynak_bağ.kuvvet_yerleşimini_yoksay,
        });
    }
    if seri.etiket_örtüşmesini_gizle {
        let mut kabul = Vec::<Dikdörtgen>::new();
        // ECharts LabelManager, hideOverlap adaylarını bağlı grafik öğesinin
        // sınır alanına göre büyükten küçüğe sıralar (`priority = w * h`).
        // Eşitlikte kararlı veri sırası korunur.
        let mut adaylar = düğümler
            .iter()
            .enumerate()
            .filter(|(_, düğüm)| düğüm.etiket.göster || seri.etiket_göster)
            .map(|(sıra, düğüm)| (sıra, düğüm.boyut * düğüm.boyut))
            .collect::<Vec<_>>();
        adaylar.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        for (sıra, _) in adaylar {
            let kutu = etiket_kutusu(&düğümler[sıra]);
            if kabul
                .iter()
                .copied()
                .any(|eski| kutular_kesişir(eski, kutu))
            {
                düğümler[sıra].etiket_gizli = true;
            } else {
                kabul.push(kutu);
            }
        }
    }
    Ok(GrafoYerleşimSonucu {
        veri_alanı,
        görünüm_alanı,
        düğüm_ölçeği,
        düğümler,
        bağlar: yerleşik_bağlar,
    })
}

fn yol_kuadratik(yol: &mut Yol, başlangıç: (f32, f32), kontrol: (f32, f32), bitiş: (f32, f32)) {
    let k1 = (
        başlangıç.0 + (kontrol.0 - başlangıç.0) * 2.0 / 3.0,
        başlangıç.1 + (kontrol.1 - başlangıç.1) * 2.0 / 3.0,
    );
    let k2 = (
        bitiş.0 + (kontrol.0 - bitiş.0) * 2.0 / 3.0,
        bitiş.1 + (kontrol.1 - bitiş.1) * 2.0 / 3.0,
    );
    yol.kübik(k1, k2, bitiş);
}

#[allow(clippy::too_many_arguments)]
fn etiketi_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    etiket: &Etiket,
    metin: &str,
    konum: (f32, f32),
    dönüş: f32,
    yatay: YatayHiza,
    dikey: DikeyHiza,
    varsayılan_renk: Renk,
    opaklık: f32,
) {
    if metin.is_empty() {
        return;
    }
    let boyut = etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let renk = etiket.yazı.renk.unwrap_or(varsayılan_renk).opaklık(opaklık);
    let dönüşüm = AfinMatris::ötele(konum.0, konum.1).çarp(AfinMatris::döndür(dönüş));
    let satırlar = metin.split('\n').collect::<Vec<_>>();
    let satır_yüksekliği = etiket.yazı.satır_yüksekliği.unwrap_or(boyut);
    let toplam = satır_yüksekliği * satırlar.len() as f32;
    let ilk = match dikey {
        DikeyHiza::Üst => boyut / 2.0,
        DikeyHiza::Orta => -toplam / 2.0 + boyut / 2.0,
        DikeyHiza::Alt => -toplam + boyut / 2.0,
    };
    for (sıra, satır) in satırlar.into_iter().enumerate() {
        yüzey.dönüşümlü_yazı(
            satır,
            (0.0, ilk + sıra as f32 * satır_yüksekliği),
            yatay,
            DikeyHiza::Orta,
            boyut,
            renk,
            etiket.yazı.kalın,
            dönüşüm,
        );
    }
}

fn kenar_sembolünü_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    sembol: &Sembol,
    boyut: f32,
    konum: (f32, f32),
    yön: (f32, f32),
    renk: Renk,
    opaklık: f32,
) {
    if matches!(sembol, Sembol::Yok) || boyut <= 0.0 {
        return;
    }
    let açı = yön.1.atan2(yön.0).to_degrees() + 90.0;
    sembol_stilli_dönüşümlü_çiz(
        yüzey,
        sembol,
        konum,
        boyut,
        açı,
        renk,
        Some(&Dolgu::Düz(renk)),
        None,
        opaklık,
        false,
    );
}

/// Hazırlanmış Graph sahnesini boyar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Grafoİsabeti {
    /// [`GrafoYerleşimSonucu::düğümler`] içindeki sıra.
    Düğüm(usize),
    /// [`GrafoYerleşimSonucu::bağlar`] içindeki sıra.
    Bağ(usize),
}

fn bağ_isabet_geometrisi(bağ: &GrafoYerleşikBağ) -> İsabetGeometrisi {
    let noktalar = if let Some(kontrol) = bağ.kontrol {
        (0..=20)
            .map(|sıra| kuadratik_nokta(bağ.başlangıç, kontrol, bağ.bitiş, sıra as f32 / 20.0))
            .collect()
    } else {
        vec![bağ.başlangıç, bağ.bitiş]
    };
    İsabetGeometrisi::ÇokluÇizgi {
        noktalar,
        tolerans: (bağ.çizgi_stili.kalınlık.unwrap_or(1.0) / 2.0 + 3.0).max(5.0),
    }
}

/// Düğümler üstte, bağlar altta olacak biçimde Graph isabetini çözer.
pub fn grafo_isabetini_bul(
    yerleşim: &GrafoYerleşimSonucu,
    nokta: (f32, f32),
) -> Option<Grafoİsabeti> {
    for (sıra, düğüm) in yerleşim.düğümler.iter().enumerate().rev() {
        let yarıçap = (düğüm.boyut / 2.0 + 3.0).max(8.0);
        if (nokta.0 - düğüm.konum.0).powi(2) + (nokta.1 - düğüm.konum.1).powi(2)
            <= yarıçap * yarıçap
        {
            return Some(Grafoİsabeti::Düğüm(sıra));
        }
    }
    yerleşim
        .bağlar
        .iter()
        .enumerate()
        .rev()
        .find(|(_, bağ)| bağ_isabet_geometrisi(bağ).içeriyor_mu(nokta))
        .map(|(sıra, _)| Grafoİsabeti::Bağ(sıra))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GrafoEtkinDurum {
    Normal,
    Vurgu,
    Bulanık,
    Seçili,
}

fn vurgu_odağı(
    yerleşim: &GrafoYerleşimSonucu,
    isabet: Option<Grafoİsabeti>,
) -> crate::model::grafo::GrafoVurguOdağı {
    use crate::model::grafo::GrafoVurguOdağı;
    match isabet {
        Some(Grafoİsabeti::Düğüm(sıra)) => yerleşim
            .düğümler
            .get(sıra)
            .filter(|düğüm| düğüm.vurgu.devre_dışı != Some(true))
            .and_then(|düğüm| düğüm.vurgu.odak)
            .unwrap_or(GrafoVurguOdağı::Yok),
        Some(Grafoİsabeti::Bağ(sıra)) => yerleşim
            .bağlar
            .get(sıra)
            .filter(|bağ| bağ.vurgu.devre_dışı != Some(true))
            .and_then(|bağ| bağ.vurgu.odak)
            .unwrap_or(GrafoVurguOdağı::Yok),
        None => GrafoVurguOdağı::Yok,
    }
}

fn düğüm_etkin_durumu(
    yerleşim: &GrafoYerleşimSonucu,
    sıra: usize,
    isabet: Option<Grafoİsabeti>,
) -> GrafoEtkinDurum {
    use crate::model::grafo::GrafoVurguOdağı;
    if isabet == Some(Grafoİsabeti::Düğüm(sıra))
        && yerleşim.düğümler[sıra].vurgu.devre_dışı != Some(true)
    {
        return GrafoEtkinDurum::Vurgu;
    }
    let bulanık = match (isabet, vurgu_odağı(yerleşim, isabet)) {
        (Some(Grafoİsabeti::Düğüm(hedef)), GrafoVurguOdağı::Kendisi) => sıra != hedef,
        (Some(Grafoİsabeti::Düğüm(hedef)), GrafoVurguOdağı::Komşuluk) => {
            sıra != hedef && !yerleşim.düğümler[hedef].komşu_düğümler.contains(&sıra)
        }
        (Some(Grafoİsabeti::Bağ(hedef)), GrafoVurguOdağı::Kendisi) => {
            let _ = hedef;
            true
        }
        (Some(Grafoİsabeti::Bağ(hedef)), GrafoVurguOdağı::Komşuluk) => yerleşim
            .bağlar
            .get(hedef)
            .is_some_and(|bağ| sıra != bağ.kaynak_sırası && sıra != bağ.hedef_sırası),
        _ => false,
    };
    if bulanık {
        GrafoEtkinDurum::Bulanık
    } else if yerleşim.düğümler[sıra].başlangıçta_seçili {
        GrafoEtkinDurum::Seçili
    } else {
        GrafoEtkinDurum::Normal
    }
}

fn bağ_etkin_durumu(
    yerleşim: &GrafoYerleşimSonucu,
    sıra: usize,
    isabet: Option<Grafoİsabeti>,
) -> GrafoEtkinDurum {
    use crate::model::grafo::GrafoVurguOdağı;
    if isabet == Some(Grafoİsabeti::Bağ(sıra))
        && yerleşim.bağlar[sıra].vurgu.devre_dışı != Some(true)
    {
        return GrafoEtkinDurum::Vurgu;
    }
    let bulanık = match (isabet, vurgu_odağı(yerleşim, isabet)) {
        (Some(Grafoİsabeti::Düğüm(hedef)), GrafoVurguOdağı::Kendisi) => {
            let _ = hedef;
            true
        }
        (Some(Grafoİsabeti::Düğüm(hedef)), GrafoVurguOdağı::Komşuluk) => {
            !yerleşim.düğümler[hedef].komşu_bağlar.contains(&sıra)
        }
        (Some(Grafoİsabeti::Bağ(hedef)), GrafoVurguOdağı::Kendisi) => sıra != hedef,
        (Some(Grafoİsabeti::Bağ(hedef)), GrafoVurguOdağı::Komşuluk) => sıra != hedef,
        _ => false,
    };
    if bulanık {
        GrafoEtkinDurum::Bulanık
    } else {
        GrafoEtkinDurum::Normal
    }
}

pub fn grafo_yerleşimini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &GrafoSerisi,
    yerleşim: &GrafoYerleşimSonucu,
    seri_sırası: usize,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    grafo_yerleşimini_durumla_çiz(
        yüzey,
        seri,
        yerleşim,
        seri_sırası,
        ilerleme,
        None,
        isabetler,
    );
}

/// Hazırlanmış Graph sahnesini etkin emphasis/blur/select durumuyla boyar.
#[allow(clippy::too_many_arguments)]
pub fn grafo_yerleşimini_durumla_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &GrafoSerisi,
    yerleşim: &GrafoYerleşimSonucu,
    seri_sırası: usize,
    ilerleme: f32,
    vurgulu: Option<Grafoİsabeti>,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let opaklık = ilerleme.clamp(0.0, 1.0);
    for (bağ_sırası, bağ) in yerleşim.bağlar.iter().enumerate() {
        let etkin = bağ_etkin_durumu(yerleşim, bağ_sırası, vurgulu);
        let durum = match etkin {
            GrafoEtkinDurum::Vurgu => Some(&bağ.vurgu),
            GrafoEtkinDurum::Bulanık => Some(&bağ.bulanık),
            GrafoEtkinDurum::Seçili => Some(&bağ.seçili),
            GrafoEtkinDurum::Normal => None,
        };
        let durum_stili = durum.and_then(|durum| durum.çizgi_stili.as_ref());
        let mut stil = bağ.çizgi_stili.clone();
        if let Some(yama) = durum_stili {
            stil = çizgi_stili_yama_uygula(&stil, yama);
        }
        let normal_opaklığı = bağ.çizgi_stili.opaklık.unwrap_or(0.5);
        let hedef_opaklık = if etkin == GrafoEtkinDurum::Bulanık
            && durum_stili.and_then(|stil| stil.opaklık).is_none()
        {
            normal_opaklığı * 0.1
        } else {
            stil.opaklık.unwrap_or(normal_opaklığı)
        };
        let çizgi_opaklığı = hedef_opaklık * opaklık;
        let kalınlık = stil.kalınlık.unwrap_or(1.0).max(0.0);
        let tür = stil.tür.unwrap_or(ÇizgiTürü::Düz);
        let renk = match stil.renk.as_ref() {
            Some(GrafoKenarBoyası::Renk(renk)) => *renk,
            Some(GrafoKenarBoyası::Kaynak) => yerleşim.düğümler[bağ.kaynak_sırası].renk.temsilî(),
            Some(GrafoKenarBoyası::Hedef) => yerleşim.düğümler[bağ.hedef_sırası].renk.temsilî(),
            None => bağ.renk,
        };
        let mut yol = Yol::yeni();
        yol.taşı(bağ.başlangıç);
        if let Some(kontrol) = bağ.kontrol {
            yol_kuadratik(&mut yol, bağ.başlangıç, kontrol, bağ.bitiş);
        } else {
            yol.çiz(bağ.bitiş);
        }
        if let Some(gölge_rengi) = stil.gölge_rengi
            && (stil.gölge_bulanıklığı.unwrap_or(0.0) > 0.0
                || stil.gölge_kayması.unwrap_or((0.0, 0.0)) != (0.0, 0.0))
        {
            yüzey.yol_çizgi_gölgesi(
                &yol,
                kalınlık,
                tür,
                gölge_rengi.opaklık(çizgi_opaklığı),
                stil.gölge_bulanıklığı.unwrap_or(0.0),
                stil.gölge_kayması.unwrap_or((0.0, 0.0)),
            );
        }
        yüzey.yol_çiz(&yol, kalınlık, renk.opaklık(çizgi_opaklığı), tür);
        let kaynak_yön = bağ.kontrol.map_or(
            (bağ.bitiş.0 - bağ.başlangıç.0, bağ.bitiş.1 - bağ.başlangıç.1),
            |kontrol| (kontrol.0 - bağ.başlangıç.0, kontrol.1 - bağ.başlangıç.1),
        );
        let hedef_yön = bağ.kontrol.map_or(kaynak_yön, |kontrol| {
            (bağ.bitiş.0 - kontrol.0, bağ.bitiş.1 - kontrol.1)
        });
        kenar_sembolünü_çiz(
            yüzey,
            &bağ.kaynak_sembolü,
            bağ.kaynak_sembol_boyutu,
            bağ.başlangıç,
            (-kaynak_yön.0, -kaynak_yön.1),
            renk,
            çizgi_opaklığı,
        );
        kenar_sembolünü_çiz(
            yüzey,
            &bağ.hedef_sembolü,
            bağ.hedef_sembol_boyutu,
            bağ.bitiş,
            hedef_yön,
            renk,
            çizgi_opaklığı,
        );
        let etiket = durum
            .and_then(|durum| durum.kenar_etiketi.as_ref().or(durum.etiket.as_ref()))
            .map_or_else(|| bağ.etiket.clone(), |yama| yama.uygula(&bağ.etiket));
        if etiket.göster {
            let etiket_opaklığı = if normal_opaklığı > f32::EPSILON {
                opaklık * (hedef_opaklık / normal_opaklığı)
            } else {
                opaklık * hedef_opaklık
            };
            etiketi_çiz(
                yüzey,
                &etiket,
                &bağ.etiket_metni,
                bağ.etiket_konumu,
                bağ.etiket_dönüşü,
                YatayHiza::Orta,
                DikeyHiza::Alt,
                tema::birincil_metin(),
                etiket_opaklığı,
            );
        }
        if !seri.sessiz {
            isabetler.push(İsabetBölgesi {
                seri_sırası,
                veri_sırası: bağ.veri_sırası,
                seri_adı: seri.ad.clone(),
                ad: Some(format!("{} > {}", bağ.kaynak, bağ.hedef)),
                değer: bağ.değer,
                geometri: bağ_isabet_geometrisi(bağ),
            });
        }
    }

    for (düğüm_sırası, düğüm) in yerleşim.düğümler.iter().enumerate() {
        if !düğüm.konum.0.is_finite() || !düğüm.konum.1.is_finite() {
            continue;
        }
        let etkin = düğüm_etkin_durumu(yerleşim, düğüm_sırası, vurgulu);
        let durum = match etkin {
            GrafoEtkinDurum::Vurgu => Some(&düğüm.vurgu),
            GrafoEtkinDurum::Bulanık => Some(&düğüm.bulanık),
            GrafoEtkinDurum::Seçili => Some(&düğüm.seçili),
            GrafoEtkinDurum::Normal => None,
        };
        let durum_stili = durum.and_then(|durum| durum.öğe_stili.as_ref());
        let mut stil = düğüm.öğe_stili.clone();
        if let Some(yama) = durum_stili {
            stil = öğe_stili_yama_uygula(&stil, yama);
        }
        let normal_opaklığı = düğüm.öğe_stili.opaklık.unwrap_or(1.0);
        let hedef_opaklık = if etkin == GrafoEtkinDurum::Bulanık
            && durum_stili.and_then(|stil| stil.opaklık).is_none()
        {
            normal_opaklığı * 0.1
        } else {
            stil.opaklık.unwrap_or(normal_opaklığı)
        };
        let durum_opaklık_çarpanı = if normal_opaklığı > f32::EPSILON {
            hedef_opaklık / normal_opaklığı
        } else {
            hedef_opaklık
        };
        let dolgu = durum_stili.and_then(|stil| stil.renk.as_ref()).map_or_else(
            || düğüm.renk.opaklık(durum_opaklık_çarpanı),
            |renk| renk.opaklık(hedef_opaklık),
        );
        let durum_ölçeği = if etkin == GrafoEtkinDurum::Vurgu {
            durum.and_then(|durum| durum.ölçek).unwrap_or(1.0)
        } else {
            1.0
        };
        let boyut = düğüm.boyut * durum_ölçeği * opaklık.max(0.01);
        if let Some(gölge_rengi) = stil.gölge_rengi
            && let Some(yol) = crate::grafik::sembol_yolu(&düğüm.sembol, düğüm.konum, boyut, false)
            && (stil.gölge_bulanıklığı.unwrap_or(0.0) > 0.0
                || stil.gölge_kayması.unwrap_or((0.0, 0.0)) != (0.0, 0.0))
        {
            yüzey.yol_gölgesi(
                &yol,
                gölge_rengi.opaklık(opaklık * durum_opaklık_çarpanı),
                stil.gölge_bulanıklığı.unwrap_or(0.0),
                stil.gölge_kayması.unwrap_or((0.0, 0.0)),
            );
        }
        let kenarlık = stil
            .kenarlık_kalınlığı
            .filter(|kalınlık| *kalınlık > 0.0)
            .map(|kalınlık| {
                (
                    kalınlık,
                    stil.kenarlık_rengi
                        .unwrap_or_else(|| dolgu.temsilî())
                        .opaklık(durum_opaklık_çarpanı),
                )
            });
        sembol_stilli_dönüşümlü_çiz(
            yüzey,
            &düğüm.sembol,
            düğüm.konum,
            boyut,
            0.0,
            dolgu.temsilî(),
            Some(&dolgu),
            kenarlık,
            1.0,
            false,
        );
        let etiket = durum
            .and_then(|durum| durum.etiket.as_ref())
            .map_or_else(|| düğüm.etiket.clone(), |yama| yama.uygula(&düğüm.etiket));
        if (seri.etiket_göster || etiket.göster)
            && düğüm.boyut >= seri.etiket_eşiği
            && !düğüm.etiket_gizli
        {
            let iç_etiket = matches!(
                etiket.konum,
                EtiketKonumu::İç
                    | EtiketKonumu::Merkez
                    | EtiketKonumu::İçÜst
                    | EtiketKonumu::İçAlt
                    | EtiketKonumu::İçSol
                    | EtiketKonumu::İçSağ
                    | EtiketKonumu::İçSolÜst
                    | EtiketKonumu::İçSağÜst
                    | EtiketKonumu::İçSolAlt
                    | EtiketKonumu::İçSağAlt
            );
            etiketi_çiz(
                yüzey,
                &etiket,
                &düğüm.etiket_metni,
                düğüm.etiket_konumu,
                düğüm.etiket_dönüşü,
                düğüm.etiket_yatay_hizası,
                düğüm.etiket_dikey_hizası,
                if iç_etiket {
                    Renk::BEYAZ
                } else {
                    // ECharts 6.1 varsayılan Graph label rengi.
                    Renk::onaltılık(0x333333)
                },
                opaklık * durum_opaklık_çarpanı,
            );
        }
        if !seri.sessiz {
            isabetler.push(İsabetBölgesi {
                seri_sırası,
                veri_sırası: düğüm.veri_sırası,
                seri_adı: seri.ad.clone(),
                ad: Some(düğüm.ad.clone()),
                değer: düğüm.değer,
                geometri: İsabetGeometrisi::Daire {
                    merkez: düğüm.konum,
                    yarıçap: (boyut / 2.0 + 3.0).max(8.0),
                },
            });
        }
    }
}

/// Eski görünüm çağrı yüzeyi; Calendar/Matrix eşlemesini içeride kurar.
#[allow(clippy::too_many_arguments)]
pub fn grafo_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &GrafoSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    görünüm: (f32, f32, f32),
    kaymalar: &[(usize, f32, f32)],
    takvim: Option<&TakvimYerleşimi>,
    matris: Option<&MatrisYerleşimi>,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let harita = |_: usize, düğüm: &GrafoDüğümü| -> Option<(f32, f32)> {
        if let Some(takvim) = takvim {
            return düğüm
                .takvim_tarihi_ms
                .and_then(|tarih| takvim.veriden_noktaya(tarih));
        }
        if let Some(matris) = matris {
            return düğüm
                .matris_koordinatı
                .as_ref()
                .and_then(|(x, y)| matris.veriden_noktaya(x.clone(), y.clone()));
        }
        None
    };
    let harita_ref = (takvim.is_some() || matris.is_some())
        .then_some(&harita as &dyn Fn(usize, &GrafoDüğümü) -> Option<(f32, f32)>);
    if let Ok(yerleşim) = grafo_yerleşimi_kur(
        seri,
        tuval,
        palet,
        görünüm,
        kaymalar,
        harita_ref,
        &HashSet::new(),
    ) {
        grafo_yerleşimini_çiz(çizici, seri, &yerleşim, genel_sıra, ilerleme, isabetler);
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::model::grafo::{
        GrafoDaireselAyarı, GrafoDurumu, GrafoKuvveti, GrafoSerisi, GrafoVurguOdağı,
        GrafoYerleşimi, GrafoÖğeStili,
    };
    use crate::model::takvim::TakvimKoordinatı;
    use crate::yardimci::takvim::{TakvimAnı, takvimden_ana};

    fn alan() -> Dikdörtgen {
        Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0)
    }

    #[test]
    fn resmi_kuvvet_sürtünmesi_durana_dek_ve_sabit_düğümle_çalışır() {
        let seri = GrafoSerisi::yeni()
            .yerleşim(GrafoYerleşimi::Kuvvet)
            .kuvvet(
                GrafoKuvveti::yeni()
                    .itme(100.0)
                    .kenar_uzunluğu(5.0)
                    .yerçekimi(0.0),
            )
            .düğümler([
                GrafoDüğümü::yeni("sabit", 20.0)
                    .konum(350.0, 262.5)
                    .sabit(true),
                GrafoDüğümü::yeni("a", 10.0),
                GrafoDüğümü::yeni("b", 10.0),
            ])
            .bağlar([("sabit", "a"), ("a", "b")]);
        let yerleşim = grafo_yerleşimi_kur(
            &seri,
            alan(),
            &|_| Renk::SİYAH,
            (0.0, 0.0, 1.0),
            &[],
            None,
            &HashSet::new(),
        )
        .expect("yerleşim");
        assert_eq!(yerleşim.düğümler[0].ham_konum, (350.0, 262.5));
        assert!(yerleşim.düğümler[1].ham_konum.0.is_finite());
        assert!(yerleşim.düğümler[2].ham_konum.1.is_finite());
    }

    #[test]
    fn dairesel_sembol_boyutunu_ve_döndürülmüş_etiketi_korur() {
        let seri = GrafoSerisi::yeni()
            .yerleşim(GrafoYerleşimi::Dairesel)
            .dairesel(GrafoDaireselAyarı::yeni().etiketi_döndür(true))
            .etiket_göster(true)
            .etiket_eşiği(0.0)
            .düğümler([
                GrafoDüğümü::yeni("a", 10.0),
                GrafoDüğümü::yeni("b", 40.0),
                GrafoDüğümü::yeni("c", 10.0),
            ]);
        let yerleşim = grafo_yerleşimi_kur(
            &seri,
            alan(),
            &|_| Renk::SİYAH,
            (0.0, 0.0, 1.0),
            &[],
            None,
            &HashSet::new(),
        )
        .expect("yerleşim");
        assert_eq!(yerleşim.düğümler.len(), 3);
        assert!(
            yerleşim
                .düğümler
                .iter()
                .any(|düğüm| düğüm.etiket_dönüşü != 0.0)
        );
    }

    #[test]
    fn takvime_bağlı_düğüm_tarih_hücresinin_merkezine_yerleşir() {
        let tarih = takvimden_ana(TakvimAnı {
            yıl: 2017,
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let seri = GrafoSerisi::yeni()
            .takvim_sırası(0)
            .düğümler([GrafoDüğümü::yeni("2017-01-01", 15.0)
                .değerli(260.0)
                .takvim_tarihi(tarih)]);
        let yerleşim = TakvimYerleşimi::kur(&TakvimKoordinatı::yıl(2017), (700.0, 525.0))
            .expect("takvim yerleşimi kurulmalı");
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let mut isabetler = Vec::new();

        grafo_çiz(
            &mut yüzey,
            &seri,
            0,
            alan(),
            &|_| Renk::SİYAH,
            1.0,
            (0.0, 0.0, 1.0),
            &[],
            Some(&yerleşim),
            None,
            &mut isabetler,
        );

        assert_eq!(isabetler.len(), 1);
        assert!(matches!(
            isabetler[0].geometri,
            İsabetGeometrisi::Daire {
                merkez: (90.0, 70.0),
                ..
            }
        ));
    }

    #[test]
    fn kaynak_hedef_sembolleri_kenari_dugum_sinirinda_kirpar() {
        let seri = GrafoSerisi::yeni()
            .yerleşim(GrafoYerleşimi::Yok)
            .kenar_sembolleri(Sembol::Daire, Sembol::Üçgen)
            .düğümler([
                GrafoDüğümü::yeni("a", 40.0).konum(100.0, 100.0),
                GrafoDüğümü::yeni("b", 20.0).konum(300.0, 100.0),
            ])
            .bağlar([("a", "b")]);
        let yerleşim = grafo_yerleşimi_kur(
            &seri,
            alan(),
            &|_| Renk::SİYAH,
            (0.0, 0.0, 1.0),
            &[],
            None,
            &HashSet::new(),
        )
        .expect("yerleşim");
        assert!((yerleşim.bağlar[0].başlangıç.0 - 20.0).abs() > 1.0);
        assert!(yerleşim.bağlar[0].başlangıç.0 > yerleşim.düğümler[0].konum.0);
        assert!(yerleşim.bağlar[0].bitiş.0 < yerleşim.düğümler[1].konum.0);
    }

    #[test]
    fn dugum_kenardan_once_isabet_alir_ve_kenar_ayri_hedeftir() {
        let seri = GrafoSerisi::yeni()
            .yerleşim(GrafoYerleşimi::Yok)
            .düğümler([
                GrafoDüğümü::yeni("a", 30.0).konum(100.0, 100.0),
                GrafoDüğümü::yeni("b", 30.0).konum(300.0, 100.0),
            ])
            .bağlar([("a", "b")]);
        let yerleşim = grafo_yerleşimi_kur(
            &seri,
            alan(),
            &|_| Renk::SİYAH,
            (0.0, 0.0, 1.0),
            &[],
            None,
            &HashSet::new(),
        )
        .expect("yerleşim");
        assert_eq!(
            grafo_isabetini_bul(&yerleşim, yerleşim.düğümler[0].konum),
            Some(Grafoİsabeti::Düğüm(0))
        );
        let orta = (
            (yerleşim.bağlar[0].başlangıç.0 + yerleşim.bağlar[0].bitiş.0) / 2.0,
            (yerleşim.bağlar[0].başlangıç.1 + yerleşim.bağlar[0].bitiş.1) / 2.0,
        );
        assert_eq!(
            grafo_isabetini_bul(&yerleşim, orta),
            Some(Grafoİsabeti::Bağ(0))
        );
    }

    #[test]
    fn emphasis_blur_select_stilleri_ve_kenar_isabetleri_boyanir() {
        let eski_odak = GrafoSerisi::yeni().eski_komşuluk_odağı(false);
        assert_eq!(eski_odak.vurgu.odak, Some(GrafoVurguOdağı::Komşuluk));
        let açık_odak = GrafoSerisi::yeni()
            .vurgu(GrafoDurumu::yeni().odak(GrafoVurguOdağı::Kendisi))
            .eski_komşuluk_odağı(true);
        assert_eq!(açık_odak.vurgu.odak, Some(GrafoVurguOdağı::Kendisi));

        let seri = GrafoSerisi::yeni()
            .yerleşim(GrafoYerleşimi::Yok)
            .düğümler([
                GrafoDüğümü::yeni("a", 30.0).konum(100.0, 100.0).vurgu(
                    GrafoDurumu::yeni()
                        .odak(GrafoVurguOdağı::Kendisi)
                        .öğe_stili(GrafoÖğeStili::yeni().renk(0xff0000)),
                ),
                GrafoDüğümü::yeni("b", 30.0)
                    .konum(300.0, 100.0)
                    .başlangıçta_seçili(true)
                    .seçili(GrafoDurumu::yeni().öğe_stili(GrafoÖğeStili::yeni().renk(0x00ff00))),
            ])
            .bağlar([("a", "b")]);
        let yerleşim = grafo_yerleşimi_kur(
            &seri,
            alan(),
            &|_| Renk::onaltılık(0x336699),
            (0.0, 0.0, 1.0),
            &[],
            None,
            &HashSet::new(),
        )
        .expect("yerleşim");
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let mut isabetler = Vec::new();
        grafo_yerleşimini_durumla_çiz(
            &mut yüzey,
            &seri,
            &yerleşim,
            0,
            1.0,
            Some(Grafoİsabeti::Düğüm(0)),
            &mut isabetler,
        );
        let döküm = yüzey.döküm();
        assert!(döküm.contains("#ff0000@1.0"), "{döküm}");
        assert!(döküm.contains("#336699@0.1"), "{döküm}");
        assert!(
            isabetler
                .iter()
                .any(|isabet| matches!(isabet.geometri, İsabetGeometrisi::ÇokluÇizgi { .. }))
        );

        let mut seçili_yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        grafo_yerleşimini_durumla_çiz(
            &mut seçili_yüzey,
            &seri,
            &yerleşim,
            0,
            1.0,
            None,
            &mut Vec::new(),
        );
        assert!(
            seçili_yüzey.döküm().contains("#00ff00@1.0"),
            "{}",
            seçili_yüzey.döküm()
        );
    }

    #[test]
    fn dairesel_dugum_suruklemesi_yaricapi_korur() {
        let seri = GrafoSerisi::yeni()
            .yerleşim(GrafoYerleşimi::Dairesel)
            .düğümler([
                GrafoDüğümü::yeni("a", 10.0),
                GrafoDüğümü::yeni("b", 10.0),
                GrafoDüğümü::yeni("c", 10.0),
            ]);
        let taban = grafo_yerleşimi_kur(
            &seri,
            alan(),
            &|_| Renk::SİYAH,
            (0.0, 0.0, 1.0),
            &[],
            None,
            &HashSet::new(),
        )
        .expect("taban");
        let sürüklenmiş = grafo_yerleşimi_kur(
            &seri,
            alan(),
            &|_| Renk::SİYAH,
            (0.0, 0.0, 1.0),
            &[(0, 80.0, 45.0)],
            None,
            &HashSet::new(),
        )
        .expect("sürüklenmiş");
        let merkez = taban.görünüm_alanı.merkez();
        let yarıçap = |konum: (f32, f32)| {
            ((konum.0 - merkez.0).powi(2) + (konum.1 - merkez.1).powi(2)).sqrt()
        };
        assert!(
            (yarıçap(taban.düğümler[0].konum) - yarıçap(sürüklenmiş.düğümler[0].konum)).abs()
                < 0.001
        );
        assert_ne!(taban.düğümler[0].konum, sürüklenmiş.düğümler[0].konum);
    }
}
