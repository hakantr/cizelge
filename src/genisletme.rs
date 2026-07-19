//! `echarts.use` ve `register*` genişletme yüzeylerinin Rust karşılığı.
//!
//! Paketler bağımlılıklarına göre bir kez kurulur. Kurulum bağlamı option
//! ön işleyicileri, scheduler görevleri, dataset dönüşümleri, action'lar,
//! loading çizicileri ve Geo dışı koordinat sistemi üreticilerini aynı yaşam
//! döngüsünde toplar. Böylece statik Rust kaydı kullanılsa da ECharts'ın
//! `preprocessor -> processor -> layout -> visual -> render` sırası korunur.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::cizim::Sahne;
use crate::eylem::EylemKayıtDefteri;
use crate::hata::BilesenHatasi;
use crate::koordinat::Dikdörtgen;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::veri_kumesi::{DönüşümKayıtDefteri, VeriDönüşümü};
use crate::zamanlayici::{ArtımlıGörev, Zamanlayıcı};

/// `registerPreprocessor` karşılığı. Bağımlılıklar yalnız diğer option ön
/// işleyicilerini adlandırır ve kayıt sırasından bağımsız çözülür.
pub trait SeçenekÖnİşleyicisi: Send {
    fn kimlik(&self) -> &str;

    fn bağımlılıklar(&self) -> &[String] {
        &[]
    }

    fn işle(&mut self, seçenekler: &mut GrafikSeçenekleri) -> Result<(), BilesenHatasi>;
}

/// Option ön işleyicilerini atomik bir kopya üzerinde bağımlılık sırasıyla
/// çalıştırır. Bir işleyici hata verirse çağıranın etkin option'ı değişmez.
#[derive(Default)]
pub struct ÖnİşlemeKayıtDefteri {
    işleyiciler: Vec<Box<dyn SeçenekÖnİşleyicisi>>,
    sıra: Option<Vec<usize>>,
}

impl ÖnİşlemeKayıtDefteri {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn kaydet(
        &mut self,
        işleyici: impl SeçenekÖnİşleyicisi + 'static,
    ) -> Result<(), BilesenHatasi> {
        let kimlik = işleyici.kimlik().trim();
        if kimlik.is_empty() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "preprocessor.id",
                ayrıntı: "ön işleyici kimliği boş olamaz".to_owned(),
            });
        }
        if self
            .işleyiciler
            .iter()
            .any(|kayıt| kayıt.kimlik() == kimlik)
        {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "preprocessor.id",
                ayrıntı: format!("`{kimlik}` ön işleyicisi zaten kayıtlı"),
            });
        }
        self.işleyiciler.push(Box::new(işleyici));
        self.sıra = None;
        Ok(())
    }

    pub fn uygula(&mut self, seçenekler: &mut GrafikSeçenekleri) -> Result<(), BilesenHatasi> {
        if self.sıra.is_none() {
            let kimlikler: Vec<String> = self
                .işleyiciler
                .iter()
                .map(|işleyici| işleyici.kimlik().to_owned())
                .collect();
            let bağımlılıklar: Vec<Vec<String>> = self
                .işleyiciler
                .iter()
                .map(|işleyici| işleyici.bağımlılıklar().to_vec())
                .collect();
            self.sıra = Some(bağımlılık_sırası(
                &kimlikler,
                &bağımlılıklar,
                "preprocessor.dependencies",
            )?);
        }

        let mut aday = seçenekler.clone();
        let sıra = self.sıra.clone().unwrap_or_default();
        for sıra in sıra {
            let işleyici = self
                .işleyiciler
                .get_mut(sıra)
                .ok_or(BilesenHatasi::EksikVeri {
                    bileşen: "preprocessor",
                    sıra,
                })?;
            işleyici.işle(&mut aday)?;
        }
        aday.doğrula()?;
        *seçenekler = aday;
        Ok(())
    }
}

