//! İmleyici çizimi — `echarts/src/component/marker` görünümlerinin
//! karşılığı: im alanları serilerin altına, im çizgileri ve raptiyeler
//! serilerin üstüne boyanır.

use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, keskin, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::deger::VeriDeğeri;
use crate::model::imleyici::{
    İmAlanıDeğeri, İmDeğeri, İmYönü, İmleyiciler, İmÇizgisi, İmÇizgisiEtiketKonumu,
    İmÇizgisiEtiketYaması, İmÇizgisiUcu, İmÇizgisiUçSimgesi,
};
use crate::model::seri::Seri;
use crate::model::stil::{Etiket, YazıDikeyHizası, YazıYatayHizası};
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

fn seri_xy_kapsamı(seri: &Seri, kartezyen: &Kartezyen2B) -> Option<([f64; 2], [f64; 2])> {
    let mut x = [f64::INFINITY, f64::NEG_INFINITY];
    let mut y = [f64::INFINITY, f64::NEG_INFINITY];
    for (sıra, öğe) in seri.veri().iter().enumerate() {
        let nokta = match &öğe.değer {
            VeriDeğeri::Çift([x, y]) => Some((*x, *y)),
            VeriDeğeri::Dizi(değerler) if değerler.len() >= 2 => {
                Some((değerler[0], değerler[1]))
            }
            değer => değer.sayı().map(|değer| {
                if kartezyen.y.ölçek.kategorik_mi() && !kartezyen.x.ölçek.kategorik_mi() {
                    (değer, sıra as f64)
                } else {
                    (sıra as f64, değer)
                }
            }),
        };
        let Some((x_değeri, y_değeri)) = nokta else {
            continue;
        };
        if x_değeri.is_finite() {
            x[0] = x[0].min(x_değeri);
            x[1] = x[1].max(x_değeri);
        }
        if y_değeri.is_finite() {
            y[0] = y[0].min(y_değeri);
            y[1] = y[1].max(y_değeri);
        }
    }
    (x[0].is_finite() && x[1].is_finite() && y[0].is_finite() && y[1].is_finite()).then_some((x, y))
}

