//! Resmî `graphic` galeri kartlarının tipli Cizelge karşılıkları.

#![allow(dead_code)]

use cizelge::cizim::{DikeyHiza, YatayHiza};
use cizelge::hazir::*;
use cizelge::koordinat::Dikdörtgen;

#[path = "uyum_veri/perlin.rs"]
mod graphic_perlin;

fn kanıt_rastgele(tohum: &mut u32) -> f64 {
    *tohum = tohum.wrapping_add(0x6d2b_79f5);
    let mut t = (*tohum ^ (*tohum >> 15)).wrapping_mul(1 | *tohum);
    t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
    f64::from(t ^ (t >> 14)) / 4_294_967_296.0
}

fn düz_stil(renk: Renk) -> SahneStili {
    SahneStili {
        dolgu: Some(Dolgu::Düz(renk)),
        ..SahneStili::default()
    }
}

fn graphic_stroke_animation(genişlik: f32, yükseklik: f32) -> GrafikSeçenekleri {
    let metin = GrafikÖğesi::metin("Apache ECharts")
        .yazı_boyutu(80.0)
        .kalın(true)
        .yazı_rengi(Renk::SAYDAM)
        .yazı_hizası(YatayHiza::Orta, DikeyHiza::Orta)
        // Canvas/zrender ile ab_glyph'in 80 px sans-serif görünür glif
        // merkezi arasındaki sabit alt-piksel çapa farkı.
        .dönüşüm(YerelDönüşüm {
            x: genişlik / 2.0,
            y: yükseklik / 2.0 - 4.3,
            ..YerelDönüşüm::default()
        })
        .stil(SahneStili {
            dolgu: None,
            çizgi_rengi: Some(Renk::SİYAH),
            çizgi_kalınlığı: 1.0,
            çizgi_deseni: vec![0.0, 200.0],
            ..SahneStili::default()
        })
        .anahtar_kare_animasyonu(
            GrafikAnahtarKareAnimasyonu::yeni(3000.0)
                .döngü(true)
                .kare(
                    GrafikAnahtarKare::yeni(0.7)
                        .dolgu(Renk::SAYDAM)
                        .çizgi_deseni([200.0, 0.0], 200.0),
                )
                .kare(GrafikAnahtarKare::yeni(0.8).dolgu(Renk::SAYDAM))
                .kare(GrafikAnahtarKare::yeni(1.0).dolgu(Renk::SİYAH)),
        );

    GrafikSeçenekleri::yeni().grafik(GrafikBileşeni::yeni().öğe(metin))
}

fn graphic_loading() -> GrafikSeçenekleri {
    let çocuklar = (0..7).map(|sıra| {
        GrafikÖğesi::şekil(SahneŞekli::Dikdörtgen {
            kutu: Dikdörtgen::yeni(0.0, -40.0, 10.0, 80.0),
            yarıçap: [0.0; 4],
        })
        .dönüşüm(YerelDönüşüm {
            x: sıra as f32 * 20.0,
            ..YerelDönüşüm::default()
        })
        .stil(düz_stil(Renk::onaltılık(0x5470c6)))
        .anahtar_kare_animasyonu(
            GrafikAnahtarKareAnimasyonu::yeni(1000.0)
                .gecikme(sıra as f32 * 200.0)
                .döngü(true)
                .kare(
                    GrafikAnahtarKare::yeni(0.5)
                        .ölçek(1.0, 0.3)
                        .yumuşatma(cizelge::animasyon::Yumuşatma::KübikGiriş),
                )
                .kare(
                    GrafikAnahtarKare::yeni(1.0)
                        .ölçek(1.0, 1.0)
                        .yumuşatma(cizelge::animasyon::Yumuşatma::KübikÇıkış),
                ),
        )
    });
    let grup = GrafikÖğesi::grup(çocuklar)
        .sol(YatayKonum::Orta)
        .üst(DikeyKonum::Orta);
    GrafikSeçenekleri::yeni().grafik(GrafikBileşeni::yeni().öğe(grup))
}

