//! Güneş patlaması (sunburst) — `echarts/src/chart/sunburst` karşılığı.
//!
//! Yerleşim, `sunburstLayout.ts`; görsel kalıtımı, `sunburstVisual.ts`;
//! sektör ve etiket davranışı ise `SunburstPiece.ts` ile aynı veri ağacını
//! izler. Hesaplanan [`GüneşDilimi`] listesi rasterdan bağımsız yapısal
//! doğrulamanın da tek doğruluk noktasıdır.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::yuzey::yuvarlatılmış_dilim_yolu;
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::grafik::pasta::zengin_etiketi_hizalı_yaz;
use crate::koordinat::Dikdörtgen;
use crate::model::Uzunluk;
use crate::model::agac::{
    AğaçDüğümü, AğaçVurguOdağı, GüneşPatlamasıDurumu, GüneşPatlamasıRenkKaynağı,
    GüneşPatlamasıSeviyesi, GüneşPatlamasıSırası, GüneşPatlamasıÖğeStili,
};
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::seri::{
    GüneşPatlamasıEtiketParametreleri, GüneşPatlamasıSerisi, GüneşPatlamasıSıralamaParametreleri,
    GüneşPatlamasıYolBilgisi,
};
use crate::model::stil::{
    Etiket, EtiketDöndürme, EtiketKonumu, EtiketYaması, YazıDikeyHizası, YazıYatayHizası,
    zengin_metin_içeriği, ÇizgiTürü,
};
use crate::model::veri_kumesi::BoyutSeçici;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

const RADYAN: f64 = std::f64::consts::PI / 180.0;

/// zrender `color.lift(color, level)` pozitif kolu. Zrender her RGB
/// kanalını bit düzeyi tamsayıya indirger; düz kayan nokta karışımı son
/// rasterda bazı kanalları bir piksel değeri fazla yuvarlar.
fn zrender_rengi_aç(renk: Renk, oran: f32) -> Renk {
    let oran = oran.clamp(0.0, 1.0);
    let aç = |kanal: f32| {
        let kanal = (kanal.clamp(0.0, 1.0) * 255.0).round();
        (kanal + (255.0 - kanal) * oran).floor() / 255.0
    };
    Renk {
        kırmızı: aç(renk.kırmızı),
        yeşil: aç(renk.yeşil),
        mavi: aç(renk.mavi),
        alfa: renk.alfa,
    }
}

#[derive(Clone, Copy, Debug)]
struct DüğümBilgisi {
    veri_sırası: usize,
    derinlik: usize,
    yükseklik: usize,
    değer: f64,
    üst_sıra: usize,
}

#[derive(Clone, Debug)]
struct ÇözülmüşKatman {
    öğe_stili: GüneşPatlamasıÖğeStili,
    etiket: Etiket,
    vurgu: GüneşPatlamasıDurumu,
    bulanık: GüneşPatlamasıDurumu,
    seçili: GüneşPatlamasıDurumu,
}

/// Yerleşimi, kalıtılmış stili ve etiket geometrisi çözülmüş sektör.
#[derive(Clone, Debug)]
pub struct GüneşDilimi {
    pub veri_sırası: usize,
    pub ad: String,
    pub değer: f64,
    /// Ham ağacın mutlak derinliği; ilk veri halkası `1`dir.
    pub derinlik: usize,
    pub yol: Vec<String>,
    pub ata_sıraları: Vec<usize>,
    pub merkez: (f32, f32),
    pub iç_yarıçap: f32,
    pub dış_yarıçap: f32,
    pub açı0: f32,
    pub açı1: f32,
    pub saat_yönünde: bool,
    pub dolgu: Dolgu,
    pub öğe_stili: GüneşPatlamasıÖğeStili,
    pub etiket: Etiket,
    pub etiket_metni: String,
    pub etiket_konumu: (f32, f32),
    pub etiket_dönüşü: f32,
    pub etiket_yatay_hizası: YatayHiza,
    pub etiket_dikey_hizası: DikeyHiza,
    pub vurgu: GüneşPatlamasıDurumu,
    pub bulanık: GüneşPatlamasıDurumu,
    pub seçili: GüneşPatlamasıDurumu,
    pub bağlantı: Option<String>,
    pub hedef: Option<String>,
}

/// Rasterdan bağımsız Sunburst geometri kanıtı.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GüneşPatlamasıSahneÖzeti {
    pub dilim_sayısı: usize,
    pub etiket_sayısı: usize,
    pub koordinat_sayısı: usize,
    pub fnv1a_64: u64,
}

