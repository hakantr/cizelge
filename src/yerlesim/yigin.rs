//! Yığınlama — ECharts'ın `stack` davranışının portu: aynı yığın adını
//! taşıyan seriler, işaretlerine göre ayrı ayrı birikir
//! (`data/helper/dataStackHelper.ts` içindeki `samesign` davranışı).

use std::collections::HashMap;

use crate::model::deger::VeriÖğesi;
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
pub fn yığın_aralıkları(
    seriler: &[Seri], görünür: &[bool]
) -> Vec<Vec<YığınAralığı>> {
    yığın_aralıkları_seçici(seriler, görünür, |_, _, _, öğe| öğe.değer.sayı())
}

/// Yığın değer boyutunu koordinat sistemine göre seçerek aralıkları hesaplar.
/// Kartezyen barlarda ECharts `getBaseAxis()` zaman eksenini de taban kabul
/// eder; y-zaman tabanlı `[değer, tarih]` verisinde yığılan değer bu nedenle
/// çiftin ilk boyutudur. Genel yordam, çizgi ve klasik kategori barlarının
/// tarihsel `VeriDeğeri::sayı` seçimini değiştirmeden bu boyut seçimini
/// çağırana bırakır.
pub fn yığın_aralıkları_seçici<F>(
    seriler: &[Seri],
    görünür: &[bool],
    mut değer_seç: F,
) -> Vec<Vec<YığınAralığı>>
where
    F: FnMut(usize, &Seri, usize, &VeriÖğesi) -> Option<f64>,
{
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
                    let birikim = match değer_seç(sıra, seri, i, öğe) {
                        Some(d) if d.is_finite() && d >= 0.0 => pozitif.get_mut(i).map(|b| (b, d)),
                        Some(d) if d.is_finite() => negatif.get_mut(i).map(|b| (b, d)),
                        _ => None,
                    };
                    match birikim {
                        Some((birikmiş, d)) => {
                            let taban = *birikmiş;
                            *birikmiş = taban + d;
                            seri_aralıkları.push(Some((taban, taban + d)));
                        }
                        None => seri_aralıkları.push(None),
                    }
                }
            }
            None => {
                for (i, öğe) in veri.iter().enumerate() {
                    match değer_seç(sıra, seri, i, öğe) {
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
#[allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod testler {
    use super::*;
    use crate::model::seri::{Seri, SütunSerisi, ÇizgiSerisi};

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

    #[test]
    fn secici_zaman_tabanli_yatay_sutunun_ilk_boyutunu_yigar() {
        let s1: Seri = SütunSerisi::yeni()
            .yığın("t")
            .veri([[10.0, 1000.0], [20.0, 2000.0]])
            .into();
        let s2: Seri = SütunSerisi::yeni()
            .yığın("t")
            .veri([[5.0, 1000.0], [-7.0, 2000.0]])
            .into();
        let aralıklar =
            yığın_aralıkları_seçici(&[s1, s2], &[true, true], |_, _, _, öğe| öğe.değer.x());

        assert_eq!(aralıklar[0], vec![Some((0.0, 10.0)), Some((0.0, 20.0))]);
        assert_eq!(aralıklar[1], vec![Some((10.0, 15.0)), Some((0.0, -7.0))]);
    }
}
