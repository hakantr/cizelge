//! Kiriş (chord) diyagramı örneği.
//!
//! Çalıştırma: `cargo run --example kiris`

use cizelge::hazir::*;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Bölgeler Arası Göç"))
        .gösterge(Gösterge::yeni().veri(["Kuzey", "Güney", "Doğu", "Batı"]))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(
            KirişSerisi::yeni()
                .ad("Göç")
                .düğümler([
                    KirişDüğümü::yeni("Kuzey").öğe_stili(KirişÖğeStili::yeni().renk("#5470c6")),
                    KirişDüğümü::yeni("Güney").öğe_stili(KirişÖğeStili::yeni().renk("#91cc75")),
                    KirişDüğümü::yeni("Doğu").öğe_stili(KirişÖğeStili::yeni().renk("#fac858")),
                    KirişDüğümü::yeni("Batı").öğe_stili(KirişÖğeStili::yeni().renk("#ee6666")),
                ])
                .ayrıntılı_bağlar([
                    KirişBağı::yeni("Kuzey", "Güney", 12.0),
                    KirişBağı::yeni("Güney", "Doğu", 8.0),
                    KirişBağı::yeni("Doğu", "Kuzey", 5.0),
                    KirişBağı::yeni("Kuzey", "Batı", 6.0),
                    KirişBağı::yeni("Batı", "Güney", 4.0),
                ])
                .dolgu_açısı(4.0)
                .en_küçük_açı(3.0)
                .öğe_stili(
                    KirişÖğeStili::yeni()
                        .kenarlık_rengi("#ffffff")
                        .kenarlık_kalınlığı(1.0)
                        .kenarlık_yarıçapı([0.0, 5.0]),
                )
                .çizgi_stili(
                    KirişÇizgiStili::yeni()
                        .renk("gradient")
                        .opaklık(0.45)
                        .eğrilik(0.7),
                )
                .etiket(Etiket::yeni().göster(true).uzaklık(7.0))
                .vurgu(
                    KirişDurumu::yeni()
                        .odak(KirişVurguOdağı::Komşuluk)
                        .çizgi_stili(KirişÇizgiStili::yeni().opaklık(0.75)),
                ),
        )
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
