//! Isı haritası serisi — `echarts/src/chart/heatmap` karşılığı (kartezyen
//! kip). Veri öğeleri `[x sırası, y sırası, değer]` dizileridir; hücre
//! renkleri görsel eşlemeden çözülür.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::YatayKonum;
use crate::model::bilesen::Yön;
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
        let Some(dizi) = öğe.değer.dizi() else {
            continue;
        };
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
        çizici.dikdörtgen(
            d,
            &Dolgu::Düz(renk),
            seri.öğe_stili.kenarlık_yarıçapı,
            kenarlık,
        );

        if seri.etiket.göster {
            let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            // Koyu hücrede beyaz, açık hücrede koyu metin.
            let parlaklık = 0.299 * renk.kırmızı + 0.587 * renk.yeşil + 0.114 * renk.mavi;
            let yazı_rengi = seri.etiket.yazı.renk.unwrap_or(if parlaklık < 0.55 {
                Renk::BEYAZ
            } else {
                tema::birincil_metin()
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

/// Görsel eşleme bileşenini seçeneklerdeki kutu yerleşimine göre çizer:
/// sürekli kipte gradyan çubuğu, parçalı kipte tıklanabilir dilim listesi.
/// Parçalı kipte her dilimin isabet kutusu döndürülür.
pub fn görsel_eşleme_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    eşleme: &GörselEşleme,
    kapsam: [f64; 2],
) -> Vec<(Dikdörtgen, usize)> {
    if !eşleme.göster {
        return Vec::new();
    }
    if eşleme.parçalı_mı() {
        // PiecewiseModel varsayılanları: 20×14 simge, 10 px itemGap ve
        // ECharts 6 orta boy bileşen iç boşluğu (15 px). Dikey/inverse=false
        // düzeninde yüksek değer üstte görünür.
        const KUTU_GENİŞLİĞİ: f32 = 20.0;
        const KUTU_YÜKSEKLİĞİ: f32 = 14.0;
        const ÖĞE_BOŞLUĞU: f32 = 10.0;
        const METİN_BOŞLUĞU: f32 = 10.0;
        const İÇ_BOŞLUK: f32 = 15.0;
        let boyut = tema::YAZI_KÜÇÜK;
        let mut kutular = Vec::new();
        let n = eşleme.parçalar.len();
        let içerik_yüksekliği =
            n as f32 * KUTU_YÜKSEKLİĞİ + n.saturating_sub(1) as f32 * ÖĞE_BOŞLUĞU;
        let üst = eşleme
            .üst
            .map(|üst| üst.çöz(çizici.yükseklik()) + İÇ_BOŞLUK)
            .unwrap_or_else(|| {
                çizici.yükseklik()
                    - eşleme.alt.çöz(çizici.yükseklik())
                    - İÇ_BOŞLUK
                    - içerik_yüksekliği
            });
        let sağa_yaslı = eşleme.sağ.is_some() || eşleme.sol == YatayKonum::Sağ;
        let en_geniş_etiket = eşleme
            .parçalar
            .iter()
            .map(|parça| çizici.yazı_ölç(&parça.etiket_metni(), boyut).0)
            .fold(0.0_f32, f32::max);
        let içerik_genişliği = KUTU_GENİŞLİĞİ + METİN_BOŞLUĞU + en_geniş_etiket;
        let içerik_solu = if let Some(sağ) = eşleme.sağ {
            çizici.genişlik() - sağ.çöz(çizici.genişlik()) - İÇ_BOŞLUK - içerik_genişliği
        } else {
            match eşleme.sol {
                YatayKonum::Sol => İÇ_BOŞLUK,
                YatayKonum::Orta => (çizici.genişlik() - içerik_genişliği) / 2.0,
                YatayKonum::Sağ => çizici.genişlik() - 10.0 - İÇ_BOŞLUK - içerik_genişliği,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()) + İÇ_BOŞLUK,
            }
        };
        let kutu_x = if sağa_yaslı {
            içerik_solu + en_geniş_etiket + METİN_BOŞLUĞU
        } else {
            içerik_solu
        };
        for (satır, (i, parça)) in eşleme.parçalar.iter().enumerate().rev().enumerate() {
            let y = üst + satır as f32 * (KUTU_YÜKSEKLİĞİ + ÖĞE_BOŞLUĞU);
            let açık = eşleme.parça_açık_mı(i);
            let renk = if açık {
                parça.renk
            } else {
                tema::devre_dışı()
            };
            let kutu = Dikdörtgen::yeni(kutu_x, y, KUTU_GENİŞLİĞİ, KUTU_YÜKSEKLİĞİ);
            çizici.dikdörtgen(kutu, &Dolgu::Düz(renk), [3.0; 4], None);
            let yazı_rengi = if açık {
                tema::ikincil_metin()
            } else {
                tema::devre_dışı()
            };
            let etiket = parça.etiket_metni();
            let (etiket_x, yatay_hiza) = if sağa_yaslı {
                (kutu_x - METİN_BOŞLUĞU, YatayHiza::Sağ)
            } else {
                (kutu_x + KUTU_GENİŞLİĞİ + METİN_BOŞLUĞU, YatayHiza::Sol)
            };
            çizici.yazı(
                &etiket,
                (etiket_x, y + KUTU_YÜKSEKLİĞİ / 2.0),
                yatay_hiza,
                DikeyHiza::Orta,
                boyut,
                yazı_rengi,
                false,
            );
            kutular.push((
                Dikdörtgen::yeni(içerik_solu, y, içerik_genişliği, KUTU_YÜKSEKLİĞİ),
                i,
            ));
        }
        return kutular;
    }
    if eşleme.renkler.is_empty() {
        return Vec::new();
    }
    if eşleme.yön == Yön::Yatay {
        const ŞERİT_GENİŞLİĞİ: f32 = 140.0;
        const ŞERİT_YÜKSEKLİĞİ: f32 = 20.0;
        const METİN_BOŞLUĞU: f32 = 10.0;
        let boyut = tema::YAZI_KÜÇÜK;
        let yüksek = eşleme
            .metin
            .as_ref()
            .map(|(yüksek, _)| yüksek.clone())
            .unwrap_or_else(|| binlik_ayır(kapsam[1]));
        let düşük = eşleme
            .metin
            .as_ref()
            .map(|(_, düşük)| düşük.clone())
            .unwrap_or_else(|| binlik_ayır(kapsam[0]));
        let düşük_genişliği = çizici.yazı_ölç(&düşük, boyut).0;
        let yüksek_genişliği = çizici.yazı_ölç(&yüksek, boyut).0;
        let toplam_genişlik =
            düşük_genişliği + METİN_BOŞLUĞU + ŞERİT_GENİŞLİĞİ + METİN_BOŞLUĞU + yüksek_genişliği;
        let grup_x = if let Some(sağ) = eşleme.sağ {
            çizici.genişlik() - sağ.çöz(çizici.genişlik()) - toplam_genişlik
        } else {
            match eşleme.sol {
                YatayKonum::Sol => 10.0,
                YatayKonum::Orta => (çizici.genişlik() - toplam_genişlik) / 2.0,
                YatayKonum::Sağ => çizici.genişlik() - 10.0 - toplam_genişlik,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()),
            }
        };
        let y = eşleme
            .üst
            .map(|üst| üst.çöz(çizici.yükseklik()))
            .unwrap_or_else(|| {
                çizici.yükseklik() - eşleme.alt.çöz(çizici.yükseklik()) - ŞERİT_YÜKSEKLİĞİ
            });
        let şerit_x = grup_x + düşük_genişliği + METİN_BOŞLUĞU;
        let durak_sayısı = eşleme.renkler.len().saturating_sub(1).max(1) as f32;
        let duraklar = eşleme
            .renkler
            .iter()
            .enumerate()
            .map(|(sıra, renk)| crate::renk::RenkDurağı::yeni(sıra as f32 / durak_sayısı, *renk))
            .collect();
        çizici.dikdörtgen(
            Dikdörtgen::yeni(şerit_x, y, ŞERİT_GENİŞLİĞİ, ŞERİT_YÜKSEKLİĞİ),
            &crate::renk::Dolgu::doğrusal(0.0, 0.0, 1.0, 0.0, duraklar),
            [0.0; 4],
            None,
        );
        çizici.yazı(
            &düşük,
            (şerit_x - METİN_BOŞLUĞU, y + ŞERİT_YÜKSEKLİĞİ / 2.0),
            YatayHiza::Sağ,
            DikeyHiza::Orta,
            boyut,
            tema::ikincil_metin(),
            false,
        );
        çizici.yazı(
            &yüksek,
            (
                şerit_x + ŞERİT_GENİŞLİĞİ + METİN_BOŞLUĞU,
                y + ŞERİT_YÜKSEKLİĞİ / 2.0,
            ),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            boyut,
            tema::ikincil_metin(),
            false,
        );
        return Vec::new();
    }
    const GENİŞLİK: f32 = 14.0;
    const YÜKSEKLİK: f32 = 130.0;
    let x = if let Some(sağ) = eşleme.sağ {
        çizici.genişlik() - sağ.çöz(çizici.genişlik()) - GENİŞLİK
    } else {
        match eşleme.sol {
            YatayKonum::Sol => 10.0,
            YatayKonum::Orta => (çizici.genişlik() - GENİŞLİK) / 2.0,
            YatayKonum::Sağ => çizici.genişlik() - 10.0 - GENİŞLİK,
            YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()),
        }
    };
    let y = eşleme
        .üst
        .map(|üst| üst.çöz(çizici.yükseklik()))
        .unwrap_or_else(|| çizici.yükseklik() - eşleme.alt.çöz(çizici.yükseklik()) - YÜKSEKLİK);

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
    let yüksek = eşleme
        .metin
        .as_ref()
        .map(|(yüksek, _)| yüksek.clone())
        .unwrap_or_else(|| binlik_ayır(kapsam[1]));
    let düşük = eşleme
        .metin
        .as_ref()
        .map(|(_, düşük)| düşük.clone())
        .unwrap_or_else(|| binlik_ayır(kapsam[0]));
    çizici.yazı(
        &yüksek,
        (x + GENİŞLİK / 2.0, y - 4.0),
        YatayHiza::Orta,
        DikeyHiza::Alt,
        boyut,
        tema::ikincil_metin(),
        false,
    );
    çizici.yazı(
        &düşük,
        (x + GENİŞLİK / 2.0, y + YÜKSEKLİK + 4.0),
        YatayHiza::Orta,
        DikeyHiza::Üst,
        boyut,
        tema::ikincil_metin(),
        false,
    );
    Vec::new()
}
