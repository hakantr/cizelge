//! ECharts `component/axis/ParallelAxisView.ts` çizimi.

use crate::bilesen::eksen_cizimi::eksenleri_çiz;
use crate::cizim::olay::ParalelEksenBölgesi;
use crate::cizim::ÇizimYüzeyi;
use crate::koordinat::{Dikdörtgen, ParalelYerleşimi};
use crate::model::paralel::ParalelYerleşim;
use crate::renk::Dolgu;
use crate::tema;

/// Eksen çizgileri/etiketleri ile var olan `activeIntervals` örtülerini
/// seri çizgilerinin üstünde boyar ve doğrusal seçim hedeflerini döndürür.
pub fn paralel_eksenlerini_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    yerleşim: &ParalelYerleşimi,
) -> Vec<ParalelEksenBölgesi> {
    let mut bölgeler = Vec::with_capacity(yerleşim.eksenler.len());
    for eksen in &yerleşim.eksenler {
        let dikey = yerleşim.yön == ParalelYerleşim::Yatay;
        let alan = if dikey {
            Dikdörtgen::yeni(eksen.konum, yerleşim.alan.y, 0.0, yerleşim.alan.yükseklik)
        } else {
            Dikdörtgen::yeni(yerleşim.alan.x, eksen.konum, yerleşim.alan.genişlik, 0.0)
        };

        // BrushController kaplamaları AxisBuilder grubuyla aynı yüksek
        // katmandadır; örtüyü önce çizmek çizgi/çentiklerin seçimin üstünde
        // okunur kalmasını sağlar.
        for aralık in &eksen.etkin_aralıklar {
            let ilk = eksen.eksen.veriden_piksele(aralık[0]);
            let ikinci = eksen.eksen.veriden_piksele(aralık[1]);
            let baş = ilk.min(ikinci);
            let uzunluk = (ikinci - ilk).abs();
            let genişlik = eksen.alan_seçim_stili.genişlik;
            let seçim = if dikey {
                Dikdörtgen::yeni(eksen.konum - genişlik / 2.0, baş, genişlik, uzunluk)
            } else {
                Dikdörtgen::yeni(baş, eksen.konum - genişlik / 2.0, uzunluk, genişlik)
            };
            çizici.dikdörtgen(
                seçim,
                &Dolgu::Düz(
                    eksen
                        .alan_seçim_stili
                        .renk
                        .opaklık(eksen.alan_seçim_stili.opaklık),
                ),
                [0.0; 4],
                (eksen.alan_seçim_stili.kenarlık_kalınlığı > 0.0).then_some((
                    eksen.alan_seçim_stili.kenarlık_kalınlığı,
                    eksen.alan_seçim_stili.kenarlık_rengi,
                )),
            );
        }

        let mut çalışma = eksen.eksen.clone();
        if let Some(en_çok) = eksen.ad_kısaltma_genişliği
            && let Some(ad) = çalışma.seçenek.ad.as_deref()
        {
            let boyut = çalışma.seçenek.ad_yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            çalışma.seçenek.ad = Some(metni_kısalt(çizici, ad, boyut, en_çok));
        }
        eksenleri_çiz(çizici, alan, &[&çalışma]);

        if çalışma.seçenek.göster {
            let eksen_uzunluğu = if dikey {
                yerleşim.alan.yükseklik
            } else {
                yerleşim.alan.genişlik
            };
            let ek = (eksen_uzunluğu.abs() * 0.1).min(30.0);
            let genişlik = eksen.alan_seçim_stili.genişlik.max(1.0);
            let şerit = if dikey {
                Dikdörtgen::yeni(
                    eksen.konum - genişlik / 2.0,
                    yerleşim.alan.y - ek,
                    genişlik,
                    yerleşim.alan.yükseklik + ek * 2.0,
                )
            } else {
                Dikdörtgen::yeni(
                    yerleşim.alan.x - ek,
                    eksen.konum - genişlik / 2.0,
                    yerleşim.alan.genişlik + ek * 2.0,
                    genişlik,
                )
            };
            bölgeler.push(ParalelEksenBölgesi {
                paralel_sırası: yerleşim.bileşen_sırası,
                eksen_bileşen_sırası: eksen.bileşen_sırası,
                boyut: eksen.ana_boyut(),
                şerit,
                dikey,
                gerçek_zamanlı: eksen.gerçek_zamanlı,
                eksen: çalışma,
            });
        }
    }
    bölgeler
}

fn metni_kısalt(çizici: &dyn ÇizimYüzeyi, metin: &str, boyut: f32, en_çok: f32) -> String {
    if en_çok <= 0.0 || çizici.yazı_ölç(metin, boyut).0 <= en_çok {
        return metin.to_owned();
    }
    let üç_nokta = "...";
    let üç_nokta_genişliği = çizici.yazı_ölç(üç_nokta, boyut).0;
    if üç_nokta_genişliği > en_çok {
        return String::new();
    }
    let mut sonuç = String::new();
    for karakter in metin.chars() {
        let mut aday = sonuç.clone();
        aday.push(karakter);
        if çizici.yazı_ölç(&aday, boyut).0 + üç_nokta_genişliği > en_çok {
            break;
        }
        sonuç = aday;
    }
    sonuç.push_str(üç_nokta);
    sonuç
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::koordinat::ParalelYerleşimi;
    use crate::model::paralel::{ParalelEkseni, ParalelKoordinatı};
    use crate::model::seri::ParalelSerisi;

    #[test]
    fn eksen_secim_ortusunu_ve_hit_seridini_uretir() {
        let seri = ParalelSerisi::yeni()
            .boyutlar(["A", "B"])
            .veri([vec![10.0, 20.0], vec![30.0, 40.0]]);
        let yerleşim = ParalelYerleşimi::kur(
            &ParalelKoordinatı::yeni(),
            0,
            &[
                ParalelEkseni::yeni(0).etkin_aralık(10.0, 20.0),
                ParalelEkseni::yeni(1),
            ],
            &[&seri],
            (600.0, 450.0),
        )
        .unwrap();
        let mut yüzey = KayıtYüzeyi::yeni(600.0, 450.0);
        let bölgeler = paralel_eksenlerini_çiz(&mut yüzey, &yerleşim);
        assert_eq!(bölgeler.len(), 2);
        assert!(bölgeler[0].dikey);
        assert!(bölgeler[0].şerit.genişlik >= 20.0);
        assert!(!yüzey.komutlar.is_empty());
    }
}
