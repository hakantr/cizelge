use crate::Nokta;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrafikDonusumu {
    kaynak_bas: Nokta,
    kaynak_son: Nokta,
    hedef_bas: Nokta,
    hedef_son: Nokta,
}

impl GrafikDonusumu {
    pub fn yeni(
        kaynak_bas: Nokta,
        kaynak_son: Nokta,
        hedef_bas: Nokta,
        hedef_son: Nokta,
    ) -> Result<Self, DonusumHatasi> {
        if ![
            kaynak_bas.x,
            kaynak_bas.y,
            kaynak_son.x,
            kaynak_son.y,
            hedef_bas.x,
            hedef_bas.y,
            hedef_son.x,
            hedef_son.y,
        ]
        .into_iter()
        .all(f64::is_finite)
        {
            return Err(DonusumHatasi::SonluDegil);
        }
        if (kaynak_son.x - kaynak_bas.x).abs() < f64::EPSILON
            || (kaynak_son.y - kaynak_bas.y).abs() < f64::EPSILON
            || (hedef_son.x - hedef_bas.x).abs() < f64::EPSILON
            || (hedef_son.y - hedef_bas.y).abs() < f64::EPSILON
        {
            return Err(DonusumHatasi::DejenereAlan);
        }
        Ok(Self {
            kaynak_bas,
            kaynak_son,
            hedef_bas,
            hedef_son,
        })
    }

    pub fn ekrana(self, nokta: Nokta) -> Nokta {
        Nokta {
            x: haritala(
                nokta.x,
                self.kaynak_bas.x,
                self.kaynak_son.x,
                self.hedef_bas.x,
                self.hedef_son.x,
            ),
            y: haritala(
                nokta.y,
                self.kaynak_bas.y,
                self.kaynak_son.y,
                self.hedef_bas.y,
                self.hedef_son.y,
            ),
        }
    }

    pub fn veriye(self, nokta: Nokta) -> Nokta {
        Nokta {
            x: haritala(
                nokta.x,
                self.hedef_bas.x,
                self.hedef_son.x,
                self.kaynak_bas.x,
                self.kaynak_son.x,
            ),
            y: haritala(
                nokta.y,
                self.hedef_bas.y,
                self.hedef_son.y,
                self.kaynak_bas.y,
                self.kaynak_son.y,
            ),
        }
    }

    pub fn kaynak_alani(self) -> (Nokta, Nokta) {
        (self.kaynak_bas, self.kaynak_son)
    }
}

fn haritala(deger: f64, alan_bas: f64, alan_son: f64, hedef_bas: f64, hedef_son: f64) -> f64 {
    let oran = (deger - alan_bas) / (alan_son - alan_bas);
    hedef_bas + oran * (hedef_son - hedef_bas)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DonusumHatasi {
    SonluDegil,
    DejenereAlan,
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn ekran_veri_roundtrip_ve_ters_y_ekseni() {
        let donusum = GrafikDonusumu::yeni(
            Nokta::yeni(0.0, 0.0),
            Nokta::yeni(100.0, 50.0),
            Nokta::yeni(0.0, 500.0),
            Nokta::yeni(1_000.0, 0.0),
        )
        .unwrap();
        let kaynak = Nokta::yeni(25.0, 10.0);
        let geri = donusum.veriye(donusum.ekrana(kaynak));
        assert!((geri.x - kaynak.x).abs() < f64::EPSILON);
        assert!((geri.y - kaynak.y).abs() < f64::EPSILON);
    }
}