/// Loading uzantısına verilen renderer-bağımsız durum.
#[derive(Clone, Debug, PartialEq)]
pub struct YüklemeGirdisi {
    pub genişlik: f32,
    pub yükseklik: f32,
    pub metin: Option<String>,
    pub ilerleme: Option<f32>,
}

/// `registerLoading` karşılığı. Üretilen sahne Kayıt/Piksel/SVG/gpui
/// yüzeylerinden herhangi birine gönderilebilir.
pub trait YüklemeÇizicisi: Send + Sync {
    fn tür_adı(&self) -> &str;

    fn sahne(&self, girdi: &YüklemeGirdisi) -> Result<Sahne, BilesenHatasi>;
}

#[derive(Default)]
pub struct YüklemeKayıtDefteri {
    çiziciler: BTreeMap<String, Arc<dyn YüklemeÇizicisi>>,
}

impl YüklemeKayıtDefteri {
    pub fn kaydet(
        &mut self, çizici: impl YüklemeÇizicisi + 'static
    ) -> Result<(), BilesenHatasi> {
        let tür = çizici.tür_adı().trim().to_owned();
        kayıt_adını_doğrula(&tür, "loading.type")?;
        if self.çiziciler.contains_key(&tür) {
            return Err(yinelenen_kayıt("loading.type", &tür));
        }
        self.çiziciler.insert(tür, Arc::new(çizici));
        Ok(())
    }

    pub fn sahne(&self, tür: &str, girdi: &YüklemeGirdisi) -> Result<Sahne, BilesenHatasi> {
        boyutları_doğrula(girdi.genişlik, girdi.yükseklik, "loading.size")?;
        if let Some(ilerleme) = girdi.ilerleme
            && (!ilerleme.is_finite() || !(0.0..=1.0).contains(&ilerleme))
        {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "loading.progress",
                ayrıntı: "ilerleme 0..=1 aralığında sonlu olmalı".to_owned(),
            });
        }
        self.çiziciler
            .get(tür)
            .ok_or_else(|| kayıtlı_değil("loading.type", tür))?
            .sahne(girdi)
    }
}

/// Özel, Geo dışı coordinate-system örneği.
pub trait ÖzelKoordinatSistemi: Send + Sync {
    fn veriden_piksele(&self, değer: [f64; 2]) -> Result<[f32; 2], BilesenHatasi>;

    fn pikselden_veriye(&self, piksel: [f32; 2]) -> Result<[f64; 2], BilesenHatasi>;

    fn piksel_içeriyor_mu(&self, piksel: [f32; 2]) -> bool;
}

/// `registerCoordinateSystem(type, creator)` karşılığı.
pub trait KoordinatSistemiÜreticisi: Send + Sync {
    fn tür_adı(&self) -> &str;

    fn oluştur(
        &self,
        seçenekler: &GrafikSeçenekleri,
        alan: Dikdörtgen,
    ) -> Result<Box<dyn ÖzelKoordinatSistemi>, BilesenHatasi>;
}

#[derive(Default)]
pub struct KoordinatSistemiKayıtDefteri {
    üreticiler: BTreeMap<String, Arc<dyn KoordinatSistemiÜreticisi>>,
}

impl KoordinatSistemiKayıtDefteri {
    pub fn kaydet(
        &mut self,
        üretici: impl KoordinatSistemiÜreticisi + 'static,
    ) -> Result<(), BilesenHatasi> {
        let tür = üretici.tür_adı().trim().to_owned();
        kayıt_adını_doğrula(&tür, "coordinateSystem.type")?;
        let küçük = tür.to_ascii_lowercase();
        if küçük == "geo" || küçük == "map" || küçük.ends_with("gl") {
            return Err(BilesenHatasi::Desteklenmeyen {
                özellik: "coordinateSystem",
                ayrıntı: format!("`{tür}` bu reponun Geo/Map/GL dışı kapsamının dışında"),
            });
        }
        if self.üreticiler.contains_key(&tür) {
            return Err(yinelenen_kayıt("coordinateSystem.type", &tür));
        }
        self.üreticiler.insert(tür, Arc::new(üretici));
        Ok(())
    }

