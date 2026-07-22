//! ECharts 6.1 `matrix` bileşeninin yüzeyden bağımsız çizimi.

use crate::cizim::olay::{MatrisHedefTürü, MatrisHücreBölgesi, İsabetGeometrisi};
use crate::cizim::sahne::yuvarlak_dikdörtgen_yolu;
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, MatrisHücreTürü, MatrisYerleşimi};
use crate::model::matris::{MatrisEtiketiBağlamı, MatrisKoordinatı};
use crate::model::stil::{ÇizgiTürü, ÖğeStili};
use crate::renk::{Dolgu, Renk};
use crate::tema;

/// ECharts MatrixView `createMatrixRect` ile aynı yönde alt-piksel hizası.
/// Başlangıç ve bitiş kenarlarını ayrı ayrı hizalamak, komşu hücrelerle
/// dış sınırın aynı fiziksel piksel üzerinde üst üste gelmesini sağlar.
fn matris_konumunu_keskinleştir(konum: f32, kalınlık: f32) -> f32 {
    let iki_kat = (konum * 2.0).round();
    if ((iki_kat as i64 + kalınlık.round() as i64).rem_euclid(2)) == 0 {
        iki_kat / 2.0
    } else {
        (iki_kat + 1.0) / 2.0
    }
}

fn matris_kutusunu_keskinleştir(
    kutu: crate::koordinat::Dikdörtgen,
    kalınlık: f32,
) -> crate::koordinat::Dikdörtgen {
    if kalınlık <= 0.0 {
        return kutu;
    }
    let x = matris_konumunu_keskinleştir(kutu.x, kalınlık);
    let y = matris_konumunu_keskinleştir(kutu.y, kalınlık);
    let sağ = matris_konumunu_keskinleştir(kutu.sağ(), kalınlık);
    let alt = matris_konumunu_keskinleştir(kutu.alt(), kalınlık);
    crate::koordinat::Dikdörtgen::yeni(x, y, sağ - x, alt - y)
}

/// Çözümlenmiş matrix gövdesini, hiyerarşik başlıklarını ve özel/birleşik
/// hücrelerini boyar. Yerleşim ayrıca matrix'e bağlı serilerce paylaşılır.
pub fn matris_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &MatrisKoordinatı,
    yerleşim: &MatrisYerleşimi,
) -> Vec<MatrisHücreBölgesi> {
    let mut etkileşimler = Vec::new();
    let arkaplan_opaklığı = seçenek.arkaplan_stili.opaklık.unwrap_or(1.0);
    let arkaplan = seçenek
        .arkaplan_stili
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(Renk::SAYDAM))
        .opaklık(arkaplan_opaklığı);
    dikdörtgen_gölgesi_çiz(
        yüzey,
        yerleşim.dış_kutu,
        seçenek.arkaplan_stili.kenarlık_yarıçapı,
        &seçenek.arkaplan_stili,
    );
    yüzey.dikdörtgen(
        yerleşim.dış_kutu,
        &arkaplan,
        seçenek.arkaplan_stili.kenarlık_yarıçapı,
        None,
    );

    let sınır_z2 = seçenek.kenarlık_z2.unwrap_or(99);
    let mut hücreler = yerleşim.hücreler.iter().collect::<Vec<_>>();
    hücreler.sort_by_key(|hücre| hücre.z2);
    for hücre in hücreler.iter().copied().filter(|hücre| hücre.z2 < sınır_z2) {
        let çizilen = hücre_çiz(yüzey, hücre, yerleşim.bileşen_sırası);
        hücre_etkileşimlerini_ekle(
            &mut etkileşimler,
            seçenek,
            yerleşim.bileşen_sırası,
            hücre,
            çizilen,
        );
    }

    // MatrixView dış sınırı ve x/y başlık ayırıcılarını normal hücre
    // kenarlıklarının üstünde, açık yüksek-z2 hücrelerin altında tutar.
    if seçenek.x.göster && seçenek.x.ayırıcı.kalınlık > 0.0 {
        let y = matris_konumunu_keskinleştir(yerleşim.gövde_kutusu.y, seçenek.x.ayırıcı.kalınlık);
        yüzey.çizgi(
            (yerleşim.dış_kutu.x, y),
            (yerleşim.dış_kutu.sağ(), y),
            seçenek.x.ayırıcı.kalınlık,
            seçenek.x.ayırıcı.renk.unwrap_or_else(tema::nötr_40),
            seçenek.x.ayırıcı.tür,
        );
    }
    if seçenek.y.göster && seçenek.y.ayırıcı.kalınlık > 0.0 {
        let x = matris_konumunu_keskinleştir(yerleşim.gövde_kutusu.x, seçenek.y.ayırıcı.kalınlık);
        yüzey.çizgi(
            (x, yerleşim.dış_kutu.y),
            (x, yerleşim.dış_kutu.alt()),
            seçenek.y.ayırıcı.kalınlık,
            seçenek.y.ayırıcı.renk.unwrap_or_else(tema::nötr_40),
            seçenek.y.ayırıcı.tür,
        );
    }
    let arkaplan_kenarlığı = seçenek
        .arkaplan_stili
        .kenarlık_rengi
        .map(|renk| {
            (
                seçenek.arkaplan_stili.kenarlık_kalınlığı.max(0.0),
                renk.opaklık(arkaplan_opaklığı),
            )
        })
        .filter(|(kalınlık, _)| *kalınlık > 0.0);
    if let Some((kalınlık, renk)) = arkaplan_kenarlığı {
        let kenarlık_kutusu = matris_kutusunu_keskinleştir(
            yerleşim.dış_kutu,
            seçenek.arkaplan_stili.kenarlık_kalınlığı,
        );
        dikdörtgen_kenarlığı_çiz(
            yüzey,
            kenarlık_kutusu,
            seçenek.arkaplan_stili.kenarlık_yarıçapı,
            kalınlık,
            renk,
            seçenek.arkaplan_stili.kenarlık_türü,
        );
    }

    for hücre in hücreler.into_iter().filter(|hücre| hücre.z2 >= sınır_z2) {
        let çizilen = hücre_çiz(yüzey, hücre, yerleşim.bileşen_sırası);
        hücre_etkileşimlerini_ekle(
            &mut etkileşimler,
            seçenek,
            yerleşim.bileşen_sırası,
            hücre,
            çizilen,
        );
    }

    etkileşimler
}

