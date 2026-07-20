//! Bağımsız ECharts `calendar` koordinat bileşeni çizimi.

use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::TakvimYerleşimi;
use crate::model::stil::{YazıDikeyHizası, YazıYatayHizası};
use crate::model::takvim::{
    TakvimEtiketTarafı, TakvimKoordinatı, TakvimYönü, TakvimYılEtiketiKonumu,
};
use crate::renk::Dolgu;
use crate::tema;
use crate::yardimci::takvim::{TakvimAnı, andan_takvime, takvimden_ana};
use crate::yerel::{ay_kısaltması, etkin_yerel};

const GÜN_MS: f64 = 86_400_000.0;

#[derive(Clone, Debug)]
struct AySınırı {
    zaman_ms: f64,
    çizgi: Vec<(f32, f32)>,
    ilk_nokta: (f32, f32),
    son_nokta: (f32, f32),
    hücre_solu_üstü: (f32, f32),
}

fn yatay_hiza(hiza: Option<YazıYatayHizası>, varsayılan: YatayHiza) -> YatayHiza {
    match hiza {
        Some(YazıYatayHizası::Sol) => YatayHiza::Sol,
        Some(YazıYatayHizası::Orta) => YatayHiza::Orta,
        Some(YazıYatayHizası::Sağ) => YatayHiza::Sağ,
        None => varsayılan,
    }
}

fn dikey_hiza(hiza: Option<YazıDikeyHizası>, varsayılan: DikeyHiza) -> DikeyHiza {
    match hiza {
        Some(YazıDikeyHizası::Üst) => DikeyHiza::Üst,
        Some(YazıDikeyHizası::Orta) => DikeyHiza::Orta,
        Some(YazıDikeyHizası::Alt) => DikeyHiza::Alt,
        None => varsayılan,
    }
}

fn haftanın_günü(gün: i64) -> usize {
    // 1970-01-01 Perşembe: JS Date#getDay() dizininde 4.
    (gün.rem_euclid(7) as usize + 4) % 7
}

fn sonraki_ayın_başı(an: TakvimAnı) -> f64 {
    let (yıl, ay) = if an.ay == 12 {
        (an.yıl.saturating_add(1), 1)
    } else {
        (an.yıl, an.ay + 1)
    };
    takvimden_ana(TakvimAnı {
        yıl,
        ay,
        gün: 1,
        saat: 0,
        dakika: 0,
        saniye: 0,
        milisaniye: 0,
    })
}

fn ay_sınırı_zamanları(yerleşim: &TakvimYerleşimi) -> Vec<f64> {
    let başlangıç = yerleşim.başlangıç_günü as f64 * GÜN_MS;
    let bitiş = yerleşim.bitiş_günü as f64 * GÜN_MS;
    let mut sonuç = vec![başlangıç];
    let mut sonraki = sonraki_ayın_başı(andan_takvime(başlangıç));
    while sonraki <= bitiş {
        sonuç.push(sonraki);
        sonraki = sonraki_ayın_başı(andan_takvime(sonraki));
    }
    sonuç.push((yerleşim.bitiş_günü.saturating_add(1)) as f64 * GÜN_MS);
    sonuç
}

fn ay_sınırı(yerleşim: &TakvimYerleşimi, zaman_ms: f64) -> Option<AySınırı> {
    let hücre = yerleşim.kısıtsız_hücre(zaman_ms)?;
    let gün = (zaman_ms / GÜN_MS).floor() as i64;
    let satır = (haftanın_günü(gün) + 7 - yerleşim.ilk_gün) % 7;
    let adım_x = yerleşim.hücre_genişliği + yerleşim.hücre_boşluğu;
    let adım_y = yerleşim.hücre_yüksekliği + yerleşim.hücre_boşluğu;
    let gövde = yerleşim.gövde_kutusu;
    let çizgi = match yerleşim.yön {
        TakvimYönü::Yatay => {
            let önceki_x = hücre.x;
            let sonraki_x = hücre.x + adım_x;
            let geçiş_y = gövde.y + satır as f32 * adım_y;
            let üst_x = if satır == 0 { önceki_x } else { sonraki_x };
            if satır == 0 {
                vec![(üst_x, gövde.y), (önceki_x, gövde.alt())]
            } else {
                vec![
                    (üst_x, gövde.y),
                    (sonraki_x, geçiş_y),
                    (önceki_x, geçiş_y),
                    (önceki_x, gövde.alt()),
                ]
            }
        }
        TakvimYönü::Dikey => {
            let önceki_y = hücre.y;
            let sonraki_y = hücre.y + adım_y;
            let geçiş_x = gövde.x + satır as f32 * adım_x;
            let sol_y = if satır == 0 { önceki_y } else { sonraki_y };
            if satır == 0 {
                vec![(gövde.x, sol_y), (gövde.sağ(), önceki_y)]
            } else {
                vec![
                    (gövde.x, sol_y),
                    (geçiş_x, sonraki_y),
                    (geçiş_x, önceki_y),
                    (gövde.sağ(), önceki_y),
                ]
            }
        }
    };
    Some(AySınırı {
        zaman_ms,
        ilk_nokta: *çizgi.first()?,
        son_nokta: *çizgi.last()?,
        hücre_solu_üstü: (hücre.x, hücre.y),
        çizgi,
    })
}

