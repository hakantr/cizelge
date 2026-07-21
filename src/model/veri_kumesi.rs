//! Veri kümesi — ECharts `dataset` + `encode` + `transform`
//! bileşenlerinin karşılığı: seriler, ortak bir tablodan boyut adlarıyla
//! beslenir; süzme/sıralama dönüşümleri tablo üzerinde zincirlenir.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::hata::BilesenHatasi;
use crate::model::deger::VeriDeğeri;

/// `dataset.seriesLayoutBy`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum SeriYerleşimi {
    #[default]
    Sütun,
    Satır,
}

/// `dataset.sourceHeader`; sayısal değer birden çok başlık satırı/sütunu
/// bulunan kaynakları da kapsar.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum KaynakBaşlığı {
    #[default]
    Otomatik,
    Sayı(usize),
}

/// ECharts `DimensionType` karşılığı.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BoyutTürü {
    Sayı,
    Sıralı,
    Zaman,
    Mantıksal,
    #[default]
    Bilinmeyen,
}

/// Açık veya kaynaktan çıkarılmış boyut tanımı.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct BoyutTanımı {
    pub ad: String,
    pub görünen_ad: Option<String>,
    pub tür: BoyutTürü,
}

impl BoyutTanımı {
    pub fn yeni(ad: impl Into<String>) -> Self {
        Self {
            ad: ad.into(),
            görünen_ad: None,
            tür: BoyutTürü::Bilinmeyen,
        }
    }

    pub fn tür(mut self, tür: BoyutTürü) -> Self {
        self.tür = tür;
        self
    }

    pub fn görünen_ad(mut self, ad: impl Into<String>) -> Self {
        self.görünen_ad = Some(ad.into());
        self
    }
}

impl<S: Into<String>> From<S> for BoyutTanımı {
    fn from(ad: S) -> Self {
        Self::yeni(ad)
    }
}

/// JavaScript TypedArray ailelerinin sahiplikli Rust karşılığı.
#[derive(Clone, PartialEq, Debug)]
pub enum TürlüSayıDizisi {
    F32(Vec<f32>),
    F64(Vec<f64>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    U32(Vec<u32>),
    U64(Vec<u64>),
}

impl TürlüSayıDizisi {
    pub fn len(&self) -> usize {
        match self {
            Self::F32(d) => d.len(),
            Self::F64(d) => d.len(),
            Self::I32(d) => d.len(),
            Self::I64(d) => d.len(),
            Self::U32(d) => d.len(),
            Self::U64(d) => d.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn değer(&self, sıra: usize) -> Option<VeriDeğeri> {
        match self {
            Self::F32(d) => d.get(sıra).copied().map(VeriDeğeri::from),
            Self::F64(d) => d.get(sıra).copied().map(VeriDeğeri::from),
            Self::I32(d) => d.get(sıra).copied().map(VeriDeğeri::from),
            Self::I64(d) => d.get(sıra).copied().map(VeriDeğeri::from),
            Self::U32(d) => d.get(sıra).copied().map(VeriDeğeri::from),
            Self::U64(d) => d
                .get(sıra)
                .copied()
                .map(|değer| VeriDeğeri::Sayı(değer as f64)),
        }
    }
}

/// Desteklenen bütün `dataset.source` biçimleri. Nesne satırları ve anahtarlı
/// sütunlarda `Vec<(anahtar, değer)>` kullanılması JavaScript özellik sırasını
/// belirlenimci biçimde korur.
#[derive(Clone, PartialEq, Debug)]
pub enum VeriKaynağı {
    DiziSatırlar(Vec<Vec<VeriDeğeri>>),
    NesneSatırlar(Vec<Vec<(String, VeriDeğeri)>>),
    AnahtarlıSütunlar(Vec<(String, Vec<VeriDeğeri>)>),
    TürlüDizi {
        değerler: TürlüSayıDizisi,
        boyut_sayısı: usize,
    },
}

/// Kaynağı normalleştirmek için ECharts `SourceMetaRawOption` karşılığı.
#[derive(Clone, PartialEq, Debug)]
pub struct KaynakSeçenekleri {
    pub yerleşim: SeriYerleşimi,
    pub başlık: KaynakBaşlığı,
    pub boyutlar: Vec<BoyutTanımı>,
}

impl Default for KaynakSeçenekleri {
    fn default() -> Self {
        Self {
            yerleşim: SeriYerleşimi::Sütun,
            başlık: KaynakBaşlığı::Otomatik,
            boyutlar: Vec::new(),
        }
    }
}

/// Sütunlu, seçili ham indeksleri koruyan veri deposu. Süzme yalnız indeks
/// görünümünü kopyalar; ham hücreler `Arc` ile paylaşılır.
#[derive(Clone, PartialEq, Debug)]
pub struct VeriDeposu {
    pub boyutlar: Vec<BoyutTanımı>,
    sütunlar: Arc<Vec<Vec<VeriDeğeri>>>,
    indeksler: Option<Vec<usize>>,
}

impl VeriDeposu {
    pub fn kaynaktan(
        kaynak: VeriKaynağı,
        seçenekler: KaynakSeçenekleri,
    ) -> Result<Self, BilesenHatasi> {
        let (mut boyutlar, satırlar) = kaynağı_normalleştir(kaynak, &seçenekler)?;
        if boyutlar.is_empty() {
            let sütun_sayısı = satırlar.iter().map(Vec::len).max().unwrap_or(0);
            boyutlar = (0..sütun_sayısı)
                .map(|sıra| BoyutTanımı::yeni(format!("boyut{sıra}")))
                .collect();
        }
        boyutları_doğrula(&boyutlar)?;
        let sütun_sayısı = boyutlar.len();
        let mut sütunlar = vec![Vec::with_capacity(satırlar.len()); sütun_sayısı];
        for satır in satırlar {
            for sütun in 0..sütun_sayısı {
                let değer = satır.get(sütun).cloned().unwrap_or(VeriDeğeri::Boş);
                if let Some(hedef) = sütunlar.get_mut(sütun) {
                    hedef.push(değer);
                }
            }
        }
        for (sıra, boyut) in boyutlar.iter_mut().enumerate() {
            if boyut.tür == BoyutTürü::Bilinmeyen {
                boyut.tür = sütunlar
                    .get(sıra)
                    .map(|sütun| boyut_türünü_bul(sütun))
                    .unwrap_or(BoyutTürü::Bilinmeyen);
            }
        }
        Ok(Self {
            boyutlar,
            sütunlar: Arc::new(sütunlar),
            indeksler: None,
        })
    }

    pub fn satırlardan(
        boyutlar: impl IntoIterator<Item = BoyutTanımı>,
        satırlar: Vec<Vec<VeriDeğeri>>,
    ) -> Result<Self, BilesenHatasi> {
        Self::kaynaktan(
            VeriKaynağı::DiziSatırlar(satırlar),
            KaynakSeçenekleri {
                yerleşim: SeriYerleşimi::Sütun,
                başlık: KaynakBaşlığı::Sayı(0),
                boyutlar: boyutlar.into_iter().collect(),
            },
        )
    }

    pub fn sayım(&self) -> usize {
        self.indeksler
            .as_ref()
            .map(Vec::len)
            .unwrap_or_else(|| self.ham_sayım())
    }

    pub fn ham_sayım(&self) -> usize {
        self.sütunlar.first().map(Vec::len).unwrap_or(0)
    }

    pub fn boyut_sırası(&self, seçici: &BoyutSeçici) -> Option<usize> {
        match seçici {
            BoyutSeçici::Sıra(sıra) => (*sıra < self.boyutlar.len()).then_some(*sıra),
            BoyutSeçici::Ad(ad) => self.boyutlar.iter().position(|boyut| boyut.ad == *ad),
        }
    }

    pub fn değer(&self, satır: usize, boyut: &BoyutSeçici) -> Option<&VeriDeğeri> {
        let sütun = self.boyut_sırası(boyut)?;
        let ham = self.ham_indeks(satır)?;
        self.sütunlar.get(sütun)?.get(ham)
    }

    pub fn satır(&self, sıra: usize) -> Option<Vec<&VeriDeğeri>> {
        let ham = self.ham_indeks(sıra)?;
        Some(
            self.sütunlar
                .iter()
                .filter_map(|sütun| sütun.get(ham))
                .collect(),
        )
    }

    pub fn satırları_kopyala(&self) -> Vec<Vec<VeriDeğeri>> {
        (0..self.sayım())
            .filter_map(|sıra| self.satır(sıra))
            .map(|satır| satır.into_iter().cloned().collect())
            .collect()
    }

    pub fn kapsam(&self, boyut: &BoyutSeçici) -> Result<[f64; 2], BilesenHatasi> {
        let sütun = self
            .boyut_sırası(boyut)
            .ok_or_else(|| boyut_hatası(boyut))?;
        let mut en_az = f64::INFINITY;
        let mut en_çok = f64::NEG_INFINITY;
        for sıra in 0..self.sayım() {
            let Some(ham) = self.ham_indeks(sıra) else {
                continue;
            };
            let Some(değer) = self
                .sütunlar
                .get(sütun)
                .and_then(|değerler| değerler.get(ham))
                .and_then(VeriDeğeri::sayı)
                .filter(|değer| değer.is_finite())
            else {
                continue;
            };
            en_az = en_az.min(değer);
            en_çok = en_çok.max(değer);
        }
        Ok([en_az, en_çok])
    }

    pub fn süz(&self, mut koşul: impl FnMut(usize, &[&VeriDeğeri]) -> bool) -> VeriDeposu {
        let mut indeksler = Vec::new();
        for sıra in 0..self.sayım() {
            let Some(ham) = self.ham_indeks(sıra) else {
                continue;
            };
            let satır: Vec<&VeriDeğeri> = self
                .sütunlar
                .iter()
                .filter_map(|sütun| sütun.get(ham))
                .collect();
            if koşul(sıra, &satır) {
                indeksler.push(ham);
            }
        }
        VeriDeposu {
            boyutlar: self.boyutlar.clone(),
            sütunlar: self.sütunlar.clone(),
            indeksler: Some(indeksler),
        }
    }

    pub fn aralık_seç(
        &self,
        boyut: &BoyutSeçici,
        en_az: f64,
        en_çok: f64,
    ) -> Result<VeriDeposu, BilesenHatasi> {
        let sütun = self
            .boyut_sırası(boyut)
            .ok_or_else(|| boyut_hatası(boyut))?;
        Ok(self.süz(|_, satır| {
            satır
                .get(sütun)
                .and_then(|değer| değer.sayı())
                .map(|değer| değer >= en_az && değer <= en_çok)
                // ECharts `selectRange` çizgi boşluklarını korumak için NaN'i
                // dışarı atmaz.
                .unwrap_or(true)
        }))
    }

    pub fn sırala(&self, anahtarlar: &[SıralamaAnahtarı]) -> Result<VeriDeposu, BilesenHatasi> {
        let çözülmüş: Result<Vec<_>, _> = anahtarlar
            .iter()
            .map(|anahtar| {
                self.boyut_sırası(&anahtar.boyut)
                    .map(|sıra| (sıra, anahtar.düzen))
                    .ok_or_else(|| boyut_hatası(&anahtar.boyut))
            })
            .collect();
        let çözülmüş = çözülmüş?;
        let mut indeksler: Vec<usize> = (0..self.sayım())
            .filter_map(|sıra| self.ham_indeks(sıra))
            .collect();
        indeksler.sort_by(|a, b| {
            for (sütun, düzen) in &çözülmüş {
                let av = self.sütunlar.get(*sütun).and_then(|d| d.get(*a));
                let bv = self.sütunlar.get(*sütun).and_then(|d| d.get(*b));
                let sıra = değerleri_karşılaştır(av, bv);
                if sıra != Ordering::Equal {
                    return if *düzen == SıralamaDüzeni::Artan {
                        sıra
                    } else {
                        sıra.reverse()
                    };
                }
            }
            a.cmp(b)
        });
        Ok(VeriDeposu {
            boyutlar: self.boyutlar.clone(),
            sütunlar: self.sütunlar.clone(),
            indeksler: Some(indeksler),
        })
    }

    /// Yalnız ham (süzülmemiş) depoya satır eklenebilir.
    pub fn ekle(&mut self, satırlar: Vec<Vec<VeriDeğeri>>) -> Result<[usize; 2], BilesenHatasi> {
        if self.indeksler.is_some() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "veri_deposu.ekle",
                ayrıntı: "süzülmüş görünüme appendData uygulanamaz".to_owned(),
            });
        }
        let başlangıç = self.ham_sayım();
        let sütunlar = Arc::make_mut(&mut self.sütunlar);
        for satır in satırlar {
            for sütun in 0..self.boyutlar.len() {
                let değer = satır.get(sütun).cloned().unwrap_or(VeriDeğeri::Boş);
                if let Some(hedef) = sütunlar.get_mut(sütun) {
                    hedef.push(değer);
                }
            }
        }
        Ok([başlangıç, self.ham_sayım()])
    }

    fn ham_indeks(&self, sıra: usize) -> Option<usize> {
        match &self.indeksler {
            Some(indeksler) => indeksler.get(sıra).copied(),
            None => (sıra < self.ham_sayım()).then_some(sıra),
        }
    }
}

/// Encode ve dönüşüm seçeneklerinde boyuta adla veya sırayla erişim.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum BoyutSeçici {
    Ad(String),
    Sıra(usize),
}

impl BoyutSeçici {
    pub fn ad(ad: impl Into<String>) -> Self {
        Self::Ad(ad.into())
    }
}

impl From<usize> for BoyutSeçici {
    fn from(sıra: usize) -> Self {
        Self::Sıra(sıra)
    }
}

impl From<&str> for BoyutSeçici {
    fn from(ad: &str) -> Self {
        Self::Ad(ad.to_owned())
    }
}

/// Seri türünden bağımsız `encode` yolları.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Kodlama {
    pub x: Vec<BoyutSeçici>,
    pub y: Vec<BoyutSeçici>,
    pub yarıçap: Vec<BoyutSeçici>,
    pub açı: Vec<BoyutSeçici>,
    pub öğe_adı: Vec<BoyutSeçici>,
    pub seri_adı: Vec<BoyutSeçici>,
    pub ipucu: Vec<BoyutSeçici>,
    pub etiket: Vec<BoyutSeçici>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SıralamaDüzeni {
    Artan,
    Azalan,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SıralamaAnahtarı {
    pub boyut: BoyutSeçici,
    pub düzen: SıralamaDüzeni,
}

impl SıralamaAnahtarı {
    pub fn artan(boyut: impl Into<BoyutSeçici>) -> Self {
        Self {
            boyut: boyut.into(),
            düzen: SıralamaDüzeni::Artan,
        }
    }

    pub fn azalan(boyut: impl Into<BoyutSeçici>) -> Self {
        Self {
            boyut: boyut.into(),
            düzen: SıralamaDüzeni::Azalan,
        }
    }
}

/// Sütunlu veri tablosu (`dataset.source`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct VeriKümesi {
    /// Sütun (boyut) adları.
    pub boyutlar: Vec<String>,
    /// Satırlar; her satır boyut sayısı kadar hücre içerir.
    pub satırlar: Vec<Vec<VeriDeğeri>>,
}

