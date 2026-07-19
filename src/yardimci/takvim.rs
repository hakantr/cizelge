//! Takvim yardımcıları — zaman ölçeği için UTC tabanlı tarih dönüşümleri.
//!
//! ECharts zaman ölçeği JS `Date` üzerine kuruludur; burada bağımlılıksız,
//! Howard Hinnant'ın gün-sayısı algoritmalarıyla eşdeğer dönüşüm yapılır.

/// Takvim anı (UTC).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TakvimAnı {
    pub yıl: i32,
    /// 1–12
    pub ay: u32,
    /// 1–31
    pub gün: u32,
    pub saat: u32,
    pub dakika: u32,
    pub saniye: u32,
    pub milisaniye: u32,
}

const GÜN_MS: i64 = 86_400_000;

/// 1970-01-01'den bu yana geçen gün sayısını (`z`) yıl/ay/gün üçlüsüne çevirir.
fn günden_tarihe(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let dönem = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let dönem_günü = (z - dönem * 146_097) as u64; // [0, 146096]
    let dönem_yılı =
        (dönem_günü - dönem_günü / 1460 + dönem_günü / 36524 - dönem_günü / 146096) / 365;
    let yıl = dönem_yılı as i64 + dönem * 400;
    let yıl_günü = dönem_günü - (365 * dönem_yılı + dönem_yılı / 4 - dönem_yılı / 100);
    let mp = (5 * yıl_günü + 2) / 153;
    let gün = (yıl_günü - (153 * mp + 2) / 5 + 1) as u32;
    let ay = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((yıl + if ay <= 2 { 1 } else { 0 }) as i32, ay, gün)
}

/// Yıl/ay/gün üçlüsünü 1970-01-01'den bu yana geçen gün sayısına çevirir.
fn tarihten_güne(yıl: i32, ay: u32, gün: u32) -> i64 {
    let yıl = yıl as i64 - if ay <= 2 { 1 } else { 0 };
    let dönem = if yıl >= 0 { yıl } else { yıl - 399 } / 400;
    let dönem_yılı = (yıl - dönem * 400) as u64;
    let ay = ay as u64;
    let yıl_günü = (153 * (if ay > 2 { ay - 3 } else { ay + 9 }) + 2) / 5 + gün as u64 - 1;
    let dönem_günü = dönem_yılı * 365 + dönem_yılı / 4 - dönem_yılı / 100 + yıl_günü;
    dönem * 146_097 + dönem_günü as i64 - 719_468
}

/// Unix milisaniyesini takvim anına çevirir (UTC).
pub fn andan_takvime(ms: f64) -> TakvimAnı {
    let ms = ms as i64;
    let gün_sayısı = ms.div_euclid(GÜN_MS);
    let gün_içi = ms.rem_euclid(GÜN_MS);
    let (yıl, ay, gün) = günden_tarihe(gün_sayısı);
    TakvimAnı {
        yıl,
        ay,
        gün,
        saat: (gün_içi / 3_600_000) as u32,
        dakika: (gün_içi / 60_000 % 60) as u32,
        saniye: (gün_içi / 1000 % 60) as u32,
        milisaniye: (gün_içi % 1000) as u32,
    }
}

/// Takvim anını Unix milisaniyesine çevirir (UTC).
pub fn takvimden_ana(t: TakvimAnı) -> f64 {
    let gün_sayısı = tarihten_güne(t.yıl, t.ay, t.gün);
    (gün_sayısı * GÜN_MS
        + t.saat as i64 * 3_600_000
        + t.dakika as i64 * 60_000
        + t.saniye as i64 * 1000
        + t.milisaniye as i64) as f64
}

/// Bir anı verilen milisaniye biriminin tabanına indirger.
pub fn birime_indirge(ms: f64, birim_ms: f64) -> f64 {
    (ms / birim_ms).floor() * birim_ms
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
    fn gidiş_dönüş() {
        // 2026-07-16 12:30:45.250 UTC
        let t = TakvimAnı {
            yıl: 2026,
            ay: 7,
            gün: 16,
            saat: 12,
            dakika: 30,
            saniye: 45,
            milisaniye: 250,
        };
        assert_eq!(andan_takvime(takvimden_ana(t)), t);
    }

    #[test]
    fn dönem_başlangıcı() {
        let t = andan_takvime(0.0);
        assert_eq!((t.yıl, t.ay, t.gün), (1970, 1, 1));
    }
}
