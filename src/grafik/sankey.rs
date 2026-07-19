//! Sankey serisi — `echarts/src/chart/sankey` karşılığı: düğümler
//! kaynaklardan uzaklığa göre katmanlanır, bağlar değerle orantılı
//! kalınlıkta kübik şeritlerle çizilir.

use std::collections::HashMap;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::seri::{SankeyBağı, SankeySerisi};
use crate::renk::{Dolgu, Renk};
use crate::tema;

struct SankeyDüğümü {
    ad: String,
    katman: usize,
    değer: f64,
    alan: Dikdörtgen,
    renk: Renk,
    /// Giden/gelen bağ istif kaydırmaları (piksel).
    giden_kaydırma: f32,
    gelen_kaydırma: f32,
}

/// Sankey'i çizer; döngü oluşturan bağlar atlanır.
#[allow(clippy::too_many_arguments)]
pub fn sankey_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &SankeySerisi,
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

    // 1) Düğüm listesi: açık liste + bağlardan türetilenler.
    let mut adlar: Vec<String> = seri.düğümler.clone();
    for bağ in &seri.bağlar {
        for ad in [&bağ.kaynak, &bağ.hedef] {
            if !adlar.iter().any(|a| a == ad) {
                adlar.push(ad.clone());
            }
        }
    }
    if adlar.is_empty() {
        return;
    }
    let sıra_bul: HashMap<&str, usize> = adlar
        .iter()
        .enumerate()
        .map(|(i, a)| (a.as_str(), i))
        .collect();

    // 2) Katman ataması: kaynaklardan en uzun yol (döngüler atlanır).
    let mut katmanlar = vec![0usize; adlar.len()];
    let geçerli_bağlar: Vec<&SankeyBağı> = seri
        .bağlar
        .iter()
        .filter(|b| b.değer.is_finite() && b.değer > 0.0 && b.kaynak != b.hedef)
        .collect();
    for _ in 0..adlar.len() {
        let mut değişti = false;
        for bağ in &geçerli_bağlar {
            let (Some(&k), Some(&h)) = (
                sıra_bul.get(bağ.kaynak.as_str()),
                sıra_bul.get(bağ.hedef.as_str()),
            ) else {
                continue;
            };
            let hedef_katman = katmanlar.get(k).copied().unwrap_or(0) + 1;
            if katmanlar.get(h).copied().unwrap_or(0) < hedef_katman
                && hedef_katman <= adlar.len()
                && let Some(kayıt) = katmanlar.get_mut(h)
            {
                *kayıt = hedef_katman;
                değişti = true;
            }
        }
        if !değişti {
            break;
        }
    }
    let katman_sayısı = katmanlar.iter().copied().max().unwrap_or(0) + 1;

    // 3) Düğüm değerleri: max(gelen, giden).
    let mut giden = vec![0.0f64; adlar.len()];
    let mut gelen = vec![0.0f64; adlar.len()];
    for bağ in &geçerli_bağlar {
        if let Some(&k) = sıra_bul.get(bağ.kaynak.as_str())
            && let Some(kayıt) = giden.get_mut(k)
        {
            *kayıt += bağ.değer;
        }
        if let Some(&h) = sıra_bul.get(bağ.hedef.as_str())
            && let Some(kayıt) = gelen.get_mut(h)
        {
            *kayıt += bağ.değer;
        }
    }

    // 4) Katman içi yerleşim: yükseklik değerle orantılı.
    let mut düğümler: Vec<SankeyDüğümü> = Vec::with_capacity(adlar.len());
    let katman_x = |katman: usize| -> f32 {
        if katman_sayısı <= 1 {
            alan.x
        } else {
            alan.x
                + (katman as f32 / (katman_sayısı - 1) as f32)
                    * (alan.genişlik - seri.düğüm_genişliği)
        }
    };
    // Ölçek: en dolu katmanın toplamına göre.
    let mut katman_toplamları = vec![0.0f64; katman_sayısı];
    let mut katman_sayaçları = vec![0usize; katman_sayısı];
    for (i, _) in adlar.iter().enumerate() {
        let değer = giden
            .get(i)
            .copied()
            .unwrap_or(0.0)
            .max(gelen.get(i).copied().unwrap_or(0.0))
            .max(1e-9);
        let katman = katmanlar.get(i).copied().unwrap_or(0);
        if let Some(kayıt) = katman_toplamları.get_mut(katman) {
            *kayıt += değer;
        }
        if let Some(kayıt) = katman_sayaçları.get_mut(katman) {
            *kayıt += 1;
        }
    }
    let ölçek = (0..katman_sayısı)
        .map(|k| {
            let boşluklar =
                seri.düğüm_boşluğu * katman_sayaçları.get(k).copied().unwrap_or(1) as f32;
            let kullanılabilir = (alan.yükseklik - boşluklar).max(10.0) as f64;
            kullanılabilir / katman_toplamları.get(k).copied().unwrap_or(1.0).max(1e-9)
        })
        .fold(f64::INFINITY, f64::min);

    let mut katman_y = vec![alan.y; katman_sayısı];
    for (i, ad) in adlar.iter().enumerate() {
        let katman = katmanlar.get(i).copied().unwrap_or(0);
        let değer = giden
            .get(i)
            .copied()
            .unwrap_or(0.0)
            .max(gelen.get(i).copied().unwrap_or(0.0))
            .max(1e-9);
        let yükseklik = ((değer * ölçek) as f32).max(3.0);
        let y = katman_y.get(katman).copied().unwrap_or(alan.y);
        if let Some(kayıt) = katman_y.get_mut(katman) {
            *kayıt = y + yükseklik + seri.düğüm_boşluğu;
        }
        düğümler.push(SankeyDüğümü {
            ad: ad.clone(),
            katman,
            değer,
            alan: Dikdörtgen::yeni(katman_x(katman), y, seri.düğüm_genişliği, yükseklik),
            renk: palet(i),
            giden_kaydırma: 0.0,
            gelen_kaydırma: 0.0,
        });
    }

    // 5) Bağ şeritleri.
    let opaklık = 0.35 * ilerleme.clamp(0.0, 1.0);
    for bağ in &geçerli_bağlar {
        let (Some(&k), Some(&h)) = (
            sıra_bul.get(bağ.kaynak.as_str()),
            sıra_bul.get(bağ.hedef.as_str()),
        ) else {
            continue;
        };
        let kalınlık = ((bağ.değer * ölçek) as f32).max(1.5);
        let (k_alan, k_renk, k_kaydırma) = match düğümler.get(k) {
            Some(d) => (d.alan, d.renk, d.giden_kaydırma),
            None => continue,
        };
        let h_kaydırma = match düğümler.get(h) {
            Some(d) => d.gelen_kaydırma,
            None => continue,
        };
        let h_alan = match düğümler.get(h) {
            Some(d) => d.alan,
            None => continue,
        };

        let y0 = k_alan.y + k_kaydırma;
        let y1 = h_alan.y + h_kaydırma;
        let x0 = k_alan.sağ();
        let x1 = h_alan.x;
        let orta = (x0 + x1) / 2.0;

        let mut şerit = Yol::yeni();
        şerit.taşı((x0, y0));
        şerit.kübik((orta, y0), (orta, y1), (x1, y1));
        şerit.çiz((x1, y1 + kalınlık));
        şerit.kübik(
            (orta, y1 + kalınlık),
            (orta, y0 + kalınlık),
            (x0, y0 + kalınlık),
        );
        şerit.kapat();
        çizici.yol_doldur(&şerit, &Dolgu::Düz(k_renk.opaklık(opaklık)));

        if let Some(d) = düğümler.get_mut(k) {
            d.giden_kaydırma += kalınlık;
        }
        if let Some(d) = düğümler.get_mut(h) {
            d.gelen_kaydırma += kalınlık;
        }
    }

    // 6) Düğümler + etiketler.
    for (i, düğüm) in düğümler.iter().enumerate() {
        çizici.dikdörtgen(düğüm.alan, &Dolgu::Düz(düğüm.renk), [2.0; 4], None);
        let sağda = düğüm.katman + 1 < katman_sayısı;
        let (x, hiza) = if sağda {
            (düğüm.alan.sağ() + 6.0, YatayHiza::Sol)
        } else {
            (düğüm.alan.x - 6.0, YatayHiza::Sağ)
        };
        çizici.yazı(
            &düğüm.ad,
            (x, düğüm.alan.y + düğüm.alan.yükseklik / 2.0),
            hiza,
            DikeyHiza::Orta,
            tema::YAZI_KÜÇÜK,
            tema::birincil_metin(),
            false,
        );
        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: i,
            seri_adı: seri.ad.clone(),
            ad: Some(düğüm.ad.clone()),
            değer: Some(düğüm.değer),
            geometri: İsabetGeometrisi::Dikdörtgen(düğüm.alan),
        });
    }
}
