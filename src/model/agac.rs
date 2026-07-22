//! Hiyerarşik veri modeli — ECharts `data/Tree.ts`'in sadeleştirilmiş
//! karşılığı; ağaç haritası (treemap), güneş patlaması (sunburst) ve ağaç
//! (tree) serilerinin ortak veri yapısı.

use crate::model::Uzunluk;
use crate::model::seri::Sembol;
use crate::model::stil::{EtiketYaması, ÇizgiStili, ÖğeStili};
use crate::renk::Renk;

/// Tree yerleşimi (`series-tree.layout`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçYerleşimi {
    /// ECharts `'orthogonal'`.
    #[default]
    Dik,
    /// ECharts `'radial'`.
    Radyal,
}

/// Dik Tree yerleşiminin büyüme yönü (`series-tree.orient`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçYönü {
    /// ECharts `'LR'` / geriye uyumlu `'horizontal'`.
    #[default]
    SoldanSağa,
    /// ECharts `'RL'`.
    SağdanSola,
    /// ECharts `'TB'` / geriye uyumlu `'vertical'`.
    ÜsttenAlta,
    /// ECharts `'BT'`.
    AlttanÜste,
}

/// Tree üst-çocuk bağı geometrisi (`series-tree.edgeShape`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçKenarBiçimi {
    /// İki kontrol noktalı Bezier (`'curve'`).
    #[default]
    Eğri,
    /// Ortak çatallı dik parçalar (`'polyline'`).
    Kırık,
}

/// Tree görünümünde izin verilen gezinme (`series-tree.roam`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçGezinmesi {
    #[default]
    Kapalı,
    Açık,
    Kaydır,
    Ölçekle,
}

impl AğaçGezinmesi {
    pub fn kaydırılabilir(self) -> bool {
        matches!(self, Self::Açık | Self::Kaydır)
    }

    pub fn ölçeklenebilir(self) -> bool {
        matches!(self, Self::Açık | Self::Ölçekle)
    }
}

/// Vurguda ilişkili düğüm kümesi (`emphasis.focus`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçVurguOdağı {
    #[default]
    Yok,
    Ata,
    AltSoy,
    İlişkili,
    Öz,
}

/// Hiyerarşideki tek düğüm.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AğaçDüğümü {
    /// Kararlı veri kimliği (`data[i].id`); diff sırasında add/update/remove
    /// eşlemesinin ad değişikliğinden bağımsız kalmasını sağlar.
    pub kimlik: Option<String>,
    pub ad: String,
    /// Yaprak değeri; dallarda `None` ise çocuk toplamı kullanılır.
    pub değer: Option<f64>,
    pub çocuklar: Vec<AğaçDüğümü>,
    /// Açık renk; verilmezse üst düzey paletten, alt düzeyler üstten türetilir.
    pub renk: Option<Renk>,
    /// Düğümün ilk Tree görünümünde kapalı olup olmadığı. `None`, serinin
    /// `ilk_ağaç_derinliği` kararını kullanır.
    pub daraltılmış: Option<bool>,
    pub kategori: Option<usize>,
    pub bağlantı: Option<String>,
    pub hedef: Option<String>,
    pub sembol: Option<Sembol>,
    pub sembol_boyutu: Option<f32>,
    pub sembol_döndürme: Option<f32>,
    pub sembol_kayması: Option<(Uzunluk, Uzunluk)>,
    pub sembol_oranını_koru: Option<bool>,
    pub öğe_stili: Option<ÖğeStili>,
    /// Tree'de bu düğümü üstüne bağlayan kenarın stili.
    pub çizgi_stili: Option<ÇizgiStili>,
    pub etiket: Option<EtiketYaması>,
    pub vurgu_öğe_stili: Option<ÖğeStili>,
    pub vurgu_çizgi_stili: Option<ÇizgiStili>,
    pub vurgu_etiketi: Option<EtiketYaması>,
    pub bulanık_öğe_stili: Option<ÖğeStili>,
    pub bulanık_çizgi_stili: Option<ÇizgiStili>,
    pub bulanık_etiketi: Option<EtiketYaması>,
    pub seçili_öğe_stili: Option<ÖğeStili>,
    pub seçili_çizgi_stili: Option<ÇizgiStili>,
    pub seçili_etiketi: Option<EtiketYaması>,
}

impl AğaçDüğümü {
    /// Yaprak düğüm.
    pub fn yaprak(ad: impl Into<String>, değer: f64) -> Self {
        AğaçDüğümü {
            ad: ad.into(),
            değer: Some(değer),
            ..Default::default()
        }
    }

