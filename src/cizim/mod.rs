//! Çizim katmanı — zrender'ın gpui üzerindeki karşılığı.
//!
//! [`yuzey::ÇizimYüzeyi`] soyutlaması üzerinden çizilir: gpui gerçeklemesi
//! [`cizici::Çizici`], test gerçeklemesi [`kayit::KayıtYüzeyi`]dir.

#[cfg(feature = "gpui")]
pub mod cizici;
pub mod gorunum;
#[cfg(feature = "gpui")]
pub mod pencere;
#[cfg(feature = "png")]
pub mod piksel;
pub mod kayit;
pub mod olay;
pub mod svg;
pub mod yuzey;

#[cfg(feature = "gpui")]
pub use cizici::Çizici;
#[cfg(feature = "png")]
pub use piksel::{png_dışa_aktar, PikselYüzeyi};
pub use kayit::KayıtYüzeyi;
pub use olay::{GrafikOlayı, İsabetBölgesi, İsabetGeometrisi};
pub use svg::{svg_dışa_aktar, SvgYüzeyi};
pub use yuzey::{keskin, DikeyHiza, SATIR_ORANI, YatayHiza, Yol, ÇizimYüzeyi};
