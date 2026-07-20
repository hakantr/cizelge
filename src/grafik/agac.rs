//! Ağaç (tree) serisi — `echarts/src/chart/tree` karşılığı: hiyerarşi,
//! soldan sağa düzenli (tidy) yerleşimle çizilir; yapraklar dikeyde eşit
//! aralıklı, iç düğümler çocuklarının ortasındadır.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::sembol_çiz;
use crate::koordinat::Dikdörtgen;
use crate::model::agac::AğaçDüğümü;
use crate::model::seri::AğaçSerisi;
use crate::renk::Renk;
use crate::tema;

/// Yerleşimi hesaplanmış düğüm.
struct YerleşikDüğüm {
    ad: String,
    değer: Option<f64>,
    konum: (f32, f32),
    üst: Option<usize>,
    yaprak: bool,
}

/// Ağacı yerleştirir: yaprak sayacıyla dikey sıra, derinlikle yatay konum.
fn yerleştir(
    düğüm: &AğaçDüğümü,
    derinlik: usize,
    üst: Option<usize>,
    yaprak_sayacı: &mut usize,
    sonuç: &mut Vec<YerleşikDüğüm>,
) -> (usize, f32) {
    let kendi_sıra = sonuç.len();
    sonuç.push(YerleşikDüğüm {
        ad: düğüm.ad.clone(),
        değer: düğüm.değer,
        konum: (derinlik as f32, 0.0),
        üst,
        yaprak: düğüm.yaprak_mı(),
    });
    if düğüm.yaprak_mı() {
        let y = *yaprak_sayacı as f32;
        *yaprak_sayacı += 1;
        if let Some(kayıt) = sonuç.get_mut(kendi_sıra) {
            kayıt.konum.1 = y;
        }
        (kendi_sıra, y)
    } else {
        let mut ilk_y = f32::INFINITY;
        let mut son_y = f32::NEG_INFINITY;
        for çocuk in &düğüm.çocuklar {
            let (_, y) = yerleştir(çocuk, derinlik + 1, Some(kendi_sıra), yaprak_sayacı, sonuç);
            ilk_y = ilk_y.min(y);
            son_y = son_y.max(y);
        }
        let y = if ilk_y.is_finite() {
            (ilk_y + son_y) / 2.0
        } else {
            0.0
        };
        if let Some(kayıt) = sonuç.get_mut(kendi_sıra) {
            kayıt.konum.1 = y;
        }
        (kendi_sıra, y)
    }
}

/// Ağaç serisini çizer.
pub fn ağaç_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &AğaçSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    seri_rengi: Renk,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let alan = Dikdörtgen::yeni(
        tuval.x + seri.sol.çöz(tuval.genişlik),
        tuval.y + seri.üst.çöz(tuval.yükseklik),
        seri.genişlik.çöz(tuval.genişlik),
        seri.yükseklik.çöz(tuval.yükseklik),
    );

    // Yerleşim (soyut koordinatlarda) — sonra alana ölçeklenir.
    let mut düğümler = Vec::new();
    let mut yaprak_sayacı = 0usize;
    for kök in &seri.kökler {
        yerleştir(kök, 0, None, &mut yaprak_sayacı, &mut düğümler);
    }
    if düğümler.is_empty() {
        return;
    }
    let maks_derinlik = düğümler
        .iter()
        .map(|d| d.konum.0)
        .fold(0.0f32, f32::max)
        .max(1.0);
    let yaprak_sayısı = yaprak_sayacı.max(2) as f32 - 1.0;

    let ölçekle = |soyut: (f32, f32)| -> (f32, f32) {
        (
            alan.x + (soyut.0 / maks_derinlik) * alan.genişlik * ilerleme.clamp(0.0, 1.0),
            alan.y + (soyut.1 / yaprak_sayısı) * alan.yükseklik,
        )
    };

    // 1) Bağlantılar (yatay kübik eğriler).
    for düğüm in &düğümler {
        let Some(üst_sıra) = düğüm.üst else {
            continue;
        };
        let Some(üst) = düğümler.get(üst_sıra) else {
            continue;
        };
        let (x0, y0) = ölçekle(üst.konum);
        let (x1, y1) = ölçekle(düğüm.konum);
        let orta = (x0 + x1) / 2.0;
        let mut yol = Yol::yeni();
        yol.taşı((x0, y0));
        yol.kübik((orta, y0), (orta, y1), (x1, y1));
        çizici.yol_çiz(
            &yol,
            1.5,
            tema::nötr_30(),
            crate::model::stil::ÇizgiTürü::Düz,
        );
    }

    // 2) Düğümler ve etiketler.
    for (i, düğüm) in düğümler.iter().enumerate() {
        let konum = ölçekle(düğüm.konum);
        sembol_çiz(
            çizici,
            &crate::model::seri::Sembol::İçiBoşDaire,
            konum,
            seri.sembol_boyutu,
            seri_rengi,
        );
        let boyut = tema::YAZI_KÜÇÜK;
        if düğüm.yaprak {
            çizici.yazı(
                &düğüm.ad,
                (konum.0 + seri.sembol_boyutu / 2.0 + 5.0, konum.1),
                YatayHiza::Sol,
                DikeyHiza::Orta,
                boyut,
                tema::birincil_metin(),
                false,
            );
        } else {
            çizici.yazı(
                &düğüm.ad,
                (konum.0, konum.1 - seri.sembol_boyutu / 2.0 - 4.0),
                YatayHiza::Orta,
                DikeyHiza::Alt,
                boyut,
                tema::ikincil_metin(),
                false,
            );
        }
        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: i,
            seri_adı: seri.ad.clone(),
            ad: Some(düğüm.ad.clone()),
            değer: düğüm.değer,
            geometri: İsabetGeometrisi::Daire {
                merkez: konum,
                yarıçap: (seri.sembol_boyutu / 2.0 + 4.0).max(8.0),
            },
        });
    }
}
