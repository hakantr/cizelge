use ortak_tipler::tema::Renk;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EksenIsareti {
    pub deger: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Eksen {
    baslangic: f64,
    bitis: f64,
    isaretler: Vec<EksenIsareti>,
}

impl Eksen {
    pub fn dogrusal(baslangic: f64, bitis: f64, aralik_sayisi: u32) -> Result<Self, EksenHatasi> {
        if !baslangic.is_finite() || !bitis.is_finite() {
            return Err(EksenHatasi::SonluDegil);
        }
        if (bitis - baslangic).abs() < f64::EPSILON {
            return Err(EksenHatasi::EsitAlan);
        }
        if aralik_sayisi == 0 {
            return Err(EksenHatasi::SifirAralik);
        }
        let adim = (bitis - baslangic) / f64::from(aralik_sayisi);
        let isaretler = (0..=aralik_sayisi)
            .map(|sira| EksenIsareti {
                deger: baslangic + f64::from(sira) * adim,
            })
            .collect();
        Ok(Self {
            baslangic,
            bitis,
            isaretler,
        })
    }

    pub fn alan(&self) -> (f64, f64) {
        (self.baslangic, self.bitis)
    }

    pub fn isaretler(&self) -> &[EksenIsareti] {
        &self.isaretler
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EsikliRenkOlcegi {
    esikler: Vec<f64>,
    renkler: Vec<Renk>,
}

impl EsikliRenkOlcegi {
    pub fn yeni(esikler: Vec<f64>, renkler: Vec<Renk>) -> Result<Self, EksenHatasi> {
        if renkler.len() != esikler.len().saturating_add(1) {
            return Err(EksenHatasi::RenkSayisi);
        }
        if esikler.iter().any(|esik| !esik.is_finite()) {
            return Err(EksenHatasi::SonluDegil);
        }
        if esikler.windows(2).any(|cift| cift[0] >= cift[1]) {
            return Err(EksenHatasi::SiraliEsikGerekli);
        }
        Ok(Self { esikler, renkler })
    }

    pub fn renk(&self, deger: f64) -> Option<Renk> {
        deger.is_finite().then(|| {
            let sira = self.esikler.partition_point(|esik| deger >= *esik);
            self.renkler[sira]
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EksenHatasi {
    SonluDegil,
    EsitAlan,
    SifirAralik,
    RenkSayisi,
    SiraliEsikGerekli,
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn eksen_uclari_korur() {
        let eksen = Eksen::dogrusal(-10.0, 10.0, 4).unwrap();
        assert_eq!(eksen.isaretler().len(), 5);
        assert!((eksen.isaretler()[2].deger - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn renk_esikleri_sirali_ve_tam_olmalidir() {
        let kirmizi = Renk::rgb(0xFF0000);
        let yesil = Renk::rgb(0x00FF00);
        let olcek = EsikliRenkOlcegi::yeni(vec![10.0], vec![yesil, kirmizi]).unwrap();
        assert_eq!(olcek.renk(9.0), Some(yesil));
        assert_eq!(olcek.renk(10.0), Some(kirmizi));
        assert_eq!(olcek.renk(f64::NAN), None);
    }
}
