//! `graphic` bileşenini ortak hiyerarşik sahneye dönüştürür.

use std::collections::BTreeMap;

use crate::cizim::yuzey::{DikeyHiza, YatayHiza};
use crate::cizim::{Sahne, SahneDüğümü, SahneMetni, SahneÖğesi};
use crate::eylem::EylemDeğeri;
use crate::koordinat::Dikdörtgen;
use crate::model::grafik_bileseni::{
    GrafikAnahtarKareAnimasyonu, GrafikBileşeni, GrafikMetinKonumu, GrafikSınırlamaKipi,
    GrafikÖğesi, GrafikÖğeİçeriği,
};
use crate::model::{DikeyKonum, YatayKonum};

/// Sahne yaprağından kullanıcının `graphic` öğesine geri eşleme.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GrafikÖğeBilgisi {
    pub kimlik: Option<String>,
    pub ad: Option<String>,
    pub bilgi: BTreeMap<String, EylemDeğeri>,
    /// Kök `graphic.elements` dizisinden başlayan iç içe öğe sıraları.
    pub yol: Vec<usize>,
    pub sürüklenebilir: bool,
}

/// Çizime hazır sahne ve olay hedefi bilgileri.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GrafikSahnesi {
    pub sahne: Sahne,
    pub öğe_bilgileri: BTreeMap<String, GrafikÖğeBilgisi>,
}

/// Tuval boyutuna göre yüzde/kenar yerleşimlerini çözüp sahneyi kurar.
pub fn grafik_sahnesi_hazırla(
    grafik: &GrafikBileşeni,
    genişlik: f32,
    yükseklik: f32,
) -> GrafikSahnesi {
    grafik_sahnesi_hazırla_zamanda(grafik, genişlik, yükseklik, 0.0, true)
}

/// Graphic keyframe zamanını değerlendirip aynı ortak sahneyi kurar.
/// `zaman_sn`, görünümün monoton animasyon saatidir; `animasyon_açık=false`
/// ECharts `option.animation: false` gibi bütün keyframe izlerini tabanda
/// dondurur.
pub fn grafik_sahnesi_hazırla_zamanda(
    grafik: &GrafikBileşeni,
    genişlik: f32,
    yükseklik: f32,
    zaman_sn: f32,
    animasyon_açık: bool,
) -> GrafikSahnesi {
    let mut sonuç = GrafikSahnesi::default();
    for (sıra, öğe) in grafik.öğeler.iter().enumerate() {
        let zamanlı;
        let öğe = if animasyon_açık {
            zamanlı = grafik_öğesini_zamanda(öğe, zaman_sn);
            &zamanlı
        } else {
            öğe
        };
        let düğüm = öğeyi_hazırla(
            öğe,
            &sıra.to_string(),
            (genişlik, yükseklik),
            &mut sonuç.öğe_bilgileri,
        );
        // İç kimlikler dizin yolundan üretildiği için ekleme yalnız dahili
        // bir programlama hatasında başarısız olabilir.
        sonuç
            .sahne
            .ekle(düğüm)
            .expect("graphic iç sahne kimlikleri benzersiz olmalı");
    }
    sonuç
}

fn grafik_öğesini_zamanda(öğe: &GrafikÖğesi, zaman_sn: f32) -> GrafikÖğesi {
    let mut sonuç = öğe.clone();
    if let GrafikÖğeİçeriği::Grup(çocuklar) = &öğe.içerik {
        sonuç.içerik = GrafikÖğeİçeriği::Grup(
            çocuklar
                .iter()
                .map(|çocuk| grafik_öğesini_zamanda(çocuk, zaman_sn))
                .collect(),
        );
    }
    for animasyon in &öğe.anahtar_kare_animasyonları {
        sonuç = tek_anahtar_kare_animasyonunu_uygula(&sonuç, animasyon, zaman_sn);
    }
    sonuç
}

