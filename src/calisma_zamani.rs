//! ECharts örneği yaşam döngüsü ve `setOption` sözleşmesi.
//!
//! Bu katman çizim yüzeyinden bağımsızdır. Aynı seçenek durumu gpui, PNG,
//! SVG ve başsız doğrulama koşucuları tarafından paylaşılır. Birleştirme
//! sırası ECharts 6.1'deki `GlobalModel._mergeOption` ve
//! `mappingToExists` davranışını izler: açık `id`, ardından `name`, ardından
//! dizi sırası. Değişiklik önce bir kopyaya uygulanır ve doğrulanır; hata
//! durumunda etkin model değişmez.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::animasyon::Yumuşatma;
use crate::hata::BilesenHatasi;
use crate::koordinat::Kartezyen2B;
use crate::model::bilesen::{
    AraçKutusu, Başlık, Fırça, FırçaSeçimAlanı, Gösterge, Izgara, İpucu
};
use crate::model::deger::VeriÖğesi;
use crate::model::eksen::{Eksen, EksenKırılması};
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::grafik_bileseni::GrafikBileşeni;
use crate::model::hatlar::HatVerisi;
use crate::model::kutupsal::KutupsalKoordinat;
use crate::model::matris::MatrisKoordinatı;
use crate::model::radar::RadarKoordinatı;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::Seri;
use crate::model::takvim::TakvimKoordinatı;
use crate::model::tek_eksen::TekEksen;
use crate::model::veri_kumesi::{VeriKümesi, VeriKümesiTanımı};
use crate::model::yakinlastirma::{VeriYakınlaştırma, YakınlaştırmaDeğeri};
use crate::model::zaman_seridi::ZamanŞeridi;
use crate::renk::{Dolgu, Renk};
use crate::yerel::{TÜRKÇE, Yerel};

/// `replaceMerge` içinde kullanılabilen kök option yolları.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeçenekAlanı {
    Başlık,
    Gösterge,
    Izgara,
    Izgaralar,
    XEkseni,
    YEkseni,
    XEksenleri,
    YEksenleri,
    Seriler,
    İpucu,
    GörselEşleme,
    Radar,
    Kutupsal,
    Matris,
    Takvimler,
    TekEksenler,
    VeriKümesi,
    VeriYakınlaştırmaları,
    AraçKutusu,
    Fırça,
    Grafik,
    ZamanŞeridi,
    Palet,
    Arkaplan,
    Koyu,
    Yerel,
    Animasyon,
    AnimasyonSüresi,
    AnimasyonSüresiGüncelleme,
    AnimasyonEğrisi,
}

const TÜM_ALANLAR: [SeçenekAlanı; 30] = [
    SeçenekAlanı::Başlık,
    SeçenekAlanı::Gösterge,
    SeçenekAlanı::Izgara,
    SeçenekAlanı::Izgaralar,
    SeçenekAlanı::XEkseni,
    SeçenekAlanı::YEkseni,
    SeçenekAlanı::XEksenleri,
    SeçenekAlanı::YEksenleri,
    SeçenekAlanı::Seriler,
    SeçenekAlanı::İpucu,
    SeçenekAlanı::GörselEşleme,
    SeçenekAlanı::Radar,
    SeçenekAlanı::Kutupsal,
    SeçenekAlanı::Matris,
    SeçenekAlanı::Takvimler,
    SeçenekAlanı::TekEksenler,
    SeçenekAlanı::VeriKümesi,
    SeçenekAlanı::VeriYakınlaştırmaları,
    SeçenekAlanı::AraçKutusu,
    SeçenekAlanı::Fırça,
    SeçenekAlanı::Grafik,
    SeçenekAlanı::ZamanŞeridi,
    SeçenekAlanı::Palet,
    SeçenekAlanı::Arkaplan,
    SeçenekAlanı::Koyu,
    SeçenekAlanı::Yerel,
    SeçenekAlanı::Animasyon,
    SeçenekAlanı::AnimasyonSüresi,
    SeçenekAlanı::AnimasyonSüresiGüncelleme,
    SeçenekAlanı::AnimasyonEğrisi,
];

/// Rust modelinde bir option nesnesinde gerçekten bulunan alanları koruyan
/// yama. `GrafikSeçenekleri` somut öntanımlılar taşır; yalnız onu kullanmak
/// JavaScript'teki "alan verilmedi" ile "öntanımlı değer açıkça verildi"
/// ayrımını kaybettirirdi. Bu tip o ayrımı açık tutar.
#[derive(Clone, Debug)]
pub struct SeçenekYaması {
    değer: GrafikSeçenekleri,
    sağlanan: BTreeSet<SeçenekAlanı>,
    x_ekseni_veri_yamaları: Vec<EksenVeriYaması>,
    y_ekseni_veri_yamaları: Vec<EksenVeriYaması>,
    eksen_kırılma_yamaları: Vec<EksenKırılmaYaması>,
    seri_veri_yamaları: Vec<SeriVeriYaması>,
}

#[derive(Clone, Debug)]
struct EksenVeriYaması {
    sıra: usize,
    veri: Vec<String>,
}

#[derive(Clone, Debug)]
struct EksenKırılmaYaması {
    boyut: EksenBoyutu,
    sıra: usize,
    kırılmalar: Vec<EksenKırılması>,
}

#[derive(Clone, Debug)]
struct SeriVeriYaması {
    seçici: SeriSeçici,
    veri: Vec<VeriÖğesi>,
}

impl Default for SeçenekYaması {
    fn default() -> Self {
        Self::yeni()
    }
}

impl SeçenekYaması {
    /// Hiçbir option yolunu değiştirmeyen boş yama.
    pub fn yeni() -> Self {
        Self {
            değer: GrafikSeçenekleri::default(),
            sağlanan: BTreeSet::new(),
            x_ekseni_veri_yamaları: Vec::new(),
            y_ekseni_veri_yamaları: Vec::new(),
            eksen_kırılma_yamaları: Vec::new(),
            seri_veri_yamaları: Vec::new(),
        }
    }

    /// Bütün kök yolları açıkça sağlanmış tam option nesnesi.
    pub fn tam(değer: GrafikSeçenekleri) -> Self {
        Self {
            değer,
            sağlanan: TÜM_ALANLAR.into_iter().collect(),
            x_ekseni_veri_yamaları: Vec::new(),
            y_ekseni_veri_yamaları: Vec::new(),
            eksen_kırılma_yamaları: Vec::new(),
            seri_veri_yamaları: Vec::new(),
        }
    }

    pub fn sağlandı_mı(&self, alan: SeçenekAlanı) -> bool {
        self.sağlanan.contains(&alan)
    }

    pub fn değer(&self) -> &GrafikSeçenekleri {
        &self.değer
    }

    pub fn başlık(mut self, başlık: Başlık) -> Self {
        self.değer.başlık = Some(başlık);
        self.değer.başlıklar.clear();
        self.sağlanan.insert(SeçenekAlanı::Başlık);
        self
    }

    pub fn başlıklar(mut self, başlıklar: impl IntoIterator<Item = Başlık>) -> Self {
        self.değer.başlık = None;
        self.değer.başlıklar = başlıklar.into_iter().collect();
        self.sağlanan.insert(SeçenekAlanı::Başlık);
        self
    }

    pub fn başlığı_kaldır(mut self) -> Self {
        self.değer.başlık = None;
        self.değer.başlıklar.clear();
        self.sağlanan.insert(SeçenekAlanı::Başlık);
        self
    }

    pub fn gösterge(mut self, gösterge: Gösterge) -> Self {
        self.değer.gösterge = Some(gösterge);
        self.sağlanan.insert(SeçenekAlanı::Gösterge);
        self
    }

    pub fn göstergeyi_kaldır(mut self) -> Self {
        self.değer.gösterge = None;
        self.sağlanan.insert(SeçenekAlanı::Gösterge);
        self
    }

    pub fn ızgara(mut self, ızgara: Izgara) -> Self {
        self.değer.ızgara = ızgara;
        self.sağlanan.insert(SeçenekAlanı::Izgara);
        self
    }

    pub fn ızgaralar(mut self, ızgaralar: impl IntoIterator<Item = Izgara>) -> Self {
        self.değer.ızgaralar = ızgaralar.into_iter().collect();
        self.sağlanan.insert(SeçenekAlanı::Izgaralar);
        self
    }

    pub fn x_ekseni(mut self, eksen: Eksen) -> Self {
        self.değer.x_ekseni = Some(eksen);
        self.sağlanan.insert(SeçenekAlanı::XEkseni);
        self
    }

    pub fn x_eksenini_kaldır(mut self) -> Self {
        self.değer.x_ekseni = None;
        self.sağlanan.insert(SeçenekAlanı::XEkseni);
        self
    }

    pub fn y_ekseni(mut self, eksen: Eksen) -> Self {
        self.değer.y_ekseni = Some(eksen);
        self.sağlanan.insert(SeçenekAlanı::YEkseni);
        self
    }

    pub fn y_eksenini_kaldır(mut self) -> Self {
        self.değer.y_ekseni = None;
        self.sağlanan.insert(SeçenekAlanı::YEkseni);
        self
    }

    pub fn x_eksenleri(mut self, eksenler: impl IntoIterator<Item = Eksen>) -> Self {
        self.değer.x_eksenleri = eksenler.into_iter().collect();
        self.sağlanan.insert(SeçenekAlanı::XEksenleri);
        self
    }

    pub fn y_eksenleri(mut self, eksenler: impl IntoIterator<Item = Eksen>) -> Self {
        self.değer.y_eksenleri = eksenler.into_iter().collect();
        self.sağlanan.insert(SeçenekAlanı::YEksenleri);
        self
    }

