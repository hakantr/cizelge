//! Radar (örümcek ağı) koordinatı ve serisi — `echarts/src/coord/radar` ile
//! `chart/radar` karşılığı.

use std::collections::HashSet;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::pasta::zengin_etiketi_hizalı_yaz;
use crate::grafik::sembol_stilli_çiz;
use crate::koordinat::Dikdörtgen;
use crate::model::deger::VeriÖğesi;
use crate::model::radar::{RadarKoordinatı, RadarŞekli};
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{RadarDurumYaması, RadarSerisi, Sembol, Seri};
use crate::model::stil::{AlanStili, Etiket, EtiketKonumu, ÇizgiStili, ÇizgiTürü, ÖğeStili};
use crate::model::veri_kumesi::BoyutSeçici;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yardimci::sayi::{GüzelKip, güzel_sayı, nicelik_üssü, yuvarla};

/// Bir gösterge kolunun çözülmüş sayısal kapsamı.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadarGöstergeKapsamı {
    pub en_az: f64,
    pub en_çok: f64,
}

/// Çözülmüş radar geometrisi.
#[derive(Clone, Debug)]
pub struct RadarDüzeni {
    pub merkez: (f32, f32),
    pub iç_yarıçap: f32,
    pub yarıçap: f32,
    /// Her göstergenin ekran yön vektörü (birim).
    pub yönler: Vec<(f32, f32)>,
    pub kapsamlar: Vec<RadarGöstergeKapsamı>,
}

fn radar_aralığını_artır(aralık: f64) -> f64 {
    let üs = nicelik_üssü(aralık);
    let üs10 = 10_f64.powi(üs);
    let f = (aralık / üs10).round() as i32;
    let sonraki = match f {
        0 => 1,
        2 => 3,
        3 => 5,
        1 | 5 => f * 2,
        _ => f.max(1) * 2,
    };
    yuvarla(sonraki as f64 * üs10, (-üs).max(0) as usize)
}

/// `axisAlignTicks.scaleCalcAlign`in radar için kullandığı, 0..splitNumber
/// yapay ölçeğine hizalanmış kapsam. Her gösterge tam aynı sayıda halka
/// üretir; otomatik uçlar veri kapsamını küçültmeden güzel adımlara açılır.
fn radar_hizalı_kapsam(
    mut veri: [f64; 2],
    sabit_en_az: Option<f64>,
    sabit_en_çok: Option<f64>,
    sıfırı_içer: bool,
    bölme: usize,
) -> RadarGöstergeKapsamı {
    if !veri[0].is_finite() || !veri[1].is_finite() {
        veri = [0.0, 1.0];
    }
    if sıfırı_içer {
        veri[0] = veri[0].min(0.0);
        veri[1] = veri[1].max(0.0);
    }
    if let Some(en_az) = sabit_en_az {
        veri[0] = en_az;
    }
    if let Some(en_çok) = sabit_en_çok {
        veri[1] = en_çok;
    }
    if veri[1] < veri[0] {
        veri.reverse();
    }
    if veri[0] == veri[1] {
        if veri[0] == 0.0 {
            veri[1] = 1.0;
        } else if sabit_en_çok.is_some() {
            veri[0] -= veri[0].abs() / 2.0;
        } else {
            let genişleme = veri[0].abs() / 2.0;
            veri[0] -= genişleme;
            veri[1] += genişleme;
        }
    }
    let bölme = bölme.max(1);
    if sabit_en_az.is_some() && sabit_en_çok.is_some() {
        return RadarGöstergeKapsamı {
            en_az: veri[0],
            en_çok: veri[1],
        };
    }
    let mut aralık = güzel_sayı((veri[1] - veri[0]) / bölme as f64, GüzelKip::EnKüçük);
    for _ in 0..50 {
        let h = (-nicelik_üssü(aralık)).max(0) as usize + 2;
        if let Some(en_az) = sabit_en_az {
            let en_çok = yuvarla(en_az + aralık * bölme as f64, h);
            if en_çok >= veri[1] - aralık * 1e-9 {
                return RadarGöstergeKapsamı { en_az, en_çok };
            }
        } else if let Some(en_çok) = sabit_en_çok {
            let en_az = yuvarla(en_çok - aralık * bölme as f64, h);
            if en_az <= veri[0] + aralık * 1e-9 {
                return RadarGöstergeKapsamı { en_az, en_çok };
            }
        } else {
            let mut en_az = yuvarla((veri[0] / aralık).ceil() * aralık, h);
            let mut en_çok = yuvarla((veri[1] / aralık).floor() * aralık, h);
            let mevcut = ((en_çok - en_az) / aralık).round() as isize;
            if mevcut <= bölme as isize {
                let fazla = (bölme as isize - mevcut).max(0) as usize;
                if sıfırı_içer && veri[0] == 0.0 {
                    en_çok += aralık * fazla as f64;
                } else if sıfırı_içer && veri[1] == 0.0 {
                    en_az -= aralık * fazla as f64;
                } else {
                    let alt = fazla / 2;
                    let üst = fazla - alt;
                    en_az -= aralık * alt as f64;
                    en_çok += aralık * üst as f64;
                }
                en_az = yuvarla(en_az, h);
                en_çok = yuvarla(en_çok, h);
                if en_az <= veri[0] + aralık * 1e-9 && en_çok >= veri[1] - aralık * 1e-9 {
                    return RadarGöstergeKapsamı { en_az, en_çok };
                }
            }
        }
        aralık = radar_aralığını_artır(aralık);
    }
    RadarGöstergeKapsamı {
        en_az: veri[0],
        en_çok: veri[1],
    }
}

