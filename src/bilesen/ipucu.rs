//! İpucu (tooltip) penceresi — `echarts/src/component/tooltip` karşılığı.

use crate::cizim::{DikeyHiza, SATIR_ORANI, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::bilesen::{Tetikleme, İpucu, İpucuKonumu};
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};
use crate::tema;

/// İpucundaki bir satır: renkli im + ad + değer.
#[derive(Clone, Debug)]
pub struct İpucuSatırı {
    pub im_rengi: Option<Renk>,
    pub ad: String,
    pub değer: String,
}

const İÇ_BOŞLUK: f32 = 10.0;
const İM_ÇAPI: f32 = 10.0;
const SÜTUN_ARASI: f32 = 20.0;
const İMLEÇ_KAÇIĞI: f32 = 21.2;

/// İpucu penceresini çizer. `konum` grafik yerel fare noktasıdır; pencere
/// tuval sınırları içinde kalacak biçimde konumlanır.
pub fn ipucu_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seçenek: &İpucu,
    konum: (f32, f32),
    başlık: Option<&str>,
    satırlar: &[İpucuSatırı],
) {
    if satırlar.is_empty() && başlık.is_none() {
        return;
    }
    let boyut = seçenek.yazı.boyut.unwrap_or(tema::YAZI_ORTA);
    let satır_yüksekliği = boyut * SATIR_ORANI + 2.0;
    let grup_başlığı_mı = |satır: &İpucuSatırı| satır.im_rengi.is_none() && satır.değer.is_empty();
    let çoklu_eksen_grubu = başlık.is_some()
        && satırlar
            .iter()
            .enumerate()
            .any(|(sıra, satır)| sıra > 0 && grup_başlığı_mı(satır));
    let tek_html_grubu = başlık.is_some() && !çoklu_eksen_grubu;

    // Ölçüm.
    let başlık_genişliği = başlık.map(|b| çizici.yazı_ölç(b, boyut).0).unwrap_or(0.0);
    let mut içerik_genişliği = başlık_genişliği;
    for satır in satırlar {
        let im = if satır.im_rengi.is_some() {
            İM_ÇAPI + 6.0
        } else {
            0.0
        };
        let değer_genişliği = if satır.değer.is_empty() {
            0.0
        } else {
            SÜTUN_ARASI + çizici.yazı_ölç(&satır.değer, boyut).0
        };
        let genişlik = im + çizici.yazı_ölç(&satır.ad, boyut).0 + değer_genişliği;
        içerik_genişliği = içerik_genişliği.max(genişlik);
    }
    let kutu_genişliği = içerik_genişliği
        + İÇ_BOŞLUK * 2.0
        + if tek_html_grubu {
            // HTML tooltip'in `box-sizing: content-box` kutusunda iki adet
            // 1 px kenarlık, içeriğin ve 10 px dolguların dışındadır.
            2.0
        } else if çoklu_eksen_grubu {
            1.3
        } else {
            0.0
        };
    let başlık_yüksekliği = if başlık.is_some() {
        satır_yüksekliği
    } else {
        0.0
    };
    let kutu_yüksekliği = if çoklu_eksen_grubu {
        // ECharts HTML tooltip markup'ı: 14 px başlık/satır, başlık ile ilk
        // seri arasında 10 px, eksen grupları arasında 20 px. Merkezler ilk
        // başlıktan itibaren +24 / +34 piksel ilerler; dış kutu 10 px
        // padding ve 1 px kenarlık taşır.
        let mut son_merkez = 18.0;
        for satır in satırlar {
            son_merkez += if grup_başlığı_mı(satır) {
                34.0
            } else {
                24.0
            };
        }
        son_merkez + 17.0
    } else if tek_html_grubu {
        // 14 px başlık, her satırdan önce 10 px boşluk, 14 px
        // satır ve iki yanda 10 px dolgu + 1 px kenarlık.
        36.0 + satırlar.len() as f32 * 24.0
    } else {
        başlık_yüksekliği + satırlar.len() as f32 * satır_yüksekliği + İÇ_BOŞLUK * 2.0
    };

    // Konumlandırma: sağ alta; taşarsa çevir, tuvale kıstır.
    let (imleç_kaçığı_x, imleç_kaçığı_y) = if çoklu_eksen_grubu {
        (20.25, 20.5)
    } else {
        (İMLEÇ_KAÇIĞI, İMLEÇ_KAÇIĞI)
    };
    let üstte = seçenek.konum == İpucuKonumu::Üst;
    let (mut x, mut y) = if üstte {
        // TooltipHTMLContent CSS dönüşümü konumu tam piksele indirger.
        // Ok da veri noktasına değil bu yerleştirilmiş kutunun merkezine
        // bağlanır.
        (
            (konum.0 - kutu_genişliği / 2.0).floor(),
            (konum.1 - kutu_yüksekliği - 10.0).floor(),
        )
    } else {
        (konum.0 + imleç_kaçığı_x, konum.1 + imleç_kaçığı_y)
    };
    if !üstte && x + kutu_genişliği > çizici.genişlik() {
        x = konum.0 - imleç_kaçığı_x - kutu_genişliği;
    }
    if !üstte && y + kutu_yüksekliği > çizici.yükseklik() {
        y = konum.1 - imleç_kaçığı_y - kutu_yüksekliği;
    }
    x = x.clamp(0.0, (çizici.genişlik() - kutu_genişliği).max(0.0));
    y = y.clamp(0.0, (çizici.yükseklik() - kutu_yüksekliği).max(0.0));

    let kutu = Dikdörtgen::yeni(x, y, kutu_genişliği, kutu_yüksekliği);

    // Kutu: gölge + arka plan + kenarlık.
    let gölge = if çoklu_eksen_grubu {
        tema::ipucu_gölgesi().opaklık(0.6)
    } else {
        tema::ipucu_gölgesi()
    };
    çizici.gölge(kutu, 4.0, gölge, if üstte { 8.0 } else { 10.0 });
    let arkaplan = seçenek.arkaplan.unwrap_or(tema::ipucu_arkaplanı());
    // TooltipHTMLContent, kutunun ve okun kenarlığını en yakın
    // veri öğesinin görsel rengiyle çözer (`nearPointColor`).
    let kenarlık_rengi = (seçenek.tetikleme == Tetikleme::Öğe)
        .then(|| satırlar.iter().find_map(|satır| satır.im_rengi))
        .flatten()
        .unwrap_or_else(tema::ipucu_kenarlığı);
    if üstte {
        // TooltipHTMLContent'in aşağı bakan 10×10 px döndürülmüş oku.
        // Üst yarısı kutunun altında kaldığı için kutu daha sonra çizilir.
        let merkez_x = (x + kutu_genişliği / 2.0).clamp(x + 8.0, x + kutu_genişliği - 8.0);
        let merkez_y = kutu.alt() - 0.25;
        let mut ok = Yol::yeni();
        ok.taşı((merkez_x, merkez_y - 5.0));
        ok.çiz((merkez_x + 5.0, merkez_y));
        ok.çiz((merkez_x, merkez_y + 5.0));
        ok.çiz((merkez_x - 5.0, merkez_y));
        ok.kapat();
        çizici.yol_doldur(&ok, &Dolgu::Düz(arkaplan));
        çizici.yol_çiz(&ok, 1.0, kenarlık_rengi, ÇizgiTürü::Düz);
    }
    // CSS 1 px kenarlığı kutunun içine yerleşir ve tam piksel
    // merkezlerinden geçer. Yüzeyin merkezli vuruşunu yarım piksel içe
    // almak, kenarın %50 alfa ile incelmesini önler.
    let kutu_yolu = Dikdörtgen::yeni(
        kutu.x + 0.5,
        kutu.y + 0.5,
        (kutu.genişlik - 1.0).max(0.1),
        (kutu.yükseklik - 1.0).max(0.1),
    );
    çizici.dikdörtgen(
        kutu_yolu,
        &Dolgu::Düz(arkaplan),
        [3.5; 4],
        Some((1.0, kenarlık_rengi)),
    );

    let metin_rengi = seçenek.yazı.renk.unwrap_or(tema::ipucu_metni());
    let yatay_iç_başlangıç = x + İÇ_BOŞLUK + if tek_html_grubu { 1.0 } else { 0.0 };
    let mut satır_y = if çoklu_eksen_grubu {
        y + 19.0
    } else if tek_html_grubu {
        y + 18.0
    } else {
        y + İÇ_BOŞLUK + satır_yüksekliği / 2.0
    };

    if let Some(b) = başlık {
        çizici.yazı(
            b,
            (
                yatay_iç_başlangıç,
                satır_y + if tek_html_grubu { 1.0 } else { 0.0 },
            ),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            boyut,
            metin_rengi,
            false,
        );
        if !çoklu_eksen_grubu {
            satır_y += if tek_html_grubu {
                24.0
            } else {
                satır_yüksekliği
            };
        }
    }

    for satır in satırlar {
        if çoklu_eksen_grubu {
            satır_y += if grup_başlığı_mı(satır) {
                34.0
            } else {
                24.0
            };
        }
        let mut metin_x = yatay_iç_başlangıç;
        if let Some(renk) = satır.im_rengi {
            çizici.daire(
                (metin_x + İM_ÇAPI / 2.0, satır_y),
                if tek_html_grubu {
                    (İM_ÇAPI - 1.0) / 2.0
                } else {
                    İM_ÇAPI / 2.0
                },
                Some(&Dolgu::Düz(renk)),
                None,
            );
            metin_x += İM_ÇAPI + 6.0;
        }
        çizici.yazı(
            &satır.ad,
            (metin_x, satır_y + if tek_html_grubu { 1.0 } else { 0.0 }),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            boyut,
            metin_rengi,
            false,
        );
        // Değer sağa hizalı ve kalın (ECharts görünümü).
        if !satır.değer.is_empty() {
            let değer_sağı = x + kutu_genişliği
                - İÇ_BOŞLUK
                - if çoklu_eksen_grubu || tek_html_grubu {
                    1.0
                } else {
                    0.0
                };
            çizici.yazı(
                &satır.değer,
                (değer_sağı, satır_y + if tek_html_grubu { 1.0 } else { 0.0 }),
                YatayHiza::Sağ,
                DikeyHiza::Orta,
                boyut,
                metin_rengi,
                true,
            );
        }
        if !çoklu_eksen_grubu {
            satır_y += satır_yüksekliği;
        }
    }
}
