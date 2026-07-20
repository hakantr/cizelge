//! ECharts `calendar` koordinat bileşeninin option modeli.

use crate::model::stil::{Etiket, YazıStili, ÇizgiStili, ÖğeStili};
use crate::model::{DikeyKonum, Uzunluk, YatayKonum};
use crate::yardimci::takvim::{TakvimAnı, takvimden_ana};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TakvimYönü {
    #[default]
    Yatay,
    Dikey,
}

/// Gün ve ay etiketlerinin takvim gövdesinin hangi tarafında yer aldığı
/// (`dayLabel.position` / `monthLabel.position`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TakvimEtiketTarafı {
    #[default]
    Başlangıç,
    Bitiş,
}

/// Yıl etiketinin takvim gövdesine göre konumu (`yearLabel.position`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TakvimYılEtiketiKonumu {
    /// Yatay takvimde sol, dikey takvimde üst.
    #[default]
    Otomatik,
    Üst,
    Alt,
    Sol,
    Sağ,
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
    pub sol: Option<YatayKonum>,
    pub sağ: Option<Uzunluk>,
    pub üst: Option<DikeyKonum>,
    pub alt: Option<Uzunluk>,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    pub yön: TakvimYönü,
    /// Haftanın ilk günü: ECharts ile aynı biçimde `0=Pazar .. 6=Cumartesi`.
    pub ilk_gün: usize,
    /// `None` ilgili boyutu mevcut alana otomatik sığdırır.
    pub hücre_genişliği: Option<f32>,
    pub hücre_yüksekliği: Option<f32>,
    pub hücre_boşluğu: f32,
    pub gün_etiketi: Etiket,
    pub gün_etiketi_tarafı: TakvimEtiketTarafı,
    /// Hücre boyutuna göre çözülebilen `dayLabel.margin`.
    pub gün_etiketi_kenar_boşluğu: Uzunluk,
    /// `dayLabel.nameMap`; boşsa etkin yerelin tek harfli gün adları.
    pub gün_adları: Option<Vec<String>>,
    pub ay_etiketi: Etiket,
    pub ay_etiketi_tarafı: TakvimEtiketTarafı,
    pub ay_etiketi_kenar_boşluğu: f32,
    pub ay_etiketi_ortala: bool,
    /// `monthLabel.nameMap`; boşsa etkin yerelin ay kısaltmaları.
    pub ay_adları: Option<Vec<String>>,
    pub yıl_etiketi: Etiket,
    pub yıl_etiketi_konumu: TakvimYılEtiketiKonumu,
    pub yıl_etiketi_kenar_boşluğu: f32,
    pub öğe_stili: ÖğeStili,
    pub ayırıcı: ÇizgiStili,
    pub sessiz: bool,
}

