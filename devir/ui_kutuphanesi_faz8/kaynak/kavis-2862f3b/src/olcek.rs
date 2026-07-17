#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OlcekHatasi {
    SonluDegil,
    EsitAlan,
    EsitAralik,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DogrusalOlcek {
    alan_bas: f64,
    alan_son: f64,
    aralik_bas: f64,
    aralik_son: f64,
}

impl DogrusalOlcek {
    pub fn yeni(
        alan_bas: f64,
        alan_son: f64,
        aralik_bas: f64,
        aralik_son: f64,
    ) -> Result<Self, OlcekHatasi> {
        if ![alan_bas, alan_son, aralik_bas, aralik_son]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err(OlcekHatasi::SonluDegil);
        }
        if (alan_son - alan_bas).abs() < f64::EPSILON {
            return Err(OlcekHatasi::EsitAlan);
        }
        if (aralik_son - aralik_bas).abs() < f64::EPSILON {
            return Err(OlcekHatasi::EsitAralik);
        }
        Ok(Self {
            alan_bas,
            alan_son,
            aralik_bas,
            aralik_son,
        })
    }

    pub fn haritala(self, deger: f64) -> f64 {
        let oran = (deger - self.alan_bas) / (self.alan_son - self.alan_bas);
        self.aralik_bas + oran * (self.aralik_son - self.aralik_bas)
    }

    pub fn tersine(self, deger: f64) -> f64 {
        let oran = (deger - self.aralik_bas) / (self.aralik_son - self.aralik_bas);
        self.alan_bas + oran * (self.alan_son - self.alan_bas)
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn ters_aralik_ve_roundtrip() {
        let olcek = DogrusalOlcek::yeni(0.0, 100.0, 500.0, 0.0).unwrap();
        assert!((olcek.haritala(25.0) - 375.0).abs() < f64::EPSILON);
        assert!((olcek.tersine(olcek.haritala(42.0)) - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dejenere_alan_reddedilir() {
        assert_eq!(
            DogrusalOlcek::yeni(1.0, 1.0, 0.0, 100.0),
            Err(OlcekHatasi::EsitAlan)
        );
    }
}
