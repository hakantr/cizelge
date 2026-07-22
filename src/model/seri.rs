//! Seri seçenekleri — ECharts'taki `series` tanımlarının karşılığı
//! (`echarts/src/chart/*/…SeriesModel`).

use std::fmt;
use std::sync::Arc;

use crate::cizim::{SvgYolHatası, Yol};
use crate::koordinat::Dikdörtgen;
use crate::model::Uzunluk;
use crate::model::agac::{
    AğaçGezinmesi, AğaçKenarBiçimi, AğaçVurguOdağı, AğaçYerleşimi, AğaçYönü
};
use crate::model::bilesen::İpucu;
use crate::model::deger::{VeriDeğeri, VeriÖğesi, veri_listesi};
use crate::model::eksen::Eksen;
pub use crate::model::hatlar::{
    HatEfekti, HatKoordinatSistemi, HatKoordinatı, HatNoktası, HatVerisi, HatlarSerisi,
};
use crate::model::imleyici::{İmAlanı, İmNoktası, İmleyiciler, İmÇizgisi};
use crate::model::matris::MatrisAralığı;
use crate::model::stil::{
    AlanStili, Biçimleyici, Etiket, EtiketDöndürme, EtiketKonumu, EtiketYaması, YazıStili,
    ÇizgiStili, ÖğeStili,
};
use crate::model::veri_kumesi::SeriYerleşimi;
use crate::renk::{Dolgu, Renk};

/// Sembol biçimi (`symbol`).
#[derive(Clone, PartialEq, Debug, Default)]
pub enum Sembol {
    /// ECharts çizgi serisinin öntanımlısı (`'emptyCircle'`).
    #[default]
    İçiBoşDaire,
    Daire,
    Kare,
    /// ECharts `roundRect`; köşe yarıçapı kısa kenarın dörtte biridir.
    YuvarlakDikdörtgen,
    Üçgen,
    Elmas,
    /// ECharts `path://...` biçimindeki özel SVG sembolü.
    SvgYolu(Arc<Yol>),
    Yok,
}

impl Sembol {
    /// ECharts `path://...` sembolünü ortak çizim yoluna çözer.
    ///
    /// `path://` öneki isteğe bağlıdır; böylece hem doğrudan ECharts option
    /// değerleri hem de yalnız SVG `d` içeriği aynı API'den geçirilebilir.
    pub fn svg_yolu(veri: impl AsRef<str>) -> Result<Self, SvgYolHatası> {
        let veri = veri
            .as_ref()
            .strip_prefix("path://")
            .unwrap_or(veri.as_ref());
        Yol::svg_path_data(veri).map(|yol| Self::SvgYolu(Arc::new(yol)))
    }

    /// Önceden çözülmüş bir yolu özel sembol olarak kullanır.
    pub fn yoldan(yol: Yol) -> Self {
        Self::SvgYolu(Arc::new(yol))
    }
}

/// Serinin bağlı olduğu eksenler (`xAxisIndex` / `yAxisIndex`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct EksenBağı {
    pub x: usize,
    pub y: usize,
}

/// Çizgi örnekleme kipi (`sampling`): büyük veri setlerinde çizimden önce
/// nokta sayısını görünür piksel sayısına indirger.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Örnekleme {
    /// En Büyük Üçgen Üç Kova (LTTB): görsel biçimi en iyi koruyan seçim.
    Lttb,
    /// Kova ortalaması.
    Ortalama,
}

/// Basamaklı çizgi kipi (`step`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Basamak {
    Baş,
    Orta,
    Son,
}

/// Resimli sütun ayarları (`pictorialBar` karşılığı): sütun, tekrarlanan
/// sembollerle çizilir.
#[derive(Clone, PartialEq, Debug)]
pub struct Piktogram {
    pub sembol: Sembol,
    /// Sembol çapı.
    pub boyut: f32,
    /// Semboller arası boşluk.
    pub aralık: f32,
}

impl Default for Piktogram {
    fn default() -> Self {
        Piktogram {
            sembol: Sembol::Daire,
            boyut: 14.0,
            aralık: 4.0,
        }
    }
}

/// Pasta grafiklerinde gül (Nightingale) kipi (`roseType`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GülTürü {
    /// Yarıçap değerle orantılı (`'radius'`).
    Yarıçap,
    /// Dilim açıları eşit, yarıçap değerle orantılı (`'area'`).
    Alan,
}

/// Saçılım sembol boyutu: sabit ya da veriye bağlı işlev (`symbolSize`).
#[derive(Clone)]
pub enum SembolBoyutu {
    Sabit(f32),
    İşlev(Arc<dyn Fn(&VeriÖğesi) -> f32 + Send + Sync>),
    Bağlamlıİşlev(Arc<dyn Fn(&VeriÖğesi, VeriİşlevBağlamı) -> f32 + Send + Sync>),
}

impl SembolBoyutu {
    /// Eski tek-parametreli API için bağlamsız çözüm. Bağlamlı işlevlerde
    /// veri sırası `0` kabul edilir; çizim hattı her zaman
    /// [`Self::bağlamla_çöz`] kullanır.
    pub fn çöz(&self, öğe: &VeriÖğesi) -> f32 {
        self.bağlamla_çöz(öğe, 0)
    }

    pub fn bağlamla_çöz(&self, öğe: &VeriÖğesi, veri_sırası: usize) -> f32 {
        match self {
            SembolBoyutu::Sabit(b) => *b,
            SembolBoyutu::İşlev(f) => f(öğe),
            SembolBoyutu::Bağlamlıİşlev(f) => f(öğe, VeriİşlevBağlamı { veri_sırası }),
        }
    }
}

impl fmt::Debug for SembolBoyutu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SembolBoyutu::Sabit(b) => f.debug_tuple("Sabit").field(b).finish(),
            SembolBoyutu::İşlev(_) => f.write_str("İşlev(..)"),
            SembolBoyutu::Bağlamlıİşlev(_) => f.write_str("Bağlamlıİşlev(..)"),
        }
    }
}

impl From<f32> for SembolBoyutu {
    fn from(b: f32) -> Self {
        SembolBoyutu::Sabit(b)
    }
}

/// ECharts veri callback'lerindeki `params.dataIndex` karşılığı.
///
/// Callback'in aldığı [`VeriÖğesi`] ham `value`/öğe seçeneklerini taşırken
/// bu bağlam, öğenin seri içindeki özgün sırasını verir. Filtreleme ve kırpma
/// sonrasında da sıra değişmez.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VeriİşlevBağlamı {
    pub veri_sırası: usize,
}

/// Saçılım `itemStyle.color` callback'i. Düz renklerin yanında ECharts
/// grafik gradyanları ve görüntü desenleri de [`Dolgu`] olarak döndürülebilir.
#[derive(Clone)]
pub struct ÖğeRengiİşlevi(Arc<dyn Fn(&VeriÖğesi, VeriİşlevBağlamı) -> Dolgu + Send + Sync>);

impl ÖğeRengiİşlevi {
    fn yeni(
        işlev: impl Fn(&VeriÖğesi, VeriİşlevBağlamı) -> Dolgu + Send + Sync + 'static
    ) -> Self {
        Self(Arc::new(işlev))
    }

    pub fn çöz(&self, öğe: &VeriÖğesi, veri_sırası: usize) -> Dolgu {
        (self.0)(öğe, VeriİşlevBağlamı { veri_sırası })
    }
}

impl fmt::Debug for ÖğeRengiİşlevi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ÖğeRengiİşlevi(..)")
    }
}

/// Çizgi serisi (`series-line`).
#[derive(Clone, Debug)]
pub struct ÇizgiSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    /// `0.0` düz çizgi; `true` karşılığı `0.5`tir (ECharts `smooth`).
    pub yumuşaklık: f32,
    pub basamak: Option<Basamak>,
    pub sembol: Sembol,
    pub sembol_boyutu: f32,
    pub sembol_göster: bool,
    /// `showAllSymbol`: `None`, ECharts'ın `'auto'` kuralıdır; kategori
    /// aralığı daraldığında semboller eksen etiketleriyle seyreltilir.
    pub tüm_sembolleri_göster: Option<bool>,
    pub çizgi_stili: ÇizgiStili,
    pub öğe_stili: ÖğeStili,
    pub alan_stili: Option<AlanStili>,
    pub yığın: Option<String>,
    pub boşları_bağla: bool,
    pub etiket: Etiket,
    /// Çizginin son görünür noktasına bağlı etiket (`endLabel`).
    pub uç_etiketi: Etiket,
    /// `labelLayout.moveOverlap: 'shiftY'`: aynı eksen çiftindeki çizgi
    /// etiketlerini dikeyde çakışmayacak biçimde kaydırır.
    pub etiket_örtüşmesini_dikey_kaydır: bool,
    pub imleyiciler: İmleyiciler,
    /// Büyük veri örneklemesi (`sampling`).
    pub örnekleme: Option<Örnekleme>,
    /// Bağlı eksenler (`xAxisIndex`/`yAxisIndex`).
    pub eksen_bağı: EksenBağı,
    /// Kutupsal koordinatta çizilir (`coordinateSystem: 'polar'`).
    pub kutupsal: bool,
    /// Bağlı kutupsal koordinat sırası (`polarIndex`).
    pub kutupsal_sırası: usize,
    /// Veri kümesi eşlemesi: `(ad/kategori boyutu, değer boyutu)` (`encode`).
    pub eşleme: Option<(String, String)>,
    /// Bağlı `dataset` dizisi sırası (`datasetIndex`).
    pub veri_kümesi_sırası: usize,
    /// `seriesLayoutBy`: ortak kaynağı sütun ya da satır serileri olarak oku.
    pub seri_yerleşimi: SeriYerleşimi,
}

impl Default for ÇizgiSerisi {
    fn default() -> Self {
        ÇizgiSerisi {
            ad: None,
            veri: Vec::new(),
            yumuşaklık: 0.0,
            basamak: None,
            sembol: Sembol::İçiBoşDaire,
            // ECharts 6.1 `LineSeries.defaultOption.symbolSize`.
            sembol_boyutu: 6.0,
            sembol_göster: true,
            tüm_sembolleri_göster: None,
            çizgi_stili: ÇizgiStili::default(),
            öğe_stili: ÖğeStili::default(),
            alan_stili: None,
            yığın: None,
            boşları_bağla: false,
            etiket: Etiket {
                // ECharts LineSeries.defaultOption.label.position = 'top'.
                konum: crate::model::stil::EtiketKonumu::Üst,
                ..Etiket::default()
            },
            uç_etiketi: Etiket {
                uzaklık: 8.0,
                ..Etiket::default()
            },
            etiket_örtüşmesini_dikey_kaydır: false,
            imleyiciler: İmleyiciler::default(),
            örnekleme: None,
            eksen_bağı: EksenBağı::default(),
            kutupsal: false,
            kutupsal_sırası: 0,
            eşleme: None,
            veri_kümesi_sırası: 0,
            seri_yerleşimi: SeriYerleşimi::Sütun,
        }
    }
}

impl ÇizgiSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Seriyi veri kümesine bağlar: `(ad/kategori boyutu, değer boyutu)`.
    pub fn eşle(mut self, ad_boyutu: impl Into<String>, değer_boyutu: impl Into<String>) -> Self {
        self.eşleme = Some((ad_boyutu.into(), değer_boyutu.into()));
        self
    }

    pub fn veri_kümesi_sırası(mut self, sıra: usize) -> Self {
        self.veri_kümesi_sırası = sıra;
        self
    }

    pub fn seri_yerleşimi(mut self, yerleşim: SeriYerleşimi) -> Self {
        self.seri_yerleşimi = yerleşim;
        self
    }

    /// Seriyi kutupsal koordinata bağlar (`coordinateSystem: 'polar'`).
    pub fn kutupsal(mut self, açık: bool) -> Self {
        self.kutupsal = açık;
        self
    }

    /// Seriyi belirtilen `polarIndex` bileşenine bağlar.
    pub fn kutupsal_sırası(mut self, sıra: usize) -> Self {
        self.kutupsal = true;
        self.kutupsal_sırası = sıra;
        self
    }

    /// Seriyi verilen x/y eksen sıralarına bağlar (`xAxisIndex`/`yAxisIndex`).
    pub fn eksenler(mut self, x: usize, y: usize) -> Self {
        self.eksen_bağı = EksenBağı { x, y };
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self
    }

    /// `yumuşat(true)` ⇒ ECharts'taki `smooth: true` (0.5).
    pub fn yumuşat(mut self, açık: bool) -> Self {
        self.yumuşaklık = if açık { 0.5 } else { 0.0 };
        self
    }

    pub fn yumuşaklık(mut self, oran: f32) -> Self {
        self.yumuşaklık = oran.clamp(0.0, 1.0);
        self
    }

    pub fn basamak(mut self, basamak: Basamak) -> Self {
        self.basamak = Some(basamak);
        self
    }

    pub fn sembol(mut self, sembol: Sembol) -> Self {
        self.sembol = sembol;
        self
    }

    pub fn sembol_boyutu(mut self, boyut: f32) -> Self {
        self.sembol_boyutu = boyut;
        self
    }

    pub fn sembol_göster(mut self, göster: bool) -> Self {
        self.sembol_göster = göster;
        self
    }

    pub fn tüm_sembolleri_göster(mut self, göster: bool) -> Self {
        self.tüm_sembolleri_göster = Some(göster);
        self
    }

    /// `showAllSymbol: 'auto'` davranışına döner.
    pub fn sembolleri_otomatik_seyrelt(mut self) -> Self {
        self.tüm_sembolleri_göster = None;
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = stil;
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn alan_stili(mut self, stil: AlanStili) -> Self {
        self.alan_stili = Some(stil);
        self
    }

    pub fn yığın(mut self, yığın: impl Into<String>) -> Self {
        self.yığın = Some(yığın.into());
        self
    }

    pub fn boşları_bağla(mut self, bağla: bool) -> Self {
        self.boşları_bağla = bağla;
        self
    }

    pub fn örnekleme(mut self, örnekleme: Örnekleme) -> Self {
        self.örnekleme = Some(örnekleme);
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn uç_etiketi(mut self, etiket: Etiket) -> Self {
        self.uç_etiketi = etiket;
        self
    }

    pub fn etiket_örtüşmesini_dikey_kaydır(mut self, kaydır: bool) -> Self {
        self.etiket_örtüşmesini_dikey_kaydır = kaydır;
        self
    }

    pub fn im_çizgisi(mut self, im: İmÇizgisi) -> Self {
        self.imleyiciler.çizgi = Some(im);
        self
    }

    pub fn im_noktası(mut self, im: İmNoktası) -> Self {
        self.imleyiciler.nokta = Some(im);
        self
    }

    pub fn im_alanı(mut self, im: İmAlanı) -> Self {
        self.imleyiciler.alan = Some(im);
        self
    }
}

/// Sütun serisi (`series-bar`).
#[derive(Clone, Debug)]
pub struct SütunSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub yığın: Option<String>,
    /// ZRender çizim sırası (`series.bar.z`), ECharts öntanımlısı 2.
    pub z: i32,
    /// İşaretçi/isabet olaylarını kapatır (`series.bar.silent`).
    pub sessiz: bool,
    /// Teğetsel polar sütunun iki ucunu yarım daireyle kapatır
    /// (`series.bar.roundCap`). Kartezyen ve radyal polar sütunlarda etkisizdir.
    pub yuvarlak_uç: bool,
    /// `barWidth` — verilmezse otomatik hesaplanır.
    pub genişlik: Option<Uzunluk>,
    /// `barMaxWidth`.
    pub en_çok_genişlik: Option<Uzunluk>,
    /// `barMinWidth`.
    pub en_az_genişlik: Option<Uzunluk>,
    /// Aynı kategorideki sütunlar arasındaki boşluk, sütun genişliğine
    /// oranla (`barGap`, öntanımlı `'30%'`).
    pub sütun_boşluğu: Option<Uzunluk>,
    /// Kategoriler arasındaki boşluk, bant genişliğine oranla
    /// (`barCategoryGap`).
    pub kategori_boşluğu: Option<Uzunluk>,
    pub öğe_stili: ÖğeStili,
    /// Fare/odak vurgusunda normal stile bindirilen
    /// `emphasis.itemStyle`.
    pub vurgu_öğe_stili: ÖğeStili,
    /// `showBackground`: her veri sütununun değer ekseni boyunca arka planı.
    pub arka_plan_göster: bool,
    /// `backgroundStyle`; `None`, ECharts'ın yarı saydam gri varsayılanıdır.
    pub arka_plan_stili: Option<ÖğeStili>,
    pub etiket: Etiket,
    pub imleyiciler: İmleyiciler,
    /// Sütunu tekrarlanan sembollerle çizer (`pictorialBar`).
    pub piktogram: Option<Piktogram>,
    /// Bağlı eksenler (`xAxisIndex`/`yAxisIndex`).
    pub eksen_bağı: EksenBağı,
    /// Kutupsal koordinatta çizilir (`coordinateSystem: 'polar'`).
    pub kutupsal: bool,
    /// Bağlı kutupsal koordinat sırası (`polarIndex`).
    pub kutupsal_sırası: usize,
    /// Veri kümesi eşlemesi: `(x boyutu, y boyutu)` (`encode.x/y`).
    pub eşleme: Option<(String, String)>,
    /// Bağlı `dataset` dizisi sırası (`datasetIndex`).
    pub veri_kümesi_sırası: usize,
    /// `seriesLayoutBy`.
    pub seri_yerleşimi: SeriYerleşimi,
    /// Büyük veri için tek bir toplu yol kullanır (`large`). ECharts bar
    /// serisinde bu kip açıkça etkinleştirilir.
    pub büyük: bool,
    /// Toplu çizime geçilecek veri sayısı (`largeThreshold`).
    pub büyük_eşiği: usize,
    /// Bir artımlı çizim parçasındaki öğe sayısı (`progressive`).
    pub aşamalı: usize,
    /// Artımlı işleme eşiği (`progressiveThreshold`).
    pub aşamalı_eşiği: usize,
}

impl Default for SütunSerisi {
    fn default() -> Self {
        SütunSerisi {
            ad: None,
            veri: Vec::new(),
            yığın: None,
            z: 2,
            sessiz: false,
            yuvarlak_uç: false,
            genişlik: None,
            en_çok_genişlik: None,
            en_az_genişlik: None,
            sütun_boşluğu: None,
            kategori_boşluğu: None,
            öğe_stili: ÖğeStili::default(),
            vurgu_öğe_stili: ÖğeStili::default(),
            arka_plan_göster: false,
            arka_plan_stili: None,
            // zrender'in bağlı metin öntanımlısı `inside`dır. Line serisi
            // kendi `top` öntanımlısını taşırken BarSeries taşımaz.
            etiket: Etiket {
                konum: crate::model::stil::EtiketKonumu::İç,
                ..Etiket::default()
            },
            imleyiciler: İmleyiciler::default(),
            piktogram: None,
            eksen_bağı: EksenBağı::default(),
            kutupsal: false,
            kutupsal_sırası: 0,
            eşleme: None,
            veri_kümesi_sırası: 0,
            seri_yerleşimi: SeriYerleşimi::Sütun,
            büyük: false,
            büyük_eşiği: 400,
            aşamalı: 3_000,
            aşamalı_eşiği: 3_000,
        }
    }
}

