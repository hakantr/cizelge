//! Sankey seçenek modeli — `echarts/src/chart/sankey/SankeySeries.ts`
//! karşılığı. Düğüm, bağ, seviye ve durum yamaları ayrı tutulur; böylece
//! ECharts'ın seri → level → veri öğesi kalıtımı açık sıfırları kaybetmez.

use crate::model::Uzunluk;
use crate::model::agac::AğaçGezinmesi;
use crate::model::bilesen::İpucu;
use crate::model::matris::MatrisAralığı;
use crate::model::stil::{
    Etiket, EtiketKonumu, EtiketYaması, YazıStili, ÇizgiTürü, ÖğeStili
};
use crate::renk::{Dolgu, Renk};

/// `series-sankey.orient`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SankeyYönü {
    #[default]
    Yatay,
    Dikey,
}

/// `series-sankey.nodeAlign`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SankeyDüğümHizası {
    #[default]
    İkiYana,
    Sol,
    Sağ,
}

/// Çakışma çözümünde kaynak sıranın korunup korunmayacağı (`sort`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SankeySırası {
    #[default]
    Azalan,
    Veri,
}

/// Sankey düğüm/bağ vurgusunun ilişki kapsamı (`emphasis.focus`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SankeyVurguOdağı {
    #[default]
    Yok,
    Öz,
    Seri,
    Komşuluk,
    Yörünge,
}

/// Eski `focusNodeAdjacency` seçeneğinin tam değer uzayı.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SankeyKomşulukOdağı {
    #[default]
    Kapalı,
    Gelen,
    Giden,
    Tümü,
}

/// Sankey bağ dolgusu. `source`, `target` ve `gradient` ancak yerleşimden
/// sonra uç düğüm renkleri bilindiğinde çözülebilir.
#[derive(Clone, PartialEq, Debug)]
pub enum SankeyKenarBoyası {
    Dolgu(Dolgu),
    Kaynak,
    Hedef,
    Gradyan,
}

impl From<Dolgu> for SankeyKenarBoyası {
    fn from(dolgu: Dolgu) -> Self {
        Self::Dolgu(dolgu)
    }
}

impl From<Renk> for SankeyKenarBoyası {
    fn from(renk: Renk) -> Self {
        Self::Dolgu(Dolgu::Düz(renk))
    }
}

impl From<u32> for SankeyKenarBoyası {
    fn from(renk: u32) -> Self {
        Self::from(Renk::from(renk))
    }
}

impl From<&str> for SankeyKenarBoyası {
    fn from(renk: &str) -> Self {
        match renk {
            "source" | "kaynak" => Self::Kaynak,
            "target" | "hedef" => Self::Hedef,
            "gradient" | "gradyan" => Self::Gradyan,
            diğer => Self::Dolgu(Dolgu::from(diğer)),
        }
    }
}

/// Sankey `itemStyle` yaması.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct SankeyÖğeStili {
    pub renk: Option<Dolgu>,
    pub kenarlık_rengi: Option<Renk>,
    pub kenarlık_kalınlığı: Option<f32>,
    pub kenarlık_türü: Option<ÇizgiTürü>,
    pub kenarlık_yarıçapı: Option<[f32; 4]>,
    pub opaklık: Option<f32>,
    pub gölge_bulanıklığı: Option<f32>,
    pub gölge_rengi: Option<Renk>,
    pub gölge_kayması: Option<(f32, f32)>,
}

