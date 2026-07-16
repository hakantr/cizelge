//! Sütun genişlik/kaydırma yerleşimi — `echarts/src/layout/barGrid.ts`
//! içindeki `calcBarWidthAndOffset` portu.

use std::collections::HashMap;

use crate::model::Uzunluk;

/// Yerleşime giren bir serinin bilgileri.
#[derive(Clone, Debug)]
pub struct SütunSerisiBilgisi {
    /// Yığın kimliği; yığınsız seriler `__ec_stack_<sıra>` benzeri tekil
    /// kimlik alır (`getSeriesStackId` davranışı).
    pub yığın_kimliği: String,
    pub genişlik: Option<Uzunluk>,
    pub en_çok_genişlik: Option<Uzunluk>,
    pub en_az_genişlik: Option<Uzunluk>,
    pub sütun_boşluğu: Option<Uzunluk>,
    pub kategori_boşluğu: Option<Uzunluk>,
}

/// Bir yığın sütununun bant içindeki konumu.
#[derive(Clone, Copy, Debug)]
pub struct SütunKonumu {
    /// Bant merkezine göre kaydırma (sol kenar).
    pub kaydırma: f32,
    pub genişlik: f32,
}

#[derive(Default)]
struct YığınBilgisi {
    genişlik: f32,
    en_çok: f32,
    en_az: Option<f32>,
}

fn uzunluk_çöz(u: &Option<Uzunluk>, bütün: f32) -> Option<f32> {
    u.as_ref().map(|u| u.çöz(bütün))
}

