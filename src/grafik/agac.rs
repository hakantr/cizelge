//! Ağaç (tree) serisi — `echarts/src/chart/tree` karşılığı.
//!
//! Yerleşim, ECharts'ın D3/Reingold–Tilford tabanlı `layoutHelper.ts`
//! algoritmasını izler. Dik dört yön, radyal görünüm, eğri/kırık kenarlar,
//! ilk derinlik ve düğüm bazlı daraltma aynı görünür ağaç üstünde çözülür.

use std::collections::HashSet;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::yuzey::YolKomutu;
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::{sembol_dönüşümlü_yolu, sembol_stilli_dönüşümlü_çiz};
use crate::koordinat::Dikdörtgen;
use crate::model::agac::{
    AğaçDüğümü, AğaçKenarBiçimi, AğaçVurguOdağı, AğaçYerleşimi, AğaçYönü
};
use crate::model::seri::{AğaçSerisi, Sembol};
use crate::model::stil::{
    Etiket, EtiketDöndürme, EtiketKonumu, YazıDikeyHizası, YazıYatayHizası, ÇizgiStili, ÖğeStili,
};
use crate::renk::{Dolgu, Renk};
use crate::tema;

#[derive(Clone, Copy, Debug)]
struct HiyerarşiKaydı {
    öntaslak: f32,
    değiştirici: f32,
    değişim: f32,
    kaydırma: f32,
    kardeş_sırası: usize,
    iplik: Option<usize>,
    ata: usize,
    öntanımlı_ata: Option<usize>,
}

impl HiyerarşiKaydı {
    fn yeni(kendi: usize, kardeş_sırası: usize) -> Self {
        Self {
            öntaslak: 0.0,
            değiştirici: 0.0,
            değişim: 0.0,
            kaydırma: 0.0,
            kardeş_sırası,
            iplik: None,
            ata: kendi,
            öntanımlı_ata: None,
        }
    }
}

/// Çizimden önce çözülmüş görünür düğüm. `kaynak = None` sanal köktür.
#[derive(Debug)]
struct YerleşikDüğüm<'a> {
    kaynak: Option<&'a AğaçDüğümü>,
    veri_sırası: usize,
    üst: Option<usize>,
    çocuklar: Vec<usize>,
    derinlik: usize,
    açık: bool,
    hiyerarşi: HiyerarşiKaydı,
    sıra_koordinatı: f32,
    konum: (f32, f32),
    ham: (f32, f32),
}

fn alt_ağaç_boyutu(düğüm: &AğaçDüğümü) -> usize {
    1 + düğüm.çocuklar.iter().map(alt_ağaç_boyutu).sum::<usize>()
}

fn düğüm_açık_mı(seri: &AğaçSerisi, düğüm: &AğaçDüğümü, derinlik: usize) -> bool {
    düğüm.daraltılmış.map(|dar| !dar).unwrap_or_else(|| {
        !seri.genişlet_ve_daralt
            || seri.ilk_ağaç_derinliği < 0
            // ECharts ağaca bir sanal kök ekler; gerçek kökün derinliği 1'dir.
            || derinlik.saturating_add(1) as isize <= seri.ilk_ağaç_derinliği
    })
}

fn görünür_düğümü_ekle<'a>(
    düğüm: &'a AğaçDüğümü,
    seri: &AğaçSerisi,
    derinlik: usize,
    üst: usize,
    kardeş_sırası: usize,
    veri_sayacı: &mut usize,
    düğümler: &mut Vec<YerleşikDüğüm<'a>>,
) -> usize {
    let kendi = düğümler.len();
    let veri_sırası = *veri_sayacı;
    *veri_sayacı = veri_sayacı.saturating_add(1);
    let açık = düğüm_açık_mı(seri, düğüm, derinlik);
    düğümler.push(YerleşikDüğüm {
        kaynak: Some(düğüm),
        veri_sırası,
        üst: Some(üst),
        çocuklar: Vec::new(),
        derinlik: derinlik.saturating_add(1),
        açık,
        hiyerarşi: HiyerarşiKaydı::yeni(kendi, kardeş_sırası),
        sıra_koordinatı: 0.0,
        konum: (0.0, 0.0),
        ham: (0.0, 0.0),
    });
    if açık {
        for (sıra, çocuk) in düğüm.çocuklar.iter().enumerate() {
            let çocuk_sırası = görünür_düğümü_ekle(
                çocuk,
                seri,
                derinlik + 1,
                kendi,
                sıra,
                veri_sayacı,
                düğümler,
            );
            düğümler[kendi].çocuklar.push(çocuk_sırası);
        }
    } else {
        *veri_sayacı =
            veri_sayacı.saturating_add(düğüm.çocuklar.iter().map(alt_ağaç_boyutu).sum::<usize>());
    }
    kendi
}