#[derive(Default)]
struct ÇizilenMatrisHücresi {
    biçimlenmiş_değer: Option<String>,
    olay_adı: Option<String>,
    etiket_kutusu: Option<Dikdörtgen>,
}

fn hücre_etkileşimlerini_ekle(
    sonuç: &mut Vec<MatrisHücreBölgesi>,
    seçenek: &MatrisKoordinatı,
    bileşen_sırası: usize,
    hücre: &crate::koordinat::MatrisHücreYerleşimi,
    çizilen: ÇizilenMatrisHücresi,
) {
    let ipucu_göster =
        seçenek.ipucu.as_ref().is_some_and(|ipucu| ipucu.göster) && hücre.değer.is_some();
    let ipucu = ipucu_göster
        && seçenek
            .ipucu
            .as_ref()
            .is_some_and(|ipucu| ipucu.içerik_göster);
    let olay_tetikle = seçenek.tetikleme_olayı;
    let etiket_etkileşimli = !hücre
        .etiket
        .sessiz
        .unwrap_or(!(olay_tetikle || ipucu_göster));
    let hedef_türü = match hücre.tür {
        MatrisHücreTürü::XBaşlığı => MatrisHedefTürü::X,
        MatrisHücreTürü::YBaşlığı => MatrisHedefTürü::Y,
        MatrisHücreTürü::Gövde | MatrisHücreTürü::BirleşikGövde => {
            MatrisHedefTürü::Gövde
        }
        MatrisHücreTürü::Köşe | MatrisHücreTürü::BirleşikKöşe => {
            MatrisHedefTürü::Köşe
        }
    };
    let ortak = |geometri| MatrisHücreBölgesi {
        bileşen_sırası,
        hedef_türü,
        ad: çizilen.olay_adı.clone(),
        ipucu_adı: çizilen.biçimlenmiş_değer.clone(),
        değer: hücre.değer.clone(),
        koordinat: [hücre.x_aralığı[0], hücre.y_aralığı[0]],
        geometri,
        imleç: hücre.imleç.clone(),
        ipucu,
        olay_tetikle,
    };

    // `cellRect.silent`: açık değer yoksa dolgusuz hücre ECharts'ta
    // varsayılan olarak yalnız kenarlıkla isabet üretmez.
    if !hücre.sessiz && (ipucu || olay_tetikle || hücre.imleç.is_some()) {
        sonuç.push(ortak(İsabetGeometrisi::Dikdörtgen(hücre.kutu)));
    }

    // Bağlı etiket, tooltip/triggerEvent açıkken host dikdörtgenden bağımsız
    // bir olay hedefidir. Yalnız gerçek ölçülmüş ve hücreye kırpılmış metin
    // alanını kaydet.
    if etiket_etkileşimli
        && (ipucu || olay_tetikle || hücre.imleç.is_some())
        && let Some(kutu) = çizilen.etiket_kutusu
    {
        sonuç.push(ortak(İsabetGeometrisi::Dikdörtgen(kutu)));
    }
}

