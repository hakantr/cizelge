//! Resmî ECharts Custom galeri kartlarının ortak `renderItem` API'siyle
//! hazırlanan belirlenimci uyum fixture'ları.

use std::sync::Arc;

use cizelge::cizim::{DikeyHiza, YatayHiza, Yol};
use cizelge::hazir::*;
use cizelge::koordinat::Dikdörtgen;

pub fn resmi(id: &str, durum: &str) -> Option<GrafikSeçenekleri> {
    match id {
        "custom-profit" => Some(custom_profit()),
        "custom-error-scatter" => Some(custom_error_scatter()),
        "custom-bar-trend" => Some(custom_bar_trend()),
        "custom-cartesian-polygon" => Some(custom_cartesian_polygon()),
        "custom-error-bar" => Some(custom_error_bar()),
        "custom-profile" => Some(custom_profile()),
        "cycle-plot" => Some(cycle_plot()),
        "custom-polar-heatmap" => Some(custom_polar_heatmap()),
        "flame-graph" => Some(flame_graph()),
        "custom-gauge" => Some(custom_gauge()),
        "pie-parliament-transition" => Some(pie_parliament_transition(durum)),
        "wind-barb" => Some(wind_barb()),
        "custom-gantt-flight" => Some(custom_gantt_flight()),
        "circle-packing-with-d3" => Some(circle_packing_with_d3()),
        "custom-spiral-race" => Some(custom_spiral_race(durum)),
        _ => None,
    }
}

fn resmi_sayısal_dizi(id: &str, değişken: &str) -> Vec<Vec<f64>> {
    let yol = std::env::current_dir()
        .expect("çalışma dizini")
        .join("../echarts-examples/public/examples/ts")
        .join(format!("{id}.ts"));
    let kaynak = std::fs::read_to_string(&yol)
        .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", yol.display()));
    let işaret = format!("const {değişken} = ");
    let başlangıç = kaynak
        .find(&işaret)
        .map(|sıra| sıra + işaret.len())
        .unwrap_or_else(|| panic!("{id}: {değişken} dizisi bulunamadı"));
    let kalan = &kaynak[başlangıç..];
    let bitiş = kalan
        .find(";")
        .unwrap_or_else(|| panic!("{id}: {değişken} dizi sonu bulunamadı"));
    serde_json::from_str(kalan[..bitiş].trim())
        .unwrap_or_else(|hata| panic!("{id}: {değişken} JSON çözülemedi: {hata}"))
}

fn rastgele(tohum: &mut u32) -> f64 {
    *tohum = tohum.wrapping_add(0x6d2b_79f5);
    let mut t = (*tohum ^ (*tohum >> 15)).wrapping_mul(1 | *tohum);
    t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
    f64::from(t ^ (t >> 14)) / 4_294_967_296.0
}

fn yuvarla(değer: f64, basamak: u32) -> f64 {
    let çarpan = 10_f64.powi(basamak as i32);
    (değer * çarpan).round() / çarpan
}

fn düz_renk(dolgu: &Dolgu) -> Renk {
    match dolgu {
        Dolgu::Düz(renk) => *renk,
        _ => Renk::SİYAH,
    }
}

fn çizgi_stili(renk: impl Into<Renk>, kalınlık: f32) -> SahneStili {
    SahneStili {
        dolgu: None,
        çizgi_rengi: Some(renk.into()),
        çizgi_kalınlığı: kalınlık,
        ..SahneStili::default()
    }
}

fn doğrudan_metin_öğesi(
    metin: impl Into<String>,
    konum: (f32, f32),
    yatay: YatayHiza,
    dikey: DikeyHiza,
    boyut: f32,
    renk: impl Into<Renk>,
) -> GrafikÖğesi {
    let mut metin = SahneMetni::yeni(metin, konum);
    metin.yatay = yatay;
    metin.dikey = dikey;
    metin.boyut = boyut;
    metin.renk = renk.into();
    GrafikÖğesi::metin("")
        .sessiz(true)
        .dönüşüm(YerelDönüşüm::default())
        .stil(SahneStili {
            dolgu: None,
            ..SahneStili::default()
        })
        .with_metin(metin)
}

fn png_sahne_resmi(kaynak: impl AsRef<std::path::Path>, kutu: Dikdörtgen) -> SahneResmi {
    let kaynak = kaynak.as_ref();
    let görüntü = image::open(kaynak)
        .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", kaynak.display()))
        .to_rgba8();
    let (genişlik, yükseklik) = görüntü.dimensions();
    SahneResmi::rgba(
        kaynak.display().to_string(),
        kutu,
        genişlik,
        yükseklik,
        görüntü.into_raw(),
    )
    .expect("geçerli PNG sahne resmi")
}

fn iso_utc_milisaniye(metin: &str) -> f64 {
    let sayı = |baş: usize, son: usize| {
        metin
            .get(baş..son)
            .and_then(|parça| parça.parse::<i64>().ok())
            .unwrap_or(0)
    };
    let yıl = sayı(0, 4);
    let ay = sayı(5, 7);
    let gün = sayı(8, 10);
    let saat = sayı(11, 13);
    let dakika = sayı(14, 16);
    let saniye = sayı(17, 19);
    // Howard Hinnant'ın `days_from_civil` dönüşümü: Gregoryen
    // tarihi Unix epoch'undan itibaren gün sayısına çevirir.
    let düzeltilmiş_yıl = yıl - i64::from(ay <= 2);
    let çağ = düzeltilmiş_yıl.div_euclid(400);
    let çağ_yılı = düzeltilmiş_yıl - çağ * 400;
    let mart_ayı = ay + if ay > 2 { -3 } else { 9 };
    let yıl_günü = (153 * mart_ayı + 2) / 5 + gün - 1;
    let çağ_günü = çağ_yılı * 365 + çağ_yılı / 4 - çağ_yılı / 100 + yıl_günü;
    let günler = çağ * 146_097 + çağ_günü - 719_468;
    (günler * 86_400_000 + saat * 3_600_000 + dakika * 60_000 + saniye * 1_000) as f64
}

trait Metinİçeriği {
    fn with_metin(self, metin: SahneMetni) -> Self;
}

impl Metinİçeriği for GrafikÖğesi {
    fn with_metin(mut self, metin: SahneMetni) -> Self {
        self.içerik = GrafikÖğeİçeriği::Metin(metin);
        self
    }
}

fn custom_profit() -> GrafikSeçenekleri {
    let renkler = [
        "#4f81bd", "#c0504d", "#9bbb59", "#604a7b", "#948a54", "#e46c0b",
    ];
    let satırlar = [
        (10.0, 16.0, 3.0, "A"),
        (16.0, 18.0, 15.0, "B"),
        (18.0, 26.0, 12.0, "C"),
        (26.0, 32.0, 22.0, "D"),
        (32.0, 56.0, 7.0, "E"),
        (56.0, 62.0, 17.0, "F"),
    ];
    let veri = satırlar
        .iter()
        .zip(renkler)
        .map(|((baş, son, kâr, ad), renk)| {
            VeriÖğesi::adlı(
                *ad,
                VeriDeğeri::KarmaDizi(vec![
                    (*baş).into(),
                    (*son).into(),
                    (*kâr).into(),
                    (*ad).into(),
                ]),
            )
            .stil(ÖğeStili::yeni().renk(renk))
        })
        .collect::<Vec<_>>();

    let özel = ÖzelSeri::yeni()
        .veri(veri)
        .kodla("x", [0, 1])
        .kodla("y", [2])
        .kodla("tooltip", [0, 1, 2])
        .öğe_çizimi(|api| {
            let baş = api.sayısal_değer(0usize)?;
            let son = api.sayısal_değer(1usize)?;
            let kâr = api.sayısal_değer(2usize)?;
            let nokta = api.koordinat(&[baş, kâr])?;
            let boyut = api.boyut(&[son - baş, kâr], None);
            let kutu = Dikdörtgen::yeni(nokta[0], nokta[1], boyut[0], boyut[1]);
            let renk = düz_renk(&api.renk);
            Some(GrafikÖğesi::grup([
                GrafikÖğesi::dikdörtgen(kutu).stil(api.stil()),
                doğrudan_metin_öğesi(
                    format!("{kâr:.0}"),
                    (kutu.x + kutu.genişlik / 2.0, kutu.y - 5.0),
                    YatayHiza::Orta,
                    DikeyHiza::Alt,
                    12.0,
                    renk,
                ),
            ]))
        });

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Profit").sol("center").üst(25))
        .ipucu(İpucu::yeni())
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer())
        .seri(özel)
}

const ERROR_DIMENSIONS: [&str; 7] = [
    "name",
    "Price",
    "Prime cost",
    "Prime cost min",
    "Prime cost max",
    "Price min",
    "Price max",
];