fn öğe_stili_yama_uygula(
    taban: &GüneşPatlamasıÖğeStili,
    yama: &GüneşPatlamasıÖğeStili,
) -> GüneşPatlamasıÖğeStili {
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

fn ortak_öğe_stili_yaması(düğüm: &AğaçDüğümü) -> Option<GüneşPatlamasıÖğeStili> {
    let stil = düğüm.öğe_stili.as_ref()?;
    let mut yama = GüneşPatlamasıÖğeStili::yeni();
    yama.renk.clone_from(&stil.renk);
    yama.kenarlık_rengi = stil.kenarlık_rengi;
    if stil.kenarlık_kalınlığı > 0.0 {
        yama.kenarlık_kalınlığı = Some(stil.kenarlık_kalınlığı);
    }
    yama.kenarlık_türü = Some(stil.kenarlık_türü);
    if stil.kenarlık_yarıçapı != [0.0; 4] {
        yama.kenarlık_yarıçapı = Some(crate::model::agac::GüneşPatlamasıKöşeYarıçapı(
            stil.kenarlık_yarıçapı.map(Uzunluk::Piksel),
        ));
    }
    yama.opaklık = stil.opaklık;
    if stil.gölge_bulanıklığı > 0.0 {
        yama.gölge_bulanıklığı = Some(stil.gölge_bulanıklığı);
        yama.gölge_rengi = stil.gölge_rengi;
        yama.gölge_kayması = Some(stil.gölge_kayması);
    }
    Some(yama)
}

fn durum_yama_uygula(
    taban: &GüneşPatlamasıDurumu,
    yama: &GüneşPatlamasıDurumu,
) -> GüneşPatlamasıDurumu {
    let mut sonuç = taban.clone();
    if let Some(stil) = &yama.öğe_stili {
        sonuç.öğe_stili = Some(match &sonuç.öğe_stili {
            Some(taban) => öğe_stili_yama_uygula(taban, stil),
            None => stil.clone(),
        });
    }
    if let Some(etiket) = &yama.etiket {
        sonuç.etiket = Some(match &sonuç.etiket {
            Some(taban) => etiket_yaması_yama_uygula(taban, etiket),
            None => etiket.clone(),
        });
    }
    if yama.odak.is_some() {
        sonuç.odak = yama.odak;
    }
    sonuç
}

fn etiket_yaması_yama_uygula(taban: &EtiketYaması, yama: &EtiketYaması) -> EtiketYaması {
    let mut sonuç = taban.clone();
    macro_rules! alan {
        ($ad:ident) => {
            if yama.$ad.is_some() {
                sonuç.$ad.clone_from(&yama.$ad);
            }
        };
    }
    alan!(göster);
    alan!(sessiz);
    alan!(konum);
    alan!(kayma);
    alan!(biçimleyici);
    alan!(yazı);
    if let Some(zengin) = &yama.zengin {
        sonuç
            .zengin
            .get_or_insert_with(Default::default)
            .extend(zengin.clone());
    }
    alan!(uzaklık);
    alan!(dış_hiza);
    alan!(kenar_uzaklığı);
    alan!(taşma_payını);
    alan!(çizgi_uzaklığı);
    alan!(kenar_boşluğu);
    alan!(en_küçük_boşluk);
    alan!(döndürme);
    alan!(en_küçük_açı);
    alan!(yatay_hiza);
    alan!(dikey_hiza);
    sonuç
}

fn seviye_uygula(
    taban: &ÇözülmüşKatman, seviye: &GüneşPatlamasıSeviyesi
) -> ÇözülmüşKatman {
    let mut sonuç = taban.clone();
    if let Some(stil) = &seviye.öğe_stili {
        sonuç.öğe_stili = öğe_stili_yama_uygula(&sonuç.öğe_stili, stil);
    }
    if let Some(etiket) = &seviye.etiket {
        sonuç.etiket = etiket.uygula(&sonuç.etiket);
    }
    sonuç.vurgu = durum_yama_uygula(&sonuç.vurgu, &seviye.vurgu);
    sonuç.bulanık = durum_yama_uygula(&sonuç.bulanık, &seviye.bulanık);
    sonuç.seçili = durum_yama_uygula(&sonuç.seçili, &seviye.seçili);
    sonuç
}

fn düğüm_uygula(
    taban: &ÇözülmüşKatman, düğüm: &AğaçDüğümü
) -> ÇözülmüşKatman {
    let mut sonuç = taban.clone();
    if let Some(stil) = ortak_öğe_stili_yaması(düğüm) {
        sonuç.öğe_stili = öğe_stili_yama_uygula(&sonuç.öğe_stili, &stil);
    }
    if let Some(stil) = &düğüm.güneş_patlaması_öğe_stili {
        sonuç.öğe_stili = öğe_stili_yama_uygula(&sonuç.öğe_stili, stil);
    }
    if let Some(renk) = düğüm.renk {
        sonuç.öğe_stili.renk = Some(Dolgu::Düz(renk));
    }
    if let Some(etiket) = &düğüm.etiket {
        sonuç.etiket = etiket.uygula(&sonuç.etiket);
    }
    if let Some(durum) = &düğüm.güneş_patlaması_vurgusu {
        sonuç.vurgu = durum_yama_uygula(&sonuç.vurgu, durum);
    }
    if let Some(durum) = &düğüm.güneş_patlaması_bulanıklığı {
        sonuç.bulanık = durum_yama_uygula(&sonuç.bulanık, durum);
    }
    if let Some(durum) = &düğüm.güneş_patlaması_seçilisi {
        sonuç.seçili = durum_yama_uygula(&sonuç.seçili, durum);
    }
    sonuç
}

fn seri_katmanı(seri: &GüneşPatlamasıSerisi) -> ÇözülmüşKatman {
    ÇözülmüşKatman {
        öğe_stili: seri.öğe_stili.clone(),
        etiket: seri.etiket.clone(),
        vurgu: seri.vurgu.clone(),
        bulanık: seri.bulanık.clone(),
        seçili: seri.seçili.clone(),
    }
}

fn bilgi_haritası(
    seri: &GüneşPatlamasıSerisi
) -> (HashMap<usize, DüğümBilgisi>, f64, usize) {
    fn gez(
        düğüm: &AğaçDüğümü,
        derinlik: usize,
        üst_sıra: usize,
        sıra: &mut usize,
        çıktı: &mut HashMap<usize, DüğümBilgisi>,
    ) -> (f64, usize) {
        let veri_sırası = *sıra;
        *sıra = sıra.saturating_add(1);
        let mut çocuk_toplamı = 0.0;
        let mut en_büyük_çocuk_yüksekliği = 0usize;
        for çocuk in &düğüm.çocuklar {
            let (değer, çocuk_yüksekliği) = gez(çocuk, derinlik + 1, üst_sıra, sıra, çıktı);
            çocuk_toplamı += değer;
            en_büyük_çocuk_yüksekliği = en_büyük_çocuk_yüksekliği.max(çocuk_yüksekliği);
        }
        // ECharts TreeNode.height yaprakta 1'dir; alt ağacın düğüm
        // yüksekliğini (kenar sayısını değil) tutar.
        let yükseklik = en_büyük_çocuk_yüksekliği.saturating_add(1);
        let ham = düğüm
            .değer
            .or_else(|| düğüm.değerler.first().copied().flatten());
        let değer = ham
            .filter(|değer| değer.is_finite())
            .unwrap_or(çocuk_toplamı)
            .max(0.0);
        çıktı.insert(
            std::ptr::from_ref(düğüm).addr(),
            DüğümBilgisi {
                veri_sırası,
                derinlik,
                yükseklik,
                değer,
                üst_sıra,
            },
        );
        (değer, yükseklik)
    }

    let mut çıktı = HashMap::new();
    let mut sıra = 0usize;
    let mut toplam = 0.0;
    let mut sanal_yükseklik = 0usize;
    for (üst_sıra, düğüm) in seri.kökler.iter().enumerate() {
        let (değer, yükseklik) = gez(düğüm, 1, üst_sıra, &mut sıra, &mut çıktı);
        toplam += değer;
        sanal_yükseklik = sanal_yükseklik.max(yükseklik.saturating_add(1));
    }
    (çıktı, toplam, sanal_yükseklik)
}

fn görünüm_kökü<'a>(
    kökler: &'a [AğaçDüğümü],
    yol: &[String],
) -> (Option<&'a AğaçDüğümü>, usize) {
    let mut çocuklar = kökler;
    let mut kök = None;
    let mut uzunluk = 0usize;
    for ad in yol {
        let Some(düğüm) = çocuklar.iter().find(|düğüm| &düğüm.ad == ad) else {
            break;
        };
        kök = Some(düğüm);
        uzunluk = uzunluk.saturating_add(1);
        çocuklar = &düğüm.çocuklar;
    }
    (kök, uzunluk)
}

