//! Tema nehri (themeRiver) örneği: kaynak bazında enerji üretiminin
//! zaman içindeki akışı; koyu tema açık.
//!
//! Çalıştırma: `cargo run --example tema_nehri`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    let katmanlar = ["Rüzgar", "Güneş", "Hidro", "Doğalgaz"];
    let mut veri: Vec<(f64, f64, String)> = Vec::new();
    for (k, katman) in katmanlar.iter().enumerate() {
        for x in 0..12 {
            // Belirlenimci ama dalgalı bir örnek dizi.
            let taban = 18.0 + k as f64 * 6.0;
            let dalga = ((x as f64 / 11.0) * std::f64::consts::TAU + k as f64).sin() * 10.0;
            veri.push((x as f64, (taban + dalga).max(2.0), (*katman).to_string()));
        }
    }
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Enerji Üretimi Akışı"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .koyu(true)
        .seri(TemaNehriSerisi::yeni().ad("Üretim").veri(veri))
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