impl VeriKümesi {
    pub fn yeni<S: Into<String>>(boyutlar: impl IntoIterator<Item = S>) -> Self {
        VeriKümesi {
            boyutlar: boyutlar.into_iter().map(Into::into).collect(),
            satırlar: Vec::new(),
        }
    }

    /// Bütün desteklenen `dataset.source` biçimlerinden eski tablo
    /// cephesini üretir.
    pub fn kaynaktan(
        kaynak: VeriKaynağı,
        seçenekler: KaynakSeçenekleri,
    ) -> Result<Self, BilesenHatasi> {
        let depo = VeriDeposu::kaynaktan(kaynak, seçenekler)?;
        Ok(Self {
            boyutlar: depo.boyutlar.iter().map(|boyut| boyut.ad.clone()).collect(),
            satırlar: depo.satırları_kopyala(),
        })
    }

    /// Sütunlu `DataStore` görünümü.
    pub fn depoya(&self) -> Result<VeriDeposu, BilesenHatasi> {
        VeriDeposu::satırlardan(
            self.boyutlar.iter().cloned().map(BoyutTanımı::yeni),
            self.satırlar.clone(),
        )
    }

    /// Bir sütunlu depoyu, görünür satır sırasını koruyarak kök option'da
    /// taşınabilen veri kümesine dönüştürür.
    pub fn depodan(depo: &VeriDeposu) -> Self {
        Self {
            boyutlar: depo.boyutlar.iter().map(|boyut| boyut.ad.clone()).collect(),
            satırlar: depo.satırları_kopyala(),
        }
    }

    /// Aynı `dataset.source` tablosunun `seriesLayoutBy: 'row'` görünümü.
    /// Normal sütun görünümündeki ilk boyut/ilk hücreler yeni başlıkları,
    /// kalan boyutlar da yeni satırları oluşturur.
    pub fn seri_yerleşimiyle(&self, yerleşim: SeriYerleşimi) -> Self {
        if yerleşim == SeriYerleşimi::Sütun || self.boyutlar.is_empty() {
            return self.clone();
        }
        let mut boyutlar = Vec::with_capacity(self.satırlar.len() + 1);
        boyutlar.push(
            self.boyutlar
                .first()
                .cloned()
                .unwrap_or_else(|| "boyut0".to_owned()),
        );
        boyutlar.extend(
            self.satırlar
                .iter()
                .map(|satır| satır.first().and_then(değer_metin).unwrap_or_default()),
        );

        let satırlar = self
            .boyutlar
            .iter()
            .enumerate()
            .skip(1)
            .map(|(sütun, boyut_adı)| {
                let mut satır = Vec::with_capacity(self.satırlar.len() + 1);
                satır.push(VeriDeğeri::Metin(boyut_adı.clone()));
                satır.extend(
                    self.satırlar
                        .iter()
                        .map(|kaynak| kaynak.get(sütun).cloned().unwrap_or(VeriDeğeri::Boş)),
                );
                satır
            })
            .collect();
        Self {
            boyutlar, satırlar
        }
    }

    /// Satır ekler; hücre sayısı boyut sayısından azsa `Boş` ile tamamlanır.
    pub fn satır(mut self, hücreler: impl IntoIterator<Item = VeriDeğeri>) -> Self {
        let mut satır: Vec<VeriDeğeri> = hücreler.into_iter().collect();
        satır.resize(self.boyutlar.len(), VeriDeğeri::Boş);
        self.satırlar.push(satır);
        self
    }

    /// `(metin, sayı...)` biçimindeki kayıtlardan hızlı kurulum: ilk sütun
    /// metin (kategori), kalanlar sayıdır.
    pub fn kayıtlar<S: Into<String>>(
        mut self,
        kayıtlar: impl IntoIterator<Item = (S, Vec<f64>)>,
    ) -> Self {
        for (ad, sayılar) in kayıtlar {
            let mut satır: Vec<VeriDeğeri> = vec![VeriDeğeri::Metin(ad.into())];
            satır.extend(sayılar.into_iter().map(VeriDeğeri::Sayı));
            satır.resize(self.boyutlar.len(), VeriDeğeri::Boş);
            self.satırlar.push(satır);
        }
        self
    }

    /// Boyut adının sütun sırası.
    pub fn boyut_sırası(&self, ad: &str) -> Option<usize> {
        self.boyutlar.iter().position(|b| b == ad)
    }

    /// Bir boyutun hücresi.
    pub fn hücre(&self, satır: usize, boyut: &str) -> Option<&VeriDeğeri> {
        let sütun = self.boyut_sırası(boyut)?;
        self.satırlar.get(satır)?.get(sütun)
    }

    /// Boyutu sayı listesi olarak döker (sayı olmayanlar `NaN`).
    pub fn sayılar(&self, boyut: &str) -> Result<Vec<f64>, BilesenHatasi> {
        let sütun = self
            .boyut_sırası(boyut)
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "veri_kümesi.boyut",
                ayrıntı: format!("`{boyut}` boyutu yok"),
            })?;
        Ok(self
            .satırlar
            .iter()
            .map(|satır| satır.get(sütun).and_then(|h| h.sayı()).unwrap_or(f64::NAN))
            .collect())
    }

    /// Boyutu metin listesi olarak döker.
    pub fn metinler(&self, boyut: &str) -> Result<Vec<String>, BilesenHatasi> {
        let sütun = self
            .boyut_sırası(boyut)
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "veri_kümesi.boyut",
                ayrıntı: format!("`{boyut}` boyutu yok"),
            })?;
        Ok(self
            .satırlar
            .iter()
            .map(|satır| match satır.get(sütun) {
                Some(VeriDeğeri::Metin(m)) => m.clone(),
                Some(VeriDeğeri::Sayı(s)) => crate::yardimci::bicim::ondalık_kırp(*s),
                _ => String::new(),
            })
            .collect())
    }

    // ------------------------------------------------------------------
    // Dönüşümler (`transform` karşılığı) — zincirlenebilir, kaynak tabloyu
    // değiştirmez.
    // ------------------------------------------------------------------

    /// Süzme dönüşümü (`transform: filter`).
    pub fn süz(&self, koşul: impl Fn(&[VeriDeğeri]) -> bool) -> VeriKümesi {
        VeriKümesi {
            boyutlar: self.boyutlar.clone(),
            satırlar: self
                .satırlar
                .iter()
                .filter(|satır| koşul(satır))
                .cloned()
                .collect(),
        }
    }

    /// Sıralama dönüşümü (`transform: sort`): verilen boyutun sayısal
    /// değerine göre.
    pub fn sırala(&self, boyut: &str, artan: bool) -> Result<VeriKümesi, BilesenHatasi> {
        let sütun = self
            .boyut_sırası(boyut)
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "veri_kümesi.sırala",
                ayrıntı: format!("`{boyut}` boyutu yok"),
            })?;
        let mut satırlar = self.satırlar.clone();
        satırlar.sort_by(|a, b| {
            let av = a.get(sütun).and_then(|h| h.sayı()).unwrap_or(f64::NAN);
            let bv = b.get(sütun).and_then(|h| h.sayı()).unwrap_or(f64::NAN);
            let sıra = av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal);
            if artan { sıra } else { sıra.reverse() }
        });
        Ok(VeriKümesi {
            boyutlar: self.boyutlar.clone(),
            satırlar,
        })
    }
}

/// Kök `dataset: []` dizisindeki kaynak veya built-in dönüşüm girdisi.
/// `kaynak: None`, ECharts gibi `datasetIndex: 0`ı upstream kabul eder.
#[derive(Clone, PartialEq, Debug)]
pub enum VeriKümesiTanımı {
    Kaynak(VeriKümesi),
    Sırala {
        kaynak: Option<usize>,
        anahtarlar: Vec<SıralamaAnahtarı>,
    },
    Süz {
        kaynak: Option<usize>,
        koşul: SüzmeKoşulu,
    },
    Histogram {
        kaynak: Option<usize>,
        dönüşüm: HistogramDönüşümü,
    },
    Kümele {
        kaynak: Option<usize>,
        dönüşüm: KümelemeDönüşümü,
    },
    Regresyon {
        kaynak: Option<usize>,
        dönüşüm: RegresyonDönüşümü,
    },
}

impl VeriKümesiTanımı {
    pub fn kaynak(küme: VeriKümesi) -> Self {
        Self::Kaynak(küme)
    }

    pub fn sırala(anahtarlar: impl IntoIterator<Item = SıralamaAnahtarı>) -> Self {
        Self::Sırala {
            kaynak: None,
            anahtarlar: anahtarlar.into_iter().collect(),
        }
    }

    pub fn kaynaktan_sırala(
        kaynak: usize,
        anahtarlar: impl IntoIterator<Item = SıralamaAnahtarı>,
    ) -> Self {
        Self::Sırala {
            kaynak: Some(kaynak),
            anahtarlar: anahtarlar.into_iter().collect(),
        }
    }

    pub fn süz(koşul: SüzmeKoşulu) -> Self {
        Self::Süz {
            kaynak: None,
            koşul,
        }
    }

    pub fn kaynaktan_süz(kaynak: usize, koşul: SüzmeKoşulu) -> Self {
        Self::Süz {
            kaynak: Some(kaynak),
            koşul,
        }
    }

    /// `ecStat:histogram` dönüşümünü varsayılan ilk dataset'e uygular.
    pub fn histogram(dönüşüm: HistogramDönüşümü) -> Self {
        Self::Histogram {
            kaynak: None,
            dönüşüm,
        }
    }

    /// `ecStat:histogram` dönüşümünü açık upstream dataset'e uygular.
    pub fn kaynaktan_histogram(kaynak: usize, dönüşüm: HistogramDönüşümü) -> Self {
        Self::Histogram {
            kaynak: Some(kaynak),
            dönüşüm,
        }
    }

    /// `ecStat:clustering` dönüşümünü varsayılan ilk dataset'e uygular.
    pub fn kümele(dönüşüm: KümelemeDönüşümü) -> Self {
        Self::Kümele {
            kaynak: None,
            dönüşüm,
        }
    }

    /// `ecStat:clustering` dönüşümünü açık upstream dataset'e uygular.
    pub fn kaynaktan_kümele(kaynak: usize, dönüşüm: KümelemeDönüşümü) -> Self {
        Self::Kümele {
            kaynak: Some(kaynak),
            dönüşüm,
        }
    }

    /// `ecStat:regression` dönüşümünü varsayılan ilk dataset'e uygular.
    pub fn regresyon(dönüşüm: RegresyonDönüşümü) -> Self {
        Self::Regresyon {
            kaynak: None,
            dönüşüm,
        }
    }

    /// `ecStat:regression` dönüşümünü açık upstream dataset'e uygular.
    pub fn kaynaktan_regresyon(kaynak: usize, dönüşüm: RegresyonDönüşümü) -> Self {
        Self::Regresyon {
            kaynak: Some(kaynak),
            dönüşüm,
        }
    }
}

impl From<VeriKümesi> for VeriKümesiTanımı {
    fn from(küme: VeriKümesi) -> Self {
        Self::Kaynak(küme)
    }
}

/// `dataset: []` kaynak/dönüşüm dizisini sırayla yürütür. Dönüşüm girdisi
/// açık upstream vermiyorsa ECharts `queryReferringComponents` davranışıyla
/// ilk dataset'i (`fromDatasetIndex: 0`) kullanır.
pub fn veri_kümelerini_çöz(
    tanımlar: &[VeriKümesiTanımı],
) -> Result<Vec<VeriKümesi>, BilesenHatasi> {
    let mut sonuçlar: Vec<VeriKümesi> = Vec::with_capacity(tanımlar.len());
    for tanım in tanımlar {
        let sonuç = match tanım {
            VeriKümesiTanımı::Kaynak(küme) => küme.clone(),
            VeriKümesiTanımı::Sırala { kaynak, anahtarlar } => {
                let kaynak_sırası = kaynak.unwrap_or(0);
                let upstream = sonuçlar
                    .get(kaynak_sırası)
                    .ok_or(BilesenHatasi::EksikVeri {
                        bileşen: "dataset.fromDatasetIndex",
                        sıra: kaynak_sırası,
                    })?;
                let depo = upstream.depoya()?;
                VeriKümesi::depodan(&depo.sırala(anahtarlar)?)
            }
            VeriKümesiTanımı::Süz { kaynak, koşul } => {
                let kaynak_sırası = kaynak.unwrap_or(0);
                let upstream = sonuçlar
                    .get(kaynak_sırası)
                    .ok_or(BilesenHatasi::EksikVeri {
                        bileşen: "dataset.fromDatasetIndex",
                        sıra: kaynak_sırası,
                    })?;
                let depo = upstream.depoya()?;
                koşul.doğrula(&depo)?;
                VeriKümesi::depodan(&depo.süz(|satır, _| koşul.değerlendir(&depo, satır)))
            }
            VeriKümesiTanımı::Histogram {
                kaynak, dönüşüm
            } => {
                let kaynak_sırası = kaynak.unwrap_or(0);
                let upstream = sonuçlar
                    .get(kaynak_sırası)
                    .ok_or(BilesenHatasi::EksikVeri {
                        bileşen: "dataset.fromDatasetIndex",
                        sıra: kaynak_sırası,
                    })?;
                let depo = upstream.depoya()?;
                let çıktılar = dönüşüm.uygula(&[depo])?;
                let ilk = çıktılar
                    .first()
                    .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                        alan: "transform.result",
                        ayrıntı: "ecStat:histogram veri sonucu üretmedi".to_owned(),
                    })?;
                VeriKümesi::depodan(ilk)
            }
            VeriKümesiTanımı::Kümele {
                kaynak, dönüşüm
            } => {
                let kaynak_sırası = kaynak.unwrap_or(0);
                let upstream = sonuçlar
                    .get(kaynak_sırası)
                    .ok_or(BilesenHatasi::EksikVeri {
                        bileşen: "dataset.fromDatasetIndex",
                        sıra: kaynak_sırası,
                    })?;
                let depo = upstream.depoya()?;
                let çıktılar = dönüşüm.uygula(&[depo])?;
                let ilk = çıktılar
                    .first()
                    .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                        alan: "transform.result",
                        ayrıntı: "ecStat:clustering veri sonucu üretmedi".to_owned(),
                    })?;
                VeriKümesi::depodan(ilk)
            }
            VeriKümesiTanımı::Regresyon {
                kaynak, dönüşüm
            } => {
                let kaynak_sırası = kaynak.unwrap_or(0);
                let upstream = sonuçlar
                    .get(kaynak_sırası)
                    .ok_or(BilesenHatasi::EksikVeri {
                        bileşen: "dataset.fromDatasetIndex",
                        sıra: kaynak_sırası,
                    })?;
                let depo = upstream.depoya()?;
                let çıktılar = dönüşüm.uygula(&[depo])?;
                let ilk = çıktılar
                    .first()
                    .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                        alan: "transform.result",
                        ayrıntı: "ecStat:regression veri sonucu üretmedi".to_owned(),
                    })?;
                VeriKümesi::depodan(ilk)
            }
        };
        sonuçlar.push(sonuç);
    }
    Ok(sonuçlar)
}

/// Kayıtlı ya da kullanıcı tanımlı dataset dönüşümü. Bir dönüşüm birden çok
/// upstream alabilir ve birden çok sonuç üretebilir.
pub trait VeriDönüşümü: Send + Sync {
    fn tür_adı(&self) -> &str;

    fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Karşılaştırmaİşlemi {
    Eşit,
    EşitDeğil,
    Küçük,
    KüçükEşit,
    Büyük,
    BüyükEşit,
    İçerir,
}

/// Built-in filter dönüşümünün iç içe mantıksal koşulları.
#[derive(Clone, PartialEq, Debug)]
pub enum SüzmeKoşulu {
    Karşılaştır {
        boyut: BoyutSeçici,
        işlem: Karşılaştırmaİşlemi,
        değer: VeriDeğeri,
    },
    Arasında {
        boyut: BoyutSeçici,
        en_az: f64,
        en_çok: f64,
    },
    Ve(Vec<SüzmeKoşulu>),
    Veya(Vec<SüzmeKoşulu>),
    Değil(Box<SüzmeKoşulu>),
}

impl SüzmeKoşulu {
    fn doğrula(&self, depo: &VeriDeposu) -> Result<(), BilesenHatasi> {
        match self {
            Self::Karşılaştır { boyut, .. } | Self::Arasında { boyut, .. } => {
                if depo.boyut_sırası(boyut).is_none() {
                    return Err(boyut_hatası(boyut));
                }
            }
            Self::Ve(koşullar) | Self::Veya(koşullar) => {
                for koşul in koşullar {
                    koşul.doğrula(depo)?;
                }
            }
            Self::Değil(koşul) => koşul.doğrula(depo)?,
        }
        Ok(())
    }

    fn değerlendir(&self, depo: &VeriDeposu, sıra: usize) -> bool {
        match self {
            Self::Karşılaştır {
                boyut,
                işlem,
                değer,
            } => depo
                .değer(sıra, boyut)
                .map(|aday| karşılaştır(aday, değer, *işlem))
                .unwrap_or(false),
            Self::Arasında {
                boyut,
                en_az,
                en_çok,
            } => depo
                .değer(sıra, boyut)
                .and_then(VeriDeğeri::sayı)
                .map(|değer| değer >= *en_az && değer <= *en_çok)
                .unwrap_or(false),
            Self::Ve(koşullar) => koşullar.iter().all(|koşul| koşul.değerlendir(depo, sıra)),
            Self::Veya(koşullar) => koşullar.iter().any(|koşul| koşul.değerlendir(depo, sıra)),
            Self::Değil(koşul) => !koşul.değerlendir(depo, sıra),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SüzmeDönüşümü {
    pub koşul: SüzmeKoşulu,
}

impl VeriDönüşümü for SüzmeDönüşümü {
    fn tür_adı(&self) -> &str {
        "filter"
    }

    fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
        let kaynak = tek_upstream(upstream, "filter")?;
        self.koşul.doğrula(kaynak)?;
        Ok(vec![
            kaynak.süz(|sıra, _| self.koşul.değerlendir(kaynak, sıra)),
        ])
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SıralamaDönüşümü {
    pub anahtarlar: Vec<SıralamaAnahtarı>,
}

impl VeriDönüşümü for SıralamaDönüşümü {
    fn tür_adı(&self) -> &str {
        "sort"
    }

    fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
        let kaynak = tek_upstream(upstream, "sort")?;
        Ok(vec![kaynak.sırala(&self.anahtarlar)?])
    }
}

/// `echarts-stat` histogramının kutu sayısı yöntemi (`config.method`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HistogramEşikYöntemi {
    /// `ceil(sqrt(n))`; echarts-stat öntanımlısı ve en çok 50 kutu.
    #[default]
    Karekök,
    Scott,
    FreedmanDiaconis,
    Sturges,
}

/// `echarts-stat` paketinin `ecStat:histogram` dönüşümü.
///
/// İlk sonuç, resmî eklentideki beş boyutu (`MeanOfV0V1`, `VCount`,
/// `V0`, `V1`, `DisplayableName`); ikinci sonuç ise custom-series için
/// `[alt, üst, adet]` satırlarını üretir. `dimensions` birden çok boyut
/// içerirse echarts-stat gibi ilk boyut dağılıma girer, diğerleri satırın
/// sayısal geçerlilik denetimine katılır.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct HistogramDönüşümü {
    pub yöntem: HistogramEşikYöntemi,
    /// `config.dimensions`; boşsa bütün kaynak boyutları doğrulanır ve ilk
    /// boyut dağılıma girer.
    pub boyutlar: Vec<BoyutSeçici>,
}

impl HistogramDönüşümü {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn yöntem(mut self, yöntem: HistogramEşikYöntemi) -> Self {
        self.yöntem = yöntem;
        self
    }

    pub fn boyut(mut self, boyut: impl Into<BoyutSeçici>) -> Self {
        self.boyutlar = vec![boyut.into()];
        self
    }

    pub fn boyutlar(mut self, boyutlar: impl IntoIterator<Item = impl Into<BoyutSeçici>>) -> Self {
        self.boyutlar = boyutlar.into_iter().map(Into::into).collect();
        self
    }
}

fn histogram_sabit(değer: f64, basamak: usize) -> f64 {
    crate::yardimci::sayi::yuvarla(değer, basamak)
}

fn histogram_çeyreği(sıralı: &[f64], oran: f64) -> f64 {
    let uzunluk = sıralı.len();
    if uzunluk == 0 {
        return 0.0;
    }
    if oran <= 0.0 || uzunluk < 2 {
        return sıralı.first().copied().unwrap_or(0.0);
    }
    if oran >= 1.0 {
        return sıralı.last().copied().unwrap_or(0.0);
    }
    let h = (uzunluk.saturating_sub(1) as f64) * oran;
    let sıra = h.floor() as usize;
    let a = sıralı.get(sıra).copied().unwrap_or(0.0);
    let b = sıralı.get(sıra.saturating_add(1)).copied().unwrap_or(a);
    a + (b - a) * (h - sıra as f64)
}

fn histogram_kutu_sayısı(
    değerler: &[f64],
    yöntem: HistogramEşikYöntemi,
    en_az: f64,
    en_çok: f64,
) -> usize {
    let uzunluk = değerler.len();
    let ham = match yöntem {
        HistogramEşikYöntemi::Karekök => (uzunluk as f64).sqrt().ceil().min(50.0),
        HistogramEşikYöntemi::Sturges => (uzunluk as f64).log2().ceil() + 1.0,
        HistogramEşikYöntemi::Scott => {
            let ortalama = değerler.iter().sum::<f64>() / uzunluk.max(1) as f64;
            let kareler = değerler
                .iter()
                .map(|değer| (değer - ortalama).powi(2))
                .sum::<f64>();
            let sapma = if uzunluk < 2 {
                0.0
            } else {
                (kareler / uzunluk.saturating_sub(1) as f64).sqrt()
            };
            ((en_çok - en_az) / (3.5 * sapma * (uzunluk as f64).powf(-1.0 / 3.0))).ceil()
        }
        HistogramEşikYöntemi::FreedmanDiaconis => {
            let mut sıralı = değerler.to_vec();
            sıralı.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            let çeyrek_açıklığı =
                histogram_çeyreği(&sıralı, 0.75) - histogram_çeyreği(&sıralı, 0.25);
            ((en_çok - en_az) / (2.0 * çeyrek_açıklığı * (uzunluk as f64).powf(-1.0 / 3.0))).ceil()
        }
    };
    if ham.is_finite() && ham > 0.0 {
        ham as usize
    } else {
        0
    }
}

fn histogram_adım(başlangıç: f64, bitiş: f64, kutu_sayısı: usize) -> Option<(f64, usize)> {
    if kutu_sayısı == 0 {
        return None;
    }
    let ilk_adım = (bitiş - başlangıç).abs() / kutu_sayısı as f64;
    if !ilk_adım.is_finite() || ilk_adım <= 0.0 {
        return None;
    }
    let hassasiyet = crate::yardimci::sayi::nicelik_üssü(ilk_adım);
    let mut adım = 10_f64.powi(hassasiyet);
    let hata = ilk_adım / adım;
    if hata >= 50_f64.sqrt() {
        adım *= 10.0;
    } else if hata >= 10_f64.sqrt() {
        adım *= 5.0;
    } else if hata >= 2_f64.sqrt() {
        adım *= 2.0;
    }
    let sabit_hassasiyet = if hassasiyet < 0 {
        usize::try_from(-hassasiyet).unwrap_or(0)
    } else {
        0
    };
    let yönlü = if bitiş >= başlangıç {
        adım
    } else {
        -adım
    };
    let sonuç = histogram_sabit(yönlü, sabit_hassasiyet);
    (sonuç.is_finite() && sonuç != 0.0).then_some((sonuç, sabit_hassasiyet))
}

impl VeriDönüşümü for HistogramDönüşümü {
    fn tür_adı(&self) -> &str {
        "ecStat:histogram"
    }

    fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
        let kaynak = tek_upstream(upstream, self.tür_adı())?;
        if kaynak.boyutlar.is_empty() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.dimensions",
                ayrıntı: "histogram en az bir kaynak boyutu gerektirir".to_owned(),
            });
        }
        let doğrulanacak = if self.boyutlar.is_empty() {
            (0..kaynak.boyutlar.len()).collect::<Vec<_>>()
        } else {
            self.boyutlar
                .iter()
                .map(|boyut| {
                    kaynak
                        .boyut_sırası(boyut)
                        .ok_or_else(|| boyut_hatası(boyut))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        let hedef = doğrulanacak.first().copied().unwrap_or(0);
        let değerler = kaynak
            .satırları_kopyala()
            .into_iter()
            .filter_map(|satır| {
                doğrulanacak
                    .iter()
                    .all(|boyut| {
                        satır
                            .get(*boyut)
                            .and_then(VeriDeğeri::sayı)
                            .is_some_and(|değer| !değer.is_nan())
                    })
                    .then(|| satır.get(hedef).and_then(VeriDeğeri::sayı))
                    .flatten()
            })
            .filter(|değer| değer.is_finite())
            .collect::<Vec<_>>();
        let en_az = değerler.iter().copied().reduce(f64::min);
        let en_çok = değerler.iter().copied().reduce(f64::max);
        let (Some(en_az), Some(en_çok)) = (en_az, en_çok) else {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.dimensions",
                ayrıntı: "histogram seçilen boyutta sayısal veri bulamadı".to_owned(),
            });
        };
        let kutu_sayısı = histogram_kutu_sayısı(&değerler, self.yöntem, en_az, en_çok);
        let Some((adım, sabit_hassasiyet)) = histogram_adım(en_az, en_çok, kutu_sayısı) else {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.method",
                ayrıntı: "histogram kutu aralığı hesaplanamadı".to_owned(),
            });
        };
        let başlangıç = histogram_sabit((en_az / adım).ceil() * adım, sabit_hassasiyet);
        let bitiş = histogram_sabit((en_çok / adım).floor() * adım, sabit_hassasiyet);
        let aralık_sayısı = histogram_sabit((bitiş - başlangıç) / adım, sabit_hassasiyet).ceil();
        if !aralık_sayısı.is_finite() || !(0.0..=100_000.0).contains(&aralık_sayısı) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.method",
                ayrıntı: "histogram kutu sayısı güvenli sınırın dışında".to_owned(),
            });
        }
        let sınırlar = (0..=aralık_sayısı as usize)
            .map(|sıra| histogram_sabit(başlangıç + sıra as f64 * adım, sabit_hassasiyet))
            .collect::<Vec<_>>();
        let mut adetler = vec![0usize; sınırlar.len().saturating_add(1)];
        for değer in &değerler {
            let sıra = sınırlar.partition_point(|sınır| sınır <= değer);
            if let Some(adet) = adetler.get_mut(sıra) {
                *adet = adet.saturating_add(1);
            }
        }

        let mut ana_satırlar = Vec::with_capacity(adetler.len());
        let mut özel_satırlar = Vec::with_capacity(adetler.len());
        for (sıra, adet) in adetler.into_iter().enumerate() {
            let alt = if sıra > 0 {
                sınırlar.get(sıra - 1).copied().unwrap_or(en_az)
            } else {
                let ilk = sınırlar.first().copied().unwrap_or(en_az);
                if ilk - en_az == adım {
                    en_az
                } else {
                    ilk - adım
                }
            };
            let üst = if sıra < sınırlar.len() {
                sınırlar.get(sıra).copied().unwrap_or(en_çok)
            } else {
                let son = sınırlar.last().copied().unwrap_or(en_çok);
                if en_çok - son == adım {
                    en_çok
                } else {
                    son + adım
                }
            };
            let ortalama = histogram_sabit((alt + üst) / 2.0, sabit_hassasiyet);
            let ad = format!(
                "{} - {}",
                crate::yardimci::bicim::ondalık_kırp(alt),
                crate::yardimci::bicim::ondalık_kırp(üst)
            );
            ana_satırlar.push(vec![
                ortalama.into(),
                (adet as f64).into(),
                alt.into(),
                üst.into(),
                ad.into(),
            ]);
            özel_satırlar.push(vec![alt.into(), üst.into(), (adet as f64).into()]);
        }

        Ok(vec![
            VeriDeposu::satırlardan(
                [
                    BoyutTanımı::yeni("MeanOfV0V1").tür(BoyutTürü::Sayı),
                    BoyutTanımı::yeni("VCount").tür(BoyutTürü::Sayı),
                    BoyutTanımı::yeni("V0").tür(BoyutTürü::Sayı),
                    BoyutTanımı::yeni("V1").tür(BoyutTürü::Sayı),
                    BoyutTanımı::yeni("DisplayableName").tür(BoyutTürü::Sıralı),
                ],
                ana_satırlar,
            )?,
            VeriDeposu::satırlardan(
                [
                    BoyutTanımı::yeni("boyut0").tür(BoyutTürü::Sayı),
                    BoyutTanımı::yeni("boyut1").tür(BoyutTürü::Sayı),
                    BoyutTanımı::yeni("boyut2").tür(BoyutTürü::Sayı),
                ],
                özel_satırlar,
            )?,
        ])
    }
}

/// `echarts-stat` paketinin `ecStat:clustering` dönüşümü.
///
/// Gerçekleme, paketteki `hierarchicalKMeans` algoritmasını izler: bütün
/// veri önce tek küme kabul edilir, toplam karesel hatayı en çok azaltan
/// küme iki-means ile bölünür ve istenen küme sayısına kadar sürdürülür.
#[derive(Clone, PartialEq, Debug)]
pub struct KümelemeDönüşümü {
    pub küme_sayısı: usize,
    /// Hesaba katılacak mevcut boyutlar; boşsa bütün kaynak boyutları.
    pub boyutlar: Vec<BoyutSeçici>,
    /// `outputClusterIndexDimension.index`.
    pub çıktı_küme_sırası: usize,
    /// `outputClusterIndexDimension.name`.
    pub çıktı_küme_adı: Option<String>,
    /// `outputCentroidDimensions`: merkez değerlerinin yazılacağı yeni
    /// boyut sıraları.
    pub çıktı_merkez_boyutları: Vec<usize>,
    /// ECharts `Math.random` akışının SSR/kanıt karşılığı.
    pub tohum: u32,
}

impl KümelemeDönüşümü {
    pub fn yeni(küme_sayısı: usize, çıktı_küme_sırası: usize) -> Self {
        Self {
            küme_sayısı,
            boyutlar: Vec::new(),
            çıktı_küme_sırası,
            çıktı_küme_adı: None,
            çıktı_merkez_boyutları: Vec::new(),
            tohum: 0x5eed_1234,
        }
    }

