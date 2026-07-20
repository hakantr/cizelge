//! Bileşen seçenekleri — `title`, `legend`, `grid`, `tooltip` tanımlarının
//! karşılığı.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::model::deger::VeriDeğeri;
use crate::model::stil::{Biçimleyici, YazıStili};
use crate::model::{DikeyKonum, Uzunluk, YatayKonum};
use crate::renk::Renk;

/// Başlık metninin açık yatay hizası (`title.textAlign`).
///
/// `None` bırakıldığında ECharts gibi `left` değerinden türetilir. Açık bir
/// değer verildiğinde `left`, metnin doğrudan çapasıdır; özellikle birden çok
/// başlığın yüzde konumlarına ortalanmasında bu ayrım önemlidir.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BaşlıkMetinHizası {
    Sol,
    Orta,
    Sağ,
}

/// Başlık bileşeni (`title`).
#[derive(Clone, PartialEq, Debug)]
pub struct Başlık {
    pub göster: bool,
    pub metin: Option<String>,
    pub alt_metin: Option<String>,
    pub sol: YatayKonum,
    pub metin_hizası: Option<BaşlıkMetinHizası>,
    /// Üst kenardan uzaklık; öntanımlı iç boşluk kadar.
    pub üst: Option<Uzunluk>,
    /// Başlık kutusunun CSS sıralı eş boşluğu (`padding`).
    pub iç_boşluk: f32,
    /// Ana ve alt başlık arasındaki boşluk (`itemGap`).
    pub öğe_boşluğu: f32,
    /// Başlık kutusu dolgusu ve kenarlığı.
    pub arkaplan: Option<Renk>,
    pub kenarlık_rengi: Option<Renk>,
    pub kenarlık_kalınlığı: f32,
    pub kenarlık_yarıçapı: [f32; 4],
    pub yazı: YazıStili,
    pub alt_yazı: YazıStili,
}

impl Default for Başlık {
    fn default() -> Self {
        Self {
            göster: true,
            metin: None,
            alt_metin: None,
            sol: YatayKonum::Orta,
            metin_hizası: None,
            üst: Some(Uzunluk::Piksel(15.0)),
            iç_boşluk: 5.0,
            öğe_boşluğu: 10.0,
            arkaplan: None,
            kenarlık_rengi: None,
            kenarlık_kalınlığı: 0.0,
            kenarlık_yarıçapı: [0.0; 4],
            yazı: YazıStili::default(),
            alt_yazı: YazıStili::default(),
        }
    }
}

impl Başlık {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn metin(mut self, metin: impl Into<String>) -> Self {
        self.metin = Some(metin.into());
        self
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn alt_metin(mut self, alt: impl Into<String>) -> Self {
        self.alt_metin = Some(alt.into());
        self
    }

    pub fn sol(mut self, sol: impl Into<YatayKonum>) -> Self {
        self.sol = sol.into();
        self
    }

    pub fn metin_hizası(mut self, hiza: BaşlıkMetinHizası) -> Self {
        self.metin_hizası = Some(hiza);
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = Some(üst.into());
        self
    }

    pub fn iç_boşluk(mut self, boşluk: f32) -> Self {
        self.iç_boşluk = boşluk.max(0.0);
        self
    }

    pub fn öğe_boşluğu(mut self, boşluk: f32) -> Self {
        self.öğe_boşluğu = boşluk.max(0.0);
        self
    }

    pub fn arkaplan(mut self, renk: impl Into<Renk>) -> Self {
        self.arkaplan = Some(renk.into());
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

    pub fn kenarlık_yarıçapı(
        mut self,
        yarıçap: impl Into<crate::model::stil::KöşeYarıçapı>,
    ) -> Self {
        self.kenarlık_yarıçapı = yarıçap.into().0;
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn alt_yazı(mut self, yazı: YazıStili) -> Self {
        self.alt_yazı = yazı;
        self
    }
}

/// Gösterge yerleşim yönü (`legend.orient`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Yön {
    #[default]
    Yatay,
    Dikey,
}

/// Gösterge simgesi (`legend.icon`); `None` seri türüne göre seçilir.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GöstergeSimgesi {
    YuvarlakKöşeliKare,
    Daire,
    Çizgi,
}

/// `legend.selectedMode`: kapalı, çoklu (`true`) veya tekli (`single`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GöstergeSeçimKipi {
    Kapalı,
    #[default]
    Çoklu,
    Tek,
}

