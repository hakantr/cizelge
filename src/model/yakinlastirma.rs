//! Veri yakınlaştırma seçenekleri — ECharts `dataZoom` bileşeninin
//! karşılığı. `İç` tür fare tekerleği/sürüklemeyle, `Sürgü` tür alt
//! şeritteki tutamaçlarla pencereyi değiştirir.

use crate::model::Uzunluk;
use crate::model::seri::Sembol;

/// `dataZoom.startValue` / `endValue` eksen ucu.
#[derive(Clone, PartialEq, Debug)]
pub enum YakınlaştırmaDeğeri {
    /// Değer ve zaman eksenlerinde sayısal değer; kategori ekseninde sıra.
    Sayı(f64),
    /// Kategori adı.
    Kategori(String),
}

impl YakınlaştırmaDeğeri {
    /// Eksenin sayısal koordinat uzayına çözer.
    pub fn çöz(&self, kategoriler: &[String]) -> Option<f64> {
        match self {
            YakınlaştırmaDeğeri::Sayı(değer) => değer.is_finite().then_some(*değer),
            YakınlaştırmaDeğeri::Kategori(ad) => kategoriler
                .iter()
                .position(|aday| aday == ad)
                .map(|sıra| sıra as f64),
        }
    }
}

macro_rules! sayı_yakınlaştırma_değeri {
    ($($tür:ty),+ $(,)?) => {
        $(
            impl From<$tür> for YakınlaştırmaDeğeri {
                fn from(değer: $tür) -> Self {
                    YakınlaştırmaDeğeri::Sayı(değer as f64)
                }
            }
        )+
    };
}

sayı_yakınlaştırma_değeri!(f64, f32, i64, i32, usize, u64, u32);

impl From<String> for YakınlaştırmaDeğeri {
    fn from(değer: String) -> Self {
        YakınlaştırmaDeğeri::Kategori(değer)
    }
}

impl From<&str> for YakınlaştırmaDeğeri {
    fn from(değer: &str) -> Self {
        YakınlaştırmaDeğeri::Kategori(değer.to_owned())
    }
}

/// `dataZoom.filterMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum YakınlaştırmaSüzmeKipi {
    /// Pencere dışındaki satırları veri listesinden süzer (`filter`).
    #[default]
    Süz,
    /// Bütün boyutları aynı tarafta kalan satırları süzer (`weakFilter`).
    ZayıfSüz,
    /// Pencere dışındaki değerleri boş değer yapar (`empty`).
    Boşalt,
    /// Veriyi değiştirmez; yalnız eksen kapsamını daraltır (`none`).
    Yok,
}

/// Yakınlaştırma türü (`dataZoom.type`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum YakınlaştırmaTürü {
    /// Izgara içinde tekerlek + sürükleme (`'inside'`).
    #[default]
    İç,
    /// Alt şerit sürgüsü (`'slider'`).
    Sürgü,
}

