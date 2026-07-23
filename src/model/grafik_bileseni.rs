//! Serbest grafik bileşeni — ECharts `graphic` / zrender öğelerinin
//! bildirime dayalı seçenek karşılığı.
//!
//! Öğeler hiyerarşik olabilir; ortak dönüşüm, yerleşim, görünürlük ve olay
//! alanları içerikten bağımsız tutulur. Çizim katmanı bu modeli aynı sahne
//! ağacına dönüştürerek PNG, SVG, kayıt ve gpui yüzeylerinde ortak davranış
//! sağlar.

use std::collections::{BTreeMap, BTreeSet};

use crate::animasyon::Yumuşatma;
use crate::cizim::{
    GörselDurum, KırpmaYolu, OdakKapsamı, SahneMetni, SahneResmi, SahneStilYaması, SahneStili,
    SahneŞekli, YerelDönüşüm,
};
use crate::eylem::EylemDeğeri;
use crate::hata::BilesenHatasi;
use crate::koordinat::Dikdörtgen;
use crate::model::{DikeyKonum, Uzunluk, YatayKonum};
use crate::renk::{Dolgu, Renk};

/// ECharts `graphic` kök bileşeni.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GrafikBileşeni {
    pub öğeler: Vec<GrafikÖğesi>,
}

impl GrafikBileşeni {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn öğe(mut self, öğe: GrafikÖğesi) -> Self {
        self.öğeler.push(öğe);
        self
    }

    pub fn öğeler(mut self, öğeler: impl IntoIterator<Item = GrafikÖğesi>) -> Self {
        self.öğeler.extend(öğeler);
        self
    }

    /// Kimlikli bir öğenin `ignore` alanını değiştirir. Bu, olay
    /// dinleyicisinden yapılan küçük `setOption({graphic: ...})`
    /// güncellemelerinin doğrudan karşılığıdır.
    pub fn yoksaymayı_ayarla(&mut self, kimlik: &str, yoksay: bool) -> bool {
        self.öğeler
            .iter_mut()
            .any(|öğe| öğe.yoksaymayı_ayarla(kimlik, yoksay))
    }

    /// İç içe gruplar dahil açık `id` taşıyan öğeyi döndürür.
    pub fn öğeyi_bul(&self, kimlik: &str) -> Option<&GrafikÖğesi> {
        self.öğeler.iter().find_map(|öğe| öğe.öğeyi_bul(kimlik))
    }

    /// İç içe gruplar dahil açık `id` taşıyan öğeyi değiştirilebilir döndürür.
    pub fn öğeyi_bul_mut(&mut self, kimlik: &str) -> Option<&mut GrafikÖğesi> {
        self.öğeler
            .iter_mut()
            .find_map(|öğe| öğe.öğeyi_bul_mut(kimlik))
    }

    /// Bir öğenin kökten başlayan dizi sıra yolunu bulur. Kimliksiz ama
    /// sürüklenebilir öğeler, sahne isabetinden bu kararlı yolla güncellenir.
    pub fn öğe_yolu(&self, kimlik: &str) -> Option<Vec<usize>> {
        let mut yol = Vec::new();
        öğe_yolunu_bul(&self.öğeler, kimlik, &mut yol).then_some(yol)
    }

    pub fn öğeyi_yolda(&self, yol: &[usize]) -> Option<&GrafikÖğesi> {
        öğeyi_yolda_bul(&self.öğeler, yol)
    }

    pub fn öğeyi_yolda_mut(&mut self, yol: &[usize]) -> Option<&mut GrafikÖğesi> {
        öğeyi_yolda_bul_mut(&mut self.öğeler, yol)
    }

    /// ECharts `graphic.elements[].$action: 'merge'` karşılığı.
    pub fn öğeyi_birleştir(&mut self, kimlik: &str, yama: &GrafikÖğeYaması) -> bool {
        let Some(öğe) = self.öğeyi_bul_mut(kimlik) else {
            return false;
        };
        yama.uygula(öğe);
        true
    }

    /// ECharts `graphic.elements[].$action: 'replace'` karşılığı.
    pub fn öğeyi_değiştir(&mut self, kimlik: &str, mut yeni: GrafikÖğesi) -> bool {
        if yeni.kimlik.is_none() {
            yeni.kimlik = Some(kimlik.to_owned());
        }
        öğeyi_değiştir(&mut self.öğeler, kimlik, &mut Some(yeni))
    }

    /// ECharts `graphic.elements[].$action: 'remove'` karşılığı.
    pub fn öğeyi_kaldır(&mut self, kimlik: &str) -> bool {
        öğeyi_kaldır(&mut self.öğeler, kimlik)
    }