    /// ECharts'ın `setOption({ xAxis: [{ data }] })` biçimindeki iç içe
    /// yamasının tipli karşılığı. Yalnız seçilen kategori verisi değişir;
    /// eksenin türü, adı, sınırları ve bütün görsel seçenekleri korunur.
    pub fn x_ekseni_verisi<S: Into<String>>(
        mut self,
        sıra: usize,
        veri: impl IntoIterator<Item = S>,
    ) -> Self {
        self.x_ekseni_veri_yamaları.push(EksenVeriYaması {
            sıra,
            veri: veri.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// ECharts'ın `setOption({ yAxis: [{ data }] })` biçimindeki iç içe
    /// yamasının tipli karşılığı. Yalnız seçilen kategori verisi değişir.
    pub fn y_ekseni_verisi<S: Into<String>>(
        mut self,
        sıra: usize,
        veri: impl IntoIterator<Item = S>,
    ) -> Self {
        self.y_ekseni_veri_yamaları.push(EksenVeriYaması {
            sıra,
            veri: veri.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// `setOption({xAxis: [{breaks}]})`: yalnız seçilen eksenin kırık
    /// listesini değiştirir, diğer eksen seçeneklerini korur.
    pub fn x_ekseni_kırılmaları(
        mut self,
        sıra: usize,
        kırılmalar: impl IntoIterator<Item = EksenKırılması>,
    ) -> Self {
        self.eksen_kırılma_yamaları.push(EksenKırılmaYaması {
            boyut: EksenBoyutu::X,
            sıra,
            kırılmalar: kırılmalar.into_iter().collect(),
        });
        self
    }

    /// `setOption({yAxis: [{breaks}]})` iç içe yaması.
    pub fn y_ekseni_kırılmaları(
        mut self,
        sıra: usize,
        kırılmalar: impl IntoIterator<Item = EksenKırılması>,
    ) -> Self {
        self.eksen_kırılma_yamaları.push(EksenKırılmaYaması {
            boyut: EksenBoyutu::Y,
            sıra,
            kırılmalar: kırılmalar.into_iter().collect(),
        });
        self
    }

    /// `setOption({singleAxis: [{breaks}]})` iç içe yaması.
    pub fn tek_eksen_kırılmaları(
        mut self,
        sıra: usize,
        kırılmalar: impl IntoIterator<Item = EksenKırılması>,
    ) -> Self {
        self.eksen_kırılma_yamaları.push(EksenKırılmaYaması {
            boyut: EksenBoyutu::Tek,
            sıra,
            kırılmalar: kırılmalar.into_iter().collect(),
        });
        self
    }

    /// Yamadaki seri dizisini sıfırlar ve kimliksiz bir seri ekler.
    pub fn seri(mut self, seri: impl Into<Seri>) -> Self {
        if !self.sağlanan.contains(&SeçenekAlanı::Seriler) {
            self.değer.seriler.clear();
            self.değer.seri_kimlikleri.clear();
        }
        self.değer.seriler.push(seri.into());
        self.değer.seri_kimlikleri.push(None);
        self.sağlanan.insert(SeçenekAlanı::Seriler);
        self
    }

    /// Yamadaki seri dizisini sıfırlar ve açık `series.id` ile seri ekler.
    pub fn kimlikli_seri(mut self, kimlik: impl Into<String>, seri: impl Into<Seri>) -> Self {
        if !self.sağlanan.contains(&SeçenekAlanı::Seriler) {
            self.değer.seriler.clear();
            self.değer.seri_kimlikleri.clear();
        }
        self.değer.seriler.push(seri.into());
        self.değer.seri_kimlikleri.push(Some(kimlik.into()));
        self.sağlanan.insert(SeçenekAlanı::Seriler);
        self
    }

    /// Açıkça boş `series: []` yamayı ifade eder.
    pub fn serileri_boşalt(mut self) -> Self {
        self.değer.seriler.clear();
        self.değer.seri_kimlikleri.clear();
        self.sağlanan.insert(SeçenekAlanı::Seriler);
        self
    }

    /// ECharts'ın `setOption({ series: [{ data }] })` biçimindeki iç içe
    /// yamasının tipli karşılığı. Hedef seri sıra, `id` veya `name` ile
    /// çözülür; yalnız veri deposu değiştirilir, serinin türü ve diğer bütün
    /// seçenekleri korunur.
    pub fn seri_verisi<T: Into<VeriÖğesi>>(
        mut self,
        seçici: SeriSeçici,
        veri: impl IntoIterator<Item = T>,
    ) -> Self {
        self.seri_veri_yamaları.push(SeriVeriYaması {
            seçici,
            veri: veri.into_iter().map(Into::into).collect(),
        });
        self
    }

    pub fn ipucu(mut self, ipucu: İpucu) -> Self {
        self.değer.ipucu = Some(ipucu);
        self.sağlanan.insert(SeçenekAlanı::İpucu);
        self
    }

    pub fn ipucunu_kaldır(mut self) -> Self {
        self.değer.ipucu = None;
        self.sağlanan.insert(SeçenekAlanı::İpucu);
        self
    }

    pub fn görsel_eşleme(mut self, eşleme: GörselEşleme) -> Self {
        self.değer.görsel_eşleme = Some(eşleme);
        self.değer.görsel_eşlemeler.clear();
        self.sağlanan.insert(SeçenekAlanı::GörselEşleme);
        self
    }

    pub fn görsel_eşlemeler(
        mut self, eşlemeler: impl IntoIterator<Item = GörselEşleme>
    ) -> Self {
        self.değer.görsel_eşleme = None;
        self.değer.görsel_eşlemeler = eşlemeler.into_iter().collect();
        self.sağlanan.insert(SeçenekAlanı::GörselEşleme);
        self
    }

    pub fn görsel_eşlemeyi_kaldır(mut self) -> Self {
        self.değer.görsel_eşleme = None;
        self.değer.görsel_eşlemeler.clear();
        self.sağlanan.insert(SeçenekAlanı::GörselEşleme);
        self
    }

    pub fn radar(mut self, radar: RadarKoordinatı) -> Self {
        self.değer.radar = Some(radar);
        self.sağlanan.insert(SeçenekAlanı::Radar);
        self
    }

    pub fn radarı_kaldır(mut self) -> Self {
        self.değer.radar = None;
        self.sağlanan.insert(SeçenekAlanı::Radar);
        self
    }

    pub fn kutupsal(mut self, kutupsal: KutupsalKoordinat) -> Self {
        self.değer.kutupsal = Some(kutupsal);
        self.sağlanan.insert(SeçenekAlanı::Kutupsal);
        self
    }

    pub fn kutupsalı_kaldır(mut self) -> Self {
        self.değer.kutupsal = None;
        self.sağlanan.insert(SeçenekAlanı::Kutupsal);
        self
    }

    pub fn matris(mut self, matris: MatrisKoordinatı) -> Self {
        self.değer.matris = Some(matris);
        self.sağlanan.insert(SeçenekAlanı::Matris);
        self
    }

    pub fn matrisi_kaldır(mut self) -> Self {
        self.değer.matris = None;
        self.sağlanan.insert(SeçenekAlanı::Matris);
        self
    }

    pub fn takvimler(mut self, takvimler: impl IntoIterator<Item = TakvimKoordinatı>) -> Self {
        self.değer.takvimler = takvimler.into_iter().collect();
        self.sağlanan.insert(SeçenekAlanı::Takvimler);
        self
    }

    pub fn tek_eksenler(mut self, eksenler: impl IntoIterator<Item = TekEksen>) -> Self {
        self.değer.tek_eksenler = eksenler.into_iter().collect();
        self.sağlanan.insert(SeçenekAlanı::TekEksenler);
        self
    }

    pub fn veri_kümesi(mut self, küme: VeriKümesi) -> Self {
        self.değer.veri_kümesi = Some(küme);
        self.değer.veri_kümeleri.clear();
        self.sağlanan.insert(SeçenekAlanı::VeriKümesi);
        self
    }

    pub fn veri_kümeleri(
        mut self, tanımlar: impl IntoIterator<Item = VeriKümesiTanımı>
    ) -> Self {
        self.değer.veri_kümesi = None;
        self.değer.veri_kümeleri = tanımlar.into_iter().collect();
        self.sağlanan.insert(SeçenekAlanı::VeriKümesi);
        self
    }

    pub fn veri_kümesini_kaldır(mut self) -> Self {
        self.değer.veri_kümesi = None;
        self.değer.veri_kümeleri.clear();
        self.sağlanan.insert(SeçenekAlanı::VeriKümesi);
        self
    }

    pub fn veri_yakınlaştırmaları(
        mut self,
        yakınlaştırmalar: impl IntoIterator<Item = VeriYakınlaştırma>,
    ) -> Self {
        self.değer.veri_yakınlaştırmaları = yakınlaştırmalar.into_iter().collect();
        self.sağlanan.insert(SeçenekAlanı::VeriYakınlaştırmaları);
        self
    }

    pub fn araç_kutusu(mut self, araç_kutusu: AraçKutusu) -> Self {
        self.değer.araç_kutusu = Some(araç_kutusu);
        self.sağlanan.insert(SeçenekAlanı::AraçKutusu);
        self
    }

    pub fn araç_kutusunu_kaldır(mut self) -> Self {
        self.değer.araç_kutusu = None;
        self.sağlanan.insert(SeçenekAlanı::AraçKutusu);
        self
    }

    pub fn fırça(mut self, fırça: Fırça) -> Self {
        self.değer.fırça = Some(fırça);
        self.sağlanan.insert(SeçenekAlanı::Fırça);
        self
    }

    pub fn fırçayı_kaldır(mut self) -> Self {
        self.değer.fırça = None;
        self.sağlanan.insert(SeçenekAlanı::Fırça);
        self
    }

    pub fn grafik(mut self, grafik: GrafikBileşeni) -> Self {
        self.değer.grafik = Some(grafik);
        self.sağlanan.insert(SeçenekAlanı::Grafik);
        self
    }

    pub fn grafiği_kaldır(mut self) -> Self {
        self.değer.grafik = None;
        self.sağlanan.insert(SeçenekAlanı::Grafik);
        self
    }

    pub fn zaman_şeridi(mut self, zaman_şeridi: ZamanŞeridi) -> Self {
        self.değer.zaman_şeridi = Some(zaman_şeridi);
        self.sağlanan.insert(SeçenekAlanı::ZamanŞeridi);
        self
    }

    pub fn zaman_şeridini_kaldır(mut self) -> Self {
        self.değer.zaman_şeridi = None;
        self.sağlanan.insert(SeçenekAlanı::ZamanŞeridi);
        self
    }

    pub fn palet<R: Into<Renk>>(mut self, renkler: impl IntoIterator<Item = R>) -> Self {
        self.değer.palet = renkler.into_iter().map(Into::into).collect();
        self.sağlanan.insert(SeçenekAlanı::Palet);
        self
    }

    pub fn arkaplan(mut self, dolgu: impl Into<Dolgu>) -> Self {
        self.değer.arkaplan = Some(dolgu.into());
        self.sağlanan.insert(SeçenekAlanı::Arkaplan);
        self
    }

    pub fn arkaplanı_kaldır(mut self) -> Self {
        self.değer.arkaplan = None;
        self.sağlanan.insert(SeçenekAlanı::Arkaplan);
        self
    }

    pub fn koyu(mut self, açık: bool) -> Self {
        self.değer.koyu = açık;
        self.sağlanan.insert(SeçenekAlanı::Koyu);
        self
    }

    pub fn yerel(mut self, yerel: &'static Yerel) -> Self {
        self.değer.yerel = yerel;
        self.sağlanan.insert(SeçenekAlanı::Yerel);
        self
    }

    pub fn animasyon(mut self, açık: bool) -> Self {
        self.değer.animasyon = açık;
        self.sağlanan.insert(SeçenekAlanı::Animasyon);
        self
    }

    pub fn animasyon_süresi(mut self, ms: f32) -> Self {
        self.değer.animasyon_süresi = ms;
        self.sağlanan.insert(SeçenekAlanı::AnimasyonSüresi);
        self
    }

    pub fn animasyon_süresi_güncelleme(mut self, ms: f32) -> Self {
        self.değer.animasyon_süresi_güncelleme = ms;
        self.sağlanan
            .insert(SeçenekAlanı::AnimasyonSüresiGüncelleme);
        self
    }

    pub fn animasyon_eğrisi(mut self, eğri: Yumuşatma) -> Self {
        self.değer.animasyon_eğrisi = eğri;
        self.sağlanan.insert(SeçenekAlanı::AnimasyonEğrisi);
        self
    }
}

impl From<GrafikSeçenekleri> for SeçenekYaması {
    fn from(değer: GrafikSeçenekleri) -> Self {
        Self::tam(değer)
    }
}

/// `setOption` davranış bayrakları.
#[derive(Clone, Debug, Default)]
pub struct SeçenekAyarlamaKipi {
    pub birleştirme_yok: bool,
    pub değiştirerek_birleştir: BTreeSet<SeçenekAlanı>,
    pub tembel_güncelle: bool,
    pub sessiz: bool,
}

impl SeçenekAyarlamaKipi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn birleştirme_yok(mut self, açık: bool) -> Self {
        self.birleştirme_yok = açık;
        self
    }

    pub fn değiştirerek(mut self, alan: SeçenekAlanı) -> Self {
        self.değiştirerek_birleştir.insert(alan);
        self
    }

    pub fn tembel(mut self, açık: bool) -> Self {
        self.tembel_güncelle = açık;
        self
    }

    pub fn sessiz(mut self, açık: bool) -> Self {
        self.sessiz = açık;
        self
    }
}

/// Responsive `media.query`; bütün sağlanan sınırlar birlikte sağlanmalıdır.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MedyaSorgusu {
    pub en_az_genişlik: Option<f32>,
    pub en_çok_genişlik: Option<f32>,
    pub en_az_yükseklik: Option<f32>,
    pub en_çok_yükseklik: Option<f32>,
    pub en_az_en_boy_oranı: Option<f32>,
    pub en_çok_en_boy_oranı: Option<f32>,
}

impl MedyaSorgusu {
    pub fn uyuyor_mu(&self, genişlik: f32, yükseklik: f32) -> bool {
        if !genişlik.is_finite() || !yükseklik.is_finite() || genişlik <= 0.0 || yükseklik <= 0.0
        {
            return false;
        }
        let oran = genişlik / yükseklik;
        alt_sınır(self.en_az_genişlik, genişlik)
            && üst_sınır(self.en_çok_genişlik, genişlik)
            && alt_sınır(self.en_az_yükseklik, yükseklik)
            && üst_sınır(self.en_çok_yükseklik, yükseklik)
            && alt_sınır(self.en_az_en_boy_oranı, oran)
            && üst_sınır(self.en_çok_en_boy_oranı, oran)
    }
}

fn alt_sınır(sınır: Option<f32>, değer: f32) -> bool {
    sınır.map(|sınır| değer >= sınır).unwrap_or(true)
}

fn üst_sınır(sınır: Option<f32>, değer: f32) -> bool {
    sınır.map(|sınır| değer <= sınır).unwrap_or(true)
}

#[derive(Clone, Debug)]
pub struct MedyaKuralı {
    /// `None`, sorgulu hiçbir kural eşleşmezse uygulanan öntanımlı media'dır.
    pub sorgu: Option<MedyaSorgusu>,
    pub seçenek: SeçenekYaması,
}

/// `baseOption + timeline.options + media` bileşik option biçimi.
#[derive(Clone, Debug, Default)]
pub struct BileşikSeçenekler {
    pub temel: GrafikSeçenekleri,
    pub zaman_kareleri: Vec<SeçenekYaması>,
    pub medya: Vec<MedyaKuralı>,
}

impl BileşikSeçenekler {
    pub fn yeni(temel: GrafikSeçenekleri) -> Self {
        Self {
            temel,
            ..Self::default()
        }
    }

    pub fn zaman_karesi(mut self, seçenek: impl Into<SeçenekYaması>) -> Self {
        self.zaman_kareleri.push(seçenek.into());
        self
    }

    pub fn medya(mut self, sorgu: MedyaSorgusu, seçenek: impl Into<SeçenekYaması>) -> Self {
        self.medya.push(MedyaKuralı {
            sorgu: Some(sorgu),
            seçenek: seçenek.into(),
        });
        self
    }

    pub fn öntanımlı_medya(mut self, seçenek: impl Into<SeçenekYaması>) -> Self {
        self.medya.push(MedyaKuralı {
            sorgu: None,
            seçenek: seçenek.into(),
        });
        self
    }

    /// Seçilen timeline karesini ve o an eşleşen bütün media sorgularını
    /// normal merge ile temel option üzerine uygular.
    pub fn çöz(
        &self,
        genişlik: f32,
        yükseklik: f32,
        zaman_karesi: Option<usize>,
    ) -> Result<GrafikSeçenekleri, BilesenHatasi> {
        boyutu_doğrula(genişlik, yükseklik, 1.0)?;
        let kip = SeçenekAyarlamaKipi::default();
        let mut sonuç = self.temel.clone();
        if let Some(sıra) = zaman_karesi {
            let kare = self
                .zaman_kareleri
                .get(sıra)
                .ok_or(BilesenHatasi::EksikVeri {
                    bileşen: "timeline.options",
                    sıra,
                })?;
            yamayı_uygula(&mut sonuç, kare, &kip)?;
            if let Some(zaman_şeridi) = sonuç.zaman_şeridi.as_mut() {
                let toplam = zaman_şeridi.veri.len();
                zaman_şeridi.geçerli_sıra = if toplam == 0 {
                    0
                } else if zaman_şeridi.döngü {
                    sıra % toplam
                } else {
                    sıra.min(toplam - 1)
                };
            }
        }

        let eşleşenler: Vec<&MedyaKuralı> = self
            .medya
            .iter()
            .filter(|kural| {
                kural
                    .sorgu
                    .as_ref()
                    .map(|sorgu| sorgu.uyuyor_mu(genişlik, yükseklik))
                    .unwrap_or(false)
            })
            .collect();
        if eşleşenler.is_empty() {
            for kural in self.medya.iter().filter(|kural| kural.sorgu.is_none()) {
                yamayı_uygula(&mut sonuç, &kural.seçenek, &kip)?;
            }
        } else {
            for kural in eşleşenler {
                yamayı_uygula(&mut sonuç, &kural.seçenek, &kip)?;
            }
        }
        seçenekleri_doğrula(&sonuç)?;
        Ok(sonuç)
    }
}

/// Grafik örneğinin renderer ve platformdan bağımsız başlatma seçenekleri.
#[derive(Clone, Debug, PartialEq)]
pub struct ÖrnekBaşlatmaSeçenekleri {
    pub genişlik: f32,
    pub yükseklik: f32,
    pub aygıt_piksel_oranı: f32,
    pub çizici: ÇiziciTürü,
    pub yerel: &'static Yerel,
    pub tema: Option<String>,
    pub kaba_imleç: bool,
    pub kirli_dikdörtgen: bool,
    pub başsız: bool,
}

impl Default for ÖrnekBaşlatmaSeçenekleri {
    fn default() -> Self {
        Self {
            genişlik: 700.0,
            yükseklik: 525.0,
            aygıt_piksel_oranı: 1.0,
            çizici: ÇiziciTürü::Gpui,
            yerel: &TÜRKÇE,
            tema: None,
            kaba_imleç: false,
            kirli_dikdörtgen: false,
            başsız: false,
        }
    }
}

/// ECharts `renderer`/SSR seçiminin açık Rust karşılığı.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ÇiziciTürü {
    Gpui,
    Piksel,
    Svg,
    Kayıt,
}

/// Seri hedefleyen API'lerde `seriesIndex`/`seriesId`/`seriesName` seçicisi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeriSeçici {
    Sıra(usize),
    Kimlik(String),
    Ad(String),
}

/// Yerleşik legend action'larının model işlemleri.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GöstergeSeçimEylemi {
    Seç,
    SeçimiKaldır,
    Değiştir,
    TümünüSeç,
    TersiniSeç,
}

/// Axis-break action seçicisinin hedef koordinat boyutu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EksenBoyutu {
    X,
    Y,
    Tek,
}