fn graphic_wave_animation(genişlik: f32, yükseklik: f32) -> GrafikSeçenekleri {
    let mut tohum = 0x5eed_1234;
    let gürültü = graphic_perlin::Perlin2::yeni(kanıt_rastgele(&mut tohum));
    let mut öğeler = Vec::new();
    let mut x = 20.0;
    while x < genişlik {
        let mut y = 20.0;
        while y < yükseklik {
            let rastgele = gürültü.değer(x as f64 / 500.0, y as f64 / 500.0 + 100.0) as f32;
            öğeler.push(
                GrafikÖğesi::şekil(SahneŞekli::Daire {
                    merkez: (0.0, 0.0),
                    yarıçap: 22.0,
                })
                .dönüşüm(YerelDönüşüm {
                    x,
                    y,
                    ..YerelDönüşüm::default()
                })
                .stil(düz_stil(Renk::SİYAH))
                .anahtar_kare_animasyonu(
                    GrafikAnahtarKareAnimasyonu::yeni(4000.0)
                        .gecikme((rastgele - 1.0) * 4000.0)
                        .döngü(true)
                        .kare(
                            GrafikAnahtarKare::yeni(0.5)
                                .dolgu(Renk::BEYAZ)
                                .ölçek(5.0 / 22.0, 5.0 / 22.0)
                                .yumuşatma(cizelge::animasyon::Yumuşatma::SinüzoidalGirişÇıkış),
                        )
                        .kare(
                            GrafikAnahtarKare::yeni(1.0)
                                .dolgu(Renk::SİYAH)
                                .ölçek(1.0, 1.0)
                                .yumuşatma(cizelge::animasyon::Yumuşatma::SinüzoidalGirişÇıkış),
                        ),
                ),
            );
            y += 40.0;
        }
        x += 40.0;
    }
    GrafikSeçenekleri::yeni()
        .arkaplan(Renk::BEYAZ)
        .grafik(GrafikBileşeni::yeni().öğeler(öğeler))
}

fn line_graphic() -> GrafikSeçenekleri {
    let filigran = GrafikÖğesi::grup([
        GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(-200.0, -25.0, 400.0, 50.0))
            .z(0, 100.0, 0.0)
            .stil(düz_stil(Renk::kyma(0.0, 0.0, 0.0, 0.3))),
        GrafikÖğesi::metin("ECHARTS LINE CHART")
            .yazı_boyutu(26.0)
            .kalın(true)
            .yazı_rengi(Renk::BEYAZ)
            .yazı_hizası(YatayHiza::Orta, DikeyHiza::Orta)
            .z(0, 100.0, 0.0),
    ])
    .dönüşüm(YerelDönüşüm {
        x: 590.0,
        y: 415.0,
        dönüş: std::f32::consts::FRAC_PI_4,
        ..YerelDönüşüm::default()
    })
    .z(0, 100.0, 0.0);

    let açıklama = GrafikÖğesi::grup([
        GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(70.0, 217.5, 240.0, 90.0))
            .z(0, 100.0, 0.0)
            .stil(SahneStili {
                dolgu: Some(Dolgu::Düz(Renk::BEYAZ)),
                çizgi_rengi: Some(Renk::onaltılık(0x555555)),
                çizgi_kalınlığı: 1.0,
                gölge_rengi: Some(Renk::kyma(0.0, 0.0, 0.0, 0.2)),
                gölge_bulanıklığı: 8.0,
                gölge_kayması: (3.0, 3.0),
                ..SahneStili::default()
            }),
        GrafikÖğesi::metin(
            "xAxis represents temperature in °C,\n\
             yAxis represents altitude in km, An\n\
             image watermark in the upper right,\n\
             This text block can be placed in any\n\
             place",
        )
        .dönüşüm(YerelDönüşüm {
            x: 87.0,
            y: 229.5,
            ..YerelDönüşüm::default()
        })
        .yazı_boyutu(14.0)
        .yazı_ailesi("Times New Roman")
        .satır_yüksekliği(14.0)
        .yazı_rengi(Renk::onaltılık(0x333333))
        .z(0, 100.0, 0.0),
    ]);

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ızgara(
            Izgara::yeni()
                .sol(Uzunluk::Yüzde(3.0))
                .sağ(Uzunluk::Yüzde(4.0))
                .alt(Uzunluk::Yüzde(3.0))
                .etiketi_kapsa(true),
        )
        .x_ekseni(Eksen::değer().etiket_biçimleyici("{value} °C"))
        .y_ekseni(
            Eksen::kategori()
                .çizgi(EksenÇizgisi::yeni().sıfır(EksenSıfırKipi::Kapalı))
                .etiket_biçimleyici("{value} km")
                .kenar_boşluğu(true)
                .veri(["0", "10", "20", "30", "40", "50", "60", "70", "80"]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("高度(km)与气温(°C)变化关系")
                .yumuşat(true)
                .veri([15.0, -50.0, -56.5, -46.5, -22.1, -2.5, -27.7, -55.7, -76.5]),
        )
        .grafik(GrafikBileşeni::yeni().öğeler([filigran, açıklama]))
}