impl SütunSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Seriyi veri kümesine bağlar: `(x boyutu, y boyutu)` (`encode.x/y`).
    ///
    /// X boyutu sayısal bir eksene sayı, kategori eksenine sırasal değer
    /// olarak akar. Dataset'in diğer boyutları tooltip/visualMap için veri
    /// öğesinde korunur.
    pub fn eşle(mut self, x_boyutu: impl Into<String>, y_boyutu: impl Into<String>) -> Self {
        self.eşleme = Some((x_boyutu.into(), y_boyutu.into()));
        self
    }

    pub fn veri_kümesi_sırası(mut self, sıra: usize) -> Self {
        self.veri_kümesi_sırası = sıra;
        self
    }

    pub fn seri_yerleşimi(mut self, yerleşim: SeriYerleşimi) -> Self {
        self.seri_yerleşimi = yerleşim;
        self
    }

    /// ECharts `series.bar.large`.
    pub fn büyük(mut self, açık: bool) -> Self {
        self.büyük = açık;
        self
    }

    pub fn büyük_eşiği(mut self, eşik: usize) -> Self {
        self.büyük_eşiği = eşik;
        self
    }

    pub fn aşamalı(mut self, parça: usize) -> Self {
        self.aşamalı = parça.max(1);
        self
    }

    pub fn aşamalı_eşiği(mut self, eşik: usize) -> Self {
        self.aşamalı_eşiği = eşik;
        self
    }

    /// Seriyi kutupsal koordinata bağlar (`coordinateSystem: 'polar'`).
    pub fn kutupsal(mut self, açık: bool) -> Self {
        self.kutupsal = açık;
        self
    }

    /// Seriyi belirtilen `polarIndex` bileşenine bağlar.
    pub fn kutupsal_sırası(mut self, sıra: usize) -> Self {
        self.kutupsal = true;
        self.kutupsal_sırası = sıra;
        self
    }

    /// Seriyi verilen x/y eksen sıralarına bağlar (`xAxisIndex`/`yAxisIndex`).
    pub fn eksenler(mut self, x: usize, y: usize) -> Self {
        self.eksen_bağı = EksenBağı { x, y };
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self
    }

    pub fn yığın(mut self, yığın: impl Into<String>) -> Self {
        self.yığın = Some(yığın.into());
        self
    }

    pub fn z(mut self, z: i32) -> Self {
        self.z = z;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }

    pub fn yuvarlak_uç(mut self, yuvarlak: bool) -> Self {
        self.yuvarlak_uç = yuvarlak;
        self
    }

    pub fn genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(genişlik.into());
        self
    }

    pub fn en_çok_genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.en_çok_genişlik = Some(genişlik.into());
        self
    }

    pub fn en_az_genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.en_az_genişlik = Some(genişlik.into());
        self
    }

    pub fn sütun_boşluğu(mut self, boşluk: impl Into<Uzunluk>) -> Self {
        self.sütun_boşluğu = Some(boşluk.into());
        self
    }

    pub fn kategori_boşluğu(mut self, boşluk: impl Into<Uzunluk>) -> Self {
        self.kategori_boşluğu = Some(boşluk.into());
        self
    }

    /// Sütunu tekrarlanan sembollerle çizer (`pictorialBar` karşılığı).
    pub fn piktogram(mut self, piktogram: Piktogram) -> Self {
        self.piktogram = Some(piktogram);
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn vurgu_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.vurgu_öğe_stili = stil;
        self
    }

    pub fn arka_plan_göster(mut self, göster: bool) -> Self {
        self.arka_plan_göster = göster;
        self
    }

    pub fn arka_plan_stili(mut self, stil: ÖğeStili) -> Self {
        self.arka_plan_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn im_çizgisi(mut self, im: İmÇizgisi) -> Self {
        self.imleyiciler.çizgi = Some(im);
        self
    }

    pub fn im_noktası(mut self, im: İmNoktası) -> Self {
        self.imleyiciler.nokta = Some(im);
        self
    }

    pub fn im_alanı(mut self, im: İmAlanı) -> Self {
        self.imleyiciler.alan = Some(im);
        self
    }
}

/// Etiket kılavuz çizgisi (`labelLine`). Pasta dış etiketleri ve taşınmış
/// scatter etiketleri aynı ECharts çizgi modeliyle çizilir.
#[derive(Clone, PartialEq, Debug)]
pub struct EtiketÇizgisi {
    pub göster: bool,
    /// Dilimden dışa uzanan ilk parça (`length`).
    pub uzunluk1: f32,
    /// Yatay ikinci parça (`length2`).
    pub uzunluk2: f32,
    /// `false`/`true`/sayısal `smooth`; 0 düz, 0.3 ECharts `true` karşılığı.
    pub yumuşaklık: f32,
    pub en_küçük_dönüş_açısı: f32,
    pub en_büyük_yüzey_açısı: f32,
    pub stil: ÇizgiStili,
}

impl Default for EtiketÇizgisi {
    fn default() -> Self {
        EtiketÇizgisi {
            göster: true,
            uzunluk1: 15.0,
            // ECharts `PieSeries.defaultOption.labelLine.length2`.
            uzunluk2: 30.0,
            yumuşaklık: 0.0,
            en_küçük_dönüş_açısı: 90.0,
            en_büyük_yüzey_açısı: 90.0,
            stil: ÇizgiStili {
                kalınlık: 1.0,
                ..Default::default()
            },
        }
    }
}

impl EtiketÇizgisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn uzunluk1(mut self, uzunluk: f32) -> Self {
        self.uzunluk1 = uzunluk.max(0.0);
        self
    }

    pub fn uzunluk2(mut self, uzunluk: f32) -> Self {
        self.uzunluk2 = uzunluk.max(0.0);
        self
    }

    pub fn yumuşaklık(mut self, yumuşaklık: f32) -> Self {
        self.yumuşaklık = yumuşaklık.clamp(0.0, 1.0);
        self
    }

    pub fn en_küçük_dönüş_açısı(mut self, derece: f32) -> Self {
        self.en_küçük_dönüş_açısı = derece.clamp(0.0, 180.0);
        self
    }

    pub fn en_büyük_yüzey_açısı(mut self, derece: f32) -> Self {
        self.en_büyük_yüzey_açısı = derece.clamp(0.0, 180.0);
        self
    }

    pub fn stil(mut self, stil: ÇizgiStili) -> Self {
        self.stil = stil;
        self
    }
}

/// `series.labelLayout` geri çağrısına verilen, çizimden bağımsız yerleşim
/// bilgisi. Koordinatlar grafik yüzeyinin mantıksal piksel uzayındadır.
#[derive(Clone, Debug, PartialEq)]
pub struct EtiketYerleşimParametreleri {
    pub veri_sırası: usize,
    pub veri_adı: String,
    pub değer: f64,
    pub etiket_kutusu: Dikdörtgen,
    pub etiket_çizgisi_noktaları: Option<[(f32, f32); 3]>,
}

/// `series.labelLayout.moveOverlap` ekseni.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EtiketÖrtüşmeKaydırması {
    /// Etiketleri taşıma.
    #[default]
    Yok,
    /// Yatay sırayı koruyarak çakışmaları gider (`shiftX`).
    X,
    /// Dikey sırayı koruyarak çakışmaları gider (`shiftY`).
    Y,
}

/// `series.labelLayout` geri çağrısının değiştirebildiği değerler.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EtiketYerleşimSonucu {
    /// Tuval piksel uzayında mutlak etiket x konumu.
    pub x: Option<f32>,
    /// Tuval piksel uzayında mutlak etiket y konumu.
    pub y: Option<f32>,
    pub yatay_hiza: Option<crate::model::stil::YazıYatayHizası>,
    pub dikey_hiza: Option<crate::model::stil::YazıDikeyHizası>,
    pub örtüşme_kaydırması: EtiketÖrtüşmeKaydırması,
    /// Taşımadan sonra kalan çakışmalarda düşük öncelikli etiketi gizler.
    pub çakışanı_gizle: bool,
    pub etiket_çizgisi_noktaları: Option<[(f32, f32); 3]>,
}

impl EtiketYerleşimSonucu {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn x(mut self, x: f32) -> Self {
        self.x = Some(x);
        self
    }

    pub fn y(mut self, y: f32) -> Self {
        self.y = Some(y);
        self
    }

    pub fn yatay_hiza(mut self, hiza: crate::model::stil::YazıYatayHizası) -> Self {
        self.yatay_hiza = Some(hiza);
        self
    }

    pub fn dikey_hiza(mut self, hiza: crate::model::stil::YazıDikeyHizası) -> Self {
        self.dikey_hiza = Some(hiza);
        self
    }

    pub fn örtüşme_kaydırması(mut self, kaydırma: EtiketÖrtüşmeKaydırması) -> Self {
        self.örtüşme_kaydırması = kaydırma;
        self
    }

    pub fn çakışanı_gizle(mut self, gizle: bool) -> Self {
        self.çakışanı_gizle = gizle;
        self
    }

    pub fn etiket_çizgisi_noktaları(mut self, noktalar: [(f32, f32); 3]) -> Self {
        self.etiket_çizgisi_noktaları = Some(noktalar);
        self
    }
}

type EtiketYerleşimGeriÇağrısı =
    Arc<dyn Fn(&EtiketYerleşimParametreleri) -> EtiketYerleşimSonucu + Send + Sync>;

/// Klonlanabilir `labelLayout` işlevi sarmalayıcısı.
#[derive(Clone)]
pub struct EtiketYerleşimİşlevi(EtiketYerleşimGeriÇağrısı);

impl EtiketYerleşimİşlevi {
    pub fn yeni(
        işlev: impl Fn(&EtiketYerleşimParametreleri) -> EtiketYerleşimSonucu + Send + Sync + 'static,
    ) -> Self {
        Self(Arc::new(işlev))
    }

    pub fn uygula(&self, parametreler: &EtiketYerleşimParametreleri) -> EtiketYerleşimSonucu {
        (self.0)(parametreler)
    }
}

impl fmt::Debug for EtiketYerleşimİşlevi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EtiketYerleşimİşlevi(..)")
    }
}

/// Pasta serisi (`series-pie`).
#[derive(Clone, Debug)]
pub struct PastaSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    /// Takvim koordinatına bağlıysa `calendarIndex`; `None`, klasik
    /// görünüm kutusu yerleşimidir.
    pub takvim_sırası: Option<usize>,
    /// `coordinateSystem: 'calendar'` kullanımındaki tarih merkezidir.
    pub takvim_merkez_tarihi: Option<f64>,
    /// Matrix koordinatına bağlıysa `matrixIndex`.
    pub matris_sırası: Option<usize>,
    /// `coordinateSystem: 'matrix'` kullanımındaki hücre/aralık merkezi.
    pub matris_merkezi: Option<(MatrisAralığı, MatrisAralığı)>,
    /// Seri görünüm kutusu (`left/right/top/bottom/width/height`). Yüzdeler
    /// ana çizim alanına göre çözülür.
    pub sol: Uzunluk,
    pub sağ: Uzunluk,
    pub üst: Uzunluk,
    pub alt: Uzunluk,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    /// `(iç, dış)` yarıçap; ECharts öntanımlısı `[0, '50%']`.
    pub yarıçap: (Uzunluk, Uzunluk),
    /// Merkez `(x, y)`; öntanımlı `('50%', '50%')`.
    pub merkez: (Uzunluk, Uzunluk),
    /// Derece cinsinden başlangıç açısı (`startAngle`, öntanımlı 90).
    pub başlangıç_açısı: f32,
    /// Derece cinsinden bitiş açısı; `None`, ECharts `'auto'` karşılığıdır.
    pub bitiş_açısı: Option<f32>,
    pub saat_yönünde: bool,
    /// Dilimler arasındaki açı boşluğu (`padAngle`, derece).
    pub dolgu_açısı: f32,
    /// En küçük dilim açısı (`minAngle`, derece).
    pub en_küçük_açı: f32,
    /// Bundan dar dilimlerin etiketi gösterilmez (`minShowLabelAngle`).
    pub en_küçük_etiket_açısı: f32,
    /// Seçili dilimin radyal kayması (`selectedOffset`).
    pub seçili_uzaklığı: f32,
    pub gül_türü: Option<GülTürü>,
    /// Tüm değerler sıfırken eşit dilimler gösterilsin.
    pub sıfır_toplamı_göster: bool,
    /// Geçerli veri kalmadığında boş halka gösterilsin.
    pub boş_daire_göster: bool,
    pub boş_daire_stili: ÖğeStili,
    /// Dış etiket çakışmalarını taşı/gizle (`avoidLabelOverlap`).
    pub etiket_çakışmasını_önle: bool,
    /// Tooltip/etiket yüzde hassasiyeti (`percentPrecision`).
    pub yüzde_hassasiyeti: u8,
    pub öğe_stili: ÖğeStili,
    pub etiket: Etiket,
    pub etiket_çizgisi: EtiketÇizgisi,
    pub etiket_yerleşimi: Option<EtiketYerleşimİşlevi>,
    /// Veri kümesi eşlemesi: `(ad boyutu, değer boyutu)` (`encode`).
    pub eşleme: Option<(String, String)>,
    /// Bağlı `dataset` dizisi sırası (`datasetIndex`).
    pub veri_kümesi_sırası: usize,
    /// `seriesLayoutBy`.
    pub seri_yerleşimi: SeriYerleşimi,
}

impl Default for PastaSerisi {
    fn default() -> Self {
        PastaSerisi {
            ad: None,
            veri: Vec::new(),
            takvim_sırası: None,
            takvim_merkez_tarihi: None,
            matris_sırası: None,
            matris_merkezi: None,
            sol: Uzunluk::Piksel(0.0),
            sağ: Uzunluk::Piksel(0.0),
            üst: Uzunluk::Piksel(0.0),
            alt: Uzunluk::Piksel(0.0),
            genişlik: None,
            yükseklik: None,
            yarıçap: (Uzunluk::Yüzde(0.0), Uzunluk::Yüzde(50.0)),
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(50.0)),
            başlangıç_açısı: 90.0,
            bitiş_açısı: None,
            saat_yönünde: true,
            dolgu_açısı: 0.0,
            en_küçük_açı: 0.0,
            en_küçük_etiket_açısı: 0.0,
            seçili_uzaklığı: 10.0,
            gül_türü: None,
            sıfır_toplamı_göster: true,
            boş_daire_göster: true,
            boş_daire_stili: ÖğeStili::yeni().renk(0xd3d3d3),
            etiket_çakışmasını_önle: true,
            yüzde_hassasiyeti: 2,
            öğe_stili: ÖğeStili::default(),
            etiket: Etiket {
                göster: true,
                konum: crate::model::stil::EtiketKonumu::Dış,
                ..Default::default()
            },
            etiket_çizgisi: EtiketÇizgisi::default(),
            etiket_yerleşimi: None,
            eşleme: None,
            veri_kümesi_sırası: 0,
            seri_yerleşimi: SeriYerleşimi::Sütun,
        }
    }
}

impl PastaSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Seriyi veri kümesine bağlar: `(ad/kategori boyutu, değer boyutu)`.
    pub fn eşle(mut self, ad_boyutu: impl Into<String>, değer_boyutu: impl Into<String>) -> Self {
        self.eşleme = Some((ad_boyutu.into(), değer_boyutu.into()));
        self
    }

    pub fn veri_kümesi_sırası(mut self, sıra: usize) -> Self {
        self.veri_kümesi_sırası = sıra;
        self
    }

    pub fn seri_yerleşimi(mut self, yerleşim: SeriYerleşimi) -> Self {
        self.seri_yerleşimi = yerleşim;
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self
    }

    /// Pastayı belirtilen `calendarIndex` bileşenine bağlar.
    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = Some(sıra);
        self.matris_sırası = None;
        self.matris_merkezi = None;
        self
    }

    /// Pasta merkezini takvimdeki bir tarihe bağlar. Açık bir
    /// `calendarIndex` verilmediyse ECharts gibi sıfırıncı takvim seçilir.
    pub fn takvim_merkezi(mut self, tarih_ms: f64) -> Self {
        self.takvim_sırası.get_or_insert(0);
        self.takvim_merkez_tarihi = Some(tarih_ms);
        self.matris_sırası = None;
        self.matris_merkezi = None;
        self
    }

    /// Pastayı belirtilen `matrixIndex` bileşenine bağlar.
    pub fn matris_sırası(mut self, sıra: usize) -> Self {
        self.matris_sırası = Some(sıra);
        self.takvim_sırası = None;
        self.takvim_merkez_tarihi = None;
        self
    }

    /// Pasta merkezini Matrix hücresi ya da birleşik aralığına bağlar.
    pub fn matris_merkezi(
        mut self,
        x: impl Into<MatrisAralığı>,
        y: impl Into<MatrisAralığı>,
    ) -> Self {
        self.matris_sırası.get_or_insert(0);
        self.matris_merkezi = Some((x.into(), y.into()));
        self.takvim_sırası = None;
        self.takvim_merkez_tarihi = None;
        self
    }

    pub fn sol(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sol = değer.into();
        self
    }

    pub fn sağ(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sağ = değer.into();
        self
    }

    pub fn üst(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.üst = değer.into();
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

    pub fn görünüm_kutusu(
        mut self,
        sol: impl Into<Uzunluk>,
        sağ: impl Into<Uzunluk>,
        üst: impl Into<Uzunluk>,
        alt: impl Into<Uzunluk>,
    ) -> Self {
        self.sol = sol.into();
        self.sağ = sağ.into();
        self.üst = üst.into();
        self.alt = alt.into();
        self
    }

    /// Tek değer verilirse iç yarıçap 0 kabul edilir.
    pub fn yarıçap(mut self, dış: impl Into<Uzunluk>) -> Self {
        self.yarıçap = (Uzunluk::Piksel(0.0), dış.into());
        self
    }

    /// Halka (donut) için `(iç, dış)` yarıçap.
    pub fn halka(mut self, iç: impl Into<Uzunluk>, dış: impl Into<Uzunluk>) -> Self {
        self.yarıçap = (iç.into(), dış.into());
        self
    }

    pub fn merkez(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.merkez = (x.into(), y.into());
        self
    }

    pub fn başlangıç_açısı(mut self, derece: f32) -> Self {
        self.başlangıç_açısı = derece;
        self
    }

    pub fn bitiş_açısı(mut self, derece: f32) -> Self {
        self.bitiş_açısı = Some(derece);
        self
    }

    pub fn otomatik_bitiş_açısı(mut self) -> Self {
        self.bitiş_açısı = None;
        self
    }

    pub fn saat_yönünde(mut self, saat_yönünde: bool) -> Self {
        self.saat_yönünde = saat_yönünde;
        self
    }

    pub fn dolgu_açısı(mut self, derece: f32) -> Self {
        self.dolgu_açısı = derece.max(0.0);
        self
    }

    pub fn en_küçük_açı(mut self, derece: f32) -> Self {
        self.en_küçük_açı = derece.max(0.0);
        self
    }

    pub fn en_küçük_etiket_açısı(mut self, derece: f32) -> Self {
        self.en_küçük_etiket_açısı = derece.max(0.0);
        self
    }

    pub fn seçili_uzaklığı(mut self, uzaklık: f32) -> Self {
        self.seçili_uzaklığı = uzaklık.max(0.0);
        self
    }

    pub fn gül_türü(mut self, tür: GülTürü) -> Self {
        self.gül_türü = Some(tür);
        self
    }

    pub fn sıfır_toplamı_göster(mut self, göster: bool) -> Self {
        self.sıfır_toplamı_göster = göster;
        self
    }

    pub fn boş_daire_göster(mut self, göster: bool) -> Self {
        self.boş_daire_göster = göster;
        self
    }

    pub fn boş_daire_stili(mut self, stil: ÖğeStili) -> Self {
        self.boş_daire_stili = stil;
        self
    }

    pub fn etiket_çakışmasını_önle(mut self, önle: bool) -> Self {
        self.etiket_çakışmasını_önle = önle;
        self
    }

    pub fn yüzde_hassasiyeti(mut self, basamak: u8) -> Self {
        self.yüzde_hassasiyeti = basamak.min(20);
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

    pub fn etiket_çizgisi(mut self, çizgi: EtiketÇizgisi) -> Self {
        self.etiket_çizgisi = çizgi;
        self
    }

    pub fn etiket_yerleşimi(
        mut self,
        işlev: impl Fn(&EtiketYerleşimParametreleri) -> EtiketYerleşimSonucu + Send + Sync + 'static,
    ) -> Self {
        self.etiket_yerleşimi = Some(EtiketYerleşimİşlevi::yeni(işlev));
        self
    }
}

/// Mum serisi (`series-candlestick`). Veri öğeleri
/// `[açılış, kapanış, en düşük, en yüksek]` dizileridir.
#[derive(Clone, Debug)]
pub struct MumSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    /// Yükselen (kapanış ≥ açılış) mum rengi (`itemStyle.color`).
    pub yükselen_renk: crate::renk::Renk,
    /// Düşen mum rengi (`itemStyle.color0`).
    pub düşen_renk: crate::renk::Renk,
    /// Yükselen mumun fitil/gövde kenarlığı (`itemStyle.borderColor`).
    pub yükselen_kenarlık_rengi: crate::renk::Renk,
    /// Düşen mumun fitil/gövde kenarlığı (`itemStyle.borderColor0`).
    pub düşen_kenarlık_rengi: crate::renk::Renk,
    /// Gövde genişliğinin bant genişliğine oranı. ECharts, açık bir
    /// `barWidth` verilmediğinde bant genişliğinin yarısını kullanır.
    pub gövde_oranı: f32,
    pub kenarlık_kalınlığı: f32,
    pub imleyiciler: İmleyiciler,
    /// Bağlı eksenler (`xAxisIndex`/`yAxisIndex`).
    pub eksen_bağı: EksenBağı,
    /// Veri kümesi eşlemesi: kategori boyutu ile
    /// `[open, close, lowest, highest]` boyutları (`encode.x/y`).
    pub eşleme: Option<(String, [String; 4])>,
    /// Bağlı `dataset` dizisi sırası (`datasetIndex`).
    pub veri_kümesi_sırası: usize,
    /// `seriesLayoutBy`.
    pub seri_yerleşimi: SeriYerleşimi,
    /// ECharts candlestick öntanımlısı gibi büyük veri çizimini açar.
    pub büyük: bool,
    /// Toplu çizime geçilecek veri sayısı (`largeThreshold`).
    pub büyük_eşiği: usize,
    /// Bir artımlı çizim parçasındaki öğe sayısı (`progressive`).
    pub aşamalı: usize,
    /// Artımlı işleme eşiği (`progressiveThreshold`).
    pub aşamalı_eşiği: usize,
}