    pub fn eylemi_uygula(&mut self, eylem: GrafikÖğeEylemi) -> bool {
        match eylem {
            GrafikÖğeEylemi::Birleştir { kimlik, yama } => {
                self.öğeyi_birleştir(&kimlik, &yama)
            }
            GrafikÖğeEylemi::Değiştir { kimlik, öğe } => {
                self.öğeyi_değiştir(&kimlik, öğe)
            }
            GrafikÖğeEylemi::Kaldır { kimlik } => self.öğeyi_kaldır(&kimlik),
        }
    }

    pub fn doğrula(&self) -> Result<(), BilesenHatasi> {
        let mut kimlikler = BTreeSet::new();
        for öğe in &self.öğeler {
            öğe.doğrula(&mut kimlikler)?;
        }
        Ok(())
    }
}

/// `setOption({graphic:{elements:[{$action: ...}]}})` içindeki öğe eylemi.
#[derive(Clone, Debug, PartialEq)]
pub enum GrafikÖğeEylemi {
    Birleştir {
        kimlik: String,
        yama: GrafikÖğeYaması,
    },
    Değiştir {
        kimlik: String,
        öğe: GrafikÖğesi,
    },
    Kaldır {
        kimlik: String,
    },
}

impl GrafikÖğeEylemi {
    pub fn birleştir(kimlik: impl Into<String>, yama: GrafikÖğeYaması) -> Self {
        Self::Birleştir {
            kimlik: kimlik.into(),
            yama,
        }
    }

    pub fn değiştir(kimlik: impl Into<String>, öğe: GrafikÖğesi) -> Self {
        Self::Değiştir {
            kimlik: kimlik.into(),
            öğe,
        }
    }

    pub fn kaldır(kimlik: impl Into<String>) -> Self {
        Self::Kaldır {
            kimlik: kimlik.into(),
        }
    }
}

/// Graphic öğesinin yalnız sağlanan alanlarını değiştiren tipli merge
/// yaması. `Option<Option<T>>`, açık `null` ile alanı temizleme ayrımını
/// korur (`name`, `textContent`, `cursor`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GrafikÖğeYaması {
    pub ad: Option<Option<String>>,
    pub bilgi: Option<BTreeMap<String, EylemDeğeri>>,
    pub içerik: Option<GrafikÖğeİçeriği>,
    pub grup_boyutu: Option<Option<(f32, f32)>>,
    pub sınırlama: Option<GrafikSınırlamaKipi>,
    pub yerleşim: Option<GrafikYerleşimi>,
    pub dönüşüm: Option<YerelDönüşüm>,
    pub kırpmalar: Option<Vec<KırpmaYolu>>,
    pub stil: SahneStilYaması,
    pub durum_stilleri: Option<BTreeMap<GörselDurum, SahneStilYaması>>,
    pub durum: Option<GörselDurum>,
    pub odak: Option<OdakKapsamı>,
    pub bağlı_metin: Option<Option<GrafikBağlıMetni>>,
    pub anahtar_kare_animasyonları: Option<Vec<GrafikAnahtarKareAnimasyonu>>,
    pub zlevel: Option<i32>,
    pub z: Option<f32>,
    pub z2: Option<f32>,
    pub yoksay: Option<bool>,
    pub görünmez: Option<bool>,
    pub sessiz: Option<bool>,
    pub sürüklenebilir: Option<bool>,
    pub imleç: Option<Option<String>>,
}

