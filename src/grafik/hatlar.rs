//! GL olmayan çekirdek ECharts `series.lines` çizicisi.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::sembol_çiz;
use crate::model::hatlar::{HatEfekti, HatNoktası, HatVerisi, HatlarSerisi};
use crate::renk::Renk;
use crate::tema;

struct ÇözümlüHat {
    yol: Yol,
    örnekler: Vec<(f32, f32)>,
}

/// Bir lines serisini verilen koordinat dönüştürücüsü üzerinde boyar.
#[allow(clippy::too_many_arguments)]
pub fn hatlar_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &HatlarSerisi,
    seri_sırası: usize,
    eşle: &dyn Fn(&HatNoktası) -> Option<(f32, f32)>,
    seri_rengi: Renk,
    ilerleme: f32,
    zaman_sn: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    for (veri_sırası, veri) in seri.veri.iter().enumerate() {
        let Some(çözüm) = hattı_çöz(seri, veri, eşle, ilerleme) else {
            continue;
        };
        let stil = veri.çizgi_stili.as_ref().unwrap_or(&seri.çizgi_stili);
        let renk = stil
            .renk
            .unwrap_or(seri_rengi)
            .opaklık(stil.opaklık.clamp(0.0, 1.0));
        yüzey.yol_çiz(&çözüm.yol, stil.kalınlık.max(0.0), renk, stil.tür);

        uç_sembollerini_çiz(yüzey, seri, veri, &çözüm.örnekler, seri_rengi);
        etiketi_çiz(yüzey, seri, veri, &çözüm.örnekler, renk);
        efekti_çiz(
            yüzey,
            veri.efekt.as_ref().unwrap_or(&seri.efekt),
            &çözüm.örnekler,
            renk,
            zaman_sn,
        );

        isabetler.push(İsabetBölgesi {
            seri_sırası,
            veri_sırası,
            seri_adı: seri.ad.clone(),
            ad: veri.ad.clone().or_else(|| uç_adı(veri)),
            değer: veri.değer.sayı(),
            geometri: İsabetGeometrisi::ÇokluÇizgi {
                noktalar: çözüm.örnekler,
                tolerans: (stil.kalınlık / 2.0 + 4.0).max(6.0),
            },
        });
    }
}

fn hattı_çöz(
    seri: &HatlarSerisi,
    veri: &HatVerisi,
    eşle: &dyn Fn(&HatNoktası) -> Option<(f32, f32)>,
    ilerleme: f32,
) -> Option<ÇözümlüHat> {
    let noktalar: Vec<(f32, f32)> = veri.koordinatlar.iter().filter_map(eşle).collect();
    if noktalar.len() < 2 {
        return None;
    }
    if seri.çoklu_çizgi {
        let örnekler = çoklu_çizgiyi_kısalt(&noktalar, ilerleme.clamp(0.0, 1.0));
        return yoldan_çöz(örnekler);
    }

    let başlangıç = noktalar.first().copied()?;
    let bitiş = noktalar.get(1).copied()?;
    let eğrilik = veri.eğrilik.unwrap_or(0.0);
    if eğrilik.abs() <= f32::EPSILON {
        let uç = ara_nokta(başlangıç, bitiş, ilerleme.clamp(0.0, 1.0));
        return yoldan_çöz(vec![başlangıç, uç]);
    }
    let kontrol = (
        (başlangıç.0 + bitiş.0) / 2.0 - (başlangıç.1 - bitiş.1) * eğrilik,
        (başlangıç.1 + bitiş.1) / 2.0 - (bitiş.0 - başlangıç.0) * eğrilik,
    );
    let son_t = ilerleme.clamp(0.0, 1.0);
    let örnekler: Vec<(f32, f32)> = (0..=32)
        .map(|sıra| ikinci_derece(başlangıç, kontrol, bitiş, son_t * sıra as f32 / 32.0))
        .collect();
    if son_t < 0.999 {
        return yoldan_çöz(örnekler);
    }
    let mut yol = Yol::yeni();
    yol.taşı(başlangıç);
    let k1 = ara_nokta(başlangıç, kontrol, 2.0 / 3.0);
    let k2 = ara_nokta(bitiş, kontrol, 2.0 / 3.0);
    yol.kübik(k1, k2, bitiş);
    Some(ÇözümlüHat { yol, örnekler })
}

