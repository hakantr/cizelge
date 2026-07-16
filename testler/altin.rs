#![allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Altın (golden) görsel regresyon testleri.
//!
//! Grafikler [`KayıtYüzeyi`] üzerine boyanır; üretilen komut dökümü
//! `testler/altin/*.txt` dosyalarındaki beklenen çıktıyla karşılaştırılır.
//! Altınları yeniden üretmek için:
//!
//! ```bash
//! ALTIN_GUNCELLE=1 cargo test --test altin
//! ```

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use cizelge::hazir::*;

fn altın_karşılaştır(ad: &str, içerik: &str) {
    let yol = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testler/altin")
        .join(format!("{ad}.txt"));
    if std::env::var("ALTIN_GUNCELLE").is_ok() {
        fs::create_dir_all(yol.parent().unwrap()).unwrap();
        fs::write(&yol, içerik).unwrap();
        return;
    }
    let beklenen = fs::read_to_string(&yol).unwrap_or_else(|_| {
        panic!("altın dosyası yok: {} — ALTIN_GUNCELLE=1 ile üretin", yol.display())
    });
    if beklenen != içerik {
        // Farkı okunur biçimde göster.
        for (satır_no, (b, ü)) in beklenen.lines().zip(içerik.lines()).enumerate() {
            if b != ü {
                panic!(
                    "altın uyuşmazlığı ({ad}), satır {}:\n  beklenen: {b}\n  üretilen: {ü}",
                    satır_no + 1
                );
            }
        }
        panic!(
            "altın uyuşmazlığı ({ad}): satır sayısı farklı (beklenen {}, üretilen {})",
            beklenen.lines().count(),
            içerik.lines().count()
        );
    }
}

fn boya_ve_dök(seçenekler: GrafikSeçenekleri) -> String {
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    // Animasyonsuz, faresiz, tüm seriler açık.
    grafiği_boya(&mut yüzey, &seçenekler, 1.0, 0.0, None, &HashSet::new());
    yüzey.döküm()
}

#[test]
fn cizgi_serisi() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Çizgi"))
        .gösterge(Gösterge::yeni().üst(28.0))
        .x_ekseni(Eksen::kategori().veri(["A", "B", "C", "D"]).kenar_boşluğu(false))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Birinci")
                .veri([10.0, 40.0, 30.0, 60.0])
                .yumuşat(true)
                .alan_stili(AlanStili::yeni()),
        )
        .seri(ÇizgiSerisi::yeni().ad("İkinci").veri([
            VeriÖğesi::from(5.0),
            VeriÖğesi::from(15.0),
            VeriÖğesi::from(Some(25.0)),
            VeriÖğesi::from(None::<f64>),
        ]));
    altın_karşılaştır("cizgi_serisi", &boya_ve_dök(seçenekler));
}

#[test]
fn sutun_yigin() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["Ç1", "Ç2", "Ç3"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(SütunSerisi::yeni().ad("A").yığın("t").veri([10.0, 20.0, 30.0]))
        .seri(SütunSerisi::yeni().ad("B").yığın("t").veri([5.0, 10.0, -15.0]))
        .seri(
            SütunSerisi::yeni()
                .ad("C")
                .veri([12.0, 8.0, 22.0])
                .öğe_stili(ÖğeStili::yeni().kenarlık_yarıçapı([3.0, 3.0, 0.0, 0.0])),
        );
    altın_karşılaştır("sutun_yigin", &boya_ve_dök(seçenekler));
}

#[test]
fn pasta_halka() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .seri(
            PastaSerisi::yeni()
                .ad("Pay")
                .halka("40%", "70%")
                .veri([("Bir", 60.0), ("İki", 30.0), ("Üç", 10.0)]),
        );
    altın_karşılaştır("pasta_halka", &boya_ve_dök(seçenekler));
}

#[test]
fn sacilim_degerli() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .animasyon(false)
        .seri(
            SaçılımSerisi::yeni()
                .ad("Noktalar")
                .sembol_boyutu(12.0)
                .veri([[1.0, 2.0], [3.0, 5.0], [7.0, 4.0]]),
        );
    altın_karşılaştır("sacilim_degerli", &boya_ve_dök(seçenekler));
}

#[test]
fn gradyan_ve_log() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["a", "b", "c", "d", "e"]))
        .y_ekseni(Eksen::log())
        .animasyon(false)
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Üstel")
                .veri([1.0, 10.0, 100.0, 40.0, 1000.0])
                .alan_stili(AlanStili::yeni().renk(Dolgu::doğrusal(
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    vec![
                        RenkDurağı::yeni(0.0, Renk::onaltılık(0x5070dd)),
                        RenkDurağı::yeni(0.5, Renk::onaltılık(0xb6d634)),
                        RenkDurağı::yeni(1.0, Renk::onaltılık(0x5070dd).alfa_ile(0.0)),
                    ],
                ))),
        );
    altın_karşılaştır("gradyan_ve_log", &boya_ve_dök(seçenekler));
}

