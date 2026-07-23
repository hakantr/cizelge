//! Galeri: TÜM çizelge türleri tek pencerede — solda katlanabilir ağaç
//! menü, sağda seçilen çizelge. Üst çubuktaki düğmelerle canlı veri
//! düzenlenir (nokta ekle/çıkar, değerleri karıştır, koyu tema) ve her
//! değişiklik geçiş animasyonuyla uygulanır; kütüphanenin canlı çalıştığı
//! böylece doğrulanır.
//!
//! Çalıştırma: `cargo run --example galeri`

use cizelge::hazir::*;
use gpui::{
    App, Bounds, Context, Entity, IntoElement, MouseButton, MouseDownEvent, Render, SharedString,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_platform::application;
use serde::Deserialize;

// ---------------------------------------------------------------------
// Canlı düzenlenen galeri durumu ve belirlenimci veri üretimi
// ---------------------------------------------------------------------

/// Düğmelerle değişen, tüm üreticilere aktarılan durum.
#[derive(Clone, Copy)]
struct Durum {
    /// Kartezyen serilerde nokta/kategori sayısı.
    nokta: usize,
    /// Değer karıştırma tohumu ("Değerleri Karıştır" artırır).
    tohum: u64,
    /// Koyu tema.
    koyu: bool,
}

/// Belirlenimci sözde rastgele `0..1` (LCG karması; `rand` bağımlılığı
/// almadan tekrarlanabilir canlı veri).
fn karma(tohum: u64, i: usize) -> f64 {
    let h = tohum
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add((i as u64).wrapping_mul(1_442_695_040_888_963_407))
        .wrapping_add(1_013_904_223);
    let h = h ^ (h >> 33);
    (h % 10_000) as f64 / 10_000.0
}

/// Durumdan tek değer: `taban + karma * aralık`.
fn değer(d: &Durum, i: usize, taban: f64, aralık: f64) -> f64 {
    (taban + karma(d.tohum, i) * aralık).round()
}

/// `nokta` uzunluğunda değer listesi (`kanal` seriler arası ayrım için).
fn sayılar(d: &Durum, kanal: usize, taban: f64, aralık: f64) -> Vec<f64> {
    (0..d.nokta)
        .map(|i| değer(d, i.wrapping_add(kanal.wrapping_mul(1000)), taban, aralık))
        .collect()
}

/// `G1..Gn` kategorileri.
fn kategoriler(n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("G{i}")).collect()
}

// ---------------------------------------------------------------------
// Çizelge üreticileri — her biri durumdan beslenir
// ---------------------------------------------------------------------

type Üretici = fn(&Durum) -> GrafikSeçenekleri;

fn temel(başlık: &str) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin(başlık.to_string()))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .araç_kutusu(AraçKutusu::yeni().svg_kaydet(true).png_kaydet(true))
        .ızgara(Izgara::yeni().sol(60.0).sağ(30.0).üst(60.0).alt(45.0))
}

fn çizgi(d: &Durum) -> GrafikSeçenekleri {
    temel("Çizgi")
        .x_ekseni(Eksen::kategori().veri(kategoriler(d.nokta)))
        .y_ekseni(Eksen::değer())
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Ziyaret")
                .veri(sayılar(d, 0, 400.0, 900.0)),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Kayıt")
                .yumuşat(true)
                .veri(sayılar(d, 1, 150.0, 500.0)),
        )
}

fn alan(d: &Durum) -> GrafikSeçenekleri {
    temel("Alan (yumuşak)")
        .x_ekseni(
            Eksen::kategori()
                .veri(kategoriler(d.nokta))
                .kenar_boşluğu(false),
        )
        .y_ekseni(Eksen::değer())
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Akış")
                .yumuşat(true)
                .sembol_göster(false)
                .alan_stili(AlanStili::yeni().renk(Dolgu::doğrusal(
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    vec![
                        RenkDurağı::yeni(0.0, Renk::onaltılık(0x5070dd).alfa_ile(0.45)),
                        RenkDurağı::yeni(1.0, Renk::onaltılık(0x5070dd).alfa_ile(0.02)),
                    ],
                )))
                .veri(sayılar(d, 2, 200.0, 700.0)),
        )
}

fn sütun(d: &Durum) -> GrafikSeçenekleri {
    temel("Sütun")
        .x_ekseni(Eksen::kategori().veri(kategoriler(d.nokta)))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("2025")
                .veri(sayılar(d, 3, 200.0, 500.0))
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı([4.0, 4.0, 0.0, 0.0])),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("2026")
                .veri(sayılar(d, 4, 300.0, 550.0))
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı([4.0, 4.0, 0.0, 0.0])),
        )
}

fn yığın_sütun(d: &Durum) -> GrafikSeçenekleri {
    let mut s = temel("Yığın Sütun")
        .x_ekseni(Eksen::kategori().veri(kategoriler(d.nokta)))
        .y_ekseni(Eksen::değer())
        .gösterge(Gösterge::yeni().üst(30.0));
    for (k, ad) in ["Doğrudan", "Reklam", "Arama"].iter().enumerate() {
        s = s.seri(SütunSerisi::yeni().ad(*ad).yığın("kaynak").veri(sayılar(
            d,
            5 + k,
            80.0,
            300.0,
        )));
    }
    s
}

