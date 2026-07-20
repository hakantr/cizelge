//! Kutupsal koordinat sistemi — `echarts/src/coord/polar` ve kutupsal
//! seri görünümlerinin karşılığı. Açısal ve radyal eksenler kategori ya da
//! sayısal ölçek taşıyabilir.

use std::collections::HashSet;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::hatlar::hatlar_çiz;
use crate::grafik::sembol_çiz;
use crate::koordinat::Dikdörtgen;
use crate::model::Uzunluk;
use crate::model::deger::VeriDeğeri;
use crate::model::eksen::EksenTürü;
use crate::model::kutupsal::KutupsalKoordinat;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{Sembol, Seri};
use crate::model::stil::{
    Etiket, EtiketDöndürme, EtiketKonumu, YazıDikeyHizası, YazıYatayHizası
};
use crate::olcek::{AralıkÖlçeği, KategorikÖlçek, Ölçek};
use crate::renk::Dolgu;
use crate::tema;
use crate::yardimci::bicim::ondalık_kırp;
use crate::yardimci::sayi::doğrusal_eşle;
use crate::yerlesim::yigin::YığınAralığı;

/// Çözülmüş kutupsal düzen.
pub struct KutupsalDüzen {
    pub merkez: (f32, f32),
    pub iç_yarıçap: f32,
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

fn kutupsal_sütun_uzunluğunu_çöz(uzunluk: Uzunluk, bant_açısı: f32) -> f32 {
    match uzunluk {
        Uzunluk::Yüzde(yüzde) => bant_açısı * yüzde / 100.0,
        // Polar angleAxis üzerindeki sayısal barWidth/barMaxWidth değerleri
        // açı koordinatı birimindedir; ECharts yerleşimi bunları derece
        // olarak sektöre çevirir.
        Uzunluk::Piksel(derece) => derece.to_radians(),
    }
}

fn kutupsal_sütun_açıklığı(seri: &crate::model::seri::SütunSerisi, bant: f32) -> f32 {
    let mut açıklık = seri.genişlik.map_or_else(
        || {
            let kategori_boşluğu = seri
                .kategori_boşluğu
                .map(|boşluk| kutupsal_sütun_uzunluğunu_çöz(boşluk, bant))
                .unwrap_or(bant * 0.2);
            (bant - kategori_boşluğu).max(0.0)
        },
        |genişlik| kutupsal_sütun_uzunluğunu_çöz(genişlik, bant),
    );
    if let Some(en_çok) = seri.en_çok_genişlik {
        açıklık = açıklık.min(kutupsal_sütun_uzunluğunu_çöz(en_çok, bant));
    }
    if let Some(en_az) = seri.en_az_genişlik {
        açıklık = açıklık.max(kutupsal_sütun_uzunluğunu_çöz(en_az, bant));
    }
    açıklık.clamp(0.0, bant)
}

fn yazı_yatay_hizası(hiza: YazıYatayHizası) -> YatayHiza {
    match hiza {
        YazıYatayHizası::Sol => YatayHiza::Sol,
        YazıYatayHizası::Orta => YatayHiza::Orta,
        YazıYatayHizası::Sağ => YatayHiza::Sağ,
    }
}

fn yazı_dikey_hizası(hiza: YazıDikeyHizası) -> DikeyHiza {
    match hiza {
        YazıDikeyHizası::Üst => DikeyHiza::Üst,
        YazıDikeyHizası::Orta => DikeyHiza::Orta,
        YazıDikeyHizası::Alt => DikeyHiza::Alt,
    }
}

#[allow(clippy::too_many_arguments)]
fn kutupsal_sütun_etiketi_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    etiket: &Etiket,
    seri_adı: &str,
    veri_adı: &str,
    değer: f64,
    merkez: (f32, f32),
    başlangıç_yarıçapı: f32,
    bitiş_yarıçapı: f32,
    orta_açı: f32,
    dolgu: &Dolgu,
) {
    if !etiket.göster {
        return;
    }

    let pozitif = bitiş_yarıçapı >= başlangıç_yarıçapı;
    let konum = match etiket.konum {
        EtiketKonumu::Dış => {
            if pozitif {
                EtiketKonumu::Bitiş
            } else {
                EtiketKonumu::Başlangıç
            }
        }
        diğer => diğer,
    };
    let uzaklık = etiket.uzaklık;
    let (yarıçap, doğal_yatay, doğal_dikey, içeride) = match konum {
        EtiketKonumu::Başlangıç => (
            başlangıç_yarıçapı - if pozitif { uzaklık } else { -uzaklık },
            YatayHiza::Orta,
            DikeyHiza::Üst,
            false,
        ),
        EtiketKonumu::İçBaşlangıç => (
            başlangıç_yarıçapı + if pozitif { uzaklık } else { -uzaklık },
            YatayHiza::Orta,
            DikeyHiza::Alt,
            true,
        ),
        EtiketKonumu::Bitiş => (
            bitiş_yarıçapı + if pozitif { uzaklık } else { -uzaklık },
            YatayHiza::Orta,
            DikeyHiza::Alt,
            false,
        ),
        EtiketKonumu::İçBitiş => (
            bitiş_yarıçapı - if pozitif { uzaklık } else { -uzaklık },
            YatayHiza::Orta,
            DikeyHiza::Üst,
            true,
        ),
        // `middle`, zrender'ın bağlı `inside` varsayılanı ve Kartezyen'e
        // özgü konumların kutupsal güvenli geri düşüşü sektör merkezidir.
        _ => (
            (başlangıç_yarıçapı + bitiş_yarıçapı) / 2.0,
            YatayHiza::Orta,
            DikeyHiza::Orta,
            true,
        ),
    };
    let nokta = (
        merkez.0 + yarıçap * orta_açı.cos() + etiket.kayma.0,
        merkez.1 + yarıçap * orta_açı.sin() + etiket.kayma.1,
    );
    let yatay = etiket
        .yatay_hiza
        .map(yazı_yatay_hizası)
        .unwrap_or(doğal_yatay);
    let dikey = etiket
        .dikey_hiza
        .map(yazı_dikey_hizası)
        .unwrap_or(doğal_dikey);
    let ham = ondalık_kırp(değer);
    let metin = etiket
        .biçimleyici
        .as_ref()
        .map(|biçimleyici| biçimleyici.uygula_bağlamla(değer, &ham, seri_adı, veri_adı))
        .unwrap_or(ham);
    let renk = etiket.yazı.renk.unwrap_or_else(|| {
        if içeride {
            dolgu.zrender_iç_etiket_stili(tema::koyu_mu()).0
        } else {
            tema::birincil_metin()
        }
    });
    let dönüş = match etiket.döndürme {
        // `series.bar.label.rotate` verilmediğinde zrender sektör metnini
        // otomatik olarak yayı izleyen teğetsel doğrultuya döndürür ve
        // `middle` konumunu okunur tarafta yarım tur çevirir.
        EtiketDöndürme::Yok | EtiketDöndürme::Teğetsel => {
            let mut açı = (std::f32::consts::PI * 1.5 - orta_açı).rem_euclid(std::f32::consts::TAU);
            if konum == EtiketKonumu::Merkez
                && açı > std::f32::consts::FRAC_PI_2
                && açı < std::f32::consts::PI * 1.5
            {
                açı -= std::f32::consts::PI;
            }
            -açı
        }
        EtiketDöndürme::Derece(derece) => -derece.to_radians(),
        EtiketDöndürme::Radyal => orta_açı,
        EtiketDöndürme::TeğetselÇevirmesiz => {
            -(std::f32::consts::PI * 1.5 - orta_açı).rem_euclid(std::f32::consts::TAU)
        }
    };
    çizici.dönüşümlü_yazı(
        &metin,
        (0.0, 0.0),
        yatay,
        dikey,
        etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
        renk.opaklık(etiket.yazı.opaklık.unwrap_or(1.0)),
        etiket.yazı.kalın,
        AfinMatris::ötele(nokta.0, nokta.1).çarp(AfinMatris::döndür(dönüş)),
    );
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
        self.iç_yarıçap + (oran as f32) * (self.yarıçap - self.iç_yarıçap)
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
    let yarıçap = koordinat.yarıçap.çöz(taban).max(0.0);
    let iç_yarıçap = koordinat.iç_yarıçap.çöz(taban).max(0.0).min(yarıçap);

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
        iç_yarıçap,
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

/// Kutupsal ağı iki ECharts z-katmanında çizer. Bölme çizgileri seri
/// sektörlerinin altında; eksen çizgisi, çentik ve etiketler üstündedir.
pub fn kutupsal_ağ_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    koordinat: &KutupsalKoordinat,
    düzen: &KutupsalDüzen,
    ön_plan: bool,
) {
    // RadiusAxisView ekseni ilk veri merkezine değil angleAxis extent'inin
    // başlangıç ışınına yerleştirir; onBand kategori ekseninde ayrım görünür.
    let başlangıç = -(düzen.başlangıç_açısı.to_radians());
    let radyal_yön = (başlangıç.cos(), başlangıç.sin());
    let etiket_normali = (radyal_yön.1, -radyal_yön.0);

    let radyal_bölmeler = if düzen.radyal_kategorik && düzen.radyal_kenar_boşluğu {
        let sayı = düzen.radyal_ölçek.kategori_sayısı().max(1);
        let açıklık = düzen.yarıçap - düzen.iç_yarıçap;
        (0..=sayı)
            .map(|sıra| düzen.iç_yarıçap + açıklık * sıra as f32 / sayı as f32)
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
    if !ön_plan && radyal_bölme_göster {
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
    if ön_plan
        && koordinat.radyal_eksen.göster
        && koordinat.radyal_eksen.çentik.göster.unwrap_or(true)
    {
        let renk = koordinat
            .radyal_eksen
            .çentik
            .renk
            .unwrap_or_else(tema::eksen_çentiği);
        let uzunluk = koordinat.radyal_eksen.çentik.uzunluk;
        for çentik in düzen.radyal_ölçek.çentikler() {
            let yarıçap = düzen.yarıçapa(çentik.değer);
            let nokta = (
                düzen.merkez.0 + yarıçap * radyal_yön.0,
                düzen.merkez.1 + yarıçap * radyal_yön.1,
            );
            çizici.çizgi(
                nokta,
                (
                    nokta.0 + etiket_normali.0 * uzunluk,
                    nokta.1 + etiket_normali.1 * uzunluk,
                ),
                1.0,
                renk,
                crate::model::stil::ÇizgiTürü::Düz,
            );
        }
    }

    if ön_plan && koordinat.radyal_eksen.göster && koordinat.radyal_eksen.etiket.göster {
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
        let başlangıç_noktası = (
            düzen.merkez.0 + düzen.iç_yarıçap * açı.cos(),
            düzen.merkez.1 + düzen.iç_yarıçap * açı.sin(),
        );
        let uç = (
            düzen.merkez.0 + düzen.yarıçap * açı.cos(),
            düzen.merkez.1 + düzen.yarıçap * açı.sin(),
        );
        if !ön_plan && açısal_bölme_göster {
            çizici.çizgi(
                başlangıç_noktası,
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
        if ön_plan
            && koordinat.açısal_eksen.göster
            && koordinat.açısal_eksen.çentik.göster.unwrap_or(true)
        {
            let uzunluk = koordinat.açısal_eksen.çentik.uzunluk;
            çizici.çizgi(
                uç,
                (uç.0 + uzunluk * açı.cos(), uç.1 + uzunluk * açı.sin()),
                1.0,
                koordinat
                    .açısal_eksen
                    .çentik
                    .renk
                    .unwrap_or_else(tema::eksen_çentiği),
                crate::model::stil::ÇizgiTürü::Düz,
            );
        }
        if !ön_plan || !koordinat.açısal_eksen.göster || !koordinat.açısal_eksen.etiket.göster
        {
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

    if !ön_plan {
        return;
    }

    // angleAxis.axisLine dış halkadır; radiusAxis.axisLine başlangıç açısı
    // boyunca iç yarıçaptan dış halkaya uzanır.
    if koordinat.açısal_eksen.göster && koordinat.açısal_eksen.çizgi.göster.unwrap_or(true) {
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
    if koordinat.radyal_eksen.göster && koordinat.radyal_eksen.çizgi.göster.unwrap_or(true) {
        çizici.çizgi(
            (
                düzen.merkez.0 + düzen.iç_yarıçap * radyal_yön.0,
                düzen.merkez.1 + düzen.iç_yarıçap * radyal_yön.1,
            ),
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
                let dilim_açıklığı = kutupsal_sütun_açıklığı(s, bant);
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
                    let veri_öğesi = s.veri.get(j);
                    let öğe_stili = veri_öğesi.and_then(|öğe| öğe.stil.as_ref());
                    let dolgu = öğe_stili
                        .and_then(|stil| stil.renk.clone())
                        .or_else(|| s.öğe_stili.renk.clone())
                        .unwrap_or(Dolgu::Düz(renk));
                    let opaklık = öğe_stili
                        .and_then(|stil| stil.opaklık)
                        .or(s.öğe_stili.opaklık)
                        .unwrap_or(1.0);
                    let kenarlık_rengi = öğe_stili
                        .and_then(|stil| stil.kenarlık_rengi)
                        .or(s.öğe_stili.kenarlık_rengi);
                    let kenarlık_kalınlığı = öğe_stili
                        .filter(|stil| stil.kenarlık_kalınlığı > 0.0)
                        .map(|stil| stil.kenarlık_kalınlığı)
                        .unwrap_or(s.öğe_stili.kenarlık_kalınlığı);
                    let kenarlık = kenarlık_rengi
                        .filter(|_| kenarlık_kalınlığı > 0.0)
                        .map(|renk| (kenarlık_kalınlığı, renk));
                    let dolgu = dolgu.opaklık(opaklık);
                    çizici.dilim(
                        düzen.merkez,
                        iç.min(dış),
                        iç.max(dış),
                        orta - dilim_açıklığı / 2.0,
                        orta + dilim_açıklığı / 2.0,
                        &dolgu,
                        kenarlık,
                    );
                    let öğe_etiketi = veri_öğesi
                        .and_then(|öğe| öğe.etiket.as_ref())
                        .map(|yama| yama.uygula(&s.etiket));
                    let etiket = öğe_etiketi.as_ref().unwrap_or(&s.etiket);
                    let veri_adı = veri_öğesi
                        .and_then(|öğe| öğe.ad.as_deref())
                        .map(str::to_owned)
                        .unwrap_or_else(|| düzen.açısal_ölçek.etiket(j as f64));
                    kutupsal_sütun_etiketi_çiz(
                        çizici,
                        etiket,
                        s.ad.as_deref().unwrap_or(""),
                        &veri_adı,
                        veri_öğesi
                            .and_then(|öğe| öğe.değer.sayı())
                            .unwrap_or(*tepe - *taban),
                        düzen.merkez,
                        iç,
                        dış,
                        orta,
                        &dolgu,
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
                        sembol_çiz(çizici, &s.sembol, *n, s.sembol_boyutu, renk);
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
                    sembol_çiz(çizici, &s.sembol, nokta, boyut, renk.opaklık(0.8));
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
    use crate::model::seri::{SaçılımSerisi, SütunSerisi, ÇizgiSerisi};
    use crate::model::stil::{Etiket, EtiketKonumu};

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
    fn polar_yarıçap_aralığı_ve_radyal_sütun_öntanımlısı_echartsı_izler() {
        let seri = SütunSerisi::yeni()
            .kutupsal(true)
            .etiket(Etiket::yeni().göster(true).konum(EtiketKonumu::Merkez))
            .veri([2.0, 1.2, 2.4, 3.6]);
        let seçenekler = GrafikSeçenekleri::yeni()
            .kutupsal(
                KutupsalKoordinat::yeni()
                    .yarıçap_aralığı(30, "80%")
                    .başlangıç_açısı(75.0)
                    .açısal_eksen(Eksen::kategori().veri(["a", "b", "c", "d"]))
                    .radyal_eksen(Eksen::değer().en_çok(4.0)),
            )
            .seri(seri.clone());
        let düzen = kutupsal_kur(
            seçenekler.kutupsal.as_ref().unwrap(),
            &seçenekler,
            &[vec![Some((0.0, 2.0)); 4]],
            &[true],
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
        );

        assert!((düzen.iç_yarıçap - 30.0).abs() < 0.01);
        assert!((düzen.yarıçap - 210.0).abs() < 0.01);
        assert!((düzen.yarıçapa(0.0) - 30.0).abs() < 0.01);
        assert!((düzen.yarıçapa(4.0) - 210.0).abs() < 0.01);
        assert!((düzen.açı(0.0).to_degrees() + 30.0).abs() < 0.01);
        assert!(
            (kutupsal_sütun_açıklığı(&seri, düzen.bant_açısı()).to_degrees() - 72.0).abs() < 0.01
        );
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
