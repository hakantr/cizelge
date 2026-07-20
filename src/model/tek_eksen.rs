//! Tek eksenli koordinat bileşeni — ECharts `singleAxis` karşılığı.
//!
//! Eksenin ölçek ve görünüm seçenekleri ortak [`Eksen`] modelini kullanır;
//! bu sarmalayıcı yalnız `singleAxis`e özgü kutu yerleşimini, yönü ve eksen
//! kenarını taşır.

use crate::model::Uzunluk;
use crate::model::eksen::{Eksen, EksenEtiketi, EksenKonumu, EksenTürü};
use crate::model::stil::ÇizgiTürü;

/// Tek eksenli koordinatın yerleşim yönü (`singleAxis.orient`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TekEksenYönü {
    #[default]
    Yatay,
    Dikey,
}

/// Tek eksenin koordinat kutusundaki kenarı (`singleAxis.position`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TekEksenKonumu {
    Üst,
    #[default]
    Alt,
    Sol,
    Sağ,
}

impl TekEksenKonumu {
    pub(crate) fn eksen_konumu(self) -> EksenKonumu {
        match self {
            Self::Üst => EksenKonumu::Üst,
            Self::Alt => EksenKonumu::Alt,
            Self::Sol => EksenKonumu::Sol,
            Self::Sağ => EksenKonumu::Sağ,
        }
    }

    pub(crate) fn yön(self) -> TekEksenYönü {
        match self {
            Self::Üst | Self::Alt => TekEksenYönü::Yatay,
            Self::Sol | Self::Sağ => TekEksenYönü::Dikey,
        }
    }
}

/// Tek eksenli koordinat sistemi (`singleAxis`).
#[derive(Clone, PartialEq, Debug)]
pub struct TekEksen {
    /// Ortak eksen/ölçek seçenekleri.
    pub eksen: Eksen,
    pub sol: Option<Uzunluk>,
    pub sağ: Option<Uzunluk>,
    pub üst: Option<Uzunluk>,
    pub alt: Option<Uzunluk>,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    pub yön: TekEksenYönü,
    pub konum: TekEksenKonumu,
    /// `singleAxis.splitLine.lineStyle.opacity`; ECharts öntanımlısı 0.2.
    pub bölme_çizgisi_opaklığı: f32,
    /// Bileşene bağlı eksen ipucunu açar (`singleAxis.tooltip.show`).
    pub ipucu_göster: bool,
}

impl Default for TekEksen {
    fn default() -> Self {
        Self::eksenle(Eksen::değer())
    }
}

impl TekEksen {
    fn eksen_öntanımlarını_uygula(mut eksen: Eksen) -> Eksen {
        // `singleAxis` kendi axisDefault'ını taşır: Cartesian eksenlerden
        // farklı olarak çizgi, çentik ve bölme çizgisi açık başlar.
        eksen.çizgi.göster = Some(true);
        eksen.çentik.göster = Some(true);
        eksen.çentik.uzunluk = 6.0;
        eksen.bölme_çizgisi.göster = Some(true);
        eksen.bölme_çizgisi.tür = ÇizgiTürü::Kesikli;
        eksen
    }

    /// Sayısal tek eksen (`type: 'value'`).
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Kategori tek ekseni (`type: 'category'`).
    pub fn kategori() -> Self {
        Self::eksenle(Eksen::kategori())
    }

    /// Zaman tek ekseni (`type: 'time'`).
    pub fn zaman() -> Self {
        Self::eksenle(Eksen::zaman())
    }

    /// Logaritmik tek eksen (`type: 'log'`).
    pub fn log() -> Self {
        Self::eksenle(Eksen::log())
    }

