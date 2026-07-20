//! Zaman şeridi (`timeline`) seçenek modeli.
//!
//! Alanlar ECharts 6.1 `TimelineModel` ve `SliderTimelineModel` seçeneklerini
//! izler. `baseOption + options` birleşimi [`crate::calisma_zamani::BileşikSeçenekler`]
//! tarafından yapılır; bu model şeridin veri, yerleşim, oynatma ve görünüm
//! sözleşmesini taşır.

use crate::animasyon::Yumuşatma;
use crate::model::bilesen::Yön;
use crate::model::deger::VeriDeğeri;
use crate::model::stil::{YazıStili, ÇizgiStili, ÖğeStili};
use crate::model::{DikeyKonum, Uzunluk, YatayKonum};
use crate::renk::{Dolgu, Renk};

/// `timeline.axisType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ZamanŞeridiEksenTürü {
    #[default]
    Zaman,
    Kategori,
    Değer,
}

/// `timeline.symbol` ve checkpoint sembolü.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ZamanŞeridiSimgesi {
    #[default]
    Daire,
    Yok,
}

/// Etiketin şeride göre tarafı veya piksel uzaklığı.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ZamanŞeridiEtiketKonumu {
    #[default]
    Otomatik,
    Sol,
    Sağ,
    Üst,
    Alt,
    Uzaklık(f32),
}

/// Oynat/önceki/sonraki kontrollerinin ana eksendeki tarafı.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ZamanŞeridiKontrolKonumu {
    #[default]
    Sol,
    Sağ,
    Üst,
    Alt,
}

/// `timeline.label`.
#[derive(Clone, PartialEq, Debug)]
pub struct ZamanŞeridiEtiketi {
    pub göster: bool,
    pub konum: ZamanŞeridiEtiketKonumu,
    /// `None`, ECharts'ın otomatik aralık seçimini kullanır.
    pub aralık: Option<usize>,
    pub döndürme: f32,
    pub biçimleyici: Option<String>,
    pub yazı: YazıStili,
}

impl Default for ZamanŞeridiEtiketi {
    fn default() -> Self {
        Self {
            göster: true,
            konum: ZamanŞeridiEtiketKonumu::Otomatik,
            aralık: None,
            döndürme: 0.0,
            biçimleyici: None,
            yazı: YazıStili::default(),
        }
    }
}

impl ZamanŞeridiEtiketi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn konum(mut self, konum: ZamanŞeridiEtiketKonumu) -> Self {
        self.konum = konum;
        self
    }

    pub fn aralık(mut self, aralık: usize) -> Self {
        self.aralık = Some(aralık);
        self
    }

    pub fn döndürme(mut self, derece: f32) -> Self {
        self.döndürme = derece;
        self
    }

    pub fn biçimleyici(mut self, biçimleyici: impl Into<String>) -> Self {
        self.biçimleyici = Some(biçimleyici.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }
}

/// `timeline.checkpointStyle`.
#[derive(Clone, PartialEq, Debug)]
pub struct ZamanŞeridiKontrolNoktasıStili {
    pub simge: ZamanŞeridiSimgesi,
    pub simge_boyutu: f32,
    pub öğe_stili: ÖğeStili,
    pub animasyon: bool,
    pub animasyon_süresi: f32,
    pub animasyon_eğrisi: Yumuşatma,
}

impl Default for ZamanŞeridiKontrolNoktasıStili {
    fn default() -> Self {
        Self {
            simge: ZamanŞeridiSimgesi::Daire,
            simge_boyutu: 15.0,
            öğe_stili: ÖğeStili::default(),
            animasyon: true,
            animasyon_süresi: 300.0,
            animasyon_eğrisi: Yumuşatma::KübikGirişÇıkış,
        }
    }
}

impl ZamanŞeridiKontrolNoktasıStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn simge(mut self, simge: ZamanŞeridiSimgesi) -> Self {
        self.simge = simge;
        self
    }

    pub fn simge_boyutu(mut self, boyut: f32) -> Self {
        self.simge_boyutu = boyut.max(0.0);
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn animasyon(mut self, açık: bool) -> Self {
        self.animasyon = açık;
        self
    }

    pub fn animasyon_süresi(mut self, ms: f32) -> Self {
        self.animasyon_süresi = ms.max(0.0);
        self
    }

    pub fn animasyon_eğrisi(mut self, eğri: Yumuşatma) -> Self {
        self.animasyon_eğrisi = eğri;
        self
    }
}