impl SankeyÖğeStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn renk(mut self, renk: impl Into<Dolgu>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn kenarlık_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.kenarlık_rengi = Some(renk.into());
        self
    }

    pub fn kenarlık_kalınlığı(mut self, kalınlık: f32) -> Self {
        self.kenarlık_kalınlığı = kalınlık.is_finite().then(|| kalınlık.max(0.0));
        self
    }

    pub fn kenarlık_türü(mut self, tür: ÇizgiTürü) -> Self {
        self.kenarlık_türü = Some(tür);
        self
    }

    pub fn kenarlık_yarıçapı(
        mut self,
        yarıçap: impl Into<crate::model::stil::KöşeYarıçapı>,
    ) -> Self {
        self.kenarlık_yarıçapı = Some(yarıçap.into().0.map(|değer| değer.max(0.0)));
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = opaklık.is_finite().then(|| opaklık.clamp(0.0, 1.0));
        self
    }

    pub fn gölge_bulanıklığı(mut self, bulanıklık: f32) -> Self {
        self.gölge_bulanıklığı = bulanıklık.is_finite().then(|| bulanıklık.max(0.0));
        self
    }

    pub fn gölge_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.gölge_rengi = Some(renk.into());
        self
    }

    pub fn gölge_kayması(mut self, x: f32, y: f32) -> Self {
        if x.is_finite() && y.is_finite() {
            self.gölge_kayması = Some((x, y));
        }
        self
    }

    pub fn taban(mut self, stil: ÖğeStili) -> Self {
        self.renk = stil.renk;
        self.kenarlık_rengi = stil.kenarlık_rengi;
        self.kenarlık_kalınlığı = Some(stil.kenarlık_kalınlığı.max(0.0));
        self.kenarlık_türü = Some(stil.kenarlık_türü);
        self.kenarlık_yarıçapı = Some(stil.kenarlık_yarıçapı.map(|değer| değer.max(0.0)));
        self.opaklık = stil.opaklık;
        self.gölge_bulanıklığı = Some(stil.gölge_bulanıklığı.max(0.0));
        self.gölge_rengi = stil.gölge_rengi;
        self.gölge_kayması = Some(stil.gölge_kayması);
        self
    }
}

/// Sankey şeridinin doldurma/kenar/gölge seçenekleri (`lineStyle`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct SankeyÇizgiStili {
    pub renk: Option<SankeyKenarBoyası>,
    pub opaklık: Option<f32>,
    pub eğrilik: Option<f32>,
    pub kalınlık: Option<f32>,
    pub tür: Option<ÇizgiTürü>,
    pub gölge_bulanıklığı: Option<f32>,
    pub gölge_rengi: Option<Renk>,
    pub gölge_kayması: Option<(f32, f32)>,
}

impl SankeyÇizgiStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn seri_varsayılanı() -> Self {
        Self {
            renk: Some(SankeyKenarBoyası::from(Renk::onaltılık(0x86878c))),
            opaklık: Some(0.2),
            eğrilik: Some(0.5),
            kalınlık: Some(0.0),
            tür: Some(ÇizgiTürü::Düz),
            gölge_bulanıklığı: Some(0.0),
            gölge_kayması: Some((0.0, 0.0)),
            ..Self::default()
        }
    }

    pub fn renk(mut self, renk: impl Into<SankeyKenarBoyası>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = opaklık.is_finite().then(|| opaklık.clamp(0.0, 1.0));
        self
    }

    pub fn eğrilik(mut self, eğrilik: f32) -> Self {
        self.eğrilik = eğrilik.is_finite().then_some(eğrilik);
        self
    }

    pub fn kalınlık(mut self, kalınlık: f32) -> Self {
        self.kalınlık = kalınlık.is_finite().then(|| kalınlık.max(0.0));
        self
    }

    pub fn tür(mut self, tür: ÇizgiTürü) -> Self {
        self.tür = Some(tür);
        self
    }

    pub fn gölge_bulanıklığı(mut self, bulanıklık: f32) -> Self {
        self.gölge_bulanıklığı = bulanıklık.is_finite().then(|| bulanıklık.max(0.0));
        self
    }

    pub fn gölge_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.gölge_rengi = Some(renk.into());
        self
    }

    pub fn gölge_kayması(mut self, x: f32, y: f32) -> Self {
        if x.is_finite() && y.is_finite() {
            self.gölge_kayması = Some((x, y));
        }
        self
    }
}

/// Normal dışındaki Sankey durum katmanı.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct SankeyDurumu {
    pub öğe_stili: Option<SankeyÖğeStili>,
    pub çizgi_stili: Option<SankeyÇizgiStili>,
    pub etiket: Option<EtiketYaması>,
    pub kenar_etiketi: Option<EtiketYaması>,
    pub odak: Option<SankeyVurguOdağı>,
    pub devre_dışı: Option<bool>,
}

impl SankeyDurumu {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn öğe_stili(mut self, stil: SankeyÖğeStili) -> Self {
        self.öğe_stili = Some(stil);
        self
    }

