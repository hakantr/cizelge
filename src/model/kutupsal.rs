//! Kutupsal koordinat seçenekleri — ECharts `polar` + `angleAxis` +
//! `radiusAxis` bileşenlerinin sadeleştirilmiş karşılığı.

use crate::model::Uzunluk;
use crate::model::eksen::Eksen;

/// Kutupsal koordinat sistemi (`polar`).
#[derive(Clone, PartialEq, Debug)]
pub struct KutupsalKoordinat {
    pub merkez: (Uzunluk, Uzunluk),
    /// Radyal eksenin iç yarıçapı (`polar.radius[0]`).
    pub iç_yarıçap: Uzunluk,
    /// Radyal eksenin dış yarıçapı (`polar.radius[1]` ya da tekil
    /// `polar.radius`).
    pub yarıçap: Uzunluk,
    /// Açısal eksen (`angleAxis`): verisi doluysa kategorik.
    pub açısal_eksen: Eksen,
    /// Radyal eksen (`radiusAxis`): değer/kategori/zaman/log ekseni.
    pub radyal_eksen: Eksen,
    /// Açısal eksenin başlangıcı, derece (`angleAxis.startAngle`).
    pub başlangıç_açısı: f32,
    /// Açı değerleri saat yönünde artsın (`angleAxis.clockwise`).
    pub saat_yönü: bool,
}

impl Default for KutupsalKoordinat {
    fn default() -> Self {
        KutupsalKoordinat {
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(50.0)),
            iç_yarıçap: Uzunluk::Piksel(0.0),
            yarıçap: Uzunluk::Yüzde(80.0),
            açısal_eksen: Eksen::değer().bölme_sayısı(12),
            radyal_eksen: Eksen::değer(),
            başlangıç_açısı: 90.0,
            saat_yönü: true,
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
        self.iç_yarıçap = Uzunluk::Piksel(0.0);
        self.yarıçap = yarıçap.into();
        self
    }

    /// ECharts `polar.radius: [inner, outer]` biçimi.
    pub fn yarıçap_aralığı(
        mut self, iç: impl Into<Uzunluk>, dış: impl Into<Uzunluk>
    ) -> Self {
        self.iç_yarıçap = iç.into();
        self.yarıçap = dış.into();
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

    pub fn başlangıç_açısı(mut self, derece: f32) -> Self {
        self.başlangıç_açısı = derece;
        self
    }

    pub fn saat_yönü(mut self, saat_yönü: bool) -> Self {
        self.saat_yönü = saat_yönü;
        self
    }
}
