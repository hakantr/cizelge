//! Kutupsal koordinat sistemi — `echarts/src/coord/polar` ve kutupsal
//! seri görünümlerinin karşılığı. Açısal ve radyal eksenler kategori ya da
//! sayısal ölçek taşıyabilir.

use std::collections::HashSet;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::hatlar::hatlar_çiz;
use crate::grafik::sembol_çiz;
use crate::koordinat::Dikdörtgen;
use crate::model::deger::VeriDeğeri;
use crate::model::eksen::EksenTürü;
use crate::model::kutupsal::KutupsalKoordinat;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{Sembol, Seri};
use crate::olcek::{AralıkÖlçeği, KategorikÖlçek, Ölçek};
use crate::renk::Dolgu;
use crate::tema;
use crate::yardimci::sayi::doğrusal_eşle;
use crate::yerlesim::yigin::YığınAralığı;

/// Çözülmüş kutupsal düzen.
pub struct KutupsalDüzen {
    pub merkez: (f32, f32),
    pub yarıçap: f32,
    pub açısal_ölçek: Ölçek,
    pub radyal_ölçek: Ölçek,
    /// Açısal eksen kategorik mi (bant yerleşimi)?
    pub açısal_kategorik: bool,
    /// Radyal eksen kategorik mi (eş merkezli bant yerleşimi)?
    pub radyal_kategorik: bool,
    pub açısal_kenar_boşluğu: bool,
    pub radyal_kenar_boşluğu: bool,
    pub radyal_ters: bool,
    pub başlangıç_açısı: f32,
    pub saat_yönü: bool,
}

impl KutupsalDüzen {
    /// Açısal değeri ekran radyanına çevirir (0 üstte, saat yönü).
    pub fn açı(&self, değer: f64) -> f32 {
        let oran = if self.açısal_kategorik {
            let n = self.açısal_ölçek.kategori_sayısı().max(1) as f64;
            if self.açısal_kenar_boşluğu {
                (değer + 0.5) / n
            } else {
                // Tam daire üzerindeki onBand=false kategori ekseni son
                // kategoriden sonra bir bant daha ayırır; ilk ve son etiket
                // aynı ışına yığılmaz.
                değer / n
            }
        } else {
            self.açısal_ölçek.oranla(değer)
        };
        let başlangıç = -(self.başlangıç_açısı as f64).to_radians();
        let yön = if self.saat_yönü { 1.0 } else { -1.0 };
        (başlangıç + yön * oran * std::f64::consts::TAU) as f32
    }

    /// Bant açıklığı (radyan) — kutupsal sütunlar için.
    pub fn bant_açısı(&self) -> f32 {
        let n = self.açısal_ölçek.kategori_sayısı().max(1) as f32;
        std::f32::consts::TAU / n
    }

    /// Radyal değeri yarıçapa çevirir.
    pub fn yarıçapa(&self, değer: f64) -> f32 {
        let oran = if self.radyal_kategorik {
            let n = self.radyal_ölçek.kategori_sayısı().max(1) as f64;
            if self.radyal_kenar_boşluğu {
                (değer + 0.5) / n
            } else if n > 1.0 {
                değer / (n - 1.0)
            } else {
                0.5
            }
        } else {
            match &self.radyal_ölçek {
                // Polar.dataToPoint varsayılan olarak clamp etmez. Özellikle
                // radiusAxis.min=0 ile negatif yarıçaplı gül eğrileri merkezin
                // ötesine taşınarak karşı yaprağı oluşturur.
                Ölçek::Aralık(ölçek) => {
                    doğrusal_eşle(değer, ölçek.kapsam, [0.0, 1.0], false)
                }
                _ => self.radyal_ölçek.oranla(değer),
            }
        };
        let oran = if self.radyal_ters { 1.0 - oran } else { oran };
        (oran as f32) * self.yarıçap
    }

    /// Veri çiftini ekran noktasına çevirir.
    pub fn nokta(&self, açısal: f64, radyal: f64) -> (f32, f32) {
        let açı = self.açı(açısal);
        let yarıçap = self.yarıçapa(radyal);
        (
            self.merkez.0 + yarıçap * açı.cos(),
            self.merkez.1 + yarıçap * açı.sin(),
        )
    }
}