fn sıralı_çocuklar<'a>(
    seri: &GüneşPatlamasıSerisi,
    düğümler: &'a [AğaçDüğümü],
    bilgiler: &HashMap<usize, DüğümBilgisi>,
) -> Vec<&'a AğaçDüğümü> {
    let mut çıktı = düğümler.iter().collect::<Vec<_>>();
    let bilgi = |düğüm: &AğaçDüğümü| {
        bilgiler
            .get(&std::ptr::from_ref(düğüm).addr())
            .copied()
            .unwrap_or(DüğümBilgisi {
                veri_sırası: 0,
                derinlik: 0,
                yükseklik: 0,
                değer: 0.0,
                üst_sıra: 0,
            })
    };
    if let Some(işlev) = &seri.sıralama_işlevi {
        çıktı.sort_by(|a, b| {
            let a = bilgi(a);
            let b = bilgi(b);
            işlev.karşılaştır(
                &GüneşPatlamasıSıralamaParametreleri {
                    veri_sırası: a.veri_sırası,
                    derinlik: a.derinlik,
                    yükseklik: a.yükseklik,
                    değer: a.değer,
                },
                &GüneşPatlamasıSıralamaParametreleri {
                    veri_sırası: b.veri_sırası,
                    derinlik: b.derinlik,
                    yükseklik: b.yükseklik,
                    değer: b.değer,
                },
            )
        });
        return çıktı;
    }
    match seri.sıralama {
        GüneşPatlamasıSırası::Azalan => çıktı.sort_by(|a, b| {
            let a = bilgi(a);
            let b = bilgi(b);
            b.değer
                .partial_cmp(&a.değer)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.veri_sırası.cmp(&b.veri_sırası))
        }),
        GüneşPatlamasıSırası::Artan => çıktı.sort_by(|a, b| {
            let a = bilgi(a);
            let b = bilgi(b);
            a.değer
                .partial_cmp(&b.değer)
                .unwrap_or(Ordering::Equal)
                .then_with(|| b.veri_sırası.cmp(&a.veri_sırası))
        }),
        GüneşPatlamasıSırası::Veri => {}
    }
    çıktı
}

fn görsel_değer(
    düğüm: &AğaçDüğümü, bilgi: DüğümBilgisi, eşleme: &GörselEşleme
) -> f64 {
    match eşleme.boyut.as_ref() {
        Some(BoyutSeçici::Sıra(sıra)) => düğüm
            .değerler
            .get(*sıra)
            .copied()
            .flatten()
            .filter(|değer| değer.is_finite())
            .unwrap_or(bilgi.değer),
        Some(BoyutSeçici::Ad(ad)) if ad != "value" => bilgi.değer,
        _ => bilgi.değer,
    }
}

fn etiket_geometrisi(
    etiket: &Etiket,
    merkez: (f32, f32),
    iç: f32,
    dış: f32,
    açı0: f32,
    açı1: f32,
) -> ((f32, f32), f32, YatayHiza, DikeyHiza) {
    let orta = (açı0 + açı1) / 2.0;
    let (sinüs, kosinüs) = orta.sin_cos();
    let normalize = |açı: f32| açı.rem_euclid(std::f32::consts::TAU);
    let normal = normalize(match etiket.döndürme {
        EtiketDöndürme::Teğetsel | EtiketDöndürme::TeğetselÇevirmesiz => {
            std::f32::consts::FRAC_PI_2 - orta
        }
        _ => orta,
    });
    let çevrilsin =
        normal > std::f32::consts::FRAC_PI_2 + 1e-7 && normal < std::f32::consts::PI * 1.5 - 1e-7;

    let açık_hiza = etiket.yatay_hiza;
    let (yarıçap, doğal_hiza) = if etiket.konum == EtiketKonumu::Dış {
        (
            dış + etiket.uzaklık,
            if çevrilsin {
                YatayHiza::Sağ
            } else {
                YatayHiza::Sol
            },
        )
    } else {
        match açık_hiza {
            Some(YazıYatayHizası::Sol) => (
                iç + etiket.uzaklık,
                if çevrilsin {
                    YatayHiza::Sağ
                } else {
                    YatayHiza::Sol
                },
            ),
            Some(YazıYatayHizası::Sağ) => (
                dış - etiket.uzaklık,
                if çevrilsin {
                    YatayHiza::Sol
                } else {
                    YatayHiza::Sağ
                },
            ),
            _ if iç == 0.0 && ((açı1 - açı0).abs() - std::f32::consts::TAU).abs() <= 1e-4 => {
                (0.0, YatayHiza::Orta)
            }
            _ => ((iç + dış) / 2.0, YatayHiza::Orta),
        }
    };
    let yatay = doğal_hiza;
    let dikey = match etiket.dikey_hiza {
        Some(YazıDikeyHizası::Üst) => DikeyHiza::Üst,
        Some(YazıDikeyHizası::Alt) => DikeyHiza::Alt,
        _ => DikeyHiza::Orta,
    };
    let dönüş = match etiket.döndürme {
        EtiketDöndürme::Radyal => {
            normalize(-orta)
                + if çevrilsin {
                    std::f32::consts::PI
                } else {
                    0.0
                }
        }
        EtiketDöndürme::Teğetsel => {
            normalize(std::f32::consts::FRAC_PI_2 - orta)
                + if çevrilsin {
                    std::f32::consts::PI
                } else {
                    0.0
                }
        }
        EtiketDöndürme::TeğetselÇevirmesiz => normalize(std::f32::consts::FRAC_PI_2 - orta),
        EtiketDöndürme::Derece(derece) => derece.to_radians(),
        EtiketDöndürme::Yok => 0.0,
    };
    (
        (
            merkez.0 + yarıçap * kosinüs + etiket.kayma.0,
            merkez.1 + yarıçap * sinüs + etiket.kayma.1,
        ),
        normalize(dönüş),
        yatay,
        dikey,
    )
}

