//! Başlık bileşeni çizimi — `echarts/src/component/title.ts` karşılığı.

use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::bilesen::{Başlık, BaşlıkMetinHizası};
use crate::model::{DikeyKonum, Uzunluk, YatayKonum};
use crate::renk::{Dolgu, Renk};
use crate::tema;

fn metin_ölç(
    çizici: &dyn ÇizimYüzeyi,
    metin: Option<&str>,
    boyut: f32,
    satır_yüksekliği: f32,
    kalın: bool,
) -> (f32, f32) {
    let Some(metin) = metin else {
        return (0.0, 0.0);
    };
    let satırlar: Vec<&str> = metin.split('\n').collect();
    let genişlik = satırlar
        .iter()
        .map(|satır| çizici.stilli_yazı_ölç(satır, boyut, kalın).0)
        .fold(0.0_f32, f32::max);
    (genişlik, satır_yüksekliği * satırlar.len() as f32)
}

#[allow(clippy::too_many_arguments)]
fn çok_satırlı_yazı(
    çizici: &mut dyn ÇizimYüzeyi,
    metin: &str,
    çapa_x: f32,
    üst: f32,
    hiza: YatayHiza,
    boyut: f32,
    satır_yüksekliği: f32,
    renk: Renk,
    kalın: bool,
) {
    let iç_ofset = ((satır_yüksekliği - boyut) / 2.0).max(0.0);
    for (sıra, satır) in metin.split('\n').enumerate() {
        çizici.yazı(
            satır,
            (çapa_x, üst + sıra as f32 * satır_yüksekliği + iç_ofset),
            hiza,
            DikeyHiza::Üst,
            boyut,
            renk,
            kalın,
        );
    }
}

/// Başlığı çizer.
pub fn başlık_çiz(çizici: &mut dyn ÇizimYüzeyi, başlık: &Başlık) {
    let alan = Dikdörtgen::yeni(0.0, 0.0, çizici.genişlik(), çizici.yükseklik());
    başlık_çiz_alanda(çizici, başlık, alan);
}

