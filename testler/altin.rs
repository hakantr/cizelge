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
    grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());
    yüzey.döküm()
}

fn fareli_girdi(fare: (f32, f32)) -> BoyamaGirdisi {
    BoyamaGirdisi { fare: Some(fare), ..Default::default() }
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
    grafiği_boya(&mut yüzey, &seçenekler, &fareli_girdi((400.0, 300.0)));
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
fn radar() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .gösterge(Gösterge::yeni())
        .radar(RadarKoordinatı::yeni().göstergeler([
            ("Satış", 100.0),
            ("Pazarlama", 100.0),
            ("Geliştirme", 100.0),
            ("Destek", 100.0),
            ("Yönetim", 100.0),
        ]))
        .animasyon(false)
        .seri(RadarSerisi::yeni().ad("Bütçe").alan_stili(AlanStili::yeni().opaklık(0.3)).veri([
            ("Plan", vec![80.0, 60.0, 90.0, 40.0, 70.0]),
            ("Gerçekleşen", vec![70.0, 75.0, 60.0, 55.0, 50.0]),
        ]));
    altın_karşılaştır("radar", &boya_ve_dök(seçenekler));
}

#[test]
fn çapraz_imleç() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe).imleç(İmleçTürü::Çapraz))
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .animasyon(false)
        .seri(SaçılımSerisi::yeni().ad("N").veri([[1.0, 2.0], [4.0, 6.0]]));
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    grafiği_boya(&mut yüzey, &seçenekler, &fareli_girdi((400.0, 300.0)));
    altın_karşılaştır("capraz_imlec", &yüzey.döküm());
}

#[test]
fn resimli_sütun_ve_özel_seri() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["A", "B", "C"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(
            SütunSerisi::yeni()
                .ad("Stok")
                .veri([3.0, 6.0, 4.0])
                .piktogram(Piktogram::default()),
        )
        .seri(
            ÖzelSeri::yeni()
                .ad("Özel")
                .veri([1.0, 2.0])
                .çizim(|yüzey, bağlam| {
                    // Kullanıcı çizimi: ızgara köşesine bir işaret.
                    yüzey.daire(
                        (bağlam.alan.x + 12.0, bağlam.alan.y + 12.0),
                        6.0,
                        Some(&Dolgu::Düz(bağlam.renk)),
                        None,
                    );
                }),
        );
    altın_karşılaştır("resimli_sutun_ve_ozel_seri", &boya_ve_dök(seçenekler));
}

#[test]
fn örnekleme_altını() {
    let veri: Vec<f64> = (0..5000).map(|i| ((i as f64) * 0.05).sin() * 50.0 + 60.0).collect();
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori())
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Sinyal")
                .sembol_göster(false)
                .örnekleme(Örnekleme::Lttb)
                .veri(veri),
        );
    // Altın karşılaştırması yerine boyut sınırı: 5000 nokta ızgara
    // genişliğine (≤ ~800 komut) inmiş olmalı.
    let döküm = boya_ve_dök(seçenekler);
    let satır_sayısı = döküm.lines().count();
    assert!(satır_sayısı < 1000, "örnekleme etkisiz: {satır_sayısı} satır");
}

#[test]
fn çoklu_ızgara_ve_ikincil_eksen() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        // Üst ızgara: çizgi (sol y) + ikincil sağ y ekseninde ikinci çizgi.
        .ızgara_ekle(Izgara::yeni().sol(60.0).sağ(60.0).üst(30.0).alt("55%"))
        // Alt ızgara: sütun.
        .ızgara_ekle(Izgara::yeni().sol(60.0).sağ(60.0).üst("60%").alt(40.0))
        .x_ekseni_ekle(Eksen::kategori().veri(["A", "B", "C", "D"]).ızgara_sırası(0))
        .x_ekseni_ekle(Eksen::kategori().veri(["A", "B", "C", "D"]).ızgara_sırası(1))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(0))
        .y_ekseni_ekle(Eksen::değer().ölçekli(true).ızgara_sırası(0))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(1))
        .seri(ÇizgiSerisi::yeni().ad("Sıcaklık").veri([10.0, 14.0, 12.0, 18.0]))
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Nem")
                .eksenler(0, 1)
                .veri([55.0, 60.0, 52.0, 66.0]),
        )
        .seri(
            SütunSerisi::yeni()
                .ad("Yağış")
                .eksenler(1, 2)
                .veri([4.0, 9.0, 6.0, 2.0]),
        );
    altın_karşılaştır("coklu_izgara", &boya_ve_dök(seçenekler));
}

