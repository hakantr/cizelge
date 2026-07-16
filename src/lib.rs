//! # çizelge
//!
//! [Apache ECharts](https://echarts.apache.org)'ın, Zed editörünün arayüz
//! çatısı olan [gpui](https://gpui.rs) üzerinde çalışan yerli Rust uyarlaması.
//!
//! Modül düzeni ECharts kaynak ağacını (`src/scale`, `src/coord`, `src/model`,
//! `src/chart`, `src/component`) birebir izler; çekirdek sayısal algoritmalar
//! ("güzel" çentik üretimi, sütun genişlik/kaydırma yerleşimi, yumuşak eğri
//! kontrol noktaları, yığınlama) ilgili ECharts gerçeklemelerinin doğrudan
//! portudur.
//!
//! ```ignore
//! use cizelge::hazir::*;
//!
//! let seçenekler = GrafikSeçenekleri::yeni()
//!     .başlık(Başlık::yeni().metin("Örnek"))
//!     .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
//!     .x_ekseni(Eksen::kategori().veri(["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"]))
//!     .y_ekseni(Eksen::değer())
//!     .seri(ÇizgiSerisi::yeni().veri([150, 230, 224, 218, 135, 147, 260]));
//! ```
#![allow(uncommon_codepoints)]
#![allow(mixed_script_confusables)]
#![allow(confusable_idents)]

pub mod animasyon;
pub mod bilesen;
pub mod cizim;
pub mod grafik;
pub mod hata;
pub mod koordinat;
pub mod model;
pub mod olcek;
pub mod renk;
pub mod tema;
pub mod yardimci;
pub mod yerlesim;

pub use cizim::gorunum::{grafiği_boya, BoyamaÇıktısı, GrafikGörünümü};
pub use cizim::{
    GrafikOlayı, KayıtYüzeyi, ÇizimYüzeyi, İsabetBölgesi, İsabetGeometrisi,
};
pub use hata::{BilesenHatasi, BilesenTanisi};
pub use model::bilesen::{Başlık, Gösterge, Izgara, Tetikleme, Yön, İmleçTürü, İpucu};
pub use model::deger::{VeriDeğeri, VeriÖğesi};
pub use model::eksen::{
    AraÇentik, BölmeAlanı, BölmeÇizgisi, Eksen, EksenEtiketi, EksenKonumu, EksenTürü,
    EksenÇentiği, EksenÇizgisi,
};
pub use model::imleyici::{
    İmAlanı, İmAlanıTanımı, İmDeğeri, İmNoktası, İmNoktasıTanımı, İmYönü, İmleyiciler,
    İmÇizgisi, İmÇizgisiTanımı,
};
pub use model::gorsel_esleme::GörselEşleme;
pub use model::radar::{RadarGöstergesi, RadarKoordinatı, RadarŞekli};
pub use model::secenekler::GrafikSeçenekleri;
pub use model::seri::{
    Basamak, GülTürü, GöstergeSaatiSerisi, HuniSerisi, HuniSıralaması, RadarSerisi,
    IsıHaritasıSerisi, KutuSerisi, MumSerisi, PastaSerisi,
    SaçılımSerisi, Sembol, Seri,
    SütunSerisi, ÇizgiSerisi,
};
pub use model::stil::{
    AlanStili, Biçimleyici, Etiket, EtiketKonumu, YazıStili, ÇizgiStili, ÇizgiTürü, ÖğeStili,
};
pub use model::Uzunluk;
pub use renk::{Dolgu, Renk, RenkDurağı};

/// Sık kullanılan tiplerin topluca içe aktarımı (ECharts'taki `echarts` ana
/// girişinin karşılığı).
pub mod hazir {
    pub use crate::cizim::gorunum::{grafiği_boya, BoyamaÇıktısı, GrafikGörünümü};
    pub use crate::cizim::{GrafikOlayı, KayıtYüzeyi, ÇizimYüzeyi};
    pub use crate::hata::{BilesenHatasi, BilesenTanisi};
    pub use crate::model::bilesen::{
        Başlık, Gösterge, Izgara, Tetikleme, Yön, İmleçTürü, İpucu,
    };
    pub use crate::model::deger::{VeriDeğeri, VeriÖğesi};
    pub use crate::model::eksen::{
        AraÇentik, BölmeAlanı, BölmeÇizgisi, Eksen, EksenEtiketi, EksenKonumu, EksenTürü,
        EksenÇentiği, EksenÇizgisi,
    };
    pub use crate::model::imleyici::{
        İmAlanı, İmAlanıTanımı, İmDeğeri, İmNoktası, İmNoktasıTanımı, İmYönü, İmleyiciler,
        İmÇizgisi, İmÇizgisiTanımı,
    };
    pub use crate::model::gorsel_esleme::GörselEşleme;
    pub use crate::model::radar::{RadarGöstergesi, RadarKoordinatı, RadarŞekli};
    pub use crate::model::secenekler::GrafikSeçenekleri;
    pub use crate::model::seri::{
        Basamak, GülTürü, GöstergeSaatiSerisi, HuniSerisi, HuniSıralaması, RadarSerisi,
    IsıHaritasıSerisi, KutuSerisi, MumSerisi, PastaSerisi,
    SaçılımSerisi, Sembol, Seri,
    SütunSerisi, ÇizgiSerisi,
    };
    pub use crate::model::stil::{
        AlanStili, Biçimleyici, Etiket, EtiketKonumu, YazıStili, ÇizgiStili, ÇizgiTürü, ÖğeStili,
    };
    pub use crate::model::Uzunluk;
    pub use crate::renk::{Dolgu, Renk, RenkDurağı};
}
