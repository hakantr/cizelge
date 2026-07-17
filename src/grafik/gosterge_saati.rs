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
    let şerit = seri.şerit_kalınlığı.max(2.0);
    let kapsam = [seri.en_az, seri.en_çok.max(seri.en_az + 1e-9)];

    let değer_oranı = |değer: f64| doğrusal_eşle(değer, kapsam, [0.0, 1.0], true);
    let orandan_açı = |oran: f64| {
        seri.başlangıç_açısı as f64
            + (seri.bitiş_açısı as f64 - seri.başlangıç_açısı as f64) * oran
    };

    // 1) Renk bantlı yay (axisLine).
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
            &Dolgu::Düz(tema::nötr_15()),
            None,
        );
    }

    // Banda göre çentik/etiket/ibre rengini çözer.
    let banttaki_renk = |oran: f64| -> Renk {
        for (bant_sonu, renk) in &seri.renk_bantları {
            if oran <= *bant_sonu as f64 + 1e-9 {
                return *renk;
            }
        }
        tema::eksen_çizgisi()
    };

    // 2) Çentikler ve etiketler.
    let bölme = seri.bölme_sayısı.max(1);
    for i in 0..=bölme {
        let oran = i as f64 / bölme as f64;
        let açı = ekran_açısı(orandan_açı(oran));
        let (kos, sin) = (açı.cos(), açı.sin());
        let dış = yarıçap - şerit;
        let iç = dış - seri.çentik_uzunluğu.max(2.0);
        çizici.çizgi(
            (merkez.0 + iç * kos, merkez.1 + iç * sin),
            (merkez.0 + dış * kos, merkez.1 + dış * sin),
            2.0,
            banttaki_renk(oran),
            crate::model::stil::ÇizgiTürü::Düz,
        );
        if seri.etiketleri_göster {
            let değer = doğrusal_eşle(oran, [0.0, 1.0], kapsam, true);
            let konum_yarıçapı = iç - 6.0;
            çizici.yazı(
                &binlik_ayır(değer),
                (
                    merkez.0 + konum_yarıçapı * kos,
                    merkez.1 + konum_yarıçapı * sin,
                ),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                seri.etiket_boyutu,
                tema::ikincil_metin(),
                false,
            );
        }
    }

    // 3) Değer, ibre ve ayrıntı yazısı.
    let Some(öğe) = seri.veri.first() else { return };
    let Some(değer) = öğe.değer.sayı() else { return };
    let animasyonlu = kapsam[0] + (değer - kapsam[0]) * ilerleme.clamp(0.0, 1.0) as f64;
    let oran = değer_oranı(animasyonlu);
    let açı = ekran_açısı(orandan_açı(oran));
    let ibre_rengi = banttaki_renk(oran);

    // İbre: dönmüş ince yamuk.
    let ibre_uzunluğu = seri.ibre_uzunluğu.çöz(yarıçap);
    let (kos, sin) = (açı.cos(), açı.sin());
    let (dik_kos, dik_sin) = (-sin, kos);
    let taban_yarı = 4.0;
    let mut ibre = Yol::yeni();
    ibre.taşı((
        merkez.0 + ibre_uzunluğu * kos,
        merkez.1 + ibre_uzunluğu * sin,
    ));
    ibre.çiz((
        merkez.0 + taban_yarı * dik_kos - 10.0 * kos,
        merkez.1 + taban_yarı * dik_sin - 10.0 * sin,
    ));
    ibre.çiz((
        merkez.0 - taban_yarı * dik_kos - 10.0 * kos,
        merkez.1 - taban_yarı * dik_sin - 10.0 * sin,
    ));
    ibre.kapat();
    çizici.yol_doldur(&ibre, &Dolgu::Düz(ibre_rengi));
    çizici.daire(merkez, 5.0, Some(&Dolgu::Düz(ibre_rengi)), None);

    // Ayrıntı (detail): değer yazısı.
    if seri.değeri_göster {
        let metin = match &seri.değer_biçimleyici {
            Some(b) => b.uygula(değer, &binlik_ayır(değer)),
            None => binlik_ayır(değer),
        };
        çizici.yazı(
            &metin,
            (merkez.0, merkez.1 + yarıçap * 0.45),
            YatayHiza::Orta,
            DikeyHiza::Orta,
            seri.değer_boyutu,
            ibre_rengi,
            true,
        );
        if let Some(ad) = &öğe.ad {
            çizici.yazı(
                ad,
                (merkez.0, merkez.1 + yarıçap * 0.45 + seri.değer_boyutu * 1.1),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                tema::YAZI_KÜÇÜK,
                tema::ikincil_metin(),
                false,
            );
        }
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
