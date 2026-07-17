//! Ağaç haritası (treemap) örneği: kareselleştirilmiş disk kullanımı.
//!
//! Çalıştırma: `cargo run --example agac_haritasi`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Disk Kullanımı"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(AğaçHaritasıSerisi::yeni().ad("Disk").kökler([
            AğaçDüğümü::dal(
                "Belgeler",
                vec![
                    AğaçDüğümü::yaprak("Raporlar", 32.0),
                    AğaçDüğümü::yaprak("Sunumlar", 18.0),
                    AğaçDüğümü::yaprak("Tablolar", 9.0),
                ],
            ),
            AğaçDüğümü::dal(
                "Medya",
                vec![
                    AğaçDüğümü::yaprak("Video", 60.0),
                    AğaçDüğümü::yaprak("Müzik", 25.0),
                    AğaçDüğümü::yaprak("Fotoğraf", 15.0),
                ],
            ),
            AğaçDüğümü::dal(
                "Geliştirme",
                vec![
                    AğaçDüğümü::yaprak("Depolar", 22.0),
                    AğaçDüğümü::yaprak("Derleme", 14.0),
                ],
            ),
            AğaçDüğümü::yaprak("Sistem", 20.0),
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
