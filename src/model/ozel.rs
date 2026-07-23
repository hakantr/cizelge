//! ECharts `series-custom` için öğe-bazlı `renderItem` sözleşmesi.
//!
//! Eski doğrudan [`ÇizimYüzeyi`](crate::cizim::ÇizimYüzeyi) callback'i
//! kaynak uyumluluğu için `seri.rs` içinde kalır. Bu modül ise resmi
//! `renderItem(params, api)` yüzeyini, bütün çizim yüzeylerinin paylaştığı
//! sahne/graphic öğelerine bağlar.

use std::fmt;
use std::sync::Arc;

use crate::cizim::SahneStili;
use crate::grafik::kutupsal::KutupsalDüzen;
use crate::koordinat::{
    Dikdörtgen, Kartezyen2B, MatrisYerleşimi, TakvimYerleşimi, TekEksenYerleşimi,
};
use crate::model::deger::{VeriDeğeri, VeriÖğesi};
use crate::model::grafik_bileseni::GrafikÖğesi;
use crate::model::matris::MatrisAralığı;
use crate::renk::{Dolgu, Renk};

/// `series.coordinateSystem` seçenekleri. Geo, proje kapsamı gereği burada
/// bilerek yer almaz.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ÖzelSeriKoordinatSistemi {
    Yok,
    #[default]
    Kartezyen2B,
    Kutupsal,
    TekEksen,
    Takvim,
    Matris,
}

/// `params.coordSys` içindeki yüzeyden bağımsız yerleşim özeti.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ÖzelKoordinatTanımı {
    Yok {
        x: f32,
        y: f32,
        genişlik: f32,
        yükseklik: f32,
    },
    Kartezyen2B {
        x: f32,
        y: f32,
        genişlik: f32,
        yükseklik: f32,
    },
    Kutupsal {
        merkez_x: f32,
        merkez_y: f32,
        iç_yarıçap: f32,
        yarıçap: f32,
        başlangıç_açısı: f32,
        bitiş_açısı: f32,
    },
    TekEksen {
        x: f32,
        y: f32,
        genişlik: f32,
        yükseklik: f32,
    },
    Takvim {
        x: f32,
        y: f32,
        genişlik: f32,
        yükseklik: f32,
        hücre_genişliği: f32,
        hücre_yüksekliği: f32,
    },
    Matris {
        x: f32,
        y: f32,
        genişlik: f32,
        yükseklik: f32,
    },
}

impl ÖzelKoordinatTanımı {
    pub fn alan(self) -> Dikdörtgen {
        match self {
            Self::Yok {
                x,
                y,
                genişlik,
                yükseklik,
            }
            | Self::Kartezyen2B {
                x,
                y,
                genişlik,
                yükseklik,
            }
            | Self::TekEksen {
                x,
                y,
                genişlik,
                yükseklik,
            }
            | Self::Takvim {
                x,
                y,
                genişlik,
                yükseklik,
                ..
            }
            | Self::Matris {
                x,
                y,
                genişlik,
                yükseklik,
            } => Dikdörtgen::yeni(x, y, genişlik, yükseklik),
            Self::Kutupsal {
                merkez_x,
                merkez_y,
                yarıçap,
                ..
            } => Dikdörtgen::yeni(
                merkez_x - yarıçap,
                merkez_y - yarıçap,
                2.0 * yarıçap,
                2.0 * yarıçap,
            ),
        }
    }
}

/// `api.value` / `api.ordinalRawValue` boyut seçicisi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ÖzelBoyut {
    Sıra(usize),
    Ad(String),
}

impl From<usize> for ÖzelBoyut {
    fn from(değer: usize) -> Self {
        Self::Sıra(değer)
    }
}

impl From<&str> for ÖzelBoyut {
    fn from(değer: &str) -> Self {
        Self::Ad(değer.to_owned())
    }
}

impl From<String> for ÖzelBoyut {
    fn from(değer: String) -> Self {
        Self::Ad(değer)
    }
}

/// `api.visual(...)` dönüş değerleri.
#[derive(Clone, Debug, PartialEq)]
pub enum ÖzelGörselDeğeri {
    Renk(Dolgu),
    Opaklık(f32),
    Simge(String),
    SimgeBoyutu(f32),
}

