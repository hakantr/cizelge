//! Mum (candlestick) grafiği örneği: OHLC verisi + ortalama im çizgisi.
//!
//! Çalıştırma: `cargo run --example mum`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Hisse Fiyatı"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(Izgara::yeni().sol("8%").sağ("6%").alt(50.0).etiketi_kapsa(true))
        .x_ekseni(Eksen::kategori().veri([
            "Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz", "Pzt2", "Sal2", "Çar2",
        ]))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(
            MumSerisi::yeni()
                .ad("BIST")
                .veri([
                    [120.0, 132.0, 114.0, 138.0],
                    [132.0, 128.0, 122.0, 140.0],
                    [128.0, 145.0, 126.0, 148.0],
                    [145.0, 141.0, 134.0, 150.0],
                    [141.0, 155.0, 139.0, 158.0],
                    [155.0, 149.0, 144.0, 160.0],
                    [149.0, 162.0, 147.0, 165.0],
                    [162.0, 158.0, 152.0, 168.0],
                    [158.0, 171.0, 156.0, 174.0],
                    [171.0, 166.0, 160.0, 176.0],
                ])
                .im_çizgisi(İmÇizgisi::yeni().yatay(İmDeğeri::Değer(150.0))),
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
