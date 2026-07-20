//! Uyum kanıt hattı için belirlenimci, başsız PNG fixture üreticisi.
//!
//! Bu örnek kullanıcı galerisi değildir. `tools/uyum/kanit.mjs`, kilitli
//! ECharts referanslarıyla karşılaştırılacak kareleri bu ikili üzerinden
//! üretir; boyama hattı gerçek `PikselYüzeyi` ve `grafiği_boya` yoludur.

use std::path::PathBuf;
use std::sync::Arc;

use base64::Engine as _;
use cizelge::hazir::*;
use cizelge::yardimci::bicim::ondalık_kırp;
use serde::Deserialize;
use serde::de::DeserializeOwned;

#[path = "uyum_veri/area_rainfall.rs"]
mod area_rainfall_verisi;
#[path = "uyum_veri/custom_calendar_icon.rs"]
mod custom_calendar_icon_verisi;
#[path = "uyum_veri/perlin.rs"]
mod perlin;

struct Girdi {
    id: String,
    çıktı: PathBuf,
    kare: f32,
    durum: String,
    genişlik: f32,
    yükseklik: f32,
}

/// Resmî referans üreticisinin sabitlediği Mulberry32 akışı.
fn kanıt_rastgele(tohum: &mut u32) -> f64 {
    *tohum = tohum.wrapping_add(0x6d2b_79f5);
    let mut t = (*tohum ^ (*tohum >> 15)).wrapping_mul(1 | *tohum);
    t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
    f64::from(t ^ (t >> 14)) / 4_294_967_296.0
}

fn argümanları_oku() -> Result<Girdi, String> {
    let mut id = None;
    let mut çıktı = None;
    let mut kare = 1.0_f32;
    let mut durum = String::from("başlangıç");
    let mut genişlik = 700.0_f32;
    let mut yükseklik = 525.0_f32;
    let mut argümanlar = std::env::args().skip(1);
    while let Some(argüman) = argümanlar.next() {
        match argüman.as_str() {
            "--id" => id = argümanlar.next(),
            "--output" => çıktı = argümanlar.next().map(PathBuf::from),
            "--frame" => {
                let ham = argümanlar
                    .next()
                    .ok_or_else(|| "--frame değeri eksik".to_string())?;
                kare = ham
                    .parse::<f32>()
                    .map_err(|hata| format!("geçersiz --frame: {hata}"))?;
            }
            "--state" => {
                durum = argümanlar
                    .next()
                    .ok_or_else(|| "--state değeri eksik".to_string())?;
            }
            "--width" => {
                genişlik = argümanlar
                    .next()
                    .ok_or_else(|| "--width değeri eksik".to_string())?
                    .parse::<f32>()
                    .map_err(|hata| format!("geçersiz --width: {hata}"))?;
            }
            "--height" => {
                yükseklik = argümanlar
                    .next()
                    .ok_or_else(|| "--height değeri eksik".to_string())?
                    .parse::<f32>()
                    .map_err(|hata| format!("geçersiz --height: {hata}"))?;
            }
            bilinmeyen => return Err(format!("bilinmeyen argüman: {bilinmeyen}")),
        }
    }
    Ok(Girdi {
        id: id.ok_or_else(|| "--id zorunludur".to_string())?,
        çıktı: çıktı.ok_or_else(|| "--output zorunludur".to_string())?,
        kare: kare.clamp(0.0, 1.0),
        durum,
        genişlik: genişlik.max(1.0),
        yükseklik: yükseklik.max(1.0),
    })
}

fn line_simple() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer())
        .seri(ÇizgiSerisi::yeni().veri([150.0, 230.0, 224.0, 218.0, 135.0, 147.0, 260.0]))
}

fn line_markline() -> GrafikSeçenekleri {
    use İmÇizgisiEtiketKonumu as Konum;

    let konumlar = [
        ("start", Konum::Başlangıç),
        ("middle", Konum::Orta),
        ("end", Konum::Bitiş),
        ("insideStart", Konum::İçBaşlangıç),
        ("insideStartTop", Konum::İçBaşlangıçÜst),
        ("insideStartBottom", Konum::İçBaşlangıçAlt),
        ("insideMiddle", Konum::İçOrta),
        ("insideMiddleTop", Konum::İçOrtaÜst),
        ("insideMiddleBottom", Konum::İçOrtaAlt),
        ("insideEnd", Konum::İçBitiş),
        ("insideEndTop", Konum::İçBitişÜst),
        ("insideEndBottom", Konum::İçBitişAlt),
    ];
    let mut im_çizgisi = İmÇizgisi::yeni()
        .etiket(
            Etiket::yeni()
                .göster(true)
                .yazı(YazıStili::yeni().boyut(14.0).renk("#333")),
        )
        .etiket_uzaklığı(20.0, 8.0);
    for (sıra, (ad, konum)) in konumlar.into_iter().enumerate() {
        im_çizgisi = im_çizgisi.tanım(
            İmÇizgisiTanımı::yeni(
                İmYönü::Yatay,
                İmDeğeri::Değer(1.8 - 0.2 * (sıra / 3) as f64),
            )
            .ad(ad)
            .etiket(
                İmÇizgisiEtiketYaması::yeni()
                    .biçimleyici("{b}")
                    .konum(konum),
            ),
        );
        if ad != "middle" {
            let metin = if ad == "insideMiddle" {
                "insideMiddle / middle"
            } else {
                ad
            };
            im_çizgisi = im_çizgisi.parça(
                İmÇizgisiParçası::koordinatlar((0.0, 0.3), (3.0, 1.0))
                    .ad(format!("start: {ad}"))
                    .etiket(
                        İmÇizgisiEtiketYaması::yeni()
                            .biçimleyici(metin)
                            .konum(konum),
                    ),
            );
        }
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ızgara(Izgara::yeni().üst(30).sol(60).sağ(60).alt(40))
        .x_ekseni(
            Eksen::kategori()
                .veri(["A", "B", "C", "D", "E"])
                .bölme_alanı_göster(true),
        )
        .y_ekseni(Eksen::değer().en_çok(2.0))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("line")
                .sembol_boyutu(6.0)
                .im_çizgisi(im_çizgisi)
                .veri([0.3, 1.4, 1.2, 1.0, 0.6]),
        )
}

fn line_marker() -> GrafikSeçenekleri {
    let mut en_düşük_çizgileri = İmÇizgisi::yeni().yatay(İmDeğeri::Ortalama);
    // Resmî örnekteki ikinci markLine, serinin en büyük noktasından
    // grafiğin sağ kenarına uzanır ve sağ uçta "Max" etiketini taşır.
    en_düşük_çizgileri.parçalar.push(İmÇizgisiParçası {
        ad: Some("Max".to_owned()),
        başlangıç: İmÇizgisiUcu::İstatistik(İmDeğeri::EnBüyük),
        bitiş: İmÇizgisiUcu::Koordinat(6.0, 5.0),
        başlangıç_simgesi: İmÇizgisiUçSimgesi::Daire,
        bitiş_simgesi: İmÇizgisiUçSimgesi::Yok,
        etiket: None,
    });

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Temperature Change in the Coming Week")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_yakınlaştırma(true)
                .veri_görünümü(true)
                .sihirli_tür(true, true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .x_ekseni(
            Eksen::kategori()
                .kenar_boşluğu(false)
                .veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .y_ekseni(Eksen::değer().etiket_biçimleyici("{value} °C"))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Highest")
                .im_noktası(İmNoktası::yeni().en_büyük().en_küçük())
                .im_çizgisi(İmÇizgisi::yeni().yatay(İmDeğeri::Ortalama))
                .veri([10, 11, 13, 11, 12, 12, 9]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Lowest")
                .im_noktası(İmNoktası::yeni().adlı_koordinat_değeri("周最低", 1.0, -1.5, -2.0))
                .im_çizgisi(en_düşük_çizgileri)
                .veri([1, -2, 2, 5, 3, 2, 0]),
        )
}

fn bar_simple() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer())
        .seri(SütunSerisi::yeni().veri([120.0, 200.0, 150.0, 80.0, 70.0, 110.0, 130.0]))
}

fn bar1() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Rainfall vs Evaporation")
                .alt_metin("Fake Data")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(
            Gösterge::yeni()
                .iç_boşluk(15.0)
                .veri(["Rainfall", "Evaporation"]),
        )
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_görünümü(true)
                .sihirli_tür(true, true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .x_ekseni(Eksen::kategori().veri([
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ]))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("Rainfall")
                .im_noktası(İmNoktası::yeni().en_büyük().en_küçük())
                .im_çizgisi(İmÇizgisi::yeni().yatay(İmDeğeri::Ortalama))
                .veri([
                    2.0, 4.9, 7.0, 23.2, 25.6, 76.7, 135.6, 162.2, 32.6, 20.0, 6.4, 3.3,
                ]),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Evaporation")
                .im_noktası(
                    İmNoktası::yeni()
                        .adlı_koordinat_değeri("Max", 7.0, 183.0, 182.2)
                        .adlı_koordinat_değeri("Min", 11.0, 3.0, 2.3),
                )
                .im_çizgisi(İmÇizgisi::yeni().yatay(İmDeğeri::Ortalama))
                .veri([
                    2.6, 5.9, 9.0, 26.4, 28.7, 70.7, 175.6, 182.2, 48.7, 18.8, 6.0, 2.3,
                ]),
        )
}

fn mix_line_bar() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Çapraz),
        )
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_görünümü(true)
                .sihirli_tür(true, true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .gösterge(Gösterge::yeni().iç_boşluk(15.0).veri([
            "Evaporation",
            "Precipitation",
            "Temperature",
        ]))
        .x_ekseni_ekle(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni_ekle(
            Eksen::değer()
                .ad("Precipitation")
                .en_az(0.0)
                .en_çok(250.0)
                .bölme_sayısı(5)
                .etiket_biçimleyici("{value} ml"),
        )
        .y_ekseni_ekle(
            Eksen::değer()
                .ad("Temperature")
                .en_az(0.0)
                .en_çok(25.0)
                .bölme_sayısı(5)
                .etiket_biçimleyici("{value} °C"),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Evaporation")
                .veri([2.0, 4.9, 7.0, 23.2, 25.6, 76.7, 135.6]),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Precipitation")
                .veri([2.6, 5.9, 9.0, 26.4, 28.7, 70.7, 175.6]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Temperature")
                .eksenler(0, 1)
                .veri([2.0, 2.2, 3.3, 4.5, 6.3, 10.2, 20.3]),
        )
}

fn multiple_x_axis() -> GrafikSeçenekleri {
    let çizgi = |renk: u32| {
        EksenÇizgisi::yeni()
            .sıfır(EksenSıfırKipi::Kapalı)
            .renk(renk)
    };
    let çentik = EksenÇentiği {
        etiketle_hizala: true,
        ..Default::default()
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .palet([0x5470c6u32, 0xee6666u32])
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Kapalı)
                .imleç(İmleçTürü::Çapraz),
        )
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .ızgara(Izgara::yeni().üst(70).alt(50))
        .x_ekseni_ekle(
            Eksen::kategori()
                .çentik(çentik.clone())
                .çizgi(çizgi(0xee6666))
                .veri([
                    "2016-1", "2016-2", "2016-3", "2016-4", "2016-5", "2016-6", "2016-7", "2016-8",
                    "2016-9", "2016-10", "2016-11", "2016-12",
                ]),
        )
        .x_ekseni_ekle(
            Eksen::kategori()
                .çentik(çentik)
                .çizgi(çizgi(0x5470c6))
                .veri([
                    "2015-1", "2015-2", "2015-3", "2015-4", "2015-5", "2015-6", "2015-7", "2015-8",
                    "2015-9", "2015-10", "2015-11", "2015-12",
                ]),
        )
        .y_ekseni(Eksen::değer())
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Precipitation(2015)")
                .eksenler(1, 0)
                .yumuşat(true)
                .veri([
                    2.6, 5.9, 9.0, 26.4, 28.7, 70.7, 175.6, 182.2, 48.7, 18.8, 6.0, 2.3,
                ]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Precipitation(2016)")
                .yumuşat(true)
                .veri([
                    3.9, 5.9, 11.1, 18.7, 48.3, 69.2, 231.6, 46.6, 55.4, 18.4, 10.3, 0.7,
                ]),
        )
}

fn multiple_y_axis() -> GrafikSeçenekleri {
    let değer_ekseni = |ad: &str, konum: EksenKonumu, renk: u32, kaydırma: f32, birim: &str| {
        Eksen::değer()
            .ad(ad)
            .konum(konum)
            .kaydırma(kaydırma)
            .çentik_hizala(true)
            .çizgi(EksenÇizgisi::yeni().göster(true).renk(renk))
            .etiket_biçimleyici(format!("{{value}} {birim}"))
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .palet([0x5070ddu32, 0xb6d634u32, 0x505372u32])
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Çapraz),
        )
        .ızgara(Izgara::yeni().sağ("20%"))
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_görünümü(true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .gösterge(Gösterge::yeni().iç_boşluk(15.0).veri([
            "Evaporation",
            "Precipitation",
            "Temperature",
        ]))
        .x_ekseni(
            Eksen::kategori()
                .çentik(EksenÇentiği {
                    etiketle_hizala: true,
                    ..Default::default()
                })
                .veri([
                    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
                    "Dec",
                ]),
        )
        .y_ekseni_ekle(değer_ekseni(
            "Evaporation",
            EksenKonumu::Sağ,
            0x5070dd,
            0.0,
            "ml",
        ))
        .y_ekseni_ekle(değer_ekseni(
            "Precipitation",
            EksenKonumu::Sağ,
            0xb6d634,
            80.0,
            "ml",
        ))
        .y_ekseni_ekle(değer_ekseni("温度", EksenKonumu::Sol, 0x505372, 0.0, "°C"))
        .seri(SütunSerisi::yeni().ad("Evaporation").veri([
            2.0, 4.9, 7.0, 23.2, 25.6, 76.7, 135.6, 162.2, 32.6, 20.0, 6.4, 3.3,
        ]))
        .seri(
            SütunSerisi::yeni()
                .ad("Precipitation")
                .eksenler(0, 1)
                .veri([
                    2.6, 5.9, 9.0, 26.4, 28.7, 70.7, 175.6, 182.2, 48.7, 18.8, 6.0, 2.3,
                ]),
        )
        .seri(ÇizgiSerisi::yeni().ad("Temperature").eksenler(0, 2).veri([
            2.0, 2.2, 3.3, 4.5, 6.3, 10.2, 20.3, 23.4, 23.0, 16.5, 12.0, 6.2,
        ]))
}

fn line_smooth() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer())
        .seri(
            ÇizgiSerisi::yeni()
                .yumuşat(true)
                .veri([820, 932, 901, 934, 1290, 1330, 1320]),
        )
}

fn area_basic() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(
            Eksen::kategori()
                .kenar_boşluğu(false)
                .veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .y_ekseni(Eksen::değer())
        .seri(
            ÇizgiSerisi::yeni()
                .alan_stili(AlanStili::default())
                .veri([820, 932, 901, 934, 1290, 1330, 1320]),
        )
}

/// JavaScript `Math.round` davranışı. Rust'ın `round` yöntemi negatif yarım
/// değerleri sıfırdan uzağa yuvarladığından, ECharts örnek verisinin rastgele
/// yürüyüşünü birebir korumak için `floor(x + 0.5)` kullanılır.
fn javascript_yuvarla(değer: f64) -> f64 {
    (değer + 0.5).floor()
}

/// Pozitif örnek verilerinde JavaScript `toFixed(1)` ile aynı onda bir
/// hassasiyetini sayısal değere geri çevirir.
fn javascript_onda_bir(değer: f64) -> f64 {
    (değer * 10.0).round() / 10.0
}

fn area_simple() -> GrafikSeçenekleri {
    const GÜN_MS: f64 = 86_400_000.0;
    let mut tohum = 0x5eed_1234;
    let mut taban =
        cizelge::yardimci::takvim::takvimden_ana(cizelge::yardimci::takvim::TakvimAnı {
            yıl: 1968,
            ay: 10,
            gün: 3,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
    let mut tarihler = Vec::with_capacity(19_999);
    let mut değerler = Vec::with_capacity(20_000);
    let mut önceki = kanıt_rastgele(&mut tohum) * 300.0;
    değerler.push(önceki);
    for _ in 1..20_000 {
        taban += GÜN_MS;
        let tarih = cizelge::yardimci::takvim::andan_takvime(taban);
        tarihler.push(format!("{}/{}/{}", tarih.yıl, tarih.ay, tarih.gün));
        önceki = javascript_yuvarla((kanıt_rastgele(&mut tohum) - 0.5) * 20.0 + önceki);
        değerler.push(önceki);
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Large Area Chart")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_yakınlaştırma(true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .x_ekseni(Eksen::kategori().kenar_boşluğu(false).veri(tarihler))
        .y_ekseni(Eksen::değer().sayısal_kenar_boşluğu(0.0, "100%"))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(0.0, 10.0))
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().aralık(0.0, 10.0))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Fake Data")
                .sembol(Sembol::Yok)
                .örnekleme(Örnekleme::Lttb)
                .öğe_stili(ÖğeStili::yeni().renk("rgb(255, 70, 131)"))
                .alan_stili(AlanStili::yeni().renk(Dolgu::doğrusal(
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    vec![
                        RenkDurağı::yeni(0.0, "rgb(255, 158, 68)"),
                        RenkDurağı::yeni(1.0, "rgb(255, 70, 131)"),
                    ],
                )))
                .veri(değerler),
        )
}

fn area_time_axis() -> GrafikSeçenekleri {
    const GÜN_MS: f64 = 86_400_000.0;
    let mut tohum = 0x5eed_1234;
    // ECharts örneği yerel 1988-10-03 gece yarısını kullanır. Cizelge zaman
    // ekseni UTC tabanlı olduğundan aynı takvim gününü UTC'de kurmak, veri ve
    // çentik geometrisini saat diliminden bağımsız ve belirlenimci tutar.
    let mut taban =
        cizelge::yardimci::takvim::takvimden_ana(cizelge::yardimci::takvim::TakvimAnı {
            yıl: 1988,
            ay: 10,
            gün: 3,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
    let mut değer = kanıt_rastgele(&mut tohum) * 300.0;
    let mut veri = Vec::with_capacity(20_000);
    veri.push(VeriÖğesi::yeni([taban, değer]));
    for _ in 1..20_000 {
        taban += GÜN_MS;
        değer = javascript_yuvarla((kanıt_rastgele(&mut tohum) - 0.5) * 20.0 + değer);
        veri.push(VeriÖğesi::yeni([taban, değer]));
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Large Ara Chart")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_yakınlaştırma(true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .x_ekseni(Eksen::zaman().sayısal_kenar_boşluğu(0.0, 0.0))
        .y_ekseni(Eksen::değer().sayısal_kenar_boşluğu(0.0, "100%"))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(0.0, 20.0))
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().aralık(0.0, 20.0))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Fake Data")
                .yumuşat(true)
                .sembol(Sembol::Yok)
                .alan_stili(AlanStili::default())
                .veri(veri),
        )
}

fn area_rainfall() -> Result<GrafikSeçenekleri, String> {
    const SAAT_MS: f64 = 3_600_000.0;
    let (akış, yağış) = area_rainfall_verisi::verileri_çöz()?;
    let başlangıç =
        cizelge::yardimci::takvim::takvimden_ana(cizelge::yardimci::takvim::TakvimAnı {
            yıl: 2009,
            ay: 6,
            gün: 12,
            saat: 2,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
    let mut kategoriler = (0..akış.len())
        .map(|sıra| {
            let an = cizelge::yardimci::takvim::andan_takvime(başlangıç + sıra as f64 * SAAT_MS);
            format!("{}/{}/{}\n{}:00", an.yıl, an.ay, an.gün, an.saat)
        })
        .collect::<Vec<_>>();

    // Resmî fixture'ın ilk haftasındaki üç tarih yazım hatası görünür
    // pencerenin dışında olsa da kategori dizisini kaynakla birebir korur.
    for sıra in [94_usize, 118, 166] {
        let kategori = kategoriler
            .get_mut(sıra)
            .ok_or_else(|| format!("area-rainfall kategori {sıra} eksik"))?;
        *kategori = "2009/6/15\n0:00".to_owned();
    }

    let im_alanı = |başlangıç: f64, bitiş: f64| {
        İmAlanı::yeni()
            .x_aralığı("", başlangıç, bitiş)
            .stil(ÖğeStili::yeni().opaklık(0.3))
    };

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Rainfall and Flow Relationship")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ızgara(Izgara::yeni().alt(80))
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_yakınlaştırma(true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Çapraz),
        )
        .gösterge(
            Gösterge::yeni()
                .sol(10.0)
                .iç_boşluk(15.0)
                .veri(["Flow", "Rainfall"]),
        )
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().aralık(65.0, 85.0))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(65.0, 85.0))
        .x_ekseni(
            Eksen::kategori()
                .kenar_boşluğu(false)
                .çizgi(EksenÇizgisi::yeni().sıfır(EksenSıfırKipi::Kapalı))
                .veri(kategoriler),
        )
        .y_ekseni_ekle(Eksen::değer().ad("Flow(m³/s)"))
        .y_ekseni_ekle(
            Eksen::değer()
                .ad("Rainfall(mm)")
                .ad_konumu(EksenAdKonumu::Başlangıç)
                .çentik_hizala(true)
                .ters(true),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Flow")
                .alan_stili(AlanStili::default())
                .çizgi_stili(ÇizgiStili::yeni().kalınlık(1.0))
                .im_alanı(im_alanı(2_213.0, 2_453.0))
                .veri(akış),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Rainfall")
                .eksenler(0, 1)
                .alan_stili(AlanStili::default())
                .çizgi_stili(ÇizgiStili::yeni().kalınlık(1.0))
                .im_alanı(im_alanı(2_165.0, 2_405.0))
                .veri(yağış),
        ))
}

fn dynamic_data2(durum: &str) -> Result<GrafikSeçenekleri, String> {
    const GÜN_MS: f64 = 86_400_000.0;
    const SON_DURUM_TİKİ: usize = 20;
    const TİK_BAŞINA_NOKTA: usize = 5;

    let mut tohum = 0x5eed_1234;
    let mut an = cizelge::yardimci::takvim::takvimden_ana(cizelge::yardimci::takvim::TakvimAnı {
        yıl: 1997,
        ay: 10,
        gün: 3,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let mut değer = kanıt_rastgele(&mut tohum) * 1000.0;
    let mut yeni_nokta = || {
        an += GÜN_MS;
        değer += kanıt_rastgele(&mut tohum) * 21.0 - 10.0;
        VeriÖğesi::yeni([an, javascript_yuvarla(değer)])
    };
    let mut veri = (0..1000).map(|_| yeni_nokta()).collect::<Vec<_>>();

    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .başlık(
            Başlık::yeni()
                .metin("Dynamic Data & Time Axis")
                .iç_boşluk(15.0),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç_animasyonu(false)
                .bağlamlı_biçimleyici(|parametreler| {
                    let Some(parametre) = parametreler.first() else {
                        return String::new();
                    };
                    let Some(zaman) = parametre.değer.x() else {
                        return String::new();
                    };
                    let Some(değer) = parametre.değer.sayı() else {
                        return String::new();
                    };
                    let tarih = cizelge::yardimci::takvim::andan_takvime(zaman);
                    format!("{}/{}/{} : {değer}", tarih.gün, tarih.ay, tarih.yıl)
                }),
        )
        .x_ekseni(Eksen::zaman().bölme_çizgisi_göster(false))
        .y_ekseni(
            Eksen::değer()
                .sayısal_kenar_boşluğu(0.0, "100%")
                .bölme_çizgisi_göster(false),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Fake Data")
                .sembol_göster(false)
                .veri(veri.clone()),
        );

    if durum != "son" {
        return Ok(seçenekler);
    }

    // Resmî örneğin setInterval callback'ini yirmi kez yeniden oynat. Her
    // tikteki data-only setOption yaması, seri türü/adı/sembol ayarlarını
    // koruyan gerçek çalışma zamanı yolundan geçer.
    let mut çalışma = GrafikÇalışmaZamanı::yeni(
        ÖrnekBaşlatmaSeçenekleri {
            yerel: &İNGİLİZCE,
            ..ÖrnekBaşlatmaSeçenekleri::default()
        },
        seçenekler,
    )
    .map_err(|hata| hata.to_string())?;
    for _ in 0..SON_DURUM_TİKİ {
        veri.drain(..TİK_BAŞINA_NOKTA);
        veri.extend((0..TİK_BAŞINA_NOKTA).map(|_| yeni_nokta()));
        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().seri_verisi(SeriSeçici::Sıra(0), veri.iter().cloned()),
                SeçenekAyarlamaKipi::default(),
            )
            .map_err(|hata| hata.to_string())?;
    }
    çalışma.seçenekleri_al().map_err(|hata| hata.to_string())
}

/// Referans tarayıcısı 2024-01-01 03:00:00 (Europe/Istanbul) anına
/// sabitlenir. `toLocaleTimeString()` çıktısını aynı en-US biçiminde üretir.
fn dinamik_saat_etiketi(milisaniye: i64) -> String {
    let toplam_saniye = (3 * 60 * 60 + milisaniye.div_euclid(1000)).rem_euclid(24 * 60 * 60);
    let saat_24 = toplam_saniye / 3600;
    let dakika = toplam_saniye % 3600 / 60;
    let saniye = toplam_saniye % 60;
    let dönem = if saat_24 < 12 { "AM" } else { "PM" };
    let saat_12 = match saat_24 % 12 {
        0 => 12,
        saat => saat,
    };
    format!("{saat_12}:{dakika:02}:{saniye:02} {dönem}")
}

fn dynamic_data(durum: &str) -> Result<GrafikSeçenekleri, String> {
    const SON_DURUM_TİKİ: usize = 10;
    const TİK_MS: i64 = 2_100;

    let mut tohum = 0x5eed_1234;
    let mut kategoriler = (-9_i64..=0)
        .map(|sıra| dinamik_saat_etiketi(sıra * 2_000))
        .collect::<Vec<_>>();
    let mut sıra_kategorileri = (0..10).map(|sıra| sıra.to_string()).collect::<Vec<_>>();
    let mut sütun_verisi = (0..10)
        .map(|_| javascript_yuvarla(kanıt_rastgele(&mut tohum) * 1000.0))
        .collect::<Vec<_>>();
    let mut çizgi_verisi = (0..10)
        .map(|_| javascript_onda_bir(kanıt_rastgele(&mut tohum) * 10.0 + 5.0))
        .collect::<Vec<_>>();

    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .başlık(Başlık::yeni().metin("Dynamic Data").iç_boşluk(15.0))
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Çapraz)
                .imleç_etiketi_arkaplanı("#283b56"),
        )
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_görünümü(true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().göster(false).aralık(0.0, 100.0))
        .x_ekseni_ekle(
            Eksen::kategori()
                .kenar_boşluğu(true)
                .veri(kategoriler.iter().cloned()),
        )
        .x_ekseni_ekle(
            Eksen::kategori()
                .kenar_boşluğu(true)
                .veri(sıra_kategorileri.iter().cloned()),
        )
        .y_ekseni_ekle(
            Eksen::değer()
                .ad("Price")
                .ölçekli(true)
                .en_az(0.0)
                .en_çok(30.0)
                .sayısal_kenar_boşluğu(0.2, 0.2),
        )
        .y_ekseni_ekle(
            Eksen::değer()
                .ad("Order")
                .ölçekli(true)
                .en_az(0.0)
                .en_çok(1200.0)
                .sayısal_kenar_boşluğu(0.2, 0.2),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Dynamic Bar")
                .eksenler(1, 1)
                .veri(sütun_verisi.iter().copied()),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Dynamic Line")
                .veri(çizgi_verisi.iter().copied()),
        );

    if durum != "son" {
        return Ok(seçenekler);
    }

    // Resmî 2100 ms callback'i on kez yeniden oynatılır. Eksen ve seri
    // yamaları yalnız `data` yollarını değiştirir; ilk option'daki tür,
    // eksen bağı, ad ve görsel seçenekler aynen korunur.
    let mut çalışma = GrafikÇalışmaZamanı::yeni(
        ÖrnekBaşlatmaSeçenekleri {
            yerel: &İNGİLİZCE,
            ..ÖrnekBaşlatmaSeçenekleri::default()
        },
        seçenekler,
    )
    .map_err(|hata| hata.to_string())?;
    for tik in 0..SON_DURUM_TİKİ {
        sütun_verisi.remove(0);
        sütun_verisi.push(javascript_yuvarla(kanıt_rastgele(&mut tohum) * 1000.0));
        çizgi_verisi.remove(0);
        çizgi_verisi.push(javascript_onda_bir(kanıt_rastgele(&mut tohum) * 10.0 + 5.0));
        kategoriler.remove(0);
        kategoriler.push(dinamik_saat_etiketi((tik as i64 + 1) * TİK_MS));
        sıra_kategorileri.remove(0);
        sıra_kategorileri.push((11 + tik).to_string());

        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni()
                    .x_ekseni_verisi(0, kategoriler.iter().cloned())
                    .x_ekseni_verisi(1, sıra_kategorileri.iter().cloned())
                    .seri_verisi(SeriSeçici::Sıra(0), sütun_verisi.iter().copied())
                    .seri_verisi(SeriSeçici::Sıra(1), çizgi_verisi.iter().copied()),
                SeçenekAyarlamaKipi::default(),
            )
            .map_err(|hata| hata.to_string())?;
    }
    çalışma.seçenekleri_al().map_err(|hata| hata.to_string())
}