    pub fn oluştur(
        &self,
        tür: &str,
        seçenekler: &GrafikSeçenekleri,
        alan: Dikdörtgen,
    ) -> Result<Box<dyn ÖzelKoordinatSistemi>, BilesenHatasi> {
        boyutları_doğrula(alan.genişlik, alan.yükseklik, "coordinateSystem.size")?;
        self.üreticiler
            .get(tür)
            .ok_or_else(|| kayıtlı_değil("coordinateSystem.type", tür))?
            .oluştur(seçenekler, alan)
    }
}

/// Bir genişletme paketinin kayıt yapabildiği bütün çekirdek yüzeyler.
#[derive(Default)]
pub struct GenişletmeBağlamı {
    ön_işleme: ÖnİşlemeKayıtDefteri,
    zamanlayıcı: Zamanlayıcı,
    dönüşümler: DönüşümKayıtDefteri,
    eylemler: EylemKayıtDefteri,
    yüklemeler: YüklemeKayıtDefteri,
    koordinat_sistemleri: KoordinatSistemiKayıtDefteri,
}

impl GenişletmeBağlamı {
    pub fn ön_işleyici_kaydet(
        &mut self,
        işleyici: impl SeçenekÖnİşleyicisi + 'static,
    ) -> Result<(), BilesenHatasi> {
        self.ön_işleme.kaydet(işleyici)
    }

    pub fn görev_kaydet(
        &mut self,
        görev: impl ArtımlıGörev + 'static,
    ) -> Result<(), BilesenHatasi> {
        self.zamanlayıcı.kaydet(görev)
    }

    pub fn dönüşüm_kaydet(
        &mut self,
        dönüşüm: impl VeriDönüşümü + 'static,
    ) -> Result<(), BilesenHatasi> {
        self.dönüşümler.kaydet(dönüşüm)
    }

    pub fn yükleme_kaydet(
        &mut self,
        çizici: impl YüklemeÇizicisi + 'static,
    ) -> Result<(), BilesenHatasi> {
        self.yüklemeler.kaydet(çizici)
    }

    pub fn koordinat_sistemi_kaydet(
        &mut self,
        üretici: impl KoordinatSistemiÜreticisi + 'static,
    ) -> Result<(), BilesenHatasi> {
        self.koordinat_sistemleri.kaydet(üretici)
    }

    pub fn ön_işleme_mut(&mut self) -> &mut ÖnİşlemeKayıtDefteri {
        &mut self.ön_işleme
    }

    pub fn zamanlayıcı_mut(&mut self) -> &mut Zamanlayıcı {
        &mut self.zamanlayıcı
    }

    pub fn dönüşümler(&self) -> &DönüşümKayıtDefteri {
        &self.dönüşümler
    }

    pub fn dönüşümler_mut(&mut self) -> &mut DönüşümKayıtDefteri {
        &mut self.dönüşümler
    }

    pub fn eylemler(&self) -> &EylemKayıtDefteri {
        &self.eylemler
    }

    pub fn eylemler_mut(&mut self) -> &mut EylemKayıtDefteri {
        &mut self.eylemler
    }

    pub fn yüklemeler(&self) -> &YüklemeKayıtDefteri {
        &self.yüklemeler
    }

    pub fn koordinat_sistemleri(&self) -> &KoordinatSistemiKayıtDefteri {
        &self.koordinat_sistemleri
    }
}

/// Tek bir `echarts.use(extension)` paketi.
pub trait Genişletme: Send {
    fn kimlik(&self) -> &str;

    fn bağımlılıklar(&self) -> &[String] {
        &[]
    }

    fn kur(&mut self, bağlam: &mut GenişletmeBağlamı) -> Result<(), BilesenHatasi>;
}

/// Paketleri bir kez, bağımlılık sırasıyla kuran `echarts.use` kayıt defteri.
#[derive(Default)]
pub struct GenişletmeKayıtDefteri {
    genişletmeler: Vec<Box<dyn Genişletme>>,
    bağlam: GenişletmeBağlamı,
    hazır: bool,
    kurulum_sırası: Vec<String>,
}