#[test]
fn yakınlaştırma_penceresi() {
    // Kategorik eksende %25–%75 penceresi + sürgü şeridi.
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["A", "B", "C", "D", "E", "F", "G", "H"]))
        .y_ekseni(Eksen::değer())
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(25.0, 75.0))
        .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().aralık(25.0, 75.0))
        .animasyon(false)
        .seri(SütunSerisi::yeni().ad("S").veri([
            1.0, 3.0, 5.0, 7.0, 9.0, 11.0, 13.0, 15.0,
        ]));
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());
    assert_eq!(çıktı.iç_yakınlaştırmalar.len(), 1);
    assert_eq!(çıktı.sürgüler.len(), 1);
    altın_karşılaştır("yakinlastirma", &yüzey.döküm());
}

#[test]
fn değer_ekseni_penceresi() {
    // Sayısal x ekseninde pencere: kapsam daraltılır, çentikler pencereye
    // göre yeniden hesaplanır.
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::değer().ölçekli(true))
        .y_ekseni(Eksen::değer().ölçekli(true))
        .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(0.0, 50.0))
        .animasyon(false)
        .seri(SaçılımSerisi::yeni().ad("N").veri([
            [0.0, 1.0],
            [10.0, 5.0],
            [20.0, 3.0],
            [40.0, 8.0],
        ]));
    altın_karşılaştır("deger_ekseni_penceresi", &boya_ve_dök(seçenekler));
}

#[test]
fn parçalı_eşleme() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["a", "b", "c"]))
        .y_ekseni(Eksen::kategori().veri(["x", "y"]))
        .görsel_eşleme(GörselEşleme::yeni().parçalar([
            EşlemeParçası::yeni(None, Some(5.0), 0x67e0e3u32).etiket("Düşük"),
            EşlemeParçası::yeni(Some(5.0), Some(10.0), 0x37a2dau32).etiket("Orta"),
            EşlemeParçası::yeni(Some(10.0), None, 0xfd666du32).etiket("Yüksek"),
        ]))
        .animasyon(false)
        .seri(IsıHaritasıSerisi::yeni().ad("V").veri([
            [0.0, 0.0, 2.0],
            [1.0, 0.0, 7.0],
            [2.0, 0.0, 12.0],
            [0.0, 1.0, 4.0],
            [1.0, 1.0, 9.0],
            [2.0, 1.0, 15.0],
        ]));
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());
    assert_eq!(çıktı.eşleme_kutuları.len(), 3);
    altın_karşılaştır("parcali_esleme", &yüzey.döküm());
}

#[test]
fn etkileşim_araçları() {
    // Kaydırmalı gösterge (2. sayfa), araç kutusu ve fırça kaplaması.
    let çok_seri: Vec<Seri> = (0..12)
        .map(|i| {
            Seri::from(
                ÇizgiSerisi::yeni()
                    .ad(format!("Uzun Seri Adı {i}"))
                    .veri([i as f64, (i + 2) as f64]),
            )
        })
        .collect();
    let seçenekler = GrafikSeçenekleri::yeni()
        .gösterge(Gösterge::yeni().kaydırılabilir(true))
        .araç_kutusu(AraçKutusu::yeni())
        .fırça(Fırça::yeni())
        .x_ekseni(Eksen::kategori().veri(["A", "B"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seriler(çok_seri);
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    let girdi = BoyamaGirdisi {
        gösterge_sayfası: 1,
        fırça: Some([200.0, 200.0, 400.0, 380.0]),
        ..Default::default()
    };
    let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &girdi);
    assert_eq!(çıktı.gösterge_okları.len(), 2);
    assert_eq!(çıktı.araç_düğmeleri.len(), 1);
    assert!(çıktı.gösterge_kutuları.len() < 12, "sayfalama uygulanmalıydı");
    altın_karşılaştır("etkilesim_araclari", &yüzey.döküm());
}

#[test]
fn kutupsal_seriler() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .kutupsal(
            KutupsalKoordinat::yeni()
                .açısal_eksen(Eksen::kategori().veri(["K", "KD", "D", "GD", "G", "GB", "B", "KB"])),
        )
        .animasyon(false)
        .seri(
            SütunSerisi::yeni()
                .ad("Rüzgar")
                .kutupsal(true)
                .veri([4.0, 7.0, 3.0, 6.0, 8.0, 5.0, 2.0, 6.5]),
        )
        .seri(
            ÇizgiSerisi::yeni()
                .ad("Ortalama")
                .kutupsal(true)
                .veri([3.0, 5.0, 4.0, 5.5, 6.0, 4.5, 3.0, 5.0]),
        );
    altın_karşılaştır("kutupsal", &boya_ve_dök(seçenekler));
}

#[test]
fn ağaç_haritası() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .başlık(Başlık::yeni().metin("Disk"))
        .animasyon(false)
        .seri(AğaçHaritasıSerisi::yeni().ad("Disk").kökler([
            AğaçDüğümü::dal(
                "Belgeler",
                vec![
                    AğaçDüğümü::yaprak("Raporlar", 32.0),
                    AğaçDüğümü::yaprak("Sunumlar", 18.0),
                ],
            ),
            AğaçDüğümü::dal(
                "Medya",
                vec![
                    AğaçDüğümü::yaprak("Video", 60.0),
                    AğaçDüğümü::yaprak("Müzik", 25.0),
                    AğaçDüğümü::yaprak("Fotoğraf", 15.0),
                ],
            ),
            AğaçDüğümü::yaprak("Sistem", 20.0),
        ]));
    altın_karşılaştır("agac_haritasi", &boya_ve_dök(seçenekler));
}

