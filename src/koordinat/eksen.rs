//! Çalışma zamanı ekseni — `echarts/src/coord/Axis.ts` karşılığı.
//!
//! Bir ölçeği piksel aralığına bağlar; kategori eksenlerinde bant (aralıklı
//! yerleşim) hesabını üstlenir.

use crate::model::eksen::{Eksen, EksenKonumu, EksenKırılmaBilgisi, EksenKırılmaUcu};
use crate::model::yakinlastirma::YakınlaştırmaSüzmeKipi;
use crate::olcek::{KırılmaEşleyici, Çentik, ÇözülmüşEksenKırılması, Ölçek};
use crate::yardimci::sayi::doğrusal_eşle;

/// Ölçek + piksel aralığı: veriyi ekran koordinatına eşleyen eksen.
#[derive(Clone, Debug)]
pub struct ÇalışmaEkseni {
    pub seçenek: Eksen,
    pub ölçek: Ölçek,
    /// Piksel aralığı `[baş, son]` (dikey eksenlerde baş alttadır).
    pub piksel: [f32; 2],
    /// Kategori ekseninde bant yerleşimi (`boundaryGap: true`).
    pub bantlı: bool,
    pub konum: EksenKonumu,
    /// Veri yakınlaştırma penceresi: görünür değer aralığı (kategorik
    /// eksenlerde sıra uzayında). Sayısal eksende bu alan, pencere dışı
    /// noktaların kenara sıkıştırılmadan ızgara dışında hesaplanmasını ve
    /// seri kırpmasının devreye girmesini sağlar. `None` = tam kapsam.
    pub pencere: Option<(f64, f64)>,
    /// Pencerenin ham eksen kapsamındaki `0..=1` oranları. `startValue` /
    /// `endValue` kullanıldığında sürgü tutamaçlarının yüzde seçeneklerinden
    /// değil çözülmüş değer penceresinden çizilmesini sağlar.
    pub yakınlaştırma_oranları: Option<(f32, f32)>,
    /// Pencerenin seri verisine uygulanma biçimi (`dataZoom.filterMode`).
    /// Pencere yokken bu değer etkisizdir.
    pub yakınlaştırma_süzme_kipi: YakınlaştırmaSüzmeKipi,
    /// Etkin kapsam için çözülmüş ECharts 6 kırık ölçek katmanı.
    pub kırılma_eşleyici: Option<KırılmaEşleyici>,
}

impl ÇalışmaEkseni {
    pub fn yeni(seçenek: Eksen, ölçek: Ölçek, piksel: [f32; 2], konum: EksenKonumu) -> Self {
        let bantlı = seçenek.bantlı_mı() && ölçek.kategorik_mi();
        let kırılma_eşleyici = KırılmaEşleyici::kur(&seçenek.kırılmalar, ölçek.kapsam());
        ÇalışmaEkseni {
            seçenek,
            ölçek,
            piksel,
            bantlı,
            konum,
            pencere: None,
            yakınlaştırma_oranları: None,
            yakınlaştırma_süzme_kipi: YakınlaştırmaSüzmeKipi::Yok,
            kırılma_eşleyici,
        }
    }

    /// Yakınlaştırma penceresini oranlarla (0..=1) uygular; yalnız kategorik
    /// eksenlerde kullanılır, sayısal eksenlerde kapsam kurulurken daraltılır.
    pub fn pencere_uygula(&mut self, başlangıç: f32, bitiş: f32) {
        let kapsam = self.ölçek.kapsam();
        let açıklık = kapsam[1] - kapsam[0];
        if açıklık <= 0.0 {
            return;
        }
        let mut p0 = kapsam[0] + açıklık * başlangıç.clamp(0.0, 1.0) as f64;
        let mut p1 = kapsam[0] + açıklık * bitiş.clamp(0.0, 1.0) as f64;
        // Ordinal ölçek pencere uçlarını en yakın tam kategori indisine
        // oturtur; yüzdelik uçların kesir izi bant aralığına taşınmaz.
        if self.bantlı {
            p0 = p0.round();
            p1 = p1.round();
        }
        self.değer_penceresi_uygula(p0, p1);
    }

    /// Çözülmüş veri değerleriyle yakınlaştırma penceresi uygular.
    pub fn değer_penceresi_uygula(&mut self, başlangıç: f64, bitiş: f64) {
        if başlangıç.is_finite() && bitiş.is_finite() && bitiş > başlangıç {
            self.pencere = Some((başlangıç, bitiş));
            self.kırılma_eşleyici =
                KırılmaEşleyici::kur(&self.seçenek.kırılmalar, [başlangıç, bitiş]);
        }
    }