/// Radar koordinat geometrisini açık göstergelerle çözer. Bu geriye uyumlu
/// yol, otomatik göstergelerde `[0, 1]` kapsamına düşer.
pub fn radar_düzeni(koordinat: &RadarKoordinatı, tuval: Dikdörtgen) -> RadarDüzeni {
    radar_düzeni_serilerle(koordinat, tuval, &[])
}

/// Radar koordinat geometrisini, aynı `radarIndex`e bağlı serilerden
/// türetilen gösterge kapsamlarıyla çözer.
pub fn radar_düzeni_serilerle(
    koordinat: &RadarKoordinatı,
    tuval: Dikdörtgen,
    seriler: &[&RadarSerisi],
) -> RadarDüzeni {
    let merkez = (
        tuval.x + koordinat.merkez.0.çöz(tuval.genişlik),
        tuval.y + koordinat.merkez.1.çöz(tuval.yükseklik),
    );
    let taban = tuval.genişlik.min(tuval.yükseklik) / 2.0;
    let mut iç_yarıçap = koordinat.iç_yarıçap.çöz(taban).max(0.0);
    let mut yarıçap = koordinat.yarıçap.çöz(taban).max(0.0);
    if yarıçap < iç_yarıçap {
        std::mem::swap(&mut yarıçap, &mut iç_yarıçap);
    }
    let n = koordinat.göstergeler.len().max(1);
    let başlangıç = koordinat.başlangıç_açısı.to_radians();
    let işaret = if koordinat.saat_yönü { -1.0 } else { 1.0 };
    let yönler = (0..n)
        .map(|i| {
            let açı = başlangıç + işaret * i as f32 * std::f32::consts::TAU / n as f32;
            // ECharts koordinatı yukarı yönlü Y kullanıp ekrana `cy-r*sin`
            // ile döner; bu işaret gösterge sırasını kayıpsız korur.
            (açı.cos(), -açı.sin())
        })
        .collect();
    let kapsamlar = koordinat
        .göstergeler
        .iter()
        .enumerate()
        .map(|(gösterge_sırası, gösterge)| {
            let mut veri_kapsamı = [f64::INFINITY, f64::NEG_INFINITY];
            for seri in seriler {
                for öğe in &seri.veri {
                    let Some(değer) = öğe
                        .değer
                        .dizi()
                        .and_then(|değerler| değerler.get(gösterge_sırası))
                        .copied()
                        .filter(|değer| değer.is_finite())
                    else {
                        continue;
                    };
                    veri_kapsamı[0] = veri_kapsamı[0].min(değer);
                    veri_kapsamı[1] = veri_kapsamı[1].max(değer);
                }
            }
            if !veri_kapsamı[0].is_finite() || !veri_kapsamı[1].is_finite() {
                veri_kapsamı = if gösterge.en_çok_belirtildi || gösterge.en_az_belirtildi {
                    [gösterge.en_az, gösterge.en_çok]
                } else {
                    [0.0, 1.0]
                };
            }
            // RadarModel, yalnız pozitif `max` verildiğinde `min: 0`ı;
            // yalnız negatif `min` verildiğinde `max: 0`ı indicator
            // seçeneğine yazar. Bu uçlar alignTicks aşamasında sabittir.
            let sabit_en_az = gösterge
                .en_az_belirtildi
                .then_some(gösterge.en_az)
                .or_else(|| (gösterge.en_çok_belirtildi && gösterge.en_çok > 0.0).then_some(0.0));
            let sabit_en_çok = gösterge
                .en_çok_belirtildi
                .then_some(gösterge.en_çok)
                .or_else(|| (gösterge.en_az_belirtildi && gösterge.en_az < 0.0).then_some(0.0));
            radar_hizalı_kapsam(
                veri_kapsamı,
                sabit_en_az,
                sabit_en_çok,
                koordinat.sıfırı_içer,
                koordinat.bölme_sayısı,
            )
        })
        .collect();
    RadarDüzeni {
        merkez,
        iç_yarıçap,
        yarıçap,
        yönler,
        kapsamlar,
    }
}

