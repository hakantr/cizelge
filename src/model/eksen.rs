//! Eksen seçenekleri — ECharts'taki `xAxis` / `yAxis` tanımlarının karşılığı
//! (`echarts/src/coord/axisCommonTypes.ts` ve `axisDefault.ts`).

use crate::model::stil::{Biçimleyici, YazıStili, ÇizgiTürü};
use crate::renk::Renk;

/// Eksen türü (`axis.type`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EksenTürü {
    /// Sayısal değer ekseni (`'value'`).
    #[default]
    Değer,
    /// Kategori ekseni (`'category'`).
    Kategori,
    /// Zaman ekseni (`'time'`), değerler Unix milisaniyesi.
    Zaman,
    /// Logaritmik eksen (`'log'`).
    Log,
}

/// Değer/zaman eksenlerinde `boundaryGap` uçlarından biri. ECharts sayı
/// değerini doğrudan kapsam oranı, yüzde metnini de yüzde olarak yorumlar.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SayısalKenarBoşluğu {
    Oran(f64),
    Yüzde(f64),
}

impl SayısalKenarBoşluğu {
    pub fn çöz(self, açıklık: f64) -> f64 {
        match self {
            Self::Oran(oran) => açıklık.max(0.0) * oran.max(0.0),
            Self::Yüzde(yüzde) => açıklık.max(0.0) * (yüzde / 100.0).max(0.0),
        }
    }
}

impl From<f64> for SayısalKenarBoşluğu {
    fn from(değer: f64) -> Self {
        Self::Oran(değer)
    }
}

impl From<f32> for SayısalKenarBoşluğu {
    fn from(değer: f32) -> Self {
        Self::Oran(değer as f64)
    }
}

impl From<&str> for SayısalKenarBoşluğu {
    fn from(değer: &str) -> Self {
        değer
            .trim()
            .strip_suffix('%')
            .and_then(|yüzde| yüzde.parse().ok())
            .map(Self::Yüzde)
            .unwrap_or_else(|| Self::Oran(değer.parse().unwrap_or(0.0)))
    }
}

impl From<String> for SayısalKenarBoşluğu {
    fn from(değer: String) -> Self {
        Self::from(değer.as_str())
    }
}

/// Eksenin çizildiği kenar (`axis.position`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EksenKonumu {
    Alt,
    Üst,
    Sol,
    Sağ,
}

/// Eksen adının eksen boyunca yerleşimi (`nameLocation`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EksenAdKonumu {
    Başlangıç,
    Orta,
    #[default]
    Bitiş,
}

/// `axisLine.onZero`: sıfırda kesişme davranışı.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EksenSıfırKipi {
    /// ECharts 6.1 öntanımlısı (`'auto'`).
    #[default]
    Otomatik,
    Açık,
    Kapalı,
}

/// Eksen çizgisi (`axisLine`).
#[derive(Clone, PartialEq, Debug)]
pub struct EksenÇizgisi {
    pub göster: Option<bool>,
    pub sıfır: EksenSıfırKipi,
    /// Kesişilecek dik eksen sırası (`axisLine.onZeroAxisIndex`).
    pub sıfır_eksen_sırası: Option<usize>,
    pub renk: Option<Renk>,
    pub kalınlık: f32,
}

impl Default for EksenÇizgisi {
    fn default() -> Self {
        EksenÇizgisi {
            göster: None,
            sıfır: EksenSıfırKipi::Otomatik,
            sıfır_eksen_sırası: None,
            renk: None,
            kalınlık: 1.0,
        }
    }
}

impl EksenÇizgisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = Some(göster);
        self
    }

    pub fn sıfır(mut self, kip: EksenSıfırKipi) -> Self {
        self.sıfır = kip;
        self
    }

    pub fn sıfır_eksen_sırası(mut self, sıra: usize) -> Self {
        self.sıfır_eksen_sırası = Some(sıra);
        self
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn kalınlık(mut self, kalınlık: f32) -> Self {
        self.kalınlık = kalınlık.max(0.0);
        self
    }
}

/// Eksen çentiği (`axisTick`).
#[derive(Clone, PartialEq, Debug)]
pub struct EksenÇentiği {
    pub göster: Option<bool>,
    pub uzunluk: f32,
    /// Kategori eksenlerinde çentiği etiketle hizalar
    /// (`axisTick.alignWithLabel`).
    pub etiketle_hizala: bool,
}

impl Default for EksenÇentiği {
    fn default() -> Self {
        EksenÇentiği {
            göster: None,
            uzunluk: 5.0,
            etiketle_hizala: false,
        }
    }
}