    /// Piksel aralığı, `ters` seçeneği uygulanmış haliyle.
    fn etkin_piksel(&self) -> [f64; 2] {
        if self.seçenek.ters {
            [self.piksel[1] as f64, self.piksel[0] as f64]
        } else {
            [self.piksel[0] as f64, self.piksel[1] as f64]
        }
    }

    /// Veri değerini piksele eşler (`Axis#dataToCoord`).
    pub fn veriden_piksele(&self, değer: f64) -> f32 {
        let oran = if !self.bantlı
            && let Some(eşleyici) = &self.kırılma_eşleyici
        {
            let kapsam = eşleyici.kapsam();
            let başlangıç = eşleyici.içe(kapsam[0]);
            let bitiş = eşleyici.içe(kapsam[1]);
            (eşleyici.içe(değer) - başlangıç) / (bitiş - başlangıç).max(1e-12)
        } else {
            match (self.pencere, self.bantlı) {
                (Some((p0, p1)), true) => (değer - p0 + 0.5) / (p1 - p0 + 1.0),
                (Some((p0, p1)), false) => (değer - p0) / (p1 - p0).max(1e-12),
                (None, true) => {
                    let n = self.ölçek.kategori_sayısı().max(1) as f64;
                    (değer + 0.5) / n
                }
                (None, false) => self.ölçek.oranla(değer),
            }
        };
        // Pencere dışı değerler ızgara dışına taşar; çizim kırpılır.
        doğrusal_eşle(
            oran,
            [0.0, 1.0],
            self.etkin_piksel(),
            self.pencere.is_none() && self.ölçek.kategorik_mi(),
        ) as f32
    }

    /// Pikseli veri değerine eşler (`Axis#coordToData`).
    pub fn pikselden_veriye(&self, piksel: f32) -> f64 {
        let oran = doğrusal_eşle(piksel as f64, self.etkin_piksel(), [0.0, 1.0], true);
        if !self.bantlı
            && let Some(eşleyici) = &self.kırılma_eşleyici
        {
            let kapsam = eşleyici.kapsam();
            let başlangıç = eşleyici.içe(kapsam[0]);
            let bitiş = eşleyici.içe(kapsam[1]);
            return eşleyici.dışa(başlangıç + oran * (bitiş - başlangıç));
        }
        match (self.pencere, self.bantlı) {
            (Some((p0, p1)), true) => {
                let n = self.ölçek.kategori_sayısı().max(1) as f64;
                (p0 + oran * (p1 - p0 + 1.0) - 0.5)
                    .round()
                    .clamp(0.0, n - 1.0)
            }
            (Some((p0, p1)), false) => p0 + oran * (p1 - p0),
            (None, true) => {
                let n = self.ölçek.kategori_sayısı().max(1) as f64;
                (oran * n - 0.5).round().clamp(0.0, n - 1.0)
            }
            (None, false) => self.ölçek.orandan(oran),
        }
    }

    /// Bant genişliği, piksel (`Axis#getBandWidth`).
    pub fn bant_genişliği(&self) -> f32 {
        let uzunluk = (self.piksel[1] - self.piksel[0]).abs();
        if self.bantlı {
            let n = match self.pencere {
                Some((p0, p1)) => ((p1 - p0 + 1.0) as f32).max(1.0),
                None => self.ölçek.kategori_sayısı().max(1) as f32,
            };
            uzunluk / n
        } else if self.ölçek.kategorik_mi() {
            // `boundaryGap: false` kategorilerde bant merkez değil ardışık
            // kategori aralığıdır. dataZoom açıkken tam kategori sayısını
            // kullanmak otomatik label interval'ini binlerce kat büyütür.
            let aralık = self
                .pencere
                .map(|(p0, p1)| p1 - p0)
                .unwrap_or_else(|| {
                    let kapsam = self.ölçek.kapsam();
                    kapsam[1] - kapsam[0]
                })
                .max(1.0);
            uzunluk / aralık as f32
        } else {
            let çentikler = self.ölçek.çentikler();
            if çentikler.len() > 1 {
                uzunluk / (çentikler.len() - 1) as f32
            } else {
                uzunluk
            }
        }
    }