fn pasta(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Pasta"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(PastaSerisi::yeni().ad("Pay").merkez("50%", "55%").veri([
            ("Chrome", değer(d, 21, 30.0, 40.0)),
            ("Safari", değer(d, 22, 10.0, 20.0)),
            ("Edge", değer(d, 23, 5.0, 12.0)),
            ("Firefox", değer(d, 24, 4.0, 10.0)),
            ("Diğer", değer(d, 25, 2.0, 8.0)),
        ]))
}

fn halka(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Halka Pasta"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst(30.0))
        .seri(
            PastaSerisi::yeni()
                .ad("Erişim")
                .halka("38%", "62%")
                .merkez("50%", "56%")
                .veri([
                    ("Mobil", değer(d, 31, 30.0, 45.0)),
                    ("Masaüstü", değer(d, 32, 20.0, 30.0)),
                    ("Tablet", değer(d, 33, 5.0, 15.0)),
                    ("TV", değer(d, 34, 2.0, 8.0)),
                ]),
        )
}

fn saçılım(d: &Durum) -> GrafikSeçenekleri {
    let veri: Vec<VeriÖğesi> = (0..d.nokta.wrapping_mul(2))
        .map(|i| {
            VeriÖğesi::from([
                değer(d, 40 + i * 2, 150.0, 40.0),
                değer(d, 41 + i * 2, 45.0, 45.0),
            ])
        })
        .collect();
    temel("Saçılım")
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(
            SaçılımSerisi::yeni()
                .ad("Kişiler")
                .sembol_boyutu(12.0)
                .veri(veri),
        )
}

fn efektli_saçılım(d: &Durum) -> GrafikSeçenekleri {
    let uyarı: Vec<VeriÖğesi> = (0..4)
        .map(|i| VeriÖğesi::from([değer(d, 50 + i, 2.0, 10.0), değer(d, 55 + i, 2.0, 7.0)]))
        .collect();
    let normal: Vec<VeriÖğesi> = (0..d.nokta)
        .map(|i| VeriÖğesi::from([değer(d, 60 + i, 1.0, 12.0), değer(d, 80 + i, 1.0, 8.0)]))
        .collect();
    temel("Efektli Saçılım")
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(
            SaçılımSerisi::yeni()
                .ad("Uyarılar")
                .sembol_boyutu(14.0)
                .efektli(true)
                .veri(uyarı),
        )
        .seri(
            SaçılımSerisi::yeni()
                .ad("Normal")
                .sembol_boyutu(9.0)
                .veri(normal),
        )
}

fn mum(d: &Durum) -> GrafikSeçenekleri {
    // Tohumlu rastgele yürüyüş: açılış = önceki kapanış.
    let mut veri: Vec<VeriÖğesi> = Vec::new();
    let mut kapanış = 130.0;
    for i in 0..d.nokta {
        let açılış = kapanış;
        kapanış = (açılış + (karma(d.tohum, 90 + i) - 0.48) * 24.0).max(40.0);
        let düşük = açılış.min(kapanış) - karma(d.tohum, 120 + i) * 8.0;
        let yüksek = açılış.max(kapanış) + karma(d.tohum, 150 + i) * 8.0;
        veri.push(VeriÖğesi::from([açılış, kapanış, düşük, yüksek]));
    }
    temel("Şamdan (Mum)")
        .x_ekseni(Eksen::kategori().veri(kategoriler(d.nokta)))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(
            MumSerisi::yeni()
                .ad("BIST")
                .veri(veri)
                .im_çizgisi(İmÇizgisi::yeni().yatay(İmDeğeri::Ortalama)),
        )
}

fn kutu(d: &Durum) -> GrafikSeçenekleri {
    let veri: Vec<VeriÖğesi> = (0..5)
        .map(|k| {
            let orta = değer(d, 200 + k, 850.0, 250.0);
            let yay = 60.0 + karma(d.tohum, 210 + k) * 120.0;
            VeriÖğesi::from([
                orta - 2.0 * yay,
                orta - yay,
                orta,
                orta + yay,
                orta + 2.0 * yay,
            ])
        })
        .collect();
    temel("Kutu (Boxplot)")
        .x_ekseni(Eksen::kategori().veri(["D1", "D2", "D3", "D4", "D5"]))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(KutuSerisi::yeni().ad("Ölçüm").veri(veri))
}

fn ısı(d: &Durum) -> GrafikSeçenekleri {
    let saatler = ["00", "03", "06", "09", "12", "15", "18", "21"];
    let günler = ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"];
    let mut veri: Vec<VeriÖğesi> = Vec::new();
    for g in 0..günler.len() {
        for s in 0..saatler.len() {
            veri.push(VeriÖğesi::from([
                s as f64,
                g as f64,
                değer(d, 300 + g * 8 + s, 0.0, 19.0),
            ]));
        }
    }
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Isı Haritası"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .ızgara(Izgara::yeni().sol(70.0).sağ(30.0).üst(60.0).alt(40.0))
        .x_ekseni(Eksen::kategori().veri(saatler))
        .y_ekseni(Eksen::kategori().veri(günler))
        .görsel_eşleme(GörselEşleme::yeni())
        .seri(IsıHaritasıSerisi::yeni().ad("Yoğunluk").veri(veri))
}

