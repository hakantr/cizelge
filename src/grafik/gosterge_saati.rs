//! Gösterge saati (gauge) serisi — `echarts/src/chart/gauge` karşılığı:
//! renk bantlı yay, çentikler, etiketler, ibre ve değer yazısı.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::yuzey::{dilim_yolu, yuvarlak_uçlu_dilim_yolu};
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, yolu_dönüştür, ÇizimYüzeyi};
use crate::grafik::pasta::zengin_etiketi_yaz;
use crate::grafik::sembol_yolu;
use crate::koordinat::Dikdörtgen;
use crate::model::seri::{GöstergeSaatiSerisi, Sembol};
use crate::model::stil::{Etiket, YazıStili, ÇizgiTürü};
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yardimci::sayi::doğrusal_eşle;

/// ECharts gauge açısı (derece, saat yönünün tersine artan matematik
/// açısı) → ekran radyanı (y aşağı).
fn ekran_açısı(derece: f64) -> f32 {
    (-derece.to_radians()) as f32
}

fn gösterge_dilimi_yolu(
    merkez: (f32, f32),
    iç_yarıçap: f32,
    dış_yarıçap: f32,
    açı0: f32,
    açı1: f32,
    yuvarlak_uç: bool,
) -> Yol {
    if yuvarlak_uç {
        yuvarlak_uçlu_dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1)
    } else {
        dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1)
    }
}

fn gösterge_dilimini_boya(
    çizici: &mut dyn ÇizimYüzeyi,
    yol: &Yol,
    dolgu: &Dolgu,
    kenarlık: Option<(f32, Renk)>,
) {
    if yol.boş_mu() {
        return;
    }
    çizici.yol_doldur(yol, dolgu);
    if let Some((kalınlık, renk)) = kenarlık
        && kalınlık > 0.0
    {
        çizici.yol_çiz(yol, kalınlık, renk, ÇizgiTürü::Düz);
    }
}

/// `createSymbol(path://…, x, y, width, height, keepAspect)` eşdeğeri.
/// Kaynak sembolün kesin sınır kutusunu hedef dikdörtgene taşır.
fn ibre_simgesi_yolu(simge: &Sembol, hedef: Dikdörtgen, oranı_koru: bool) -> Option<Yol> {
    let kaynak = match simge {
        Sembol::SvgYolu(yol) => (**yol).clone(),
        _ => sembol_yolu(simge, (0.0, 0.0), 2.0, false)?,
    };
    let kutu = kaynak.kesin_sınır_kutusu()?;
    if kutu.genişlik <= f32::EPSILON || kutu.yükseklik <= f32::EPSILON {
        return None;
    }
    let mut hedef = hedef;
    if oranı_koru {
        let ölçek = (hedef.genişlik / kutu.genişlik).min(hedef.yükseklik / kutu.yükseklik);
        let genişlik = kutu.genişlik * ölçek;
        let yükseklik = kutu.yükseklik * ölçek;
        hedef.x += (hedef.genişlik - genişlik) / 2.0;
        hedef.y += (hedef.yükseklik - yükseklik) / 2.0;
        hedef.genişlik = genişlik;
        hedef.yükseklik = yükseklik;
    }
    let x_ölçeği = hedef.genişlik / kutu.genişlik;
    let y_ölçeği = hedef.yükseklik / kutu.yükseklik;
    Some(yolu_dönüştür(
        &kaynak,
        AfinMatris::yeni(
            x_ölçeği,
            0.0,
            0.0,
            y_ölçeği,
            hedef.x - kutu.x * x_ölçeği,
            hedef.y - kutu.y * y_ölçeği,
        ),
    ))
}

