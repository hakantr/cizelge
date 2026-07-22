//! Huni (funnel) serisi — `echarts/src/chart/funnel` karşılığı: yerleşim
//! (`funnelLayout`) ve görünüm (`FunnelView`) aynı sahne sözleşmesini izler.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::pasta::zengin_etiketi_hizalı_yaz;
use crate::koordinat::Dikdörtgen;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{
    EtiketÇizgisi, HuniDurumYaması, HuniHizası, HuniSerisi, HuniSıralaması, HuniYönü,
};
use crate::model::stil::{
    Etiket, EtiketDöndürme, EtiketKonumu, YazıDikeyHizası, YazıYatayHizası, ÖğeStili,
};
use crate::renk::Dolgu;
use crate::tema;
use crate::yardimci::bicim::ondalık_kırp;
use std::cmp::Ordering;
use std::collections::HashSet;

/// Yerleşimi hesaplanmış bir huni dilimi (yamuk).
#[derive(Clone, Debug)]
pub struct HuniDilimi {
    /// Ham veri sırası; sıralama ve legend süzmesinden sonra da değişmez.
    pub sıra: usize,
    pub ad: String,
    pub değer: f64,
    /// Yamuğun köşeleri ECharts `funnelLayout` sırasındadır.
    pub köşeler: [(f32, f32); 4],
    pub dolgu: Dolgu,
}

impl HuniDilimi {
    pub fn sınır_kutusu(&self) -> Dikdörtgen {
        let (mut sol, mut sağ) = (f32::INFINITY, f32::NEG_INFINITY);
        let (mut üst, mut alt) = (f32::INFINITY, f32::NEG_INFINITY);
        for &(x, y) in &self.köşeler {
            sol = sol.min(x);
            sağ = sağ.max(x);
            üst = üst.min(y);
            alt = alt.max(y);
        }
        Dikdörtgen::yeni(sol, üst, (sağ - sol).max(0.0), (alt - üst).max(0.0))
    }

    pub fn içeriyor_mu(&self, nokta: (f32, f32)) -> bool {
        let mut içeride = false;
        let mut önceki = self.köşeler.len() - 1;
        for sıra in 0..self.köşeler.len() {
            let (xi, yi) = self.köşeler[sıra];
            let (xj, yj) = self.köşeler[önceki];
            if ((yi > nokta.1) != (yj > nokta.1))
                && nokta.0 < (xj - xi) * (nokta.1 - yi) / (yj - yi) + xi
            {
                içeride = !içeride;
            }
            önceki = sıra;
        }
        içeride
    }
}

/// `BoxLayoutOptionMixin`in Funnel görünüm dikdörtgeni.
pub fn huni_görünüm_alanı(seri: &HuniSerisi, ana: Dikdörtgen) -> Dikdörtgen {
    let sol = seri.sol.çöz(ana.genişlik);
    let sağ = seri.sağ.çöz(ana.genişlik);
    let üst = seri.üst.çöz(ana.yükseklik);
    let alt = seri.alt.çöz(ana.yükseklik);
    let genişlik = seri
        .genişlik
        .map(|değer| değer.çöz(ana.genişlik))
        .unwrap_or(ana.genişlik - sol - sağ)
        .max(0.0);
    let yükseklik = seri
        .yükseklik
        .map(|değer| değer.çöz(ana.yükseklik))
        .unwrap_or(ana.yükseklik - üst - alt)
        .max(0.0);
    Dikdörtgen::yeni(ana.x + sol, ana.y + üst, genişlik, yükseklik)
}

fn doğrusal_eşle(değer: f64, alan: [f64; 2], çıktı: [f32; 2]) -> f32 {
    let açıklık = alan[1] - alan[0];
    if açıklık.abs() <= f64::EPSILON {
        return if (çıktı[1] - çıktı[0]).abs() <= f32::EPSILON {
            çıktı[0]
        } else {
            (çıktı[0] + çıktı[1]) / 2.0
        };
    }
    if açıklık > 0.0 {
        if değer <= alan[0] {
            return çıktı[0];
        }
        if değer >= alan[1] {
            return çıktı[1];
        }
    } else {
        if değer >= alan[0] {
            return çıktı[0];
        }
        if değer <= alan[1] {
            return çıktı[1];
        }
    }
    çıktı[0] + (çıktı[1] - çıktı[0]) * ((değer - alan[0]) / açıklık) as f32
}

