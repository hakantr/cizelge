//! Çizim katmanı — zrender'ın gpui üzerindeki karşılığı.
//!
//! [`yuzey::ÇizimYüzeyi`] soyutlaması üzerinden çizilir: gpui gerçeklemesi
//! [`cizici::Çizici`], test gerçeklemesi [`kayit::KayıtYüzeyi`]dir.

#[cfg(feature = "gpui")]
pub mod cizici;
pub mod donusum;
pub mod gorunum;
pub mod kayit;
pub mod olay;
#[cfg(feature = "gpui")]
pub mod pencere;
#[cfg(feature = "png")]
pub mod piksel;
pub mod sahne;
pub mod svg;
pub mod svg_yol;
pub mod yuzey;

#[cfg(feature = "gpui")]
pub use cizici::Çizici;
pub use donusum::AfinMatris;
pub use kayit::KayıtYüzeyi;
pub use olay::{
    AğaçHaritasıKökYönü, GrafikOlayı, MatrisHedefTürü, MatrisHücreBölgesi, ParalelEksenBölgesi,
    ParalelGenişletmeBölgesi, SihirliSeriTürü, İsabetBölgesi, İsabetGeometrisi,
};
#[cfg(feature = "png")]
pub use piksel::{PikselYüzeyi, png_dışa_aktar};
pub use sahne::{
    GörselDurum, KırpmaYolu, OdakKapsamı, Sahne, SahneDüğümü, SahneFarkı, SahneMetni, SahneResmi,
    SahneStilYaması, SahneStili, SahneÖğesi, Sahneİsabeti, SahneŞekli, YerelDönüşüm, yolu_dönüştür,
};
pub use svg::{SvgYüzeyi, svg_dışa_aktar};
pub use svg_yol::SvgYolHatası;
pub use yuzey::{DikeyHiza, SATIR_ORANI, YatayHiza, Yol, keskin, ÇizimYüzeyi};
