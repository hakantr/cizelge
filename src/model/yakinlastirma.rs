//! Veri yakınlaştırma seçenekleri — ECharts `dataZoom` bileşeninin
//! karşılığı. `İç` tür fare tekerleği/sürüklemeyle, `Sürgü` tür alt
//! şeritteki tutamaçlarla pencereyi değiştirir.

/// Yakınlaştırma türü (`dataZoom.type`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum YakınlaştırmaTürü {
    /// Izgara içinde tekerlek + sürükleme (`'inside'`).
    #[default]
    İç,
    /// Alt şerit sürgüsü (`'slider'`).
    Sürgü,
}

/// Veri yakınlaştırma tanımı (`dataZoom` öğesi).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VeriYakınlaştırma {
    pub tür: YakınlaştırmaTürü,
    /// Bağlı x ekseninin sırası (`xAxisIndex`).
    pub x_eksen_sırası: usize,
    /// Pencere başlangıcı, yüzde `0..=100` (`start`).
    pub başlangıç: f32,
    /// Pencere bitişi, yüzde `0..=100` (`end`).
    pub bitiş: f32,
}

impl Default for VeriYakınlaştırma {
    fn default() -> Self {
        VeriYakınlaştırma {
            tür: YakınlaştırmaTürü::İç,
            x_eksen_sırası: 0,
            başlangıç: 0.0,
            bitiş: 100.0,
        }
    }
}

impl VeriYakınlaştırma {
    /// Izgara içi yakınlaştırma (`'inside'`).
    pub fn iç() -> Self {
        Self::default()
    }

    /// Alt şerit sürgüsü (`'slider'`).
    pub fn sürgü() -> Self {
        VeriYakınlaştırma { tür: YakınlaştırmaTürü::Sürgü, ..Default::default() }
    }

    pub fn x_eksen_sırası(mut self, sıra: usize) -> Self {
        self.x_eksen_sırası = sıra;
        self
    }

    /// Başlangıç penceresi, yüzde.
    pub fn aralık(mut self, başlangıç: f32, bitiş: f32) -> Self {
        self.başlangıç = başlangıç.clamp(0.0, 100.0);
        self.bitiş = bitiş.clamp(self.başlangıç, 100.0);
        self
    }

    /// Pencere oranları `0..=1`.
    pub fn oranlar(&self) -> (f32, f32) {
        (self.başlangıç / 100.0, self.bitiş / 100.0)
    }

    /// Pencere etkin mi (tam açıklıktan farklı mı)?
    pub fn etkin_mi(&self) -> bool {
        self.başlangıç > 0.001 || self.bitiş < 99.999
    }
}
