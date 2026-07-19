//! Çalışma zamanı ekseni — `echarts/src/coord/Axis.ts` karşılığı.
//!
//! Bir ölçeği piksel aralığına bağlar; kategori eksenlerinde bant (aralıklı
//! yerleşim) hesabını üstlenir.

use crate::model::eksen::{Eksen, EksenKonumu};
use crate::olcek::{Çentik, Ölçek};
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
}

impl ÇalışmaEkseni {
    pub fn yeni(seçenek: Eksen, ölçek: Ölçek, piksel: [f32; 2], konum: EksenKonumu) -> Self {
        let bantlı = seçenek.bantlı_mı() && ölçek.kategorik_mi();
        ÇalışmaEkseni {
            seçenek,
            ölçek,
            piksel,
            bantlı,
            konum,
            pencere: None,
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
        let oran = match (self.pencere, self.bantlı) {
            (Some((p0, p1)), true) => (değer - p0 + 0.5) / (p1 - p0 + 1.0),
            (Some((p0, p1)), false) => (değer - p0) / (p1 - p0).max(1e-12),
            (None, true) => {
                let n = self.ölçek.kategori_sayısı().max(1) as f64;
                (değer + 0.5) / n
            }
            (None, false) => self.ölçek.oranla(değer),
        };
        // Pencere dışı değerler ızgara dışına taşar; çizim kırpılır.
        doğrusal_eşle(
            oran,
            [0.0, 1.0],
            self.etkin_piksel(),
            self.pencere.is_none(),
        ) as f32
    }

    /// Pikseli veri değerine eşler (`Axis#coordToData`).
    pub fn pikselden_veriye(&self, piksel: f32) -> f64 {
        let oran = doğrusal_eşle(piksel as f64, self.etkin_piksel(), [0.0, 1.0], true);
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

    /// Etiket çentikleri: `(piksel konumu, çentik)` çiftleri. Kategori
    /// eksenlerinde etiketler bant ortasındadır; pencere dışı çentikler
    /// atlanır.
    pub fn etiket_çentikleri(&self) -> Vec<(f32, Çentik)> {
        self.ölçek
            .çentikler()
            .into_iter()
            .filter(|ç| self.pencerede_mi(ç.değer))
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
            self.ölçek
                .çentikler()
                .into_iter()
                .filter(|ç| self.pencerede_mi(ç.değer))
                .map(|ç| self.veriden_piksele(ç.değer))
                .collect()
        }
    }

    /// Ara (minör) çentiklerin piksel konumları.
    pub fn ara_çentik_pikselleri(&self, bölme_sayısı: usize) -> Vec<f32> {
        self.ölçek
            .ara_çentikler(bölme_sayısı)
            .into_iter()
            .map(|d| self.veriden_piksele(d))
            .collect()
    }

    /// Yatay eksen mi?
    pub fn yatay_mı(&self) -> bool {
        matches!(self.konum, EksenKonumu::Alt | EksenKonumu::Üst)
    }
}