/// Matrix gibi başka bir koordinat sisteminin verdiği yerel kutu içinde
/// başlık yerleşimi. `left/right/top/bottom` bu kutuya göre çözülür.
pub fn başlık_çiz_alanda(
    çizici: &mut dyn ÇizimYüzeyi, başlık: &Başlık, alan: Dikdörtgen
) {
    if !başlık.göster {
        return;
    }
    let tuval_genişliği = alan.genişlik;
    let tuval_yüksekliği = alan.yükseklik;

    let metin_boyutu = başlık.yazı.boyut.unwrap_or(tema::BAŞLIK_BOYUTU);
    let alt_boyut = başlık.alt_yazı.boyut.unwrap_or(tema::ALT_BAŞLIK_BOYUTU);
    let metin_satırı = başlık.yazı.satır_yüksekliği.unwrap_or(metin_boyutu);
    let alt_satırı = başlık.alt_yazı.satır_yüksekliği.unwrap_or(alt_boyut);
    let ana_kalın = if başlık.yazı.kalınlık_belirtildi {
        başlık.yazı.kalın
    } else {
        true
    };

    let (ana_genişlik, ana_yükseklik) = metin_ölç(
        çizici,
        başlık.metin.as_deref(),
        metin_boyutu,
        metin_satırı,
        ana_kalın,
    );
    let (alt_genişlik, alt_yükseklik) = metin_ölç(
        çizici,
        başlık.alt_metin.as_deref(),
        alt_boyut,
        alt_satırı,
        başlık.alt_yazı.kalın,
    );
    let blok_genişliği = ana_genişlik.max(alt_genişlik);
    let blok_yüksekliği = ana_yükseklik
        + if başlık.alt_metin.is_some() {
            başlık.öğe_boşluğu + alt_yükseklik
        } else {
            0.0
        };

    // `TitleView`, açık width seçeneğini `groupRect.width` ile değiştirir;
    // sağ/alt çözümü bu ölçülen içerik boyutuna ve padding'e göre yapılır.
    let sağdan = başlık.sağ.map(|u| u.çöz(tuval_genişliği));
    let x = alan.x
        + sağdan.map_or_else(
            || match başlık.sol {
                YatayKonum::Sol => başlık.iç_boşluk,
                YatayKonum::Orta => (tuval_genişliği - blok_genişliği) / 2.0,
                YatayKonum::Sağ => tuval_genişliği - blok_genişliği - başlık.iç_boşluk,
                YatayKonum::Değer(u) => u.çöz(tuval_genişliği) + başlık.iç_boşluk,
            },
            |sağ| tuval_genişliği - sağ - başlık.iç_boşluk - blok_genişliği,
        );
    let açık_hiza = başlık.metin_hizası.map(|hiza| match hiza {
        BaşlıkMetinHizası::Sol => YatayHiza::Sol,
        BaşlıkMetinHizası::Orta => YatayHiza::Orta,
        BaşlıkMetinHizası::Sağ => YatayHiza::Sağ,
    });
    let hiza = açık_hiza.unwrap_or_else(|| {
        // Sayısal/yüzdeli `right`, ECharts'ta geçerli bir textAlign anahtarına
        // dönüşmez ve zrender'in sol hizasına düşer.
        if başlık.sağ.is_some() {
            YatayHiza::Sol
        } else {
            match başlık.sol {
                YatayKonum::Orta => YatayHiza::Orta,
                YatayKonum::Sağ => YatayHiza::Sağ,
                _ => YatayHiza::Sol,
            }
        }
    });
    // ECharts `TitleView`: açık `textAlign` verildiğinde layoutRect.x
    // değiştirilmez ve `left` doğrudan yazı çapası olur. Otomatik hizadaysa
    // grup genişliği kadar düzeltme uygulanır.
    let çapa_x = if açık_hiza.is_some() {
        x
    } else {
        match hiza {
            YatayHiza::Sol => x,
            YatayHiza::Orta => x + blok_genişliği / 2.0,
            YatayHiza::Sağ => x + blok_genişliği,
        }
    };

    let üst_alt_çapası = başlık.üst.and_then(|üst| match üst {
        DikeyKonum::Üst => Some((alan.y + başlık.iç_boşluk, 0.0)),
        DikeyKonum::Orta => Some((alan.y + tuval_yüksekliği / 2.0, -alt_yükseklik / 2.0)),
        DikeyKonum::Alt => Some((alan.y + tuval_yüksekliği - başlık.iç_boşluk, -alt_yükseklik)),
        DikeyKonum::Değer(_) => None,
    });
    let mut y = if let Some(alt) = başlık.alt {
        alan.y + tuval_yüksekliği - alt.çöz(tuval_yüksekliği) - başlık.iç_boşluk - blok_yüksekliği
    } else {
        match başlık
            .üst
            .unwrap_or(DikeyKonum::Değer(Uzunluk::Piksel(0.0)))
        {
            DikeyKonum::Üst => alan.y + başlık.iç_boşluk,
            DikeyKonum::Orta => alan.y + tuval_yüksekliği / 2.0 - ana_yükseklik / 2.0,
            DikeyKonum::Alt => alan.y + tuval_yüksekliği - başlık.iç_boşluk - ana_yükseklik,
            DikeyKonum::Değer(u) => alan.y + u.çöz(tuval_yüksekliği) + başlık.iç_boşluk,
        }
    };

    let kutu_solu = match hiza {
        YatayHiza::Sol => çapa_x - başlık.iç_boşluk,
        YatayHiza::Orta => çapa_x - blok_genişliği / 2.0 - başlık.iç_boşluk,
        YatayHiza::Sağ => çapa_x - blok_genişliği - başlık.iç_boşluk,
    };
    let kutu = Dikdörtgen::yeni(
        kutu_solu,
        y - başlık.iç_boşluk,
        blok_genişliği + 2.0 * başlık.iç_boşluk,
        blok_yüksekliği + 2.0 * başlık.iç_boşluk,
    );
    if başlık.arkaplan.is_some() || başlık.kenarlık_kalınlığı > 0.0 {
        çizici.dikdörtgen(
            kutu,
            &Dolgu::Düz(başlık.arkaplan.unwrap_or(Renk::SAYDAM)),
            başlık.kenarlık_yarıçapı,
            (başlık.kenarlık_kalınlığı > 0.0).then_some((
                başlık.kenarlık_kalınlığı,
                başlık.kenarlık_rengi.unwrap_or_else(tema::birincil_metin),
            )),
        );
    }

    if let Some(metin) = &başlık.metin {
        let renk = başlık.yazı.renk.unwrap_or(tema::birincil_metin());
        çok_satırlı_yazı(
            çizici,
            metin,
            çapa_x,
            y,
            hiza,
            metin_boyutu,
            metin_satırı,
            renk,
            ana_kalın,
        );
        y += ana_yükseklik + başlık.öğe_boşluğu;
    } else if başlık.alt_metin.is_some() {
        // `TitleView` alt metni, boş ana metnin yüksekliği (0) sonrasında
        // yine `itemGap` kadar aşağı yerleştirir.
        y += başlık.öğe_boşluğu;
    }
    if let Some(alt) = &başlık.alt_metin {
        if let Some((çapa, alt_düzeltmesi)) = üst_alt_çapası {
            y = çapa + ana_yükseklik + başlık.öğe_boşluğu + alt_düzeltmesi;
        }
        let renk = başlık.alt_yazı.renk.unwrap_or(tema::üçüncül_metin());
        çok_satırlı_yazı(
            çizici,
            alt,
            çapa_x,
            y,
            hiza,
            alt_boyut,
            alt_satırı,
            renk,
            başlık.alt_yazı.kalın,
        );
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::model::stil::YazıStili;

    #[test]
    fn sag_alt_baslik_olculen_metni_kenarlara_yerlestirir() {
        let başlık = Başlık::yeni()
            .metin("SELECTED DATA INDICES: \n[Series 0] 2, 3, 4, 5")
            .sağ("10%")
            .alt(0)
            // Pinned ECharts TitleView bunu metin ölçüsüyle değiştirir.
            .genişlik(100)
            .iç_boşluk(15.0)
            .arkaplan("#333")
            .yazı(YazıStili::yeni().boyut(12.0).renk("#fff").kalın(false));
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);

        başlık_çiz(&mut yüzey, &başlık);

        assert_eq!(başlık.genişlik, Some(crate::model::Uzunluk::Piksel(100.0)));
        assert_eq!(
            yüzey.komutlar[0],
            "dikdörtgen (434.4,471.0 195.6x54.0) #333333@1.0 r=[0.0,0.0,0.0,0.0]"
        );
        assert_eq!(
            yüzey.komutlar[1],
            "yazı \"SELECTED DATA INDICES: \" (449.4,486.0) sol/üst b=12.0 #ffffff@1.0"
        );
        assert_eq!(
            yüzey.komutlar[2],
            "yazı \"[Series 0] 2, 3, 4, 5\" (449.4,498.0) sol/üst b=12.0 #ffffff@1.0"
        );
    }
}
