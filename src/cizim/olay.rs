//! Olay sistemi — ECharts'ın `chart.on('click', …)` API'sinin gpui
//! karşılığı. Boyama sırasında her serinin öğeleri için isabet bölgeleri
//! toplanır; fare olayları bu bölgelerle eşlenip [`GrafikOlayı`] olarak
//! gpui `EventEmitter` üzerinden yayımlanır.

use std::collections::BTreeMap;

use crate::koordinat::Dikdörtgen;

/// Bir veri öğesinin tıklama/isabet geometrisi.
#[derive(Clone, Debug)]
pub enum İsabetGeometrisi {
    Dikdörtgen(Dikdörtgen),
    /// Kapalı çokgen; funnel dilimleri gibi sınır kutusu boşluk içeren
    /// şekillerde olay hedefini gerçek boyalı alanla aynı tutar.
    Çokgen {
        noktalar: Vec<(f32, f32)>,
    },
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
    /// Çizgi/polyline etrafındaki isabet bandı (`series.lines`, graph edge).
    ÇokluÇizgi {
        noktalar: Vec<(f32, f32)>,
        tolerans: f32,
    },
}

impl İsabetGeometrisi {
    pub fn içeriyor_mu(&self, n: (f32, f32)) -> bool {
        match self {
            İsabetGeometrisi::Dikdörtgen(d) => d.içeriyor_mu(n),
            İsabetGeometrisi::Çokgen { noktalar } => {
                if noktalar.len() < 3 {
                    return false;
                }
                let mut içeride = false;
                let mut önceki = noktalar.len() - 1;
                for sıra in 0..noktalar.len() {
                    let (xi, yi) = noktalar[sıra];
                    let (xj, yj) = noktalar[önceki];
                    if ((yi > n.1) != (yj > n.1)) && n.0 < (xj - xi) * (n.1 - yi) / (yj - yi) + xi {
                        içeride = !içeride;
                    }
                    önceki = sıra;
                }
                içeride
            }
            İsabetGeometrisi::Daire { merkez, yarıçap } => {
                let dx = n.0 - merkez.0;
                let dy = n.1 - merkez.1;
                dx * dx + dy * dy <= yarıçap * yarıçap
            }
            İsabetGeometrisi::Halka {
                merkez,
                iç_yarıçap,
                dış_yarıçap,
                açı0,
                açı1,
            } => {
                let dx = n.0 - merkez.0;
                let dy = n.1 - merkez.1;
                let uzaklık = (dx * dx + dy * dy).sqrt();
                if uzaklık < *iç_yarıçap || uzaklık > *dış_yarıçap {
                    return false;
                }
                let tau = std::f32::consts::TAU;
                let (a0, a1) = if açı1 >= açı0 {
                    (*açı0, *açı1)
                } else {
                    (*açı1, *açı0)
                };
                let göreli = (dy.atan2(dx) - a0).rem_euclid(tau);
                göreli <= a1 - a0
            }
            İsabetGeometrisi::ÇokluÇizgi { noktalar, tolerans } => {
                noktalar.windows(2).any(|uçlar| match uçlar {
                    [a, b] => noktadan_parçaya_uzaklık(n, *a, *b) <= *tolerans,
                    _ => false,
                })
            }
        }
    }

    /// Geometrinin temsilî merkezi (fırça seçimi için).
    pub fn merkez(&self) -> (f32, f32) {
        match self {
            İsabetGeometrisi::Dikdörtgen(d) => d.merkez(),
            İsabetGeometrisi::Çokgen { noktalar } => {
                if noktalar.is_empty() {
                    return (0.0, 0.0);
                }
                let toplam = noktalar.iter().fold((0.0, 0.0), |toplam, nokta| {
                    (toplam.0 + nokta.0, toplam.1 + nokta.1)
                });
                (
                    toplam.0 / noktalar.len() as f32,
                    toplam.1 / noktalar.len() as f32,
                )
            }
            İsabetGeometrisi::Daire { merkez, .. } => *merkez,
            İsabetGeometrisi::Halka {
                merkez,
                iç_yarıçap,
                dış_yarıçap,
                açı0,
                açı1,
            } => {
                let orta_açı = (açı0 + açı1) / 2.0;
                let orta_yarıçap = (iç_yarıçap + dış_yarıçap) / 2.0;
                (
                    merkez.0 + orta_yarıçap * orta_açı.cos(),
                    merkez.1 + orta_yarıçap * orta_açı.sin(),
                )
            }
            İsabetGeometrisi::ÇokluÇizgi { noktalar, .. } => {
                let Some(ilk) = noktalar.first().copied() else {
                    return (0.0, 0.0);
                };
                let Some(son) = noktalar.last().copied() else {
                    return ilk;
                };
                ((ilk.0 + son.0) / 2.0, (ilk.1 + son.1) / 2.0)
            }
        }
    }

