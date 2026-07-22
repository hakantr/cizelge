//! Tema nehri (`themeRiver`) — ECharts `ThemeRiverSeries`,
//! `themeRiverLayout` ve `ThemeRiverView` davranışlarının portu.
//!
//! Seri, gerçek `singleAxis` koordinatını kullanır. Her katmandaki eksik
//! eksen değerleri sıfırla tamamlanır; katmanlar ilk görülme sırasıyla
//! gruplanır ve ECharts'ın silhouette taban çizgisi üzerinde yığılır.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::cizgi::yumuşak_parça_ekle;
use crate::koordinat::{Dikdörtgen, TekEksenYerleşimi};
use crate::model::seri::TemaNehriSerisi;
use crate::model::stil::{
    Etiket, YazıDikeyHizası, YazıYatayHizası, zengin_metin_içeriği, ÖğeStili,
};
use crate::model::tek_eksen::TekEksenYönü;
use crate::renk::{Dolgu, Renk};
use crate::tema;

#[derive(Clone, Debug)]
struct Katman {
    ad: String,
    değerler: Vec<f64>,
    veri_sıraları: Vec<Option<usize>>,
}

#[derive(Clone, Debug)]
struct Bant {
    katman_sırası: usize,
    ad: String,
    değerler: Vec<f64>,
    veri_sıraları: Vec<Option<usize>>,
    ilk_kenar: Vec<(f32, f32)>,
    ikinci_kenar: Vec<(f32, f32)>,
}

/// Katman adlarını ECharts `groupData` gibi ilk görülme sırasıyla döndürür.
pub fn tema_nehri_katman_adları(seri: &TemaNehriSerisi) -> Vec<String> {
    let mut adlar = Vec::new();
    for (_, _, ad) in &seri.veri {
        if !adlar.contains(ad) {
            adlar.push(ad.clone());
        }
    }
    adlar
}

/// Katmanın `itemStyle.color` → `series.color` → global palet önceliğiyle
/// çözülen dolgusu.
pub fn tema_nehri_katman_dolgusu(
    seri: &TemaNehriSerisi,
    katman_sırası: usize,
    palet: &dyn Fn(usize) -> Renk,
) -> Dolgu {
    seri.öğe_stili
        .renk
        .clone()
        .or_else(|| {
            (!seri.renkler.is_empty())
                .then(|| seri.renkler[katman_sırası % seri.renkler.len()].clone())
        })
        .unwrap_or_else(|| Dolgu::Düz(palet(katman_sırası)))
}

fn katmanları_kur(
    seri: &TemaNehriSerisi,
    görünür: &dyn Fn(&str) -> bool,
) -> (Vec<f64>, Vec<Katman>) {
    let mut x_değerleri = seri
        .veri
        .iter()
        .filter_map(|(x, _, _)| x.is_finite().then_some(*x))
        .collect::<Vec<_>>();
    x_değerleri.sort_by(f64::total_cmp);
    x_değerleri.dedup_by(|a, b| a.total_cmp(b).is_eq());

    let katmanlar = tema_nehri_katman_adları(seri)
        .into_iter()
        .filter(|ad| görünür(ad))
        .map(|ad| {
            let mut değerler = Vec::with_capacity(x_değerleri.len());
            let mut veri_sıraları = Vec::with_capacity(x_değerleri.len());
            for x in &x_değerleri {
                let kayıt = seri
                    .veri
                    .iter()
                    .enumerate()
                    .find(|(_, (aday_x, _, aday_ad))| {
                        aday_x.total_cmp(x).is_eq() && aday_ad == &ad
                    });
                değerler.push(
                    kayıt
                        .map(|(_, (_, değer, _))| *değer)
                        .filter(|değer| değer.is_finite())
                        .unwrap_or(0.0),
                );
                veri_sıraları.push(kayıt.map(|(sıra, _)| sıra));
            }
            Katman {
                ad,
                değerler,
                veri_sıraları,
            }
        })
        .collect();
    (x_değerleri, katmanlar)
}

