//! ECharts 6.1 `matrix` bileşeninin yüzeyden bağımsız çizimi.

use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::MatrisYerleşimi;
use crate::model::matris::{MatrisEtiketiBağlamı, MatrisKoordinatı};
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
    yüzey.dikdörtgen(
        yerleşim.dış_kutu,
        &arkaplan,
        seçenek.arkaplan_stili.kenarlık_yarıçapı,
        None,
    );

    let sınır_z2 = seçenek.kenarlık_z2.unwrap_or(99);
    let mut hücreler = yerleşim.hücreler.iter().collect::<Vec<_>>();
    hücreler.sort_by_key(|hücre| hücre.z2);
    for hücre in hücreler.iter().copied().filter(|hücre| hücre.z2 < sınır_z2) {
        hücre_çiz(yüzey, hücre, yerleşim.bileşen_sırası);
    }

    // MatrixView dış sınırı ve x/y başlık ayırıcılarını normal hücre
    // kenarlıklarının üstünde, açık yüksek-z2 hücrelerin altında tutar.
    if seçenek.x.göster && seçenek.x.ayırıcı.kalınlık > 0.0 {
        yüzey.çizgi(
            (yerleşim.dış_kutu.x, yerleşim.gövde_kutusu.y),
            (yerleşim.dış_kutu.sağ(), yerleşim.gövde_kutusu.y),
            seçenek.x.ayırıcı.kalınlık,
            seçenek.x.ayırıcı.renk.unwrap_or_else(tema::nötr_40),
            seçenek.x.ayırıcı.tür,
        );
    }
    if seçenek.y.göster && seçenek.y.ayırıcı.kalınlık > 0.0 {
        yüzey.çizgi(
            (yerleşim.gövde_kutusu.x, yerleşim.dış_kutu.y),
            (yerleşim.gövde_kutusu.x, yerleşim.dış_kutu.alt()),
            seçenek.y.ayırıcı.kalınlık,
            seçenek.y.ayırıcı.renk.unwrap_or_else(tema::nötr_40),
            seçenek.y.ayırıcı.tür,
        );
    }
    let arkaplan_kenarlığı = seçenek
        .arkaplan_stili
        .kenarlık_rengi
        .map(|renk| (seçenek.arkaplan_stili.kenarlık_kalınlığı.max(0.0), renk))
        .filter(|(kalınlık, _)| *kalınlık > 0.0);
    if arkaplan_kenarlığı.is_some() {
        yüzey.dikdörtgen(
            yerleşim.dış_kutu,
            &Dolgu::Düz(Renk::SAYDAM),
            seçenek.arkaplan_stili.kenarlık_yarıçapı,
            arkaplan_kenarlığı,
        );
    }

    for hücre in hücreler.into_iter().filter(|hücre| hücre.z2 >= sınır_z2) {
        hücre_çiz(yüzey, hücre, yerleşim.bileşen_sırası);
    }
}

fn hücre_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    hücre: &crate::koordinat::MatrisHücreYerleşimi,
    bileşen_sırası: usize,
) {
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

    if !hücre.etiket.göster {
        return;
    }
    let Some(değer) = hücre.değer.as_deref() else {
        return;
    };
    let bağlam = MatrisEtiketiBağlamı {
        bileşen_sırası,
        ad: değer.to_owned(),
        değer: değer.to_owned(),
        koordinat: [hücre.x_aralığı[0], hücre.y_aralığı[0]],
    };
    let metin = if let Some(biçimleyici) = &hücre.etiket_bağlamlı_biçimleyici {
        biçimleyici.uygula(&bağlam)
    } else if let Some(biçimleyici) = &hücre.etiket.biçimleyici {
        biçimleyici
            .uygula(0.0, değer)
            .replace("{name}", değer)
            .replace(
                "{coord}",
                &format!("[{}, {}]", bağlam.koordinat[0], bağlam.koordinat[1]),
            )
    } else {
        değer.to_owned()
    };
    let merkez = hücre.kutu.merkez();
    let boyut = hücre.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let satır_yüksekliği = hücre.etiket.yazı.satır_yüksekliği.unwrap_or(boyut).max(0.0);
    let mut satırlar = metni_satırlara_sar(
        yüzey,
        &metin,
        // Matrix label kutusu varsayılan olarak ek yatay padding uygulamaz.
        // Yalnız raster sınırına taşmayı önleyen birer piksellik pay bırak.
        (hücre.kutu.genişlik - 2.0).max(0.0),
        boyut,
        hücre.etiket.yazı.kalın,
    );
    let en_çok_satır =
        (((hücre.kutu.yükseklik - 4.0).max(0.0) / satır_yüksekliği).floor() as usize).max(1);
    satırlar.truncate(en_çok_satır);
    let ilk_y = merkez.1 + hücre.etiket.kayma.1
        - satır_yüksekliği * satırlar.len().saturating_sub(1) as f32 / 2.0;
    for (sıra, satır) in satırlar.into_iter().enumerate() {
        yüzey.yazı(
            &satır,
            (
                merkez.0 + hücre.etiket.kayma.0,
                ilk_y + sıra as f32 * satır_yüksekliği,
            ),
            YatayHiza::Orta,
            DikeyHiza::Orta,
            boyut,
            hücre.etiket.yazı.renk.unwrap_or_else(tema::ikincil_metin),
            hücre.etiket.yazı.kalın,
        );
    }
}

fn metni_satırlara_sar(
    yüzey: &dyn ÇizimYüzeyi,
    metin: &str,
    en_çok_genişlik: f32,
    boyut: f32,
    kalın: bool,
) -> Vec<String> {
    let mut sonuç = Vec::new();
    for açık_satır in metin.split('\n') {
        if açık_satır.is_empty() {
            sonuç.push(String::new());
            continue;
        }
        let mut satır = String::new();
        for karakter in açık_satır.chars() {
            let mut aday = satır.clone();
            aday.push(karakter);
            let genişlik = yüzey.stilli_yazı_ölç(&aday, boyut, kalın).0;
            if !satır.is_empty() && genişlik > en_çok_genişlik {
                sonuç.push(std::mem::take(&mut satır));
            }
            satır.push(karakter);
        }
        if !satır.is_empty() {
            sonuç.push(satır);
        }
    }
    if sonuç.is_empty() {
        sonuç.push(String::new());
    }
    sonuç
}