impl GrafikÖğeYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn konum(mut self, x: f32, y: f32) -> Self {
        let mut dönüşüm = self.dönüşüm.unwrap_or_default();
        dönüşüm.x = x;
        dönüşüm.y = y;
        self.dönüşüm = Some(dönüşüm);
        self
    }

    pub fn dönüşüm(mut self, dönüşüm: YerelDönüşüm) -> Self {
        self.dönüşüm = Some(dönüşüm);
        self
    }

    pub fn içerik(mut self, içerik: GrafikÖğeİçeriği) -> Self {
        self.içerik = Some(içerik);
        self
    }

    pub fn bilgiler(mut self, bilgiler: impl IntoIterator<Item = (String, EylemDeğeri)>) -> Self {
        self.bilgi = Some(bilgiler.into_iter().collect());
        self
    }

    pub fn stil(mut self, stil: SahneStilYaması) -> Self {
        self.stil = stil;
        self
    }

    pub fn yoksay(mut self, yoksay: bool) -> Self {
        self.yoksay = Some(yoksay);
        self
    }

    pub fn görünmez(mut self, görünmez: bool) -> Self {
        self.görünmez = Some(görünmez);
        self
    }

    pub fn sürüklenebilir(mut self, sürüklenebilir: bool) -> Self {
        self.sürüklenebilir = Some(sürüklenebilir);
        self
    }

    pub fn imleç(mut self, imleç: impl Into<String>) -> Self {
        self.imleç = Some(Some(imleç.into()));
        self
    }

    fn uygula(&self, öğe: &mut GrafikÖğesi) {
        if let Some(ad) = &self.ad {
            öğe.ad = ad.clone();
        }
        if let Some(bilgi) = &self.bilgi {
            öğe.bilgi = bilgi.clone();
        }
        if let Some(içerik) = &self.içerik {
            öğe.içerik = içerik.clone();
        }
        if let Some(grup_boyutu) = self.grup_boyutu {
            öğe.grup_boyutu = grup_boyutu;
        }
        if let Some(sınırlama) = self.sınırlama {
            öğe.sınırlama = sınırlama;
        }
        if let Some(yerleşim) = self.yerleşim {
            öğe.yerleşim = yerleşim;
        }
        if let Some(dönüşüm) = self.dönüşüm {
            öğe.dönüşüm = dönüşüm;
        }
        if let Some(kırpmalar) = &self.kırpmalar {
            öğe.kırpmalar = kırpmalar.clone();
        }
        self.stil.uygula(&mut öğe.stil);
        if let Some(durum_stilleri) = &self.durum_stilleri {
            öğe.durum_stilleri = durum_stilleri.clone();
        }
        if let Some(durum) = self.durum {
            öğe.durum = durum;
        }
        if let Some(odak) = self.odak {
            öğe.odak = odak;
        }
        if let Some(bağlı_metin) = &self.bağlı_metin {
            öğe.bağlı_metin = bağlı_metin.clone();
        }
        if let Some(animasyonlar) = &self.anahtar_kare_animasyonları {
            öğe.anahtar_kare_animasyonları = animasyonlar.clone();
        }
        if let Some(zlevel) = self.zlevel {
            öğe.zlevel = zlevel;
        }
        if let Some(z) = self.z {
            öğe.z = z;
        }
        if let Some(z2) = self.z2 {
            öğe.z2 = z2;
        }
        if let Some(yoksay) = self.yoksay {
            öğe.yoksay = yoksay;
        }
        if let Some(görünmez) = self.görünmez {
            öğe.görünmez = görünmez;
        }
        if let Some(sessiz) = self.sessiz {
            öğe.sessiz = sessiz;
        }
        if let Some(sürüklenebilir) = self.sürüklenebilir {
            öğe.sürüklenebilir = sürüklenebilir;
        }
        if let Some(imleç) = &self.imleç {
            öğe.imleç = imleç.clone();
        }
    }
}

fn öğe_yolunu_bul(öğeler: &[GrafikÖğesi], kimlik: &str, yol: &mut Vec<usize>) -> bool {
    for (sıra, öğe) in öğeler.iter().enumerate() {
        yol.push(sıra);
        if öğe.kimlik.as_deref() == Some(kimlik) {
            return true;
        }
        if let GrafikÖğeİçeriği::Grup(çocuklar) = &öğe.içerik
            && öğe_yolunu_bul(çocuklar, kimlik, yol)
        {
            return true;
        }
        yol.pop();
    }
    false
}

fn öğeyi_yolda_bul<'a>(
    öğeler: &'a [GrafikÖğesi], yol: &[usize]
) -> Option<&'a GrafikÖğesi> {
    let (&ilk, kalan) = yol.split_first()?;
    let öğe = öğeler.get(ilk)?;
    if kalan.is_empty() {
        return Some(öğe);
    }
    let GrafikÖğeİçeriği::Grup(çocuklar) = &öğe.içerik else {
        return None;
    };
    öğeyi_yolda_bul(çocuklar, kalan)
}

fn öğeyi_yolda_bul_mut<'a>(
    öğeler: &'a mut [GrafikÖğesi],
    yol: &[usize],
) -> Option<&'a mut GrafikÖğesi> {
    let (&ilk, kalan) = yol.split_first()?;
    let öğe = öğeler.get_mut(ilk)?;
    if kalan.is_empty() {
        return Some(öğe);
    }
    let GrafikÖğeİçeriği::Grup(çocuklar) = &mut öğe.içerik else {
        return None;
    };
    öğeyi_yolda_bul_mut(çocuklar, kalan)
}

fn öğeyi_değiştir(
    öğeler: &mut [GrafikÖğesi],
    kimlik: &str,
    yeni: &mut Option<GrafikÖğesi>,
) -> bool {
    for öğe in öğeler {
        if öğe.kimlik.as_deref() == Some(kimlik) {
            if let Some(yeni) = yeni.take() {
                *öğe = yeni;
            }
            return true;
        }
        if let GrafikÖğeİçeriği::Grup(çocuklar) = &mut öğe.içerik
            && öğeyi_değiştir(çocuklar, kimlik, yeni)
        {
            return true;
        }
    }
    false
}

