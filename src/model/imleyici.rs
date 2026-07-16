//! İmleyici seçenekleri — ECharts'taki `markLine`, `markPoint`, `markArea`
//! bileşenlerinin karşılığı (`echarts/src/component/marker`).

use crate::model::stil::{Etiket, YazıStili, ÇizgiStili, ÇizgiTürü, ÖğeStili};

/// İm değeri: sabit sayı ya da seri verisinden türetilen istatistik
/// (`markLine.data[i].type: 'average' | 'min' | 'max'`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum İmDeğeri {
    Değer(f64),
    Ortalama,
    EnKüçük,
    EnBüyük,
}

/// İm çizgisinin yönü.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum İmYönü {
    /// Değer ekseni üzerinde sabit — yatay çizgi (`yAxis: …`).
    Yatay,
    /// Kategori/x ekseni üzerinde sabit — dikey çizgi (`xAxis: …`).
    Dikey,
}

/// Tek bir im çizgisi tanımı (`markLine.data` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct İmÇizgisiTanımı {
    pub ad: Option<String>,
    pub yön: İmYönü,
    pub değer: İmDeğeri,
}

/// İm çizgisi (`markLine`): seriye bağlı yatay/dikey başvuru çizgileri.
#[derive(Clone, PartialEq, Debug)]
pub struct İmÇizgisi {
    pub veri: Vec<İmÇizgisiTanımı>,
    /// Öntanımlı: seri renginde kesikli.
    pub stil: ÇizgiStili,
    pub etiket: Etiket,
}

impl Default for İmÇizgisi {
    fn default() -> Self {
        İmÇizgisi {
            veri: Vec::new(),
            stil: ÇizgiStili { kalınlık: 1.0, tür: ÇizgiTürü::Kesikli, ..Default::default() },
            etiket: Etiket { göster: true, ..Default::default() },
        }
    }
}

impl İmÇizgisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Yatay çizgi ekler (`{ yAxis: değer }` / `{ type: 'average' }`).
    pub fn yatay(mut self, değer: İmDeğeri) -> Self {
        self.veri.push(İmÇizgisiTanımı { ad: None, yön: İmYönü::Yatay, değer });
        self
    }

    /// Dikey çizgi ekler (`{ xAxis: değer }`).
    pub fn dikey(mut self, değer: İmDeğeri) -> Self {
        self.veri.push(İmÇizgisiTanımı { ad: None, yön: İmYönü::Dikey, değer });
        self
    }

    /// Adlandırılmış tanım ekler.
    pub fn tanım(mut self, tanım: İmÇizgisiTanımı) -> Self {
        self.veri.push(tanım);
        self
    }

    pub fn stil(mut self, stil: ÇizgiStili) -> Self {
        self.stil = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }
}

/// Tek bir im noktası tanımı (`markPoint.data` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct İmNoktasıTanımı {
    pub ad: Option<String>,
    /// İstatistik konumu (en büyük/en küçük değerli nokta) ya da
    /// `koordinat` ile doğrudan `(x, y)`.
    pub değer: Option<İmDeğeri>,
    pub koordinat: Option<(f64, f64)>,
}

/// İm noktası (`markPoint`): raptiye biçimli değer vurguları.
#[derive(Clone, PartialEq, Debug)]
pub struct İmNoktası {
    pub veri: Vec<İmNoktasıTanımı>,
    /// Raptiye çapı (`symbolSize`, ECharts öntanımlısı 50).
    pub boyut: f32,
    pub etiket: Etiket,
}

impl Default for İmNoktası {
    fn default() -> Self {
        İmNoktası {
            veri: Vec::new(),
            boyut: 42.0,
            etiket: Etiket {
                göster: true,
                yazı: YazıStili { kalın: true, ..Default::default() },
                ..Default::default()
            },
        }
    }
}

impl İmNoktası {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// En büyük değerli noktayı imler (`{ type: 'max' }`).
    pub fn en_büyük(mut self) -> Self {
        self.veri.push(İmNoktasıTanımı {
            ad: Some("En Büyük".to_string()),
            değer: Some(İmDeğeri::EnBüyük),
            koordinat: None,
        });
        self
    }

    /// En küçük değerli noktayı imler (`{ type: 'min' }`).
    pub fn en_küçük(mut self) -> Self {
        self.veri.push(İmNoktasıTanımı {
            ad: Some("En Küçük".to_string()),
            değer: Some(İmDeğeri::EnKüçük),
            koordinat: None,
        });
        self
    }

    /// Doğrudan `(x, y)` koordinatına im koyar (`{ coord: [x, y] }`).
    pub fn koordinat(mut self, x: f64, y: f64) -> Self {
        self.veri.push(İmNoktasıTanımı { ad: None, değer: None, koordinat: Some((x, y)) });
        self
    }

    pub fn boyut(mut self, boyut: f32) -> Self {
        self.boyut = boyut;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }
}

/// Tek bir im alanı tanımı (`markArea.data` öğesi): eksen değerleriyle
/// sınırlanan dikdörtgen. `None` uç, ızgara kenarına uzanır.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct İmAlanıTanımı {
    pub x0: Option<f64>,
    pub x1: Option<f64>,
    pub y0: Option<f64>,
    pub y1: Option<f64>,
}

/// İm alanı (`markArea`): vurgulanan bölgeler.
#[derive(Clone, PartialEq, Debug)]
pub struct İmAlanı {
    pub veri: Vec<(Option<String>, İmAlanıTanımı)>,
    /// Öntanımlı: seri renginin %15 opaklısı.
    pub stil: ÖğeStili,
    pub etiket: Etiket,
}

impl Default for İmAlanı {
    fn default() -> Self {
        İmAlanı {
            veri: Vec::new(),
            stil: ÖğeStili::default(),
            etiket: Etiket::default(),
        }
    }
}

impl İmAlanı {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// X aralığını vurgular (tüm yükseklik boyunca).
    pub fn x_aralığı(mut self, ad: impl Into<String>, x0: f64, x1: f64) -> Self {
        self.veri.push((
            Some(ad.into()),
            İmAlanıTanımı { x0: Some(x0), x1: Some(x1), ..Default::default() },
        ));
        self
    }

    /// Y aralığını vurgular (tüm genişlik boyunca).
    pub fn y_aralığı(mut self, ad: impl Into<String>, y0: f64, y1: f64) -> Self {
        self.veri.push((
            Some(ad.into()),
            İmAlanıTanımı { y0: Some(y0), y1: Some(y1), ..Default::default() },
        ));
        self
    }

    pub fn tanım(mut self, ad: Option<String>, tanım: İmAlanıTanımı) -> Self {
        self.veri.push((ad, tanım));
        self
    }

    pub fn stil(mut self, stil: ÖğeStili) -> Self {
        self.stil = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }
}

/// Bir serinin imleyici üçlüsü.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct İmleyiciler {
    pub çizgi: Option<İmÇizgisi>,
    pub nokta: Option<İmNoktası>,
    pub alan: Option<İmAlanı>,
}

impl İmleyiciler {
    pub fn boş_mu(&self) -> bool {
        self.çizgi.is_none() && self.nokta.is_none() && self.alan.is_none()
    }
}