    /// Eksenin piksel uzunluğu.
    pub fn uzunluk(&self) -> f32 {
        (self.piksel[1] - self.piksel[0]).abs()
    }

    /// Değer, etkin pencerenin içinde mi?
    pub fn pencerede_mi(&self, değer: f64) -> bool {
        match self.pencere {
            Some((p0, p1)) => {
                let pay = if self.bantlı { 0.5 } else { 1e-9 };
                değer >= p0 - pay && değer <= p1 + pay
            }
            None => true,
        }
    }

    /// Veri öğesinin bu eksenin dataZoom işlemcisinden sonra çizimde kalıp
    /// kalmadığı. `none` yalnız koordinat kapsamını daraltıp veriyi korur;
    /// diğer kiplerde tek boyutlu çizgi öğesi pencere dışında bırakılır.
    pub fn veri_penceresinde_mi(&self, değer: f64) -> bool {
        self.pencere.is_none()
            || self.yakınlaştırma_süzme_kipi == YakınlaştırmaSüzmeKipi::Yok
            || self.pencerede_mi(değer)
    }

    /// Etiket çentikleri: `(piksel konumu, çentik)` çiftleri. Kategori
    /// eksenlerinde etiketler bant ortasındadır; pencere dışı çentikler
    /// atlanır.
    pub fn etiket_çentikleri(&self) -> Vec<(f32, Çentik)> {
        self.işlenmiş_çentikler()
            .into_iter()
            .filter(|ç| self.pencerede_mi(ç.değer))
            .filter(|çentik| {
                let kapsam = self
                    .kırılma_eşleyici
                    .as_ref()
                    .map(KırılmaEşleyici::kapsam)
                    .unwrap_or_else(|| self.ölçek.kapsam());
                if çentik.kırılma.is_none()
                    && (çentik.değer - kapsam[0]).abs() <= 1e-9
                    && self.seçenek.etiket.en_az_etiketini_göster == Some(false)
                {
                    return false;
                }
                if çentik.kırılma.is_none()
                    && (çentik.değer - kapsam[1]).abs() <= 1e-9
                    && self.seçenek.etiket.en_çok_etiketini_göster == Some(false)
                {
                    return false;
                }
                true
            })
            .map(|ç| (self.veriden_piksele(ç.değer), ç))
            .collect()
    }

    /// Çizgi çentikleri: eksen üstündeki işaret ve bölme çizgisi konumları.
    /// Bantlı kategori ekseninde bant sınırlarına düşer
    /// (`alignWithLabel: false` davranışı).
    pub fn çizgi_çentikleri(&self, etiketle_hizala: bool) -> Vec<f32> {
        if self.bantlı && !etiketle_hizala {
            let (b0, bant_sayısı) = match self.pencere {
                Some((p0, p1)) => (p0, (p1 - p0 + 1.0).round().max(1.0) as usize),
                None => (0.0, self.ölçek.kategori_sayısı().max(1)),
            };
            let [pik0, pik1] = self.etkin_piksel();
            (0..=bant_sayısı)
                .map(|i| {
                    let _ = b0;
                    doğrusal_eşle(
                        i as f64 / bant_sayısı as f64,
                        [0.0, 1.0],
                        [pik0, pik1],
                        true,
                    ) as f32
                })
                .collect()
        } else {
            self.işlenmiş_çentikler()
                .into_iter()
                .filter(|ç| self.pencerede_mi(ç.değer))
                .map(|ç| self.veriden_piksele(ç.değer))
                .collect()
        }
    }

    /// Bölme çizgisi ve alanı konumları. ECharts bu iki bileşeni
    /// `breakTicks: 'none'` ve `pruneByBreak: 'preserve_extent_bound'`
    /// seçenekleriyle kurar: kırılmaya yakın normal çentikler budanır,
    /// kırılmanın özel uç çentikleri ise eksen işareti/etiketi için korunup
    /// ızgaraya taşınmaz.
    pub fn bölme_çentikleri(&self) -> Vec<f32> {
        if self.bantlı {
            return self.çizgi_çentikleri(false);
        }
        self.işlenmiş_çentikler()
            .into_iter()
            .filter(|çentik| çentik.kırılma.is_none())
            .filter(|çentik| self.pencerede_mi(çentik.değer))
            .map(|çentik| self.veriden_piksele(çentik.değer))
            .collect()
    }

