//! SVG `pathData` çözücüsü.
//!
//! ECharts/zrender'ın `path` öğelerinde kabul ettiği standart SVG yol
//! komutlarını [`Yol`] komutlarına dönüştürür. Karesel Bezier ve eliptik yaylar
//! çizim yüzeylerinin ortak kübik Bezier ilkelinde temsil edilir.

use std::fmt;

use crate::koordinat::Dikdörtgen;

use super::yuzey::{Yol, YolKomutu};

/// SVG yol verisindeki sözdizimi hatası.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SvgYolHatası {
    /// Hatanın UTF-8 bayt konumu.
    pub konum: usize,
    pub ayrıntı: String,
}

impl SvgYolHatası {
    fn yeni(konum: usize, ayrıntı: impl Into<String>) -> Self {
        Self {
            konum,
            ayrıntı: ayrıntı.into(),
        }
    }
}

impl fmt::Display for SvgYolHatası {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SVG yol verisi {}. baytta geçersiz: {}",
            self.konum, self.ayrıntı
        )
    }
}

impl std::error::Error for SvgYolHatası {}

struct Çözücü<'a> {
    veri: &'a str,
    konum: usize,
}

impl<'a> Çözücü<'a> {
    fn yeni(veri: &'a str) -> Self {
        Self { veri, konum: 0 }
    }

    fn bayt(&self) -> Option<u8> {
        self.veri.as_bytes().get(self.konum).copied()
    }

    fn ayırıcıları_geç(&mut self) {
        while self
            .bayt()
            .is_some_and(|b| b == b',' || b.is_ascii_whitespace())
        {
            self.konum += 1;
        }
    }

    fn bitti_mi(&mut self) -> bool {
        self.ayırıcıları_geç();
        self.konum >= self.veri.len()
    }

    fn komut_var_mı(&mut self) -> bool {
        self.ayırıcıları_geç();
        self.bayt().is_some_and(|b| b.is_ascii_alphabetic())
    }

    fn komut(&mut self) -> Option<char> {
        self.ayırıcıları_geç();
        let bayt = self.bayt()?;
        if !bayt.is_ascii_alphabetic() {
            return None;
        }
        self.konum += 1;
        Some(char::from(bayt))
    }

    fn sayı_var_mı(&mut self) -> bool {
        self.ayırıcıları_geç();
        self.bayt()
            .is_some_and(|b| b.is_ascii_digit() || matches!(b, b'+' | b'-' | b'.'))
    }

    fn sayı(&mut self) -> Result<f32, SvgYolHatası> {
        self.ayırıcıları_geç();
        let başlangıç = self.konum;
        let baytlar = self.veri.as_bytes();

        if baytlar
            .get(self.konum)
            .is_some_and(|b| matches!(b, b'+' | b'-'))
        {
            self.konum += 1;
        }

        let tam_sayı_başı = self.konum;
        while baytlar.get(self.konum).is_some_and(u8::is_ascii_digit) {
            self.konum += 1;
        }
        let mut rakam_var = self.konum > tam_sayı_başı;

        if baytlar.get(self.konum) == Some(&b'.') {
            self.konum += 1;
            let kesir_başı = self.konum;
            while baytlar.get(self.konum).is_some_and(u8::is_ascii_digit) {
                self.konum += 1;
            }
            rakam_var |= self.konum > kesir_başı;
        }
        if !rakam_var {
            return Err(SvgYolHatası::yeni(başlangıç, "sayı bekleniyordu"));
        }

        if baytlar
            .get(self.konum)
            .is_some_and(|b| matches!(b, b'e' | b'E'))
        {
            self.konum += 1;
            if baytlar
                .get(self.konum)
                .is_some_and(|b| matches!(b, b'+' | b'-'))
            {
                self.konum += 1;
            }
            let üs_başı = self.konum;
            while baytlar.get(self.konum).is_some_and(u8::is_ascii_digit) {
                self.konum += 1;
            }
            if self.konum == üs_başı {
                return Err(SvgYolHatası::yeni(başlangıç, "üs rakamları eksik"));
            }
        }

        let ham = &self.veri[başlangıç..self.konum];
        let sayı = ham
            .parse::<f32>()
            .map_err(|_| SvgYolHatası::yeni(başlangıç, format!("`{ham}` sayı değil")))?;
        if !sayı.is_finite() {
            return Err(SvgYolHatası::yeni(başlangıç, "sayı sonlu olmalı"));
        }
        Ok(sayı)
    }

