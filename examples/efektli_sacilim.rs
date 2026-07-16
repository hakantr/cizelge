//! Efektli saçılım (effectScatter) örneği: dalga halkalı vurgular.
//!
//! Çalıştırma: `cargo run --example efektli_sacilim`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Canlı Sinyaller"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .ızgara(Izgara::yeni().sol(60.0).sağ(30.0).alt(50.0))
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(
            SaçılımSerisi::yeni()
                .ad("Uyarılar")
                .sembol_boyutu(14.0)
                .efektli(true)
                .veri([[2.0, 8.0], [5.0, 3.0], [8.0, 6.5], [11.0, 4.2]]),
        )
        .seri(
            SaçılımSerisi::yeni()
                .ad("Normal")
                .sembol_boyutu(9.0)
                .veri([[3.0, 5.0], [6.5, 7.2], [9.0, 2.4], [12.0, 7.8]]),
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
