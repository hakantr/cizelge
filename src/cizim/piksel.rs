//! Piksel yüzeyi — [`ÇizimYüzeyi`]nin rasterleştirici gerçeklemesi ve PNG
//! dışa aktarımı (`toolbox.feature.saveAsImage` png biçimi).
//!
//! Rasterleştirme `tiny-skia` (BSD-3-Clause) iledir; yazılar, `fontdb`
//! (MIT) ile bulunan sistem yazı tipinden `ab_glyph` (Apache-2.0) ile
//! biçimlenir. Yazı tipi bulunamazsa yazılar atlanır, ölçüm belirlenimci
//! yaklaşıkla sürer — hiçbir yol panik üretmez.

use std::sync::Arc;

use ab_glyph::{Font, FontVec, ScaleFont};
use tiny_skia as ts;

use crate::cizim::yuzey::{DikeyHiza, YatayHiza, Yol, YolKomutu, ÇizimYüzeyi};
use crate::hata::{BilesenHatasi, BilesenTanisi};
use crate::koordinat::Dikdörtgen;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

/// Sistem yazı tipleri (normal + kalın).
struct YazıTakımı {
    normal: Arc<FontVec>,
    kalın: Arc<FontVec>,
}

/// Yazı tipini fontdb ile sistemden yükler; bulunamazsa `None`.
fn yazı_takımı_yükle() -> Option<YazıTakımı> {
    let mut veritabanı = fontdb::Database::new();
    veritabanı.load_system_fonts();
    let yüz_yükle = |ağırlık: fontdb::Weight| -> Option<Arc<FontVec>> {
        let kimlik = veritabanı.query(&fontdb::Query {
            families: &[fontdb::Family::SansSerif],
            weight: ağırlık,
            ..fontdb::Query::default()
        })?;
        veritabanı
            .with_face_data(kimlik, |veri, indeks| {
                FontVec::try_from_vec_and_index(veri.to_vec(), indeks).ok()
            })
            .flatten()
            .map(Arc::new)
    };
    let normal = yüz_yükle(fontdb::Weight::NORMAL)?;
    let kalın = yüz_yükle(fontdb::Weight::BOLD).unwrap_or_else(|| normal.clone());
    Some(YazıTakımı { normal, kalın })
}

/// Raster çizim yüzeyi. Koordinatlar mantıksaldır; `ölçek` (pixelRatio)
/// fiziksel çözünürlüğü belirler.
pub struct PikselYüzeyi {
    harita: ts::Pixmap,
    mantıksal: (f32, f32),
    ölçek: f32,
    /// Etkin kırpma maskesi (fiziksel çözünürlükte).
    kırpma: Option<ts::Mask>,
    yazılar: Option<YazıTakımı>,
    /// Çizim sırasında yutulan kurtarılabilir sorunlar.
    tanılar: Vec<BilesenTanisi>,
}

impl PikselYüzeyi {
    /// Yüzey kurar; `ölçek` fiziksel/mantıksal piksel oranıdır (≥ 0.1).
    pub fn yeni(genişlik: f32, yükseklik: f32, ölçek: f32) -> Result<Self, BilesenHatasi> {
        let ölçek = if ölçek.is_finite() && ölçek >= 0.1 { ölçek } else { 1.0 };
        let fg = (genişlik * ölçek).round().max(1.0) as u32;
        let fy = (yükseklik * ölçek).round().max(1.0) as u32;
        let mut harita =
            ts::Pixmap::new(fg, fy).ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "piksel_yüzeyi.boyut",
                ayrıntı: format!("{fg}x{fy} piksel haritası kurulamadı"),
            })?;
        // PNG şeffaf kalmasın: beyaz taban (ECharts `saveAsImage`
        // `backgroundColor` öntanımlısı); koyu tema zemini üstüne boyanır.
        harita.fill(ts::Color::WHITE);
        Ok(PikselYüzeyi {
            harita,
            mantıksal: (genişlik, yükseklik),
            ölçek,
            kırpma: None,
            yazılar: yazı_takımı_yükle(),
            tanılar: Vec::new(),
        })
    }

    /// Çizim sırasında biriken tanılar.
    pub fn tanılar(&self) -> &[BilesenTanisi] {
        &self.tanılar
    }

    /// PNG baytlarına kodlar.
    pub fn png_kodla(&self) -> Result<Vec<u8>, BilesenHatasi> {
        self.harita
            .encode_png()
            .map_err(|hata| BilesenHatasi::GeçersizSeçenek {
                alan: "piksel_yüzeyi.png",
                ayrıntı: format!("PNG kodlanamadı: {hata}"),
            })
    }

    fn dönüşüm(&self) -> ts::Transform {
        ts::Transform::from_scale(self.ölçek, self.ölçek)
    }

    fn doldur(&mut self, yol: &ts::Path, boya: &ts::Paint) {
        self.harita.fill_path(
            yol,
            boya,
            ts::FillRule::Winding,
            self.dönüşüm(),
            self.kırpma.as_ref(),
        );
    }

    fn tanı(&mut self, bileşen: &'static str, ayrıntı: String) {
        self.tanılar.push(BilesenTanisi::yeni(
            bileşen,
            BilesenHatasi::GeçersizSeçenek { alan: "piksel_yüzeyi", ayrıntı },
        ));
        let _ = bileşen;
    }
}

