use gpui::{Context, Entity, EventEmitter, Render, SharedString, Subscription, Window, prelude::*};

use crate::{
    CizgiGrafik, CizgiGrafikOlayi, GrafikButcesi, GrafikHatasi, GrafikNoktasi, GrafikTuru,
    cizgi_grafik::veriyi_dogrula,
};

macro_rules! grafik_yuzeyi {
    ($(#[$meta:meta])* $ad:ident, $tur:ident) => {
        $(#[$meta])*
        pub struct $ad {
            ic: Entity<CizgiGrafik>,
            _abonelik: Subscription,
        }

        impl $ad {
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
                let butce = butce.dogrula()?;
                veriyi_dogrula(&noktalar, butce)?;
                let ad = ad.into();
                let ic = cx.new(move |cx| {
                    CizgiGrafik::dogrulanmis(ad, noktalar, butce, GrafikTuru::$tur, cx)
                });
                let abonelik = cx.subscribe(&ic, |_, _, olay: &CizgiGrafikOlayi, cx| {
                    cx.emit(olay.clone());
                });
                Ok(Self {
                    ic,
                    _abonelik: abonelik,
                })
            }

            pub fn grafik(&self) -> &Entity<CizgiGrafik> {
                &self.ic
            }
        }

        impl EventEmitter<CizgiGrafikOlayi> for $ad {}

        impl Render for $ad {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                self.ic.clone()
            }
        }
    };
}

grafik_yuzeyi!(
    /// Kategorik değerleri sıfır tabanlı ayrı çubuklarla çizer.
    CubukGrafik,
    Cubuk
);
grafik_yuzeyi!(
    /// Trend çizgisinin altındaki alanı doldurur; min/max örneklemesi kullanır.
    AlanGrafigi,
    Alan
);
grafik_yuzeyi!(
    /// Pozitif payları açısal dilimlere dönüştürür.
    PastaGrafik,
    Pasta
);
grafik_yuzeyi!(
    /// Pozitif payları merkez boşluklu açısal dilimlere dönüştürür.
    HalkaGrafik,
    Halka
);
grafik_yuzeyi!(
    /// İki boyutlu noktaları hücre tabanlı örnekleyerek çizer.
    DagilimGrafigi,
    Dagilim
);
grafik_yuzeyi!(
    /// Üçüncü sayısal boyutu nokta alanına dönüştürür.
    BaloncukGrafik,
    Baloncuk
);
grafik_yuzeyi!(
    /// Aynı ölçekte çubuk ve trend çizgisini birlikte çizer.
    BirlesikGrafik,
    Birlesik
);
grafik_yuzeyi!(
    /// Hücre yoğunluğunu opaklıkla kodlayan erişilebilir ısı haritasıdır.
    IsiHaritasi,
    IsiHaritasi
);
grafik_yuzeyi!(
    /// Pozitif ağırlıkları alan paylarına dönüştüren ağaç haritasıdır.
    AgacHaritasi,
    AgacHaritasi
);
grafik_yuzeyi!(
    /// Ardışık akış ağırlıklarını genişlik kodlu eğrilerle çizer.
    SankeyGrafigi,
    Sankey
);
grafik_yuzeyi!(
    /// 0–100 aralığındaki ilk değeri yarım daire ölçerinde gösterir.
    Gosterge,
    Gosterge
);
grafik_yuzeyi!(
    /// Kompakt, lejantsız trend görünümüdür.
    MiniCizgi,
    MiniCizgi
);
grafik_yuzeyi!(
    /// İlk güvenli değeri ve eğilim payını sunan KPI görünümüdür.
    Kpi,
    Kpi
);
grafik_yuzeyi!(
    /// Pano içinde tek metrik ve özet grafiği sunan kutudur.
    PanoKutusu,
    PanoKutusu
);

#[cfg(test)]
mod testler {
    use std::any::TypeId;

    use super::*;

    #[test]
    fn grafik_profilleri_alias_degil_ayri_tiplerdir() {
        let tipler = [
            TypeId::of::<CubukGrafik>(),
            TypeId::of::<AlanGrafigi>(),
            TypeId::of::<PastaGrafik>(),
            TypeId::of::<HalkaGrafik>(),
            TypeId::of::<DagilimGrafigi>(),
            TypeId::of::<BaloncukGrafik>(),
            TypeId::of::<BirlesikGrafik>(),
            TypeId::of::<IsiHaritasi>(),
            TypeId::of::<AgacHaritasi>(),
            TypeId::of::<SankeyGrafigi>(),
            TypeId::of::<Gosterge>(),
            TypeId::of::<MiniCizgi>(),
            TypeId::of::<Kpi>(),
            TypeId::of::<PanoKutusu>(),
        ];
        for (sira, tip) in tipler.iter().enumerate() {
            assert!(!tipler[..sira].contains(tip));
        }
        assert!(!tipler.contains(&TypeId::of::<CizgiGrafik>()));
    }
}