/// `api.barLayout` içindeki bir seri/yığın bandı.
#[derive(Clone, Debug, PartialEq)]
pub struct ÖzelSütunYerleşimi {
    pub seri_sırası: usize,
    pub kaydırma: f32,
    pub genişlik: f32,
}

/// `params.actionType` ve `params.itemPayload` karşılığı.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ÖzelEylemBağlamı {
    pub eylem_türü: Option<String>,
    pub öğe_yükü: Option<String>,
}

/// Bir custom çizim turunda tüm veri öğelerinin paylaştığı mutable bağlamın
/// Rust karşılığı. `renderItem` closure'ları sıralı çağrıldığı için önceki
/// öğelerin hesapladığı yerleşim burada tutulabilir.
#[derive(Clone, Debug, Default)]
pub struct ÖzelTurBağlamı {
    pub işaretler: std::collections::BTreeMap<String, f64>,
    pub noktalar: std::collections::BTreeMap<String, Vec<(f32, f32)>>,
    pub metinler: std::collections::BTreeMap<String, String>,
}

/// Çizim katmanının gerçek koordinat nesnelerine geçici başvurusu.
#[derive(Clone, Copy)]
pub(crate) enum ÖzelKoordinatHaritası<'a> {
    Yok,
    Kartezyen2B(&'a Kartezyen2B),
    Kutupsal(&'a KutupsalDüzen),
    TekEksen(&'a TekEksenYerleşimi),
    Takvim(&'a TakvimYerleşimi),
    Matris(&'a MatrisYerleşimi),
}

impl fmt::Debug for ÖzelKoordinatHaritası<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Yok => "Yok",
            Self::Kartezyen2B(_) => "Kartezyen2B(..)",
            Self::Kutupsal(_) => "Kutupsal(..)",
            Self::TekEksen(_) => "TekEksen(..)",
            Self::Takvim(_) => "Takvim(..)",
            Self::Matris(_) => "Matris(..)",
        })
    }
}

/// ECharts `renderItem(params, api)` birleşik bağlamı.
pub struct ÖzelÖğeBağlamı<'a> {
    pub veri_sırası: usize,
    pub ham_veri_sırası: usize,
    pub içerideki_veri_sırası: usize,
    pub seri_sırası: usize,
    pub seri_adı: Option<&'a str>,
    pub seri_kimliği: Option<&'a str>,
    pub veri: &'a [VeriÖğesi],
    pub koordinat_sistemi: ÖzelSeriKoordinatSistemi,
    pub koordinat_tanımı: ÖzelKoordinatTanımı,
    pub görünüm_genişliği: f32,
    pub görünüm_yüksekliği: f32,
    pub kodlama: &'a [(String, Vec<usize>)],
    pub güncel_seri_sıraları: &'a [usize],
    pub sütun_yerleşimleri: &'a [ÖzelSütunYerleşimi],
    pub renk: Dolgu,
    pub opaklık: f32,
    pub ilerleme: f32,
    pub eylem: &'a ÖzelEylemBağlamı,
    /// Her çizim turunda bir kez hesaplanan kullanıcı yerleşimini paylaşır.
    pub tur: &'a std::sync::Mutex<ÖzelTurBağlamı>,
    pub(crate) harita: ÖzelKoordinatHaritası<'a>,
}

impl fmt::Debug for ÖzelÖğeBağlamı<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ÖzelÖğeBağlamı")
            .field("veri_sırası", &self.veri_sırası)
            .field("seri_sırası", &self.seri_sırası)
            .field("seri_adı", &self.seri_adı)
            .field("koordinat_sistemi", &self.koordinat_sistemi)
            .field("koordinat_tanımı", &self.koordinat_tanımı)
            .field("harita", &self.harita)
            .finish()
    }
}