fn imleyiciler(d: &Durum) -> GrafikSeçenekleri {
    temel("İmleyiciler")
        .ızgara(Izgara::yeni().sol(60.0).sağ(90.0).üst(60.0).alt(45.0))
        .x_ekseni(Eksen::kategori().veri(kategoriler(d.nokta)))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("Üretim")
                .veri(sayılar(d, 9, 250.0, 250.0))
                .im_çizgisi(İmÇizgisi::yeni().yatay(İmDeğeri::Ortalama))
                .im_noktası(İmNoktası::yeni().en_büyük().en_küçük()),
        )
}

fn çoklu_ızgara(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Çoklu Izgara + Bağlantılı İmleç"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen).bağlantılı(true))
        .ızgara_ekle(Izgara::yeni().sol(60.0).sağ(30.0).üst(55.0).alt("52%"))
        .ızgara_ekle(Izgara::yeni().sol(60.0).sağ(30.0).üst("58%").alt(40.0))
        .x_ekseni_ekle(
            Eksen::kategori()
                .veri(kategoriler(d.nokta))
                .ızgara_sırası(0),
        )
        .x_ekseni_ekle(
            Eksen::kategori()
                .veri(kategoriler(d.nokta))
                .ızgara_sırası(1),
        )
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(0))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(1))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Sıcaklık")
                .veri(sayılar(d, 10, 8.0, 20.0)),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Yağış")
                .eksenler(1, 1)
                .veri(sayılar(d, 11, 1.0, 12.0)),
        )
}

fn yakınlaştırma(d: &Durum) -> GrafikSeçenekleri {
    let n = d.nokta.wrapping_mul(10).max(40);
    let veri: Vec<f64> = (0..n)
        .map(|i| {
            let t = i as f64;
            60.0 + (t * 0.12).sin() * 25.0 + (t * 0.4).cos() * 8.0 + karma(d.tohum, 400 + i) * 10.0
        })
        .collect();
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Veri Yakınlaştırma"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(Izgara::yeni().sol(60.0).sağ(30.0).üst(60.0).alt(80.0))
        .x_ekseni(Eksen::kategori().veri(kategoriler(n)).kenar_boşluğu(false))
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

fn kutupsal(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Kutupsal"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .kutupsal(
            KutupsalKoordinat::yeni().açısal_eksen(
                Eksen::kategori().veri(["K", "KD", "D", "GD", "G", "GB", "B", "KB"]),
            ),
        )
        .seri(
            SütunSerisi::yeni().ad("Hız").kutupsal(true).veri(
                (0..8)
                    .map(|i| değer(d, 500 + i, 2.0, 7.0))
                    .collect::<Vec<_>>(),
            ),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Ortalama")
                .kutupsal(true)
                .sembol_boyutu(6.0)
                .veri(
                    (0..8)
                        .map(|i| değer(d, 510 + i, 3.0, 4.0))
                        .collect::<Vec<_>>(),
                ),
        )
}

fn radar(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Radar"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst(30.0))
        .radar(RadarKoordinatı::yeni().göstergeler([
            ("Satış", 100.0),
            ("Yönetim", 100.0),
            ("BT", 100.0),
            ("Destek", 100.0),
            ("Geliştirme", 100.0),
            ("Pazarlama", 100.0),
        ]))
        .seri(
            RadarSerisi::yeni()
                .ad("Bütçe")
                .alan_stili(AlanStili::yeni().opaklık(0.25))
                .veri([
                    (
                        "Ayrılan",
                        (0..6)
                            .map(|i| değer(d, 520 + i, 40.0, 55.0))
                            .collect::<Vec<_>>(),
                    ),
                    (
                        "Harcanan",
                        (0..6)
                            .map(|i| değer(d, 530 + i, 30.0, 60.0))
                            .collect::<Vec<_>>(),
                    ),
                ]),
        )
}

fn takvim(d: &Durum) -> GrafikSeçenekleri {
    let gün_ms = 86_400_000.0f64;
    let yıl_başı = 1_767_225_600_000.0f64; // 2026-01-01 UTC
    let veri: Vec<VeriÖğesi> = (0..365)
        .map(|g| {
            VeriÖğesi::from(vec![
                yıl_başı + g as f64 * gün_ms,
                değer(d, 600 + g as usize, 0.0, 12.0),
            ])
        })
        .collect();
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Takvim Isısı — 2026"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .görsel_eşleme(GörselEşleme::yeni())
        .seri(TakvimSerisi::yeni(2026).ad("Katkılar").veri(veri))
}