fn yoldan_çöz(örnekler: Vec<(f32, f32)>) -> Option<ÇözümlüHat> {
    let başlangıç = örnekler.first().copied()?;
    if örnekler.len() < 2 {
        return None;
    }
    let mut yol = Yol::yeni();
    yol.taşı(başlangıç);
    for nokta in örnekler.iter().skip(1) {
        yol.çiz(*nokta);
    }
    Some(ÇözümlüHat { yol, örnekler })
}

fn çoklu_çizgiyi_kısalt(noktalar: &[(f32, f32)], ilerleme: f32) -> Vec<(f32, f32)> {
    if ilerleme >= 0.999 {
        return noktalar.to_vec();
    }
    let uzunluklar: Vec<f32> = noktalar
        .windows(2)
        .filter_map(|uçlar| match uçlar {
            [a, b] => Some(uzaklık(*a, *b)),
            _ => None,
        })
        .collect();
    let hedef = uzunluklar.iter().sum::<f32>() * ilerleme;
    let Some(başlangıç) = noktalar.first().copied() else {
        return Vec::new();
    };
    let mut sonuç = vec![başlangıç];
    let mut geçen = 0.0;
    for (uçlar, uzunluk) in noktalar.windows(2).zip(uzunluklar) {
        let [a, b] = uçlar else { continue };
        if geçen + uzunluk <= hedef {
            sonuç.push(*b);
            geçen += uzunluk;
            continue;
        }
        let oran = ((hedef - geçen) / uzunluk.max(f32::EPSILON)).clamp(0.0, 1.0);
        sonuç.push(ara_nokta(*a, *b, oran));
        break;
    }
    sonuç
}

fn uç_sembollerini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &HatlarSerisi,
    veri: &HatVerisi,
    noktalar: &[(f32, f32)],
    renk: Renk,
) {
    let semboller = veri.semboller.as_ref().unwrap_or(&seri.semboller);
    let boyutlar = veri.sembol_boyutları.unwrap_or(seri.sembol_boyutları);
    if let Some(başlangıç) = noktalar.first().copied() {
        sembol_çiz(yüzey, &semboller[0], başlangıç, boyutlar[0], renk);
    }
    if let Some(bitiş) = noktalar.last().copied() {
        sembol_çiz(yüzey, &semboller[1], bitiş, boyutlar[1], renk);
    }
}

fn etiketi_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &HatlarSerisi,
    veri: &HatVerisi,
    noktalar: &[(f32, f32)],
    renk: Renk,
) {
    let etiket = veri.etiket.as_ref().unwrap_or(&seri.etiket);
    if !etiket.göster {
        return;
    }
    let Some(konum) = noktalar.last().copied() else {
        return;
    };
    let ham = veri.ad.clone().or_else(|| uç_adı(veri)).unwrap_or_default();
    let metin = etiket
        .biçimleyici
        .as_ref()
        .map(|biçimleyici| biçimleyici.uygula(veri.değer.sayı().unwrap_or(f64::NAN), &ham))
        .unwrap_or(ham);
    yüzey.yazı(
        &metin,
        (konum.0 + 5.0, konum.1),
        YatayHiza::Sol,
        DikeyHiza::Orta,
        etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
        etiket.yazı.renk.unwrap_or(renk),
        etiket.yazı.kalın,
    );
}