#[test]
fn güneş_patlaması() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .seri(GüneşPatlamasıSerisi::yeni().ad("Kaynak").kökler([
            AğaçDüğümü::dal(
                "A",
                vec![AğaçDüğümü::yaprak("A1", 4.0), AğaçDüğümü::yaprak("A2", 6.0)],
            ),
            AğaçDüğümü::dal(
                "B",
                vec![
                    AğaçDüğümü::yaprak("B1", 3.0),
                    AğaçDüğümü::dal("B2", vec![AğaçDüğümü::yaprak("B2a", 2.0)]),
                ],
            ),
        ]));
    altın_karşılaştır("gunes_patlamasi", &boya_ve_dök(seçenekler));
}

#[test]
fn svg_dışa_aktarım() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["A", "B", "C"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(
            ÇizgiSerisi::yeni()
                .ad("S")
                .veri([3.0, 7.0, 5.0])
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
    let svg = svg_dışa_aktar(&seçenekler, 800.0, 600.0);
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("<linearGradient"), "çok duraklı gradyan defs'e yazılmalı");
    assert!(svg.contains("<path"));
    assert!(svg.contains("<text"));
    // Altın olarak da sakla (belirlenimci üretim).
    altın_karşılaştır("svg_disa_aktarim", &svg);
}

#[test]
fn ağaç_serisi() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .seri(AğaçSerisi::yeni().ad("Kuruluş").kökler([AğaçDüğümü::dal(
            "Genel Müdür",
            vec![
                AğaçDüğümü::dal(
                    "Mühendislik",
                    vec![
                        AğaçDüğümü::yaprak("Arayüz", 12.0),
                        AğaçDüğümü::yaprak("Altyapı", 9.0),
                    ],
                ),
                AğaçDüğümü::dal(
                    "Satış",
                    vec![AğaçDüğümü::yaprak("Yurt İçi", 7.0)],
                ),
                AğaçDüğümü::yaprak("İK", 4.0),
            ],
        )]));
    altın_karşılaştır("agac_serisi", &boya_ve_dök(seçenekler));
}

#[test]
fn sankey() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .seri(SankeySerisi::yeni().ad("Enerji").bağlar([
            ("Kömür", "Elektrik", 30.0),
            ("Gaz", "Elektrik", 20.0),
            ("Güneş", "Elektrik", 12.0),
            ("Elektrik", "Konut", 35.0),
            ("Elektrik", "Sanayi", 27.0),
            ("Gaz", "Konut", 8.0),
        ]));
    altın_karşılaştır("sankey", &boya_ve_dök(seçenekler));
}

