//! SVG yüzeyi — [`ÇizimYüzeyi`]nin SVG üreten gerçeklemesi: grafik,
//! gpui olmadan bir `.svg` belgesine dışa aktarılır (Faz 7 çıktı hedefi).
//!
//! Metin ölçümü belirlenimcidir (karakter × boyut × 0.6); bu nedenle SVG
//! çıktıdaki yazı yerleşimi, ekrandaki gpui ölçümüyle piksel piksel aynı
//! olmayabilir.

use std::fmt::Write as _;

use crate::cizim::donusum::AfinMatris;
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
            YolKomutu::Yay {
                yarıçap,
                büyük_yay,
                süpürme,
                uç,
            } => {
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

fn taban64(veri: &[u8]) -> String {
    const ALFABE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut çıktı = String::with_capacity(veri.len().div_ceil(3) * 4);
    for parça in veri.chunks(3) {
        let (a, b, c, uzunluk) = match parça {
            [a, b, c] => (*a, *b, *c, 3),
            [a, b] => (*a, *b, 0, 2),
            [a] => (*a, 0, 0, 1),
            _ => continue,
        };
        let değer = (u32::from(a) << 16) | (u32::from(b) << 8) | u32::from(c);
        for kaydırma in [18_u32, 12, 6, 0] {
            let sıra = ((değer >> kaydırma) & 0x3f) as usize;
            çıktı.push(char::from(*ALFABE.get(sıra).unwrap_or(&b'A')));
        }
        if uzunluk == 2 {
            çıktı.pop();
            çıktı.push('=');
        } else if uzunluk == 1 {
            çıktı.pop();
            çıktı.pop();
            çıktı.push_str("==");
        }
    }
    çıktı
}

/// Premultiplied RGBA8 deseni alfa maskeli BITMAPV4 olarak gömer. SVG
/// `image` veri URI'si böylece png özelliği kapalı derlemelerde de kayıpsız
/// ve kendi kendine yeterlidir.
fn desen_bmp(desen: &crate::renk::GörüntüDeseni) -> Option<Vec<u8>> {
    let satır = usize::try_from(desen.genişlik).ok()?.checked_mul(4)?;
    let görüntü_boyutu = satır.checked_mul(usize::try_from(desen.yükseklik).ok()?)?;
    let başlık = 14_usize + 108;
    let toplam = başlık.checked_add(görüntü_boyutu)?;
    let mut çıktı = Vec::with_capacity(toplam);
    çıktı.extend_from_slice(b"BM");
    çıktı.extend_from_slice(&u32::try_from(toplam).ok()?.to_le_bytes());
    çıktı.extend_from_slice(&[0; 4]);
    çıktı.extend_from_slice(&u32::try_from(başlık).ok()?.to_le_bytes());
    çıktı.extend_from_slice(&108_u32.to_le_bytes());
    çıktı.extend_from_slice(&i32::try_from(desen.genişlik).ok()?.to_le_bytes());
    çıktı.extend_from_slice(&i32::try_from(desen.yükseklik).ok()?.to_le_bytes());
    çıktı.extend_from_slice(&1_u16.to_le_bytes());
    çıktı.extend_from_slice(&32_u16.to_le_bytes());
    çıktı.extend_from_slice(&3_u32.to_le_bytes());
    çıktı.extend_from_slice(&u32::try_from(görüntü_boyutu).ok()?.to_le_bytes());
    çıktı.extend_from_slice(&[0; 16]);
    çıktı.extend_from_slice(&0x00ff_0000_u32.to_le_bytes());
    çıktı.extend_from_slice(&0x0000_ff00_u32.to_le_bytes());
    çıktı.extend_from_slice(&0x0000_00ff_u32.to_le_bytes());
    çıktı.extend_from_slice(&0xff00_0000_u32.to_le_bytes());
    çıktı.extend_from_slice(&0x7352_4742_u32.to_le_bytes());
    çıktı.extend_from_slice(&[0; 48]);
    for satır in desen.pikseller.chunks_exact(satır).rev() {
        for piksel in satır.chunks_exact(4) {
            let [kırmızı, yeşil, mavi, alfa] = piksel else {
                continue;
            };
            let düzelt = |kanal: u8| {
                if *alfa == 0 {
                    0
                } else {
                    ((u16::from(kanal) * 255 + u16::from(*alfa) / 2) / u16::from(*alfa)).min(255)
                        as u8
                }
            };
            çıktı.extend_from_slice(&[düzelt(*mavi), düzelt(*yeşil), düzelt(*kırmızı), *alfa]);
        }
    }
    (çıktı.len() == toplam).then_some(çıktı)
}

/// SVG belgesi üreten çizim yüzeyi.
pub struct SvgYüzeyi {
    genişlik: f32,
    yükseklik: f32,
    gövde: String,
    tanımlar: String,
    gradyan_sayacı: usize,
    kırpma_sayacı: usize,
    gölge_sayacı: usize,
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
            gölge_sayacı: 0,
            açık_gruplar: 0,
        }
    }

    /// Dolguyu SVG boya referansına çevirir; gradyanlar `<defs>`e yazılır.
    fn dolgu_svg(&mut self, dolgu: &Dolgu) -> String {
        match dolgu {
            Dolgu::Düz(r) => renk_svg(*r),
            Dolgu::Desen(desen) => {
                self.gradyan_sayacı += 1;
                let kimlik = format!("des{}", self.gradyan_sayacı);
                let veri = desen_bmp(desen)
                    .map(|bmp| taban64(&bmp))
                    .unwrap_or_default();
                let _ = write!(
                    self.tanımlar,
                    r#"<pattern id="{kimlik}" patternUnits="userSpaceOnUse" width="{}" height="{}"><image width="{}" height="{}" opacity="{}" href="data:image/bmp;base64,{veri}"/></pattern>"#,
                    desen.genişlik, desen.yükseklik, desen.genişlik, desen.yükseklik, desen.opaklık
                );
                format!("url(#{kimlik})")
            }
            Dolgu::DoğrusalGradyan {
                x,
                y,
                x2,
                y2,
                duraklar,
            } => {
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
            Dolgu::RadyalGradyan {
                x,
                y,
                yarıçap,
                duraklar,
            } => {
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
            r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="butt" stroke-linejoin="bevel"{}/>"#,
            yol_svg(yol),
            renk_svg(renk),
            kalınlık,
            desen
        );
    }

    fn yol_dolgulu_çiz(&mut self, yol: &Yol, kalınlık: f32, dolgu: &Dolgu, tür: ÇizgiTürü) {
        if yol.boş_mu() || kalınlık <= 0.0 {
            return;
        }
        let boya = self.dolgu_svg(dolgu);
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
            r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="butt" stroke-linejoin="bevel"{}/>"#,
            yol_svg(yol),
            boya,
            kalınlık,
            desen
        );
    }

    fn yol_gölgesi(&mut self, yol: &Yol, renk: Renk, bulanıklık: f32, kayma: (f32, f32)) {
        if yol.boş_mu() || bulanıklık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        self.gölge_sayacı += 1;
        let kimlik = format!("golge{}", self.gölge_sayacı);
        let _ = write!(
            self.tanımlar,
            r#"<filter id="{kimlik}" filterUnits="userSpaceOnUse" x="0" y="0" width="{}" height="{}"><feGaussianBlur stdDeviation="{}"/><feOffset dx="{}" dy="{}"/></filter>"#,
            self.genişlik,
            self.yükseklik,
            bulanıklık * 0.5,
            kayma.0,
            kayma.1
        );
        let _ = write!(
            self.gövde,
            r#"<path d="{}" fill="{}" filter="url(#{kimlik})"/>"#,
            yol_svg(yol),
            renk_svg(renk)
        );
    }

    fn yol_çizgi_gölgesi(
        &mut self,
        yol: &Yol,
        kalınlık: f32,
        tür: ÇizgiTürü,
        renk: Renk,
        bulanıklık: f32,
        kayma: (f32, f32),
    ) {
        if yol.boş_mu() || kalınlık <= 0.0 || bulanıklık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        self.gölge_sayacı += 1;
        let kimlik = format!("golge{}", self.gölge_sayacı);
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
            self.tanımlar,
            r#"<filter id="{kimlik}" filterUnits="userSpaceOnUse" x="0" y="0" width="{}" height="{}"><feGaussianBlur stdDeviation="{}"/><feOffset dx="{}" dy="{}"/></filter>"#,
            self.genişlik,
            self.yükseklik,
            bulanıklık * 0.5,
            kayma.0,
            kayma.1
        );
        let _ = write!(
            self.gövde,
            r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}"{} filter="url(#{kimlik})"/>"#,
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

    fn yol_kırpılı(&mut self, yol: &Yol, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi)) {
        self.kırpma_sayacı += 1;
        let kimlik = format!("krp{}", self.kırpma_sayacı);
        let _ = write!(
            self.tanımlar,
            r#"<clipPath id="{kimlik}"><path d="{}"/></clipPath>"#,
            yol_svg(yol)
        );
        let _ = write!(self.gövde, r#"<g clip-path="url(#{kimlik})">"#);
        self.açık_gruplar += 1;
        işlev(self);
        self.açık_gruplar = self.açık_gruplar.saturating_sub(1);
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

    fn dönüşümlü_yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
        dönüşüm: AfinMatris,
    ) -> (f32, f32) {
        let _ = write!(
            self.gövde,
            r#"<g transform="matrix({} {} {} {} {} {})">"#,
            dönüşüm.a, dönüşüm.b, dönüşüm.c, dönüşüm.d, dönüşüm.e, dönüşüm.f
        );
        self.açık_gruplar += 1;
        let ölçü = self.yazı(metin, konum, yatay, dikey, boyut, renk, kalın);
        self.açık_gruplar = self.açık_gruplar.saturating_sub(1);
        self.gövde.push_str("</g>");
        ölçü
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

#[cfg(test)]
mod testler {
    use super::*;
    use crate::renk::RenkDurağı;

    #[test]
    fn gradyan_vuruşu_svg_boya_referansını_korur() {
        let mut yüzey = SvgYüzeyi::yeni(120.0, 80.0);
        let mut yol = Yol::yeni();
        yol.taşı((10.0, 20.0));
        yol.çiz((110.0, 60.0));
        let gradyan = Dolgu::doğrusal(
            0.0,
            0.0,
            1.0,
            0.0,
            vec![
                RenkDurağı::yeni(0.0, 0x5070ddu32),
                RenkDurağı::yeni(1.0, 0xd4dcf7u32),
            ],
        );

        yüzey.yol_dolgulu_çiz(&yol, 2.0, &gradyan, ÇizgiTürü::Düz);
        let belge = yüzey.belge();
        assert!(belge.contains(r#"<linearGradient id="grd1""#));
        assert!(belge.contains(r#"stroke="url(#grd1)""#));
        assert!(belge.contains(r#"stroke-width="2""#));
    }
}
