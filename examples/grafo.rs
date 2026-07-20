//! Grafo (graph) örneği: belirlenimci kuvvet yerleşimi.
//!
//! Çalıştırma: `cargo run --example grafo`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("İlişki Ağı"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(
            GrafoSerisi::yeni()
                .ad("Ağ")
                .etiket_göster(true)
                .düğümler([
                    GrafoDüğümü::yeni("Çekirdek", 34.0).kategori(0),
                    GrafoDüğümü::yeni("Model", 22.0).kategori(1),
                    GrafoDüğümü::yeni("Çizim", 22.0).kategori(1),
                    GrafoDüğümü::yeni("Ölçek", 16.0).kategori(2),
                    GrafoDüğümü::yeni("Eksen", 16.0).kategori(2),
                    GrafoDüğümü::yeni("Seri", 18.0).kategori(2),
                    GrafoDüğümü::yeni("Olay", 14.0).kategori(3),
                    GrafoDüğümü::yeni("İpucu", 12.0).kategori(3),
                ])
                .bağlar([
                    ("Çekirdek", "Model"),
                    ("Çekirdek", "Çizim"),
                    ("Model", "Ölçek"),
                    ("Model", "Seri"),
                    ("Çizim", "Eksen"),
                    ("Çizim", "Olay"),
                    ("Olay", "İpucu"),
                    ("Seri", "Eksen"),
                ]),
        )
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(860.0), px(640.0)), cx);
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