fn bantları_kur(
    seri: &TemaNehriSerisi,
    yerleşim: &TekEksenYerleşimi,
    görünür: &dyn Fn(&str) -> bool,
) -> Vec<Bant> {
    let (x_değerleri, katmanlar) = katmanları_kur(seri, görünür);
    if katmanlar.is_empty() || x_değerleri.is_empty() {
        return Vec::new();
    }

    let toplamlar = (0..x_değerleri.len())
        .map(|x_sırası| {
            katmanlar
                .iter()
                .map(|katman| katman.değerler[x_sırası])
                .sum::<f64>()
        })
        .collect::<Vec<_>>();
    let en_büyük_toplam = toplamlar
        .iter()
        .copied()
        .filter(|değer| değer.is_finite())
        .fold(0.0_f64, f64::max);
    if en_büyük_toplam <= f64::EPSILON {
        return Vec::new();
    }

    // ECharts `computeBaseline`: en yüksek toplam üst sınıra değerken diğer
    // kesitler kalan boşluğun yarısı kadar içeri alınır.
    let tabanlar = toplamlar
        .iter()
        .map(|toplam| (en_büyük_toplam - toplam) / 2.0)
        .collect::<Vec<_>>();
    let dik_boyut = match yerleşim.yön {
        TekEksenYönü::Yatay => yerleşim.alan.yükseklik,
        TekEksenYönü::Dikey => yerleşim.alan.genişlik,
    };
    let başlangıç_boşluğu = seri.sınır_boşluğu[0].çöz(dik_boyut).max(0.0);
    let bitiş_boşluğu = seri.sınır_boşluğu[1].çöz(dik_boyut).max(0.0);
    let kullanılabilir = (dik_boyut - başlangıç_boşluğu - bitiş_boşluğu).max(0.0);
    let değer_ölçeği = f64::from(kullanılabilir) / en_büyük_toplam;

    let mut bantlar = Vec::with_capacity(katmanlar.len());
    for (katman_sırası, katman) in katmanlar.iter().enumerate() {
        let mut ilk_kenar = Vec::with_capacity(x_değerleri.len());
        let mut ikinci_kenar = Vec::with_capacity(x_değerleri.len());
        for (x_sırası, x) in x_değerleri.iter().enumerate() {
            let önceki = katmanlar
                .iter()
                .take(katman_sırası)
                .map(|önceki| önceki.değerler[x_sırası])
                .sum::<f64>();
            let ilk = (tabanlar[x_sırası] + önceki) * değer_ölçeği;
            let ikinci = ilk + katman.değerler[x_sırası] * değer_ölçeği;
            let eksen_noktası = yerleşim.veriden_noktaya(*x);
            match yerleşim.yön {
                TekEksenYönü::Yatay => {
                    ilk_kenar.push((
                        eksen_noktası.0,
                        yerleşim.alan.y + başlangıç_boşluğu + ilk as f32,
                    ));
                    ikinci_kenar.push((
                        eksen_noktası.0,
                        yerleşim.alan.y + başlangıç_boşluğu + ikinci as f32,
                    ));
                }
                TekEksenYönü::Dikey => {
                    ilk_kenar.push((
                        yerleşim.alan.x + başlangıç_boşluğu + ilk as f32,
                        eksen_noktası.1,
                    ));
                    ikinci_kenar.push((
                        yerleşim.alan.x + başlangıç_boşluğu + ikinci as f32,
                        eksen_noktası.1,
                    ));
                }
            }
        }
        bantlar.push(Bant {
            katman_sırası,
            ad: katman.ad.clone(),
            değerler: katman.değerler.clone(),
            veri_sıraları: katman.veri_sıraları.clone(),
            ilk_kenar,
            ikinci_kenar,
        });
    }
    bantlar
}

fn bant_yolu(bant: &Bant) -> Yol {
    let mut yol = Yol::yeni();
    yumuşak_parça_ekle(&mut yol, &bant.ilk_kenar, 0.4, true);
    let mut ters = bant.ikinci_kenar.clone();
    ters.reverse();
    yumuşak_parça_ekle(&mut yol, &ters, 0.4, false);
    yol.kapat();
    yol
}

