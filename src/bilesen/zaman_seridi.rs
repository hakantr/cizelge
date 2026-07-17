//! Zaman şeridi (timeline) — `echarts/src/component/timeline` karşılığı:
//! pencerenin altında kare noktaları + oynat/durdur düğmesi çizer.
//! Kare listesi ve oynatma durumu görünüm katmanında tutulur
//! (`GrafikGörünümü::film`); burası yalnız çizim ve isabet kutularıdır.

use crate::cizim::{keskin, ÇizimYüzeyi, Yol};
use crate::koordinat::Dikdörtgen;
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
        let x = if toplam > 1 { sol + sıra as f32 * adım } else { (sol + sağ) / 2.0 };
        let seçili = sıra == geçerli;
        if seçili {
            // Geçerli kare: dolu + halka vurgusu.
            yüzey.daire((x, orta_y), 7.5, Some(&Dolgu::Düz(vurgu.opaklık(0.25))), None);
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
