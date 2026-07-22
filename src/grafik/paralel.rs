//! ECharts `chart/parallel/ParallelView.ts` seri çizimi.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::{ParalelEtkinlik, ParalelYerleşimi};
use crate::model::deger::{VeriDeğeri, VeriÖğesi};
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::seri::ParalelSerisi;
use crate::model::stil::{EtiketKonumu, YazıDikeyHizası, YazıYatayHizası};
use crate::model::veri_kumesi::BoyutSeçici;
use crate::renk::Renk;
use crate::tema;

#[derive(Clone, Debug)]
struct ParalelÇizgisi {
    veri_sırası: usize,
    noktalar: Vec<(f32, f32)>,
}

/// zrender `graphic/helper/smoothBezier` ile `Polyline.buildPath` portu.
///
/// Parallel serisi, Kartezyen `LineView`in uç aşımı sınırlı eğrisini değil,
/// zrender'ın açık çoklu-çizgi kontrol noktalarını kullanır. Uç noktaların
/// tutamaçları kendi üstündedir; iç noktaların iki tutamacı da komşu parça
/// uzunlukları oranında ölçeklenir.
fn paralel_yolu(noktalar: &[(f32, f32)], yumuşaklık: f32) -> Yol {
    let mut yol = Yol::yeni();
    let Some(&ilk) = noktalar.first() else {
        return yol;
    };
    yol.taşı(ilk);
    if noktalar.len() == 1 {
        return yol;
    }
    if yumuşaklık == 0.0 || !yumuşaklık.is_finite() {
        for &nokta in noktalar.iter().skip(1) {
            yol.çiz(nokta);
        }
        return yol;
    }

    let mut kontroller = Vec::with_capacity(noktalar.len() * 2 - 2);
    for (sıra, &nokta) in noktalar.iter().enumerate() {
        if sıra == 0 || sıra + 1 == noktalar.len() {
            kontroller.push(nokta);
            continue;
        }
        let önceki = noktalar[sıra - 1];
        let sonraki = noktalar[sıra + 1];
        let vektör = (
            (sonraki.0 - önceki.0) * yumuşaklık,
            (sonraki.1 - önceki.1) * yumuşaklık,
        );
        let önceki_uzaklık = ((nokta.0 - önceki.0).powi(2) + (nokta.1 - önceki.1).powi(2)).sqrt();
        let sonraki_uzaklık =
            ((nokta.0 - sonraki.0).powi(2) + (nokta.1 - sonraki.1).powi(2)).sqrt();
        let toplam = önceki_uzaklık + sonraki_uzaklık;
        let (önceki_oranı, sonraki_oranı) = if toplam != 0.0 {
            (önceki_uzaklık / toplam, sonraki_uzaklık / toplam)
        } else {
            (önceki_uzaklık, sonraki_uzaklık)
        };
        kontroller.push((
            nokta.0 - vektör.0 * önceki_oranı,
            nokta.1 - vektör.1 * önceki_oranı,
        ));
        kontroller.push((
            nokta.0 + vektör.0 * sonraki_oranı,
            nokta.1 + vektör.1 * sonraki_oranı,
        ));
    }

    for sıra in 0..noktalar.len() - 1 {
        yol.kübik(
            kontroller[sıra * 2],
            kontroller[sıra * 2 + 1],
            noktalar[sıra + 1],
        );
    }
    yol
}