fn hücre_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    hücre: &crate::koordinat::MatrisHücreYerleşimi,
    bileşen_sırası: usize,
) -> ÇizilenMatrisHücresi {
    let opaklık = hücre.öğe_stili.opaklık.unwrap_or(1.0);
    let dolgu = hücre
        .öğe_stili
        .renk
        .clone()
        .unwrap_or(Dolgu::Düz(Renk::SAYDAM))
        .opaklık(opaklık);
    let kenarlık = hücre
        .öğe_stili
        .kenarlık_rengi
        .map(|renk| {
            (
                hücre.öğe_stili.kenarlık_kalınlığı.max(0.0),
                renk.opaklık(opaklık),
            )
        })
        .filter(|(kalınlık, _)| *kalınlık > 0.0);
    let kutu = kenarlık
        .map(|(kalınlık, _)| matris_kutusunu_keskinleştir(hücre.kutu, kalınlık))
        .unwrap_or(hücre.kutu);
    dikdörtgen_gölgesi_çiz(
        yüzey,
        kutu,
        hücre.öğe_stili.kenarlık_yarıçapı,
        &hücre.öğe_stili,
    );
    yüzey.dikdörtgen(kutu, &dolgu, hücre.öğe_stili.kenarlık_yarıçapı, None);
    if let Some((kalınlık, renk)) = kenarlık {
        dikdörtgen_kenarlığı_çiz(
            yüzey,
            kutu,
            hücre.öğe_stili.kenarlık_yarıçapı,
            kalınlık,
            renk,
            hücre.öğe_stili.kenarlık_türü,
        );
    }

    let Some(değer) = hücre.değer.as_deref() else {
        return ÇizilenMatrisHücresi::default();
    };
    let bağlam = MatrisEtiketiBağlamı {
        bileşen_sırası,
        ad: değer.to_owned(),
        değer: değer.to_owned(),
        koordinat: [hücre.x_aralığı[0], hücre.y_aralığı[0]],
    };
    let metin = if let Some(biçimleyici) = &hücre.etiket_bağlamlı_biçimleyici {
        biçimleyici.uygula(&bağlam)
    } else if let Some(biçimleyici) = &hücre.etiket.biçimleyici {
        biçimleyici
            .uygula(0.0, değer)
            .replace("{name}", değer)
            .replace(
                "{coord}",
                &format!("[{}, {}]", bağlam.koordinat[0], bağlam.koordinat[1]),
            )
    } else {
        değer.to_owned()
    };
    let mut çizilen = ÇizilenMatrisHücresi {
        biçimlenmiş_değer: Some(metin.clone()),
        ..Default::default()
    };
    if !hücre.etiket.göster {
        return çizilen;
    }
    çizilen.olay_adı = Some(metin.clone());
    let merkez = kutu.merkez();
    let boyut = hücre.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let satır_yüksekliği = hücre.etiket.yazı.satır_yüksekliği.unwrap_or(boyut).max(0.0);
    // ECharts MatrixModel `defaultLabelOption.padding` değeri [2, 3, 2, 3]
    // olsa da hücre/katman etiketi bunu açıkça geçersiz kılabilir. Metni hem
    // sararken hem de kullanılabilir satır sayısını hesaplarken gerçek iç
    // boşluğu kullan; asimetrik boşlukta içerik kutusunun merkezini de taşı.
    let [üst, sağ, alt, sol] = hücre.etiket.yazı.iç_boşluk.unwrap_or([2.0, 3.0, 2.0, 3.0]);
    let içerik_merkezi = (merkez.0 + (sol - sağ) / 2.0, merkez.1 + (üst - alt) / 2.0);
    let mut satırlar = metni_satırlara_sar(
        yüzey,
        &metin,
        (kutu.genişlik - sol - sağ).max(0.0),
        boyut,
        hücre.etiket.yazı.kalın,
    );
    let en_çok_satır =
        (((kutu.yükseklik - üst - alt).max(0.0) / satır_yüksekliği).floor() as usize).max(1);
    satırlar.truncate(en_çok_satır);
    let ilk_y = içerik_merkezi.1 + hücre.etiket.kayma.1
        - satır_yüksekliği * satırlar.len().saturating_sub(1) as f32 / 2.0;
    for (sıra, satır) in satırlar.into_iter().enumerate() {
        let çapa = (
            içerik_merkezi.0 + hücre.etiket.kayma.0,
            ilk_y + sıra as f32 * satır_yüksekliği,
        );
        // MatrixView etiketi bağımsız bir Text değil, hücre Rect'ine bağlı
        // `textContent` öğesidir. Dönüş açısı sıfır olsa da zrender saf
        // ötelemeyi üst öğenin affine matrisi altında rasterler.
        let dönüşüm = AfinMatris::ötele(çapa.0, çapa.1 + 0.2);
        let ölçü = yüzey.stilli_yazı_ölç(&satır, boyut, hücre.etiket.yazı.kalın);
        let satır_kutusu = Dikdörtgen::yeni(
            çapa.0 - ölçü.0 / 2.0,
            çapa.1 + 0.2 - ölçü.1 / 2.0,
            ölçü.0,
            ölçü.1,
        );
        if let Some(kırpılmış) = dikdörtgen_kesişimi(satır_kutusu, kutu) {
            çizilen.etiket_kutusu = Some(match çizilen.etiket_kutusu {
                Some(toplam) => dikdörtgen_birleşimi(toplam, kırpılmış),
                None => kırpılmış,
            });
        }
        if let Some(aile) = hücre.etiket.yazı.aile.as_deref() {
            yüzey.dönüşümlü_aileli_yazı(
                &satır,
                (0.0, 0.0),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                boyut,
                hücre.etiket.yazı.renk.unwrap_or_else(tema::ikincil_metin),
                hücre.etiket.yazı.kalın,
                aile,
                dönüşüm,
            );
        } else {
            yüzey.dönüşümlü_yazı(
                &satır,
                (0.0, 0.0),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                boyut,
                hücre.etiket.yazı.renk.unwrap_or_else(tema::ikincil_metin),
                hücre.etiket.yazı.kalın,
                dönüşüm,
            );
        }
    }
    çizilen
}