#[allow(clippy::too_many_arguments)]
fn düğümü_yerleştir(
    seri: &GüneşPatlamasıSerisi,
    düğüm: &AğaçDüğümü,
    bilgiler: &HashMap<usize, DüğümBilgisi>,
    merkez: (f32, f32),
    boyut: f32,
    taban_iç: f32,
    halka: f32,
    kök_derinliği: usize,
    rollup: bool,
    ağaç_yüksekliği: usize,
    toplam: f64,
    birim_açı: f64,
    yön: f64,
    başlangıç: f64,
    palet: &dyn Fn(usize) -> Renk,
    eşlemeler: &[(&GörselEşleme, [f64; 2])],
    yol: &mut Vec<String>,
    atalar: &mut Vec<usize>,
    çıktı: &mut Vec<GüneşDilimi>,
) -> f64 {
    let bilgi = bilgiler
        .get(&std::ptr::from_ref(düğüm).addr())
        .copied()
        .unwrap_or(DüğümBilgisi {
            veri_sırası: 0,
            derinlik: 1,
            yükseklik: 0,
            değer: 0.0,
            üst_sıra: 0,
        });
    let mut açı = if toplam == 0.0 && seri.sıfır_toplamı_göster {
        birim_açı
    } else {
        bilgi.değer * birim_açı
    };
    açı = açı.max(f64::from(seri.en_küçük_açı) * RADYAN);
    if !açı.is_finite() {
        açı = 0.0;
    }
    let bitiş = başlangıç + yön * açı;
    let görünüm_derinliği = if rollup {
        bilgi
            .derinlik
            .saturating_sub(kök_derinliği)
            .saturating_add(1)
    } else {
        bilgi
            .derinlik
            .saturating_sub(kök_derinliği)
            .saturating_sub(1)
    };
    let mut iç = taban_iç + halka * görünüm_derinliği as f32;
    let mut dış = iç + halka;
    if let Some((seviye_içi, seviye_dışı)) = seri
        .seviyeler
        .get(bilgi.derinlik)
        .and_then(|seviye| seviye.yarıçap)
    {
        iç = seviye_içi.çöz(boyut);
        dış = seviye_dışı.çöz(boyut);
    }
    let taban = seri_katmanı(seri);
    let seviye = seri
        .seviyeler
        .get(bilgi.derinlik)
        .map_or(taban.clone(), |seviye| seviye_uygula(&taban, seviye));
    let katman = düğüm_uygula(&seviye, düğüm);
    let açık_renk = katman.öğe_stili.renk.is_some();
    let palet_sırası = match seri.renk_kaynağı {
        GüneşPatlamasıRenkKaynağı::Seri => bilgi.üst_sıra,
        GüneşPatlamasıRenkKaynağı::Veri => bilgi.veri_sırası,
    };
    let mut taban_renk = palet(palet_sırası);
    if bilgi.derinlik > 1 && !açık_renk {
        let payda = ağaç_yüksekliği.saturating_sub(1).max(1) as f32;
        taban_renk = zrender_rengi_aç(
            taban_renk,
            ((bilgi.derinlik - 1) as f32 / payda * 0.5).clamp(0.0, 0.5),
        );
    }
    let mut dolgu = katman
        .öğe_stili
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(taban_renk));
    if !eşlemeler.is_empty() {
        let mut renk = dolgu.temsilî();
        for (eşleme, kapsam) in eşlemeler {
            renk = eşleme.rengi_uygula(görsel_değer(düğüm, bilgi, eşleme), *kapsam, renk);
        }
        dolgu = Dolgu::Düz(renk);
    }
    yol.push(düğüm.ad.clone());
    let etiket_metni = seri.etiket_biçimleyicisi.as_ref().map_or_else(
        || {
            katman.etiket.biçimleyici.as_ref().map_or_else(
                || düğüm.ad.clone(),
                |biçimleyici| {
                    let değer = binlik_ayır(bilgi.değer);
                    if katman.etiket.zengin.is_empty() {
                        biçimleyici.uygula_bağlamla(
                            bilgi.değer,
                            &değer,
                            seri.ad.as_deref().unwrap_or(""),
                            &düğüm.ad,
                        )
                    } else {
                        biçimleyici.uygula_bağlamla_zengin(
                            bilgi.değer,
                            &değer,
                            seri.ad.as_deref().unwrap_or(""),
                            &düğüm.ad,
                        )
                    }
                },
            )
        },
        |biçimleyici| {
            let mut ağaç_yolu = Vec::with_capacity(yol.len() + 1);
            ağaç_yolu.push(GüneşPatlamasıYolBilgisi {
                ad: seri.ad.clone().unwrap_or_default(),
                veri_sırası: None,
                değer: toplam,
            });
            for (sıra, ad) in atalar.iter().copied().zip(yol.iter()) {
                let değer = bilgiler
                    .values()
                    .find(|bilgi| bilgi.veri_sırası == sıra)
                    .map_or(0.0, |bilgi| bilgi.değer);
                ağaç_yolu.push(GüneşPatlamasıYolBilgisi {
                    ad: ad.clone(),
                    veri_sırası: Some(sıra),
                    değer,
                });
            }
            ağaç_yolu.push(GüneşPatlamasıYolBilgisi {
                ad: düğüm.ad.clone(),
                veri_sırası: Some(bilgi.veri_sırası),
                değer: bilgi.değer,
            });
            biçimleyici.uygula(&GüneşPatlamasıEtiketParametreleri {
                seri_adı: seri.ad.clone(),
                ad: düğüm.ad.clone(),
                veri_sırası: bilgi.veri_sırası,
                değer: bilgi.değer,
                derinlik: bilgi.derinlik,
                yükseklik: bilgi.yükseklik,
                ağaç_yolu,
            })
        },
    );
    let (etiket_konumu, etiket_dönüşü, etiket_yatay_hizası, etiket_dikey_hizası) =
        etiket_geometrisi(
            &katman.etiket,
            merkez,
            iç,
            dış,
            başlangıç as f32,
            bitiş as f32,
        );
    // `renderLabelForZeroData` yalnız bağlı metni gizler. Sıfır açılı
    // sektör yine veri/graphicEl sırasına katılır; action ve yapısal kanıt
    // dataIndex'i bu öğeyi atlamamalıdır.
    let mut etiket = katman.etiket;
    if !seri.sıfır_veri_etiketini_göster && bilgi.değer == 0.0 {
        etiket.göster = false;
    }
    çıktı.push(GüneşDilimi {
        veri_sırası: bilgi.veri_sırası,
        ad: düğüm.ad.clone(),
        değer: bilgi.değer,
        derinlik: bilgi.derinlik,
        yol: yol.clone(),
        ata_sıraları: atalar.clone(),
        merkez,
        iç_yarıçap: iç,
        dış_yarıçap: dış,
        açı0: başlangıç as f32,
        açı1: bitiş as f32,
        saat_yönünde: seri.saat_yönünde,
        dolgu,
        öğe_stili: katman.öğe_stili,
        etiket,
        etiket_metni,
        etiket_konumu,
        etiket_dönüşü,
        etiket_yatay_hizası,
        etiket_dikey_hizası,
        vurgu: katman.vurgu,
        bulanık: katman.bulanık,
        seçili: katman.seçili,
        bağlantı: düğüm.bağlantı.clone(),
        hedef: düğüm.hedef.clone(),
    });

    atalar.push(bilgi.veri_sırası);
    let mut kardeş_açısı = 0.0;
    for çocuk in sıralı_çocuklar(seri, &düğüm.çocuklar, bilgiler) {
        kardeş_açısı += düğümü_yerleştir(
            seri,
            çocuk,
            bilgiler,
            merkez,
            boyut,
            taban_iç,
            halka,
            kök_derinliği,
            rollup,
            ağaç_yüksekliği,
            toplam,
            birim_açı,
            yön,
            başlangıç + kardeş_açısı,
            palet,
            eşlemeler,
            yol,
            atalar,
            çıktı,
        );
    }
    atalar.pop();
    yol.pop();
    bitiş - başlangıç
}