    pub fn boyutlar(mut self, boyutlar: impl IntoIterator<Item = impl Into<BoyutSeçici>>) -> Self {
        self.boyutlar = boyutlar.into_iter().map(Into::into).collect();
        self
    }

    pub fn çıktı_küme_adı(mut self, ad: impl Into<String>) -> Self {
        self.çıktı_küme_adı = Some(ad.into());
        self
    }

    pub fn çıktı_merkez_boyutları(mut self, boyutlar: impl IntoIterator<Item = usize>) -> Self {
        self.çıktı_merkez_boyutları = boyutlar.into_iter().collect();
        self
    }

    pub fn tohum(mut self, tohum: u32) -> Self {
        self.tohum = tohum;
        self
    }
}

#[derive(Clone, Debug)]
struct İkiMeansSonucu {
    merkezler: Vec<Vec<f64>>,
    atamalar: Vec<(usize, f64)>,
}

#[derive(Clone, Debug)]
struct HiyerarşikKümelemeSonucu {
    merkezler: Vec<Vec<f64>>,
    atamalar: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct KümelemeRastgelesi(u32);

impl KümelemeRastgelesi {
    fn sıradaki(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = (self.0 ^ (self.0 >> 15)).wrapping_mul(1 | self.0);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }
}

fn kümeleme_merkezi(veri: &[Vec<f64>], boyut_sayısı: usize) -> Vec<f64> {
    if veri.is_empty() {
        return vec![f64::NAN; boyut_sayısı];
    }
    (0..boyut_sayısı)
        .map(|boyut| {
            veri.iter()
                .filter_map(|satır| satır.get(boyut))
                .sum::<f64>()
                / veri.len() as f64
        })
        .collect()
}

fn kümeleme_uzaklığı(veri: &[f64], merkez: &[f64], açıklıklar: &[f64]) -> f64 {
    veri.iter()
        .zip(merkez)
        .zip(açıklıklar)
        .filter(|(_, açıklık)| **açıklık != 0.0)
        .map(|((değer, merkez), açıklık)| ((değer - merkez) / açıklık).powi(2))
        .sum()
}

fn iki_means(
    veri: &[Vec<f64>],
    açıklıklar: &[f64],
    rastgele: &mut KümelemeRastgelesi,
) -> İkiMeansSonucu {
    let boyut_sayısı = açıklıklar.len();
    let mut en_azlar = vec![f64::INFINITY; boyut_sayısı];
    let mut en_çoklar = vec![f64::NEG_INFINITY; boyut_sayısı];
    for satır in veri {
        for boyut in 0..boyut_sayısı {
            let değer = satır[boyut];
            en_azlar[boyut] = en_azlar[boyut].min(değer);
            en_çoklar[boyut] = en_çoklar[boyut].max(değer);
        }
    }
    let mut merkezler = vec![vec![0.0; boyut_sayısı]; 2];
    // ecStat `createRandCent`: boyut dış döngü, merkez iç döngüdür.
    for boyut in 0..boyut_sayısı {
        let açıklık = en_çoklar[boyut] - en_azlar[boyut];
        for merkez in &mut merkezler {
            merkez[boyut] = en_azlar[boyut] + açıklık * rastgele.sıradaki();
        }
    }

    let mut atamalar = vec![(0usize, 0.0f64); veri.len()];
    let mut değişti = true;
    let mut geçiş = 0usize;
    while değişti && geçiş < 10_000 {
        değişti = false;
        for (sıra, satır) in veri.iter().enumerate() {
            let mut en_kısa = f64::INFINITY;
            let mut en_yakın = 0usize;
            for (merkez_sırası, merkez) in merkezler.iter().enumerate() {
                let uzaklık = kümeleme_uzaklığı(satır, merkez, açıklıklar);
                if uzaklık < en_kısa {
                    en_kısa = uzaklık;
                    en_yakın = merkez_sırası;
                }
            }
            if atamalar[sıra].0 != en_yakın {
                değişti = true;
            }
            atamalar[sıra] = (en_yakın, en_kısa);
        }
        for (merkez_sırası, merkez) in merkezler.iter_mut().enumerate() {
            let öğeler = veri
                .iter()
                .zip(&atamalar)
                .filter(|(_, atama)| atama.0 == merkez_sırası)
                .map(|(satır, _)| satır.clone())
                .collect::<Vec<_>>();
            *merkez = kümeleme_merkezi(&öğeler, boyut_sayısı);
        }
        geçiş += 1;
    }
    İkiMeansSonucu {
        merkezler,
        atamalar,
    }
}

fn hiyerarşik_k_means(
    veri: &[Vec<f64>],
    küme_sayısı: usize,
    tohum: u32,
) -> HiyerarşikKümelemeSonucu {
    let boyut_sayısı = veri.first().map(Vec::len).unwrap_or(0);
    let mut en_azlar = vec![f64::INFINITY; boyut_sayısı];
    let mut en_çoklar = vec![f64::NEG_INFINITY; boyut_sayısı];
    for satır in veri {
        for boyut in 0..boyut_sayısı {
            let değer = satır[boyut];
            en_azlar[boyut] = en_azlar[boyut].min(değer);
            en_çoklar[boyut] = en_çoklar[boyut].max(değer);
        }
    }
    let açıklıklar = en_azlar
        .iter()
        .zip(en_çoklar)
        .map(|(en_az, en_çok)| en_çok - en_az)
        .collect::<Vec<_>>();
    let ilk_merkez = kümeleme_merkezi(veri, boyut_sayısı);
    let mut merkezler = vec![ilk_merkez.clone()];
    let mut atamalar = vec![0usize; veri.len()];
    let mut uzaklıklar = veri
        .iter()
        .map(|satır| kümeleme_uzaklığı(satır, &ilk_merkez, &açıklıklar))
        .collect::<Vec<_>>();
    let mut rastgele = KümelemeRastgelesi(tohum);

    while merkezler.len() < küme_sayısı {
        let mut en_düşük_hata = f64::INFINITY;
        let mut seçilen: Option<(usize, İkiMeansSonucu)> = None;
        for bölünecek in 0..merkezler.len() {
            let küme_verisi = veri
                .iter()
                .zip(&atamalar)
                .filter(|(_, atama)| **atama == bölünecek)
                .map(|(satır, _)| satır.clone())
                .collect::<Vec<_>>();
            if küme_verisi.is_empty() {
                continue;
            }
            let iki = iki_means(&küme_verisi, &açıklıklar, &mut rastgele);
            let bölünmüş_hata = iki.atamalar.iter().map(|atama| atama.1).sum::<f64>();
            let bölünmeyen_hata = atamalar
                .iter()
                .zip(&uzaklıklar)
                .filter(|(atama, _)| **atama != bölünecek)
                .map(|(_, uzaklık)| *uzaklık)
                .sum::<f64>();
            let toplam = bölünmüş_hata + bölünmeyen_hata;
            if toplam < en_düşük_hata {
                en_düşük_hata = toplam;
                seçilen = Some((bölünecek, iki));
            }
        }

        let Some((bölünecek, iki)) = seçilen else {
            break;
        };
        let yeni_küme = merkezler.len();
        merkezler[bölünecek] = iki.merkezler[0].clone();
        merkezler.push(iki.merkezler[1].clone());
        let mut yerel_sıra = 0usize;
        for sıra in 0..veri.len() {
            if atamalar[sıra] != bölünecek {
                continue;
            }
            let yerel = iki.atamalar[yerel_sıra];
            atamalar[sıra] = if yerel.0 == 0 {
                bölünecek
            } else {
                yeni_küme
            };
            uzaklıklar[sıra] = yerel.1;
            yerel_sıra += 1;
        }
    }

    HiyerarşikKümelemeSonucu {
        merkezler,
        atamalar,
    }
}

impl VeriDönüşümü for KümelemeDönüşümü {
    fn tür_adı(&self) -> &str {
        "ecStat:clustering"
    }

    fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
        let kaynak = tek_upstream(upstream, self.tür_adı())?;
        if self.küme_sayısı == 0 {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.clusterCount",
                ayrıntı: "küme sayısı sıfırdan büyük olmalı".to_owned(),
            });
        }
        let boyut_sıraları = if self.boyutlar.is_empty() {
            (0..kaynak.boyutlar.len()).collect::<Vec<_>>()
        } else {
            self.boyutlar
                .iter()
                .map(|boyut| {
                    kaynak
                        .boyut_sırası(boyut)
                        .ok_or_else(|| boyut_hatası(boyut))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        if boyut_sıraları.is_empty() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.dimensions",
                ayrıntı: "kümeleme en az bir sayısal boyut gerektirir".to_owned(),
            });
        }

        // ecStat `dataPreprocess`, seçili boyutları sayısal olmayan satırları
        // sonuçtan çıkarır; kalan ham hücreler aynen korunur.
        let mut ham_satırlar = Vec::new();
        let mut sayısal_veri = Vec::new();
        for satır in kaynak.satırları_kopyala() {
            let değerler = boyut_sıraları
                .iter()
                .map(|boyut| satır.get(*boyut).and_then(VeriDeğeri::sayı))
                .collect::<Option<Vec<_>>>();
            if let Some(değerler) = değerler
                && değerler.iter().all(|değer| !değer.is_nan())
            {
                ham_satırlar.push(satır);
                sayısal_veri.push(değerler);
            }
        }
        if sayısal_veri.is_empty() || self.küme_sayısı > sayısal_veri.len() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.clusterCount",
                ayrıntı: format!(
                    "{} geçerli satır {} kümeye ayrılamaz",
                    sayısal_veri.len(),
                    self.küme_sayısı
                ),
            });
        }

        let sonuç = hiyerarşik_k_means(&sayısal_veri, self.küme_sayısı, self.tohum);
        let en_büyük_boyut = self
            .çıktı_merkez_boyutları
            .iter()
            .copied()
            .chain(std::iter::once(self.çıktı_küme_sırası))
            .max()
            .unwrap_or(self.çıktı_küme_sırası);
        let mut çıktı_boyutları = kaynak.boyutlar.clone();
        while çıktı_boyutları.len() <= en_büyük_boyut {
            let sıra = çıktı_boyutları.len();
            çıktı_boyutları.push(BoyutTanımı::yeni(format!("boyut{sıra}")).tür(BoyutTürü::Sayı));
        }
        if let Some(ad) = &self.çıktı_küme_adı {
            çıktı_boyutları[self.çıktı_küme_sırası].ad = ad.clone();
        }
        çıktı_boyutları[self.çıktı_küme_sırası].tür = BoyutTürü::Sayı;

        for (sıra, satır) in ham_satırlar.iter_mut().enumerate() {
            satır.resize(çıktı_boyutları.len(), VeriDeğeri::Boş);
            let küme = sonuç.atamalar[sıra];
            satır[self.çıktı_küme_sırası] = (küme as f64).into();
            if let Some(merkez) = sonuç.merkezler.get(küme) {
                for (merkez_boyutu, çıktı_sırası) in self.çıktı_merkez_boyutları.iter().enumerate()
                {
                    if let Some(değer) = merkez.get(merkez_boyutu) {
                        satır[*çıktı_sırası] = (*değer).into();
                    }
                }
            }
        }

        let merkez_boyutları = boyut_sıraları
            .iter()
            .filter_map(|sıra| kaynak.boyutlar.get(*sıra).cloned())
            .collect::<Vec<_>>();
        let merkez_satırları = sonuç
            .merkezler
            .iter()
            .map(|merkez| merkez.iter().copied().map(VeriDeğeri::from).collect())
            .collect();
        Ok(vec![
            VeriDeposu::satırlardan(çıktı_boyutları, ham_satırlar)?,
            VeriDeposu::satırlardan(merkez_boyutları, merkez_satırları)?,
        ])
    }
}

/// `echarts-stat` regresyon yöntemleri (`config.method`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RegresyonYöntemi {
    /// En küçük kareler doğrusu; ecStat öntanımlısı.
    #[default]
    Doğrusal,
    /// Sabit terimi sıfıra kilitli doğru (`linearThroughOrigin`).
    OrijindenDoğrusal,
    Üstel,
    Logaritmik,
    Polinom,
}

/// Regresyon formülünün üçüncü çıktı boyutuna yazılacağı satırlar
/// (`config.formulaOn`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RegresyonFormülKonumu {
    Başlangıç,
    #[default]
    Son,
    Tümü,
    Yok,
}

/// `echarts-stat` paketinin `ecStat:regression` dönüşümü.
///
/// Dönüşüm kaynak satırları değiştirmeden seçilen x/y boyutlarında tahmin
/// değerleri üretir, x'e göre sıralar ve ecStat gibi formülü üçüncü boyuta
/// yerleştirir. Bu sayede çıktı doğrudan `datasetIndex` ile çizgi serisine
/// bağlanabilir.
#[derive(Clone, PartialEq, Debug)]
pub struct RegresyonDönüşümü {
    pub yöntem: RegresyonYöntemi,
    /// Polinom derecesi (`config.order`); öntanımlı 2.
    pub derece: usize,
    /// `[x, y]` kaynak boyutları; boşsa ilk iki boyut.
    pub boyutlar: Vec<BoyutSeçici>,
    pub formül_konumu: RegresyonFormülKonumu,
}

impl RegresyonDönüşümü {
    pub fn yeni(yöntem: RegresyonYöntemi) -> Self {
        Self {
            yöntem,
            derece: 2,
            boyutlar: Vec::new(),
            formül_konumu: RegresyonFormülKonumu::Son,
        }
    }

    pub fn derece(mut self, derece: usize) -> Self {
        self.derece = derece;
        self
    }

    pub fn boyutlar(mut self, boyutlar: impl IntoIterator<Item = impl Into<BoyutSeçici>>) -> Self {
        self.boyutlar = boyutlar.into_iter().map(Into::into).collect();
        self
    }

    pub fn formül_konumu(mut self, konum: RegresyonFormülKonumu) -> Self {
        self.formül_konumu = konum;
        self
    }
}

impl Default for RegresyonDönüşümü {
    fn default() -> Self {
        Self::yeni(RegresyonYöntemi::Doğrusal)
    }
}

#[derive(Clone, Debug)]
struct RegresyonSonucu {
    satırlar: Vec<Vec<VeriDeğeri>>,
    ifade: String,
}