impl ÖzelÖğeBağlamı<'_> {
    pub fn öğe(&self) -> Option<&VeriÖğesi> {
        self.veri.get(self.veri_sırası)
    }

    pub fn değer(&self, boyut: impl Into<ÖzelBoyut>) -> Option<VeriDeğeri> {
        let öğe = self.öğe()?;
        match boyut.into() {
            ÖzelBoyut::Ad(ad) => öğe.boyut(&ad).cloned(),
            ÖzelBoyut::Sıra(sıra) => veri_boyutu(öğe, sıra),
        }
    }

    pub fn sayısal_değer(&self, boyut: impl Into<ÖzelBoyut>) -> Option<f64> {
        self.değer(boyut).and_then(|değer| değer.sayı())
    }

    /// Kategori boyutunun formatlanmamış ham değerini döndürür.
    pub fn sıralı_ham_değer(&self, boyut: impl Into<ÖzelBoyut>) -> Option<String> {
        self.değer(boyut).as_ref().map(veri_değeri_metni)
    }

    /// `api.coord`: polar için `[x, y, radius, angle]`, diğer iki boyutlu
    /// koordinatlarda `[x, y]` döndürür.
    pub fn koordinat(&self, değerler: &[f64]) -> Option<Vec<f32>> {
        match self.harita {
            ÖzelKoordinatHaritası::Yok => None,
            ÖzelKoordinatHaritası::Kartezyen2B(k) => {
                let (x, y) = k.nokta(*değerler.first()?, *değerler.get(1)?);
                Some(vec![x, y])
            }
            ÖzelKoordinatHaritası::Kutupsal(k) => {
                // Resmi custom API girdi sırası `[radius, angle]`dır.
                let radyal = *değerler.first()?;
                let açısal = *değerler.get(1)?;
                let (x, y) = k.nokta(açısal, radyal);
                // ECharts Custom API açıyı matematiksel kutupsal yönde
                // (pozitif Y yukarı) verir; Cizelge'nin iç düzen açısı ise
                // ekran uzayındadır (pozitif Y aşağı). Resmî renderItem
                // örneklerindeki `startAngle: -(coord[3] + ...)` ancak bu
                // işaret dönüşümüyle aynı sektörü üretir.
                Some(vec![x, y, k.yarıçapa(radyal), -k.açı(açısal)])
            }
            ÖzelKoordinatHaritası::TekEksen(k) => {
                let (x, y) = k.veriden_noktaya(*değerler.first()?);
                Some(vec![x, y])
            }
            ÖzelKoordinatHaritası::Takvim(k) => {
                let (x, y) = k.veriden_noktaya(*değerler.first()?)?;
                Some(vec![x, y])
            }
            ÖzelKoordinatHaritası::Matris(k) => {
                let x = sayıyı_sıraya(*değerler.first()?)?;
                let y = sayıyı_sıraya(*değerler.get(1)?)?;
                let (x, y) = k.veriden_noktaya(x, y)?;
                Some(vec![x, y])
            }
        }
    }

    /// `api.size(dataSize, dataItem)`. `veri_değerleri` verilmezse geçerli
    /// öğenin ilk boyutları merkez değer olarak kullanılır.
    pub fn boyut(&self, veri_boyutları: &[f64], veri_değerleri: Option<&[f64]>) -> Vec<f32> {
        let merkez = veri_değerleri
            .map(Vec::from)
            .unwrap_or_else(|| vec![0.0; veri_boyutları.len().max(2)]);
        match self.harita {
            ÖzelKoordinatHaritası::Yok => veri_boyutları.iter().map(|v| *v as f32).collect(),
            ÖzelKoordinatHaritası::Kartezyen2B(k) => vec![
                eksen_boyutu(
                    &k.x,
                    veri_boyutları.first().copied().unwrap_or(0.0),
                    merkez.first().copied().unwrap_or(0.0),
                ),
                eksen_boyutu(
                    &k.y,
                    veri_boyutları.get(1).copied().unwrap_or(0.0),
                    merkez.get(1).copied().unwrap_or(0.0),
                ),
            ],
            ÖzelKoordinatHaritası::Kutupsal(k) => {
                let radyal_merkez = merkez.first().copied().unwrap_or(0.0);
                let açısal_merkez = merkez.get(1).copied().unwrap_or(0.0);
                let radyal_boyut = veri_boyutları.first().copied().unwrap_or(0.0);
                let açısal_boyut = veri_boyutları.get(1).copied().unwrap_or(0.0);
                vec![
                    (k.yarıçapa(radyal_merkez + radyal_boyut / 2.0)
                        - k.yarıçapa(radyal_merkez - radyal_boyut / 2.0))
                    .abs(),
                    açı_farkı(
                        k.açı(açısal_merkez + açısal_boyut / 2.0),
                        k.açı(açısal_merkez - açısal_boyut / 2.0),
                    ),
                ]
            }
            ÖzelKoordinatHaritası::TekEksen(k) => {
                let değer = merkez.first().copied().unwrap_or(0.0);
                let boyut = veri_boyutları.first().copied().unwrap_or(0.0);
                let a = k.veriden_noktaya(değer - boyut / 2.0);
                let b = k.veriden_noktaya(değer + boyut / 2.0);
                vec![((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()]
            }
            ÖzelKoordinatHaritası::Takvim(k) => {
                vec![k.hücre_genişliği, k.hücre_yüksekliği]
            }
            ÖzelKoordinatHaritası::Matris(k) => {
                let x = merkez.first().copied().and_then(sayıyı_sıraya).unwrap_or(0);
                let y = merkez.get(1).copied().and_then(sayıyı_sıraya).unwrap_or(0);
                k.veriden_yerleşime(&MatrisAralığı::from(x), &MatrisAralığı::from(y), true)
                    .map_or_else(
                        || vec![0.0, 0.0],
                        |alan| vec![alan.genişlik, alan.yükseklik],
                    )
            }
        }
    }

    pub fn genişlik(&self) -> f32 {
        self.görünüm_genişliği
    }

    pub fn yükseklik(&self) -> f32 {
        self.görünüm_yüksekliği
    }

    pub fn görsel(&self, ad: &str) -> Option<ÖzelGörselDeğeri> {
        match ad {
            "color" | "renk" => Some(ÖzelGörselDeğeri::Renk(self.renk.clone())),
            "opacity" | "opaklık" => Some(ÖzelGörselDeğeri::Opaklık(self.opaklık)),
            _ => None,
        }
    }

    /// ECharts'ın eski `api.style()` yardımcısının sahne stili karşılığı.
    pub fn stil(&self) -> SahneStili {
        let mut stil = SahneStili {
            dolgu: Some(self.renk.clone()),
            opaklık: self.opaklık,
            ..SahneStili::default()
        };
        if let Some(öğe_stili) = self.öğe().and_then(|öğe| öğe.stil.as_ref()) {
            if let Some(renk) = &öğe_stili.renk {
                stil.dolgu = Some(renk.clone());
            }
            stil.çizgi_rengi = öğe_stili.kenarlık_rengi;
            stil.çizgi_kalınlığı = öğe_stili.kenarlık_kalınlığı;
            stil.çizgi_türü = öğe_stili.kenarlık_türü;
            stil.opaklık *= öğe_stili.opaklık.unwrap_or(1.0);
            stil.gölge_rengi = öğe_stili.gölge_rengi;
            stil.gölge_bulanıklığı = öğe_stili.gölge_bulanıklığı;
        }
        stil
    }

    pub fn sütun_yerleşimi(&self, seri_sırası: usize) -> Option<&ÖzelSütunYerleşimi> {
        self.sütun_yerleşimleri
            .iter()
            .find(|yerleşim| yerleşim.seri_sırası == seri_sırası)
    }

    pub fn güncel_seri_sıraları(&self) -> &[usize] {
        self.güncel_seri_sıraları
    }

    /// `api.font`: zrender'ın CSS font kısa gösterimini üretir.
    pub fn yazı_tipi(&self, boyut: f32, kalın: bool, aile: Option<&str>) -> String {
        format!(
            "{}{}px {}",
            if kalın { "bold " } else { "" },
            boyut.max(0.0),
            aile.unwrap_or("sans-serif")
        )
    }
}

/// `renderItem` sonucu üzerinde enter/update/during sözleşmesi.
pub type ÖzelSırasında = Arc<dyn Fn(&mut GrafikÖğesi, f32) + Send + Sync>;

#[derive(Clone)]
pub struct ÖzelÖğeÇıktısı {
    pub öğe: GrafikÖğesi,
    /// Veri diff anahtarı; yoksa data `id/name/index` sırası kullanılır.
    pub anahtar: Option<String>,
    /// Enter başlangıç öğesi. Aynı tipteki şekil/dönüşüm/stil alanları
    /// `ilerleme` boyunca ara değerlenir.
    pub girişten: Option<GrafikÖğesi>,
    /// Leave hedefi; veri farkı ve animasyon kayıtlarında korunur.
    pub çıkışa: Option<GrafikÖğesi>,
    pub geçiş: Vec<String>,
    pub biçim_dönüşümü: bool,
    pub bilgi: Option<String>,
    pub sırasında: Option<ÖzelSırasında>,
}

impl fmt::Debug for ÖzelÖğeÇıktısı {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ÖzelÖğeÇıktısı")
            .field("öğe", &self.öğe)
            .field("anahtar", &self.anahtar)
            .field("girişten", &self.girişten)
            .field("çıkışa", &self.çıkışa)
            .field("geçiş", &self.geçiş)
            .field("biçim_dönüşümü", &self.biçim_dönüşümü)
            .field("bilgi", &self.bilgi)
            .field("sırasında", &self.sırasında.as_ref().map(|_| "İşlev(..)"))
            .finish()
    }
}