fn halka_noktaları(düzen: &RadarDüzeni, yarıçap: f32) -> Vec<(f32, f32)> {
    düzen
        .yönler
        .iter()
        .map(|(x, y)| (düzen.merkez.0 + yarıçap * x, düzen.merkez.1 + yarıçap * y))
        .collect()
}

fn çokgen_yolu(noktalar: &[(f32, f32)]) -> Yol {
    let mut yol = Yol::yeni();
    for (sıra, nokta) in noktalar.iter().enumerate() {
        if sıra == 0 {
            yol.taşı(*nokta);
        } else {
            yol.çiz(*nokta);
        }
    }
    yol.kapat();
    yol
}

fn çokgen_halka_yolu(dış: &[(f32, f32)], iç: &[(f32, f32)]) -> Yol {
    let mut yol = çokgen_yolu(dış);
    if iç
        .iter()
        .any(|nokta| (nokta.0 - iç[0].0).abs() > 1e-5 || (nokta.1 - iç[0].1).abs() > 1e-5)
    {
        for (sıra, nokta) in iç.iter().rev().enumerate() {
            if sıra == 0 {
                yol.taşı(*nokta);
            } else {
                yol.çiz(*nokta);
            }
        }
        yol.kapat();
    }
    yol
}

fn renk_döndür(renkler: &[Renk], sıra: usize, öntanımlı: Renk) -> Renk {
    if renkler.is_empty() {
        öntanımlı
    } else {
        renkler[sıra % renkler.len()]
    }
}

fn dolgu_döndür(renkler: &[Dolgu], sıra: usize) -> Dolgu {
    if renkler.is_empty() {
        let öntanımlı = tema::bölme_alanı_renkleri();
        Dolgu::Düz(öntanımlı[sıra % öntanımlı.len()])
    } else {
        renkler[sıra % renkler.len()].clone()
    }
}

