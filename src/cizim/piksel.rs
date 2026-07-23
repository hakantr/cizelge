//! Piksel yüzeyi — [`ÇizimYüzeyi`]nin rasterleştirici gerçeklemesi ve PNG
//! dışa aktarımı (`toolbox.feature.saveAsImage` png biçimi).
//!
//! Rasterleştirme `tiny-skia` (BSD-3-Clause) iledir; yazılar, `fontdb`
//! (MIT) ile bulunan sistem yazı tipinden `ab_glyph` (Apache-2.0) ile
//! biçimlenir. Yazı tipi bulunamazsa yazılar atlanır, ölçüm belirlenimci
//! yaklaşıkla sürer — hiçbir yol panik üretmez.

use std::sync::Arc;

use ab_glyph::{Font, FontVec, OutlineCurve, ScaleFont};
use tiny_skia as ts;

use crate::cizim::donusum::AfinMatris;
use crate::cizim::yuzey::{
    DikeyHiza, YatayHiza, Yol, YolKomutu, daire_yolu, ÇizimYüzeyi, çizgi_deseni_normalleştir,
};
use crate::hata::{BilesenHatasi, BilesenTanisi};
use crate::koordinat::Dikdörtgen;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

// `ab_glyph::PxScale` cap-height değil fontun iç em yüksekliğidir. Chromium
// canvas'ın CSS pikseliyle aynı görünen Arial gliflerini üretmek için em
// ölçeği 13.5/12 oranında düzeltilir; yerleşim hâlâ ECharts'taki 12 px metrik
// kutusunu kullanır.
const YAZI_RASTER_ORANI: f32 = 13.42 / 12.0;
// ab_glyph'in em kutusu taban çizgisini Chromium canvas'a göre iki mantıksal
// piksel aşağıda bırakır. Bütün hizalarda aynı taban düzeltmesi gerekir.
const YAZI_TABAN_DÜZELTMESİ: f32 = -1.3;
// Affine metin tiny-skia ile doğrudan glif dış hatlarından rasterlenir;
// hint'li ab_glyph maskesinin taban düzeltmesini aynen paylaşmaz.
const DÖNÜŞÜMLÜ_YAZI_TABAN_DÜZELTMESİ: f32 = -1.3;
// Chromium/Skia glif maskesi küçük Arial'de düşük örtülü kenar piksellerini
// ab_glyph'ten daha görünür tutarken gövdeyi yumuşakça doyuma taşır. Aynı
// dış hatta uygulanan aktarım eğrisi bu gri-ölçek kapsama davranışını taşır.
const YAZI_KAPSAMA_ÇARPANI: f32 = 1.15;
const YAZI_KAPSAMA_KUVVETİ: f32 = 0.8;
// Dönüşümlü dış hatlar tiny-skia'da CoreText/Skia'dan daha ince kalır.
// İkinci örtü geçişi, geometriyi büyütmeden aynı kenar yoğunluğunu üretir.
const DÖNÜŞÜMLÜ_YAZI_EK_OPAKLIĞI: f32 = 0.4;
const AÇIK_DÖNÜŞÜMLÜ_YAZI_EK_OPAKLIĞI: f32 = 1.0;
// Zrender'ın `strokeFirst` metni, salt dolgu metninden biraz farklı bir
// piksel merkezine oturur. Konturlu glif yolunu yerel eksende aynı merkeze
// taşımak için gereken sabit alt-piksel taban eki.
const KONTURLU_YAZI_TABAN_EKİ: f32 = 0.12;
// Canvas/Skia'nın kesikli 1 CSS px metin konturu tiny-skia'nın geometrik
// kesikli vuruşundan daha düşük örtü üretir. Yalnız Graphic dash izini,
// mevcut düz kontur rasterini değiştirmeden kalibre et.
const KONTURLU_YAZI_VURUŞ_ORANI: f32 = 0.72;

/// Sistem yazı tipleri (normal + kalın).
#[derive(Clone)]
struct YazıTakımı {
    normal: Arc<FontVec>,
    kalın: Arc<FontVec>,
    /// `ab_glyph::PxScale` fontun yüz metrik kutusunu kullanır; CSS
    /// `font-size` ise aileye özgü em kutusudur. Arial için yukarıdaki
    /// genel oran, Verdana için daha yüksek em/yüz oranı gerekir.
    raster_oranı: f32,
    /// Chromium/CoreText bir glif Arial'de yoksa metni aynı CSS satırında
    /// uygun sistem sans-serif yüzüyle sürdürür. Başta CJK olmak üzere bu
    /// yüzler aynı geri düşüm zincirini raster çıktıda korur.
    yedekler: Vec<YazıYedeği>,
}

#[derive(Clone)]
struct YazıYedeği {
    normal: Arc<FontVec>,
    kalın: Arc<FontVec>,
}

impl YazıTakımı {
    fn birincil(&self, kalın: bool) -> &Arc<FontVec> {
        if kalın { &self.kalın } else { &self.normal }
    }

    /// Karakteri taşıyan yüz, o yüzün ab_glyph raster oranı ve sabit yüz
    /// kimliği. Latin Arial için Canvas metrik düzeltmesi gerekir; CJK
    /// yüzleri tam em genişliğinde olduğundan CSS pikseliyle 1:1 ölçeklenir.
    fn karakter_yüzü(&self, karakter: char, kalın: bool) -> (&Arc<FontVec>, f32, usize) {
        let birincil = self.birincil(kalın);
        if birincil.glyph_id(karakter).0 != 0 {
            return (birincil, self.raster_oranı, 0);
        }
        for (sıra, yedek) in self.yedekler.iter().enumerate() {
            let yüz = if kalın { &yedek.kalın } else { &yedek.normal };
            if yüz.glyph_id(karakter).0 != 0 {
                return (yüz, 1.0, sıra + 1);
            }
        }
        (birincil, self.raster_oranı, 0)
    }
}

#[cfg(target_os = "macos")]
fn mac_cjk_yazı_tipi(boyut: f32, kalın: bool) -> Option<core_text::font::CTFont> {
    let ad = if kalın {
        "PingFangSC-Semibold"
    } else {
        "PingFangSC-Regular"
    };
    core_text::font::new_from_name(ad, f64::from(boyut)).ok()
}

#[cfg(target_os = "macos")]
fn mac_cjk_glifi(
    yazı_tipi: &core_text::font::CTFont,
    karakter: char,
) -> Option<(core_graphics::font::CGGlyph, f32)> {
    let mut utf16 = [0_u16; 2];
    let birimler = karakter.encode_utf16(&mut utf16);
    if birimler.len() != 1 {
        return None;
    }
    let mut glif = 0_u16;
    // SAFETY: Bir UTF-16 birimi ve bir CGGlyph için geçerli, aynı uzunlukta
    // giriş/çıkış tamponları veriliyor.
    let bulundu = unsafe { yazı_tipi.get_glyphs_for_characters(birimler.as_ptr(), &mut glif, 1) };
    if !bulundu || glif == 0 {
        return None;
    }
    let mut ilerleme = core_graphics::geometry::CGSize::new(0.0, 0.0);
    // SAFETY: Tek glif ve tek CGSize tamponu aynı `count=1` ile sunuluyor.
    unsafe {
        yazı_tipi.get_advances_for_glyphs(
            core_text::font_descriptor::kCTFontOrientationHorizontal,
            &glif,
            &mut ilerleme,
            1,
        );
    }
    Some((glif, ilerleme.width as f32))
}

#[cfg(target_os = "macos")]
fn mac_cjk_dönüşümlü_glif_yolu(
    yazı_tipi: &core_text::font::CTFont,
    glif: core_graphics::font::CGGlyph,
    kalem: f32,
    taban_y: f32,
    dönüşüm: AfinMatris,
) -> Option<ts::Path> {
    use core_graphics::geometry::CGAffineTransform;
    use core_graphics::path::CGPathElementType;
    use std::cell::RefCell;

    let kimlik = CGAffineTransform::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let cg_yolu = yazı_tipi.create_path_for_glyph(glif, &kimlik).ok()?;
    let kurucu = RefCell::new(ts::PathBuilder::new());
    cg_yolu.apply(&|öğe| {
        let nokta = |sıra: usize| {
            let nokta = öğe.points()[sıra];
            dönüşüm.noktayı_dönüştür((kalem + nokta.x as f32, taban_y - nokta.y as f32))
        };
        let mut kurucu = kurucu.borrow_mut();
        match öğe.element_type {
            CGPathElementType::MoveToPoint => {
                let p = nokta(0);
                kurucu.move_to(p.0, p.1);
            }
            CGPathElementType::AddLineToPoint => {
                let p = nokta(0);
                kurucu.line_to(p.0, p.1);
            }
            CGPathElementType::AddQuadCurveToPoint => {
                let p1 = nokta(0);
                let p2 = nokta(1);
                kurucu.quad_to(p1.0, p1.1, p2.0, p2.1);
            }
            CGPathElementType::AddCurveToPoint => {
                let p1 = nokta(0);
                let p2 = nokta(1);
                let p3 = nokta(2);
                kurucu.cubic_to(p1.0, p1.1, p2.0, p2.1, p3.0, p3.1);
            }
            CGPathElementType::CloseSubpath => kurucu.close(),
        }
    });
    kurucu.into_inner().finish()
}

#[cfg(target_os = "macos")]
fn mac_cjk_glif_çiz(
    harita: &mut ts::Pixmap,
    kırpma: Option<&[u8]>,
    yazı_tipi: &core_text::font::CTFont,
    glif: core_graphics::font::CGGlyph,
    x: f32,
    taban_y: f32,
    renk: Renk,
) {
    use core_graphics::base::{kCGBitmapByteOrder32Big, kCGImageAlphaPremultipliedLast};
    use core_graphics::color_space::CGColorSpace;
    use core_graphics::context::CGContext;
    use core_graphics::geometry::CGPoint;

    let genişlik = harita.width() as usize;
    let yükseklik = harita.height() as usize;
    let satır_baytı = genişlik.saturating_mul(4);
    let önceki = kırpma.map(|_| harita.data().to_vec());
    {
        let veri = harita.data_mut();
        let bağlam = CGContext::create_bitmap_context(
            Some(veri.as_mut_ptr().cast()),
            genişlik,
            yükseklik,
            8,
            satır_baytı,
            &CGColorSpace::create_device_rgb(),
            kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big,
        );
        bağlam.set_allows_antialiasing(true);
        bağlam.set_should_antialias(true);
        // Headless Chromium ekran görüntüsü gri tonlu metin örtüsü kullanır;
        // LCD alt-piksel renk saçakları kapatılır, alt-piksel konum korunur.
        bağlam.set_allows_font_smoothing(false);
        bağlam.set_should_smooth_fonts(false);
        bağlam.set_allows_font_subpixel_positioning(true);
        bağlam.set_should_subpixel_position_fonts(true);
        bağlam.set_rgb_fill_color(
            f64::from(renk.kırmızı),
            f64::from(renk.yeşil),
            f64::from(renk.mavi),
            f64::from(renk.alfa),
        );
        let konum = CGPoint::new(f64::from(x), yükseklik as f64 - f64::from(taban_y));
        yazı_tipi.draw_glyphs(&[glif], &[konum], bağlam.clone());
        // Chromium'un renkli/gri CJK örtüsü CoreGraphics'in tek geçişinden
        // daha doygundur; ikinci düşük alfa geçişi bunu yaklaştırır. Saf
        // siyah gliflerde aynı işlem küçük 12 px metni gereğinden kalın
        // yaptığı için tek gri-ton geçişi korunur.
        let saf_siyah =
            renk.kırmızı <= f32::EPSILON && renk.yeşil <= f32::EPSILON && renk.mavi <= f32::EPSILON;
        if !saf_siyah {
            bağlam.set_alpha(0.5);
            yazı_tipi.draw_glyphs(&[glif], &[konum], bağlam);
        }
    }

    if let (Some(maske), Some(önceki)) = (kırpma, önceki) {
        let veri = harita.data_mut();
        for (piksel_sırası, (yeni, eski)) in veri
            .chunks_exact_mut(4)
            .zip(önceki.chunks_exact(4))
            .enumerate()
        {
            let alfa = maske.get(piksel_sırası).copied().unwrap_or(0) as f32 / 255.0;
            if alfa >= 1.0 {
                continue;
            }
            for kanal in 0..4 {
                if let (Some(yeni_kanal), Some(eski_kanal)) = (yeni.get_mut(kanal), eski.get(kanal))
                {
                    *yeni_kanal = (f32::from(*eski_kanal) * (1.0 - alfa)
                        + f32::from(*yeni_kanal) * alfa)
                        .round() as u8;
                }
            }
        }
    }
}