fn efekti_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    efekt: &HatEfekti,
    noktalar: &[(f32, f32)],
    çizgi_rengi: Renk,
    zaman_sn: f32,
) {
    if !efekt.göster || noktalar.len() < 2 {
        return;
    }
    let toplam = yol_uzunluğu(noktalar);
    if toplam <= f32::EPSILON {
        return;
    }
    let gecikmeli = (zaman_sn - efekt.gecikme_ms.max(0.0) / 1000.0).max(0.0);
    let dönem = if efekt.sabit_hız > 0.0 {
        toplam / efekt.sabit_hız
    } else {
        efekt.dönem_sn.max(0.001)
    };
    let ham = gecikmeli / dönem;
    if !efekt.döngü && ham > 1.0 {
        return;
    }
    let mut oran = if efekt.döngü {
        ham.rem_euclid(1.0)
    } else {
        ham.clamp(0.0, 1.0)
    };
    if efekt.gidiş_dönüş {
        oran = if oran <= 0.5 {
            oran * 2.0
        } else {
            (1.0 - oran) * 2.0
        };
    }
    let renk = efekt.renk.unwrap_or(çizgi_rengi);
    let iz = efekt.iz_uzunluğu.clamp(0.0, 1.0);
    if iz > 0.0 {
        for sıra in (1..=8).rev() {
            let uzaklık = iz * sıra as f32 / 8.0;
            let t = (oran - uzaklık).max(0.0);
            if let Some(nokta) = yol_noktası(noktalar, t) {
                let alfa = (1.0 - sıra as f32 / 9.0) * 0.45;
                sembol_çiz(
                    yüzey,
                    &efekt.sembol,
                    nokta,
                    efekt.sembol_boyutu * (1.0 - sıra as f32 / 16.0),
                    renk.opaklık(alfa),
                );
            }
        }
    }
    if let Some(nokta) = yol_noktası(noktalar, oran) {
        sembol_çiz(yüzey, &efekt.sembol, nokta, efekt.sembol_boyutu, renk);
    }
}

fn uç_adı(veri: &HatVerisi) -> Option<String> {
    match (&veri.kaynak_adı, &veri.hedef_adı) {
        (Some(kaynak), Some(hedef)) => Some(format!("{kaynak} > {hedef}")),
        (Some(kaynak), None) => Some(kaynak.clone()),
        (None, Some(hedef)) => Some(hedef.clone()),
        (None, None) => None,
    }
}

fn ikinci_derece(
    başlangıç: (f32, f32),
    kontrol: (f32, f32),
    bitiş: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let ters = 1.0 - t;
    (
        ters * ters * başlangıç.0 + 2.0 * ters * t * kontrol.0 + t * t * bitiş.0,
        ters * ters * başlangıç.1 + 2.0 * ters * t * kontrol.1 + t * t * bitiş.1,
    )
}

