//! Kimlikli hiyerarşik sahne grafiği — zrender `Storage`/`Element` karşılığı.
//!
//! Düğümler yerel koordinatta saklanır; dünya matrisi çizim ve isabet
//! anında hesaplanır. Görüntü listesi `zlevel/z/z2` ve eklenme sırasıyla
//! kararlı biçimde sıralanır. Aynı ağaç Kayıt, SVG, Piksel ve gpui
//! yüzeylerine gönderilebilir.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::cizim::donusum::AfinMatris;
use crate::cizim::yuzey::{
    DikeyHiza, YatayHiza, Yol, YolKomutu, daire_yolu, dilim_yolu, ÇizimYüzeyi,
};
use crate::hata::BilesenHatasi;
use crate::koordinat::Dikdörtgen;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

/// zrender `Transformable` alanları.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct YerelDönüşüm {
    pub x: f32,
    pub y: f32,
    pub ölçek_x: f32,
    pub ölçek_y: f32,
    pub dönüş: f32,
    pub köken_x: f32,
    pub köken_y: f32,
    /// translate/rotate/scale dışında kalan açık affine dönüşüm.
    pub ek: AfinMatris,
}

impl Default for YerelDönüşüm {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            ölçek_x: 1.0,
            ölçek_y: 1.0,
            dönüş: 0.0,
            köken_x: 0.0,
            köken_y: 0.0,
            ek: AfinMatris::BİRİM,
        }
    }
}

impl YerelDönüşüm {
    pub fn matris(self) -> AfinMatris {
        AfinMatris::ötele(self.x, self.y)
            .çarp(AfinMatris::ötele(self.köken_x, self.köken_y))
            .çarp(AfinMatris::döndür(self.dönüş))
            .çarp(AfinMatris::ölçekle(self.ölçek_x, self.ölçek_y))
            .çarp(AfinMatris::ötele(-self.köken_x, -self.köken_y))
            .çarp(self.ek)
    }

    pub fn ötele(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn ölçekle(mut self, x: f32, y: f32) -> Self {
        self.ölçek_x = x;
        self.ölçek_y = y;
        self
    }

    pub fn döndür(mut self, radyan: f32) -> Self {
        self.dönüş = radyan;
        self
    }

    pub fn köken(mut self, x: f32, y: f32) -> Self {
        self.köken_x = x;
        self.köken_y = y;
        self
    }
}

/// Primitive ve bileşik path geometrileri.
#[derive(Clone, Debug, PartialEq)]
pub enum SahneŞekli {
    Yol(Yol),
    Çizgi {
        başlangıç: (f32, f32),
        bitiş: (f32, f32),
    },
    ÇokluÇizgi(Vec<(f32, f32)>),
    Çokgen(Vec<(f32, f32)>),
    Dikdörtgen {
        kutu: Dikdörtgen,
        yarıçap: [f32; 4],
    },
    Daire {
        merkez: (f32, f32),
        yarıçap: f32,
    },
    Halka {
        merkez: (f32, f32),
        iç_yarıçap: f32,
        dış_yarıçap: f32,
    },
    Dilim {
        merkez: (f32, f32),
        iç_yarıçap: f32,
        dış_yarıçap: f32,
        başlangıç_açısı: f32,
        bitiş_açısı: f32,
    },
    Yay {
        merkez: (f32, f32),
        yarıçap: f32,
        başlangıç_açısı: f32,
        bitiş_açısı: f32,
        saat_yönü: bool,
    },
    KübikBezier {
        başlangıç: (f32, f32),
        k1: (f32, f32),
        k2: (f32, f32),
        bitiş: (f32, f32),
    },
    Bileşik(Vec<SahneŞekli>),
}

impl SahneŞekli {
    pub fn yol(&self) -> Yol {
        match self {
            Self::Yol(yol) => yol.clone(),
            Self::Çizgi {
                başlangıç, bitiş
            } => {
                let mut yol = Yol::yeni();
                yol.taşı(*başlangıç);
                yol.çiz(*bitiş);
                yol
            }
            Self::ÇokluÇizgi(noktalar) => noktaların_yolu(noktalar, false),
            Self::Çokgen(noktalar) => noktaların_yolu(noktalar, true),
            Self::Dikdörtgen { kutu, yarıçap } => yuvarlak_dikdörtgen_yolu(*kutu, *yarıçap),
            Self::Daire { merkez, yarıçap } => daire_yolu(*merkez, *yarıçap),
            Self::Halka {
                merkez,
                iç_yarıçap,
                dış_yarıçap,
            } => dilim_yolu(
                *merkez,
                *iç_yarıçap,
                *dış_yarıçap,
                0.0,
                std::f32::consts::TAU * 0.9999,
            ),
            Self::Dilim {
                merkez,
                iç_yarıçap,
                dış_yarıçap,
                başlangıç_açısı,
                bitiş_açısı,
            } => dilim_yolu(
                *merkez,
                *iç_yarıçap,
                *dış_yarıçap,
                *başlangıç_açısı,
                *bitiş_açısı,
            ),
            Self::Yay {
                merkez,
                yarıçap,
                başlangıç_açısı,
                bitiş_açısı,
                saat_yönü,
            } => {
                let başlangıç = (
                    merkez.0 + yarıçap * başlangıç_açısı.cos(),
                    merkez.1 + yarıçap * başlangıç_açısı.sin(),
                );
                let bitiş = (
                    merkez.0 + yarıçap * bitiş_açısı.cos(),
                    merkez.1 + yarıçap * bitiş_açısı.sin(),
                );
                let açıklık = açı_açıklığı(*başlangıç_açısı, *bitiş_açısı, *saat_yönü);
                let mut yol = Yol::yeni();
                yol.taşı(başlangıç);
                yol.yay(
                    *yarıçap,
                    açıklık.abs() > std::f32::consts::PI,
                    *saat_yönü,
                    bitiş,
                );
                yol
            }
            Self::KübikBezier {
                başlangıç,
                k1,
                k2,
                bitiş,
            } => {
                let mut yol = Yol::yeni();
                yol.taşı(*başlangıç);
                yol.kübik(*k1, *k2, *bitiş);
                yol
            }
            Self::Bileşik(şekiller) => {
                let mut yol = Yol::yeni();
                for şekil in şekiller {
                    yol.komutlar.extend(şekil.yol().komutlar);
                }
                yol
            }
        }
    }