/// Eksen etiketi (`axisLabel`).
#[derive(Clone, PartialEq, Debug)]
pub struct EksenEtiketi {
    pub göster: bool,
    pub yazı: YazıStili,
    pub biçimleyici: Option<Biçimleyici>,
    /// Etiket ile eksen arasındaki boşluk (`axisLabel.margin`).
    pub boşluk: f32,
    /// Derece cinsinden dönüş (`axisLabel.rotate`); pozitif değer ECharts
    /// Canvas koordinatında saat yönünün tersine döner.
    pub döndürme: f32,
    /// Açık kategori aralığı (`axisLabel.interval`): `0` bütün etiketler,
    /// `1` birer atlayarak. `None`, ECharts'ın otomatik hesabıdır.
    pub aralık: Option<usize>,
}

impl Default for EksenEtiketi {
    fn default() -> Self {
        EksenEtiketi {
            göster: true,
            yazı: YazıStili::default(),
            biçimleyici: None,
            boşluk: 8.0,
            döndürme: 0.0,
            aralık: None,
        }
    }
}

impl EksenEtiketi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn biçimleyici(mut self, biçimleyici: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(biçimleyici.into());
        self
    }

    pub fn boşluk(mut self, boşluk: f32) -> Self {
        self.boşluk = boşluk.max(0.0);
        self
    }

    pub fn döndür(mut self, derece: f32) -> Self {
        self.döndürme = if derece.is_finite() { derece } else { 0.0 };
        self
    }

    pub fn aralık(mut self, atlanan: usize) -> Self {
        self.aralık = Some(atlanan);
        self
    }

    pub fn otomatik_aralık(mut self) -> Self {
        self.aralık = None;
        self
    }
}

/// Ara (minör) çentikler (`minorTick`).
#[derive(Clone, PartialEq, Debug)]
pub struct AraÇentik {
    pub göster: bool,
    /// Ana çentik aralığının kaça bölüneceği (`splitNumber`, öntanımlı 5).
    pub bölme_sayısı: usize,
    /// Çentik uzunluğu (`length`, öntanımlı 3).
    pub uzunluk: f32,
}

impl Default for AraÇentik {
    fn default() -> Self {
        AraÇentik {
            göster: false,
            bölme_sayısı: 5,
            uzunluk: 3.0,
        }
    }
}

/// Bölme alanı (`splitArea`): ana çentikler arasında dönüşümlü renkli
/// bantlar.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct BölmeAlanı {
    pub göster: bool,
    /// Dönüşümlü bant renkleri (`areaStyle.color`); boş bırakılırsa çizim
    /// anında etkin temadan çözülür (koyu tema sonradan seçilse de doğru).
    pub renkler: Vec<Renk>,
}

/// Bölme çizgileri (`splitLine`).
#[derive(Clone, PartialEq, Debug)]
pub struct BölmeÇizgisi {
    /// `None` ise tür öntanımlısı geçerlidir: değer/log/zaman eksenlerinde
    /// açık, kategori ekseninde kapalı.
    pub göster: Option<bool>,
    pub renk: Option<Renk>,
    pub tür: ÇizgiTürü,
}

impl Default for BölmeÇizgisi {
    fn default() -> Self {
        BölmeÇizgisi {
            göster: None,
            renk: None,
            tür: ÇizgiTürü::Düz,
        }
    }
}

/// Eksen seçenekleri (`xAxis` / `yAxis`).
#[derive(Clone, PartialEq, Debug)]
pub struct Eksen {
    pub tür: EksenTürü,
    pub ad: Option<String>,
    pub ad_konumu: EksenAdKonumu,
    /// Eksen çizgisi ile eksen adı arasındaki boşluk (`nameGap`).
    pub ad_boşluğu: f32,
    /// Kategori ekseni verisi.
    pub veri: Vec<String>,
    /// Kategori ekseninde uçlarda yarım bant boşluğu bırakılsın mı
    /// (`boundaryGap`)? Kategoride öntanımlı `true`.
    pub kenar_boşluğu: Option<bool>,
    /// Değer/zaman ekseni `boundaryGap: [alt, üst]` uçları.
    pub sayısal_kenar_boşluğu: Option<[SayısalKenarBoşluğu; 2]>,
    pub en_az: Option<f64>,
    pub en_çok: Option<f64>,
    /// `false` ise kapsam sıfırı içerecek şekilde genişletilir; ECharts'taki
    /// `scale` seçeneğinin tersidir (`scale: true` ⇔ `sıfırı_içer: false`).
    pub sıfırı_içer: bool,
    /// `splitNumber`, öntanımlı 5.
    pub bölme_sayısı: usize,
    /// Çentik hizalama (`alignTicks`): aynı ızgaradaki ilk değer ekseninin
    /// bölme sayısına uyar; bölme çizgileri üst üste düşer (yalnız değer
    /// eksenlerinde anlamlıdır).
    pub çentik_hizala: bool,
    pub en_küçük_adım: Option<f64>,
    pub en_büyük_adım: Option<f64>,
    /// Log ekseni tabanı (`logBase`), öntanımlı 10.
    pub log_tabanı: f64,
    pub ters: bool,
    pub konum: Option<EksenKonumu>,
    /// Ekseni seçilen kenardan dışarı taşır (`offset`).
    pub kaydırma: f32,
    /// Bağlı olduğu ızgaranın `ızgaralar` listesindeki sırası
    /// (`gridIndex`).
    pub ızgara_sırası: usize,
    pub çizgi: EksenÇizgisi,
    pub çentik: EksenÇentiği,
    /// Ara (minör) çentikler (`minorTick`); yalnız değer/log eksenlerinde.
    pub ara_çentik: AraÇentik,
    pub etiket: EksenEtiketi,
    pub bölme_çizgisi: BölmeÇizgisi,
    /// Ara bölme çizgileri (`minorSplitLine`); ara çentik konumlarında.
    pub ara_bölme_çizgisi: BölmeÇizgisi,
    /// Bölme alanı (`splitArea`).
    pub bölme_alanı: BölmeAlanı,
}

