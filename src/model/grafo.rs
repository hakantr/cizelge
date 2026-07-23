//! İlişki ağı (`series.graph`) seçenek modeli.
//!
//! Bu yüzey kilitli ECharts `GraphSeries.ts` modelini izler. Düğüm,
//! kategori ve bağ düzeyindeki seçenekler seri varsayılanlarının üstüne
//! yama olarak uygulanır; `data/nodes` ile `links/edges` aynı depoyu
//! paylaşır. Geo dalı proje kapsamı dışında tutulur, diğer çekirdek
//! koordinat sistemleri modelde açıkça temsil edilir.

use crate::model::Uzunluk;
use crate::model::bilesen::İpucu;
use crate::model::deger::VeriDeğeri;
use crate::model::matris::MatrisAralığı;
use crate::model::seri::{EksenBağı, Sembol};
use crate::model::stil::{Etiket, EtiketYaması, ÇizgiTürü};
use crate::renk::{Dolgu, Renk};

/// Graph serisinin bağlı olduğu çekirdek koordinat sistemi.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoKoordinatSistemi {
    #[default]
    Görünüm,
    Kartezyen2B,
    Kutupsal,
    TekEksen,
    Takvim,
    Matris,
}

/// Graph görünümünde izin verilen gezinme (`roam`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoGezinmesi {
    #[default]
    Kapalı,
    /// `true`: kaydırma ve ölçekleme.
    Açık,
    /// `'pan'` / `'move'`.
    Kaydır,
    /// `'zoom'` / `'scale'`.
    Ölçekle,
}

impl GrafoGezinmesi {
    pub fn kaydırılabilir(self) -> bool {
        matches!(self, Self::Açık | Self::Kaydır)
    }

    pub fn ölçeklenebilir(self) -> bool {
        matches!(self, Self::Açık | Self::Ölçekle)
    }
}

/// İşaretçi gezinmesinin başlayabileceği alan (`roamTrigger`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoGezinmeTetikleyicisi {
    /// Seri görünümünün kendi sınır dikdörtgeni (`null` / `'selfRect'`).
    #[default]
    KendiAlanı,
    /// Bütün tuval (`'global'`).
    Global,
}

/// Açık genişlik/yükseklik kutusunda veri en-boy oranını koruma kipi.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoEnBoyKoruma {
    #[default]
    Kapalı,
    İçer,
    Kapla,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoEnBoyYatayHizası {
    Sol,
    #[default]
    Orta,
    Sağ,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoEnBoyDikeyHizası {
    Üst,
    #[default]
    Orta,
    Alt,
}

/// Graph yerleşim algoritması (`layout`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoYerleşimi {
    /// Açık x/y veya bağlı koordinat sistemi (`null` / `none`).
    Yok,
    /// ECharts'ın d3 kökenli kuvvet çözücüsü.
    #[default]
    Kuvvet,
    /// Sembol boyutlarına göre çember üzerinde yerleşim.
    Dairesel,
}

/// Kuvvet başlangıç yerleşimi (`force.initLayout`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GrafoKuvvetBaşlangıcı {
    Yok,
    Dairesel,
}

/// Tek değer ya da veri kapsamı boyunca doğrusal eşlenen aralık.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct GrafoAralığı(pub f32, pub f32);

impl GrafoAralığı {
    pub fn yeni(en_az: f32, en_çok: f32) -> Self {
        Self(en_az, en_çok)
    }

    pub fn tek(değer: f32) -> Self {
        Self(değer, değer)
    }
}

impl From<f32> for GrafoAralığı {
    fn from(değer: f32) -> Self {
        Self::tek(değer)
    }
}

impl From<i32> for GrafoAralığı {
    fn from(değer: i32) -> Self {
        Self::tek(değer as f32)
    }
}

impl From<[f32; 2]> for GrafoAralığı {
    fn from([en_az, en_çok]: [f32; 2]) -> Self {
        Self(en_az, en_çok)
    }
}

/// ECharts `force` alt seçeneği.
#[derive(Clone, PartialEq, Debug)]
pub struct GrafoKuvveti {
    pub başlangıç_yerleşimi: Option<GrafoKuvvetBaşlangıcı>,
    pub itme: GrafoAralığı,
    pub yerçekimi: f32,
    pub sürtünme: f32,
    pub kenar_uzunluğu: GrafoAralığı,
    pub yerleşim_animasyonu: bool,
}

impl Default for GrafoKuvveti {
    fn default() -> Self {
        Self {
            başlangıç_yerleşimi: None,
            itme: GrafoAralığı(0.0, 50.0),
            yerçekimi: 0.1,
            sürtünme: 0.6,
            kenar_uzunluğu: GrafoAralığı::tek(30.0),
            yerleşim_animasyonu: true,
        }
    }
}

impl GrafoKuvveti {
    pub fn yeni() -> Self {
        Self::default()
    }
    pub fn başlangıç_yerleşimi(mut self, değer: GrafoKuvvetBaşlangıcı) -> Self {
        self.başlangıç_yerleşimi = Some(değer);
        self
    }
    pub fn itme(mut self, değer: impl Into<GrafoAralığı>) -> Self {
        self.itme = değer.into();
        self
    }
    pub fn yerçekimi(mut self, değer: f32) -> Self {
        if değer.is_finite() {
            self.yerçekimi = değer;
        }
        self
    }
    pub fn sürtünme(mut self, değer: f32) -> Self {
        if değer.is_finite() {
            self.sürtünme = değer.max(0.0);
        }
        self
    }
    pub fn kenar_uzunluğu(mut self, değer: impl Into<GrafoAralığı>) -> Self {
        self.kenar_uzunluğu = değer.into();
        self
    }
    pub fn yerleşim_animasyonu(mut self, açık: bool) -> Self {
        self.yerleşim_animasyonu = açık;
        self
    }
}