/// Gösterge bileşeni (`legend`).
#[derive(Clone, PartialEq, Debug)]
pub struct Gösterge {
    pub göster: bool,
    pub yön: Yön,
    pub sol: YatayKonum,
    /// Sağ kenardan uzaklık (`right`). `Some` olduğunda yatay yerleşimde
    /// `left` yerine kullanılır.
    pub sağ: Option<Uzunluk>,
    pub üst: Option<Uzunluk>,
    /// Alt kenardan uzaklık (`bottom`); ECharts 6.1 öntanımlısı 15.
    pub alt: Option<Uzunluk>,
    /// Legend kutusunun iç boşluğu (`padding`).
    pub iç_boşluk: f32,
    /// Simge genişliği (`itemWidth`, öntanımlı 25).
    pub simge_genişliği: f32,
    /// Simge yüksekliği (`itemHeight`, öntanımlı 14).
    pub simge_yüksekliği: f32,
    /// Öğeler arası boşluk (`itemGap`, öntanımlı 10).
    pub öğe_boşluğu: f32,
    pub yazı: YazıStili,
    pub simge: Option<GöstergeSimgesi>,
    /// Yalnızca bu adlar gösterilsin (`legend.data`); boşsa hepsi.
    pub veri: Vec<String>,
    /// Sığmayan öğeler için sayfalı kaydırma (`type: 'scroll'`).
    pub kaydırılabilir: bool,
    /// Kullanıcı/programatik seçim kipi (`selectedMode`).
    pub seçim_kipi: GöstergeSeçimKipi,
    /// Ad → seçili haritası. Haritada bulunmayan ad ECharts gibi seçilidir.
    pub seçili: BTreeMap<String, bool>,
    /// `selector: true`: tümünü seç / tersini seç düğmeleri.
    pub seçiciler: bool,
    /// Kapalı öğe rengi (`inactiveColor`).
    pub devre_dışı_rengi: Renk,
    /// Legend ad biçimleyicisi (`formatter`).
    pub biçimleyici: Option<Biçimleyici>,
}

impl Default for Gösterge {
    fn default() -> Self {
        Gösterge {
            göster: true,
            yön: Yön::Yatay,
            sol: YatayKonum::Orta,
            sağ: None,
            üst: None,
            alt: Some(Uzunluk::Piksel(15.0)),
            iç_boşluk: 5.0,
            simge_genişliği: 25.0,
            simge_yüksekliği: 14.0,
            öğe_boşluğu: 8.0,
            yazı: YazıStili::default(),
            simge: None,
            veri: Vec::new(),
            kaydırılabilir: false,
            seçim_kipi: GöstergeSeçimKipi::Çoklu,
            seçili: BTreeMap::new(),
            seçiciler: false,
            devre_dışı_rengi: Renk::onaltılık(0xcccccc),
            biçimleyici: None,
        }
    }
}

impl Gösterge {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn yön(mut self, yön: Yön) -> Self {
        self.yön = yön;
        self
    }

    pub fn sol(mut self, sol: impl Into<YatayKonum>) -> Self {
        self.sol = sol.into();
        self.sağ = None;
        self
    }