/// `expandAxisBreak` / `collapseAxisBreak` / `toggleAxisBreak` işlemi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EksenKırılmaEylemi {
    Genişlet,
    Daralt,
    Değiştir,
}

/// Bir axis-break action'ının olay yüküne taşınan eski/yeni durum kaydı.
#[derive(Clone, Debug, PartialEq)]
pub struct EksenKırılmaDeğişikliği {
    pub boyut: EksenBoyutu,
    pub eksen_sırası: usize,
    pub başlangıç: f64,
    pub bitiş: f64,
    pub eski_genişletilmiş: bool,
    pub genişletilmiş: bool,
}

impl SeriSeçici {
    pub fn kimlik(kimlik: impl Into<String>) -> Self {
        Self::Kimlik(kimlik.into())
    }

    pub fn ad(ad: impl Into<String>) -> Self {
        Self::Ad(ad.into())
    }
}

/// Başsız çalışma zamanının gözlenebilir yaşam döngüsü olayları.
#[derive(Clone, Debug, PartialEq)]
pub enum ÇalışmaOlayı {
    SeçenekDeğişti,
    YenidenÇizildi,
    BoyutDeğişti {
        genişlik: f32,
        yükseklik: f32,
        aygıt_piksel_oranı: f32,
    },
    Temizlendi,
    Kapatıldı,
    VeriEklendi {
        seri_sırası: usize,
        adet: usize,
    },
    YakınlaştırmaDeğişti {
        değişiklikler: Vec<(usize, f32, f32)>,
    },
    GörselAralıkDeğişti {
        sıra: usize,
        seçili: [f64; 2],
    },
    GörselParçalarDeğişti {
        sıra: usize,
        seçili: BTreeMap<usize, bool>,
    },
    GöstergeDeğişti {
        seçili: BTreeMap<String, bool>,
    },
    EksenKırılmasıDeğişti {
        değişiklikler: Vec<EksenKırılmaDeğişikliği>,
    },
    GeriYüklendi,
    YüklemeDeğişti {
        görünür: bool,
        metin: Option<String>,
    },
}

/// Tek bir grafik örneğinin option, boyut ve yaşam döngüsü durumu.
#[derive(Clone, Debug)]
pub struct GrafikÇalışmaZamanı {
    seçenekler: GrafikSeçenekleri,
    /// Toolbox `restore` için ilk `setOption` tabanının değişmez kopyası.
    /// ECharts `OptionManager.mountOption(true)` de yeniden yaratırken ilk
    /// base option yedeğini kullanır.
    geri_yükleme_seçenekleri: GrafikSeçenekleri,
    başlatma: ÖrnekBaşlatmaSeçenekleri,
    kapatıldı: bool,
    yeniden_çizim_bekliyor: bool,
    bekleyen_olay_sessiz: bool,
    yükleme: Option<String>,
    olaylar: Vec<ÇalışmaOlayı>,
}

impl GrafikÇalışmaZamanı {
    pub fn yeni(
        mut başlatma: ÖrnekBaşlatmaSeçenekleri,
        mut seçenekler: GrafikSeçenekleri,
    ) -> Result<Self, BilesenHatasi> {
        boyutu_doğrula(
            başlatma.genişlik,
            başlatma.yükseklik,
            başlatma.aygıt_piksel_oranı,
        )?;
        seçenekler.yerel = başlatma.yerel;
        seçenekleri_doğrula(&seçenekler)?;
        // -0.0, olay/golden çıktılarında kararsızlık üretmesin.
        başlatma.genişlik += 0.0;
        başlatma.yükseklik += 0.0;
        Ok(Self {
            geri_yükleme_seçenekleri: seçenekler.clone(),
            seçenekler,
            başlatma,
            kapatıldı: false,
            yeniden_çizim_bekliyor: false,
            bekleyen_olay_sessiz: true,
            yükleme: None,
            olaylar: Vec::new(),
        })
    }

    pub fn bileşik_seçeneklerle(
        başlatma: ÖrnekBaşlatmaSeçenekleri,
        seçenekler: &BileşikSeçenekler,
        zaman_karesi: Option<usize>,
    ) -> Result<Self, BilesenHatasi> {
        let çözülmüş = seçenekler.çöz(başlatma.genişlik, başlatma.yükseklik, zaman_karesi)?;
        Self::yeni(başlatma, çözülmüş)
    }

    pub fn başlatma(&self) -> &ÖrnekBaşlatmaSeçenekleri {
        &self.başlatma
    }

    /// ECharts `getOption` karşılığı; çağıranın iç modeli değiştirememesi için
    /// ayrık bir kopya döndürür.
    pub fn seçenekleri_al(&self) -> Result<GrafikSeçenekleri, BilesenHatasi> {
        self.açık_mı("getOption")?;
        Ok(self.seçenekler.clone())
    }