#[test]
fn grafo() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .seri(
            GrafoSerisi::yeni()
                .ad("Ağ")
                .düğümler([
                    GrafoDüğümü::yeni("A", 24.0).kategori(0),
                    GrafoDüğümü::yeni("B", 16.0).kategori(1),
                    GrafoDüğümü::yeni("C", 16.0).kategori(1),
                    GrafoDüğümü::yeni("D", 12.0).kategori(2),
                    GrafoDüğümü::yeni("E", 12.0).kategori(2),
                ])
                .bağlar([("A", "B"), ("A", "C"), ("B", "D"), ("C", "E"), ("D", "E")]),
        );
    altın_karşılaştır("grafo", &boya_ve_dök(seçenekler));
}

#[test]
fn kiriş() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .seri(KirişSerisi::yeni().ad("Göç").bağlar([
            ("Kuzey", "Güney", 12.0),
            ("Güney", "Doğu", 8.0),
            ("Doğu", "Kuzey", 5.0),
            ("Kuzey", "Batı", 6.0),
        ]));
    altın_karşılaştır("kiris", &boya_ve_dök(seçenekler));
}

#[test]
fn paralel_koordinat() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .seri(
            ParalelSerisi::yeni()
                .ad("Ölçümler")
                .boyutlar(["Fiyat", "Ağırlık", "Puan", "Stok"])
                .veri([
                    vec![12.0, 3.4, 8.2, 40.0],
                    vec![9.5, 2.1, 6.4, 65.0],
                    vec![15.2, 4.8, 9.1, 20.0],
                    vec![7.8, 1.9, 5.5, 80.0],
                ]),
        );
    altın_karşılaştır("paralel", &boya_ve_dök(seçenekler));
}

#[test]
fn takvim_ısısı() {
    // 2026'nın ilk 60 günü, belirlenimci değerlerle.
    let gün_ms = 86_400_000.0f64;
    let yıl_başı = 1_767_225_600_000.0f64; // 2026-01-01 UTC
    let veri: Vec<VeriÖğesi> = (0..60)
        .map(|g| VeriÖğesi::from(vec![yıl_başı + g as f64 * gün_ms, ((g * 7) % 13) as f64]))
        .collect();
    let seçenekler = GrafikSeçenekleri::yeni()
        .görsel_eşleme(GörselEşleme::yeni())
        .animasyon(false)
        .seri(TakvimSerisi::yeni(2026).ad("Katkılar").veri(veri));
    altın_karşılaştır("takvim_isisi", &boya_ve_dök(seçenekler));
}

