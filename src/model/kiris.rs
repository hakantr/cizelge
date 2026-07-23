//! Kiriş (`series.chord`) seçenek modeli.
//!
//! Alanlar ECharts'ın kilitli `src/chart/chord/ChordSeries.ts` yüzeyini
//! izler. Düğüm ve bağların normal/vurgu/bulanık/seçili katmanları ayrı
//! tutulur; bu sayede yalnız açık alanlar seri varsayılanlarının üstüne
//! uygulanır.

use crate::model::Uzunluk;
use crate::model::bilesen::İpucu;
use crate::model::deger::VeriDeğeri;
use crate::model::stil::{Etiket, EtiketKonumu, EtiketYaması, ÇizgiTürü};
use crate::renk::{Dolgu, Renk};

/// `lineStyle.color`: düz boya ya da uç düğümlerden türetilen boya.
#[derive(Clone, PartialEq, Debug)]
pub enum KirişKenarBoyası {
    Dolgu(Dolgu),
    Kaynak,
    Hedef,
    Gradyan,
}

impl From<Dolgu> for KirişKenarBoyası {
    fn from(değer: Dolgu) -> Self {
        Self::Dolgu(değer)
    }
}

impl From<Renk> for KirişKenarBoyası {
    fn from(değer: Renk) -> Self {
        Self::Dolgu(Dolgu::Düz(değer))
    }
}

impl From<u32> for KirişKenarBoyası {
    fn from(değer: u32) -> Self {
        Self::from(Renk::from(değer))
    }
}

impl From<&str> for KirişKenarBoyası {
    fn from(değer: &str) -> Self {
        match değer {
            "source" | "kaynak" => Self::Kaynak,
            "target" | "hedef" => Self::Hedef,
            "gradient" | "gradyan" => Self::Gradyan,
            diğer => Self::Dolgu(Dolgu::from(diğer)),
        }
    }
}

/// `emphasis.focus` kapsamı.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum KirişVurguOdağı {
    Yok,
    Kendisi,
    #[default]
    Komşuluk,
    Seri,
}

/// Zrender sektör köşe sırası: iç-başlangıç, iç-bitiş,
/// dış-başlangıç, dış-bitiş.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct KirişKöşeYarıçapı(pub [Uzunluk; 4]);

impl From<f32> for KirişKöşeYarıçapı {
    fn from(değer: f32) -> Self {
        Self([Uzunluk::Piksel(değer); 4])
    }
}

impl From<i32> for KirişKöşeYarıçapı {
    fn from(değer: i32) -> Self {
        Self::from(değer as f32)
    }
}

impl From<[f32; 2]> for KirişKöşeYarıçapı {
    fn from([iç, dış]: [f32; 2]) -> Self {
        Self([
            Uzunluk::Piksel(iç),
            Uzunluk::Piksel(iç),
            Uzunluk::Piksel(dış),
            Uzunluk::Piksel(dış),
        ])
    }
}

impl From<[f32; 4]> for KirişKöşeYarıçapı {
    fn from(değer: [f32; 4]) -> Self {
        Self(değer.map(Uzunluk::Piksel))
    }
}

impl From<[Uzunluk; 2]> for KirişKöşeYarıçapı {
    fn from([iç, dış]: [Uzunluk; 2]) -> Self {
        Self([iç, iç, dış, dış])
    }
}

impl From<[Uzunluk; 4]> for KirişKöşeYarıçapı {
    fn from(değer: [Uzunluk; 4]) -> Self {
        Self(değer)
    }
}

/// Chord sektörünün `itemStyle` yaması.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct KirişÖğeStili {
    pub renk: Option<Dolgu>,
    pub kenarlık_rengi: Option<Renk>,
    pub kenarlık_kalınlığı: Option<f32>,
    pub kenarlık_türü: Option<ÇizgiTürü>,
    pub kenarlık_yarıçapı: Option<KirişKöşeYarıçapı>,
    pub opaklık: Option<f32>,
    pub gölge_bulanıklığı: Option<f32>,
    pub gölge_rengi: Option<Renk>,
    pub gölge_kayması: Option<(f32, f32)>,
}