    /// ECharts `setOption` karşılığı. Geçersiz yamada işlem bütünüyle geri
    /// alınır. `lazyUpdate`, modeli hemen günceller ama render olayını
    /// [`Self::bekleyeni_çiz`] çağrısına kadar erteler.
    pub fn seçenekleri_ayarla(
        &mut self,
        yama: impl Into<SeçenekYaması>,
        kip: SeçenekAyarlamaKipi,
    ) -> Result<(), BilesenHatasi> {
        self.açık_mı("setOption")?;
        let yama = yama.into();
        let mut aday = if kip.birleştirme_yok {
            GrafikSeçenekleri::default()
        } else {
            self.seçenekler.clone()
        };

        yamayı_uygula(&mut aday, &yama, &kip)?;
        seçenekleri_doğrula(&aday)?;
        self.seçenekler = aday;

        if kip.tembel_güncelle {
            self.yeniden_çizim_bekliyor = true;
            self.bekleyen_olay_sessiz &= kip.sessiz;
        } else if !kip.sessiz {
            self.olaylar.push(ÇalışmaOlayı::SeçenekDeğişti);
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(())
    }

    /// Birikmiş `lazyUpdate` render'ını belirlenimci olarak tamamlar.
    pub fn bekleyeni_çiz(&mut self) -> Result<bool, BilesenHatasi> {
        self.açık_mı("flush")?;
        if !self.yeniden_çizim_bekliyor {
            return Ok(false);
        }
        self.yeniden_çizim_bekliyor = false;
        if !self.bekleyen_olay_sessiz {
            self.olaylar.push(ÇalışmaOlayı::SeçenekDeğişti);
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        self.bekleyen_olay_sessiz = true;
        Ok(true)
    }

    pub fn yeniden_çizim_bekliyor_mu(&self) -> bool {
        self.yeniden_çizim_bekliyor
    }

    /// ECharts 6 `expandAxisBreak`, `collapseAxisBreak` ve
    /// `toggleAxisBreak` action'larının atomik model güncellemesi.
    /// `sıralar=None` seçilen boyuttaki bütün eksenleri hedefler; kırılmalar
    /// kullanıcı option'ındaki özgün `start`/`end` çiftiyle tanınır.
    pub fn eksen_kırılmalarını_ayarla(
        &mut self,
        boyut: EksenBoyutu,
        sıralar: Option<&[usize]>,
        kırılmalar: &[(f64, f64)],
        eylem: EksenKırılmaEylemi,
        sessiz: bool,
    ) -> Result<Vec<EksenKırılmaDeğişikliği>, BilesenHatasi> {
        self.açık_mı("dispatchAction.axisBreak")?;
        if let Some((başlangıç, bitiş)) = kırılmalar
            .iter()
            .copied()
            .find(|(başlangıç, bitiş)| !başlangıç.is_finite() || !bitiş.is_finite())
        {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "action.breaks",
                ayrıntı: format!("kırılma uçları sonlu olmalı: start={başlangıç}, end={bitiş}"),
            });
        }

        let toplam = eksen_sayısı(&self.seçenekler, boyut);
        let hedefler = sıralar
            .map(|sıralar| sıralar.to_vec())
            .unwrap_or_else(|| (0..toplam).collect());
        if let Some(&geçersiz) = hedefler.iter().find(|sıra| **sıra >= toplam) {
            return Err(BilesenHatasi::EksikVeri {
                bileşen: match boyut {
                    EksenBoyutu::X => "xAxis",
                    EksenBoyutu::Y => "yAxis",
                    EksenBoyutu::Tek => "singleAxis",
                },
                sıra: geçersiz,
            });
        }

        let mut aday = self.seçenekler.clone();
        let mut değişiklikler = Vec::new();
        let mut görülen = BTreeSet::new();
        for eksen_sırası in hedefler {
            if !görülen.insert(eksen_sırası) {
                continue;
            }
            let Some(eksen) = ekseni_mut(&mut aday, boyut, eksen_sırası) else {
                continue;
            };
            for &(başlangıç, bitiş) in kırılmalar {
                let Some(kırılma) = eksen
                    .kırılmalar
                    .iter_mut()
                    .find(|kırılma| kırılma.başlangıç == başlangıç && kırılma.bitiş == bitiş)
                else {
                    // Resmî uygulama bulunamayan tanımlayıcıyı yalnız
                    // geliştirme uyarısıyla atlar; action başarısız olmaz.
                    continue;
                };
                let eski = kırılma.genişletilmiş;
                kırılma.genişletilmiş = match eylem {
                    EksenKırılmaEylemi::Genişlet => true,
                    EksenKırılmaEylemi::Daralt => false,
                    EksenKırılmaEylemi::Değiştir => !eski,
                };
                değişiklikler.push(EksenKırılmaDeğişikliği {
                    boyut,
                    eksen_sırası,
                    başlangıç,
                    bitiş,
                    eski_genişletilmiş: eski,
                    genişletilmiş: kırılma.genişletilmiş,
                });
            }
        }

        seçenekleri_doğrula(&aday)?;
        self.seçenekler = aday;
        if !sessiz && !değişiklikler.is_empty() {
            self.olaylar.push(ÇalışmaOlayı::EksenKırılmasıDeğişti {
                değişiklikler: değişiklikler.clone(),
            });
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(değişiklikler)
    }

    /// `dispatchAction({type: "dataZoom"})` model güncellemesi.
    ///
    /// Seçici verilmezse bütün dataZoom bileşenleri hedeflenir. Bir sıra
    /// verildiğinde ECharts `findEffectedDataZooms` gibi aynı ekseni yöneten
    /// bağlı dataZoom bileşenleri de birlikte güncellenir. Eksik `start` veya
    /// `end`, bileşenin mevcut ucunu korur; işlem bütün hedefler için atomiktir.
    pub fn veri_yakınlaştırmayı_ayarla(
        &mut self,
        sıra: Option<usize>,
        başlangıç: Option<f32>,
        bitiş: Option<f32>,
        sessiz: bool,
    ) -> Result<Vec<(usize, f32, f32)>, BilesenHatasi> {
        self.veri_yakınlaştırma_aralığını_ayarla(sıra, başlangıç, bitiş, None, None, sessiz)
    }

    /// Yüzde ve değer tabanlı `dataZoom` uçlarını tek action işlemi olarak
    /// uygular. Aynı uçta hem yüzde hem değer verilirse ECharts
    /// `DataZoomModel.setRawRange` gibi yüzde önceliklidir ve eski değer ucu
    /// temizlenir. Yalnız değer verilirse yüzde alanı geri dönüş değeri olarak
    /// korunur.
    pub(crate) fn veri_yakınlaştırma_aralığını_ayarla(
        &mut self,
        sıra: Option<usize>,
        başlangıç: Option<f32>,
        bitiş: Option<f32>,
        başlangıç_değeri: Option<YakınlaştırmaDeğeri>,
        bitiş_değeri: Option<YakınlaştırmaDeğeri>,
        sessiz: bool,
    ) -> Result<Vec<(usize, f32, f32)>, BilesenHatasi> {
        self.açık_mı("dispatchAction.dataZoom")?;
        if başlangıç.is_none()
            && bitiş.is_none()
            && başlangıç_değeri.is_none()
            && bitiş_değeri.is_none()
        {
            return Ok(Vec::new());
        }
        for (alan, değer) in [("dataZoom.start", başlangıç), ("dataZoom.end", bitiş)] {
            if let Some(değer) = değer
                && (!değer.is_finite() || !(0.0..=100.0).contains(&değer))
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan,
                    ayrıntı: format!("{değer} değeri 0..=100 aralığında sonlu olmalı"),
                });
            }
        }

        let sıralar = self.bağlı_yakınlaştırma_sıraları(sıra)?;
        let mut aday = self.seçenekler.clone();
        let mut değişiklikler = Vec::with_capacity(sıralar.len());
        for &hedef in &sıralar {
            let yakınlaştırma =
                aday.veri_yakınlaştırmaları
                    .get_mut(hedef)
                    .ok_or(BilesenHatasi::EksikVeri {
                        bileşen: "dataZoom",
                        sıra: hedef,
                    })?;
            if let Some(başlangıç) = başlangıç {
                yakınlaştırma.başlangıç = başlangıç;
                yakınlaştırma.başlangıç_değeri = None;
            } else if let Some(değer) = başlangıç_değeri.clone() {
                yakınlaştırma.başlangıç_değeri = Some(değer);
            }
            if let Some(bitiş) = bitiş {
                yakınlaştırma.bitiş = bitiş;
                yakınlaştırma.bitiş_değeri = None;
            } else if let Some(değer) = bitiş_değeri.clone() {
                yakınlaştırma.bitiş_değeri = Some(değer);
            }
            let yeni_başlangıç = yakınlaştırma.başlangıç;
            let yeni_bitiş = yakınlaştırma.bitiş;
            if yeni_başlangıç > yeni_bitiş {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "dataZoom.start/end",
                    ayrıntı: format!(
                        "başlangıç ({yeni_başlangıç}) bitişten ({yeni_bitiş}) büyük olamaz"
                    ),
                });
            }
            değişiklikler.push((hedef, yeni_başlangıç, yeni_bitiş));
        }
        seçenekleri_doğrula(&aday)?;
        self.seçenekler = aday;
        if !sessiz && !değişiklikler.is_empty() {
            self.olaylar.push(ÇalışmaOlayı::YakınlaştırmaDeğişti {
                değişiklikler: değişiklikler.clone(),
            });
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(değişiklikler)
    }

    /// `dispatchAction({type: "brush", areas})` model güncellemesi.
    /// `areas` verilmezse ECharts gibi mevcut seçim korunur; brush bileşeni
    /// olmayan bir grafikte eylem güvenli bir no-op'tur.
    pub fn fırça_alanlarını_ayarla(
        &mut self,
        alanlar: Option<Vec<FırçaSeçimAlanı>>,
        sessiz: bool,
    ) -> Result<bool, BilesenHatasi> {
        self.açık_mı("dispatchAction.brush")?;
        let Some(alanlar) = alanlar else {
            return Ok(false);
        };
        let Some(fırça) = self.seçenekler.fırça.as_mut() else {
            return Ok(false);
        };
        fırça.alanlar = alanlar;
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::SeçenekDeğişti);
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(true)
    }

    /// `dispatchAction({type: "selectDataRange"})` model güncellemesi.
    pub fn görsel_aralığı_ayarla(
        &mut self,
        sıra: Option<usize>,
        seçili: [f64; 2],
        sessiz: bool,
    ) -> Result<[f64; 2], BilesenHatasi> {
        self.açık_mı("dispatchAction.selectDataRange")?;
        if !seçili.iter().all(|değer| değer.is_finite()) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "visualMap.selected",
                ayrıntı: "iki uç da sonlu sayı olmalı".to_owned(),
            });
        }
        let sıra = sıra.unwrap_or(0);
        let eşleme = if let Some(tekil) = self.seçenekler.görsel_eşleme.as_mut() {
            (sıra == 0).then_some(tekil)
        } else {
            self.seçenekler.görsel_eşlemeler.get_mut(sıra)
        }
        .ok_or(BilesenHatasi::EksikVeri {
            bileşen: "visualMap",
            sıra,
        })?;
        if eşleme.parçalı_mı() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "visualMap.selected",
                ayrıntı: "dizi biçimli selected yalnız sürekli visualMap içindir".to_owned(),
            });
        }
        let seçili = [seçili[0].min(seçili[1]), seçili[0].max(seçili[1])];
        eşleme.seçili_aralık = Some(seçili);
        if !sessiz {
            self.olaylar
                .push(ÇalışmaOlayı::GörselAralıkDeğişti { sıra, seçili });
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(seçili)
    }

    /// Parçalı `visualMap` için `selectDataRange.selected` nesnesini modele
    /// uygular. ECharts gibi belirtilmeyen anahtarlar seçili sayılmaz.
    pub fn görsel_parçalarını_ayarla(
        &mut self,
        sıra: Option<usize>,
        seçili: BTreeMap<usize, bool>,
        sessiz: bool,
    ) -> Result<BTreeMap<usize, bool>, BilesenHatasi> {
        self.açık_mı("dispatchAction.selectDataRange")?;
        let sıra = sıra.unwrap_or(0);
        let eşleme = if let Some(tekil) = self.seçenekler.görsel_eşleme.as_mut() {
            (sıra == 0).then_some(tekil)
        } else {
            self.seçenekler.görsel_eşlemeler.get_mut(sıra)
        }
        .ok_or(BilesenHatasi::EksikVeri {
            bileşen: "visualMap",
            sıra,
        })?;
        if !eşleme.parçalı_mı() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "visualMap.selected",
                ayrıntı: "nesne biçimli selected yalnız parçalı visualMap içindir".to_owned(),
            });
        }
        let parça_sayısı = eşleme.parça_sayısı();
        if let Some(geçersiz) = seçili.keys().find(|sıra| **sıra >= parça_sayısı) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "visualMap.selected",
                ayrıntı: format!("{geçersiz} parça anahtarı 0..{parça_sayısı} dışında"),
            });
        }
        let seçili = (0..parça_sayısı)
            .map(|parça| (parça, seçili.get(&parça).copied().unwrap_or(false)))
            .collect::<BTreeMap<_, _>>();
        eşleme.kapalı_parçalar = seçili
            .iter()
            .filter_map(|(parça, açık)| (!*açık).then_some(*parça))
            .collect();
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::GörselParçalarDeğişti {
                sıra,
                seçili: seçili.clone(),
            });
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(seçili)
    }

    /// Toolbox `restore` eylemi. Etkileşimle veya sonraki normal merge'lerle
    /// değişmiş model yerine örneğin ilk base option yedeğini yeniden kurar.
    pub fn geri_yükle(&mut self, sessiz: bool) -> Result<(), BilesenHatasi> {
        self.açık_mı("dispatchAction.restore")?;
        self.seçenekler = self.geri_yükleme_seçenekleri.clone();
        self.yeniden_çizim_bekliyor = false;
        self.bekleyen_olay_sessiz = true;
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::GeriYüklendi);
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(())
    }

    /// `legendSelect`/`legendUnSelect`/`legendToggleSelect` ve selector
    /// action'larının ortak model güncellemesi.
    pub fn gösterge_seçimini_ayarla(
        &mut self,
        eylem: GöstergeSeçimEylemi,
        ad: Option<&str>,
        sessiz: bool,
    ) -> Result<BTreeMap<String, bool>, BilesenHatasi> {
        self.açık_mı("dispatchAction.legend")?;
        let adlar = gösterge_adları(&self.seçenekler);
        let ad_gerekli = matches!(
            eylem,
            GöstergeSeçimEylemi::Seç
                | GöstergeSeçimEylemi::SeçimiKaldır
                | GöstergeSeçimEylemi::Değiştir
        );
        let ad = if ad_gerekli {
            let ad =
                ad.filter(|ad| !ad.is_empty())
                    .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                        alan: "legend.action.name",
                        ayrıntı: "bu legend action için name gerekli".to_owned(),
                    })?;
            if !adlar.iter().any(|aday| aday == ad) {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "legend.action.name",
                    ayrıntı: format!("`{ad}` legend verisinde yok"),
                });
            }
            Some(ad)
        } else {
            None
        };
        let gösterge = self
            .seçenekler
            .gösterge
            .as_mut()
            .ok_or(BilesenHatasi::EksikVeri {
                bileşen: "legend",
                sıra: 0,
            })?;
        match eylem {
            GöstergeSeçimEylemi::Seç => gösterge.seç(ad.unwrap_or_default(), &adlar),
            GöstergeSeçimEylemi::SeçimiKaldır => {
                gösterge.seçimi_kaldır(ad.unwrap_or_default());
            }
            GöstergeSeçimEylemi::Değiştir => {
                gösterge.seçimi_değiştir(ad.unwrap_or_default(), &adlar);
            }
            GöstergeSeçimEylemi::TümünüSeç => gösterge.tümünü_seç(&adlar),
            GöstergeSeçimEylemi::TersiniSeç => gösterge.tersini_seç(&adlar),
        }
        let seçili: BTreeMap<String, bool> = adlar
            .into_iter()
            .map(|ad| {
                let değer = gösterge.seçili_mi(&ad);
                (ad, değer)
            })
            .collect();
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::GöstergeDeğişti {
                seçili: seçili.clone(),
            });
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(seçili)
    }

    /// ECharts `clear` karşılığı. Örnek/renderer ayarları korunur.
    pub fn temizle(&mut self, sessiz: bool) -> Result<(), BilesenHatasi> {
        self.açık_mı("clear")?;
        let boş = GrafikSeçenekleri {
            yerel: self.başlatma.yerel,
            ..GrafikSeçenekleri::default()
        };
        self.seçenekler = boş;
        self.yeniden_çizim_bekliyor = false;
        self.bekleyen_olay_sessiz = true;
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::Temizlendi);
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(())
    }

    /// ECharts `dispose` karşılığı; ikinci çağrı güvenli ve etkisizdir.
    pub fn kapat(&mut self) {
        if self.kapatıldı {
            return;
        }
        self.kapatıldı = true;
        self.yeniden_çizim_bekliyor = false;
        self.yükleme = None;
        self.olaylar.push(ÇalışmaOlayı::Kapatıldı);
    }

    pub fn kapatıldı_mı(&self) -> bool {
        self.kapatıldı
    }

    /// ECharts `resize` karşılığı.
    pub fn boyutlandır(
        &mut self,
        genişlik: f32,
        yükseklik: f32,
        aygıt_piksel_oranı: Option<f32>,
        sessiz: bool,
    ) -> Result<(), BilesenHatasi> {
        self.açık_mı("resize")?;
        let oran = aygıt_piksel_oranı.unwrap_or(self.başlatma.aygıt_piksel_oranı);
        boyutu_doğrula(genişlik, yükseklik, oran)?;
        self.başlatma.genişlik = genişlik;
        self.başlatma.yükseklik = yükseklik;
        self.başlatma.aygıt_piksel_oranı = oran;
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::BoyutDeğişti {
                genişlik,
                yükseklik,
                aygıt_piksel_oranı: oran,
            });
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(())
    }

    /// Responsive paketi mevcut örnek boyutunda yeniden çözüp atomik olarak
    /// uygular. Bu yöntem `resize` sonrasındaki media değerlendirmesidir.
    pub fn bileşik_seçenekleri_çöz(
        &mut self,
        seçenekler: &BileşikSeçenekler,
        zaman_karesi: Option<usize>,
        sessiz: bool,
    ) -> Result<(), BilesenHatasi> {
        self.açık_mı("media")?;
        let aday = seçenekler.çöz(
            self.başlatma.genişlik,
            self.başlatma.yükseklik,
            zaman_karesi,
        )?;
        seçenekleri_doğrula(&aday)?;
        self.seçenekler = aday;
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::SeçenekDeğişti);
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(())
    }

    /// `convertToPixel` (Cartesian2D dalı).
    pub fn piksele_çevir(
        &self,
        koordinat: &Kartezyen2B,
        değer: [f64; 2],
    ) -> Result<[f32; 2], BilesenHatasi> {
        self.açık_mı("convertToPixel")?;
        if !değer.into_iter().all(f64::is_finite) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "convertToPixel.value",
                ayrıntı: "koordinat değerleri sonlu olmalı".to_owned(),
            });
        }
        let [x_değeri, y_değeri] = değer;
        let (x, y) = koordinat.nokta(x_değeri, y_değeri);
        Ok([x, y])
    }

    /// `convertFromPixel` (Cartesian2D dalı).
    pub fn pikselden_çevir(
        &self,
        koordinat: &Kartezyen2B,
        piksel: [f32; 2],
    ) -> Result<[f64; 2], BilesenHatasi> {
        self.açık_mı("convertFromPixel")?;
        if !piksel.into_iter().all(f32::is_finite) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "convertFromPixel.value",
                ayrıntı: "piksel değerleri sonlu olmalı".to_owned(),
            });
        }
        let [x, y] = piksel;
        Ok([
            koordinat.x.pikselden_veriye(x),
            koordinat.y.pikselden_veriye(y),
        ])
    }

    /// `containPixel` (Cartesian2D grid dalı).
    pub fn piksel_içeriyor_mu(
        &self,
        koordinat: &Kartezyen2B,
        piksel: [f32; 2],
    ) -> Result<bool, BilesenHatasi> {
        self.açık_mı("containPixel")?;
        let sonlu = piksel.into_iter().all(f32::is_finite);
        let [x, y] = piksel;
        Ok(sonlu && koordinat.alan.içeriyor_mu((x, y)))
    }

    /// `appendData` karşılığı. Seçici çözülür, veri destekleyen serinin
    /// deposuna eklenir ve model yine doğrulanır.
    pub fn veri_ekle(
        &mut self,
        seçici: SeriSeçici,
        veri: impl IntoIterator<Item = VeriÖğesi>,
        sessiz: bool,
    ) -> Result<usize, BilesenHatasi> {
        self.açık_mı("appendData")?;
        let sıra = seri_sırasını_bul(&self.seçenekler, &seçici).ok_or_else(|| {
            BilesenHatasi::EksikVeri {
                bileşen: "appendData.series",
                sıra: seçici_sıra_ipucu(&seçici),
            }
        })?;
        let eklenecek: Vec<VeriÖğesi> = veri.into_iter().collect();
        let adet = eklenecek.len();
        let Some(seri) = self.seçenekler.seriler.get_mut(sıra) else {
            return Err(BilesenHatasi::EksikVeri {
                bileşen: "appendData.series",
                sıra,
            });
        };
        let Some(depo) = seri.veri_mut() else {
            return Err(BilesenHatasi::Desteklenmeyen {
                özellik: "appendData",
                ayrıntı: format!("{sıra}. seri ayrı bir hiyerarşik/bağ veri modeli kullanıyor"),
            });
        };
        depo.extend(eklenecek);
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::VeriEklendi {
                seri_sırası: sıra,
                adet,
            });
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(sıra)
    }

    /// Bağ tabanlı çekirdek `series.lines` için tipli `appendData`.
    pub fn hat_verisi_ekle(
        &mut self,
        seçici: SeriSeçici,
        veri: impl IntoIterator<Item = HatVerisi>,
        sessiz: bool,
    ) -> Result<usize, BilesenHatasi> {
        self.açık_mı("appendData")?;
        let sıra = seri_sırasını_bul(&self.seçenekler, &seçici).ok_or_else(|| {
            BilesenHatasi::EksikVeri {
                bileşen: "appendData.series",
                sıra: seçici_sıra_ipucu(&seçici),
            }
        })?;
        let eklenecek: Vec<HatVerisi> = veri.into_iter().collect();
        let adet = eklenecek.len();
        let Some(seri) = self.seçenekler.seriler.get_mut(sıra) else {
            return Err(BilesenHatasi::EksikVeri {
                bileşen: "appendData.series",
                sıra,
            });
        };
        let Seri::Hatlar(hatlar) = seri else {
            return Err(BilesenHatasi::Desteklenmeyen {
                özellik: "appendData.lines",
                ayrıntı: format!("{sıra}. seri `lines` değildir"),
            });
        };
        hatlar.veri.extend(eklenecek);
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::VeriEklendi {
                seri_sırası: sıra,
                adet,
            });
            self.olaylar.push(ÇalışmaOlayı::YenidenÇizildi);
        }
        Ok(sıra)
    }

    pub fn yüklemeyi_göster(
        &mut self,
        metin: Option<impl Into<String>>,
        sessiz: bool,
    ) -> Result<(), BilesenHatasi> {
        self.açık_mı("showLoading")?;
        self.yükleme = metin.map(Into::into);
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::YüklemeDeğişti {
                görünür: true,
                metin: self.yükleme.clone(),
            });
        }
        Ok(())
    }

    pub fn yüklemeyi_gizle(&mut self, sessiz: bool) -> Result<(), BilesenHatasi> {
        self.açık_mı("hideLoading")?;
        self.yükleme = None;
        if !sessiz {
            self.olaylar.push(ÇalışmaOlayı::YüklemeDeğişti {
                görünür: false,
                metin: None,
            });
        }
        Ok(())
    }

    pub fn yükleme_metni(&self) -> Option<&str> {
        self.yükleme.as_deref()
    }

    /// Olay günlüğünü tüketir. Etkileşim testleri bu yöntemi snapshot'lar.
    pub fn olayları_al(&mut self) -> Vec<ÇalışmaOlayı> {
        std::mem::take(&mut self.olaylar)
    }

    fn bağlı_yakınlaştırma_sıraları(
        &self,
        sıra: Option<usize>,
    ) -> Result<Vec<usize>, BilesenHatasi> {
        let toplam = self.seçenekler.veri_yakınlaştırmaları.len();
        let Some(sıra) = sıra else {
            return Ok((0..toplam).collect());
        };
        let başlangıç =
            self.seçenekler
                .veri_yakınlaştırmaları
                .get(sıra)
                .ok_or(BilesenHatasi::EksikVeri {
                    bileşen: "dataZoom",
                    sıra,
                })?;
        Ok(self
            .seçenekler
            .veri_yakınlaştırmaları
            .iter()
            .enumerate()
            .filter_map(|(sıra, yakınlaştırma)| {
                başlangıç
                    .aynı_eksenleri_hedefler(yakınlaştırma)
                    .then_some(sıra)
            })
            .collect())
    }

    fn açık_mı(&self, işlem: &'static str) -> Result<(), BilesenHatasi> {
        if self.kapatıldı {
            return Err(BilesenHatasi::KapatılmışÖrnek { işlem });
        }
        Ok(())
    }
}

