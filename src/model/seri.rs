//! Seri seçenekleri — ECharts'taki `series` tanımlarının karşılığı
//! (`echarts/src/chart/*/…SeriesModel`).

use std::fmt;
use std::sync::Arc;

use crate::cizim::{SvgYolHatası, Yol};
use crate::koordinat::Dikdörtgen;
use crate::model::Uzunluk;
use crate::model::deger::{VeriÖğesi, veri_listesi};
pub use crate::model::hatlar::{
    HatEfekti, HatKoordinatSistemi, HatKoordinatı, HatNoktası, HatVerisi, HatlarSerisi,
};
use crate::model::imleyici::{İmAlanı, İmNoktası, İmleyiciler, İmÇizgisi};
use crate::model::stil::{AlanStili, Etiket, ÇizgiStili, ÖğeStili};
use crate::model::veri_kumesi::SeriYerleşimi;
use crate::renk::Dolgu;

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
    /// Veri kümesi eşlemesi: `(x boyutu, y boyutu)` (`encode.x/y`).
    pub eşleme: Option<(String, String)>,
    /// Bağlı `dataset` dizisi sırası (`datasetIndex`).
    pub veri_kümesi_sırası: usize,
    /// `seriesLayoutBy`.
    pub seri_yerleşimi: SeriYerleşimi,
}