impl KirişÖğeStili {
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
    pub fn kenarlık_yarıçapı(mut self, değer: impl Into<KirişKöşeYarıçapı>) -> Self {
        self.kenarlık_yarıçapı = Some(değer.into());
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

/// Chord şeridinin `lineStyle` yaması.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct KirişÇizgiStili {
    pub renk: Option<KirişKenarBoyası>,
    pub opaklık: Option<f32>,
    pub kalınlık: Option<f32>,
    pub tür: Option<ÇizgiTürü>,
    pub eğrilik: Option<f32>,
    pub gölge_bulanıklığı: Option<f32>,
    pub gölge_rengi: Option<Renk>,
    pub gölge_kayması: Option<(f32, f32)>,
}

impl KirişÇizgiStili {
    pub fn yeni() -> Self {
        Self::default()
    }
    pub fn seri_varsayılanı() -> Self {
        Self {
            renk: Some(KirişKenarBoyası::Kaynak),
            opaklık: Some(0.2),
            kalınlık: Some(0.0),
            tür: Some(ÇizgiTürü::Düz),
            eğrilik: Some(0.7),
            gölge_bulanıklığı: Some(0.0),
            gölge_kayması: Some((0.0, 0.0)),
            ..Self::default()
        }
    }
    pub fn renk(mut self, değer: impl Into<KirişKenarBoyası>) -> Self {
        self.renk = Some(değer.into());
        self
    }
    pub fn opaklık(mut self, değer: f32) -> Self {
        self.opaklık = değer.is_finite().then(|| değer.clamp(0.0, 1.0));
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

/// Normal dışındaki düğüm/bağ durum katmanı.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct KirişDurumu {
    pub öğe_stili: Option<KirişÖğeStili>,
    pub çizgi_stili: Option<KirişÇizgiStili>,
    pub etiket: Option<EtiketYaması>,
    pub kenar_etiketi: Option<EtiketYaması>,
    pub odak: Option<KirişVurguOdağı>,
    pub ölçek: Option<f32>,
    pub devre_dışı: Option<bool>,
}

impl KirişDurumu {
    pub fn yeni() -> Self {
        Self::default()
    }
    pub fn öğe_stili(mut self, değer: KirişÖğeStili) -> Self {
        self.öğe_stili = Some(değer);
        self
    }
    pub fn çizgi_stili(mut self, değer: KirişÇizgiStili) -> Self {
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
    pub fn odak(mut self, değer: KirişVurguOdağı) -> Self {
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

/// `series.chord.data[]` / `nodes[]` öğesi.
#[derive(Clone, PartialEq, Debug)]
pub struct KirişDüğümü {
    pub kimlik: Option<String>,
    pub ad: String,
    pub değer: Option<VeriDeğeri>,
    pub öğe_stili: Option<KirişÖğeStili>,
    pub etiket: Option<EtiketYaması>,
    pub vurgu: KirişDurumu,
    pub bulanık: KirişDurumu,
    pub seçili: KirişDurumu,
    pub başlangıçta_seçili: bool,
}

impl KirişDüğümü {
    pub fn yeni(ad: impl Into<String>) -> Self {
        Self {
            kimlik: None,
            ad: ad.into(),
            değer: None,
            öğe_stili: None,
            etiket: None,
            vurgu: KirişDurumu::default(),
            bulanık: KirişDurumu::default(),
            seçili: KirişDurumu::default(),
            başlangıçta_seçili: false,
        }
    }
    pub fn kimlik(mut self, değer: impl Into<String>) -> Self {
        self.kimlik = Some(değer.into());
        self
    }
    pub fn değer(mut self, değer: impl Into<VeriDeğeri>) -> Self {
        self.değer = Some(değer.into());
        self
    }
    pub fn öğe_stili(mut self, değer: KirişÖğeStili) -> Self {
        self.öğe_stili = Some(değer);
        self
    }
    pub fn etiket(mut self, değer: EtiketYaması) -> Self {
        self.etiket = Some(değer);
        self
    }
    pub fn vurgu(mut self, değer: KirişDurumu) -> Self {
        self.vurgu = değer;
        self
    }
    pub fn bulanık(mut self, değer: KirişDurumu) -> Self {
        self.bulanık = değer;
        self
    }
    pub fn seçili(mut self, değer: KirişDurumu) -> Self {
        self.seçili = değer;
        self
    }
    pub fn başlangıçta_seçili(mut self, değer: bool) -> Self {
        self.başlangıçta_seçili = değer;
        self
    }
}

impl From<&str> for KirişDüğümü {
    fn from(değer: &str) -> Self {
        Self::yeni(değer)
    }
}

impl From<String> for KirişDüğümü {
    fn from(değer: String) -> Self {
        Self::yeni(değer)
    }
}

/// `series.chord.links[]` / `edges[]` öğesi.
#[derive(Clone, PartialEq, Debug)]
pub struct KirişBağı {
    pub kaynak: String,
    pub hedef: String,
    pub değer: f64,
    pub çizgi_stili: Option<KirişÇizgiStili>,
    pub kenar_etiketi: Option<EtiketYaması>,
    pub vurgu: KirişDurumu,
    pub bulanık: KirişDurumu,
    pub seçili: KirişDurumu,
}

impl KirişBağı {
    pub fn yeni(kaynak: impl Into<String>, hedef: impl Into<String>, değer: f64) -> Self {
        Self {
            kaynak: kaynak.into(),
            hedef: hedef.into(),
            değer,
            çizgi_stili: None,
            kenar_etiketi: None,
            vurgu: KirişDurumu::default(),
            bulanık: KirişDurumu::default(),
            seçili: KirişDurumu::default(),
        }
    }
    pub fn çizgi_stili(mut self, değer: KirişÇizgiStili) -> Self {
        self.çizgi_stili = Some(değer);
        self
    }
    pub fn kenar_etiketi(mut self, değer: EtiketYaması) -> Self {
        self.kenar_etiketi = Some(değer);
        self
    }
    pub fn vurgu(mut self, değer: KirişDurumu) -> Self {
        self.vurgu = değer;
        self
    }
    pub fn bulanık(mut self, değer: KirişDurumu) -> Self {
        self.bulanık = değer;
        self
    }
    pub fn seçili(mut self, değer: KirişDurumu) -> Self {
        self.seçili = değer;
        self
    }
}

/// Kiriş serisi (`series.chord`).
#[derive(Clone, Debug)]
pub struct KirişSerisi {
    pub kimlik: Option<String>,
    pub ad: Option<String>,
    pub z: i32,
    pub sessiz: bool,
    pub gösterge_vurgusu: bool,
    pub renkler: Vec<Renk>,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub sağ: Option<Uzunluk>,
    pub alt: Option<Uzunluk>,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    pub merkez: (Uzunluk, Uzunluk),
    pub yarıçap: (Uzunluk, Uzunluk),
    pub saat_yönünde: bool,
    pub başlangıç_açısı: f32,
    /// API yüzeyi korunur. Kilitli ECharts 6 yerleşimi `endAngle` verilse
    /// dahi tam çember kullandığından renderer aynı davranışı izler.
    pub bitiş_açısı: Option<f32>,
    pub dolgu_açısı: f32,
    pub en_küçük_açı: f32,
    pub düğümler: Vec<KirişDüğümü>,
    pub bağlar: Vec<KirişBağı>,
    pub öğe_stili: KirişÖğeStili,
    pub çizgi_stili: KirişÇizgiStili,
    pub etiket: Etiket,
    pub kenar_etiketi: Etiket,
    pub vurgu: KirişDurumu,
    pub bulanık: KirişDurumu,
    pub seçili: KirişDurumu,
    pub ipucu: Option<İpucu>,
}

impl Default for KirişSerisi {
    fn default() -> Self {
        let mut etiket = Etiket::yeni()
            .göster(true)
            .konum(EtiketKonumu::Dış)
            .uzaklık(5.0);
        etiket.sessiz = Some(false);
        Self {
            kimlik: None,
            ad: None,
            z: 2,
            sessiz: false,
            gösterge_vurgusu: true,
            renkler: Vec::new(),
            sol: Uzunluk::Piksel(0.0),
            üst: Uzunluk::Piksel(0.0),
            sağ: Some(Uzunluk::Piksel(0.0)),
            alt: Some(Uzunluk::Piksel(0.0)),
            genişlik: None,
            yükseklik: None,
            merkez: (Uzunluk::Yüzde(50.0), Uzunluk::Yüzde(50.0)),
            yarıçap: (Uzunluk::Yüzde(70.0), Uzunluk::Yüzde(80.0)),
            saat_yönünde: true,
            başlangıç_açısı: 90.0,
            bitiş_açısı: None,
            dolgu_açısı: 3.0,
            en_küçük_açı: 0.0,
            düğümler: Vec::new(),
            bağlar: Vec::new(),
            öğe_stili: KirişÖğeStili::yeni().kenarlık_yarıçapı([0.0, 5.0]),
            çizgi_stili: KirişÇizgiStili::seri_varsayılanı(),
            etiket,
            kenar_etiketi: Etiket::default(),
            vurgu: KirişDurumu::yeni()
                .odak(KirişVurguOdağı::Komşuluk)
                .çizgi_stili(KirişÇizgiStili::yeni().opaklık(0.5)),
            bulanık: KirişDurumu::default(),
            seçili: KirişDurumu::default(),
            ipucu: None,
        }
    }
}

impl KirişSerisi {
    pub fn yeni() -> Self {
        Self::default()
    }
    pub fn kimlik(mut self, değer: impl Into<String>) -> Self {
        self.kimlik = Some(değer.into());
        self
    }
    pub fn ad(mut self, değer: impl Into<String>) -> Self {
        self.ad = Some(değer.into());
        self
    }
    pub fn z(mut self, değer: i32) -> Self {
        self.z = değer;
        self
    }
    pub fn sessiz(mut self, değer: bool) -> Self {
        self.sessiz = değer;
        self
    }
    pub fn gösterge_vurgusu(mut self, değer: bool) -> Self {
        self.gösterge_vurgusu = değer;
        self
    }
    pub fn renkler(mut self, değer: impl IntoIterator<Item = impl Into<Renk>>) -> Self {
        self.renkler = değer.into_iter().map(Into::into).collect();
        self
    }
    pub fn sol(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sol = değer.into();
        self
    }
    pub fn üst(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.üst = değer.into();
        self
    }
    pub fn sağ(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(değer.into());
        self.genişlik = None;
        self
    }
    pub fn alt(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.alt = Some(değer.into());
        self.yükseklik = None;
        self
    }
    pub fn genişlik(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(değer.into());
        self.sağ = None;
        self
    }
    pub fn yükseklik(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(değer.into());
        self.alt = None;
        self
    }
    pub fn merkez(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.merkez = (x.into(), y.into());
        self
    }
    pub fn yarıçap(mut self, iç: impl Into<Uzunluk>, dış: impl Into<Uzunluk>) -> Self {
        self.yarıçap = (iç.into(), dış.into());
        self
    }
    pub fn saat_yönünde(mut self, değer: bool) -> Self {
        self.saat_yönünde = değer;
        self
    }
    pub fn başlangıç_açısı(mut self, değer: f32) -> Self {
        self.başlangıç_açısı = değer;
        self
    }
    pub fn bitiş_açısı(mut self, değer: f32) -> Self {
        self.bitiş_açısı = Some(değer);
        self
    }
    pub fn otomatik_bitiş_açısı(mut self) -> Self {
        self.bitiş_açısı = None;
        self
    }
    pub fn dolgu_açısı(mut self, değer: f32) -> Self {
        self.dolgu_açısı = değer.max(0.0);
        self
    }
    pub fn en_küçük_açı(mut self, değer: f32) -> Self {
        self.en_küçük_açı = değer.max(0.0);
        self
    }
    pub fn düğümler(
        mut self, değer: impl IntoIterator<Item = impl Into<KirişDüğümü>>
    ) -> Self {
        self.düğümler = değer.into_iter().map(Into::into).collect();
        self
    }
    pub fn ayrıntılı_bağlar(mut self, değer: impl IntoIterator<Item = KirişBağı>) -> Self {
        self.bağlar = değer.into_iter().collect();
        self
    }
    /// Kısa ECharts-benzeri bağ kurucusu. Düğümler açık verilmemişse ilk
    /// görülme sırasıyla bağ uçlarından türetilir.
    pub fn bağlar<S: Into<String>>(
        mut self,
        değer: impl IntoIterator<Item = (S, S, f64)>,
    ) -> Self {
        self.bağlar = değer
            .into_iter()
            .map(|(k, h, d)| KirişBağı::yeni(k, h, d))
            .collect();
        self
    }
    pub fn öğe_stili(mut self, değer: KirişÖğeStili) -> Self {
        self.öğe_stili = değer;
        self
    }
    pub fn çizgi_stili(mut self, değer: KirişÇizgiStili) -> Self {
        self.çizgi_stili = değer;
        self
    }
    pub fn etiket(mut self, değer: Etiket) -> Self {
        self.etiket = değer;
        self
    }
    pub fn kenar_etiketi(mut self, değer: Etiket) -> Self {
        self.kenar_etiketi = değer;
        self
    }
    pub fn vurgu(mut self, değer: KirişDurumu) -> Self {
        self.vurgu = değer;
        self
    }
    pub fn bulanık(mut self, değer: KirişDurumu) -> Self {
        self.bulanık = değer;
        self
    }
    pub fn seçili(mut self, değer: KirişDurumu) -> Self {
        self.seçili = değer;
        self
    }
    pub fn ipucu(mut self, değer: İpucu) -> Self {
        self.ipucu = Some(değer);
        self
    }
}
