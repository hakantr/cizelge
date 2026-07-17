//! Çizim katmanı — zrender'ın gpui üzerindeki karşılığı.
//!
//! [`yuzey::ÇizimYüzeyi`] soyutlaması üzerinden çizilir: gpui gerçeklemesi
//! [`cizici::Çizici`], test gerçeklemesi [`kayit::KayıtYüzeyi`]dir.

pub mod cizici;
pub mod gorunum;
pub mod kayit;
pub mod olay;
pub mod svg;
pub mod yuzey;

pub use cizici::Çizici;
pub use kayit::KayıtYüzeyi;
pub use olay::{GrafikOlayı, İsabetBölgesi, İsabetGeometrisi};
pub use svg::{svg_dışa_aktar, SvgYüzeyi};
pub use yuzey::{keskin, DikeyHiza, SATIR_ORANI, YatayHiza, Yol, ÇizimYüzeyi};