fn line_stack() -> GrafikSeçenekleri {
    let seri = |ad: &str, veri: [i32; 7]| ÇizgiSerisi::yeni().ad(ad).yığın("Total").veri(veri);
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Stacked Line").iç_boşluk(15.0))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(Gösterge::yeni().iç_boşluk(15.0).veri([
            "Email",
            "Union Ads",
            "Video Ads",
            "Direct",
            "Search Engine",
        ]))
        .ızgara(
            Izgara::yeni()
                .sol("3%")
                .sağ("4%")
                .alt("3%")
                .etiketi_kapsa(true),
        )
        .araç_kutusu(AraçKutusu::yeni().png_kaydet(true))
        .x_ekseni(
            Eksen::kategori()
                .kenar_boşluğu(false)
                .veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .y_ekseni(Eksen::değer())
        .seri(seri("Email", [120, 132, 101, 134, 90, 230, 210]))
        .seri(seri("Union Ads", [220, 182, 191, 234, 290, 330, 310]))
        .seri(seri("Video Ads", [150, 232, 201, 154, 190, 330, 410]))
        .seri(seri("Direct", [320, 332, 301, 334, 390, 330, 320]))
        .seri(seri(
            "Search Engine",
            [820, 932, 901, 934, 1290, 1330, 1320],
        ))
}

fn line_style() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer())
        .seri(
            ÇizgiSerisi::yeni()
                .sembol(Sembol::Üçgen)
                .sembol_boyutu(20.0)
                .çizgi_stili(
                    ÇizgiStili::yeni()
                        .renk(0x5470c6u32)
                        .kalınlık(4.0)
                        .tür(ÇizgiTürü::Kesikli),
                )
                .öğe_stili(
                    ÖğeStili::yeni()
                        .renk(0xffff00u32)
                        .kenarlık_rengi(0xee6666u32)
                        .kenarlık_kalınlığı(3.0),
                )
                .veri([120, 200, 150, 80, 70, 110, 130]),
        )
}

fn line_step() -> GrafikSeçenekleri {
    let seri = |ad: &str, basamak: Basamak, veri: [i32; 7]| {
        ÇizgiSerisi::yeni().ad(ad).basamak(basamak).veri(veri)
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Step Line").iç_boşluk(15.0))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(
            Gösterge::yeni()
                .iç_boşluk(15.0)
                .veri(["Step Start", "Step Middle", "Step End"]),
        )
        .ızgara(
            Izgara::yeni()
                .sol("3%")
                .sağ("4%")
                .alt("3%")
                .etiketi_kapsa(true),
        )
        .araç_kutusu(AraçKutusu::yeni().png_kaydet(true))
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer())
        .seri(seri(
            "Step Start",
            Basamak::Baş,
            [120, 132, 101, 134, 90, 230, 210],
        ))
        .seri(seri(
            "Step Middle",
            Basamak::Orta,
            [220, 282, 201, 234, 290, 430, 410],
        ))
        .seri(seri(
            "Step End",
            Basamak::Son,
            [450, 432, 401, 454, 590, 530, 510],
        ))
}

fn line_in_cartesian_coordinate_system() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::değer())
        .y_ekseni(Eksen::değer())
        .seri(ÇizgiSerisi::yeni().veri([[10.0, 40.0], [50.0, 100.0], [40.0, 20.0]]))
}

fn line_y_category() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(
            Gösterge::yeni()
                .iç_boşluk(15.0)
                .veri(["Altitude (km) vs. temperature (°C)"]),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(
            Izgara::yeni()
                .sol("3%")
                .sağ("4%")
                .alt("3%")
                .etiketi_kapsa(true),
        )
        .x_ekseni(Eksen::değer().etiket_biçimleyici("{value} °C"))
        .y_ekseni(
            Eksen::kategori()
                .çizgi(EksenÇizgisi::yeni().sıfır(EksenSıfırKipi::Kapalı))
                .etiket_biçimleyici("{value} km")
                .kenar_boşluğu(false)
                .veri(["0", "10", "20", "30", "40", "50", "60", "70", "80"]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Altitude (km) vs. temperature (°C)")
                .sembol(Sembol::Daire)
                .sembol_boyutu(10.0)
                .yumuşat(true)
                .çizgi_stili(
                    ÇizgiStili::yeni()
                        .kalınlık(3.0)
                        .gölge_rengi(Renk::kyma(0.0, 0.0, 0.0, 0.3))
                        .gölge_bulanıklığı(10.0)
                        .gölge_kayması(0.0, 8.0),
                )
                .veri([15.0, -50.0, -56.5, -46.5, -22.1, -2.5, -27.7, -55.7, -76.5]),
        )
}

fn line_log() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Log Axis")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().sol("left").iç_boşluk(15.0))
        .ızgara(
            Izgara::yeni()
                .sol("3%")
                .sağ("4%")
                .alt("3%")
                .etiketi_kapsa(true),
        )
        .x_ekseni(
            Eksen::kategori()
                .ad("x")
                .bölme_çizgisi_göster(false)
                .veri(["A", "B", "C", "D", "E", "F", "G", "H", "I"]),
        )
        .y_ekseni(Eksen::log().ad("y").ara_bölme_çizgisi_göster(true))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Log2")
                .veri([1, 3, 9, 27, 81, 247, 741, 2223, 6669]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Log3")
                .veri([1, 2, 4, 8, 16, 32, 64, 128, 256]),
        )
        .seri(ÇizgiSerisi::yeni().ad("Log1/2").veri([
            0.5,
            0.25,
            0.125,
            0.0625,
            0.03125,
            0.015625,
            0.0078125,
            0.00390625,
            0.001953125,
        ]))
}

fn line_polar() -> GrafikSeçenekleri {
    let veri = (0..=100)
        .map(|sıra| {
            let açı = f64::from(sıra) / 100.0 * 360.0;
            let yarıçap = 5.0 * (1.0 + açı.to_radians().sin());
            [yarıçap, açı]
        })
        .collect::<Vec<_>>();

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Two Value-Axes in Polar")
                .iç_boşluk(15.0),
        )
        .gösterge(Gösterge::yeni().iç_boşluk(15.0).veri(["line"]))
        .kutupsal(
            KutupsalKoordinat::yeni()
                .başlangıç_açısı(0.0)
                .açısal_eksen(Eksen::değer().bölme_sayısı(12)),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Çapraz),
        )
        .seri(ÇizgiSerisi::yeni().ad("line").kutupsal(true).veri(veri))
}

fn line_polar2() -> GrafikSeçenekleri {
    let veri = (0..=360)
        .map(|sıra| {
            let t = f64::from(sıra) / 180.0 * std::f64::consts::PI;
            [f64::sin(2.0 * t) * f64::cos(2.0 * t), f64::from(sıra)]
        })
        .collect::<Vec<_>>();

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Two Value-Axes in Polar")
                .iç_boşluk(15.0),
        )
        .gösterge(Gösterge::yeni().iç_boşluk(15.0).veri(["line"]))
        .kutupsal(
            KutupsalKoordinat::yeni()
                .merkez("50%", "54%")
                .başlangıç_açısı(0.0)
                .açısal_eksen(Eksen::değer().bölme_sayısı(12))
                .radyal_eksen(Eksen::değer().en_az(0.0)),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Çapraz),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("line")
                .kutupsal(true)
                .sembol_göster(false)
                .veri(veri),
        )
}

fn line_function() -> GrafikSeçenekleri {
    let işlev = |x: f64| {
        let x = x / 10.0;
        x.sin() * (x * 2.0 + 1.0).cos() * (x * 3.0 + 2.0).sin() * 50.0
    };
    let mut veri = Vec::with_capacity(4_001);
    let mut x = -200.0_f64;
    while x <= 200.0 {
        veri.push([x, işlev(x)]);
        x += 0.1;
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ızgara(Izgara::yeni().üst(40).sol(50).sağ(40).alt(50))
        .x_ekseni(
            Eksen::değer()
                .ad("x")
                .ara_çentik_göster(true)
                .ara_bölme_çizgisi_göster(true),
        )
        .y_ekseni(
            Eksen::değer()
                .ad("y")
                .en_az(-100.0)
                .en_çok(100.0)
                .ara_çentik_göster(true)
                .ara_bölme_çizgisi_göster(true),
        )
        .veri_yakınlaştırma(
            VeriYakınlaştırma::iç()
                .değer_aralığı(-20.0, 20.0)
                .süzme_kipi(YakınlaştırmaSüzmeKipi::Yok),
        )
        .veri_yakınlaştırma(
            VeriYakınlaştırma::iç()
                .y_eksen_sırası(0)
                .değer_aralığı(-20.0, 20.0)
                .süzme_kipi(YakınlaştırmaSüzmeKipi::Yok),
        )
        .seri(ÇizgiSerisi::yeni().sembol_göster(false).veri(veri))
}

fn bump_chart() -> GrafikSeçenekleri {
    let adlar = [
        "Orange",
        "Tomato",
        "Apple",
        "Sakana",
        "Banana",
        "Iwashi",
        "Snappy Fish",
        "Lemon",
        "Pasta",
    ];
    let yıllar = ["2001", "2002", "2003", "2004", "2005", "2006"];
    let mut tohum = 0x5eed_1234_u32;
    let mut sıralama = (1..=adlar.len() as i32).collect::<Vec<_>>();
    let mut seri_verileri = vec![Vec::<i32>::new(); adlar.len()];
    for _ in &yıllar {
        let mut kalan = sıralama.len();
        while kalan > 0 {
            let rastgele = (kanıt_rastgele(&mut tohum) * kalan as f64).floor() as usize;
            kalan -= 1;
            sıralama.swap(kalan, rastgele);
        }
        for (sıra, değer) in sıralama.iter().copied().enumerate() {
            if let Some(veri) = seri_verileri.get_mut(sıra) {
                veri.push(değer);
            }
        }
    }

    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Bump Chart (Ranking)").iç_boşluk(15.0))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .ızgara(Izgara::yeni().sol(30).sağ(110).alt(30).etiketi_kapsa(true))
        .araç_kutusu(AraçKutusu::yeni().png_kaydet(true))
        .x_ekseni(
            Eksen::kategori()
                .kenar_boşluğu(false)
                .bölme_çizgisi_göster(true)
                .etiket(
                    EksenEtiketi::yeni()
                        .boşluk(30.0)
                        .yazı(YazıStili::yeni().boyut(16.0)),
                )
                .veri(yıllar),
        )
        .y_ekseni(
            Eksen::değer()
                .en_az(1.0)
                .en_çok(adlar.len() as f64)
                .bölme_sayısı(adlar.len() - 1)
                .en_küçük_adım(1.0)
                .en_büyük_adım(1.0)
                .ters(true)
                .etiket(
                    EksenEtiketi::yeni()
                        .boşluk(30.0)
                        .biçimleyici("#{value}")
                        .yazı(YazıStili::yeni().boyut(16.0)),
                ),
        );
    for (sıra, ad) in adlar.iter().enumerate() {
        seçenekler = seçenekler.seri(
            ÇizgiSerisi::yeni()
                .ad(*ad)
                .sembol_boyutu(20.0)
                .yumuşat(true)
                .çizgi_stili(ÇizgiStili::yeni().kalınlık(4.0))
                .uç_etiketi(Etiket::yeni().göster(true).biçimleyici("{a}").uzaklık(20.0))
                .veri(seri_verileri.get(sıra).cloned().unwrap_or_default()),
        );
    }
    seçenekler
}

fn line_sections() -> GrafikSeçenekleri {
    let görsel = GörselEşleme::yeni().göster(false).boyut(0usize).parçalar([
        EşlemeParçası::aralık(None, true, Some(6.0), true, "green"),
        EşlemeParçası::aralık(Some(6.0), false, Some(8.0), true, "red"),
        EşlemeParçası::aralık(Some(8.0), false, Some(14.0), true, "green"),
        EşlemeParçası::aralık(Some(14.0), false, Some(17.0), true, "red"),
        EşlemeParçası::aralık(Some(17.0), false, None, true, "green"),
    ]);
    let alanlar = İmAlanı::yeni()
        .x_aralığı("Morning Peak", 6.0, 8.0)
        .x_aralığı("Evening Peak", 14.0, 17.0)
        .stil(ÖğeStili::yeni().renk("rgba(255, 173, 177, 0.4)"));

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Distribution of Electricity")
                .alt_metin("Fake Data")
                .iç_boşluk(15.0),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Çapraz),
        )
        .araç_kutusu(AraçKutusu::yeni().png_kaydet(true))
        .x_ekseni(Eksen::kategori().kenar_boşluğu(false).veri([
            "00:00", "01:15", "02:30", "03:45", "05:00", "06:15", "07:30", "08:45", "10:00",
            "11:15", "12:30", "13:45", "15:00", "16:15", "17:30", "18:45", "20:00", "21:15",
            "22:30", "23:45",
        ]))
        .y_ekseni(Eksen::değer().etiket_biçimleyici("{value} W"))
        .görsel_eşleme(görsel)
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Electricity")
                .yumuşat(true)
                .im_alanı(alanlar)
                .veri([
                    300, 280, 250, 260, 270, 300, 550, 500, 400, 390, 380, 390, 400, 500, 600, 750,
                    800, 700, 600, 400,
                ]),
        )
}

fn area_pieces() -> GrafikSeçenekleri {
    let im_çizgileri = İmÇizgisi::yeni()
        .dikey(İmDeğeri::Değer(1.0))
        .dikey(İmDeğeri::Değer(3.0))
        .dikey(İmDeğeri::Değer(5.0))
        .dikey(İmDeğeri::Değer(7.0))
        .uç_simgeleri(İmÇizgisiUçSimgesi::Yok, İmÇizgisiUçSimgesi::Yok)
        .etiket(Etiket::yeni().göster(false));
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().kenar_boşluğu(false).veri([
            "2019-10-10",
            "2019-10-11",
            "2019-10-12",
            "2019-10-13",
            "2019-10-14",
            "2019-10-15",
            "2019-10-16",
            "2019-10-17",
            "2019-10-18",
        ]))
        .y_ekseni(Eksen::değer().sayısal_kenar_boşluğu(0.0, "30%"))
        .görsel_eşleme(
            GörselEşleme::yeni()
                .göster(false)
                .boyut(0usize)
                .seri_sırası(0)
                .parçalar([
                    EşlemeParçası::aralık(
                        Some(1.0),
                        false,
                        Some(3.0),
                        false,
                        "rgba(0, 0, 180, 0.4)",
                    ),
                    EşlemeParçası::aralık(
                        Some(5.0),
                        false,
                        Some(7.0),
                        false,
                        "rgba(0, 0, 180, 0.4)",
                    ),
                ]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .yumuşaklık(0.6)
                .sembol(Sembol::Yok)
                .çizgi_stili(ÇizgiStili::yeni().renk("#5470C6").kalınlık(5.0))
                .alan_stili(AlanStili::default())
                .im_çizgisi(im_çizgileri)
                .veri([200, 560, 750, 580, 250, 300, 450, 300, 100]),
        )
}

fn line_gradient() -> GrafikSeçenekleri {
    let tarihler = [
        "2000-06-05",
        "2000-06-06",
        "2000-06-07",
        "2000-06-08",
        "2000-06-09",
        "2000-06-10",
        "2000-06-11",
        "2000-06-12",
        "2000-06-13",
        "2000-06-14",
        "2000-06-15",
        "2000-06-16",
        "2000-06-17",
        "2000-06-18",
        "2000-06-19",
        "2000-06-20",
        "2000-06-21",
        "2000-06-22",
        "2000-06-23",
        "2000-06-24",
        "2000-06-25",
        "2000-06-26",
        "2000-06-27",
        "2000-06-28",
        "2000-06-29",
        "2000-06-30",
        "2000-07-01",
        "2000-07-02",
        "2000-07-03",
        "2000-07-04",
        "2000-07-05",
        "2000-07-06",
        "2000-07-07",
        "2000-07-08",
        "2000-07-09",
        "2000-07-10",
        "2000-07-11",
        "2000-07-12",
        "2000-07-13",
        "2000-07-14",
        "2000-07-15",
        "2000-07-16",
        "2000-07-17",
        "2000-07-18",
        "2000-07-19",
        "2000-07-20",
        "2000-07-21",
        "2000-07-22",
        "2000-07-23",
        "2000-07-24",
    ];
    let değerler = [
        116, 129, 135, 86, 73, 85, 73, 68, 92, 130, 245, 139, 115, 111, 309, 206, 137, 128, 85, 94,
        71, 106, 84, 93, 85, 73, 83, 125, 107, 82, 44, 72, 106, 107, 66, 91, 92, 113, 107, 131,
        111, 64, 69, 88, 77, 83, 111, 57, 55, 60,
    ];
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .görsel_eşlemeler([
            GörselEşleme::yeni()
                .göster(false)
                .seri_sırası(0)
                .en_az(0.0)
                .en_çok(400.0),
            GörselEşleme::yeni()
                .göster(false)
                .seri_sırası(1)
                .boyut(0usize)
                .en_az(0.0)
                .en_çok((tarihler.len() - 1) as f64),
        ])
        .başlık(
            Başlık::yeni()
                .metin("Gradient along the y axis")
                .iç_boşluk(15.0),
        )
        .başlık_ekle(
            Başlık::yeni()
                .metin("Gradient along the x axis")
                .üst("55%")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara_ekle(Izgara::yeni().alt("60%"))
        .ızgara_ekle(Izgara::yeni().üst("60%"))
        .x_ekseni_ekle(Eksen::kategori().veri(tarihler))
        .x_ekseni_ekle(Eksen::kategori().ızgara_sırası(1).veri(tarihler))
        .y_ekseni_ekle(Eksen::değer())
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(1))
        .seri(ÇizgiSerisi::yeni().sembol_göster(false).veri(değerler))
        .seri(
            ÇizgiSerisi::yeni()
                .eksenler(1, 1)
                .sembol_göster(false)
                .veri(değerler),
        )
}

fn line_aqi() -> Result<GrafikSeçenekleri, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/data/asset/data/aqi-beijing.json");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let veri: Vec<(String, f64)> = serde_json::from_str(&kaynak)
        .map_err(|hata| format!("{} ayrıştırılamadı: {hata}", dosya.display()))?;
    let (tarihler, değerler): (Vec<_>, Vec<_>) = veri.into_iter().unzip();

    let görsel = GörselEşleme::yeni()
        .üst(50)
        .sağ(10)
        .aralık_dışı_renk("#999")
        .parçalar([
            EşlemeParçası::aralık(Some(0.0), false, Some(50.0), true, "#93CE07").etiket("0 - 50"),
            EşlemeParçası::aralık(Some(50.0), false, Some(100.0), true, "#FBDB0F")
                .etiket("50 - 100"),
            EşlemeParçası::aralık(Some(100.0), false, Some(150.0), true, "#FC7D02")
                .etiket("100 - 150"),
            EşlemeParçası::aralık(Some(150.0), false, Some(200.0), true, "#FD0100")
                .etiket("150 - 200"),
            EşlemeParçası::aralık(Some(200.0), false, Some(300.0), true, "#AA069F")
                .etiket("200 - 300"),
            EşlemeParçası::gt(300.0, "#AC3B2A").etiket("> 300"),
        ]);
    let im_çizgileri = [50.0, 100.0, 150.0, 200.0, 300.0]
        .into_iter()
        .fold(İmÇizgisi::yeni(), |çizgiler, değer| {
            çizgiler.yatay(İmDeğeri::Değer(değer))
        })
        .stil(
            ÇizgiStili::yeni()
                .renk("#333")
                .kalınlık(1.0)
                .tür(ÇizgiTürü::Kesikli),
        );

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Beijing AQI")
                .sol("1%")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(Izgara::yeni().sol("5%").sağ("15%").alt("10%"))
        .x_ekseni(Eksen::kategori().veri(tarihler))
        .y_ekseni(Eksen::değer())
        .araç_kutusu(
            AraçKutusu::yeni()
                .sağ(10)
                .veri_yakınlaştırma(true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().başlangıç_değeri("2014-06-01"))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç())
        .görsel_eşleme(görsel)
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Beijing AQI")
                .im_çizgisi(im_çizgileri)
                .veri(değerler),
        ))
}

fn confidence_band() -> Result<GrafikSeçenekleri, String> {
    #[derive(Deserialize)]
    struct GüvenAralığıÖğesi {
        l: f64,
        u: f64,
        date: String,
        value: f64,
    }

    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/data/asset/data/confidence-band.json");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let veri: Vec<GüvenAralığıÖğesi> = serde_json::from_str(&kaynak)
        .map_err(|hata| format!("{} ayrıştırılamadı: {hata}", dosya.display()))?;
    let taban = -veri
        .iter()
        .map(|öğe| öğe.l)
        .fold(f64::INFINITY, f64::min)
        .floor();
    let tarihler = veri.iter().map(|öğe| öğe.date.clone()).collect::<Vec<_>>();
    let alt = veri.iter().map(|öğe| öğe.l + taban).collect::<Vec<_>>();
    let aralık = veri.iter().map(|öğe| öğe.u - öğe.l).collect::<Vec<_>>();
    let orta = veri.iter().map(|öğe| öğe.value + taban).collect::<Vec<_>>();

    let tarih_biçimleyici = Biçimleyici::İşlev(Arc::new(|değer, metin| {
        if değer.round() == 0.0 {
            return metin.to_owned();
        }
        let parçalar = metin.split('-').collect::<Vec<_>>();
        match (parçalar.get(1), parçalar.get(2)) {
            (Some(ay), Some(gün)) => format!(
                "{}-{}",
                ay.trim_start_matches('0'),
                gün.trim_start_matches('0')
            ),
            _ => metin.to_owned(),
        }
    }));
    let yüzde_biçimleyici = Biçimleyici::İşlev(Arc::new(move |değer, _| {
        format!("{:.0}%", (değer - taban) * 100.0)
    }));

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Confidence Band")
                .alt_metin("Example in MetricsGraphics.js")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Çapraz),
        )
        .ızgara(
            Izgara::yeni()
                .sol("3%")
                .sağ("4%")
                .alt("3%")
                .etiketi_kapsa(true),
        )
        .x_ekseni(
            Eksen::kategori()
                .kenar_boşluğu(false)
                .etiket_biçimleyici(tarih_biçimleyici)
                .veri(tarihler),
        )
        .y_ekseni(
            Eksen::değer()
                .bölme_sayısı(3)
                .etiket_biçimleyici(yüzde_biçimleyici),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("L")
                .çizgi_stili(ÇizgiStili::yeni().opaklık(0.0))
                .yığın("confidence-band")
                .sembol(Sembol::Yok)
                .veri(alt),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("U")
                .çizgi_stili(ÇizgiStili::yeni().opaklık(0.0))
                .alan_stili(AlanStili::yeni().renk("#ccc"))
                .yığın("confidence-band")
                .sembol(Sembol::Yok)
                .veri(aralık),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .öğe_stili(ÖğeStili::yeni().renk("#333"))
                .sembol_göster(false)
                .veri(orta),
        ))
}

fn line_race() -> Result<GrafikSeçenekleri, String> {
    let ülkeler = [
        "Finland",
        "France",
        "Germany",
        "Iceland",
        "Norway",
        "Poland",
        "Russia",
        "United Kingdom",
    ];
    let ülke_verisi = |ad: &str| {
        VeriKümesiTanımı::kaynaktan_süz(
            0,
            SüzmeKoşulu::Ve(vec![
                SüzmeKoşulu::Karşılaştır {
                    boyut: BoyutSeçici::ad("Year"),
                    işlem: Karşılaştırmaİşlemi::BüyükEşit,
                    değer: 1950.into(),
                },
                SüzmeKoşulu::Karşılaştır {
                    boyut: BoyutSeçici::ad("Country"),
                    işlem: Karşılaştırmaİşlemi::Eşit,
                    değer: ad.into(),
                },
            ]),
        )
    };
    let mut veri_kümeleri = vec![VeriKümesiTanımı::kaynak(yaşam_beklentisi_verisi()?)];
    veri_kümeleri.extend(ülkeler.iter().map(|ülke| ülke_verisi(ülke)));

    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Income of Germany and France since 1950")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .veri_kümeleri(veri_kümeleri)
        .ızgara(Izgara::yeni().sağ(140))
        .x_ekseni(Eksen::kategori())
        .y_ekseni(Eksen::değer().ad("Income"));
    for (sıra, ülke) in ülkeler.into_iter().enumerate() {
        let etiket_ülkesi = ülke.to_owned();
        seçenekler = seçenekler.seri(
            ÇizgiSerisi::yeni()
                .ad(ülke)
                .veri_kümesi_sırası(sıra + 1)
                .eşle("Year", "Income")
                .sembol_göster(false)
                .etiket_örtüşmesini_dikey_kaydır(true)
                .uç_etiketi(Etiket::yeni().göster(true).uzaklık(8.0).biçimleyici(
                    Biçimleyici::İşlev(Arc::new(move |değer, _| {
                        format!("{etiket_ülkesi}: {değer:.0}")
                    })),
                )),
        );
    }
    Ok(seçenekler)
}

