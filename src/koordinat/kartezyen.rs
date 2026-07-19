//! Kartezyen 2B koordinat sistemi — `echarts/src/coord/cartesian` karşılığı.

use crate::koordinat::eksen::ÇalışmaEkseni;

/// Basit piksel dikdörtgeni.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Dikdörtgen {
    pub x: f32,
    pub y: f32,
    pub genişlik: f32,
    pub yükseklik: f32,
}

impl Dikdörtgen {
    pub fn yeni(x: f32, y: f32, genişlik: f32, yükseklik: f32) -> Self {
        Dikdörtgen {
            x,
            y,
            genişlik,
            yükseklik,
        }
    }

    pub fn sağ(&self) -> f32 {
        self.x + self.genişlik
    }

    pub fn alt(&self) -> f32 {
        self.y + self.yükseklik
    }

    pub fn merkez(&self) -> (f32, f32) {
        (self.x + self.genişlik / 2.0, self.y + self.yükseklik / 2.0)
    }

    pub fn içeriyor_mu(&self, nokta: (f32, f32)) -> bool {
        nokta.0 >= self.x && nokta.0 <= self.sağ() && nokta.1 >= self.y && nokta.1 <= self.alt()
    }
}

/// x + y ekseni çifti ve ızgara alanı (`Cartesian2D`).
#[derive(Clone, Debug)]
pub struct Kartezyen2B {
    pub x: ÇalışmaEkseni,
    pub y: ÇalışmaEkseni,
    pub alan: Dikdörtgen,
}

impl Kartezyen2B {
    /// Veri çiftini piksel noktasına eşler (`Cartesian2D#dataToPoint`).
    pub fn nokta(&self, x_değeri: f64, y_değeri: f64) -> (f32, f32) {
        (
            self.x.veriden_piksele(x_değeri),
            self.y.veriden_piksele(y_değeri),
        )
    }
}