fn ecstat_katsayı_metni(değer: f64, basamak: usize) -> String {
    let ölçek = 10_f64.powi(basamak as i32);
    // JavaScript Math.round, negatif yarımlarda da +∞ yönüne gider.
    let yuvarlanmış = (değer.mul_add(ölçek, 0.5)).floor() / ölçek;
    if yuvarlanmış == 0.0 {
        return "0".to_owned();
    }
    let metin = format!("{yuvarlanmış:.basamak$}");
    metin.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn ecstat_gauss(mut matris: Vec<Vec<f64>>, bilinmeyen_sayısı: usize) -> Vec<f64> {
    // echarts-stat `gaussianElimination` matrisi transpoze düzende tutar:
    // son satır sağ taraf, önceki satırlar katsayı sütunlarıdır.
    let uzunluk = matris.len();
    for sıra in 0..uzunluk.saturating_sub(1) {
        let mut en_büyük = sıra;
        for aday in sıra + 1..uzunluk - 1 {
            if matris[sıra][aday].abs() > matris[sıra][en_büyük].abs() {
                en_büyük = aday;
            }
        }
        for satır in sıra..uzunluk {
            matris[satır].swap(sıra, en_büyük);
        }
        for sütun in sıra + 1..uzunluk - 1 {
            for satır in (sıra..uzunluk).rev() {
                let düşülecek = matris[satır][sıra] / matris[sıra][sıra] * matris[sıra][sütun];
                matris[satır][sütun] -= düşülecek;
            }
        }
    }

    let mut sonuç = vec![0.0; bilinmeyen_sayısı];
    let sağ_taraf = uzunluk - 1;
    for sıra in (0..uzunluk - 1).rev() {
        let toplam = (sıra + 1..uzunluk - 1)
            .map(|sütun| matris[sütun][sıra] * sonuç[sütun])
            .sum::<f64>();
        sonuç[sıra] = (matris[sağ_taraf][sıra] - toplam) / matris[sıra][sıra];
    }
    sonuç
}

fn regresyonu_hesapla(
    ham_satırlar: &[Vec<VeriDeğeri>],
    sayısal_veri: &[[f64; 2]],
    x_boyutu: usize,
    y_boyutu: usize,
    yöntem: RegresyonYöntemi,
    derece: usize,
) -> RegresyonSonucu {
    let satır_sayısı = sayısal_veri.len() as f64;
    let (tahminler, ifade) = match yöntem {
        RegresyonYöntemi::Doğrusal => {
            let x_toplamı = sayısal_veri.iter().map(|[x, _]| x).sum::<f64>();
            let y_toplamı = sayısal_veri.iter().map(|[_, y]| y).sum::<f64>();
            let xy_toplamı = sayısal_veri.iter().map(|[x, y]| x * y).sum::<f64>();
            let xx_toplamı = sayısal_veri.iter().map(|[x, _]| x * x).sum::<f64>();
            let eğim = (satır_sayısı * xy_toplamı - x_toplamı * y_toplamı)
                / (satır_sayısı * xx_toplamı - x_toplamı * x_toplamı);
            let kesişim = y_toplamı / satır_sayısı - eğim * x_toplamı / satır_sayısı;
            (
                sayısal_veri
                    .iter()
                    .map(|[x, _]| [*x, eğim.mul_add(*x, kesişim)])
                    .collect::<Vec<_>>(),
                format!(
                    "y = {}x + {}",
                    ecstat_katsayı_metni(eğim, 2),
                    ecstat_katsayı_metni(kesişim, 2)
                ),
            )
        }
        RegresyonYöntemi::OrijindenDoğrusal => {
            let xx_toplamı = sayısal_veri.iter().map(|[x, _]| x * x).sum::<f64>();
            let xy_toplamı = sayısal_veri.iter().map(|[x, y]| x * y).sum::<f64>();
            let eğim = xy_toplamı / xx_toplamı;
            (
                sayısal_veri
                    .iter()
                    .map(|[x, _]| [*x, x * eğim])
                    .collect::<Vec<_>>(),
                format!("y = {}x", ecstat_katsayı_metni(eğim, 2)),
            )
        }
        RegresyonYöntemi::Üstel => {
            let mut x_toplamı = 0.0;
            let mut y_toplamı = 0.0;
            let mut xxy_toplamı = 0.0;
            let mut xy_toplamı = 0.0;
            let mut y_lny_toplamı = 0.0;
            let mut xy_lny_toplamı = 0.0;
            for [x, y] in sayısal_veri {
                x_toplamı += x;
                y_toplamı += y;
                xy_toplamı += x * y;
                xxy_toplamı += x * x * y;
                y_lny_toplamı += y * y.ln();
                xy_lny_toplamı += x * y * y.ln();
            }
            let payda = y_toplamı * xxy_toplamı - xy_toplamı * xy_toplamı;
            let katsayı =
                ((xxy_toplamı * y_lny_toplamı - xy_toplamı * xy_lny_toplamı) / payda).exp();
            let indis = (y_toplamı * xy_lny_toplamı - xy_toplamı * y_lny_toplamı) / payda;
            (
                sayısal_veri
                    .iter()
                    .map(|[x, _]| [*x, katsayı * (indis * x).exp()])
                    .collect::<Vec<_>>(),
                format!(
                    "y = {}e^({}x)",
                    ecstat_katsayı_metni(katsayı, 2),
                    ecstat_katsayı_metni(indis, 2)
                ),
            )
        }
        RegresyonYöntemi::Logaritmik => {
            let ln_x_toplamı = sayısal_veri.iter().map(|[x, _]| x.ln()).sum::<f64>();
            let y_ln_x_toplamı = sayısal_veri.iter().map(|[x, y]| y * x.ln()).sum::<f64>();
            let y_toplamı = sayısal_veri.iter().map(|[_, y]| y).sum::<f64>();
            let ln_x_kare_toplamı = sayısal_veri
                .iter()
                .map(|[x, _]| x.ln().powi(2))
                .sum::<f64>();
            let eğim = (satır_sayısı * y_ln_x_toplamı - y_toplamı * ln_x_toplamı)
                / (satır_sayısı * ln_x_kare_toplamı - ln_x_toplamı * ln_x_toplamı);
            let kesişim = (y_toplamı - eğim * ln_x_toplamı) / satır_sayısı;
            (
                sayısal_veri
                    .iter()
                    .map(|[x, _]| [*x, eğim.mul_add(x.ln(), kesişim)])
                    .collect::<Vec<_>>(),
                format!(
                    "y = {} + {}ln(x)",
                    ecstat_katsayı_metni(kesişim, 2),
                    ecstat_katsayı_metni(eğim, 2)
                ),
            )
        }
        RegresyonYöntemi::Polinom => {
            let katsayı_sayısı = derece + 1;
            let mut matris = Vec::with_capacity(katsayı_sayısı + 1);
            let mut sağ_taraf = Vec::with_capacity(katsayı_sayısı);
            for satır in 0..katsayı_sayısı {
                sağ_taraf.push(
                    sayısal_veri
                        .iter()
                        .map(|[x, y]| y * x.powi(satır as i32))
                        .sum::<f64>(),
                );
                matris.push(
                    (0..katsayı_sayısı)
                        .map(|sütun| {
                            sayısal_veri
                                .iter()
                                .map(|[x, _]| x.powi((satır + sütun) as i32))
                                .sum::<f64>()
                        })
                        .collect::<Vec<_>>(),
                );
            }
            matris.push(sağ_taraf);
            let katsayılar = ecstat_gauss(matris, katsayı_sayısı);
            let tahminler = sayısal_veri
                .iter()
                .map(|[x, _]| {
                    let y = katsayılar
                        .iter()
                        .enumerate()
                        .map(|(üs, katsayı)| katsayı * x.powi(üs as i32))
                        .sum::<f64>();
                    [*x, y]
                })
                .collect::<Vec<_>>();
            let mut ifade = String::from("y = ");
            for üs in (0..katsayılar.len()).rev() {
                let katsayı = ecstat_katsayı_metni(katsayılar[üs], üs.saturating_add(1).max(2));
                if üs > 1 {
                    ifade.push_str(&format!("{katsayı}x^{üs} + "));
                } else if üs == 1 {
                    ifade.push_str(&format!("{katsayı}x + "));
                } else {
                    ifade.push_str(&katsayı);
                }
            }
            (tahminler, ifade)
        }
    };

    let mut satırlar = ham_satırlar
        .iter()
        .zip(tahminler)
        .map(|(ham, [x, y])| {
            let mut satır = ham.clone();
            let gerekli = x_boyutu.max(y_boyutu) + 1;
            satır.resize(gerekli, VeriDeğeri::Boş);
            satır[x_boyutu] = x.into();
            satır[y_boyutu] = y.into();
            satır
        })
        .collect::<Vec<_>>();
    satırlar.sort_by(|a, b| {
        let a = a.get(x_boyutu).and_then(VeriDeğeri::sayı);
        let b = b.get(x_boyutu).and_then(VeriDeğeri::sayı);
        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
    });
    RegresyonSonucu { satırlar, ifade }
}

impl VeriDönüşümü for RegresyonDönüşümü {
    fn tür_adı(&self) -> &str {
        "ecStat:regression"
    }

    fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
        let kaynak = tek_upstream(upstream, self.tür_adı())?;
        let boyutlar = if self.boyutlar.is_empty() {
            vec![BoyutSeçici::Sıra(0), BoyutSeçici::Sıra(1)]
        } else {
            self.boyutlar.clone()
        };
        if boyutlar.len() != 2 {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.dimensions",
                ayrıntı: "regresyon tam olarak x ve y olmak üzere iki boyut gerektirir"
                    .to_owned(),
            });
        }
        if self.yöntem == RegresyonYöntemi::Polinom && self.derece == 0 {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.order",
                ayrıntı: "polinom derecesi sıfırdan büyük olmalı".to_owned(),
            });
        }
        let x_boyutu = kaynak
            .boyut_sırası(&boyutlar[0])
            .ok_or_else(|| boyut_hatası(&boyutlar[0]))?;
        let y_boyutu = kaynak
            .boyut_sırası(&boyutlar[1])
            .ok_or_else(|| boyut_hatası(&boyutlar[1]))?;

        // ecStat `dataPreprocess`, seçili iki boyutu sayı olmayan satırları
        // dönüşümden çıkarır ve kalan kaynak hücrelerini aynen korur.
        let mut ham_satırlar = Vec::new();
        let mut sayısal_veri = Vec::new();
        for satır in kaynak.satırları_kopyala() {
            let x = satır.get(x_boyutu).and_then(VeriDeğeri::sayı);
            let y = satır.get(y_boyutu).and_then(VeriDeğeri::sayı);
            if let (Some(x), Some(y)) = (x, y)
                && !x.is_nan()
                && !y.is_nan()
            {
                ham_satırlar.push(satır);
                sayısal_veri.push([x, y]);
            }
        }
        if sayısal_veri.is_empty() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.dimensions",
                ayrıntı: "regresyon için geçerli sayısal satır yok".to_owned(),
            });
        }
        if self.yöntem == RegresyonYöntemi::Polinom && self.derece >= sayısal_veri.len() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.order",
                ayrıntı: format!(
                    "{} nokta {}. derece polinomu belirleyemez",
                    sayısal_veri.len(),
                    self.derece
                ),
            });
        }

        let mut sonuç = regresyonu_hesapla(
            &ham_satırlar,
            &sayısal_veri,
            x_boyutu,
            y_boyutu,
            self.yöntem,
            self.derece,
        );
        let mut çıktı_boyutları = kaynak.boyutlar.clone();
        if self.formül_konumu != RegresyonFormülKonumu::Yok {
            while çıktı_boyutları.len() <= 2 {
                let sıra = çıktı_boyutları.len();
                çıktı_boyutları
                    .push(BoyutTanımı::yeni(format!("boyut{sıra}")).tür(BoyutTürü::Bilinmeyen));
            }
            // ecStat dönüşüm sarmalayıcısı üçüncü dimensionInfo'yu boş bir
            // tanımla değiştirir; adlandırılmış Rust depoda kararlı karşılığı.
            çıktı_boyutları[2] = BoyutTanımı::yeni("boyut2");
            let son = sonuç.satırlar.len().saturating_sub(1);
            for (sıra, satır) in sonuç.satırlar.iter_mut().enumerate() {
                satır.resize(çıktı_boyutları.len(), VeriDeğeri::Boş);
                let göster = match self.formül_konumu {
                    RegresyonFormülKonumu::Başlangıç => sıra == 0,
                    RegresyonFormülKonumu::Son => sıra == son,
                    RegresyonFormülKonumu::Tümü => true,
                    RegresyonFormülKonumu::Yok => false,
                };
                satır[2] = if göster {
                    sonuç.ifade.clone().into()
                } else {
                    String::new().into()
                };
            }
        }
        Ok(vec![VeriDeposu::satırlardan(
            çıktı_boyutları,
            sonuç.satırlar,
        )?])
    }
}

/// Built-in `boxplot` dönüşümünün bıyık sınırı (`boundIQR`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KutuSınırı {
    /// `Q1/Q3 ± katsayı * IQR`; ECharts öntanımlısı `1.5`.
    ÇeyreklerArası(f64),
    /// `boundIQR: 'none' | 0`: gözlenen en küçük/en büyük değer.
    UçDeğerler,
}

impl Default for KutuSınırı {
    fn default() -> Self {
        Self::ÇeyreklerArası(1.5)
    }
}

/// ECharts built-in `boxplot` dataset dönüşümü. İlk sonuç kutu özetlerini,
/// ikinci sonuç aykırı değerleri üretir.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct KutuDönüşümü {
    pub sınır: KutuSınırı,
    pub öğe_adı_biçimi: Option<String>,
}

impl KutuDönüşümü {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn çeyrekler_arası_sınır(mut self, katsayı: f64) -> Self {
        self.sınır = if katsayı.is_finite() && katsayı > 0.0 {
            KutuSınırı::ÇeyreklerArası(katsayı)
        } else {
            KutuSınırı::UçDeğerler
        };
        self
    }

    pub fn uç_değerleri_kullan(mut self) -> Self {
        self.sınır = KutuSınırı::UçDeğerler;
        self
    }

    pub fn öğe_adı_biçimi(mut self, biçim: impl Into<String>) -> Self {
        self.öğe_adı_biçimi = Some(biçim.into());
        self
    }
}

impl VeriDönüşümü for KutuDönüşümü {
    fn tür_adı(&self) -> &str {
        "boxplot"
    }

    fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
        let kaynak = tek_upstream(upstream, self.tür_adı())?;
        let mut kutular = Vec::with_capacity(kaynak.sayım());
        let mut aykırılar = Vec::new();

        for sıra in 0..kaynak.sayım() {
            let mut değerler: Vec<f64> = kaynak
                .satır(sıra)
                .unwrap_or_default()
                .into_iter()
                .filter_map(VeriDeğeri::sayı)
                .filter(|değer| değer.is_finite())
                .collect();
            değerler.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            let ad = self
                .öğe_adı_biçimi
                .as_deref()
                .map(|biçim| biçim.replace("{value}", &sıra.to_string()))
                .unwrap_or_else(|| sıra.to_string());
            let (Some(en_az), Some(ç1), Some(ortanca), Some(ç3), Some(en_çok)) = (
                değerler.first().copied(),
                quantile_type7(değerler.clone(), 0.25),
                quantile_type7(değerler.clone(), 0.5),
                quantile_type7(değerler.clone(), 0.75),
                değerler.last().copied(),
            ) else {
                kutular.push(vec![
                    ad.into(),
                    VeriDeğeri::Boş,
                    VeriDeğeri::Boş,
                    VeriDeğeri::Boş,
                    VeriDeğeri::Boş,
                    VeriDeğeri::Boş,
                ]);
                continue;
            };
            let (düşük, yüksek) = match self.sınır {
                KutuSınırı::UçDeğerler => (en_az, en_çok),
                KutuSınırı::ÇeyreklerArası(katsayı) => {
                    let sınır = katsayı * (ç3 - ç1);
                    (en_az.max(ç1 - sınır), en_çok.min(ç3 + sınır))
                }
            };
            kutular.push(vec![
                ad.clone().into(),
                düşük.into(),
                ç1.into(),
                ortanca.into(),
                ç3.into(),
                yüksek.into(),
            ]);
            aykırılar.extend(
                değerler
                    .into_iter()
                    .filter(|değer| *değer < düşük || *değer > yüksek)
                    .map(|değer| vec![ad.clone().into(), değer.into()]),
            );
        }