fn öğeyi_kaldır(öğeler: &mut Vec<GrafikÖğesi>, kimlik: &str) -> bool {
    if let Some(sıra) = öğeler
        .iter()
        .position(|öğe| öğe.kimlik.as_deref() == Some(kimlik))
    {
        öğeler.remove(sıra);
        return true;
    }
    öğeler.iter_mut().any(|öğe| {
        let GrafikÖğeİçeriği::Grup(çocuklar) = &mut öğe.içerik else {
            return false;
        };
        öğeyi_kaldır(çocuklar, kimlik)
    })
}

/// Bir `graphic` öğesinin içeriği (`type`).
#[derive(Clone, Debug, PartialEq)]
pub enum GrafikÖğeİçeriği {
    Grup(Vec<GrafikÖğesi>),
    Şekil(SahneŞekli),
    Metin(SahneMetni),
    Resim(SahneResmi),
}

/// Graphic group `bounding`: dönüşümlü bütün alt ağacı veya grubun ham,
/// dönüşüm öncesi alt-ağaç sınırını yerleşimde kullanır.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GrafikSınırlamaKipi {
    #[default]
    Tümü,
    Ham,
}

/// ECharts/zrender `keyframeAnimation.keyframes` içindeki tek kısmi kare.
/// Yalnız sağlanan alanlar kendi izini oluşturur; sağlanmayan alanlar öğenin
/// taban değerini ve önceki anahtar karenin değerini korur.
#[derive(Clone, Debug, PartialEq)]
pub struct GrafikAnahtarKare {
    pub yüzde: f32,
    pub yumuşatma: Yumuşatma,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub ölçek_x: Option<f32>,
    pub ölçek_y: Option<f32>,
    pub dönüş: Option<f32>,
    pub stil: SahneStilYaması,
    pub şekil: Option<crate::cizim::SahneŞekli>,
}

impl GrafikAnahtarKare {
    pub fn yeni(yüzde: f32) -> Self {
        Self {
            yüzde,
            yumuşatma: Yumuşatma::Doğrusal,
            x: None,
            y: None,
            ölçek_x: None,
            ölçek_y: None,
            dönüş: None,
            stil: SahneStilYaması::default(),
            şekil: None,
        }
    }

    pub fn yumuşatma(mut self, yumuşatma: Yumuşatma) -> Self {
        self.yumuşatma = yumuşatma;
        self
    }

    pub fn konum(mut self, x: f32, y: f32) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    pub fn ölçek(mut self, x: f32, y: f32) -> Self {
        self.ölçek_x = Some(x);
        self.ölçek_y = Some(y);
        self
    }

    pub fn dönüş(mut self, radyan: f32) -> Self {
        self.dönüş = Some(radyan);
        self
    }

    pub fn stil(mut self, stil: SahneStilYaması) -> Self {
        self.stil = stil;
        self
    }

    pub fn dolgu(mut self, dolgu: impl Into<Dolgu>) -> Self {
        self.stil.dolgu = Some(dolgu.into());
        self
    }

    pub fn çizgi_deseni(mut self, desen: impl IntoIterator<Item = f32>, kayma: f32) -> Self {
        self.stil.çizgi_deseni = Some(desen.into_iter().collect());
        self.stil.çizgi_deseni_kayması = Some(kayma);
        self
    }

    pub fn şekil(mut self, şekil: crate::cizim::SahneŞekli) -> Self {
        self.şekil = Some(şekil);
        self
    }

    pub(crate) fn uygula(&self, öğe: &mut GrafikÖğesi) {
        if let Some(x) = self.x {
            öğe.dönüşüm.x = x;
        }
        if let Some(y) = self.y {
            öğe.dönüşüm.y = y;
        }
        if let Some(ölçek) = self.ölçek_x {
            öğe.dönüşüm.ölçek_x = ölçek;
        }
        if let Some(ölçek) = self.ölçek_y {
            öğe.dönüşüm.ölçek_y = ölçek;
        }
        if let Some(dönüş) = self.dönüş {
            öğe.dönüşüm.dönüş = dönüş;
        }
        self.stil.uygula(&mut öğe.stil);
        if let Some(Dolgu::Düz(renk)) = &self.stil.dolgu
            && let GrafikÖğeİçeriği::Metin(metin) = &mut öğe.içerik
        {
            metin.renk = *renk;
        }
        if let Some(şekil) = &self.şekil
            && let GrafikÖğeİçeriği::Şekil(mevcut) = &mut öğe.içerik
        {
            *mevcut = şekil.clone();
        }
    }
}