    /// Ara (minör) çentiklerin piksel konumları.
    pub fn ara_çentik_pikselleri(&self, bölme_sayısı: usize) -> Vec<f32> {
        self.ölçek
            .ara_çentikler(bölme_sayısı)
            .into_iter()
            .filter(|değer| !self.kırılmada_mı(*değer))
            .map(|d| self.veriden_piksele(d))
            .collect()
    }

    /// Etkin kapsamla kesişen kırılmalar.
    pub fn görünür_kırılmalar(&self) -> Vec<ÇözülmüşEksenKırılması> {
        self.kırılma_eşleyici
            .as_ref()
            .map(KırılmaEşleyici::görünür_kırılmalar)
            .unwrap_or_default()
    }

    /// Kırılma alanlarının veri ve piksel uçları.
    pub fn kırılma_piksel_aralıkları(&self) -> Vec<(f32, f32, ÇözülmüşEksenKırılması)> {
        self.görünür_kırılmalar()
            .into_iter()
            .map(|kırılma| {
                (
                    self.veriden_piksele(kırılma.başlangıç),
                    self.veriden_piksele(kırılma.bitiş),
                    kırılma,
                )
            })
            .collect()
    }

    fn kırılmada_mı(&self, değer: f64) -> bool {
        self.görünür_kırılmalar()
            .iter()
            .any(|kırılma| değer > kırılma.başlangıç && değer < kırılma.bitiş)
    }