/// `visualMap.dimension` değerinin bütün ham ağaçtaki kapsamı.
pub fn güneş_patlaması_görsel_kapsamı(
    seri: &GüneşPatlamasıSerisi,
    eşleme: &GörselEşleme,
) -> [f64; 2] {
    let (bilgiler, _, _) = bilgi_haritası(seri);
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    fn gez(
        düğümler: &[AğaçDüğümü],
        bilgiler: &HashMap<usize, DüğümBilgisi>,
        eşleme: &GörselEşleme,
        kapsam: &mut [f64; 2],
    ) {
        for düğüm in düğümler {
            if let Some(bilgi) = bilgiler.get(&std::ptr::from_ref(düğüm).addr()).copied() {
                let değer = görsel_değer(düğüm, bilgi, eşleme);
                if değer.is_finite() {
                    kapsam[0] = kapsam[0].min(değer);
                    kapsam[1] = kapsam[1].max(değer);
                }
            }
            gez(&düğüm.çocuklar, bilgiler, eşleme, kapsam);
        }
    }
    gez(&seri.kökler, &bilgiler, eşleme, &mut kapsam);
    if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
        kapsam = [0.0, 0.0];
    }
    eşleme.kapsam_çöz(kapsam)
}

/// Resmî yerleşim ve kalıtım kurallarıyla sektör listesini üretir.
pub fn güneş_patlaması_dilimleri(
    seri: &GüneşPatlamasıSerisi,
    tuval: Dikdörtgen,
    kök_yolu: &[String],
    palet: &dyn Fn(usize) -> Renk,
    eşlemeler: &[(&GörselEşleme, [f64; 2])],
) -> Vec<GüneşDilimi> {
    let merkez = (
        tuval.x + seri.merkez.0.çöz(tuval.genişlik),
        tuval.y + seri.merkez.1.çöz(tuval.yükseklik),
    );
    let boyut = tuval.genişlik.min(tuval.yükseklik) / 2.0;
    let iç = seri.yarıçap.0.çöz(boyut);
    let dış = seri.yarıçap.1.çöz(boyut);
    let (bilgiler, sanal_toplam, sanal_yükseklik) = bilgi_haritası(seri);
    let (etkin_kök, inilen) = görünüm_kökü(&seri.kökler, kök_yolu);
    let rollup = etkin_kök.is_some();
    let (kök_derinliği, kök_yüksekliği, toplam, çocuklar): (_, _, _, &[AğaçDüğümü]) = etkin_kök
        .map_or((0, sanal_yükseklik, sanal_toplam, &seri.kökler), |kök| {
            let bilgi = bilgiler
                .get(&std::ptr::from_ref(kök).addr())
                .copied()
                .unwrap_or(DüğümBilgisi {
                    veri_sırası: 0,
                    derinlik: inilen,
                    yükseklik: 0,
                    değer: 0.0,
                    üst_sıra: 0,
                });
            (
                bilgi.derinlik,
                bilgi.yükseklik,
                bilgi.değer,
                std::slice::from_ref(kök),
            )
        });
    let seviye_sayısı = if rollup {
        kök_yüksekliği.saturating_add(1)
    } else {
        kök_yüksekliği.saturating_sub(1)
    };
    let halka = (dış - iç) / seviye_sayısı.max(1) as f32;
    let geçerli_çocuk = etkin_kök
        .map_or(&seri.kökler, |kök| &kök.çocuklar)
        .iter()
        .filter(|düğüm| {
            bilgiler
                .get(&std::ptr::from_ref(*düğüm).addr())
                .is_some_and(|bilgi| bilgi.değer.is_finite())
        })
        .count();
    let bölen = if toplam != 0.0 {
        toplam
    } else {
        geçerli_çocuk as f64
    };
    let birim_açı = if bölen > 0.0 {
        std::f64::consts::TAU / bölen
    } else {
        0.0
    };
    let başlangıç = -f64::from(seri.başlangıç_açısı) * RADYAN;
    let yön = if seri.saat_yönünde { 1.0 } else { -1.0 };
    let mut çıktı = Vec::new();
    let mut yol = kök_yolu.iter().take(inilen).cloned().collect::<Vec<_>>();
    if etkin_kök.is_some() {
        yol.pop();
    }
    let mut atalar = Vec::new();
    let mut kardeş_açısı = 0.0;
    for düğüm in sıralı_çocuklar(seri, çocuklar, &bilgiler) {
        kardeş_açısı += düğümü_yerleştir(
            seri,
            düğüm,
            &bilgiler,
            merkez,
            boyut,
            iç,
            halka,
            kök_derinliği,
            rollup,
            sanal_yükseklik,
            toplam,
            birim_açı,
            yön,
            başlangıç + kardeş_açısı,
            palet,
            eşlemeler,
            &mut yol,
            &mut atalar,
            &mut çıktı,
        );
    }
    çıktı
}

fn köşe_yarıçapları(stil: &GüneşPatlamasıÖğeStili, kalınlık: f32) -> [f32; 4] {
    stil.kenarlık_yarıçapı
        .map(|yarıçap| yarıçap.0.map(|değer| değer.çöz(kalınlık).max(0.0)))
        .unwrap_or([0.0; 4])
}

fn durum_uygula(
    dilim: &GüneşDilimi,
    durum: &GüneşPatlamasıDurumu,
) -> (GüneşPatlamasıÖğeStili, Etiket) {
    let stil = durum.öğe_stili.as_ref().map_or_else(
        || dilim.öğe_stili.clone(),
        |yama| öğe_stili_yama_uygula(&dilim.öğe_stili, yama),
    );
    let etiket = durum
        .etiket
        .as_ref()
        .map_or_else(|| dilim.etiket.clone(), |yama| yama.uygula(&dilim.etiket));
    (stil, etiket)
}

fn ilişkili_mi(odak: AğaçVurguOdağı, vurgulu: &GüneşDilimi, aday: &GüneşDilimi) -> bool {
    match odak {
        AğaçVurguOdağı::Yok => true,
        AğaçVurguOdağı::Öz => aday.veri_sırası == vurgulu.veri_sırası,
        AğaçVurguOdağı::Ata => {
            aday.veri_sırası == vurgulu.veri_sırası
                || vurgulu.ata_sıraları.contains(&aday.veri_sırası)
        }
        AğaçVurguOdağı::AltSoy => {
            aday.veri_sırası == vurgulu.veri_sırası
                || aday.ata_sıraları.contains(&vurgulu.veri_sırası)
        }
        AğaçVurguOdağı::İlişkili => {
            aday.veri_sırası == vurgulu.veri_sırası
                || vurgulu.ata_sıraları.contains(&aday.veri_sırası)
                || aday.ata_sıraları.contains(&vurgulu.veri_sırası)
        }
    }
}

