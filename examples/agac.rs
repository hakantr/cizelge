//! Ağaç (tree) örneği: kuruluş şeması, düzenli yatay yerleşim.
//!
//! Çalıştırma: `cargo run --example agac`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Kuruluş Şeması"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(AğaçSerisi::yeni().ad("Kuruluş").kökler([AğaçDüğümü::dal(
            "Genel Müdür",
            vec![
                AğaçDüğümü::dal(
                    "Mühendislik",
                    vec![
                        AğaçDüğümü::yaprak("Arayüz Ekibi", 12.0),
                        AğaçDüğümü::yaprak("Altyapı Ekibi", 9.0),
                        AğaçDüğümü::yaprak("Veri Ekibi", 6.0),
                    ],
                ),
                AğaçDüğümü::dal(
                    "Satış",
                    vec![
                        AğaçDüğümü::yaprak("Yurt İçi", 7.0),
                        AğaçDüğümü::yaprak("Yurt Dışı", 5.0),
                    ],
                ),
                AğaçDüğümü::yaprak("İnsan Kaynakları", 4.0),
                AğaçDüğümü::yaprak("Finans", 3.0),
            ],
        )]))
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
