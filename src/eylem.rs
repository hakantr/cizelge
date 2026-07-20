//! `dispatchAction`, action kayıt defteri, olay sorguları ve bağlı grafikler.
//!
//! Bu modül gpui olaylarından bağımsızdır; başsız etkileşim senaryoları da
//! aynı yükleri ve sorgu eşlemesini kullanır.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::calisma_zamani::{
    EksenBoyutu, EksenKırılmaDeğişikliği, EksenKırılmaEylemi, GrafikÇalışmaZamanı,
    GöstergeSeçimEylemi, SeriSeçici,
};
use crate::hata::BilesenHatasi;
use crate::model::deger::{VeriDeğeri, VeriÖğesi};

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
            if yük.al("startValue").is_some() || yük.al("endValue").is_some() {
                return Err(BilesenHatasi::Desteklenmeyen {
                    özellik: "dataZoom.startValue/endValue",
                    ayrıntı: "bu model şu anda yüzde tabanlı start/end aralığı kullanıyor"
                        .to_owned(),
                });
            }
            let sıra = isteğe_bağlı_sıra(yük, "dataZoomIndex")?;
            let başlangıç = isteğe_bağlı_sayı(yük, "start")?.map(|değer| değer as f32);
            let bitiş = isteğe_bağlı_sayı(yük, "end")?.map(|değer| değer as f32);
            let değişiklikler =
                çalışma.veri_yakınlaştırmayı_ayarla(sıra, başlangıç, bitiş, true)?;

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
    use crate::model::bilesen::{Başlık, Gösterge, GöstergeSeçimKipi, Izgara};
    use crate::model::eksen::{Eksen, EksenKırılması};
    use crate::model::gorsel_esleme::GörselEşleme;
    use crate::model::secenekler::GrafikSeçenekleri;
    use crate::model::seri::ÇizgiSerisi;
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