    fn çift(&mut self) -> Result<(f32, f32), SvgYolHatası> {
        Ok((self.sayı()?, self.sayı()?))
    }

    fn bayrak(&mut self) -> Result<bool, SvgYolHatası> {
        let konum = self.konum;
        match self.sayı()? {
            0.0 => Ok(false),
            1.0 => Ok(true),
            _ => Err(SvgYolHatası::yeni(konum, "yay bayrağı 0 ya da 1 olmalı")),
        }
    }

    fn hata(&self, ayrıntı: impl Into<String>) -> SvgYolHatası {
        SvgYolHatası::yeni(self.konum, ayrıntı)
    }
}

fn ekle(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 + b.0, a.1 + b.1)
}

fn göreli_nokta(nokta: (f32, f32), geçerli: (f32, f32), göreli: bool) -> (f32, f32) {
    if göreli {
        ekle(geçerli, nokta)
    } else {
        nokta
    }
}

fn yansıt(kontrol: (f32, f32), merkez: (f32, f32)) -> (f32, f32) {
    (2.0 * merkez.0 - kontrol.0, 2.0 * merkez.1 - kontrol.1)
}

fn kareseli_kübiğe_çevir(
    yol: &mut Yol,
    başlangıç: (f32, f32),
    kontrol: (f32, f32),
    uç: (f32, f32),
) {
    yol.kübik(
        (
            başlangıç.0 + (kontrol.0 - başlangıç.0) * 2.0 / 3.0,
            başlangıç.1 + (kontrol.1 - başlangıç.1) * 2.0 / 3.0,
        ),
        (
            uç.0 + (kontrol.0 - uç.0) * 2.0 / 3.0,
            uç.1 + (kontrol.1 - uç.1) * 2.0 / 3.0,
        ),
        uç,
    );
}

