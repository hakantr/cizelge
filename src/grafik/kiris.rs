#![allow(clippy::indexing_slicing)]
// Düğüm/bağ indisleri tek bir doğrulamalı graph kurulumunda üretilir ve
// aynı değişmez korunarak açı dizilerine taşınır. Bu modüldeki doğrudan
// indisleme, o kapalı yerleşim çekirdeğinin okunabilirliğini korur.

//! Kiriş (Chord) yerleşimi ve görünümü.
//!
//! Hesap sırası kilitli ECharts `chordLayout.ts`, `ChordPiece.ts` ve
//! `ChordEdge.ts` kaynaklarının Rust karşılığıdır. Raster benzerliğinden
//! bağımsız denetim için bütün sektör ve şerit geometrisi dışa açılır.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::kiris::{
    KirişBağı, KirişDurumu, KirişDüğümü, KirişKenarBoyası, KirişSerisi, KirişVurguOdağı,
    KirişÇizgiStili, KirişÖğeStili,
};
use crate::model::stil::{
    Etiket, EtiketKonumu, YazıDikeyHizası, YazıYatayHizası, ÇizgiTürü
};
use crate::renk::{Dolgu, Renk, RenkDurağı};
use crate::tema;

const RADYAN: f64 = std::f64::consts::PI / 180.0;

type KirişKaynakBağı = (KirişBağı, usize, usize);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct KirişHatası(pub String);