impl Default for MumSerisi {
    fn default() -> Self {
        MumSerisi {
            ad: None,
            veri: Vec::new(),
            // ECharts v5 öntanımlıları: color '#eb5454', color0 '#47b262'.
            yükselen_renk: crate::renk::Renk::onaltılık(0xeb5454),
            düşen_renk: crate::renk::Renk::onaltılık(0x47b262),
            yükselen_kenarlık_rengi: crate::renk::Renk::onaltılık(0xeb5454),
            düşen_kenarlık_rengi: crate::renk::Renk::onaltılık(0x47b262),
            gövde_oranı: 0.5,
            kenarlık_kalınlığı: 1.0,
            imleyiciler: İmleyiciler::default(),
            eksen_bağı: EksenBağı::default(),
            eşleme: None,
            veri_kümesi_sırası: 0,
            seri_yerleşimi: SeriYerleşimi::Sütun,
            büyük: true,
            büyük_eşiği: 600,
            aşamalı: 3_000,
            aşamalı_eşiği: 10_000,
        }
    }
}

impl MumSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Seriyi veri kümesine bağlar. `y_boyutları` sırası ECharts
    /// `encode.y: [open, close, lowest, highest]` ile aynıdır.
    pub fn eşle<X, Y>(mut self, x_boyutu: X, y_boyutları: [Y; 4]) -> Self
    where
        X: Into<String>,
        Y: Into<String>,
    {
        self.eşleme = Some((x_boyutu.into(), y_boyutları.map(Into::into)));
        self
    }

    pub fn veri_kümesi_sırası(mut self, sıra: usize) -> Self {
        self.veri_kümesi_sırası = sıra;
        self
    }

    pub fn seri_yerleşimi(mut self, yerleşim: SeriYerleşimi) -> Self {
        self.seri_yerleşimi = yerleşim;
        self
    }

    /// Seriyi verilen x/y eksen sıralarına bağlar (`xAxisIndex`/`yAxisIndex`).
    pub fn eksenler(mut self, x: usize, y: usize) -> Self {
        self.eksen_bağı = EksenBağı { x, y };
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    /// Veri: `[açılış, kapanış, en düşük, en yüksek]` dizileri.
    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self
    }

    pub fn yükselen_renk(mut self, renk: impl Into<crate::renk::Renk>) -> Self {
        self.yükselen_renk = renk.into();
        self
    }

    pub fn düşen_renk(mut self, renk: impl Into<crate::renk::Renk>) -> Self {
        self.düşen_renk = renk.into();
        self
    }

    pub fn yükselen_kenarlık_rengi(mut self, renk: impl Into<crate::renk::Renk>) -> Self {
        self.yükselen_kenarlık_rengi = renk.into();
        self
    }

    pub fn düşen_kenarlık_rengi(mut self, renk: impl Into<crate::renk::Renk>) -> Self {
        self.düşen_kenarlık_rengi = renk.into();
        self
    }

    pub fn büyük(mut self, açık: bool) -> Self {
        self.büyük = açık;
        self
    }

    pub fn büyük_eşiği(mut self, eşik: usize) -> Self {
        self.büyük_eşiği = eşik;
        self
    }

    pub fn aşamalı(mut self, parça: usize) -> Self {
        self.aşamalı = parça.max(1);
        self
    }

    pub fn aşamalı_eşiği(mut self, eşik: usize) -> Self {
        self.aşamalı_eşiği = eşik;
        self
    }

    pub fn im_çizgisi(mut self, im: İmÇizgisi) -> Self {
        self.imleyiciler.çizgi = Some(im);
        self
    }

    pub fn im_noktası(mut self, im: İmNoktası) -> Self {
        self.imleyiciler.nokta = Some(im);
        self
    }
}

/// Kutu serisi (`series-boxplot`). Veri öğeleri
/// `[en düşük, Ç1, ortanca, Ç3, en yüksek]` dizileridir.
#[derive(Clone, Debug)]
pub struct KutuSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub öğe_stili: ÖğeStili,
    /// Gövde genişliğinin bant genişliğine oranı.
    pub gövde_oranı: f32,
    /// Açık oran yokken ECharts'ın bant tabanlı `boxWidth` hesabını kullan.
    pub otomatik_gövde_genişliği: bool,
    pub imleyiciler: İmleyiciler,
    /// Bağlı eksenler (`xAxisIndex`/`yAxisIndex`).
    pub eksen_bağı: EksenBağı,
}

impl Default for KutuSerisi {
    fn default() -> Self {
        KutuSerisi {
            ad: None,
            veri: Vec::new(),
            öğe_stili: ÖğeStili::default(),
            gövde_oranı: 0.5,
            otomatik_gövde_genişliği: true,
            imleyiciler: İmleyiciler::default(),
            eksen_bağı: EksenBağı::default(),
        }
    }
}

impl KutuSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Seriyi verilen x/y eksen sıralarına bağlar (`xAxisIndex`/`yAxisIndex`).
    pub fn eksenler(mut self, x: usize, y: usize) -> Self {
        self.eksen_bağı = EksenBağı { x, y };
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    /// Veri: `[en düşük, Ç1, ortanca, Ç3, en yüksek]` dizileri.
    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn gövde_oranı(mut self, oran: f32) -> Self {
        self.gövde_oranı = oran.clamp(0.05, 1.0);
        self.otomatik_gövde_genişliği = false;
        self
    }

    pub fn otomatik_gövde_genişliği(mut self) -> Self {
        self.otomatik_gövde_genişliği = true;
        self
    }
}

/// Isı haritası serisi (`series-heatmap`, kartezyen kip). Veri öğeleri
/// `[x sırası, y sırası, değer]` dizileridir; her iki eksen de kategorik
/// olmalıdır. Hücre renkleri seçeneklerdeki görsel eşlemeden çözülür.
#[derive(Clone, Debug)]
pub struct IsıHaritasıSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub öğe_stili: ÖğeStili,
    /// `emphasis.itemStyle`; yalnız sağlanan vurgu alanları normal stili
    /// örter. Kartezyen hover ve tooltip vurgusunda uygulanır.
    pub vurgu_öğe_stili: ÖğeStili,
    /// Hücreler arası boşluk, piksel.
    pub hücre_boşluğu: f32,
    pub etiket: Etiket,
    /// Bağlı eksenler (`xAxisIndex`/`yAxisIndex`).
    pub eksen_bağı: EksenBağı,
    /// Matrix koordinatına bağlıysa `matrixIndex`; `None`, kartezyendir.
    pub matris_sırası: Option<usize>,
    /// Veri sırasıyla eşleşen açık Matrix hücre/aralık koordinatları.
    /// Boş girişlerde `[x,y,value]` dizisinin ilk iki sayısal boyutu kullanılır.
    pub matris_koordinatları: Vec<Option<(MatrisAralığı, MatrisAralığı)>>,
}

impl Default for IsıHaritasıSerisi {
    fn default() -> Self {
        IsıHaritasıSerisi {
            ad: None,
            veri: Vec::new(),
            öğe_stili: ÖğeStili::default(),
            vurgu_öğe_stili: ÖğeStili::default(),
            hücre_boşluğu: 1.0,
            etiket: Etiket::default(),
            eksen_bağı: EksenBağı::default(),
            matris_sırası: None,
            matris_koordinatları: Vec::new(),
        }
    }
}

impl IsıHaritasıSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Seriyi verilen x/y eksen sıralarına bağlar (`xAxisIndex`/`yAxisIndex`).
    pub fn eksenler(mut self, x: usize, y: usize) -> Self {
        self.eksen_bağı = EksenBağı { x, y };
        self.matris_sırası = None;
        self
    }

    /// Seriyi Matrix koordinatına bağlar (`coordinateSystem: 'matrix'`).
    pub fn matris_sırası(mut self, sıra: usize) -> Self {
        self.matris_sırası = Some(sıra);
        self
    }

    pub fn matris_koordinatları<X, Y>(
        mut self,
        koordinatlar: impl IntoIterator<Item = Option<(X, Y)>>,
    ) -> Self
    where
        X: Into<MatrisAralığı>,
        Y: Into<MatrisAralığı>,
    {
        self.matris_koordinatları = koordinatlar
            .into_iter()
            .map(|koordinat| koordinat.map(|(x, y)| (x.into(), y.into())))
            .collect();
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    /// Veri: `[x sırası, y sırası, değer]` dizileri.
    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn vurgu_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.vurgu_öğe_stili = stil;
        self
    }

    pub fn hücre_boşluğu(mut self, boşluk: f32) -> Self {
        self.hücre_boşluğu = boşluk;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }
}

/// ECharts büyük saçılım hattının kullandığı iç içe olmayan
/// `Float32Array` veri deposu. Değerler `[x0, y0, x1, y1, ...]` sırasındadır;
/// tamamlanmamış son değer güvenle yok sayılır.
#[derive(Clone, Debug)]
pub struct DüzSaçılımVerisi {
    değerler: Arc<[f32]>,
}

impl DüzSaçılımVerisi {
    pub fn yeni(değerler: impl Into<Arc<[f32]>>) -> Self {
        Self {
            değerler: değerler.into(),
        }
    }

    /// XY noktası sayısı.
    pub fn len(&self) -> usize {
        self.değerler.len() / 2
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Ham, iç içe olmayan `f32` dilimi.
    pub fn değerler(&self) -> &[f32] {
        &self.değerler
    }

    /// Özgün veri sırasındaki XY çiftini döndürür.
    pub fn xy(&self, sıra: usize) -> Option<(f64, f64)> {
        let başlangıç = sıra.checked_mul(2)?;
        let x = *self.değerler.get(başlangıç)?;
        let y = *self.değerler.get(başlangıç + 1)?;
        Some((f64::from(x), f64::from(y)))
    }
}

/// Saçılım serisi (`series-scatter`).
#[derive(Clone, Debug)]
pub struct SaçılımSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    /// ECharts'ın `Float32Array` kabul eden büyük-veri yolu. `Some` iken
    /// `veri` boş tutulur ve XY çiftleri bu depodan okunur.
    pub düz_veri: Option<DüzSaçılımVerisi>,
    pub sembol: Sembol,
    pub sembol_boyutu: SembolBoyutu,
    /// Özel SVG yolunun en/boy oranını korur (`symbolKeepAspect`).
    pub sembol_oranını_koru: bool,
    pub öğe_stili: ÖğeStili,
    /// `emphasis.itemStyle`; vurgulanan sembolde yalnız açık alanlar normal
    /// stilin üstüne uygulanır.
    pub vurgu_öğe_stili: ÖğeStili,
    /// Seri düzeyindeki işlevsel `itemStyle.color`. Veri öğesinin açık
    /// `itemStyle.color` değeri bu callback'in önüne geçer.
    pub öğe_rengi_işlevi: Option<ÖğeRengiİşlevi>,
    pub etiket: Etiket,
    /// Sembol ile `labelLayout` tarafından taşınan etiket arasındaki kılavuz
    /// çizgisi (`labelLine`). `None`, scatter öntanımlısı olan gizli durumdur.
    pub etiket_çizgisi: Option<EtiketÇizgisi>,
    /// Etiketi tuval piksel uzayında taşıyan ve örtüşme politikasını seçen
    /// `labelLayout` geri çağrısı.
    pub etiket_yerleşimi: Option<EtiketYerleşimİşlevi>,
    /// Dataset `encode.label` için kullanılan boyut adı.
    pub etiket_boyutu: Option<String>,
    pub imleyiciler: İmleyiciler,
    /// Dalga efekti (`effectScatter` karşılığı): `efektli(true)` ile açılır.
    pub efektli: bool,
    /// Dalganın ulaştığı en büyük ölçek (`rippleEffect.scale`, öntanımlı 2.5).
    pub efekt_ölçeği: f32,
    /// Bir dalga turunun süresi, saniye (`rippleEffect.period`, öntanımlı 4).
    pub efekt_süresi_sn: f32,
    /// Dalga yalnız vuruş olarak çizilir (`rippleEffect.brushType: 'stroke'`).
    pub efekt_vuruşlu: bool,
    /// Bağlı eksenler (`xAxisIndex`/`yAxisIndex`).
    pub eksen_bağı: EksenBağı,
    /// Kutupsal koordinatta çizilir (`coordinateSystem: 'polar'`).
    pub kutupsal: bool,
    /// Bağlı kutupsal koordinat sırası (`polarIndex`).
    pub kutupsal_sırası: usize,
    /// Takvim koordinatına bağlıysa `calendarIndex`; `None`, kartezyen ya da
    /// başka bir koordinat sistemindeki saçılım demektir.
    pub takvim_sırası: Option<usize>,
    /// Matrix koordinatına bağlıysa `matrixIndex`.
    pub matris_sırası: Option<usize>,
    /// Veri sırasıyla eşleşen açık Matrix hücre/aralık koordinatları.
    pub matris_koordinatları: Vec<Option<(MatrisAralığı, MatrisAralığı)>>,
    /// Tek eksenli koordinata bağlıysa `singleAxisIndex`.
    pub tek_eksen_sırası: Option<usize>,
    /// ZRender tuval katmanı (`zlevel`). Takvim bileşeninde pozitif değer,
    /// seriyi ay/etiket üst katmanının da üzerine taşır.
    pub z_seviyesi: i32,
    /// İşaretçi/isabet olaylarını kapatır (`silent`).
    pub sessiz: bool,
    /// Paleti seri yerine her veri öğesinde ilerletir (`colorBy: 'data'`).
    pub veriye_göre_renk: bool,
    /// Veri kümesi eşlemesi: `(x boyutu, y boyutu)` (`encode.x/y`).
    pub eşleme: Option<(String, String)>,
    /// Bağlı `dataset` dizisi sırası (`datasetIndex`).
    pub veri_kümesi_sırası: usize,
    /// `seriesLayoutBy`.
    pub seri_yerleşimi: SeriYerleşimi,
    /// Büyük sembol çizimini açar (`large`).
    pub büyük: bool,
    /// Büyük çizim yoluna geçilecek en küçük veri sayısı
    /// (`largeThreshold`, ECharts öntanımlısı 2000).
    pub büyük_eşiği: usize,
    /// Bir artımlı çizim parçasındaki nokta sayısı (`progressive`, ECharts
    /// scatter öntanımlısı 5000).
    pub aşamalı: usize,
    /// Artımlı işleme eşiği (`progressiveThreshold`, ECharts öntanımlısı
    /// 10000).
    pub aşamalı_eşiği: usize,
    /// Seri animasyonunun kapatılacağı veri sayısı (`animationThreshold`,
    /// ECharts öntanımlısı 2000). Eşiğin tam üstünde animasyon devre dışıdır.
    pub animasyon_eşiği: usize,
}

impl Default for SaçılımSerisi {
    fn default() -> Self {
        SaçılımSerisi {
            ad: None,
            veri: Vec::new(),
            düz_veri: None,
            sembol: Sembol::Daire,
            sembol_boyutu: SembolBoyutu::Sabit(10.0),
            sembol_oranını_koru: false,
            öğe_stili: ÖğeStili::default(),
            vurgu_öğe_stili: ÖğeStili::default(),
            öğe_rengi_işlevi: None,
            etiket: Etiket::default(),
            etiket_çizgisi: None,
            etiket_yerleşimi: None,
            etiket_boyutu: None,
            imleyiciler: İmleyiciler::default(),
            efektli: false,
            efekt_ölçeği: 2.5,
            efekt_süresi_sn: 4.0,
            efekt_vuruşlu: false,
            eksen_bağı: EksenBağı::default(),
            kutupsal: false,
            kutupsal_sırası: 0,
            takvim_sırası: None,
            matris_sırası: None,
            matris_koordinatları: Vec::new(),
            tek_eksen_sırası: None,
            z_seviyesi: 0,
            sessiz: false,
            veriye_göre_renk: false,
            eşleme: None,
            veri_kümesi_sırası: 0,
            seri_yerleşimi: SeriYerleşimi::Sütun,
            büyük: false,
            büyük_eşiği: 2_000,
            aşamalı: 5_000,
            aşamalı_eşiği: 10_000,
            animasyon_eşiği: 2_000,
        }
    }
}

