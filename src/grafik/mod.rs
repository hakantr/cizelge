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

use crate::cizim::{AfinMatris, Yol, yolu_dönüştür, ÇizimYüzeyi};
use crate::model::seri::Sembol;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

/// Bir veri noktası sembolü çizer. `boyut`, ECharts'taki gibi çaptır.
pub fn sembol_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    sembol: &Sembol,
    merkez: (f32, f32),
    boyut: f32,
    renk: Renk,
) {
    sembol_stilli_çiz(çizici, sembol, merkez, boyut, renk, None, None, 1.0, false);
}

/// Sembolün dolgu/gölge için ortak yolunu üretir. Özel SVG yolları zrender
/// `makePath` gibi kesin sınır kutusundan hedef sembol kutusuna taşınır.
pub(crate) fn sembol_yolu(
    sembol: &Sembol,
    merkez: (f32, f32),
    boyut: f32,
    oranı_koru: bool,
) -> Option<Yol> {
    let yarıçap = boyut / 2.0;
    if yarıçap <= 0.0 || matches!(sembol, Sembol::Yok) {
        return None;
    }
    let mut yol = Yol::yeni();
    match sembol {
        Sembol::Daire | Sembol::İçiBoşDaire => {
            yol.taşı((merkez.0 + yarıçap, merkez.1));
            yol.yay(yarıçap, false, true, (merkez.0 - yarıçap, merkez.1));
            yol.yay(yarıçap, false, true, (merkez.0 + yarıçap, merkez.1));
        }
        Sembol::Kare => {
            yol.taşı((merkez.0 - yarıçap, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1 + yarıçap));
        }
        Sembol::YuvarlakDikdörtgen => {
            let köşe = boyut / 4.0;
            yol.taşı((merkez.0 - yarıçap + köşe, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap - köşe, merkez.1 - yarıçap));
            yol.yay(
                köşe,
                false,
                true,
                (merkez.0 + yarıçap, merkez.1 - yarıçap + köşe),
            );
            yol.çiz((merkez.0 + yarıçap, merkez.1 + yarıçap - köşe));
            yol.yay(
                köşe,
                false,
                true,
                (merkez.0 + yarıçap - köşe, merkez.1 + yarıçap),
            );
            yol.çiz((merkez.0 - yarıçap + köşe, merkez.1 + yarıçap));
            yol.yay(
                köşe,
                false,
                true,
                (merkez.0 - yarıçap, merkez.1 + yarıçap - köşe),
            );
            yol.çiz((merkez.0 - yarıçap, merkez.1 - yarıçap + köşe));
            yol.yay(
                köşe,
                false,
                true,
                (merkez.0 - yarıçap + köşe, merkez.1 - yarıçap),
            );
        }
        Sembol::Üçgen => {
            yol.taşı((merkez.0, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1 + yarıçap));
        }
        Sembol::Elmas => {
            yol.taşı((merkez.0, merkez.1 - yarıçap));
            yol.çiz((merkez.0 + yarıçap, merkez.1));
            yol.çiz((merkez.0, merkez.1 + yarıçap));
            yol.çiz((merkez.0 - yarıçap, merkez.1));
        }
        Sembol::SvgYolu(kaynak) => {
            let kutu = kaynak.kesin_sınır_kutusu()?;
            if kutu.genişlik <= f32::EPSILON || kutu.yükseklik <= f32::EPSILON {
                return None;
            }
            let mut x_ölçeği = boyut / kutu.genişlik;
            let mut y_ölçeği = boyut / kutu.yükseklik;
            if oranı_koru {
                let ölçek = x_ölçeği.min(y_ölçeği);
                x_ölçeği = ölçek;
                y_ölçeği = ölçek;
            }
            let dönüşüm = AfinMatris::yeni(
                x_ölçeği,
                0.0,
                0.0,
                y_ölçeği,
                merkez.0 - (kutu.x + kutu.genişlik / 2.0) * x_ölçeği,
                merkez.1 - (kutu.y + kutu.yükseklik / 2.0) * y_ölçeği,
            );
            return Some(yolu_dönüştür(kaynak, dönüşüm));
        }
        Sembol::Yok => return None,
    }
    yol.kapat();
    Some(yol)
}

/// Sembolü seri/veri `itemStyle` dolgusu, kenarlığı ve opaklığıyla çizer.
/// `dolgu` verilmezse dolu semboller seri rengini, `emptyCircle` ise ECharts
/// gibi beyaz iç ve seri rengi halka kullanır.
#[allow(clippy::too_many_arguments)]
pub fn sembol_stilli_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    sembol: &Sembol,
    merkez: (f32, f32),
    boyut: f32,
    renk: Renk,
    dolgu: Option<&Dolgu>,
    kenarlık: Option<(f32, Renk)>,
    opaklık: f32,
    oranı_koru: bool,
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
        Sembol::YuvarlakDikdörtgen => {
            let d = crate::koordinat::Dikdörtgen::yeni(
                merkez.0 - yarıçap,
                merkez.1 - yarıçap,
                boyut,
                boyut,
            );
            çizici.dikdörtgen(d, &dolgu, [boyut / 4.0; 4], kenarlık);
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
        Sembol::SvgYolu(_) => {
            if let Some(yol) = sembol_yolu(sembol, merkez, boyut, oranı_koru) {
                çizici.yol_doldur(&yol, &dolgu);
                if let Some((kalınlık, renk)) = kenarlık {
                    çizici.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
                }
            }
        }
    }
}

