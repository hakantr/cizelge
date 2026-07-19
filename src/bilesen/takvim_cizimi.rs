//! Bağımsız ECharts `calendar` koordinat bileşeni çizimi.

use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::TakvimYerleşimi;
use crate::model::takvim::{TakvimKoordinatı, TakvimYönü};
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::takvim::andan_takvime;
use crate::yerel::{ay_kısaltması, etkin_yerel};

const GÜN_MS: f64 = 86_400_000.0;

pub fn takvim_bileşeni_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &TakvimKoordinatı,
    yerleşim: &TakvimYerleşimi,
) {
    let dolgu = seçenek
        .öğe_stili
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(Renk::SAYDAM));
    let kenarlık = seçenek
        .öğe_stili
        .kenarlık_rengi
        .map(|renk| (seçenek.öğe_stili.kenarlık_kalınlığı.max(0.0), renk))
        .filter(|(kalınlık, _)| *kalınlık > 0.0);
    let gün_adedi = yerleşim
        .bitiş_günü
        .saturating_sub(yerleşim.başlangıç_günü)
        .saturating_add(1);
    let mut son_ay = None;
    for sıra in 0..gün_adedi {
        let zaman = (yerleşim.başlangıç_günü.saturating_add(sıra)) as f64 * GÜN_MS;
        let Some(hücre) = yerleşim.hücre(zaman) else {
            continue;
        };
        yüzey.dikdörtgen(hücre, &dolgu, seçenek.öğe_stili.kenarlık_yarıçapı, kenarlık);
        let an = andan_takvime(zaman);
        if seçenek.ay_etiketi.göster && son_ay != Some((an.yıl, an.ay)) {
            son_ay = Some((an.yıl, an.ay));
            let (konum, yatay, dikey) = match seçenek.yön {
                TakvimYönü::Yatay => (
                    (hücre.x, yerleşim.gövde_kutusu.y - 5.0),
                    YatayHiza::Sol,
                    DikeyHiza::Alt,
                ),
                TakvimYönü::Dikey => (
                    (yerleşim.gövde_kutusu.x - 5.0, hücre.y),
                    YatayHiza::Sağ,
                    DikeyHiza::Üst,
                ),
            };
            yüzey.yazı(
                ay_kısaltması(an.ay),
                konum,
                yatay,
                dikey,
                seçenek.ay_etiketi.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
                seçenek
                    .ay_etiketi
                    .yazı
                    .renk
                    .unwrap_or_else(tema::ikincil_metin),
                seçenek.ay_etiketi.yazı.kalın,
            );
        }
    }

    if seçenek.gün_etiketi.göster {
        for göreli in 0..7usize {
            let gün_sırası = (yerleşim.ilk_gün + göreli) % 7;
            let Some(ad) = etkin_yerel().gün_kısaltmaları.get(gün_sırası) else {
                continue;
            };
            let (konum, yatay) = match seçenek.yön {
                TakvimYönü::Yatay => (
                    (
                        yerleşim.gövde_kutusu.x - 5.0,
                        yerleşim.gövde_kutusu.y
                            + göreli as f32 * (yerleşim.hücre_yüksekliği + yerleşim.hücre_boşluğu)
                            + yerleşim.hücre_yüksekliği / 2.0,
                    ),
                    YatayHiza::Sağ,
                ),
                TakvimYönü::Dikey => (
                    (
                        yerleşim.gövde_kutusu.x
                            + göreli as f32 * (yerleşim.hücre_genişliği + yerleşim.hücre_boşluğu)
                            + yerleşim.hücre_genişliği / 2.0,
                        yerleşim.gövde_kutusu.y - 5.0,
                    ),
                    YatayHiza::Orta,
                ),
            };
            yüzey.yazı(
                ad,
                konum,
                yatay,
                if seçenek.yön == TakvimYönü::Yatay {
                    DikeyHiza::Orta
                } else {
                    DikeyHiza::Alt
                },
                seçenek.gün_etiketi.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
                seçenek
                    .gün_etiketi
                    .yazı
                    .renk
                    .unwrap_or_else(tema::üçüncül_metin),
                seçenek.gün_etiketi.yazı.kalın,
            );
        }
    }

    if seçenek.yıl_etiketi.göster {
        let yıl = andan_takvime(seçenek.aralık.başlangıç_ms).yıl.to_string();
        yüzey.yazı(
            &yıl,
            (yerleşim.dış_kutu.x, yerleşim.dış_kutu.alt() + 4.0),
            YatayHiza::Sol,
            DikeyHiza::Üst,
            seçenek.yıl_etiketi.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
            seçenek
                .yıl_etiketi
                .yazı
                .renk
                .unwrap_or_else(tema::ikincil_metin),
            seçenek.yıl_etiketi.yazı.kalın,
        );
    }
}
