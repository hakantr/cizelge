//! Seri çizicileri — `echarts/src/chart` dizininin karşılığı.

pub mod cizgi;
pub mod imleyici;
pub mod isi;
pub mod mum;
pub mod pasta;
pub mod sacilim;
pub mod sutun;

use crate::cizim::ÇizimYüzeyi;
use crate::model::seri::Sembol;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

/// Bir veri noktası sembolü çizer. `boyut`, ECharts'taki gibi çaptır.
pub fn sembol_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    sembol: Sembol,
    merkez: (f32, f32),
    boyut: f32,
    renk: Renk,
) {
    let yarıçap = boyut / 2.0;
    if yarıçap <= 0.0 {
        return;
    }
    match sembol {
        Sembol::Yok => {}
        Sembol::İçiBoşDaire => {
            // ECharts `emptyCircle`: beyaz iç, seri renginde halka.
            çizici.daire(merkez, yarıçap, Some(&Dolgu::Düz(Renk::BEYAZ)), Some((1.5, renk)));
        }
        Sembol::Daire => {
            çizici.daire(merkez, yarıçap, Some(&Dolgu::Düz(renk)), None);
        }
        Sembol::Kare => {
            let d = crate::koordinat::Dikdörtgen::yeni(
                merkez.0 - yarıçap,
                merkez.1 - yarıçap,
                boyut,
                boyut,
            );
            çizici.dikdörtgen(d, &Dolgu::Düz(renk), [0.0; 4], None);
        }
        Sembol::Üçgen => {
            let mut yol = crate::cizim::Yol::yeni();
            yol.taşı((merkez.0, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1 + yarıçap));
            yol.kapat();
            çizici.yol_doldur(&yol, &Dolgu::Düz(renk));
        }
        Sembol::Elmas => {
            let mut yol = crate::cizim::Yol::yeni();
            yol.taşı((merkez.0, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1));
            yol.çiz((merkez.0, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1));
            yol.kapat();
            çizici.yol_doldur(&yol, &Dolgu::Düz(renk));
        }
    }
}

/// Çizgi stilinin çözülmüş görünümü.
pub fn çizgi_stili_çöz(
    stil: &crate::model::stil::ÇizgiStili,
    seri_rengi: Renk,
) -> (Renk, f32, ÇizgiTürü) {
    (
        stil.renk.unwrap_or(seri_rengi).opaklık(stil.opaklık),
        stil.kalınlık,
        stil.tür,
    )
}
