//! Kutupsal koordinat seçenekleri — ECharts `polar` + `angleAxis` +
//! `radiusAxis` bileşenlerinin sadeleştirilmiş karşılığı.

use crate::model::eksen::Eksen;
use crate::model::Uzunluk;

/// Kutupsal koordinat sistemi (`polar`).
#[derive(Clone, PartialEq, Debug)]
pub struct KutupsalKoordinat {
    pub merkez: (Uzunluk, Uzunluk),
    pub yarıçap: Uzunluk,
    /// Açısal eksen (`angleAxis`): verisi doluysa kategorik.
    pub açısal_eksen: Eksen,
    /// Radyal eksen (`radiusAxis`): değer ekseni.
    pub radyal_eksen: Eksen,
}

impl Default for KutupsalKoordinat {
    fn default() -> Self {
        KutupsalKoordinat {
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(55.0)),
            yarıçap: Uzunluk::Yüzde(70.0),
            açısal_eksen: Eksen::kategori(),
            radyal_eksen: Eksen::değer(),
        }
    }
}

impl KutupsalKoordinat {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn merkez(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.merkez = (x.into(), y.into());
        self
    }

    pub fn yarıçap(mut self, yarıçap: impl Into<Uzunluk>) -> Self {
        self.yarıçap = yarıçap.into();
        self
    }

    pub fn açısal_eksen(mut self, eksen: Eksen) -> Self {
        self.açısal_eksen = eksen;
        self
    }

    pub fn radyal_eksen(mut self, eksen: Eksen) -> Self {
        self.radyal_eksen = eksen;
        self
    }
}
