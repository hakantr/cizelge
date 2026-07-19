//! Paralel koordinat — `echarts/src/coord/parallel` + `chart/parallel`
//! karşılığı: dikey eksenler yan yana, her veri öğesi eksenler boyunca
//! bir çoklu çizgidir.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, keskin, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::seri::ParalelSerisi;
use crate::olcek::AralıkÖlçeği;
use crate::renk::Renk;
use crate::tema;

/// Paralel koordinat serisini çizer.
#[allow(clippy::too_many_arguments)]
pub fn paralel_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &ParalelSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    seri_rengi: Renk,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let boyut_sayısı = seri.boyutlar.len();
    if boyut_sayısı < 2 {
        return;
    }
    let alan = Dikdörtgen::yeni(
        tuval.x + seri.sol.çöz(tuval.genişlik),
        tuval.y + seri.üst.çöz(tuval.yükseklik),
        seri.genişlik.çöz(tuval.genişlik),
        seri.yükseklik.çöz(tuval.yükseklik),
    );

    // Her boyutun ölçeği: veri kapsamı + seçenek sınırları.
    let ölçekler: Vec<AralıkÖlçeği> = seri
        .boyutlar
        .iter()
        .enumerate()
        .map(|(b, boyut)| {
            let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
            for öğe in &seri.veri {
                if let Some(değer) = öğe.değer.dizi().and_then(|d| d.get(b))
                    && değer.is_finite()
                {
                    kapsam[0] = kapsam[0].min(*değer);
                    kapsam[1] = kapsam[1].max(*değer);
                }
            }
            if !kapsam[0].is_finite() {
                kapsam = [0.0, 1.0];
            }
            AralıkÖlçeği::kur(kapsam, boyut.en_az, boyut.en_çok, false, 5, None, None)
        })
        .collect();

    let eksen_x =
        |b: usize| -> f32 { alan.x + (b as f32 / (boyut_sayısı - 1) as f32) * alan.genişlik };
    let değer_y = |b: usize, değer: f64| -> f32 {
        let oran = ölçekler.get(b).map(|ö| ö.oranla(değer)).unwrap_or(0.0) as f32;
        alan.alt() - oran * alan.yükseklik
    };

    // 1) Eksenler: çizgi + çentik etiketleri + boyut adı.
    for (b, boyut) in seri.boyutlar.iter().enumerate() {
        let x = keskin(eksen_x(b));
        çizici.çizgi(
            (x, alan.y),
            (x, alan.alt()),
            1.0,
            tema::eksen_çizgisi(),
            crate::model::stil::ÇizgiTürü::Düz,
        );
        if let Some(ölçek) = ölçekler.get(b) {
            for çentik in ölçek.çentikler() {
                let y = keskin(değer_y(b, çentik.değer));
                çizici.çizgi(
                    (x, y),
                    (x + 4.0, y),
                    1.0,
                    tema::eksen_çentiği(),
                    crate::model::stil::ÇizgiTürü::Düz,
                );
                çizici.yazı(
                    &ölçek.etiket(çentik.değer),
                    (x + 6.0, y),
                    YatayHiza::Sol,
                    DikeyHiza::Orta,
                    10.0,
                    tema::üçüncül_metin(),
                    false,
                );
            }
        }
        çizici.yazı(
            &boyut.ad,
            (x, alan.y - 8.0),
            YatayHiza::Orta,
            DikeyHiza::Alt,
            tema::YAZI_KÜÇÜK,
            tema::ikincil_metin(),
            false,
        );
    }

    // 2) Veri çizgileri.
    let opaklık = seri.çizgi_stili.opaklık * 0.6 * ilerleme.clamp(0.0, 1.0);
    let renk = seri.çizgi_stili.renk.unwrap_or(seri_rengi);
    for (j, öğe) in seri.veri.iter().enumerate() {
        let Some(dizi) = öğe.değer.dizi() else {
            continue;
        };
        let noktalar: Vec<(f32, f32)> = dizi
            .iter()
            .take(boyut_sayısı)
            .enumerate()
            .map(|(b, değer)| (eksen_x(b), değer_y(b, *değer)))
            .collect();
        if noktalar.len() < 2 {
            continue;
        }
        let mut yol = Yol::yeni();
        if let Some(&ilk) = noktalar.first() {
            yol.taşı(ilk);
        }
        for n in noktalar.iter().skip(1) {
            yol.çiz(*n);
        }
        çizici.yol_çiz(
            &yol,
            seri.çizgi_stili.kalınlık.max(1.0),
            renk.opaklık(opaklık),
            seri.çizgi_stili.tür,
        );
        // İsabet: ilk eksendeki nokta.
        if let Some(&ilk) = noktalar.first() {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: j,
                seri_adı: seri.ad.clone(),
                ad: öğe.ad.clone(),
                değer: dizi.first().copied(),
                geometri: İsabetGeometrisi::Daire {
                    merkez: ilk,
                    yarıçap: 8.0,
                },
            });
        }
    }
}