/// Yazı tipini fontdb ile sistemden yükler; bulunamazsa `None`.
fn yazı_takımı_yükle(serif: bool) -> Option<YazıTakımı> {
    yazı_takımı_adla_yükle(None, serif)
}

fn yazı_takımı_adla_yükle(aile: Option<&str>, serif: bool) -> Option<YazıTakımı> {
    let mut veritabanı = fontdb::Database::new();
    veritabanı.load_system_fonts();
    let aileler = if let Some(aile) = aile {
        vec![
            fontdb::Family::Name(aile),
            if serif {
                fontdb::Family::Serif
            } else {
                fontdb::Family::SansSerif
            },
        ]
    } else if serif {
        vec![
            fontdb::Family::Name("Times New Roman"),
            fontdb::Family::Name("Times"),
            fontdb::Family::Serif,
        ]
    } else {
        vec![fontdb::Family::Name("Arial"), fontdb::Family::SansSerif]
    };
    let kimlikten_yüz = |kimlik: fontdb::ID| -> Option<Arc<FontVec>> {
        veritabanı
            .with_face_data(kimlik, |veri, indeks| {
                FontVec::try_from_vec_and_index(veri.to_vec(), indeks).ok()
            })
            .flatten()
            .map(Arc::new)
    };
    let yüz_yükle = |ağırlık: fontdb::Weight| -> Option<(fontdb::ID, Arc<FontVec>)> {
        let kimlik = veritabanı.query(&fontdb::Query {
            // ECharts'ın macOS/Linux Chromium'daki `sans-serif` çözümü
            // Arial metrikleridir. `SansSerif` fontconfig sırasına bırakılırsa
            // macOS'ta Verdana seçilebildiği için etiket ölçüleri ve dolayısıyla
            // bütün yerleşim kayar. Arial bulunmayan sistemlerde genel aile
            // güvenli geri düşüştür; CI profili kullanılan fontu kilitler.
            families: &aileler,
            weight: ağırlık,
            ..fontdb::Query::default()
        })?;
        kimlikten_yüz(kimlik).map(|yüz| (kimlik, yüz))
    };
    let (normal_kimliği, normal) = yüz_yükle(fontdb::Weight::NORMAL)?;
    let (kalın_kimliği, kalın) =
        yüz_yükle(fontdb::Weight::BOLD).unwrap_or_else(|| (normal_kimliği, normal.clone()));

    // Sıra, Chromium'un macOS ve yaygın Linux CI profillerindeki sans-serif
    // CJK geri düşümünü izler. Bulunmayan aileler sessizce atlanır.
    let mut kullanılan = vec![normal_kimliği, kalın_kimliği];
    let mut yedekler = Vec::new();
    for aile in [
        "PingFang SC",
        "Hiragino Sans GB",
        "Heiti SC",
        "STHeiti",
        "Noto Sans CJK SC",
        "Noto Sans SC",
        "WenQuanYi Zen Hei",
        "Arial Unicode MS",
    ] {
        let normal_id = veritabanı.query(&fontdb::Query {
            families: &[fontdb::Family::Name(aile)],
            weight: fontdb::Weight::NORMAL,
            ..fontdb::Query::default()
        });
        let Some(normal_id) = normal_id else { continue };
        if kullanılan.contains(&normal_id) {
            continue;
        }
        let Some(yedek_normal) = kimlikten_yüz(normal_id) else {
            continue;
        };
        let kalın_id = veritabanı
            .query(&fontdb::Query {
                families: &[fontdb::Family::Name(aile)],
                weight: fontdb::Weight::BOLD,
                ..fontdb::Query::default()
            })
            .unwrap_or(normal_id);
        let yedek_kalın = kimlikten_yüz(kalın_id).unwrap_or_else(|| yedek_normal.clone());
        kullanılan.push(normal_id);
        kullanılan.push(kalın_id);
        yedekler.push(YazıYedeği {
            normal: yedek_normal,
            kalın: yedek_kalın,
        });
    }
    Some(YazıTakımı {
        normal,
        kalın,
        // Chromium Canvas `measureText`, aynı sistem Verdana dosyasında
        // ab_glyph'in genel Arial oranından yaklaşık %8 daha geniş em
        // ölçüsü verir. Aileye özgü oran hem kesmeyi hem rasteri aynı
        // metrikle yürütür.
        raster_oranı: if aile.is_some() {
            YAZI_RASTER_ORANI * 1.08
        } else {
            YAZI_RASTER_ORANI
        },
        yedekler,
    })
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
    serif_yazılar: Option<YazıTakımı>,
    /// Custom/rich text içindeki açık `fontFamily: 'Verdana'` yüzü.
    verdana_yazılar: Option<YazıTakımı>,
    /// Çizim sırasında yutulan kurtarılabilir sorunlar.
    tanılar: Vec<BilesenTanisi>,
}

impl PikselYüzeyi {
    /// Yüzey kurar; `ölçek` fiziksel/mantıksal piksel oranıdır (≥ 0.1).
    pub fn yeni(genişlik: f32, yükseklik: f32, ölçek: f32) -> Result<Self, BilesenHatasi> {
        let ölçek = if ölçek.is_finite() && ölçek >= 0.1 {
            ölçek
        } else {
            1.0
        };
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
            yazılar: yazı_takımı_yükle(false),
            // Ölçüm çizimden önce yapılabildiği için serif takımı yüzeyle
            // birlikte yüklenir; böylece `measureText` ve gerçek glif aynı
            // aileyi kullanır.
            serif_yazılar: yazı_takımı_yükle(true),
            // Açık `fontFamily: 'Verdana'` kullanılana kadar ikinci font
            // veritabanını kurma. Varsayılan Arial yolunun yüz/raster
            // seçimini bağımsız tutar ve açılışı gereksiz yere pahalılaştırmaz.
            verdana_yazılar: None,
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

    /// Canvas'ın eksene paralel `Rect` taramasını alan örtüsüyle uygular.
    /// tiny-skia'nın genel yol tarayıcısı, bir pikselden dar kalan sağ/alt
    /// kesirleri atlayabildiği için özellikle sık sütunlarda Chromium
    /// Canvas'tan görünür biçimde ayrılıyordu.
    fn düz_dikdörtgen_doldur(&mut self, d: Dikdörtgen, renk: Renk) {
        let x0 = d.x * self.ölçek;
        let y0 = d.y * self.ölçek;
        let x1 = d.sağ() * self.ölçek;
        let y1 = d.alt() * self.ölçek;
        let genişlik = self.harita.width() as i32;
        let yükseklik = self.harita.height() as i32;
        let ilk_x = (x0.floor() as i32).clamp(0, genişlik);
        let son_x = (x1.ceil() as i32).clamp(0, genişlik);
        let ilk_y = (y0.floor() as i32).clamp(0, yükseklik);
        let son_y = (y1.ceil() as i32).clamp(0, yükseklik);
        if ilk_x >= son_x || ilk_y >= son_y || renk.alfa <= 0.0 {
            return;
        }

        let kaynak = [
            (renk.kırmızı.clamp(0.0, 1.0) * 255.0).round(),
            (renk.yeşil.clamp(0.0, 1.0) * 255.0).round(),
            (renk.mavi.clamp(0.0, 1.0) * 255.0).round(),
        ];
        let maske = self.kırpma.as_ref().map(|m| m.data());
        let veriler = self.harita.pixels_mut();
        for py in ilk_y..son_y {
            let örtü_y = (y1.min(py as f32 + 1.0) - y0.max(py as f32)).clamp(0.0, 1.0);
            for px in ilk_x..son_x {
                let örtü_x = (x1.min(px as f32 + 1.0) - x0.max(px as f32)).clamp(0.0, 1.0);
                let dizin = (py * genişlik + px) as usize;
                let maske_alfası =
                    maske.and_then(|m| m.get(dizin)).copied().unwrap_or(255) as f32 / 255.0;
                let alfa = örtü_x * örtü_y * renk.alfa.clamp(0.0, 1.0) * maske_alfası;
                if alfa <= 0.0 {
                    continue;
                }
                let Some(piksel) = veriler.get_mut(dizin) else {
                    continue;
                };
                let eski = *piksel;
                let kalan = 1.0 - alfa;
                let yeni = ts::PremultipliedColorU8::from_rgba(
                    (kaynak[0] * alfa + eski.red() as f32 * kalan).round() as u8,
                    (kaynak[1] * alfa + eski.green() as f32 * kalan).round() as u8,
                    (kaynak[2] * alfa + eski.blue() as f32 * kalan).round() as u8,
                    (255.0 * alfa + eski.alpha() as f32 * kalan).round() as u8,
                );
                if let Some(yeni) = yeni {
                    *piksel = yeni;
                }
            }
        }
    }

    /// Canvas `fillRect` üzerinde tekrar eden görüntü desenini, düz renkli
    /// hızlı yol ile aynı kesirli alan örtüsüyle boyar. Genel tiny-skia yol
    /// tarayıcısı komşu kesirli hücrelerin ortak kenarında bir pikseli fazla
    /// soldurabildiğinden yoğun matrix/decal döşemelerinde görünür dikiş
    /// bırakıyordu.
    fn desenli_dikdörtgen_doldur(
        &mut self, d: Dikdörtgen, desen: &crate::renk::GörüntüDeseni
    ) {
        let x0 = d.x;
        let y0 = d.y;
        let x1 = d.sağ();
        let y1 = d.alt();
        let genişlik = self.harita.width() as i32;
        let yükseklik = self.harita.height() as i32;
        let ilk_x = (x0.floor() as i32).clamp(0, genişlik);
        let son_x = (x1.ceil() as i32).clamp(0, genişlik);
        let ilk_y = (y0.floor() as i32).clamp(0, yükseklik);
        let son_y = (y1.ceil() as i32).clamp(0, yükseklik);
        let desen_genişliği = desen.genişlik as i32;
        let desen_yüksekliği = desen.yükseklik as i32;
        if ilk_x >= son_x
            || ilk_y >= son_y
            || desen_genişliği <= 0
            || desen_yüksekliği <= 0
            || desen.opaklık <= 0.0
        {
            return;
        }

        let maske = self.kırpma.as_ref().map(|m| m.data());
        let veriler = self.harita.pixels_mut();
        for py in ilk_y..son_y {
            let örtü_y = (y1.min(py as f32 + 1.0) - y0.max(py as f32)).clamp(0.0, 1.0);
            let desen_y = py.rem_euclid(desen_yüksekliği) as usize;
            for px in ilk_x..son_x {
                let örtü_x = (x1.min(px as f32 + 1.0) - x0.max(px as f32)).clamp(0.0, 1.0);
                let desen_x = px.rem_euclid(desen_genişliği) as usize;
                let desen_dizini = (desen_y * desen.genişlik as usize + desen_x) * 4;
                let Some(kaynak) = desen.pikseller.get(desen_dizini..desen_dizini + 4) else {
                    continue;
                };
                let dizin = (py * genişlik + px) as usize;
                let maske_alfası =
                    maske.and_then(|m| m.get(dizin)).copied().unwrap_or(255) as f32 / 255.0;
                let örtü = örtü_x * örtü_y * desen.opaklık.clamp(0.0, 1.0) * maske_alfası;
                let kaynak_alfası = kaynak[3] as f32 / 255.0;
                let alfa = kaynak_alfası * örtü;
                if alfa <= 0.0 {
                    continue;
                }
                let Some(piksel) = veriler.get_mut(dizin) else {
                    continue;
                };
                let eski = *piksel;
                let kalan = 1.0 - alfa;
                let yeni = ts::PremultipliedColorU8::from_rgba(
                    (kaynak[0] as f32 * örtü + eski.red() as f32 * kalan).round() as u8,
                    (kaynak[1] as f32 * örtü + eski.green() as f32 * kalan).round() as u8,
                    (kaynak[2] as f32 * örtü + eski.blue() as f32 * kalan).round() as u8,
                    (kaynak[3] as f32 * örtü + eski.alpha() as f32 * kalan).round() as u8,
                );
                if let Some(yeni) = yeni {
                    *piksel = yeni;
                }
            }
        }
    }

    fn tanı(&mut self, bileşen: &'static str, ayrıntı: String) {
        self.tanılar.push(BilesenTanisi::yeni(
            bileşen,
            BilesenHatasi::GeçersizSeçenek {
                alan: "piksel_yüzeyi",
                ayrıntı,
            },
        ));
        let _ = bileşen;
    }

    /// A8 kaynak maskesini Chromium Canvas gölgesindeki gibi bulanıklaştırıp
    /// renk/kayma/kırpma ile ana piksel haritasına birleştirir.
    fn gölge_maskesini_boya(
        &mut self,
        maske: Vec<u8>,
        renk: Renk,
        bulanıklık: f32,
        kayma: (f32, f32),
    ) {
        let (genişlik, yükseklik) = (self.harita.width() as usize, self.harita.height() as usize);
        // zrender DPR'yi `shadowBlur`a uygular; Chromium/Blink Canvas değeri
        // `blur / 2` sigmasına çevirip Skia A8 maske filtresine gönderir.
        let maske = gauss_bulanık_maske(maske, genişlik, yükseklik, bulanıklık * self.ölçek * 0.5);
        let kayma_x = (kayma.0 * self.ölçek).round() as isize;
        let kayma_y = (kayma.1 * self.ölçek).round() as isize;
        let kırpma = self.kırpma.as_ref().map(|m| m.data());
        for (dizin, piksel) in self.harita.pixels_mut().iter_mut().enumerate() {
            let x = dizin % genişlik;
            let y = dizin / genişlik;
            let kaynak_x = x as isize - kayma_x;
            let kaynak_y = y as isize - kayma_y;
            if kaynak_x < 0 || kaynak_y < 0 {
                continue;
            }
            let kaynak_x = kaynak_x as usize;
            let kaynak_y = kaynak_y as usize;
            if kaynak_x >= genişlik || kaynak_y >= yükseklik {
                continue;
            }
            let kaynak_dizini = kaynak_y.saturating_mul(genişlik).saturating_add(kaynak_x);
            let mut alfa = f32::from(maske.get(kaynak_dizini).copied().unwrap_or(0)) / 255.0
                * renk.alfa.clamp(0.0, 1.0);
            if let Some(kırpma) = kırpma {
                alfa *= f32::from(kırpma.get(dizin).copied().unwrap_or(0)) / 255.0;
            }
            if alfa <= 0.0 {
                continue;
            }
            let eski = *piksel;
            let kalan = 1.0 - alfa;
            if let Some(yeni) = ts::PremultipliedColorU8::from_rgba(
                (renk.kırmızı.clamp(0.0, 1.0) * 255.0 * alfa + f32::from(eski.red()) * kalan)
                    .floor() as u8,
                (renk.yeşil.clamp(0.0, 1.0) * 255.0 * alfa + f32::from(eski.green()) * kalan)
                    .floor() as u8,
                (renk.mavi.clamp(0.0, 1.0) * 255.0 * alfa + f32::from(eski.blue()) * kalan).floor()
                    as u8,
                (255.0 * alfa + f32::from(eski.alpha()) * kalan).round() as u8,
            ) {
                *piksel = yeni;
            }
        }
    }

    fn kırpma_yoluyla(&mut self, yol: &ts::Path, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi)) {
        let önceki = self.kırpma.take();
        let yeni = if let Some(mut maske) = önceki.clone() {
            maske.intersect_path(yol, ts::FillRule::Winding, true, self.dönüşüm());
            Some(maske)
        } else {
            ts::Mask::new(self.harita.width(), self.harita.height()).map(|mut maske| {
                maske.fill_path(yol, ts::FillRule::Winding, true, self.dönüşüm());
                maske
            })
        };
        if yeni.is_none() {
            self.tanı("kırpılı", "kırpma maskesi kurulamadı".to_owned());
        }
        self.kırpma = yeni;
        işlev(self.olarak());
        self.kırpma = önceki;
    }

