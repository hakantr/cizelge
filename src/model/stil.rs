//! Stil seçenekleri — ECharts'taki `lineStyle`, `itemStyle`, `areaStyle`,
//! `textStyle` ve `label` tanımlarının karşılığı.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::model::Uzunluk;
use crate::renk::{Dolgu, Renk};

/// Çizgi türü (`lineStyle.type`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ÇizgiTürü {
    #[default]
    Düz,
    Kesikli,
    Noktalı,
}

/// Çizgi stili (`lineStyle`).
#[derive(Clone, PartialEq, Debug)]
pub struct ÇizgiStili {
    pub renk: Option<Renk>,
    pub kalınlık: f32,
    pub tür: ÇizgiTürü,
    pub opaklık: f32,
    /// Canvas/zrender çizgi gölgesi (`shadowBlur`, `shadowColor`,
    /// `shadowOffsetX/Y`).
    pub gölge_bulanıklığı: f32,
    pub gölge_rengi: Option<Renk>,
    pub gölge_kayması: (f32, f32),
}

impl Default for ÇizgiStili {
    fn default() -> Self {
        ÇizgiStili {
            renk: None,
            kalınlık: 2.0,
            tür: ÇizgiTürü::Düz,
            opaklık: 1.0,
            gölge_bulanıklığı: 0.0,
            gölge_rengi: None,
            gölge_kayması: (0.0, 0.0),
        }
    }
}

impl ÇizgiStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn kalınlık(mut self, kalınlık: f32) -> Self {
        self.kalınlık = kalınlık;
        self
    }

    pub fn tür(mut self, tür: ÇizgiTürü) -> Self {
        self.tür = tür;
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = opaklık;
        self
    }

    pub fn gölge_bulanıklığı(mut self, bulanıklık: f32) -> Self {
        self.gölge_bulanıklığı = bulanıklık.max(0.0);
        self
    }

    pub fn gölge_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.gölge_rengi = Some(renk.into());
        self
    }

    pub fn gölge_kayması(mut self, x: f32, y: f32) -> Self {
        self.gölge_kayması = (x, y);
        self
    }
}

/// Öğe stili (`itemStyle`): sembol, sütun, dilim vb. dolgusu ve kenarlığı.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ÖğeStili {
    pub renk: Option<Dolgu>,
    pub kenarlık_rengi: Option<Renk>,
    pub kenarlık_kalınlığı: f32,
    /// Öğe kenarlığının çizgi türü (`itemStyle.borderType`).
    pub kenarlık_türü: ÇizgiTürü,
    /// Köşe yarıçapları: `[sol üst, sağ üst, sağ alt, sol alt]`
    /// (`itemStyle.borderRadius`).
    pub kenarlık_yarıçapı: [f32; 4],
    pub opaklık: Option<f32>,
    /// Canvas/zrender şekil gölgesi (`shadowBlur`, `shadowColor`,
    /// `shadowOffsetX/Y`).
    pub gölge_bulanıklığı: f32,
    pub gölge_rengi: Option<Renk>,
    pub gölge_kayması: (f32, f32),
}

impl ÖğeStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn renk(mut self, renk: impl Into<Dolgu>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn kenarlık_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.kenarlık_rengi = Some(renk.into());
        self
    }

    pub fn kenarlık_kalınlığı(mut self, kalınlık: f32) -> Self {
        self.kenarlık_kalınlığı = kalınlık;
        self
    }

    pub fn kenarlık_türü(mut self, tür: ÇizgiTürü) -> Self {
        self.kenarlık_türü = tür;
        self
    }

    pub fn kenarlık_yarıçapı(mut self, yarıçap: impl Into<KöşeYarıçapı>) -> Self {
        self.kenarlık_yarıçapı = yarıçap.into().0;
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = Some(opaklık);
        self
    }

    pub fn gölge_bulanıklığı(mut self, bulanıklık: f32) -> Self {
        self.gölge_bulanıklığı = bulanıklık.max(0.0);
        self
    }

    pub fn gölge_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.gölge_rengi = Some(renk.into());
        self
    }

    pub fn gölge_kayması(mut self, x: f32, y: f32) -> Self {
        self.gölge_kayması = (x, y);
        self
    }
}

