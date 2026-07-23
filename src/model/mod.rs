//! Seçenek modeli — ECharts'ın bildirime dayalı `option` API'sinin Rust
//! karşılığı (`echarts/src/model` ve bileşen tanımları).

pub mod agac;
pub mod bilesen;
pub mod deger;
pub mod eksen;
pub mod gorsel_esleme;
pub mod grafik_bileseni;
pub mod hatlar;
pub mod imleyici;
pub mod kiris;
pub mod kutupsal;
pub mod matris;
pub mod paralel;
pub mod radar;
pub mod sankey;
pub mod secenekler;
pub mod seri;
pub mod stil;
pub mod takvim;
pub mod tek_eksen;
pub mod veri_kumesi;
pub mod yakinlastirma;
pub mod zaman_seridi;

use crate::yardimci::sayi::yüzde_çöz;

/// Piksel ya da yüzde cinsinden uzunluk; ECharts'taki `10 | '10%'`
/// seçeneklerinin karşılığı.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Uzunluk {
    Piksel(f32),
    /// `Yüzde(10.0)` = `'10%'`.
    Yüzde(f32),
}

impl Uzunluk {
    /// `bütün`e göre piksel değerini çözer.
    pub fn çöz(&self, bütün: f32) -> f32 {
        match self {
            Uzunluk::Piksel(p) => *p,
            Uzunluk::Yüzde(y) => y / 100.0 * bütün,
        }
    }
}

impl From<f32> for Uzunluk {
    fn from(p: f32) -> Self {
        Uzunluk::Piksel(p)
    }
}

impl From<i32> for Uzunluk {
    fn from(p: i32) -> Self {
        Uzunluk::Piksel(p as f32)
    }
}

impl From<&str> for Uzunluk {
    fn from(s: &str) -> Self {
        let s = s.trim();
        if s.ends_with('%') {
            Uzunluk::Yüzde((yüzde_çöz(s, 100.0)) as f32)
        } else {
            Uzunluk::Piksel(s.parse::<f32>().unwrap_or(0.0))
        }
    }
}

/// Yatay hizalama / konum belirtimi (`left: 'center'` vb.).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum YatayKonum {
    #[default]
    Sol,
    Orta,
    Sağ,
    Değer(Uzunluk),
}

impl From<Uzunluk> for YatayKonum {
    fn from(u: Uzunluk) -> Self {
        YatayKonum::Değer(u)
    }
}

impl From<f32> for YatayKonum {
    fn from(p: f32) -> Self {
        YatayKonum::Değer(Uzunluk::Piksel(p))
    }
}

impl From<&str> for YatayKonum {
    fn from(s: &str) -> Self {
        match s.trim() {
            "sol" | "left" => YatayKonum::Sol,
            "orta" | "center" | "middle" => YatayKonum::Orta,
            "sağ" | "right" => YatayKonum::Sağ,
            diğer => YatayKonum::Değer(Uzunluk::from(diğer)),
        }
    }
}

/// Dikey hizalama / konum belirtimi (`top: 'center'` vb.).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum DikeyKonum {
    #[default]
    Üst,
    Orta,
    Alt,
    Değer(Uzunluk),
}

impl From<Uzunluk> for DikeyKonum {
    fn from(u: Uzunluk) -> Self {
        DikeyKonum::Değer(u)
    }
}

impl From<f32> for DikeyKonum {
    fn from(p: f32) -> Self {
        DikeyKonum::Değer(Uzunluk::Piksel(p))
    }
}

impl From<i32> for DikeyKonum {
    fn from(p: i32) -> Self {
        DikeyKonum::Değer(Uzunluk::Piksel(p as f32))
    }
}

impl From<&str> for DikeyKonum {
    fn from(s: &str) -> Self {
        match s.trim() {
            "üst" | "top" => DikeyKonum::Üst,
            "orta" | "center" | "middle" => DikeyKonum::Orta,
            "alt" | "bottom" => DikeyKonum::Alt,
            diğer => DikeyKonum::Değer(Uzunluk::from(diğer)),
        }
    }
}
