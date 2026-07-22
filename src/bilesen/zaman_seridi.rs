//! Zaman şeridi (timeline) — `echarts/src/component/timeline` karşılığı.
//!
//! Eski [`zaman_şeridi_çiz`] işlevi `GrafikGörünümü::film` için geriye
//! uyumlu, yalın alt şeridi korur. [`seçenekli_zaman_şeridi_çiz`], genel
//! `timeline` option modelini yatay/dikey yerleşim, kontrol, etiket,
//! checkpoint ve ters eksen davranışıyla çizer.

use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, keskin, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::bilesen::Yön;
use crate::model::deger::VeriDeğeri;
use crate::model::stil::ÇizgiTürü;
use crate::model::zaman_seridi::{
    ZamanŞeridi, ZamanŞeridiEtiketKonumu, ZamanŞeridiKontrolKonumu, ZamanŞeridiSimgesi,
};
use crate::model::{DikeyKonum, YatayKonum};
use crate::renk::Dolgu;
use crate::tema;

/// Zaman şeridi düğmeleri.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZamanŞeridiEylemi {
    /// Verilen kareye atla.
    Kare(usize),
    /// Oynat/durdur geçişi.
    OynatDurdur,
}

/// Şeridin kapladığı alt bant yüksekliği (piksel).
pub const ŞERİT_YÜKSEKLİĞİ: f32 = 36.0;

/// Zaman şeridini pencerenin altına çizer; tıklanabilir kutuları döndürür.
pub fn zaman_şeridi_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    geçerli: usize,
    toplam: usize,
    oynuyor: bool,
) -> Vec<(Dikdörtgen, ZamanŞeridiEylemi)> {
    let mut kutular = Vec::new();
    if toplam == 0 {
        return kutular;
    }
    let genişlik = yüzey.genişlik();
    let yükseklik = yüzey.yükseklik();
    let orta_y = keskin(yükseklik - ŞERİT_YÜKSEKLİĞİ / 2.0);
    let vurgu = crate::tema::palet_rengi(0);

    // 1) Oynat/durdur düğmesi (solda): oynarken iki dikey çubuk, dururken
    //    üçgen.
    let düğme_merkezi = (24.0, orta_y);
    let yarıçap = 9.0;
    yüzey.daire(düğme_merkezi, yarıçap + 3.0, None, Some((1.0, vurgu)));
    if oynuyor {
        for kayma in [-2.5, 2.5] {
            let çubuk = Dikdörtgen::yeni(
                düğme_merkezi.0 + kayma - 1.25,
                düğme_merkezi.1 - 4.5,
                2.5,
                9.0,
            );
            yüzey.dikdörtgen(çubuk, &Dolgu::Düz(vurgu), [1.0; 4], None);
        }
    } else {
        let mut üçgen = Yol::yeni();
        üçgen.taşı((düğme_merkezi.0 - 3.0, düğme_merkezi.1 - 5.0));
        üçgen.çiz((düğme_merkezi.0 + 5.0, düğme_merkezi.1));
        üçgen.çiz((düğme_merkezi.0 - 3.0, düğme_merkezi.1 + 5.0));
        üçgen.kapat();
        yüzey.yol_doldur(&üçgen, &Dolgu::Düz(vurgu));
    }
    kutular.push((
        Dikdörtgen::yeni(
            düğme_merkezi.0 - yarıçap - 4.0,
            düğme_merkezi.1 - yarıçap - 4.0,
            (yarıçap + 4.0) * 2.0,
            (yarıçap + 4.0) * 2.0,
        ),
        ZamanŞeridiEylemi::OynatDurdur,
    ));

    // 2) Eksen çizgisi + kare noktaları.
    let sol = 52.0;
    let sağ = (genişlik - 24.0).max(sol + 1.0);
    yüzey.çizgi(
        (sol, orta_y),
        (sağ, orta_y),
        2.0,
        tema::nötr_15(),
        crate::model::stil::ÇizgiTürü::Düz,
    );
    let adım = if toplam > 1 {
        (sağ - sol) / (toplam - 1) as f32
    } else {
        0.0
    };
    for sıra in 0..toplam {
        let x = if toplam > 1 {
            sol + sıra as f32 * adım
        } else {
            (sol + sağ) / 2.0
        };
        let seçili = sıra == geçerli;
        if seçili {
            // Geçerli kare: dolu + halka vurgusu.
            yüzey.daire(
                (x, orta_y),
                7.5,
                Some(&Dolgu::Düz(vurgu.opaklık(0.25))),
                None,
            );
            yüzey.daire((x, orta_y), 5.0, Some(&Dolgu::Düz(vurgu)), None);
        } else {
            yüzey.daire(
                (x, orta_y),
                4.0,
                Some(&Dolgu::Düz(tema::nötr_00())),
                Some((1.5, vurgu.opaklık(0.7))),
            );
        }
        kutular.push((
            Dikdörtgen::yeni(x - 9.0, orta_y - 9.0, 18.0, 18.0),
            ZamanŞeridiEylemi::Kare(sıra),
        ));
    }
    kutular
}

