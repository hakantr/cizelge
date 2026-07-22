//! ECharts `parallel` koordinatı ve `parallelAxis` bileşen seçenekleri.

use crate::model::Uzunluk;
use crate::model::eksen::{
    BölmeÇizgisi, Eksen, EksenAdKonumu, EksenEtiketi, EksenTürü, EksenÇentiği, EksenÇizgisi,
};
use crate::model::stil::YazıStili;
use crate::renk::Renk;

/// `parallel.layout`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ParalelYerleşim {
    #[default]
    Yatay,
    Dikey,
}

/// `parallel.axisExpandTriggerOn`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ParalelGenişletmeTetikleyicisi {
    #[default]
    Tıklama,
    FareHareketi,
}

/// `parallelAxis.areaSelectStyle`.
#[derive(Clone, PartialEq, Debug)]
pub struct ParalelAlanSeçimStili {
    pub genişlik: f32,
    pub kenarlık_kalınlığı: f32,
    pub kenarlık_rengi: Renk,
    pub renk: Renk,
    pub opaklık: f32,
}

impl Default for ParalelAlanSeçimStili {
    fn default() -> Self {
        Self {
            genişlik: 20.0,
            kenarlık_kalınlığı: 1.0,
            kenarlık_rengi: Renk::onaltılık(0xa0c5e8),
            renk: Renk::onaltılık(0xa0c5e8),
            opaklık: 0.3,
        }
    }
}

impl ParalelAlanSeçimStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn genişlik(mut self, genişlik: f32) -> Self {
        self.genişlik = genişlik.max(0.0);
        self
    }

    pub fn kenarlık_kalınlığı(mut self, kalınlık: f32) -> Self {
        self.kenarlık_kalınlığı = kalınlık.max(0.0);
        self
    }

    pub fn kenarlık_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.kenarlık_rengi = renk.into();
        self
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = renk.into();
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = opaklık.clamp(0.0, 1.0);
        self
    }
}

/// `parallel` koordinat bileşeni.
#[derive(Clone, PartialEq, Debug)]
pub struct ParalelKoordinatı {
    pub kimlik: Option<String>,
    pub z: i32,
    pub sol: Option<Uzunluk>,
    pub sağ: Option<Uzunluk>,
    pub üst: Option<Uzunluk>,
    pub alt: Option<Uzunluk>,
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    pub yerleşim: ParalelYerleşim,
    pub eksen_genişletilebilir: bool,
    pub eksen_genişletme_merkezi: Option<f32>,
    pub eksen_genişletme_sayısı: usize,
    pub eksen_genişletme_genişliği: f32,
    pub eksen_genişletme_tetikleyicisi: ParalelGenişletmeTetikleyicisi,
    pub eksen_genişletme_oranı: f32,
    pub eksen_genişletme_gecikmesi_ms: u64,
    pub eksen_genişletme_kaydırma_tetik_alanı: [Option<f32>; 3],
    pub eksen_genişletme_penceresi: Option<[f32; 2]>,
    pub eksen_varsayılanı: Eksen,
}

impl Default for ParalelKoordinatı {
    fn default() -> Self {
        // `ParallelAxisModel.defaultOption.z`: eksenler seri çizgilerinin
        // (`series.parallel.z = 2`) üstünde boyanır. Kullanıcı açık bir
        // `parallelAxisDefault.z` verirse bu değer doğal olarak ezilir.
        let mut eksen_varsayılanı = Eksen::değer();
        eksen_varsayılanı.z = 10;
        Self {
            kimlik: None,
            z: 0,
            sol: Some(Uzunluk::Piksel(80.0)),
            sağ: Some(Uzunluk::Piksel(80.0)),
            üst: Some(Uzunluk::Piksel(60.0)),
            alt: Some(Uzunluk::Piksel(60.0)),
            genişlik: None,
            yükseklik: None,
            yerleşim: ParalelYerleşim::Yatay,
            eksen_genişletilebilir: false,
            eksen_genişletme_merkezi: None,
            eksen_genişletme_sayısı: 0,
            eksen_genişletme_genişliği: 50.0,
            eksen_genişletme_tetikleyicisi: ParalelGenişletmeTetikleyicisi::Tıklama,
            eksen_genişletme_oranı: 17.0,
            eksen_genişletme_gecikmesi_ms: 50,
            eksen_genişletme_kaydırma_tetik_alanı: [Some(-0.15), Some(0.05), Some(0.4)],
            eksen_genişletme_penceresi: None,
            eksen_varsayılanı,
        }
    }
}