const ERROR_ROWS: [(&str, [f64; 6]); 17] = [
    (
        "Blouse \"Blue Viola\"",
        [101.88, 99.75, 76.75, 116.75, 69.88, 119.88],
    ),
    (
        "Dress \"Daisy\"",
        [155.8, 144.03, 126.03, 156.03, 129.8, 188.8],
    ),
    (
        "Trousers \"Cutesy Classic\"",
        [203.25, 173.56, 151.56, 187.56, 183.25, 249.25],
    ),
    (
        "Dress \"Morning Dew\"",
        [256.0, 120.5, 98.5, 136.5, 236.0, 279.0],
    ),
    (
        "Turtleneck \"Dark Chocolate\"",
        [408.89, 294.75, 276.75, 316.75, 385.89, 427.89],
    ),
    (
        "Jumper \"Early Spring\"",
        [427.36, 430.24, 407.24, 452.24, 399.36, 461.36],
    ),
    (
        "Breeches \"Summer Mood\"",
        [356.0, 135.5, 123.5, 151.5, 333.0, 387.0],
    ),
    (
        "Dress \"Mauve Chamomile\"",
        [406.0, 95.5, 73.5, 111.5, 366.0, 429.0],
    ),
    (
        "Dress \"Flying Tits\"",
        [527.36, 503.24, 488.24, 525.24, 485.36, 551.36],
    ),
    (
        "Dress \"Singing Nightingales\"",
        [587.36, 543.24, 518.24, 555.24, 559.36, 624.36],
    ),
    (
        "Sundress \"Cloudy weather\"",
        [603.36, 407.24, 392.24, 419.24, 581.36, 627.36],
    ),
    (
        "Sundress \"East motives\"",
        [633.36, 477.24, 445.24, 487.24, 594.36, 652.36],
    ),
    (
        "Sweater \"Cold morning\"",
        [517.36, 437.24, 416.24, 454.24, 488.36, 565.36],
    ),
    (
        "Trousers \"Lavender Fields\"",
        [443.36, 387.24, 370.24, 413.24, 412.36, 484.36],
    ),
    (
        "Jumper \"Coffee with Milk\"",
        [543.36, 307.24, 288.24, 317.24, 509.36, 574.36],
    ),
    (
        "Blouse \"Blooming Cactus\"",
        [790.36, 277.24, 254.24, 295.24, 764.36, 818.36],
    ),
    (
        "Sweater \"Fluffy Comfort\"",
        [790.34, 678.34, 660.34, 690.34, 762.34, 824.34],
    ),
];

fn error_custom_verisi() -> Vec<VeriÖğesi> {
    ERROR_ROWS
        .iter()
        .map(|(ad, sayılar)| {
            let mut değerler = vec![VeriDeğeri::from(*ad)];
            değerler.extend(sayılar.iter().copied().map(VeriDeğeri::from));
            VeriÖğesi::adlı(*ad, VeriDeğeri::KarmaDizi(değerler)).boyutlar(
                ERROR_DIMENSIONS.iter().enumerate().map(|(sıra, ad)| {
                    let değer = if sıra == 0 {
                        VeriDeğeri::from((*ad).to_owned())
                    } else {
                        VeriDeğeri::from(sayılar[sıra - 1])
                    };
                    ((*ad).to_owned(), değer)
                }),
            )
        })
        .collect()
}

fn error_scatter_verisi() -> Vec<VeriÖğesi> {
    ERROR_ROWS
        .iter()
        .map(|(ad, v)| {
            VeriÖğesi::adlı(*ad, [v[1], v[0]]).boyutlar([
                ("Prime cost".to_owned(), VeriDeğeri::from(v[1])),
                ("Price".to_owned(), VeriDeğeri::from(v[0])),
            ])
        })
        .collect()
}

fn custom_error_scatter() -> GrafikSeçenekleri {
    let özel = ÖzelSeri::yeni()
        .ad("error")
        .veri(error_custom_verisi())
        .kodla("x", [2, 3, 4])
        .kodla("y", [1, 5, 6])
        .z(100)
        .öğe_çizimi(|api| {
            let x = api.sayısal_değer(2usize)?;
            let y = api.sayısal_değer(1usize)?;
            let x0 = api.sayısal_değer(3usize)?;
            let x1 = api.sayısal_değer(4usize)?;
            let y0 = api.sayısal_değer(5usize)?;
            let y1 = api.sayısal_değer(6usize)?;
            let renk = düz_renk(&api.renk);
            let stil = çizgi_stili(renk, 1.0);
            let merkez = api.koordinat(&[x, y])?;
            let x_düşük = api.koordinat(&[x0, y])?;
            let x_yüksek = api.koordinat(&[x1, y])?;
            let y_düşük = api.koordinat(&[x, y0])?;
            let y_yüksek = api.koordinat(&[x, y1])?;
            let çizgi = |a, b| {
                GrafikÖğesi::şekil(SahneŞekli::Çizgi {
                    başlangıç: a,
                    bitiş: b,
                })
                .stil(stil.clone())
            };
            Some(GrafikÖğesi::grup([
                çizgi(
                    (x_yüksek[0], merkez[1] - 5.0),
                    (x_yüksek[0], merkez[1] + 5.0),
                ),
                çizgi((x_yüksek[0], merkez[1]), (x_düşük[0], merkez[1])),
                çizgi((x_düşük[0], merkez[1] - 5.0), (x_düşük[0], merkez[1] + 5.0)),
                çizgi(
                    (merkez[0] - 5.0, y_yüksek[1]),
                    (merkez[0] + 5.0, y_yüksek[1]),
                ),
                çizgi((merkez[0], y_yüksek[1]), (merkez[0], y_düşük[1])),
                çizgi((merkez[0] - 5.0, y_düşük[1]), (merkez[0] + 5.0, y_düşük[1])),
            ]))
        });
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(Gösterge::yeni().veri(["bar", "error"]).üst(25))
        .ipucu(İpucu::yeni())
        .ızgara(Izgara::yeni().üst(60))
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü())
        .veri_yakınlaştırma(VeriYakınlaştırma::iç())
        .x_ekseni(Eksen::değer())
        .y_ekseni(Eksen::değer())
        .seri(
            SaçılımSerisi::yeni()
                .ad("error")
                .eşle("Prime cost", "Price")
                .öğe_stili(ÖğeStili::yeni().renk("#77bef7"))
                .veri(error_scatter_verisi()),
        )
        .seri(özel)
}

fn custom_bar_trend() -> GrafikSeçenekleri {
    const YIL: usize = 7;
    const KATEGORİ: usize = 30;
    let mut tohum = 0x5eed_1234;
    let mut veri_listeleri: Vec<Vec<f64>> = vec![Vec::new(); YIL];
    let mut özel_veri = Vec::new();
    let kategoriler = (0..KATEGORİ)
        .map(|sıra| format!("category{sıra}"))
        .collect::<Vec<_>>();
    for kategori in 0..KATEGORİ {
        let ilk = rastgele(&mut tohum) * 1000.0;
        let mut satır = vec![kategori as f64];
        for yıl in 0..YIL {
            let değer = if yıl == 0 {
                yuvarla(ilk, 2)
            } else {
                yuvarla(
                    (veri_listeleri[yıl - 1][kategori] + (rastgele(&mut tohum) - 0.5) * 200.0)
                        .max(0.0),
                    2,
                )
            };
            veri_listeleri[yıl].push(değer);
            satır.push(değer);
        }
        özel_veri.push(VeriÖğesi::yeni(satır));
    }
    let özel = ÖzelSeri::yeni()
        .ad("trend")
        .veri(özel_veri)
        .kodla("x", [0])
        .kodla("y", 1..=YIL)
        .z(100)
        .öğe_çizimi(|api| {
            let x = api.sayısal_değer(0usize)?;
            let mut noktalar = Vec::new();
            for seri_sırası in api.güncel_seri_sıraları() {
                if *seri_sırası == api.seri_sırası {
                    continue;
                }
                let değer = api.sayısal_değer(*seri_sırası)?;
                let mut nokta = api.koordinat(&[x, değer])?;
                if let Some(yerleşim) = api.sütun_yerleşimi(*seri_sırası) {
                    nokta[0] += yerleşim.kaydırma + yerleşim.genişlik / 2.0;
                }
                nokta[1] -= 20.0;
                noktalar.push((nokta[0], nokta[1]));
            }
            Some(
                GrafikÖğesi::şekil(SahneŞekli::ÇokluÇizgi(noktalar))
                    .stil(çizgi_stili(düz_renk(&api.renk), 2.0)),
            )
        });
    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(
            Gösterge::yeni()
                .veri(
                    std::iter::once("trend".to_owned()).chain((2010..2017).map(|y| y.to_string())),
                )
                .üst(30),
        )
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().aralık(50.0, 70.0))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(50.0, 70.0))
        .x_ekseni(Eksen::kategori().veri(kategoriler))
        .y_ekseni(Eksen::değer())
        .seri(özel);
    for (yıl, veri) in veri_listeleri.into_iter().enumerate() {
        seçenekler = seçenekler.seri(
            SütunSerisi::yeni()
                .ad((2010 + yıl).to_string())
                .öğe_stili(ÖğeStili::yeni().opaklık(0.5))
                .veri(veri),
        );
    }
    seçenekler
}

fn custom_cartesian_polygon() -> GrafikSeçenekleri {
    let mut tohum = 0x5eed_1234;
    let veri = (0..7)
        .map(|_| {
            [
                (rastgele(&mut tohum) * 100.0).round(),
                (rastgele(&mut tohum) * 400.0).round(),
            ]
        })
        .collect::<Vec<_>>();
    let tümü = Arc::new(veri.clone());
    let özel = ÖzelSeri::yeni()
        .veri(veri)
        .kodla("x", [0])
        .kodla("y", [1])
        .kırp(true)
        .öğe_çizimi(move |api| {
            if api.veri_sırası != 0 {
                return None;
            }
            let noktalar = tümü
                .iter()
                .filter_map(|değer| api.koordinat(değer))
                .map(|nokta| (nokta[0], nokta[1]))
                .collect::<Vec<_>>();
            let renk = düz_renk(&api.renk);
            Some(
                GrafikÖğesi::şekil(SahneŞekli::Çokgen(noktalar)).stil(SahneStili {
                    dolgu: Some(Dolgu::Düz(renk)),
                    çizgi_rengi: Some(renk.açıklık_ile(0.58)),
                    çizgi_kalınlığı: 1.0,
                    ..SahneStili::default()
                }),
            )
        });
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(Gösterge::yeni().veri(["bar", "error"]))
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().süzme_kipi(YakınlaştırmaSüzmeKipi::Yok))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().süzme_kipi(YakınlaştırmaSüzmeKipi::Yok))
        .x_ekseni(Eksen::değer())
        .y_ekseni(Eksen::değer())
        .seri(özel)
}

