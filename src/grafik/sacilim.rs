//! Saçılım serisi çizimi — `echarts/src/chart/scatter` karşılığı.

use std::borrow::Cow;

use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::sembol_stilli_çiz;
use crate::koordinat::{
    Dikdörtgen, Kartezyen2B, MatrisYerleşimi, TakvimYerleşimi, TekEksenYerleşimi, ÇalışmaEkseni,
};
use crate::model::deger::{VeriDeğeri, VeriÖğesi};
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::matris::MatrisAralığı;
use crate::model::seri::{
    EtiketYerleşimParametreleri, EtiketYerleşimSonucu, EtiketÖrtüşmeKaydırması, SaçılımSerisi,
};
use crate::model::stil::{EtiketDöndürme, EtiketKonumu, YazıDikeyHizası, YazıYatayHizası};
use crate::model::veri_kumesi::BoyutSeçici;
use crate::renk::{Dolgu, Renk};
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
        VeriDeğeri::Boş | VeriDeğeri::Çift(_) | VeriDeğeri::Dizi(_) | VeriDeğeri::KarmaDizi(_) =>
        {
            return None;
        }
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
    /// `colorBy: 'data'` palet anahtarı; kategorik eksende kategori sırası,
    /// iki sayısal eksende `None` (ham veri sırası kullanılır).
    pub palet_sırası: Option<usize>,
}

/// `LargeSymbolPath` için veri sırasını koruyan iç içe olmayan ekran
/// koordinatları. Kırpılan/geçersiz öğeler kendi yerlerinde `NaN, NaN`
/// taşır; böylece hover sonucu doğrudan özgün `dataIndex` olur.
#[derive(Clone, Debug)]
pub struct BüyükSaçılımNoktaları {
    pub konumlar: Vec<f32>,
    pub boyut: f32,
}

/// Bir scatter serisine uygulanacak, kapsamı çözülmüş `visualMap` girdisi.
/// ECharts görsel kodlama aşaması aynı seride birden çok bileşenin farklı
/// kanallarını sırayla birleştirir.
pub type SaçılımGörselEşlemesi<'a> = (&'a GörselEşleme, [f64; 2]);

/// ECharts scatter verisinin ilk iki boyutunu kartezyen koordinata çözer.
/// `[x, y, ...]` biçimindeki ek boyutlar sembol boyutu, etiket, tooltip ve
/// visualMap gibi kanallar için veri öğesinde korunur.
pub(crate) fn saçılım_xy(değer: &VeriDeğeri, sıra: usize) -> Option<(f64, f64)> {
    if değer.boş_mu() {
        return None;
    }
    match değer {
        VeriDeğeri::Çift([x, y]) => Some((*x, *y)),
        VeriDeğeri::Dizi(değerler) if değerler.len() >= 2 => Some((değerler[0], değerler[1])),
        _ => değer.sayı().map(|y| (sıra as f64, y)),
    }
}

#[derive(Clone, Copy)]
struct TitremeÖğesi {
    sabit: f64,
    kayan: f64,
    yarıçap: f64,
}

fn titreme_yönünde_yerleştir(
    öğeler: &[TitremeÖğesi],
    sabit: f64,
    kayan: f64,
    yarıçap: f64,
    titreme: f64,
    boşluk: f64,
    yön: f64,
) -> f64 {
    let mut yeni = kayan;
    let mut sıra = 0usize;
    while sıra < öğeler.len() {
        let öğe = öğeler[sıra];
        let dx = sabit - öğe.sabit;
        let dy = yeni - öğe.kayan;
        let toplam_yarıçap = yarıçap + öğe.yarıçap + boşluk;
        if dx * dx + dy * dy < toplam_yarıçap * toplam_yarıçap {
            let kök = (toplam_yarıçap * toplam_yarıçap - dx * dx).max(0.0).sqrt();
            let gereken = öğe.kayan + kök * yön;
            if (gereken - kayan).abs() > titreme / 2.0 {
                return f64::MAX;
            }
            if (yön > 0.0 && gereken > yeni) || (yön < 0.0 && gereken < yeni) {
                yeni = gereken;
                sıra = 0;
                continue;
            }
        }
        sıra += 1;
    }
    yeni
}

fn titreme_rastgelesi(durum: &mut u32) -> f64 {
    *durum = durum.wrapping_add(0x6d2b_79f5);
    let mut t = (*durum ^ (*durum >> 15)).wrapping_mul(1 | *durum);
    t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
    (t ^ (t >> 14)) as f64 / 4_294_967_296.0
}

fn titremeyi_eksende_uygula(
    noktalar: &mut [SaçılımNoktası], eksen: &ÇalışmaEkseni, x_mi: bool
) {
    if !eksen.ölçek.kategorik_mi() || eksen.seçenek.titreme <= 0.0 {
        return;
    }
    let titreme = eksen.seçenek.titreme;
    let bant = eksen.bant_genişliği() as f64;
    let mut yerleşenler = Vec::with_capacity(noktalar.len());
    // Görsel kanıt hattı Math.random'ı aynı Mulberry32 tohumu ile sabitler;
    // çekirdekteki sabit akış, yeniden boyamalarda nokta sıçramasını önler.
    let mut rastgele = eksen.seçenek.titreme_tohumu;
    for nokta in noktalar {
        let (sabit, kayan) = if x_mi {
            (nokta.konum.1 as f64, nokta.konum.0 as f64)
        } else {
            (nokta.konum.0 as f64, nokta.konum.1 as f64)
        };
        let yarıçap = nokta.boyut as f64 / 2.0;
        let etkin_titreme = titreme.min((bant - yarıçap * 2.0).max(0.0));
        let mut rastgele_yer = || kayan + (titreme_rastgelesi(&mut rastgele) - 0.5) * etkin_titreme;
        let yeni = if eksen.seçenek.titreme_örtüşmesi {
            rastgele_yer()
        } else {
            let artı = titreme_yönünde_yerleştir(
                &yerleşenler,
                sabit,
                kayan,
                yarıçap,
                etkin_titreme,
                eksen.seçenek.titreme_boşluğu,
                1.0,
            );
            let eksi = titreme_yönünde_yerleştir(
                &yerleşenler,
                sabit,
                kayan,
                yarıçap,
                etkin_titreme,
                eksen.seçenek.titreme_boşluğu,
                -1.0,
            );
            let aday = if (artı - kayan).abs() < (eksi - kayan).abs() {
                artı
            } else {
                eksi
            };
            if (aday - kayan).abs() > etkin_titreme / 2.0
                || (aday - kayan).abs() > bant / 2.0 - yarıçap
            {
                rastgele_yer()
            } else {
                yerleşenler.push(TitremeÖğesi {
                    sabit,
                    kayan: aday,
                    yarıçap,
                });
                aday
            }
        };
        if x_mi {
            nokta.konum.0 = yeni as f32;
        } else {
            nokta.konum.1 = yeni as f32;
        }
    }
}

fn titremeyi_uygula(noktalar: &mut [SaçılımNoktası], kartezyen: &Kartezyen2B) {
    if kartezyen.x.ölçek.kategorik_mi() && kartezyen.x.seçenek.titreme > 0.0 {
        titremeyi_eksende_uygula(noktalar, &kartezyen.x, true);
    } else if kartezyen.y.ölçek.kategorik_mi() && kartezyen.y.seçenek.titreme > 0.0 {
        titremeyi_eksende_uygula(noktalar, &kartezyen.y, false);
    }
}

/// Serinin piksel noktalarını üretir. Veri `[x, y]` çifti değilse `x`
/// olarak veri sırası kullanılır.
pub fn saçılım_noktaları(
    seri: &SaçılımSerisi,
    kartezyen: &Kartezyen2B,
) -> Vec<SaçılımNoktası> {
    if seri.düz_veri.is_some() {
        let mut sonuç = Vec::with_capacity(seri.veri_sayısı());
        for (sıra, x, y) in seri.düz_xy_iter() {
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            let konum = kartezyen.nokta(x, y);
            if !kartezyen.alan.içeriyor_mu(konum) {
                continue;
            }
            sonuç.push(SaçılımNoktası {
                sıra,
                konum,
                boyut: seri.düz_boyut_çöz(sıra, x, y),
                x_değeri: x,
                y_değeri: y,
                palet_sırası: None,
            });
        }
        titremeyi_uygula(&mut sonuç, kartezyen);
        return sonuç;
    }

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
            None => match saçılım_xy(&öğe.değer, i) {
                Some(koordinat) => koordinat,
                None => continue,
            },
        };
        sonuç.push(SaçılımNoktası {
            sıra: i,
            konum: kartezyen.nokta(x, y),
            boyut: seri.sembol_boyutu.bağlamla_çöz(öğe, i),
            x_değeri: x,
            y_değeri: y,
            palet_sırası: if kartezyen.x.ölçek.kategorik_mi() {
                Some(x.max(0.0).round() as usize)
            } else if kartezyen.y.ölçek.kategorik_mi() {
                Some(y.max(0.0).round() as usize)
            } else {
                None
            },
        });
    }
    titremeyi_uygula(&mut sonuç, kartezyen);
    // ECharts `SymbolDraw`, scatter grubuna bir clip-path takmak yerine
    // sembol merkezini koordinat alanıyla sınar. Böylece merkez sınırdaysa
    // sembolün dışarı taşan yarısı kesilmez; merkez dışarıdaysa öğe hiç
    // çizilmez. Jitter yerleşimi de bu sınamadan önce uygulanır.
    sonuç.retain(|nokta| kartezyen.alan.içeriyor_mu(nokta.konum));
    sonuç
}

