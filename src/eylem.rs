//! `dispatchAction`, action kayıt defteri, olay sorguları ve bağlı grafikler.
//!
//! Bu modül gpui olaylarından bağımsızdır; başsız etkileşim senaryoları da
//! aynı yükleri ve sorgu eşlemesini kullanır.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::calisma_zamani::{
    AğaçHaritasıKökDikdörtgeni, EksenBoyutu, EksenKırılmaDeğişikliği, EksenKırılmaEylemi,
    GrafikÇalışmaZamanı, GöstergeSeçimEylemi, SeriSeçici,
};
use crate::hata::BilesenHatasi;
use crate::model::bilesen::{
    FırçaKoordinatAralığı, FırçaKoordinatı, FırçaSeçimAlanı, FırçaTürü
};
use crate::model::deger::{VeriDeğeri, VeriÖğesi};
use crate::model::yakinlastirma::YakınlaştırmaDeğeri;

/// JSON-benzeri action/event yük değeri.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum EylemDeğeri {
    #[default]
    Boş,
    Mantıksal(bool),
    Sayı(f64),
    Metin(String),
    Dizi(Vec<EylemDeğeri>),
    Nesne(BTreeMap<String, EylemDeğeri>),
}

impl EylemDeğeri {
    pub fn sayı(&self) -> Option<f64> {
        match self {
            Self::Sayı(sayı) => Some(*sayı),
            Self::Metin(metin) => metin.parse().ok(),
            _ => None,
        }
    }

    pub fn metin(&self) -> Option<&str> {
        match self {
            Self::Metin(metin) => Some(metin),
            _ => None,
        }
    }

    pub fn dizi(&self) -> Option<&[EylemDeğeri]> {
        match self {
            Self::Dizi(dizi) => Some(dizi),
            _ => None,
        }
    }

    pub fn nesne(&self) -> Option<&BTreeMap<String, EylemDeğeri>> {
        match self {
            Self::Nesne(nesne) => Some(nesne),
            _ => None,
        }
    }

    pub fn mantıksal(&self) -> Option<bool> {
        match self {
            Self::Mantıksal(değer) => Some(*değer),
            _ => None,
        }
    }
}

impl From<f64> for EylemDeğeri {
    fn from(değer: f64) -> Self {
        Self::Sayı(değer)
    }
}

impl From<f32> for EylemDeğeri {
    fn from(değer: f32) -> Self {
        Self::Sayı(değer as f64)
    }
}

impl From<i32> for EylemDeğeri {
    fn from(değer: i32) -> Self {
        Self::Sayı(değer as f64)
    }
}

impl From<usize> for EylemDeğeri {
    fn from(değer: usize) -> Self {
        Self::Sayı(değer as f64)
    }
}

impl From<bool> for EylemDeğeri {
    fn from(değer: bool) -> Self {
        Self::Mantıksal(değer)
    }
}

impl From<&str> for EylemDeğeri {
    fn from(değer: &str) -> Self {
        Self::Metin(değer.to_owned())
    }
}

impl From<String> for EylemDeğeri {
    fn from(değer: String) -> Self {
        Self::Metin(değer)
    }
}

/// `dispatchAction(payload)`; `batch` boş değilse alt yükler sırayla çalışır.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EylemYükü {
    pub tür: String,
    pub sessiz: bool,
    pub alanlar: BTreeMap<String, EylemDeğeri>,
    pub batch: Vec<EylemYükü>,
}

impl EylemYükü {
    pub fn yeni(tür: impl Into<String>) -> Self {
        Self {
            tür: tür.into(),
            ..Self::default()
        }
    }

    pub fn alan(mut self, ad: impl Into<String>, değer: impl Into<EylemDeğeri>) -> Self {
        self.alanlar.insert(ad.into(), değer.into());
        self
    }

    pub fn sessiz(mut self, açık: bool) -> Self {
        self.sessiz = açık;
        self
    }

    pub fn batch(mut self, yükler: impl IntoIterator<Item = EylemYükü>) -> Self {
        self.batch = yükler.into_iter().collect();
        self
    }

    pub fn al(&self, ad: &str) -> Option<&EylemDeğeri> {
        self.alanlar.get(ad)
    }

    pub fn seri_seçici(&self) -> Option<SeriSeçici> {
        if let Some(kimlik) = self.al("seriesId").and_then(EylemDeğeri::metin) {
            return Some(SeriSeçici::kimlik(kimlik));
        }
        if let Some(ad) = self.al("seriesName").and_then(EylemDeğeri::metin) {
            return Some(SeriSeçici::ad(ad));
        }
        self.al("seriesIndex")
            .and_then(EylemDeğeri::sayı)
            .filter(|sıra| sıra.is_finite() && *sıra >= 0.0)
            .map(|sıra| SeriSeçici::Sıra(sıra as usize))
    }
}

/// Action handler'ın yayımladığı normalize olay yükü.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct OlayYükü {
    pub tür: String,
    pub bileşen_türü: Option<String>,
    pub bileşen_alt_türü: Option<String>,
    pub bileşen_sırası: Option<usize>,
    pub bileşen_kimliği: Option<String>,
    pub bileşen_adı: Option<String>,
    pub seri_sırası: Option<usize>,
    pub seri_kimliği: Option<String>,
    pub seri_adı: Option<String>,
    pub veri_sırası: Option<usize>,
    pub veri_adı: Option<String>,
    pub öğe: Option<String>,
    pub alanlar: BTreeMap<String, EylemDeğeri>,
}

impl OlayYükü {
    pub fn yeni(tür: impl Into<String>) -> Self {
        Self {
            tür: tür.into(),
            ..Self::default()
        }
    }
}

/// `chart.on(event, query, handler)` sorgusu.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct OlaySorgusu {
    pub bileşen_türü: Option<String>,
    pub bileşen_alt_türü: Option<String>,
    pub bileşen_sırası: Option<usize>,
    pub bileşen_kimliği: Option<String>,
    pub bileşen_adı: Option<String>,
    pub seri_sırası: Option<usize>,
    pub seri_kimliği: Option<String>,
    pub seri_adı: Option<String>,
    pub veri_sırası: Option<usize>,
    pub veri_adı: Option<String>,
    pub öğe: Option<String>,
}

impl OlaySorgusu {
    pub fn seri_kimliği(mut self, kimlik: impl Into<String>) -> Self {
        self.seri_kimliği = Some(kimlik.into());
        self
    }

    pub fn seri_adı(mut self, ad: impl Into<String>) -> Self {
        self.seri_adı = Some(ad.into());
        self
    }

    pub fn veri_sırası(mut self, sıra: usize) -> Self {
        self.veri_sırası = Some(sıra);
        self
    }

    pub fn öğe(mut self, öğe: impl Into<String>) -> Self {
        self.öğe = Some(öğe.into());
        self
    }

    pub fn uyuyor_mu(&self, olay: &OlayYükü) -> bool {
        eşleş(&self.bileşen_türü, &olay.bileşen_türü)
            && eşleş(&self.bileşen_alt_türü, &olay.bileşen_alt_türü)
            && eşleş(&self.bileşen_sırası, &olay.bileşen_sırası)
            && eşleş(&self.bileşen_kimliği, &olay.bileşen_kimliği)
            && eşleş(&self.bileşen_adı, &olay.bileşen_adı)
            && eşleş(&self.seri_sırası, &olay.seri_sırası)
            && eşleş(&self.seri_kimliği, &olay.seri_kimliği)
            && eşleş(&self.seri_adı, &olay.seri_adı)
            && eşleş(&self.veri_sırası, &olay.veri_sırası)
            && eşleş(&self.veri_adı, &olay.veri_adı)
            && eşleş(&self.öğe, &olay.öğe)
    }
}

fn eşleş<T: PartialEq>(beklenen: &Option<T>, gerçek: &Option<T>) -> bool {
    beklenen
        .as_ref()
        .map(|değer| Some(değer) == gerçek.as_ref())
        .unwrap_or(true)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EylemGüncellemesi {
    Yok,
    Görünüm,
    Dönüşüm,
    Tam,
}

type Eylemİşlevi = dyn Fn(&mut GrafikÇalışmaZamanı, &EylemYükü) -> Result<Vec<OlayYükü>, BilesenHatasi>
    + Send
    + Sync;

struct EylemKaydı {
    olay_türü: String,
    güncelleme: EylemGüncellemesi,
    işlev: Arc<Eylemİşlevi>,
}

/// `registerAction` ve `dispatchAction` karşılığı.
#[derive(Default)]
pub struct EylemKayıtDefteri {
    kayıtlar: BTreeMap<String, EylemKaydı>,
}

impl EylemKayıtDefteri {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn kaydet<F>(
        &mut self,
        eylem_türü: impl Into<String>,
        olay_türü: impl Into<String>,
        güncelleme: EylemGüncellemesi,
        işlev: F,
    ) -> Result<(), BilesenHatasi>
    where
        F: Fn(&mut GrafikÇalışmaZamanı, &EylemYükü) -> Result<Vec<OlayYükü>, BilesenHatasi>
            + Send
            + Sync
            + 'static,
    {
        let eylem_türü = eylem_türü.into();
        if eylem_türü.is_empty() || self.kayıtlar.contains_key(&eylem_türü) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "action.type",
                ayrıntı: if eylem_türü.is_empty() {
                    "eylem türü boş olamaz".to_owned()
                } else {
                    format!("`{eylem_türü}` eylemi zaten kayıtlı")
                },
            });
        }
        self.kayıtlar.insert(
            eylem_türü,
            EylemKaydı {
                olay_türü: olay_türü.into(),
                güncelleme,
                işlev: Arc::new(işlev),
            },
        );
        Ok(())
    }

    pub fn güncelleme_türü(&self, eylem_türü: &str) -> Option<EylemGüncellemesi> {
        self.kayıtlar.get(eylem_türü).map(|kayıt| kayıt.güncelleme)
    }

    /// Batch yükünü sırayla çalıştırır. Her alt yük üst türü miras alır;
    /// `silent` yalnız olay yayınını bastırır, model güncellemesini değil.
    pub fn gönder(
        &self,
        çalışma: &mut GrafikÇalışmaZamanı,
        yük: &EylemYükü,
    ) -> Result<Vec<OlayYükü>, BilesenHatasi> {
        if !yük.batch.is_empty() {
            let mut olaylar = Vec::new();
            for alt in &yük.batch {
                let mut alt = alt.clone();
                if alt.tür.is_empty() {
                    alt.tür = yük.tür.clone();
                }
                alt.sessiz |= yük.sessiz;
                olaylar.extend(self.tek_gönder(çalışma, &alt)?);
            }
            return Ok(olaylar);
        }
        self.tek_gönder(çalışma, yük)
    }

    fn tek_gönder(
        &self,
        çalışma: &mut GrafikÇalışmaZamanı,
        yük: &EylemYükü,
    ) -> Result<Vec<OlayYükü>, BilesenHatasi> {
        let kayıt =
            self.kayıtlar
                .get(&yük.tür)
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "action.type",
                    ayrıntı: format!("`{}` eylemi kayıtlı değil", yük.tür),
                })?;
        let mut olaylar = (kayıt.işlev)(çalışma, yük)?;
        if yük.sessiz {
            olaylar.clear();
        } else {
            for olay in &mut olaylar {
                if olay.tür.is_empty() {
                    olay.tür = kayıt.olay_türü.clone();
                }
            }
        }
        Ok(olaylar)
    }
}

type Olayİşlevi = dyn Fn(&OlayYükü) + Send + Sync;

struct Dinleyici {
    kimlik: u64,
    tür: String,
    sorgu: OlaySorgusu,
    işlev: Arc<Olayİşlevi>,
}

/// `on`/`off` olay kayıt defteri ve query süzmesi.
#[derive(Default)]
pub struct OlayKayıtDefteri {
    sıradaki_kimlik: u64,
    dinleyiciler: Vec<Dinleyici>,
}

impl OlayKayıtDefteri {
    pub fn dinle<F>(&mut self, tür: impl Into<String>, sorgu: OlaySorgusu, işlev: F) -> u64
    where
        F: Fn(&OlayYükü) + Send + Sync + 'static,
    {
        self.sıradaki_kimlik = self.sıradaki_kimlik.saturating_add(1);
        let kimlik = self.sıradaki_kimlik;
        self.dinleyiciler.push(Dinleyici {
            kimlik,
            tür: tür.into(),
            sorgu,
            işlev: Arc::new(işlev),
        });
        kimlik
    }

    pub fn bırak(&mut self, kimlik: u64) -> bool {
        let önce = self.dinleyiciler.len();
        self.dinleyiciler
            .retain(|dinleyici| dinleyici.kimlik != kimlik);
        önce != self.dinleyiciler.len()
    }

    pub fn yayınla(&self, olay: &OlayYükü) -> usize {
        let mut adet = 0usize;
        for dinleyici in &self.dinleyiciler {
            if dinleyici.tür == olay.tür && dinleyici.sorgu.uyuyor_mu(olay) {
                (dinleyici.işlev)(olay);
                adet = adet.saturating_add(1);
            }
        }
        adet
    }
}

/// `echarts.connect(group)` hedef yönlendirme tablosu. Grafik örneklerinin
/// sahipliği çağıranda kalır; bu tip yalnız döngüsüz hedef listesini üretir.
#[derive(Clone, Debug, Default)]
pub struct BağlıGrafikler {
    gruplar: BTreeMap<String, BTreeSet<String>>,
    örnek_grubu: BTreeMap<String, String>,
}

impl BağlıGrafikler {
    pub fn bağla(&mut self, örnek: impl Into<String>, grup: impl Into<String>) {
        let örnek = örnek.into();
        let grup = grup.into();
        self.ayır(&örnek);
        self.gruplar
            .entry(grup.clone())
            .or_default()
            .insert(örnek.clone());
        self.örnek_grubu.insert(örnek, grup);
    }

    pub fn ayır(&mut self, örnek: &str) -> bool {
        let Some(grup) = self.örnek_grubu.remove(örnek) else {
            return false;
        };
        if let Some(örnekler) = self.gruplar.get_mut(&grup) {
            örnekler.remove(örnek);
            if örnekler.is_empty() {
                self.gruplar.remove(&grup);
            }
        }
        true
    }