fn custom_error_bar() -> GrafikSeçenekleri {
    let mut tohum = 0x5eed_1234;
    let mut kategoriler = Vec::new();
    let mut hata = Vec::new();
    let mut sütun = Vec::new();
    for sıra in 0..100 {
        let değer = rastgele(&mut tohum) * 1000.0;
        kategoriler.push(format!("category{sıra}"));
        hata.push([
            sıra as f64,
            (değer - rastgele(&mut tohum) * 100.0).max(0.0).round(),
            (değer + rastgele(&mut tohum) * 80.0).round(),
        ]);
        sütun.push(yuvarla(değer, 2));
    }
    let özel = ÖzelSeri::yeni()
        .ad("error")
        .veri(hata)
        .kodla("x", [0])
        .kodla("y", [1, 2])
        .z(100)
        .öğe_çizimi(|api| {
            let x = api.sayısal_değer(0usize)?;
            let yüksek = api.sayısal_değer(1usize)?;
            let düşük = api.sayısal_değer(2usize)?;
            let yüksek_nokta = api.koordinat(&[x, yüksek])?;
            let düşük_nokta = api.koordinat(&[x, düşük])?;
            let yarım = api.boyut(&[1.0, 0.0], None)[0] * 0.1;
            let stil = çizgi_stili(düz_renk(&api.renk), 1.5);
            let çizgi = |a, b| {
                GrafikÖğesi::şekil(SahneŞekli::Çizgi {
                    başlangıç: a,
                    bitiş: b,
                })
                .stil(stil.clone())
            };
            Some(GrafikÖğesi::grup([
                çizgi(
                    (yüksek_nokta[0] - yarım, yüksek_nokta[1]),
                    (yüksek_nokta[0] + yarım, yüksek_nokta[1]),
                ),
                çizgi(
                    (yüksek_nokta[0], yüksek_nokta[1]),
                    (düşük_nokta[0], düşük_nokta[1]),
                ),
                çizgi(
                    (düşük_nokta[0] - yarım, düşük_nokta[1]),
                    (düşük_nokta[0] + yarım, düşük_nokta[1]),
                ),
            ]))
        });
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Error bar chart").üst(25))
        .gösterge(Gösterge::yeni().veri(["bar", "error"]).üst(30).sağ(30))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().aralık(50.0, 70.0))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(50.0, 70.0))
        .x_ekseni(Eksen::kategori().veri(kategoriler))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("bar")
                .öğe_stili(ÖğeStili::yeni().renk("#77bef7"))
                .veri(sütun),
        )
        .seri(özel)
}

fn custom_profile() -> GrafikSeçenekleri {
    const BAŞLANGIÇ: f64 = 1_704_067_200_000.0;
    let kategoriler = ["categoryA", "categoryB", "categoryC"];
    let türler = [
        ("JS Heap", "#7b9ce1"),
        ("Documents", "#bd6d6c"),
        ("Nodes", "#75d874"),
        ("Listeners", "#e0bc78"),
        ("GPU Memory", "#dc77dc"),
        ("GPU", "#72b362"),
    ];
    let mut tohum = 0x5eed_1234;
    let mut veri = Vec::new();
    for kategori in 0..kategoriler.len() {
        let mut taban = BAŞLANGIÇ;
        for _ in 0..10 {
            let tür_sırası = (rastgele(&mut tohum) * (türler.len() - 1) as f64).round() as usize;
            let süre = (rastgele(&mut tohum) * 10_000.0).round();
            let başlangıç = taban;
            taban += süre;
            veri.push(
                VeriÖğesi::adlı(
                    türler[tür_sırası].0,
                    vec![kategori as f64, başlangıç, taban, süre],
                )
                .stil(ÖğeStili::yeni().renk(türler[tür_sırası].1).opaklık(0.8)),
            );
            taban += (rastgele(&mut tohum) * 2000.0).round();
        }
    }
    let özel = ÖzelSeri::yeni()
        .veri(veri)
        .kodla("x", [1, 2])
        .kodla("y", [0])
        .kırp(true)
        .öğe_çizimi(|api| {
            let kategori = api.sayısal_değer(0usize)?;
            let başlangıç = api.sayısal_değer(1usize)?;
            let bitiş = api.sayısal_değer(2usize)?;
            let baş = api.koordinat(&[başlangıç, kategori])?;
            let son = api.koordinat(&[bitiş, kategori])?;
            let yükseklik = api.boyut(&[0.0, 1.0], None)[1] * 0.6;
            Some(
                GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(
                    baş[0],
                    baş[1] - yükseklik / 2.0,
                    son[0] - baş[0],
                    yükseklik,
                ))
                .stil(api.stil()),
            )
        });
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Profile").sol("center").üst(25))
        .ipucu(İpucu::yeni())
        .ızgara(Izgara::yeni().yükseklik(300))
        .veri_yakınlaştırma(
            VeriYakınlaştırma::sürgü()
                .süzme_kipi(YakınlaştırmaSüzmeKipi::ZayıfSüz)
                .veri_gölgesi(false)
                .üst(400),
        )
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().süzme_kipi(YakınlaştırmaSüzmeKipi::ZayıfSüz))
        .x_ekseni(
            Eksen::değer()
                .en_az(BAŞLANGIÇ)
                .ölçekli(true)
                .etiket_biçimleyici(Biçimleyici::İşlev(Arc::new(|değer, _| {
                    format!("{} ms", (değer - BAŞLANGIÇ).max(0.0).round())
                }))),
        )
        .y_ekseni(Eksen::kategori().veri(kategoriler))
        .seri(özel)
}

fn cycle_plot() -> GrafikSeçenekleri {
    const HAM: [[f64; 13]; 11] = [
        [
            2002., 14., 21., 25., 21., 26., 32., 27., 20., 10., 11., 5., 5.,
        ],
        [
            2003., 18., 24., 28., 24., 33., 37., 30., 25., 13., 14., 6., 6.,
        ],
        [
            2004., 22., 31., 36., 28., 37., 43., 35., 30., 13., 13., 7., 7.,
        ],
        [
            2005., 25., 32., 38., 34., 39., 48., 38., 29., 14., 14., 8., 8.,
        ],
        [
            2006., 29., 38., 47., 33., 44., 57., 41., 39., 16., 16., 9., 8.,
        ],
        [
            2007., 29., 35., 49., 34., 43., 57., 41., 37., 20., 17., 9., 10.,
        ],
        [
            2008., 22., 32., 37., 30., 35., 44., 38., 31., 16., 17., 8., 7.,
        ],
        [
            2009., 25., 34., 41., 33., 39., 47., 44., 32., 17., 17., 9., 8.,
        ],
        [
            2010., 26., 35., 46., 40., 47., 61., 47., 41., 20., 18., 9., 10.,
        ],
        [
            2011., 29., 39., 55., 38., 55., 67., 53., 41., 19., 20., 11., 11.,
        ],
        [
            2012., 38., 48., 60., 49., 57., 79., 62., 54., 26., 26., 13., 11.,
        ],
    ];
    let aylar = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let aya_göre = (0..12)
        .map(|ay| {
            let mut satır = vec![ay as f64];
            satır.extend(HAM.iter().map(|yıl| yıl[ay + 1]));
            satır
        })
        .collect::<Vec<_>>();
    let ortalama = aya_göre
        .iter()
        .enumerate()
        .map(|(ay, satır)| [ay as f64, satır[1..].iter().sum::<f64>() / 11.0])
        .collect::<Vec<_>>();
    let ortalama_serisi = ÖzelSeri::yeni()
        .ad("Average")
        .veri(ortalama)
        .kodla("x", [0])
        .kodla("y", [1])
        .öğe_çizimi(|api| {
            let x = api.sayısal_değer(0usize)?;
            let y = api.sayısal_değer(1usize)?;
            let bant = api.boyut(&[0.0, 0.0], None)[0] * 0.85;
            let p = api.koordinat(&[x, y])?;
            Some(
                GrafikÖğesi::şekil(SahneŞekli::Çizgi {
                    başlangıç: (p[0] - bant / 2.0, p[1]),
                    bitiş: (p[0] + bant / 2.0, p[1]),
                })
                .stil(çizgi_stili(düz_renk(&api.renk), 2.0)),
            )
        });
    let eğilim_serisi = ÖzelSeri::yeni()
        .ad("Trend by year (2002 - 2012)")
        .veri(aya_göre)
        .kodla("x", [0])
        .kodla("y", 1..=11)
        .öğe_çizimi(|api| {
            let ay = api.sayısal_değer(0usize)?;
            let birim = api.boyut(&[0.0, 0.0], None)[0] * 0.85 / 10.0;
            let mut noktalar = Vec::new();
            for yıl in 0..11 {
                let değer = api.sayısal_değer(yıl + 1)?;
                let p = api.koordinat(&[ay, değer])?;
                noktalar.push((p[0] + birim * (yıl as f32 - 5.5), p[1]));
            }
            Some(
                GrafikÖğesi::şekil(SahneŞekli::ÇokluÇizgi(noktalar))
                    .stil(çizgi_stili(düz_renk(&api.renk), 2.0)),
            )
        });
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Sales Trends by Year within Each Month")
                .alt_metin("Sample of Cycle Plot")
                .sol("center")
                .üst(25),
        )
        .gösterge(
            Gösterge::yeni()
                .veri(["Trend by year (2002 - 2012)", "Average"])
                .üst(80),
        )
        .ipucu(İpucu::yeni())
        .ızgara(Izgara::yeni().üst(120).alt(70))
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü())
        .veri_yakınlaştırma(VeriYakınlaştırma::iç())
        .x_ekseni(Eksen::kategori().veri(aylar))
        .y_ekseni(Eksen::değer().sayısal_kenar_boşluğu(0.0, "20%"))
        .seri(ortalama_serisi)
        .seri(eğilim_serisi)
}