    pub fn sınır_kutusu(&self) -> Option<Dikdörtgen> {
        self.yol().sınır_kutusu()
    }

    pub fn içeriyor_mu(&self, nokta: (f32, f32), çizgi_kalınlığı: f32) -> bool {
        match self {
            Self::Dikdörtgen { kutu, .. } => kutu.içeriyor_mu(nokta),
            Self::Daire { merkez, yarıçap } => {
                uzaklık_kare(nokta, *merkez) <= yarıçap * yarıçap
            }
            Self::Halka {
                merkez,
                iç_yarıçap,
                dış_yarıçap,
            } => {
                let uzaklık = uzaklık_kare(nokta, *merkez).sqrt();
                uzaklık >= *iç_yarıçap && uzaklık <= *dış_yarıçap
            }
            Self::Dilim {
                merkez,
                iç_yarıçap,
                dış_yarıçap,
                başlangıç_açısı,
                bitiş_açısı,
            } => {
                let uzaklık = uzaklık_kare(nokta, *merkez).sqrt();
                let açı = (nokta.1 - merkez.1).atan2(nokta.0 - merkez.0);
                uzaklık >= *iç_yarıçap
                    && uzaklık <= *dış_yarıçap
                    && açı_arasında_mı(açı, *başlangıç_açısı, *bitiş_açısı)
            }
            Self::Çizgi {
                başlangıç, bitiş
            } => doğruya_uzaklık(nokta, *başlangıç, *bitiş) <= çizgi_kalınlığı.max(2.0) / 2.0,
            Self::Yay { .. } | Self::ÇokluÇizgi(_) | Self::KübikBezier { .. } | Self::Yol(_) => {
                let alt_yollar = yolu_düzleştir(&self.yol(), 0.75);
                alt_yollar.iter().any(|noktalar| {
                    çokgen_içeriyor(noktalar, nokta)
                        || çoklu_çizgi_uzaklığı(noktalar, nokta) <= çizgi_kalınlığı.max(2.0) / 2.0
                })
            }
            Self::Çokgen(noktalar) => çokgen_içeriyor(noktalar, nokta),
            Self::Bileşik(şekiller) => şekiller
                .iter()
                .any(|şekil| şekil.içeriyor_mu(nokta, çizgi_kalınlığı)),
        }
    }
}

/// Bir düğümün normal çizim stili.
#[derive(Clone, Debug, PartialEq)]
pub struct SahneStili {
    pub dolgu: Option<Dolgu>,
    pub çizgi_rengi: Option<Renk>,
    pub çizgi_kalınlığı: f32,
    pub çizgi_türü: ÇizgiTürü,
    pub opaklık: f32,
    pub gölge_rengi: Option<Renk>,
    pub gölge_bulanıklığı: f32,
}

impl Default for SahneStili {
    fn default() -> Self {
        Self {
            dolgu: Some(Dolgu::Düz(Renk::SİYAH)),
            çizgi_rengi: None,
            çizgi_kalınlığı: 1.0,
            çizgi_türü: ÇizgiTürü::Düz,
            opaklık: 1.0,
            gölge_rengi: None,
            gölge_bulanıklığı: 0.0,
        }
    }
}

/// Durum stilinde yalnız sağlanan değerler normal stili örter.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SahneStilYaması {
    pub dolgu: Option<Dolgu>,
    pub çizgi_rengi: Option<Renk>,
    pub çizgi_kalınlığı: Option<f32>,
    pub çizgi_türü: Option<ÇizgiTürü>,
    pub opaklık: Option<f32>,
}