impl ParalelKoordinatı {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn kimlik(mut self, kimlik: impl Into<String>) -> Self {
        self.kimlik = Some(kimlik.into());
        self
    }

    pub fn z(mut self, z: i32) -> Self {
        self.z = z;
        self
    }

    pub fn sol(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sol = Some(değer.into());
        self
    }

    pub fn sağ(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(değer.into());
        self
    }

    pub fn üst(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.üst = Some(değer.into());
        self
    }

    pub fn alt(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.alt = Some(değer.into());
        self
    }

    pub fn genişlik(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(değer.into());
        self
    }

    pub fn yükseklik(mut self, değer: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(değer.into());
        self
    }

    pub fn yerleşim(mut self, yerleşim: ParalelYerleşim) -> Self {
        self.yerleşim = yerleşim;
        self
    }

    pub fn eksen_genişletilebilir(mut self, açık: bool) -> Self {
        self.eksen_genişletilebilir = açık;
        self
    }

    pub fn eksen_genişletme_merkezi(mut self, merkez: f32) -> Self {
        self.eksen_genişletme_merkezi = merkez.is_finite().then_some(merkez);
        self
    }

    pub fn eksen_genişletme_sayısı(mut self, sayı: usize) -> Self {
        self.eksen_genişletme_sayısı = sayı;
        self
    }

    pub fn eksen_genişletme_genişliği(mut self, genişlik: f32) -> Self {
        self.eksen_genişletme_genişliği = genişlik.max(0.0);
        self
    }

    pub fn eksen_genişletme_tetikleyicisi(
        mut self,
        tetikleyici: ParalelGenişletmeTetikleyicisi,
    ) -> Self {
        self.eksen_genişletme_tetikleyicisi = tetikleyici;
        self
    }

    pub fn eksen_genişletme_oranı(mut self, oran: f32) -> Self {
        self.eksen_genişletme_oranı = oran.max(0.0);
        self
    }

    pub fn eksen_genişletme_gecikmesi_ms(mut self, gecikme: u64) -> Self {
        self.eksen_genişletme_gecikmesi_ms = gecikme;
        self
    }

    pub fn eksen_genişletme_kaydırma_tetik_alanı(mut self, alan: [Option<f32>; 3]) -> Self {
        self.eksen_genişletme_kaydırma_tetik_alanı = alan;
        self
    }

    pub fn eksen_genişletme_penceresi(mut self, pencere: [f32; 2]) -> Self {
        self.eksen_genişletme_penceresi = Some(pencere);
        self
    }