/// Ağ (ızgara) çizimi: bölme halkaları, bölme alanları, kollar ve gösterge
/// adları. Bileşen her koordinat için yalnız bir kez çağrılmalıdır.
pub fn radar_ağı_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    koordinat: &RadarKoordinatı,
    düzen: &RadarDüzeni,
) {
    let bölme = koordinat.bölme_sayısı.max(1);
    let açıklık = düzen.yarıçap - düzen.iç_yarıçap;

    // ECharts `splitArea`, iç halkadan dış halkaya renkleri döndürür ve
    // alanları gerçek halkalar olarak boyar; saydam bir iç bant dış bandın
    // rengini miras almamalıdır.
    if koordinat.bölme_alanı_göster && koordinat.bölme_alanı.göster {
        for sıra in 0..bölme {
            let r0 = düzen.iç_yarıçap + açıklık * sıra as f32 / bölme as f32;
            let r1 = düzen.iç_yarıçap + açıklık * (sıra + 1) as f32 / bölme as f32;
            let dolgu = dolgu_döndür(&koordinat.bölme_alanı.renkler, sıra)
                .opaklık(koordinat.bölme_alanı.opaklık);
            match koordinat.şekil {
                RadarŞekli::Daire => {
                    if koordinat.bölme_alanı.gölge_bulanıklığı > 0.0
                        && let Some(gölge) = koordinat.bölme_alanı.gölge_rengi
                    {
                        // Ring gölgesi tam diskten değil, splitArea'nın
                        // gerçek halka maskesinden türetilir. Tam disk
                        // maskesi iç halkaların tamamını `shadowColor` ile
                        // karartır; zrender `graphic.Ring` deliği korur.
                        let dış_yol = crate::cizim::yuzey::dilim_yolu(
                            düzen.merkez,
                            r0,
                            r1,
                            0.0,
                            std::f32::consts::TAU * 0.999_999,
                        );
                        çizici.yol_gölgesi(
                            &dış_yol,
                            gölge,
                            koordinat.bölme_alanı.gölge_bulanıklığı,
                            koordinat.bölme_alanı.gölge_kayması,
                        );
                    }
                    çizici.dilim(
                        düzen.merkez,
                        r0,
                        r1,
                        0.0,
                        std::f32::consts::TAU * 0.999_999,
                        &dolgu,
                        None,
                    );
                }
                RadarŞekli::Çokgen => {
                    let dış = halka_noktaları(düzen, r1);
                    let iç = halka_noktaları(düzen, r0);
                    let yol = çokgen_halka_yolu(&dış, &iç);
                    if koordinat.bölme_alanı.gölge_bulanıklığı > 0.0
                        && let Some(gölge) = koordinat.bölme_alanı.gölge_rengi
                    {
                        çizici.yol_gölgesi(
                            &yol,
                            gölge,
                            koordinat.bölme_alanı.gölge_bulanıklığı,
                            koordinat.bölme_alanı.gölge_kayması,
                        );
                    }
                    çizici.yol_doldur(&yol, &dolgu);
                }
            }
        }
    }

    if koordinat.bölme_çizgisi.göster {
        for sıra in 0..=bölme {
            let yarıçap = düzen.iç_yarıçap + açıklık * sıra as f32 / bölme as f32;
            if yarıçap <= 1e-6 {
                continue;
            }
            let renk = renk_döndür(
                &koordinat.bölme_çizgisi.renkler,
                sıra,
                tema::bölme_çizgisi(),
            )
            .opaklık(koordinat.bölme_çizgisi.stil.opaklık);
            let yol = match koordinat.şekil {
                RadarŞekli::Daire => crate::cizim::yuzey::daire_yolu(düzen.merkez, yarıçap),
                RadarŞekli::Çokgen => çokgen_yolu(&halka_noktaları(düzen, yarıçap)),
            };
            çizici.yol_çiz(
                &yol,
                koordinat.bölme_çizgisi.stil.kalınlık,
                renk,
                koordinat.bölme_çizgisi.stil.tür,
            );
        }
    }

    // Kollar ve gösterge adları.
    for (sıra, (x, y)) in düzen.yönler.iter().enumerate() {
        if koordinat.eksen_çizgisi.göster {
            let başlangıç = (
                düzen.merkez.0 + düzen.iç_yarıçap * x,
                düzen.merkez.1 + düzen.iç_yarıçap * y,
            );
            let uç = (
                düzen.merkez.0 + düzen.yarıçap * x,
                düzen.merkez.1 + düzen.yarıçap * y,
            );
            let renk = renk_döndür(&koordinat.eksen_çizgisi.renkler, sıra, tema::nötr_20())
                .opaklık(koordinat.eksen_çizgisi.stil.opaklık);
            çizici.çizgi(
                başlangıç,
                uç,
                koordinat.eksen_çizgisi.stil.kalınlık,
                renk,
                koordinat.eksen_çizgisi.stil.tür,
            );
        }

        let Some(gösterge) = koordinat.göstergeler.get(sıra) else {
            continue;
        };
        if !koordinat.eksen_adı.göster {
            continue;
        }
        let metin = koordinat
            .eksen_adı
            .biçimleyici
            .as_ref()
            .map(|biçimleyici| biçimleyici.uygula(0.0, &gösterge.ad))
            .unwrap_or_else(|| gösterge.ad.clone());
        let konum = (
            düzen.merkez.0 + (düzen.yarıçap + koordinat.eksen_adı.boşluk) * x,
            düzen.merkez.1 + (düzen.yarıçap + koordinat.eksen_adı.boşluk) * y,
        );
        let yatay = if x.abs() < 0.2 {
            YatayHiza::Orta
        } else if *x > 0.0 {
            YatayHiza::Sol
        } else {
            YatayHiza::Sağ
        };
        // AxisBuilder `endTextLayout`, eğik ve yatay kolların adını uç
        // noktasına dikey ortalar; yalnız tam dikey kollarda metni radarın
        // dışına doğru üstten/alttan hizalar. Yönün yalnız Y işaretine
        // bakmak eğik adları yarım satır kadar içeri/dışarı kaydırır.
        let dikey = if x.abs() < 0.2 {
            if *y > 0.0 {
                DikeyHiza::Üst
            } else {
                DikeyHiza::Alt
            }
        } else {
            DikeyHiza::Orta
        };
        let etiket = Etiket::yeni()
            .göster(true)
            .yazı(koordinat.eksen_adı.yazı.clone());
        zengin_etiketi_hizalı_yaz(
            çizici,
            &metin,
            &etiket,
            konum,
            yatay,
            dikey,
            gösterge.renk.unwrap_or_else(tema::ikincil_metin),
            0.0,
        );
    }
}