impl SahneStilYaması {
    fn uygula(&self, stil: &mut SahneStili) {
        if let Some(dolgu) = &self.dolgu {
            stil.dolgu = Some(dolgu.clone());
        }
        if let Some(renk) = self.çizgi_rengi {
            stil.çizgi_rengi = Some(renk);
        }
        if let Some(kalınlık) = self.çizgi_kalınlığı {
            stil.çizgi_kalınlığı = kalınlık;
        }
        if let Some(tür) = self.çizgi_türü {
            stil.çizgi_türü = tür;
        }
        if let Some(opaklık) = self.opaklık {
            stil.opaklık = opaklık;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum GörselDurum {
    #[default]
    Normal,
    Vurgu,
    Bulanık,
    Seçili,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OdakKapsamı {
    #[default]
    Yok,
    Seri,
    KoordinatSistemi,
    Tümü,
}

/// Yerel şekil ve dönüşümüyle bir clip path.
#[derive(Clone, Debug, PartialEq)]
pub struct KırpmaYolu {
    pub şekil: SahneŞekli,
    pub dönüşüm: YerelDönüşüm,
}

impl KırpmaYolu {
    pub fn yeni(şekil: SahneŞekli) -> Self {
        Self {
            şekil,
            dönüşüm: YerelDönüşüm::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SahneMetni {
    pub metin: String,
    pub konum: (f32, f32),
    pub yatay: YatayHiza,
    pub dikey: DikeyHiza,
    pub boyut: f32,
    pub renk: Renk,
    pub kalın: bool,
    /// CSS `fontFamily`; `None` yüzeyin sans-serif öntanımlısını kullanır.
    pub aile: Option<String>,
}

impl SahneMetni {
    pub fn yeni(metin: impl Into<String>, konum: (f32, f32)) -> Self {
        Self {
            metin: metin.into(),
            konum,
            yatay: YatayHiza::Sol,
            dikey: DikeyHiza::Üst,
            boyut: 12.0,
            renk: Renk::SİYAH,
            kalın: false,
            aile: None,
        }
    }

    pub fn sınır_kutusu(&self) -> Dikdörtgen {
        let genişlik = self.metin.chars().count() as f32 * self.boyut * 0.6;
        let yükseklik = self.boyut * crate::cizim::yuzey::SATIR_ORANI;
        let x = match self.yatay {
            YatayHiza::Sol => self.konum.0,
            YatayHiza::Orta => self.konum.0 - genişlik / 2.0,
            YatayHiza::Sağ => self.konum.0 - genişlik,
        };
        let y = match self.dikey {
            DikeyHiza::Üst => self.konum.1,
            DikeyHiza::Orta => self.konum.1 - yükseklik / 2.0,
            DikeyHiza::Alt => self.konum.1 - yükseklik,
        };
        Dikdörtgen::yeni(x, y, genişlik, yükseklik)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SahneResmi {
    pub kaynak: String,
    pub kutu: Dikdörtgen,
    /// Resim yüklenene kadar ve kayıt yüzeyinde kullanılan belirlenimci renk.
    pub yer_tutucu: Dolgu,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SahneÖğesi {
    Grup(Vec<SahneDüğümü>),
    Şekil(SahneŞekli),
    Metin(SahneMetni),
    Resim(SahneResmi),
}

/// Sahne ağacının tek kimlikli öğesi.
#[derive(Clone, Debug, PartialEq)]
pub struct SahneDüğümü {
    pub kimlik: String,
    pub öğe: SahneÖğesi,
    pub dönüşüm: YerelDönüşüm,
    pub kırpmalar: Vec<KırpmaYolu>,
    pub stil: SahneStili,
    pub durum_stilleri: BTreeMap<GörselDurum, SahneStilYaması>,
    pub durum: GörselDurum,
    pub odak: OdakKapsamı,
    pub zlevel: i32,
    pub z: f32,
    pub z2: f32,
    pub görünür: bool,
    pub sessiz: bool,
    pub sürüklenebilir: bool,
    pub imleç: Option<String>,
}

impl SahneDüğümü {
    pub fn grup(kimlik: impl Into<String>) -> Self {
        Self::yeni(kimlik, SahneÖğesi::Grup(Vec::new()))
    }

    pub fn şekil(kimlik: impl Into<String>, şekil: SahneŞekli) -> Self {
        Self::yeni(kimlik, SahneÖğesi::Şekil(şekil))
    }

    pub fn metin(kimlik: impl Into<String>, metin: SahneMetni) -> Self {
        Self::yeni(kimlik, SahneÖğesi::Metin(metin))
    }

    pub fn resim(kimlik: impl Into<String>, resim: SahneResmi) -> Self {
        Self::yeni(kimlik, SahneÖğesi::Resim(resim))
    }

    fn yeni(kimlik: impl Into<String>, öğe: SahneÖğesi) -> Self {
        Self {
            kimlik: kimlik.into(),
            öğe,
            dönüşüm: YerelDönüşüm::default(),
            kırpmalar: Vec::new(),
            stil: SahneStili::default(),
            durum_stilleri: BTreeMap::new(),
            durum: GörselDurum::Normal,
            odak: OdakKapsamı::Yok,
            zlevel: 0,
            z: 0.0,
            z2: 0.0,
            görünür: true,
            sessiz: false,
            sürüklenebilir: false,
            imleç: None,
        }
    }

    pub fn çocuk(mut self, çocuk: SahneDüğümü) -> Self {
        if let SahneÖğesi::Grup(çocuklar) = &mut self.öğe {
            çocuklar.push(çocuk);
        }
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

    pub fn z(mut self, zlevel: i32, z: f32, z2: f32) -> Self {
        self.zlevel = zlevel;
        self.z = z;
        self.z2 = z2;
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

    pub fn etkin_stil(&self) -> SahneStili {
        let mut stil = self.stil.clone();
        if self.durum != GörselDurum::Normal
            && let Some(yama) = self.durum_stilleri.get(&self.durum)
        {
            yama.uygula(&mut stil);
        }
        stil.opaklık = stil.opaklık.clamp(0.0, 1.0);
        stil
    }

    /// Düğümün bütün alt ağacını ve yerel dönüşümlerini kapsayan, ebeveyn
    /// koordinatındaki sınır kutusu. `graphic` yerleşimi ile dış
    /// bütünleştiriciler aynı geometri hesabını kullanır.
    pub fn sınır_kutusu(&self) -> Option<Dikdörtgen> {
        düğüm_sınır_kutusu(self, AfinMatris::BİRİM)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sahneİsabeti {
    pub kimlik: String,
    pub yerel_nokta: (f32, f32),
    pub dünya_matrisi: AfinMatris,
    pub imleç: Option<String>,
    pub sürüklenebilir: bool,
    pub yakalanmış: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SahneFarkı {
    pub giren: Vec<String>,
    pub güncellenen: Vec<String>,
    pub çıkan: Vec<String>,
}

/// Kök sahne ve pointer capture durumu.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sahne {
    pub kökler: Vec<SahneDüğümü>,
    yakalanan: Option<String>,
}

impl Sahne {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ekle(&mut self, düğüm: SahneDüğümü) -> Result<(), BilesenHatasi> {
        if düğüm.kimlik.is_empty() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "scene.id",
                ayrıntı: "sahne kimliği boş olamaz".to_owned(),
            });
        }
        let mut yeni_kimlikler = BTreeSet::new();
        kimlikleri_topla(&düğüm, &mut yeni_kimlikler)?;
        if yeni_kimlikler
            .iter()
            .any(|kimlik| self.bul(kimlik).is_some())
        {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "scene.id",
                ayrıntı: "sahne kimlikleri benzersiz olmalı".to_owned(),
            });
        }
        self.kökler.push(düğüm);
        Ok(())
    }

    pub fn bul(&self, kimlik: &str) -> Option<&SahneDüğümü> {
        self.kökler
            .iter()
            .find_map(|düğüm| düğüm_bul(düğüm, kimlik))
    }

    pub fn bul_mut(&mut self, kimlik: &str) -> Option<&mut SahneDüğümü> {
        self.kökler
            .iter_mut()
            .find_map(|düğüm| düğüm_bul_mut(düğüm, kimlik))
    }

    pub fn kaldır(&mut self, kimlik: &str) -> Option<SahneDüğümü> {
        if let Some(sıra) = self.kökler.iter().position(|düğüm| düğüm.kimlik == kimlik) {
            let çıkan = self.kökler.remove(sıra);
            if self.yakalanan.as_deref() == Some(kimlik) {
                self.yakalanan = None;
            }
            return Some(çıkan);
        }
        for kök in &mut self.kökler {
            if let Some(çıkan) = çocuğu_kaldır(kök, kimlik) {
                if self.yakalanan.as_deref() == Some(kimlik) {
                    self.yakalanan = None;
                }
                return Some(çıkan);
            }
        }
        None
    }

    pub fn işaretçiyi_yakala(&mut self, kimlik: &str) -> Result<(), BilesenHatasi> {
        if self.bul(kimlik).is_none() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "scene.pointerCapture",
                ayrıntı: format!("`{kimlik}` düğümü yok"),
            });
        }
        self.yakalanan = Some(kimlik.to_owned());
        Ok(())
    }

    pub fn işaretçiyi_bırak(&mut self) {
        self.yakalanan = None;
    }

    pub fn çiz(&self, yüzey: &mut dyn ÇizimYüzeyi) {
        let mut görüntü = self.görüntü_listesi();
        görüntü.sort_by(görüntü_sırası);
        for kayıt in görüntü {
            kırpıp_çiz(yüzey, &kayıt.kırpmalar, 0, &kayıt);
        }
    }

    pub fn isabet(&self, dünya_noktası: (f32, f32)) -> Option<Sahneİsabeti> {
        let mut görüntü = self.görüntü_listesi();
        görüntü.sort_by(görüntü_sırası);
        if let Some(yakalanan) = &self.yakalanan
            && let Some(kayıt) = görüntü
                .iter()
                .find(|kayıt| kayıt.düğüm.kimlik == *yakalanan)
        {
            let yerel = kayıt
                .dünya
                .ters()
                .map(|ters| ters.noktayı_dönüştür(dünya_noktası))
                .unwrap_or(dünya_noktası);
            return Some(isabet_sonucu(kayıt, yerel, true));
        }
        for kayıt in görüntü.iter().rev() {
            if kayıt.sessiz || !kırpmalar_içeriyor(&kayıt.kırpmalar, dünya_noktası) {
                continue;
            }
            let Some(ters) = kayıt.dünya.ters() else {
                continue;
            };
            let yerel = ters.noktayı_dönüştür(dünya_noktası);
            if öğe_içeriyor(kayıt.düğüm, yerel) {
                return Some(isabet_sonucu(kayıt, yerel, false));
            }
        }
        None
    }

    /// Hedef → kök sıralı olay yayılım yolu.
    pub fn olay_yolu(&self, hedef: &str) -> Vec<String> {
        let mut yol = Vec::new();
        for kök in &self.kökler {
            if düğüm_yolu(kök, hedef, &mut yol) {
                yol.reverse();
                return yol;
            }
        }
        Vec::new()
    }

    pub fn fark(&self, yeni: &Sahne) -> SahneFarkı {
        let eski = self.kimlik_kümesi();
        let yeni_küme = yeni.kimlik_kümesi();
        SahneFarkı {
            giren: yeni_küme.difference(&eski).cloned().collect(),
            güncellenen: yeni_küme.intersection(&eski).cloned().collect(),
            çıkan: eski.difference(&yeni_küme).cloned().collect(),
        }
    }

    fn kimlik_kümesi(&self) -> BTreeSet<String> {
        let mut küme = BTreeSet::new();
        for kök in &self.kökler {
            kimlikleri_topla_güvenli(kök, &mut küme);
        }
        küme
    }

    fn görüntü_listesi(&self) -> Vec<GörüntüKaydı<'_>> {
        let mut sonuç = Vec::new();
        let mut eklenme = 0usize;
        for kök in &self.kökler {
            görüntü_topla(
                kök,
                AfinMatris::BİRİM,
                &[],
                false,
                (0, 0.0, 0.0),
                &mut eklenme,
                &mut sonuç,
            );
        }
        sonuç
    }
}

#[derive(Clone)]
struct DünyaKırpması {
    şekil: SahneŞekli,
    dünya: AfinMatris,
}

struct GörüntüKaydı<'a> {
    düğüm: &'a SahneDüğümü,
    dünya: AfinMatris,
    kırpmalar: Vec<DünyaKırpması>,
    sessiz: bool,
    zlevel: i32,
    z: f32,
    z2: f32,
    eklenme: usize,
}

#[allow(clippy::too_many_arguments)]
fn görüntü_topla<'a>(
    düğüm: &'a SahneDüğümü,
    ebeveyn: AfinMatris,
    üst_kırpmalar: &[DünyaKırpması],
    üst_sessiz: bool,
    üst_z: (i32, f32, f32),
    eklenme: &mut usize,
    sonuç: &mut Vec<GörüntüKaydı<'a>>,
) {
    if !düğüm.görünür || !düğüm.dönüşüm.matris().sonlu_mu() {
        return;
    }
    let dünya = ebeveyn.çarp(düğüm.dönüşüm.matris());
    let mut kırpmalar = üst_kırpmalar.to_vec();
    for kırpma in &düğüm.kırpmalar {
        kırpmalar.push(DünyaKırpması {
            şekil: kırpma.şekil.clone(),
            dünya: dünya.çarp(kırpma.dönüşüm.matris()),
        });
    }
    let sessiz = üst_sessiz || düğüm.sessiz;
    let z = (
        üst_z.0.saturating_add(düğüm.zlevel),
        üst_z.1 + düğüm.z,
        üst_z.2 + düğüm.z2,
    );
    match &düğüm.öğe {
        SahneÖğesi::Grup(çocuklar) => {
            for çocuk in çocuklar {
                görüntü_topla(çocuk, dünya, &kırpmalar, sessiz, z, eklenme, sonuç);
            }
        }
        SahneÖğesi::Şekil(_) | SahneÖğesi::Metin(_) | SahneÖğesi::Resim(_) => {
            let sıra = *eklenme;
            *eklenme = eklenme.saturating_add(1);
            sonuç.push(GörüntüKaydı {
                düğüm,
                dünya,
                kırpmalar,
                sessiz,
                zlevel: z.0,
                z: z.1,
                z2: z.2,
                eklenme: sıra,
            });
        }
    }
}

fn görüntü_sırası(a: &GörüntüKaydı<'_>, b: &GörüntüKaydı<'_>) -> Ordering {
    a.zlevel
        .cmp(&b.zlevel)
        .then_with(|| a.z.total_cmp(&b.z))
        .then_with(|| a.z2.total_cmp(&b.z2))
        .then_with(|| a.eklenme.cmp(&b.eklenme))
}

fn kırpıp_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    kırpmalar: &[DünyaKırpması],
    sıra: usize,
    kayıt: &GörüntüKaydı<'_>,
) {
    let Some(kırpma) = kırpmalar.get(sıra) else {
        düğümü_çiz(yüzey, kayıt);
        return;
    };
    let yol = yolu_dönüştür(&kırpma.şekil.yol(), kırpma.dünya);
    let mut içerik = |yüzey: &mut dyn ÇizimYüzeyi| {
        kırpıp_çiz(yüzey, kırpmalar, sıra.saturating_add(1), kayıt)
    };
    yüzey.yol_kırpılı(&yol, &mut içerik);
}

fn düğümü_çiz(yüzey: &mut dyn ÇizimYüzeyi, kayıt: &GörüntüKaydı<'_>) {
    let stil = kayıt.düğüm.etkin_stil();
    if stil.opaklık <= 0.0 {
        return;
    }
    match &kayıt.düğüm.öğe {
        SahneÖğesi::Şekil(şekil) => {
            let yol = yolu_dönüştür(&şekil.yol(), kayıt.dünya);
            if let Some(dolgu) = &stil.dolgu {
                yüzey.yol_doldur(&yol, &dolgu_opaklık(dolgu, stil.opaklık));
            }
            if let Some(renk) = stil.çizgi_rengi {
                let ölçek = ((kayıt.dünya.x_ölçeği() + kayıt.dünya.y_ölçeği()) / 2.0).max(0.0);
                yüzey.yol_çiz(
                    &yol,
                    stil.çizgi_kalınlığı * ölçek,
                    renk.opaklık(stil.opaklık),
                    stil.çizgi_türü,
                );
            }
        }
        SahneÖğesi::Metin(metin) => {
            if let Some(aile) = metin.aile.as_deref() {
                let _ = yüzey.dönüşümlü_aileli_yazı(
                    &metin.metin,
                    metin.konum,
                    metin.yatay,
                    metin.dikey,
                    metin.boyut,
                    metin.renk.opaklık(stil.opaklık),
                    metin.kalın,
                    aile,
                    kayıt.dünya,
                );
            } else {
                let _ = yüzey.dönüşümlü_yazı(
                    &metin.metin,
                    metin.konum,
                    metin.yatay,
                    metin.dikey,
                    metin.boyut,
                    metin.renk.opaklık(stil.opaklık),
                    metin.kalın,
                    kayıt.dünya,
                );
            }
        }
        SahneÖğesi::Resim(resim) => {
            let yol = yolu_dönüştür(
                &SahneŞekli::Dikdörtgen {
                    kutu: resim.kutu,
                    yarıçap: [0.0; 4],
                }
                .yol(),
                kayıt.dünya,
            );
            yüzey.yol_doldur(&yol, &dolgu_opaklık(&resim.yer_tutucu, stil.opaklık));
        }
        SahneÖğesi::Grup(_) => {}
    }
}

fn kırpmalar_içeriyor(kırpmalar: &[DünyaKırpması], nokta: (f32, f32)) -> bool {
    kırpmalar.iter().all(|kırpma| {
        kırpma
            .dünya
            .ters()
            .map(|ters| kırpma.şekil.içeriyor_mu(ters.noktayı_dönüştür(nokta), 0.0))
            .unwrap_or(false)
    })
}

fn öğe_içeriyor(düğüm: &SahneDüğümü, nokta: (f32, f32)) -> bool {
    let stil = düğüm.etkin_stil();
    match &düğüm.öğe {
        SahneÖğesi::Şekil(şekil) => şekil.içeriyor_mu(nokta, stil.çizgi_kalınlığı),
        SahneÖğesi::Metin(metin) => metin.sınır_kutusu().içeriyor_mu(nokta),
        SahneÖğesi::Resim(resim) => resim.kutu.içeriyor_mu(nokta),
        SahneÖğesi::Grup(_) => false,
    }
}

fn düğüm_sınır_kutusu(düğüm: &SahneDüğümü, ebeveyn: AfinMatris) -> Option<Dikdörtgen> {
    let dünya = ebeveyn.çarp(düğüm.dönüşüm.matris());
    match &düğüm.öğe {
        SahneÖğesi::Grup(çocuklar) => çocuklar
            .iter()
            .filter_map(|çocuk| düğüm_sınır_kutusu(çocuk, dünya))
            .reduce(kutuları_birleştir),
        SahneÖğesi::Şekil(şekil) => şekil
            .sınır_kutusu()
            .map(|kutu| kutuyu_dönüştür(kutu, dünya)),
        SahneÖğesi::Metin(metin) => Some(kutuyu_dönüştür(metin.sınır_kutusu(), dünya)),
        SahneÖğesi::Resim(resim) => Some(kutuyu_dönüştür(resim.kutu, dünya)),
    }
}

fn kutuyu_dönüştür(kutu: Dikdörtgen, matris: AfinMatris) -> Dikdörtgen {
    let noktalar = [
        matris.noktayı_dönüştür((kutu.x, kutu.y)),
        matris.noktayı_dönüştür((kutu.sağ(), kutu.y)),
        matris.noktayı_dönüştür((kutu.sağ(), kutu.alt())),
        matris.noktayı_dönüştür((kutu.x, kutu.alt())),
    ];
    let min_x = noktalar
        .iter()
        .map(|nokta| nokta.0)
        .fold(f32::INFINITY, f32::min);
    let max_x = noktalar
        .iter()
        .map(|nokta| nokta.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = noktalar
        .iter()
        .map(|nokta| nokta.1)
        .fold(f32::INFINITY, f32::min);
    let max_y = noktalar
        .iter()
        .map(|nokta| nokta.1)
        .fold(f32::NEG_INFINITY, f32::max);
    Dikdörtgen::yeni(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn kutuları_birleştir(a: Dikdörtgen, b: Dikdörtgen) -> Dikdörtgen {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Dikdörtgen::yeni(x, y, a.sağ().max(b.sağ()) - x, a.alt().max(b.alt()) - y)
}

fn isabet_sonucu(
    kayıt: &GörüntüKaydı<'_>,
    yerel_nokta: (f32, f32),
    yakalanmış: bool,
) -> Sahneİsabeti {
    Sahneİsabeti {
        kimlik: kayıt.düğüm.kimlik.clone(),
        yerel_nokta,
        dünya_matrisi: kayıt.dünya,
        imleç: kayıt.düğüm.imleç.clone(),
        sürüklenebilir: kayıt.düğüm.sürüklenebilir,
        yakalanmış,
    }
}

fn kimlikleri_topla(
    düğüm: &SahneDüğümü,
    kimlikler: &mut BTreeSet<String>,
) -> Result<(), BilesenHatasi> {
    if düğüm.kimlik.is_empty() || !kimlikler.insert(düğüm.kimlik.clone()) {
        return Err(BilesenHatasi::GeçersizSeçenek {
            alan: "scene.id",
            ayrıntı: "boş veya yinelenen alt düğüm kimliği".to_owned(),
        });
    }
    if let SahneÖğesi::Grup(çocuklar) = &düğüm.öğe {
        for çocuk in çocuklar {
            kimlikleri_topla(çocuk, kimlikler)?;
        }
    }
    Ok(())
}

fn kimlikleri_topla_güvenli(düğüm: &SahneDüğümü, kimlikler: &mut BTreeSet<String>) {
    kimlikler.insert(düğüm.kimlik.clone());
    if let SahneÖğesi::Grup(çocuklar) = &düğüm.öğe {
        for çocuk in çocuklar {
            kimlikleri_topla_güvenli(çocuk, kimlikler);
        }
    }
}

fn düğüm_bul<'a>(düğüm: &'a SahneDüğümü, kimlik: &str) -> Option<&'a SahneDüğümü> {
    if düğüm.kimlik == kimlik {
        return Some(düğüm);
    }
    match &düğüm.öğe {
        SahneÖğesi::Grup(çocuklar) => {
            çocuklar.iter().find_map(|çocuk| düğüm_bul(çocuk, kimlik))
        }
        _ => None,
    }
}

fn düğüm_bul_mut<'a>(
    düğüm: &'a mut SahneDüğümü,
    kimlik: &str,
) -> Option<&'a mut SahneDüğümü> {
    if düğüm.kimlik == kimlik {
        return Some(düğüm);
    }
    match &mut düğüm.öğe {
        SahneÖğesi::Grup(çocuklar) => çocuklar
            .iter_mut()
            .find_map(|çocuk| düğüm_bul_mut(çocuk, kimlik)),
        _ => None,
    }
}

fn çocuğu_kaldır(düğüm: &mut SahneDüğümü, kimlik: &str) -> Option<SahneDüğümü> {
    let SahneÖğesi::Grup(çocuklar) = &mut düğüm.öğe else {
        return None;
    };
    if let Some(sıra) = çocuklar.iter().position(|çocuk| çocuk.kimlik == kimlik) {
        return Some(çocuklar.remove(sıra));
    }
    çocuklar
        .iter_mut()
        .find_map(|çocuk| çocuğu_kaldır(çocuk, kimlik))
}

fn düğüm_yolu(düğüm: &SahneDüğümü, hedef: &str, yol: &mut Vec<String>) -> bool {
    if düğüm.kimlik == hedef {
        yol.push(düğüm.kimlik.clone());
        return true;
    }
    if let SahneÖğesi::Grup(çocuklar) = &düğüm.öğe {
        for çocuk in çocuklar {
            if düğüm_yolu(çocuk, hedef, yol) {
                yol.push(düğüm.kimlik.clone());
                return true;
            }
        }
    }
    false
}

fn noktaların_yolu(noktalar: &[(f32, f32)], kapalı: bool) -> Yol {
    let mut yol = Yol::yeni();
    let Some(ilk) = noktalar.first() else {
        return yol;
    };
    yol.taşı(*ilk);
    for nokta in noktalar.iter().skip(1) {
        yol.çiz(*nokta);
    }
    if kapalı {
        yol.kapat();
    }
    yol
}

pub(crate) fn yuvarlak_dikdörtgen_yolu(kutu: Dikdörtgen, yarıçap: [f32; 4]) -> Yol {
    let en_büyük = kutu.genişlik.min(kutu.yükseklik).max(0.0) / 2.0;
    let [sol_üst, sağ_üst, sağ_alt, sol_alt] = yarıçap.map(|r| r.clamp(0.0, en_büyük));
    let mut yol = Yol::yeni();
    yol.taşı((kutu.x + sol_üst, kutu.y));
    yol.çiz((kutu.sağ() - sağ_üst, kutu.y));
    if sağ_üst > 0.0 {
        yol.yay(sağ_üst, false, true, (kutu.sağ(), kutu.y + sağ_üst));
    }
    yol.çiz((kutu.sağ(), kutu.alt() - sağ_alt));
    if sağ_alt > 0.0 {
        yol.yay(sağ_alt, false, true, (kutu.sağ() - sağ_alt, kutu.alt()));
    }
    yol.çiz((kutu.x + sol_alt, kutu.alt()));
    if sol_alt > 0.0 {
        yol.yay(sol_alt, false, true, (kutu.x, kutu.alt() - sol_alt));
    }
    yol.çiz((kutu.x, kutu.y + sol_üst));
    if sol_üst > 0.0 {
        yol.yay(sol_üst, false, true, (kutu.x + sol_üst, kutu.y));
    }
    yol.kapat();
    yol
}

/// Yayları çoklu çizgiye açıp bütün komutları affine dönüştürür. Kübik
/// Bezier affine altında yine kübik kaldığından kontrol noktaları doğrudan
/// dönüştürülür.
pub fn yolu_dönüştür(yol: &Yol, matris: AfinMatris) -> Yol {
    let mut sonuç = Yol::yeni();
    let mut geçerli = (0.0, 0.0);
    let mut alt_yol_başı = (0.0, 0.0);
    for komut in &yol.komutlar {
        match *komut {
            YolKomutu::Taşı(nokta) => {
                geçerli = nokta;
                alt_yol_başı = nokta;
                sonuç.taşı(matris.noktayı_dönüştür(nokta));
            }
            YolKomutu::Çiz(nokta) => {
                geçerli = nokta;
                sonuç.çiz(matris.noktayı_dönüştür(nokta));
            }
            YolKomutu::Kübik { k1, k2, uç } => {
                geçerli = uç;
                sonuç.kübik(
                    matris.noktayı_dönüştür(k1),
                    matris.noktayı_dönüştür(k2),
                    matris.noktayı_dönüştür(uç),
                );
            }
            YolKomutu::Yay {
                yarıçap,
                büyük_yay,
                süpürme,
                uç,
            } => {
                for nokta in yay_noktaları(geçerli, yarıçap, büyük_yay, süpürme, uç, 0.08)
                {
                    sonuç.çiz(matris.noktayı_dönüştür(nokta));
                }
                geçerli = uç;
            }
            YolKomutu::Kapat => {
                geçerli = alt_yol_başı;
                sonuç.kapat();
            }
        }
    }
    sonuç
}

fn yolu_düzleştir(yol: &Yol, tolerans: f32) -> Vec<Vec<(f32, f32)>> {
    let mut sonuç = Vec::new();
    let mut etkin = Vec::new();
    let mut geçerli = (0.0, 0.0);
    let mut başlangıç = (0.0, 0.0);
    for komut in &yol.komutlar {
        match *komut {
            YolKomutu::Taşı(nokta) => {
                if !etkin.is_empty() {
                    sonuç.push(std::mem::take(&mut etkin));
                }
                etkin.push(nokta);
                geçerli = nokta;
                başlangıç = nokta;
            }
            YolKomutu::Çiz(nokta) => {
                etkin.push(nokta);
                geçerli = nokta;
            }
            YolKomutu::Kübik { k1, k2, uç } => {
                let yaklaşık_uzunluk = uzaklık(geçerli, k1) + uzaklık(k1, k2) + uzaklık(k2, uç);
                let adım = (yaklaşık_uzunluk / tolerans.max(0.1)).ceil() as usize;
                for sıra in 1..=adım.clamp(2, 128) {
                    let t = sıra as f32 / adım.clamp(2, 128) as f32;
                    etkin.push(kübik_nokta(geçerli, k1, k2, uç, t));
                }
                geçerli = uç;
            }
            YolKomutu::Yay {
                yarıçap,
                büyük_yay,
                süpürme,
                uç,
            } => {
                etkin.extend(yay_noktaları(
                    geçerli,
                    yarıçap,
                    büyük_yay,
                    süpürme,
                    uç,
                    tolerans / yarıçap.max(1.0),
                ));
                geçerli = uç;
            }
            YolKomutu::Kapat => {
                etkin.push(başlangıç);
                geçerli = başlangıç;
            }
        }
    }
    if !etkin.is_empty() {
        sonuç.push(etkin);
    }
    sonuç
}

fn yay_noktaları(
    başlangıç: (f32, f32),
    yarıçap: f32,
    büyük_yay: bool,
    süpürme: bool,
    uç: (f32, f32),
    açı_adımı: f32,
) -> Vec<(f32, f32)> {
    let dx = uç.0 - başlangıç.0;
    let dy = uç.1 - başlangıç.1;
    let kiriş = dx.hypot(dy);
    if kiriş < 1e-6 {
        return Vec::new();
    }
    let yarıçap = yarıçap.abs().max(kiriş / 2.0);
    let orta = ((başlangıç.0 + uç.0) / 2.0, (başlangıç.1 + uç.1) / 2.0);
    let yükseklik = (yarıçap * yarıçap - (kiriş / 2.0).powi(2)).max(0.0).sqrt();
    let normal = (-dy / kiriş, dx / kiriş);
    let işaret = if büyük_yay != süpürme { 1.0 } else { -1.0 };
    let merkez = (
        orta.0 + normal.0 * yükseklik * işaret,
        orta.1 + normal.1 * yükseklik * işaret,
    );
    let açı0 = (başlangıç.1 - merkez.1).atan2(başlangıç.0 - merkez.0);
    let açı1 = (uç.1 - merkez.1).atan2(uç.0 - merkez.0);
    let açıklık = açı_açıklığı(açı0, açı1, süpürme);
    let adet = (açıklık.abs() / açı_adımı.max(0.02)).ceil() as usize;
    (1..=adet.clamp(2, 256))
        .map(|sıra| {
            let açı = açı0 + açıklık * sıra as f32 / adet.clamp(2, 256) as f32;
            (
                merkez.0 + yarıçap * açı.cos(),
                merkez.1 + yarıçap * açı.sin(),
            )
        })
        .collect()
}

fn açı_açıklığı(başlangıç: f32, bitiş: f32, saat_yönü: bool) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut açıklık = bitiş - başlangıç;
    if saat_yönü && açıklık < 0.0 {
        açıklık += tau;
    } else if !saat_yönü && açıklık > 0.0 {
        açıklık -= tau;
    }
    açıklık
}