/// ECharts `keyframeAnimation` zaman çizelgesi. Süre ve gecikme, resmî
/// option sözleşmesindeki gibi milisaniyedir.
#[derive(Clone, Debug, PartialEq)]
pub struct GrafikAnahtarKareAnimasyonu {
    pub süre_ms: f32,
    pub gecikme_ms: f32,
    pub döngü: bool,
    pub yumuşatma: Yumuşatma,
    pub kareler: Vec<GrafikAnahtarKare>,
}

impl GrafikAnahtarKareAnimasyonu {
    pub fn yeni(süre_ms: f32) -> Self {
        Self {
            süre_ms,
            gecikme_ms: 0.0,
            döngü: false,
            yumuşatma: Yumuşatma::Doğrusal,
            kareler: Vec::new(),
        }
    }

    pub fn gecikme(mut self, gecikme_ms: f32) -> Self {
        self.gecikme_ms = gecikme_ms;
        self
    }

    pub fn döngü(mut self, döngü: bool) -> Self {
        self.döngü = döngü;
        self
    }

    pub fn yumuşatma(mut self, yumuşatma: Yumuşatma) -> Self {
        self.yumuşatma = yumuşatma;
        self
    }

    pub fn kare(mut self, kare: GrafikAnahtarKare) -> Self {
        self.kareler.push(kare);
        self
    }
}

/// ECharts `graphic` öğesi. Alan adları zrender sözleşmesindeki karşılıkları
/// izler: `ignore`, `invisible`, `silent`, `draggable`, `cursor`, `zlevel`,
/// `z`, `z2`, dönüşüm ve bağlı `textContent`.
#[derive(Clone, Debug, PartialEq)]
pub struct GrafikÖğesi {
    pub kimlik: Option<String>,
    pub ad: Option<String>,
    /// ECharts `info`: olay yükünde aynen geri verilen kullanıcı verisi.
    pub bilgi: BTreeMap<String, EylemDeğeri>,
    pub içerik: GrafikÖğeİçeriği,
    /// Group `width/height`; alt öğelerin yüzde/kenar yerleşim kabıdır.
    pub grup_boyutu: Option<(f32, f32)>,
    pub sınırlama: GrafikSınırlamaKipi,
    pub yerleşim: GrafikYerleşimi,
    pub dönüşüm: YerelDönüşüm,
    pub kırpmalar: Vec<KırpmaYolu>,
    pub stil: SahneStili,
    pub durum_stilleri: BTreeMap<GörselDurum, SahneStilYaması>,
    pub durum: GörselDurum,
    pub odak: OdakKapsamı,
    pub bağlı_metin: Option<GrafikBağlıMetni>,
    pub anahtar_kare_animasyonları: Vec<GrafikAnahtarKareAnimasyonu>,
    pub zlevel: i32,
    pub z: f32,
    pub z2: f32,
    /// zrender `ignore`: çizim listesine ve isabet sınamasına katılmaz.
    pub yoksay: bool,
    /// zrender `invisible`: çizilmez fakat isabet sınamasına katılır.
    pub görünmez: bool,
    pub sessiz: bool,
    pub sürüklenebilir: bool,
    pub imleç: Option<String>,
}

impl GrafikÖğesi {
    fn yeni(içerik: GrafikÖğeİçeriği) -> Self {
        Self {
            kimlik: None,
            ad: None,
            bilgi: BTreeMap::new(),
            içerik,
            grup_boyutu: None,
            sınırlama: GrafikSınırlamaKipi::Tümü,
            yerleşim: GrafikYerleşimi::default(),
            dönüşüm: YerelDönüşüm::default(),
            kırpmalar: Vec::new(),
            stil: SahneStili::default(),
            durum_stilleri: BTreeMap::new(),
            durum: GörselDurum::Normal,
            odak: OdakKapsamı::Yok,
            bağlı_metin: None,
            anahtar_kare_animasyonları: Vec::new(),
            zlevel: 0,
            z: 0.0,
            z2: 0.0,
            yoksay: false,
            görünmez: false,
            sessiz: false,
            sürüklenebilir: false,
            imleç: None,
        }
    }

    pub fn grup(çocuklar: impl IntoIterator<Item = GrafikÖğesi>) -> Self {
        Self::yeni(GrafikÖğeİçeriği::Grup(çocuklar.into_iter().collect()))
    }

    pub fn grup_boyutu(mut self, genişlik: f32, yükseklik: f32) -> Self {
        self.grup_boyutu = Some((genişlik.max(0.0), yükseklik.max(0.0)));
        self
    }