    /// Gösterge kutusunu sağ kenara göre yerleştirir (`right`).
    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(sağ.into());
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = Some(üst.into());
        self.alt = None;
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = Some(alt.into());
        self.üst = None;
        self
    }

    pub fn iç_boşluk(mut self, boşluk: f32) -> Self {
        self.iç_boşluk = boşluk.max(0.0);
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn simge(mut self, simge: GöstergeSimgesi) -> Self {
        self.simge = Some(simge);
        self
    }

    pub fn veri<S: Into<String>>(mut self, veri: impl IntoIterator<Item = S>) -> Self {
        self.veri = veri.into_iter().map(Into::into).collect();
        self
    }

    /// Sığmayan öğeleri sayfalı kaydırmayla gösterir (`type: 'scroll'`).
    pub fn kaydırılabilir(mut self, açık: bool) -> Self {
        self.kaydırılabilir = açık;
        self
    }

    pub fn seçim_kipi(mut self, kip: GöstergeSeçimKipi) -> Self {
        self.seçim_kipi = kip;
        self
    }

    pub fn seçili(mut self, ad: impl Into<String>, seçili: bool) -> Self {
        self.seçili.insert(ad.into(), seçili);
        self
    }

    pub fn seçili_haritası(mut self, harita: impl IntoIterator<Item = (String, bool)>) -> Self {
        self.seçili = harita.into_iter().collect();
        self
    }

    pub fn seçiciler(mut self, açık: bool) -> Self {
        self.seçiciler = açık;
        self
    }

    pub fn devre_dışı_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.devre_dışı_rengi = renk.into();
        self
    }

    pub fn biçimleyici(mut self, biçimleyici: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(biçimleyici.into());
        self
    }

    pub fn seçili_mi(&self, ad: &str) -> bool {
        self.seçili.get(ad).copied().unwrap_or(true)
    }

    pub fn seç(&mut self, ad: &str, adlar: &[String]) {
        if self.seçim_kipi == GöstergeSeçimKipi::Tek {
            for aday in adlar {
                self.seçili.insert(aday.clone(), aday == ad);
            }
        } else {
            self.seçili.insert(ad.to_owned(), true);
        }
    }

    pub fn seçimi_kaldır(&mut self, ad: &str) {
        if self.seçim_kipi != GöstergeSeçimKipi::Tek {
            self.seçili.insert(ad.to_owned(), false);
        }
    }

    pub fn seçimi_değiştir(&mut self, ad: &str, adlar: &[String]) {
        if self.seçili_mi(ad) {
            self.seçimi_kaldır(ad);
        } else {
            self.seç(ad, adlar);
        }
    }

    pub fn tümünü_seç(&mut self, adlar: &[String]) {
        for ad in adlar {
            self.seçili.insert(ad.clone(), true);
        }
    }

    pub fn tersini_seç(&mut self, adlar: &[String]) {
        for ad in adlar {
            self.seçili.insert(ad.clone(), !self.seçili_mi(ad));
        }
    }
}

/// Araç kutusu (`toolbox`). Özellikler ECharts'taki `toolbox.feature`
/// nesnesi gibi açıkça etkinleştirilir; boş bir araç kutusu düğme üretmez.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AraçKutusuÖzelliği {
    VeriGörünümü,
    VeriYakınlaştırma,
    SihirliÇizgi,
    SihirliSütun,
    SihirliYığın,
    GeriYükle,
    SvgKaydet,
    PngKaydet,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AraçKutusu {
    pub göster: bool,
    pub yön: Yön,
    pub sol: YatayKonum,
    /// Sağ kenardan açık uzaklık (`toolbox.right`). `Some` olduğunda
    /// ECharts kutu yerleşimindeki `left` karşılığının önüne geçer.
    pub sağ: Option<Uzunluk>,
    pub üst: DikeyKonum,
    /// Yakınlaştırma/gösterge durumunu ilk seçeneklere döndürür
    /// (`feature.restore`).
    pub geri_yükle: bool,
    /// Grafiği SVG dosyası olarak kaydetme düğmesi
    /// (`feature.saveAsImage` karşılığı; çıktı biçimi SVG'dir).
    pub svg_kaydet: bool,
    /// Grafiği PNG dosyası olarak kaydetme düğmesi
    /// (`feature.saveAsImage`, `type: 'png'`; `png` özelliği gerekir).
    pub png_kaydet: bool,
    /// Salt-okunur/güvenli veri görünümü (`feature.dataView`).
    pub veri_görünümü: bool,
    /// Dikdörtgen seçimiyle eksen yakınlaştırma ve geçmişe dönme araçları
    /// (`feature.dataZoom`: `zoom` + `back`).
    pub veri_yakınlaştırma: bool,
    /// Seri türünü çizgiye dönüştürme (`magicType: line`).
    pub sihirli_çizgi: bool,
    /// Seri türünü sütuna dönüştürme (`magicType: bar`).
    pub sihirli_sütun: bool,
    /// `magicType: 'stack'`.
    pub sihirli_yığın: bool,
    /// `toolbox.feature` anahtarlarının eklenme sırası. JavaScript nesne
    /// sırası ECharts'ın düğme sırasını belirlediği için builder çağrıları
    /// aynı sırayı burada korur.
    pub özellik_sırası: Vec<AraçKutusuÖzelliği>,
}

