//! Gösterge (legend) bileşeni — `echarts/src/component/legend` karşılığı.
//!
//! Öğeler yatay tek satır ya da dikey sütun olarak yerleştirilir; tıklama
//! isabet kutuları çağırana bildirilir.

use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::YatayKonum;
use crate::model::bilesen::{Gösterge, GöstergeSimgesi, Yön};
use crate::renk::{Dolgu, Renk};
use crate::tema;

/// Göstergedeki tek öğe.
#[derive(Clone, Debug)]
pub struct GöstergeÖğesi {
    pub ad: String,
    pub renk: Renk,
    pub simge: GöstergeSimgesi,
    /// Çizgi serisinin `lineStyle.width` değeri. `None`, öntanımlı 2 px'dir;
    /// sıfır olduğunda legend simgesinde yalnız seri sembolü kalır.
    pub çizgi_kalınlığı: Option<f32>,
    /// Seri/veri `itemStyle`ından miras alınan legend simgesi kenarlığı.
    pub kenarlık: Option<(f32, Renk)>,
    pub kapalı: bool,
}

const SİMGE_METİN_ARASI: f32 = 5.0;

/// Gösterge çiziminin çıktısı.
#[derive(Default)]
pub struct GöstergeÇıktısı {
    /// Her öğenin `(isabet kutusu, ad)` çifti (grafik yerel).
    pub kutular: Vec<(Dikdörtgen, String)>,
    /// Kaydırma okları: `(kutu, yön)` — yön -1 önceki, +1 sonraki sayfa.
    pub oklar: Vec<(Dikdörtgen, i32)>,
    /// Toplam sayfa sayısı (kaydırmalı değilse 1).
    pub sayfa_sayısı: usize,
}

