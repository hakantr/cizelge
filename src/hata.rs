//! Hata ve tanı modeli — panik yasağının temel taşı.
//!
//! Çalışma zamanında hiçbir bileşen panik üretmez: kurtarılabilir her sorun
//! ya [`BilesenHatasi`] olarak çağırana döner ya da boyama sırasında
//! [`BilesenTanisi`] olarak tanı kanalına yazılır ve çizim, o öğe atlanarak
//! sürer. `GrafikGörünümü`, biriken tanıları gpui olayı olarak yayımlar
//! (`EventEmitter<BilesenTanisi>`).

use std::fmt;

/// Bileşenlerden dönen, panik yerine geçen tipli hata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BilesenHatasi {
    /// Paylaşılan durum (RefCell) o an başka erişim tarafından kilitli;
    /// işlem güvenle atlandı.
    KilitliDurum { bileşen: &'static str },
    /// Beklenen veri yok ya da sınır dışı.
    EksikVeri { bileşen: &'static str, sıra: usize },
    /// Seçenek doğrulaması başarısız; işlem geri alındı.
    GeçersizSeçenek { alan: &'static str, ayrıntı: String },
    /// Metin çözümlemesi başarısız (renk, uzunluk, yüzde…).
    ÇözümlemeHatası { girdi: String, hedef: &'static str },
    /// Sayısal işlem güvenli aralık dışında.
    SayısalTaşma { bileşen: &'static str },
}

impl fmt::Display for BilesenHatasi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BilesenHatasi::KilitliDurum { bileşen } => {
                write!(f, "{bileşen}: paylaşılan durum kilitli, işlem atlandı")
            }
            BilesenHatasi::EksikVeri { bileşen, sıra } => {
                write!(f, "{bileşen}: {sıra}. sırada beklenen veri yok")
            }
            BilesenHatasi::GeçersizSeçenek { alan, ayrıntı } => {
                write!(f, "geçersiz seçenek `{alan}`: {ayrıntı}")
            }
            BilesenHatasi::ÇözümlemeHatası { girdi, hedef } => {
                write!(f, "`{girdi}` girdisi {hedef} olarak çözümlenemedi")
            }
            BilesenHatasi::SayısalTaşma { bileşen } => {
                write!(f, "{bileşen}: sayısal değer güvenli aralık dışında")
            }
        }
    }
}

impl std::error::Error for BilesenHatasi {}

/// Tanı kanalına yazılan tek kayıt: hangi bileşen, hangi hata.
///
/// Boyama hattı hatada durmaz; sorunlu öğeyi atlar, tanıyı bu kayıtla
/// bildirir. `GrafikGörünümü` üzerinden gpui olayı olarak dinlenebilir:
///
/// ```ignore
/// cx.subscribe(&grafik, |_, _, tanı: &BilesenTanisi, _| {
///     eprintln!("çizelge tanısı: {tanı}");
/// }).detach();
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BilesenTanisi {
    pub bileşen: &'static str,
    pub hata: BilesenHatasi,
}

impl BilesenTanisi {
    pub fn yeni(bileşen: &'static str, hata: BilesenHatasi) -> Self {
        BilesenTanisi { bileşen, hata }
    }
}

impl fmt::Display for BilesenTanisi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.bileşen, self.hata)
    }
}