fn öğe_stili_yama_uygula(taban: &ÖğeStili, yama: &ÖğeStili) -> ÖğeStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk = yama.renk.clone();
    }
    if yama.kenarlık_rengi.is_some() {
        sonuç.kenarlık_rengi = yama.kenarlık_rengi;
    }
    if yama.kenarlık_kalınlığı != 0.0 {
        sonuç.kenarlık_kalınlığı = yama.kenarlık_kalınlığı;
    }
    if yama.kenarlık_türü != ÇizgiTürü::Düz {
        sonuç.kenarlık_türü = yama.kenarlık_türü;
    }
    if yama.kenarlık_yarıçapı != [0.0; 4] {
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

fn radar_normal_öğe_stili(seri: &RadarSerisi, sıra: usize) -> ÖğeStili {
    let mut stil = seri.öğe_stili.clone();
    if let Some(yama) = seri.veri.get(sıra).and_then(|öğe| öğe.stil.as_ref()) {
        stil = öğe_stili_yama_uygula(&stil, yama);
    }
    if let Some(yama) = seri
        .veri_ayarları
        .get(sıra)
        .and_then(|ayar| ayar.öğe_stili.as_ref())
    {
        stil = öğe_stili_yama_uygula(&stil, yama);
    }
    stil
}

fn radar_görsel_değeri(öğe: &VeriÖğesi, boyut: Option<&BoyutSeçici>) -> Option<f64> {
    match boyut {
        Some(BoyutSeçici::Sıra(sıra)) => öğe
            .değer
            .dizi()
            .and_then(|değerler| değerler.get(*sıra))
            .copied()
            .or_else(|| öğe.boyutlar.get(*sıra).and_then(|(_, değer)| değer.sayı())),
        Some(BoyutSeçici::Ad(ad)) => öğe.boyut(ad).and_then(|değer| değer.sayı()),
        None => öğe
            .değer
            .dizi()
            .and_then(|değerler| değerler.last())
            .copied()
            .or_else(|| öğe.değer.sayı()),
    }
}

pub fn radar_görsel_kapsamı(
    seçenekler: &GrafikSeçenekleri,
    eşleme: &crate::model::gorsel_esleme::GörselEşleme,
) -> [f64; 2] {
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    for (seri_sırası, seri) in seçenekler.seriler.iter().enumerate() {
        if !eşleme.seriye_uygulanır_mı(seri_sırası) {
            continue;
        }
        let Seri::Radar(radar) = seri else { continue };
        for öğe in &radar.veri {
            let Some(değer) =
                radar_görsel_değeri(öğe, eşleme.boyut.as_ref()).filter(|değer| değer.is_finite())
            else {
                continue;
            };
            kapsam[0] = kapsam[0].min(değer);
            kapsam[1] = kapsam[1].max(değer);
        }
    }
    if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
        [0.0, 1.0]
    } else {
        kapsam
    }
}

/// Radar veri öğesinin palet/itemStyle/visualMap zincirinden çözülen rengi.
/// Legend ve seri görünümü aynı işlevi kullanarak renk sapmasını önler.
pub fn radar_öğe_rengi(
    seçenekler: &GrafikSeçenekleri,
    seri_sırası: usize,
    veri_sırası: usize,
    palet_sırası: usize,
) -> Renk {
    let Some(Seri::Radar(seri)) = seçenekler.seriler.get(seri_sırası) else {
        return seçenekler.palet_rengi(palet_sırası);
    };
    let Some(öğe) = seri.veri.get(veri_sırası) else {
        return seçenekler.palet_rengi(palet_sırası);
    };
    let mut renk = radar_normal_öğe_stili(seri, veri_sırası)
        .renk
        .as_ref()
        .map(Dolgu::temsilî)
        .unwrap_or_else(|| seçenekler.palet_rengi(palet_sırası));
    for eşleme in seçenekler.seri_görsel_eşlemeleri(seri_sırası) {
        let Some(değer) = radar_görsel_değeri(öğe, eşleme.boyut.as_ref()) else {
            continue;
        };
        let kapsam = eşleme.kapsam_çöz(radar_görsel_kapsamı(seçenekler, eşleme));
        renk = eşleme.rengi_uygula(değer, kapsam, renk);
    }
    renk
}

