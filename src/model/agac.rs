//! Hiyerarşik veri modeli — ECharts `data/Tree.ts`'in sadeleştirilmiş
//! karşılığı; ağaç haritası (treemap), güneş patlaması (sunburst) ve ağaç
//! (tree) serilerinin ortak veri yapısı.

use crate::renk::Renk;

/// Hiyerarşideki tek düğüm.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AğaçDüğümü {
    pub ad: String,
    /// Yaprak değeri; dallarda `None` ise çocuk toplamı kullanılır.
    pub değer: Option<f64>,
    pub çocuklar: Vec<AğaçDüğümü>,
    /// Açık renk; verilmezse üst düzey paletten, alt düzeyler üstten türetilir.
    pub renk: Option<Renk>,
}

impl AğaçDüğümü {
    /// Yaprak düğüm.
    pub fn yaprak(ad: impl Into<String>, değer: f64) -> Self {
        AğaçDüğümü { ad: ad.into(), değer: Some(değer), ..Default::default() }
    }

    /// Dal düğümü (değeri çocuk toplamından türeyen).
    pub fn dal(ad: impl Into<String>, çocuklar: Vec<AğaçDüğümü>) -> Self {
        AğaçDüğümü { ad: ad.into(), çocuklar, ..Default::default() }
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
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
    kökler: &'a [AğaçDüğümü],
    yol: &[String],
) -> (&'a [AğaçDüğümü], usize) {
    let mut etkin = kökler;
    let mut derinlik = 0usize;
    for ad in yol {
        let Some(düğüm) = etkin.iter().find(|d| &d.ad == ad) else { break };
        if düğüm.çocuklar.is_empty() {
            break;
        }
        etkin = &düğüm.çocuklar;
        derinlik = derinlik.saturating_add(1);
    }
    (etkin, derinlik)
}
