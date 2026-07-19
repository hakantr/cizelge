//! Zaman ölçeği — `echarts/src/scale/Time.ts` uyarlaması.
//!
//! Değerler Unix milisaniyesidir. Açıklığa göre insan-dostu bir birim
//! (saniye, dakika, saat, gün, ay, yıl…) seçilir ve çentikler o birimin
//! sınırlarına hizalanır.

use crate::olcek::Çentik;
use crate::yardimci::sayi::doğrusal_eşle;
use crate::yardimci::takvim::{TakvimAnı, andan_takvime, takvimden_ana};
use crate::yerel::ay_kısaltması;

const SANİYE: f64 = 1000.0;
const DAKİKA: f64 = 60.0 * SANİYE;
const SAAT: f64 = 60.0 * DAKİKA;
const GÜN: f64 = 24.0 * SAAT;
const YIL_YAKLAŞIK: f64 = 365.25 * GÜN;

/// Çentik üretiminde kullanılan zaman birimi.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZamanBirimi {
    Milisaniye,
    Saniye,
    Dakika,
    Saat,
    Gün,
    Ay,
    Yıl,
}

/// Zaman ekseni ölçeği (`TimeScale`).
#[derive(Clone, Debug)]
pub struct ZamanÖlçeği {
    pub kapsam: [f64; 2],
    pub birim: ZamanBirimi,
    /// Birim cinsinden adım (örn. birim=Dakika, adım=15 → çeyrek saat).
    pub birim_adımı: f64,
    /// Nice tick seçiminin kırılmalar düşüldükten sonraki yaklaşık aralığı.
    pub yaklaşık_aralık: f64,
}

impl ZamanÖlçeği {
    pub fn kur(veri_kapsamı: [f64; 2], bölme_sayısı: usize) -> Self {
        let etkin_açıklık = (veri_kapsamı[1] - veri_kapsamı[0]).abs();
        Self::kur_etkin_açıklıkla(veri_kapsamı, bölme_sayısı, etkin_açıklık)
    }

    /// Kırık eksende ham kapsamı koruyup nice tick adımını sıkıştırılmış
    /// açıklıktan seçer (`getScaleLinearSpanEffective`).
    pub fn kur_etkin_açıklıkla(
        veri_kapsamı: [f64; 2],
        bölme_sayısı: usize,
        etkin_açıklık: f64,
    ) -> Self {
        let mut kapsam = veri_kapsamı;
        if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
            kapsam = [0.0, GÜN];
        }
        if kapsam[0] == kapsam[1] {
            kapsam[1] = kapsam[0] + GÜN;
        }
        if kapsam[1] < kapsam[0] {
            kapsam.reverse();
        }
        let açıklık = kapsam[1] - kapsam[0];
        let etkin_açıklık = if etkin_açıklık.is_finite() && etkin_açıklık > 0.0 {
            etkin_açıklık
        } else {
            açıklık
        };
        let hedef_adım = etkin_açıklık / bölme_sayısı.max(1) as f64;

        // Aday adımlar: (birim, birim adımı, milisaniye karşılığı).
        let adaylar: [(ZamanBirimi, f64, f64); 19] = [
            (ZamanBirimi::Saniye, 1.0, SANİYE),
            (ZamanBirimi::Saniye, 5.0, 5.0 * SANİYE),
            (ZamanBirimi::Saniye, 15.0, 15.0 * SANİYE),
            (ZamanBirimi::Saniye, 30.0, 30.0 * SANİYE),
            (ZamanBirimi::Dakika, 1.0, DAKİKA),
            (ZamanBirimi::Dakika, 5.0, 5.0 * DAKİKA),
            (ZamanBirimi::Dakika, 15.0, 15.0 * DAKİKA),
            (ZamanBirimi::Dakika, 30.0, 30.0 * DAKİKA),
            (ZamanBirimi::Saat, 1.0, SAAT),
            (ZamanBirimi::Saat, 3.0, 3.0 * SAAT),
            (ZamanBirimi::Saat, 6.0, 6.0 * SAAT),
            (ZamanBirimi::Saat, 12.0, 12.0 * SAAT),
            (ZamanBirimi::Gün, 1.0, GÜN),
            (ZamanBirimi::Gün, 2.0, 2.0 * GÜN),
            (ZamanBirimi::Gün, 7.0, 7.0 * GÜN),
            (ZamanBirimi::Ay, 1.0, YIL_YAKLAŞIK / 12.0),
            (ZamanBirimi::Ay, 3.0, YIL_YAKLAŞIK / 4.0),
            (ZamanBirimi::Ay, 6.0, YIL_YAKLAŞIK / 2.0),
            (ZamanBirimi::Yıl, 1.0, YIL_YAKLAŞIK),
        ];

        let mut birim = ZamanBirimi::Milisaniye;
        let mut birim_adımı = hedef_adım.max(1.0);
        if hedef_adım >= SANİYE {
            let mut seçildi = false;
            for (b, ba, ms) in adaylar {
                if ms >= hedef_adım {
                    birim = b;
                    birim_adımı = ba;
                    seçildi = true;
                    break;
                }
            }
            if !seçildi {
                // Çok yıllık: yıl adımını güzel sayıya çek.
                birim = ZamanBirimi::Yıl;
                let yıl_adımı = (hedef_adım / YIL_YAKLAŞIK).ceil();
                birim_adımı = güzel_yıl_adımı(yıl_adımı);
            }
        }

