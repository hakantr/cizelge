//! Gösterge (legend) bileşeni — `echarts/src/component/legend` karşılığı.
//!
//! Öğeler yatay tek satır ya da dikey sütun olarak yerleştirilir; tıklama
//! isabet kutuları çağırana bildirilir.

use crate::cizim::{DikeyHiza, YatayHiza, ÇizimYüzeyi};
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
    let satır_yüksekliği = seçenek.simge_yüksekliği.max(boyut * 1.4);

    // Öğe genişliklerini ölç.
    let genişlikler: Vec<f32> = öğeler
        .iter()
        .map(|ö| {
            seçenek.simge_genişliği + SİMGE_METİN_ARASI + çizici.yazı_ölç(&ö.ad, boyut).0
        })
        .collect();

    let üst = seçenek.üst.map(|u| u.çöz(çizici.yükseklik())).unwrap_or(ÜST_BOŞLUK);

    // Kaydırmalı yatay gösterge: sayfaya böl.
    if seçenek.kaydırılabilir && seçenek.yön == Yön::Yatay {
        let denetim_genişliği = 96.0;
        let kullanılabilir = (çizici.genişlik() - denetim_genişliği - ÜST_BOŞLUK * 2.0).max(60.0);
        // Sayfaları doldur.
        let mut sayfalar: Vec<(usize, usize)> = Vec::new(); // (başlangıç, uzunluk)
        let mut başlangıç = 0usize;
        while başlangıç < öğeler.len() {
            let mut genişlik_toplam = 0.0f32;
            let mut uzunluk = 0usize;
            for g in genişlikler.iter().skip(başlangıç) {
                let ek = if uzunluk == 0 { *g } else { seçenek.öğe_boşluğu + *g };
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
        let mut x = ÜST_BOŞLUK;
        for (öğe, genişlik) in öğeler
            .iter()
            .zip(&genişlikler)
            .skip(s_baş)
            .take(s_uzunluk)
        {
            let kutu = Dikdörtgen::yeni(x, üst, *genişlik, satır_yüksekliği);
            öğe_çiz(çizici, seçenek, öğe, kutu, boyut);
            kutular.push((kutu, öğe.ad.clone()));
            x += genişlik + seçenek.öğe_boşluğu;
        }

        // Denetimler: ‹ n/m ›
        let mut oklar = Vec::new();
        let denetim_x = çizici.genişlik() - denetim_genişliği;
        let orta_y = üst + satır_yüksekliği / 2.0;
        let sol_ok = Dikdörtgen::yeni(denetim_x, üst, 18.0, satır_yüksekliği);
        let sağ_ok = Dikdörtgen::yeni(denetim_x + 70.0, üst, 18.0, satır_yüksekliği);
        for (kutu, işaret, yön) in [(sol_ok, "‹", -1i32), (sağ_ok, "›", 1i32)] {
            çizici.dikdörtgen(kutu, &Dolgu::Düz(tema::NÖTR_10), [3.0; 4], None);
            çizici.yazı(
                işaret,
                kutu.merkez(),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                boyut,
                tema::İKİNCİL_METİN,
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
            tema::İKİNCİL_METİN,
            false,
        );
        return GöstergeÇıktısı { kutular, oklar, sayfa_sayısı };
    }

    let mut kutular = Vec::with_capacity(öğeler.len());
    match seçenek.yön {
        Yön::Yatay => {
            let toplam: f32 = genişlikler.iter().sum::<f32>()
                + seçenek.öğe_boşluğu * (öğeler.len().saturating_sub(1)) as f32;
            let mut x = match seçenek.sol {
                YatayKonum::Sol => ÜST_BOŞLUK,
                YatayKonum::Orta => (çizici.genişlik() - toplam) / 2.0,
                YatayKonum::Sağ => çizici.genişlik() - toplam - ÜST_BOŞLUK,
                YatayKonum::Değer(u) => u.çöz(çizici.genişlik()),
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
                YatayKonum::Orta => (çizici.genişlik() - en_geniş) / 2.0,
                YatayKonum::Sağ => çizici.genişlik() - en_geniş - ÜST_BOŞLUK,
                YatayKonum::Değer(u) => u.çöz(çizici.genişlik()),
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
    GöstergeÇıktısı { kutular, oklar: Vec::new(), sayfa_sayısı: 1 }
}

fn öğe_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
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
