//! Aralık (sayısal değer) ölçeği — `echarts/src/scale/Interval.ts` ile
//! `scale/helper.ts` içindeki `intervalScaleNiceTicks` portu.

use crate::olcek::Çentik;
use crate::yardimci::bicim::çentik_değeri_biçimle;
use crate::yardimci::sayi::{
    GüzelKip, doğrusal_eşle, geçerli_kapsam_sayısı, güzel_sayı, hassasiyet, yuvarla,
};

/// "Güzel" çentik hesabının sonucu (`intervalScaleNiceTicksResult`).
#[derive(Clone, Copy, Debug)]
pub struct GüzelÇentikSonucu {
    pub adım: f64,
    pub adım_hassasiyeti: usize,
    pub güzel_kapsam: [f64; 2],
}

/// `scale/helper.ts` içindeki `getIntervalPrecision` portu:
/// çentikler için iki basamak fazla hassasiyet.
pub fn adım_hassasiyeti(adım: f64) -> usize {
    hassasiyet(adım) + 2
}

/// `scale/helper.ts` içindeki `intervalScaleNiceTicks` portu.
pub fn güzel_çentikler(
    kapsam: [f64; 2],
    bölme_sayısı: usize,
    en_küçük_adım: Option<f64>,
    en_büyük_adım: Option<f64>,
) -> GüzelÇentikSonucu {
    let açıklık = kapsam[1] - kapsam[0];
    let mut adım = güzel_sayı(açıklık / bölme_sayısı.max(1) as f64, GüzelKip::Yuvarlak);
    if let Some(ek) = en_küçük_adım
        && adım < ek
    {
        adım = ek;
    }
    if let Some(eb) = en_büyük_adım
        && adım > eb
    {
        adım = eb;
    }
    let h = adım_hassasiyeti(adım);
    // Özgün kapsamın içinde kalan "güzelleştirilmiş" kapsam.
    let güzel_kapsam = [
        yuvarla((kapsam[0] / adım).ceil() * adım, h),
        yuvarla((kapsam[1] / adım).floor() * adım, h),
    ];
    GüzelÇentikSonucu {
        adım,
        adım_hassasiyeti: h,
        güzel_kapsam,
    }
}

/// Kapsamı geçerli hale getirir: uçlar eşitse genişletir, geçersizse
/// `[0, 1]`e düşer. `scale/helper.ts` içindeki
/// `intervalScaleEnsureValidExtent` portu.
pub fn kapsamı_geçerle(ham: [f64; 2], sabit_uçlar: [bool; 2]) -> [f64; 2] {
    let mut kapsam = ham;
    if kapsam[0] == kapsam[1] {
        if kapsam[0] != 0.0 {
            // Uçlar eşit ama sıfır değil: iki yana doğru genişlet (#13154).
            let genişleme = kapsam[0].abs();
            if !sabit_uçlar[1] {
                kapsam[1] += genişleme / 2.0;
                kapsam[0] -= genişleme / 2.0;
            } else {
                kapsam[0] -= genişleme / 2.0;
            }
        } else {
            kapsam[1] = 1.0;
        }
    }
    if !geçerli_kapsam_sayısı(kapsam[0]) || !geçerli_kapsam_sayısı(kapsam[1]) {
        kapsam = [0.0, 1.0];
    }
    if kapsam[1] < kapsam[0] {
        kapsam.reverse();
    }
    kapsam
}

/// Sayısal değer ekseni ölçeği (`IntervalScale`).
#[derive(Clone, Debug)]
pub struct AralıkÖlçeği {
    pub kapsam: [f64; 2],
    pub adım: f64,
    pub adım_hassasiyeti: usize,
    güzel_kapsam: [f64; 2],
}

