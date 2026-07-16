//! Pasta grafiği örneği: halka (donut) biçimi, dış etiketler, gösterge ile
//! dilim açma/kapama ve öğe ipucu.
//!
//! Çalıştırma: `cargo run --example pasta`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(
            Başlık::yeni()
                .metin("Erişim Kaynağı")
                .alt_metin("kurgusal veri")
                .sol("orta"),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst(56.0))
        .seri(
            PastaSerisi::yeni()
                .ad("Erişim Kaynağı")
                .halka("40%", "68%")
                .merkez("50%", "56%")
                .veri([
                    ("Doğrudan", 1048.0),
                    ("E-posta", 735.0),
                    ("Reklam", 580.0),
                    ("Video", 484.0),
                    ("Arama Motoru", 300.0),
                ])
                .öğe_stili(
                    ÖğeStili::yeni()
                        .kenarlık_rengi(Renk::BEYAZ)
                        .kenarlık_kalınlığı(2.0),
                ),
        )
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(800.0), px(640.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(sınırlar)),
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(|_| GrafikGörünümü::yeni(seçenekler())),
        )
        .unwrap();
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
