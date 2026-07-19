//! Veri değerleri — ECharts serilerindeki `data` girdilerinin karşılığı.

use crate::model::stil::{EtiketYaması, ÖğeStili};

/// Tek bir veri değeri. ECharts `null` (boş) değerleri destekler; kartezyen
/// olmayan seriler için `(x, y)` çifti de tutabilir.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum VeriDeğeri {
    #[default]
    Boş,
    Sayı(f64),
    /// `[x, y]` biçiminde çift (saçılım / değer-değer eksenleri).
    Çift([f64; 2]),
    /// Çok değerli öğe: mum `[açılış, kapanış, en düşük, en yüksek]`,
    /// kutu `[en düşük, Ç1, ortanca, Ç3, en yüksek]`.
    Dizi(Vec<f64>),
    Metin(String),
    Mantıksal(bool),
    /// Unix milisaniyesi olarak zaman. Ayrı tür tutulması, sayı sütunuyla
    /// otomatik zaman boyutunu birbirine karıştırmaz.
    Zaman(i64),
}

impl VeriDeğeri {
    /// Birincil sayısal değer (çiftlerde `y`; çok değerli dizilerde yok —
    /// yorum seriye aittir).
    pub fn sayı(&self) -> Option<f64> {
        match self {
            VeriDeğeri::Sayı(s) => Some(*s),
            VeriDeğeri::Çift([_, y]) => Some(*y),
            VeriDeğeri::Dizi(_) => None,
            VeriDeğeri::Metin(m) => m.parse().ok(),
            VeriDeğeri::Mantıksal(değer) => Some(if *değer { 1.0 } else { 0.0 }),
            VeriDeğeri::Zaman(ms) => Some(*ms as f64),
            VeriDeğeri::Boş => None,
        }
    }

    /// Çok değerli dizi içeriği.
    pub fn dizi(&self) -> Option<&[f64]> {
        match self {
            VeriDeğeri::Dizi(d) => Some(d),
            _ => None,
        }
    }

    /// Çift değerin `x` bileşeni.
    pub fn x(&self) -> Option<f64> {
        match self {
            VeriDeğeri::Çift([x, _]) => Some(*x),
            _ => None,
        }
    }

    pub fn boş_mu(&self) -> bool {
        match self {
            VeriDeğeri::Boş => true,
            VeriDeğeri::Sayı(s) => s.is_nan(),
            VeriDeğeri::Çift([x, y]) => x.is_nan() || y.is_nan(),
            VeriDeğeri::Dizi(d) => d.is_empty() || d.iter().any(|v| v.is_nan()),
            VeriDeğeri::Metin(_) | VeriDeğeri::Mantıksal(_) | VeriDeğeri::Zaman(_) => false,
        }
    }
}

/// Tek bir veri öğesi: değer + isteğe bağlı ad ve stil
/// (ECharts'taki `{ value, name, itemStyle }` nesne biçimi).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct VeriÖğesi {
    pub ad: Option<String>,
    pub değer: VeriDeğeri,
    /// Dataset'ten gelen bütün boyutlar. Seri koordinat değeri dışında kalan
    /// boyutlar visualMap, tooltip ve encode kanallarında kaybolmaz.
    pub boyutlar: Vec<(String, VeriDeğeri)>,
    pub stil: Option<ÖğeStili>,
    /// Öğeye özgü etiket seçenekleri (`data[i].label`). Seri etiketini
    /// geçersiz kılmak isteyen nesne biçimli verilerde kullanılır.
    pub etiket: Option<EtiketYaması>,
    /// İlk veri seçim durumu (`data[i].selected`).
    pub seçili: bool,
}

impl VeriÖğesi {
    pub fn yeni(değer: impl Into<VeriDeğeri>) -> Self {
        VeriÖğesi {
            ad: None,
            değer: değer.into(),
            boyutlar: Vec::new(),
            stil: None,
            etiket: None,
            seçili: false,
        }
    }

    pub fn adlı(ad: impl Into<String>, değer: impl Into<VeriDeğeri>) -> Self {
        VeriÖğesi {
            ad: Some(ad.into()),
            değer: değer.into(),
            boyutlar: Vec::new(),
            stil: None,
            etiket: None,
            seçili: false,
        }
    }