fn ay_sınırları(yerleşim: &TakvimYerleşimi) -> Vec<AySınırı> {
    ay_sınırı_zamanları(yerleşim)
        .into_iter()
        .filter_map(|zaman| ay_sınırı(yerleşim, zaman))
        .collect()
}

/// Takvim gün hücrelerinin zeminini çizer. Seri şekilleri bu katmanın
/// üstünde, ayırıcılar ile yazılar ise onların da üstünde yaşar.
pub fn takvim_arka_planı_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &TakvimKoordinatı,
    yerleşim: &TakvimYerleşimi,
) {
    let dolgu = seçenek
        .öğe_stili
        .renk
        .clone()
        .unwrap_or_else(|| Dolgu::Düz(tema::nötr_00()));
    let kenarlık_kalınlığı = seçenek.öğe_stili.kenarlık_kalınlığı.max(0.0);
    let kenarlık = (kenarlık_kalınlığı > 0.0).then(|| {
        (
            kenarlık_kalınlığı,
            seçenek
                .öğe_stili
                .kenarlık_rengi
                .unwrap_or_else(tema::nötr_10),
        )
    });
    let gün_adedi = yerleşim
        .bitiş_günü
        .saturating_sub(yerleşim.başlangıç_günü)
        .saturating_add(1);
    for sıra in 0..gün_adedi {
        let zaman = (yerleşim.başlangıç_günü.saturating_add(sıra)) as f64 * GÜN_MS;
        if let Some(hücre) = yerleşim.hücre(zaman) {
            yüzey.dikdörtgen(hücre, &dolgu, seçenek.öğe_stili.kenarlık_yarıçapı, kenarlık);
        }
    }
}

fn ayırıcıları_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &TakvimKoordinatı,
    yerleşim: &TakvimYerleşimi,
    sınırlar: &[AySınırı],
) {
    let kalınlık = seçenek.ayırıcı.kalınlık.max(0.0);
    if kalınlık <= 0.0 || sınırlar.len() < 2 {
        return;
    }
    let renk = seçenek
        .ayırıcı
        .renk
        .unwrap_or_else(tema::eksen_çizgisi)
        .opaklık(seçenek.ayırıcı.opaklık.clamp(0.0, 1.0));
    for sınır in sınırlar {
        yüzey.çoklu_çizgi(&sınır.çizgi, kalınlık, renk, seçenek.ayırıcı.tür);
    }
    let yarım = kalınlık / 2.0;
    let Some(ilk) = sınırlar.first() else {
        return;
    };
    let Some(son) = sınırlar.last() else { return };
    match yerleşim.yön {
        TakvimYönü::Yatay => {
            yüzey.çizgi(
                (ilk.ilk_nokta.0 - yarım, ilk.ilk_nokta.1),
                (son.ilk_nokta.0 + yarım, son.ilk_nokta.1),
                kalınlık,
                renk,
                seçenek.ayırıcı.tür,
            );
            yüzey.çizgi(
                (ilk.son_nokta.0 - yarım, ilk.son_nokta.1),
                (son.son_nokta.0 + yarım, son.son_nokta.1),
                kalınlık,
                renk,
                seçenek.ayırıcı.tür,
            );
        }
        TakvimYönü::Dikey => {
            yüzey.çizgi(
                (ilk.ilk_nokta.0, ilk.ilk_nokta.1 - yarım),
                (son.ilk_nokta.0, son.ilk_nokta.1 + yarım),
                kalınlık,
                renk,
                seçenek.ayırıcı.tür,
            );
            yüzey.çizgi(
                (ilk.son_nokta.0, ilk.son_nokta.1 - yarım),
                (son.son_nokta.0, son.son_nokta.1 + yarım),
                kalınlık,
                renk,
                seçenek.ayırıcı.tür,
            );
        }
    }
}