/// `circular` alt seçeneği.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct GrafoDaireselAyarı {
    pub etiketi_döndür: bool,
}

impl GrafoDaireselAyarı {
    pub fn yeni() -> Self {
        Self::default()
    }
    pub fn etiketi_döndür(mut self, açık: bool) -> Self {
        self.etiketi_döndür = açık;
        self
    }
}

/// Çoklu bağların otomatik eğrilik dizisi.
#[derive(Clone, PartialEq, Debug)]
pub enum GrafoOtomatikEğrilik {
    Kapalı,
    Uzunluk(usize),
    Değerler(Vec<f32>),
}

/// `lineStyle.color`: düz renk veya uç düğümden türetilen renk.
#[derive(Clone, PartialEq, Debug)]
pub enum GrafoKenarBoyası {
    Renk(Renk),
    Kaynak,
    Hedef,
}

impl From<Renk> for GrafoKenarBoyası {
    fn from(değer: Renk) -> Self {
        Self::Renk(değer)
    }
}

impl From<u32> for GrafoKenarBoyası {
    fn from(değer: u32) -> Self {
        Self::Renk(Renk::from(değer))
    }
}

impl From<&str> for GrafoKenarBoyası {
    fn from(değer: &str) -> Self {
        match değer {
            "source" | "kaynak" => Self::Kaynak,
            "target" | "hedef" => Self::Hedef,
            diğer => Self::Renk(Renk::from(diğer)),
        }
    }
}

impl From<String> for GrafoKenarBoyası {
    fn from(değer: String) -> Self {
        Self::from(değer.as_str())
    }
}

/// Seri/kategori/düğüm `itemStyle` yaması.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct GrafoÖğeStili {
    pub renk: Option<Dolgu>,
    pub kenarlık_rengi: Option<Renk>,
    pub kenarlık_kalınlığı: Option<f32>,
    pub kenarlık_türü: Option<ÇizgiTürü>,
    pub opaklık: Option<f32>,
    pub gölge_bulanıklığı: Option<f32>,
    pub gölge_rengi: Option<Renk>,
    pub gölge_kayması: Option<(f32, f32)>,
}

impl GrafoÖğeStili {
    pub fn yeni() -> Self {
        Self::default()
    }
    pub fn renk(mut self, değer: impl Into<Dolgu>) -> Self {
        self.renk = Some(değer.into());
        self
    }
    pub fn kenarlık_rengi(mut self, değer: impl Into<Renk>) -> Self {
        self.kenarlık_rengi = Some(değer.into());
        self
    }
    pub fn kenarlık_kalınlığı(mut self, değer: f32) -> Self {
        self.kenarlık_kalınlığı = değer.is_finite().then(|| değer.max(0.0));
        self
    }
    pub fn kenarlık_türü(mut self, değer: ÇizgiTürü) -> Self {
        self.kenarlık_türü = Some(değer);
        self
    }
    pub fn opaklık(mut self, değer: f32) -> Self {
        self.opaklık = değer.is_finite().then(|| değer.clamp(0.0, 1.0));
        self
    }
    pub fn gölge_bulanıklığı(mut self, değer: f32) -> Self {
        self.gölge_bulanıklığı = değer.is_finite().then(|| değer.max(0.0));
        self
    }
    pub fn gölge_rengi(mut self, değer: impl Into<Renk>) -> Self {
        self.gölge_rengi = Some(değer.into());
        self
    }
    pub fn gölge_kayması(mut self, x: f32, y: f32) -> Self {
        if x.is_finite() && y.is_finite() {
            self.gölge_kayması = Some((x, y));
        }
        self
    }
}

/// Seri/bağ `lineStyle` yaması.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct GrafoÇizgiStili {
    pub renk: Option<GrafoKenarBoyası>,
    pub kalınlık: Option<f32>,
    pub tür: Option<ÇizgiTürü>,
    pub opaklık: Option<f32>,
    pub eğrilik: Option<f32>,
    pub gölge_bulanıklığı: Option<f32>,
    pub gölge_rengi: Option<Renk>,
    pub gölge_kayması: Option<(f32, f32)>,
}

impl GrafoÇizgiStili {
    pub fn yeni() -> Self {
        Self::default()
    }
    pub fn seri_varsayılanı() -> Self {
        Self {
            renk: Some(GrafoKenarBoyası::Renk(crate::tema::nötr_50())),
            kalınlık: Some(1.0),
            tür: Some(ÇizgiTürü::Düz),
            opaklık: Some(0.5),
            eğrilik: Some(0.0),
            gölge_bulanıklığı: Some(0.0),
            gölge_kayması: Some((0.0, 0.0)),
            ..Self::default()
        }
    }
    pub fn renk(mut self, değer: impl Into<GrafoKenarBoyası>) -> Self {
        self.renk = Some(değer.into());
        self
    }
    pub fn kalınlık(mut self, değer: f32) -> Self {
        self.kalınlık = değer.is_finite().then(|| değer.max(0.0));
        self
    }
    pub fn tür(mut self, değer: ÇizgiTürü) -> Self {
        self.tür = Some(değer);
        self
    }
    pub fn opaklık(mut self, değer: f32) -> Self {
        self.opaklık = değer.is_finite().then(|| değer.clamp(0.0, 1.0));
        self
    }
    pub fn eğrilik(mut self, değer: f32) -> Self {
        self.eğrilik = değer.is_finite().then_some(değer);
        self
    }
    pub fn gölge_bulanıklığı(mut self, değer: f32) -> Self {
        self.gölge_bulanıklığı = değer.is_finite().then(|| değer.max(0.0));
        self
    }
    pub fn gölge_rengi(mut self, değer: impl Into<Renk>) -> Self {
        self.gölge_rengi = Some(değer.into());
        self
    }
    pub fn gölge_kayması(mut self, x: f32, y: f32) -> Self {
        if x.is_finite() && y.is_finite() {
            self.gölge_kayması = Some((x, y));
        }
        self
    }
}

