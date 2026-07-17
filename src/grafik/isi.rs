//! Isı haritası serisi — `echarts/src/chart/heatmap` karşılığı (kartezyen
//! kip). Veri öğeleri `[x sırası, y sırası, değer]` dizileridir; hücre
//! renkleri görsel eşlemeden çözülür.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::seri::IsıHaritasıSerisi;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

/// Isı haritası serisinin değer kapsamı (görsel eşleme için).
pub fn ısı_değer_kapsamı(seri: &IsıHaritasıSerisi) -> [f64; 2] {
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    for öğe in &seri.veri {
        if let Some(&değer) = öğe.değer.dizi().and_then(|dizi| dizi.get(2))
            && değer.is_finite()
        {
            kapsam[0] = kapsam[0].min(değer);
            kapsam[1] = kapsam[1].max(değer);
        }
    }
    if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
        [0.0, 1.0]
    } else {
        kapsam
    }
}

/// Isı haritasını çizer; hücreler `eşleme` ile renklendirilir.
#[allow(clippy::too_many_arguments)]
pub fn ısı_haritası_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &IsıHaritasıSerisi,
    genel_sıra: usize,
    kartezyen: &Kartezyen2B,
    eşleme: &GörselEşleme,
    eşleme_kapsamı: [f64; 2],
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let x_bant = kartezyen.x.bant_genişliği();
    let y_bant = kartezyen.y.bant_genişliği();
    let boşluk = seri.hücre_boşluğu.max(0.0);
    let opaklık = ilerleme.clamp(0.0, 1.0);

    for (i, öğe) in seri.veri.iter().enumerate() {
        let Some(dizi) = öğe.değer.dizi() else { continue };
        let (Some(&x_sırası), Some(&y_sırası), Some(&değer)) =
            (dizi.first(), dizi.get(1), dizi.get(2))
        else {
            continue;
        };
        if !değer.is_finite() {
            continue;
        }
        // Parçalı eşlemede kapalı dilime düşen hücre çizilmez.
        if eşleme.parçalı_mı() {
            match eşleme.parça_bul(değer) {
                Some(parça) if eşleme.parça_açık_mı(parça) => {}
                _ => continue,
            }
        }
        let merkez_x = kartezyen.x.veriden_piksele(x_sırası);
        let merkez_y = kartezyen.y.veriden_piksele(y_sırası);
        let d = Dikdörtgen::yeni(
            merkez_x - x_bant / 2.0 + boşluk / 2.0,
            merkez_y - y_bant / 2.0 + boşluk / 2.0,
            (x_bant - boşluk).max(1.0),
            (y_bant - boşluk).max(1.0),
        );
        let renk = eşleme.renk_çöz(değer, eşleme_kapsamı).opaklık(opaklık);
        let kenarlık = seri
            .öğe_stili
            .kenarlık_rengi
            .map(|r| (seri.öğe_stili.kenarlık_kalınlığı.max(1.0), r));
        çizici.dikdörtgen(d, &Dolgu::Düz(renk), seri.öğe_stili.kenarlık_yarıçapı, kenarlık);

        if seri.etiket.göster {
            let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            // Koyu hücrede beyaz, açık hücrede koyu metin.
            let parlaklık = 0.299 * renk.kırmızı + 0.587 * renk.yeşil + 0.114 * renk.mavi;
            let yazı_rengi = seri.etiket.yazı.renk.unwrap_or(if parlaklık < 0.55 {
                Renk::BEYAZ
            } else {
                tema::BİRİNCİL_METİN
            });
            çizici.yazı(
                &binlik_ayır(değer),
                d.merkez(),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                boyut,
                yazı_rengi,
                false,
            );
        }

        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: i,
            seri_adı: seri.ad.clone(),
            ad: öğe.ad.clone(),
            değer: Some(değer),
            geometri: İsabetGeometrisi::Dikdörtgen(d),
        });
    }
}