/// Önceki radar veri öğelerinin toplamı; `colorBy: data` palet kapsamını
/// seri sınırlarında sıfırlamayan ECharts görsel hattının karşılığı.
pub fn radar_palet_başlangıcı(seçenekler: &GrafikSeçenekleri, seri_sırası: usize) -> usize {
    seçenekler
        .seriler
        .iter()
        .take(seri_sırası)
        .filter_map(|seri| match seri {
            Seri::Radar(radar) => Some(radar.veri.len()),
            _ => None,
        })
        .sum()
}

fn radar_veri_noktaları(
    değerler: &[f64], düzen: &RadarDüzeni, ilerleme: f32
) -> Vec<(f32, f32)> {
    düzen
        .yönler
        .iter()
        .enumerate()
        .map(|(gösterge_sırası, (x, y))| {
            let kapsam =
                düzen
                    .kapsamlar
                    .get(gösterge_sırası)
                    .copied()
                    .unwrap_or(RadarGöstergeKapsamı {
                        en_az: 0.0,
                        en_çok: 1.0,
                    });
            let değer = değerler
                .get(gösterge_sırası)
                .copied()
                .unwrap_or(kapsam.en_az);
            let oran = if kapsam.en_çok > kapsam.en_az {
                ((değer - kapsam.en_az) / (kapsam.en_çok - kapsam.en_az)).clamp(0.0, 1.0) as f32
            } else {
                0.0
            };
            let yarıçap = (düzen.iç_yarıçap + (düzen.yarıçap - düzen.iç_yarıçap) * oran)
                * ilerleme.clamp(0.0, 1.0);
            (düzen.merkez.0 + yarıçap * x, düzen.merkez.1 + yarıçap * y)
        })
        .collect()
}

/// Fare konumunun vurduğu en üst radar veri öğesi. Alan dolgusu kapalı olsa
/// da ECharts itemGroup'undaki çizgi/sembol hedefi için çokgen içi güvenli
/// bir isabet yüzeyi sağlar.
pub fn radar_vurgusu(
    seri: &RadarSerisi,
    düzen: &RadarDüzeni,
    kapalı: &HashSet<String>,
    ilerleme: f32,
    fare: (f32, f32),
) -> Option<usize> {
    seri.veri
        .iter()
        .enumerate()
        .rev()
        .find_map(|(veri_sırası, öğe)| {
            if öğe.ad.as_ref().is_some_and(|ad| kapalı.contains(ad)) {
                return None;
            }
            let değerler = öğe.değer.dizi()?;
            let noktalar = radar_veri_noktaları(değerler, düzen, ilerleme);
            İsabetGeometrisi::Çokgen { noktalar }
                .içeriyor_mu(fare)
                .then_some(veri_sırası)
        })
}

fn durum_uygula(
    durum: &RadarDurumYaması,
    çizgi: &mut ÇizgiStili,
    alan: &mut Option<AlanStili>,
    öğe: &mut ÖğeStili,
    etiket: &mut Etiket,
) {
    if let Some(stil) = &durum.çizgi_stili {
        *çizgi = stil.clone();
    }
    if let Some(stil) = &durum.alan_stili {
        *alan = Some(stil.clone());
    }
    if let Some(stil) = &durum.öğe_stili {
        *öğe = öğe_stili_yama_uygula(öğe, stil);
    }
    if let Some(yeni) = &durum.etiket {
        *etiket = yeni.clone();
    }
}

fn etiket_konumu(
    etiket: &Etiket,
    nokta: (f32, f32),
    sembol_boyutu: f32,
) -> ((f32, f32), YatayHiza, DikeyHiza) {
    let yarı = sembol_boyutu / 2.0;
    match etiket.konum {
        EtiketKonumu::Üst => (
            (nokta.0, nokta.1 - yarı - etiket.uzaklık),
            YatayHiza::Orta,
            DikeyHiza::Alt,
        ),
        EtiketKonumu::Alt => (
            (nokta.0, nokta.1 + yarı + etiket.uzaklık),
            YatayHiza::Orta,
            DikeyHiza::Üst,
        ),
        EtiketKonumu::Sol => (
            (nokta.0 - yarı - etiket.uzaklık, nokta.1),
            YatayHiza::Sağ,
            DikeyHiza::Orta,
        ),
        EtiketKonumu::Sağ => (
            (nokta.0 + yarı + etiket.uzaklık, nokta.1),
            YatayHiza::Sol,
            DikeyHiza::Orta,
        ),
        _ => (nokta, YatayHiza::Orta, DikeyHiza::Orta),
    }
}