fn vurgu_stilini_uygula(taban: &ÖğeStili, yama: &ÖğeStili) -> ÖğeStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk = yama.renk.clone();
    }
    if yama.kenarlık_rengi.is_some() {
        sonuç.kenarlık_rengi = yama.kenarlık_rengi;
    }
    if yama.kenarlık_kalınlığı != 0.0 {
        sonuç.kenarlık_kalınlığı = yama.kenarlık_kalınlığı;
        sonuç.kenarlık_türü = yama.kenarlık_türü;
    }
    if yama.opaklık.is_some() {
        sonuç.opaklık = yama.opaklık;
    }
    if yama.gölge_bulanıklığı != 0.0 {
        sonuç.gölge_bulanıklığı = yama.gölge_bulanıklığı;
    }
    if yama.gölge_rengi.is_some() {
        sonuç.gölge_rengi = yama.gölge_rengi;
    }
    if yama.gölge_kayması != (0.0, 0.0) {
        sonuç.gölge_kayması = yama.gölge_kayması;
    }
    sonuç
}

fn yatay_hiza(etiket: &Etiket) -> YatayHiza {
    match etiket
        .yatay_hiza
        .or(etiket.yazı.yatay_hiza)
        .unwrap_or(YazıYatayHizası::Sol)
    {
        YazıYatayHizası::Sol => YatayHiza::Sol,
        YazıYatayHizası::Orta => YatayHiza::Orta,
        YazıYatayHizası::Sağ => YatayHiza::Sağ,
    }
}

fn dikey_hiza(etiket: &Etiket) -> DikeyHiza {
    match etiket
        .dikey_hiza
        .or(etiket.yazı.dikey_hiza)
        .unwrap_or(YazıDikeyHizası::Orta)
    {
        YazıDikeyHizası::Üst => DikeyHiza::Üst,
        YazıDikeyHizası::Orta => DikeyHiza::Orta,
        YazıDikeyHizası::Alt => DikeyHiza::Alt,
    }
}

fn isabet_çokgeni(bant: &Bant) -> Vec<(f32, f32)> {
    let mut noktalar = bant.ilk_kenar.clone();
    noktalar.extend(bant.ikinci_kenar.iter().rev().copied());
    noktalar
}