fn gösterge_adları(seçenekler: &GrafikSeçenekleri) -> Vec<String> {
    if let Some(gösterge) = &seçenekler.gösterge
        && !gösterge.veri.is_empty()
    {
        return benzersiz_adlar(gösterge.veri.iter().cloned());
    }
    let mut adlar = Vec::new();
    for seri in &seçenekler.seriler {
        match seri {
            Seri::Pasta(seri) => adlar.extend(seri.veri.iter().filter_map(|öğe| öğe.ad.clone())),
            Seri::Huni(seri) => adlar.extend(seri.veri.iter().filter_map(|öğe| öğe.ad.clone())),
            Seri::Radar(seri) => adlar.extend(seri.veri.iter().filter_map(|öğe| öğe.ad.clone())),
            _ => {
                if let Some(ad) = seri.ad().filter(|ad| !ad.is_empty()) {
                    adlar.push(ad.to_owned());
                }
            }
        }
    }
    benzersiz_adlar(adlar)
}

fn benzersiz_adlar(adlar: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut görülen = HashSet::new();
    adlar
        .into_iter()
        .filter(|ad| !ad.is_empty() && görülen.insert(ad.clone()))
        .collect()
}

fn boyutu_doğrula(genişlik: f32, yükseklik: f32, oran: f32) -> Result<(), BilesenHatasi> {
    for (alan, değer) in [
        ("genişlik", genişlik),
        ("yükseklik", yükseklik),
        ("aygıt_piksel_oranı", oran),
    ] {
        if !değer.is_finite() || değer <= 0.0 {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan,
                ayrıntı: format!("{değer} sıfırdan büyük sonlu bir değer olmalı"),
            });
        }
    }
    Ok(())
}