fn açı_arasında_mı(açı: f32, başlangıç: f32, bitiş: f32) -> bool {
    let tau = std::f32::consts::TAU;
    let açıklık = (bitiş - başlangıç).rem_euclid(tau);
    (açı - başlangıç).rem_euclid(tau) <= açıklık
}

fn kübik_nokta(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let u = 1.0 - t;
    let a = u * u * u;
    let b = 3.0 * u * u * t;
    let c = 3.0 * u * t * t;
    let d = t * t * t;
    (
        a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
        a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
    )
}

fn çokgen_içeriyor(noktalar: &[(f32, f32)], nokta: (f32, f32)) -> bool {
    if noktalar.len() < 3 {
        return false;
    }
    let mut içeride = false;
    let mut önceki = noktalar.last().copied().unwrap_or_default();
    for geçerli in noktalar {
        let kesişiyor = ((geçerli.1 > nokta.1) != (önceki.1 > nokta.1))
            && (nokta.0
                < (önceki.0 - geçerli.0) * (nokta.1 - geçerli.1) / (önceki.1 - geçerli.1)
                    + geçerli.0);
        if kesişiyor {
            içeride = !içeride;
        }
        önceki = *geçerli;
    }
    içeride
}

fn çoklu_çizgi_uzaklığı(noktalar: &[(f32, f32)], nokta: (f32, f32)) -> f32 {
    noktalar
        .windows(2)
        .filter_map(|çift| match çift {
            [a, b] => Some(doğruya_uzaklık(nokta, *a, *b)),
            _ => None,
        })
        .fold(f32::INFINITY, f32::min)
}

