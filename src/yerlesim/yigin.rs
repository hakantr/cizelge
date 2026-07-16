//! Yığınlama — ECharts'ın `stack` davranışının portu: aynı yığın adını
//! taşıyan seriler, işaretlerine göre ayrı ayrı birikir
//! (`data/helper/dataStackHelper.ts` içindeki `samesign` davranışı).

use std::collections::HashMap;

use crate::model::seri::Seri;

/// Bir serinin bir veri noktasındaki dikey aralığı: `(taban, tepe)`.
/// Yığınsız serilerde taban 0'dır. `None` boş değeri gösterir.
pub type YığınAralığı = Option<(f64, f64)>;

/// Serinin yığın kimliği; yığın yoksa seri sırasından tekil kimlik üretir
/// (`getSeriesStackId` karşılığı).
pub fn yığın_kimliği(seri: &Seri, sıra: usize) -> String {
    let yığın = match seri {
        Seri::Çizgi(s) => s.yığın.as_deref(),
        Seri::Sütun(s) => s.yığın.as_deref(),
        _ => None,
    };
    match yığın {
        Some(ad) => format!("__yığın_{ad}"),
        None => format!("__seri_{sıra}"),
    }
}

/// Görünür serilerin her veri noktası için `(taban, tepe)` aralıklarını
/// hesaplar. `görünür[i] == false` olan seriler birikime katılmaz ve boş
/// liste alır.
///
/// Yalnızca çizgi ve sütun serileri yığınlanır; aynı yığında pozitif ve
/// negatif değerler ayrı birikir.
pub fn yığın_aralıkları(seriler: &[Seri], görünür: &[bool]) -> Vec<Vec<YığınAralığı>> {
    // yığın adı -> (pozitif birikimler, negatif birikimler)
    let mut birikimler: HashMap<String, (Vec<f64>, Vec<f64>)> = HashMap::new();
    let mut sonuç: Vec<Vec<YığınAralığı>> = Vec::with_capacity(seriler.len());

    for (sıra, seri) in seriler.iter().enumerate() {
        if !görünür.get(sıra).copied().unwrap_or(true) {
            sonuç.push(Vec::new());
            continue;
        }
        let yığın_adı = match seri {
            Seri::Çizgi(s) => s.yığın.clone(),
            Seri::Sütun(s) => s.yığın.clone(),
            _ => None,
        };
        let veri = seri.veri();
        let mut seri_aralıkları: Vec<YığınAralığı> = Vec::with_capacity(veri.len());

        match yığın_adı {
            Some(ad) => {
                let (pozitif, negatif) = birikimler
                    .entry(ad)
                    .or_insert_with(|| (Vec::new(), Vec::new()));
                if pozitif.len() < veri.len() {
                    pozitif.resize(veri.len(), 0.0);
                    negatif.resize(veri.len(), 0.0);
                }
                for (i, öğe) in veri.iter().enumerate() {
                    match öğe.değer.sayı() {
                        Some(d) if d.is_finite() => {
                            if d >= 0.0 {
                                let taban = pozitif[i];
                                pozitif[i] = taban + d;
                                seri_aralıkları.push(Some((taban, taban + d)));
                            } else {
                                let taban = negatif[i];
                                negatif[i] = taban + d;
                                seri_aralıkları.push(Some((taban, taban + d)));
                            }
                        }
                        _ => seri_aralıkları.push(None),
                    }
                }
            }
            None => {
                for öğe in veri {
                    match öğe.değer.sayı() {
                        Some(d) if d.is_finite() => seri_aralıkları.push(Some((0.0, d))),
                        _ => seri_aralıkları.push(None),
                    }
                }
            }
        }
        sonuç.push(seri_aralıkları);
    }
    sonuç
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::model::seri::{Seri, ÇizgiSerisi};

    #[test]
    fn yığın_birikimi() {
        let s1: Seri = ÇizgiSerisi::yeni().yığın("t").veri([10.0, 20.0]).into();
        let s2: Seri = ÇizgiSerisi::yeni().yığın("t").veri([5.0, 7.0]).into();
        let aralıklar = yığın_aralıkları(&[s1, s2], &[true, true]);
        assert_eq!(aralıklar[0][0], Some((0.0, 10.0)));
        assert_eq!(aralıklar[1][0], Some((10.0, 15.0)));
        assert_eq!(aralıklar[1][1], Some((20.0, 27.0)));
    }

    #[test]
    fn negatifler_ayrı_birikir() {
        let s1: Seri = ÇizgiSerisi::yeni().yığın("t").veri([10.0]).into();
        let s2: Seri = ÇizgiSerisi::yeni().yığın("t").veri([-4.0]).into();
        let aralıklar = yığın_aralıkları(&[s1, s2], &[true, true]);
        assert_eq!(aralıklar[0][0], Some((0.0, 10.0)));
        assert_eq!(aralıklar[1][0], Some((0.0, -4.0)));
    }
}