    pub fn çizgi_stili(mut self, stil: SankeyÇizgiStili) -> Self {
        self.çizgi_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: EtiketYaması) -> Self {
        self.etiket = Some(etiket);
        self
    }

    pub fn kenar_etiketi(mut self, etiket: EtiketYaması) -> Self {
        self.kenar_etiketi = Some(etiket);
        self
    }

    pub fn odak(mut self, odak: SankeyVurguOdağı) -> Self {
        self.odak = Some(odak);
        self
    }

    pub fn devre_dışı(mut self, devre_dışı: bool) -> Self {
        self.devre_dışı = Some(devre_dışı);
        self
    }
}

/// `series-sankey.data[i]` / `nodes[i]`.
#[derive(Clone, PartialEq, Debug)]
pub struct SankeyDüğümü {
    pub kimlik: Option<String>,
    pub ad: String,
    pub değer: Option<f64>,
    pub yerel_x: Option<f32>,
    pub yerel_y: Option<f32>,
    pub derinlik: Option<usize>,
    pub sürüklenebilir: Option<bool>,
    pub komşuluk_odağı: Option<SankeyKomşulukOdağı>,
    pub öğe_stili: Option<SankeyÖğeStili>,
    pub etiket: Option<EtiketYaması>,
    pub vurgu: SankeyDurumu,
    pub bulanık: SankeyDurumu,
    pub seçili: SankeyDurumu,
}

impl SankeyDüğümü {
    pub fn yeni(ad: impl Into<String>) -> Self {
        Self {
            kimlik: None,
            ad: ad.into(),
            değer: None,
            yerel_x: None,
            yerel_y: None,
            derinlik: None,
            sürüklenebilir: None,
            komşuluk_odağı: None,
            öğe_stili: None,
            etiket: None,
            vurgu: SankeyDurumu::default(),
            bulanık: SankeyDurumu::default(),
            seçili: SankeyDurumu::default(),
        }
    }

    pub fn kimlik(mut self, kimlik: impl Into<String>) -> Self {
        self.kimlik = Some(kimlik.into());
        self
    }

    pub fn değer(mut self, değer: f64) -> Self {
        self.değer = değer.is_finite().then_some(değer);
        self
    }

    pub fn yerel_konum(mut self, x: f32, y: f32) -> Self {
        if x.is_finite() && y.is_finite() {
            self.yerel_x = Some(x);
            self.yerel_y = Some(y);
        }
        self
    }

    pub fn derinlik(mut self, derinlik: usize) -> Self {
        self.derinlik = Some(derinlik);
        self
    }

    pub fn sürüklenebilir(mut self, sürüklenebilir: bool) -> Self {
        self.sürüklenebilir = Some(sürüklenebilir);
        self
    }

    pub fn komşuluk_odağı(mut self, odak: SankeyKomşulukOdağı) -> Self {
        self.komşuluk_odağı = Some(odak);
        self
    }

    pub fn öğe_stili(mut self, stil: SankeyÖğeStili) -> Self {
        self.öğe_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: EtiketYaması) -> Self {
        self.etiket = Some(etiket);
        self
    }

    pub fn vurgu(mut self, durum: SankeyDurumu) -> Self {
        self.vurgu = durum;
        self
    }

    pub fn bulanık(mut self, durum: SankeyDurumu) -> Self {
        self.bulanık = durum;
        self
    }

    pub fn seçili(mut self, durum: SankeyDurumu) -> Self {
        self.seçili = durum;
        self
    }
}

impl From<&str> for SankeyDüğümü {
    fn from(ad: &str) -> Self {
        Self::yeni(ad)
    }
}

impl From<String> for SankeyDüğümü {
    fn from(ad: String) -> Self {
        Self::yeni(ad)
    }
}

/// `series-sankey.links[i]` / `edges[i]`.
#[derive(Clone, PartialEq, Debug)]
pub struct SankeyBağı {
    pub kaynak: String,
    pub hedef: String,
    pub değer: f64,
    pub çizgi_stili: Option<SankeyÇizgiStili>,
    pub kenar_etiketi: Option<EtiketYaması>,
    pub komşuluk_odağı: Option<SankeyKomşulukOdağı>,
    pub vurgu: SankeyDurumu,
    pub bulanık: SankeyDurumu,
    pub seçili: SankeyDurumu,
}

