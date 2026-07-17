//! Radar (örümcek ağı) koordinatı ve serisi — `echarts/src/coord/radar` ile
//! `chart/radar` karşılığı.

use std::collections::HashSet;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::sembol_çiz;
use crate::koordinat::Dikdörtgen;
use crate::model::radar::{RadarKoordinatı, RadarŞekli};
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::RadarSerisi;
use crate::renk::Dolgu;
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

/// Çözülmüş radar geometrisi.
pub struct RadarDüzeni {
    pub merkez: (f32, f32),
    pub yarıçap: f32,
    /// Her göstergenin ekran yön vektörü (birim).
    pub yönler: Vec<(f32, f32)>,
}

/// Radar koordinat geometrisini çözer. Gösterge 0 üsttedir; sıralar saat
/// yönünde ilerler.
pub fn radar_düzeni(koordinat: &RadarKoordinatı, tuval: Dikdörtgen) -> RadarDüzeni {
    let merkez = (
        tuval.x + koordinat.merkez.0.çöz(tuval.genişlik),
        tuval.y + koordinat.merkez.1.çöz(tuval.yükseklik),
    );
    let taban = tuval.genişlik.min(tuval.yükseklik) / 2.0;
    let yarıçap = koordinat.yarıçap.çöz(taban);
    let n = koordinat.göstergeler.len().max(1);
    let yönler = (0..n)
        .map(|i| {
            let açı = -std::f32::consts::FRAC_PI_2
                + i as f32 * std::f32::consts::TAU / n as f32;
            (açı.cos(), açı.sin())
        })
        .collect();
    RadarDüzeni { merkez, yarıçap, yönler }
}

/// Ağ (ızgara) çizimi: bölme halkaları, dönüşümlü bölme alanları, kollar ve
/// gösterge adları.
pub fn radar_ağı_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    koordinat: &RadarKoordinatı,
    düzen: &RadarDüzeni,
) {
    let bölme = koordinat.bölme_sayısı.max(1);

    let halka_yolu = |oran: f32| -> Yol {
        let yarıçap = düzen.yarıçap * oran;
        let mut yol = Yol::yeni();
        match koordinat.şekil {
            RadarŞekli::Çokgen => {
                for (i, (kos, sin)) in düzen.yönler.iter().enumerate() {
                    let nokta = (
                        düzen.merkez.0 + yarıçap * kos,
                        düzen.merkez.1 + yarıçap * sin,
                    );
                    if i == 0 {
                        yol.taşı(nokta);
                    } else {
                        yol.çiz(nokta);
                    }
                }
                yol.kapat();
            }
            RadarŞekli::Daire => {
                yol = crate::cizim::yuzey::daire_yolu(düzen.merkez, yarıçap);
            }
        }
        yol
    };

    // 1) Dönüşümlü bölme alanları (dıştan içe).
    if koordinat.bölme_alanı_göster {
        let bölme_renkleri = tema::bölme_alanı_renkleri();
        for s in (1..=bölme).rev() {
            let renk = bölme_renkleri
                .get(s % bölme_renkleri.len())
                .copied()
                .unwrap_or(tema::nötr_05());
            çizici.yol_doldur(&halka_yolu(s as f32 / bölme as f32), &Dolgu::Düz(renk));
        }
    }

    // 2) Bölme halkaları.
    for s in 1..=bölme {
        çizici.yol_çiz(
            &halka_yolu(s as f32 / bölme as f32),
            1.0,
            tema::bölme_çizgisi(),
            crate::model::stil::ÇizgiTürü::Düz,
        );
    }

    // 3) Kollar ve gösterge adları.
    for (i, (kos, sin)) in düzen.yönler.iter().enumerate() {
        let uç = (
            düzen.merkez.0 + düzen.yarıçap * kos,
            düzen.merkez.1 + düzen.yarıçap * sin,
        );
        çizici.çizgi(
            düzen.merkez,
            uç,
            1.0,
            tema::bölme_çizgisi(),
            crate::model::stil::ÇizgiTürü::Düz,
        );
        let Some(gösterge) = koordinat.göstergeler.get(i) else { continue };
        let etiket_konumu = (
            düzen.merkez.0 + (düzen.yarıçap + 12.0) * kos,
            düzen.merkez.1 + (düzen.yarıçap + 12.0) * sin,
        );
        let yatay = if kos.abs() < 0.3 {
            YatayHiza::Orta
        } else if *kos > 0.0 {
            YatayHiza::Sol
        } else {
            YatayHiza::Sağ
        };
        let dikey = if sin.abs() < 0.3 {
            DikeyHiza::Orta
        } else if *sin > 0.0 {
            DikeyHiza::Üst
        } else {
            DikeyHiza::Alt
        };
        çizici.yazı(
            &gösterge.ad,
            etiket_konumu,
            yatay,
            dikey,
            tema::YAZI_KÜÇÜK,
            tema::ikincil_metin(),
            false,
        );
    }
}

