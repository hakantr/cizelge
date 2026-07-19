//! Kiriş (chord) serisi — `echarts/src/chart/chord` karşılığı: düğümler
//! çember üzerinde yay dilimleri, akışlar merkezden geçen kübik şeritlerdir.

use std::collections::HashMap;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::seri::KirişSerisi;
use crate::renk::{Dolgu, Renk};
use crate::tema;

/// Kirişi çizer.
#[allow(clippy::too_many_arguments)]
pub fn kiriş_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &KirişSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    // Düğüm listesi bağlardan türetilir.
    let mut adlar: Vec<String> = Vec::new();
    for (k, h, _) in &seri.bağlar {
        for ad in [k, h] {
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

    // Düğüm toplamları (giden + gelen).
    let mut toplamlar = vec![0.0f64; adlar.len()];
    for (k, h, değer) in &seri.bağlar {
        if !değer.is_finite() || *değer <= 0.0 {
            continue;
        }
        for ad in [k, h] {
            if let Some(&i) = sıra_bul.get(ad.as_str())
                && let Some(kayıt) = toplamlar.get_mut(i)
            {
                *kayıt += değer;
            }
        }
    }
    let genel_toplam: f64 = toplamlar.iter().sum();
    if genel_toplam <= 0.0 {
        return;
    }

    let merkez = (
        tuval.x + seri.merkez.0.çöz(tuval.genişlik),
        tuval.y + seri.merkez.1.çöz(tuval.yükseklik),
    );
    let taban = tuval.genişlik.min(tuval.yükseklik) / 2.0;
    let dış = seri.yarıçap.çöz(taban);
    let iç = dış - seri.şerit_kalınlığı.max(4.0);

    // Yay dilimleri: aralarda boşluk payı.
    let boşluk = 0.02f64 * std::f64::consts::TAU;
    let kullanılabilir =
        std::f64::consts::TAU * ilerleme.clamp(0.0, 1.0) as f64 - boşluk * adlar.len() as f64;
    if kullanılabilir <= 0.0 {
        return;
    }
    let mut açılar: Vec<(f32, f32)> = Vec::with_capacity(adlar.len());
    let mut açı = -std::f64::consts::FRAC_PI_2;
    for toplam in &toplamlar {
        let pay = kullanılabilir * (toplam / genel_toplam);
        açılar.push((açı as f32, (açı + pay) as f32));
        açı += pay + boşluk;
    }

    // Her düğümün bağ istif ilerlemesi (radyan).
    let mut istif: Vec<f32> = açılar.iter().map(|(a0, _)| *a0).collect();

    // 1) Akış şeritleri.
    for (k, h, değer) in &seri.bağlar {
        if !değer.is_finite() || *değer <= 0.0 {
            continue;
        }
        let (Some(&ki), Some(&hi)) = (sıra_bul.get(k.as_str()), sıra_bul.get(h.as_str())) else {
            continue;
        };
        let pay = (kullanılabilir * (değer / genel_toplam)) as f32;
        let (Some(&k_başı), Some(&h_başı)) = (istif.get(ki), istif.get(hi)) else {
            continue;
        };
        let (k0, k1) = (k_başı, k_başı + pay);
        let (h0, h1) = (h_başı, h_başı + pay);
        if let Some(kayıt) = istif.get_mut(ki) {
            *kayıt = k1;
        }
        if ki != hi
            && let Some(kayıt) = istif.get_mut(hi)
        {
            *kayıt = h1;
        }

        let uç = |açı: f32| (merkez.0 + iç * açı.cos(), merkez.1 + iç * açı.sin());
        let renk = palet(ki).opaklık(0.4 * ilerleme.clamp(0.0, 1.0));
        let mut şerit = Yol::yeni();
        şerit.taşı(uç(k0));
        şerit.yay(iç, false, true, uç(k1));
        şerit.kübik(merkez, merkez, uç(h0));
        şerit.yay(iç, false, true, uç(h1));
        şerit.kübik(merkez, merkez, uç(k0));
        şerit.kapat();
        çizici.yol_doldur(&şerit, &Dolgu::Düz(renk));
    }

    // 2) Yay dilimleri + etiketler.
    for (i, ad) in adlar.iter().enumerate() {
        let Some(&(a0, a1)) = açılar.get(i) else {
            continue;
        };
        let renk = palet(i);
        çizici.dilim(merkez, iç, dış, a0, a1, &Dolgu::Düz(renk), None);
        let orta = (a0 + a1) / 2.0;
        let konum = (
            merkez.0 + (dış + 10.0) * orta.cos(),
            merkez.1 + (dış + 10.0) * orta.sin(),
        );
        let yatay = if orta.cos().abs() < 0.3 {
            YatayHiza::Orta
        } else if orta.cos() > 0.0 {
            YatayHiza::Sol
        } else {
            YatayHiza::Sağ
        };
        çizici.yazı(
            ad,
            konum,
            yatay,
            DikeyHiza::Orta,
            tema::YAZI_KÜÇÜK,
            tema::ikincil_metin(),
            false,
        );
        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: i,
            seri_adı: seri.ad.clone(),
            ad: Some(ad.clone()),
            değer: toplamlar.get(i).copied(),
            geometri: İsabetGeometrisi::Halka {
                merkez,
                iç_yarıçap: iç,
                dış_yarıçap: dış,
                açı0: a0,
                açı1: a1,
            },
        });
    }
}
