//! Seri seçenekleri — ECharts'taki `series` tanımlarının karşılığı
//! (`echarts/src/chart/*/…SeriesModel`).

use std::fmt;
use std::sync::Arc;

use crate::model::deger::{veri_listesi, VeriÖğesi};
use crate::model::stil::{AlanStili, Etiket, ÇizgiStili, ÖğeStili};
use crate::model::Uzunluk;
use crate::renk::Dolgu;

/// Sembol biçimi (`symbol`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Sembol {
    /// ECharts çizgi serisinin öntanımlısı (`'emptyCircle'`).
    #[default]
    İçiBoşDaire,
    Daire,
    Kare,
    Üçgen,
    Elmas,
    Yok,
}

/// Basamaklı çizgi kipi (`step`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Basamak {
    Baş,
    Orta,
    Son,
}

/// Pasta grafiklerinde gül (Nightingale) kipi (`roseType`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GülTürü {
    /// Yarıçap değerle orantılı (`'radius'`).
    Yarıçap,
    /// Alan değerle orantılı (`'area'`).
    Alan,
}

/// Saçılım sembol boyutu: sabit ya da veriye bağlı işlev (`symbolSize`).
#[derive(Clone)]
pub enum SembolBoyutu {
    Sabit(f32),
    İşlev(Arc<dyn Fn(&VeriÖğesi) -> f32 + Send + Sync>),
}

impl SembolBoyutu {
    pub fn çöz(&self, öğe: &VeriÖğesi) -> f32 {
        match self {
            SembolBoyutu::Sabit(b) => *b,
            SembolBoyutu::İşlev(f) => f(öğe),
        }
    }
}

impl fmt::Debug for SembolBoyutu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SembolBoyutu::Sabit(b) => f.debug_tuple("Sabit").field(b).finish(),
            SembolBoyutu::İşlev(_) => f.write_str("İşlev(..)"),
        }
    }
}

impl From<f32> for SembolBoyutu {
    fn from(b: f32) -> Self {
        SembolBoyutu::Sabit(b)
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
    pub çizgi_stili: ÇizgiStili,
    pub öğe_stili: ÖğeStili,
    pub alan_stili: Option<AlanStili>,
    pub yığın: Option<String>,
    pub boşları_bağla: bool,
    pub etiket: Etiket,
}

impl Default for ÇizgiSerisi {
    fn default() -> Self {
        ÇizgiSerisi {
            ad: None,
            veri: Vec::new(),
            yumuşaklık: 0.0,
            basamak: None,
            sembol: Sembol::İçiBoşDaire,
            sembol_boyutu: 4.0,
            sembol_göster: true,
            çizgi_stili: ÇizgiStili::default(),
            öğe_stili: ÖğeStili::default(),
            alan_stili: None,
            yığın: None,
            boşları_bağla: false,
            etiket: Etiket::default(),
        }
    }
}

impl ÇizgiSerisi {
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

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
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
    pub etiket: Etiket,
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
            etiket: Etiket::default(),
        }
    }
}

impl SütunSerisi {
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

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }
}

/// Pasta dilimi etiket çizgisi (`labelLine`).
#[derive(Clone, PartialEq, Debug)]
pub struct EtiketÇizgisi {
    pub göster: bool,
    /// Dilimden dışa uzanan ilk parça (`length`).
    pub uzunluk1: f32,
    /// Yatay ikinci parça (`length2`).
    pub uzunluk2: f32,
}

impl Default for EtiketÇizgisi {
    fn default() -> Self {
        EtiketÇizgisi { göster: true, uzunluk1: 15.0, uzunluk2: 15.0 }
    }
}