/// ECharts polar veri boyut sırası `[radius, angle, ...]`dır. Üçüncü ve
/// sonraki boyutlar symbolSize/tooltip gibi görsel kanallar için korunur.
fn kutupsal_değerler(değer: &VeriDeğeri) -> Option<(f64, f64)> {
    match değer {
        VeriDeğeri::Çift([radyal, açısal]) => Some((*radyal, *açısal)),
        VeriDeğeri::Dizi(boyutlar) => Some((*boyutlar.first()?, *boyutlar.get(1)?)),
        _ => None,
    }
}

/// Kutupsal serilerin radyal kapsamını toplar ve düzeni kurar.
pub fn kutupsal_kur(
    koordinat: &KutupsalKoordinat,
    seçenekler: &GrafikSeçenekleri,
    aralıklar: &[Vec<YığınAralığı>],
    görünürler: &[bool],
    tuval: Dikdörtgen,
) -> KutupsalDüzen {
    let merkez = (
        tuval.x + koordinat.merkez.0.çöz(tuval.genişlik),
        tuval.y + koordinat.merkez.1.çöz(tuval.yükseklik),
    );
    let taban = tuval.genişlik.min(tuval.yükseklik) / 2.0;
    let yarıçap = koordinat.yarıçap.çöz(taban);

    // ECharts polar boyut sırası `[radius, angle]`dır. Tek değerli
    // kategori serilerinde radyal değer yığın aralığından, çiftlerde ise
    // doğrudan ilk boyuttan gelir.
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    let mut açısal_kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    let mut en_uzun = 0usize;
    let kapsa = |hedef: &mut [f64; 2], değer: f64| {
        if değer.is_finite() {
            hedef[0] = hedef[0].min(değer);
            hedef[1] = hedef[1].max(değer);
        }
    };
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        if !seri.kutupsal_mı() || !görünürler.get(i).copied().unwrap_or(false) {
            continue;
        }
        en_uzun = en_uzun.max(seri.veri().len());
        if let Seri::Hatlar(hatlar) = seri {
            for veri in &hatlar.veri {
                en_uzun = en_uzun.max(veri.koordinatlar.len());
                for nokta in &veri.koordinatlar {
                    if let (Some(açısal), Some(radyal)) = (nokta.x.sayı(), nokta.y.sayı()) {
                        kapsa(&mut açısal_kapsam, açısal);
                        kapsa(&mut kapsam, radyal);
                    }
                }
            }
            continue;
        }
        let çiftli = seri
            .veri()
            .iter()
            .any(|öğe| kutupsal_değerler(&öğe.değer).is_some());
        if çiftli {
            for öğe in seri.veri() {
                if let Some((radyal, açısal)) = kutupsal_değerler(&öğe.değer) {
                    kapsa(&mut kapsam, radyal);
                    kapsa(&mut açısal_kapsam, açısal);
                }
            }
            continue;
        }
        if let Some(seri_aralıkları) = aralıklar.get(i) {
            for (sıra, aralık) in seri_aralıkları.iter().enumerate() {
                if let Some(aralık) = aralık {
                    for v in [aralık.0, aralık.1] {
                        kapsa(&mut kapsam, v);
                    }
                    kapsa(&mut açısal_kapsam, sıra as f64);
                }
            }
        }
    }
    if !kapsam[0].is_finite() {
        kapsam = [0.0, 1.0];
    }

    if !açısal_kapsam[0].is_finite() {
        açısal_kapsam = [0.0, en_uzun.saturating_sub(1).max(1) as f64];
    }

    let açısal_kategorik = koordinat.açısal_eksen.tür == EksenTürü::Kategori;
    let radyal_kategorik = koordinat.radyal_eksen.tür == EksenTürü::Kategori;
    let açısal_ölçek = if açısal_kategorik {
        Ölçek::Kategorik(KategorikÖlçek::yeni(koordinat.açısal_eksen.veri.clone()))
    } else {
        Ölçek::Aralık(AralıkÖlçeği::kur(
            açısal_kapsam,
            koordinat.açısal_eksen.en_az,
            koordinat.açısal_eksen.en_çok,
            koordinat.açısal_eksen.sıfırı_içer,
            koordinat.açısal_eksen.bölme_sayısı,
            koordinat.açısal_eksen.en_küçük_adım,
            koordinat.açısal_eksen.en_büyük_adım,
        ))
    };
    let radyal_ölçek = if radyal_kategorik {
        Ölçek::Kategorik(KategorikÖlçek::yeni(koordinat.radyal_eksen.veri.clone()))
    } else {
        Ölçek::Aralık(AralıkÖlçeği::kur(
            kapsam,
            koordinat.radyal_eksen.en_az,
            koordinat.radyal_eksen.en_çok,
            koordinat.radyal_eksen.sıfırı_içer,
            koordinat.radyal_eksen.bölme_sayısı,
            koordinat.radyal_eksen.en_küçük_adım,
            koordinat.radyal_eksen.en_büyük_adım,
        ))
    };

    KutupsalDüzen {
        merkez,
        yarıçap,
        açısal_ölçek,
        radyal_ölçek,
        açısal_kategorik,
        radyal_kategorik,
        açısal_kenar_boşluğu: koordinat.açısal_eksen.kenar_boşluğu.unwrap_or(true),
        radyal_kenar_boşluğu: koordinat.radyal_eksen.kenar_boşluğu.unwrap_or(true),
        radyal_ters: koordinat.radyal_eksen.ters,
        başlangıç_açısı: koordinat.başlangıç_açısı,
        saat_yönü: koordinat.saat_yönü ^ koordinat.açısal_eksen.ters,
    }
}

