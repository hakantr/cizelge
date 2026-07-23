//! `series-custom` öğe görünümü.

use std::collections::BTreeSet;
use std::sync::Mutex;

use crate::bilesen::grafik::grafik_sahnesi_hazırla;
use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{
    GörselDurum, KırpmaYolu, SahneFarkı, SahneStili, SahneŞekli, ÇizimYüzeyi
};
use crate::koordinat::{Dikdörtgen, ÇalışmaEkseni};
use crate::model::deger::{VeriDeğeri, VeriÖğesi};
use crate::model::grafik_bileseni::{GrafikBileşeni, GrafikÖğesi, GrafikÖğeİçeriği};
use crate::model::ozel::{
    ÖzelKoordinatHaritası, ÖzelKoordinatTanımı, ÖzelSütunYerleşimi, ÖzelTurBağlamı, ÖzelÖğeBağlamı,
};
use crate::model::seri::ÖzelSeri;
use crate::model::yakinlastirma::YakınlaştırmaSüzmeKipi;
use crate::renk::{Dolgu, Renk};

/// Bir `renderItem` turunun çizim katmanına döndürdüğü kanıt/etkileşim özeti.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ÖzelÇizimSonucu {
    pub vurgulu_veri: Option<usize>,
    pub anahtarlar: Vec<String>,
    pub kutular: Vec<Option<Dikdörtgen>>,
}

/// Custom veri diff'inin ECharts `DataDiffer` karşılığı. Anahtarlar açık
/// `id`, veri adı ya da son çare ham sıra ile üretilir.
pub fn özel_veri_farkı(eski: &[String], yeni: &[String]) -> SahneFarkı {
    let eski = eski.iter().cloned().collect::<BTreeSet<_>>();
    let yeni = yeni.iter().cloned().collect::<BTreeSet<_>>();
    SahneFarkı {
        giren: yeni.difference(&eski).cloned().collect(),
        güncellenen: yeni.intersection(&eski).cloned().collect(),
        çıkan: eski.difference(&yeni).cloned().collect(),
    }
}

/// Koordinat görünümünün çözdüğü bütün bağımlılıklar. Geo türü bilerek yoktur.
pub(crate) struct ÖzelÇizimOrtamı<'a> {
    pub koordinat_tanımı: ÖzelKoordinatTanımı,
    pub harita: ÖzelKoordinatHaritası<'a>,
    pub görünüm: (f32, f32),
    pub güncel_seri_sıraları: &'a [usize],
    pub sütun_yerleşimleri: &'a [ÖzelSütunYerleşimi],
    pub renkler: &'a [Dolgu],
    pub ilerleme: f32,
    pub fare: Option<(f32, f32)>,
    pub programatik_vurgu: Option<usize>,
}

