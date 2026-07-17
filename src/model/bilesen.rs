//! Bileşen seçenekleri — `title`, `legend`, `grid`, `tooltip` tanımlarının
//! karşılığı.

use crate::model::stil::{Biçimleyici, YazıStili};
use crate::model::{Uzunluk, YatayKonum};
use crate::renk::Renk;

/// Başlık bileşeni (`title`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Başlık {
    pub metin: Option<String>,
    pub alt_metin: Option<String>,
    pub sol: YatayKonum,
    /// Üst kenardan uzaklık; öntanımlı iç boşluk kadar.
    pub üst: Option<Uzunluk>,
    pub yazı: YazıStili,
    pub alt_yazı: YazıStili,
}

impl Başlık {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn metin(mut self, metin: impl Into<String>) -> Self {
        self.metin = Some(metin.into());
        self
    }

    pub fn alt_metin(mut self, alt: impl Into<String>) -> Self {
        self.alt_metin = Some(alt.into());
        self
    }

    pub fn sol(mut self, sol: impl Into<YatayKonum>) -> Self {
        self.sol = sol.into();
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = Some(üst.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn alt_yazı(mut self, yazı: YazıStili) -> Self {
        self.alt_yazı = yazı;
        self
    }
}

/// Gösterge yerleşim yönü (`legend.orient`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Yön {
    #[default]
    Yatay,
    Dikey,
}

/// Gösterge simgesi (`legend.icon`); `None` seri türüne göre seçilir.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GöstergeSimgesi {
    YuvarlakKöşeliKare,
    Daire,
    Çizgi,
}

/// Gösterge bileşeni (`legend`).
#[derive(Clone, PartialEq, Debug)]
pub struct Gösterge {
    pub göster: bool,
    pub yön: Yön,
    pub sol: YatayKonum,
    pub üst: Option<Uzunluk>,
    /// Simge genişliği (`itemWidth`, öntanımlı 25).
    pub simge_genişliği: f32,
    /// Simge yüksekliği (`itemHeight`, öntanımlı 14).
    pub simge_yüksekliği: f32,
    /// Öğeler arası boşluk (`itemGap`, öntanımlı 10).
    pub öğe_boşluğu: f32,
    pub yazı: YazıStili,
    pub simge: Option<GöstergeSimgesi>,
    /// Yalnızca bu adlar gösterilsin (`legend.data`); boşsa hepsi.
    pub veri: Vec<String>,
    /// Sığmayan öğeler için sayfalı kaydırma (`type: 'scroll'`).
    pub kaydırılabilir: bool,
}

impl Default for Gösterge {
    fn default() -> Self {
        Gösterge {
            göster: true,
            yön: Yön::Yatay,
            sol: YatayKonum::Orta,
            üst: None,
            simge_genişliği: 25.0,
            simge_yüksekliği: 14.0,
            öğe_boşluğu: 10.0,
            yazı: YazıStili::default(),
            simge: None,
            veri: Vec::new(),
            kaydırılabilir: false,
        }
    }
}

impl Gösterge {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn yön(mut self, yön: Yön) -> Self {
        self.yön = yön;
        self
    }

