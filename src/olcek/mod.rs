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
    /// (`Scale#getMinorTicks`). Aralık ölçeğinde eşit, log ölçeğinde
    /// log-uzayında eşit (geometrik) bölme uygulanır; kategori ve zaman
    /// ölçeklerinde üretilmez.
    pub fn ara_çentikler(&self, bölme_sayısı: usize) -> Vec<f64> {
        let bölme = bölme_sayısı.max(2);
        let ana: Vec<f64> = self.çentikler().iter().map(|ç| ç.değer).collect();
        let mut sonuç = Vec::new();
        match self {
            Ölçek::Aralık(_) => {
                for çift in ana.windows(2) {
                    if let [a, b] = çift {
                        let adım = (b - a) / bölme as f64;
                        for i in 1..bölme {
                            sonuç.push(a + adım * i as f64);
                        }
                    }
                }
            }
            Ölçek::Log(_) => {
                for çift in ana.windows(2) {
                    if let [a, b] = çift {
                        if *a <= 0.0 || *b <= 0.0 {
                            continue;
                        }
                        let oran = (b / a).powf(1.0 / bölme as f64);
                        let mut değer = *a;
                        for _ in 1..bölme {
                            değer *= oran;
                            sonuç.push(değer);
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
}
