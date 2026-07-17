//! Güneş patlaması (sunburst) — `echarts/src/chart/sunburst` karşılığı:
//! hiyerarşi, iç içe halkalarda açısal paylara bölünür.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::ÇizimYüzeyi;
use crate::koordinat::Dikdörtgen;
use crate::model::agac::AğaçDüğümü;
use crate::model::seri::GüneşPatlamasıSerisi;
use crate::renk::{Dolgu, Renk};

/// Ağaç derinliği (kök çocukları = 1).
fn derinlik(düğümler: &[AğaçDüğümü]) -> usize {
    düğümler
        .iter()
        .map(|d| 1 + derinlik(&d.çocuklar))
        .max()
        .unwrap_or(0)
}

#[allow(clippy::too_many_arguments)]
fn dilimleri_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    düğümler: &[AğaçDüğümü],
    merkez: (f32, f32),
    açı0: f32,
    açı1: f32,
    seviye: usize,
    halka: f32,
    iç_taban: f32,
    üst_renk: Option<Renk>,
    palet: &dyn Fn(usize) -> Renk,
    genel_sıra: usize,
    seri_adı: &Option<String>,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let toplam: f64 = düğümler.iter().map(|d| d.etkin_değer()).sum();
    if toplam <= 0.0 {
        return;
    }
    let mut açı = açı0;
    for (i, düğüm) in düğümler.iter().enumerate() {
        let değer = düğüm.etkin_değer();
        if değer <= 0.0 {
            continue;
        }
        let pay = ((değer / toplam) * (açı1 - açı0) as f64) as f32;
        let renk = düğüm.renk.unwrap_or_else(|| match üst_renk {
            Some(üst) => üst.karıştır(Renk::BEYAZ, 0.18 * seviye as f32),
            None => palet(i),
        });
        let iç = iç_taban + (seviye as f32) * halka;
        let dış = iç + halka - 1.0;
        çizici.dilim(
            merkez,
            iç,
            dış,
            açı,
            açı + pay,
            &Dolgu::Düz(renk),
            Some((1.0, Renk::BEYAZ)),
        );
        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: isabetler.len(),
            seri_adı: seri_adı.clone(),
            ad: Some(düğüm.ad.clone()),
            değer: Some(değer),
            geometri: İsabetGeometrisi::Halka {
                merkez,
                iç_yarıçap: iç,
                dış_yarıçap: dış,
                açı0: açı,
                açı1: açı + pay,
            },
        });
        if !düğüm.çocuklar.is_empty() {
            dilimleri_çiz(
                çizici,
                &düğüm.çocuklar,
                merkez,
                açı,
                açı + pay,
                seviye + 1,
                halka,
                iç_taban,
                Some(renk),
                palet,
                genel_sıra,
                seri_adı,
                isabetler,
            );
        }
        açı += pay;
    }
}

/// Güneş patlamasını çizer.
pub fn güneş_patlaması_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &GüneşPatlamasıSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let merkez = (
        tuval.x + seri.merkez.0.çöz(tuval.genişlik),
        tuval.y + seri.merkez.1.çöz(tuval.yükseklik),
    );
    let taban = tuval.genişlik.min(tuval.yükseklik) / 2.0;
    let dış = seri.yarıçap.1.çöz(taban);
    let iç = seri.yarıçap.0.çöz(taban);
    let seviye_sayısı = derinlik(&seri.kökler).max(1);
    let halka = ((dış - iç) / seviye_sayısı as f32).max(2.0);

    let açı0 = -std::f32::consts::FRAC_PI_2;
    let açı1 = açı0 + std::f32::consts::TAU * ilerleme.clamp(0.0, 1.0);
    dilimleri_çiz(
        çizici,
        &seri.kökler,
        merkez,
        açı0,
        açı1,
        0,
        halka,
        iç,
        None,
        palet,
        genel_sıra,
        &seri.ad,
        isabetler,
    );
}