/// Kutupsal ağı çizer: radyal halkalar + değer etiketleri, açısal ışınlar
/// + kategori/değer etiketleri.
pub fn kutupsal_ağ_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    koordinat: &KutupsalKoordinat,
    düzen: &KutupsalDüzen,
) {
    // RadiusAxisView ekseni ilk veri merkezine değil angleAxis extent'inin
    // başlangıç ışınına yerleştirir; onBand kategori ekseninde ayrım görünür.
    let başlangıç = -(düzen.başlangıç_açısı.to_radians());
    let radyal_yön = (başlangıç.cos(), başlangıç.sin());
    let etiket_normali = (radyal_yön.1, -radyal_yön.0);

    let radyal_bölmeler = if düzen.radyal_kategorik && düzen.radyal_kenar_boşluğu {
        let sayı = düzen.radyal_ölçek.kategori_sayısı().max(1);
        (0..=sayı)
            .map(|sıra| düzen.yarıçap * sıra as f32 / sayı as f32)
            .collect::<Vec<_>>()
    } else {
        düzen
            .radyal_ölçek
            .çentikler()
            .into_iter()
            .map(|çentik| düzen.yarıçapa(çentik.değer))
            .collect::<Vec<_>>()
    };
    let radyal_bölme_göster = koordinat
        .radyal_eksen
        .bölme_çizgisi
        .göster
        .unwrap_or(koordinat.radyal_eksen.tür != EksenTürü::Kategori);
    if radyal_bölme_göster {
        for yarıçap in radyal_bölmeler {
            if yarıçap <= 0.5 {
                continue;
            }
            let yol = crate::cizim::yuzey::daire_yolu(düzen.merkez, yarıçap);
            çizici.yol_çiz(
                &yol,
                1.0,
                koordinat
                    .radyal_eksen
                    .bölme_çizgisi
                    .renk
                    .unwrap_or_else(tema::bölme_çizgisi),
                koordinat.radyal_eksen.bölme_çizgisi.tür,
            );
        }
    }

    // Radyal eksen etiketleri, kategori ekseninde bant merkezlerine düşer.
    if koordinat.radyal_eksen.etiket.göster {
        for çentik in düzen.radyal_ölçek.çentikler() {
            let yarıçap = düzen.yarıçapa(çentik.değer);
            let eksen_noktası = (
                düzen.merkez.0 + yarıçap * radyal_yön.0,
                düzen.merkez.1 + yarıçap * radyal_yön.1,
            );
            let etiket_noktası = (
                eksen_noktası.0 + etiket_normali.0 * 8.0,
                eksen_noktası.1 + etiket_normali.1 * 8.0,
            );
            let yatay = if etiket_normali.0 > 0.3 {
                YatayHiza::Sol
            } else if etiket_normali.0 < -0.3 {
                YatayHiza::Sağ
            } else {
                YatayHiza::Orta
            };
            let dikey = if etiket_normali.1 > 0.3 {
                DikeyHiza::Üst
            } else if etiket_normali.1 < -0.3 {
                DikeyHiza::Alt
            } else {
                DikeyHiza::Orta
            };
            let metin = düzen.radyal_ölçek.etiket(çentik.değer);
            let yazı = &koordinat.radyal_eksen.etiket.yazı;
            let boyut = yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            let renk = yazı
                .renk
                .unwrap_or_else(tema::eksen_etiketi)
                .opaklık(yazı.opaklık.unwrap_or(1.0));
            let döndürme = koordinat.radyal_eksen.etiket.döndürme;
            if döndürme.abs() <= f32::EPSILON {
                çizici.yazı(
                    &metin,
                    etiket_noktası,
                    yatay,
                    dikey,
                    boyut,
                    renk,
                    yazı.kalın,
                );
            } else {
                çizici.dönüşümlü_yazı(
                    &metin,
                    (0.0, 0.0),
                    yatay,
                    dikey,
                    boyut,
                    renk,
                    yazı.kalın,
                    AfinMatris::ötele(etiket_noktası.0 + 0.3, etiket_noktası.1 + 0.3)
                        .çarp(AfinMatris::döndür(-döndürme.to_radians())),
                );
            }
        }
    }

    // Açısal ışınlar + etiketler.
    let mut çentikler = düzen.açısal_ölçek.çentikler();
    if !düzen.açısal_kategorik && çentikler.len() > 1 {
        // Tam dairede kapsamın iki ucu aynı ışına düşer; ECharts son
        // (360 gibi) etiketi yinelenen 0 etiketi yerine göstermez.
        çentikler.pop();
    }
    let açısal_bölme_göster = koordinat
        .açısal_eksen
        .bölme_çizgisi
        .göster
        .unwrap_or(koordinat.açısal_eksen.tür != EksenTürü::Kategori);
    for çentik in &çentikler {
        let açı = if düzen.açısal_kategorik {
            let n = düzen.açısal_ölçek.kategori_sayısı().max(1) as f64;
            let oran = çentik.değer / n;
            let başlangıç = -(düzen.başlangıç_açısı as f64).to_radians();
            let yön = if düzen.saat_yönü { 1.0 } else { -1.0 };
            (başlangıç + yön * oran * std::f64::consts::TAU) as f32
        } else {
            düzen.açı(çentik.değer)
        };
        let uç = (
            düzen.merkez.0 + düzen.yarıçap * açı.cos(),
            düzen.merkez.1 + düzen.yarıçap * açı.sin(),
        );
        if açısal_bölme_göster {
            çizici.çizgi(
                düzen.merkez,
                uç,
                1.0,
                koordinat
                    .açısal_eksen
                    .bölme_çizgisi
                    .renk
                    .unwrap_or_else(tema::bölme_çizgisi),
                koordinat.açısal_eksen.bölme_çizgisi.tür,
            );
        }
        if !koordinat.açısal_eksen.etiket.göster {
            continue;
        }
        // Etiket bant ortasında (kategorik) ya da ışında.
        let etiket_açısı = if düzen.açısal_kategorik {
            düzen.açı(çentik.değer)
        } else {
            açı
        };
        let konum = (
            düzen.merkez.0 + (düzen.yarıçap + 8.0) * etiket_açısı.cos(),
            düzen.merkez.1 + (düzen.yarıçap + 8.0) * etiket_açısı.sin(),
        );
        let yatay = if etiket_açısı.cos().abs() < 0.3 {
            YatayHiza::Orta
        } else if etiket_açısı.cos() > 0.0 {
            YatayHiza::Sol
        } else {
            YatayHiza::Sağ
        };
        let dikey = if etiket_açısı.sin() > 0.3 {
            DikeyHiza::Üst
        } else if etiket_açısı.sin() < -0.3 {
            DikeyHiza::Alt
        } else {
            DikeyHiza::Orta
        };
        let yazı = &koordinat.açısal_eksen.etiket.yazı;
        çizici.yazı(
            &düzen.açısal_ölçek.etiket(çentik.değer),
            (konum.0, konum.1 + 0.2),
            yatay,
            dikey,
            yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
            yazı
                .renk
                .unwrap_or_else(tema::eksen_etiketi)
                .opaklık(yazı.opaklık.unwrap_or(1.0)),
            yazı.kalın,
        );
    }

    // angleAxis.axisLine dış halkadır; radiusAxis.axisLine başlangıç açısı
    // boyunca merkezden dış halkaya uzanır.
    if koordinat.açısal_eksen.çizgi.göster.unwrap_or(true) {
        let dış = crate::cizim::yuzey::daire_yolu(düzen.merkez, düzen.yarıçap);
        çizici.yol_çiz(
            &dış,
            koordinat.açısal_eksen.çizgi.kalınlık,
            koordinat
                .açısal_eksen
                .çizgi
                .renk
                .unwrap_or_else(tema::eksen_çizgisi),
            crate::model::stil::ÇizgiTürü::Düz,
        );
    }
    if koordinat.radyal_eksen.çizgi.göster.unwrap_or(true) {
        çizici.çizgi(
            düzen.merkez,
            (
                düzen.merkez.0 + düzen.yarıçap * radyal_yön.0,
                düzen.merkez.1 + düzen.yarıçap * radyal_yön.1,
            ),
            koordinat.radyal_eksen.çizgi.kalınlık,
            koordinat
                .radyal_eksen
                .çizgi
                .renk
                .unwrap_or_else(tema::eksen_çizgisi),
            crate::model::stil::ÇizgiTürü::Düz,
        );
    }
}