impl SankeyBağı {
    pub fn yeni(kaynak: impl Into<String>, hedef: impl Into<String>, değer: f64) -> Self {
        Self {
            kaynak: kaynak.into(),
            hedef: hedef.into(),
            değer,
            çizgi_stili: None,
            kenar_etiketi: None,
            komşuluk_odağı: None,
            vurgu: SankeyDurumu::default(),
            bulanık: SankeyDurumu::default(),
            seçili: SankeyDurumu::default(),
        }
    }

    pub fn çizgi_stili(mut self, stil: SankeyÇizgiStili) -> Self {
        self.çizgi_stili = Some(stil);
        self
    }

    pub fn kenar_etiketi(mut self, etiket: EtiketYaması) -> Self {
        self.kenar_etiketi = Some(etiket);
        self
    }

    pub fn komşuluk_odağı(mut self, odak: SankeyKomşulukOdağı) -> Self {
        self.komşuluk_odağı = Some(odak);
        self
    }

    pub fn vurgu(mut self, durum: SankeyDurumu) -> Self {
        self.vurgu = durum;
        self
    }

    pub fn bulanık(mut self, durum: SankeyDurumu) -> Self {
        self.bulanık = durum;
        self
    }

    pub fn seçili(mut self, durum: SankeyDurumu) -> Self {
        self.seçili = durum;
        self
    }
}

/// `series-sankey.levels[i]`.
#[derive(Clone, PartialEq, Debug)]
pub struct SankeySeviyesi {
    pub derinlik: usize,
    pub öğe_stili: Option<SankeyÖğeStili>,
    pub etiket: Option<EtiketYaması>,
    pub çizgi_stili: Option<SankeyÇizgiStili>,
}

impl SankeySeviyesi {
    pub fn yeni(derinlik: usize) -> Self {
        Self {
            derinlik,
            öğe_stili: None,
            etiket: None,
            çizgi_stili: None,
        }
    }

    pub fn öğe_stili(mut self, stil: SankeyÖğeStili) -> Self {
        self.öğe_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: EtiketYaması) -> Self {
        self.etiket = Some(etiket);
        self
    }

    pub fn çizgi_stili(mut self, stil: SankeyÇizgiStili) -> Self {
        self.çizgi_stili = Some(stil);
        self
    }
}

/// Sankey serisi (`series-sankey`).
#[derive(Clone, Debug)]
pub struct SankeySerisi {
    pub kimlik: Option<String>,
    pub ad: Option<String>,
    pub z: i32,
    pub sessiz: bool,
    pub renkler: Vec<Renk>,
    pub düğümler: Vec<SankeyDüğümü>,
    pub bağlar: Vec<SankeyBağı>,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub sağ: Option<Uzunluk>,
    pub alt: Option<Uzunluk>,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    pub yön: SankeyYönü,
    pub düğüm_genişliği: f32,
    pub düğüm_boşluğu: f32,
    pub sürüklenebilir: bool,
    pub yerleşim_yinelemesi: usize,
    pub sıralama: SankeySırası,
    pub düğüm_hizası: SankeyDüğümHizası,
    pub gezinme: AğaçGezinmesi,
    pub gezinme_tetikleyicisi_global: bool,
    pub merkez: Option<(Uzunluk, Uzunluk)>,
    pub yakınlaştırma: f32,
    pub en_küçük_ölçek: f32,
    pub en_büyük_ölçek: f32,
    pub öğe_stili: SankeyÖğeStili,
    pub çizgi_stili: SankeyÇizgiStili,
    pub etiket: Etiket,
    pub kenar_etiketi: Etiket,
    pub vurgu: SankeyDurumu,
    pub bulanık: SankeyDurumu,
    pub seçili: SankeyDurumu,
    pub komşuluk_odağı: SankeyKomşulukOdağı,
    pub seviyeler: Vec<SankeySeviyesi>,
    pub ipucu: Option<İpucu>,
    pub takvim_sırası: Option<usize>,
    pub takvim_koordinatı: Option<f64>,
    pub matris_sırası: Option<usize>,
    pub matris_koordinatı: Option<(MatrisAralığı, MatrisAralığı)>,
}

