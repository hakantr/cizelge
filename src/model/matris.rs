//! ECharts 6.1 `matrix` bileşeninin option modeli.

use std::{fmt, sync::Arc};

use crate::model::Uzunluk;
use crate::model::stil::{Etiket, ÇizgiStili, ÖğeStili};
use crate::renk::Renk;

/// `matrix.x/y/body/corner.label.formatter` işlevine aktarılan resmi
/// ECharts bağlamı.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatrisEtiketiBağlamı {
    pub bileşen_sırası: usize,
    pub ad: String,
    pub değer: String,
    pub koordinat: [isize; 2],
}

type MatrisEtiketiBiçimleyiciİşlevi = dyn Fn(&MatrisEtiketiBağlamı) -> String + Send + Sync;

/// Klonlanabilir `matrix.label.formatter(params)` sarmalayıcısı.
#[derive(Clone)]
pub struct MatrisEtiketiBiçimleyicisi(Arc<MatrisEtiketiBiçimleyiciİşlevi>);

impl MatrisEtiketiBiçimleyicisi {
    pub fn yeni(
        biçimleyici: impl Fn(&MatrisEtiketiBağlamı) -> String + Send + Sync + 'static,
    ) -> Self {
        Self(Arc::new(biçimleyici))
    }

    pub fn uygula(&self, bağlam: &MatrisEtiketiBağlamı) -> String {
        (self.0)(bağlam)
    }
}

impl fmt::Debug for MatrisEtiketiBiçimleyicisi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MatrisEtiketiBiçimleyicisi(..)")
    }
}

impl PartialEq for MatrisEtiketiBiçimleyicisi {
    fn eq(&self, diğer: &Self) -> bool {
        Arc::ptr_eq(&self.0, &diğer.0)
    }
}

/// Hiyerarşik x/y başlık hücresi. Yapraklar gövde satır/sütunlarını belirler.
#[derive(Clone, Debug, PartialEq)]
pub struct MatrisBoyutHücresi {
    pub değer: String,
    pub boyut: Option<Uzunluk>,
    pub çocuklar: Vec<MatrisBoyutHücresi>,
    pub öğe_stili: Option<ÖğeStili>,
    pub etiket: Option<Etiket>,
    pub etiket_bağlamlı_biçimleyici: Option<MatrisEtiketiBiçimleyicisi>,
    pub sessiz: Option<bool>,
    pub z2: Option<i32>,
}

