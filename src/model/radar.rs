//! Radar koordinat seçenekleri — ECharts `radar` bileşeninin karşılığı.

use crate::model::Uzunluk;
use crate::model::stil::{Biçimleyici, YazıStili, ÇizgiStili};
use crate::renk::{Dolgu, Renk};

/// Radar ağının biçimi (`radar.shape`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RadarŞekli {
    #[default]
    Çokgen,
    Daire,
}

/// Radar göstergesi (`radar.indicator` öğesi): bir kolun adı ve aralığı.
///
/// ECharts, açık `min`/`max` olmayan göstergelerin kapsamını o radara bağlı
/// bütün serilerin verisinden çıkarır. Değer alanları kaynak uyumluluğu için
/// sayısal tutulur; `*_belirtildi` bayrakları otomatik kapsamı ayırır.
#[derive(Clone, PartialEq, Debug)]
pub struct RadarGöstergesi {
    pub ad: String,
    pub en_az: f64,
    pub en_çok: f64,
    pub en_az_belirtildi: bool,
    pub en_çok_belirtildi: bool,
    /// `indicator[i].color`, ortak `axisName.color` seçeneğini geçersiz kılar.
    pub renk: Option<Renk>,
}

impl RadarGöstergesi {
    /// Adı ve açık üst sınırı olan gösterge.
    pub fn yeni(ad: impl Into<String>, en_çok: f64) -> Self {
        Self {
            ad: ad.into(),
            en_az: 0.0,
            en_çok,
            en_az_belirtildi: false,
            en_çok_belirtildi: true,
            renk: None,
        }
    }

    /// Yalnız adı verilen, kapsamı bağlı serilerden türetilecek gösterge.
    pub fn otomatik(ad: impl Into<String>) -> Self {
        Self {
            ad: ad.into(),
            en_az: 0.0,
            en_çok: 0.0,
            en_az_belirtildi: false,
            en_çok_belirtildi: false,
            renk: None,
        }
    }

    pub fn en_az(mut self, en_az: f64) -> Self {
        self.en_az = en_az;
        self.en_az_belirtildi = true;
        self
    }

    pub fn en_çok(mut self, en_çok: f64) -> Self {
        self.en_çok = en_çok;
        self.en_çok_belirtildi = true;
        self
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
        self
    }
}

/// Gösterge kollarının uç adları (`radar.axisName`).
#[derive(Clone, PartialEq, Debug)]
pub struct RadarEksenAdı {
    pub göster: bool,
    pub biçimleyici: Option<Biçimleyici>,
    pub yazı: YazıStili,
    /// `axisNameGap`, ECharts 6.1 öntanımlısı 15 pikseldir.
    pub boşluk: f32,
}

impl Default for RadarEksenAdı {
    fn default() -> Self {
        Self {
            göster: true,
            biçimleyici: None,
            yazı: YazıStili::default(),
            boşluk: 15.0,
        }
    }
}

impl RadarEksenAdı {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn biçimleyici(mut self, biçimleyici: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(biçimleyici.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn boşluk(mut self, boşluk: f32) -> Self {
        self.boşluk = boşluk.max(0.0);
        self
    }
}

/// Radar `axisLine` ve `splitLine` seçeneklerinin ortak modeli.
#[derive(Clone, PartialEq, Debug)]
pub struct RadarÇizgileri {
    pub göster: bool,
    /// Tek renk veya halka sırasıyla dönen renk listesi.
    pub renkler: Vec<Renk>,
    pub stil: ÇizgiStili,
}

impl Default for RadarÇizgileri {
    fn default() -> Self {
        Self {
            göster: true,
            renkler: Vec::new(),
            stil: ÇizgiStili::yeni().kalınlık(1.0),
        }
    }
}

impl RadarÇizgileri {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renkler = vec![renk.into()];
        self
    }

    pub fn renkler<R: Into<Renk>>(mut self, renkler: impl IntoIterator<Item = R>) -> Self {
        self.renkler = renkler.into_iter().map(Into::into).collect();
        self
    }

    pub fn stil(mut self, stil: ÇizgiStili) -> Self {
        self.stil = stil;
        self
    }
}

/// Radar bölme alanları (`radar.splitArea`).
#[derive(Clone, PartialEq, Debug)]
pub struct RadarBölmeAlanı {
    pub göster: bool,
    /// İç halkadan dış halkaya doğru dönen dolgu listesi.
    pub renkler: Vec<Dolgu>,
    pub opaklık: f32,
    pub gölge_bulanıklığı: f32,
    pub gölge_rengi: Option<Renk>,
    pub gölge_kayması: (f32, f32),
}

impl Default for RadarBölmeAlanı {
    fn default() -> Self {
        Self {
            göster: true,
            renkler: Vec::new(),
            opaklık: 1.0,
            gölge_bulanıklığı: 0.0,
            gölge_rengi: None,
            gölge_kayması: (0.0, 0.0),
        }
    }
}

impl RadarBölmeAlanı {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn renkler<D: Into<Dolgu>>(mut self, renkler: impl IntoIterator<Item = D>) -> Self {
        self.renkler = renkler.into_iter().map(Into::into).collect();
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = opaklık.clamp(0.0, 1.0);
        self
    }

