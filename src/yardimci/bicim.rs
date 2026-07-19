//! Biçimleme yardımcıları — `echarts/src/util/format.ts` portu.

use crate::yardimci::sayi::{hassasiyet, yuvarla};

/// Binlik ayraç ekler: `12345678.123` → `"12.345.678,123"` yerine ECharts ile
/// birebir uyum için uluslararası gösterim kullanılır: `"12,345,678.123"`.
/// `util/format.ts` içindeki `addCommas` portu.
pub fn binlik_ayır(x: f64) -> String {
    if !x.is_finite() {
        return "-".to_string();
    }
    let s = ondalık_kırp(x);
    let (tam, ondalık) = match s.split_once('.') {
        Some((t, o)) => (t.to_string(), Some(o.to_string())),
        None => (s, None),
    };
    let (işaret, basamaklar) = match tam.strip_prefix('-') {
        Some(b) => ("-", b),
        None => ("", tam.as_str()),
    };
    let mut öbekli = String::new();
    let bayt = basamaklar.as_bytes();
    for (i, b) in bayt.iter().enumerate() {
        öbekli.push(*b as char);
        let kalan = bayt.len() - 1 - i;
        if kalan > 0 && kalan % 3 == 0 {
            öbekli.push(',');
        }
    }
    match ondalık {
        Some(o) => format!("{işaret}{öbekli}.{o}"),
        None => format!("{işaret}{öbekli}"),
    }
}

/// Kayan noktalı sayıyı bilimsel gösterimsiz ve sondaki sıfırlar kırpılmış
/// olarak metne çevirir.
pub fn ondalık_kırp(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e15 {
        return format!("{}", x as i64);
    }
    let h = hassasiyet(x).min(15);
    let s = format!("{x:.h$}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Eksen çentiği değerini, ölçeğin adım hassasiyetiyle biçimler.
pub fn çentik_değeri_biçimle(değer: f64, adım_hassasiyeti: usize) -> String {
    binlik_ayır(yuvarla(değer, adım_hassasiyeti))
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
    fn binlik() {
        assert_eq!(binlik_ayır(12345678.0), "12,345,678");
        assert_eq!(binlik_ayır(-1234.5), "-1,234.5");
        assert_eq!(binlik_ayır(999.0), "999");
    }

    #[test]
    fn kırpma() {
        assert_eq!(ondalık_kırp(1.50), "1.5");
        assert_eq!(ondalık_kırp(100.0), "100");
    }
}