fn ay_etiketlerini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &TakvimKoordinatı,
    yerleşim: &TakvimYerleşimi,
    sınırlar: &[AySınırı],
) {
    if !seçenek.ay_etiketi.göster || sınırlar.len() < 2 {
        return;
    }
    let boyut = seçenek.ay_etiketi.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let renk = seçenek
        .ay_etiketi
        .yazı
        .renk
        .unwrap_or_else(tema::ikincil_metin);
    let başlangıçta = seçenek.ay_etiketi_tarafı == TakvimEtiketTarafı::Başlangıç;
    for (sınır, sonraki) in sınırlar.iter().zip(sınırlar.iter().skip(1)) {
        let tarih = andan_takvime(sınır.zaman_ms);
        let metin = seçenek
            .ay_adları
            .as_ref()
            .and_then(|adlar| adlar.get(tarih.ay.saturating_sub(1) as usize))
            .map(String::as_str)
            .unwrap_or_else(|| ay_kısaltması(tarih.ay));
        let (konum, doğal_yatay, doğal_dikey) = match yerleşim.yön {
            TakvimYönü::Yatay => {
                let x = if seçenek.ay_etiketi_ortala {
                    (sınır.hücre_solu_üstü.0 + sonraki.ilk_nokta.0) / 2.0
                } else if başlangıçta {
                    sınır.ilk_nokta.0
                } else {
                    sınır.son_nokta.0
                };
                let y = if başlangıçta {
                    yerleşim.gövde_kutusu.y - seçenek.ay_etiketi_kenar_boşluğu
                } else {
                    yerleşim.gövde_kutusu.alt() + seçenek.ay_etiketi_kenar_boşluğu
                };
                (
                    (x, y),
                    if seçenek.ay_etiketi_ortala {
                        YatayHiza::Orta
                    } else {
                        YatayHiza::Sol
                    },
                    if başlangıçta {
                        DikeyHiza::Alt
                    } else {
                        DikeyHiza::Üst
                    },
                )
            }
            TakvimYönü::Dikey => {
                let y = if seçenek.ay_etiketi_ortala {
                    (sınır.hücre_solu_üstü.1 + sonraki.ilk_nokta.1) / 2.0
                } else if başlangıçta {
                    sınır.ilk_nokta.1
                } else {
                    sınır.son_nokta.1
                };
                let x = if başlangıçta {
                    yerleşim.gövde_kutusu.x - seçenek.ay_etiketi_kenar_boşluğu
                } else {
                    yerleşim.gövde_kutusu.sağ() + seçenek.ay_etiketi_kenar_boşluğu
                };
                (
                    (x, y),
                    if başlangıçta {
                        YatayHiza::Sağ
                    } else {
                        YatayHiza::Sol
                    },
                    if seçenek.ay_etiketi_ortala {
                        DikeyHiza::Orta
                    } else {
                        DikeyHiza::Üst
                    },
                )
            }
        };
        yüzey.yazı(
            metin,
            konum,
            yatay_hiza(seçenek.ay_etiketi.yatay_hiza, doğal_yatay),
            dikey_hiza(seçenek.ay_etiketi.dikey_hiza, doğal_dikey),
            boyut,
            renk,
            seçenek.ay_etiketi.yazı.kalın,
        );
    }
}