    pub fn sınırlama(mut self, sınırlama: GrafikSınırlamaKipi) -> Self {
        self.sınırlama = sınırlama;
        self
    }

    pub fn şekil(şekil: SahneŞekli) -> Self {
        Self::yeni(GrafikÖğeİçeriği::Şekil(şekil))
    }

    pub fn dikdörtgen(kutu: Dikdörtgen) -> Self {
        Self::şekil(SahneŞekli::Dikdörtgen {
            kutu,
            yarıçap: [0.0; 4],
        })
    }

    pub fn metin(metin: impl Into<String>) -> Self {
        Self::yeni(GrafikÖğeİçeriği::Metin(
            SahneMetni::yeni(metin, (0.0, 0.0)),
        ))
    }

    pub fn resim(resim: SahneResmi) -> Self {
        Self::yeni(GrafikÖğeİçeriği::Resim(resim))
    }

    pub fn kimlik(mut self, kimlik: impl Into<String>) -> Self {
        self.kimlik = Some(kimlik.into());
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn bilgi(mut self, anahtar: impl Into<String>, değer: impl Into<EylemDeğeri>) -> Self {
        self.bilgi.insert(anahtar.into(), değer.into());
        self
    }

    pub fn bilgiler(mut self, bilgiler: impl IntoIterator<Item = (String, EylemDeğeri)>) -> Self {
        self.bilgi.extend(bilgiler);
        self
    }

    pub fn sol(mut self, sol: impl Into<YatayKonum>) -> Self {
        self.yerleşim.sol = Some(sol.into());
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.yerleşim.sağ = Some(sağ.into());
        self
    }

    pub fn üst(mut self, üst: impl Into<DikeyKonum>) -> Self {
        self.yerleşim.üst = Some(üst.into());
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.yerleşim.alt = Some(alt.into());
        self
    }

    pub fn dönüşüm(mut self, dönüşüm: YerelDönüşüm) -> Self {
        self.dönüşüm = dönüşüm;
        self
    }

    pub fn stil(mut self, stil: SahneStili) -> Self {
        self.stil = stil;
        self
    }

    pub fn bağlı_metin(mut self, metin: GrafikBağlıMetni) -> Self {
        self.bağlı_metin = Some(metin);
        self
    }

    pub fn anahtar_kare_animasyonu(mut self, animasyon: GrafikAnahtarKareAnimasyonu) -> Self {
        self.anahtar_kare_animasyonları.push(animasyon);
        self
    }

    pub fn kırp(mut self, kırpma: KırpmaYolu) -> Self {
        self.kırpmalar.push(kırpma);
        self
    }

    pub fn durum_stili(mut self, durum: GörselDurum, stil: SahneStilYaması) -> Self {
        self.durum_stilleri.insert(durum, stil);
        self
    }

    pub fn z(mut self, zlevel: i32, z: f32, z2: f32) -> Self {
        self.zlevel = zlevel;
        self.z = z;
        self.z2 = z2;
        self
    }

    pub fn yoksay(mut self, yoksay: bool) -> Self {
        self.yoksay = yoksay;
        self
    }

    pub fn görünmez(mut self, görünmez: bool) -> Self {
        self.görünmez = görünmez;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }

    pub fn sürüklenebilir(mut self, sürüklenebilir: bool) -> Self {
        self.sürüklenebilir = sürüklenebilir;
        self
    }

    pub fn imleç(mut self, imleç: impl Into<String>) -> Self {
        self.imleç = Some(imleç.into());
        self
    }

    pub fn köşe_yarıçapı(
        mut self,
        yarıçap: impl Into<crate::model::stil::KöşeYarıçapı>,
    ) -> Self {
        if let GrafikÖğeİçeriği::Şekil(SahneŞekli::Dikdörtgen {
            yarıçap: mevcut, ..
        }) = &mut self.içerik
        {
            *mevcut = yarıçap.into().0;
        }
        self
    }

    pub fn yazı_boyutu(mut self, boyut: f32) -> Self {
        if let GrafikÖğeİçeriği::Metin(metin) = &mut self.içerik {
            metin.boyut = boyut;
        }
        self
    }

    pub fn yazı_rengi(mut self, renk: impl Into<Renk>) -> Self {
        if let GrafikÖğeİçeriği::Metin(metin) = &mut self.içerik {
            metin.renk = renk.into();
        }
        self
    }

    pub fn kalın(mut self, kalın: bool) -> Self {
        if let GrafikÖğeİçeriği::Metin(metin) = &mut self.içerik {
            metin.kalın = kalın;
        }
        self
    }

    /// Serbest `graphic` metninin CSS `fontFamily` değeri.
    pub fn yazı_ailesi(mut self, aile: impl Into<String>) -> Self {
        if let GrafikÖğeİçeriği::Metin(metin) = &mut self.içerik {
            metin.aile = Some(aile.into());
        }
        self
    }

    /// Serbest `graphic` metninin `textAlign` ve `textVerticalAlign`
    /// değerleri.
    pub fn yazı_hizası(
        mut self,
        yatay: crate::cizim::YatayHiza,
        dikey: crate::cizim::DikeyHiza,
    ) -> Self {
        if let GrafikÖğeİçeriği::Metin(metin) = &mut self.içerik {
            metin.yatay = yatay;
            metin.dikey = dikey;
        }
        self
    }

    /// Graphic text `style.width` + `overflow: 'break'/'breakAll'`.
    pub fn yazı_sarma(mut self, genişlik: f32, her_yerden: bool) -> Self {
        if let GrafikÖğeİçeriği::Metin(metin) = &mut self.içerik {
            metin.en_çok_genişlik = Some(genişlik.max(0.0));
            metin.taşma = if her_yerden {
                crate::cizim::SahneMetinTaşması::HerYerdenKır
            } else {
                crate::cizim::SahneMetinTaşması::SözcükKır
            };
        }
        self
    }

    pub fn satır_yüksekliği(mut self, yükseklik: f32) -> Self {
        if let GrafikÖğeİçeriği::Metin(metin) = &mut self.içerik {
            metin.satır_yüksekliği = Some(yükseklik.max(0.0));
        }
        self
    }

    fn öğeyi_bul(&self, kimlik: &str) -> Option<&GrafikÖğesi> {
        if self.kimlik.as_deref() == Some(kimlik) {
            return Some(self);
        }
        let GrafikÖğeİçeriği::Grup(çocuklar) = &self.içerik else {
            return None;
        };
        çocuklar.iter().find_map(|çocuk| çocuk.öğeyi_bul(kimlik))
    }

    fn öğeyi_bul_mut(&mut self, kimlik: &str) -> Option<&mut GrafikÖğesi> {
        if self.kimlik.as_deref() == Some(kimlik) {
            return Some(self);
        }
        let GrafikÖğeİçeriği::Grup(çocuklar) = &mut self.içerik else {
            return None;
        };
        çocuklar
            .iter_mut()
            .find_map(|çocuk| çocuk.öğeyi_bul_mut(kimlik))
    }

    fn yoksaymayı_ayarla(&mut self, kimlik: &str, yoksay: bool) -> bool {
        if self.kimlik.as_deref() == Some(kimlik) {
            self.yoksay = yoksay;
            return true;
        }
        if let GrafikÖğeİçeriği::Grup(çocuklar) = &mut self.içerik {
            return çocuklar
                .iter_mut()
                .any(|çocuk| çocuk.yoksaymayı_ayarla(kimlik, yoksay));
        }
        false
    }

    fn doğrula(&self, kimlikler: &mut BTreeSet<String>) -> Result<(), BilesenHatasi> {
        if let Some(kimlik) = &self.kimlik
            && (kimlik.is_empty() || !kimlikler.insert(kimlik.clone()))
        {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "graphic.elements.id",
                ayrıntı: "graphic öğe kimlikleri boş olamaz ve benzersiz olmalıdır".to_owned(),
            });
        }
        let sayılar = [
            self.dönüşüm.x,
            self.dönüşüm.y,
            self.dönüşüm.ölçek_x,
            self.dönüşüm.ölçek_y,
            self.dönüşüm.dönüş,
            self.dönüşüm.köken_x,
            self.dönüşüm.köken_y,
            self.z,
            self.z2,
            self.grup_boyutu.map_or(0.0, |boyut| boyut.0),
            self.grup_boyutu.map_or(0.0, |boyut| boyut.1),
            self.stil.çizgi_kalınlığı,
            self.stil.opaklık,
        ];
        if sayılar.iter().any(|değer| !değer.is_finite()) || !self.dönüşüm.ek.sonlu_mu() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "graphic.elements",
                ayrıntı: "graphic dönüşüm, z ve stil sayıları sonlu olmalıdır".to_owned(),
            });
        }
        if self
            .grup_boyutu
            .is_some_and(|(genişlik, yükseklik)| genişlik < 0.0 || yükseklik < 0.0)
        {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "graphic.elements.width/height",
                ayrıntı: "graphic grup boyutları negatif olamaz".to_owned(),
            });
        }
        for animasyon in &self.anahtar_kare_animasyonları {
            if !animasyon.süre_ms.is_finite()
                || animasyon.süre_ms <= 0.0
                || !animasyon.gecikme_ms.is_finite()
                || animasyon.kareler.iter().any(|kare| {
                    !kare.yüzde.is_finite()
                        || !(0.0..=1.0).contains(&kare.yüzde)
                        || [kare.x, kare.y, kare.ölçek_x, kare.ölçek_y, kare.dönüş]
                            .into_iter()
                            .flatten()
                            .any(|değer| !değer.is_finite())
                })
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "graphic.elements.keyframeAnimation",
                    ayrıntı:
                        "graphic anahtar kare süreleri, yüzdeleri ve değerleri geçerli olmalıdır"
                            .to_owned(),
                });
            }
        }
        if let GrafikÖğeİçeriği::Grup(çocuklar) = &self.içerik {
            for çocuk in çocuklar {
                çocuk.doğrula(kimlikler)?;
            }
        }
        Ok(())
    }
}