/// Graph vurgu odağı.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GrafoVurguOdağı {
    #[default]
    Yok,
    Kendisi,
    Komşuluk,
    Seri,
}

/// Normal dışındaki düğüm/bağ durum katmanı.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct GrafoDurumu {
    pub öğe_stili: Option<GrafoÖğeStili>,
    pub çizgi_stili: Option<GrafoÇizgiStili>,
    pub etiket: Option<EtiketYaması>,
    pub kenar_etiketi: Option<EtiketYaması>,
    pub odak: Option<GrafoVurguOdağı>,
    pub ölçek: Option<f32>,
    pub devre_dışı: Option<bool>,
}

impl GrafoDurumu {
    pub fn yeni() -> Self {
        Self::default()
    }
    pub fn öğe_stili(mut self, değer: GrafoÖğeStili) -> Self {
        self.öğe_stili = Some(değer);
        self
    }
    pub fn çizgi_stili(mut self, değer: GrafoÇizgiStili) -> Self {
        self.çizgi_stili = Some(değer);
        self
    }
    pub fn etiket(mut self, değer: EtiketYaması) -> Self {
        self.etiket = Some(değer);
        self
    }
    pub fn kenar_etiketi(mut self, değer: EtiketYaması) -> Self {
        self.kenar_etiketi = Some(değer);
        self
    }
    pub fn odak(mut self, değer: GrafoVurguOdağı) -> Self {
        self.odak = Some(değer);
        self
    }
    pub fn ölçek(mut self, değer: f32) -> Self {
        self.ölçek = değer.is_finite().then(|| değer.max(0.0));
        self
    }
    pub fn devre_dışı(mut self, değer: bool) -> Self {
        self.devre_dışı = Some(değer);
        self
    }
}

/// Bağ ucu; ECharts sayı sırası, id veya ad kabul eder.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GrafoUcu {
    Sıra(usize),
    Kimlik(String),
}

impl From<usize> for GrafoUcu {
    fn from(değer: usize) -> Self {
        Self::Sıra(değer)
    }
}

impl From<i32> for GrafoUcu {
    fn from(değer: i32) -> Self {
        Self::Sıra(değer.max(0) as usize)
    }
}

impl From<String> for GrafoUcu {
    fn from(değer: String) -> Self {
        Self::Kimlik(değer)
    }
}

impl From<&str> for GrafoUcu {
    fn from(değer: &str) -> Self {
        Self::Kimlik(değer.to_string())
    }
}

/// Graph bağ öğesi (`links` / `edges`).
#[derive(Clone, PartialEq, Debug)]
pub struct GrafoBağı {
    pub kaynak: GrafoUcu,
    pub hedef: GrafoUcu,
    pub değer: Option<f64>,
    pub semboller: Option<[Sembol; 2]>,
    pub sembol_boyutları: Option<[f32; 2]>,
    pub kuvvet_yerleşimini_yoksay: bool,
    pub çizgi_stili: Option<GrafoÇizgiStili>,
    pub etiket: Option<EtiketYaması>,
    pub vurgu: GrafoDurumu,
    pub bulanık: GrafoDurumu,
    pub seçili: GrafoDurumu,
}

impl GrafoBağı {
    pub fn yeni(kaynak: impl Into<GrafoUcu>, hedef: impl Into<GrafoUcu>) -> Self {
        Self {
            kaynak: kaynak.into(),
            hedef: hedef.into(),
            değer: None,
            semboller: None,
            sembol_boyutları: None,
            kuvvet_yerleşimini_yoksay: false,
            çizgi_stili: None,
            etiket: None,
            vurgu: GrafoDurumu::default(),
            bulanık: GrafoDurumu::default(),
            seçili: GrafoDurumu::default(),
        }
    }
    pub fn değer(mut self, değer: f64) -> Self {
        self.değer = değer.is_finite().then_some(değer);
        self
    }
    pub fn semboller(mut self, kaynak: Sembol, hedef: Sembol) -> Self {
        self.semboller = Some([kaynak, hedef]);
        self
    }
    pub fn sembol_boyutları(mut self, kaynak: f32, hedef: f32) -> Self {
        self.sembol_boyutları = Some([kaynak.max(0.0), hedef.max(0.0)]);
        self
    }
    pub fn kuvvet_yerleşimini_yoksay(mut self, değer: bool) -> Self {
        self.kuvvet_yerleşimini_yoksay = değer;
        self
    }
    pub fn çizgi_stili(mut self, değer: GrafoÇizgiStili) -> Self {
        self.çizgi_stili = Some(değer);
        self
    }
    pub fn etiket(mut self, değer: EtiketYaması) -> Self {
        self.etiket = Some(değer);
        self
    }
    pub fn vurgu(mut self, değer: GrafoDurumu) -> Self {
        self.vurgu = değer;
        self
    }
    pub fn bulanık(mut self, değer: GrafoDurumu) -> Self {
        self.bulanık = değer;
        self
    }
    pub fn seçili(mut self, değer: GrafoDurumu) -> Self {
        self.seçili = değer;
        self
    }
}

/// Düğüm kategori seçicisi.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GrafoKategoriSeçimi {
    Sıra(usize),
    Ad(String),
}

/// Graph kategori öğesi.
#[derive(Clone, PartialEq, Debug)]
pub struct GrafoKategorisi {
    pub ad: String,
    pub değer: Option<VeriDeğeri>,
    pub sembol: Option<Sembol>,
    pub boyut: Option<f32>,
    pub öğe_stili: Option<GrafoÖğeStili>,
    pub etiket: Option<EtiketYaması>,
    pub vurgu: GrafoDurumu,
    pub bulanık: GrafoDurumu,
    pub seçili: GrafoDurumu,
}