fn gün_etiketlerini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &TakvimKoordinatı,
    yerleşim: &TakvimYerleşimi,
) {
    if !seçenek.gün_etiketi.göster {
        return;
    }
    let boyut = seçenek.gün_etiketi.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let renk = seçenek
        .gün_etiketi
        .yazı
        .renk
        .unwrap_or_else(tema::ikincil_metin);
    let başlangıçta = seçenek.gün_etiketi_tarafı == TakvimEtiketTarafı::Başlangıç;
    let hücre_ölçüsü = yerleşim.hücre_genişliği.min(yerleşim.hücre_yüksekliği);
    let kenar_boşluğu = seçenek.gün_etiketi_kenar_boşluğu.çöz(hücre_ölçüsü);
    for göreli in 0..7usize {
        let echarts_günü = (yerleşim.ilk_gün + göreli) % 7;
        let metin = seçenek
            .gün_adları
            .as_ref()
            .and_then(|adlar| adlar.get(echarts_günü))
            .map(String::as_str)
            .or_else(|| {
                let yerel_sıra = if echarts_günü == 0 {
                    6
                } else {
                    echarts_günü - 1
                };
                etkin_yerel().gün_kısaltmaları.get(yerel_sıra).copied()
            })
            .unwrap_or("?");
        // ECharts, dayOfWeekShort yoksa kısaltmanın ilk Unicode harfini
        // kullanır. Açık nameMap ise içeriği olduğu gibi korunur.
        let kısa;
        let metin = if seçenek.gün_adları.is_some() {
            metin
        } else {
            kısa = metin.chars().next().unwrap_or('?').to_string();
            &kısa
        };
        let (konum, doğal_yatay, doğal_dikey) = match yerleşim.yön {
            TakvimYönü::Yatay => (
                (
                    if başlangıçta {
                        yerleşim.gövde_kutusu.x - kenar_boşluğu
                    } else {
                        yerleşim.gövde_kutusu.sağ() + kenar_boşluğu
                    },
                    yerleşim.gövde_kutusu.y
                        + göreli as f32 * (yerleşim.hücre_yüksekliği + yerleşim.hücre_boşluğu)
                        + yerleşim.hücre_yüksekliği / 2.0,
                ),
                if başlangıçta {
                    YatayHiza::Sağ
                } else {
                    YatayHiza::Sol
                },
                DikeyHiza::Orta,
            ),
            TakvimYönü::Dikey => (
                (
                    yerleşim.gövde_kutusu.x
                        + göreli as f32 * (yerleşim.hücre_genişliği + yerleşim.hücre_boşluğu)
                        + yerleşim.hücre_genişliği / 2.0,
                    if başlangıçta {
                        yerleşim.gövde_kutusu.y - kenar_boşluğu
                    } else {
                        yerleşim.gövde_kutusu.alt() + kenar_boşluğu
                    },
                ),
                YatayHiza::Orta,
                if başlangıçta {
                    DikeyHiza::Alt
                } else {
                    DikeyHiza::Üst
                },
            ),
        };
        yüzey.yazı(
            metin,
            konum,
            yatay_hiza(seçenek.gün_etiketi.yatay_hiza, doğal_yatay),
            dikey_hiza(seçenek.gün_etiketi.dikey_hiza, doğal_dikey),
            boyut,
            renk,
            seçenek.gün_etiketi.yazı.kalın,
        );
    }
}

fn yıl_etiketini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &TakvimKoordinatı,
    yerleşim: &TakvimYerleşimi,
) {
    if !seçenek.yıl_etiketi.göster {
        return;
    }
    let başlangıç = andan_takvime(seçenek.aralık.başlangıç_ms).yıl;
    let bitiş = andan_takvime(seçenek.aralık.bitiş_ms).yıl;
    let metin = if bitiş > başlangıç {
        format!("{başlangıç}-{bitiş}")
    } else {
        başlangıç.to_string()
    };
    let konum = match seçenek.yıl_etiketi_konumu {
        TakvimYılEtiketiKonumu::Otomatik if seçenek.yön == TakvimYönü::Yatay => {
            TakvimYılEtiketiKonumu::Sol
        }
        TakvimYılEtiketiKonumu::Otomatik => TakvimYılEtiketiKonumu::Üst,
        konum => konum,
    };
    let gövde = yerleşim.gövde_kutusu;
    let pay = seçenek.yıl_etiketi_kenar_boşluğu;
    let (çapa, doğal_yatay, doğal_dikey, dönüş) = match konum {
        TakvimYılEtiketiKonumu::Üst | TakvimYılEtiketiKonumu::Otomatik => (
            ((gövde.x + gövde.sağ()) / 2.0, gövde.y - pay),
            YatayHiza::Orta,
            DikeyHiza::Alt,
            None,
        ),
        TakvimYılEtiketiKonumu::Alt => (
            ((gövde.x + gövde.sağ()) / 2.0, gövde.alt() + pay),
            YatayHiza::Orta,
            DikeyHiza::Üst,
            None,
        ),
        TakvimYılEtiketiKonumu::Sol => (
            (gövde.x - pay, (gövde.y + gövde.alt()) / 2.0),
            YatayHiza::Orta,
            DikeyHiza::Alt,
            Some(-std::f32::consts::FRAC_PI_2),
        ),
        TakvimYılEtiketiKonumu::Sağ => (
            (gövde.sağ() + pay, (gövde.y + gövde.alt()) / 2.0),
            YatayHiza::Orta,
            DikeyHiza::Üst,
            Some(-std::f32::consts::FRAC_PI_2),
        ),
    };
    let yatay = yatay_hiza(seçenek.yıl_etiketi.yatay_hiza, doğal_yatay);
    let dikey = dikey_hiza(seçenek.yıl_etiketi.dikey_hiza, doğal_dikey);
    let boyut = seçenek.yıl_etiketi.yazı.boyut.unwrap_or(20.0);
    let renk = seçenek.yıl_etiketi.yazı.renk.unwrap_or_else(tema::nötr_50);
    if let Some(açı) = dönüş {
        yüzey.dönüşümlü_yazı(
            &metin,
            (0.0, 0.0),
            yatay,
            dikey,
            boyut,
            renk,
            seçenek.yıl_etiketi.yazı.kalın,
            AfinMatris::ötele(çapa.0, çapa.1).çarp(AfinMatris::döndür(açı)),
        );
    } else {
        yüzey.yazı(
            &metin,
            çapa,
            yatay,
            dikey,
            boyut,
            renk,
            seçenek.yıl_etiketi.yazı.kalın,
        );
    }
}