/// Öğenin kendi sınır kutusunu tuval içinde konumlandıran ECharts
/// `left/right/top/bottom` alanları.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GrafikYerleşimi {
    pub sol: Option<YatayKonum>,
    pub sağ: Option<Uzunluk>,
    pub üst: Option<DikeyKonum>,
    pub alt: Option<Uzunluk>,
}

/// Bağlı `textContent` konumu (`textConfig.position`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum GrafikMetinKonumu {
    #[default]
    İç,
    Üst,
    Alt,
    Sol,
    Sağ,
    /// Ev sahibi öğenin yerel sınır kutusunun sol üstüne göre açık nokta.
    Değer(f32, f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct GrafikBağlıMetni {
    pub metin: String,
    pub konum: GrafikMetinKonumu,
    pub boyut: f32,
    pub renk: Renk,
    pub kalın: bool,
    pub sessiz: bool,
}

impl GrafikBağlıMetni {
    pub fn yeni(metin: impl Into<String>) -> Self {
        Self {
            metin: metin.into(),
            konum: GrafikMetinKonumu::İç,
            boyut: 12.0,
            renk: Renk::SİYAH,
            kalın: false,
            // zrender textContent, ev sahibinin olay hedefini örtmez.
            sessiz: true,
        }
    }

    pub fn konum(mut self, konum: GrafikMetinKonumu) -> Self {
        self.konum = konum;
        self
    }

    pub fn boyut(mut self, boyut: f32) -> Self {
        self.boyut = boyut;
        self
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = renk.into();
        self
    }

    pub fn kalın(mut self, kalın: bool) -> Self {
        self.kalın = kalın;
        self
    }

    pub fn sessiz(mut self, sessiz: bool) -> Self {
        self.sessiz = sessiz;
        self
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    fn iç_içe_bileşen() -> GrafikBileşeni {
        GrafikBileşeni::yeni().öğe(
            GrafikÖğesi::grup([
                GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(0.0, 0.0, 20.0, 10.0))
                    .kimlik("taşınan")
                    .sürüklenebilir(true),
                GrafikÖğesi::metin("eski").kimlik("değişen"),
                GrafikÖğesi::metin("silinecek").kimlik("silinen"),
            ])
            .kimlik("kök"),
        )
    }

    #[test]
    fn graphic_merge_replace_remove_ve_dizi_yolu_iç_içe_öğeleri_korur() {
        let mut grafik = iç_içe_bileşen();
        let yol = grafik.öğe_yolu("taşınan").expect("nested id yolu");
        assert_eq!(yol, vec![0, 0]);

        let yama = GrafikÖğeYaması::yeni()
            .konum(45.0, 30.0)
            .stil(SahneStilYaması {
                opaklık: Some(0.4),
                ..SahneStilYaması::default()
            });
        assert!(grafik.eylemi_uygula(GrafikÖğeEylemi::birleştir("taşınan", yama)));
        let taşınan = grafik.öğeyi_yolda(&yol).expect("yoldaki öğe");
        assert_eq!((taşınan.dönüşüm.x, taşınan.dönüşüm.y), (45.0, 30.0));
        assert_eq!(taşınan.stil.opaklık, 0.4);
        assert!(taşınan.sürüklenebilir);

        assert!(grafik.eylemi_uygula(GrafikÖğeEylemi::değiştir(
            "değişen",
            GrafikÖğesi::metin("yeni")
        )));
        let değişen = grafik.öğeyi_bul("değişen").expect("replace id korunur");
        let GrafikÖğeİçeriği::Metin(metin) = &değişen.içerik else {
            panic!("metin bekleniyordu")
        };
        assert_eq!(metin.metin, "yeni");

        assert!(grafik.eylemi_uygula(GrafikÖğeEylemi::kaldır("silinen")));
        assert!(grafik.öğeyi_bul("silinen").is_none());
        grafik.doğrula().expect("eylem sonrası model geçerli");
    }
}