impl Default for TakvimKoordinatı {
    fn default() -> Self {
        Self {
            aralık: TakvimAralığı::default(),
            sol: Some(YatayKonum::Değer(Uzunluk::Piksel(80.0))),
            sağ: None,
            üst: Some(DikeyKonum::Değer(Uzunluk::Piksel(60.0))),
            alt: None,
            genişlik: None,
            yükseklik: None,
            yön: TakvimYönü::Yatay,
            ilk_gün: 0,
            hücre_genişliği: Some(20.0),
            hücre_yüksekliği: Some(20.0),
            hücre_boşluğu: 0.0,
            gün_etiketi: Etiket::yeni().göster(true),
            gün_etiketi_tarafı: TakvimEtiketTarafı::Başlangıç,
            gün_etiketi_kenar_boşluğu: Uzunluk::Piksel(10.0),
            gün_adları: None,
            ay_etiketi: Etiket::yeni().göster(true),
            ay_etiketi_tarafı: TakvimEtiketTarafı::Başlangıç,
            ay_etiketi_kenar_boşluğu: 10.0,
            ay_etiketi_ortala: true,
            ay_adları: None,
            yıl_etiketi: Etiket::yeni()
                .göster(true)
                .yazı(YazıStili::yeni().boyut(20.0).kalın(true)),
            yıl_etiketi_konumu: TakvimYılEtiketiKonumu::Otomatik,
            yıl_etiketi_kenar_boşluğu: 30.0,
            // Renkler çizim anında tema belirteçlerinden çözülür.
            öğe_stili: ÖğeStili::yeni().kenarlık_kalınlığı(1.0),
            ayırıcı: ÇizgiStili::yeni().kalınlık(1.0),
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
        self.sol = Some(YatayKonum::Değer(sol.into()));
        self.sağ = None;
        self.üst = Some(DikeyKonum::Değer(üst.into()));
        self.alt = None;
        self.genişlik = Some(genişlik.into());
        self.yükseklik = Some(yükseklik.into());
        // ECharts, açık width/height verildiğinde ilgili cellSize boyutunu
        // `auto` olarak normalleştirir.
        self.hücre_genişliği = None;
        self.hücre_yüksekliği = None;
        self
    }

    pub fn sol(mut self, sol: impl Into<YatayKonum>) -> Self {
        self.sol = Some(sol.into());
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(sağ.into());
        self
    }

    pub fn üst(mut self, üst: impl Into<DikeyKonum>) -> Self {
        self.üst = Some(üst.into());
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    pub fn genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(genişlik.into());
        self.hücre_genişliği = None;
        self
    }

    pub fn yükseklik(mut self, yükseklik: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(yükseklik.into());
        self.hücre_yüksekliği = None;
        self
    }

    pub fn hücre_boyutu(mut self, genişlik: Option<f32>, yükseklik: Option<f32>) -> Self {
        self.hücre_genişliği = genişlik;
        self.hücre_yüksekliği = yükseklik;
        self
    }

    pub fn hücre_boşluğu(mut self, boşluk: f32) -> Self {
        self.hücre_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn gün_etiketi(mut self, etiket: Etiket) -> Self {
        self.gün_etiketi = etiket;
        self
    }

    pub fn gün_etiketi_tarafı(mut self, taraf: TakvimEtiketTarafı) -> Self {
        self.gün_etiketi_tarafı = taraf;
        self
    }

    pub fn gün_etiketi_kenar_boşluğu(mut self, boşluk: impl Into<Uzunluk>) -> Self {
        self.gün_etiketi_kenar_boşluğu = boşluk.into();
        self
    }

    pub fn gün_adları<S: Into<String>>(mut self, adlar: impl IntoIterator<Item = S>) -> Self {
        self.gün_adları = Some(adlar.into_iter().map(Into::into).collect());
        self
    }

    pub fn ay_etiketi(mut self, etiket: Etiket) -> Self {
        self.ay_etiketi = etiket;
        self
    }

    pub fn ay_etiketi_tarafı(mut self, taraf: TakvimEtiketTarafı) -> Self {
        self.ay_etiketi_tarafı = taraf;
        self
    }

    pub fn ay_etiketi_kenar_boşluğu(mut self, boşluk: f32) -> Self {
        self.ay_etiketi_kenar_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn ay_etiketi_ortala(mut self, ortala: bool) -> Self {
        self.ay_etiketi_ortala = ortala;
        self
    }

    pub fn ay_adları<S: Into<String>>(mut self, adlar: impl IntoIterator<Item = S>) -> Self {
        self.ay_adları = Some(adlar.into_iter().map(Into::into).collect());
        self
    }

    pub fn yıl_etiketi(mut self, etiket: Etiket) -> Self {
        self.yıl_etiketi = etiket;
        self
    }

    pub fn yıl_etiketi_konumu(mut self, konum: TakvimYılEtiketiKonumu) -> Self {
        self.yıl_etiketi_konumu = konum;
        self
    }

    pub fn yıl_etiketi_kenar_boşluğu(mut self, boşluk: f32) -> Self {
        self.yıl_etiketi_kenar_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn ayırıcı(mut self, stil: ÇizgiStili) -> Self {
        self.ayırıcı = stil;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }
}