/// Radar serisini çizer: her veri öğesi bir çokgendir.
#[allow(clippy::too_many_arguments)]
pub fn radar_serisi_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &RadarSerisi,
    genel_sıra: usize,
    koordinat: &RadarKoordinatı,
    düzen: &RadarDüzeni,
    seçenekler: &GrafikSeçenekleri,
    kapalı: &HashSet<String>,
    ilerleme: f32,
    vurgulu: Option<usize>,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let ilerleme = ilerleme.clamp(0.0, 1.0);
    let palet_başlangıcı = radar_palet_başlangıcı(seçenekler, genel_sıra);
    for (veri_sırası, veri_öğesi) in seri.veri.iter().enumerate() {
        let ad = veri_öğesi
            .ad
            .clone()
            .unwrap_or_else(|| format!("{veri_sırası}"));
        if kapalı.contains(&ad) {
            continue;
        }
        let Some(değerler) = veri_öğesi.değer.dizi() else {
            continue;
        };
        let ayar = seri.veri_ayarları.get(veri_sırası);
        let renk = radar_öğe_rengi(
            seçenekler,
            genel_sıra,
            veri_sırası,
            palet_başlangıcı + veri_sırası,
        );

        let noktalar = radar_veri_noktaları(değerler, düzen, ilerleme);
        if noktalar.len() < 3 {
            continue;
        }
        let yol = çokgen_yolu(&noktalar);

        let mut çizgi_stili = ayar
            .and_then(|ayar| ayar.çizgi_stili.clone())
            .unwrap_or_else(|| seri.çizgi_stili.clone());
        let mut alan_stili = ayar
            .and_then(|ayar| ayar.alan_stili.clone())
            .or_else(|| seri.alan_stili.clone());
        let mut öğe_stili = radar_normal_öğe_stili(seri, veri_sırası);
        let mut etiket = ayar
            .and_then(|ayar| ayar.etiket.clone())
            .unwrap_or_else(|| seri.etiket.clone());
        if let Some(yama) = &veri_öğesi.etiket {
            etiket = yama.uygula(&etiket);
        }
        let seçili = veri_öğesi.seçili;
        if seçili {
            durum_uygula(
                &seri.seçili,
                &mut çizgi_stili,
                &mut alan_stili,
                &mut öğe_stili,
                &mut etiket,
            );
            if let Some(ayar) = ayar {
                durum_uygula(
                    &ayar.seçili,
                    &mut çizgi_stili,
                    &mut alan_stili,
                    &mut öğe_stili,
                    &mut etiket,
                );
            }
        }
        if vurgulu == Some(veri_sırası) {
            durum_uygula(
                &seri.vurgu,
                &mut çizgi_stili,
                &mut alan_stili,
                &mut öğe_stili,
                &mut etiket,
            );
            if let Some(ayar) = ayar {
                durum_uygula(
                    &ayar.vurgu,
                    &mut çizgi_stili,
                    &mut alan_stili,
                    &mut öğe_stili,
                    &mut etiket,
                );
            }
        }
        let opaklık = öğe_stili.opaklık.unwrap_or(1.0) * ilerleme;

        // RadarView çocuk sırası: polyline, polygon, symbols.
        let çizgi_rengi = çizgi_stili
            .renk
            .unwrap_or(renk)
            .opaklık(çizgi_stili.opaklık * opaklık);
        if çizgi_stili.gölge_bulanıklığı > 0.0
            && let Some(gölge) = çizgi_stili.gölge_rengi
        {
            çizici.yol_çizgi_gölgesi(
                &yol,
                çizgi_stili.kalınlık,
                çizgi_stili.tür,
                gölge.opaklık(opaklık),
                çizgi_stili.gölge_bulanıklığı,
                çizgi_stili.gölge_kayması,
            );
        }
        çizici.yol_çiz(&yol, çizgi_stili.kalınlık, çizgi_rengi, çizgi_stili.tür);

        if let Some(alan) = &alan_stili {
            let dolgu = alan
                .renk
                .clone()
                .unwrap_or(Dolgu::Düz(renk))
                .opaklık(alan.opaklık * opaklık);
            çizici.yol_doldur(&yol, &dolgu);
        }

        let sembol = ayar
            .and_then(|ayar| ayar.sembol.as_ref())
            .unwrap_or(&seri.sembol);
        let sembol_boyutu = ayar
            .and_then(|ayar| ayar.sembol_boyutu)
            .unwrap_or(seri.sembol_boyutu);
        if seri.sembol_göster && !matches!(sembol, Sembol::Yok) {
            let dolgu = öğe_stili.renk.as_ref();
            let kenarlık = öğe_stili
                .kenarlık_rengi
                .filter(|_| öğe_stili.kenarlık_kalınlığı > 0.0)
                .map(|renk| (öğe_stili.kenarlık_kalınlığı, renk));
            for (gösterge_sırası, nokta) in noktalar.iter().enumerate() {
                sembol_stilli_çiz(
                    çizici,
                    sembol,
                    *nokta,
                    sembol_boyutu,
                    renk,
                    dolgu,
                    kenarlık,
                    opaklık,
                    false,
                );
                if etiket.göster {
                    let değer = değerler.get(gösterge_sırası).copied().unwrap_or_default();
                    let ham = binlik_ayır(değer);
                    let metin = etiket
                        .biçimleyici
                        .as_ref()
                        .map(|biçimleyici| {
                            biçimleyici.uygula_bağlamla(
                                değer,
                                &ham,
                                seri.ad.as_deref().unwrap_or(""),
                                &ad,
                            )
                        })
                        .unwrap_or(ham);
                    let (konum, yatay, dikey) = etiket_konumu(&etiket, *nokta, sembol_boyutu);
                    zengin_etiketi_hizalı_yaz(
                        çizici,
                        &metin,
                        &etiket,
                        konum,
                        yatay,
                        dikey,
                        renk.opaklık(opaklık),
                        0.0,
                    );
                }
            }
        }

        if !seri.sessiz && !koordinat.sessiz {
            isabetler.push(İsabetBölgesi {
                seri_sırası: genel_sıra,
                veri_sırası,
                seri_adı: seri.ad.clone(),
                ad: veri_öğesi.ad.clone(),
                değer: değerler.last().copied(),
                geometri: İsabetGeometrisi::Çokgen {
                    noktalar: noktalar.clone(),
                },
            });
        }
    }
}