impl Default for SütunSerisi {
    fn default() -> Self {
        SütunSerisi {
            ad: None,
            veri: Vec::new(),
            yığın: None,
            genişlik: None,
            en_çok_genişlik: None,
            en_az_genişlik: None,
            sütun_boşluğu: None,
            kategori_boşluğu: None,
            öğe_stili: ÖğeStili::default(),
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
            eşleme: None,
            veri_kümesi_sırası: 0,
            seri_yerleşimi: SeriYerleşimi::Sütun,
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

    /// Seriyi kutupsal koordinata bağlar (`coordinateSystem: 'polar'`).
    pub fn kutupsal(mut self, açık: bool) -> Self {
        self.kutupsal = açık;
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
        self
    }

    /// Pasta merkezini takvimdeki bir tarihe bağlar. Açık bir
    /// `calendarIndex` verilmediyse ECharts gibi sıfırıncı takvim seçilir.
    pub fn takvim_merkezi(mut self, tarih_ms: f64) -> Self {
        self.takvim_sırası.get_or_insert(0);
        self.takvim_merkez_tarihi = Some(tarih_ms);
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
    /// Gövde genişliğinin bant genişliğine oranı. ECharts, açık bir
    /// `barWidth` verilmediğinde bant genişliğinin yarısını kullanır.
    pub gövde_oranı: f32,
    pub kenarlık_kalınlığı: f32,
    pub imleyiciler: İmleyiciler,
    /// Bağlı eksenler (`xAxisIndex`/`yAxisIndex`).
    pub eksen_bağı: EksenBağı,
}

impl Default for MumSerisi {
    fn default() -> Self {
        MumSerisi {
            ad: None,
            veri: Vec::new(),
            // ECharts v5 öntanımlıları: color '#eb5454', color0 '#47b262'.
            yükselen_renk: crate::renk::Renk::onaltılık(0xeb5454),
            düşen_renk: crate::renk::Renk::onaltılık(0x47b262),
            gövde_oranı: 0.5,
            kenarlık_kalınlığı: 1.0,
            imleyiciler: İmleyiciler::default(),
            eksen_bağı: EksenBağı::default(),
        }
    }
}

impl MumSerisi {
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

    pub fn im_çizgisi(mut self, im: İmÇizgisi) -> Self {
        self.imleyiciler.çizgi = Some(im);
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

/// Saçılım serisi (`series-scatter`).
#[derive(Clone, Debug)]
pub struct SaçılımSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub sembol: Sembol,
    pub sembol_boyutu: SembolBoyutu,
    /// Özel SVG yolunun en/boy oranını korur (`symbolKeepAspect`).
    pub sembol_oranını_koru: bool,
    pub öğe_stili: ÖğeStili,
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
    /// Takvim koordinatına bağlıysa `calendarIndex`; `None`, kartezyen ya da
    /// başka bir koordinat sistemindeki saçılım demektir.
    pub takvim_sırası: Option<usize>,
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
}

impl Default for SaçılımSerisi {
    fn default() -> Self {
        SaçılımSerisi {
            ad: None,
            veri: Vec::new(),
            sembol: Sembol::Daire,
            sembol_boyutu: SembolBoyutu::Sabit(10.0),
            sembol_oranını_koru: false,
            öğe_stili: ÖğeStili::default(),
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
            takvim_sırası: None,
            tek_eksen_sırası: None,
            z_seviyesi: 0,
            sessiz: false,
            veriye_göre_renk: false,
            eşleme: None,
            veri_kümesi_sırası: 0,
            seri_yerleşimi: SeriYerleşimi::Sütun,
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
        }
        self
    }

    /// Seriyi takvim koordinatına bağlar (`coordinateSystem: 'calendar'`,
    /// `calendarIndex`).
    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = Some(sıra);
        self.tek_eksen_sırası = None;
        self.kutupsal = false;
        self
    }

    /// Seriyi tek eksenli koordinata bağlar (`coordinateSystem:
    /// 'singleAxis'`, `singleAxisIndex`).
    pub fn tek_eksen_sırası(mut self, sıra: usize) -> Self {
        self.tek_eksen_sırası = Some(sıra);
        self.takvim_sırası = None;
        self.kutupsal = false;
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
        self.kutupsal = false;
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

/// Huni serisi (`series-funnel`).
#[derive(Clone, Debug)]
pub struct HuniSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
    pub sıralama: HuniSıralaması,
    /// Dilimler arası dikey boşluk (`gap`).
    pub dilim_boşluğu: f32,
    /// En dar dilimin genişliği (`minSize`).
    pub en_az_genişlik: Uzunluk,
    /// En geniş dilimin genişliği (`maxSize`).
    pub en_çok_genişlik: Uzunluk,
    pub öğe_stili: ÖğeStili,
    pub etiket: Etiket,
}

impl Default for HuniSerisi {
    fn default() -> Self {
        HuniSerisi {
            ad: None,
            veri: Vec::new(),
            sol: Uzunluk::Yüzde(10.0),
            üst: Uzunluk::Piksel(60.0),
            genişlik: Uzunluk::Yüzde(80.0),
            yükseklik: Uzunluk::Yüzde(70.0),
            sıralama: HuniSıralaması::Azalan,
            dilim_boşluğu: 2.0,
            en_az_genişlik: Uzunluk::Yüzde(0.0),
            en_çok_genişlik: Uzunluk::Yüzde(100.0),
            öğe_stili: ÖğeStili {
                kenarlık_rengi: Some(crate::renk::Renk::BEYAZ),
                kenarlık_kalınlığı: 1.0,
                ..Default::default()
            },
            etiket: Etiket {
                göster: true,
                konum: crate::model::stil::EtiketKonumu::İç,
                ..Default::default()
            },
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
        self
    }

    pub fn sıralama(mut self, sıralama: HuniSıralaması) -> Self {
        self.sıralama = sıralama;
        self
    }

    pub fn dilim_boşluğu(mut self, boşluk: f32) -> Self {
        self.dilim_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }
}

/// Gösterge saati serisi (`series-gauge`). Tek değerli veri beklenir.
#[derive(Clone, Debug)]
pub struct GöstergeSaatiSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub en_az: f64,
    pub en_çok: f64,
    /// Başlangıç açısı, derece (`startAngle`, öntanımlı 225).
    pub başlangıç_açısı: f32,
    /// Bitiş açısı, derece (`endAngle`, öntanımlı -45).
    pub bitiş_açısı: f32,
    pub merkez: (Uzunluk, Uzunluk),
    pub yarıçap: Uzunluk,
    /// Renk bantları: `(bant sonu oranı 0..=1, renk)` — `axisLine.lineStyle.color`.
    pub renk_bantları: Vec<(f32, crate::renk::Renk)>,
    /// Yay şeridinin kalınlığı (`axisLine.lineStyle.width`).
    pub şerit_kalınlığı: f32,
    pub bölme_sayısı: usize,
    pub çentik_uzunluğu: f32,
    pub etiketleri_göster: bool,
    pub etiket_boyutu: f32,
    /// İbre uzunluğu, yarıçapa oranla ya da piksel (`pointer.length`).
    pub ibre_uzunluğu: Uzunluk,
    /// Değer yazısı (`detail.show`).
    pub değeri_göster: bool,
    pub değer_boyutu: f32,
    pub değer_biçimleyici: Option<crate::model::stil::Biçimleyici>,
}

impl Default for GöstergeSaatiSerisi {
    fn default() -> Self {
        GöstergeSaatiSerisi {
            ad: None,
            veri: Vec::new(),
            en_az: 0.0,
            en_çok: 100.0,
            başlangıç_açısı: 225.0,
            bitiş_açısı: -45.0,
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(55.0)),
            yarıçap: Uzunluk::Yüzde(75.0),
            renk_bantları: vec![
                (0.3, crate::renk::Renk::onaltılık(0x67e0e3)),
                (0.7, crate::renk::Renk::onaltılık(0x37a2da)),
                (1.0, crate::renk::Renk::onaltılık(0xfd666d)),
            ],
            şerit_kalınlığı: 18.0,
            bölme_sayısı: 10,
            çentik_uzunluğu: 8.0,
            etiketleri_göster: true,
            etiket_boyutu: crate::tema::YAZI_KÜÇÜK,
            ibre_uzunluğu: Uzunluk::Yüzde(60.0),
            değeri_göster: true,
            değer_boyutu: 24.0,
            değer_biçimleyici: None,
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
        self
    }

    pub fn aralık(mut self, en_az: f64, en_çok: f64) -> Self {
        self.en_az = en_az;
        self.en_çok = en_çok;
        self
    }

    pub fn renk_bantları<R: Into<crate::renk::Renk>>(
        mut self,
        bantlar: impl IntoIterator<Item = (f32, R)>,
    ) -> Self {
        self.renk_bantları = bantlar.into_iter().map(|(o, r)| (o, r.into())).collect();
        self
    }

    pub fn değer_biçimleyici(mut self, b: impl Into<crate::model::stil::Biçimleyici>) -> Self {
        self.değer_biçimleyici = Some(b.into());
        self
    }
}

/// Radar serisi (`series-radar`). Her veri öğesi, koordinattaki gösterge
/// sayısı kadar değerli bir dizidir; öğe adı göstergede (legend) listelenir.
#[derive(Clone, Debug)]
pub struct RadarSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub çizgi_stili: ÇizgiStili,
    pub alan_stili: Option<AlanStili>,
    pub sembol: Sembol,
    pub sembol_boyutu: f32,
    pub sembol_göster: bool,
}

impl Default for RadarSerisi {
    fn default() -> Self {
        RadarSerisi {
            ad: None,
            veri: Vec::new(),
            çizgi_stili: ÇizgiStili::default(),
            alan_stili: None,
            sembol: Sembol::Daire,
            sembol_boyutu: 6.0,
            sembol_göster: true,
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
}

/// Özel seri çizim bağlamı: kullanıcının çizim işlevine geçirilir.
pub struct ÖzelBağlam<'a> {
    /// Etkin koordinat sisteminin çizim alanı (koordinat yoksa tuvalin tamamı).
    pub alan: crate::koordinat::Dikdörtgen,
    /// Kartezyen koordinat sistemi (eksenler kuruluysa).
    pub kartezyen: Option<&'a crate::koordinat::Kartezyen2B>,
    /// Takvim koordinat sistemi (`coordinateSystem: 'calendar'` ise).
    pub takvim: Option<&'a crate::koordinat::TakvimYerleşimi>,
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
        }
        self
    }

    /// Seriyi bir takvim koordinatına bağlar (`coordinateSystem: 'calendar'`
    /// ve `calendarIndex`). Çizim bağlamındaki `takvim` alanı bu yerleşimi
    /// taşır; tarihleri piksele çevirmek için `veriden_noktaya` kullanılabilir.
    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = Some(sıra);
        self.kartezyen_gerekli = false;
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

/// Ağaç serisi (`series-tree`): soldan sağa düzenli yerleşim.
#[derive(Clone, Debug)]
pub struct AğaçSerisi {
    pub ad: Option<String>,
    pub kökler: Vec<crate::model::agac::AğaçDüğümü>,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
    pub sembol_boyutu: f32,
}

impl Default for AğaçSerisi {
    fn default() -> Self {
        AğaçSerisi {
            ad: None,
            kökler: Vec::new(),
            sol: Uzunluk::Yüzde(10.0),
            üst: Uzunluk::Piksel(60.0),
            genişlik: Uzunluk::Yüzde(72.0),
            yükseklik: Uzunluk::Yüzde(78.0),
            sembol_boyutu: 9.0,
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

    pub fn kökler(
        mut self,
        kökler: impl IntoIterator<Item = crate::model::agac::AğaçDüğümü>,
    ) -> Self {
        self.kökler = kökler.into_iter().collect();
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
        self
    }
}

/// Grafo yerleşimi (`graph.layout`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoYerleşimi {
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
    /// Takvim koordinatına bağlıysa `calendarIndex`.
    pub takvim_sırası: Option<usize>,
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
            takvim_sırası: None,
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
        self
    }

    pub fn etiket_eşiği(mut self, eşik: f32) -> Self {
        self.etiket_eşiği = eşik.max(0.0);
        self
    }

    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = Some(sıra);
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
    pub boyutlar: Vec<ParalelBoyut>,
    /// Her öğe, boyut sayısı kadar değerli bir dizidir.
    pub veri: Vec<VeriÖğesi>,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
    pub çizgi_stili: ÇizgiStili,
}

impl Default for ParalelSerisi {
    fn default() -> Self {
        ParalelSerisi {
            ad: None,
            boyutlar: Vec::new(),
            veri: Vec::new(),
            sol: Uzunluk::Yüzde(8.0),
            üst: Uzunluk::Piksel(70.0),
            genişlik: Uzunluk::Yüzde(84.0),
            yükseklik: Uzunluk::Yüzde(70.0),
            çizgi_stili: ÇizgiStili {
                kalınlık: 1.0,
                ..Default::default()
            },
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

    pub fn boyutlar<S: Into<String>>(mut self, boyutlar: impl IntoIterator<Item = S>) -> Self {
        self.boyutlar = boyutlar.into_iter().map(ParalelBoyut::yeni).collect();
        self
    }

    pub fn veri<T: Into<VeriÖğesi>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri_listesi(veri);
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
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
}

impl Default for TemaNehriSerisi {
    fn default() -> Self {
        TemaNehriSerisi {
            ad: None,
            veri: Vec::new(),
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
                | Seri::Isı(_)
                | Seri::Hatlar(HatlarSerisi {
                    koordinat_sistemi: HatKoordinatSistemi::Kartezyen2B,
                    ..
                })
        ) || matches!(self, Seri::Saçılım(s) if s.takvim_sırası.is_none() && s.tek_eksen_sırası.is_none())
            || matches!(self, Seri::Özel(s) if s.kartezyen_gerekli)
    }

    /// Tek eksenli koordinat sisteminde mi çizilir?
    pub fn tek_eksen_mi(&self) -> bool {
        matches!(self, Seri::Saçılım(s) if s.tek_eksen_sırası.is_some())
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
            Seri::Mum(_)
            | Seri::Kutu(_)
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