impl Default for AraçKutusu {
    fn default() -> Self {
        AraçKutusu {
            göster: true,
            yön: Yön::Yatay,
            sol: YatayKonum::Sağ,
            sağ: None,
            üst: DikeyKonum::Üst,
            geri_yükle: false,
            svg_kaydet: false,
            png_kaydet: false,
            veri_görünümü: false,
            veri_yakınlaştırma: false,
            sihirli_çizgi: false,
            sihirli_sütun: false,
            sihirli_yığın: false,
            özellik_sırası: Vec::new(),
        }
    }
}

impl AraçKutusu {
    pub fn yeni() -> Self {
        Self::default()
    }

    fn özellik_durumunu_ayarla(&mut self, özellik: AraçKutusuÖzelliği, açık: bool) {
        self.özellik_sırası.retain(|aday| *aday != özellik);
        if açık {
            self.özellik_sırası.push(özellik);
        }
    }

    pub fn yön(mut self, yön: Yön) -> Self {
        self.yön = yön;
        self
    }

    pub fn sol(mut self, sol: impl Into<YatayKonum>) -> Self {
        self.sol = sol.into();
        self.sağ = None;
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(sağ.into());
        self
    }

    pub fn üst(mut self, üst: impl Into<DikeyKonum>) -> Self {
        self.üst = üst.into();
        self
    }

    /// İlk seçenek durumuna dönme düğmesini açar (`feature.restore`).
    pub fn geri_yükle(mut self, açık: bool) -> Self {
        self.geri_yükle = açık;
        self.özellik_durumunu_ayarla(AraçKutusuÖzelliği::GeriYükle, açık);
        self
    }

    /// SVG kaydet düğmesini açar (`saveAsImage`).
    pub fn svg_kaydet(mut self, açık: bool) -> Self {
        self.svg_kaydet = açık;
        self.özellik_durumunu_ayarla(AraçKutusuÖzelliği::SvgKaydet, açık);
        self
    }

    /// PNG kaydet düğmesini açar (`saveAsImage`, `type: 'png'`).
    pub fn png_kaydet(mut self, açık: bool) -> Self {
        self.png_kaydet = açık;
        self.özellik_durumunu_ayarla(AraçKutusuÖzelliği::PngKaydet, açık);
        self
    }

    pub fn veri_görünümü(mut self, açık: bool) -> Self {
        self.veri_görünümü = açık;
        self.özellik_durumunu_ayarla(AraçKutusuÖzelliği::VeriGörünümü, açık);
        self
    }

    /// Eksen seçmeli yakınlaştırma ve geri alma düğmelerini açar
    /// (`toolbox.feature.dataZoom`).
    pub fn veri_yakınlaştırma(mut self, açık: bool) -> Self {
        self.veri_yakınlaştırma = açık;
        self.özellik_durumunu_ayarla(AraçKutusuÖzelliği::VeriYakınlaştırma, açık);
        self
    }

    pub fn sihirli_tür(mut self, çizgi: bool, sütun: bool) -> Self {
        self.sihirli_çizgi = çizgi;
        self.sihirli_sütun = sütun;
        self.özellik_durumunu_ayarla(AraçKutusuÖzelliği::SihirliÇizgi, çizgi);
        self.özellik_durumunu_ayarla(AraçKutusuÖzelliği::SihirliSütun, sütun);
        self
    }

    pub fn sihirli_yığın(mut self, açık: bool) -> Self {
        self.sihirli_yığın = açık;
        self.özellik_durumunu_ayarla(AraçKutusuÖzelliği::SihirliYığın, açık);
        self
    }
}

/// Fırça (`brush`): dikdörtgen seçim.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Fırça {
    pub etkin: bool,
}

impl Fırça {
    pub fn yeni() -> Self {
        Fırça { etkin: true }
    }
}

/// Kartezyen ızgara bileşeni (`grid`).
#[derive(Clone, PartialEq, Debug)]
pub struct Izgara {
    pub sol: Uzunluk,
    pub sağ: Uzunluk,
    pub üst: Uzunluk,
    pub alt: Uzunluk,
    /// İlgili kenarın kullanıcı tarafından açıkça verilip verilmediği.
    /// Sabit boyutlu kutuda yalnız `right`/`bottom` verildiğinde yerleşim
    /// karşı kenardan çözülür; model varsayılanı açık seçenek sayılmaz.
    pub sol_açık: bool,
    pub sağ_açık: bool,
    pub üst_açık: bool,
    pub alt_açık: bool,
    /// Açık genişlik/yükseklik verilirse karşı kenar boşluğunun önüne geçer
    /// (`grid.width` / `grid.height`).
    pub genişlik: Option<Uzunluk>,
    pub yükseklik: Option<Uzunluk>,
    /// Eksen etiketleri ızgara alanına dahil edilsin mi (`containLabel`)?
    pub etiketi_kapsa: bool,
}