fn tema_nehri(d: &Durum) -> GrafikSeçenekleri {
    let katmanlar = ["Rüzgar", "Güneş", "Hidro", "Doğalgaz"];
    let n = d.nokta.max(4);
    let mut veri: Vec<(f64, f64, String)> = Vec::new();
    for (k, katman) in katmanlar.iter().enumerate() {
        for x in 0..n {
            veri.push((
                x as f64,
                değer(d, 700 + k * 64 + x, 8.0, 40.0),
                (*katman).to_string(),
            ));
        }
    }
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Tema Nehri"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .tek_eksen(TekEksen::yeni())
        .seri(TemaNehriSerisi::yeni().ad("Üretim").veri(veri))
}

fn ağaç_haritası(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Ağaç Haritası"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(AğaçHaritasıSerisi::yeni().ad("Disk").kökler([
            AğaçDüğümü::dal(
                "Belgeler",
                vec![
                    AğaçDüğümü::yaprak("Raporlar", değer(d, 801, 10.0, 30.0)),
                    AğaçDüğümü::yaprak("Sunumlar", değer(d, 802, 8.0, 20.0)),
                    AğaçDüğümü::yaprak("Tablolar", değer(d, 803, 4.0, 12.0)),
                ],
            ),
            AğaçDüğümü::dal(
                "Medya",
                vec![
                    AğaçDüğümü::yaprak("Video", değer(d, 804, 30.0, 40.0)),
                    AğaçDüğümü::yaprak("Müzik", değer(d, 805, 10.0, 20.0)),
                    AğaçDüğümü::yaprak("Fotoğraf", değer(d, 806, 8.0, 15.0)),
                ],
            ),
            AğaçDüğümü::yaprak("Sistem", değer(d, 807, 10.0, 20.0)),
        ]))
}

fn güneş(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Güneş Patlaması"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(GüneşPatlamasıSerisi::yeni().ad("Trafik").kökler([
            AğaçDüğümü::dal(
                "Arama",
                vec![
                    AğaçDüğümü::yaprak("Organik", değer(d, 811, 20.0, 30.0)),
                    AğaçDüğümü::yaprak("Reklam", değer(d, 812, 8.0, 15.0)),
                ],
            ),
            AğaçDüğümü::dal(
                "Sosyal",
                vec![
                    AğaçDüğümü::yaprak("Video", değer(d, 813, 6.0, 12.0)),
                    AğaçDüğümü::dal(
                        "Mikroblog",
                        vec![
                            AğaçDüğümü::yaprak("Paylaşım", değer(d, 814, 3.0, 6.0)),
                            AğaçDüğümü::yaprak("Profil", değer(d, 815, 2.0, 5.0)),
                        ],
                    ),
                ],
            ),
            AğaçDüğümü::yaprak("Doğrudan", değer(d, 816, 10.0, 20.0)),
        ]))
}

fn ağaç(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Ağaç"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(AğaçSerisi::yeni().ad("Kuruluş").kökler([AğaçDüğümü::dal(
            "Genel Müdür",
            vec![
                AğaçDüğümü::dal(
                    "Mühendislik",
                    vec![
                        AğaçDüğümü::yaprak("Arayüz", değer(d, 821, 5.0, 12.0)),
                        AğaçDüğümü::yaprak("Altyapı", değer(d, 822, 4.0, 10.0)),
                        AğaçDüğümü::yaprak("Veri", değer(d, 823, 3.0, 8.0)),
                    ],
                ),
                AğaçDüğümü::dal(
                    "Satış",
                    vec![
                        AğaçDüğümü::yaprak("Yurt İçi", değer(d, 824, 3.0, 8.0)),
                        AğaçDüğümü::yaprak("Yurt Dışı", değer(d, 825, 2.0, 6.0)),
                    ],
                ),
                AğaçDüğümü::yaprak("Finans", değer(d, 826, 2.0, 5.0)),
            ],
        )]))
}

fn sankey(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Sankey"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(SankeySerisi::yeni().ad("Enerji").bağlar([
            ("Kömür", "Elektrik", değer(d, 831, 15.0, 20.0)),
            ("Doğalgaz", "Elektrik", değer(d, 832, 10.0, 15.0)),
            ("Güneş", "Elektrik", değer(d, 833, 6.0, 12.0)),
            ("Rüzgar", "Elektrik", değer(d, 834, 8.0, 14.0)),
            ("Elektrik", "Konut", değer(d, 835, 18.0, 20.0)),
            ("Elektrik", "Sanayi", değer(d, 836, 15.0, 18.0)),
            ("Elektrik", "Ulaşım", değer(d, 837, 6.0, 10.0)),
        ]))
}