#[test]
fn ipucu_ve_imlec() {
    // Fare ızgara içinde: eksen imleci + ipucu penceresi de kayda girer.
    let seçenekler = GrafikSeçenekleri::yeni()
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen).imleç(İmleçTürü::Gölge))
        .x_ekseni(Eksen::kategori().veri(["A", "B", "C"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(SütunSerisi::yeni().ad("S").veri([3.0, 7.0, 5.0]));
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    grafiği_boya(
        &mut yüzey,
        &seçenekler,
        1.0,
        0.0,
        Some((400.0, 300.0)),
        &HashSet::new(),
    );
    altın_karşılaştır("ipucu_ve_imlec", &yüzey.döküm());
}

#[test]
fn imleyiciler() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["A", "B", "C", "D"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(
            SütunSerisi::yeni()
                .ad("Satış")
                .veri([12.0, 30.0, 18.0, 24.0])
                .im_çizgisi(İmÇizgisi::yeni().yatay(İmDeğeri::Ortalama))
                .im_noktası(İmNoktası::yeni().en_büyük().en_küçük())
                .im_alanı(İmAlanı::yeni().x_aralığı("Kampanya", 1.0, 2.0)),
        );
    altın_karşılaştır("imleyiciler", &boya_ve_dök(seçenekler));
}

#[test]
fn ara_çentikler_ve_bölme_alanı() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(
            Eksen::kategori()
                .veri(["A", "B", "C"])
                .bölme_alanı_göster(true),
        )
        .y_ekseni(
            Eksen::değer()
                .ara_çentik_göster(true)
                .ara_bölme_çizgisi_göster(true),
        )
        .animasyon(false)
        .seri(ÇizgiSerisi::yeni().ad("S").veri([2.0, 9.0, 5.0]));
    altın_karşılaştır("ara_centikler_ve_bolme_alani", &boya_ve_dök(seçenekler));
}

#[test]
fn mum_ve_kutu() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["Pzt", "Sal", "Çar", "Per"]))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .animasyon(false)
        .seri(MumSerisi::yeni().ad("Hisse").veri([
            [20.0, 34.0, 10.0, 38.0],
            [40.0, 35.0, 30.0, 50.0],
            [31.0, 38.0, 33.0, 44.0],
            [38.0, 15.0, 5.0, 42.0],
        ]))
        .seri(KutuSerisi::yeni().ad("Dağılım").veri([
            [8.0, 14.0, 20.0, 27.0, 35.0],
            [10.0, 18.0, 25.0, 32.0, 45.0],
            [12.0, 16.0, 22.0, 28.0, 36.0],
            [7.0, 11.0, 15.0, 21.0, 30.0],
        ]));
    altın_karşılaştır("mum_ve_kutu", &boya_ve_dök(seçenekler));
}

#[test]
fn ısı_haritası() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["00:00", "06:00", "12:00", "18:00"]))
        .y_ekseni(Eksen::kategori().veri(["Pzt", "Sal", "Çar"]))
        .görsel_eşleme(GörselEşleme::yeni())
        .animasyon(false)
        .seri(
            IsıHaritasıSerisi::yeni()
                .ad("Yoğunluk")
                .etiket(Etiket::yeni().göster(true))
                .veri([
                    [0.0, 0.0, 5.0],
                    [1.0, 0.0, 7.0],
                    [2.0, 0.0, 12.0],
                    [3.0, 0.0, 3.0],
                    [0.0, 1.0, 8.0],
                    [1.0, 1.0, 2.0],
                    [2.0, 1.0, 10.0],
                    [3.0, 1.0, 6.0],
                    [0.0, 2.0, 1.0],
                    [1.0, 2.0, 9.0],
                    [2.0, 2.0, 4.0],
                    [3.0, 2.0, 11.0],
                ]),
        );
    altın_karşılaştır("isi_haritasi", &boya_ve_dök(seçenekler));
}

#[test]
fn efektli_saçılım() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .animasyon(false)
        .seri(
            SaçılımSerisi::yeni()
                .ad("Sinyal")
                .sembol_boyutu(14.0)
                .efektli(true)
                .veri([[2.0, 3.0], [5.0, 6.0]]),
        );
    altın_karşılaştır("efektli_sacilim", &boya_ve_dök(seçenekler));
}

#[test]
fn huni() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Dönüşüm"))
        .animasyon(false)
        .seri(HuniSerisi::yeni().ad("Dönüşüm").veri([
            ("Ziyaret", 100.0),
            ("Tıklama", 80.0),
            ("Sepet", 40.0),
            ("Sipariş", 20.0),
        ]));
    altın_karşılaştır("huni", &boya_ve_dök(seçenekler));
}

#[test]
fn gösterge_saati() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .seri(
            GöstergeSaatiSerisi::yeni()
                .ad("Basınç")
                .değer(72.5, "Yük")
                .değer_biçimleyici("{value} %"),
        );
    altın_karşılaştır("gosterge_saati", &boya_ve_dök(seçenekler));
}

#[test]
fn isabet_bölgeleri_üretilir() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["A", "B"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(SütunSerisi::yeni().ad("S").veri([3.0, 7.0]))
        .seri(PastaSerisi::yeni().ad("P").yarıçap("30%").veri([("X", 1.0), ("Y", 2.0)]));
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    let çıktı = grafiği_boya(&mut yüzey, &seçenekler, 1.0, 0.0, None, &HashSet::new());
    // 2 sütun + 2 dilim = 4 tıklanabilir bölge.
    assert_eq!(çıktı.isabetler.len(), 4);
    // Sütun bölgesinin içi gerçekten isabet sayılmalı.
    let ilk = &çıktı.isabetler[0];
    if let cizelge::İsabetGeometrisi::Dikdörtgen(d) = &ilk.geometri {
        assert!(ilk.geometri.içeriyor_mu(d.merkez()));
    } else {
        panic!("ilk bölge sütun dikdörtgeni olmalıydı");
    }
}
