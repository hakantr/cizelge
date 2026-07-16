//! Çizgi grafiği örneği: yumuşatılmış çizgiler, alan dolgusu, gösterge ve
//! eksen tetiklemeli ipucu.
//!
//! Çalıştırma: `cargo run --example cizgi`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(
            Başlık::yeni()
                .metin("Haftalık Sıcaklık")
                .alt_metin("çizelge örneği"),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(Gösterge::yeni().üst(28.0))
        .ızgara(Izgara::yeni().sol("8%").sağ("5%").alt(50.0).etiketi_kapsa(true))
        .x_ekseni(
            Eksen::kategori()
                .veri(["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"])
                .kenar_boşluğu(false),
        )
        .y_ekseni(Eksen::değer().etiket_biçimleyici("{value} °C"))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("En Yüksek")
                .veri([11.0, 13.0, 15.0, 13.0, 12.0, 16.0, 21.0])
                .yumuşat(true),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("En Düşük")
                .veri([1.0, -2.0, 2.0, 5.0, 3.0, 2.0, 0.0])
                .yumuşat(true)
                .çizgi_stili(ÇizgiStili::yeni().tür(ÇizgiTürü::Kesikli)),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Ortalama")
                .veri([6.0, 5.5, 8.5, 9.0, 7.5, 9.0, 10.5])
                .yumuşat(true)
                .alan_stili(AlanStili::yeni().opaklık(0.25)),
        )
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(900.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(sınırlar)),
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(|_| GrafikGörünümü::yeni(seçenekler())),
        )
        .unwrap();
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
