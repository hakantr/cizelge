//! Güneş patlaması (sunburst) örneği: iç içe halkalarla hiyerarşi.
//!
//! Çalıştırma: `cargo run --example gunes`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Trafik Kaynakları"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(GüneşPatlamasıSerisi::yeni().ad("Trafik").kökler([
            AğaçDüğümü::dal(
                "Arama",
                vec![
                    AğaçDüğümü::yaprak("Organik", 40.0),
                    AğaçDüğümü::yaprak("Reklam", 18.0),
                ],
            ),
            AğaçDüğümü::dal(
                "Sosyal",
                vec![
                    AğaçDüğümü::yaprak("Video", 14.0),
                    AğaçDüğümü::dal(
                        "Mikroblog",
                        vec![
                            AğaçDüğümü::yaprak("Paylaşım", 6.0),
                            AğaçDüğümü::yaprak("Profil", 4.0),
                        ],
                    ),
                ],
            ),
            AğaçDüğümü::yaprak("Doğrudan", 22.0),
        ]))
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(760.0), px(660.0)), cx);
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