/// Tema nehri serisini bağlı `singleAxis` yerleşiminde çizer.
#[allow(clippy::too_many_arguments)]
pub fn tema_nehri_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &TemaNehriSerisi,
    genel_sıra: usize,
    yerleşim: &TekEksenYerleşimi,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    görünür: &dyn Fn(&str) -> bool,
    fare: Option<(f32, f32)>,
    programatik_veri: Option<usize>,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let bantlar = bantları_kur(seri, yerleşim, görünür);
    if bantlar.is_empty() {
        return;
    }

    let vurgulu_katman = programatik_veri
        .and_then(|veri_sırası| seri.veri.get(veri_sırası))
        .and_then(|(_, _, ad)| bantlar.iter().position(|bant| &bant.ad == ad))
        .or_else(|| {
            let fare = fare?;
            bantlar.iter().rposition(|bant| {
                İsabetGeometrisi::Çokgen {
                    noktalar: isabet_çokgeni(bant),
                }
                .içeriyor_mu(fare)
            })
        });

    let ilerleme = ilerleme.clamp(0.0, 1.0);
    let kırpma = match yerleşim.yön {
        TekEksenYönü::Yatay => Dikdörtgen::yeni(
            yerleşim.alan.x,
            yerleşim.alan.y,
            yerleşim.alan.genişlik * ilerleme,
            yerleşim.alan.yükseklik,
        ),
        TekEksenYönü::Dikey => {
            let yükseklik = yerleşim.alan.yükseklik * ilerleme;
            Dikdörtgen::yeni(
                yerleşim.alan.x,
                yerleşim.alan.alt() - yükseklik,
                yerleşim.alan.genişlik,
                yükseklik,
            )
        }
    };

    let mut boya = |yüzey: &mut dyn ÇizimYüzeyi| {
        for bant in &bantlar {
            let vurgulu = vurgulu_katman == Some(bant.katman_sırası);
            let stil = if vurgulu {
                vurgu_stilini_uygula(&seri.öğe_stili, &seri.vurgu_öğe_stili)
            } else {
                seri.öğe_stili.clone()
            };
            let yol = bant_yolu(bant);
            let taban_dolgu = tema_nehri_katman_dolgusu(seri, bant.katman_sırası, palet);
            let dolgu = stil
                .renk
                .clone()
                .unwrap_or(taban_dolgu)
                .opaklık(stil.opaklık.unwrap_or(1.0));
            if stil.gölge_bulanıklığı > 0.0
                && let Some(gölge) = stil.gölge_rengi
            {
                yüzey.yol_gölgesi(
                    &yol,
                    gölge.opaklık(stil.opaklık.unwrap_or(1.0)),
                    stil.gölge_bulanıklığı,
                    stil.gölge_kayması,
                );
            }
            yüzey.yol_doldur(&yol, &dolgu);
            if stil.kenarlık_kalınlığı > 0.0
                && let Some(kenarlık) = stil.kenarlık_rengi
            {
                yüzey.yol_çiz(&yol, stil.kenarlık_kalınlığı, kenarlık, stil.kenarlık_türü);
            }
        }

        // Zrender bağlı metinleri ana yolların bir üst z2 katmanında tutar;
        // bu nedenle sonraki katman dolguları önceki etiketleri örtmez.
        for bant in &bantlar {
            let vurgulu = vurgulu_katman == Some(bant.katman_sırası);
            let etiket = if vurgulu {
                seri.vurgu_etiketi.uygula(&seri.etiket)
            } else {
                seri.etiket.clone()
            };
            if etiket.göster
                && let (Some(&ilk), Some(&ikinci)) =
                    (bant.ilk_kenar.first(), bant.ikinci_kenar.first())
            {
                let değer = bant.değerler.first().copied().unwrap_or_default();
                let metin = etiket
                    .biçimleyici
                    .as_ref()
                    .map(|biçimleyici| biçimleyici.uygula(değer, &bant.ad))
                    .map(zengin_metin_içeriği)
                    .unwrap_or_else(|| bant.ad.clone());
                let konum = match yerleşim.yön {
                    // ThemeRiverView, `position: 'left'` öntanımlısına rağmen
                    // etiketi ilk layout noktasının 4 px soluna açıkça koyar.
                    TekEksenYönü::Yatay => (
                        ilk.0 - seri.etiket_boşluğu + etiket.kayma.0,
                        (ilk.1 + ikinci.1) / 2.0 + etiket.kayma.1,
                    ),
                    TekEksenYönü::Dikey => (
                        (ilk.0 + ikinci.0) / 2.0 + etiket.kayma.0,
                        ilk.1 + seri.etiket_boşluğu + etiket.kayma.1,
                    ),
                };
                let renk = etiket
                    .yazı
                    .renk
                    // Bağlı metin `position: null` ile yolun dışında sayılır:
                    // zrender auto outside-fill olarak #333 ve zemin renkli
                    // 2 px kontur seçer.
                    .unwrap_or_else(|| {
                        if tema::koyu_mu() {
                            Renk::onaltılık(0xeeeeee)
                        } else {
                            Renk::onaltılık(0x333333)
                        }
                    })
                    .opaklık(etiket.yazı.opaklık.unwrap_or(1.0));
                yüzey.dönüşümlü_konturlu_yazı(
                    &metin,
                    konum,
                    yatay_hiza(&etiket),
                    dikey_hiza(&etiket),
                    etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
                    renk,
                    etiket.yazı.kalın,
                    tema::nötr_00(),
                    2.0,
                    AfinMatris::BİRİM,
                );
            }
        }
    };
    if ilerleme < 1.0 {
        çizici.kırpılı(kırpma, &mut boya);
    } else {
        boya(çizici);
    }

    if seri.sessiz {
        return;
    }
    for bant in &bantlar {
        for x_sırası in 0..bant.değerler.len() {
            let Some(veri_sırası) = bant.veri_sıraları[x_sırası] else {
                continue;
            };
            let değer = bant.değerler[x_sırası];
            if değer == 0.0 {
                continue;
            }
            let ilk = bant.ilk_kenar[x_sırası];
            let ikinci = bant.ikinci_kenar[x_sırası];
            let önceki = x_sırası
                .checked_sub(1)
                .map(|sıra| bant.ilk_kenar[sıra])
                .unwrap_or(ilk);
            let sonraki = bant.ilk_kenar.get(x_sırası + 1).copied().unwrap_or(ilk);
            let geometri = match yerleşim.yön {
                TekEksenYönü::Yatay => {
                    let sol = if x_sırası == 0 {
                        ilk.0
                    } else {
                        (önceki.0 + ilk.0) / 2.0
                    };
                    let sağ = if x_sırası + 1 == bant.ilk_kenar.len() {
                        ilk.0
                    } else {
                        (ilk.0 + sonraki.0) / 2.0
                    };
                    İsabetGeometrisi::Dikdörtgen(Dikdörtgen::yeni(
                        sol.min(sağ),
                        ilk.1.min(ikinci.1),
                        (sağ - sol).abs().max(1.0),
                        (ikinci.1 - ilk.1).abs().max(1.0),
                    ))
                }
                TekEksenYönü::Dikey => {
                    let üst = if x_sırası == 0 {
                        ilk.1
                    } else {
                        (önceki.1 + ilk.1) / 2.0
                    };
                    let alt = if x_sırası + 1 == bant.ilk_kenar.len() {
                        ilk.1
                    } else {
                        (ilk.1 + sonraki.1) / 2.0
                    };
                    İsabetGeometrisi::Dikdörtgen(Dikdörtgen::yeni(
                        ilk.0.min(ikinci.0),
                        üst.min(alt),
                        (ikinci.0 - ilk.0).abs().max(1.0),
                        (alt - üst).abs().max(1.0),
                    ))
                }
            };
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası,
                seri_adı: seri.ad.clone(),
                ad: Some(bant.ad.clone()),
                değer: Some(değer),
                geometri,
            });
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod testler {
    use super::*;
    use crate::model::Uzunluk;
    use crate::model::tek_eksen::TekEksen;
    use crate::olcek::{AralıkÖlçeği, Ölçek};

    fn yerleşim() -> TekEksenYerleşimi {
        TekEksenYerleşimi::kur(
            &TekEksen::yeni(),
            (100.0, 100.0),
            Ölçek::Aralık(AralıkÖlçeği::kur(
                [0.0, 2.0],
                Some(0.0),
                Some(2.0),
                false,
                2,
                None,
                None,
            )),
        )
    }

    #[test]
    fn eksik_zamanlari_sifirla_tamamlar_ve_ilk_ad_sirasini_korur() {
        let seri =
            TemaNehriSerisi::yeni().veri([(0.0, 2.0, "B"), (0.0, 1.0, "A"), (1.0, 3.0, "A")]);
        let (x, katmanlar) = katmanları_kur(&seri, &|_| true);
        assert_eq!(x, vec![0.0, 1.0]);
        assert_eq!(katmanlar[0].ad, "B");
        assert_eq!(katmanlar[0].değerler, vec![2.0, 0.0]);
        assert_eq!(katmanlar[1].ad, "A");
        assert_eq!(katmanlar[1].değerler, vec![1.0, 3.0]);
    }

    #[test]
    fn resmi_siluet_tabanini_boundary_gap_icinde_kurar() {
        let seri = TemaNehriSerisi::yeni().veri([
            (0.0, 2.0, "A"),
            (1.0, 4.0, "A"),
            (0.0, 2.0, "B"),
            (1.0, 0.0, "B"),
        ]);
        let bantlar = bantları_kur(&seri, &yerleşim(), &|_| true);
        assert_eq!(bantlar.len(), 2);
        // 90 px singleAxis yüksekliğinin %10 + %10'u çıkar: 72 px.
        // Her iki kesitte en büyük toplam 4 olduğundan ky=18.
        assert!((bantlar[0].ilk_kenar[0].1 - 14.0).abs() < 1e-4);
        assert!((bantlar[0].ikinci_kenar[0].1 - 50.0).abs() < 1e-4);
    }

    #[test]
    fn resmi_model_ontanimlarini_tasir() {
        let seri = TemaNehriSerisi::yeni();
        assert_eq!(seri.tek_eksen_sırası, 0);
        assert_eq!(seri.sınır_boşluğu, [Uzunluk::Yüzde(10.0); 2]);
        assert!(seri.etiket.göster);
        assert_eq!(seri.etiket.yazı.boyut, Some(11.0));
        assert_eq!(seri.etiket_boşluğu, 4.0);
    }
}