/// Huni yerleşimi, ECharts `funnelLayout.ts` ile aynı sıralama, dinamik
/// değer kapsamı, `minSize/maxSize`, yön ve hizalama kurallarını uygular.
pub fn huni_yerleşimi(
    seri: &HuniSerisi,
    seçenekler: &GrafikSeçenekleri,
    tuval: Dikdörtgen,
    kapalı: &HashSet<String>,
    _ilerleme: f32,
) -> Vec<HuniDilimi> {
    let alan = huni_görünüm_alanı(seri, tuval);
    let mut görünürler: Vec<(usize, String, f64)> = seri
        .veri
        .iter()
        .enumerate()
        .filter_map(|(sıra, öğe)| {
            let ad = öğe.ad.clone().unwrap_or_else(|| format!("{sıra}"));
            let değer = öğe.değer.sayı()?;
            (!kapalı.contains(&ad) && değer.is_finite()).then_some((sıra, ad, değer))
        })
        .collect();
    match seri.sıralama {
        HuniSıralaması::Azalan => {
            görünürler.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal))
        }
        HuniSıralaması::Artan => {
            görünürler.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
        }
        HuniSıralaması::Yok => {}
    }
    if görünürler.is_empty() {
        return Vec::new();
    }

    let veri_en_az = görünürler
        .iter()
        .map(|(_, _, değer)| *değer)
        .fold(f64::INFINITY, f64::min);
    let veri_en_çok = görünürler
        .iter()
        .map(|(_, _, değer)| *değer)
        .fold(f64::NEG_INFINITY, f64::max);
    let en_az = seri.en_az.unwrap_or(veri_en_az.min(0.0));
    let en_çok = seri.en_çok.unwrap_or(veri_en_çok);
    let yatay = seri.yön == HuniYönü::Yatay;
    let çapraz_boyut = if yatay {
        alan.yükseklik
    } else {
        alan.genişlik
    };
    let boyut_aralığı = [
        seri.en_az_genişlik.çöz(çapraz_boyut),
        seri.en_çok_genişlik.çöz(çapraz_boyut),
    ];
    let ana_boyut = if yatay {
        alan.genişlik
    } else {
        alan.yükseklik
    };
    let mut boşluk = seri.dilim_boşluğu;
    let mut öğe_boyutu = (ana_boyut - boşluk * (görünürler.len().saturating_sub(1)) as f32)
        / görünürler.len() as f32;
    let mut x = alan.x;
    let mut y = alan.y;
    if seri.sıralama == HuniSıralaması::Artan {
        öğe_boyutu = -öğe_boyutu;
        boşluk = -boşluk;
        if yatay {
            x += alan.genişlik;
        } else {
            y += alan.yükseklik;
        }
        görünürler.reverse();
    }

    let çizgi = |öğe: Option<&(usize, String, f64)>, kaydırma: f32| {
        // ECharts, son çizginin olmayan veri değerini `0` kabul eder.
        let değer = öğe.map(|(_, _, değer)| *değer).unwrap_or(0.0);
        let boyut = doğrusal_eşle(değer, [en_az, en_çok], boyut_aralığı);
        if yatay {
            let y0 = match seri.hiza {
                HuniHizası::Üst => alan.y,
                HuniHizası::Alt => alan.alt() - boyut,
                _ => alan.y + (alan.yükseklik - boyut) / 2.0,
            };
            [(kaydırma, y0), (kaydırma, y0 + boyut)]
        } else {
            let x0 = match seri.hiza {
                HuniHizası::Sol => alan.x,
                HuniHizası::Sağ => alan.sağ() - boyut,
                _ => alan.x + (alan.genişlik - boyut) / 2.0,
            };
            [(x0, kaydırma), (x0 + boyut, kaydırma)]
        }
    };

    let mut dilimler = Vec::with_capacity(görünürler.len());
    for (konum, (sıra, ad, değer)) in görünürler.iter().enumerate() {
        let yama = seri.öğe_yamaları.get(*sıra).and_then(Option::as_ref);
        let boyut = if yatay {
            yama.and_then(|yama| yama.genişlik)
                .map(|değer| değer.çöz(alan.genişlik))
                .unwrap_or(öğe_boyutu.abs())
                .copysign(öğe_boyutu)
        } else {
            yama.and_then(|yama| yama.yükseklik)
                .map(|değer| değer.çöz(alan.yükseklik))
                .unwrap_or(öğe_boyutu.abs())
                .copysign(öğe_boyutu)
        };
        let başlangıç = çizgi(Some(&görünürler[konum]), if yatay { x } else { y });
        let bitiş = çizgi(
            görünürler.get(konum + 1),
            if yatay { x + boyut } else { y + boyut },
        );
        if yatay {
            x += boyut + boşluk;
        } else {
            y += boyut + boşluk;
        }
        let dolgu = seri
            .veri
            .get(*sıra)
            .and_then(|öğe| öğe.stil.as_ref())
            .and_then(|stil| stil.renk.clone())
            .or_else(|| seri.öğe_stili.renk.clone())
            .unwrap_or_else(|| Dolgu::Düz(seçenekler.palet_rengi(*sıra)));
        dilimler.push(HuniDilimi {
            sıra: *sıra,
            ad: ad.clone(),
            değer: *değer,
            köşeler: [başlangıç[0], başlangıç[1], bitiş[1], bitiş[0]],
            dolgu,
        });
    }
    dilimler
}

