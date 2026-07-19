//! Logaritmik ölçek — `echarts/src/scale/Log.ts` portu.
//!
//! Değerler log uzayına taşınır, "güzel" çentikler log uzayında tam sayı
//! adımlarla üretilir ve `taban^k` olarak geri çevrilir.

use crate::olcek::Çentik;
use crate::yardimci::bicim::binlik_ayır;
use crate::yardimci::sayi::{GüzelKip, doğrusal_eşle, güzel_sayı, yuvarla};

/// Logaritmik değer ekseni ölçeği (`LogScale`).
#[derive(Clone, Debug)]
pub struct LogÖlçeği {
    pub taban: f64,
    /// Log uzayındaki kapsam.
    pub log_kapsam: [f64; 2],
    /// Log uzayındaki çentik adımı (≥ 1 tam sayı).
    pub adım: f64,
}

impl LogÖlçeği {
    /// `scale/helper.ts` içindeki `logScaleLogTick` portu.
    pub fn log_dönüşümü(değer: f64, taban: f64) -> f64 {
        değer.ln() / taban.ln()
    }

    pub fn kur(
        veri_kapsamı: [f64; 2],
        taban: f64,
        sabit_en_az: Option<f64>,
        sabit_en_çok: Option<f64>,
        bölme_sayısı: usize,
    ) -> Self {
        let taban = if taban > 1.0 { taban } else { 10.0 };
        let mut en_az = sabit_en_az.unwrap_or(veri_kapsamı[0]);
        let mut en_çok = sabit_en_çok.unwrap_or(veri_kapsamı[1]);
        if !(en_az.is_finite() && en_az > 0.0) {
            en_az = 1.0;
        }
        if !(en_çok.is_finite() && en_çok > 0.0) {
            en_çok = taban;
        }
        if en_çok < en_az {
            std::mem::swap(&mut en_az, &mut en_çok);
        }

        let mut log_kapsam = [
            Self::log_dönüşümü(en_az, taban),
            Self::log_dönüşümü(en_çok, taban),
        ];

        // Log uzayında güzel adım: en az 1 olacak biçimde tam sayıya yuvarla.
        let açıklık = log_kapsam[1] - log_kapsam[0];
        let ham_adım = güzel_sayı(açıklık / bölme_sayısı.max(1) as f64, GüzelKip::Yuvarlak);
        let adım = ham_adım.round().max(1.0);

        // Sabitlenmemiş uçları adım katlarına genişlet.
        if sabit_en_az.is_none() {
            log_kapsam[0] = (log_kapsam[0] / adım).floor() * adım;
        }
        if sabit_en_çok.is_none() {
            log_kapsam[1] = (log_kapsam[1] / adım).ceil() * adım;
        }
        if log_kapsam[0] == log_kapsam[1] {
            log_kapsam[1] += adım;
        }

        LogÖlçeği {
            taban,
            log_kapsam,
            adım,
        }
    }

    /// Veri uzayındaki kapsam.
    pub fn veri_kapsamı(&self) -> [f64; 2] {
        [
            self.taban.powf(self.log_kapsam[0]),
            self.taban.powf(self.log_kapsam[1]),
        ]
    }

    pub fn oranla(&self, değer: f64) -> f64 {
        if değer <= 0.0 || !değer.is_finite() {
            return 0.0;
        }
        doğrusal_eşle(
            Self::log_dönüşümü(değer, self.taban),
            self.log_kapsam,
            [0.0, 1.0],
            true,
        )
    }

    pub fn orandan(&self, oran: f64) -> f64 {
        self.taban
            .powf(doğrusal_eşle(oran, [0.0, 1.0], self.log_kapsam, true))
    }

    pub fn çentikler(&self) -> Vec<Çentik> {
        let mut sonuç = Vec::new();
        let mut k = self.log_kapsam[0];
        let güvenlik_sınırı = 1000;
        let mut sayaç = 0;
        while k <= self.log_kapsam[1] + 1e-9 && sayaç < güvenlik_sınırı {
            // `logScalePowTick`: yuvarlama hatasına karşı log-uzayı değerini
            // önce tam sayıya oturt.
            let düz_k = yuvarla(k, 9);
            sonuç.push(Çentik {
                değer: self.taban.powf(düz_k),
            });
            k += self.adım;
            sayaç += 1;
        }
        sonuç
    }

    pub fn etiket(&self, değer: f64) -> String {
        // 6'nın `5.999999999999999` görünmesini önle (#4158).
        // ECharts `LogScale.getLabel` çağrısını `IntervalScale.getLabel`'a
        // yönlendirir; o da `addCommas` ile binlik ayraçlarını ekler.
        binlik_ayır(yuvarla(değer, 9))
    }
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

    #[test]
    fn onluk_taban() {
        let ö = LogÖlçeği::kur([3.0, 700.0], 10.0, None, None, 5);
        let ç: Vec<f64> = ö.çentikler().iter().map(|t| t.değer).collect();
        assert_eq!(ç, vec![1.0, 10.0, 100.0, 1000.0]);
    }

    #[test]
    fn oranlar() {
        let ö = LogÖlçeği::kur([1.0, 1000.0], 10.0, None, None, 5);
        assert!((ö.oranla(10.0) - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn etiketler_interval_olcegi_gibi_binlik_ayrac_kullanir() {
        let ö = LogÖlçeği::kur([0.001, 10_000.0], 10.0, None, None, 5);
        assert_eq!(ö.etiket(0.001), "0.001");
        assert_eq!(ö.etiket(10_000.0), "10,000");
    }
}