fn veri_metni(değer: &VeriDeğeri) -> String {
    match değer {
        VeriDeğeri::Boş => String::new(),
        VeriDeğeri::Sayı(sayı) => {
            if sayı.fract().abs() < f64::EPSILON {
                format!("{sayı:.0}")
            } else {
                sayı.to_string()
            }
        }
        VeriDeğeri::Çift([x, y]) => format!("{x}, {y}"),
        VeriDeğeri::Dizi(dizi) => dizi
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        VeriDeğeri::KarmaDizi(dizi) => dizi.iter().map(veri_metni).collect::<Vec<_>>().join(", "),
        VeriDeğeri::Metin(metin) => metin.clone(),
        VeriDeğeri::Mantıksal(değer) => değer.to_string(),
        VeriDeğeri::Zaman(ms) => ms.to_string(),
    }
}

fn yatay_değer(konum: YatayKonum, bütün: f32, boyut: f32, boşluk: [f32; 4]) -> f32 {
    match konum {
        YatayKonum::Sol => boşluk[3],
        // getLayoutRect önce dış kutuyu padding ile küçültülmüş alanda
        // ortalar, sonra içerik başlangıcına sol padding'i ekler; iki işlem
        // birbirini götürür.
        YatayKonum::Orta => bütün / 2.0 - boyut / 2.0,
        YatayKonum::Sağ => bütün - boyut - boşluk[1] - boşluk[3],
        YatayKonum::Değer(uzunluk) => uzunluk.çöz(bütün) + boşluk[3],
    }
}

fn dikey_değer(konum: DikeyKonum, bütün: f32, boyut: f32, boşluk: [f32; 4]) -> f32 {
    match konum {
        DikeyKonum::Üst => boşluk[0],
        DikeyKonum::Orta => bütün / 2.0 - boyut / 2.0,
        DikeyKonum::Alt => bütün - boyut - boşluk[0] - boşluk[2],
        DikeyKonum::Değer(uzunluk) => uzunluk.çöz(bütün) + boşluk[0],
    }
}

