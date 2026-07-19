//! GL olmayan ECharts çekirdek `series.lines` option modeli.
//!
//! Geo bilinçli olarak bu tipe dahil değildir. Koordinat sistemi kurucuda
//! zorunludur; böylece ECharts'ın Geo öntanımı sessizce etkinleşemez.

use crate::hata::BilesenHatasi;
use crate::model::deger::VeriDeğeri;
use crate::model::seri::{EksenBağı, Sembol};
use crate::model::stil::{Etiket, ÇizgiStili};
use crate::renk::Renk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HatKoordinatSistemi {
    Kartezyen2B,
    Kutupsal,
    Takvim,
    Matris,
}

/// Bir `lines.coords` eksen değeri. Matrix kategori adı, calendar zamanı ve
/// sayısal Cartesian/polar değeri kayıpsız biçimde aynı modelde taşınır.
#[derive(Clone, Debug, PartialEq)]
pub enum HatKoordinatı {
    Sayı(f64),
    Metin(String),
    Zaman(i64),
}

impl HatKoordinatı {
    pub fn sayı(&self) -> Option<f64> {
        match self {
            Self::Sayı(değer) => Some(*değer),
            Self::Metin(değer) => değer.parse().ok(),
            Self::Zaman(değer) => Some(*değer as f64),
        }
    }

    pub fn metin(&self) -> Option<&str> {
        match self {
            Self::Metin(değer) => Some(değer),
            Self::Sayı(_) | Self::Zaman(_) => None,
        }
    }
}

impl HatKoordinatSistemi {
    /// ECharts option adını çözer. Geo kapsam dışıdır ve açık tanı üretir.
    pub fn çöz(ad: &str) -> Result<Self, BilesenHatasi> {
        match ad {
            "cartesian2d" => Ok(Self::Kartezyen2B),
            "polar" => Ok(Self::Kutupsal),
            "calendar" => Ok(Self::Takvim),
            "matrix" => Ok(Self::Matris),
            "geo" | "map" | "bmap" => Err(BilesenHatasi::Desteklenmeyen {
                özellik: "series.lines.coordinateSystem",
                ayrıntı: format!(
                    "`{ad}` Geo/Map kapsamı bu uyum hedefinde bilinçli olarak dışarıda"
                ),
            }),
            _ => Err(BilesenHatasi::GeçersizSeçenek {
                alan: "series.lines.coordinateSystem",
                ayrıntı: format!("bilinmeyen koordinat sistemi `{ad}`"),
            }),
        }
    }
}

impl From<f64> for HatKoordinatı {
    fn from(değer: f64) -> Self {
        Self::Sayı(değer)
    }
}

impl From<f32> for HatKoordinatı {
    fn from(değer: f32) -> Self {
        Self::Sayı(değer as f64)
    }
}

impl From<i32> for HatKoordinatı {
    fn from(değer: i32) -> Self {
        Self::Sayı(değer as f64)
    }
}

impl From<i64> for HatKoordinatı {
    fn from(değer: i64) -> Self {
        Self::Zaman(değer)
    }
}

impl From<&str> for HatKoordinatı {
    fn from(değer: &str) -> Self {
        Self::Metin(değer.to_owned())
    }
}

