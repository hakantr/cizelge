//! Grafo (graph) serisi — `echarts/src/chart/graph` karşılığı. Kuvvet
//! yerleşimi belirlenimcidir: düğümler çember üzerinde başlar, sabit
//! sayıda itme/çekme yinelemesiyle dengeye yaklaşır; aynı girdi her zaman
//! aynı görüntüyü üretir (altın testlerle uyumlu).

use std::collections::HashMap;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::seri::{GrafoSerisi, GrafoYerleşimi};
use crate::renk::{Dolgu, Renk};
use crate::tema;

/// Grafoyu çizer.
#[allow(clippy::too_many_arguments)]
pub fn grafo_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &GrafoSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    görünüm: (f32, f32, f32),
    kaymalar: &[(usize, f32, f32)],
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    if seri.düğümler.is_empty() {
        return;
    }
    let merkez = (
        tuval.x + seri.merkez.0.çöz(tuval.genişlik),
        tuval.y + seri.merkez.1.çöz(tuval.yükseklik),
    );
    let taban = tuval.genişlik.min(tuval.yükseklik) / 2.0;
    let yarıçap = seri.yarıçap.çöz(taban);

    let sıra_bul: HashMap<&str, usize> = seri
        .düğümler
        .iter()
        .enumerate()
        .map(|(i, d)| (d.ad.as_str(), i))
        .collect();
    let bağ_sıraları: Vec<(usize, usize)> = seri
        .bağlar
        .iter()
        .filter_map(|(k, h)| {
            let k = *sıra_bul.get(k.as_str())?;
            let h = *sıra_bul.get(h.as_str())?;
            (k != h).then_some((k, h))
        })
        .collect();

    // 1) Başlangıç: çember üzerinde eşit aralık (belirlenimci).
    let n = seri.düğümler.len();
    let mut konumlar: Vec<(f32, f32)> = (0..n)
        .map(|i| {
            let açı = -std::f32::consts::FRAC_PI_2
                + i as f32 * std::f32::consts::TAU / n as f32;
            (
                merkez.0 + yarıçap * 0.7 * açı.cos(),
                merkez.1 + yarıçap * 0.7 * açı.sin(),
            )
        })
        .collect();

    // 2) Kuvvet yerleşimi (Fruchterman–Reingold benzeri, sabit yineleme).
    if seri.yerleşim == GrafoYerleşimi::Kuvvet && n > 1 {
        let k = (yarıçap * yarıçap * std::f32::consts::PI / n as f32).sqrt().max(8.0);
        let yineleme = 120usize;
        for adım in 0..yineleme {
            let sıcaklık = yarıçap * 0.12 * (1.0 - adım as f32 / yineleme as f32) + 0.5;
            let mut kuvvetler = vec![(0.0f32, 0.0f32); n];

            // İtme (her çift).
            for i in 0..n {
                for j in (i + 1)..n {
                    let (Some(a), Some(b)) = (konumlar.get(i), konumlar.get(j)) else {
                        continue;
                    };
                    let dx = a.0 - b.0;
                    let dy = a.1 - b.1;
                    let uzaklık = (dx * dx + dy * dy).sqrt().max(0.5);
                    let itme = k * k / uzaklık * seri.itme;
                    let (ux, uy) = (dx / uzaklık, dy / uzaklık);
                    if let Some(f) = kuvvetler.get_mut(i) {
                        f.0 += ux * itme;
                        f.1 += uy * itme;
                    }
                    if let Some(f) = kuvvetler.get_mut(j) {
                        f.0 -= ux * itme;
                        f.1 -= uy * itme;
                    }
                }
            }
            // Çekme (bağlar).
            for (i, j) in &bağ_sıraları {
                let (Some(a), Some(b)) = (konumlar.get(*i), konumlar.get(*j)) else {
                    continue;
                };
                let dx = a.0 - b.0;
                let dy = a.1 - b.1;
                let uzaklık = (dx * dx + dy * dy).sqrt().max(0.5);
                let çekme = uzaklık * uzaklık / k / seri.kenar_uzunluğu.max(0.1);
                let (ux, uy) = (dx / uzaklık, dy / uzaklık);
                if let Some(f) = kuvvetler.get_mut(*i) {
                    f.0 -= ux * çekme;
                    f.1 -= uy * çekme;
                }
                if let Some(f) = kuvvetler.get_mut(*j) {
                    f.0 += ux * çekme;
                    f.1 += uy * çekme;
                }
            }
            // Merkeze hafif yerçekimi + uygulama.
            for (konum, kuvvet) in konumlar.iter_mut().zip(&kuvvetler) {
                let gx = (merkez.0 - konum.0) * 0.02;
                let gy = (merkez.1 - konum.1) * 0.02;
                let fx = kuvvet.0 + gx;
                let fy = kuvvet.1 + gy;
                let büyüklük = (fx * fx + fy * fy).sqrt().max(1e-6);
                let adım_boyu = büyüklük.min(sıcaklık);
                konum.0 += fx / büyüklük * adım_boyu;
                konum.1 += fy / büyüklük * adım_boyu;
            }
        }
    }

    // 2b) Gezinme (roam) dönüşümü: merkez odaklı ölçek + kaydırma; ardından
    // sürüklenen düğümlerin bireysel kaymaları.
    let (kayma_x, kayma_y, ölçek) = görünüm;
    let ölçek = if ölçek.is_finite() && ölçek > 0.01 { ölçek } else { 1.0 };
    if ölçek != 1.0 || kayma_x != 0.0 || kayma_y != 0.0 {
        for konum in konumlar.iter_mut() {
            konum.0 = merkez.0 + (konum.0 - merkez.0) * ölçek + kayma_x;
            konum.1 = merkez.1 + (konum.1 - merkez.1) * ölçek + kayma_y;
        }
    }
    for (sıra, dx, dy) in kaymalar {
        if let Some(konum) = konumlar.get_mut(*sıra) {
            konum.0 += dx;
            konum.1 += dy;
        }
    }

    // 3) Bağlar.
    let opaklık = ilerleme.clamp(0.0, 1.0);
    for (i, j) in &bağ_sıraları {
        let (Some(a), Some(b)) = (konumlar.get(*i), konumlar.get(*j)) else { continue };
        çizici.çizgi(
            *a,
            *b,
            1.0,
            tema::nötr_30().opaklık(opaklık),
            crate::model::stil::ÇizgiTürü::Düz,
        );
    }

    // 4) Düğümler + etiketler.
    for (i, düğüm) in seri.düğümler.iter().enumerate() {
        let Some(&konum) = konumlar.get(i) else { continue };
        let boyut = düğüm.boyut.max(4.0) * opaklık.max(0.01) * ölçek;
        let renk = palet(düğüm.kategori.unwrap_or(i));
        çizici.daire(
            konum,
            boyut / 2.0,
            Some(&Dolgu::Düz(renk)),
            Some((1.5, Renk::BEYAZ)),
        );
        if düğüm.boyut >= seri.etiket_eşiği {
            çizici.yazı(
                &düğüm.ad,
                (konum.0, konum.1 + boyut / 2.0 + 3.0),
                YatayHiza::Orta,
                DikeyHiza::Üst,
                tema::YAZI_KÜÇÜK,
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
                yarıçap: (boyut / 2.0 + 3.0).max(8.0),
            },
        });
    }
}