fn custom_polar_heatmap() -> GrafikSeçenekleri {
    let saatler = [
        "12a", "1a", "2a", "3a", "4a", "5a", "6a", "7a", "8a", "9a", "10a", "11a", "12p", "1p",
        "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "10p", "11p",
    ];
    let günler = [
        "Saturday",
        "Friday",
        "Thursday",
        "Wednesday",
        "Tuesday",
        "Monday",
        "Sunday",
    ];
    let veri = resmi_sayısal_dizi("custom-polar-heatmap", "data");
    let en_çok = veri
        .iter()
        .filter_map(|satır| satır.get(2))
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    let özel = ÖzelSeri::yeni()
        .ad("Punch Card")
        .koordinat_sistemi(ÖzelSeriKoordinatSistemi::Kutupsal)
        .kutupsal_sırası(0)
        .veri(veri)
        .kodla("radius", [0])
        .kodla("angle", [1])
        .öğe_çizimi(|api| {
            let gün = api.sayısal_değer(0usize)?;
            let saat = api.sayısal_değer(1usize)?;
            let koordinat = api.koordinat(&[gün, saat])?;
            let boyut = api.boyut(&[1.0, 1.0], Some(&[gün, saat]));
            let ÖzelKoordinatTanımı::Kutupsal {
                merkez_x, merkez_y, ..
            } = api.koordinat_tanımı
            else {
                return None;
            };
            Some(
                GrafikÖğesi::şekil(SahneŞekli::Dilim {
                    merkez: (merkez_x, merkez_y),
                    iç_yarıçap: (koordinat[2] - boyut[0] / 2.0).max(0.0),
                    dış_yarıçap: koordinat[2] + boyut[0] / 2.0,
                    başlangıç_açısı: -(koordinat[3] + boyut[1] / 2.0),
                    bitiş_açısı: -(koordinat[3] - boyut[1] / 2.0),
                })
                .stil(api.stil()),
            )
        });

    let mut açı = Eksen::kategori()
        .veri(saatler)
        .kenar_boşluğu(false)
        .bölme_çizgisi_göster(true)
        .çizgi(EksenÇizgisi::yeni().göster(false));
    açı.bölme_çizgisi.renk = Some("#ddd".into());
    açı.bölme_çizgisi.tür = ÇizgiTürü::Kesikli;

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(Gösterge::yeni().veri(["Punch Card"]).alt(25))
        .kutupsal(
            KutupsalKoordinat::yeni()
                .açısal_eksen(açı)
                .radyal_eksen(Eksen::kategori().veri(günler).z(100)),
        )
        .ipucu(İpucu::yeni())
        .görsel_eşleme(
            GörselEşleme::yeni()
                .en_az(0.0)
                .en_çok(en_çok)
                .boyut(2usize)
                .üst("middle")
                .hesaplanabilir(true),
        )
        .seri(özel)
}

fn flame_graph() -> GrafikSeçenekleri {
    fn renk(ad: &str) -> Renk {
        match ad.split_whitespace().next().unwrap_or_default() {
            "root" => "#8fd3e8",
            "genunix" => "#d95850",
            "unix" => "#eb8146",
            "ufs" => "#ffb248",
            "FSS" => "#f2d643",
            "namefs" => "#ebdba4",
            "doorfs" => "#fcce10",
            "lofs" => "#b5c334",
            "zfs" => "#1bca93",
            _ => "#5470c6",
        }
        .into()
    }

    fn dolaş(
        düğüm: &serde_json::Value,
        başlangıç: f64,
        seviye: usize,
        kök: f64,
        en_derin: &mut usize,
        çıktı: &mut Vec<VeriÖğesi>,
    ) {
        let değer = düğüm["value"].as_f64().unwrap_or_default();
        let ad = düğüm["name"].as_str().unwrap_or_default();
        let kimlik = düğüm["id"].as_str().unwrap_or(ad);
        *en_derin = (*en_derin).max(seviye);
        çıktı.push(
            VeriÖğesi::adlı(
                kimlik,
                VeriDeğeri::KarmaDizi(vec![
                    (seviye as f64).into(),
                    başlangıç.into(),
                    (başlangıç + değer).into(),
                    ad.into(),
                    (değer / kök * 100.0).into(),
                ]),
            )
            .stil(ÖğeStili::yeni().renk(renk(ad))),
        );
        let mut çocuk_başı = başlangıç;
        if let Some(çocuklar) = düğüm["children"].as_array() {
            for çocuk in çocuklar {
                dolaş(çocuk, çocuk_başı, seviye + 1, kök, en_derin, çıktı);
                çocuk_başı += çocuk["value"].as_f64().unwrap_or_default();
            }
        }
    }

    let yol = std::env::current_dir()
        .expect("çalışma dizini")
        .join("../echarts-examples/public/data/asset/data/stack-trace.json");
    let kök_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&yol)
            .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", yol.display())),
    )
    .expect("stack-trace JSON");
    let kök_değeri = kök_json["value"].as_f64().unwrap_or(1.0);
    let mut veri = Vec::new();
    let mut en_derin = 0;
    dolaş(&kök_json, 0.0, 0, kök_değeri, &mut en_derin, &mut veri);

    let özel = ÖzelSeri::yeni()
        .veri(veri)
        .kodla("x", [0, 1, 2])
        .kodla("y", [0])
        .öğe_çizimi(|api| {
            let seviye = api.sayısal_değer(0usize)?;
            let başlangıç = api.koordinat(&[api.sayısal_değer(1usize)?, seviye])?;
            let bitiş = api.koordinat(&[api.sayısal_değer(2usize)?, seviye])?;
            let yükseklik = api.boyut(&[0.0, 1.0], None)[1];
            let genişlik = bitiş[0] - başlangıç[0];
            let kutu = Dikdörtgen::yeni(
                başlangıç[0],
                başlangıç[1] - yükseklik / 2.0,
                genişlik,
                (yükseklik - 2.0).max(0.0),
            );
            let mut çocuklar = vec![
                GrafikÖğesi::dikdörtgen(kutu)
                    .köşe_yarıçapı(2.0)
                    .stil(api.stil()),
            ];
            let ad = api.sıralı_ham_değer(3usize).unwrap_or_default();
            let kullanılabilir = (genişlik - 4.0).max(0.0);
            if kullanılabilir > 0.0 {
                çocuklar.push(doğrudan_metin_öğesi(
                    ad,
                    // zrender `textConfig.position: 'insideLeft'` için
                    // öntanımlı `textDistance` 5 pikseldir.
                    (kutu.x + 5.0, kutu.y + kutu.yükseklik / 2.0),
                    YatayHiza::Sol,
                    DikeyHiza::Orta,
                    12.0,
                    "#000",
                ));
                if let Some(GrafikÖğesi {
                    içerik: GrafikÖğeİçeriği::Metin(metin),
                    ..
                }) = çocuklar.last_mut()
                {
                    metin.aile = Some("Verdana".to_owned());
                    metin.en_çok_genişlik = Some(kullanılabilir);
                    metin.üç_nokta = "..".to_owned();
                    metin.en_az_karakter = 1;
                }
            }
            Some(GrafikÖğesi::grup(çocuklar))
        });

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .arkaplan(Dolgu::doğrusal(
            0.0,
            0.0,
            0.0,
            1.0,
            vec![
                RenkDurağı::yeni(0.05, "#eee"),
                RenkDurağı::yeni(0.95, "#eeeeb0"),
            ],
        ))
        .başlık(
            Başlık::yeni()
                .metin("Flame Graph")
                .sol("center")
                .üst(20)
                .yazı(YazıStili::yeni().aile("Verdana").kalın(false).boyut(20.0)),
        )
        .araç_kutusu(AraçKutusu::yeni().geri_yükle(true).sağ(20).üst(20))
        .ipucu(İpucu::yeni())
        .x_ekseni(Eksen::değer().göster(false).en_az(0.0))
        .y_ekseni(
            Eksen::değer()
                .göster(false)
                .en_az(0.0)
                .en_çok(en_derin as f64),
        )
        .seri(özel)
}

