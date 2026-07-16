//! Pasta serisi çizimi — `echarts/src/chart/pie` (yerleşim + görünüm)
//! karşılığı.

use std::collections::HashSet;

use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{GülTürü, PastaSerisi};
use crate::model::stil::EtiketKonumu;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

/// Yerleşimi hesaplanmış bir pasta dilimi.
#[derive(Clone, Debug)]
pub struct Dilim {
    /// Veri dizisindeki sıra (renk ataması için özgün sıra).
    pub sıra: usize,
    pub ad: String,
    pub değer: f64,
    /// Görünür toplam içindeki pay `0..=1`.
    pub oran: f64,
    /// Ekran koordinatında başlangıç/bitiş açısı (radyan).
    pub açı0: f32,
    pub açı1: f32,
    pub iç_yarıçap: f32,
    pub dış_yarıçap: f32,
    pub renk: Renk,
    pub merkez: (f32, f32),
}

impl Dilim {
    /// Nokta dilimin içinde mi (ipucu isabeti için)?
    pub fn içeriyor_mu(&self, nokta: (f32, f32)) -> bool {
        let dx = nokta.0 - self.merkez.0;
        let dy = nokta.1 - self.merkez.1;
        let uzaklık = (dx * dx + dy * dy).sqrt();
        if uzaklık < self.iç_yarıçap || uzaklık > self.dış_yarıçap {
            return false;
        }
        let açı = dy.atan2(dx);
        let tau = std::f32::consts::TAU;
        let (a0, a1) = if self.açı1 >= self.açı0 {
            (self.açı0, self.açı1)
        } else {
            (self.açı1, self.açı0)
        };
        let göreli = (açı - a0).rem_euclid(tau);
        göreli <= (a1 - a0)
    }
}

/// Pasta yerleşimi — `pieLayout.ts` karşılığı. `kapalı` gösterge ile
/// gizlenen dilim adlarıdır; `ilerleme` giriş animasyonunun açı çarpanıdır.
pub fn pasta_yerleşimi(
    seri: &PastaSerisi,
    seçenekler: &GrafikSeçenekleri,
    alan: Dikdörtgen,
    kapalı: &HashSet<String>,
    ilerleme: f32,
) -> Vec<Dilim> {
    let merkez = (
        alan.x + seri.merkez.0.çöz(alan.genişlik),
        alan.y + seri.merkez.1.çöz(alan.yükseklik),
    );
    // ECharts: yüzde yarıçaplar görünür alanın kısa kenarının yarısına
    // oranlıdır.
    let taban_yarıçap = alan.genişlik.min(alan.yükseklik) / 2.0;
    let iç = seri.yarıçap.0.çöz(taban_yarıçap);
    let dış = seri.yarıçap.1.çöz(taban_yarıçap);

    let görünürler: Vec<(usize, &crate::model::deger::VeriÖğesi)> = seri
        .veri
        .iter()
        .enumerate()
        .filter(|(_, ö)| {
            let ad = ö.ad.clone().unwrap_or_default();
            !kapalı.contains(&ad) && ö.değer.sayı().map(|d| d > 0.0).unwrap_or(false)
        })
        .collect();

    let toplam: f64 = görünürler
        .iter()
        .filter_map(|(_, ö)| ö.değer.sayı())
        .sum();
    if toplam <= 0.0 {
        return Vec::new();
    }
    let en_büyük: f64 = görünürler
        .iter()
        .filter_map(|(_, ö)| ö.değer.sayı())
        .fold(0.0, f64::max);

    let tau = std::f64::consts::TAU;
    let yön: f64 = if seri.saat_yönünde { 1.0 } else { -1.0 };
    // ECharts `startAngle: 90` üstten başlar; ekran koordinatında -90°'dir.
    let mut açı = -(seri.başlangıç_açısı as f64).to_radians();
    let ilerleme = ilerleme.clamp(0.0, 1.0) as f64;

    let mut dilimler = Vec::with_capacity(görünürler.len());
    for (sıra, öğe) in görünürler {
        let değer = öğe.değer.sayı().unwrap_or(0.0);
        let oran = değer / toplam;
        let pay = oran * tau * ilerleme * yön;

        let dış_dilim = match seri.gül_türü {
            None => dış,
            Some(GülTürü::Yarıçap) => {
                iç + (dış - iç) * (değer / en_büyük.max(1e-12)) as f32
            }
            Some(GülTürü::Alan) => {
                iç + (dış - iç) * ((değer / en_büyük.max(1e-12)) as f32).sqrt()
            }
        };

        let renk = öğe
            .stil
            .as_ref()
            .and_then(|s| s.renk.as_ref())
            .map(|d| d.temsilî())
            .unwrap_or_else(|| seçenekler.palet_rengi(sıra));

        dilimler.push(Dilim {
            sıra,
            ad: öğe.ad.clone().unwrap_or_else(|| format!("{sıra}")),
            değer,
            oran,
            açı0: açı as f32,
            açı1: (açı + pay) as f32,
            iç_yarıçap: iç,
            dış_yarıçap: dış_dilim,
            renk,
            merkez,
        });
        açı += pay;
    }
    dilimler
}