    pub fn hedefler(&self, kaynak: &str) -> Vec<String> {
        let Some(grup) = self.örnek_grubu.get(kaynak) else {
            return Vec::new();
        };
        self.gruplar
            .get(grup)
            .into_iter()
            .flat_map(|örnekler| örnekler.iter())
            .filter(|örnek| örnek.as_str() != kaynak)
            .cloned()
            .collect()
    }
}

/// Resmi `appendData` action fixture'larında kullanılan hazır işleyici.
pub fn append_data_eylemini_kaydet(kayıt: &mut EylemKayıtDefteri) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "appendData",
        "dataappended",
        EylemGüncellemesi::Tam,
        |çalışma, yük| {
            let seçici = yük
                .seri_seçici()
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "action.series",
                    ayrıntı: "seriesIndex, seriesId veya seriesName gerekli".to_owned(),
                })?;
            let veri = yük
                .al("data")
                .and_then(EylemDeğeri::dizi)
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "action.data",
                    ayrıntı: "appendData için data dizisi gerekli".to_owned(),
                })?
                .iter()
                .map(eylem_değerinden_veri)
                .collect::<Result<Vec<_>, _>>()?;
            let adet = veri.len();
            let sıra = çalışma.veri_ekle(seçici, veri, true)?;
            let mut olay = OlayYükü {
                seri_sırası: Some(sıra),
                ..OlayYükü::default()
            };
            olay.alanlar
                .insert("count".to_owned(), EylemDeğeri::from(adet));
            Ok(vec![olay])
        },
    )
}

/// Çekirdek `dataZoom` action'ını kaydeder. Aynı ekseni yöneten dataZoom
/// bileşenlerinin bağlı güncellenmesi çalışma zamanı tarafından yapılır.
pub fn veri_yakınlaştırma_eylemini_kaydet(
    kayıt: &mut EylemKayıtDefteri,
) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "dataZoom",
        "datazoom",
        EylemGüncellemesi::Tam,
        |çalışma, yük| {
            if yük.al("dataZoomId").is_some() {
                return Err(BilesenHatasi::Desteklenmeyen {
                    özellik: "dataZoomId",
                    ayrıntı: "Rust dataZoom modelinde henüz bileşen kimliği bulunmuyor"
                        .to_owned(),
                });
            }
            let sıra = isteğe_bağlı_sıra(yük, "dataZoomIndex")?;
            let başlangıç = isteğe_bağlı_sayı(yük, "start")?.map(|değer| değer as f32);
            let bitiş = isteğe_bağlı_sayı(yük, "end")?.map(|değer| değer as f32);
            let başlangıç_değeri = isteğe_bağlı_yakınlaştırma_değeri(yük, "startValue")?;
            let bitiş_değeri = isteğe_bağlı_yakınlaştırma_değeri(yük, "endValue")?;
            let değişiklikler = çalışma.veri_yakınlaştırma_aralığını_ayarla(
                sıra,
                başlangıç,
                bitiş,
                başlangıç_değeri.clone(),
                bitiş_değeri.clone(),
                true,
            )?;

            let mut olay = OlayYükü {
                bileşen_türü: Some("dataZoom".to_owned()),
                bileşen_sırası: sıra,
                ..OlayYükü::default()
            };
            if let Some(başlangıç) = başlangıç {
                olay.alanlar
                    .insert("start".to_owned(), EylemDeğeri::from(başlangıç));
            }
            if let Some(bitiş) = bitiş {
                olay.alanlar
                    .insert("end".to_owned(), EylemDeğeri::from(bitiş));
            }
            if let Some(değer) = başlangıç_değeri {
                olay.alanlar.insert(
                    "startValue".to_owned(),
                    yakınlaştırma_değerinden_eylem(değer),
                );
            }
            if let Some(değer) = bitiş_değeri {
                olay.alanlar
                    .insert("endValue".to_owned(), yakınlaştırma_değerinden_eylem(değer));
            }
            olay.alanlar.insert(
                "affectedIndices".to_owned(),
                EylemDeğeri::Dizi(
                    değişiklikler
                        .iter()
                        .map(|(sıra, _, _)| EylemDeğeri::from(*sıra))
                        .collect(),
                ),
            );
            Ok(vec![olay])
        },
    )
}

/// ECharts `brush` action'ı. Programatik `coordRange` alanlarını modelde
/// tutar; veri koordinatından piksele dönüşüm ve `brushLink` görseli ortak
/// boyama hattında çözülür.
pub fn fırça_eylemini_kaydet(kayıt: &mut EylemKayıtDefteri) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "brush",
        "brush",
        EylemGüncellemesi::Görünüm,
        |çalışma, yük| {
            let alanlar = yük.al("areas").map(fırça_alanlarını_oku).transpose()?;
            let değişti = çalışma.fırça_alanlarını_ayarla(alanlar, true)?;
            let mut olay = OlayYükü {
                bileşen_türü: Some("brush".to_owned()),
                bileşen_sırası: Some(0),
                ..OlayYükü::default()
            };
            if let Some(alanlar) = yük.al("areas") {
                olay.alanlar.insert("areas".to_owned(), alanlar.clone());
            }
            olay.alanlar
                .insert("changed".to_owned(), EylemDeğeri::from(değişti));
            Ok(vec![olay])
        },
    )
}

/// `axisAreaSelect` ve `parallelAxisExpand` eylemlerini kaydeder.
pub fn paralel_eylemlerini_kaydet(kayıt: &mut EylemKayıtDefteri) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "axisAreaSelect",
        "axisAreaSelected",
        EylemGüncellemesi::Görünüm,
        |çalışma, yük| {
            let eksen_sırası = isteğe_bağlı_sıra(yük, "parallelAxisIndex")?;
            let eksen_kimliği = yük.al("parallelAxisId").and_then(EylemDeğeri::metin);
            let aralıklar = paralel_aralıklarını_oku(yük.al("intervals").ok_or_else(|| {
                BilesenHatasi::GeçersizSeçenek {
                    alan: "axisAreaSelect.intervals",
                    ayrıntı: "iki sayılı aralıklardan oluşan dizi gerekli".to_owned(),
                }
            })?)?;
            let hedefler = çalışma.paralel_eksen_aralıklarını_ayarla(
                eksen_sırası,
                eksen_kimliği,
                aralıklar.clone(),
                true,
            )?;
            let ilk = hedefler.first().copied();
            Ok(vec![OlayYükü {
                bileşen_türü: Some("parallelAxis".to_owned()),
                bileşen_sırası: ilk,
                bileşen_kimliği: eksen_kimliği.map(str::to_owned),
                alanlar: BTreeMap::from([
                    (
                        "parallelAxisIndex".to_owned(),
                        EylemDeğeri::Dizi(hedefler.into_iter().map(EylemDeğeri::from).collect()),
                    ),
                    (
                        "intervals".to_owned(),
                        EylemDeğeri::Dizi(
                            aralıklar
                                .iter()
                                .map(|aralık| {
                                    EylemDeğeri::Dizi(
                                        aralık.iter().copied().map(EylemDeğeri::from).collect(),
                                    )
                                })
                                .collect(),
                        ),
                    ),
                ]),
                ..OlayYükü::default()
            }])
        },
    )?;

    kayıt.kaydet(
        "parallelAxisExpand",
        "parallelAxisExpand",
        EylemGüncellemesi::Tam,
        |çalışma, yük| {
            let paralel_sırası = isteğe_bağlı_sıra(yük, "parallelIndex")?;
            let paralel_kimliği = yük.al("parallelId").and_then(EylemDeğeri::metin);
            let pencere =
                paralel_penceresini_oku(yük.al("axisExpandWindow").ok_or_else(|| {
                    BilesenHatasi::GeçersizSeçenek {
                        alan: "parallelAxisExpand.axisExpandWindow",
                        ayrıntı: "iki sayılı pencere gerekli".to_owned(),
                    }
                })?)?;
            let hedefler = çalışma.paralel_genişletme_penceresini_ayarla(
                paralel_sırası,
                paralel_kimliği,
                pencere,
                true,
            )?;
            let ilk = hedefler.first().copied();
            Ok(vec![OlayYükü {
                bileşen_türü: Some("parallel".to_owned()),
                bileşen_sırası: ilk,
                bileşen_kimliği: paralel_kimliği.map(str::to_owned),
                alanlar: BTreeMap::from([
                    (
                        "parallelIndex".to_owned(),
                        EylemDeğeri::Dizi(hedefler.into_iter().map(EylemDeğeri::from).collect()),
                    ),
                    (
                        "axisExpandWindow".to_owned(),
                        EylemDeğeri::Dizi(pencere.into_iter().map(EylemDeğeri::from).collect()),
                    ),
                ]),
                ..OlayYükü::default()
            }])
        },
    )
}

fn sankey_seçicileri(
    çalışma: &GrafikÇalışmaZamanı,
    yük: &EylemYükü,
    bileşen: &'static str,
) -> Result<Vec<SeriSeçici>, BilesenHatasi> {
    if let Some(seçici) = yük.seri_seçici() {
        return Ok(vec![seçici]);
    }
    let seçiciler = çalışma
        .seçenekleri_al()?
        .seriler
        .iter()
        .enumerate()
        .filter_map(|(sıra, seri)| {
            matches!(seri, crate::model::seri::Seri::Sankey(_)).then_some(SeriSeçici::Sıra(sıra))
        })
        .collect::<Vec<_>>();
    if seçiciler.is_empty() {
        return Err(BilesenHatasi::EksikVeri { bileşen, sıra: 0 });
    }
    Ok(seçiciler)
}

fn sankey_olayı(
    çalışma: &GrafikÇalışmaZamanı,
    seri_sırası: usize,
    veri_sırası: Option<usize>,
) -> Result<OlayYükü, BilesenHatasi> {
    let seçenekler = çalışma.seçenekleri_al()?;
    let seri = seçenekler.seriler.get(seri_sırası);
    let veri_adı = veri_sırası.and_then(|veri_sırası| match seri {
        Some(crate::model::seri::Seri::Sankey(seri)) => {
            seri.düğümler.get(veri_sırası).map(|düğüm| düğüm.ad.clone())
        }
        _ => None,
    });
    Ok(OlayYükü {
        bileşen_türü: Some("series".to_owned()),
        bileşen_alt_türü: Some("sankey".to_owned()),
        seri_sırası: Some(seri_sırası),
        seri_kimliği: seri.and_then(|seri| match seri {
            crate::model::seri::Seri::Sankey(seri) => seri.kimlik.clone(),
            _ => None,
        }),
        seri_adı: seri.and_then(|seri| seri.ad().map(str::to_owned)),
        veri_sırası,
        veri_adı,
        ..OlayYükü::default()
    })
}

/// Sankey'in resmî `dragNode` ve `sankeyRoam` action'larını kaydeder.
pub fn sankey_eylemlerini_kaydet(kayıt: &mut EylemKayıtDefteri) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "dragNode",
        "dragnode",
        EylemGüncellemesi::Tam,
        |çalışma, yük| {
            let veri_sırası = isteğe_bağlı_sıra(yük, "dataIndex")?.ok_or_else(|| {
                BilesenHatasi::GeçersizSeçenek {
                    alan: "dragNode.dataIndex",
                    ayrıntı: "negatif olmayan tam dataIndex gerekli".to_owned(),
                }
            })?;
            let yerel_x = isteğe_bağlı_sayı(yük, "localX")?.ok_or_else(|| {
                BilesenHatasi::GeçersizSeçenek {
                    alan: "dragNode.localX",
                    ayrıntı: "localX gerekli".to_owned(),
                }
            })? as f32;
            let yerel_y = isteğe_bağlı_sayı(yük, "localY")?.ok_or_else(|| {
                BilesenHatasi::GeçersizSeçenek {
                    alan: "dragNode.localY",
                    ayrıntı: "localY gerekli".to_owned(),
                }
            })? as f32;
            let seçiciler = sankey_seçicileri(çalışma, yük, "dragNode.series")?;
            let mut olaylar = Vec::with_capacity(seçiciler.len());
            for seçici in seçiciler {
                let (seri_sırası, _) = çalışma.sankey_düğümünü_sürükle(
                    seçici,
                    veri_sırası,
                    yerel_x,
                    yerel_y,
                    true,
                )?;
                let mut olay = sankey_olayı(çalışma, seri_sırası, Some(veri_sırası))?;
                olay.alanlar.extend([
                    ("localX".to_owned(), EylemDeğeri::from(yerel_x)),
                    ("localY".to_owned(), EylemDeğeri::from(yerel_y)),
                ]);
                olaylar.push(olay);
            }
            Ok(olaylar)
        },
    )?;
    kayıt.kaydet(
        "sankeyRoam",
        "sankeyRoam",
        EylemGüncellemesi::Yok,
        |çalışma, yük| {
            let dx = isteğe_bağlı_sayı(yük, "dx")?.map(|değer| değer as f32);
            let dy = isteğe_bağlı_sayı(yük, "dy")?.map(|değer| değer as f32);
            if dx.is_some() != dy.is_some() {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "sankeyRoam.dx/dy",
                    ayrıntı: "pan için dx ve dy birlikte gerekli".to_owned(),
                });
            }
            let zoom = isteğe_bağlı_sayı(yük, "zoom")?.map(|değer| değer as f32);
            if dx.is_none() && zoom.is_none() {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "sankeyRoam",
                    ayrıntı: "dx/dy veya zoom gerekli".to_owned(),
                });
            }
            let origin_x = isteğe_bağlı_sayı(yük, "originX")?.map(|değer| değer as f32);
            let origin_y = isteğe_bağlı_sayı(yük, "originY")?.map(|değer| değer as f32);
            if origin_x.is_some() != origin_y.is_some() {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "sankeyRoam.originX/originY",
                    ayrıntı: "zoom kökeni için originX ve originY birlikte gerekli".to_owned(),
                });
            }
            let seçiciler = sankey_seçicileri(çalışma, yük, "sankeyRoam.series")?;
            let mut olaylar = Vec::with_capacity(seçiciler.len());
            for seçici in seçiciler {
                let (seri_sırası, merkez, yakınlaştırma) = çalışma.sankey_görünümünü_değiştir(
                    seçici, dx, dy, zoom, origin_x, origin_y, true,
                )?;
                let mut olay = sankey_olayı(çalışma, seri_sırası, None)?;
                if let (Some(dx), Some(dy)) = (dx, dy) {
                    olay.alanlar.extend([
                        ("dx".to_owned(), EylemDeğeri::from(dx)),
                        ("dy".to_owned(), EylemDeğeri::from(dy)),
                    ]);
                }
                if let Some(zoom) = zoom {
                    olay.alanlar
                        .insert("zoom".to_owned(), EylemDeğeri::from(zoom));
                }
                if let (Some(origin_x), Some(origin_y)) = (origin_x, origin_y) {
                    olay.alanlar.extend([
                        ("originX".to_owned(), EylemDeğeri::from(origin_x)),
                        ("originY".to_owned(), EylemDeğeri::from(origin_y)),
                    ]);
                }
                olay.alanlar.extend([
                    (
                        "center".to_owned(),
                        EylemDeğeri::Dizi(vec![merkez.0.into(), merkez.1.into()]),
                    ),
                    ("currentZoom".to_owned(), yakınlaştırma.into()),
                ]);
                olaylar.push(olay);
            }
            Ok(olaylar)
        },
    )
}