fn doğruya_uzaklık(nokta: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let ab = (b.0 - a.0, b.1 - a.1);
    let uzunluk_kare = ab.0 * ab.0 + ab.1 * ab.1;
    if uzunluk_kare <= 1e-12 {
        return uzaklık(nokta, a);
    }
    let t = (((nokta.0 - a.0) * ab.0 + (nokta.1 - a.1) * ab.1) / uzunluk_kare).clamp(0.0, 1.0);
    uzaklık(nokta, (a.0 + t * ab.0, a.1 + t * ab.1))
}

fn uzaklık(a: (f32, f32), b: (f32, f32)) -> f32 {
    uzaklık_kare(a, b).sqrt()
}

fn uzaklık_kare(a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)
}

fn dolgu_opaklık(dolgu: &Dolgu, opaklık: f32) -> Dolgu {
    match dolgu {
        Dolgu::Düz(renk) => Dolgu::Düz(renk.opaklık(opaklık)),
        Dolgu::Desen(desen) => Dolgu::Desen(desen.clone().opaklık(desen.opaklık * opaklık)),
        Dolgu::DoğrusalGradyan {
            x,
            y,
            x2,
            y2,
            duraklar,
        } => Dolgu::DoğrusalGradyan {
            x: *x,
            y: *y,
            x2: *x2,
            y2: *y2,
            duraklar: duraklar
                .iter()
                .map(|durak| crate::renk::RenkDurağı {
                    konum: durak.konum,
                    renk: durak.renk.opaklık(opaklık),
                })
                .collect(),
        },
        Dolgu::RadyalGradyan {
            x,
            y,
            yarıçap,
            duraklar,
        } => Dolgu::RadyalGradyan {
            x: *x,
            y: *y,
            yarıçap: *yarıçap,
            duraklar: duraklar
                .iter()
                .map(|durak| crate::renk::RenkDurağı {
                    konum: durak.konum,
                    renk: durak.renk.opaklık(opaklık),
                })
                .collect(),
        },
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
    use crate::cizim::{KayıtYüzeyi, SvgYüzeyi};

    fn kare(kimlik: &str, x: f32, y: f32, boyut: f32) -> SahneDüğümü {
        SahneDüğümü::şekil(
            kimlik,
            SahneŞekli::Dikdörtgen {
                kutu: Dikdörtgen::yeni(x, y, boyut, boyut),
                yarıçap: [0.0; 4],
            },
        )
    }

    #[test]
    fn ic_ice_transform_ve_clip_hit_test() {
        let çocuk = kare("hedef", 0.0, 0.0, 20.0).kırp(KırpmaYolu::yeni(SahneŞekli::Daire {
            merkez: (10.0, 10.0),
            yarıçap: 8.0,
        }));
        let grup = SahneDüğümü::grup("grup")
            .dönüşüm(
                YerelDönüşüm::default()
                    .ötele(100.0, 50.0)
                    .döndür(std::f32::consts::FRAC_PI_2),
            )
            .çocuk(çocuk);
        let mut sahne = Sahne::yeni();
        sahne.ekle(grup).unwrap();
        let dünya_merkezi = (90.0, 60.0);
        let isabet = sahne.isabet(dünya_merkezi).unwrap();
        assert_eq!(isabet.kimlik, "hedef");
        assert!((isabet.yerel_nokta.0 - 10.0).abs() < 1e-4);
        assert!(sahne.isabet((99.5, 50.5)).is_none());
    }

    #[test]
    fn z_sirasi_silent_ve_pointer_capture() {
        let alt = kare("alt", 0.0, 0.0, 20.0).z(0, 0.0, 0.0);
        let mut üst = kare("üst", 0.0, 0.0, 20.0).z(0, 1.0, 0.0);
        üst.sessiz = true;
        let mut sahne = Sahne::yeni();
        sahne.ekle(alt).unwrap();
        sahne.ekle(üst).unwrap();
        assert_eq!(sahne.isabet((5.0, 5.0)).unwrap().kimlik, "alt");
        sahne.işaretçiyi_yakala("üst").unwrap();
        let yakalanan = sahne.isabet((500.0, 500.0)).unwrap();
        assert_eq!(yakalanan.kimlik, "üst");
        assert!(yakalanan.yakalanmış);
        sahne.işaretçiyi_bırak();
        assert!(sahne.isabet((500.0, 500.0)).is_none());
    }

    #[test]
    fn durum_stili_normal_stili_miras_alir() {
        let mut düğüm = kare("k", 0.0, 0.0, 10.0)
            .stil(SahneStili {
                dolgu: Some(Dolgu::Düz(Renk::onaltılık(0xff0000))),
                çizgi_rengi: Some(Renk::SİYAH),
                çizgi_kalınlığı: 2.0,
                ..SahneStili::default()
            })
            .durum_stili(
                GörselDurum::Vurgu,
                SahneStilYaması {
                    opaklık: Some(0.5),
                    çizgi_kalınlığı: Some(4.0),
                    ..SahneStilYaması::default()
                },
            );
        düğüm.durum = GörselDurum::Vurgu;
        let stil = düğüm.etkin_stil();
        assert_eq!(stil.çizgi_kalınlığı, 4.0);
        assert_eq!(stil.çizgi_rengi, Some(Renk::SİYAH));
        assert_eq!(stil.opaklık, 0.5);
    }

    #[test]
    fn kayit_ve_svg_ayni_sahne_agacini_tuketir() {
        let mut sahne = Sahne::yeni();
        sahne
            .ekle(
                SahneDüğümü::grup("g")
                    .dönüşüm(YerelDönüşüm::default().ötele(20.0, 10.0))
                    .çocuk(kare("k", 0.0, 0.0, 10.0))
                    .çocuk(SahneDüğümü::metin(
                        "m",
                        SahneMetni::yeni("Merhaba", (0.0, 20.0)),
                    )),
            )
            .unwrap();
        let mut kayıt = KayıtYüzeyi::yeni(100.0, 100.0);
        sahne.çiz(&mut kayıt);
        assert!(kayıt.döküm().contains("T(20.0,10.0)"));
        assert!(kayıt.döküm().contains("dönüşümlü-yazı \"Merhaba\""));

        let mut svg = SvgYüzeyi::yeni(100.0, 100.0);
        sahne.çiz(&mut svg);
        let belge = svg.belge();
        assert!(belge.contains("matrix(1 0 0 1 20 10)"));
        assert!(belge.contains("Merhaba"));
    }

    #[test]
    fn kimlik_farki_olay_yolu_ve_kaldirma() {
        let mut eski = Sahne::yeni();
        eski.ekle(SahneDüğümü::grup("g").çocuk(kare("a", 0.0, 0.0, 1.0)))
            .unwrap();
        let mut yeni = Sahne::yeni();
        yeni.ekle(
            SahneDüğümü::grup("g")
                .çocuk(kare("a", 0.0, 0.0, 1.0))
                .çocuk(kare("b", 0.0, 0.0, 1.0)),
        )
        .unwrap();
        let fark = eski.fark(&yeni);
        assert_eq!(fark.giren, vec!["b"]);
        assert_eq!(fark.güncellenen, vec!["a", "g"]);
        assert_eq!(yeni.olay_yolu("b"), vec!["g", "b"]);
        assert!(yeni.kaldır("b").is_some());
        assert!(yeni.bul("b").is_none());
    }
}