    pub fn sol(mut self, sol: impl Into<YatayKonum>) -> Self {
        self.sol = sol.into();
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = Some(üst.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn simge(mut self, simge: GöstergeSimgesi) -> Self {
        self.simge = Some(simge);
        self
    }

    pub fn veri<S: Into<String>>(mut self, veri: impl IntoIterator<Item = S>) -> Self {
        self.veri = veri.into_iter().map(Into::into).collect();
        self
    }

    /// Sığmayan öğeleri sayfalı kaydırmayla gösterir (`type: 'scroll'`).
    pub fn kaydırılabilir(mut self, açık: bool) -> Self {
        self.kaydırılabilir = açık;
        self
    }
}

/// Araç kutusu (`toolbox`): şimdilik "geri yükle" düğmesi.
#[derive(Clone, PartialEq, Debug)]
pub struct AraçKutusu {
    pub göster: bool,
    /// Yakınlaştırma/gösterge durumunu ilk seçeneklere döndürür
    /// (`feature.restore`).
    pub geri_yükle: bool,
    /// Grafiği SVG dosyası olarak kaydetme düğmesi
    /// (`feature.saveAsImage` karşılığı; çıktı biçimi SVG'dir).
    pub svg_kaydet: bool,
}

impl Default for AraçKutusu {
    fn default() -> Self {
        AraçKutusu { göster: true, geri_yükle: true, svg_kaydet: false }
    }
}

impl AraçKutusu {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// SVG kaydet düğmesini açar (`saveAsImage`).
    pub fn svg_kaydet(mut self, açık: bool) -> Self {
        self.svg_kaydet = açık;
        self
    }
}

/// Fırça (`brush`): dikdörtgen seçim.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Fırça {
    pub etkin: bool,
}

impl Fırça {
    pub fn yeni() -> Self {
        Fırça { etkin: true }
    }
}

/// Kartezyen ızgara bileşeni (`grid`).
#[derive(Clone, PartialEq, Debug)]
pub struct Izgara {
    pub sol: Uzunluk,
    pub sağ: Uzunluk,
    pub üst: Uzunluk,
    pub alt: Uzunluk,
    /// Eksen etiketleri ızgara alanına dahil edilsin mi (`containLabel`)?
    pub etiketi_kapsa: bool,
}

impl Default for Izgara {
    fn default() -> Self {
        // ECharts ızgara öntanımlıları: `left/right: '10%'`, `top/bottom: 60`.
        Izgara {
            sol: Uzunluk::Yüzde(10.0),
            sağ: Uzunluk::Yüzde(10.0),
            üst: Uzunluk::Piksel(60.0),
            alt: Uzunluk::Piksel(60.0),
            etiketi_kapsa: false,
        }
    }
}

impl Izgara {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn sol(mut self, sol: impl Into<Uzunluk>) -> Self {
        self.sol = sol.into();
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = sağ.into();
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = üst.into();
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = alt.into();
        self
    }

    pub fn etiketi_kapsa(mut self, kapsa: bool) -> Self {
        self.etiketi_kapsa = kapsa;
        self
    }
}

/// İpucu tetikleme kipi (`tooltip.trigger`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tetikleme {
    /// Tek öğe üzerinde (`'item'`) — ECharts öntanımlısı.
    #[default]
    Öğe,
    /// Eksen boyunca (`'axis'`).
    Eksen,
    Kapalı,
}

/// Eksen imleci türü (`tooltip.axisPointer.type`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum İmleçTürü {
    #[default]
    Çizgi,
    Gölge,
    Çapraz,
    Yok,
}

/// İpucu bileşeni (`tooltip`).
#[derive(Clone, PartialEq, Debug)]
pub struct İpucu {
    pub göster: bool,
    pub tetikleme: Tetikleme,
    pub imleç: İmleçTürü,
    pub arkaplan: Option<Renk>,
    pub yazı: YazıStili,
    /// Değer biçimleyici (satırlardaki sayılar için).
    pub değer_biçimleyici: Option<Biçimleyici>,
    /// Satır biçimleyici (`tooltip.formatter`): `{a}` seri adı, `{b}` öğe/
    /// kategori adı, `{c}` değer. Verilirse satırlar bu şablonla yazılır.
    pub biçimleyici: Option<Biçimleyici>,
    /// Eksen imleci bağlantısı (`axisPointer.link: 'all'`): çoklu ızgarada
    /// imleç çizgisi aynı kategori sırasında tüm ızgaralarda çizilir.
    pub bağlantılı: bool,
}

impl Default for İpucu {
    fn default() -> Self {
        İpucu {
            göster: true,
            tetikleme: Tetikleme::Öğe,
            imleç: İmleçTürü::Çizgi,
            arkaplan: None,
            yazı: YazıStili::default(),
            değer_biçimleyici: None,
            biçimleyici: None,
            bağlantılı: false,
        }
    }
}

impl İpucu {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn tetikleme(mut self, tetikleme: Tetikleme) -> Self {
        self.tetikleme = tetikleme;
        self
    }

    pub fn imleç(mut self, imleç: İmleçTürü) -> Self {
        self.imleç = imleç;
        self
    }

    pub fn arkaplan(mut self, renk: impl Into<Renk>) -> Self {
        self.arkaplan = Some(renk.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn değer_biçimleyici(mut self, b: impl Into<Biçimleyici>) -> Self {
        self.değer_biçimleyici = Some(b.into());
        self
    }

    /// Satır biçimleyici (`formatter`): `{a}` seri adı, `{b}` öğe/kategori
    /// adı, `{c}` değer.
    pub fn biçimleyici(mut self, b: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(b.into());
        self
    }

    /// Eksen imlecini tüm ızgaralara bağlar (`axisPointer.link: 'all'`).
    pub fn bağlantılı(mut self, açık: bool) -> Self {
        self.bağlantılı = açık;
        self
    }
}
