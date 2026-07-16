//! Sütun serisi çizimi — `echarts/src/chart/bar/BarView.ts` karşılığı.
//! Genişlik/kaydırma hesabı [`crate::yerlesim::sutun`] portunu kullanır.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::seri::SütunSerisi;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yerlesim::sutun::{sütun_yerleşimi, SütunKonumu, SütunSerisiBilgisi};
use crate::yerlesim::yigin::YığınAralığı;

/// Çizime giren bir sütun serisi.
pub struct SütunGirdisi<'s> {
    pub seri: &'s SütunSerisi,
    /// Seçeneklerdeki genel seri sırası (renk çözümü için).
    pub genel_sıra: usize,
    pub aralıklar: &'s [YığınAralığı],
    pub renk: Renk,
}

fn yığın_kimliği(seri: &SütunSerisi, genel_sıra: usize) -> String {
    match &seri.yığın {
        Some(ad) => format!("__yığın_{ad}"),
        None => format!("__seri_{genel_sıra}"),
    }
}

/// Görünür sütun serilerinin bant içi yerleşimini hesaplar.
pub fn yerleşim_hesapla(
    girdiler: &[SütunGirdisi],
    bant_genişliği: f32,
) -> Vec<SütunKonumu> {
    let bilgiler: Vec<SütunSerisiBilgisi> = girdiler
        .iter()
        .map(|g| SütunSerisiBilgisi {
            yığın_kimliği: yığın_kimliği(g.seri, g.genel_sıra),
            genişlik: g.seri.genişlik,
            en_çok_genişlik: g.seri.en_çok_genişlik,
            en_az_genişlik: g.seri.en_az_genişlik,
            sütun_boşluğu: g.seri.sütun_boşluğu,
            kategori_boşluğu: g.seri.kategori_boşluğu,
        })
        .collect();
    let düzen = sütun_yerleşimi(bant_genişliği, &bilgiler);
    girdiler
        .iter()
        .map(|g| {
            düzen
                .get(&yığın_kimliği(g.seri, g.genel_sıra))
                .copied()
                .unwrap_or(SütunKonumu { kaydırma: 0.0, genişlik: 0.0 })
        })
        .collect()
}

/// Tüm görünür sütun serilerini çizer. Kategori ekseni y ise sütunlar yatay
/// çizilir. Çizilen her sütun için `isabetler`e tıklama bölgesi eklenir.
pub fn sütunları_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    girdiler: &[SütunGirdisi],
    kartezyen: &Kartezyen2B,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    if girdiler.is_empty() {
        return;
    }
    let yatay = kartezyen.y.ölçek.kategorik_mi() && !kartezyen.x.ölçek.kategorik_mi();
    let bant_ekseni = if yatay { &kartezyen.y } else { &kartezyen.x };
    let değer_ekseni = if yatay { &kartezyen.x } else { &kartezyen.y };
    let konumlar = yerleşim_hesapla(girdiler, bant_ekseni.bant_genişliği());

    for (girdi, konum) in girdiler.iter().zip(&konumlar) {
        let seri = girdi.seri;
        for (i, aralık) in girdi.aralıklar.iter().enumerate() {
            let Some((taban, tepe)) = aralık else { continue };
            let Some(veri_öğesi) = seri.veri.get(i) else { continue };
            let bant_merkezi = bant_ekseni.veriden_piksele(i as f64);
            let kenar = bant_merkezi + konum.kaydırma;

            let taban_p = değer_ekseni.veriden_piksele(*taban);
            let tepe_p = değer_ekseni.veriden_piksele(*tepe);
            // Giriş animasyonu: sütun tabandan büyür.
            let tepe_p = taban_p + (tepe_p - taban_p) * ilerleme.clamp(0.0, 1.0);

            let d = if yatay {
                Dikdörtgen::yeni(
                    taban_p.min(tepe_p),
                    kenar,
                    (tepe_p - taban_p).abs(),
                    konum.genişlik,
                )
            } else {
                Dikdörtgen::yeni(
                    kenar,
                    taban_p.min(tepe_p),
                    konum.genişlik,
                    (tepe_p - taban_p).abs(),
                )
            };

            let öğe_stili = veri_öğesi.stil.as_ref();
            let dolgu = öğe_stili
                .and_then(|s| s.renk.clone())
                .or_else(|| seri.öğe_stili.renk.clone())
                .unwrap_or(Dolgu::Düz(girdi.renk));
            let yarıçap = öğe_stili
                .map(|s| s.kenarlık_yarıçapı)
                .unwrap_or(seri.öğe_stili.kenarlık_yarıçapı);
            let kenarlık = seri
                .öğe_stili
                .kenarlık_rengi
                .map(|r| (seri.öğe_stili.kenarlık_kalınlığı.max(1.0), r));
            let opaklık = seri.öğe_stili.opaklık.unwrap_or(1.0);

            çizici.dikdörtgen(d, &dolgu.opaklık(opaklık), yarıçap, kenarlık);

            isabetler.push(İsabetBölgesi {
                seri_sırası: girdi.genel_sıra,
                veri_sırası: i,
                seri_adı: seri.ad.clone(),
                ad: veri_öğesi.ad.clone(),
                değer: veri_öğesi.değer.sayı(),
                geometri: İsabetGeometrisi::Dikdörtgen(d),
            });

            // Değer etiketi.
            if seri.etiket.göster {
                if let Some(değer) = veri_öğesi.değer.sayı() {
                    let metin = match &seri.etiket.biçimleyici {
                        Some(b) => b.uygula(değer, &binlik_ayır(değer)),
                        None => binlik_ayır(değer),
                    };
                    let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                    let renk = seri.etiket.yazı.renk.unwrap_or(tema::BİRİNCİL_METİN);
                    if yatay {
                        let x = if değer >= 0.0 { d.sağ() + 4.0 } else { d.x - 4.0 };
                        let hiza = if değer >= 0.0 { YatayHiza::Sol } else { YatayHiza::Sağ };
                        çizici.yazı(
                            &metin,
                            (x, d.y + d.yükseklik / 2.0),
                            hiza,
                            DikeyHiza::Orta,
                            boyut,
                            renk,
                            false,
                        );
                    } else {
                        let (y, hiza) = if değer >= 0.0 {
                            (d.y - 4.0, DikeyHiza::Alt)
                        } else {
                            (d.alt() + 4.0, DikeyHiza::Üst)
                        };
                        çizici.yazı(
                            &metin,
                            (d.x + d.genişlik / 2.0, y),
                            YatayHiza::Orta,
                            hiza,
                            boyut,
                            renk,
                            false,
                        );
                    }
                }
            }
        }
    }
}
