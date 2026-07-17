//! Zaman şeridi (timeline) örneği: yıllara göre üretim karelerinin
//! kendiliğinden oynatılması. Alttaki noktalarla kare seçilebilir,
//! soldaki düğmeyle oynatma durdurulur.
//!
//! Çalıştırma: `cargo run --example zaman_seridi`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn kare(yıl: i32, değerler: [f64; 4]) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin(format!("Üretim — {yıl}")))
        .x_ekseni(Eksen::kategori().veri(["Rüzgar", "Güneş", "Hidro", "Doğalgaz"]))
        .y_ekseni(Eksen::değer())
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .seri(SütunSerisi::yeni().ad("GWh").veri(değerler))
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(960.0), px(620.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(sınırlar)),
                focus: true,
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| {
                    GrafikGörünümü::film(
                        vec![
                            kare(2022, [34.0, 18.0, 62.0, 88.0]),
                            kare(2023, [45.0, 27.0, 58.0, 80.0]),
                            kare(2024, [58.0, 41.0, 66.0, 71.0]),
                            kare(2025, [72.0, 58.0, 61.0, 60.0]),
                            kare(2026, [90.0, 76.0, 68.0, 48.0]),
                        ],
                        1600.0,
                    )
                })
            },
        )
        .unwrap_or_else(|hata| {
            eprintln!("Pencere açılamadı: {hata}");
            std::process::exit(1);
        });
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