/// Köşe yarıçapı belirtimi: tek sayı ya da dörtlü dizi.
pub struct KöşeYarıçapı(pub [f32; 4]);

impl From<f32> for KöşeYarıçapı {
    fn from(hepsi: f32) -> Self {
        KöşeYarıçapı([hepsi; 4])
    }
}

impl From<[f32; 4]> for KöşeYarıçapı {
    fn from(dört: [f32; 4]) -> Self {
        KöşeYarıçapı(dört)
    }
}

/// Alan stili (`areaStyle`).
#[derive(Clone, PartialEq, Debug)]
pub struct AlanStili {
    pub renk: Option<Dolgu>,
    /// ECharts öntanımlısı 0.7'dir.
    pub opaklık: f32,
}

impl Default for AlanStili {
    fn default() -> Self {
        AlanStili {
            renk: None,
            opaklık: 0.7,
        }
    }
}

impl AlanStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn renk(mut self, renk: impl Into<Dolgu>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = opaklık;
        self
    }
}

/// Yazı stili (`textStyle`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct YazıStili {
    pub renk: Option<Renk>,
    /// Metin ve arka planın ortak opaklığı (`opacity`).
    pub opaklık: Option<f32>,
    pub boyut: Option<f32>,
    /// Açık satır yüksekliği (`lineHeight`). `None` ise yazı boyutu.
    pub satır_yüksekliği: Option<f32>,
    pub kalın: bool,
    /// `fontWeight` açıkça ayarlandı mı? Bileşenlerin kalıtılan
    /// öntanımlılarını (başlıkta kalın, çoğu etikette normal) korur.
    pub kalınlık_belirtildi: bool,
    pub aile: Option<String>,
    /// Metin kutusu/rich-text parçası arka planı (`backgroundColor`).
    pub arkaplan: Option<Dolgu>,
    pub kenarlık_rengi: Option<Renk>,
    pub kenarlık_kalınlığı: Option<f32>,
    /// CSS sıralı köşe yarıçapları: sol üst, sağ üst, sağ alt, sol alt.
    pub kenarlık_yarıçapları: Option<[f32; 4]>,
    /// CSS sıralı iç boşluk: üst, sağ, alt, sol.
    pub iç_boşluk: Option<[f32; 4]>,
    /// Açık rich-text içerik genişliği/yüksekliği. Genişlik yüzde olabilir.
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<f32>,
    pub yatay_hiza: Option<YazıYatayHizası>,
    pub dikey_hiza: Option<YazıDikeyHizası>,
    /// `overflow: 'truncate'`; açık değilse ECharts/zrender'ın öntanımlı
    /// taşan metin davranışı korunur.
    pub taşmayı_kısalt: bool,
}