/// SVG eliptik yayını en çok 90 derecelik kübik Bezier parçalarına çevirir.
#[allow(clippy::too_many_arguments)]
fn eliptik_yayı_ekle(
    yol: &mut Yol,
    başlangıç: (f32, f32),
    mut rx: f32,
    mut ry: f32,
    dönüş_derece: f32,
    büyük_yay: bool,
    süpürme: bool,
    uç: (f32, f32),
) {
    rx = rx.abs();
    ry = ry.abs();
    if rx <= f32::EPSILON || ry <= f32::EPSILON {
        yol.çiz(uç);
        return;
    }
    if (başlangıç.0 - uç.0).abs() <= f32::EPSILON && (başlangıç.1 - uç.1).abs() <= f32::EPSILON
    {
        return;
    }

    let phi = dönüş_derece.rem_euclid(360.0).to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();
    let dx = (başlangıç.0 - uç.0) / 2.0;
    let dy = (başlangıç.1 - uç.1) / 2.0;
    let x1 = cos_phi * dx + sin_phi * dy;
    let y1 = -sin_phi * dx + cos_phi * dy;

    let oran = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry);
    if oran > 1.0 {
        let ölçek = oran.sqrt();
        rx *= ölçek;
        ry *= ölçek;
    }

    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let payda = rx2 * y1 * y1 + ry2 * x1 * x1;
    let pay = (rx2 * ry2 - payda).max(0.0);
    let işaret = if büyük_yay == süpürme { -1.0 } else { 1.0 };
    let katsayı = if payda <= f32::EPSILON {
        0.0
    } else {
        işaret * (pay / payda).sqrt()
    };
    let cx1 = katsayı * rx * y1 / ry;
    let cy1 = -katsayı * ry * x1 / rx;
    let merkez = (
        cos_phi * cx1 - sin_phi * cy1 + (başlangıç.0 + uç.0) / 2.0,
        sin_phi * cx1 + cos_phi * cy1 + (başlangıç.1 + uç.1) / 2.0,
    );

    let u = ((x1 - cx1) / rx, (y1 - cy1) / ry);
    let v = ((-x1 - cx1) / rx, (-y1 - cy1) / ry);
    let başlangıç_açısı = u.1.atan2(u.0);
    let mut açıklık = (u.0 * v.1 - u.1 * v.0).atan2(u.0 * v.0 + u.1 * v.1);
    if süpürme && açıklık < 0.0 {
        açıklık += std::f32::consts::TAU;
    } else if !süpürme && açıklık > 0.0 {
        açıklık -= std::f32::consts::TAU;
    }
    let parça_sayısı = (açıklık.abs() / std::f32::consts::FRAC_PI_2)
        .ceil()
        .max(1.0) as usize;
    let parça_açısı = açıklık / parça_sayısı as f32;

    let nokta = |açı: f32| {
        let (sin, cos) = açı.sin_cos();
        (
            merkez.0 + cos_phi * rx * cos - sin_phi * ry * sin,
            merkez.1 + sin_phi * rx * cos + cos_phi * ry * sin,
        )
    };
    let türev = |açı: f32| {
        let (sin, cos) = açı.sin_cos();
        (
            -cos_phi * rx * sin - sin_phi * ry * cos,
            -sin_phi * rx * sin + cos_phi * ry * cos,
        )
    };

    for sıra in 0..parça_sayısı {
        let açı0 = başlangıç_açısı + sıra as f32 * parça_açısı;
        let açı1 = açı0 + parça_açısı;
        let p0 = if sıra == 0 {
            başlangıç
        } else {
            nokta(açı0)
        };
        let p1 = if sıra + 1 == parça_sayısı {
            uç
        } else {
            nokta(açı1)
        };
        let d0 = türev(açı0);
        let d1 = türev(açı1);
        let alfa = 4.0 / 3.0 * (parça_açısı / 4.0).tan();
        yol.kübik(
            (p0.0 + alfa * d0.0, p0.1 + alfa * d0.1),
            (p1.0 - alfa * d1.0, p1.1 - alfa * d1.1),
            p1,
        );
    }
}

