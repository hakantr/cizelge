//! ECharts `Calendar` veri/piksel dönüşümü.

use crate::hata::BilesenHatasi;
use crate::koordinat::Dikdörtgen;
use crate::model::takvim::{TakvimKoordinatı, TakvimYönü};

const GÜN_MS: f64 = 86_400_000.0;

fn gün_sayısı(ms: f64) -> Option<i64> {
    ms.is_finite().then(|| (ms / GÜN_MS).floor() as i64)
}

/// Unix gününden haftanın gününe (`0=Pazartesi`).
fn haftanın_günü(gün: i64) -> usize {
    (gün.rem_euclid(7) as usize + 3) % 7
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
        let dış_kutu = Dikdörtgen::yeni(
            seçenek.sol.çöz(tuval.0),
            seçenek.üst.çöz(tuval.1),
            seçenek.genişlik.çöz(tuval.0).max(1.0),
            seçenek.yükseklik.çöz(tuval.1).max(1.0),
        );
        let gün_pay = if seçenek.gün_etiketi.göster {
            34.0
        } else {
            0.0
        };
        let ay_pay = if seçenek.ay_etiketi.göster {
            22.0
        } else {
            0.0
        };
        let gövde_kutusu = match seçenek.yön {
            TakvimYönü::Yatay => Dikdörtgen::yeni(
                dış_kutu.x + gün_pay,
                dış_kutu.y + ay_pay,
                (dış_kutu.genişlik - gün_pay).max(1.0),
                (dış_kutu.yükseklik - ay_pay).max(1.0),
            ),
            TakvimYönü::Dikey => Dikdörtgen::yeni(
                dış_kutu.x + ay_pay,
                dış_kutu.y + gün_pay,
                (dış_kutu.genişlik - ay_pay).max(1.0),
                (dış_kutu.yükseklik - gün_pay).max(1.0),
            ),
        };
        let ilk_gün = seçenek.ilk_gün % 7;
        let ilk_hafta_günü = (haftanın_günü(başlangıç_günü) + 7 - ilk_gün) % 7;
        let gün_adedi = bitiş_günü.saturating_sub(başlangıç_günü).saturating_add(1) as usize;
        let hafta_sayısı = gün_adedi.saturating_add(ilk_hafta_günü).saturating_add(6) / 7;
        let boşluk = seçenek.hücre_boşluğu.max(0.0);
        let (otomatik_g, otomatik_y) = match seçenek.yön {
            TakvimYönü::Yatay => (
                gövde_kutusu.genişlik / hafta_sayısı.max(1) as f32 - boşluk,
                gövde_kutusu.yükseklik / 7.0 - boşluk,
            ),
            TakvimYönü::Dikey => (
                gövde_kutusu.genişlik / 7.0 - boşluk,
                gövde_kutusu.yükseklik / hafta_sayısı.max(1) as f32 - boşluk,
            ),
        };
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
        let sıra = gün.saturating_sub(self.başlangıç_günü) as usize;
        let göreli = sıra.saturating_add(self.ilk_hafta_günü);
        let hafta = göreli / 7;
        let hafta_günü = göreli % 7;
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
}