fn dikdörtgen_gölgesi_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    kutu: Dikdörtgen,
    yarıçap: [f32; 4],
    stil: &ÖğeStili,
) {
    if stil.gölge_bulanıklığı <= 0.0 {
        return;
    }
    let Some(renk) = stil.gölge_rengi else {
        return;
    };
    let yol = yuvarlak_dikdörtgen_yolu(kutu, yarıçap);
    yüzey.yol_gölgesi(
        &yol,
        renk.opaklık(stil.opaklık.unwrap_or(1.0)),
        stil.gölge_bulanıklığı,
        stil.gölge_kayması,
    );
}

fn dikdörtgen_kenarlığı_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    kutu: Dikdörtgen,
    yarıçap: [f32; 4],
    kalınlık: f32,
    renk: Renk,
    tür: ÇizgiTürü,
) {
    if tür == ÇizgiTürü::Düz {
        yüzey.dikdörtgen(
            kutu,
            &Dolgu::Düz(Renk::SAYDAM),
            yarıçap,
            Some((kalınlık, renk)),
        );
    } else {
        yüzey.yol_çiz(
            &yuvarlak_dikdörtgen_yolu(kutu, yarıçap),
            kalınlık,
            renk,
            tür,
        );
    }
}

fn dikdörtgen_kesişimi(a: Dikdörtgen, b: Dikdörtgen) -> Option<Dikdörtgen> {
    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = a.sağ().min(b.sağ());
    let y1 = a.alt().min(b.alt());
    (x1 > x0 && y1 > y0).then(|| Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0))
}

fn dikdörtgen_birleşimi(a: Dikdörtgen, b: Dikdörtgen) -> Dikdörtgen {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = a.sağ().max(b.sağ());
    let y1 = a.alt().max(b.alt());
    Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0)
}