/// ECharts `pointsLayout(..., true)` + `LargeSymbolDraw` karşılığı. Büyük
/// kipte sembol boyutu seri düzeyinde tek değerdir; işlevsel boyut varsa
/// normal sembol hattına geri dönülür.
pub fn büyük_saçılım_noktaları(
    seri: &SaçılımSerisi,
    kartezyen: &Kartezyen2B,
) -> Option<BüyükSaçılımNoktaları> {
    let crate::model::seri::SembolBoyutu::Sabit(boyut) = seri.sembol_boyutu else {
        return None;
    };
    let mut konumlar = Vec::with_capacity(seri.veri_sayısı().saturating_mul(2));
    let mut ekle = |koordinat: Option<(f64, f64)>| {
        if let Some((x, y)) = koordinat
            && x.is_finite()
            && y.is_finite()
        {
            let konum = kartezyen.nokta(x, y);
            if kartezyen.alan.içeriyor_mu(konum) {
                konumlar.extend_from_slice(&[konum.0, konum.1]);
                return;
            }
        }
        konumlar.extend_from_slice(&[f32::NAN, f32::NAN]);
    };
    if seri.düz_veri.is_some() {
        for (_, x, y) in seri.düz_xy_iter() {
            ekle(Some((x, y)));
        }
    } else {
        for (sıra, öğe) in seri.veri.iter().enumerate() {
            let koordinat = match &seri.eşleme {
                Some((x_boyutu, y_boyutu)) => {
                    let x = eksen_değeri(öğe, x_boyutu, &kartezyen.x);
                    let y = eksen_değeri(öğe, y_boyutu, &kartezyen.y);
                    x.zip(y)
                }
                None => saçılım_xy(&öğe.değer, sıra),
            };
            ekle(koordinat);
        }
    }
    Some(BüyükSaçılımNoktaları { konumlar, boyut })
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
                boyut: seri.sembol_boyutu.bağlamla_çöz(öğe, sıra),
                x_değeri: tarih,
                y_değeri: değer,
                palet_sırası: None,
            })
        })
        .collect()
}

/// Matrix koordinatına bağlı scatter/effectScatter noktalarını üretir.
/// Açık `matris_koordinatları` yoksa `[x, y, value]` dizisinin ilk iki
/// boyutu gövde sıra numarası kabul edilir.
pub fn matris_saçılım_noktaları(
    seri: &SaçılımSerisi,
    matris: &MatrisYerleşimi,
) -> Vec<SaçılımNoktası> {
    seri.veri
        .iter()
        .enumerate()
        .filter_map(|(sıra, öğe)| {
            let sayısal = öğe.değer.dizi();
            let açık = seri.matris_koordinatları.get(sıra).and_then(Option::as_ref);
            let (x, y) = match açık {
                Some((x, y)) => (x.clone(), y.clone()),
                None => {
                    let dizi = sayısal?;
                    let x = *dizi.first()?;
                    let y = *dizi.get(1)?;
                    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
                        return None;
                    }
                    (
                        MatrisAralığı::from(x.round() as usize),
                        MatrisAralığı::from(y.round() as usize),
                    )
                }
            };
            let görsel_değer = sayısal
                .and_then(|dizi| dizi.get(2).copied().or_else(|| dizi.get(1).copied()))
                .or_else(|| öğe.değer.sayı())?;
            Some(SaçılımNoktası {
                sıra,
                konum: matris.veriden_noktaya(x, y)?,
                boyut: seri.sembol_boyutu.bağlamla_çöz(öğe, sıra),
                x_değeri: sayısal
                    .and_then(|dizi| dizi.first().copied())
                    .unwrap_or(sıra as f64),
                y_değeri: görsel_değer,
                palet_sırası: None,
            })
        })
        .collect()
}

/// `singleAxis`e bağlı scatter/effectScatter noktalarını üretir. Veri
/// ECharts'taki gibi `[eksenDeğeri, görselDeğer, ...]` biçimindedir; ikinci
/// boyut sembol boyutu, etiket, tooltip ve visualMap için korunur.
pub fn tek_eksen_saçılım_noktaları(
    seri: &SaçılımSerisi,
    yerleşim: &TekEksenYerleşimi,
) -> Vec<SaçılımNoktası> {
    let mut noktalar = seri
        .veri
        .iter()
        .enumerate()
        .filter_map(|(sıra, öğe)| {
            let (eksen_değeri, görsel_değer) = saçılım_xy(&öğe.değer, sıra)?;
            let konum = yerleşim.veriden_noktaya(eksen_değeri);
            yerleşim.içeriyor_mu(konum).then_some(SaçılımNoktası {
                sıra,
                konum,
                boyut: seri.sembol_boyutu.bağlamla_çöz(öğe, sıra),
                x_değeri: eksen_değeri,
                y_değeri: görsel_değer,
                palet_sırası: None,
            })
        })
        .collect::<Vec<_>>();
    titremeyi_eksende_uygula(
        &mut noktalar,
        &yerleşim.eksen,
        yerleşim.yön == crate::model::tek_eksen::TekEksenYönü::Yatay,
    );
    noktalar
}

/// Scatter/effectScatter ikinci koordinat boyutunun görsel eşleme kapsamı.
pub fn saçılım_değer_kapsamı(seri: &SaçılımSerisi) -> [f64; 2] {
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    let düz_değerler = seri.düz_xy_iter().map(|(_, _, y)| y);
    let nesne_değerleri = seri
        .veri
        .iter()
        .enumerate()
        .filter_map(|(sıra, öğe)| saçılım_xy(&öğe.değer, sıra).map(|(_, y)| y));
    for değer in düz_değerler.chain(nesne_değerleri) {
        if değer.is_finite() {
            kapsam[0] = kapsam[0].min(değer);
            kapsam[1] = kapsam[1].max(değer);
        }
    }
    if kapsam[0].is_finite() {
        kapsam
    } else {
        [0.0, 1.0]
    }
}

fn saçılım_görsel_ham_değeri<'a>(
    seri: &'a SaçılımSerisi,
    eşleme: &GörselEşleme,
    sıra: usize,
) -> Option<Cow<'a, VeriDeğeri>> {
    if let Some((x, y)) = seri.düz_veri.as_ref().and_then(|veri| veri.xy(sıra)) {
        return match eşleme.boyut.as_ref() {
            None | Some(BoyutSeçici::Sıra(1)) => Some(Cow::Owned(VeriDeğeri::Sayı(y))),
            Some(BoyutSeçici::Sıra(0)) => Some(Cow::Owned(VeriDeğeri::Sayı(x))),
            Some(BoyutSeçici::Ad(ad)) if ad == "x" => Some(Cow::Owned(VeriDeğeri::Sayı(x))),
            Some(BoyutSeçici::Ad(ad)) if ad == "y" => Some(Cow::Owned(VeriDeğeri::Sayı(y))),
            Some(BoyutSeçici::Sıra(_)) | Some(BoyutSeçici::Ad(_)) => None,
        };
    }
    let öğe = seri.veri.get(sıra)?;
    match eşleme.boyut.as_ref() {
        None => saçılım_xy(&öğe.değer, sıra).map(|(_, y)| Cow::Owned(VeriDeğeri::Sayı(y))),
        Some(BoyutSeçici::Sıra(boyut)) => {
            if !öğe.boyutlar.is_empty() {
                return öğe
                    .boyutlar
                    .get(*boyut)
                    .map(|(_, değer)| Cow::Borrowed(değer));
            }
            match &öğe.değer {
                VeriDeğeri::Dizi(değerler) => değerler
                    .get(*boyut)
                    .map(|değer| Cow::Owned(VeriDeğeri::Sayı(*değer))),
                VeriDeğeri::Çift(değerler) => değerler
                    .get(*boyut)
                    .map(|değer| Cow::Owned(VeriDeğeri::Sayı(*değer))),
                _ if *boyut == 0 => öğe
                    .değer
                    .x()
                    .or(Some(sıra as f64))
                    .map(|değer| Cow::Owned(VeriDeğeri::Sayı(değer))),
                _ if *boyut == 1 => Some(Cow::Borrowed(&öğe.değer)),
                _ => None,
            }
        }
        Some(BoyutSeçici::Ad(ad)) if ad == "x" => {
            if let Some(değer) = seri.eşleme.as_ref().and_then(|(x, _)| öğe.boyut(x)) {
                Some(Cow::Borrowed(değer))
            } else {
                saçılım_xy(&öğe.değer, sıra).map(|(x, _)| Cow::Owned(VeriDeğeri::Sayı(x)))
            }
        }
        Some(BoyutSeçici::Ad(ad)) if ad == "y" => {
            if let Some(değer) = seri.eşleme.as_ref().and_then(|(_, y)| öğe.boyut(y)) {
                Some(Cow::Borrowed(değer))
            } else {
                saçılım_xy(&öğe.değer, sıra).map(|(_, y)| Cow::Owned(VeriDeğeri::Sayı(y)))
            }
        }
        Some(BoyutSeçici::Ad(ad)) => öğe.boyut(ad).map(Cow::Borrowed),
    }
}

fn saçılım_görsel_değeri(
    seri: &SaçılımSerisi,
    eşleme: &GörselEşleme,
    sıra: usize,
) -> Option<f64> {
    saçılım_görsel_ham_değeri(seri, eşleme, sıra).and_then(|değer| değer.sayı())
}

