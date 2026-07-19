//! ECharts 6.1 `matrix` bileşeninin yüzeyden bağımsız çizimi.

use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::{MatrisHücreTürü, MatrisYerleşimi};
use crate::model::matris::MatrisKoordinatı;
use crate::renk::{Dolgu, Renk};
use crate::tema;

/// Çözümlenmiş matrix gövdesini, hiyerarşik başlıklarını ve özel/birleşik
/// hücrelerini boyar. Yerleşim ayrıca matrix'e bağlı serilerce paylaşılır.
pub fn matris_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &MatrisKoordinatı,
    yerleşim: &MatrisYerleşimi,
) {
    let arkaplan = seçenek
        .arkaplan_stili
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(Renk::SAYDAM));
    let arkaplan_kenarlığı = seçenek
        .arkaplan_stili
        .kenarlık_rengi
        .map(|renk| (seçenek.arkaplan_stili.kenarlık_kalınlığı.max(0.0), renk))
        .filter(|(kalınlık, _)| *kalınlık > 0.0);
    yüzey.dikdörtgen(
        yerleşim.dış_kutu,
        &arkaplan,
        seçenek.arkaplan_stili.kenarlık_yarıçapı,
        arkaplan_kenarlığı,
    );

    for hücre in &yerleşim.hücreler {
        let dolgu = hücre
            .öğe_stili
            .renk
            .clone()
            .unwrap_or(Dolgu::Düz(Renk::SAYDAM));
        let kenarlık = hücre
            .öğe_stili
            .kenarlık_rengi
            .map(|renk| (hücre.öğe_stili.kenarlık_kalınlığı.max(0.0), renk))
            .filter(|(kalınlık, _)| *kalınlık > 0.0);
        yüzey.dikdörtgen(
            hücre.kutu,
            &dolgu,
            hücre.öğe_stili.kenarlık_yarıçapı,
            kenarlık,
        );

        let başlık = matches!(
            hücre.tür,
            MatrisHücreTürü::XBaşlığı | MatrisHücreTürü::YBaşlığı
        );
        if !hücre.etiket.göster && !başlık {
            continue;
        }
        let Some(değer) = hücre.değer.as_deref() else {
            continue;
        };
        let metin = hücre
            .etiket
            .biçimleyici
            .as_ref()
            .map(|biçimleyici| biçimleyici.uygula(0.0, değer))
            .unwrap_or_else(|| değer.to_owned());
        yüzey.yazı(
            &metin,
            hücre.kutu.merkez(),
            YatayHiza::Orta,
            DikeyHiza::Orta,
            hücre.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
            hücre.etiket.yazı.renk.unwrap_or_else(tema::ikincil_metin),
            hücre.etiket.yazı.kalın,
        );
    }
}
