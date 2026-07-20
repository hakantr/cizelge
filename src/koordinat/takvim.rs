//! ECharts `Calendar` veri/piksel dönüşümü.

use crate::hata::BilesenHatasi;
use crate::koordinat::Dikdörtgen;
use crate::model::takvim::{TakvimKoordinatı, TakvimYönü};
use crate::model::{DikeyKonum, YatayKonum};

const GÜN_MS: f64 = 86_400_000.0;

fn gün_sayısı(ms: f64) -> Option<i64> {
    ms.is_finite().then(|| (ms / GÜN_MS).floor() as i64)
}

/// Unix gününden ECharts/JS haftanın gününe (`0=Pazar .. 6=Cumartesi`).
fn haftanın_günü(gün: i64) -> usize {
    // 1970-01-01 Perşembe'dir (JS `Date#getDay()` dizininde 4).
    (gün.rem_euclid(7) as usize + 4) % 7
}

#[derive(Clone, Debug, PartialEq)]
pub struct TakvimYerleşimi {
    pub dış_kutu: Dikdörtgen,
    pub gövde_kutusu: Dikdörtgen,
    pub başlangıç_günü: i64,
    pub bitiş_günü: i64,
    pub hafta_sayısı: usize,
    pub hücre_genişliği: f32,
    pub hücre_yüksekliği: f32,
    pub hücre_boşluğu: f32,
    pub ilk_hafta_günü: usize,
    pub ilk_gün: usize,
    pub yön: TakvimYönü,
}