/// Kutupsal serileri çizer (sütun dilimleri, çizgiler, saçılım noktaları).
#[allow(clippy::too_many_arguments)]
pub fn kutupsal_serileri_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seçenekler: &GrafikSeçenekleri,
    düzen: &KutupsalDüzen,
    aralıklar: &[Vec<YığınAralığı>],
    görünürler: &[bool],
    kapalı: &HashSet<String>,
    ilerleme: f32,
    zaman_sn: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let _ = kapalı;
    let ilerleme = ilerleme.clamp(0.0, 1.0);
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        if !seri.kutupsal_mı() || !görünürler.get(i).copied().unwrap_or(false) {
            continue;
        }
        let renk = seçenekler.seri_rengi(i);
        match seri {
            Seri::Sütun(s) => {
                let bant = düzen.bant_açısı();
                let dilim_açıklığı = bant * 0.6;
                for (j, aralık) in aralıklar
                    .get(i)
                    .map(Vec::as_slice)
                    .unwrap_or(&[])
                    .iter()
                    .enumerate()
                {
                    let Some((taban, tepe)) = aralık else {
                        continue;
                    };
                    let orta = düzen.açı(j as f64);
                    let iç = düzen.yarıçapa(*taban);
                    let dış_tam = düzen.yarıçapa(*tepe);
                    let dış = iç + (dış_tam - iç) * ilerleme;
                    let dolgu = s.öğe_stili.renk.clone().unwrap_or(Dolgu::Düz(renk));
                    çizici.dilim(
                        düzen.merkez,
                        iç.min(dış),
                        iç.max(dış),
                        orta - dilim_açıklığı / 2.0,
                        orta + dilim_açıklığı / 2.0,
                        &dolgu,
                        None,
                    );
                    isabetler.push(İsabetBölgesi {
                        seri_sırası: i,
                        veri_sırası: j,
                        seri_adı: s.ad.clone(),
                        ad: s.veri.get(j).and_then(|ö| ö.ad.clone()),
                        değer: s.veri.get(j).and_then(|ö| ö.değer.sayı()),
                        geometri: İsabetGeometrisi::Halka {
                            merkez: düzen.merkez,
                            iç_yarıçap: iç.min(dış),
                            dış_yarıçap: iç.max(dış),
                            açı0: orta - dilim_açıklığı / 2.0,
                            açı1: orta + dilim_açıklığı / 2.0,
                        },
                    });
                }
            }
            Seri::Çizgi(s) => {
                let çiftli = s
                    .veri
                    .iter()
                    .any(|öğe| kutupsal_değerler(&öğe.değer).is_some());
                let noktalar: Vec<(f32, f32)> = if çiftli {
                    s.veri
                        .iter()
                        .filter_map(|öğe| {
                            let (radyal, açısal) = kutupsal_değerler(&öğe.değer)?;
                            Some(düzen.nokta(açısal, radyal * ilerleme as f64))
                        })
                        .collect()
                } else {
                    aralıklar
                        .get(i)
                        .map(Vec::as_slice)
                        .unwrap_or(&[])
                        .iter()
                        .enumerate()
                        .filter_map(|(j, aralık)| {
                            aralık.map(|(_, tepe)| düzen.nokta(j as f64, tepe * ilerleme as f64))
                        })
                        .collect()
                };
                if noktalar.len() >= 2 {
                    let mut yol = Yol::yeni();
                    yol.taşı(noktalar.first().copied().unwrap_or(düzen.merkez));
                    for n in noktalar.iter().skip(1) {
                        yol.çiz(*n);
                    }
                    let çizgi_rengi = s.çizgi_stili.renk.unwrap_or(renk);
                    çizici.yol_çiz(&yol, s.çizgi_stili.kalınlık, çizgi_rengi, s.çizgi_stili.tür);
                }
                if s.sembol_göster && s.sembol != Sembol::Yok {
                    for n in &noktalar {
                        sembol_çiz(çizici, s.sembol, *n, s.sembol_boyutu, renk);
                    }
                }
            }
            Seri::Saçılım(s) => {
                for (j, öğe) in s.veri.iter().enumerate() {
                    let Some((radyal, açısal)) = kutupsal_değerler(&öğe.değer) else {
                        continue;
                    };
                    let nokta = düzen.nokta(açısal, radyal);
                    let boyut = s.sembol_boyutu.çöz(öğe) * ilerleme;
                    sembol_çiz(çizici, s.sembol, nokta, boyut, renk.opaklık(0.8));
                    if !s.sessiz {
                        isabetler.push(İsabetBölgesi {
                            seri_sırası: i,
                            veri_sırası: j,
                            seri_adı: s.ad.clone(),
                            ad: öğe.ad.clone(),
                            değer: Some(radyal),
                            geometri: İsabetGeometrisi::Daire {
                                merkez: nokta,
                                yarıçap: (boyut / 2.0 + 3.0).max(8.0),
                            },
                        });
                    }
                }
            }
            Seri::Hatlar(s) => {
                let çiz = |yüzey: &mut dyn ÇizimYüzeyi, isabetler: &mut Vec<İsabetBölgesi>| {
                    hatlar_çiz(
                        yüzey,
                        s,
                        i,
                        &|nokta| Some(düzen.nokta(nokta.x.sayı()?, nokta.y.sayı()?)),
                        renk,
                        ilerleme,
                        zaman_sn,
                        isabetler,
                    );
                };
                if s.kırp {
                    let yol = crate::cizim::yuzey::daire_yolu(düzen.merkez, düzen.yarıçap);
                    çizici.yol_kırpılı(&yol, &mut |kırpılı| çiz(kırpılı, isabetler));
                } else {
                    çiz(çizici, isabetler);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod testler {
    use super::*;
    use crate::model::eksen::{Eksen, EksenEtiketi, EksenÇizgisi};
    use crate::model::seri::{SaçılımSerisi, ÇizgiSerisi};

    #[test]
    fn iki_değer_ekseni_radius_angle_sırasını_ve_saat_yönünü_kullanır() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .kutupsal(KutupsalKoordinat::yeni().başlangıç_açısı(0.0))
            .seri(
                ÇizgiSerisi::yeni()
                    .kutupsal(true)
                    .veri([[5.0, 0.0], [10.0, 360.0]]),
            );
        let düzen = kutupsal_kur(
            seçenekler.kutupsal.as_ref().unwrap(),
            &seçenekler,
            &[Vec::new()],
            &[true],
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
        );

        assert_eq!(düzen.açısal_ölçek.kapsam(), [0.0, 360.0]);
        assert_eq!(düzen.radyal_ölçek.kapsam(), [0.0, 10.0]);
        let sağ = düzen.nokta(0.0, 10.0);
        let alt = düzen.nokta(90.0, 10.0);
        assert!((sağ.0 - 560.0).abs() < 0.01 && (sağ.1 - 262.5).abs() < 0.01);
        assert!((alt.0 - 350.0).abs() < 0.01 && (alt.1 - 472.5).abs() < 0.01);
    }

    #[test]
    fn açık_sıfır_alt_sınırı_negatif_yarıçapı_kırpmaz() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .kutupsal(
                KutupsalKoordinat::yeni()
                    .başlangıç_açısı(0.0)
                    .radyal_eksen(crate::model::eksen::Eksen::değer().en_az(0.0)),
            )
            .seri(
                ÇizgiSerisi::yeni()
                    .kutupsal(true)
                    .veri([[-0.5, 0.0], [0.5, 90.0]]),
            );
        let düzen = kutupsal_kur(
            seçenekler.kutupsal.as_ref().unwrap(),
            &seçenekler,
            &[Vec::new()],
            &[true],
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
        );

        assert!(düzen.yarıçapa(-0.5) < 0.0);
        assert!(düzen.yarıçapa(0.5) > 0.0);
    }

    #[test]
    fn kategorik_radius_ve_boundary_gap_kapalı_angle_polar_scatterı_yerleştirir() {
        let saatler = (0..24).map(|saat| saat.to_string()).collect::<Vec<_>>();
        let günler = (0..7).map(|gün| gün.to_string()).collect::<Vec<_>>();
        let seçenekler = GrafikSeçenekleri::yeni()
            .kutupsal(
                KutupsalKoordinat::yeni()
                    .açısal_eksen(
                        Eksen::kategori()
                            .kenar_boşluğu(false)
                            .veri(saatler)
                            .çizgi(EksenÇizgisi::yeni().göster(false)),
                    )
                    .radyal_eksen(
                        Eksen::kategori()
                            .veri(günler)
                            .etiket(EksenEtiketi::yeni().döndür(45.0)),
                    ),
            )
            .seri(
                SaçılımSerisi::yeni()
                    .kutupsal(true)
                    .veri([[0.0, 0.0, 5.0], [6.0, 12.0, 1.0]]),
            );
        let düzen = kutupsal_kur(
            seçenekler.kutupsal.as_ref().unwrap(),
            &seçenekler,
            &[Vec::new()],
            &[true],
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
        );

        assert!(düzen.açısal_kategorik);
        assert!(düzen.radyal_kategorik);
        assert!(!düzen.açısal_kenar_boşluğu);
        assert_eq!(
            kutupsal_değerler(&seçenekler.seriler[0].veri()[0].değer),
            Some((0.0, 0.0))
        );
        let cumartesi_gece = düzen.nokta(0.0, 0.0);
        let pazar_öğlen = düzen.nokta(12.0, 6.0);
        assert!((cumartesi_gece.0 - 350.0).abs() < 0.01);
        assert!((cumartesi_gece.1 - 247.5).abs() < 0.01);
        assert!((pazar_öğlen.0 - 350.0).abs() < 0.01);
        assert!((pazar_öğlen.1 - 457.5).abs() < 0.01);
    }
}