/// Seçilen `visualMap.dimension` için scatter veri kapsamı.
pub fn saçılım_görsel_kapsamı(seri: &SaçılımSerisi, eşleme: &GörselEşleme) -> [f64; 2] {
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    for sıra in 0..seri.veri_sayısı() {
        if let Some(değer) = saçılım_görsel_değeri(seri, eşleme, sıra)
            && değer.is_finite()
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

/// `visualMap.inRange.symbolSize` kanallarını yerleşmiş scatter noktalarına
/// uygular. Boyut, isabet alanları ve etiket önceliği tarafından da görülsün
/// diye çizimden önce nokta geometrisine yazılır.
pub fn saçılım_nokta_boyutlarını_eşle(
    seri: &SaçılımSerisi,
    noktalar: &mut [SaçılımNoktası],
    eşlemeler: &[SaçılımGörselEşlemesi<'_>],
) {
    for nokta in noktalar {
        for (eşleme, kapsam) in eşlemeler {
            if let Some(değer) = saçılım_görsel_değeri(seri, eşleme, nokta.sıra) {
                nokta.boyut = eşleme.sembol_boyutu_çöz(değer, *kapsam, nokta.boyut);
            }
        }
    }
}

/// Saçılım serisini çizer; `vurgulu` ipucuyla öne çıkarılan noktadır.
/// `zaman_sn`, sürekli dalga efekti için geçen süredir (saniye).
#[allow(clippy::too_many_arguments)]
fn saçılım_etiketini_yaz(
    çizici: &mut dyn ÇizimYüzeyi,
    metin: &str,
    konum: (f32, f32),
    yatay: YatayHiza,
    dikey: DikeyHiza,
    boyut: f32,
    renk: Renk,
    kalın: bool,
    kontur: Option<Renk>,
    dönüşüm: Option<AfinMatris>,
    bağlı_öteleme: bool,
) {
    // Matrix SymbolDraw etiketleri hücre dönüşümüne bağlı Text öğeleridir.
    // Genel dönüştürülmüş metin tabanı mevcut kartların hinting fazını
    // korur; Matrix'in zrender fazı dolgu için +0,2, kontur için +0,36 px'tir.
    let taban = if bağlı_öteleme {
        if kontur.is_some() { 0.36 } else { 0.2 }
    } else {
        0.0
    };
    match (kontur, dönüşüm) {
        (Some(kontur), Some(dönüşüm)) => {
            çizici.dönüşümlü_konturlu_yazı(
                metin,
                (konum.0, konum.1 + taban),
                yatay,
                dikey,
                boyut,
                renk,
                kalın,
                kontur,
                2.0,
                dönüşüm,
            );
        }
        (Some(kontur), None) => {
            çizici.dönüşümlü_konturlu_yazı(
                metin,
                (0.0, taban),
                yatay,
                dikey,
                boyut,
                renk,
                kalın,
                kontur,
                2.0,
                AfinMatris::ötele(konum.0, konum.1),
            );
        }
        (None, Some(dönüşüm)) => {
            çizici.dönüşümlü_yazı(
                metin,
                (konum.0, konum.1 + taban),
                yatay,
                dikey,
                boyut,
                renk,
                kalın,
                dönüşüm,
            );
        }
        (None, None) if bağlı_öteleme => {
            çizici.dönüşümlü_yazı(
                metin,
                (0.0, taban),
                yatay,
                dikey,
                boyut,
                renk,
                kalın,
                AfinMatris::ötele(konum.0, konum.1),
            );
        }
        (None, None) => {
            çizici.yazı(metin, konum, yatay, dikey, boyut, renk, kalın);
        }
    }
}

#[derive(Clone, Debug)]
struct YerleşimliSaçılımEtiketi {
    nokta: (f32, f32),
    sembol_boyutu: f32,
    metin: String,
    çapa: (f32, f32),
    yatay: YatayHiza,
    dikey: DikeyHiza,
    boyut: f32,
    renk: Renk,
    kalın: bool,
    kontur: Option<Renk>,
    döndürme: EtiketDöndürme,
    metin_kutusu: Dikdörtgen,
    çakışma_kutusu: Dikdörtgen,
    yerleşim: EtiketYerleşimSonucu,
    gizli: bool,
    öncelik: f32,
}

fn saçılım_metin_kutusu(
    çizici: &dyn ÇizimYüzeyi,
    metin: &str,
    çapa: (f32, f32),
    yatay: YatayHiza,
    dikey: DikeyHiza,
    boyut: f32,
) -> Dikdörtgen {
    let satırlar = metin.split('\n').collect::<Vec<_>>();
    let genişlik = satırlar
        .iter()
        .map(|satır| çizici.yazı_ölç(satır, boyut).0)
        .fold(0.0_f32, f32::max);
    let yükseklik = if satırlar.len() == 1 {
        çizici.yazı_ölç(metin, boyut).1
    } else {
        boyut * satırlar.len() as f32
    };
    let x = match yatay {
        YatayHiza::Sol => çapa.0,
        YatayHiza::Orta => çapa.0 - genişlik / 2.0,
        YatayHiza::Sağ => çapa.0 - genişlik,
    };
    let y = match dikey {
        DikeyHiza::Üst => çapa.1,
        DikeyHiza::Orta => çapa.1 - yükseklik / 2.0,
        DikeyHiza::Alt => çapa.1 - yükseklik,
    };
    Dikdörtgen::yeni(x, y, genişlik, yükseklik)
}

fn saçılım_çakışma_kutusu(kutu: Dikdörtgen, en_küçük_boşluk: f32) -> Dikdörtgen {
    let pay = en_küçük_boşluk.max(0.0) / 2.0;
    Dikdörtgen::yeni(
        kutu.x - pay,
        kutu.y - pay,
        kutu.genişlik + pay * 2.0,
        kutu.yükseklik + pay * 2.0,
    )
}

fn saçılım_etiketini_eksende_taşı(
    etiket: &mut YerleşimliSaçılımEtiketi,
    eksen: usize,
    fark: f32,
) {
    if fark.abs() <= f32::EPSILON {
        return;
    }
    if eksen == 0 {
        etiket.çapa.0 += fark;
        etiket.metin_kutusu.x += fark;
        etiket.çakışma_kutusu.x += fark;
    } else {
        etiket.çapa.1 += fark;
        etiket.metin_kutusu.y += fark;
        etiket.çakışma_kutusu.y += fark;
    }
}

fn saçılım_etiket_aralığını_taşı(
    etiketler: &mut [YerleşimliSaçılımEtiketi],
    sıra: &[usize],
    eksen: usize,
    başlangıç: usize,
    bitiş: usize,
    fark: f32,
) {
    for yer in başlangıç..bitiş {
        if let Some(&etiket_sırası) = sıra.get(yer)
            && let Some(etiket) = etiketler.get_mut(etiket_sırası)
        {
            saçılım_etiketini_eksende_taşı(etiket, eksen, fark);
        }
    }
}

// `sıra`, aynı etiket diliminden üretilen geçerli indekslerin sıralı
// görünümüdür; resmi algoritmanın aralık kaydırmalarını indeksle taşımak
// burada dilim eşzamanlı ödünçlerinden daha açıktır.
#[allow(clippy::indexing_slicing, clippy::needless_range_loop)]
fn saçılım_etiket_boşluklarını_sıkıştır(
    etiketler: &mut [YerleşimliSaçılımEtiketi],
    sıra: &[usize],
    eksen: usize,
    fark: f32,
    en_büyük_oran: f32,
) {
    let uzunluk = sıra.len();
    if uzunluk < 2 {
        return;
    }
    let konum = |kutu: Dikdörtgen| if eksen == 0 { kutu.x } else { kutu.y };
    let boyut = |kutu: Dikdörtgen| {
        if eksen == 0 {
            kutu.genişlik
        } else {
            kutu.yükseklik
        }
    };
    let boşluklar = (1..uzunluk)
        .map(|yer| {
            let önceki = etiketler[sıra[yer - 1]].çakışma_kutusu;
            let şimdiki = etiketler[sıra[yer]].çakışma_kutusu;
            (konum(şimdiki) - konum(önceki) - boyut(önceki)).max(0.0)
        })
        .collect::<Vec<_>>();
    let toplam = boşluklar.iter().sum::<f32>();
    if toplam <= f32::EPSILON {
        return;
    }
    let oran = (fark.abs() / toplam).min(en_büyük_oran);
    if fark > 0.0 {
        for yer in 0..uzunluk - 1 {
            saçılım_etiket_aralığını_taşı(
                etiketler,
                sıra,
                eksen,
                0,
                yer + 1,
                boşluklar[yer] * oran,
            );
        }
    } else {
        for yer in (1..uzunluk).rev() {
            saçılım_etiket_aralığını_taşı(
                etiketler,
                sıra,
                eksen,
                yer,
                uzunluk,
                -boşluklar[yer - 1] * oran,
            );
        }
    }
}

/// ECharts `shiftLayoutOnXY`: etiket sırasını korur, önce çakışmaları ileri
/// iter, ardından boşlukları tuval sınırlarına sığacak kadar sıkar.
#[allow(clippy::indexing_slicing)]
fn saçılım_etiketlerini_eksende_kaydır(
    etiketler: &mut [YerleşimliSaçılımEtiketi],
    eksen: usize,
    sınır: f32,
) {
    let kip = if eksen == 0 {
        EtiketÖrtüşmeKaydırması::X
    } else {
        EtiketÖrtüşmeKaydırması::Y
    };
    let mut sıra = etiketler
        .iter()
        .enumerate()
        .filter_map(|(sıra, etiket)| (etiket.yerleşim.örtüşme_kaydırması == kip).then_some(sıra))
        .collect::<Vec<_>>();
    if sıra.len() < 2 {
        return;
    }
    sıra.sort_by(|a, b| {
        let a = etiketler[*a].çakışma_kutusu;
        let b = etiketler[*b].çakışma_kutusu;
        let ak = if eksen == 0 { a.x } else { a.y };
        let bk = if eksen == 0 { b.x } else { b.y };
        ak.partial_cmp(&bk).unwrap_or(std::cmp::Ordering::Equal)
    });
    let konum = |kutu: Dikdörtgen| if eksen == 0 { kutu.x } else { kutu.y };
    let boyut = |kutu: Dikdörtgen| {
        if eksen == 0 {
            kutu.genişlik
        } else {
            kutu.yükseklik
        }
    };

    let mut önceki_bitiş = 0.0_f32;
    for &etiket_sırası in &sıra {
        let kutu = etiketler[etiket_sırası].çakışma_kutusu;
        let fark = konum(kutu) - önceki_bitiş;
        if fark < 0.0 {
            saçılım_etiketini_eksende_taşı(&mut etiketler[etiket_sırası], eksen, -fark);
        }
        let kutu = etiketler[etiket_sırası].çakışma_kutusu;
        önceki_bitiş = konum(kutu) + boyut(kutu);
    }

    let sınır_boşlukları = |etiketler: &[YerleşimliSaçılımEtiketi]| {
        let ilk = etiketler[sıra[0]].çakışma_kutusu;
        let son = etiketler[*sıra.last().unwrap_or(&sıra[0])].çakışma_kutusu;
        (konum(ilk), sınır - konum(son) - boyut(son))
    };
    let (sol_boşluk, sağ_boşluk) = sınır_boşlukları(etiketler);
    if sol_boşluk < 0.0 {
        saçılım_etiket_boşluklarını_sıkıştır(etiketler, &sıra, eksen, -sol_boşluk, 0.8);
    }
    if sağ_boşluk < 0.0 {
        saçılım_etiket_boşluklarını_sıkıştır(etiketler, &sıra, eksen, sağ_boşluk, 0.8);
    }

    for yön in [1.0_f32, -1.0] {
        let (sol_boşluk, sağ_boşluk) = sınır_boşlukları(etiketler);
        let (bu, öteki) = if yön > 0.0 {
            (sol_boşluk, sağ_boşluk)
        } else {
            (sağ_boşluk, sol_boşluk)
        };
        if bu < 0.0 {
            let ötekinden = öteki.min(-bu).max(0.0);
            if ötekinden > 0.0 {
                saçılım_etiket_aralığını_taşı(
                    etiketler,
                    &sıra,
                    eksen,
                    0,
                    sıra.len(),
                    ötekinden * yön,
                );
                let kalan = ötekinden + bu;
                if kalan < 0.0 {
                    saçılım_etiket_boşluklarını_sıkıştır(
                        etiketler,
                        &sıra,
                        eksen,
                        -kalan * yön,
                        1.0,
                    );
                }
            } else {
                saçılım_etiket_boşluklarını_sıkıştır(
                    etiketler,
                    &sıra,
                    eksen,
                    -bu * yön,
                    1.0,
                );
            }
        }
    }

    let (sol_boşluk, sağ_boşluk) = sınır_boşlukları(etiketler);
    for (sınır_sırası, taşma) in [sol_boşluk.min(0.0), sağ_boşluk.min(0.0)]
        .into_iter()
        .enumerate()
    {
        if taşma >= 0.0 {
            continue;
        }
        let yön = if sınır_sırası == 0 { 1.0 } else { -1.0 };
        let mut kalan = -taşma;
        let her_biri = (kalan / (sıra.len() - 1) as f32).ceil();
        for yer in 0..sıra.len() - 1 {
            if yön > 0.0 {
                saçılım_etiket_aralığını_taşı(
                    etiketler,
                    &sıra,
                    eksen,
                    0,
                    yer + 1,
                    her_biri,
                );
            } else {
                saçılım_etiket_aralığını_taşı(
                    etiketler,
                    &sıra,
                    eksen,
                    sıra.len() - yer - 1,
                    sıra.len(),
                    -her_biri,
                );
            }
            kalan -= her_biri;
            if kalan <= 0.0 {
                break;
            }
        }
    }
}

fn saçılım_etiket_kutuları_örtüşüyor(a: Dikdörtgen, b: Dikdörtgen) -> bool {
    const DOKUNMA_EŞİĞİ: f32 = 0.05;
    a.x < b.sağ() - DOKUNMA_EŞİĞİ
        && b.x < a.sağ() - DOKUNMA_EŞİĞİ
        && a.y < b.alt() - DOKUNMA_EŞİĞİ
        && b.y < a.alt() - DOKUNMA_EŞİĞİ
}

#[allow(clippy::indexing_slicing)]
fn çakışan_saçılım_etiketlerini_gizle(etiketler: &mut [YerleşimliSaçılımEtiketi]) {
    let mut sıra = etiketler
        .iter()
        .enumerate()
        .filter_map(|(sıra, etiket)| etiket.yerleşim.çakışanı_gizle.then_some(sıra))
        .collect::<Vec<_>>();
    sıra.sort_by(|a, b| {
        etiketler[*b]
            .öncelik
            .partial_cmp(&etiketler[*a].öncelik)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(b))
    });
    let mut görünen = Vec::new();
    for sıra in sıra {
        let kutu = etiketler[sıra].çakışma_kutusu;
        if görünen
            .iter()
            .any(|kutu2| saçılım_etiket_kutuları_örtüşüyor(*kutu2, kutu))
        {
            etiketler[sıra].gizli = true;
        } else {
            görünen.push(kutu);
        }
    }
}

fn saçılım_etiket_çizgisi_noktaları(
    etiket: &YerleşimliSaçılımEtiketi,
    uzunluk2: f32,
) -> [(f32, f32); 3] {
    let kutu = etiket.metin_kutusu;
    let adaylar = [
        ((kutu.x + kutu.genişlik / 2.0, kutu.y), (0.0_f32, -1.0_f32)),
        ((kutu.sağ(), kutu.y + kutu.yükseklik / 2.0), (1.0, 0.0)),
        ((kutu.x + kutu.genişlik / 2.0, kutu.alt()), (0.0, 1.0)),
        ((kutu.x, kutu.y + kutu.yükseklik / 2.0), (-1.0, 0.0)),
    ];
    let mut en_iyi = [(etiket.nokta.0, etiket.nokta.1); 3];
    let mut en_kısa = f32::INFINITY;
    for (etiket_ucu, yön) in adaylar {
        let dirsek = (
            etiket_ucu.0 + yön.0 * uzunluk2,
            etiket_ucu.1 + yön.1 * uzunluk2,
        );
        let dx = dirsek.0 - etiket.nokta.0;
        let dy = dirsek.1 - etiket.nokta.1;
        let uzaklık = (dx * dx + dy * dy).sqrt();
        let yarıçap = etiket.sembol_boyutu / 2.0;
        let sembol_ucu = if uzaklık > f32::EPSILON {
            (
                etiket.nokta.0 + dx / uzaklık * yarıçap,
                etiket.nokta.1 + dy / uzaklık * yarıçap,
            )
        } else {
            etiket.nokta
        };
        let açıklık = (uzaklık - yarıçap).max(0.0);
        if açıklık < en_kısa {
            en_kısa = açıklık;
            en_iyi = [sembol_ucu, dirsek, etiket_ucu];
        }
    }
    en_iyi
}

fn yerleşimli_saçılım_etiketini_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    etiket: &YerleşimliSaçılımEtiketi,
    bağlı_öteleme: bool,
) {
    let satırlar = etiket.metin.split('\n').collect::<Vec<_>>();
    if satırlar.len() == 1 {
        match etiket.döndürme {
            EtiketDöndürme::Derece(derece) if derece.abs() > f32::EPSILON => {
                saçılım_etiketini_yaz(
                    çizici,
                    &etiket.metin,
                    (0.0, 0.0),
                    etiket.yatay,
                    etiket.dikey,
                    etiket.boyut,
                    etiket.renk,
                    etiket.kalın,
                    etiket.kontur,
                    Some(
                        AfinMatris::ötele(etiket.çapa.0, etiket.çapa.1)
                            .çarp(AfinMatris::döndür(-derece.to_radians())),
                    ),
                    bağlı_öteleme,
                );
            }
            _ => saçılım_etiketini_yaz(
                çizici,
                &etiket.metin,
                etiket.çapa,
                etiket.yatay,
                etiket.dikey,
                etiket.boyut,
                etiket.renk,
                etiket.kalın,
                etiket.kontur,
                None,
                bağlı_öteleme,
            ),
        }
        return;
    }

    let toplam_yükseklik = etiket.boyut * satırlar.len() as f32;
    let ilk_satır_y = match etiket.dikey {
        DikeyHiza::Üst => etiket.boyut / 2.0,
        DikeyHiza::Orta => -toplam_yükseklik / 2.0 + etiket.boyut / 2.0,
        DikeyHiza::Alt => -toplam_yükseklik + etiket.boyut / 2.0,
    };
    let dönüşüm = match etiket.döndürme {
        EtiketDöndürme::Derece(derece) if derece.abs() > f32::EPSILON => Some(
            AfinMatris::ötele(etiket.çapa.0, etiket.çapa.1)
                .çarp(AfinMatris::döndür(-derece.to_radians())),
        ),
        _ => None,
    };
    for (satır_sırası, satır) in satırlar.into_iter().enumerate() {
        if satır.is_empty() {
            continue;
        }
        let y = ilk_satır_y + satır_sırası as f32 * etiket.boyut;
        if let Some(dönüşüm) = dönüşüm {
            saçılım_etiketini_yaz(
                çizici,
                satır,
                (0.0, y),
                etiket.yatay,
                DikeyHiza::Orta,
                etiket.boyut,
                etiket.renk,
                etiket.kalın,
                etiket.kontur,
                Some(dönüşüm),
                bağlı_öteleme,
            );
        } else {
            saçılım_etiketini_yaz(
                çizici,
                satır,
                (etiket.çapa.0, etiket.çapa.1 + y),
                etiket.yatay,
                DikeyHiza::Orta,
                etiket.boyut,
                etiket.renk,
                etiket.kalın,
                etiket.kontur,
                None,
                bağlı_öteleme,
            );
        }
    }
}

fn yerleşimli_saçılım_etiketlerini_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &SaçılımSerisi,
    noktalar: &[SaçılımNoktası],
    opaklık: f32,
    nokta_rengi: &dyn Fn(&SaçılımNoktası) -> Renk,
    bağlı_öteleme: bool,
) {
    let mut etiketler = Vec::new();
    for nokta in noktalar {
        let renk = nokta_rengi(nokta);
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
            VeriDeğeri::KarmaDizi(değerler) => değerler
                .iter()
                .filter_map(|değer| match değer {
                    VeriDeğeri::Boş => None,
                    VeriDeğeri::Sayı(değer) => Some(ondalık_kırp(*değer)),
                    VeriDeğeri::Metin(metin) => Some(metin.clone()),
                    VeriDeğeri::Zaman(ms) => Some(ms.to_string()),
                    VeriDeğeri::Mantıksal(değer) => Some(değer.to_string()),
                    VeriDeğeri::Çift([x, y]) => {
                        Some(format!("{},{}", ondalık_kırp(*x), ondalık_kırp(*y)))
                    }
                    VeriDeğeri::Dizi(değerler) => Some(
                        değerler
                            .iter()
                            .map(|değer| ondalık_kırp(*değer))
                            .collect::<Vec<_>>()
                            .join(","),
                    ),
                    VeriDeğeri::KarmaDizi(_) => None,
                })
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
        let (mut çapa, doğal_yatay, doğal_dikey) = match etiket.konum {
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
        çapa.0 += etiket.kayma.0;
        çapa.1 += etiket.kayma.1;
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
        let (etiket_rengi, etiket_konturu) = match etiket.yazı.renk {
            Some(renk) => (renk, None),
            None if etiket.konum == EtiketKonumu::İç => {
                let (metin, kontur) = seri
                    .öğe_stili
                    .renk
                    .as_ref()
                    .map(|dolgu| dolgu.zrender_iç_etiket_stili(tema::koyu_mu()))
                    .unwrap_or_else(|| renk.zrender_iç_etiket_stili(tema::koyu_mu()));
                (
                    metin.opaklık(opaklık),
                    kontur.map(|kontur| kontur.opaklık(opaklık)),
                )
            }
            None => (tema::birincil_metin().opaklık(opaklık), None),
        };
        let metin_kutusu = saçılım_metin_kutusu(çizici, &metin, çapa, yatay, dikey, boyut);
        let çakışma_kutusu = saçılım_çakışma_kutusu(metin_kutusu, etiket.en_küçük_boşluk);
        let mut aday = YerleşimliSaçılımEtiketi {
            nokta: nokta.konum,
            sembol_boyutu: nokta.boyut,
            metin,
            çapa,
            yatay,
            dikey,
            boyut,
            renk: etiket_rengi,
            kalın: etiket.yazı.kalın,
            kontur: etiket_konturu,
            döndürme: etiket.döndürme,
            metin_kutusu,
            çakışma_kutusu,
            yerleşim: EtiketYerleşimSonucu::default(),
            gizli: false,
            öncelik: nokta.boyut * nokta.boyut,
        };
        if let Some(işlev) = &seri.etiket_yerleşimi {
            let doğal_çizgi = seri
                .etiket_çizgisi
                .as_ref()
                .filter(|çizgi| çizgi.göster)
                .map(|çizgi| saçılım_etiket_çizgisi_noktaları(&aday, çizgi.uzunluk2));
            let sonuç = işlev.uygula(&EtiketYerleşimParametreleri {
                veri_sırası: nokta.sıra,
                veri_adı: öğe.ad.clone().unwrap_or_default(),
                değer: biçim_değeri,
                etiket_kutusu: metin_kutusu,
                etiket_çizgisi_noktaları: doğal_çizgi,
            });
            if let Some(x) = sonuç.x {
                aday.çapa.0 = x;
            }
            if let Some(y) = sonuç.y {
                aday.çapa.1 = y;
            }
            // LabelManager mutlak x/y verildiğinde etiketi sembolün bağlı
            // `textConfig.position`ından ayırır; zrender'ın serbest metin
            // kutusu açık verticalAlign yoksa verilen y'den aşağı akar.
            if (sonuç.x.is_some() || sonuç.y.is_some()) && sonuç.dikey_hiza.is_none() {
                aday.dikey = DikeyHiza::Üst;
            }
            if let Some(hiza) = sonuç.yatay_hiza {
                aday.yatay = match hiza {
                    YazıYatayHizası::Sol => YatayHiza::Sol,
                    YazıYatayHizası::Orta => YatayHiza::Orta,
                    YazıYatayHizası::Sağ => YatayHiza::Sağ,
                };
            }
            if let Some(hiza) = sonuç.dikey_hiza {
                aday.dikey = match hiza {
                    YazıDikeyHizası::Üst => DikeyHiza::Üst,
                    YazıDikeyHizası::Orta => DikeyHiza::Orta,
                    YazıDikeyHizası::Alt => DikeyHiza::Alt,
                };
            }
            aday.metin_kutusu = saçılım_metin_kutusu(
                çizici,
                &aday.metin,
                aday.çapa,
                aday.yatay,
                aday.dikey,
                aday.boyut,
            );
            aday.çakışma_kutusu =
                saçılım_çakışma_kutusu(aday.metin_kutusu, etiket.en_küçük_boşluk);
            aday.yerleşim = sonuç;
        }
        etiketler.push(aday);
    }

    saçılım_etiketlerini_eksende_kaydır(etiketler.as_mut_slice(), 0, çizici.genişlik());
    saçılım_etiketlerini_eksende_kaydır(etiketler.as_mut_slice(), 1, çizici.yükseklik());
    çakışan_saçılım_etiketlerini_gizle(&mut etiketler);

    if let Some(çizgi) = seri.etiket_çizgisi.as_ref().filter(|çizgi| çizgi.göster) {
        for etiket in etiketler.iter().filter(|etiket| !etiket.gizli) {
            let noktalar = etiket
                .yerleşim
                .etiket_çizgisi_noktaları
                .unwrap_or_else(|| saçılım_etiket_çizgisi_noktaları(etiket, çizgi.uzunluk2));
            let mut yol = Yol::yeni();
            yol.taşı(noktalar[0]);
            yol.çiz(noktalar[1]);
            yol.çiz(noktalar[2]);
            çizici.yol_çiz(
                &yol,
                çizgi.stil.kalınlık,
                çizgi
                    .stil
                    .renk
                    .unwrap_or(etiket.renk)
                    .opaklık(çizgi.stil.opaklık),
                çizgi.stil.tür,
            );
        }
    }
    for etiket in etiketler.iter().filter(|etiket| !etiket.gizli) {
        yerleşimli_saçılım_etiketini_çiz(çizici, etiket, bağlı_öteleme);
    }
}

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
    saçılım_çiz_eşlemeli(
        çizici,
        seri,
        noktalar,
        seri_rengi,
        ilerleme,
        zaman_sn,
        vurgulu,
        None,
        &tema::PALET,
    );
}

