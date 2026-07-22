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
        if hedef_adım > 7.5 * GÜN && hedef_adım < 31.0 * GÜN {
            // ECharts `getDateInterval`: ay seviyesine geçmeden önce günlük
            // alt seviyeyi 7/16 günlük aralıkla korur. Leveled time axis bu
            // sayede kısa çok-haftalı kapsamlarda ay sınırlarıyla birlikte
            // haftalık etiketleri de üretir.
            birim = ZamanBirimi::Gün;
            birim_adımı = if hedef_adım > 16.0 * GÜN { 16.0 } else { 7.0 };
        } else if (31.0 * GÜN..=YIL_YAKLAŞIK).contains(&hedef_adım) {
            // ECharts `TimeScale`, yaklaşık aralık ay düzeyine ulaştığında
            // bir üst sabit milisaniye adayını doğrudan kullanmaz;
            // `getMonthInterval` ile takvim ayı adımını 1/2/3/6 seçer. Bu
            // özellikle 2–4 yıllık eksenlerde nice interval yarım yıl olsa
            // bile çentiklerin çeyrek yıllık üretilmesini sağlar.
            birim = ZamanBirimi::Ay;
            let yaklaşık_ay = hedef_adım / (30.0 * GÜN);
            birim_adımı = if yaklaşık_ay > 6.0 {
                6.0
            } else if yaklaşık_ay > 3.0 {
                3.0
            } else if yaklaşık_ay > 2.0 {
                2.0
            } else {
                1.0
            };
        } else if hedef_adım >= SANİYE {
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
        doğrusal_eşle(değer, self.kapsam, [0.0, 1.0], false)
    }

    pub fn orandan(&self, oran: f64) -> f64 {
        doğrusal_eşle(oran, [0.0, 1.0], self.kapsam, false)
    }

    pub fn çentikler(&self) -> Vec<Çentik> {
        if self.birim == ZamanBirimi::Gün && (self.birim_adımı - 7.0).abs() < f64::EPSILON {
            return haftalık_kademeli_çentikler(self.kapsam);
        }
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
            ZamanBirimi::Gün => {
                if t.gün == 1 {
                    ay_adı.to_string()
                } else {
                    t.gün.to_string()
                }
            }
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

/// ECharts `createIntervalTicks`in week → month iki seviyeli kısa kapsamı.
/// Her ayın 1'i üst seviye, 8/15/22/29'u alt seviye olur; önceki ayın son
/// haftası yeni aya taşıyorsa ilk taşan gün de korunur. ECharts ham kapsam
/// uçlarını `notNice` olarak üretip öntanımlı `showMinLabel/showMaxLabel`
/// çözümünde etiketi ve ona bağlı ana çentiği gizlediğinden, görünür çentik
/// listesine yalnız nice seviyeler alınır.
fn haftalık_kademeli_çentikler(kapsam: [f64; 2]) -> Vec<Çentik> {
    let başlangıç = andan_takvime(kapsam[0]);
    let bitiş = andan_takvime(kapsam[1]);
    let mut yıl = başlangıç.yıl;
    let mut ay = başlangıç.ay;
    let mut ilk_ay = true;
    let mut sonuç = Vec::new();
    loop {
        let ay_başı = takvimden_ana(TakvimAnı {
            yıl,
            ay,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        if ay_başı > kapsam[1] {
            break;
        }
        if ay_başı >= kapsam[0] {
            sonuç.push(Çentik {
                değer: ay_başı,
                kırılma: None,
            });
        }

        if !ilk_ay {
            let (önceki_yıl, önceki_ay) = if ay == 1 {
                (yıl - 1, 12)
            } else {
                (yıl, ay - 1)
            };
            let önceki_gün_sayısı = aydaki_gün_sayısı(önceki_yıl, önceki_ay);
            let önceki_son_hafta = 1 + ((önceki_gün_sayısı - 1) / 7) * 7;
            let taşan_gün = önceki_son_hafta + 7 - önceki_gün_sayısı;
            if taşan_gün > 1 {
                let değer = takvimden_ana(TakvimAnı {
                    yıl,
                    ay,
                    gün: taşan_gün,
                    saat: 0,
                    dakika: 0,
                    saniye: 0,
                    milisaniye: 0,
                });
                if değer >= kapsam[0] && değer <= kapsam[1] {
                    sonuç.push(Çentik {
                        değer,
                        kırılma: None,
                    });
                }
            }
        }

        let gün_sayısı = aydaki_gün_sayısı(yıl, ay);
        for gün in (8..=gün_sayısı).step_by(7) {
            let değer = takvimden_ana(TakvimAnı {
                yıl,
                ay,
                gün,
                saat: 0,
                dakika: 0,
                saniye: 0,
                milisaniye: 0,
            });
            if değer >= kapsam[0] && değer <= kapsam[1] {
                sonuç.push(Çentik {
                    değer,
                    kırılma: None,
                });
            }
        }

        if yıl == bitiş.yıl && ay == bitiş.ay {
            break;
        }
        ay += 1;
        if ay > 12 {
            ay = 1;
            yıl += 1;
        }
        ilk_ay = false;
    }
    sonuç.sort_by(|a, b| a.değer.total_cmp(&b.değer));
    sonuç.dedup_by(|a, b| (a.değer - b.değer).abs() <= f64::EPSILON);
    sonuç
}

fn aydaki_gün_sayısı(yıl: i32, ay: u32) -> u32 {
    match ay {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if yıl % 400 == 0 || (yıl % 4 == 0 && yıl % 100 != 0) => 29,
        2 => 28,
        _ => 30,
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

    #[test]
    fn cok_yillik_kapsam_echarts_gibi_ceyrek_yil_centikleri_uretir() {
        let başlangıç = takvimden_ana(TakvimAnı {
            yıl: 1997,
            ay: 10,
            gün: 4,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let ölçek = ZamanÖlçeği::kur([başlangıç, başlangıç + 999.0 * GÜN], 6);

        assert_eq!(ölçek.birim, ZamanBirimi::Ay);
        assert_eq!(ölçek.birim_adımı, 3.0);
        let tarihler = ölçek
            .çentikler()
            .into_iter()
            .map(|çentik| {
                let an = andan_takvime(çentik.değer);
                (an.yıl, an.ay, an.gün)
            })
            .collect::<Vec<_>>();
        assert_eq!(tarihler.first(), Some(&(1998, 1, 1)));
        assert!(tarihler.contains(&(1998, 4, 1)));
        assert!(tarihler.contains(&(1999, 10, 1)));
        assert_eq!(tarihler.last(), Some(&(2000, 4, 1)));
    }

    #[test]
    fn iki_aylik_kapsam_hafta_ve_ay_seviyelerini_birlikte_uretir() {
        let başlangıç = takvimden_ana(TakvimAnı {
            yıl: 2025,
            ay: 5,
            gün: 5,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let bitiş = takvimden_ana(TakvimAnı {
            yıl: 2025,
            ay: 7,
            gün: 7,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let ölçek = ZamanÖlçeği::kur([başlangıç, bitiş], 6);

        assert_eq!(ölçek.birim, ZamanBirimi::Gün);
        assert_eq!(ölçek.birim_adımı, 7.0);
        let tarihler = ölçek
            .çentikler()
            .into_iter()
            .map(|çentik| {
                let an = andan_takvime(çentik.değer);
                (an.ay, an.gün)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            tarihler,
            vec![
                (5, 8),
                (5, 15),
                (5, 22),
                (5, 29),
                (6, 1),
                (6, 5),
                (6, 8),
                (6, 15),
                (6, 22),
                (6, 29),
                (7, 1),
                (7, 6),
            ]
        );
    }
}
