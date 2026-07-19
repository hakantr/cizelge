//! ECharts 6.1 `matrix` bileşeninin option modeli.

use crate::model::Uzunluk;
use crate::model::stil::{Etiket, ÇizgiStili, ÖğeStili};
use crate::renk::Renk;

/// Hiyerarşik x/y başlık hücresi. Yapraklar gövde satır/sütunlarını belirler.
#[derive(Clone, Debug, PartialEq)]
pub struct MatrisBoyutHücresi {
    pub değer: String,
    pub boyut: Option<Uzunluk>,
    pub çocuklar: Vec<MatrisBoyutHücresi>,
    pub öğe_stili: Option<ÖğeStili>,
    pub etiket: Option<Etiket>,
}

impl MatrisBoyutHücresi {
    pub fn yeni(değer: impl Into<String>) -> Self {
        Self {
            değer: değer.into(),
            boyut: None,
            çocuklar: Vec::new(),
            öğe_stili: None,
            etiket: None,
        }
    }

    pub fn boyut(mut self, boyut: impl Into<Uzunluk>) -> Self {
        self.boyut = Some(boyut.into());
        self
    }

    pub fn çocuk(mut self, çocuk: MatrisBoyutHücresi) -> Self {
        self.çocuklar.push(çocuk);
        self
    }

    pub fn çocuklar(mut self, çocuklar: impl IntoIterator<Item = MatrisBoyutHücresi>) -> Self {
        self.çocuklar.extend(çocuklar);
        self
    }
}

impl From<&str> for MatrisBoyutHücresi {
    fn from(değer: &str) -> Self {
        Self::yeni(değer)
    }
}

impl From<String> for MatrisBoyutHücresi {
    fn from(değer: String) -> Self {
        Self::yeni(değer)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatrisBoyutu {
    pub göster: bool,
    pub veri: Vec<MatrisBoyutHücresi>,
    pub uzunluk: Option<usize>,
    pub seviye_boyutları: Vec<Option<Uzunluk>>,
    pub seviye_boyutu: Option<Uzunluk>,
    pub öğe_stili: ÖğeStili,
    pub etiket: Etiket,
    pub ayırıcı: ÇizgiStili,
}

impl Default for MatrisBoyutu {
    fn default() -> Self {
        Self {
            göster: true,
            veri: Vec::new(),
            uzunluk: None,
            seviye_boyutları: Vec::new(),
            seviye_boyutu: None,
            öğe_stili: ÖğeStili::yeni()
                .kenarlık_rengi(Renk::onaltılık(0xe0e6f1))
                .kenarlık_kalınlığı(1.0),
            etiket: Etiket::yeni().göster(true),
            ayırıcı: ÇizgiStili::yeni()
                .renk(Renk::onaltılık(0xb7c1d0))
                .kalınlık(1.0),
        }
    }
}

impl MatrisBoyutu {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn veri<T: Into<MatrisBoyutHücresi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri.into_iter().map(Into::into).collect();
        self
    }

    pub fn uzunluk(mut self, uzunluk: usize) -> Self {
        self.uzunluk = Some(uzunluk);
        self
    }

    pub fn seviye_boyutu(mut self, boyut: impl Into<Uzunluk>) -> Self {
        self.seviye_boyutu = Some(boyut.into());
        self
    }
}

/// Matris gövdesindeki bir hücreyi ya da aralığı adresler.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MatrisKonumu {
    Sıra(isize),
    Değer(String),
}

impl From<usize> for MatrisKonumu {
    fn from(sıra: usize) -> Self {
        Self::Sıra(sıra as isize)
    }
}

impl From<isize> for MatrisKonumu {
    fn from(sıra: isize) -> Self {
        Self::Sıra(sıra)
    }
}

impl From<&str> for MatrisKonumu {
    fn from(değer: &str) -> Self {
        Self::Değer(değer.to_owned())
    }
}

impl From<String> for MatrisKonumu {
    fn from(değer: String) -> Self {
        Self::Değer(değer)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatrisAralığı {
    Tek(MatrisKonumu),
    Aralık(MatrisKonumu, MatrisKonumu),
    Tümü,
}

impl<T: Into<MatrisKonumu>> From<T> for MatrisAralığı {
    fn from(değer: T) -> Self {
        Self::Tek(değer.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatrisGövdeHücresi {
    pub değer: Option<String>,
    pub x: MatrisAralığı,
    pub y: MatrisAralığı,
    pub hücreleri_birleştir: bool,
    pub öğe_stili: Option<ÖğeStili>,
    pub etiket: Option<Etiket>,
    pub sessiz: Option<bool>,
}

impl MatrisGövdeHücresi {
    pub fn yeni(x: impl Into<MatrisAralığı>, y: impl Into<MatrisAralığı>) -> Self {
        Self {
            değer: None,
            x: x.into(),
            y: y.into(),
            hücreleri_birleştir: false,
            öğe_stili: None,
            etiket: None,
            sessiz: None,
        }
    }

    pub fn değer(mut self, değer: impl Into<String>) -> Self {
        self.değer = Some(değer.into());
        self
    }

    pub fn birleştir(mut self, açık: bool) -> Self {
        self.hücreleri_birleştir = açık;
        self
    }
}

/// Kök `matrix` component option'ı.
#[derive(Clone, Debug, PartialEq)]
pub struct MatrisKoordinatı {
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub sağ: Uzunluk,
    pub alt: Uzunluk,
    pub x: MatrisBoyutu,
    pub y: MatrisBoyutu,
    pub gövde_verisi: Vec<MatrisGövdeHücresi>,
    pub gövde_stili: ÖğeStili,
    pub arkaplan_stili: ÖğeStili,
    pub tetikleme_olayı: bool,
}

impl Default for MatrisKoordinatı {
    fn default() -> Self {
        Self {
            sol: Uzunluk::Yüzde(10.0),
            üst: Uzunluk::Yüzde(10.0),
            sağ: Uzunluk::Yüzde(10.0),
            alt: Uzunluk::Yüzde(10.0),
            x: MatrisBoyutu::default(),
            y: MatrisBoyutu::default(),
            gövde_verisi: Vec::new(),
            gövde_stili: ÖğeStili::yeni()
                .kenarlık_rengi(Renk::onaltılık(0xe0e6f1))
                .kenarlık_kalınlığı(1.0),
            arkaplan_stili: ÖğeStili::yeni()
                .kenarlık_rengi(Renk::onaltılık(0x6e7e92))
                .kenarlık_kalınlığı(1.0),
            tetikleme_olayı: false,
        }
    }
}

impl MatrisKoordinatı {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn x(mut self, x: MatrisBoyutu) -> Self {
        self.x = x;
        self
    }

    pub fn y(mut self, y: MatrisBoyutu) -> Self {
        self.y = y;
        self
    }

    pub fn gövde_hücresi(mut self, hücre: MatrisGövdeHücresi) -> Self {
        self.gövde_verisi.push(hücre);
        self
    }

    pub fn sol(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sol = değer.into();
        self
    }

    pub fn üst(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.üst = değer.into();
        self
    }

    pub fn sağ(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sağ = değer.into();
        self
    }

    pub fn alt(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.alt = değer.into();
        self
    }
}