impl AralıkÖlçeği {
    /// Verilen bölme sayısına HİZALI ölçek kurar (`alignTicks`): adım,
    /// kapsamı tam `bölme` aralığa bölen güzel bir değere yükseltilir;
    /// böylece aynı ızgaradaki değer eksenlerinin bölme çizgileri üst üste
    /// düşer.
    pub fn hizalı_kur(
        veri_kapsamı: [f64; 2],
        sabit_en_az: Option<f64>,
        sabit_en_çok: Option<f64>,
        sıfırı_içer: bool,
        bölme: usize,
    ) -> Self {
        let mut kapsam = veri_kapsamı;
        if !geçerli_kapsam_sayısı(kapsam[0]) {
            kapsam[0] = 0.0;
        }
        if !geçerli_kapsam_sayısı(kapsam[1]) {
            kapsam[1] = 1.0;
        }
        if sıfırı_içer {
            kapsam[0] = kapsam[0].min(0.0);
            kapsam[1] = kapsam[1].max(0.0);
        }
        if let Some(ea) = sabit_en_az {
            kapsam[0] = ea;
        }
        if let Some(eç) = sabit_en_çok {
            kapsam[1] = eç;
        }
        let sabit_uçlar = [sabit_en_az.is_some(), sabit_en_çok.is_some()];
        let kapsam = kapsamı_geçerle(kapsam, sabit_uçlar);

        let bölme_f = bölme.max(1) as f64;
        let mut adım = güzel_sayı((kapsam[1] - kapsam[0]) / bölme_f, GüzelKip::Tavan);
        for _ in 0..12 {
            let alt = if sabit_en_az.is_some() {
                kapsam[0]
            } else {
                (kapsam[0] / adım).floor() * adım
            };
            let üst = alt + adım * bölme_f;
            if üst + adım * 1e-9 >= kapsam[1] || sabit_en_çok.is_some() {
                let h = adım_hassasiyeti(adım);
                let alt = yuvarla(alt, h);
                let üst = if sabit_en_çok.is_some() {
                    kapsam[1]
                } else {
                    yuvarla(üst, h)
                };
                return AralıkÖlçeği {
                    kapsam: [alt, üst],
                    adım,
                    adım_hassasiyeti: h,
                    güzel_kapsam: [alt, üst],
                };
            }
            // Sığmadı: bir üst güzel adıma çık.
            adım = güzel_sayı(adım * 1.6, GüzelKip::Tavan);
        }
        Self::kur(
            veri_kapsamı,
            sabit_en_az,
            sabit_en_çok,
            sıfırı_içer,
            bölme,
            None,
            None,
        )
    }

    /// Veri kapsamından ölçek kurar.
    ///
    /// * `sabit_en_az` / `sabit_en_çok` — eksen seçeneğindeki `min`/`max`.
    /// * `sıfırı_içer` — ECharts'taki `scale: false` davranışı: kapsam sıfırı
    ///   içerecek biçimde genişletilir.
    /// * `bölme_sayısı` — `splitNumber`, öntanımlı 5.
    pub fn kur(
        veri_kapsamı: [f64; 2],
        sabit_en_az: Option<f64>,
        sabit_en_çok: Option<f64>,
        sıfırı_içer: bool,
        bölme_sayısı: usize,
        en_küçük_adım: Option<f64>,
        en_büyük_adım: Option<f64>,
    ) -> Self {
        let mut kapsam = veri_kapsamı;
        if !geçerli_kapsam_sayısı(kapsam[0]) {
            kapsam[0] = 0.0;
        }
        if !geçerli_kapsam_sayısı(kapsam[1]) {
            kapsam[1] = 1.0;
        }
        if sıfırı_içer {
            kapsam[0] = kapsam[0].min(0.0);
            kapsam[1] = kapsam[1].max(0.0);
        }
        if let Some(ea) = sabit_en_az {
            kapsam[0] = ea;
        }
        if let Some(eç) = sabit_en_çok {
            kapsam[1] = eç;
        }
        let sabit_uçlar = [sabit_en_az.is_some(), sabit_en_çok.is_some()];
        let mut kapsam = kapsamı_geçerle(kapsam, sabit_uçlar);

        let sonuç = güzel_çentikler(kapsam, bölme_sayısı, en_küçük_adım, en_büyük_adım);

        // `Interval.ts` içindeki `calcNiceExtent`: sabitlenmemiş uçlar en
        // yakın adım katına genişletilir.
        if sabit_en_az.is_none() {
            kapsam[0] = yuvarla(
                (kapsam[0] / sonuç.adım).floor() * sonuç.adım,
                sonuç.adım_hassasiyeti,
            );
        }
        if sabit_en_çok.is_none() {
            kapsam[1] = yuvarla(
                (kapsam[1] / sonuç.adım).ceil() * sonuç.adım,
                sonuç.adım_hassasiyeti,
            );
        }

        // Kapsam değiştiği için güzel kapsamı yeniden kırp.
        let güzel_kapsam = [
            yuvarla(
                (kapsam[0] / sonuç.adım).ceil() * sonuç.adım,
                sonuç.adım_hassasiyeti,
            ),
            yuvarla(
                (kapsam[1] / sonuç.adım).floor() * sonuç.adım,
                sonuç.adım_hassasiyeti,
            ),
        ];

        AralıkÖlçeği {
            kapsam,
            adım: sonuç.adım,
            adım_hassasiyeti: sonuç.adım_hassasiyeti,
            güzel_kapsam,
        }
    }

