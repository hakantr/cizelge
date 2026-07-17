//! Veri yakınlaştırma (dataZoom) örneği: fare tekerleğiyle yakınlaş,
//! sürükleyerek kaydır; alttaki sürgünün tutamaçlarını çek.
//!
//! Çalıştırma: `cargo run --example yakinlastirma`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    // Belirlenimci sahte veri: 120 günlük seri.
    let veri: Vec<f64> = (0..120)
        .map(|i| {
            let t = i as f64;
            60.0 + (t * 0.12).sin() * 25.0 + (t * 0.4).cos() * 8.0
        })
        .collect();
    let günler: Vec<String> = (1..=120).map(|g| format!("G{g}")).collect();

    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Veri Yakınlaştırma"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(Izgara::yeni().sol(60.0).sağ(30.0).üst(60.0).alt(80.0))
        .x_ekseni(Eksen::kategori().veri(günler).kenar_boşluğu(false))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(30.0, 70.0))
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().aralık(30.0, 70.0))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Değer")
                .sembol_göster(false)
                .yumuşat(true)
                .alan_stili(AlanStili::yeni().opaklık(0.2))
                .veri(veri),
        )
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(960.0), px(600.0)), cx);
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