impl Default for SankeySerisi {
    fn default() -> Self {
        Self {
            kimlik: None,
            ad: None,
            z: 2,
            sessiz: false,
            renkler: Vec::new(),
            düğümler: Vec::new(),
            bağlar: Vec::new(),
            sol: Uzunluk::Yüzde(5.0),
            üst: Uzunluk::Yüzde(5.0),
            sağ: Some(Uzunluk::Yüzde(20.0)),
            alt: Some(Uzunluk::Yüzde(5.0)),
            genişlik: None,
            yükseklik: None,
            yön: SankeyYönü::Yatay,
            düğüm_genişliği: 20.0,
            düğüm_boşluğu: 8.0,
            sürüklenebilir: true,
            yerleşim_yinelemesi: 32,
            sıralama: SankeySırası::Azalan,
            düğüm_hizası: SankeyDüğümHizası::İkiYana,
            gezinme: AğaçGezinmesi::Kapalı,
            gezinme_tetikleyicisi_global: true,
            merkez: None,
            yakınlaştırma: 1.0,
            en_küçük_ölçek: 0.2,
            en_büyük_ölçek: 8.0,
            öğe_stili: SankeyÖğeStili::default(),
            çizgi_stili: SankeyÇizgiStili::seri_varsayılanı(),
            etiket: Etiket::yeni()
                .göster(true)
                .konum(EtiketKonumu::Sağ)
                .yazı(YazıStili::yeni().boyut(12.0)),
            kenar_etiketi: Etiket::yeni()
                .göster(false)
                .konum(EtiketKonumu::İç)
                .yazı(YazıStili::yeni().boyut(12.0)),
            vurgu: SankeyDurumu::yeni()
                .etiket(EtiketYaması::yeni().göster(true))
                .çizgi_stili(SankeyÇizgiStili::yeni().opaklık(0.5)),
            bulanık: SankeyDurumu::default(),
            seçili: SankeyDurumu::yeni()
                .öğe_stili(SankeyÖğeStili::yeni().kenarlık_rengi(Renk::onaltılık(0x3c3c41))),
            komşuluk_odağı: SankeyKomşulukOdağı::Kapalı,
            seviyeler: Vec::new(),
            ipucu: None,
            takvim_sırası: None,
            takvim_koordinatı: None,
            matris_sırası: None,
            matris_koordinatı: None,
        }
    }
}

impl SankeySerisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn kimlik(mut self, kimlik: impl Into<String>) -> Self {
        self.kimlik = Some(kimlik.into());
        self
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

    pub fn renkler(mut self, renkler: impl IntoIterator<Item = impl Into<Renk>>) -> Self {
        self.renkler = renkler.into_iter().map(Into::into).collect();
        self
    }

    pub fn düğümler<T: Into<SankeyDüğümü>>(
        mut self,
        düğümler: impl IntoIterator<Item = T>,
    ) -> Self {
        self.düğümler = düğümler.into_iter().map(Into::into).collect();
        self
    }

    pub fn bağlar<S: Into<String>>(
        mut self,
        bağlar: impl IntoIterator<Item = (S, S, f64)>,
    ) -> Self {
        self.bağlar = bağlar
            .into_iter()
            .map(|(kaynak, hedef, değer)| SankeyBağı::yeni(kaynak, hedef, değer))
            .collect();
        self
    }

    pub fn ayrıntılı_bağlar(mut self, bağlar: impl IntoIterator<Item = SankeyBağı>) -> Self {
        self.bağlar = bağlar.into_iter().collect();
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
        self.genişlik = Some(genişlik.into());
        self.sağ = None;
        self
    }

    pub fn yükseklik(mut self, yükseklik: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(yükseklik.into());
        self.alt = None;
        self
    }

    pub fn yön(mut self, yön: SankeyYönü) -> Self {
        self.yön = yön;
        self
    }

    pub fn düğüm_genişliği(mut self, genişlik: f32) -> Self {
        self.düğüm_genişliği = genişlik.max(0.0);
        self
    }

