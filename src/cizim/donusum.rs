//! 2B affine dönüşüm — zrender `Transformable` matris düzeni `[a,b,c,d,e,f]`.

/// Noktayı `x' = a*x + c*y + e`, `y' = b*x + d*y + f` ile dönüştürür.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AfinMatris {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Default for AfinMatris {
    fn default() -> Self {
        Self::BİRİM
    }
}

impl AfinMatris {
    pub const BİRİM: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub const fn yeni(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub fn ötele(x: f32, y: f32) -> Self {
        Self::yeni(1.0, 0.0, 0.0, 1.0, x, y)
    }

    pub fn ölçekle(x: f32, y: f32) -> Self {
        Self::yeni(x, 0.0, 0.0, y, 0.0, 0.0)
    }

    pub fn döndür(açı: f32) -> Self {
        let (sinüs, kosinüs) = açı.sin_cos();
        Self::yeni(kosinüs, sinüs, -sinüs, kosinüs, 0.0, 0.0)
    }

    /// `self * sağ`: önce `sağ`, ardından `self` uygulanır.
    pub fn çarp(self, sağ: Self) -> Self {
        Self {
            a: self.a * sağ.a + self.c * sağ.b,
            b: self.b * sağ.a + self.d * sağ.b,
            c: self.a * sağ.c + self.c * sağ.d,
            d: self.b * sağ.c + self.d * sağ.d,
            e: self.a * sağ.e + self.c * sağ.f + self.e,
            f: self.b * sağ.e + self.d * sağ.f + self.f,
        }
    }

    pub fn noktayı_dönüştür(self, nokta: (f32, f32)) -> (f32, f32) {
        (
            self.a * nokta.0 + self.c * nokta.1 + self.e,
            self.b * nokta.0 + self.d * nokta.1 + self.f,
        )
    }

    pub fn vektörü_dönüştür(self, vektör: (f32, f32)) -> (f32, f32) {
        (
            self.a * vektör.0 + self.c * vektör.1,
            self.b * vektör.0 + self.d * vektör.1,
        )
    }

    pub fn ters(self) -> Option<Self> {
        let determinant = self.a * self.d - self.b * self.c;
        if !determinant.is_finite() || determinant.abs() < 1e-12 {
            return None;
        }
        let ters = 1.0 / determinant;
        Some(Self {
            a: self.d * ters,
            b: -self.b * ters,
            c: -self.c * ters,
            d: self.a * ters,
            e: (self.c * self.f - self.d * self.e) * ters,
            f: (self.b * self.e - self.a * self.f) * ters,
        })
    }

    pub fn determinant(self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    pub fn x_ölçeği(self) -> f32 {
        self.a.hypot(self.b)
    }

    pub fn y_ölçeği(self) -> f32 {
        self.c.hypot(self.d)
    }

    pub fn dönüş_açısı(self) -> f32 {
        self.b.atan2(self.a)
    }

    pub fn dizi(self) -> [f32; 6] {
        [self.a, self.b, self.c, self.d, self.e, self.f]
    }

    pub fn sonlu_mu(self) -> bool {
        [self.a, self.b, self.c, self.d, self.e, self.f]
            .into_iter()
            .all(f32::is_finite)
    }
}

impl From<[f32; 6]> for AfinMatris {
    fn from(m: [f32; 6]) -> Self {
        let [a, b, c, d, e, f] = m;
        Self::yeni(a, b, c, d, e, f)
    }
}

impl From<AfinMatris> for [f32; 6] {
    fn from(m: AfinMatris) -> Self {
        m.dizi()
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn carpim_ters_ve_nokta() {
        let m = AfinMatris::ötele(10.0, -2.0)
            .çarp(AfinMatris::döndür(std::f32::consts::FRAC_PI_2))
            .çarp(AfinMatris::ölçekle(2.0, 3.0));
        let dünya = m.noktayı_dönüştür((1.0, 2.0));
        assert!((dünya.0 - 4.0).abs() < 1e-5);
        assert!(dünya.1.abs() < 1e-5);
        let yerel = m
            .ters()
            .map(|ters| ters.noktayı_dönüştür(dünya))
            .unwrap_or((f32::NAN, f32::NAN));
        assert!((yerel.0 - 1.0).abs() < 1e-5);
        assert!((yerel.1 - 2.0).abs() < 1e-5);
    }
}