    pub fn oranla(&self, değer: f64) -> f64 {
        doğrusal_eşle(değer, self.kapsam, [0.0, 1.0], true)
    }

    pub fn orandan(&self, oran: f64) -> f64 {
        doğrusal_eşle(oran, [0.0, 1.0], self.kapsam, true)
    }

    /// `Interval.ts` içindeki `getTicks` portu: güzel kapsamda adım adım
    /// ilerler; kapsam uçları güzel kapsamın dışındaysa uç çentik olarak
    /// eklenir.
    pub fn çentikler(&self) -> Vec<Çentik> {
        let mut sonuç = Vec::new();
        let adım = self.adım;
        if adım <= 0.0 {
            return sonuç;
        }
        let [gk0, gk1] = self.güzel_kapsam;

        if self.kapsam[0] < gk0 {
            sonuç.push(Çentik {
                değer: self.kapsam[0],
                kırılma: None,
            });
        }
        let mut değer = gk0;
        // Kayan nokta birikimini önlemek için yuvarlayarak ilerle.
        let güvenlik_sınırı = 10_000;
        let mut sayaç = 0;
        while değer <= gk1 + adım * 1e-6 && sayaç < güvenlik_sınırı {
            sonuç.push(Çentik {
                değer: yuvarla(değer, self.adım_hassasiyeti),
                kırılma: None,
            });
            değer += adım;
            sayaç += 1;
        }
        if let Some(son) = sonuç.last()
            && self.kapsam[1] > son.değer
        {
            sonuç.push(Çentik {
                değer: self.kapsam[1],
                kırılma: None,
            });
        }
        sonuç
    }

    pub fn etiket(&self, değer: f64) -> String {
        çentik_değeri_biçimle(değer, self.adım_hassasiyeti)
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
    fn tipik_kapsam() {
        let ö = AralıkÖlçeği::kur([0.0, 230.0], None, None, true, 5, None, None);
        assert_eq!(ö.kapsam, [0.0, 250.0]);
        assert_eq!(ö.adım, 50.0);
        let ç: Vec<f64> = ö.çentikler().iter().map(|t| t.değer).collect();
        assert_eq!(ç, vec![0.0, 50.0, 100.0, 150.0, 200.0, 250.0]);
    }

    #[test]
    fn eşit_uçlar() {
        let ö = AralıkÖlçeği::kur([5.0, 5.0], None, None, false, 5, None, None);
        assert!(ö.kapsam[0] < ö.kapsam[1]);
    }

    #[test]
    fn negatif_değerler() {
        let ö = AralıkÖlçeği::kur([-120.0, 80.0], None, None, true, 5, None, None);
        assert!(ö.kapsam[0] <= -120.0);
        assert!(ö.kapsam[1] >= 80.0);
    }
}