fn paralel_çizgileri_boya(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &ParalelSerisi,
    çizgiler: &[ParalelÇizgisi],
    renkler: &[Renk],
    vurgulu: Option<usize>,
) {
    for (çizgi, &renk) in çizgiler.iter().zip(renkler) {
        let Some(öğe) = seri.veri.get(çizgi.veri_sırası) else {
            continue;
        };
        let durum_stili = if vurgulu == Some(çizgi.veri_sırası) {
            seri.vurgu_çizgi_stili.as_ref()
        } else if öğe.seçili {
            seri.seçili_çizgi_stili.as_ref()
        } else {
            None
        };
        let durum_etiketi = if vurgulu == Some(çizgi.veri_sırası) {
            seri.vurgu_etiketi.as_ref()
        } else if öğe.seçili {
            seri.seçili_etiketi.as_ref()
        } else {
            None
        };
        let stil = durum_stili.unwrap_or(&seri.çizgi_stili);
        let yol = paralel_yolu(&çizgi.noktalar, seri.yumuşaklık);
        if let Some(gölge_rengi) = stil.gölge_rengi
            && (stil.gölge_bulanıklığı > 0.0 || stil.gölge_kayması != (0.0, 0.0))
        {
            yüzey.yol_çizgi_gölgesi(
                &yol,
                stil.kalınlık.max(0.0),
                stil.tür,
                gölge_rengi.opaklık(renk.alfa),
                stil.gölge_bulanıklığı,
                stil.gölge_kayması,
            );
        }
        yüzey.yol_çiz(&yol, stil.kalınlık.max(0.0), renk, stil.tür);
        paralel_etiketini_çiz(yüzey, seri, öğe, çizgi, renk, durum_etiketi);
    }
}

/// Çizim sonucu bulunan vurgulu veri sırası. Çağıran bunu tooltip içeriği
/// ve programatik `showTip` hattı için kullanır.
#[allow(clippy::too_many_arguments)]
pub fn paralel_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &ParalelSerisi,
    genel_sıra: usize,
    yerleşim: &ParalelYerleşimi,
    seri_rengi: Renk,
    görsel_eşlemeler: &[&GörselEşleme],
    ilerleme: f32,
    fare: Option<(f32, f32)>,
    programatik_vurgu: Option<usize>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> Option<usize> {
    if yerleşim.eksenler.len() < 2 {
        return None;
    }
    let çizgiler = seri
        .veri
        .iter()
        .enumerate()
        .filter_map(|(veri_sırası, öğe)| {
            let noktalar = yerleşim
                .eksenler
                .iter()
                .filter_map(|eksen| yerleşim.veriden_noktaya(öğe, eksen))
                .collect::<Vec<_>>();
            (noktalar.len() >= 2).then_some(ParalelÇizgisi {
                veri_sırası,
                noktalar,
            })
        })
        .collect::<Vec<_>>();

    let vurgulu = programatik_vurgu
        .filter(|sıra| *sıra < seri.veri.len())
        .or_else(|| {
            (!seri.sessiz).then_some(fare).flatten().and_then(|fare| {
                çizgiler.iter().rev().find_map(|çizgi| {
                    İsabetGeometrisi::ÇokluÇizgi {
                        noktalar: çizgi.noktalar.clone(),
                        tolerans: seri.çizgi_stili.kalınlık.max(2.0) / 2.0 + 4.0,
                    }
                    .içeriyor_mu(fare)
                    .then_some(çizgi.veri_sırası)
                })
            })
        });
    let kapsamlar = görsel_eşlemeler
        .iter()
        .map(|eşleme| paralel_görsel_kapsamı(seri, eşleme))
        .collect::<Vec<_>>();
    let renkler = çizgiler
        .iter()
        .map(|çizgi| {
            let öğe = &seri.veri[çizgi.veri_sırası];
            let durum_stili = if vurgulu == Some(çizgi.veri_sırası) {
                seri.vurgu_çizgi_stili.as_ref()
            } else if öğe.seçili {
                seri.seçili_çizgi_stili.as_ref()
            } else {
                None
            };
            let stil = durum_stili.unwrap_or(&seri.çizgi_stili);
            let mut renk = stil.renk.unwrap_or(seri_rengi);
            if let Some(öğe_rengi) = öğe
                .stil
                .as_ref()
                .and_then(|stil| stil.renk.as_ref())
                .map(|dolgu| dolgu.temsilî())
            {
                renk = öğe_rengi;
            }
            for (eşleme, kapsam) in görsel_eşlemeler.iter().zip(&kapsamlar) {
                if let Some(ham) = paralel_görsel_ham_değeri(seri, öğe, eşleme) {
                    if eşleme.kategorik_mi() {
                        renk = eşleme.kategori_rengi_uygula(&ham, renk);
                    } else if let Some(değer) = ham.sayı().filter(|değer| değer.is_finite()) {
                        renk = eşleme.rengi_uygula(değer, *kapsam, renk);
                    }
                }
            }
            let etkinlik_opaklığı = match yerleşim.veri_etkinliği(öğe) {
                ParalelEtkinlik::Normal => stil.opaklık,
                ParalelEtkinlik::Etkin => seri.aktif_opaklık,
                ParalelEtkinlik::EtkinDeğil => seri.etkin_değil_opaklık,
            };
            let öğe_opaklığı = öğe
                .stil
                .as_ref()
                .and_then(|stil| stil.opaklık)
                .unwrap_or(1.0);
            renk.opaklık(
                etkinlik_opaklığı.clamp(0.0, 1.0)
                    * öğe_opaklığı.clamp(0.0, 1.0)
                    * ilerleme.clamp(0.0, 1.0),
            )
        })
        .collect::<Vec<_>>();

    // İlk giriş animasyonu ECharts createGridClipShape gibi koordinat
    // kutusunu yerleşim yönünde açar; bütün noktalar son geometrisinde kalır.
    let oran = ilerleme.clamp(0.0, 1.0);
    let kırpma = match yerleşim.yön {
        crate::model::paralel::ParalelYerleşim::Yatay => crate::koordinat::Dikdörtgen::yeni(
            yerleşim.alan.x,
            yerleşim.alan.y,
            yerleşim.alan.genişlik * oran,
            yerleşim.alan.yükseklik,
        ),
        crate::model::paralel::ParalelYerleşim::Dikey => crate::koordinat::Dikdörtgen::yeni(
            yerleşim.alan.x,
            yerleşim.alan.y,
            yerleşim.alan.genişlik,
            yerleşim.alan.yükseklik * oran,
        ),
    };
    let mut gövde = |yüzey: &mut dyn ÇizimYüzeyi| {
        paralel_çizgileri_boya(yüzey, seri, &çizgiler, &renkler, vurgulu);
    };
    çizici.kırpılı(kırpma, &mut gövde);

    if seri.sessiz {
        return vurgulu;
    }
    for çizgi in çizgiler {
        let Some(öğe) = seri.veri.get(çizgi.veri_sırası) else {
            continue;
        };
        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: çizgi.veri_sırası,
            seri_adı: seri.ad.clone(),
            ad: öğe.ad.clone(),
            değer: yerleşim
                .eksenler
                .iter()
                .find_map(|eksen| yerleşim.eksen_değeri(öğe, eksen)),
            geometri: İsabetGeometrisi::ÇokluÇizgi {
                noktalar: çizgi.noktalar,
                tolerans: seri.çizgi_stili.kalınlık.max(2.0) / 2.0 + 4.0,
            },
        });
    }
    vurgulu
}