impl Default for Izgara {
    fn default() -> Self {
        // ECharts 6.1 `GridModel.defaultOption` öntanımlıları.
        Izgara {
            sol: Uzunluk::Yüzde(15.0),
            sağ: Uzunluk::Yüzde(10.0),
            üst: Uzunluk::Piksel(65.0),
            alt: Uzunluk::Piksel(80.0),
            sol_açık: false,
            sağ_açık: false,
            üst_açık: false,
            alt_açık: false,
            genişlik: None,
            yükseklik: None,
            etiketi_kapsa: false,
        }
    }
}

impl Izgara {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn sol(mut self, sol: impl Into<Uzunluk>) -> Self {
        self.sol = sol.into();
        self.sol_açık = true;
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = sağ.into();
        self.sağ_açık = true;
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = üst.into();
        self.üst_açık = true;
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = alt.into();
        self.alt_açık = true;
        self
    }

    pub fn genişlik(mut self, genişlik: impl Into<Uzunluk>) -> Self {
        self.genişlik = Some(genişlik.into());
        self
    }

    pub fn yükseklik(mut self, yükseklik: impl Into<Uzunluk>) -> Self {
        self.yükseklik = Some(yükseklik.into());
        self
    }

    pub fn etiketi_kapsa(mut self, kapsa: bool) -> Self {
        self.etiketi_kapsa = kapsa;
        self
    }
}

/// İpucu tetikleme kipi (`tooltip.trigger`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tetikleme {
    /// Tek öğe üzerinde (`'item'`) — ECharts öntanımlısı.
    #[default]
    Öğe,
    /// Eksen boyunca (`'axis'`).
    Eksen,
    Kapalı,
}

/// Tooltip kutusunun veri öğesine göre yerleşimi (`tooltip.position`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum İpucuKonumu {
    /// Kutuyu imlecin sağ altına yerleştirir (ECharts öntanımlısı).
    #[default]
    İmleç,
    /// Kutuyu veri öğesinin üst orta noktasına yerleştirir (`'top'`).
    Üst,
}

/// Eksen imleci türü (`tooltip.axisPointer.type`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum İmleçTürü {
    #[default]
    Çizgi,
    Gölge,
    Çapraz,
    Yok,
}

/// `tooltip.formatter` işlevine iletilen seri/veri bağlamı. ECharts'ın
/// `CallbackDataParams` dizisindeki görünür alanların tipli karşılığıdır.
#[derive(Clone, PartialEq, Debug)]
pub struct İpucuParametresi {
    pub seri_sırası: usize,
    pub seri_adı: String,
    pub veri_sırası: usize,
    pub ad: String,
    pub değer: VeriDeğeri,
}

type İpucuBiçimleyiciİşlevi = dyn Fn(&[İpucuParametresi]) -> String + Send + Sync;

/// Parametre dizisini alan işlev biçimli `tooltip.formatter`.
#[derive(Clone)]
pub struct İpucuBiçimleyicisi(Arc<İpucuBiçimleyiciİşlevi>);

impl İpucuBiçimleyicisi {
    pub fn yeni(
        biçimleyici: impl Fn(&[İpucuParametresi]) -> String + Send + Sync + 'static,
    ) -> Self {
        Self(Arc::new(biçimleyici))
    }

    pub fn uygula(&self, parametreler: &[İpucuParametresi]) -> String {
        (self.0)(parametreler)
    }
}

impl fmt::Debug for İpucuBiçimleyicisi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("İpucuBiçimleyicisi(..)")
    }
}

impl PartialEq for İpucuBiçimleyicisi {
    fn eq(&self, diğer: &Self) -> bool {
        Arc::ptr_eq(&self.0, &diğer.0)
    }
}