fn grafo(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Grafo (İlişki Ağı)"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(
            GrafoSerisi::yeni()
                .ad("Ağ")
                .yerleşim(GrafoYerleşimi::Kuvvet)
                .düğümler([
                    GrafoDüğümü::yeni("Çekirdek", değer(d, 841, 24.0, 16.0) as f32).kategori(0),
                    GrafoDüğümü::yeni("Model", değer(d, 842, 14.0, 12.0) as f32).kategori(1),
                    GrafoDüğümü::yeni("Çizim", değer(d, 843, 14.0, 12.0) as f32).kategori(1),
                    GrafoDüğümü::yeni("Ölçek", değer(d, 844, 10.0, 8.0) as f32).kategori(2),
                    GrafoDüğümü::yeni("Eksen", değer(d, 845, 10.0, 8.0) as f32).kategori(2),
                    GrafoDüğümü::yeni("Seri", değer(d, 846, 10.0, 10.0) as f32).kategori(2),
                    GrafoDüğümü::yeni("Olay", değer(d, 847, 8.0, 8.0) as f32).kategori(3),
                ])
                .bağlar([
                    ("Çekirdek", "Model"),
                    ("Çekirdek", "Çizim"),
                    ("Model", "Ölçek"),
                    ("Model", "Seri"),
                    ("Çizim", "Eksen"),
                    ("Çizim", "Olay"),
                    ("Seri", "Eksen"),
                ]),
        )
}

fn kiriş(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Kiriş (Chord)"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .seri(KirişSerisi::yeni().ad("Göç").bağlar([
            ("Kuzey", "Güney", değer(d, 851, 6.0, 10.0)),
            ("Güney", "Doğu", değer(d, 852, 4.0, 8.0)),
            ("Doğu", "Kuzey", değer(d, 853, 3.0, 6.0)),
            ("Kuzey", "Batı", değer(d, 854, 3.0, 7.0)),
            ("Batı", "Güney", değer(d, 855, 2.0, 6.0)),
        ]))
}

fn paralel(d: &Durum) -> GrafikSeçenekleri {
    let veri: Vec<VeriÖğesi> = (0..d.nokta)
        .map(|i| {
            VeriÖğesi::from(vec![
                değer(d, 860 + i * 4, 5.0, 12.0),
                değer(d, 861 + i * 4, 1.0, 4.0),
                değer(d, 862 + i * 4, 4.0, 6.0),
                değer(d, 863 + i * 4, 10.0, 80.0),
            ])
        })
        .collect();
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Paralel Koordinatlar"))
        .seri(
            ParalelSerisi::yeni()
                .ad("Ölçümler")
                .boyutlar(["Fiyat", "Ağırlık", "Puan", "Stok"])
                .veri(veri),
        )
}

fn gösterge_saati(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Gösterge Saati"))
        .seri(
            GöstergeSaatiSerisi::yeni()
                .ad("Yük")
                .değer(değer(d, 870, 15.0, 80.0), "CPU")
                .değer_biçimleyici("{value} %"),
        )
}

fn huni(d: &Durum) -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Huni"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst(30.0))
        .seri(HuniSerisi::yeni().ad("Dönüşüm").veri([
            ("Gösterim", 100.0),
            ("Tıklama", değer(d, 881, 55.0, 30.0)),
            ("Ziyaret", değer(d, 882, 35.0, 20.0)),
            ("Sepet", değer(d, 883, 20.0, 15.0)),
            ("Sipariş", değer(d, 884, 5.0, 12.0)),
        ]))
}

#[derive(Clone, Deserialize)]
struct ManifestBaşlığı {
    en: String,
}

#[derive(Clone, Deserialize)]
struct ManifestKanıtı {
    api: String,
    #[serde(rename = "statik_görsel")]
    statik_görsel: String,
    etkileşim: String,
}

#[derive(Clone, Deserialize)]
struct ManifestKaydı {
    id: String,
    #[serde(rename = "başlık")]
    başlık: ManifestBaşlığı,
    #[serde(rename = "kategoriler")]
    kategoriler: Vec<String>,
    difficulty: u32,
    since: Option<String>,
    #[serde(rename = "resmi_sayfada_görünür")]
    resmi_sayfada_görünür: bool,
    cizelge_fixture: Option<String>,
    kapsam_durumu: String,
    #[serde(rename = "sahip_faz")]
    sahip_faz: Option<u32>,
    #[serde(rename = "kanıt")]
    kanıt: ManifestKanıtı,
}

#[derive(Deserialize)]
struct UyumÖzeti {
    #[serde(rename = "görünür_kapsam_içi")]
    görünür_kapsam_içi: usize,
    #[serde(rename = "tam_kanıtlı")]
    tam_kanıtlı: usize,
    #[serde(rename = "kategori_sırası")]
    kategori_sırası: Vec<String>,
}

fn manifesti_oku() -> (Vec<ManifestKaydı>, UyumÖzeti) {
    let manifest =
        serde_json::from_str::<Vec<ManifestKaydı>>(include_str!("../uyum/galeri_manifest.json"));
    let özet = serde_json::from_str::<UyumÖzeti>(include_str!("../uyum/ozet.json"));
    match (manifest, özet) {
        (Ok(manifest), Ok(özet)) => (manifest, özet),
        (manifest, özet) => {
            if let Err(hata) = manifest {
                eprintln!("Galeri manifesti okunamadı: {hata}");
            }
            if let Err(hata) = özet {
                eprintln!("Uyum özeti okunamadı: {hata}");
            }
            (
                Vec::new(),
                UyumÖzeti {
                    görünür_kapsam_içi: 0,
                    tam_kanıtlı: 0,
                    kategori_sırası: Vec::new(),
                },
            )
        }
    }
}