/// Tree dal aç/kapat action'ını kaydeder. `seriesIndex`/`seriesId`/
/// `seriesName` verilmezse ECharts `eachComponent` gibi bütün Tree
/// serilerine aynı `dataIndex` uygulanır.
pub fn ağaç_eylemlerini_kaydet(kayıt: &mut EylemKayıtDefteri) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "treeExpandAndCollapse",
        "treeExpandAndCollapse",
        EylemGüncellemesi::Tam,
        |çalışma, yük| {
            let veri_sırası = isteğe_bağlı_sıra(yük, "dataIndex")?.ok_or_else(|| {
                BilesenHatasi::GeçersizSeçenek {
                    alan: "treeExpandAndCollapse.dataIndex",
                    ayrıntı: "negatif olmayan tam dataIndex gerekli".to_owned(),
                }
            })?;
            let seçiciler = if let Some(seçici) = yük.seri_seçici() {
                vec![seçici]
            } else {
                çalışma
                    .seçenekleri_al()?
                    .seriler
                    .iter()
                    .enumerate()
                    .filter_map(|(sıra, seri)| {
                        matches!(seri, crate::model::seri::Seri::Ağaç(_))
                            .then_some(SeriSeçici::Sıra(sıra))
                    })
                    .collect()
            };
            if seçiciler.is_empty() {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "treeExpandAndCollapse.series",
                    sıra: 0,
                });
            }
            let mut olaylar = Vec::with_capacity(seçiciler.len());
            for seçici in seçiciler {
                let (seri_sırası, ad, daraltılmış) =
                    çalışma.ağaç_daraltmasını_değiştir(seçici, veri_sırası, true)?;
                let seçenekler = çalışma.seçenekleri_al()?;
                let seri = seçenekler.seriler.get(seri_sırası);
                olaylar.push(OlayYükü {
                    bileşen_türü: Some("series".to_owned()),
                    bileşen_alt_türü: Some("tree".to_owned()),
                    seri_sırası: Some(seri_sırası),
                    seri_kimliği: seri.and_then(|seri| match seri {
                        crate::model::seri::Seri::Ağaç(ağaç) => ağaç.kimlik.clone(),
                        _ => None,
                    }),
                    seri_adı: seri.and_then(|seri| seri.ad().map(str::to_owned)),
                    veri_sırası: Some(veri_sırası),
                    veri_adı: Some(ad),
                    alanlar: BTreeMap::from([(
                        "collapsed".to_owned(),
                        EylemDeğeri::from(daraltılmış),
                    )]),
                    ..OlayYükü::default()
                });
            }
            Ok(olaylar)
        },
    )
}

fn ağaç_haritası_seçicileri(
    çalışma: &GrafikÇalışmaZamanı,
    yük: &EylemYükü,
    bileşen: &'static str,
) -> Result<Vec<SeriSeçici>, BilesenHatasi> {
    if let Some(seçici) = yük.seri_seçici() {
        return Ok(vec![seçici]);
    }
    let seçiciler = çalışma
        .seçenekleri_al()?
        .seriler
        .iter()
        .enumerate()
        .filter_map(|(sıra, seri)| {
            matches!(seri, crate::model::seri::Seri::AğaçHaritası(_))
                .then_some(SeriSeçici::Sıra(sıra))
        })
        .collect::<Vec<_>>();
    if seçiciler.is_empty() {
        return Err(BilesenHatasi::EksikVeri { bileşen, sıra: 0 });
    }
    Ok(seçiciler)
}

fn ağaç_haritası_hedefi(
    çalışma: &GrafikÇalışmaZamanı,
    seçici: &SeriSeçici,
    yük: &EylemYükü,
) -> Result<Option<usize>, BilesenHatasi> {
    if let Some(sıra) = isteğe_bağlı_sıra(yük, "dataIndex")? {
        return Ok(Some(sıra));
    }
    let hedef_kimliği = yük
        .al("targetNodeId")
        .or_else(|| yük.al("targetNode"))
        .and_then(EylemDeğeri::metin);
    let Some(hedef_kimliği) = hedef_kimliği else {
        return Ok(None);
    };
    let seçenekler = çalışma.seçenekleri_al()?;
    let seri_sırası = match seçici {
        SeriSeçici::Sıra(sıra) => Some(*sıra),
        SeriSeçici::Kimlik(kimlik) => (0..seçenekler.seriler.len())
            .find(|&sıra| seçenekler.seri_kimliği(sıra) == Some(kimlik.as_str())),
        SeriSeçici::Ad(ad) => seçenekler
            .seriler
            .iter()
            .position(|seri| seri.ad() == Some(ad.as_str())),
    }
    .ok_or(BilesenHatasi::EksikVeri {
        bileşen: "treemap.series",
        sıra: 0,
    })?;
    let Some(crate::model::seri::Seri::AğaçHaritası(seri)) = seçenekler.seriler.get(seri_sırası)
    else {
        return Err(BilesenHatasi::Desteklenmeyen {
            özellik: "treemap targetNodeId",
            ayrıntı: format!("{seri_sırası}. seri `treemap` değildir"),
        });
    };
    seri.düğüm_sırası_kimlikle(hedef_kimliği)
        .map(Some)
        .ok_or(BilesenHatasi::GeçersizSeçenek {
            alan: "treemap.targetNodeId",
            ayrıntı: format!("`{hedef_kimliği}` düğümü bulunamadı"),
        })
}

fn ağaç_haritası_kök_dikdörtgenini_oku(
    yük: &EylemYükü,
) -> Result<Option<AğaçHaritasıKökDikdörtgeni>, BilesenHatasi> {
    let Some(değer) = yük.al("rootRect") else {
        return Ok(None);
    };
    let nesne = değer
        .nesne()
        .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
            alan: "treemap.rootRect",
            ayrıntı: "rootRect nesne olmalı".to_owned(),
        })?;
    let sayı = |alan: &'static str| {
        nesne
            .get(alan)
            .and_then(EylemDeğeri::sayı)
            .filter(|değer| değer.is_finite())
            .map(|değer| değer as f32)
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "treemap.rootRect",
                ayrıntı: format!("`{alan}` sonlu sayı olmalı"),
            })
    };
    Ok(Some(AğaçHaritasıKökDikdörtgeni::yeni(
        sayı("x")?,
        sayı("y")?,
        sayı("width")?,
        sayı("height")?,
    )))
}

fn ağaç_haritası_olayı(
    çalışma: &GrafikÇalışmaZamanı,
    seri_sırası: usize,
    veri_sırası: Option<usize>,
) -> Result<OlayYükü, BilesenHatasi> {
    let seçenekler = çalışma.seçenekleri_al()?;
    let seri = seçenekler.seriler.get(seri_sırası);
    let veri_adı = veri_sırası.and_then(|veri_sırası| match seri {
        Some(crate::model::seri::Seri::AğaçHaritası(seri)) => {
            seri.düğüm(veri_sırası).map(|düğüm| düğüm.ad.clone())
        }
        _ => None,
    });
    Ok(OlayYükü {
        bileşen_türü: Some("series".to_owned()),
        bileşen_alt_türü: Some("treemap".to_owned()),
        seri_sırası: Some(seri_sırası),
        seri_kimliği: seri.and_then(|seri| match seri {
            crate::model::seri::Seri::AğaçHaritası(seri) => seri.kimlik.clone(),
            _ => None,
        }),
        seri_adı: seri.and_then(|seri| seri.ad().map(str::to_owned)),
        veri_sırası,
        veri_adı,
        ..OlayYükü::default()
    })
}

/// Treemap'in dört resmî view action'ını (`treemapRootToNode`,
/// `treemapZoomToNode`, `treemapRender`, `treemapMove`) kaydeder.
pub fn ağaç_haritası_eylemlerini_kaydet(
    kayıt: &mut EylemKayıtDefteri,
) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "treemapRootToNode",
        "treemapRootToNode",
        EylemGüncellemesi::Görünüm,
        |çalışma, yük| {
            let seçiciler = ağaç_haritası_seçicileri(çalışma, yük, "treemapRootToNode.series")?;
            let mut olaylar = Vec::with_capacity(seçiciler.len());
            for seçici in seçiciler {
                let hedef = ağaç_haritası_hedefi(çalışma, &seçici, yük)?;
                let (seri_sırası, yol, yön) =
                    çalışma.ağaç_haritası_köküne_git(seçici, hedef, true)?;
                let mut olay = ağaç_haritası_olayı(çalışma, seri_sırası, hedef)?;
                olay.alanlar.insert(
                    "direction".to_owned(),
                    EylemDeğeri::from(match yön {
                        crate::cizim::olay::AğaçHaritasıKökYönü::Aşağı => "drillDown",
                        crate::cizim::olay::AğaçHaritasıKökYönü::Yukarı => "rollUp",
                    }),
                );
                olay.alanlar.insert(
                    "path".to_owned(),
                    EylemDeğeri::Dizi(yol.into_iter().map(EylemDeğeri::from).collect()),
                );
                olaylar.push(olay);
            }
            Ok(olaylar)
        },
    )?;
    kayıt.kaydet(
        "treemapZoomToNode",
        "treemapZoomToNode",
        EylemGüncellemesi::Görünüm,
        |çalışma, yük| {
            let seçiciler = ağaç_haritası_seçicileri(çalışma, yük, "treemapZoomToNode.series")?;
            let mut olaylar = Vec::with_capacity(seçiciler.len());
            for seçici in seçiciler {
                let hedef = ağaç_haritası_hedefi(çalışma, &seçici, yük)?.ok_or_else(|| {
                    BilesenHatasi::GeçersizSeçenek {
                        alan: "treemapZoomToNode.dataIndex",
                        ayrıntı: "dataIndex veya targetNodeId gerekli".to_owned(),
                    }
                })?;
                let seri_sırası =
                    çalışma.ağaç_haritası_düğümüne_yakınlaştır(seçici, hedef, true)?;
                olaylar.push(ağaç_haritası_olayı(çalışma, seri_sırası, Some(hedef))?);
            }
            Ok(olaylar)
        },
    )?;
    for tür in ["treemapRender", "treemapMove"] {
        kayıt.kaydet(
            tür,
            tür,
            EylemGüncellemesi::Görünüm,
            |çalışma, yük| {
                let seçiciler = ağaç_haritası_seçicileri(çalışma, yük, "treemapRender.series")?;
                let dikdörtgen = ağaç_haritası_kök_dikdörtgenini_oku(yük)?;
                let mut olaylar = Vec::with_capacity(seçiciler.len());
                for seçici in seçiciler {
                    let seri_sırası = çalışma.ağaç_haritası_kök_dikdörtgenini_ayarla(
                        seçici,
                        dikdörtgen,
                        true,
                    )?;
                    let mut olay = ağaç_haritası_olayı(çalışma, seri_sırası, None)?;
                    if let Some(dikdörtgen) = dikdörtgen {
                        olay.alanlar.insert(
                            "rootRect".to_owned(),
                            EylemDeğeri::Nesne(BTreeMap::from([
                                ("x".to_owned(), EylemDeğeri::from(dikdörtgen.x)),
                                ("y".to_owned(), EylemDeğeri::from(dikdörtgen.y)),
                                ("width".to_owned(), EylemDeğeri::from(dikdörtgen.genişlik)),
                                ("height".to_owned(), EylemDeğeri::from(dikdörtgen.yükseklik)),
                            ])),
                        );
                    }
                    olaylar.push(olay);
                }
                Ok(olaylar)
            },
        )?;
    }
    Ok(())
}

fn güneş_patlaması_seçicileri(
    çalışma: &GrafikÇalışmaZamanı,
    yük: &EylemYükü,
    bileşen: &'static str,
) -> Result<Vec<SeriSeçici>, BilesenHatasi> {
    if let Some(seçici) = yük.seri_seçici() {
        return Ok(vec![seçici]);
    }
    let seçiciler = çalışma
        .seçenekleri_al()?
        .seriler
        .iter()
        .enumerate()
        .filter_map(|(sıra, seri)| {
            matches!(seri, crate::model::seri::Seri::GüneşPatlaması(_))
                .then_some(SeriSeçici::Sıra(sıra))
        })
        .collect::<Vec<_>>();
    if seçiciler.is_empty() {
        return Err(BilesenHatasi::EksikVeri { bileşen, sıra: 0 });
    }
    Ok(seçiciler)
}

