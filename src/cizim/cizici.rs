//! Çizici — [`ÇizimYüzeyi`]nin gpui gerçeklemesi: yolları lyon üzerinden
//! döşer, metni gpui metin sistemiyle biçimler, çok duraklı doğrusal
//! gradyanları kırpma bantlarıyla birebir çizer.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    App, BorderStyle, Bounds, BoxShadow, Corners, Edges, FontWeight, PathBuilder, PathStyle,
    Pixels, Point, SharedString, ShapedLine, StrokeOptions, TextAlign, TextRun, Window,
    linear_color_stop, linear_gradient, point, px, quad, size,
};
use lyon::tessellation::{LineCap, LineJoin};

use crate::cizim::yuzey::{DikeyHiza, SATIR_ORANI, YatayHiza, Yol, YolKomutu, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk, RenkDurağı};

/// Kareler arası metin ölçüm önbelleği: `(metin, boyut bitleri) → genişlik`.
pub type ÖlçümÖnbelleği = Rc<RefCell<HashMap<(String, u32), f32>>>;

/// Çizgi türünü kesik desenine çevirir.
fn kesik_deseni(tür: ÇizgiTürü, kalınlık: f32) -> Option<[Pixels; 2]> {
    let k = kalınlık.max(1.0);
    match tür {
        ÇizgiTürü::Düz => None,
        ÇizgiTürü::Kesikli => Some([px(4.0 * k), px(2.0 * k)]),
        ÇizgiTürü::Noktalı => Some([px(k), px(k)]),
    }
}

/// gpui `Window` üzerinde, yüzey yerel koordinatlarıyla çizim yapan katman.
pub struct Çizici<'a, 'b> {
    pub pencere: &'a mut Window,
    pub uygulama: &'b mut App,
    /// Tuvalin pencere içindeki sol üst köşesi.
    pub köken: (f32, f32),
    genişlik: f32,
    yükseklik: f32,
    ölçüm_önbelleği: Option<ÖlçümÖnbelleği>,
}

impl<'a, 'b> Çizici<'a, 'b> {
    pub fn yeni(
        pencere: &'a mut Window,
        uygulama: &'b mut App,
        sınırlar: Bounds<Pixels>,
        ölçüm_önbelleği: Option<ÖlçümÖnbelleği>,
    ) -> Self {
        Çizici {
            pencere,
            uygulama,
            köken: (
                f32::from(sınırlar.origin.x),
                f32::from(sınırlar.origin.y),
            ),
            genişlik: f32::from(sınırlar.size.width),
            yükseklik: f32::from(sınırlar.size.height),
            ölçüm_önbelleği,
        }
    }

    /// Yüzey yerel noktayı pencere koordinatına çevirir.
    pub fn mutlak(&self, n: (f32, f32)) -> Point<Pixels> {
        point(px(self.köken.0 + n.0), px(self.köken.1 + n.1))
    }

    /// Yüzey yerel dikdörtgeni pencere sınırlarına çevirir.
    pub fn sınırlar(&self, d: Dikdörtgen) -> Bounds<Pixels> {
        Bounds {
            origin: self.mutlak((d.x, d.y)),
            size: size(px(d.genişlik.max(0.0)), px(d.yükseklik.max(0.0))),
        }
    }

    fn yol_kur(&self, yol: &Yol, kurucu: &mut PathBuilder) {
        for komut in &yol.komutlar {
            match *komut {
                YolKomutu::Taşı(n) => kurucu.move_to(self.mutlak(n)),
                YolKomutu::Çiz(n) => kurucu.line_to(self.mutlak(n)),
                YolKomutu::Kübik { k1, k2, uç } => {
                    kurucu.cubic_bezier_to(self.mutlak(uç), self.mutlak(k1), self.mutlak(k2))
                }
                YolKomutu::Yay { yarıçap, büyük_yay, süpürme, uç } => kurucu.arc_to(
                    point(px(yarıçap), px(yarıçap)),
                    px(0.0),
                    büyük_yay,
                    süpürme,
                    self.mutlak(uç),
                ),
                YolKomutu::Kapat => kurucu.close(),
            }
        }
    }

    fn yolu_boya(&mut self, yol: &Yol, arkaplan: gpui::Background) {
        let mut kurucu = PathBuilder::fill();
        self.yol_kur(yol, &mut kurucu);
        if let Ok(gpui_yolu) = kurucu.build() {
            self.pencere.paint_path(gpui_yolu, arkaplan);
        }
    }