        Ok(vec![
            VeriDeposu::satırlardan(
                ["ItemName", "Low", "Q1", "Q2", "Q3", "High"]
                    .into_iter()
                    .map(BoyutTanımı::yeni),
                kutular,
            )?,
            VeriDeposu::satırlardan(
                ["ItemName", "Outlier"].into_iter().map(BoyutTanımı::yeni),
                aykırılar,
            )?,
        ])
    }
}

/// Gruplu toplama sonucunda bir boyutun nasıl üretileceği. Çeyrekler,
/// `echarts-simple-transform` gibi doğrusal enterpolasyonlu Type-7
/// quantile tanımını kullanır.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToplamaYöntemi {
    EnAz,
    Çeyrek1,
    Ortanca,
    Çeyrek3,
    EnÇok,
    /// Grubun ilk satırındaki ham değer (genellikle grup anahtarı).
    İlk,
}

/// `ecSimpleTransform:aggregate.config.resultDimensions` öğesi.
#[derive(Clone, PartialEq, Debug)]
pub struct ToplamaBoyutu {
    pub ad: String,
    pub kaynak: BoyutSeçici,
    pub yöntem: ToplamaYöntemi,
}

impl ToplamaBoyutu {
    pub fn yeni(
        ad: impl Into<String>,
        kaynak: impl Into<BoyutSeçici>,
        yöntem: ToplamaYöntemi,
    ) -> Self {
        Self {
            ad: ad.into(),
            kaynak: kaynak.into(),
            yöntem,
        }
    }

    pub fn en_az(ad: impl Into<String>, kaynak: impl Into<BoyutSeçici>) -> Self {
        Self::yeni(ad, kaynak, ToplamaYöntemi::EnAz)
    }

    pub fn çeyrek1(ad: impl Into<String>, kaynak: impl Into<BoyutSeçici>) -> Self {
        Self::yeni(ad, kaynak, ToplamaYöntemi::Çeyrek1)
    }

    pub fn ortanca(ad: impl Into<String>, kaynak: impl Into<BoyutSeçici>) -> Self {
        Self::yeni(ad, kaynak, ToplamaYöntemi::Ortanca)
    }

    pub fn çeyrek3(ad: impl Into<String>, kaynak: impl Into<BoyutSeçici>) -> Self {
        Self::yeni(ad, kaynak, ToplamaYöntemi::Çeyrek3)
    }

    pub fn en_çok(ad: impl Into<String>, kaynak: impl Into<BoyutSeçici>) -> Self {
        Self::yeni(ad, kaynak, ToplamaYöntemi::EnÇok)
    }

    pub fn ilk(ad: impl Into<String>, kaynak: impl Into<BoyutSeçici>) -> Self {
        Self::yeni(ad, kaynak, ToplamaYöntemi::İlk)
    }
}

/// `echarts-simple-transform` paketinin gruplu aggregate dönüşümünün yerli
/// karşılığı. Kayıt defterine `ecSimpleTransform:aggregate` adıyla
/// kaydedilebilir.
#[derive(Clone, PartialEq, Debug)]
pub struct ToplamaDönüşümü {
    pub grupla: BoyutSeçici,
    pub sonuç_boyutları: Vec<ToplamaBoyutu>,
}

impl ToplamaDönüşümü {
    pub fn yeni(
        grupla: impl Into<BoyutSeçici>,
        sonuç_boyutları: impl IntoIterator<Item = ToplamaBoyutu>,
    ) -> Self {
        Self {
            grupla: grupla.into(),
            sonuç_boyutları: sonuç_boyutları.into_iter().collect(),
        }
    }
}

fn quantile_type7(mut değerler: Vec<f64>, oran: f64) -> Option<f64> {
    değerler.retain(|değer| değer.is_finite());
    değerler.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let ilk = *değerler.first()?;
    if değerler.len() == 1 {
        return Some(ilk);
    }
    let konum = (değerler.len() - 1) as f64 * oran.clamp(0.0, 1.0);
    let alt = konum.floor() as usize;
    let üst = konum.ceil() as usize;
    let a = *değerler.get(alt)?;
    let b = *değerler.get(üst)?;
    Some(a + (b - a) * (konum - alt as f64))
}

impl VeriDönüşümü for ToplamaDönüşümü {
    fn tür_adı(&self) -> &str {
        "ecSimpleTransform:aggregate"
    }

    fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
        let kaynak = tek_upstream(upstream, self.tür_adı())?;
        let grup_sırası = kaynak
            .boyut_sırası(&self.grupla)
            .ok_or_else(|| boyut_hatası(&self.grupla))?;
        if self.sonuç_boyutları.is_empty() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.config.resultDimensions",
                ayrıntı: "aggregate en az bir sonuç boyutu gerektirir".to_owned(),
            });
        }
        let çözülmüş: Result<Vec<_>, _> = self
            .sonuç_boyutları
            .iter()
            .map(|boyut| {
                kaynak
                    .boyut_sırası(&boyut.kaynak)
                    .map(|sıra| (boyut, sıra))
                    .ok_or_else(|| boyut_hatası(&boyut.kaynak))
            })
            .collect();
        let çözülmüş = çözülmüş?;

        // JavaScript Map ekleme sırasını korur; grup çıktısı da kaynakta ilk
        // görülme sırasındadır.
        let mut gruplar: Vec<(VeriDeğeri, Vec<usize>)> = Vec::new();
        for sıra in 0..kaynak.sayım() {
            let anahtar = kaynak
                .değer(sıra, &BoyutSeçici::Sıra(grup_sırası))
                .cloned()
                .unwrap_or(VeriDeğeri::Boş);
            if let Some((_, sıralar)) = gruplar.iter_mut().find(|(aday, _)| aday == &anahtar) {
                sıralar.push(sıra);
            } else {
                gruplar.push((anahtar, vec![sıra]));
            }
        }

        let mut satırlar = Vec::with_capacity(gruplar.len());
        for (_, grup_satırları) in gruplar {
            let mut satır = Vec::with_capacity(çözülmüş.len());
            for (boyut, kaynak_sırası) in &çözülmüş {
                let ilk_değer = grup_satırları
                    .first()
                    .and_then(|sıra| kaynak.değer(*sıra, &BoyutSeçici::Sıra(*kaynak_sırası)))
                    .cloned()
                    .unwrap_or(VeriDeğeri::Boş);
                let değer = match boyut.yöntem {
                    ToplamaYöntemi::İlk => ilk_değer,
                    yöntem => {
                        let sayılar: Vec<f64> = grup_satırları
                            .iter()
                            .filter_map(|sıra| {
                                kaynak.değer(*sıra, &BoyutSeçici::Sıra(*kaynak_sırası))
                            })
                            .filter_map(VeriDeğeri::sayı)
                            .collect();
                        let sonuç = match yöntem {
                            ToplamaYöntemi::EnAz => sayılar.iter().copied().reduce(f64::min),
                            ToplamaYöntemi::Çeyrek1 => quantile_type7(sayılar, 0.25),
                            ToplamaYöntemi::Ortanca => quantile_type7(sayılar, 0.5),
                            ToplamaYöntemi::Çeyrek3 => quantile_type7(sayılar, 0.75),
                            ToplamaYöntemi::EnÇok => sayılar.iter().copied().reduce(f64::max),
                            ToplamaYöntemi::İlk => None,
                        };
                        sonuç.map(VeriDeğeri::Sayı).unwrap_or(VeriDeğeri::Boş)
                    }
                };
                satır.push(değer);
            }
            satırlar.push(satır);
        }

        let boyutlar = self
            .sonuç_boyutları
            .iter()
            .map(|boyut| BoyutTanımı::yeni(boyut.ad.clone()));
        Ok(vec![VeriDeposu::satırlardan(boyutlar, satırlar)?])
    }
}

/// Kullanıcı dönüşümlerinin adla kaydı. Kayıt bir örneğe değil kitaplığa
/// aittir; trait nesnesi `Send + Sync` olduğundan başsız/progressive
/// koşucularda da güvenle paylaşılır.
#[derive(Default)]
pub struct DönüşümKayıtDefteri {
    dönüşümler: BTreeMap<String, Arc<dyn VeriDönüşümü>>,
}

impl DönüşümKayıtDefteri {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn kaydet(
        &mut self, dönüşüm: impl VeriDönüşümü + 'static
    ) -> Result<(), BilesenHatasi> {
        let ad = dönüşüm.tür_adı().trim().to_owned();
        if ad.is_empty() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.type",
                ayrıntı: "dönüşüm türü boş olamaz".to_owned(),
            });
        }
        if self.dönüşümler.contains_key(&ad) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.type",
                ayrıntı: format!("`{ad}` dönüşümü zaten kayıtlı"),
            });
        }
        self.dönüşümler.insert(ad, Arc::new(dönüşüm));
        Ok(())
    }

    pub fn çalıştır(
        &self,
        tür: &str,
        upstream: &[VeriDeposu],
    ) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
        let dönüşüm = self
            .dönüşümler
            .get(tür)
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "transform.type",
                ayrıntı: format!("`{tür}` dönüşümü kayıtlı değil"),
            })?;
        dönüşüm.uygula(upstream)
    }
}

/// `dataset.fromDatasetId` ve çok sonuçlu transform zincirlerinin
/// belirlenimci, yürütülmüş görünümü.
#[derive(Clone, Default)]
pub struct VeriKümesiZinciri {
    sonuçlar: BTreeMap<String, Vec<VeriDeposu>>,
}

impl VeriKümesiZinciri {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn kaynak_ekle(
        &mut self,
        kimlik: impl Into<String>,
        kaynak: VeriKaynağı,
        seçenekler: KaynakSeçenekleri,
    ) -> Result<(), BilesenHatasi> {
        let kimlik = kimlik.into();
        self.kimliği_doğrula(&kimlik)?;
        self.sonuçlar
            .insert(kimlik, vec![VeriDeposu::kaynaktan(kaynak, seçenekler)?]);
        Ok(())
    }

    pub fn dönüştür(
        &mut self,
        kimlik: impl Into<String>,
        upstream: impl IntoIterator<Item = (String, usize)>,
        dönüşüm: &dyn VeriDönüşümü,
    ) -> Result<(), BilesenHatasi> {
        let kimlik = kimlik.into();
        self.kimliği_doğrula(&kimlik)?;
        let girdiler: Result<Vec<_>, _> = upstream
            .into_iter()
            .map(|(kaynak, sonuç_sırası)| {
                self.al(&kaynak, sonuç_sırası).cloned().ok_or_else(|| {
                    BilesenHatasi::GeçersizSeçenek {
                        alan: "dataset.fromDatasetId",
                        ayrıntı: format!("`{kaynak}` kümesinin {sonuç_sırası}. sonucu yok"),
                    }
                })
            })
            .collect();
        let sonuç = dönüşüm.uygula(&girdiler?)?;
        if sonuç.is_empty() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "transform.result",
                ayrıntı: "dönüşüm en az bir veri sonucu üretmeli".to_owned(),
            });
        }
        self.sonuçlar.insert(kimlik, sonuç);
        Ok(())
    }

    pub fn al(&self, kimlik: &str, sonuç_sırası: usize) -> Option<&VeriDeposu> {
        self.sonuçlar.get(kimlik)?.get(sonuç_sırası)
    }

    fn kimliği_doğrula(&self, kimlik: &str) -> Result<(), BilesenHatasi> {
        if kimlik.is_empty() || self.sonuçlar.contains_key(kimlik) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "dataset.id",
                ayrıntı: if kimlik.is_empty() {
                    "dataset kimliği boş olamaz".to_owned()
                } else {
                    format!("`{kimlik}` dataset kimliği yinelendi")
                },
            });
        }
        Ok(())
    }
}

fn kaynağı_normalleştir(
    kaynak: VeriKaynağı,
    seçenekler: &KaynakSeçenekleri,
) -> Result<(Vec<BoyutTanımı>, Vec<Vec<VeriDeğeri>>), BilesenHatasi> {
    match kaynak {
        VeriKaynağı::DiziSatırlar(dizi) => {
            let başlık = match seçenekler.başlık {
                KaynakBaşlığı::Otomatik => {
                    otomatik_başlık_sayısı(&dizi, seçenekler.yerleşim)
                }
                KaynakBaşlığı::Sayı(sayı) => sayı,
            };
            let mut boyutlar = seçenekler.boyutlar.clone();
            let satırlar = match seçenekler.yerleşim {
                SeriYerleşimi::Sütun => {
                    if boyutlar.is_empty() && başlık == 1 {
                        boyutlar = dizi
                            .first()
                            .map(|satır| satır.iter().map(boyut_adı).collect())
                            .unwrap_or_default();
                    }
                    dizi.into_iter().skip(başlık).collect()
                }
                SeriYerleşimi::Satır => {
                    if boyutlar.is_empty() && başlık == 1 {
                        boyutlar = dizi
                            .iter()
                            .map(|satır| {
                                satır
                                    .first()
                                    .map(boyut_adı)
                                    .unwrap_or_else(|| BoyutTanımı::yeni(""))
                            })
                            .collect();
                    }
                    let sütun_sayısı = dizi.iter().map(Vec::len).max().unwrap_or(0);
                    (başlık..sütun_sayısı)
                        .map(|sütun| {
                            dizi.iter()
                                .map(|satır| satır.get(sütun).cloned().unwrap_or(VeriDeğeri::Boş))
                                .collect()
                        })
                        .collect()
                }
            };
            Ok((boyutlar, satırlar))
        }
        VeriKaynağı::NesneSatırlar(nesneler) => {
            let mut boyutlar = seçenekler.boyutlar.clone();
            if boyutlar.is_empty() {
                boyutlar = nesneler
                    .iter()
                    .find(|satır| !satır.is_empty())
                    .map(|satır| {
                        satır
                            .iter()
                            .map(|(ad, _)| BoyutTanımı::yeni(ad.clone()))
                            .collect()
                    })
                    .unwrap_or_default();
            }
            let satırlar = nesneler
                .into_iter()
                .map(|nesne| {
                    boyutlar
                        .iter()
                        .map(|boyut| {
                            nesne
                                .iter()
                                .find(|(ad, _)| ad == &boyut.ad)
                                .map(|(_, değer)| değer.clone())
                                .unwrap_or(VeriDeğeri::Boş)
                        })
                        .collect()
                })
                .collect();
            Ok((boyutlar, satırlar))
        }
        VeriKaynağı::AnahtarlıSütunlar(sütunlar) => {
            let boyutlar = if seçenekler.boyutlar.is_empty() {
                sütunlar
                    .iter()
                    .map(|(ad, _)| BoyutTanımı::yeni(ad.clone()))
                    .collect()
            } else {
                seçenekler.boyutlar.clone()
            };
            let satır_sayısı = sütunlar.iter().map(|(_, d)| d.len()).max().unwrap_or(0);
            let satırlar = (0..satır_sayısı)
                .map(|satır| {
                    boyutlar
                        .iter()
                        .map(|boyut| {
                            sütunlar
                                .iter()
                                .find(|(ad, _)| ad == &boyut.ad)
                                .and_then(|(_, değerler)| değerler.get(satır))
                                .cloned()
                                .unwrap_or(VeriDeğeri::Boş)
                        })
                        .collect()
                })
                .collect();
            Ok((boyutlar, satırlar))
        }
        VeriKaynağı::TürlüDizi {
            değerler,
            boyut_sayısı,
        } => {
            if boyut_sayısı == 0
                || seçenekler.boyutlar.len() != boyut_sayısı
                || değerler.len() % boyut_sayısı != 0
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "dataset.typedArray",
                    ayrıntı: format!(
                        "typed array için {boyut_sayısı} açık boyut ve tam satırlar gerekli ({} değer, {} tanım)",
                        değerler.len(),
                        seçenekler.boyutlar.len()
                    ),
                });
            }
            let satır_sayısı = değerler.len() / boyut_sayısı;
            let satırlar = (0..satır_sayısı)
                .map(|satır| {
                    (0..boyut_sayısı)
                        .map(|sütun| {
                            değerler
                                .değer(satır * boyut_sayısı + sütun)
                                .unwrap_or(VeriDeğeri::Boş)
                        })
                        .collect()
                })
                .collect();
            Ok((seçenekler.boyutlar.clone(), satırlar))
        }
    }
}