/// Radar serisini çizer: her veri öğesi bir çokgendir.
#[allow(clippy::too_many_arguments)]
pub fn radar_serisi_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &RadarSerisi,
    genel_sıra: usize,
    koordinat: &RadarKoordinatı,
    düzen: &RadarDüzeni,
    seçenekler: &GrafikSeçenekleri,
    kapalı: &HashSet<String>,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let ilerleme = ilerleme.clamp(0.0, 1.0);
    for (j, öğe) in seri.veri.iter().enumerate() {
        let ad = öğe.ad.clone().unwrap_or_else(|| format!("{j}"));
        if kapalı.contains(&ad) {
            continue;
        }
        let Some(değerler) = öğe.değer.dizi() else { continue };

        let renk = öğe
            .stil
            .as_ref()
            .and_then(|s| s.renk.as_ref())
            .map(|d| d.temsilî())
            .unwrap_or_else(|| seçenekler.palet_rengi(j));

        // Köşe noktaları.
        let noktalar: Vec<(f32, f32)> = düzen
            .yönler
            .iter()
            .enumerate()
            .map(|(i, (kos, sin))| {
                let gösterge = koordinat.göstergeler.get(i);
                let en_az = gösterge.map(|g| g.en_az).unwrap_or(0.0);
                let en_çok = gösterge.map(|g| g.en_çok).unwrap_or(100.0);
                let değer = değerler.get(i).copied().unwrap_or(en_az);
                let oran = if en_çok > en_az {
                    ((değer - en_az) / (en_çok - en_az)).clamp(0.0, 1.0) as f32
                } else {
                    0.0
                };
                let uzaklık = düzen.yarıçap * oran * ilerleme;
                (
                    düzen.merkez.0 + uzaklık * kos,
                    düzen.merkez.1 + uzaklık * sin,
                )
            })
            .collect();
        if noktalar.len() < 3 {
            continue;
        }

        let mut yol = Yol::yeni();
        for (i, nokta) in noktalar.iter().enumerate() {
            if i == 0 {
                yol.taşı(*nokta);
            } else {
                yol.çiz(*nokta);
            }
        }
        yol.kapat();

        if let Some(alan) = &seri.alan_stili {
            let dolgu = alan
                .renk
                .clone()
                .unwrap_or(Dolgu::Düz(renk))
                .opaklık(alan.opaklık);
            çizici.yol_doldur(&yol, &dolgu);
        }
        let çizgi_rengi = seri.çizgi_stili.renk.unwrap_or(renk);
        çizici.yol_çiz(
            &yol,
            seri.çizgi_stili.kalınlık,
            çizgi_rengi.opaklık(seri.çizgi_stili.opaklık),
            seri.çizgi_stili.tür,
        );
        if seri.sembol_göster {
            for nokta in &noktalar {
                sembol_çiz(çizici, seri.sembol, *nokta, seri.sembol_boyutu, renk);
            }
        }

        // İsabet: köşe sembolleri.
        for nokta in &noktalar {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: j,
                seri_adı: seri.ad.clone(),
                ad: Some(ad.clone()),
                değer: None,
                geometri: İsabetGeometrisi::Daire {
                    merkez: *nokta,
                    yarıçap: (seri.sembol_boyutu / 2.0 + 3.0).max(8.0),
                },
            });
        }
    }
}

/// Radar öğesinin ipucu satır metni: değerleri gösterge adlarıyla eşler.
pub fn radar_ipucu_satırları(
    seri: &RadarSerisi,
    koordinat: &RadarKoordinatı,
    veri_sırası: usize,
) -> Vec<(String, String)> {
    let Some(öğe) = seri.veri.get(veri_sırası) else { return Vec::new() };
    let Some(değerler) = öğe.değer.dizi() else { return Vec::new() };
    koordinat
        .göstergeler
        .iter()
        .zip(değerler.iter())
        .map(|(g, v)| (g.ad.clone(), binlik_ayır(*v)))
        .collect()
}
