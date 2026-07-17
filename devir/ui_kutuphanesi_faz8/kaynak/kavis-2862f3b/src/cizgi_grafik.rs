use std::{cell::Cell, collections::BTreeSet, ops::Range, rc::Rc};

use gpui::{
    AccessibleAction, App, Bounds, Context, EventEmitter, FocusHandle, Focusable, KeyDownEvent,
    MouseButton, MouseDownEvent, PathBuilder, Pixels, Render, Role, ScrollDelta, ScrollWheelEvent,
    SharedString, Window, canvas, div, fill, point, prelude::*, px, size,
};
use ortak_bilesenler::{gpui_rengi, tema_rolleri};
use ortak_bilesenler_cekirdek::GuvenliDegerTemsili;
use ortak_tipler::KararliKimlik;

use crate::{
    DogrusalOlcek, Nokta, en_yakin_nokta, esit_aralik_indeksleri, izgara_indeksleri,
    lttb_indeksleri, min_maks_indeksleri,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GrafikTuru {
    #[default]
    Cizgi,
    Alan,
    Cubuk,
    Pasta,
    Halka,
    Dagilim,
    Baloncuk,
    Birlesik,
    IsiHaritasi,
    AgacHaritasi,
    Sankey,
    Gosterge,
    MiniCizgi,
    Kpi,
    PanoKutusu,
}

impl GrafikTuru {
    const fn varsayilan_ornekleme(self) -> OrneklemeStratejisi {
        match self {
            Self::Cizgi | Self::MiniCizgi | Self::Birlesik => OrneklemeStratejisi::Lttb,
            Self::Alan | Self::Cubuk => OrneklemeStratejisi::MinMaks,
            Self::Dagilim | Self::Baloncuk | Self::IsiHaritasi => OrneklemeStratejisi::Izgara,
            Self::Pasta
            | Self::Halka
            | Self::AgacHaritasi
            | Self::Sankey
            | Self::Gosterge
            | Self::Kpi
            | Self::PanoKutusu => OrneklemeStratejisi::EsitAralik,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrneklemeStratejisi {
    Yok,
    Lttb,
    MinMaks,
    Izgara,
    EsitAralik,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrafikButcesi {
    pub azami_nokta: usize,
    pub azami_tahmini_bayt: usize,
    pub azami_etiket_bayt: usize,
    pub azami_cizim_noktasi: usize,
    pub azami_csv_bayt: usize,
}

impl Default for GrafikButcesi {
    fn default() -> Self {
        Self {
            azami_nokta: 1_000_000,
            azami_tahmini_bayt: 128 * 1024 * 1024,
            azami_etiket_bayt: 4 * 1024,
            azami_cizim_noktasi: 1_200,
            azami_csv_bayt: 64 * 1024 * 1024,
        }
    }
}

impl GrafikButcesi {
    pub fn dogrula(self) -> Result<Self, GrafikHatasi> {
        if self.azami_nokta == 0
            || self.azami_tahmini_bayt == 0
            || self.azami_etiket_bayt == 0
            || self.azami_cizim_noktasi < 3
            || self.azami_csv_bayt == 0
        {
            return Err(GrafikHatasi::GecersizButce);
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrafikHatasi {
    GecersizButce,
    NoktaButcesiAsildi,
    BaytButcesiAsildi,
    EtiketButcesiAsildi,
    SonluOlmayanDeger,
    GecersizBoyut,
    YinelenenKimlik,
    CsvButcesiAsildi,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GrafikTabloSatiri {
    pub kimlik: KararliKimlik,
    pub maskeli_etiket: String,
    pub x: f64,
    pub y: f64,
    pub boyut: f64,
}

#[derive(Clone, Debug)]
pub struct GrafikNoktasi {
    pub kimlik: KararliKimlik,
    pub konum: Nokta,
    pub etiket: GuvenliDegerTemsili,
    pub boyut: f64,
}

impl GrafikNoktasi {
    pub fn yeni(
        kimlik: impl Into<KararliKimlik>,
        konum: Nokta,
        etiket: GuvenliDegerTemsili,
    ) -> Self {
        Self {
            kimlik: kimlik.into(),
            konum,
            etiket,
            boyut: 1.0,
        }
    }

    pub fn boyut(mut self, boyut: f64) -> Self {
        self.boyut = boyut;
        self
    }
}

/// Erişilebilir alternatif görünüm, milyon satırı tek seferde ayırmak yerine
/// satırları istenen görünür aralıkta üretir.
pub struct GrafikTabloModeli<'a> {
    noktalar: &'a [GrafikNoktasi],
}

impl GrafikTabloModeli<'_> {
    pub fn satir_sayisi(&self) -> usize {
        self.noktalar.len()
    }

    pub fn satir(&self, sira: usize) -> Option<GrafikTabloSatiri> {
        self.noktalar.get(sira).map(tablo_satiri)
    }

    pub fn aralik(&self, aralik: Range<usize>) -> Vec<GrafikTabloSatiri> {
        let bas = aralik.start.min(self.noktalar.len());
        let son = aralik.end.min(self.noktalar.len()).max(bas);
        self.noktalar[bas..son].iter().map(tablo_satiri).collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CizgiGrafikOlayi {
    NoktaEtkinlesti(KararliKimlik),
    SecimDegisti(Vec<KararliKimlik>),
    GorunumDegisti { x_bas: f64, x_son: f64 },
    LejantDegisti(bool),
}

pub struct CizgiGrafik {
    odak: FocusHandle,
    ad: SharedString,
    noktalar: Vec<GrafikNoktasi>,
    etkin: Option<usize>,
    secili: BTreeSet<KararliKimlik>,
    butce: GrafikButcesi,
    tur: GrafikTuru,
    ornekleme: Option<OrneklemeStratejisi>,
    gorunum_x: Option<(f64, f64)>,
    lejant_gorunur: bool,
    cizim_siniri: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl CizgiGrafik {
    pub fn yeni(
        ad: impl Into<SharedString>,
        noktalar: Vec<GrafikNoktasi>,
        cx: &mut Context<Self>,
    ) -> Result<Self, GrafikHatasi> {
        Self::butceli(ad, noktalar, GrafikButcesi::default(), cx)
    }

    pub fn butceli(
        ad: impl Into<SharedString>,
        noktalar: Vec<GrafikNoktasi>,
        butce: GrafikButcesi,
        cx: &mut Context<Self>,
    ) -> Result<Self, GrafikHatasi> {
        Self::profille(ad, noktalar, butce, GrafikTuru::Cizgi, cx)
    }

    pub(crate) fn profille(
        ad: impl Into<SharedString>,
        noktalar: Vec<GrafikNoktasi>,
        butce: GrafikButcesi,
        tur: GrafikTuru,
        cx: &mut Context<Self>,
    ) -> Result<Self, GrafikHatasi> {
        let butce = butce.dogrula()?;
        veriyi_dogrula(&noktalar, butce)?;
        Ok(Self::dogrulanmis(ad, noktalar, butce, tur, cx))
    }

    pub(crate) fn dogrulanmis(
        ad: impl Into<SharedString>,
        noktalar: Vec<GrafikNoktasi>,
        butce: GrafikButcesi,
        tur: GrafikTuru,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            odak: cx.focus_handle().tab_stop(true),
            ad: ad.into(),
            etkin: (!noktalar.is_empty()).then_some(0),
            noktalar,
            secili: BTreeSet::new(),
            butce,
            tur,
            ornekleme: None,
            gorunum_x: None,
            lejant_gorunur: true,
            cizim_siniri: Rc::new(Cell::new(None)),
        }
    }

    pub fn ornekleme(mut self, strateji: OrneklemeStratejisi) -> Self {
        self.ornekleme = Some(strateji);
        self
    }

    pub fn etkin(&self) -> Option<&GrafikNoktasi> {
        self.etkin.and_then(|sira| self.noktalar.get(sira))
    }

    pub fn secili_kimlikler(&self) -> impl Iterator<Item = &KararliKimlik> {
        self.secili.iter()
    }

    pub fn erisebilir_tablo(&self) -> GrafikTabloModeli<'_> {
        GrafikTabloModeli {
            noktalar: &self.noktalar,
        }
    }

    /// Dışa aktarma görünür gösterme oturumundan bağımsızdır. Etiket güvenli
    /// kalıcı kanaldan gelir; CSV formülü olarak yorumlanabilecek ilk karakter
    /// apostrofla etkisizleştirilir ve çıktı byte bütçesinde fail-closed kalır.
    pub fn guvenli_csv(&self) -> Result<String, GrafikHatasi> {
        let mut csv = String::from("etiket,x,y,boyut\n");
        for nokta in &self.noktalar {
            let satir = tablo_satiri(nokta);
            csv.push_str(&csv_alani(&formul_guvenli(&satir.maskeli_etiket)));
            csv.push(',');
            csv.push_str(&satir.x.to_string());
            csv.push(',');
            csv.push_str(&satir.y.to_string());
            csv.push(',');
            csv.push_str(&satir.boyut.to_string());
            csv.push('\n');
            if csv.len() > self.butce.azami_csv_bayt {
                return Err(GrafikHatasi::CsvButcesiAsildi);
            }
        }
        Ok(csv)
    }

    pub fn gorunumu_sifirla(&mut self, cx: &mut Context<Self>) {
        self.gorunum_x = None;
        cx.notify();
    }

    fn ilerlet(&mut self, ileri: bool, cx: &mut Context<Self>) {
        if self.noktalar.is_empty() {
            return;
        }
        let mevcut = self.etkin.unwrap_or(0);
        let hedef = if ileri {
            (mevcut + 1).min(self.noktalar.len() - 1)
        } else {
            mevcut.saturating_sub(1)
        };
        self.etkinlestir(hedef, cx);
    }

    fn etkinlestir(&mut self, sira: usize, cx: &mut Context<Self>) {
        if let Some(nokta) = self.noktalar.get(sira) {
            self.etkin = Some(sira);
            cx.emit(CizgiGrafikOlayi::NoktaEtkinlesti(nokta.kimlik.clone()));
            cx.notify();
        }
    }

    fn secimi_degistir(&mut self, eklemeli: bool, cx: &mut Context<Self>) {
        let Some(kimlik) = self.etkin().map(|nokta| nokta.kimlik.clone()) else {
            return;
        };
        if !eklemeli {
            self.secili.clear();
        }
        if !self.secili.insert(kimlik.clone()) {
            self.secili.remove(&kimlik);
        }
        cx.emit(CizgiGrafikOlayi::SecimDegisti(
            self.secili.iter().cloned().collect(),
        ));
        cx.notify();
    }

    fn veri_x_alani(&self) -> Option<(f64, f64)> {
        let (min, max) = min_max(self.noktalar.iter().map(|nokta| nokta.konum.x));
        min.is_finite().then_some(genislet(min, max))
    }

    fn yakinlastir(&mut self, oran: f64, cx: &mut Context<Self>) {
        let Some(tum) = self.veri_x_alani() else {
            return;
        };
        let mevcut = self.gorunum_x.unwrap_or(tum);
        let merkez = f64::midpoint(mevcut.0, mevcut.1);
        let tum_genislik = tum.1 - tum.0;
        let genislik = ((mevcut.1 - mevcut.0) / oran).clamp(tum_genislik / 10_000.0, tum_genislik);
        self.gorunum_x = Some(sinirla_x(
            (merkez - genislik / 2.0, merkez + genislik / 2.0),
            tum,
        ));
        self.gorunum_olayini_yay(cx);
    }

    fn kaydir(&mut self, oran: f64, cx: &mut Context<Self>) {
        let Some(tum) = self.veri_x_alani() else {
            return;
        };
        let mevcut = self.gorunum_x.unwrap_or(tum);
        let delta = (mevcut.1 - mevcut.0) * oran;
        self.gorunum_x = Some(sinirla_x((mevcut.0 + delta, mevcut.1 + delta), tum));
        self.gorunum_olayini_yay(cx);
    }

    fn gorunum_olayini_yay(&mut self, cx: &mut Context<Self>) {
        if let Some((x_bas, x_son)) = self.gorunum_x.or_else(|| self.veri_x_alani()) {
            cx.emit(CizgiGrafikOlayi::GorunumDegisti { x_bas, x_son });
        }
        cx.notify();
    }

    fn klavye(&mut self, olay: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        match olay.keystroke.key.as_str() {
            "left" | "up" => self.ilerlet(false, cx),
            "right" | "down" => self.ilerlet(true, cx),
            "home" if !self.noktalar.is_empty() => self.etkinlestir(0, cx),
            "end" if !self.noktalar.is_empty() => self.etkinlestir(self.noktalar.len() - 1, cx),
            "enter" | "space" => self.secimi_degistir(olay.keystroke.modifiers.shift, cx),
            "+" | "=" => self.yakinlastir(1.25, cx),
            "-" => self.yakinlastir(0.8, cx),
            "0" => self.gorunumu_sifirla(cx),
            "pageup" => self.kaydir(-0.8, cx),
            "pagedown" => self.kaydir(0.8, cx),
            "l" => {
                self.lejant_gorunur = !self.lejant_gorunur;
                cx.emit(CizgiGrafikOlayi::LejantDegisti(self.lejant_gorunur));
                cx.notify();
            }
            _ => return false,
        }
        true
    }

    fn tekerlek(&mut self, olay: &ScrollWheelEvent, cx: &mut Context<Self>) {
        let delta = match olay.delta {
            ScrollDelta::Pixels(piksel) => f64::from(f32::from(piksel.y)),
            ScrollDelta::Lines(satir) => f64::from(satir.y) * 16.0,
        };
        if olay.modifiers.control || olay.modifiers.platform {
            self.yakinlastir(if delta <= 0.0 { 1.12 } else { 1.0 / 1.12 }, cx);
        } else {
            self.kaydir((delta / 600.0).clamp(-0.5, 0.5), cx);
        }
    }

    fn fare_basildi(&mut self, olay: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(bounds) = self.cizim_siniri.get() else {
            return;
        };
        let ekran_noktalari = self.ekran_noktalari(bounds);
        let hedef = Nokta::yeni(
            f64::from(f32::from(olay.position.x - bounds.left())),
            f64::from(f32::from(olay.position.y - bounds.top())),
        );
        if let Some(sira) = en_yakin_nokta(&ekran_noktalari, hedef, 14.0) {
            self.etkinlestir(sira, cx);
            self.secimi_degistir(olay.modifiers.shift, cx);
        }
    }

    fn gorunen_indeksler(&self) -> Vec<usize> {
        let ham: Vec<_> = self.noktalar.iter().map(|nokta| nokta.konum).collect();
        let adaylar: Vec<_> = if let Some((bas, son)) = self.gorunum_x {
            ham.iter()
                .enumerate()
                .filter_map(|(sira, nokta)| (nokta.x >= bas && nokta.x <= son).then_some(sira))
                .collect()
        } else {
            (0..ham.len()).collect()
        };
        if adaylar.len() <= self.butce.azami_cizim_noktasi {
            return adaylar;
        }
        let aday_noktalar: Vec<_> = adaylar.iter().map(|sira| ham[*sira]).collect();
        let strateji = self
            .ornekleme
            .unwrap_or_else(|| self.tur.varsayilan_ornekleme());
        let yerel = match strateji {
            OrneklemeStratejisi::Yok => (0..aday_noktalar.len()).collect(),
            OrneklemeStratejisi::Lttb => {
                lttb_indeksleri(&aday_noktalar, self.butce.azami_cizim_noktasi)
            }
            OrneklemeStratejisi::MinMaks => {
                min_maks_indeksleri(&aday_noktalar, self.butce.azami_cizim_noktasi)
            }
            OrneklemeStratejisi::Izgara => {
                izgara_indeksleri(&aday_noktalar, self.butce.azami_cizim_noktasi)
            }
            OrneklemeStratejisi::EsitAralik => {
                esit_aralik_indeksleri(aday_noktalar.len(), self.butce.azami_cizim_noktasi)
            }
        };
        yerel.into_iter().map(|sira| adaylar[sira]).collect()
    }

    fn cizim_noktalari(&self) -> Vec<Nokta> {
        self.gorunen_indeksler()
            .into_iter()
            .map(|sira| self.noktalar[sira].konum)
            .collect()
    }

    fn ekran_noktalari(&self, bounds: Bounds<Pixels>) -> Vec<Nokta> {
        let ham: Vec<_> = self.noktalar.iter().map(|nokta| nokta.konum).collect();
        let Some((x, y)) = olcekler(&ham, self.tur, bounds) else {
            return Vec::new();
        };
        ham.into_iter()
            .map(|nokta| Nokta::yeni(x.haritala(nokta.x), y.haritala(nokta.y)))
            .collect()
    }
}

impl EventEmitter<CizgiGrafikOlayi> for CizgiGrafik {}

impl Focusable for CizgiGrafik {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.odak.clone()
    }
}

impl Render for CizgiGrafik {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let roller = tema_rolleri(cx);
        let odakta = self.odak.is_focused(window);
        let noktalar = self.cizim_noktalari();
        let boyutlar: Vec<_> = self
            .gorunen_indeksler()
            .into_iter()
            .map(|sira| self.noktalar[sira].boyut)
            .collect();
        let renk = gpui_rengi(roller.sinir.odak);
        let tur = self.tur;
        let etkin_metin = self.etkin().map(|nokta| {
            format!(
                "{}: x={}, y={}",
                nokta.etiket.maskeli_metin(),
                nokta.konum.x,
                nokta.konum.y
            )
        });
        let ozet = format!(
            "{} nokta · {} seçili · {} çiziliyor",
            self.noktalar.len(),
            self.secili.len(),
            noktalar.len()
        );
        let erisilebilir_deger =
            format!("{} · {ozet}", etkin_metin.as_deref().unwrap_or("Veri yok"));
        let cizim_siniri = self.cizim_siniri.clone();
        let a11y_onceki = cx.weak_entity();
        let a11y_sonraki = cx.weak_entity();
        let a11y_sec = cx.weak_entity();

        div()
            .id(("cizgi-grafik", cx.entity_id()))
            .debug_selector(|| "faz8-cizgi-grafik".to_owned())
            .role(Role::Group)
            .aria_label(self.ad.clone())
            .aria_value(erisilebilir_deger)
            .track_focus(&self.odak)
            .on_key_down(cx.listener(|bu, olay: &KeyDownEvent, _window, cx| {
                if bu.klavye(olay, cx) {
                    cx.stop_propagation();
                }
            }))
            .on_scroll_wheel(cx.listener(|bu, olay: &ScrollWheelEvent, _, cx| {
                bu.tekerlek(olay, cx);
            }))
            .on_a11y_action(AccessibleAction::Decrement, move |_, _, cx| {
                let _ = a11y_onceki.update(cx, |bu, cx| bu.ilerlet(false, cx));
            })
            .on_a11y_action(AccessibleAction::Increment, move |_, _, cx| {
                let _ = a11y_sonraki.update(cx, |bu, cx| bu.ilerlet(true, cx));
            })
            .on_a11y_action(AccessibleAction::Click, move |_, _, cx| {
                let _ = a11y_sec.update(cx, |bu, cx| bu.secimi_degistir(false, cx));
            })
            .on_mouse_down(MouseButton::Left, cx.listener(Self::fare_basildi))
            .w_full()
            .h(px(if tur == GrafikTuru::MiniCizgi {
                96.0
            } else {
                280.0
            }))
            .flex()
            .flex_col()
            .gap_1()
            .border_1()
            .border_color(gpui_rengi(if odakta {
                roller.sinir.odak
            } else {
                roller.sinir.normal
            }))
            .bg(gpui_rengi(roller.yuzey.panel))
            .child(
                canvas(
                    move |bounds, _, _| cizim_siniri.set(Some(bounds)),
                    move |bounds, (), window, _| {
                        grafik_ciz(&noktalar, &boyutlar, tur, bounds, window, renk);
                    },
                )
                .flex_grow(1.0)
                .w_full(),
            )
            .when(
                self.lejant_gorunur && tur != GrafikTuru::MiniCizgi,
                |yuzey| {
                    yuzey.child(
                        div()
                            .flex_none()
                            .h(px(20.0))
                            .px_2()
                            .text_xs()
                            .text_color(gpui_rengi(roller.metin.soluk))
                            .child(ozet),
                    )
                },
            )
            .child(
                div()
                    .flex_none()
                    .h(px(28.0))
                    .px_2()
                    .text_xs()
                    .text_color(gpui_rengi(roller.metin.normal))
                    .child(etkin_metin.unwrap_or_else(|| "Veri yok".to_owned())),
            )
    }
}

pub(crate) fn veriyi_dogrula(
    noktalar: &[GrafikNoktasi],
    butce: GrafikButcesi,
) -> Result<(), GrafikHatasi> {
    if noktalar.len() > butce.azami_nokta {
        return Err(GrafikHatasi::NoktaButcesiAsildi);
    }
    let mut kimlikler = BTreeSet::new();
    let mut tahmini_bayt = noktalar
        .len()
        .saturating_mul(std::mem::size_of::<GrafikNoktasi>());
    for nokta in noktalar {
        if !nokta.konum.x.is_finite() || !nokta.konum.y.is_finite() {
            return Err(GrafikHatasi::SonluOlmayanDeger);
        }
        if !nokta.boyut.is_finite() || nokta.boyut < 0.0 {
            return Err(GrafikHatasi::GecersizBoyut);
        }
        if !kimlikler.insert(nokta.kimlik.clone()) {
            return Err(GrafikHatasi::YinelenenKimlik);
        }
        let etiket_bayt = nokta.etiket.kalici_metin().len();
        if etiket_bayt > butce.azami_etiket_bayt {
            return Err(GrafikHatasi::EtiketButcesiAsildi);
        }
        tahmini_bayt = tahmini_bayt
            .saturating_add(nokta.kimlik.deger().len())
            .saturating_add(etiket_bayt);
        if tahmini_bayt > butce.azami_tahmini_bayt {
            return Err(GrafikHatasi::BaytButcesiAsildi);
        }
    }
    Ok(())
}

fn tablo_satiri(nokta: &GrafikNoktasi) -> GrafikTabloSatiri {
    GrafikTabloSatiri {
        kimlik: nokta.kimlik.clone(),
        maskeli_etiket: nokta.etiket.kalici_metin().to_owned(),
        x: nokta.konum.x,
        y: nokta.konum.y,
        boyut: nokta.boyut,
    }
}

fn olcekler(
    noktalar: &[Nokta],
    tur: GrafikTuru,
    bounds: Bounds<Pixels>,
) -> Option<(DogrusalOlcek, DogrusalOlcek)> {
    if noktalar.is_empty() {
        return None;
    }
    let (min_x, max_x) = min_max(noktalar.iter().map(|nokta| nokta.x));
    let (mut min_y, mut max_y) = min_max(noktalar.iter().map(|nokta| nokta.y));
    if matches!(
        tur,
        GrafikTuru::Cubuk | GrafikTuru::Alan | GrafikTuru::Birlesik
    ) {
        min_y = min_y.min(0.0);
        max_y = max_y.max(0.0);
    }
    let (min_x, max_x) = genislet(min_x, max_x);
    let (min_y, max_y) = genislet(min_y, max_y);
    Some((
        DogrusalOlcek::yeni(min_x, max_x, 0.0, f64::from(bounds.size.width)).ok()?,
        DogrusalOlcek::yeni(min_y, max_y, f64::from(bounds.size.height), 0.0).ok()?,
    ))
}

#[allow(clippy::cast_possible_truncation)]
fn grafik_ciz(
    noktalar: &[Nokta],
    boyutlar: &[f64],
    tur: GrafikTuru,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    renk: gpui::Rgba,
) {
    if noktalar.is_empty() {
        return;
    }
    match tur {
        GrafikTuru::Pasta | GrafikTuru::Halka => pasta_ciz(noktalar, tur, bounds, window, renk),
        GrafikTuru::AgacHaritasi => agac_haritasi_ciz(noktalar, bounds, window, renk),
        GrafikTuru::Sankey => sankey_ciz(noktalar, bounds, window, renk),
        GrafikTuru::Gosterge => gosterge_ciz(noktalar, bounds, window, renk),
        GrafikTuru::Kpi | GrafikTuru::PanoKutusu => kpi_ciz(noktalar, bounds, window, renk),
        _ => kartezyen_ciz(noktalar, boyutlar, tur, bounds, window, renk),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn kartezyen_ciz(
    noktalar: &[Nokta],
    boyutlar: &[f64],
    tur: GrafikTuru,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    renk: gpui::Rgba,
) {
    let Some((x, y)) = olcekler(noktalar, tur, bounds) else {
        return;
    };
    match tur {
        GrafikTuru::Cizgi | GrafikTuru::MiniCizgi => {
            cizgi_ciz(noktalar, x, y, bounds, window, renk);
        }
        GrafikTuru::Alan => {
            let taban = y.haritala(0.0);
            for nokta in noktalar {
                let merkez_x = x.haritala(nokta.x);
                let deger_y = y.haritala(nokta.y);
                window.paint_quad(fill(
                    Bounds {
                        origin: point(
                            bounds.left() + px(merkez_x as f32),
                            bounds.top() + px(deger_y.min(taban) as f32),
                        ),
                        size: size(px(2.0), px((deger_y - taban).abs().max(1.0) as f32)),
                    },
                    renk.alpha(0.35),
                ));
            }
            cizgi_ciz(noktalar, x, y, bounds, window, renk);
        }
        GrafikTuru::Cubuk | GrafikTuru::Birlesik => {
            cubuk_ciz(noktalar, x, y, bounds, window, renk.alpha(0.55));
            if tur == GrafikTuru::Birlesik {
                cizgi_ciz(noktalar, x, y, bounds, window, renk);
            }
        }
        GrafikTuru::Dagilim | GrafikTuru::Baloncuk | GrafikTuru::IsiHaritasi => {
            for (sira, nokta) in noktalar.iter().enumerate() {
                let merkez = grafik_noktasi(*nokta, x, y, bounds);
                let cap = if tur == GrafikTuru::Baloncuk {
                    boyutlar
                        .get(sira)
                        .copied()
                        .unwrap_or(1.0)
                        .sqrt()
                        .clamp(2.0, 24.0)
                } else if tur == GrafikTuru::IsiHaritasi {
                    10.0
                } else {
                    3.0
                };
                let alfa = if tur == GrafikTuru::IsiHaritasi {
                    (nokta.y.abs() / 100.0).clamp(0.15, 1.0) as f32
                } else {
                    1.0
                };
                window.paint_quad(fill(
                    Bounds {
                        origin: point(merkez.x - px(cap as f32), merkez.y - px(cap as f32)),
                        size: size(px((cap * 2.0) as f32), px((cap * 2.0) as f32)),
                    },
                    renk.alpha(alfa),
                ));
            }
        }
        _ => {}
    }
}

fn cizgi_ciz(
    noktalar: &[Nokta],
    x: DogrusalOlcek,
    y: DogrusalOlcek,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    renk: gpui::Rgba,
) {
    if noktalar.len() < 2 {
        return;
    }
    let mut yol = PathBuilder::stroke(px(2.0));
    for (sira, nokta) in noktalar.iter().enumerate() {
        let konum = grafik_noktasi(*nokta, x, y, bounds);
        if sira == 0 {
            yol.move_to(konum);
        } else {
            yol.line_to(konum);
        }
    }
    if let Ok(yol) = yol.build() {
        window.paint_path(yol, renk);
    }
}

#[allow(clippy::cast_possible_truncation)]
fn cubuk_ciz(
    noktalar: &[Nokta],
    x: DogrusalOlcek,
    y: DogrusalOlcek,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    renk: gpui::Rgba,
) {
    let sayi = f64::from(u32::try_from(noktalar.len()).unwrap_or(u32::MAX));
    let genislik = (f64::from(bounds.size.width) / sayi * 0.7).max(1.0);
    let taban = y.haritala(0.0);
    for nokta in noktalar {
        let merkez_x = x.haritala(nokta.x);
        let deger_y = y.haritala(nokta.y);
        let ust = deger_y.min(taban);
        window.paint_quad(fill(
            Bounds {
                origin: point(
                    bounds.left() + px((merkez_x - genislik / 2.0) as f32),
                    bounds.top() + px(ust as f32),
                ),
                size: size(
                    px(genislik as f32),
                    px((deger_y - taban).abs().max(1.0) as f32),
                ),
            },
            renk,
        ));
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn pasta_ciz(
    noktalar: &[Nokta],
    tur: GrafikTuru,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    renk: gpui::Rgba,
) {
    let toplam: f64 = noktalar.iter().map(|nokta| nokta.y.max(0.0)).sum();
    if toplam <= f64::EPSILON {
        return;
    }
    let merkez = bounds.center();
    let yaricap = f32::from(bounds.size.width.min(bounds.size.height)) * 0.42;
    let mut aci = -std::f64::consts::FRAC_PI_2;
    for (sira, nokta) in noktalar.iter().enumerate() {
        let pay = nokta.y.max(0.0) / toplam;
        let sonraki = aci + pay * std::f64::consts::TAU;
        let adim = ((pay * 96.0).ceil() as usize).clamp(2, 96);
        let mut yol = if tur == GrafikTuru::Halka {
            PathBuilder::stroke(px(yaricap * 0.34))
        } else {
            PathBuilder::fill()
        };
        if tur == GrafikTuru::Pasta {
            yol.move_to(merkez);
        }
        for parca in 0..=adim {
            let oran = parca as f64 / adim as f64;
            let parca_aci = aci + (sonraki - aci) * oran;
            let yay_noktasi = point(
                merkez.x + px((parca_aci.cos() * f64::from(yaricap)) as f32),
                merkez.y + px((parca_aci.sin() * f64::from(yaricap)) as f32),
            );
            if parca == 0 && tur == GrafikTuru::Halka {
                yol.move_to(yay_noktasi);
            } else {
                yol.line_to(yay_noktasi);
            }
        }
        if tur == GrafikTuru::Pasta {
            yol.close();
        }
        if let Ok(yol) = yol.build() {
            let alfa = 0.35 + 0.65 * ((sira % 7) as f32 / 6.0);
            window.paint_path(yol, renk.alpha(alfa));
        }
        aci = sonraki;
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn agac_haritasi_ciz(
    noktalar: &[Nokta],
    bounds: Bounds<Pixels>,
    window: &mut Window,
    renk: gpui::Rgba,
) {
    let toplam: f64 = noktalar.iter().map(|nokta| nokta.y.abs()).sum();
    if toplam <= f64::EPSILON {
        return;
    }
    let mut sol = bounds.left();
    for (sira, nokta) in noktalar.iter().enumerate() {
        let genislik = bounds.size.width * ((nokta.y.abs() / toplam) as f32);
        window.paint_quad(fill(
            Bounds {
                origin: point(sol, bounds.top()),
                size: size(genislik.max(px(1.0)), bounds.size.height),
            },
            renk.alpha(0.35 + 0.65 * ((sira % 7) as f32 / 6.0)),
        ));
        sol += genislik;
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn sankey_ciz(noktalar: &[Nokta], bounds: Bounds<Pixels>, window: &mut Window, renk: gpui::Rgba) {
    if noktalar.len() < 2 {
        return;
    }
    let adim = f32::from(bounds.size.height) / noktalar.len() as f32;
    for (sira, cift) in noktalar.windows(2).enumerate() {
        let bas = point(bounds.left(), bounds.top() + px((sira as f32 + 0.5) * adim));
        let son = point(
            bounds.right(),
            bounds.top() + px((sira as f32 + 1.5) * adim),
        );
        let mut yol = PathBuilder::stroke(px(cift[0].y.abs().sqrt().clamp(1.0, 18.0) as f32));
        yol.move_to(bas);
        yol.cubic_bezier_to(
            son,
            point(bounds.left() + bounds.size.width * 0.35, bas.y),
            point(bounds.left() + bounds.size.width * 0.65, son.y),
        );
        if let Ok(yol) = yol.build() {
            window.paint_path(yol, renk.alpha(0.7));
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn gosterge_ciz(noktalar: &[Nokta], bounds: Bounds<Pixels>, window: &mut Window, renk: gpui::Rgba) {
    let deger = noktalar
        .first()
        .map_or(0.0, |nokta| nokta.y.clamp(0.0, 100.0));
    let merkez = point(bounds.center().x, bounds.bottom() - px(8.0));
    let yaricap = f32::from(bounds.size.width.min(bounds.size.height * 2.0)) * 0.42;
    for sira in 0..40 {
        let oran = f64::from(sira) / 39.0;
        let aci = std::f64::consts::PI * (1.0 + oran);
        let nokta = point(
            merkez.x + px((aci.cos() * f64::from(yaricap)) as f32),
            merkez.y + px((aci.sin() * f64::from(yaricap)) as f32),
        );
        let etkin = oran * 100.0 <= deger;
        window.paint_quad(fill(
            Bounds {
                origin: point(nokta.x - px(2.0), nokta.y - px(2.0)),
                size: size(px(4.0), px(4.0)),
            },
            renk.alpha(if etkin { 1.0 } else { 0.18 }),
        ));
    }
}

#[allow(clippy::cast_possible_truncation)]
fn kpi_ciz(noktalar: &[Nokta], bounds: Bounds<Pixels>, window: &mut Window, renk: gpui::Rgba) {
    let deger = noktalar.first().map_or(0.0, |nokta| nokta.y);
    let oran = (deger.abs() / (deger.abs() + 100.0)).clamp(0.0, 1.0) as f32;
    window.paint_quad(fill(
        Bounds {
            origin: point(bounds.left(), bounds.bottom() - px(8.0)),
            size: size(bounds.size.width * oran, px(8.0)),
        },
        renk,
    ));
}

#[allow(clippy::cast_possible_truncation)]
fn grafik_noktasi(
    nokta: Nokta,
    x: DogrusalOlcek,
    y: DogrusalOlcek,
    bounds: Bounds<Pixels>,
) -> gpui::Point<Pixels> {
    point(
        bounds.left() + px(x.haritala(nokta.x) as f32),
        bounds.top() + px(y.haritala(nokta.y) as f32),
    )
}

fn min_max(degerler: impl Iterator<Item = f64>) -> (f64, f64) {
    degerler.fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), deger| {
        (min.min(deger), max.max(deger))
    })
}

fn genislet(min: f64, max: f64) -> (f64, f64) {
    if (max - min).abs() < f64::EPSILON {
        (min - 0.5, max + 0.5)
    } else {
        (min, max)
    }
}

fn sinirla_x(mut alan: (f64, f64), tum: (f64, f64)) -> (f64, f64) {
    let genislik = alan.1 - alan.0;
    if alan.0 < tum.0 {
        alan = (tum.0, tum.0 + genislik);
    }
    if alan.1 > tum.1 {
        alan = (tum.1 - genislik, tum.1);
    }
    alan
}

fn csv_alani(deger: &str) -> String {
    if deger.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", deger.replace('"', "\"\""))
    } else {
        deger.to_owned()
    }
}

fn formul_guvenli(deger: &str) -> String {
    if deger.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{deger}")
    } else {
        deger.to_owned()
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use gpui::{Modifiers, TestAppContext};
    use ortak_bilesenler::{OrtakBilesenAyarlari, baslat};
    use ortak_bilesenler_cekirdek::{HassasMetin, SunumYapilandirmasi, VeriHassasiyeti};
    use ui_test_destegi::{pencereyi_etkinlestir, test_baglamini_baslat, tusu_bas_birak};

    fn etiket(metin: &str) -> GuvenliDegerTemsili {
        GuvenliDegerTemsili::olustur(
            HassasMetin::yeni(metin),
            SunumYapilandirmasi::yeni().derle().unwrap(),
        )
    }

    #[test]
    fn sonlu_olmayan_ve_yinelenen_veri_sessizce_dusurulmez() {
        let butce = GrafikButcesi::default();
        assert_eq!(
            veriyi_dogrula(
                &[GrafikNoktasi::yeni(
                    "a",
                    Nokta::yeni(f64::NAN, 1.0),
                    etiket("a")
                )],
                butce
            ),
            Err(GrafikHatasi::SonluOlmayanDeger)
        );
        assert_eq!(
            veriyi_dogrula(
                &[
                    GrafikNoktasi::yeni("a", Nokta::yeni(0.0, 1.0), etiket("a")),
                    GrafikNoktasi::yeni("a", Nokta::yeni(1.0, 2.0), etiket("b")),
                ],
                butce,
            ),
            Err(GrafikHatasi::YinelenenKimlik)
        );
    }

    #[test]
    fn hassas_etiket_tablo_ve_csvde_guvenli_kanaldadir() {
        let hassas = GuvenliDegerTemsili::olustur(
            HassasMetin::yeni("=Ayşe Yılmaz"),
            SunumYapilandirmasi::yeni()
                .veri_hassasiyeti(VeriHassasiyeti::Kisisel)
                .derle()
                .unwrap(),
        );
        let noktalar = vec![GrafikNoktasi::yeni("a", Nokta::yeni(1.0, 2.0), hassas)];
        veriyi_dogrula(&noktalar, GrafikButcesi::default()).unwrap();
        let satir = tablo_satiri(&noktalar[0]);
        assert!(!satir.maskeli_etiket.contains("Ayşe"));
        assert!(!format!("{noktalar:?}").contains("Ayşe"));
        assert!(formul_guvenli("=SUM(A1)").starts_with('\''));
    }

    #[test]
    fn butceler_sinirda_kabul_asimda_red() {
        let butce = GrafikButcesi {
            azami_nokta: 1,
            azami_tahmini_bayt: 1024,
            azami_etiket_bayt: 8,
            azami_cizim_noktasi: 3,
            azami_csv_bayt: 128,
        };
        assert!(
            veriyi_dogrula(
                &[GrafikNoktasi::yeni("a", Nokta::yeni(0.0, 0.0), etiket("a"))],
                butce
            )
            .is_ok()
        );
        assert_eq!(
            veriyi_dogrula(
                &[
                    GrafikNoktasi::yeni("a", Nokta::yeni(0.0, 0.0), etiket("a")),
                    GrafikNoktasi::yeni("b", Nokta::yeni(1.0, 1.0), etiket("b")),
                ],
                butce,
            ),
            Err(GrafikHatasi::NoktaButcesiAsildi)
        );
    }

    #[gpui::test]
    fn pointer_ve_klavye_ayni_etkin_secim_ve_zoom_durumuna_baglidir(cx: &mut TestAppContext) {
        test_baglamini_baslat(cx, |tema, cx| {
            baslat(
                OrtakBilesenAyarlari {
                    tema_saglayici: Some(tema),
                    ..OrtakBilesenAyarlari::default()
                },
                cx,
            )
            .expect("test teması geçerli");
        });
        let (grafik, cx) = cx.add_window_view(|_, cx| {
            CizgiGrafik::yeni(
                "Etkileşim testi",
                vec![
                    GrafikNoktasi::yeni("a", Nokta::yeni(0.0, 0.0), etiket("A")),
                    GrafikNoktasi::yeni("b", Nokta::yeni(1.0, 1.0), etiket("B")),
                    GrafikNoktasi::yeni("c", Nokta::yeni(2.0, 2.0), etiket("C")),
                ],
                cx,
            )
            .expect("test verisi geçerli")
        });
        pencereyi_etkinlestir(cx);
        let sinir = cx
            .debug_bounds("faz8-cizgi-grafik")
            .expect("grafik çizildi");
        cx.simulate_click(
            point(sinir.center().x, sinir.top() + px(112.0)),
            Modifiers::none(),
        );
        grafik.read_with(cx, |grafik, _| {
            assert_eq!(grafik.etkin().map(|nokta| nokta.kimlik.deger()), Some("b"));
            assert!(grafik.secili.contains(&KararliKimlik::from("b")));
        });

        grafik.update_in(cx, |grafik, window, cx| grafik.odak.focus(window, cx));
        tusu_bas_birak(cx, "right");
        tusu_bas_birak(cx, "space");
        tusu_bas_birak(cx, "+");
        grafik.read_with(cx, |grafik, _| {
            assert_eq!(grafik.etkin().map(|nokta| nokta.kimlik.deger()), Some("c"));
            assert!(grafik.secili.contains(&KararliKimlik::from("c")));
            assert!(grafik.gorunum_x.is_some());
        });
    }
}
