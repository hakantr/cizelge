//! İpucu (tooltip) penceresi — `echarts/src/component/tooltip` karşılığı.

use crate::cizim::{DikeyHiza, SATIR_ORANI, YatayHiza, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::bilesen::İpucu;
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
    let kutu_genişliği =
        içerik_genişliği + İÇ_BOŞLUK * 2.0 + if çoklu_eksen_grubu { 1.3 } else { 0.0 };
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
    } else {
        başlık_yüksekliği + satırlar.len() as f32 * satır_yüksekliği + İÇ_BOŞLUK * 2.0
    };

    // Konumlandırma: sağ alta; taşarsa çevir, tuvale kıstır.
    let (imleç_kaçığı_x, imleç_kaçığı_y) = if çoklu_eksen_grubu {
        (20.25, 20.5)
    } else {
        (İMLEÇ_KAÇIĞI, İMLEÇ_KAÇIĞI)
    };
    let mut x = konum.0 + imleç_kaçığı_x;
    let mut y = konum.1 + imleç_kaçığı_y;
    if x + kutu_genişliği > çizici.genişlik() {
        x = konum.0 - imleç_kaçığı_x - kutu_genişliği;
    }
    if y + kutu_yüksekliği > çizici.yükseklik() {
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
    çizici.gölge(kutu, 4.0, gölge, 10.0);
    let arkaplan = seçenek.arkaplan.unwrap_or(tema::ipucu_arkaplanı());
    çizici.dikdörtgen(
        kutu,
        &Dolgu::Düz(arkaplan),
        [4.0; 4],
        Some((1.0, tema::ipucu_kenarlığı())),
    );

    let metin_rengi = seçenek.yazı.renk.unwrap_or(tema::ipucu_metni());
    let mut satır_y = if çoklu_eksen_grubu {
        y + 19.0
    } else {
        y + İÇ_BOŞLUK + satır_yüksekliği / 2.0
    };

    if let Some(b) = başlık {
        çizici.yazı(
            b,
            (x + İÇ_BOŞLUK, satır_y),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            boyut,
            metin_rengi,
            false,
        );
        if !çoklu_eksen_grubu {
            satır_y += satır_yüksekliği;
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
        let mut metin_x = x + İÇ_BOŞLUK;
        if let Some(renk) = satır.im_rengi {
            çizici.daire(
                (metin_x + İM_ÇAPI / 2.0, satır_y),
                İM_ÇAPI / 2.0,
                Some(&Dolgu::Düz(renk)),
                None,
            );
            metin_x += İM_ÇAPI + 6.0;
        }
        çizici.yazı(
            &satır.ad,
            (metin_x, satır_y),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            boyut,
            metin_rengi,
            false,
        );
        // Değer sağa hizalı ve kalın (ECharts görünümü).
        if !satır.değer.is_empty() {
            let değer_sağı =
                x + kutu_genişliği - İÇ_BOŞLUK - if çoklu_eksen_grubu { 1.0 } else { 0.0 };
            çizici.yazı(
                &satır.değer,
                (değer_sağı, satır_y),
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