/// Öğeleri tek bir ortak sahnede zlevel/z/z2 sırasıyla çizer ve gerçek
/// dönüşümlü sınırlarından item hit bölgeleri üretir.
pub(crate) fn özel_öğeleri_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &ÖzelSeri,
    seri_sırası: usize,
    ortam: ÖzelÇizimOrtamı<'_>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> ÖzelÇizimSonucu {
    let Some(çizim) = &seri.öğe_çizimi else {
        return ÖzelÇizimSonucu::default();
    };
    let tur = Mutex::new(ÖzelTurBağlamı::default());
    let mut öğeler = Vec::new();
    let mut anahtarlar = Vec::new();
    let mut veri_sıraları = Vec::new();

    let mut içerideki_veri_sırası = 0;
    for (veri_sırası, veri_öğesi) in seri.veri.iter().enumerate() {
        // CustomSeriesView kendi clipPath'ini varsayılan olarak kapalı tutar;
        // buna karşın dataZoom veri işlemcisi `renderItem` çağrısından önce
        // satırları encode edilen x/y boyutlarıyla süzer. Kırpmayla aynı şey
        // değildir: pencere dışındaki bir polyline tümüyle çağrılmamalıdır.
        if !özel_veri_penceresinde_mi(seri, veri_öğesi, ortam.harita) {
            continue;
        }
        let bu_içerideki_veri_sırası = içerideki_veri_sırası;
        içerideki_veri_sırası += 1;
        let renk = ortam
            .renkler
            .get(veri_sırası)
            .cloned()
            .unwrap_or(Dolgu::Düz(Renk::SİYAH));
        let bağlam = ÖzelÖğeBağlamı {
            veri_sırası,
            ham_veri_sırası: veri_sırası,
            içerideki_veri_sırası: bu_içerideki_veri_sırası,
            seri_sırası,
            seri_adı: seri.ad.as_deref(),
            seri_kimliği: seri.kimlik.as_deref(),
            veri: &seri.veri,
            koordinat_sistemi: seri.koordinat_sistemi,
            koordinat_tanımı: ortam.koordinat_tanımı,
            görünüm_genişliği: ortam.görünüm.0,
            görünüm_yüksekliği: ortam.görünüm.1,
            kodlama: &seri.kodlama,
            güncel_seri_sıraları: ortam.güncel_seri_sıraları,
            sütun_yerleşimleri: ortam.sütun_yerleşimleri,
            renk,
            opaklık: 1.0,
            ilerleme: ortam.ilerleme,
            eylem: &seri.eylem_bağlamı,
            tur: &tur,
            harita: ortam.harita,
        };
        let Some(mut çıktı) = çizim(&bağlam) else {
            continue;
        };
        if let Some(giriş) = &çıktı.girişten {
            çıktı.öğe = öğeyi_ara_değerle(giriş, &çıktı.öğe, ortam.ilerleme, &çıktı.geçiş);
        }
        if let Some(sırasında) = &çıktı.sırasında {
            sırasında(&mut çıktı.öğe, ortam.ilerleme);
        }
        çıktı.öğe.z += seri.z as f32;
        çıktı.öğe.sessiz |= seri.sessiz;
        if seri.kırp {
            çıktı
                .öğe
                .kırpmalar
                .push(KırpmaYolu::yeni(SahneŞekli::Dikdörtgen {
                    kutu: ortam.koordinat_tanımı.alan(),
                    yarıçap: [0.0; 4],
                }));
        }

        let anahtar = çıktı.anahtar.unwrap_or_else(|| {
            veri_öğesi
                .ad
                .clone()
                .unwrap_or_else(|| veri_sırası.to_string())
        });
        anahtarlar.push(anahtar);
        veri_sıraları.push(veri_sırası);
        öğeler.push(çıktı.öğe);
    }

    // Ön sahne yalnız işaretçi hedefini belirler. Vurgu durumu uygulandıktan
    // sonra aynı öğeler tek sahnede yeniden hazırlanır; böylece z sırası tüm
    // item'lar arasında korunur.
    let ön = grafik_sahnesi_hazırla(
        &GrafikBileşeni::yeni().öğeler(öğeler.clone()),
        ortam.görünüm.0,
        ortam.görünüm.1,
    );
    let fare_vurgusu = ortam.fare.and_then(|fare| {
        ön.sahne.isabet(fare).and_then(|isabet| {
            isabet
                .kimlik
                .strip_prefix("graphic:")
                .and_then(|kalan| kalan.split(['.', ':']).next())
                .and_then(|sıra| sıra.parse::<usize>().ok())
                .and_then(|öğe_sırası| veri_sıraları.get(öğe_sırası).copied())
        })
    });
    let vurgulu = ortam.programatik_vurgu.or(fare_vurgusu);
    for (öğe, veri_sırası) in öğeler.iter_mut().zip(&veri_sıraları) {
        let durum = if vurgulu == Some(*veri_sırası) {
            GörselDurum::Vurgu
        } else if seri.veri.get(*veri_sırası).is_some_and(|veri| veri.seçili) {
            GörselDurum::Seçili
        } else {
            GörselDurum::Normal
        };
        görsel_durumu_uygula(öğe, durum);
    }

    let hazır = grafik_sahnesi_hazırla(
        &GrafikBileşeni::yeni().öğeler(öğeler),
        ortam.görünüm.0,
        ortam.görünüm.1,
    );
    hazır.sahne.çiz(yüzey);

    let mut kutular = Vec::with_capacity(hazır.sahne.kökler.len());
    for (öğe_sırası, kök) in hazır.sahne.kökler.iter().enumerate() {
        let veri_sırası = veri_sıraları.get(öğe_sırası).copied().unwrap_or(öğe_sırası);
        let mut kutu = kök.sınır_kutusu();
        if seri.kırp {
            kutu = kutu.and_then(|kutu| dikdörtgen_kesişimi(kutu, ortam.koordinat_tanımı.alan()));
        }
        kutular.push(kutu);
        if !seri.sessiz
            && let Some(kutu) = kutu
        {
            let veri = seri.veri.get(veri_sırası);
            isabetler.push(İsabetBölgesi {
                seri_sırası,
                veri_sırası,
                seri_adı: seri.ad.clone(),
                ad: veri.and_then(|öğe| öğe.ad.clone()),
                değer: veri.and_then(öğe_sayısal_değeri),
                geometri: İsabetGeometrisi::Dikdörtgen(kutu),
            });
        }
    }

    ÖzelÇizimSonucu {
        vurgulu_veri: vurgulu,
        anahtarlar,
        kutular,
    }
}