impl TakvimYerleşimi {
    pub fn kur(seçenek: &TakvimKoordinatı, tuval: (f32, f32)) -> Result<Self, BilesenHatasi> {
        let Some(başlangıç_günü) = gün_sayısı(seçenek.aralık.başlangıç_ms) else {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "calendar.range",
                ayrıntı: "başlangıç sonlu bir Unix milisaniyesi olmalıdır".to_owned(),
            });
        };
        let Some(bitiş_günü) = gün_sayısı(seçenek.aralık.bitiş_ms) else {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "calendar.range",
                ayrıntı: "bitiş sonlu bir Unix milisaniyesi olmalıdır".to_owned(),
            });
        };
        if bitiş_günü < başlangıç_günü {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "calendar.range",
                ayrıntı: "bitiş başlangıçtan önce olamaz".to_owned(),
            });
        }
        let ilk_gün = seçenek.ilk_gün % 7;
        let ilk_hafta_günü = (haftanın_günü(başlangıç_günü) + 7 - ilk_gün) % 7;
        let gün_adedi = bitiş_günü.saturating_sub(başlangıç_günü).saturating_add(1) as usize;
        let hafta_sayısı = gün_adedi.saturating_add(ilk_hafta_günü).saturating_add(6) / 7;
        let boşluk = seçenek.hücre_boşluğu.max(0.0);
        let (sütun_sayısı, satır_sayısı) = match seçenek.yön {
            TakvimYönü::Yatay => (hafta_sayısı.max(1), 7usize),
            TakvimYönü::Dikey => (7usize, hafta_sayısı.max(1)),
        };

        let sayısal_sol = match seçenek.sol {
            Some(YatayKonum::Değer(uzunluk)) => Some(uzunluk.çöz(tuval.0)),
            _ => None,
        };
        let sayısal_üst = match seçenek.üst {
            Some(DikeyKonum::Değer(uzunluk)) => Some(uzunluk.çöz(tuval.1)),
            _ => None,
        };
        let sağ = seçenek.sağ.map(|uzunluk| uzunluk.çöz(tuval.0));
        let alt = seçenek.alt.map(|uzunluk| uzunluk.çöz(tuval.1));

        // Calendar._update: sabit cellSize ilgili box boyutunu belirler;
        // `auto` boyutta ise box yerleşimi çözülüp hücre sayısına bölünür.
        let genişlik = seçenek.hücre_genişliği.map_or_else(
            || {
                seçenek.genişlik.map_or_else(
                    || match (sayısal_sol, sağ) {
                        (Some(sol), Some(sağ)) => tuval.0 - sol - sağ,
                        (Some(sol), None) => tuval.0 - sol,
                        (None, Some(sağ)) => tuval.0 - sağ,
                        (None, None) => tuval.0,
                    },
                    |uzunluk| uzunluk.çöz(tuval.0),
                )
            },
            |hücre| hücre * sütun_sayısı as f32 + boşluk * sütun_sayısı.saturating_sub(1) as f32,
        );
        let yükseklik = seçenek.hücre_yüksekliği.map_or_else(
            || {
                seçenek.yükseklik.map_or_else(
                    || match (sayısal_üst, alt) {
                        (Some(üst), Some(alt)) => tuval.1 - üst - alt,
                        (Some(üst), None) => tuval.1 - üst,
                        (None, Some(alt)) => tuval.1 - alt,
                        (None, None) => tuval.1,
                    },
                    |uzunluk| uzunluk.çöz(tuval.1),
                )
            },
            |hücre| hücre * satır_sayısı as f32 + boşluk * satır_sayısı.saturating_sub(1) as f32,
        );
        let genişlik = genişlik.max(1.0);
        let yükseklik = yükseklik.max(1.0);
        let x = match seçenek.sol {
            Some(YatayKonum::Sol) => 0.0,
            Some(YatayKonum::Orta) => (tuval.0 - genişlik) / 2.0,
            Some(YatayKonum::Sağ) => tuval.0 - genişlik,
            Some(YatayKonum::Değer(uzunluk)) => uzunluk.çöz(tuval.0),
            None => tuval.0 - sağ.unwrap_or(0.0) - genişlik,
        };
        let y = match seçenek.üst {
            Some(DikeyKonum::Üst) => 0.0,
            Some(DikeyKonum::Orta) => (tuval.1 - yükseklik) / 2.0,
            Some(DikeyKonum::Alt) => tuval.1 - yükseklik,
            Some(DikeyKonum::Değer(uzunluk)) => uzunluk.çöz(tuval.1),
            None => tuval.1 - alt.unwrap_or(0.0) - yükseklik,
        };
        let dış_kutu = Dikdörtgen::yeni(x, y, genişlik, yükseklik);
        // ECharts'ta ay/gün/yıl etiketleri gövdenin dışında yaşar; box
        // boyutundan iç pay ayırmazlar.
        let gövde_kutusu = dış_kutu;
        let kullanılabilir_g =
            (gövde_kutusu.genişlik - boşluk * sütun_sayısı.saturating_sub(1) as f32).max(1.0);
        let kullanılabilir_y =
            (gövde_kutusu.yükseklik - boşluk * satır_sayısı.saturating_sub(1) as f32).max(1.0);
        let otomatik_g = kullanılabilir_g / sütun_sayısı as f32;
        let otomatik_y = kullanılabilir_y / satır_sayısı as f32;
        Ok(Self {
            dış_kutu,
            gövde_kutusu,
            başlangıç_günü,
            bitiş_günü,
            hafta_sayısı,
            hücre_genişliği: seçenek.hücre_genişliği.unwrap_or(otomatik_g).max(1.0),
            hücre_yüksekliği: seçenek.hücre_yüksekliği.unwrap_or(otomatik_y).max(1.0),
            hücre_boşluğu: boşluk,
            ilk_hafta_günü,
            ilk_gün,
            yön: seçenek.yön,
        })
    }

    pub fn hücre(&self, zaman_ms: f64) -> Option<Dikdörtgen> {
        let gün = gün_sayısı(zaman_ms)?;
        if gün < self.başlangıç_günü || gün > self.bitiş_günü {
            return None;
        }
        self.gün_hücresi(gün)
    }

    /// Aralık dışındaki komşu günleri de aynı hafta ızgarasına yansıtır.
    /// Ay ayırıcılarının başlangıç/bitiş merdivenleri bu davranışı kullanır.
    pub fn kısıtsız_hücre(&self, zaman_ms: f64) -> Option<Dikdörtgen> {
        self.gün_hücresi(gün_sayısı(zaman_ms)?)
    }

    fn gün_hücresi(&self, gün: i64) -> Option<Dikdörtgen> {
        let sıra = gün.saturating_sub(self.başlangıç_günü);
        let göreli = sıra.saturating_add(self.ilk_hafta_günü as i64);
        let hafta = göreli.div_euclid(7);
        let hafta_günü = göreli.rem_euclid(7);
        let adım_x = self.hücre_genişliği + self.hücre_boşluğu;
        let adım_y = self.hücre_yüksekliği + self.hücre_boşluğu;
        let (x, y) = match self.yön {
            TakvimYönü::Yatay => (
                self.gövde_kutusu.x + hafta as f32 * adım_x,
                self.gövde_kutusu.y + hafta_günü as f32 * adım_y,
            ),
            TakvimYönü::Dikey => (
                self.gövde_kutusu.x + hafta_günü as f32 * adım_x,
                self.gövde_kutusu.y + hafta as f32 * adım_y,
            ),
        };
        Some(Dikdörtgen::yeni(
            x,
            y,
            self.hücre_genişliği,
            self.hücre_yüksekliği,
        ))
    }

    pub fn veriden_noktaya(&self, zaman_ms: f64) -> Option<(f32, f32)> {
        self.hücre(zaman_ms).map(|hücre| hücre.merkez())
    }

    pub fn noktadan_veriye(&self, nokta: (f32, f32)) -> Option<f64> {
        if !self.gövde_kutusu.içeriyor_mu(nokta) {
            return None;
        }
        let adım_x = self.hücre_genişliği + self.hücre_boşluğu;
        let adım_y = self.hücre_yüksekliği + self.hücre_boşluğu;
        let x = ((nokta.0 - self.gövde_kutusu.x) / adım_x).floor().max(0.0) as usize;
        let y = ((nokta.1 - self.gövde_kutusu.y) / adım_y).floor().max(0.0) as usize;
        let hücre_içi_x = (nokta.0 - self.gövde_kutusu.x) - x as f32 * adım_x;
        let hücre_içi_y = (nokta.1 - self.gövde_kutusu.y) - y as f32 * adım_y;
        if hücre_içi_x > self.hücre_genişliği || hücre_içi_y > self.hücre_yüksekliği {
            return None;
        }
        let (hafta, hafta_günü) = match self.yön {
            TakvimYönü::Yatay => (x, y),
            TakvimYönü::Dikey => (y, x),
        };
        if hafta_günü >= 7 {
            return None;
        }
        let göreli = hafta.saturating_mul(7).saturating_add(hafta_günü);
        let sıra = göreli.checked_sub(self.ilk_hafta_günü)?;
        let gün = self.başlangıç_günü.saturating_add(sıra as i64);
        (gün <= self.bitiş_günü).then_some(gün as f64 * GÜN_MS)
    }

    pub fn içeriyor_mu(&self, nokta: (f32, f32)) -> bool {
        self.dış_kutu.içeriyor_mu(nokta)
    }
}