fn güneş_patlaması_hedefi(
    çalışma: &GrafikÇalışmaZamanı,
    seçici: &SeriSeçici,
    yük: &EylemYükü,
) -> Result<Option<usize>, BilesenHatasi> {
    if let Some(sıra) = isteğe_bağlı_sıra(yük, "dataIndex")? {
        return Ok(Some(sıra));
    }
    let hedef_kimliği = yük
        .al("targetNodeId")
        .or_else(|| yük.al("targetNode"))
        .and_then(EylemDeğeri::metin);
    let Some(hedef_kimliği) = hedef_kimliği else {
        return Ok(None);
    };
    let seçenekler = çalışma.seçenekleri_al()?;
    let seri_sırası = match seçici {
        SeriSeçici::Sıra(sıra) => Some(*sıra),
        SeriSeçici::Kimlik(kimlik) => (0..seçenekler.seriler.len())
            .find(|&sıra| seçenekler.seri_kimliği(sıra) == Some(kimlik.as_str())),
        SeriSeçici::Ad(ad) => seçenekler
            .seriler
            .iter()
            .position(|seri| seri.ad() == Some(ad.as_str())),
    }
    .ok_or(BilesenHatasi::EksikVeri {
        bileşen: "sunburst.series",
        sıra: 0,
    })?;
    let Some(crate::model::seri::Seri::GüneşPatlaması(seri)) = seçenekler.seriler.get(seri_sırası)
    else {
        return Err(BilesenHatasi::Desteklenmeyen {
            özellik: "sunburst targetNodeId",
            ayrıntı: format!("{seri_sırası}. seri `sunburst` değildir"),
        });
    };
    seri.düğüm_sırası_kimlikle(hedef_kimliği)
        .map(Some)
        .ok_or(BilesenHatasi::GeçersizSeçenek {
            alan: "sunburst.targetNodeId",
            ayrıntı: format!("`{hedef_kimliği}` düğümü bulunamadı"),
        })
}

fn güneş_patlaması_olayı(
    çalışma: &GrafikÇalışmaZamanı,
    seri_sırası: usize,
    veri_sırası: Option<usize>,
) -> Result<OlayYükü, BilesenHatasi> {
    let seçenekler = çalışma.seçenekleri_al()?;
    let seri = seçenekler.seriler.get(seri_sırası);
    let veri_adı = veri_sırası.and_then(|veri_sırası| match seri {
        Some(crate::model::seri::Seri::GüneşPatlaması(seri)) => {
            seri.düğüm(veri_sırası).map(|düğüm| düğüm.ad.clone())
        }
        _ => None,
    });
    Ok(OlayYükü {
        bileşen_türü: Some("series".to_owned()),
        bileşen_alt_türü: Some("sunburst".to_owned()),
        seri_sırası: Some(seri_sırası),
        seri_kimliği: seri.and_then(|seri| match seri {
            crate::model::seri::Seri::GüneşPatlaması(seri) => seri.kimlik.clone(),
            _ => None,
        }),
        seri_adı: seri.and_then(|seri| seri.ad().map(str::to_owned)),
        veri_sırası,
        veri_adı,
        ..OlayYükü::default()
    })
}

/// Sunburst view action'ı ile ECharts'ın eski vurgu yönlendirme adlarını
/// kaydeder.
pub fn güneş_patlaması_eylemlerini_kaydet(
    kayıt: &mut EylemKayıtDefteri,
) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "sunburstRootToNode",
        "sunburstRootToNode",
        EylemGüncellemesi::Görünüm,
        |çalışma, yük| {
            let seçiciler =
                güneş_patlaması_seçicileri(çalışma, yük, "sunburstRootToNode.series")?;
            let mut olaylar = Vec::with_capacity(seçiciler.len());
            for seçici in seçiciler {
                let hedef = güneş_patlaması_hedefi(çalışma, &seçici, yük)?;
                let (seri_sırası, yol, yön) =
                    çalışma.güneş_patlaması_köküne_git(seçici, hedef, true)?;
                let mut olay = güneş_patlaması_olayı(çalışma, seri_sırası, hedef)?;
                olay.alanlar.insert(
                    "direction".to_owned(),
                    EylemDeğeri::from(match yön {
                        crate::cizim::olay::GüneşPatlamasıKökYönü::Aşağı => "drillDown",
                        crate::cizim::olay::GüneşPatlamasıKökYönü::Yukarı => "rollUp",
                    }),
                );
                olay.alanlar.insert(
                    "path".to_owned(),
                    EylemDeğeri::Dizi(yol.into_iter().map(EylemDeğeri::from).collect()),
                );
                olaylar.push(olay);
            }
            Ok(olaylar)
        },
    )?;
    for (eski_tür, yeni_tür) in [
        ("sunburstHighlight", "highlight"),
        ("sunburstUnhighlight", "downplay"),
    ] {
        kayıt.kaydet(
            eski_tür,
            eski_tür,
            EylemGüncellemesi::Yok,
            move |çalışma, yük| {
                let seçiciler = güneş_patlaması_seçicileri(çalışma, yük, "sunburst.series")?;
                let mut olaylar = Vec::with_capacity(seçiciler.len());
                for seçici in seçiciler {
                    let hedef = güneş_patlaması_hedefi(çalışma, &seçici, yük)?;
                    let seri_sırası = match seçici {
                        SeriSeçici::Sıra(sıra) => sıra,
                        _ => {
                            let (sıra, _) = çalışma.güneş_patlaması_görünümü(seçici)?;
                            sıra
                        }
                    };
                    let mut olay = güneş_patlaması_olayı(çalışma, seri_sırası, hedef)?;
                    olay.alanlar
                        .insert("forwardedType".to_owned(), EylemDeğeri::from(yeni_tür));
                    olaylar.push(olay);
                }
                Ok(olaylar)
            },
        )?;
    }
    Ok(())
}

fn paralel_aralıklarını_oku(değer: &EylemDeğeri) -> Result<Vec<[f64; 2]>, BilesenHatasi> {
    değer
        .dizi()
        .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
            alan: "axisAreaSelect.intervals",
            ayrıntı: "intervals bir dizi olmalı".to_owned(),
        })?
        .iter()
        .enumerate()
        .map(|(sıra, değer)| {
            let uçlar = değer
                .dizi()
                .filter(|uçlar| uçlar.len() == 2)
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "axisAreaSelect.intervals",
                    ayrıntı: format!("{sıra}. aralık tam iki uç içermeli"),
                })?;
            let ilk = uçlar.first().and_then(EylemDeğeri::sayı);
            let ikinci = uçlar.get(1).and_then(EylemDeğeri::sayı);
            match (ilk, ikinci) {
                (Some(ilk), Some(ikinci)) if ilk.is_finite() && ikinci.is_finite() => {
                    Ok([ilk, ikinci])
                }
                _ => Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "axisAreaSelect.intervals",
                    ayrıntı: format!("{sıra}. aralığın uçları sonlu sayı olmalı"),
                }),
            }
        })
        .collect()
}

fn paralel_penceresini_oku(değer: &EylemDeğeri) -> Result<[f32; 2], BilesenHatasi> {
    let uçlar = değer
        .dizi()
        .filter(|uçlar| uçlar.len() == 2)
        .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
            alan: "parallelAxisExpand.axisExpandWindow",
            ayrıntı: "axisExpandWindow tam iki uç içermeli".to_owned(),
        })?;
    let ilk = uçlar.first().and_then(EylemDeğeri::sayı);
    let ikinci = uçlar.get(1).and_then(EylemDeğeri::sayı);
    match (ilk, ikinci) {
        (Some(ilk), Some(ikinci))
            if ilk.is_finite()
                && ikinci.is_finite()
                && ilk >= f32::MIN as f64
                && ilk <= f32::MAX as f64
                && ikinci >= f32::MIN as f64
                && ikinci <= f32::MAX as f64 =>
        {
            Ok([ilk as f32, ikinci as f32])
        }
        _ => Err(BilesenHatasi::GeçersizSeçenek {
            alan: "parallelAxisExpand.axisExpandWindow",
            ayrıntı: "pencere uçları sonlu f32 olmalı".to_owned(),
        }),
    }
}

fn fırça_alanlarını_oku(
    değer: &EylemDeğeri,
) -> Result<Vec<FırçaSeçimAlanı>, BilesenHatasi> {
    let alanlar = değer.dizi().ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
        alan: "action.brush.areas",
        ayrıntı: "areas bir nesne dizisi olmalı".to_owned(),
    })?;
    alanlar.iter().map(fırça_alanını_oku).collect()
}

fn fırça_alanını_oku(değer: &EylemDeğeri) -> Result<FırçaSeçimAlanı, BilesenHatasi> {
    let nesne = değer
        .nesne()
        .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
            alan: "action.brush.areas",
            ayrıntı: "her alan bir nesne olmalı".to_owned(),
        })?;
    let tür_metni = nesne
        .get("brushType")
        .and_then(EylemDeğeri::metin)
        .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
            alan: "action.brush.brushType",
            ayrıntı: "rect, polygon, lineX veya lineY gerekli".to_owned(),
        })?;
    let tür = match tür_metni {
        "rect" => FırçaTürü::Dikdörtgen,
        "polygon" => FırçaTürü::Çokgen,
        "lineX" => FırçaTürü::Yatay,
        "lineY" => FırçaTürü::Dikey,
        diğer => {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "action.brush.brushType",
                ayrıntı: format!("`{diğer}` desteklenen bir brushType değil"),
            });
        }
    };
    let koordinat = nesne
        .get("coordRange")
        .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
            alan: "action.brush.coordRange",
            ayrıntı: "koordinat ekseni seçildiğinde coordRange gerekli".to_owned(),
        })?;
    let koordinat_aralığı = match tür {
        FırçaTürü::Yatay | FırçaTürü::Dikey => {
            FırçaKoordinatAralığı::Eksen(fırça_koordinat_uçlarını_oku(koordinat)?)
        }
        FırçaTürü::Dikdörtgen => {
            let boyutlar = koordinat
                .dizi()
                .filter(|boyutlar| boyutlar.len() == 2)
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "action.brush.coordRange",
                    ayrıntı: "rect coordRange [[x0,x1],[y0,y1]] olmalı".to_owned(),
                })?;
            FırçaKoordinatAralığı::Dikdörtgen {
                x: fırça_koordinat_uçlarını_oku(&boyutlar[0])?,
                y: fırça_koordinat_uçlarını_oku(&boyutlar[1])?,
            }
        }
        FırçaTürü::Çokgen => {
            let noktalar = koordinat
                .dizi()
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "action.brush.coordRange",
                    ayrıntı: "polygon coordRange nokta çiftlerinden oluşmalı".to_owned(),
                })?
                .iter()
                .map(fırça_koordinat_uçlarını_oku)
                .collect::<Result<Vec<_>, _>>()?;
            FırçaKoordinatAralığı::Çokgen(noktalar)
        }
    };
    let sıra = |ad: &str| -> Result<Option<usize>, BilesenHatasi> {
        let Some(değer) = nesne.get(ad) else {
            return Ok(None);
        };
        let sayı = değer
            .sayı()
            .filter(|sayı| {
                sayı.is_finite()
                    && *sayı >= 0.0
                    && sayı.fract() == 0.0
                    && *sayı <= usize::MAX as f64
            })
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "action.brush.axisIndex",
                ayrıntı: format!("{ad} negatif olmayan tam sayı olmalı"),
            })?;
        Ok(Some(sayı as usize))
    };
    Ok(FırçaSeçimAlanı {
        tür,
        koordinat_aralığı,
        x_ekseni_sırası: sıra("xAxisIndex")?,
        y_ekseni_sırası: sıra("yAxisIndex")?,
        dönüştürülebilir: nesne.get("transformable").and_then(EylemDeğeri::mantıksal),
    })
}

fn fırça_koordinat_uçlarını_oku(
    değer: &EylemDeğeri,
) -> Result<[FırçaKoordinatı; 2], BilesenHatasi> {
    let uçlar = değer
        .dizi()
        .filter(|uçlar| uçlar.len() == 2)
        .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
            alan: "action.brush.coordRange",
            ayrıntı: "aralık tam iki uç içermeli".to_owned(),
        })?;
    let uç = |değer: &EylemDeğeri| match değer {
        EylemDeğeri::Sayı(sayı) if sayı.is_finite() => Ok(FırçaKoordinatı::Sayı(*sayı)),
        EylemDeğeri::Metin(kategori) => Ok(FırçaKoordinatı::Kategori(kategori.clone())),
        _ => Err(BilesenHatasi::GeçersizSeçenek {
            alan: "action.brush.coordRange",
            ayrıntı: "uç sonlu sayı veya kategori adı olmalı".to_owned(),
        }),
    };
    Ok([uç(&uçlar[0])?, uç(&uçlar[1])?])
}

/// ECharts 6 kırık eksen action takımını kaydeder. Kırılmalar özgün
/// `start`/`end` çiftleriyle seçilir; `axisbreakchanged` olayı eski/yeni
/// `isExpanded` durumunu ve hedef eksen sırasını birlikte taşır.
pub fn eksen_kırılma_eylemlerini_kaydet(
    kayıt: &mut EylemKayıtDefteri,
) -> Result<(), BilesenHatasi> {
    for (eylem_türü, eylem) in [
        ("expandAxisBreak", EksenKırılmaEylemi::Genişlet),
        ("collapseAxisBreak", EksenKırılmaEylemi::Daralt),
        ("toggleAxisBreak", EksenKırılmaEylemi::Değiştir),
    ] {
        kayıt.kaydet(
            eylem_türü,
            "axisbreakchanged",
            EylemGüncellemesi::Tam,
            move |çalışma, yük| {
                for alan in [
                    "xAxisId",
                    "xAxisName",
                    "yAxisId",
                    "yAxisName",
                    "singleAxisId",
                    "singleAxisName",
                ] {
                    if yük.al(alan).is_some() {
                        return Err(BilesenHatasi::Desteklenmeyen {
                            özellik: "axisBreak id/name selector",
                            ayrıntı: format!(
                                "`{alan}` için eksen bileşen kimliği/adı modeli henüz yok; index kullanın"
                            ),
                        });
                    }
                }
                let kırılmalar = zorunlu_kırılma_tanımlayıcıları(yük)?;
                let x = isteğe_bağlı_sıralar(yük, "xAxisIndex")?;
                let y = isteğe_bağlı_sıralar(yük, "yAxisIndex")?;
                let tek = isteğe_bağlı_sıralar(yük, "singleAxisIndex")?;
                let açık_boyut = x.is_some() || y.is_some() || tek.is_some();
                let mut değişiklikler = Vec::new();
                for (boyut, sıralar) in [
                    (EksenBoyutu::X, x.as_deref()),
                    (EksenBoyutu::Y, y.as_deref()),
                    (EksenBoyutu::Tek, tek.as_deref()),
                ] {
                    if açık_boyut && sıralar.is_none() {
                        continue;
                    }
                    değişiklikler.extend(çalışma.eksen_kırılmalarını_ayarla(
                        boyut,
                        sıralar,
                        &kırılmalar,
                        eylem,
                        true,
                    )?);
                }

                let olay_kırılmaları = değişiklikler
                    .iter()
                    .map(eksen_kırılma_olay_değeri)
                    .collect::<Vec<_>>();
                Ok(vec![OlayYükü {
                    alanlar: BTreeMap::from([
                        ("fromAction".to_owned(), EylemDeğeri::from(eylem_türü)),
                        (
                            "breaks".to_owned(),
                            EylemDeğeri::Dizi(olay_kırılmaları),
                        ),
                    ]),
                    ..OlayYükü::default()
                }])
            },
        )?;
    }
    Ok(())
}