fn im_alanı_değeri_çöz(değer: İmAlanıDeğeri, kapsam: [f64; 2]) -> f64 {
    match değer {
        İmAlanıDeğeri::Değer(değer) => değer,
        İmAlanıDeğeri::VeriEnKüçük => kapsam[0],
        İmAlanıDeğeri::VeriEnBüyük => kapsam[1],
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
    // MarkLine öntanımlı `symbolSize: [8, 16]`, arrow şeklinin 8 px
    // genişlik ve 16 px yükseklik kutusudur. Çizgi doğrultusuna çevrilince
    // 16 px uzunluk, `Arrow.buildPath` gereği 2×(8×2/3) px taban verir.
    let yarı_taban = 8.0 * 2.0 / 3.0;
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

#[derive(Clone, Debug)]
struct ÇözülmüşİmÇizgisiEtiketi {
    etiket: Etiket,
    konum: İmÇizgisiEtiketKonumu,
    uzaklık: [f32; 2],
}

fn im_çizgisi_etiketini_çöz(
    im: &İmÇizgisi,
    yama: Option<&İmÇizgisiEtiketYaması>,
) -> ÇözülmüşİmÇizgisiEtiketi {
    let mut etiket = im.etiket.clone();
    let mut konum = im.etiket_konumu;
    let mut uzaklık = im.etiket_uzaklığı;
    if let Some(yama) = yama {
        if let Some(göster) = yama.göster {
            etiket.göster = göster;
        }
        if let Some(yama_konumu) = yama.konum {
            konum = yama_konumu;
        }
        if let Some(biçimleyici) = &yama.biçimleyici {
            etiket.biçimleyici = Some(biçimleyici.clone());
        }
        if let Some(yazı) = &yama.yazı {
            etiket.yazı = yazı.clone();
        }
        if let Some(yama_uzaklığı) = yama.uzaklık {
            uzaklık = yama_uzaklığı;
        }
        if let Some(hiza) = yama.yatay_hiza {
            etiket.yatay_hiza = Some(hiza);
        }
        if let Some(hiza) = yama.dikey_hiza {
            etiket.dikey_hiza = Some(hiza);
        }
    }
    ÇözülmüşİmÇizgisiEtiketi {
        etiket,
        konum,
        uzaklık,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct İmÇizgisiEtiketYerleşimi {
    çapa: (f32, f32),
    yatay: YatayHiza,
    dikey: DikeyHiza,
    /// Canvas koordinatında görünen dönüş; `None`, `start` / `end` gibi
    /// eksene paralel kalması gereken etiketleri belirtir.
    dönüş: Option<f32>,
}

/// `echarts/src/chart/helper/Line.ts#beforeUpdate` içindeki markLine etiket
/// geometrisinin düz çizgi karşılığı. İç konumlarda uzaklık önce metnin yerel
/// eksenlerinde uygulanır, ardından metin ve uzaklık birlikte çizgiye döner.
fn im_çizgisi_etiket_yerleşimi(
    başlangıç: (f32, f32),
    bitiş: (f32, f32),
    konum: İmÇizgisiEtiketKonumu,
    uzaklık: [f32; 2],
) -> Option<İmÇizgisiEtiketYerleşimi> {
    let dx = bitiş.0 - başlangıç.0;
    let dy = bitiş.1 - başlangıç.1;
    let uzunluk = dx.hypot(dy);
    if !uzunluk.is_finite() || uzunluk <= f32::EPSILON {
        return None;
    }
    let teğet = (dx / uzunluk, dy / uzunluk);

    let yerleşim = match konum {
        İmÇizgisiEtiketKonumu::Bitiş => İmÇizgisiEtiketYerleşimi {
            çapa: (
                bitiş.0 + teğet.0 * uzaklık[0],
                bitiş.1 + teğet.1 * uzaklık[1],
            ),
            yatay: if teğet.0 > 0.8 {
                YatayHiza::Sol
            } else if teğet.0 < -0.8 {
                YatayHiza::Sağ
            } else {
                YatayHiza::Orta
            },
            dikey: if teğet.1 > 0.8 {
                DikeyHiza::Üst
            } else if teğet.1 < -0.8 {
                DikeyHiza::Alt
            } else {
                DikeyHiza::Orta
            },
            dönüş: None,
        },
        İmÇizgisiEtiketKonumu::Başlangıç => İmÇizgisiEtiketYerleşimi {
            çapa: (
                başlangıç.0 - teğet.0 * uzaklık[0],
                başlangıç.1 - teğet.1 * uzaklık[1],
            ),
            yatay: if teğet.0 > 0.8 {
                YatayHiza::Sağ
            } else if teğet.0 < -0.8 {
                YatayHiza::Sol
            } else {
                YatayHiza::Orta
            },
            dikey: if teğet.1 > 0.8 {
                DikeyHiza::Alt
            } else if teğet.1 < -0.8 {
                DikeyHiza::Üst
            } else {
                DikeyHiza::Orta
            },
            dönüş: None,
        },
        _ => {
            let mut dönüş = teğet.1.atan2(teğet.0);
            if bitiş.0 < başlangıç.0 {
                dönüş += std::f32::consts::PI;
            }
            let yön = if teğet.0 < 0.0 { -1.0 } else { 1.0 };
            let (normal_uzaklığı, dikey) = match konum {
                İmÇizgisiEtiketKonumu::Orta
                | İmÇizgisiEtiketKonumu::İçBaşlangıçÜst
                | İmÇizgisiEtiketKonumu::İçOrtaÜst
                | İmÇizgisiEtiketKonumu::İçBitişÜst => (-uzaklık[1], DikeyHiza::Alt),
                İmÇizgisiEtiketKonumu::İçBaşlangıçAlt
                | İmÇizgisiEtiketKonumu::İçOrtaAlt
                | İmÇizgisiEtiketKonumu::İçBitişAlt => (uzaklık[1], DikeyHiza::Üst),
                _ => (0.0, DikeyHiza::Orta),
            };
            let (taban, doğrultu_uzaklığı, yatay) = match konum {
                İmÇizgisiEtiketKonumu::İçBaşlangıç
                | İmÇizgisiEtiketKonumu::İçBaşlangıçÜst
                | İmÇizgisiEtiketKonumu::İçBaşlangıçAlt => (
                    başlangıç,
                    uzaklık[0] * yön,
                    if teğet.0 < 0.0 {
                        YatayHiza::Sağ
                    } else {
                        YatayHiza::Sol
                    },
                ),
                İmÇizgisiEtiketKonumu::Orta
                | İmÇizgisiEtiketKonumu::İçOrta
                | İmÇizgisiEtiketKonumu::İçOrtaÜst
                | İmÇizgisiEtiketKonumu::İçOrtaAlt => (
                    ((başlangıç.0 + bitiş.0) / 2.0, (başlangıç.1 + bitiş.1) / 2.0),
                    0.0,
                    YatayHiza::Orta,
                ),
                İmÇizgisiEtiketKonumu::İçBitiş
                | İmÇizgisiEtiketKonumu::İçBitişÜst
                | İmÇizgisiEtiketKonumu::İçBitişAlt => (
                    bitiş,
                    -uzaklık[0] * yön,
                    if teğet.0 >= 0.0 {
                        YatayHiza::Sağ
                    } else {
                        YatayHiza::Sol
                    },
                ),
                İmÇizgisiEtiketKonumu::Başlangıç | İmÇizgisiEtiketKonumu::Bitiş => {
                    return None;
                }
            };
            let (sinüs, kosinüs) = dönüş.sin_cos();
            let yerel = (doğrultu_uzaklığı, normal_uzaklığı);
            İmÇizgisiEtiketYerleşimi {
                çapa: (
                    taban.0 + kosinüs * yerel.0 - sinüs * yerel.1,
                    taban.1 + sinüs * yerel.0 + kosinüs * yerel.1,
                ),
                yatay,
                dikey,
                dönüş: Some(dönüş),
            }
        }
    };
    Some(yerleşim)
}

fn yazı_yatay_hizasını_çöz(hiza: YazıYatayHizası) -> YatayHiza {
    match hiza {
        YazıYatayHizası::Sol => YatayHiza::Sol,
        YazıYatayHizası::Orta => YatayHiza::Orta,
        YazıYatayHizası::Sağ => YatayHiza::Sağ,
    }
}

fn yazı_dikey_hizasını_çöz(hiza: YazıDikeyHizası) -> DikeyHiza {
    match hiza {
        YazıDikeyHizası::Üst => DikeyHiza::Üst,
        YazıDikeyHizası::Orta => DikeyHiza::Orta,
        YazıDikeyHizası::Alt => DikeyHiza::Alt,
    }
}

fn im_çizgisi_etiketini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    başlangıç: (f32, f32),
    bitiş: (f32, f32),
    metin: &str,
    seçenek: &ÇözülmüşİmÇizgisiEtiketi,
) {
    if !seçenek.etiket.göster {
        return;
    }
    let Some(yerleşim) =
        im_çizgisi_etiket_yerleşimi(başlangıç, bitiş, seçenek.konum, seçenek.uzaklık)
    else {
        return;
    };
    let yatay = seçenek
        .etiket
        .yatay_hiza
        .map(yazı_yatay_hizasını_çöz)
        .unwrap_or(yerleşim.yatay);
    let dikey = seçenek
        .etiket
        .dikey_hiza
        .map(yazı_dikey_hizasını_çöz)
        .unwrap_or(yerleşim.dikey);
    let boyut = seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let renk = seçenek
        .etiket
        .yazı
        .renk
        .unwrap_or_else(tema::birincil_metin);
    // MarkerView, markLine etiketlerine açık zeminde 2 px beyaz
    // `textBorder` uygular. Dönüş sıfır olsa da aynı glif-vuruş yolu
    // kullanılmalı; sekiz bitmap kopyası Skia'nın `strokeFirst` sonucunu
    // özellikle küçük yazıda gereğinden fazla köşelendirir.
    let kontur_rengi = Renk::onaltılık(0xffffff);
    let matris = AfinMatris::ötele(yerleşim.çapa.0, yerleşim.çapa.1)
        .çarp(AfinMatris::döndür(yerleşim.dönüş.unwrap_or(0.0)));
    yüzey.dönüşümlü_konturlu_yazı(
        metin,
        (0.0, 0.0),
        yatay,
        dikey,
        boyut,
        renk,
        seçenek.etiket.yazı.kalın,
        kontur_rengi,
        2.0,
        matris,
    );
}

fn im_çizgisi_etiket_metni(
    seçenek: &ÇözülmüşİmÇizgisiEtiketi,
    değer: f64,
    ham: &str,
    varsayılan: &str,
    seri_adı: &str,
    veri_adı: &str,
) -> String {
    seçenek
        .etiket
        .biçimleyici
        .as_ref()
        .map(|biçimleyici| biçimleyici.uygula_bağlamla(değer, ham, seri_adı, veri_adı))
        .unwrap_or_else(|| varsayılan.to_owned())
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
    let dolgu = im_alanı_dolgusu(alan_imi, seri_rengi);
    let (x_kapsamı, y_kapsamı) =
        seri_xy_kapsamı(seri, kartezyen).unwrap_or(([0.0, 1.0], [0.0, 1.0]));

    for (ad, tanım) in &alan_imi.veri {
        let x0 = tanım
            .x0
            .map(|v| {
                kartezyen
                    .x
                    .veriden_piksele(im_alanı_değeri_çöz(v, x_kapsamı))
            })
            .unwrap_or(alan.x);
        let x1 = tanım
            .x1
            .map(|v| {
                kartezyen
                    .x
                    .veriden_piksele(im_alanı_değeri_çöz(v, x_kapsamı))
            })
            .unwrap_or(alan.sağ());
        let y0 = tanım
            .y0
            .map(|v| {
                kartezyen
                    .y
                    .veriden_piksele(im_alanı_değeri_çöz(v, y_kapsamı))
            })
            .unwrap_or(alan.y);
        let y1 = tanım
            .y1
            .map(|v| {
                kartezyen
                    .y
                    .veriden_piksele(im_alanı_değeri_çöz(v, y_kapsamı))
            })
            .unwrap_or(alan.alt());
        let d = Dikdörtgen::yeni(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
        yüzey.dikdörtgen(d, &dolgu, [0.0; 4], None);
        if alan_imi.stil.kenarlık_kalınlığı > 0.0 {
            let renk = alan_imi.stil.kenarlık_rengi.unwrap_or(seri_rengi);
            let mut kenarlık = Yol::yeni();
            // ECharts `markArea` çokgenini sol alttan başlatır. Özellikle kesikli
            // kenarlıklarda başlangıç köşesi çizgi fazını belirlediği için aynı
            // dolaşım sırasını korumak görsel eşleşmenin parçasıdır.
            kenarlık.taşı((d.x, d.alt()));
            kenarlık.çiz((d.sağ(), d.alt()));
            kenarlık.çiz((d.sağ(), d.y));
            kenarlık.çiz((d.x, d.y));
            kenarlık.kapat();
            yüzey.yol_çiz(
                &kenarlık,
                alan_imi.stil.kenarlık_kalınlığı,
                renk,
                alan_imi.stil.kenarlık_türü,
            );
        }

        if let Some(ad) = ad
            && (alan_imi.etiket.göster || !ad.is_empty())
        {
            let boyut = alan_imi.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            let renk = alan_imi
                .etiket
                .yazı
                .renk
                .unwrap_or_else(tema::birincil_metin);
            yüzey.dönüşümlü_konturlu_yazı(
                ad,
                (d.x + d.genişlik / 2.0, d.y - 5.5),
                YatayHiza::Orta,
                DikeyHiza::Alt,
                boyut,
                renk,
                alan_imi.etiket.yazı.kalın,
                Renk::onaltılık(0xffffff),
                2.0,
                AfinMatris::BİRİM,
            );
        }
    }
}

/// ECharts `MarkAreaView` otomatik dolgu rengini seri renginin %40
/// opaklığıyla kurar; `itemStyle.opacity` bu dolgunun tamamına ayrıca
/// uygulanır. Açık bir renk verilmişse yalnız genel opaklık çarpanı işler.
fn im_alanı_dolgusu(alan_imi: &crate::model::imleyici::İmAlanı, seri_rengi: Renk) -> Dolgu {
    alan_imi
        .stil
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(seri_rengi.opaklık(0.4)))
        .opaklık(alan_imi.stil.opaklık.unwrap_or(1.0).clamp(0.0, 1.0))
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
            let etiket = im_çizgisi_etiketini_çöz(çizgi_imi, parça.etiket.as_ref());
            let ham = ondalık_kırp(başlangıç_değeri);
            let metin = im_çizgisi_etiket_metni(
                &etiket,
                başlangıç_değeri,
                &ham,
                &ham,
                seri.ad().unwrap_or_default(),
                parça.ad.as_deref().unwrap_or_default(),
            );
            im_çizgisi_etiketini_çiz(yüzey, başlangıç, bitiş, &metin, &etiket);
        }
        for tanım in &çizgi_imi.veri {
            let Some(değer) = değer_çöz(tanım.değer, seri) else {
                continue;
            };
            let biçimli_değer = binlik_ayır(yuvarla(değer, 2));
            let etiket = im_çizgisi_etiketini_çöz(çizgi_imi, tanım.etiket.as_ref());
            let etiket_metni = im_çizgisi_etiket_metni(
                &etiket,
                değer,
                &biçimli_değer,
                &biçimli_değer,
                seri.ad().unwrap_or_default(),
                tanım.ad.as_deref().unwrap_or_default(),
            );
            match tanım.yön {
                İmYönü::Yatay => {
                    let y = kartezyen.y.veriden_piksele(değer);
                    let çizgi_y = keskin(y);
                    let başlangıç = (alan.x, çizgi_y);
                    let bitiş = (alan.sağ(), çizgi_y);
                    let im_başlangıcı = (alan.x, y);
                    let im_bitişi = (alan.sağ(), y);
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
                        im_başlangıcı,
                        im_bitişi,
                        uç_rengi,
                    );
                    im_uç_simgesini_çiz(
                        yüzey,
                        çizgi_imi.bitiş_simgesi,
                        im_bitişi,
                        im_başlangıcı,
                        uç_rengi,
                    );
                    im_çizgisi_etiketini_çiz(
                        yüzey,
                        im_başlangıcı,
                        im_bitişi,
                        &etiket_metni,
                        &etiket,
                    );
                }
                İmYönü::Dikey => {
                    let x = kartezyen.x.veriden_piksele(değer);
                    let çizgi_x = keskin(x);
                    let başlangıç = (çizgi_x, alan.alt());
                    let bitiş = (çizgi_x, alan.y);
                    let im_başlangıcı = (x, alan.alt());
                    let im_bitişi = (x, alan.y);
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
                        im_başlangıcı,
                        im_bitişi,
                        uç_rengi,
                    );
                    im_uç_simgesini_çiz(
                        yüzey,
                        çizgi_imi.bitiş_simgesi,
                        im_bitişi,
                        im_başlangıcı,
                        uç_rengi,
                    );
                    im_çizgisi_etiketini_çiz(
                        yüzey,
                        im_başlangıcı,
                        im_bitişi,
                        &etiket_metni,
                        &etiket,
                    );
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
        // Symbol iç etiketi, `pin` yolunun geometrik merkezine değil
        // zrender `calculateTextPosition` sonucuna yerleşir. 50 px resmi
        // raptiyede bu çapa sivri uçtan 26.142857 px yukarıdadır ve boyutla
        // doğrusal ölçeklenir. Marker varsayılanı ayrıca 2 px seri rengi
        // `textBorder` uygular.
        let yazı_merkezi = (uç.0, uç.1 - boyut * 0.522_857_1);
        yüzey.dönüşümlü_konturlu_yazı(
            metin,
            yazı_merkezi,
            YatayHiza::Orta,
            DikeyHiza::Orta,
            boyut_yazı,
            yazı_rengi,
            nokta_imi.etiket.yazı.kalın,
            renk,
            2.0,
            AfinMatris::BİRİM,
        );
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::koordinat::ÇalışmaEkseni;
    use crate::model::eksen::{Eksen, EksenKonumu};
    use crate::model::imleyici::{İmAlanı, İmÇizgisiTanımı};
    use crate::model::seri::SaçılımSerisi;
    use crate::model::stil::{YazıStili, ÇizgiTürü, ÖğeStili};
    use crate::olcek::{AralıkÖlçeği, Ölçek};

    fn yakın(sol: f32, sağ: f32) {
        assert!((sol - sağ).abs() < 1e-3, "{sol} != {sağ}");
    }

    fn değer_ekseni(kapsam: [f64; 2], piksel: [f32; 2], konum: EksenKonumu) -> ÇalışmaEkseni {
        ÇalışmaEkseni::yeni(
            Eksen::değer(),
            Ölçek::Aralık(AralıkÖlçeği::kur(
                kapsam,
                Some(kapsam[0]),
                Some(kapsam[1]),
                false,
                5,
                None,
                None,
            )),
            piksel,
            konum,
        )
    }

    fn test_kartezyeni() -> Kartezyen2B {
        Kartezyen2B {
            x: değer_ekseni([0.0, 10.0], [0.0, 100.0], EksenKonumu::Alt),
            y: değer_ekseni([0.0, 100.0], [100.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        }
    }

    #[test]
    fn markarea_otomatik_rengi_ve_itemstyle_opakligini_birlestirir() {
        let seri_rengi = Renk::onaltılık(0x5470c6);
        let otomatik = İmAlanı::yeni().stil(ÖğeStili::yeni().opaklık(0.3));
        let Dolgu::Düz(otomatik_renk) = im_alanı_dolgusu(&otomatik, seri_rengi) else {
            panic!("otomatik markArea dolgusu düz renk olmalı");
        };
        yakın(otomatik_renk.alfa, 0.12);

        let açık = İmAlanı::yeni().stil(ÖğeStili::yeni().renk("rgba(1, 2, 3, 0.5)").opaklık(0.2));
        let Dolgu::Düz(açık_renk) = im_alanı_dolgusu(&açık, seri_rengi) else {
            panic!("açık markArea dolgusu düz renk olmalı");
        };
        yakın(açık_renk.alfa, 0.1);
    }

    #[test]
    fn markarea_veri_min_max_kapsamini_ve_resmi_kesik_fazini_kullanir() {
        let seri = Seri::from(SaçılımSerisi::yeni().veri([[2.0, 30.0], [8.0, 70.0]]));
        let imleyiciler = İmleyiciler {
            alan: Some(
                İmAlanı::yeni().veri_kapsamı("Kapsam").stil(
                    ÖğeStili::yeni()
                        .kenarlık_kalınlığı(1.0)
                        .kenarlık_türü(ÇizgiTürü::Kesikli),
                ),
            ),
            ..Default::default()
        };
        let mut yüzey = KayıtYüzeyi::yeni(100.0, 100.0);

        im_alanlarını_çiz(
            &mut yüzey,
            &imleyiciler,
            &seri,
            &test_kartezyeni(),
            Renk::onaltılık(0x5470c6),
        );

        let döküm = yüzey.döküm();
        assert!(
            döküm.contains("dikdörtgen (20.0,30.0 60.0x40.0)"),
            "veri kapsamı çözülmedi: {döküm}"
        );
        assert!(
            döküm.contains("kesikli | T(20.0,70.0) Ç(80.0,70.0) Ç(80.0,30.0) Ç(20.0,30.0) Z"),
            "kesik çizgi ECharts'ın sol-alt başlangıç fazını izlemiyor: {döküm}"
        );
    }

    #[test]
    fn markline_adlandirilmis_istatistikte_varsayilan_olarak_yalniz_degeri_yazar() {
        let seri = Seri::from(
            SaçılımSerisi::yeni()
                .ad("Kadın")
                .veri([[2.0, 40.0], [8.0, 60.0]]),
        );
        let imleyiciler = İmleyiciler {
            çizgi: Some(
                İmÇizgisi::yeni()
                    .tanım(İmÇizgisiTanımı::yeni(İmYönü::Yatay, İmDeğeri::Ortalama).ad("AVG")),
            ),
            ..Default::default()
        };
        let mut yüzey = KayıtYüzeyi::yeni(100.0, 100.0);

        im_çizgi_ve_noktalarını_çiz(
            &mut yüzey,
            &imleyiciler,
            &seri,
            &test_kartezyeni(),
            Renk::onaltılık(0x5470c6),
            0.0,
        );

        let döküm = yüzey.döküm();
        assert!(döküm.contains("yazı \"50\""), "{döküm}");
        assert!(
            !döküm.contains("AVG:"),
            "ad yalnız {{b}} ile yazılmalı: {döküm}"
        );
    }

    #[test]
    fn markline_yatay_etiket_konumları_line_ts_geometrisini_izler() {
        let başlangıç = (0.0, 100.0);
        let bitiş = (100.0, 100.0);
        let uzaklık = [20.0, 8.0];

        let baş = im_çizgisi_etiket_yerleşimi(
            başlangıç,
            bitiş,
            İmÇizgisiEtiketKonumu::Başlangıç,
            uzaklık,
        )
        .expect("başlangıç etiketi");
        assert_eq!(baş.çapa, (-20.0, 100.0));
        assert_eq!(
            (baş.yatay, baş.dikey, baş.dönüş),
            (YatayHiza::Sağ, DikeyHiza::Orta, None)
        );

        let orta =
            im_çizgisi_etiket_yerleşimi(başlangıç, bitiş, İmÇizgisiEtiketKonumu::Orta, uzaklık)
                .expect("orta etiketi");
        assert_eq!(orta.çapa, (50.0, 92.0));
        assert_eq!((orta.yatay, orta.dikey), (YatayHiza::Orta, DikeyHiza::Alt));

        let iç_baş_üst = im_çizgisi_etiket_yerleşimi(
            başlangıç,
            bitiş,
            İmÇizgisiEtiketKonumu::İçBaşlangıçÜst,
            uzaklık,
        )
        .expect("iç başlangıç üst etiketi");
        assert_eq!(iç_baş_üst.çapa, (20.0, 92.0));
        assert_eq!(
            (iç_baş_üst.yatay, iç_baş_üst.dikey),
            (YatayHiza::Sol, DikeyHiza::Alt)
        );

        let iç_bitiş_alt = im_çizgisi_etiket_yerleşimi(
            başlangıç,
            bitiş,
            İmÇizgisiEtiketKonumu::İçBitişAlt,
            uzaklık,
        )
        .expect("iç bitiş alt etiketi");
        assert_eq!(iç_bitiş_alt.çapa, (80.0, 108.0));
        assert_eq!(
            (iç_bitiş_alt.yatay, iç_bitiş_alt.dikey),
            (YatayHiza::Sağ, DikeyHiza::Üst)
        );
    }

    #[test]
    fn markline_eğik_etiket_matrisi_resmi_ornekle_ayni() {
        let başlangıç = (118.0, 416.75);
        let bitiş = (466.0, 257.5);
        let yerleşim = im_çizgisi_etiket_yerleşimi(
            başlangıç,
            bitiş,
            İmÇizgisiEtiketKonumu::İçBaşlangıçÜst,
            [20.0, 8.0],
        )
        .expect("eğik etiket");
        yakın(yerleşim.çapa.0, 132.85732);
        yakın(yerleşim.çapa.1, 401.1532);
        let dönüş = yerleşim.dönüş.expect("eğik etiket dönüşü");
        yakın(dönüş.cos(), 0.9093121);
        yakın(dönüş.sin(), -0.4161148);

        let bitiş_etiketi = im_çizgisi_etiket_yerleşimi(
            başlangıç,
            bitiş,
            İmÇizgisiEtiketKonumu::Bitiş,
            [20.0, 8.0],
        )
        .expect("bitiş etiketi");
        yakın(bitiş_etiketi.çapa.0, 484.18625);
        yakın(bitiş_etiketi.çapa.1, 254.17108);
    }

    #[test]
    fn markline_oge_etiketi_genel_etiketten_miras_alir() {
        let im = İmÇizgisi::yeni()
            .etiket(
                Etiket::yeni()
                    .göster(true)
                    .biçimleyici("{b}")
                    .yazı(YazıStili::yeni().boyut(14.0)),
            )
            .etiket_uzaklığı(20.0, 8.0);
        let yama = İmÇizgisiEtiketYaması::yeni()
            .konum(İmÇizgisiEtiketKonumu::İçOrtaAlt)
            .biçimleyici("öğe");
        let çözülmüş = im_çizgisi_etiketini_çöz(&im, Some(&yama));

        assert!(çözülmüş.etiket.göster);
        assert_eq!(çözülmüş.etiket.yazı.boyut, Some(14.0));
        assert_eq!(çözülmüş.konum, İmÇizgisiEtiketKonumu::İçOrtaAlt);
        assert_eq!(çözülmüş.uzaklık, [20.0, 8.0]);
        assert_eq!(
            im_çizgisi_etiket_metni(&çözülmüş, 1.0, "1", "1", "line", "ad"),
            "öğe"
        );

        let negatif = İmÇizgisi::yeni().etiket_uzaklığı(-4.0, -2.0);
        assert_eq!(negatif.etiket_uzaklığı, [-4.0, -2.0]);
        assert_eq!(
            İmÇizgisiEtiketYaması::yeni().uzaklık(-3.0, -1.0).uzaklık,
            Some([-3.0, -1.0])
        );
    }
}