        ZamanÖlçeği {
            kapsam,
            birim,
            birim_adımı,
            yaklaşık_aralık: hedef_adım,
        }
    }

    pub fn oranla(&self, değer: f64) -> f64 {
        doğrusal_eşle(değer, self.kapsam, [0.0, 1.0], true)
    }

    pub fn orandan(&self, oran: f64) -> f64 {
        doğrusal_eşle(oran, [0.0, 1.0], self.kapsam, true)
    }

    pub fn çentikler(&self) -> Vec<Çentik> {
        let mut sonuç = Vec::new();
        let güvenlik_sınırı = 1000;
        match self.birim {
            ZamanBirimi::Yıl => {
                let ilk = andan_takvime(self.kapsam[0]);
                let adım = self.birim_adımı.max(1.0) as i32;
                let mut yıl = (ilk.yıl as f64 / adım as f64).ceil() as i32 * adım;
                let mut sayaç = 0;
                loop {
                    let t = takvimden_ana(TakvimAnı {
                        yıl,
                        ay: 1,
                        gün: 1,
                        saat: 0,
                        dakika: 0,
                        saniye: 0,
                        milisaniye: 0,
                    });
                    if t > self.kapsam[1] || sayaç >= güvenlik_sınırı {
                        break;
                    }
                    if t >= self.kapsam[0] {
                        sonuç.push(Çentik {
                            değer: t,
                            kırılma: None,
                        });
                    }
                    yıl += adım;
                    sayaç += 1;
                }
            }
            ZamanBirimi::Ay => {
                let ilk = andan_takvime(self.kapsam[0]);
                let adım = self.birim_adımı.max(1.0) as u32;
                let mut yıl = ilk.yıl;
                // Ay dizinini adım katına hizala (0 tabanlı).
                let mut ay0 = ilk.ay.saturating_sub(1);
                ay0 = ay0.div_ceil(adım) * adım;
                let mut sayaç = 0;
                loop {
                    yıl += (ay0 / 12) as i32;
                    ay0 %= 12;
                    let t = takvimden_ana(TakvimAnı {
                        yıl,
                        ay: ay0 + 1,
                        gün: 1,
                        saat: 0,
                        dakika: 0,
                        saniye: 0,
                        milisaniye: 0,
                    });
                    if t > self.kapsam[1] || sayaç >= güvenlik_sınırı {
                        break;
                    }
                    if t >= self.kapsam[0] {
                        sonuç.push(Çentik {
                            değer: t,
                            kırılma: None,
                        });
                    }
                    ay0 += adım;
                    sayaç += 1;
                }
            }
            _ => {
                let adım_ms = match self.birim {
                    ZamanBirimi::Saniye => self.birim_adımı * SANİYE,
                    ZamanBirimi::Dakika => self.birim_adımı * DAKİKA,
                    ZamanBirimi::Saat => self.birim_adımı * SAAT,
                    ZamanBirimi::Gün => self.birim_adımı * GÜN,
                    // Milisaniye ve (yukarıda ele alınan) diğer birimler.
                    _ => self.birim_adımı,
                };
                let adım_ms = if adım_ms > 0.0 { adım_ms } else { 1.0 };
                let mut t = (self.kapsam[0] / adım_ms).ceil() * adım_ms;
                let mut sayaç = 0;
                while t <= self.kapsam[1] && sayaç < güvenlik_sınırı {
                    sonuç.push(Çentik {
                        değer: t,
                        kırılma: None,
                    });
                    t += adım_ms;
                    sayaç += 1;
                }
            }
        }
        sonuç
    }

    /// Etiket biçimi birime göre seçilir (`Time.ts` içindeki kademeli
    /// `formatter` yaklaşımının sade karşılığı).
    pub fn etiket(&self, değer: f64) -> String {
        let t = andan_takvime(değer);
        let ay_adı = ay_kısaltması(t.ay);
        match self.birim {
            ZamanBirimi::Yıl => format!("{}", t.yıl),
            ZamanBirimi::Ay => {
                if t.ay == 1 {
                    format!("{}", t.yıl)
                } else {
                    ay_adı.to_string()
                }
            }
            ZamanBirimi::Gün => format!("{} {}", t.gün, ay_adı),
            ZamanBirimi::Saat | ZamanBirimi::Dakika => {
                if t.saat == 0 && t.dakika == 0 {
                    format!("{} {}", t.gün, ay_adı)
                } else {
                    format!("{:02}:{:02}", t.saat, t.dakika)
                }
            }
            ZamanBirimi::Saniye => format!("{:02}:{:02}:{:02}", t.saat, t.dakika, t.saniye),
            ZamanBirimi::Milisaniye => format!(
                "{:02}:{:02}:{:02}.{:03}",
                t.saat, t.dakika, t.saniye, t.milisaniye
            ),
        }
    }
}

fn güzel_yıl_adımı(ham: f64) -> f64 {
    let adaylar = [
        1.0, 2.0, 5.0, 10.0, 20.0, 25.0, 50.0, 100.0, 200.0, 500.0, 1000.0,
    ];
    for a in adaylar {
        if a >= ham {
            return a;
        }
    }
    ham
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
    fn günlük_çentikler() {
        // 2026-01-01 .. 2026-01-08 (UTC)
        let başlangıç = takvimden_ana(TakvimAnı {
            yıl: 2026,
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let bitiş = başlangıç + 7.0 * GÜN;
        let ö = ZamanÖlçeği::kur([başlangıç, bitiş], 7);
        assert_eq!(ö.birim, ZamanBirimi::Gün);
        assert!(!ö.çentikler().is_empty());
    }
}
