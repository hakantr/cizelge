//! Mum (candlestick) ve kutu (boxplot) serisi çizimleri —
//! `echarts/src/chart/candlestick` ve `chart/boxplot` karşılıkları.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{keskin, Yol, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::seri::{KutuSerisi, MumSerisi};
use crate::renk::{Dolgu, Renk};

/// Mum serisini çizer. Veri sırası: `[açılış, kapanış, en düşük, en yüksek]`.
pub fn mum_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &MumSerisi,
    genel_sıra: usize,
    kartezyen: &Kartezyen2B,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let bant = kartezyen.x.bant_genişliği();
    let gövde_genişliği = (bant * seri.gövde_oranı.clamp(0.05, 1.0)).max(1.0);
    let alan = kartezyen.alan;

    let gövde = |ç: &mut dyn ÇizimYüzeyi,
                     isabetler: &mut Vec<İsabetBölgesi>| {
        for (i, öğe) in seri.veri.iter().enumerate() {
            let Some(dizi) = öğe.değer.dizi() else { continue };
            let (Some(&açılış), Some(&kapanış), Some(&en_düşük), Some(&en_yüksek)) =
                (dizi.first(), dizi.get(1), dizi.get(2), dizi.get(3))
            else {
                continue;
            };
            let yükselen = kapanış >= açılış;
            let renk = if yükselen { seri.yükselen_renk } else { seri.düşen_renk };

            let x = keskin(kartezyen.x.veriden_piksele(i as f64));
            let gövde_üst = kartezyen.y.veriden_piksele(açılış.max(kapanış));
            let gövde_alt = kartezyen.y.veriden_piksele(açılış.min(kapanış));
            let tepe = kartezyen.y.veriden_piksele(en_yüksek);
            let dip = kartezyen.y.veriden_piksele(en_düşük);

            // Fitiller (gövdenin üstünde ve altında).
            let mut fitil = Yol::yeni();
            fitil.taşı((x, tepe));
            fitil.çiz((x, gövde_üst));
            fitil.taşı((x, gövde_alt));
            fitil.çiz((x, dip));
            çizici_fitil(ç, &fitil, seri.kenarlık_kalınlığı, renk);

            // Gövde.
            let d = Dikdörtgen::yeni(
                x - gövde_genişliği / 2.0,
                gövde_üst,
                gövde_genişliği,
                (gövde_alt - gövde_üst).max(1.0),
            );
            ç.dikdörtgen(
                d,
                &Dolgu::Düz(renk),
                [0.0; 4],
                Some((seri.kenarlık_kalınlığı, renk)),
            );

            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: i,
                seri_adı: seri.ad.clone(),
                ad: öğe.ad.clone(),
                değer: Some(kapanış),
                geometri: İsabetGeometrisi::Dikdörtgen(Dikdörtgen::yeni(
                    x - gövde_genişliği / 2.0,
                    tepe.min(dip),
                    gövde_genişliği,
                    (dip - tepe).abs().max(1.0),
                )),
            });
        }
    };

    if ilerleme >= 0.999 {
        gövde(çizici, isabetler);
    } else {
        // Giriş animasyonu: soldan sağa açılan kırpma.
        let kırpma = Dikdörtgen::yeni(
            alan.x,
            0.0,
            alan.genişlik * ilerleme.clamp(0.0, 1.0),
            çizici.yükseklik(),
        );
        let mut geçici = Vec::new();
        çizici.kırpılı(kırpma, &mut |ç| gövde(ç, &mut geçici));
        isabetler.append(&mut geçici);
    }
}

fn çizici_fitil(ç: &mut dyn ÇizimYüzeyi, yol: &Yol, kalınlık: f32, renk: Renk) {
    ç.yol_çiz(yol, kalınlık, renk, crate::model::stil::ÇizgiTürü::Düz);
}

/// Kutu serisini çizer. Veri sırası:
/// `[en düşük, Ç1, ortanca, Ç3, en yüksek]`.
pub fn kutu_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &KutuSerisi,
    genel_sıra: usize,
    kartezyen: &Kartezyen2B,
    seri_rengi: Renk,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let bant = kartezyen.x.bant_genişliği();
    let gövde_genişliği = (bant * seri.gövde_oranı.clamp(0.05, 1.0)).max(1.0);
    let kapak_genişliği = gövde_genişliği / 2.0;
    let renk = seri
        .öğe_stili
        .kenarlık_rengi
        .unwrap_or(seri_rengi);
    let dolgu = seri
        .öğe_stili
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(Renk::BEYAZ));
    let kalınlık = if seri.öğe_stili.kenarlık_kalınlığı > 0.0 {
        seri.öğe_stili.kenarlık_kalınlığı
    } else {
        1.0
    };

    for (i, öğe) in seri.veri.iter().enumerate() {
        let Some(dizi) = öğe.değer.dizi() else { continue };
        let (Some(&en_düşük), Some(&ç1), Some(&ortanca), Some(&ç3), Some(&en_yüksek)) = (
            dizi.first(),
            dizi.get(1),
            dizi.get(2),
            dizi.get(3),
            dizi.get(4),
        ) else {
            continue;
        };

        let x = keskin(kartezyen.x.veriden_piksele(i as f64));
        let y_düşük = keskin(kartezyen.y.veriden_piksele(en_düşük));
        let y_ç1 = kartezyen.y.veriden_piksele(ç1);
        let y_ortanca = keskin(kartezyen.y.veriden_piksele(ortanca));
        let y_ç3 = kartezyen.y.veriden_piksele(ç3);
        let y_yüksek = keskin(kartezyen.y.veriden_piksele(en_yüksek));

        // Bıyıklar ve kapaklar.
        let mut yol = Yol::yeni();
        yol.taşı((x, y_yüksek));
        yol.çiz((x, y_ç3.min(y_ç1)));
        yol.taşı((x, y_ç3.max(y_ç1)));
        yol.çiz((x, y_düşük));
        yol.taşı((x - kapak_genişliği / 2.0, y_yüksek));
        yol.çiz((x + kapak_genişliği / 2.0, y_yüksek));
        yol.taşı((x - kapak_genişliği / 2.0, y_düşük));
        yol.çiz((x + kapak_genişliği / 2.0, y_düşük));
        çizici.yol_çiz(&yol, kalınlık, renk, crate::model::stil::ÇizgiTürü::Düz);

        // Gövde: Ç1–Ç3 kutusu.
        let d = Dikdörtgen::yeni(
            x - gövde_genişliği / 2.0,
            y_ç3.min(y_ç1),
            gövde_genişliği,
            (y_ç1 - y_ç3).abs().max(1.0),
        );
        çizici.dikdörtgen(d, &dolgu, [0.0; 4], Some((kalınlık, renk)));

        // Ortanca çizgisi.
        çizici.çizgi(
            (x - gövde_genişliği / 2.0, y_ortanca),
            (x + gövde_genişliği / 2.0, y_ortanca),
            kalınlık.max(1.5),
            renk,
            crate::model::stil::ÇizgiTürü::Düz,
        );

        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: i,
            seri_adı: seri.ad.clone(),
            ad: öğe.ad.clone(),
            değer: Some(ortanca),
            geometri: İsabetGeometrisi::Dikdörtgen(Dikdörtgen::yeni(
                x - gövde_genişliği / 2.0,
                y_yüksek.min(y_düşük),
                gövde_genişliği,
                (y_düşük - y_yüksek).abs().max(1.0),
            )),
        });
    }
}