impl ÖzelÖğeÇıktısı {
    pub fn yeni(öğe: GrafikÖğesi) -> Self {
        Self {
            öğe,
            anahtar: None,
            girişten: None,
            çıkışa: None,
            geçiş: Vec::new(),
            biçim_dönüşümü: false,
            bilgi: None,
            sırasında: None,
        }
    }

    pub fn anahtar(mut self, anahtar: impl Into<String>) -> Self {
        self.anahtar = Some(anahtar.into());
        self
    }

    pub fn girişten(mut self, öğe: GrafikÖğesi) -> Self {
        self.girişten = Some(öğe);
        self
    }

    pub fn çıkışa(mut self, öğe: GrafikÖğesi) -> Self {
        self.çıkışa = Some(öğe);
        self
    }

    pub fn geçiş(mut self, alanlar: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.geçiş = alanlar.into_iter().map(Into::into).collect();
        self
    }

    pub fn biçim_dönüşümü(mut self, etkin: bool) -> Self {
        self.biçim_dönüşümü = etkin;
        self
    }

    pub fn bilgi(mut self, bilgi: impl Into<String>) -> Self {
        self.bilgi = Some(bilgi.into());
        self
    }

    pub fn sırasında(
        mut self,
        işlev: impl Fn(&mut GrafikÖğesi, f32) + Send + Sync + 'static,
    ) -> Self {
        self.sırasında = Some(Arc::new(işlev));
        self
    }
}