/// Göstergeyi çizer; kaydırılabilir kipte yalnız geçerli sayfanın
/// öğelerini ve `‹ n/m ›` denetimlerini boyar.
pub fn gösterge_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seçenek: &Gösterge,
    öğeler: &[GöstergeÖğesi],
    sayfa: usize,
) -> GöstergeÇıktısı {
    if !seçenek.göster || öğeler.is_empty() {
        return GöstergeÇıktısı::default();
    }
    let boyut = seçenek.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    // Çizgi simgesinin 11.2px dairesi, 2px vuruşla 13.2px sınır kutusu
    // üretir; kare/pasta simgesi ise tam `itemHeight` (14px) kullanır.
    // Legend box yerleşimi görünen simge sınırlarının birleşimine dayanır.
    let yalnız_çizgi = öğeler
        .iter()
        .all(|öğe| seçenek.simge.unwrap_or(öğe.simge) == GöstergeSimgesi::Çizgi);
    let satır_yüksekliği = if yalnız_çizgi {
        (seçenek.simge_yüksekliği * 0.8 + 2.0).max(boyut)
    } else {
        seçenek.simge_yüksekliği.max(boyut)
    };
    let görünen_adlar: Vec<String> = öğeler
        .iter()
        .map(|öğe| {
            seçenek
                .biçimleyici
                .as_ref()
                .map(|biçimleyici| biçimleyici.uygula(f64::NAN, &öğe.ad))
                .unwrap_or_else(|| öğe.ad.clone())
        })
        .collect();

    // Öğe genişliklerini ölç.
    let genişlikler: Vec<f32> = öğeler
        .iter()
        .zip(&görünen_adlar)
        .map(|(öğe, görünen_ad)| {
            let simge = seçenek.simge.unwrap_or(öğe.simge);
            let öğe_kenar_taşması = öğe
                .kenarlık
                .map(|(kalınlık, _)| kalınlık / 2.0)
                .unwrap_or(0.0);
            // Çizgi simgesinin 2px vuruşu yerel kutunun solundan 1px
            // taşar. Legend yerleşimi zrender sınır kutusunu kullandığı için
            // bu taşma sonraki öğenin başlangıcına da katılır.
            let çizgi_taşması = if simge == GöstergeSimgesi::Çizgi {
                öğe.çizgi_kalınlığı.unwrap_or(2.0).max(0.0) / 2.0
            } else {
                0.0
            };
            // `symbolKeepAspect`: daire, 25×14 item kutusunda 14×14 çizilir.
            // boxLayout görünen sınırı kullandığından soldaki 5.5 px iç
            // boşluk toplam öğe genişliğine katılmaz.
            let daire_iç_boşluğu = if simge == GöstergeSimgesi::Daire {
                (seçenek.simge_genişliği - seçenek.simge_yüksekliği).max(0.0) / 2.0
            } else {
                0.0
            };
            let kenar_taşması = öğe_kenar_taşması.max(çizgi_taşması);
            seçenek.simge_genişliği
                + SİMGE_METİN_ARASI
                + çizici.yazı_ölç(görünen_ad, boyut).0
                + kenar_taşması
                - daire_iç_boşluğu
        })
        .collect();

    // Plain yatay legend, zrender `boxLayout` gibi kullanılabilir genişlikte
    // otomatik satır kırar. Bütün satırlar en geniş satırın sol kenarını
    // paylaşır; kısa son satır ayrıca ortalanmaz.
    let yatay_satırlar: Vec<(usize, usize, f32)> =
        if seçenek.yön == Yön::Yatay && !seçenek.kaydırılabilir {
            let kullanılabilir = (çizici.genişlik() - seçenek.iç_boşluk * 2.0).max(1.0);
            let mut satırlar = Vec::new();
            let mut başlangıç = 0usize;
            let mut genişlik = 0.0f32;
            for (sıra, öğe_genişliği) in genişlikler.iter().copied().enumerate() {
                let ek = if sıra == başlangıç {
                    öğe_genişliği
                } else {
                    seçenek.öğe_boşluğu + öğe_genişliği
                };
                if sıra > başlangıç && genişlik + ek > kullanılabilir {
                    satırlar.push((başlangıç, sıra, genişlik));
                    başlangıç = sıra;
                    genişlik = öğe_genişliği;
                } else {
                    genişlik += ek;
                }
            }
            satırlar.push((başlangıç, genişlikler.len(), genişlik));
            satırlar
        } else {
            Vec::new()
        };
    let içerik_yüksekliği = match seçenek.yön {
        Yön::Yatay => {
            let satır_sayısı = yatay_satırlar.len().max(1);
            satır_yüksekliği * satır_sayısı as f32
                + seçenek.öğe_boşluğu * satır_sayısı.saturating_sub(1) as f32
        }
        Yön::Dikey => {
            satır_yüksekliği * öğeler.len() as f32
                + seçenek.öğe_boşluğu * öğeler.len().saturating_sub(1) as f32
        }
    };
    let üst = seçenek
        .üst
        .map(|u| u.çöz(çizici.yükseklik()) + seçenek.iç_boşluk)
        .or_else(|| {
            seçenek.alt.map(|alt| {
                çizici.yükseklik()
                    - alt.çöz(çizici.yükseklik())
                    - seçenek.iç_boşluk
                    - içerik_yüksekliği
            })
        })
        .unwrap_or(seçenek.iç_boşluk);

    // Kaydırmalı yatay gösterge: sayfaya böl.
    if seçenek.kaydırılabilir && seçenek.yön == Yön::Yatay {
        let denetim_genişliği = 96.0;
        let kullanılabilir =
            (çizici.genişlik() - denetim_genişliği - seçenek.iç_boşluk * 2.0).max(60.0);
        // Sayfaları doldur.
        let mut sayfalar: Vec<(usize, usize)> = Vec::new(); // (başlangıç, uzunluk)
        let mut başlangıç = 0usize;
        while başlangıç < öğeler.len() {
            let mut genişlik_toplam = 0.0f32;
            let mut uzunluk = 0usize;
            for g in genişlikler.iter().skip(başlangıç) {
                let ek = if uzunluk == 0 {
                    *g
                } else {
                    seçenek.öğe_boşluğu + *g
                };
                if genişlik_toplam + ek > kullanılabilir && uzunluk > 0 {
                    break;
                }
                genişlik_toplam += ek;
                uzunluk += 1;
            }
            uzunluk = uzunluk.max(1);
            sayfalar.push((başlangıç, uzunluk));
            başlangıç += uzunluk;
        }
        let sayfa_sayısı = sayfalar.len().max(1);
        let sayfa = sayfa % sayfa_sayısı;
        let (s_baş, s_uzunluk) = sayfalar.get(sayfa).copied().unwrap_or((0, öğeler.len()));

        let mut kutular = Vec::new();
        let mut x = seçenek.iç_boşluk;
        for ((öğe, görünen_ad), genişlik) in öğeler
            .iter()
            .zip(&görünen_adlar)
            .zip(&genişlikler)
            .skip(s_baş)
            .take(s_uzunluk)
        {
            let kutu = Dikdörtgen::yeni(x, üst, *genişlik, satır_yüksekliği);
            öğe_çiz(çizici, seçenek, öğe, görünen_ad, kutu, boyut);
            kutular.push((kutu, öğe.ad.clone()));
            x += genişlik + seçenek.öğe_boşluğu;
        }

        // Denetimler: ‹ n/m ›
        let mut oklar = Vec::new();
        let denetim_x = çizici.genişlik() - denetim_genişliği - seçenek.iç_boşluk;
        let orta_y = üst + satır_yüksekliği / 2.0;
        let sol_ok = Dikdörtgen::yeni(denetim_x, üst, 18.0, satır_yüksekliği);
        let sağ_ok = Dikdörtgen::yeni(denetim_x + 70.0, üst, 18.0, satır_yüksekliği);
        for (kutu, işaret, yön) in [(sol_ok, "‹", -1i32), (sağ_ok, "›", 1i32)] {
            çizici.dikdörtgen(kutu, &Dolgu::Düz(tema::nötr_10()), [3.0; 4], None);
            çizici.yazı(
                işaret,
                kutu.merkez(),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                boyut,
                tema::ikincil_metin(),
                true,
            );
            oklar.push((kutu, yön));
        }
        çizici.yazı(
            &format!("{}/{}", sayfa + 1, sayfa_sayısı),
            (denetim_x + 44.0, orta_y),
            YatayHiza::Orta,
            DikeyHiza::Orta,
            boyut,
            tema::ikincil_metin(),
            false,
        );
        return GöstergeÇıktısı {
            kutular,
            oklar,
            sayfa_sayısı,
        };
    }

    // Kaydırmalı dikey gösterge. ECharts `ScrollableLegendView`, içerik
    // penceresinin altına yatay bir `▲ 1/n ▼` denetimi koyar; son görünür
    // öğeyi sonraki sayfanın ilk öğesi olarak tekrarlar.
    if seçenek.kaydırılabilir && seçenek.yön == Yön::Dikey {
        let en_geniş = genişlikler.iter().copied().fold(0.0_f32, f32::max);
        let x = yatay_başlangıç(seçenek, çizici.genişlik(), en_geniş);
        let üst_kenar = seçenek
            .üst
            .map(|değer| değer.çöz(çizici.yükseklik()))
            .unwrap_or(0.0);
        let alt_kenar = seçenek
            .alt
            .filter(|_| seçenek.üst.is_none())
            .map(|değer| değer.çöz(çizici.yükseklik()))
            .unwrap_or(0.0);
        let kullanılabilir_yükseklik =
            (çizici.yükseklik() - üst_kenar - alt_kenar - seçenek.iç_boşluk * 2.0)
                .max(satır_yüksekliği);
        let denetim_yüksekliği = 15.0_f32;
        let içerik_yüksekliği =
            (kullanılabilir_yükseklik - denetim_yüksekliği - seçenek.öğe_boşluğu)
                .max(satır_yüksekliği);
        let adım = satır_yüksekliği + seçenek.öğe_boşluğu;
        let sayfa_adımı = ((içerik_yüksekliği / adım).floor() as usize).max(1);
        let sayfa_sayısı = (öğeler.len().saturating_sub(1) / sayfa_adımı) + 1;
        let sayfa = sayfa.min(sayfa_sayısı.saturating_sub(1));
        let başlangıç = sayfa.saturating_mul(sayfa_adımı).min(öğeler.len());
        // Sağ sınırdaki öğe ECharts'ta kırpılmış olarak görünür ve sonraki
        // sayfada yeniden başlar; bu nedenle tam sayfa adımına bir eklenir.
        let gösterilecek = sayfa_adımı.saturating_add(1);

        let mut kutular = Vec::new();
        let kırpma = Dikdörtgen::yeni(x, üst, en_geniş, içerik_yüksekliği);
        çizici.kırpılı(kırpma, &mut |yüzey| {
            for (yerel_sıra, ((öğe, görünen_ad), genişlik)) in öğeler
                .iter()
                .zip(&görünen_adlar)
                .zip(&genişlikler)
                .skip(başlangıç)
                .take(gösterilecek)
                .enumerate()
            {
                let y = üst + yerel_sıra as f32 * adım;
                let kutu = Dikdörtgen::yeni(x, y, *genişlik, satır_yüksekliği);
                öğe_çiz(yüzey, seçenek, öğe, görünen_ad, kutu, boyut);
                kutular.push((kutu, öğe.ad.clone()));
            }
        });

        // `pageText` önce `xx/xx` yer tutucusuyla düzenlenir; gerçek metin
        // daha kısa olsa da okların konumu değişmez.
        let simge = 15.0_f32;
        let denetim_öğe_boşluğu = 5.0_f32;
        let yer_tutucu = çizici.yazı_ölç("xx/xx", boyut).0;
        let denetim_genişliği = simge * 2.0 + denetim_öğe_boşluğu * 2.0 + yer_tutucu;
        let denetim_sol = x + (en_geniş - denetim_genişliği) / 2.0;
        let denetim_y = üst + kullanılabilir_yükseklik - simge / 2.0;
        let önceki_merkez = (denetim_sol + simge / 2.0, denetim_y);
        let metin_merkez = (
            denetim_sol + simge + denetim_öğe_boşluğu + yer_tutucu / 2.0,
            denetim_y,
        );
        let sonraki_merkez = (
            denetim_sol
                + simge
                + denetim_öğe_boşluğu
                + yer_tutucu
                + denetim_öğe_boşluğu
                + simge / 2.0,
            denetim_y,
        );
        let önceki_var = sayfa > 0;
        let sonraki_var = sayfa + 1 < sayfa_sayısı;
        sayfa_üçgeni_çiz(çizici, önceki_merkez, false, önceki_var);
        sayfa_üçgeni_çiz(çizici, sonraki_merkez, true, sonraki_var);
        çizici.yazı(
            &format!("{}/{}", sayfa + 1, sayfa_sayısı),
            metin_merkez,
            YatayHiza::Orta,
            DikeyHiza::Orta,
            boyut,
            tema::üçüncül_metin(),
            false,
        );

        let mut oklar = Vec::new();
        if önceki_var {
            oklar.push((
                Dikdörtgen::yeni(
                    önceki_merkez.0 - simge / 2.0,
                    önceki_merkez.1 - simge / 2.0,
                    simge,
                    simge,
                ),
                -1,
            ));
        }
        if sonraki_var {
            oklar.push((
                Dikdörtgen::yeni(
                    sonraki_merkez.0 - simge / 2.0,
                    sonraki_merkez.1 - simge / 2.0,
                    simge,
                    simge,
                ),
                1,
            ));
        }
        return GöstergeÇıktısı {
            kutular,
            oklar,
            sayfa_sayısı,
        };
    }

    let mut kutular = Vec::with_capacity(öğeler.len());
    match seçenek.yön {
        Yön::Yatay => {
            let toplam = yatay_satırlar
                .iter()
                .map(|(_, _, genişlik)| *genişlik)
                .fold(0.0f32, f32::max);
            let başlangıç_x = yatay_başlangıç(seçenek, çizici.genişlik(), toplam);
            for (satır_sırası, (başlangıç, bitiş, _)) in yatay_satırlar.iter().enumerate() {
                let mut x = başlangıç_x;
                let y = üst + satır_sırası as f32 * (satır_yüksekliği + seçenek.öğe_boşluğu);
                for sıra in *başlangıç..*bitiş {
                    let (Some(öğe), Some(görünen_ad), Some(genişlik)) = (
                        öğeler.get(sıra),
                        görünen_adlar.get(sıra),
                        genişlikler.get(sıra),
                    ) else {
                        continue;
                    };
                    let kutu = Dikdörtgen::yeni(x, y, *genişlik, satır_yüksekliği);
                    öğe_çiz(çizici, seçenek, öğe, görünen_ad, kutu, boyut);
                    kutular.push((kutu, öğe.ad.clone()));
                    x += genişlik + seçenek.öğe_boşluğu;
                }
            }
        }
        Yön::Dikey => {
            let en_geniş = genişlikler.iter().copied().fold(0.0f32, f32::max);
            let x = yatay_başlangıç(seçenek, çizici.genişlik(), en_geniş);
            let mut y = üst;
            for ((öğe, görünen_ad), genişlik) in öğeler.iter().zip(&görünen_adlar).zip(&genişlikler)
            {
                let kutu = Dikdörtgen::yeni(x, y, *genişlik, satır_yüksekliği);
                öğe_çiz(çizici, seçenek, öğe, görünen_ad, kutu, boyut);
                kutular.push((kutu, öğe.ad.clone()));
                y += satır_yüksekliği + seçenek.öğe_boşluğu;
            }
        }
    }
    GöstergeÇıktısı {
        kutular,
        oklar: Vec::new(),
        sayfa_sayısı: 1,
    }
}

