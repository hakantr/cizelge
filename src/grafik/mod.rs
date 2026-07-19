//! Seri çizicileri — `echarts/src/chart` dizininin karşılığı.

pub mod agac;
pub mod agac_haritasi;
pub mod cizgi;
pub mod gosterge_saati;
pub mod grafo;
pub mod gunes;
pub mod hatlar;
pub mod huni;
pub mod imleyici;
pub mod isi;
pub mod kiris;
pub mod kutupsal;
pub mod mum;
pub mod paralel;
pub mod pasta;
pub mod radar;
pub mod sacilim;
pub mod sankey;
pub mod sutun;
pub mod takvim_isi;
pub mod tema_nehri;

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
    sembol_stilli_çiz(çizici, sembol, merkez, boyut, renk, None, None, 1.0);
}

/// Sembolü seri/veri `itemStyle` dolgusu, kenarlığı ve opaklığıyla çizer.
/// `dolgu` verilmezse dolu semboller seri rengini, `emptyCircle` ise ECharts
/// gibi beyaz iç ve seri rengi halka kullanır.
#[allow(clippy::too_many_arguments)]
pub fn sembol_stilli_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    sembol: Sembol,
    merkez: (f32, f32),
    boyut: f32,
    renk: Renk,
    dolgu: Option<&Dolgu>,
    kenarlık: Option<(f32, Renk)>,
    opaklık: f32,
) {
    let yarıçap = boyut / 2.0;
    if yarıçap <= 0.0 {
        return;
    }
    let opaklık = opaklık.clamp(0.0, 1.0);
    let varsayılan_dolgu = Dolgu::Düz(renk);
    let dolgu = dolgu.unwrap_or(&varsayılan_dolgu).opaklık(opaklık);
    let kenarlık = kenarlık.map(|(kalınlık, renk)| (kalınlık, renk.opaklık(opaklık)));
    match sembol {
        Sembol::Yok => {}
        Sembol::İçiBoşDaire => {
            // ECharts `emptyCircle`: beyaz iç, seri renginde halka.
            let iç = if dolgu.temsilî() == renk {
                Dolgu::Düz(Renk::BEYAZ.opaklık(opaklık))
            } else {
                dolgu.clone()
            };
            çizici.daire(
                merkez,
                yarıçap,
                Some(&iç),
                // zrender `symbolPathSetColor`, empty symbol çizgisini
                // `lineWidth = 2` ile sabitler ve `strokeNoScale` uygular.
                kenarlık.or(Some((2.0, renk.opaklık(opaklık)))),
            );
        }
        Sembol::Daire => {
            çizici.daire(merkez, yarıçap, Some(&dolgu), kenarlık);
        }
        Sembol::Kare => {
            let d = crate::koordinat::Dikdörtgen::yeni(
                merkez.0 - yarıçap,
                merkez.1 - yarıçap,
                boyut,
                boyut,
            );
            çizici.dikdörtgen(d, &dolgu, [0.0; 4], kenarlık);
        }
        Sembol::Üçgen => {
            let mut yol = crate::cizim::Yol::yeni();
            yol.taşı((merkez.0, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1 + yarıçap));
            yol.kapat();
            çizici.yol_doldur(&yol, &dolgu);
            if let Some((kalınlık, renk)) = kenarlık {
                çizici.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
            }
        }
        Sembol::Elmas => {
            let mut yol = crate::cizim::Yol::yeni();
            yol.taşı((merkez.0, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1));
            yol.çiz((merkez.0, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1));
            yol.kapat();
            çizici.yol_doldur(&yol, &dolgu);
            if let Some((kalınlık, renk)) = kenarlık {
                çizici.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
            }
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