impl Default for Eksen {
    fn default() -> Self {
        Eksen {
            tür: EksenTürü::Değer,
            ad: None,
            ad_konumu: EksenAdKonumu::Bitiş,
            ad_boşluğu: 15.0,
            veri: Vec::new(),
            kenar_boşluğu: None,
            sayısal_kenar_boşluğu: None,
            en_az: None,
            en_çok: None,
            sıfırı_içer: true,
            bölme_sayısı: 5,
            çentik_hizala: false,
            en_küçük_adım: None,
            en_büyük_adım: None,
            log_tabanı: 10.0,
            ters: false,
            konum: None,
            kaydırma: 0.0,
            ızgara_sırası: 0,
            çizgi: EksenÇizgisi::default(),
            çentik: EksenÇentiği::default(),
            ara_çentik: AraÇentik::default(),
            etiket: EksenEtiketi::default(),
            bölme_çizgisi: BölmeÇizgisi::default(),
            ara_bölme_çizgisi: BölmeÇizgisi {
                göster: Some(false),
                ..Default::default()
            },
            bölme_alanı: BölmeAlanı::default(),
        }
    }
}

impl Eksen {
    /// Sayısal değer ekseni.
    pub fn değer() -> Self {
        Eksen {
            tür: EksenTürü::Değer,
            ..Default::default()
        }
    }

    /// Kategori ekseni.
    pub fn kategori() -> Self {
        Eksen {
            tür: EksenTürü::Kategori,
            ..Default::default()
        }
    }

