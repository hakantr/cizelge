#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nokta {
    pub x: f64,
    pub y: f64,
}

impl Nokta {
    pub const fn yeni(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Largest-Triangle-Three-Buckets örneklemesi. Noktaları kopyalamak yerine
/// kaynak indekslerini döndürür; kararlı kimlik/etiket kaybolmaz.
pub fn lttb_indeksleri(noktalar: &[Nokta], esik: usize) -> Vec<usize> {
    if esik == 0 || noktalar.is_empty() {
        return Vec::new();
    }
    if esik >= noktalar.len() || esik < 3 {
        return (0..noktalar.len().min(esik)).collect();
    }

    let mut secilen = Vec::with_capacity(esik);
    secilen.push(0);
    let mut onceki = 0;

    for kova in 0..(esik - 2) {
        let sonraki_bas = kova_siniri(kova + 1, noktalar.len(), esik).min(noktalar.len() - 1);
        let sonraki_son = kova_siniri(kova + 2, noktalar.len(), esik).min(noktalar.len());
        let ortalama_araligi = &noktalar[sonraki_bas..sonraki_son];
        let (ortalama_x, ortalama_y) = if ortalama_araligi.is_empty() {
            let son = noktalar[noktalar.len() - 1];
            (son.x, son.y)
        } else {
            let (x, y) = ortalama_araligi
                .iter()
                .fold((0.0, 0.0), |(x, y), nokta| (x + nokta.x, y + nokta.y));
            let sayi = usize_f64(ortalama_araligi.len());
            (x / sayi, y / sayi)
        };

        let aday_bas = kova_siniri(kova, noktalar.len(), esik).min(noktalar.len() - 1);
        let aday_son = kova_siniri(kova + 1, noktalar.len(), esik)
            .min(noktalar.len() - 1)
            .max(aday_bas + 1);
        let a = noktalar[onceki];
        let (secilen_indeks, _) = (aday_bas..aday_son)
            .map(|indeks| {
                let b = noktalar[indeks];
                let alan =
                    ((a.x - ortalama_x) * (b.y - a.y) - (a.x - b.x) * (ortalama_y - a.y)).abs();
                (indeks, alan)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((aday_bas, 0.0));
        secilen.push(secilen_indeks);
        onceki = secilen_indeks;
    }
    secilen.push(noktalar.len() - 1);
    secilen
}

/// Çubuk ve scatter gibi yerel şekli koruması gereken profiller için
/// uçları koruyan deterministik eşit-aralık örneklemesi.
pub fn esit_aralik_indeksleri(nokta_sayisi: usize, esik: usize) -> Vec<usize> {
    if esik == 0 || nokta_sayisi == 0 {
        return Vec::new();
    }
    if esik >= nokta_sayisi {
        return (0..nokta_sayisi).collect();
    }
    if esik == 1 {
        return vec![0];
    }
    (0..esik)
        .map(|sira| sira.saturating_mul(nokta_sayisi - 1) / esik.saturating_sub(1).max(1))
        .collect()
}

/// Alan/çubuk profillerinde her kovadaki yerel minimum ve maksimumu korur.
/// Sonuç kaynak sırasındadır; böylece sivri uçlar ve negatif/pozitif geçişler
/// eşit-aralık örneklemesinde kaybolmaz.
pub fn min_maks_indeksleri(noktalar: &[Nokta], esik: usize) -> Vec<usize> {
    if esik == 0 || noktalar.is_empty() {
        return Vec::new();
    }
    if esik >= noktalar.len() {
        return (0..noktalar.len()).collect();
    }
    if esik < 3 {
        return esit_aralik_indeksleri(noktalar.len(), esik);
    }

    let kova_sayisi = (esik / 2).max(1);
    let mut sonuc = Vec::with_capacity(esik);
    for kova in 0..kova_sayisi {
        let bas = kova.saturating_mul(noktalar.len()) / kova_sayisi;
        let son = ((kova + 1).saturating_mul(noktalar.len()) / kova_sayisi).max(bas + 1);
        let aralik = bas..son.min(noktalar.len());
        let min = aralik
            .clone()
            .min_by(|a, b| noktalar[*a].y.total_cmp(&noktalar[*b].y));
        let max = aralik.max_by(|a, b| noktalar[*a].y.total_cmp(&noktalar[*b].y));
        if let (Some(min), Some(max)) = (min, max) {
            if min <= max {
                sonuc.push(min);
                if min != max {
                    sonuc.push(max);
                }
            } else {
                sonuc.push(max);
                sonuc.push(min);
            }
        }
    }
    sonuc.sort_unstable();
    sonuc.dedup();
    if sonuc.len() > esik {
        sonuc = esit_aralik_indeksleri(sonuc.len(), esik)
            .into_iter()
            .map(|sira| sonuc[sira])
            .collect();
    }
    sonuc
}

/// Dağılım/baloncuk profillerinde iki boyutlu hücre başına tek temsilci
/// seçer. Girdi sırası deterministik bağ kırıcıdır ve sonuç kaynak sırasına
/// döndürülür.
pub fn izgara_indeksleri(noktalar: &[Nokta], esik: usize) -> Vec<usize> {
    use std::collections::BTreeMap;

    if esik == 0 || noktalar.is_empty() {
        return Vec::new();
    }
    if esik >= noktalar.len() {
        return (0..noktalar.len()).collect();
    }
    let (min_x, max_x, min_y, max_y) = noktalar.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), nokta| {
            (
                min_x.min(nokta.x),
                max_x.max(nokta.x),
                min_y.min(nokta.y),
                max_y.max(nokta.y),
            )
        },
    );
    let kenar = tam_kare_koku(esik).max(1);
    let x_span = (max_x - min_x).max(f64::EPSILON);
    let y_span = (max_y - min_y).max(f64::EPSILON);
    let mut hucreler = BTreeMap::new();
    for (sira, nokta) in noktalar.iter().enumerate() {
        let x = (((nokta.x - min_x) / x_span) * usize_f64(kenar)).floor();
        let y = (((nokta.y - min_y) / y_span) * usize_f64(kenar)).floor();
        let x = f64_usize(x).min(kenar - 1);
        let y = f64_usize(y).min(kenar - 1);
        hucreler.entry((x, y)).or_insert(sira);
        if hucreler.len() == esik {
            break;
        }
    }
    let mut sonuc: Vec<_> = hucreler.into_values().collect();
    sonuc.sort_unstable();
    sonuc
}

fn kova_siniri(carpan: usize, nokta_sayisi: usize, esik: usize) -> usize {
    carpan.saturating_mul(nokta_sayisi.saturating_sub(2)) / esik.saturating_sub(2).max(1) + 1
}

fn usize_f64(deger: usize) -> f64 {
    f64::from(u32::try_from(deger).unwrap_or(u32::MAX))
}

fn tam_kare_koku(deger: usize) -> usize {
    let mut aday = 1usize;
    while aday.saturating_mul(aday) < deger {
        aday = aday.saturating_add(1);
    }
    aday
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn f64_usize(deger: f64) -> usize {
    if deger.is_finite() && deger > 0.0 {
        deger as usize
    } else {
        0
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn ilk_son_ve_tepe_korunur() {
        let noktalar: Vec<_> = (0..100)
            .map(|x| Nokta::yeni(f64::from(x), if x == 50 { 1000.0 } else { 1.0 }))
            .collect();
        let indeksler = lttb_indeksleri(&noktalar, 10);
        assert_eq!(indeksler.first(), Some(&0));
        assert_eq!(indeksler.last(), Some(&99));
        assert!(indeksler.contains(&50));
        assert_eq!(indeksler.len(), 10);
    }

    #[test]
    fn bos_ve_kucuk_esik_guvenli() {
        assert!(lttb_indeksleri(&[], 10).is_empty());
        assert_eq!(lttb_indeksleri(&[Nokta::yeni(0.0, 0.0)], 1), vec![0]);
    }

    #[test]
    fn esit_aralik_uclari_korur() {
        assert_eq!(esit_aralik_indeksleri(100, 4), vec![0, 33, 66, 99]);
        assert_eq!(esit_aralik_indeksleri(2, 10), vec![0, 1]);
        assert!(esit_aralik_indeksleri(10, 0).is_empty());
    }

    #[test]
    fn min_maks_sivri_uclari_korur() {
        let noktalar: Vec<_> = (0..100)
            .map(|x| {
                Nokta::yeni(
                    f64::from(x),
                    if x == 24 {
                        -100.0
                    } else if x == 75 {
                        200.0
                    } else {
                        1.0
                    },
                )
            })
            .collect();
        let sonuc = min_maks_indeksleri(&noktalar, 12);
        assert!(sonuc.contains(&24));
        assert!(sonuc.contains(&75));
        assert!(sonuc.len() <= 12);
    }

    #[test]
    fn izgara_ayni_hucreyi_bir_kez_secer() {
        let noktalar: Vec<_> = (0..10_000)
            .map(|sira| Nokta::yeni(f64::from(sira % 100), f64::from(sira / 100)))
            .collect();
        let sonuc = izgara_indeksleri(&noktalar, 100);
        assert!(!sonuc.is_empty());
        assert!(sonuc.len() <= 100);
        assert!(sonuc.windows(2).all(|cift| cift[0] < cift[1]));
    }
}
