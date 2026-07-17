//! Paralel koordinatlar örneği.
//!
//! Çalıştırma: `cargo run --example paralel`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Ürün Karşılaştırma"))
        .seri(
            ParalelSerisi::yeni()
                .ad("Ölçümler")
                .boyutlar(["Fiyat", "Ağırlık", "Puan", "Stok"])
                .veri([
                    VeriÖğesi::from(vec![12.0, 3.4, 8.2, 40.0]),
                    VeriÖğesi::from(vec![9.5, 2.1, 6.4, 65.0]),
                    VeriÖğesi::from(vec![15.2, 4.8, 9.1, 20.0]),
                    VeriÖğesi::from(vec![7.8, 1.9, 5.5, 80.0]),
                    VeriÖğesi::from(vec![11.3, 2.8, 7.7, 55.0]),
                ]),
        )
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
