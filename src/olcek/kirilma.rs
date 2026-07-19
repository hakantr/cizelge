//! ECharts 6 kırık ölçek eşlemesi (`src/scale/breakImpl.ts`).
//!
//! Ham veri aralıklarını eksen uzunluğundan düşürür, her kırılma için mutlak
//! ya da yüzdelik bir görsel boşluk bırakır ve dönüşümü iki yönde uygular.

use crate::model::eksen::{EksenKırılmaBoşluğu, EksenKırılması};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ÇözülmüşEksenKırılması {
    pub başlangıç: f64,
    pub bitiş: f64,
    pub gerçek_boşluk: f64,
}

#[derive(Clone, Copy)]
struct HazırKırılma {
    başlangıç: f64,
    bitiş: f64,
    boşluk: EksenKırılmaBoşluğu,
    gerçek_boşluk: f64,
}

/// Kırılmalar uygulanmış doğrusal ölçek katmanı.
#[derive(Clone)]
pub struct KırılmaEşleyici {
    kapsam: [f64; 2],
    kırılmalar: Vec<HazırKırılma>,
}

impl std::fmt::Debug for KırılmaEşleyici {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KırılmaEşleyici")
            .field("kapsam", &self.kapsam)
            .field("kırılmalar", &self.kırılmalar())
            .finish()
    }
}

impl KırılmaEşleyici {
    pub fn kur(seçenekler: &[EksenKırılması], kapsam: [f64; 2]) -> Option<Self> {
        let mut kapsam = kapsam;
        if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
            return None;
        }
        if kapsam[1] < kapsam[0] {
            kapsam.reverse();
        }
        if kapsam[1] <= kapsam[0] {
            return None;
        }

