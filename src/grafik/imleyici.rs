//! İmleyici çizimi — `echarts/src/component/marker` görünümlerinin
//! karşılığı: im alanları serilerin altına, im çizgileri ve raptiyeler
//! serilerin üstüne boyanır.

use crate::cizim::{keskin, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::imleyici::{İmDeğeri, İmYönü, İmleyiciler};
use crate::model::seri::Seri;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

/// Serinin sonlu sayısal değerleri.
fn değerler(seri: &Seri) -> Vec<f64> {
    seri.veri()
        .iter()
        .filter_map(|ö| ö.değer.sayı())
        .filter(|d| d.is_finite())
        .collect()
}

/// İm değerini seriye göre çözer; veri yoksa `None`.
fn değer_çöz(değer: İmDeğeri, seri: &Seri) -> Option<f64> {
    match değer {
        İmDeğeri::Değer(d) => d.is_finite().then_some(d),
        İmDeğeri::Ortalama => {
            let d = değerler(seri);
            if d.is_empty() {
                None
            } else {
                Some(d.iter().sum::<f64>() / d.len() as f64)
            }
        }
        İmDeğeri::EnKüçük => değerler(seri).into_iter().reduce(f64::min),
        İmDeğeri::EnBüyük => değerler(seri).into_iter().reduce(f64::max),
    }
}

/// İstatistiğin denk geldiği veri sırası (im noktası x konumu için).
fn değer_sırası(değer: İmDeğeri, seri: &Seri) -> Option<usize> {
    let hedef = değer_çöz(değer, seri)?;
    match değer {
        İmDeğeri::Ortalama => {
            let sayaç = seri.veri().iter().filter(|ö| ö.değer.sayı().is_some()).count();
            (sayaç > 0).then_some(sayaç / 2)
        }
        _ => seri
            .veri()
            .iter()
            .position(|ö| ö.değer.sayı().map(|d| d == hedef).unwrap_or(false)),
    }
}

/// İm alanlarını çizer (serilerin altına).
pub fn im_alanlarını_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    imleyiciler: &İmleyiciler,
    seri: &Seri,
    kartezyen: &Kartezyen2B,
    seri_rengi: Renk,
) {
    let Some(alan_imi) = &imleyiciler.alan else { return };
    let alan = kartezyen.alan;
    let dolgu = alan_imi
        .stil
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(seri_rengi.opaklık(0.15)));
    let _ = seri;

    for (ad, tanım) in &alan_imi.veri {
        let x0 = tanım.x0.map(|v| kartezyen.x.veriden_piksele(v)).unwrap_or(alan.x);
        let x1 = tanım.x1.map(|v| kartezyen.x.veriden_piksele(v)).unwrap_or(alan.sağ());
        let y0 = tanım.y0.map(|v| kartezyen.y.veriden_piksele(v)).unwrap_or(alan.y);
        let y1 = tanım.y1.map(|v| kartezyen.y.veriden_piksele(v)).unwrap_or(alan.alt());
        let d = Dikdörtgen::yeni(
            x0.min(x1),
            y0.min(y1),
            (x1 - x0).abs(),
            (y1 - y0).abs(),
        );
        yüzey.dikdörtgen(d, &dolgu, [0.0; 4], None);

        if let Some(ad) = ad
            && (alan_imi.etiket.göster || !ad.is_empty()) {
                let boyut = alan_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                let renk = alan_imi.etiket.yazı.renk.unwrap_or(seri_rengi);
                yüzey.yazı(
                    ad,
                    (d.x + d.genişlik / 2.0, d.y + 4.0),
                    YatayHiza::Orta,
                    DikeyHiza::Üst,
                    boyut,
                    renk,
                    false,
                );
            }
    }
}