#[test]
fn veri_kümesi_eşleme() {
    // Tek tablo → sütun + pasta, encode ile.
    let küme = VeriKümesi::yeni(["ürün", "satış"]).kayıtlar([
        ("Elma", vec![120.0]),
        ("Armut", vec![80.0]),
        ("Kiraz", vec![160.0]),
    ]);
    let seçenekler = GrafikSeçenekleri::yeni()
        .veri_kümesi(küme)
        .x_ekseni(Eksen::kategori().veri(["Elma", "Armut", "Kiraz"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(SütunSerisi::yeni().ad("Satış").eşle("ürün", "satış"))
        .seri(
            PastaSerisi::yeni()
                .ad("Pay")
                .eşle("ürün", "satış")
                .halka("8%", "16%")
                .merkez("84%", "24%"),
        );
    altın_karşılaştır("veri_kumesi", &boya_ve_dök(seçenekler));
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
    let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());
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

#[test]
fn tema_nehri() {
    let mut veri: Vec<(f64, f64, String)> = Vec::new();
    for (k, katman) in ["Rüzgar", "Güneş", "Hidro"].iter().enumerate() {
        for x in 0..8 {
            let değer = 20.0 + ((x * (k + 3)) % 7) as f64 * 6.0;
            veri.push((x as f64, değer, (*katman).to_string()));
        }
    }
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .seri(TemaNehriSerisi::yeni().ad("Üretim").veri(veri));
    altın_karşılaştır("tema_nehri", &boya_ve_dök(seçenekler));
}

#[test]
fn koyu_tema() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .koyu(true)
        .x_ekseni(Eksen::kategori().veri(["Pzt", "Sal", "Çar"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(ÇizgiSerisi::yeni().ad("Ziyaret").veri([120.0, 200.0, 150.0]));
    altın_karşılaştır("koyu_tema", &boya_ve_dök(seçenekler));
}

#[test]
fn koyu_tema_açıktan_farklı() {
    let kur = |koyu: bool| {
        GrafikSeçenekleri::yeni()
            .koyu(koyu)
            .x_ekseni(Eksen::kategori().veri(["A", "B"]))
            .y_ekseni(Eksen::değer())
            .animasyon(false)
            .seri(SütunSerisi::yeni().ad("S").veri([3.0, 7.0]))
    };
    let açık = boya_ve_dök(kur(false));
    let koyu = boya_ve_dök(kur(true));
    assert_ne!(açık, koyu, "koyu tema çıktısı açık temadan farklı olmalı");
    // Koyu kipin iş parçacığı yerelinde kalıcı iz bırakmadığını da doğrula:
    // aynı açık seçenekler ikinci kez aynı çıktıyı vermeli.
    assert_eq!(açık, boya_ve_dök(kur(false)));
}

#[test]
fn zaman_şeridi() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .x_ekseni(Eksen::kategori().veri(["A", "B"]))
        .y_ekseni(Eksen::değer())
        .animasyon(false)
        .seri(SütunSerisi::yeni().ad("S").veri([3.0, 7.0]));
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    let girdi = BoyamaGirdisi { zaman_şeridi: Some((1, 5, true)), ..Default::default() };
    let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &girdi);
    // Oynat/durdur + 5 kare noktası tıklanabilir olmalı.
    assert_eq!(çıktı.zaman_düğmeleri.len(), 6);
    assert!(matches!(
        çıktı.zaman_düğmeleri.first(),
        Some((_, ZamanŞeridiEylemi::OynatDurdur))
    ));
    altın_karşılaştır("zaman_seridi", &yüzey.döküm());
}

#[test]
fn bağlantılı_imleç() {
    // İki ızgara; imleç üstteyken bağlantı sayesinde ALTTA da çizgi çizilir.
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen).bağlantılı(true))
        .ızgara_ekle(Izgara::yeni().sol(60.0).sağ(60.0).üst(30.0).alt("55%"))
        .ızgara_ekle(Izgara::yeni().sol(60.0).sağ(60.0).üst("60%").alt(40.0))
        .x_ekseni_ekle(Eksen::kategori().veri(["A", "B", "C", "D"]).ızgara_sırası(0))
        .x_ekseni_ekle(Eksen::kategori().veri(["A", "B", "C", "D"]).ızgara_sırası(1))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(0))
        .y_ekseni_ekle(Eksen::değer().ızgara_sırası(1))
        .seri(ÇizgiSerisi::yeni().ad("Üst").veri([10.0, 14.0, 12.0, 18.0]))
        .seri(SütunSerisi::yeni().ad("Alt").eksenler(1, 1).veri([4.0, 9.0, 6.0, 2.0]));
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    // Fare üst ızgarada, "B" kategorisi üzerinde.
    grafiği_boya(&mut yüzey, &seçenekler, &fareli_girdi((300.0, 120.0)));
    altın_karşılaştır("baglantili_imlec", &yüzey.döküm());
}

#[test]
fn ipucu_biçimleyici() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .ipucu(
            İpucu::yeni()
                .tetikleme(Tetikleme::Eksen)
                .biçimleyici(Biçimleyici::Şablon("{a} → {b}: {c} kWh".into())),
        )
        .x_ekseni(Eksen::kategori().veri(["Pzt", "Sal"]))
        .y_ekseni(Eksen::değer())
        .seri(SütunSerisi::yeni().ad("Tüketim").veri([120.0, 80.0]));
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    grafiği_boya(&mut yüzey, &seçenekler, &fareli_girdi((250.0, 300.0)));
    let döküm = yüzey.döküm();
    assert!(
        döküm.contains("Tüketim → Pzt: 120 kWh"),
        "biçimleyici satırı bulunamadı:\n{döküm}"
    );
}

#[test]
fn araç_kutusu_svg_düğmesi() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .araç_kutusu(AraçKutusu::yeni().svg_kaydet(true))
        .x_ekseni(Eksen::kategori().veri(["A", "B"]))
        .y_ekseni(Eksen::değer())
        .seri(SütunSerisi::yeni().ad("S").veri([3.0, 7.0]));
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());
    // Sıfırla + SVG kaydet: iki düğme, ikisi de tıklanabilir kutulu.
    assert_eq!(çıktı.araç_düğmeleri.len(), 2);
    assert!(çıktı
        .araç_düğmeleri
        .iter()
        .any(|(_, tür)| *tür == cizelge::AraçTürü::SvgKaydet));
}