impl Yol {
    /// SVG `d`/ECharts `pathData` metnini ortak yüzey yoluna çözer.
    ///
    /// `M/L/H/V/C/S/Q/T/A/Z` komutlarının mutlak ve göreli biçimleri,
    /// tekrarlanan parametre grupları ve üslü sayı yazımı desteklenir.
    pub fn svg_path_data(veri: &str) -> Result<Self, SvgYolHatası> {
        let mut çözücü = Çözücü::yeni(veri);
        let mut yol = Yol::yeni();
        let mut etkin_komut = None;
        let mut geçerli = (0.0, 0.0);
        let mut alt_yol_başı = (0.0, 0.0);
        let mut son_kübik_kontrol = None;
        let mut son_karesel_kontrol = None;

        while !çözücü.bitti_mi() {
            if çözücü.komut_var_mı() {
                etkin_komut = çözücü.komut();
            }
            let komut = etkin_komut.ok_or_else(|| çözücü.hata("ilk komut `M`/`m` olmalı"))?;
            let göreli = komut.is_ascii_lowercase();
            let büyük = komut.to_ascii_uppercase();
            if yol.komutlar.is_empty() && büyük != 'M' {
                return Err(çözücü.hata("ilk komut `M`/`m` olmalı"));
            }

            if büyük == 'Z' {
                yol.kapat();
                geçerli = alt_yol_başı;
                son_kübik_kontrol = None;
                son_karesel_kontrol = None;
                etkin_komut = None;
                continue;
            }

            let mut grup_sayısı = 0usize;
            while çözücü.sayı_var_mı() {
                grup_sayısı += 1;
                match büyük {
                    'M' => {
                        let nokta = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        if grup_sayısı == 1 {
                            yol.taşı(nokta);
                            alt_yol_başı = nokta;
                        } else {
                            yol.çiz(nokta);
                        }
                        geçerli = nokta;
                        son_kübik_kontrol = None;
                        son_karesel_kontrol = None;
                    }
                    'L' => {
                        let nokta = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        yol.çiz(nokta);
                        geçerli = nokta;
                        son_kübik_kontrol = None;
                        son_karesel_kontrol = None;
                    }
                    'H' => {
                        let x = çözücü.sayı()?;
                        geçerli.0 = if göreli { geçerli.0 + x } else { x };
                        yol.çiz(geçerli);
                        son_kübik_kontrol = None;
                        son_karesel_kontrol = None;
                    }
                    'V' => {
                        let y = çözücü.sayı()?;
                        geçerli.1 = if göreli { geçerli.1 + y } else { y };
                        yol.çiz(geçerli);
                        son_kübik_kontrol = None;
                        son_karesel_kontrol = None;
                    }
                    'C' => {
                        let k1 = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        let k2 = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        let uç = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        yol.kübik(k1, k2, uç);
                        geçerli = uç;
                        son_kübik_kontrol = Some(k2);
                        son_karesel_kontrol = None;
                    }
                    'S' => {
                        let k1 =
                            son_kübik_kontrol.map_or(geçerli, |kontrol| yansıt(kontrol, geçerli));
                        let k2 = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        let uç = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        yol.kübik(k1, k2, uç);
                        geçerli = uç;
                        son_kübik_kontrol = Some(k2);
                        son_karesel_kontrol = None;
                    }
                    'Q' => {
                        let kontrol = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        let uç = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        kareseli_kübiğe_çevir(&mut yol, geçerli, kontrol, uç);
                        geçerli = uç;
                        son_kübik_kontrol = None;
                        son_karesel_kontrol = Some(kontrol);
                    }
                    'T' => {
                        let kontrol =
                            son_karesel_kontrol.map_or(geçerli, |önceki| yansıt(önceki, geçerli));
                        let uç = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        kareseli_kübiğe_çevir(&mut yol, geçerli, kontrol, uç);
                        geçerli = uç;
                        son_kübik_kontrol = None;
                        son_karesel_kontrol = Some(kontrol);
                    }
                    'A' => {
                        let rx = çözücü.sayı()?;
                        let ry = çözücü.sayı()?;
                        let dönüş = çözücü.sayı()?;
                        let büyük_yay = çözücü.bayrak()?;
                        let süpürme = çözücü.bayrak()?;
                        let uç = göreli_nokta(çözücü.çift()?, geçerli, göreli);
                        eliptik_yayı_ekle(
                            &mut yol,
                            geçerli,
                            rx,
                            ry,
                            dönüş,
                            büyük_yay,
                            süpürme,
                            uç,
                        );
                        geçerli = uç;
                        son_kübik_kontrol = None;
                        son_karesel_kontrol = None;
                    }
                    _ => {
                        return Err(çözücü.hata(format!("`{komut}` komutu desteklenmiyor")));
                    }
                }
            }
            if grup_sayısı == 0 {
                return Err(çözücü.hata(format!("`{komut}` komutunun parametreleri eksik")));
            }
            // `M` sonrasındaki örtük çiftler `L` olarak yorumlanır.
            if büyük == 'M' {
                etkin_komut = Some(if göreli { 'l' } else { 'L' });
            }
        }
        Ok(yol)
    }

