//! Kiriş (chord) diyagramı örneği.
//!
//! Çalıştırma: `cargo run --example kiris`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Bölgeler Arası Göç"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(KirişSerisi::yeni().ad("Göç").bağlar([
            ("Kuzey", "Güney", 12.0),
            ("Güney", "Doğu", 8.0),
            ("Doğu", "Kuzey", 5.0),
            ("Kuzey", "Batı", 6.0),
            ("Batı", "Güney", 4.0),
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