impl YazıStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.opaklık = Some(opaklık.clamp(0.0, 1.0));
        self
    }

    pub fn boyut(mut self, boyut: f32) -> Self {
        self.boyut = Some(boyut);
        self
    }

    pub fn satır_yüksekliği(mut self, yükseklik: f32) -> Self {
        self.satır_yüksekliği = Some(yükseklik.max(0.0));
        self
    }

    pub fn kalın(mut self, kalın: bool) -> Self {
        self.kalın = kalın;
        self.kalınlık_belirtildi = true;
        self
    }

    pub fn aile(mut self, aile: impl Into<String>) -> Self {
        self.aile = Some(aile.into());
        self
    }

    pub fn arkaplan(mut self, arkaplan: impl Into<Dolgu>) -> Self {
        self.arkaplan = Some(arkaplan.into());
        self
    }

    pub fn kenarlık_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.kenarlık_rengi = Some(renk.into());
        self
    }

    pub fn kenarlık_kalınlığı(mut self, kalınlık: f32) -> Self {
        self.kenarlık_kalınlığı = Some(kalınlık.max(0.0));
        self
    }

    pub fn kenarlık_yarıçapı(mut self, yarıçap: f32) -> Self {
        self.kenarlık_yarıçapları = Some([yarıçap.max(0.0); 4]);
        self
    }

    pub fn kenarlık_yarıçapları(mut self, yarıçaplar: [f32; 4]) -> Self {
        self.kenarlık_yarıçapları = Some(yarıçaplar.map(|yarıçap| yarıçap.max(0.0)));
        self
    }

    pub fn iç_boşluk(mut self, boşluk: [f32; 4]) -> Self {
        // zrender rich-text padding değerlerini olduğu gibi kullanır.
        // Özellikle gauge-speed örneğindeki `[0, 0, -20, 10]`, birim
        // koşusunu taban çizgisine doğru kaydırmak için kasıtlıdır.
        self.iç_boşluk = Some(boşluk.map(|değer| if değer.is_finite() { değer } else { 0.0 }));
        self
    }

    pub fn eş_iç_boşluk(mut self, boşluk: f32) -> Self {
        self.iç_boşluk = Some([if boşluk.is_finite() { boşluk } else { 0.0 }; 4]);
        self
    }

    pub fn genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(genişlik.into());
        self
    }

    pub fn yükseklik(mut self, yükseklik: f32) -> Self {
        self.yükseklik = Some(yükseklik.max(0.0));
        self
    }

    pub fn yatay_hiza(mut self, hiza: YazıYatayHizası) -> Self {
        self.yatay_hiza = Some(hiza);
        self
    }

    pub fn dikey_hiza(mut self, hiza: YazıDikeyHizası) -> Self {
        self.dikey_hiza = Some(hiza);
        self
    }

    pub fn taşmayı_kısalt(mut self, kısalt: bool) -> Self {
        self.taşmayı_kısalt = kısalt;
        self
    }

    /// Bir rich-text/veri öğesi yamasını mevcut yazı stilinin üzerine
    /// uygular; yalnız açık alanlar kalıtılan değeri değiştirir.
    pub(crate) fn yama_uygula(&self, yama: &YazıStili) -> YazıStili {
        let mut sonuç = self.clone();
        if yama.renk.is_some() {
            sonuç.renk = yama.renk;
        }
        if yama.opaklık.is_some() {
            sonuç.opaklık = yama.opaklık;
        }
        if yama.boyut.is_some() {
            sonuç.boyut = yama.boyut;
        }
        if yama.satır_yüksekliği.is_some() {
            sonuç.satır_yüksekliği = yama.satır_yüksekliği;
        }
        if yama.kalınlık_belirtildi {
            sonuç.kalın = yama.kalın;
            sonuç.kalınlık_belirtildi = true;
        }
        if yama.aile.is_some() {
            sonuç.aile.clone_from(&yama.aile);
        }
        if yama.arkaplan.is_some() {
            sonuç.arkaplan.clone_from(&yama.arkaplan);
        }
        if yama.kenarlık_rengi.is_some() {
            sonuç.kenarlık_rengi = yama.kenarlık_rengi;
        }
        if yama.kenarlık_kalınlığı.is_some() {
            sonuç.kenarlık_kalınlığı = yama.kenarlık_kalınlığı;
        }
        if yama.kenarlık_yarıçapları.is_some() {
            sonuç.kenarlık_yarıçapları = yama.kenarlık_yarıçapları;
        }
        if yama.iç_boşluk.is_some() {
            sonuç.iç_boşluk = yama.iç_boşluk;
        }
        if yama.genişlik.is_some() {
            sonuç.genişlik = yama.genişlik;
        }
        if yama.yükseklik.is_some() {
            sonuç.yükseklik = yama.yükseklik;
        }
        if yama.yatay_hiza.is_some() {
            sonuç.yatay_hiza = yama.yatay_hiza;
        }
        if yama.dikey_hiza.is_some() {
            sonuç.dikey_hiza = yama.dikey_hiza;
        }
        if yama.taşmayı_kısalt {
            sonuç.taşmayı_kısalt = true;
        }
        sonuç
    }
}

