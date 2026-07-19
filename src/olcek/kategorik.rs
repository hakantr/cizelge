//! Kategorik (sırasal) ölçek — `echarts/src/scale/Ordinal.ts` portu.

use crate::olcek::Çentik;
use crate::yardimci::sayi::doğrusal_eşle;

/// Kategori ekseni ölçeği (`OrdinalScale`).
///
/// Kapsamı `[0, kategori_sayısı - 1]` tam sayı aralığıdır; bant (aralıklı
/// yerleşim) hesabı çalışma ekseninde yapılır.
#[derive(Clone, Debug, Default)]
pub struct KategorikÖlçek {
    pub kategoriler: Vec<String>,
}

impl KategorikÖlçek {
    pub fn yeni(kategoriler: Vec<String>) -> Self {
        KategorikÖlçek { kategoriler }
    }

    pub fn kapsam(&self) -> [f64; 2] {
        if self.kategoriler.is_empty() {
            [0.0, 0.0]
        } else {
            [0.0, (self.kategoriler.len() - 1) as f64]
        }
    }

    pub fn oranla(&self, değer: f64) -> f64 {
        doğrusal_eşle(değer, self.kapsam(), [0.0, 1.0], true)
    }

    pub fn orandan(&self, oran: f64) -> f64 {
        doğrusal_eşle(oran, [0.0, 1.0], self.kapsam(), true).round()
    }

    /// Her kategori için bir çentik (`ordinalScaleCreateTicks`in
    /// aralıksız hali; etiket seyreltme çizim aşamasında yapılır).
    pub fn çentikler(&self) -> Vec<Çentik> {
        (0..self.kategoriler.len())
            .map(|i| Çentik {
                değer: i as f64,
                kırılma: None,
            })
            .collect()
    }

    pub fn etiket(&self, değer: f64) -> String {
        let i = değer.round() as isize;
        if i < 0 {
            return String::new();
        }
        self.kategoriler
            .get(i as usize)
            .cloned()
            .unwrap_or_default()
    }
}
