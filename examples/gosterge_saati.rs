//! Gösterge saati (gauge) örneği: renk bantlı ibre.
//!
//! Çalıştırma: `cargo run --example gosterge_saati`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Sunucu Yükü"))
        .seri(
            GöstergeSaatiSerisi::yeni()
                .ad("Yük")
                .değer(68.0, "CPU")
                .renk_bantları([(0.3, "#67e0e3"), (0.7, "#37a2da"), (1.0, "#fd666d")])
                .şerit(true, 18.0)
                .değer_biçimleyici("{value} %"),
        )
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(900.0), px(620.0)), cx);
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
