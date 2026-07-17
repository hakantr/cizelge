//! Takvim ısı haritası örneği.
//!
//! Çalıştırma: `cargo run --example takvim`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    // 2026 yılı, belirlenimci günlük değerler.
    let gün_ms = 86_400_000.0f64;
    let yıl_başı = 1_767_225_600_000.0f64; // 2026-01-01 UTC
    let veri: Vec<VeriÖğesi> = (0..365)
        .map(|g| {
            let değer = ((g * 7) % 13) as f64 + if g % 11 == 0 { 6.0 } else { 0.0 };
            VeriÖğesi::from(vec![yıl_başı + g as f64 * gün_ms, değer])
        })
        .collect();
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Katkı Takvimi — 2026"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .görsel_eşleme(GörselEşleme::yeni())
        .seri(TakvimSerisi::yeni(2026).ad("Katkılar").veri(veri))
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
