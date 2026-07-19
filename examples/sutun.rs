//! Sütun grafiği örneği: gruplu + yığılmış sütunlar, gölge eksen imleci.
//!
//! Çalıştırma: `cargo run --example sutun`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Aylık Yağış ve Buharlaşma"))
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Gölge),
        )
        .gösterge(Gösterge::yeni().üst(28.0))
        .ızgara(
            Izgara::yeni()
                .sol("6%")
                .sağ("4%")
                .alt(45.0)
                .etiketi_kapsa(true),
        )
        .x_ekseni(Eksen::kategori().veri([
            "Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas", "Ara",
        ]))
        .y_ekseni(Eksen::değer().etiket_biçimleyici("{value} mm"))
        .seri(
            SütunSerisi::yeni()
                .ad("Yağış")
                .veri([
                    42.0, 38.5, 61.2, 84.6, 91.0, 130.4, 175.6, 182.2, 88.7, 39.8, 24.0, 43.3,
                ])
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı([3.0, 3.0, 0.0, 0.0])),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Buharlaşma")
                .veri([
                    26.0, 29.9, 50.0, 66.4, 78.7, 105.6, 135.6, 162.2, 69.7, 30.6, 12.0, 23.3,
                ])
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı([3.0, 3.0, 0.0, 0.0])),
        )
        .seri(SütunSerisi::yeni().ad("Sulama").yığın("kaynak").veri([
            12.0, 14.0, 18.0, 22.0, 26.0, 32.0, 41.0, 44.0, 28.0, 16.0, 9.0, 11.0,
        ]))
        .seri(SütunSerisi::yeni().ad("Şebeke").yığın("kaynak").veri([
            8.0, 9.5, 12.0, 15.5, 18.0, 22.5, 29.0, 31.0, 19.5, 11.0, 6.5, 7.5,
        ]))
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(1000.0), px(620.0)), cx);
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