fn seçenekleri_doğrula(seçenekler: &GrafikSeçenekleri) -> Result<(), BilesenHatasi> {
    seçenekler.doğrula()?;
    let mut görülen = HashSet::new();
    for sıra in 0..seçenekler.seriler.len() {
        let Some(kimlik) = seçenekler.seri_kimliği(sıra) else {
            continue;
        };
        if !görülen.insert(kimlik) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "series.id",
                ayrıntı: format!(
                    "`{kimlik}` kimliği bir option içinde yalnız bir kez kullanılabilir"
                ),
            });
        }
    }
    Ok(())
}

fn yamayı_uygula(
    hedef: &mut GrafikSeçenekleri,
    yama: &SeçenekYaması,
    kip: &SeçenekAyarlamaKipi,
) -> Result<(), BilesenHatasi> {
    let öntanımlı = GrafikSeçenekleri::default();

    macro_rules! alanı_uygula {
        ($alan:ident, $tür:expr) => {
            if yama.sağlandı_mı($tür) {
                hedef.$alan = yama.değer.$alan.clone();
            } else if kip.değiştirerek_birleştir.contains(&$tür) {
                hedef.$alan = öntanımlı.$alan.clone();
            }
        };
    }

    alanı_uygula!(başlık, SeçenekAlanı::Başlık);
    alanı_uygula!(başlıklar, SeçenekAlanı::Başlık);
    alanı_uygula!(gösterge, SeçenekAlanı::Gösterge);
    alanı_uygula!(ızgara, SeçenekAlanı::Izgara);
    alanı_uygula!(ızgaralar, SeçenekAlanı::Izgaralar);
    alanı_uygula!(x_ekseni, SeçenekAlanı::XEkseni);
    alanı_uygula!(y_ekseni, SeçenekAlanı::YEkseni);
    alanı_uygula!(x_eksenleri, SeçenekAlanı::XEksenleri);
    alanı_uygula!(y_eksenleri, SeçenekAlanı::YEksenleri);
    alanı_uygula!(ipucu, SeçenekAlanı::İpucu);
    if yama.sağlandı_mı(SeçenekAlanı::GörselEşleme) {
        hedef.görsel_eşleme = yama.değer.görsel_eşleme.clone();
        hedef.görsel_eşlemeler = yama.değer.görsel_eşlemeler.clone();
    } else if kip
        .değiştirerek_birleştir
        .contains(&SeçenekAlanı::GörselEşleme)
    {
        hedef.görsel_eşleme = öntanımlı.görsel_eşleme.clone();
        hedef.görsel_eşlemeler = öntanımlı.görsel_eşlemeler.clone();
    }
    alanı_uygula!(radar, SeçenekAlanı::Radar);
    alanı_uygula!(kutupsal, SeçenekAlanı::Kutupsal);
    alanı_uygula!(matris, SeçenekAlanı::Matris);
    alanı_uygula!(takvimler, SeçenekAlanı::Takvimler);
    alanı_uygula!(tek_eksenler, SeçenekAlanı::TekEksenler);
    alanı_uygula!(veri_kümesi, SeçenekAlanı::VeriKümesi);
    alanı_uygula!(veri_kümeleri, SeçenekAlanı::VeriKümesi);
    alanı_uygula!(veri_yakınlaştırmaları, SeçenekAlanı::VeriYakınlaştırmaları);
    alanı_uygula!(araç_kutusu, SeçenekAlanı::AraçKutusu);
    alanı_uygula!(fırça, SeçenekAlanı::Fırça);
    alanı_uygula!(grafik, SeçenekAlanı::Grafik);
    alanı_uygula!(zaman_şeridi, SeçenekAlanı::ZamanŞeridi);
    alanı_uygula!(palet, SeçenekAlanı::Palet);
    alanı_uygula!(arkaplan, SeçenekAlanı::Arkaplan);
    alanı_uygula!(koyu, SeçenekAlanı::Koyu);
    if yama.sağlandı_mı(SeçenekAlanı::Yerel) {
        hedef.yerel = yama.değer.yerel;
    } else if kip.değiştirerek_birleştir.contains(&SeçenekAlanı::Yerel) {
        hedef.yerel = öntanımlı.yerel;
    }
    alanı_uygula!(animasyon, SeçenekAlanı::Animasyon);
    alanı_uygula!(animasyon_süresi, SeçenekAlanı::AnimasyonSüresi);
    alanı_uygula!(
        animasyon_süresi_güncelleme,
        SeçenekAlanı::AnimasyonSüresiGüncelleme
    );
    alanı_uygula!(animasyon_eğrisi, SeçenekAlanı::AnimasyonEğrisi);

    if yama.sağlandı_mı(SeçenekAlanı::Seriler) {
        let değiştir =
            kip.birleştirme_yok || kip.değiştirerek_birleştir.contains(&SeçenekAlanı::Seriler);
        serileri_birleştir(hedef, &yama.değer, değiştir);
    } else if kip.değiştirerek_birleştir.contains(&SeçenekAlanı::Seriler) {
        hedef.seriler.clear();
        hedef.seri_kimlikleri.clear();
    }

    for veri_yaması in &yama.x_ekseni_veri_yamaları {
        eksen_verisini_yamala(hedef, true, veri_yaması)?;
    }
    for veri_yaması in &yama.y_ekseni_veri_yamaları {
        eksen_verisini_yamala(hedef, false, veri_yaması)?;
    }
    for kırılma_yaması in &yama.eksen_kırılma_yamaları {
        eksen_kırılmalarını_yamala(hedef, kırılma_yaması)?;
    }

    for veri_yaması in &yama.seri_veri_yamaları {
        let sıra = seri_sırasını_bul(hedef, &veri_yaması.seçici).ok_or_else(|| {
            BilesenHatasi::EksikVeri {
                bileşen: "setOption.series",
                sıra: seçici_sıra_ipucu(&veri_yaması.seçici),
            }
        })?;
        let seri = hedef
            .seriler
            .get_mut(sıra)
            .ok_or(BilesenHatasi::EksikVeri {
                bileşen: "setOption.series",
                sıra,
            })?;
        let depo = seri
            .veri_mut()
            .ok_or_else(|| BilesenHatasi::Desteklenmeyen {
                özellik: "setOption.series.data",
                ayrıntı: format!("{sıra}. seri ayrı bir hiyerarşik/bağ veri modeli kullanıyor"),
            })?;
        *depo = veri_yaması.veri.clone();
    }

    Ok(())
}

fn eksen_verisini_yamala(
    hedef: &mut GrafikSeçenekleri,
    yatay: bool,
    yama: &EksenVeriYaması,
) -> Result<(), BilesenHatasi> {
    let (bileşen, eksen) = if yatay {
        let eksen = if hedef.x_eksenleri.is_empty() {
            (yama.sıra == 0).then(|| hedef.x_ekseni.get_or_insert_with(Eksen::kategori))
        } else {
            hedef.x_eksenleri.get_mut(yama.sıra)
        };
        ("setOption.xAxis", eksen)
    } else {
        let eksen = if hedef.y_eksenleri.is_empty() {
            (yama.sıra == 0).then(|| hedef.y_ekseni.get_or_insert_with(Eksen::değer))
        } else {
            hedef.y_eksenleri.get_mut(yama.sıra)
        };
        ("setOption.yAxis", eksen)
    };
    let eksen = eksen.ok_or(BilesenHatasi::EksikVeri {
        bileşen,
        sıra: yama.sıra,
    })?;
    eksen.veri.clone_from(&yama.veri);
    Ok(())
}

fn eksen_kırılmalarını_yamala(
    hedef: &mut GrafikSeçenekleri,
    yama: &EksenKırılmaYaması,
) -> Result<(), BilesenHatasi> {
    let bileşen = match yama.boyut {
        EksenBoyutu::X => "setOption.xAxis",
        EksenBoyutu::Y => "setOption.yAxis",
        EksenBoyutu::Tek => "setOption.singleAxis",
    };
    let eksen = ekseni_mut(hedef, yama.boyut, yama.sıra).ok_or(BilesenHatasi::EksikVeri {
        bileşen,
        sıra: yama.sıra,
    })?;
    eksen.kırılmalar.clone_from(&yama.kırılmalar);
    Ok(())
}

fn eksen_sayısı(seçenekler: &GrafikSeçenekleri, boyut: EksenBoyutu) -> usize {
    match boyut {
        EksenBoyutu::X => {
            if seçenekler.x_eksenleri.is_empty() {
                usize::from(seçenekler.x_ekseni.is_some())
            } else {
                seçenekler.x_eksenleri.len()
            }
        }
        EksenBoyutu::Y => {
            if seçenekler.y_eksenleri.is_empty() {
                usize::from(seçenekler.y_ekseni.is_some())
            } else {
                seçenekler.y_eksenleri.len()
            }
        }
        EksenBoyutu::Tek => seçenekler.tek_eksenler.len(),
    }
}

fn ekseni_mut(
    seçenekler: &mut GrafikSeçenekleri,
    boyut: EksenBoyutu,
    sıra: usize,
) -> Option<&mut Eksen> {
    match boyut {
        EksenBoyutu::X if seçenekler.x_eksenleri.is_empty() => (sıra == 0)
            .then_some(seçenekler.x_ekseni.as_mut())
            .flatten(),
        EksenBoyutu::X => seçenekler.x_eksenleri.get_mut(sıra),
        EksenBoyutu::Y if seçenekler.y_eksenleri.is_empty() => (sıra == 0)
            .then_some(seçenekler.y_ekseni.as_mut())
            .flatten(),
        EksenBoyutu::Y => seçenekler.y_eksenleri.get_mut(sıra),
        EksenBoyutu::Tek => seçenekler
            .tek_eksenler
            .get_mut(sıra)
            .map(|tek| &mut tek.eksen),
    }
}

