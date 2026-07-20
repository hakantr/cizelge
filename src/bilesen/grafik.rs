//! `graphic` bileşenini ortak hiyerarşik sahneye dönüştürür.

use std::collections::BTreeMap;

use crate::cizim::yuzey::{DikeyHiza, YatayHiza};
use crate::cizim::{Sahne, SahneDüğümü, SahneMetni, SahneÖğesi};
use crate::koordinat::Dikdörtgen;
use crate::model::grafik_bileseni::{
    GrafikBileşeni, GrafikMetinKonumu, GrafikÖğesi, GrafikÖğeİçeriği,
};
use crate::model::{DikeyKonum, YatayKonum};

/// Sahne yaprağından kullanıcının `graphic` öğesine geri eşleme.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GrafikÖğeBilgisi {
    pub kimlik: Option<String>,
    pub ad: Option<String>,
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
    let mut sonuç = GrafikSahnesi::default();
    for (sıra, öğe) in grafik.öğeler.iter().enumerate() {
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
    };
    let mut çocuklar = Vec::new();

    match &öğe.içerik {
        GrafikÖğeİçeriği::Grup(alt_öğeler) => {
            for (sıra, çocuk) in alt_öğeler.iter().enumerate() {
                çocuklar.push(öğeyi_hazırla(
                    çocuk,
                    &format!("{yol}.{sıra}"),
                    kap,
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
    düğüm.kırpmalar = öğe.kırpmalar.clone();
    düğüm.zlevel = öğe.zlevel;
    düğüm.z = öğe.z;
    düğüm.z2 = öğe.z2;
    düğüm.görünür = !öğe.yoksay;
    düğüm.sessiz = öğe.sessiz;
    düğüm.sürüklenebilir = öğe.sürüklenebilir;
    düğüm.imleç = öğe.imleç.clone();

    if let Some(sınır) = düğüm.sınır_kutusu() {
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
    use crate::cizim::{KayıtYüzeyi, SahneStili};
    use crate::model::grafik_bileseni::GrafikBağlıMetni;
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
}
