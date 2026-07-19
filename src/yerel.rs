//! Yerelleştirme (i18n) — `echarts/src/i18n` (langTR/langEN) karşılığı.
//!
//! Ay/gün adları ve arayüz metinleri etkin yerelden çözülür; etkin yerel,
//! boyama başında [`yerel_ayarla`] ile seçilir (koyu tema kipiyle aynı
//! iş parçacığı yerel düzen). Sayı biçimi ECharts'taki gibi yerelden
//! bağımsızdır (`addCommas` uluslararası gösterim kullanır).

use std::cell::Cell;

/// Bir dilin ad ve metin tabloları. Gün dizileri Pazartesi başlangıçlıdır
/// (takvim koordinatının satır düzeniyle aynı sıra).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Yerel {
    /// BCP-47 dil kodu (`"tr"`, `"en"`).
    pub kod: &'static str,
    pub aylar: [&'static str; 12],
    pub ay_kısaltmaları: [&'static str; 12],
    /// Pazartesi → Pazar.
    pub günler: [&'static str; 7],
    /// Pazartesi → Pazar.
    pub gün_kısaltmaları: [&'static str; 7],
    /// Araç kutusu: geri yükle düğmesi başlığı.
    pub geri_yükle: &'static str,
    /// Zaman şeridi: oynat/durdur başlıkları.
    pub oynat: &'static str,
    pub durdur: &'static str,
}

/// Türkçe (`langTR`) — öntanımlı yerel.
pub const TÜRKÇE: Yerel = Yerel {
    kod: "tr",
    aylar: [
        "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim",
        "Kasım", "Aralık",
    ],
    ay_kısaltmaları: [
        "Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas", "Ara",
    ],
    günler: [
        "Pazartesi",
        "Salı",
        "Çarşamba",
        "Perşembe",
        "Cuma",
        "Cumartesi",
        "Pazar",
    ],
    gün_kısaltmaları: ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"],
    geri_yükle: "Geri yükle",
    oynat: "Oynat",
    durdur: "Durdur",
};

/// İngilizce (`langEN`).
pub const İNGİLİZCE: Yerel = Yerel {
    kod: "en",
    aylar: [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ],
    ay_kısaltmaları: [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ],
    günler: [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ],
    gün_kısaltmaları: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
    geri_yükle: "Restore",
    oynat: "Play",
    durdur: "Pause",
};

std::thread_local! {
    static ETKİN: Cell<&'static Yerel> = const { Cell::new(&TÜRKÇE) };
}

/// Etkin yereli seçer; `grafiği_boya` her karede seçeneklerden çağırır.
pub fn yerel_ayarla(yerel: &'static Yerel) {
    ETKİN.with(|y| y.set(yerel));
}

/// Etkin yerel.
pub fn etkin_yerel() -> &'static Yerel {
    ETKİN.with(|y| y.get())
}

/// Etkin yerelden ay kısaltması (`ay`: 1–12; aralık dışına `"?"`).
pub fn ay_kısaltması(ay: u32) -> &'static str {
    ay.checked_sub(1)
        .and_then(|i| etkin_yerel().ay_kısaltmaları.get(i as usize))
        .copied()
        .unwrap_or("?")
}

/// Etkin yerelden gün kısaltması (`gün`: 0 = Pazartesi … 6 = Pazar).
pub fn gün_kısaltması(gün: usize) -> &'static str {
    etkin_yerel()
        .gün_kısaltmaları
        .get(gün)
        .copied()
        .unwrap_or("?")
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod testler {
    use super::*;

    #[test]
    fn yerel_geçişi() {
        yerel_ayarla(&TÜRKÇE);
        assert_eq!(ay_kısaltması(1), "Oca");
        assert_eq!(gün_kısaltması(0), "Pzt");
        yerel_ayarla(&İNGİLİZCE);
        assert_eq!(ay_kısaltması(1), "Jan");
        assert_eq!(gün_kısaltması(6), "Sun");
        yerel_ayarla(&TÜRKÇE);
    }

    #[test]
    fn aralık_dışı_güvenli() {
        yerel_ayarla(&TÜRKÇE);
        assert_eq!(ay_kısaltması(0), "?");
        assert_eq!(ay_kısaltması(13), "?");
        assert_eq!(gün_kısaltması(7), "?");
    }
}
