//! Sankey örneği: enerji akış diyagramı.
//!
//! Çalıştırma: `cargo run --example sankey`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Enerji Akışı"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(SankeySerisi::yeni().ad("Enerji").bağlar([
            ("Kömür", "Elektrik", 30.0),
            ("Doğalgaz", "Elektrik", 20.0),
            ("Güneş", "Elektrik", 12.0),
            ("Rüzgar", "Elektrik", 15.0),
            ("Elektrik", "Konut", 35.0),
            ("Elektrik", "Sanayi", 30.0),
            ("Elektrik", "Ulaşım", 12.0),
            ("Doğalgaz", "Konut", 8.0),
            ("Doğalgaz", "Sanayi", 6.0),
        ]))
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
            |_, cx| cx.new(|_| GrafikGörünümü::yeni(seçenekler())),
        )
        .unwrap_or_else(|hata| {
            eprintln!("Pencere açılamadı: {hata}");
            std::process::exit(1);
        });
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