#[cfg(test)]
mod testler {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::model::takvim::{TakvimAralığı, TakvimKoordinatı, TakvimYönü};

    #[test]
    fn yatay_ve_dikey_tarih_dönüşümü_tersinir() {
        let aralık = TakvimAralığı::yıl(2024);
        for yön in [TakvimYönü::Yatay, TakvimYönü::Dikey] {
            let seçenek = TakvimKoordinatı::yıl(2024).yön(yön);
            let yerleşim = TakvimYerleşimi::kur(&seçenek, (800.0, 260.0)).unwrap();
            let zaman = aralık.başlangıç_ms + 123.0 * GÜN_MS;
            let nokta = yerleşim.veriden_noktaya(zaman).unwrap();
            let geri = yerleşim.noktadan_veriye(nokta).unwrap();
            assert_eq!(geri, zaman);
        }
    }

    #[test]
    fn aralık_dışı_tarih_yoktur() {
        let seçenek = TakvimKoordinatı::yıl(2024);
        let yerleşim = TakvimYerleşimi::kur(&seçenek, (800.0, 260.0)).unwrap();
        assert!(
            yerleşim
                .veriden_noktaya(seçenek.aralık.bitiş_ms + GÜN_MS)
                .is_none()
        );
    }

    #[test]
    fn resmi_2016_isı_haritası_kutusu_ve_hücreleri_çözülür() {
        let seçenek = TakvimKoordinatı::yıl(2016)
            .sol(30.0)
            .sağ(30)
            .üst(120)
            .hücre_boyutu(None, Some(13.0));
        let yerleşim = TakvimYerleşimi::kur(&seçenek, (700.0, 450.0)).unwrap();
        assert_eq!(yerleşim.hafta_sayısı, 53);
        assert!((yerleşim.gövde_kutusu.x - 30.0).abs() < 1e-5);
        assert!((yerleşim.gövde_kutusu.y - 120.0).abs() < 1e-5);
        assert!((yerleşim.gövde_kutusu.genişlik - 640.0).abs() < 1e-5);
        assert!((yerleşim.gövde_kutusu.yükseklik - 91.0).abs() < 1e-5);
        assert!((yerleşim.hücre_genişliği - 640.0 / 53.0).abs() < 1e-5);
        let ilk = yerleşim.hücre(seçenek.aralık.başlangıç_ms).unwrap();
        assert!((ilk.x - 30.0).abs() < 1e-5);
        // 1 Ocak 2016 Cuma; Pazar başlangıcında altıncı satırdır.
        assert!((ilk.y - 185.0).abs() < 1e-5);
    }
}