/// Sembolü merkezi çevresinde ECharts `symbolRotate` derecesiyle döndürür.
/// Dairelerde dönüş görünümü değiştirmediğinden yerel yüzey ilkelini korur;
/// diğer semboller aynı yol, dolgu ve kenarlık sözleşmesiyle boyanır.
#[allow(clippy::too_many_arguments)]
pub fn sembol_stilli_dönüşümlü_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    sembol: &Sembol,
    merkez: (f32, f32),
    boyut: f32,
    derece: f32,
    renk: Renk,
    dolgu: Option<&Dolgu>,
    kenarlık: Option<(f32, Renk)>,
    opaklık: f32,
    oranı_koru: bool,
) {
    if !derece.is_finite()
        || derece.abs() <= f32::EPSILON
        || matches!(sembol, Sembol::Daire | Sembol::İçiBoşDaire | Sembol::Yok)
    {
        sembol_stilli_çiz(
            çizici,
            sembol,
            merkez,
            boyut,
            renk,
            dolgu,
            kenarlık,
            opaklık,
            oranı_koru,
        );
        return;
    }
    let Some(yol) = sembol_dönüşümlü_yolu(sembol, merkez, boyut, oranı_koru, derece) else {
        return;
    };
    let opaklık = opaklık.clamp(0.0, 1.0);
    let varsayılan_dolgu = Dolgu::Düz(renk);
    let dolgu = dolgu.unwrap_or(&varsayılan_dolgu).opaklık(opaklık);
    çizici.yol_doldur(&yol, &dolgu);
    if let Some((kalınlık, kenarlık)) = kenarlık
        && kalınlık > 0.0
    {
        çizici.yol_çiz(&yol, kalınlık, kenarlık.opaklık(opaklık), ÇizgiTürü::Düz);
    }
}

pub(crate) fn sembol_dönüşümlü_yolu(
    sembol: &Sembol,
    merkez: (f32, f32),
    boyut: f32,
    oranı_koru: bool,
    derece: f32,
) -> Option<Yol> {
    let yol = sembol_yolu(sembol, merkez, boyut, oranı_koru)?;
    if !derece.is_finite()
        || derece.abs() <= f32::EPSILON
        || matches!(sembol, Sembol::Daire | Sembol::İçiBoşDaire)
    {
        return Some(yol);
    }
    let dönüşüm = AfinMatris::ötele(merkez.0, merkez.1)
        .çarp(AfinMatris::döndür(derece.to_radians()))
        .çarp(AfinMatris::ötele(-merkez.0, -merkez.1));
    Some(yolu_dönüştür(&yol, dönüşüm))
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

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn svg_sembol_path_onekini_cozer_ve_orani_koruyarak_ortalar() {
        let sembol = Sembol::svg_yolu("path://M0 0H20V10H0Z").expect("SVG yolu çözülmeli");
        let yol = sembol_yolu(&sembol, (50.0, 50.0), 40.0, true).expect("sembol yolu");
        let kutu = yol.kesin_sınır_kutusu().expect("sınır kutusu");
        assert!((kutu.x - 30.0).abs() < 1e-4);
        assert!((kutu.y - 40.0).abs() < 1e-4);
        assert!((kutu.genişlik - 40.0).abs() < 1e-4);
        assert!((kutu.yükseklik - 20.0).abs() < 1e-4);

        let kaplayan = sembol_yolu(&sembol, (50.0, 50.0), 40.0, false)
            .expect("kaplayan sembol yolu")
            .kesin_sınır_kutusu()
            .expect("kaplayan sınır kutusu");
        assert!((kaplayan.y - 30.0).abs() < 1e-4);
        assert!((kaplayan.yükseklik - 40.0).abs() < 1e-4);
    }

    #[test]
    fn yuvarlak_dikdortgen_kisa_kenarin_dortte_birini_kullanir() {
        let yol = sembol_yolu(&Sembol::YuvarlakDikdörtgen, (20.0, 30.0), 16.0, false)
            .expect("roundRect yolu");
        let kutu = yol.kesin_sınır_kutusu().expect("roundRect sınırı");
        assert!((kutu.x - 12.0).abs() < 1e-4);
        assert!((kutu.y - 22.0).abs() < 1e-4);
        assert!((kutu.genişlik - 16.0).abs() < 1e-4);
        assert!((kutu.yükseklik - 16.0).abs() < 1e-4);
    }
}