/// Veri yakınlaştırma tanımı (`dataZoom` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct VeriYakınlaştırma {
    pub tür: YakınlaştırmaTürü,
    /// `dataZoom.show`; `false` bileşeni görünmez kılar, fakat pencereyi ve
    /// programatik `dataZoom` güncellemelerini etkin tutar.
    pub göster: bool,
    /// Bağlı ilk x ekseninin sırası (`xAxisIndex`). Tek eksenli eski API'nin
    /// kaynak uyumluluğu için açık tutulur; ek hedefler
    /// `ek_x_eksen_sıraları` içindedir.
    pub x_eksen_sırası: usize,
    /// `xAxisIndex: [..]` dizisindeki ilk öğeden sonraki hedefler.
    pub ek_x_eksen_sıraları: Vec<usize>,
    /// `Some` ise yakınlaştırma x yerine bu y eksenine bağlıdır
    /// (`yAxisIndex`) ve sürgü dikey çizilir.
    pub y_eksen_sırası: Option<usize>,
    /// `yAxisIndex: [..]` dizisindeki ilk öğeden sonraki hedefler.
    pub ek_y_eksen_sıraları: Vec<usize>,
    /// Pencere başlangıcı, yüzde `0..=100` (`start`).
    pub başlangıç: f32,
    /// Pencere bitişi, yüzde `0..=100` (`end`).
    pub bitiş: f32,
    /// Değer tabanlı başlangıç; verildiyse yüzde başlangıcın önüne geçer.
    pub başlangıç_değeri: Option<YakınlaştırmaDeğeri>,
    /// Değer tabanlı bitiş; verildiyse yüzde bitişin önüne geçer.
    pub bitiş_değeri: Option<YakınlaştırmaDeğeri>,
    /// Pencere dışındaki verinin işlenme biçimi (`filterMode`).
    pub süzme_kipi: YakınlaştırmaSüzmeKipi,
    /// Sürükleme sırasında verinin eşzamanlı güncellenmesi (`realtime`).
    /// Kapalı olduğunda etkileşim katmanı pencereyi bırakma anında uygular.
    pub gerçek_zamanlı: bool,
    /// İsteğe bağlı kutu yerleşimi (`left/top/bottom/width/height`).
    pub sol: Option<Uzunluk>,
    pub sağ: Option<Uzunluk>,
    pub üst: Option<Uzunluk>,
    pub alt: Option<Uzunluk>,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    /// Veri gölgesi (`showDataShadow`).
    pub veri_gölgesi: bool,
    /// Sürgü uçlarının özel simgesi (`handleIcon`). `None`, ECharts'ın
    /// öntanımlı tutamaç yolunu kullanır.
    pub tutamaç_simgesi: Option<Sembol>,
    /// Tutamaç boyu (`handleSize`); yüzde değer sürgünün kısa kenarına göre
    /// çözülür.
    pub tutamaç_boyutu: Uzunluk,
}

impl Default for VeriYakınlaştırma {
    fn default() -> Self {
        VeriYakınlaştırma {
            tür: YakınlaştırmaTürü::İç,
            göster: true,
            x_eksen_sırası: 0,
            ek_x_eksen_sıraları: Vec::new(),
            y_eksen_sırası: None,
            ek_y_eksen_sıraları: Vec::new(),
            başlangıç: 0.0,
            bitiş: 100.0,
            başlangıç_değeri: None,
            bitiş_değeri: None,
            süzme_kipi: YakınlaştırmaSüzmeKipi::Süz,
            gerçek_zamanlı: true,
            sol: None,
            sağ: None,
            üst: None,
            alt: None,
            genişlik: None,
            yükseklik: None,
            veri_gölgesi: true,
            tutamaç_simgesi: None,
            tutamaç_boyutu: Uzunluk::Yüzde(100.0),
        }
    }
}

impl VeriYakınlaştırma {
    /// Izgara içi yakınlaştırma (`'inside'`).
    pub fn iç() -> Self {
        Self::default()
    }

    /// Alt şerit sürgüsü (`'slider'`).
    pub fn sürgü() -> Self {
        VeriYakınlaştırma {
            tür: YakınlaştırmaTürü::Sürgü,
            ..Default::default()
        }
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn x_eksen_sırası(mut self, sıra: usize) -> Self {
        self.x_eksen_sırası = sıra;
        self.ek_x_eksen_sıraları.clear();
        self.y_eksen_sırası = None;
        self.ek_y_eksen_sıraları.clear();
        self
    }

    /// Yakınlaştırmayı birden çok x eksenine bağlar
    /// (`xAxisIndex: [0, 1, ...]`). Boş hedef listesi ECharts'ın otomatik
    /// seçimine denk gelmediği için öntanımlı `0` eksenine düşer.
    pub fn x_eksenleri(mut self, sıralar: impl IntoIterator<Item = usize>) -> Self {
        let mut sıralar = sıralar.into_iter();
        self.x_eksen_sırası = sıralar.next().unwrap_or(0);
        self.ek_x_eksen_sıraları = sıralar
            .filter(|sıra| *sıra != self.x_eksen_sırası)
            .collect();
        self.ek_x_eksen_sıraları.sort_unstable();
        self.ek_x_eksen_sıraları.dedup();
        self.y_eksen_sırası = None;
        self.ek_y_eksen_sıraları.clear();
        self
    }

    /// Yakınlaştırmayı y eksenine bağlar (`yAxisIndex`); yön dikey olur.
    pub fn y_eksen_sırası(mut self, sıra: usize) -> Self {
        self.y_eksen_sırası = Some(sıra);
        self.ek_y_eksen_sıraları.clear();
        self.ek_x_eksen_sıraları.clear();
        self
    }

    /// Yakınlaştırmayı birden çok y eksenine bağlar
    /// (`yAxisIndex: [0, 1, ...]`).
    pub fn y_eksenleri(mut self, sıralar: impl IntoIterator<Item = usize>) -> Self {
        let mut sıralar = sıralar.into_iter();
        let ilk = sıralar.next().unwrap_or(0);
        self.y_eksen_sırası = Some(ilk);
        self.ek_y_eksen_sıraları = sıralar.filter(|sıra| *sıra != ilk).collect();
        self.ek_y_eksen_sıraları.sort_unstable();
        self.ek_y_eksen_sıraları.dedup();
        self.ek_x_eksen_sıraları.clear();
        self
    }

    /// Bileşenin hedeflediği x eksenlerini ECharts dizi sırasıyla döndürür.
    pub fn hedef_x_eksenleri(&self) -> impl Iterator<Item = usize> + '_ {
        std::iter::once(self.x_eksen_sırası).chain(self.ek_x_eksen_sıraları.iter().copied())
    }