/// ECharts `getLayoutRect(..., timeline.padding)` karşılığı olan içerik
/// kutusu. Açık genişlik/yükseklik iç boşluktan etkilenmez; açık olmayan
/// boyutlar iki kenar ve iç boşluk çıkarılarak hesaplanır.
fn seçenek_kutusu(yüzey: &dyn ÇizimYüzeyi, seçenek: &ZamanŞeridi) -> Dikdörtgen {
    let bütün_g = yüzey.genişlik();
    let bütün_y = yüzey.yükseklik();
    let boşluk = seçenek.iç_boşluk;
    let yatay_boşluk = boşluk[1] + boşluk[3];
    let dikey_boşluk = boşluk[0] + boşluk[2];
    let sol_sayısı = seçenek.sol.and_then(|sol| match sol {
        YatayKonum::Değer(uzunluk) => Some(uzunluk.çöz(bütün_g)),
        YatayKonum::Sol => Some(0.0),
        YatayKonum::Orta | YatayKonum::Sağ => None,
    });
    let sağ = seçenek.sağ.map(|değer| değer.çöz(bütün_g));
    let üst_sayısı = seçenek.üst.and_then(|üst| match üst {
        DikeyKonum::Değer(uzunluk) => Some(uzunluk.çöz(bütün_y)),
        DikeyKonum::Üst => Some(0.0),
        DikeyKonum::Orta | DikeyKonum::Alt => None,
    });
    let alt = seçenek.alt.map(|değer| değer.çöz(bütün_y));
    let genişlik = seçenek
        .genişlik
        .map(|değer| değer.çöz(bütün_g))
        .unwrap_or_else(|| {
            (bütün_g - sol_sayısı.unwrap_or(0.0) - sağ.unwrap_or(0.0) - yatay_boşluk).max(0.0)
        });
    let yükseklik = seçenek
        .yükseklik
        .map(|değer| değer.çöz(bütün_y))
        .unwrap_or_else(|| {
            (bütün_y - üst_sayısı.unwrap_or(0.0) - alt.unwrap_or(0.0) - dikey_boşluk).max(0.0)
        });
    let x = if let Some(sol) = seçenek.sol {
        yatay_değer(sol, bütün_g, genişlik, boşluk)
    } else {
        bütün_g - sağ.unwrap_or(0.0) - genişlik - yatay_boşluk + boşluk[3]
    };
    let y = if let Some(üst) = seçenek.üst {
        dikey_değer(üst, bütün_y, yükseklik, boşluk)
    } else {
        bütün_y - alt.unwrap_or(0.0) - yükseklik - dikey_boşluk + boşluk[0]
    };
    Dikdörtgen::yeni(x, y, genişlik, yükseklik)
}

fn öğe_rengi(
    stil: &crate::model::stil::ÖğeStili,
    öntanımlı: crate::renk::Renk,
) -> crate::renk::Renk {
    let renk = stil.renk.as_ref().map(Dolgu::temsilî).unwrap_or(öntanımlı);
    stil.opaklık
        .map(|opaklık| renk.opaklık(opaklık))
        .unwrap_or(renk)
}

fn etiket_metni(seçenek: &ZamanŞeridi, sıra: usize) -> String {
    let ham = seçenek
        .veri
        .get(sıra)
        .map(veri_metni)
        .unwrap_or_else(|| sıra.to_string());
    seçenek
        .etiket
        .biçimleyici
        .as_ref()
        .map(|kalıp| kalıp.replace("{value}", &ham))
        .unwrap_or(ham)
}

fn üçgen(yüzey: &mut dyn ÇizimYüzeyi, noktalar: [(f32, f32); 3], renk: crate::renk::Renk) {
    let mut yol = Yol::yeni();
    yol.taşı(noktalar[0]);
    yol.çiz(noktalar[1]);
    yol.çiz(noktalar[2]);
    yol.kapat();
    yüzey.yol_doldur(&yol, &Dolgu::Düz(renk));
}

fn oynat_düğmesi(
    yüzey: &mut dyn ÇizimYüzeyi,
    merkez: (f32, f32),
    boyut: f32,
    oynuyor: bool,
    renk: crate::renk::Renk,
) {
    let yarıçap = boyut * 0.43;
    yüzey.daire(merkez, yarıçap, None, Some((2.5, renk)));
    if oynuyor {
        let yükseklik = boyut * 0.36;
        let genişlik = boyut * 0.09;
        for kayma in [-boyut * 0.12, boyut * 0.12] {
            yüzey.dikdörtgen(
                Dikdörtgen::yeni(
                    merkez.0 + kayma - genişlik / 2.0,
                    merkez.1 - yükseklik / 2.0,
                    genişlik,
                    yükseklik,
                ),
                &Dolgu::Düz(renk),
                [0.0; 4],
                None,
            );
        }
    } else {
        üçgen(
            yüzey,
            [
                (merkez.0 - boyut * 0.13, merkez.1 - boyut * 0.20),
                (merkez.0 + boyut * 0.22, merkez.1),
                (merkez.0 - boyut * 0.13, merkez.1 + boyut * 0.20),
            ],
            renk,
        );
    }
}