fn etiketi_çiz(
    çizici: &mut dyn ÇizimYüzeyi, dilim: &GüneşDilimi, etiket: &Etiket, opaklık: f32
) {
    let açı = (dilim.açı1 - dilim.açı0).abs();
    if !etiket.göster
        || etiket
            .en_küçük_açı
            .is_some_and(|derece| açı < derece.to_radians())
        || dilim.etiket_metni.is_empty()
    {
        return;
    }
    let otomatik_iç_stil = dilim.dolgu.temsilî().zrender_iç_etiket_stili(false);
    let renk = etiket
        .yazı
        .renk
        .unwrap_or_else(|| {
            if etiket.konum == EtiketKonumu::Dış {
                tema::birincil_metin()
            } else {
                // SunburstPiece bağlı metni `textConfig.inside=true` ile
                // kurar. Açık label.color yoksa zrender dolgu parlaklığına
                // göre #333/#eee/#ccc iç metin rengini otomatik seçer.
                otomatik_iç_stil.0
            }
        })
        .opaklık(etiket.yazı.opaklık.unwrap_or(1.0) * opaklık);
    let kontur = etiket
        .yazı
        .kenarlık_rengi
        .zip(etiket.yazı.kenarlık_kalınlığı)
        .filter(|(_, kalınlık)| *kalınlık > 0.0)
        .or_else(|| {
            // Zrender bağlı Text öğesinde açık bir label.color yoksa
            // `autoStroke` kullanır: dış etikette tuval arka planı (açık
            // kipte beyaz), iç etikette ise yalnız gerektiğinde sektör
            // dolgusu; iki durumda da varsayılan çizgi genişliği 2 px'tir.
            etiket.yazı.renk.is_none().then(|| {
                if etiket.konum == EtiketKonumu::Dış {
                    Some((Renk::BEYAZ, 2.0))
                } else {
                    otomatik_iç_stil.1.map(|renk| (renk, 2.0))
                }
            })?
        })
        .map(|(renk, kalınlık)| {
            (
                renk.opaklık(etiket.yazı.opaklık.unwrap_or(1.0) * opaklık),
                kalınlık,
            )
        });
    let (konum, dönüş, yatay, dikey) = etiket_geometrisi(
        etiket,
        dilim.merkez,
        dilim.iç_yarıçap,
        dilim.dış_yarıçap,
        dilim.açı0,
        dilim.açı1,
    );
    if !etiket.zengin.is_empty() {
        zengin_etiketi_hizalı_yaz(
            çizici,
            &dilim.etiket_metni,
            etiket,
            konum,
            yatay,
            dikey,
            renk,
            -dönüş,
        );
        return;
    }
    let metin = zengin_metin_içeriği(dilim.etiket_metni.clone());
    let boyut = etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    // zrender `Transformable.rotation` ekranın y-aşağı uzayında pozitif
    // değeri saat yönünün tersine uygular. `AfinMatris::döndür` Canvas
    // y-aşağı uzayında pozitif değeri saat yönüne çevirdiğinden işaret
    // burada terslenir; hesaplanan/kanıtlanan model açısı değişmez.
    let dönüşüm = AfinMatris::ötele(konum.0, konum.1).çarp(AfinMatris::döndür(-dönüş));
    let iç_boşluk = etiket.yazı.iç_boşluk.unwrap_or([0.0; 4]);
    let yerel_x = match yatay {
        YatayHiza::Sol => iç_boşluk[3],
        YatayHiza::Orta => (iç_boşluk[3] - iç_boşluk[1]) / 2.0,
        YatayHiza::Sağ => -iç_boşluk[1],
    };
    let yerel_y = match dikey {
        DikeyHiza::Üst => iç_boşluk[0],
        DikeyHiza::Orta => 0.0,
        DikeyHiza::Alt => -iç_boşluk[2],
    };
    let satırlar = metin.split('\n').collect::<Vec<_>>();
    let satır_yüksekliği = etiket.yazı.satır_yüksekliği.unwrap_or(boyut);
    let orta_sıra = (satırlar.len().saturating_sub(1)) as f32 / 2.0;
    for (sıra, satır) in satırlar.into_iter().enumerate() {
        let satır_konumu = (
            yerel_x,
            yerel_y + (sıra as f32 - orta_sıra) * satır_yüksekliği,
        );
        if let (Some(bulanıklık), Some(gölge_rengi)) = (
            etiket.yazı.metin_gölge_bulanıklığı,
            etiket.yazı.metin_gölge_rengi,
        ) && bulanıklık > 0.0
        {
            çizici.dönüşümlü_yazı_gölgesi(
                satır,
                satır_konumu,
                yatay,
                dikey,
                boyut,
                etiket.yazı.kalın,
                gölge_rengi.opaklık(etiket.yazı.opaklık.unwrap_or(1.0) * opaklık),
                bulanıklık,
                etiket.yazı.metin_gölge_kayması.unwrap_or((0.0, 0.0)),
                dönüşüm,
            );
        }
        if let Some((kontur_rengi, kontur_kalınlığı)) = kontur {
            çizici.dönüşümlü_konturlu_yazı(
                satır,
                satır_konumu,
                yatay,
                dikey,
                boyut,
                renk,
                etiket.yazı.kalın,
                kontur_rengi,
                kontur_kalınlığı,
                dönüşüm,
            );
        } else {
            çizici.dönüşümlü_aileli_yazı(
                satır,
                satır_konumu,
                yatay,
                dikey,
                boyut,
                renk,
                etiket.yazı.kalın,
                etiket.yazı.aile.as_deref().unwrap_or("sans-serif"),
                dönüşüm,
            );
        }
    }
}

