//! Eksen çizimi — `echarts/src/component/axis` (AxisBuilder) karşılığı:
//! bölme çizgileri, eksen çizgisi, çentikler ve etiketler.
//!
//! İnce (1 px) çizgiler [`keskin`] ile fiziksel piksel ızgarasına oturtulur;
//! eksen ve bölme çizgilerinin bulanıklaşması böyle önlenir.

use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, keskin, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, ÇalışmaEkseni};
use crate::model::eksen::EksenAdKonumu;
use crate::model::eksen::EksenKonumu;
use crate::model::eksen::{EksenSıfırKipi, EksenTürü};
use crate::model::stil::ÇizgiTürü;
use crate::tema;

// AxisBuilder'ın tspan'leri `textBaseline=middle` ile bağladığı eksen
// gruplarında Chromium'un Arial tabanı, genel metin kutusundan 0,2 px daha
// aşağıdadır. Denge yerel metin uzayında uygulanır; böylece döndürülmüş
// etiketlerde de dönüşümle birlikte doğru yönde taşınır.
const EKSEN_YAZI_TABAN_DENGESİ: f32 = 0.2;

/// Etiket metni: biçimleyici uygulanmış çentik etiketi.
fn etiket_metni(eksen: &ÇalışmaEkseni, değer: f64) -> String {
    let ham = eksen.ölçek.etiket(değer);
    match &eksen.seçenek.etiket.biçimleyici {
        Some(b) => b.uygula(değer, &ham),
        None => ham,
    }
}

/// Kategori ekseninin otomatik `axisLabel.interval` çizim adımı. Çizgi
/// serisinin `showAllSymbol: 'auto'` davranışı aynı görünür etiket kümesini
/// izlediğinden bu hesap AxisBuilder ile LineView arasında ortaktır.
pub(crate) fn kategori_etiket_adımı(
    çizici: &dyn ÇizimYüzeyi, eksen: &ÇalışmaEkseni
) -> usize {
    if !eksen.ölçek.kategorik_mi() {
        return 1;
    }
    if let Some(aralık) = eksen.seçenek.etiket.aralık {
        return aralık.saturating_add(1);
    }

    let boyut = eksen.seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let çentikler = eksen.etiket_çentikleri();
    // ECharts büyük ordinal pencerelerde en fazla yaklaşık 40 etiketi
    // ölçerek aralığı kestirir.
    let örnek_adımı = if çentikler.len() > 40 {
        (çentikler.len() / 40).max(1)
    } else {
        1
    };
    let en_geniş = çentikler
        .iter()
        .step_by(örnek_adımı)
        .map(|(_, ç)| çizici.yazı_ölç(&etiket_metni(eksen, ç.değer), boyut).0)
        .fold(0.0_f32, f32::max);
    // Sabit Arial yüzünün ab_glyph advance toplamı, Chromium Canvas
    // `measureText().width` sonucundan bu boyutta yaklaşık 0,06 px geniştir.
    // Çok sık ordinal eksenlerde bu küçük fark tam interval eşiğini bir
    // kategori aşabiliyor (grid-multiple: 155 yerine resmî 154 adım).
    let en_geniş = (en_geniş - 0.06).max(0.0);
    // `calculateCategoryInterval`, alt-piksel unitSpan'i korur ve zrender
    // metin sınırına 1.3 güvenlik çarpanı uygular.
    let bant = eksen.bant_genişliği().max(f32::EPSILON);
    let gerekli = if eksen.yatay_mı() {
        en_geniş * 1.3
    } else {
        boyut * 1.3
    };
    (gerekli / bant).floor().max(0.0) as usize + 1
}

/// ECharts `ordinalScaleCreateTicks`: yakınlaştırılmış kategori ekseninde
/// görünür etiketleri sıfıra göre sabit bir adıma hizalar. Böylece pencere
/// kayarken etiketler ve `showAllSymbol: 'auto'` sembolleri yer değiştirmez.
pub(crate) fn kategori_görünür_sıraları(
    eksen: &ÇalışmaEkseni,
    adım: usize,
) -> Option<Vec<usize>> {
    if !eksen.ölçek.kategorik_mi() {
        return None;
    }
    let çentikler = eksen.etiket_çentikleri();
    let ilk = çentikler.first()?.1.değer.round();
    let son = çentikler.last()?.1.değer.round();
    if ilk < 0.0 || son < ilk || son > usize::MAX as f64 {
        return Some(Vec::new());
    }
    let ilk = ilk as usize;
    let son = son as usize;
    let adım = adım.max(1);
    let çentik_sayısı = son.saturating_sub(ilk).saturating_add(1);
    let başlangıç = if ilk != 0 && adım > 1 && çentik_sayısı as f64 / adım as f64 > 2.0 {
        ilk.div_ceil(adım).saturating_mul(adım)
    } else {
        ilk
    };
    Some((başlangıç..=son).step_by(adım).collect())
}

