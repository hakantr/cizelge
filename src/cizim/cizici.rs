//! Çizici — gpui `Window` boyama API'si üzerine kurulmuş, grafik yerel
//! koordinatlarında çalışan çizim yardımcısı (zrender `Painter` karşılığı).

use gpui::{
    App, BorderStyle, Bounds, BoxShadow, Corners, Edges, FontWeight, PathBuilder, PathStyle,
    Pixels, Point, SharedString, ShapedLine, StrokeOptions, TextAlign, TextRun, Window, point,
    px, quad, size,
};
use lyon::tessellation::{LineCap, LineJoin};

use crate::koordinat::Dikdörtgen;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

/// Yazı satır yüksekliğinin yazı boyutuna oranı.
pub const SATIR_ORANI: f32 = 1.4;

/// Yatay yazı hizası.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum YatayHiza {
    #[default]
    Sol,
    Orta,
    Sağ,
}

/// Dikey yazı hizası.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DikeyHiza {
    Üst,
    #[default]
    Orta,
    Alt,
}

/// Yol komutu — koordinatlar grafik yerelidir.
#[derive(Clone, Copy, Debug)]
pub enum YolKomutu {
    Taşı((f32, f32)),
    Çiz((f32, f32)),
    /// Kübik Bezier: kontrol noktaları `k1`, `k2`, bitiş `uç`.
    Kübik {
        k1: (f32, f32),
        k2: (f32, f32),
        uç: (f32, f32),
    },
    /// Yay: `yarıçap`, `büyük_yay`, `süpürme` (SVG bayrakları) ile `uç`a.
    Yay {
        yarıçap: f32,
        büyük_yay: bool,
        süpürme: bool,
        uç: (f32, f32),
    },
    Kapat,
}

/// Komut listesinden oluşan yol (zrender `PathProxy` karşılığı).
#[derive(Clone, Debug, Default)]
pub struct Yol {
    pub komutlar: Vec<YolKomutu>,
}

impl Yol {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn taşı(&mut self, n: (f32, f32)) {
        self.komutlar.push(YolKomutu::Taşı(n));
    }

    pub fn çiz(&mut self, n: (f32, f32)) {
        self.komutlar.push(YolKomutu::Çiz(n));
    }

    pub fn kübik(&mut self, k1: (f32, f32), k2: (f32, f32), uç: (f32, f32)) {
        self.komutlar.push(YolKomutu::Kübik { k1, k2, uç });
    }

    pub fn yay(&mut self, yarıçap: f32, büyük_yay: bool, süpürme: bool, uç: (f32, f32)) {
        self.komutlar.push(YolKomutu::Yay { yarıçap, büyük_yay, süpürme, uç });
    }

    pub fn kapat(&mut self) {
        self.komutlar.push(YolKomutu::Kapat);
    }

    pub fn boş_mu(&self) -> bool {
        self.komutlar.len() < 2
    }
}

/// Çizgi türünü kesik desenine çevirir.
fn kesik_deseni(tür: ÇizgiTürü, kalınlık: f32) -> Option<[Pixels; 2]> {
    let k = kalınlık.max(1.0);
    match tür {
        ÇizgiTürü::Düz => None,
        ÇizgiTürü::Kesikli => Some([px(4.0 * k), px(2.0 * k)]),
        ÇizgiTürü::Noktalı => Some([px(k), px(k)]),
    }
}

/// gpui `Window` üzerinde, grafik yerel koordinatlarıyla çizim yapan katman.
pub struct Çizici<'a, 'b> {
    pub pencere: &'a mut Window,
    pub uygulama: &'b mut App,
    /// Tuvalin pencere içindeki sol üst köşesi.
    pub köken: (f32, f32),
    pub genişlik: f32,
    pub yükseklik: f32,
}