fn otomatik_başlık_sayısı(dizi: &[Vec<VeriDeğeri>], yerleşim: SeriYerleşimi) -> usize {
    let değerler: Vec<_> = match yerleşim {
        SeriYerleşimi::Sütun => dizi
            .first()
            .map(|satır| satır.iter().take(10).collect())
            .unwrap_or_default(),
        SeriYerleşimi::Satır => dizi
            .iter()
            .take(10)
            .filter_map(|satır| satır.first())
            .collect(),
    };
    let mut sonuç = None;
    for değer in değerler {
        match değer {
            VeriDeğeri::Boş => {}
            VeriDeğeri::Metin(metin) if metin == "-" => {}
            VeriDeğeri::Metin(_) if sonuç.is_none() => sonuç = Some(1),
            VeriDeğeri::Metin(_) => {}
            _ => sonuç = Some(0),
        }
    }
    sonuç.unwrap_or(0)
}

fn boyut_adı(değer: &VeriDeğeri) -> BoyutTanımı {
    BoyutTanımı::yeni(değer_metin(değer).unwrap_or_default())
}

fn boyutları_doğrula(boyutlar: &[BoyutTanımı]) -> Result<(), BilesenHatasi> {
    let mut adlar = std::collections::HashSet::new();
    for boyut in boyutlar {
        if !boyut.ad.is_empty() && !adlar.insert(&boyut.ad) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "dataset.dimensions",
                ayrıntı: format!("`{}` boyut adı yinelendi", boyut.ad),
            });
        }
    }
    Ok(())
}

fn boyut_türünü_bul(sütun: &[VeriDeğeri]) -> BoyutTürü {
    let mut tür = BoyutTürü::Bilinmeyen;
    for değer in sütun.iter().filter(|değer| !değer.boş_mu()) {
        let aday = match değer {
            VeriDeğeri::Sayı(_) | VeriDeğeri::Çift(_) | VeriDeğeri::Dizi(_) => {
                BoyutTürü::Sayı
            }
            VeriDeğeri::Metin(_) => BoyutTürü::Sıralı,
            VeriDeğeri::Mantıksal(_) => BoyutTürü::Mantıksal,
            VeriDeğeri::Zaman(_) => BoyutTürü::Zaman,
            VeriDeğeri::Boş => continue,
        };
        if tür == BoyutTürü::Bilinmeyen {
            tür = aday;
        } else if tür != aday {
            return if aday == BoyutTürü::Sıralı || tür == BoyutTürü::Sıralı {
                BoyutTürü::Sıralı
            } else {
                BoyutTürü::Bilinmeyen
            };
        }
    }
    tür
}

fn boyut_hatası(boyut: &BoyutSeçici) -> BilesenHatasi {
    BilesenHatasi::GeçersizSeçenek {
        alan: "dataset.dimension",
        ayrıntı: format!("{boyut:?} boyutu yok"),
    }
}

fn tek_upstream<'a>(
    upstream: &'a [VeriDeposu],
    tür: &str,
) -> Result<&'a VeriDeposu, BilesenHatasi> {
    if upstream.len() != 1 {
        return Err(BilesenHatasi::GeçersizSeçenek {
            alan: "transform.upstream",
            ayrıntı: format!(
                "`{tür}` tam bir upstream bekler, {} verildi",
                upstream.len()
            ),
        });
    }
    upstream.first().ok_or(BilesenHatasi::EksikVeri {
        bileşen: "transform.upstream",
        sıra: 0,
    })
}

fn karşılaştır(
    sol: &VeriDeğeri, sağ: &VeriDeğeri, işlem: Karşılaştırmaİşlemi
) -> bool {
    if işlem == Karşılaştırmaİşlemi::İçerir {
        return değer_metin(sol)
            .zip(değer_metin(sağ))
            .map(|(sol, sağ)| sol.contains(&sağ))
            .unwrap_or(false);
    }
    let sıra = değerleri_karşılaştır(Some(sol), Some(sağ));
    match işlem {
        Karşılaştırmaİşlemi::Eşit => sıra == Ordering::Equal,
        Karşılaştırmaİşlemi::EşitDeğil => sıra != Ordering::Equal,
        Karşılaştırmaİşlemi::Küçük => sıra == Ordering::Less,
        Karşılaştırmaİşlemi::KüçükEşit => sıra != Ordering::Greater,
        Karşılaştırmaİşlemi::Büyük => sıra == Ordering::Greater,
        Karşılaştırmaİşlemi::BüyükEşit => sıra != Ordering::Less,
        Karşılaştırmaİşlemi::İçerir => false,
    }
}

fn değerleri_karşılaştır(sol: Option<&VeriDeğeri>, sağ: Option<&VeriDeğeri>) -> Ordering {
    match (sol, sağ) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(sol), Some(sağ)) if sol.boş_mu() && sağ.boş_mu() => Ordering::Equal,
        (Some(sol), Some(_)) if sol.boş_mu() => Ordering::Greater,
        (Some(_), Some(sağ)) if sağ.boş_mu() => Ordering::Less,
        (Some(sol), Some(sağ)) => match (sol.sayı(), sağ.sayı()) {
            (Some(sol), Some(sağ)) if sol.is_finite() && sağ.is_finite() => sol.total_cmp(&sağ),
            _ => değer_metin(sol)
                .unwrap_or_default()
                .cmp(&değer_metin(sağ).unwrap_or_default()),
        },
    }
}