fn yatay_başlangıç(
    seçenek: &Gösterge, yüzey_genişliği: f32, içerik_genişliği: f32
) -> f32 {
    if let Some(sağ) = seçenek.sağ {
        return yüzey_genişliği
            - sağ.çöz(yüzey_genişliği)
            - seçenek.iç_boşluk
            - içerik_genişliği;
    }
    match seçenek.sol {
        YatayKonum::Sol => seçenek.iç_boşluk,
        YatayKonum::Orta => (yüzey_genişliği - içerik_genişliği) / 2.0,
        YatayKonum::Sağ => yüzey_genişliği - içerik_genişliği - seçenek.iç_boşluk,
        YatayKonum::Değer(u) => u.çöz(yüzey_genişliği) + seçenek.iç_boşluk,
    }
}

fn sayfa_üçgeni_çiz(
    çizici: &mut dyn ÇizimYüzeyi, merkez: (f32, f32), aşağı: bool, etkin: bool
) {
    let yarı = 7.5_f32;
    let mut yol = Yol::yeni();
    if aşağı {
        yol.taşı((merkez.0 - yarı, merkez.1 - yarı));
        yol.çiz((merkez.0 + yarı, merkez.1 - yarı));
        yol.çiz((merkez.0, merkez.1 + yarı));
    } else {
        yol.taşı((merkez.0 - yarı, merkez.1 + yarı));
        yol.çiz((merkez.0 + yarı, merkez.1 + yarı));
        yol.çiz((merkez.0, merkez.1 - yarı));
    }
    yol.kapat();
    let renk = if etkin {
        Renk::onaltılık(0x6578ba)
    } else {
        Renk::onaltılık(0xe0e4f2)
    };
    çizici.yol_doldur(&yol, &Dolgu::Düz(renk));
}

