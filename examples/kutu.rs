//! Kutu (boxplot) grafiği örneği: beş sayılık özetler.
//!
//! Çalıştırma: `cargo run --example kutu`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Deney Dağılımları"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(
            Izgara::yeni()
                .sol("8%")
                .sağ("6%")
                .alt(50.0)
                .etiketi_kapsa(true),
        )
        .x_ekseni(Eksen::kategori().veri(["D1", "D2", "D3", "D4", "D5"]))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(KutuSerisi::yeni().ad("Ölçüm").veri([
            [850.0, 940.0, 980.0, 1070.0, 1180.0],
            [740.0, 850.0, 900.0, 1000.0, 1090.0],
            [810.0, 920.0, 960.0, 1040.0, 1130.0],
            [720.0, 800.0, 850.0, 920.0, 1000.0],
            [880.0, 970.0, 1020.0, 1120.0, 1250.0],
        ]))
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