    /// Ortak eksen modelinden bir `singleAxis` oluşturur.
    pub fn eksenle(eksen: Eksen) -> Self {
        Self {
            eksen: Self::eksen_öntanımlarını_uygula(eksen),
            sol: Some(Uzunluk::Yüzde(5.0)),
            sağ: Some(Uzunluk::Yüzde(5.0)),
            üst: Some(Uzunluk::Yüzde(5.0)),
            alt: Some(Uzunluk::Yüzde(5.0)),
            genişlik: None,
            yükseklik: None,
            yön: TekEksenYönü::Yatay,
            konum: TekEksenKonumu::Alt,
            bölme_çizgisi_opaklığı: 0.2,
            ipucu_göster: true,
        }
    }

    pub fn eksen(mut self, eksen: Eksen) -> Self {
        self.eksen = Self::eksen_öntanımlarını_uygula(eksen);
        self
    }

    pub fn veri<S: Into<String>>(mut self, veri: impl IntoIterator<Item = S>) -> Self {
        self.eksen.veri = veri.into_iter().map(Into::into).collect();
        self
    }

    pub fn tür(mut self, tür: EksenTürü) -> Self {
        self.eksen.tür = tür;
        self
    }

    pub fn kenar_boşluğu(mut self, açık: bool) -> Self {
        self.eksen.kenar_boşluğu = Some(açık);
        self
    }

    pub fn etiket(mut self, etiket: EksenEtiketi) -> Self {
        self.eksen.etiket = etiket;
        self
    }

    pub fn sol(mut self, sol: impl Into<Uzunluk>) -> Self {
        self.sol = Some(sol.into());
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(sağ.into());
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = Some(üst.into());
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = Some(alt.into());
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

    /// Sağ kenarı otomatik bırakır; açık genişlikle birlikte kullanılabilir.
    pub fn sağ_otomatik(mut self) -> Self {
        self.sağ = None;
        self
    }

    /// Alt kenarı otomatik bırakır; açık yükseklikle birlikte kullanılabilir.
    pub fn alt_otomatik(mut self) -> Self {
        self.alt = None;
        self
    }

    pub fn yön(mut self, yön: TekEksenYönü) -> Self {
        self.yön = yön;
        if self.konum.yön() != yön {
            self.konum = match yön {
                TekEksenYönü::Yatay => TekEksenKonumu::Alt,
                TekEksenYönü::Dikey => TekEksenKonumu::Sol,
            };
        }
        self
    }

    pub fn konum(mut self, konum: TekEksenKonumu) -> Self {
        self.konum = konum;
        self.yön = konum.yön();
        self
    }

    pub fn bölme_çizgisi_opaklığı(mut self, opaklık: f32) -> Self {
        self.bölme_çizgisi_opaklığı = opaklık.clamp(0.0, 1.0);
        self
    }

    pub fn ipucu_göster(mut self, göster: bool) -> Self {
        self.ipucu_göster = göster;
        self
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn resmi_ontanimlari_tasir() {
        let eksen = TekEksen::kategori();
        assert_eq!(eksen.sol, Some(Uzunluk::Yüzde(5.0)));
        assert_eq!(eksen.sağ, Some(Uzunluk::Yüzde(5.0)));
        assert_eq!(eksen.eksen.çizgi.göster, Some(true));
        assert_eq!(eksen.eksen.çentik.göster, Some(true));
        assert_eq!(eksen.eksen.çentik.uzunluk, 6.0);
        assert_eq!(eksen.eksen.bölme_çizgisi.göster, Some(true));
        assert_eq!(eksen.eksen.bölme_çizgisi.tür, ÇizgiTürü::Kesikli);
        assert_eq!(eksen.bölme_çizgisi_opaklığı, 0.2);
    }

    #[test]
    fn konum_yonu_birlikte_degistirir() {
        let eksen = TekEksen::yeni().konum(TekEksenKonumu::Sağ);
        assert_eq!(eksen.yön, TekEksenYönü::Dikey);
        let eksen = eksen.yön(TekEksenYönü::Yatay);
        assert_eq!(eksen.konum, TekEksenKonumu::Alt);
    }
}