/// `timeline.controlStyle`.
#[derive(Clone, PartialEq, Debug)]
pub struct ZamanŞeridiKontrolStili {
    pub göster: bool,
    pub oynat_göster: bool,
    pub önceki_göster: bool,
    pub sonraki_göster: bool,
    pub öğe_boyutu: f32,
    pub öğe_boşluğu: f32,
    pub konum: ZamanŞeridiKontrolKonumu,
    pub renk: Option<Renk>,
    pub kenarlık_rengi: Option<Renk>,
    pub kenarlık_kalınlığı: f32,
}

impl Default for ZamanŞeridiKontrolStili {
    fn default() -> Self {
        Self {
            göster: true,
            oynat_göster: true,
            önceki_göster: true,
            sonraki_göster: true,
            öğe_boyutu: 24.0,
            öğe_boşluğu: 12.0,
            konum: ZamanŞeridiKontrolKonumu::Sol,
            renk: None,
            kenarlık_rengi: None,
            kenarlık_kalınlığı: 0.0,
        }
    }
}

impl ZamanŞeridiKontrolStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn oynat_düğmesi(mut self, göster: bool) -> Self {
        self.oynat_göster = göster;
        self
    }

    pub fn önceki_düğmesi(mut self, göster: bool) -> Self {
        self.önceki_göster = göster;
        self
    }

    pub fn sonraki_düğmesi(mut self, göster: bool) -> Self {
        self.sonraki_göster = göster;
        self
    }

    pub fn öğe_boyutu(mut self, boyut: f32) -> Self {
        self.öğe_boyutu = boyut.max(0.0);
        self
    }

    pub fn öğe_boşluğu(mut self, boşluk: f32) -> Self {
        self.öğe_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn konum(mut self, konum: ZamanŞeridiKontrolKonumu) -> Self {
        self.konum = konum;
        self
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn kenarlık_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.kenarlık_rengi = Some(renk.into());
        self
    }

    pub fn kenarlık_kalınlığı(mut self, kalınlık: f32) -> Self {
        self.kenarlık_kalınlığı = kalınlık.max(0.0);
        self
    }
}

/// Slider zaman şeridi (`timeline: { type: 'slider' }`).
#[derive(Clone, PartialEq, Debug)]
pub struct ZamanŞeridi {
    pub göster: bool,
    pub eksen_türü: ZamanŞeridiEksenTürü,
    pub geçerli_sıra: usize,
    pub otomatik_oynat: bool,
    pub geri_sar: bool,
    pub döngü: bool,
    pub oynatma_aralığı: f32,
    pub gerçek_zamanlı: bool,
    pub yön: Yön,
    pub ters: bool,
    pub sol: Option<YatayKonum>,
    pub sağ: Option<Uzunluk>,
    pub üst: Option<DikeyKonum>,
    pub alt: Option<Uzunluk>,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    /// CSS sırası: üst, sağ, alt, sol.
    pub iç_boşluk: [f32; 4],
    pub arkaplan: Option<Dolgu>,
    pub kenarlık_rengi: Option<Renk>,
    pub kenarlık_kalınlığı: f32,
    pub simge: ZamanŞeridiSimgesi,
    pub simge_boyutu: f32,
    pub çizgi_göster: bool,
    pub çizgi_stili: ÇizgiStili,
    pub öğe_stili: ÖğeStili,
    pub kontrol_noktası_stili: ZamanŞeridiKontrolNoktasıStili,
    pub kontrol_stili: ZamanŞeridiKontrolStili,
    pub etiket: ZamanŞeridiEtiketi,
    pub veri: Vec<VeriDeğeri>,
}

impl Default for ZamanŞeridi {
    fn default() -> Self {
        Self {
            göster: true,
            eksen_türü: ZamanŞeridiEksenTürü::Zaman,
            geçerli_sıra: 0,
            otomatik_oynat: false,
            geri_sar: false,
            döngü: true,
            oynatma_aralığı: 2_000.0,
            gerçek_zamanlı: true,
            yön: Yön::Yatay,
            ters: false,
            sol: Some(YatayKonum::Değer(Uzunluk::Yüzde(20.0))),
            sağ: Some(Uzunluk::Yüzde(20.0)),
            üst: None,
            alt: Some(Uzunluk::Piksel(0.0)),
            genişlik: None,
            yükseklik: Some(Uzunluk::Piksel(40.0)),
            iç_boşluk: [15.0; 4],
            arkaplan: None,
            kenarlık_rengi: None,
            kenarlık_kalınlığı: 0.0,
            simge: ZamanŞeridiSimgesi::Daire,
            simge_boyutu: 12.0,
            çizgi_göster: true,
            çizgi_stili: ÇizgiStili::default(),
            öğe_stili: ÖğeStili::default(),
            kontrol_noktası_stili: ZamanŞeridiKontrolNoktasıStili::default(),
            kontrol_stili: ZamanŞeridiKontrolStili::default(),
            etiket: ZamanŞeridiEtiketi::default(),
            veri: Vec::new(),
        }
    }
}