fn custom_gauge() -> GrafikSeçenekleri {
    const DIŞ_YARIÇAP: f32 = 200.0;
    const İÇ_YARIÇAP: f32 = 170.0;
    const İBRE_İÇ_YARIÇAPI: f32 = 40.0;
    const PANEL_İÇ_YARIÇAPI: f32 = 140.0;

    let kaynak = std::env::current_dir()
        .expect("çalışma dizini")
        .join("../echarts-examples/public/data/asset/img/custom-gauge-panel.png");
    let özel = ÖzelSeri::yeni()
        .koordinat_sistemi(ÖzelSeriKoordinatSistemi::Kutupsal)
        .kutupsal_sırası(0)
        .veri([[1.0, 156.0]])
        .kodla("radius", [0])
        .kodla("angle", [1])
        .öğe_çizimi(move |api| {
            let değer = api.sayısal_değer(1usize)? as f32;
            let koordinat = api.koordinat(&[api.sayısal_değer(0usize)?, f64::from(değer)])?;
            let ÖzelKoordinatTanımı::Kutupsal {
                merkez_x, merkez_y, ..
            } = api.koordinat_tanımı
            else {
                return None;
            };
            let panel_kutusu = Dikdörtgen::yeni(
                merkez_x - DIŞ_YARIÇAP,
                merkez_y - DIŞ_YARIÇAP,
                DIŞ_YARIÇAP * 2.0,
                DIŞ_YARIÇAP * 2.0,
            );
            let panel = png_sahne_resmi(&kaynak, panel_kutusu);
            let uç_açı = koordinat[3];
            let halka =
                GrafikÖğesi::resim(panel.clone()).kırp(KırpmaYolu::yeni(SahneŞekli::Dilim {
                    merkez: (merkez_x, merkez_y),
                    iç_yarıçap: İÇ_YARIÇAP,
                    dış_yarıçap: DIŞ_YARIÇAP,
                    başlangıç_açısı: 0.0,
                    bitiş_açısı: -uç_açı,
                }));
            let kutupsal_nokta = |yarıçap: f32, açı: f32| {
                (
                    açı.cos() * yarıçap + merkez_x,
                    -açı.sin() * yarıçap + merkez_y,
                )
            };
            let ibre_noktaları = vec![
                kutupsal_nokta(DIŞ_YARIÇAP, uç_açı),
                kutupsal_nokta(DIŞ_YARIÇAP, uç_açı + std::f32::consts::PI * 0.03),
                kutupsal_nokta(İBRE_İÇ_YARIÇAPI, uç_açı),
            ];
            let ibre = GrafikÖğesi::resim(panel)
                .kırp(KırpmaYolu::yeni(SahneŞekli::Çokgen(ibre_noktaları)));
            let iç_panel = GrafikÖğesi::şekil(SahneŞekli::Daire {
                merkez: (merkez_x, merkez_y),
                yarıçap: PANEL_İÇ_YARIÇAPI,
            })
            .stil(SahneStili {
                dolgu: Some(Dolgu::Düz(Renk::BEYAZ)),
                gölge_rengi: Some("rgba(76,107,167,0.4)".into()),
                gölge_bulanıklığı: 25.0,
                ..SahneStili::default()
            });
            let metin = doğrudan_metin_öğesi(
                format!("{:.0}%", değer / 200.0 * 100.0),
                (merkez_x, merkez_y),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                50.0,
                "rgb(0,50,190)",
            )
            .kalın(true);
            Some(GrafikÖğesi::grup([halka, ibre, iç_panel, metin]))
        });

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni())
        .kutupsal(
            KutupsalKoordinat::yeni()
                .başlangıç_açısı(0.0)
                .açısal_eksen(Eksen::değer().göster(false).en_az(0.0).en_çok(200.0))
                .radyal_eksen(Eksen::değer().göster(false)),
        )
        .seri(özel)
}

fn pie_parliament_transition(durum: &str) -> GrafikSeçenekleri {
    const RENKLER: [&str; 6] = [
        "#5070dd", "#b6d634", "#505372", "#ff994d", "#0ca8df", "#ffd10a",
    ];
    let veri = [
        (800.0, "A"),
        (635.0, "B"),
        (580.0, "C"),
        (484.0, "D"),
        (300.0, "E"),
        (200.0, "F"),
    ];
    if durum != "parliament" {
        return GrafikSeçenekleri::yeni().animasyon(false).seri(
            PastaSerisi::yeni()
                .halka("30%", "80%")
                .etiket(Etiket::yeni().göster(false))
                .veri(veri.iter().enumerate().map(|(sıra, (değer, ad))| {
                    VeriÖğesi::adlı(*ad, *değer).stil(ÖğeStili::yeni().renk(RENKLER[sıra]))
                })),
        );
    }

    let toplam = veri.iter().map(|(değer, _)| *değer).sum::<f64>();
    let mut açılar = Vec::with_capacity(veri.len() + 1);
    let mut açı = -std::f64::consts::FRAC_PI_2;
    for (değer, _) in veri {
        açılar.push(açı);
        açı += değer / toplam * std::f64::consts::TAU;
    }
    açılar.push(-std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU);
    let açılar = Arc::new(açılar);
    let özel = ÖzelSeri::yeni()
        .kimlik("distribution")
        .koordinat_sistemi(ÖzelSeriKoordinatSistemi::Yok)
        .veri(veri.iter().map(|(değer, ad)| VeriÖğesi::adlı(*ad, *değer)))
        .öğe_çizimi(move |api| {
            let sıra = api.veri_sırası;
            let görünüm = api.görünüm_genişliği.min(api.görünüm_yüksekliği);
            let iç_yarıçap = 0.30 * görünüm / 2.0;
            let dış_yarıçap = 0.80 * görünüm / 2.0;
            let merkez = (api.görünüm_genişliği * 0.5, api.görünüm_yüksekliği * 0.5);
            let nokta_boyutu = görünüm / 50.0;
            let adım = nokta_boyutu + 3.0;
            let satır_sayısı = ((dış_yarıçap - iç_yarıçap) / adım).ceil() as usize;
            let başlangıç = *açılar.get(sıra)?;
            let bitiş = *açılar.get(sıra + 1)?;
            let mut çocuklar = Vec::new();
            let mut yarıçap = f64::from(iç_yarıçap);
            for _ in 0..satır_sayısı {
                let koltuk_sayısı = (std::f64::consts::TAU * yarıçap / f64::from(adım))
                    .round()
                    .max(1.0);
                let yeni_boyut = std::f64::consts::TAU * yarıçap / koltuk_sayısı;
                let mut k = (başlangıç * yarıçap / yeni_boyut).floor() * yeni_boyut;
                let son = (bitiş * yarıçap / yeni_boyut).floor() * yeni_boyut - 1e-6;
                while k < son {
                    let kutupsal_açı = k / yarıçap;
                    çocuklar.push(
                        GrafikÖğesi::şekil(SahneŞekli::Daire {
                            merkez: (
                                merkez.0 + (kutupsal_açı.cos() * yarıçap) as f32,
                                merkez.1 + (kutupsal_açı.sin() * yarıçap) as f32,
                            ),
                            yarıçap: nokta_boyutu / 2.0,
                        })
                        .stil(SahneStili {
                            dolgu: Some(Dolgu::Düz(RENKLER[sıra].into())),
                            ..SahneStili::default()
                        }),
                    );
                    k += yeni_boyut;
                }
                yarıçap += f64::from(adım);
            }
            Some(GrafikÖğesi::grup(çocuklar))
        });
    GrafikSeçenekleri::yeni().animasyon(false).seri(özel)
}