/// Pasta serisi (`series-pie`).
#[derive(Clone, Debug)]
pub struct PastaSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    /// `(iç, dış)` yarıçap; ECharts öntanımlısı `[0, '75%']`.
    pub yarıçap: (Uzunluk, Uzunluk),
    /// Merkez `(x, y)`; öntanımlı `('50%', '50%')`.
    pub merkez: (Uzunluk, Uzunluk),
    /// Derece cinsinden başlangıç açısı (`startAngle`, öntanımlı 90).
    pub başlangıç_açısı: f32,
    pub saat_yönünde: bool,
    pub gül_türü: Option<GülTürü>,
    pub öğe_stili: ÖğeStili,
    pub etiket: Etiket,
    pub etiket_çizgisi: EtiketÇizgisi,
}

impl Default for PastaSerisi {
    fn default() -> Self {
        PastaSerisi {
            ad: None,
            veri: Vec::new(),
            yarıçap: (Uzunluk::Yüzde(0.0), Uzunluk::Yüzde(75.0)),
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(50.0)),
            başlangıç_açısı: 90.0,
            saat_yönünde: true,
            gül_türü: None,
            öğe_stili: ÖğeStili::default(),
            etiket: Etiket { göster: true, konum: crate::model::stil::EtiketKonumu::Dış, ..Default::default() },
            etiket_çizgisi: EtiketÇizgisi::default(),
        }
    }
}

impl PastaSerisi {
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

    pub fn saat_yönünde(mut self, saat_yönünde: bool) -> Self {
        self.saat_yönünde = saat_yönünde;
        self
    }

    pub fn gül_türü(mut self, tür: GülTürü) -> Self {
        self.gül_türü = Some(tür);
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
}

/// Saçılım serisi (`series-scatter`).
#[derive(Clone, Debug)]
pub struct SaçılımSerisi {
    pub ad: Option<String>,
    pub veri: Vec<VeriÖğesi>,
    pub sembol: Sembol,
    pub sembol_boyutu: SembolBoyutu,
    pub öğe_stili: ÖğeStili,
    pub etiket: Etiket,
}

impl Default for SaçılımSerisi {
    fn default() -> Self {
        SaçılımSerisi {
            ad: None,
            veri: Vec::new(),
            sembol: Sembol::Daire,
            sembol_boyutu: SembolBoyutu::Sabit(10.0),
            öğe_stili: ÖğeStili::default(),
            etiket: Etiket::default(),
        }
    }
}

impl SaçılımSerisi {
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

    pub fn sembol(mut self, sembol: Sembol) -> Self {
        self.sembol = sembol;
        self
    }

    pub fn sembol_boyutu(mut self, boyut: impl Into<SembolBoyutu>) -> Self {
        self.sembol_boyutu = boyut.into();
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

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
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
}

impl Seri {
    pub fn ad(&self) -> Option<&str> {
        match self {
            Seri::Çizgi(s) => s.ad.as_deref(),
            Seri::Sütun(s) => s.ad.as_deref(),
            Seri::Pasta(s) => s.ad.as_deref(),
            Seri::Saçılım(s) => s.ad.as_deref(),
        }
    }

    /// Kartezyen koordinat sisteminde mi çizilir?
    pub fn kartezyen_mi(&self) -> bool {
        matches!(self, Seri::Çizgi(_) | Seri::Sütun(_) | Seri::Saçılım(_))
    }

    pub fn veri(&self) -> &[VeriÖğesi] {
        match self {
            Seri::Çizgi(s) => &s.veri,
            Seri::Sütun(s) => &s.veri,
            Seri::Pasta(s) => &s.veri,
            Seri::Saçılım(s) => &s.veri,
        }
    }

    /// Serinin açıkça verilmiş dolgusu (`itemStyle.color`).
    pub fn açık_renk(&self) -> Option<&Dolgu> {
        match self {
            Seri::Çizgi(s) => s.öğe_stili.renk.as_ref(),
            Seri::Sütun(s) => s.öğe_stili.renk.as_ref(),
            Seri::Pasta(s) => s.öğe_stili.renk.as_ref(),
            Seri::Saçılım(s) => s.öğe_stili.renk.as_ref(),
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
