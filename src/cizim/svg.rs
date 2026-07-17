//! SVG yüzeyi — [`ÇizimYüzeyi`]nin SVG üreten gerçeklemesi: grafik,
//! gpui olmadan bir `.svg` belgesine dışa aktarılır (Faz 7 çıktı hedefi).
//!
//! Metin ölçümü belirlenimcidir (karakter × boyut × 0.6); bu nedenle SVG
//! çıktıdaki yazı yerleşimi, ekrandaki gpui ölçümüyle piksel piksel aynı
//! olmayabilir.

use std::fmt::Write as _;

use crate::cizim::yuzey::{DikeyHiza, SATIR_ORANI, YatayHiza, Yol, YolKomutu, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

fn renk_svg(r: Renk) -> String {
    format!(
        "rgba({},{},{},{})",
        (r.kırmızı * 255.0).round() as u8,
        (r.yeşil * 255.0).round() as u8,
        (r.mavi * 255.0).round() as u8,
        (r.alfa * 1000.0).round() / 1000.0
    )
}

fn yol_svg(yol: &Yol) -> String {
    let mut d = String::new();
    for komut in &yol.komutlar {
        match *komut {
            YolKomutu::Taşı(n) => {
                let _ = write!(d, "M{:.1} {:.1} ", n.0, n.1);
            }
            YolKomutu::Çiz(n) => {
                let _ = write!(d, "L{:.1} {:.1} ", n.0, n.1);
            }
            YolKomutu::Kübik { k1, k2, uç } => {
                let _ = write!(
                    d,
                    "C{:.1} {:.1} {:.1} {:.1} {:.1} {:.1} ",
                    k1.0, k1.1, k2.0, k2.1, uç.0, uç.1
                );
            }
            YolKomutu::Yay { yarıçap, büyük_yay, süpürme, uç } => {
                let _ = write!(
                    d,
                    "A{:.1} {:.1} 0 {} {} {:.1} {:.1} ",
                    yarıçap,
                    yarıçap,
                    if büyük_yay { 1 } else { 0 },
                    if süpürme { 1 } else { 0 },
                    uç.0,
                    uç.1
                );
            }
            YolKomutu::Kapat => d.push_str("Z "),
        }
    }
    d.trim_end().to_string()
}

fn kaçır(metin: &str) -> String {
    metin
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// SVG belgesi üreten çizim yüzeyi.
pub struct SvgYüzeyi {
    genişlik: f32,
    yükseklik: f32,
    gövde: String,
    tanımlar: String,
    gradyan_sayacı: usize,
    kırpma_sayacı: usize,
    açık_gruplar: usize,
}

impl SvgYüzeyi {
    pub fn yeni(genişlik: f32, yükseklik: f32) -> Self {
        SvgYüzeyi {
            genişlik,
            yükseklik,
            gövde: String::new(),
            tanımlar: String::new(),
            gradyan_sayacı: 0,
            kırpma_sayacı: 0,
            açık_gruplar: 0,
        }
    }

    /// Dolguyu SVG boya referansına çevirir; gradyanlar `<defs>`e yazılır.
    fn dolgu_svg(&mut self, dolgu: &Dolgu) -> String {
        match dolgu {
            Dolgu::Düz(r) => renk_svg(*r),
            Dolgu::DoğrusalGradyan { x, y, x2, y2, duraklar } => {
                self.gradyan_sayacı += 1;
                let kimlik = format!("grd{}", self.gradyan_sayacı);
                let _ = write!(
                    self.tanımlar,
                    r#"<linearGradient id="{kimlik}" x1="{x}" y1="{y}" x2="{x2}" y2="{y2}">"#
                );
                for durak in duraklar {
                    let _ = write!(
                        self.tanımlar,
                        r#"<stop offset="{}" stop-color="{}"/>"#,
                        durak.konum,
                        renk_svg(durak.renk)
                    );
                }
                self.tanımlar.push_str("</linearGradient>");
                format!("url(#{kimlik})")
            }
            Dolgu::RadyalGradyan { x, y, yarıçap, duraklar } => {
                self.gradyan_sayacı += 1;
                let kimlik = format!("grd{}", self.gradyan_sayacı);
                let _ = write!(
                    self.tanımlar,
                    r#"<radialGradient id="{kimlik}" cx="{x}" cy="{y}" r="{yarıçap}">"#
                );
                for durak in duraklar {
                    let _ = write!(
                        self.tanımlar,
                        r#"<stop offset="{}" stop-color="{}"/>"#,
                        durak.konum,
                        renk_svg(durak.renk)
                    );
                }
                self.tanımlar.push_str("</radialGradient>");
                format!("url(#{kimlik})")
            }
        }
    }

    /// Tam SVG belgesini üretir.
    pub fn belge(&self) -> String {
        format!(
            concat!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{g}" height="{y}" "#,
                r#"viewBox="0 0 {g} {y}" font-family="sans-serif">"#,
                "<defs>{tanımlar}</defs>{gövde}{kapanışlar}</svg>"
            ),
            g = self.genişlik,
            y = self.yükseklik,
            tanımlar = self.tanımlar,
            gövde = self.gövde,
            kapanışlar = "</g>".repeat(self.açık_gruplar),
        )
    }
}

impl ÇizimYüzeyi for SvgYüzeyi {
    fn genişlik(&self) -> f32 {
        self.genişlik
    }

    fn yükseklik(&self) -> f32 {
        self.yükseklik
    }

    fn yol_doldur(&mut self, yol: &Yol, dolgu: &Dolgu) {
        if yol.boş_mu() {
            return;
        }
        let boya = self.dolgu_svg(dolgu);
        let _ = write!(
            self.gövde,
            r#"<path d="{}" fill="{}"/>"#,
            yol_svg(yol),
            boya
        );
    }

    fn yol_çiz(&mut self, yol: &Yol, kalınlık: f32, renk: Renk, tür: ÇizgiTürü) {
        if yol.boş_mu() || kalınlık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        let desen = match tür {
            ÇizgiTürü::Düz => String::new(),
            ÇizgiTürü::Kesikli => format!(
                r#" stroke-dasharray="{} {}""#,
                4.0 * kalınlık.max(1.0),
                2.0 * kalınlık.max(1.0)
            ),
            ÇizgiTürü::Noktalı => format!(
                r#" stroke-dasharray="{} {}""#,
                kalınlık.max(1.0),
                kalınlık.max(1.0)
            ),
        };
        let _ = write!(
            self.gövde,
            r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"{}/>"#,
            yol_svg(yol),
            renk_svg(renk),
            kalınlık,
            desen
        );
    }

    fn dikdörtgen(
        &mut self,
        d: Dikdörtgen,
        dolgu: &Dolgu,
        yarıçap: [f32; 4],
        kenarlık: Option<(f32, Renk)>,
    ) {
        if d.genişlik <= 0.0 || d.yükseklik <= 0.0 {
            return;
        }
        let boya = self.dolgu_svg(dolgu);
        let rx = yarıçap.iter().fold(0.0f32, |a, b| a.max(*b));
        let kenar = match kenarlık {
            Some((kalınlık, renk)) if kalınlık > 0.0 => format!(
                r#" stroke="{}" stroke-width="{}""#,
                renk_svg(renk),
                kalınlık
            ),
            _ => String::new(),
        };
        let _ = write!(
            self.gövde,
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="{:.1}" fill="{}"{}/>"#,
            d.x, d.y, d.genişlik, d.yükseklik, rx, boya, kenar
        );
    }

    fn gölge(&mut self, _d: Dikdörtgen, _yarıçap: f32, _renk: Renk, _bulanıklık: f32) {
        // SVG çıktıda gölge yumuşatması uygulanmaz (filtre maliyeti).
    }

    fn kırpılı(&mut self, d: Dikdörtgen, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi)) {
        self.kırpma_sayacı += 1;
        let kimlik = format!("krp{}", self.kırpma_sayacı);
        let _ = write!(
            self.tanımlar,
            r#"<clipPath id="{kimlik}"><rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}"/></clipPath>"#,
            d.x, d.y, d.genişlik, d.yükseklik
        );
        let _ = write!(self.gövde, r#"<g clip-path="url(#{kimlik})">"#);
        self.açık_gruplar += 1;
        işlev(self);
        self.açık_gruplar -= 1;
        self.gövde.push_str("</g>");
    }

    fn yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
    ) -> (f32, f32) {
        if metin.is_empty() {
            return (0.0, 0.0);
        }
        let (genişlik, yükseklik) = self.yazı_ölç(metin, boyut);
        let çapa = match yatay {
            YatayHiza::Sol => "start",
            YatayHiza::Orta => "middle",
            YatayHiza::Sağ => "end",
        };
        // Dikey hizalama: satır kutusunun ortasına göre taban çizgisi kaydırması.
        let y = match dikey {
            DikeyHiza::Üst => konum.1 + boyut * 0.95,
            DikeyHiza::Orta => konum.1 + boyut * 0.35,
            DikeyHiza::Alt => konum.1 - yükseklik + boyut * 0.95,
        };
        let _ = write!(
            self.gövde,
            r#"<text x="{:.1}" y="{:.1}" text-anchor="{}" font-size="{}" fill="{}"{}>{}</text>"#,
            konum.0,
            y,
            çapa,
            boyut,
            renk_svg(renk),
            if kalın { r#" font-weight="bold""# } else { "" },
            kaçır(metin)
        );
        (genişlik, yükseklik)
    }

    fn yazı_ölç(&self, metin: &str, boyut: f32) -> (f32, f32) {
        (
            metin.chars().count() as f32 * boyut * 0.6,
            boyut * SATIR_ORANI,
        )
    }

    fn olarak(&mut self) -> &mut dyn ÇizimYüzeyi {
        self
    }
}

/// Grafiği SVG belgesine dışa aktarır.
pub fn svg_dışa_aktar(
    seçenekler: &crate::model::secenekler::GrafikSeçenekleri,
    genişlik: f32,
    yükseklik: f32,
) -> String {
    let mut yüzey = SvgYüzeyi::yeni(genişlik, yükseklik);
    crate::cizim::gorunum::grafiği_boya(
        &mut yüzey,
        seçenekler,
        &crate::cizim::gorunum::BoyamaGirdisi::default(),
    );
    yüzey.belge()
}
