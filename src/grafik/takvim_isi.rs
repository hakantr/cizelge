//! Takvim koordinatı — `echarts/src/coord/calendar` karşılığı: bir yılın
//! günleri hafta sütunları × haftanın günü satırlarında hücrelere dizilir;
//! değerler görsel eşlemeyle renklendirilir.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::seri::TakvimSerisi;
use crate::renk::Dolgu;
use crate::tema;
use crate::yardimci::takvim::{andan_takvime, takvimden_ana, TakvimAnı};
use crate::yerel::{ay_kısaltması, etkin_yerel};

const GÜN_MS: f64 = 86_400_000.0;

/// Unix gününden haftanın gününe (0 = Pazartesi).
fn haftanın_günü(gün_sayısı: i64) -> usize {
    // 1970-01-01 Perşembe'dir (Pzt=0 dizininde 3).
    ((gün_sayısı % 7 + 7 + 3) % 7) as usize
}

/// Serinin değer kapsamı (görsel eşleme için).
pub fn takvim_değer_kapsamı(seri: &TakvimSerisi) -> [f64; 2] {
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    for öğe in &seri.veri {
        if let Some(&değer) = öğe.değer.dizi().and_then(|d| d.get(1))
            && değer.is_finite() {
                kapsam[0] = kapsam[0].min(değer);
                kapsam[1] = kapsam[1].max(değer);
            }
    }
    if kapsam[0].is_finite() { kapsam } else { [0.0, 1.0] }
}

/// Takvim ısı haritasını çizer.
#[allow(clippy::too_many_arguments)]
pub fn takvim_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &TakvimSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    eşleme: &GörselEşleme,
    eşleme_kapsamı: [f64; 2],
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let alan = Dikdörtgen::yeni(
        tuval.x + seri.sol.çöz(tuval.genişlik),
        tuval.y + seri.üst.çöz(tuval.yükseklik),
        seri.genişlik.çöz(tuval.genişlik),
        seri.yükseklik.çöz(tuval.yükseklik),
    );

    // Yıl aralığı.
    let yıl_başı = takvimden_ana(TakvimAnı {
        yıl: seri.yıl,
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let yıl_sonu = takvimden_ana(TakvimAnı {
        yıl: seri.yıl.saturating_add(1),
        ay: 1,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    });
    let ilk_gün = (yıl_başı / GÜN_MS) as i64;
    let gün_sayısı = ((yıl_sonu - yıl_başı) / GÜN_MS).round() as i64;
    let ilk_hafta_günü = haftanın_günü(ilk_gün) as i64;
    let hafta_sayısı = ((gün_sayısı + ilk_hafta_günü) as f32 / 7.0).ceil() as usize;

    let sol_pay = 34.0;
    let üst_pay = 22.0;
    let hücre_g =
        ((alan.genişlik - sol_pay) / hafta_sayısı.max(1) as f32 - seri.hücre_boşluğu).max(2.0);
    let hücre_y = ((alan.yükseklik - üst_pay) / 7.0 - seri.hücre_boşluğu).max(2.0);
    let adım_g = hücre_g + seri.hücre_boşluğu;
    let adım_y = hücre_y + seri.hücre_boşluğu;

    // Değer arama tablosu (gün → değer).
    let mut değerler: Vec<Option<f64>> = vec![None; gün_sayısı.max(0) as usize];
    for öğe in &seri.veri {
        let Some(dizi) = öğe.değer.dizi() else { continue };
        let (Some(&ms), Some(&değer)) = (dizi.first(), dizi.get(1)) else { continue };
        if !ms.is_finite() || !değer.is_finite() {
            continue;
        }
        let gün = ((ms / GÜN_MS) as i64).saturating_sub(ilk_gün);
        if gün >= 0
            && let Some(kayıt) = değerler.get_mut(gün as usize) {
                *kayıt = Some(değer);
            }
    }

    let hücre_konumu = |gün: i64| -> Dikdörtgen {
        let hafta = ((gün + ilk_hafta_günü) / 7) as f32;
        let gün_satırı = haftanın_günü(ilk_gün + gün) as f32;
        Dikdörtgen::yeni(
            alan.x + sol_pay + hafta * adım_g,
            alan.y + üst_pay + gün_satırı * adım_y,
            hücre_g,
            hücre_y,
        )
    };

    // 1) Gün adları (solda, bir satır atlayarak).
    for (satır, ad) in etkin_yerel().gün_kısaltmaları.iter().enumerate() {
        if satır % 2 != 0 {
            continue;
        }
        çizici.yazı(
            ad,
            (
                alan.x + sol_pay - 6.0,
                alan.y + üst_pay + satır as f32 * adım_y + hücre_y / 2.0,
            ),
            YatayHiza::Sağ,
            DikeyHiza::Orta,
            10.0,
            tema::üçüncül_metin(),
            false,
        );
    }

    // 2) Hücreler + ay etiketleri.
    let opaklık = ilerleme.clamp(0.0, 1.0);
    let mut önceki_ay = 0u32;
    for gün in 0..gün_sayısı {
        let an = andan_takvime(yıl_başı + gün as f64 * GÜN_MS);
        let hücre = hücre_konumu(gün);
        // Ay başında üstte ay etiketi.
        if an.ay != önceki_ay {
            önceki_ay = an.ay;
            çizici.yazı(
                ay_kısaltması(an.ay),
                (hücre.x, alan.y + üst_pay - 6.0),
                YatayHiza::Sol,
                DikeyHiza::Alt,
                10.0,
                tema::ikincil_metin(),
                false,
            );
        }
        let değer = değerler.get(gün as usize).copied().flatten();
        let renk = match değer {
            Some(d) => {
                // Parçalı eşlemede kapalı dilim hücreleri boş görünür.
                if eşleme.parçalı_mı() {
                    match eşleme.parça_bul(d) {
                        Some(parça) if eşleme.parça_açık_mı(parça) => {
                            eşleme.renk_çöz(d, eşleme_kapsamı)
                        }
                        _ => tema::nötr_05(),
                    }
                } else {
                    eşleme.renk_çöz(d, eşleme_kapsamı)
                }
            }
            None => tema::nötr_05(),
        };
        çizici.dikdörtgen(
            hücre,
            &Dolgu::Düz(renk.opaklık(opaklık)),
            [2.0; 4],
            None,
        );
        if let Some(d) = değer {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: gün as usize,
                seri_adı: seri.ad.clone(),
                ad: Some(format!(
                    "{} {} {}",
                    an.gün,
                    ay_kısaltması(an.ay),
                    an.yıl
                )),
                değer: Some(d),
                geometri: İsabetGeometrisi::Dikdörtgen(hücre),
            });
        }
    }
}