fn tek_anahtar_kare_animasyonunu_uygula(
    taban: &GrafikÖğesi,
    animasyon: &GrafikAnahtarKareAnimasyonu,
    zaman_sn: f32,
) -> GrafikÖğesi {
    if animasyon.süre_ms <= 0.0 || animasyon.kareler.is_empty() {
        return taban.clone();
    }
    let geçen = zaman_sn.max(0.0) * 1000.0 - animasyon.gecikme_ms;
    if geçen < 0.0 {
        return taban.clone();
    }
    let ham_oran = if animasyon.döngü {
        geçen.rem_euclid(animasyon.süre_ms) / animasyon.süre_ms
    } else {
        (geçen / animasyon.süre_ms).clamp(0.0, 1.0)
    };
    let oran = animasyon.yumuşatma.uygula(ham_oran);
    let mut kareler = animasyon.kareler.iter().collect::<Vec<_>>();
    kareler.sort_by(|a, b| a.yüzde.total_cmp(&b.yüzde));

    let mut önceki_yüzde = 0.0_f32;
    let mut önceki = taban.clone();
    let mut güncel = taban.clone();
    for kare in kareler {
        kare.uygula(&mut güncel);
        if oran <= kare.yüzde {
            let açıklık = kare.yüzde - önceki_yüzde;
            let yerel = if açıklık <= f32::EPSILON {
                1.0
            } else {
                ((oran - önceki_yüzde) / açıklık).clamp(0.0, 1.0)
            };
            return crate::grafik::ozel::öğeyi_ara_değerle(
                &önceki,
                &güncel,
                kare.yumuşatma.uygula(yerel),
                &["all".to_owned()],
            );
        }
        önceki_yüzde = kare.yüzde;
        önceki = güncel.clone();
    }
    güncel
}