fn özel_veri_penceresinde_mi(
    seri: &ÖzelSeri,
    öğe: &VeriÖğesi,
    harita: ÖzelKoordinatHaritası<'_>,
) -> bool {
    let ÖzelKoordinatHaritası::Kartezyen2B(kartezyen) = harita else {
        return true;
    };
    let x_öntanımlı = [0];
    let y_öntanımlı = [1];
    let x_boyutları = seri
        .kodlama
        .iter()
        .find(|(kanal, _)| kanal == "x")
        .map(|(_, boyutlar)| boyutlar.as_slice())
        .unwrap_or(&x_öntanımlı);
    let y_boyutları = seri
        .kodlama
        .iter()
        .find(|(kanal, _)| kanal == "y")
        .map(|(_, boyutlar)| boyutlar.as_slice())
        .unwrap_or(&y_öntanımlı);
    eksen_penceresinden_geçer(&kartezyen.x, öğe, x_boyutları)
        && eksen_penceresinden_geçer(&kartezyen.y, öğe, y_boyutları)
}

fn eksen_penceresinden_geçer(
    eksen: &ÇalışmaEkseni, öğe: &VeriÖğesi, boyutlar: &[usize]
) -> bool {
    // Açık `encode.x: []` / `encode.y: []`, serinin ilgili dataZoom
    // işlemcisine bağlı olmadığını belirtir (Gantt apron katmanı).
    if boyutlar.is_empty() {
        return true;
    }
    let Some((baş, son)) = eksen.pencere else {
        return true;
    };
    if eksen.yakınlaştırma_süzme_kipi == YakınlaştırmaSüzmeKipi::Yok {
        return true;
    }
    let değerler = boyutlar.iter().filter_map(|&boyut| {
        let değer = crate::model::ozel::veri_boyutu(öğe, boyut)?;
        değer.sayı().or_else(|| match değer {
            VeriDeğeri::Metin(ad) => eksen.ölçek.kategori_sırası(&ad),
            _ => None,
        })
    });
    match eksen.yakınlaştırma_süzme_kipi {
        YakınlaştırmaSüzmeKipi::Yok => true,
        YakınlaştırmaSüzmeKipi::Süz | YakınlaştırmaSüzmeKipi::Boşalt => değerler
            .clone()
            .all(|değer| değer.is_finite() && değer >= baş && değer <= son),
        YakınlaştırmaSüzmeKipi::ZayıfSüz => {
            let mut değer_var = false;
            let mut solda = false;
            let mut sağda = false;
            for değer in değerler.filter(|değer| değer.is_finite()) {
                değer_var = true;
                if değer >= baş && değer <= son {
                    return true;
                }
                solda |= değer < baş;
                sağda |= değer > son;
            }
            değer_var && solda && sağda
        }
    }
}