/// [`saçılım_çiz`] ile aynı çizimi, varsa `visualMap` rengini her noktanın
/// ikinci veri boyutuna ayrı ayrı uygulayarak gerçekleştirir.
#[allow(clippy::too_many_arguments)]
pub fn saçılım_çiz_eşlemeli(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &SaçılımSerisi,
    noktalar: &[SaçılımNoktası],
    seri_rengi: Renk,
    ilerleme: f32,
    zaman_sn: f32,
    vurgulu: Option<usize>,
    görsel_eşleme: Option<(&GörselEşleme, [f64; 2])>,
    palet: &[Renk],
) {
    let eşlemeler = görsel_eşleme.into_iter().collect::<Vec<_>>();
    saçılım_çiz_çoklu_eşlemeli(
        çizici,
        seri,
        noktalar,
        seri_rengi,
        ilerleme,
        zaman_sn,
        vurgulu,
        &eşlemeler,
        palet,
    );
}

/// Büyük saçılım noktalarını ECharts'ın küçük sembol Canvas hızlandırmasıyla
/// boyar. `LargeSymbolDraw` kenarlığı/öğe durumlarını bilerek atlar ve bütün
/// noktalar için tek seri dolgusu kullanır. Büyük dizi, `progressive`
/// boyutunda parçalara ayrılarak yüzeye aktarılır; tek karelik başsız çizimde
/// parçalar ardışık tamamlanır.
pub fn büyük_saçılım_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &SaçılımSerisi,
    noktalar: &BüyükSaçılımNoktaları,
    seri_rengi: Renk,
    ilerleme: f32,
) {
    let opaklık = seri.öğe_stili.opaklık.unwrap_or(0.8);
    let dolgu = seri
        .öğe_stili
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(seri_rengi))
        .opaklık(opaklık);
    let boyut = noktalar.boyut * ilerleme.clamp(0.0, 1.0);
    if boyut <= 0.0 {
        return;
    }
    let parça_nokta_sayısı = if seri.veri_sayısı() >= seri.aşamalı_eşiği {
        seri.aşamalı.max(1)
    } else {
        seri.veri_sayısı().max(1)
    };
    let parça_değer_sayısı = parça_nokta_sayısı.saturating_mul(2).max(2);
    for parça in noktalar.konumlar.chunks(parça_değer_sayısı) {
        çizici.büyük_saçılım_noktaları(parça, boyut, &dolgu);
    }
}