/// `Renk` → tiny-skia rengi.
fn renk_çevir(renk: Renk) -> ts::Color {
    ts::Color::from_rgba(
        renk.kırmızı.clamp(0.0, 1.0),
        renk.yeşil.clamp(0.0, 1.0),
        renk.mavi.clamp(0.0, 1.0),
        renk.alfa.clamp(0.0, 1.0),
    )
    .unwrap_or(ts::Color::BLACK)
}

/// SVG uç-nokta yayını çizgi parçalarına örnekler (dairesel, tek yarıçap).
fn yay_örnekle(
    kurucu: &mut ts::PathBuilder,
    başlangıç: (f32, f32),
    yarıçap: f32,
    büyük_yay: bool,
    süpürme: bool,
    uç: (f32, f32),
) {
    let (x0, y0) = başlangıç;
    let (x1, y1) = uç;
    let dx = x1 - x0;
    let dy = y1 - y0;
    let uzaklık = (dx * dx + dy * dy).sqrt();
    if uzaklık < 1e-6 {
        return;
    }
    // Yarıçap kirişin yarısından küçükse SVG kuralınca büyütülür.
    let yarıçap = yarıçap.max(uzaklık / 2.0);
    let orta = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let yükselti = (yarıçap * yarıçap - (uzaklık / 2.0) * (uzaklık / 2.0)).max(0.0).sqrt();
    // Kirişe dik birim vektör; merkez seçimi SVG F.6.5 işaret kuralıyla.
    let (nx, ny) = (-dy / uzaklık, dx / uzaklık);
    let işaret = if büyük_yay != süpürme { 1.0 } else { -1.0 };
    let merkez = (orta.0 + nx * yükselti * işaret, orta.1 + ny * yükselti * işaret);

    let açı0 = (y0 - merkez.1).atan2(x0 - merkez.0);
    let açı1 = (y1 - merkez.1).atan2(x1 - merkez.0);
    let tau = std::f32::consts::TAU;
    let mut açıklık = açı1 - açı0;
    if süpürme {
        // Ekran koordinatında pozitif açı saat yönüdür.
        if açıklık < 0.0 {
            açıklık += tau;
        }
    } else if açıklık > 0.0 {
        açıklık -= tau;
    }
    let adım_sayısı = ((açıklık.abs() / (std::f32::consts::PI / 24.0)).ceil() as usize)
        .clamp(2, 128);
    for i in 1..=adım_sayısı {
        let açı = açı0 + açıklık * (i as f32 / adım_sayısı as f32);
        kurucu.line_to(merkez.0 + yarıçap * açı.cos(), merkez.1 + yarıçap * açı.sin());
    }
}

/// [`Yol`] → tiny-skia yolu (mantıksal koordinatta).
fn yol_çevir(yol: &Yol) -> Option<ts::Path> {
    let mut kurucu = ts::PathBuilder::new();
    let mut geçerli = (0.0f32, 0.0f32);
    let mut başlanmış = false;
    for komut in &yol.komutlar {
        match *komut {
            YolKomutu::Taşı(n) => {
                kurucu.move_to(n.0, n.1);
                geçerli = n;
                başlanmış = true;
            }
            YolKomutu::Çiz(n) => {
                if !başlanmış {
                    kurucu.move_to(n.0, n.1);
                    başlanmış = true;
                } else {
                    kurucu.line_to(n.0, n.1);
                }
                geçerli = n;
            }
            YolKomutu::Kübik { k1, k2, uç } => {
                kurucu.cubic_to(k1.0, k1.1, k2.0, k2.1, uç.0, uç.1);
                geçerli = uç;
            }
            YolKomutu::Yay { yarıçap, büyük_yay, süpürme, uç } => {
                yay_örnekle(&mut kurucu, geçerli, yarıçap, büyük_yay, süpürme, uç);
                geçerli = uç;
            }
            YolKomutu::Kapat => kurucu.close(),
        }
    }
    kurucu.finish()
}

