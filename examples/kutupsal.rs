//! Kutupsal koordinat örneği: yön bazlı sütunlar + çizgi.
//!
//! Çalıştırma: `cargo run --example kutupsal`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Rüzgar Gülü"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .kutupsal(
            KutupsalKoordinat::yeni().açısal_eksen(
                Eksen::kategori().veri(["K", "KD", "D", "GD", "G", "GB", "B", "KB"]),
            ),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Hız")
                .kutupsal(true)
                .veri([4.0, 7.0, 3.0, 6.0, 8.0, 5.0, 2.0, 6.5]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Ortalama")
                .kutupsal(true)
                .sembol_boyutu(6.0)
                .veri([3.0, 5.0, 4.0, 5.5, 6.0, 4.5, 3.0, 5.0]),
        )
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(760.0), px(640.0)), cx);
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
