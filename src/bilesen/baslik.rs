//! Başlık bileşeni çizimi — `echarts/src/component/title.ts` karşılığı.

use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::model::bilesen::Başlık;
use crate::model::YatayKonum;
use crate::tema;

const İÇ_BOŞLUK: f32 = 8.0;
const SATIR_ARASI: f32 = 4.0;

/// Başlığı çizer.
pub fn başlık_çiz(çizici: &mut dyn ÇizimYüzeyi, başlık: &Başlık) {
    let tuval_genişliği = çizici.genişlik();

    let metin_boyutu = başlık.yazı.boyut.unwrap_or(tema::BAŞLIK_BOYUTU);
    let alt_boyut = başlık.alt_yazı.boyut.unwrap_or(tema::ALT_BAŞLIK_BOYUTU);

    let ana_genişlik = başlık
        .metin
        .as_deref()
        .map(|m| çizici.yazı_ölç(m, metin_boyutu).0)
        .unwrap_or(0.0);
    let alt_genişlik = başlık
        .alt_metin
        .as_deref()
        .map(|m| çizici.yazı_ölç(m, alt_boyut).0)
        .unwrap_or(0.0);
    let blok_genişliği = ana_genişlik.max(alt_genişlik);

    let x = match başlık.sol {
        YatayKonum::Sol => İÇ_BOŞLUK,
        YatayKonum::Orta => (tuval_genişliği - blok_genişliği) / 2.0,
        YatayKonum::Sağ => tuval_genişliği - blok_genişliği - İÇ_BOŞLUK,
        YatayKonum::Değer(u) => u.çöz(tuval_genişliği),
    };
    let hiza = match başlık.sol {
        YatayKonum::Orta => YatayHiza::Orta,
        YatayKonum::Sağ => YatayHiza::Sağ,
        _ => YatayHiza::Sol,
    };
    let çapa_x = match hiza {
        YatayHiza::Sol => x,
        YatayHiza::Orta => x + blok_genişliği / 2.0,
        YatayHiza::Sağ => x + blok_genişliği,
    };

    let mut y = başlık
        .üst
        .map(|u| u.çöz(çizici.yükseklik()))
        .unwrap_or(İÇ_BOŞLUK);

    if let Some(metin) = &başlık.metin {
        let renk = başlık.yazı.renk.unwrap_or(tema::birincil_metin());
        let (_, yükseklik) = çizici.yazı(
            metin,
            (çapa_x, y),
            hiza,
            DikeyHiza::Üst,
            metin_boyutu,
            renk,
            true,
        );
        y += yükseklik + SATIR_ARASI;
    }
    if let Some(alt) = &başlık.alt_metin {
        let renk = başlık.alt_yazı.renk.unwrap_or(tema::üçüncül_metin());
        çizici.yazı(alt, (çapa_x, y), hiza, DikeyHiza::Üst, alt_boyut, renk, false);
    }
}
