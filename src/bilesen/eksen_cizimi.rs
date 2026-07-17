//! Eksen çizimi — `echarts/src/component/axis` (AxisBuilder) karşılığı:
//! bölme çizgileri, eksen çizgisi, çentikler ve etiketler.
//!
//! İnce (1 px) çizgiler [`keskin`] ile fiziksel piksel ızgarasına oturtulur;
//! eksen ve bölme çizgilerinin bulanıklaşması böyle önlenir.

use crate::cizim::{keskin, DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, ÇalışmaEkseni};
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

/// Izgaranın bölme alanlarını ve bölme çizgilerini çizer (serilerin altında
/// kalması için önce çağrılır).
pub fn bölme_çizgilerini_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    alan: Dikdörtgen,
    eksenler: &[&ÇalışmaEkseni],
) {
    // 1) Bölme alanları (`splitArea`): ana çentikler arasında dönüşümlü
    //    bantlar; çizgilerin de altında kalır.
    for eksen in eksenler {
        if !eksen.seçenek.bölme_alanı.göster
            || eksen.seçenek.bölme_alanı.renkler.is_empty()
        {
            continue;
        }
        let renkler = &eksen.seçenek.bölme_alanı.renkler;
        let konumlar = eksen.çizgi_çentikleri(false);
        for (i, çift) in konumlar.windows(2).enumerate() {
            let [a, b] = çift else { continue };
            let Some(renk) = renkler.get(i % renkler.len()) else { continue };
            let d = if eksen.yatay_mı() {
                crate::koordinat::Dikdörtgen::yeni(
                    a.min(*b),
                    alan.y,
                    (b - a).abs(),
                    alan.yükseklik,
                )
            } else {
                crate::koordinat::Dikdörtgen::yeni(
                    alan.x,
                    a.min(*b),
                    alan.genişlik,
                    (b - a).abs(),
                )
            };
            çizici.dikdörtgen(d, &crate::renk::Dolgu::Düz(*renk), [0.0; 4], None);
        }
    }

    // 2) Ara bölme çizgileri (`minorSplitLine`).
    for eksen in eksenler {
        if !eksen.seçenek.ara_bölme_çizgisi.göster.unwrap_or(false) {
            continue;
        }
        let renk = eksen
            .seçenek
            .ara_bölme_çizgisi
            .renk
            .unwrap_or(tema::ARA_BÖLME_ÇİZGİSİ);
        let tür = eksen.seçenek.ara_bölme_çizgisi.tür;
        for konum in eksen.ara_çentik_pikselleri(eksen.seçenek.ara_çentik.bölme_sayısı) {
            let konum = keskin(konum);
            if eksen.yatay_mı() {
                çizici.çizgi((konum, alan.y), (konum, alan.alt()), 1.0, renk, tür);
            } else {
                çizici.çizgi((alan.x, konum), (alan.sağ(), konum), 1.0, renk, tür);
            }
        }
    }

    // 3) Ana bölme çizgileri (`splitLine`).
    for eksen in eksenler {
        if !eksen.seçenek.bölme_görünür_mü() {
            continue;
        }
        let renk = eksen.seçenek.bölme_çizgisi.renk.unwrap_or(tema::BÖLME_ÇİZGİSİ);
        let tür = eksen.seçenek.bölme_çizgisi.tür;
        for konum in eksen.çizgi_çentikleri(false) {
            let konum = keskin(konum);
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
pub fn eksenleri_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    alan: Dikdörtgen,
    eksenler: &[&ÇalışmaEkseni],
) {
    for eksen in eksenler {
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
        let sabit_keskin = keskin(sabit);

        // 1) Eksen çizgisi.
        if eksen.seçenek.çizgi_görünür_mü() {
            let renk = eksen.seçenek.çizgi.renk.unwrap_or(tema::EKSEN_ÇİZGİSİ);
            let kalınlık = eksen.seçenek.çizgi.kalınlık;
            let konum = if kalınlık <= 1.5 { sabit_keskin } else { sabit };
            if eksen.yatay_mı() {
                çizici.çizgi(
                    (alan.x, konum),
                    (alan.sağ(), konum),
                    kalınlık,
                    renk,
                    ÇizgiTürü::Düz,
                );
            } else {
                çizici.çizgi(
                    (konum, alan.y),
                    (konum, alan.alt()),
                    kalınlık,
                    renk,
                    ÇizgiTürü::Düz,
                );
            }
        }

        // Kategori eksenlerinde seyreltme adımı: sığmayan etiket VE
        // çentikler `interval` mantığıyla atlanır (ECharts davranışı).
        let boyut_ön = eksen.seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
        let adım = if eksen.ölçek.kategorik_mi() {
            let çentikler = eksen.etiket_çentikleri();
            let en_geniş = çentikler
                .iter()
                .map(|(_, ç)| çizici.yazı_ölç(&etiket_metni(eksen, ç.değer), boyut_ön).0)
                .fold(0.0f32, f32::max);
            let bant = eksen.bant_genişliği().max(1.0);
            let gerekli = if eksen.yatay_mı() { en_geniş + 8.0 } else { boyut_ön * 1.6 };
            (gerekli / bant).ceil().max(1.0) as usize
        } else {
            1
        };

        // 2) Çentikler.
        if eksen.seçenek.çentik_görünür_mü() {
            let renk = tema::EKSEN_ÇENTİĞİ;
            let uzunluk = eksen.seçenek.çentik.uzunluk;
            for (i, konum) in eksen
                .çizgi_çentikleri(eksen.seçenek.çentik.etiketle_hizala)
                .into_iter()
                .enumerate()
            {
                if i % adım != 0 {
                    continue;
                }
                let konum = keskin(konum);
                if eksen.yatay_mı() {
                    çizici.çizgi(
                        (konum, sabit_keskin),
                        (konum, sabit_keskin + dış_yön * uzunluk),
                        1.0,
                        renk,
                        ÇizgiTürü::Düz,
                    );
                } else {
                    çizici.çizgi(
                        (sabit_keskin, konum),
                        (sabit_keskin + dış_yön * uzunluk, konum),
                        1.0,
                        renk,
                        ÇizgiTürü::Düz,
                    );
                }
            }
        }

        // 2b) Ara çentikler (`minorTick`).
        if eksen.seçenek.ara_çentik.göster {
            let renk = tema::EKSEN_ARA_ÇENTİĞİ;
            let uzunluk = eksen.seçenek.ara_çentik.uzunluk;
            for konum in eksen.ara_çentik_pikselleri(eksen.seçenek.ara_çentik.bölme_sayısı)
            {
                let konum = keskin(konum);
                if eksen.yatay_mı() {
                    çizici.çizgi(
                        (konum, sabit_keskin),
                        (konum, sabit_keskin + dış_yön * uzunluk),
                        1.0,
                        renk,
                        ÇizgiTürü::Düz,
                    );
                } else {
                    çizici.çizgi(
                        (sabit_keskin, konum),
                        (sabit_keskin + dış_yön * uzunluk, konum),
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