fn görsel_durumu_uygula(öğe: &mut GrafikÖğesi, durum: GörselDurum) {
    öğe.durum = durum;
    if let GrafikÖğeİçeriği::Grup(çocuklar) = &mut öğe.içerik {
        for çocuk in çocuklar {
            görsel_durumu_uygula(çocuk, durum);
        }
    }
}

fn dikdörtgen_kesişimi(a: Dikdörtgen, b: Dikdörtgen) -> Option<Dikdörtgen> {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let sağ = a.sağ().min(b.sağ());
    let alt = a.alt().min(b.alt());
    (sağ >= x && alt >= y).then(|| Dikdörtgen::yeni(x, y, sağ - x, alt - y))
}

fn öğe_sayısal_değeri(öğe: &VeriÖğesi) -> Option<f64> {
    öğe.değer.sayı().or_else(|| match &öğe.değer {
        VeriDeğeri::Dizi(değerler) => değerler.last().copied(),
        VeriDeğeri::KarmaDizi(değerler) => değerler.iter().rev().find_map(VeriDeğeri::sayı),
        _ => None,
    })
}

fn öğeyi_ara_değerle(
    başlangıç: &GrafikÖğesi,
    hedef: &GrafikÖğesi,
    t: f32,
    alanlar: &[String],
) -> GrafikÖğesi {
    let t = t.clamp(0.0, 1.0);
    let tümü = alanlar.iter().any(|alan| alan == "all");
    let izin = |alan: &str| tümü || alanlar.iter().any(|aday| aday == alan);
    let mut sonuç = hedef.clone();
    // Custom varsayılan geçişi transform x/y'dir. shape/style yalnız açık
    // transition alanıyla ara değerlenir.
    if alanlar.is_empty() || izin("x") || izin("transform") {
        sonuç.dönüşüm.x = ara(başlangıç.dönüşüm.x, hedef.dönüşüm.x, t);
    }
    if alanlar.is_empty() || izin("y") || izin("transform") {
        sonuç.dönüşüm.y = ara(başlangıç.dönüşüm.y, hedef.dönüşüm.y, t);
    }
    if izin("transform") {
        sonuç.dönüşüm.ölçek_x = ara(başlangıç.dönüşüm.ölçek_x, hedef.dönüşüm.ölçek_x, t);
        sonuç.dönüşüm.ölçek_y = ara(başlangıç.dönüşüm.ölçek_y, hedef.dönüşüm.ölçek_y, t);
        sonuç.dönüşüm.dönüş = ara(başlangıç.dönüşüm.dönüş, hedef.dönüşüm.dönüş, t);
        sonuç.dönüşüm.köken_x = ara(başlangıç.dönüşüm.köken_x, hedef.dönüşüm.köken_x, t);
        sonuç.dönüşüm.köken_y = ara(başlangıç.dönüşüm.köken_y, hedef.dönüşüm.köken_y, t);
    }
    if izin("style") {
        sonuç.stil = stili_ara_değerle(&başlangıç.stil, &hedef.stil, t);
    }
    sonuç.içerik = içeriği_ara_değerle(&başlangıç.içerik, &hedef.içerik, t, alanlar);
    sonuç
}