fn wind_barb() -> GrafikSeçenekleri {
    let yol = std::env::current_dir()
        .expect("çalışma dizini")
        .join("../echarts-examples/public/data/asset/data/wind-barb-hobart.json");
    let ham: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&yol)
            .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", yol.display())),
    )
    .expect("wind-barb JSON");
    let mut özel_veri = Vec::new();
    let mut dalga_verisi = Vec::new();
    let mut hız_verisi = Vec::new();
    for kayıt in ham["data"].as_array().into_iter().flatten() {
        let zaman = iso_utc_milisaniye(kayıt["time"].as_str().unwrap_or_default());
        let hız = kayıt["windSpeed"].as_f64().unwrap_or_default();
        let yön = kayıt["R"].as_str().unwrap_or_default();
        let dalga = kayıt["waveHeight"].as_f64().unwrap_or_default();
        özel_veri.push(VeriÖğesi::from(VeriDeğeri::KarmaDizi(vec![
            zaman.into(),
            hız.into(),
            yön.into(),
            dalga.into(),
        ])));
        dalga_verisi.push([zaman, dalga]);
        hız_verisi.push([zaman, hız]);
    }
    let mut hava_verisi = Vec::new();
    for kayıt in ham["forecast"].as_array().into_iter().flatten() {
        hava_verisi.push(VeriÖğesi::from(VeriDeğeri::KarmaDizi(vec![
            // ECharts `parseDate('yyyy-MM-dd')` Chrome'un yerel gece
            // yarısını kullanır. Kilitli tarayıcı Europe/Istanbul (+03:00).
            (iso_utc_milisaniye(kayıt["localDate"].as_str().unwrap_or_default())
                - 3.0 * 3_600_000.0)
                .into(),
            0.0.into(),
            kayıt["skyIcon"].as_str().unwrap_or_default().into(),
            kayıt["minTemp"].as_f64().unwrap_or_default().into(),
            kayıt["maxTemp"].as_f64().unwrap_or_default().into(),
        ])));
    }

    let ok = ÖzelSeri::yeni()
        .veri(özel_veri.clone())
        .kodla("x", [0])
        .kodla("y", [1])
        .z(10)
        .öğe_çizimi(|api| {
            let nokta = api.koordinat(&[api.sayısal_değer(0usize)?, api.sayısal_değer(1usize)?])?;
            let açı = match api.sıralı_ham_değer(2usize)?.as_str() {
                "W" => 0,
                "WSW" => 1,
                "SW" => 2,
                "SSW" => 3,
                "S" => 4,
                "SSE" => 5,
                "SE" => 6,
                "ESE" => 7,
                "E" => 8,
                "ENE" => 9,
                "NE" => 10,
                "NNE" => 11,
                "N" => 12,
                "NNW" => 13,
                "NW" => 14,
                "WNW" => 15,
                _ => 0,
            } as f32
                * std::f32::consts::PI
                / 8.0;
            let kaynak = [
                (-10.0, 10.0),
                (16.0, 10.0),
                (16.0, 1.0),
                (31.0, 16.0),
                (16.0, 31.0),
                (16.0, 22.0),
                (-10.0, 22.0),
            ];
            let noktalar = kaynak
                .into_iter()
                .map(|(x, y)| {
                    (
                        (x + 10.0) / 41.0 * 18.0 - 9.0,
                        // zrender `makePath(..., rect, 'center')` en-boy
                        // oranını korur. 41×30 kaynak yol 18×18 kutuya
                        // genişliğe göre sığar; dikeyde gerilmez.
                        (y - 1.0) / 41.0 * 18.0 - (30.0 / 41.0 * 18.0) / 2.0,
                    )
                })
                .collect();
            let mut stil = api.stil();
            stil.çizgi_rengi = Some("#555".into());
            stil.çizgi_kalınlığı = 1.0;
            Some(
                GrafikÖğesi::şekil(SahneŞekli::Çokgen(noktalar))
                    .dönüşüm(
                        YerelDönüşüm::default()
                            .ötele(nokta[0], nokta[1])
                            .döndür(açı),
                    )
                    .stil(stil),
            )
        });

    let resim_dizini = std::env::current_dir()
        .expect("çalışma dizini")
        .join("../echarts-examples/public/data/asset/img/weather");
    let hava = ÖzelSeri::yeni()
        .veri(hava_verisi)
        .eksenler(0, 2)
        .kodla("x", [0])
        .z(11)
        .öğe_çizimi(move |api| {
            let zaman = api.sayısal_değer(0usize)?;
            let nokta = api.koordinat(&[zaman + 86_400_000.0 / 2.0, 0.0])?;
            let simge = api.sıralı_ham_değer(2usize)?;
            let dosya = match simge.as_str() {
                "Showers" => "showers_128.png",
                "Sunny" => "sunny_128.png",
                _ => "cloudy_128.png",
            };
            let resim = png_sahne_resmi(
                resim_dizini.join(dosya),
                Dikdörtgen::yeni(nokta[0] - 22.5, 87.5, 45.0, 45.0),
            );
            let metin = doğrudan_metin_öğesi(
                format!(
                    "{} - {}°",
                    api.sıralı_ham_değer(3usize)?,
                    api.sıralı_ham_değer(4usize)?
                ),
                (nokta[0], 80.0),
                YatayHiza::Orta,
                DikeyHiza::Alt,
                14.0,
                "#000",
            );
            Some(GrafikÖğesi::grup([GrafikÖğesi::resim(resim), metin]))
        });

    let dalga = ÇizgiSerisi::yeni()
        .eksenler(0, 1)
        .z(2)
        .sembol_göster(false)
        .çizgi_stili(ÇizgiStili::yeni().renk("rgba(88,160,253,1)"))
        .öğe_stili(ÖğeStili::yeni().renk("rgba(88,160,253,1)"))
        // ECharts LineSeries `areaStyle` öntanımlı opaklığı 0.7'dir ve
        // gradient durağının kendi alfa değeriyle çarpılır.
        .alan_stili(AlanStili::yeni().opaklık(0.7).renk(Dolgu::doğrusal(
            0.0,
            0.0,
            0.0,
            1.0,
            vec![
                RenkDurağı::yeni(0.0, "rgba(88,160,253,1)"),
                RenkDurağı::yeni(0.5, "rgba(88,160,253,0.7)"),
                RenkDurağı::yeni(1.0, "rgba(88,160,253,0)"),
            ],
        )))
        .veri(dalga_verisi);
    let hız = ÇizgiSerisi::yeni()
        .z(1)
        .sembol(Sembol::Yok)
        .çizgi_stili(ÇizgiStili::yeni().renk("#aaa").tür(ÇizgiTürü::Noktalı))
        .veri(hız_verisi);

    let mut y0 = Eksen::değer()
        .ad("风速（节）")
        .ad_konumu(EksenAdKonumu::Orta)
        .ad_boşluğu(35.0);
    y0.çizgi.renk = Some("#666".into());
    y0.bölme_çizgisi.renk = Some("#ddd".into());
    let mut y1 = Eksen::değer()
        .ad("浪高（米）")
        .ad_konumu(EksenAdKonumu::Orta)
        .ad_boşluğu(35.0)
        .en_çok(6.0);
    y1.çizgi.renk = Some("#015DD5".into());
    y1.bölme_çizgisi.göster = Some(false);
    let mut x = Eksen::zaman()
        .en_büyük_adım(86_400_000.0)
        .zaman_dilimi_dakikası(180)
        .etiket_biçimleyici(Biçimleyici::İşlev(Arc::new(|_, metin| {
            // Kilitli resmî galeri referansı İngilizce locale ile alınır.
            if metin == "Tem" {
                "Jul".to_owned()
            } else {
                metin.to_owned()
            }
        })));
    x.bölme_çizgisi.renk = Some("#ddd".into());

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("天气 风向 风速 海浪 预报")
                .alt_metin("示例数据源于 www.seabreeze.com.au")
                .sol("center")
                .üst(25),
        )
        .ipucu(İpucu::yeni())
        .ızgara(Izgara::yeni().üst(160).alt(125))
        .x_ekseni(x)
        // ECharts `yAxis: [...]`: tekil `y_ekseni` kurucusu son çağrıda
        // önceki ekseni değiştirir; dizi semantiği için ekleme API'si
        // kullanılmalıdır. Üçüncü eksene sonlu bir kapsam vermek ayrıca
        // yalnız x konumunu kullanan hava `renderItem` koordinatını tanımlı
        // tutar.
        .y_ekseni_ekle(y0)
        .y_ekseni_ekle(y1)
        .y_ekseni_ekle(Eksen::değer().göster(false).en_az(0.0).en_çok(1.0))
        .görsel_eşleme(
            GörselEşleme::yeni()
                .yön(Yön::Yatay)
                .sol("center")
                .alt(10)
                .seri_sırası(1)
                .boyut(1usize)
                // ECharts parçaları seçenek dizisinde büyükten küçüğe
                // tanımlansa da yatay göstergede küçükten büyüğe dizer.
                .parçalar([
                    EşlemeParçası::lt(11.0, "#D33C3E").etiket("微风（小于 11 节）"),
                    EşlemeParçası::yeni(Some(11.0), Some(17.0), "#f4e9a3")
                        .etiket("中风（11  ~ 17 节）"),
                    EşlemeParçası::gte(17.0, "#18BF12").etiket("大风（>=17节）"),
                ]),
        )
        .veri_yakınlaştırma(VeriYakınlaştırma::iç())
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().alt(50))
        .seri(dalga)
        .seri(ok)
        .seri(hız)
        .seri(hava)
}

