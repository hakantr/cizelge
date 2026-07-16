//! Isı haritası örneği: gün × saat yoğunluğu, görsel eşleme çubuğu.
//!
//! Çalıştırma: `cargo run --example isi`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    let saatler = ["00", "03", "06", "09", "12", "15", "18", "21"];
    let günler = ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"];
    let mut veri: Vec<VeriÖğesi> = Vec::new();
    for (g, _) in günler.iter().enumerate() {
        for (s, _) in saatler.iter().enumerate() {
            // Kurgusal ama belirlenimci yoğunluk deseni.
            let değer = ((g * 7 + s * 3) % 13) as f64 + if (2..=4).contains(&s) { 6.0 } else { 0.0 };
            veri.push(VeriÖğesi::from([s as f64, g as f64, değer]));
        }
    }
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Haftalık Yoğunluk"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .ızgara(Izgara::yeni().sol(70.0).sağ(30.0).üst(60.0).alt(40.0))
        .x_ekseni(Eksen::kategori().veri(saatler))
        .y_ekseni(Eksen::kategori().veri(günler))
        .görsel_eşleme(GörselEşleme::yeni())
        .seri(IsıHaritasıSerisi::yeni().ad("Yoğunluk").veri(veri))
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