/// Güneş patlamasını çizer; `kök_yolu`, `sunburstRootToNode` görünümünü
/// temsil eder ve merkezdeki sanal kök halkası bir üst düğüme döner.
#[allow(clippy::too_many_arguments)]
pub fn güneş_patlaması_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &GüneşPatlamasıSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    eşlemeler: &[(&GörselEşleme, [f64; 2])],
    ilerleme: f32,
    kök_yolu: &[String],
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
    kırıntılar: &mut Vec<(Dikdörtgen, usize, usize)>,
) {
    let dilimler = güneş_patlaması_dilimleri(seri, tuval, kök_yolu, palet, eşlemeler);
    let vurgulu = fare.and_then(|fare| {
        dilimler.iter().rev().find(|dilim| {
            İsabetGeometrisi::Halka {
                merkez: dilim.merkez,
                iç_yarıçap: dilim.iç_yarıçap,
                dış_yarıçap: dilim.dış_yarıçap,
                açı0: dilim.açı0,
                açı1: dilim.açı1,
            }
            .içeriyor_mu(fare)
        })
    });
    let odak = vurgulu.and_then(|dilim| dilim.vurgu.odak);
    let ilerleme = ilerleme.clamp(0.0, 1.0);

    // SunburstPiece ilk çizimde her sektörü r0'dan gerçek r'ye büyütür.
    // Bütün sektörler önce, bütün bağlı metinler sonra boyanır; zrender'ın
    // sector z2=2 / text z2=4 katman sırası böyle korunur.
    for dilim in &dilimler {
        let etkin_durum = if vurgulu.is_some_and(|v| v.veri_sırası == dilim.veri_sırası) {
            Some(&dilim.vurgu)
        } else if let (Some(vurgulu), Some(odak)) = (vurgulu, odak)
            && !ilişkili_mi(odak, vurgulu, dilim)
        {
            Some(&dilim.bulanık)
        } else {
            None
        };
        let (stil, _) = etkin_durum.map_or_else(
            || (dilim.öğe_stili.clone(), dilim.etiket.clone()),
            |durum| durum_uygula(dilim, durum),
        );
        let opaklık = stil.opaklık.unwrap_or(1.0).clamp(0.0, 1.0);
        let dolgu = etkin_durum
            .and_then(|durum| durum.öğe_stili.as_ref())
            .and_then(|stil| stil.renk.clone())
            .unwrap_or_else(|| dilim.dolgu.clone())
            .opaklık(opaklık);
        let dış = dilim.iç_yarıçap + (dilim.dış_yarıçap - dilim.iç_yarıçap) * ilerleme;
        let köşeler = köşe_yarıçapları(&stil, (dış - dilim.iç_yarıçap).abs());
        let yol = yuvarlatılmış_dilim_yolu(
            dilim.merkez,
            dilim.iç_yarıçap,
            dış,
            dilim.açı0,
            dilim.açı1,
            köşeler,
        );
        let kenarlık_kalınlığı = stil.kenarlık_kalınlığı.unwrap_or(0.0).max(0.0);
        if stil.gölge_bulanıklığı.unwrap_or(0.0) > 0.0
            && let Some(renk) = stil.gölge_rengi
        {
            let bulanıklık = stil.gölge_bulanıklığı.unwrap_or(0.0);
            let kayma = stil.gölge_kayması.unwrap_or((0.0, 0.0));
            let dolgu_alfası = dolgu.temsilî().alfa.clamp(0.0, 1.0);
            if dolgu_alfası > 0.0 {
                çizici.yol_gölgesi(&yol, renk.opaklık(dolgu_alfası), bulanıklık, kayma);
            } else if kenarlık_kalınlığı > 0.0 {
                // Canvas saydam dolgudan gölge üretmez; aynı Path'in
                // kalıtılmış stroke'u ise kendi maskesiyle gölgelenir.
                çizici.yol_çizgi_gölgesi(
                    &yol,
                    kenarlık_kalınlığı,
                    stil.kenarlık_türü.unwrap_or(ÇizgiTürü::Düz),
                    renk,
                    bulanıklık,
                    kayma,
                );
            }
        }
        çizici.yol_doldur(&yol, &dolgu);
        if kenarlık_kalınlığı > 0.0
            && let Some(renk) = stil.kenarlık_rengi
        {
            çizici.yol_çiz(
                &yol,
                kenarlık_kalınlığı,
                renk.opaklık(opaklık),
                stil.kenarlık_türü.unwrap_or(ÇizgiTürü::Düz),
            );
        }
        if !seri.sessiz {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: dilim.veri_sırası,
                seri_adı: seri.ad.clone(),
                ad: Some(dilim.ad.clone()),
                değer: Some(dilim.değer),
                geometri: İsabetGeometrisi::Halka {
                    merkez: dilim.merkez,
                    iç_yarıçap: dilim.iç_yarıçap,
                    dış_yarıçap: dilim.dış_yarıçap,
                    açı0: dilim.açı0,
                    açı1: dilim.açı1,
                },
            });
        }
    }
    for dilim in &dilimler {
        let (_, etiket) = if vurgulu.is_some_and(|v| v.veri_sırası == dilim.veri_sırası) {
            durum_uygula(dilim, &dilim.vurgu)
        } else if let (Some(vurgulu), Some(odak)) = (vurgulu, odak)
            && !ilişkili_mi(odak, vurgulu, dilim)
        {
            durum_uygula(dilim, &dilim.bulanık)
        } else {
            (dilim.öğe_stili.clone(), dilim.etiket.clone())
        };
        etiketi_çiz(çizici, dilim, &etiket, ilerleme);
    }

    if !kök_yolu.is_empty() {
        let merkez = dilimler.first().map(|dilim| dilim.merkez).unwrap_or((
            tuval.x + seri.merkez.0.çöz(tuval.genişlik),
            tuval.y + seri.merkez.1.çöz(tuval.yükseklik),
        ));
        let boyut = tuval.genişlik.min(tuval.yükseklik) / 2.0;
        let iç = seri.yarıçap.0.çöz(boyut);
        let dış = dilimler
            .iter()
            .map(|dilim| dilim.iç_yarıçap)
            .fold(f32::INFINITY, f32::min);
        let rollup_dış = if dış.is_finite() { dış } else { iç };
        let taban = seri_katmanı(seri);
        let katman = seri
            .seviyeler
            .first()
            .map_or(taban.clone(), |seviye| seviye_uygula(&taban, seviye));
        let opaklık = katman.öğe_stili.opaklık.unwrap_or(1.0);
        let dolgu = katman
            .öğe_stili
            .renk
            .clone()
            .unwrap_or_else(|| Dolgu::Düz(tema::nötr_50()))
            .opaklık(opaklık);
        çizici.yuvarlatılmış_dilim(
            merkez,
            iç,
            rollup_dış,
            -f32::from(seri.başlangıç_açısı).to_radians(),
            -f32::from(seri.başlangıç_açısı).to_radians() + std::f32::consts::TAU,
            köşe_yarıçapları(&katman.öğe_stili, rollup_dış - iç),
            &dolgu,
            katman
                .öğe_stili
                .kenarlık_rengi
                .zip(katman.öğe_stili.kenarlık_kalınlığı)
                .map(|(renk, kalınlık)| (kalınlık, renk)),
        );
        let yarıçap = rollup_dış.max(1.0);
        kırıntılar.push((
            Dikdörtgen::yeni(
                merkez.0 - yarıçap,
                merkez.1 - yarıçap,
                yarıçap * 2.0,
                yarıçap * 2.0,
            ),
            genel_sıra,
            kök_yolu.len().saturating_sub(1),
        ));
    }
}