fn paralel_etiketini_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &ParalelSerisi,
    öğe: &VeriÖğesi,
    çizgi: &ParalelÇizgisi,
    çizgi_rengi: Renk,
    durum: Option<&crate::model::stil::EtiketYaması>,
) {
    let mut çözülmüş = öğe
        .etiket
        .as_ref()
        .map(|yama| yama.uygula(&seri.etiket))
        .unwrap_or_else(|| seri.etiket.clone());
    if let Some(durum) = durum {
        çözülmüş = durum.uygula(&çözülmüş);
    }
    let etiket = &çözülmüş;
    if !etiket.göster {
        return;
    }
    let ham = öğe.ad.clone().unwrap_or_else(|| paralel_satır_metni(öğe));
    let değer = öğe.değer.sayı().unwrap_or(çizgi.veri_sırası as f64);
    let metin = etiket
        .biçimleyici
        .as_ref()
        .map(|biçimleyici| biçimleyici.uygula(değer, &ham))
        .unwrap_or(ham);
    let (taban, yön) = match etiket.konum {
        EtiketKonumu::Başlangıç | EtiketKonumu::İçBaşlangıç | EtiketKonumu::Sol => {
            (çizgi.noktalar.first().copied(), (-1.0, 0.0))
        }
        EtiketKonumu::Üst | EtiketKonumu::İçÜst | EtiketKonumu::SağÜst | EtiketKonumu::SolÜst => {
            (çizgi.noktalar.last().copied(), (0.0, -1.0))
        }
        EtiketKonumu::Alt | EtiketKonumu::İçAlt | EtiketKonumu::SağAlt | EtiketKonumu::SolAlt => {
            (çizgi.noktalar.last().copied(), (0.0, 1.0))
        }
        EtiketKonumu::İç | EtiketKonumu::Merkez => (
            çizgi.noktalar.get(çizgi.noktalar.len() / 2).copied(),
            (0.0, 0.0),
        ),
        _ => (çizgi.noktalar.last().copied(), (1.0, 0.0)),
    };
    let Some(taban) = taban else { return };
    let uzaklık = etiket.uzaklık;
    let konum = (
        taban.0 + yön.0 * uzaklık + etiket.kayma.0,
        taban.1 + yön.1 * uzaklık + etiket.kayma.1,
    );
    let yatay = etiket.yatay_hiza.map(yatay_hiza).unwrap_or(if yön.0 < 0.0 {
        YatayHiza::Sağ
    } else if yön.0 > 0.0 {
        YatayHiza::Sol
    } else {
        YatayHiza::Orta
    });
    let dikey = etiket.dikey_hiza.map(dikey_hiza).unwrap_or(if yön.1 < 0.0 {
        DikeyHiza::Alt
    } else if yön.1 > 0.0 {
        DikeyHiza::Üst
    } else {
        DikeyHiza::Orta
    });
    çizici.yazı(
        &metin,
        konum,
        yatay,
        dikey,
        etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
        etiket
            .yazı
            .renk
            .unwrap_or(çizgi_rengi)
            .opaklık(etiket.yazı.opaklık.unwrap_or(1.0)),
        etiket.yazı.kalın,
    );
}