fn serileri_birleştir(
    hedef: &mut GrafikSeçenekleri, gelen: &GrafikSeçenekleri, değiştir: bool
) {
    if değiştir {
        hedef.seriler = gelen.seriler.clone();
        hedef.seri_kimlikleri = (0..gelen.seriler.len())
            .map(|sıra| gelen.seri_kimliği(sıra).map(str::to_owned))
            .collect();
        return;
    }

    let mut kullanılan = vec![false; hedef.seriler.len()];
    for gelen_sıra in 0..gelen.seriler.len() {
        let Some(gelen_seri) = gelen.seriler.get(gelen_sıra) else {
            continue;
        };
        let gelen_kimlik = gelen.seri_kimliği(gelen_sıra);
        let gelen_ad = gelen_seri.ad().filter(|ad| !ad.is_empty());

        let eşleşme = if let Some(kimlik) = gelen_kimlik {
            (0..hedef.seriler.len()).find(|&sıra| {
                !kullanılan.get(sıra).copied().unwrap_or(true)
                    && hedef.seri_kimliği(sıra) == Some(kimlik)
            })
        } else {
            gelen_ad
                .and_then(|ad| {
                    (0..hedef.seriler.len()).find(|&sıra| {
                        !kullanılan.get(sıra).copied().unwrap_or(true)
                            && hedef
                                .seriler
                                .get(sıra)
                                .and_then(Seri::ad)
                                .filter(|eski_ad| !eski_ad.is_empty())
                                == Some(ad)
                    })
                })
                .or_else(|| {
                    (gelen_sıra < hedef.seriler.len()
                        && !kullanılan.get(gelen_sıra).copied().unwrap_or(true))
                    .then_some(gelen_sıra)
                })
        };

        if let Some(eski_sıra) = eşleşme {
            if let Some(eski) = hedef.seriler.get_mut(eski_sıra) {
                *eski = gelen_seri.clone();
            }
            if let Some(kullanıldı) = kullanılan.get_mut(eski_sıra) {
                *kullanıldı = true;
            }
            seri_kimlik_uzunluğunu_tamamla(hedef);
            if let Some(yuva) = hedef.seri_kimlikleri.get_mut(eski_sıra)
                && let Some(kimlik) = gelen_kimlik
            {
                *yuva = Some(kimlik.to_owned());
            }
        } else {
            hedef.seriler.push(gelen_seri.clone());
            hedef.seri_kimlikleri.push(gelen_kimlik.map(str::to_owned));
            kullanılan.push(true);
        }
    }
}

fn seri_kimlik_uzunluğunu_tamamla(seçenekler: &mut GrafikSeçenekleri) {
    if seçenekler.seri_kimlikleri.len() < seçenekler.seriler.len() {
        seçenekler
            .seri_kimlikleri
            .resize(seçenekler.seriler.len(), None);
    }
}

fn seri_sırasını_bul(seçenekler: &GrafikSeçenekleri, seçici: &SeriSeçici) -> Option<usize> {
    match seçici {
        SeriSeçici::Sıra(sıra) => (*sıra < seçenekler.seriler.len()).then_some(*sıra),
        SeriSeçici::Kimlik(kimlik) => (0..seçenekler.seriler.len())
            .find(|&sıra| seçenekler.seri_kimliği(sıra) == Some(kimlik.as_str())),
        SeriSeçici::Ad(ad) => seçenekler
            .seriler
            .iter()
            .position(|seri| seri.ad() == Some(ad.as_str())),
    }
}

