//! Tema nehri (themeRiver) — `echarts/src/chart/themeRiver` ve tek eksen
//! (`coord/single`) karşılığı: katmanlar, siluet taban çizgisi etrafında
//! yığılmış yumuşak bantlar olarak akar; altta tek değer ekseni çizilir.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{keskin, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::cizgi::yumuşak_parça_ekle;
use crate::koordinat::Dikdörtgen;
use crate::model::seri::TemaNehriSerisi;
use crate::olcek::AralıkÖlçeği;
use crate::renk::{Dolgu, Renk};
use crate::tema;

/// Tema nehrini çizer.
#[allow(clippy::too_many_arguments)]
pub fn tema_nehri_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &TemaNehriSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let alan = Dikdörtgen::yeni(
        tuval.x + seri.sol.çöz(tuval.genişlik),
        tuval.y + seri.üst.çöz(tuval.yükseklik),
        seri.genişlik.çöz(tuval.genişlik),
        seri.yükseklik.çöz(tuval.yükseklik),
    );

    // Katman adları (ilk görülme sırasıyla) ve x değerleri.
    let mut katmanlar: Vec<String> = Vec::new();
    let mut x_değerleri: Vec<f64> = Vec::new();
    for (x, _, katman) in &seri.veri {
        if !katmanlar.iter().any(|k| k == katman) {
            katmanlar.push(katman.clone());
        }
        if !x_değerleri.iter().any(|v| (v - x).abs() < 1e-9) {
            x_değerleri.push(*x);
        }
    }
    x_değerleri.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if katmanlar.is_empty() || x_değerleri.len() < 2 {
        return;
    }

    // Değer tablosu: katman × x.
    let değer_bul = |katman: &str, x: f64| -> f64 {
        seri.veri
            .iter()
            .find(|(vx, _, vk)| (vx - x).abs() < 1e-9 && vk == katman)
            .map(|(_, d, _)| *d)
            .unwrap_or(0.0)
            .max(0.0)
    };

    // Her x'te kümülatif yığın ve siluet taban çizgisi (-toplam/2).
    let toplamlar: Vec<f64> = x_değerleri
        .iter()
        .map(|x| katmanlar.iter().map(|k| değer_bul(k, *x)).sum())
        .collect();
    let en_büyük_toplam = toplamlar.iter().copied().fold(0.0, f64::max).max(1e-9);

    // Tek eksen ölçeği (x).
    let x_kapsamı = [
        x_değerleri.first().copied().unwrap_or(0.0),
        x_değerleri.last().copied().unwrap_or(1.0),
    ];
    let x_ölçeği = AralıkÖlçeği::kur(x_kapsamı, None, None, false, 6, None, None);

    let x_piksel = |x: f64| -> f32 {
        let oran = if x_kapsamı[1] > x_kapsamı[0] {
            (x - x_kapsamı[0]) / (x_kapsamı[1] - x_kapsamı[0])
        } else {
            0.0
        };
        alan.x + (oran as f32) * alan.genişlik
    };
    let orta_y = alan.y + alan.yükseklik / 2.0;
    let değer_ölçeği =
        (alan.yükseklik * 0.9) as f64 / en_büyük_toplam * ilerleme.clamp(0.0, 1.0) as f64;

    // Katman bantları: alt sınır = önceki katmanların toplamı - toplam/2.
    let opaklık = 0.85;
    for (k_sıra, katman) in katmanlar.iter().enumerate() {
        let renk = palet(k_sıra);
        let mut üst_eğri: Vec<(f32, f32)> = Vec::with_capacity(x_değerleri.len());
        let mut alt_eğri: Vec<(f32, f32)> = Vec::with_capacity(x_değerleri.len());
        let mut en_geniş: Option<(f32, f32, f64)> = None;

        for (xi, x) in x_değerleri.iter().enumerate() {
            let toplam = toplamlar.get(xi).copied().unwrap_or(0.0);
            let öncekiler: f64 = katmanlar
                .iter()
                .take(k_sıra)
                .map(|k| değer_bul(k, *x))
                .sum();
            let değer = değer_bul(katman, *x);
            let taban = -toplam / 2.0 + öncekiler;
            let px = x_piksel(*x);
            let alt_y = orta_y + (taban * değer_ölçeği) as f32;
            let üst_y = orta_y + ((taban + değer) * değer_ölçeği) as f32;
            alt_eğri.push((px, alt_y));
            üst_eğri.push((px, üst_y));
            if en_geniş.map(|(_, _, d)| değer > d).unwrap_or(değer > 0.0) {
                en_geniş = Some((px, (alt_y + üst_y) / 2.0, değer));
            }
        }

        // Bant: yumuşak üst eğri + ters yumuşak alt eğri.
        let mut yol = Yol::yeni();
        yumuşak_parça_ekle(&mut yol, &üst_eğri, 0.5, true);
        let mut ters_alt = alt_eğri.clone();
        ters_alt.reverse();
        yumuşak_parça_ekle(&mut yol, &ters_alt, 0.5, false);
        yol.kapat();
        çizici.yol_doldur(&yol, &Dolgu::Düz(renk.opaklık(opaklık)));

        // Katman adı en geniş noktada.
        if let Some((ex, ey, _)) = en_geniş {
            çizici.yazı(
                katman,
                (ex, ey),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                tema::yazı_küçük(),
                Renk::BEYAZ,
                false,
            );
        }

        // İsabetler: her x noktasında bant dilimi.
        for (xi, (üst, alt)) in üst_eğri.iter().zip(alt_eğri.iter()).enumerate() {
            let x = x_değerleri.get(xi).copied().unwrap_or(0.0);
            let değer = değer_bul(katman, x);
            if değer <= 0.0 {
                continue;
            }
            let yarı = (x_piksel(x_kapsamı[1]) - x_piksel(x_kapsamı[0]))
                / (x_değerleri.len().max(2) as f32 - 1.0)
                / 2.0;
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: k_sıra * x_değerleri.len() + xi,
                seri_adı: seri.ad.clone(),
                ad: Some(katman.clone()),
                değer: Some(değer),
                geometri: İsabetGeometrisi::Dikdörtgen(Dikdörtgen::yeni(
                    üst.0 - yarı,
                    üst.1.min(alt.1),
                    yarı * 2.0,
                    (alt.1 - üst.1).abs().max(1.0),
                )),
            });
        }
    }

    // Tek eksen (altta): çizgi + çentikler + etiketler.
    let eksen_y = keskin(alan.alt() + 12.0);
    çizici.çizgi(
        (alan.x, eksen_y),
        (alan.sağ(), eksen_y),
        1.0,
        tema::eksen_çizgisi(),
        crate::model::stil::ÇizgiTürü::Düz,
    );
    for çentik in x_ölçeği.çentikler() {
        let px = keskin(x_piksel(çentik.değer));
        if px < alan.x - 0.6 || px > alan.sağ() + 0.6 {
            continue;
        }
        çizici.çizgi(
            (px, eksen_y),
            (px, eksen_y + 5.0),
            1.0,
            tema::eksen_çentiği(),
            crate::model::stil::ÇizgiTürü::Düz,
        );
        çizici.yazı(
            &x_ölçeği.etiket(çentik.değer),
            (px, eksen_y + 8.0),
            YatayHiza::Orta,
            DikeyHiza::Üst,
            tema::yazı_küçük(),
            tema::eksen_etiketi(),
            false,
        );
    }
}