fn yatay_hiza(hiza: YazıYatayHizası) -> YatayHiza {
    match hiza {
        YazıYatayHizası::Sol => YatayHiza::Sol,
        YazıYatayHizası::Orta => YatayHiza::Orta,
        YazıYatayHizası::Sağ => YatayHiza::Sağ,
    }
}

fn dikey_hiza(hiza: YazıDikeyHizası) -> DikeyHiza {
    match hiza {
        YazıDikeyHizası::Üst => DikeyHiza::Üst,
        YazıDikeyHizası::Orta => DikeyHiza::Orta,
        YazıDikeyHizası::Alt => DikeyHiza::Alt,
    }
}

fn paralel_satır_metni(öğe: &VeriÖğesi) -> String {
    match &öğe.değer {
        VeriDeğeri::Boş => String::new(),
        VeriDeğeri::Sayı(değer) => crate::yardimci::bicim::ondalık_kırp(*değer),
        VeriDeğeri::Çift([x, y]) => format!(
            "{},{}",
            crate::yardimci::bicim::ondalık_kırp(*x),
            crate::yardimci::bicim::ondalık_kırp(*y)
        ),
        VeriDeğeri::Dizi(dizi) => dizi
            .iter()
            .map(|değer| crate::yardimci::bicim::ondalık_kırp(*değer))
            .collect::<Vec<_>>()
            .join(","),
        VeriDeğeri::KarmaDizi(dizi) => dizi
            .iter()
            .map(paralel_değer_metni)
            .collect::<Vec<_>>()
            .join(","),
        VeriDeğeri::Metin(metin) => metin.clone(),
        VeriDeğeri::Mantıksal(değer) => değer.to_string(),
        VeriDeğeri::Zaman(ms) => ms.to_string(),
    }
}

fn paralel_değer_metni(değer: &VeriDeğeri) -> String {
    match değer {
        VeriDeğeri::Boş => "-".to_owned(),
        VeriDeğeri::Sayı(değer) => crate::yardimci::bicim::ondalık_kırp(*değer),
        VeriDeğeri::Çift([x, y]) => format!("{x},{y}"),
        VeriDeğeri::Dizi(dizi) => dizi
            .iter()
            .map(|değer| crate::yardimci::bicim::ondalık_kırp(*değer))
            .collect::<Vec<_>>()
            .join(","),
        VeriDeğeri::KarmaDizi(dizi) => dizi
            .iter()
            .map(paralel_değer_metni)
            .collect::<Vec<_>>()
            .join(","),
        VeriDeğeri::Metin(metin) => metin.clone(),
        VeriDeğeri::Mantıksal(değer) => değer.to_string(),
        VeriDeğeri::Zaman(ms) => ms.to_string(),
    }
}

