//! Sütun serisi çizimi — `echarts/src/chart/bar/BarView.ts` karşılığı.
//! Genişlik/kaydırma hesabı [`crate::yerlesim::sutun`] portunu kullanır.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::grafik::pasta::zengin_etiketi_yaz;
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::seri::SütunSerisi;
use crate::model::stil::{
    Etiket, EtiketDöndürme, EtiketKonumu, YazıDikeyHizası, YazıYatayHizası
};
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::ondalık_kırp;
use crate::yerlesim::sutun::{SütunKonumu, SütunSerisiBilgisi, sütun_yerleşimi};
use crate::yerlesim::yigin::YığınAralığı;

/// Çizime giren bir sütun serisi.
pub struct SütunGirdisi<'s> {
    pub seri: &'s SütunSerisi,
    /// Serinin kendi değer eksenini koruyan koordinat çifti. Aynı kategori
    /// eksenindeki sütunlar farklı değer eksenlerine bağlı olsa da ortak bant
    /// yerleşimini paylaşır.
    pub kartezyen: Kartezyen2B,
    /// Seçeneklerdeki genel seri sırası (renk çözümü için).
    pub genel_sıra: usize,
    pub aralıklar: &'s [YığınAralığı],
    pub renk: Renk,
}

fn yığın_kimliği(seri: &SütunSerisi, genel_sıra: usize) -> String {
    match &seri.yığın {
        Some(ad) => format!("__yığın_{}_{}_{ad}", seri.eksen_bağı.x, seri.eksen_bağı.y),
        None => format!("__seri_{genel_sıra}"),
    }
}

/// zrender `Path.getInsideTextFill`: iç etiket rengi ana şeklin
/// parlaklığına göre `#333`, `#eee` ya da `#ccc` olur. Bu ayrım özellikle
/// ECharts paletindeki açık yeşil/turuncu sütunlarda okunabilirliği korur.
fn otomatik_iç_yazı_rengi(ana_renk: Renk) -> Renk {
    let parlaklık =
        (0.299 * ana_renk.kırmızı + 0.587 * ana_renk.yeşil + 0.114 * ana_renk.mavi) * ana_renk.alfa;
    if parlaklık > 0.5 {
        Renk::onaltılık(0x333333)
    } else if parlaklık > 0.2 {
        Renk::onaltılık(0xeeeeee)
    } else {
        Renk::onaltılık(0xcccccc)
    }
}

/// zrender `Rect` bağlı metin konumlarının piksel karşılığı. Dönen son
/// değer etiketin şeklin içinde olup olmadığını da bildirir; açık bir yazı
/// rengi yoksa ECharts iç etiketlerde beyaz, dış etiketlerde tema metnini
/// seçer.
fn sütun_etiket_yerleşimi(
    d: Dikdörtgen,
    etiket: &Etiket,
    yatay: bool,
    pozitif: bool,
) -> ((f32, f32), YatayHiza, DikeyHiza, bool) {
    let uzaklık = etiket.uzaklık;
    let konum = match etiket.konum {
        EtiketKonumu::Dış => {
            if yatay {
                if pozitif {
                    EtiketKonumu::Sağ
                } else {
                    EtiketKonumu::Sol
                }
            } else if pozitif {
                EtiketKonumu::Üst
            } else {
                EtiketKonumu::Alt
            }
        }
        diğer => diğer,
    };
    let orta_x = d.x + d.genişlik / 2.0;
    let orta_y = d.y + d.yükseklik / 2.0;
    match konum {
        EtiketKonumu::Üst => (
            (orta_x, d.y - uzaklık),
            YatayHiza::Orta,
            DikeyHiza::Alt,
            false,
        ),
        EtiketKonumu::Alt => (
            (orta_x, d.alt() + uzaklık),
            YatayHiza::Orta,
            DikeyHiza::Üst,
            false,
        ),
        EtiketKonumu::Sol => (
            (d.x - uzaklık, orta_y),
            YatayHiza::Sağ,
            DikeyHiza::Orta,
            false,
        ),
        EtiketKonumu::Sağ => (
            (d.sağ() + uzaklık, orta_y),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            false,
        ),
        EtiketKonumu::İç | EtiketKonumu::Merkez => {
            ((orta_x, orta_y), YatayHiza::Orta, DikeyHiza::Orta, true)
        }
        EtiketKonumu::İçÜst => (
            (orta_x, d.y + uzaklık),
            YatayHiza::Orta,
            DikeyHiza::Üst,
            true,
        ),
        EtiketKonumu::İçAlt => (
            (orta_x, d.alt() - uzaklık),
            YatayHiza::Orta,
            DikeyHiza::Alt,
            true,
        ),
        EtiketKonumu::İçSol => (
            (d.x + uzaklık, orta_y),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            true,
        ),
        EtiketKonumu::İçSağ => (
            (d.sağ() - uzaklık, orta_y),
            YatayHiza::Sağ,
            DikeyHiza::Orta,
            true,
        ),
        EtiketKonumu::İçSolÜst => (
            (d.x + uzaklık, d.y + uzaklık),
            YatayHiza::Sol,
            DikeyHiza::Üst,
            true,
        ),
        EtiketKonumu::İçSağÜst => (
            (d.sağ() - uzaklık, d.y + uzaklık),
            YatayHiza::Sağ,
            DikeyHiza::Üst,
            true,
        ),
        EtiketKonumu::İçSolAlt => (
            (d.x + uzaklık, d.alt() - uzaklık),
            YatayHiza::Sol,
            DikeyHiza::Alt,
            true,
        ),
        EtiketKonumu::İçSağAlt => (
            (d.sağ() - uzaklık, d.alt() - uzaklık),
            YatayHiza::Sağ,
            DikeyHiza::Alt,
            true,
        ),
        // `Dış` yukarıda yönlü yerleşime dönüştürülür; bu dal yalnız ileri
        // bir enum değişikliğinde güvenli merkez geri düşüşü olarak kalır.
        EtiketKonumu::Dış => ((orta_x, orta_y), YatayHiza::Orta, DikeyHiza::Orta, true),
    }
}

