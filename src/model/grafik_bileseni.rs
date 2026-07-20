//! Serbest grafik bileşeni — ECharts `graphic` / zrender öğelerinin
//! bildirime dayalı seçenek karşılığı.
//!
//! Öğeler hiyerarşik olabilir; ortak dönüşüm, yerleşim, görünürlük ve olay
//! alanları içerikten bağımsız tutulur. Çizim katmanı bu modeli aynı sahne
//! ağacına dönüştürerek PNG, SVG, kayıt ve gpui yüzeylerinde ortak davranış
//! sağlar.

use std::collections::{BTreeMap, BTreeSet};

use crate::cizim::{
    GörselDurum, KırpmaYolu, OdakKapsamı, SahneMetni, SahneResmi, SahneStilYaması, SahneStili,
    SahneŞekli, YerelDönüşüm,
};
use crate::hata::BilesenHatasi;
use crate::koordinat::Dikdörtgen;
use crate::model::{DikeyKonum, Uzunluk, YatayKonum};
use crate::renk::Renk;

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

    pub fn doğrula(&self) -> Result<(), BilesenHatasi> {
        let mut kimlikler = BTreeSet::new();
        for öğe in &self.öğeler {
            öğe.doğrula(&mut kimlikler)?;
        }
        Ok(())
    }
}

/// Bir `graphic` öğesinin içeriği (`type`).
#[derive(Clone, Debug, PartialEq)]
pub enum GrafikÖğeİçeriği {
    Grup(Vec<GrafikÖğesi>),
    Şekil(SahneŞekli),
    Metin(SahneMetni),
    Resim(SahneResmi),
}

/// ECharts `graphic` öğesi. Alan adları zrender sözleşmesindeki karşılıkları
/// izler: `ignore`, `invisible`, `silent`, `draggable`, `cursor`, `zlevel`,
/// `z`, `z2`, dönüşüm ve bağlı `textContent`.
#[derive(Clone, Debug, PartialEq)]
pub struct GrafikÖğesi {
    pub kimlik: Option<String>,
    pub ad: Option<String>,
    pub içerik: GrafikÖğeİçeriği,
    pub yerleşim: GrafikYerleşimi,
    pub dönüşüm: YerelDönüşüm,
    pub kırpmalar: Vec<KırpmaYolu>,
    pub stil: SahneStili,
    pub durum_stilleri: BTreeMap<GörselDurum, SahneStilYaması>,
    pub durum: GörselDurum,
    pub odak: OdakKapsamı,
    pub bağlı_metin: Option<GrafikBağlıMetni>,
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
            içerik,
            yerleşim: GrafikYerleşimi::default(),
            dönüşüm: YerelDönüşüm::default(),
            kırpmalar: Vec::new(),
            stil: SahneStili::default(),
            durum_stilleri: BTreeMap::new(),
            durum: GörselDurum::Normal,
            odak: OdakKapsamı::Yok,
            bağlı_metin: None,
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
            self.stil.çizgi_kalınlığı,
            self.stil.opaklık,
        ];
        if sayılar.iter().any(|değer| !değer.is_finite()) || !self.dönüşüm.ek.sonlu_mu() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "graphic.elements",
                ayrıntı: "graphic dönüşüm, z ve stil sayıları sonlu olmalıdır".to_owned(),
            });
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
