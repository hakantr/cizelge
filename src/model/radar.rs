//! Radar koordinat seçenekleri — ECharts `radar` bileşeninin karşılığı.

use crate::model::Uzunluk;

/// Radar ağının biçimi (`radar.shape`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RadarŞekli {
    #[default]
    Çokgen,
    Daire,
}

/// Radar göstergesi (`radar.indicator` öğesi): bir kolun adı ve aralığı.
#[derive(Clone, PartialEq, Debug)]
pub struct RadarGöstergesi {
    pub ad: String,
    pub en_az: f64,
    pub en_çok: f64,
}

impl RadarGöstergesi {
    pub fn yeni(ad: impl Into<String>, en_çok: f64) -> Self {
        RadarGöstergesi {
            ad: ad.into(),
            en_az: 0.0,
            en_çok,
        }
    }
}

/// Radar koordinat sistemi (`radar` seçeneği).
#[derive(Clone, PartialEq, Debug)]
pub struct RadarKoordinatı {
    pub göstergeler: Vec<RadarGöstergesi>,
    pub merkez: (Uzunluk, Uzunluk),
    pub yarıçap: Uzunluk,
    /// Halkalar arası bölme sayısı (`splitNumber`, öntanımlı 5).
    pub bölme_sayısı: usize,
    pub şekil: RadarŞekli,
    /// Dönüşümlü bölme alanları çizilsin mi (`splitArea.show`)?
    pub bölme_alanı_göster: bool,
}

impl Default for RadarKoordinatı {
    fn default() -> Self {
        RadarKoordinatı {
            göstergeler: Vec::new(),
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(55.0)),
            yarıçap: Uzunluk::Yüzde(70.0),
            bölme_sayısı: 5,
            şekil: RadarŞekli::Çokgen,
            bölme_alanı_göster: true,
        }
    }
}

impl RadarKoordinatı {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Göstergeler: `(ad, en_çok)` çiftleri.
    pub fn göstergeler<S: Into<String>>(
        mut self,
        göstergeler: impl IntoIterator<Item = (S, f64)>,
    ) -> Self {
        self.göstergeler = göstergeler
            .into_iter()
            .map(|(ad, en_çok)| RadarGöstergesi::yeni(ad, en_çok))
            .collect();
        self
    }

    pub fn merkez(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.merkez = (x.into(), y.into());
        self
    }

    pub fn yarıçap(mut self, yarıçap: impl Into<Uzunluk>) -> Self {
        self.yarıçap = yarıçap.into();
        self
    }

    pub fn şekil(mut self, şekil: RadarŞekli) -> Self {
        self.şekil = şekil;
        self
    }

    pub fn bölme_sayısı(mut self, sayı: usize) -> Self {
        self.bölme_sayısı = sayı.max(1);
        self
    }
}