impl SaçılımSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Seriyi veri kümesine bağlar: `(x boyutu, y boyutu)` (`encode.x/y`).
    ///
    /// X boyutu sayısal bir eksene sayı, kategori eksenine sırasal değer
    /// olarak akar. Dataset'in diğer boyutları tooltip/visualMap için veri
    /// öğesinde korunur.
    pub fn eşle(mut self, x_boyutu: impl Into<String>, y_boyutu: impl Into<String>) -> Self {
        self.eşleme = Some((x_boyutu.into(), y_boyutu.into()));
        self
    }

    pub fn veri_kümesi_sırası(mut self, sıra: usize) -> Self {
        self.veri_kümesi_sırası = sıra;
        self
    }

    pub fn seri_yerleşimi(mut self, yerleşim: SeriYerleşimi) -> Self {
        self.seri_yerleşimi = yerleşim;
        self
    }

    /// Seriyi kutupsal koordinata bağlar (`coordinateSystem: 'polar'`).
    pub fn kutupsal(mut self, açık: bool) -> Self {
        self.kutupsal = açık;
        if açık {
            self.takvim_sırası = None;
            self.tek_eksen_sırası = None;
            self.matris_sırası = None;
        }
        self
    }

    /// Seriyi belirtilen `polarIndex` bileşenine bağlar.
    pub fn kutupsal_sırası(mut self, sıra: usize) -> Self {
        self.kutupsal = true;
        self.kutupsal_sırası = sıra;
        self.takvim_sırası = None;
        self.tek_eksen_sırası = None;
        self.matris_sırası = None;
        self
    }

    /// Seriyi takvim koordinatına bağlar (`coordinateSystem: 'calendar'`,
    /// `calendarIndex`).
    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = Some(sıra);
        self.tek_eksen_sırası = None;
        self.matris_sırası = None;
        self.kutupsal = false;
        self
    }

    /// Seriyi tek eksenli koordinata bağlar (`coordinateSystem:
    /// 'singleAxis'`, `singleAxisIndex`).
    pub fn tek_eksen_sırası(mut self, sıra: usize) -> Self {
        self.tek_eksen_sırası = Some(sıra);
        self.takvim_sırası = None;
        self.matris_sırası = None;
        self.kutupsal = false;
        self
    }

    /// Seriyi Matrix koordinatına bağlar (`coordinateSystem: 'matrix'`).
    pub fn matris_sırası(mut self, sıra: usize) -> Self {
        self.matris_sırası = Some(sıra);
        self.takvim_sırası = None;
        self.tek_eksen_sırası = None;
        self.kutupsal = false;
        self
    }

    pub fn matris_koordinatları<X, Y>(
        mut self,
        koordinatlar: impl IntoIterator<Item = Option<(X, Y)>>,
    ) -> Self
    where
        X: Into<MatrisAralığı>,
        Y: Into<MatrisAralığı>,
    {
        self.matris_koordinatları = koordinatlar
            .into_iter()
            .map(|koordinat| koordinat.map(|(x, y)| (x.into(), y.into())))
            .collect();
        self
    }

    pub fn z_seviyesi(mut self, seviye: i32) -> Self {
        self.z_seviyesi = seviye;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }

    pub fn veriye_göre_renklendir(mut self, açık: bool) -> Self {
        self.veriye_göre_renk = açık;
        self
    }

    /// Seriyi verilen x/y eksen sıralarına bağlar (`xAxisIndex`/`yAxisIndex`).
    pub fn eksenler(mut self, x: usize, y: usize) -> Self {
        self.eksen_bağı = EksenBağı { x, y };
        self.takvim_sırası = None;
        self.tek_eksen_sırası = None;
        self.matris_sırası = None;
        self.kutupsal = false;
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self.düz_veri = None;
        self
    }

    /// İç içe olmayan `Float32Array` eşdeğerini kopyalamadan bağlar.
    ///
    /// Girdi `[x0, y0, x1, y1, ...]` düzenindedir. Bu yol, milyonlarca
    /// nokta için her satırda [`VeriÖğesi`] ayırma maliyetini önler.
    pub fn düz_veri(mut self, veri: impl Into<Arc<[f32]>>) -> Self {
        self.düz_veri = Some(DüzSaçılımVerisi::yeni(veri));
        self.veri.clear();
        self
    }

    /// Serideki gerçek veri öğesi sayısı.
    pub fn veri_sayısı(&self) -> usize {
        self.düz_veri
            .as_ref()
            .map(DüzSaçılımVerisi::len)
            .unwrap_or(self.veri.len())
    }

    /// Veri sırasındaki sayısal XY çiftini döndürür. Dataset `encode`
    /// kullanan nesne verisinde eşleme eksen bağlamıyla çözüldüğünden burada
    /// yalnız doğal ilk iki boyut ele alınır.
    pub fn xy(&self, sıra: usize) -> Option<(f64, f64)> {
        match &self.düz_veri {
            Some(veri) => veri.xy(sıra),
            None => self
                .veri
                .get(sıra)
                .and_then(|öğe| crate::grafik::sacilim::saçılım_xy(&öğe.değer, sıra)),
        }
    }

    pub fn büyük(mut self, açık: bool) -> Self {
        self.büyük = açık;
        self
    }

    pub fn büyük_eşiği(mut self, eşik: usize) -> Self {
        self.büyük_eşiği = eşik;
        self
    }

    pub fn aşamalı(mut self, parça_boyutu: usize) -> Self {
        self.aşamalı = parça_boyutu.max(1);
        self
    }

    pub fn aşamalı_eşiği(mut self, eşik: usize) -> Self {
        self.aşamalı_eşiği = eşik;
        self
    }

    pub fn animasyon_eşiği(mut self, eşik: usize) -> Self {
        self.animasyon_eşiği = eşik;
        self
    }

    /// ECharts `SeriesModel::isAnimationEnabled` veri sayısı kapısı.
    pub fn animasyon_eşiğinde_mi(&self) -> bool {
        self.veri_sayısı() <= self.animasyon_eşiği
    }

    /// ECharts `pipelineContext.large` koşulunun seri düzeyindeki karşılığı.
    pub fn büyük_etkin_mi(&self) -> bool {
        self.büyük && self.veri_sayısı() >= self.büyük_eşiği
    }

    pub(crate) fn veri_boş_mu(&self) -> bool {
        self.veri.is_empty()
            && self
                .düz_veri
                .as_ref()
                .is_none_or(DüzSaçılımVerisi::is_empty)
    }

    pub(crate) fn düz_değerler(&self) -> Option<&[f32]> {
        self.düz_veri.as_ref().map(DüzSaçılımVerisi::değerler)
    }

    pub(crate) fn düz_boyut_çöz(&self, sıra: usize, x: f64, y: f64) -> f32 {
        match &self.sembol_boyutu {
            SembolBoyutu::Sabit(boyut) => *boyut,
            boyut => boyut.bağlamla_çöz(&VeriÖğesi::yeni([x, y]), sıra),
        }
    }

    pub(crate) fn düz_xy_iter(&self) -> impl Iterator<Item = (usize, f64, f64)> + '_ {
        self.düz_değerler()
            .into_iter()
            .flat_map(|değerler| değerler.chunks_exact(2))
            .enumerate()
            .map(|(sıra, çift)| (sıra, f64::from(çift[0]), f64::from(çift[1])))
    }

    pub fn sembol(mut self, sembol: Sembol) -> Self {
        self.sembol = sembol;
        self
    }

    pub fn sembol_boyutu(mut self, boyut: impl Into<SembolBoyutu>) -> Self {
        self.sembol_boyutu = boyut.into();
        self
    }

    /// Özel SVG sembolünü kutusunda ortalayıp en/boy oranını korur
    /// (`symbolKeepAspect`).
    pub fn sembol_oranını_koru(mut self, koru: bool) -> Self {
        self.sembol_oranını_koru = koru;
        self
    }

    pub fn vurgu_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.vurgu_öğe_stili = stil;
        self
    }

    /// Dalga efektini açar (ECharts `effectScatter` serisinin karşılığı).
    pub fn efektli(mut self, açık: bool) -> Self {
        self.efektli = açık;
        self
    }

    pub fn efekt_ölçeği(mut self, ölçek: f32) -> Self {
        self.efekt_ölçeği = ölçek.max(1.0);
        self
    }

    pub fn efekt_süresi_sn(mut self, saniye: f32) -> Self {
        self.efekt_süresi_sn = saniye.max(0.1);
        self
    }

    pub fn efekt_vuruşlu(mut self, vuruşlu: bool) -> Self {
        self.efekt_vuruşlu = vuruşlu;
        self
    }

    /// Veriye bağlı sembol boyutu.
    pub fn sembol_boyutu_işlevi(
        mut self,
        işlev: impl Fn(&VeriÖğesi) -> f32 + Send + Sync + 'static,
    ) -> Self {
        self.sembol_boyutu = SembolBoyutu::İşlev(Arc::new(işlev));
        self
    }

    /// `symbolSize(value, params)` biçimindeki callback. Bağlamın
    /// `veri_sırası` alanı ECharts `params.dataIndex` ile aynıdır.
    pub fn sembol_boyutu_bağlamlı_işlevi(
        mut self,
        işlev: impl Fn(&VeriÖğesi, VeriİşlevBağlamı) -> f32 + Send + Sync + 'static,
    ) -> Self {
        self.sembol_boyutu = SembolBoyutu::Bağlamlıİşlev(Arc::new(işlev));
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    /// `itemStyle.color(value, params)` biçimindeki callback. Dönen değer
    /// düz renk, gradyan veya görüntü deseni olabilir.
    pub fn öğe_rengi_işlevi<D>(
        mut self,
        işlev: impl Fn(&VeriÖğesi, VeriİşlevBağlamı) -> D + Send + Sync + 'static,
    ) -> Self
    where
        D: Into<Dolgu> + 'static,
    {
        self.öğe_rengi_işlevi = Some(ÖğeRengiİşlevi::yeni(move |öğe, bağlam| {
            işlev(öğe, bağlam).into()
        }));
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn etiket_çizgisi(mut self, çizgi: EtiketÇizgisi) -> Self {
        self.etiket_çizgisi = Some(çizgi);
        self
    }

    pub fn etiket_yerleşimi(
        mut self,
        işlev: impl Fn(&EtiketYerleşimParametreleri) -> EtiketYerleşimSonucu + Send + Sync + 'static,
    ) -> Self {
        self.etiket_yerleşimi = Some(EtiketYerleşimİşlevi::yeni(işlev));
        self
    }

    /// Dataset etiket metnini bir boyuta bağlar (`encode.label`).
    pub fn etiket_boyutunu_eşle(mut self, boyut: impl Into<String>) -> Self {
        self.etiket_boyutu = Some(boyut.into());
        self
    }

    pub fn im_çizgisi(mut self, im: İmÇizgisi) -> Self {
        self.imleyiciler.çizgi = Some(im);
        self
    }

    pub fn im_noktası(mut self, im: İmNoktası) -> Self {
        self.imleyiciler.nokta = Some(im);
        self
    }

    pub fn im_alanı(mut self, im: İmAlanı) -> Self {
        self.imleyiciler.alan = Some(im);
        self
    }
}

/// Huni sıralaması (`funnel.sort`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HuniSıralaması {
    #[default]
    Azalan,
    Artan,
    Yok,
}

/// Huni akış yönü (`funnel.orient`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HuniYönü {
    #[default]
    Dikey,
    Yatay,
}

/// Değer genişliği/yüksekliğinin görünüm kutusundaki hizası
/// (`funnel.funnelAlign`). Dikey hunide `Sol/Orta/Sağ`, yatay hunide
/// `Üst/Orta/Alt` kullanılır.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HuniHizası {
    Sol,
    #[default]
    Orta,
    Sağ,
    Üst,
    Alt,
}

/// Huni durumlarındaki `labelLine` yaması. `None` alanlar normal durumdan
/// miras alınır.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct HuniEtiketÇizgisiYaması {
    pub göster: Option<bool>,
    pub uzunluk: Option<f32>,
    pub stil: Option<ÇizgiStili>,
}

impl HuniEtiketÇizgisiYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = Some(göster);
        self
    }

    pub fn uzunluk(mut self, uzunluk: f32) -> Self {
        self.uzunluk = Some(uzunluk.max(0.0));
        self
    }

    pub fn stil(mut self, stil: ÇizgiStili) -> Self {
        self.stil = Some(stil);
        self
    }

    pub(crate) fn uygula(&self, taban: &EtiketÇizgisi) -> EtiketÇizgisi {
        let mut sonuç = taban.clone();
        if let Some(göster) = self.göster {
            sonuç.göster = göster;
        }
        if let Some(uzunluk) = self.uzunluk {
            sonuç.uzunluk1 = uzunluk;
        }
        if let Some(stil) = &self.stil {
            sonuç.stil = stil.clone();
        }
        sonuç
    }
}

/// Huni `emphasis` / `blur` / `select` durum yaması.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct HuniDurumYaması {
    pub öğe_stili: ÖğeStili,
    pub etiket: EtiketYaması,
    pub etiket_çizgisi: HuniEtiketÇizgisiYaması,
}

impl HuniDurumYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn etiket(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.etiket = etiket.into();
        self
    }

    pub fn etiket_çizgisi(mut self, çizgi: HuniEtiketÇizgisiYaması) -> Self {
        self.etiket_çizgisi = çizgi;
        self
    }
}

/// Nesne biçimli huni verisine özgü seçenekler. Normal `itemStyle` ve
/// `label`, [`VeriÖğesi`] üzerinde kalır; burada FunnelDataItemOption'a özgü
/// `itemStyle.width/height`, `labelLine` ve durum yamaları tutulur.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct HuniVeriYaması {
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    pub etiket_çizgisi: HuniEtiketÇizgisiYaması,
    pub vurgu: HuniDurumYaması,
    pub bulanık: HuniDurumYaması,
    pub seçim: HuniDurumYaması,
}

impl HuniVeriYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(genişlik.into());
        self
    }

    pub fn yükseklik(mut self, yükseklik: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(yükseklik.into());
        self
    }

    pub fn etiket_çizgisi(mut self, çizgi: HuniEtiketÇizgisiYaması) -> Self {
        self.etiket_çizgisi = çizgi;
        self
    }

    pub fn vurgu(mut self, durum: HuniDurumYaması) -> Self {
        self.vurgu = durum;
        self
    }

    pub fn bulanık(mut self, durum: HuniDurumYaması) -> Self {
        self.bulanık = durum;
        self
    }

    pub fn seçim(mut self, durum: HuniDurumYaması) -> Self {
        self.seçim = durum;
        self
    }
}

/// Huni serisi (`series-funnel`).
#[derive(Clone, Debug)]
pub struct HuniSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub sol: Uzunluk,
    pub sağ: Uzunluk,
    pub üst: Uzunluk,
    pub alt: Uzunluk,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    /// Açık değilse veri kapsamından türetilir (`min` / `max`).
    pub en_az: Option<f64>,
    pub en_çok: Option<f64>,
    pub sıralama: HuniSıralaması,
    pub yön: HuniYönü,
    pub hiza: HuniHizası,
    /// Dilimler arası dikey boşluk (`gap`).
    pub dilim_boşluğu: f32,
    /// En dar dilimin genişliği (`minSize`).
    pub en_az_genişlik: Uzunluk,
    /// En geniş dilimin genişliği (`maxSize`).
    pub en_çok_genişlik: Uzunluk,
    /// ZRender çizim sırası (`z`), ECharts öntanımlısı 2.
    pub z: i32,
    /// İşaretçi/isabet olaylarını kapatır (`silent`).
    pub sessiz: bool,
    pub öğe_stili: ÖğeStili,
    pub etiket: Etiket,
    pub etiket_çizgisi: EtiketÇizgisi,
    pub vurgu: HuniDurumYaması,
    pub bulanık: HuniDurumYaması,
    pub seçim: HuniDurumYaması,
    pub öğe_yamaları: Vec<Option<HuniVeriYaması>>,
    /// Veri kümesi eşlemesi: `(ad boyutu, değer boyutu)` (`encode`).
    pub eşleme: Option<(String, String)>,
    pub veri_kümesi_sırası: usize,
    pub seri_yerleşimi: SeriYerleşimi,
}

impl Default for HuniSerisi {
    fn default() -> Self {
        HuniSerisi {
            ad: None,
            veri: Vec::new(),
            sol: Uzunluk::Piksel(80.0),
            sağ: Uzunluk::Piksel(80.0),
            üst: Uzunluk::Piksel(60.0),
            alt: Uzunluk::Piksel(65.0),
            genişlik: None,
            yükseklik: None,
            en_az: None,
            en_çok: None,
            sıralama: HuniSıralaması::Azalan,
            yön: HuniYönü::Dikey,
            hiza: HuniHizası::Orta,
            dilim_boşluğu: 0.0,
            en_az_genişlik: Uzunluk::Yüzde(0.0),
            en_çok_genişlik: Uzunluk::Yüzde(100.0),
            z: 2,
            sessiz: false,
            öğe_stili: ÖğeStili {
                kenarlık_rengi: Some(crate::renk::Renk::BEYAZ),
                kenarlık_kalınlığı: 1.0,
                ..Default::default()
            },
            etiket: Etiket {
                göster: true,
                konum: crate::model::stil::EtiketKonumu::Dış,
                ..Default::default()
            },
            etiket_çizgisi: EtiketÇizgisi {
                uzunluk1: 20.0,
                uzunluk2: 0.0,
                ..Default::default()
            },
            vurgu: HuniDurumYaması {
                etiket: EtiketYaması::yeni().göster(true),
                ..Default::default()
            },
            bulanık: HuniDurumYaması::default(),
            seçim: HuniDurumYaması::default(),
            öğe_yamaları: Vec::new(),
            eşleme: None,
            veri_kümesi_sırası: 0,
            seri_yerleşimi: SeriYerleşimi::Sütun,
        }
    }
}

impl HuniSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self.öğe_yamaları.resize(self.veri.len(), None);
        self
    }

    pub fn sol(mut self, sol: impl Into<Uzunluk>) -> Self {
        self.sol = sol.into();
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = sağ.into();
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = üst.into();
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = alt.into();
        self
    }

    pub fn genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(genişlik.into());
        self
    }

    pub fn yükseklik(mut self, yükseklik: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(yükseklik.into());
        self
    }

    pub fn değer_aralığı(mut self, en_az: f64, en_çok: f64) -> Self {
        self.en_az = Some(en_az);
        self.en_çok = Some(en_çok);
        self
    }

    pub fn otomatik_değer_aralığı(mut self) -> Self {
        self.en_az = None;
        self.en_çok = None;
        self
    }

    pub fn sıralama(mut self, sıralama: HuniSıralaması) -> Self {
        self.sıralama = sıralama;
        self
    }

    pub fn yön(mut self, yön: HuniYönü) -> Self {
        self.yön = yön;
        self
    }

    pub fn hiza(mut self, hiza: HuniHizası) -> Self {
        self.hiza = hiza;
        self
    }

    pub fn dilim_boşluğu(mut self, boşluk: f32) -> Self {
        self.dilim_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn boyut_aralığı(
        mut self,
        en_az: impl Into<Uzunluk>,
        en_çok: impl Into<Uzunluk>,
    ) -> Self {
        self.en_az_genişlik = en_az.into();
        self.en_çok_genişlik = en_çok.into();
        self
    }

    pub fn z(mut self, z: i32) -> Self {
        self.z = z;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn etiket_çizgisi(mut self, çizgi: EtiketÇizgisi) -> Self {
        self.etiket_çizgisi = çizgi;
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn vurgu(mut self, durum: HuniDurumYaması) -> Self {
        self.vurgu = durum;
        self
    }

    pub fn bulanık(mut self, durum: HuniDurumYaması) -> Self {
        self.bulanık = durum;
        self
    }

    pub fn seçim(mut self, durum: HuniDurumYaması) -> Self {
        self.seçim = durum;
        self
    }

    pub fn öğe_yaması(mut self, sıra: usize, yama: HuniVeriYaması) -> Self {
        if self.öğe_yamaları.len() <= sıra {
            self.öğe_yamaları.resize(sıra + 1, None);
        }
        self.öğe_yamaları[sıra] = Some(yama);
        self
    }

    pub fn eşle(mut self, ad_boyutu: impl Into<String>, değer_boyutu: impl Into<String>) -> Self {
        self.eşleme = Some((ad_boyutu.into(), değer_boyutu.into()));
        self
    }

    pub fn veri_kümesi_sırası(mut self, sıra: usize) -> Self {
        self.veri_kümesi_sırası = sıra;
        self
    }

    pub fn seri_yerleşimi(mut self, yerleşim: SeriYerleşimi) -> Self {
        self.seri_yerleşimi = yerleşim;
        self
    }
}

/// Gauge veri öğesindeki `title` / `detail` yaması. `None` alanlar seri
/// düzeyindeki seçeneği miras alır; bu, ECharts `data[i].title/detail`
/// model zincirinin kayıpsız karşılığıdır.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct GöstergeMetinYaması {
    pub göster: Option<bool>,
    pub merkez_kayması: Option<(Uzunluk, Uzunluk)>,
    pub biçimleyici: Option<Biçimleyici>,
    pub değer_animasyonu: Option<bool>,
    /// `detail.precision`; yalnız değer animasyonunun ara karelerinde
    /// interpolasyon değerini yuvarlar.
    pub duyarlılık: Option<usize>,
    pub stil: YazıStili,
    pub zengin: std::collections::BTreeMap<String, YazıStili>,
    pub renk_miras: Option<bool>,
    pub arkaplan_miras: Option<bool>,
    pub kenarlık_miras: Option<bool>,
}

impl GöstergeMetinYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = Some(göster);
        self
    }

    pub fn merkez_kayması<X: Into<Uzunluk>, Y: Into<Uzunluk>>(mut self, x: X, y: Y) -> Self {
        self.merkez_kayması = Some((x.into(), y.into()));
        self
    }

    pub fn biçimleyici(mut self, biçimleyici: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(biçimleyici.into());
        self
    }

    pub fn değer_animasyonu(mut self, açık: bool) -> Self {
        self.değer_animasyonu = Some(açık);
        self
    }

    pub fn duyarlılık(mut self, basamak: usize) -> Self {
        self.duyarlılık = Some(basamak);
        self
    }

    pub fn stil(mut self, stil: YazıStili) -> Self {
        self.stil = stil;
        self
    }

    pub fn zengin_stil(mut self, ad: impl Into<String>, stil: YazıStili) -> Self {
        self.zengin.insert(ad.into(), stil);
        self
    }

    pub fn renk_miras(mut self, miras: bool) -> Self {
        self.renk_miras = Some(miras);
        self
    }

    pub fn arkaplan_miras(mut self, miras: bool) -> Self {
        self.arkaplan_miras = Some(miras);
        self
    }

    pub fn kenarlık_miras(mut self, miras: bool) -> Self {
        self.kenarlık_miras = Some(miras);
        self
    }
}

/// Gauge veri öğesindeki `pointer` yaması.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct GöstergeİbreYaması {
    pub göster: Option<bool>,
    pub simge: Option<Sembol>,
    pub merkez_kayması: Option<(Uzunluk, Uzunluk)>,
    pub uzunluk: Option<Uzunluk>,
    pub genişlik: Option<f32>,
    pub oranı_koru: Option<bool>,
    pub stil: Option<ÖğeStili>,
    pub renk_otomatik: Option<bool>,
}

impl GöstergeİbreYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = Some(göster);
        self
    }

    pub fn simge(mut self, simge: Sembol) -> Self {
        self.simge = Some(simge);
        self
    }

    pub fn merkez_kayması<X: Into<Uzunluk>, Y: Into<Uzunluk>>(mut self, x: X, y: Y) -> Self {
        self.merkez_kayması = Some((x.into(), y.into()));
        self
    }

    pub fn uzunluk(mut self, uzunluk: impl Into<Uzunluk>) -> Self {
        self.uzunluk = Some(uzunluk.into());
        self
    }

    pub fn genişlik(mut self, genişlik: f32) -> Self {
        self.genişlik = Some(genişlik.max(0.0));
        self
    }

    pub fn oranı_koru(mut self, koru: bool) -> Self {
        self.oranı_koru = Some(koru);
        self
    }

    pub fn stil(mut self, stil: ÖğeStili) -> Self {
        self.stil = Some(stil);
        self
    }

    pub fn renk_otomatik(mut self, otomatik: bool) -> Self {
        self.renk_otomatik = Some(otomatik);
        self
    }
}

/// Gauge veri öğesindeki `progress` yaması.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct GöstergeİlerlemeYaması {
    pub göster: Option<bool>,
    pub stil: Option<ÖğeStili>,
    pub renk_otomatik: Option<bool>,
}

impl GöstergeİlerlemeYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = Some(göster);
        self
    }

    pub fn stil(mut self, stil: ÖğeStili) -> Self {
        self.stil = Some(stil);
        self
    }

    pub fn renk_otomatik(mut self, otomatik: bool) -> Self {
        self.renk_otomatik = Some(otomatik);
        self
    }
}

/// Nesne biçimli gauge verisi (`data[i]`). Genel değer/ad/itemStyle
/// alanlarının yanında gauge'a özgü alt seçenekleri de taşır.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct GöstergeVeriÖğesi {
    pub öğe: VeriÖğesi,
    pub ibre: GöstergeİbreYaması,
    pub ilerleme: GöstergeİlerlemeYaması,
    pub başlık: GöstergeMetinYaması,
    pub ayrıntı: GöstergeMetinYaması,
}

impl GöstergeVeriÖğesi {
    pub fn yeni(değer: impl Into<crate::model::deger::VeriDeğeri>) -> Self {
        Self {
            öğe: VeriÖğesi::yeni(değer),
            ..Self::default()
        }
    }

    pub fn adlı(
        ad: impl Into<String>, değer: impl Into<crate::model::deger::VeriDeğeri>
    ) -> Self {
        Self {
            öğe: VeriÖğesi::adlı(ad, değer),
            ..Self::default()
        }
    }

    pub fn stil(mut self, stil: ÖğeStili) -> Self {
        self.öğe.stil = Some(stil);
        self
    }

    pub fn ibre(mut self, yama: GöstergeİbreYaması) -> Self {
        self.ibre = yama;
        self
    }

    pub fn ilerleme(mut self, yama: GöstergeİlerlemeYaması) -> Self {
        self.ilerleme = yama;
        self
    }

    pub fn başlık(mut self, yama: GöstergeMetinYaması) -> Self {
        self.başlık = yama;
        self
    }

    pub fn ayrıntı(mut self, yama: GöstergeMetinYaması) -> Self {
        self.ayrıntı = yama;
        self
    }
}