fn içeriği_ara_değerle(
    başlangıç: &GrafikÖğeİçeriği,
    hedef: &GrafikÖğeİçeriği,
    t: f32,
    alanlar: &[String],
) -> GrafikÖğeİçeriği {
    let biçim = alanlar.iter().any(|alan| alan == "all" || alan == "shape");
    match (başlangıç, hedef) {
        (GrafikÖğeİçeriği::Grup(a), GrafikÖğeİçeriği::Grup(b)) => {
            GrafikÖğeİçeriği::Grup(
                b.iter()
                    .enumerate()
                    .map(|(sıra, hedef)| {
                        a.get(sıra).map_or_else(
                            || hedef.clone(),
                            |baş| öğeyi_ara_değerle(baş, hedef, t, alanlar),
                        )
                    })
                    .collect(),
            )
        }
        (GrafikÖğeİçeriği::Şekil(a), GrafikÖğeİçeriği::Şekil(b)) if biçim => {
            GrafikÖğeİçeriği::Şekil(şekli_ara_değerle(a, b, t))
        }
        (GrafikÖğeİçeriği::Metin(a), GrafikÖğeİçeriği::Metin(b)) if biçim => {
            let mut metin = b.clone();
            metin.konum = (ara(a.konum.0, b.konum.0, t), ara(a.konum.1, b.konum.1, t));
            metin.boyut = ara(a.boyut, b.boyut, t);
            metin.renk = a.renk.karıştır(b.renk, t);
            GrafikÖğeİçeriği::Metin(metin)
        }
        (GrafikÖğeİçeriği::Resim(a), GrafikÖğeİçeriği::Resim(b)) if biçim => {
            let mut resim = b.clone();
            resim.kutu = kutuyu_ara_değerle(a.kutu, b.kutu, t);
            GrafikÖğeİçeriği::Resim(resim)
        }
        _ => hedef.clone(),
    }
}

fn şekli_ara_değerle(başlangıç: &SahneŞekli, hedef: &SahneŞekli, t: f32) -> SahneŞekli {
    match (başlangıç, hedef) {
        (
            SahneŞekli::Dikdörtgen {
                kutu: a,
                yarıçap: ar,
            },
            SahneŞekli::Dikdörtgen {
                kutu: b,
                yarıçap: br,
            },
        ) => SahneŞekli::Dikdörtgen {
            kutu: kutuyu_ara_değerle(*a, *b, t),
            yarıçap: std::array::from_fn(|i| {
                ar.get(i)
                    .zip(br.get(i))
                    .map_or(0.0, |(a, b)| ara(*a, *b, t))
            }),
        },
        (
            SahneŞekli::Daire {
                merkez: a,
                yarıçap: ar,
            },
            SahneŞekli::Daire {
                merkez: b,
                yarıçap: br,
            },
        ) => SahneŞekli::Daire {
            merkez: noktayı_ara_değerle(*a, *b, t),
            yarıçap: ara(*ar, *br, t),
        },
        (
            SahneŞekli::Halka {
                merkez: a,
                iç_yarıçap: ai,
                dış_yarıçap: ad,
            },
            SahneŞekli::Halka {
                merkez: b,
                iç_yarıçap: bi,
                dış_yarıçap: bd,
            },
        ) => SahneŞekli::Halka {
            merkez: noktayı_ara_değerle(*a, *b, t),
            iç_yarıçap: ara(*ai, *bi, t),
            dış_yarıçap: ara(*ad, *bd, t),
        },
        (
            SahneŞekli::Dilim {
                merkez: a,
                iç_yarıçap: ai,
                dış_yarıçap: ad,
                başlangıç_açısı: ab,
                bitiş_açısı: ason,
            },
            SahneŞekli::Dilim {
                merkez: b,
                iç_yarıçap: bi,
                dış_yarıçap: bd,
                başlangıç_açısı: bb,
                bitiş_açısı: bson,
            },
        ) => SahneŞekli::Dilim {
            merkez: noktayı_ara_değerle(*a, *b, t),
            iç_yarıçap: ara(*ai, *bi, t),
            dış_yarıçap: ara(*ad, *bd, t),
            başlangıç_açısı: ara(*ab, *bb, t),
            bitiş_açısı: ara(*ason, *bson, t),
        },
        (
            SahneŞekli::Çizgi {
                başlangıç: aa,
                bitiş: ab,
            },
            SahneŞekli::Çizgi {
                başlangıç: ba,
                bitiş: bb,
            },
        ) => SahneŞekli::Çizgi {
            başlangıç: noktayı_ara_değerle(*aa, *ba, t),
            bitiş: noktayı_ara_değerle(*ab, *bb, t),
        },
        (SahneŞekli::ÇokluÇizgi(a), SahneŞekli::ÇokluÇizgi(b)) => {
            SahneŞekli::ÇokluÇizgi(noktaları_ara_değerle(a, b, t))
        }
        (SahneŞekli::Çokgen(a), SahneŞekli::Çokgen(b)) => {
            SahneŞekli::Çokgen(noktaları_ara_değerle(a, b, t))
        }
        _ => hedef.clone(),
    }
}

