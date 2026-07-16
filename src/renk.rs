//! Renk modeli, ECharts renk seçenekleriyle uyumlu metin çözümleme ve gpui
//! boya tiplerine dönüşüm.

use gpui::{Background, Hsla, Rgba, linear_color_stop, linear_gradient};

/// Bileşenleri `0.0..=1.0` aralığında bir KYMA (RGBA) rengi.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Renk {
    pub kırmızı: f32,
    pub yeşil: f32,
    pub mavi: f32,
    pub alfa: f32,
}

impl Renk {
    pub const SAYDAM: Renk = Renk { kırmızı: 0.0, yeşil: 0.0, mavi: 0.0, alfa: 0.0 };
    pub const BEYAZ: Renk = Renk { kırmızı: 1.0, yeşil: 1.0, mavi: 1.0, alfa: 1.0 };
    pub const SİYAH: Renk = Renk { kırmızı: 0.0, yeşil: 0.0, mavi: 0.0, alfa: 1.0 };

    pub const fn kyma(kırmızı: f32, yeşil: f32, mavi: f32, alfa: f32) -> Self {
        Renk { kırmızı, yeşil, mavi, alfa }
    }

    /// `0xRRGGBB` onaltılık değerinden.
    pub const fn onaltılık(değer: u32) -> Self {
        Renk {
            kırmızı: ((değer >> 16) & 0xff) as f32 / 255.0,
            yeşil: ((değer >> 8) & 0xff) as f32 / 255.0,
            mavi: (değer & 0xff) as f32 / 255.0,
            alfa: 1.0,
        }
    }

    /// ECharts tarzı renk metinlerini çözer: `#rgb`, `#rrggbb`, `#rrggbbaa`,
    /// `rgb(r,g,b)`, `rgba(r,g,b,a)` ve birkaç yaygın ad.
    pub fn çöz(metin: &str) -> Option<Self> {
        let s = metin.trim();
        if let Some(onaltı) = s.strip_prefix('#') {
            return match onaltı.len() {
                3 => {
                    let v = u32::from_str_radix(onaltı, 16).ok()?;
                    Some(Renk::kyma(
                        ((v >> 8) & 0xf) as f32 / 15.0,
                        ((v >> 4) & 0xf) as f32 / 15.0,
                        (v & 0xf) as f32 / 15.0,
                        1.0,
                    ))
                }
                6 => Some(Renk::onaltılık(u32::from_str_radix(onaltı, 16).ok()?)),
                8 => {
                    let v = u32::from_str_radix(onaltı, 16).ok()?;
                    Some(Renk::kyma(
                        ((v >> 24) & 0xff) as f32 / 255.0,
                        ((v >> 16) & 0xff) as f32 / 255.0,
                        ((v >> 8) & 0xff) as f32 / 255.0,
                        (v & 0xff) as f32 / 255.0,
                    ))
                }
                _ => None,
            };
        }
        if let Some(gövde) = s
            .strip_prefix("rgba(")
            .or_else(|| s.strip_prefix("rgb("))
            .and_then(|g| g.strip_suffix(')'))
        {
            let parçalar: Vec<f32> = gövde
                .split(',')
                .map(|p| p.trim().parse::<f32>().unwrap_or(0.0))
                .collect();
            if parçalar.len() >= 3 {
                return Some(Renk::kyma(
                    parçalar[0] / 255.0,
                    parçalar[1] / 255.0,
                    parçalar[2] / 255.0,
                    if parçalar.len() > 3 { parçalar[3] } else { 1.0 },
                ));
            }
            return None;
        }
        match s {
            "saydam" | "transparent" | "none" => Some(Renk::SAYDAM),
            "siyah" | "black" => Some(Renk::SİYAH),
            "beyaz" | "white" => Some(Renk::BEYAZ),
            "kırmızı" | "red" => Some(Renk::onaltılık(0xff0000)),
            "yeşil" | "green" => Some(Renk::onaltılık(0x008000)),
            "mavi" | "blue" => Some(Renk::onaltılık(0x0000ff)),
            "sarı" | "yellow" => Some(Renk::onaltılık(0xffff00)),
            "gri" | "gray" | "grey" => Some(Renk::onaltılık(0x808080)),
            _ => None,
        }
    }

    pub fn alfa_ile(mut self, alfa: f32) -> Self {
        self.alfa = alfa;
        self
    }

    pub fn opaklık(mut self, çarpan: f32) -> Self {
        self.alfa *= çarpan;
        self
    }

    /// İki renk arasında doğrusal ara değer.
    pub fn karıştır(self, diğer: Renk, t: f32) -> Renk {
        let t = t.clamp(0.0, 1.0);
        Renk {
            kırmızı: self.kırmızı + (diğer.kırmızı - self.kırmızı) * t,
            yeşil: self.yeşil + (diğer.yeşil - self.yeşil) * t,
            mavi: self.mavi + (diğer.mavi - self.mavi) * t,
            alfa: self.alfa + (diğer.alfa - self.alfa) * t,
        }
    }

    pub fn gpui_rgba(self) -> Rgba {
        Rgba { r: self.kırmızı, g: self.yeşil, b: self.mavi, a: self.alfa }
    }

