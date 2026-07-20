//! Mum (candlestick) ve kutu (boxplot) serisi çizimleri —
//! `echarts/src/chart/candlestick` ve `chart/boxplot` karşılıkları.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{Yol, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::seri::{KutuSerisi, MumSerisi};
use crate::renk::{Dolgu, Renk};

/// zrender `subPixelOptimize(position, 1, positiveOrNegative)` karşılığı.
/// Mum gövdesi iki kenarı içe, fitili ise öntanımlı (negatif) yöne
/// yuvarlar; genel çizgi keskinleştirmesinden farklı olan bu ayrıntı ardışık
/// kategori merkezlerinde bir piksellik salınımı önler.
fn mum_alt_pikseli(konum: f32, pozitif: bool) -> f32 {
    let iki_kat = (konum * 2.0).round();
    if ((iki_kat as i64 + 1).rem_euclid(2)) == 0 {
        iki_kat / 2.0
    } else if pozitif {
        (iki_kat + 1.0) / 2.0
    } else {
        (iki_kat - 1.0) / 2.0
    }
}

/// Mum serisini çizer. Veri sırası: `[açılış, kapanış, en düşük, en yüksek]`.
pub fn mum_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &MumSerisi,
    genel_sıra: usize,
    kartezyen: &Kartezyen2B,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    if seri.büyük && seri.veri.len() >= seri.büyük_eşiği {
        mum_büyük_çiz(çizici, seri, kartezyen);
        return;
    }
    let bant = kartezyen.x.bant_genişliği();
    let gövde_genişliği = (bant * seri.gövde_oranı.clamp(0.05, 1.0)).max(1.0);
    let alan = kartezyen.alan;

    let gövde = |ç: &mut dyn ÇizimYüzeyi, isabetler: &mut Vec<İsabetBölgesi>| {
        for (i, öğe) in seri.veri.iter().enumerate() {
            if !kartezyen.x.pencerede_mi(i as f64) {
                continue;
            }
            let Some(dizi) = öğe.değer.dizi() else {
                continue;
            };
            let (Some(&açılış), Some(&kapanış), Some(&en_düşük), Some(&en_yüksek)) =
                (dizi.first(), dizi.get(1), dizi.get(2), dizi.get(3))
            else {
                continue;
            };
            let yükselen = kapanış >= açılış;
            let (dolgu_rengi, kenarlık_rengi) = if yükselen {
                (seri.yükselen_renk, seri.yükselen_kenarlık_rengi)
            } else {
                (seri.düşen_renk, seri.düşen_kenarlık_rengi)
            };

            let ham_x = kartezyen.x.veriden_piksele(i as f64);
            let x = mum_alt_pikseli(ham_x, false);
            let gövde_üst = kartezyen.y.veriden_piksele(açılış.max(kapanış));
            let gövde_alt = kartezyen.y.veriden_piksele(açılış.min(kapanış));
            let tepe = kartezyen.y.veriden_piksele(en_yüksek);
            let dip = kartezyen.y.veriden_piksele(en_düşük);

            // `candlestickLayout.isSimpleBox`: gövde 1,3 pikselden darsa
            // zrender yalnız yüksek–düşük çizgisini boyar.
            if gövde_genişliği <= 1.3 {
                let mut basit = Yol::yeni();
                basit.taşı((x, tepe));
                basit.çiz((x, dip));
                çizici_fitil(ç, &basit, seri.kenarlık_kalınlığı, kenarlık_rengi);
                isabetler.push(İsabetBölgesi {
                    seri_sırası: genel_sıra,
                    veri_sırası: i,
                    seri_adı: seri.ad.clone(),
                    ad: öğe.ad.clone(),
                    değer: Some(kapanış),
                    geometri: İsabetGeometrisi::Dikdörtgen(Dikdörtgen::yeni(
                        ham_x - 0.5,
                        tepe.min(dip),
                        1.0,
                        (dip - tepe).abs().max(1.0),
                    )),
                });
                continue;
            }

            // Fitiller (gövdenin üstünde ve altında).
            let mut fitil = Yol::yeni();
            fitil.taşı((x, tepe));
            fitil.çiz((x, gövde_üst));
            fitil.taşı((x, gövde_alt));
            fitil.çiz((x, dip));
            çizici_fitil(ç, &fitil, seri.kenarlık_kalınlığı, kenarlık_rengi);

            // Gövde.
            let sol = mum_alt_pikseli(ham_x - gövde_genişliği / 2.0, true);
            let sağ = mum_alt_pikseli(ham_x + gövde_genişliği / 2.0, false);
            let d = Dikdörtgen::yeni(
                sol,
                gövde_üst,
                (sağ - sol).max(1.0),
                (gövde_alt - gövde_üst).max(1.0),
            );
            ç.dikdörtgen(
                d,
                &Dolgu::Düz(dolgu_rengi),
                [0.0; 4],
                Some((seri.kenarlık_kalınlığı, kenarlık_rengi)),
            );

            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: i,
                seri_adı: seri.ad.clone(),
                ad: öğe.ad.clone(),
                değer: Some(kapanış),
                geometri: İsabetGeometrisi::Dikdörtgen(Dikdörtgen::yeni(
                    ham_x - gövde_genişliği / 2.0,
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

/// ECharts `LargeBoxPath`: her mum için gövde yerine yüksek–düşük arasında
/// tek çizgi üretir ve işarete göre iki toplu yolda boyar. Bu yol milyonlarca
/// ayrı zrender öğesi/isabet bölgesi üretmez.
fn mum_büyük_çiz(çizici: &mut dyn ÇizimYüzeyi, seri: &MumSerisi, kartezyen: &Kartezyen2B) {
    let mut yükselen = Yol {
        komutlar: Vec::with_capacity(seri.veri.len()),
    };
    let mut düşen = Yol {
        komutlar: Vec::with_capacity(seri.veri.len()),
    };
    let mut önceki_kapanış = None;

    for (sıra, öğe) in seri.veri.iter().enumerate() {
        let Some(dizi) = öğe.değer.dizi() else {
            continue;
        };
        let (Some(&açılış), Some(&kapanış), Some(&en_düşük), Some(&en_yüksek)) =
            (dizi.first(), dizi.get(1), dizi.get(2), dizi.get(3))
        else {
            continue;
        };
        let pozitif = if açılış < kapanış {
            true
        } else if açılış > kapanış {
            false
        } else {
            önceki_kapanış.is_none_or(|önceki| önceki <= kapanış)
        };
        önceki_kapanış = Some(kapanış);
        if !kartezyen.x.pencerede_mi(sıra as f64) {
            continue;
        }
        let x = kartezyen.x.veriden_piksele(sıra as f64);
        let y_düşük = kartezyen.y.veriden_piksele(en_düşük);
        let y_yüksek = kartezyen.y.veriden_piksele(en_yüksek);
        if !x.is_finite() || !y_düşük.is_finite() || !y_yüksek.is_finite() {
            continue;
        }
        let yol = if pozitif {
            &mut yükselen
        } else {
            &mut düşen
        };
        yol.taşı((x, y_düşük));
        yol.çiz((x, y_yüksek));
    }

    çizici_fitil(
        çizici,
        &yükselen,
        seri.kenarlık_kalınlığı,
        seri.yükselen_kenarlık_rengi,
    );
    çizici_fitil(
        çizici,
        &düşen,
        seri.kenarlık_kalınlığı,
        seri.düşen_kenarlık_rengi,
    );
}

fn çizici_fitil(ç: &mut dyn ÇizimYüzeyi, yol: &Yol, kalınlık: f32, renk: Renk) {
    ç.yol_çiz(yol, kalınlık, renk, crate::model::stil::ÇizgiTürü::Düz);
}

/// Kutu serisini çizer. Veri sırası:
/// `[en düşük, Ç1, ortanca, Ç3, en yüksek]`.
#[allow(clippy::too_many_arguments)]
pub fn kutu_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &KutuSerisi,
    genel_sıra: usize,
    kartezyen: &Kartezyen2B,
    grup_sırası: usize,
    grup_sayısı: usize,
    seri_rengi: Renk,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let yatay = kartezyen.y.ölçek.kategorik_mi() && !kartezyen.x.ölçek.kategorik_mi();
    let bant = if yatay {
        kartezyen.y.bant_genişliği()
    } else {
        kartezyen.x.bant_genişliği()
    };
    let grup_sayısı = grup_sayısı.max(1);
    let kullanılabilir_genişlik = (bant * 0.8 - 2.0).max(0.0);
    let kutu_boşluğu = kullanılabilir_genişlik / grup_sayısı as f32 * 0.3;
    let ham_kutu_genişliği = (kullanılabilir_genişlik
        - kutu_boşluğu * grup_sayısı.saturating_sub(1) as f32)
        / grup_sayısı as f32;
    // `boxplotLayout.calculateBase`: seri ofseti, genişlik alt/üst
    // sınırlarına uygulanmadan önceki ortak kutu genişliğiyle hesaplanır.
    let grup_ofseti = ham_kutu_genişliği / 2.0 - kullanılabilir_genişlik / 2.0
        + grup_sırası.min(grup_sayısı - 1) as f32 * (kutu_boşluğu + ham_kutu_genişliği);
    let gövde_genişliği = if seri.otomatik_gövde_genişliği {
        ham_kutu_genişliği.clamp(7.0, 50.0)
    } else {
        (bant * seri.gövde_oranı.clamp(0.05, 1.0)).max(1.0)
    };
    // ECharts `layEndLine`: bıyık kapağı gövdeyle aynı genişliktedir.
    let kapak_genişliği = gövde_genişliği;
    let renk = seri.öğe_stili.kenarlık_rengi.unwrap_or(seri_rengi);
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
        let Some(dizi) = öğe.değer.dizi() else {
            continue;
        };
        let (Some(&en_düşük), Some(&ç1), Some(&ortanca), Some(&ç3), Some(&en_yüksek)) = (
            dizi.first(),
            dizi.get(1),
            dizi.get(2),
            dizi.get(3),
            dizi.get(4),
        ) else {
            continue;
        };

        if yatay {
            // Boxplot yolu ECharts'ta `strokeNoScale` ile ham koordinatları
            // kullanır; eksen/çentik çizgilerindeki yarım-piksel keskinleme
            // burada uygulanmaz.
            let y = kartezyen.y.veriden_piksele(i as f64) + grup_ofseti;
            let x_düşük = kartezyen.x.veriden_piksele(en_düşük);
            let x_ç1 = kartezyen.x.veriden_piksele(ç1);
            let x_ortanca = kartezyen.x.veriden_piksele(ortanca);
            let x_ç3 = kartezyen.x.veriden_piksele(ç3);
            let x_yüksek = kartezyen.x.veriden_piksele(en_yüksek);

            // Bıyıklar ve kapaklar.
            let mut yol = Yol::yeni();
            yol.taşı((x_düşük, y));
            yol.çiz((x_ç1, y));
            yol.taşı((x_ç3, y));
            yol.çiz((x_yüksek, y));
            yol.taşı((x_düşük, y - kapak_genişliği / 2.0));
            yol.çiz((x_düşük, y + kapak_genişliği / 2.0));
            yol.taşı((x_yüksek, y - kapak_genişliği / 2.0));
            yol.çiz((x_yüksek, y + kapak_genişliği / 2.0));
            çizici.yol_çiz(&yol, kalınlık, renk, crate::model::stil::ÇizgiTürü::Düz);

            // Gövde: Ç1–Ç3 kutusu.
            let d = Dikdörtgen::yeni(
                x_ç1.min(x_ç3),
                y - gövde_genişliği / 2.0,
                (x_ç3 - x_ç1).abs().max(1.0),
                gövde_genişliği,
            );
            çizici.dikdörtgen(d, &dolgu, [0.0; 4], Some((kalınlık, renk)));

            // Ortanca çizgisi.
            çizici.çizgi(
                (x_ortanca, y - gövde_genişliği / 2.0),
                (x_ortanca, y + gövde_genişliği / 2.0),
                kalınlık,
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
                    x_düşük.min(x_yüksek),
                    y - gövde_genişliği / 2.0,
                    (x_yüksek - x_düşük).abs().max(1.0),
                    gövde_genişliği,
                )),
            });
            continue;
        }

        let x = kartezyen.x.veriden_piksele(i as f64) + grup_ofseti;
        let y_düşük = kartezyen.y.veriden_piksele(en_düşük);
        let y_ç1 = kartezyen.y.veriden_piksele(ç1);
        let y_ortanca = kartezyen.y.veriden_piksele(ortanca);
        let y_ç3 = kartezyen.y.veriden_piksele(ç3);
        let y_yüksek = kartezyen.y.veriden_piksele(en_yüksek);

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
            kalınlık,
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod testler {
    use super::*;
    use crate::cizim::gorunum::{BoyamaGirdisi, grafiği_boya};
    use crate::cizim::kayit::KayıtYüzeyi;
    use crate::model::eksen::Eksen;
    use crate::model::secenekler::GrafikSeçenekleri;

    #[test]
    fn buyuk_mumlar_oge_basina_sekil_yerine_iki_isaret_yolu_cizer() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .x_ekseni(Eksen::kategori().veri(["A", "B", "C"]))
            .y_ekseni(Eksen::değer())
            .seri(
                MumSerisi::yeni()
                    .büyük_eşiği(3)
                    .yükselen_kenarlık_rengi(0x123456)
                    .düşen_kenarlık_rengi(0x654321)
                    .veri([
                        [10.0, 12.0, 8.0, 14.0],
                        [12.0, 9.0, 7.0, 13.0],
                        [9.0, 11.0, 8.0, 12.0],
                    ]),
            );
        let mut yüzey = KayıtYüzeyi::yeni(300.0, 200.0);

        let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());
        let döküm = yüzey.döküm();

        assert!(döküm.contains("çiz #123456@1.0 k=1.0 düz |"));
        assert!(döküm.contains("çiz #654321@1.0 k=1.0 düz |"));
        assert!(!döküm.contains("dikdörtgen #eb5454"));
        assert!(çıktı.isabetler.is_empty());
    }
}