    /// Bileşenin hedeflediği y eksenlerini ECharts dizi sırasıyla döndürür.
    pub fn hedef_y_eksenleri(&self) -> impl Iterator<Item = usize> + '_ {
        self.y_eksen_sırası
            .into_iter()
            .chain(self.ek_y_eksen_sıraları.iter().copied())
    }

    pub fn x_eksenini_hedefler(&self, sıra: usize) -> bool {
        self.y_eksen_sırası.is_none() && self.hedef_x_eksenleri().any(|hedef| hedef == sıra)
    }

    pub fn y_eksenini_hedefler(&self, sıra: usize) -> bool {
        self.hedef_y_eksenleri().any(|hedef| hedef == sıra)
    }

    /// İki dataZoom bileşeninin aynı eksen kümesini yönettiğini bildirir.
    /// ECharts'ta hedef dizi sırası anlamlı değildir; karşılaştırma bu yüzden
    /// kümeleri sıralayıp yinelenenleri atar.
    pub fn aynı_eksenleri_hedefler(&self, diğer: &Self) -> bool {
        if self.dikey_mi() != diğer.dikey_mi() {
            return false;
        }
        let mut bu: Vec<_> = if self.dikey_mi() {
            self.hedef_y_eksenleri().collect()
        } else {
            self.hedef_x_eksenleri().collect()
        };
        let mut öteki: Vec<_> = if diğer.dikey_mi() {
            diğer.hedef_y_eksenleri().collect()
        } else {
            diğer.hedef_x_eksenleri().collect()
        };
        bu.sort_unstable();
        bu.dedup();
        öteki.sort_unstable();
        öteki.dedup();
        bu == öteki
    }

    pub fn sol(mut self, sol: impl Into<Uzunluk>) -> Self {
        self.sol = Some(sol.into());
        self.sağ = None;
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(sağ.into());
        self.sol = None;
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = Some(üst.into());
        self.alt = None;
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = Some(alt.into());
        self.üst = None;
        self
    }

    pub fn genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(genişlik.into());
        self
    }

    pub fn yükseklik(mut self, yükseklik: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(yükseklik.into());
        self
    }

    pub fn veri_gölgesi(mut self, göster: bool) -> Self {
        self.veri_gölgesi = göster;
        self
    }

    /// Sürgünün iki ucunda kullanılacak özel `handleIcon` simgesini ayarlar.
    /// `Sembol::svg_yolu("path://...")`, ECharts option değerini doğrudan
    /// taşımak için kullanılabilir.
    pub fn tutamaç_simgesi(mut self, simge: Sembol) -> Self {
        self.tutamaç_simgesi = Some(simge);
        self
    }

    /// `handleSize`; örneğin `"80%"` sürgünün 30 px kısa kenarında 24 px
    /// tutamaç yüksekliği üretir.
    pub fn tutamaç_boyutu(mut self, boyut: impl Into<Uzunluk>) -> Self {
        self.tutamaç_boyutu = boyut.into();
        self
    }

    pub fn dikey_mi(&self) -> bool {
        self.y_eksen_sırası.is_some()
    }

    /// Başlangıç penceresi, yüzde.
    pub fn aralık(mut self, başlangıç: f32, bitiş: f32) -> Self {
        self.başlangıç = başlangıç.clamp(0.0, 100.0);
        self.bitiş = bitiş.clamp(self.başlangıç, 100.0);
        self.başlangıç_değeri = None;
        self.bitiş_değeri = None;
        self
    }

    /// `startValue`.
    pub fn başlangıç_değeri(mut self, değer: impl Into<YakınlaştırmaDeğeri>) -> Self {
        self.başlangıç_değeri = Some(değer.into());
        self
    }

    /// `endValue`.
    pub fn bitiş_değeri(mut self, değer: impl Into<YakınlaştırmaDeğeri>) -> Self {
        self.bitiş_değeri = Some(değer.into());
        self
    }

    /// `startValue` / `endValue` çiftini ayarlar.
    pub fn değer_aralığı(
        mut self,
        başlangıç: impl Into<YakınlaştırmaDeğeri>,
        bitiş: impl Into<YakınlaştırmaDeğeri>,
    ) -> Self {
        self.başlangıç_değeri = Some(başlangıç.into());
        self.bitiş_değeri = Some(bitiş.into());
        self
    }

    pub fn süzme_kipi(mut self, kip: YakınlaştırmaSüzmeKipi) -> Self {
        self.süzme_kipi = kip;
        self
    }

    pub fn gerçek_zamanlı(mut self, gerçek_zamanlı: bool) -> Self {
        self.gerçek_zamanlı = gerçek_zamanlı;
        self
    }

    /// Pencere oranları `0..=1`.
    pub fn oranlar(&self) -> (f32, f32) {
        (self.başlangıç / 100.0, self.bitiş / 100.0)
    }

    /// Pencere etkin mi (tam açıklıktan farklı mı)?
    pub fn etkin_mi(&self) -> bool {
        self.başlangıç_değeri.is_some()
            || self.bitiş_değeri.is_some()
            || self.başlangıç > 0.001
            || self.bitiş < 99.999
    }

    /// Yüzde ya da değer uçlarını eksenin ham kapsamına çözer. Dönen ilk
    /// çift değer penceresi, ikinci çift bunun tam kapsamdaki oranlarıdır.
    pub fn pencere_çöz(
        &self,
        kategoriler: &[String],
        tam_kapsam: [f64; 2],
    ) -> Option<([f64; 2], (f32, f32))> {
        if !self.etkin_mi() {
            return None;
        }
        let mut kapsam = tam_kapsam;
        if !kapsam[0].is_finite() || !kapsam[1].is_finite() || kapsam[1] < kapsam[0] {
            kapsam = [0.0, 1.0];
        }
        let açıklık = (kapsam[1] - kapsam[0]).max(0.0);
        let yüzde_başı = kapsam[0] + açıklık * f64::from(self.başlangıç) / 100.0;
        let yüzde_sonu = kapsam[0] + açıklık * f64::from(self.bitiş) / 100.0;
        let başlangıç = self
            .başlangıç_değeri
            .as_ref()
            .and_then(|değer| değer.çöz(kategoriler))
            .unwrap_or(yüzde_başı)
            .clamp(kapsam[0], kapsam[1]);
        let bitiş = self
            .bitiş_değeri
            .as_ref()
            .and_then(|değer| değer.çöz(kategoriler))
            .unwrap_or(yüzde_sonu)
            .clamp(kapsam[0], kapsam[1]);
        if bitiş < başlangıç {
            return None;
        }
        let oranlar = if açıklık > 0.0 {
            (
                ((başlangıç - kapsam[0]) / açıklık) as f32,
                ((bitiş - kapsam[0]) / açıklık) as f32,
            )
        } else {
            (0.0, 1.0)
        };
        Some(([başlangıç, bitiş], oranlar))
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn slider_realtime_kapali_ve_coklu_eksen_hedefleri_korunur() {
        let yakınlaştırma = VeriYakınlaştırma::sürgü()
            .x_eksenleri([0, 1])
            .süzme_kipi(YakınlaştırmaSüzmeKipi::Boşalt)
            .gerçek_zamanlı(false);

        assert!(!yakınlaştırma.gerçek_zamanlı);
        assert_eq!(
            yakınlaştırma.hedef_x_eksenleri().collect::<Vec<_>>(),
            [0, 1]
        );
        assert_eq!(yakınlaştırma.süzme_kipi, YakınlaştırmaSüzmeKipi::Boşalt);
    }
}