    /// Zaman ekseni.
    pub fn zaman() -> Self {
        Eksen {
            tür: EksenTürü::Zaman,
            // `timeAxis` öntanımlısı değer ekseninden ayrılır: altı bölme
            // ister ve ana splitLine'ı kapatır.
            bölme_sayısı: 6,
            bölme_çizgisi: BölmeÇizgisi {
                göster: Some(false),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Logaritmik eksen.
    pub fn log() -> Self {
        Eksen {
            tür: EksenTürü::Log,
            sıfırı_içer: false,
            ..Default::default()
        }
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn ad_konumu(mut self, konum: EksenAdKonumu) -> Self {
        self.ad_konumu = konum;
        self
    }

    pub fn ad_boşluğu(mut self, boşluk: f32) -> Self {
        self.ad_boşluğu = if boşluk.is_finite() { boşluk } else { 15.0 };
        self
    }

    pub fn veri<S: Into<String>>(mut self, veri: impl IntoIterator<Item = S>) -> Self {
        self.veri = veri.into_iter().map(Into::into).collect();
        self
    }

    pub fn kenar_boşluğu(mut self, açık: bool) -> Self {
        self.kenar_boşluğu = Some(açık);
        self
    }

    pub fn sayısal_kenar_boşluğu(
        mut self,
        alt: impl Into<SayısalKenarBoşluğu>,
        üst: impl Into<SayısalKenarBoşluğu>,
    ) -> Self {
        self.sayısal_kenar_boşluğu = Some([alt.into(), üst.into()]);
        self
    }

    pub fn en_az(mut self, değer: f64) -> Self {
        self.en_az = Some(değer);
        self
    }

    pub fn en_çok(mut self, değer: f64) -> Self {
        self.en_çok = Some(değer);
        self
    }

    /// ECharts `scale: true` karşılığı: kapsam sıfıra zorlanmaz.
    pub fn ölçekli(mut self, ölçekli: bool) -> Self {
        self.sıfırı_içer = !ölçekli;
        self
    }

    pub fn bölme_sayısı(mut self, sayı: usize) -> Self {
        self.bölme_sayısı = sayı.max(1);
        self
    }

    /// Çentik hizalamayı açar (`alignTicks`).
    pub fn çentik_hizala(mut self, açık: bool) -> Self {
        self.çentik_hizala = açık;
        self
    }

    pub fn en_küçük_adım(mut self, adım: f64) -> Self {
        self.en_küçük_adım = Some(adım);
        self
    }

    pub fn en_büyük_adım(mut self, adım: f64) -> Self {
        self.en_büyük_adım = Some(adım);
        self
    }

    pub fn log_tabanı(mut self, taban: f64) -> Self {
        self.log_tabanı = taban;
        self
    }

    pub fn ters(mut self, ters: bool) -> Self {
        self.ters = ters;
        self
    }

    pub fn konum(mut self, konum: EksenKonumu) -> Self {
        self.konum = Some(konum);
        self
    }

    pub fn kaydırma(mut self, piksel: f32) -> Self {
        self.kaydırma = if piksel.is_finite() {
            piksel.max(0.0)
        } else {
            0.0
        };
        self
    }

    /// Ekseni `ızgaralar` listesindeki bir ızgaraya bağlar (`gridIndex`).
    pub fn ızgara_sırası(mut self, sıra: usize) -> Self {
        self.ızgara_sırası = sıra;
        self
    }

    pub fn çizgi(mut self, çizgi: EksenÇizgisi) -> Self {
        self.çizgi = çizgi;
        self
    }

    pub fn çentik(mut self, çentik: EksenÇentiği) -> Self {
        self.çentik = çentik;
        self
    }

    pub fn etiket(mut self, etiket: EksenEtiketi) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn etiket_biçimleyici(mut self, b: impl Into<Biçimleyici>) -> Self {
        self.etiket.biçimleyici = Some(b.into());
        self
    }

    pub fn bölme_çizgisi(mut self, bölme: BölmeÇizgisi) -> Self {
        self.bölme_çizgisi = bölme;
        self
    }

    pub fn bölme_çizgisi_göster(mut self, göster: bool) -> Self {
        self.bölme_çizgisi.göster = Some(göster);
        self
    }

    pub fn ara_çentik(mut self, ara: AraÇentik) -> Self {
        self.ara_çentik = ara;
        self
    }

    pub fn ara_çentik_göster(mut self, göster: bool) -> Self {
        self.ara_çentik.göster = göster;
        self
    }

    pub fn ara_bölme_çizgisi_göster(mut self, göster: bool) -> Self {
        self.ara_bölme_çizgisi.göster = Some(göster);
        self
    }

    pub fn bölme_alanı(mut self, alan: BölmeAlanı) -> Self {
        self.bölme_alanı = alan;
        self
    }

    pub fn bölme_alanı_göster(mut self, göster: bool) -> Self {
        self.bölme_alanı.göster = göster;
        self
    }

    /// Kategori ekseninde bant yerleşimi geçerli mi?
    pub fn bantlı_mı(&self) -> bool {
        self.tür == EksenTürü::Kategori && self.kenar_boşluğu.unwrap_or(true)
    }

    /// Eksen çizgisi öntanımlı görünürlüğü: kategori ve zaman eksenlerinde
    /// açık, değer eksenlerinde de açıktır (ECharts v5+ value ekseninde
    /// kapalıdır ama kartezyen ızgarada alt eksen çizgisi beklenir; burada
    /// kategori/zamanda açık, değer/logda kapalı bırakılır).
    pub fn çizgi_görünür_mü(&self) -> bool {
        self.çizgi
            .göster
            .unwrap_or(matches!(self.tür, EksenTürü::Kategori | EksenTürü::Zaman))
    }

    /// Çentik öntanımlı görünürlüğü: yalnızca kategori/zaman eksenlerinde.
    pub fn çentik_görünür_mü(&self) -> bool {
        self.çentik
            .göster
            .unwrap_or(matches!(self.tür, EksenTürü::Kategori | EksenTürü::Zaman))
    }

    /// Bölme çizgisi öntanımlı görünürlüğü: değer/log eksenlerinde açık,
    /// kategori/zaman eksenlerinde kapalı.
    pub fn bölme_görünür_mü(&self) -> bool {
        self.bölme_çizgisi
            .göster
            .unwrap_or(!matches!(self.tür, EksenTürü::Kategori | EksenTürü::Zaman))
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn zaman_ekseni_resmi_bolme_varsayilanlarini_kullanir() {
        let eksen = Eksen::zaman();
        assert_eq!(eksen.bölme_sayısı, 6);
        assert!(!eksen.bölme_görünür_mü());
    }
}