fn değer_metin(değer: &VeriDeğeri) -> Option<String> {
    match değer {
        VeriDeğeri::Boş => None,
        VeriDeğeri::Sayı(sayı) => Some(crate::yardimci::bicim::ondalık_kırp(*sayı)),
        VeriDeğeri::Çift([x, y]) => Some(format!("{x},{y}")),
        VeriDeğeri::Dizi(dizi) => Some(
            dizi.iter()
                .map(|sayı| crate::yardimci::bicim::ondalık_kırp(*sayı))
                .collect::<Vec<_>>()
                .join(","),
        ),
        VeriDeğeri::Metin(metin) => Some(metin.clone()),
        VeriDeğeri::Mantıksal(değer) => Some(değer.to_string()),
        VeriDeğeri::Zaman(ms) => Some(ms.to_string()),
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod testler {
    use super::*;

    fn örnek() -> VeriKümesi {
        VeriKümesi::yeni(["ürün", "satış", "kâr"]).kayıtlar([
            ("Elma", vec![120.0, 30.0]),
            ("Armut", vec![80.0, 22.0]),
            ("Kiraz", vec![160.0, 41.0]),
        ])
    }

    #[test]
    fn boyut_erişimi() {
        let küme = örnek();
        assert_eq!(küme.sayılar("satış").unwrap(), vec![120.0, 80.0, 160.0]);
        assert_eq!(
            küme.metinler("ürün").unwrap(),
            vec!["Elma", "Armut", "Kiraz"]
        );
        assert!(küme.sayılar("yok").is_err());
    }

    #[test]
    fn dönüşümler() {
        let küme = örnek();
        let sıralı = küme.sırala("satış", false).unwrap();
        assert_eq!(
            sıralı.metinler("ürün").unwrap(),
            vec!["Kiraz", "Elma", "Armut"]
        );
        let süzülü = küme.süz(|satır| satır.get(1).and_then(|h| h.sayı()).unwrap_or(0.0) > 100.0);
        assert_eq!(süzülü.satırlar.len(), 2);
    }

    #[test]
    fn çoklu_dataset_varsayılan_sıfırıncı_ve_açık_upstreami_çözer() {
        let kaynak = örnek();
        let zincir = vec![
            VeriKümesiTanımı::kaynak(kaynak),
            VeriKümesiTanımı::sırala([SıralamaAnahtarı::azalan("satış")]),
            VeriKümesiTanımı::kaynaktan_süz(
                0,
                SüzmeKoşulu::Arasında {
                    boyut: BoyutSeçici::ad("satış"),
                    en_az: 100.0,
                    en_çok: 130.0,
                },
            ),
        ];
        let sonuçlar = veri_kümelerini_çöz(&zincir).unwrap();
        assert_eq!(
            sonuçlar[1].metinler("ürün").unwrap(),
            vec!["Kiraz", "Elma", "Armut"]
        );
        assert_eq!(sonuçlar[2].metinler("ürün").unwrap(), vec!["Elma"]);
    }

    #[test]
    fn kardeş_süzmelerin_ikisi_de_varsayılan_olarak_sıfırıncı_dataseti_kullanır() {
        let kaynak = VeriKümesi::yeni(["ürün", "satış", "yıl"])
            .satır(["Kek".into(), 123.into(), 2011.into()])
            .satır(["Tahıl".into(), 231.into(), 2011.into()])
            .satır(["Kek".into(), 143.into(), 2012.into()]);
        let yıla_göre = |yıl: i32| {
            VeriKümesiTanımı::süz(SüzmeKoşulu::Karşılaştır {
                boyut: BoyutSeçici::ad("yıl"),
                işlem: Karşılaştırmaİşlemi::Eşit,
                değer: yıl.into(),
            })
        };

        let sonuçlar = veri_kümelerini_çöz(&[
            VeriKümesiTanımı::kaynak(kaynak),
            yıla_göre(2011),
            yıla_göre(2012),
        ])
        .unwrap();

        assert_eq!(sonuçlar[1].metinler("ürün").unwrap(), vec!["Kek", "Tahıl"]);
        assert_eq!(sonuçlar[2].metinler("ürün").unwrap(), vec!["Kek"]);
    }

    #[test]
    fn array_rows_otomatik_baslik_ve_boyut_turu() {
        let kaynak = VeriKaynağı::DiziSatırlar(vec![
            vec!["ürün".into(), "satış".into(), "stokta".into()],
            vec!["Elma".into(), 12.into(), true.into()],
            vec!["Armut".into(), 7.into(), false.into()],
        ]);
        let depo = VeriDeposu::kaynaktan(kaynak, KaynakSeçenekleri::default()).unwrap();
        assert_eq!(depo.sayım(), 2);
        assert_eq!(depo.boyutlar[0].ad, "ürün");
        assert_eq!(depo.boyutlar[0].tür, BoyutTürü::Sıralı);
        assert_eq!(depo.boyutlar[1].tür, BoyutTürü::Sayı);
        assert_eq!(depo.boyutlar[2].tür, BoyutTürü::Mantıksal);
        assert_eq!(depo.değer(1, &BoyutSeçici::ad("satış")), Some(&7.into()));
    }

    #[test]
    fn row_layout_ilk_sutunu_boyut_adi_sayar() {
        let kaynak = VeriKaynağı::DiziSatırlar(vec![
            vec!["ürün".into(), "Elma".into(), "Armut".into()],
            vec!["satış".into(), 12.into(), 7.into()],
        ]);
        let depo = VeriDeposu::kaynaktan(
            kaynak,
            KaynakSeçenekleri {
                yerleşim: SeriYerleşimi::Satır,
                ..KaynakSeçenekleri::default()
            },
        )
        .unwrap();
        assert_eq!(depo.sayım(), 2);
        assert_eq!(depo.boyutlar[0].ad, "ürün");
        assert_eq!(depo.boyutlar[1].ad, "satış");
        assert_eq!(
            depo.değer(0, &BoyutSeçici::ad("ürün")),
            Some(&VeriDeğeri::Metin("Elma".to_owned()))
        );
    }

    #[test]
    fn object_keyed_columns_ve_typed_array_normalize_edilir() {
        let nesneler = VeriKaynağı::NesneSatırlar(vec![
            vec![("x".to_owned(), 1.into()), ("y".to_owned(), 2.into())],
            vec![("x".to_owned(), 3.into())],
        ]);
        let nesne_deposu = VeriDeposu::kaynaktan(nesneler, KaynakSeçenekleri::default()).unwrap();
        assert_eq!(nesne_deposu.sayım(), 2);
        assert!(matches!(
            nesne_deposu.değer(1, &BoyutSeçici::ad("y")),
            Some(VeriDeğeri::Boş)
        ));

        let sütunlar = VeriKaynağı::AnahtarlıSütunlar(vec![
            ("x".to_owned(), vec![1.into(), 2.into()]),
            ("y".to_owned(), vec![3.into()]),
        ]);
        let sütun_deposu = VeriDeposu::kaynaktan(sütunlar, KaynakSeçenekleri::default()).unwrap();
        assert_eq!(sütun_deposu.sayım(), 2);

        let türlü = VeriKaynağı::TürlüDizi {
            değerler: TürlüSayıDizisi::F32(vec![1.0, 2.0, 3.0, 4.0]),
            boyut_sayısı: 2,
        };
        let türlü_depo = VeriDeposu::kaynaktan(
            türlü,
            KaynakSeçenekleri {
                boyutlar: vec![BoyutTanımı::yeni("x"), BoyutTanımı::yeni("y")],
                ..KaynakSeçenekleri::default()
            },
        )
        .unwrap();
        assert_eq!(türlü_depo.sayım(), 2);
        assert_eq!(
            türlü_depo.değer(1, &BoyutSeçici::ad("y")),
            Some(&VeriDeğeri::Sayı(4.0))
        );
    }

    #[test]
    fn datastore_indeks_gorunumu_sort_filter_range_append() {
        let mut depo = örnek().depoya().unwrap();
        assert_eq!(
            depo.kapsam(&BoyutSeçici::ad("satış")).unwrap(),
            [80.0, 160.0]
        );
        let süzülü = depo
            .aralık_seç(&BoyutSeçici::ad("satış"), 100.0, 130.0)
            .unwrap();
        assert_eq!(süzülü.sayım(), 1);
        let sıralı = depo.sırala(&[SıralamaAnahtarı::azalan("satış")]).unwrap();
        assert_eq!(
            sıralı.değer(0, &BoyutSeçici::ad("ürün")),
            Some(&VeriDeğeri::Metin("Kiraz".to_owned()))
        );
        assert_eq!(
            depo.ekle(vec![vec!["Muz".into(), 90.into(), 20.into()]])
                .unwrap(),
            [3, 4]
        );
        assert_eq!(depo.sayım(), 4);
        assert!(süzülü.clone().ekle(vec![vec![]]).is_err());
    }

    #[test]
    fn builtin_transform_ve_dataset_zinciri() {
        let kaynak = örnek().depoya().unwrap();
        let süz = SüzmeDönüşümü {
            koşul: SüzmeKoşulu::Ve(vec![
                SüzmeKoşulu::Karşılaştır {
                    boyut: BoyutSeçici::ad("satış"),
                    işlem: Karşılaştırmaİşlemi::BüyükEşit,
                    değer: 100.into(),
                },
                SüzmeKoşulu::Karşılaştır {
                    boyut: BoyutSeçici::ad("ürün"),
                    işlem: Karşılaştırmaİşlemi::İçerir,
                    değer: "a".into(),
                },
            ]),
        };
        let süzülü = süz.uygula(std::slice::from_ref(&kaynak)).unwrap();
        assert_eq!(süzülü[0].sayım(), 2);

        let sırala = SıralamaDönüşümü {
            anahtarlar: vec![SıralamaAnahtarı::azalan("satış")],
        };
        let mut zincir = VeriKümesiZinciri::yeni();
        zincir
            .kaynak_ekle(
                "ham",
                VeriKaynağı::DiziSatırlar(vec![
                    vec!["ürün".into(), "satış".into()],
                    vec!["A".into(), 1.into()],
                    vec!["B".into(), 3.into()],
                ]),
                KaynakSeçenekleri::default(),
            )
            .unwrap();
        zincir
            .dönüştür("sıralı", [("ham".to_owned(), 0)], &sırala)
            .unwrap();
        assert_eq!(
            zincir
                .al("sıralı", 0)
                .unwrap()
                .değer(0, &BoyutSeçici::ad("ürün")),
            Some(&VeriDeğeri::Metin("B".to_owned()))
        );
    }

    fn resmi_histogram_kaynağı() -> VeriKümesi {
        let satırlar = [
            [8.3, 143.0],
            [8.6, 214.0],
            [8.8, 251.0],
            [10.5, 26.0],
            [10.7, 86.0],
            [10.8, 93.0],
            [11.0, 176.0],
            [11.0, 39.0],
            [11.1, 221.0],
            [11.2, 188.0],
            [11.3, 57.0],
            [11.4, 91.0],
            [11.4, 191.0],
            [11.7, 8.0],
            [12.0, 196.0],
            [12.9, 177.0],
            [12.9, 153.0],
            [13.3, 201.0],
            [13.7, 199.0],
            [13.8, 47.0],
            [14.0, 81.0],
            [14.2, 98.0],
            [14.5, 121.0],
            [16.0, 37.0],
            [16.3, 12.0],
            [17.3, 105.0],
            [17.5, 168.0],
            [17.9, 84.0],
            [18.0, 197.0],
            [18.0, 155.0],
            [20.6, 125.0],
        ];
        satırlar
            .into_iter()
            .fold(VeriKümesi::yeni(["v0", "v1"]), |küme, [v0, v1]| {
                küme.satır([v0.into(), v1.into()])
            })
    }

    #[test]
    fn ecstat_histogram_resmi_kutu_sinirlarini_ve_iki_sonucu_uretir() {
        let kaynak = resmi_histogram_kaynağı();
        let x = HistogramDönüşümü::yeni()
            .uygula(&[kaynak.depoya().unwrap()])
            .unwrap();
        assert_eq!(x.len(), 2);
        assert_eq!(
            x[0].boyutlar
                .iter()
                .map(|boyut| boyut.ad.as_str())
                .collect::<Vec<_>>(),
            ["MeanOfV0V1", "VCount", "V0", "V1", "DisplayableName"]
        );
        assert_eq!(
            x[0].satırları_kopyala(),
            vec![
                vec![
                    9.0.into(),
                    3.0.into(),
                    8.0.into(),
                    10.0.into(),
                    "8 - 10".into()
                ],
                vec![
                    11.0.into(),
                    11.0.into(),
                    10.0.into(),
                    12.0.into(),
                    "10 - 12".into()
                ],
                vec![
                    13.0.into(),
                    6.0.into(),
                    12.0.into(),
                    14.0.into(),
                    "12 - 14".into()
                ],
                vec![
                    15.0.into(),
                    3.0.into(),
                    14.0.into(),
                    16.0.into(),
                    "14 - 16".into()
                ],
                vec![
                    17.0.into(),
                    5.0.into(),
                    16.0.into(),
                    18.0.into(),
                    "16 - 18".into()
                ],
                vec![
                    19.0.into(),
                    2.0.into(),
                    18.0.into(),
                    20.0.into(),
                    "18 - 20".into()
                ],
                vec![
                    21.0.into(),
                    1.0.into(),
                    20.0.into(),
                    22.0.into(),
                    "20 - 22".into()
                ],
            ]
        );
        assert_eq!(
            x[1].satırları_kopyala()[0],
            vec![8.0.into(), 10.0.into(), 3.0.into()]
        );

        let y = HistogramDönüşümü::yeni()
            .boyut("v1")
            .uygula(&[kaynak.depoya().unwrap()])
            .unwrap();
        assert_eq!(
            y[0].satırları_kopyala()
                .iter()
                .map(|satır| satır[1].sayı().unwrap() as usize)
                .collect::<Vec<_>>(),
            [6, 7, 4, 10, 3, 1]
        );
        assert_eq!(
            y[0].satırları_kopyala()[0],
            vec![
                25.0.into(),
                6.0.into(),
                0.0.into(),
                50.0.into(),
                "0 - 50".into(),
            ]
        );

        let zincir = veri_kümelerini_çöz(&[
            VeriKümesiTanımı::kaynak(kaynak),
            VeriKümesiTanımı::histogram(HistogramDönüşümü::yeni()),
            VeriKümesiTanımı::histogram(HistogramDönüşümü::yeni().boyut(1usize)),
        ])
        .unwrap();
        assert_eq!(zincir[1].satırlar.len(), 7);
        assert_eq!(zincir[2].satırlar.len(), 6);
    }

    #[test]
    fn ecstat_histogram_tum_esik_yontemlerinde_ornek_sayisini_korur() {
        let kaynak = resmi_histogram_kaynağı().depoya().unwrap();
        for yöntem in [
            HistogramEşikYöntemi::Karekök,
            HistogramEşikYöntemi::Scott,
            HistogramEşikYöntemi::FreedmanDiaconis,
            HistogramEşikYöntemi::Sturges,
        ] {
            let çıktı = HistogramDönüşümü::yeni()
                .yöntem(yöntem)
                .boyut("v0")
                .uygula(std::slice::from_ref(&kaynak))
                .unwrap();
            let toplam = çıktı[0]
                .satırları_kopyala()
                .iter()
                .filter_map(|satır| satır.get(1).and_then(VeriDeğeri::sayı))
                .sum::<f64>();
            assert_eq!(toplam, 31.0, "{yöntem:?}");
        }
    }

    #[test]
    fn ecstat_hiyerarsik_k_means_kume_ve_merkez_boyutlarini_uretir() {
        let kaynak = VeriKümesi::yeni(["x", "y"])
            .satır([0.0.into(), 0.0.into()])
            .satır([0.0.into(), 1.0.into()])
            .satır([10.0.into(), 10.0.into()])
            .satır([11.0.into(), 10.0.into()])
            .satır([(-10.0).into(), 8.0.into()])
            .satır([(-11.0).into(), 9.0.into()]);
        let dönüşüm = KümelemeDönüşümü::yeni(3, 2)
            .çıktı_küme_adı("cluster")
            .çıktı_merkez_boyutları([3, 4])
            .tohum(0x5eed_1234);

        let çıktılar = dönüşüm.uygula(&[kaynak.depoya().unwrap()]).unwrap();

        assert_eq!(çıktılar.len(), 2);
        assert_eq!(
            çıktılar[0]
                .satırları_kopyala()
                .into_iter()
                .map(|satır| satır[2].sayı().unwrap() as usize)
                .collect::<Vec<_>>(),
            vec![0, 0, 1, 1, 2, 2]
        );
        assert_eq!(
            çıktılar[0].satırları_kopyala()[0],
            vec![0.0.into(), 0.0.into(), 0.0.into(), 0.0.into(), 0.5.into()]
        );
        assert_eq!(
            çıktılar[1].satırları_kopyala(),
            vec![
                vec![0.0.into(), 0.5.into()],
                vec![10.5.into(), 10.0.into()],
                vec![(-10.5).into(), 8.5.into()],
            ]
        );

        let zincir = veri_kümelerini_çöz(&[
            VeriKümesiTanımı::kaynak(kaynak),
            VeriKümesiTanımı::kümele(dönüşüm),
        ])
        .unwrap();
        assert_eq!(
            zincir[1].boyutlar,
            ["x", "y", "cluster", "boyut3", "boyut4"]
        );
    }

    #[test]
    fn ecstat_regresyon_yontemleri_sirali_nokta_ve_formul_uretir() {
        let doğrusal = VeriKümesi::yeni(["x", "y"])
            .satır([2.0.into(), 5.0.into()])
            .satır([0.0.into(), 1.0.into()])
            .satır([1.0.into(), 3.0.into()]);
        let çıktı = RegresyonDönüşümü::yeni(RegresyonYöntemi::Doğrusal)
            .uygula(&[doğrusal.depoya().unwrap()])
            .unwrap();
        assert_eq!(
            çıktı[0].satırları_kopyala(),
            vec![
                vec![0.0.into(), 1.0.into(), "".into()],
                vec![1.0.into(), 3.0.into(), "".into()],
                vec![2.0.into(), 5.0.into(), "y = 2x + 1".into()],
            ]
        );

        let üstel = VeriKümesi::yeni(["x", "y"])
            .satır([1.0.into(), (2.0 * 0.5_f64.exp()).into()])
            .satır([2.0.into(), (2.0 * 1.0_f64.exp()).into()])
            .satır([3.0.into(), (2.0 * 1.5_f64.exp()).into()])
            .satır([4.0.into(), (2.0 * 2.0_f64.exp()).into()]);
        let çıktı = RegresyonDönüşümü::yeni(RegresyonYöntemi::Üstel)
            .formül_konumu(RegresyonFormülKonumu::Başlangıç)
            .uygula(&[üstel.depoya().unwrap()])
            .unwrap();
        assert_eq!(
            çıktı[0].satırları_kopyala()[0][2],
            VeriDeğeri::from("y = 2e^(0.5x)")
        );

        let logaritmik = VeriKümesi::yeni(["x", "y"])
            .satır([1.0.into(), 4.0.into()])
            .satır([2.0.into(), (4.0 + 3.0 * 2.0_f64.ln()).into()])
            .satır([4.0.into(), (4.0 + 3.0 * 4.0_f64.ln()).into()]);
        let çıktı = RegresyonDönüşümü::yeni(RegresyonYöntemi::Logaritmik)
            .uygula(&[logaritmik.depoya().unwrap()])
            .unwrap();
        assert_eq!(
            çıktı[0].satırları_kopyala()[2][2],
            VeriDeğeri::from("y = 4 + 3ln(x)")
        );

        let polinom = VeriKümesi::yeni(["x", "y"])
            .satır([0.0.into(), 1.0.into()])
            .satır([1.0.into(), 6.0.into()])
            .satır([2.0.into(), 17.0.into()])
            .satır([3.0.into(), 34.0.into()]);
        let zincir = veri_kümelerini_çöz(&[
            VeriKümesiTanımı::kaynak(polinom),
            VeriKümesiTanımı::regresyon(
                RegresyonDönüşümü::yeni(RegresyonYöntemi::Polinom).derece(2),
            ),
        ])
        .unwrap();
        assert_eq!(zincir[1].boyutlar, ["x", "y", "boyut2"]);
        assert_eq!(
            zincir[1].satırlar[3][2],
            VeriDeğeri::from("y = 3x^2 + 2x + 1")
        );
    }

    #[test]
    fn kullanici_transformu_cok_sonuc_uretebilir() {
        struct Böl;
        impl VeriDönüşümü for Böl {
            fn tür_adı(&self) -> &str {
                "örnek:böl"
            }

            fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
                let kaynak = tek_upstream(upstream, self.tür_adı())?;
                Ok(vec![
                    kaynak.süz(|sıra, _| sıra % 2 == 0),
                    kaynak.süz(|sıra, _| sıra % 2 == 1),
                ])
            }
        }

        let kaynak = örnek().depoya().unwrap();
        let mut kayıt = DönüşümKayıtDefteri::yeni();
        kayıt.kaydet(Böl).unwrap();
        let sonuçlar = kayıt.çalıştır("örnek:böl", &[kaynak]).unwrap();
        assert_eq!(sonuçlar.len(), 2);
        assert_eq!(sonuçlar[0].sayım(), 2);
        assert_eq!(sonuçlar[1].sayım(), 1);
        assert!(kayıt.kaydet(Böl).is_err());
    }

    #[test]
    fn aggregate_grupları_ve_type7_çeyrekleri_hesaplar() {
        let kaynak = VeriKümesi::yeni(["Country", "Income"])
            .satır(["A".into(), 1.into()])
            .satır(["A".into(), 2.into()])
            .satır(["B".into(), 10.into()])
            .satır(["A".into(), 3.into()])
            .satır(["B".into(), 20.into()])
            .satır(["A".into(), 4.into()])
            .depoya()
            .unwrap();
        let dönüşüm = ToplamaDönüşümü::yeni(
            "Country",
            [
                ToplamaBoyutu::en_az("min", "Income"),
                ToplamaBoyutu::çeyrek1("Q1", "Income"),
                ToplamaBoyutu::ortanca("median", "Income"),
                ToplamaBoyutu::çeyrek3("Q3", "Income"),
                ToplamaBoyutu::en_çok("max", "Income"),
                ToplamaBoyutu::ilk("Country", "Country"),
            ],
        );

        let sonuçlar = dönüşüm.uygula(&[kaynak]).unwrap();
        let sonuç = &sonuçlar[0];
        assert_eq!(sonuç.sayım(), 2);
        assert_eq!(
            sonuç.satırları_kopyala()[0],
            vec![
                1.0.into(),
                1.75.into(),
                2.5.into(),
                3.25.into(),
                4.0.into(),
                "A".into(),
            ]
        );
        // Gruplar JavaScript Map gibi kaynakta ilk görülme sırasını korur.
        assert_eq!(
            sonuç.değer(1, &BoyutSeçici::ad("Country")),
            Some(&VeriDeğeri::Metin("B".to_owned()))
        );
    }

    #[test]
    fn boxplot_transform_özet_ve_aykırı_sonuçlarını_üretir() {
        let kaynak = VeriKümesi::yeni(["a", "b", "c", "d", "e"])
            .satır([1.into(), 2.into(), 3.into(), 4.into(), 100.into()])
            .depoya()
            .unwrap();
        let sonuçlar = KutuDönüşümü::yeni()
            .öğe_adı_biçimi("expr {value}")
            .uygula(&[kaynak])
            .unwrap();

        assert_eq!(sonuçlar.len(), 2);
        assert_eq!(
            sonuçlar[0].satırları_kopyala()[0],
            vec![
                "expr 0".into(),
                1.0.into(),
                2.0.into(),
                3.0.into(),
                4.0.into(),
                7.0.into(),
            ]
        );
        assert_eq!(
            sonuçlar[1].satırları_kopyala(),
            vec![vec!["expr 0".into(), 100.0.into()]]
        );
    }
}