impl GenişletmeKayıtDefteri {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn kullan(&mut self, genişletme: impl Genişletme + 'static) -> Result<(), BilesenHatasi> {
        if self.hazır {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "extension.lifecycle",
                ayrıntı: "hazırlanmış kayıt defterine yeni paket eklenemez".to_owned(),
            });
        }
        let kimlik = genişletme.kimlik().trim();
        kayıt_adını_doğrula(kimlik, "extension.id")?;
        if self
            .genişletmeler
            .iter()
            .any(|kayıt| kayıt.kimlik() == kimlik)
        {
            return Err(yinelenen_kayıt("extension.id", kimlik));
        }
        self.genişletmeler.push(Box::new(genişletme));
        Ok(())
    }

    /// Bağımlılıkları çözer ve paketleri bir kez kurar. İkinci çağrı
    /// ECharts `use` gibi etkisiz ve başarılıdır.
    pub fn hazırla(&mut self) -> Result<(), BilesenHatasi> {
        if self.hazır {
            return Ok(());
        }
        let kimlikler: Vec<String> = self
            .genişletmeler
            .iter()
            .map(|genişletme| genişletme.kimlik().to_owned())
            .collect();
        let bağımlılıklar: Vec<Vec<String>> = self
            .genişletmeler
            .iter()
            .map(|genişletme| genişletme.bağımlılıklar().to_vec())
            .collect();
        let sıra = bağımlılık_sırası(&kimlikler, &bağımlılıklar, "extension.dependencies")?;
        let (genişletmeler, bağlam) = (&mut self.genişletmeler, &mut self.bağlam);
        for sıra in sıra {
            let genişletme = genişletmeler
                .get_mut(sıra)
                .ok_or(BilesenHatasi::EksikVeri {
                    bileşen: "extension",
                    sıra,
                })?;
            genişletme.kur(bağlam)?;
            self.kurulum_sırası.push(genişletme.kimlik().to_owned());
        }
        self.hazır = true;
        Ok(())
    }

    pub fn bağlam(&self) -> &GenişletmeBağlamı {
        &self.bağlam
    }

    pub fn bağlam_mut(&mut self) -> Result<&mut GenişletmeBağlamı, BilesenHatasi> {
        if !self.hazır {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "extension.lifecycle",
                ayrıntı: "bağlam kullanılmadan önce hazırlanmalı".to_owned(),
            });
        }
        Ok(&mut self.bağlam)
    }

    pub fn kurulum_sırası(&self) -> &[String] {
        &self.kurulum_sırası
    }
}

fn bağımlılık_sırası(
    kimlikler: &[String],
    bağımlılıklar: &[Vec<String>],
    alan: &'static str,
) -> Result<Vec<usize>, BilesenHatasi> {
    let mevcut: BTreeSet<&str> = kimlikler.iter().map(String::as_str).collect();
    for (sıra, bağımlılıklar) in bağımlılıklar.iter().enumerate() {
        for bağımlılık in bağımlılıklar {
            if !mevcut.contains(bağımlılık.as_str()) {
                let kimlik = kimlikler.get(sıra).map(String::as_str).unwrap_or("?");
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan,
                    ayrıntı: format!("`{kimlik}` kaydının `{bağımlılık}` bağımlılığı yok"),
                });
            }
        }
    }

    let mut kalan: Vec<usize> = (0..kimlikler.len()).collect();
    let mut tamamlanan = BTreeSet::new();
    let mut sonuç = Vec::with_capacity(kalan.len());
    while !kalan.is_empty() {
        let seçim = kalan.iter().position(|sıra| {
            bağımlılıklar
                .get(*sıra)
                .map(|bağımlılıklar| {
                    bağımlılıklar
                        .iter()
                        .all(|bağımlılık| tamamlanan.contains(bağımlılık))
                })
                .unwrap_or(false)
        });
        let Some(seçim) = seçim else {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan,
                ayrıntı: "döngüsel bağımlılık var".to_owned(),
            });
        };
        let sıra = kalan.remove(seçim);
        if let Some(kimlik) = kimlikler.get(sıra) {
            tamamlanan.insert(kimlik.clone());
        }
        sonuç.push(sıra);
    }
    Ok(sonuç)
}