impl GrafoKategorisi {
    pub fn yeni(ad: impl Into<String>) -> Self {
        Self {
            ad: ad.into(),
            değer: None,
            sembol: None,
            boyut: None,
            öğe_stili: None,
            etiket: None,
            vurgu: GrafoDurumu::default(),
            bulanık: GrafoDurumu::default(),
            seçili: GrafoDurumu::default(),
        }
    }
    pub fn değer(mut self, değer: impl Into<VeriDeğeri>) -> Self {
        self.değer = Some(değer.into());
        self
    }
    pub fn sembol(mut self, değer: Sembol) -> Self {
        self.sembol = Some(değer);
        self
    }
    pub fn boyut(mut self, değer: f32) -> Self {
        self.boyut = değer.is_finite().then(|| değer.max(0.0));
        self
    }
    pub fn öğe_stili(mut self, değer: GrafoÖğeStili) -> Self {
        self.öğe_stili = Some(değer);
        self
    }
    pub fn etiket(mut self, değer: EtiketYaması) -> Self {
        self.etiket = Some(değer);
        self
    }
    pub fn vurgu(mut self, değer: GrafoDurumu) -> Self {
        self.vurgu = değer;
        self
    }
    pub fn bulanık(mut self, değer: GrafoDurumu) -> Self {
        self.bulanık = değer;
        self
    }
    pub fn seçili(mut self, değer: GrafoDurumu) -> Self {
        self.seçili = değer;
        self
    }
}

/// Graph düğümü (`data` / `nodes`).
#[derive(Clone, PartialEq, Debug)]
pub struct GrafoDüğümü {
    pub kimlik: Option<String>,
    pub ad: String,
    /// Geriye uyumlu birincil sayısal değer.
    pub değer: Option<f64>,
    /// Sayı/dizi/metin dahil gerçek Graph `value`.
    pub ham_değer: Option<VeriDeğeri>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    /// Takvim koordinatındaki tarih (`data[i][0]`, Unix milisaniyesi).
    pub takvim_tarihi_ms: Option<f64>,
    /// Matrix koordinatındaki hücre ya da aralık.
    pub matris_koordinatı: Option<(MatrisAralığı, MatrisAralığı)>,
    pub sembol: Option<Sembol>,
    /// Eski API ile uyumlu, ECharts `symbolSize` çapı.
    pub boyut: f32,
    /// `symbolSize` düğüm üzerinde açıkça verildi mi? `false` olduğunda
    /// kategori ve seri sembol boyutu kalıtılır.
    pub boyut_açık: bool,
    pub boyut_çifti: Option<[f32; 2]>,
    /// Eski sayı kategorisi yolu.
    pub kategori: Option<usize>,
    pub kategori_seçimi: Option<GrafoKategoriSeçimi>,
    pub sabit: bool,
    pub sürüklenebilir: Option<bool>,
    pub imleç: Option<String>,
    pub öğe_stili: Option<GrafoÖğeStili>,
    pub etiket: Option<EtiketYaması>,
    pub vurgu: GrafoDurumu,
    pub bulanık: GrafoDurumu,
    pub seçili: GrafoDurumu,
    pub başlangıçta_seçili: bool,
}

impl GrafoDüğümü {
    pub fn yeni(ad: impl Into<String>, boyut: f32) -> Self {
        Self {
            kimlik: None,
            ad: ad.into(),
            değer: None,
            ham_değer: None,
            x: None,
            y: None,
            takvim_tarihi_ms: None,
            matris_koordinatı: None,
            sembol: None,
            boyut: if boyut.is_finite() {
                boyut.max(0.0)
            } else {
                10.0
            },
            boyut_açık: true,
            boyut_çifti: None,
            kategori: None,
            kategori_seçimi: None,
            sabit: false,
            sürüklenebilir: None,
            imleç: None,
            öğe_stili: None,
            etiket: None,
            vurgu: GrafoDurumu::default(),
            bulanık: GrafoDurumu::default(),
            seçili: GrafoDurumu::default(),
            başlangıçta_seçili: false,
        }
    }

    /// ECharts nesne verisindeki açık `symbolSize` alanı bulunmayan düğüm.
    /// Boyut, önce kategoriden sonra seriden kalıtılır.
    pub fn varsayılan(ad: impl Into<String>) -> Self {
        let mut düğüm = Self::yeni(ad, 10.0);
        düğüm.boyut_açık = false;
        düğüm
    }
    pub fn kimlik(mut self, değer: impl Into<String>) -> Self {
        self.kimlik = Some(değer.into());
        self
    }
    pub fn kategori(mut self, kategori: usize) -> Self {
        self.kategori = Some(kategori);
        self.kategori_seçimi = Some(GrafoKategoriSeçimi::Sıra(kategori));
        self
    }
    pub fn kategori_adı(mut self, kategori: impl Into<String>) -> Self {
        self.kategori = None;
        self.kategori_seçimi = Some(GrafoKategoriSeçimi::Ad(kategori.into()));
        self
    }
    pub fn değerli(mut self, değer: f64) -> Self {
        self.değer = değer.is_finite().then_some(değer);
        self.ham_değer = self.değer.map(VeriDeğeri::from);
        self
    }
    pub fn ham_değer(mut self, değer: impl Into<VeriDeğeri>) -> Self {
        let değer = değer.into();
        self.değer = değer.sayı();
        self.ham_değer = Some(değer);
        self
    }
    pub fn konum(mut self, x: f32, y: f32) -> Self {
        if x.is_finite() && y.is_finite() {
            self.x = Some(x);
            self.y = Some(y);
        }
        self
    }
    pub fn sembol(mut self, değer: Sembol) -> Self {
        self.sembol = Some(değer);
        self
    }
    pub fn boyut(mut self, değer: f32) -> Self {
        if değer.is_finite() {
            self.boyut = değer.max(0.0);
            self.boyut_çifti = None;
            self.boyut_açık = true;
        }
        self
    }
    pub fn boyut_çifti(mut self, genişlik: f32, yükseklik: f32) -> Self {
        if genişlik.is_finite() && yükseklik.is_finite() {
            self.boyut_çifti = Some([genişlik.max(0.0), yükseklik.max(0.0)]);
            self.boyut = (genişlik + yükseklik).max(0.0) / 2.0;
            self.boyut_açık = true;
        }
        self
    }
    pub fn sabit(mut self, değer: bool) -> Self {
        self.sabit = değer;
        self
    }
    pub fn sürüklenebilir(mut self, değer: bool) -> Self {
        self.sürüklenebilir = Some(değer);
        self
    }
    pub fn imleç(mut self, değer: impl Into<String>) -> Self {
        self.imleç = Some(değer.into());
        self
    }
    pub fn öğe_stili(mut self, değer: GrafoÖğeStili) -> Self {
        self.öğe_stili = Some(değer);
        self
    }
    pub fn etiket(mut self, değer: EtiketYaması) -> Self {
        self.etiket = Some(değer);
        self
    }
    pub fn vurgu(mut self, değer: GrafoDurumu) -> Self {
        self.vurgu = değer;
        self
    }
    pub fn bulanık(mut self, değer: GrafoDurumu) -> Self {
        self.bulanık = değer;
        self
    }
    pub fn seçili(mut self, değer: GrafoDurumu) -> Self {
        self.seçili = değer;
        self
    }
    pub fn başlangıçta_seçili(mut self, değer: bool) -> Self {
        self.başlangıçta_seçili = değer;
        self
    }