fn noktaları_ara_değerle(a: &[(f32, f32)], b: &[(f32, f32)], t: f32) -> Vec<(f32, f32)> {
    b.iter()
        .enumerate()
        .map(|(sıra, b)| a.get(sıra).map_or(*b, |a| noktayı_ara_değerle(*a, *b, t)))
        .collect()
}

fn stili_ara_değerle(a: &SahneStili, b: &SahneStili, t: f32) -> SahneStili {
    let dolgu = match (&a.dolgu, &b.dolgu) {
        (Some(Dolgu::Düz(a)), Some(Dolgu::Düz(b))) => Some(Dolgu::Düz(a.karıştır(*b, t))),
        _ => b.dolgu.clone(),
    };
    SahneStili {
        dolgu,
        çizgi_rengi: match (a.çizgi_rengi, b.çizgi_rengi) {
            (Some(a), Some(b)) => Some(a.karıştır(b, t)),
            _ => b.çizgi_rengi,
        },
        çizgi_kalınlığı: ara(a.çizgi_kalınlığı, b.çizgi_kalınlığı, t),
        çizgi_türü: b.çizgi_türü,
        opaklık: ara(a.opaklık, b.opaklık, t),
        gölge_rengi: match (a.gölge_rengi, b.gölge_rengi) {
            (Some(a), Some(b)) => Some(a.karıştır(b, t)),
            _ => b.gölge_rengi,
        },
        gölge_bulanıklığı: ara(a.gölge_bulanıklığı, b.gölge_bulanıklığı, t),
    }
}

fn kutuyu_ara_değerle(a: Dikdörtgen, b: Dikdörtgen, t: f32) -> Dikdörtgen {
    Dikdörtgen::yeni(
        ara(a.x, b.x, t),
        ara(a.y, b.y, t),
        ara(a.genişlik, b.genişlik, t),
        ara(a.yükseklik, b.yükseklik, t),
    )
}

fn noktayı_ara_değerle(a: (f32, f32), b: (f32, f32), t: f32) -> (f32, f32) {
    (ara(a.0, b.0, t), ara(a.1, b.1, t))
}

fn ara(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn veri_farkı_giris_guncelleme_cikisi_ayirir() {
        let fark = özel_veri_farkı(
            &["a".to_owned(), "b".to_owned()],
            &["b".to_owned(), "c".to_owned()],
        );
        assert_eq!(fark.giren, ["c"]);
        assert_eq!(fark.güncellenen, ["b"]);
        assert_eq!(fark.çıkan, ["a"]);
    }

    #[test]
    fn shape_gecisi_dikdortgeni_ara_degerler() {
        let başlangıç = GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(0.0, 0.0, 10.0, 10.0));
        let hedef = GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(10.0, 20.0, 30.0, 40.0));
        let sonuç = öğeyi_ara_değerle(&başlangıç, &hedef, 0.5, &["shape".to_owned()]);
        let GrafikÖğeİçeriği::Şekil(SahneŞekli::Dikdörtgen { kutu, .. }) = sonuç.içerik
        else {
            panic!("dikdörtgen bekleniyordu");
        };
        assert_eq!(kutu, Dikdörtgen::yeni(5.0, 10.0, 20.0, 25.0));
    }
}