fn paralel_boyut_değeri(öğe: &VeriÖğesi, boyut: usize) -> Option<VeriDeğeri> {
    match &öğe.değer {
        VeriDeğeri::Dizi(dizi) => dizi.get(boyut).copied().map(VeriDeğeri::Sayı),
        VeriDeğeri::KarmaDizi(dizi) => dizi.get(boyut).cloned(),
        VeriDeğeri::Çift(çift) => çift.get(boyut).copied().map(VeriDeğeri::Sayı),
        _ if boyut == 0 => Some(öğe.değer.clone()),
        _ => öğe.boyutlar.get(boyut).map(|(_, değer)| değer.clone()),
    }
}

fn paralel_görsel_ham_değeri(
    seri: &ParalelSerisi,
    öğe: &VeriÖğesi,
    eşleme: &GörselEşleme,
) -> Option<VeriDeğeri> {
    match eşleme.boyut.as_ref() {
        None => paralel_boyut_değeri(öğe, 0),
        Some(BoyutSeçici::Sıra(sıra)) => paralel_boyut_değeri(öğe, *sıra),
        Some(BoyutSeçici::Ad(ad)) => öğe.boyut(ad).cloned().or_else(|| {
            seri.boyutlar
                .iter()
                .position(|boyut| &boyut.ad == ad)
                .and_then(|sıra| paralel_boyut_değeri(öğe, sıra))
        }),
    }
}

pub fn paralel_görsel_kapsamı(seri: &ParalelSerisi, eşleme: &GörselEşleme) -> [f64; 2] {
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    for öğe in &seri.veri {
        if let Some(değer) = paralel_görsel_ham_değeri(seri, öğe, eşleme)
            .and_then(|değer| değer.sayı())
            .filter(|değer| değer.is_finite())
        {
            kapsam[0] = kapsam[0].min(değer);
            kapsam[1] = kapsam[1].max(değer);
        }
    }
    if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
        kapsam = [0.0, 1.0];
    }
    eşleme.kapsam_çöz(kapsam)
}