fn ayrım(düğümler: &[YerleşikDüğüm<'_>], a: usize, b: usize, radyal: bool) -> f32 {
    let taban = if düğümler[a].üst == düğümler[b].üst {
        1.0
    } else {
        2.0
    };
    if radyal {
        taban / düğümler[a].derinlik.max(1) as f32
    } else {
        taban
    }
}

fn sonraki_sol(düğümler: &[YerleşikDüğüm<'_>], sıra: usize) -> Option<usize> {
    düğümler[sıra]
        .çocuklar
        .first()
        .copied()
        .or(düğümler[sıra].hiyerarşi.iplik)
}

fn sonraki_sağ(düğümler: &[YerleşikDüğüm<'_>], sıra: usize) -> Option<usize> {
    düğümler[sıra]
        .çocuklar
        .last()
        .copied()
        .or(düğümler[sıra].hiyerarşi.iplik)
}

fn alt_ağacı_kaydır(
    düğümler: &mut [YerleşikDüğüm<'_>], sol: usize, sağ: usize, kayma: f32
) {
    let payda = düğümler[sağ]
        .hiyerarşi
        .kardeş_sırası
        .saturating_sub(düğümler[sol].hiyerarşi.kardeş_sırası)
        .max(1) as f32;
    let değişim = kayma / payda;
    düğümler[sağ].hiyerarşi.değişim -= değişim;
    düğümler[sağ].hiyerarşi.kaydırma += kayma;
    düğümler[sağ].hiyerarşi.değiştirici += kayma;
    düğümler[sağ].hiyerarşi.öntaslak += kayma;
    düğümler[sol].hiyerarşi.değişim += değişim;
}

fn sonraki_ata(
    düğümler: &[YerleşikDüğüm<'_>], iç_sol: usize, düğüm: usize, ata: usize
) -> usize {
    let aday = düğümler[iç_sol].hiyerarşi.ata;
    if düğümler[aday].üst == düğümler[düğüm].üst {
        aday
    } else {
        ata
    }
}

fn paylaştır(
    düğümler: &mut [YerleşikDüğüm<'_>],
    alt_v: usize,
    alt_w: Option<usize>,
    mut ata: usize,
    radyal: bool,
) -> usize {
    let Some(alt_w) = alt_w else { return ata };
    let Some(üst) = düğümler[alt_v].üst else {
        return ata;
    };
    let Some(&ilk_kardeş) = düğümler[üst].çocuklar.first() else {
        return ata;
    };
    let mut dış_sağ = alt_v;
    let mut iç_sağ = alt_v;
    let mut dış_sol = ilk_kardeş;
    let mut iç_sol = alt_w;
    let mut toplam_dış_sağ = düğümler[dış_sağ].hiyerarşi.değiştirici;
    let mut toplam_iç_sağ = düğümler[iç_sağ].hiyerarşi.değiştirici;
    let mut toplam_dış_sol = düğümler[dış_sol].hiyerarşi.değiştirici;
    let mut toplam_iç_sol = düğümler[iç_sol].hiyerarşi.değiştirici;

    loop {
        let yeni_iç_sol = sonraki_sağ(düğümler, iç_sol);
        let yeni_iç_sağ = sonraki_sol(düğümler, iç_sağ);
        let (Some(yeni_iç_sol), Some(yeni_iç_sağ)) = (yeni_iç_sol, yeni_iç_sağ) else {
            break;
        };
        iç_sol = yeni_iç_sol;
        iç_sağ = yeni_iç_sağ;
        dış_sağ = sonraki_sağ(düğümler, dış_sağ).unwrap_or(dış_sağ);
        dış_sol = sonraki_sol(düğümler, dış_sol).unwrap_or(dış_sol);
        düğümler[dış_sağ].hiyerarşi.ata = alt_v;
        let kayma = düğümler[iç_sol].hiyerarşi.öntaslak + toplam_iç_sol
            - düğümler[iç_sağ].hiyerarşi.öntaslak
            - toplam_iç_sağ
            + ayrım(düğümler, iç_sol, iç_sağ, radyal);
        if kayma > 0.0 {
            let sol_ata = sonraki_ata(düğümler, iç_sol, alt_v, ata);
            alt_ağacı_kaydır(düğümler, sol_ata, alt_v, kayma);
            toplam_iç_sağ += kayma;
            toplam_dış_sağ += kayma;
        }
        toplam_iç_sol += düğümler[iç_sol].hiyerarşi.değiştirici;
        toplam_iç_sağ += düğümler[iç_sağ].hiyerarşi.değiştirici;
        toplam_dış_sağ += düğümler[dış_sağ].hiyerarşi.değiştirici;
        toplam_dış_sol += düğümler[dış_sol].hiyerarşi.değiştirici;
    }

    if sonraki_sağ(düğümler, iç_sol).is_some() && sonraki_sağ(düğümler, dış_sağ).is_none()
    {
        düğümler[dış_sağ].hiyerarşi.iplik = sonraki_sağ(düğümler, iç_sol);
        düğümler[dış_sağ].hiyerarşi.değiştirici += toplam_iç_sol - toplam_dış_sağ;
    }
    if sonraki_sol(düğümler, iç_sağ).is_some() && sonraki_sol(düğümler, dış_sol).is_none()
    {
        düğümler[dış_sol].hiyerarşi.iplik = sonraki_sol(düğümler, iç_sağ);
        düğümler[dış_sol].hiyerarşi.değiştirici += toplam_iç_sağ - toplam_dış_sol;
        ata = alt_v;
    }
    ata
}

fn kaydırmaları_uygula(düğümler: &mut [YerleşikDüğüm<'_>], sıra: usize) {
    let çocuklar = düğümler[sıra].çocuklar.clone();
    let mut kayma = 0.0;
    let mut değişim = 0.0;
    for çocuk in çocuklar.into_iter().rev() {
        düğümler[çocuk].hiyerarşi.öntaslak += kayma;
        düğümler[çocuk].hiyerarşi.değiştirici += kayma;
        değişim += düğümler[çocuk].hiyerarşi.değişim;
        kayma += düğümler[çocuk].hiyerarşi.kaydırma + değişim;
    }
}

fn ilk_yürüyüş(düğümler: &mut [YerleşikDüğüm<'_>], sıra: usize, radyal: bool) {
    let çocuklar = düğümler[sıra].çocuklar.clone();
    for çocuk in çocuklar.iter().copied() {
        ilk_yürüyüş(düğümler, çocuk, radyal);
    }
    let Some(üst) = düğümler[sıra].üst else {
        return;
    };
    let kardeş_sırası = düğümler[sıra].hiyerarşi.kardeş_sırası;
    let önceki = kardeş_sırası
        .checked_sub(1)
        .and_then(|önceki| düğümler[üst].çocuklar.get(önceki).copied());
    if !çocuklar.is_empty() {
        kaydırmaları_uygula(düğümler, sıra);
        let orta = (düğümler[çocuklar[0]].hiyerarşi.öntaslak
            + düğümler[*çocuklar.last().unwrap_or(&çocuklar[0])]
                .hiyerarşi
                .öntaslak)
            / 2.0;
        if let Some(önceki) = önceki {
            düğümler[sıra].hiyerarşi.öntaslak =
                düğümler[önceki].hiyerarşi.öntaslak + ayrım(düğümler, sıra, önceki, radyal);
            düğümler[sıra].hiyerarşi.değiştirici = düğümler[sıra].hiyerarşi.öntaslak - orta;
        } else {
            düğümler[sıra].hiyerarşi.öntaslak = orta;
        }
    } else if let Some(önceki) = önceki {
        düğümler[sıra].hiyerarşi.öntaslak =
            düğümler[önceki].hiyerarşi.öntaslak + ayrım(düğümler, sıra, önceki, radyal);
    }
    let öntanımlı = düğümler[üst]
        .hiyerarşi
        .öntanımlı_ata
        .or_else(|| düğümler[üst].çocuklar.first().copied())
        .unwrap_or(sıra);
    düğümler[üst].hiyerarşi.öntanımlı_ata =
        Some(paylaştır(düğümler, sıra, önceki, öntanımlı, radyal));
}

fn ikinci_yürüyüş(düğümler: &mut [YerleşikDüğüm<'_>], sıra: usize) {
    let üst_değiştiricisi = düğümler[sıra]
        .üst
        .map(|üst| düğümler[üst].hiyerarşi.değiştirici)
        .unwrap_or(0.0);
    düğümler[sıra].sıra_koordinatı = düğümler[sıra].hiyerarşi.öntaslak + üst_değiştiricisi;
    düğümler[sıra].hiyerarşi.değiştirici += üst_değiştiricisi;
    let çocuklar = düğümler[sıra].çocuklar.clone();
    for çocuk in çocuklar {
        ikinci_yürüyüş(düğümler, çocuk);
    }
}

fn görünüm_alanı(seri: &AğaçSerisi, tuval: Dikdörtgen) -> Dikdörtgen {
    let sol = seri.sol.çöz(tuval.genişlik);
    let üst = seri.üst.çöz(tuval.yükseklik);
    let genişlik = seri
        .sağ
        .map(|sağ| tuval.genişlik - sol - sağ.çöz(tuval.genişlik))
        .unwrap_or_else(|| seri.genişlik.çöz(tuval.genişlik))
        .max(0.0);
    let yükseklik = seri
        .alt
        .map(|alt| tuval.yükseklik - üst - alt.çöz(tuval.yükseklik))
        .unwrap_or_else(|| seri.yükseklik.çöz(tuval.yükseklik))
        .max(0.0);
    Dikdörtgen::yeni(tuval.x + sol, tuval.y + üst, genişlik, yükseklik)
}

/// Tree serisinin kutu alanı; gpui roam isabeti de aynı çözümü kullanır.
pub fn ağaç_alanı(seri: &AğaçSerisi, tuval: Dikdörtgen) -> Dikdörtgen {
    görünüm_alanı(seri, tuval)
}

fn yerleşimi_hesapla<'a>(
    seri: &'a AğaçSerisi,
    alan: Dikdörtgen,
    ilerleme: f32,
    görünüm: (f32, f32, f32),
) -> Vec<YerleşikDüğüm<'a>> {
    let mut düğümler = vec![YerleşikDüğüm {
        kaynak: None,
        veri_sırası: usize::MAX,
        üst: None,
        çocuklar: Vec::new(),
        derinlik: 0,
        açık: true,
        hiyerarşi: HiyerarşiKaydı::yeni(0, 0),
        sıra_koordinatı: 0.0,
        konum: (0.0, 0.0),
        ham: (0.0, 0.0),
    }];
    let mut veri_sayacı = 0usize;
    for (sıra, kök) in seri.kökler.iter().enumerate() {
        let kök_sırası =
            görünür_düğümü_ekle(kök, seri, 0, 0, sıra, &mut veri_sayacı, &mut düğümler);
        düğümler[0].çocuklar.push(kök_sırası);
    }
    let Some(&gerçek_kök) = düğümler[0].çocuklar.first() else {
        return Vec::new();
    };
    // ECharts TreeLayout, veri dizisinin ilk öğesini gerçek kök kabul eder.
    // İkinci ve sonraki kökler veri/diff modelinde kalır fakat yerleşmez.
    let radyal = seri.yerleşim == AğaçYerleşimi::Radyal;
    ilk_yürüyüş(&mut düğümler, gerçek_kök, radyal);
    düğümler[0].hiyerarşi.değiştirici = -düğümler[gerçek_kök].hiyerarşi.öntaslak;
    ikinci_yürüyüş(&mut düğümler, gerçek_kök);

    let görünürler = alt_soy_sıraları(&düğümler, gerçek_kök, true);
    let mut sol = gerçek_kök;
    let mut sağ = gerçek_kök;
    let mut alt = gerçek_kök;
    for &sıra in &görünürler {
        if düğümler[sıra].sıra_koordinatı < düğümler[sol].sıra_koordinatı {
            sol = sıra;
        }
        if düğümler[sıra].sıra_koordinatı > düğümler[sağ].sıra_koordinatı {
            sağ = sıra;
        }
        if düğümler[sıra].derinlik > düğümler[alt].derinlik {
            alt = sıra;
        }
    }
    let delta = if sol == sağ {
        1.0
    } else {
        ayrım(&düğümler, sol, sağ, radyal) / 2.0
    };
    let tx = delta - düğümler[sol].sıra_koordinatı;
    let sıra_payda = (düğümler[sağ].sıra_koordinatı + delta + tx).max(f32::EPSILON);
    let derinlik_payda = düğümler[alt].derinlik.saturating_sub(1).max(1) as f32;
    let ilerleme = ilerleme.clamp(0.0, 1.0);

    for &sıra in &görünürler {
        let sıra_konumu = düğümler[sıra].sıra_koordinatı;
        let derinlik = düğümler[sıra].derinlik.saturating_sub(1) as f32;
        if radyal {
            let açı = (sıra_konumu + tx) * std::f32::consts::TAU / sıra_payda;
            let yarıçap =
                derinlik * (alan.genişlik.min(alan.yükseklik) / 2.0) / derinlik_payda * ilerleme;
            düğümler[sıra].ham = (açı, yarıçap);
            let açı = açı - std::f32::consts::FRAC_PI_2;
            düğümler[sıra].konum = (
                alan.merkez().0 + yarıçap * açı.cos(),
                alan.merkez().1 + yarıçap * açı.sin(),
            );
        } else {
            let çapraz = (sıra_konumu + tx) / sıra_payda;
            let ana = derinlik / derinlik_payda * ilerleme;
            düğümler[sıra].konum = match seri.yön {
                AğaçYönü::SoldanSağa => (
                    alan.x + ana * alan.genişlik,
                    alan.y + çapraz * alan.yükseklik,
                ),
                AğaçYönü::SağdanSola => (
                    alan.sağ() - ana * alan.genişlik,
                    alan.y + çapraz * alan.yükseklik,
                ),
                AğaçYönü::ÜsttenAlta => (
                    alan.x + çapraz * alan.genişlik,
                    alan.y + ana * alan.yükseklik,
                ),
                AğaçYönü::AlttanÜste => (
                    alan.x + çapraz * alan.genişlik,
                    alan.alt() - ana * alan.yükseklik,
                ),
            };
        }
    }

    let model_ölçeği = seri.yakınlaştırma.max(0.01);
    let toplam_ölçek = (model_ölçeği * görünüm.2.max(0.01)).clamp(0.01, 100.0);
    let merkez = seri
        .merkez
        .map(|(x, y)| {
            (
                alan.x + x.çöz(alan.genişlik),
                alan.y + y.çöz(alan.yükseklik),
            )
        })
        .unwrap_or_else(|| alan.merkez());
    for &sıra in &görünürler {
        let konum = düğümler[sıra].konum;
        düğümler[sıra].konum = (
            merkez.0 + (konum.0 - merkez.0) * toplam_ölçek + görünüm.0,
            merkez.1 + (konum.1 - merkez.1) * toplam_ölçek + görünüm.1,
        );
        // Radial Bezier'in ham yarıçapı grup ölçeğini taşır; açı değişmez.
        if radyal {
            düğümler[sıra].ham.1 *= toplam_ölçek;
        }
    }
    düğümler
}

fn alt_soy_sıraları(
    düğümler: &[YerleşikDüğüm<'_>],
    kök: usize,
    kendini_ekle: bool,
) -> Vec<usize> {
    fn gez(düğümler: &[YerleşikDüğüm<'_>], sıra: usize, çıktı: &mut Vec<usize>) {
        çıktı.push(sıra);
        for &çocuk in &düğümler[sıra].çocuklar {
            gez(düğümler, çocuk, çıktı);
        }
    }
    let mut çıktı = Vec::new();
    if kendini_ekle {
        gez(düğümler, kök, &mut çıktı);
    } else {
        for &çocuk in &düğümler[kök].çocuklar {
            gez(düğümler, çocuk, &mut çıktı);
        }
    }
    çıktı
}

fn ata_sıraları(düğümler: &[YerleşikDüğüm<'_>], sıra: usize) -> Vec<usize> {
    let mut sonuç = Vec::new();
    let mut geçerli = Some(sıra);
    while let Some(i) = geçerli {
        if düğümler[i].kaynak.is_some() {
            sonuç.push(i);
        }
        geçerli = düğümler[i].üst;
    }
    sonuç
}

fn öğe_stilini_yamala(taban: &ÖğeStili, yama: &ÖğeStili) -> ÖğeStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk.clone_from(&yama.renk);
    }
    if yama.kenarlık_rengi.is_some() {
        sonuç.kenarlık_rengi = yama.kenarlık_rengi;
    }
    if yama.kenarlık_kalınlığı > 0.0 || taban.kenarlık_kalınlığı == 0.0 {
        sonuç.kenarlık_kalınlığı = yama.kenarlık_kalınlığı;
    }
    if yama.opaklık.is_some() {
        sonuç.opaklık = yama.opaklık;
    }
    if yama.gölge_bulanıklığı > 0.0 || yama.gölge_rengi.is_some() {
        sonuç.gölge_bulanıklığı = yama.gölge_bulanıklığı;
        sonuç.gölge_rengi = yama.gölge_rengi;
        sonuç.gölge_kayması = yama.gölge_kayması;
    }
    sonuç
}

fn çizgi_stilini_yamala(taban: &ÇizgiStili, yama: &ÇizgiStili) -> ÇizgiStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk = yama.renk;
    }
    sonuç.kalınlık = yama.kalınlık;
    sonuç.tür = yama.tür;
    sonuç.opaklık = yama.opaklık;
    if yama.gölge_bulanıklığı > 0.0 || yama.gölge_rengi.is_some() {
        sonuç.gölge_bulanıklığı = yama.gölge_bulanıklığı;
        sonuç.gölge_rengi = yama.gölge_rengi;
        sonuç.gölge_kayması = yama.gölge_kayması;
    }
    sonuç
}

fn çözülmüş_öğe_stili(
    seri: &AğaçSerisi,
    düğüm: &AğaçDüğümü,
    durum: DüğümDurumu,
) -> ÖğeStili {
    let mut stil = seri.öğe_stili.clone();
    if let Some(renk) = düğüm.renk {
        stil.renk = Some(Dolgu::Düz(renk));
    }
    if let Some(yama) = &düğüm.öğe_stili {
        stil = öğe_stilini_yamala(&stil, yama);
    }
    let yama = match durum {
        DüğümDurumu::Vurgu => düğüm
            .vurgu_öğe_stili
            .as_ref()
            .or(seri.vurgu_öğe_stili.as_ref()),
        DüğümDurumu::Bulanık => düğüm
            .bulanık_öğe_stili
            .as_ref()
            .or(seri.bulanık_öğe_stili.as_ref()),
        DüğümDurumu::Normal => None,
    };
    if let Some(yama) = yama {
        stil = öğe_stilini_yamala(&stil, yama);
    }
    if durum == DüğümDurumu::Bulanık && yama.is_none() {
        stil.opaklık = Some(stil.opaklık.unwrap_or(1.0) * 0.1);
    }
    stil
}

fn çözülmüş_çizgi_stili(
    seri: &AğaçSerisi,
    düğüm: &AğaçDüğümü,
    durum: DüğümDurumu,
) -> ÇizgiStili {
    let mut stil = seri.çizgi_stili.clone();
    if let Some(yama) = &düğüm.çizgi_stili {
        stil = çizgi_stilini_yamala(&stil, yama);
    }
    let yama = match durum {
        DüğümDurumu::Vurgu => düğüm
            .vurgu_çizgi_stili
            .as_ref()
            .or(seri.vurgu_çizgi_stili.as_ref()),
        DüğümDurumu::Bulanık => düğüm
            .bulanık_çizgi_stili
            .as_ref()
            .or(seri.bulanık_çizgi_stili.as_ref()),
        DüğümDurumu::Normal => None,
    };
    if let Some(yama) = yama {
        stil = çizgi_stilini_yamala(&stil, yama);
    }
    if durum == DüğümDurumu::Bulanık && yama.is_none() {
        stil.opaklık *= 0.1;
    }
    stil
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DüğümDurumu {
    Normal,
    Vurgu,
    Bulanık,
}

fn düğüm_durumları(
    seri: &AğaçSerisi,
    düğümler: &[YerleşikDüğüm<'_>],
    fare: Option<(f32, f32)>,
    düğüm_ölçeği: f32,
) -> Vec<DüğümDurumu> {
    let vurgulu = fare.and_then(|fare| {
        düğümler
            .iter()
            .enumerate()
            .skip(1)
            .rev()
            .find_map(|(sıra, düğüm)| {
                let kaynak = düğüm.kaynak?;
                let boyut = kaynak.sembol_boyutu.unwrap_or(seri.sembol_boyutu) * düğüm_ölçeği;
                let dx = fare.0 - düğüm.konum.0;
                let dy = fare.1 - düğüm.konum.1;
                (dx * dx + dy * dy <= (boyut / 2.0 + 4.0).max(8.0).powi(2)).then_some(sıra)
            })
    });
    let Some(vurgulu) = vurgulu else {
        return vec![DüğümDurumu::Normal; düğümler.len()];
    };
    let mut odak = HashSet::from([vurgulu]);
    match seri.vurgu_odağı {
        AğaçVurguOdağı::Ata => odak.extend(ata_sıraları(düğümler, vurgulu)),
        AğaçVurguOdağı::AltSoy => {
            odak.extend(alt_soy_sıraları(düğümler, vurgulu, true));
        }
        AğaçVurguOdağı::İlişkili => {
            odak.extend(ata_sıraları(düğümler, vurgulu));
            odak.extend(alt_soy_sıraları(düğümler, vurgulu, true));
        }
        AğaçVurguOdağı::Yok | AğaçVurguOdağı::Öz => {}
    }
    (0..düğümler.len())
        .map(|sıra| {
            if sıra == vurgulu {
                DüğümDurumu::Vurgu
            } else if seri.vurgu_odağı != AğaçVurguOdağı::Yok && !odak.contains(&sıra) {
                DüğümDurumu::Bulanık
            } else {
                DüğümDurumu::Normal
            }
        })
        .collect()
}

fn radyal_nokta(merkez: (f32, f32), açı: f32, yarıçap: f32) -> (f32, f32) {
    let açı = açı - std::f32::consts::FRAC_PI_2;
    (
        merkez.0 + yarıçap * açı.cos(),
        merkez.1 + yarıçap * açı.sin(),
    )
}

fn eğri_kenar_yolu(
    seri: &AğaçSerisi,
    alan: Dikdörtgen,
    üst: &YerleşikDüğüm<'_>,
    çocuk: &YerleşikDüğüm<'_>,
) -> Yol {
    let mut yol = Yol::yeni();
    yol.taşı(üst.konum);
    let eğrilik = seri.kenar_eğriliği;
    if seri.yerleşim == AğaçYerleşimi::Radyal {
        let merkez = alan.merkez();
        let c1 = radyal_nokta(
            merkez,
            üst.ham.0,
            üst.ham.1 + (çocuk.ham.1 - üst.ham.1) * eğrilik,
        );
        let c2 = radyal_nokta(
            merkez,
            çocuk.ham.0,
            çocuk.ham.1 + (üst.ham.1 - çocuk.ham.1) * eğrilik,
        );
        yol.kübik(c1, c2, çocuk.konum);
    } else {
        let (c1, c2) = match seri.yön {
            AğaçYönü::SoldanSağa | AğaçYönü::SağdanSola => (
                (
                    üst.konum.0 + (çocuk.konum.0 - üst.konum.0) * eğrilik,
                    üst.konum.1,
                ),
                (
                    çocuk.konum.0 + (üst.konum.0 - çocuk.konum.0) * eğrilik,
                    çocuk.konum.1,
                ),
            ),
            AğaçYönü::ÜsttenAlta | AğaçYönü::AlttanÜste => (
                (
                    üst.konum.0,
                    üst.konum.1 + (çocuk.konum.1 - üst.konum.1) * eğrilik,
                ),
                (
                    çocuk.konum.0,
                    çocuk.konum.1 + (üst.konum.1 - çocuk.konum.1) * eğrilik,
                ),
            ),
        };
        yol.kübik(c1, c2, çocuk.konum);
    }
    yol
}

fn kırık_kenar_yolu(
    seri: &AğaçSerisi,
    üst: &YerleşikDüğüm<'_>,
    çocuklar: &[&YerleşikDüğüm<'_>],
) -> Yol {
    let mut yol = Yol::yeni();
    let Some(ilk) = çocuklar.first() else {
        return yol;
    };
    if çocuklar.len() == 1 {
        yol.taşı(üst.konum);
        yol.çiz(ilk.konum);
        return yol;
    }
    let son = çocuklar.last().unwrap_or(ilk);
    let yatay_büyüme = matches!(seri.yön, AğaçYönü::SoldanSağa | AğaçYönü::SağdanSola);
    if yatay_büyüme {
        let çatal_x = üst.konum.0 + (son.konum.0 - üst.konum.0) * seri.kenar_çatal_oranı;
        yol.taşı(üst.konum);
        yol.çiz((çatal_x, üst.konum.1));
        yol.taşı(ilk.konum);
        yol.çiz((çatal_x, ilk.konum.1));
        yol.çiz((çatal_x, son.konum.1));
        yol.çiz(son.konum);
        for çocuk in çocuklar
            .iter()
            .skip(1)
            .take(çocuklar.len().saturating_sub(2))
        {
            yol.taşı(çocuk.konum);
            yol.çiz((çatal_x, çocuk.konum.1));
        }
    } else {
        let çatal_y = üst.konum.1 + (son.konum.1 - üst.konum.1) * seri.kenar_çatal_oranı;
        yol.taşı(üst.konum);
        yol.çiz((üst.konum.0, çatal_y));
        yol.taşı(ilk.konum);
        yol.çiz((ilk.konum.0, çatal_y));
        yol.çiz((son.konum.0, çatal_y));
        yol.çiz(son.konum);
        for çocuk in çocuklar
            .iter()
            .skip(1)
            .take(çocuklar.len().saturating_sub(2))
        {
            yol.taşı(çocuk.konum);
            yol.çiz((çocuk.konum.0, çatal_y));
        }
    }
    yol
}

fn yolu_stille_çiz(çizici: &mut dyn ÇizimYüzeyi, yol: &Yol, stil: &ÇizgiStili) {
    let renk = stil
        .renk
        .unwrap_or_else(tema::nötr_20)
        .opaklık(stil.opaklık);
    if let Some(gölge) = stil.gölge_rengi
        && (stil.gölge_bulanıklığı > 0.0 || stil.gölge_kayması != (0.0, 0.0))
    {
        çizici.yol_gölgesi(yol, gölge, stil.gölge_bulanıklığı, stil.gölge_kayması);
    }
    çizici.yol_çiz(yol, stil.kalınlık.max(0.0), renk, stil.tür);
}

fn hiza_çöz(etiket: &Etiket, doğal: (YatayHiza, DikeyHiza)) -> (YatayHiza, DikeyHiza) {
    let yatay = etiket
        .yatay_hiza
        .map(|h| match h {
            YazıYatayHizası::Sol => YatayHiza::Sol,
            YazıYatayHizası::Orta => YatayHiza::Orta,
            YazıYatayHizası::Sağ => YatayHiza::Sağ,
        })
        .unwrap_or(doğal.0);
    let dikey = etiket
        .dikey_hiza
        .map(|h| match h {
            YazıDikeyHizası::Üst => DikeyHiza::Üst,
            YazıDikeyHizası::Orta => DikeyHiza::Orta,
            YazıDikeyHizası::Alt => DikeyHiza::Alt,
        })
        .unwrap_or(doğal.1);
    (yatay, dikey)
}

fn etiket_geometrisi(
    seri: &AğaçSerisi,
    düğümler: &[YerleşikDüğüm<'_>],
    sıra: usize,
    etiket: &Etiket,
    sembol_boyutu: f32,
) -> ((f32, f32), YatayHiza, DikeyHiza, f32) {
    let düğüm = &düğümler[sıra];
    let yarıçap = sembol_boyutu / 2.0;
    if seri.yerleşim == AğaçYerleşimi::Radyal {
        let gerçek_kök = düğümler[0].çocuklar[0];
        let kök = &düğümler[gerçek_kök];
        let (mut açı, sol_etiket) =
            if sıra == gerçek_kök && düğüm.açık && !düğüm.çocuklar.is_empty() {
                let ilk = düğümler[*düğüm.çocuklar.first().unwrap_or(&sıra)].konum;
                let son = düğümler[*düğüm.çocuklar.last().unwrap_or(&sıra)].konum;
                let merkez = ((ilk.0 + son.0) / 2.0, (ilk.1 + son.1) / 2.0);
                let mut açı = (merkez.1 - kök.konum.1).atan2(merkez.0 - kök.konum.0);
                if açı < 0.0 {
                    açı += std::f32::consts::TAU;
                }
                let sol = merkez.0 < kök.konum.0;
                if sol {
                    açı -= std::f32::consts::PI;
                }
                (açı, sol)
            } else {
                let mut açı = (düğüm.konum.1 - kök.konum.1).atan2(düğüm.konum.0 - kök.konum.0);
                if açı < 0.0 {
                    açı += std::f32::consts::TAU;
                }
                let uç = düğüm
                    .kaynak
                    .is_some_and(|kaynak| kaynak.çocuklar.is_empty() || !düğüm.açık);
                let sol = if uç {
                    düğüm.konum.0 < kök.konum.0
                } else {
                    düğüm.konum.0 > kök.konum.0
                };
                if (uç && sol) || (!uç && !sol) {
                    açı -= std::f32::consts::PI;
                }
                (açı, sol)
            };
        if !açı.is_finite() {
            açı = 0.0;
        }
        let yön = if sol_etiket { -1.0 } else { 1.0 };
        let doğal = if sol_etiket {
            (YatayHiza::Sağ, DikeyHiza::Orta)
        } else {
            (YatayHiza::Sol, DikeyHiza::Orta)
        };
        let (yatay, dikey) = hiza_çöz(etiket, doğal);
        let dönüş = match etiket.döndürme {
            EtiketDöndürme::Derece(derece) => -derece.to_radians(),
            // zrender, radyal Tree'nin otomatik etiket dönüşünü düğümün
            // yerel eksenine uygular. Bizim AfinMatris/Canvas yönümüz için
            // aynı ekran yönü `açı`dır (açık `label.rotate` ise diğer
            // etiketlerle aynı ECharts derece sözleşmesini korur).
            _ => açı,
        };
        // `textConfig.position: left|right` önce sembolün yerel uzayında
        // çözülür, ardından etiket ve uzaklık vektörü birlikte döner.
        // Yalnız metni döndürüp çapayı yatay bırakmak yoğun ağaçlarda
        // etiketleri daire yerine elmas biçiminde topluyordu.
        let uzaklık = yön * (yarıçap + etiket.uzaklık);
        let çapa = (
            düğüm.konum.0 + uzaklık * dönüş.cos(),
            düğüm.konum.1 + uzaklık * dönüş.sin(),
        );
        return (çapa, yatay, dikey, dönüş);
    }

    let (çapa, doğal) = match etiket.konum {
        EtiketKonumu::Sol | EtiketKonumu::Dış => (
            (düğüm.konum.0 - yarıçap - etiket.uzaklık, düğüm.konum.1),
            (YatayHiza::Sağ, DikeyHiza::Orta),
        ),
        EtiketKonumu::Sağ => (
            (düğüm.konum.0 + yarıçap + etiket.uzaklık, düğüm.konum.1),
            (YatayHiza::Sol, DikeyHiza::Orta),
        ),
        EtiketKonumu::Üst => (
            (düğüm.konum.0, düğüm.konum.1 - yarıçap - etiket.uzaklık),
            (YatayHiza::Orta, DikeyHiza::Alt),
        ),
        EtiketKonumu::Alt => (
            (düğüm.konum.0, düğüm.konum.1 + yarıçap + etiket.uzaklık),
            (YatayHiza::Orta, DikeyHiza::Üst),
        ),
        _ => (düğüm.konum, (YatayHiza::Orta, DikeyHiza::Orta)),
    };
    let (yatay, dikey) = hiza_çöz(etiket, doğal);
    let dönüş = match etiket.döndürme {
        EtiketDöndürme::Derece(derece) => -derece.to_radians(),
        _ => 0.0,
    };
    (çapa, yatay, dikey, dönüş)
}

/// Tree yerleşiminin rasterleyiciden bağımsız, kararlı geometri özeti.
/// Resmî örnek kanıtı düğüm/kenar/etiket koordinatlarını bu özetle ayrıca
/// kilitler; yoğun yazının toplam piksel oranında gerçek yerleşim hatasını
/// saklamasını önler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AğaçSahneÖzeti {
    pub düğüm_sayısı: usize,
    pub kenar_sayısı: usize,
    pub kenar_yolu_sayısı: usize,
    pub etiket_sayısı: usize,
    pub daraltılmış_düğüm_sayısı: usize,
    pub koordinat_sayısı: usize,
    pub fnv1a_64: u64,
}

fn özet_bayt(özet: &mut u64, bayt: u8) {
    *özet ^= u64::from(bayt);
    *özet = özet.wrapping_mul(0x0000_0100_0000_01b3);
}

fn özet_u64(özet: &mut u64, değer: u64) {
    for bayt in değer.to_le_bytes() {
        özet_bayt(özet, bayt);
    }
}

fn özet_f32(özet: &mut u64, değer: f32) {
    let nicemlenmiş = if değer.is_finite() {
        (f64::from(değer) * 1_000.0).round() as i64
    } else {
        i64::MIN
    };
    özet_u64(özet, nicemlenmiş as u64);
}

fn özet_metin(özet: &mut u64, metin: &str) {
    özet_u64(özet, metin.len() as u64);
    for &bayt in metin.as_bytes() {
        özet_bayt(özet, bayt);
    }
}

fn özet_yol(özet: &mut u64, yol: &Yol, koordinat_sayısı: &mut usize) {
    özet_u64(özet, yol.komutlar.len() as u64);
    for komut in &yol.komutlar {
        match *komut {
            YolKomutu::Taşı(nokta) => {
                özet_bayt(özet, 0);
                özet_f32(özet, nokta.0);
                özet_f32(özet, nokta.1);
                *koordinat_sayısı = koordinat_sayısı.saturating_add(2);
            }
            YolKomutu::Çiz(nokta) => {
                özet_bayt(özet, 1);
                özet_f32(özet, nokta.0);
                özet_f32(özet, nokta.1);
                *koordinat_sayısı = koordinat_sayısı.saturating_add(2);
            }
            YolKomutu::Kübik { k1, k2, uç } => {
                özet_bayt(özet, 2);
                for değer in [k1.0, k1.1, k2.0, k2.1, uç.0, uç.1] {
                    özet_f32(özet, değer);
                }
                *koordinat_sayısı = koordinat_sayısı.saturating_add(6);
            }
            YolKomutu::Yay {
                yarıçap,
                büyük_yay,
                süpürme,
                uç,
            } => {
                özet_bayt(özet, 3);
                özet_f32(özet, yarıçap);
                özet_bayt(özet, u8::from(büyük_yay));
                özet_bayt(özet, u8::from(süpürme));
                özet_f32(özet, uç.0);
                özet_f32(özet, uç.1);
                *koordinat_sayısı = koordinat_sayısı.saturating_add(3);
            }
            YolKomutu::Kapat => özet_bayt(özet, 4),
        }
    }
}

/// `ağaç_çiz` ile aynı yerleşim ve görünüm dönüşümünün kesin özetini üretir.
pub fn ağaç_sahne_özeti(
    seri: &AğaçSerisi,
    tuval: Dikdörtgen,
    görünüm: (f32, f32, f32),
) -> AğaçSahneÖzeti {
    let alan = görünüm_alanı(seri, tuval);
    let düğümler = yerleşimi_hesapla(seri, alan, 1.0, görünüm);
    let mut özet = 0xcbf2_9ce4_8422_2325_u64;
    let mut düğüm_sayısı = 0usize;
    let mut kenar_sayısı = 0usize;
    let mut kenar_yolu_sayısı = 0usize;
    let mut etiket_sayısı = 0usize;
    let mut daraltılmış_düğüm_sayısı = 0usize;
    let mut koordinat_sayısı = 0usize;

    özet_bayt(&mut özet, seri.yerleşim as u8);
    özet_bayt(&mut özet, seri.yön as u8);
    özet_bayt(&mut özet, seri.kenar_biçimi as u8);
    özet_f32(&mut özet, seri.kenar_çatal_oranı);
    özet_f32(&mut özet, seri.kenar_eğriliği);

    for (sıra, düğüm) in düğümler.iter().enumerate().skip(1) {
        let Some(kaynak) = düğüm.kaynak else {
            continue;
        };
        düğüm_sayısı = düğüm_sayısı.saturating_add(1);
        özet_u64(&mut özet, düğüm.veri_sırası as u64);
        let üst_veri_sırası = düğüm
            .üst
            .and_then(|üst| düğümler.get(üst))
            .and_then(|üst| üst.kaynak.map(|_| üst.veri_sırası))
            .unwrap_or(usize::MAX);
        özet_u64(&mut özet, üst_veri_sırası as u64);
        özet_u64(&mut özet, düğüm.derinlik as u64);
        özet_f32(&mut özet, düğüm.konum.0);
        özet_f32(&mut özet, düğüm.konum.1);
        koordinat_sayısı = koordinat_sayısı.saturating_add(2);
        özet_bayt(&mut özet, u8::from(düğüm.açık));
        özet_metin(&mut özet, &kaynak.ad);
        if !düğüm.açık && !kaynak.çocuklar.is_empty() {
            daraltılmış_düğüm_sayısı = daraltılmış_düğüm_sayısı.saturating_add(1);
        }
        if düğüm
            .üst
            .and_then(|üst| düğümler.get(üst))
            .is_some_and(|üst| üst.kaynak.is_some())
        {
            kenar_sayısı = kenar_sayısı.saturating_add(1);
        }

        let görünür_uç = kaynak.çocuklar.is_empty() || !düğüm.açık;
        let mut etiket = if görünür_uç {
            seri.yaprak_etiketi.uygula(&seri.etiket)
        } else {
            seri.etiket.clone()
        };
        if let Some(yama) = &kaynak.etiket {
            etiket = yama.uygula(&etiket);
        }
        if etiket.göster {
            let boyut = kaynak.sembol_boyutu.unwrap_or(seri.sembol_boyutu);
            let sembol_kayması = kaynak.sembol_kayması.unwrap_or(seri.sembol_kayması);
            let sembol_kayması = (sembol_kayması.0.çöz(boyut), sembol_kayması.1.çöz(boyut));
            let sembol_döndürme = kaynak
                .sembol_döndürme
                .or(seri.sembol_döndürme)
                .unwrap_or(0.0);
            özet_f32(&mut özet, sembol_kayması.0);
            özet_f32(&mut özet, sembol_kayması.1);
            özet_f32(&mut özet, sembol_döndürme);
            özet_bayt(
                &mut özet,
                u8::from(
                    kaynak
                        .sembol_oranını_koru
                        .unwrap_or(seri.sembol_oranını_koru),
                ),
            );
            let (çapa, yatay, dikey, dönüş) =
                etiket_geometrisi(seri, &düğümler, sıra, &etiket, boyut);
            özet_f32(&mut özet, çapa.0 + etiket.kayma.0 + sembol_kayması.0);
            özet_f32(&mut özet, çapa.1 + etiket.kayma.1 + sembol_kayması.1);
            özet_f32(&mut özet, dönüş);
            özet_bayt(&mut özet, yatay as u8);
            özet_bayt(&mut özet, dikey as u8);
            koordinat_sayısı = koordinat_sayısı.saturating_add(3);
            etiket_sayısı = etiket_sayısı.saturating_add(1);
        }
    }

    match seri.kenar_biçimi {
        AğaçKenarBiçimi::Kırık if seri.yerleşim == AğaçYerleşimi::Dik => {
            for düğüm in düğümler.iter().skip(1) {
                if düğüm.kaynak.is_none() || düğüm.çocuklar.is_empty() || !düğüm.açık {
                    continue;
                }
                let çocuklar = düğüm
                    .çocuklar
                    .iter()
                    .filter_map(|&çocuk| düğümler.get(çocuk))
                    .collect::<Vec<_>>();
                let yol = kırık_kenar_yolu(seri, düğüm, &çocuklar);
                özet_yol(&mut özet, &yol, &mut koordinat_sayısı);
                kenar_yolu_sayısı = kenar_yolu_sayısı.saturating_add(1);
            }
        }
        AğaçKenarBiçimi::Eğri | AğaçKenarBiçimi::Kırık => {
            // Radyal + polyline ECharts'ta geçersizdir; üretim çizicisinin
            // güvenli eğri düşüşü sahne özetinde de birebir korunur.
            for düğüm in düğümler.iter().skip(1) {
                let Some(üst_sıra) = düğüm.üst else {
                    continue;
                };
                if düğümler[üst_sıra].kaynak.is_none() {
                    continue;
                }
                let yol = eğri_kenar_yolu(seri, alan, &düğümler[üst_sıra], düğüm);
                özet_yol(&mut özet, &yol, &mut koordinat_sayısı);
                kenar_yolu_sayısı = kenar_yolu_sayısı.saturating_add(1);
            }
        }
    }

    AğaçSahneÖzeti {
        düğüm_sayısı,
        kenar_sayısı,
        kenar_yolu_sayısı,
        etiket_sayısı,
        daraltılmış_düğüm_sayısı,
        koordinat_sayısı,
        fnv1a_64: özet,
    }
}

#[allow(clippy::too_many_arguments)]
fn etiketi_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &AğaçSerisi,
    düğümler: &[YerleşikDüğüm<'_>],
    sıra: usize,
    kaynak: &AğaçDüğümü,
    durum: DüğümDurumu,
    sembol_boyutu: f32,
    sembol_kayması: (f32, f32),
) {
    let görünür_uç = kaynak.çocuklar.is_empty() || !düğümler[sıra].açık;
    let mut etiket = if görünür_uç {
        seri.yaprak_etiketi.uygula(&seri.etiket)
    } else {
        seri.etiket.clone()
    };
    if let Some(yama) = &kaynak.etiket {
        etiket = yama.uygula(&etiket);
    }
    let durum_yaması = match durum {
        DüğümDurumu::Vurgu => kaynak
            .vurgu_etiketi
            .as_ref()
            .or(seri.vurgu_etiketi.as_ref()),
        DüğümDurumu::Bulanık => kaynak
            .bulanık_etiketi
            .as_ref()
            .or(seri.bulanık_etiketi.as_ref()),
        DüğümDurumu::Normal => None,
    };
    if let Some(yama) = durum_yaması {
        etiket = yama.uygula(&etiket);
    }
    if !etiket.göster {
        return;
    }
    let (mut çapa, yatay, dikey, dönüş) =
        etiket_geometrisi(seri, düğümler, sıra, &etiket, sembol_boyutu);
    çapa.0 += etiket.kayma.0 + sembol_kayması.0;
    çapa.1 += etiket.kayma.1 + sembol_kayması.1;
    let metin = etiket
        .biçimleyici
        .as_ref()
        .map(|b| {
            b.uygula_bağlamla(
                kaynak.değer.unwrap_or_default(),
                &kaynak.ad,
                seri.ad.as_deref().unwrap_or_default(),
                &kaynak.ad,
            )
        })
        .unwrap_or_else(|| kaynak.ad.clone());
    let boyut = etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let mut renk = etiket.yazı.renk.unwrap_or_else(tema::birincil_metin);
    let mut opaklık = etiket.yazı.opaklık.unwrap_or(1.0);
    if durum == DüğümDurumu::Bulanık && durum_yaması.is_none() {
        opaklık *= 0.1;
    }
    renk = renk.opaklık(opaklık);
    let dönüşüm = AfinMatris::ötele(çapa.0, çapa.1).çarp(AfinMatris::döndür(dönüş));

    if let Some(arkaplan) = &etiket.yazı.arkaplan {
        let (metin_genişliği, metin_yüksekliği) = çizici.yazı_ölç(&metin, boyut);
        let [üst, sağ, alt, sol] = etiket.yazı.iç_boşluk.unwrap_or([0.0; 4]);
        let genişlik = metin_genişliği + sol + sağ;
        let yükseklik = metin_yüksekliği + üst + alt;
        let x = match yatay {
            YatayHiza::Sol => -sol,
            YatayHiza::Orta => -genişlik / 2.0,
            YatayHiza::Sağ => -genişlik + sağ,
        };
        let y = match dikey {
            DikeyHiza::Üst => -üst,
            DikeyHiza::Orta => -yükseklik / 2.0,
            DikeyHiza::Alt => -yükseklik + alt,
        };
        let mut yol = Yol::yeni();
        let köşeler = [
            (x, y),
            (x + genişlik, y),
            (x + genişlik, y + yükseklik),
            (x, y + yükseklik),
        ];
        yol.taşı(dönüşüm.noktayı_dönüştür(köşeler[0]));
        for köşe in köşeler.iter().skip(1) {
            yol.çiz(dönüşüm.noktayı_dönüştür(*köşe));
        }
        yol.kapat();
        çizici.yol_doldur(&yol, &arkaplan.opaklık(opaklık));
        if let (Some(kenarlık), Some(kalınlık)) =
            (etiket.yazı.kenarlık_rengi, etiket.yazı.kenarlık_kalınlığı)
            && kalınlık > 0.0
        {
            çizici.yol_çiz(
                &yol,
                kalınlık,
                kenarlık.opaklık(opaklık),
                crate::model::stil::ÇizgiTürü::Düz,
            );
        }
    }
    if let Some(aile) = etiket.yazı.aile.as_deref() {
        çizici.dönüşümlü_aileli_yazı(
            &metin,
            (0.0, 0.0),
            yatay,
            dikey,
            boyut,
            renk,
            etiket.yazı.kalın,
            aile,
            dönüşüm,
        );
    } else {
        çizici.dönüşümlü_yazı(
            &metin,
            (0.0, 0.0),
            yatay,
            dikey,
            boyut,
            renk,
            etiket.yazı.kalın,
            dönüşüm,
        );
    }
}

/// Ağaç serisini çizer.
#[allow(clippy::too_many_arguments)]
pub fn ağaç_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &AğaçSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    seri_rengi: Renk,
    ilerleme: f32,
    görünüm: (f32, f32, f32),
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let alan = görünüm_alanı(seri, tuval);
    let düğümler = yerleşimi_hesapla(seri, alan, ilerleme, görünüm);
    if düğümler.len() <= 1 {
        return;
    }
    let toplam_ölçek = (seri.yakınlaştırma * görünüm.2.max(0.01)).max(0.01);
    let düğüm_ölçeği = ((toplam_ölçek - 1.0) * seri.düğüm_ölçek_oranı + 1.0).max(0.01);
    let durumlar = düğüm_durumları(seri, &düğümler, fare, düğüm_ölçeği);

    // 1) Kenarlar düğümlerin altında çizilir.
    match seri.kenar_biçimi {
        AğaçKenarBiçimi::Eğri => {
            for (sıra, düğüm) in düğümler.iter().enumerate().skip(1) {
                let (Some(üst_sıra), Some(kaynak)) = (düğüm.üst, düğüm.kaynak) else {
                    continue;
                };
                if düğümler[üst_sıra].kaynak.is_none() {
                    continue;
                }
                let yol = eğri_kenar_yolu(seri, alan, &düğümler[üst_sıra], düğüm);
                let stil = çözülmüş_çizgi_stili(seri, kaynak, durumlar[sıra]);
                yolu_stille_çiz(çizici, &yol, &stil);
            }
        }
        AğaçKenarBiçimi::Kırık if seri.yerleşim == AğaçYerleşimi::Dik => {
            for (sıra, düğüm) in düğümler.iter().enumerate().skip(1) {
                let Some(kaynak) = düğüm.kaynak else {
                    continue;
                };
                if düğüm.çocuklar.is_empty() || !düğüm.açık {
                    continue;
                }
                let çocuklar = düğüm
                    .çocuklar
                    .iter()
                    .filter_map(|&çocuk| düğümler.get(çocuk))
                    .collect::<Vec<_>>();
                let yol = kırık_kenar_yolu(seri, düğüm, &çocuklar);
                let stil = çözülmüş_çizgi_stili(seri, kaynak, durumlar[sıra]);
                yolu_stille_çiz(çizici, &yol, &stil);
            }
        }
        AğaçKenarBiçimi::Kırık => {
            // ECharts bu birleşimi geliştirme kipinde hata sayar; üretimde
            // görünümü kaybetmemek için radyal eğriye güvenli düşüş yapılır.
            for (sıra, düğüm) in düğümler.iter().enumerate().skip(1) {
                let (Some(üst_sıra), Some(kaynak)) = (düğüm.üst, düğüm.kaynak) else {
                    continue;
                };
                if düğümler[üst_sıra].kaynak.is_none() {
                    continue;
                }
                let yol = eğri_kenar_yolu(seri, alan, &düğümler[üst_sıra], düğüm);
                let stil = çözülmüş_çizgi_stili(seri, kaynak, durumlar[sıra]);
                yolu_stille_çiz(çizici, &yol, &stil);
            }
        }
    }

    // 2) Semboller, etiketler ve gerçek veri sıralı isabetler.
    for (sıra, düğüm) in düğümler.iter().enumerate().skip(1) {
        let Some(kaynak) = düğüm.kaynak else {
            continue;
        };
        let stil = çözülmüş_öğe_stili(seri, kaynak, durumlar[sıra]);
        let temel_renk = stil.renk.as_ref().map(Dolgu::temsilî).unwrap_or(seri_rengi);
        let mut boyut = kaynak.sembol_boyutu.unwrap_or(seri.sembol_boyutu) * düğüm_ölçeği;
        if durumlar[sıra] == DüğümDurumu::Vurgu && seri.vurgu_ölçekle {
            boyut *= 1.1;
        }
        let mut sembol = kaynak.sembol.as_ref().unwrap_or(&seri.sembol);
        let dolu_sembol;
        if matches!(sembol, Sembol::İçiBoşDaire) && !düğüm.açık && !kaynak.çocuklar.is_empty()
        {
            dolu_sembol = Sembol::Daire;
            sembol = &dolu_sembol;
        }
        let kayma = kaynak.sembol_kayması.unwrap_or(seri.sembol_kayması);
        let kayma = (kayma.0.çöz(boyut), kayma.1.çöz(boyut));
        let konum = (düğüm.konum.0 + kayma.0, düğüm.konum.1 + kayma.1);
        let oranı_koru = kaynak
            .sembol_oranını_koru
            .unwrap_or(seri.sembol_oranını_koru);
        let döndürme = kaynak
            .sembol_döndürme
            .or(seri.sembol_döndürme)
            .unwrap_or(0.0);
        let opaklık = stil.opaklık.unwrap_or(1.0);
        if let Some(gölge) = stil.gölge_rengi
            && (stil.gölge_bulanıklığı > 0.0 || stil.gölge_kayması != (0.0, 0.0))
            && let Some(yol) =
                sembol_dönüşümlü_yolu(sembol, konum, boyut, oranı_koru, döndürme)
        {
            çizici.yol_gölgesi(
                &yol,
                gölge.opaklık(opaklık),
                stil.gölge_bulanıklığı,
                stil.gölge_kayması,
            );
        }
        let kenarlık = (stil.kenarlık_kalınlığı > 0.0).then_some((
            stil.kenarlık_kalınlığı,
            stil.kenarlık_rengi.unwrap_or(temel_renk),
        ));
        sembol_stilli_dönüşümlü_çiz(
            çizici,
            sembol,
            konum,
            boyut,
            döndürme,
            temel_renk,
            stil.renk.as_ref(),
            kenarlık,
            opaklık,
            oranı_koru,
        );
        etiketi_çiz(
            çizici,
            seri,
            &düğümler,
            sıra,
            kaynak,
            durumlar[sıra],
            boyut,
            kayma,
        );
        if !seri.sessiz {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: düğüm.veri_sırası,
                seri_adı: seri.ad.clone(),
                ad: Some(kaynak.ad.clone()),
                değer: kaynak.değer,
                geometri: İsabetGeometrisi::Daire {
                    merkez: konum,
                    yarıçap: (boyut / 2.0 + 4.0).max(8.0),
                },
            });
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;

    fn örnek_ağaç() -> AğaçDüğümü {
        AğaçDüğümü::dal(
            "root",
            vec![
                AğaçDüğümü::dal(
                    "a",
                    vec![AğaçDüğümü::yaprak("a1", 1.0), AğaçDüğümü::yaprak("a2", 2.0)],
                ),
                AğaçDüğümü::dal("b", vec![AğaçDüğümü::yaprak("b1", 3.0)]),
            ],
        )
    }

    #[test]
    fn dört_dik_yön_aynı_düğüm_sayısını_korur() {
        for yön in [
            AğaçYönü::SoldanSağa,
            AğaçYönü::SağdanSola,
            AğaçYönü::ÜsttenAlta,
            AğaçYönü::AlttanÜste,
        ] {
            let seri = AğaçSerisi::yeni()
                .kökler([örnek_ağaç()])
                .yön(yön)
                .ilk_ağaç_derinliği(-1);
            let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
            let mut isabetler = Vec::new();
            ağaç_çiz(
                &mut yüzey,
                &seri,
                0,
                Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
                Renk::onaltılık(0x5070dd),
                1.0,
                (0.0, 0.0, 1.0),
                None,
                &mut isabetler,
            );
            assert_eq!(isabetler.len(), 6);
        }
    }

    #[test]
    fn ilk_derinlik_ve_açık_daraltılmış_yaması_görünür_alt_soyu_belirler() {
        let seri = AğaçSerisi::yeni()
            .kökler([örnek_ağaç()])
            .ilk_ağaç_derinliği(1);
        let alan = görünüm_alanı(&seri, Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0));
        let düğümler = yerleşimi_hesapla(&seri, alan, 1.0, (0.0, 0.0, 1.0));
        assert_eq!(düğümler.iter().filter(|d| d.kaynak.is_some()).count(), 3);

        let açık = AğaçSerisi::yeni()
            .kökler([örnek_ağaç().daraltılmış(false)])
            .ilk_ağaç_derinliği(0);
        let alan = görünüm_alanı(&açık, Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0));
        let düğümler = yerleşimi_hesapla(&açık, alan, 1.0, (0.0, 0.0, 1.0));
        assert_eq!(düğümler.iter().filter(|d| d.kaynak.is_some()).count(), 3);
    }

    #[test]
    fn radyal_kök_merkezde_ve_alt_soy_yarıçapta_yerleşir() {
        let seri = AğaçSerisi::yeni()
            .kökler([örnek_ağaç()])
            .yerleşim(AğaçYerleşimi::Radyal)
            .ilk_ağaç_derinliği(-1);
        let alan = görünüm_alanı(&seri, Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0));
        let düğümler = yerleşimi_hesapla(&seri, alan, 1.0, (0.0, 0.0, 1.0));
        let kök = düğümler[0].çocuklar[0];
        assert!((düğümler[kök].konum.0 - alan.merkez().0).abs() < 1e-4);
        assert!((düğümler[kök].konum.1 - alan.merkez().1).abs() < 1e-4);
        assert!(düğümler.iter().skip(2).any(|d| d.ham.1 > 0.0));
    }

    #[test]
    fn kırık_kenar_tek_çocukta_doğrudan_çizgidir() {
        let seri = AğaçSerisi::yeni()
            .kenar_biçimi(AğaçKenarBiçimi::Kırık)
            .kökler([AğaçDüğümü::dal(
                "root",
                vec![AğaçDüğümü::yaprak("leaf", 1.0)],
            )])
            .ilk_ağaç_derinliği(-1);
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let mut isabetler = Vec::new();
        ağaç_çiz(
            &mut yüzey,
            &seri,
            0,
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            Renk::onaltılık(0x5070dd),
            1.0,
            (0.0, 0.0, 1.0),
            None,
            &mut isabetler,
        );
        assert!(yüzey.döküm().contains("çiz "));
    }

    #[test]
    fn sembol_yüzde_kayması_ve_döndürmesi_çizime_ve_isabete_yansır() {
        fn çiz(seri: &AğaçSerisi) -> (String, (f32, f32)) {
            let mut yüzey = KayıtYüzeyi::yeni(200.0, 120.0);
            let mut isabetler = Vec::new();
            ağaç_çiz(
                &mut yüzey,
                seri,
                0,
                Dikdörtgen::yeni(0.0, 0.0, 200.0, 120.0),
                Renk::onaltılık(0x5070dd),
                1.0,
                (0.0, 0.0, 1.0),
                None,
                &mut isabetler,
            );
            let merkez = isabetler
                .iter()
                .find_map(|isabet| match isabet.geometri {
                    İsabetGeometrisi::Daire { merkez, .. } => Some(merkez),
                    _ => None,
                })
                .expect("Tree düğümü isabet alanı üretmeli");
            (yüzey.döküm(), merkez)
        }

        let temel = AğaçSerisi::yeni()
            .kökler([AğaçDüğümü::yaprak("root", 1.0)])
            .sembol(Sembol::Kare)
            .sembol_boyutu(10.0)
            .ilk_ağaç_derinliği(-1);
        let dönüşlü = temel
            .clone()
            .sembol_kayması(crate::model::Uzunluk::Yüzde(100.0), 0)
            .sembol_döndürme(45.0);

        let (temel_döküm, temel_merkez) = çiz(&temel);
        let (dönüşlü_döküm, dönüşlü_merkez) = çiz(&dönüşlü);
        assert!((dönüşlü_merkez.0 - temel_merkez.0 - 10.0).abs() < 1e-4);
        assert!((dönüşlü_merkez.1 - temel_merkez.1).abs() < 1e-4);
        assert_ne!(temel_döküm, dönüşlü_döküm);
    }
}
