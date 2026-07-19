//! ECharts `calendar` koordinat bileşeninin option modeli.

use crate::model::Uzunluk;
use crate::model::stil::{Etiket, ÇizgiStili, ÖğeStili};
use crate::renk::Renk;
use crate::yardimci::takvim::{TakvimAnı, takvimden_ana};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TakvimYönü {
    #[default]
    Yatay,
    Dikey,
}

/// Takvim aralığı; iki uç da kapsanır ve Unix milisaniyesidir.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TakvimAralığı {
    pub başlangıç_ms: f64,
    pub bitiş_ms: f64,
}

impl TakvimAralığı {
    pub fn yeni(başlangıç_ms: f64, bitiş_ms: f64) -> Self {
        Self {
            başlangıç_ms,
            bitiş_ms,
        }
    }

    pub fn yıl(yıl: i32) -> Self {
        let başlangıç_ms = takvimden_ana(TakvimAnı {
            yıl,
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let bitiş_ms = takvimden_ana(TakvimAnı {
            yıl: yıl.saturating_add(1),
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        }) - 1.0;
        Self::yeni(başlangıç_ms, bitiş_ms)
    }
}

impl Default for TakvimAralığı {
    fn default() -> Self {
        Self::yıl(2026)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TakvimKoordinatı {
    pub aralık: TakvimAralığı,
    pub sol: Uzunluk,
    pub üst: Uzunluk,
    pub genişlik: Uzunluk,
    pub yükseklik: Uzunluk,
    pub yön: TakvimYönü,
    /// Haftanın ilk günü: `0=Pazartesi .. 6=Pazar`.
    pub ilk_gün: usize,
    /// `None` ilgili boyutu mevcut alana otomatik sığdırır.
    pub hücre_genişliği: Option<f32>,
    pub hücre_yüksekliği: Option<f32>,
    pub hücre_boşluğu: f32,
    pub gün_etiketi: Etiket,
    pub ay_etiketi: Etiket,
    pub yıl_etiketi: Etiket,
    pub öğe_stili: ÖğeStili,
    pub ayırıcı: ÇizgiStili,
    pub sessiz: bool,
}

impl Default for TakvimKoordinatı {
    fn default() -> Self {
        Self {
            aralık: TakvimAralığı::default(),
            sol: Uzunluk::Yüzde(4.0),
            üst: Uzunluk::Piksel(60.0),
            genişlik: Uzunluk::Yüzde(92.0),
            yükseklik: Uzunluk::Piksel(190.0),
            yön: TakvimYönü::Yatay,
            ilk_gün: 0,
            hücre_genişliği: None,
            hücre_yüksekliği: None,
            hücre_boşluğu: 2.0,
            gün_etiketi: Etiket::yeni().göster(true),
            ay_etiketi: Etiket::yeni().göster(true),
            yıl_etiketi: Etiket::default(),
            öğe_stili: ÖğeStili::yeni()
                .kenarlık_rengi(Renk::onaltılık(0xcccccc))
                .kenarlık_kalınlığı(1.0),
            ayırıcı: ÇizgiStili::yeni()
                .renk(Renk::onaltılık(0xaaaaaa))
                .kalınlık(1.0),
            sessiz: true,
        }
    }
}

impl TakvimKoordinatı {
    pub fn yeni(aralık: TakvimAralığı) -> Self {
        Self {
            aralık,
            ..Self::default()
        }
    }

    pub fn yıl(yıl: i32) -> Self {
        Self::yeni(TakvimAralığı::yıl(yıl))
    }

    pub fn aralık(mut self, aralık: TakvimAralığı) -> Self {
        self.aralık = aralık;
        self
    }

    pub fn yön(mut self, yön: TakvimYönü) -> Self {
        self.yön = yön;
        self
    }

    pub fn ilk_gün(mut self, ilk_gün: usize) -> Self {
        self.ilk_gün = ilk_gün % 7;
        self
    }

    pub fn konum(
        mut self,
        sol: impl Into<Uzunluk>,
        üst: impl Into<Uzunluk>,
        genişlik: impl Into<Uzunluk>,
        yükseklik: impl Into<Uzunluk>,
    ) -> Self {
        self.sol = sol.into();
        self.üst = üst.into();
        self.genişlik = genişlik.into();
        self.yükseklik = yükseklik.into();
        self
    }

    pub fn hücre_boyutu(mut self, genişlik: Option<f32>, yükseklik: Option<f32>) -> Self {
        self.hücre_genişliği = genişlik;
        self.hücre_yüksekliği = yükseklik;
        self
    }
}