#[test]
fn ağaç_haritası_inme() {
    let seri = AğaçHaritasıSerisi::yeni().ad("Disk").kökler([
        AğaçDüğümü::dal(
            "Medya",
            vec![
                AğaçDüğümü::yaprak("Video", 60.0),
                AğaçDüğümü::yaprak("Müzik", 25.0),
            ],
        ),
        AğaçDüğümü::yaprak("Sistem", 20.0),
    ]);
    let seçenekler = GrafikSeçenekleri::yeni().animasyon(false).seri(seri);
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    let girdi = BoyamaGirdisi {
        hiyerarşi_yolu: vec!["Medya".to_string()],
        ..Default::default()
    };
    let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &girdi);
    // Kırıntı şeridi: "Tümü" + "Medya".
    assert_eq!(çıktı.kırıntılar.len(), 2);
    let döküm = yüzey.döküm();
    assert!(döküm.contains("Tümü"), "kırıntı şeridi yok:\n{döküm}");
    assert!(döküm.contains("Video"), "inilen dal çizilmedi:\n{döküm}");
    altın_karşılaştır("agac_haritasi_inme", &döküm);

    // Yaprağa inme denemesi son geçerli düzeyde durur (görüntü değişmez).
    let mut yüzey2 = KayıtYüzeyi::yeni(800.0, 600.0);
    let girdi2 = BoyamaGirdisi {
        hiyerarşi_yolu: vec!["Medya".to_string(), "Video".to_string()],
        ..Default::default()
    };
    grafiği_boya(&mut yüzey2, &seçenekler, &girdi2);
    assert_eq!(döküm, yüzey2.döküm());
}

#[test]
fn güneş_patlaması_odak() {
    let seri = GüneşPatlamasıSerisi::yeni().ad("Trafik").kökler([
        AğaçDüğümü::dal(
            "Sosyal",
            vec![
                AğaçDüğümü::yaprak("Video", 14.0),
                AğaçDüğümü::yaprak("Blog", 6.0),
            ],
        ),
        AğaçDüğümü::yaprak("Doğrudan", 22.0),
    ]);
    let seçenekler = GrafikSeçenekleri::yeni().animasyon(false).seri(seri);
    let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
    let girdi = BoyamaGirdisi {
        hiyerarşi_yolu: vec!["Sosyal".to_string()],
        ..Default::default()
    };
    let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &girdi);
    // Merkezde "geri" düğmesi: tıklanınca yol bir üst düzeye kırpılır.
    assert_eq!(çıktı.kırıntılar.len(), 1);
    assert_eq!(çıktı.kırıntılar.first().map(|(_, u)| *u), Some(0));
    assert!(yüzey.döküm().contains("⌂"));
}

#[test]
fn grafo_gezinme_dönüşümü() {
    let kur = || {
        GrafikSeçenekleri::yeni().animasyon(false).seri(
            GrafoSerisi::yeni()
                .ad("Ağ")
                .düğümler([
                    GrafoDüğümü::yeni("A", 20.0),
                    GrafoDüğümü::yeni("B", 16.0),
                ])
                .bağlar([("A", "B")]),
        )
    };
    let mut düz = KayıtYüzeyi::yeni(800.0, 600.0);
    grafiği_boya(&mut düz, &kur(), &BoyamaGirdisi::default());
    // Yakınlaştırma + kaydırma + düğüm kayması çıktıyı değiştirir.
    let mut gezinmeli = KayıtYüzeyi::yeni(800.0, 600.0);
    let girdi = BoyamaGirdisi {
        grafo_görünümü: (24.0, -10.0, 1.5),
        grafo_kaymaları: vec![(1, 30.0, 12.0)],
        ..Default::default()
    };
    grafiği_boya(&mut gezinmeli, &kur(), &girdi);
    assert_ne!(düz.döküm(), gezinmeli.döküm());
}

