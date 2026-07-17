//! Ağaç haritası (treemap) — `echarts/src/chart/treemap` karşılığı.
//! Yerleşim, kareselleştirilmiş (squarified) ağaç haritası yaklaşımıyla
//! hesaplanır: her satır/sütun öbeği, en kötü en-boy oranını en aza
//! indirecek biçimde doldurulur.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::agac::AğaçDüğümü;
use crate::model::seri::AğaçHaritasıSerisi;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

/// Yerleşimi hesaplanmış hücre.
#[derive(Clone, Debug)]
pub struct AğaçHücresi {
    pub ad: String,
    pub değer: f64,
    pub alan: Dikdörtgen,
    pub renk: Renk,
    pub derinlik: usize,
    pub yaprak: bool,
}

/// Verilen değerleri (azalan sıralı) alana kareselleştirerek yerleştirir;
/// her değer için bir dikdörtgen döner.
fn kareselleştir(değerler: &[f64], alan: Dikdörtgen) -> Vec<Dikdörtgen> {
    let toplam: f64 = değerler.iter().sum();
    let mut sonuç = Vec::with_capacity(değerler.len());
    if toplam <= 0.0 || alan.genişlik <= 0.0 || alan.yükseklik <= 0.0 {
        for _ in değerler {
            sonuç.push(Dikdörtgen::yeni(alan.x, alan.y, 0.0, 0.0));
        }
        return sonuç;
    }
    let ölçek = (alan.genişlik as f64 * alan.yükseklik as f64) / toplam;
    let alanlar: Vec<f64> = değerler.iter().map(|d| d * ölçek).collect();

    let mut kalan = alan;
    let mut i = 0usize;
    while i < alanlar.len() {
        // Satırı büyüt: en kötü oran iyileştiği sürece ekle.
        let kısa_kenar = kalan.genişlik.min(kalan.yükseklik) as f64;
        let mut satır_sonu = i + 1;
        let mut en_iyi_oran = f64::INFINITY;
        while let Some(satır) = alanlar.get(i..satır_sonu) {
            let oran = en_kötü_oran(satır, kısa_kenar.max(1e-9));
            if oran <= en_iyi_oran && satır_sonu <= alanlar.len() {
                en_iyi_oran = oran;
                if satır_sonu == alanlar.len() {
                    break;
                }
                satır_sonu += 1;
            } else {
                satır_sonu -= 1;
                break;
            }
        }
        let satır_sonu = satır_sonu.clamp(i + 1, alanlar.len());
        let satır_alanı: f64 = alanlar
            .get(i..satır_sonu)
            .map(|s| s.iter().sum())
            .unwrap_or(0.0);

        // Satırı yerleştir: kısa kenara dik şerit.
        let yatay_şerit = kalan.genişlik >= kalan.yükseklik;
        if yatay_şerit {
            // Dikey şerit: soldan sağa dolan sütun.
            let şerit_genişliği = (satır_alanı / kalan.yükseklik.max(1.0) as f64) as f32;
            let mut y = kalan.y;
            for a in alanlar.get(i..satır_sonu).unwrap_or(&[]) {
                let yükseklik = (a / şerit_genişliği.max(1e-6) as f64) as f32;
                sonuç.push(Dikdörtgen::yeni(kalan.x, y, şerit_genişliği, yükseklik));
                y += yükseklik;
            }
            kalan = Dikdörtgen::yeni(
                kalan.x + şerit_genişliği,
                kalan.y,
                (kalan.genişlik - şerit_genişliği).max(0.0),
                kalan.yükseklik,
            );
        } else {
            // Yatay şerit: üstten alta dolan satır.
            let şerit_yüksekliği = (satır_alanı / kalan.genişlik.max(1.0) as f64) as f32;
            let mut x = kalan.x;
            for a in alanlar.get(i..satır_sonu).unwrap_or(&[]) {
                let genişlik = (a / şerit_yüksekliği.max(1e-6) as f64) as f32;
                sonuç.push(Dikdörtgen::yeni(x, kalan.y, genişlik, şerit_yüksekliği));
                x += genişlik;
            }
            kalan = Dikdörtgen::yeni(
                kalan.x,
                kalan.y + şerit_yüksekliği,
                kalan.genişlik,
                (kalan.yükseklik - şerit_yüksekliği).max(0.0),
            );
        }
        i = satır_sonu;
    }
    sonuç
}

/// Satırdaki en kötü (en uç) en-boy oranı.
fn en_kötü_oran(satır: &[f64], kenar: f64) -> f64 {
    let toplam: f64 = satır.iter().sum();
    if toplam <= 0.0 {
        return f64::INFINITY;
    }
    let şerit = toplam / kenar;
    satır
        .iter()
        .map(|a| {
            let diğer = a / şerit;
            (şerit / diğer.max(1e-12)).max(diğer / şerit.max(1e-12))
        })
        .fold(0.0, f64::max)
}