fn custom_gantt_flight() -> GrafikSeçenekleri {
    fn json_değeri(değer: &serde_json::Value) -> VeriDeğeri {
        match değer {
            serde_json::Value::Number(sayı) => sayı.as_f64().unwrap_or_default().into(),
            serde_json::Value::String(metin) => metin.as_str().into(),
            serde_json::Value::Bool(mantıksal) => (*mantıksal).into(),
            _ => VeriDeğeri::Boş,
        }
    }

    fn dikdörtgeni_kırp(d: Dikdörtgen, alan: Dikdörtgen) -> Option<Dikdörtgen> {
        let x = d.x.max(alan.x);
        let y = d.y.max(alan.y);
        let sağ = d.sağ().min(alan.sağ());
        let alt = d.alt().min(alan.alt());
        (sağ > x && alt > y).then(|| Dikdörtgen::yeni(x, y, sağ - x, alt - y))
    }

    let yol = std::env::current_dir()
        .expect("çalışma dizini")
        .join("../echarts-examples/public/data/asset/data/airport-schedule.json");
    let ham: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&yol)
            .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", yol.display())),
    )
    .expect("airport-schedule JSON");
    let uçuşlar = ham["flight"]["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|satır| satır.as_array())
        .map(|satır| {
            VeriÖğesi::from(VeriDeğeri::KarmaDizi(
                satır.iter().map(json_değeri).collect(),
            ))
        })
        .collect::<Vec<_>>();
    let apronlar = ham["parkingApron"]["data"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(sıra, satır)| satır.as_array().map(|satır| (sıra, satır)))
        .map(|(sıra, satır)| {
            let mut değerler = vec![(sıra as f64).into()];
            değerler.extend(satır.iter().map(json_değeri));
            VeriÖğesi::from(VeriDeğeri::KarmaDizi(değerler))
        })
        .collect::<Vec<_>>();
    let apron_sayısı = apronlar.len();

    let uçuş = ÖzelSeri::yeni()
        .kimlik("flightData")
        .veri(uçuşlar)
        .kodla("x", [1, 2])
        .kodla("y", [0])
        .kodla("tooltip", [0, 1, 2])
        .öğe_çizimi(|api| {
            let kategori = api.sayısal_değer(0usize)?;
            let varış = api.koordinat(&[api.sayısal_değer(1usize)?, kategori])?;
            let kalkış = api.koordinat(&[api.sayısal_değer(2usize)?, kategori])?;
            let yükseklik = api.boyut(&[0.0, 1.0], None)[1] * 0.6;
            let uzunluk = kalkış[0] - varış[0];
            let alan = api.koordinat_tanımı.alan();
            let ham_kutu = Dikdörtgen::yeni(varış[0], varış[1] - yükseklik, uzunluk, yükseklik);
            let normal = dikdörtgeni_kırp(ham_kutu, alan);
            let vip = dikdörtgeni_kırp(
                Dikdörtgen::yeni(
                    ham_kutu.x,
                    ham_kutu.y,
                    ham_kutu.genişlik / 2.0,
                    ham_kutu.yükseklik,
                ),
                alan,
            );
            let mut çocuklar = Vec::new();
            if let Some(kutu) = normal {
                çocuklar.push(GrafikÖğesi::dikdörtgen(kutu).stil(api.stil()));
            }
            if api.sıralı_ham_değer(4usize).as_deref() == Some("true")
                && let Some(kutu) = vip
            {
                çocuklar.push(GrafikÖğesi::dikdörtgen(kutu).stil(SahneStili {
                    dolgu: Some(Dolgu::Düz("#ddb30b".into())),
                    ..SahneStili::default()
                }));
            }
            let numara = api.sıralı_ham_değer(3usize).unwrap_or_default();
            let yaklaşık_metin_genişliği = numara.chars().count() as f32 * 7.0;
            if uzunluk > yaklaşık_metin_genişliği + 40.0
                && varış[0] + uzunluk >= 180.0
                && let Some(görünür_kutu) = normal
            {
                çocuklar.push(
                    doğrudan_metin_öğesi(
                        numara,
                        görünür_kutu.merkez(),
                        YatayHiza::Orta,
                        DikeyHiza::Orta,
                        12.0,
                        "#fff",
                    )
                    .z(0, 0.0, 2.0),
                );
            }
            (!çocuklar.is_empty()).then(|| GrafikÖğesi::grup(çocuklar))
        });

    let apron = ÖzelSeri::yeni()
        .veri(apronlar)
        .kodla("x", std::iter::empty::<usize>())
        .kodla("y", [0])
        .öğe_çizimi(|api| {
            let y = api.koordinat(&[0.0, api.sayısal_değer(0usize)?])?[1];
            if y < api.koordinat_tanımı.alan().y + 5.0 {
                return None;
            }
            let mut yol = Yol::yeni();
            let sx = 90.0 / 70.0;
            yol.taşı((0.0, 0.0));
            yol.çiz((0.0, -20.0));
            yol.çiz((30.0 * sx, -20.0));
            yol.kübik((42.0 * sx, -20.0), (38.0 * sx, -1.0), (50.0 * sx, -1.0));
            yol.çiz((90.0, -1.0));
            yol.çiz((90.0, 0.0));
            yol.kapat();
            let şerit = GrafikÖğesi::şekil(SahneŞekli::Yol(yol)).stil(SahneStili {
                dolgu: Some(Dolgu::Düz("#368c6c".into())),
                ..SahneStili::default()
            });
            let ad = doğrudan_metin_öğesi(
                api.sıralı_ham_değer(1usize).unwrap_or_default(),
                (24.0, -3.0),
                YatayHiza::Orta,
                DikeyHiza::Alt,
                12.0,
                "#fff",
            );
            let tür = doğrudan_metin_öğesi(
                api.sıralı_ham_değer(2usize).unwrap_or_default(),
                (75.0, -2.0),
                YatayHiza::Orta,
                DikeyHiza::Alt,
                12.0,
                "#000",
            );
            Some(
                GrafikÖğesi::grup([şerit, ad, tür])
                    .dönüşüm(YerelDönüşüm::default().ötele(10.0, y)),
            )
        });

    let mut x = Eksen::zaman()
        .konum(EksenKonumu::Üst)
        .en_büyük_adım(3_600_000.0)
        .zaman_dilimi_dakikası(180);
    x.çizgi.göster = Some(false);
    x.çentik.renk = Some("#929ABA".into());
    x.etiket.yazı = YazıStili::yeni().renk("#929ABA");
    let mut y = Eksen::değer().en_az(0.0).en_çok(apron_sayısı as f64);
    y.çizgi.göster = Some(false);
    y.çentik.göster = Some(false);
    y.etiket.göster = false;
    y.bölme_çizgisi.göster = Some(false);

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Gantt of Airport Flight")
                .sol("center")
                .üst(25),
        )
        .ipucu(İpucu::yeni())
        .ızgara(Izgara::yeni().üst(70).alt(20).sol(100).sağ(20))
        .x_ekseni(x)
        .y_ekseni(y)
        .veri_yakınlaştırma(
            VeriYakınlaştırma::sürgü()
                .süzme_kipi(YakınlaştırmaSüzmeKipi::ZayıfSüz)
                .yükseklik(20)
                .alt(0)
                .aralık(0.0, 26.0)
                .tutamaç_boyutu("80%"),
        )
        .veri_yakınlaştırma(
            VeriYakınlaştırma::iç()
                .süzme_kipi(YakınlaştırmaSüzmeKipi::ZayıfSüz)
                .aralık(0.0, 26.0),
        )
        .veri_yakınlaştırma(
            VeriYakınlaştırma::sürgü()
                .y_eksen_sırası(0)
                .genişlik(10)
                .sağ(10)
                .üst(70)
                .alt(20)
                .aralık(95.0, 100.0)
                .tutamaç_boyutu(0)
                .veri_gölgesi(false),
        )
        .veri_yakınlaştırma(
            VeriYakınlaştırma::iç()
                .y_eksen_sırası(0)
                .aralık(95.0, 100.0),
        )
        .seri(uçuş)
        .seri(apron)
}

fn circle_packing_with_d3() -> GrafikSeçenekleri {
    let yol = std::env::current_dir()
        .expect("çalışma dizini")
        .join("examples/uyum_veri/circle-packing-layout.json");
    let yerleşim: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&yol)
            .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", yol.display())),
    )
    .expect("circle-packing yerleşim JSON");
    let en_derin = yerleşim["maxDepth"].as_f64().unwrap_or(6.0);
    let veri = yerleşim["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|düğüm| {
            let kimlik = düğüm["id"].as_str().unwrap_or_default();
            VeriÖğesi::adlı(
                kimlik,
                VeriDeğeri::KarmaDizi(vec![
                    kimlik.into(),
                    düğüm["value"].as_f64().unwrap_or_default().into(),
                    düğüm["depth"].as_f64().unwrap_or_default().into(),
                    düğüm["index"].as_f64().unwrap_or_default().into(),
                    düğüm["x"].as_f64().unwrap_or_default().into(),
                    düğüm["y"].as_f64().unwrap_or_default().into(),
                    düğüm["r"].as_f64().unwrap_or_default().into(),
                    düğüm["leaf"].as_bool().unwrap_or(false).into(),
                    düğüm["label"].as_str().unwrap_or_default().into(),
                ]),
            )
        })
        .collect::<Vec<_>>();

    let seri = ÖzelSeri::yeni()
        .koordinat_sistemi(ÖzelSeriKoordinatSistemi::Yok)
        .veri(veri)
        .kodla("tooltip", [1])
        .aşamalı(0, 3_000)
        .öğe_çizimi(|api| {
            let x = api.sayısal_değer(4usize)? as f32;
            let y = api.sayısal_değer(5usize)? as f32;
            let yarıçap = api.sayısal_değer(6usize)? as f32;
            let derinlik = api.sayısal_değer(2usize)? as f32;
            let daire = GrafikÖğesi::şekil(SahneŞekli::Daire {
                merkez: (x, y),
                yarıçap,
            })
            .stil(api.stil());
            let mut çocuklar = vec![daire];
            if api.sıralı_ham_değer(7usize).as_deref() == Some("true") {
                let etiket = api.sıralı_ham_değer(8usize).unwrap_or_default();
                let satırlar = etiket.lines().collect::<Vec<_>>();
                let yazı_boyutu = yarıçap / 3.0;
                // zrender Path.getInsideTextFill: koyu iki halka için açık,
                // daha aydınlık derinlikler için #333. Renk eşiği resmî
                // visualMap gradyanının gerçek luminans sonucuna denktir.
                let yazı_rengi = if derinlik <= 1.0 { "#eee" } else { "#333" };
                for (satır_sırası, satır) in satırlar.iter().enumerate() {
                    let kayma =
                        (satır_sırası as f32 - (satırlar.len() as f32 - 1.0) / 2.0) * yazı_boyutu;
                    let mut metin = SahneMetni::yeni(*satır, (x, y + kayma));
                    metin.yatay = YatayHiza::Orta;
                    metin.dikey = DikeyHiza::Orta;
                    metin.boyut = yazı_boyutu;
                    metin.renk = yazı_rengi.into();
                    metin.aile = Some("Arial".to_owned());
                    metin.en_çok_genişlik = Some(yarıçap * 1.3);
                    çocuklar.push(
                        GrafikÖğesi::metin("")
                            .with_metin(metin)
                            .sessiz(true)
                            .z(0, 0.0, 1.0),
                    );
                }
            }
            Some(GrafikÖğesi::grup(çocuklar).z(0, 0.0, derinlik * 2.0))
        });

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni())
        .görsel_eşleme(
            GörselEşleme::yeni()
                .göster(false)
                .en_az(0.0)
                .en_çok(en_derin)
                .boyut(2usize)
                .renkler(["#006edd", "#e0ffff"]),
        )
        .seri(seri)
}