/// Pasta serisini çizer; `vurgulu` ipucuyla öne çıkarılan dilimin sırasıdır.
pub fn pasta_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &PastaSerisi,
    dilimler: &[Dilim],
    vurgulu: Option<usize>,
) {
    // 1) Dilimler.
    for (i, dilim) in dilimler.iter().enumerate() {
        // ECharts `emphasis.scaleSize` davranışı: vurgulu dilim büyür.
        let dış = if vurgulu == Some(i) {
            dilim.dış_yarıçap + 6.0
        } else {
            dilim.dış_yarıçap
        };
        let kenarlık = seri
            .öğe_stili
            .kenarlık_rengi
            .map(|r| (seri.öğe_stili.kenarlık_kalınlığı.max(1.0), r));
        let opaklık = seri.öğe_stili.opaklık.unwrap_or(1.0);
        çizici.dilim(
            dilim.merkez,
            dilim.iç_yarıçap,
            dış,
            dilim.açı0,
            dilim.açı1,
            &Dolgu::Düz(dilim.renk.opaklık(opaklık)),
            kenarlık,
        );
    }

    // 2) Etiketler ve etiket çizgileri.
    if !seri.etiket.göster {
        return;
    }
    let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    for dilim in dilimler {
        let orta_açı = (dilim.açı0 + dilim.açı1) / 2.0;
        let metin = match &seri.etiket.biçimleyici {
            Some(b) => b.uygula(dilim.değer, &dilim.ad),
            None => dilim.ad.clone(),
        };
        match seri.etiket.konum {
            EtiketKonumu::Merkez => {
                let renk = seri.etiket.yazı.renk.unwrap_or(tema::BİRİNCİL_METİN);
                çizici.yazı(
                    &metin,
                    dilim.merkez,
                    YatayHiza::Orta,
                    DikeyHiza::Orta,
                    boyut,
                    renk,
                    seri.etiket.yazı.kalın,
                );
            }
            EtiketKonumu::İç => {
                let yarıçap = (dilim.iç_yarıçap + dilim.dış_yarıçap) / 2.0;
                let konum = (
                    dilim.merkez.0 + yarıçap * orta_açı.cos(),
                    dilim.merkez.1 + yarıçap * orta_açı.sin(),
                );
                let renk = seri.etiket.yazı.renk.unwrap_or(Renk::BEYAZ);
                çizici.yazı(
                    &metin,
                    konum,
                    YatayHiza::Orta,
                    DikeyHiza::Orta,
                    boyut,
                    renk,
                    seri.etiket.yazı.kalın,
                );
            }
            _ => {
                // Dış etiket: dilimden çıkan kırık çizgi + metin.
                let sağda = orta_açı.cos() >= 0.0;
                let u1 = seri.etiket_çizgisi.uzunluk1;
                let u2 = seri.etiket_çizgisi.uzunluk2;
                let b0 = (
                    dilim.merkez.0 + dilim.dış_yarıçap * orta_açı.cos(),
                    dilim.merkez.1 + dilim.dış_yarıçap * orta_açı.sin(),
                );
                let b1 = (
                    dilim.merkez.0 + (dilim.dış_yarıçap + u1) * orta_açı.cos(),
                    dilim.merkez.1 + (dilim.dış_yarıçap + u1) * orta_açı.sin(),
                );
                let b2 = (b1.0 + if sağda { u2 } else { -u2 }, b1.1);
                if seri.etiket_çizgisi.göster {
                    let mut yol = Yol::yeni();
                    yol.taşı(b0);
                    yol.çiz(b1);
                    yol.çiz(b2);
                    çizici.yol_çiz(&yol, 1.0, dilim.renk, crate::model::stil::ÇizgiTürü::Düz);
                }
                let renk = seri.etiket.yazı.renk.unwrap_or(tema::BİRİNCİL_METİN);
                çizici.yazı(
                    &metin,
                    (b2.0 + if sağda { 4.0 } else { -4.0 }, b2.1),
                    if sağda { YatayHiza::Sol } else { YatayHiza::Sağ },
                    DikeyHiza::Orta,
                    boyut,
                    renk,
                    seri.etiket.yazı.kalın,
                );
            }
        }
    }
}

/// İpucu satır metni: `değer (%oran)`.
pub fn dilim_değer_metni(dilim: &Dilim) -> String {
    format!(
        "{} (%{:.1})",
        binlik_ayır(dilim.değer),
        dilim.oran * 100.0
    )
}