/// Gösterge saati serisi (`series-gauge`). Birden çok veri öğesinin ibre,
/// progress, title ve detail seçeneklerini ECharts gibi ayrı ayrı işler.
#[derive(Clone, Debug)]
pub struct GöstergeSaatiSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    /// `veri` ile aynı sıradaki gauge'a özgü data-item yamaları.
    pub veri_ayarları: Vec<GöstergeVeriÖğesi>,
    /// Seri/veri görsel stili (`itemStyle`); pointer ve progress bunu
    /// miras alır.
    pub öğe_stili: ÖğeStili,
    pub en_az: f64,
    pub en_çok: f64,
    /// Başlangıç açısı, derece (`startAngle`, öntanımlı 225).
    pub başlangıç_açısı: f32,
    /// Bitiş açısı, derece (`endAngle`, öntanımlı -45).
    pub bitiş_açısı: f32,
    /// `clockwise`.
    pub saat_yönünde: bool,
    pub merkez: (Uzunluk, Uzunluk),
    pub yarıçap: Uzunluk,
    /// Renk bantları: `(bant sonu oranı 0..=1, renk)` —
    /// `axisLine.lineStyle.color`. Boş dizi ECharts 6.1'in tema-bağımlı
    /// `[[1, tokens.color.neutral10]]` öntanımlısını kullanır.
    pub renk_bantları: Vec<(f32, crate::renk::Renk)>,
    /// Gösterge yayını çiz (`axisLine.show`).
    pub şeridi_göster: bool,
    /// Yay şeridinin kalınlığı (`axisLine.lineStyle.width`).
    pub şerit_kalınlığı: f32,
    /// `axisLine.roundCap`.
    pub şerit_yuvarlak_uç: bool,
    /// `axisLine.lineStyle` içindeki opaklık/gölge/kenarlık yetenekleri.
    pub şerit_stili: ÇizgiStili,
    /// Değere kadar uzanan ön yay (`progress.show`).
    pub ilerlemeyi_göster: bool,
    /// `progress.width`.
    pub ilerleme_kalınlığı: f32,
    /// `progress.itemStyle.color`; verilmezse seri palet rengi kullanılır.
    pub ilerleme_rengi: Option<crate::renk::Renk>,
    /// `progress.roundCap`.
    pub ilerleme_yuvarlak_uç: bool,
    /// `progress.overlap` ve `progress.clip`.
    pub ilerleme_örtüşmesi: bool,
    pub ilerleme_kırp: bool,
    pub ilerleme_stili: ÖğeStili,
    pub ilerleme_rengi_otomatik: bool,
    pub bölme_sayısı: usize,
    /// Ana bölme çizgileri (`splitLine.show`).
    pub çentikleri_göster: bool,
    /// `splitLine.length`.
    pub çentik_uzunluğu: f32,
    /// `splitLine.distance`; yay şeridinin iç kenarından olan ek uzaklık.
    pub çentik_mesafesi: f32,
    /// `splitLine.lineStyle.width`.
    pub çentik_kalınlığı: f32,
    pub çentik_rengi: Option<crate::renk::Renk>,
    pub çentik_rengi_otomatik: bool,
    pub çentik_stili: ÇizgiStili,
    /// Ara eksen çentikleri (`axisTick`).
    pub ara_çentikleri_göster: bool,
    pub ara_çentik_sayısı: usize,
    pub ara_çentik_uzunluğu: Uzunluk,
    pub ara_çentik_mesafesi: f32,
    pub ara_çentik_kalınlığı: f32,
    pub ara_çentik_rengi: Option<crate::renk::Renk>,
    pub ara_çentik_rengi_otomatik: bool,
    pub ara_çentik_stili: ÇizgiStili,
    pub etiketleri_göster: bool,
    pub etiket_mesafesi: f32,
    pub etiket_boyutu: f32,
    pub etiket_rengi: Option<crate::renk::Renk>,
    pub etiket_rengi_miras: bool,
    pub etiket_biçimleyici: Option<crate::model::stil::Biçimleyici>,
    pub etiket_döndürme: EtiketDöndürme,
    pub etiket_stili: YazıStili,
    /// İbre (`pointer.show`).
    pub ibreyi_göster: bool,
    /// İbre uzunluğu, yarıçapa oranla ya da piksel (`pointer.length`).
    pub ibre_uzunluğu: Uzunluk,
    pub ibre_genişliği: f32,
    pub ibre_rengi: Option<crate::renk::Renk>,
    /// `pointer.icon`; `None`, ECharts'ın yerleşik PointerPath şeklidir.
    pub ibre_simgesi: Option<Sembol>,
    /// `pointer.offsetCenter`, gauge yarıçapına göre çözülür.
    pub ibre_merkez_kayması: (Uzunluk, Uzunluk),
    /// `pointer.keepAspect`.
    pub ibre_oranı_koru: bool,
    pub ibre_üstte: bool,
    pub ibre_stili: ÖğeStili,
    pub ibre_rengi_otomatik: bool,
    /// Merkez dayanağı (`anchor`).
    pub dayanağı_göster: bool,
    pub dayanak_üstte: bool,
    pub dayanak_boyutu: f32,
    pub dayanak_simgesi: Sembol,
    pub dayanak_merkez_kayması: (Uzunluk, Uzunluk),
    pub dayanak_oranı_koru: bool,
    pub dayanak_stili: ÖğeStili,
    /// Veri adı (`title.show`).
    pub adı_göster: bool,
    pub ad_merkez_kayması: (Uzunluk, Uzunluk),
    pub ad_boyutu: f32,
    pub ad_rengi: Option<crate::renk::Renk>,
    pub ad_rengi_miras: bool,
    pub ad_biçimleyici: Option<Biçimleyici>,
    pub ad_stili: YazıStili,
    /// Değer yazısı (`detail.show`).
    pub değeri_göster: bool,
    pub değer_merkez_kayması: (Uzunluk, Uzunluk),
    pub değer_boyutu: f32,
    pub değer_rengi: Option<crate::renk::Renk>,
    pub değer_rengi_miras: bool,
    pub değer_arkaplanı_miras: bool,
    pub değer_kenarlığı_miras: bool,
    pub değer_kalın: bool,
    /// `detail.precision`; ECharts gibi yalnız değer animasyonunun ara
    /// karelerindeki interpolasyon değerine uygulanır.
    pub değer_duyarlılığı: Option<usize>,
    /// `detail.valueAnimation`: detail metni de seri geçiş değerini izler.
    pub değer_animasyonu: bool,
    pub değer_biçimleyici: Option<crate::model::stil::Biçimleyici>,
    /// `detail` text-style yaması: arka plan, kenarlık, boyutlar, satır
    /// yüksekliği ve iç boşluk dâhil.
    pub değer_stili: crate::model::stil::YazıStili,
    /// `detail.rich` adlandırılmış metin koşuları.
    pub değer_zengin: std::collections::BTreeMap<String, crate::model::stil::YazıStili>,
}

impl Default for GöstergeSaatiSerisi {
    fn default() -> Self {
        GöstergeSaatiSerisi {
            ad: None,
            veri: Vec::new(),
            veri_ayarları: Vec::new(),
            öğe_stili: ÖğeStili::default(),
            en_az: 0.0,
            en_çok: 100.0,
            başlangıç_açısı: 225.0,
            bitiş_açısı: -45.0,
            saat_yönünde: true,
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(50.0)),
            yarıçap: Uzunluk::Yüzde(75.0),
            renk_bantları: Vec::new(),
            şeridi_göster: true,
            şerit_kalınlığı: 10.0,
            şerit_yuvarlak_uç: false,
            şerit_stili: ÇizgiStili {
                kalınlık: 10.0,
                ..ÇizgiStili::default()
            },
            ilerlemeyi_göster: false,
            ilerleme_kalınlığı: 10.0,
            ilerleme_rengi: None,
            ilerleme_yuvarlak_uç: false,
            ilerleme_örtüşmesi: true,
            ilerleme_kırp: true,
            ilerleme_stili: ÖğeStili::default(),
            ilerleme_rengi_otomatik: false,
            bölme_sayısı: 10,
            çentikleri_göster: true,
            çentik_uzunluğu: 10.0,
            çentik_mesafesi: 10.0,
            çentik_kalınlığı: 3.0,
            çentik_rengi: None,
            çentik_rengi_otomatik: false,
            çentik_stili: ÇizgiStili {
                kalınlık: 3.0,
                ..ÇizgiStili::default()
            },
            ara_çentikleri_göster: true,
            ara_çentik_sayısı: 5,
            ara_çentik_uzunluğu: Uzunluk::Piksel(6.0),
            ara_çentik_mesafesi: 10.0,
            ara_çentik_kalınlığı: 1.0,
            ara_çentik_rengi: None,
            ara_çentik_rengi_otomatik: false,
            ara_çentik_stili: ÇizgiStili {
                kalınlık: 1.0,
                ..ÇizgiStili::default()
            },
            etiketleri_göster: true,
            etiket_mesafesi: 15.0,
            etiket_boyutu: crate::tema::YAZI_KÜÇÜK,
            etiket_rengi: None,
            etiket_rengi_miras: false,
            etiket_biçimleyici: None,
            etiket_döndürme: EtiketDöndürme::Yok,
            etiket_stili: YazıStili::default(),
            ibreyi_göster: true,
            ibre_uzunluğu: Uzunluk::Yüzde(60.0),
            ibre_genişliği: 6.0,
            ibre_rengi: None,
            ibre_simgesi: None,
            ibre_merkez_kayması: (Uzunluk::Piksel(0.0), Uzunluk::Piksel(0.0)),
            ibre_oranı_koru: false,
            ibre_üstte: true,
            ibre_stili: ÖğeStili::default(),
            ibre_rengi_otomatik: false,
            dayanağı_göster: false,
            dayanak_üstte: false,
            dayanak_boyutu: 6.0,
            dayanak_simgesi: Sembol::Daire,
            dayanak_merkez_kayması: (Uzunluk::Piksel(0.0), Uzunluk::Piksel(0.0)),
            dayanak_oranı_koru: false,
            // `neutral00` ile ilk tema rengi çizim anında çözülür; böylece
            // aynı option nesnesi açık/koyu temada yeniden kullanılabilir.
            dayanak_stili: ÖğeStili::default(),
            adı_göster: true,
            ad_merkez_kayması: (Uzunluk::Piksel(0.0), Uzunluk::Yüzde(20.0)),
            ad_boyutu: crate::tema::YAZI_BÜYÜK,
            ad_rengi: None,
            ad_rengi_miras: false,
            ad_biçimleyici: None,
            ad_stili: YazıStili::default(),
            değeri_göster: true,
            değer_merkez_kayması: (Uzunluk::Piksel(0.0), Uzunluk::Yüzde(40.0)),
            değer_boyutu: 30.0,
            değer_rengi: None,
            değer_rengi_miras: false,
            değer_arkaplanı_miras: false,
            değer_kenarlığı_miras: false,
            değer_kalın: true,
            değer_duyarlılığı: None,
            değer_animasyonu: false,
            değer_biçimleyici: None,
            değer_stili: crate::model::stil::YazıStili::default(),
            değer_zengin: std::collections::BTreeMap::new(),
        }
    }
}

impl GöstergeSaatiSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    /// Tek değer + ad (`data: [{ value, name }]`).
    pub fn değer(mut self, değer: f64, ad: impl Into<String>) -> Self {
        self.veri = vec![VeriÖğesi::adlı(ad, değer)];
        self.veri_ayarları.clear();
        self
    }

    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self.veri_ayarları.clear();
        self
    }

    pub fn gösterge_verisi<T: Into<GöstergeVeriÖğesi>>(
        mut self,
        veri: impl IntoIterator<Item = T>,
    ) -> Self {
        self.veri_ayarları = veri.into_iter().map(Into::into).collect();
        self.veri = self
            .veri_ayarları
            .iter()
            .map(|ayar| ayar.öğe.clone())
            .collect();
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn aralık(mut self, en_az: f64, en_çok: f64) -> Self {
        self.en_az = en_az;
        self.en_çok = en_çok;
        self
    }

    pub fn açılar(mut self, başlangıç: f32, bitiş: f32) -> Self {
        self.başlangıç_açısı = başlangıç;
        self.bitiş_açısı = bitiş;
        self
    }

    pub fn saat_yönünde(mut self, saat_yönünde: bool) -> Self {
        self.saat_yönünde = saat_yönünde;
        self
    }

    pub fn merkez<X: Into<Uzunluk>, Y: Into<Uzunluk>>(mut self, x: X, y: Y) -> Self {
        self.merkez = (x.into(), y.into());
        self
    }

    pub fn yarıçap(mut self, yarıçap: impl Into<Uzunluk>) -> Self {
        self.yarıçap = yarıçap.into();
        self
    }

    pub fn renk_bantları<R: Into<crate::renk::Renk>>(
        mut self,
        bantlar: impl IntoIterator<Item = (f32, R)>,
    ) -> Self {
        self.renk_bantları = bantlar.into_iter().map(|(o, r)| (o, r.into())).collect();
        self
    }

    pub fn şerit(mut self, göster: bool, kalınlık: f32) -> Self {
        self.şeridi_göster = göster;
        self.şerit_kalınlığı = kalınlık.max(0.0);
        self.şerit_stili.kalınlık = self.şerit_kalınlığı;
        self
    }

    pub fn şerit_yuvarlak_uç(mut self, açık: bool) -> Self {
        self.şerit_yuvarlak_uç = açık;
        self
    }

    pub fn şerit_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.şerit_kalınlığı = stil.kalınlık.max(0.0);
        self.şerit_stili = stil;
        self
    }

    pub fn ilerleme(mut self, göster: bool, kalınlık: f32) -> Self {
        self.ilerlemeyi_göster = göster;
        self.ilerleme_kalınlığı = kalınlık.max(0.0);
        self
    }

    pub fn ilerleme_rengi(mut self, renk: impl Into<crate::renk::Renk>) -> Self {
        self.ilerleme_rengi = Some(renk.into());
        self.ilerleme_rengi_otomatik = false;
        self
    }

    pub fn ilerleme_yuvarlak_uç(mut self, açık: bool) -> Self {
        self.ilerleme_yuvarlak_uç = açık;
        self
    }

    pub fn ilerleme_örtüşmesi(mut self, örtüş: bool) -> Self {
        self.ilerleme_örtüşmesi = örtüş;
        self
    }

    pub fn ilerleme_kırp(mut self, kırp: bool) -> Self {
        self.ilerleme_kırp = kırp;
        self
    }

    pub fn ilerleme_stili(mut self, stil: ÖğeStili) -> Self {
        self.ilerleme_stili = stil;
        self
    }

    pub fn ilerleme_rengi_otomatik(mut self, otomatik: bool) -> Self {
        self.ilerleme_rengi_otomatik = otomatik;
        if otomatik {
            self.ilerleme_rengi = None;
        }
        self
    }

    pub fn bölme_sayısı(mut self, sayı: usize) -> Self {
        self.bölme_sayısı = sayı.max(1);
        self
    }

    pub fn ana_çentikler(
        mut self,
        göster: bool,
        uzunluk: f32,
        mesafe: f32,
        kalınlık: f32,
    ) -> Self {
        self.çentikleri_göster = göster;
        self.çentik_uzunluğu = uzunluk.max(0.0);
        self.çentik_mesafesi = mesafe;
        self.çentik_kalınlığı = kalınlık.max(0.0);
        self
    }

    pub fn ana_çentik_rengi(mut self, renk: impl Into<crate::renk::Renk>) -> Self {
        self.çentik_rengi = Some(renk.into());
        self.çentik_rengi_otomatik = false;
        self
    }

    pub fn ana_çentik_rengi_otomatik(mut self, otomatik: bool) -> Self {
        self.çentik_rengi_otomatik = otomatik;
        if otomatik {
            self.çentik_rengi = None;
        }
        self
    }

    pub fn ana_çentik_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çentik_uzunluğu = self.çentik_uzunluğu.max(0.0);
        self.çentik_kalınlığı = stil.kalınlık.max(0.0);
        self.çentik_rengi = stil.renk;
        self.çentik_stili = stil;
        self
    }

    pub fn ara_çentikler(
        mut self,
        göster: bool,
        sayı: usize,
        uzunluk: impl Into<Uzunluk>,
        mesafe: f32,
        kalınlık: f32,
    ) -> Self {
        self.ara_çentikleri_göster = göster;
        self.ara_çentik_sayısı = sayı.max(1);
        self.ara_çentik_uzunluğu = uzunluk.into();
        self.ara_çentik_mesafesi = mesafe;
        self.ara_çentik_kalınlığı = kalınlık.max(0.0);
        self
    }

    pub fn ara_çentik_rengi(mut self, renk: impl Into<crate::renk::Renk>) -> Self {
        self.ara_çentik_rengi = Some(renk.into());
        self.ara_çentik_rengi_otomatik = false;
        self
    }

    pub fn ara_çentik_rengi_otomatik(mut self, otomatik: bool) -> Self {
        self.ara_çentik_rengi_otomatik = otomatik;
        if otomatik {
            self.ara_çentik_rengi = None;
        }
        self
    }

    pub fn ara_çentik_stili(mut self, stil: ÇizgiStili) -> Self {
        self.ara_çentik_kalınlığı = stil.kalınlık.max(0.0);
        self.ara_çentik_rengi = stil.renk;
        self.ara_çentik_stili = stil;
        self
    }

    pub fn eksen_etiketleri(mut self, göster: bool, mesafe: f32, boyut: f32) -> Self {
        self.etiketleri_göster = göster;
        self.etiket_mesafesi = mesafe;
        self.etiket_boyutu = boyut.max(0.0);
        self
    }

    pub fn eksen_etiket_rengi(mut self, renk: impl Into<crate::renk::Renk>) -> Self {
        self.etiket_rengi = Some(renk.into());
        self.etiket_rengi_miras = false;
        self
    }

    pub fn eksen_etiket_rengi_miras(mut self, miras: bool) -> Self {
        self.etiket_rengi_miras = miras;
        if miras {
            self.etiket_rengi = None;
        }
        self
    }

    pub fn eksen_etiket_döndürme(mut self, döndürme: EtiketDöndürme) -> Self {
        self.etiket_döndürme = döndürme;
        self
    }

    pub fn eksen_etiket_stili(mut self, stil: YazıStili) -> Self {
        self.etiket_stili = stil;
        self
    }

    pub fn etiket_biçimleyici(
        mut self,
        biçimleyici: impl Into<crate::model::stil::Biçimleyici>,
    ) -> Self {
        self.etiket_biçimleyici = Some(biçimleyici.into());
        self
    }

    pub fn ibre(mut self, göster: bool, uzunluk: impl Into<Uzunluk>, genişlik: f32) -> Self {
        self.ibreyi_göster = göster;
        self.ibre_uzunluğu = uzunluk.into();
        self.ibre_genişliği = genişlik.max(0.0);
        self
    }

    pub fn ibre_simgesi(mut self, simge: Sembol) -> Self {
        self.ibre_simgesi = Some(simge);
        self
    }

    pub fn ibre_üstte(mut self, üstte: bool) -> Self {
        self.ibre_üstte = üstte;
        self
    }

    pub fn ibre_stili(mut self, stil: ÖğeStili) -> Self {
        self.ibre_stili = stil;
        self
    }

    pub fn ibre_rengi_otomatik(mut self, otomatik: bool) -> Self {
        self.ibre_rengi_otomatik = otomatik;
        if otomatik {
            self.ibre_rengi = None;
        }
        self
    }

    pub fn ibre_merkez_kayması<X: Into<Uzunluk>, Y: Into<Uzunluk>>(mut self, x: X, y: Y) -> Self {
        self.ibre_merkez_kayması = (x.into(), y.into());
        self
    }

    pub fn ibre_oranı_koru(mut self, koru: bool) -> Self {
        self.ibre_oranı_koru = koru;
        self
    }

    pub fn dayanak(mut self, göster: bool, boyut: f32) -> Self {
        self.dayanağı_göster = göster;
        self.dayanak_boyutu = boyut.max(0.0);
        self
    }

    pub fn dayanak_üstte(mut self, üstte: bool) -> Self {
        self.dayanak_üstte = üstte;
        self
    }

    pub fn dayanak_simgesi(mut self, simge: Sembol) -> Self {
        self.dayanak_simgesi = simge;
        self
    }

    pub fn dayanak_merkez_kayması<X: Into<Uzunluk>, Y: Into<Uzunluk>>(
        mut self,
        x: X,
        y: Y,
    ) -> Self {
        self.dayanak_merkez_kayması = (x.into(), y.into());
        self
    }

    pub fn dayanak_oranı_koru(mut self, koru: bool) -> Self {
        self.dayanak_oranı_koru = koru;
        self
    }

    pub fn dayanak_stili(mut self, stil: ÖğeStili) -> Self {
        self.dayanak_stili = stil;
        self
    }

    pub fn ad_göster(mut self, göster: bool) -> Self {
        self.adı_göster = göster;
        self
    }

    pub fn ad_merkez_kayması<X: Into<Uzunluk>, Y: Into<Uzunluk>>(mut self, x: X, y: Y) -> Self {
        self.ad_merkez_kayması = (x.into(), y.into());
        self
    }

    pub fn ad_stili(mut self, stil: YazıStili) -> Self {
        self.ad_stili = stil;
        self
    }

    pub fn ad_biçimleyici(mut self, biçimleyici: impl Into<Biçimleyici>) -> Self {
        self.ad_biçimleyici = Some(biçimleyici.into());
        self
    }

    pub fn ad_rengi_miras(mut self, miras: bool) -> Self {
        self.ad_rengi_miras = miras;
        if miras {
            self.ad_rengi = None;
        }
        self
    }

    pub fn değer_göster(mut self, göster: bool) -> Self {
        self.değeri_göster = göster;
        self
    }

    pub fn değer_animasyonu(mut self, açık: bool) -> Self {
        self.değer_animasyonu = açık;
        self
    }

    pub fn değer_duyarlılığı(mut self, basamak: usize) -> Self {
        self.değer_duyarlılığı = Some(basamak.min(20));
        self
    }

    pub fn değer_rengi_miras(mut self, miras: bool) -> Self {
        self.değer_rengi_miras = miras;
        if miras {
            self.değer_rengi = None;
        }
        self
    }

    pub fn değer_arkaplanı_miras(mut self, miras: bool) -> Self {
        self.değer_arkaplanı_miras = miras;
        self
    }

    pub fn değer_kenarlığı_miras(mut self, miras: bool) -> Self {
        self.değer_kenarlığı_miras = miras;
        self
    }

    pub fn değer_merkez_kayması<X: Into<Uzunluk>, Y: Into<Uzunluk>>(
        mut self,
        x: X,
        y: Y,
    ) -> Self {
        self.değer_merkez_kayması = (x.into(), y.into());
        self
    }

    pub fn değer_biçimleyici(mut self, b: impl Into<crate::model::stil::Biçimleyici>) -> Self {
        self.değer_biçimleyici = Some(b.into());
        self
    }

    pub fn değer_stili(mut self, stil: crate::model::stil::YazıStili) -> Self {
        self.değer_stili = stil;
        self
    }

    pub fn değer_zengin_stil(
        mut self,
        ad: impl Into<String>,
        stil: crate::model::stil::YazıStili,
    ) -> Self {
        self.değer_zengin.insert(ad.into(), stil);
        self
    }
}