fn custom_spiral_race(durum: &str) -> GrafikSeçenekleri {
    const RADYAN_ADIMI: f32 = std::f32::consts::PI / 45.0;
    const SÜTUN_GENİŞLİĞİ_DEĞERİ: f32 = 0.4;
    const YARIÇAP_ADIMI_DEĞERİ: f32 = 4.0;
    const BAŞLANGIÇ_RADYANI: f32 = std::f32::consts::FRAC_PI_2;
    const RENKLER: [(&str, &str); 3] = [
        ("#5470c6", "#2747a5"),
        ("#91cc75", "#447f27"),
        ("#fac858", "#a0761c"),
    ];
    const RADYAN_ETİKETLERİ: [&str; 12] = [
        "Aries",
        "Taurus",
        "Gemini",
        "Cancer",
        "Leo",
        "Virgo",
        "Libra",
        "Scorpius",
        "Sagittarius",
        "Capricornus",
        "Aquarius",
        "Pisces",
    ];
    const VERİLER: [[[f64; 2]; 3]; 9] = [
        [[1.0, 3.0], [2.0, 6.0], [3.0, 9.0]],
        [[1.0, 12.0], [2.0, 16.0], [3.0, 14.0]],
        [[1.0, 17.0], [2.0, 22.0], [3.0, 19.0]],
        [[1.0, 19.0], [2.0, 33.0], [3.0, 24.0]],
        [[1.0, 24.0], [2.0, 42.0], [3.0, 29.0]],
        [[1.0, 27.0], [2.0, 47.0], [3.0, 41.0]],
        [[1.0, 36.0], [2.0, 52.0], [3.0, 52.0]],
        [[1.0, 46.0], [2.0, 59.0], [3.0, 63.0]],
        [[1.0, 60.0], [2.0, 63.0], [3.0, 69.0]],
    ];

    fn spiral_yarıçapı(başlangıç: f32, radyan: f32, adım: f32) -> f32 {
        başlangıç + adım * ((BAŞLANGIÇ_RADYANI - radyan) / (std::f32::consts::PI * 2.0))
    }

    fn kutupsal_nokta(merkez: (f32, f32), yarıçap: f32, radyan: f32) -> (f32, f32) {
        (
            radyan.cos() * yarıçap + merkez.0,
            -radyan.sin() * yarıçap + merkez.1,
        )
    }

    fn spiral_noktaları(
        merkez: (f32, f32),
        genişlik: f32,
        başlangıç_yarıçapı: f32,
        bitiş_radyanı: f32,
    ) -> Vec<(f32, f32)> {
        let mut noktalar = Vec::new();
        let yarıçap_adımı = genişlik / SÜTUN_GENİŞLİĞİ_DEĞERİ * YARIÇAP_ADIMI_DEĞERİ;
        let bitiş = bitiş_radyanı - RADYAN_ADIMI;
        let mut radyan = BAŞLANGIÇ_RADYANI;
        while radyan > bitiş {
            if radyan < bitiş_radyanı {
                radyan = bitiş_radyanı;
            }
            let yarıçap = spiral_yarıçapı(başlangıç_yarıçapı - genişlik, radyan, yarıçap_adımı);
            noktalar.push(kutupsal_nokta(merkez, yarıçap, radyan));
            radyan -= RADYAN_ADIMI;
        }
        radyan = bitiş_radyanı;
        while radyan < BAŞLANGIÇ_RADYANI + RADYAN_ADIMI {
            if radyan > BAŞLANGIÇ_RADYANI {
                radyan = BAŞLANGIÇ_RADYANI;
            }
            let yarıçap = spiral_yarıçapı(başlangıç_yarıçapı + genişlik, radyan, yarıçap_adımı);
            noktalar.push(kutupsal_nokta(merkez, yarıçap, radyan));
            radyan += RADYAN_ADIMI;
        }
        noktalar
    }

    fn yazı_öğesi(
        metin: &str,
        konum: (f32, f32),
        boyut: f32,
        renk: &str,
        z2: f32,
    ) -> GrafikÖğesi {
        doğrudan_metin_öğesi(metin, konum, YatayHiza::Orta, DikeyHiza::Orta, boyut, renk)
            .sessiz(true)
            .z(0, 0.0, z2)
    }

    fn konturlu_yazı(
        çocuklar: &mut Vec<GrafikÖğesi>,
        metin: &str,
        konum: (f32, f32),
        boyut: f32,
        renk: &str,
    ) {
        // zrender `stroke:'#fff', lineWidth:3` rasterine yakın dairesel
        // kontur. Metin modeli dolgu ve konturu ayrı sahne öğeleriyle
        // koruduğundan PNG/SVG/GPUI yüzeylerinde aynı sonuç alınır.
        for (dx, dy) in [
            (-1.5, 0.0),
            (1.5, 0.0),
            (0.0, -1.5),
            (0.0, 1.5),
            (-1.1, -1.1),
            (1.1, -1.1),
            (-1.1, 1.1),
            (1.1, 1.1),
        ] {
            çocuklar.push(yazı_öğesi(
                metin,
                (konum.0 + dx, konum.1 + dy),
                boyut,
                "#fff",
                50.0,
            ));
        }
        çocuklar.push(yazı_öğesi(metin, konum, boyut, renk, 51.0));
    }

    let veri_sırası = match durum {
        "son" => 8,
        "güncelleme" => 1,
        _ => 0,
    };
    let veri = VERİLER[veri_sırası];
    let en_çok_yarıçap = veri
        .iter()
        .map(|[başlangıç, bitiş]| başlangıç + YARIÇAP_ADIMI_DEĞERİ as f64 * (bitiş / 12.0))
        .fold(0.0_f64, f64::max)
        .mul_add(1.2, 0.0)
        .ceil();

    let seri = ÖzelSeri::yeni()
        .koordinat_sistemi(ÖzelSeriKoordinatSistemi::Kutupsal)
        .kutupsal_sırası(0)
        .veri(veri)
        .öğe_çizimi(|api| {
            let başlangıç_değeri = api.sayısal_değer(0usize)?;
            let bitiş_değeri = api.sayısal_değer(1usize)?;
            let koordinat = api.koordinat(&[başlangıç_değeri, bitiş_değeri])?;
            let başlangıç_yarıçapı = koordinat[2];
            let bitiş_radyanı = koordinat[3];
            let genişlik = api.koordinat(&[f64::from(SÜTUN_GENİŞLİĞİ_DEĞERİ), 0.0])?[2];
            let ÖzelKoordinatTanımı::Kutupsal {
                merkez_x, merkez_y, ..
            } = api.koordinat_tanımı
            else {
                return None;
            };
            let merkez = (merkez_x, merkez_y);
            let (dolgu, yazı_rengi) = RENKLER[api.veri_sırası.min(RENKLER.len() - 1)];
            let çokgen = GrafikÖğesi::şekil(SahneŞekli::Çokgen(spiral_noktaları(
                merkez,
                genişlik,
                başlangıç_yarıçapı,
                bitiş_radyanı,
            )))
            .stil(SahneStili {
                dolgu: Some(Dolgu::Düz(dolgu.into())),
                ..SahneStili::default()
            });

            let yarıçap_adımı = genişlik / SÜTUN_GENİŞLİĞİ_DEĞERİ * YARIÇAP_ADIMI_DEĞERİ;
            let etiket_yarıçapı =
                spiral_yarıçapı(başlangıç_yarıçapı, bitiş_radyanı, yarıçap_adımı);
            let etiket_konumu = kutupsal_nokta(
                merkez,
                etiket_yarıçapı,
                bitiş_radyanı - 10.0 / etiket_yarıçapı,
            );
            let tur_oranı = (BAŞLANGIÇ_RADYANI - bitiş_radyanı) / (std::f32::consts::PI * 2.0);
            let tur = tur_oranı.floor() as i32;
            let yüzde = tur_oranı.rem_euclid(1.0) * 100.0;
            let yüzde_metni = format!("{yüzde:.1}%");

            let mut çocuklar = vec![çokgen];
            // Rich text ilk satırı: 16 px "Round " + 24 px tur sayısı.
            // Chromium ölçülerindeki iki koşunun ortak merkezi korunur.
            konturlu_yazı(
                &mut çocuklar,
                "Round ",
                (etiket_konumu.0 - 6.674, etiket_konumu.1 - 9.0),
                16.0,
                yazı_rengi,
            );
            konturlu_yazı(
                &mut çocuklar,
                &tur.to_string(),
                (etiket_konumu.0 + 25.797, etiket_konumu.1 - 9.0),
                24.0,
                yazı_rengi,
            );
            konturlu_yazı(
                &mut çocuklar,
                &yüzde_metni,
                (etiket_konumu.0, etiket_konumu.1 + 12.0),
                18.0,
                yazı_rengi,
            );
            Some(GrafikÖğesi::grup(çocuklar))
        });

    let mut açısal = Eksen::değer()
        .en_az(0.0)
        .en_çok(12.0)
        .bölme_sayısı(12)
        .bölme_alanı_göster(true)
        .etiket_biçimleyici(Biçimleyici::İşlev(Arc::new(|değer, _| {
            let sıra = değer.round() as usize;
            RADYAN_ETİKETLERİ
                .get(sıra)
                .copied()
                .unwrap_or("")
                .to_owned()
        })));
    açısal.etiket.yazı = YazıStili::yeni().renk("rgba(0,0,0,0.2)");
    açısal.çizgi.renk = Some("rgba(0,0,0,0.2)".into());

    let mut radyal = Eksen::değer()
        .en_az(0.0)
        .en_çok(en_çok_yarıçap)
        .aralık(1.0)
        .bölme_çizgisi_göster(false)
        .çentik(EksenÇentiği::yeni().göster(false))
        .etiket_biçimleyici(Biçimleyici::İşlev(Arc::new(|değer, _| {
            match değer.round() as i32 {
                1 => "A".to_owned(),
                2 => "B".to_owned(),
                3 => "C".to_owned(),
                _ => String::new(),
            }
        })));
    radyal.etiket.yazı = YazıStili::yeni().renk("rgba(0,0,0,0.6)");
    radyal.çizgi.renk = Some("rgba(0,0,0,0.2)".into());

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni())
        .kutupsal(
            KutupsalKoordinat::yeni()
                .açısal_eksen(açısal)
                .radyal_eksen(radyal),
        )
        .seri(seri)
}

// `examples/*.rs` Cargo tarafından bağımsız örnek hedefi olarak da
// keşfedilir; asıl kullanım `uyum_fixture` içindeki modüldür.
#[allow(dead_code)]
fn main() {}