/// Görünür sütun serilerinin bant içi yerleşimini hesaplar.
pub fn yerleşim_hesapla(girdiler: &[SütunGirdisi], bant_genişliği: f32) -> Vec<SütunKonumu> {
    let bilgiler: Vec<SütunSerisiBilgisi> = girdiler
        .iter()
        .map(|g| SütunSerisiBilgisi {
            yığın_kimliği: yığın_kimliği(g.seri, g.genel_sıra),
            genişlik: g.seri.genişlik,
            en_çok_genişlik: g.seri.en_çok_genişlik,
            en_az_genişlik: g.seri.en_az_genişlik,
            sütun_boşluğu: g.seri.sütun_boşluğu,
            kategori_boşluğu: g.seri.kategori_boşluğu,
        })
        .collect();
    let düzen = sütun_yerleşimi(bant_genişliği, &bilgiler);
    girdiler
        .iter()
        .map(|g| {
            düzen
                .get(&yığın_kimliği(g.seri, g.genel_sıra))
                .copied()
                .unwrap_or(SütunKonumu {
                    kaydırma: 0.0,
                    genişlik: 0.0,
                })
        })
        .collect()
}

/// Tüm görünür sütun serilerini çizer. Kategori ekseni y ise sütunlar yatay
/// çizilir. Çizilen her sütun için `isabetler`e tıklama bölgesi eklenir.
pub fn sütunları_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    girdiler: &[SütunGirdisi],
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let Some(ilk_girdi) = girdiler.first() else {
        return;
    };
    let ilk_kartezyen = &ilk_girdi.kartezyen;
    let yatay = ilk_kartezyen.y.ölçek.kategorik_mi() && !ilk_kartezyen.x.ölçek.kategorik_mi();
    let bant_ekseni = if yatay {
        &ilk_kartezyen.y
    } else {
        &ilk_kartezyen.x
    };
    let konumlar = yerleşim_hesapla(girdiler, bant_ekseni.bant_genişliği());

    for (girdi, konum) in girdiler.iter().zip(&konumlar) {
        let seri = girdi.seri;
        let değer_ekseni = if yatay {
            &girdi.kartezyen.x
        } else {
            &girdi.kartezyen.y
        };
        for (i, aralık) in girdi.aralıklar.iter().enumerate() {
            let Some((taban, tepe)) = aralık else {
                continue;
            };
            let Some(veri_öğesi) = seri.veri.get(i) else {
                continue;
            };
            let bant_merkezi = bant_ekseni.veriden_piksele(i as f64);
            let kenar = bant_merkezi + konum.kaydırma;

            let taban_p = değer_ekseni.veriden_piksele(*taban);
            let tepe_p = değer_ekseni.veriden_piksele(*tepe);
            // Giriş animasyonu: sütun tabandan büyür.
            let tepe_p = taban_p + (tepe_p - taban_p) * ilerleme.clamp(0.0, 1.0);

            let d = if yatay {
                Dikdörtgen::yeni(
                    taban_p.min(tepe_p),
                    kenar,
                    (tepe_p - taban_p).abs(),
                    konum.genişlik,
                )
            } else {
                Dikdörtgen::yeni(
                    kenar,
                    taban_p.min(tepe_p),
                    konum.genişlik,
                    (tepe_p - taban_p).abs(),
                )
            };

            let öğe_stili = veri_öğesi.stil.as_ref();
            let dolgu = öğe_stili
                .and_then(|s| s.renk.clone())
                .or_else(|| seri.öğe_stili.renk.clone())
                .unwrap_or(Dolgu::Düz(girdi.renk));
            let yarıçap = öğe_stili
                .map(|s| s.kenarlık_yarıçapı)
                .unwrap_or(seri.öğe_stili.kenarlık_yarıçapı);
            let kenarlık = seri
                .öğe_stili
                .kenarlık_rengi
                .map(|r| (seri.öğe_stili.kenarlık_kalınlığı.max(1.0), r));
            let opaklık = seri.öğe_stili.opaklık.unwrap_or(1.0);

            if seri.arka_plan_göster {
                let arka = seri.arka_plan_stili.as_ref();
                let arka_d = if yatay {
                    Dikdörtgen::yeni(
                        girdi.kartezyen.alan.x,
                        kenar,
                        girdi.kartezyen.alan.genişlik,
                        konum.genişlik,
                    )
                } else {
                    Dikdörtgen::yeni(
                        kenar,
                        girdi.kartezyen.alan.y,
                        konum.genişlik,
                        girdi.kartezyen.alan.yükseklik,
                    )
                };
                let arka_dolgu = arka.and_then(|stil| stil.renk.clone()).unwrap_or_else(|| {
                    Dolgu::Düz(Renk::kyma(180.0 / 255.0, 180.0 / 255.0, 180.0 / 255.0, 0.2))
                });
                let arka_yarıçap = arka.map(|stil| stil.kenarlık_yarıçapı).unwrap_or([0.0; 4]);
                let arka_kenarlık = arka.and_then(|stil| {
                    stil.kenarlık_rengi
                        .filter(|_| stil.kenarlık_kalınlığı > 0.0)
                        .map(|renk| (stil.kenarlık_kalınlığı, renk))
                });
                let arka_opaklık = arka.and_then(|stil| stil.opaklık).unwrap_or(1.0);
                çizici.dikdörtgen(
                    arka_d,
                    &arka_dolgu.opaklık(arka_opaklık),
                    arka_yarıçap,
                    arka_kenarlık,
                );
            }

            if let Some(pik) = &seri.piktogram {
                // Resimli sütun: tabandan tepeye tekrarlanan semboller.
                let adım = pik.boyut + pik.aralık;
                let sembol_rengi = dolgu.temsilî().opaklık(opaklık);
                if yatay {
                    let sayı = ((d.genişlik / adım).floor() as usize).max(1);
                    let orta_y = d.y + d.yükseklik / 2.0;
                    for k in 0..sayı {
                        crate::grafik::sembol_çiz(
                            çizici,
                            &pik.sembol,
                            (d.x + adım * k as f32 + pik.boyut / 2.0, orta_y),
                            pik.boyut,
                            sembol_rengi,
                        );
                    }
                } else {
                    let sayı = ((d.yükseklik / adım).floor() as usize).max(1);
                    let orta_x = d.x + d.genişlik / 2.0;
                    for k in 0..sayı {
                        crate::grafik::sembol_çiz(
                            çizici,
                            &pik.sembol,
                            (orta_x, d.alt() - adım * k as f32 - pik.boyut / 2.0),
                            pik.boyut,
                            sembol_rengi,
                        );
                    }
                }
            } else {
                çizici.dikdörtgen(d, &dolgu.opaklık(opaklık), yarıçap, kenarlık);
            }

            isabetler.push(İsabetBölgesi {
                seri_sırası: girdi.genel_sıra,
                veri_sırası: i,
                seri_adı: seri.ad.clone(),
                ad: veri_öğesi.ad.clone(),
                değer: veri_öğesi.değer.sayı(),
                geometri: İsabetGeometrisi::Dikdörtgen(d),
            });

            // Değer etiketi.
            let öğe_etiketi = veri_öğesi
                .etiket
                .as_ref()
                .map(|yama| yama.uygula(&seri.etiket));
            let etiket = öğe_etiketi.as_ref().unwrap_or(&seri.etiket);
            if etiket.göster
                && let Some(değer) = veri_öğesi.değer.sayı()
            {
                let ham = ondalık_kırp(değer);
                let veri_adı = veri_öğesi
                    .ad
                    .clone()
                    .unwrap_or_else(|| bant_ekseni.ölçek.etiket(i as f64));
                let (metin, zengin_metin) = match &etiket.biçimleyici {
                    Some(b) => {
                        let seri_adı = seri.ad.as_deref().unwrap_or("");
                        (
                            b.uygula_bağlamla(değer, &ham, seri_adı, &veri_adı),
                            Some(b.uygula_bağlamla_zengin(değer, &ham, seri_adı, &veri_adı)),
                        )
                    }
                    None => (ondalık_kırp(değer), None),
                };
                let boyut = etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                let (nokta, doğal_yatay_hiza, doğal_dikey_hiza, içeride) =
                    sütun_etiket_yerleşimi(d, etiket, yatay, değer >= 0.0);
                let yatay_hiza = etiket
                    .yatay_hiza
                    .map(|hiza| match hiza {
                        YazıYatayHizası::Sol => YatayHiza::Sol,
                        YazıYatayHizası::Orta => YatayHiza::Orta,
                        YazıYatayHizası::Sağ => YatayHiza::Sağ,
                    })
                    .unwrap_or(doğal_yatay_hiza);
                let dikey_hiza = etiket
                    .dikey_hiza
                    .map(|hiza| match hiza {
                        YazıDikeyHizası::Üst => DikeyHiza::Üst,
                        YazıDikeyHizası::Orta => DikeyHiza::Orta,
                        YazıDikeyHizası::Alt => DikeyHiza::Alt,
                    })
                    .unwrap_or(doğal_dikey_hiza);
                let renk = etiket.yazı.renk.unwrap_or_else(|| {
                    if içeride {
                        otomatik_iç_yazı_rengi(dolgu.temsilî().opaklık(opaklık))
                    } else {
                        tema::birincil_metin()
                    }
                });
                match etiket.döndürme {
                    EtiketDöndürme::Derece(derece) if derece.abs() > f32::EPSILON => {
                        if let Some(zengin_metin) =
                            zengin_metin.as_deref().filter(|zengin| *zengin != metin)
                        {
                            // ECharts rich text her `{style|...}` parçasını
                            // ayrı tspan olarak rasterleştirir. Boş stil bile
                            // glif hinting başlangıcını sıfırladığından dönen
                            // etiketlerde tek birleşik dizeden farklıdır.
                            zengin_etiketi_yaz(
                                çizici,
                                zengin_metin,
                                etiket,
                                nokta,
                                yatay_hiza,
                                renk,
                                -derece.to_radians(),
                            );
                        } else {
                            çizici.dönüşümlü_yazı(
                                &metin,
                                (0.0, 0.0),
                                yatay_hiza,
                                dikey_hiza,
                                boyut,
                                renk,
                                etiket.yazı.kalın,
                                AfinMatris::ötele(nokta.0, nokta.1)
                                    .çarp(AfinMatris::döndür(-derece.to_radians())),
                            );
                        }
                    }
                    _ => {
                        çizici.yazı(
                            &metin,
                            nokta,
                            yatay_hiza,
                            dikey_hiza,
                            boyut,
                            renk,
                            etiket.yazı.kalın,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn iç_yazı_rengi_zrender_parlaklık_eşiklerini_izler() {
        assert_eq!(
            otomatik_iç_yazı_rengi(Renk::onaltılık(0x5070dd)),
            Renk::onaltılık(0xeeeeee)
        );
        assert_eq!(
            otomatik_iç_yazı_rengi(Renk::onaltılık(0xb6d634)),
            Renk::onaltılık(0x333333)
        );
        assert_eq!(
            otomatik_iç_yazı_rengi(Renk::onaltılık(0x111111)),
            Renk::onaltılık(0xcccccc)
        );
    }
}