    /// Metni gerçekten çizilecek yazı ağırlığıyla ölçer. Canvas/zrender
    /// merkez ve sağ hizayı kalın yüzün kendi metrikleriyle çözer; normal yüz
    /// genişliğini kullanmak özellikle ortalanmış başlıkları sağa kaydırır.
    fn yazı_ölç_ağırlıklı(&self, metin: &str, boyut: f32, kalın: bool) -> (f32, f32) {
        self.yazı_ölç_ağırlıklı_takımla(metin, boyut, kalın, self.yazılar.as_ref())
    }

    fn yazı_ölç_ağırlıklı_takımla(
        &self,
        metin: &str,
        boyut: f32,
        kalın: bool,
        yazılar: Option<&YazıTakımı>,
    ) -> (f32, f32) {
        if let Some(yazılar) = yazılar {
            #[cfg(target_os = "macos")]
            let mac_cjk = mac_cjk_yazı_tipi(boyut * self.ölçek, kalın);
            let mut genişlik = 0.0f32;
            let mut önceki: Option<(usize, ab_glyph::GlyphId, bool)> = None;
            for karakter in metin.chars() {
                let birincilde_var = yazılar.birincil(kalın).glyph_id(karakter).0 != 0;
                #[cfg(target_os = "macos")]
                if !birincilde_var
                    && let Some((_, ilerleme)) = mac_cjk
                        .as_ref()
                        .and_then(|yazı_tipi| mac_cjk_glifi(yazı_tipi, karakter))
                {
                    genişlik += ilerleme;
                    önceki = None;
                    continue;
                }
                let (yazı_tipi, oran, yüz_sırası) = yazılar.karakter_yüzü(karakter, kalın);
                let ölçekli =
                    yazı_tipi.as_scaled(ab_glyph::PxScale::from(boyut * self.ölçek * oran));
                let kimlik = ölçekli.glyph_id(karakter);
                if let Some((önceki_yüz, ö, önceki_boşluk)) = önceki
                    && önceki_yüz == yüz_sırası
                    && !önceki_boşluk
                    && !karakter.is_whitespace()
                {
                    genişlik += ölçekli.kern(ö, kimlik);
                }
                genişlik += ölçekli.h_advance(kimlik);
                önceki = Some((yüz_sırası, kimlik, karakter.is_whitespace()));
            }
            (genişlik / self.ölçek, boyut)
        } else {
            // Yazı tipi yoksa KayıtYüzeyi ile aynı belirlenimci yaklaşık.
            (metin.chars().count() as f32 * boyut * 0.6, boyut)
        }
    }
}