/// Manifestteki resmi kimliği mevcut yerel fixture üreticisine bağlar.
/// Bu tablo yalnız render üreticilerini tutar; liste/kategori/kanıt durumu
/// bütünüyle üretilmiş manifestten gelir.
fn fixture_üreticisi(id: &str) -> Option<Üretici> {
    match id {
        "line-simple" => Some(çizgi),
        "area-basic" => Some(alan),
        "bar-simple" => Some(sütun),
        "bar-stack" => Some(yığın_sütun),
        "pie-simple" => Some(pasta),
        "pie-doughnut" => Some(halka),
        "scatter-simple" => Some(saçılım),
        "scatter-effect" => Some(efektli_saçılım),
        "candlestick-simple" => Some(mum),
        "boxplot-light-velocity" => Some(kutu),
        "heatmap-cartesian" => Some(ısı),
        "line-marker" => Some(imleyiciler),
        "grid-multiple" => Some(çoklu_ızgara),
        "mix-zoom-on-value" => Some(yakınlaştırma),
        "bar-polar-real-estate" => Some(kutupsal),
        "radar" => Some(radar),
        "calendar-simple" => Some(takvim),
        "themeRiver-basic" => Some(tema_nehri),
        "treemap-simple" => Some(ağaç_haritası),
        "sunburst-simple" => Some(güneş),
        "tree-basic" => Some(ağaç),
        "sankey-simple" => Some(sankey),
        "graph-simple" => Some(grafo),
        "chord-simple" => Some(kiriş),
        "parallel-simple" => Some(paralel),
        "gauge-simple" => Some(gösterge_saati),
        "funnel" => Some(huni),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// Galeri görünümü
// ---------------------------------------------------------------------

struct Galeri {
    grafik: Entity<GrafikGörünümü>,
    durum: Durum,
    manifest: Vec<ManifestKaydı>,
    kategoriler: Vec<String>,
    görünür_hedef: usize,
    tam_kanıtlı: usize,
    kategori: Option<String>,
    seçili: usize,
    detay: bool,
}

impl Galeri {
    fn yeni(cx: &mut Context<Self>) -> Self {
        let durum = Durum {
            nokta: 7,
            tohum: 1,
            koyu: false,
        };
        let (tüm_manifest, özet) = manifesti_oku();
        let manifest: Vec<_> = tüm_manifest
            .into_iter()
            .filter(|kayıt| {
                kayıt.resmi_sayfada_görünür && !kayıt.kapsam_durumu.starts_with("kapsam_dışı_")
            })
            .collect();
        let seçili = manifest
            .iter()
            .position(|kayıt| kayıt.id == "line-simple")
            .unwrap_or(0);
        let ilk_id = manifest
            .get(seçili)
            .map(|kayıt| kayıt.id.as_str())
            .unwrap_or("");
        let grafik = cx.new(|_| GrafikGörünümü::yeni(Self::üret(ilk_id, &durum)));
        Galeri {
            grafik,
            durum,
            manifest,
            kategoriler: özet.kategori_sırası,
            görünür_hedef: özet.görünür_kapsam_içi,
            tam_kanıtlı: özet.tam_kanıtlı,
            kategori: None,
            seçili,
            detay: false,
        }
    }

    fn üret(id: &str, durum: &Durum) -> GrafikSeçenekleri {
        fixture_üreticisi(id)
            .map(|üretici| üretici(durum).koyu(durum.koyu))
            .unwrap_or_else(|| {
                GrafikSeçenekleri::yeni()
                    .koyu(durum.koyu)
                    .başlık(Başlık::yeni().metin(format!("Kanıt bekliyor · {id}")))
            })
    }

    /// Grafiği güncel durumla yeniden kurar (geçiş animasyonlu).
    fn uygula(&mut self, cx: &mut Context<Self>) {
        let id = self
            .manifest
            .get(self.seçili)
            .map(|kayıt| kayıt.id.as_str())
            .unwrap_or("");
        let seçenekler = Self::üret(id, &self.durum);
        self.grafik.update(cx, |grafik, cx| {
            if let Err(hata) = grafik.seçenekleri_değiştir(seçenekler, cx) {
                eprintln!("Seçenekler uygulanamadı: {hata}");
            }
        });
        cx.notify();
    }

    fn kategori_sayısı(&self, kategori: &str) -> usize {
        self.manifest
            .iter()
            .filter(|kayıt| kayıt.kategoriler.iter().any(|aday| aday == kategori))
            .count()
    }
}

impl Render for Galeri {
    fn render(&mut self, _pencere: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let koyu = self.durum.koyu;
        // Kabuk renkleri (çizelge kendi temasını kendisi çözer).
        let zemin = if koyu {
            gpui::rgb(0x141418)
        } else {
            gpui::rgb(0xf4f7fd)
        };
        let panel = if koyu {
            gpui::rgb(0x1b1b20)
        } else {
            gpui::rgb(0xffffff)
        };
        let metin = if koyu {
            gpui::rgb(0xe8ebf0)
        } else {
            gpui::rgb(0x3c3c41)
        };
        let soluk = if koyu {
            gpui::rgb(0x9ea0a5)
        } else {
            gpui::rgb(0x86878c)
        };
        let vurgu = gpui::rgb(0x5070dd);
        let vurgu_zemini = if koyu {
            gpui::rgb(0x28304d)
        } else {
            gpui::rgb(0xe8edfb)
        };
        let çizgi_rengi = if koyu {
            gpui::rgb(0x303034)
        } else {
            gpui::rgb(0xe8ebf0)
        };

        // --- Resmi Explore.vue sırasından üretilen kategori menüsü ---
        let mut menü = div()
            .id("galeri-menü")
            .w(px(220.0))
            .h_full()
            .flex()
            .flex_col()
            .flex_none()
            .overflow_y_scroll()
            .bg(panel)
            .border_r_1()
            .border_color(çizgi_rengi)
            .p_2()
            .child(
                div()
                    .p_2()
                    .text_lg()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(metin)
                    .child("Cizelge Examples"),
            );
        let tüm_seçili = self.kategori.is_none();
        menü = menü.child(
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .cursor_pointer()
                .text_color(if tüm_seçili { vurgu } else { metin })
                .when(tüm_seçili, |s| {
                    s.bg(vurgu_zemini).font_weight(gpui::FontWeight::SEMIBOLD)
                })
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|bu, _: &MouseDownEvent, _, cx| {
                        bu.kategori = None;
                        bu.detay = false;
                        cx.notify();
                    }),
                )
                .child(SharedString::from(format!("All ({})", self.manifest.len()))),
        );
        for kategori_adı in self.kategoriler.clone() {
            let sayı = self.kategori_sayısı(&kategori_adı);
            if sayı == 0 {
                continue;
            }
            let seçili = self.kategori.as_deref() == Some(kategori_adı.as_str());
            let tıklanan = kategori_adı.clone();
            menü = menü.child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .cursor_pointer()
                    .text_color(if seçili { vurgu } else { metin })
                    .when(seçili, |s| {
                        s.bg(vurgu_zemini).font_weight(gpui::FontWeight::SEMIBOLD)
                    })
                    .hover(move |s| if seçili { s } else { s.bg(vurgu_zemini) })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |bu, _: &MouseDownEvent, _, cx| {
                            bu.kategori = Some(tıklanan.clone());
                            bu.detay = false;
                            cx.notify();
                        }),
                    )
                    .child(SharedString::from(format!("{kategori_adı} ({sayı})"))),
            );
        }

        // --- Üst düğme çubuğu ---
        let düğme = |etiket: &str| {
            div()
                .px_3()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(çizgi_rengi)
                .bg(panel)
                .text_color(metin)
                .text_sm()
                .cursor_pointer()
                .hover(move |s| s.bg(vurgu_zemini))
                .child(SharedString::from(etiket.to_string()))
        };
        let durum_yazısı = SharedString::from(format!(
            "{} / {} tam kanıtlı · {} yerel fixture · {} nokta · tohum {}",
            self.tam_kanıtlı,
            self.görünür_hedef,
            self.manifest
                .iter()
                .filter(|kayıt| kayıt.cizelge_fixture.is_some())
                .count(),
            self.durum.nokta,
            self.durum.tohum
        ));
        let araç_çubuğu = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .p_2()
            .border_b_1()
            .border_color(çizgi_rengi)
            .bg(panel)
            .child(
                div()
                    .text_color(soluk)
                    .text_sm()
                    .flex_1()
                    .child(durum_yazısı),
            )
            .child(düğme("− Nokta").on_mouse_down(
                MouseButton::Left,
                cx.listener(|bu, _: &MouseDownEvent, _, cx| {
                    bu.durum.nokta = bu.durum.nokta.saturating_sub(1).max(2);
                    bu.uygula(cx);
                }),
            ))
            .child(düğme("＋ Nokta").on_mouse_down(
                MouseButton::Left,
                cx.listener(|bu, _: &MouseDownEvent, _, cx| {
                    bu.durum.nokta = bu.durum.nokta.saturating_add(1).min(24);
                    bu.uygula(cx);
                }),
            ))
            .child(düğme("🎲 Değerleri Karıştır").on_mouse_down(
                MouseButton::Left,
                cx.listener(|bu, _: &MouseDownEvent, _, cx| {
                    bu.durum.tohum = bu.durum.tohum.wrapping_add(1);
                    bu.uygula(cx);
                }),
            ))
            .child(
                düğme(if koyu {
                    "☀ Açık Tema"
                } else {
                    "🌙 Koyu Tema"
                })
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|bu, _: &MouseDownEvent, _, cx| {
                        bu.durum.koyu = !bu.durum.koyu;
                        bu.uygula(cx);
                    }),
                ),
            );

        let içerik =
            if self.detay {
                let kayıt = self.manifest.get(self.seçili).cloned();
                let geri = div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(çizgi_rengi)
                    .cursor_pointer()
                    .child("← Galeriye dön")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|bu, _: &MouseDownEvent, _, cx| {
                            bu.detay = false;
                            cx.notify();
                        }),
                    );
                let mut ayrıntı = div()
                    .id("galeri-detay")
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .p_3()
                    .gap_3()
                    .child(geri);
                if let Some(kayıt) = kayıt {
                    let fixture_durumu = if kayıt.cizelge_fixture.is_some() {
                        "Yerel fixture bağlı · kanıt kapıları bekliyor"
                    } else {
                        "Yerel fixture bekliyor"
                    };
                    ayrıntı =
                        ayrıntı.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_3()
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(500.0))
                                        .h_full()
                                        .rounded_lg()
                                        .bg(panel)
                                        .child(self.grafik.clone()),
                                )
                                .child(
                                    div()
                                        .w(px(310.0))
                                        .flex_none()
                                        .p_3()
                                        .rounded_lg()
                                        .border_1()
                                        .border_color(çizgi_rengi)
                                        .bg(panel)
                                        .text_color(metin)
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .child(SharedString::from(kayıt.başlık.en.clone())),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(soluk)
                                                .child(SharedString::from(kayıt.id.clone())),
                                        )
                                        .child(div().mt_2().child(SharedString::from(
                                            kayıt.kategoriler.join(" · "),
                                        )))
                                        .child(
                                            div()
                                                .mt_2()
                                                .text_color(gpui::rgb(0xc2410c))
                                                .child(fixture_durumu),
                                        )
                                        .child(div().mt_3().child(SharedString::from(format!(
                                            "API: {}",
                                            kayıt.kanıt.api
                                        ))))
                                        .child(div().child(SharedString::from(format!(
                                            "Görsel: {}",
                                            kayıt.kanıt.statik_görsel
                                        ))))
                                        .child(div().child(SharedString::from(format!(
                                            "Etkileşim: {}",
                                            kayıt.kanıt.etkileşim
                                        ))))
                                        .child(div().mt_3().text_sm().text_color(soluk).child(
                                            SharedString::from(format!(
                                                "Faz {} · zorluk {} · since {}",
                                                kayıt.sahip_faz.unwrap_or(0),
                                                kayıt.difficulty,
                                                kayıt.since.unwrap_or_else(|| "—".to_string())
                                            )),
                                        )),
                                ),
                        );
                }
                ayrıntı
            } else {
                let indeksler: Vec<usize> = self
                    .manifest
                    .iter()
                    .enumerate()
                    .filter(|(_, kayıt)| {
                        self.kategori.as_ref().is_none_or(|kategori| {
                            kayıt.kategoriler.iter().any(|aday| aday == kategori)
                        })
                    })
                    .map(|(sıra, _)| sıra)
                    .collect();
                let mut kartlar = div()
                    .id("galeri-kartlar")
                    .flex_1()
                    .h_full()
                    .overflow_y_scroll()
                    .p_3()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_start()
                    .gap_3();
                for sıra in indeksler {
                    let Some(kayıt) = self.manifest.get(sıra) else {
                        continue;
                    };
                    let bağlı = kayıt.cizelge_fixture.is_some();
                    let üst_renk = if bağlı {
                        gpui::rgb(0xd97706)
                    } else {
                        gpui::rgb(0xdc2626)
                    };
                    kartlar = kartlar.child(
                        div()
                            .id(("galeri-kartı", sıra))
                            .w(px(260.0))
                            .h(px(190.0))
                            .flex_none()
                            .p_3()
                            .rounded_lg()
                            .border_1()
                            .border_color(çizgi_rengi)
                            .border_t_4()
                            .border_color(üst_renk)
                            .bg(panel)
                            .cursor_pointer()
                            .hover(move |s| s.bg(vurgu_zemini))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |bu, _: &MouseDownEvent, _, cx| {
                                    bu.seçili = sıra;
                                    bu.detay = true;
                                    bu.uygula(cx);
                                }),
                            )
                            .child(
                                div()
                                    .h(px(92.0))
                                    .rounded_md()
                                    .bg(if koyu {
                                        gpui::rgb(0x111318)
                                    } else {
                                        gpui::rgb(0xf1f3f8)
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(soluk)
                                    .text_sm()
                                    .child(if bağlı {
                                        "Fixture bağlı"
                                    } else {
                                        "Kanıt bekliyor"
                                    }),
                            )
                            .child(
                                div()
                                    .mt_2()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(metin)
                                    .child(SharedString::from(kayıt.başlık.en.clone())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(soluk)
                                    .child(SharedString::from(kayıt.id.clone())),
                            )
                            .child(div().mt_1().text_xs().text_color(üst_renk).child(
                                SharedString::from(format!(
                                    "API {} · görsel {}",
                                    kayıt.kanıt.api, kayıt.kanıt.statik_görsel
                                )),
                            )),
                    );
                }
                kartlar
            };

        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(zemin)
            .text_color(metin)
            .child(menü)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(araç_çubuğu)
                    .child(içerik),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let sınırlar = Bounds::centered(None, size(px(1360.0), px(840.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(sınırlar)),
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(Galeri::yeni),
        )
        .unwrap_or_else(|hata| {
            eprintln!("Pencere açılamadı: {hata}");
            std::process::exit(1);
        });
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
