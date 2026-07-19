//! Ölçekler — `echarts/src/scale` dizininin portu.
//!
//! Her ölçek, veri uzayındaki bir kapsamı `[0, 1]` birim aralığına eşler ve
//! eksen üzerinde gösterilecek "güzel" çentikleri üretir.

pub mod aralik;
pub mod kategorik;
pub mod log;
pub mod zaman;

pub use aralik::AralıkÖlçeği;
pub use kategorik::KategorikÖlçek;
pub use log::LogÖlçeği;
pub use zaman::ZamanÖlçeği;

/// Eksen üzerindeki tek bir çentik (`ScaleTick` karşılığı).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Çentik {
    pub değer: f64,
}

/// Tüm ölçek türlerini saran toplam tip (`Scale` taban sınıfının karşılığı).
#[derive(Clone, Debug)]
pub enum Ölçek {
    Aralık(AralıkÖlçeği),
    Kategorik(KategorikÖlçek),
    Log(LogÖlçeği),
    Zaman(ZamanÖlçeği),
}

impl Ölçek {
    /// Veri uzayındaki kapsam `[en_az, en_çok]`.
    pub fn kapsam(&self) -> [f64; 2] {
        match self {
            Ölçek::Aralık(ö) => ö.kapsam,
            Ölçek::Kategorik(ö) => ö.kapsam(),
            Ölçek::Log(ö) => ö.veri_kapsamı(),
            Ölçek::Zaman(ö) => ö.kapsam,
        }
    }

    /// Veri değerini `[0, 1]` birim aralığına oranlar (`Scale#normalize`).
    pub fn oranla(&self, değer: f64) -> f64 {
        match self {
            Ölçek::Aralık(ö) => ö.oranla(değer),
            Ölçek::Kategorik(ö) => ö.oranla(değer),
            Ölçek::Log(ö) => ö.oranla(değer),
            Ölçek::Zaman(ö) => ö.oranla(değer),
        }
    }

    /// `[0, 1]` birim aralığındaki oranı veri değerine çevirir (`Scale#scale`).
    pub fn orandan(&self, oran: f64) -> f64 {
        match self {
            Ölçek::Aralık(ö) => ö.orandan(oran),
            Ölçek::Kategorik(ö) => ö.orandan(oran),
            Ölçek::Log(ö) => ö.orandan(oran),
            Ölçek::Zaman(ö) => ö.orandan(oran),
        }
    }

    /// Eksen çentikleri (`Scale#getTicks`).
    pub fn çentikler(&self) -> Vec<Çentik> {
        match self {
            Ölçek::Aralık(ö) => ö.çentikler(),
            Ölçek::Kategorik(ö) => ö.çentikler(),
            Ölçek::Log(ö) => ö.çentikler(),
            Ölçek::Zaman(ö) => ö.çentikler(),
        }
    }

    /// Ana çentikler arasındaki ara (minör) çentik değerleri
    /// (`Scale#getMinorTicks`). ECharts, log ölçeğinde de ardışık ana
    /// çentiklerin *ham değer* aralığını eşit parçalara böler; bulunan değerler
    /// daha sonra log koordinatına taşındığı için ekrandaki aralıklar eşit
    /// görünmez. Kategori ve zaman ölçeklerinde ara çentik üretilmez.
    pub fn ara_çentikler(&self, bölme_sayısı: usize) -> Vec<f64> {
        let bölme = bölme_sayısı.max(2);
        let ana: Vec<f64> = self.çentikler().iter().map(|ç| ç.değer).collect();
        let mut sonuç = Vec::new();
        match self {
            Ölçek::Aralık(_) | Ölçek::Log(_) => {
                for çift in ana.windows(2) {
                    if let [a, b] = çift {
                        let adım = (b - a) / bölme as f64;
                        for i in 1..bölme {
                            sonuç.push(a + adım * i as f64);
                        }
                    }
                }
            }
            Ölçek::Kategorik(_) | Ölçek::Zaman(_) => {}
        }
        sonuç
    }

    /// Çentik değerinin görüntülenecek etiketi (`Scale#getLabel`).
    pub fn etiket(&self, değer: f64) -> String {
        match self {
            Ölçek::Aralık(ö) => ö.etiket(değer),
            Ölçek::Kategorik(ö) => ö.etiket(değer),
            Ölçek::Log(ö) => ö.etiket(değer),
            Ölçek::Zaman(ö) => ö.etiket(değer),
        }
    }

    /// Kategorik (sırasal) ölçek mi?
    pub fn kategorik_mi(&self) -> bool {
        matches!(self, Ölçek::Kategorik(_))
    }

    /// Kategori sayısı (kategorik olmayanlarda 0).
    pub fn kategori_sayısı(&self) -> usize {
        match self {
            Ölçek::Kategorik(ö) => ö.kategoriler.len(),
            _ => 0,
        }
    }

    /// Kategori adının sıra değerini döndürür.
    pub fn kategori_sırası(&self, ad: &str) -> Option<f64> {
        match self {
            Ölçek::Kategorik(ölçek) => ölçek
                .kategoriler
                .iter()
                .position(|kategori| kategori == ad)
                .map(|sıra| sıra as f64),
            _ => None,
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn log_ara_centikleri_ham_deger_araligini_boler() {
        let ölçek = Ölçek::Log(LogÖlçeği::kur(
            [0.001, 0.01],
            10.0,
            Some(0.001),
            Some(0.01),
            5,
        ));
        let çentikler = ölçek.ara_çentikler(5);
        assert_eq!(çentikler.len(), 4);
        for (gerçek, beklenen) in çentikler.iter().zip([0.0028, 0.0046, 0.0064, 0.0082]) {
            assert!((*gerçek - beklenen).abs() < 1e-12);
        }
    }
}