    /// Normal çentikleri kırılma çevresinde budar ve kırılmanın iki ucunu
    /// yüksek öncelikli ayrı çentikler olarak ekler.
    fn işlenmiş_çentikler(&self) -> Vec<Çentik> {
        let mut çentikler = self.ölçek.çentikler();
        let Some(eşleyici) = &self.kırılma_eşleyici else {
            return çentikler;
        };
        let kapsam = eşleyici.kapsam();
        // TimeScale uç çentikleri nice diziden ayrı ve `notNice` olarak
        // ekler. Kırık eksende uçlar, etiket görünürlüğünden bağımsız olarak
        // kırılma eşlemesinin parçasıdır.
        çentikler.push(Çentik {
            değer: kapsam[0],
            kırılma: None,
        });
        çentikler.push(Çentik {
            değer: kapsam[1],
            kırılma: None,
        });

        let yaklaşık_aralık = match &self.ölçek {
            Ölçek::Zaman(ölçek) => ölçek.yaklaşık_aralık,
            _ => {
                let farklar = çentikler
                    .windows(2)
                    .filter_map(|çift| match çift {
                        [ilk, ikinci] => Some((ikinci.değer - ilk.değer).abs()),
                        _ => None,
                    })
                    .filter(|fark| *fark > 0.0 && fark.is_finite());
                farklar
                    .fold(f64::INFINITY, f64::min)
                    .min((kapsam[1] - kapsam[0]).abs() / self.seçenek.bölme_sayısı.max(1) as f64)
            }
        };
        let budama_boşluğu = if yaklaşık_aralık.is_finite() {
            yaklaşık_aralık * 0.75
        } else {
            0.0
        };
        for kırılma in eşleyici.görünür_kırılmalar() {
            çentikler.retain(|çentik| {
                !(çentik.değer > kırılma.başlangıç - budama_boşluğu
                    && çentik.değer < kırılma.bitiş + budama_boşluğu)
            });
            çentikler.push(Çentik {
                değer: kırılma.başlangıç,
                kırılma: Some(EksenKırılmaBilgisi {
                    tür: EksenKırılmaUcu::Başlangıç,
                    başlangıç: kırılma.başlangıç,
                    bitiş: kırılma.bitiş,
                }),
            });
            çentikler.push(Çentik {
                değer: kırılma.bitiş,
                kırılma: Some(EksenKırılmaBilgisi {
                    tür: EksenKırılmaUcu::Bitiş,
                    başlangıç: kırılma.başlangıç,
                    bitiş: kırılma.bitiş,
                }),
            });
        }
        çentikler.sort_by(|a, b| {
            a.değer
                .partial_cmp(&b.değer)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        çentikler.dedup_by(|a, b| {
            if (a.değer - b.değer).abs() > 1e-9 {
                return false;
            }
            if a.kırılma.is_none() && b.kırılma.is_some() {
                *a = *b;
            }
            true
        });
        çentikler
    }

    /// Yatay eksen mi?
    pub fn yatay_mı(&self) -> bool {
        matches!(self.konum, EksenKonumu::Alt | EksenKonumu::Üst)
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::model::eksen::{Eksen, EksenKırılmaUcu, EksenKırılması};
    use crate::olcek::{AralıkÖlçeği, KategorikÖlçek};

    #[test]
    fn aralıksız_kategori_zoom_birimini_alt_piksel_hesaplar() {
        let kategoriler = (0..20_000).map(|sıra| sıra.to_string()).collect();
        let mut eksen = ÇalışmaEkseni::yeni(
            Eksen::kategori().kenar_boşluğu(false),
            Ölçek::Kategorik(KategorikÖlçek::yeni(kategoriler)),
            [105.0, 630.0],
            EksenKonumu::Alt,
        );
        eksen.değer_penceresi_uygula(0.0, 2_000.0);
        assert!((eksen.bant_genişliği() - 0.2625).abs() < 1e-6);
    }

    #[test]
    fn filter_none_pencere_dışındaki_veriyi_korur() {
        let mut eksen = ÇalışmaEkseni::yeni(
            Eksen::değer(),
            Ölçek::Kategorik(KategorikÖlçek::yeni(vec!["a".into(), "b".into()])),
            [0.0, 100.0],
            EksenKonumu::Alt,
        );
        eksen.değer_penceresi_uygula(0.0, 1.0);
        eksen.yakınlaştırma_süzme_kipi = YakınlaştırmaSüzmeKipi::Yok;
        assert!(eksen.veri_penceresinde_mi(2.0));
        eksen.yakınlaştırma_süzme_kipi = YakınlaştırmaSüzmeKipi::Süz;
        assert!(!eksen.veri_penceresinde_mi(2.0));
    }

    #[test]
    fn kırık_eksen_piksel_eslemesini_ve_uc_centiklerini_korur() {
        let seçenek = Eksen::değer()
            .en_az(0.0)
            .en_çok(100.0)
            .kırılma(EksenKırılması::yeni(20.0, 40.0).boşluk(5.0));
        let ölçek = Ölçek::Aralık(AralıkÖlçeği::kur(
            [0.0, 100.0],
            Some(0.0),
            Some(100.0),
            false,
            5,
            None,
            None,
        ));
        let eksen = ÇalışmaEkseni::yeni(seçenek, ölçek, [0.0, 100.0], EksenKonumu::Alt);

        // 20 birim gizlenip yerine 5 birim boşluk kaldığından etkin açıklık
        // 85'tir; kırılmanın ekrandaki genişliği bunun 5/85'i olur.
        let kırılma_başı = eksen.veriden_piksele(20.0);
        let kırılma_sonu = eksen.veriden_piksele(40.0);
        assert!((kırılma_sonu - kırılma_başı - 100.0 * 5.0 / 85.0).abs() < 1e-5);
        for değer in [0.0, 10.0, 20.0, 25.0, 40.0, 70.0, 100.0] {
            let dönüş = eksen.pikselden_veriye(eksen.veriden_piksele(değer));
            assert!((dönüş - değer).abs() < 1e-5, "{değer} -> {dönüş}");
        }

        let çentikler = eksen.etiket_çentikleri();
        assert!(
            !çentikler
                .iter()
                .any(|(_, çentik)| çentik.değer > 20.0 && çentik.değer < 40.0)
        );
        let uçlar = çentikler
            .iter()
            .filter_map(|(_, çentik)| çentik.kırılma.map(|bilgi| bilgi.tür))
            .collect::<Vec<_>>();
        assert_eq!(
            uçlar,
            vec![EksenKırılmaUcu::Başlangıç, EksenKırılmaUcu::Bitiş]
        );

        let bölmeler = eksen.bölme_çentikleri();
        assert!(
            !bölmeler
                .iter()
                .any(|konum| (*konum - kırılma_başı).abs() < 1e-5)
        );
        assert!(
            !bölmeler
                .iter()
                .any(|konum| (*konum - kırılma_sonu).abs() < 1e-5)
        );
        let eksen_çentikleri = eksen.çizgi_çentikleri(false);
        assert!(
            eksen_çentikleri
                .iter()
                .any(|konum| (*konum - kırılma_başı).abs() < 1e-5)
        );
        assert!(
            eksen_çentikleri
                .iter()
                .any(|konum| (*konum - kırılma_sonu).abs() < 1e-5)
        );
    }
}
