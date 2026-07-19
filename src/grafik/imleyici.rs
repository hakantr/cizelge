//! İmleyici çizimi — `echarts/src/component/marker` görünümlerinin
//! karşılığı: im alanları serilerin altına, im çizgileri ve raptiyeler
//! serilerin üstüne boyanır.

use crate::cizim::{DikeyHiza, YatayHiza, Yol, keskin, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::imleyici::{
    İmDeğeri, İmYönü, İmleyiciler, İmÇizgisiUcu, İmÇizgisiUçSimgesi
};
use crate::model::seri::Seri;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::{binlik_ayır, ondalık_kırp};
use crate::yardimci::sayi::yuvarla;

fn otomatik_iç_yazı_rengi(ana_renk: Renk) -> Renk {
    let parlaklık =
        (0.299 * ana_renk.kırmızı + 0.587 * ana_renk.yeşil + 0.114 * ana_renk.mavi) * ana_renk.alfa;
    if parlaklık > 0.5 {
        Renk::onaltılık(0x333333)
    } else if parlaklık > 0.2 {
        Renk::onaltılık(0xeeeeee)
    } else {
        Renk::onaltılık(0xcccccc)
    }
}

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
            let sayaç = seri
                .veri()
                .iter()
                .filter(|ö| ö.değer.sayı().is_some())
                .count();
            (sayaç > 0).then_some(sayaç / 2)
        }
        _ => seri
            .veri()
            .iter()
            .position(|ö| ö.değer.sayı().map(|d| d == hedef).unwrap_or(false)),
    }
}

fn çizgi_ucu_çöz(
    uç: İmÇizgisiUcu,
    seri: &Seri,
    kartezyen: &Kartezyen2B,
    kategori_kaydırması: f32,
) -> Option<((f32, f32), f64)> {
    match uç {
        İmÇizgisiUcu::Koordinat(x, y) => {
            let mut nokta = kartezyen.nokta(x, y);
            if kartezyen.x.ölçek.kategorik_mi() {
                nokta.0 += kategori_kaydırması;
            } else if kartezyen.y.ölçek.kategorik_mi() {
                nokta.1 += kategori_kaydırması;
            }
            Some((nokta, y))
        }
        İmÇizgisiUcu::İstatistik(istatistik) => {
            let sıra = değer_sırası(istatistik, seri)?;
            let değer = değer_çöz(istatistik, seri)?;
            let nokta = if kartezyen.x.ölçek.kategorik_mi() {
                let mut nokta = kartezyen.nokta(sıra as f64, değer);
                nokta.0 += kategori_kaydırması;
                nokta
            } else if kartezyen.y.ölçek.kategorik_mi() {
                let mut nokta = kartezyen.nokta(değer, sıra as f64);
                nokta.1 += kategori_kaydırması;
                nokta
            } else {
                let öğe = seri.veri().get(sıra)?;
                kartezyen.nokta(öğe.değer.x().unwrap_or(sıra as f64), değer)
            };
            Some((nokta, değer))
        }
    }
}

fn im_okunu_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    başlangıç: (f32, f32),
    bitiş: (f32, f32),
    renk: Renk,
) {
    let dx = bitiş.0 - başlangıç.0;
    let dy = bitiş.1 - başlangıç.1;
    let uzunluk = dx.hypot(dy);
    if uzunluk <= f32::EPSILON {
        return;
    }
    let ux = dx / uzunluk;
    let uy = dy / uzunluk;
    let nx = -uy;
    let ny = ux;
    let mut ok = Yol::yeni();
    ok.taşı(bitiş);
    let yarı_taban = 16.0 * 2.0 / 3.0;
    let taban_orta = (bitiş.0 - ux * 16.0, bitiş.1 - uy * 16.0);
    ok.çiz((
        taban_orta.0 + nx * yarı_taban,
        taban_orta.1 + ny * yarı_taban,
    ));
    // `echarts/src/util/symbol.ts` Arrow: tabanın dörtte üçündeki iç
    // çentik, varsayılan ok simgesini dolu üçgenden ayırır.
    ok.çiz((bitiş.0 - ux * 12.0, bitiş.1 - uy * 12.0));
    ok.çiz((
        taban_orta.0 - nx * yarı_taban,
        taban_orta.1 - ny * yarı_taban,
    ));
    ok.kapat();
    yüzey.yol_doldur(&ok, &Dolgu::Düz(renk));
}