    pub fn eksen_varsayılanı(mut self, eksen: Eksen) -> Self {
        self.eksen_varsayılanı = eksen;
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct ParalelEksenAçıkları {
    tam: bool,
    göster: bool,
    z: bool,
    tür: bool,
    ad: bool,
    ad_konumu: bool,
    ad_boşluğu: bool,
    ad_yazı: bool,
    veri: bool,
    en_az: bool,
    en_çok: bool,
    ölçekli: bool,
    bölme_sayısı: bool,
    ters: bool,
    çizgi: bool,
    çentik: bool,
    etiket: bool,
    bölme_çizgisi: bool,
}

/// `parallelAxis` bileşeni. `dim`, seri veri satırındaki boyut sırasıdır.
#[derive(Clone, PartialEq, Debug)]
pub struct ParalelEkseni {
    pub kimlik: Option<String>,
    pub boyutlar: Vec<usize>,
    pub paralel_sırası: usize,
    pub paralel_kimliği: Option<String>,
    pub eksen: Eksen,
    pub alan_seçim_stili: ParalelAlanSeçimStili,
    pub gerçek_zamanlı: bool,
    pub etkin_aralıklar: Vec<[f64; 2]>,
    açıklar: ParalelEksenAçıkları,
}

impl ParalelEkseni {
    pub fn yeni(boyut: usize) -> Self {
        Self {
            kimlik: None,
            boyutlar: vec![boyut],
            paralel_sırası: 0,
            paralel_kimliği: None,
            eksen: Eksen::değer(),
            alan_seçim_stili: ParalelAlanSeçimStili::default(),
            gerçek_zamanlı: true,
            etkin_aralıklar: Vec::new(),
            açıklar: ParalelEksenAçıkları::default(),
        }
    }

    pub fn kimlik(mut self, kimlik: impl Into<String>) -> Self {
        self.kimlik = Some(kimlik.into());
        self
    }

    pub fn boyutlar(mut self, boyutlar: impl IntoIterator<Item = usize>) -> Self {
        self.boyutlar = boyutlar.into_iter().collect();
        self
    }

    pub fn paralel_sırası(mut self, sıra: usize) -> Self {
        self.paralel_sırası = sıra;
        self
    }

    pub fn paralel_kimliği(mut self, kimlik: impl Into<String>) -> Self {
        self.paralel_kimliği = Some(kimlik.into());
        self
    }

    /// Tam axis seçeneği; `parallelAxisDefault` mirasını bilinçli olarak
    /// kapatır. Alan bazlı kurucular yalnız dokundukları değeri mirasın
    /// üstüne yazar.
    pub fn eksen(mut self, eksen: Eksen) -> Self {
        self.eksen = eksen;
        self.açıklar.tam = true;
        self
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.eksen.göster = göster;
        self.açıklar.göster = true;
        self
    }

    pub fn z(mut self, z: i32) -> Self {
        self.eksen.z = z;
        self.açıklar.z = true;
        self
    }

    pub fn tür(mut self, tür: EksenTürü) -> Self {
        self.eksen.tür = tür;
        self.açıklar.tür = true;
        self
    }

    pub fn kategori(mut self) -> Self {
        self.eksen.tür = EksenTürü::Kategori;
        self.açıklar.tür = true;
        self
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.eksen.ad = Some(ad.into());
        self.açıklar.ad = true;
        self
    }

    pub fn ad_konumu(mut self, konum: EksenAdKonumu) -> Self {
        self.eksen.ad_konumu = konum;
        self.açıklar.ad_konumu = true;
        self
    }

    pub fn ad_boşluğu(mut self, boşluk: f32) -> Self {
        self.eksen.ad_boşluğu = boşluk;
        self.açıklar.ad_boşluğu = true;
        self
    }

    pub fn ad_yazı(mut self, yazı: YazıStili) -> Self {
        self.eksen.ad_yazı = yazı;
        self.açıklar.ad_yazı = true;
        self
    }

    pub fn veri<S: Into<String>>(mut self, veri: impl IntoIterator<Item = S>) -> Self {
        self.eksen.veri = veri.into_iter().map(Into::into).collect();
        self.açıklar.veri = true;
        self
    }

    pub fn en_az(mut self, değer: f64) -> Self {
        self.eksen.en_az = Some(değer);
        self.açıklar.en_az = true;
        self
    }

    pub fn en_çok(mut self, değer: f64) -> Self {
        self.eksen.en_çok = Some(değer);
        self.açıklar.en_çok = true;
        self
    }

    pub fn ölçekli(mut self, ölçekli: bool) -> Self {
        self.eksen.sıfırı_içer = !ölçekli;
        self.açıklar.ölçekli = true;
        self
    }

    pub fn bölme_sayısı(mut self, sayı: usize) -> Self {
        self.eksen.bölme_sayısı = sayı.max(1);
        self.eksen.bölme_sayısı_belirtildi = true;
        self.açıklar.bölme_sayısı = true;
        self
    }

    pub fn ters(mut self, ters: bool) -> Self {
        self.eksen.ters = ters;
        self.açıklar.ters = true;
        self
    }

    pub fn çizgi(mut self, çizgi: EksenÇizgisi) -> Self {
        self.eksen.çizgi = çizgi;
        self.açıklar.çizgi = true;
        self
    }

    pub fn çentik(mut self, çentik: EksenÇentiği) -> Self {
        self.eksen.çentik = çentik;
        self.açıklar.çentik = true;
        self
    }

    pub fn etiket(mut self, etiket: EksenEtiketi) -> Self {
        self.eksen.etiket = etiket;
        self.açıklar.etiket = true;
        self
    }

    pub fn bölme_çizgisi(mut self, çizgi: BölmeÇizgisi) -> Self {
        self.eksen.bölme_çizgisi = çizgi;
        self.açıklar.bölme_çizgisi = true;
        self
    }

    pub fn alan_seçim_stili(mut self, stil: ParalelAlanSeçimStili) -> Self {
        self.alan_seçim_stili = stil;
        self
    }

    pub fn gerçek_zamanlı(mut self, açık: bool) -> Self {
        self.gerçek_zamanlı = açık;
        self
    }

    pub fn etkin_aralık(mut self, başlangıç: f64, bitiş: f64) -> Self {
        self.etkin_aralıklar.push([başlangıç, bitiş]);
        self
    }

    pub fn etkin_aralıklar(mut self, aralıklar: impl IntoIterator<Item = [f64; 2]>) -> Self {
        self.etkin_aralıklar = aralıklar.into_iter().collect();
        self
    }

    pub(crate) fn çöz(&self, varsayılan: &Eksen) -> Eksen {
        if self.açıklar.tam {
            return self.eksen.clone();
        }
        let mut sonuç = varsayılan.clone();
        if self.açıklar.göster {
            sonuç.göster = self.eksen.göster;
        }
        if self.açıklar.z {
            sonuç.z = self.eksen.z;
        }
        if self.açıklar.tür {
            sonuç.tür = self.eksen.tür;
        }
        if self.açıklar.ad {
            sonuç.ad = self.eksen.ad.clone();
        }
        if self.açıklar.ad_konumu {
            sonuç.ad_konumu = self.eksen.ad_konumu;
        }
        if self.açıklar.ad_boşluğu {
            sonuç.ad_boşluğu = self.eksen.ad_boşluğu;
        }
        if self.açıklar.ad_yazı {
            sonuç.ad_yazı = self.eksen.ad_yazı.clone();
        }
        if self.açıklar.veri {
            sonuç.veri = self.eksen.veri.clone();
        }
        if self.açıklar.en_az {
            sonuç.en_az = self.eksen.en_az;
        }
        if self.açıklar.en_çok {
            sonuç.en_çok = self.eksen.en_çok;
        }
        if self.açıklar.ölçekli {
            sonuç.sıfırı_içer = self.eksen.sıfırı_içer;
        }
        if self.açıklar.bölme_sayısı {
            sonuç.bölme_sayısı = self.eksen.bölme_sayısı;
            sonuç.bölme_sayısı_belirtildi = true;
        }
        if self.açıklar.ters {
            sonuç.ters = self.eksen.ters;
        }
        if self.açıklar.çizgi {
            sonuç.çizgi = self.eksen.çizgi.clone();
        }
        if self.açıklar.çentik {
            sonuç.çentik = self.eksen.çentik.clone();
        }
        if self.açıklar.etiket {
            sonuç.etiket = self.eksen.etiket.clone();
        }
        if self.açıklar.bölme_çizgisi {
            sonuç.bölme_çizgisi = self.eksen.bölme_çizgisi.clone();
        }
        sonuç
    }
}