    /// Kübik Bezier uç noktalarıyla birlikte iç ekstremumları da hesaba
    /// katan sınır kutusu. SVG `pathData` şekillerini hedef kutuya sığdırmak
    /// için kontrol noktalarını kullanan kaba kutudan daha uygundur.
    pub fn kesin_sınır_kutusu(&self) -> Option<Dikdörtgen> {
        let mut en_küçük = (f32::INFINITY, f32::INFINITY);
        let mut en_büyük = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut geçerli = (0.0, 0.0);
        let mut alt_yol_başı = (0.0, 0.0);

        for komut in &self.komutlar {
            match *komut {
                YolKomutu::Taşı(nokta) => {
                    sınırı_genişlet(&mut en_küçük, &mut en_büyük, nokta);
                    geçerli = nokta;
                    alt_yol_başı = nokta;
                }
                YolKomutu::Çiz(nokta) => {
                    sınırı_genişlet(&mut en_küçük, &mut en_büyük, geçerli);
                    sınırı_genişlet(&mut en_küçük, &mut en_büyük, nokta);
                    geçerli = nokta;
                }
                YolKomutu::Kübik { k1, k2, uç } => {
                    kübik_sınırı_genişlet(&mut en_küçük, &mut en_büyük, geçerli, k1, k2, uç);
                    geçerli = uç;
                }
                YolKomutu::Yay {
                    yarıçap,
                    büyük_yay,
                    süpürme,
                    uç,
                } => {
                    let mut kübikler = Yol::yeni();
                    kübikler.taşı(geçerli);
                    eliptik_yayı_ekle(
                        &mut kübikler,
                        geçerli,
                        yarıçap,
                        yarıçap,
                        0.0,
                        büyük_yay,
                        süpürme,
                        uç,
                    );
                    if let Some(kutu) = kübikler.kesin_sınır_kutusu() {
                        sınırı_genişlet(&mut en_küçük, &mut en_büyük, (kutu.x, kutu.y));
                        sınırı_genişlet(&mut en_küçük, &mut en_büyük, (kutu.sağ(), kutu.alt()));
                    }
                    geçerli = uç;
                }
                YolKomutu::Kapat => {
                    sınırı_genişlet(&mut en_küçük, &mut en_büyük, alt_yol_başı);
                    geçerli = alt_yol_başı;
                }
            }
        }

        en_küçük.0.is_finite().then(|| {
            Dikdörtgen::yeni(
                en_küçük.0,
                en_küçük.1,
                en_büyük.0 - en_küçük.0,
                en_büyük.1 - en_küçük.1,
            )
        })
    }
}

fn sınırı_genişlet(
    en_küçük: &mut (f32, f32), en_büyük: &mut (f32, f32), nokta: (f32, f32)
) {
    en_küçük.0 = en_küçük.0.min(nokta.0);
    en_küçük.1 = en_küçük.1.min(nokta.1);
    en_büyük.0 = en_büyük.0.max(nokta.0);
    en_büyük.1 = en_büyük.1.max(nokta.1);
}

