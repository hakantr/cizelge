//! Pano örneği: tek pencerede dört ayrı grafik (çizgi, sütun, pasta,
//! saçılım) — gpui yerleşimiyle birleştirme.
//!
//! Çalıştırma: `cargo run --example pano`

use cizelge::hazir::*;
use gpui::{
    App, Bounds, Context, Entity, IntoElement, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, size,
};
use gpui_platform::application;

struct Pano {
    çizgi: Entity<GrafikGörünümü>,
    sütun: Entity<GrafikGörünümü>,
    pasta: Entity<GrafikGörünümü>,
    saçılım: Entity<GrafikGörünümü>,
}

impl Pano {
    fn yeni(cx: &mut Context<Self>) -> Self {
        Pano {
            çizgi: cx.new(|_| GrafikGörünümü::yeni(çizgi_seçenekleri())),
            sütun: cx.new(|_| GrafikGörünümü::yeni(sütun_seçenekleri())),
            pasta: cx.new(|_| GrafikGörünümü::yeni(pasta_seçenekleri())),
            saçılım: cx.new(|_| GrafikGörünümü::yeni(saçılım_seçenekleri())),
        }
    }
}

impl Render for Pano {
    fn render(&mut self, _pencere: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let hücre = |görünüm: Entity<GrafikGörünümü>| {
            div()
                .flex_1()
                .m_1()
                .rounded_lg()
                .bg(gpui::white())
                .child(görünüm)
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(gpui::rgb(0xf4f7fd))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .child(hücre(self.çizgi.clone()))
                    .child(hücre(self.sütun.clone())),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .child(hücre(self.pasta.clone()))
                    .child(hücre(self.saçılım.clone())),
            )
    }
}

fn çizgi_seçenekleri() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Haftalık Ziyaret"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(Izgara::yeni().sol(50.0).sağ(20.0).üst(45.0).alt(35.0))
        .x_ekseni(Eksen::kategori().veri(["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"]))
        .y_ekseni(Eksen::değer())
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Ziyaret")
                .veri([820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0])
                .yumuşat(true)
                .alan_stili(AlanStili::yeni().renk(Dolgu::doğrusal(
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    vec![
                        RenkDurağı::yeni(0.0, Renk::onaltılık(0x5070dd).alfa_ile(0.45)),
                        RenkDurağı::yeni(1.0, Renk::onaltılık(0x5070dd).alfa_ile(0.02)),
                    ],
                ))),
        )
}

fn sütun_seçenekleri() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Çeyrek Gelir"))
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Gölge),
        )
        .ızgara(Izgara::yeni().sol(55.0).sağ(20.0).üst(45.0).alt(35.0))
        .x_ekseni(Eksen::kategori().veri(["Ç1", "Ç2", "Ç3", "Ç4"]))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("2025")
                .veri([320.0, 412.0, 501.0, 634.0])
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı([4.0, 4.0, 0.0, 0.0])),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("2026")
                .veri([420.0, 482.0, 591.0, 754.0])
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı([4.0, 4.0, 0.0, 0.0])),
        )
}

fn pasta_seçenekleri() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Tarayıcı Payı"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(
            PastaSerisi::yeni()
                .ad("Tarayıcı")
                .halka("35%", "62%")
                .merkez("50%", "55%")
                .veri([
                    ("Chrome", 61.0),
                    ("Safari", 18.0),
                    ("Edge", 9.0),
                    ("Firefox", 7.0),
                    ("Diğer", 5.0),
                ]),
        )
}

fn saçılım_seçenekleri() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Boy / Kilo"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .ızgara(Izgara::yeni().sol(55.0).sağ(20.0).üst(45.0).alt(35.0))
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(SaçılımSerisi::yeni().ad("Kişiler").sembol_boyutu(12.0).veri([
            [161.2, 51.6],
            [167.5, 59.0],
            [159.5, 49.2],
            [157.0, 63.0],
            [155.8, 53.6],
            [170.0, 59.0],
            [174.0, 73.4],
            [166.2, 61.9],
            [178.2, 76.8],
            [168.9, 62.3],
            [180.3, 83.2],
            [172.7, 68.5],
        ]))
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(sınırlar)),
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(Pano::yeni),
        )
        .unwrap();
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