fn öğe_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seçenek: &Gösterge,
    öğe: &GöstergeÖğesi,
    görünen_ad: &str,
    kutu: Dikdörtgen,
    yazı_boyutu: f32,
) {
    let renk = if öğe.kapalı {
        seçenek.devre_dışı_rengi
    } else {
        öğe.renk
    };
    let simge = seçenek.simge.unwrap_or(öğe.simge);
    // `kutu` zrender grubunun görünen sınırıdır. Çizgi vuruşu ve seri
    // itemStyle kenarlığı gerçek item koordinatının soluna taşabildiği için
    // simge/metin çapası bu taşma kadar içeri alınır.
    let kenar_taşması = öğe
        .kenarlık
        .map(|(kalınlık, _)| kalınlık / 2.0)
        .unwrap_or(0.0);
    let yatay_taşma = kenar_taşması.max(if simge == GöstergeSimgesi::Çizgi {
        öğe.çizgi_kalınlığı.unwrap_or(2.0).max(0.0) / 2.0
    } else {
        0.0
    });
    let içerik_x = kutu.x + yatay_taşma;
    let daire_iç_boşluğu = if simge == GöstergeSimgesi::Daire {
        (seçenek.simge_genişliği - seçenek.simge_yüksekliği).max(0.0) / 2.0
    } else {
        0.0
    };
    // Dairede `kutu.x`, kırpılmış sembolün görünen soludur; zrender öğe
    // koordinatını görünmeyen keep-aspect payı kadar sola taşır.
    let simge_x = içerik_x - daire_iç_boşluğu;
    // Line legend'in sınır kutusu yerel y=0.4'ten başlasa da sembol ve
    // metin tabanının ortak merkezi item koordinatında daima y=7'dir.
    let orta_y = kutu.y + kenar_taşması + seçenek.simge_yüksekliği / 2.0;

    match simge {
        GöstergeSimgesi::YuvarlakKöşeliKare => {
            let d = Dikdörtgen::yeni(
                içerik_x,
                orta_y - seçenek.simge_yüksekliği / 2.0,
                seçenek.simge_genişliği,
                seçenek.simge_yüksekliği,
            );
            çizici.dikdörtgen(d, &Dolgu::Düz(renk), [3.0; 4], öğe.kenarlık);
        }
        GöstergeSimgesi::Daire => {
            let yarıçap = seçenek.simge_yüksekliği / 2.0;
            çizici.daire(
                (simge_x + seçenek.simge_genişliği / 2.0, orta_y),
                yarıçap,
                Some(&Dolgu::Düz(renk)),
                None,
            );
        }
        GöstergeSimgesi::Çizgi => {
            // ECharts çizgi serisi simgesi: yatay çizgi + ortada içi boş nokta.
            let çizgi_kalınlığı = öğe.çizgi_kalınlığı.unwrap_or(2.0);
            if çizgi_kalınlığı > 0.0 {
                çizici.çizgi(
                    (içerik_x, orta_y),
                    (içerik_x + seçenek.simge_genişliği, orta_y),
                    çizgi_kalınlığı,
                    renk,
                    crate::model::stil::ÇizgiTürü::Düz,
                );
            }
            çizici.daire(
                (içerik_x + seçenek.simge_genişliği / 2.0, orta_y),
                // LegendView, seri sembolünü `itemHeight * 0.8`
                // boyutunda ölçekler (14px öntanımlıda çap 11.2px).
                seçenek.simge_yüksekliği * 0.4,
                Some(&Dolgu::Düz(Renk::BEYAZ)),
                Some((2.0, renk)),
            );
        }
    }

    let yazı_rengi = if öğe.kapalı {
        seçenek.devre_dışı_rengi
    } else {
        seçenek.yazı.renk.unwrap_or(tema::ikincil_metin())
    };
    çizici.yazı(
        görünen_ad,
        (
            simge_x + seçenek.simge_genişliği + SİMGE_METİN_ARASI,
            orta_y,
        ),
        YatayHiza::Sol,
        DikeyHiza::Orta,
        yazı_boyutu,
        yazı_rengi,
        false,
    );
}