#[test]
fn çentik_hizalama() {
    // İki değer y-ekseni: hizalama açıkken ikisi de aynı sayıda bölmeye
    // sahip olmalı (bölme çizgileri üst üste düşer).
    let kur = |hizala: bool| {
        GrafikSeçenekleri::yeni()
            .animasyon(false)
            .x_ekseni(Eksen::kategori().veri(["A", "B", "C"]))
            .y_ekseni_ekle(Eksen::değer())
            .y_ekseni_ekle(Eksen::değer().ölçekli(true).çentik_hizala(hizala))
            .seri(ÇizgiSerisi::yeni().ad("Sıcaklık").veri([10.0, 24.0, 17.0]))
            .seri(
                ÇizgiSerisi::yeni()
                    .ad("Nem")
                    .eksenler(0, 1)
                    .veri([53.0, 61.0, 47.0]),
            )
    };
    let hizalı = boya_ve_dök(kur(true));
    let serbest = boya_ve_dök(kur(false));
    assert_ne!(hizalı, serbest, "hizalama çıktıyı değiştirmeli");
    altın_karşılaştır("centik_hizalama", &hizalı);
}

#[test]
fn otomatik_örnekleme() {
    // 6000 nokta, örnekleme SEÇİLMEDEN: piksel başına birden çok nokta
    // düştüğü için LTTB kendiliğinden devreye girer (progressive karşılığı).
    let veri: Vec<f64> = (0..6000).map(|i| ((i % 97) as f64) * 0.5).collect();
    let kategoriler: Vec<String> = (0..6000).map(|i| format!("N{i}")).collect();
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(kategoriler))
        .y_ekseni(Eksen::değer())
        .seri(ÇizgiSerisi::yeni().ad("Büyük").sembol_göster(false).veri(veri));
    let döküm = boya_ve_dök(seçenekler);
    // Çizgi yolu, ızgara genişliğini (800px'ten küçük) aşan nokta içermemeli:
    // tek "çiz" satırındaki nokta sayısı kabaca hedefe inmiş olmalı.
    let en_uzun_satır = döküm.lines().map(str::len).max().unwrap_or(0);
    assert!(
        en_uzun_satır < 40_000,
        "örnekleme devreye girmedi: en uzun satır {en_uzun_satır} bayt"
    );
}

/// PNG dışa aktarım: imza, boyutlar ve tema farkı (`png` özelliğiyle).
#[cfg(feature = "png")]
#[test]
fn png_dışa_aktarım() {
    let kur = |koyu: bool| {
        GrafikSeçenekleri::yeni()
            .koyu(koyu)
            .başlık(Başlık::yeni().metin("PNG Deneme"))
            .x_ekseni(Eksen::kategori().veri(["A", "B", "C"]))
            .y_ekseni(Eksen::değer())
            .animasyon(false)
            .seri(SütunSerisi::yeni().ad("S").veri([3.0, 7.0, 5.0]))
    };
    let baytlar = png_dışa_aktar(&kur(false), 400.0, 300.0, 2.0).unwrap();
    // PNG imzası.
    assert_eq!(baytlar.get(..8), Some(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A][..]));
    // IHDR boyutları: 2× ölçekle 800×600 fiziksel piksel.
    let genişlik = u32::from_be_bytes([baytlar[16], baytlar[17], baytlar[18], baytlar[19]]);
    let yükseklik = u32::from_be_bytes([baytlar[20], baytlar[21], baytlar[22], baytlar[23]]);
    assert_eq!((genişlik, yükseklik), (800, 600));
    // Koyu tema çıktısı açık temadan farklı olmalı.
    let koyu = png_dışa_aktar(&kur(true), 400.0, 300.0, 2.0).unwrap();
    assert_ne!(baytlar, koyu);
}

/// PikselYüzeyi, kayıt yüzeyiyle aynı boyama hattını çalıştırır: isabet
/// bölgeleri birebir aynı üretilmeli.
#[cfg(feature = "png")]
#[test]
fn piksel_yüzeyi_isabetleri_kayıtla_aynı() {
    let seçenekler = GrafikSeçenekleri::yeni()
        .animasyon(false)
        .x_ekseni(Eksen::kategori().veri(["A", "B"]))
        .y_ekseni(Eksen::değer())
        .seri(SütunSerisi::yeni().ad("S").veri([3.0, 7.0]));
    let mut kayıt = KayıtYüzeyi::yeni(800.0, 600.0);
    let kayıt_çıktısı = grafiği_boya(&mut kayıt, &seçenekler, &BoyamaGirdisi::default());
    let mut piksel = PikselYüzeyi::yeni(800.0, 600.0, 1.0).unwrap();
    let piksel_çıktısı = grafiği_boya(&mut piksel, &seçenekler, &BoyamaGirdisi::default());
    assert_eq!(kayıt_çıktısı.isabetler.len(), piksel_çıktısı.isabetler.len());
}