fn seçici_sıra_ipucu(seçici: &SeriSeçici) -> usize {
    match seçici {
        SeriSeçici::Sıra(sıra) => *sıra,
        SeriSeçici::Kimlik(_) | SeriSeçici::Ad(_) => usize::MAX,
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
    use crate::model::bilesen::Başlık;
    use crate::model::deger::VeriDeğeri;
    use crate::model::eksen::EksenKırılmaAlanı;
    use crate::model::grafik_bileseni::{GrafikBileşeni, GrafikÖğesi};
    use crate::model::seri::{PastaSerisi, ÇizgiSerisi};

    fn çalışma(seçenekler: GrafikSeçenekleri) -> GrafikÇalışmaZamanı {
        GrafikÇalışmaZamanı::yeni(ÖrnekBaşlatmaSeçenekleri::default(), seçenekler).unwrap()
    }

    fn ilk_değer(seçenekler: &GrafikSeçenekleri, sıra: usize) -> f64 {
        seçenekler.seriler[sıra].veri()[0]
            .değer
            .sayı()
            .expect("sayısal test verisi")
    }

    #[test]
    fn normal_birleştirme_id_ad_ve_sıra_önceliğini_izler() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .kimlikli_seri("a", ÇizgiSerisi::yeni().ad("A").veri([1]))
            .seri(ÇizgiSerisi::yeni().ad("B").veri([2]))
            .seri(ÇizgiSerisi::yeni().veri([3]));
        let mut çalışma = çalışma(başlangıç);
        let yama = SeçenekYaması::yeni()
            .kimlikli_seri("a", ÇizgiSerisi::yeni().ad("Yeni A").veri([10]))
            .seri(ÇizgiSerisi::yeni().ad("B").veri([20]))
            .seri(ÇizgiSerisi::yeni().veri([30]));

        çalışma
            .seçenekleri_ayarla(yama, SeçenekAyarlamaKipi::default())
            .unwrap();
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert_eq!(sonuç.seriler.len(), 3);
        assert_eq!(ilk_değer(&sonuç, 0), 10.0);
        assert_eq!(ilk_değer(&sonuç, 1), 20.0);
        assert_eq!(ilk_değer(&sonuç, 2), 30.0);
        assert_eq!(sonuç.seri_kimliği(0), Some("a"));
    }

    #[test]
    fn graphic_kok_yamasi_gorunurlugu_degistirir_ve_kaldirilabilir() {
        let düğme = |yoksay| {
            GrafikBileşeni::yeni().öğe(
                GrafikÖğesi::dikdörtgen(
                    crate::koordinat::Dikdörtgen::yeni(0.0, 0.0, 140.0, 24.0),
                )
                .kimlik("collapseAxisBreakBtn")
                .yoksay(yoksay),
            )
        };
        let mut çalışma = çalışma(GrafikSeçenekleri::yeni().grafik(düğme(true)));

        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().grafik(düğme(false)),
                SeçenekAyarlamaKipi::default(),
            )
            .unwrap();
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert!(!sonuç.grafik.as_ref().unwrap().öğeler[0].yoksay);

        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().grafiği_kaldır(),
                SeçenekAyarlamaKipi::default(),
            )
            .unwrap();
        assert!(çalışma.seçenekleri_al().unwrap().grafik.is_none());
    }

    #[test]
    fn normal_birleştirme_yamada_olmayan_seriyi_korur() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .kimlikli_seri("a", ÇizgiSerisi::yeni().veri([1]))
            .kimlikli_seri("b", ÇizgiSerisi::yeni().veri([2]));
        let mut çalışma = çalışma(başlangıç);
        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().kimlikli_seri("a", ÇizgiSerisi::yeni().veri([9])),
                SeçenekAyarlamaKipi::default(),
            )
            .unwrap();
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert_eq!(sonuç.seriler.len(), 2);
        assert_eq!(ilk_değer(&sonuç, 1), 2.0);
    }

    #[test]
    fn seri_verisi_yaması_diger_seri_alanlarini_korur() {
        let başlangıç = GrafikSeçenekleri::yeni().kimlikli_seri(
            "akış",
            ÇizgiSerisi::yeni()
                .ad("Fake Data")
                .sembol_göster(false)
                .veri([1, 2, 3]),
        );
        let mut çalışma = çalışma(başlangıç);

        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().seri_verisi(SeriSeçici::kimlik("akış"), [7, 8, 9]),
                SeçenekAyarlamaKipi::default(),
            )
            .unwrap();

        let sonuç = çalışma.seçenekleri_al().unwrap();
        let Seri::Çizgi(çizgi) = &sonuç.seriler[0] else {
            panic!("çizgi serisi bekleniyordu");
        };
        assert_eq!(çizgi.ad.as_deref(), Some("Fake Data"));
        assert!(!çizgi.sembol_göster);
        assert_eq!(
            çizgi
                .veri
                .iter()
                .filter_map(|öğe| öğe.değer.sayı())
                .collect::<Vec<_>>(),
            vec![7.0, 8.0, 9.0]
        );
        assert_eq!(sonuç.seri_kimliği(0), Some("akış"));
        assert_eq!(
            çalışma.olayları_al(),
            vec![ÇalışmaOlayı::SeçenekDeğişti, ÇalışmaOlayı::YenidenÇizildi]
        );
    }

    #[test]
    fn seri_verisi_yaması_geçersiz_seçicide_atomiktir() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .başlık(Başlık::yeni().metin("Korunacak"))
            .seri(ÇizgiSerisi::yeni().veri([1, 2]));
        let mut çalışma = çalışma(başlangıç);

        let hata = çalışma.seçenekleri_ayarla(
            SeçenekYaması::yeni()
                .başlık(Başlık::yeni().metin("Uygulanmamalı"))
                .seri_verisi(SeriSeçici::Sıra(4), [9]),
            SeçenekAyarlamaKipi::default(),
        );

        assert!(matches!(
            hata,
            Err(BilesenHatasi::EksikVeri {
                bileşen: "setOption.series",
                sıra: 4
            })
        ));
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert_eq!(
            sonuç
                .başlık
                .as_ref()
                .and_then(|başlık| başlık.metin.as_deref()),
            Some("Korunacak")
        );
        assert_eq!(ilk_değer(&sonuç, 0), 1.0);
    }

    #[test]
    fn eksen_verisi_yaması_diger_eksen_alanlarini_korur() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .x_ekseni_ekle(
                Eksen::kategori()
                    .ad("Saat")
                    .kenar_boşluğu(true)
                    .veri(["eski-a", "eski-b"]),
            )
            .x_ekseni_ekle(
                Eksen::kategori()
                    .ad("Sıra")
                    .kenar_boşluğu(false)
                    .veri(["0", "1"]),
            )
            .y_ekseni_ekle(Eksen::kategori().ad("Grup").veri(["eski"]))
            .seri(ÇizgiSerisi::yeni().veri([1, 2]));
        let mut çalışma = çalışma(başlangıç);

        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni()
                    .x_ekseni_verisi(0, ["yeni-a", "yeni-b"])
                    .x_ekseni_verisi(1, ["8", "9"])
                    .y_ekseni_verisi(0, ["yeni"]),
                SeçenekAyarlamaKipi::default(),
            )
            .unwrap();

        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert_eq!(sonuç.x_eksenleri[0].veri, ["yeni-a", "yeni-b"]);
        assert_eq!(sonuç.x_eksenleri[0].ad.as_deref(), Some("Saat"));
        assert_eq!(sonuç.x_eksenleri[0].kenar_boşluğu, Some(true));
        assert_eq!(sonuç.x_eksenleri[1].veri, ["8", "9"]);
        assert_eq!(sonuç.x_eksenleri[1].ad.as_deref(), Some("Sıra"));
        assert_eq!(sonuç.x_eksenleri[1].kenar_boşluğu, Some(false));
        assert_eq!(sonuç.y_eksenleri[0].veri, ["yeni"]);
        assert_eq!(sonuç.y_eksenleri[0].ad.as_deref(), Some("Grup"));
        assert_eq!(
            çalışma.olayları_al(),
            vec![ÇalışmaOlayı::SeçenekDeğişti, ÇalışmaOlayı::YenidenÇizildi]
        );
    }

    #[test]
    fn eksen_verisi_yaması_geçersiz_sırada_atomiktir() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .başlık(Başlık::yeni().metin("Korunacak"))
            .x_ekseni(Eksen::kategori().veri(["A", "B"]))
            .seri(ÇizgiSerisi::yeni().veri([1, 2]));
        let mut çalışma = çalışma(başlangıç);

        let hata = çalışma.seçenekleri_ayarla(
            SeçenekYaması::yeni()
                .başlık(Başlık::yeni().metin("Uygulanmamalı"))
                .x_ekseni_verisi(1, ["C"]),
            SeçenekAyarlamaKipi::default(),
        );

        assert!(matches!(
            hata,
            Err(BilesenHatasi::EksikVeri {
                bileşen: "setOption.xAxis",
                sıra: 1
            })
        ));
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert_eq!(
            sonuç
                .başlık
                .as_ref()
                .and_then(|başlık| başlık.metin.as_deref()),
            Some("Korunacak")
        );
        assert_eq!(sonuç.etkin_x_eksenleri()[0].veri, ["A", "B"]);
    }

    #[test]
    fn eksen_kirilma_yamasi_diger_eksen_alanlarini_korur() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .x_ekseni(Eksen::kategori().veri(["A", "B"]))
            .y_ekseni(
                Eksen::değer()
                    .ad("Fiyat")
                    .kırılma_alanı(
                        EksenKırılmaAlanı::yeni()
                            .opaklık(1.0)
                            .zikzak_genliği(2.0)
                            .zikzak_z(200),
                    )
                    .kırılma(EksenKırılması::yeni(5_000.0, 100_000.0).boşluk("2%")),
            )
            .seri(ÇizgiSerisi::yeni().veri([1, 2]));
        let mut çalışma = çalışma(başlangıç);

        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().y_ekseni_kırılmaları(
                    0,
                    [
                        EksenKırılması::yeni(5_000.0, 100_000.0).boşluk("2%"),
                        EksenKırılması::yeni(2_000.0, 3_000.0).boşluk("2%"),
                    ],
                ),
                SeçenekAyarlamaKipi::default(),
            )
            .unwrap();

        let sonuç = çalışma.seçenekleri_al().unwrap();
        let eksen = sonuç.y_ekseni.as_ref().unwrap();
        assert_eq!(eksen.ad.as_deref(), Some("Fiyat"));
        assert_eq!(eksen.kırılmalar.len(), 2);
        assert_eq!(eksen.kırılmalar[1].başlangıç, 2_000.0);
        assert_eq!(eksen.kırılmalar[1].bitiş, 3_000.0);
        assert_eq!(eksen.kırılma_alanı.opaklık, 1.0);
        assert_eq!(eksen.kırılma_alanı.zikzak_genliği, 2.0);
        assert_eq!(eksen.kırılma_alanı.zikzak_z, 200);
    }

    #[test]
    fn replace_merge_eslesmeyen_seriyi_siler() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .kimlikli_seri("a", ÇizgiSerisi::yeni().veri([1]))
            .kimlikli_seri("b", ÇizgiSerisi::yeni().veri([2]));
        let mut çalışma = çalışma(başlangıç);
        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().kimlikli_seri("b", ÇizgiSerisi::yeni().veri([8])),
                SeçenekAyarlamaKipi::yeni().değiştirerek(SeçenekAlanı::Seriler),
            )
            .unwrap();
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert_eq!(sonuç.seriler.len(), 1);
        assert_eq!(sonuç.seri_kimliği(0), Some("b"));
        assert_eq!(ilk_değer(&sonuç, 0), 8.0);
    }

    #[test]
    fn not_merge_eski_kok_bilesenleri_birakir() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .başlık(Başlık::yeni().metin("Eski"))
            .seri(ÇizgiSerisi::yeni().veri([1]));
        let mut çalışma = çalışma(başlangıç);
        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().seri(ÇizgiSerisi::yeni().veri([7])),
                SeçenekAyarlamaKipi::yeni().birleştirme_yok(true),
            )
            .unwrap();
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert!(sonuç.başlık.is_none());
        assert_eq!(sonuç.seriler.len(), 1);
        assert_eq!(ilk_değer(&sonuç, 0), 7.0);
    }

    #[test]
    fn gecersiz_yama_atomik_olarak_geri_alinir() {
        let başlangıç = GrafikSeçenekleri::yeni()
            .başlık(Başlık::yeni().metin("Korunacak"))
            .seri(ÇizgiSerisi::yeni().veri([1]));
        let mut çalışma = çalışma(başlangıç);
        let hata = çalışma.seçenekleri_ayarla(
            SeçenekYaması::yeni().animasyon_süresi(f32::NAN),
            SeçenekAyarlamaKipi::default(),
        );
        assert!(hata.is_err());
        let sonuç = çalışma.seçenekleri_al().unwrap();
        assert_eq!(
            sonuç.başlık.as_ref().and_then(|b| b.metin.as_deref()),
            Some("Korunacak")
        );
        assert_eq!(ilk_değer(&sonuç, 0), 1.0);
    }

    #[test]
    fn yinelenen_series_id_reddedilir() {
        let mut çalışma = çalışma(GrafikSeçenekleri::default());
        let sonuç = çalışma.seçenekleri_ayarla(
            SeçenekYaması::yeni()
                .kimlikli_seri("aynı", ÇizgiSerisi::yeni().veri([1]))
                .kimlikli_seri("aynı", ÇizgiSerisi::yeni().veri([2])),
            SeçenekAyarlamaKipi::yeni().değiştirerek(SeçenekAlanı::Seriler),
        );
        assert!(matches!(
            sonuç,
            Err(BilesenHatasi::GeçersizSeçenek {
                alan: "series.id",
                ..
            })
        ));
        assert!(çalışma.seçenekleri_al().unwrap().seriler.is_empty());
    }

    #[test]
    fn lazy_update_modeli_hemen_renderi_flush_sirasinda_degistirir() {
        let mut çalışma = çalışma(GrafikSeçenekleri::default());
        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().seri(ÇizgiSerisi::yeni().veri([4])),
                SeçenekAyarlamaKipi::yeni().tembel(true),
            )
            .unwrap();
        assert_eq!(çalışma.seçenekleri_al().unwrap().seriler.len(), 1);
        assert!(çalışma.olayları_al().is_empty());
        assert!(çalışma.yeniden_çizim_bekliyor_mu());
        assert!(çalışma.bekleyeni_çiz().unwrap());
        assert_eq!(
            çalışma.olayları_al(),
            vec![ÇalışmaOlayı::SeçenekDeğişti, ÇalışmaOlayı::YenidenÇizildi]
        );
    }

    #[test]
    fn silent_tum_set_option_olaylarini_bastirir() {
        let mut çalışma = çalışma(GrafikSeçenekleri::default());
        çalışma
            .seçenekleri_ayarla(
                SeçenekYaması::yeni().seri(ÇizgiSerisi::yeni().veri([4])),
                SeçenekAyarlamaKipi::yeni().sessiz(true),
            )
            .unwrap();
        assert!(çalışma.olayları_al().is_empty());
    }

    #[test]
    fn append_data_id_ile_seriyi_bulur() {
        let başlangıç =
            GrafikSeçenekleri::yeni().kimlikli_seri("hedef", ÇizgiSerisi::yeni().veri([1]));
        let mut çalışma = çalışma(başlangıç);
        let sıra = çalışma
            .veri_ekle(
                SeriSeçici::kimlik("hedef"),
                [VeriÖğesi::from(2), VeriÖğesi::from(3)],
                false,
            )
            .unwrap();
        assert_eq!(sıra, 0);
        let sonuç = çalışma.seçenekleri_al().unwrap();
        let sayılar: Vec<_> = sonuç.seriler[0]
            .veri()
            .iter()
            .filter_map(|öğe| match öğe.değer {
                VeriDeğeri::Sayı(sayı) => Some(sayı),
                _ => None,
            })
            .collect();
        assert_eq!(sayılar, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn append_data_hiyerarsik_seride_tipli_hata_verir() {
        let başlangıç =
            GrafikSeçenekleri::yeni().seri(PastaSerisi::yeni().veri([VeriÖğesi::adlı("A", 1)]));
        let mut çalışma = çalışma(başlangıç);
        // Pasta düz veri depoladığı için desteklenir; hiyerarşik veri kullanan
        // AğaçHaritası ayrıca kendi düğüm API'sinden güncellenmelidir.
        assert!(
            çalışma
                .veri_ekle(SeriSeçici::Sıra(0), [VeriÖğesi::adlı("B", 2)], true)
                .is_ok()
        );
    }

    #[test]
    fn resize_clear_dispose_yasam_dongusu() {
        let mut çalışma = çalışma(
            GrafikSeçenekleri::yeni()
                .başlık(Başlık::yeni().metin("Silinecek"))
                .seri(ÇizgiSerisi::yeni().veri([1])),
        );
        çalışma.boyutlandır(800.0, 600.0, Some(2.0), false).unwrap();
        assert_eq!(çalışma.başlatma().genişlik, 800.0);
        assert!(çalışma.boyutlandır(0.0, 10.0, None, false).is_err());
        çalışma.temizle(false).unwrap();
        assert!(çalışma.seçenekleri_al().unwrap().seriler.is_empty());
        çalışma.kapat();
        çalışma.kapat();
        assert!(matches!(
            çalışma.seçenekleri_al(),
            Err(BilesenHatasi::KapatılmışÖrnek {
                işlem: "getOption"
            })
        ));
    }

    #[test]
    fn base_timeline_ve_media_birlesme_sirasi() {
        let paket = BileşikSeçenekler::yeni(
            GrafikSeçenekleri::yeni()
                .başlık(Başlık::yeni().metin("Temel"))
                .seri(ÇizgiSerisi::yeni().veri([1])),
        )
        .zaman_karesi(
            SeçenekYaması::yeni()
                .başlık(Başlık::yeni().metin("Kare"))
                .kimlikli_seri("kare", ÇizgiSerisi::yeni().veri([2])),
        )
        .medya(
            MedyaSorgusu {
                en_çok_genişlik: Some(500.0),
                ..MedyaSorgusu::default()
            },
            SeçenekYaması::yeni().animasyon(false),
        )
        .öntanımlı_medya(SeçenekYaması::yeni().koyu(true));

        let dar = paket.çöz(400.0, 300.0, Some(0)).unwrap();
        assert_eq!(
            dar.başlık.as_ref().and_then(|b| b.metin.as_deref()),
            Some("Kare")
        );
        assert!(!dar.animasyon);
        assert!(!dar.koyu, "eşleşen sorgu varken default media uygulanmaz");
        assert_eq!(dar.seriler.len(), 2, "normal merge temel seriyi korur");

        let geniş = paket.çöz(800.0, 600.0, None).unwrap();
        assert!(geniş.koyu);
        assert!(geniş.animasyon);
        assert!(paket.çöz(800.0, 600.0, Some(9)).is_err());
    }

    #[test]
    fn timeline_karesi_modelin_geçerli_sırasını_da_günceller() {
        let paket = BileşikSeçenekler::yeni(
            GrafikSeçenekleri::yeni()
                .zaman_şeridi(ZamanŞeridi::yeni().veri(["ilk", "orta", "son"]))
                .kimlikli_seri("temel", ÇizgiSerisi::yeni().veri([0])),
        )
        .zaman_karesi(SeçenekYaması::yeni().kimlikli_seri("kare", ÇizgiSerisi::yeni().veri([1])))
        .zaman_karesi(SeçenekYaması::yeni().kimlikli_seri("kare", ÇizgiSerisi::yeni().veri([2])))
        .zaman_karesi(SeçenekYaması::yeni().kimlikli_seri("kare", ÇizgiSerisi::yeni().veri([3])));

        let sonuç = paket.çöz(800.0, 600.0, Some(2)).unwrap();
        assert_eq!(
            sonuç.zaman_şeridi.as_ref().map(|şerit| şerit.geçerli_sıra),
            Some(2)
        );
        assert_eq!(sonuç.seriler.len(), 2, "timeline normal merge uygular");
        assert_eq!(ilk_değer(&sonuç, 1), 3.0);
    }

    #[test]
    fn convert_to_from_and_contain_pixel_round_trip() {
        use crate::koordinat::{Dikdörtgen, ÇalışmaEkseni};
        use crate::model::eksen::{Eksen, EksenKonumu};
        use crate::olcek::{AralıkÖlçeği, Ölçek};

        let ölçek = || {
            Ölçek::Aralık(AralıkÖlçeği::kur(
                [0.0, 100.0],
                Some(0.0),
                Some(100.0),
                false,
                5,
                None,
                None,
            ))
        };
        let koordinat = Kartezyen2B {
            x: ÇalışmaEkseni::yeni(Eksen::değer(), ölçek(), [10.0, 210.0], EksenKonumu::Alt),
            y: ÇalışmaEkseni::yeni(Eksen::değer(), ölçek(), [210.0, 10.0], EksenKonumu::Sol),
            alan: Dikdörtgen::yeni(10.0, 10.0, 200.0, 200.0),
        };
        let çalışma = çalışma(GrafikSeçenekleri::default());
        let piksel = çalışma.piksele_çevir(&koordinat, [25.0, 75.0]).unwrap();
        assert!((piksel[0] - 60.0).abs() < 1e-4);
        assert!((piksel[1] - 60.0).abs() < 1e-4);
        let geri = çalışma.pikselden_çevir(&koordinat, piksel).unwrap();
        assert!((geri[0] - 25.0).abs() < 1e-6);
        assert!((geri[1] - 75.0).abs() < 1e-6);
        assert!(çalışma.piksel_içeriyor_mu(&koordinat, piksel).unwrap());
        assert!(
            !çalışma
                .piksel_içeriyor_mu(&koordinat, [500.0, 500.0])
                .unwrap()
        );
    }
}