/// Dilim geometrisini kararlı FNV-1a özetiyle kilitler.
pub fn güneş_patlaması_sahne_özeti(
    seri: &GüneşPatlamasıSerisi,
    tuval: Dikdörtgen,
    kök_yolu: &[String],
    palet: &dyn Fn(usize) -> Renk,
    eşlemeler: &[(&GörselEşleme, [f64; 2])],
) -> GüneşPatlamasıSahneÖzeti {
    fn bayt(özet: &mut u64, değer: u8) {
        *özet ^= u64::from(değer);
        *özet = özet.wrapping_mul(0x0000_0100_0000_01b3);
    }
    fn sayı(özet: &mut u64, değer: u64) {
        for bayt_değeri in değer.to_le_bytes() {
            bayt(özet, bayt_değeri);
        }
    }
    let dilimler = güneş_patlaması_dilimleri(seri, tuval, kök_yolu, palet, eşlemeler);
    let mut özet = 0xcbf2_9ce4_8422_2325_u64;
    sayı(&mut özet, dilimler.len() as u64);
    let mut etiket_sayısı = 0usize;
    for dilim in &dilimler {
        sayı(&mut özet, dilim.veri_sırası as u64);
        sayı(&mut özet, dilim.derinlik as u64);
        for koordinat in [
            dilim.merkez.0,
            dilim.merkez.1,
            dilim.iç_yarıçap,
            dilim.dış_yarıçap,
            dilim.açı0,
            dilim.açı1,
            dilim.etiket_konumu.0,
            dilim.etiket_konumu.1,
            dilim.etiket_dönüşü,
        ] {
            sayı(&mut özet, ((koordinat * 1_000.0).round() as i64) as u64);
        }
        for kanal in {
            let renk = dilim.dolgu.temsilî();
            [renk.kırmızı, renk.yeşil, renk.mavi, renk.alfa]
        } {
            bayt(&mut özet, (kanal.clamp(0.0, 1.0) * 255.0).round() as u8);
        }
        for bayt_değeri in dilim.ad.as_bytes() {
            bayt(&mut özet, *bayt_değeri);
        }
        if dilim.etiket.göster
            && !dilim.etiket_metni.is_empty()
            && !dilim
                .etiket
                .en_küçük_açı
                .is_some_and(|derece| (dilim.açı1 - dilim.açı0).abs() < derece.to_radians())
        {
            etiket_sayısı = etiket_sayısı.saturating_add(1);
        }
    }
    GüneşPatlamasıSahneÖzeti {
        dilim_sayısı: dilimler.len(),
        etiket_sayısı,
        koordinat_sayısı: dilimler.len().saturating_mul(9),
        fnv1a_64: özet,
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::model::agac::{GüneşPatlamasıKöşeYarıçapı, GüneşPatlamasıSeviyesi};

    fn seri() -> GüneşPatlamasıSerisi {
        GüneşPatlamasıSerisi::yeni().halka(0, "90%").kökler([
            AğaçDüğümü::dal(
                "A",
                vec![AğaçDüğümü::yaprak("A1", 2.0), AğaçDüğümü::yaprak("A2", 1.0)],
            ),
            AğaçDüğümü::yaprak("B", 1.0),
        ])
    }

    #[test]
    fn resmi_birim_aciyi_ve_on_sirali_indeksleri_korur() {
        let dilimler = güneş_patlaması_dilimleri(
            &seri(),
            Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0),
            &[],
            &|sıra| [0x5070ddu32, 0xb6d634][sıra % 2].into(),
            &[],
        );
        assert_eq!(dilimler.len(), 4);
        assert_eq!(dilimler[0].ad, "A");
        assert_eq!(dilimler[0].veri_sırası, 0);
        assert!((dilimler[0].açı1 - dilimler[0].açı0 - 1.5 * std::f32::consts::PI).abs() < 1e-5);
        assert_eq!(dilimler[1].veri_sırası, 1);
        assert_eq!(dilimler[3].veri_sırası, 3);
    }

    #[test]
    fn animation_type_iki_resmi_degeri_modelde_korur() {
        use crate::model::agac::GüneşPatlamasıAnimasyonTürü;

        assert_eq!(
            GüneşPatlamasıSerisi::yeni().animasyon_türü,
            GüneşPatlamasıAnimasyonTürü::Genişleme
        );
        assert_eq!(
            GüneşPatlamasıSerisi::yeni()
                .animasyon_türü(GüneşPatlamasıAnimasyonTürü::Ölçek)
                .animasyon_türü,
            GüneşPatlamasıAnimasyonTürü::Ölçek
        );
    }

    #[test]
    fn drill_down_rollup_halkasina_bir_seviye_ayirir() {
        let seri = seri();
        let dilimler = güneş_patlaması_dilimleri(
            &seri,
            Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0),
            &["A".to_owned()],
            &|_| 0x5070ddu32.into(),
            &[],
        );
        assert_eq!(dilimler[0].ad, "A");
        assert!(dilimler[0].iç_yarıçap > 0.0);
        assert!((dilimler[0].açı1 - dilimler[0].açı0 - std::f32::consts::TAU).abs() < 1e-5);
        assert!(dilimler[1].iç_yarıçap >= dilimler[0].dış_yarıçap - 1e-5);
    }

    #[test]
    fn mutlak_level_yaricapi_ve_yuzde_kose_yaricapi_korunur() {
        let seri = seri().seviyeler([
            GüneşPatlamasıSeviyesi::yeni(),
            GüneşPatlamasıSeviyesi::yeni()
                .yarıçap("10%", "40%")
                .öğe_stili(GüneşPatlamasıÖğeStili::yeni().kenarlık_yarıçapı(
                    GüneşPatlamasıKöşeYarıçapı::from([
                        Uzunluk::Yüzde(50.0),
                        Uzunluk::Yüzde(50.0),
                        Uzunluk::Piksel(7.0),
                        Uzunluk::Piksel(7.0),
                    ]),
                )),
        ]);
        let dilimler = güneş_patlaması_dilimleri(
            &seri,
            Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0),
            &[],
            &|_| 0x5070ddu32.into(),
            &[],
        );
        assert_eq!(dilimler[0].iç_yarıçap, 15.0);
        assert_eq!(dilimler[0].dış_yarıçap, 60.0);
        assert_eq!(
            köşe_yarıçapları(&dilimler[0].öğe_stili, 45.0),
            [22.5, 22.5, 7.0, 7.0]
        );
    }

    #[test]
    fn sahne_ozeti_ayni_veride_kararlidir() {
        let alan = Dikdörtgen::yeni(0.0, 0.0, 600.0, 400.0);
        let palet = |sıra: usize| -> Renk { [0x5070ddu32, 0xb6d634u32][sıra % 2].into() };
        let ilk = güneş_patlaması_sahne_özeti(&seri(), alan, &[], &palet, &[]);
        let ikinci = güneş_patlaması_sahne_özeti(&seri(), alan, &[], &palet, &[]);
        assert_eq!(ilk, ikinci);
        assert_eq!(ilk.dilim_sayısı, 4);
        assert!(ilk.etiket_sayısı > 0);
    }
}