/// Sabitlenmiş `echarts-examples` TypeScript kaynağındaki yalın dizi
/// değişmezini okur. Bu örneğin 3.079 öğelik üç dizisini ikinci bir kopya
/// halinde elle tutmak yerine, resmi kaynağın kendisini tek veri otoritesi
/// yapar; seçenek modeli ve bütün boyama yine cizelge çekirdeğindedir.
fn resmi_javascript_dizisi<T: DeserializeOwned>(
    kaynak: &str, belirteç: &str
) -> Result<T, String> {
    let belirteç_başı = kaynak
        .find(belirteç)
        .ok_or_else(|| format!("resmi kaynakta `{belirteç}` bulunamadı"))?;
    let dizi_başı = kaynak[belirteç_başı..]
        .find('[')
        .map(|sıra| belirteç_başı + sıra)
        .ok_or_else(|| format!("`{belirteç}` sonrasında dizi bulunamadı"))?;
    let mut derinlik = 0usize;
    let mut tırnak = None;
    let mut kaçış = false;
    let mut dizi_sonu = None;
    for (göreli, karakter) in kaynak[dizi_başı..].char_indices() {
        if let Some(açık_tırnak) = tırnak {
            if kaçış {
                kaçış = false;
            } else if karakter == '\\' {
                kaçış = true;
            } else if karakter == açık_tırnak {
                tırnak = None;
            }
            continue;
        }
        match karakter {
            '\'' | '"' => tırnak = Some(karakter),
            '[' => derinlik += 1,
            ']' => {
                derinlik = derinlik.saturating_sub(1);
                if derinlik == 0 {
                    dizi_sonu = Some(dizi_başı + göreli + karakter.len_utf8());
                    break;
                }
            }
            _ => {}
        }
    }
    let dizi_sonu = dizi_sonu.ok_or_else(|| format!("`{belirteç}` dizisi kapanmıyor"))?;
    let json = kaynak[dizi_başı..dizi_sonu].replace('\'', "\"");
    serde_json::from_str(&json)
        .map_err(|hata| format!("`{belirteç}` dizisi ayrıştırılamadı: {hata}"))
}

fn grid_multiple() -> Result<GrafikSeçenekleri, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/examples/ts/grid-multiple.ts");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let mut zaman: Vec<String> = resmi_javascript_dizisi(&kaynak, "let timeData")?;
    // Resmî örnek dizi tanımından hemen sonra aynı dönüşümü uygular.
    for etiket in &mut zaman {
        if let Some(kısaltılmış) = etiket.strip_prefix("2009/") {
            *etiket = kısaltılmış.to_owned();
        }
    }
    let buharlaşma: Vec<f64> = resmi_javascript_dizisi(&kaynak, "name: 'Evaporation'")?;
    let yağış: Vec<f64> = resmi_javascript_dizisi(&kaynak, "name: 'Rainfall'")?;
    if zaman.len() != buharlaşma.len() || zaman.len() != yağış.len() {
        return Err(format!(
            "grid-multiple resmi dizileri farklı uzunlukta: zaman={}, buharlaşma={}, yağış={}",
            zaman.len(),
            buharlaşma.len(),
            yağış.len()
        ));
    }

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Rainfall vs Evaporation")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(
            Gösterge::yeni()
                .sol(10.0)
                // Kanıt ön işlemcisi resmî örneğin açık olmayan `padding`
                // değerini bütün bileşenlerde 15 px'e sabitler.
                .iç_boşluk(15.0)
                .veri(["Evaporation", "Rainfall"]),
        )
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_yakınlaştırma(true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .veri_yakınlaştırma(
            VeriYakınlaştırma::sürgü()
                .x_eksenleri([0, 1])
                .aralık(30.0, 70.0),
        )
        .veri_yakınlaştırma(
            VeriYakınlaştırma::iç()
                .x_eksenleri([0, 1])
                .aralık(30.0, 70.0),
        )
        .ızgara_ekle(Izgara::yeni().sol(60).sağ(50).yükseklik("35%"))
        .ızgara_ekle(Izgara::yeni().sol(60).sağ(50).üst("55%").yükseklik("35%"))
        .x_ekseni_ekle(Eksen::kategori().kenar_boşluğu(false).veri(zaman.clone()))
        .x_ekseni_ekle(
            Eksen::kategori()
                .ızgara_sırası(1)
                .kenar_boşluğu(false)
                .konum(EksenKonumu::Üst)
                .veri(zaman),
        )
        .y_ekseni_ekle(Eksen::değer().ad("Evaporation(m³/s)").en_çok(500.0))
        .y_ekseni_ekle(
            Eksen::değer()
                .ızgara_sırası(1)
                .ad("Rainfall(mm)")
                .ters(true),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Evaporation")
                .sembol_boyutu(8.0)
                .veri(buharlaşma),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Rainfall")
                .eksenler(1, 1)
                .sembol_boyutu(8.0)
                .veri(yağış),
        ))
}

fn saat_dakika(zaman: f64) -> String {
    let an = cizelge::yardimci::takvim::andan_takvime(zaman);
    format!("{:02}:{:02}", an.saat, an.dakika)
}

fn iki_ondalık(değer: f64) -> f64 {
    // İki intraday üreticisi her adımda yalnız tam sent (veya onda bir)
    // eklediğinden `Number#toFixed(2)` burada tam sayı sent uzayına iner.
    (değer * 100.0).round() / 100.0
}

fn intraday_breaks_1() -> GrafikSeçenekleri {
    const DAKİKA: f64 = 60_000.0;
    const GÜN: f64 = 86_400_000.0;
    const BAŞLANGIÇ: f64 = 1_712_655_000_000.0; // 2024-04-09T09:30:00Z
    const BİTİŞ: f64 = 1_712_966_399_000.0; // 2024-04-12T23:59:59Z
    const AÇILIŞ_DAKİKASI: f64 = 9.0 * 60.0 + 30.0;
    const KAPANIŞ_DAKİKASI: f64 = 16.0 * 60.0;

    let mut tohum = 0x5eed_1234_u32;
    let mut zaman = BAŞLANGIÇ;
    let mut gün_başı = BAŞLANGIÇ - AÇILIŞ_DAKİKASI * DAKİKA;
    let mut kapanış = gün_başı + KAPANIŞ_DAKİKASI * DAKİKA;
    let mut değer = 1669.0_f64;
    let mut kırılma_sıçraması = false;
    let mut veri = Vec::new();
    let mut kırılmalar = Vec::new();
    let mut en_az = f64::INFINITY;

    while zaman <= BİTİŞ {
        let rastgele = kanıt_rastgele(&mut tohum);
        let ham = (rastgele - 0.5 * (değer / 1000.0).sin()) * 20.0 * 100.0;
        let fark = if kırılma_sıçraması {
            kırılma_sıçraması = false;
            ham.floor() / 10.0
        } else {
            ham.floor() / 100.0
        };
        değer = iki_ondalık(değer + fark);
        en_az = en_az.min(değer);
        veri.push(VeriÖğesi::yeni([zaman, değer]));
        zaman += DAKİKA;

        if zaman > kapanış {
            // Resmî örnek NaN satırını çizgi segmentini açıkça kesmek için
            // ekler; x değeri son kapsamı 16:01'e kadar taşır.
            veri.push(VeriÖğesi::yeni([zaman, f64::NAN]));
            let kırılma_başı = kapanış;
            gün_başı += GÜN;
            zaman = gün_başı + AÇILIŞ_DAKİKASI * DAKİKA;
            kapanış = gün_başı + KAPANIŞ_DAKİKASI * DAKİKA;
            kırılma_sıçraması = true;
            kırılmalar.push(EksenKırılması::yeni(kırılma_başı, zaman).boşluk("1%"));
        }
    }

    let etiket = EksenEtiketi::yeni()
        .en_az_etiketini_göster(true)
        .en_çok_etiketini_göster(true)
        .bağlamlı_biçimleyici(|değer, _, bağlam| {
            let saat = saat_dakika(değer);
            if bağlam.kırılma.is_some() {
                let gün = cizelge::yardimci::takvim::andan_takvime(değer).gün;
                format!("{saat}\n{gün:02}d")
            } else {
                saat
            }
        });
    let kırılma_alanı = EksenKırılmaAlanı::yeni()
        .zikzak_genliği(0.0)
        .tıklayınca_genişlet(false)
        .kenarlık_göster(false)
        .opaklık(0.0);

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Intraday Chart with Breaks (Multiple Days)")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        // Resmî `grid.outerBounds` çözümünün ürettiği kesin iç dikdörtgen.
        .ızgara(
            Izgara::yeni()
                .sol(105)
                .sağ(70)
                .üst(111)
                .yükseklik(224.5_f32),
        )
        .x_ekseni(
            Eksen::zaman()
                .sayısal_kenar_boşluğu(0.0, 0.0)
                .etiket(etiket)
                .kırılmalar(kırılmalar)
                .kırılma_alanı(kırılma_alanı),
        )
        .y_ekseni(Eksen::değer().en_az(en_az))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç())
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().üst("73%"))
        .seri(
            ÇizgiSerisi::yeni()
                .sembol(Sembol::Yok)
                .alan_stili(AlanStili::default())
                .veri(veri),
        )
}

fn intraday_breaks_2() -> GrafikSeçenekleri {
    const DAKİKA: f64 = 60_000.0;
    const BAŞLANGIÇ: f64 = 1_712_655_000_000.0; // 2024-04-09T09:30:00Z
    const BİTİŞ: f64 = 1_712_674_800_000.0; // 2024-04-09T15:00:00Z
    const KIRILMA_BAŞI: f64 = 1_712_662_200_000.0; // 11:30
    const KIRILMA_SONU: f64 = 1_712_667_600_000.0; // 13:00

    let mut tohum = 0x5eed_1234_u32;
    let mut zaman = BAŞLANGIÇ;
    let mut değer = 1669.0_f64;
    let mut veri = Vec::new();
    let mut en_az = f64::INFINITY;
    while zaman <= BİTİŞ {
        if zaman <= KIRILMA_BAŞI || zaman >= KIRILMA_SONU {
            let ham = (kanıt_rastgele(&mut tohum) - 0.5 * (değer / 1000.0).sin()) * 20.0 * 100.0;
            değer += ham.floor() / 100.0;
            değer = iki_ondalık(değer);
            en_az = en_az.min(değer);
            veri.push(VeriÖğesi::yeni([zaman, değer]));
        }
        zaman += DAKİKA;
    }

    let etiket = EksenEtiketi::yeni()
        .en_az_etiketini_göster(true)
        .en_çok_etiketini_göster(true)
        .bağlamlı_biçimleyici(|değer, _, bağlam| match bağlam.kırılma {
            Some(kırılma) if kırılma.tür == EksenKırılmaUcu::Başlangıç => format!(
                "{}/{}",
                saat_dakika(kırılma.başlangıç),
                saat_dakika(kırılma.bitiş)
            ),
            Some(_) => String::new(),
            None => saat_dakika(değer),
        });

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Intraday Chart with Breaks (Single Day)")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(Izgara::yeni().sol(105).sağ(70).üst(65).yükseklik(380))
        .x_ekseni(
            Eksen::zaman()
                .sayısal_kenar_boşluğu(0.0, 0.0)
                .etiket(etiket)
                .kırılma(EksenKırılması::yeni(KIRILMA_BAŞI, KIRILMA_SONU))
                .kırılma_alanı(
                    EksenKırılmaAlanı::yeni()
                        .zikzak_genliği(0.0)
                        .tıklayınca_genişlet(false),
                )
                .kırılma_etiketi_örtüşmesini_taşı(false),
        )
        .y_ekseni(Eksen::değer().en_az(en_az))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç())
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü())
        .seri(ÇizgiSerisi::yeni().sembol(Sembol::Yok).veri(veri))
}

fn area_stack() -> GrafikSeçenekleri {
    let seri = |ad: &str, veri: [i32; 7]| {
        ÇizgiSerisi::yeni()
            .ad(ad)
            .yığın("Total")
            .alan_stili(AlanStili::default())
            .veri(veri)
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Stacked Area Chart").iç_boşluk(15.0))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(Gösterge::yeni().iç_boşluk(15.0).veri([
            "Email",
            "Union Ads",
            "Video Ads",
            "Direct",
            "Search Engine",
        ]))
        .araç_kutusu(AraçKutusu::yeni().png_kaydet(true))
        .x_ekseni(
            Eksen::kategori()
                .kenar_boşluğu(false)
                .veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .y_ekseni(Eksen::değer())
        .seri(seri("Email", [120, 132, 101, 134, 90, 230, 210]))
        .seri(seri("Union Ads", [220, 182, 191, 234, 290, 330, 310]))
        .seri(seri("Video Ads", [150, 232, 201, 154, 190, 330, 410]))
        .seri(seri("Direct", [320, 332, 301, 334, 390, 330, 320]))
        .seri(
            seri("Search Engine", [820, 932, 901, 934, 1290, 1330, 1320])
                .etiket(Etiket::yeni().göster(true).konum(EtiketKonumu::Üst)),
        )
}

fn area_stack_gradient() -> GrafikSeçenekleri {
    let alan = |üst: u32, alt: u32| {
        AlanStili::yeni().opaklık(0.8).renk(Dolgu::doğrusal(
            0.0,
            0.0,
            0.0,
            1.0,
            vec![RenkDurağı::yeni(0.0, üst), RenkDurağı::yeni(1.0, alt)],
        ))
    };
    let seri = |ad: &str, üst: u32, alt: u32, veri: [i32; 7]| {
        ÇizgiSerisi::yeni()
            .ad(ad)
            .yığın("Total")
            .yumuşat(true)
            .çizgi_stili(ÇizgiStili::yeni().kalınlık(0.0))
            .sembol_göster(false)
            .alan_stili(alan(üst, alt))
            .veri(veri)
    };

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .palet([
            0x80ffa5u32,
            0x00ddffu32,
            0x37a2ffu32,
            0xff0087u32,
            0xffbf00u32,
        ])
        .başlık(
            Başlık::yeni()
                .metin("Gradient Stacked Area Chart")
                .iç_boşluk(15.0),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Çapraz),
        )
        .gösterge(
            Gösterge::yeni()
                .iç_boşluk(15.0)
                .veri(["Line 1", "Line 2", "Line 3", "Line 4", "Line 5"]),
        )
        .araç_kutusu(AraçKutusu::yeni().png_kaydet(true))
        .x_ekseni(
            Eksen::kategori()
                .kenar_boşluğu(false)
                .veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .y_ekseni(Eksen::değer())
        .seri(seri(
            "Line 1",
            0x80ffa5,
            0x01bfec,
            [140, 232, 101, 264, 90, 340, 250],
        ))
        .seri(seri(
            "Line 2",
            0x00ddff,
            0x4d77ff,
            [120, 282, 111, 234, 220, 340, 310],
        ))
        .seri(seri(
            "Line 3",
            0x37a2ff,
            0x7415db,
            [320, 132, 201, 334, 190, 130, 220],
        ))
        .seri(seri(
            "Line 4",
            0xff0087,
            0x87009d,
            [220, 402, 231, 134, 190, 230, 120],
        ))
        .seri(
            seri(
                "Line 5",
                0xffbf00,
                0xe03e4c,
                [220, 302, 181, 234, 210, 290, 150],
            )
            .etiket(Etiket::yeni().göster(true).konum(EtiketKonumu::Üst)),
        )
}

fn bar_background() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .arka_plan_göster(true)
                .veri([120, 200, 150, 80, 70, 110, 130]),
        )
}

fn bar_tick_align() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .ızgara(
            Izgara::yeni()
                .sol("3%")
                .sağ("4%")
                .alt("3%")
                .etiketi_kapsa(true),
        )
        .x_ekseni(
            Eksen::kategori()
                .çentik(EksenÇentiği {
                    etiketle_hizala: true,
                    ..Default::default()
                })
                .veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("Direct")
                .genişlik("60%")
                .veri([10, 52, 200, 334, 390, 330, 220]),
        )
}

fn bar_data_color() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer())
        .seri(SütunSerisi::yeni().veri([
            VeriÖğesi::yeni(120),
            VeriÖğesi::yeni(200).stil(ÖğeStili::yeni().renk(0x505372u32)),
            VeriÖğesi::yeni(150),
            VeriÖğesi::yeni(80),
            VeriÖğesi::yeni(70),
            VeriÖğesi::yeni(110),
            VeriÖğesi::yeni(130),
        ]))
}

fn bar_stack_border_radius() -> GrafikSeçenekleri {
    let veri = |değerler: [Option<i32>; 7], tepeler: [bool; 7]| {
        değerler
            .into_iter()
            .zip(tepeler)
            .map(|(değer, tepe)| {
                VeriÖğesi::yeni(değer.map(f64::from).unwrap_or(f64::NAN)).stil(
                    ÖğeStili::yeni().kenarlık_yarıçapı(if tepe {
                        [20.0, 20.0, 0.0, 0.0]
                    } else {
                        [0.0; 4]
                    }),
                )
            })
            .collect::<Vec<_>>()
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer())
        .seri(SütunSerisi::yeni().ad("a").yığın("a").veri(veri(
            [
                Some(120),
                Some(200),
                Some(150),
                Some(80),
                Some(70),
                Some(110),
                Some(130),
            ],
            [false, false, false, false, false, true, true],
        )))
        .seri(SütunSerisi::yeni().ad("b").yığın("a").veri(veri(
            [Some(10), Some(46), Some(64), None, Some(0), None, Some(0)],
            [false, true, true, false, false, false, false],
        )))
        .seri(SütunSerisi::yeni().ad("c").yığın("a").veri(veri(
            [Some(30), None, Some(0), Some(20), Some(10), None, Some(0)],
            [true, false, false, true, true, false, false],
        )))
        .seri(SütunSerisi::yeni().ad("d").yığın("b").veri(veri(
            [Some(30), None, Some(0), Some(20), Some(10), None, Some(0)],
            [false, false, false, true, true, false, false],
        )))
        .seri(SütunSerisi::yeni().ad("e").yığın("b").veri(veri(
            [
                Some(10),
                Some(20),
                Some(150),
                Some(0),
                None,
                Some(50),
                Some(10),
            ],
            [true, true, true, false, false, true, true],
        )))
}

fn bar_y_category() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("World Population").iç_boşluk(15.0))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .x_ekseni(Eksen::değer())
        .y_ekseni(Eksen::kategori().veri(["Brazil", "Indonesia", "USA", "India", "China", "World"]))
        .seri(
            SütunSerisi::yeni()
                .ad("2011")
                .veri([18203, 23489, 29034, 104970, 131744, 630230]),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("2012")
                .veri([19325, 23438, 31000, 121594, 134141, 681807]),
        )
}

fn bar_y_category_stack() -> GrafikSeçenekleri {
    let seri = |ad: &str, veri: [i32; 7]| {
        SütunSerisi::yeni()
            .ad(ad)
            .yığın("total")
            .etiket(Etiket::yeni().göster(true))
            .veri(veri)
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Gölge),
        )
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .x_ekseni(Eksen::değer())
        .y_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .seri(seri("Direct", [320, 302, 301, 334, 390, 330, 320]))
        .seri(seri("Mail Ad", [120, 132, 101, 134, 90, 230, 210]))
        .seri(seri("Affiliate Ad", [220, 182, 191, 234, 290, 330, 310]))
        .seri(seri("Video Ad", [150, 212, 201, 154, 190, 330, 410]))
        .seri(seri(
            "Search Engine",
            [820, 832, 901, 934, 1290, 1330, 1320],
        ))
}

fn bar_negative2() -> GrafikSeçenekleri {
    let sağ_etiket = || EtiketYaması::yeni().konum(EtiketKonumu::Sağ);
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Bar Chart with Negative Value")
                .iç_boşluk(15.0),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Gölge),
        )
        .ızgara(Izgara::yeni().üst(80).alt(30))
        .x_ekseni(
            Eksen::değer()
                .konum(EksenKonumu::Üst)
                .bölme_çizgisi(BölmeÇizgisi {
                    tür: ÇizgiTürü::Kesikli,
                    ..BölmeÇizgisi::default()
                }),
        )
        .y_ekseni(
            Eksen::kategori()
                .veri([
                    "ten", "nine", "eight", "seven", "six", "five", "four", "three", "two", "one",
                ])
                .çizgi(EksenÇizgisi {
                    göster: Some(false),
                    ..EksenÇizgisi::default()
                })
                .etiket(EksenEtiketi {
                    göster: false,
                    ..EksenEtiketi::default()
                })
                .çentik(EksenÇentiği {
                    göster: Some(false),
                    ..EksenÇentiği::default()
                })
                .bölme_çizgisi_göster(false),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Cost")
                .yığın("Total")
                .etiket(Etiket::yeni().göster(true).biçimleyici("{b}"))
                .veri([
                    VeriÖğesi::yeni(-0.07).etiket(sağ_etiket()),
                    VeriÖğesi::yeni(-0.09).etiket(sağ_etiket()),
                    VeriÖğesi::yeni(0.2),
                    VeriÖğesi::yeni(0.44),
                    VeriÖğesi::yeni(-0.23).etiket(sağ_etiket()),
                    VeriÖğesi::yeni(0.08),
                    VeriÖğesi::yeni(-0.17).etiket(sağ_etiket()),
                    VeriÖğesi::yeni(0.47),
                    VeriÖğesi::yeni(-0.36).etiket(sağ_etiket()),
                    VeriÖğesi::yeni(0.18),
                ]),
        )
}

fn bar_negative() -> GrafikSeçenekleri {
    let seri = |ad: &str, yığın: Option<&str>, konum: EtiketKonumu, veri: [i32; 7]| {
        let mut seri = SütunSerisi::yeni()
            .ad(ad)
            .etiket(Etiket::yeni().göster(true).konum(konum))
            .veri(veri);
        if let Some(yığın) = yığın {
            seri = seri.yığın(yığın);
        }
        seri
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Gölge),
        )
        .gösterge(
            Gösterge::yeni()
                .iç_boşluk(15.0)
                .veri(["Profit", "Expenses", "Income"]),
        )
        .x_ekseni(Eksen::değer())
        .y_ekseni(
            Eksen::kategori()
                .çentik(EksenÇentiği {
                    göster: Some(false),
                    ..EksenÇentiği::default()
                })
                .veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .seri(seri(
            "Profit",
            None,
            EtiketKonumu::İç,
            [200, 170, 240, 244, 200, 220, 210],
        ))
        .seri(seri(
            "Income",
            Some("Total"),
            EtiketKonumu::İç,
            [320, 302, 341, 374, 390, 450, 420],
        ))
        .seri(seri(
            "Expenses",
            Some("Total"),
            EtiketKonumu::Sol,
            [-120, -132, -101, -134, -190, -230, -210],
        ))
}

fn bar_stack() -> GrafikSeçenekleri {
    let seri = |ad: &str, yığın: Option<&str>, veri: [i32; 7]| {
        let mut seri = SütunSerisi::yeni().ad(ad).veri(veri);
        if let Some(yığın) = yığın {
            seri = seri.yığın(yığın);
        }
        seri
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Gölge),
        )
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer())
        .seri(seri("Direct", None, [320, 332, 301, 334, 390, 330, 320]))
        .seri(seri(
            "Email",
            Some("Ad"),
            [120, 132, 101, 134, 90, 230, 210],
        ))
        .seri(seri(
            "Union Ads",
            Some("Ad"),
            [220, 182, 191, 234, 290, 330, 310],
        ))
        .seri(seri(
            "Video Ads",
            Some("Ad"),
            [150, 232, 201, 154, 190, 330, 410],
        ))
        .seri(
            seri(
                "Search Engine",
                None,
                [862, 1018, 964, 1026, 1679, 1600, 1570],
            )
            .im_çizgisi(
                İmÇizgisi::yeni().istatistik_parçası(İmDeğeri::EnKüçük, İmDeğeri::EnBüyük),
            ),
        )
        .seri(
            seri(
                "Baidu",
                Some("Search Engine"),
                [620, 732, 701, 734, 1090, 1130, 1120],
            )
            .genişlik(5),
        )
        .seri(seri(
            "Google",
            Some("Search Engine"),
            [120, 132, 101, 134, 290, 230, 220],
        ))
        .seri(seri(
            "Bing",
            Some("Search Engine"),
            [60, 72, 71, 74, 190, 130, 110],
        ))
        .seri(seri(
            "Others",
            Some("Search Engine"),
            [62, 82, 91, 84, 109, 110, 120],
        ))
}

fn bar_waterfall() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Waterfall Chart")
                .alt_metin("Living Expenses in Shenzhen")
                .iç_boşluk(15.0),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Gölge),
        )
        .ızgara(
            Izgara::yeni()
                .sol("3%")
                .sağ("4%")
                .alt("3%")
                .etiketi_kapsa(true),
        )
        .x_ekseni(Eksen::kategori().bölme_çizgisi_göster(false).veri([
            "Total",
            "Rent",
            "Utilities",
            "Transportation",
            "Meals",
            "Other",
        ]))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("Placeholder")
                .yığın("Total")
                .öğe_stili(
                    ÖğeStili::yeni()
                        .renk(Renk::SAYDAM)
                        .kenarlık_rengi(Renk::SAYDAM),
                )
                .veri([0, 1700, 1400, 1200, 300, 0]),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Life Cost")
                .yığın("Total")
                .etiket(Etiket::yeni().göster(true))
                .veri([2900, 1200, 300, 200, 900, 300]),
        )
}

fn bar_waterfall2() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Accumulated Waterfall Chart")
                .iç_boşluk(15.0),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Gölge),
        )
        .gösterge(
            Gösterge::yeni()
                .iç_boşluk(15.0)
                .veri(["Expenses", "Income"]),
        )
        .ızgara(
            Izgara::yeni()
                .sol("3%")
                .sağ("4%")
                .alt("3%")
                .etiketi_kapsa(true),
        )
        .x_ekseni(Eksen::kategori().veri([
            "Nov 1", "Nov 2", "Nov 3", "Nov 4", "Nov 5", "Nov 6", "Nov 7", "Nov 8", "Nov 9",
            "Nov 10", "Nov 11",
        ]))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .ad("Placeholder")
                .yığın("Total")
                .öğe_stili(
                    ÖğeStili::yeni()
                        .renk(Renk::SAYDAM)
                        .kenarlık_rengi(Renk::SAYDAM),
                )
                .veri([0, 900, 1245, 1530, 1376, 1376, 1511, 1689, 1856, 1495, 1292]),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Income")
                .yığın("Total")
                .etiket(Etiket::yeni().göster(true).konum(EtiketKonumu::Üst))
                .veri([
                    Some(900),
                    Some(345),
                    Some(393),
                    None,
                    None,
                    Some(135),
                    Some(178),
                    Some(286),
                    None,
                    None,
                    None,
                ]),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Expenses")
                .yığın("Total")
                .etiket(Etiket::yeni().göster(true).konum(EtiketKonumu::Alt))
                .veri([
                    None,
                    None,
                    None,
                    Some(108),
                    Some(154),
                    None,
                    None,
                    None,
                    Some(119),
                    Some(361),
                    Some(203),
                ]),
        )
}