    /// Eksene hizalı, ikiden çok duraklı doğrusal gradyanı, her ardışık
    /// durak çifti için yolun o banda kırpılmış kopyasını iki duraklı gpui
    /// gradyanıyla boyayarak birebir çizer.
    fn bantlı_gradyan_doldur(
        &mut self,
        yol: &Yol,
        x: f32,
        y: f32,
        x2: f32,
        y2: f32,
        duraklar: &[RenkDurağı],
    ) {
        let Some(kutu) = yol.sınır_kutusu() else { return };

        // Gradyanı ileri yöne çevir (x2 ≥ x, y2 ≥ y).
        let dikey = (x - x2).abs() < 1e-6;
        let (baş, son) = if dikey { (y, y2) } else { (x, x2) };
        let mut duraklar: Vec<RenkDurağı> = duraklar.to_vec();
        let (baş, son) = if son < baş {
            duraklar = duraklar
                .into_iter()
                .rev()
                .map(|d| RenkDurağı { konum: 1.0 - d.konum, renk: d.renk })
                .collect();
            (son, baş)
        } else {
            (baş, son)
        };
        duraklar.sort_by(|a, b| {
            a.konum
                .partial_cmp(&b.konum)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Durak konumunu (gradyan doğrusu üzerinde) sınır kutusu eksen
        // oranına çevirir; gpui gradyanı yol sınırları üzerinden örneklenir.
        let eksen_oranı = |k: f32| (baş + k * (son - baş)).clamp(0.0, 1.0);

        // Uçlardaki düz bölgeler dahil ardışık çiftler.
        let (Some(ilk), Some(sonuncu)) = (duraklar.first(), duraklar.last()) else {
            return;
        };
        let mut çiftler: Vec<(f32, f32, Renk, Renk)> = Vec::new();
        if eksen_oranı(ilk.konum) > 0.0 {
            çiftler.push((0.0, eksen_oranı(ilk.konum), ilk.renk, ilk.renk));
        }
        for p in duraklar.windows(2) {
            if let [a, b] = p {
                çiftler.push((eksen_oranı(a.konum), eksen_oranı(b.konum), a.renk, b.renk));
            }
        }
        if eksen_oranı(sonuncu.konum) < 1.0 {
            çiftler.push((eksen_oranı(sonuncu.konum), 1.0, sonuncu.renk, sonuncu.renk));
        }

        let açı = if dikey { 180.0 } else { 90.0 };
        for (k0, k1, r0, r1) in çiftler {
            let bant = if dikey {
                Dikdörtgen::yeni(
                    kutu.x,
                    kutu.y + k0 * kutu.yükseklik,
                    kutu.genişlik,
                    (k1 - k0) * kutu.yükseklik,
                )
            } else {
                Dikdörtgen::yeni(
                    kutu.x + k0 * kutu.genişlik,
                    kutu.y,
                    (k1 - k0) * kutu.genişlik,
                    kutu.yükseklik,
                )
            };
            if bant.genişlik <= 0.0 || bant.yükseklik <= 0.0 {
                continue;
            }
            let arkaplan = linear_gradient(
                açı,
                linear_color_stop(r0.gpui_hsla(), k0),
                linear_color_stop(r1.gpui_hsla(), k1),
            );
            let sınır = self.sınırlar(bant);
            let köken = self.köken;
            let (g, yük) = (self.genişlik, self.yükseklik);
            let uygulama: &mut App = self.uygulama;
            let önbellek = self.ölçüm_önbelleği.clone();
            self.pencere.paint_layer(sınır, |pencere| {
                let mut iç = Çizici {
                    pencere,
                    uygulama,
                    köken,
                    genişlik: g,
                    yükseklik: yük,
                    ölçüm_önbelleği: önbellek,
                };
                iç.yolu_boya(yol, arkaplan);
            });
        }
    }

    fn şekillendir(&self, metin: &str, boyut: f32, kalın: bool, renk: Renk) -> ShapedLine {
        let temiz: String = metin.replace(['\n', '\r'], " ");
        let paylaşımlı: SharedString = SharedString::from(temiz);
        let mut yazı_tipi = self.pencere.text_style().font();
        yazı_tipi.weight = if kalın { FontWeight::BOLD } else { FontWeight::NORMAL };
        let koşu = TextRun {
            len: paylaşımlı.len(),
            font: yazı_tipi,
            color: renk.gpui_hsla(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        self.pencere
            .text_system()
            .shape_line(paylaşımlı, px(boyut), &[koşu], None)
    }
}

impl ÇizimYüzeyi for Çizici<'_, '_> {
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
        if let Dolgu::DoğrusalGradyan { x, y, x2, y2, duraklar } = dolgu {
            let eksene_hizalı = (x - x2).abs() < 1e-6 || (y - y2).abs() < 1e-6;
            if duraklar.len() > 2 && eksene_hizalı {
                self.bantlı_gradyan_doldur(yol, *x, *y, *x2, *y2, duraklar);
                return;
            }
        }
        self.yolu_boya(yol, dolgu.gpui_arkaplan());
    }

    fn yol_çiz(&mut self, yol: &Yol, kalınlık: f32, renk: Renk, tür: ÇizgiTürü) {
        if yol.boş_mu() || kalınlık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        let seçenekler = StrokeOptions::default()
            .with_line_width(kalınlık)
            .with_line_join(LineJoin::Round)
            .with_start_cap(LineCap::Round)
            .with_end_cap(LineCap::Round);
        let mut kurucu = PathBuilder::stroke(px(kalınlık))
            .with_style(PathStyle::Stroke(seçenekler));
        if let Some(desen) = kesik_deseni(tür, kalınlık) {
            kurucu = kurucu.dash_array(&desen);
        }
        self.yol_kur(yol, &mut kurucu);
        if let Ok(gpui_yolu) = kurucu.build() {
            self.pencere.paint_path(gpui_yolu, renk.gpui_hsla());
        }
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
        let köşeler = Corners {
            top_left: px(yarıçap[0]),
            top_right: px(yarıçap[1]),
            bottom_right: px(yarıçap[2]),
            bottom_left: px(yarıçap[3]),
        };
        let (kenar_kalınlığı, kenar_rengi) = kenarlık.unwrap_or((0.0, Renk::SAYDAM));
        self.pencere.paint_quad(quad(
            self.sınırlar(d),
            köşeler,
            dolgu.gpui_arkaplan(),
            Edges::all(px(kenar_kalınlığı)),
            kenar_rengi.gpui_hsla(),
            BorderStyle::Solid,
        ));
    }

    fn gölge(&mut self, d: Dikdörtgen, yarıçap: f32, renk: Renk, bulanıklık: f32) {
        self.pencere.paint_drop_shadows(
            self.sınırlar(d),
            Corners::all(px(yarıçap)),
            &[BoxShadow {
                color: renk.gpui_hsla(),
                offset: point(px(0.0), px(2.0)),
                blur_radius: px(bulanıklık),
                spread_radius: px(0.0),
                inset: false,
            }],
        );
    }

    fn kırpılı(&mut self, d: Dikdörtgen, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi)) {
        let sınır = self.sınırlar(d);
        let köken = self.köken;
        let (genişlik, yükseklik) = (self.genişlik, self.yükseklik);
        let önbellek = self.ölçüm_önbelleği.clone();
        let uygulama: &mut App = self.uygulama;
        self.pencere.paint_layer(sınır, |pencere| {
            let mut iç = Çizici {
                pencere,
                uygulama,
                köken,
                genişlik,
                yükseklik,
                ölçüm_önbelleği: önbellek,
            };
            işlev(&mut iç);
        });
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
        let satır = self.şekillendir(metin, boyut, kalın, renk);
        let genişlik = f32::from(satır.width());
        let satır_yüksekliği = boyut * SATIR_ORANI;
        let x = match yatay {
            YatayHiza::Sol => konum.0,
            YatayHiza::Orta => konum.0 - genişlik / 2.0,
            YatayHiza::Sağ => konum.0 - genişlik,
        };
        let y = match dikey {
            DikeyHiza::Üst => konum.1,
            DikeyHiza::Orta => konum.1 - satır_yüksekliği / 2.0,
            DikeyHiza::Alt => konum.1 - satır_yüksekliği,
        };
        let başlangıç = self.mutlak((x, y));
        satır
            .paint(
                başlangıç,
                px(satır_yüksekliği),
                TextAlign::Left,
                None,
                self.pencere,
                self.uygulama,
            )
            .ok();
        (genişlik, satır_yüksekliği)
    }

    fn yazı_ölç(&self, metin: &str, boyut: f32) -> (f32, f32) {
        if metin.is_empty() {
            return (0.0, 0.0);
        }
        let yükseklik = boyut * SATIR_ORANI;
        if let Some(önbellek) = &self.ölçüm_önbelleği {
            let anahtar = (metin.to_string(), boyut.to_bits());
            if let Ok(kayıt) = önbellek.try_borrow()
                && let Some(genişlik) = kayıt.get(&anahtar) {
                    return (*genişlik, yükseklik);
                }
            let satır = self.şekillendir(metin, boyut, false, Renk::SİYAH);
            let genişlik = f32::from(satır.width());
            // Önbellek kilitliyse ölçüm yine de geçerlidir; kayıt atlanır.
            if let Ok(mut kayıt) = önbellek.try_borrow_mut() {
                kayıt.insert(anahtar, genişlik);
            }
            return (genişlik, yükseklik);
        }
        let satır = self.şekillendir(metin, boyut, false, Renk::SİYAH);
        (f32::from(satır.width()), yükseklik)
    }

    fn olarak(&mut self) -> &mut dyn ÇizimYüzeyi {
        self
    }
}