/// Radar serisi (`series-radar`). Her veri öğesi, koordinattaki gösterge
/// sayısı kadar değerli bir dizidir; öğe adı göstergede (legend) listelenir.
#[derive(Clone, Debug, Default)]
pub struct RadarDurumYaması {
    pub çizgi_stili: Option<ÇizgiStili>,
    pub alan_stili: Option<AlanStili>,
    pub öğe_stili: Option<ÖğeStili>,
    pub etiket: Option<Etiket>,
}

impl RadarDurumYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = Some(stil);
        self
    }

    pub fn alan_stili(mut self, stil: AlanStili) -> Self {
        self.alan_stili = Some(stil);
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
}

/// Nesne biçimli `series-radar.data[i]` seçenekleri. Ham değer/ad ortak
/// [`VeriÖğesi`] üzerinde kalır; radar'a özgü sembol, çizgi, alan ve durum
/// yamaları bu dizide özgün veri sırasıyla tutulur.
#[derive(Clone, Debug, Default)]
pub struct RadarVeriYaması {
    pub sembol: Option<Sembol>,
    pub sembol_boyutu: Option<f32>,
    pub çizgi_stili: Option<ÇizgiStili>,
    pub alan_stili: Option<AlanStili>,
    pub öğe_stili: Option<ÖğeStili>,
    pub etiket: Option<Etiket>,
    pub vurgu: RadarDurumYaması,
    pub bulanık: RadarDurumYaması,
    pub seçili: RadarDurumYaması,
}

impl RadarVeriYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn sembol(mut self, sembol: Sembol) -> Self {
        self.sembol = Some(sembol);
        self
    }

    pub fn sembol_boyutu(mut self, boyut: f32) -> Self {
        self.sembol_boyutu = Some(boyut.max(0.0));
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = Some(stil);
        self
    }

    pub fn alan_stili(mut self, stil: AlanStili) -> Self {
        self.alan_stili = Some(stil);
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

    pub fn vurgu(mut self, yama: RadarDurumYaması) -> Self {
        self.vurgu = yama;
        self
    }

    pub fn bulanık(mut self, yama: RadarDurumYaması) -> Self {
        self.bulanık = yama;
        self
    }

    pub fn seçili(mut self, yama: RadarDurumYaması) -> Self {
        self.seçili = yama;
        self
    }
}

#[derive(Clone, Debug)]
pub struct RadarSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub veri_ayarları: Vec<RadarVeriYaması>,
    pub radar_sırası: usize,
    pub çizgi_stili: ÇizgiStili,
    pub alan_stili: Option<AlanStili>,
    pub öğe_stili: ÖğeStili,
    pub etiket: Etiket,
    pub sembol: Sembol,
    pub sembol_boyutu: f32,
    pub sembol_göster: bool,
    pub vurgu: RadarDurumYaması,
    pub bulanık: RadarDurumYaması,
    pub seçili: RadarDurumYaması,
    pub z: i32,
    pub sessiz: bool,
}

impl Default for RadarSerisi {
    fn default() -> Self {
        RadarSerisi {
            ad: None,
            veri: Vec::new(),
            veri_ayarları: Vec::new(),
            radar_sırası: 0,
            çizgi_stili: ÇizgiStili::default(),
            alan_stili: None,
            öğe_stili: ÖğeStili::default(),
            etiket: Etiket::yeni().konum(EtiketKonumu::Üst),
            sembol: Sembol::Daire,
            sembol_boyutu: 8.0,
            sembol_göster: true,
            vurgu: RadarDurumYaması::default(),
            bulanık: RadarDurumYaması::default(),
            seçili: RadarDurumYaması::default(),
            z: 2,
            sessiz: false,
        }
    }
}

impl RadarSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    /// Veri öğeleri: `(ad, değerler)` çiftleri.
    pub fn veri<S: Into<String>>(mut self, veri: impl IntoIterator<Item = (S, Vec<f64>)>) -> Self {
        self.veri = veri
            .into_iter()
            .map(|(ad, değerler)| VeriÖğesi::adlı(ad, değerler))
            .collect();
        self.veri_ayarları
            .resize_with(self.veri.len(), Default::default);
        self
    }

    pub fn veri_öğeleri<T: Into<VeriÖğesi>>(
        mut self,
        veri: impl IntoIterator<Item = T>,
    ) -> Self {
        self.veri = veri_listesi(veri);
        self.veri_ayarları
            .resize_with(self.veri.len(), Default::default);
        self
    }

    pub fn radar_sırası(mut self, sıra: usize) -> Self {
        self.radar_sırası = sıra;
        self
    }

    pub fn alan_stili(mut self, stil: AlanStili) -> Self {
        self.alan_stili = Some(stil);
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = stil;
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

    pub fn sembol(mut self, sembol: Sembol) -> Self {
        self.sembol_göster = !matches!(sembol, Sembol::Yok);
        self.sembol = sembol;
        self
    }

    pub fn sembol_boyutu(mut self, boyut: f32) -> Self {
        self.sembol_boyutu = boyut.max(0.0);
        self
    }

    pub fn sembol_göster(mut self, göster: bool) -> Self {
        self.sembol_göster = göster;
        self
    }

    pub fn vurgu(mut self, yama: RadarDurumYaması) -> Self {
        self.vurgu = yama;
        self
    }

    pub fn bulanık(mut self, yama: RadarDurumYaması) -> Self {
        self.bulanık = yama;
        self
    }

    pub fn seçili(mut self, yama: RadarDurumYaması) -> Self {
        self.seçili = yama;
        self
    }

    pub fn veri_ayarı(mut self, sıra: usize, ayar: RadarVeriYaması) -> Self {
        if self.veri_ayarları.len() <= sıra {
            self.veri_ayarları.resize_with(sıra + 1, Default::default);
        }
        self.veri_ayarları[sıra] = ayar;
        self
    }

    pub fn z(mut self, z: i32) -> Self {
        self.z = z;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }
}

/// Özel seri çizim bağlamı: kullanıcının çizim işlevine geçirilir.
pub struct ÖzelBağlam<'a> {
    /// Etkin koordinat sisteminin çizim alanı (koordinat yoksa tuvalin tamamı).
    pub alan: crate::koordinat::Dikdörtgen,
    /// Kartezyen koordinat sistemi (eksenler kuruluysa).
    pub kartezyen: Option<&'a crate::koordinat::Kartezyen2B>,
    /// Takvim koordinat sistemi (`coordinateSystem: 'calendar'` ise).
    pub takvim: Option<&'a crate::koordinat::TakvimYerleşimi>,
    /// Matrix koordinat sistemi (`coordinateSystem: 'matrix'` ise).
    pub matris: Option<&'a crate::koordinat::MatrisYerleşimi>,
    pub veri: &'a [VeriÖğesi],
    /// Paletten çözülen seri rengi.
    pub renk: crate::renk::Renk,
    /// Giriş animasyonu ilerlemesi `0..=1`.
    pub ilerleme: f32,
}

/// Özel çizim işlevi (`series-custom` içindeki `renderItem` karşılığı).
pub type ÖzelÇizim = Arc<dyn Fn(&mut dyn crate::cizim::ÇizimYüzeyi, &ÖzelBağlam) + Send + Sync>;

/// Özel seri (`series-custom`): çizim tümüyle kullanıcı işlevine bırakılır.
/// Bu aynı zamanda üçüncü taraf seri türleri için eklenti noktasıdır.
#[derive(Clone)]
pub struct ÖzelSeri {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub çizim: Option<ÖzelÇizim>,
    /// Eksen/ızgara kurulumu gerekli mi? `false` ise tuvalin tamamı verilir.
    pub kartezyen_gerekli: bool,
    /// Bağlı takvim (`calendarIndex`); doluysa kartezyen kurulmaz.
    pub takvim_sırası: Option<usize>,
    /// Bağlı Matrix (`matrixIndex`); doluysa kartezyen kurulmaz.
    pub matris_sırası: Option<usize>,
    /// `matrix.x/y.data` boşken bu seriden toplanacak ordinal boyutlar.
    pub matris_x_kategorileri: Vec<String>,
    pub matris_y_kategorileri: Vec<String>,
    /// Bağlı eksenler (`xAxisIndex`/`yAxisIndex`).
    pub eksen_bağı: EksenBağı,
}

impl fmt::Debug for ÖzelSeri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ÖzelSeri")
            .field("ad", &self.ad)
            .field("veri", &self.veri.len())
            .field("kartezyen_gerekli", &self.kartezyen_gerekli)
            .field("takvim_sırası", &self.takvim_sırası)
            .field("matris_sırası", &self.matris_sırası)
            .finish()
    }
}

impl Default for ÖzelSeri {
    fn default() -> Self {
        ÖzelSeri {
            ad: None,
            veri: Vec::new(),
            çizim: None,
            kartezyen_gerekli: true,
            takvim_sırası: None,
            matris_sırası: None,
            matris_x_kategorileri: Vec::new(),
            matris_y_kategorileri: Vec::new(),
            eksen_bağı: EksenBağı::default(),
        }
    }
}

impl ÖzelSeri {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Seriyi verilen x/y eksen sıralarına bağlar (`xAxisIndex`/`yAxisIndex`).
    pub fn eksenler(mut self, x: usize, y: usize) -> Self {
        self.eksen_bağı = EksenBağı { x, y };
        self.kartezyen_gerekli = true;
        self.takvim_sırası = None;
        self.matris_sırası = None;
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self
    }

    /// Çizim işlevini ayarlar.
    pub fn çizim(
        mut self,
        işlev: impl Fn(&mut dyn crate::cizim::ÇizimYüzeyi, &ÖzelBağlam) + Send + Sync + 'static,
    ) -> Self {
        self.çizim = Some(Arc::new(işlev));
        self
    }

    pub fn kartezyen_gerekli(mut self, gerekli: bool) -> Self {
        self.kartezyen_gerekli = gerekli;
        if gerekli {
            self.takvim_sırası = None;
            self.matris_sırası = None;
        }
        self
    }

    /// Seriyi bir takvim koordinatına bağlar (`coordinateSystem: 'calendar'`
    /// ve `calendarIndex`). Çizim bağlamındaki `takvim` alanı bu yerleşimi
    /// taşır; tarihleri piksele çevirmek için `veriden_noktaya` kullanılabilir.
    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = Some(sıra);
        self.matris_sırası = None;
        self.kartezyen_gerekli = false;
        self
    }

    /// Seriyi bir Matrix koordinatına bağlar. Çizim bağlamındaki `matris`
    /// üzerinden `veriden_noktaya` ve `veriden_yerleşime` kullanılabilir.
    pub fn matris_sırası(mut self, sıra: usize) -> Self {
        self.matris_sırası = Some(sıra);
        self.takvim_sırası = None;
        self.kartezyen_gerekli = false;
        self
    }

    /// Dataset `encode.x/y` ordinal toplamasının açık Rust karşılığı.
    pub fn matris_kategorileri<X: Into<String>, Y: Into<String>>(
        mut self,
        x: impl IntoIterator<Item = X>,
        y: impl IntoIterator<Item = Y>,
    ) -> Self {
        self.matris_x_kategorileri = x.into_iter().map(Into::into).collect();
        self.matris_y_kategorileri = y.into_iter().map(Into::into).collect();
        self
    }
}

/// Ağaç haritası serisi (`series-treemap`): kareselleştirilmiş yerleşim.
#[derive(Clone, Debug)]
pub struct AğaçHaritasıSerisi {
    pub ad: Option<String>,
    pub kökler: Vec<crate::model::agac::AğaçDüğümü>,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
    /// Hücreler arası boşluk.
    pub hücre_boşluğu: f32,
    /// Gösterilecek en çok derinlik (0 = yalnız kökler).
    pub en_çok_derinlik: usize,
}

impl Default for AğaçHaritasıSerisi {
    fn default() -> Self {
        AğaçHaritasıSerisi {
            ad: None,
            kökler: Vec::new(),
            sol: Uzunluk::Yüzde(5.0),
            üst: Uzunluk::Piksel(50.0),
            genişlik: Uzunluk::Yüzde(90.0),
            yükseklik: Uzunluk::Yüzde(80.0),
            hücre_boşluğu: 2.0,
            en_çok_derinlik: 2,
        }
    }
}

impl AğaçHaritasıSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn kökler(
        mut self,
        kökler: impl IntoIterator<Item = crate::model::agac::AğaçDüğümü>,
    ) -> Self {
        self.kökler = kökler.into_iter().collect();
        self
    }

    pub fn en_çok_derinlik(mut self, derinlik: usize) -> Self {
        self.en_çok_derinlik = derinlik;
        self
    }
}

/// Güneş patlaması serisi (`series-sunburst`): iç içe halkalar.
#[derive(Clone, Debug)]
pub struct GüneşPatlamasıSerisi {
    pub ad: Option<String>,
    pub kökler: Vec<crate::model::agac::AğaçDüğümü>,
    pub merkez: (Uzunluk, Uzunluk),
    /// `(iç, dış)` yarıçap.
    pub yarıçap: (Uzunluk, Uzunluk),
}

impl Default for GüneşPatlamasıSerisi {
    fn default() -> Self {
        GüneşPatlamasıSerisi {
            ad: None,
            kökler: Vec::new(),
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(55.0)),
            yarıçap: (Uzunluk::Yüzde(12.0), Uzunluk::Yüzde(75.0)),
        }
    }
}

impl GüneşPatlamasıSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn kökler(
        mut self,
        kökler: impl IntoIterator<Item = crate::model::agac::AğaçDüğümü>,
    ) -> Self {
        self.kökler = kökler.into_iter().collect();
        self
    }

    pub fn halka(mut self, iç: impl Into<Uzunluk>, dış: impl Into<Uzunluk>) -> Self {
        self.yarıçap = (iç.into(), dış.into());
        self
    }
}

/// Ağaç serisi (`series-tree`): ECharts'ın orthogonal/radial Tree modeli.
#[derive(Clone, Debug)]
pub struct AğaçSerisi {
    pub kimlik: Option<String>,
    pub ad: Option<String>,
    pub kökler: Vec<crate::model::agac::AğaçDüğümü>,
    pub z: i32,
    pub sessiz: bool,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    /// `Some` iken genişlik sağ kenardan çözülür; açık `genişlik(...)`
    /// çağrısı bunu `None` yapar.
    pub sağ: Option<Uzunluk>,
    /// `Some` iken yükseklik alt kenardan çözülür.
    pub alt: Option<Uzunluk>,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
    pub yerleşim: AğaçYerleşimi,
    pub yön: AğaçYönü,
    pub kenar_biçimi: AğaçKenarBiçimi,
    /// `edgeForkPosition`; `0..=1` görünüm oranı.
    pub kenar_çatal_oranı: f32,
    /// `lineStyle.curveness`.
    pub kenar_eğriliği: f32,
    pub sembol: Sembol,
    pub sembol_boyutu: f32,
    pub sembol_döndürme: Option<f32>,
    pub sembol_kayması: (Uzunluk, Uzunluk),
    pub sembol_oranını_koru: bool,
    pub genişlet_ve_daralt: bool,
    /// ECharts `initialTreeDepth`; negatif değer tüm ağacı açar.
    pub ilk_ağaç_derinliği: isize,
    pub gezinme: AğaçGezinmesi,
    pub düğüm_ölçek_oranı: f32,
    pub merkez: Option<(Uzunluk, Uzunluk)>,
    pub yakınlaştırma: f32,
    pub öğe_stili: ÖğeStili,
    pub çizgi_stili: ÇizgiStili,
    pub etiket: Etiket,
    pub yaprak_etiketi: EtiketYaması,
    pub vurgu_odağı: AğaçVurguOdağı,
    pub vurgu_ölçekle: bool,
    pub vurgu_öğe_stili: Option<ÖğeStili>,
    pub vurgu_çizgi_stili: Option<ÇizgiStili>,
    pub vurgu_etiketi: Option<EtiketYaması>,
    pub bulanık_öğe_stili: Option<ÖğeStili>,
    pub bulanık_çizgi_stili: Option<ÇizgiStili>,
    pub bulanık_etiketi: Option<EtiketYaması>,
    pub seçili_öğe_stili: Option<ÖğeStili>,
    pub seçili_çizgi_stili: Option<ÇizgiStili>,
    pub seçili_etiketi: Option<EtiketYaması>,
    pub ipucu: Option<İpucu>,
    /// Büyük ağaçlarda kararlı çizim parça boyutu.
    pub ilerlemeli: usize,
    pub ilerlemeli_eşik: usize,
}

impl Default for AğaçSerisi {
    fn default() -> Self {
        AğaçSerisi {
            kimlik: None,
            ad: None,
            kökler: Vec::new(),
            z: 2,
            sessiz: false,
            sol: Uzunluk::Yüzde(12.0),
            üst: Uzunluk::Yüzde(12.0),
            sağ: Some(Uzunluk::Yüzde(12.0)),
            alt: Some(Uzunluk::Yüzde(12.0)),
            genişlik: Uzunluk::Yüzde(76.0),
            yükseklik: Uzunluk::Yüzde(76.0),
            yerleşim: AğaçYerleşimi::Dik,
            yön: AğaçYönü::SoldanSağa,
            kenar_biçimi: AğaçKenarBiçimi::Eğri,
            kenar_çatal_oranı: 0.5,
            kenar_eğriliği: 0.5,
            sembol: Sembol::İçiBoşDaire,
            sembol_boyutu: 7.0,
            sembol_döndürme: None,
            sembol_kayması: (Uzunluk::Piksel(0.0), Uzunluk::Piksel(0.0)),
            sembol_oranını_koru: false,
            genişlet_ve_daralt: true,
            ilk_ağaç_derinliği: 2,
            gezinme: AğaçGezinmesi::Kapalı,
            düğüm_ölçek_oranı: 0.4,
            merkez: None,
            yakınlaştırma: 1.0,
            öğe_stili: ÖğeStili::yeni()
                .renk(Renk::onaltılık(0xb0c4de))
                .kenarlık_kalınlığı(1.5),
            çizgi_stili: ÇizgiStili::yeni()
                .renk(Renk::onaltılık(0xcfd2d7))
                .kalınlık(1.5),
            etiket: Etiket::yeni().göster(true).konum(EtiketKonumu::Sağ),
            yaprak_etiketi: EtiketYaması::yeni(),
            vurgu_odağı: AğaçVurguOdağı::Yok,
            vurgu_ölçekle: true,
            vurgu_öğe_stili: None,
            vurgu_çizgi_stili: None,
            vurgu_etiketi: None,
            bulanık_öğe_stili: None,
            bulanık_çizgi_stili: None,
            bulanık_etiketi: None,
            seçili_öğe_stili: None,
            seçili_çizgi_stili: None,
            seçili_etiketi: None,
            ipucu: None,
            ilerlemeli: 500,
            ilerlemeli_eşik: 3_000,
        }
    }
}

