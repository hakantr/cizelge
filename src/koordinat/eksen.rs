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
}

impl ÇalışmaEkseni {
    pub fn yeni(seçenek: Eksen, ölçek: Ölçek, piksel: [f32; 2], konum: EksenKonumu) -> Self {
        let bantlı = seçenek.bantlı_mı() && ölçek.kategorik_mi();
        ÇalışmaEkseni { seçenek, ölçek, piksel, bantlı, konum }
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
        let oran = if self.bantlı {
            let n = self.ölçek.kategori_sayısı().max(1) as f64;
            (değer + 0.5) / n
        } else {
            self.ölçek.oranla(değer)
        };
        doğrusal_eşle(oran, [0.0, 1.0], self.etkin_piksel(), true) as f32
    }

    /// Pikseli veri değerine eşler (`Axis#coordToData`).
    pub fn pikselden_veriye(&self, piksel: f32) -> f64 {
        let oran = doğrusal_eşle(piksel as f64, self.etkin_piksel(), [0.0, 1.0], true);
        if self.bantlı {
            let n = self.ölçek.kategori_sayısı().max(1) as f64;
            (oran * n - 0.5).round().clamp(0.0, n - 1.0)
        } else {
            self.ölçek.orandan(oran)
        }
    }

    /// Bant genişliği, piksel (`Axis#getBandWidth`).
    pub fn bant_genişliği(&self) -> f32 {
        let uzunluk = (self.piksel[1] - self.piksel[0]).abs();
        if self.bantlı {
            let n = self.ölçek.kategori_sayısı().max(1) as f32;
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

    /// Etiket çentikleri: `(piksel konumu, çentik)` çiftleri. Kategori
    /// eksenlerinde etiketler bant ortasındadır.
    pub fn etiket_çentikleri(&self) -> Vec<(f32, Çentik)> {
        self.ölçek
            .çentikler()
            .into_iter()
            .map(|ç| (self.veriden_piksele(ç.değer), ç))
            .collect()
    }

    /// Çizgi çentikleri: eksen üstündeki işaret ve bölme çizgisi konumları.
    /// Bantlı kategori ekseninde bant sınırlarına düşer
    /// (`alignWithLabel: false` davranışı).
    pub fn çizgi_çentikleri(&self, etiketle_hizala: bool) -> Vec<f32> {
        if self.bantlı && !etiketle_hizala {
            let n = self.ölçek.kategori_sayısı().max(1);
            let [p0, p1] = self.etkin_piksel();
            (0..=n)
                .map(|i| {
                    doğrusal_eşle(i as f64 / n as f64, [0.0, 1.0], [p0, p1], true) as f32
                })
                .collect()
        } else {
            self.ölçek
                .çentikler()
                .into_iter()
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