/// Birden çok `visualMap` bileşeninin bağımsız renk, açıklık, opaklık ve
/// sembol boyutu kanallarını ECharts görsel kodlama sırasıyla birleştirir.
#[allow(clippy::too_many_arguments)]
pub fn saçılım_çiz_çoklu_eşlemeli(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &SaçılımSerisi,
    noktalar: &[SaçılımNoktası],
    seri_rengi: Renk,
    ilerleme: f32,
    zaman_sn: f32,
    vurgulu: Option<usize>,
    görsel_eşlemeler: &[SaçılımGörselEşlemesi<'_>],
    palet: &[Renk],
) {
    saçılım_çiz_çoklu_eşlemeli_kipli(
        çizici,
        seri,
        noktalar,
        seri_rengi,
        ilerleme,
        zaman_sn,
        vurgulu,
        görsel_eşlemeler,
        palet,
        false,
    );
}

/// Matrix hücresine bağlı SymbolDraw metninin saf öteleme/raster fazını
/// koruyan dahili çizim girişi.
#[allow(clippy::too_many_arguments)]
pub(crate) fn matris_saçılım_çiz_çoklu_eşlemeli(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &SaçılımSerisi,
    noktalar: &[SaçılımNoktası],
    seri_rengi: Renk,
    ilerleme: f32,
    zaman_sn: f32,
    vurgulu: Option<usize>,
    görsel_eşlemeler: &[SaçılımGörselEşlemesi<'_>],
    palet: &[Renk],
) {
    saçılım_çiz_çoklu_eşlemeli_kipli(
        çizici,
        seri,
        noktalar,
        seri_rengi,
        ilerleme,
        zaman_sn,
        vurgulu,
        görsel_eşlemeler,
        palet,
        true,
    );
}

#[allow(clippy::too_many_arguments)]
fn saçılım_çiz_çoklu_eşlemeli_kipli(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &SaçılımSerisi,
    noktalar: &[SaçılımNoktası],
    seri_rengi: Renk,
    ilerleme: f32,
    zaman_sn: f32,
    vurgulu: Option<usize>,
    görsel_eşlemeler: &[SaçılımGörselEşlemesi<'_>],
    palet: &[Renk],
    bağlı_öteleme: bool,
) {
    let ilerleme = if seri.animasyon_eşiğinde_mi() {
        ilerleme
    } else {
        1.0
    };
    // `scatter` öntanımlı 0.8, `effectScatter` ise 1.0 opaklıktadır.
    let opaklık = seri
        .öğe_stili
        .opaklık
        .unwrap_or(if seri.efektli { 1.0 } else { 0.8 });
    let görsel_renk_kanalı = görsel_eşlemeler.iter().any(|(eşleme, _)| {
        eşleme.renk_kanalı
            || eşleme.renk_açıklığı.is_some()
            || eşleme.opaklık.is_some()
            || eşleme.aralık_dışı_renk_kanalı
    });
    // `itemStyle.color(value, params)` ECharts görsel kodlama aşamasında bir
    // kez çözülür. Sonuçları özgün dataIndex ile önbelleğe almak callback'in
    // etiket ve vurgu geçişlerinde yeniden çalışmasını da önler.
    let mut nokta_görselleri: Vec<Option<(Renk, Option<Dolgu>)>> = vec![None; seri.veri.len()];
    for nokta in noktalar {
        let Some(öğe) = seri.veri.get(nokta.sıra) else {
            continue;
        };
        let dolgu = öğe
            .stil
            .as_ref()
            .and_then(|stil| stil.renk.clone())
            .or_else(|| {
                seri.öğe_rengi_işlevi
                    .as_ref()
                    .map(|işlev| işlev.çöz(öğe, nokta.sıra))
            })
            .or_else(|| seri.öğe_stili.renk.clone());
        let mut renk = dolgu.as_ref().map(Dolgu::temsilî).unwrap_or_else(|| {
            if seri.veriye_göre_renk {
                let palet_sırası = nokta.palet_sırası.unwrap_or(nokta.sıra);
                palet
                    .get(palet_sırası % palet.len().max(1))
                    .copied()
                    .unwrap_or_else(|| tema::palet_rengi(palet_sırası))
            } else {
                seri_rengi
            }
        });
        for (eşleme, kapsam) in görsel_eşlemeler {
            if let Some(değer) = saçılım_görsel_ham_değeri(seri, eşleme, nokta.sıra) {
                if eşleme.kategorik_mi() {
                    renk = eşleme.kategori_rengi_uygula(&değer, renk);
                } else if let Some(sayısal) = değer.sayı() {
                    renk = eşleme.rengi_uygula(sayısal, *kapsam, renk);
                }
            }
        }
        let dolgu = if görsel_renk_kanalı {
            Some(Dolgu::Düz(renk))
        } else {
            dolgu
        };
        if let Some(görsel) = nokta_görselleri.get_mut(nokta.sıra) {
            *görsel = Some((renk, dolgu));
        }
    }
    let nokta_rengi = |nokta: &SaçılımNoktası| {
        nokta_görselleri
            .get(nokta.sıra)
            .and_then(Option::as_ref)
            .map(|(renk, _)| *renk)
            .unwrap_or(seri_rengi)
    };
    // EffectSymbol çekirdeği önce, z2=99 dalgaları sonra boyar.
    for nokta in noktalar {
        let vurgulu_mu = vurgulu == Some(nokta.sıra);
        let boyut = nokta.boyut * ilerleme.clamp(0.0, 1.0) * if vurgulu_mu { 1.15 } else { 1.0 };
        let öğe_stili = seri.veri.get(nokta.sıra).and_then(|öğe| öğe.stil.as_ref());
        let mut dolgu = nokta_görselleri
            .get(nokta.sıra)
            .and_then(Option::as_ref)
            .and_then(|(_, dolgu)| dolgu.clone());
        if vurgulu_mu && let Some(vurgu_dolgusu) = &seri.vurgu_öğe_stili.renk {
            dolgu = Some(vurgu_dolgusu.clone());
        }
        let renk = dolgu
            .as_ref()
            .map(Dolgu::temsilî)
            .unwrap_or_else(|| nokta_rengi(nokta));
        let nokta_opaklığı = if vurgulu_mu {
            seri.vurgu_öğe_stili.opaklık.unwrap_or(1.0)
        } else {
            öğe_stili.and_then(|stil| stil.opaklık).unwrap_or(opaklık)
        };
        if let Some(gölge_rengi) = seri.öğe_stili.gölge_rengi
            && (seri.öğe_stili.gölge_bulanıklığı > 0.0
                || seri.öğe_stili.gölge_kayması != (0.0, 0.0))
            && let Some(yol) = crate::grafik::sembol_yolu(
                &seri.sembol,
                nokta.konum,
                boyut,
                seri.sembol_oranını_koru,
            )
        {
            çizici.yol_gölgesi(
                &yol,
                gölge_rengi.opaklık(nokta_opaklığı),
                seri.öğe_stili.gölge_bulanıklığı,
                seri.öğe_stili.gölge_kayması,
            );
        }
        let kenarlık_stili = if vurgulu_mu
            && (seri.vurgu_öğe_stili.kenarlık_kalınlığı > 0.0
                || seri.vurgu_öğe_stili.kenarlık_rengi.is_some())
        {
            &seri.vurgu_öğe_stili
        } else {
            &seri.öğe_stili
        };
        let kenarlık = (kenarlık_stili.kenarlık_kalınlığı > 0.0).then(|| {
            (
                kenarlık_stili.kenarlık_kalınlığı,
                kenarlık_stili.kenarlık_rengi.unwrap_or(renk),
            )
        });
        sembol_stilli_çiz(
            çizici,
            &seri.sembol,
            nokta.konum,
            boyut,
            renk,
            dolgu.as_ref(),
            kenarlık,
            nokta_opaklığı,
            seri.sembol_oranını_koru,
        );
    }

    // Dataset `encode.label` dâhil saçılım etiketleri. Öğe yaması seri
    // etiketini miras alır; açık align/verticalAlign/rotate değerleri
    // zrender bağlı metin yerleşimine aktarılır.
    if seri.etiket_yerleşimi.is_some() || seri.etiket_çizgisi.is_some() {
        yerleşimli_saçılım_etiketlerini_çiz(
            çizici,
            seri,
            noktalar,
            opaklık,
            &nokta_rengi,
            bağlı_öteleme,
        );
    } else {
        for nokta in noktalar {
            let renk = nokta_rengi(nokta);
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
                VeriDeğeri::KarmaDizi(değerler) => değerler
                    .iter()
                    .filter_map(|değer| match değer {
                        VeriDeğeri::Boş => None,
                        VeriDeğeri::Sayı(değer) => Some(ondalık_kırp(*değer)),
                        VeriDeğeri::Metin(metin) => Some(metin.clone()),
                        VeriDeğeri::Zaman(ms) => Some(ms.to_string()),
                        VeriDeğeri::Mantıksal(değer) => Some(değer.to_string()),
                        VeriDeğeri::Çift([x, y]) => {
                            Some(format!("{},{}", ondalık_kırp(*x), ondalık_kırp(*y)))
                        }
                        VeriDeğeri::Dizi(değerler) => Some(
                            değerler
                                .iter()
                                .map(|değer| ondalık_kırp(*değer))
                                .collect::<Vec<_>>()
                                .join(","),
                        ),
                        VeriDeğeri::KarmaDizi(_) => None,
                    })
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
            let (mut çapa, doğal_yatay, doğal_dikey) = match etiket.konum {
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
            çapa.0 += etiket.kayma.0;
            çapa.1 += etiket.kayma.1;
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
            // SymbolDraw, iç etikette açık renk yokken path dolgusuna göre
            // otomatik karşıt renk ve gerektiğinde 2 px kontur kullanır.
            let (etiket_rengi, etiket_konturu) = match etiket.yazı.renk {
                Some(renk) => (renk, None),
                None if etiket.konum == EtiketKonumu::İç => {
                    let (metin, kontur) = seri
                        .öğe_stili
                        .renk
                        .as_ref()
                        .map(|dolgu| dolgu.zrender_iç_etiket_stili(tema::koyu_mu()))
                        .unwrap_or_else(|| renk.zrender_iç_etiket_stili(tema::koyu_mu()));
                    (
                        metin.opaklık(opaklık),
                        kontur.map(|kontur| kontur.opaklık(opaklık)),
                    )
                }
                None => (tema::birincil_metin().opaklık(opaklık), None),
            };
            let satırlar = metin.split('\n').collect::<Vec<_>>();
            if satırlar.len() == 1 {
                match etiket.döndürme {
                    EtiketDöndürme::Derece(derece) if derece.abs() > f32::EPSILON => {
                        saçılım_etiketini_yaz(
                            çizici,
                            &metin,
                            (0.0, 0.0),
                            yatay,
                            dikey,
                            boyut,
                            etiket_rengi,
                            etiket.yazı.kalın,
                            etiket_konturu,
                            Some(
                                AfinMatris::ötele(çapa.0, çapa.1)
                                    .çarp(AfinMatris::döndür(-derece.to_radians())),
                            ),
                            bağlı_öteleme,
                        );
                    }
                    _ => {
                        saçılım_etiketini_yaz(
                            çizici,
                            &metin,
                            çapa,
                            yatay,
                            dikey,
                            boyut,
                            etiket_rengi,
                            etiket.yazı.kalın,
                            etiket_konturu,
                            None,
                            bağlı_öteleme,
                        );
                    }
                }
                continue;
            }

            // zrender düz metinde öntanımlı lineHeight olarak font boyutunu
            // kullanır ve sondaki boş satırları da blok yüksekliğine katar.
            let toplam_yükseklik = boyut * satırlar.len() as f32;
            let ilk_satır_y = match dikey {
                DikeyHiza::Üst => boyut / 2.0,
                DikeyHiza::Orta => -toplam_yükseklik / 2.0 + boyut / 2.0,
                DikeyHiza::Alt => -toplam_yükseklik + boyut / 2.0,
            };
            let dönüşüm = match etiket.döndürme {
                EtiketDöndürme::Derece(derece) if derece.abs() > f32::EPSILON => Some(
                    AfinMatris::ötele(çapa.0, çapa.1)
                        .çarp(AfinMatris::döndür(-derece.to_radians())),
                ),
                _ => None,
            };
            for (satır_sırası, satır) in satırlar.into_iter().enumerate() {
                if satır.is_empty() {
                    continue;
                }
                let y = ilk_satır_y + satır_sırası as f32 * boyut;
                if let Some(dönüşüm) = dönüşüm {
                    saçılım_etiketini_yaz(
                        çizici,
                        satır,
                        (0.0, y),
                        yatay,
                        DikeyHiza::Orta,
                        boyut,
                        etiket_rengi,
                        etiket.yazı.kalın,
                        etiket_konturu,
                        Some(dönüşüm),
                        bağlı_öteleme,
                    );
                } else {
                    saçılım_etiketini_yaz(
                        çizici,
                        satır,
                        (çapa.0, çapa.1 + y),
                        yatay,
                        DikeyHiza::Orta,
                        boyut,
                        etiket_rengi,
                        etiket.yazı.kalın,
                        etiket_konturu,
                        None,
                        bağlı_öteleme,
                    );
                }
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
            let renk = nokta_rengi(nokta);
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
    use crate::cizim::KayıtYüzeyi;
    use crate::koordinat::{Dikdörtgen, ÇalışmaEkseni};
    use crate::model::eksen::{Eksen, EksenKonumu};
    use crate::model::stil::Etiket;
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
    fn düz_float32_veri_data_index_ve_xy_değerlerini_korur() {
        let seri = SaçılımSerisi::yeni()
            .veri([[99.0, 99.0]])
            .düz_veri(vec![1.25_f32, 2.5, 3.75, 4.5, 9.0]);

        assert!(seri.veri.is_empty());
        assert_eq!(seri.veri_sayısı(), 2);
        assert_eq!(seri.xy(0), Some((1.25, 2.5)));
        assert_eq!(seri.xy(1), Some((3.75, 4.5)));
        assert_eq!(seri.xy(2), None);
    }

    #[test]
    fn scatter_animation_threshold_esigin_tam_ustunde_gecisi_kapatir() {
        let eşikte = SaçılımSerisi::yeni()
            .animasyon_eşiği(2)
            .veri([[1.0, 1.0], [2.0, 2.0]]);
        let üstünde =
            SaçılımSerisi::yeni()
                .animasyon_eşiği(2)
                .veri([[1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]);

        assert!(eşikte.animasyon_eşiğinde_mi());
        assert!(!üstünde.animasyon_eşiğinde_mi());
    }

    #[test]
    fn büyük_scatter_kırpılan_sırayı_nan_ile_korur_ve_parçalı_toplu_çizer() {
        let seri = SaçılımSerisi::yeni()
            .düz_veri(vec![1.0_f32, 1.0, 2.0, 2.0, 20.0, 20.0])
            .sembol_boyutu(3.0)
            .öğe_stili(crate::model::stil::ÖğeStili::yeni().opaklık(0.4))
            .büyük(true)
            .büyük_eşiği(2)
            .aşamalı_eşiği(2)
            .aşamalı(2);
        let kartezyen = Kartezyen2B {
            x: değer_ekseni([0.0, 10.0], [0.0, 100.0], EksenKonumu::Alt),
            y: değer_ekseni([0.0, 10.0], [100.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        };

        let noktalar = büyük_saçılım_noktaları(&seri, &kartezyen).expect("büyük yol kurulmalı");
        assert_eq!(noktalar.konumlar.len(), 6);
        assert_eq!(&noktalar.konumlar[..4], &[10.0, 90.0, 20.0, 80.0]);
        assert!(noktalar.konumlar[4].is_nan());
        assert!(noktalar.konumlar[5].is_nan());

        let mut yüzey = KayıtYüzeyi::yeni(100.0, 100.0);
        büyük_saçılım_çiz(&mut yüzey, &seri, &noktalar, Renk::onaltılık(0x5470c6), 1.0);
        let döküm = yüzey.döküm();
        let komutlar = döküm
            .lines()
            .filter(|satır| satır.starts_with("büyük-saçılım"))
            .collect::<Vec<_>>();
        assert_eq!(komutlar.len(), 1);
        assert!(komutlar[0].contains("adet=2"));
        assert!(komutlar[0].contains("@0.4"));
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
    fn çok_boyutlu_scatter_ilk_iki_boyutu_koordinata_kalanını_sembole_aktarır() {
        let seri = SaçılımSerisi::yeni()
            .sembol_boyutu_işlevi(|öğe| {
                öğe
                    .değer
                    .dizi()
                    .and_then(|değerler| değerler.get(2))
                    .copied()
                    .unwrap_or_default() as f32
                    * 2.0
            })
            .veri([[3.0, 2.0, 7.0]]);
        let kartezyen = Kartezyen2B {
            x: değer_ekseni([0.0, 6.0], [0.0, 120.0], EksenKonumu::Alt),
            y: değer_ekseni([0.0, 4.0], [80.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 120.0, 80.0),
        };

        let noktalar = saçılım_noktaları(&seri, &kartezyen);

        assert_eq!(noktalar.len(), 1);
        assert_eq!(noktalar[0].konum, (60.0, 40.0));
        assert_eq!(noktalar[0].x_değeri, 3.0);
        assert_eq!(noktalar[0].y_değeri, 2.0);
        assert_eq!(noktalar[0].boyut, 14.0);
        assert_eq!(saçılım_değer_kapsamı(&seri), [2.0, 2.0]);
    }

    #[test]
    fn scatter_callback_bağlamı_özgün_data_index_sırasını_taşır() {
        let boyutlar = [6.0_f32, 14.0, 22.0];
        let renkler = [
            Renk::onaltılık(0xaa0000),
            Renk::onaltılık(0x00aa00),
            Renk::onaltılık(0x0000aa),
        ];
        let seri = SaçılımSerisi::yeni()
            .sembol_boyutu_bağlamlı_işlevi(move |_, bağlam| boyutlar[bağlam.veri_sırası])
            .öğe_rengi_işlevi(move |_, bağlam| renkler[bağlam.veri_sırası])
            .veri([[1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]);
        let kartezyen = Kartezyen2B {
            x: değer_ekseni([0.0, 4.0], [0.0, 100.0], EksenKonumu::Alt),
            y: değer_ekseni([0.0, 4.0], [100.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        };

        let noktalar = saçılım_noktaları(&seri, &kartezyen);
        let renk_işlevi = seri
            .öğe_rengi_işlevi
            .as_ref()
            .expect("renk callback'i korunmalı");

        assert_eq!(
            noktalar.iter().map(|nokta| nokta.boyut).collect::<Vec<_>>(),
            boyutlar
        );
        assert_eq!(renk_işlevi.çöz(&seri.veri[2], 2).temsilî(), renkler[2]);
    }

    #[test]
    fn visual_map_sayısal_boyutu_scatter_sembol_çapına_uygulanır() {
        let seri =
            SaçılımSerisi::yeni().veri([[1.0, 20.0, 0.0], [2.0, 40.0, 125.0], [3.0, 60.0, 250.0]]);
        let kartezyen = Kartezyen2B {
            x: değer_ekseni([0.0, 4.0], [0.0, 100.0], EksenKonumu::Alt),
            y: değer_ekseni([0.0, 80.0], [100.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        };
        let eşleme = GörselEşleme::yeni()
            .boyut(2usize)
            .en_az(0.0)
            .en_çok(250.0)
            .sembol_boyutu(10.0, 70.0);
        let mut noktalar = saçılım_noktaları(&seri, &kartezyen);
        let kapsam = saçılım_görsel_kapsamı(&seri, &eşleme);

        saçılım_nokta_boyutlarını_eşle(&seri, &mut noktalar, &[(&eşleme, kapsam)]);

        assert_eq!(kapsam, [0.0, 250.0]);
        assert_eq!(noktalar[0].boyut, 10.0);
        assert_eq!(noktalar[1].boyut, 40.0);
        assert_eq!(noktalar[2].boyut, 70.0);
    }

    #[test]
    fn çoklu_visual_map_boyut_ve_açıklık_kanallarını_ayrı_boyutlardan_okur() {
        let seri = SaçılımSerisi::yeni().veri([vec![1.0, 20.0, 125.0, 0.0, 0.0, 0.0, 25.0]]);
        let boyut = GörselEşleme::yeni()
            .boyut(2usize)
            .en_az(0.0)
            .en_çok(250.0)
            .sembol_boyutu(10.0, 70.0);
        let açıklık = GörselEşleme::yeni()
            .boyut(6usize)
            .en_az(0.0)
            .en_çok(50.0)
            .renk_açıklığı(0.9, 0.5);
        let taban = Renk::onaltılık(0xdd4444);

        let boyut_değeri = saçılım_görsel_değeri(&seri, &boyut, 0).unwrap();
        let açıklık_değeri = saçılım_görsel_değeri(&seri, &açıklık, 0).unwrap();
        let renk = açıklık.rengi_uygula(açıklık_değeri, [0.0, 50.0], taban);

        assert_eq!(boyut_değeri, 125.0);
        assert_eq!(açıklık_değeri, 25.0);
        assert!((boyut.sembol_boyutu_çöz(boyut_değeri, [0.0, 250.0], 10.0) - 40.0).abs() < 1e-6);
        assert_ne!(renk, taban);
        assert!(renk.kırmızı > renk.yeşil && renk.yeşil >= renk.mavi);
    }

    #[test]
    fn kategorik_visual_map_metin_boyutunu_renge_ardindan_acikliga_aktarir() {
        let seri = SaçılımSerisi::yeni().veri([VeriÖğesi::yeni([10.0, 20.0]).boyutlar([
            ("protein".to_owned(), 10.0.into()),
            ("calcium".to_owned(), 20.0.into()),
            ("group".to_owned(), "Dairy".into()),
            ("index".to_owned(), 50.0.into()),
        ])]);
        let kategori = GörselEşleme::yeni()
            .boyut(2usize)
            .kategoriler(["Dairy", "Spices"])
            .renkler([0xdf5a5au32, 0xdf775a]);
        let açıklık = GörselEşleme::yeni()
            .boyut(3usize)
            .en_çok(100.0)
            .renk_açıklığı(0.15, 0.6);

        let ham = saçılım_görsel_ham_değeri(&seri, &kategori, 0).unwrap();
        assert_eq!(&*ham, &VeriDeğeri::Metin("Dairy".to_owned()));
        assert_eq!(saçılım_görsel_kapsamı(&seri, &kategori), [0.0, 1.0]);
        let grup_rengi = kategori.kategori_rengi_uygula(&ham, Renk::SİYAH);
        assert_eq!(grup_rengi, Renk::onaltılık(0xdf5a5a));
        let son = açıklık.rengi_uygula(50.0, [0.0, 100.0], grup_rengi);
        assert_eq!(son, grup_rengi.açıklık_ile(0.375));

        let noktalar = [SaçılımNoktası {
            sıra: 0,
            konum: (50.0, 50.0),
            boyut: 10.0,
            x_değeri: 10.0,
            y_değeri: 20.0,
            palet_sırası: None,
        }];
        let mut yüzey = KayıtYüzeyi::yeni(100.0, 100.0);
        saçılım_çiz_çoklu_eşlemeli(
            &mut yüzey,
            &seri,
            &noktalar,
            Renk::onaltılık(0x5470c6),
            1.0,
            0.0,
            None,
            &[(&kategori, [0.0, 1.0]), (&açıklık, [0.0, 100.0])],
            &tema::PALET,
        );
        assert!(yüzey.döküm().contains("#a01f1f@0.8"), "{}", yüzey.döküm());
    }

    #[test]
    fn kategori_ekseni_titremesi_bant_icinde_ve_yeniden_boyamada_sabittir() {
        let seri = SaçılımSerisi::yeni().veri([[0.0, 1.0], [0.0, 2.0]]);
        let kartezyen = Kartezyen2B {
            x: ÇalışmaEkseni::yeni(
                Eksen::kategori().titreme(20.0),
                Ölçek::Kategorik(KategorikÖlçek::yeni(vec!["A".to_string()])),
                [0.0, 100.0],
                EksenKonumu::Alt,
            ),
            y: değer_ekseni([0.0, 3.0], [100.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        };

        let ilk = saçılım_noktaları(&seri, &kartezyen);
        let ikinci = saçılım_noktaları(&seri, &kartezyen);

        assert_eq!(ilk[0].konum, ikinci[0].konum);
        assert_eq!(ilk[1].konum, ikinci[1].konum);
        assert!((ilk[0].konum.0 - 50.0).abs() <= 10.0);
        assert!((ilk[1].konum.0 - 50.0).abs() <= 10.0);
        assert_ne!(ilk[0].konum.0, ilk[1].konum.0);
    }

    #[test]
    fn ortusmesiz_titreme_ayni_noktalar_arasinda_sembol_payini_korur() {
        let seri = SaçılımSerisi::yeni()
            .sembol_boyutu(4.0)
            .veri([[0.0, 1.0], [0.0, 1.0]]);
        let kartezyen = Kartezyen2B {
            x: ÇalışmaEkseni::yeni(
                Eksen::kategori()
                    .titreme(20.0)
                    .titreme_örtüşmesi(false)
                    .titreme_boşluğu(2.0),
                Ölçek::Kategorik(KategorikÖlçek::yeni(vec!["A".to_string()])),
                [0.0, 100.0],
                EksenKonumu::Alt,
            ),
            y: değer_ekseni([0.0, 2.0], [100.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        };

        let noktalar = saçılım_noktaları(&seri, &kartezyen);

        assert!((noktalar[0].konum.0 - noktalar[1].konum.0).abs() >= 6.0 - 1e-4);
    }

    #[test]
    fn alan_disindaki_scatter_merkezi_atilir_sinirdaki_korunur() {
        let seri = SaçılımSerisi::yeni().veri([[0.0, 5.0], [5.0, 5.0], [12.0, 5.0]]);
        let kartezyen = Kartezyen2B {
            x: değer_ekseni([0.0, 10.0], [0.0, 100.0], EksenKonumu::Alt),
            y: değer_ekseni([0.0, 10.0], [100.0, 0.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        };

        let noktalar = saçılım_noktaları(&seri, &kartezyen);

        assert_eq!(noktalar.len(), 2);
        assert_eq!(noktalar[0].konum.0, 0.0);
        assert_eq!(noktalar[1].konum.0, 50.0);
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

    #[test]
    fn çok_satırlı_scatter_etiketi_boş_satırları_blok_yüksekliğinde_korur() {
        let seri = SaçılımSerisi::yeni()
            .sembol_boyutu(0.0)
            .etiket_boyutunu_eşle("etiket")
            .etiket(
                Etiket::yeni()
                    .göster(true)
                    .yazı(crate::model::stil::YazıStili::yeni().renk(Renk::SİYAH)),
            )
            .veri([VeriÖğesi::from([0.0, 1.0])
                .boyutlar([("etiket".to_string(), "1\n\n初四\n\n".into())])]);
        let noktalar = [SaçılımNoktası {
            sıra: 0,
            konum: (50.0, 50.0),
            boyut: 0.0,
            x_değeri: 0.0,
            y_değeri: 1.0,
            palet_sırası: None,
        }];
        let mut yüzey = KayıtYüzeyi::yeni(100.0, 100.0);

        saçılım_çiz(&mut yüzey, &seri, &noktalar, Renk::SİYAH, 1.0, 0.0, None);

        let döküm = yüzey.döküm();
        assert!(
            döküm.contains("yazı \"1\" (50.0,26.0) orta/orta"),
            "{döküm}"
        );
        assert!(
            döküm.contains("yazı \"初四\" (50.0,50.0) orta/orta"),
            "{döküm}"
        );
    }

    #[test]
    fn scatter_etiket_kayması_bağlı_metin_çapasına_eklenir() {
        let seri = SaçılımSerisi::yeni()
            .sembol_boyutu(0.0)
            .etiket(
                Etiket::yeni().göster(true).kayma(-30.0, -30.0).yazı(
                    crate::model::stil::YazıStili::yeni()
                        .boyut(14.0)
                        .renk(Renk::SİYAH),
                ),
            )
            .veri([[0.0, 1.0]]);
        let noktalar = [SaçılımNoktası {
            sıra: 0,
            konum: (50.0, 50.0),
            boyut: 0.0,
            x_değeri: 0.0,
            y_değeri: 1.0,
            palet_sırası: None,
        }];
        let mut yüzey = KayıtYüzeyi::yeni(100.0, 100.0);

        saçılım_çiz(&mut yüzey, &seri, &noktalar, Renk::SİYAH, 1.0, 0.0, None);

        let döküm = yüzey.döküm();
        assert!(
            döküm.contains("yazı \"0,1\" (20.0,20.0) orta/orta"),
            "{döküm}"
        );
    }

    fn yerleşimli_test_etiketi(
        kutu: Dikdörtgen,
        öncelik: f32,
        kaydırma: EtiketÖrtüşmeKaydırması,
        gizle: bool,
    ) -> YerleşimliSaçılımEtiketi {
        YerleşimliSaçılımEtiketi {
            nokta: (10.0, 50.0),
            sembol_boyutu: 10.0,
            metin: "etiket".to_owned(),
            çapa: (kutu.x, kutu.y),
            yatay: YatayHiza::Sol,
            dikey: DikeyHiza::Üst,
            boyut: 12.0,
            renk: Renk::SİYAH,
            kalın: false,
            kontur: None,
            döndürme: EtiketDöndürme::Yok,
            metin_kutusu: kutu,
            çakışma_kutusu: kutu,
            yerleşim: EtiketYerleşimSonucu::yeni()
                .örtüşme_kaydırması(kaydırma)
                .çakışanı_gizle(gizle),
            gizli: false,
            öncelik,
        }
    }

    #[test]
    fn label_layout_shift_y_sırayı_koruyup_çakışmayı_ileri_iter() {
        let mut etiketler = vec![
            yerleşimli_test_etiketi(
                Dikdörtgen::yeni(10.0, 10.0, 20.0, 10.0),
                1.0,
                EtiketÖrtüşmeKaydırması::Y,
                false,
            ),
            yerleşimli_test_etiketi(
                Dikdörtgen::yeni(10.0, 14.0, 20.0, 10.0),
                1.0,
                EtiketÖrtüşmeKaydırması::Y,
                false,
            ),
            yerleşimli_test_etiketi(
                Dikdörtgen::yeni(10.0, 18.0, 20.0, 10.0),
                1.0,
                EtiketÖrtüşmeKaydırması::Y,
                false,
            ),
        ];

        saçılım_etiketlerini_eksende_kaydır(&mut etiketler, 1, 100.0);

        assert_eq!(etiketler[0].çakışma_kutusu.y, 10.0);
        assert_eq!(etiketler[1].çakışma_kutusu.y, 20.0);
        assert_eq!(etiketler[2].çakışma_kutusu.y, 30.0);
    }

    #[test]
    fn label_layout_hide_overlap_büyük_sembolün_etiketini_korur() {
        let mut etiketler = vec![
            yerleşimli_test_etiketi(
                Dikdörtgen::yeni(10.0, 10.0, 30.0, 12.0),
                4.0,
                EtiketÖrtüşmeKaydırması::Yok,
                true,
            ),
            yerleşimli_test_etiketi(
                Dikdörtgen::yeni(20.0, 10.0, 30.0, 12.0),
                16.0,
                EtiketÖrtüşmeKaydırması::Yok,
                true,
            ),
        ];

        çakışan_saçılım_etiketlerini_gizle(&mut etiketler);

        assert!(etiketler[0].gizli);
        assert!(!etiketler[1].gizli);
    }

    #[test]
    fn scatter_label_line_sembol_ve_metin_kutusunun_en_yakın_kenarlarına_bağlanır() {
        let etiket = yerleşimli_test_etiketi(
            Dikdörtgen::yeni(60.0, 45.0, 20.0, 10.0),
            1.0,
            EtiketÖrtüşmeKaydırması::Yok,
            false,
        );

        let noktalar = saçılım_etiket_çizgisi_noktaları(&etiket, 5.0);

        assert_eq!(noktalar, [(15.0, 50.0), (55.0, 50.0), (60.0, 50.0)]);
    }

    #[test]
    fn scatter_vurgu_oge_stili_normal_dolgunun_ustune_yazilir() {
        let seri = SaçılımSerisi::yeni()
            .öğe_stili(crate::model::stil::ÖğeStili::yeni().renk(0x5470c6))
            .vurgu_öğe_stili(crate::model::stil::ÖğeStili::yeni().renk(0xffffff))
            .veri([[1.0, 1.0]]);
        let noktalar = [SaçılımNoktası {
            sıra: 0,
            konum: (50.0, 50.0),
            boyut: 8.0,
            x_değeri: 1.0,
            y_değeri: 1.0,
            palet_sırası: None,
        }];
        let mut yüzey = KayıtYüzeyi::yeni(100.0, 100.0);

        saçılım_çiz(
            &mut yüzey,
            &seri,
            &noktalar,
            Renk::onaltılık(0x5470c6),
            1.0,
            0.0,
            Some(0),
        );

        let döküm = yüzey.döküm();
        assert!(döküm.contains("#ffffff@1.0"), "{döküm}");
    }
}
