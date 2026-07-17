//! Olay sistemi — ECharts'ın `chart.on('click', …)` API'sinin gpui
//! karşılığı. Boyama sırasında her serinin öğeleri için isabet bölgeleri
//! toplanır; fare olayları bu bölgelerle eşlenip [`GrafikOlayı`] olarak
//! gpui `EventEmitter` üzerinden yayımlanır.

use crate::koordinat::Dikdörtgen;

/// Bir veri öğesinin tıklama/isabet geometrisi.
#[derive(Clone, Debug)]
pub enum İsabetGeometrisi {
    Dikdörtgen(Dikdörtgen),
    Daire {
        merkez: (f32, f32),
        yarıçap: f32,
    },
    /// Halka parçası (pasta dilimi); açılar radyan, ekran koordinatı.
    Halka {
        merkez: (f32, f32),
        iç_yarıçap: f32,
        dış_yarıçap: f32,
        açı0: f32,
        açı1: f32,
    },
}

impl İsabetGeometrisi {
    pub fn içeriyor_mu(&self, n: (f32, f32)) -> bool {
        match self {
            İsabetGeometrisi::Dikdörtgen(d) => d.içeriyor_mu(n),
            İsabetGeometrisi::Daire { merkez, yarıçap } => {
                let dx = n.0 - merkez.0;
                let dy = n.1 - merkez.1;
                dx * dx + dy * dy <= yarıçap * yarıçap
            }
            İsabetGeometrisi::Halka { merkez, iç_yarıçap, dış_yarıçap, açı0, açı1 } => {
                let dx = n.0 - merkez.0;
                let dy = n.1 - merkez.1;
                let uzaklık = (dx * dx + dy * dy).sqrt();
                if uzaklık < *iç_yarıçap || uzaklık > *dış_yarıçap {
                    return false;
                }
                let tau = std::f32::consts::TAU;
                let (a0, a1) = if açı1 >= açı0 { (*açı0, *açı1) } else { (*açı1, *açı0) };
                let göreli = (dy.atan2(dx) - a0).rem_euclid(tau);
                göreli <= a1 - a0
            }
        }
    }

    /// Geometrinin temsilî merkezi (fırça seçimi için).
    pub fn merkez(&self) -> (f32, f32) {
        match self {
            İsabetGeometrisi::Dikdörtgen(d) => d.merkez(),
            İsabetGeometrisi::Daire { merkez, .. } => *merkez,
            İsabetGeometrisi::Halka { merkez, iç_yarıçap, dış_yarıçap, açı0, açı1 } => {
                let orta_açı = (açı0 + açı1) / 2.0;
                let orta_yarıçap = (iç_yarıçap + dış_yarıçap) / 2.0;
                (
                    merkez.0 + orta_yarıçap * orta_açı.cos(),
                    merkez.1 + orta_yarıçap * orta_açı.sin(),
                )
            }
        }
    }

    /// Geometriyi verilen kadar öteler (yüzey-yerel → pencere-mutlak dönüşümü).
    pub fn kaydır(&self, dx: f32, dy: f32) -> İsabetGeometrisi {
        match self {
            İsabetGeometrisi::Dikdörtgen(d) => İsabetGeometrisi::Dikdörtgen(
                Dikdörtgen::yeni(d.x + dx, d.y + dy, d.genişlik, d.yükseklik),
            ),
            İsabetGeometrisi::Daire { merkez, yarıçap } => İsabetGeometrisi::Daire {
                merkez: (merkez.0 + dx, merkez.1 + dy),
                yarıçap: *yarıçap,
            },
            İsabetGeometrisi::Halka { merkez, iç_yarıçap, dış_yarıçap, açı0, açı1 } => {
                İsabetGeometrisi::Halka {
                    merkez: (merkez.0 + dx, merkez.1 + dy),
                    iç_yarıçap: *iç_yarıçap,
                    dış_yarıçap: *dış_yarıçap,
                    açı0: *açı0,
                    açı1: *açı1,
                }
            }
        }
    }
}

/// Boyama sırasında toplanan, tıklanabilir bir veri öğesi bölgesi.
#[derive(Clone, Debug)]
pub struct İsabetBölgesi {
    pub seri_sırası: usize,
    pub veri_sırası: usize,
    pub seri_adı: Option<String>,
    /// Öğenin adı (pasta dilimi, kategorili veri).
    pub ad: Option<String>,
    pub değer: Option<f64>,
    pub geometri: İsabetGeometrisi,
}

/// Grafikten yayımlanan olaylar (ECharts `chart.on(...)` karşılığı).
/// `GrafikGörünümü`, gpui `EventEmitter<GrafikOlayı>` uygular; şöyle
/// dinlenir:
///
/// ```ignore
/// cx.subscribe(&grafik, |_, _, olay: &GrafikOlayı, _| {
///     println!("{olay:?}");
/// }).detach();
/// ```
#[derive(Clone, Debug)]
pub enum GrafikOlayı {
    /// Bir veri öğesine tıklandı (`'click'`).
    ÖğeTıklandı {
        seri_sırası: usize,
        veri_sırası: usize,
        seri_adı: Option<String>,
        ad: Option<String>,
        değer: Option<f64>,
    },
    /// Gösterge öğesi tıklanıp bir ad açıldı/kapandı
    /// (`'legendselectchanged'`).
    GöstergeDeğişti { ad: String, görünür: bool },
    /// Veri yakınlaştırma penceresi değişti (`'datazoom'`).
    YakınlaştırmaDeğişti {
        /// `veri_yakınlaştırmaları` içindeki sıra.
        sıra: usize,
        /// Yüzde `0..=100`.
        başlangıç: f32,
        bitiş: f32,
    },
    /// Fırça seçimi tamamlandı (`'brushselected'`): kapsanan öğeler.
    FırçaSeçildi {
        /// `(seri sırası, veri sırası)` çiftleri.
        öğeler: Vec<(usize, usize)>,
    },
    /// Araç kutusundan "geri yükle" tıklandı (`'restore'`).
    GeriYüklendi,
    /// Zaman şeridinde kare değişti (`'timelinechanged'`).
    ZamanKaresiDeğişti { sıra: usize },
}