impl From<GrafikÖğesi> for ÖzelÖğeÇıktısı {
    fn from(öğe: GrafikÖğesi) -> Self {
        Self::yeni(öğe)
    }
}

pub type ÖzelÖğeÇizimi =
    Arc<dyn for<'a> Fn(&ÖzelÖğeBağlamı<'a>) -> Option<ÖzelÖğeÇıktısı> + Send + Sync>;

pub(crate) fn veri_boyutu(öğe: &VeriÖğesi, sıra: usize) -> Option<VeriDeğeri> {
    match &öğe.değer {
        VeriDeğeri::Çift(değerler) => match sıra {
            0 => Some(VeriDeğeri::Sayı(değerler[0])),
            1 => Some(VeriDeğeri::Sayı(değerler[1])),
            _ => None,
        },
        VeriDeğeri::Dizi(değerler) => değerler.get(sıra).copied().map(VeriDeğeri::Sayı),
        VeriDeğeri::KarmaDizi(değerler) => değerler.get(sıra).cloned(),
        _ => (sıra == 0).then(|| öğe.değer.clone()),
    }
}

fn veri_değeri_metni(değer: &VeriDeğeri) -> String {
    match değer {
        VeriDeğeri::Boş => String::new(),
        VeriDeğeri::Sayı(v) => v.to_string(),
        VeriDeğeri::Çift(v) => format!("{},{}", v[0], v[1]),
        VeriDeğeri::Dizi(v) => v
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
        VeriDeğeri::KarmaDizi(v) => v
            .iter()
            .map(veri_değeri_metni)
            .collect::<Vec<_>>()
            .join(","),
        VeriDeğeri::Metin(v) => v.clone(),
        VeriDeğeri::Mantıksal(v) => v.to_string(),
        VeriDeğeri::Zaman(v) => v.to_string(),
    }
}