/// ECharts `fixAxisOnZero` + `cartesianAxisHelper.layout` özeti. Eksen
/// çizgisi/tikleri sıfırda kesişebilir; etiketler ise ham dış kenarda kalır.
fn sıfırdaki_çizgi_konumu(
    eksen: &ÇalışmaEkseni,
    eksenler: &[&ÇalışmaEkseni],
    alan: Dikdörtgen,
) -> Option<f32> {
    if eksen.seçenek.çizgi.sıfır == EksenSıfırKipi::Kapalı {
        return None;
    }
    let dikler: Vec<&ÇalışmaEkseni> = eksenler
        .iter()
        .copied()
        .filter(|aday| aday.yatay_mı() != eksen.yatay_mı())
        .filter(|aday| matches!(aday.seçenek.tür, EksenTürü::Değer | EksenTürü::Log))
        .filter(|aday| {
            let kapsam = aday.ölçek.kapsam();
            kapsam[0] <= 0.0 && kapsam[1] >= 0.0
        })
        .collect();
    let hedef = match eksen.seçenek.çizgi.sıfır_eksen_sırası {
        Some(sıra) => dikler.get(sıra).copied(),
        None => dikler.first().copied(),
    }?;
    let konum = hedef.veriden_piksele(0.0);
    Some(if eksen.yatay_mı() {
        konum.clamp(alan.y, alan.alt())
    } else {
        konum.clamp(alan.x, alan.sağ())
    })
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
    let tema_bantları = tema::bölme_alanı_renkleri().to_vec();
    for eksen in eksenler {
        if !eksen.seçenek.bölme_alanı.göster {
            continue;
        }
        let renkler = if eksen.seçenek.bölme_alanı.renkler.is_empty() {
            &tema_bantları
        } else {
            &eksen.seçenek.bölme_alanı.renkler
        };
        let konumlar = eksen.çizgi_çentikleri(false);
        for (i, çift) in konumlar.windows(2).enumerate() {
            let [a, b] = çift else { continue };
            let Some(renk) = renkler.get(i % renkler.len()) else {
                continue;
            };
            let d = if eksen.yatay_mı() {
                crate::koordinat::Dikdörtgen::yeni(
                    a.min(*b),
                    alan.y,
                    (b - a).abs(),
                    alan.yükseklik,
                )
            } else {
                crate::koordinat::Dikdörtgen::yeni(alan.x, a.min(*b), alan.genişlik, (b - a).abs())
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
            .unwrap_or(tema::ara_bölme_çizgisi());
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
        let renk = eksen
            .seçenek
            .bölme_çizgisi
            .renk
            .unwrap_or(tema::bölme_çizgisi());
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
    çizici: &mut dyn ÇizimYüzeyi, alan: Dikdörtgen, eksenler: &[&ÇalışmaEkseni]
) {
    for eksen in eksenler {
        // ECharts `axisLine.show: 'auto'`: eksen çizgisi yalnız dikindeki
        // eksen interval (değer) ya da log ölçeğiyse otomatik görünür.
        // Time ölçeği bu kararda kategori ölçeği gibi davranır.
        let dik_değer_veya_log_var = eksenler.iter().any(|diğer| {
            diğer.yatay_mı() != eksen.yatay_mı()
                && matches!(diğer.seçenek.tür, EksenTürü::Değer | EksenTürü::Log)
        });
        // `createCartesianAxisViewCommonPartBuilder`: axisLine/axisTick
        // yalnız dik eksen interval veya log ölçeğiyse otomatik görünür.
        // Time ölçeği kategori gibi davranır; time×value grafiğinde x
        // çizgisi görünürken y çizgisi gizlenir.
        let otomatik_çizgi = dik_değer_veya_log_var;
        // `axisTick.show: 'auto'`, dik eksen sürekli değilse kapanır. Ayrıca
        // kategori ekseni bantlıysa (`boundaryGap: true`) tikler kategori
        // merkezlerine değil bant sınırlarına düşeceğinden ECharts tarafından
        // otomatik olarak gizlenir.
        let bantlı_kategori =
            eksen.seçenek.tür == EksenTürü::Kategori && eksen.seçenek.kenar_boşluğu.unwrap_or(true);
        let otomatik_çentik = dik_değer_veya_log_var && !bantlı_kategori;
        let çizgi_göster = eksen.seçenek.çizgi.göster.unwrap_or(otomatik_çizgi);
        let çentik_göster = eksen.seçenek.çentik.göster.unwrap_or(otomatik_çentik);

        // Eksenin sabit (dik) konumu.
        let sabit = match eksen.konum {
            EksenKonumu::Alt => alan.alt() + eksen.seçenek.kaydırma,
            EksenKonumu::Üst => alan.y - eksen.seçenek.kaydırma,
            EksenKonumu::Sol => alan.x - eksen.seçenek.kaydırma,
            EksenKonumu::Sağ => alan.sağ() + eksen.seçenek.kaydırma,
        };
        let çizgi_sabiti = sıfırdaki_çizgi_konumu(eksen, eksenler, alan).unwrap_or(sabit);
        let dış_yön: f32 = match eksen.konum {
            EksenKonumu::Alt | EksenKonumu::Sağ => 1.0,
            EksenKonumu::Üst | EksenKonumu::Sol => -1.0,
        };
        let çizgi_sabiti_keskin = keskin(çizgi_sabiti);

        // 1) Eksen çizgisi.
        if çizgi_göster {
            let renk = eksen.seçenek.çizgi.renk.unwrap_or(tema::eksen_çizgisi());
            let kalınlık = eksen.seçenek.çizgi.kalınlık;
            let konum = if kalınlık <= 1.5 {
                çizgi_sabiti_keskin
            } else {
                çizgi_sabiti
            };
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
        let adım = kategori_etiket_adımı(çizici, eksen);
        let görünür_kategori_sıraları = kategori_görünür_sıraları(eksen, adım);

        // 2) Çentikler.
        if çentik_göster {
            let renk = tema::eksen_çentiği();
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
                        (konum, çizgi_sabiti_keskin),
                        (konum, çizgi_sabiti_keskin + dış_yön * uzunluk),
                        1.0,
                        renk,
                        ÇizgiTürü::Düz,
                    );
                } else {
                    çizici.çizgi(
                        (çizgi_sabiti_keskin, konum),
                        (çizgi_sabiti_keskin + dış_yön * uzunluk, konum),
                        1.0,
                        renk,
                        ÇizgiTürü::Düz,
                    );
                }
            }
        }

        // 2b) Ara çentikler (`minorTick`).
        if eksen.seçenek.ara_çentik.göster {
            let renk = tema::eksen_ara_çentiği();
            let uzunluk = eksen.seçenek.ara_çentik.uzunluk;
            for konum in eksen.ara_çentik_pikselleri(eksen.seçenek.ara_çentik.bölme_sayısı) {
                let konum = keskin(konum);
                if eksen.yatay_mı() {
                    çizici.çizgi(
                        (konum, çizgi_sabiti_keskin),
                        (konum, çizgi_sabiti_keskin + dış_yön * uzunluk),
                        1.0,
                        renk,
                        ÇizgiTürü::Düz,
                    );
                } else {
                    çizici.çizgi(
                        (çizgi_sabiti_keskin, konum),
                        (çizgi_sabiti_keskin + dış_yön * uzunluk, konum),
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
            let renk = eksen
                .seçenek
                .etiket
                .yazı
                .renk
                .unwrap_or(tema::eksen_etiketi());
            let boşluk = eksen.seçenek.etiket.boşluk;
            let çentikler = eksen.etiket_çentikleri();

            for (i, (konum, çentik)) in çentikler.iter().enumerate() {
                let kategori_görünür = görünür_kategori_sıraları.as_ref().is_none_or(|sıralar| {
                    let değer = çentik.değer.round();
                    değer >= 0.0
                        && değer <= usize::MAX as f64
                        && sıralar.binary_search(&(değer as usize)).is_ok()
                });
                if !kategori_görünür || (görünür_kategori_sıraları.is_none() && i % adım != 0)
                {
                    continue;
                }
                let metin = etiket_metni(eksen, çentik.değer);
                if eksen.yatay_mı() {
                    let çapa = (*konum, sabit + dış_yön * boşluk);
                    let derece = eksen.seçenek.etiket.döndürme;
                    if derece.abs() <= f32::EPSILON {
                        let dikey = if dış_yön > 0.0 {
                            DikeyHiza::Üst
                        } else {
                            DikeyHiza::Alt
                        };
                        çizici.yazı(
                            &metin,
                            (çapa.0, çapa.1 + EKSEN_YAZI_TABAN_DENGESİ),
                            YatayHiza::Orta,
                            dikey,
                            boyut,
                            renk,
                            false,
                        );
                    } else {
                        // AxisBuilder pozitif `rotate` değerini Canvas'ta
                        // saat yönünün tersine uygular. Alt eksende pozitif
                        // metin sağdan, negatif metin soldan çapa alır.
                        let yatay = if derece > 0.0 {
                            YatayHiza::Sağ
                        } else {
                            YatayHiza::Sol
                        };
                        çizici.dönüşümlü_yazı(
                            &metin,
                            (0.0, EKSEN_YAZI_TABAN_DENGESİ),
                            yatay,
                            DikeyHiza::Orta,
                            boyut,
                            renk,
                            false,
                            AfinMatris::ötele(çapa.0, çapa.1)
                                .çarp(AfinMatris::döndür(-derece.to_radians())),
                        );
                    }
                } else {
                    let yatay = if dış_yön > 0.0 {
                        YatayHiza::Sol
                    } else {
                        YatayHiza::Sağ
                    };
                    çizici.yazı(
                        &metin,
                        (sabit + dış_yön * boşluk, *konum + EKSEN_YAZI_TABAN_DENGESİ),
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
            // AxisBuilder başlangıç/bitişi fiziksel tuval kenarına değil,
            // eksenin veri yönüne göre yorumlar. `inverse: true` bu yönü
            // çevirdiğinden ad çapaları da yer değiştirir.
            let ad_konumu = if eksen.seçenek.ters {
                match eksen.seçenek.ad_konumu {
                    EksenAdKonumu::Başlangıç => EksenAdKonumu::Bitiş,
                    EksenAdKonumu::Orta => EksenAdKonumu::Orta,
                    EksenAdKonumu::Bitiş => EksenAdKonumu::Başlangıç,
                }
            } else {
                eksen.seçenek.ad_konumu
            };
            if eksen.yatay_mı() {
                let (çapa, yatay, dikey) = match ad_konumu {
                    EksenAdKonumu::Başlangıç => (
                        (alan.x - eksen.seçenek.ad_boşluğu, sabit),
                        YatayHiza::Sağ,
                        DikeyHiza::Orta,
                    ),
                    EksenAdKonumu::Orta => (
                        (
                            alan.x + alan.genişlik / 2.0,
                            sabit + dış_yön * eksen.seçenek.ad_boşluğu,
                        ),
                        YatayHiza::Orta,
                        if dış_yön > 0.0 {
                            DikeyHiza::Üst
                        } else {
                            DikeyHiza::Alt
                        },
                    ),
                    EksenAdKonumu::Bitiş => (
                        (alan.sağ() + eksen.seçenek.ad_boşluğu, sabit),
                        YatayHiza::Sol,
                        DikeyHiza::Orta,
                    ),
                };
                çizici.yazı(
                    ad,
                    (çapa.0, çapa.1 + EKSEN_YAZI_TABAN_DENGESİ),
                    yatay,
                    dikey,
                    boyut,
                    tema::eksen_etiketi(),
                    false,
                );
            } else {
                let (çapa, dikey) = match ad_konumu {
                    EksenAdKonumu::Başlangıç => (
                        (sabit, alan.alt() + eksen.seçenek.ad_boşluğu),
                        DikeyHiza::Üst,
                    ),
                    EksenAdKonumu::Orta => (
                        (
                            sabit + dış_yön * eksen.seçenek.ad_boşluğu,
                            alan.y + alan.yükseklik / 2.0,
                        ),
                        DikeyHiza::Orta,
                    ),
                    EksenAdKonumu::Bitiş => {
                        ((sabit, alan.y - eksen.seçenek.ad_boşluğu), DikeyHiza::Alt)
                    }
                };
                çizici.yazı(
                    ad,
                    (çapa.0, çapa.1 + EKSEN_YAZI_TABAN_DENGESİ),
                    YatayHiza::Orta,
                    dikey,
                    boyut,
                    tema::eksen_etiketi(),
                    false,
                );
            }
        }
    }
}