fn ara_nokta(a: (f32, f32), b: (f32, f32), t: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

fn uzaklık(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
}

fn yol_uzunluğu(noktalar: &[(f32, f32)]) -> f32 {
    noktalar
        .windows(2)
        .filter_map(|uçlar| match uçlar {
            [a, b] => Some(uzaklık(*a, *b)),
            _ => None,
        })
        .sum()
}

fn yol_noktası(noktalar: &[(f32, f32)], oran: f32) -> Option<(f32, f32)> {
    let hedef = yol_uzunluğu(noktalar) * oran.clamp(0.0, 1.0);
    let mut geçen = 0.0;
    for uçlar in noktalar.windows(2) {
        let [a, b] = uçlar else { continue };
        let uzunluk = uzaklık(*a, *b);
        if geçen + uzunluk >= hedef {
            return Some(ara_nokta(
                *a,
                *b,
                ((hedef - geçen) / uzunluk.max(f32::EPSILON)).clamp(0.0, 1.0),
            ));
        }
        geçen += uzunluk;
    }
    noktalar.last().copied()
}

#[cfg(test)]
mod testler {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::model::hatlar::{HatKoordinatSistemi, HatVerisi};

    fn bütün_grafiği_çiz(seçenekler: crate::model::secenekler::GrafikSeçenekleri) {
        let mut yüzey = KayıtYüzeyi::yeni(640.0, 360.0);
        let çıktı = crate::cizim::gorunum::grafiği_boya(
            &mut yüzey,
            &seçenekler,
            &crate::cizim::gorunum::BoyamaGirdisi {
                zaman_sn: 1.5,
                ..Default::default()
            },
        );
        assert_eq!(çıktı.isabetler.len(), 1, "{}", yüzey.döküm());
    }

    #[test]
    fn düz_eğri_sembol_etiket_ve_efekt_çizilir() {
        let seri = HatlarSerisi::yeni(HatKoordinatSistemi::Kartezyen2B)
            .semboller(
                crate::model::seri::Sembol::Daire,
                crate::model::seri::Sembol::Üçgen,
            )
            .etiket(crate::model::stil::Etiket::yeni().göster(true))
            .efekt(HatEfekti::yeni().göster(true))
            .veri([HatVerisi::yeni([(0.0, 0.0), (1.0, 1.0)])
                .uç_adları("A", "B")
                .eğrilik(0.2)]);
        let mut yüzey = KayıtYüzeyi::yeni(100.0, 100.0);
        let mut isabetler = Vec::new();
        hatlar_çiz(
            &mut yüzey,
            &seri,
            0,
            &|nokta| {
                Some((
                    nokta.x.sayı()? as f32 * 80.0 + 10.0,
                    nokta.y.sayı()? as f32 * 80.0 + 10.0,
                ))
            },
            Renk::onaltılık(0x3366cc),
            1.0,
            2.0,
            &mut isabetler,
        );
        assert_eq!(isabetler.len(), 1);
        assert!(!yüzey.komutlar.is_empty());
    }

    #[test]
    fn kartezyen_bağı_uçtan_uca_çizilir() {
        let seri = HatlarSerisi::yeni(HatKoordinatSistemi::Kartezyen2B)
            .veri([HatVerisi::yeni([(0.0, 2.0), (10.0, 8.0)])]);
        bütün_grafiği_çiz(
            crate::model::secenekler::GrafikSeçenekleri::yeni()
                .x_ekseni(crate::model::eksen::Eksen::değer())
                .y_ekseni(crate::model::eksen::Eksen::değer())
                .seri(seri),
        );
    }

    #[test]
    fn kutupsal_bağı_uçtan_uca_çizilir() {
        let seri = HatlarSerisi::yeni(HatKoordinatSistemi::Kutupsal)
            .veri([HatVerisi::yeni([(0.0, 2.0), (3.0, 8.0)])]);
        bütün_grafiği_çiz(
            crate::model::secenekler::GrafikSeçenekleri::yeni()
                .kutupsal(crate::model::kutupsal::KutupsalKoordinat::yeni())
                .seri(seri),
        );
    }

    #[test]
    fn takvim_bağı_uçtan_uca_çizilir() {
        let takvim = crate::model::takvim::TakvimKoordinatı::yıl(2024);
        let başlangıç = takvim.aralık.başlangıç_ms as i64;
        let gün = 86_400_000i64;
        let seri = HatlarSerisi::yeni(HatKoordinatSistemi::Takvim).veri([HatVerisi::yeni([
            (başlangıç, 0.0),
            (başlangıç + 40 * gün, 0.0),
        ])]);
        bütün_grafiği_çiz(
            crate::model::secenekler::GrafikSeçenekleri::yeni()
                .takvim(takvim)
                .seri(seri),
        );
    }

    #[test]
    fn matris_bağı_uçtan_uca_çizilir() {
        let matris = crate::model::matris::MatrisKoordinatı::yeni()
            .x(crate::model::matris::MatrisBoyutu::yeni().veri(["A", "B"]))
            .y(crate::model::matris::MatrisBoyutu::yeni().veri(["1", "2"]));
        let seri = HatlarSerisi::yeni(HatKoordinatSistemi::Matris)
            .veri([HatVerisi::yeni([("A", "1"), ("B", "2")])]);
        bütün_grafiği_çiz(
            crate::model::secenekler::GrafikSeçenekleri::yeni()
                .matris(matris)
                .seri(seri),
        );
    }
}