fn ok_düğmesi(
    yüzey: &mut dyn ÇizimYüzeyi,
    merkez: (f32, f32),
    boyut: f32,
    yön: (f32, f32),
    renk: crate::renk::Renk,
) {
    let dik = (-yön.1, yön.0);
    let uç = (
        merkez.0 + yön.0 * boyut * 0.32,
        merkez.1 + yön.1 * boyut * 0.32,
    );
    let arka = (
        merkez.0 - yön.0 * boyut * 0.24,
        merkez.1 - yön.1 * boyut * 0.24,
    );
    let yarı = boyut * 0.28;
    let mut yol = Yol::yeni();
    yol.taşı((arka.0 + dik.0 * yarı, arka.1 + dik.1 * yarı));
    yol.çiz(uç);
    yol.çiz((arka.0 - dik.0 * yarı, arka.1 - dik.1 * yarı));
    yüzey.yol_çiz(&yol, 3.0, renk, ÇizgiTürü::Düz);
}

fn sonraki_sıra(seçenek: &ZamanŞeridi, geçerli: usize, toplam: usize, ileri: bool) -> usize {
    if toplam == 0 {
        return 0;
    }
    let artı = if seçenek.ters { !ileri } else { ileri };
    if artı {
        if geçerli + 1 < toplam {
            geçerli + 1
        } else if seçenek.döngü {
            0
        } else {
            toplam - 1
        }
    } else if geçerli > 0 {
        geçerli - 1
    } else if seçenek.döngü {
        toplam - 1
    } else {
        0
    }
}

