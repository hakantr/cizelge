//! Saçılım serisi çizimi — `echarts/src/chart/scatter` karşılığı.

use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::sembol_stilli_çiz;
use crate::koordinat::{Kartezyen2B, TakvimYerleşimi};
use crate::model::deger::{VeriDeğeri, VeriÖğesi};
use crate::model::seri::{SaçılımSerisi, Sembol};
use crate::model::stil::{EtiketDöndürme, EtiketKonumu, YazıDikeyHizası, YazıYatayHizası};
use crate::renk::Renk;
use crate::tema;
use crate::yardimci::bicim::ondalık_kırp;

fn eksen_değeri(
    öğe: &VeriÖğesi,
    boyut: &str,
    eksen: &crate::koordinat::ÇalışmaEkseni,
) -> Option<f64> {
    let değer = öğe.boyut(boyut)?;
    if !eksen.ölçek.kategorik_mi() {
        return değer.sayı().filter(|değer| değer.is_finite());
    }
    let ad = match değer {
        VeriDeğeri::Metin(ad) => ad.clone(),
        VeriDeğeri::Sayı(değer) => crate::yardimci::bicim::ondalık_kırp(*değer),
        VeriDeğeri::Zaman(değer) => değer.to_string(),
        VeriDeğeri::Mantıksal(değer) => değer.to_string(),
        VeriDeğeri::Boş | VeriDeğeri::Çift(_) | VeriDeğeri::Dizi(_) => return None,
    };
    eksen.ölçek.kategori_sırası(&ad)
}

/// Yerleşimi hesaplanmış bir saçılım noktası.
#[derive(Clone, Copy, Debug)]
pub struct SaçılımNoktası {
    pub sıra: usize,
    pub konum: (f32, f32),
    /// Sembol çapı.
    pub boyut: f32,
    pub x_değeri: f64,
    pub y_değeri: f64,
}

/// Serinin piksel noktalarını üretir. Veri `[x, y]` çifti değilse `x`
/// olarak veri sırası kullanılır.
pub fn saçılım_noktaları(
    seri: &SaçılımSerisi,
    kartezyen: &Kartezyen2B,
) -> Vec<SaçılımNoktası> {
    let mut sonuç = Vec::with_capacity(seri.veri.len());
    for (i, öğe) in seri.veri.iter().enumerate() {
        let (x, y) = match &seri.eşleme {
            Some((x_boyutu, y_boyutu)) => {
                let (Some(x), Some(y)) = (
                    eksen_değeri(öğe, x_boyutu, &kartezyen.x),
                    eksen_değeri(öğe, y_boyutu, &kartezyen.y),
                ) else {
                    continue;
                };
                (x, y)
            }
            None => {
                if öğe.değer.boş_mu() {
                    continue;
                }
                let Some(y) = öğe.değer.sayı() else {
                    continue;
                };
                (öğe.değer.x().unwrap_or(i as f64), y)
            }
        };
        sonuç.push(SaçılımNoktası {
            sıra: i,
            konum: kartezyen.nokta(x, y),
            boyut: seri.sembol_boyutu.çöz(öğe),
            x_değeri: x,
            y_değeri: y,
        });
    }
    sonuç
}

/// Takvim koordinatına bağlı scatter/effectScatter noktalarını üretir.
/// Veri ECharts'taki gibi `[tarih, değer]` çiftidir; tarih hücre merkezine,
/// ikinci boyut sembol boyutu/etiket/ipucu değerine akar.
pub fn takvim_saçılım_noktaları(
    seri: &SaçılımSerisi,
    takvim: &TakvimYerleşimi,
) -> Vec<SaçılımNoktası> {
    seri.veri
        .iter()
        .enumerate()
        .filter_map(|(sıra, öğe)| {
            let tarih = öğe.değer.x()?;
            let değer = öğe.değer.sayı()?;
            Some(SaçılımNoktası {
                sıra,
                konum: takvim.veriden_noktaya(tarih)?,
                boyut: seri.sembol_boyutu.çöz(öğe),
                x_değeri: tarih,
                y_değeri: değer,
            })
        })
        .collect()
}

