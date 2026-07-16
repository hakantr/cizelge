//! Gösterge (legend) bileşeni — `echarts/src/component/legend` karşılığı.
//!
//! Öğeler yatay tek satır ya da dikey sütun olarak yerleştirilir; tıklama
//! isabet kutuları çağırana bildirilir.

use crate::cizim::{DikeyHiza, YatayHiza, Çizici};
use crate::koordinat::Dikdörtgen;
use crate::model::bilesen::{Gösterge, GöstergeSimgesi, Yön};
use crate::model::YatayKonum;
use crate::renk::{Dolgu, Renk};
use crate::tema;

/// Göstergedeki tek öğe.
#[derive(Clone, Debug)]
pub struct GöstergeÖğesi {
    pub ad: String,
    pub renk: Renk,
    pub simge: GöstergeSimgesi,
    pub kapalı: bool,
}

const ÜST_BOŞLUK: f32 = 5.0;
const SİMGE_METİN_ARASI: f32 = 5.0;

/// Göstergeyi çizer ve her öğenin `(isabet kutusu, ad)` çiftini döndürür.
/// Kutular grafik yerel koordinatındadır.
pub fn gösterge_çiz(
    çizici: &mut Çizici,
    seçenek: &Gösterge,
    öğeler: &[GöstergeÖğesi],
) -> Vec<(Dikdörtgen, String)> {
    if !seçenek.göster || öğeler.is_empty() {
        return Vec::new();
    }
    let boyut = seçenek.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let satır_yüksekliği = seçenek.simge_yüksekliği.max(boyut * 1.4);

    // Öğe genişliklerini ölç.
    let genişlikler: Vec<f32> = öğeler
        .iter()
        .map(|ö| {
            seçenek.simge_genişliği + SİMGE_METİN_ARASI + çizici.yazı_ölç(&ö.ad, boyut).0
        })
        .collect();

    let üst = seçenek.üst.map(|u| u.çöz(çizici.yükseklik)).unwrap_or(ÜST_BOŞLUK);

    let mut kutular = Vec::with_capacity(öğeler.len());
    match seçenek.yön {
        Yön::Yatay => {
            let toplam: f32 = genişlikler.iter().sum::<f32>()
                + seçenek.öğe_boşluğu * (öğeler.len().saturating_sub(1)) as f32;
            let mut x = match seçenek.sol {
                YatayKonum::Sol => ÜST_BOŞLUK,
                YatayKonum::Orta => (çizici.genişlik - toplam) / 2.0,
                YatayKonum::Sağ => çizici.genişlik - toplam - ÜST_BOŞLUK,
                YatayKonum::Değer(u) => u.çöz(çizici.genişlik),
            };
            for (öğe, genişlik) in öğeler.iter().zip(&genişlikler) {
                let kutu = Dikdörtgen::yeni(x, üst, *genişlik, satır_yüksekliği);
                öğe_çiz(çizici, seçenek, öğe, kutu, boyut);
                kutular.push((kutu, öğe.ad.clone()));
                x += genişlik + seçenek.öğe_boşluğu;
            }
        }
        Yön::Dikey => {
            let en_geniş = genişlikler.iter().copied().fold(0.0f32, f32::max);
            let x = match seçenek.sol {
                YatayKonum::Sol => ÜST_BOŞLUK,
                YatayKonum::Orta => (çizici.genişlik - en_geniş) / 2.0,
                YatayKonum::Sağ => çizici.genişlik - en_geniş - ÜST_BOŞLUK,
                YatayKonum::Değer(u) => u.çöz(çizici.genişlik),
            };
            let mut y = üst;
            for (öğe, genişlik) in öğeler.iter().zip(&genişlikler) {
                let kutu = Dikdörtgen::yeni(x, y, *genişlik, satır_yüksekliği);
                öğe_çiz(çizici, seçenek, öğe, kutu, boyut);
                kutular.push((kutu, öğe.ad.clone()));
                y += satır_yüksekliği + seçenek.öğe_boşluğu;
            }
        }
    }
    kutular
}

fn öğe_çiz(
    çizici: &mut Çizici,
    seçenek: &Gösterge,
    öğe: &GöstergeÖğesi,
    kutu: Dikdörtgen,
    yazı_boyutu: f32,
) {
    let renk = if öğe.kapalı { tema::DEVRE_DIŞI } else { öğe.renk };
    let simge = seçenek.simge.unwrap_or(öğe.simge);
    let orta_y = kutu.y + kutu.yükseklik / 2.0;

    match simge {
        GöstergeSimgesi::YuvarlakKöşeliKare => {
            let d = Dikdörtgen::yeni(
                kutu.x,
                orta_y - seçenek.simge_yüksekliği / 2.0,
                seçenek.simge_genişliği,
                seçenek.simge_yüksekliği,
            );
            çizici.dikdörtgen(d, &Dolgu::Düz(renk), [3.0; 4], None);
        }
        GöstergeSimgesi::Daire => {
            let yarıçap = seçenek.simge_yüksekliği / 2.0;
            çizici.daire(
                (kutu.x + seçenek.simge_genişliği / 2.0, orta_y),
                yarıçap,
                Some(&Dolgu::Düz(renk)),
                None,
            );
        }
        GöstergeSimgesi::Çizgi => {
            // ECharts çizgi serisi simgesi: yatay çizgi + ortada içi boş nokta.
            çizici.çizgi(
                (kutu.x, orta_y),
                (kutu.x + seçenek.simge_genişliği, orta_y),
                2.0,
                renk,
                crate::model::stil::ÇizgiTürü::Düz,
            );
            çizici.daire(
                (kutu.x + seçenek.simge_genişliği / 2.0, orta_y),
                4.0,
                Some(&Dolgu::Düz(Renk::BEYAZ)),
                Some((2.0, renk)),
            );
        }
    }

    let yazı_rengi = if öğe.kapalı {
        tema::DEVRE_DIŞI
    } else {
        seçenek.yazı.renk.unwrap_or(tema::İKİNCİL_METİN)
    };
    çizici.yazı(
        &öğe.ad,
        (kutu.x + seçenek.simge_genişliği + SİMGE_METİN_ARASI, orta_y),
        YatayHiza::Sol,
        DikeyHiza::Orta,
        yazı_boyutu,
        yazı_rengi,
        false,
    );
}