fn sayıyı_sıraya(değer: f64) -> Option<isize> {
    (değer.is_finite() && değer.fract().abs() <= 1e-9).then_some(değer as isize)
}

fn eksen_boyutu(eksen: &crate::koordinat::ÇalışmaEkseni, boyut: f64, merkez: f64) -> f32 {
    if eksen.ölçek.kategorik_mi() {
        eksen.bant_genişliği()
    } else {
        (eksen.veriden_piksele(merkez + boyut / 2.0) - eksen.veriden_piksele(merkez - boyut / 2.0))
            .abs()
    }
}

fn açı_farkı(a: f32, b: f32) -> f32 {
    let fark = (a - b).abs().rem_euclid(std::f32::consts::TAU);
    fark.min(std::f32::consts::TAU - fark)
}

/// Düz renk kolaylığı; fixture ve kullanıcı kodunda `api.visual('color')`
/// dönüşünü açmak için.
impl ÖzelGörselDeğeri {
    pub fn renk(self) -> Option<Dolgu> {
        match self {
            Self::Renk(renk) => Some(renk),
            _ => None,
        }
    }

    pub fn düz_renk(self) -> Option<Renk> {
        match self {
            Self::Renk(Dolgu::Düz(renk)) => Some(renk),
            _ => None,
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::koordinat::ÇalışmaEkseni;
    use crate::model::eksen::{Eksen, EksenKonumu};
    use crate::olcek::{AralıkÖlçeği, Ölçek};

    #[test]
    fn kartezyen_coord_ve_size_aynı_eksenleri_kullanır() {
        let x = ÇalışmaEkseni::yeni(
            Eksen::değer(),
            Ölçek::Aralık(AralıkÖlçeği::kur(
                [0.0, 10.0],
                Some(0.0),
                Some(10.0),
                false,
                5,
                None,
                None,
            )),
            [0.0, 100.0],
            EksenKonumu::Alt,
        );
        let y = ÇalışmaEkseni::yeni(
            Eksen::değer(),
            Ölçek::Aralık(AralıkÖlçeği::kur(
                [0.0, 10.0],
                Some(0.0),
                Some(10.0),
                false,
                5,
                None,
                None,
            )),
            [100.0, 0.0],
            EksenKonumu::Sol,
        );
        let kartezyen = Kartezyen2B {
            x,
            y,
            alan: Dikdörtgen::yeni(0.0, 0.0, 100.0, 100.0),
        };
        let veri = vec![VeriÖğesi::yeni(vec![5.0, 5.0])];
        let eylem = ÖzelEylemBağlamı::default();
        let tur = std::sync::Mutex::new(ÖzelTurBağlamı::default());
        let bağlam = ÖzelÖğeBağlamı {
            veri_sırası: 0,
            ham_veri_sırası: 0,
            içerideki_veri_sırası: 0,
            seri_sırası: 0,
            seri_adı: None,
            seri_kimliği: None,
            veri: &veri,
            koordinat_sistemi: ÖzelSeriKoordinatSistemi::Kartezyen2B,
            koordinat_tanımı: ÖzelKoordinatTanımı::Kartezyen2B {
                x: 0.0,
                y: 0.0,
                genişlik: 100.0,
                yükseklik: 100.0,
            },
            görünüm_genişliği: 100.0,
            görünüm_yüksekliği: 100.0,
            kodlama: &[],
            güncel_seri_sıraları: &[0],
            sütun_yerleşimleri: &[],
            renk: Dolgu::Düz(Renk::SİYAH),
            opaklık: 1.0,
            ilerleme: 1.0,
            eylem: &eylem,
            tur: &tur,
            harita: ÖzelKoordinatHaritası::Kartezyen2B(&kartezyen),
        };
        assert_eq!(bağlam.koordinat(&[5.0, 5.0]), Some(vec![50.0, 50.0]));
        assert_eq!(
            bağlam.boyut(&[2.0, 4.0], Some(&[5.0, 5.0])),
            vec![20.0, 40.0]
        );
    }
}