impl MatrisBoyutHücresi {
    pub fn yeni(değer: impl Into<String>) -> Self {
        Self {
            değer: değer.into(),
            boyut: None,
            çocuklar: Vec::new(),
            öğe_stili: None,
            etiket: None,
            etiket_bağlamlı_biçimleyici: None,
            sessiz: None,
            z2: None,
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

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = Some(etiket);
        self
    }

    pub fn etiket_bağlamlı_biçimleyici(
        mut self,
        biçimleyici: impl Fn(&MatrisEtiketiBağlamı) -> String + Send + Sync + 'static,
    ) -> Self {
        self.etiket_bağlamlı_biçimleyici = Some(MatrisEtiketiBiçimleyicisi::yeni(biçimleyici));
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = Some(sessiz);
        self
    }

    pub fn z2(mut self, z2: i32) -> Self {
        self.z2 = Some(z2);
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
    pub etiket_bağlamlı_biçimleyici: Option<MatrisEtiketiBiçimleyicisi>,
    pub ayırıcı: ÇizgiStili,
    pub sessiz: Option<bool>,
    pub z2: Option<i32>,
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
                // `tokens.color.borderTint`.
                .kenarlık_rengi(Renk::onaltılık(0xcfd2d7))
                .kenarlık_kalınlığı(1.0),
            etiket: Etiket::yeni().göster(true).uzaklık(0.0),
            etiket_bağlamlı_biçimleyici: None,
            ayırıcı: ÇizgiStili::yeni()
                // `tokens.color.border`.
                .renk(Renk::onaltılık(0xb7b9be))
                .kalınlık(1.0),
            sessiz: None,
            z2: None,
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

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn seviye_boyutları(
        mut self,
        boyutlar: impl IntoIterator<Item = Option<Uzunluk>>,
    ) -> Self {
        self.seviye_boyutları = boyutlar.into_iter().collect();
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn etiket_bağlamlı_biçimleyici(
        mut self,
        biçimleyici: impl Fn(&MatrisEtiketiBağlamı) -> String + Send + Sync + 'static,
    ) -> Self {
        self.etiket_bağlamlı_biçimleyici = Some(MatrisEtiketiBiçimleyicisi::yeni(biçimleyici));
        self
    }

    pub fn ayırıcı(mut self, stil: ÇizgiStili) -> Self {
        self.ayırıcı = stil;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = Some(sessiz);
        self
    }

    pub fn z2(mut self, z2: i32) -> Self {
        self.z2 = Some(z2);
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
    pub etiket_bağlamlı_biçimleyici: Option<MatrisEtiketiBiçimleyicisi>,
    pub sessiz: Option<bool>,
    pub koordinatı_sınırla: bool,
    pub z2: Option<i32>,
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
            etiket_bağlamlı_biçimleyici: None,
            sessiz: None,
            koordinatı_sınırla: false,
            z2: None,
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

    pub fn koordinatı_sınırla(mut self, açık: bool) -> Self {
        self.koordinatı_sınırla = açık;
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = Some(etiket);
        self
    }

    pub fn etiket_bağlamlı_biçimleyici(
        mut self,
        biçimleyici: impl Fn(&MatrisEtiketiBağlamı) -> String + Send + Sync + 'static,
    ) -> Self {
        self.etiket_bağlamlı_biçimleyici = Some(MatrisEtiketiBiçimleyicisi::yeni(biçimleyici));
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = Some(sessiz);
        self
    }

    pub fn z2(mut self, z2: i32) -> Self {
        self.z2 = Some(z2);
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
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    pub yatay_ortala: bool,
    pub dikey_ortala: bool,
    pub x: MatrisBoyutu,
    pub y: MatrisBoyutu,
    pub gövde_verisi: Vec<MatrisGövdeHücresi>,
    pub gövde_stili: ÖğeStili,
    pub gövde_etiketi: Etiket,
    pub gövde_etiketi_bağlamlı_biçimleyici: Option<MatrisEtiketiBiçimleyicisi>,
    pub gövde_sessiz: Option<bool>,
    pub gövde_z2: Option<i32>,
    pub köşe_verisi: Vec<MatrisGövdeHücresi>,
    pub köşe_stili: ÖğeStili,
    pub köşe_etiketi: Etiket,
    pub köşe_etiketi_bağlamlı_biçimleyici: Option<MatrisEtiketiBiçimleyicisi>,
    pub köşe_sessiz: Option<bool>,
    pub köşe_z2: Option<i32>,
    pub arkaplan_stili: ÖğeStili,
    pub kenarlık_z2: Option<i32>,
    pub tetikleme_olayı: bool,
}

impl Default for MatrisKoordinatı {
    fn default() -> Self {
        Self {
            sol: Uzunluk::Yüzde(10.0),
            üst: Uzunluk::Yüzde(10.0),
            sağ: Uzunluk::Yüzde(10.0),
            alt: Uzunluk::Yüzde(10.0),
            genişlik: None,
            yükseklik: None,
            yatay_ortala: false,
            dikey_ortala: false,
            x: MatrisBoyutu::default(),
            y: MatrisBoyutu::default(),
            gövde_verisi: Vec::new(),
            gövde_stili: ÖğeStili::yeni()
                // `tokens.color.borderTint`.
                .kenarlık_rengi(Renk::onaltılık(0xcfd2d7))
                .kenarlık_kalınlığı(1.0),
            gövde_etiketi: Etiket::yeni().göster(true).uzaklık(0.0),
            gövde_etiketi_bağlamlı_biçimleyici: None,
            gövde_sessiz: None,
            gövde_z2: None,
            köşe_verisi: Vec::new(),
            köşe_stili: ÖğeStili::yeni(),
            köşe_etiketi: Etiket::yeni().göster(true).uzaklık(0.0),
            köşe_etiketi_bağlamlı_biçimleyici: None,
            köşe_sessiz: None,
            köşe_z2: None,
            arkaplan_stili: ÖğeStili::yeni()
                // `tokens.color.axisLine`.
                .kenarlık_rengi(Renk::onaltılık(0x54555a))
                .kenarlık_kalınlığı(1.0),
            kenarlık_z2: None,
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

    pub fn köşe_hücresi(mut self, hücre: MatrisGövdeHücresi) -> Self {
        self.köşe_verisi.push(hücre);
        self
    }

    pub fn gövde_stili(mut self, stil: ÖğeStili) -> Self {
        self.gövde_stili = stil;
        self
    }

    pub fn gövde_etiketi(mut self, etiket: Etiket) -> Self {
        self.gövde_etiketi = etiket;
        self
    }

    pub fn gövde_etiketi_bağlamlı_biçimleyici(
        mut self,
        biçimleyici: impl Fn(&MatrisEtiketiBağlamı) -> String + Send + Sync + 'static,
    ) -> Self {
        self.gövde_etiketi_bağlamlı_biçimleyici =
            Some(MatrisEtiketiBiçimleyicisi::yeni(biçimleyici));
        self
    }

    pub fn gövde_sessiz(mut self, sessiz: bool) -> Self {
        self.gövde_sessiz = Some(sessiz);
        self
    }

    pub fn gövde_z2(mut self, z2: i32) -> Self {
        self.gövde_z2 = Some(z2);
        self
    }

    pub fn köşe_stili(mut self, stil: ÖğeStili) -> Self {
        self.köşe_stili = stil;
        self
    }

    pub fn köşe_etiketi(mut self, etiket: Etiket) -> Self {
        self.köşe_etiketi = etiket;
        self
    }

    pub fn köşe_etiketi_bağlamlı_biçimleyici(
        mut self,
        biçimleyici: impl Fn(&MatrisEtiketiBağlamı) -> String + Send + Sync + 'static,
    ) -> Self {
        self.köşe_etiketi_bağlamlı_biçimleyici =
            Some(MatrisEtiketiBiçimleyicisi::yeni(biçimleyici));
        self
    }

    pub fn köşe_sessiz(mut self, sessiz: bool) -> Self {
        self.köşe_sessiz = Some(sessiz);
        self
    }

    pub fn köşe_z2(mut self, z2: i32) -> Self {
        self.köşe_z2 = Some(z2);
        self
    }

    pub fn arkaplan_stili(mut self, stil: ÖğeStili) -> Self {
        self.arkaplan_stili = stil;
        self
    }

    pub fn kenarlık_z2(mut self, z2: i32) -> Self {
        self.kenarlık_z2 = Some(z2);
        self
    }

    pub fn tetikleme_olayı(mut self, açık: bool) -> Self {
        self.tetikleme_olayı = açık;
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

    pub fn genişlik(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(değer.into());
        self
    }

    pub fn yükseklik(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(değer.into());
        self
    }

    /// `left: 'center'` karşılığı. Açık genişlik yoksa kalan kutunun
    /// genişliği kullanılır.
    pub fn yatay_ortala(mut self) -> Self {
        self.yatay_ortala = true;
        self
    }

    /// `top: 'middle'` karşılığı.
    pub fn dikey_ortala(mut self) -> Self {
        self.dikey_ortala = true;
        self
    }
}