impl ZamanŞeridi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn eksen_türü(mut self, tür: ZamanŞeridiEksenTürü) -> Self {
        self.eksen_türü = tür;
        self
    }

    pub fn geçerli_sıra(mut self, sıra: usize) -> Self {
        self.geçerli_sıra = sıra;
        self
    }

    pub fn otomatik_oynat(mut self, açık: bool) -> Self {
        self.otomatik_oynat = açık;
        self
    }

    pub fn geri_sar(mut self, açık: bool) -> Self {
        self.geri_sar = açık;
        self
    }

    pub fn döngü(mut self, açık: bool) -> Self {
        self.döngü = açık;
        self
    }

    pub fn oynatma_aralığı(mut self, ms: f32) -> Self {
        self.oynatma_aralığı = ms.max(0.0);
        self
    }

    pub fn gerçek_zamanlı(mut self, açık: bool) -> Self {
        self.gerçek_zamanlı = açık;
        self
    }

    pub fn yön(mut self, yön: Yön) -> Self {
        self.yön = yön;
        self
    }

    pub fn ters(mut self, ters: bool) -> Self {
        self.ters = ters;
        self
    }

    pub fn sol(mut self, sol: impl Into<YatayKonum>) -> Self {
        self.sol = Some(sol.into());
        if self.genişlik.is_some() {
            self.sağ = None;
        }
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(sağ.into());
        if self.genişlik.is_some() {
            self.sol = None;
        }
        self
    }

    pub fn üst(mut self, üst: impl Into<DikeyKonum>) -> Self {
        self.üst = Some(üst.into());
        if self.yükseklik.is_some() {
            self.alt = None;
        }
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = Some(alt.into());
        if self.yükseklik.is_some() {
            self.üst = None;
        }
        self
    }

    pub fn genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(genişlik.into());
        if self.sağ.is_some() {
            self.sol = None;
        }
        self
    }

    pub fn yükseklik(mut self, yükseklik: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(yükseklik.into());
        if self.üst.is_some() {
            self.alt = None;
        }
        self
    }

    pub fn iç_boşluk(mut self, boşluk: f32) -> Self {
        self.iç_boşluk = [boşluk.max(0.0); 4];
        self
    }

    pub fn iç_boşluklar(mut self, boşluk: [f32; 4]) -> Self {
        self.iç_boşluk = boşluk.map(|değer| değer.max(0.0));
        self
    }

    pub fn arkaplan(mut self, dolgu: impl Into<Dolgu>) -> Self {
        self.arkaplan = Some(dolgu.into());
        self
    }

    pub fn kenarlık(mut self, kalınlık: f32, renk: impl Into<Renk>) -> Self {
        self.kenarlık_kalınlığı = kalınlık.max(0.0);
        self.kenarlık_rengi = Some(renk.into());
        self
    }

    pub fn simge(mut self, simge: ZamanŞeridiSimgesi) -> Self {
        self.simge = simge;
        self
    }

    pub fn simge_boyutu(mut self, boyut: f32) -> Self {
        self.simge_boyutu = boyut.max(0.0);
        self
    }

    pub fn çizgi_göster(mut self, göster: bool) -> Self {
        self.çizgi_göster = göster;
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = stil;
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn kontrol_noktası_stili(mut self, stil: ZamanŞeridiKontrolNoktasıStili) -> Self {
        self.kontrol_noktası_stili = stil;
        self
    }

    pub fn kontrol_stili(mut self, stil: ZamanŞeridiKontrolStili) -> Self {
        self.kontrol_stili = stil;
        self
    }

    pub fn etiket(mut self, etiket: ZamanŞeridiEtiketi) -> Self {
        self.etiket = etiket;
        self
    }

    pub fn veri<T: Into<VeriDeğeri>>(mut self, veri: impl IntoIterator<Item = T>) -> Self {
        self.veri = veri.into_iter().map(Into::into).collect();
        self
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn sağ_ve_genişlik_soldaki_öntanımlıyı_bırakmaz() {
        let şerit = ZamanŞeridi::yeni().sağ(50).genişlik(10);
        assert_eq!(şerit.sol, None);
        assert_eq!(şerit.sağ, Some(Uzunluk::Piksel(50.0)));
    }

    #[test]
    fn ortalanmış_üst_ve_yükseklik_alt_öntanımlıyı_bırakmaz() {
        let şerit = ZamanŞeridi::yeni().üst("center").yükseklik(300);
        assert_eq!(şerit.üst, Some(DikeyKonum::Orta));
        assert_eq!(şerit.alt, None);
    }
}