fn öğeyi_hazırla(
    öğe: &GrafikÖğesi,
    yol: &str,
    kap: (f32, f32),
    bilgiler: &mut BTreeMap<String, GrafikÖğeBilgisi>,
) -> SahneDüğümü {
    let temel = format!("graphic:{yol}");
    let bilgi = GrafikÖğeBilgisi {
        kimlik: öğe.kimlik.clone(),
        ad: öğe.ad.clone(),
        bilgi: öğe.bilgi.clone(),
        yol: yol
            .split('.')
            .filter_map(|parça| parça.parse::<usize>().ok())
            .collect(),
        sürüklenebilir: öğe.sürüklenebilir,
    };
    let mut çocuklar = Vec::new();

    match &öğe.içerik {
        GrafikÖğeİçeriği::Grup(alt_öğeler) => {
            let çocuk_kabı = öğe.grup_boyutu.unwrap_or((0.0, 0.0));
            for (sıra, çocuk) in alt_öğeler.iter().enumerate() {
                çocuklar.push(öğeyi_hazırla(
                    çocuk,
                    &format!("{yol}.{sıra}"),
                    çocuk_kabı,
                    bilgiler,
                ));
            }
        }
        GrafikÖğeİçeriği::Şekil(şekil) => {
            let kimlik = format!("{temel}:content");
            bilgiler.insert(kimlik.clone(), bilgi.clone());
            let mut stil = öğe.stil.clone();
            if öğe.görünmez {
                stil.opaklık = 0.0;
            }
            let mut düğüm = SahneDüğümü::şekil(kimlik, şekil.clone()).stil(stil);
            düğüm.durum_stilleri = öğe.durum_stilleri.clone();
            düğüm.durum = öğe.durum;
            düğüm.odak = öğe.odak;
            düğüm.sessiz = öğe.sessiz;
            düğüm.sürüklenebilir = öğe.sürüklenebilir;
            düğüm.imleç = öğe.imleç.clone();
            çocuklar.push(düğüm);
        }
        GrafikÖğeİçeriği::Metin(metin) => {
            let kimlik = format!("{temel}:content");
            bilgiler.insert(kimlik.clone(), bilgi.clone());
            let mut stil = öğe.stil.clone();
            if öğe.görünmez {
                stil.opaklık = 0.0;
            }
            let mut düğüm = SahneDüğümü::metin(kimlik, metin.clone()).stil(stil);
            düğüm.durum_stilleri = öğe.durum_stilleri.clone();
            düğüm.durum = öğe.durum;
            düğüm.odak = öğe.odak;
            düğüm.sessiz = öğe.sessiz;
            düğüm.sürüklenebilir = öğe.sürüklenebilir;
            düğüm.imleç = öğe.imleç.clone();
            çocuklar.push(düğüm);
        }
        GrafikÖğeİçeriği::Resim(resim) => {
            let kimlik = format!("{temel}:content");
            bilgiler.insert(kimlik.clone(), bilgi.clone());
            let mut stil = öğe.stil.clone();
            if öğe.görünmez {
                stil.opaklık = 0.0;
            }
            let mut düğüm = SahneDüğümü::resim(kimlik, resim.clone()).stil(stil);
            düğüm.durum_stilleri = öğe.durum_stilleri.clone();
            düğüm.durum = öğe.durum;
            düğüm.odak = öğe.odak;
            düğüm.sessiz = öğe.sessiz;
            düğüm.sürüklenebilir = öğe.sürüklenebilir;
            düğüm.imleç = öğe.imleç.clone();
            çocuklar.push(düğüm);
        }
    }

    let mut düğüm = SahneDüğümü::grup(temel.clone());
    düğüm.öğe = SahneÖğesi::Grup(çocuklar);
    // zrender `textContent`, ev sahibinin yerleşim sınır kutusunu büyütmez.
    // Bağlı metni eklemeden önceki ağacı bu amaçla ayrı tut.
    let mut yerleşim_düğümü = düğüm.clone();

    if let Some(bağlı) = &öğe.bağlı_metin {
        let sınır = düğüm
            .sınır_kutusu()
            .unwrap_or_else(|| Dikdörtgen::yeni(0.0, 0.0, 0.0, 0.0));
        let (konum, yatay, dikey) = bağlı_metin_yerleşimi(bağlı.konum, sınır);
        let mut metin = SahneMetni::yeni(&bağlı.metin, konum);
        metin.yatay = yatay;
        metin.dikey = dikey;
        metin.boyut = bağlı.boyut;
        metin.renk = bağlı.renk;
        metin.kalın = bağlı.kalın;
        let kimlik = format!("{temel}:textContent");
        bilgiler.insert(kimlik.clone(), bilgi);
        let mut metin_düğümü = SahneDüğümü::metin(kimlik, metin);
        metin_düğümü.sessiz = bağlı.sessiz || öğe.sessiz;
        if öğe.görünmez {
            metin_düğümü.stil.opaklık = 0.0;
        }
        if let SahneÖğesi::Grup(çocuklar) = &mut düğüm.öğe {
            çocuklar.push(metin_düğümü);
        }
    }

    düğüm.dönüşüm = öğe.dönüşüm;
    yerleşim_düğümü.dönüşüm = öğe.dönüşüm;
    if öğe.sınırlama == GrafikSınırlamaKipi::Ham {
        yerleşim_düğümü.dönüşüm = crate::cizim::YerelDönüşüm::default();
    }
    düğüm.kırpmalar = öğe.kırpmalar.clone();
    düğüm.zlevel = öğe.zlevel;
    düğüm.z = öğe.z;
    düğüm.z2 = öğe.z2;
    düğüm.görünür = !öğe.yoksay;
    düğüm.sessiz = öğe.sessiz;
    düğüm.sürüklenebilir = öğe.sürüklenebilir;
    düğüm.imleç = öğe.imleç.clone();

    if let Some(sınır) = yerleşim_düğümü.sınır_kutusu() {
        let dx = yatay_kayma(&öğe.yerleşim, sınır, kap.0);
        let dy = dikey_kayma(&öğe.yerleşim, sınır, kap.1);
        düğüm.dönüşüm.x += dx;
        düğüm.dönüşüm.y += dy;
    }

    düğüm
}

fn yatay_kayma(
    yerleşim: &crate::model::grafik_bileseni::GrafikYerleşimi,
    sınır: Dikdörtgen,
    genişlik: f32,
) -> f32 {
    if let Some(sol) = yerleşim.sol {
        let hedef = match sol {
            YatayKonum::Sol => 0.0,
            YatayKonum::Orta => (genişlik - sınır.genişlik) / 2.0,
            YatayKonum::Sağ => genişlik - sınır.genişlik,
            YatayKonum::Değer(değer) => değer.çöz(genişlik),
        };
        hedef - sınır.x
    } else if let Some(sağ) = yerleşim.sağ {
        genişlik - sağ.çöz(genişlik) - sınır.sağ()
    } else {
        0.0
    }
}