fn im_uç_simgesini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    simge: İmÇizgisiUçSimgesi,
    uç: (f32, f32),
    karşı_uç: (f32, f32),
    renk: Renk,
) {
    match simge {
        İmÇizgisiUçSimgesi::Yok => {}
        İmÇizgisiUçSimgesi::Daire => {
            yüzey.daire(uç, 4.0, Some(&Dolgu::Düz(renk)), None);
        }
        İmÇizgisiUçSimgesi::Ok => im_okunu_çiz(yüzey, karşı_uç, uç, renk),
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
    let Some(alan_imi) = &imleyiciler.alan else {
        return;
    };
    let alan = kartezyen.alan;
    let dolgu = alan_imi
        .stil
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(seri_rengi.opaklık(0.15)));
    let _ = seri;

    for (ad, tanım) in &alan_imi.veri {
        let x0 = tanım
            .x0
            .map(|v| kartezyen.x.veriden_piksele(v))
            .unwrap_or(alan.x);
        let x1 = tanım
            .x1
            .map(|v| kartezyen.x.veriden_piksele(v))
            .unwrap_or(alan.sağ());
        let y0 = tanım
            .y0
            .map(|v| kartezyen.y.veriden_piksele(v))
            .unwrap_or(alan.y);
        let y1 = tanım
            .y1
            .map(|v| kartezyen.y.veriden_piksele(v))
            .unwrap_or(alan.alt());
        let d = Dikdörtgen::yeni(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
        yüzey.dikdörtgen(d, &dolgu, [0.0; 4], None);

        if let Some(ad) = ad
            && (alan_imi.etiket.göster || !ad.is_empty())
        {
            let boyut = alan_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            let renk = alan_imi
                .etiket
                .yazı
                .renk
                .unwrap_or_else(tema::birincil_metin);
            yüzey.yazı(
                ad,
                (d.x + d.genişlik / 2.0, d.y - 4.0),
                YatayHiza::Orta,
                DikeyHiza::Alt,
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
    kategori_kaydırması: f32,
) {
    let alan = kartezyen.alan;

    // 1) İm çizgileri.
    if let Some(çizgi_imi) = &imleyiciler.çizgi {
        let renk = çizgi_imi.stil.renk.unwrap_or(seri_rengi);
        for parça in &çizgi_imi.parçalar {
            let Some((başlangıç, başlangıç_değeri)) =
                çizgi_ucu_çöz(parça.başlangıç, seri, kartezyen, kategori_kaydırması)
            else {
                continue;
            };
            let Some((bitiş, _)) =
                çizgi_ucu_çöz(parça.bitiş, seri, kartezyen, kategori_kaydırması)
            else {
                continue;
            };
            let renk = renk.opaklık(çizgi_imi.stil.opaklık);
            yüzey.çizgi(
                başlangıç,
                bitiş,
                çizgi_imi.stil.kalınlık,
                renk,
                çizgi_imi.stil.tür,
            );
            im_uç_simgesini_çiz(yüzey, parça.başlangıç_simgesi, başlangıç, bitiş, renk);
            im_uç_simgesini_çiz(yüzey, parça.bitiş_simgesi, bitiş, başlangıç, renk);
            if çizgi_imi.etiket.göster {
                let boyut = çizgi_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                let metin = parça
                    .ad
                    .clone()
                    .unwrap_or_else(|| ondalık_kırp(başlangıç_değeri));
                let dx = bitiş.0 - başlangıç.0;
                let dy = bitiş.1 - başlangıç.1;
                let uzunluk = dx.hypot(dy).max(f32::EPSILON);
                yüzey.yazı(
                    &metin,
                    (bitiş.0 + dx / uzunluk * 5.0, bitiş.1 + dy / uzunluk * 5.0),
                    YatayHiza::Sol,
                    DikeyHiza::Orta,
                    boyut,
                    çizgi_imi.etiket.yazı.renk.unwrap_or(tema::birincil_metin()),
                    çizgi_imi.etiket.yazı.kalın,
                );
            }
        }
        for tanım in &çizgi_imi.veri {
            let Some(değer) = değer_çöz(tanım.değer, seri) else {
                continue;
            };
            let biçimli_değer = binlik_ayır(yuvarla(değer, 2));
            let etiket_metni = tanım
                .ad
                .clone()
                .map(|ad| format!("{ad}: {biçimli_değer}"))
                .unwrap_or(biçimli_değer);
            match tanım.yön {
                İmYönü::Yatay => {
                    let y = keskin(kartezyen.y.veriden_piksele(değer));
                    let başlangıç = (alan.x, y);
                    let bitiş = (alan.sağ(), y);
                    yüzey.çizgi(
                        başlangıç,
                        bitiş,
                        çizgi_imi.stil.kalınlık,
                        renk.opaklık(çizgi_imi.stil.opaklık),
                        çizgi_imi.stil.tür,
                    );
                    let uç_rengi = renk.opaklık(çizgi_imi.stil.opaklık);
                    im_uç_simgesini_çiz(
                        yüzey,
                        çizgi_imi.başlangıç_simgesi,
                        başlangıç,
                        bitiş,
                        uç_rengi,
                    );
                    im_uç_simgesini_çiz(
                        yüzey,
                        çizgi_imi.bitiş_simgesi,
                        bitiş,
                        başlangıç,
                        uç_rengi,
                    );
                    if çizgi_imi.etiket.göster {
                        let boyut = çizgi_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                        yüzey.yazı(
                            &etiket_metni,
                            (alan.sağ() + 5.0, y),
                            YatayHiza::Sol,
                            DikeyHiza::Orta,
                            boyut,
                            çizgi_imi
                                .etiket
                                .yazı
                                .renk
                                .unwrap_or_else(tema::birincil_metin),
                            false,
                        );
                    }
                }
                İmYönü::Dikey => {
                    let x = keskin(kartezyen.x.veriden_piksele(değer));
                    let başlangıç = (x, alan.alt());
                    let bitiş = (x, alan.y);
                    yüzey.çizgi(
                        başlangıç,
                        bitiş,
                        çizgi_imi.stil.kalınlık,
                        renk.opaklık(çizgi_imi.stil.opaklık),
                        çizgi_imi.stil.tür,
                    );
                    let uç_rengi = renk.opaklık(çizgi_imi.stil.opaklık);
                    im_uç_simgesini_çiz(
                        yüzey,
                        çizgi_imi.başlangıç_simgesi,
                        başlangıç,
                        bitiş,
                        uç_rengi,
                    );
                    im_uç_simgesini_çiz(
                        yüzey,
                        çizgi_imi.bitiş_simgesi,
                        bitiş,
                        başlangıç,
                        uç_rengi,
                    );
                    if çizgi_imi.etiket.göster {
                        let boyut = çizgi_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                        yüzey.yazı(
                            &etiket_metni,
                            (x, alan.y - 4.0),
                            YatayHiza::Orta,
                            DikeyHiza::Alt,
                            boyut,
                            çizgi_imi
                                .etiket
                                .yazı
                                .renk
                                .unwrap_or_else(tema::birincil_metin),
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
            let (x_değeri, y_değeri, etiket_değeri) = match (tanım.koordinat, tanım.değer) {
                (Some((x, y)), değer) => (
                    x,
                    y,
                    değer.and_then(|değer| değer_çöz(değer, seri)).unwrap_or(y),
                ),
                (None, Some(değer)) => {
                    let Some(sıra) = değer_sırası(değer, seri) else {
                        continue;
                    };
                    let Some(y) = değer_çöz(değer, seri) else {
                        continue;
                    };
                    let x = seri
                        .veri()
                        .get(sıra)
                        .and_then(|ö| ö.değer.x())
                        .unwrap_or(sıra as f64);
                    (x, y, y)
                }
                (None, None) => continue,
            };
            let (mut x, mut y) = kartezyen.nokta(x_değeri, y_değeri);
            // MarkPoint, sütun serisinin kategori koordinatını hem
            // istatistik hem açık `xAxis`/`yAxis` tanımlarında serinin kendi
            // bar yerleşimine taşır.
            if kartezyen.x.ölçek.kategorik_mi() {
                x += kategori_kaydırması;
            } else if kartezyen.y.ölçek.kategorik_mi() {
                y += kategori_kaydırması;
            }
            raptiye_çiz(
                yüzey,
                (x, y),
                nokta_imi.boyut,
                seri_rengi,
                &binlik_ayır(etiket_değeri),
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
    // `echarts/src/util/symbol.ts` içindeki Pin yolunun doğrudan portu.
    // `symbolSize`, dış yerleşim kutusudur; gerçek yuvarlak başın çapı bu
    // kutunun 3/5'idir ve damla ucu tam veri koordinatında kalır.
    let genişlik = boyut / 5.0 * 3.0;
    let yükseklik = genişlik.max(boyut);
    let yarıçap = genişlik / 2.0;
    let dy = yarıçap * yarıçap / (yükseklik - yarıçap).max(f32::EPSILON);
    let merkez = (uç.0, uç.1 - yükseklik + yarıçap + dy);
    let açı = (dy / yarıçap.max(f32::EPSILON)).clamp(-1.0, 1.0).asin();
    let dx = açı.cos() * yarıçap;
    let teğet_x = açı.sin();
    let teğet_y = açı.cos();
    let kontrol = yarıçap * 0.6;
    let uç_kontrolü = yarıçap * 0.7;

    let sol_teğet = (uç.0 - dx, merkez.1 + dy);
    let sağ_teğet = (uç.0 + dx, merkez.1 + dy);
    let mut gövde = Yol::yeni();
    gövde.taşı(sol_teğet);
    gövde.yay(yarıçap, true, true, sağ_teğet);
    gövde.kübik(
        (
            uç.0 + dx - teğet_x * kontrol,
            merkez.1 + dy + teğet_y * kontrol,
        ),
        (uç.0, uç.1 - uç_kontrolü),
        uç,
    );
    gövde.kübik(
        (uç.0, uç.1 - uç_kontrolü),
        (
            uç.0 - dx + teğet_x * kontrol,
            merkez.1 + dy + teğet_y * kontrol,
        ),
        sol_teğet,
    );
    gövde.kapat();
    yüzey.yol_doldur(&gövde, &Dolgu::Düz(renk));

    if nokta_imi.etiket.göster {
        let boyut_yazı = nokta_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
        let yazı_rengi = nokta_imi
            .etiket
            .yazı
            .renk
            .unwrap_or_else(|| otomatik_iç_yazı_rengi(renk));
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