/// Dolgu → tiny-skia boyası. Gradyan uçları, yolun sınır kutusunun birim
/// uzayındadır (ECharts `LinearGradient`/`RadialGradient` sözleşmesi).
fn boya_çevir<'a>(dolgu: &Dolgu, sınır: Option<Dikdörtgen>) -> Option<ts::Paint<'a>> {
    let mut boya = ts::Paint { anti_alias: true, ..ts::Paint::default() };
    match dolgu {
        Dolgu::Düz(renk) => {
            boya.set_color(renk_çevir(*renk));
            Some(boya)
        }
        Dolgu::DoğrusalGradyan { x, y, x2, y2, duraklar } => {
            let s = sınır?;
            let noktalar = (
                ts::Point::from_xy(s.x + x * s.genişlik, s.y + y * s.yükseklik),
                ts::Point::from_xy(s.x + x2 * s.genişlik, s.y + y2 * s.yükseklik),
            );
            let duraklar: Vec<ts::GradientStop> = duraklar
                .iter()
                .map(|d| ts::GradientStop::new(d.konum.clamp(0.0, 1.0), renk_çevir(d.renk)))
                .collect();
            boya.shader = ts::LinearGradient::new(
                noktalar.0,
                noktalar.1,
                duraklar,
                ts::SpreadMode::Pad,
                ts::Transform::identity(),
            )?;
            Some(boya)
        }
        Dolgu::RadyalGradyan { x, y, yarıçap, duraklar } => {
            let s = sınır?;
            let merkez =
                ts::Point::from_xy(s.x + x * s.genişlik, s.y + y * s.yükseklik);
            let duraklar: Vec<ts::GradientStop> = duraklar
                .iter()
                .map(|d| ts::GradientStop::new(d.konum.clamp(0.0, 1.0), renk_çevir(d.renk)))
                .collect();
            boya.shader = ts::RadialGradient::new(
                merkez,
                merkez,
                yarıçap * s.genişlik.max(s.yükseklik),
                duraklar,
                ts::SpreadMode::Pad,
                ts::Transform::identity(),
            )?;
            Some(boya)
        }
    }
}

/// Çizgi türünden vuruş deseni (Çizici ile aynı oranlar).
fn vuruş_yap(kalınlık: f32, tür: ÇizgiTürü) -> ts::Stroke {
    let kalınlık = kalınlık.max(0.1);
    ts::Stroke {
        width: kalınlık,
        dash: match tür {
            ÇizgiTürü::Düz => None,
            ÇizgiTürü::Kesikli => {
                ts::StrokeDash::new(vec![4.0 * kalınlık, 2.0 * kalınlık], 0.0)
            }
            ÇizgiTürü::Noktalı => ts::StrokeDash::new(vec![kalınlık, kalınlık], 0.0),
        },
        ..ts::Stroke::default()
    }
}

impl ÇizimYüzeyi for PikselYüzeyi {
    fn genişlik(&self) -> f32 {
        self.mantıksal.0
    }

    fn yükseklik(&self) -> f32 {
        self.mantıksal.1
    }

    fn yol_doldur(&mut self, yol: &Yol, dolgu: &Dolgu) {
        let Some(ts_yolu) = yol_çevir(yol) else { return };
        let Some(boya) = boya_çevir(dolgu, yol.sınır_kutusu()) else { return };
        self.doldur(&ts_yolu, &boya);
    }