fn kayıt_adını_doğrula(ad: &str, alan: &'static str) -> Result<(), BilesenHatasi> {
    if ad.is_empty() {
        return Err(BilesenHatasi::GeçersizSeçenek {
            alan,
            ayrıntı: "kayıt adı boş olamaz".to_owned(),
        });
    }
    Ok(())
}

fn yinelenen_kayıt(alan: &'static str, ad: &str) -> BilesenHatasi {
    BilesenHatasi::GeçersizSeçenek {
        alan,
        ayrıntı: format!("`{ad}` zaten kayıtlı"),
    }
}

fn kayıtlı_değil(alan: &'static str, ad: &str) -> BilesenHatasi {
    BilesenHatasi::GeçersizSeçenek {
        alan,
        ayrıntı: format!("`{ad}` kayıtlı değil"),
    }
}

fn boyutları_doğrula(
    genişlik: f32,
    yükseklik: f32,
    alan: &'static str,
) -> Result<(), BilesenHatasi> {
    if !genişlik.is_finite() || !yükseklik.is_finite() || genişlik <= 0.0 || yükseklik <= 0.0 {
        return Err(BilesenHatasi::GeçersizSeçenek {
            alan,
            ayrıntı: "genişlik ve yükseklik sıfırdan büyük sonlu değerler olmalı".to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod testler {
    use std::ops::Range;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::calisma_zamani::{GrafikÇalışmaZamanı, ÖrnekBaşlatmaSeçenekleri};
    use crate::eylem::{EylemGüncellemesi, EylemYükü, OlayYükü};
    use crate::model::deger::VeriDeğeri;
    use crate::model::veri_kumesi::{BoyutTanımı, VeriDeposu};
    use crate::zamanlayici::{GörevAşaması, GörevBağlamı, ZamanlayıcıDurumu};

    struct İzÖnİşleyici {
        kimlik: String,
        bağımlılıklar: Vec<String>,
        iz: Arc<Mutex<Vec<String>>>,
        süre: f32,
    }

    impl SeçenekÖnİşleyicisi for İzÖnİşleyici {
        fn kimlik(&self) -> &str {
            &self.kimlik
        }

        fn bağımlılıklar(&self) -> &[String] {
            &self.bağımlılıklar
        }

        fn işle(&mut self, seçenekler: &mut GrafikSeçenekleri) -> Result<(), BilesenHatasi> {
            self.iz.lock().unwrap().push(self.kimlik.clone());
            seçenekler.animasyon_süresi = self.süre;
            Ok(())
        }
    }

    struct TekGörev;

    impl ArtımlıGörev for TekGörev {
        fn kimlik(&self) -> &str {
            "paket.görev"
        }

        fn aşama(&self) -> GörevAşaması {
            GörevAşaması::Yerleşim
        }

        fn hazırla(&mut self, bağlam: &mut GörevBağlamı) -> Result<usize, BilesenHatasi> {
            bağlam.iz.push("görev:hazır".to_owned());
            Ok(1)
        }

        fn çalıştır(
            &mut self,
            aralık: Range<usize>,
            bağlam: &mut GörevBağlamı,
        ) -> Result<(), BilesenHatasi> {
            bağlam
                .iz
                .push(format!("görev:{}..{}", aralık.start, aralık.end));
            Ok(())
        }
    }

    struct KimlikDönüşümü;

    impl VeriDönüşümü for KimlikDönüşümü {
        fn tür_adı(&self) -> &str {
            "kimlik"
        }

        fn uygula(&self, upstream: &[VeriDeposu]) -> Result<Vec<VeriDeposu>, BilesenHatasi> {
            Ok(upstream.to_vec())
        }
    }

    struct BoşYükleme;

    impl YüklemeÇizicisi for BoşYükleme {
        fn tür_adı(&self) -> &str {
            "boş"
        }

        fn sahne(&self, _: &YüklemeGirdisi) -> Result<Sahne, BilesenHatasi> {
            Ok(Sahne::yeni())
        }
    }

    struct DoğrusalKoordinat {
        alan: Dikdörtgen,
    }

    impl ÖzelKoordinatSistemi for DoğrusalKoordinat {
        fn veriden_piksele(&self, değer: [f64; 2]) -> Result<[f32; 2], BilesenHatasi> {
            Ok([
                self.alan.x + değer[0] as f32 * self.alan.genişlik,
                self.alan.y + değer[1] as f32 * self.alan.yükseklik,
            ])
        }

        fn pikselden_veriye(&self, piksel: [f32; 2]) -> Result<[f64; 2], BilesenHatasi> {
            Ok([
                ((piksel[0] - self.alan.x) / self.alan.genişlik) as f64,
                ((piksel[1] - self.alan.y) / self.alan.yükseklik) as f64,
            ])
        }

        fn piksel_içeriyor_mu(&self, piksel: [f32; 2]) -> bool {
            self.alan.içeriyor_mu((piksel[0], piksel[1]))
        }
    }

    struct DoğrusalÜretici;

    impl KoordinatSistemiÜreticisi for DoğrusalÜretici {
        fn tür_adı(&self) -> &str {
            "doğrusal-test"
        }

        fn oluştur(
            &self,
            _: &GrafikSeçenekleri,
            alan: Dikdörtgen,
        ) -> Result<Box<dyn ÖzelKoordinatSistemi>, BilesenHatasi> {
            Ok(Box::new(DoğrusalKoordinat { alan }))
        }
    }

    struct Paket {
        kimlik: String,
        bağımlılıklar: Vec<String>,
        iz: Arc<Mutex<Vec<String>>>,
        kayıt_yap: bool,
    }

    impl Genişletme for Paket {
        fn kimlik(&self) -> &str {
            &self.kimlik
        }

        fn bağımlılıklar(&self) -> &[String] {
            &self.bağımlılıklar
        }

        fn kur(&mut self, bağlam: &mut GenişletmeBağlamı) -> Result<(), BilesenHatasi> {
            self.iz.lock().unwrap().push(self.kimlik.clone());
            if !self.kayıt_yap {
                return Ok(());
            }
            bağlam.görev_kaydet(TekGörev)?;
            bağlam.dönüşüm_kaydet(KimlikDönüşümü)?;
            bağlam.yükleme_kaydet(BoşYükleme)?;
            bağlam.koordinat_sistemi_kaydet(DoğrusalÜretici)?;
            bağlam.eylemler_mut().kaydet(
                "özel",
                "özelolay",
                EylemGüncellemesi::Yok,
                |_, _| Ok(vec![OlayYükü::default()]),
            )?;
            Ok(())
        }
    }

    #[test]
    fn preprocessor_bagimlilik_sirasi_ve_atomiklik() {
        let iz = Arc::new(Mutex::new(Vec::new()));
        let mut kayıt = ÖnİşlemeKayıtDefteri::yeni();
        kayıt
            .kaydet(İzÖnİşleyici {
                kimlik: "son".to_owned(),
                bağımlılıklar: vec!["ilk".to_owned()],
                iz: iz.clone(),
                süre: 20.0,
            })
            .unwrap();
        kayıt
            .kaydet(İzÖnİşleyici {
                kimlik: "ilk".to_owned(),
                bağımlılıklar: vec![],
                iz: iz.clone(),
                süre: 10.0,
            })
            .unwrap();
        let mut seçenekler = GrafikSeçenekleri::default();
        kayıt.uygula(&mut seçenekler).unwrap();
        assert_eq!(*iz.lock().unwrap(), vec!["ilk", "son"]);
        assert_eq!(seçenekler.animasyon_süresi, 20.0);
    }

    #[test]
    fn use_paketleri_ve_tum_register_yuzeyleri_birlikte_calismali() {
        let iz = Arc::new(Mutex::new(Vec::new()));
        let mut kayıt = GenişletmeKayıtDefteri::yeni();
        kayıt
            .kullan(Paket {
                kimlik: "b".to_owned(),
                bağımlılıklar: vec!["a".to_owned()],
                iz: iz.clone(),
                kayıt_yap: true,
            })
            .unwrap();
        kayıt
            .kullan(Paket {
                kimlik: "a".to_owned(),
                bağımlılıklar: vec![],
                iz: iz.clone(),
                kayıt_yap: false,
            })
            .unwrap();
        kayıt.hazırla().unwrap();
        kayıt.hazırla().unwrap();
        assert_eq!(kayıt.kurulum_sırası(), &["a", "b"]);
        assert_eq!(*iz.lock().unwrap(), vec!["a", "b"]);

        let bağlam = kayıt.bağlam_mut().unwrap();
        bağlam.zamanlayıcı_mut().hazırla().unwrap();
        bağlam.zamanlayıcı_mut().adım(1).unwrap();
        assert_eq!(
            bağlam.zamanlayıcı_mut().durum(),
            ZamanlayıcıDurumu::Tamamlandı
        );

        let depo =
            VeriDeposu::satırlardan([BoyutTanımı::yeni("x")], vec![vec![VeriDeğeri::from(1)]])
                .unwrap();
        assert_eq!(
            bağlam
                .dönüşümler()
                .çalıştır("kimlik", &[depo])
                .unwrap()
                .len(),
            1
        );

        let mut çalışma = GrafikÇalışmaZamanı::yeni(
            ÖrnekBaşlatmaSeçenekleri::default(),
            GrafikSeçenekleri::default(),
        )
        .unwrap();
        assert_eq!(
            bağlam
                .eylemler()
                .gönder(&mut çalışma, &EylemYükü::yeni("özel"))
                .unwrap()[0]
                .tür,
            "özelolay"
        );

        assert!(
            bağlam
                .yüklemeler()
                .sahne(
                    "boş",
                    &YüklemeGirdisi {
                        genişlik: 100.0,
                        yükseklik: 50.0,
                        metin: None,
                        ilerleme: Some(0.5),
                    },
                )
                .unwrap()
                .kökler
                .is_empty()
        );
        let koordinat = bağlam
            .koordinat_sistemleri()
            .oluştur(
                "doğrusal-test",
                &GrafikSeçenekleri::default(),
                Dikdörtgen::yeni(10.0, 20.0, 100.0, 50.0),
            )
            .unwrap();
        assert_eq!(koordinat.veriden_piksele([0.5, 0.5]).unwrap(), [60.0, 45.0]);
        assert!(koordinat.piksel_içeriyor_mu([60.0, 45.0]));
    }

    #[test]
    fn geo_map_ve_gl_koordinat_kaydi_acikca_reddedilir() {
        struct Yasak(&'static str);
        impl KoordinatSistemiÜreticisi for Yasak {
            fn tür_adı(&self) -> &str {
                self.0
            }

            fn oluştur(
                &self,
                _: &GrafikSeçenekleri,
                _: Dikdörtgen,
            ) -> Result<Box<dyn ÖzelKoordinatSistemi>, BilesenHatasi> {
                Err(BilesenHatasi::Desteklenmeyen {
                    özellik: self.0,
                    ayrıntı: "yasak koordinat üreticisi çağrılmamalı".to_owned(),
                })
            }
        }
        for tür in ["geo", "map", "scatterGL"] {
            let mut kayıt = KoordinatSistemiKayıtDefteri::default();
            assert!(matches!(
                kayıt.kaydet(Yasak(tür)),
                Err(BilesenHatasi::Desteklenmeyen { .. })
            ));
        }
    }
}