fn dikey_kayma(
    yerleşim: &crate::model::grafik_bileseni::GrafikYerleşimi,
    sınır: Dikdörtgen,
    yükseklik: f32,
) -> f32 {
    if let Some(üst) = yerleşim.üst {
        let hedef = match üst {
            DikeyKonum::Üst => 0.0,
            DikeyKonum::Orta => (yükseklik - sınır.yükseklik) / 2.0,
            DikeyKonum::Alt => yükseklik - sınır.yükseklik,
            DikeyKonum::Değer(değer) => değer.çöz(yükseklik),
        };
        hedef - sınır.y
    } else if let Some(alt) = yerleşim.alt {
        yükseklik - alt.çöz(yükseklik) - sınır.alt()
    } else {
        0.0
    }
}

fn bağlı_metin_yerleşimi(
    konum: GrafikMetinKonumu,
    sınır: Dikdörtgen,
) -> ((f32, f32), YatayHiza, DikeyHiza) {
    match konum {
        GrafikMetinKonumu::İç => (sınır.merkez(), YatayHiza::Orta, DikeyHiza::Orta),
        GrafikMetinKonumu::Üst => ((sınır.merkez().0, sınır.y), YatayHiza::Orta, DikeyHiza::Alt),
        GrafikMetinKonumu::Alt => (
            (sınır.merkez().0, sınır.alt()),
            YatayHiza::Orta,
            DikeyHiza::Üst,
        ),
        GrafikMetinKonumu::Sol => ((sınır.x, sınır.merkez().1), YatayHiza::Sağ, DikeyHiza::Orta),
        GrafikMetinKonumu::Sağ => (
            (sınır.sağ(), sınır.merkez().1),
            YatayHiza::Sol,
            DikeyHiza::Orta,
        ),
        GrafikMetinKonumu::Değer(x, y) => {
            ((sınır.x + x, sınır.y + y), YatayHiza::Sol, DikeyHiza::Üst)
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::{KayıtYüzeyi, SahneStili, SahneŞekli, YerelDönüşüm};
    use crate::model::grafik_bileseni::{
        GrafikAnahtarKare, GrafikAnahtarKareAnimasyonu, GrafikBağlıMetni,
    };
    use crate::renk::{Dolgu, Renk};

    #[test]
    fn dikdörtgen_yerleşimi_bağlı_metni_ve_isabeti_ortaktır() {
        let grafik = GrafikBileşeni::yeni().öğe(
            GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(0.0, 0.0, 140.0, 24.0))
                .kimlik("düğme")
                .ad("Kapat")
                .sol(5.0)
                .üst(5.0)
                .stil(SahneStili {
                    dolgu: Some(Dolgu::Düz(Renk::onaltılık(0xeeeeee))),
                    çizgi_rengi: Some(Renk::onaltılık(0x999999)),
                    ..SahneStili::default()
                })
                .bağlı_metin(GrafikBağlıMetni::yeni("Collapse Axis Breaks").boyut(13.0)),
        );
        let hazır = grafik_sahnesi_hazırla(&grafik, 800.0, 600.0);
        assert_eq!(hazır.sahne.bul("graphic:0").unwrap().dönüşüm.x, 5.0);
        assert_eq!(hazır.sahne.bul("graphic:0").unwrap().dönüşüm.y, 5.0);
        let isabet = hazır.sahne.isabet((75.0, 17.0)).expect("düğme isabeti");
        assert_eq!(
            hazır.öğe_bilgileri[&isabet.kimlik].kimlik.as_deref(),
            Some("düğme")
        );

        let mut yüzey = KayıtYüzeyi::yeni(800.0, 600.0);
        hazır.sahne.çiz(&mut yüzey);
        assert!(
            yüzey
                .komutlar
                .iter()
                .any(|komut| komut.contains("Collapse Axis Breaks"))
        );
    }

    #[test]
    fn ignore_cizim_ve_isabet_listesinden_cikarir() {
        let grafik = GrafikBileşeni::yeni().öğe(
            GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(0.0, 0.0, 20.0, 20.0))
                .kimlik("gizli")
                .yoksay(true),
        );
        let hazır = grafik_sahnesi_hazırla(&grafik, 100.0, 100.0);
        assert!(hazır.sahne.isabet((10.0, 10.0)).is_none());
        let mut yüzey = KayıtYüzeyi::yeni(100.0, 100.0);
        hazır.sahne.çiz(&mut yüzey);
        assert!(yüzey.komutlar.is_empty());
    }

    #[test]
    fn anahtar_kare_gecikme_dongu_sekil_stil_ve_donusumu_ara_degerler() {
        let öğe = GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(0.0, 0.0, 10.0, 40.0))
            .dönüşüm(YerelDönüşüm {
                x: 20.0,
                y: 30.0,
                ..YerelDönüşüm::default()
            })
            .stil(SahneStili {
                dolgu: Some(Dolgu::Düz(Renk::SİYAH)),
                ..SahneStili::default()
            })
            .anahtar_kare_animasyonu(
                GrafikAnahtarKareAnimasyonu::yeni(1000.0)
                    .gecikme(-250.0)
                    .döngü(true)
                    .kare(
                        GrafikAnahtarKare::yeni(0.5)
                            .ölçek(1.0, 0.25)
                            .dolgu(Renk::BEYAZ),
                    )
                    .kare(GrafikAnahtarKare::yeni(1.0).ölçek(1.0, 1.0)),
            );
        let grafik = GrafikBileşeni::yeni().öğe(öğe);
        let çeyrek = grafik_sahnesi_hazırla_zamanda(&grafik, 100.0, 100.0, 0.0, true);
        let kök = çeyrek.sahne.bul("graphic:0").expect("animated root");
        assert!((kök.dönüşüm.ölçek_y - 0.625).abs() < 1e-5);

        let kapalı = grafik_sahnesi_hazırla_zamanda(&grafik, 100.0, 100.0, 0.0, false);
        assert_eq!(kapalı.sahne.bul("graphic:0").unwrap().dönüşüm.ölçek_y, 1.0);
    }

    #[test]
    fn görünmez_sürükleme_tutamacı_isabete_yol_ve_draggable_bilgisi_taşır() {
        let grafik = GrafikBileşeni::yeni().öğe(
            GrafikÖğesi::şekil(SahneŞekli::Daire {
                merkez: (0.0, 0.0),
                yarıçap: 10.0,
            })
            .kimlik("nokta")
            .bilgi("dataIndex", 2_i32)
            .dönüşüm(YerelDönüşüm {
                x: 40.0,
                y: 30.0,
                ..YerelDönüşüm::default()
            })
            .görünmez(true)
            .sürüklenebilir(true),
        );
        let hazır = grafik_sahnesi_hazırla(&grafik, 100.0, 100.0);
        let isabet = hazır.sahne.isabet((40.0, 30.0)).expect("invisible hit");
        let bilgi = &hazır.öğe_bilgileri[&isabet.kimlik];
        assert_eq!(bilgi.yol, vec![0]);
        assert!(bilgi.sürüklenebilir);
        assert_eq!(bilgi.kimlik.as_deref(), Some("nokta"));
        assert_eq!(
            bilgi.bilgi.get("dataIndex"),
            Some(&EylemDeğeri::from(2_i32))
        );
    }

    #[test]
    fn nested_group_boyutu_cocuk_yerlesimini_ve_ham_siniri_ayri_cozer() {
        let çocuk = GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(0.0, 0.0, 20.0, 10.0))
            .sol(YatayKonum::Orta)
            .üst(DikeyKonum::Orta);
        let grafik = GrafikBileşeni::yeni().öğe(
            GrafikÖğesi::grup([çocuk])
                .grup_boyutu(100.0, 80.0)
                .sınırlama(GrafikSınırlamaKipi::Ham)
                .sol(10.0)
                .üst(20.0),
        );
        let hazır = grafik_sahnesi_hazırla(&grafik, 400.0, 300.0);
        let grup = hazır.sahne.bul("graphic:0").unwrap();
        let çocuk = hazır.sahne.bul("graphic:0.0").unwrap();

        assert_eq!((çocuk.dönüşüm.x, çocuk.dönüşüm.y), (40.0, 35.0));
        assert_eq!((grup.dönüşüm.x, grup.dönüşüm.y), (-30.0, -15.0));
        assert!(hazır.sahne.isabet((20.0, 25.0)).is_some());
    }
}