fn dikey_şerit(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &ZamanŞeridi,
    kutu: Dikdörtgen,
    geçerli: usize,
    toplam: usize,
    oynuyor: bool,
) -> Vec<(Dikdörtgen, ZamanŞeridiEylemi)> {
    let mut isabetler = Vec::new();
    let kontrol = &seçenek.kontrol_stili;
    let kontrol_boyutu = if kontrol.göster {
        kontrol.öğe_boyutu
    } else {
        0.0
    };
    let boşluk = if kontrol.göster {
        kontrol.öğe_boşluğu
    } else {
        0.0
    };
    let boyut_ve_boşluk = kontrol_boyutu + boşluk;
    let mut eksen_sol = 0.0;
    let mut eksen_sağ = kutu.yükseklik;
    let mut oynat = None;
    let mut önceki = None;
    let mut sonraki = None;
    let başta = matches!(
        kontrol.konum,
        ZamanŞeridiKontrolKonumu::Sol | ZamanŞeridiKontrolKonumu::Alt
    );
    if kontrol.göster {
        if başta {
            if kontrol.oynat_göster {
                oynat = Some(0.0);
                eksen_sol += boyut_ve_boşluk;
            }
            if kontrol.önceki_göster {
                önceki = Some(eksen_sol);
                eksen_sol += boyut_ve_boşluk;
            }
            if kontrol.sonraki_göster {
                sonraki = Some(eksen_sağ - kontrol_boyutu);
                eksen_sağ -= boyut_ve_boşluk;
            }
        } else {
            if kontrol.oynat_göster {
                oynat = Some(eksen_sağ - kontrol_boyutu);
                eksen_sağ -= boyut_ve_boşluk;
            }
            if kontrol.önceki_göster {
                önceki = Some(0.0);
                eksen_sol += boyut_ve_boşluk;
            }
            if kontrol.sonraki_göster {
                sonraki = Some(eksen_sağ - kontrol_boyutu);
                eksen_sağ -= boyut_ve_boşluk;
            }
        }
    }
    let mut eksen_kapsamı = [eksen_sol, eksen_sağ];
    if seçenek.ters {
        eksen_kapsamı.reverse();
    }

    let etiket_uzaklığı = match seçenek.etiket.konum {
        ZamanŞeridiEtiketKonumu::Uzaklık(uzaklık) => uzaklık,
        ZamanŞeridiEtiketKonumu::Sol => -10.0,
        ZamanŞeridiEtiketKonumu::Sağ => 10.0,
        ZamanŞeridiEtiketKonumu::Otomatik => {
            if kutu.x + kutu.genişlik / 2.0 < yüzey.genişlik() / 2.0 {
                10.0
            } else {
                -10.0
            }
        }
        ZamanŞeridiEtiketKonumu::Üst => -10.0,
        ZamanŞeridiEtiketKonumu::Alt => 10.0,
    };
    // SliderTimelineView, pozitif sayısal etiket konumunda kontrol
    // simgelerinin sol sınırını viewRect'e dayar. 24 px'lik öntanımlı
    // kontrolde eksen bu yüzden kutunun x değerinden 12 px sağdadır.
    let eksen_x = if etiket_uzaklığı >= 0.0 {
        kutu.x + kontrol_boyutu / 2.0
    } else {
        kutu.x + kutu.genişlik - kontrol_boyutu / 2.0
    };
    let ekran_y = |yerel: f32| kutu.y + kutu.yükseklik - yerel;
    let koordinat = |sıra: usize| {
        if toplam <= 1 {
            (eksen_kapsamı[0] + eksen_kapsamı[1]) / 2.0
        } else {
            eksen_kapsamı[0]
                + (eksen_kapsamı[1] - eksen_kapsamı[0]) * sıra as f32 / (toplam - 1) as f32
        }
    };

    let çizgi_rengi = seçenek
        .çizgi_stili
        .renk
        .unwrap_or_else(tema::aksan_10)
        .opaklık(seçenek.çizgi_stili.opaklık);
    if seçenek.çizgi_göster {
        yüzey.çizgi(
            (eksen_x, ekran_y(eksen_kapsamı[0])),
            (eksen_x, ekran_y(eksen_kapsamı[1])),
            seçenek.çizgi_stili.kalınlık,
            çizgi_rengi,
            seçenek.çizgi_stili.tür,
        );
        let seçili_koordinat = koordinat(geçerli);
        yüzey.çizgi(
            (eksen_x, ekran_y(eksen_kapsamı[0])),
            (eksen_x, ekran_y(seçili_koordinat)),
            seçenek.çizgi_stili.kalınlık,
            tema::aksan_30(),
            seçenek.çizgi_stili.tür,
        );
    }

    let etiket_rengi = seçenek.etiket.yazı.renk.unwrap_or_else(tema::üçüncül_metin);
    let etiket_boyutu = seçenek.etiket.yazı.boyut.unwrap_or(12.0);
    let etiket_adımı = seçenek.etiket.aralık.map(|aralık| aralık + 1).unwrap_or(1);
    for sıra in 0..toplam {
        let y = ekran_y(koordinat(sıra));
        if seçenek.simge == ZamanŞeridiSimgesi::Daire {
            let renk = if sıra < geçerli {
                tema::aksan_40()
            } else {
                öğe_rengi(&seçenek.öğe_stili, tema::aksan_20())
            };
            yüzey.daire(
                (eksen_x, y),
                seçenek.simge_boyutu / 2.0,
                Some(&Dolgu::Düz(renk)),
                seçenek
                    .öğe_stili
                    .kenarlık_rengi
                    .map(|kenarlık| (seçenek.öğe_stili.kenarlık_kalınlığı, kenarlık)),
            );
        }
        if seçenek.etiket.göster && sıra % etiket_adımı == 0 {
            let x = eksen_x + etiket_uzaklığı;
            let hiza = if etiket_uzaklığı >= 0.0 {
                YatayHiza::Sol
            } else {
                YatayHiza::Sağ
            };
            let metin = etiket_metni(seçenek, sıra);
            if seçenek.etiket.döndürme.abs() > f32::EPSILON {
                let açı = seçenek.etiket.döndürme.to_radians();
                let dönüşüm = AfinMatris::ötele(x, y)
                    .çarp(AfinMatris::döndür(açı))
                    .çarp(AfinMatris::ötele(-x, -y));
                yüzey.dönüşümlü_yazı(
                    &metin,
                    (x, y),
                    hiza,
                    DikeyHiza::Orta,
                    etiket_boyutu,
                    etiket_rengi,
                    seçenek.etiket.yazı.kalın,
                    dönüşüm,
                );
            } else {
                yüzey.yazı(
                    &metin,
                    (x, y),
                    hiza,
                    DikeyHiza::Orta,
                    etiket_boyutu,
                    etiket_rengi,
                    seçenek.etiket.yazı.kalın,
                );
            }
        }
        isabetler.push((
            Dikdörtgen::yeni(eksen_x - 9.0, y - 9.0, 18.0, 18.0),
            ZamanŞeridiEylemi::Kare(sıra),
        ));
    }

    if seçenek.kontrol_noktası_stili.simge == ZamanŞeridiSimgesi::Daire {
        let y = ekran_y(koordinat(geçerli));
        let stil = &seçenek.kontrol_noktası_stili.öğe_stili;
        let renk = öğe_rengi(stil, tema::aksan_50());
        yüzey.daire(
            (eksen_x, y),
            seçenek.kontrol_noktası_stili.simge_boyutu / 2.0,
            Some(&Dolgu::Düz(renk)),
            stil.kenarlık_rengi
                .map(|kenarlık| (stil.kenarlık_kalınlığı, kenarlık)),
        );
    }

    let kontrol_rengi = kontrol.renk.unwrap_or_else(tema::aksan_50);
    let simge_merkezi =
        |yerel: f32, simge_boyutu: f32| (eksen_x, ekran_y(yerel + simge_boyutu / 2.0));
    if let Some(yerel) = oynat {
        let merkez = simge_merkezi(yerel, kontrol_boyutu);
        oynat_düğmesi(yüzey, merkez, kontrol_boyutu, oynuyor, kontrol_rengi);
        isabetler.push((
            Dikdörtgen::yeni(
                merkez.0 - kontrol_boyutu / 2.0,
                merkez.1 - kontrol_boyutu / 2.0,
                kontrol_boyutu,
                kontrol_boyutu,
            ),
            ZamanŞeridiEylemi::OynatDurdur,
        ));
    }
    let ok_boyutu = kontrol_boyutu * 0.75;
    if let Some(yerel) = önceki {
        let merkez = simge_merkezi(yerel, ok_boyutu);
        ok_düğmesi(yüzey, merkez, ok_boyutu, (0.0, 1.0), kontrol_rengi);
        isabetler.push((
            Dikdörtgen::yeni(
                merkez.0 - kontrol_boyutu / 2.0,
                merkez.1 - kontrol_boyutu / 2.0,
                kontrol_boyutu,
                kontrol_boyutu,
            ),
            ZamanŞeridiEylemi::Kare(sonraki_sıra(seçenek, geçerli, toplam, false)),
        ));
    }
    if let Some(yerel) = sonraki {
        let merkez = simge_merkezi(yerel, ok_boyutu);
        ok_düğmesi(yüzey, merkez, ok_boyutu, (0.0, -1.0), kontrol_rengi);
        isabetler.push((
            Dikdörtgen::yeni(
                merkez.0 - kontrol_boyutu / 2.0,
                merkez.1 - kontrol_boyutu / 2.0,
                kontrol_boyutu,
                kontrol_boyutu,
            ),
            ZamanŞeridiEylemi::Kare(sonraki_sıra(seçenek, geçerli, toplam, true)),
        ));
    }
    isabetler
}

