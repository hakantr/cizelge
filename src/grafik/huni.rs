//! Huni (funnel) serisi — `echarts/src/chart/funnel` karşılığı: yerleşim
//! (`funnelLayout`) ve görünüm tek dosyada.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{HuniSıralaması, HuniSerisi};
use crate::model::stil::EtiketKonumu;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use std::collections::HashSet;

/// Yerleşimi hesaplanmış bir huni dilimi (yamuk).
#[derive(Clone, Debug)]
pub struct HuniDilimi {
    pub sıra: usize,
    pub ad: String,
    pub değer: f64,
    /// Yamuğun köşeleri: üst sol, üst sağ, alt sağ, alt sol.
    pub köşeler: [(f32, f32); 4],
    pub renk: Renk,
}

impl HuniDilimi {
    pub fn sınır_kutusu(&self) -> Dikdörtgen {
        let üst = self.köşeler[0].1;
        let alt = self.köşeler[2].1;
        let sol = self.köşeler[0].0.min(self.köşeler[3].0);
        let sağ = self.köşeler[1].0.max(self.köşeler[2].0);
        Dikdörtgen::yeni(sol, üst, sağ - sol, alt - üst)
    }
}

/// Huni yerleşimi: değerler sıralanır, her öğe genişliği değeriyle orantılı
/// bir yamuğa dönüşür.
pub fn huni_yerleşimi(
    seri: &HuniSerisi,
    seçenekler: &GrafikSeçenekleri,
    tuval: Dikdörtgen,
    kapalı: &HashSet<String>,
    ilerleme: f32,
) -> Vec<HuniDilimi> {
    let alan = Dikdörtgen::yeni(
        tuval.x + seri.sol.çöz(tuval.genişlik),
        tuval.y + seri.üst.çöz(tuval.yükseklik),
        seri.genişlik.çöz(tuval.genişlik),
        seri.yükseklik.çöz(tuval.yükseklik),
    );

    let mut görünürler: Vec<(usize, String, f64)> = seri
        .veri
        .iter()
        .enumerate()
        .filter_map(|(i, ö)| {
            let ad = ö.ad.clone().unwrap_or_else(|| format!("{i}"));
            let değer = ö.değer.sayı()?;
            (!kapalı.contains(&ad) && değer.is_finite() && değer >= 0.0)
                .then_some((i, ad, değer))
        })
        .collect();
    match seri.sıralama {
        HuniSıralaması::Azalan => {
            görünürler.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
        }
        HuniSıralaması::Artan => {
            görünürler.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        }
        HuniSıralaması::Yok => {}
    }
    if görünürler.is_empty() {
        return Vec::new();
    }

    let en_büyük = görünürler
        .iter()
        .map(|(_, _, d)| *d)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(1e-12);
    let n = görünürler.len() as f32;
    let dilim_yüksekliği =
        ((alan.yükseklik - seri.dilim_boşluğu * (n - 1.0)) / n).max(1.0);
    let ilerleme = ilerleme.clamp(0.0, 1.0);

    let genişlik_çöz = |değer: f64| -> f32 {
        let oran = (değer / en_büyük) as f32;
        let en_az = seri.en_az_genişlik.çöz(alan.genişlik);
        let en_çok = seri.en_çok_genişlik.çöz(alan.genişlik);
        (en_az + (en_çok - en_az) * oran) * ilerleme
    };

    let merkez_x = alan.x + alan.genişlik / 2.0;
    let mut dilimler = Vec::with_capacity(görünürler.len());
    for (satır, (sıra, ad, değer)) in görünürler.iter().enumerate() {
        let üst_y = alan.y + satır as f32 * (dilim_yüksekliği + seri.dilim_boşluğu);
        let alt_y = üst_y + dilim_yüksekliği;
        let üst_genişlik = genişlik_çöz(*değer);
        // Alt kenar bir sonraki dilimin genişliğine daralır (klasik huni).
        let alt_genişlik = görünürler
            .get(satır + 1)
            .map(|(_, _, sonraki)| genişlik_çöz(*sonraki))
            .unwrap_or_else(|| üst_genişlik * 0.35);
        let renk = seri
            .veri
            .get(*sıra)
            .and_then(|ö| ö.stil.as_ref())
            .and_then(|s| s.renk.as_ref())
            .map(|d| d.temsilî())
            .unwrap_or_else(|| seçenekler.palet_rengi(*sıra));
        dilimler.push(HuniDilimi {
            sıra: *sıra,
            ad: ad.clone(),
            değer: *değer,
            köşeler: [
                (merkez_x - üst_genişlik / 2.0, üst_y),
                (merkez_x + üst_genişlik / 2.0, üst_y),
                (merkez_x + alt_genişlik / 2.0, alt_y),
                (merkez_x - alt_genişlik / 2.0, alt_y),
            ],
            renk,
        });
    }
    dilimler
}