    /// Düğümü takvim koordinatındaki bir güne bağlar.
    pub fn takvim_tarihi(mut self, tarih_ms: f64) -> Self {
        self.takvim_tarihi_ms = tarih_ms.is_finite().then_some(tarih_ms);
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

    /// Ham `value` dizisinin verilen boyutundaki sayısal değer.
    pub fn sayısal_boyut(&self, sıra: usize) -> Option<f64> {
        match self.ham_değer.as_ref()? {
            VeriDeğeri::Sayı(değer) => (sıra == 0).then_some(*değer),
            VeriDeğeri::Çift(değerler) => değerler.get(sıra).copied(),
            VeriDeğeri::Dizi(değerler) => değerler.get(sıra).copied(),
            VeriDeğeri::KarmaDizi(değerler) => değerler.get(sıra).and_then(VeriDeğeri::sayı),
            VeriDeğeri::Zaman(değer) => (sıra == 0).then_some(*değer as f64),
            VeriDeğeri::Metin(değer) => (sıra == 0).then(|| değer.parse().ok()).flatten(),
            VeriDeğeri::Mantıksal(değer) => (sıra == 0).then_some(f64::from(*değer)),
            VeriDeğeri::Boş => None,
        }
        .filter(|değer| değer.is_finite())
    }

    /// Koordinat sistemi boyutunu çözer; kategorik eksende metin değerini
    /// eksenin ordinal sırasına dönüştürür.
    pub fn koordinat_boyutu(&self, sıra: usize, kategoriler: &[String]) -> Option<f64> {
        let ham = self.ham_değer.as_ref()?;
        let değer = match ham {
            VeriDeğeri::KarmaDizi(değerler) => değerler.get(sıra)?,
            VeriDeğeri::Metin(_)
            | VeriDeğeri::Sayı(_)
            | VeriDeğeri::Zaman(_)
            | VeriDeğeri::Mantıksal(_)
            | VeriDeğeri::Boş
                if sıra == 0 =>
            {
                ham
            }
            _ => return self.sayısal_boyut(sıra),
        };
        değer.sayı().filter(|değer| değer.is_finite()).or_else(|| {
            let VeriDeğeri::Metin(ad) = değer else {
                return None;
            };
            kategoriler
                .iter()
                .position(|kategori| kategori == ad)
                .map(|sıra| sıra as f64)
        })
    }

    /// Kartezyen eksenlerin kategori/değer birleşimine göre ECharts'ın
    /// `createSeriesData` boyut tamamlama kuralını uygular.
    pub fn kartezyen_değerleri(
        &self,
        veri_sırası: usize,
        x_kategorik: bool,
        y_kategorik: bool,
    ) -> Option<(f64, f64)> {
        let ham = self.ham_değer.as_ref();
        let bileşik = matches!(
            ham,
            Some(VeriDeğeri::Çift(_) | VeriDeğeri::Dizi(_) | VeriDeğeri::KarmaDizi(_))
        );
        if bileşik {
            return self.sayısal_boyut(0).zip(self.sayısal_boyut(1));
        }
        let değer = self
            .sayısal_boyut(0)
            .or(self.değer)
            .filter(|değer| değer.is_finite())?;
        match (x_kategorik, y_kategorik) {
            (true, false) => Some((veri_sırası as f64, değer)),
            (false, true) => Some((değer, veri_sırası as f64)),
            // İki değer ekseninde tek skaler yalnız ilk boyutu doldurur;
            // ikinci koordinat yoktur ve düğüm çizilmez.
            (false, false) => None,
            // ECharts iki kategori eksenini dizi olmayan kaynakta ilk
            // ordinal boyutla tamamlar; ikinci skaler kategori değeridir.
            (true, true) => Some((veri_sırası as f64, değer)),
        }
    }
}

/// Graph serisi (`series.graph`).
#[derive(Clone, Debug)]
pub struct GrafoSerisi {
    pub kimlik: Option<String>,
    pub ad: Option<String>,
    pub düğümler: Vec<GrafoDüğümü>,
    /// Eski kısa API: `(kaynak adı, hedef adı)`.
    pub bağlar: Vec<(String, String)>,
    pub ayrıntılı_bağlar: Vec<GrafoBağı>,
    pub kategoriler: Vec<GrafoKategorisi>,
    pub koordinat_sistemi: GrafoKoordinatSistemi,
    pub yerleşim: GrafoYerleşimi,
    pub dairesel: GrafoDaireselAyarı,
    pub kuvvet: GrafoKuvveti,
    pub otomatik_eğrilik: GrafoOtomatikEğrilik,
    pub sol: Option<Uzunluk>,
    pub üst: Option<Uzunluk>,
    pub sağ: Option<Uzunluk>,
    pub alt: Option<Uzunluk>,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    pub en_boy_koruma: GrafoEnBoyKoruma,
    pub en_boy_yatay_hizası: GrafoEnBoyYatayHizası,
    pub en_boy_dikey_hizası: GrafoEnBoyDikeyHizası,
    /// Roam merkez seçeneği; `None` görünüm kutusu merkezidir.
    pub merkez: Option<(Uzunluk, Uzunluk)>,
    pub yakınlaştırma: f32,
    pub en_küçük_yakınlaştırma: Option<f32>,
    pub en_büyük_yakınlaştırma: Option<f32>,
    pub düğüm_ölçek_oranı: f32,
    pub gezinme: GrafoGezinmesi,
    pub gezinme_tetikleyicisi: GrafoGezinmeTetikleyicisi,
    pub sürüklenebilir: bool,
    pub gösterge_vurgusu: bool,
    pub sessiz: bool,
    pub sembol: Sembol,
    pub sembol_boyutu: f32,
    /// Eski örneklerin yalnız büyük düğüm etiketi göstermesi için eşik.
    pub etiket_eşiği: f32,
    pub etiket_göster: bool,
    pub etiket: Etiket,
    pub kenar_etiketi: Etiket,
    pub öğe_stili: crate::model::stil::ÖğeStili,
    pub çizgi_stili: crate::model::stil::ÇizgiStili,
    pub grafo_öğe_stili: GrafoÖğeStili,
    pub grafo_çizgi_stili: GrafoÇizgiStili,
    pub kenar_sembolleri: [Sembol; 2],
    pub kenar_sembol_boyutları: [f32; 2],
    /// Eski hedef-ok kolaylığı.
    pub hedef_oku: bool,
    pub hedef_oku_boyutu: f32,
    pub vurgu: GrafoDurumu,
    pub bulanık: GrafoDurumu,
    pub seçili: GrafoDurumu,
    pub etiket_örtüşmesini_gizle: bool,
    pub eksen_bağı: EksenBağı,
    pub kutupsal_sırası: usize,
    pub tek_eksen_sırası: Option<usize>,
    pub takvim_sırası: Option<usize>,
    pub matris_sırası: Option<usize>,
    pub z: i32,
    pub ipucu: Option<İpucu>,
    /// SetOption sırasında korunmuş düğüm noktaları. Normal kullanıcılar
    /// için dinamik graph sürekliliğini, kanıt koşucusunda kilitli resmî
    /// yerleşimin yeniden oynatılmasını sağlar. Noktalar tuval uzayındadır.
    pub korunmuş_noktalar: Option<Vec<(f32, f32)>>,
    /// Kuvvet başlangıçlarının belirlenimci üretimi.
    pub rastgele_tohumu: u32,
}

impl Default for GrafoSerisi {
    fn default() -> Self {
        let etiket = Etiket::yeni().biçimleyici("{b}");
        Self {
            kimlik: None,
            ad: None,
            düğümler: Vec::new(),
            bağlar: Vec::new(),
            ayrıntılı_bağlar: Vec::new(),
            kategoriler: Vec::new(),
            koordinat_sistemi: GrafoKoordinatSistemi::Görünüm,
            yerleşim: GrafoYerleşimi::Yok,
            dairesel: GrafoDaireselAyarı::default(),
            kuvvet: GrafoKuvveti::default(),
            otomatik_eğrilik: GrafoOtomatikEğrilik::Kapalı,
            sol: None,
            üst: None,
            sağ: None,
            alt: None,
            genişlik: None,
            yükseklik: None,
            en_boy_koruma: GrafoEnBoyKoruma::Kapalı,
            en_boy_yatay_hizası: GrafoEnBoyYatayHizası::Orta,
            en_boy_dikey_hizası: GrafoEnBoyDikeyHizası::Orta,
            merkez: None,
            yakınlaştırma: 1.0,
            en_küçük_yakınlaştırma: None,
            en_büyük_yakınlaştırma: None,
            düğüm_ölçek_oranı: 0.6,
            gezinme: GrafoGezinmesi::Kapalı,
            gezinme_tetikleyicisi: GrafoGezinmeTetikleyicisi::KendiAlanı,
            sürüklenebilir: false,
            gösterge_vurgusu: true,
            sessiz: false,
            sembol: Sembol::Daire,
            sembol_boyutu: 10.0,
            // ECharts Graph bütün `label.show` düğümlerini sembol
            // boyutundan bağımsız çizer. Alan yalnız eski uygulamaların
            // isteğe bağlı büyük-düğüm süzgeci olarak korunur.
            etiket_eşiği: 0.0,
            etiket_göster: false,
            etiket,
            kenar_etiketi: Etiket::yeni()
                .konum(crate::model::stil::EtiketKonumu::İç)
                .uzaklık(5.0),
            öğe_stili: crate::model::stil::ÖğeStili::default(),
            çizgi_stili: crate::model::stil::ÇizgiStili::yeni()
                .kalınlık(1.0)
                .opaklık(0.5),
            grafo_öğe_stili: GrafoÖğeStili::default(),
            grafo_çizgi_stili: GrafoÇizgiStili::seri_varsayılanı(),
            kenar_sembolleri: [Sembol::Yok, Sembol::Yok],
            kenar_sembol_boyutları: [10.0, 10.0],
            hedef_oku: false,
            hedef_oku_boyutu: 10.0,
            vurgu: GrafoDurumu::yeni()
                .ölçek(1.1)
                .etiket(EtiketYaması::yeni().göster(true)),
            bulanık: GrafoDurumu::default(),
            seçili: GrafoDurumu::yeni()
                .öğe_stili(GrafoÖğeStili::yeni().kenarlık_rengi(crate::tema::aksan_50())),
            etiket_örtüşmesini_gizle: false,
            eksen_bağı: EksenBağı::default(),
            kutupsal_sırası: 0,
            tek_eksen_sırası: None,
            takvim_sırası: None,
            matris_sırası: None,
            z: 2,
            ipucu: None,
            korunmuş_noktalar: None,
            rastgele_tohumu: 0x5eed_1234,
        }
    }
}

impl GrafoSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }
    pub fn kimlik(mut self, değer: impl Into<String>) -> Self {
        self.kimlik = Some(değer.into());
        self
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
            .map(|(kaynak, hedef)| (kaynak.into(), hedef.into()))
            .collect();
        self.ayrıntılı_bağlar = self
            .bağlar
            .iter()
            .map(|(kaynak, hedef)| GrafoBağı::yeni(kaynak.clone(), hedef.clone()))
            .collect();
        self
    }
    pub fn ayrıntılı_bağlar(mut self, bağlar: impl IntoIterator<Item = GrafoBağı>) -> Self {
        self.ayrıntılı_bağlar = bağlar.into_iter().collect();
        self.bağlar = self
            .ayrıntılı_bağlar
            .iter()
            .filter_map(|bağ| match (&bağ.kaynak, &bağ.hedef) {
                (GrafoUcu::Kimlik(k), GrafoUcu::Kimlik(h)) => Some((k.clone(), h.clone())),
                _ => None,
            })
            .collect();
        self
    }
    pub fn kategoriler(mut self, değer: impl IntoIterator<Item = GrafoKategorisi>) -> Self {
        self.kategoriler = değer.into_iter().collect();
        self
    }
    pub fn koordinat_sistemi(mut self, değer: GrafoKoordinatSistemi) -> Self {
        self.koordinat_sistemi = değer;
        self
    }
    pub fn yerleşim(mut self, değer: GrafoYerleşimi) -> Self {
        self.yerleşim = değer;
        self
    }
    pub fn dairesel(mut self, değer: GrafoDaireselAyarı) -> Self {
        self.dairesel = değer;
        self
    }
    pub fn kuvvet(mut self, değer: GrafoKuvveti) -> Self {
        self.kuvvet = değer;
        self
    }
    pub fn otomatik_eğrilik(mut self, değer: GrafoOtomatikEğrilik) -> Self {
        self.otomatik_eğrilik = değer;
        self
    }
    pub fn kutu(
        mut self,
        sol: impl Into<Uzunluk>,
        üst: impl Into<Uzunluk>,
        genişlik: impl Into<Uzunluk>,
        yükseklik: impl Into<Uzunluk>,
    ) -> Self {
        self.sol = Some(sol.into());
        self.üst = Some(üst.into());
        self.genişlik = Some(genişlik.into());
        self.yükseklik = Some(yükseklik.into());
        self.sağ = None;
        self.alt = None;
        self
    }
    pub fn sol(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sol = Some(değer.into());
        self
    }
    pub fn üst(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.üst = Some(değer.into());
        self
    }
    pub fn sağ(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(değer.into());
        self
    }
    pub fn alt(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.alt = Some(değer.into());
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
    pub fn en_boy_koruma(mut self, değer: GrafoEnBoyKoruma) -> Self {
        self.en_boy_koruma = değer;
        self
    }
    pub fn en_boy_hizası(
        mut self,
        yatay: GrafoEnBoyYatayHizası,
        dikey: GrafoEnBoyDikeyHizası,
    ) -> Self {
        self.en_boy_yatay_hizası = yatay;
        self.en_boy_dikey_hizası = dikey;
        self
    }
    pub fn merkez(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.merkez = Some((x.into(), y.into()));
        self
    }
    pub fn yakınlaştırma(mut self, değer: f32) -> Self {
        if değer.is_finite() && değer > 0.0 {
            self.yakınlaştırma = değer;
        }
        self
    }
    pub fn yakınlaştırma_sınırı(mut self, en_az: f32, en_çok: f32) -> Self {
        if en_az.is_finite() && en_çok.is_finite() && en_az > 0.0 && en_çok >= en_az {
            self.en_küçük_yakınlaştırma = Some(en_az);
            self.en_büyük_yakınlaştırma = Some(en_çok);
        }
        self
    }
    pub fn düğüm_ölçek_oranı(mut self, değer: f32) -> Self {
        if değer.is_finite() {
            self.düğüm_ölçek_oranı = değer.max(0.0);
        }
        self
    }
    pub fn gezinme(mut self, açık: bool) -> Self {
        self.gezinme = if açık {
            GrafoGezinmesi::Açık
        } else {
            GrafoGezinmesi::Kapalı
        };
        self
    }
    pub fn gezinme_kipi(mut self, kip: GrafoGezinmesi) -> Self {
        self.gezinme = kip;
        self
    }
    pub fn gezinme_tetikleyicisi(mut self, değer: GrafoGezinmeTetikleyicisi) -> Self {
        self.gezinme_tetikleyicisi = değer;
        self
    }
    pub fn sürüklenebilir(mut self, açık: bool) -> Self {
        self.sürüklenebilir = açık;
        self
    }
    pub fn gösterge_vurgusu(mut self, açık: bool) -> Self {
        self.gösterge_vurgusu = açık;
        self
    }
    pub fn sessiz(mut self, açık: bool) -> Self {
        self.sessiz = açık;
        self
    }
    pub fn sembol(mut self, değer: Sembol) -> Self {
        self.sembol = değer;
        self
    }
    pub fn sembol_boyutu(mut self, değer: f32) -> Self {
        if değer.is_finite() {
            self.sembol_boyutu = değer.max(0.0);
        }
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
    pub fn kenar_etiketi(mut self, değer: Etiket) -> Self {
        self.kenar_etiketi = değer;
        self
    }
    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.koordinat_sistemi = GrafoKoordinatSistemi::Takvim;
        self.takvim_sırası = Some(sıra);
        self.matris_sırası = None;
        self.tek_eksen_sırası = None;
        self
    }
    pub fn matris_sırası(mut self, sıra: usize) -> Self {
        self.koordinat_sistemi = GrafoKoordinatSistemi::Matris;
        self.matris_sırası = Some(sıra);
        self.takvim_sırası = None;
        self.tek_eksen_sırası = None;
        self
    }
    pub fn kartezyen(mut self, x_ekseni: usize, y_ekseni: usize) -> Self {
        self.koordinat_sistemi = GrafoKoordinatSistemi::Kartezyen2B;
        self.eksen_bağı = EksenBağı {
            x: x_ekseni,
            y: y_ekseni,
        };
        self
    }
    pub fn kutupsal(mut self, sıra: usize) -> Self {
        self.koordinat_sistemi = GrafoKoordinatSistemi::Kutupsal;
        self.kutupsal_sırası = sıra;
        self
    }
    pub fn tek_eksen(mut self, sıra: usize) -> Self {
        self.koordinat_sistemi = GrafoKoordinatSistemi::TekEksen;
        self.tek_eksen_sırası = Some(sıra);
        self
    }
    pub fn z(mut self, z: i32) -> Self {
        self.z = z;
        self
    }
    pub fn öğe_stili(mut self, stil: crate::model::stil::ÖğeStili) -> Self {
        self.grafo_öğe_stili = GrafoÖğeStili {
            renk: stil.renk.clone(),
            kenarlık_rengi: stil.kenarlık_rengi,
            kenarlık_kalınlığı: Some(stil.kenarlık_kalınlığı),
            kenarlık_türü: Some(stil.kenarlık_türü),
            opaklık: stil.opaklık,
            gölge_bulanıklığı: Some(stil.gölge_bulanıklığı),
            gölge_rengi: stil.gölge_rengi,
            gölge_kayması: Some(stil.gölge_kayması),
        };
        self.öğe_stili = stil;
        self
    }
    pub fn grafo_öğe_stili(mut self, stil: GrafoÖğeStili) -> Self {
        self.grafo_öğe_stili = stil;
        self
    }
    pub fn çizgi_stili(mut self, stil: crate::model::stil::ÇizgiStili) -> Self {
        self.grafo_çizgi_stili = GrafoÇizgiStili {
            renk: stil.renk.map(GrafoKenarBoyası::Renk),
            kalınlık: Some(stil.kalınlık),
            tür: Some(stil.tür),
            opaklık: Some(stil.opaklık),
            eğrilik: Some(0.0),
            gölge_bulanıklığı: Some(stil.gölge_bulanıklığı),
            gölge_rengi: stil.gölge_rengi,
            gölge_kayması: Some(stil.gölge_kayması),
        };
        self.çizgi_stili = stil;
        self
    }
    pub fn grafo_çizgi_stili(mut self, stil: GrafoÇizgiStili) -> Self {
        self.grafo_çizgi_stili = stil;
        self
    }
    pub fn kenar_sembolleri(mut self, kaynak: Sembol, hedef: Sembol) -> Self {
        self.hedef_oku = matches!(hedef, Sembol::Üçgen);
        self.kenar_sembolleri = [kaynak, hedef];
        self
    }
    pub fn kenar_sembol_boyutları(mut self, kaynak: f32, hedef: f32) -> Self {
        self.kenar_sembol_boyutları = [kaynak.max(0.0), hedef.max(0.0)];
        self.hedef_oku_boyutu = hedef.max(0.0);
        self
    }
    pub fn hedef_oku(mut self, açık: bool) -> Self {
        self.hedef_oku = açık;
        self.kenar_sembolleri[1] = if açık { Sembol::Üçgen } else { Sembol::Yok };
        self
    }
    pub fn hedef_oku_boyutu(mut self, boyut: f32) -> Self {
        self.hedef_oku_boyutu = boyut.max(0.0);
        self.kenar_sembol_boyutları[1] = self.hedef_oku_boyutu;
        self
    }
    pub fn vurgu(mut self, değer: GrafoDurumu) -> Self {
        self.vurgu = değer;
        self
    }
    /// ECharts'ın eski `focusNodeAdjacency` seçeneği için uyumluluk
    /// yüzeyi. Resmî `backwardCompat` ön-işleyicisi, seçenek `false` olsa
    /// bile yalnız varlığına bakar ve açık bir `emphasis.focus` yoksa
    /// `adjacency` yazar; burada da aynı öncelik korunur.
    pub fn eski_komşuluk_odağı(mut self, _değer: bool) -> Self {
        if self.vurgu.odak.is_none() {
            self.vurgu.odak = Some(GrafoVurguOdağı::Komşuluk);
        }
        self
    }
    pub fn bulanık(mut self, değer: GrafoDurumu) -> Self {
        self.bulanık = değer;
        self
    }
    pub fn seçili(mut self, değer: GrafoDurumu) -> Self {
        self.seçili = değer;
        self
    }
    pub fn etiket_örtüşmesini_gizle(mut self, açık: bool) -> Self {
        self.etiket_örtüşmesini_gizle = açık;
        self
    }
    pub fn korunmuş_noktalar(mut self, değer: impl IntoIterator<Item = (f32, f32)>) -> Self {
        self.korunmuş_noktalar = Some(değer.into_iter().collect());
        self
    }
    pub fn rastgele_tohumu(mut self, değer: u32) -> Self {
        self.rastgele_tohumu = değer;
        self
    }
    pub fn ipucu(mut self, değer: İpucu) -> Self {
        self.ipucu = Some(değer);
        self
    }
}