fn bar_stack_normalization() -> GrafikSeçenekleri {
    let ham = [
        [100.0, 302.0, 301.0, 334.0, 390.0, 330.0, 320.0],
        [320.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
        [220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
        [150.0, 212.0, 201.0, 154.0, 190.0, 330.0, 410.0],
        [820.0, 832.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
    ];
    let toplamlar: [f64; 7] = std::array::from_fn(|veri_sırası| {
        ham.iter().filter_map(|seri| seri.get(veri_sırası)).sum()
    });
    let adlar = [
        "Direct",
        "Mail Ad",
        "Affiliate Ad",
        "Video Ad",
        "Search Engine",
    ];
    let yüzde = Biçimleyici::İşlev(Arc::new(|değer, _| {
        format!("{}%", (değer * 1000.0).round() / 10.0)
    }));

    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(
            Gösterge::yeni()
                .iç_boşluk(15.0)
                .seçim_kipi(GöstergeSeçimKipi::Kapalı),
        )
        .x_ekseni(Eksen::kategori().veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]))
        .y_ekseni(Eksen::değer());
    for (ad, ham_seri) in adlar.iter().zip(&ham) {
        let veri = std::array::from_fn::<_, 7, _>(|veri_sırası| {
            let toplam = toplamlar.get(veri_sırası).copied().unwrap_or_default();
            if toplam <= 0.0 {
                0.0
            } else {
                ham_seri.get(veri_sırası).copied().unwrap_or_default() / toplam
            }
        });
        seçenekler = seçenekler.seri(
            SütunSerisi::yeni()
                .ad(*ad)
                .yığın("total")
                .genişlik("60%")
                .etiket(Etiket::yeni().göster(true).biçimleyici(yüzde.clone()))
                .veri(veri),
        );
    }
    seçenekler
}

fn bar_label_rotation() -> GrafikSeçenekleri {
    let etiket = Etiket::yeni()
        .göster(true)
        .konum(EtiketKonumu::İçAlt)
        .uzaklık(15.0)
        .yatay_hiza(YazıYatayHizası::Sol)
        .dikey_hiza(YazıDikeyHizası::Orta)
        .döndürme(EtiketDöndürme::Derece(90.0))
        .biçimleyici("{c}  {name|{a}}")
        .yazı(YazıStili::yeni().boyut(16.0));
    let seri =
        |ad: &str, veri: [i32; 5]| SütunSerisi::yeni().ad(ad).etiket(etiket.clone()).veri(veri);

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .imleç(İmleçTürü::Gölge),
        )
        .gösterge(
            Gösterge::yeni()
                .iç_boşluk(15.0)
                .veri(["Forest", "Steppe", "Desert", "Wetland"]),
        )
        .araç_kutusu(
            AraçKutusu::yeni()
                .yön(Yön::Dikey)
                .sol("right")
                .üst("center")
                .veri_görünümü(true)
                .sihirli_tür(true, true)
                .sihirli_yığın(true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .x_ekseni(
            Eksen::kategori()
                .çentik(EksenÇentiği {
                    göster: Some(false),
                    ..EksenÇentiği::default()
                })
                .veri(["2012", "2013", "2014", "2015", "2016"]),
        )
        .y_ekseni(Eksen::değer())
        .seri(seri("Forest", [320, 332, 301, 334, 390]).sütun_boşluğu("0%"))
        .seri(seri("Steppe", [220, 182, 191, 234, 290]))
        .seri(seri("Desert", [150, 232, 201, 154, 190]))
        .seri(seri("Wetland", [98, 77, 101, 99, 40]))
}

fn data_transform_sort_bar() -> Result<GrafikSeçenekleri, String> {
    let kaynak = VeriKümesi::yeni(["name", "age", "profession", "score", "date"])
        .satır([
            "Hannah Krause".into(),
            41.into(),
            "Engineer".into(),
            314.into(),
            "2011-02-12".into(),
        ])
        .satır([
            "Zhao Qian".into(),
            20.into(),
            "Teacher".into(),
            351.into(),
            "2011-03-01".into(),
        ])
        .satır([
            "Jasmin Krause ".into(),
            52.into(),
            "Musician".into(),
            287.into(),
            "2011-02-14".into(),
        ])
        .satır([
            "Li Lei".into(),
            37.into(),
            "Teacher".into(),
            219.into(),
            "2011-02-18".into(),
        ])
        .satır([
            "Karle Neumann".into(),
            25.into(),
            "Engineer".into(),
            253.into(),
            "2011-04-02".into(),
        ])
        .satır([
            "Adrian Groß".into(),
            19.into(),
            "Teacher".into(),
            "-".into(),
            "2011-01-16".into(),
        ])
        .satır([
            "Mia Neumann".into(),
            71.into(),
            "Engineer".into(),
            165.into(),
            "2011-03-19".into(),
        ])
        .satır([
            "Böhm Fuchs".into(),
            36.into(),
            "Musician".into(),
            318.into(),
            "2011-02-24".into(),
        ])
        .satır([
            "Han Meimei".into(),
            67.into(),
            "Engineer".into(),
            366.into(),
            "2011-03-12".into(),
        ]);
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .veri_kümesi_ekle(VeriKümesiTanımı::kaynak(kaynak))
        .veri_kümesi_ekle(VeriKümesiTanımı::sırala([SıralamaAnahtarı::azalan(
            "score",
        )]))
        .x_ekseni(Eksen::kategori().etiket(EksenEtiketi::yeni().aralık(0).döndür(30.0)))
        .y_ekseni(Eksen::değer())
        .seri(
            SütunSerisi::yeni()
                .veri_kümesi_sırası(1)
                .eşle("name", "score"),
        ))
}

fn dataset_simple0() -> GrafikSeçenekleri {
    let kaynak = VeriKümesi::yeni(["product", "2015", "2016", "2017"]).kayıtlar([
        ("Matcha Latte", vec![43.3, 85.8, 93.7]),
        ("Milk Tea", vec![83.1, 73.4, 55.1]),
        ("Cheese Cocoa", vec![86.4, 65.2, 82.5]),
        ("Walnut Brownie", vec![72.4, 53.9, 39.1]),
    ]);
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .ipucu(İpucu::yeni())
        .veri_kümesi(kaynak)
        .x_ekseni(Eksen::kategori())
        .y_ekseni(Eksen::değer())
        .seri(SütunSerisi::yeni())
        .seri(SütunSerisi::yeni())
        .seri(SütunSerisi::yeni())
}

fn dataset_simple1() -> Result<GrafikSeçenekleri, String> {
    let nesne = |ürün: &str, y2015: f64, y2016: f64, y2017: f64| {
        vec![
            ("product".to_owned(), ürün.into()),
            ("2015".to_owned(), y2015.into()),
            ("2016".to_owned(), y2016.into()),
            ("2017".to_owned(), y2017.into()),
        ]
    };
    let kaynak = VeriKümesi::kaynaktan(
        VeriKaynağı::NesneSatırlar(vec![
            nesne("Matcha Latte", 43.3, 85.8, 93.7),
            nesne("Milk Tea", 83.1, 73.4, 55.1),
            nesne("Cheese Cocoa", 86.4, 65.2, 82.5),
            nesne("Walnut Brownie", 72.4, 53.9, 39.1),
        ]),
        KaynakSeçenekleri {
            boyutlar: ["product", "2015", "2016", "2017"]
                .into_iter()
                .map(BoyutTanımı::yeni)
                .collect(),
            ..KaynakSeçenekleri::default()
        },
    )
    .map_err(|hata| hata.to_string())?;
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .ipucu(İpucu::yeni())
        .veri_kümesi(kaynak)
        .x_ekseni(Eksen::kategori())
        .y_ekseni(Eksen::değer())
        .seri(SütunSerisi::yeni())
        .seri(SütunSerisi::yeni())
        .seri(SütunSerisi::yeni()))
}

fn dataset_series_layout_by() -> GrafikSeçenekleri {
    let kaynak = VeriKümesi::yeni(["product", "2012", "2013", "2014", "2015"]).kayıtlar([
        ("Matcha Latte", vec![41.1, 30.4, 65.1, 53.3]),
        ("Milk Tea", vec![86.5, 92.1, 85.7, 83.1]),
        ("Cheese Cocoa", vec![24.1, 67.2, 79.5, 86.4]),
    ]);
    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .ipucu(İpucu::yeni())
        .veri_kümesi(kaynak)
        .ızgara_ekle(Izgara::yeni().alt("55%"))
        .ızgara_ekle(Izgara::yeni().üst("55%"))
        .x_ekseni_ekle(Eksen::kategori().ızgara_sırası(0))
        .x_ekseni_ekle(Eksen::kategori().ızgara_sırası(1))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(0))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(1));
    for _ in 0..3 {
        seçenekler = seçenekler.seri(SütunSerisi::yeni().seri_yerleşimi(SeriYerleşimi::Satır));
    }
    for _ in 0..4 {
        seçenekler = seçenekler.seri(SütunSerisi::yeni().eksenler(1, 1));
    }
    seçenekler
}

fn dataset_encode0() -> GrafikSeçenekleri {
    let kaynak = VeriKümesi::yeni(["score", "amount", "product"])
        .satır([89.3.into(), 58_212.into(), "Matcha Latte".into()])
        .satır([57.1.into(), 78_254.into(), "Milk Tea".into()])
        .satır([74.4.into(), 41_032.into(), "Cheese Cocoa".into()])
        .satır([50.1.into(), 12_755.into(), "Cheese Brownie".into()])
        .satır([89.7.into(), 20_145.into(), "Matcha Cocoa".into()])
        .satır([68.1.into(), 79_146.into(), "Tea".into()])
        .satır([19.6.into(), 91_852.into(), "Orange Juice".into()])
        .satır([10.6.into(), 101_852.into(), "Lemon Juice".into()])
        .satır([32.7.into(), 20_112.into(), "Walnut Brownie".into()]);
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .veri_kümesi(kaynak)
        .ızgara(Izgara::yeni().etiketi_kapsa(true))
        .x_ekseni(Eksen::değer().ad("amount"))
        .y_ekseni(Eksen::kategori())
        .görsel_eşleme(
            GörselEşleme::yeni()
                .yön(Yön::Yatay)
                .sol("center")
                // Resmî seçenek `bottom` vermiyor; ECharts varsayılanı 0,
                // 15 px bileşen padding'i çubuğun gerçek alt boşluğunu kurar.
                .alt(0)
                .en_az(10.0)
                .en_çok(100.0)
                .metin("High Score", "Low Score")
                .boyut("score")
                .renkler([0x65b581u32, 0xffce34u32, 0xfd665fu32]),
        )
        .seri(SütunSerisi::yeni().eşle("product", "amount"))
}

fn dataset_default() -> GrafikSeçenekleri {
    let kaynak = VeriKümesi::yeni(["product", "2012", "2013", "2014", "2015", "2016", "2017"])
        .kayıtlar([
            ("Milk Tea", vec![86.5, 92.1, 85.7, 83.1, 73.4, 55.1]),
            ("Matcha Latte", vec![41.1, 30.4, 65.1, 53.3, 83.8, 98.7]),
            ("Cheese Cocoa", vec![24.1, 67.2, 79.5, 86.4, 65.2, 82.5]),
            ("Walnut Brownie", vec![55.2, 67.1, 69.2, 72.4, 53.9, 39.1]),
        ]);
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .ipucu(İpucu::yeni())
        .veri_kümesi(kaynak)
        .seri(PastaSerisi::yeni().yarıçap("20%").merkez("25%", "30%"))
        .seri(
            PastaSerisi::yeni()
                .yarıçap("20%")
                .merkez("75%", "30%")
                .eşle("product", "2013"),
        )
        .seri(
            PastaSerisi::yeni()
                .yarıçap("20%")
                .merkez("25%", "75%")
                .eşle("product", "2014"),
        )
        .seri(
            PastaSerisi::yeni()
                .yarıçap("20%")
                .merkez("75%", "75%")
                .eşle("product", "2015"),
        )
}

fn data_transform_multiple_pie() -> GrafikSeçenekleri {
    let mut kaynak = VeriKümesi::yeni(["Product", "Sales", "Price", "Year"]);
    for (ürün, satış, fiyat, yıl) in [
        ("Cake", 123, 32, 2011),
        ("Cereal", 231, 14, 2011),
        ("Tofu", 235, 5, 2011),
        ("Dumpling", 341, 25, 2011),
        ("Biscuit", 122, 29, 2011),
        ("Cake", 143, 30, 2012),
        ("Cereal", 201, 19, 2012),
        ("Tofu", 255, 7, 2012),
        ("Dumpling", 241, 27, 2012),
        ("Biscuit", 102, 34, 2012),
        ("Cake", 153, 28, 2013),
        ("Cereal", 181, 21, 2013),
        ("Tofu", 395, 4, 2013),
        ("Dumpling", 281, 31, 2013),
        ("Biscuit", 92, 39, 2013),
        ("Cake", 223, 29, 2014),
        ("Cereal", 211, 17, 2014),
        ("Tofu", 345, 3, 2014),
        ("Dumpling", 211, 35, 2014),
        ("Biscuit", 72, 24, 2014),
    ] {
        kaynak = kaynak.satır([ürün.into(), satış.into(), fiyat.into(), yıl.into()]);
    }
    let yıla_göre = |yıl: i32| {
        VeriKümesiTanımı::süz(SüzmeKoşulu::Karşılaştır {
            boyut: BoyutSeçici::ad("Year"),
            işlem: Karşılaştırmaİşlemi::Eşit,
            değer: yıl.into(),
        })
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .veri_kümeleri([
            VeriKümesiTanımı::kaynak(kaynak),
            yıla_göre(2011),
            yıla_göre(2012),
            yıla_göre(2013),
        ])
        // 700×525 görünümde resmî `media.minAspectRatio: 1` dalı etkindir.
        .seri(
            PastaSerisi::yeni()
                .yarıçap(50)
                .merkez("25%", "50%")
                .veri_kümesi_sırası(1),
        )
        .seri(
            PastaSerisi::yeni()
                .yarıçap(50)
                .merkez("50%", "50%")
                .veri_kümesi_sırası(2),
        )
        .seri(
            PastaSerisi::yeni()
                .yarıçap(50)
                .merkez("75%", "50%")
                .veri_kümesi_sırası(3),
        )
}

fn dataset_link(yıl: &str) -> GrafikSeçenekleri {
    let kaynak = VeriKümesi::yeni(["product", "2012", "2013", "2014", "2015", "2016", "2017"])
        .kayıtlar([
            ("Milk Tea", vec![56.5, 82.1, 88.7, 70.1, 53.4, 85.1]),
            ("Matcha Latte", vec![51.1, 51.4, 55.1, 53.3, 73.8, 68.7]),
            ("Cheese Cocoa", vec![40.1, 62.2, 69.5, 36.4, 45.2, 32.5]),
            ("Walnut Brownie", vec![25.2, 37.1, 41.2, 18.0, 33.9, 49.1]),
        ]);
    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(Gösterge::yeni().iç_boşluk(15.0))
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .içerik_göster(false),
        )
        .veri_kümesi(kaynak)
        .ızgara(Izgara::yeni().üst("55%"))
        .x_ekseni(Eksen::kategori())
        .y_ekseni(Eksen::değer());
    for _ in 0..4 {
        seçenekler = seçenekler.seri(
            ÇizgiSerisi::yeni()
                .yumuşat(true)
                .seri_yerleşimi(SeriYerleşimi::Satır),
        );
    }
    seçenekler.seri(
        PastaSerisi::yeni()
            .yarıçap("30%")
            .merkez("50%", "25%")
            .eşle("product", yıl)
            .etiket(
                Etiket::yeni()
                    .göster(true)
                    .konum(EtiketKonumu::Dış)
                    .biçimleyici(format!("{{b}}: {{@{yıl}}} ({{d}}%)")),
            ),
    )
}

fn yaşam_beklentisi_verisi() -> Result<VeriKümesi, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/data/asset/data/life-expectancy-table.json");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let ham: Vec<Vec<serde_json::Value>> = serde_json::from_str(&kaynak)
        .map_err(|hata| format!("{} ayrıştırılamadı: {hata}", dosya.display()))?;
    let satırlar = ham
        .into_iter()
        .map(|satır| {
            satır
                .into_iter()
                .map(|değer| match değer {
                    serde_json::Value::Null => VeriDeğeri::Boş,
                    serde_json::Value::Bool(değer) => değer.into(),
                    serde_json::Value::Number(değer) => {
                        değer.as_f64().map(Into::into).unwrap_or(VeriDeğeri::Boş)
                    }
                    serde_json::Value::String(değer) => değer.into(),
                    _ => VeriDeğeri::Boş,
                })
                .collect()
        })
        .collect();
    VeriKümesi::kaynaktan(
        VeriKaynağı::DiziSatırlar(satırlar),
        KaynakSeçenekleri::default(),
    )
    .map_err(|hata| hata.to_string())
}

fn data_transform_filter() -> Result<GrafikSeçenekleri, String> {
    let ülke = |ad: &str| {
        VeriKümesiTanımı::kaynaktan_süz(
            0,
            SüzmeKoşulu::Ve(vec![
                SüzmeKoşulu::Karşılaştır {
                    boyut: BoyutSeçici::ad("Year"),
                    işlem: Karşılaştırmaİşlemi::BüyükEşit,
                    değer: 1950.into(),
                },
                SüzmeKoşulu::Karşılaştır {
                    boyut: BoyutSeçici::ad("Country"),
                    işlem: Karşılaştırmaİşlemi::Eşit,
                    değer: ad.into(),
                },
            ]),
        )
    };
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Income of Germany and France since 1950")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .veri_kümeleri([
            VeriKümesiTanımı::kaynak(yaşam_beklentisi_verisi()?),
            ülke("Germany"),
            ülke("France"),
        ])
        .x_ekseni(Eksen::kategori())
        .y_ekseni(Eksen::değer().ad("Income"))
        .seri(
            ÇizgiSerisi::yeni()
                .veri_kümesi_sırası(1)
                .eşle("Year", "Income")
                .sembol_göster(false),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .veri_kümesi_sırası(2)
                .eşle("Year", "Income")
                .sembol_göster(false),
        ))
}

fn dataset_encode1() -> Result<GrafikSeçenekleri, String> {
    let etiket = || EksenEtiketi::yeni().döndür(50.0).aralık(0);
    let saçılım = |x: &str, y: &str, sıra: usize| {
        SaçılımSerisi::yeni()
            .sembol_boyutu(2.5)
            .eksenler(sıra, sıra)
            .eşle(x, y)
    };
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .gösterge(Gösterge::yeni())
        .ipucu(İpucu::yeni())
        .araç_kutusu(
            AraçKutusu::yeni()
                .sol(YatayKonum::Orta)
                .veri_yakınlaştırma(true),
        )
        .ızgara_ekle(Izgara::yeni().sağ("57%").alt("57%"))
        .ızgara_ekle(Izgara::yeni().sol("57%").alt("57%"))
        .ızgara_ekle(Izgara::yeni().sağ("57%").üst("57%"))
        .ızgara_ekle(Izgara::yeni().sol("57%").üst("57%"))
        .x_ekseni_ekle(
            Eksen::değer()
                .ızgara_sırası(0)
                .ad("Income")
                .etiket(etiket()),
        )
        .x_ekseni_ekle(
            Eksen::kategori()
                .ızgara_sırası(1)
                .ad("Country")
                .kenar_boşluğu(false)
                .etiket(etiket()),
        )
        .x_ekseni_ekle(
            Eksen::değer()
                .ızgara_sırası(2)
                .ad("Income")
                .etiket(etiket()),
        )
        .x_ekseni_ekle(
            Eksen::değer()
                .ızgara_sırası(3)
                .ad("Life Expectancy")
                .etiket(etiket()),
        )
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(0).ad("Life Expectancy"))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(1).ad("Income"))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(2).ad("Population"))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(3).ad("Population"))
        .veri_kümesi(yaşam_beklentisi_verisi()?)
        .seri(saçılım("Income", "Life Expectancy", 0))
        .seri(saçılım("Country", "Income", 1))
        .seri(saçılım("Income", "Population", 2))
        .seri(saçılım("Life Expectancy", "Population", 3)))
}

fn data_transform_aggregate() -> Result<GrafikSeçenekleri, String> {
    let ham = yaşam_beklentisi_verisi()?
        .depoya()
        .map_err(|hata| hata.to_string())?;
    let süzme = SüzmeDönüşümü {
        koşul: SüzmeKoşulu::Karşılaştır {
            boyut: BoyutSeçici::ad("Year"),
            işlem: Karşılaştırmaİşlemi::BüyükEşit,
            değer: 1950.into(),
        },
    };
    let since_year = süzme
        .uygula(&[ham])
        .map_err(|hata| hata.to_string())?
        .into_iter()
        .next()
        .ok_or_else(|| "filter sonucu yok".to_owned())?;
    let toplama = ToplamaDönüşümü::yeni(
        "Country",
        [
            ToplamaBoyutu::en_az("min", "Income"),
            ToplamaBoyutu::çeyrek1("Q1", "Income"),
            ToplamaBoyutu::ortanca("median", "Income"),
            ToplamaBoyutu::çeyrek3("Q3", "Income"),
            ToplamaBoyutu::en_çok("max", "Income"),
            ToplamaBoyutu::ilk("Country", "Country"),
        ],
    );
    let toplanmış = toplama
        .uygula(std::slice::from_ref(&since_year))
        .map_err(|hata| hata.to_string())?
        .into_iter()
        .next()
        .ok_or_else(|| "aggregate sonucu yok".to_owned())?;
    let sıralı = SıralamaDönüşümü {
        anahtarlar: vec![SıralamaAnahtarı::artan("Q3")],
    }
    .uygula(&[toplanmış])
    .map_err(|hata| hata.to_string())?
    .into_iter()
    .next()
    .ok_or_else(|| "sort sonucu yok".to_owned())?;

    let sayısal_değer = |satır: usize, boyut: &str| {
        sıralı
            .değer(satır, &BoyutSeçici::ad(boyut))
            .and_then(VeriDeğeri::sayı)
            .ok_or_else(|| format!("{satır}. satırda `{boyut}` sayısı yok"))
    };
    let ülke = |satır: usize| {
        sıralı
            .değer(satır, &BoyutSeçici::ad("Country"))
            .and_then(|değer| match değer {
                VeriDeğeri::Metin(metin) => Some(metin.clone()),
                _ => None,
            })
            .ok_or_else(|| format!("{satır}. satırda Country metni yok"))
    };
    let mut ülkeler = Vec::with_capacity(sıralı.sayım());
    let mut kutular = Vec::with_capacity(sıralı.sayım());
    for satır in 0..sıralı.sayım() {
        let ad = ülke(satır)?;
        let özet = ["min", "Q1", "median", "Q3", "max"]
            .into_iter()
            .map(|boyut| sayısal_değer(satır, boyut))
            .collect::<Result<Vec<_>, _>>()?;
        ülkeler.push(ad.clone());
        kutular.push(VeriÖğesi::adlı(ad, VeriDeğeri::Dizi(özet)));
    }

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Income since 1950").iç_boşluk(15.0))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(Gösterge::yeni().iç_boşluk(15.0).seçili("detail", false))
        .ızgara(Izgara::yeni().alt(140))
        .x_ekseni(
            Eksen::değer()
                .ad("Income")
                .ad_konumu(EksenAdKonumu::Orta)
                .ad_boşluğu(30.0)
                .ölçekli(true),
        )
        .y_ekseni(Eksen::kategori().veri(ülkeler))
        .veri_kümesi(VeriKümesi::depodan(&since_year))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç())
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().yükseklik(20).alt(60))
        .seri(
            KutuSerisi::yeni()
                .ad("boxplot")
                .öğe_stili(ÖğeStili::yeni().renk(0xb8c5f2u32))
                .veri(kutular),
        )
        .seri(
            SaçılımSerisi::yeni()
                .ad("detail")
                .sembol_boyutu(6.0)
                .öğe_stili(ÖğeStili::yeni().renk(0xd00000u32))
                .eşle("Income", "Country")
                .etiket_boyutunu_eşle("Year")
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .konum(EtiketKonumu::Üst)
                        .yatay_hiza(YazıYatayHizası::Sol)
                        .dikey_hiza(YazıDikeyHizası::Orta)
                        .döndürme(EtiketDöndürme::Derece(90.0))
                        .yazı(YazıStili::yeni().boyut(12.0)),
                ),
        ))
}

/// Referans üreticisindeki sabitlenmiş `Math.random` (Mulberry32) akışının
/// 32 bit JavaScript işlemleriyle birebir karşılığı.
struct Mulberry32 {
    durum: u32,
}

impl Mulberry32 {
    fn yeni(durum: u32) -> Self {
        Self { durum }
    }

    fn sonraki(&mut self) -> f64 {
        self.durum = self.durum.wrapping_add(0x6D2B79F5);
        let mut t = (self.durum ^ (self.durum >> 15)).wrapping_mul(1 | self.durum);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }
}

type KutuDönüşümSerileri = (Vec<String>, Vec<VeriÖğesi>, Vec<VeriÖğesi>);

