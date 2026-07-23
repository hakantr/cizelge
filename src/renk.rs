//! Renk modeli, ECharts renk seçenekleriyle uyumlu metin çözümleme ve gpui
//! boya tiplerine dönüşüm.

use std::sync::Arc;

#[cfg(feature = "gpui")]
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
    pub const SAYDAM: Renk = Renk {
        kırmızı: 0.0,
        yeşil: 0.0,
        mavi: 0.0,
        alfa: 0.0,
    };
    pub const BEYAZ: Renk = Renk {
        kırmızı: 1.0,
        yeşil: 1.0,
        mavi: 1.0,
        alfa: 1.0,
    };
    pub const SİYAH: Renk = Renk {
        kırmızı: 0.0,
        yeşil: 0.0,
        mavi: 0.0,
        alfa: 1.0,
    };

    pub const fn kyma(kırmızı: f32, yeşil: f32, mavi: f32, alfa: f32) -> Self {
        Renk {
            kırmızı,
            yeşil,
            mavi,
            alfa,
        }
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
            if let [kırmızı, yeşil, mavi, kalan @ ..] = parçalar.as_slice() {
                return Some(Renk::kyma(
                    kırmızı / 255.0,
                    yeşil / 255.0,
                    mavi / 255.0,
                    // CSS Color, 1'in üzerindeki alfa kanalını tam opaklığa
                    // sıkıştırır. ECharts örnek verisinde `rgba(...,255)`
                    // biçimi de bulunduğu için zrender davranışını koru.
                    kalan.first().copied().unwrap_or(1.0).clamp(0.0, 1.0),
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

    /// zrender `Path#getInsideTextFill/Stroke` karşılığı. Dolgu
    /// parlaklığına göre iç etiket rengini ve gerekirse otomatik
    /// konturun rengini döndürür.
    pub fn zrender_iç_etiket_stili(self, koyu_kip: bool) -> (Renk, Option<Renk>) {
        let parlaklık = (0.299 * self.kırmızı + 0.587 * self.yeşil + 0.114 * self.mavi) * self.alfa;
        let metin = if parlaklık > 0.5 {
            Renk::onaltılık(0x333333)
        } else if parlaklık > 0.2 {
            Renk::onaltılık(0xeeeeee)
        } else {
            Renk::onaltılık(0xcccccc)
        };
        let metin_parlaklığı = 0.299 * metin.kırmızı + 0.587 * metin.yeşil + 0.114 * metin.mavi;
        let metin_koyu = metin_parlaklığı < 0.4;
        let kontur = (koyu_kip == metin_koyu).then_some(self);
        (metin, kontur)
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

    /// Rengin HSL açıklığını (`colorLightness`) değiştirir. Ton, doygunluk
    /// ve alfa korunur; açıklık ECharts/zrender gibi `0..=1` aralığına
    /// sıkıştırılır.
    pub fn açıklık_ile(self, açıklık: f32) -> Renk {
        let en_büyük = self.kırmızı.max(self.yeşil).max(self.mavi);
        let en_küçük = self.kırmızı.min(self.yeşil).min(self.mavi);
        let fark = en_büyük - en_küçük;
        let eski_açıklık = (en_büyük + en_küçük) / 2.0;
        let doygunluk = if fark <= f32::EPSILON {
            0.0
        } else {
            fark / (1.0 - (2.0 * eski_açıklık - 1.0).abs()).max(f32::EPSILON)
        };
        let ton = if fark <= f32::EPSILON {
            0.0
        } else if (en_büyük - self.kırmızı).abs() <= f32::EPSILON {
            ((self.yeşil - self.mavi) / fark).rem_euclid(6.0)
        } else if (en_büyük - self.yeşil).abs() <= f32::EPSILON {
            (self.mavi - self.kırmızı) / fark + 2.0
        } else {
            (self.kırmızı - self.yeşil) / fark + 4.0
        };
        let açıklık = açıklık.clamp(0.0, 1.0);
        let kroma = (1.0 - (2.0 * açıklık - 1.0).abs()) * doygunluk;
        let x = kroma * (1.0 - (ton.rem_euclid(2.0) - 1.0).abs());
        let (kırmızı, yeşil, mavi) = match ton {
            t if t < 1.0 => (kroma, x, 0.0),
            t if t < 2.0 => (x, kroma, 0.0),
            t if t < 3.0 => (0.0, kroma, x),
            t if t < 4.0 => (0.0, x, kroma),
            t if t < 5.0 => (x, 0.0, kroma),
            _ => (kroma, 0.0, x),
        };
        let eşik = açıklık - kroma / 2.0;
        // zrender `modifyHSL`, sonucu `rgba(...)` metnine çevirirken RGB
        // kanallarını CSS baytına yuvarlar. Bu ara rengin daha sonra başka
        // bir visualMap HSL kanalına girdiği zincirlerde yuvarlamayı burada
        // korumak gerekir.
        let bayt = |kanal: f32| (kanal.clamp(0.0, 1.0) * 255.0).round() / 255.0;
        Renk::kyma(
            bayt(kırmızı + eşik),
            bayt(yeşil + eşik),
            bayt(mavi + eşik),
            self.alfa,
        )
    }

    /// Rengin HSL doygunluğunu (`colorSaturation`) değiştirir; ton,
    /// açıklık ve alfa korunur. zrender'ın `modifyHSL` kanal sırası ve CSS
    /// bayt yuvarlamasıyla aynı kararlı sonucu üretir.
    pub fn doygunluk_ile(self, doygunluk: f32) -> Renk {
        let en_büyük = self.kırmızı.max(self.yeşil).max(self.mavi);
        let en_küçük = self.kırmızı.min(self.yeşil).min(self.mavi);
        let fark = en_büyük - en_küçük;
        let açıklık = (en_büyük + en_küçük) / 2.0;
        let ton = if fark <= f32::EPSILON {
            0.0
        } else if (en_büyük - self.kırmızı).abs() <= f32::EPSILON {
            ((self.yeşil - self.mavi) / fark).rem_euclid(6.0)
        } else if (en_büyük - self.yeşil).abs() <= f32::EPSILON {
            (self.mavi - self.kırmızı) / fark + 2.0
        } else {
            (self.kırmızı - self.yeşil) / fark + 4.0
        };
        let doygunluk = doygunluk.clamp(0.0, 1.0);
        let kroma = (1.0 - (2.0 * açıklık - 1.0).abs()) * doygunluk;
        let x = kroma * (1.0 - (ton.rem_euclid(2.0) - 1.0).abs());
        let (kırmızı, yeşil, mavi) = match ton {
            t if t < 1.0 => (kroma, x, 0.0),
            t if t < 2.0 => (x, kroma, 0.0),
            t if t < 3.0 => (0.0, kroma, x),
            t if t < 4.0 => (0.0, x, kroma),
            t if t < 5.0 => (x, 0.0, kroma),
            _ => (kroma, 0.0, x),
        };
        let eşik = açıklık - kroma / 2.0;
        let bayt = |kanal: f32| (kanal.clamp(0.0, 1.0) * 255.0).round() / 255.0;
        Renk::kyma(
            bayt(kırmızı + eşik),
            bayt(yeşil + eşik),
            bayt(mavi + eşik),
            self.alfa,
        )
    }

    /// Rengin HSL tonunu derece cinsinden mutlak olarak değiştirir.
    /// ECharts/zrender `color.modifyHSL(color, hue)` davranışını izleyerek
    /// dereceyi en yakın tam sayıya yuvarlar ve `0..=360` aralığına kıstırır;
    /// doygunluk, açıklık ve alfa korunur.
    pub fn ton_ile(self, derece: f32) -> Renk {
        let en_büyük = self.kırmızı.max(self.yeşil).max(self.mavi);
        let en_küçük = self.kırmızı.min(self.yeşil).min(self.mavi);
        let fark = en_büyük - en_küçük;
        let açıklık = (en_büyük + en_küçük) / 2.0;
        let doygunluk = if fark <= f32::EPSILON {
            0.0
        } else {
            fark / (1.0 - (2.0 * açıklık - 1.0).abs()).max(f32::EPSILON)
        };
        let ton = derece.round().clamp(0.0, 360.0).rem_euclid(360.0) / 60.0;
        let kroma = (1.0 - (2.0 * açıklık - 1.0).abs()) * doygunluk;
        let x = kroma * (1.0 - (ton.rem_euclid(2.0) - 1.0).abs());
        let (kırmızı, yeşil, mavi) = match ton {
            t if t < 1.0 => (kroma, x, 0.0),
            t if t < 2.0 => (x, kroma, 0.0),
            t if t < 3.0 => (0.0, kroma, x),
            t if t < 4.0 => (0.0, x, kroma),
            t if t < 5.0 => (x, 0.0, kroma),
            _ => (kroma, 0.0, x),
        };
        let eşik = açıklık - kroma / 2.0;
        let bayt = |kanal: f32| (kanal.clamp(0.0, 1.0) * 255.0).round() / 255.0;
        Renk::kyma(
            bayt(kırmızı + eşik),
            bayt(yeşil + eşik),
            bayt(mavi + eşik),
            self.alfa,
        )
    }

    #[cfg(feature = "gpui")]
    pub fn gpui_rgba(self) -> Rgba {
        Rgba {
            r: self.kırmızı,
            g: self.yeşil,
            b: self.mavi,
            a: self.alfa,
        }
    }

    #[cfg(feature = "gpui")]
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

#[cfg(feature = "gpui")]
impl From<Renk> for Hsla {
    fn from(r: Renk) -> Hsla {
        r.gpui_hsla()
    }
}

#[cfg(feature = "gpui")]
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

/// Canvas `CanvasPattern.repeat` seçenekleri.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DesenTekrarı {
    #[default]
    Tekrar,
    Yatay,
    Dikey,
    Tek,
    /// ECharts/zrender rich-text `backgroundColor: { image }` davranışı:
    /// görüntüyü hedef kutunun tamamına esnetir.
    Sığdır,
}

/// Renderer'lardan bağımsız, önden çarpılmış RGBA8 görüntü deseni.
///
/// Piksel verisi kurucuda doğrulanır ve premultiplied biçime çevrilir. Bu
/// sayede raster yüzey aynı baytları doğrudan görüntü gölgelendiricisine
/// verebilir; SVG yüzeyi de kayıpsız olarak gömebilir.
#[derive(Clone, PartialEq, Debug)]
pub struct GörüntüDeseni {
    pub genişlik: u32,
    pub yükseklik: u32,
    pub pikseller: Arc<[u8]>,
    pub tekrar: DesenTekrarı,
    pub opaklık: f32,
}

impl GörüntüDeseni {
    /// Düz (premultiplied olmayan) RGBA8 baytlarından desen kurar. Boyutla
    /// uyuşmayan veri `None` döndürür; çalışma zamanı panik üretmez.
    pub fn rgba(
        genişlik: u32,
        yükseklik: u32,
        mut pikseller: Vec<u8>,
        tekrar: DesenTekrarı,
    ) -> Option<Self> {
        let beklenen = usize::try_from(genişlik)
            .ok()?
            .checked_mul(usize::try_from(yükseklik).ok()?)?
            .checked_mul(4)?;
        if genişlik == 0 || yükseklik == 0 || pikseller.len() != beklenen {
            return None;
        }
        for piksel in pikseller.chunks_exact_mut(4) {
            if let [kırmızı, yeşil, mavi, alfa] = piksel {
                let alfa_değeri = u16::from(*alfa);
                for kanal in [kırmızı, yeşil, mavi] {
                    *kanal = ((u16::from(*kanal) * alfa_değeri + 127) / 255) as u8;
                }
            }
        }
        Some(Self {
            genişlik,
            yükseklik,
            pikseller: pikseller.into(),
            tekrar,
            opaklık: 1.0,
        })
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = opaklık.clamp(0.0, 1.0);
        self
    }

    fn temsilî(&self) -> Renk {
        let Some([kırmızı, yeşil, mavi, alfa]) = self.pikseller.get(0..4) else {
            return Renk::SAYDAM;
        };
        let alfa = f32::from(*alfa) / 255.0;
        if alfa <= f32::EPSILON {
            return Renk::SAYDAM;
        }
        Renk::kyma(
            (f32::from(*kırmızı) / 255.0 / alfa).clamp(0.0, 1.0),
            (f32::from(*yeşil) / 255.0 / alfa).clamp(0.0, 1.0),
            (f32::from(*mavi) / 255.0 / alfa).clamp(0.0, 1.0),
            alfa * self.opaklık,
        )
    }
}

impl RenkDurağı {
    pub fn yeni(konum: f32, renk: impl Into<Renk>) -> Self {
        RenkDurağı {
            konum,
            renk: renk.into(),
        }
    }
}

/// Dolgu boyası: düz renk, doğrusal ya da radyal gradyan. ECharts'taki
/// `color: '#abc' | new graphic.LinearGradient(...) | new graphic.RadialGradient(...)`
/// seçeneğinin karşılığı.
#[derive(Clone, PartialEq, Debug)]
pub enum Dolgu {
    Düz(Renk),
    /// Canvas `createPattern(image, repeat)` karşılığı.
    Desen(GörüntüDeseni),
    /// Doğrusal gradyan. `(x, y)` → `(x2, y2)` uçları, tıpkı
    /// `echarts.graphic.LinearGradient` gibi öğenin birim sınır kutusundadır.
    DoğrusalGradyan {
        x: f32,
        y: f32,
        x2: f32,
        y2: f32,
        duraklar: Vec<RenkDurağı>,
    },
    /// Radyal gradyan (`echarts.graphic.RadialGradient` karşılığı).
    /// Merkez `(x, y)` ve `yarıçap` birim sınır kutusundadır; daire/dilim
    /// ilkellerinde durak konumu iç→dış yarıçapa eşlenerek eşmerkezli
    /// halkalarla yaklaşıklanır.
    RadyalGradyan {
        x: f32,
        y: f32,
        yarıçap: f32,
        duraklar: Vec<RenkDurağı>,
    },
}

impl Dolgu {
    pub fn doğrusal(x: f32, y: f32, x2: f32, y2: f32, duraklar: Vec<RenkDurağı>) -> Self {
        Dolgu::DoğrusalGradyan {
            x,
            y,
            x2,
            y2,
            duraklar,
        }
    }

    /// zrender path iç-etiket karşılığı. Düz renklerde parlaklık hesabı
    /// yapılır; gradyan ve görüntü desenleri renk metni olmadığından
    /// `Path#getInsideTextFill` açık etikete, kontursuz olarak düşer.
    pub fn zrender_iç_etiket_stili(&self, koyu_kip: bool) -> (Renk, Option<Renk>) {
        match self {
            Dolgu::Düz(renk) => renk.zrender_iç_etiket_stili(koyu_kip),
            Dolgu::DoğrusalGradyan { .. } | Dolgu::RadyalGradyan { .. } | Dolgu::Desen(_) => {
                (Renk::onaltılık(0xcccccc), None)
            }
        }
    }

    pub fn radyal(x: f32, y: f32, yarıçap: f32, duraklar: Vec<RenkDurağı>) -> Self {
        Dolgu::RadyalGradyan {
            x,
            y,
            yarıçap,
            duraklar,
        }
    }

    /// Temsilî düz renk (gradyanlarda ilk durak) — gösterge imleri ve ipucu
    /// noktaları için kullanılır.
    pub fn temsilî(&self) -> Renk {
        match self {
            Dolgu::Düz(r) => *r,
            Dolgu::Desen(desen) => desen.temsilî(),
            Dolgu::DoğrusalGradyan { duraklar, .. } | Dolgu::RadyalGradyan { duraklar, .. } => {
                duraklar.first().map(|d| d.renk).unwrap_or(Renk::SİYAH)
            }
        }
    }

    fn durakları_soldur(duraklar: &[RenkDurağı], çarpan: f32) -> Vec<RenkDurağı> {
        duraklar
            .iter()
            .map(|d| RenkDurağı {
                konum: d.konum,
                renk: d.renk.opaklık(çarpan),
            })
            .collect()
    }

    pub fn opaklık(&self, çarpan: f32) -> Dolgu {
        match self {
            Dolgu::Düz(r) => Dolgu::Düz(r.opaklık(çarpan)),
            Dolgu::Desen(desen) => Dolgu::Desen(desen.clone().opaklık(desen.opaklık * çarpan)),
            Dolgu::DoğrusalGradyan {
                x,
                y,
                x2,
                y2,
                duraklar,
            } => Dolgu::DoğrusalGradyan {
                x: *x,
                y: *y,
                x2: *x2,
                y2: *y2,
                duraklar: Self::durakları_soldur(duraklar, çarpan),
            },
            Dolgu::RadyalGradyan {
                x,
                y,
                yarıçap,
                duraklar,
            } => Dolgu::RadyalGradyan {
                x: *x,
                y: *y,
                yarıçap: *yarıçap,
                duraklar: Self::durakları_soldur(duraklar, çarpan),
            },
        }
    }

    /// gpui [`Background`] tipine dönüştürür.
    ///
    /// gpui doğal olarak iki duraklı doğrusal gradyan destekler; daha çok
    /// duraklı gradyanlar ilk ve son durakla yaklaşıklanır (çok duraklı
    /// doğrusal gradyanların tam karşılığı, gpui yüzeyinde bantlama ile
    /// çizilir — bkz. `cizim::cizici`). Radyal gradyan burada orta renge
    /// düşer; daire/dilim ilkellerinde halkalarla yaklaşıklanır.
    #[cfg(feature = "gpui")]
    pub fn gpui_arkaplan(&self) -> Background {
        match self {
            Dolgu::Düz(r) => r.gpui_hsla().into(),
            // gpui'nin `Background` tipi görüntü shader'ı taşımıyor. Yol
            // yüzeyi deseni ayrıca boyar; yalnız doğrudan Background isteyen
            // çağrılar için temsilî renk kullanılır.
            Dolgu::Desen(desen) => desen.temsilî().gpui_hsla().into(),
            Dolgu::RadyalGradyan { duraklar, .. } => {
                let orta = duraklar
                    .get(duraklar.len() / 2)
                    .map(|d| d.renk)
                    .unwrap_or(Renk::SAYDAM);
                orta.gpui_hsla().into()
            }
            Dolgu::DoğrusalGradyan {
                x,
                y,
                x2,
                y2,
                duraklar,
            } => {
                let (Some(ilk), Some(son)) = (duraklar.first(), duraklar.last()) else {
                    return Renk::SAYDAM.gpui_hsla().into();
                };
                if duraklar.len() == 1 {
                    return ilk.renk.gpui_hsla().into();
                }
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
#[allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
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
        assert_eq!(
            Renk::çöz("rgba(128, 155, 72, 255)"),
            Some(Renk::kyma(128.0 / 255.0, 155.0 / 255.0, 72.0 / 255.0, 1.0))
        );
    }

    #[test]
    fn hsl_tonu_zrender_modify_hsl_sonucuyla_eslesir() {
        let rgb8 = |renk: Renk| {
            [
                (renk.kırmızı * 255.0).round() as u8,
                (renk.yeşil * 255.0).round() as u8,
                (renk.mavi * 255.0).round() as u8,
            ]
        };
        let taban = Renk::onaltılık(0x5a94df);

        assert_eq!(rgb8(taban.ton_ile(0.0)), [223, 90, 90]);
        assert_eq!(rgb8(taban.ton_ile(13.0)), [223, 119, 90]);
        assert_eq!(rgb8(taban.ton_ile(156.0)), [90, 223, 170]);
        assert_eq!(rgb8(taban.ton_ile(312.0)), [223, 90, 196]);
    }

    #[test]
    fn zrender_iç_etiket_parlaklığı_ve_konturu_path_dolgusunu_izler() {
        let mavi = Renk::onaltılık(0x5070dd);
        let yeşil = Renk::onaltılık(0xb6d634);
        let siyah = Renk::SİYAH;

        assert_eq!(
            mavi.zrender_iç_etiket_stili(false),
            (Renk::onaltılık(0xeeeeee), Some(mavi))
        );
        assert_eq!(
            yeşil.zrender_iç_etiket_stili(false),
            (Renk::onaltılık(0x333333), None)
        );
        assert_eq!(
            siyah.zrender_iç_etiket_stili(false),
            (Renk::onaltılık(0xcccccc), Some(siyah))
        );
        assert_eq!(
            yeşil.zrender_iç_etiket_stili(true),
            (Renk::onaltılık(0x333333), Some(yeşil))
        );

        let gradyan = Dolgu::doğrusal(
            0.0,
            0.0,
            1.0,
            0.0,
            vec![
                RenkDurağı::yeni(0.0, Renk::SİYAH),
                RenkDurağı::yeni(1.0, Renk::BEYAZ),
            ],
        );
        assert_eq!(
            gradyan.zrender_iç_etiket_stili(false),
            (Renk::onaltılık(0xcccccc), None)
        );
    }

    #[test]
    fn görüntü_deseni_rgba_boyutunu_doğrular_ve_önden_çarpar() {
        assert!(GörüntüDeseni::rgba(1, 1, vec![1, 2, 3], DesenTekrarı::Tekrar).is_none());
        let desen =
            GörüntüDeseni::rgba(1, 1, vec![200, 100, 50, 128], DesenTekrarı::Tekrar).unwrap();
        assert_eq!(&*desen.pikseller, &[100, 50, 25, 128]);
    }

    #[test]
    fn desen_opaklığı_birikimli_çarpılır() {
        let desen = GörüntüDeseni::rgba(1, 1, vec![255, 255, 255, 255], DesenTekrarı::Tekrar)
            .unwrap()
            .opaklık(0.8);
        let Dolgu::Desen(sönük) = Dolgu::Desen(desen).opaklık(0.5) else {
            panic!("desen dolgusu türünü korumalı");
        };
        assert!((sönük.opaklık - 0.4).abs() < 1e-6);
    }
}