fn kübik_değer(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

fn kübik_ekstremumlar(p0: f32, p1: f32, p2: f32, p3: f32) -> [Option<f32>; 2] {
    let a = 3.0 * (-p0 + 3.0 * p1 - 3.0 * p2 + p3);
    let b = 6.0 * (p0 - 2.0 * p1 + p2);
    let c = 3.0 * (p1 - p0);
    if a.abs() <= 1e-8 {
        if b.abs() <= 1e-8 {
            return [None, None];
        }
        let t = -c / b;
        return [((0.0..1.0).contains(&t)).then_some(t), None];
    }
    let ayıraç = b * b - 4.0 * a * c;
    if ayıraç < 0.0 {
        return [None, None];
    }
    let kök = ayıraç.sqrt();
    let t0 = (-b + kök) / (2.0 * a);
    let t1 = (-b - kök) / (2.0 * a);
    [
        ((0.0..1.0).contains(&t0)).then_some(t0),
        ((0.0..1.0).contains(&t1)).then_some(t1),
    ]
}

fn kübik_sınırı_genişlet(
    en_küçük: &mut (f32, f32),
    en_büyük: &mut (f32, f32),
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
) {
    sınırı_genişlet(en_küçük, en_büyük, p0);
    sınırı_genişlet(en_küçük, en_büyük, p3);
    for t in kübik_ekstremumlar(p0.0, p1.0, p2.0, p3.0)
        .into_iter()
        .flatten()
    {
        sınırı_genişlet(
            en_küçük,
            en_büyük,
            (
                kübik_değer(p0.0, p1.0, p2.0, p3.0, t),
                kübik_değer(p0.1, p1.1, p2.1, p3.1, t),
            ),
        );
    }
    for t in kübik_ekstremumlar(p0.1, p1.1, p2.1, p3.1)
        .into_iter()
        .flatten()
    {
        sınırı_genişlet(
            en_küçük,
            en_büyük,
            (
                kübik_değer(p0.0, p1.0, p2.0, p3.0, t),
                kübik_değer(p0.1, p1.1, p2.1, p3.1, t),
            ),
        );
    }
}

#[cfg(test)]
mod testler {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used)]

    use super::*;

    #[test]
    fn göreli_ve_örtük_svg_komutlarını_çözer() {
        let yol = Yol::svg_path_data("M 10 10 l5-2 3 4 h2 v-3 z").unwrap();
        assert_eq!(
            yol.komutlar,
            vec![
                YolKomutu::Taşı((10.0, 10.0)),
                YolKomutu::Çiz((15.0, 8.0)),
                YolKomutu::Çiz((18.0, 12.0)),
                YolKomutu::Çiz((20.0, 12.0)),
                YolKomutu::Çiz((20.0, 9.0)),
                YolKomutu::Kapat,
            ]
        );
    }

    #[test]
    fn yumuşak_ve_karesel_komutları_kübiğe_çevirir() {
        let yol = Yol::svg_path_data("M0 0 C1 2 3 2 4 0 S7-2 8 0 Q9 2 10 0 T12 0").unwrap();
        assert_eq!(yol.komutlar.len(), 5);
        assert!(matches!(
            yol.komutlar[2],
            YolKomutu::Kübik {
                k1: (5.0, -2.0),
                ..
            }
        ));
        assert!(matches!(
            yol.komutlar[4],
            YolKomutu::Kübik {
                uç: (12.0, 0.0),
                ..
            }
        ));
    }

    #[test]
    fn eliptik_yayı_kübik_parçalara_çevirir() {
        let yol = Yol::svg_path_data("M10 20 A30 15 25 0 1 80 40").unwrap();
        assert!(yol.komutlar.len() >= 2);
        assert!(
            yol.komutlar
                .iter()
                .skip(1)
                .all(|komut| matches!(komut, YolKomutu::Kübik { .. }))
        );
        assert!(matches!(
            yol.komutlar.last(),
            Some(YolKomutu::Kübik { uç, .. }) if *uç == (80.0, 40.0)
        ));
    }

    #[test]
    fn kesin_kutu_kontrol_noktalarını_değil_eğriyi_ölçer() {
        let yol = Yol::svg_path_data("M0 0 C0 100 10 100 10 0").unwrap();
        let kaba = yol.sınır_kutusu().unwrap();
        let kesin = yol.kesin_sınır_kutusu().unwrap();
        assert_eq!(kaba.yükseklik, 100.0);
        assert!((kesin.yükseklik - 75.0).abs() < 1e-4);
    }

    #[test]
    fn eksik_grup_açıklamalı_hata_verir() {
        let hata = Yol::svg_path_data("M0 0 L10").unwrap_err();
        assert!(hata.ayrıntı.contains("sayı"));
    }
}
