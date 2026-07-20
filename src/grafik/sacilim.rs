//! Saçılım serisi çizimi — `echarts/src/chart/scatter` karşılığı.

use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::sembol_stilli_çiz;
use crate::koordinat::{Kartezyen2B, TakvimYerleşimi};
use crate::model::deger::{VeriDeğeri, VeriÖğesi};
use crate::model::gorsel_esleme::GörselEşleme;
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
    /// `colorBy: 'data'` palet anahtarı; kategorik eksende kategori sırası,
    /// iki sayısal eksende `None` (ham veri sırası kullanılır).
    pub palet_sırası: Option<usize>,
}

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

fn titremeyi_uygula(noktalar: &mut [SaçılımNoktası], kartezyen: &Kartezyen2B) {
    let (x_mi, eksen) = if kartezyen.x.ölçek.kategorik_mi() && kartezyen.x.seçenek.titreme > 0.0
    {
        (true, &kartezyen.x)
    } else if kartezyen.y.ölçek.kategorik_mi() && kartezyen.y.seçenek.titreme > 0.0 {
        (false, &kartezyen.y)
    } else {
        return;
    };
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
            None => match saçılım_xy(&öğe.değer, i) {
                Some(koordinat) => koordinat,
                None => continue,
            },
        };
        sonuç.push(SaçılımNoktası {
            sıra: i,
            konum: kartezyen.nokta(x, y),
            boyut: seri.sembol_boyutu.çöz(öğe),
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
                palet_sırası: None,
            })
        })
        .collect()
}

/// Scatter/effectScatter ikinci koordinat boyutunun görsel eşleme kapsamı.
pub fn saçılım_değer_kapsamı(seri: &SaçılımSerisi) -> [f64; 2] {
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    for değer in seri
        .veri
        .iter()
        .enumerate()
        .filter_map(|(sıra, öğe)| saçılım_xy(&öğe.değer, sıra).map(|(_, y)| y))
    {
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
) {
    match (kontur, dönüşüm) {
        (Some(kontur), Some(dönüşüm)) => {
            çizici.dönüşümlü_konturlu_yazı(
                metin,
                konum,
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
                (0.0, 0.0),
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
            çizici.dönüşümlü_yazı(metin, konum, yatay, dikey, boyut, renk, kalın, dönüşüm);
        }
        (None, None) => {
            çizici.yazı(metin, konum, yatay, dikey, boyut, renk, kalın);
        }
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
    // `scatter` öntanımlı 0.8, `effectScatter` ise 1.0 opaklıktadır.
    let opaklık = seri
        .öğe_stili
        .opaklık
        .unwrap_or(if seri.efektli { 1.0 } else { 0.8 });
    let taban_rengi = seri
        .öğe_stili
        .renk
        .as_ref()
        .map(|d| d.temsilî())
        .unwrap_or(seri_rengi);
    let nokta_rengi = |nokta: &SaçılımNoktası| {
        if let Some(renk) = seri
            .veri
            .get(nokta.sıra)
            .and_then(|öğe| öğe.stil.as_ref())
            .and_then(|stil| stil.renk.as_ref())
        {
            renk.temsilî()
        } else if seri.öğe_stili.renk.is_some() {
            taban_rengi
        } else if let Some((eşleme, kapsam)) = görsel_eşleme {
            eşleme.renk_çöz(nokta.y_değeri, kapsam)
        } else if seri.veriye_göre_renk {
            let palet_sırası = nokta.palet_sırası.unwrap_or(nokta.sıra);
            palet
                .get(palet_sırası % palet.len().max(1))
                .copied()
                .unwrap_or_else(|| tema::palet_rengi(palet_sırası))
        } else {
            taban_rengi
        }
    };
    // EffectSymbol çekirdeği önce, z2=99 dalgaları sonra boyar.
    for nokta in noktalar {
        let renk = nokta_rengi(nokta);
        let vurgulu_mu = vurgulu == Some(nokta.sıra);
        let boyut = nokta.boyut * ilerleme.clamp(0.0, 1.0) * if vurgulu_mu { 1.15 } else { 1.0 };
        let öğe_stili = seri.veri.get(nokta.sıra).and_then(|öğe| öğe.stil.as_ref());
        let nokta_opaklığı = if vurgulu_mu {
            1.0
        } else {
            öğe_stili.and_then(|stil| stil.opaklık).unwrap_or(opaklık)
        };
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
            öğe_stili
                .and_then(|stil| stil.renk.as_ref())
                .or(seri.öğe_stili.renk.as_ref()),
            kenarlık,
            nokta_opaklığı,
        );
    }

    // Dataset `encode.label` dâhil saçılım etiketleri. Öğe yaması seri
    // etiketini miras alır; açık align/verticalAlign/rotate değerleri
    // zrender bağlı metin yerleşimine aktarılır.
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
                AfinMatris::ötele(çapa.0, çapa.1).çarp(AfinMatris::döndür(-derece.to_radians())),
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
                );
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
}
