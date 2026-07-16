//! İmleyici örneği: ortalama/min/maks çizgileri, raptiyeler ve alan vurgusu.
//!
//! Çalıştırma: `cargo run --example imleyiciler`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Aylık Üretim"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(Izgara::yeni().sol("8%").sağ(90.0).alt(50.0).etiketi_kapsa(true))
        .x_ekseni(Eksen::kategori().veri([
            "Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu",
        ]))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("Üretim")
                .veri([320.0, 280.0, 410.0, 360.0, 450.0, 390.0, 470.0, 340.0])
                .im_çizgisi(İmÇizgisi::yeni().yatay(İmDeğeri::Ortalama))
                .im_noktası(İmNoktası::yeni().en_büyük().en_küçük())
                .im_alanı(İmAlanı::yeni().x_aralığı("Bakım", 4.0, 5.0)),
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