/// Sürekli ve parçalı `visualMap` için resmî `selectDataRange` action'ı.
pub fn görsel_aralık_eylemini_kaydet(
    kayıt: &mut EylemKayıtDefteri
) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "selectDataRange",
        "dataRangeSelected",
        EylemGüncellemesi::Tam,
        |çalışma, yük| {
            if yük.al("visualMapId").is_some() {
                return Err(BilesenHatasi::Desteklenmeyen {
                    özellik: "visualMapId",
                    ayrıntı: "Rust visualMap modelinde henüz bileşen kimliği bulunmuyor"
                        .to_owned(),
                });
            }
            let sıra = isteğe_bağlı_sıra(yük, "visualMapIndex")?;
            let sıra = sıra.unwrap_or(0);
            let selected = yük
                .al("selected")
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "visualMap.selected",
                    ayrıntı: "sürekli kipte iki sayılı dizi, parçalı kipte bool nesnesi gerekli"
                        .to_owned(),
                })?;
            let (alt_tür, olay_değeri) = if let Some(değerler) = selected.dizi() {
                let seçili = (değerler.len() == 2)
                    .then(|| Some([değerler.first()?.sayı()?, değerler.get(1)?.sayı()?]))
                    .flatten()
                    .filter(|değerler| değerler.iter().all(|değer| değer.is_finite()))
                    .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                        alan: "visualMap.selected",
                        ayrıntı: "iki sonlu sayıdan oluşan dizi gerekli".to_owned(),
                    })?;
                let seçili = çalışma.görsel_aralığı_ayarla(Some(sıra), seçili, true)?;
                (
                    "continuous",
                    EylemDeğeri::Dizi(seçili.into_iter().map(EylemDeğeri::from).collect()),
                )
            } else if let Some(nesne) = selected.nesne() {
                let mut seçili = BTreeMap::new();
                for (anahtar, değer) in nesne {
                    let parça =
                        anahtar
                            .parse::<usize>()
                            .map_err(|_| BilesenHatasi::GeçersizSeçenek {
                                alan: "visualMap.selected",
                                ayrıntı: format!("geçersiz parça anahtarı: {anahtar}"),
                            })?;
                    let açık =
                        değer
                            .mantıksal()
                            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                                alan: "visualMap.selected",
                                ayrıntı: format!("{anahtar} anahtarında bool değer gerekli"),
                            })?;
                    seçili.insert(parça, açık);
                }
                let seçili = çalışma.görsel_parçalarını_ayarla(Some(sıra), seçili, true)?;
                (
                    "piecewise",
                    EylemDeğeri::Nesne(
                        seçili
                            .into_iter()
                            .map(|(parça, açık)| (parça.to_string(), EylemDeğeri::from(açık)))
                            .collect(),
                    ),
                )
            } else {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "visualMap.selected",
                    ayrıntı: "sürekli kipte iki sayılı dizi, parçalı kipte bool nesnesi gerekli"
                        .to_owned(),
                });
            };
            Ok(vec![OlayYükü {
                bileşen_türü: Some("visualMap".to_owned()),
                bileşen_alt_türü: Some(alt_tür.to_owned()),
                bileşen_sırası: Some(sıra),
                alanlar: BTreeMap::from([("selected".to_owned(), olay_değeri)]),
                ..OlayYükü::default()
            }])
        },
    )
}

/// Toolbox `restore` action'ını kaydeder.
pub fn geri_yükleme_eylemini_kaydet(kayıt: &mut EylemKayıtDefteri) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "restore",
        "restore",
        EylemGüncellemesi::Tam,
        |çalışma, _| {
            çalışma.geri_yükle(true)?;
            Ok(vec![OlayYükü::default()])
        },
    )
}

/// Plain/scroll legend'in beş yerleşik action'ını kaydeder.
pub fn gösterge_eylemlerini_kaydet(kayıt: &mut EylemKayıtDefteri) -> Result<(), BilesenHatasi> {
    for (eylem_türü, olay_türü, işlem, ad_gerekli) in [
        (
            "legendToggleSelect",
            "legendselectchanged",
            GöstergeSeçimEylemi::Değiştir,
            true,
        ),
        (
            "legendAllSelect",
            "legendselectall",
            GöstergeSeçimEylemi::TümünüSeç,
            false,
        ),
        (
            "legendInverseSelect",
            "legendinverseselect",
            GöstergeSeçimEylemi::TersiniSeç,
            false,
        ),
        (
            "legendSelect",
            "legendselected",
            GöstergeSeçimEylemi::Seç,
            true,
        ),
        (
            "legendUnSelect",
            "legendunselected",
            GöstergeSeçimEylemi::SeçimiKaldır,
            true,
        ),
    ] {
        kayıt.kaydet(
            eylem_türü,
            olay_türü,
            EylemGüncellemesi::Tam,
            move |çalışma, yük| {
                let ad = yük.al("name").and_then(EylemDeğeri::metin);
                if ad_gerekli && ad.is_none() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "legend.action.name",
                        ayrıntı: "bu legend action için name gerekli".to_owned(),
                    });
                }
                let seçili = çalışma.gösterge_seçimini_ayarla(işlem, ad, true)?;
                let mut olay = OlayYükü {
                    bileşen_türü: Some("legend".to_owned()),
                    bileşen_sırası: Some(0),
                    ..OlayYükü::default()
                };
                if let Some(ad) = ad {
                    olay.alanlar
                        .insert("name".to_owned(), EylemDeğeri::from(ad));
                }
                olay.alanlar.insert(
                    "selected".to_owned(),
                    EylemDeğeri::Nesne(
                        seçili
                            .into_iter()
                            .map(|(ad, seçili)| (ad, EylemDeğeri::from(seçili)))
                            .collect(),
                    ),
                );
                if !ad_gerekli {
                    olay.alanlar.insert(
                        "legendIndex".to_owned(),
                        EylemDeğeri::Dizi(vec![EylemDeğeri::from(0usize)]),
                    );
                }
                Ok(vec![olay])
            },
        )?;
    }
    Ok(())
}

/// `updateAxisPointer` action'ı. Kategori ekseninde piksel koordinatını ham
/// kategori sırasına çözer ve ECharts olayındaki `axesInfo`/`seriesDataIndices`
/// yapısını yayınlar; dinleyici bu bilgiyi `setOption` ile bağlı serilere
/// uygulayabilir.
pub fn eksen_imleci_eylemini_kaydet(kayıt: &mut EylemKayıtDefteri) -> Result<(), BilesenHatasi> {
    kayıt.kaydet(
        "updateAxisPointer",
        "updateAxisPointer",
        EylemGüncellemesi::Görünüm,
        |çalışma, yük| {
            let x_sırası = isteğe_bağlı_sıra(yük, "xAxisIndex")?;
            let y_sırası = isteğe_bağlı_sıra(yük, "yAxisIndex")?;
            let x_mi = x_sırası.is_some() || y_sırası.is_none();
            let eksen_sırası = if x_mi {
                x_sırası.unwrap_or(0)
            } else {
                y_sırası.unwrap_or(0)
            };

            let seçenekler = çalışma.seçenekleri_al()?;
            let (seçenekler, hatalar) = seçenekler.veri_kümesini_uygula();
            if let Some(hata) = hatalar.into_iter().next() {
                return Err(hata);
            }
            let eksenler = if x_mi {
                seçenekler.etkin_x_eksenleri()
            } else {
                seçenekler.etkin_y_eksenleri()
            };
            let eksen = eksenler.get(eksen_sırası).ok_or(BilesenHatasi::EksikVeri {
                bileşen: if x_mi { "xAxis" } else { "yAxis" },
                sıra: eksen_sırası,
            })?;
            if eksen.tür != crate::model::eksen::EksenTürü::Kategori {
                return Err(BilesenHatasi::Desteklenmeyen {
                    özellik: "updateAxisPointer.valueAxis",
                    ayrıntı: "pikselden olay değeri çözümü şu anda kategori eksenini hedefler"
                        .to_owned(),
                });
            }

            let mut kategoriler = eksen.veri.clone();
            if kategoriler.is_empty() {
                for seri in &seçenekler.seriler {
                    if !seri.kartezyen_mi() {
                        continue;
                    }
                    let bağ = seri.eksen_bağı();
                    if (x_mi && bağ.x != eksen_sırası) || (!x_mi && bağ.y != eksen_sırası) {
                        continue;
                    }
                    for ad in seri.veri().iter().filter_map(|öğe| öğe.ad.as_ref()) {
                        if !kategoriler.contains(ad) {
                            kategoriler.push(ad.clone());
                        }
                    }
                }
            }
            if kategoriler.is_empty() {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "updateAxisPointer.axis",
                    ayrıntı: "kategori ekseninde çözülecek veri yok".to_owned(),
                });
            }

            let açık_sıra =
                isteğe_bağlı_sıra(yük, "value")?.or(isteğe_bağlı_sıra(yük, "dataIndex")?);
            let kategori_sırası = if let Some(sıra) = açık_sıra {
                sıra
            } else {
                let piksel_alanı = if x_mi { "x" } else { "y" };
                let piksel = isteğe_bağlı_sayı(yük, piksel_alanı)?.ok_or_else(|| {
                    BilesenHatasi::GeçersizSeçenek {
                        alan: "updateAxisPointer",
                        ayrıntı: format!("value/dataIndex veya {piksel_alanı} gerekli"),
                    }
                })? as f32;
                eksen_imleci_kategori_sırası(
                    &seçenekler,
                    çalışma.başlatma().genişlik,
                    çalışma.başlatma().yükseklik,
                    eksen,
                    piksel,
                    x_mi,
                    kategoriler.len(),
                )
            };
            let kategori = kategoriler.get(kategori_sırası).cloned().ok_or_else(|| {
                BilesenHatasi::GeçersizSeçenek {
                    alan: "updateAxisPointer.value",
                    ayrıntı: format!(
                        "{kategori_sırası} kategori sırası {} öğelik eksenin dışında",
                        kategoriler.len()
                    ),
                }
            })?;

            let seri_indeksleri = seçenekler
                .seriler
                .iter()
                .enumerate()
                .filter(|(_, seri)| {
                    if !seri.kartezyen_mi() || kategori_sırası >= seri.veri().len() {
                        return false;
                    }
                    let bağ = seri.eksen_bağı();
                    (x_mi && bağ.x == eksen_sırası) || (!x_mi && bağ.y == eksen_sırası)
                })
                .map(|(seri_sırası, _)| {
                    EylemDeğeri::Nesne(BTreeMap::from([
                        ("seriesIndex".to_owned(), EylemDeğeri::from(seri_sırası)),
                        ("dataIndex".to_owned(), EylemDeğeri::from(kategori_sırası)),
                    ]))
                })
                .collect();
            let eksen_bilgisi = EylemDeğeri::Nesne(BTreeMap::from([
                (
                    "axisDim".to_owned(),
                    EylemDeğeri::from(if x_mi { "x" } else { "y" }),
                ),
                ("axisIndex".to_owned(), EylemDeğeri::from(eksen_sırası)),
                ("value".to_owned(), EylemDeğeri::from(kategori_sırası)),
                ("valueLabel".to_owned(), EylemDeğeri::from(kategori)),
                (
                    "seriesDataIndices".to_owned(),
                    EylemDeğeri::Dizi(seri_indeksleri),
                ),
            ]));
            let mut olay = OlayYükü {
                bileşen_türü: Some("axisPointer".to_owned()),
                bileşen_sırası: Some(eksen_sırası),
                ..OlayYükü::default()
            };
            olay.alanlar.insert(
                "axesInfo".to_owned(),
                EylemDeğeri::Dizi(vec![eksen_bilgisi]),
            );
            for alan in ["x", "y", "currTrigger"] {
                if let Some(değer) = yük.al(alan) {
                    olay.alanlar.insert(alan.to_owned(), değer.clone());
                }
            }
            Ok(vec![olay])
        },
    )
}

fn eksen_imleci_kategori_sırası(
    seçenekler: &crate::model::secenekler::GrafikSeçenekleri,
    genişlik: f32,
    yükseklik: f32,
    eksen: &crate::model::eksen::Eksen,
    piksel: f32,
    x_mi: bool,
    kategori_sayısı: usize,
) -> usize {
    let ızgaralar = seçenekler.etkin_ızgaralar();
    let ızgara = ızgaralar
        .get(eksen.ızgara_sırası)
        .cloned()
        .unwrap_or_default();
    let sol = ızgara.sol.çöz(genişlik);
    let sağ = ızgara
        .genişlik
        .map(|uzunluk| sol + uzunluk.çöz(genişlik))
        .unwrap_or_else(|| genişlik - ızgara.sağ.çöz(genişlik));
    let üst = ızgara.üst.çöz(yükseklik);
    let alt = ızgara
        .yükseklik
        .map(|uzunluk| üst + uzunluk.çöz(yükseklik))
        .unwrap_or_else(|| yükseklik - ızgara.alt.çöz(yükseklik));
    let (başlangıç, uzunluk, konum) = if x_mi {
        (sol, (sağ - sol).max(1.0), piksel)
    } else {
        (üst, (alt - üst).max(1.0), alt - (piksel - üst))
    };
    let oran = ((konum - başlangıç) / uzunluk).clamp(0.0, 1.0);
    if eksen.kenar_boşluğu.unwrap_or(true) {
        ((oran * kategori_sayısı as f32).floor() as usize).min(kategori_sayısı - 1)
    } else if kategori_sayısı <= 1 {
        0
    } else {
        ((oran * (kategori_sayısı - 1) as f32).round() as usize).min(kategori_sayısı - 1)
    }
}