    /// Dal düğümü (değeri çocuk toplamından türeyen).
    pub fn dal(ad: impl Into<String>, çocuklar: Vec<AğaçDüğümü>) -> Self {
        AğaçDüğümü {
            ad: ad.into(),
            çocuklar,
            ..Default::default()
        }
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn kimlik(mut self, kimlik: impl Into<String>) -> Self {
        self.kimlik = Some(kimlik.into());
        self
    }

    pub fn daraltılmış(mut self, daraltılmış: bool) -> Self {
        self.daraltılmış = Some(daraltılmış);
        self
    }

    pub fn kategori(mut self, kategori: usize) -> Self {
        self.kategori = Some(kategori);
        self
    }

    pub fn bağlantı(mut self, bağlantı: impl Into<String>) -> Self {
        self.bağlantı = Some(bağlantı.into());
        self
    }

    pub fn hedef(mut self, hedef: impl Into<String>) -> Self {
        self.hedef = Some(hedef.into());
        self
    }

    pub fn sembol(mut self, sembol: Sembol) -> Self {
        self.sembol = Some(sembol);
        self
    }

    pub fn sembol_boyutu(mut self, boyut: f32) -> Self {
        self.sembol_boyutu = Some(boyut.max(0.0));
        self
    }

    pub fn sembol_döndürme(mut self, derece: f32) -> Self {
        self.sembol_döndürme = derece.is_finite().then_some(derece);
        self
    }

    pub fn sembol_kayması(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.sembol_kayması = Some((x.into(), y.into()));
        self
    }

    pub fn sembol_oranını_koru(mut self, koru: bool) -> Self {
        self.sembol_oranını_koru = Some(koru);
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = Some(stil);
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.etiket = Some(etiket.into());
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

    pub fn değerli(mut self, değer: f64) -> Self {
        self.değer = Some(değer);
        self
    }

    /// Etkin değer: verilmişse kendisi, yoksa çocukların toplamı.
    pub fn etkin_değer(&self) -> f64 {
        match self.değer {
            Some(d) if d.is_finite() => d,
            _ => self.çocuklar.iter().map(|ç| ç.etkin_değer()).sum(),
        }
    }

    pub fn yaprak_mı(&self) -> bool {
        self.çocuklar.is_empty()
    }
}

/// Ad zincirini (kök yolu) izleyerek etkin kök listesini bulur — ağaç
/// haritası inme (drill-down) ve güneş patlaması odak gezinmesi için.
/// Ad bulunamazsa ya da bulunan düğüm yapraksa iniş orada durur.
/// Dönen ikinci değer: gerçekten inilen adım sayısıdır.
pub fn yolu_çöz<'a>(
    kökler: &'a [AğaçDüğümü], yol: &[String]
) -> (&'a [AğaçDüğümü], usize) {
    let mut etkin = kökler;
    let mut derinlik = 0usize;
    for ad in yol {
        let Some(düğüm) = etkin.iter().find(|d| &d.ad == ad) else {
            break;
        };
        if düğüm.çocuklar.is_empty() {
            break;
        }
        etkin = &düğüm.çocuklar;
        derinlik = derinlik.saturating_add(1);
    }
    (etkin, derinlik)
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::model::stil::EtiketYaması;

    #[test]
    fn tree_dugumu_resmi_veri_sembol_ve_durum_alanlarini_korur() {
        let düğüm = AğaçDüğümü::dal("root", vec![AğaçDüğümü::yaprak("leaf", 3.0)])
            .kimlik("node-0")
            .daraltılmış(true)
            .kategori(2)
            .bağlantı("https://example.invalid")
            .hedef("_blank")
            .sembol(Sembol::Kare)
            .sembol_boyutu(18.0)
            .sembol_döndürme(30.0)
            .sembol_kayması("25%", -2)
            .sembol_oranını_koru(true)
            .öğe_stili(ÖğeStili::yeni().renk("#123456"))
            .çizgi_stili(ÇizgiStili::yeni().kalınlık(3.0))
            .etiket(EtiketYaması::yeni().göster(true))
            .vurgu_etiketi(EtiketYaması::yeni().göster(false))
            .bulanık_öğe_stili(ÖğeStili::yeni().opaklık(0.2))
            .seçili_çizgi_stili(ÇizgiStili::yeni().kalınlık(5.0));

        assert_eq!(düğüm.kimlik.as_deref(), Some("node-0"));
        assert_eq!(düğüm.daraltılmış, Some(true));
        assert_eq!(düğüm.kategori, Some(2));
        assert_eq!(düğüm.bağlantı.as_deref(), Some("https://example.invalid"));
        assert_eq!(düğüm.hedef.as_deref(), Some("_blank"));
        assert_eq!(düğüm.sembol, Some(Sembol::Kare));
        assert_eq!(düğüm.sembol_boyutu, Some(18.0));
        assert_eq!(düğüm.sembol_döndürme, Some(30.0));
        assert_eq!(
            düğüm.sembol_kayması,
            Some((Uzunluk::Yüzde(25.0), Uzunluk::Piksel(-2.0)))
        );
        assert_eq!(düğüm.sembol_oranını_koru, Some(true));
        assert_eq!(düğüm.çocuklar.len(), 1);
        assert_eq!(düğüm.etkin_değer(), 3.0);
        assert!(düğüm.öğe_stili.is_some());
        assert!(düğüm.çizgi_stili.is_some());
        assert!(düğüm.etiket.is_some());
        assert!(düğüm.vurgu_etiketi.is_some());
        assert!(düğüm.bulanık_öğe_stili.is_some());
        assert!(düğüm.seçili_çizgi_stili.is_some());
    }
}