fn serif_ailesi_mi(aile: &str) -> bool {
    let aile = aile.trim().to_ascii_lowercase();
    aile == "serif"
        || aile.contains("times")
        || aile.contains("georgia")
        || (aile.contains("serif") && !aile.contains("sans-serif"))
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

/// SVG uç-nokta yayını en çok 90°'lik kübik Bezier parçalarına çevirir.
/// Düz çizgi örneklemesi özellikle büyük pasta/halka kenarlarında Canvas
/// rasterinden testere dişi kadar ayrılıyordu; standart
/// `4/3 * tan(açı/4)` yaklaşımı çemberi alt-piksel doğrulukta korur.
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
    let yükselti = (yarıçap * yarıçap - (uzaklık / 2.0) * (uzaklık / 2.0))
        .max(0.0)
        .sqrt();
    // Kirişe dik birim vektör; merkez seçimi SVG F.6.5 işaret kuralıyla.
    let (nx, ny) = (-dy / uzaklık, dx / uzaklık);
    let işaret = if büyük_yay != süpürme { 1.0 } else { -1.0 };
    let merkez = (
        orta.0 + nx * yükselti * işaret,
        orta.1 + ny * yükselti * işaret,
    );

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
    let parça_sayısı = ((açıklık.abs() / std::f32::consts::FRAC_PI_2).ceil() as usize).clamp(1, 8);
    let parça_açısı = açıklık / parça_sayısı as f32;
    let mut şimdiki_açı = açı0;
    for _ in 0..parça_sayısı {
        let sonraki_açı = şimdiki_açı + parça_açısı;
        let k = (4.0 / 3.0) * (parça_açısı / 4.0).tan() * yarıçap;
        let başlangıç = (
            merkez.0 + yarıçap * şimdiki_açı.cos(),
            merkez.1 + yarıçap * şimdiki_açı.sin(),
        );
        let bitiş = (
            merkez.0 + yarıçap * sonraki_açı.cos(),
            merkez.1 + yarıçap * sonraki_açı.sin(),
        );
        let kontrol1 = (
            başlangıç.0 - k * şimdiki_açı.sin(),
            başlangıç.1 + k * şimdiki_açı.cos(),
        );
        let kontrol2 = (
            bitiş.0 + k * sonraki_açı.sin(),
            bitiş.1 - k * sonraki_açı.cos(),
        );
        kurucu.cubic_to(
            kontrol1.0, kontrol1.1, kontrol2.0, kontrol2.1, bitiş.0, bitiş.1,
        );
        şimdiki_açı = sonraki_açı;
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
            YolKomutu::Yay {
                yarıçap,
                büyük_yay,
                süpürme,
                uç,
            } => {
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
fn boya_çevir<'a>(dolgu: &'a Dolgu, sınır: Option<Dikdörtgen>) -> Option<ts::Paint<'a>> {
    let mut boya = ts::Paint {
        anti_alias: true,
        ..ts::Paint::default()
    };
    match dolgu {
        Dolgu::Düz(renk) => {
            boya.set_color(renk_çevir(*renk));
            Some(boya)
        }
        Dolgu::Desen(desen) => {
            let piksel =
                ts::PixmapRef::from_bytes(&desen.pikseller, desen.genişlik, desen.yükseklik)?;
            let (yayılma, süzme, dönüşüm) = if desen.tekrar == crate::renk::DesenTekrarı::Sığdır
            {
                let sınır = sınır?;
                if sınır.genişlik <= 0.0 || sınır.yükseklik <= 0.0 {
                    return None;
                }
                (
                    ts::SpreadMode::Pad,
                    ts::FilterQuality::Bilinear,
                    ts::Transform::from_row(
                        sınır.genişlik / desen.genişlik as f32,
                        0.0,
                        0.0,
                        sınır.yükseklik / desen.yükseklik as f32,
                        sınır.x,
                        sınır.y,
                    ),
                )
            } else {
                (
                    ts::SpreadMode::Repeat,
                    ts::FilterQuality::Nearest,
                    ts::Transform::identity(),
                )
            };
            boya.shader = ts::Pattern::new(piksel, yayılma, süzme, desen.opaklık, dönüşüm);
            Some(boya)
        }
        Dolgu::DoğrusalGradyan {
            x,
            y,
            x2,
            y2,
            duraklar,
        } => {
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
        Dolgu::RadyalGradyan {
            x,
            y,
            yarıçap,
            duraklar,
        } => {
            let s = sınır?;
            let merkez = ts::Point::from_xy(s.x + x * s.genişlik, s.y + y * s.yükseklik);
            let duraklar: Vec<ts::GradientStop> = duraklar
                .iter()
                .map(|d| ts::GradientStop::new(d.konum.clamp(0.0, 1.0), renk_çevir(d.renk)))
                .collect();
            boya.shader = ts::RadialGradient::new(
                merkez,
                merkez,
                yarıçap * s.genişlik.min(s.yükseklik),
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
        // zrender Path öntanımlısı ve ECharts LineView'in açık tercihi:
        // `lineCap: butt`, `lineJoin: bevel`.
        line_cap: ts::LineCap::Butt,
        line_join: ts::LineJoin::Bevel,
        dash: match tür {
            ÇizgiTürü::Düz => None,
            ÇizgiTürü::Kesikli => ts::StrokeDash::new(vec![4.0 * kalınlık, 2.0 * kalınlık], 0.0),
            ÇizgiTürü::Noktalı => ts::StrokeDash::new(vec![kalınlık, kalınlık], 0.0),
        },
        ..ts::Stroke::default()
    }
}

/// Chromium 131'in kullandığı Skia `PlanGauss` pencere planı. Canvas gölgesi
/// bir resim filtresi değil A8 maske filtresi olduğundan, Skia üç kutu
/// geçişini tek bir kayan toplamda birleştirir ve yalnız sonuçta 8 bite
/// yuvarlar.
struct GaussPlanı {
    ağırlık: u64,
    kenar: usize,
    kayan_pencere: usize,
    geçiş_boyutları: [usize; 3],
}

impl GaussPlanı {
    fn yeni(sigma: f32) -> Option<Self> {
        // `SkBlurMaskFilterImpl::computeXformedSigma` raster maske yolunu 128
        // ile sınırlar. Blink bu işleve Canvas `shadowBlur / 2` gönderir.
        let sigma = sigma.clamp(0.0, 128.0) as f64;
        if sigma < 2.0 {
            return None;
        }
        let pencere = (sigma * 3.0 * (2.0 * std::f64::consts::PI).sqrt() / 4.0 + 0.5)
            .floor()
            .max(1.0) as usize;
        let geçiş_boyutları = [
            pencere.saturating_sub(1),
            pencere.saturating_sub(1),
            if pencere % 2 == 1 {
                pencere.saturating_sub(1)
            } else {
                pencere
            },
        ];
        let kenar = if pencere % 2 == 1 {
            3 * ((pencere - 1) / 2)
        } else {
            3 * (pencere / 2) - 1
        };
        let pencere2 = pencere.saturating_mul(pencere);
        let pencere3 = pencere2.saturating_mul(pencere);
        let bölen = if pencere % 2 == 1 {
            pencere3
        } else {
            pencere3.saturating_add(pencere2)
        };
        let ağırlık = ((1_u64 << 32) as f64 / bölen as f64).round() as u64;
        Some(Self {
            ağırlık,
            kenar,
            kayan_pencere: 2 * kenar + 1,
            geçiş_boyutları,
        })
    }
}

struct GaussTarayıcı<'a> {
    plan: &'a GaussPlanı,
    tamponlar: [Vec<u32>; 3],
    imleçler: [usize; 3],
    toplamlar: [u32; 3],
}

impl<'a> GaussTarayıcı<'a> {
    fn yeni(plan: &'a GaussPlanı) -> Self {
        Self {
            plan,
            tamponlar: plan.geçiş_boyutları.map(|boyut| vec![0; boyut]),
            imleçler: [0; 3],
            toplamlar: [0; 3],
        }
    }

    // Skia'nın üç kutulu PlanGauss taramasının sabit üç geçişli çekirdeği.
    // Dizilerin üçü de aynı `[T; 3]` yapısında kurulup yalnız `0..3`
    // aralığında dolaşıldığı için doğrudan indeksleme burada kanıtlıdır.
    #[allow(clippy::indexing_slicing)]
    fn ilerlet(&mut self, ön_kenar: u8) -> u8 {
        self.toplamlar[0] = self.toplamlar[0].saturating_add(u32::from(ön_kenar));
        self.toplamlar[1] = self.toplamlar[1].saturating_add(self.toplamlar[0]);
        self.toplamlar[2] = self.toplamlar[2].saturating_add(self.toplamlar[1]);

        let sonuç = ((self.plan.ağırlık * u64::from(self.toplamlar[2]) + (1_u64 << 31)) >> 32)
            .min(255) as u8;

        for geçiş in (0..3).rev() {
            let çıkarılacak = self.tamponlar[geçiş][self.imleçler[geçiş]];
            self.toplamlar[geçiş] = self.toplamlar[geçiş].saturating_sub(çıkarılacak);
            self.tamponlar[geçiş][self.imleçler[geçiş]] = if geçiş == 0 {
                u32::from(ön_kenar)
            } else {
                self.toplamlar[geçiş - 1]
            };
            self.imleçler[geçiş] = (self.imleçler[geçiş] + 1) % self.tamponlar[geçiş].len();
        }
        sonuç
    }
}

// Çıkış uzunluğu ve iki imleç aşağıdaki döngülerden önce birlikte türetilir;
// bu sıcak piksel yolunda sınır kontrollerini her örnekte yinelemiyoruz.
#[allow(clippy::indexing_slicing)]
fn gauss_satırı(kaynak: &[u8], plan: &GaussPlanı) -> Vec<u8> {
    let mut hedef = vec![0_u8; kaynak.len().saturating_add(2 * plan.kenar)];
    let boş_geçiş = plan.kayan_pencere.saturating_sub(kaynak.len());
    let mut ileri = GaussTarayıcı::yeni(plan);
    for (dizin, değer) in kaynak.iter().copied().enumerate() {
        hedef[dizin] = ileri.ilerlet(değer);
    }
    for sıra in 0..boş_geçiş {
        hedef[kaynak.len() + sıra] = ileri.ilerlet(0);
    }

    let başlangıç = kaynak.len() + boş_geçiş;
    let mut hedef_imleci = hedef.len();
    let mut kaynak_imleci = kaynak.len();
    let mut geri = GaussTarayıcı::yeni(plan);
    while hedef_imleci > başlangıç {
        hedef_imleci -= 1;
        kaynak_imleci -= 1;
        hedef[hedef_imleci] = geri.ilerlet(kaynak[kaynak_imleci]);
    }
    hedef
}

#[allow(clippy::indexing_slicing)]
fn küçük_gauss_bulanık_maske(
    maske: Vec<u8>,
    genişlik: usize,
    yükseklik: usize,
    sigma: f32,
) -> Vec<u8> {
    let yarıçap = sigma.round().max(1.0) as usize;
    let pencere = (2 * yarıçap + 1) as u32;
    let mut kaynak = maske;
    let mut hedef = vec![0_u8; kaynak.len()];
    for _ in 0..3 {
        for y in 0..yükseklik {
            for x in 0..genişlik {
                let başlangıç = x.saturating_sub(yarıçap);
                let bitiş = (x + yarıçap).min(genişlik.saturating_sub(1));
                let toplam = (başlangıç..=bitiş)
                    .map(|sütun| u32::from(kaynak[y * genişlik + sütun]))
                    .sum::<u32>();
                hedef[y * genişlik + x] = (toplam / pencere).min(255) as u8;
            }
        }
        for y in 0..yükseklik {
            for x in 0..genişlik {
                let başlangıç = y.saturating_sub(yarıçap);
                let bitiş = (y + yarıçap).min(yükseklik.saturating_sub(1));
                let toplam = (başlangıç..=bitiş)
                    .map(|satır| u32::from(hedef[satır * genişlik + x]))
                    .sum::<u32>();
                kaynak[y * genişlik + x] = (toplam / pencere).min(255) as u8;
            }
        }
    }
    kaynak
}

#[allow(clippy::indexing_slicing)]
fn gauss_bulanık_maske(
    maske: Vec<u8>, genişlik: usize, yükseklik: usize, sigma: f32
) -> Vec<u8> {
    if maske.is_empty() || genişlik == 0 || yükseklik == 0 || sigma <= 0.0 {
        return maske;
    }
    let Some(plan) = GaussPlanı::yeni(sigma) else {
        return küçük_gauss_bulanık_maske(maske, genişlik, yükseklik, sigma);
    };

    let bulanık_genişlik = genişlik + 2 * plan.kenar;
    let mut yatay_devrik = vec![0_u8; bulanık_genişlik.saturating_mul(yükseklik)];
    for y in 0..yükseklik {
        let satır = &maske[y * genişlik..(y + 1) * genişlik];
        let bulanık = gauss_satırı(satır, &plan);
        for (x, değer) in bulanık.into_iter().enumerate() {
            yatay_devrik[x * yükseklik + y] = değer;
        }
    }

    let mut sonuç = vec![0_u8; maske.len()];
    for x in 0..genişlik {
        let devrik_x = x + plan.kenar;
        let sütun = &yatay_devrik[devrik_x * yükseklik..(devrik_x + 1) * yükseklik];
        let bulanık = gauss_satırı(sütun, &plan);
        for y in 0..yükseklik {
            sonuç[y * genişlik + x] = bulanık[y + plan.kenar];
        }
    }
    sonuç
}

impl ÇizimYüzeyi for PikselYüzeyi {
    fn genişlik(&self) -> f32 {
        self.mantıksal.0
    }

    fn yükseklik(&self) -> f32 {
        self.mantıksal.1
    }

    fn yol_doldur(&mut self, yol: &Yol, dolgu: &Dolgu) {
        let Some(ts_yolu) = yol_çevir(yol) else {
            return;
        };
        let Some(boya) = boya_çevir(dolgu, yol.sınır_kutusu()) else {
            return;
        };
        self.doldur(&ts_yolu, &boya);
    }

    fn daire(
        &mut self,
        merkez: (f32, f32),
        yarıçap: f32,
        dolgu: Option<&Dolgu>,
        kenarlık: Option<(f32, Renk)>,
    ) {
        if yarıçap <= 0.0 {
            return;
        }
        let yol = daire_yolu(merkez, yarıçap);
        if let Some(dolgu) = dolgu {
            // Canvas/zrender radyal gradyanı sembolün sınır kutusunda tek
            // shader ile örnekler; halka yaklaşımı rasterde bant üretir.
            let sınır = Dikdörtgen::yeni(
                merkez.0 - yarıçap,
                merkez.1 - yarıçap,
                yarıçap * 2.0,
                yarıçap * 2.0,
            );
            if let (Some(ts_yolu), Some(boya)) = (yol_çevir(&yol), boya_çevir(dolgu, Some(sınır)))
            {
                self.doldur(&ts_yolu, &boya);
            }
        }
        if let Some((kalınlık, renk)) = kenarlık
            && kalınlık > 0.0
        {
            self.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
        }
    }

    fn yol_çiz(&mut self, yol: &Yol, kalınlık: f32, renk: Renk, tür: ÇizgiTürü) {
        if yol.boş_mu() || kalınlık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        let Some(ts_yolu) = yol_çevir(yol) else {
            return;
        };
        let mut boya = ts::Paint {
            anti_alias: true,
            force_hq_pipeline: renk.alfa < 1.0,
            ..ts::Paint::default()
        };
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

    fn yol_çizgi_deseni(
        &mut self,
        yol: &Yol,
        kalınlık: f32,
        renk: Renk,
        desen: &[f32],
        kayma: f32,
    ) {
        if yol.boş_mu() || kalınlık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        let Some(ts_yolu) = yol_çevir(yol) else {
            return;
        };
        let mut boya = ts::Paint {
            anti_alias: true,
            ..ts::Paint::default()
        };
        boya.set_color(renk_çevir(renk));
        let mut vuruş = vuruş_yap(kalınlık, ÇizgiTürü::Düz);
        let geçerli = çizgi_deseni_normalleştir(desen);
        if !geçerli.is_empty() {
            vuruş.dash = ts::StrokeDash::new(geçerli, if kayma.is_finite() { kayma } else { 0.0 });
        }
        self.harita.stroke_path(
            &ts_yolu,
            &boya,
            &vuruş,
            self.dönüşüm(),
            self.kırpma.as_ref(),
        );
    }

    fn yol_dolgulu_çiz(&mut self, yol: &Yol, kalınlık: f32, dolgu: &Dolgu, tür: ÇizgiTürü) {
        if yol.boş_mu() || kalınlık <= 0.0 {
            return;
        }
        let Some(ts_yolu) = yol_çevir(yol) else {
            return;
        };
        let Some(boya) = boya_çevir(dolgu, yol.sınır_kutusu()) else {
            return;
        };
        self.harita.stroke_path(
            &ts_yolu,
            &boya,
            &vuruş_yap(kalınlık, tür),
            self.dönüşüm(),
            self.kırpma.as_ref(),
        );
    }

    fn yol_gölgesi(&mut self, yol: &Yol, renk: Renk, bulanıklık: f32, kayma: (f32, f32)) {
        if yol.boş_mu() || bulanıklık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        let Some(ts_yolu) = yol_çevir(yol) else {
            return;
        };
        let Some(mut yol_maskesi) = ts::Mask::new(self.harita.width(), self.harita.height()) else {
            self.tanı("yol_gölgesi", "gölge maskesi ayrılamadı".to_owned());
            return;
        };
        yol_maskesi.fill_path(&ts_yolu, ts::FillRule::Winding, true, self.dönüşüm());
        self.gölge_maskesini_boya(yol_maskesi.data().to_vec(), renk, bulanıklık, kayma);
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
        let Some(ts_yolu) = yol_çevir(yol) else {
            return;
        };
        let Some(mut kaynak) = ts::Pixmap::new(self.harita.width(), self.harita.height()) else {
            self.tanı("yol_çizgi_gölgesi", "gölge haritası ayrılamadı".to_owned());
            return;
        };
        let mut boya = ts::Paint {
            anti_alias: true,
            ..ts::Paint::default()
        };
        boya.set_color(ts::Color::WHITE);
        kaynak.stroke_path(
            &ts_yolu,
            &boya,
            &vuruş_yap(kalınlık, tür),
            self.dönüşüm(),
            None,
        );
        let maske = kaynak
            .pixels()
            .iter()
            .map(|piksel| piksel.alpha())
            .collect();
        self.gölge_maskesini_boya(maske, renk, bulanıklık, kayma);
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
        if kenarlık.is_none()
            && yarıçap.iter().all(|yarıçap| yarıçap.abs() <= f32::EPSILON)
            && let Dolgu::Düz(renk) = dolgu
        {
            self.düz_dikdörtgen_doldur(d, *renk);
            return;
        }
        if kenarlık.is_none()
            && yarıçap.iter().all(|yarıçap| yarıçap.abs() <= f32::EPSILON)
            && (self.ölçek - 1.0).abs() <= f32::EPSILON
            && let Dolgu::Desen(desen) = dolgu
            && desen.tekrar == crate::renk::DesenTekrarı::Tekrar
        {
            self.desenli_dikdörtgen_doldur(d, desen);
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

        let Some(ts_yolu) = yol_çevir(&yol) else {
            return;
        };
        if let Some(boya) = boya_çevir(dolgu, Some(d)) {
            self.doldur(&ts_yolu, &boya);
        }
        if let Some((kalınlık, renk)) = kenarlık
            && kalınlık > 0.0
        {
            let mut boya = ts::Paint {
                anti_alias: true,
                ..ts::Paint::default()
            };
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

    fn büyük_saçılım_noktaları(&mut self, konumlar: &[f32], boyut: f32, dolgu: &Dolgu) {
        if boyut <= 0.0 {
            return;
        }
        let yarı = boyut / 2.0;
        if let Dolgu::Düz(renk) = dolgu {
            // LargeSymbolPath'in `< 4px` Canvas hızlandırması her noktayı
            // ayrı `fillRect` ile boyar. Ayrı geçişler, üst üste gelen yarı
            // saydam noktaların alfa birikimini de birebir korur.
            for çift in konumlar.chunks_exact(2) {
                let [x, y] = çift else { continue };
                if !x.is_finite() || !y.is_finite() {
                    continue;
                }
                self.düz_dikdörtgen_doldur(
                    Dikdörtgen::yeni(*x - yarı, *y - yarı, boyut, boyut),
                    *renk,
                );
            }
            return;
        }
        for çift in konumlar.chunks_exact(2) {
            let [x, y] = çift else { continue };
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            self.dikdörtgen(
                Dikdörtgen::yeni(*x - yarı, *y - yarı, boyut, boyut),
                dolgu,
                [0.0; 4],
                None,
            );
        }
    }

    fn gölge(&mut self, d: Dikdörtgen, yarıçap: f32, renk: Renk, bulanıklık: f32) {
        if d.genişlik <= 0.0 || d.yükseklik <= 0.0 || bulanıklık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        let r = yarıçap.clamp(0.0, d.genişlik.min(d.yükseklik) / 2.0);
        let mut yol = Yol::yeni();
        yol.taşı((d.x + r, d.y));
        yol.çiz((d.sağ() - r, d.y));
        if r > 0.0 {
            yol.yay(r, false, true, (d.sağ(), d.y + r));
        }
        yol.çiz((d.sağ(), d.alt() - r));
        if r > 0.0 {
            yol.yay(r, false, true, (d.sağ() - r, d.alt()));
        }
        yol.çiz((d.x + r, d.alt()));
        if r > 0.0 {
            yol.yay(r, false, true, (d.x, d.alt() - r));
        }
        yol.çiz((d.x, d.y + r));
        if r > 0.0 {
            yol.yay(r, false, true, (d.x + r, d.y));
        }
        yol.kapat();
        let Some(ts_yolu) = yol_çevir(&yol) else {
            return;
        };
        let Some(mut maske) = ts::Mask::new(self.harita.width(), self.harita.height()) else {
            self.tanı("gölge", "gölge maskesi ayrılamadı".to_owned());
            return;
        };
        maske.fill_path(&ts_yolu, ts::FillRule::Winding, true, self.dönüşüm());
        // TooltipHTMLContent öntanımlısı: `1px 2px 10px rgba(0,0,0,.2)`.
        self.gölge_maskesini_boya(maske.data().to_vec(), renk, bulanıklık, (1.0, 2.0));
    }

    fn kırpılı(&mut self, d: Dikdörtgen, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi)) {
        let Some(dikdörtgen) =
            ts::Rect::from_xywh(d.x, d.y, d.genişlik.max(0.1), d.yükseklik.max(0.1))
        else {
            self.tanı("kırpılı", format!("kırpma yolu kurulamadı: {d:?}"));
            return;
        };
        let yol = ts::PathBuilder::from_rect(dikdörtgen);
        self.kırpma_yoluyla(&yol, işlev);
    }

    fn yol_kırpılı(&mut self, yol: &Yol, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi)) {
        let Some(yol) = yol_çevir(yol) else {
            self.tanı("yol_kırpılı", "kırpma yolu çözümlenemedi".to_owned());
            return;
        };
        self.kırpma_yoluyla(&yol, işlev);
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
        // Küçük Arial, CoreText/Skia'da 12 px glifin doğrusal küçültülmüş
        // maskesi değildir: 10 px'te hinting tabanı yaklaşık 0,3 px aşağı
        // taşır ve kenar örtüsünü biraz azaltır. Bu geçiş, Sankey gibi yoğun
        // 10 px etiketlerde Canvas çıktısını korurken 12 px ve üstündeki
        // mevcut ECharts kalibrasyonunu değiştirmez.
        let küçük_yazı_oranı = ((12.0 - boyut) / 2.0).clamp(0.0, 1.0);
        // CoreText/Skia hinting geçişi 10 px'te belirgindir; 9 px ve altında
        // aynı taban itmesini sürdürmek glifi bir raster satırı aşağı taşır.
        // Üst kolda 12 px'e kadar önceki doğrusal sönümü koru.
        let küçük_yazı_taban_oranı = if boyut <= 10.0 {
            (boyut - 9.0).clamp(0.0, 1.0)
        } else {
            küçük_yazı_oranı
        };
        let (genişlik, yükseklik) = self.yazı_ölç_ağırlıklı(metin, boyut, kalın);
        let Some(yazılar) = self.yazılar.clone() else {
            return (genişlik, yükseklik);
        };
        let açık_aile = yazılar.raster_oranı > YAZI_RASTER_ORANI * 1.01;
        let taban_düzeltmesi = if açık_aile {
            YAZI_TABAN_DÜZELTMESİ - 1.0
        } else {
            YAZI_TABAN_DÜZELTMESİ + 0.3 * küçük_yazı_taban_oranı
        };
        let kapsama_çarpanı = if açık_aile {
            0.95
        } else {
            YAZI_KAPSAMA_ÇARPANI - 0.1 * küçük_yazı_oranı
        };
        let birincil_yazı_tipi = if kalın {
            yazılar.kalın.clone()
        } else {
            yazılar.normal.clone()
        };
        let birincil_ölçekli = birincil_yazı_tipi.as_scaled(ab_glyph::PxScale::from(
            boyut * self.ölçek * yazılar.raster_oranı,
        ));

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
        let satır = birincil_ölçekli.ascent() - birincil_ölçekli.descent();
        // Kutunun içinde dikey ortalanmış taban çizgisi.
        let taban_y = üst
            + (yükseklik * self.ölçek - satır) / 2.0
            + birincil_ölçekli.ascent()
            + taban_düzeltmesi * self.ölçek;

        let (kırmızı, yeşil, mavi, alfa) = (
            (renk.kırmızı.clamp(0.0, 1.0) * 255.0) as u16,
            (renk.yeşil.clamp(0.0, 1.0) * 255.0) as u16,
            (renk.mavi.clamp(0.0, 1.0) * 255.0) as u16,
            renk.alfa.clamp(0.0, 1.0),
        );
        let harita_g = self.harita.width() as i32;
        let harita_y = self.harita.height() as i32;
        #[cfg(target_os = "macos")]
        let mac_cjk = mac_cjk_yazı_tipi(boyut * self.ölçek, kalın);

        let mut kalem = x0;
        let mut önceki: Option<(usize, ab_glyph::GlyphId, bool)> = None;
        for karakter in metin.chars() {
            #[cfg(target_os = "macos")]
            if yazılar.birincil(kalın).glyph_id(karakter).0 == 0
                && let Some((glif, ilerleme)) = mac_cjk
                    .as_ref()
                    .and_then(|yazı_tipi| mac_cjk_glifi(yazı_tipi, karakter))
            {
                if let Some(yazı_tipi) = mac_cjk.as_ref() {
                    let kırpma = self.kırpma.as_ref().map(|maske| maske.data().to_vec());
                    mac_cjk_glif_çiz(
                        &mut self.harita,
                        kırpma.as_deref(),
                        yazı_tipi,
                        glif,
                        kalem,
                        taban_y,
                        renk,
                    );
                }
                kalem += ilerleme;
                önceki = None;
                continue;
            }
            let (yazı_tipi, oran, yüz_sırası) = yazılar.karakter_yüzü(karakter, kalın);
            let yazı_tipi = yazı_tipi.clone();
            let ölçekli = yazı_tipi.as_scaled(ab_glyph::PxScale::from(boyut * self.ölçek * oran));
            let kimlik = ölçekli.glyph_id(karakter);
            if let Some((önceki_yüz, ö, önceki_boşluk)) = önceki
                && önceki_yüz == yüz_sırası
                && !önceki_boşluk
                && !karakter.is_whitespace()
            {
                kalem += ölçekli.kern(ö, kimlik);
            }
            let konumlu =
                kimlik.with_scale_and_position(ölçekli.scale(), ab_glyph::point(kalem, taban_y));
            kalem += ölçekli.h_advance(kimlik);
            önceki = Some((yüz_sırası, kimlik, karakter.is_whitespace()));

            let Some(dış_hat) = yazı_tipi.outline_glyph(konumlu) else {
                continue;
            };
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
                let örtü =
                    (kapsama_çarpanı * örtü.clamp(0.0, 1.0).powf(YAZI_KAPSAMA_KUVVETİ)).min(1.0);
                let mut a = örtü * alfa;
                if let Some(maske) = &kırpma_verisi {
                    let m = maske.get(dizin).copied().unwrap_or(0) as f32 / 255.0;
                    a *= m;
                }
                if a <= 0.0 {
                    return;
                }
                let Some(piksel) = veriler.get_mut(dizin) else {
                    return;
                };
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

    fn aileli_yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
        aile: &str,
    ) -> (f32, f32) {
        let verdana = aile.eq_ignore_ascii_case("Verdana");
        if verdana && self.verdana_yazılar.is_none() {
            self.verdana_yazılar = yazı_takımı_adla_yükle(Some("Verdana"), false);
        }
        if serif_ailesi_mi(aile) && self.serif_yazılar.is_none() {
            self.serif_yazılar = yazı_takımı_yükle(true);
        }
        if verdana && self.verdana_yazılar.is_some() {
            std::mem::swap(&mut self.yazılar, &mut self.verdana_yazılar);
            let sonuç = self.yazı(metin, konum, yatay, dikey, boyut, renk, kalın);
            std::mem::swap(&mut self.yazılar, &mut self.verdana_yazılar);
            sonuç
        } else if serif_ailesi_mi(aile) && self.serif_yazılar.is_some() {
            std::mem::swap(&mut self.yazılar, &mut self.serif_yazılar);
            let sonuç = self.yazı(metin, konum, yatay, dikey, boyut, renk, kalın);
            std::mem::swap(&mut self.yazılar, &mut self.serif_yazılar);
            sonuç
        } else {
            self.yazı(metin, konum, yatay, dikey, boyut, renk, kalın)
        }
    }

    fn yazı_ölç(&self, metin: &str, boyut: f32) -> (f32, f32) {
        self.yazı_ölç_ağırlıklı(metin, boyut, false)
    }

    fn aileli_yazı_ölç(&self, metin: &str, boyut: f32, aile: &str) -> (f32, f32) {
        let takım = if aile.eq_ignore_ascii_case("Verdana") {
            self.verdana_yazılar.as_ref().or(self.yazılar.as_ref())
        } else if serif_ailesi_mi(aile) {
            self.serif_yazılar.as_ref().or(self.yazılar.as_ref())
        } else {
            self.yazılar.as_ref()
        };
        self.yazı_ölç_ağırlıklı_takımla(metin, boyut, false, takım)
    }

    fn aileli_stilli_yazı_ölç(
        &self,
        metin: &str,
        boyut: f32,
        kalın: bool,
        aile: &str,
    ) -> (f32, f32) {
        let takım = if aile.eq_ignore_ascii_case("Verdana") {
            self.verdana_yazılar.as_ref().or(self.yazılar.as_ref())
        } else if serif_ailesi_mi(aile) {
            self.serif_yazılar.as_ref().or(self.yazılar.as_ref())
        } else {
            self.yazılar.as_ref()
        };
        self.yazı_ölç_ağırlıklı_takımla(metin, boyut, kalın, takım)
    }

    fn stilli_yazı_ölç(&self, metin: &str, boyut: f32, kalın: bool) -> (f32, f32) {
        let (genişlik, yükseklik) = self.yazı_ölç_ağırlıklı(metin, boyut, kalın);
        // Aynı sistem sans yazı tipi kullanılsa da zrender/Canvas2D'nin
        // ilerleme toplamı ab_glyph'ten yaklaşık binde 0,918 daha kısa.
        // Bu yöntem rich-text yerleşimi için kullanılır; glif rasterini ya da
        // genel eksen/gösterge ölçülerini değiştirmeden resmî kutu sınırını
        // korur (özellikle sınıra tam oturan yüzde rozetlerinde).
        const ZRENDER_İLERLEME_ORANI: f32 = 0.999_082;
        (genişlik * ZRENDER_İLERLEME_ORANI, yükseklik)
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
        let ölçü = self.yazı_ölç_ağırlıklı(metin, boyut, kalın);
        if metin.is_empty() || !dönüşüm.sonlu_mu() {
            return ölçü;
        }
        let Some(ters) = dönüşüm.ters() else {
            let dünya = dönüşüm.noktayı_dönüştür(konum);
            return self.yazı(metin, dünya, yatay, dikey, boyut, renk, kalın);
        };
        let Some(yazılar) = &self.yazılar else {
            return ölçü;
        };
        let yazı_tipi = if kalın {
            yazılar.kalın.clone()
        } else {
            yazılar.normal.clone()
        };
        let ölçekli = yazı_tipi.as_scaled(ab_glyph::PxScale::from(
            boyut * self.ölçek * yazılar.raster_oranı,
        ));

        let x0 = match yatay {
            YatayHiza::Sol => konum.0,
            YatayHiza::Orta => konum.0 - ölçü.0 / 2.0,
            YatayHiza::Sağ => konum.0 - ölçü.0,
        };
        let üst = match dikey {
            DikeyHiza::Üst => konum.1,
            DikeyHiza::Orta => konum.1 - ölçü.1 / 2.0,
            DikeyHiza::Alt => konum.1 - ölçü.1,
        };

        // Canvas, dönüşümlü metni önce eksene paralel bir bitmap üretip onu
        // döndürerek değil, glif dış hatlarını etkin dönüşüm altında Skia'ya
        // vererek rasterleştirir. Özellikle 50° eksen etiketlerinde önceki
        // bitmap + bilinear örnekleme iki kez yumuşatma yapıyordu. Arial'ın
        // ham konturlarını doğrudan tiny-skia yoluna çevirmek aynı tek-geçiş
        // davranışını sağlar. Konturu bulunmayan renkli/bitmap gliflerde alttaki
        // güvenli maske yolu kullanılmaya devam eder.
        let vektör_ölçekli =
            yazı_tipi.as_scaled(ab_glyph::PxScale::from(boyut * yazılar.raster_oranı));
        let satır = vektör_ölçekli.ascent() - vektör_ölçekli.descent();
        let taban_y = üst
            + (ölçü.1 - satır) / 2.0
            + vektör_ölçekli.ascent()
            + DÖNÜŞÜMLÜ_YAZI_TABAN_DÜZELTMESİ;
        let ölçek_çarpanı = vektör_ölçekli.scale_factor();
        let mut kalem = x0;
        let mut önceki: Option<(ab_glyph::GlyphId, bool)> = None;
        let mut yol = ts::PathBuilder::new();
        let mut son: Option<(f32, f32)> = None;
        let mut kontur_var = false;
        let mut dönüşümlü_yollar = Vec::new();
        #[cfg(target_os = "macos")]
        let mac_cjk = mac_cjk_yazı_tipi(boyut, kalın);
        for karakter in metin.chars() {
            #[cfg(target_os = "macos")]
            if yazılar.birincil(kalın).glyph_id(karakter).0 == 0
                && let Some((glif, ilerleme)) = mac_cjk
                    .as_ref()
                    .and_then(|yazı_tipi| mac_cjk_glifi(yazı_tipi, karakter))
                && let Some(glif_yolu) = mac_cjk.as_ref().and_then(|yazı_tipi| {
                    mac_cjk_dönüşümlü_glif_yolu(yazı_tipi, glif, kalem, taban_y, dönüşüm)
                })
            {
                dönüşümlü_yollar.push(glif_yolu);
                kalem += ilerleme;
                önceki = None;
                son = None;
                continue;
            }
            let kimlik = vektör_ölçekli.glyph_id(karakter);
            if let Some((önceki_kimlik, önceki_boşluk)) = önceki
                && !önceki_boşluk
                && !karakter.is_whitespace()
            {
                kalem += vektör_ölçekli.kern(önceki_kimlik, kimlik);
            }
            if let Some(dış_hat) = yazı_tipi.outline(kimlik) {
                let dönüştür = |nokta: ab_glyph::Point| {
                    dönüşüm.noktayı_dönüştür((
                        kalem + nokta.x * ölçek_çarpanı.horizontal,
                        taban_y - nokta.y * ölçek_çarpanı.vertical,
                    ))
                };
                for eğri in dış_hat.curves {
                    let başlangıç = match &eğri {
                        OutlineCurve::Line(p0, _)
                        | OutlineCurve::Quad(p0, _, _)
                        | OutlineCurve::Cubic(p0, _, _, _) => dönüştür(*p0),
                    };
                    if son.is_none_or(|son| {
                        (son.0 - başlangıç.0).abs() > 1e-4 || (son.1 - başlangıç.1).abs() > 1e-4
                    }) {
                        yol.move_to(başlangıç.0, başlangıç.1);
                    }
                    son = Some(match eğri {
                        OutlineCurve::Line(_, p1) => {
                            let p1 = dönüştür(p1);
                            yol.line_to(p1.0, p1.1);
                            p1
                        }
                        OutlineCurve::Quad(_, p1, p2) => {
                            let p1 = dönüştür(p1);
                            let p2 = dönüştür(p2);
                            yol.quad_to(p1.0, p1.1, p2.0, p2.1);
                            p2
                        }
                        OutlineCurve::Cubic(_, p1, p2, p3) => {
                            let p1 = dönüştür(p1);
                            let p2 = dönüştür(p2);
                            let p3 = dönüştür(p3);
                            yol.cubic_to(p1.0, p1.1, p2.0, p2.1, p3.0, p3.1);
                            p3
                        }
                    });
                    kontur_var = true;
                }
            }
            kalem += vektör_ölçekli.h_advance(kimlik);
            önceki = Some((kimlik, karakter.is_whitespace()));
        }
        if kontur_var && let Some(yol) = yol.finish() {
            dönüşümlü_yollar.push(yol);
        }
        if !dönüşümlü_yollar.is_empty() {
            let mut boya = ts::Paint::default();
            boya.set_color(renk_çevir(renk));
            boya.anti_alias = true;
            for yol in &dönüşümlü_yollar {
                self.doldur(yol, &boya);
            }
            // CoreText/Chromium'un küçük döndürülmüş gliflerdeki hinting
            // örtüsü tiny-skia'nın salt yol örtüsünden biraz daha dolgundur.
            // Aynı yolu düşük alfa ile ikinci kez geçirmek yalnız kenar
            // örtüsünü doyurur; tam kapalı gövde ve geometri değişmez.
            // Skia'nın LCD/grayscale metin rasteri açık ön planı koyu bir
            // dolgu üzerinde tam örtüye daha hızlı taşır. tiny-skia yol
            // örtüsü renk bağımsız olduğundan bu fark yalnız açık dönen
            // etiketlerde ikinci geçişin tam alfa kullanmasıyla karşılanır.
            let parlaklık = 0.299 * renk.kırmızı + 0.587 * renk.yeşil + 0.114 * renk.mavi;
            let ek_opaklık = if parlaklık > 0.5 {
                AÇIK_DÖNÜŞÜMLÜ_YAZI_EK_OPAKLIĞI
            } else {
                DÖNÜŞÜMLÜ_YAZI_EK_OPAKLIĞI
            };
            boya.set_color(renk_çevir(renk.opaklık(ek_opaklık)));
            for yol in &dönüşümlü_yollar {
                self.doldur(yol, &boya);
            }
            return ölçü;
        }

        // Dış hatların em kutusundan taşan italik/aksan piksellerini de
        // örneklemek için yerel maskeye küçük bir güvenlik payı eklenir.
        let pay = 3.0 / self.ölçek.max(0.1);
        let yerel_en_az = (x0 - pay, üst - pay);
        let yerel_en_çok = (x0 + ölçü.0 + pay, üst + ölçü.1 + pay);
        let maske_genişliği = ((yerel_en_çok.0 - yerel_en_az.0) * self.ölçek)
            .ceil()
            .max(1.0) as usize;
        let maske_yüksekliği = ((yerel_en_çok.1 - yerel_en_az.1) * self.ölçek)
            .ceil()
            .max(1.0) as usize;
        let Some(maske_uzunluğu) = maske_genişliği.checked_mul(maske_yüksekliği) else {
            self.tanı(
                "dönüşümlü_yazı",
                "metin maskesi boyutu taşma üretti".to_owned(),
            );
            return ölçü;
        };
        let mut metin_maskesi = vec![0.0f32; maske_uzunluğu];
        let satır = ölçekli.ascent() - ölçekli.descent();
        let taban_y = (üst - yerel_en_az.1) * self.ölçek
            + (ölçü.1 * self.ölçek - satır) / 2.0
            + ölçekli.ascent()
            + DÖNÜŞÜMLÜ_YAZI_TABAN_DÜZELTMESİ * self.ölçek;
        let mut kalem = (x0 - yerel_en_az.0) * self.ölçek;
        let mut önceki: Option<(ab_glyph::GlyphId, bool)> = None;
        for karakter in metin.chars() {
            let kimlik = ölçekli.glyph_id(karakter);
            if let Some((önceki_kimlik, önceki_boşluk)) = önceki
                && !önceki_boşluk
                && !karakter.is_whitespace()
            {
                kalem += ölçekli.kern(önceki_kimlik, kimlik);
            }
            let konumlu =
                kimlik.with_scale_and_position(ölçekli.scale(), ab_glyph::point(kalem, taban_y));
            kalem += ölçekli.h_advance(kimlik);
            önceki = Some((kimlik, karakter.is_whitespace()));
            let Some(dış_hat) = yazı_tipi.outline_glyph(konumlu) else {
                continue;
            };
            let sınırlar = dış_hat.px_bounds();
            dış_hat.draw(|gx, gy, örtü| {
                let px = sınırlar.min.x as i32 + gx as i32;
                let py = sınırlar.min.y as i32 + gy as i32;
                if px < 0 || py < 0 {
                    return;
                }
                let (px, py) = (px as usize, py as usize);
                if px >= maske_genişliği || py >= maske_yüksekliği {
                    return;
                }
                let Some(dizin) = py
                    .checked_mul(maske_genişliği)
                    .and_then(|satır| satır.checked_add(px))
                else {
                    return;
                };
                let örtü = (YAZI_KAPSAMA_ÇARPANI * örtü.clamp(0.0, 1.0).powf(YAZI_KAPSAMA_KUVVETİ))
                    .min(1.0);
                if let Some(hedef) = metin_maskesi.get_mut(dizin) {
                    *hedef = 1.0 - (1.0 - *hedef) * (1.0 - örtü);
                }
            });
        }

        let köşeler = [
            yerel_en_az,
            (yerel_en_çok.0, yerel_en_az.1),
            yerel_en_çok,
            (yerel_en_az.0, yerel_en_çok.1),
        ];
        let mut dünya_en_az = (f32::INFINITY, f32::INFINITY);
        let mut dünya_en_çok = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        for köşe in köşeler {
            let dünya = dönüşüm.noktayı_dönüştür(köşe);
            dünya_en_az.0 = dünya_en_az.0.min(dünya.0);
            dünya_en_az.1 = dünya_en_az.1.min(dünya.1);
            dünya_en_çok.0 = dünya_en_çok.0.max(dünya.0);
            dünya_en_çok.1 = dünya_en_çok.1.max(dünya.1);
        }
        let harita_genişliği = self.harita.width() as i32;
        let harita_yüksekliği = self.harita.height() as i32;
        let ilk_x = (dünya_en_az.0 * self.ölçek).floor() as i32;
        let son_x = (dünya_en_çok.0 * self.ölçek).ceil() as i32;
        let ilk_y = (dünya_en_az.1 * self.ölçek).floor() as i32;
        let son_y = (dünya_en_çok.1 * self.ölçek).ceil() as i32;
        let ilk_x = ilk_x.clamp(0, harita_genişliği);
        let son_x = son_x.clamp(0, harita_genişliği);
        let ilk_y = ilk_y.clamp(0, harita_yüksekliği);
        let son_y = son_y.clamp(0, harita_yüksekliği);

        let maske_değeri = |x: i32, y: i32| -> f32 {
            if x < 0 || y < 0 {
                return 0.0;
            }
            let (x, y) = (x as usize, y as usize);
            if x >= maske_genişliği || y >= maske_yüksekliği {
                return 0.0;
            }
            y.checked_mul(maske_genişliği)
                .and_then(|satır| satır.checked_add(x))
                .and_then(|dizin| metin_maskesi.get(dizin))
                .copied()
                .unwrap_or(0.0)
        };
        let kırpma_verisi = self.kırpma.as_ref().map(|maske| maske.data().to_vec());
        let (kırmızı, yeşil, mavi, renk_alfası) = (
            (renk.kırmızı.clamp(0.0, 1.0) * 255.0).round(),
            (renk.yeşil.clamp(0.0, 1.0) * 255.0).round(),
            (renk.mavi.clamp(0.0, 1.0) * 255.0).round(),
            renk.alfa.clamp(0.0, 1.0),
        );
        let veriler = self.harita.pixels_mut();
        for py in ilk_y..son_y {
            for px in ilk_x..son_x {
                let dünya = (
                    (px as f32 + 0.5) / self.ölçek,
                    (py as f32 + 0.5) / self.ölçek,
                );
                let yerel = ters.noktayı_dönüştür(dünya);
                let mx = (yerel.0 - yerel_en_az.0) * self.ölçek - 0.5;
                let my = (yerel.1 - yerel_en_az.1) * self.ölçek - 0.5;
                let x_taban = mx.floor() as i32;
                let y_taban = my.floor() as i32;
                let tx = mx - x_taban as f32;
                let ty = my - y_taban as f32;
                let üst = maske_değeri(x_taban, y_taban) * (1.0 - tx)
                    + maske_değeri(x_taban + 1, y_taban) * tx;
                let alt = maske_değeri(x_taban, y_taban + 1) * (1.0 - tx)
                    + maske_değeri(x_taban + 1, y_taban + 1) * tx;
                let mut alfa = (üst * (1.0 - ty) + alt * ty) * renk_alfası;
                let Some(dizin) = py
                    .checked_mul(harita_genişliği)
                    .and_then(|satır| satır.checked_add(px))
                    .map(|dizin| dizin as usize)
                else {
                    continue;
                };
                if let Some(kırpma) = &kırpma_verisi {
                    alfa *= kırpma.get(dizin).copied().unwrap_or(0) as f32 / 255.0;
                }
                if alfa <= 0.0 {
                    continue;
                }
                let Some(piksel) = veriler.get_mut(dizin) else {
                    continue;
                };
                let eski = *piksel;
                let kalan = 1.0 - alfa;
                let yeni = ts::PremultipliedColorU8::from_rgba(
                    (kırmızı * alfa + eski.red() as f32 * kalan).round() as u8,
                    (yeşil * alfa + eski.green() as f32 * kalan).round() as u8,
                    (mavi * alfa + eski.blue() as f32 * kalan).round() as u8,
                    (255.0 * alfa + eski.alpha() as f32 * kalan).round() as u8,
                );
                if let Some(yeni) = yeni {
                    *piksel = yeni;
                }
            }
        }
        ölçü
    }

    fn dönüşümlü_yazı_gölgesi(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        kalın: bool,
        renk: Renk,
        bulanıklık: f32,
        kayma: (f32, f32),
        dönüşüm: AfinMatris,
    ) {
        if metin.is_empty() || bulanıklık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        let Some(boş_harita) = ts::Pixmap::new(self.harita.width(), self.harita.height()) else {
            self.tanı(
                "dönüşümlü_yazı_gölgesi",
                "metin gölgesi haritası ayrılamadı".to_owned(),
            );
            return;
        };
        let ana_harita = std::mem::replace(&mut self.harita, boş_harita);
        self.dönüşümlü_yazı(
            metin,
            konum,
            yatay,
            dikey,
            boyut,
            Renk::BEYAZ,
            kalın,
            dönüşüm,
        );
        let maske = self
            .harita
            .pixels()
            .iter()
            .map(|piksel| piksel.alpha())
            .collect::<Vec<_>>();
        self.harita = ana_harita;
        self.gölge_maskesini_boya(maske, renk, bulanıklık, kayma);
    }

    fn dönüşümlü_aileli_yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
        aile: &str,
        dönüşüm: AfinMatris,
    ) -> (f32, f32) {
        let verdana = aile.eq_ignore_ascii_case("Verdana");
        if verdana && self.verdana_yazılar.is_none() {
            self.verdana_yazılar = yazı_takımı_adla_yükle(Some("Verdana"), false);
        }
        if serif_ailesi_mi(aile) && self.serif_yazılar.is_none() {
            self.serif_yazılar = yazı_takımı_yükle(true);
        }
        if verdana && self.verdana_yazılar.is_some() {
            std::mem::swap(&mut self.yazılar, &mut self.verdana_yazılar);
            let sonuç =
                self.dönüşümlü_yazı(metin, konum, yatay, dikey, boyut, renk, kalın, dönüşüm);
            std::mem::swap(&mut self.yazılar, &mut self.verdana_yazılar);
            sonuç
        } else if serif_ailesi_mi(aile) && self.serif_yazılar.is_some() {
            std::mem::swap(&mut self.yazılar, &mut self.serif_yazılar);
            let sonuç = self.dönüşümlü_yazı(
                metin,
                (konum.0, konum.1 - 1.0),
                yatay,
                dikey,
                boyut,
                renk,
                kalın,
                dönüşüm,
            );
            std::mem::swap(&mut self.yazılar, &mut self.serif_yazılar);
            sonuç
        } else {
            self.dönüşümlü_yazı(metin, konum, yatay, dikey, boyut, renk, kalın, dönüşüm)
        }
    }

    fn dönüşümlü_konturlu_yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
        kontur_rengi: Renk,
        kontur_kalınlığı: f32,
        dönüşüm: AfinMatris,
    ) -> (f32, f32) {
        let ölçü = self.yazı_ölç_ağırlıklı(metin, boyut, kalın);
        let yol = if metin.is_empty() || !dönüşüm.sonlu_mu() || kontur_kalınlığı <= 0.0 {
            None
        } else {
            self.yazılar.as_ref().and_then(|yazılar| {
                let yazı_tipi = if kalın {
                    yazılar.kalın.clone()
                } else {
                    yazılar.normal.clone()
                };
                let ölçekli =
                    yazı_tipi.as_scaled(ab_glyph::PxScale::from(boyut * yazılar.raster_oranı));
                let x0 = match yatay {
                    YatayHiza::Sol => konum.0,
                    YatayHiza::Orta => konum.0 - ölçü.0 / 2.0,
                    YatayHiza::Sağ => konum.0 - ölçü.0,
                };
                let üst = match dikey {
                    DikeyHiza::Üst => konum.1,
                    DikeyHiza::Orta => konum.1 - ölçü.1 / 2.0,
                    DikeyHiza::Alt => konum.1 - ölçü.1,
                };
                let satır = ölçekli.ascent() - ölçekli.descent();
                let taban_y = üst
                    + (ölçü.1 - satır) / 2.0
                    + ölçekli.ascent()
                    + DÖNÜŞÜMLÜ_YAZI_TABAN_DÜZELTMESİ
                    + KONTURLU_YAZI_TABAN_EKİ;
                let ölçek_çarpanı = ölçekli.scale_factor();
                let mut kalem = x0;
                let mut önceki: Option<(ab_glyph::GlyphId, bool)> = None;
                let mut kurucu = ts::PathBuilder::new();
                let mut son: Option<(f32, f32)> = None;
                let mut dış_hat_var = false;
                for karakter in metin.chars() {
                    let kimlik = ölçekli.glyph_id(karakter);
                    if let Some((önceki_kimlik, önceki_boşluk)) = önceki
                        && !önceki_boşluk
                        && !karakter.is_whitespace()
                    {
                        kalem += ölçekli.kern(önceki_kimlik, kimlik);
                    }
                    if let Some(dış_hat) = yazı_tipi.outline(kimlik) {
                        let dönüştür = |nokta: ab_glyph::Point| {
                            dönüşüm.noktayı_dönüştür((
                                kalem + nokta.x * ölçek_çarpanı.horizontal,
                                taban_y - nokta.y * ölçek_çarpanı.vertical,
                            ))
                        };
                        for eğri in dış_hat.curves {
                            let başlangıç = match &eğri {
                                OutlineCurve::Line(p0, _)
                                | OutlineCurve::Quad(p0, _, _)
                                | OutlineCurve::Cubic(p0, _, _, _) => dönüştür(*p0),
                            };
                            if son.is_none_or(|son| {
                                (son.0 - başlangıç.0).abs() > 1e-4
                                    || (son.1 - başlangıç.1).abs() > 1e-4
                            }) {
                                kurucu.move_to(başlangıç.0, başlangıç.1);
                            }
                            son = Some(match eğri {
                                OutlineCurve::Line(_, p1) => {
                                    let p1 = dönüştür(p1);
                                    kurucu.line_to(p1.0, p1.1);
                                    p1
                                }
                                OutlineCurve::Quad(_, p1, p2) => {
                                    let p1 = dönüştür(p1);
                                    let p2 = dönüştür(p2);
                                    kurucu.quad_to(p1.0, p1.1, p2.0, p2.1);
                                    p2
                                }
                                OutlineCurve::Cubic(_, p1, p2, p3) => {
                                    let p1 = dönüştür(p1);
                                    let p2 = dönüştür(p2);
                                    let p3 = dönüştür(p3);
                                    kurucu.cubic_to(p1.0, p1.1, p2.0, p2.1, p3.0, p3.1);
                                    p3
                                }
                            });
                            dış_hat_var = true;
                        }
                    }
                    kalem += ölçekli.h_advance(kimlik);
                    önceki = Some((kimlik, karakter.is_whitespace()));
                }
                dış_hat_var.then(|| kurucu.finish()).flatten()
            })
        };

        let yazı_dönüşümü = dönüşüm.çarp(AfinMatris::ötele(0.0, KONTURLU_YAZI_TABAN_EKİ));
        if let Some(yol) = yol {
            let mut boya = ts::Paint {
                anti_alias: true,
                ..ts::Paint::default()
            };
            boya.set_color(renk_çevir(kontur_rengi));
            let vuruş = ts::Stroke {
                width: kontur_kalınlığı,
                ..ts::Stroke::default()
            };
            self.harita
                .stroke_path(&yol, &boya, &vuruş, self.dönüşüm(), self.kırpma.as_ref());
        } else if kontur_kalınlığı > 0.0 {
            let yarıçap = kontur_kalınlığı / 2.0;
            let köşegen = yarıçap * std::f32::consts::FRAC_1_SQRT_2;
            for (x, y) in [
                (-yarıçap, 0.0),
                (yarıçap, 0.0),
                (0.0, -yarıçap),
                (0.0, yarıçap),
                (-köşegen, -köşegen),
                (köşegen, -köşegen),
                (-köşegen, köşegen),
                (köşegen, köşegen),
            ] {
                self.dönüşümlü_yazı(
                    metin,
                    konum,
                    yatay,
                    dikey,
                    boyut,
                    kontur_rengi,
                    kalın,
                    AfinMatris::ötele(x, y).çarp(yazı_dönüşümü),
                );
            }
        }
        self.dönüşümlü_yazı(
            metin,
            konum,
            yatay,
            dikey,
            boyut,
            renk,
            kalın,
            yazı_dönüşümü,
        )
    }

    fn dönüşümlü_desenli_konturlu_yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
        kontur_rengi: Renk,
        kontur_kalınlığı: f32,
        desen: &[f32],
        desen_kayması: f32,
        dönüşüm: AfinMatris,
    ) -> (f32, f32) {
        let ölçü = self.yazı_ölç_ağırlıklı(metin, boyut, kalın);
        let yol = if metin.is_empty() || !dönüşüm.sonlu_mu() || kontur_kalınlığı <= 0.0 {
            None
        } else {
            self.yazılar.as_ref().and_then(|yazılar| {
                let yazı_tipi = if kalın {
                    yazılar.kalın.clone()
                } else {
                    yazılar.normal.clone()
                };
                let ölçekli =
                    yazı_tipi.as_scaled(ab_glyph::PxScale::from(boyut * yazılar.raster_oranı));
                let x0 = match yatay {
                    YatayHiza::Sol => konum.0,
                    YatayHiza::Orta => konum.0 - ölçü.0 / 2.0,
                    YatayHiza::Sağ => konum.0 - ölçü.0,
                };
                let üst = match dikey {
                    DikeyHiza::Üst => konum.1,
                    DikeyHiza::Orta => konum.1 - ölçü.1 / 2.0,
                    DikeyHiza::Alt => konum.1 - ölçü.1,
                };
                let satır = ölçekli.ascent() - ölçekli.descent();
                let taban_y = üst
                    + (ölçü.1 - satır) / 2.0
                    + ölçekli.ascent()
                    + DÖNÜŞÜMLÜ_YAZI_TABAN_DÜZELTMESİ
                    + KONTURLU_YAZI_TABAN_EKİ;
                let ölçek_çarpanı = ölçekli.scale_factor();
                let mut kalem = x0;
                let mut önceki: Option<(ab_glyph::GlyphId, bool)> = None;
                let mut kurucu = ts::PathBuilder::new();
                let mut son: Option<(f32, f32)> = None;
                let mut dış_hat_var = false;
                for karakter in metin.chars() {
                    let kimlik = ölçekli.glyph_id(karakter);
                    if let Some((önceki_kimlik, önceki_boşluk)) = önceki
                        && !önceki_boşluk
                        && !karakter.is_whitespace()
                    {
                        kalem += ölçekli.kern(önceki_kimlik, kimlik);
                    }
                    if let Some(dış_hat) = yazı_tipi.outline(kimlik) {
                        let dönüştür = |nokta: ab_glyph::Point| {
                            dönüşüm.noktayı_dönüştür((
                                kalem + nokta.x * ölçek_çarpanı.horizontal,
                                taban_y - nokta.y * ölçek_çarpanı.vertical,
                            ))
                        };
                        for eğri in dış_hat.curves {
                            let başlangıç = match &eğri {
                                OutlineCurve::Line(p0, _)
                                | OutlineCurve::Quad(p0, _, _)
                                | OutlineCurve::Cubic(p0, _, _, _) => dönüştür(*p0),
                            };
                            if son.is_none_or(|son| {
                                (son.0 - başlangıç.0).abs() > 1e-4
                                    || (son.1 - başlangıç.1).abs() > 1e-4
                            }) {
                                kurucu.move_to(başlangıç.0, başlangıç.1);
                            }
                            son = Some(match eğri {
                                OutlineCurve::Line(_, p1) => {
                                    let p1 = dönüştür(p1);
                                    kurucu.line_to(p1.0, p1.1);
                                    p1
                                }
                                OutlineCurve::Quad(_, p1, p2) => {
                                    let p1 = dönüştür(p1);
                                    let p2 = dönüştür(p2);
                                    kurucu.quad_to(p1.0, p1.1, p2.0, p2.1);
                                    p2
                                }
                                OutlineCurve::Cubic(_, p1, p2, p3) => {
                                    let p1 = dönüştür(p1);
                                    let p2 = dönüştür(p2);
                                    let p3 = dönüştür(p3);
                                    kurucu.cubic_to(p1.0, p1.1, p2.0, p2.1, p3.0, p3.1);
                                    p3
                                }
                            });
                            dış_hat_var = true;
                        }
                    }
                    kalem += ölçekli.h_advance(kimlik);
                    önceki = Some((kimlik, karakter.is_whitespace()));
                }
                dış_hat_var.then(|| kurucu.finish()).flatten()
            })
        };

        let yazı_dönüşümü = dönüşüm.çarp(AfinMatris::ötele(0.0, KONTURLU_YAZI_TABAN_EKİ));
        if let Some(yol) = yol {
            let mut boya = ts::Paint {
                anti_alias: true,
                ..ts::Paint::default()
            };
            boya.set_color(renk_çevir(kontur_rengi));
            let mut vuruş = ts::Stroke {
                width: kontur_kalınlığı * KONTURLU_YAZI_VURUŞ_ORANI,
                ..ts::Stroke::default()
            };
            let geçerli = çizgi_deseni_normalleştir(desen);
            if !geçerli.is_empty() {
                vuruş.dash = ts::StrokeDash::new(
                    geçerli,
                    if desen_kayması.is_finite() {
                        desen_kayması
                    } else {
                        0.0
                    },
                );
            }
            self.harita
                .stroke_path(&yol, &boya, &vuruş, self.dönüşüm(), self.kırpma.as_ref());
        } else if desen.is_empty() {
            return self.dönüşümlü_konturlu_yazı(
                metin,
                konum,
                yatay,
                dikey,
                boyut,
                renk,
                kalın,
                kontur_rengi,
                kontur_kalınlığı,
                dönüşüm,
            );
        }
        self.dönüşümlü_yazı(
            metin,
            konum,
            yatay,
            dikey,
            boyut,
            renk,
            kalın,
            yazı_dönüşümü,
        )
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod testler {
    use super::*;
    use crate::renk::RenkDurağı;

    fn kırmızı_mavi_radyal(yarıçap: f32) -> Dolgu {
        Dolgu::radyal(
            0.5,
            0.5,
            yarıçap,
            vec![
                RenkDurağı::yeni(0.0, 0xff0000u32),
                RenkDurağı::yeni(1.0, 0x0000ffu32),
            ],
        )
    }

    #[test]
    fn yarim_piksel_vurusu_resmi_genisligi_korur() {
        assert!((vuruş_yap(0.5, ÇizgiTürü::Düz).width - 0.5).abs() < f32::EPSILON);
        assert!((vuruş_yap(1.0, ÇizgiTürü::Düz).width - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn radyal_gradyan_kısa_kenarı_yarıçap_ölçeği_olarak_kullanır() {
        let mut yüzey = PikselYüzeyi::yeni(40.0, 20.0, 1.0).unwrap();
        yüzey.dikdörtgen(
            Dikdörtgen::yeni(0.0, 0.0, 40.0, 20.0),
            &kırmızı_mavi_radyal(0.5),
            [0.0; 4],
            None,
        );

        let merkez = yüzey.harita.pixel(20, 10).unwrap();
        let yarıçap_dışı = yüzey.harita.pixel(35, 10).unwrap();
        assert!(merkez.red() > 220 && merkez.blue() < 35);
        assert!(yarıçap_dışı.red() < 15 && yarıçap_dışı.blue() > 240);
    }

    #[test]
    fn radyal_gradyanlı_daire_tek_shader_ile_doldurulur() {
        let mut yüzey = PikselYüzeyi::yeni(40.0, 40.0, 1.0).unwrap();
        yüzey.daire((20.0, 20.0), 10.0, Some(&kırmızı_mavi_radyal(1.0)), None);

        let merkez = yüzey.harita.pixel(20, 20).unwrap();
        let dış = yüzey.harita.pixel(5, 20).unwrap();
        assert!(merkez.red() > merkez.blue());
        assert_eq!((dış.red(), dış.green(), dış.blue()), (255, 255, 255));
    }

    #[test]
    fn dönüşümlü_yazı_rasterde_gerçekten_döner() {
        let mut yüzey = PikselYüzeyi::yeni(120.0, 120.0, 1.0).unwrap();
        if yüzey.yazılar.is_none() {
            return;
        }
        yüzey.dönüşümlü_yazı(
            "MMMM",
            (0.0, 0.0),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            16.0,
            Renk::SİYAH,
            false,
            AfinMatris::ötele(60.0, 100.0).çarp(AfinMatris::döndür(-std::f32::consts::FRAC_PI_2)),
        );
        let mut en_az = (u32::MAX, u32::MAX);
        let mut en_çok = (0u32, 0u32);
        let mut sayı = 0usize;
        for y in 0..yüzey.harita.height() {
            for x in 0..yüzey.harita.width() {
                let piksel = yüzey.harita.pixel(x, y).unwrap();
                if piksel.red() < 250 || piksel.green() < 250 || piksel.blue() < 250 {
                    sayı += 1;
                    en_az.0 = en_az.0.min(x);
                    en_az.1 = en_az.1.min(y);
                    en_çok.0 = en_çok.0.max(x);
                    en_çok.1 = en_çok.1.max(y);
                }
            }
        }
        assert!(sayı > 0);
        assert!(en_çok.1 - en_az.1 > en_çok.0 - en_az.0);
    }
}