/// İm çizgilerini ve raptiyeleri çizer (serilerin üstüne).
pub fn im_çizgi_ve_noktalarını_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    imleyiciler: &İmleyiciler,
    seri: &Seri,
    kartezyen: &Kartezyen2B,
    seri_rengi: Renk,
) {
    let alan = kartezyen.alan;

    // 1) İm çizgileri.
    if let Some(çizgi_imi) = &imleyiciler.çizgi {
        let renk = çizgi_imi.stil.renk.unwrap_or(seri_rengi);
        for tanım in &çizgi_imi.veri {
            let Some(değer) = değer_çöz(tanım.değer, seri) else { continue };
            let etiket_metni = tanım
                .ad
                .clone()
                .map(|ad| format!("{ad}: {}", binlik_ayır(değer)))
                .unwrap_or_else(|| binlik_ayır(değer));
            match tanım.yön {
                İmYönü::Yatay => {
                    let y = keskin(kartezyen.y.veriden_piksele(değer));
                    yüzey.çizgi(
                        (alan.x, y),
                        (alan.sağ(), y),
                        çizgi_imi.stil.kalınlık,
                        renk.opaklık(çizgi_imi.stil.opaklık),
                        çizgi_imi.stil.tür,
                    );
                    if çizgi_imi.etiket.göster {
                        let boyut =
                            çizgi_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                        yüzey.yazı(
                            &etiket_metni,
                            (alan.sağ() + 4.0, y),
                            YatayHiza::Sol,
                            DikeyHiza::Orta,
                            boyut,
                            çizgi_imi.etiket.yazı.renk.unwrap_or(renk),
                            false,
                        );
                    }
                }
                İmYönü::Dikey => {
                    let x = keskin(kartezyen.x.veriden_piksele(değer));
                    yüzey.çizgi(
                        (x, alan.y),
                        (x, alan.alt()),
                        çizgi_imi.stil.kalınlık,
                        renk.opaklık(çizgi_imi.stil.opaklık),
                        çizgi_imi.stil.tür,
                    );
                    if çizgi_imi.etiket.göster {
                        let boyut =
                            çizgi_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                        yüzey.yazı(
                            &etiket_metni,
                            (x, alan.y - 4.0),
                            YatayHiza::Orta,
                            DikeyHiza::Alt,
                            boyut,
                            çizgi_imi.etiket.yazı.renk.unwrap_or(renk),
                            false,
                        );
                    }
                }
            }
        }
    }

    // 2) İm noktaları (raptiyeler).
    if let Some(nokta_imi) = &imleyiciler.nokta {
        for tanım in &nokta_imi.veri {
            let (x_değeri, y_değeri) = match (tanım.koordinat, tanım.değer) {
                (Some((x, y)), _) => (x, y),
                (None, Some(değer)) => {
                    let Some(sıra) = değer_sırası(değer, seri) else { continue };
                    let Some(y) = değer_çöz(değer, seri) else { continue };
                    let x = seri
                        .veri()
                        .get(sıra)
                        .and_then(|ö| ö.değer.x())
                        .unwrap_or(sıra as f64);
                    (x, y)
                }
                (None, None) => continue,
            };
            let (x, y) = kartezyen.nokta(x_değeri, y_değeri);
            raptiye_çiz(
                yüzey,
                (x, y),
                nokta_imi.boyut,
                seri_rengi,
                &binlik_ayır(y_değeri),
                nokta_imi,
            );
        }
    }
}

/// ECharts'ın `pin` sembolü: damla gövde + içinde değer etiketi.
fn raptiye_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    uç: (f32, f32),
    boyut: f32,
    renk: Renk,
    metin: &str,
    nokta_imi: &crate::model::imleyici::İmNoktası,
) {
    let yarıçap = boyut / 2.0;
    let merkez = (uç.0, uç.1 - yarıçap * 1.4);

    // Gövde: daire + uca inen üçgen kuyruk.
    yüzey.daire(merkez, yarıçap, Some(&Dolgu::Düz(renk)), None);
    let mut kuyruk = Yol::yeni();
    let açı = 0.45f32;
    kuyruk.taşı(uç);
    kuyruk.çiz((
        merkez.0 - yarıçap * açı.sin(),
        merkez.1 + yarıçap * açı.cos(),
    ));
    kuyruk.çiz((
        merkez.0 + yarıçap * açı.sin(),
        merkez.1 + yarıçap * açı.cos(),
    ));
    kuyruk.kapat();
    yüzey.yol_doldur(&kuyruk, &Dolgu::Düz(renk));

    if nokta_imi.etiket.göster {
        let boyut_yazı = nokta_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
        let yazı_rengi = nokta_imi.etiket.yazı.renk.unwrap_or(Renk::BEYAZ);
        yüzey.yazı(
            metin,
            merkez,
            YatayHiza::Orta,
            DikeyHiza::Orta,
            boyut_yazı,
            yazı_rengi,
            nokta_imi.etiket.yazı.kalın,
        );
    }
}