fn kutu_dönüşümünü_serilere_çevir(
    sonuçlar: Vec<VeriDeposu>,
    yatay: bool,
) -> Result<KutuDönüşümSerileri, String> {
    let kutu_deposu = sonuçlar
        .first()
        .ok_or_else(|| "boxplot özet sonucu yok".to_owned())?;
    let aykırı_deposu = sonuçlar
        .get(1)
        .ok_or_else(|| "boxplot aykırı sonucu yok".to_owned())?;
    let mut kategoriler = Vec::with_capacity(kutu_deposu.sayım());
    let mut kutular = Vec::with_capacity(kutu_deposu.sayım());
    for satır in kutu_deposu.satırları_kopyala() {
        let ad = match satır.first() {
            Some(VeriDeğeri::Metin(ad)) => ad.clone(),
            _ => return Err("boxplot ItemName metni yok".to_owned()),
        };
        let özet = satır
            .iter()
            .skip(1)
            .take(5)
            .map(|değer| {
                değer
                    .sayı()
                    .ok_or_else(|| format!("{ad} boxplot özeti sayısal değil"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        kategoriler.push(ad.clone());
        kutular.push(VeriÖğesi::adlı(ad, VeriDeğeri::Dizi(özet)));
    }

    let mut aykırılar = Vec::with_capacity(aykırı_deposu.sayım());
    for satır in aykırı_deposu.satırları_kopyala() {
        let ad = match satır.first() {
            Some(VeriDeğeri::Metin(ad)) => ad,
            _ => return Err("aykırı ItemName metni yok".to_owned()),
        };
        let kategori = kategoriler
            .iter()
            .position(|kategori| kategori == ad)
            .ok_or_else(|| format!("aykırı kategori bulunamadı: {ad}"))?
            as f64;
        let değer = satır
            .get(1)
            .and_then(VeriDeğeri::sayı)
            .ok_or_else(|| format!("{ad} aykırı değeri sayısal değil"))?;
        aykırılar.push(if yatay {
            VeriÖğesi::yeni([değer, kategori])
        } else {
            VeriÖğesi::yeni([kategori, değer])
        });
    }
    Ok((kategoriler, kutular, aykırılar))
}

fn boxplot_multi() -> Result<GrafikSeçenekleri, String> {
    let mut rastgele = Mulberry32::yeni(0x5eed1234);
    let mut bütün_kutular = Vec::with_capacity(3);
    let mut kategoriler = Vec::new();
    for veri_sırası in 0..3 {
        let satırlar = (0..18)
            .map(|_| {
                (0..100)
                    .map(|_| VeriDeğeri::from(rastgele.sonraki() * 200.0))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let kaynak = VeriDeposu::satırlardan(
            (0..100).map(|sıra| BoyutTanımı::yeni(format!("sample{sıra}"))),
            satırlar,
        )
        .map_err(|hata| hata.to_string())?;
        let sonuçlar = KutuDönüşümü::yeni()
            .uygula(&[kaynak])
            .map_err(|hata| hata.to_string())?;
        let (bu_kategoriler, kutular, _) = kutu_dönüşümünü_serilere_çevir(sonuçlar, false)?;
        if veri_sırası == 0 {
            kategoriler = bu_kategoriler;
        }
        bütün_kutular.push(kutular);
    }

    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Multiple Categories").iç_boşluk(15.0))
        .gösterge(Gösterge::yeni().üst("10%").iç_boşluk(15.0))
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Öğe)
                .imleç(İmleçTürü::Gölge),
        )
        .ızgara(Izgara::yeni().sol("10%").üst("20%").sağ("10%").alt("15%"))
        .x_ekseni(
            Eksen::kategori()
                .veri(kategoriler)
                .bölme_alanı_göster(true)
                .bölme_çizgisi_göster(false),
        )
        .y_ekseni(
            Eksen::değer()
                .ad("Value")
                .en_az(-400.0)
                .en_çok(600.0)
                .bölme_alanı_göster(false),
        )
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(0.0, 20.0))
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().üst("90%").aralık(0.0, 20.0));
    for (sıra, kutular) in bütün_kutular.into_iter().enumerate() {
        seçenekler = seçenekler.seri(
            KutuSerisi::yeni()
                .ad(format!("category{sıra}"))
                .veri(kutular),
        );
    }
    Ok(seçenekler)
}

fn michelson_morley_kaynağı() -> Result<VeriDeposu, String> {
    let satırlar: [[f64; 20]; 5] = [
        [
            850.0, 740.0, 900.0, 1070.0, 930.0, 850.0, 950.0, 980.0, 980.0, 880.0, 1000.0, 980.0,
            930.0, 650.0, 760.0, 810.0, 1000.0, 1000.0, 960.0, 960.0,
        ],
        [
            960.0, 940.0, 960.0, 940.0, 880.0, 800.0, 850.0, 880.0, 900.0, 840.0, 830.0, 790.0,
            810.0, 880.0, 880.0, 830.0, 800.0, 790.0, 760.0, 800.0,
        ],
        [
            880.0, 880.0, 880.0, 860.0, 720.0, 720.0, 620.0, 860.0, 970.0, 950.0, 880.0, 910.0,
            850.0, 870.0, 840.0, 840.0, 850.0, 840.0, 840.0, 840.0,
        ],
        [
            890.0, 810.0, 810.0, 820.0, 800.0, 770.0, 760.0, 740.0, 750.0, 760.0, 910.0, 920.0,
            890.0, 860.0, 880.0, 720.0, 840.0, 850.0, 850.0, 780.0,
        ],
        [
            890.0, 840.0, 780.0, 810.0, 760.0, 810.0, 790.0, 810.0, 820.0, 850.0, 870.0, 870.0,
            810.0, 740.0, 810.0, 940.0, 950.0, 800.0, 810.0, 870.0,
        ],
    ];
    VeriDeposu::satırlardan(
        (0..20).map(|sıra| BoyutTanımı::yeni(format!("sample{sıra}"))),
        satırlar
            .into_iter()
            .map(|satır| satır.into_iter().map(VeriDeğeri::from).collect())
            .collect(),
    )
    .map_err(|hata| hata.to_string())
}

fn boxplot_light_velocity(yatay: bool) -> Result<GrafikSeçenekleri, String> {
    let sonuçlar = KutuDönüşümü::yeni()
        .öğe_adı_biçimi("expr {value}")
        .uygula(&[michelson_morley_kaynağı()?])
        .map_err(|hata| hata.to_string())?;
    let (kategoriler, kutular, aykırılar) = kutu_dönüşümünü_serilere_çevir(sonuçlar, yatay)?;

    let açıklama_yazısı = if yatay {
        YazıStili::yeni().boyut(14.0)
    } else {
        YazıStili::yeni()
            .boyut(14.0)
            .satır_yüksekliği(20.0)
            .kalın(false)
    };
    let ana_başlık = Başlık::yeni()
        .metin("Michelson-Morley Experiment")
        .iç_boşluk(15.0);
    let açıklama = Başlık::yeni()
        .metin("upper: Q3 + 1.5 * IQR \nlower: Q1 - 1.5 * IQR")
        .sol("10%")
        .üst("90%")
        .iç_boşluk(15.0)
        .kenarlık_rengi(0x999999u32)
        .kenarlık_kalınlığı(1.0)
        .yazı(açıklama_yazısı);
    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(ana_başlık)
        .başlık_ekle(açıklama)
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Öğe)
                .imleç(İmleçTürü::Gölge),
        )
        .ızgara(Izgara::yeni().sol("10%").sağ("10%").alt("15%"));
    if yatay {
        seçenekler = seçenekler
            .x_ekseni(
                Eksen::değer()
                    .ad("km/s minus 299,000")
                    .bölme_alanı_göster(true),
            )
            .y_ekseni(
                Eksen::kategori()
                    .veri(kategoriler)
                    .bölme_alanı_göster(false)
                    .bölme_çizgisi_göster(false),
            );
    } else {
        seçenekler = seçenekler
            .x_ekseni(
                Eksen::kategori()
                    .veri(kategoriler)
                    .bölme_alanı_göster(false)
                    .bölme_çizgisi_göster(false),
            )
            .y_ekseni(
                Eksen::değer()
                    .ad("km/s minus 299,000")
                    .bölme_alanı_göster(true),
            );
    }
    Ok(seçenekler
        .seri(KutuSerisi::yeni().ad("boxplot").veri(kutular))
        .seri(SaçılımSerisi::yeni().ad("outlier").veri(aykırılar)))
}

fn scatter_simple() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::değer())
        .y_ekseni(Eksen::değer())
        .seri(SaçılımSerisi::yeni().sembol_boyutu(20.0).veri([
            [10.0, 8.04],
            [8.07, 6.95],
            [13.0, 7.58],
            [9.05, 8.81],
            [11.0, 8.33],
            [14.0, 7.66],
            [13.4, 6.81],
            [10.0, 6.33],
            [14.0, 8.96],
            [12.5, 6.82],
            [9.15, 7.2],
            [11.5, 7.2],
            [3.03, 4.23],
            [12.2, 7.83],
            [2.02, 4.47],
            [1.05, 3.33],
            [4.05, 4.96],
            [6.03, 7.24],
            [12.0, 6.26],
            [12.0, 8.84],
            [7.08, 5.82],
            [5.02, 5.68],
        ]))
}

fn scatter_punch_card() -> Result<GrafikSeçenekleri, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/examples/ts/scatter-punchCard.ts");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let saatler: Vec<String> = resmi_javascript_dizisi(&kaynak, "const hours")?;
    let günler: Vec<String> = resmi_javascript_dizisi(&kaynak, "const days")?;
    let ham_veri: Vec<[f64; 3]> = resmi_javascript_dizisi(&kaynak, "const data")?;
    let veri = ham_veri
        .into_iter()
        // Resmî örnekteki `.map`: [gün, saat, değer] -> [saat, gün, değer].
        .map(|[gün, saat, değer]| [saat, gün, değer])
        .collect::<Vec<_>>();

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(Başlık::yeni().metin("Punch Card of Github").iç_boşluk(15.0))
        .gösterge(
            Gösterge::yeni()
                .veri(["Punch Card"])
                .sol("right")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().konum(İpucuKonumu::Üst))
        .ızgara(Izgara::yeni().sol(2).alt(10).sağ(10).etiketi_kapsa(true))
        .x_ekseni(
            Eksen::kategori()
                .veri(saatler)
                .kenar_boşluğu(false)
                .bölme_çizgisi_göster(true)
                .çizgi(EksenÇizgisi::yeni().göster(false)),
        )
        .y_ekseni(
            Eksen::kategori()
                .veri(günler)
                .çizgi(EksenÇizgisi::yeni().göster(false)),
        )
        .seri(
            SaçılımSerisi::yeni()
                .ad("Punch Card")
                .sembol_boyutu_işlevi(|öğe| {
                    öğe
                        .değer
                        .dizi()
                        .and_then(|değerler| değerler.get(2))
                        .copied()
                        .unwrap_or_default() as f32
                        * 2.0
                })
                .veri(veri),
        ))
}

fn scatter_anscombe_quartet() -> Result<GrafikSeçenekleri, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/examples/ts/scatter-anscombe-quartet.ts");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let veri: Vec<Vec<[f64; 2]>> = resmi_javascript_dizisi(&kaynak, "const dataAll")?;
    if veri.len() != 4 {
        return Err(format!(
            "{} dört Anscombe veri grubu içermiyor",
            dosya.display()
        ));
    }

    let im_çizgisi = || {
        İmÇizgisi::yeni()
            .stil(ÇizgiStili::yeni().kalınlık(1.0).tür(ÇizgiTürü::Düz))
            .etiket(
                Etiket::yeni()
                    .göster(true)
                    .biçimleyici("y = 0.5 * x + 3")
                    .yatay_hiza(YazıYatayHizası::Sağ),
            )
            .koordinat_parçası((0.0, 3.0), (20.0, 13.0))
            .parça_simgeleri(İmÇizgisiUçSimgesi::Yok, İmÇizgisiUçSimgesi::Yok)
    };

    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Anscombe's quartet")
                .sol("center")
                .üst(0)
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().biçimleyici("Group {a}: ({c})"))
        .ızgara_ekle(
            Izgara::yeni()
                .sol("7%")
                .üst("7%")
                .genişlik("38%")
                .yükseklik("38%"),
        )
        .ızgara_ekle(
            Izgara::yeni()
                .sağ("7%")
                .üst("7%")
                .genişlik("38%")
                .yükseklik("38%"),
        )
        .ızgara_ekle(
            Izgara::yeni()
                .sol("7%")
                .alt("7%")
                .genişlik("38%")
                .yükseklik("38%"),
        )
        .ızgara_ekle(
            Izgara::yeni()
                .sağ("7%")
                .alt("7%")
                .genişlik("38%")
                .yükseklik("38%"),
        );
    for sıra in 0..4 {
        seçenekler = seçenekler
            .x_ekseni_ekle(Eksen::değer().ızgara_sırası(sıra).en_az(0.0).en_çok(20.0))
            .y_ekseni_ekle(Eksen::değer().ızgara_sırası(sıra).en_az(0.0).en_çok(15.0))
            .seri(
                SaçılımSerisi::yeni()
                    .ad(["I", "II", "III", "IV"][sıra])
                    .eksenler(sıra, sıra)
                    .im_çizgisi(im_çizgisi())
                    .veri(veri[sıra].clone()),
            );
    }
    Ok(seçenekler)
}

fn scatter_jitter() -> GrafikSeçenekleri {
    let mut tohum = 0x5eed_1234_u32;
    let mut veri = Vec::with_capacity(7_000);
    for gün in 0..7 {
        for sıra in 0..1_000 {
            let y = (sıra as f64).tan() / 2.0 + 7.0;
            veri.push([gün as f64, y, kanıt_rastgele(&mut tohum)]);
        }
    }
    // Resmî kaynak jitter genişliğini `chartWidth - grid.left - grid.right`
    // üzerinden hesaplar: (700 - 80 - 50) / 7 * 0.8.
    let titreme = (700.0 - 80.0 - 50.0) / 7.0 * 0.8;

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Scatter with Jittering")
                .iç_boşluk(15.0),
        )
        .ızgara(Izgara::yeni().sol(80).sağ(50))
        .x_ekseni(
            Eksen::kategori()
                .titreme(titreme)
                // Kaynak, jitter yerleşiminden önce üçüncü boyut için 7.000
                // Math.random çağrısı tüketir; aynı akışın kalan durumu.
                .titreme_tohumu(tohum)
                .veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .y_ekseni(Eksen::değer().en_az(0.0).en_çok(10.0))
        .seri(
            SaçılımSerisi::yeni()
                .ad("Sleeping Hours")
                .veriye_göre_renklendir(true)
                .öğe_stili(ÖğeStili::yeni().opaklık(0.4))
                .veri(veri),
        )
}

fn scatter_jitter_avoid_overlap() -> GrafikSeçenekleri {
    let mut tohum = 0x5eed_1234_u32;
    let mut veri = Vec::with_capacity(210);
    for gün in 0..7 {
        for sıra in 0..30 {
            let y = (sıra as f64).tan() / 2.0 + 7.0;
            veri.push([gün as f64, y, kanıt_rastgele(&mut tohum)]);
        }
    }
    let titreme = (700.0 - 80.0 - 50.0) / 7.0 * 0.8;

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Scatter with Jittering")
                .iç_boşluk(15.0),
        )
        .ızgara(Izgara::yeni().sol(80).sağ(50))
        .x_ekseni(
            Eksen::kategori()
                .titreme(titreme)
                .titreme_örtüşmesi(false)
                .titreme_tohumu(tohum)
                .veri(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .y_ekseni(Eksen::değer().en_az(0.0).en_çok(10.0))
        .seri(
            SaçılımSerisi::yeni()
                .ad("Sleeping Hours")
                .veriye_göre_renklendir(true)
                .veri(veri),
        )
}

fn resmi_ülke_saçılım_verisi(kimlik: &str) -> Result<Vec<Vec<VeriÖğesi>>, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../echarts-examples/public/examples/ts/{kimlik}.ts"
    ));
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let ham: Vec<Vec<Vec<serde_json::Value>>> = resmi_javascript_dizisi(&kaynak, "const data")?;
    if ham.len() != 2 {
        return Err(format!(
            "{} iki resmi ülke veri grubu içermiyor",
            dosya.display()
        ));
    }
    ham
        .into_iter()
        .enumerate()
        .map(|(grup_sırası, grup)| {
            grup
                .into_iter()
                .enumerate()
                .map(|(satır_sırası, satır)| {
                    let sayı = |sıra: usize| {
                        satır.get(sıra).and_then(serde_json::Value::as_f64).ok_or_else(|| {
                            format!(
                                "{kimlik} grup {grup_sırası} satır {satır_sırası} boyut {sıra} sayısal değil"
                            )
                        })
                    };
                    let ülke = satır
                        .get(3)
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| {
                            format!(
                                "{kimlik} grup {grup_sırası} satır {satır_sırası} ülke adı değil"
                            )
                        })?;
                    let yıl = sayı(4)?;
                    Ok(VeriÖğesi::adlı(ülke, vec![sayı(0)?, sayı(1)?, sayı(2)?])
                        .boyutlar([("year".to_owned(), VeriDeğeri::Sayı(yıl))]))
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .collect::<Result<Vec<_>, String>>()
}

fn ülke_kabarcık_boyutu(öğe: &VeriÖğesi) -> f32 {
    öğe
        .değer
        .dizi()
        .and_then(|değerler| değerler.get(2))
        .copied()
        .unwrap_or_default()
        .sqrt() as f32
        / 500.0
}

fn bubble_gradient() -> Result<GrafikSeçenekleri, String> {
    let veri_grupları = resmi_ülke_saçılım_verisi("bubble-gradient")?;

    let bölme = BölmeÇizgisi {
        göster: Some(true),
        renk: None,
        tür: ÇizgiTürü::Kesikli,
    };
    let kabarcık = |ad: &str, veri: Vec<VeriÖğesi>, dolgu: Dolgu, gölge: Renk| {
        SaçılımSerisi::yeni()
            .ad(ad)
            .veri(veri)
            .sembol_boyutu_işlevi(ülke_kabarcık_boyutu)
            .öğe_stili(
                ÖğeStili::yeni()
                    .renk(dolgu)
                    .gölge_bulanıklığı(10.0)
                    .gölge_rengi(gölge)
                    .gölge_kayması(0.0, 5.0),
            )
    };
    let kırmızı = Dolgu::radyal(
        0.4,
        0.3,
        1.0,
        vec![
            RenkDurağı {
                konum: 0.0,
                renk: Renk::from("rgb(251, 118, 123)"),
            },
            RenkDurağı {
                konum: 1.0,
                renk: Renk::from("rgb(204, 46, 72)"),
            },
        ],
    );
    let mavi = Dolgu::radyal(
        0.4,
        0.3,
        1.0,
        vec![
            RenkDurağı {
                konum: 0.0,
                renk: Renk::from("rgb(129, 227, 238)"),
            },
            RenkDurağı {
                konum: 1.0,
                renk: Renk::from("rgb(25, 183, 207)"),
            },
        ],
    );

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .arkaplan(Dolgu::radyal(
            0.3,
            0.3,
            0.8,
            vec![
                RenkDurağı {
                    konum: 0.0,
                    renk: Renk::from("#f7f8fa"),
                },
                RenkDurağı {
                    konum: 1.0,
                    renk: Renk::from("#cdd0d5"),
                },
            ],
        ))
        .başlık(
            Başlık::yeni()
                .metin("Life Expectancy and GDP by Country")
                .sol("5%")
                .üst("3%")
                .iç_boşluk(15.0),
        )
        .gösterge(
            Gösterge::yeni()
                .sağ("10%")
                .üst("3%")
                .iç_boşluk(15.0)
                .veri(["1990", "2015"]),
        )
        .ızgara(Izgara::yeni().sol("8%").üst("10%"))
        .x_ekseni(Eksen::değer().bölme_çizgisi(bölme.clone()))
        .y_ekseni(Eksen::değer().ölçekli(true).bölme_çizgisi(bölme))
        .seri(kabarcık(
            "1990",
            veri_grupları[0].clone(),
            kırmızı,
            Renk::from("rgba(120, 36, 50, 0.5)"),
        ))
        .seri(kabarcık(
            "2015",
            veri_grupları[1].clone(),
            mavi,
            Renk::from("rgba(25, 100, 150, 0.5)"),
        )))
}

fn scatter_label_align_top() -> Result<GrafikSeçenekleri, String> {
    let veri = resmi_ülke_saçılım_verisi("scatter-label-align-top")?
        .into_iter()
        .next()
        .ok_or_else(|| "scatter-label-align-top 1990 verisi yok".to_owned())?;
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::değer())
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(
            SaçılımSerisi::yeni()
                .ad("1990")
                .veri(veri)
                .sembol_boyutu_işlevi(ülke_kabarcık_boyutu)
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .biçimleyici("{b}")
                        .en_küçük_boşluk(10.0)
                        .konum(EtiketKonumu::Üst),
                )
                .etiket_çizgisi(
                    EtiketÇizgisi::yeni()
                        .uzunluk2(5.0)
                        .stil(ÇizgiStili::yeni().kalınlık(1.0).renk("#bbb")),
                )
                .etiket_yerleşimi(|_| {
                    EtiketYerleşimSonucu::yeni()
                        .y(20.0)
                        .yatay_hiza(YazıYatayHizası::Orta)
                        .örtüşme_kaydırması(EtiketÖrtüşmeKaydırması::X)
                        .çakışanı_gizle(true)
                }),
        ))
}

fn scatter_label_align_right() -> Result<GrafikSeçenekleri, String> {
    let veri = resmi_ülke_saçılım_verisi("scatter-label-align-right")?
        .into_iter()
        .next()
        .ok_or_else(|| "scatter-label-align-right 1990 verisi yok".to_owned())?;
    let gizli_bölme = BölmeÇizgisi {
        göster: Some(false),
        renk: None,
        tür: ÇizgiTürü::Düz,
    };
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ızgara(Izgara::yeni().sol(40).sağ(130))
        .x_ekseni(Eksen::değer().bölme_çizgisi(gizli_bölme.clone()))
        .y_ekseni(Eksen::değer().ölçekli(true).bölme_çizgisi(gizli_bölme))
        .seri(
            SaçılımSerisi::yeni()
                .ad("1990")
                .veri(veri)
                .sembol_boyutu_işlevi(ülke_kabarcık_boyutu)
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .biçimleyici("{b}")
                        .en_küçük_boşluk(2.0)
                        .konum(EtiketKonumu::Sağ),
                )
                .etiket_çizgisi(
                    EtiketÇizgisi::yeni()
                        .uzunluk2(5.0)
                        .stil(ÇizgiStili::yeni().kalınlık(1.0).renk("#bbb")),
                )
                .etiket_yerleşimi(|_| {
                    EtiketYerleşimSonucu::yeni()
                        .x(600.0)
                        .örtüşme_kaydırması(EtiketÖrtüşmeKaydırması::Y)
                }),
        ))
}

fn resmi_aqi_saçılım_verisi() -> Result<Vec<Vec<VeriÖğesi>>, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/examples/ts/scatter-aqi-color.ts");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let boyut_adları = ["date", "AQIindex", "PM25", "PM10", "CO", "NO2", "SO2"];

    ["const dataBJ", "const dataSH", "const dataGZ"]
        .into_iter()
        .map(|belirteç| {
            let satırlar: Vec<Vec<serde_json::Value>> =
                resmi_javascript_dizisi(&kaynak, belirteç)?;
            satırlar
                .into_iter()
                .enumerate()
                .map(|(satır_sırası, satır)| {
                    let sayılar = (0..7)
                        .map(|boyut_sırası| {
                            satır
                                .get(boyut_sırası)
                                .and_then(serde_json::Value::as_f64)
                                .ok_or_else(|| {
                                    format!(
                                        "{belirteç} satır {satır_sırası} boyut {boyut_sırası} sayısal değil"
                                    )
                                })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    let durum = satır
                        .get(7)
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| {
                            format!("{belirteç} satır {satır_sırası} hava durumu metin değil")
                        })?;
                    let mut boyutlar = boyut_adları
                        .iter()
                        .zip(&sayılar)
                        .map(|(ad, değer)| ((*ad).to_owned(), VeriDeğeri::Sayı(*değer)))
                        .collect::<Vec<_>>();
                    boyutlar.push(("status".to_owned(), VeriDeğeri::Metin(durum.to_owned())));
                    Ok(VeriÖğesi::yeni(sayılar).boyutlar(boyutlar))
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .collect()
}

fn scatter_aqi_color() -> Result<GrafikSeçenekleri, String> {
    let veri = resmi_aqi_saçılım_verisi()?;
    let bölme_çizgisi = BölmeÇizgisi {
        göster: Some(false),
        renk: None,
        tür: ÇizgiTürü::Düz,
    };
    let öğe_stili = ÖğeStili::yeni()
        .opaklık(0.8)
        .gölge_bulanıklığı(10.0)
        .gölge_rengi("rgba(0,0,0,0.3)");
    let seri = |ad: &str, veri: Vec<VeriÖğesi>| {
        SaçılımSerisi::yeni()
            .ad(ad)
            .öğe_stili(öğe_stili.clone())
            .veri(veri)
    };

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .palet(["#dd4444", "#fec42c", "#80F1BE"])
        .gösterge(
            Gösterge::yeni()
                .üst(10)
                .iç_boşluk(15.0)
                .yazı(YazıStili::yeni().boyut(16.0))
                .veri(["北京", "上海", "广州"]),
        )
        .ızgara(Izgara::yeni().sol("10%").sağ(150).üst("18%").alt("10%"))
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .x_ekseni(
            Eksen::değer()
                .ad("日期")
                .ad_boşluğu(16.0)
                .ad_yazı(YazıStili::yeni().boyut(16.0))
                .en_çok(31.0)
                .bölme_çizgisi(bölme_çizgisi.clone()),
        )
        .y_ekseni(
            Eksen::değer()
                .ad("AQI指数")
                .ad_boşluğu(20.0)
                .ad_yazı(YazıStili::yeni().boyut(16.0))
                .bölme_çizgisi(bölme_çizgisi),
        )
        .görsel_eşlemeler([
            GörselEşleme::yeni()
                .sol("right")
                .üst("10%")
                .boyut(2usize)
                .en_az(0.0)
                .en_çok(250.0)
                .öğe_genişliği(30.0)
                .öğe_yüksekliği(120.0)
                .hesaplanabilir(true)
                .metin("圆形大小：PM2.5", "")
                .metin_boşluğu(30.0)
                .sembol_boyutu(10.0, 70.0)
                .aralık_dışı_sembol_boyutu(10.0, 70.0)
                .aralık_dışı_renk("rgba(255,255,255,0.4)")
                .denetleyici_renkleri(["#c23531"])
                .denetleyici_aralık_dışı_renk("#999"),
            GörselEşleme::yeni()
                .sol("right")
                .alt("5%")
                .boyut(6usize)
                .en_az(0.0)
                .en_çok(50.0)
                .öğe_yüksekliği(120.0)
                .metin("明暗：二氧化硫", "")
                .metin_boşluğu(30.0)
                .renk_açıklığı(0.9, 0.5)
                .aralık_dışı_renk("rgba(255,255,255,0.4)")
                .denetleyici_renkleri(["#c23531"])
                .denetleyici_aralık_dışı_renk("#999"),
        ])
        .seri(seri("北京", veri[0].clone()))
        .seri(seri("上海", veri[1].clone()))
        .seri(seri("广州", veri[2].clone())))
}

fn scatter_weight() -> Result<GrafikSeçenekleri, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/examples/ts/scatter-weight.ts");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let kadın: Vec<[f64; 2]> = resmi_javascript_dizisi(&kaynak, "name: 'Female'")?;
    let erkek: Vec<[f64; 2]> = resmi_javascript_dizisi(&kaynak, "name: 'Male'")?;

    let seri = |ad: &str, veri: Vec<[f64; 2]>, dikey_im: f64, ortalama_adı: &str| {
        let alan = İmAlanı::yeni()
            .veri_kapsamı(format!("{ad} Data Range"))
            .stil(
                ÖğeStili::yeni()
                    .renk("transparent")
                    .kenarlık_kalınlığı(1.0)
                    .kenarlık_türü(ÇizgiTürü::Kesikli),
            );
        let çizgi = İmÇizgisi::yeni()
            .stil(ÇizgiStili::yeni().kalınlık(1.0).tür(ÇizgiTürü::Düz))
            .tanım(İmÇizgisiTanımı::yeni(İmYönü::Yatay, İmDeğeri::Ortalama).ad(ortalama_adı))
            .dikey(İmDeğeri::Değer(dikey_im));
        SaçılımSerisi::yeni()
            .ad(ad)
            .veri(veri)
            .im_alanı(alan)
            .im_noktası(İmNoktası::yeni().en_büyük().en_küçük())
            .im_çizgisi(çizgi)
    };

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Male and female height and weight distribution")
                .alt_metin("Data from: Heinz 2003")
                .iç_boşluk(15.0),
        )
        .ızgara(
            Izgara::yeni()
                .sol("3%")
                .sağ("7%")
                .alt("7%")
                .etiketi_kapsa(true),
        )
        .ipucu(
            İpucu::yeni()
                .imleç(İmleçTürü::Çapraz)
                .bağlamlı_biçimleyici(|parametreler| {
                    let Some(parametre) = parametreler.first() else {
                        return String::new();
                    };
                    match &parametre.değer {
                        VeriDeğeri::Çift([boy, ağırlık]) => format!(
                            "{} :<br/>{}cm {}kg ",
                            parametre.seri_adı,
                            ondalık_kırp(*boy),
                            ondalık_kırp(*ağırlık)
                        ),
                        değer => format!(
                            "{} :<br/>{} : {}kg ",
                            parametre.seri_adı,
                            parametre.ad,
                            değer.sayı().map(ondalık_kırp).unwrap_or_default()
                        ),
                    }
                }),
        )
        .araç_kutusu(AraçKutusu::yeni().veri_yakınlaştırma(true).fırça_türleri([
            FırçaAracıTürü::Dikdörtgen,
            FırçaAracıTürü::Çokgen,
            FırçaAracıTürü::Temizle,
        ]))
        .fırça(Fırça::default())
        .gösterge(
            Gösterge::yeni()
                .sol("center")
                .alt(10)
                .iç_boşluk(15.0)
                .veri(["Female", "Male"]),
        )
        .x_ekseni(
            Eksen::değer()
                .ölçekli(true)
                .etiket_biçimleyici("{value} cm")
                .bölme_çizgisi_göster(false),
        )
        .y_ekseni(
            Eksen::değer()
                .ölçekli(true)
                .etiket_biçimleyici("{value} kg")
                .bölme_çizgisi_göster(false),
        )
        .seri(seri("Female", kadın, 160.0, "AVG"))
        .seri(seri("Male", erkek, 170.0, "Average")))
}