    fn yol_çiz(&mut self, yol: &Yol, kalınlık: f32, renk: Renk, tür: ÇizgiTürü) {
        let Some(ts_yolu) = yol_çevir(yol) else { return };
        let mut boya = ts::Paint { anti_alias: true, ..ts::Paint::default() };
        boya.set_color(renk_çevir(renk));
        let vuruş = vuruş_yap(kalınlık, tür);
        self.harita.stroke_path(
            &ts_yolu,
            &boya,
            &vuruş,
            self.dönüşüm(),
            self.kırpma.as_ref(),
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
        // Köşe yarıçaplı dikdörtgen yolu (çeyrek yaylar).
        let en_büyük = (d.genişlik.min(d.yükseklik)) / 2.0;
        let [sü, saü, saa, sa] = yarıçap.map(|y| y.clamp(0.0, en_büyük));
        let mut yol = Yol::yeni();
        yol.taşı((d.x + sü, d.y));
        yol.çiz((d.sağ() - saü, d.y));
        if saü > 0.0 {
            yol.yay(saü, false, true, (d.sağ(), d.y + saü));
        }
        yol.çiz((d.sağ(), d.alt() - saa));
        if saa > 0.0 {
            yol.yay(saa, false, true, (d.sağ() - saa, d.alt()));
        }
        yol.çiz((d.x + sa, d.alt()));
        if sa > 0.0 {
            yol.yay(sa, false, true, (d.x, d.alt() - sa));
        }
        yol.çiz((d.x, d.y + sü));
        if sü > 0.0 {
            yol.yay(sü, false, true, (d.x + sü, d.y));
        }
        yol.kapat();

        let Some(ts_yolu) = yol_çevir(&yol) else { return };
        if let Some(boya) = boya_çevir(dolgu, Some(d)) {
            self.doldur(&ts_yolu, &boya);
        }
        if let Some((kalınlık, renk)) = kenarlık
            && kalınlık > 0.0 {
                let mut boya = ts::Paint { anti_alias: true, ..ts::Paint::default() };
                boya.set_color(renk_çevir(renk));
                let vuruş = vuruş_yap(kalınlık, ÇizgiTürü::Düz);
                self.harita.stroke_path(
                    &ts_yolu,
                    &boya,
                    &vuruş,
                    self.dönüşüm(),
                    self.kırpma.as_ref(),
                );
            }
    }

    fn gölge(&mut self, d: Dikdörtgen, yarıçap: f32, renk: Renk, bulanıklık: f32) {
        // Yumuşatma: dışa doğru genişleyen, gitgide soluklaşan katmanlar.
        let katman_sayısı = 5usize;
        for i in 0..katman_sayısı {
            let taşma = bulanıklık * (i as f32 + 1.0) / katman_sayısı as f32;
            let katman = Dikdörtgen::yeni(
                d.x - taşma / 2.0,
                d.y - taşma / 2.0 + 1.0,
                d.genişlik + taşma,
                d.yükseklik + taşma,
            );
            let soluk = renk.opaklık(1.0 / (katman_sayısı as f32 * 1.5));
            self.dikdörtgen(
                katman,
                &Dolgu::Düz(soluk),
                [yarıçap + taşma / 2.0; 4],
                None,
            );
        }
    }

    fn kırpılı(&mut self, d: Dikdörtgen, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi)) {
        let önceki = self.kırpma.take();
        let yeni = ts::Mask::new(self.harita.width(), self.harita.height());
        match (yeni, ts::Rect::from_xywh(d.x, d.y, d.genişlik.max(0.1), d.yükseklik.max(0.1))) {
            (Some(mut maske), Some(dikdörtgen)) => {
                let yol = ts::PathBuilder::from_rect(dikdörtgen);
                maske.fill_path(&yol, ts::FillRule::Winding, true, self.dönüşüm());
                self.kırpma = Some(maske);
            }
            _ => {
                self.tanı("kırpılı", format!("kırpma maskesi kurulamadı: {d:?}"));
            }
        }
        işlev(self.olarak());
        self.kırpma = önceki;
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
        let (genişlik, yükseklik) = self.yazı_ölç(metin, boyut);
        let Some(yazılar) = &self.yazılar else {
            return (genişlik, yükseklik);
        };
        let yazı_tipi =
            if kalın { yazılar.kalın.clone() } else { yazılar.normal.clone() };
        let ölçekli = yazı_tipi.as_scaled(ab_glyph::PxScale::from(boyut * self.ölçek));

        let x0 = match yatay {
            YatayHiza::Sol => konum.0,
            YatayHiza::Orta => konum.0 - genişlik / 2.0,
            YatayHiza::Sağ => konum.0 - genişlik,
        } * self.ölçek;
        let üst = match dikey {
            DikeyHiza::Üst => konum.1,
            DikeyHiza::Orta => konum.1 - yükseklik / 2.0,
            DikeyHiza::Alt => konum.1 - yükseklik,
        } * self.ölçek;
        let satır = ölçekli.ascent() - ölçekli.descent();
        // Kutunun içinde dikey ortalanmış taban çizgisi.
        let taban_y = üst + (yükseklik * self.ölçek - satır) / 2.0 + ölçekli.ascent();

        let (kırmızı, yeşil, mavi, alfa) = (
            (renk.kırmızı.clamp(0.0, 1.0) * 255.0) as u16,
            (renk.yeşil.clamp(0.0, 1.0) * 255.0) as u16,
            (renk.mavi.clamp(0.0, 1.0) * 255.0) as u16,
            renk.alfa.clamp(0.0, 1.0),
        );
        let harita_g = self.harita.width() as i32;
        let harita_y = self.harita.height() as i32;

        let mut kalem = x0;
        let mut önceki: Option<ab_glyph::GlyphId> = None;
        for karakter in metin.chars() {
            let kimlik = ölçekli.glyph_id(karakter);
            if let Some(ö) = önceki {
                kalem += ölçekli.kern(ö, kimlik);
            }
            let konumlu = kimlik
                .with_scale_and_position(ölçekli.scale(), ab_glyph::point(kalem, taban_y));
            kalem += ölçekli.h_advance(kimlik);
            önceki = Some(kimlik);
            let Some(dış_hat) = yazı_tipi.outline_glyph(konumlu) else { continue };
            let sınırlar = dış_hat.px_bounds();
            // Piksel örtüsünü elle harmanla (kaynak-üstte).
            let kırpma_verisi = self.kırpma.as_ref().map(|m| m.data().to_vec());
            let veriler = self.harita.pixels_mut();
            dış_hat.draw(|gx, gy, örtü| {
                let px = sınırlar.min.x as i32 + gx as i32;
                let py = sınırlar.min.y as i32 + gy as i32;
                if px < 0 || py < 0 || px >= harita_g || py >= harita_y {
                    return;
                }
                let dizin = (py * harita_g + px) as usize;
                let mut a = örtü.clamp(0.0, 1.0) * alfa;
                if let Some(maske) = &kırpma_verisi {
                    let m = maske.get(dizin).copied().unwrap_or(0) as f32 / 255.0;
                    a *= m;
                }
                if a <= 0.0 {
                    return;
                }
                let Some(piksel) = veriler.get_mut(dizin) else { return };
                let eski = *piksel;
                let kalan = 1.0 - a;
                let yeni = ts::PremultipliedColorU8::from_rgba(
                    ((kırmızı as f32 * a) + eski.red() as f32 * kalan) as u8,
                    ((yeşil as f32 * a) + eski.green() as f32 * kalan) as u8,
                    ((mavi as f32 * a) + eski.blue() as f32 * kalan) as u8,
                    ((255.0 * a) + eski.alpha() as f32 * kalan) as u8,
                );
                if let Some(yeni) = yeni {
                    *piksel = yeni;
                }
            });
        }
        (genişlik, yükseklik)
    }