    /// Geometriyi verilen kadar öteler (yüzey-yerel → pencere-mutlak dönüşümü).
    pub fn kaydır(&self, dx: f32, dy: f32) -> İsabetGeometrisi {
        match self {
            İsabetGeometrisi::Dikdörtgen(d) => İsabetGeometrisi::Dikdörtgen(Dikdörtgen::yeni(
                d.x + dx,
                d.y + dy,
                d.genişlik,
                d.yükseklik,
            )),
            İsabetGeometrisi::Çokgen { noktalar } => İsabetGeometrisi::Çokgen {
                noktalar: noktalar
                    .iter()
                    .map(|nokta| (nokta.0 + dx, nokta.1 + dy))
                    .collect(),
            },
            İsabetGeometrisi::Daire { merkez, yarıçap } => İsabetGeometrisi::Daire {
                merkez: (merkez.0 + dx, merkez.1 + dy),
                yarıçap: *yarıçap,
            },
            İsabetGeometrisi::Halka {
                merkez,
                iç_yarıçap,
                dış_yarıçap,
                açı0,
                açı1,
            } => İsabetGeometrisi::Halka {
                merkez: (merkez.0 + dx, merkez.1 + dy),
                iç_yarıçap: *iç_yarıçap,
                dış_yarıçap: *dış_yarıçap,
                açı0: *açı0,
                açı1: *açı1,
            },
            İsabetGeometrisi::ÇokluÇizgi { noktalar, tolerans } => {
                İsabetGeometrisi::ÇokluÇizgi {
                    noktalar: noktalar
                        .iter()
                        .map(|nokta| (nokta.0 + dx, nokta.1 + dy))
                        .collect(),
                    tolerans: *tolerans,
                }
            }
        }
    }
}

fn noktadan_parçaya_uzaklık(
    nokta: (f32, f32), başlangıç: (f32, f32), bitiş: (f32, f32)
) -> f32 {
    let dx = bitiş.0 - başlangıç.0;
    let dy = bitiş.1 - başlangıç.1;
    let uzunluk_kare = dx * dx + dy * dy;
    if uzunluk_kare <= f32::EPSILON {
        return ((nokta.0 - başlangıç.0).powi(2) + (nokta.1 - başlangıç.1).powi(2)).sqrt();
    }
    let t = (((nokta.0 - başlangıç.0) * dx + (nokta.1 - başlangıç.1) * dy) / uzunluk_kare)
        .clamp(0.0, 1.0);
    let en_yakın = (başlangıç.0 + dx * t, başlangıç.1 + dy * t);
    ((nokta.0 - en_yakın.0).powi(2) + (nokta.1 - en_yakın.1).powi(2)).sqrt()
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
    /// Serbest `graphic` öğesine tıklandı. Kimlik ve ad, ECharts olay
    /// parametresindeki `element.id` / `name` değerleridir.
    GrafikÖğesiTıklandı {
        kimlik: Option<String>,
        ad: Option<String>,
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
    /// Sürekli görsel eşleme aralığı değişti (`'dataRangeSelected'`).
    GörselAralıkDeğişti {
        /// `visualMap` bileşen sırası.
        sıra: usize,
        alt: f64,
        üst: f64,
    },
    /// Parçalı görsel eşleme seçimi değişti (`'dataRangeSelected'`).
    GörselParçalarDeğişti {
        /// `visualMap` bileşen sırası.
        sıra: usize,
        /// Düşükten yükseğe parça sırası → seçili durumu.
        seçili: BTreeMap<usize, bool>,
    },
    /// Fırça seçimi tamamlandı (`'brushselected'`): kapsanan öğeler.
    FırçaSeçildi {
        /// `(seri sırası, veri sırası)` çiftleri.
        öğeler: Vec<(usize, usize)>,
    },
    /// Araç kutusundan "geri yükle" tıklandı (`'restore'`).
    GeriYüklendi,
    /// Toolbox `dataView`; güvenli yerel görünümü açması için ev sahibine
    /// yapılandırılmış istek gönderir.
    VeriGörünümüİstendi,
    /// Toolbox `magicType`; ev sahibi/çalışma zamanı seri dönüşümünü uygular.
    SihirliTürİstendi { tür: SihirliSeriTürü },
    /// Zaman şeridinde kare değişti (`'timelinechanged'`).
    ZamanKaresiDeğişti { sıra: usize },
    /// Araç kutusundan grafik SVG olarak kaydedildi (`saveAsImage`).
    SvgKaydedildi { yol: String },
    /// Araç kutusundan grafik PNG olarak kaydedildi (`saveAsImage`).
    PngKaydedildi { yol: String },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SihirliSeriTürü {
    Çizgi,
    Sütun,
    /// Uyumlu sütun/çizgi serilerini ortak bir yığına al (`magicType: stack`).
    Yığın,
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn cokgen_isabeti_sinir_kutusu_boslugunu_hedef_saymaz() {
        let yamuk = İsabetGeometrisi::Çokgen {
            noktalar: vec![(0.0, 0.0), (10.0, 0.0), (7.0, 10.0), (3.0, 10.0)],
        };

        assert!(yamuk.içeriyor_mu((5.0, 5.0)));
        assert!(!yamuk.içeriyor_mu((0.5, 9.0)));
        assert_eq!(yamuk.merkez(), (5.0, 5.0));
        assert!(yamuk.kaydır(20.0, 30.0).içeriyor_mu((25.0, 35.0)));
    }
}
