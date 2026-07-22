//! Grafo (graph) serisi — `echarts/src/chart/graph` karşılığı. Kuvvet
//! yerleşimi belirlenimcidir: düğümler çember üzerinde başlar, sabit
//! sayıda itme/çekme yinelemesiyle dengeye yaklaşır; aynı girdi her zaman
//! aynı görüntüyü üretir (altın testlerle uyumlu).

use std::collections::HashMap;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, MatrisYerleşimi, TakvimYerleşimi};
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
    takvim: Option<&TakvimYerleşimi>,
    matris: Option<&MatrisYerleşimi>,
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

    // 1) Takvim koordinatında ilk veri boyutu doğrudan tarih hücresine;
    // görünüm koordinatında düğümler belirlenimci çembere yerleşir.
    let n = seri.düğümler.len();
    let mut konumlar: Vec<(f32, f32)> = if let Some(takvim) = takvim {
        seri.düğümler
            .iter()
            .map(|düğüm| {
                düğüm
                    .takvim_tarihi_ms
                    .and_then(|tarih| takvim.veriden_noktaya(tarih))
                    .unwrap_or((f32::NAN, f32::NAN))
            })
            .collect()
    } else if let Some(matris) = matris {
        seri.düğümler
            .iter()
            .map(|düğüm| {
                düğüm
                    .matris_koordinatı
                    .as_ref()
                    .and_then(|(x, y)| matris.veriden_noktaya(x.clone(), y.clone()))
                    .unwrap_or((f32::NAN, f32::NAN))
            })
            .collect()
    } else {
        (0..n)
            .map(|i| {
                let açı =
                    -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / n as f32;
                (
                    merkez.0 + yarıçap * 0.7 * açı.cos(),
                    merkez.1 + yarıçap * 0.7 * açı.sin(),
                )
            })
            .collect()
    };

    // 2) Kuvvet yerleşimi (Fruchterman–Reingold benzeri, sabit yineleme).
    if takvim.is_none() && matris.is_none() && seri.yerleşim == GrafoYerleşimi::Kuvvet && n > 1 {
        let k = (yarıçap * yarıçap * std::f32::consts::PI / n as f32)
            .sqrt()
            .max(8.0);
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
    let ölçek = if ölçek.is_finite() && ölçek > 0.01 {
        ölçek
    } else {
        1.0
    };
    if takvim.is_none() && matris.is_none() && (ölçek != 1.0 || kayma_x != 0.0 || kayma_y != 0.0)
    {
        for konum in konumlar.iter_mut() {
            konum.0 = merkez.0 + (konum.0 - merkez.0) * ölçek + kayma_x;
            konum.1 = merkez.1 + (konum.1 - merkez.1) * ölçek + kayma_y;
        }
    }
    if takvim.is_none() && matris.is_none() {
        for (sıra, dx, dy) in kaymalar {
            if let Some(konum) = konumlar.get_mut(*sıra) {
                konum.0 += dx;
                konum.1 += dy;
            }
        }
    }

    // 3) Bağlar.
    let opaklık = ilerleme.clamp(0.0, 1.0);
    let çizgi_rengi = seri.çizgi_stili.renk.unwrap_or_else(tema::nötr_50);
    let çizgi_opaklığı = opaklık * seri.çizgi_stili.opaklık.clamp(0.0, 1.0);
    for (i, j) in &bağ_sıraları {
        let (Some(a), Some(b)) = (konumlar.get(*i), konumlar.get(*j)) else {
            continue;
        };
        if !a.0.is_finite() || !a.1.is_finite() || !b.0.is_finite() || !b.1.is_finite() {
            continue;
        }
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let uzunluk = (dx * dx + dy * dy).sqrt();
        if uzunluk <= f32::EPSILON {
            continue;
        }
        let birim = (dx / uzunluk, dy / uzunluk);
        let hedef_yarıçapı = seri
            .düğümler
            .get(*j)
            .map(|düğüm| düğüm.boyut.max(0.0) * ölçek / 2.0)
            .unwrap_or_default();
        let bitiş = if seri.hedef_oku {
            (
                b.0 - birim.0 * hedef_yarıçapı,
                b.1 - birim.1 * hedef_yarıçapı,
            )
        } else {
            *b
        };
        çizici.çizgi(
            *a,
            bitiş,
            seri.çizgi_stili.kalınlık,
            çizgi_rengi.opaklık(çizgi_opaklığı),
            seri.çizgi_stili.tür,
        );
        if seri.hedef_oku && seri.hedef_oku_boyutu > 0.0 {
            let boyut = seri.hedef_oku_boyutu;
            let dik = (-birim.1, birim.0);
            let taban = (bitiş.0 - birim.0 * boyut, bitiş.1 - birim.1 * boyut);
            let çentik = (
                bitiş.0 - birim.0 * boyut * 0.75,
                bitiş.1 - birim.1 * boyut * 0.75,
            );
            let mut ok = Yol::yeni();
            ok.taşı(bitiş);
            ok.çiz((
                taban.0 + dik.0 * boyut * 2.0 / 3.0,
                taban.1 + dik.1 * boyut * 2.0 / 3.0,
            ));
            ok.çiz(çentik);
            ok.çiz((
                taban.0 - dik.0 * boyut * 2.0 / 3.0,
                taban.1 - dik.1 * boyut * 2.0 / 3.0,
            ));
            ok.kapat();
            çizici.yol_doldur(&ok, &Dolgu::Düz(çizgi_rengi.opaklık(çizgi_opaklığı)));
        }
    }

    // 4) Düğümler + etiketler.
    for (i, düğüm) in seri.düğümler.iter().enumerate() {
        let Some(&konum) = konumlar.get(i) else {
            continue;
        };
        if !konum.0.is_finite() || !konum.1.is_finite() {
            continue;
        }
        let boyut = düğüm.boyut.max(4.0) * opaklık.max(0.01) * ölçek;
        // ECharts'ta kategori atanmamış bütün graph düğümleri serinin
        // palet rengini paylaşır; düğüm sırası yalnız açık kategoriyi seçer.
        let palet_rengi = palet(düğüm.kategori.unwrap_or(genel_sıra));
        let dolgu = seri
            .öğe_stili
            .renk
            .clone()
            .unwrap_or(Dolgu::Düz(palet_rengi))
            .opaklık(seri.öğe_stili.opaklık.unwrap_or(1.0));
        if let Some(gölge_rengi) = seri.öğe_stili.gölge_rengi
            && (seri.öğe_stili.gölge_bulanıklığı > 0.0
                || seri.öğe_stili.gölge_kayması != (0.0, 0.0))
        {
            let yarıçap = boyut / 2.0;
            let mut yol = Yol::yeni();
            yol.taşı((konum.0 + yarıçap, konum.1));
            yol.yay(yarıçap, false, true, (konum.0 - yarıçap, konum.1));
            yol.yay(yarıçap, false, true, (konum.0 + yarıçap, konum.1));
            yol.kapat();
            çizici.yol_gölgesi(
                &yol,
                gölge_rengi.opaklık(opaklık),
                seri.öğe_stili.gölge_bulanıklığı,
                seri.öğe_stili.gölge_kayması,
            );
        }
        let kenarlık = (seri.öğe_stili.kenarlık_kalınlığı > 0.0).then(|| {
            (
                seri.öğe_stili.kenarlık_kalınlığı,
                seri.öğe_stili.kenarlık_rengi.unwrap_or(palet_rengi),
            )
        });
        çizici.daire(konum, boyut / 2.0, Some(&dolgu), kenarlık);
        if seri.etiket_göster && düğüm.boyut >= seri.etiket_eşiği {
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

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::model::seri::GrafoDüğümü;
    use crate::model::takvim::TakvimKoordinatı;
    use crate::yardimci::takvim::{TakvimAnı, takvimden_ana};

    #[test]
    fn takvime_bağlı_düğüm_tarih_hücresinin_merkezine_yerleşir() {
        let tarih = takvimden_ana(TakvimAnı {
            yıl: 2017,
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let seri = GrafoSerisi::yeni()
            .takvim_sırası(0)
            .düğümler([GrafoDüğümü::yeni("2017-01-01", 15.0)
                .değerli(260.0)
                .takvim_tarihi(tarih)]);
        let yerleşim = TakvimYerleşimi::kur(&TakvimKoordinatı::yıl(2017), (700.0, 525.0))
            .expect("takvim yerleşimi kurulmalı");
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let mut isabetler = Vec::new();

        grafo_çiz(
            &mut yüzey,
            &seri,
            0,
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            &|_| Renk::SİYAH,
            1.0,
            (0.0, 0.0, 1.0),
            &[],
            Some(&yerleşim),
            None,
            &mut isabetler,
        );

        assert_eq!(isabetler.len(), 1);
        assert!(matches!(
            isabetler[0].geometri,
            İsabetGeometrisi::Daire {
                merkez: (90.0, 70.0),
                ..
            }
        ));
    }
}