        let mut kırılmalar = seçenekler
            .iter()
            .filter(|kırılma| !kırılma.genişletilmiş)
            .filter_map(|kırılma| {
                let mut başlangıç = kırılma.başlangıç;
                let mut bitiş = kırılma.bitiş;
                if !başlangıç.is_finite() || !bitiş.is_finite() {
                    return None;
                }
                if bitiş < başlangıç {
                    std::mem::swap(&mut başlangıç, &mut bitiş);
                }
                let boşluk = match kırılma.boşluk {
                    EksenKırılmaBoşluğu::Değer(değer) if değer.is_finite() && değer >= 0.0 => {
                        EksenKırılmaBoşluğu::Değer(değer)
                    }
                    EksenKırılmaBoşluğu::Yüzde(oran)
                        if oran.is_finite() && (0.0..(1.0 - 1e-5)).contains(&oran) =>
                    {
                        EksenKırılmaBoşluğu::Yüzde(oran)
                    }
                    _ => EksenKırılmaBoşluğu::Değer(0.0),
                };
                Some(HazırKırılma {
                    başlangıç,
                    bitiş,
                    boşluk: if başlangıç == bitiş {
                        EksenKırılmaBoşluğu::Değer(0.0)
                    } else {
                        boşluk
                    },
                    gerçek_boşluk: 0.0,
                })
            })
            .collect::<Vec<_>>();
        kırılmalar.sort_by(|a, b| {
            a.başlangıç
                .partial_cmp(&b.başlangıç)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        // ECharts çakışan sonraki kırılmayı tanıyla birlikte atar. Model
        // doğrulaması kullanıcıya hatayı bildirir; çalışma hattı yine de
        // paniklememek için aynı güvenli davranışı korur.
        let mut son = f64::NEG_INFINITY;
        kırılmalar.retain(|kırılma| {
            let geçerli = son <= kırılma.başlangıç;
            son = son.max(kırılma.bitiş);
            geçerli
        });
        if kırılmalar.is_empty() {
            return None;
        }

        kırılma_boşluklarını_çöz(&mut kırılmalar, kapsam);
        Some(Self {
            kapsam, kırılmalar
        })
    }

    pub fn kapsam(&self) -> [f64; 2] {
        self.kapsam
    }

    pub fn kırılmalar(&self) -> Vec<ÇözülmüşEksenKırılması> {
        self.kırılmalar
            .iter()
            .map(|kırılma| ÇözülmüşEksenKırılması {
                başlangıç: kırılma.başlangıç,
                bitiş: kırılma.bitiş,
                gerçek_boşluk: kırılma.gerçek_boşluk,
            })
            .collect()
    }

    /// Etkin kapsamla kesişen kırılma uçları. Görsel alan ve kırılma
    /// etiketleri yalnız bu kırpılmış çiftleri kullanır.
    pub fn görünür_kırılmalar(&self) -> Vec<ÇözülmüşEksenKırılması> {
        self.kırılmalar
            .iter()
            .filter_map(|kırılma| {
                let başlangıç = kırılma.başlangıç.max(self.kapsam[0]);
                let bitiş = kırılma.bitiş.min(self.kapsam[1]);
                (başlangıç < bitiş
                    || (başlangıç == bitiş
                        && başlangıç > self.kapsam[0]
                        && başlangıç < self.kapsam[1]))
                    .then_some(ÇözülmüşEksenKırılması {
                        başlangıç,
                        bitiş,
                        gerçek_boşluk: kırılma.gerçek_boşluk,
                    })
            })
            .collect()
    }

    /// Ham değeri kırılmaların süreleri düşülmüş doğrusal uzaya taşır.
    pub fn içe(&self, değer: f64) -> f64 {
        let mut geçen = 0.0;
        let mut son_kırılma_bitişi = 0.0;
        for kırılma in &self.kırılmalar {
            if değer <= kırılma.bitiş {
                if değer > kırılma.başlangıç {
                    let açıklık = kırılma.bitiş - kırılma.başlangıç;
                    geçen += kırılma.başlangıç - son_kırılma_bitişi
                        + if açıklık > 0.0 {
                            (değer - kırılma.başlangıç) / açıklık * kırılma.gerçek_boşluk
                        } else {
                            0.0
                        };
                } else {
                    geçen += değer - son_kırılma_bitişi;
                }
                return geçen;
            }
            geçen += kırılma.başlangıç - son_kırılma_bitişi + kırılma.gerçek_boşluk;
            son_kırılma_bitişi = kırılma.bitiş;
        }
        geçen + değer - son_kırılma_bitişi
    }

    /// Kırılmalar uygulanmış doğrusal uzayı yeniden ham veriye açar.
    pub fn dışa(&self, geçen: f64) -> f64 {
        let mut son_geçen_bitiş = 0.0;
        let mut son_kırılma_bitişi = 0.0;
        for kırılma in &self.kırılmalar {
            let geçen_başlangıç = son_geçen_bitiş + kırılma.başlangıç - son_kırılma_bitişi;
            let geçen_bitiş = geçen_başlangıç + kırılma.gerçek_boşluk;
            if geçen <= geçen_bitiş {
                if geçen > geçen_başlangıç && kırılma.gerçek_boşluk > 0.0 {
                    return kırılma.başlangıç
                        + (geçen - geçen_başlangıç) / kırılma.gerçek_boşluk
                            * (kırılma.bitiş - kırılma.başlangıç);
                }
                return son_kırılma_bitişi + geçen - son_geçen_bitiş;
            }
            son_geçen_bitiş = geçen_bitiş;
            son_kırılma_bitişi = kırılma.bitiş;
        }
        son_kırılma_bitişi + geçen - son_geçen_bitiş
    }

    pub fn etkin_açıklık(&self) -> f64 {
        (self.içe(self.kapsam[1]) - self.içe(self.kapsam[0])).max(0.0)
    }
}

fn kırılma_boşluklarını_çöz(kırılmalar: &mut [HazırKırılma], kapsam: [f64; 2]) {
    let yüzde_toplamı = kırılmalar
        .iter()
        .filter_map(|kırılma| match kırılma.boşluk {
            EksenKırılmaBoşluğu::Yüzde(oran) => Some(oran),
            EksenKırılmaBoşluğu::Değer(_) => None,
        })
        .sum::<f64>();

    let mut pay_düzeltmesi = 0.0;
    let mut yüzde_payda_düzeltmesi = 0.0;
    for kırılma in kırılmalar.iter() {
        let başlangıç = kırılma.başlangıç.max(kapsam[0]);
        let bitiş = kırılma.bitiş.min(kapsam[1]);
        if bitiş < başlangıç {
            continue;
        }
        let başlangıç_kırpıldı = başlangıç != kırılma.başlangıç;
        let bitiş_kırpıldı = bitiş != kırılma.bitiş;
        // Kırılma kapsamın iki ucunu birden aşıyorsa resmi uygulama pay/payda
        // düzeltmesine katmaz; tüm görünür alan kırılma boşluğuna dönüşür.
        if başlangıç_kırpıldı && bitiş_kırpıldı {
            continue;
        }
        let kırpılmış_açıklık = (bitiş - başlangıç).max(0.0);
        let oran = if başlangıç_kırpıldı || bitiş_kırpıldı {
            kırpılmış_açıklık / (kırılma.bitiş - kırılma.başlangıç).max(f64::EPSILON)
        } else {
            1.0
        };
        match kırılma.boşluk {
            EksenKırılmaBoşluğu::Değer(boşluk) => {
                pay_düzeltmesi += (boşluk - kırpılmış_açıklık) * oran;
            }
            EksenKırılmaBoşluğu::Yüzde(yüzde) => {
                pay_düzeltmesi -= kırpılmış_açıklık * oran;
                yüzde_payda_düzeltmesi += yüzde * oran;
            }
        }
    }

    let payda = 1.0 - yüzde_payda_düzeltmesi;
    let yüzde_gerçek_toplamı = if yüzde_toplamı > 0.0 && payda > 1e-12 {
        (yüzde_toplamı * ((kapsam[1] - kapsam[0]) + pay_düzeltmesi) / payda).max(0.0)
    } else {
        0.0
    };
    for kırılma in kırılmalar {
        kırılma.gerçek_boşluk = match kırılma.boşluk {
            EksenKırılmaBoşluğu::Değer(değer) => değer,
            EksenKırılmaBoşluğu::Yüzde(yüzde) if yüzde_toplamı > 0.0 => {
                yüzde_gerçek_toplamı * yüzde / yüzde_toplamı
            }
            EksenKırılmaBoşluğu::Yüzde(_) => 0.0,
        };
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
mod testler {
    use super::*;

    #[test]
    fn mutlak_bosluk_resmi_elapse_ornegini_izler() {
        let eşleyici = KırılmaEşleyici::kur(
            &[
                EksenKırılması::yeni(-400.0, -300.0).boşluk(27.0),
                EksenKırılması::yeni(-100.0, 100.0).boşluk(10.0),
                EksenKırılması::yeni(200.0, 400.0).boşluk(300.0),
            ],
            [-500.0, 500.0],
        )
        .unwrap();
        for (ham, geçen) in [
            (-400.0, -400.0),
            (-300.0, -373.0),
            (-100.0, -173.0),
            (100.0, -163.0),
            (200.0, -63.0),
            (400.0, 237.0),
        ] {
            assert!((eşleyici.içe(ham) - geçen).abs() < 1e-9);
            assert!((eşleyici.dışa(geçen) - ham).abs() < 1e-9);
        }
    }

    #[test]
    fn yuzdelik_bosluk_son_etkin_acikligin_oranidir() {
        let eşleyici = KırılmaEşleyici::kur(
            &[EksenKırılması::yeni(20.0, 40.0).boşluk("10%")],
            [0.0, 100.0],
        )
        .unwrap();
        let kırılma = eşleyici.kırılmalar()[0];
        assert!((kırılma.gerçek_boşluk / eşleyici.etkin_açıklık() - 0.1).abs() < 1e-12);
    }

    #[test]
    fn bosluk_icinde_iki_yonlu_esleme_tersinirdir() {
        let eşleyici = KırılmaEşleyici::kur(
            &[EksenKırılması::yeni(20.0, 40.0).boşluk(5.0)],
            [0.0, 100.0],
        )
        .unwrap();
        for ham in [0.0, 10.0, 20.0, 25.0, 40.0, 70.0, 100.0] {
            assert!((eşleyici.dışa(eşleyici.içe(ham)) - ham).abs() < 1e-10);
        }
    }
}