fn stil_yaması(taban: &ÖğeStili, yama: &ÖğeStili) -> ÖğeStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk.clone_from(&yama.renk);
    }
    if yama.kenarlık_rengi.is_some() {
        sonuç.kenarlık_rengi = yama.kenarlık_rengi;
    }
    if yama.kenarlık_kalınlığı > 0.0 {
        sonuç.kenarlık_kalınlığı = yama.kenarlık_kalınlığı;
    }
    if yama.kenarlık_türü != Default::default() {
        sonuç.kenarlık_türü = yama.kenarlık_türü;
    }
    if yama.kenarlık_yarıçapı.iter().any(|yarıçap| *yarıçap > 0.0) {
        sonuç.kenarlık_yarıçapı = yama.kenarlık_yarıçapı;
    }
    if yama.opaklık.is_some() {
        sonuç.opaklık = yama.opaklık;
    }
    if yama.gölge_bulanıklığı > 0.0 {
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

fn durum_uygula(
    durum: &HuniDurumYaması,
    öğe_stili: &mut ÖğeStili,
    etiket: &mut Etiket,
    etiket_çizgisi: &mut EtiketÇizgisi,
) {
    *öğe_stili = stil_yaması(öğe_stili, &durum.öğe_stili);
    *etiket = durum.etiket.uygula(etiket);
    *etiket_çizgisi = durum.etiket_çizgisi.uygula(etiket_çizgisi);
}

fn etiket_metni(seri: &HuniSerisi, dilim: &HuniDilimi, etiket: &Etiket) -> String {
    let ham = ondalık_kırp(dilim.değer);
    let toplam = seri
        .veri
        .iter()
        .filter_map(|öğe| öğe.değer.sayı())
        .filter(|değer| değer.is_finite())
        .sum::<f64>();
    let yüzde = if toplam.abs() <= f64::EPSILON {
        0.0
    } else {
        (dilim.değer / toplam * 10_000.0).round() / 100.0
    };
    match &etiket.biçimleyici {
        Some(biçimleyici) => biçimleyici
            .uygula_bağlamla_zengin(
                dilim.değer,
                &ham,
                seri.ad.as_deref().unwrap_or(""),
                &dilim.ad,
            )
            .replace("{d}", &ondalık_kırp(yüzde)),
        None => dilim.ad.clone(),
    }
}

fn etiket_yerleşimi(
    dilim: &HuniDilimi,
    etiket: &Etiket,
    çizgi: &EtiketÇizgisi,
    yatay: bool,
) -> ((f32, f32), YatayHiza, [(f32, f32); 2], bool) {
    let p = dilim.köşeler;
    let içeride = matches!(
        etiket.konum,
        EtiketKonumu::İç | EtiketKonumu::Merkez | EtiketKonumu::İçSol | EtiketKonumu::İçSağ
    );
    if içeride {
        let (x, y, hiza) = match etiket.konum {
            EtiketKonumu::İçSol => (
                (p[0].0 + p[3].0) / 2.0 + 5.0,
                (p[0].1 + p[3].1) / 2.0,
                YatayHiza::Sol,
            ),
            EtiketKonumu::İçSağ => (
                (p[1].0 + p[2].0) / 2.0 - 5.0,
                (p[1].1 + p[2].1) / 2.0,
                YatayHiza::Sağ,
            ),
            _ => (
                p.iter().map(|nokta| nokta.0).sum::<f32>() / 4.0,
                p.iter().map(|nokta| nokta.1).sum::<f32>() / 4.0,
                YatayHiza::Orta,
            ),
        };
        return ((x, y), hiza, [(x, y), (x, y)], true);
    }

    let uzunluk = çizgi.uzunluk1;
    let (baş, son, metin, hiza) = match etiket.konum {
        EtiketKonumu::Sol => {
            let baş = ((p[3].0 + p[0].0) / 2.0, (p[3].1 + p[0].1) / 2.0);
            let son = if yatay {
                (baş.0, baş.1 + uzunluk)
            } else {
                (baş.0 - uzunluk, baş.1)
            };
            let metin = if yatay {
                (son.0, son.1 + 5.0)
            } else {
                (son.0 - 5.0, son.1)
            };
            (
                baş,
                son,
                metin,
                if yatay {
                    YatayHiza::Orta
                } else {
                    YatayHiza::Sağ
                },
            )
        }
        EtiketKonumu::Üst => {
            let baş = ((p[3].0 + p[0].0) / 2.0, (p[3].1 + p[0].1) / 2.0);
            let son = if yatay {
                (baş.0, baş.1 - uzunluk)
            } else {
                (baş.0 - uzunluk, baş.1)
            };
            let metin = if yatay {
                (son.0, son.1 - 5.0)
            } else {
                (son.0 - 5.0, son.1)
            };
            (
                baş,
                son,
                metin,
                if yatay {
                    YatayHiza::Orta
                } else {
                    YatayHiza::Sağ
                },
            )
        }
        EtiketKonumu::Alt => {
            let baş = ((p[1].0 + p[2].0) / 2.0, (p[1].1 + p[2].1) / 2.0);
            let son = if yatay {
                (baş.0, baş.1 + uzunluk)
            } else {
                (baş.0 + uzunluk, baş.1)
            };
            let metin = if yatay {
                (son.0, son.1 + 5.0)
            } else {
                (son.0 + 5.0, son.1)
            };
            (
                baş,
                son,
                metin,
                if yatay {
                    YatayHiza::Orta
                } else {
                    YatayHiza::Sol
                },
            )
        }
        EtiketKonumu::SağÜst | EtiketKonumu::SolÜst => {
            let sağ = etiket.konum == EtiketKonumu::SağÜst;
            let baş = if yatay {
                if sağ { p[3] } else { p[0] }
            } else if sağ {
                p[1]
            } else {
                p[0]
            };
            let son = if yatay {
                (baş.0, baş.1 - uzunluk)
            } else {
                (baş.0 + if sağ { uzunluk } else { -uzunluk }, baş.1)
            };
            let metin = if yatay {
                (son.0, son.1 - 5.0)
            } else {
                (son.0 + if sağ { 5.0 } else { -5.0 }, son.1)
            };
            (
                baş,
                son,
                metin,
                if yatay {
                    YatayHiza::Orta
                } else if sağ {
                    YatayHiza::Sol
                } else {
                    YatayHiza::Sağ
                },
            )
        }
        EtiketKonumu::SağAlt | EtiketKonumu::SolAlt => {
            let sağ = etiket.konum == EtiketKonumu::SağAlt;
            let baş = if yatay {
                if sağ { p[2] } else { p[1] }
            } else if sağ {
                p[2]
            } else {
                p[3]
            };
            let son = if yatay {
                (baş.0, baş.1 + uzunluk)
            } else {
                (baş.0 + if sağ { uzunluk } else { -uzunluk }, baş.1)
            };
            let metin = if yatay {
                (son.0, son.1 + 5.0)
            } else {
                (son.0 + if sağ { 5.0 } else { -5.0 }, son.1)
            };
            (
                baş,
                son,
                metin,
                if yatay {
                    YatayHiza::Orta
                } else if sağ {
                    YatayHiza::Sol
                } else {
                    YatayHiza::Sağ
                },
            )
        }
        _ => {
            // `outer` ve yönle uyumsuz konumlar ECharts'taki gibi sağ
            // (yatayda alt) tarafa düşer.
            let baş = ((p[1].0 + p[2].0) / 2.0, (p[1].1 + p[2].1) / 2.0);
            let son = if yatay {
                (baş.0, baş.1 + uzunluk)
            } else {
                (baş.0 + uzunluk, baş.1)
            };
            let metin = if yatay {
                (son.0, son.1 + 5.0)
            } else {
                (son.0 + 5.0, son.1)
            };
            (
                baş,
                son,
                metin,
                if yatay {
                    YatayHiza::Orta
                } else {
                    YatayHiza::Sol
                },
            )
        }
    };
    (metin, hiza, [baş, son], false)
}

/// Huniyi çizer ve isabet bölgelerini toplar. `vurgulu`, ham veri sırasıdır.
pub fn huni_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &HuniSerisi,
    genel_sıra: usize,
    dilimler: &[HuniDilimi],
    vurgulu: Option<usize>,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let giriş_opaklığı = ilerleme.clamp(0.0, 1.0);
    let yatay = seri.yön == HuniYönü::Yatay;
    for dilim in dilimler {
        let veri_öğesi = seri.veri.get(dilim.sıra);
        let veri_yaması = seri.öğe_yamaları.get(dilim.sıra).and_then(Option::as_ref);
        let mut öğe_stili = veri_öğesi
            .and_then(|öğe| öğe.stil.as_ref())
            .map(|stil| stil_yaması(&seri.öğe_stili, stil))
            .unwrap_or_else(|| seri.öğe_stili.clone());
        let mut etiket = veri_öğesi
            .and_then(|öğe| öğe.etiket.as_ref())
            .map(|yama| yama.uygula(&seri.etiket))
            .unwrap_or_else(|| seri.etiket.clone());
        let mut etiket_çizgisi = veri_yaması
            .map(|yama| yama.etiket_çizgisi.uygula(&seri.etiket_çizgisi))
            .unwrap_or_else(|| seri.etiket_çizgisi.clone());

        if veri_öğesi.is_some_and(|öğe| öğe.seçili) {
            durum_uygula(
                &seri.seçim,
                &mut öğe_stili,
                &mut etiket,
                &mut etiket_çizgisi,
            );
            if let Some(yama) = veri_yaması {
                durum_uygula(
                    &yama.seçim,
                    &mut öğe_stili,
                    &mut etiket,
                    &mut etiket_çizgisi,
                );
            }
            // FunnelSeries.defaultOption.select.itemStyle.borderColor,
            // açık bir yama verilmediyse etkin temanın `primary` belirtecidir.
            let açık_seçim_kenarı = seri.seçim.öğe_stili.kenarlık_rengi.is_some()
                || veri_yaması.is_some_and(|yama| yama.seçim.öğe_stili.kenarlık_rengi.is_some());
            if !açık_seçim_kenarı {
                öğe_stili.kenarlık_rengi = Some(tema::birincil_metin());
            }
        }
        if vurgulu == Some(dilim.sıra) {
            durum_uygula(
                &seri.vurgu,
                &mut öğe_stili,
                &mut etiket,
                &mut etiket_çizgisi,
            );
            if let Some(yama) = veri_yaması {
                durum_uygula(
                    &yama.vurgu,
                    &mut öğe_stili,
                    &mut etiket,
                    &mut etiket_çizgisi,
                );
            }
        }
        let etkin_dolgu = öğe_stili
            .renk
            .clone()
            .unwrap_or_else(|| dilim.dolgu.clone());

        let mut yol = Yol::yeni();
        yol.taşı(dilim.köşeler[0]);
        for &nokta in &dilim.köşeler[1..] {
            yol.çiz(nokta);
        }
        yol.kapat();
        let opaklık = öğe_stili.opaklık.unwrap_or(1.0) * giriş_opaklığı;
        if öğe_stili.gölge_bulanıklığı > 0.0
            && let Some(renk) = öğe_stili.gölge_rengi
        {
            çizici.yol_gölgesi(
                &yol,
                renk.opaklık(opaklık),
                öğe_stili.gölge_bulanıklığı,
                öğe_stili.gölge_kayması,
            );
        }
        çizici.yol_doldur(&yol, &etkin_dolgu.opaklık(opaklık));
        if öğe_stili.kenarlık_kalınlığı > 0.0
            && let Some(kenar_rengi) = öğe_stili.kenarlık_rengi
        {
            çizici.yol_çiz(
                &yol,
                öğe_stili.kenarlık_kalınlığı,
                kenar_rengi.opaklık(opaklık),
                öğe_stili.kenarlık_türü,
            );
        }

        if etiket.göster {
            let (konum, doğal_hiza, çizgi_noktaları, içeride) =
                etiket_yerleşimi(dilim, &etiket, &etiket_çizgisi, yatay);
            if !içeride && etiket_çizgisi.göster {
                let mut çizgi_yolu = Yol::yeni();
                çizgi_yolu.taşı(çizgi_noktaları[0]);
                çizgi_yolu.çiz(çizgi_noktaları[1]);
                let stil = &etiket_çizgisi.stil;
                let renk = stil
                    .renk
                    .unwrap_or_else(|| etkin_dolgu.temsilî())
                    .opaklık(stil.opaklık * opaklık);
                çizici.yol_çiz(&çizgi_yolu, stil.kalınlık, renk, stil.tür);
            }
            let yatay_hiza = etiket
                .yatay_hiza
                .map(|hiza| match hiza {
                    YazıYatayHizası::Sol => YatayHiza::Sol,
                    YazıYatayHizası::Orta => YatayHiza::Orta,
                    YazıYatayHizası::Sağ => YatayHiza::Sağ,
                })
                .unwrap_or(doğal_hiza);
            let dikey_hiza = etiket
                .dikey_hiza
                .map(|hiza| match hiza {
                    YazıDikeyHizası::Üst => DikeyHiza::Üst,
                    YazıDikeyHizası::Orta => DikeyHiza::Orta,
                    YazıDikeyHizası::Alt => DikeyHiza::Alt,
                })
                .unwrap_or(DikeyHiza::Orta);
            let varsayılan_renk = etiket.yazı.renk.unwrap_or_else(|| {
                if içeride {
                    etkin_dolgu.zrender_iç_etiket_stili(tema::koyu_mu()).0
                } else {
                    tema::birincil_metin()
                }
            });
            let yazı_opaklığı = etiket.yazı.opaklık.unwrap_or(1.0) * opaklık;
            let dönüş = match etiket.döndürme {
                EtiketDöndürme::Derece(derece) => -derece.to_radians(),
                _ => 0.0,
            };
            zengin_etiketi_hizalı_yaz(
                çizici,
                &etiket_metni(seri, dilim, &etiket),
                &etiket,
                konum,
                yatay_hiza,
                dikey_hiza,
                varsayılan_renk.opaklık(yazı_opaklığı),
                dönüş,
            );
        }

        if !seri.sessiz {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: dilim.sıra,
                seri_adı: seri.ad.clone(),
                ad: Some(dilim.ad.clone()),
                değer: Some(dilim.değer),
                geometri: İsabetGeometrisi::Çokgen {
                    noktalar: dilim.köşeler.to_vec(),
                },
            });
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::model::deger::VeriÖğesi;

    fn alan() -> Dikdörtgen {
        Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0)
    }

    #[test]
    fn resmi_son_dilim_min_size_ile_sifira_kapanir() {
        let seri = HuniSerisi::yeni()
            .sol("10%")
            .üst(60)
            .alt(60)
            .genişlik("80%")
            .değer_aralığı(0.0, 100.0)
            .dilim_boşluğu(2.0)
            .veri([
                VeriÖğesi::adlı("Visit", 60.0),
                VeriÖğesi::adlı("Inquiry", 40.0),
                VeriÖğesi::adlı("Order", 20.0),
                VeriÖğesi::adlı("Click", 80.0),
                VeriÖğesi::adlı("Show", 100.0),
            ]);
        let dilimler = huni_yerleşimi(
            &seri,
            &GrafikSeçenekleri::yeni(),
            alan(),
            &HashSet::new(),
            1.0,
        );
        assert_eq!(dilimler[0].ad, "Show");
        assert!((dilimler[0].köşeler[0].0 - 70.0).abs() < 1e-5);
        assert!((dilimler.last().unwrap().köşeler[2].0 - 350.0).abs() < 1e-5);
        assert!((dilimler.last().unwrap().köşeler[3].0 - 350.0).abs() < 1e-5);
    }

    #[test]
    fn artan_huni_alttan_üste_ve_sola_hizali_yerlesir() {
        let seri = HuniSerisi::yeni()
            .sol(0)
            .sağ(0)
            .üst(0)
            .alt(0)
            .hiza(HuniHizası::Sol)
            .sıralama(HuniSıralaması::Artan)
            .veri([("az", 10.0), ("çok", 100.0)]);
        let dilimler = huni_yerleşimi(
            &seri,
            &GrafikSeçenekleri::yeni(),
            alan(),
            &HashSet::new(),
            1.0,
        );
        assert_eq!(dilimler[0].ad, "çok");
        assert!((dilimler[0].köşeler[0].1 - 525.0).abs() < 1e-5);
        assert!((dilimler[0].köşeler[0].0 - 0.0).abs() < 1e-5);
        assert_eq!(dilimler[1].ad, "az");
    }

    #[test]
    fn yatay_huni_degeri_yukseklige_ve_akisi_genislige_esler() {
        let seri = HuniSerisi::yeni()
            .sol(0)
            .sağ(0)
            .üst(0)
            .alt(0)
            .yön(HuniYönü::Yatay)
            .hiza(HuniHizası::Üst)
            .veri([("çok", 100.0), ("az", 50.0)]);
        let dilimler = huni_yerleşimi(
            &seri,
            &GrafikSeçenekleri::yeni(),
            alan(),
            &HashSet::new(),
            1.0,
        );
        assert_eq!(dilimler[0].köşeler[0], (0.0, 0.0));
        assert!((dilimler[0].köşeler[1].1 - 525.0).abs() < 1e-5);
        assert!((dilimler[0].köşeler[2].0 - 350.0).abs() < 1e-5);
    }

    #[test]
    fn vurgu_dolgusu_normal_palet_dolgusunun_ustune_yazilir() {
        let seri = HuniSerisi::yeni()
            .veri([("A", 100.0)])
            .vurgu(HuniDurumYaması::yeni().öğe_stili(ÖğeStili::yeni().renk("#ff0000")));
        let dilimler = huni_yerleşimi(
            &seri,
            &GrafikSeçenekleri::yeni(),
            alan(),
            &HashSet::new(),
            1.0,
        );
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);
        huni_çiz(
            &mut yüzey,
            &seri,
            0,
            &dilimler,
            Some(0),
            1.0,
            &mut Vec::new(),
        );

        assert!(yüzey.komutlar.iter().any(|komut| komut.contains("#ff0000")));
    }
}
