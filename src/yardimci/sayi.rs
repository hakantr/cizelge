//! Sayısal yardımcılar — `echarts/src/util/number.ts` portu.

/// [`güzel_sayı`] için yuvarlama kipi; ECharts'taki `NICE_MODE_ROUND` /
/// `NICE_MODE_MIN` sabitlerinin ve öntanımlı "tavan" davranışının karşılığı.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GüzelKip {
    Yuvarlak,
    Tavan,
    EnKüçük,
}

/// Bir değeri `tanım` aralığından `hedef` aralığına doğrusal eşler.
/// `util/number.ts` içindeki `linearMap` portu.
pub fn doğrusal_eşle(değer: f64, tanım: [f64; 2], hedef: [f64; 2], kıstır: bool) -> f64 {
    let tanım_farkı = tanım[1] - tanım[0];
    let hedef_farkı = hedef[1] - hedef[0];

    if tanım_farkı == 0.0 {
        return if hedef_farkı == 0.0 {
            hedef[0]
        } else {
            (hedef[0] + hedef[1]) / 2.0
        };
    }

    if kıstır {
        if tanım_farkı > 0.0 {
            if değer <= tanım[0] {
                return hedef[0];
            } else if değer >= tanım[1] {
                return hedef[1];
            }
        } else if değer >= tanım[0] {
            return hedef[0];
        } else if değer <= tanım[1] {
            return hedef[1];
        }
    }

    (değer - tanım[0]) / tanım_farkı * hedef_farkı + hedef[0]
}

/// `"%12"` tarzı olmayan, `"12%"` biçimindeki yüzde metnini `bütün`e göre
/// çözer; düz sayı metnini olduğu gibi geçirir.
/// `util/number.ts` içindeki `parsePercent` portu.
pub fn yüzde_çöz(metin: &str, bütün: f64) -> f64 {
    let s = metin.trim();
    if let Some(p) = s.strip_suffix('%') {
        p.trim().parse::<f64>().unwrap_or(0.0) / 100.0 * bütün
    } else {
        s.parse::<f64>().unwrap_or(0.0)
    }
}

/// Kayan nokta hatalarını gidermek için `x`'i en çok `hassasiyet` ondalık
/// basamağa yuvarlar. `util/number.ts` içindeki `round` portu
/// (`+(+x).toFixed(precision)` davranışı).
pub fn yuvarla(x: f64, hassasiyet: usize) -> f64 {
    let hassasiyet = hassasiyet.min(20);
    format!("{x:.hassasiyet$}").parse::<f64>().unwrap_or(x)
}

/// Değerin anlamlı ondalık basamak sayısı.
/// `util/number.ts` içindeki `getPrecision` portu.
pub fn hassasiyet(değer: f64) -> usize {
    if değer.is_nan() || !değer.is_finite() {
        return 0;
    }
    if (değer.round() - değer).abs() <= f64::EPSILON * değer.abs().max(1.0) {
        return 0;
    }
    let mut e = 10f64;
    let mut sayaç = 1usize;
    while sayaç < 15
        && ((değer * e).round() / e - değer).abs() > f64::EPSILON * değer.abs().max(1.0)
    {
        e *= 10.0;
        sayaç += 1;
    }
    sayaç
}

/// Sayının nicelik üssü; örn. `nicelik_üssü(9876.0) == 3`.
/// `util/number.ts` içindeki `quantityExponent` portu (#11249 düzeltmesiyle).
pub fn nicelik_üssü(değer: f64) -> i32 {
    if değer == 0.0 {
        return 0;
    }
    let mut üs = (değer.ln() / std::f64::consts::LN_10).floor() as i32;
    // `Math.log` hassasiyet kaybına karşı geri kazanım (#11249).
    if değer / 10f64.powi(üs) >= 10.0 {
        üs += 1;
    }
    üs
}

/// Sayının niceliği; örn. `nicelik(0.24) == 0.1`.
pub fn nicelik(değer: f64) -> f64 {
    10f64.powi(nicelik_üssü(değer))
}

/// `değer`e yaklaşık "güzel" bir sayı bulur (Graphics Gems, "Nice Numbers
/// for Graph Labels"). `util/number.ts` içindeki `nice` portu.
pub fn güzel_sayı(değer: f64, kip: GüzelKip) -> f64 {
    let üs = nicelik_üssü(değer);
    let üs10 = 10f64.powi(üs);
    let f = değer / üs10;
    let güzel_f: f64 = match kip {
        GüzelKip::EnKüçük => 1.0,
        GüzelKip::Yuvarlak => {
            if f < 1.5 {
                1.0
            } else if f < 2.5 {
                2.0
            } else if f < 4.0 {
                3.0
            } else if f < 7.0 {
                5.0
            } else {
                10.0
            }
        }
        GüzelKip::Tavan => {
            if f < 1.0 {
                1.0
            } else if f < 2.0 {
                2.0
            } else if f < 3.0 {
                3.0
            } else if f < 5.0 {
                5.0
            } else {
                10.0
            }
        }
    };
    let sonuç = güzel_f * üs10;
    // 20, JS `toFixed`in desteklediği en yüksek hassasiyettir.
    if üs >= -20 {
        yuvarla(sonuç, (-üs).max(0) as usize)
    } else {
        sonuç
    }
}

/// Kapsam ucu olarak kullanılabilir bir sayı mı?
pub fn geçerli_kapsam_sayısı(v: f64) -> bool {
    v.is_finite()
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn güzel_yuvarlak() {
        assert_eq!(güzel_sayı(20.0 / 5.0, GüzelKip::Yuvarlak), 5.0);
        assert_eq!(güzel_sayı(0.0054321, GüzelKip::Yuvarlak), 0.005);
        assert_eq!(güzel_sayı(987.12345, GüzelKip::Yuvarlak), 1000.0);
        assert_eq!(güzel_sayı(13.0, GüzelKip::Yuvarlak), 10.0);
        assert_eq!(güzel_sayı(0.35, GüzelKip::Yuvarlak), 0.3);
    }

    #[test]
    fn doğrusal_eşleme() {
        assert_eq!(doğrusal_eşle(5.0, [0.0, 10.0], [0.0, 100.0], false), 50.0);
        assert_eq!(doğrusal_eşle(-5.0, [0.0, 10.0], [0.0, 100.0], true), 0.0);
    }

    #[test]
    fn hassasiyet_hesabı() {
        assert_eq!(hassasiyet(1.25), 2);
        assert_eq!(hassasiyet(10.0), 0);
    }

    #[test]
    fn nicelik_üsleri() {
        assert_eq!(nicelik_üssü(9876.0), 3);
        assert_eq!(nicelik_üssü(0.09876), -2);
        assert_eq!(nicelik_üssü(0.0), 0);
    }

    #[test]
    fn yüzde_çözümleme() {
        assert_eq!(yüzde_çöz("50%", 200.0), 100.0);
        assert_eq!(yüzde_çöz("42", 200.0), 42.0);
    }
}