impl<'a, 'b> Çizici<'a, 'b> {
    pub fn yeni(
        pencere: &'a mut Window,
        uygulama: &'b mut App,
        sınırlar: Bounds<Pixels>,
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
        }
    }

    /// Grafik yerel noktayı pencere koordinatına çevirir.
    pub fn mutlak(&self, n: (f32, f32)) -> Point<Pixels> {
        point(px(self.köken.0 + n.0), px(self.köken.1 + n.1))
    }

    /// Grafik yerel dikdörtgeni pencere sınırlarına çevirir.
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

    /// Yolu dolgu ile boyar.
    pub fn yol_doldur(&mut self, yol: &Yol, dolgu: &Dolgu) {
        if yol.boş_mu() {
            return;
        }
        let mut kurucu = PathBuilder::fill();
        self.yol_kur(yol, &mut kurucu);
        if let Ok(gpui_yolu) = kurucu.build() {
            self.pencere.paint_path(gpui_yolu, dolgu.gpui_arkaplan());
        }
    }

    /// Yolu verilen kalınlık ve türde çizgiler.
    pub fn yol_çiz(&mut self, yol: &Yol, kalınlık: f32, renk: Renk, tür: ÇizgiTürü) {
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

    /// İki nokta arasına çizgi çeker.
    pub fn çizgi(
        &mut self,
        a: (f32, f32),
        b: (f32, f32),
        kalınlık: f32,
        renk: Renk,
        tür: ÇizgiTürü,
    ) {
        let mut yol = Yol::yeni();
        yol.taşı(a);
        yol.çiz(b);
        self.yol_çiz(&yol, kalınlık, renk, tür);
    }

    /// Ardışık noktalardan çoklu çizgi çeker.
    pub fn çoklu_çizgi(
        &mut self,
        noktalar: &[(f32, f32)],
        kalınlık: f32,
        renk: Renk,
        tür: ÇizgiTürü,
    ) {
        if noktalar.len() < 2 {
            return;
        }
        let mut yol = Yol::yeni();
        yol.taşı(noktalar[0]);
        for n in &noktalar[1..] {
            yol.çiz(*n);
        }
        self.yol_çiz(&yol, kalınlık, renk, tür);
    }

    /// Dikdörtgen boyar; `yarıçap` köşe sırası `[sol üst, sağ üst, sağ alt,
    /// sol alt]`, `kenarlık` `(kalınlık, renk)` çiftidir.
    pub fn dikdörtgen(
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

    /// Daire boyar; istenirse ayrıca kenarlık halkası çizer.
    pub fn daire(
        &mut self,
        merkez: (f32, f32),
        yarıçap: f32,
        dolgu: Option<&Dolgu>,
        kenarlık: Option<(f32, Renk)>,
    ) {
        if yarıçap <= 0.0 {
            return;
        }
        let (mx, my) = merkez;
        let mut yol = Yol::yeni();
        yol.taşı((mx + yarıçap, my));
        yol.yay(yarıçap, false, true, (mx - yarıçap, my));
        yol.yay(yarıçap, false, true, (mx + yarıçap, my));
        yol.kapat();
        if let Some(dolgu) = dolgu {
            self.yol_doldur(&yol, dolgu);
        }
        if let Some((kalınlık, renk)) = kenarlık {
            if kalınlık > 0.0 {
                self.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
            }
        }
    }

    /// Pasta dilimi (halka parçası) boyar. Açılar radyandır ve ekran
    /// koordinatındadır (0 → sağ, pozitif yön saat yönü).
    pub fn dilim(
        &mut self,
        merkez: (f32, f32),
        iç_yarıçap: f32,
        dış_yarıçap: f32,
        açı0: f32,
        açı1: f32,
        dolgu: &Dolgu,
        kenarlık: Option<(f32, Renk)>,
    ) {
        if dış_yarıçap <= 0.0 || (açı1 - açı0).abs() < 1e-5 {
            return;
        }
        // Tam daireye çok yakın dilimler yay uçlarının çakışmaması için
        // kırpılır.
        let tam_tur = std::f32::consts::TAU;
        let açıklık = (açı1 - açı0).clamp(-tam_tur * 0.9999, tam_tur * 0.9999);
        let açı1 = açı0 + açıklık;

        let (mx, my) = merkez;
        let uç = |yarıçap: f32, açı: f32| (mx + yarıçap * açı.cos(), my + yarıçap * açı.sin());
        let büyük = açıklık.abs() > std::f32::consts::PI;
        let süpürme = açıklık > 0.0;

        let mut yol = Yol::yeni();
        if iç_yarıçap > 0.5 {
            yol.taşı(uç(iç_yarıçap, açı0));
            yol.çiz(uç(dış_yarıçap, açı0));
            yol.yay(dış_yarıçap, büyük, süpürme, uç(dış_yarıçap, açı1));
            yol.çiz(uç(iç_yarıçap, açı1));
            yol.yay(iç_yarıçap, büyük, !süpürme, uç(iç_yarıçap, açı0));
            yol.kapat();
        } else {
            yol.taşı(merkez);
            yol.çiz(uç(dış_yarıçap, açı0));
            yol.yay(dış_yarıçap, büyük, süpürme, uç(dış_yarıçap, açı1));
            yol.kapat();
        }
        self.yol_doldur(&yol, dolgu);
        if let Some((kalınlık, renk)) = kenarlık {
            if kalınlık > 0.0 {
                self.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
            }
        }
    }

    /// Gölge boyar (ipucu penceresi vb. için).
    pub fn gölge(&mut self, d: Dikdörtgen, yarıçap: f32, renk: Renk, bulanıklık: f32) {
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

    /// Çizimi verilen dikdörtgene kırparak `işlev`i çalıştırır
    /// (animasyon kırpması ve ızgara taşma denetimi için).
    pub fn kırp<R>(&mut self, d: Dikdörtgen, işlev: impl FnOnce(&mut Çizici) -> R) -> R {
        let sınır = self.sınırlar(d);
        let köken = self.köken;
        let (genişlik, yükseklik) = (self.genişlik, self.yükseklik);
        let uygulama: &mut App = self.uygulama;
        self.pencere.paint_layer(sınır, |pencere| {
            let mut iç = Çizici { pencere, uygulama, köken, genişlik, yükseklik };
            işlev(&mut iç)
        })
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

    /// Yazının kaplayacağı `(genişlik, yükseklik)` boyutunu ölçer.
    pub fn yazı_ölç(&self, metin: &str, boyut: f32) -> (f32, f32) {
        if metin.is_empty() {
            return (0.0, 0.0);
        }
        let satır = self.şekillendir(metin, boyut, false, Renk::SİYAH);
        (f32::from(satır.width()), boyut * SATIR_ORANI)
    }

    /// Tek satır yazı boyar; `konum`, hizaya göre çapa noktasıdır.
    /// Çizilen `(genişlik, yükseklik)` döner.
    pub fn yazı(
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
}
