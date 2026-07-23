//! Olay sistemi örneği: dilim/sütun tıklamaları, gösterge ve görsel eşleme
//! değişimleri `GrafikOlayı` olarak yayımlanır; alt çubukta son olay gösterilir.
//!
//! Çalıştırma: `cargo run --example olaylar`

use cizelge::hazir::*;
use gpui::{
    App, Bounds, Context, Entity, IntoElement, Render, SharedString, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};
use gpui_platform::application;

struct Kök {
    grafik: Entity<GrafikGörünümü>,
    son_olay: SharedString,
}

impl Kök {
    fn yeni(cx: &mut Context<Self>) -> Self {
        let grafik = cx.new(|_| GrafikGörünümü::yeni(seçenekler()));
        cx.subscribe(&grafik, |bu: &mut Kök, _, olay: &GrafikOlayı, cx| {
            bu.son_olay = match olay {
                GrafikOlayı::ÖğeTıklandı {
                    seri_adı,
                    ad,
                    değer,
                    ..
                } => format!(
                    "Tıklandı → seri: {}, öğe: {}, değer: {}",
                    seri_adı.as_deref().unwrap_or("-"),
                    ad.as_deref().unwrap_or("-"),
                    değer.map(|d| d.to_string()).unwrap_or_else(|| "-".into())
                )
                .into(),
                GrafikOlayı::AğaçGenişletmeDeğişti {
                    ad, daraltılmış, ..
                } => format!(
                    "Tree → {ad} {}",
                    if *daraltılmış {
                        "daraltıldı"
                    } else {
                        "genişletildi"
                    }
                )
                .into(),
                GrafikOlayı::AğaçGezinmeDeğişti {
                    seri_sırası,
                    kayma_x,
                    kayma_y,
                    ölçek,
                } => format!("Tree {seri_sırası} roam → ({kayma_x:.1}, {kayma_y:.1}), {ölçek:.2}×")
                    .into(),
                GrafikOlayı::SankeyDüğümüSürüklendi {
                    seri_sırası,
                    ad,
                    yerel_x,
                    yerel_y,
                    ..
                } => format!("Sankey {seri_sırası} düğümü → {ad} ({yerel_x:.3}, {yerel_y:.3})")
                    .into(),
                GrafikOlayı::AğaçHaritasıKöküDeğişti {
                    seri_sırası,
                    yol,
                    yön,
                    ..
                } => format!(
                    "Treemap {seri_sırası} kökü → {} ({yön:?})",
                    if yol.is_empty() {
                        "Tümü".to_owned()
                    } else {
                        yol.join(" / ")
                    }
                )
                .into(),
                GrafikOlayı::GüneşPatlamasıKöküDeğişti {
                    seri_sırası,
                    yol,
                    yön,
                    ..
                } => format!(
                    "Sunburst {seri_sırası} kökü → {} ({yön:?})",
                    if yol.is_empty() {
                        "Tümü".to_owned()
                    } else {
                        yol.join(" / ")
                    }
                )
                .into(),
                GrafikOlayı::Bağlantıİstendi {
                    seri_sırası,
                    url,
                    hedef,
                    ..
                } => format!("Treemap {seri_sırası} bağlantısı → {url} ({hedef})").into(),
                GrafikOlayı::MatrisHücresiTıklandı {
                    bileşen_sırası,
                    hedef_türü,
                    ad,
                    değer,
                    koordinat: [x, y],
                } => format!(
                    "Matrix {bileşen_sırası} → {hedef_türü:?} [{}, {}], ad: {}, değer: {}",
                    x,
                    y,
                    ad.as_deref().unwrap_or("-"),
                    değer.as_deref().unwrap_or("-")
                )
                .into(),
                GrafikOlayı::GrafikÖğesiTıklandı { kimlik, ad } => format!(
                    "Graphic tıklandı → kimlik: {}, ad: {}",
                    kimlik.as_deref().unwrap_or("-"),
                    ad.as_deref().unwrap_or("-")
                )
                .into(),
                GrafikOlayı::GöstergeDeğişti { ad, görünür } => format!(
                    "Gösterge → {ad} artık {}",
                    if *görünür { "görünür" } else { "gizli" }
                )
                .into(),
                GrafikOlayı::YakınlaştırmaDeğişti {
                    sıra,
                    başlangıç,
                    bitiş,
                } => format!("Yakınlaştırma {sıra} → %{başlangıç:.0}–%{bitiş:.0}").into(),
                GrafikOlayı::GörselAralıkDeğişti { sıra, alt, üst } => {
                    format!("Görsel eşleme {sıra} → {alt:.2}–{üst:.2}").into()
                }
                GrafikOlayı::GörselParçalarDeğişti { sıra, seçili } => format!(
                    "Parçalı eşleme {sıra} → {}/{} parça seçili",
                    seçili.values().filter(|açık| **açık).count(),
                    seçili.len()
                )
                .into(),
                GrafikOlayı::ParalelAlanSeçildi {
                    eksen_sırası,
                    aralıklar,
                } => format!(
                    "ParallelAxis {eksen_sırası} → {} etkin aralık",
                    aralıklar.len()
                )
                .into(),
                GrafikOlayı::ParalelEksenGenişletildi {
                    paralel_sırası,
                    pencere: [başlangıç, bitiş],
                } => format!(
                    "Parallel {paralel_sırası} → genişletme penceresi {başlangıç:.1}–{bitiş:.1}"
                )
                .into(),
                GrafikOlayı::FırçaSeçildi { öğeler } => {
                    format!("Fırça → {} öğe seçildi", öğeler.len()).into()
                }
                GrafikOlayı::GeriYüklendi => "Geri yüklendi".into(),
                GrafikOlayı::VeriGörünümüİstendi => "Veri görünümü istendi".into(),
                GrafikOlayı::SihirliTürİstendi { tür } => {
                    format!("Sihirli tür → {tür:?}").into()
                }
                GrafikOlayı::ZamanKaresiDeğişti { sıra } => {
                    format!("Zaman şeridi → {}. kare", sıra.saturating_add(1)).into()
                }
                GrafikOlayı::SvgKaydedildi { yol } => format!("SVG kaydedildi → {yol}").into(),
                GrafikOlayı::PngKaydedildi { yol } => format!("PNG kaydedildi → {yol}").into(),
            };
            cx.notify();
        })
        .detach();
        Kök {
            grafik,
            son_olay: "Bir dilime, sütuna ya da gösterge öğesine tıklayın".into(),
        }
    }
}

impl Render for Kök {
    fn render(&mut self, _pencere: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(gpui::white())
            .child(div().flex_1().child(self.grafik.clone()))
            .child(
                div()
                    .h(px(36.0))
                    .px_4()
                    .flex()
                    .items_center()
                    .bg(gpui::rgb(0xf4f7fd))
                    .text_sm()
                    .child(self.son_olay.clone()),
            )
    }
}

fn seçenekler() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Olay Örneği"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst(28.0))
        .ızgara(Izgara::yeni().sol(50.0).sağ("42%").üst(70.0).alt(40.0))
        .x_ekseni(Eksen::kategori().veri(["Pzt", "Sal", "Çar", "Per", "Cum"]))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("Satış")
                .veri([5.0, 20.0, 36.0, 10.0, 14.0]),
        )
        .seri(
            PastaSerisi::yeni()
                .ad("Pay")
                .halka("18%", "30%")
                .merkez("78%", "55%")
                .veri([("Doğrudan", 335.0), ("E-posta", 310.0), ("Reklam", 234.0)]),
        )
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
            |_, cx| cx.new(Kök::yeni),
        )
        .unwrap_or_else(|hata| {
            eprintln!("Pencere açılamadı: {hata}");
            std::process::exit(1);
        });
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