    pub fn düğüm_boşluğu(mut self, boşluk: f32) -> Self {
        self.düğüm_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn sürüklenebilir(mut self, sürüklenebilir: bool) -> Self {
        self.sürüklenebilir = sürüklenebilir;
        self
    }

    pub fn yerleşim_yinelemesi(mut self, yineleme: usize) -> Self {
        self.yerleşim_yinelemesi = yineleme;
        self
    }

    pub fn sıralama(mut self, sıralama: SankeySırası) -> Self {
        self.sıralama = sıralama;
        self
    }

    pub fn düğüm_hizası(mut self, hiza: SankeyDüğümHizası) -> Self {
        self.düğüm_hizası = hiza;
        self
    }

    pub fn gezinme(mut self, gezinme: AğaçGezinmesi) -> Self {
        self.gezinme = gezinme;
        self
    }

    /// `roamTrigger: 'global' | 'self'` karşılığı. `true`, ECharts
    /// öntanımlısı olan global işaretçi tetiklemesini seçer.
    pub fn gezinme_tetikleyicisi_global(mut self, global: bool) -> Self {
        self.gezinme_tetikleyicisi_global = global;
        self
    }

    pub fn komşuluk_odağı(mut self, odak: SankeyKomşulukOdağı) -> Self {
        self.komşuluk_odağı = odak;
        self
    }

    pub fn merkez(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.merkez = Some((x.into(), y.into()));
        self
    }

    pub fn yakınlaştırma(mut self, yakınlaştırma: f32) -> Self {
        if yakınlaştırma.is_finite() {
            self.yakınlaştırma = yakınlaştırma.max(0.01);
        }
        self
    }

    pub fn ölçek_sınırı(mut self, en_küçük: f32, en_büyük: f32) -> Self {
        if en_küçük.is_finite() && en_büyük.is_finite() && en_küçük > 0.0 && en_büyük >= en_küçük
        {
            self.en_küçük_ölçek = en_küçük;
            self.en_büyük_ölçek = en_büyük;
        }
        self
    }

    pub fn öğe_stili(mut self, stil: SankeyÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn çizgi_stili(mut self, stil: SankeyÇizgiStili) -> Self {
        self.çizgi_stili = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn kenar_etiketi(mut self, etiket: Etiket) -> Self {
        self.kenar_etiketi = etiket;
        self
    }

    pub fn vurgu(mut self, durum: SankeyDurumu) -> Self {
        self.vurgu = durum;
        self
    }

    pub fn bulanık(mut self, durum: SankeyDurumu) -> Self {
        self.bulanık = durum;
        self
    }

    pub fn seçili(mut self, durum: SankeyDurumu) -> Self {
        self.seçili = durum;
        self
    }

    pub fn seviyeler(mut self, seviyeler: impl IntoIterator<Item = SankeySeviyesi>) -> Self {
        self.seviyeler = seviyeler.into_iter().collect();
        self
    }

    pub fn ipucu(mut self, ipucu: İpucu) -> Self {
        self.ipucu = Some(ipucu);
        self
    }

    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = Some(sıra);
        self.matris_sırası = None;
        self.matris_koordinatı = None;
        self
    }

    pub fn takvim_hücresi(mut self, tarih_ms: f64) -> Self {
        self.takvim_sırası.get_or_insert(0);
        self.takvim_koordinatı = tarih_ms.is_finite().then_some(tarih_ms);
        self.matris_sırası = None;
        self.matris_koordinatı = None;
        self
    }

    pub fn matris_sırası(mut self, sıra: usize) -> Self {
        self.matris_sırası = Some(sıra);
        self.takvim_sırası = None;
        self.takvim_koordinatı = None;
        self
    }

    pub fn matris_hücresi(
        mut self,
        x: impl Into<MatrisAralığı>,
        y: impl Into<MatrisAralığı>,
    ) -> Self {
        self.matris_sırası.get_or_insert(0);
        self.matris_koordinatı = Some((x.into(), y.into()));
        self.takvim_sırası = None;
        self.takvim_koordinatı = None;
        self
    }

    /// `dragNode` action'ının model karşılığı.
    pub fn düğüm_konumunu_ayarla(
        &mut self,
        veri_sırası: usize,
        yerel_x: f32,
        yerel_y: f32,
    ) -> Option<String> {
        if !yerel_x.is_finite() || !yerel_y.is_finite() {
            return None;
        }
        let düğüm = self.düğümler.get_mut(veri_sırası)?;
        düğüm.yerel_x = Some(yerel_x);
        düğüm.yerel_y = Some(yerel_y);
        Some(düğüm.ad.clone())
    }
}
