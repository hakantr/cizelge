//! Saçılım (kabarcık) grafiği örneği: iki değer ekseni, veriye bağlı sembol
//! boyutu ve öğe ipucu.
//!
//! Çalıştırma: `cargo run --example sacilim`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    let a_verisi: Vec<[f64; 2]> = vec![
        [10.0, 8.04],
        [8.07, 6.95],
        [13.0, 7.58],
        [9.05, 8.81],
        [11.0, 8.33],
        [14.0, 7.66],
        [13.4, 6.81],
        [10.0, 6.33],
        [14.0, 8.96],
        [12.5, 6.82],
        [9.15, 7.2],
        [11.5, 7.2],
        [3.03, 4.23],
        [12.2, 7.83],
        [2.02, 4.47],
        [1.05, 3.33],
        [4.05, 4.96],
        [6.03, 7.24],
        [12.0, 6.26],
        [12.0, 8.84],
        [7.08, 5.82],
        [5.02, 5.68],
    ];
    let b_verisi: Vec<[f64; 2]> = vec![
        [2.0, 2.8],
        [4.0, 3.5],
        [6.0, 4.6],
        [8.0, 5.4],
        [10.0, 5.9],
        [12.0, 6.7],
        [14.0, 7.6],
        [16.0, 8.6],
        [18.0, 9.2],
    ];

    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Saçılım Örneği"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst(28.0))
        .ızgara(Izgara::yeni().sol("7%").sağ("5%").alt(50.0).etiketi_kapsa(true))
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(
            SaçılımSerisi::yeni()
                .ad("A Kümesi")
                .veri(a_verisi)
                // Kabarcık boyutu y değeriyle büyür.
                .sembol_boyutu_işlevi(|öğe| {
                    öğe.değer.sayı().map(|y| (y * 2.4) as f32).unwrap_or(10.0)
                }),
        )
        .seri(
            SaçılımSerisi::yeni()
                .ad("B Kümesi")
                .veri(b_verisi)
                .sembol_boyutu(14.0)
                .sembol(Sembol::Elmas),
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