impl AğaçSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn kimlik(mut self, kimlik: impl Into<String>) -> Self {
        self.kimlik = Some(kimlik.into());
        self
    }

    pub fn kökler(
        mut self,
        kökler: impl IntoIterator<Item = crate::model::agac::AğaçDüğümü>,
    ) -> Self {
        self.kökler = kökler.into_iter().collect();
        self
    }

    pub fn z(mut self, z: i32) -> Self {
        self.z = z;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }

    pub fn sol(mut self, sol: impl Into<Uzunluk>) -> Self {
        self.sol = sol.into();
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = üst.into();
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(sağ.into());
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    pub fn genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.genişlik = genişlik.into();
        self.sağ = None;
        self
    }

    pub fn yükseklik(mut self, yükseklik: impl Into<Uzunluk>) -> Self {
        self.yükseklik = yükseklik.into();
        self.alt = None;
        self
    }

    pub fn yerleşim(mut self, yerleşim: AğaçYerleşimi) -> Self {
        self.yerleşim = yerleşim;
        self
    }

    pub fn yön(mut self, yön: AğaçYönü) -> Self {
        self.yön = yön;
        self
    }

    pub fn kenar_biçimi(mut self, biçim: AğaçKenarBiçimi) -> Self {
        self.kenar_biçimi = biçim;
        self
    }

    pub fn kenar_çatal_yüzdesi(mut self, yüzde: f32) -> Self {
        self.kenar_çatal_oranı = (yüzde / 100.0).clamp(0.0, 1.0);
        self
    }

    pub fn kenar_çatal_oranı(mut self, oran: f32) -> Self {
        self.kenar_çatal_oranı = oran.clamp(0.0, 1.0);
        self
    }

    pub fn kenar_eğriliği(mut self, eğrilik: f32) -> Self {
        self.kenar_eğriliği = eğrilik.clamp(0.0, 1.0);
        self
    }

    pub fn sembol(mut self, sembol: Sembol) -> Self {
        self.sembol = sembol;
        self
    }

    pub fn sembol_boyutu(mut self, boyut: f32) -> Self {
        self.sembol_boyutu = boyut.max(0.0);
        self
    }

    pub fn sembol_döndürme(mut self, derece: f32) -> Self {
        self.sembol_döndürme = derece.is_finite().then_some(derece);
        self
    }

    pub fn sembol_kayması(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.sembol_kayması = (x.into(), y.into());
        self
    }

    pub fn sembol_oranını_koru(mut self, koru: bool) -> Self {
        self.sembol_oranını_koru = koru;
        self
    }

    pub fn genişlet_ve_daralt(mut self, açık: bool) -> Self {
        self.genişlet_ve_daralt = açık;
        self
    }

    pub fn ilk_ağaç_derinliği(mut self, derinlik: isize) -> Self {
        self.ilk_ağaç_derinliği = derinlik;
        self
    }

    pub fn gezinme(mut self, gezinme: AğaçGezinmesi) -> Self {
        self.gezinme = gezinme;
        self
    }

    pub fn düğüm_ölçek_oranı(mut self, oran: f32) -> Self {
        self.düğüm_ölçek_oranı = oran.max(0.0);
        self
    }

    pub fn merkez(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.merkez = Some((x.into(), y.into()));
        self
    }

    pub fn yakınlaştırma(mut self, yakınlaştırma: f32) -> Self {
        self.yakınlaştırma = yakınlaştırma.max(0.01);
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn yaprak_etiketi(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.yaprak_etiketi = etiket.into();
        self
    }

    pub fn vurgu_odağı(mut self, odak: AğaçVurguOdağı) -> Self {
        self.vurgu_odağı = odak;
        self
    }

    pub fn vurgu_ölçekle(mut self, ölçekle: bool) -> Self {
        self.vurgu_ölçekle = ölçekle;
        self
    }

    pub fn vurgu_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.vurgu_öğe_stili = Some(stil);
        self
    }

    pub fn vurgu_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.vurgu_çizgi_stili = Some(stil);
        self
    }

    pub fn vurgu_etiketi(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.vurgu_etiketi = Some(etiket.into());
        self
    }

    pub fn bulanık_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.bulanık_öğe_stili = Some(stil);
        self
    }

    pub fn bulanık_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.bulanık_çizgi_stili = Some(stil);
        self
    }

    pub fn bulanık_etiketi(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.bulanık_etiketi = Some(etiket.into());
        self
    }

    pub fn seçili_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.seçili_öğe_stili = Some(stil);
        self
    }

    pub fn seçili_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.seçili_çizgi_stili = Some(stil);
        self
    }

    pub fn seçili_etiketi(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.seçili_etiketi = Some(etiket.into());
        self
    }

    /// Ön-sıralı ECharts `dataIndex` için kökten düğüme ad yolunu verir.
    /// Daraltılmış alt ağaçlar da veri modelinde kaldığından sıra hesabına
    /// katılır; renderer isabet sırasıyla birebir aynıdır.
    pub fn düğüm_yolu(&self, hedef: usize) -> Option<Vec<String>> {
        fn ara(
            düğümler: &[crate::model::agac::AğaçDüğümü],
            hedef: usize,
            sayaç: &mut usize,
            yol: &mut Vec<String>,
        ) -> Option<Vec<String>> {
            for düğüm in düğümler {
                let sıra = *sayaç;
                *sayaç = sayaç.saturating_add(1);
                yol.push(düğüm.ad.clone());
                if sıra == hedef {
                    return Some(yol.clone());
                }
                if let Some(bulunan) = ara(&düğüm.çocuklar, hedef, sayaç, yol) {
                    return Some(bulunan);
                }
                yol.pop();
            }
            None
        }

        let mut sayaç = 0;
        let mut yol = Vec::new();
        ara(&self.kökler, hedef, &mut sayaç, &mut yol)
    }

    /// `treeExpandAndCollapse` eyleminin model karşılığı. Dönüş değeri
    /// `(düğüm adı, yeni collapsed durumu)`dur; yapraklar değişmez.
    pub fn düğüm_daraltmasını_değiştir(&mut self, hedef: usize) -> Option<(String, bool)> {
        fn ara(
            düğümler: &mut [crate::model::agac::AğaçDüğümü],
            hedef: usize,
            sayaç: &mut usize,
            derinlik: usize,
            genişlet_ve_daralt: bool,
            ilk_derinlik: isize,
        ) -> Option<(String, bool)> {
            for düğüm in düğümler {
                let sıra = *sayaç;
                *sayaç = sayaç.saturating_add(1);
                if sıra == hedef {
                    if düğüm.çocuklar.is_empty() {
                        return None;
                    }
                    let açık = düğüm.daraltılmış.map(|dar| !dar).unwrap_or_else(|| {
                        !genişlet_ve_daralt
                            || ilk_derinlik < 0
                            || derinlik.saturating_add(1) as isize <= ilk_derinlik
                    });
                    let daraltılmış = açık;
                    düğüm.daraltılmış = Some(daraltılmış);
                    return Some((düğüm.ad.clone(), daraltılmış));
                }
                if let Some(bulunan) = ara(
                    &mut düğüm.çocuklar,
                    hedef,
                    sayaç,
                    derinlik.saturating_add(1),
                    genişlet_ve_daralt,
                    ilk_derinlik,
                ) {
                    return Some(bulunan);
                }
            }
            None
        }

        let genişlet_ve_daralt = self.genişlet_ve_daralt;
        let ilk_derinlik = self.ilk_ağaç_derinliği;
        let mut sayaç = 0;
        ara(
            &mut self.kökler,
            hedef,
            &mut sayaç,
            0,
            genişlet_ve_daralt,
            ilk_derinlik,
        )
    }

    pub fn ipucu(mut self, ipucu: İpucu) -> Self {
        self.ipucu = Some(ipucu);
        self
    }

    pub fn ilerlemeli(mut self, parça_boyutu: usize) -> Self {
        self.ilerlemeli = parça_boyutu.max(1);
        self
    }

    pub fn ilerlemeli_eşik(mut self, eşik: usize) -> Self {
        self.ilerlemeli_eşik = eşik;
        self
    }
}

/// Sankey bağı (`links` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct SankeyBağı {
    pub kaynak: String,
    pub hedef: String,
    pub değer: f64,
}

/// Sankey serisi (`series-sankey`): katmanlı akış diyagramı.
#[derive(Clone, Debug)]
pub struct SankeySerisi {
    pub ad: Option<String>,
    /// Açık düğüm listesi; boşsa bağlardan türetilir.
    pub düğümler: Vec<String>,
    pub bağlar: Vec<SankeyBağı>,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
    pub düğüm_genişliği: f32,
    pub düğüm_boşluğu: f32,
}

impl Default for SankeySerisi {
    fn default() -> Self {
        SankeySerisi {
            ad: None,
            düğümler: Vec::new(),
            bağlar: Vec::new(),
            sol: Uzunluk::Yüzde(8.0),
            üst: Uzunluk::Piksel(60.0),
            genişlik: Uzunluk::Yüzde(80.0),
            yükseklik: Uzunluk::Yüzde(75.0),
            düğüm_genişliği: 18.0,
            düğüm_boşluğu: 10.0,
        }
    }
}

impl SankeySerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    /// Bağlar: `(kaynak, hedef, değer)` üçlüleri.
    pub fn bağlar<S: Into<String>>(
        mut self,
        bağlar: impl IntoIterator<Item = (S, S, f64)>,
    ) -> Self {
        self.bağlar = bağlar
            .into_iter()
            .map(|(k, h, d)| SankeyBağı {
                kaynak: k.into(),
                hedef: h.into(),
                değer: d,
            })
            .collect();
        self
    }
}

/// Grafo düğümü (`graph` `data` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct GrafoDüğümü {
    pub ad: String,
    pub değer: Option<f64>,
    /// Takvim koordinatındaki tarih (`data[i][0]`, Unix milisaniyesi).
    pub takvim_tarihi_ms: Option<f64>,
    /// Matrix koordinatındaki hücre ya da aralık.
    pub matris_koordinatı: Option<(MatrisAralığı, MatrisAralığı)>,
    /// Sembol çapı (`symbolSize`).
    pub boyut: f32,
    /// Renk grubu (palet sırası); `None` düğüm sırasını kullanır.
    pub kategori: Option<usize>,
}

impl GrafoDüğümü {
    pub fn yeni(ad: impl Into<String>, boyut: f32) -> Self {
        GrafoDüğümü {
            ad: ad.into(),
            değer: None,
            takvim_tarihi_ms: None,
            matris_koordinatı: None,
            boyut,
            kategori: None,
        }
    }

    pub fn kategori(mut self, kategori: usize) -> Self {
        self.kategori = Some(kategori);
        self
    }

    pub fn değerli(mut self, değer: f64) -> Self {
        self.değer = Some(değer);
        self
    }

    /// Düğümü takvim koordinatındaki bir güne bağlar.
    pub fn takvim_tarihi(mut self, tarih_ms: f64) -> Self {
        self.takvim_tarihi_ms = Some(tarih_ms);
        self.matris_koordinatı = None;
        self
    }

    pub fn matris_koordinatı(
        mut self,
        x: impl Into<MatrisAralığı>,
        y: impl Into<MatrisAralığı>,
    ) -> Self {
        self.matris_koordinatı = Some((x.into(), y.into()));
        self.takvim_tarihi_ms = None;
        self
    }
}

/// Grafo yerleşimi (`graph.layout`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoYerleşimi {
    /// Veri/koordinat sisteminin verdiği konum (`null` / `'none'`).
    Yok,
    /// Kuvvet yönlendirmeli (`'force'`) — belirlenimci.
    #[default]
    Kuvvet,
    /// Çember üzerinde (`'circular'`).
    Dairesel,
}

/// Grafo serisi (`series-graph`).
#[derive(Clone, Debug)]
pub struct GrafoSerisi {
    pub ad: Option<String>,
    pub düğümler: Vec<GrafoDüğümü>,
    /// Bağlar: `(kaynak ad, hedef ad)`.
    pub bağlar: Vec<(String, String)>,
    pub yerleşim: GrafoYerleşimi,
    pub merkez: (Uzunluk, Uzunluk),
    pub yarıçap: Uzunluk,
    /// İtme çarpanı (`force.repulsion` ölçeği).
    pub itme: f32,
    /// Bağ uzunluğu çarpanı (`force.edgeLength` ölçeği).
    pub kenar_uzunluğu: f32,
    /// Bu çaptan büyük düğümlerde ad etiketi gösterilir.
    pub etiket_eşiği: f32,
    /// Normal durumda etiket çizilir mi (`label.show`).
    pub etiket_göster: bool,
    /// Düğüm etiketi seçenekleri (`label`). Eski `etiket_göster` alanı
    /// kaynak uyumu için korunur; çizim ikisinden herhangi biri açıksa
    /// etiketi üretir.
    pub etiket: Etiket,
    /// Takvim koordinatına bağlıysa `calendarIndex`.
    pub takvim_sırası: Option<usize>,
    /// Matrix koordinatına bağlıysa `matrixIndex`.
    pub matris_sırası: Option<usize>,
    /// Seri çizim sırası (`z`); CalendarView öntanımlı z=2'dir.
    pub z: i32,
    /// Düğüm `itemStyle`ı.
    pub öğe_stili: ÖğeStili,
    /// Kenar `lineStyle`ı.
    pub çizgi_stili: ÇizgiStili,
    /// Hedef uçta öntanımlı 10 px ok (`edgeSymbol: ['none', 'arrow']`).
    pub hedef_oku: bool,
    pub hedef_oku_boyutu: f32,
}

impl Default for GrafoSerisi {
    fn default() -> Self {
        GrafoSerisi {
            ad: None,
            düğümler: Vec::new(),
            bağlar: Vec::new(),
            yerleşim: GrafoYerleşimi::Kuvvet,
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(55.0)),
            yarıçap: Uzunluk::Yüzde(78.0),
            itme: 1.0,
            kenar_uzunluğu: 1.0,
            etiket_eşiği: 12.0,
            etiket_göster: false,
            etiket: Etiket::yeni(),
            takvim_sırası: None,
            matris_sırası: None,
            z: 2,
            öğe_stili: ÖğeStili::default(),
            çizgi_stili: ÇizgiStili::yeni().kalınlık(1.0).opaklık(0.5),
            hedef_oku: false,
            hedef_oku_boyutu: 10.0,
        }
    }
}

impl GrafoSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn düğümler(mut self, düğümler: impl IntoIterator<Item = GrafoDüğümü>) -> Self {
        self.düğümler = düğümler.into_iter().collect();
        self
    }

    pub fn bağlar<S: Into<String>>(mut self, bağlar: impl IntoIterator<Item = (S, S)>) -> Self {
        self.bağlar = bağlar
            .into_iter()
            .map(|(k, h)| (k.into(), h.into()))
            .collect();
        self
    }

    pub fn yerleşim(mut self, yerleşim: GrafoYerleşimi) -> Self {
        self.yerleşim = yerleşim;
        self
    }

    pub fn etiket_göster(mut self, göster: bool) -> Self {
        self.etiket_göster = göster;
        self.etiket.göster = göster;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket_göster = etiket.göster;
        self.etiket = etiket;
        self
    }

    pub fn etiket_eşiği(mut self, eşik: f32) -> Self {
        self.etiket_eşiği = eşik.max(0.0);
        self
    }

    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = Some(sıra);
        self.matris_sırası = None;
        self
    }

    pub fn matris_sırası(mut self, sıra: usize) -> Self {
        self.matris_sırası = Some(sıra);
        self.takvim_sırası = None;
        self
    }

    pub fn z(mut self, z: i32) -> Self {
        self.z = z;
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = stil;
        self
    }

    pub fn hedef_oku(mut self, açık: bool) -> Self {
        self.hedef_oku = açık;
        self
    }

    pub fn hedef_oku_boyutu(mut self, boyut: f32) -> Self {
        self.hedef_oku_boyutu = boyut.max(0.0);
        self
    }
}

/// Kiriş serisi (`series-chord`): çember üzerindeki düğümler arasında
/// merkezden geçen akış şeritleri.
#[derive(Clone, Debug)]
pub struct KirişSerisi {
    pub ad: Option<String>,
    /// Akışlar: `(kaynak, hedef, değer)`.
    pub bağlar: Vec<(String, String, f64)>,
    pub merkez: (Uzunluk, Uzunluk),
    pub yarıçap: Uzunluk,
    pub şerit_kalınlığı: f32,
}

impl Default for KirişSerisi {
    fn default() -> Self {
        KirişSerisi {
            ad: None,
            bağlar: Vec::new(),
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(55.0)),
            yarıçap: Uzunluk::Yüzde(72.0),
            şerit_kalınlığı: 16.0,
        }
    }
}

impl KirişSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn bağlar<S: Into<String>>(
        mut self,
        bağlar: impl IntoIterator<Item = (S, S, f64)>,
    ) -> Self {
        self.bağlar = bağlar
            .into_iter()
            .map(|(k, h, d)| (k.into(), h.into(), d))
            .collect();
        self
    }
}

/// Paralel koordinat boyutu (`parallelAxis` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct ParalelBoyut {
    pub ad: String,
    pub en_az: Option<f64>,
    pub en_çok: Option<f64>,
}

impl ParalelBoyut {
    pub fn yeni(ad: impl Into<String>) -> Self {
        ParalelBoyut {
            ad: ad.into(),
            en_az: None,
            en_çok: None,
        }
    }
}

/// Paralel koordinat serisi (`series-parallel`).
#[derive(Clone, Debug)]
pub struct ParalelSerisi {
    pub ad: Option<String>,
    /// SeriesModel varsayılanı `z: 2`.
    pub z: i32,
    pub sessiz: bool,
    /// Bağlı `parallel` bileşeninin sırası (`parallelIndex`).
    pub paralel_sırası: usize,
    /// İndekse alternatif `parallelId` bağı.
    pub paralel_kimliği: Option<String>,
    pub boyutlar: Vec<ParalelBoyut>,
    /// Her öğe, boyut sayısı kadar değerli bir dizidir.
    pub veri: Vec<VeriÖğesi>,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
    pub çizgi_stili: ÇizgiStili,
    pub vurgu_çizgi_stili: Option<ÇizgiStili>,
    pub bulanık_çizgi_stili: Option<ÇizgiStili>,
    pub seçili_çizgi_stili: Option<ÇizgiStili>,
    pub etiket: Etiket,
    pub vurgu_etiketi: Option<EtiketYaması>,
    pub bulanık_etiketi: Option<EtiketYaması>,
    pub seçili_etiketi: Option<EtiketYaması>,
    pub aktif_opaklık: f32,
    pub etkin_değil_opaklık: f32,
    /// `smooth: true` için 0.3, kapalı için 0; sayısal smooth doğrudan.
    pub yumuşaklık: f32,
    pub gerçek_zamanlı: bool,
    pub ipucu: Option<İpucu>,
    /// `series.parallel.parallelAxisDefault`; örtük koordinat üretiminde
    /// parallel bileşeninin eksen varsayılanına aktarılır.
    pub eksen_varsayılanı: Option<Eksen>,
    /// Artımlı çizim parça boyutu (`progressive`, varsayılan 300).
    pub ilerlemeli: usize,
}

impl Default for ParalelSerisi {
    fn default() -> Self {
        ParalelSerisi {
            ad: None,
            z: 2,
            sessiz: false,
            paralel_sırası: 0,
            paralel_kimliği: None,
            boyutlar: Vec::new(),
            veri: Vec::new(),
            sol: Uzunluk::Yüzde(8.0),
            üst: Uzunluk::Piksel(70.0),
            genişlik: Uzunluk::Yüzde(84.0),
            yükseklik: Uzunluk::Yüzde(70.0),
            çizgi_stili: ÇizgiStili {
                kalınlık: 1.0,
                opaklık: 0.45,
                ..Default::default()
            },
            vurgu_çizgi_stili: None,
            bulanık_çizgi_stili: None,
            seçili_çizgi_stili: None,
            etiket: Etiket::yeni().göster(false),
            vurgu_etiketi: Some(EtiketYaması::yeni().göster(false)),
            bulanık_etiketi: None,
            seçili_etiketi: None,
            aktif_opaklık: 1.0,
            etkin_değil_opaklık: 0.05,
            yumuşaklık: 0.0,
            gerçek_zamanlı: true,
            ipucu: None,
            eksen_varsayılanı: None,
            ilerlemeli: 300,
        }
    }
}

