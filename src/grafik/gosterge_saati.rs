//! Gösterge saati (gauge) serisi — `echarts/src/chart/gauge` karşılığı:
//! renk bantlı yay, çentikler, etiketler, ibre ve değer yazısı.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::seri::GöstergeSaatiSerisi;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yardimci::sayi::doğrusal_eşle;

/// ECharts gauge açısı (derece, saat yönünün tersine artan matematik
/// açısı) → ekran radyanı (y aşağı).
fn ekran_açısı(derece: f64) -> f32 {
    (-derece.to_radians()) as f32
}

/// Gösterge saatini çizer.
pub fn gösterge_saati_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &GöstergeSaatiSerisi,
    genel_sıra: usize,
    seri_rengi: Renk,
    tuval: Dikdörtgen,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let merkez = (
        tuval.x + seri.merkez.0.çöz(tuval.genişlik),
        tuval.y + seri.merkez.1.çöz(tuval.yükseklik),
    );
    let taban_yarıçap = tuval.genişlik.min(tuval.yükseklik) / 2.0;
    let yarıçap = seri.yarıçap.çöz(taban_yarıçap);
    if yarıçap <= 0.0 {
        return;
    }
    let şerit = seri.şerit_kalınlığı.max(0.0);
    let kapsam = [seri.en_az, seri.en_çok.max(seri.en_az + 1e-9)];

    let değer_oranı = |değer: f64| doğrusal_eşle(değer, kapsam, [0.0, 1.0], true);
    let orandan_açı = |oran: f64| {
        seri.başlangıç_açısı as f64 + (seri.bitiş_açısı as f64 - seri.başlangıç_açısı as f64) * oran
    };

    // 1) Renk bantlı yay (axisLine). Boş renk dizisi, ECharts 6.1'in
    // `tokens.color.neutral10` öntanımlısıdır; modelde somutlaştırılmaz ki
    // koyu tema aynı option nesnesiyle doğru belirteci çözebilsin.
    if seri.şeridi_göster && şerit > 0.0 {
        if seri.renk_bantları.is_empty() {
            çizici.dilim(
                merkez,
                yarıçap - şerit,
                yarıçap,
                ekran_açısı(orandan_açı(0.0)),
                ekran_açısı(orandan_açı(1.0)),
                &Dolgu::Düz(tema::nötr_10()),
                None,
            );
        } else {
            let mut önceki_oran = 0.0f64;
            for (bant_sonu, renk) in &seri.renk_bantları {
                let son = (*bant_sonu as f64).clamp(önceki_oran, 1.0);
                if son > önceki_oran {
                    çizici.dilim(
                        merkez,
                        yarıçap - şerit,
                        yarıçap,
                        ekran_açısı(orandan_açı(önceki_oran)),
                        ekran_açısı(orandan_açı(son)),
                        &Dolgu::Düz(*renk),
                        None,
                    );
                }
                önceki_oran = son;
            }
            if önceki_oran < 1.0 {
                çizici.dilim(
                    merkez,
                    yarıçap - şerit,
                    yarıçap,
                    ekran_açısı(orandan_açı(önceki_oran)),
                    ekran_açısı(orandan_açı(1.0)),
                    &Dolgu::Düz(tema::nötr_10()),
                    None,
                );
            }
        }
    }

    // 2) Ana bölme çizgileri, etiketler ve her bölmedeki ara çentikler.
    // Uzaklıklar ECharts GaugeView ile aynı biçimde yay kalınlığının iç
    // kenarından ölçülür.
    let bölme = seri.bölme_sayısı.max(1);
    for i in 0..=bölme {
        let oran = i as f64 / bölme as f64;
        let açı = ekran_açısı(orandan_açı(oran));
        let (kos, sin) = (açı.cos(), açı.sin());
        let ana_dış = yarıçap - şerit - seri.çentik_mesafesi;
        let ana_iç = ana_dış - seri.çentik_uzunluğu.max(0.0);
        if seri.çentikleri_göster && seri.çentik_kalınlığı > 0.0 {
            çizici.çizgi(
                (merkez.0 + ana_dış * kos, merkez.1 + ana_dış * sin),
                (merkez.0 + ana_iç * kos, merkez.1 + ana_iç * sin),
                seri.çentik_kalınlığı,
                seri.çentik_rengi.unwrap_or_else(tema::eksen_çentiği),
                crate::model::stil::ÇizgiTürü::Düz,
            );
        }
        if seri.etiketleri_göster {
            let değer = doğrusal_eşle(oran, [0.0, 1.0], kapsam, true);
            let ham = binlik_ayır(değer);
            let metin = seri
                .etiket_biçimleyici
                .as_ref()
                .map_or_else(|| ham.clone(), |biçim| biçim.uygula(değer, &ham));
            let konum_yarıçapı = yarıçap
                - seri.çentik_uzunluğu.max(0.0)
                - seri.çentik_mesafesi
                - seri.etiket_mesafesi;
            let yatay = if kos < -0.4 {
                YatayHiza::Sol
            } else if kos > 0.4 {
                YatayHiza::Sağ
            } else {
                YatayHiza::Orta
            };
            let dikey = if sin < -0.8 {
                DikeyHiza::Üst
            } else if sin > 0.8 {
                DikeyHiza::Alt
            } else {
                DikeyHiza::Orta
            };
            çizici.yazı(
                &metin,
                (
                    merkez.0 + konum_yarıçapı * kos,
                    merkez.1 + konum_yarıçapı * sin,
                ),
                yatay,
                dikey,
                seri.etiket_boyutu,
                seri.etiket_rengi.unwrap_or_else(tema::eksen_etiketi),
                false,
            );
        }
        if seri.ara_çentikleri_göster && i != bölme && seri.ara_çentik_kalınlığı > 0.0 {
            let ara_sayısı = seri.ara_çentik_sayısı.max(1);
            let ara_uzunluğu = seri.ara_çentik_uzunluğu.çöz(yarıçap).max(0.0);
            let ara_dış = yarıçap - şerit - seri.ara_çentik_mesafesi;
            let ara_iç = ara_dış - ara_uzunluğu;
            for j in 0..=ara_sayısı {
                let ara_oranı = (i as f64 + j as f64 / ara_sayısı as f64) / bölme as f64;
                let ara_açısı = ekran_açısı(orandan_açı(ara_oranı));
                let (ara_kos, ara_sin) = (ara_açısı.cos(), ara_açısı.sin());
                çizici.çizgi(
                    (merkez.0 + ara_dış * ara_kos, merkez.1 + ara_dış * ara_sin),
                    (merkez.0 + ara_iç * ara_kos, merkez.1 + ara_iç * ara_sin),
                    seri.ara_çentik_kalınlığı,
                    seri.ara_çentik_rengi
                        .unwrap_or_else(tema::eksen_ara_çentiği),
                    crate::model::stil::ÇizgiTürü::Düz,
                );
            }
        }
    }

    // 3) Veri adı, ayrıntı yazısı ve ibre. ECharts'ta varsayılan ibre
    // `showAbove: true` olduğu için metinler önce, ibre en son boyanır.
    let Some(öğe) = seri.veri.first() else {
        return;
    };
    let Some(değer) = öğe.değer.sayı() else {
        return;
    };
    let animasyonlu = kapsam[0] + (değer - kapsam[0]) * ilerleme.clamp(0.0, 1.0) as f64;
    let oran = değer_oranı(animasyonlu);
    let açı = ekran_açısı(orandan_açı(oran));
    if seri.adı_göster
        && let Some(ad) = &öğe.ad
    {
        çizici.yazı(
            ad,
            (
                merkez.0 + seri.ad_merkez_kayması.0.çöz(yarıçap),
                merkez.1 + seri.ad_merkez_kayması.1.çöz(yarıçap),
            ),
            YatayHiza::Orta,
            DikeyHiza::Orta,
            seri.ad_boyutu,
            seri.ad_rengi.unwrap_or_else(tema::ikincil_metin),
            false,
        );
    }
    if seri.değeri_göster {
        let görüntülenen_değer = if seri.değer_animasyonu {
            animasyonlu
        } else {
            değer
        };
        let metin = match &seri.değer_biçimleyici {
            Some(b) => b.uygula(görüntülenen_değer, &binlik_ayır(görüntülenen_değer)),
            None => binlik_ayır(görüntülenen_değer),
        };
        çizici.yazı(
            &metin,
            (
                merkez.0 + seri.değer_merkez_kayması.0.çöz(yarıçap),
                merkez.1 + seri.değer_merkez_kayması.1.çöz(yarıçap),
            ),
            YatayHiza::Orta,
            DikeyHiza::Orta,
            seri.değer_boyutu,
            seri.değer_rengi.unwrap_or_else(tema::birincil_metin),
            seri.değer_kalın,
        );
    }

    if seri.ibreyi_göster {
        let ibre_uzunluğu = seri.ibre_uzunluğu.çöz(yarıçap).max(0.0);
        let ibre_genişliği = seri.ibre_genişliği.max(0.0);
        let (kos, sin) = (açı.cos(), açı.sin());
        let (dik_kos, dik_sin) = (-sin, kos);
        let arka_çarpanı = if ibre_genişliği >= ibre_uzunluğu / 3.0 {
            1.0
        } else {
            2.0
        };
        let mut ibre = Yol::yeni();
        ibre.taşı((
            merkez.0 - kos * ibre_genişliği * arka_çarpanı,
            merkez.1 - sin * ibre_genişliği * arka_çarpanı,
        ));
        ibre.çiz((
            merkez.0 + dik_kos * ibre_genişliği,
            merkez.1 + dik_sin * ibre_genişliği,
        ));
        ibre.çiz((
            merkez.0 + kos * ibre_uzunluğu,
            merkez.1 + sin * ibre_uzunluğu,
        ));
        ibre.çiz((
            merkez.0 - dik_kos * ibre_genişliği,
            merkez.1 - dik_sin * ibre_genişliği,
        ));
        ibre.kapat();
        çizici.yol_doldur(&ibre, &Dolgu::Düz(seri.ibre_rengi.unwrap_or(seri_rengi)));
    }

    if seri.ilerlemeyi_göster && seri.ilerleme_kalınlığı > 0.0 && oran > 0.0 {
        çizici.dilim(
            merkez,
            (yarıçap - seri.ilerleme_kalınlığı).max(0.0),
            yarıçap,
            ekran_açısı(orandan_açı(0.0)),
            ekran_açısı(orandan_açı(oran)),
            &Dolgu::Düz(seri.ilerleme_rengi.unwrap_or(seri_rengi)),
            None,
        );
    }

    isabetler.push(İsabetBölgesi {
        seri_sırası: genel_sıra,
        veri_sırası: 0,
        seri_adı: seri.ad.clone(),
        ad: öğe.ad.clone(),
        değer: Some(değer),
        geometri: İsabetGeometrisi::Daire { merkez, yarıçap },
    });
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;

    #[test]
    fn echarts_6_1_ontanimli_geometri_ana_ara_centik_ve_metni_korur() {
        tema::koyu_ayarla(false);
        let seri = GöstergeSaatiSerisi::yeni()
            .ad("Pressure")
            .değer(50.0, "SCORE")
            .değer_biçimleyici("{value}");
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let mut isabetler = Vec::new();

        gösterge_saati_çiz(
            &mut yüzey,
            &seri,
            0,
            Renk::onaltılık(0x5070dd),
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            1.0,
            &mut isabetler,
        );

        let döküm = yüzey.döküm();
        assert!(döküm.starts_with("doldur #e8ebf0@1.0 | T(217.9,394.6)"));
        assert_eq!(
            yüzey
                .komutlar
                .iter()
                .filter(|komut| komut.starts_with("çiz "))
                .count(),
            71,
            "11 ana bölme + 10 × 6 ara çentik çizilmeli"
        );
        assert!(döküm.contains("yazı \"50\" (350.0,100.6) orta/üst b=12.0"));
        assert!(döküm.contains("yazı \"SCORE\" (350.0,301.9) orta/orta b=16.0"));
        assert!(döküm.contains("yazı \"50\" (350.0,341.3) orta/orta b=30.0"));
        assert!(döküm.contains(
            "doldur #5070dd@1.0 | T(350.0,274.5) Ç(356.0,262.5) Ç(350.0,144.4) Ç(344.0,262.5) Z"
        ));
        assert!(matches!(
            isabetler.first().map(|isabet| isabet.geometri.clone()),
            Some(İsabetGeometrisi::Daire {
                merkez: (350.0, 262.5),
                yarıçap: 196.875
            })
        ));

        tema::koyu_ayarla(true);
        let mut koyu_yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        gösterge_saati_çiz(
            &mut koyu_yüzey,
            &seri,
            0,
            Renk::onaltılık(0x5070dd),
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            1.0,
            &mut Vec::new(),
        );
        assert!(koyu_yüzey.döküm().starts_with("doldur #232328@1.0"));
        tema::koyu_ayarla(false);
    }

    #[test]
    fn progress_yayi_ve_detail_value_animation_ayni_gecis_degerini_izler() {
        tema::koyu_ayarla(false);
        let seri = GöstergeSaatiSerisi::yeni()
            .değer(50.0, "SCORE")
            .ilerleme(true, 10.0)
            .değer_animasyonu(true);
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);

        gösterge_saati_çiz(
            &mut yüzey,
            &seri,
            0,
            Renk::onaltılık(0x5070dd),
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            0.5,
            &mut Vec::new(),
        );

        let palet_dolguları = yüzey
            .komutlar
            .iter()
            .filter(|komut| komut.starts_with("doldur #5070dd@1.0"))
            .collect::<Vec<_>>();
        assert_eq!(palet_dolguları.len(), 2, "ibre ve progress yayı çizilmeli");
        let ibre = palet_dolguları.first().expect("ibre dolgusu");
        let progress = palet_dolguları.last().expect("progress dolgusu");
        assert!(ibre.contains(" Ç(") && !ibre.contains(" Y("));
        assert!(progress.contains(" Y("), "progress bir yay yolu olmalı");
        assert!(
            yüzey
                .komutlar
                .iter()
                .any(|komut| komut.starts_with("yazı \"25\" (350.0,341.3)")),
            "detail.valueAnimation yarı karede 25 göstermeli"
        );
    }
}