/// Düğüm listesini alana yerleştirir (özyinelemeli).
#[allow(clippy::too_many_arguments)]
fn düğümleri_yerleştir(
    düğümler: &[AğaçDüğümü],
    alan: Dikdörtgen,
    derinlik: usize,
    en_çok_derinlik: usize,
    boşluk: f32,
    üst_renk: Option<Renk>,
    palet: &dyn Fn(usize) -> Renk,
    hücreler: &mut Vec<AğaçHücresi>,
) {
    // Azalan değere göre sırala (kareselleştirme koşulu).
    let mut sıralı: Vec<(usize, &AğaçDüğümü, f64)> = düğümler
        .iter()
        .enumerate()
        .map(|(i, d)| (i, d, d.etkin_değer()))
        .filter(|(_, _, v)| *v > 0.0)
        .collect();
    sıralı.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let değerler: Vec<f64> = sıralı.iter().map(|(_, _, v)| *v).collect();
    let kutular = kareselleştir(&değerler, alan);

    for ((özgün_sıra, düğüm, değer), kutu) in sıralı.iter().zip(kutular) {
        let kutu = Dikdörtgen::yeni(
            kutu.x + boşluk / 2.0,
            kutu.y + boşluk / 2.0,
            (kutu.genişlik - boşluk).max(0.5),
            (kutu.yükseklik - boşluk).max(0.5),
        );
        let renk = düğüm.renk.unwrap_or_else(|| match üst_renk {
            // Alt düzeyler üst rengin açık tonları.
            Some(üst) => üst.karıştır(Renk::BEYAZ, 0.18 * derinlik as f32),
            None => palet(*özgün_sıra),
        });
        hücreler.push(AğaçHücresi {
            ad: düğüm.ad.clone(),
            değer: *değer,
            alan: kutu,
            renk,
            derinlik,
            yaprak: düğüm.yaprak_mı() || derinlik >= en_çok_derinlik,
        });
        if !düğüm.yaprak_mı() && derinlik < en_çok_derinlik {
            // Dal başlığı için üstten pay bırak.
            let iç = Dikdörtgen::yeni(
                kutu.x + 2.0,
                kutu.y + 16.0,
                (kutu.genişlik - 4.0).max(0.5),
                (kutu.yükseklik - 18.0).max(0.5),
            );
            düğümleri_yerleştir(
                &düğüm.çocuklar,
                iç,
                derinlik + 1,
                en_çok_derinlik,
                boşluk,
                Some(renk),
                palet,
                hücreler,
            );
        }
    }
}

/// Ağaç haritasını çizer ve isabet bölgelerini toplar.
pub fn ağaç_haritası_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &AğaçHaritasıSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let alan = Dikdörtgen::yeni(
        tuval.x + seri.sol.çöz(tuval.genişlik),
        tuval.y + seri.üst.çöz(tuval.yükseklik),
        seri.genişlik.çöz(tuval.genişlik),
        seri.yükseklik.çöz(tuval.yükseklik),
    );
    let mut hücreler = Vec::new();
    düğümleri_yerleştir(
        &seri.kökler,
        alan,
        0,
        seri.en_çok_derinlik.max(1),
        seri.hücre_boşluğu,
        None,
        palet,
        &mut hücreler,
    );

    let opaklık = ilerleme.clamp(0.0, 1.0);
    for hücre in &hücreler {
        çizici.dikdörtgen(
            hücre.alan,
            &Dolgu::Düz(hücre.renk.opaklık(opaklık)),
            [2.0; 4],
            Some((1.0, Renk::BEYAZ)),
        );
        // Etiket: hücreye sığıyorsa.
        let boyut = if hücre.derinlik == 0 { tema::YAZI_KÜÇÜK } else { 11.0 };
        let (metin_g, metin_y) = çizici.yazı_ölç(&hücre.ad, boyut);
        if metin_g + 6.0 < hücre.alan.genişlik && metin_y < hücre.alan.yükseklik {
            let parlaklık = 0.299 * hücre.renk.kırmızı
                + 0.587 * hücre.renk.yeşil
                + 0.114 * hücre.renk.mavi;
            let yazı_rengi =
                if parlaklık < 0.55 { Renk::BEYAZ } else { tema::BİRİNCİL_METİN };
            if hücre.yaprak {
                çizici.yazı(
                    &hücre.ad,
                    hücre.alan.merkez(),
                    YatayHiza::Orta,
                    DikeyHiza::Orta,
                    boyut,
                    yazı_rengi,
                    false,
                );
            } else {
                çizici.yazı(
                    &hücre.ad,
                    (hücre.alan.x + 4.0, hücre.alan.y + 2.0),
                    YatayHiza::Sol,
                    DikeyHiza::Üst,
                    boyut,
                    yazı_rengi,
                    true,
                );
            }
        }
        if hücre.yaprak {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası: isabetler.len(),
                seri_adı: seri.ad.clone(),
                ad: Some(hücre.ad.clone()),
                değer: Some(hücre.değer),
                geometri: İsabetGeometrisi::Dikdörtgen(hücre.alan),
            });
        }
    }
}

/// Hücre değer metni (ipucu için).
pub fn hücre_değer_metni(değer: f64) -> String {
    binlik_ayır(değer)
}
