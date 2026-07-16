//! Stil seçenekleri — ECharts'taki `lineStyle`, `itemStyle`, `areaStyle`,
//! `textStyle` ve `label` tanımlarının karşılığı.

use std::fmt;
use std::sync::Arc;

use crate::renk::{Dolgu, Renk};

/// Çizgi türü (`lineStyle.type`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ÇizgiTürü {
    #[default]
    Düz,
    Kesikli,
    Noktalı,
}

/// Çizgi stili (`lineStyle`).
#[derive(Clone, PartialEq, Debug)]
pub struct ÇizgiStili {
    pub renk: Option<Renk>,
    pub kalınlık: f32,
    pub tür: ÇizgiTürü,
    pub opaklık: f32,
}

impl Default for ÇizgiStili {
    fn default() -> Self {
        ÇizgiStili { renk: None, kalınlık: 2.0, tür: ÇizgiTürü::Düz, opaklık: 1.0 }
    }
}

impl ÇizgiStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn kalınlık(mut self, kalınlık: f32) -> Self {
        self.kalınlık = kalınlık;
        self
    }

    pub fn tür(mut self, tür: ÇizgiTürü) -> Self {
        self.tür = tür;
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = opaklık;
        self
    }
}

/// Öğe stili (`itemStyle`): sembol, sütun, dilim vb. dolgusu ve kenarlığı.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ÖğeStili {
    pub renk: Option<Dolgu>,
    pub kenarlık_rengi: Option<Renk>,
    pub kenarlık_kalınlığı: f32,
    /// Köşe yarıçapları: `[sol üst, sağ üst, sağ alt, sol alt]`
    /// (`itemStyle.borderRadius`).
    pub kenarlık_yarıçapı: [f32; 4],
    pub opaklık: Option<f32>,
}

impl ÖğeStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn renk(mut self, renk: impl Into<Dolgu>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn kenarlık_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.kenarlık_rengi = Some(renk.into());
        self
    }

    pub fn kenarlık_kalınlığı(mut self, kalınlık: f32) -> Self {
        self.kenarlık_kalınlığı = kalınlık;
        self
    }

    pub fn kenarlık_yarıçapı(mut self, yarıçap: impl Into<KöşeYarıçapı>) -> Self {
        self.kenarlık_yarıçapı = yarıçap.into().0;
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = Some(opaklık);
        self
    }
}

/// Köşe yarıçapı belirtimi: tek sayı ya da dörtlü dizi.
pub struct KöşeYarıçapı(pub [f32; 4]);

impl From<f32> for KöşeYarıçapı {
    fn from(hepsi: f32) -> Self {
        KöşeYarıçapı([hepsi; 4])
    }
}

impl From<[f32; 4]> for KöşeYarıçapı {
    fn from(dört: [f32; 4]) -> Self {
        KöşeYarıçapı(dört)
    }
}

/// Alan stili (`areaStyle`).
#[derive(Clone, PartialEq, Debug)]
pub struct AlanStili {
    pub renk: Option<Dolgu>,
    /// ECharts öntanımlısı 0.7'dir.
    pub opaklık: f32,
}

impl Default for AlanStili {
    fn default() -> Self {
        AlanStili { renk: None, opaklık: 0.7 }
    }
}

impl AlanStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn renk(mut self, renk: impl Into<Dolgu>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = opaklık;
        self
    }
}

/// Yazı stili (`textStyle`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct YazıStili {
    pub renk: Option<Renk>,
    pub boyut: Option<f32>,
    pub kalın: bool,
    pub aile: Option<String>,
}

impl YazıStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn boyut(mut self, boyut: f32) -> Self {
        self.boyut = Some(boyut);
        self
    }

    pub fn kalın(mut self, kalın: bool) -> Self {
        self.kalın = kalın;
        self
    }

    pub fn aile(mut self, aile: impl Into<String>) -> Self {
        self.aile = Some(aile.into());
        self
    }
}

/// Etiket konumu (`label.position`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EtiketKonumu {
    #[default]
    Üst,
    İç,
    Dış,
    Merkez,
    Alt,
}

/// Değer biçimleyici: `"{value} °C"` tarzı şablon ya da işlev.
#[derive(Clone)]
pub enum Biçimleyici {
    /// `{value}` yer tutucusu değerle değiştirilir; kategori eksenlerinde
    /// `{value}` kategori adıdır.
    Şablon(String),
    İşlev(Arc<dyn Fn(f64, &str) -> String + Send + Sync>),
}

impl Biçimleyici {
    pub fn uygula(&self, değer: f64, metin: &str) -> String {
        match self {
            Biçimleyici::Şablon(ş) => ş.replace("{value}", metin),
            Biçimleyici::İşlev(f) => f(değer, metin),
        }
    }
}

impl fmt::Debug for Biçimleyici {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Biçimleyici::Şablon(ş) => f.debug_tuple("Şablon").field(ş).finish(),
            Biçimleyici::İşlev(_) => f.write_str("İşlev(..)"),
        }
    }
}

impl PartialEq for Biçimleyici {
    fn eq(&self, diğer: &Self) -> bool {
        match (self, diğer) {
            (Biçimleyici::Şablon(a), Biçimleyici::Şablon(b)) => a == b,
            (Biçimleyici::İşlev(a), Biçimleyici::İşlev(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl From<&str> for Biçimleyici {
    fn from(ş: &str) -> Self {
        Biçimleyici::Şablon(ş.to_string())
    }
}

impl From<String> for Biçimleyici {
    fn from(ş: String) -> Self {
        Biçimleyici::Şablon(ş)
    }
}

/// Veri etiketi (`label`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Etiket {
    pub göster: bool,
    pub konum: EtiketKonumu,
    pub biçimleyici: Option<Biçimleyici>,
    pub yazı: YazıStili,
}

impl Etiket {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn konum(mut self, konum: EtiketKonumu) -> Self {
        self.konum = konum;
        self
    }

    pub fn biçimleyici(mut self, b: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(b.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }
}