/// `calcBarWidthAndOffset` portu: her yığın kimliği için bant merkezine göre
/// kaydırma ve genişlik hesaplar.
pub fn sütun_yerleşimi(
    bant_genişliği: f32,
    seriler: &[SütunSerisiBilgisi],
) -> HashMap<String, SütunKonumu> {
    let mut kalan_genişlik = bant_genişliği;
    let mut otomatik_sayısı: usize = 0;
    let mut kategori_boşluğu_seçeneği: Option<Uzunluk> = None;
    // ECharts: ilk serinin `defaultBarGap`i %30'dur.
    let mut sütun_boşluğu_oranı: f32 = 0.3;

    let mut yığın_sırası: Vec<String> = Vec::new();
    let mut yığınlar: HashMap<String, YığınBilgisi> = HashMap::new();

    for bilgi in seriler {
        let kimlik = &bilgi.yığın_kimliği;
        if !yığınlar.contains_key(kimlik) {
            otomatik_sayısı += 1;
            yığın_sırası.push(kimlik.clone());
            yığınlar.insert(kimlik.clone(), YığınBilgisi::default());
        }
        let yığın = yığınlar.get_mut(kimlik).unwrap();

        if let Some(g) = uzunluk_çöz(&bilgi.genişlik, bant_genişliği) {
            if yığın.genişlik == 0.0 && g > 0.0 {
                // #6312: genişlik burada kısıtlanmaz.
                yığın.genişlik = g;
                let kırpılmış = g.min(kalan_genişlik);
                kalan_genişlik -= kırpılmış;
            }
        }
        if let Some(eç) = uzunluk_çöz(&bilgi.en_çok_genişlik, bant_genişliği) {
            if eç > 0.0 {
                yığın.en_çok = eç;
            }
        }
        if let Some(ea) = uzunluk_çöz(&bilgi.en_az_genişlik, bant_genişliği) {
            if ea > 0.0 {
                yığın.en_az = Some(ea);
            }
        }
        if let Some(b) = &bilgi.sütun_boşluğu {
            sütun_boşluğu_oranı = b.çöz(1.0);
        }
        if bilgi.kategori_boşluğu.is_some() {
            kategori_boşluğu_seçeneği = bilgi.kategori_boşluğu;
        }
    }

    // Grupta sütun sayısı arttıkça kategoriler arası boşluk daralır; yoksa
    // sütunlar aşırı incelir.
    let kategori_boşluğu = match kategori_boşluğu_seçeneği {
        Some(u) => u.çöz(bant_genişliği),
        None => {
            let yüzde = (35.0 - yığın_sırası.len() as f32 * 4.0).max(15.0);
            yüzde / 100.0 * bant_genişliği
        }
    };

    let mut otomatik_genişlik = ((kalan_genişlik - kategori_boşluğu)
        / (otomatik_sayısı as f32 + (otomatik_sayısı as f32 - 1.0) * sütun_boşluğu_oranı))
        .max(0.0);

    // Otomatik genişliğin en çok / en az sınırlarını aşan sütunları sabitle.
    for kimlik in &yığın_sırası {
        let yığın = yığınlar.get_mut(kimlik).unwrap();
        if yığın.genişlik == 0.0 {
            let mut son_genişlik = otomatik_genişlik;
            if yığın.en_çok > 0.0 && yığın.en_çok < son_genişlik {
                son_genişlik = yığın.en_çok.min(kalan_genişlik);
            }
            // `minWidth` önceliklidir: sütunun görünür kalmasını belirler.
            if let Some(ea) = yığın.en_az {
                if ea > son_genişlik {
                    son_genişlik = ea;
                }
            }
            if son_genişlik != otomatik_genişlik {
                yığın.genişlik = son_genişlik;
                kalan_genişlik -= son_genişlik + sütun_boşluğu_oranı * son_genişlik;
                otomatik_sayısı -= 1;
            }
        } else {
            // `barMinWidth/barMaxWidth`, `barWidth`ten önceliklidir (CSS gibi).
            let mut son_genişlik = yığın.genişlik;
            if yığın.en_çok > 0.0 {
                son_genişlik = son_genişlik.min(yığın.en_çok);
            }
            if let Some(ea) = yığın.en_az {
                son_genişlik = son_genişlik.max(ea);
            }
            yığın.genişlik = son_genişlik;
            kalan_genişlik -= son_genişlik + sütun_boşluğu_oranı * son_genişlik;
            otomatik_sayısı -= 1;
        }
    }

    // Genişliği yeniden hesapla.
    otomatik_genişlik = ((kalan_genişlik - kategori_boşluğu)
        / (otomatik_sayısı as f32 + (otomatik_sayısı as f32 - 1.0) * sütun_boşluğu_oranı))
        .max(0.0);

    let mut genişlik_toplamı = 0.0;
    let mut son_genişlik = 0.0;
    for kimlik in &yığın_sırası {
        let yığın = yığınlar.get_mut(kimlik).unwrap();
        if yığın.genişlik == 0.0 {
            yığın.genişlik = otomatik_genişlik;
        }
        son_genişlik = yığın.genişlik;
        genişlik_toplamı += yığın.genişlik * (1.0 + sütun_boşluğu_oranı);
    }
    if !yığın_sırası.is_empty() {
        genişlik_toplamı -= son_genişlik * sütun_boşluğu_oranı;
    }

    let mut sonuç = HashMap::new();
    let mut kaydırma = -genişlik_toplamı / 2.0;
    for kimlik in &yığın_sırası {
        let yığın = &yığınlar[kimlik];
        sonuç.insert(
            kimlik.clone(),
            SütunKonumu { kaydırma, genişlik: yığın.genişlik },
        );
        kaydırma += yığın.genişlik * (1.0 + sütun_boşluğu_oranı);
    }
    sonuç
}

#[cfg(test)]
mod testler {
    use super::*;

    fn bilgi(kimlik: &str) -> SütunSerisiBilgisi {
        SütunSerisiBilgisi {
            yığın_kimliği: kimlik.to_string(),
            genişlik: None,
            en_çok_genişlik: None,
            en_az_genişlik: None,
            sütun_boşluğu: None,
            kategori_boşluğu: None,
        }
    }

    #[test]
    fn iki_seri_ortalanır() {
        let düzen = sütun_yerleşimi(100.0, &[bilgi("a"), bilgi("b")]);
        let a = düzen["a"];
        let b = düzen["b"];
        assert!(a.genişlik > 0.0);
        assert!((a.genişlik - b.genişlik).abs() < 1e-4);
        // Yerleşim bant merkezine göre bakışıktır.
        assert!((a.kaydırma + b.kaydırma + b.genişlik).abs() < 1e-3);
    }

    #[test]
    fn yığın_paylaşımı() {
        let mut a = bilgi("toplam");
        let mut b = bilgi("toplam");
        a.genişlik = None;
        b.genişlik = None;
        let düzen = sütun_yerleşimi(100.0, &[a, b]);
        // Aynı yığındaki seriler tek sütun konumunu paylaşır.
        assert_eq!(düzen.len(), 1);
    }
}