/// Etiket konumu (`label.position`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EtiketKonumu {
    Üst,
    Alt,
    Sol,
    Sağ,
    /// Funnel dış etiket köşeleri (`rightTop`, `rightBottom`,
    /// `leftTop`, `leftBottom`).
    SağÜst,
    SağAlt,
    SolÜst,
    SolAlt,
    /// Kutupsal sütunda değer aralığının başlangıcı (`'start'`).
    Başlangıç,
    /// Kutupsal sütunda başlangıcın şekil içindeki yanı
    /// (`'insideStart'`).
    İçBaşlangıç,
    /// Kutupsal sütunda değer aralığının bitişi (`'end'`).
    Bitiş,
    /// Kutupsal sütunda bitişin şekil içindeki yanı (`'insideEnd'`).
    İçBitiş,
    #[default]
    İç,
    İçÜst,
    İçAlt,
    İçSol,
    İçSağ,
    İçSolÜst,
    İçSağÜst,
    İçSolAlt,
    İçSağAlt,
    Dış,
    Merkez,
}

/// Pasta dış etiketinin yatay hizalama stratejisi (`label.alignTo`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DışEtiketHizası {
    /// Etiket kendi kırık çizgi ucunu izler (`'none'`).
    #[default]
    Yok,
    /// Aynı taraftaki etiketleri en uzak label-line ucuna hizalar.
    EtiketÇizgisi,
    /// Metni seri görünüm kutusunun kenarına hizalar.
    Kenar,
}

/// Etiket döndürme kipi (`label.rotate`).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum EtiketDöndürme {
    #[default]
    Yok,
    Derece(f32),
    Radyal,
    Teğetsel,
    /// ECharts `'tangential-noflip'`.
    TeğetselÇevirmesiz,
}

/// Açık `label.align` değeri. Verilmediğinde şekil/konum eşlemesi kendi
/// doğal hizasını seçer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum YazıYatayHizası {
    Sol,
    Orta,
    Sağ,
}

/// Açık `label.verticalAlign` değeri.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum YazıDikeyHizası {
    Üst,
    Orta,
    Alt,
}

/// Biçimleyici işlev imzası: `(değer, ham metin) → biçimli metin`.
pub type Biçimleyiciİşlevi = Arc<dyn Fn(f64, &str) -> String + Send + Sync>;

/// Değer biçimleyici: `"{value} °C"` tarzı şablon ya da işlev.
#[derive(Clone)]
pub enum Biçimleyici {
    /// `{value}` yer tutucusu değerle değiştirilir; kategori eksenlerinde
    /// `{value}` kategori adıdır.
    Şablon(String),
    İşlev(Biçimleyiciİşlevi),
}

impl Biçimleyici {
    pub fn uygula(&self, değer: f64, metin: &str) -> String {
        self.uygula_bağlamla(değer, metin, "", "")
    }

    /// ECharts seri etiketi şablonundaki ortak yer tutucuları uygular:
    /// `{a}` seri adı, `{b}` veri/kategori adı, `{c}` ham değer ve
    /// Cizelge'nin geriye uyumlu `{value}` eş adı.
    pub fn uygula_bağlamla(
        &self,
        değer: f64,
        metin: &str,
        seri_adı: &str,
        veri_adı: &str,
    ) -> String {
        zengin_metin_içeriği(self.uygula_bağlamla_zengin(değer, metin, seri_adı, veri_adı))
    }

    /// Yer tutucuları çözerken ECharts rich-text belirteçlerini korur.
    /// Çizim motorları koşuları kendi stil haritasıyla ölçmek/çizmek
    /// istediğinde bu yol kullanılır.
    pub(crate) fn uygula_bağlamla_zengin(
        &self,
        değer: f64,
        metin: &str,
        seri_adı: &str,
        veri_adı: &str,
    ) -> String {
        match self {
            Biçimleyici::Şablon(ş) => ş
                .replace("{value}", metin)
                .replace("{a}", seri_adı)
                .replace("{b}", veri_adı)
                .replace("{c}", metin),
            Biçimleyici::İşlev(f) => f(değer, metin),
        }
    }
}

