//! Eksen çizimi — `echarts/src/component/axis` (AxisBuilder) karşılığı:
//! bölme çizgileri, eksen çizgisi, çentikler ve etiketler.

use crate::cizim::{DikeyHiza, YatayHiza, Çizici};
use crate::koordinat::{Kartezyen2B, ÇalışmaEkseni};
use crate::model::eksen::EksenKonumu;
use crate::model::stil::ÇizgiTürü;
use crate::tema;

/// Etiket metni: biçimleyici uygulanmış çentik etiketi.
fn etiket_metni(eksen: &ÇalışmaEkseni, değer: f64) -> String {
    let ham = eksen.ölçek.etiket(değer);
    match &eksen.seçenek.etiket.biçimleyici {
        Some(b) => b.uygula(değer, &ham),
        None => ham,
    }
}

/// Izgaranın bölme çizgilerini çizer (serilerin altında kalması için önce
/// çağrılır).
pub fn bölme_çizgilerini_çiz(çizici: &mut Çizici, kartezyen: &Kartezyen2B) {
    let alan = kartezyen.alan;
    for eksen in [&kartezyen.x, &kartezyen.y] {
        if !eksen.seçenek.bölme_görünür_mü() {
            continue;
        }
        let renk = eksen.seçenek.bölme_çizgisi.renk.unwrap_or(tema::BÖLME_ÇİZGİSİ);
        let tür = eksen.seçenek.bölme_çizgisi.tür;
        for konum in eksen.çizgi_çentikleri(false) {
            if eksen.yatay_mı() {
                // Dikey bölme çizgisi.
                çizici.çizgi((konum, alan.y), (konum, alan.alt()), 1.0, renk, tür);
            } else {
                çizici.çizgi((alan.x, konum), (alan.sağ(), konum), 1.0, renk, tür);
            }
        }
    }
}

/// Eksen çizgisi, çentikler ve etiketleri çizer.
pub fn eksenleri_çiz(çizici: &mut Çizici, kartezyen: &Kartezyen2B) {
    let alan = kartezyen.alan;
    for eksen in [&kartezyen.x, &kartezyen.y] {
        // Eksenin sabit (dik) konumu.
        let sabit = match eksen.konum {
            EksenKonumu::Alt => alan.alt(),
            EksenKonumu::Üst => alan.y,
            EksenKonumu::Sol => alan.x,
            EksenKonumu::Sağ => alan.sağ(),
        };
        let dış_yön: f32 = match eksen.konum {
            EksenKonumu::Alt | EksenKonumu::Sağ => 1.0,
            EksenKonumu::Üst | EksenKonumu::Sol => -1.0,
        };

        // 1) Eksen çizgisi.
        if eksen.seçenek.çizgi_görünür_mü() {
            let renk = eksen.seçenek.çizgi.renk.unwrap_or(tema::EKSEN_ÇİZGİSİ);
            let kalınlık = eksen.seçenek.çizgi.kalınlık;
            if eksen.yatay_mı() {
                çizici.çizgi((alan.x, sabit), (alan.sağ(), sabit), kalınlık, renk, ÇizgiTürü::Düz);
            } else {
                çizici.çizgi((sabit, alan.y), (sabit, alan.alt()), kalınlık, renk, ÇizgiTürü::Düz);
            }
        }

        // 2) Çentikler.
        if eksen.seçenek.çentik_görünür_mü() {
            let renk = tema::EKSEN_ÇENTİĞİ;
            let uzunluk = eksen.seçenek.çentik.uzunluk;
            for konum in eksen.çizgi_çentikleri(eksen.seçenek.çentik.etiketle_hizala) {
                if eksen.yatay_mı() {
                    çizici.çizgi(
                        (konum, sabit),
                        (konum, sabit + dış_yön * uzunluk),
                        1.0,
                        renk,
                        ÇizgiTürü::Düz,
                    );
                } else {
                    çizici.çizgi(
                        (sabit, konum),
                        (sabit + dış_yön * uzunluk, konum),
                        1.0,
                        renk,
                        ÇizgiTürü::Düz,
                    );
                }
            }
        }

        // 3) Etiketler.
        if eksen.seçenek.etiket.göster {
            let boyut = eksen.seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            let renk = eksen.seçenek.etiket.yazı.renk.unwrap_or(tema::EKSEN_ETİKETİ);
            let boşluk = eksen.seçenek.etiket.boşluk;
            let çentikler = eksen.etiket_çentikleri();

            // Kategori etiketlerinde otomatik seyreltme: sığmayan etiketler
            // `interval` mantığıyla atlanır.
            let adım = if eksen.yatay_mı() && eksen.ölçek.kategorik_mi() {
                let en_geniş = çentikler
                    .iter()
                    .map(|(_, ç)| çizici.yazı_ölç(&etiket_metni(eksen, ç.değer), boyut).0)
                    .fold(0.0f32, f32::max);
                let bant = eksen.bant_genişliği().max(1.0);
                ((en_geniş + 8.0) / bant).ceil().max(1.0) as usize
            } else {
                1
            };

            for (i, (konum, çentik)) in çentikler.iter().enumerate() {
                if i % adım != 0 {
                    continue;
                }
                let metin = etiket_metni(eksen, çentik.değer);
                if eksen.yatay_mı() {
                    let dikey = if dış_yön > 0.0 { DikeyHiza::Üst } else { DikeyHiza::Alt };
                    çizici.yazı(
                        &metin,
                        (*konum, sabit + dış_yön * boşluk),
                        YatayHiza::Orta,
                        dikey,
                        boyut,
                        renk,
                        false,
                    );
                } else {
                    let yatay = if dış_yön > 0.0 { YatayHiza::Sol } else { YatayHiza::Sağ };
                    çizici.yazı(
                        &metin,
                        (sabit + dış_yön * boşluk, *konum),
                        yatay,
                        DikeyHiza::Orta,
                        boyut,
                        renk,
                        false,
                    );
                }
            }
        }

        // 4) Eksen adı.
        if let Some(ad) = &eksen.seçenek.ad {
            let boyut = tema::YAZI_KÜÇÜK;
            if eksen.yatay_mı() {
                çizici.yazı(
                    ad,
                    (alan.sağ() + 8.0, sabit),
                    YatayHiza::Sol,
                    DikeyHiza::Orta,
                    boyut,
                    tema::EKSEN_ETİKETİ,
                    false,
                );
            } else {
                çizici.yazı(
                    ad,
                    (sabit, alan.y - 8.0),
                    YatayHiza::Orta,
                    DikeyHiza::Alt,
                    boyut,
                    tema::EKSEN_ETİKETİ,
                    false,
                );
            }
        }
    }
}