fn candlestick_simple() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(["2017-10-24", "2017-10-25", "2017-10-26", "2017-10-27"]))
        .y_ekseni(Eksen::değer())
        .seri(MumSerisi::yeni().veri([
            [20.0, 34.0, 10.0, 38.0],
            [40.0, 35.0, 30.0, 50.0],
            [31.0, 38.0, 33.0, 44.0],
            [38.0, 15.0, 5.0, 42.0],
        ]))
}

fn heatmap_cartesian(seçili_aralık: bool) -> GrafikSeçenekleri {
    const SAATLER: [&str; 24] = [
        "12a", "1a", "2a", "3a", "4a", "5a", "6a", "7a", "8a", "9a", "10a", "11a", "12p", "1p",
        "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "10p", "11p",
    ];
    const GÜNLER: [&str; 7] = [
        "Saturday",
        "Friday",
        "Thursday",
        "Wednesday",
        "Tuesday",
        "Monday",
        "Sunday",
    ];
    #[rustfmt::skip]
    const DEĞERLER: [[f64; 24]; 7] = [
        [5.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 4.0, 1.0, 1.0, 3.0, 4.0, 6.0, 4.0, 4.0, 3.0, 3.0, 2.0, 5.0],
        [7.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 2.0, 2.0, 6.0, 9.0, 11.0, 6.0, 7.0, 8.0, 12.0, 5.0, 5.0, 7.0, 2.0],
        [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 2.0, 1.0, 9.0, 8.0, 10.0, 6.0, 5.0, 5.0, 5.0, 7.0, 4.0, 2.0, 4.0],
        [7.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 5.0, 4.0, 7.0, 14.0, 13.0, 12.0, 9.0, 5.0, 5.0, 10.0, 6.0, 4.0, 4.0, 1.0],
        [1.0, 3.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 4.0, 4.0, 2.0, 4.0, 4.0, 14.0, 12.0, 1.0, 8.0, 5.0, 3.0, 7.0, 3.0, 0.0],
        [2.0, 1.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 4.0, 1.0, 5.0, 10.0, 5.0, 7.0, 11.0, 6.0, 0.0, 5.0, 3.0, 4.0, 2.0, 0.0],
        [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 1.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 6.0],
    ];
    let veri = DEĞERLER
        .iter()
        .enumerate()
        .flat_map(|(y, satır)| {
            satır.iter().enumerate().map(move |(x, değer)| {
                // Resmî kaynak `item[2] || '-'` kullanır; sayısal olmayan
                // boş hücreyi heatmap çizicisinin atladığı NaN ile taşırız.
                VeriÖğesi::from([
                    x as f64,
                    y as f64,
                    if *değer == 0.0 { f64::NAN } else { *değer },
                ])
            })
        })
        .collect::<Vec<_>>();

    let mut eşleme = GörselEşleme::yeni()
        .en_az(0.0)
        .en_çok(10.0)
        .hesaplanabilir(true)
        .yön(Yön::Yatay)
        .sol("center")
        .alt("15%");
    if seçili_aralık {
        eşleme = eşleme.seçili_aralık(3.0, 7.0);
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().konum(İpucuKonumu::Üst))
        .ızgara(Izgara::yeni().üst("10%").yükseklik("50%"))
        .x_ekseni(Eksen::kategori().veri(SAATLER).bölme_alanı_göster(true))
        .y_ekseni(Eksen::kategori().veri(GÜNLER).bölme_alanı_göster(true))
        .görsel_eşleme(eşleme)
        .seri(
            IsıHaritasıSerisi::yeni()
                .ad("Punch Card")
                .hücre_boşluğu(0.0)
                .etiket(Etiket::yeni().göster(true))
                .vurgu_öğe_stili(
                    ÖğeStili::yeni()
                        .gölge_bulanıklığı(10.0)
                        .gölge_rengi("rgba(0, 0, 0, 0.5)"),
                )
                .veri(veri),
        )
}

fn heatmap_large_verisi() -> (Vec<String>, Vec<String>, Vec<VeriÖğesi>) {
    // Referans sayfası `Math.random`ı 0x5eed1234 Mulberry32 tohumu ile
    // sabitler; resmî noisejs yardımcısına giden ilk değer burada aynıdır.
    let mut rastgele_tohumu = 0x5eed_1234;
    let gürültü = perlin::Perlin2::yeni(kanıt_rastgele(&mut rastgele_tohumu));
    let mut veri = Vec::with_capacity(201 * 101);
    for x in 0..=200 {
        for y in 0..=100 {
            veri.push(VeriÖğesi::from([
                x as f64,
                y as f64,
                gürültü.değer(x as f64 / 40.0, y as f64 / 20.0) + 0.5,
            ]));
        }
    }
    let x_verisi = (0..=200).map(|değer| değer.to_string()).collect::<Vec<_>>();
    // Kaynak örnek bilerek 100 kategoride durur; y=100 veri satırı eksen
    // clip sınırını doğrulayan taşan son satırdır.
    let y_verisi = (0..100).map(|değer| değer.to_string()).collect::<Vec<_>>();
    (x_verisi, y_verisi, veri)
}

fn heatmap_large() -> GrafikSeçenekleri {
    let (x_verisi, y_verisi, veri) = heatmap_large_verisi();

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni())
        .x_ekseni(Eksen::kategori().veri(x_verisi))
        .y_ekseni(Eksen::kategori().veri(y_verisi))
        .görsel_eşleme(
            GörselEşleme::yeni()
                .en_az(0.0)
                .en_çok(1.0)
                .hesaplanabilir(true)
                .sol(0.0_f32)
                .alt(0.0_f32)
                .renkler([
                    "#313695", "#4575b4", "#74add1", "#abd9e9", "#e0f3f8", "#ffffbf", "#fee090",
                    "#fdae61", "#f46d43", "#d73027", "#a50026",
                ]),
        )
        .seri(
            IsıHaritasıSerisi::yeni()
                .ad("Gaussian")
                .hücre_boşluğu(0.0)
                .vurgu_öğe_stili(
                    ÖğeStili::yeni()
                        .kenarlık_rengi("#333")
                        .kenarlık_kalınlığı(1.0),
                )
                .veri(veri),
        )
}

fn heatmap_large_piecewise(parça_kapalı: bool) -> GrafikSeçenekleri {
    let (x_verisi, y_verisi, veri) = heatmap_large_verisi();
    let mut eşleme = GörselEşleme::yeni()
        .en_az(0.0)
        .en_çok(1.0)
        .hesaplanabilir(true)
        .bölme_sayısı(8)
        .sol("right")
        .üst("center")
        .renkler([
            "#313695", "#4575b4", "#74add1", "#abd9e9", "#e0f3f8", "#ffffbf", "#fee090", "#fdae61",
            "#f46d43", "#d73027", "#a50026",
        ]);
    if parça_kapalı {
        eşleme = eşleme.parça_seçimi(3, false);
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni())
        .ızgara(Izgara::yeni().sol(40).sağ(140))
        .x_ekseni(Eksen::kategori().veri(x_verisi))
        .y_ekseni(Eksen::kategori().veri(y_verisi))
        .görsel_eşleme(eşleme)
        .seri(
            IsıHaritasıSerisi::yeni()
                .ad("Gaussian")
                .hücre_boşluğu(0.0)
                .vurgu_öğe_stili(
                    ÖğeStili::yeni()
                        .kenarlık_rengi("#333")
                        .kenarlık_kalınlığı(1.0),
                )
                .veri(veri),
        )
}