impl fmt::Display for KirişHatası {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for KirişHatası {}

#[derive(Clone, Debug)]
pub struct KirişYerleşikDüğüm {
    pub veri_sırası: usize,
    pub kimlik: Option<String>,
    pub ad: String,
    pub değer: f64,
    pub merkez: (f32, f32),
    pub iç_yarıçap: f32,
    pub dış_yarıçap: f32,
    pub başlangıç_açısı: f32,
    pub bitiş_açısı: f32,
    pub saat_yönünde: bool,
    pub renk: Dolgu,
    pub öğe_stili: KirişÖğeStili,
    pub köşe_yarıçapları: [f32; 4],
    pub etiket: Etiket,
    pub etiket_metni: String,
    pub etiket_konumu: (f32, f32),
    pub etiket_yatay_hizası: YatayHiza,
    pub etiket_dikey_hizası: DikeyHiza,
    pub vurgu: KirişDurumu,
    pub bulanık: KirişDurumu,
    pub seçili: KirişDurumu,
    pub başlangıçta_seçili: bool,
    pub komşu_bağlar: Vec<usize>,
    pub komşu_düğümler: Vec<usize>,
    oran: f64,
}

#[derive(Clone, Debug)]
pub struct KirişYerleşikBağ {
    pub veri_sırası: usize,
    pub kaynak_sırası: usize,
    pub hedef_sırası: usize,
    pub kaynak: String,
    pub hedef: String,
    pub değer: f64,
    pub kaynak_başlangıç_açısı: f32,
    pub kaynak_bitiş_açısı: f32,
    pub hedef_başlangıç_açısı: f32,
    pub hedef_bitiş_açısı: f32,
    pub kaynak1: (f32, f32),
    pub kaynak2: (f32, f32),
    pub hedef1: (f32, f32),
    pub hedef2: (f32, f32),
    pub merkez: (f32, f32),
    pub yarıçap: f32,
    pub saat_yönünde: bool,
    pub dolgu: Dolgu,
    pub çizgi_stili: KirişÇizgiStili,
    pub kenar_etiketi: Etiket,
    pub etiket_metni: String,
    pub etiket_konumu: (f32, f32),
    pub vurgu: KirişDurumu,
    pub bulanık: KirişDurumu,
    pub seçili: KirişDurumu,
}

#[derive(Clone, Debug)]
pub struct KirişYerleşimi {
    pub alan: Dikdörtgen,
    pub düğümler: Vec<KirişYerleşikDüğüm>,
    pub bağlar: Vec<KirişYerleşikBağ>,
}

fn öğe_stili_yama_uygula(taban: &KirişÖğeStili, yama: &KirişÖğeStili) -> KirişÖğeStili {
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
    if yama.kenarlık_yarıçapı.is_some() {
        sonuç.kenarlık_yarıçapı = yama.kenarlık_yarıçapı;
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

fn çizgi_stili_yama_uygula(
    taban: &KirişÇizgiStili, yama: &KirişÇizgiStili
) -> KirişÇizgiStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk.clone_from(&yama.renk);
    }
    if yama.opaklık.is_some() {
        sonuç.opaklık = yama.opaklık;
    }
    if yama.kalınlık.is_some() {
        sonuç.kalınlık = yama.kalınlık;
    }
    if yama.tür.is_some() {
        sonuç.tür = yama.tür;
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

fn durum_yama_uygula(taban: &KirişDurumu, yama: &KirişDurumu) -> KirişDurumu {
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

/// ECharts box-layout görünüm alanı.
pub fn kiriş_alanı(seri: &KirişSerisi, tuval: Dikdörtgen) -> Dikdörtgen {
    let sol = seri.sol.çöz(tuval.genişlik);
    let üst = seri.üst.çöz(tuval.yükseklik);
    let genişlik = seri.genişlik.map_or_else(
        || {
            seri.sağ.map_or(tuval.genişlik - sol, |sağ| {
                tuval.genişlik - sol - sağ.çöz(tuval.genişlik)
            })
        },
        |değer| değer.çöz(tuval.genişlik),
    );
    let yükseklik = seri.yükseklik.map_or_else(
        || {
            seri.alt.map_or(tuval.yükseklik - üst, |alt| {
                tuval.yükseklik - üst - alt.çöz(tuval.yükseklik)
            })
        },
        |değer| değer.çöz(tuval.yükseklik),
    );
    Dikdörtgen::yeni(
        tuval.x + sol,
        tuval.y + üst,
        genişlik.max(0.0),
        yükseklik.max(0.0),
    )
}

fn açıları_normalleştir(
    mut başlangıç: f64, mut bitiş: f64, saat_yönünde: bool
) -> (f64, f64) {
    let tau = std::f64::consts::TAU;
    let yeni_başlangıç = başlangıç.rem_euclid(tau);
    bitiş += yeni_başlangıç - başlangıç;
    başlangıç = yeni_başlangıç;
    if saat_yönünde {
        if bitiş - başlangıç >= tau {
            bitiş = başlangıç + tau;
        } else if başlangıç > bitiş {
            bitiş = başlangıç + (tau - (başlangıç - bitiş).rem_euclid(tau));
        }
    } else if başlangıç - bitiş >= tau {
        bitiş = başlangıç - tau;
    } else if başlangıç < bitiş {
        bitiş = başlangıç - (tau - (bitiş - başlangıç).rem_euclid(tau));
    }
    (başlangıç, bitiş)
}

fn düğümleri_ve_bağları_kur(
    seri: &KirişSerisi,
) -> Result<(Vec<KirişDüğümü>, Vec<KirişKaynakBağı>), KirişHatası> {
    let mut düğümler = seri.düğümler.clone();
    if düğümler.is_empty() {
        let mut görülen = HashSet::new();
        for bağ in &seri.bağlar {
            for ad in [&bağ.kaynak, &bağ.hedef] {
                if görülen.insert(ad.clone()) {
                    düğümler.push(KirişDüğümü::yeni(ad.clone()));
                }
            }
        }
    }
    let mut anahtarlar = HashMap::new();
    for (sıra, düğüm) in düğümler.iter().enumerate() {
        if düğüm.ad.is_empty() {
            return Err(KirişHatası(format!("{sıra}. Kiriş düğümünün adı boş")));
        }
        if anahtarlar.insert(düğüm.ad.clone(), sıra).is_some() {
            return Err(KirişHatası(format!(
                "yinelenmiş Kiriş düğümü: {}",
                düğüm.ad
            )));
        }
        if let Some(kimlik) = &düğüm.kimlik
            && let Some(eski) = anahtarlar.insert(kimlik.clone(), sıra)
            && eski != sıra
        {
            return Err(KirişHatası(format!(
                "yinelenmiş Kiriş düğüm kimliği: {kimlik}"
            )));
        }
    }
    let mut bağlar = Vec::with_capacity(seri.bağlar.len());
    for (sıra, bağ) in seri.bağlar.iter().enumerate() {
        if !bağ.değer.is_finite() || bağ.değer < 0.0 {
            return Err(KirişHatası(format!("{sıra}. Kiriş bağ değeri geçersiz")));
        }
        let kaynak = anahtarlar
            .get(&bağ.kaynak)
            .copied()
            .ok_or_else(|| KirişHatası(format!("bilinmeyen Kiriş kaynağı: {}", bağ.kaynak)))?;
        let hedef = anahtarlar
            .get(&bağ.hedef)
            .copied()
            .ok_or_else(|| KirişHatası(format!("bilinmeyen Kiriş hedefi: {}", bağ.hedef)))?;
        bağlar.push((bağ.clone(), kaynak, hedef));
    }
    Ok((düğümler, bağlar))
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

fn köşe_yarıçapları(stil: &KirişÖğeStili, kalınlık: f32) -> [f32; 4] {
    stil.kenarlık_yarıçapı
        .map(|yarıçap| yarıçap.0.map(|değer| değer.çöz(kalınlık).max(0.0)))
        .unwrap_or([0.0; 4])
}

fn etiket_geometrisi(
    etiket: &Etiket,
    merkez: (f32, f32),
    r0: f32,
    r: f32,
    a0: f32,
    a1: f32,
) -> ((f32, f32), YatayHiza, DikeyHiza) {
    let orta = (a0 + a1) / 2.0;
    let (dx, dy) = (orta.cos(), orta.sin());
    let dış = etiket.konum == EtiketKonumu::Dış;
    let yarıçap = if dış {
        r + etiket.uzaklık
    } else {
        (r + r0) / 2.0
    };
    let yatay = etiket.yatay_hiza.map_or_else(
        || {
            if dış {
                if dx > 0.0 {
                    YatayHiza::Sol
                } else {
                    YatayHiza::Sağ
                }
            } else {
                YatayHiza::Orta
            }
        },
        |hiza| match hiza {
            YazıYatayHizası::Sol => YatayHiza::Sol,
            YazıYatayHizası::Orta => YatayHiza::Orta,
            YazıYatayHizası::Sağ => YatayHiza::Sağ,
        },
    );
    let dikey = etiket.dikey_hiza.map_or_else(
        || {
            if dış {
                if dy > 0.0 {
                    DikeyHiza::Üst
                } else {
                    DikeyHiza::Alt
                }
            } else {
                DikeyHiza::Orta
            }
        },
        |hiza| match hiza {
            YazıDikeyHizası::Üst => DikeyHiza::Üst,
            YazıDikeyHizası::Orta => DikeyHiza::Orta,
            YazıDikeyHizası::Alt => DikeyHiza::Alt,
        },
    );
    (
        (
            merkez.0 + dx * yarıçap + etiket.kayma.0,
            merkez.1 + dy * yarıçap + etiket.kayma.1,
        ),
        yatay,
        dikey,
    )
}

fn şerit_yolu(bağ: &KirişYerleşikBağ, eğrilik: f32) -> Yol {
    let mut yol = Yol::yeni();
    yol.taşı(bağ.kaynak1);
    yol.yay(
        bağ.yarıçap,
        (bağ.kaynak_bitiş_açısı - bağ.kaynak_başlangıç_açısı).abs() > std::f32::consts::PI,
        bağ.saat_yönünde,
        bağ.kaynak2,
    );
    let oran = eğrilik;
    let k2 = bağ.kaynak2;
    let h1 = bağ.hedef1;
    yol.kübik(
        (
            k2.0 + (bağ.merkez.0 - k2.0) * oran,
            k2.1 + (bağ.merkez.1 - k2.1) * oran,
        ),
        (
            h1.0 + (bağ.merkez.0 - h1.0) * oran,
            h1.1 + (bağ.merkez.1 - h1.1) * oran,
        ),
        h1,
    );
    yol.yay(
        bağ.yarıçap,
        (bağ.hedef_bitiş_açısı - bağ.hedef_başlangıç_açısı).abs() > std::f32::consts::PI,
        bağ.saat_yönünde,
        bağ.hedef2,
    );
    let h2 = bağ.hedef2;
    let k1 = bağ.kaynak1;
    yol.kübik(
        (
            h2.0 + (bağ.merkez.0 - h2.0) * oran,
            h2.1 + (bağ.merkez.1 - h2.1) * oran,
        ),
        (
            k1.0 + (bağ.merkez.0 - k1.0) * oran,
            k1.1 + (bağ.merkez.1 - k1.1) * oran,
        ),
        k1,
    );
    yol.kapat();
    yol
}

/// Chord şeridinin kapalı boyalı alanını olay/tooltip sınaması için
/// örnekler. Sınır kutusu kullanmak, içbükey şeritlerin büyük boş merkezini
/// yanlışlıkla hedef yapar; bu çokgen iki yayı ve iki kübik Bézier kenarını
/// ayrı ayrı izler.
fn şerit_çokgeni(bağ: &KirişYerleşikBağ, eğrilik: f32) -> Vec<(f32, f32)> {
    fn kübik(
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
        t: f32,
    ) -> (f32, f32) {
        let u = 1.0 - t;
        let a = u * u * u;
        let b = 3.0 * u * u * t;
        let c = 3.0 * u * t * t;
        let d = t * t * t;
        (
            a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
            a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
        )
    }

    fn yayı_ekle(
        çıktı: &mut Vec<(f32, f32)>,
        merkez: (f32, f32),
        yarıçap: f32,
        başlangıç: f32,
        bitiş: f32,
    ) {
        // En çok 7,5° aralıklı örnekler dar sektörlerde de gerçek yayın
        // yeterince yakınında kalır.
        let adım = ((bitiş - başlangıç).abs() / (std::f32::consts::PI / 24.0))
            .ceil()
            .max(1.0) as usize;
        for sıra in 0..=adım {
            let t = sıra as f32 / adım as f32;
            let açı = başlangıç + (bitiş - başlangıç) * t;
            çıktı.push((
                merkez.0 + yarıçap * açı.cos(),
                merkez.1 + yarıçap * açı.sin(),
            ));
        }
    }

    fn kübiği_ekle(
        çıktı: &mut Vec<(f32, f32)>,
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
    ) {
        for sıra in 1..=16 {
            çıktı.push(kübik(p0, p1, p2, p3, sıra as f32 / 16.0));
        }
    }

    let mut noktalar = Vec::new();
    yayı_ekle(
        &mut noktalar,
        bağ.merkez,
        bağ.yarıçap,
        bağ.kaynak_başlangıç_açısı,
        bağ.kaynak_bitiş_açısı,
    );
    let kaynak_kontrolü = (
        bağ.kaynak2.0 + (bağ.merkez.0 - bağ.kaynak2.0) * eğrilik,
        bağ.kaynak2.1 + (bağ.merkez.1 - bağ.kaynak2.1) * eğrilik,
    );
    let hedef_kontrolü = (
        bağ.hedef1.0 + (bağ.merkez.0 - bağ.hedef1.0) * eğrilik,
        bağ.hedef1.1 + (bağ.merkez.1 - bağ.hedef1.1) * eğrilik,
    );
    kübiği_ekle(
        &mut noktalar,
        bağ.kaynak2,
        kaynak_kontrolü,
        hedef_kontrolü,
        bağ.hedef1,
    );
    yayı_ekle(
        &mut noktalar,
        bağ.merkez,
        bağ.yarıçap,
        bağ.hedef_başlangıç_açısı,
        bağ.hedef_bitiş_açısı,
    );
    let hedef_dönüş_kontrolü = (
        bağ.hedef2.0 + (bağ.merkez.0 - bağ.hedef2.0) * eğrilik,
        bağ.hedef2.1 + (bağ.merkez.1 - bağ.hedef2.1) * eğrilik,
    );
    let kaynak_dönüş_kontrolü = (
        bağ.kaynak1.0 + (bağ.merkez.0 - bağ.kaynak1.0) * eğrilik,
        bağ.kaynak1.1 + (bağ.merkez.1 - bağ.kaynak1.1) * eğrilik,
    );
    kübiği_ekle(
        &mut noktalar,
        bağ.hedef2,
        hedef_dönüş_kontrolü,
        kaynak_dönüş_kontrolü,
        bağ.kaynak1,
    );
    noktalar
}

fn bağ_dolgusu(
    boya: &KirişKenarBoyası,
    opaklık: f32,
    kaynak: &Dolgu,
    hedef: &Dolgu,
    geçici: &KirişYerleşikBağ,
) -> Dolgu {
    let dolgu = match boya {
        KirişKenarBoyası::Dolgu(dolgu) => dolgu.clone(),
        KirişKenarBoyası::Kaynak => kaynak.clone(),
        KirişKenarBoyası::Hedef => hedef.clone(),
        KirişKenarBoyası::Gradyan => {
            let yol = şerit_yolu(geçici, geçici.çizgi_stili.eğrilik.unwrap_or(0.7));
            let kutu = yol
                .sınır_kutusu()
                .unwrap_or(Dikdörtgen::yeni(0.0, 0.0, 1.0, 1.0));
            let s = (
                (geçici.kaynak1.0 + geçici.kaynak2.0) / 2.0,
                (geçici.kaynak1.1 + geçici.kaynak2.1) / 2.0,
            );
            let t = (
                (geçici.hedef1.0 + geçici.hedef2.0) / 2.0,
                (geçici.hedef1.1 + geçici.hedef2.1) / 2.0,
            );
            let x = if kutu.genişlik > 0.0 {
                (s.0 - kutu.x) / kutu.genişlik
            } else {
                0.0
            };
            let y = if kutu.yükseklik > 0.0 {
                (s.1 - kutu.y) / kutu.yükseklik
            } else {
                0.0
            };
            let x2 = if kutu.genişlik > 0.0 {
                (t.0 - kutu.x) / kutu.genişlik
            } else {
                1.0
            };
            let y2 = if kutu.yükseklik > 0.0 {
                (t.1 - kutu.y) / kutu.yükseklik
            } else {
                0.0
            };
            Dolgu::doğrusal(
                x,
                y,
                x2,
                y2,
                vec![
                    RenkDurağı {
                        konum: 0.0,
                        renk: kaynak.temsilî(),
                    },
                    RenkDurağı {
                        konum: 1.0,
                        renk: hedef.temsilî(),
                    },
                ],
            )
        }
    };
    dolgu.opaklık(opaklık)
}

/// ECharts `chordLayout` ile aynı düğüm/açı/şerit yerleşimi.
pub fn kiriş_yerleşimi(
    seri: &KirişSerisi,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
) -> Result<KirişYerleşimi, KirişHatası> {
    let alan = kiriş_alanı(seri, tuval);
    let (kaynak_düğümler, kaynak_bağlar) = düğümleri_ve_bağları_kur(seri)?;
    if kaynak_bağlar.is_empty() {
        return Ok(KirişYerleşimi {
            alan,
            düğümler: Vec::new(),
            bağlar: Vec::new(),
        });
    }
    let merkez = (
        alan.x + seri.merkez.0.çöz(alan.genişlik),
        alan.y + seri.merkez.1.çöz(alan.yükseklik),
    );
    let taban = alan.genişlik.min(alan.yükseklik) / 2.0;
    let r0 = seri.yarıçap.0.çöz(taban);
    let r = seri.yarıçap.1.çöz(taban);
    let mut dolgu_açısı = f64::from(seri.dolgu_açısı).to_radians().max(0.0);
    let mut en_küçük_açı = f64::from(seri.en_küçük_açı).to_radians().max(0.0);
    let başlangıç = -f64::from(seri.başlangıç_açısı) * RADYAN;
    // Kilitli ECharts chordLayout, bildirilen endAngle yerine tam tur kullanır.
    let bitiş = başlangıç + std::f64::consts::TAU;
    let (başlangıç, bitiş) = açıları_normalleştir(başlangıç, bitiş, seri.saat_yönünde);
    let toplam_açı = bitiş - başlangıç;
    let yön = if seri.saat_yönünde { 1.0 } else { -1.0 };
    let açık_düğüm_toplamı = kaynak_düğümler
        .iter()
        .filter_map(|düğüm| düğüm.değer.as_ref().and_then(|değer| değer.sayı()))
        .sum::<f64>();
    let bağ_toplamı = kaynak_bağlar
        .iter()
        .map(|(bağ, _, _)| bağ.değer)
        .sum::<f64>();
    let tümü_sıfır = açık_düğüm_toplamı == 0.0 && bağ_toplamı == 0.0;
    let mut değerler = vec![0.0_f64; kaynak_düğümler.len()];
    let mut çizilen_düğüm_sayısı = 0_usize;
    for (bağ, kaynak, hedef) in &kaynak_bağlar {
        let değer = if tümü_sıfır { 1.0 } else { bağ.değer };
        if tümü_sıfır && (değer > 0.0 || en_küçük_açı > 0.0) {
            çizilen_düğüm_sayısı += 2;
        }
        değerler[*kaynak] += değer;
        değerler[*hedef] += değer;
    }
    let mut düğüm_değeri_toplamı = 0.0;
    for (sıra, düğüm) in kaynak_düğümler.iter().enumerate() {
        if let Some(değer) = düğüm
            .değer
            .as_ref()
            .and_then(|değer| değer.sayı())
            .filter(|değer| !değer.is_nan())
        {
            değerler[sıra] = değer.max(değerler[sıra]);
        }
        if !tümü_sıfır && (değerler[sıra] > 0.0 || en_küçük_açı > 0.0) {
            çizilen_düğüm_sayısı += 1;
        }
        düğüm_değeri_toplamı += değerler[sıra];
    }
    if çizilen_düğüm_sayısı == 0 || düğüm_değeri_toplamı == 0.0 {
        return Ok(KirişYerleşimi {
            alan,
            düğümler: Vec::new(),
            bağlar: Vec::new(),
        });
    }
    if dolgu_açısı * çizilen_düğüm_sayısı as f64 >= toplam_açı.abs() {
        dolgu_açısı = ((toplam_açı.abs() - en_küçük_açı * çizilen_düğüm_sayısı as f64)
            / çizilen_düğüm_sayısı as f64)
            .max(0.0);
    }
    if (dolgu_açısı + en_küçük_açı) * çizilen_düğüm_sayısı as f64 >= toplam_açı.abs()
    {
        en_küçük_açı = (toplam_açı.abs() - dolgu_açısı * çizilen_düğüm_sayısı as f64)
            / çizilen_düğüm_sayısı as f64;
    }
    let birim_açı =
        (toplam_açı - dolgu_açısı * çizilen_düğüm_sayısı as f64 * yön) / düğüm_değeri_toplamı;
    let mut açılar = değerler
        .iter()
        .map(|değer| birim_açı * değer * yön)
        .collect::<Vec<_>>();
    let mut oranlar = vec![1.0_f64; açılar.len()];
    let mut toplam_açık = 0.0;
    let mut toplam_fazla = 0.0;
    let mut toplam_fazla_açı = 0.0;
    for açı in &açılar {
        if açı.abs() < en_küçük_açı {
            toplam_açık += en_küçük_açı - açı.abs();
        } else {
            toplam_fazla += açı.abs() - en_küçük_açı;
            toplam_fazla_açı += açı.abs();
        }
    }
    let mut olabildiğince_fazla = false;
    if toplam_açık > toplam_fazla && toplam_fazla > 0.0 {
        let ölçek = toplam_açık / toplam_fazla;
        for (sıra, açı) in açılar.iter_mut().enumerate() {
            let eski = *açı;
            if eski.abs() >= en_küçük_açı {
                *açı = eski * ölçek;
                oranlar[sıra] = ölçek;
            } else {
                *açı = en_küçük_açı;
                oranlar[sıra] = if en_küçük_açı == 0.0 {
                    1.0
                } else {
                    eski / en_küçük_açı
                };
            }
        }
    } else {
        for açı in &açılar {
            if olabildiğince_fazla {
                break;
            }
            let ödünç_oranı = if toplam_fazla_açı > 0.0 {
                (açı / toplam_fazla_açı).min(1.0)
            } else {
                0.0
            };
            if açı - ödünç_oranı * toplam_açık < en_küçük_açı {
                olabildiğince_fazla = true;
            }
        }
    }
    let mut kalan_açık = toplam_açık;
    for (sıra, açı) in açılar.iter_mut().enumerate() {
        if kalan_açık <= 0.0 {
            break;
        }
        let eski = *açı;
        if eski > en_küçük_açı && en_küçük_açı > 0.0 {
            let oran = if olabildiğince_fazla {
                1.0
            } else if toplam_fazla_açı > 0.0 {
                (eski / toplam_fazla_açı).min(1.0)
            } else {
                0.0
            };
            let ödünç = (eski - en_küçük_açı).min(kalan_açık.min(toplam_açık * oran));
            kalan_açık -= ödünç;
            *açı = eski - ödünç;
            oranlar[sıra] = *açı / eski;
        } else if en_küçük_açı > 0.0 {
            *açı = en_küçük_açı;
            oranlar[sıra] = if eski == 0.0 {
                1.0
            } else {
                en_küçük_açı / eski
            };
        }
    }
    let renkler = if seri.renkler.is_empty() {
        (0..kaynak_düğümler.len()).map(palet).collect::<Vec<_>>()
    } else {
        seri.renkler.clone()
    };
    let mut düğümler = Vec::with_capacity(kaynak_düğümler.len());
    let mut açı = başlangıç;
    let mut bağ_birikim_açısı = Vec::with_capacity(kaynak_düğümler.len());
    for (sıra, kaynak) in kaynak_düğümler.iter().enumerate() {
        let yay = açılar[sıra].max(en_küçük_açı);
        let a0 = açı;
        let a1 = açı + yay * yön;
        bağ_birikim_açısı.push(açı);
        açı += (yay + dolgu_açısı) * yön;
        let mut stil = seri.öğe_stili.clone();
        if let Some(yama) = &kaynak.öğe_stili {
            stil = öğe_stili_yama_uygula(&stil, yama);
        }
        let renk = stil.renk.clone().unwrap_or_else(|| {
            Dolgu::Düz(
                renkler
                    .get(sıra % renkler.len().max(1))
                    .copied()
                    .unwrap_or_else(|| palet(sıra)),
            )
        });
        let mut etiket = seri.etiket.clone();
        if let Some(yama) = &kaynak.etiket {
            etiket = yama.uygula(&etiket);
        }
        let (etiket_konumu, etiket_yatay_hizası, etiket_dikey_hizası) =
            etiket_geometrisi(&etiket, merkez, r0, r, a0 as f32, a1 as f32);
        let komşu_bağlar = kaynak_bağlar
            .iter()
            .enumerate()
            .filter_map(|(bağ_sırası, (_, k, h))| (*k == sıra || *h == sıra).then_some(bağ_sırası))
            .collect::<Vec<_>>();
        let mut komşu_düğümler = kaynak_bağlar
            .iter()
            .filter_map(|(_, k, h)| {
                if *k == sıra {
                    Some(*h)
                } else if *h == sıra {
                    Some(*k)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        komşu_düğümler.sort_unstable();
        komşu_düğümler.dedup();
        düğümler.push(KirişYerleşikDüğüm {
            veri_sırası: sıra,
            // createGraphFromNodeEdge, açık `id` yokken düğüm adını etkin
            // graph kimliği olarak kullanır.
            kimlik: Some(kaynak.kimlik.clone().unwrap_or_else(|| kaynak.ad.clone())),
            ad: kaynak.ad.clone(),
            değer: değerler[sıra],
            merkez,
            iç_yarıçap: r0,
            dış_yarıçap: r,
            başlangıç_açısı: a0 as f32,
            bitiş_açısı: a1 as f32,
            saat_yönünde: seri.saat_yönünde,
            renk,
            köşe_yarıçapları: köşe_yarıçapları(&stil, (r - r0).abs()),
            öğe_stili: stil,
            etiket_metni: etiket_metni(&etiket, değerler[sıra], &kaynak.ad, seri.ad.as_deref()),
            etiket,
            etiket_konumu,
            etiket_yatay_hizası,
            etiket_dikey_hizası,
            vurgu: durum_yama_uygula(&seri.vurgu, &kaynak.vurgu),
            bulanık: durum_yama_uygula(&seri.bulanık, &kaynak.bulanık),
            seçili: durum_yama_uygula(&seri.seçili, &kaynak.seçili),
            başlangıçta_seçili: kaynak.başlangıçta_seçili,
            komşu_bağlar,
            komşu_düğümler,
            oran: oranlar[sıra],
        });
    }
    let mut bağlar = Vec::with_capacity(kaynak_bağlar.len());
    for (sıra, (kaynak_bağ, kaynak_sırası, hedef_sırası)) in kaynak_bağlar.iter().enumerate()
    {
        let değer = if tümü_sıfır {
            1.0
        } else {
            kaynak_bağ.değer
        };
        let yay = birim_açı * değer * yön;
        let s0 = bağ_birikim_açısı[*kaynak_sırası];
        let s_yay = (düğümler[*kaynak_sırası].oran * yay).abs();
        let s1 = s0 + s_yay * yön;
        let t0 = bağ_birikim_açısı[*hedef_sırası];
        let t_yay = (düğümler[*hedef_sırası].oran * yay).abs();
        let t1 = t0 + t_yay * yön;
        let nokta = |a: f64| {
            (
                merkez.0 + r0 * a.cos() as f32,
                merkez.1 + r0 * a.sin() as f32,
            )
        };
        let mut stil = seri.çizgi_stili.clone();
        if let Some(yama) = &kaynak_bağ.çizgi_stili {
            stil = çizgi_stili_yama_uygula(&stil, yama);
        }
        let mut kenar_etiketi = seri.kenar_etiketi.clone();
        if let Some(yama) = &kaynak_bağ.kenar_etiketi {
            kenar_etiketi = yama.uygula(&kenar_etiketi);
        }
        let mut yerleşik = KirişYerleşikBağ {
            veri_sırası: sıra,
            kaynak_sırası: *kaynak_sırası,
            hedef_sırası: *hedef_sırası,
            kaynak: kaynak_bağ.kaynak.clone(),
            hedef: kaynak_bağ.hedef.clone(),
            değer,
            kaynak_başlangıç_açısı: s0 as f32,
            kaynak_bitiş_açısı: s1 as f32,
            hedef_başlangıç_açısı: t0 as f32,
            hedef_bitiş_açısı: t1 as f32,
            kaynak1: nokta(s0),
            kaynak2: nokta(s1),
            hedef1: nokta(t0),
            hedef2: nokta(t1),
            merkez,
            yarıçap: r0,
            saat_yönünde: seri.saat_yönünde,
            dolgu: Dolgu::Düz(Renk::SİYAH),
            çizgi_stili: stil,
            etiket_metni: kenar_etiketi.biçimleyici.as_ref().map_or_else(
                || sayı_metni(kaynak_bağ.değer),
                |b| {
                    b.uygula_bağlamla(
                        kaynak_bağ.değer,
                        &sayı_metni(kaynak_bağ.değer),
                        seri.ad.as_deref().unwrap_or_default(),
                        &format!("{} > {}", kaynak_bağ.kaynak, kaynak_bağ.hedef),
                    )
                },
            ),
            kenar_etiketi,
            etiket_konumu: (
                (nokta((s0 + s1) / 2.0).0 + nokta((t0 + t1) / 2.0).0) / 2.0,
                (nokta((s0 + s1) / 2.0).1 + nokta((t0 + t1) / 2.0).1) / 2.0,
            ),
            vurgu: durum_yama_uygula(&seri.vurgu, &kaynak_bağ.vurgu),
            bulanık: durum_yama_uygula(&seri.bulanık, &kaynak_bağ.bulanık),
            seçili: durum_yama_uygula(&seri.seçili, &kaynak_bağ.seçili),
        };
        let boya = yerleşik
            .çizgi_stili
            .renk
            .clone()
            .unwrap_or(KirişKenarBoyası::Kaynak);
        yerleşik.dolgu = bağ_dolgusu(
            &boya,
            yerleşik.çizgi_stili.opaklık.unwrap_or(0.2),
            &düğümler[*kaynak_sırası].renk,
            &düğümler[*hedef_sırası].renk,
            &yerleşik,
        );
        bağ_birikim_açısı[*kaynak_sırası] = s1;
        bağ_birikim_açısı[*hedef_sırası] = t1;
        bağlar.push(yerleşik);
    }
    Ok(KirişYerleşimi {
        alan,
        düğümler,
        bağlar,
    })
}

#[allow(clippy::too_many_arguments)]
fn etiketi_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    metin: &str,
    etiket: &Etiket,
    konum: (f32, f32),
    yatay: YatayHiza,
    dikey: DikeyHiza,
    varsayılan: Renk,
    opaklık: f32,
) {
    if !etiket.göster || metin.is_empty() {
        return;
    }
    let renk = etiket
        .yazı
        .renk
        .unwrap_or(varsayılan)
        .opaklık(etiket.yazı.opaklık.unwrap_or(1.0) * opaklık);
    çizici.aileli_yazı(
        metin,
        konum,
        yatay,
        dikey,
        etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
        renk,
        etiket.yazı.kalın,
        etiket.yazı.aile.as_deref().unwrap_or("sans-serif"),
    );
}

fn halka_içeriyor(düğüm: &KirişYerleşikDüğüm, nokta: (f32, f32)) -> bool {
    İsabetGeometrisi::Halka {
        merkez: düğüm.merkez,
        iç_yarıçap: düğüm.iç_yarıçap,
        dış_yarıçap: düğüm.dış_yarıçap,
        açı0: düğüm.başlangıç_açısı,
        açı1: düğüm.bitiş_açısı,
    }
    .içeriyor_mu(nokta)
}

enum Vurgulu {
    Düğüm(usize),
    Bağ(usize),
}

/// Kirişi çizer; fare verilirse resmî `focus: self/adjacency/series`
/// ilişkileri normal/vurgu/bulanık durum katmanlarına uygulanır.
#[allow(clippy::too_many_arguments)]
pub fn kiriş_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &KirişSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let Ok(mut yerleşim) = kiriş_yerleşimi(seri, tuval, palet) else {
        return;
    };
    let ilerleme = ilerleme.clamp(0.0, 1.0);
    if ilerleme < 1.0 {
        for düğüm in &mut yerleşim.düğümler {
            düğüm.bitiş_açısı =
                düğüm.başlangıç_açısı + (düğüm.bitiş_açısı - düğüm.başlangıç_açısı) * ilerleme;
        }
    }
    let vurgulu = fare
        .and_then(|nokta| {
            yerleşim
                .düğümler
                .iter()
                .find(|d| halka_içeriyor(d, nokta))
                .map(|d| Vurgulu::Düğüm(d.veri_sırası))
                .or_else(|| {
                    yerleşim
                        .bağlar
                        .iter()
                        .rev()
                        .find(|b| {
                            İsabetGeometrisi::Çokgen {
                                noktalar: şerit_çokgeni(b, b.çizgi_stili.eğrilik.unwrap_or(0.7)),
                            }
                            .içeriyor_mu(nokta)
                        })
                        .map(|b| Vurgulu::Bağ(b.veri_sırası))
                })
        })
        .filter(|vurgulu| match *vurgulu {
            Vurgulu::Düğüm(sıra) => !yerleşim.düğümler[sıra].vurgu.devre_dışı.unwrap_or(false),
            Vurgulu::Bağ(sıra) => !yerleşim.bağlar[sıra].vurgu.devre_dışı.unwrap_or(false),
        });
    let odak = match vurgulu {
        Some(Vurgulu::Düğüm(sıra)) => yerleşim
            .düğümler
            .get(sıra)
            .and_then(|d| d.vurgu.odak)
            .unwrap_or(KirişVurguOdağı::Komşuluk),
        Some(Vurgulu::Bağ(sıra)) => yerleşim
            .bağlar
            .get(sıra)
            .and_then(|b| b.vurgu.odak)
            .unwrap_or(KirişVurguOdağı::Komşuluk),
        None => KirişVurguOdağı::Yok,
    };
    let düğüm_odakta = |sıra: usize| match vurgulu {
        None => true,
        Some(Vurgulu::Düğüm(v)) => match odak {
            KirişVurguOdağı::Yok | KirişVurguOdağı::Seri => true,
            KirişVurguOdağı::Kendisi => sıra == v,
            KirişVurguOdağı::Komşuluk => {
                sıra == v || yerleşim.düğümler[v].komşu_düğümler.contains(&sıra)
            }
        },
        Some(Vurgulu::Bağ(v)) => match odak {
            KirişVurguOdağı::Yok | KirişVurguOdağı::Seri => true,
            KirişVurguOdağı::Kendisi => {
                sıra == yerleşim.bağlar[v].kaynak_sırası || sıra == yerleşim.bağlar[v].hedef_sırası
            }
            KirişVurguOdağı::Komşuluk => yerleşim.düğümler[sıra].komşu_bağlar.iter().any(|b| {
                *b == v
                    || yerleşim.bağlar[v].kaynak_sırası == sıra
                    || yerleşim.bağlar[v].hedef_sırası == sıra
            }),
        },
    };
    let bağ_odakta = |sıra: usize| match vurgulu {
        None => true,
        Some(Vurgulu::Düğüm(v)) => match odak {
            KirişVurguOdağı::Yok | KirişVurguOdağı::Seri => true,
            KirişVurguOdağı::Kendisi | KirişVurguOdağı::Komşuluk => {
                yerleşim.düğümler[v].komşu_bağlar.contains(&sıra)
            }
        },
        Some(Vurgulu::Bağ(v)) => match odak {
            KirişVurguOdağı::Yok | KirişVurguOdağı::Seri => true,
            KirişVurguOdağı::Kendisi => sıra == v,
            KirişVurguOdağı::Komşuluk => {
                let b = &yerleşim.bağlar[v];
                let a = &yerleşim.bağlar[sıra];
                [b.kaynak_sırası, b.hedef_sırası]
                    .iter()
                    .any(|n| *n == a.kaynak_sırası || *n == a.hedef_sırası)
            }
        },
    };
    // Şeritler sektörlerin altında kalır; taban çizgisi/sektör kenarlığı
    // bağlantı dolgusunca örtülmez.
    for bağ in &yerleşim.bağlar {
        let vurgulu_bu = matches!(vurgulu, Some(Vurgulu::Bağ(s)) if s == bağ.veri_sırası);
        let odakta = bağ_odakta(bağ.veri_sırası);
        let durum = if vurgulu_bu {
            Some(&bağ.vurgu)
        } else if !odakta {
            Some(&bağ.bulanık)
        } else {
            None
        };
        let stil = durum.and_then(|d| d.çizgi_stili.as_ref()).map_or_else(
            || bağ.çizgi_stili.clone(),
            |y| çizgi_stili_yama_uygula(&bağ.çizgi_stili, y),
        );
        let bir_opaklık = if !odakta && durum.and_then(|d| d.çizgi_stili.as_ref()).is_none() {
            0.1
        } else {
            1.0
        };
        let dolgu = bağ.dolgu.opaklık(bir_opaklık * ilerleme);
        let yol = şerit_yolu(bağ, stil.eğrilik.unwrap_or(0.7));
        if let (Some(bulanıklık), Some(renk)) = (stil.gölge_bulanıklığı, stil.gölge_rengi)
            && bulanıklık > 0.0
        {
            çizici.yol_gölgesi(
                &yol,
                renk,
                bulanıklık,
                stil.gölge_kayması.unwrap_or((0.0, 0.0)),
            );
        }
        çizici.yol_doldur(&yol, &dolgu);
        if stil.kalınlık.unwrap_or(0.0) > 0.0 {
            çizici.yol_çiz(
                &yol,
                stil.kalınlık.unwrap_or(0.0),
                dolgu.temsilî(),
                stil.tür.unwrap_or(ÇizgiTürü::Düz),
            );
        }
        let etiket = durum.and_then(|d| d.kenar_etiketi.as_ref()).map_or_else(
            || bağ.kenar_etiketi.clone(),
            |y| y.uygula(&bağ.kenar_etiketi),
        );
        etiketi_çiz(
            çizici,
            &bağ.etiket_metni,
            &etiket,
            bağ.etiket_konumu,
            YatayHiza::Orta,
            DikeyHiza::Orta,
            tema::birincil_metin(),
            bir_opaklık,
        );
        if !seri.sessiz {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: bağ.veri_sırası,
                seri_adı: seri.ad.clone(),
                ad: Some(format!("{} > {}", bağ.kaynak, bağ.hedef)),
                değer: Some(bağ.değer),
                geometri: İsabetGeometrisi::Çokgen {
                    noktalar: şerit_çokgeni(bağ, stil.eğrilik.unwrap_or(0.7)),
                },
            });
        }
    }
    for düğüm in &yerleşim.düğümler {
        let vurgulu_bu = matches!(vurgulu, Some(Vurgulu::Düğüm(s)) if s == düğüm.veri_sırası);
        let odakta = düğüm_odakta(düğüm.veri_sırası);
        let durum = if vurgulu_bu {
            Some(&düğüm.vurgu)
        } else if !odakta {
            Some(&düğüm.bulanık)
        } else if düğüm.başlangıçta_seçili {
            Some(&düğüm.seçili)
        } else {
            None
        };
        let stil = durum.and_then(|d| d.öğe_stili.as_ref()).map_or_else(
            || düğüm.öğe_stili.clone(),
            |y| öğe_stili_yama_uygula(&düğüm.öğe_stili, y),
        );
        let opaklık = stil.opaklık.unwrap_or(1.0)
            * if !odakta && durum.and_then(|d| d.öğe_stili.as_ref()).is_none() {
                0.1
            } else {
                1.0
            };
        let dolgu = stil.renk.as_ref().unwrap_or(&düğüm.renk).opaklık(opaklık);
        let köşeler = köşe_yarıçapları(&stil, (düğüm.dış_yarıçap - düğüm.iç_yarıçap).abs());
        let kenarlık = stil
            .kenarlık_rengi
            .map(|r| (stil.kenarlık_kalınlığı.unwrap_or(1.0), r.opaklık(opaklık)))
            .filter(|(k, _)| *k > 0.0);
        çizici.yuvarlatılmış_dilim(
            düğüm.merkez,
            düğüm.iç_yarıçap,
            düğüm.dış_yarıçap,
            düğüm.başlangıç_açısı,
            düğüm.bitiş_açısı,
            köşeler,
            &dolgu,
            kenarlık,
        );
        let etiket = durum
            .and_then(|d| d.etiket.as_ref())
            .map_or_else(|| düğüm.etiket.clone(), |y| y.uygula(&düğüm.etiket));
        etiketi_çiz(
            çizici,
            &düğüm.etiket_metni,
            &etiket,
            düğüm.etiket_konumu,
            düğüm.etiket_yatay_hizası,
            düğüm.etiket_dikey_hizası,
            dolgu.temsilî(),
            if odakta { 1.0 } else { 0.1 },
        );
        if !seri.sessiz {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: düğüm.veri_sırası,
                seri_adı: seri.ad.clone(),
                ad: Some(düğüm.ad.clone()),
                değer: Some(düğüm.değer),
                geometri: İsabetGeometrisi::Halka {
                    merkez: düğüm.merkez,
                    iç_yarıçap: düğüm.iç_yarıçap,
                    dış_yarıçap: düğüm.dış_yarıçap,
                    açı0: düğüm.başlangıç_açısı,
                    açı1: düğüm.bitiş_açısı,
                },
            });
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;

    #[test]
    fn resmi_baslangic_yon_ve_bag_birikimi_korunur() {
        let seri = KirişSerisi::yeni()
            .düğümler(["A", "B", "C"])
            .bağlar([("A", "B", 40.0), ("A", "C", 20.0)]);
        let yerleşim = kiriş_yerleşimi(&seri, Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0), &|i| {
            [
                Renk::from("#5470c6"),
                Renk::from("#91cc75"),
                Renk::from("#fac858"),
            ][i % 3]
        })
        .expect("yerleşim");
        assert_eq!(yerleşim.düğümler.len(), 3);
        assert_eq!(yerleşim.bağlar.len(), 2);
        assert!(
            (yerleşim.düğümler[0].başlangıç_açısı - std::f32::consts::FRAC_PI_2 * 3.0).abs() < 1e-5
        );
        assert!(
            (yerleşim.bağlar[0].kaynak_bitiş_açısı - yerleşim.bağlar[1].kaynak_başlangıç_açısı)
                .abs()
                < 1e-5
        );
    }

    #[test]
    fn min_angle_bagsiz_dugumleri_de_cizer() {
        let seri = KirişSerisi::yeni()
            .en_küçük_açı(30.0)
            .düğümler(["A", "B", "C", "D"])
            .bağlar([("A", "B", 1.0)]);
        let yerleşim = kiriş_yerleşimi(&seri, Dikdörtgen::yeni(0.0, 0.0, 600.0, 450.0), &|_| {
            Renk::from("#5470c6")
        })
        .expect("yerleşim");
        assert_eq!(yerleşim.düğümler.len(), 4);
        assert!(yerleşim.düğümler[2].değer == 0.0);
        assert!(
            (yerleşim.düğümler[2].bitiş_açısı - yerleşim.düğümler[2].başlangıç_açısı)
                .abs()
                .to_degrees()
                >= 29.999
        );
    }

    #[test]
    fn serit_boyali_alani_olay_ve_tooltip_isabeti_uretir() {
        let seri = KirişSerisi::yeni()
            .düğümler(["A", "B"])
            .bağlar([("A", "B", 10.0)]);
        let tuval = Dikdörtgen::yeni(0.0, 0.0, 600.0, 450.0);
        let palet = |sıra| [Renk::from("#5470c6"), Renk::from("#91cc75")][sıra % 2];
        let yerleşim = kiriş_yerleşimi(&seri, tuval, &palet).expect("yerleşim");
        let bağ = &yerleşim.bağlar[0];
        let kaynak_ortası = (bağ.kaynak_başlangıç_açısı + bağ.kaynak_bitiş_açısı) / 2.0;
        let iç_nokta = (
            bağ.merkez.0 + (bağ.yarıçap - 1.0) * kaynak_ortası.cos(),
            bağ.merkez.1 + (bağ.yarıçap - 1.0) * kaynak_ortası.sin(),
        );
        let geometri = İsabetGeometrisi::Çokgen {
            noktalar: şerit_çokgeni(bağ, 0.7),
        };
        assert!(geometri.içeriyor_mu(iç_nokta));

        let mut yüzey = KayıtYüzeyi::yeni(600.0, 450.0);
        let mut isabetler = Vec::new();
        kiriş_çiz(
            &mut yüzey,
            &seri,
            0,
            tuval,
            &palet,
            1.0,
            Some(iç_nokta),
            &mut isabetler,
        );
        assert_eq!(isabetler.len(), 3, "bir şerit ve iki sektör");
        assert_eq!(isabetler[0].ad.as_deref(), Some("A > B"));
        assert!(matches!(
            isabetler[0].geometri,
            İsabetGeometrisi::Çokgen { .. }
        ));
    }

    #[test]
    fn acik_curveness_kenar_etiketi_ve_disabled_vurgu_dallari_calisir() {
        let seri = KirişSerisi::yeni()
            .düğümler([
                KirişDüğümü::yeni("A").vurgu(
                    KirişDurumu::yeni()
                        .devre_dışı(true)
                        .öğe_stili(KirişÖğeStili::yeni().renk("#ff0000")),
                ),
                KirişDüğümü::yeni("B"),
            ])
            .ayrıntılı_bağlar([
                KirişBağı::yeni("A", "B", 10.0).çizgi_stili(KirişÇizgiStili::yeni().eğrilik(0.2))
            ])
            .kenar_etiketi(Etiket::yeni().göster(true).biçimleyici("bağ {c}"));
        let tuval = Dikdörtgen::yeni(0.0, 0.0, 600.0, 450.0);
        let palet = |sıra| [Renk::from("#5470c6"), Renk::from("#91cc75")][sıra % 2];
        let yerleşim = kiriş_yerleşimi(&seri, tuval, &palet).expect("yerleşim");
        let bağ = &yerleşim.bağlar[0];
        assert_eq!(bağ.çizgi_stili.eğrilik, Some(0.2));
        assert_eq!(bağ.etiket_metni, "bağ 10");
        assert_ne!(şerit_çokgeni(bağ, 0.2), şerit_çokgeni(bağ, 0.7));

        let düğüm = &yerleşim.düğümler[0];
        let açı = (düğüm.başlangıç_açısı + düğüm.bitiş_açısı) / 2.0;
        let fare = (
            düğüm.merkez.0 + (düğüm.iç_yarıçap + düğüm.dış_yarıçap) / 2.0 * açı.cos(),
            düğüm.merkez.1 + (düğüm.iç_yarıçap + düğüm.dış_yarıçap) / 2.0 * açı.sin(),
        );
        let mut normal = KayıtYüzeyi::yeni(600.0, 450.0);
        let mut devre_dışı_vurgu = KayıtYüzeyi::yeni(600.0, 450.0);
        kiriş_çiz(
            &mut normal,
            &seri,
            0,
            tuval,
            &palet,
            1.0,
            None,
            &mut Vec::new(),
        );
        kiriş_çiz(
            &mut devre_dışı_vurgu,
            &seri,
            0,
            tuval,
            &palet,
            1.0,
            Some(fare),
            &mut Vec::new(),
        );
        assert_eq!(normal.döküm(), devre_dışı_vurgu.döküm());
        assert!(normal.döküm().contains("bağ 10"));
    }
}