/// Görsel eşleme bileşenini sol alt köşeye çizer: sürekli kipte gradyan
/// çubuğu, parçalı kipte tıklanabilir dilim listesi. Parçalı kipte her
/// dilimin isabet kutusu döndürülür.
pub fn görsel_eşleme_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    eşleme: &GörselEşleme,
    kapsam: [f64; 2],
) -> Vec<(Dikdörtgen, usize)> {
    if !eşleme.göster {
        return Vec::new();
    }
    if eşleme.parçalı_mı() {
        // Dilim listesi (alttan üste).
        const KUTU: f32 = 14.0;
        const SATIR: f32 = 20.0;
        const KENAR: f32 = 10.0;
        let boyut = tema::YAZI_KÜÇÜK;
        let mut kutular = Vec::new();
        let n = eşleme.parçalar.len();
        for (i, parça) in eşleme.parçalar.iter().enumerate() {
            let y = çizici.yükseklik()
                - KENAR
                - SATIR * (n.saturating_sub(i)) as f32;
            let açık = eşleme.parça_açık_mı(i);
            let renk = if açık { parça.renk } else { tema::DEVRE_DIŞI };
            let kutu = Dikdörtgen::yeni(KENAR, y + (SATIR - KUTU) / 2.0, KUTU, KUTU);
            çizici.dikdörtgen(kutu, &Dolgu::Düz(renk), [3.0; 4], None);
            let yazı_rengi = if açık { tema::İKİNCİL_METİN } else { tema::DEVRE_DIŞI };
            let (metin_gen, _) = çizici.yazı(
                &parça.etiket_metni(),
                (KENAR + KUTU + 6.0, y + SATIR / 2.0),
                YatayHiza::Sol,
                DikeyHiza::Orta,
                boyut,
                yazı_rengi,
                false,
            );
            kutular.push((
                Dikdörtgen::yeni(KENAR, y, KUTU + 6.0 + metin_gen, SATIR),
                i,
            ));
        }
        return kutular;
    }
    if eşleme.renkler.is_empty() {
        return Vec::new();
    }
    const GENİŞLİK: f32 = 14.0;
    const YÜKSEKLİK: f32 = 130.0;
    const KENAR_BOŞLUĞU: f32 = 10.0;

    let x = KENAR_BOŞLUĞU;
    let y = çizici.yükseklik() - KENAR_BOŞLUĞU - YÜKSEKLİK;

    // Şerit: renk duraklarını dikey gradyan bantları olarak çiz
    // (üst = en yüksek değer).
    let bölme_sayısı = eşleme.renkler.len().saturating_sub(1).max(1);
    let bant_yüksekliği = YÜKSEKLİK / bölme_sayısı as f32;
    for i in 0..bölme_sayısı {
        let üst_renk = eşleme
            .renkler
            .get(eşleme.renkler.len().saturating_sub(1).saturating_sub(i))
            .copied()
            .unwrap_or(Renk::SİYAH);
        let alt_renk = eşleme
            .renkler
            .get(eşleme.renkler.len().saturating_sub(2).saturating_sub(i))
            .copied()
            .unwrap_or(üst_renk);
        let bant = Dikdörtgen::yeni(x, y + i as f32 * bant_yüksekliği, GENİŞLİK, bant_yüksekliği);
        çizici.dikdörtgen(
            bant,
            &crate::renk::Dolgu::doğrusal(
                0.0,
                0.0,
                0.0,
                1.0,
                vec![
                    crate::renk::RenkDurağı::yeni(0.0, üst_renk),
                    crate::renk::RenkDurağı::yeni(1.0, alt_renk),
                ],
            ),
            [0.0; 4],
            None,
        );
    }

    // Uç etiketleri.
    let boyut = tema::YAZI_KÜÇÜK;
    çizici.yazı(
        &binlik_ayır(kapsam[1]),
        (x + GENİŞLİK / 2.0, y - 4.0),
        YatayHiza::Orta,
        DikeyHiza::Alt,
        boyut,
        tema::İKİNCİL_METİN,
        false,
    );
    çizici.yazı(
        &binlik_ayır(kapsam[0]),
        (x + GENİŞLİK / 2.0, y + YÜKSEKLİK + 4.0),
        YatayHiza::Orta,
        DikeyHiza::Üst,
        boyut,
        tema::İKİNCİL_METİN,
        false,
    );
    Vec::new()
}