impl ParalelSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn z(mut self, z: i32) -> Self {
        self.z = z;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }

    pub fn paralel_sırası(mut self, sıra: usize) -> Self {
        self.paralel_sırası = sıra;
        self
    }

    pub fn paralel_kimliği(mut self, kimlik: impl Into<String>) -> Self {
        self.paralel_kimliği = Some(kimlik.into());
        self
    }

    pub fn boyutlar<S: Into<String>>(mut self, boyutlar: impl IntoIterator<Item = S>) -> Self {
        self.boyutlar = boyutlar.into_iter().map(ParalelBoyut::yeni).collect();
        self
    }

    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self
    }

    /// Sayı, metin, zaman ve boş değerleri aynı ECharts veri satırında taşır.
    pub fn karma_veri<R, T>(mut self, veri: impl IntoIterator<Item = R>) -> Self
    where
        R: IntoIterator<Item = T>,
        T: Into<VeriDeğeri>,
    {
        self.veri = veri
            .into_iter()
            .map(|satır| {
                VeriÖğesi::yeni(VeriDeğeri::KarmaDizi(
                    satır.into_iter().map(Into::into).collect(),
                ))
            })
            .collect();
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = stil;
        self
    }

    pub fn vurgu_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.vurgu_çizgi_stili = Some(stil);
        self
    }

    pub fn bulanık_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.bulanık_çizgi_stili = Some(stil);
        self
    }

    pub fn seçili_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.seçili_çizgi_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn vurgu_etiketi(mut self, etiket: EtiketYaması) -> Self {
        self.vurgu_etiketi = Some(etiket);
        self
    }

    pub fn bulanık_etiketi(mut self, etiket: EtiketYaması) -> Self {
        self.bulanık_etiketi = Some(etiket);
        self
    }

    pub fn seçili_etiketi(mut self, etiket: EtiketYaması) -> Self {
        self.seçili_etiketi = Some(etiket);
        self
    }

    pub fn aktif_opaklık(mut self, opaklık: f32) -> Self {
        self.aktif_opaklık = opaklık.clamp(0.0, 1.0);
        self
    }

    pub fn etkin_değil_opaklık(mut self, opaklık: f32) -> Self {
        self.etkin_değil_opaklık = opaklık.clamp(0.0, 1.0);
        self
    }

    pub fn yumuşak(mut self, açık: bool) -> Self {
        self.yumuşaklık = if açık { 0.3 } else { 0.0 };
        self
    }

    pub fn yumuşaklık(mut self, yumuşaklık: f32) -> Self {
        self.yumuşaklık = yumuşaklık.clamp(0.0, 1.0);
        self
    }

    pub fn gerçek_zamanlı(mut self, açık: bool) -> Self {
        self.gerçek_zamanlı = açık;
        self
    }

    pub fn ipucu(mut self, ipucu: İpucu) -> Self {
        self.ipucu = Some(ipucu);
        self
    }

    pub fn eksen_varsayılanı(mut self, eksen: Eksen) -> Self {
        self.eksen_varsayılanı = Some(eksen);
        self
    }

    pub fn ilerlemeli(mut self, parça_boyutu: usize) -> Self {
        self.ilerlemeli = parça_boyutu;
        self
    }
}

/// Takvim ısı serisi (`coordinateSystem: 'calendar'` + heatmap karşılığı).
/// Veri öğeleri `[gün (Unix ms), değer]` dizileridir.
#[derive(Clone, Debug)]
pub struct TakvimSerisi {
    pub ad: Option<String>,
    /// Bağlı `calendar` bileşeninin sırası (`calendarIndex`).
    pub takvim_sırası: usize,
    /// Eski, bileşensiz kullanım için korunan yıl ve yerleşim alanları.
    pub yıl: i32,
    pub veri: Vec<VeriÖğesi>,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
    pub hücre_boşluğu: f32,
}

impl Default for TakvimSerisi {
    fn default() -> Self {
        TakvimSerisi {
            ad: None,
            takvim_sırası: 0,
            yıl: 2026,
            veri: Vec::new(),
            sol: Uzunluk::Yüzde(4.0),
            üst: Uzunluk::Piksel(60.0),
            genişlik: Uzunluk::Yüzde(92.0),
            yükseklik: Uzunluk::Piksel(190.0),
            hücre_boşluğu: 2.0,
        }
    }
}

impl TakvimSerisi {
    pub fn yeni(yıl: i32) -> Self {
        TakvimSerisi {
            yıl,
            ..Default::default()
        }
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = sıra;
        self
    }

    /// Veri: `[gün ms, değer]` dizileri.
    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
        self
    }
}

/// Tema nehri serisi (`themeRiver` + tek eksen karşılığı). Veri kayıtları
/// `(x, değer, katman adı)` üçlüleridir; katmanlar siluet taban çizgisi
/// etrafında yumuşak bantlar olarak yığılır.
#[derive(Clone, Debug)]
pub struct TemaNehriSerisi {
    pub ad: Option<String>,
    /// `(x, değer, katman)` kayıtları.
    pub veri: Vec<(f64, f64, String)>,
    /// Bağlı `singleAxis` bileşeninin sırası (`singleAxisIndex`).
    pub tek_eksen_sırası: usize,
    /// Eksenin dik yönündeki iç boşluk (`boundaryGap`).
    pub sınır_boşluğu: [Uzunluk; 2],
    /// Seri içindeki katmanların renk paleti (`series.color`).
    pub renkler: Vec<Dolgu>,
    pub öğe_stili: ÖğeStili,
    pub vurgu_öğe_stili: ÖğeStili,
    pub etiket: Etiket,
    pub vurgu_etiketi: EtiketYaması,
    /// `label.margin`; öntanımlı 4 px.
    pub etiket_boşluğu: f32,
    pub sessiz: bool,
    /// Eski bileşensiz kullanımın yerleşim alanı. Bir `singleAxis`
    /// bulunduğunda bu dört alan kullanılmaz.
    #[doc(hidden)]
    pub sol: Uzunluk,
    #[doc(hidden)]
    pub üst: Uzunluk,
    #[doc(hidden)]
    pub genişlik: Uzunluk,
    #[doc(hidden)]
    pub yükseklik: Uzunluk,
}

impl Default for TemaNehriSerisi {
    fn default() -> Self {
        TemaNehriSerisi {
            ad: None,
            veri: Vec::new(),
            tek_eksen_sırası: 0,
            sınır_boşluğu: [Uzunluk::Yüzde(10.0), Uzunluk::Yüzde(10.0)],
            renkler: Vec::new(),
            öğe_stili: ÖğeStili::default(),
            vurgu_öğe_stili: ÖğeStili::default(),
            etiket: Etiket::yeni()
                .göster(true)
                .konum(EtiketKonumu::Sol)
                .yazı(YazıStili::yeni().boyut(11.0)),
            vurgu_etiketi: EtiketYaması::yeni().göster(true),
            etiket_boşluğu: 4.0,
            sessiz: false,
            sol: Uzunluk::Yüzde(8.0),
            üst: Uzunluk::Piksel(70.0),
            genişlik: Uzunluk::Yüzde(84.0),
            yükseklik: Uzunluk::Yüzde(58.0),
        }
    }
}

impl TemaNehriSerisi {
    pub fn yeni() -> Self {
        TemaNehriSerisi::default()
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn tek_eksen_sırası(mut self, sıra: usize) -> Self {
        self.tek_eksen_sırası = sıra;
        self
    }

    pub fn sınır_boşluğu(
        mut self,
        başlangıç: impl Into<Uzunluk>,
        bitiş: impl Into<Uzunluk>,
    ) -> Self {
        self.sınır_boşluğu = [başlangıç.into(), bitiş.into()];
        self
    }

    pub fn renkler<D: Into<Dolgu>>(mut self, renkler: impl IntoIterator<Item = D>) -> Self {
        self.renkler = renkler.into_iter().map(Into::into).collect();
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn vurgu_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.vurgu_öğe_stili = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn vurgu_etiketi(mut self, etiket: EtiketYaması) -> Self {
        self.vurgu_etiketi = etiket;
        self
    }

    pub fn etiket_boşluğu(mut self, boşluk: f32) -> Self {
        self.etiket_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }

    /// Veri: `(x, değer, katman)` üçlüleri.
    pub fn veri<S: Into<String>>(mut self, veri: impl IntoIterator<Item = (f64, f64, S)>) -> Self {
        self.veri = veri
            .into_iter()
            .map(|(x, değer, katman)| (x, değer, katman.into()))
            .collect();
        self
    }
}

/// Tüm seri türlerini saran toplam tip (`series` dizisinin öğesi).
#[derive(Clone, Debug)]
pub enum Seri {
    Çizgi(ÇizgiSerisi),
    Sütun(SütunSerisi),
    Pasta(PastaSerisi),
    Saçılım(SaçılımSerisi),
    Mum(MumSerisi),
    Kutu(KutuSerisi),
    Isı(IsıHaritasıSerisi),
    Huni(HuniSerisi),
    GöstergeSaati(GöstergeSaatiSerisi),
    Radar(RadarSerisi),
    Özel(ÖzelSeri),
    AğaçHaritası(AğaçHaritasıSerisi),
    GüneşPatlaması(GüneşPatlamasıSerisi),
    Ağaç(AğaçSerisi),
    Sankey(SankeySerisi),
    Grafo(GrafoSerisi),
    Kiriş(KirişSerisi),
    Paralel(ParalelSerisi),
    Takvim(TakvimSerisi),
    TemaNehri(TemaNehriSerisi),
    Hatlar(HatlarSerisi),
}

impl Seri {
    pub fn ad(&self) -> Option<&str> {
        match self {
            Seri::Çizgi(s) => s.ad.as_deref(),
            Seri::Sütun(s) => s.ad.as_deref(),
            Seri::Pasta(s) => s.ad.as_deref(),
            Seri::Saçılım(s) => s.ad.as_deref(),
            Seri::Mum(s) => s.ad.as_deref(),
            Seri::Kutu(s) => s.ad.as_deref(),
            Seri::Isı(s) => s.ad.as_deref(),
            Seri::Huni(s) => s.ad.as_deref(),
            Seri::GöstergeSaati(s) => s.ad.as_deref(),
            Seri::Radar(s) => s.ad.as_deref(),
            Seri::Özel(s) => s.ad.as_deref(),
            Seri::AğaçHaritası(s) => s.ad.as_deref(),
            Seri::GüneşPatlaması(s) => s.ad.as_deref(),
            Seri::Ağaç(s) => s.ad.as_deref(),
            Seri::Sankey(s) => s.ad.as_deref(),
            Seri::Grafo(s) => s.ad.as_deref(),
            Seri::Kiriş(s) => s.ad.as_deref(),
            Seri::Paralel(s) => s.ad.as_deref(),
            Seri::Takvim(s) => s.ad.as_deref(),
            Seri::TemaNehri(s) => s.ad.as_deref(),
            Seri::Hatlar(s) => s.ad.as_deref(),
        }
    }

    /// Kutupsal koordinatta mı çizilir?
    pub fn kutupsal_mı(&self) -> bool {
        match self {
            Seri::Çizgi(s) => s.kutupsal,
            Seri::Sütun(s) => s.kutupsal,
            Seri::Saçılım(s) => {
                s.kutupsal && s.takvim_sırası.is_none() && s.tek_eksen_sırası.is_none()
            }
            Seri::Hatlar(s) => s.koordinat_sistemi == HatKoordinatSistemi::Kutupsal,
            _ => false,
        }
    }

    /// Kutupsal serinin bağlı olduğu `polarIndex`; kutupsal olmayan
    /// serilerde `None`.
    pub fn kutupsal_sırası(&self) -> Option<usize> {
        match self {
            Seri::Çizgi(s) if s.kutupsal => Some(s.kutupsal_sırası),
            Seri::Sütun(s) if s.kutupsal => Some(s.kutupsal_sırası),
            Seri::Saçılım(s)
                if s.kutupsal && s.takvim_sırası.is_none() && s.tek_eksen_sırası.is_none() =>
            {
                Some(s.kutupsal_sırası)
            }
            Seri::Hatlar(s) if s.koordinat_sistemi == HatKoordinatSistemi::Kutupsal => {
                Some(s.kutupsal_sırası)
            }
            _ => None,
        }
    }

    /// Kartezyen koordinat sisteminde mi çizilir?
    pub fn kartezyen_mi(&self) -> bool {
        if self.kutupsal_mı() {
            return false;
        }
        matches!(
            self,
            Seri::Çizgi(_)
                | Seri::Sütun(_)
                | Seri::Mum(_)
                | Seri::Kutu(_)
                | Seri::Hatlar(HatlarSerisi {
                    koordinat_sistemi: HatKoordinatSistemi::Kartezyen2B,
                    ..
                })
        ) || matches!(self, Seri::Isı(s) if s.matris_sırası.is_none())
            || matches!(self, Seri::Saçılım(s) if s.takvim_sırası.is_none() && s.tek_eksen_sırası.is_none() && s.matris_sırası.is_none())
            || matches!(self, Seri::Özel(s) if s.kartezyen_gerekli)
    }

    /// Tek eksenli koordinat sisteminde mi çizilir?
    pub fn tek_eksen_mi(&self) -> bool {
        matches!(self, Seri::Saçılım(s) if s.tek_eksen_sırası.is_some())
            || matches!(self, Seri::TemaNehri(_))
    }

    pub fn veri(&self) -> &[VeriÖğesi] {
        match self {
            Seri::Çizgi(s) => &s.veri,
            Seri::Sütun(s) => &s.veri,
            Seri::Pasta(s) => &s.veri,
            Seri::Saçılım(s) => &s.veri,
            Seri::Mum(s) => &s.veri,
            Seri::Kutu(s) => &s.veri,
            Seri::Isı(s) => &s.veri,
            Seri::Huni(s) => &s.veri,
            Seri::GöstergeSaati(s) => &s.veri,
            Seri::Radar(s) => &s.veri,
            Seri::Özel(s) => &s.veri,
            Seri::AğaçHaritası(_)
            | Seri::GüneşPatlaması(_)
            | Seri::Ağaç(_)
            | Seri::Sankey(_)
            | Seri::Grafo(_)
            | Seri::Kiriş(_) => &[],
            Seri::Paralel(s) => &s.veri,
            Seri::Takvim(s) => &s.veri,
            Seri::TemaNehri(_) => &[],
            Seri::Hatlar(_) => &[],
        }
    }

    /// Artımlı veri eklemeyi destekleyen serinin değiştirilebilir veri
    /// deposu. Hiyerarşik/bağ tabanlı seriler farklı veri modelleri
    /// kullandığından onlar için `None` döner.
    pub fn veri_mut(&mut self) -> Option<&mut Vec<VeriÖğesi>> {
        match self {
            Seri::Çizgi(s) => Some(&mut s.veri),
            Seri::Sütun(s) => Some(&mut s.veri),
            Seri::Pasta(s) => Some(&mut s.veri),
            Seri::Saçılım(s) => Some(&mut s.veri),
            Seri::Mum(s) => Some(&mut s.veri),
            Seri::Kutu(s) => Some(&mut s.veri),
            Seri::Isı(s) => Some(&mut s.veri),
            Seri::Huni(s) => Some(&mut s.veri),
            Seri::GöstergeSaati(s) => Some(&mut s.veri),
            Seri::Radar(s) => Some(&mut s.veri),
            Seri::Özel(s) => Some(&mut s.veri),
            Seri::Paralel(s) => Some(&mut s.veri),
            Seri::Takvim(s) => Some(&mut s.veri),
            Seri::AğaçHaritası(_)
            | Seri::GüneşPatlaması(_)
            | Seri::Ağaç(_)
            | Seri::Sankey(_)
            | Seri::Grafo(_)
            | Seri::Kiriş(_)
            | Seri::TemaNehri(_) => None,
            Seri::Hatlar(_) => None,
        }
    }

    /// Serinin bağlı olduğu eksen sıraları (kartezyen olmayanlarda öntanımlı).
    pub fn eksen_bağı(&self) -> EksenBağı {
        match self {
            Seri::Çizgi(s) => s.eksen_bağı,
            Seri::Sütun(s) => s.eksen_bağı,
            Seri::Saçılım(s) => s.eksen_bağı,
            Seri::Mum(s) => s.eksen_bağı,
            Seri::Kutu(s) => s.eksen_bağı,
            Seri::Isı(s) => s.eksen_bağı,
            Seri::Özel(s) => s.eksen_bağı,
            Seri::Hatlar(s) => s.eksen_bağı,
            _ => EksenBağı::default(),
        }
    }

    /// Serinin imleyicileri (kartezyen olmayanlarda `None`).
    pub fn imleyiciler(&self) -> Option<&İmleyiciler> {
        match self {
            Seri::Çizgi(s) => Some(&s.imleyiciler),
            Seri::Sütun(s) => Some(&s.imleyiciler),
            Seri::Saçılım(s) => Some(&s.imleyiciler),
            Seri::Mum(s) => Some(&s.imleyiciler),
            Seri::Kutu(s) => Some(&s.imleyiciler),
            Seri::Pasta(_)
            | Seri::Isı(_)
            | Seri::Huni(_)
            | Seri::GöstergeSaati(_)
            | Seri::Radar(_)
            | Seri::Özel(_)
            | Seri::AğaçHaritası(_)
            | Seri::GüneşPatlaması(_)
            | Seri::Ağaç(_)
            | Seri::Sankey(_)
            | Seri::Grafo(_)
            | Seri::Kiriş(_)
            | Seri::Paralel(_)
            | Seri::Takvim(_)
            | Seri::TemaNehri(_) => None,
            Seri::Hatlar(_) => None,
        }
    }

    /// Serinin açıkça verilmiş dolgusu (`itemStyle.color`).
    pub fn açık_renk(&self) -> Option<&Dolgu> {
        match self {
            Seri::Çizgi(s) => s.öğe_stili.renk.as_ref(),
            Seri::Sütun(s) => s.öğe_stili.renk.as_ref(),
            Seri::Pasta(s) => s.öğe_stili.renk.as_ref(),
            Seri::Saçılım(s) => s.öğe_stili.renk.as_ref(),
            Seri::GöstergeSaati(s) => s.öğe_stili.renk.as_ref(),
            Seri::Mum(_)
            | Seri::Kutu(_)
            | Seri::Isı(_)
            | Seri::Huni(_)
            | Seri::Radar(_)
            | Seri::Özel(_)
            | Seri::AğaçHaritası(_)
            | Seri::GüneşPatlaması(_)
            | Seri::Sankey(_)
            | Seri::Grafo(_)
            | Seri::Kiriş(_)
            | Seri::Paralel(_)
            | Seri::Takvim(_) => None,
            Seri::Ağaç(s) => s.öğe_stili.renk.as_ref(),
            Seri::TemaNehri(s) => s.öğe_stili.renk.as_ref(),
            Seri::Hatlar(_) => None,
        }
    }
}

impl From<ÇizgiSerisi> for Seri {
    fn from(s: ÇizgiSerisi) -> Seri {
        Seri::Çizgi(s)
    }
}

impl From<SütunSerisi> for Seri {
    fn from(s: SütunSerisi) -> Seri {
        Seri::Sütun(s)
    }
}

impl From<PastaSerisi> for Seri {
    fn from(s: PastaSerisi) -> Seri {
        Seri::Pasta(s)
    }
}

impl From<SaçılımSerisi> for Seri {
    fn from(s: SaçılımSerisi) -> Seri {
        Seri::Saçılım(s)
    }
}

impl From<MumSerisi> for Seri {
    fn from(s: MumSerisi) -> Seri {
        Seri::Mum(s)
    }
}

impl From<KutuSerisi> for Seri {
    fn from(s: KutuSerisi) -> Seri {
        Seri::Kutu(s)
    }
}

impl From<IsıHaritasıSerisi> for Seri {
    fn from(s: IsıHaritasıSerisi) -> Seri {
        Seri::Isı(s)
    }
}

impl From<HuniSerisi> for Seri {
    fn from(s: HuniSerisi) -> Seri {
        Seri::Huni(s)
    }
}

impl From<GöstergeSaatiSerisi> for Seri {
    fn from(s: GöstergeSaatiSerisi) -> Seri {
        Seri::GöstergeSaati(s)
    }
}

impl From<RadarSerisi> for Seri {
    fn from(s: RadarSerisi) -> Seri {
        Seri::Radar(s)
    }
}

impl From<ÖzelSeri> for Seri {
    fn from(s: ÖzelSeri) -> Seri {
        Seri::Özel(s)
    }
}

impl From<AğaçHaritasıSerisi> for Seri {
    fn from(s: AğaçHaritasıSerisi) -> Seri {
        Seri::AğaçHaritası(s)
    }
}

impl From<GüneşPatlamasıSerisi> for Seri {
    fn from(s: GüneşPatlamasıSerisi) -> Seri {
        Seri::GüneşPatlaması(s)
    }
}

impl From<AğaçSerisi> for Seri {
    fn from(s: AğaçSerisi) -> Seri {
        Seri::Ağaç(s)
    }
}

impl From<SankeySerisi> for Seri {
    fn from(s: SankeySerisi) -> Seri {
        Seri::Sankey(s)
    }
}

impl From<GrafoSerisi> for Seri {
    fn from(s: GrafoSerisi) -> Seri {
        Seri::Grafo(s)
    }
}

impl From<KirişSerisi> for Seri {
    fn from(s: KirişSerisi) -> Seri {
        Seri::Kiriş(s)
    }
}

impl From<ParalelSerisi> for Seri {
    fn from(s: ParalelSerisi) -> Seri {
        Seri::Paralel(s)
    }
}

impl From<TakvimSerisi> for Seri {
    fn from(s: TakvimSerisi) -> Seri {
        Seri::Takvim(s)
    }
}

impl From<TemaNehriSerisi> for Seri {
    fn from(s: TemaNehriSerisi) -> Seri {
        Seri::TemaNehri(s)
    }
}

impl From<HatlarSerisi> for Seri {
    fn from(s: HatlarSerisi) -> Seri {
        Seri::Hatlar(s)
    }
}