/// Huniyi çizer ve isabet bölgelerini toplar.
pub fn huni_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &HuniSerisi,
    genel_sıra: usize,
    dilimler: &[HuniDilimi],
    vurgulu: Option<usize>,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    for (i, dilim) in dilimler.iter().enumerate() {
        let mut yol = Yol::yeni();
        yol.taşı(dilim.köşeler[0]);
        yol.çiz(dilim.köşeler[1]);
        yol.çiz(dilim.köşeler[2]);
        yol.çiz(dilim.köşeler[3]);
        yol.kapat();
        let opaklık = if vurgulu == Some(i) { 1.0 } else { seri.öğe_stili.opaklık.unwrap_or(1.0) };
        çizici.yol_doldur(&yol, &Dolgu::Düz(dilim.renk.opaklık(opaklık)));
        if let Some(kenar_rengi) = seri.öğe_stili.kenarlık_rengi {
            çizici.yol_çiz(
                &yol,
                seri.öğe_stili.kenarlık_kalınlığı.max(1.0),
                kenar_rengi,
                crate::model::stil::ÇizgiTürü::Düz,
            );
        }

        // Etiket.
        if seri.etiket.göster {
            let kutu = dilim.sınır_kutusu();
            let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            let metin = match &seri.etiket.biçimleyici {
                Some(b) => b.uygula(dilim.değer, &dilim.ad),
                None => dilim.ad.clone(),
            };
            match seri.etiket.konum {
                EtiketKonumu::İç | EtiketKonumu::Merkez => {
                    let renk = seri.etiket.yazı.renk.unwrap_or(Renk::BEYAZ);
                    çizici.yazı(
                        &metin,
                        kutu.merkez(),
                        YatayHiza::Orta,
                        DikeyHiza::Orta,
                        boyut,
                        renk,
                        seri.etiket.yazı.kalın,
                    );
                }
                _ => {
                    // Dış etiket: sağda, kılavuz çizgisiyle.
                    let (orta_x, orta_y) = kutu.merkez();
                    let uç = (kutu.sağ() + 18.0, orta_y);
                    çizici.çizgi(
                        (orta_x.max(kutu.sağ() - 4.0), orta_y),
                        uç,
                        1.0,
                        dilim.renk,
                        crate::model::stil::ÇizgiTürü::Düz,
                    );
                    let renk = seri.etiket.yazı.renk.unwrap_or(tema::birincil_metin());
                    çizici.yazı(
                        &metin,
                        (uç.0 + 4.0, uç.1),
                        YatayHiza::Sol,
                        DikeyHiza::Orta,
                        boyut,
                        renk,
                        seri.etiket.yazı.kalın,
                    );
                }
            }
        }

        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: dilim.sıra,
            seri_adı: seri.ad.clone(),
            ad: Some(dilim.ad.clone()),
            değer: Some(dilim.değer),
            geometri: İsabetGeometrisi::Dikdörtgen(dilim.sınır_kutusu()),
        });
    }
}