fn ibre_yolu(
    seri: &GöstergeSaatiSerisi,
    merkez: (f32, f32),
    yarıçap: f32,
    açı: f32,
) -> Option<Yol> {
    let uzunluk = seri.ibre_uzunluğu.çöz(yarıçap).max(0.0);
    let genişlik = seri.ibre_genişliği.max(0.0);
    if uzunluk <= 0.0 || genişlik <= 0.0 {
        return None;
    }
    let kayma = (
        seri.ibre_merkez_kayması.0.çöz(yarıçap),
        seri.ibre_merkez_kayması.1.çöz(yarıçap),
    );
    let yerel = if let Some(simge) = &seri.ibre_simgesi {
        ibre_simgesi_yolu(
            simge,
            Dikdörtgen::yeni(
                kayma.0 - genişlik / 2.0,
                kayma.1 - uzunluk,
                genişlik,
                uzunluk,
            ),
            seri.ibre_oranı_koru,
        )?
    } else {
        // ECharts PointerPath yerel olarak yukarı (-π/2) bakar; bütün şekil
        // daha sonra değer açısına döndürülür.
        let arka_çarpanı = if genişlik >= uzunluk / 3.0 { 1.0 } else { 2.0 };
        let mut yol = Yol::yeni();
        yol.taşı((kayma.0, kayma.1 + genişlik * arka_çarpanı));
        yol.çiz((kayma.0 - genişlik, kayma.1));
        yol.çiz((kayma.0, kayma.1 - uzunluk));
        yol.çiz((kayma.0 + genişlik, kayma.1));
        yol.kapat();
        yol
    };
    let dönüşüm = AfinMatris::ötele(merkez.0, merkez.1)
        .çarp(AfinMatris::döndür(açı + std::f32::consts::FRAC_PI_2));
    Some(yolu_dönüştür(&yerel, dönüşüm))
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
            let yol = gösterge_dilimi_yolu(
                merkez,
                yarıçap - şerit,
                yarıçap,
                ekran_açısı(orandan_açı(0.0)),
                ekran_açısı(orandan_açı(1.0)),
                seri.şerit_yuvarlak_uç,
            );
            gösterge_dilimini_boya(çizici, &yol, &Dolgu::Düz(tema::nötr_10()), None);
        } else {
            let mut önceki_oran = 0.0f64;
            for (bant_sonu, renk) in &seri.renk_bantları {
                let son = (*bant_sonu as f64).clamp(önceki_oran, 1.0);
                if son > önceki_oran {
                    let yol = gösterge_dilimi_yolu(
                        merkez,
                        yarıçap - şerit,
                        yarıçap,
                        ekran_açısı(orandan_açı(önceki_oran)),
                        ekran_açısı(orandan_açı(son)),
                        seri.şerit_yuvarlak_uç,
                    );
                    gösterge_dilimini_boya(çizici, &yol, &Dolgu::Düz(*renk), None);
                }
                önceki_oran = son;
            }
            if önceki_oran < 1.0 {
                let yol = gösterge_dilimi_yolu(
                    merkez,
                    yarıçap - şerit,
                    yarıçap,
                    ekran_açısı(orandan_açı(önceki_oran)),
                    ekran_açısı(orandan_açı(1.0)),
                    seri.şerit_yuvarlak_uç,
                );
                gösterge_dilimini_boya(çizici, &yol, &Dolgu::Düz(tema::nötr_10()), None);
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
        let ham = binlik_ayır(görüntülenen_değer);
        let metin = match &seri.değer_biçimleyici {
            Some(b) => b.uygula_bağlamla_zengin(
                görüntülenen_değer,
                &ham,
                seri.ad.as_deref().unwrap_or_default(),
                öğe.ad.as_deref().unwrap_or_default(),
            ),
            None => ham,
        };
        let taban = YazıStili {
            renk: seri.değer_rengi,
            boyut: Some(seri.değer_boyutu),
            satır_yüksekliği: Some(30.0),
            kalın: seri.değer_kalın,
            kalınlık_belirtildi: true,
            kenarlık_kalınlığı: Some(0.0),
            iç_boşluk: Some([5.0, 10.0, 5.0, 10.0]),
            genişlik: Some(crate::model::Uzunluk::Piksel(100.0)),
            ..YazıStili::default()
        };
        let mut yazı = taban.yama_uygula(&seri.değer_stili);
        if let Some(genişlik) = yazı.genişlik {
            yazı.genişlik = Some(crate::model::Uzunluk::Piksel(genişlik.çöz(yarıçap)));
        }
        let etiket = Etiket {
            göster: true,
            yazı,
            zengin: seri.değer_zengin.clone(),
            ..Etiket::default()
        };
        let konum = (
            merkez.0 + seri.değer_merkez_kayması.0.çöz(yarıçap),
            merkez.1 + seri.değer_merkez_kayması.1.çöz(yarıçap),
        );
        // zrender'da detail.width arka plan kutusunu sınırlar; `overflow`
        // ayrıca verilmedikçe rich koşularını üç noktayla kırpmaz. Ortak
        // etiket yerleşimi daraltmayı pie etiketleri için uyguladığından
        // gauge detail kutusunu ve taşabilen metin içeriğini iki geçişte
        // boyuyoruz.
        let kutu_görünür = etiket.yazı.arkaplan.is_some()
            || etiket.yazı.kenarlık_rengi.is_some()
                && etiket.yazı.kenarlık_kalınlığı.unwrap_or(1.0) > 0.0;
        if kutu_görünür {
            zengin_etiketi_yaz(
                çizici,
                "",
                &Etiket {
                    zengin: Default::default(),
                    ..etiket.clone()
                },
                konum,
                YatayHiza::Orta,
                seri.değer_rengi.unwrap_or_else(tema::birincil_metin),
                0.0,
            );
        }
        let mut içerik_etiketi = etiket;
        içerik_etiketi.yazı.arkaplan = None;
        içerik_etiketi.yazı.kenarlık_rengi = None;
        içerik_etiketi.yazı.kenarlık_kalınlığı = None;
        içerik_etiketi.yazı.kenarlık_yarıçapları = None;
        içerik_etiketi.yazı.genişlik = None;
        içerik_etiketi.yazı.yükseklik = None;
        içerik_etiketi.yazı.iç_boşluk = None;
        zengin_etiketi_yaz(
            çizici,
            &metin,
            &içerik_etiketi,
            konum,
            YatayHiza::Orta,
            seri.değer_rengi.unwrap_or_else(tema::birincil_metin),
            0.0,
        );
    }

    let öğe_opaklığı = seri.öğe_stili.opaklık.unwrap_or(1.0).clamp(0.0, 1.0);
    let öğe_dolgusu = seri
        .öğe_stili
        .renk
        .clone()
        .unwrap_or_else(|| Dolgu::Düz(seri_rengi))
        .opaklık(öğe_opaklığı);
    let öğe_kenarlığı = seri.öğe_stili.kenarlık_rengi.map(|renk| {
        (
            seri.öğe_stili.kenarlık_kalınlığı,
            renk.opaklık(öğe_opaklığı),
        )
    });
    let gölgeyi_boya = |çizici: &mut dyn ÇizimYüzeyi, yol: &Yol| {
        if seri.öğe_stili.gölge_bulanıklığı > 0.0
            && let Some(gölge_rengi) = seri.öğe_stili.gölge_rengi
        {
            çizici.yol_gölgesi(
                yol,
                gölge_rengi,
                seri.öğe_stili.gölge_bulanıklığı,
                seri.öğe_stili.gölge_kayması,
            );
        }
    };

    if seri.ibreyi_göster
        && let Some(ibre) = ibre_yolu(seri, merkez, yarıçap, açı)
    {
        let dolgu = seri
            .ibre_rengi
            .map(|renk| Dolgu::Düz(renk).opaklık(öğe_opaklığı))
            .unwrap_or_else(|| öğe_dolgusu.clone());
        gölgeyi_boya(çizici, &ibre);
        gösterge_dilimini_boya(çizici, &ibre, &dolgu, öğe_kenarlığı);
    }

    if seri.ilerlemeyi_göster && seri.ilerleme_kalınlığı > 0.0 && oran > 0.0 {
        let progress = gösterge_dilimi_yolu(
            merkez,
            (yarıçap - seri.ilerleme_kalınlığı).max(0.0),
            yarıçap,
            ekran_açısı(orandan_açı(0.0)),
            ekran_açısı(orandan_açı(oran)),
            seri.ilerleme_yuvarlak_uç,
        );
        let dolgu = seri
            .ilerleme_rengi
            .map(|renk| Dolgu::Düz(renk).opaklık(öğe_opaklığı))
            .unwrap_or_else(|| öğe_dolgusu.clone());
        gölgeyi_boya(çizici, &progress);
        gösterge_dilimini_boya(çizici, &progress, &dolgu, öğe_kenarlığı);
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
    use crate::model::stil::{Biçimleyici, ÖğeStili};
    use std::sync::Arc;

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
        assert!(
            döküm.contains(
                "doldur #5070dd@1.0 | T(350.0,274.5) Ç(344.0,262.5) Ç(350.0,144.4) Ç(356.0,262.5) Z"
            ),
            "{döküm}"
        );
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
                .any(|komut| komut.contains("yazı \"25\" (350.0,341.3)")),
            "detail.valueAnimation yarı karede 25 göstermeli: {}",
            yüzey.döküm()
        );
    }

    #[test]
    fn speed_gauge_round_cap_svg_ibre_golge_ve_rich_detaili_yapisal_kilitler() {
        tema::koyu_ayarla(false);
        let simge = Sembol::svg_yolu(
            "path://M5,0 C7,0 8,2 8,4 L10,100 C10,105 0,105 0,100 L2,4 C2,2 3,0 5,0 Z",
        )
        .expect("özel ibre yolu");
        let seri = GöstergeSaatiSerisi::yeni()
            .veri([100.0])
            .aralık(0.0, 240.0)
            .açılar(180.0, 0.0)
            .bölme_sayısı(12)
            .öğe_stili(
                ÖğeStili::yeni()
                    .renk("#58D9F9")
                    .gölge_rengi("rgba(0,138,255,0.45)")
                    .gölge_bulanıklığı(10.0)
                    .gölge_kayması(2.0, 2.0),
            )
            .şerit(true, 18.0)
            .şerit_yuvarlak_uç(true)
            .ilerleme(true, 18.0)
            .ilerleme_yuvarlak_uç(true)
            .ibre(true, "75%", 16.0)
            .ibre_simgesi(simge)
            .ibre_merkez_kayması(0.0, "5%")
            .ara_çentikler(true, 2, 6.0, 10.0, 2.0)
            .ara_çentik_rengi("#999")
            .ana_çentikler(true, 12.0, 10.0, 3.0)
            .ana_çentik_rengi("#999")
            .eksen_etiketleri(true, 30.0, 20.0)
            .eksen_etiket_rengi("#999")
            .ad_göster(false)
            .değer_merkez_kayması(0.0, "35%")
            .değer_animasyonu(true)
            .değer_biçimleyici(Biçimleyici::İşlev(Arc::new(|değer, _| {
                format!("{{value|{değer:.0}}}{{unit|km/h}}")
            })))
            .değer_stili(
                YazıStili::yeni()
                    .arkaplan("#fff")
                    .kenarlık_rengi("#999")
                    .kenarlık_kalınlığı(2.0)
                    .genişlik("60%")
                    .satır_yüksekliği(40.0)
                    .yükseklik(40.0)
                    .kenarlık_yarıçapı(8.0),
            )
            .değer_zengin_stil(
                "value",
                YazıStili::yeni().boyut(50.0).kalın(true).renk("#777"),
            )
            .değer_zengin_stil(
                "unit",
                YazıStili::yeni()
                    .boyut(20.0)
                    .renk("#999")
                    .iç_boşluk([0.0, 0.0, -20.0, 10.0]),
            );
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);

        gösterge_saati_çiz(
            &mut yüzey,
            &seri,
            0,
            Renk::onaltılık(0x58d9f9),
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            1.0,
            &mut Vec::new(),
        );

        let döküm = yüzey.döküm();
        assert!(
            döküm
                .lines()
                .next()
                .is_some_and(|satır| satır.contains(" Y(")),
            "axisLine.roundCap yay kapaklarını üretmeli"
        );
        assert_eq!(
            yüzey
                .komutlar
                .iter()
                .filter(|komut| komut.starts_with("çiz "))
                .count(),
            49,
            "13 splitLine + 12 × 3 axisTick çizilmeli"
        );
        assert_eq!(
            yüzey
                .komutlar
                .iter()
                .filter(|komut| komut.starts_with("yol-gölgesi #008aff@0.5 b=10.0 k=(2.0,2.0)"))
                .count(),
            2,
            "pointer ve progress aynı itemStyle gölgesini miras almalı: {döküm}"
        );
        let mavi = yüzey
            .komutlar
            .iter()
            .filter(|komut| komut.starts_with("doldur #58d9f9@1.0"))
            .collect::<Vec<_>>();
        assert_eq!(mavi.len(), 2, "özel pointer ve progress çizilmeli");
        assert!(mavi[0].contains(" K("), "SVG pointer kübik yolunu korumalı");
        assert!(mavi[1].contains(" Y("), "progress roundCap yay yolu olmalı");
        assert!(
            döküm.contains("138.1x50.0"),
            "detail.width radiusun %60'ıdır"
        );
        assert!(döküm.contains("yazı \"100\""));
        assert!(döküm.contains("yazı \"km/h\""));
    }
}