fn metni_satırlara_sar(
    yüzey: &dyn ÇizimYüzeyi,
    metin: &str,
    en_çok_genişlik: f32,
    boyut: f32,
    kalın: bool,
) -> Vec<String> {
    let mut sonuç = Vec::new();
    for açık_satır in metin.split('\n') {
        if açık_satır.is_empty() {
            sonuç.push(String::new());
            continue;
        }
        let mut satır = String::new();
        for karakter in açık_satır.chars() {
            let mut aday = satır.clone();
            aday.push(karakter);
            let genişlik = yüzey.stilli_yazı_ölç(&aday, boyut, kalın).0;
            if !satır.is_empty() && genişlik > en_çok_genişlik {
                sonuç.push(std::mem::take(&mut satır));
            }
            satır.push(karakter);
        }
        if !satır.is_empty() {
            sonuç.push(satır);
        }
    }
    if sonuç.is_empty() {
        sonuç.push(String::new());
    }
    sonuç
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::model::bilesen::İpucu;
    use crate::model::matris::{MatrisBoyutu, MatrisGövdeHücresi};
    use crate::model::stil::ÖğeStili;

    #[test]
    fn tooltip_trigger_event_ve_cursor_ayri_rect_etiket_hedefleri_uretir() {
        let seçenek = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni().imleç("pointer").veri(["A"]))
            .y(MatrisBoyutu::yeni().veri(["Y"]))
            .gövde_hücresi(
                MatrisGövdeHücresi::yeni("A", "Y")
                    .değer("D")
                    .imleç("crosshair")
                    .öğe_stili(ÖğeStili::yeni().renk(0x112233)),
            )
            .ipucu(İpucu::yeni())
            .tetikleme_olayı(true);
        let yerleşim = MatrisYerleşimi::kur_sıralı(&seçenek, (400.0, 300.0), (0, 0), 4)
            .expect("matrix yerleşimi");
        let mut yüzey = KayıtYüzeyi::yeni(400.0, 300.0);

        let bölgeler = matris_çiz(&mut yüzey, &seçenek, &yerleşim);

        let x_etiketi = bölgeler
            .iter()
            .find(|bölge| {
                bölge.hedef_türü == MatrisHedefTürü::X && bölge.ipucu_adı.as_deref() == Some("A")
            })
            .unwrap();
        assert_eq!(x_etiketi.bileşen_sırası, 4);
        assert_eq!(x_etiketi.imleç.as_deref(), Some("pointer"));
        assert!(x_etiketi.ipucu && x_etiketi.olay_tetikle);

        let gövde = bölgeler
            .iter()
            .filter(|bölge| bölge.hedef_türü == MatrisHedefTürü::Gövde)
            .collect::<Vec<_>>();
        assert_eq!(gövde.len(), 2, "dolgulu rect ve bağlı etiket ayrı hedefler");
        assert!(
            gövde
                .iter()
                .all(|bölge| bölge.imleç.as_deref() == Some("crosshair"))
        );
        assert!(gövde.iter().all(|bölge| bölge.koordinat == [0, 0]));
    }

    #[test]
    fn label_silent_ve_item_style_opacity_shadow_border_type_uygulanir() {
        let seçenek = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni()
                .etiket(crate::model::stil::Etiket::yeni().göster(true).sessiz(true))
                .veri(["A"]))
            .y(MatrisBoyutu::yeni().veri(["Y"]))
            .gövde_hücresi(
                MatrisGövdeHücresi::yeni("A", "Y").değer("D").öğe_stili(
                    ÖğeStili::yeni()
                        .renk(0x224466)
                        .opaklık(0.5)
                        .kenarlık_rengi(0xff0000)
                        .kenarlık_kalınlığı(2.0)
                        .kenarlık_türü(ÇizgiTürü::Kesikli)
                        .kenarlık_yarıçapı(8.0)
                        .gölge_rengi(0x000000)
                        .gölge_bulanıklığı(4.0)
                        .gölge_kayması(2.0, 3.0),
                ),
            )
            .ipucu(İpucu::yeni())
            .tetikleme_olayı(true);
        let yerleşim = MatrisYerleşimi::kur(&seçenek, (400.0, 300.0), (0, 0)).unwrap();
        let mut yüzey = KayıtYüzeyi::yeni(400.0, 300.0);

        let bölgeler = matris_çiz(&mut yüzey, &seçenek, &yerleşim);

        assert!(
            !bölgeler
                .iter()
                .any(|bölge| bölge.hedef_türü == MatrisHedefTürü::X),
            "label.silent, dolgusuz x rect'in etiket hedefini kapatmalı"
        );
        let döküm = yüzey.döküm();
        assert!(döküm.contains("gölge"), "{döküm}");
        assert!(
            döküm.contains("Kesikli") || döküm.contains("kesikli"),
            "{döküm}"
        );
        assert!(döküm.contains("@0.5"), "{döküm}");
        assert!(döküm.contains(" Y("), "{döküm}");
    }
}