    pub fn gpui_hsla(self) -> Hsla {
        Hsla::from(self.gpui_rgba())
    }
}

impl From<u32> for Renk {
    fn from(değer: u32) -> Self {
        Renk::onaltılık(değer)
    }
}

impl From<&str> for Renk {
    fn from(s: &str) -> Self {
        Renk::çöz(s).unwrap_or(Renk::SİYAH)
    }
}

impl From<Renk> for Hsla {
    fn from(r: Renk) -> Hsla {
        r.gpui_hsla()
    }
}

impl From<Renk> for Background {
    fn from(r: Renk) -> Background {
        r.gpui_hsla().into()
    }
}

/// Gradyanın bir durağı; `konum` `0.0..=1.0` aralığındadır.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RenkDurağı {
    pub konum: f32,
    pub renk: Renk,
}

impl RenkDurağı {
    pub fn yeni(konum: f32, renk: impl Into<Renk>) -> Self {
        RenkDurağı { konum, renk: renk.into() }
    }
}

/// Dolgu boyası: düz renk ya da doğrusal gradyan. ECharts'taki
/// `color: '#abc' | new graphic.LinearGradient(x, y, x2, y2, duraklar)`
/// seçeneğinin karşılığı.
#[derive(Clone, PartialEq, Debug)]
pub enum Dolgu {
    Düz(Renk),
    /// Doğrusal gradyan. `(x, y)` → `(x2, y2)` uçları, tıpkı
    /// `echarts.graphic.LinearGradient` gibi öğenin birim sınır kutusundadır.
    DoğrusalGradyan {
        x: f32,
        y: f32,
        x2: f32,
        y2: f32,
        duraklar: Vec<RenkDurağı>,
    },
}

impl Dolgu {
    pub fn doğrusal(x: f32, y: f32, x2: f32, y2: f32, duraklar: Vec<RenkDurağı>) -> Self {
        Dolgu::DoğrusalGradyan { x, y, x2, y2, duraklar }
    }

    /// Temsilî düz renk (gradyanlarda ilk durak) — gösterge imleri ve ipucu
    /// noktaları için kullanılır.
    pub fn temsilî(&self) -> Renk {
        match self {
            Dolgu::Düz(r) => *r,
            Dolgu::DoğrusalGradyan { duraklar, .. } => {
                duraklar.first().map(|d| d.renk).unwrap_or(Renk::SİYAH)
            }
        }
    }

    pub fn opaklık(&self, çarpan: f32) -> Dolgu {
        match self {
            Dolgu::Düz(r) => Dolgu::Düz(r.opaklık(çarpan)),
            Dolgu::DoğrusalGradyan { x, y, x2, y2, duraklar } => Dolgu::DoğrusalGradyan {
                x: *x,
                y: *y,
                x2: *x2,
                y2: *y2,
                duraklar: duraklar
                    .iter()
                    .map(|d| RenkDurağı { konum: d.konum, renk: d.renk.opaklık(çarpan) })
                    .collect(),
            },
        }
    }

    /// gpui [`Background`] tipine dönüştürür.
    ///
    /// gpui doğal olarak iki duraklı doğrusal gradyan destekler; daha çok
    /// duraklı gradyanlar ilk ve son durakla yaklaşıklanır.
    pub fn gpui_arkaplan(&self) -> Background {
        match self {
            Dolgu::Düz(r) => r.gpui_hsla().into(),
            Dolgu::DoğrusalGradyan { x, y, x2, y2, duraklar } => {
                if duraklar.is_empty() {
                    return Renk::SAYDAM.gpui_hsla().into();
                }
                if duraklar.len() == 1 {
                    return duraklar[0].renk.gpui_hsla().into();
                }
                let ilk = duraklar.first().unwrap();
                let son = duraklar.last().unwrap();
                // gpui açısı: 0° yukarıyı gösterir, saat yönünde artar.
                let dx = (x2 - x) as f64;
                let dy = (y2 - y) as f64;
                let açı = dx.atan2(-dy).to_degrees() as f32;
                linear_gradient(
                    açı,
                    linear_color_stop(ilk.renk.gpui_hsla(), ilk.konum),
                    linear_color_stop(son.renk.gpui_hsla(), son.konum),
                )
            }
        }
    }
}

impl From<Renk> for Dolgu {
    fn from(r: Renk) -> Dolgu {
        Dolgu::Düz(r)
    }
}

impl From<u32> for Dolgu {
    fn from(değer: u32) -> Dolgu {
        Dolgu::Düz(Renk::onaltılık(değer))
    }
}

impl From<&str> for Dolgu {
    fn from(s: &str) -> Dolgu {
        Dolgu::Düz(Renk::from(s))
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn onaltılık_çözümleme() {
        assert_eq!(Renk::çöz("#ff0000"), Some(Renk::onaltılık(0xff0000)));
        assert_eq!(Renk::çöz("#f00"), Some(Renk::kyma(1.0, 0.0, 0.0, 1.0)));
    }

    #[test]
    fn kyma_çözümleme() {
        assert_eq!(
            Renk::çöz("rgba(255, 0, 0, 0.5)"),
            Some(Renk::kyma(1.0, 0.0, 0.0, 0.5))
        );
    }
}