/// Bu çekirdekte hazır gelen action'ları tek seferde kaydeder. Kayıt defteri
/// bilinçli olarak boş başlar; böylece gömülü uygulamalar yalnız kullandığı
/// eylemleri ya da kendi `registerAction` karşılıklarını seçebilir.
pub fn öntanımlı_eylemleri_kaydet(kayıt: &mut EylemKayıtDefteri) -> Result<(), BilesenHatasi> {
    append_data_eylemini_kaydet(kayıt)?;
    veri_yakınlaştırma_eylemini_kaydet(kayıt)?;
    fırça_eylemini_kaydet(kayıt)?;
    paralel_eylemlerini_kaydet(kayıt)?;
    sankey_eylemlerini_kaydet(kayıt)?;
    ağaç_eylemlerini_kaydet(kayıt)?;
    ağaç_haritası_eylemlerini_kaydet(kayıt)?;
    güneş_patlaması_eylemlerini_kaydet(kayıt)?;
    eksen_kırılma_eylemlerini_kaydet(kayıt)?;
    görsel_aralık_eylemini_kaydet(kayıt)?;
    geri_yükleme_eylemini_kaydet(kayıt)?;
    gösterge_eylemlerini_kaydet(kayıt)?;
    eksen_imleci_eylemini_kaydet(kayıt)
}

fn isteğe_bağlı_sıralar(
    yük: &EylemYükü,
    alan: &'static str,
) -> Result<Option<Vec<usize>>, BilesenHatasi> {
    let Some(değer) = yük.al(alan) else {
        return Ok(None);
    };
    let değerler = değer
        .dizi()
        .map_or_else(|| vec![değer], |dizi| dizi.iter().collect());
    let mut sıralar = Vec::with_capacity(değerler.len());
    for değer in değerler {
        let sayı = değer
            .sayı()
            .filter(|sayı| {
                sayı.is_finite()
                    && *sayı >= 0.0
                    && sayı.fract() == 0.0
                    && *sayı <= usize::MAX as f64
            })
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan,
                ayrıntı: "negatif olmayan tam sayı veya bunların dizisi gerekli".to_owned(),
            })?;
        sıralar.push(sayı as usize);
    }
    Ok(Some(sıralar))
}

fn zorunlu_kırılma_tanımlayıcıları(
    yük: &EylemYükü,
) -> Result<Vec<(f64, f64)>, BilesenHatasi> {
    let değerler = yük
        .al("breaks")
        .and_then(EylemDeğeri::dizi)
        .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
            alan: "action.breaks",
            ayrıntı: "start/end nesnelerinden oluşan bir dizi gerekli".to_owned(),
        })?;
    let mut kırılmalar = Vec::with_capacity(değerler.len());
    for (sıra, değer) in değerler.iter().enumerate() {
        let nesne = değer
            .nesne()
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "action.breaks",
                ayrıntı: format!("{sıra}. öğe start/end nesnesi olmalı"),
            })?;
        let uç = |ad: &str| {
            nesne
                .get(ad)
                .and_then(EylemDeğeri::sayı)
                .filter(|değer| değer.is_finite())
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "action.breaks",
                    ayrıntı: format!("{sıra}. öğenin `{ad}` ucu sonlu sayı olmalı"),
                })
        };
        kırılmalar.push((uç("start")?, uç("end")?));
    }
    Ok(kırılmalar)
}

fn eksen_kırılma_olay_değeri(değişiklik: &EksenKırılmaDeğişikliği) -> EylemDeğeri {
    let sıra_alanı = match değişiklik.boyut {
        EksenBoyutu::X => "xAxisIndex",
        EksenBoyutu::Y => "yAxisIndex",
        EksenBoyutu::Tek => "singleAxisIndex",
    };
    EylemDeğeri::Nesne(BTreeMap::from([
        ("start".to_owned(), değişiklik.başlangıç.into()),
        ("end".to_owned(), değişiklik.bitiş.into()),
        ("isExpanded".to_owned(), değişiklik.genişletilmiş.into()),
        (
            "old".to_owned(),
            EylemDeğeri::Nesne(BTreeMap::from([(
                "isExpanded".to_owned(),
                değişiklik.eski_genişletilmiş.into(),
            )])),
        ),
        (sıra_alanı.to_owned(), değişiklik.eksen_sırası.into()),
    ]))
}

fn isteğe_bağlı_sayı(
    yük: &EylemYükü,
    alan: &'static str,
) -> Result<Option<f64>, BilesenHatasi> {
    let Some(değer) = yük.al(alan) else {
        return Ok(None);
    };
    değer
        .sayı()
        .filter(|değer| değer.is_finite())
        .map(Some)
        .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
            alan,
            ayrıntı: "sonlu bir sayı gerekli".to_owned(),
        })
}

fn isteğe_bağlı_yakınlaştırma_değeri(
    yük: &EylemYükü,
    alan: &'static str,
) -> Result<Option<YakınlaştırmaDeğeri>, BilesenHatasi> {
    let Some(değer) = yük.al(alan) else {
        return Ok(None);
    };
    match değer {
        EylemDeğeri::Sayı(sayı) if sayı.is_finite() => {
            Ok(Some(YakınlaştırmaDeğeri::Sayı(*sayı)))
        }
        EylemDeğeri::Metin(kategori) => {
            Ok(Some(YakınlaştırmaDeğeri::Kategori(kategori.clone())))
        }
        _ => Err(BilesenHatasi::GeçersizSeçenek {
            alan,
            ayrıntı: "sonlu sayı veya kategori adı gerekli".to_owned(),
        }),
    }
}

fn yakınlaştırma_değerinden_eylem(değer: YakınlaştırmaDeğeri) -> EylemDeğeri {
    match değer {
        YakınlaştırmaDeğeri::Sayı(sayı) => EylemDeğeri::Sayı(sayı),
        YakınlaştırmaDeğeri::Kategori(kategori) => EylemDeğeri::Metin(kategori),
    }
}

fn isteğe_bağlı_sıra(
    yük: &EylemYükü,
    alan: &'static str,
) -> Result<Option<usize>, BilesenHatasi> {
    let Some(değer) = isteğe_bağlı_sayı(yük, alan)? else {
        return Ok(None);
    };
    if değer < 0.0 || değer.fract() != 0.0 || değer > usize::MAX as f64 {
        return Err(BilesenHatasi::GeçersizSeçenek {
            alan,
            ayrıntı: format!("{değer} negatif olmayan tam bir sıra olmalı"),
        });
    }
    Ok(Some(değer as usize))
}

