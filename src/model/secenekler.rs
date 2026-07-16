//! Grafik seçenekleri — ECharts'taki kök `option` nesnesinin karşılığı.

use crate::animasyon::{Yumuşatma, ÖNTANIMLI_SÜRE_MS};
use crate::model::bilesen::{Başlık, Gösterge, Izgara, İpucu};
use crate::model::eksen::Eksen;
use crate::model::seri::Seri;
use crate::renk::Renk;
use crate::tema;

/// Kök grafik seçenekleri (`EChartsOption`).
#[derive(Clone, Debug)]
pub struct GrafikSeçenekleri {
    pub başlık: Option<Başlık>,
    pub gösterge: Option<Gösterge>,
    pub ızgara: Izgara,
    pub x_ekseni: Option<Eksen>,
    pub y_ekseni: Option<Eksen>,
    pub seriler: Vec<Seri>,
    pub ipucu: Option<İpucu>,
    /// Seri renk paleti (`color`).
    pub palet: Vec<Renk>,
    pub arkaplan: Option<Renk>,
    pub animasyon: bool,
    /// Giriş animasyonu süresi, ms (`animationDuration`).
    pub animasyon_süresi: f32,
    pub animasyon_eğrisi: Yumuşatma,
}

impl Default for GrafikSeçenekleri {
    fn default() -> Self {
        GrafikSeçenekleri {
            başlık: None,
            gösterge: None,
            ızgara: Izgara::default(),
            x_ekseni: None,
            y_ekseni: None,
            seriler: Vec::new(),
            ipucu: None,
            palet: tema::PALET.to_vec(),
            arkaplan: None,
            animasyon: true,
            animasyon_süresi: ÖNTANIMLI_SÜRE_MS,
            animasyon_eğrisi: Yumuşatma::KübikÇıkış,
        }
    }
}

impl GrafikSeçenekleri {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn başlık(mut self, başlık: Başlık) -> Self {
        self.başlık = Some(başlık);
        self
    }

    pub fn gösterge(mut self, gösterge: Gösterge) -> Self {
        self.gösterge = Some(gösterge);
        self
    }

    pub fn ızgara(mut self, ızgara: Izgara) -> Self {
        self.ızgara = ızgara;
        self
    }

    pub fn x_ekseni(mut self, eksen: Eksen) -> Self {
        self.x_ekseni = Some(eksen);
        self
    }

    pub fn y_ekseni(mut self, eksen: Eksen) -> Self {
        self.y_ekseni = Some(eksen);
        self
    }

    pub fn seri(mut self, seri: impl Into<Seri>) -> Self {
        self.seriler.push(seri.into());
        self
    }

    pub fn seriler<S: Into<Seri>>(mut self, seriler: impl IntoIterator<Item = S>) -> Self {
        self.seriler.extend(seriler.into_iter().map(Into::into));
        self
    }

    pub fn ipucu(mut self, ipucu: İpucu) -> Self {
        self.ipucu = Some(ipucu);
        self
    }

    pub fn palet<R: Into<Renk>>(mut self, renkler: impl IntoIterator<Item = R>) -> Self {
        self.palet = renkler.into_iter().map(Into::into).collect();
        self
    }

    pub fn arkaplan(mut self, renk: impl Into<Renk>) -> Self {
        self.arkaplan = Some(renk.into());
        self
    }

    pub fn animasyon(mut self, açık: bool) -> Self {
        self.animasyon = açık;
        self
    }

    pub fn animasyon_süresi(mut self, ms: f32) -> Self {
        self.animasyon_süresi = ms;
        self
    }

    pub fn animasyon_eğrisi(mut self, eğri: Yumuşatma) -> Self {
        self.animasyon_eğrisi = eğri;
        self
    }

    /// Serinin paletten çözülen rengi (`itemStyle.color` öncelikli).
    pub fn seri_rengi(&self, sıra: usize) -> Renk {
        self.seriler
            .get(sıra)
            .and_then(|s| s.açık_renk())
            .map(|d| d.temsilî())
            .unwrap_or_else(|| {
                if self.palet.is_empty() {
                    tema::palet_rengi(sıra)
                } else {
                    self.palet[sıra % self.palet.len()]
                }
            })
    }

    /// Paletten sıra numarasıyla renk (pasta dilimleri gibi öğe-bazlı
    /// renklendirme için).
    pub fn palet_rengi(&self, sıra: usize) -> Renk {
        if self.palet.is_empty() {
            tema::palet_rengi(sıra)
        } else {
            self.palet[sıra % self.palet.len()]
        }
    }
}