impl From<String> for HatKoordinatı {
    fn from(değer: String) -> Self {
        Self::Metin(değer)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HatNoktası {
    pub x: HatKoordinatı,
    pub y: HatKoordinatı,
}

impl HatNoktası {
    pub fn yeni(x: impl Into<HatKoordinatı>, y: impl Into<HatKoordinatı>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl<X: Into<HatKoordinatı>, Y: Into<HatKoordinatı>> From<(X, Y)> for HatNoktası {
    fn from((x, y): (X, Y)) -> Self {
        Self::yeni(x, y)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HatEfekti {
    pub göster: bool,
    pub dönem_sn: f32,
    pub gecikme_ms: f32,
    pub sabit_hız: f32,
    pub sembol: Sembol,
    pub sembol_boyutu: f32,
    pub döngü: bool,
    pub gidiş_dönüş: bool,
    pub iz_uzunluğu: f32,
    pub renk: Option<Renk>,
}

impl Default for HatEfekti {
    fn default() -> Self {
        Self {
            göster: false,
            dönem_sn: 4.0,
            gecikme_ms: 0.0,
            sabit_hız: 0.0,
            sembol: Sembol::Daire,
            sembol_boyutu: 3.0,
            döngü: true,
            gidiş_dönüş: false,
            iz_uzunluğu: 0.2,
            renk: None,
        }
    }
}

impl HatEfekti {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn dönem(mut self, saniye: f32) -> Self {
        self.dönem_sn = saniye.max(0.001);
        self
    }

    pub fn sabit_hız(mut self, piksel_saniye: f32) -> Self {
        self.sabit_hız = piksel_saniye.max(0.0);
        self
    }

    pub fn sembol(mut self, sembol: Sembol, boyut: f32) -> Self {
        self.sembol = sembol;
        self.sembol_boyutu = boyut.max(0.0);
        self
    }

    pub fn iz_uzunluğu(mut self, oran: f32) -> Self {
        self.iz_uzunluğu = oran.clamp(0.0, 1.0);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HatVerisi {
    pub ad: Option<String>,
    pub kaynak_adı: Option<String>,
    pub hedef_adı: Option<String>,
    pub koordinatlar: Vec<HatNoktası>,
    pub değer: VeriDeğeri,
    pub semboller: Option<[Sembol; 2]>,
    pub sembol_boyutları: Option<[f32; 2]>,
    pub çizgi_stili: Option<ÇizgiStili>,
    pub eğrilik: Option<f32>,
    pub etiket: Option<Etiket>,
    pub efekt: Option<HatEfekti>,
}

impl HatVerisi {
    pub fn yeni<N: Into<HatNoktası>>(koordinatlar: impl IntoIterator<Item = N>) -> Self {
        Self {
            ad: None,
            kaynak_adı: None,
            hedef_adı: None,
            koordinatlar: koordinatlar.into_iter().map(Into::into).collect(),
            değer: VeriDeğeri::Boş,
            semboller: None,
            sembol_boyutları: None,
            çizgi_stili: None,
            eğrilik: None,
            etiket: None,
            efekt: None,
        }
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn uç_adları(mut self, kaynak: impl Into<String>, hedef: impl Into<String>) -> Self {
        self.kaynak_adı = Some(kaynak.into());
        self.hedef_adı = Some(hedef.into());
        self
    }

    pub fn değer(mut self, değer: impl Into<VeriDeğeri>) -> Self {
        self.değer = değer.into();
        self
    }

    pub fn eğrilik(mut self, eğrilik: f32) -> Self {
        self.eğrilik = Some(eğrilik);
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = Some(stil);
        self
    }

    pub fn semboller(mut self, başlangıç: Sembol, bitiş: Sembol) -> Self {
        self.semboller = Some([başlangıç, bitiş]);
        self
    }

    pub fn sembol_boyutları(mut self, başlangıç: f32, bitiş: f32) -> Self {
        self.sembol_boyutları = Some([başlangıç.max(0.0), bitiş.max(0.0)]);
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = Some(etiket);
        self
    }

    pub fn efekt(mut self, efekt: HatEfekti) -> Self {
        self.efekt = Some(efekt);
        self
    }
}

#[derive(Clone, Debug)]
pub struct HatlarSerisi {
    pub ad: Option<String>,
    pub koordinat_sistemi: HatKoordinatSistemi,
    pub veri: Vec<HatVerisi>,
    pub eksen_bağı: EksenBağı,
    pub takvim_sırası: usize,
    pub matris_sırası: usize,
    pub semboller: [Sembol; 2],
    pub sembol_boyutları: [f32; 2],
    pub efekt: HatEfekti,
    pub çoklu_çizgi: bool,
    pub kırp: bool,
    pub etiket: Etiket,
    pub çizgi_stili: ÇizgiStili,
    pub büyük: bool,
    pub büyük_eşiği: usize,
    pub artımlı: usize,
    pub artımlı_eşik: usize,
    pub boyutlar: Vec<String>,
}

impl HatlarSerisi {
    pub fn yeni(koordinat_sistemi: HatKoordinatSistemi) -> Self {
        Self {
            ad: None,
            koordinat_sistemi,
            veri: Vec::new(),
            eksen_bağı: EksenBağı::default(),
            takvim_sırası: 0,
            matris_sırası: 0,
            semboller: [Sembol::Yok, Sembol::Yok],
            sembol_boyutları: [10.0, 10.0],
            efekt: HatEfekti::default(),
            çoklu_çizgi: false,
            kırp: true,
            etiket: Etiket::default(),
            çizgi_stili: ÇizgiStili {
                opaklık: 0.5,
                ..ÇizgiStili::default()
            },
            büyük: false,
            büyük_eşiği: 2_000,
            artımlı: 400,
            artımlı_eşik: 3_000,
            boyutlar: Vec::new(),
        }
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn veri(mut self, veri: impl IntoIterator<Item = HatVerisi>) -> Self {
        self.veri = veri.into_iter().collect();
        self
    }

    pub fn veri_ekle(mut self, veri: HatVerisi) -> Self {
        self.veri.push(veri);
        self
    }

    pub fn eksenler(mut self, x: usize, y: usize) -> Self {
        self.eksen_bağı = EksenBağı { x, y };
        self
    }

    pub fn takvim_sırası(mut self, sıra: usize) -> Self {
        self.takvim_sırası = sıra;
        self
    }

    pub fn matris_sırası(mut self, sıra: usize) -> Self {
        self.matris_sırası = sıra;
        self
    }

    pub fn semboller(mut self, başlangıç: Sembol, bitiş: Sembol) -> Self {
        self.semboller = [başlangıç, bitiş];
        self
    }

    pub fn sembol_boyutları(mut self, başlangıç: f32, bitiş: f32) -> Self {
        self.sembol_boyutları = [başlangıç.max(0.0), bitiş.max(0.0)];
        self
    }

    pub fn efekt(mut self, efekt: HatEfekti) -> Self {
        self.efekt = efekt;
        self
    }

    pub fn çoklu_çizgi(mut self, açık: bool) -> Self {
        self.çoklu_çizgi = açık;
        self
    }

    pub fn kırp(mut self, açık: bool) -> Self {
        self.kırp = açık;
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn büyük(mut self, açık: bool, eşik: usize) -> Self {
        self.büyük = açık;
        self.büyük_eşiği = eşik;
        self
    }

    /// ECharts'ın düz typed-array biçimini çözer:
    /// `nokta_sayısı, x, y, x, y, ...`.
    pub fn düz_veri(mut self, veri: &[f64]) -> Result<Self, BilesenHatasi> {
        let mut akış = veri.iter().copied();
        let mut sonuç = Vec::new();
        while let Some(sayı) = akış.next() {
            if !sayı.is_finite() || sayı < 0.0 || sayı.fract() != 0.0 {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "series.lines.data",
                    ayrıntı: "düz veri nokta sayısı negatif olmayan bir tamsayı olmalıdır"
                        .to_owned(),
                });
            }
            let nokta_sayısı = sayı as usize;
            let mut noktalar = Vec::with_capacity(nokta_sayısı);
            for _ in 0..nokta_sayısı {
                let (Some(x), Some(y)) = (akış.next(), akış.next()) else {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.lines.data",
                        ayrıntı: "düz veri koordinat çiftinin ortasında bitti".to_owned(),
                    });
                };
                noktalar.push(HatNoktası::yeni(x, y));
            }
            sonuç.push(HatVerisi::yeni(noktalar));
        }
        self.veri = sonuç;
        Ok(self)
    }
}

#[cfg(test)]
mod testler {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn geo_olmayan_koordinat_kurucuda_zorunludur() {
        for koordinat in [
            HatKoordinatSistemi::Kartezyen2B,
            HatKoordinatSistemi::Kutupsal,
            HatKoordinatSistemi::Takvim,
            HatKoordinatSistemi::Matris,
        ] {
            assert_eq!(HatlarSerisi::yeni(koordinat).koordinat_sistemi, koordinat);
        }
    }

    #[test]
    fn düz_typed_array_biçimi_çözülür() {
        let seri = HatlarSerisi::yeni(HatKoordinatSistemi::Kartezyen2B)
            .düz_veri(&[2.0, 1.0, 2.0, 3.0, 4.0, 3.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0])
            .unwrap();
        assert_eq!(seri.veri.len(), 2);
        assert_eq!(seri.veri.get(1).unwrap().koordinatlar.len(), 3);
    }

    #[test]
    fn eksik_düz_veri_tanı_üretir() {
        let sonuç =
            HatlarSerisi::yeni(HatKoordinatSistemi::Kartezyen2B).düz_veri(&[2.0, 1.0, 2.0, 3.0]);
        assert!(sonuç.is_err());
    }

    #[test]
    fn geo_adı_sessizce_kabul_edilmez() {
        assert!(matches!(
            HatKoordinatSistemi::çöz("geo"),
            Err(BilesenHatasi::Desteklenmeyen { .. })
        ));
        assert_eq!(
            HatKoordinatSistemi::çöz("matrix").unwrap(),
            HatKoordinatSistemi::Matris
        );
    }
}