fn eylem_değerinden_veri(değer: &EylemDeğeri) -> Result<VeriÖğesi, BilesenHatasi> {
    match değer {
        EylemDeğeri::Boş => Ok(VeriÖğesi::yeni(VeriDeğeri::Boş)),
        EylemDeğeri::Mantıksal(değer) => Ok(VeriÖğesi::yeni(*değer)),
        EylemDeğeri::Sayı(değer) => Ok(VeriÖğesi::yeni(*değer)),
        EylemDeğeri::Metin(değer) => Ok(VeriÖğesi::yeni(değer.clone())),
        EylemDeğeri::Dizi(dizi) => {
            let sayılar: Option<Vec<f64>> = dizi.iter().map(EylemDeğeri::sayı).collect();
            sayılar
                .map(|sayılar| VeriÖğesi::yeni(VeriDeğeri::Dizi(sayılar)))
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "action.data",
                    ayrıntı: "iç veri dizisi yalnız sayılardan oluşmalı".to_owned(),
                })
        }
        EylemDeğeri::Nesne(nesne) => {
            let değer = nesne
                .get("value")
                .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "action.data.value",
                    ayrıntı: "nesne veri öğesinde value gerekli".to_owned(),
                })?;
            let mut öğe = eylem_değerinden_veri(değer)?;
            öğe.ad = nesne
                .get("name")
                .and_then(EylemDeğeri::metin)
                .map(str::to_owned);
            Ok(öğe)
        }
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
    use std::sync::Mutex;

    use super::*;
    use crate::calisma_zamani::ÖrnekBaşlatmaSeçenekleri;
    use crate::model::agac::AğaçDüğümü;
    use crate::model::bilesen::{Başlık, Fırça, Gösterge, GöstergeSeçimKipi, Izgara};
    use crate::model::eksen::{Eksen, EksenKırılması};
    use crate::model::gorsel_esleme::GörselEşleme;
    use crate::model::paralel::{ParalelEkseni, ParalelKoordinatı};
    use crate::model::secenekler::GrafikSeçenekleri;
    use crate::model::seri::{
        AğaçHaritasıSerisi, AğaçSerisi, GüneşPatlamasıSerisi, ParalelSerisi, SankeySerisi,
        ÇizgiSerisi,
    };
    use crate::model::yakinlastirma::VeriYakınlaştırma;

    fn çalışma() -> GrafikÇalışmaZamanı {
        GrafikÇalışmaZamanı::yeni(
            ÖrnekBaşlatmaSeçenekleri::default(),
            GrafikSeçenekleri::yeni().kimlikli_seri("s", ÇizgiSerisi::yeni().ad("Seri").veri([1])),
        )
        .unwrap()
    }

    #[test]
    fn append_data_action_batch_ve_silent() {
        let mut kayıt = EylemKayıtDefteri::yeni();
        append_data_eylemini_kaydet(&mut kayıt).unwrap();
        let mut çalışma = çalışma();
        let batch = EylemYükü::yeni("appendData").batch([
            EylemYükü::default()
                .alan("seriesId", "s")
                .alan("data", EylemDeğeri::Dizi(vec![2.into(), 3.into()])),
            EylemYükü::default()
                .alan("seriesId", "s")
                .alan("data", EylemDeğeri::Dizi(vec![4.into()]))
                .sessiz(true),
        ]);
        let olaylar = kayıt.gönder(&mut çalışma, &batch).unwrap();
        assert_eq!(olaylar.len(), 1);
        assert_eq!(olaylar[0].tür, "dataappended");
        assert_eq!(olaylar[0].alanlar.get("count"), Some(&2usize.into()));
        assert_eq!(çalışma.seçenekleri_al().unwrap().seriler[0].veri().len(), 4);
    }

    #[test]
    fn tree_expand_and_collapse_action_modeli_ve_olay_yukunu_gunceller() {
        let seçenekler = GrafikSeçenekleri::yeni().kimlikli_seri(
            "tree-id",
            AğaçSerisi::yeni()
                .ad("Tree")
                .ilk_ağaç_derinliği(-1)
                .kökler([AğaçDüğümü::dal(
                    "root",
                    vec![AğaçDüğümü::yaprak("leaf", 7.0)],
                )]),
        );
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        ağaç_eylemlerini_kaydet(&mut kayıt).unwrap();

        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("treeExpandAndCollapse")
                    .alan("seriesId", "tree-id")
                    .alan("dataIndex", 0usize),
            )
            .unwrap();
        assert_eq!(olaylar.len(), 1);
        assert_eq!(olaylar[0].tür, "treeExpandAndCollapse");
        assert_eq!(olaylar[0].bileşen_alt_türü.as_deref(), Some("tree"));
        assert_eq!(olaylar[0].seri_sırası, Some(0));
        assert_eq!(olaylar[0].veri_adı.as_deref(), Some("root"));
        assert_eq!(
            olaylar[0].alanlar.get("collapsed"),
            Some(&EylemDeğeri::Mantıksal(true))
        );
        let seçenekler = çalışma.seçenekleri_al().unwrap();
        let crate::model::seri::Seri::Ağaç(ağaç) = &seçenekler.seriler[0] else {
            panic!("Tree serisi bekleniyordu");
        };
        assert_eq!(ağaç.kökler[0].daraltılmış, Some(true));

        let sessiz = EylemYükü::yeni("treeExpandAndCollapse")
            .alan("seriesIndex", 0usize)
            .alan("dataIndex", 0usize)
            .sessiz(true);
        assert!(kayıt.gönder(&mut çalışma, &sessiz).unwrap().is_empty());
        let seçenekler = çalışma.seçenekleri_al().unwrap();
        let crate::model::seri::Seri::Ağaç(ağaç) = &seçenekler.seriler[0] else {
            panic!("Tree serisi bekleniyordu");
        };
        assert_eq!(ağaç.kökler[0].daraltılmış, Some(false));
    }

    #[test]
    fn sankey_drag_ve_roam_actionlari_modeli_ve_resmi_olay_yukunu_korur() {
        let seçenekler = GrafikSeçenekleri::yeni().seri(
            SankeySerisi::yeni()
                .kimlik("sk")
                .ad("Sankey")
                .düğümler(["A", "B"])
                .bağlar([("A", "B", 7.0)]),
        );
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        sankey_eylemlerini_kaydet(&mut kayıt).unwrap();

        let sürükleme = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("dragNode")
                    .alan("seriesId", "sk")
                    .alan("dataIndex", 0usize)
                    .alan("localX", 0.25_f64)
                    .alan("localY", 0.4_f64),
            )
            .unwrap();
        assert_eq!(sürükleme.len(), 1);
        assert_eq!(sürükleme[0].tür, "dragnode");
        assert_eq!(sürükleme[0].bileşen_alt_türü.as_deref(), Some("sankey"));
        assert_eq!(sürükleme[0].veri_adı.as_deref(), Some("A"));
        assert_eq!(sürükleme[0].alanlar["localX"].sayı(), Some(0.25));
        let seçenekler = çalışma.seçenekleri_al().unwrap();
        let crate::model::seri::Seri::Sankey(sankey) = &seçenekler.seriler[0] else {
            panic!("Sankey serisi bekleniyordu");
        };
        assert_eq!(sankey.düğümler[0].yerel_x, Some(0.25));
        assert_eq!(sankey.düğümler[0].yerel_y, Some(0.4));

        let kaydırma = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("sankeyRoam")
                    .alan("seriesId", "sk")
                    .alan("dx", 10.0_f64)
                    .alan("dy", -5.0_f64),
            )
            .unwrap();
        assert_eq!(kaydırma[0].tür, "sankeyRoam");
        assert_eq!(kaydırma[0].alanlar["dx"].sayı(), Some(10.0));
        assert_eq!(kaydırma[0].alanlar["currentZoom"].sayı(), Some(1.0));

        let yakınlaştırma = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("sankeyRoam")
                    .alan("seriesId", "sk")
                    .alan("zoom", 1.5_f64)
                    .alan("originX", 297.5_f64)
                    .alan("originY", 262.5_f64),
            )
            .unwrap();
        assert_eq!(yakınlaştırma[0].alanlar["currentZoom"].sayı(), Some(1.5));
        let seçenekler = çalışma.seçenekleri_al().unwrap();
        let crate::model::seri::Seri::Sankey(sankey) = &seçenekler.seriler[0] else {
            panic!("Sankey serisi bekleniyordu");
        };
        assert!((sankey.yakınlaştırma - 1.5).abs() < 1e-6);
        let merkez = sankey.merkez.expect("roam merkez üretmeli");
        assert!((merkez.0.çöz(525.0) - 252.5).abs() < 1e-6);
        assert!((merkez.1.çöz(472.5) - 241.25).abs() < 1e-6);

        assert!(
            kayıt
                .gönder(
                    &mut çalışma,
                    &EylemYükü::yeni("sankeyRoam")
                        .alan("seriesId", "sk")
                        .alan("dx", 1.0_f64),
                )
                .is_err()
        );
    }

    #[test]
    fn treemap_dort_view_actioni_kok_hedef_ve_root_rect_durumunu_korur() {
        let seçenekler = GrafikSeçenekleri::yeni().seri(
            AğaçHaritasıSerisi::yeni()
                .kimlik("tm")
                .ad("Treemap")
                .kökler([AğaçDüğümü::dal(
                    "root",
                    vec![
                        AğaçDüğümü::dal("branch", vec![AğaçDüğümü::yaprak("leaf", 7.0)])
                            .kimlik("branch-id"),
                    ],
                )
                .kimlik("root-id")]),
        );
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        ağaç_haritası_eylemlerini_kaydet(&mut kayıt).unwrap();

        let kök_olayı = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("treemapRootToNode")
                    .alan("seriesId", "tm")
                    .alan("targetNodeId", "branch-id"),
            )
            .unwrap();
        assert_eq!(kök_olayı[0].tür, "treemapRootToNode");
        assert_eq!(
            kök_olayı[0].alanlar.get("direction"),
            Some(&EylemDeğeri::from("drillDown"))
        );
        assert_eq!(
            çalışma
                .ağaç_haritası_görünümü(SeriSeçici::kimlik("tm"))
                .unwrap()
                .1,
            vec!["root".to_owned(), "branch".to_owned()]
        );

        let yakınlaştırma = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("treemapZoomToNode")
                    .alan("seriesId", "tm")
                    .alan("dataIndex", 2usize),
            )
            .unwrap();
        assert_eq!(yakınlaştırma[0].veri_adı.as_deref(), Some("leaf"));
        assert_eq!(
            çalışma
                .ağaç_haritası_görünümü(SeriSeçici::kimlik("tm"))
                .unwrap()
                .3,
            Some(2)
        );

        let root_rect = EylemDeğeri::Nesne(BTreeMap::from([
            ("x".to_owned(), (-20.0_f64).into()),
            ("y".to_owned(), (-10.0_f64).into()),
            ("width".to_owned(), 900.0_f64.into()),
            ("height".to_owned(), 650.0_f64.into()),
        ]));
        kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("treemapMove")
                    .alan("seriesId", "tm")
                    .alan("rootRect", root_rect),
            )
            .unwrap();
        let görünüm = çalışma
            .ağaç_haritası_görünümü(SeriSeçici::kimlik("tm"))
            .unwrap();
        assert_eq!(
            görünüm.2,
            Some(AğaçHaritasıKökDikdörtgeni::yeni(
                -20.0, -10.0, 900.0, 650.0
            ))
        );
        assert_eq!(görünüm.3, None);

        let geri = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("treemapRootToNode").alan("seriesId", "tm"),
            )
            .unwrap();
        assert_eq!(
            geri[0].alanlar.get("direction"),
            Some(&EylemDeğeri::from("rollUp"))
        );
        assert!(
            çalışma
                .ağaç_haritası_görünümü(SeriSeçici::kimlik("tm"))
                .unwrap()
                .1
                .is_empty()
        );
    }

    #[test]
    fn treemap_action_silent_olayi_bastirir_ama_durumu_gunceller() {
        let seçenekler =
            GrafikSeçenekleri::yeni().seri(AğaçHaritasıSerisi::yeni().kimlik("tm").kökler([
                AğaçDüğümü::dal("root", vec![AğaçDüğümü::yaprak("leaf", 1.0)]),
            ]));
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        ağaç_haritası_eylemlerini_kaydet(&mut kayıt).unwrap();
        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("treemapZoomToNode")
                    .alan("seriesId", "tm")
                    .alan("dataIndex", 1usize)
                    .sessiz(true),
            )
            .unwrap();
        assert!(olaylar.is_empty());
        assert_eq!(
            çalışma
                .ağaç_haritası_görünümü(SeriSeçici::kimlik("tm"))
                .unwrap()
                .3,
            Some(1)
        );
    }

    #[test]
    fn sunburst_root_ve_eski_vurgu_actionlari_resmi_yuku_korur() {
        let seçenekler = GrafikSeçenekleri::yeni().seri(
            GüneşPatlamasıSerisi::yeni()
                .kimlik("sun")
                .ad("Sunburst")
                .kökler([AğaçDüğümü::dal(
                    "root",
                    vec![AğaçDüğümü::yaprak("leaf", 7.0).kimlik("leaf-id")],
                )]),
        );
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        güneş_patlaması_eylemlerini_kaydet(&mut kayıt).unwrap();

        let aşağı = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("sunburstRootToNode")
                    .alan("seriesId", "sun")
                    .alan("targetNodeId", "leaf-id"),
            )
            .unwrap();
        assert_eq!(aşağı[0].bileşen_alt_türü.as_deref(), Some("sunburst"));
        assert_eq!(aşağı[0].veri_adı.as_deref(), Some("leaf"));
        assert_eq!(
            aşağı[0].alanlar.get("direction"),
            Some(&EylemDeğeri::from("drillDown"))
        );
        assert_eq!(
            çalışma
                .güneş_patlaması_görünümü(SeriSeçici::kimlik("sun"))
                .unwrap()
                .1,
            vec!["root".to_owned(), "leaf".to_owned()]
        );

        let eski = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("sunburstHighlight")
                    .alan("seriesId", "sun")
                    .alan("dataIndex", 1usize),
            )
            .unwrap();
        assert_eq!(
            eski[0].alanlar.get("forwardedType"),
            Some(&EylemDeğeri::from("highlight"))
        );

        let yukarı = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("sunburstRootToNode").alan("seriesId", "sun"),
            )
            .unwrap();
        assert_eq!(
            yukarı[0].alanlar.get("direction"),
            Some(&EylemDeğeri::from("rollUp"))
        );
        assert!(
            çalışma
                .güneş_patlaması_görünümü(SeriSeçici::kimlik("sun"))
                .unwrap()
                .1
                .is_empty()
        );
    }

    #[test]
    fn data_zoom_action_bagli_bilesenleri_gunceller_ve_silent_olayi_bastirir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .x_ekseni_ekle(Eksen::kategori().veri(["A", "B"]))
            .x_ekseni_ekle(Eksen::kategori().veri(["A", "B"]))
            .y_ekseni(Eksen::değer())
            .seri(ÇizgiSerisi::yeni().veri([1, 2]))
            .veri_yakınlaştırma(VeriYakınlaştırma::iç().x_eksenleri([0, 1]))
            // Hedef dizi sırası bağlantı kimliğini değiştirmez.
            .veri_yakınlaştırma(VeriYakınlaştırma::sürgü().x_eksenleri([1, 0]))
            .veri_yakınlaştırma(VeriYakınlaştırma::iç().y_eksen_sırası(0));
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        veri_yakınlaştırma_eylemini_kaydet(&mut kayıt).unwrap();

        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("dataZoom")
                    .alan("dataZoomIndex", 0usize)
                    .alan("start", 20.0f64)
                    .alan("end", 70.0f64),
            )
            .unwrap();
        assert_eq!(olaylar.len(), 1);
        assert_eq!(olaylar[0].tür, "datazoom");
        assert_eq!(olaylar[0].bileşen_sırası, Some(0));
        assert_eq!(
            olaylar[0].alanlar.get("affectedIndices"),
            Some(&EylemDeğeri::Dizi(vec![0usize.into(), 1usize.into()]))
        );
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert_eq!(sonuç.veri_yakınlaştırmaları[0].oranlar(), (0.2, 0.7));
        assert_eq!(sonuç.veri_yakınlaştırmaları[1].oranlar(), (0.2, 0.7));
        assert_eq!(sonuç.veri_yakınlaştırmaları[2].oranlar(), (0.0, 1.0));

        let sessiz = EylemYükü::yeni("dataZoom")
            .alan("dataZoomIndex", 2usize)
            .alan("start", 10.0f64)
            .sessiz(true);
        assert!(kayıt.gönder(&mut çalışma, &sessiz).unwrap().is_empty());
        assert_eq!(
            çalışma.seçenekleri_al().unwrap().veri_yakınlaştırmaları[2].başlangıç,
            10.0
        );
    }

    #[test]
    fn data_zoom_action_kategori_deger_uclarini_uygular_ve_yuzdeyle_temizler() {
        let kategoriler = ["A", "B", "C", "D", "E", "F", "G"];
        let seçenekler = GrafikSeçenekleri::yeni()
            .x_ekseni(Eksen::kategori().veri(kategoriler))
            .y_ekseni(Eksen::değer())
            .seri(ÇizgiSerisi::yeni().veri([1, 2, 3, 4, 5, 6, 7]))
            .veri_yakınlaştırma(VeriYakınlaştırma::iç())
            .veri_yakınlaştırma(VeriYakınlaştırma::sürgü());
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        veri_yakınlaştırma_eylemini_kaydet(&mut kayıt).unwrap();

        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("dataZoom")
                    .alan("dataZoomIndex", 0usize)
                    .alan("startValue", "C")
                    .alan("endValue", "F"),
            )
            .unwrap();
        assert_eq!(olaylar[0].alanlar.get("startValue"), Some(&"C".into()));
        assert_eq!(olaylar[0].alanlar.get("endValue"), Some(&"F".into()));
        assert_eq!(
            olaylar[0].alanlar.get("affectedIndices"),
            Some(&EylemDeğeri::Dizi(vec![0usize.into(), 1usize.into()]))
        );
        let sonuç = çalışma.seçenekleri_al().unwrap();
        for yakınlaştırma in &sonuç.veri_yakınlaştırmaları {
            assert_eq!(
                yakınlaştırma.başlangıç_değeri,
                Some(YakınlaştırmaDeğeri::Kategori("C".to_owned()))
            );
            assert_eq!(
                yakınlaştırma.bitiş_değeri,
                Some(YakınlaştırmaDeğeri::Kategori("F".to_owned()))
            );
        }

        kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("dataZoom")
                    .alan("dataZoomIndex", 0usize)
                    .alan("start", 25.0),
            )
            .unwrap();
        let sonuç = çalışma.seçenekleri_al().unwrap();
        for yakınlaştırma in &sonuç.veri_yakınlaştırmaları {
            assert_eq!(yakınlaştırma.başlangıç, 25.0);
            assert_eq!(yakınlaştırma.başlangıç_değeri, None);
            assert_eq!(
                yakınlaştırma.bitiş_değeri,
                Some(YakınlaştırmaDeğeri::Kategori("F".to_owned()))
            );
        }
    }

    #[test]
    fn firca_action_line_x_kategori_araligini_korur_ve_temizler() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .x_ekseni(Eksen::kategori().veri(["2024-01-01", "2024-01-02", "2024-01-03"]))
            .y_ekseni(Eksen::değer())
            .fırça(Fırça::default())
            .seri(ÇizgiSerisi::yeni().veri([1, 2, 3]));
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        fırça_eylemini_kaydet(&mut kayıt).unwrap();

        let alan = EylemDeğeri::Nesne(BTreeMap::from([
            ("brushType".to_owned(), "lineX".into()),
            (
                "coordRange".to_owned(),
                EylemDeğeri::Dizi(vec!["2024-01-02".into(), "2024-01-03".into()]),
            ),
            ("xAxisIndex".to_owned(), 0usize.into()),
        ]));
        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("brush").alan("areas", EylemDeğeri::Dizi(vec![alan.clone()])),
            )
            .unwrap();
        assert_eq!(olaylar.len(), 1);
        assert_eq!(olaylar[0].tür, "brush");
        assert_eq!(olaylar[0].bileşen_türü.as_deref(), Some("brush"));
        assert_eq!(olaylar[0].alanlar.get("changed"), Some(&true.into()));

        let seçenekler = çalışma.seçenekleri_al().unwrap();
        let fırça = seçenekler.fırça.as_ref().unwrap();
        assert_eq!(fırça.alanlar.len(), 1);
        assert_eq!(fırça.alanlar[0].tür, FırçaTürü::Yatay);
        assert_eq!(fırça.alanlar[0].x_ekseni_sırası, Some(0));
        assert_eq!(
            fırça.alanlar[0].koordinat_aralığı,
            FırçaKoordinatAralığı::Eksen([
                FırçaKoordinatı::Kategori("2024-01-02".to_owned()),
                FırçaKoordinatı::Kategori("2024-01-03".to_owned()),
            ])
        );

        kayıt
            .gönder(&mut çalışma, &EylemYükü::yeni("brush"))
            .unwrap();
        assert_eq!(
            çalışma
                .seçenekleri_al()
                .unwrap()
                .fırça
                .as_ref()
                .unwrap()
                .alanlar
                .len(),
            1,
            "areas verilmezse mevcut seçim korunmalı"
        );
        kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("brush").alan("areas", EylemDeğeri::Dizi(Vec::new())),
            )
            .unwrap();
        assert!(
            çalışma
                .seçenekleri_al()
                .unwrap()
                .fırça
                .as_ref()
                .unwrap()
                .alanlar
                .is_empty()
        );
    }

    #[test]
    fn axis_break_actionlari_modeli_ve_refined_olayi_gunceller() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .x_ekseni(Eksen::kategori().veri(["A"]))
            .y_ekseni(Eksen::değer().kırılma(EksenKırılması::yeni(5_000.0, 100_000.0).boşluk("2%")))
            .seri(ÇizgiSerisi::yeni().veri([1]));
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        eksen_kırılma_eylemlerini_kaydet(&mut kayıt).unwrap();
        let kırılmalar = EylemDeğeri::Dizi(vec![EylemDeğeri::Nesne(BTreeMap::from([
            ("start".to_owned(), 5_000.0f64.into()),
            ("end".to_owned(), 100_000.0f64.into()),
        ]))]);

        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("expandAxisBreak")
                    .alan("yAxisIndex", 0usize)
                    .alan("breaks", kırılmalar.clone()),
            )
            .unwrap();
        assert_eq!(olaylar.len(), 1);
        assert_eq!(olaylar[0].tür, "axisbreakchanged");
        assert_eq!(
            olaylar[0]
                .alanlar
                .get("fromAction")
                .and_then(EylemDeğeri::metin),
            Some("expandAxisBreak")
        );
        let değişiklik = olaylar[0].alanlar["breaks"].dizi().unwrap()[0]
            .nesne()
            .unwrap();
        assert_eq!(değişiklik["yAxisIndex"].sayı(), Some(0.0));
        assert_eq!(değişiklik["isExpanded"].mantıksal(), Some(true));
        assert_eq!(
            değişiklik["old"].nesne().unwrap()["isExpanded"].mantıksal(),
            Some(false)
        );
        assert!(
            çalışma
                .seçenekleri_al()
                .unwrap()
                .y_ekseni
                .unwrap()
                .kırılmalar[0]
                .genişletilmiş
        );

        kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("toggleAxisBreak")
                    .alan("yAxisIndex", EylemDeğeri::Dizi(vec![0usize.into()]))
                    .alan("breaks", kırılmalar),
            )
            .unwrap();
        assert!(
            !çalışma
                .seçenekleri_al()
                .unwrap()
                .y_ekseni
                .unwrap()
                .kırılmalar[0]
                .genişletilmiş
        );
    }

    #[test]
    fn select_data_range_sürekli_görsel_eşleme_aralığını_günceller() {
        let seçenekler =
            GrafikSeçenekleri::yeni().görsel_eşleme(GörselEşleme::yeni().en_az(0.0).en_çok(10.0));
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        görsel_aralık_eylemini_kaydet(&mut kayıt).unwrap();

        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("selectDataRange").alan(
                    "selected",
                    EylemDeğeri::Dizi(vec![7.0f64.into(), 3.0f64.into()]),
                ),
            )
            .unwrap();
        assert_eq!(olaylar.len(), 1);
        assert_eq!(olaylar[0].tür, "dataRangeSelected");
        assert_eq!(olaylar[0].bileşen_türü.as_deref(), Some("visualMap"));
        assert_eq!(
            çalışma
                .seçenekleri_al()
                .unwrap()
                .görsel_eşleme
                .unwrap()
                .seçili_aralık,
            Some([3.0, 7.0])
        );
    }

    #[test]
    fn select_data_range_parcali_gorsel_esleme_secim_nesnesini_gunceller() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .görsel_eşleme(GörselEşleme::yeni().en_az(0.0).en_çok(1.0).bölme_sayısı(3));
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        görsel_aralık_eylemini_kaydet(&mut kayıt).unwrap();
        let selected = EylemDeğeri::Nesne(BTreeMap::from([
            ("0".to_owned(), true.into()),
            ("1".to_owned(), false.into()),
            ("2".to_owned(), true.into()),
        ]));

        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("selectDataRange").alan("selected", selected),
            )
            .unwrap();

        assert_eq!(olaylar[0].tür, "dataRangeSelected");
        assert_eq!(olaylar[0].bileşen_alt_türü.as_deref(), Some("piecewise"));
        assert_eq!(
            çalışma
                .seçenekleri_al()
                .unwrap()
                .görsel_eşleme
                .unwrap()
                .kapalı_parçalar,
            vec![1]
        );
        assert_eq!(
            olaylar[0]
                .alanlar
                .get("selected")
                .and_then(EylemDeğeri::nesne)
                .and_then(|seçili| seçili.get("1"))
                .and_then(EylemDeğeri::mantıksal),
            Some(false)
        );
    }

    #[test]
    fn parallel_actionlari_axis_araliklarini_ve_genisletme_penceresini_gunceller() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .paralel(
                ParalelKoordinatı::yeni()
                    .kimlik("p")
                    .eksen_genişletilebilir(true)
                    .eksen_genişletme_sayısı(2),
            )
            .paralel_eksenleri([
                ParalelEkseni::yeni(0).kimlik("a"),
                ParalelEkseni::yeni(1),
                ParalelEkseni::yeni(2),
                ParalelEkseni::yeni(3),
            ])
            .seri(
                ParalelSerisi::yeni()
                    .boyutlar(["A", "B", "C", "D"])
                    .veri([vec![1.0, 2.0, 3.0, 4.0]]),
            );
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        paralel_eylemlerini_kaydet(&mut kayıt).unwrap();

        let aralıklar = EylemDeğeri::Dizi(vec![
            EylemDeğeri::Dizi(vec![20.0.into(), 10.0.into()]),
            EylemDeğeri::Dizi(vec![30.0.into(), 40.0.into()]),
        ]);
        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("axisAreaSelect")
                    .alan("parallelAxisId", "a")
                    .alan("intervals", aralıklar),
            )
            .unwrap();
        assert_eq!(olaylar[0].tür, "axisAreaSelected");
        assert_eq!(
            çalışma.seçenekleri_al().unwrap().paralel_eksenleri[0].etkin_aralıklar,
            vec![[10.0, 20.0], [30.0, 40.0]]
        );

        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("parallelAxisExpand")
                    .alan("parallelId", "p")
                    .alan(
                        "axisExpandWindow",
                        EylemDeğeri::Dizi(vec![50.0.into(), 150.0.into()]),
                    ),
            )
            .unwrap();
        assert_eq!(olaylar[0].tür, "parallelAxisExpand");
        assert_eq!(
            çalışma
                .seçenekleri_al()
                .unwrap()
                .paralel
                .unwrap()
                .eksen_genişletme_penceresi,
            Some([50.0, 150.0])
        );
    }

    #[test]
    fn update_axis_pointer_pikseli_kategori_sirasina_ve_axes_infoya_cevirir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .ızgara(Izgara::yeni().sol(100).genişlik(200))
            .x_ekseni(Eksen::kategori().veri(["2012", "2013", "2014", "2015", "2016", "2017"]))
            .y_ekseni(Eksen::değer())
            .seri(ÇizgiSerisi::yeni().veri([1, 2, 3, 4, 5, 6]));
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        eksen_imleci_eylemini_kaydet(&mut kayıt).unwrap();

        // Açık grid.width ile [100, 300] alanındaki üçüncü bandın merkezi.
        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("updateAxisPointer")
                    .alan("x", 183.333_333)
                    .alan("y", 400.0)
                    .alan("currTrigger", "mousemove"),
            )
            .unwrap();

        assert_eq!(olaylar.len(), 1);
        assert_eq!(olaylar[0].tür, "updateAxisPointer");
        let eksenler = olaylar[0].alanlar["axesInfo"].dizi().unwrap();
        let bilgi = eksenler[0].nesne().unwrap();
        assert_eq!(bilgi["axisDim"].metin(), Some("x"));
        assert_eq!(bilgi["value"].sayı(), Some(2.0));
        assert_eq!(bilgi["valueLabel"].metin(), Some("2014"));
        assert_eq!(bilgi["seriesDataIndices"].dizi().unwrap().len(), 1);
    }

    #[test]
    fn restore_ilk_base_option_yedegini_yeniden_kurar() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .başlık(Başlık::yeni().metin("İlk"))
            .seri(ÇizgiSerisi::yeni().veri([1]))
            .veri_yakınlaştırma(VeriYakınlaştırma::iç().aralık(10.0, 90.0));
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), başlangıç).unwrap();
        çalışma
            .seçenekleri_ayarla(
                crate::calisma_zamani::SeçenekYaması::yeni()
                    .başlık(Başlık::yeni().metin("Değişti")),
                crate::calisma_zamani::SeçenekAyarlamaKipi::default(),
            )
            .unwrap();
        çalışma
            .veri_yakınlaştırmayı_ayarla(Some(0), Some(30.0), Some(60.0), true)
            .unwrap();

        let mut kayıt = EylemKayıtDefteri::yeni();
        geri_yükleme_eylemini_kaydet(&mut kayıt).unwrap();
        let olaylar = kayıt
            .gönder(&mut çalışma, &EylemYükü::yeni("restore"))
            .unwrap();
        assert_eq!(olaylar[0].tür, "restore");
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert_eq!(
            sonuç
                .başlık
                .as_ref()
                .and_then(|başlık| başlık.metin.as_deref()),
            Some("İlk")
        );
        assert_eq!(sonuç.veri_yakınlaştırmaları[0].oranlar(), (0.1, 0.9));
    }

    #[test]
    fn legend_actionlari_selected_mode_batch_ve_event_yukunu_korur() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .gösterge(Gösterge::yeni().seçim_kipi(GöstergeSeçimKipi::Tek))
            .seri(ÇizgiSerisi::yeni().ad("A").veri([1]))
            .seri(ÇizgiSerisi::yeni().ad("B").veri([2]));
        let mut çalışma =
            GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap();
        let mut kayıt = EylemKayıtDefteri::yeni();
        gösterge_eylemlerini_kaydet(&mut kayıt).unwrap();

        let olaylar = kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("legendSelect").alan("name", "B"),
            )
            .unwrap();
        assert_eq!(olaylar[0].tür, "legendselected");
        assert_eq!(
            olaylar[0].alanlar.get("selected"),
            Some(&EylemDeğeri::Nesne(BTreeMap::from([
                ("A".to_owned(), false.into()),
                ("B".to_owned(), true.into()),
            ])))
        );
        let gösterge = çalışma.seçenekleri_al().unwrap().gösterge.unwrap();
        assert!(!gösterge.seçili_mi("A"));
        assert!(gösterge.seçili_mi("B"));

        // Tekli kipte seçili öğeyi toggle etmek kapatmaz.
        kayıt
            .gönder(
                &mut çalışma,
                &EylemYükü::yeni("legendToggleSelect").alan("name", "B"),
            )
            .unwrap();
        assert!(
            çalışma
                .seçenekleri_al()
                .unwrap()
                .gösterge
                .unwrap()
                .seçili_mi("B")
        );

        let ters = kayıt
            .gönder(&mut çalışma, &EylemYükü::yeni("legendInverseSelect"))
            .unwrap();
        assert_eq!(ters[0].tür, "legendinverseselect");
        assert_eq!(
            ters[0].alanlar.get("legendIndex"),
            Some(&EylemDeğeri::Dizi(vec![0usize.into()]))
        );
    }

    #[test]
    fn olay_sorgusu_yalniz_eslesen_dinleyiciyi_cagirir() {
        let sayım = Arc::new(Mutex::new(0usize));
        let mut kayıt = OlayKayıtDefteri::default();
        let hedef = sayım.clone();
        let kimlik = kayıt.dinle(
            "click",
            OlaySorgusu::default().seri_kimliği("s").veri_sırası(2),
            move |_| {
                if let Ok(mut sayım) = hedef.lock() {
                    *sayım += 1;
                }
            },
        );
        let mut olay = OlayYükü::yeni("click");
        olay.seri_kimliği = Some("s".to_owned());
        olay.veri_sırası = Some(1);
        assert_eq!(kayıt.yayınla(&olay), 0);
        olay.veri_sırası = Some(2);
        assert_eq!(kayıt.yayınla(&olay), 1);
        assert_eq!(*sayım.lock().unwrap(), 1);
        assert!(kayıt.bırak(kimlik));
        assert_eq!(kayıt.yayınla(&olay), 0);
    }

    #[test]
    fn connected_group_kaynagi_haric_tum_hedefleri_verir() {
        let mut bağlı = BağlıGrafikler::default();
        bağlı.bağla("a", "g");
        bağlı.bağla("b", "g");
        bağlı.bağla("c", "g");
        assert_eq!(bağlı.hedefler("b"), vec!["a", "c"]);
        bağlı.bağla("c", "başka");
        assert_eq!(bağlı.hedefler("b"), vec!["a"]);
        assert!(bağlı.ayır("a"));
        assert!(bağlı.hedefler("b").is_empty());
    }

    #[test]
    fn bilinmeyen_ve_yinelenen_action_tipli_hatadir() {
        let mut kayıt = EylemKayıtDefteri::yeni();
        kayıt
            .kaydet("x", "x", EylemGüncellemesi::Yok, |_, _| Ok(vec![]))
            .unwrap();
        assert!(
            kayıt
                .kaydet("x", "x", EylemGüncellemesi::Yok, |_, _| Ok(vec![]))
                .is_err()
        );
        assert!(
            kayıt
                .gönder(&mut çalışma(), &EylemYükü::yeni("yok"))
                .is_err()
        );
    }
}