const SÜRÜKLENEBİLİR_VERİ: [[f64; 2]; 5] = [
    [40.0, -10.0],
    [-30.0, -5.0],
    [-76.5, 20.0],
    [-63.5, 40.0],
    [-22.1, 50.0],
];

fn line_draggable(durum: &str) -> GrafikSeçenekleri {
    let mut veri = SÜRÜKLENEBİLİR_VERİ;
    if durum == "sürükle" {
        veri[2] = [-40.0, 30.0];
    }
    let grid = Dikdörtgen::yeni(105.0, 42.0, 525.0, 420.0);
    let piksel = |değer: [f64; 2]| {
        (
            grid.x + ((değer[0] + 100.0) / 170.0) as f32 * grid.genişlik,
            grid.alt() - ((değer[1] + 30.0) / 90.0) as f32 * grid.yükseklik,
        )
    };
    let tutamaçlar = veri.into_iter().enumerate().map(|(sıra, değer)| {
        let konum = piksel(değer);
        GrafikÖğesi::şekil(SahneŞekli::Daire {
            merkez: (0.0, 0.0),
            yarıçap: 10.0,
        })
        .kimlik(format!("drag-point-{sıra}"))
        .dönüşüm(YerelDönüşüm {
            x: konum.0,
            y: konum.1,
            ..YerelDönüşüm::default()
        })
        .görünmez(true)
        .sürüklenebilir(true)
        .z(0, 100.0, 0.0)
    });

    let x_sürgü = VeriYakınlaştırma::sürgü()
        .x_eksen_sırası(0)
        .süzme_kipi(YakınlaştırmaSüzmeKipi::Yok);
    let y_sürgü = VeriYakınlaştırma::sürgü()
        .y_eksen_sırası(0)
        .süzme_kipi(YakınlaştırmaSüzmeKipi::Yok);
    let x_iç = VeriYakınlaştırma::iç()
        .x_eksen_sırası(0)
        .süzme_kipi(YakınlaştırmaSüzmeKipi::Yok);
    let y_iç = VeriYakınlaştırma::iç()
        .y_eksen_sırası(0)
        .süzme_kipi(YakınlaştırmaSüzmeKipi::Yok);

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Try Dragging these Points")
                .sol(YatayKonum::Orta)
                .iç_boşluk(15.0),
        )
        .ızgara(
            Izgara::yeni()
                .üst(Uzunluk::Yüzde(8.0))
                .alt(Uzunluk::Yüzde(12.0)),
        )
        .x_ekseni(
            Eksen::değer()
                .en_az(-100.0)
                .en_çok(70.0)
                .çizgi(EksenÇizgisi::yeni().sıfır(EksenSıfırKipi::Kapalı)),
        )
        .y_ekseni(
            Eksen::değer()
                .en_az(-30.0)
                .en_çok(60.0)
                .çizgi(EksenÇizgisi::yeni().sıfır(EksenSıfırKipi::Kapalı)),
        )
        .veri_yakınlaştırma(x_sürgü)
        .veri_yakınlaştırma(y_sürgü)
        .veri_yakınlaştırma(x_iç)
        .veri_yakınlaştırma(y_iç)
        .seri(
            ÇizgiSerisi::yeni()
                .yumuşat(true)
                .sembol_boyutu(20.0)
                .veri(veri),
        )
        .grafik(GrafikBileşeni::yeni().öğeler(tutamaçlar))
}

pub(crate) fn resmi(
    id: &str,
    durum: &str,
    genişlik: f32,
    yükseklik: f32,
) -> Option<GrafikSeçenekleri> {
    match id {
        "graphic-stroke-animation" => Some(graphic_stroke_animation(genişlik, yükseklik)),
        "graphic-loading" => Some(graphic_loading()),
        "line-graphic" => Some(line_graphic()),
        "graphic-wave-animation" => Some(graphic_wave_animation(genişlik, yükseklik)),
        "line-draggable" => Some(line_draggable(durum)),
        _ => None,
    }
}

// `examples/*.rs` Cargo tarafından bağımsız örnek hedefi olarak da
// keşfedilir; asıl kullanım `uyum_fixture` içindeki modüldür.
#[allow(dead_code)]
fn main() {}