fn yatay_şerit(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &ZamanŞeridi,
    kutu: Dikdörtgen,
    geçerli: usize,
    toplam: usize,
    oynuyor: bool,
) -> Vec<(Dikdörtgen, ZamanŞeridiEylemi)> {
    let mut isabetler = Vec::new();
    let kontrol = &seçenek.kontrol_stili;
    let boyut = if kontrol.göster {
        kontrol.öğe_boyutu
    } else {
        0.0
    };
    let boşluk = if kontrol.göster {
        kontrol.öğe_boşluğu
    } else {
        0.0
    };
    let mut sol = 0.0;
    let mut sağ = kutu.genişlik;
    let başta = matches!(
        kontrol.konum,
        ZamanŞeridiKontrolKonumu::Sol | ZamanŞeridiKontrolKonumu::Alt
    );
    let mut oynat = None;
    let mut önceki = None;
    let mut sonraki = None;
    if kontrol.göster {
        if başta {
            if kontrol.oynat_göster {
                oynat = Some(sol);
                sol += boyut + boşluk;
            }
            if kontrol.önceki_göster {
                önceki = Some(sol);
                sol += boyut + boşluk;
            }
            if kontrol.sonraki_göster {
                sonraki = Some(sağ - boyut);
                sağ -= boyut + boşluk;
            }
        } else {
            if kontrol.oynat_göster {
                oynat = Some(sağ - boyut);
                sağ -= boyut + boşluk;
            }
            if kontrol.önceki_göster {
                önceki = Some(sol);
                sol += boyut + boşluk;
            }
            if kontrol.sonraki_göster {
                sonraki = Some(sağ - boyut);
                sağ -= boyut + boşluk;
            }
        }
    }
    let mut kapsam = [sol, sağ];
    if seçenek.ters {
        kapsam.reverse();
    }
    let koordinat = |sıra: usize| {
        if toplam <= 1 {
            (kapsam[0] + kapsam[1]) / 2.0
        } else {
            kapsam[0] + (kapsam[1] - kapsam[0]) * sıra as f32 / (toplam - 1) as f32
        }
    };
    let etiket_uzaklığı = match seçenek.etiket.konum {
        ZamanŞeridiEtiketKonumu::Uzaklık(uzaklık) => uzaklık,
        ZamanŞeridiEtiketKonumu::Üst => -10.0,
        ZamanŞeridiEtiketKonumu::Alt => 10.0,
        ZamanŞeridiEtiketKonumu::Otomatik => {
            if kutu.y + kutu.yükseklik / 2.0 < yüzey.yükseklik() / 2.0 {
                -10.0
            } else {
                10.0
            }
        }
        ZamanŞeridiEtiketKonumu::Sol => -10.0,
        ZamanŞeridiEtiketKonumu::Sağ => 10.0,
    };
    let eksen_y = if etiket_uzaklığı >= 0.0 {
        kutu.y
    } else {
        kutu.alt()
    };
    let çizgi_rengi = seçenek
        .çizgi_stili
        .renk
        .unwrap_or_else(tema::aksan_10)
        .opaklık(seçenek.çizgi_stili.opaklık);
    if seçenek.çizgi_göster {
        yüzey.çizgi(
            (kutu.x + kapsam[0], eksen_y),
            (kutu.x + kapsam[1], eksen_y),
            seçenek.çizgi_stili.kalınlık,
            çizgi_rengi,
            seçenek.çizgi_stili.tür,
        );
        yüzey.çizgi(
            (kutu.x + kapsam[0], eksen_y),
            (kutu.x + koordinat(geçerli), eksen_y),
            seçenek.çizgi_stili.kalınlık,
            tema::aksan_30(),
            seçenek.çizgi_stili.tür,
        );
    }
    let etiket_rengi = seçenek.etiket.yazı.renk.unwrap_or_else(tema::üçüncül_metin);
    let etiket_boyutu = seçenek.etiket.yazı.boyut.unwrap_or(12.0);
    let etiket_adımı = seçenek.etiket.aralık.map(|aralık| aralık + 1).unwrap_or(1);
    for sıra in 0..toplam {
        let x = kutu.x + koordinat(sıra);
        if seçenek.simge == ZamanŞeridiSimgesi::Daire {
            yüzey.daire(
                (x, eksen_y),
                seçenek.simge_boyutu / 2.0,
                Some(&Dolgu::Düz(öğe_rengi(
                    &seçenek.öğe_stili,
                    tema::aksan_20(),
                ))),
                None,
            );
        }
        if seçenek.etiket.göster && sıra % etiket_adımı == 0 {
            yüzey.yazı(
                &etiket_metni(seçenek, sıra),
                (x, eksen_y + etiket_uzaklığı),
                YatayHiza::Orta,
                if etiket_uzaklığı >= 0.0 {
                    DikeyHiza::Üst
                } else {
                    DikeyHiza::Alt
                },
                etiket_boyutu,
                etiket_rengi,
                seçenek.etiket.yazı.kalın,
            );
        }
        isabetler.push((
            Dikdörtgen::yeni(x - 9.0, eksen_y - 9.0, 18.0, 18.0),
            ZamanŞeridiEylemi::Kare(sıra),
        ));
    }
    let seçili_x = kutu.x + koordinat(geçerli);
    yüzey.daire(
        (seçili_x, eksen_y),
        seçenek.kontrol_noktası_stili.simge_boyutu / 2.0,
        Some(&Dolgu::Düz(öğe_rengi(
            &seçenek.kontrol_noktası_stili.öğe_stili,
            tema::aksan_50(),
        ))),
        None,
    );
    let renk = kontrol.renk.unwrap_or_else(tema::aksan_50);
    if let Some(x) = oynat {
        let merkez = (kutu.x + x + boyut / 2.0, eksen_y);
        oynat_düğmesi(yüzey, merkez, boyut, oynuyor, renk);
        isabetler.push((
            Dikdörtgen::yeni(merkez.0 - boyut / 2.0, merkez.1 - boyut / 2.0, boyut, boyut),
            ZamanŞeridiEylemi::OynatDurdur,
        ));
    }
    if let Some(x) = önceki {
        let merkez = (kutu.x + x + boyut * 0.375, eksen_y);
        ok_düğmesi(yüzey, merkez, boyut * 0.75, (-1.0, 0.0), renk);
        isabetler.push((
            Dikdörtgen::yeni(merkez.0 - boyut / 2.0, merkez.1 - boyut / 2.0, boyut, boyut),
            ZamanŞeridiEylemi::Kare(sonraki_sıra(seçenek, geçerli, toplam, false)),
        ));
    }
    if let Some(x) = sonraki {
        let merkez = (kutu.x + x + boyut * 0.375, eksen_y);
        ok_düğmesi(yüzey, merkez, boyut * 0.75, (1.0, 0.0), renk);
        isabetler.push((
            Dikdörtgen::yeni(merkez.0 - boyut / 2.0, merkez.1 - boyut / 2.0, boyut, boyut),
            ZamanŞeridiEylemi::Kare(sonraki_sıra(seçenek, geçerli, toplam, true)),
        ));
    }
    isabetler
}