fn sembol_gölge_yolu(sembol: Sembol, merkez: (f32, f32), boyut: f32) -> Option<Yol> {
    let yarıçap = boyut / 2.0;
    if yarıçap <= 0.0 || sembol == Sembol::Yok {
        return None;
    }
    let mut yol = Yol::yeni();
    match sembol {
        Sembol::Daire | Sembol::İçiBoşDaire => {
            yol.taşı((merkez.0 + yarıçap, merkez.1));
            yol.yay(yarıçap, false, true, (merkez.0 - yarıçap, merkez.1));
            yol.yay(yarıçap, false, true, (merkez.0 + yarıçap, merkez.1));
        }
        Sembol::Kare => {
            yol.taşı((merkez.0 - yarıçap, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1 + yarıçap));
        }
        Sembol::Üçgen => {
            yol.taşı((merkez.0, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1 + yarıçap));
        }
        Sembol::Elmas => {
            yol.taşı((merkez.0, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1));
            yol.çiz((merkez.0, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1));
        }
        Sembol::Yok => return None,
    }
    yol.kapat();
    Some(yol)
}

/// Saçılım serisini çizer; `vurgulu` ipucuyla öne çıkarılan noktadır.
/// `zaman_sn`, sürekli dalga efekti için geçen süredir (saniye).
#[allow(clippy::too_many_arguments)]
pub fn saçılım_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &SaçılımSerisi,
    noktalar: &[SaçılımNoktası],
    seri_rengi: Renk,
    ilerleme: f32,
    zaman_sn: f32,
    vurgulu: Option<usize>,
) {
    // `scatter` öntanımlı 0.8, `effectScatter` ise 1.0 opaklıktadır.
    let opaklık = seri
        .öğe_stili
        .opaklık
        .unwrap_or(if seri.efektli { 1.0 } else { 0.8 });
    let renk = seri
        .öğe_stili
        .renk
        .as_ref()
        .map(|d| d.temsilî())
        .unwrap_or(seri_rengi);
    // EffectSymbol çekirdeği önce, z2=99 dalgaları sonra boyar.
    for nokta in noktalar {
        let vurgulu_mu = vurgulu == Some(nokta.sıra);
        let boyut = nokta.boyut * ilerleme.clamp(0.0, 1.0) * if vurgulu_mu { 1.15 } else { 1.0 };
        let nokta_opaklığı = if vurgulu_mu { 1.0 } else { opaklık };
        if let Some(gölge_rengi) = seri.öğe_stili.gölge_rengi
            && (seri.öğe_stili.gölge_bulanıklığı > 0.0
                || seri.öğe_stili.gölge_kayması != (0.0, 0.0))
            && let Some(yol) = sembol_gölge_yolu(seri.sembol, nokta.konum, boyut)
        {
            çizici.yol_gölgesi(
                &yol,
                gölge_rengi.opaklık(nokta_opaklığı),
                seri.öğe_stili.gölge_bulanıklığı,
                seri.öğe_stili.gölge_kayması,
            );
        }
        let kenarlık = (seri.öğe_stili.kenarlık_kalınlığı > 0.0).then(|| {
            (
                seri.öğe_stili.kenarlık_kalınlığı,
                seri.öğe_stili.kenarlık_rengi.unwrap_or(renk),
            )
        });
        sembol_stilli_çiz(
            çizici,
            seri.sembol,
            nokta.konum,
            boyut,
            renk,
            seri.öğe_stili.renk.as_ref(),
            kenarlık,
            nokta_opaklığı,
        );
    }

    // Dataset `encode.label` dâhil saçılım etiketleri. Öğe yaması seri
    // etiketini miras alır; açık align/verticalAlign/rotate değerleri
    // zrender bağlı metin yerleşimine aktarılır.
    for nokta in noktalar {
        let Some(öğe) = seri.veri.get(nokta.sıra) else {
            continue;
        };
        let öğe_etiketi = öğe.etiket.as_ref().map(|yama| yama.uygula(&seri.etiket));
        let etiket = öğe_etiketi.as_ref().unwrap_or(&seri.etiket);
        if !etiket.göster {
            continue;
        }
        let etiket_değeri = seri
            .etiket_boyutu
            .as_deref()
            .and_then(|boyut| öğe.boyut(boyut))
            .unwrap_or(&öğe.değer);
        let ham = match etiket_değeri {
            VeriDeğeri::Sayı(değer) => ondalık_kırp(*değer),
            VeriDeğeri::Metin(metin) => metin.clone(),
            VeriDeğeri::Zaman(ms) => ms.to_string(),
            VeriDeğeri::Mantıksal(değer) => değer.to_string(),
            VeriDeğeri::Çift([x, y]) => format!("{},{}", ondalık_kırp(*x), ondalık_kırp(*y)),
            VeriDeğeri::Dizi(değerler) => değerler
                .iter()
                .map(|değer| ondalık_kırp(*değer))
                .collect::<Vec<_>>()
                .join(","),
            VeriDeğeri::Boş => continue,
        };
        let biçim_değeri = etiket_değeri.sayı().unwrap_or(nokta.y_değeri);
        let metin = etiket
            .biçimleyici
            .as_ref()
            .map(|biçimleyici| {
                biçimleyici.uygula_bağlamla(
                    biçim_değeri,
                    &ham,
                    seri.ad.as_deref().unwrap_or(""),
                    öğe.ad.as_deref().unwrap_or(""),
                )
            })
            .unwrap_or(ham);
        let uzaklık = etiket.uzaklık + nokta.boyut / 2.0;
        let (çapa, doğal_yatay, doğal_dikey) = match etiket.konum {
            EtiketKonumu::Üst => (
                (nokta.konum.0, nokta.konum.1 - uzaklık),
                YatayHiza::Orta,
                DikeyHiza::Alt,
            ),
            EtiketKonumu::Alt => (
                (nokta.konum.0, nokta.konum.1 + uzaklık),
                YatayHiza::Orta,
                DikeyHiza::Üst,
            ),
            EtiketKonumu::Sol => (
                (nokta.konum.0 - uzaklık, nokta.konum.1),
                YatayHiza::Sağ,
                DikeyHiza::Orta,
            ),
            EtiketKonumu::Sağ => (
                (nokta.konum.0 + uzaklık, nokta.konum.1),
                YatayHiza::Sol,
                DikeyHiza::Orta,
            ),
            _ => (nokta.konum, YatayHiza::Orta, DikeyHiza::Orta),
        };
        let yatay = etiket
            .yatay_hiza
            .map(|hiza| match hiza {
                YazıYatayHizası::Sol => YatayHiza::Sol,
                YazıYatayHizası::Orta => YatayHiza::Orta,
                YazıYatayHizası::Sağ => YatayHiza::Sağ,
            })
            .unwrap_or(doğal_yatay);
        let dikey = etiket
            .dikey_hiza
            .map(|hiza| match hiza {
                YazıDikeyHizası::Üst => DikeyHiza::Üst,
                YazıDikeyHizası::Orta => DikeyHiza::Orta,
                YazıDikeyHizası::Alt => DikeyHiza::Alt,
            })
            .unwrap_or(doğal_dikey);
        let boyut = etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
        let renk = etiket.yazı.renk.unwrap_or(tema::birincil_metin());
        match etiket.döndürme {
            EtiketDöndürme::Derece(derece) if derece.abs() > f32::EPSILON => {
                çizici.dönüşümlü_yazı(
                    &metin,
                    (0.0, 0.0),
                    yatay,
                    dikey,
                    boyut,
                    renk,
                    etiket.yazı.kalın,
                    AfinMatris::ötele(çapa.0, çapa.1)
                        .çarp(AfinMatris::döndür(-derece.to_radians())),
                );
            }
            _ => {
                çizici.yazı(&metin, çapa, yatay, dikey, boyut, renk, etiket.yazı.kalın);
            }
        }
    }

    // Dalga efekti: EffectSymbol'daki üç doğrusal animatorün tam karşılığı;
    // yarıçap sembol yarıçapından `rippleEffect.scale` katına çıkarken
    // opaklık 1'den 0'a iner.
    if seri.efektli && ilerleme >= 0.999 {
        const DALGA_SAYISI: usize = 3;
        let tur = (zaman_sn / seri.efekt_süresi_sn.max(0.1)).fract();
        for nokta in noktalar {
            for d in 0..DALGA_SAYISI {
                let evre = (tur + d as f32 / DALGA_SAYISI as f32).fract();
                let yarıçap = (nokta.boyut / 2.0) * (1.0 + evre * (seri.efekt_ölçeği - 1.0));
                let alfa = 1.0 - evre;
                if alfa <= 0.001 {
                    continue;
                }
                if seri.efekt_vuruşlu {
                    çizici.daire(nokta.konum, yarıçap, None, Some((1.0, renk.alfa_ile(alfa))));
                } else {
                    çizici.daire(
                        nokta.konum,
                        yarıçap,
                        Some(&crate::renk::Dolgu::Düz(renk.alfa_ile(alfa))),
                        None,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod testler {
    use super::*;
    use crate::koordinat::{Dikdörtgen, ÇalışmaEkseni};
    use crate::model::eksen::{Eksen, EksenKonumu};
    use crate::model::takvim::TakvimKoordinatı;
    use crate::olcek::{AralıkÖlçeği, KategorikÖlçek, Ölçek};
    use crate::yardimci::takvim::{TakvimAnı, takvimden_ana};

    fn değer_ekseni(kapsam: [f64; 2], piksel: [f32; 2], konum: EksenKonumu) -> ÇalışmaEkseni {
        ÇalışmaEkseni::yeni(
            Eksen::değer(),
            Ölçek::Aralık(AralıkÖlçeği::kur(
                kapsam,
                Some(kapsam[0]),
                Some(kapsam[1]),
                false,
                5,
                None,
                None,
            )),
            piksel,
            konum,
        )
    }

    #[test]
    fn dataset_encode_sayısal_x_y_boyutlarını_koordinata_aktarır() {
        let seri = SaçılımSerisi::yeni()
            .eşle("gelir", "ömür")
            .veri([VeriÖğesi::yeni(999.0).boyutlar([
                ("gelir".to_string(), 5.0.into()),
                ("ömür".to_string(), 20.0.into()),
            ])]);
        let kartezyen = Kartezyen2B {
            x: değer_ekseni([0.0, 10.0], [0.0, 100.0], EksenKonumu::Alt),
            y: değer_ekseni([0.0, 40.0], [100.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        };
        let noktalar = saçılım_noktaları(&seri, &kartezyen);
        assert_eq!(noktalar.len(), 1);
        assert!((noktalar[0].konum.0 - 50.0).abs() < 1e-5);
        assert!((noktalar[0].konum.1 - 50.0).abs() < 1e-5);
        assert_eq!(noktalar[0].x_değeri, 5.0);
        assert_eq!(noktalar[0].y_değeri, 20.0);
    }

    #[test]
    fn dataset_encode_kategori_x_boyutunu_ordinal_sıraya_aktarır() {
        let seri = SaçılımSerisi::yeni()
            .eşle("ülke", "gelir")
            .veri([VeriÖğesi::yeni(10.0).boyutlar([
                ("ülke".to_string(), "Fransa".into()),
                ("gelir".to_string(), 10.0.into()),
            ])]);
        let kartezyen = Kartezyen2B {
            x: ÇalışmaEkseni::yeni(
                Eksen::kategori().kenar_boşluğu(false),
                Ölçek::Kategorik(KategorikÖlçek::yeni(vec![
                    "Almanya".to_string(),
                    "Fransa".to_string(),
                ])),
                [0.0, 100.0],
                EksenKonumu::Alt,
            ),
            y: değer_ekseni([0.0, 20.0], [100.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        };
        let noktalar = saçılım_noktaları(&seri, &kartezyen);
        assert_eq!(noktalar.len(), 1);
        assert!((noktalar[0].konum.0 - 100.0).abs() < 1e-5);
        assert!((noktalar[0].konum.1 - 50.0).abs() < 1e-5);
    }

    #[test]
    fn takvim_scatter_tarihi_hücre_merkezine_ve_değeri_boyuta_aktarır() {
        let tarih = takvimden_ana(TakvimAnı {
            yıl: 2017,
            ay: 1,
            gün: 1,
            saat: 0,
            dakika: 0,
            saniye: 0,
            milisaniye: 0,
        });
        let seri = SaçılımSerisi::yeni()
            .takvim_sırası(0)
            .sembol_boyutu_işlevi(|öğe| öğe.değer.sayı().unwrap_or(0.0) as f32 / 50.0)
            .veri([VeriÖğesi::from([tarih, 500.0])]);
        let yerleşim = TakvimYerleşimi::kur(&TakvimKoordinatı::yıl(2017), (700.0, 525.0))
            .expect("takvim yerleşimi kurulmalı");

        let noktalar = takvim_saçılım_noktaları(&seri, &yerleşim);

        assert_eq!(noktalar.len(), 1);
        assert_eq!(noktalar[0].konum, (90.0, 70.0));
        assert_eq!(noktalar[0].boyut, 10.0);
        assert_eq!(noktalar[0].x_değeri, tarih);
        assert_eq!(noktalar[0].y_değeri, 500.0);
    }
}