/// Tooltip ve dış doğrulama için satır değerini eksen boyutuyla metinleştirir.
pub fn paralel_ipucu_değerleri(
    seri: &ParalelSerisi,
    yerleşim: &ParalelYerleşimi,
    veri_sırası: usize,
) -> Vec<(String, String)> {
    let Some(öğe) = seri.veri.get(veri_sırası) else {
        return Vec::new();
    };
    yerleşim
        .eksenler
        .iter()
        .map(|eksen| {
            let sıra = eksen.ana_boyut();
            let ad = eksen
                .eksen
                .seçenek
                .ad
                .clone()
                .or_else(|| seri.boyutlar.get(sıra).map(|boyut| boyut.ad.clone()))
                .unwrap_or_else(|| format!("dim{sıra}"));
            let değer = paralel_boyut_değeri(öğe, sıra)
                .as_ref()
                .map(paralel_değer_metni)
                .unwrap_or_else(|| "-".to_owned());
            (ad, değer)
        })
        .collect()
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::koordinat::ParalelYerleşimi;
    use crate::model::paralel::{ParalelEkseni, ParalelKoordinatı};
    use crate::model::stil::{EtiketYaması, ÇizgiStili};

    #[test]
    fn resmi_seri_varsayilanlarini_korur() {
        let seri = ParalelSerisi::yeni();
        assert_eq!(seri.z, 2);
        assert!(!seri.sessiz);
        assert_eq!(seri.çizgi_stili.kalınlık, 1.0);
        assert_eq!(seri.çizgi_stili.opaklık, 0.45);
        assert_eq!(seri.aktif_opaklık, 1.0);
        assert_eq!(seri.etkin_değil_opaklık, 0.05);
        assert_eq!(seri.yumuşaklık, 0.0);
        assert!(seri.gerçek_zamanlı);
        assert_eq!(seri.ilerlemeli, 300);
        assert!(!seri.etiket.göster);
        assert_eq!(
            seri.vurgu_etiketi.as_ref().and_then(|etiket| etiket.göster),
            Some(false)
        );
    }

    #[test]
    fn smooth_zrender_polyline_kontrol_noktalarini_birebir_kullanir() {
        use crate::cizim::yuzey::YolKomutu;

        let yol = paralel_yolu(&[(0.0, 0.0), (10.0, 10.0), (20.0, 0.0)], 0.5);
        assert_eq!(
            yol.komutlar,
            vec![
                YolKomutu::Taşı((0.0, 0.0)),
                YolKomutu::Kübik {
                    k1: (0.0, 0.0),
                    k2: (5.0, 10.0),
                    uç: (10.0, 10.0),
                },
                YolKomutu::Kübik {
                    k1: (15.0, 10.0),
                    k2: (20.0, 0.0),
                    uç: (20.0, 0.0),
                },
            ]
        );
    }

    #[test]
    fn kategori_bos_deger_smooth_ve_coklu_cizgi_hitini_destekler() {
        let seri = ParalelSerisi::yeni()
            .ad("Ölçüm")
            .boyutlar(["A", "B", "Sınıf"])
            .karma_veri([
                vec![
                    VeriDeğeri::from(10),
                    VeriDeğeri::Boş,
                    VeriDeğeri::from("iyi"),
                ],
                vec![
                    VeriDeğeri::from(20),
                    VeriDeğeri::from(30),
                    VeriDeğeri::from("orta"),
                ],
            ])
            .yumuşak(true)
            .çizgi_stili(ÇizgiStili::yeni().kalınlık(4.0));
        let yerleşim = ParalelYerleşimi::kur(
            &ParalelKoordinatı::yeni(),
            0,
            &[
                ParalelEkseni::yeni(0),
                ParalelEkseni::yeni(1),
                ParalelEkseni::yeni(2).kategori().veri(["iyi", "orta"]),
            ],
            &[&seri],
            (600.0, 450.0),
        )
        .unwrap();
        let mut yüzey = KayıtYüzeyi::yeni(600.0, 450.0);
        let mut isabetler = Vec::new();
        paralel_çiz(
            &mut yüzey,
            &seri,
            0,
            &yerleşim,
            Renk::onaltılık(0x5470c6),
            &[],
            1.0,
            None,
            None,
            &mut isabetler,
        );
        assert_eq!(isabetler.len(), 2);
        assert!(matches!(
            &isabetler[0].geometri,
            İsabetGeometrisi::ÇokluÇizgi { noktalar, .. } if noktalar.len() == 2
        ));
        assert!(yüzey.döküm().contains("K("));
    }

    #[test]
    fn sessiz_seri_hit_uretmez_ama_programatik_durum_etiketini_cizer() {
        let seri = ParalelSerisi::yeni()
            .ad("Ölçüm")
            .boyutlar(["A", "B"])
            .veri([vec![10.0, 20.0]])
            .sessiz(true)
            .vurgu_etiketi(EtiketYaması::yeni().göster(true));
        let yerleşim =
            ParalelYerleşimi::kur(&ParalelKoordinatı::yeni(), 0, &[], &[&seri], (600.0, 450.0))
                .unwrap();
        let mut yüzey = KayıtYüzeyi::yeni(600.0, 450.0);
        let mut isabetler = Vec::new();
        let vurgu = paralel_çiz(
            &mut yüzey,
            &seri,
            0,
            &yerleşim,
            Renk::onaltılık(0x5470c6),
            &[],
            1.0,
            Some((80.0, 100.0)),
            Some(0),
            &mut isabetler,
        );
        assert_eq!(vurgu, Some(0));
        assert!(isabetler.is_empty());
        assert!(yüzey.döküm().contains("10,20"), "{}", yüzey.döküm());
    }
}
