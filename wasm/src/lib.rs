//! cizelge-wasm — çekirdeğin tarayıcı köprüsü.
//!
//! gpui'siz çekirdek (`cizelge` `default-features = false`) seçeneklerden
//! [`svg_dışa_aktar`] ile SVG üretir; buradaki `#[wasm_bindgen]`
//! işlevleri bu hattı JavaScript'e açar. Dışa açılan adlar, `#[no_mangle]`
//! ABI sınırı ASCII gerektirdiği için dosya adı kuralındaki gibi aksansız
//! yazılır (`svg_cizgi`). Girdiler bilinçli olarak yalın
//! tutulur (sayı dizisi + virgüllü kategori metni) — böylece köprü, JSON
//! çözümleyici bağımlılığı almadan çalışır.

use cizelge::hazir::*;
use wasm_bindgen::prelude::*;

/// Ortak kuruluş: başlık + koyu kip.
fn temel(başlık: &str, koyu: bool) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin(başlık.to_string()))
        .animasyon(false)
        .koyu(koyu)
}

/// Virgüllü kategori metnini listeye çevirir; boşsa `G1..Gn` üretir.
fn kategorileri_çöz(kategoriler: &str, n: usize) -> Vec<String> {
    let liste: Vec<String> = kategoriler
        .split(',')
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(str::to_string)
        .collect();
    if liste.len() >= n {
        liste
    } else {
        (1..=n).map(|i| format!("G{i}")).collect()
    }
}

/// Çizgi grafiği SVG'si.
#[wasm_bindgen]
pub fn svg_cizgi(
    başlık: &str,
    kategoriler: &str,
    değerler: &[f64],
    koyu: bool,
    genişlik: f32,
    yükseklik: f32,
) -> String {
    let seçenekler = temel(başlık, koyu)
        .x_ekseni(Eksen::kategori().veri(kategorileri_çöz(kategoriler, değerler.len())))
        .y_ekseni(Eksen::değer())
        .seri(
            ÇizgiSerisi::yeni()
                .ad(başlık.to_string())
                .yumuşat(true)
                .alan_stili(AlanStili::yeni().opaklık(0.18))
                .veri(değerler.to_vec()),
        );
    svg_dışa_aktar(&seçenekler, genişlik, yükseklik)
}

/// Sütun grafiği SVG'si.
#[wasm_bindgen]
pub fn svg_sutun(
    başlık: &str,
    kategoriler: &str,
    değerler: &[f64],
    koyu: bool,
    genişlik: f32,
    yükseklik: f32,
) -> String {
    let seçenekler = temel(başlık, koyu)
        .x_ekseni(Eksen::kategori().veri(kategorileri_çöz(kategoriler, değerler.len())))
        .y_ekseni(Eksen::değer())
        .seri(SütunSerisi::yeni().ad(başlık.to_string()).veri(değerler.to_vec()));
    svg_dışa_aktar(&seçenekler, genişlik, yükseklik)
}

/// Pasta grafiği SVG'si (`adlar` virgüllü, `değerler` aynı uzunlukta).
#[wasm_bindgen]
pub fn svg_pasta(
    başlık: &str,
    adlar: &str,
    değerler: &[f64],
    koyu: bool,
    genişlik: f32,
    yükseklik: f32,
) -> String {
    let adlar = kategorileri_çöz(adlar, değerler.len());
    let veri: Vec<(String, f64)> = adlar
        .into_iter()
        .zip(değerler.iter().copied())
        .collect();
    let seçenekler = temel(başlık, koyu).seri(
        PastaSerisi::yeni()
            .ad(başlık.to_string())
            .halka("35%", "62%")
            .merkez("50%", "55%")
            .veri(veri),
    );
    svg_dışa_aktar(&seçenekler, genişlik, yükseklik)
}

/// Ekran okuyucular için erişilebilirlik özeti (aria metni).
#[wasm_bindgen]
pub fn ozet_cizgi(başlık: &str, kategoriler: &str, değerler: &[f64]) -> String {
    let seçenekler = temel(başlık, false)
        .x_ekseni(Eksen::kategori().veri(kategorileri_çöz(kategoriler, değerler.len())))
        .seri(ÇizgiSerisi::yeni().ad(başlık.to_string()).veri(değerler.to_vec()));
    erişilebilirlik_özeti(&seçenekler)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod testler {
    use super::*;

    #[test]
    fn svg_üretimi() {
        let svg = svg_cizgi("Deneme", "A,B,C", &[1.0, 3.0, 2.0], false, 400.0, 300.0);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("Deneme"));
        let koyu = svg_cizgi("Deneme", "A,B,C", &[1.0, 3.0, 2.0], true, 400.0, 300.0);
        assert_ne!(svg, koyu);
    }
}
