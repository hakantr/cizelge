//! Çoklu ızgara / çoklu eksen örneği: üstte çift y eksenli çizgiler,
//! altta sütunlar (`grid: []`, `xAxisIndex`/`yAxisIndex` karşılıkları).
//!
//! Çalıştırma: `cargo run --example coklu_izgara`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    let aylar = ["Oca", "Şub", "Mar", "Nis", "May", "Haz"];
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Sıcaklık, Nem ve Yağış"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(Gösterge::yeni().üst(28.0))
        .ızgara_ekle(Izgara::yeni().sol(70.0).sağ(70.0).üst(60.0).alt("48%"))
        .ızgara_ekle(Izgara::yeni().sol(70.0).sağ(70.0).üst("62%").alt(50.0))
        .x_ekseni_ekle(Eksen::kategori().veri(aylar).ızgara_sırası(0))
        .x_ekseni_ekle(Eksen::kategori().veri(aylar).ızgara_sırası(1))
        .y_ekseni_ekle(
            Eksen::değer()
                .ad("°C")
                .etiket_biçimleyici("{value}°")
                .ızgara_sırası(0),
        )
        .y_ekseni_ekle(
            Eksen::değer()
                .ad("%")
                .ölçekli(true)
                .bölme_çizgisi_göster(false)
                .ızgara_sırası(0),
        )
        .y_ekseni_ekle(Eksen::değer().ad("mm").ızgara_sırası(1))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Sıcaklık")
                .yumuşat(true)
                .veri([6.0, 8.0, 12.0, 17.0, 22.0, 27.0]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Nem")
                .eksenler(0, 1)
                .yumuşat(true)
                .çizgi_stili(ÇizgiStili::yeni().tür(ÇizgiTürü::Kesikli))
                .veri([68.0, 64.0, 58.0, 55.0, 52.0, 47.0]),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Yağış")
                .eksenler(1, 2)
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı([3.0, 3.0, 0.0, 0.0]))
                .veri([84.0, 72.0, 60.0, 46.0, 32.0, 18.0]),
        )
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(960.0), px(680.0)), cx);
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