    fn yazı_ölç(&self, metin: &str, boyut: f32) -> (f32, f32) {
        if let Some(yazılar) = &self.yazılar {
            let ölçekli = yazılar
                .normal
                .as_scaled(ab_glyph::PxScale::from(boyut * self.ölçek));
            let mut genişlik = 0.0f32;
            let mut önceki: Option<ab_glyph::GlyphId> = None;
            for karakter in metin.chars() {
                let kimlik = ölçekli.glyph_id(karakter);
                if let Some(ö) = önceki {
                    genişlik += ölçekli.kern(ö, kimlik);
                }
                genişlik += ölçekli.h_advance(kimlik);
                önceki = Some(kimlik);
            }
            (genişlik / self.ölçek, boyut)
        } else {
            // Yazı tipi yok: KayıtYüzeyi ile aynı belirlenimci yaklaşık.
            (metin.chars().count() as f32 * boyut * 0.6, boyut)
        }
    }

    fn olarak(&mut self) -> &mut dyn ÇizimYüzeyi {
        self
    }
}

/// Grafiği verilen mantıksal boyutta rasterleştirip PNG baytları üretir.
/// `ölçek` fiziksel/mantıksal piksel oranıdır (`saveAsImage.pixelRatio`).
pub fn png_dışa_aktar(
    seçenekler: &GrafikSeçenekleri,
    genişlik: f32,
    yükseklik: f32,
    ölçek: f32,
) -> Result<Vec<u8>, BilesenHatasi> {
    let mut yüzey = PikselYüzeyi::yeni(genişlik, yükseklik, ölçek)?;
    crate::cizim::gorunum::grafiği_boya(
        &mut yüzey,
        seçenekler,
        &crate::cizim::gorunum::BoyamaGirdisi::default(),
    );
    yüzey.png_kodla()
}
