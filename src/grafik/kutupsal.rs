//! Kutupsal koordinat sistemi — `echarts/src/coord/polar` ve kutupsal
//! seri görünümlerinin karşılığı. Açısal eksen kategori ya da değer,
//! radyal eksen değer eksenidir.

use std::collections::HashSet;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{keskin, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::sembol_çiz;
use crate::koordinat::Dikdörtgen;
use crate::model::kutupsal::KutupsalKoordinat;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::Seri;
use crate::olcek::{AralıkÖlçeği, KategorikÖlçek, Ölçek};
use crate::renk::Dolgu;
use crate::tema;
use crate::yerlesim::yigin::YığınAralığı;

/// Çözülmüş kutupsal düzen.
pub struct KutupsalDüzen {
    pub merkez: (f32, f32),
    pub yarıçap: f32,
    pub açısal_ölçek: Ölçek,
    pub radyal_ölçek: Ölçek,
    /// Açısal eksen kategorik mi (bant yerleşimi)?
    pub açısal_kategorik: bool,
}

impl KutupsalDüzen {
    /// Açısal değeri ekran radyanına çevirir (0 üstte, saat yönü).
    pub fn açı(&self, değer: f64) -> f32 {
        let oran = if self.açısal_kategorik {
            let n = self.açısal_ölçek.kategori_sayısı().max(1) as f64;
            (değer + 0.5) / n
        } else {
            self.açısal_ölçek.oranla(değer)
        };
        (-std::f64::consts::FRAC_PI_2 + oran * std::f64::consts::TAU) as f32
    }

    /// Bant açıklığı (radyan) — kutupsal sütunlar için.
    pub fn bant_açısı(&self) -> f32 {
        let n = self.açısal_ölçek.kategori_sayısı().max(1) as f32;
        std::f32::consts::TAU / n
    }

    /// Radyal değeri yarıçapa çevirir.
    pub fn yarıçapa(&self, değer: f64) -> f32 {
        (self.radyal_ölçek.oranla(değer) as f32) * self.yarıçap
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

    // Radyal kapsam: kutupsal serilerin yığınlı değerleri.
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    let mut en_uzun = 0usize;
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        if !seri.kutupsal_mı() || !görünürler.get(i).copied().unwrap_or(false) {
            continue;
        }
        en_uzun = en_uzun.max(seri.veri().len());
        if let Some(seri_aralıkları) = aralıklar.get(i) {
            for aralık in seri_aralıkları.iter().flatten() {
                for v in [aralık.0, aralık.1] {
                    if v.is_finite() {
                        kapsam[0] = kapsam[0].min(v);
                        kapsam[1] = kapsam[1].max(v);
                    }
                }
            }
        }
    }
    if !kapsam[0].is_finite() {
        kapsam = [0.0, 1.0];
    }

    let açısal_kategorik = !koordinat.açısal_eksen.veri.is_empty();
    let açısal_ölçek = if açısal_kategorik {
        Ölçek::Kategorik(KategorikÖlçek::yeni(koordinat.açısal_eksen.veri.clone()))
    } else {
        // Değer tipli açısal eksen: kapsam veri sırasından.
        Ölçek::Aralık(AralıkÖlçeği::kur(
            [0.0, en_uzun.saturating_sub(1).max(1) as f64],
            None,
            None,
            true,
            koordinat.açısal_eksen.bölme_sayısı,
            None,
            None,
        ))
    };
    let radyal_ölçek = Ölçek::Aralık(AralıkÖlçeği::kur(
        kapsam,
        koordinat.radyal_eksen.en_az,
        koordinat.radyal_eksen.en_çok,
        koordinat.radyal_eksen.sıfırı_içer,
        koordinat.radyal_eksen.bölme_sayısı,
        None,
        None,
    ));

    KutupsalDüzen { merkez, yarıçap, açısal_ölçek, radyal_ölçek, açısal_kategorik }
}

/// Kutupsal ağı çizer: radyal halkalar + değer etiketleri, açısal ışınlar
/// + kategori/değer etiketleri.
pub fn kutupsal_ağ_çiz(çizici: &mut dyn ÇizimYüzeyi, düzen: &KutupsalDüzen) {
    // Radyal halkalar.
    for çentik in düzen.radyal_ölçek.çentikler() {
        let yarıçap = düzen.yarıçapa(çentik.değer);
        if yarıçap <= 0.5 {
            continue;
        }
        let yol = crate::cizim::yuzey::daire_yolu(düzen.merkez, yarıçap);
        çizici.yol_çiz(&yol, 1.0, tema::BÖLME_ÇİZGİSİ, crate::model::stil::ÇizgiTürü::Düz);
        çizici.yazı(
            &düzen.radyal_ölçek.etiket(çentik.değer),
            (düzen.merkez.0 + 4.0, keskin(düzen.merkez.1 - yarıçap)),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            tema::YAZI_KÜÇÜK,
            tema::EKSEN_ETİKETİ,
            false,
        );
    }

    // Açısal ışınlar + etiketler.
    let çentikler = düzen.açısal_ölçek.çentikler();
    for çentik in &çentikler {
        let açı = if düzen.açısal_kategorik {
            // Işınlar bant sınırlarına düşer.
            let n = düzen.açısal_ölçek.kategori_sayısı().max(1) as f64;
            (-std::f64::consts::FRAC_PI_2
                + (çentik.değer / n) * std::f64::consts::TAU) as f32
        } else {
            düzen.açı(çentik.değer)
        };
        let uç = (
            düzen.merkez.0 + düzen.yarıçap * açı.cos(),
            düzen.merkez.1 + düzen.yarıçap * açı.sin(),
        );
        çizici.çizgi(
            düzen.merkez,
            uç,
            1.0,
            tema::BÖLME_ÇİZGİSİ,
            crate::model::stil::ÇizgiTürü::Düz,
        );
        // Etiket bant ortasında (kategorik) ya da ışında.
        let etiket_açısı = if düzen.açısal_kategorik {
            düzen.açı(çentik.değer)
        } else {
            açı
        };
        let konum = (
            düzen.merkez.0 + (düzen.yarıçap + 12.0) * etiket_açısı.cos(),
            düzen.merkez.1 + (düzen.yarıçap + 12.0) * etiket_açısı.sin(),
        );
        let yatay = if etiket_açısı.cos().abs() < 0.3 {
            YatayHiza::Orta
        } else if etiket_açısı.cos() > 0.0 {
            YatayHiza::Sol
        } else {
            YatayHiza::Sağ
        };
        çizici.yazı(
            &düzen.açısal_ölçek.etiket(çentik.değer),
            konum,
            yatay,
            DikeyHiza::Orta,
            tema::YAZI_KÜÇÜK,
            tema::EKSEN_ETİKETİ,
            false,
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
                    let Some((taban, tepe)) = aralık else { continue };
                    let orta = düzen.açı(j as f64);
                    let iç = düzen.yarıçapa(*taban);
                    let dış_tam = düzen.yarıçapa(*tepe);
                    let dış = iç + (dış_tam - iç) * ilerleme;
                    let dolgu = s
                        .öğe_stili
                        .renk
                        .clone()
                        .unwrap_or(Dolgu::Düz(renk));
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
                let noktalar: Vec<(f32, f32)> = aralıklar
                    .get(i)
                    .map(Vec::as_slice)
                    .unwrap_or(&[])
                    .iter()
                    .enumerate()
                    .filter_map(|(j, aralık)| {
                        aralık.map(|(_, tepe)| düzen.nokta(j as f64, tepe * ilerleme as f64))
                    })
                    .collect();
                if noktalar.len() >= 2 {
                    let mut yol = Yol::yeni();
                    yol.taşı(noktalar.first().copied().unwrap_or(düzen.merkez));
                    for n in noktalar.iter().skip(1) {
                        yol.çiz(*n);
                    }
                    let çizgi_rengi = s.çizgi_stili.renk.unwrap_or(renk);
                    çizici.yol_çiz(&yol, s.çizgi_stili.kalınlık, çizgi_rengi, s.çizgi_stili.tür);
                }
                if s.sembol_göster {
                    for n in &noktalar {
                        sembol_çiz(çizici, s.sembol, *n, s.sembol_boyutu, renk);
                    }
                }
            }
            Seri::Saçılım(s) => {
                for (j, öğe) in s.veri.iter().enumerate() {
                    let Some(değer) = öğe.değer.sayı() else { continue };
                    let açısal = öğe.değer.x().unwrap_or(j as f64);
                    let nokta = düzen.nokta(açısal, değer);
                    let boyut = s.sembol_boyutu.çöz(öğe) * ilerleme;
                    sembol_çiz(çizici, s.sembol, nokta, boyut, renk.opaklık(0.8));
                    isabetler.push(İsabetBölgesi {
                        seri_sırası: i,
                        veri_sırası: j,
                        seri_adı: s.ad.clone(),
                        ad: öğe.ad.clone(),
                        değer: Some(değer),
                        geometri: İsabetGeometrisi::Daire {
                            merkez: nokta,
                            yarıçap: (boyut / 2.0 + 3.0).max(8.0),
                        },
                    });
                }
            }
            _ => {}
        }
    }
}