/// Option modelinden slider timeline çizer. `durum`, görünümün oynatma
/// durumunu modelin `currentIndex/autoPlay` değerlerinin üstüne bindirir.
pub fn seçenekli_zaman_şeridi_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenek: &ZamanŞeridi,
    durum: Option<(usize, usize, bool)>,
) -> Vec<(Dikdörtgen, ZamanŞeridiEylemi)> {
    if !seçenek.göster {
        return Vec::new();
    }
    let model_toplamı = seçenek.veri.len();
    let (mut geçerli, toplam, oynuyor) =
        durum.unwrap_or((seçenek.geçerli_sıra, model_toplamı, seçenek.otomatik_oynat));
    let toplam = if toplam == 0 { model_toplamı } else { toplam };
    if toplam == 0 {
        return Vec::new();
    }
    geçerli = if seçenek.döngü {
        geçerli % toplam
    } else {
        geçerli.min(toplam - 1)
    };
    let kutu = seçenek_kutusu(yüzey, seçenek);
    if let Some(arkaplan) = &seçenek.arkaplan {
        let dış = Dikdörtgen::yeni(
            kutu.x - seçenek.iç_boşluk[3],
            kutu.y - seçenek.iç_boşluk[0],
            kutu.genişlik + seçenek.iç_boşluk[1] + seçenek.iç_boşluk[3],
            kutu.yükseklik + seçenek.iç_boşluk[0] + seçenek.iç_boşluk[2],
        );
        yüzey.dikdörtgen(
            dış,
            arkaplan,
            [0.0; 4],
            seçenek
                .kenarlık_rengi
                .map(|renk| (seçenek.kenarlık_kalınlığı, renk)),
        );
    }
    match seçenek.yön {
        Yön::Dikey => dikey_şerit(yüzey, seçenek, kutu, geçerli, toplam, oynuyor),
        Yön::Yatay => yatay_şerit(yüzey, seçenek, kutu, geçerli, toplam, oynuyor),
    }
}