/// ECharts rich-text belirtecinin görünen içeriğini çıkarır. Stil parçaları
/// ayrı metin koşularına dönüştürülmeden önce de düz yüzeylerde doğru metin
/// içeriği korunur: `{name|Forest}` → `Forest`.
pub(crate) fn zengin_metin_içeriği(mut metin: String) -> String {
    let mut tarama = 0;
    while let Some(göreli_açılış) = metin.get(tarama..).and_then(|kalan| kalan.find('{')) {
        let açılış = tarama + göreli_açılış;
        let Some(göreli_kapanış) = metin.get(açılış + 1..).and_then(|kalan| kalan.find('}'))
        else {
            break;
        };
        let kapanış = açılış + 1 + göreli_kapanış;
        let belirteç = metin.get(açılış + 1..kapanış).unwrap_or_default();
        let Some(göreli_boru) = belirteç.find('|') else {
            // Tanımadığımız normal bir yer tutucusu daha ilerideki zengin
            // metin koşusunun bulunmasını engellememeli.
            tarama = kapanış + 1;
            continue;
        };
        let boru = açılış + 1 + göreli_boru;
        let içerik = metin.get(boru + 1..kapanış).unwrap_or_default().to_owned();
        metin.replace_range(açılış..=kapanış, &içerik);
        tarama = açılış + içerik.len();
    }
    metin
}

impl fmt::Debug for Biçimleyici {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Biçimleyici::Şablon(ş) => f.debug_tuple("Şablon").field(ş).finish(),
            Biçimleyici::İşlev(_) => f.write_str("İşlev(..)"),
        }
    }
}