fn calendar_heatmap() -> GrafikSeçenekleri {
    use cizelge::yardimci::takvim::{TakvimAnı, takvimden_ana};

    let başlangıç = takvimden_ana(TakvimAnı {
        yıl: 2016,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let bitiş = takvimden_ana(TakvimAnı {
        yıl: 2017,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let mut tohum = 0x5eed_1234;
    let mut veri = Vec::with_capacity(366);
    let mut zaman = başlangıç;
    while zaman < bitiş {
        veri.push(VeriÖğesi::from([
            zaman,
            (kanıt_rastgele(&mut tohum) * 10_000.0).floor(),
        ]));
        zaman += 86_400_000.0;
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .başlık(
            Başlık::yeni()
                .metin("Daily Step Count")
                .sol("center")
                .üst(30)
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni())
        .görsel_eşleme(
            GörselEşleme::yeni()
                .en_az(0.0)
                .en_çok(10_000.0)
                .bölme_sayısı(5)
                .yön(Yön::Yatay)
                .sol("center")
                .üst(65),
        )
        .takvim(
            TakvimKoordinatı::yıl(2016)
                .sol(30.0)
                .sağ(30)
                .üst(120)
                .hücre_boyutu(None, Some(13.0))
                .yıl_etiketi(Etiket::yeni().göster(false))
                .öğe_stili(ÖğeStili::yeni().kenarlık_kalınlığı(0.5)),
        )
        .seri(TakvimSerisi::yeni(2016).takvim_sırası(0).veri(veri))
}

fn calendar_simple() -> GrafikSeçenekleri {
    use cizelge::yardimci::takvim::{TakvimAnı, takvimden_ana};

    let başlangıç = takvimden_ana(TakvimAnı {
        yıl: 2017,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let bitiş = takvimden_ana(TakvimAnı {
        yıl: 2018,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let mut tohum = 0x5eed_1234;
    let mut veri = Vec::with_capacity(365);
    let mut zaman = başlangıç;
    while zaman < bitiş {
        veri.push(VeriÖğesi::from([
            zaman,
            (kanıt_rastgele(&mut tohum) * 10_000.0).floor(),
        ]));
        zaman += 86_400_000.0;
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .görsel_eşleme(
            GörselEşleme::yeni()
                .göster(false)
                .en_az(0.0)
                .en_çok(10_000.0),
        )
        .takvim(TakvimKoordinatı::yıl(2017))
        .seri(TakvimSerisi::yeni(2017).takvim_sırası(0).veri(veri))
}

fn calendar_vertical() -> GrafikSeçenekleri {
    use cizelge::yardimci::takvim::{TakvimAnı, takvimden_ana};

    let veri = |yıl: i32, tohum: &mut u32| {
        let başlangıç = takvimden_ana(TakvimAnı {
            yıl,
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let bitiş = takvimden_ana(TakvimAnı {
            yıl: yıl + 1,
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let mut sonuç = Vec::with_capacity(366);
        let mut zaman = başlangıç;
        while zaman < bitiş {
            sonuç.push(VeriÖğesi::from([
                zaman,
                (kanıt_rastgele(tohum) * 1000.0).floor(),
            ]));
            zaman += 86_400_000.0;
        }
        sonuç
    };
    let mut tohum = 0x5eed_1234;
    let veri_2015 = veri(2015, &mut tohum);
    let veri_2016 = veri(2016, &mut tohum);
    let veri_2017 = veri(2017, &mut tohum);

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .ipucu(İpucu::yeni().konum(İpucuKonumu::Üst))
        .görsel_eşleme(
            GörselEşleme::yeni()
                .en_az(0.0)
                .en_çok(1000.0)
                .hesaplanabilir(true)
                .yön(Yön::Dikey)
                .sol("670")
                .üst("center"),
        )
        .takvim(TakvimKoordinatı::yıl(2015).yön(TakvimYönü::Dikey))
        .takvim(
            TakvimKoordinatı::yıl(2016)
                .sol(300.0)
                .yön(TakvimYönü::Dikey),
        )
        .takvim(
            TakvimKoordinatı::yıl(2017)
                .sol(520.0)
                .alt(10)
                .hücre_boyutu(Some(20.0), None)
                .gün_etiketi_kenar_boşluğu(5)
                .yön(TakvimYönü::Dikey),
        )
        .seri(TakvimSerisi::yeni(2015).takvim_sırası(0).veri(veri_2015))
        .seri(TakvimSerisi::yeni(2016).takvim_sırası(1).veri(veri_2016))
        .seri(TakvimSerisi::yeni(2017).takvim_sırası(2).veri(veri_2017))
}

fn calendar_horizontal() -> GrafikSeçenekleri {
    use cizelge::yardimci::takvim::{TakvimAnı, takvimden_ana};

    let veri = |yıl: i32, tohum: &mut u32| {
        let başlangıç = takvimden_ana(TakvimAnı {
            yıl,
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let bitiş = takvimden_ana(TakvimAnı {
            yıl: yıl + 1,
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let mut sonuç = Vec::with_capacity(366);
        let mut zaman = başlangıç;
        while zaman < bitiş {
            sonuç.push(VeriÖğesi::from([
                zaman,
                (kanıt_rastgele(tohum) * 1000.0).floor(),
            ]));
            zaman += 86_400_000.0;
        }
        sonuç
    };
    let mut tohum = 0x5eed_1234;
    let veri_2017 = veri(2017, &mut tohum);
    let veri_2016 = veri(2016, &mut tohum);
    let veri_2015 = veri(2015, &mut tohum);

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .ipucu(İpucu::yeni().konum(İpucuKonumu::Üst))
        .görsel_eşleme(
            GörselEşleme::yeni()
                .en_az(0.0)
                .en_çok(1000.0)
                .hesaplanabilir(true)
                .yön(Yön::Yatay)
                .sol("center")
                .üst("top"),
        )
        .takvim(TakvimKoordinatı::yıl(2017).hücre_boyutu(None, Some(20.0)))
        .takvim(
            TakvimKoordinatı::yıl(2016)
                .üst(260)
                .hücre_boyutu(None, Some(20.0)),
        )
        .takvim(
            TakvimKoordinatı::yıl(2015)
                .üst(450)
                .sağ(5)
                .hücre_boyutu(None, Some(20.0)),
        )
        .seri(TakvimSerisi::yeni(2017).takvim_sırası(0).veri(veri_2017))
        .seri(TakvimSerisi::yeni(2016).takvim_sırası(1).veri(veri_2016))
        .seri(TakvimSerisi::yeni(2015).takvim_sırası(2).veri(veri_2015))
}

fn calendar_effectscatter() -> GrafikSeçenekleri {
    use cizelge::yardimci::takvim::{TakvimAnı, takvimden_ana};

    let tarih = |ay, gün| {
        takvimden_ana(TakvimAnı {
            yıl: 2016,
            ay,
            gün,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        })
    };
    let başlangıç = tarih(1, 1);
    let bitiş_sonrası = takvimden_ana(TakvimAnı {
        yıl: 2017,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let mut tohum = 0x5eed_1234;
    let mut veri = Vec::with_capacity(366);
    let mut zaman = başlangıç;
    while zaman < bitiş_sonrası {
        veri.push(VeriÖğesi::from([
            zaman,
            (kanıt_rastgele(&mut tohum) * 10_000.0).floor(),
        ]));
        zaman += 86_400_000.0;
    }
    // Resmî örnekte `Array.sort` aynı veri dizisini yerinde sıralar. Takvim
    // koordinatı tarih değerini kullandığı için tam scatter geometrisi bundan
    // etkilenmez; Top 12 için kararlı azalan bir kopya eşdeğer sonucu verir.
    let mut en_yüksekler = veri.clone();
    en_yüksekler.sort_by(|a, b| {
        b.değer
            .sayı()
            .partial_cmp(&a.değer.sayı())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    en_yüksekler.truncate(12);

    let takvim = |üst, başlangıç, bitiş, dönem: &str| {
        TakvimKoordinatı::yeni(TakvimAralığı::yeni(başlangıç, bitiş))
            .üst(üst)
            .sol("center")
            .ayırıcı(ÇizgiStili::yeni().renk("#000").kalınlık(4.0))
            .yıl_etiketi(
                Etiket::yeni()
                    .göster(true)
                    .biçimleyici(format!("{{start}}  {dönem}"))
                    .yazı(YazıStili::yeni().renk("#fff").boyut(20.0).kalın(true)),
            )
            .ay_etiketi(
                Etiket::yeni()
                    .göster(true)
                    .yazı(YazıStili::yeni().renk("#aaa")),
            )
            .gün_etiketi(
                Etiket::yeni()
                    .göster(true)
                    .yazı(YazıStili::yeni().renk("#aaa")),
            )
            .öğe_stili(
                ÖğeStili::yeni()
                    .renk("#323c48")
                    .kenarlık_kalınlığı(1.0)
                    .kenarlık_rengi("#111"),
            )
    };
    let adım_serisi = |takvim_sırası| {
        SaçılımSerisi::yeni()
            .ad("Steps")
            .takvim_sırası(takvim_sırası)
            .sembol_boyutu_işlevi(|öğe| öğe.değer.sayı().unwrap_or_default() as f32 / 500.0)
            .öğe_stili(ÖğeStili::yeni().renk("#ddb926"))
            .veri(veri.clone())
    };
    let üst_serisi = |takvim_sırası| {
        SaçılımSerisi::yeni()
            .ad("Top 12")
            .takvim_sırası(takvim_sırası)
            .z_seviyesi(1)
            .sembol_boyutu_işlevi(|öğe| öğe.değer.sayı().unwrap_or_default() as f32 / 500.0)
            .efektli(true)
            .efekt_vuruşlu(true)
            .öğe_stili(
                ÖğeStili::yeni()
                    .renk("#f4e925")
                    .gölge_bulanıklığı(10.0)
                    .gölge_rengi("#333"),
            )
            .veri(en_yüksekler.clone())
    };

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .arkaplan("#404a59")
        .başlık(
            Başlık::yeni()
                .metin("Daily Step Count in 2016")
                .alt_metin("Fake Data")
                .üst(30)
                .sol("center")
                .iç_boşluk(15.0)
                .yazı(YazıStili::yeni().renk("#fff")),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(
            Gösterge::yeni()
                .üst(30)
                .sol(100.0)
                .veri(["Steps", "Top 12"])
                .iç_boşluk(15.0)
                .yazı(YazıStili::yeni().renk("#fff")),
        )
        .takvim(takvim(120, tarih(1, 1), tarih(6, 30), "1st"))
        .takvim(takvim(340, tarih(7, 1), tarih(12, 31), "2nd"))
        .seri(adım_serisi(0))
        .seri(adım_serisi(1))
        .seri(üst_serisi(1))
        .seri(üst_serisi(0))
}

fn calendar_graph() -> GrafikSeçenekleri {
    use cizelge::yardimci::takvim::{TakvimAnı, takvimden_ana};

    let tarih = |ay, gün| {
        takvimden_ana(TakvimAnı {
            yıl: 2017,
            ay,
            gün,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        })
    };
    let graph_verisi = [
        (2, 1, 260.0),
        (2, 4, 200.0),
        (2, 9, 279.0),
        (2, 13, 847.0),
        (2, 18, 241.0),
        (2, 23, 411.0),
        (3, 14, 985.0),
    ];
    let düğümler = graph_verisi
        .iter()
        .map(|(ay, gün, değer)| {
            let zaman = tarih(*ay, *gün);
            GrafoDüğümü::yeni(format!("2017-{ay:02}-{gün:02}"), 15.0)
                .değerli(*değer)
                .takvim_tarihi(zaman)
        })
        .collect::<Vec<_>>();
    let bağlar = düğümler
        .windows(2)
        .map(|çift| (çift[0].ad.clone(), çift[1].ad.clone()))
        .collect::<Vec<_>>();

    let başlangıç = tarih(1, 1);
    let bitiş_sonrası = takvimden_ana(TakvimAnı {
        yıl: 2018,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let mut tohum = 0x5eed_1234;
    let mut ısı_verisi = Vec::with_capacity(365);
    let mut zaman = başlangıç;
    while zaman < bitiş_sonrası {
        ısı_verisi.push(VeriÖğesi::from([
            zaman,
            (kanıt_rastgele(&mut tohum) * 1000.0).floor(),
        ]));
        zaman += 86_400_000.0;
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .ipucu(İpucu::yeni())
        .görsel_eşleme(
            GörselEşleme::yeni()
                .en_az(0.0)
                .en_çok(1000.0)
                .bölme_sayısı(5)
                .renkler(["#5291FF", "#C7DBFF"])
                .seri_sırası(1)
                .yön(Yön::Yatay)
                .sol("center")
                .alt(20),
        )
        .takvim(
            TakvimKoordinatı::yeni(TakvimAralığı::yeni(tarih(2, 1), tarih(3, 31)))
                .üst("middle")
                .sol("center")
                .yön(TakvimYönü::Dikey)
                .ilk_gün(1)
                .hücre_boyutu(Some(40.0), Some(40.0))
                .yıl_etiketi(
                    Etiket::yeni()
                        .göster(true)
                        .yazı(YazıStili::yeni().boyut(30.0).kalın(true)),
                )
                .yıl_etiketi_kenar_boşluğu(50.0)
                .ay_etiketi(
                    Etiket::yeni()
                        .göster(true)
                        .yazı(YazıStili::yeni().boyut(20.0).renk("#999")),
                )
                .ay_etiketi_kenar_boşluğu(15.0),
        )
        .seri(
            GrafoSerisi::yeni()
                .takvim_sırası(0)
                .z(20)
                .hedef_oku(true)
                .öğe_stili(
                    ÖğeStili::yeni()
                        .renk("yellow")
                        .gölge_bulanıklığı(9.0)
                        .gölge_kayması(1.5, 3.0)
                        .gölge_rengi("#555"),
                )
                .çizgi_stili(
                    ÇizgiStili::yeni()
                        .renk("#D10E00")
                        .kalınlık(1.0)
                        .opaklık(1.0),
                )
                .düğümler(düğümler)
                .bağlar(bağlar),
        )
        .seri(TakvimSerisi::yeni(2017).takvim_sırası(0).veri(ısı_verisi))
}

fn calendar_lunar() -> GrafikSeçenekleri {
    use cizelge::yardimci::takvim::{TakvimAnı, takvimden_ana};

    let tarih = |ay, gün| {
        takvimden_ana(TakvimAnı {
            yıl: 2017,
            ay,
            gün,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        })
    };
    let mart = [
        ("初四", None),
        ("初五", None),
        ("初六", None),
        ("初七", None),
        ("初八", Some("驚蟄")),
        ("初九", None),
        ("初十", None),
        ("十一", None),
        ("十二", None),
        ("十三", None),
        ("十四", None),
        ("十五", None),
        ("十六", None),
        ("十七", None),
        ("十八", None),
        ("十九", None),
        ("二十", None),
        ("廿一", None),
        ("廿二", None),
        ("廿三", Some("春分")),
        ("廿四", None),
        ("廿五", None),
        ("廿六", None),
        ("廿七", None),
        ("廿八", None),
        ("廿九", None),
        ("三十", None),
        ("三月", None),
        ("初二", None),
        ("初三", None),
        ("初四", None),
    ];
    let ay_verisi = mart
        .iter()
        .enumerate()
        .map(|(sıra, (ay_günü, güneş_terimi))| {
            let gün = sıra + 1;
            VeriÖğesi::from([tarih(3, gün as u32), 1.0]).boyutlar([
                (
                    "ay_etiketi".to_string(),
                    format!("{gün}\n\n{ay_günü}\n\n").into(),
                ),
                (
                    "güneş_terimi".to_string(),
                    format!("\n\n\n{}", güneş_terimi.unwrap_or("")).into(),
                ),
            ])
        })
        .collect::<Vec<_>>();

    let başlangıç = tarih(1, 1);
    let bitiş_sonrası = takvimden_ana(TakvimAnı {
        yıl: 2018,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let mut tohum = 0x5eed_1234;
    let mut ısı_verisi = Vec::with_capacity(365);
    let mut zaman = başlangıç;
    while zaman < bitiş_sonrası {
        ısı_verisi.push(VeriÖğesi::from(
            [zaman, kanıt_rastgele(&mut tohum) * 300.0],
        ));
        zaman += 86_400_000.0;
    }

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .ipucu(
            İpucu::yeni().biçimleyici(Biçimleyici::İşlev(Arc::new(|_, ham| {
                let değer = ham.parse::<f64>().unwrap_or_default();
                format!("降雨量: {değer:.2}")
            }))),
        )
        .görsel_eşleme(
            GörselEşleme::yeni()
                .göster(false)
                .en_az(0.0)
                .en_çok(300.0)
                .hesaplanabilir(true)
                .seri_sırası(2)
                .yön(Yön::Yatay)
                .sol("center")
                .alt(20)
                .renkler(["#e0ffff", "#006edd"])
                .opaklık(0.3),
        )
        .takvim(
            TakvimKoordinatı::yeni(TakvimAralığı::yeni(tarih(3, 1), tarih(3, 31)))
                .sol("center")
                .üst("middle")
                .hücre_boyutu(Some(70.0), Some(70.0))
                .yön(TakvimYönü::Dikey)
                .ilk_gün(1)
                .yıl_etiketi(Etiket::yeni().göster(false))
                .ay_etiketi(Etiket::yeni().göster(false)),
        )
        .seri(
            SaçılımSerisi::yeni()
                .takvim_sırası(0)
                .sembol_boyutu(0.0)
                .etiket_boyutunu_eşle("ay_etiketi")
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .yazı(YazıStili::yeni().renk("#000")),
                )
                .sessiz(true)
                .veri(ay_verisi.clone()),
        )
        .seri(
            SaçılımSerisi::yeni()
                .takvim_sırası(0)
                .sembol_boyutu(0.0)
                .etiket_boyutunu_eşle("güneş_terimi")
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .yazı(YazıStili::yeni().boyut(14.0).kalın(true).renk("#a00")),
                )
                .sessiz(true)
                .veri(ay_verisi),
        )
        .seri(
            TakvimSerisi::yeni(2017)
                .ad("降雨量")
                .takvim_sırası(0)
                .veri(ısı_verisi),
        )
}

fn calendar_pie() -> GrafikSeçenekleri {
    use cizelge::yardimci::takvim::{TakvimAnı, takvimden_ana};

    let tarih = |gün| {
        takvimden_ana(TakvimAnı {
            yıl: 2017,
            ay: 2,
            gün,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        })
    };
    let tarihler = (1..=28).map(tarih).collect::<Vec<_>>();
    let mut tohum = 0x5eed_1234;

    // Resmî kaynak önce getVirtualData ile 28 rastgele scatter değeri,
    // ardından her gün için üç pasta değeri tüketir.
    let scatter_verisi = tarihler
        .iter()
        .enumerate()
        .map(|(sıra, zaman)| {
            VeriÖğesi::from([*zaman, (kanıt_rastgele(&mut tohum) * 10_000.0).floor()])
                .boyutlar([("gün".to_string(), format!("{:02}", sıra + 1).into())])
        })
        .collect::<Vec<_>>();
    let pasta_serileri = tarihler
        .iter()
        .map(|zaman| {
            let iş = (kanıt_rastgele(&mut tohum) * 24.0).round();
            let eğlence = (kanıt_rastgele(&mut tohum) * 24.0).round();
            let uyku = (kanıt_rastgele(&mut tohum) * 24.0).round();
            PastaSerisi::yeni()
                .takvim_merkezi(*zaman)
                .yarıçap(30.0)
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .konum(EtiketKonumu::İç)
                        .biçimleyici("{c}"),
                )
                .veri([
                    VeriÖğesi::adlı("Work", iş),
                    VeriÖğesi::adlı("Entertainment", eğlence),
                    VeriÖğesi::adlı("Sleep", uyku),
                ])
        })
        .collect::<Vec<_>>();

    let mut seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .ipucu(İpucu::yeni())
        .gösterge(
            Gösterge::yeni()
                .alt(20)
                .iç_boşluk(15.0)
                .veri(["Work", "Entertainment", "Sleep"]),
        )
        .takvim(
            TakvimKoordinatı::yeni(TakvimAralığı::yeni(tarih(1), tarih(28)))
                .sol("center")
                .üst("middle")
                .yön(TakvimYönü::Dikey)
                .ilk_gün(1)
                .hücre_boyutu(Some(80.0), Some(80.0))
                .yıl_etiketi(
                    Etiket::yeni()
                        .göster(false)
                        .yazı(YazıStili::yeni().boyut(30.0)),
                )
                .gün_etiketi_kenar_boşluğu(20.0)
                .gün_adları(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"])
                .ay_etiketi(Etiket::yeni().göster(false)),
        )
        .seri(
            SaçılımSerisi::yeni()
                .takvim_sırası(0)
                .sembol_boyutu(0.0)
                .etiket_boyutunu_eşle("gün")
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .kayma(-30.0, -30.0)
                        .yazı(YazıStili::yeni().boyut(14.0)),
                )
                .veri(scatter_verisi),
        );
    for pasta in pasta_serileri {
        seçenekler = seçenekler.seri(pasta);
    }
    seçenekler
}

fn custom_calendar_icon() -> Result<GrafikSeçenekleri, String> {
    use cizelge::cizim::{DikeyHiza, YatayHiza};
    use cizelge::yardimci::takvim::{TakvimAnı, andan_takvime, takvimden_ana};
    use custom_calendar_icon_verisi::{RENKLER, YERLEŞİMLER, YOLLAR};

    // zrender `makePath(..., layout: 'center')`: kaynak yolun gerçek sınır
    // kutusunu oranı bozmadan 16×16 hedefe sığdırıp merkezler.
    let ikon_yolları = YOLLAR
        .iter()
        .map(|path_data| {
            let yol = Yol::svg_path_data(path_data).map_err(|hata| hata.to_string())?;
            let kutu = yol
                .kesin_sınır_kutusu()
                .ok_or_else(|| "custom calendar SVG yolu boş".to_owned())?;
            if kutu.genişlik <= 0.0 || kutu.yükseklik <= 0.0 {
                return Err("custom calendar SVG yolunun boyutu geçersiz".to_owned());
            }
            let ölçek = (16.0 / kutu.genişlik).min(16.0 / kutu.yükseklik);
            let genişlik = kutu.genişlik * ölçek;
            let yükseklik = kutu.yükseklik * ölçek;
            let hedef_x = -8.0 + (16.0 - genişlik) / 2.0;
            let hedef_y = -8.0 + (16.0 - yükseklik) / 2.0;
            let dönüşüm = AfinMatris::ötele(hedef_x, hedef_y)
                .çarp(AfinMatris::ölçekle(ölçek, ölçek))
                .çarp(AfinMatris::ötele(-kutu.x, -kutu.y));
            Ok(yolu_dönüştür(&yol, dönüşüm))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let ikon_yolları = Arc::new(ikon_yolları);

    let başlangıç = takvimden_ana(TakvimAnı {
        yıl: 2017,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let bitiş = takvimden_ana(TakvimAnı {
        yıl: 2018,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let mart_başı = takvimden_ana(TakvimAnı {
        yıl: 2017,
        ay: 3,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let mart_sonu = takvimden_ana(TakvimAnı {
        yıl: 2017,
        ay: 3,
        gün: 31,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });

    // Resmî getVirtulData bütün 2017 boyunca aynı Math.random akışını
    // tüketir; Calendar yalnız Mart hücrelerini görünür kılar.
    let mut tohum = 0x5eed_1234;
    let mut veri = Vec::with_capacity(365);
    let mut zaman = başlangıç;
    while zaman < bitiş {
        let etkinlik_sayısı = (kanıt_rastgele(&mut tohum) * 4.0).round() as usize;
        let etkinlikler = (0..etkinlik_sayısı)
            .map(|_| (kanıt_rastgele(&mut tohum) * 3.0).round() as usize)
            .map(|sıra| sıra.to_string())
            .collect::<Vec<_>>()
            .join("|");
        let gün = andan_takvime(zaman).gün;
        veri.push(VeriÖğesi::from([zaman, 0.0]).boyutlar([
            ("etkinlikler".to_owned(), etkinlikler.into()),
            ("gün".to_owned(), format!("{gün:02}").into()),
        ]));
        zaman += 86_400_000.0;
    }

    let çizim_yolları = Arc::clone(&ikon_yolları);
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .ipucu(İpucu::yeni())
        .takvim(
            TakvimKoordinatı::yeni(TakvimAralığı::yeni(mart_başı, mart_sonu))
                .sol("center")
                .üst("middle")
                .hücre_boyutu(Some(70.0), Some(70.0))
                .yön(TakvimYönü::Dikey)
                .ilk_gün(1)
                .gün_adları(["S", "M", "T", "W", "T", "F", "S"])
                .yıl_etiketi(Etiket::yeni().göster(false))
                .ay_etiketi(Etiket::yeni().göster(false)),
        )
        .seri(
            ÖzelSeri::yeni()
                .takvim_sırası(0)
                .veri(veri)
                .çizim(move |yüzey, bağlam| {
                    let Some(takvim) = bağlam.takvim else {
                        return;
                    };
                    for öğe in bağlam.veri {
                        let Some(zaman) = öğe.değer.x() else {
                            continue;
                        };
                        let Some(hücre) = takvim.hücre(zaman) else {
                            continue;
                        };
                        let merkez = hücre.merkez();
                        let etkinlikler = match öğe.boyut("etkinlikler") {
                            Some(VeriDeğeri::Metin(metin)) if !metin.is_empty() => metin
                                .split('|')
                                .filter_map(|ham| ham.parse::<usize>().ok())
                                .collect::<Vec<_>>(),
                            _ => Vec::new(),
                        };
                        if let Some(yerleşim) = etkinlikler
                            .len()
                            .checked_sub(1)
                            .and_then(|sıra| YERLEŞİMLER.get(sıra))
                        {
                            for ((x_oranı, y_oranı), ikon_sırası) in
                                yerleşim.iter().zip(&etkinlikler)
                            {
                                let x = merkez.0 + x_oranı * takvim.hücre_genişliği;
                                let alt = -takvim.hücre_yüksekliği / 2.0 + 20.0;
                                let üst = takvim.hücre_yüksekliği / 2.0;
                                let y = merkez.1 + alt + (y_oranı + 0.5) * (üst - alt);
                                let Some(ikon) = çizim_yolları.get(*ikon_sırası) else {
                                    continue;
                                };
                                let yol = yolu_dönüştür(ikon, AfinMatris::ötele(x, y));
                                yüzey.yol_doldur(
                                    &yol,
                                    &Dolgu::Düz(Renk::from(
                                        RENKLER.get(*ikon_sırası).copied().unwrap_or("#000"),
                                    )),
                                );
                            }
                        }
                        let gün = match öğe.boyut("gün") {
                            Some(VeriDeğeri::Metin(gün)) => gün.as_str(),
                            _ => "",
                        };
                        yüzey.yazı(
                            gün,
                            (merkez.0, merkez.1 - takvim.hücre_yüksekliği / 2.0 + 15.0),
                            YatayHiza::Sol,
                            DikeyHiza::Üst,
                            14.0,
                            Renk::from("#777"),
                            false,
                        );
                    }
                }),
        ))
}

fn calendar_charts() -> GrafikSeçenekleri {
    use cizelge::yardimci::takvim::{TakvimAnı, takvimden_ana};

    let tarih = |ay, gün| {
        takvimden_ana(TakvimAnı {
            yıl: 2017,
            ay,
            gün,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        })
    };
    let yıl_başı = tarih(1, 1);
    let yıl_sonu = takvimden_ana(TakvimAnı {
        yıl: 2018,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let mut tohum = 0x5eed_1234;
    let mut sanal_veri = || {
        let mut sonuç = Vec::with_capacity(365);
        let mut zaman = yıl_başı;
        while zaman < yıl_sonu {
            sonuç.push(VeriÖğesi::from([
                zaman,
                (kanıt_rastgele(&mut tohum) * 1000.0).floor(),
            ]));
            zaman += 86_400_000.0;
        }
        sonuç
    };
    // Kaynak, dört getVirtualData çağrısını seri tanım sırasıyla tüketir.
    let şubat_ısı_verisi = sanal_veri();
    let ocak_efekt_verisi = sanal_veri();
    let mart_saçılım_verisi = sanal_veri();
    let nisan_ısı_verisi = sanal_veri();

    let graph_verisi = [
        (2, 1, 260.0),
        (2, 4, 200.0),
        (2, 9, 279.0),
        (2, 13, 847.0),
        (2, 18, 241.0),
        (2, 23, 411.0),
        (2, 27, 985.0),
    ];
    let düğümler = graph_verisi
        .iter()
        .map(|(ay, gün, değer)| {
            GrafoDüğümü::yeni(format!("2017-{ay:02}-{gün:02}"), 10.0)
                .değerli(*değer)
                .takvim_tarihi(tarih(*ay, *gün))
        })
        .collect::<Vec<_>>();
    let bağlar = düğümler
        .windows(2)
        .map(|çift| (çift[0].ad.clone(), çift[1].ad.clone()))
        .collect::<Vec<_>>();

    let takvim = |ay, son_gün, sol: Option<f32>, üst: Option<f32>, ilk_gün, tam_gün_adı| {
        let mut koordinat =
            TakvimKoordinatı::yeni(TakvimAralığı::yeni(tarih(ay, 1), tarih(ay, son_gün)))
                .yön(TakvimYönü::Dikey)
                .hücre_boyutu(Some(40.0), Some(40.0))
                .ilk_gün(ilk_gün)
                .yıl_etiketi_kenar_boşluğu(40.0)
                .ay_etiketi_kenar_boşluğu(20.0);
        if let Some(sol) = sol {
            koordinat = koordinat.sol(sol);
        }
        if let Some(üst) = üst {
            koordinat = koordinat.üst(üst);
        }
        if tam_gün_adı {
            koordinat = koordinat.gün_adları(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);
        }
        koordinat
    };

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .yerel(&İNGİLİZCE)
        .ipucu(İpucu::yeni().konum(İpucuKonumu::Üst))
        .görsel_eşlemeler([
            GörselEşleme::yeni()
                .en_az(0.0)
                .en_çok(1000.0)
                .hesaplanabilir(true)
                .seri_sıraları([2, 3, 4])
                .yön(Yön::Yatay)
                .sol("55%")
                .alt(20),
            GörselEşleme::yeni()
                .en_az(0.0)
                .en_çok(1000.0)
                .renkler(["grey"])
                .opaklık_aralığı(0.0, 0.3)
                .denetleyici_opaklık_aralığı(0.3, 0.6)
                .denetleyici_aralık_dışı_renk("#ccc")
                .seri_sırası(1)
                .yön(Yön::Yatay)
                .sol("10%")
                .alt(20),
        ])
        .takvim(takvim(2, 28, None, None, 1, false))
        .takvim(takvim(1, 31, Some(460.0), None, 0, false))
        .takvim(takvim(3, 31, None, Some(350.0), 0, false))
        .takvim(takvim(4, 30, Some(460.0), Some(350.0), 1, true))
        .seri(
            GrafoSerisi::yeni()
                .takvim_sırası(0)
                .hedef_oku(true)
                .düğümler(düğümler)
                .bağlar(bağlar),
        )
        .seri(
            TakvimSerisi::yeni(2017)
                .takvim_sırası(0)
                .veri(şubat_ısı_verisi),
        )
        .seri(
            SaçılımSerisi::yeni()
                .takvim_sırası(1)
                .efektli(true)
                .sembol_boyutu_işlevi(|öğe| öğe.değer.sayı().unwrap_or_default() as f32 / 40.0)
                .veri(ocak_efekt_verisi),
        )
        .seri(
            SaçılımSerisi::yeni()
                .takvim_sırası(2)
                .sembol_boyutu_işlevi(|öğe| öğe.değer.sayı().unwrap_or_default() as f32 / 60.0)
                .veri(mart_saçılım_verisi),
        )
        .seri(
            TakvimSerisi::yeni(2017)
                .takvim_sırası(3)
                .veri(nisan_ısı_verisi),
        )
}

fn pie_simple() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Referer of a Website")
                .alt_metin("Fake Data")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().yön(Yön::Dikey).sol("left").iç_boşluk(15.0))
        .seri(PastaSerisi::yeni().ad("Access From").yarıçap("50%").veri([
            VeriÖğesi::adlı("Search Engine", 1048),
            VeriÖğesi::adlı("Direct", 735),
            VeriÖğesi::adlı("Email", 580),
            VeriÖğesi::adlı("Union Ads", 484),
            VeriÖğesi::adlı("Video Ads", 300),
        ]))
}

fn pie_nest() -> GrafikSeçenekleri {
    let dış_etiket = Etiket::yeni()
        .göster(true)
        .konum(EtiketKonumu::Dış)
        .biçimleyici("{a|{a}}{abg|}\n{hr|}\n  {b|{b}：}{c}  {per|{d}%}  ")
        .yazı(
            YazıStili::yeni()
                .arkaplan("#F6F8FC")
                .kenarlık_rengi("#8C8D8E")
                .kenarlık_kalınlığı(1.0)
                .kenarlık_yarıçapı(4.0),
        )
        .zengin_stil(
            "a",
            YazıStili::yeni()
                .renk("#6E7079")
                .satır_yüksekliği(22.0)
                .yatay_hiza(YazıYatayHizası::Orta),
        )
        .zengin_stil(
            "hr",
            YazıStili::yeni()
                .kenarlık_rengi("#8C8D8E")
                .kenarlık_kalınlığı(1.0)
                .genişlik("100%")
                .yükseklik(0.0),
        )
        .zengin_stil(
            "b",
            YazıStili::yeni()
                .renk("#4C5058")
                .boyut(14.0)
                .kalın(true)
                .satır_yüksekliği(33.0),
        )
        .zengin_stil(
            "per",
            YazıStili::yeni()
                .renk("#fff")
                .arkaplan("#4C5058")
                .iç_boşluk([3.0, 4.0, 3.0, 4.0])
                .kenarlık_yarıçapı(4.0),
        );

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Öğe)
                .biçimleyici("{a} <br/>{b}: {c} ({d}%)"),
        )
        .gösterge(Gösterge::yeni().iç_boşluk(15.0).veri([
            "Direct",
            "Marketing",
            "Search Engine",
            "Email",
            "Union Ads",
            "Video Ads",
            "Baidu",
            "Google",
            "Bing",
            "Others",
        ]))
        .seri(
            PastaSerisi::yeni()
                .ad("Access From")
                .halka(0, "30%")
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .konum(EtiketKonumu::İç)
                        .yazı(YazıStili::yeni().boyut(14.0)),
                )
                .etiket_çizgisi(EtiketÇizgisi::yeni().göster(false))
                .veri([
                    VeriÖğesi::adlı("Search Engine", 1548),
                    VeriÖğesi::adlı("Direct", 775),
                    VeriÖğesi::adlı("Marketing", 679).seçili(true),
                ]),
        )
        .seri(
            PastaSerisi::yeni()
                .ad("Access From")
                .halka("45%", "60%")
                .etiket(dış_etiket)
                .etiket_çizgisi(EtiketÇizgisi::yeni().uzunluk1(30.0))
                .veri([
                    VeriÖğesi::adlı("Baidu", 1048),
                    VeriÖğesi::adlı("Direct", 335),
                    VeriÖğesi::adlı("Email", 310),
                    VeriÖğesi::adlı("Google", 251),
                    VeriÖğesi::adlı("Union Ads", 234),
                    VeriÖğesi::adlı("Bing", 147),
                    VeriÖğesi::adlı("Video Ads", 135),
                    VeriÖğesi::adlı("Others", 102),
                ]),
        )
}

fn hava_ikonu(ad: &str) -> Result<GörüntüDeseni, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/data/asset/img/weather")
        .join(format!("{ad}_128.png"));
    let rgba = image::open(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?
        .to_rgba8();
    let (genişlik, yükseklik) = (rgba.width(), rgba.height());
    GörüntüDeseni::rgba(genişlik, yükseklik, rgba.into_raw(), DesenTekrarı::Sığdır)
        .ok_or_else(|| format!("{} RGBA boyutları geçersiz", dosya.display()))
}

fn pie_rich_text() -> Result<GrafikSeçenekleri, String> {
    let güneşli = hava_ikonu("sunny")?;
    let bulutlu = hava_ikonu("cloudy")?;
    let sağanak = hava_ikonu("showers")?;
    let şehir_e_etiketi = EtiketYaması::yeni()
        .biçimleyici(
            "{title|{b}}{abg|}\n  {weatherHead|Weather}{valueHead|Days}{rateHead|Percent}\n{hr|}\n  {Sunny|}{value|202}{rate|55.3%}\n  {Cloudy|}{value|142}{rate|38.9%}\n  {Showers|}{value|21}{rate|5.8%}",
        )
        .yazı(
            YazıStili::yeni()
                .arkaplan("#eee")
                .kenarlık_rengi("#777")
                .kenarlık_kalınlığı(1.0)
                .kenarlık_yarıçapı(4.0),
        )
        .zengin_stil(
            "title",
            YazıStili::yeni()
                .renk("#eee")
                .yatay_hiza(YazıYatayHizası::Orta),
        )
        .zengin_stil(
            "abg",
            YazıStili::yeni()
                .arkaplan("#333")
                .genişlik("100%")
                .yatay_hiza(YazıYatayHizası::Sağ)
                .yükseklik(25.0)
                .kenarlık_yarıçapları([4.0, 4.0, 0.0, 0.0]),
        )
        .zengin_stil(
            "Sunny",
            YazıStili::yeni()
                .yükseklik(30.0)
                .yatay_hiza(YazıYatayHizası::Sol)
                .arkaplan(Dolgu::Desen(güneşli)),
        )
        .zengin_stil(
            "Cloudy",
            YazıStili::yeni()
                .yükseklik(30.0)
                .yatay_hiza(YazıYatayHizası::Sol)
                .arkaplan(Dolgu::Desen(bulutlu)),
        )
        .zengin_stil(
            "Showers",
            YazıStili::yeni()
                .yükseklik(30.0)
                .yatay_hiza(YazıYatayHizası::Sol)
                .arkaplan(Dolgu::Desen(sağanak)),
        )
        .zengin_stil(
            "weatherHead",
            YazıStili::yeni()
                .renk("#333")
                .yükseklik(24.0)
                .yatay_hiza(YazıYatayHizası::Sol),
        )
        .zengin_stil(
            "hr",
            YazıStili::yeni()
                .kenarlık_rengi("#777")
                .kenarlık_kalınlığı(0.5)
                .genişlik("100%")
                .yükseklik(0.0),
        )
        .zengin_stil(
            "value",
            YazıStili::yeni()
                .genişlik(20.0)
                .iç_boşluk([0.0, 20.0, 0.0, 30.0])
                .yatay_hiza(YazıYatayHizası::Sol),
        )
        .zengin_stil(
            "valueHead",
            YazıStili::yeni()
                .renk("#333")
                .genişlik(20.0)
                .iç_boşluk([0.0, 20.0, 0.0, 30.0])
                .yatay_hiza(YazıYatayHizası::Orta),
        )
        .zengin_stil(
            "rate",
            YazıStili::yeni()
                .genişlik(40.0)
                .yatay_hiza(YazıYatayHizası::Sağ)
                .iç_boşluk([0.0, 10.0, 0.0, 0.0]),
        )
        .zengin_stil(
            "rateHead",
            YazıStili::yeni()
                .renk("#333")
                .genişlik(40.0)
                .yatay_hiza(YazıYatayHizası::Orta)
                .iç_boşluk([0.0, 10.0, 0.0, 0.0]),
        );

    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Weather Statistics")
                .alt_metin("Fake Data")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Öğe)
                .biçimleyici("{a} <br/>{b} : {c} ({d}%)"),
        )
        .gösterge(
            Gösterge::yeni()
                .alt(10)
                .sol("center")
                .iç_boşluk(15.0)
                .veri(["CityA", "CityB", "CityD", "CityC", "CityE"]),
        )
        .seri(
            PastaSerisi::yeni()
                .yarıçap("65%")
                .merkez("50%", "50%")
                .veri([
                    VeriÖğesi::adlı("CityE", 1548).etiket(şehir_e_etiketi),
                    VeriÖğesi::adlı("CityC", 735),
                    VeriÖğesi::adlı("CityD", 510),
                    VeriÖğesi::adlı("CityB", 434),
                    VeriÖğesi::adlı("CityA", 335),
                ]),
        ))
}

fn pie_doughnut() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst("5%").iç_boşluk(15.0))
        .seri(
            PastaSerisi::yeni()
                .ad("Access From")
                .halka("40%", "70%")
                .etiket_çakışmasını_önle(false)
                .etiket(Etiket::yeni().göster(false).konum(EtiketKonumu::Merkez))
                .etiket_çizgisi(EtiketÇizgisi::yeni().göster(false))
                .veri([
                    VeriÖğesi::adlı("Search Engine", 1048),
                    VeriÖğesi::adlı("Direct", 735),
                    VeriÖğesi::adlı("Email", 580),
                    VeriÖğesi::adlı("Union Ads", 484),
                    VeriÖğesi::adlı("Video Ads", 300),
                ]),
        )
}

fn pie_rose_type_simple() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        // Resmî örnekte `top: 'bottom'` kullanılır; ECharts bunu alt kenara
        // hizalanmış legend olarak çözer. Referans ön işlemcisi padding'i
        // 15 px'e sabitler.
        .gösterge(Gösterge::yeni().alt(0).iç_boşluk(15.0))
        .araç_kutusu(AraçKutusu::yeni().veri_görünümü(true).png_kaydet(true))
        .seri(
            PastaSerisi::yeni()
                .ad("Nightingale Chart")
                .halka(50, 250)
                .merkez("50%", "50%")
                .gül_türü(GülTürü::Alan)
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı(8.0))
                .veri([
                    VeriÖğesi::adlı("rose 1", 40),
                    VeriÖğesi::adlı("rose 2", 38),
                    VeriÖğesi::adlı("rose 3", 32),
                    VeriÖğesi::adlı("rose 4", 30),
                    VeriÖğesi::adlı("rose 5", 28),
                    VeriÖğesi::adlı("rose 6", 26),
                    VeriÖğesi::adlı("rose 7", 22),
                    VeriÖğesi::adlı("rose 8", 18),
                ]),
        )
}

fn pie_rose_type() -> GrafikSeçenekleri {
    let veri = |değerler: [i32; 8]| {
        değerler
            .into_iter()
            .enumerate()
            .map(|(sıra, değer)| VeriÖğesi::adlı(format!("rose {}", sıra + 1), değer))
            .collect::<Vec<_>>()
    };
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("Nightingale Chart")
                .alt_metin("Fake Data")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        // Resmî örnekte boşluksuz adlar, dilim adlarıyla eşleşmediğinden
        // legend sağlayıcısı öğe üretmez; bu kasıtlı uyumsuzluğu da koru.
        .gösterge(Gösterge::yeni().alt(0).iç_boşluk(15.0).veri([
            "rose1", "rose2", "rose3", "rose4", "rose5", "rose6", "rose7", "rose8",
        ]))
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_görünümü(true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .seri(
            PastaSerisi::yeni()
                .ad("Radius Mode")
                .halka(20, 140)
                .merkez("25%", "50%")
                .gül_türü(GülTürü::Yarıçap)
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı(5.0))
                .etiket(Etiket::yeni().göster(false))
                .veri(veri([40, 33, 28, 22, 20, 15, 12, 10])),
        )
        .seri(
            PastaSerisi::yeni()
                .ad("Area Mode")
                .halka(20, 140)
                .merkez("75%", "50%")
                .gül_türü(GülTürü::Alan)
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı(5.0))
                .veri(veri([30, 28, 26, 24, 22, 20, 18, 16])),
        )
}

fn pie_legend() -> GrafikSeçenekleri {
    // Resmî örneğin `genData(50)` çıktısı; referans hattının sabit
    // Mulberry32 akışından üretilmiştir. Böylece hem dilim açıları hem de
    // kaydırmalı legend sayfaları her çalıştırmada aynı kalır.
    let veri = [
        ("魏路孙", 80293),
        ("谈苗屈", 21101),
        ("苗傅", 26151),
        ("汤杜柏", 82671),
        ("尹项韩", 97061),
        ("郝湛", 70500),
        ("范方岑", 44486),
        ("项倪史柏郑·马滕花", 12380),
        ("杜姜", 64425),
        ("熊孙强穆·陈于", 35031),
        ("吴何", 81702),
        ("倪何毛", 54251),
        ("马于", 76448),
        ("任倪", 40070),
        ("舒强·马元岑", 2231),
        ("郎熊·成", 6996),
        ("唐乐顾祝·汤禹", 75615),
        ("施吕", 48373),
        ("陶凤邹", 12303),
        ("熊陶", 66156),
        ("柏彭", 55144),
        ("杜潘", 51878),
        ("贝方雷", 20021),
        ("陈萧", 74769),
        ("钱安魏·孟熊毛", 29747),
        ("秦齐", 15642),
        ("伏任", 16861),
        ("顾秦", 49859),
        ("唐施柳费费·滕魏", 69248),
        ("昌马", 53820),
        ("费于", 13306),
        ("昌常", 17143),
        ("宋苗吕", 14884),
        ("鲍祁黄", 36801),
        ("陶邬韦", 6541),
        ("郑麻庞", 83939),
        ("费常", 91811),
        ("鲍方阮时戴·戚", 37116),
        ("卜齐邹·屈", 37941),
        ("成苗", 28154),
        ("章葛陶戴·贾任", 39589),
        ("闵花喻·章苏", 49646),
        ("和邹·舒狄邵", 34812),
        ("华祝周华·和花殷", 69626),
        ("姜张茅顾·吕", 56762),
        ("水平·康", 49244),
        ("金邹酆", 15920),
        ("贾贺时", 67557),
        ("平李舒", 46273),
        ("冯席", 98580),
    ];
    let adlar = veri.iter().map(|(ad, _)| *ad).collect::<Vec<_>>();
    let dilimler = veri
        .into_iter()
        .map(|(ad, değer)| VeriÖğesi::adlı(ad, değer))
        .collect::<Vec<_>>();

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık(
            Başlık::yeni()
                .metin("同名数量统计")
                .alt_metin("纯属虚构")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(
            Gösterge::yeni()
                .kaydırılabilir(true)
                .yön(Yön::Dikey)
                .sağ(10)
                .üst(20)
                .iç_boşluk(15.0)
                .veri(adlar),
        )
        .seri(
            PastaSerisi::yeni()
                .ad("姓名")
                .yarıçap("55%")
                .merkez("40%", "50%")
                .veri(dilimler),
        )
}

fn pie_custom() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .arkaplan("#2c343c")
        .başlık(
            Başlık::yeni()
                .metin("Customized Pie")
                .sol("center")
                .üst(20)
                .iç_boşluk(15.0)
                .yazı(YazıStili::yeni().renk("#ccc")),
        )
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .görsel_eşleme(
            GörselEşleme::yeni()
                .göster(false)
                .en_az(80.0)
                .en_çok(600.0)
                .renk_açıklığı(0.0, 1.0),
        )
        .seri(
            PastaSerisi::yeni()
                .ad("Access From")
                .yarıçap("55%")
                .merkez("50%", "50%")
                .gül_türü(GülTürü::Yarıçap)
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .konum(EtiketKonumu::Dış)
                        .yazı(YazıStili::yeni().renk("rgba(255, 255, 255, 0.3)")),
                )
                .etiket_çizgisi(
                    EtiketÇizgisi::yeni()
                        .uzunluk1(10.0)
                        .uzunluk2(20.0)
                        .yumuşaklık(0.2)
                        .stil(
                            ÇizgiStili::yeni()
                                .kalınlık(1.0)
                                .renk("rgba(255, 255, 255, 0.3)"),
                        ),
                )
                .öğe_stili(
                    ÖğeStili::yeni()
                        .renk("#c23531")
                        .gölge_bulanıklığı(200.0)
                        .gölge_rengi("rgba(0, 0, 0, 0.5)"),
                )
                .veri([
                    VeriÖğesi::adlı("Video Ads", 235),
                    VeriÖğesi::adlı("Union Ads", 274),
                    VeriÖğesi::adlı("Email", 310),
                    VeriÖğesi::adlı("Direct", 335),
                    VeriÖğesi::adlı("Search Engine", 400),
                ]),
        )
}

fn resmi_pasta_desenini_oku(değişken: &str) -> Result<GörüntüDeseni, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/examples/ts/pie-pattern.ts");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    let işaret = format!("const {değişken} =");
    let (_, değişken_sonrası) = kaynak
        .split_once(&işaret)
        .ok_or_else(|| format!("{} içinde {değişken} bulunamadı", dosya.display()))?;
    let (_, veri_url_sonrası) = değişken_sonrası
        .split_once("data:image/")
        .ok_or_else(|| format!("{değişken} bir görüntü veri URL'si değil"))?;
    let (mime_ve_kod, _) = veri_url_sonrası
        .split_once("';")
        .ok_or_else(|| format!("{değişken} veri URL'sinin sonu bulunamadı"))?;
    let (_, kod) = mime_ve_kod
        .split_once(";base64,")
        .ok_or_else(|| format!("{değişken} base64 kodlu değil"))?;
    let sıkıştırılmış = base64::engine::general_purpose::STANDARD
        .decode(kod.as_bytes())
        .map_err(|hata| format!("{değişken} base64 çözülemedi: {hata}"))?;
    let rgba = image::load_from_memory(&sıkıştırılmış)
        .map_err(|hata| format!("{değişken} görüntüsü çözülemedi: {hata}"))?
        .to_rgba8();
    let (genişlik, yükseklik) = (rgba.width(), rgba.height());
    GörüntüDeseni::rgba(genişlik, yükseklik, rgba.into_raw(), DesenTekrarı::Tekrar)
        .ok_or_else(|| format!("{değişken} RGBA boyutları geçersiz"))
}

fn pie_pattern() -> Result<GrafikSeçenekleri, String> {
    let pasta_deseni = resmi_pasta_desenini_oku("piePatternSrc")?;
    let arkaplan_deseni = resmi_pasta_desenini_oku("bgPatternSrc")?;
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .arkaplan(Dolgu::Desen(arkaplan_deseni))
        .başlık(
            Başlık::yeni()
                .metin("饼图纹理")
                .iç_boşluk(15.0)
                .yazı(YazıStili::yeni().renk("#235894")),
        )
        .ipucu(İpucu::yeni())
        .seri(
            PastaSerisi::yeni()
                .ad("pie")
                .seçili_uzaklığı(30.0)
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .konum(EtiketKonumu::Dış)
                        .yazı(YazıStili::yeni().boyut(18.0).renk("#235894")),
                )
                .etiket_çizgisi(
                    EtiketÇizgisi::yeni().stil(ÇizgiStili::yeni().kalınlık(1.0).renk("#235894")),
                )
                .öğe_stili(
                    ÖğeStili::yeni()
                        .renk(Dolgu::Desen(pasta_deseni))
                        .opaklık(0.7)
                        .kenarlık_kalınlığı(3.0)
                        .kenarlık_rengi("#235894"),
                )
                .veri([
                    VeriÖğesi::adlı("Search Engine", 1048),
                    VeriÖğesi::adlı("Direct", 735),
                    VeriÖğesi::adlı("Email", 580),
                    VeriÖğesi::adlı("Union Ads", 484),
                    VeriÖğesi::adlı("Video Ads", 300),
                ]),
        ))
}

fn pie_label_line_adjust() -> GrafikSeçenekleri {
    let veriler = [
        vec![
            VeriÖğesi::adlı("圣彼得堡来客", 5.6),
            VeriÖğesi::adlı("陀思妥耶夫斯基全集", 1.0),
            VeriÖğesi::adlı("史记精注全译（全6册）", 0.8),
            VeriÖğesi::adlı("加德纳艺术通史", 0.5),
            VeriÖğesi::adlı("表象与本质", 0.5),
            VeriÖğesi::adlı("其它", 3.8),
        ],
        vec![
            VeriÖğesi::adlı("银河帝国5：迈向基地", 3.8),
            VeriÖğesi::adlı("俞军产品方法论", 2.3),
            VeriÖğesi::adlı("艺术的逃难", 2.2),
            VeriÖğesi::adlı("第一次世界大战回忆录（全五卷）", 1.3),
            VeriÖğesi::adlı("Scrum 精髓", 1.2),
            VeriÖğesi::adlı("其它", 5.7),
        ],
        vec![
            VeriÖğesi::adlı("克莱因壶", 3.5),
            VeriÖğesi::adlı("投资最重要的事", 2.8),
            VeriÖğesi::adlı("简读中国史", 1.7),
            VeriÖğesi::adlı("你当像鸟飞往你的山", 1.4),
            VeriÖğesi::adlı("表象与本质", 0.5),
            VeriÖğesi::adlı("其它", 3.8),
        ],
    ];
    let mut seçenekler = GrafikSeçenekleri::yeni().animasyon(false).başlık(
        Başlık::yeni()
            .metin("阅读书籍分布")
            .sol("center")
            .iç_boşluk(15.0)
            .yazı(YazıStili::yeni().renk("#999").boyut(14.0).kalın(false)),
    );
    for (sıra, veri) in veriler.into_iter().enumerate() {
        seçenekler = seçenekler.seri(
            PastaSerisi::yeni()
                .halka(20, 60)
                .sol(150)
                .genişlik(400)
                .üst(Uzunluk::Yüzde(sıra as f32 * 33.3))
                .yükseklik("33.33%")
                .öğe_stili(
                    ÖğeStili::yeni()
                        .kenarlık_rengi("#fff")
                        .kenarlık_kalınlığı(1.0),
                )
                .etiket(
                    Etiket::yeni()
                        .göster(true)
                        .konum(EtiketKonumu::Dış)
                        .dış_hiza(DışEtiketHizası::Kenar)
                        .biçimleyici("{name|{b}}\n{time|{c} 小时}")
                        .en_küçük_boşluk(5.0)
                        .kenar_uzaklığı(10)
                        .yazı(YazıStili::yeni().satır_yüksekliği(15.0))
                        .zengin_stil("time", YazıStili::yeni().boyut(10.0).renk("#999")),
                )
                .etiket_çizgisi(
                    EtiketÇizgisi::yeni()
                        .uzunluk1(15.0)
                        .uzunluk2(0.0)
                        .en_büyük_yüzey_açısı(80.0),
                )
                .etiket_yerleşimi(|parametreler| {
                    let mut sonuç = EtiketYerleşimSonucu::default();
                    if let Some(mut noktalar) = parametreler.etiket_çizgisi_noktaları {
                        noktalar[2].0 = if parametreler.etiket_kutusu.x < 350.0 {
                            parametreler.etiket_kutusu.x
                        } else {
                            parametreler.etiket_kutusu.sağ()
                        };
                        sonuç.etiket_çizgisi_noktaları = Some(noktalar);
                    }
                    sonuç
                })
                .veri(veri),
        );
    }
    seçenekler
}

fn pie_pad_angle() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst("5%").iç_boşluk(15.0))
        .seri(
            PastaSerisi::yeni()
                .ad("Access From")
                .halka("40%", "70%")
                .dolgu_açısı(5.0)
                .etiket_çakışmasını_önle(false)
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı(10.0))
                .etiket(Etiket::yeni().göster(false).konum(EtiketKonumu::Merkez))
                .etiket_çizgisi(EtiketÇizgisi::yeni().göster(false))
                .veri([
                    VeriÖğesi::adlı("Search Engine", 1048),
                    VeriÖğesi::adlı("Direct", 735),
                    VeriÖğesi::adlı("Email", 580),
                    VeriÖğesi::adlı("Union Ads", 484),
                    VeriÖğesi::adlı("Video Ads", 300),
                ]),
        )
}

fn pie_half_donut() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst("5%").iç_boşluk(15.0))
        .seri(
            PastaSerisi::yeni()
                .ad("Access From")
                .halka("40%", "70%")
                .merkez("50%", "70%")
                .başlangıç_açısı(180.0)
                .bitiş_açısı(360.0)
                .veri([
                    VeriÖğesi::adlı("Search Engine", 1048),
                    VeriÖğesi::adlı("Direct", 735),
                    VeriÖğesi::adlı("Email", 580),
                    VeriÖğesi::adlı("Union Ads", 484),
                    VeriÖğesi::adlı("Video Ads", 300),
                ]),
        )
}

fn pie_border_radius() -> GrafikSeçenekleri {
    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe))
        .gösterge(Gösterge::yeni().üst("5%").iç_boşluk(15.0))
        .seri(
            PastaSerisi::yeni()
                .ad("Access From")
                .halka("40%", "70%")
                .etiket_çakışmasını_önle(false)
                .öğe_stili(
                    ÖğeStili::yeni()
                        .kenarlık_yarıçapı(10.0)
                        .kenarlık_rengi(Renk::BEYAZ)
                        .kenarlık_kalınlığı(2.0),
                )
                .etiket(Etiket::yeni().göster(false).konum(EtiketKonumu::Merkez))
                .etiket_çizgisi(EtiketÇizgisi::yeni().göster(false))
                .veri([
                    VeriÖğesi::adlı("Search Engine", 1048),
                    VeriÖğesi::adlı("Direct", 735),
                    VeriÖğesi::adlı("Email", 580),
                    VeriÖğesi::adlı("Union Ads", 484),
                    VeriÖğesi::adlı("Video Ads", 300),
                ]),
        )
}

fn pie_align_to() -> GrafikSeçenekleri {
    let veri = [
        VeriÖğesi::adlı("Apples", 70),
        VeriÖğesi::adlı("Strawberries", 68),
        VeriÖğesi::adlı("Bananas", 48),
        VeriÖğesi::adlı("Oranges", 40),
        VeriÖğesi::adlı("Pears", 32),
        VeriÖğesi::adlı("Pineapples", 27),
        VeriÖğesi::adlı("Grapes", 18),
    ];
    let başlık = |metin: &str, sol: &str| {
        Başlık::yeni()
            .alt_metin(metin)
            .sol(sol)
            .üst("75%")
            .metin_hizası(BaşlıkMetinHizası::Orta)
            .iç_boşluk(15.0)
    };
    let seri = |sol: Uzunluk, sağ: Uzunluk, dış_hiza, kenar_boşluğu| {
        PastaSerisi::yeni()
            .yarıçap("25%")
            .merkez("50%", "50%")
            .veri(veri.clone())
            .etiket(
                Etiket::yeni()
                    .göster(true)
                    .konum(EtiketKonumu::Dış)
                    .dış_hiza(dış_hiza)
                    .taşma_payını(5.0)
                    .kenar_boşluğu(kenar_boşluğu),
            )
            .görünüm_kutusu(sol, sağ, 0, 0)
    };

    GrafikSeçenekleri::yeni()
        .animasyon(false)
        .başlık_ekle(
            Başlık::yeni()
                .metin("Pie label alignTo")
                .sol("center")
                .iç_boşluk(15.0),
        )
        .başlık_ekle(başlık("alignTo: \"none\" (default)", "16.67%"))
        .başlık_ekle(başlık("alignTo: \"labelLine\"", "50%"))
        .başlık_ekle(başlık("alignTo: \"edge\"", "83.33%"))
        .seri(seri(
            Uzunluk::from(0),
            Uzunluk::from("66.6667%"),
            DışEtiketHizası::Yok,
            0.0,
        ))
        .seri(seri(
            Uzunluk::from("33.3333%"),
            Uzunluk::from("33.3333%"),
            DışEtiketHizası::EtiketÇizgisi,
            0.0,
        ))
        .seri(seri(
            Uzunluk::from("66.6667%"),
            Uzunluk::from(0),
            DışEtiketHizası::Kenar,
            20.0,
        ))
}

fn scatter_effect() -> Result<GrafikSeçenekleri, String> {
    let normal: Vec<[f64; 2]> = serde_json::from_str(include_str!(
        "../testler/gorsel/veri/scatter-effect-normal.json"
    ))
    .map_err(|hata| format!("scatter-effect resmi verisi okunamadı: {hata}"))?;
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .seri(
            SaçılımSerisi::yeni()
                .efektli(true)
                .sembol_boyutu(20.0)
                .veri([[172.7, 105.2], [153.4, 42.0]]),
        )
        .seri(SaçılımSerisi::yeni().veri(normal)))
}

#[derive(Deserialize)]
struct ObamaBütçesi {
    names: Vec<String>,
    #[serde(rename = "budget2011List")]
    bütçe_2011: Vec<Option<f64>>,
    #[serde(rename = "budget2012List")]
    bütçe_2012: Vec<Option<f64>>,
}

fn obama_bütçesini_oku() -> Result<ObamaBütçesi, String> {
    let dosya = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../echarts-examples/public/data/asset/data/obama_budget_proposal_2012.list.json");
    let kaynak = std::fs::read_to_string(&dosya)
        .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
    serde_json::from_str(&kaynak)
        .map_err(|hata| format!("{} ayrıştırılamadı: {hata}", dosya.display()))
}

fn mix_zoom_on_value(son: bool) -> Result<GrafikSeçenekleri, String> {
    let ObamaBütçesi {
        names,
        bütçe_2011,
        bütçe_2012,
    } = obama_bütçesini_oku()?;
    let (başlangıç, bitiş) = if son { (70.0, 100.0) } else { (94.0, 100.0) };
    let mut gösterge =
        Gösterge::yeni()
            .iç_boşluk(15.0)
            .veri(["Growth", "Budget 2011", "Budget 2012"]);
    gösterge.öğe_boşluğu = 5.0;
    Ok(GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
        .gösterge(gösterge)
        .ızgara(
            Izgara::yeni()
                .üst("12%")
                .sol("1%")
                .sağ("10%")
                .etiketi_kapsa(true),
        )
        .araç_kutusu(
            AraçKutusu::yeni()
                .veri_görünümü(true)
                .sihirli_tür(true, true)
                .geri_yükle(true)
                .png_kaydet(true),
        )
        .x_ekseni(Eksen::kategori().veri(names))
        .y_ekseni(
            Eksen::değer()
                .ad("Budget (million USD)")
                .etiket_biçimleyici(Biçimleyici::İşlev(Arc::new(|değer, _| {
                    cizelge::yardimci::bicim::binlik_ayır(değer / 1000.0)
                }))),
        )
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().aralık(başlangıç, bitiş))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(başlangıç, bitiş))
        .veri_yakınlaştırma(
            VeriYakınlaştırma::sürgü()
                // Kimlik belirtilmeyen ECharts `dataZoom` eylemi bağlı tüm
                // dataZoom bileşenlerini günceller; dikey sürgü de buna dâhil.
                .aralık(if son { 70.0 } else { 0.0 }, 100.0)
                .y_eksen_sırası(0)
                .sol("93%")
                .genişlik(30)
                .yükseklik("80%")
                .veri_gölgesi(false),
        )
        .seri(SütunSerisi::yeni().ad("Budget 2011").veri(bütçe_2011))
        .seri(SütunSerisi::yeni().ad("Budget 2012").veri(bütçe_2012)))
}

fn seçenekler(id: &str, durum: &str) -> Result<GrafikSeçenekleri, String> {
    match id {
        "line-simple" => Ok(line_simple()),
        "line-markline" => Ok(line_markline()),
        "line-marker" => Ok(line_marker()),
        "bar-simple" => Ok(bar_simple()),
        "bar1" => Ok(bar1()),
        "mix-line-bar" => Ok(mix_line_bar()),
        "multiple-x-axis" => Ok(multiple_x_axis()),
        "multiple-y-axis" => Ok(multiple_y_axis()),
        "line-smooth" => Ok(line_smooth()),
        "area-basic" => Ok(area_basic()),
        "area-simple" => Ok(area_simple()),
        "area-time-axis" => Ok(area_time_axis()),
        "area-rainfall" => area_rainfall(),
        "dynamic-data2" => dynamic_data2(durum),
        "dynamic-data" => dynamic_data(durum),
        "line-stack" => Ok(line_stack()),
        "line-style" => Ok(line_style()),
        "line-step" => Ok(line_step()),
        "line-in-cartesian-coordinate-system" => Ok(line_in_cartesian_coordinate_system()),
        "line-y-category" => Ok(line_y_category()),
        "line-log" => Ok(line_log()),
        "line-polar" => Ok(line_polar()),
        "line-polar2" => Ok(line_polar2()),
        "line-function" => Ok(line_function()),
        "bump-chart" => Ok(bump_chart()),
        "line-sections" => Ok(line_sections()),
        "area-pieces" => Ok(area_pieces()),
        "line-gradient" => Ok(line_gradient()),
        "line-aqi" => line_aqi(),
        "confidence-band" => confidence_band(),
        "line-race" => line_race(),
        "grid-multiple" => grid_multiple(),
        "intraday-breaks-1" => Ok(intraday_breaks_1()),
        "intraday-breaks-2" => Ok(intraday_breaks_2()),
        "area-stack" => Ok(area_stack()),
        "area-stack-gradient" => Ok(area_stack_gradient()),
        "bar-background" => Ok(bar_background()),
        "bar-tick-align" => Ok(bar_tick_align()),
        "bar-data-color" => Ok(bar_data_color()),
        "bar-stack-borderRadius" => Ok(bar_stack_border_radius()),
        "bar-y-category" => Ok(bar_y_category()),
        "bar-y-category-stack" => Ok(bar_y_category_stack()),
        "bar-negative2" => Ok(bar_negative2()),
        "bar-negative" => Ok(bar_negative()),
        "bar-stack" => Ok(bar_stack()),
        "bar-waterfall" => Ok(bar_waterfall()),
        "bar-waterfall2" => Ok(bar_waterfall2()),
        "bar-stack-normalization" => Ok(bar_stack_normalization()),
        "bar-label-rotation" => Ok(bar_label_rotation()),
        "data-transform-sort-bar" => data_transform_sort_bar(),
        "dataset-simple0" => Ok(dataset_simple0()),
        "dataset-simple1" => dataset_simple1(),
        "dataset-series-layout-by" => Ok(dataset_series_layout_by()),
        "dataset-encode0" => Ok(dataset_encode0()),
        "dataset-default" => Ok(dataset_default()),
        "data-transform-multiple-pie" => Ok(data_transform_multiple_pie()),
        "dataset-link" => Ok(dataset_link(if durum == "son" { "2014" } else { "2012" })),
        "data-transform-filter" => data_transform_filter(),
        "dataset-encode1" => dataset_encode1(),
        "data-transform-aggregate" => data_transform_aggregate(),
        "boxplot-multi" => boxplot_multi(),
        "boxplot-light-velocity" => boxplot_light_velocity(false),
        "boxplot-light-velocity2" => boxplot_light_velocity(true),
        "scatter-simple" => Ok(scatter_simple()),
        "scatter-anscombe-quartet" => scatter_anscombe_quartet(),
        "scatter-jitter" => Ok(scatter_jitter()),
        "doc-example/scatter-jitter-avoidOverlap" => Ok(scatter_jitter_avoid_overlap()),
        "scatter-punchCard" => scatter_punch_card(),
        "bubble-gradient" => bubble_gradient(),
        "scatter-label-align-top" => scatter_label_align_top(),
        "scatter-label-align-right" => scatter_label_align_right(),
        "scatter-aqi-color" => scatter_aqi_color(),
        "scatter-weight" => scatter_weight(),
        "candlestick-simple" => Ok(candlestick_simple()),
        "heatmap-cartesian" => Ok(heatmap_cartesian(durum == "aralık")),
        "heatmap-large" => Ok(heatmap_large()),
        "heatmap-large-piecewise" => Ok(heatmap_large_piecewise(durum == "parça")),
        "calendar-heatmap" => Ok(calendar_heatmap()),
        "calendar-simple" => Ok(calendar_simple()),
        "calendar-vertical" => Ok(calendar_vertical()),
        "calendar-horizontal" => Ok(calendar_horizontal()),
        "calendar-effectscatter" => Ok(calendar_effectscatter()),
        "calendar-graph" => Ok(calendar_graph()),
        "calendar-lunar" => Ok(calendar_lunar()),
        "calendar-pie" => Ok(calendar_pie()),
        "custom-calendar-icon" => custom_calendar_icon(),
        "calendar-charts" => Ok(calendar_charts()),
        "pie-nest" => Ok(pie_nest()),
        "pie-rich-text" => pie_rich_text(),
        "pie-simple" => Ok(pie_simple()),
        "pie-doughnut" => Ok(pie_doughnut()),
        "pie-roseType-simple" => Ok(pie_rose_type_simple()),
        "pie-roseType" => Ok(pie_rose_type()),
        "pie-legend" => Ok(pie_legend()),
        "pie-custom" => Ok(pie_custom()),
        "pie-pattern" => pie_pattern(),
        "pie-labelLine-adjust" => Ok(pie_label_line_adjust()),
        "pie-padAngle" => Ok(pie_pad_angle()),
        "pie-half-donut" => Ok(pie_half_donut()),
        "pie-borderRadius" => Ok(pie_border_radius()),
        "pie-alignTo" => Ok(pie_align_to()),
        "scatter-effect" => scatter_effect(),
        "mix-zoom-on-value" => mix_zoom_on_value(durum == "son"),
        _ => Err(format!("fixture uygulanmadı: {id}")),
    }
}

fn çalıştır() -> Result<(), String> {
    let girdi = argümanları_oku()?;
    let seçenekler = seçenekler(&girdi.id, &girdi.durum)?;
    let kanıt_faresi = if girdi.id == "dataset-link" && girdi.durum == "son" {
        Some((323.75, 400.0))
    } else if girdi.id == "dynamic-data2" && girdi.durum == "ipucu" {
        Some((472.87, 250.0))
    } else if girdi.id == "dynamic-data" && girdi.durum == "ipucu" {
        Some((446.25, 250.0))
    } else {
        None
    };
    let kanıt_ipucu_öğesi =
        (girdi.id == "heatmap-cartesian" && girdi.durum == "ipucu").then_some((0, 85));
    if std::env::var_os("UYUM_DEBUG_LAYOUT").is_some() {
        // Referans üreticisinin aynı adlı tanı kipiyle birlikte kullanılır;
        // kayıt yüzeyi gerçek boyama hattındaki kesin geometriyi verir.
        let (çözülmüş, hatalar) = seçenekler.veri_kümesini_uygula();
        eprintln!("dataset tanıları: {hatalar:?}");
        for (sıra, seri) in çözülmüş.seriler.iter().enumerate() {
            match seri {
                Seri::Saçılım(saçılım) => {
                    eprintln!(
                        "scatter[{sıra}] eşleme={:?} ilk={:?}",
                        saçılım.eşleme,
                        saçılım.veri.first()
                    );
                }
                Seri::Çizgi(çizgi) => {
                    let örnekler = [0, 1_999, 2_000, 3_999, 4_000]
                        .into_iter()
                        .filter_map(|veri_sırası| {
                            çizgi
                                .veri
                                .get(veri_sırası)
                                .map(|öğe| (veri_sırası, öğe.değer.x(), öğe.değer.sayı()))
                        })
                        .collect::<Vec<_>>();
                    eprintln!("çizgi[{sıra}] örnekleri={örnekler:?}");
                }
                _ => {}
            }
        }
        let mut kayıt = KayıtYüzeyi::yeni(girdi.genişlik, girdi.yükseklik);
        let _ = grafiği_boya(
            &mut kayıt,
            &seçenekler,
            &BoyamaGirdisi {
                fare: kanıt_faresi,
                ipucu_öğesi: kanıt_ipucu_öğesi,
                ..BoyamaGirdisi::default()
            },
        );
        eprintln!("{}", kayıt.döküm());
    }
    // Kanıt aracı, örnek metadata'sındaki 4:3 viewport'u iki renderer'a da
    // geçirip ham kareyi aynı `sharp.resize(600, 450)` adımıyla küçültür.
    let mut yüzey = PikselYüzeyi::yeni(girdi.genişlik, girdi.yükseklik, 1.0)
        .map_err(|hata| hata.to_string())?;
    if std::env::var_os("UYUM_DEBUG_LAYOUT").is_some() {
        eprintln!(
            "piksel yazı ölçüleri: 10,000={:?} 10 km={:?} -80 °C={:?} Australia={:?} Life Expectancy={:?} legend={:?}",
            yüzey.yazı_ölç("10,000", 12.0),
            yüzey.yazı_ölç("10 km", 12.0),
            yüzey.yazı_ölç("-80 °C", 12.0),
            yüzey.yazı_ölç("Australia", 12.0),
            yüzey.yazı_ölç("Life Expectancy", 12.0),
            [
                "Email",
                "Union Ads",
                "Video Ads",
                "Direct",
                "Search Engine",
                "Ads",
                "Union",
                "Video",
                "Search",
                "Engine",
                "2000-06-05",
                "middle",
                "insideStart",
                "insideStartTop",
                "insideMiddle / middle",
            ]
            .map(|metin| (metin, yüzey.yazı_ölç(metin, 12.0).0))
        );
    }
    let boyama = BoyamaGirdisi {
        // Referans ön işlemcisi seri giriş animasyonunu kapatır; `kare`
        // yalnız sürekli efekt saatini ilerletir.
        ilerleme: 1.0,
        zaman_sn: girdi.kare * 2.0,
        fare: kanıt_faresi,
        ipucu_öğesi: kanıt_ipucu_öğesi,
        ..BoyamaGirdisi::default()
    };
    let boyama_çıktısı = grafiği_boya(&mut yüzey, &seçenekler, &boyama);
    if std::env::var_os("UYUM_DEBUG_LAYOUT").is_some() {
        eprintln!(
            "piksel gösterge kutuları={:?}",
            boyama_çıktısı.gösterge_kutuları
        );
        for seri_sırası in 0..seçenekler.seriler.len() {
            let noktalar: Vec<_> = boyama_çıktısı
                .isabetler
                .iter()
                .filter(|isabet| isabet.seri_sırası == seri_sırası)
                .filter_map(|isabet| match isabet.geometri {
                    cizelge::cizim::İsabetGeometrisi::Daire { merkez, .. } => Some(merkez),
                    _ => None,
                })
                .take(5)
                .collect();
            if !noktalar.is_empty() {
                eprintln!("piksel scatter[{seri_sırası}]={noktalar:?}");
            }
        }
    }
    let png = yüzey.png_kodla().map_err(|hata| hata.to_string())?;
    if let Some(üst) = girdi.çıktı.parent() {
        std::fs::create_dir_all(üst).map_err(|hata| format!("çıktı dizini: {hata}"))?;
    }
    std::fs::write(&girdi.çıktı, png).map_err(|hata| format!("PNG yazılamadı: {hata}"))
}

fn main() {
    if let Err(hata) = çalıştır() {
        eprintln!("Uyum fixture hatası: {hata}");
        std::process::exit(1);
    }
}