    pub fn stil(mut self, stil: ÖğeStili) -> Self {
        self.stil = Some(stil);
        self
    }

    pub fn boyutlar(mut self, boyutlar: impl IntoIterator<Item = (String, VeriDeğeri)>) -> Self {
        self.boyutlar = boyutlar.into_iter().collect();
        self
    }

    pub fn boyut(&self, ad: &str) -> Option<&VeriDeğeri> {
        self.boyutlar
            .iter()
            .find(|(boyut_adı, _)| boyut_adı == ad)
            .map(|(_, değer)| değer)
    }

    pub fn etiket(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.etiket = Some(etiket.into());
        self
    }

    pub fn seçili(mut self, seçili: bool) -> Self {
        self.seçili = seçili;
        self
    }
}

impl From<f64> for VeriDeğeri {
    fn from(v: f64) -> Self {
        if v.is_nan() {
            VeriDeğeri::Boş
        } else {
            VeriDeğeri::Sayı(v)
        }
    }
}

impl From<f32> for VeriDeğeri {
    fn from(v: f32) -> Self {
        VeriDeğeri::from(v as f64)
    }
}

impl From<i32> for VeriDeğeri {
    fn from(v: i32) -> Self {
        VeriDeğeri::Sayı(v as f64)
    }
}

impl From<u32> for VeriDeğeri {
    fn from(v: u32) -> Self {
        VeriDeğeri::Sayı(v as f64)
    }
}

impl From<i64> for VeriDeğeri {
    fn from(v: i64) -> Self {
        VeriDeğeri::Sayı(v as f64)
    }
}

impl From<bool> for VeriDeğeri {
    fn from(v: bool) -> Self {
        VeriDeğeri::Mantıksal(v)
    }
}

impl From<&str> for VeriDeğeri {
    fn from(v: &str) -> Self {
        VeriDeğeri::Metin(v.to_owned())
    }
}

impl From<String> for VeriDeğeri {
    fn from(v: String) -> Self {
        VeriDeğeri::Metin(v)
    }
}

impl From<[f64; 2]> for VeriDeğeri {
    fn from(v: [f64; 2]) -> Self {
        VeriDeğeri::Çift(v)
    }
}

impl From<(f64, f64)> for VeriDeğeri {
    fn from((x, y): (f64, f64)) -> Self {
        VeriDeğeri::Çift([x, y])
    }
}

impl From<[f64; 3]> for VeriDeğeri {
    fn from(d: [f64; 3]) -> Self {
        VeriDeğeri::Dizi(d.to_vec())
    }
}

impl From<[f64; 4]> for VeriDeğeri {
    fn from(d: [f64; 4]) -> Self {
        VeriDeğeri::Dizi(d.to_vec())
    }
}

impl From<[f64; 5]> for VeriDeğeri {
    fn from(d: [f64; 5]) -> Self {
        VeriDeğeri::Dizi(d.to_vec())
    }
}

impl From<Vec<f64>> for VeriDeğeri {
    fn from(d: Vec<f64>) -> Self {
        VeriDeğeri::Dizi(d)
    }
}

impl<T: Into<VeriDeğeri>> From<Option<T>> for VeriDeğeri {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(iç) => iç.into(),
            None => VeriDeğeri::Boş,
        }
    }
}

impl<T: Into<VeriDeğeri>> From<T> for VeriÖğesi {
    fn from(v: T) -> Self {
        VeriÖğesi::yeni(v)
    }
}

impl From<(&str, f64)> for VeriÖğesi {
    fn from((ad, değer): (&str, f64)) -> Self {
        VeriÖğesi::adlı(ad, değer)
    }
}

impl From<(String, f64)> for VeriÖğesi {
    fn from((ad, değer): (String, f64)) -> Self {
        VeriÖğesi::adlı(ad, değer)
    }
}

/// Bir dizi girdiyi `Vec<VeriÖğesi>`ne çevirir.
pub fn veri_listesi<T: Into<VeriÖğesi>>(
    girdiler: impl IntoIterator<Item = T>,
) -> Vec<VeriÖğesi> {
    girdiler.into_iter().map(Into::into).collect()
}