    pub fn gölge(mut self, renk: impl Into<Renk>, bulanıklık: f32, x: f32, y: f32) -> Self {
        self.gölge_rengi = Some(renk.into());
        self.gölge_bulanıklığı = bulanıklık.max(0.0);
        self.gölge_kayması = (x, y);
        self
    }
}

/// Radar koordinat sistemi (`radar` seçeneği).
#[derive(Clone, PartialEq, Debug)]
pub struct RadarKoordinatı {
    pub göstergeler: Vec<RadarGöstergesi>,
    pub merkez: (Uzunluk, Uzunluk),
    pub iç_yarıçap: Uzunluk,
    pub yarıçap: Uzunluk,
    /// Derece cinsinden `startAngle`; ECharts öntanımlısı 90.
    pub başlangıç_açısı: f32,
    /// `clockwise`; ECharts radar öntanımlısı `false`.
    pub saat_yönü: bool,
    /// Halkalar arası bölme sayısı (`splitNumber`, öntanımlı 5).
    pub bölme_sayısı: usize,
    pub şekil: RadarŞekli,
    /// `scale: false`, otomatik kapsamlarda sıfırı içerir.
    pub sıfırı_içer: bool,
    pub eksen_adı: RadarEksenAdı,
    pub eksen_çizgisi: RadarÇizgileri,
    pub bölme_çizgisi: RadarÇizgileri,
    pub bölme_alanı: RadarBölmeAlanı,
    /// Eski kaynak API'si; etkin değer `bölme_alanı.göster` ile birlikte
    /// değerlendirilir.
    pub bölme_alanı_göster: bool,
    pub sessiz: bool,
    pub z: i32,
}

impl Default for RadarKoordinatı {
    fn default() -> Self {
        Self {
            göstergeler: Vec::new(),
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(50.0)),
            iç_yarıçap: Uzunluk::Piksel(0.0),
            yarıçap: Uzunluk::Yüzde(50.0),
            başlangıç_açısı: 90.0,
            saat_yönü: false,
            bölme_sayısı: 5,
            şekil: RadarŞekli::Çokgen,
            sıfırı_içer: true,
            eksen_adı: RadarEksenAdı::default(),
            eksen_çizgisi: RadarÇizgileri::default(),
            bölme_çizgisi: RadarÇizgileri::default(),
            bölme_alanı: RadarBölmeAlanı::default(),
            bölme_alanı_göster: true,
            sessiz: false,
            z: 0,
        }
    }
}

impl RadarKoordinatı {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Göstergeler: `(ad, en_çok)` çiftleri.
    pub fn göstergeler<S: Into<String>>(
        mut self,
        göstergeler: impl IntoIterator<Item = (S, f64)>,
    ) -> Self {
        self.göstergeler = göstergeler
            .into_iter()
            .map(|(ad, en_çok)| RadarGöstergesi::yeni(ad, en_çok))
            .collect();
        self
    }

    pub fn gösterge_listesi(
        mut self,
        göstergeler: impl IntoIterator<Item = RadarGöstergesi>,
    ) -> Self {
        self.göstergeler = göstergeler.into_iter().collect();
        self
    }

    pub fn merkez(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.merkez = (x.into(), y.into());
        self
    }

    pub fn yarıçap(mut self, yarıçap: impl Into<Uzunluk>) -> Self {
        self.iç_yarıçap = Uzunluk::Piksel(0.0);
        self.yarıçap = yarıçap.into();
        self
    }

    pub fn yarıçap_aralığı(
        mut self, iç: impl Into<Uzunluk>, dış: impl Into<Uzunluk>
    ) -> Self {
        self.iç_yarıçap = iç.into();
        self.yarıçap = dış.into();
        self
    }

    pub fn başlangıç_açısı(mut self, derece: f32) -> Self {
        self.başlangıç_açısı = derece;
        self
    }

    pub fn saat_yönü(mut self, saat_yönü: bool) -> Self {
        self.saat_yönü = saat_yönü;
        self
    }

    pub fn şekil(mut self, şekil: RadarŞekli) -> Self {
        self.şekil = şekil;
        self
    }

    pub fn bölme_sayısı(mut self, sayı: usize) -> Self {
        self.bölme_sayısı = sayı.max(1);
        self
    }

    pub fn sıfırı_içer(mut self, içer: bool) -> Self {
        self.sıfırı_içer = içer;
        self
    }

    pub fn eksen_adı(mut self, eksen_adı: RadarEksenAdı) -> Self {
        self.eksen_adı = eksen_adı;
        self
    }

    pub fn eksen_çizgisi(mut self, çizgiler: RadarÇizgileri) -> Self {
        self.eksen_çizgisi = çizgiler;
        self
    }

    pub fn bölme_çizgisi(mut self, çizgiler: RadarÇizgileri) -> Self {
        self.bölme_çizgisi = çizgiler;
        self
    }

    pub fn bölme_alanı(mut self, alan: RadarBölmeAlanı) -> Self {
        self.bölme_alanı_göster = alan.göster;
        self.bölme_alanı = alan;
        self
    }

    pub fn bölme_alanı_göster(mut self, göster: bool) -> Self {
        self.bölme_alanı_göster = göster;
        self.bölme_alanı.göster = göster;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }

    pub fn z(mut self, z: i32) -> Self {
        self.z = z;
        self
    }
}