/// Ay sınırlarını ve takvim etiketlerini seri katmanlarının üstünde çizer.
pub fn takvim_üst_katmanı_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &TakvimKoordinatı,
    yerleşim: &TakvimYerleşimi,
) {
    let sınırlar = ay_sınırları(yerleşim);
    ayırıcıları_çiz(yüzey, seçenek, yerleşim, &sınırlar);
    yıl_etiketini_çiz(yüzey, seçenek, yerleşim);
    ay_etiketlerini_çiz(yüzey, seçenek, yerleşim, &sınırlar);
    gün_etiketlerini_çiz(yüzey, seçenek, yerleşim);
}

/// Bileşeni tek çağrıda çizen geriye uyumlu yardımcı.
pub fn takvim_bileşeni_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &TakvimKoordinatı,
    yerleşim: &TakvimYerleşimi,
) {
    takvim_arka_planı_çiz(yüzey, seçenek, yerleşim);
    takvim_üst_katmanı_çiz(yüzey, seçenek, yerleşim);
}

#[cfg(test)]
mod testler {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::cizim::KayıtYüzeyi;

    #[test]
    fn yıl_takvimi_ay_sınırları_ve_tek_harfli_günleri_çizer() {
        crate::yerel::yerel_ayarla(&crate::yerel::İNGİLİZCE);
        let seçenek = TakvimKoordinatı::yıl(2016)
            .sol(30.0)
            .sağ(30)
            .üst(120)
            .hücre_boyutu(None, Some(13.0))
            .yıl_etiketi(crate::model::stil::Etiket::yeni().göster(false));
        let yerleşim = TakvimYerleşimi::kur(&seçenek, (700.0, 450.0)).unwrap();
        let sınırlar = ay_sınırları(&yerleşim);
        assert_eq!(sınırlar.len(), 13);
        assert!((sınırlar[0].ilk_nokta.0 - (30.0 + 640.0 / 53.0)).abs() < 1e-4);
        assert!((sınırlar[0].son_nokta.0 - 30.0).abs() < 1e-4);

        let mut yüzey = KayıtYüzeyi::yeni(700.0, 450.0);
        takvim_üst_katmanı_çiz(&mut yüzey, &seçenek, &yerleşim);
        let kayıt = yüzey.döküm();
        assert!(kayıt.contains("yazı \"S\""));
        assert!(kayıt.contains("yazı \"Jan\""));
    }

    #[test]
    fn yatay_takvim_yılı_solda_saat_yönünün_tersine_döner() {
        let seçenek = TakvimKoordinatı::yıl(2017);
        let yerleşim = TakvimYerleşimi::kur(&seçenek, (700.0, 525.0)).unwrap();
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);

        yıl_etiketini_çiz(&mut yüzey, &seçenek, &yerleşim);

        let kayıt = yüzey.döküm();
        assert!(kayıt.contains("dönüşümlü-yazı \"2017\""));
        assert!(kayıt.contains("m=[0.0 -1.0 1.0 0.0 50.0 130.0] Orta/Alt"));
    }
}