/// İpucu bileşeni (`tooltip`).
#[derive(Clone, PartialEq, Debug)]
pub struct İpucu {
    pub göster: bool,
    /// İpucu kutusu gösterilsin mi (`showContent`); eksen imleci bundan
    /// bağımsız olarak etkin kalabilir.
    pub içerik_göster: bool,
    pub tetikleme: Tetikleme,
    pub konum: İpucuKonumu,
    pub imleç: İmleçTürü,
    /// `tooltip.axisPointer.animation`; false canlı akışlarda imlecin veri
    /// güncellemesinin gerisinde kalmasını önler.
    pub imleç_animasyonu: bool,
    /// `tooltip.axisPointer.label.backgroundColor`.
    pub imleç_etiketi_arkaplanı: Option<Renk>,
    pub arkaplan: Option<Renk>,
    pub yazı: YazıStili,
    /// Değer biçimleyici (satırlardaki sayılar için).
    pub değer_biçimleyici: Option<Biçimleyici>,
    /// Satır biçimleyici (`tooltip.formatter`): `{a}` seri adı, `{b}` öğe/
    /// kategori adı, `{c}` değer. Verilirse satırlar bu şablonla yazılır.
    pub biçimleyici: Option<Biçimleyici>,
    /// ECharts işlev biçimli formatter'ının tüm parametre dizisini alan
    /// bağlamlı karşılığı.
    pub bağlamlı_biçimleyici: Option<İpucuBiçimleyicisi>,
    /// Eksen imleci bağlantısı (`axisPointer.link: 'all'`): çoklu ızgarada
    /// imleç çizgisi aynı kategori sırasında tüm ızgaralarda çizilir.
    pub bağlantılı: bool,
}

impl Default for İpucu {
    fn default() -> Self {
        İpucu {
            göster: true,
            içerik_göster: true,
            tetikleme: Tetikleme::Öğe,
            konum: İpucuKonumu::İmleç,
            imleç: İmleçTürü::Çizgi,
            imleç_animasyonu: true,
            imleç_etiketi_arkaplanı: None,
            arkaplan: None,
            yazı: YazıStili::default(),
            değer_biçimleyici: None,
            biçimleyici: None,
            bağlamlı_biçimleyici: None,
            bağlantılı: false,
        }
    }
}

impl İpucu {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn içerik_göster(mut self, göster: bool) -> Self {
        self.içerik_göster = göster;
        self
    }

    pub fn tetikleme(mut self, tetikleme: Tetikleme) -> Self {
        self.tetikleme = tetikleme;
        self
    }

    pub fn konum(mut self, konum: İpucuKonumu) -> Self {
        self.konum = konum;
        self
    }

    pub fn imleç(mut self, imleç: İmleçTürü) -> Self {
        self.imleç = imleç;
        self
    }

    pub fn imleç_animasyonu(mut self, açık: bool) -> Self {
        self.imleç_animasyonu = açık;
        self
    }

    pub fn imleç_etiketi_arkaplanı(mut self, renk: impl Into<Renk>) -> Self {
        self.imleç_etiketi_arkaplanı = Some(renk.into());
        self
    }

    pub fn arkaplan(mut self, renk: impl Into<Renk>) -> Self {
        self.arkaplan = Some(renk.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn değer_biçimleyici(mut self, b: impl Into<Biçimleyici>) -> Self {
        self.değer_biçimleyici = Some(b.into());
        self
    }

    /// Satır biçimleyici (`formatter`): `{a}` seri adı, `{b}` öğe/kategori
    /// adı, `{c}` değer.
    pub fn biçimleyici(mut self, b: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(b.into());
        self.bağlamlı_biçimleyici = None;
        self
    }

    /// İşlev biçimli `tooltip.formatter`; eksen tetiklemesinde aynı eksen
    /// değerine karşılık gelen bütün görünür seriler parametre dizisindedir.
    pub fn bağlamlı_biçimleyici(
        mut self,
        biçimleyici: impl Fn(&[İpucuParametresi]) -> String + Send + Sync + 'static,
    ) -> Self {
        self.bağlamlı_biçimleyici = Some(İpucuBiçimleyicisi::yeni(biçimleyici));
        self.biçimleyici = None;
        self
    }

    /// Eksen imlecini tüm ızgaralara bağlar (`axisPointer.link: 'all'`).
    pub fn bağlantılı(mut self, açık: bool) -> Self {
        self.bağlantılı = açık;
        self
    }
}
