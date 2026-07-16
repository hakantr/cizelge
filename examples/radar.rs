//! Radar grafiği örneği: plan / gerçekleşen karşılaştırması.
//!
//! Çalıştırma: `cargo run --example radar`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Bütçe Dağılımı"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst(30.0))
        .radar(RadarKoordinatı::yeni().göstergeler([
            ("Satış", 6500.0),
            ("Yönetim", 16000.0),
            ("BT", 30000.0),
            ("Destek", 38000.0),
            ("Geliştirme", 52000.0),
            ("Pazarlama", 25000.0),
        ]))
        .seri(
            RadarSerisi::yeni()
                .ad("Bütçe")
                .alan_stili(AlanStili::yeni().opaklık(0.25))
                .veri([
                    ("Ayrılan", vec![4200.0, 3000.0, 20000.0, 35000.0, 50000.0, 18000.0]),
                    ("Harcanan", vec![5000.0, 14000.0, 28000.0, 26000.0, 42000.0, 21000.0]),
                ]),
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