/// Radar öğesinin ipucu satır metni: değerleri gösterge adlarıyla eşler.
pub fn radar_ipucu_satırları(
    seri: &RadarSerisi,
    koordinat: &RadarKoordinatı,
    veri_sırası: usize,
) -> Vec<(String, String)> {
    let Some(öğe) = seri.veri.get(veri_sırası) else {
        return Vec::new();
    };
    let Some(değerler) = öğe.değer.dizi() else {
        return Vec::new();
    };
    koordinat
        .göstergeler
        .iter()
        .zip(değerler.iter())
        .map(|(gösterge, değer)| (gösterge.ad.clone(), binlik_ayır(*değer)))
        .collect()
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn resmi_saat_yonu_olmayan_gosterge_sirasi_ustten_sola_ilerler() {
        let koordinat = RadarKoordinatı::yeni().göstergeler([
            ("üst", 100.0),
            ("sol üst", 100.0),
            ("sol alt", 100.0),
            ("alt", 100.0),
            ("sağ alt", 100.0),
            ("sağ üst", 100.0),
        ]);
        let düzen = radar_düzeni(&koordinat, Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0));
        assert!(düzen.yönler[0].0.abs() < 1e-5 && düzen.yönler[0].1 < -0.99);
        assert!(düzen.yönler[1].0 < 0.0 && düzen.yönler[1].1 < 0.0);
        assert!(düzen.yönler[3].0.abs() < 1e-5 && düzen.yönler[3].1 > 0.99);
    }

    #[test]
    fn otomatik_kapsam_butun_bagli_serilerden_guzel_bolmelere_hizalanir() {
        let koordinat = RadarKoordinatı::yeni().gösterge_listesi([
            crate::model::radar::RadarGöstergesi::otomatik("a"),
            crate::model::radar::RadarGöstergesi::otomatik("b"),
            crate::model::radar::RadarGöstergesi::otomatik("c"),
        ]);
        let seri = RadarSerisi::yeni().veri([
            ("x", vec![100.0, 8.0, -80.0]),
            ("y", vec![60.0, 5.0, -100.0]),
        ]);
        let düzen = radar_düzeni_serilerle(
            &koordinat,
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            &[&seri],
        );
        assert_eq!(
            düzen.kapsamlar[0],
            RadarGöstergeKapsamı {
                en_az: 0.0,
                en_çok: 100.0
            }
        );
        assert_eq!(
            düzen.kapsamlar[1],
            RadarGöstergeKapsamı {
                en_az: 0.0,
                en_çok: 10.0
            }
        );
        assert_eq!(
            düzen.kapsamlar[2],
            RadarGöstergeKapsamı {
                en_az: -100.0,
                en_çok: 0.0
            }
        );
    }
}