impl PartialEq for Biçimleyici {
    fn eq(&self, diğer: &Self) -> bool {
        match (self, diğer) {
            (Biçimleyici::Şablon(a), Biçimleyici::Şablon(b)) => a == b,
            (Biçimleyici::İşlev(a), Biçimleyici::İşlev(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl From<&str> for Biçimleyici {
    fn from(ş: &str) -> Self {
        Biçimleyici::Şablon(ş.to_string())
    }
}

impl From<String> for Biçimleyici {
    fn from(ş: String) -> Self {
        Biçimleyici::Şablon(ş)
    }
}

/// Veri etiketi (`label`).
#[derive(Clone, PartialEq, Debug)]
pub struct Etiket {
    pub göster: bool,
    pub konum: EtiketKonumu,
    /// Ana şekille hesaplanan konuma eklenen piksel kayması
    /// (`label.offset: [x, y]`).
    pub kayma: (f32, f32),
    pub biçimleyici: Option<Biçimleyici>,
    pub yazı: YazıStili,
    /// `label.rich`: adlandırılmış metin koşularının taban yazı stilinin
    /// üstüne uygulanan yamaları.
    pub zengin: BTreeMap<String, YazıStili>,
    /// Ana şekille etiket arasındaki uzaklık (`label.distance`).
    pub uzaklık: f32,
    pub dış_hiza: DışEtiketHizası,
    /// `edgeDistance`; seri görünüm kutusunun genişliğine göre çözülür.
    pub kenar_uzaklığı: Uzunluk,
    /// `bleedMargin`; `None`, görünüm boyutuna göre 10/2 px otomatik kuralıdır.
    pub taşma_payını: Option<f32>,
    /// Metin ile label-line sonu arasındaki uzaklık.
    pub çizgi_uzaklığı: f32,
    /// Geriye uyumlu `label.margin`. ECharts'ın pie ön işlemcisi bunu yalnız
    /// `alignTo: 'edge'` iken, açık `edgeDistance` yoksa piksel cinsinden
    /// `edgeDistance` değerine dönüştürür.
    pub kenar_boşluğu: f32,
    /// Komşu etiket kutuları arasındaki en küçük boşluk (`minMargin`).
    pub en_küçük_boşluk: f32,
    pub döndürme: EtiketDöndürme,
    pub yatay_hiza: Option<YazıYatayHizası>,
    pub dikey_hiza: Option<YazıDikeyHizası>,
}

impl Default for Etiket {
    fn default() -> Self {
        Self {
            göster: false,
            // zrender bağlı metin öntanımlısıdır. Line/Pie/Funnel gibi
            // seriler kendi resmi konum öntanımlarını seri modelinde koyar.
            konum: EtiketKonumu::İç,
            kayma: (0.0, 0.0),
            biçimleyici: None,
            yazı: YazıStili::default(),
            zengin: BTreeMap::new(),
            uzaklık: 5.0,
            dış_hiza: DışEtiketHizası::Yok,
            kenar_uzaklığı: Uzunluk::Yüzde(25.0),
            taşma_payını: None,
            çizgi_uzaklığı: 5.0,
            kenar_boşluğu: 0.0,
            // Pie etiket yerleşimi `computeLabelGeometry` çağrısında açık
            // `minMargin` yokken üst/alta birer piksel `marginDefault`
            // uygular. Toplam dikey yerleşim payı bu nedenle 2 px'dir.
            en_küçük_boşluk: 2.0,
            döndürme: EtiketDöndürme::Yok,
            yatay_hiza: None,
            dikey_hiza: None,
        }
    }
}

impl Etiket {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn konum(mut self, konum: EtiketKonumu) -> Self {
        self.konum = konum;
        self
    }

    pub fn kayma(mut self, x: f32, y: f32) -> Self {
        self.kayma = (x, y);
        self
    }

    pub fn biçimleyici(mut self, b: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(b.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn zengin_stil(mut self, ad: impl Into<String>, yazı: YazıStili) -> Self {
        self.zengin.insert(ad.into(), yazı);
        self
    }

    pub fn uzaklık(mut self, uzaklık: f32) -> Self {
        self.uzaklık = uzaklık.max(0.0);
        self
    }

    pub fn dış_hiza(mut self, hiza: DışEtiketHizası) -> Self {
        self.dış_hiza = hiza;
        self
    }

    pub fn kenar_uzaklığı(mut self, uzaklık: impl Into<Uzunluk>) -> Self {
        self.kenar_uzaklığı = uzaklık.into();
        self
    }

    pub fn taşma_payını(mut self, pay: f32) -> Self {
        self.taşma_payını = Some(pay.max(0.0));
        self
    }

    pub fn otomatik_taşma_payını(mut self) -> Self {
        self.taşma_payını = None;
        self
    }

    pub fn çizgi_uzaklığı(mut self, uzaklık: f32) -> Self {
        self.çizgi_uzaklığı = uzaklık.max(0.0);
        self
    }

    pub fn kenar_boşluğu(mut self, boşluk: f32) -> Self {
        self.kenar_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn en_küçük_boşluk(mut self, boşluk: f32) -> Self {
        self.en_küçük_boşluk = boşluk.max(0.0);
        self
    }

    pub fn döndürme(mut self, döndürme: EtiketDöndürme) -> Self {
        self.döndürme = döndürme;
        self
    }

    pub fn yatay_hiza(mut self, hiza: YazıYatayHizası) -> Self {
        self.yatay_hiza = Some(hiza);
        self
    }

    pub fn dikey_hiza(mut self, hiza: YazıDikeyHizası) -> Self {
        self.dikey_hiza = Some(hiza);
        self
    }
}

/// Nesne biçimli seri verisindeki `data[i].label` yaması. ECharts veri
/// öğesi seçeneklerini seri etiket modelinin üstüne miras yoluyla uygular;
/// yalnız `position` veren bir öğe, serideki `show` ve `formatter`ı
/// kaybetmez.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct EtiketYaması {
    pub göster: Option<bool>,
    pub konum: Option<EtiketKonumu>,
    pub kayma: Option<(f32, f32)>,
    pub biçimleyici: Option<Biçimleyici>,
    pub yazı: Option<YazıStili>,
    pub zengin: Option<BTreeMap<String, YazıStili>>,
    pub uzaklık: Option<f32>,
    pub dış_hiza: Option<DışEtiketHizası>,
    pub kenar_uzaklığı: Option<Uzunluk>,
    pub taşma_payını: Option<f32>,
    pub çizgi_uzaklığı: Option<f32>,
    pub kenar_boşluğu: Option<f32>,
    pub en_küçük_boşluk: Option<f32>,
    pub döndürme: Option<EtiketDöndürme>,
    pub yatay_hiza: Option<YazıYatayHizası>,
    pub dikey_hiza: Option<YazıDikeyHizası>,
}

impl EtiketYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = Some(göster);
        self
    }

    pub fn konum(mut self, konum: EtiketKonumu) -> Self {
        self.konum = Some(konum);
        self
    }

    pub fn kayma(mut self, x: f32, y: f32) -> Self {
        self.kayma = Some((x, y));
        self
    }

    pub fn biçimleyici(mut self, biçimleyici: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(biçimleyici.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = Some(yazı);
        self
    }

    pub fn zengin_stil(mut self, ad: impl Into<String>, yazı: YazıStili) -> Self {
        self.zengin
            .get_or_insert_with(BTreeMap::new)
            .insert(ad.into(), yazı);
        self
    }

    pub fn uzaklık(mut self, uzaklık: f32) -> Self {
        self.uzaklık = Some(uzaklık.max(0.0));
        self
    }

    pub fn yatay_hiza(mut self, hiza: YazıYatayHizası) -> Self {
        self.yatay_hiza = Some(hiza);
        self
    }

    pub fn dikey_hiza(mut self, hiza: YazıDikeyHizası) -> Self {
        self.dikey_hiza = Some(hiza);
        self
    }

    pub fn uygula(&self, taban: &Etiket) -> Etiket {
        let mut sonuç = taban.clone();
        if let Some(değer) = self.göster {
            sonuç.göster = değer;
        }
        if let Some(değer) = self.konum {
            sonuç.konum = değer;
        }
        if let Some(değer) = self.kayma {
            sonuç.kayma = değer;
        }
        if let Some(değer) = &self.biçimleyici {
            sonuç.biçimleyici = Some(değer.clone());
        }
        if let Some(değer) = &self.yazı {
            sonuç.yazı = sonuç.yazı.yama_uygula(değer);
        }
        if let Some(değer) = &self.zengin {
            sonuç.zengin.extend(değer.clone());
        }
        if let Some(değer) = self.uzaklık {
            sonuç.uzaklık = değer;
        }
        if let Some(değer) = self.dış_hiza {
            sonuç.dış_hiza = değer;
        }
        if let Some(değer) = self.kenar_uzaklığı {
            sonuç.kenar_uzaklığı = değer;
        }
        if let Some(değer) = self.taşma_payını {
            sonuç.taşma_payını = Some(değer);
        }
        if let Some(değer) = self.çizgi_uzaklığı {
            sonuç.çizgi_uzaklığı = değer;
        }
        if let Some(değer) = self.kenar_boşluğu {
            sonuç.kenar_boşluğu = değer;
        }
        if let Some(değer) = self.en_küçük_boşluk {
            sonuç.en_küçük_boşluk = değer;
        }
        if let Some(değer) = self.döndürme {
            sonuç.döndürme = değer;
        }
        if let Some(değer) = self.yatay_hiza {
            sonuç.yatay_hiza = Some(değer);
        }
        if let Some(değer) = self.dikey_hiza {
            sonuç.dikey_hiza = Some(değer);
        }
        sonuç
    }
}

impl From<Etiket> for EtiketYaması {
    fn from(etiket: Etiket) -> Self {
        EtiketYaması {
            göster: Some(etiket.göster),
            konum: Some(etiket.konum),
            kayma: Some(etiket.kayma),
            biçimleyici: etiket.biçimleyici,
            yazı: Some(etiket.yazı),
            zengin: Some(etiket.zengin),
            uzaklık: Some(etiket.uzaklık),
            dış_hiza: Some(etiket.dış_hiza),
            kenar_uzaklığı: Some(etiket.kenar_uzaklığı),
            taşma_payını: etiket.taşma_payını,
            çizgi_uzaklığı: Some(etiket.çizgi_uzaklığı),
            kenar_boşluğu: Some(etiket.kenar_boşluğu),
            en_küçük_boşluk: Some(etiket.en_küçük_boşluk),
            döndürme: Some(etiket.döndürme),
            yatay_hiza: etiket.yatay_hiza,
            dikey_hiza: etiket.dikey_hiza,
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn pasta_etiketi_resmi_hizalama_varsayılanlarını_taşır() {
        let etiket = Etiket::default();
        assert_eq!(etiket.dış_hiza, DışEtiketHizası::Yok);
        assert_eq!(etiket.kenar_uzaklığı, Uzunluk::Yüzde(25.0));
        assert_eq!(etiket.taşma_payını, None);
        assert_eq!(etiket.çizgi_uzaklığı, 5.0);
        assert_eq!(etiket.döndürme, EtiketDöndürme::Yok);
    }

    #[test]
    fn pasta_etiketi_hizalama_builderları_bütün_yolları_korur() {
        let etiket = Etiket::yeni()
            .dış_hiza(DışEtiketHizası::Kenar)
            .kenar_uzaklığı("12%")
            .taşma_payını(7.0)
            .çizgi_uzaklığı(9.0)
            .kenar_boşluğu(3.0)
            .döndürme(EtiketDöndürme::Radyal);
        assert_eq!(etiket.dış_hiza, DışEtiketHizası::Kenar);
        assert_eq!(etiket.kenar_uzaklığı, Uzunluk::Yüzde(12.0));
        assert_eq!(etiket.taşma_payını, Some(7.0));
        assert_eq!(etiket.çizgi_uzaklığı, 9.0);
        assert_eq!(etiket.kenar_boşluğu, 3.0);
        assert_eq!(etiket.döndürme, EtiketDöndürme::Radyal);
    }

    #[test]
    fn veri_etiketi_yaması_seri_alanlarını_miras_alır() {
        let seri = Etiket::yeni()
            .göster(true)
            .biçimleyici("{a}/{b}/{c}")
            .kayma(3.0, -4.0)
            .uzaklık(7.0);
        let sonuç = EtiketYaması::yeni().konum(EtiketKonumu::Sağ).uygula(&seri);
        assert!(sonuç.göster);
        assert_eq!(sonuç.konum, EtiketKonumu::Sağ);
        assert_eq!(sonuç.kayma, (3.0, -4.0));
        assert_eq!(sonuç.uzaklık, 7.0);
        assert_eq!(
            sonuç
                .biçimleyici
                .as_ref()
                .map(|b| b.uygula_bağlamla(12.0, "12", "Seri", "Öğe")),
            Some("Seri/Öğe/12".to_owned())
        );
    }

    #[test]
    fn ortak_etiket_konumu_zrender_gibi_içtir() {
        assert_eq!(Etiket::default().konum, EtiketKonumu::İç);
    }

    #[test]
    fn zengin_metin_belirteci_görünen_içeriğe_indirgenir() {
        let biçimleyici = Biçimleyici::from("{c}  {name|{a}}");
        assert_eq!(
            biçimleyici.uygula_bağlamla(320.0, "320", "Forest", "2012"),
            "320  Forest"
        );

        let bilinmeyen = Biçimleyici::from("{x} / {name|Forest}");
        assert_eq!(bilinmeyen.uygula_bağlamla(0.0, "0", "", ""), "{x} / Forest");
    }

    #[test]
    fn zengin_metin_negatif_padding_degerini_korur() {
        let yazı = YazıStili::yeni().iç_boşluk([0.0, 0.0, -20.0, 10.0]);
        assert_eq!(yazı.iç_boşluk, Some([0.0, 0.0, -20.0, 10.0]));
        assert_eq!(
            YazıStili::yeni().eş_iç_boşluk(-3.0).iç_boşluk,
            Some([-3.0; 4])
        );
    }

    #[test]
    fn veri_etiketi_yaması_açık_hizaları_mirasın_üstüne_yazar() {
        let taban = Etiket::yeni()
            .yatay_hiza(YazıYatayHizası::Sağ)
            .dikey_hiza(YazıDikeyHizası::Üst);
        let sonuç = EtiketYaması::yeni()
            .yatay_hiza(YazıYatayHizası::Sol)
            .dikey_hiza(YazıDikeyHizası::Orta)
            .uygula(&taban);
        assert_eq!(sonuç.yatay_hiza, Some(YazıYatayHizası::Sol));
        assert_eq!(sonuç.dikey_hiza, Some(YazıDikeyHizası::Orta));
    }
}
