//! Grafik seçenekleri — ECharts'taki kök `option` nesnesinin karşılığı.

use std::collections::BTreeMap;

use crate::animasyon::{Yumuşatma, ÖNTANIMLI_SÜRE_MS};
use crate::model::bilesen::{AraçKutusu, Başlık, Fırça, Gösterge, Izgara, İpucu};
use crate::model::eksen::{Eksen, EksenTürü};
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::grafik_bileseni::GrafikBileşeni;
use crate::model::kutupsal::KutupsalKoordinat;
use crate::model::matris::MatrisKoordinatı;
use crate::model::radar::RadarKoordinatı;
use crate::model::seri::Seri;
use crate::model::takvim::TakvimKoordinatı;
use crate::model::tek_eksen::TekEksen;
use crate::model::veri_kumesi::{
    BoyutSeçici, SeriYerleşimi, VeriKümesi, VeriKümesiTanımı, veri_kümelerini_çöz,
};
use crate::model::yakinlastirma::VeriYakınlaştırma;
use crate::model::zaman_seridi::ZamanŞeridi;
use crate::renk::{Dolgu, Renk};
use crate::tema;

/// Kök grafik seçenekleri (`EChartsOption`).
#[derive(Clone, Debug)]
pub struct GrafikSeçenekleri {
    pub başlık: Option<Başlık>,
    /// Çoklu başlık listesi (`title: []`). Boşsa geriye uyumlu tekil
    /// `başlık` alanı kullanılır.
    pub başlıklar: Vec<Başlık>,
    pub gösterge: Option<Gösterge>,
    pub ızgara: Izgara,
    /// Çoklu ızgara listesi (`grid: []`); boşsa `ızgara` tek başına
    /// kullanılır. Eksenler `ızgara_sırası` ile bağlanır.
    pub ızgaralar: Vec<Izgara>,
    pub x_ekseni: Option<Eksen>,
    pub y_ekseni: Option<Eksen>,
    /// Çoklu x eksenleri (`xAxis: []`); boşsa `x_ekseni` kullanılır.
    pub x_eksenleri: Vec<Eksen>,
    /// Çoklu y eksenleri (`yAxis: []`); boşsa `y_ekseni` kullanılır.
    pub y_eksenleri: Vec<Eksen>,
    pub seriler: Vec<Seri>,
    /// `series[i].id` karşılığı. Kimlikler seri verisinden ayrı tutulur;
    /// böylece mevcut seri kurucularının kaynak uyumluluğu bozulmadan
    /// ECharts'ın `id`/`name`/indeks birleştirme sırası uygulanabilir.
    ///
    /// Dışarıdan `seriler` doğrudan değiştirilebildiği için bu vektör kısa
    /// olabilir; eksik girişler kimliksiz kabul edilir.
    pub seri_kimlikleri: Vec<Option<String>>,
    pub ipucu: Option<İpucu>,
    /// Görsel eşleme bileşeni (`visualMap`); ısı haritası hücre renkleri
    /// buradan çözülür.
    pub görsel_eşleme: Option<GörselEşleme>,
    /// Çoklu görsel eşleme (`visualMap: []`). Tekil alan kaynak uyumluluğu
    /// için korunur; etkin sıra tekil öğe, ardından bu listedir.
    pub görsel_eşlemeler: Vec<GörselEşleme>,
    /// Radar koordinat sistemi (`radar`).
    pub radar: Option<RadarKoordinatı>,
    /// Kutupsal koordinat sistemi (`polar` + `angleAxis` + `radiusAxis`).
    pub kutupsal: Option<KutupsalKoordinat>,
    /// ECharts 6.1 `matrix` koordinat sistemi.
    pub matris: Option<MatrisKoordinatı>,
    /// Birden çok ECharts `calendar` koordinat bileşeni.
    pub takvimler: Vec<TakvimKoordinatı>,
    /// Birden çok ECharts `singleAxis` koordinat bileşeni.
    pub tek_eksenler: Vec<TekEksen>,
    /// Ortak veri tablosu (`dataset`); seriler `eşle(...)` ile beslenir.
    pub veri_kümesi: Option<VeriKümesi>,
    /// Çoklu kaynak ve built-in dönüşüm zinciri (`dataset: []`). Boşsa
    /// geriye uyumlu tekil `veri_kümesi` kullanılır.
    pub veri_kümeleri: Vec<VeriKümesiTanımı>,
    /// Veri yakınlaştırmaları (`dataZoom`).
    pub veri_yakınlaştırmaları: Vec<VeriYakınlaştırma>,
    /// Araç kutusu (`toolbox`).
    pub araç_kutusu: Option<AraçKutusu>,
    /// Fırça (`brush`): dikdörtgen seçim.
    pub fırça: Option<Fırça>,
    /// Serbest zrender öğeleri (`graphic`).
    pub grafik: Option<GrafikBileşeni>,
    /// Zaman şeridi (`timeline`). `BileşikSeçenekler`, seçilen `options`
    /// karesini bu bileşenin `geçerli_sıra` değeriyle birleştirir.
    pub zaman_şeridi: Option<ZamanŞeridi>,
    /// Seri renk paleti (`color`).
    pub palet: Vec<Renk>,
    pub arkaplan: Option<Dolgu>,
    /// Koyu tema (`theme: 'dark'` karşılığı): eksen/yazı/ipucu renkleri koyu
    /// belirteçlerden çözülür; `arkaplan` verilmemişse koyu zemin doldurulur.
    pub koyu: bool,
    /// Yerel (`locale`): ay/gün adları ve arayüz metinleri.
    pub yerel: &'static crate::yerel::Yerel,
    pub animasyon: bool,
    /// Giriş animasyonu süresi, ms (`animationDuration`).
    pub animasyon_süresi: f32,
    /// Veri güncelleme (geçiş) animasyonu süresi, ms
    /// (`animationDurationUpdate`, ECharts öntanımlısı 300).
    pub animasyon_süresi_güncelleme: f32,
    pub animasyon_eğrisi: Yumuşatma,
}

impl Default for GrafikSeçenekleri {
    fn default() -> Self {
        GrafikSeçenekleri {
            başlık: None,
            başlıklar: Vec::new(),
            gösterge: None,
            ızgara: Izgara::default(),
            ızgaralar: Vec::new(),
            x_ekseni: None,
            y_ekseni: None,
            x_eksenleri: Vec::new(),
            y_eksenleri: Vec::new(),
            seriler: Vec::new(),
            seri_kimlikleri: Vec::new(),
            ipucu: None,
            görsel_eşleme: None,
            görsel_eşlemeler: Vec::new(),
            radar: None,
            kutupsal: None,
            matris: None,
            takvimler: Vec::new(),
            tek_eksenler: Vec::new(),
            veri_kümesi: None,
            veri_kümeleri: Vec::new(),
            veri_yakınlaştırmaları: Vec::new(),
            araç_kutusu: None,
            fırça: None,
            grafik: None,
            zaman_şeridi: None,
            palet: tema::PALET.to_vec(),
            arkaplan: None,
            koyu: false,
            yerel: &crate::yerel::TÜRKÇE,
            animasyon: true,
            animasyon_süresi: ÖNTANIMLI_SÜRE_MS,
            animasyon_süresi_güncelleme: 300.0,
            animasyon_eğrisi: Yumuşatma::KübikÇıkış,
        }
    }
}

impl GrafikSeçenekleri {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn başlık(mut self, başlık: Başlık) -> Self {
        self.başlık = Some(başlık);
        self.başlıklar.clear();
        self
    }

    /// Bir başlık bileşeni ekler (`title: [...]`). Daha önce tekil başlık
    /// kurulmuşsa onu ilk öğe olarak koruyup çoklu biçime geçirir.
    pub fn başlık_ekle(mut self, başlık: Başlık) -> Self {
        if self.başlıklar.is_empty()
            && let Some(tekil) = self.başlık.take()
        {
            self.başlıklar.push(tekil);
        }
        self.başlıklar.push(başlık);
        self
    }

    pub fn gösterge(mut self, gösterge: Gösterge) -> Self {
        self.gösterge = Some(gösterge);
        self
    }

    pub fn ızgara(mut self, ızgara: Izgara) -> Self {
        self.ızgara = ızgara;
        self
    }

    pub fn x_ekseni(mut self, eksen: Eksen) -> Self {
        self.x_ekseni = Some(eksen);
        self
    }

    pub fn y_ekseni(mut self, eksen: Eksen) -> Self {
        self.y_ekseni = Some(eksen);
        self
    }

    /// Çoklu ızgara için ızgara ekler; eksenler `ızgara_sırası(i)` ile
    /// bağlanır.
    pub fn ızgara_ekle(mut self, ızgara: Izgara) -> Self {
        self.ızgaralar.push(ızgara);
        self
    }

    /// Çoklu eksen için x ekseni ekler (`xAxis: [...]`).
    pub fn x_ekseni_ekle(mut self, eksen: Eksen) -> Self {
        self.x_eksenleri.push(eksen);
        self
    }

    /// Çoklu eksen için y ekseni ekler (`yAxis: [...]`).
    pub fn y_ekseni_ekle(mut self, eksen: Eksen) -> Self {
        self.y_eksenleri.push(eksen);
        self
    }

    /// Etkin ızgara listesi: `ızgaralar` boşsa tekil `ızgara`.
    pub fn etkin_ızgaralar(&self) -> Vec<Izgara> {
        if self.ızgaralar.is_empty() {
            vec![self.ızgara.clone()]
        } else {
            self.ızgaralar.clone()
        }
    }

    /// Etkin x eksenleri: `x_eksenleri` boşsa tekil `x_ekseni` (o da yoksa
    /// öntanımlı kategori).
    pub fn etkin_x_eksenleri(&self) -> Vec<Eksen> {
        if self.x_eksenleri.is_empty() {
            vec![self.x_ekseni.clone().unwrap_or_else(Eksen::kategori)]
        } else {
            self.x_eksenleri.clone()
        }
    }

    /// Etkin y eksenleri: `y_eksenleri` boşsa tekil `y_ekseni` (o da yoksa
    /// öntanımlı değer ekseni).
    pub fn etkin_y_eksenleri(&self) -> Vec<Eksen> {
        if self.y_eksenleri.is_empty() {
            vec![self.y_ekseni.clone().unwrap_or_else(Eksen::değer)]
        } else {
            self.y_eksenleri.clone()
        }
    }

    pub fn seri(mut self, seri: impl Into<Seri>) -> Self {
        self.seriler.push(seri.into());
        self.seri_kimlikleri.push(None);
        self
    }

    /// Açık `series.id` ile seri ekler. `id`, normal option birleştirmesinde
    /// `name` ve dizi sırasından önce kullanılır.
    pub fn kimlikli_seri(mut self, kimlik: impl Into<String>, seri: impl Into<Seri>) -> Self {
        self.seriler.push(seri.into());
        self.seri_kimlikleri.push(Some(kimlik.into()));
        self
    }

    pub fn seriler<S: Into<Seri>>(mut self, seriler: impl IntoIterator<Item = S>) -> Self {
        for seri in seriler {
            self.seriler.push(seri.into());
            self.seri_kimlikleri.push(None);
        }
        self
    }

    /// Bir serinin açık kimliğini döndürür. Boş dize kimlik sayılmaz.
    pub fn seri_kimliği(&self, sıra: usize) -> Option<&str> {
        self.seri_kimlikleri
            .get(sıra)
            .and_then(Option::as_deref)
            .filter(|kimlik| !kimlik.is_empty())
    }

    pub fn ipucu(mut self, ipucu: İpucu) -> Self {
        self.ipucu = Some(ipucu);
        self
    }

    pub fn görsel_eşleme(mut self, eşleme: GörselEşleme) -> Self {
        self.görsel_eşleme = Some(eşleme);
        self.görsel_eşlemeler.clear();
        self
    }

    /// `visualMap: []` listesini doğrudan ayarlar. Tekil geriye uyumluluk
    /// alanı temizlenir; böylece seçenek tam olarak verilen diziyi temsil eder.
    pub fn görsel_eşlemeler(
        mut self, eşlemeler: impl IntoIterator<Item = GörselEşleme>
    ) -> Self {
        self.görsel_eşleme = None;
        self.görsel_eşlemeler = eşlemeler.into_iter().collect();
        self
    }

    /// Var olan görsel eşleme sırasının sonuna bir bileşen ekler.
    pub fn görsel_eşleme_ekle(mut self, eşleme: GörselEşleme) -> Self {
        self.görsel_eşlemeler.push(eşleme);
        self
    }

    /// Tekil ve çoğul alanları ECharts bileşen sırasıyla dolaşır.
    pub fn tüm_görsel_eşlemeler(&self) -> impl DoubleEndedIterator<Item = &GörselEşleme> {
        self.görsel_eşleme
            .iter()
            .chain(self.görsel_eşlemeler.iter())
    }

    /// Seriyi hedefleyen son görsel eşleme. Birden çok renk eşlemesinde
    /// ECharts'ın son görsel meta girdisini tercih etme davranışını izler.
    pub fn seri_görsel_eşlemesi(&self, seri_sırası: usize) -> Option<&GörselEşleme> {
        self.tüm_görsel_eşlemeler()
            .rev()
            .find(|eşleme| eşleme.seriye_uygulanır_mı(seri_sırası))
    }

    /// Seriyi hedefleyen bütün `visualMap` bileşenlerini seçenek sırasıyla
    /// dolaşır. Scatter gibi bağımsız görsel kanalları birleştiren seriler,
    /// tekil geriye uyumlu seçici yerine bu görünümü kullanır.
    pub fn seri_görsel_eşlemeleri(
        &self,
        seri_sırası: usize,
    ) -> impl Iterator<Item = &GörselEşleme> {
        self.tüm_görsel_eşlemeler()
            .filter(move |eşleme| eşleme.seriye_uygulanır_mı(seri_sırası))
    }

    pub fn radar(mut self, koordinat: RadarKoordinatı) -> Self {
        self.radar = Some(koordinat);
        self
    }

    pub fn kutupsal(mut self, koordinat: KutupsalKoordinat) -> Self {
        self.kutupsal = Some(koordinat);
        self
    }

    pub fn matris(mut self, koordinat: MatrisKoordinatı) -> Self {
        self.matris = Some(koordinat);
        self
    }

    pub fn takvim(mut self, koordinat: TakvimKoordinatı) -> Self {
        self.takvimler.push(koordinat);
        self
    }

    /// `singleAxis: []` dizisine tek eksenli koordinat ekler.
    pub fn tek_eksen(mut self, koordinat: TekEksen) -> Self {
        self.tek_eksenler.push(koordinat);
        self
    }

    pub fn zaman_şeridi(mut self, zaman_şeridi: ZamanŞeridi) -> Self {
        self.zaman_şeridi = Some(zaman_şeridi);
        self
    }

    pub fn grafik(mut self, grafik: GrafikBileşeni) -> Self {
        self.grafik = Some(grafik);
        self
    }

    pub fn veri_kümesi(mut self, küme: VeriKümesi) -> Self {
        self.veri_kümesi = Some(küme);
        self.veri_kümeleri.clear();
        self
    }

    /// `dataset: []` dizisine kaynak veya dönüşüm girdisi ekler.
    pub fn veri_kümesi_ekle(mut self, tanım: impl Into<VeriKümesiTanımı>) -> Self {
        if self.veri_kümeleri.is_empty() {
            self.veri_kümesi = None;
        }
        self.veri_kümeleri.push(tanım.into());
        self
    }

    pub fn veri_kümeleri(
        mut self, tanımlar: impl IntoIterator<Item = VeriKümesiTanımı>
    ) -> Self {
        self.veri_kümesi = None;
        self.veri_kümeleri = tanımlar.into_iter().collect();
        self
    }

    /// Veri kümesine bağlı serilerin verilerini tablodan türetir
    /// (`encode` çözümü). Eşleme çözülemezse seri olduğu gibi kalır ve
    /// hata listelenir.
    pub fn veri_kümesini_uygula(&self) -> (Self, Vec<crate::hata::BilesenHatasi>) {
        let (kümeler, mut hatalar) = if self.veri_kümeleri.is_empty() {
            match &self.veri_kümesi {
                Some(küme) => (vec![küme.clone()], Vec::new()),
                None => return (self.clone(), Vec::new()),
            }
        } else {
            match veri_kümelerini_çöz(&self.veri_kümeleri) {
                Ok(kümeler) => (kümeler, Vec::new()),
                Err(hata) => return (self.clone(), vec![hata]),
            }
        };
        let mut sonuç = self.clone();
        let x_eksenleri = self.etkin_x_eksenleri();
        // Açık `encode` yoksa ECharts aynı dataset/layout grubundaki boş
        // serilere kategori boyutundan sonraki değer boyutlarını sırayla
        // tahsis eder. Satır ve sütun görünümleri birbirinden bağımsızdır.
        let mut otomatik_sıralar: BTreeMap<(usize, SeriYerleşimi), usize> = BTreeMap::new();
        for (seri_sırası, seri) in sonuç.seriler.iter_mut().enumerate() {
            let (açık_eşleme, küme_sırası, yerleşim, veri_boş, sayısal_x) = match &*seri {
                Seri::Çizgi(s) => (
                    s.eşleme.clone(),
                    s.veri_kümesi_sırası,
                    s.seri_yerleşimi,
                    s.veri.is_empty(),
                    x_eksenleri
                        .get(s.eksen_bağı.x)
                        .is_some_and(|eksen| eksen.tür != EksenTürü::Kategori),
                ),
                Seri::Sütun(s) => (
                    s.eşleme.clone(),
                    s.veri_kümesi_sırası,
                    s.seri_yerleşimi,
                    s.veri.is_empty(),
                    false,
                ),
                Seri::Saçılım(s) => (
                    s.eşleme.clone(),
                    s.veri_kümesi_sırası,
                    s.seri_yerleşimi,
                    s.veri_boş_mu(),
                    x_eksenleri
                        .get(s.eksen_bağı.x)
                        .is_some_and(|eksen| eksen.tür != EksenTürü::Kategori),
                ),
                Seri::Pasta(s) => (
                    s.eşleme.clone(),
                    s.veri_kümesi_sırası,
                    s.seri_yerleşimi,
                    s.veri.is_empty(),
                    false,
                ),
                _ => continue,
            };
            // series.data, dataset'ten daha yüksek önceliklidir.
            if !veri_boş {
                continue;
            }
            let Some(taban_küme) = kümeler.get(küme_sırası) else {
                hatalar.push(crate::hata::BilesenHatasi::EksikVeri {
                    bileşen: "dataset",
                    sıra: küme_sırası,
                });
                continue;
            };
            let küme = taban_küme.seri_yerleşimiyle(yerleşim);
            // ECharts yalnız otomatik encode ürettiğinde `seriesName`
            // boyutunu da otomatik doldurur. Kullanıcı açık `encode.x/y`
            // verdiğinde seri adı boş kalır (legend öğesi oluşmaz).
            let otomatik_eşleme = açık_eşleme.is_none();
            let (ad_boyutu, değer_boyutu) = match açık_eşleme {
                Some(eşleme) => eşleme,
                None => {
                    let değer_sırası = otomatik_sıralar
                        .entry((küme_sırası, yerleşim))
                        .and_modify(|sıra| *sıra += 1)
                        .or_insert(0);
                    let ad_boyutu = küme.boyutlar.first().cloned();
                    let değer_boyutu = küme.boyutlar.get(*değer_sırası + 1).cloned();
                    let (Some(ad_boyutu), Some(değer_boyutu)) = (ad_boyutu, değer_boyutu) else {
                        hatalar.push(crate::hata::BilesenHatasi::GeçersizSeçenek {
                            alan: "dataset.encode",
                            ayrıntı: format!(
                                "dataset[{küme_sırası}] içinde otomatik seri için yeterli boyut yok"
                            ),
                        });
                        continue;
                    };
                    (ad_boyutu, değer_boyutu)
                }
            };
            let adlar = match küme.metinler(&ad_boyutu) {
                Ok(a) => a,
                Err(hata) => {
                    hatalar.push(hata);
                    continue;
                }
            };
            let değerler = match küme.sayılar(&değer_boyutu) {
                Ok(d) => d,
                Err(hata) => {
                    hatalar.push(hata);
                    continue;
                }
            };
            let görsel_eşleme = self
                .seri_görsel_eşlemesi(seri_sırası)
                .filter(|eşleme| eşleme.boyut.is_some());
            let görsel_boyut_sırası = görsel_eşleme
                .and_then(|eşleme| eşleme.boyut.as_ref())
                .and_then(|boyut| match boyut {
                    BoyutSeçici::Sıra(sıra) => (*sıra < küme.boyutlar.len()).then_some(*sıra),
                    BoyutSeçici::Ad(ad) => küme.boyut_sırası(ad),
                });
            let görsel_kapsam = görsel_eşleme.and_then(|eşleme| {
                görsel_boyut_sırası.map(|boyut_sırası| {
                    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
                    for değer in küme
                        .satırlar
                        .iter()
                        .filter_map(|satır| satır.get(boyut_sırası))
                        .filter_map(crate::model::deger::VeriDeğeri::sayı)
                        .filter(|değer| değer.is_finite())
                    {
                        kapsam[0] = kapsam[0].min(değer);
                        kapsam[1] = kapsam[1].max(değer);
                    }
                    if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
                        kapsam = [0.0, 1.0];
                    }
                    eşleme.kapsam_çöz(kapsam)
                })
            });
            let ad_boyut_sırası = küme.boyut_sırası(&ad_boyutu);
            let veri: Vec<crate::model::deger::VeriÖğesi> = adlar
                .iter()
                .zip(değerler.iter())
                .enumerate()
                .map(|(satır_sırası, (ad, değer))| {
                    let kaynak_satır = küme.satırlar.get(satır_sırası);
                    let boyutlar: Vec<_> = küme
                        .boyutlar
                        .iter()
                        .enumerate()
                        .map(|(boyut_sırası, boyut_adı)| {
                            (
                                boyut_adı.clone(),
                                kaynak_satır
                                    .and_then(|satır| satır.get(boyut_sırası))
                                    .cloned()
                                    .unwrap_or(crate::model::deger::VeriDeğeri::Boş),
                            )
                        })
                        .collect();
                    let koordinat_değeri = if sayısal_x {
                        ad_boyut_sırası
                            .and_then(|boyut_sırası| kaynak_satır?.get(boyut_sırası))
                            .and_then(crate::model::deger::VeriDeğeri::sayı)
                            .map(|x| crate::model::deger::VeriDeğeri::Çift([x, *değer]))
                            .unwrap_or_else(|| (*değer).into())
                    } else {
                        (*değer).into()
                    };
                    let mut öğe =
                        crate::model::deger::VeriÖğesi::adlı(ad.clone(), koordinat_değeri)
                            .boyutlar(boyutlar);
                    if let (Some(eşleme), Some(boyut_sırası), Some(kapsam)) =
                        (görsel_eşleme, görsel_boyut_sırası, görsel_kapsam)
                        && eşleme.renk_kanalı
                        && let Some(görsel_değer) = kaynak_satır
                            .and_then(|satır| satır.get(boyut_sırası))
                            .and_then(crate::model::deger::VeriDeğeri::sayı)
                    {
                        öğe.stil = Some(
                            crate::model::stil::ÖğeStili::yeni()
                                .renk(eşleme.renk_çöz(görsel_değer, kapsam)),
                        );
                    }
                    öğe
                })
                .collect();
            match seri {
                Seri::Çizgi(s) => {
                    s.veri = veri;
                    if otomatik_eşleme && s.ad.is_none() {
                        s.ad = Some(değer_boyutu.clone());
                    }
                }
                Seri::Sütun(s) => {
                    s.veri = veri;
                    if otomatik_eşleme && s.ad.is_none() {
                        s.ad = Some(değer_boyutu.clone());
                    }
                }
                Seri::Saçılım(s) => {
                    s.veri = veri;
                    s.düz_veri = None;
                    if otomatik_eşleme && s.ad.is_none() {
                        s.ad = Some(değer_boyutu.clone());
                    }
                }
                Seri::Pasta(s) => {
                    s.veri = veri;
                    if otomatik_eşleme && s.ad.is_none() {
                        s.ad = Some(değer_boyutu.clone());
                    }
                }
                _ => {}
            }
        }
        (sonuç, hatalar)
    }

    /// Veri yakınlaştırma ekler (`dataZoom`).
    pub fn veri_yakınlaştırma(mut self, yakınlaştırma: VeriYakınlaştırma) -> Self {
        self.veri_yakınlaştırmaları.push(yakınlaştırma);
        self
    }

    pub fn araç_kutusu(mut self, araçlar: AraçKutusu) -> Self {
        self.araç_kutusu = Some(araçlar);
        self
    }

    pub fn fırça(mut self, fırça: Fırça) -> Self {
        self.fırça = Some(fırça);
        self
    }

    /// Verilen x eksenine bağlı etkin pencere oranları `0..=1`.
    pub fn x_penceresi(&self, eksen_sırası: usize) -> Option<(f32, f32)> {
        self.x_yakınlaştırması(eksen_sırası).map(|y| y.oranlar())
    }

    /// Verilen y eksenine bağlı etkin pencere oranları `0..=1`.
    pub fn y_penceresi(&self, eksen_sırası: usize) -> Option<(f32, f32)> {
        self.y_yakınlaştırması(eksen_sırası).map(|y| y.oranlar())
    }

    /// Verilen x eksenini yöneten ilk dataZoom bileşeni.
    pub fn x_yakınlaştırması(&self, eksen_sırası: usize) -> Option<&VeriYakınlaştırma> {
        self.veri_yakınlaştırmaları
            .iter()
            .find(|yakınlaştırma| yakınlaştırma.x_eksenini_hedefler(eksen_sırası))
    }

    /// Verilen y eksenini yöneten ilk dataZoom bileşeni.
    pub fn y_yakınlaştırması(&self, eksen_sırası: usize) -> Option<&VeriYakınlaştırma> {
        self.veri_yakınlaştırmaları
            .iter()
            .find(|yakınlaştırma| yakınlaştırma.y_eksenini_hedefler(eksen_sırası))
    }

    pub fn palet<R: Into<Renk>>(mut self, renkler: impl IntoIterator<Item = R>) -> Self {
        self.palet = renkler.into_iter().map(Into::into).collect();
        self
    }

    pub fn arkaplan(mut self, dolgu: impl Into<Dolgu>) -> Self {
        self.arkaplan = Some(dolgu.into());
        self
    }

    /// Koyu temayı açar/kapatır (`theme: 'dark'`).
    pub fn koyu(mut self, açık: bool) -> Self {
        self.koyu = açık;
        self
    }

    /// Yereli seçer (`locale`).
    pub fn yerel(mut self, yerel: &'static crate::yerel::Yerel) -> Self {
        self.yerel = yerel;
        self
    }

    pub fn animasyon(mut self, açık: bool) -> Self {
        self.animasyon = açık;
        self
    }

    pub fn animasyon_süresi(mut self, ms: f32) -> Self {
        self.animasyon_süresi = ms;
        self
    }

    pub fn animasyon_süresi_güncelleme(mut self, ms: f32) -> Self {
        self.animasyon_süresi_güncelleme = ms;
        self
    }

    pub fn animasyon_eğrisi(mut self, eğri: Yumuşatma) -> Self {
        self.animasyon_eğrisi = eğri;
        self
    }

    /// Serinin paletten çözülen rengi (`itemStyle.color` öncelikli).
    pub fn seri_rengi(&self, sıra: usize) -> Renk {
        if let Some(renk) = self.seriler.get(sıra).and_then(|seri| seri.açık_renk()) {
            return renk.temsilî();
        }
        // PaletteTask, açık bir `itemStyle.color` taşıyan seriyi paletten
        // boyamaz ve bu seri palette bir sıra da tüketmez. Waterfall'daki
        // saydam placeholder'dan sonraki görünür seri bu nedenle ilk rengi
        // alır.
        let palet_sırası = self
            .seriler
            .iter()
            .take(sıra.saturating_add(1))
            .filter(|seri| seri.açık_renk().is_none())
            .count()
            .saturating_sub(1);
        self.palet_rengi(palet_sırası)
    }

    /// Paletten sıra numarasıyla renk (pasta dilimleri gibi öğe-bazlı
    /// renklendirme için).
    pub fn palet_rengi(&self, sıra: usize) -> Renk {
        if self.palet.is_empty() {
            return tema::palet_rengi(sıra);
        }
        self.palet
            .get(sıra % self.palet.len())
            .copied()
            .unwrap_or_else(|| tema::palet_rengi(sıra))
    }

    /// Seçenekleri doğrular; ilk sorun [`BilesenHatasi`] olarak döner.
    /// `GrafikGörünümü::seçenekleri_değiştir`, hata durumunda işlemi geri
    /// alır (mevcut seçenekler korunur).
    pub fn doğrula(&self) -> Result<(), crate::hata::BilesenHatasi> {
        use crate::hata::BilesenHatasi;

        if !self.animasyon_süresi.is_finite() || self.animasyon_süresi < 0.0 {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "animasyon_süresi",
                ayrıntı: format!("{} geçerli bir süre değil", self.animasyon_süresi),
            });
        }
        if !self.animasyon_süresi_güncelleme.is_finite() || self.animasyon_süresi_güncelleme < 0.0
        {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "animasyon_süresi_güncelleme",
                ayrıntı: format!(
                    "{} geçerli bir süre değil",
                    self.animasyon_süresi_güncelleme
                ),
            });
        }
        if let Some(grafik) = &self.grafik {
            grafik.doğrula()?;
        }
        if let Some(zaman_şeridi) = &self.zaman_şeridi {
            if !zaman_şeridi.oynatma_aralığı.is_finite() || zaman_şeridi.oynatma_aralığı < 0.0
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "timeline.playInterval",
                    ayrıntı: format!(
                        "{} geçerli bir oynatma aralığı değil",
                        zaman_şeridi.oynatma_aralığı
                    ),
                });
            }
            if !zaman_şeridi.veri.is_empty()
                && zaman_şeridi.geçerli_sıra >= zaman_şeridi.veri.len()
                && !zaman_şeridi.döngü
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "timeline.currentIndex",
                    ayrıntı: format!(
                        "{} sırası {} zaman verisinin dışında",
                        zaman_şeridi.geçerli_sıra,
                        zaman_şeridi.veri.len()
                    ),
                });
            }
        }
        let x_eksenler = self.etkin_x_eksenleri();
        let y_eksenler = self.etkin_y_eksenleri();
        let x_eksen_adedi = x_eksenler.len();
        let y_eksen_adedi = y_eksenler.len();
        for (sıra, yakınlaştırma) in self.veri_yakınlaştırmaları.iter().enumerate() {
            if !yakınlaştırma.başlangıç.is_finite()
                || !yakınlaştırma.bitiş.is_finite()
                || !(0.0..=100.0).contains(&yakınlaştırma.başlangıç)
                || !(0.0..=100.0).contains(&yakınlaştırma.bitiş)
                || yakınlaştırma.başlangıç > yakınlaştırma.bitiş
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "dataZoom.start/end",
                    ayrıntı: format!(
                        "{sıra}. bileşenin aralığı 0..=100 içinde ve start <= end olmalı ({}..{})",
                        yakınlaştırma.başlangıç, yakınlaştırma.bitiş
                    ),
                });
            }
            for (alan, değer) in [
                ("dataZoom.startValue", &yakınlaştırma.başlangıç_değeri),
                ("dataZoom.endValue", &yakınlaştırma.bitiş_değeri),
            ] {
                if let Some(crate::model::yakinlastirma::YakınlaştırmaDeğeri::Sayı(değer)) = değer
                    && !değer.is_finite()
                {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan,
                        ayrıntı: format!("{sıra}. bileşenin değer ucu sonlu olmalı ({değer})"),
                    });
                }
            }
            let (bileşen, eksen_adedi, hedefler): (&str, usize, Vec<usize>) =
                if yakınlaştırma.y_eksen_sırası.is_some() {
                    (
                        "yAxis",
                        y_eksen_adedi,
                        yakınlaştırma.hedef_y_eksenleri().collect(),
                    )
                } else {
                    (
                        "xAxis",
                        x_eksen_adedi,
                        yakınlaştırma.hedef_x_eksenleri().collect(),
                    )
                };
            for eksen_sırası in hedefler {
                if eksen_sırası >= eksen_adedi {
                    return Err(BilesenHatasi::EksikVeri {
                        bileşen,
                        sıra: eksen_sırası,
                    });
                }
            }
        }
        for eksen in x_eksenler
            .iter()
            .chain(y_eksenler.iter())
            .chain(self.tek_eksenler.iter().map(|tek| &tek.eksen))
        {
            if let (Some(en_az), Some(en_çok)) = (eksen.en_az, eksen.en_çok)
                && en_az >= en_çok
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "eksen.en_az/en_çok",
                    ayrıntı: format!("en_az ({en_az}) < en_çok ({en_çok}) olmalı"),
                });
            }
            if eksen.tür == crate::model::eksen::EksenTürü::Log && eksen.log_tabanı <= 1.0 {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "eksen.log_tabanı",
                    ayrıntı: format!("log tabanı 1'den büyük olmalı ({})", eksen.log_tabanı),
                });
            }
            if !eksen.kırılmalar.is_empty()
                && eksen.tür == crate::model::eksen::EksenTürü::Kategori
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "axis.breaks",
                    ayrıntı: "kategori ekseninde kırılma desteklenmez".to_owned(),
                });
            }
            let mut aralıklar = Vec::with_capacity(eksen.kırılmalar.len());
            for (sıra, kırılma) in eksen.kırılmalar.iter().enumerate() {
                if !kırılma.başlangıç.is_finite() || !kırılma.bitiş.is_finite() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "axis.breaks.start/end",
                        ayrıntı: format!("{sıra}. kırılmanın uçları sonlu olmalı"),
                    });
                }
                let boşluk_geçerli = match kırılma.boşluk {
                    crate::model::eksen::EksenKırılmaBoşluğu::Değer(değer) => {
                        değer.is_finite() && değer >= 0.0
                    }
                    crate::model::eksen::EksenKırılmaBoşluğu::Yüzde(oran) => {
                        oran.is_finite() && (0.0..(1.0 - 1e-5)).contains(&oran)
                    }
                };
                if !boşluk_geçerli {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "axis.breaks.gap",
                        ayrıntı: format!("{sıra}. kırılmanın boşluğu geçerli değil"),
                    });
                }
                aralıklar.push((
                    kırılma.başlangıç.min(kırılma.bitiş),
                    kırılma.başlangıç.max(kırılma.bitiş),
                ));
            }
            aralıklar.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            if aralıklar
                .windows(2)
                .any(|çift| matches!(çift, [ilk, ikinci] if ilk.1 > ikinci.0))
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "axis.breaks",
                    ayrıntı: "kırılma aralıkları çakışmamalı".to_owned(),
                });
            }
        }
        for seri in &self.seriler {
            if let Seri::Saçılım(saçılım) = seri
                && let Some(tek_eksen_sırası) = saçılım.tek_eksen_sırası
                && self.tek_eksenler.get(tek_eksen_sırası).is_none()
            {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "singleAxis",
                    sıra: tek_eksen_sırası,
                });
            }
            if let Seri::Saçılım(saçılım) = seri
                && let Some(takvim_sırası) = saçılım.takvim_sırası
                && self.takvimler.get(takvim_sırası).is_none()
            {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "calendar",
                    sıra: takvim_sırası,
                });
            }
            if let Seri::Grafo(grafo) = seri
                && let Some(takvim_sırası) = grafo.takvim_sırası
                && self.takvimler.get(takvim_sırası).is_none()
            {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "calendar",
                    sıra: takvim_sırası,
                });
            }
            if let Seri::Özel(özel) = seri
                && let Some(takvim_sırası) = özel.takvim_sırası
                && self.takvimler.get(takvim_sırası).is_none()
            {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "calendar",
                    sıra: takvim_sırası,
                });
            }
            if let Seri::Pasta(p) = seri {
                if let Some(takvim_sırası) = p.takvim_sırası {
                    if self.takvimler.get(takvim_sırası).is_none() {
                        return Err(BilesenHatasi::EksikVeri {
                            bileşen: "calendar",
                            sıra: takvim_sırası,
                        });
                    }
                    if !p.takvim_merkez_tarihi.is_some_and(f64::is_finite) {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.pie.center",
                            ayrıntı: "takvim pastasının merkezi sonlu bir tarih olmalı"
                                .to_owned(),
                        });
                    }
                }
                let açılar = [
                    ("series.pie.startAngle", Some(p.başlangıç_açısı)),
                    ("series.pie.endAngle", p.bitiş_açısı),
                    ("series.pie.padAngle", Some(p.dolgu_açısı)),
                    ("series.pie.minAngle", Some(p.en_küçük_açı)),
                    (
                        "series.pie.minShowLabelAngle",
                        Some(p.en_küçük_etiket_açısı),
                    ),
                    ("series.pie.selectedOffset", Some(p.seçili_uzaklığı)),
                ];
                for (alan, değer) in açılar {
                    if let Some(değer) = değer
                        && (!değer.is_finite()
                            || (alan != "series.pie.startAngle"
                                && alan != "series.pie.endAngle"
                                && değer < 0.0))
                    {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan,
                            ayrıntı: format!("{değer} geçerli bir açı/uzaklık değil"),
                        });
                    }
                }
                if p.yüzde_hassasiyeti > 20 {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.pie.percentPrecision",
                        ayrıntı: "yüzde hassasiyeti 0..=20 aralığında olmalı".to_owned(),
                    });
                }
            }
            if let Seri::Hatlar(hatlar) = seri {
                match hatlar.koordinat_sistemi {
                    crate::model::hatlar::HatKoordinatSistemi::Kutupsal
                        if self.kutupsal.is_none() =>
                    {
                        return Err(BilesenHatasi::EksikVeri {
                            bileşen: "polar",
                            sıra: 0,
                        });
                    }
                    crate::model::hatlar::HatKoordinatSistemi::Takvim
                        if self.takvimler.get(hatlar.takvim_sırası).is_none() =>
                    {
                        return Err(BilesenHatasi::EksikVeri {
                            bileşen: "calendar",
                            sıra: hatlar.takvim_sırası,
                        });
                    }
                    crate::model::hatlar::HatKoordinatSistemi::Matris
                        if hatlar.matris_sırası != 0 || self.matris.is_none() =>
                    {
                        return Err(BilesenHatasi::EksikVeri {
                            bileşen: "matrix",
                            sıra: hatlar.matris_sırası,
                        });
                    }
                    crate::model::hatlar::HatKoordinatSistemi::Kartezyen2B
                    | crate::model::hatlar::HatKoordinatSistemi::Kutupsal
                    | crate::model::hatlar::HatKoordinatSistemi::Takvim
                    | crate::model::hatlar::HatKoordinatSistemi::Matris => {}
                }
                for (sıra, veri) in hatlar.veri.iter().enumerate() {
                    if veri.koordinatlar.len() < 2 {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.lines.data.coords",
                            ayrıntı: format!("{sıra}. hat en az iki koordinat taşımalıdır"),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Veri geçiş animasyonu için iki seçenek arasında ara değer üretir:
    /// aynı sıradaki serilerin eşit uzunluktaki sayısal verileri doğrusal
    /// olarak karıştırılır; eşleşmeyen her şey `yeni`den alınır (ECharts
    /// güncelleme animasyonunun sade karşılığı).
    pub fn ara_değerle(eski: &Self, yeni: &Self, t: f32) -> Self {
        use crate::model::deger::{VeriDeğeri, VeriÖğesi};
        let t = t.clamp(0.0, 1.0) as f64;
        let mut sonuç = yeni.clone();

        let öğe_karıştır = |e: &VeriÖğesi, y: &VeriÖğesi| -> VeriÖğesi {
            let değer = match (&e.değer, &y.değer) {
                (VeriDeğeri::Sayı(a), VeriDeğeri::Sayı(b)) => {
                    VeriDeğeri::Sayı(a + (b - a) * t)
                }
                (VeriDeğeri::Çift([ax, ay]), VeriDeğeri::Çift([bx, by])) => {
                    VeriDeğeri::Çift([ax + (bx - ax) * t, ay + (by - ay) * t])
                }
                (VeriDeğeri::Dizi(a), VeriDeğeri::Dizi(b)) if a.len() == b.len() => {
                    VeriDeğeri::Dizi(
                        a.iter()
                            .zip(b.iter())
                            .map(|(av, bv)| av + (bv - av) * t)
                            .collect(),
                    )
                }
                _ => y.değer.clone(),
            };
            VeriÖğesi {
                değer, ..y.clone()
            }
        };

        for (sıra, seri) in sonuç.seriler.iter_mut().enumerate() {
            let Some(eski_seri) = eski.seriler.get(sıra) else {
                continue;
            };
            if let (Seri::Hatlar(yeni_hatlar), Seri::Hatlar(eski_hatlar)) = (&mut *seri, eski_seri)
            {
                if yeni_hatlar.veri.len() == eski_hatlar.veri.len() {
                    for (eski_hat, yeni_hat) in eski_hatlar.veri.iter().zip(&mut yeni_hatlar.veri) {
                        if eski_hat.koordinatlar.len() != yeni_hat.koordinatlar.len() {
                            continue;
                        }
                        for (eski_nokta, yeni_nokta) in
                            eski_hat.koordinatlar.iter().zip(&mut yeni_hat.koordinatlar)
                        {
                            if let (Some(ex), Some(ey), Some(yx), Some(yy)) = (
                                eski_nokta.x.sayı(),
                                eski_nokta.y.sayı(),
                                yeni_nokta.x.sayı(),
                                yeni_nokta.y.sayı(),
                            ) {
                                yeni_nokta.x =
                                    crate::model::hatlar::HatKoordinatı::Sayı(ex + (yx - ex) * t);
                                yeni_nokta.y =
                                    crate::model::hatlar::HatKoordinatı::Sayı(ey + (yy - ey) * t);
                            }
                        }
                    }
                }
                continue;
            }
            let eski_veri = eski_seri.veri();
            let karıştır = |yeni_veri: &mut Vec<VeriÖğesi>| {
                if eski_veri.len() != yeni_veri.len() {
                    return;
                }
                for (e, y) in eski_veri.iter().zip(yeni_veri.iter_mut()) {
                    *y = öğe_karıştır(e, y);
                }
            };
            match seri {
                Seri::Çizgi(s) => karıştır(&mut s.veri),
                Seri::Sütun(s) => karıştır(&mut s.veri),
                Seri::Pasta(s) => karıştır(&mut s.veri),
                Seri::Saçılım(s) => karıştır(&mut s.veri),
                Seri::Mum(s) => karıştır(&mut s.veri),
                Seri::Kutu(s) => karıştır(&mut s.veri),
                Seri::Isı(s) => karıştır(&mut s.veri),
                Seri::Huni(s) => karıştır(&mut s.veri),
                Seri::GöstergeSaati(s) => karıştır(&mut s.veri),
                Seri::Radar(s) => karıştır(&mut s.veri),
                Seri::Özel(s) => karıştır(&mut s.veri),
                Seri::AğaçHaritası(_)
                | Seri::GüneşPatlaması(_)
                | Seri::Ağaç(_)
                | Seri::Sankey(_)
                | Seri::Grafo(_)
                | Seri::Kiriş(_)
                | Seri::TemaNehri(_) => {}
                Seri::Paralel(s) => karıştır(&mut s.veri),
                Seri::Takvim(s) => karıştır(&mut s.veri),
                Seri::Hatlar(_) => {}
            }
        }
        sonuç
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
    use crate::model::deger::VeriDeğeri;
    use crate::model::eksen::EksenKırılması;
    use crate::model::seri::{SaçılımSerisi, SütunSerisi, ÇizgiSerisi};
    use crate::model::stil::ÖğeStili;
    use crate::renk::{Dolgu, Renk};

    #[test]
    fn ara_değerleme() {
        let eski = GrafikSeçenekleri::yeni().seri(ÇizgiSerisi::yeni().veri([0.0, 10.0]));
        let yeni = GrafikSeçenekleri::yeni().seri(ÇizgiSerisi::yeni().veri([10.0, 30.0]));
        let ara = GrafikSeçenekleri::ara_değerle(&eski, &yeni, 0.5);
        let değerler: Vec<f64> = ara.seriler[0]
            .veri()
            .iter()
            .filter_map(|ö| ö.değer.sayı())
            .collect();
        assert_eq!(değerler, vec![5.0, 20.0]);
    }

    #[test]
    fn uzunluk_uyuşmazlığında_yeniye_atlar() {
        let eski = GrafikSeçenekleri::yeni().seri(ÇizgiSerisi::yeni().veri([1.0]));
        let yeni = GrafikSeçenekleri::yeni().seri(ÇizgiSerisi::yeni().veri([4.0, 8.0]));
        let ara = GrafikSeçenekleri::ara_değerle(&eski, &yeni, 0.5);
        let değerler: Vec<f64> = ara.seriler[0]
            .veri()
            .iter()
            .filter_map(|ö| ö.değer.sayı())
            .collect();
        assert_eq!(değerler, vec![4.0, 8.0]);
    }

    #[test]
    fn açık_renkli_seri_palet_sırası_tüketmez() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .seri(
                SütunSerisi::yeni()
                    .öğe_stili(ÖğeStili::yeni().renk(Renk::SAYDAM))
                    .veri([1]),
            )
            .seri(SütunSerisi::yeni().veri([2]));
        assert_eq!(seçenekler.seri_rengi(0), Renk::SAYDAM);
        assert_eq!(seçenekler.seri_rengi(1), seçenekler.palet_rengi(0));
    }

    #[test]
    fn seri_dataset_index_ile_dönüşüm_sonucunu_kullanır() {
        use crate::model::veri_kumesi::{SıralamaAnahtarı, VeriKümesi, VeriKümesiTanımı};

        let kaynak = VeriKümesi::yeni(["ad", "değer"]).kayıtlar([
            ("A", vec![2.0]),
            ("B", vec![5.0]),
            ("C", vec![1.0]),
        ]);
        let seçenekler = GrafikSeçenekleri::yeni()
            .veri_kümesi_ekle(VeriKümesiTanımı::kaynak(kaynak))
            .veri_kümesi_ekle(VeriKümesiTanımı::sırala([SıralamaAnahtarı::azalan(
                "değer",
            )]))
            .seri(
                SütunSerisi::yeni()
                    .veri_kümesi_sırası(1)
                    .eşle("ad", "değer"),
            );
        let (çözülmüş, hatalar) = seçenekler.veri_kümesini_uygula();
        assert!(hatalar.is_empty());
        let adlar: Vec<_> = çözülmüş.seriler[0]
            .veri()
            .iter()
            .filter_map(|öğe| öğe.ad.as_deref())
            .collect();
        assert_eq!(adlar, vec!["B", "A", "C"]);
    }

    #[test]
    fn dataset_deger_eksenindeki_cizginin_x_koordinatini_korur() {
        let küme = VeriKümesi::yeni(["x", "y"])
            .satır([1.0.into(), 10.0.into()])
            .satır([3.0.into(), 30.0.into()]);
        let seçenekler = GrafikSeçenekleri::yeni()
            .veri_kümesi(küme)
            .x_ekseni(Eksen::değer())
            .y_ekseni(Eksen::değer())
            .seri(ÇizgiSerisi::yeni().eşle("x", "y"));

        let (çözülmüş, hatalar) = seçenekler.veri_kümesini_uygula();

        assert!(hatalar.is_empty());
        assert_eq!(
            çözülmüş.seriler[0]
                .veri()
                .iter()
                .map(|öğe| öğe.değer.clone())
                .collect::<Vec<_>>(),
            vec![VeriDeğeri::Çift([1.0, 10.0]), VeriDeğeri::Çift([3.0, 30.0]),]
        );
    }

    #[test]
    fn dataset_boş_serilere_boyutları_ve_adları_otomatik_dağıtır() {
        let küme = VeriKümesi::yeni(["product", "2015", "2016", "2017"]).kayıtlar([
            ("Matcha Latte", vec![43.3, 85.8, 93.7]),
            ("Milk Tea", vec![83.1, 73.4, 55.1]),
        ]);
        let seçenekler = GrafikSeçenekleri::yeni()
            .veri_kümesi(küme)
            .seri(SütunSerisi::yeni())
            .seri(SütunSerisi::yeni())
            .seri(SütunSerisi::yeni());
        let (çözülmüş, hatalar) = seçenekler.veri_kümesini_uygula();
        assert!(hatalar.is_empty());
        assert_eq!(çözülmüş.seriler[0].ad(), Some("2015"));
        assert_eq!(çözülmüş.seriler[1].ad(), Some("2016"));
        assert_eq!(çözülmüş.seriler[2].ad(), Some("2017"));
        assert_eq!(
            çözülmüş.seriler[0].veri()[0].ad.as_deref(),
            Some("Matcha Latte")
        );
        assert_eq!(çözülmüş.seriler[1].veri()[1].değer.sayı(), Some(73.4));
    }

    #[test]
    fn satır_ve_sütun_series_layout_by_sayaçları_bağımsızdır() {
        use crate::model::veri_kumesi::SeriYerleşimi;

        let küme = VeriKümesi::yeni(["product", "2012", "2013"]).kayıtlar([
            ("Matcha Latte", vec![41.1, 30.4]),
            ("Milk Tea", vec![86.5, 92.1]),
        ]);
        let seçenekler = GrafikSeçenekleri::yeni()
            .veri_kümesi(küme)
            .seri(SütunSerisi::yeni().seri_yerleşimi(SeriYerleşimi::Satır))
            .seri(SütunSerisi::yeni().seri_yerleşimi(SeriYerleşimi::Satır))
            .seri(SütunSerisi::yeni())
            .seri(SütunSerisi::yeni());
        let (çözülmüş, hatalar) = seçenekler.veri_kümesini_uygula();
        assert!(hatalar.is_empty());
        assert_eq!(çözülmüş.seriler[0].ad(), Some("Matcha Latte"));
        assert_eq!(çözülmüş.seriler[1].ad(), Some("Milk Tea"));
        assert_eq!(çözülmüş.seriler[2].ad(), Some("2012"));
        assert_eq!(çözülmüş.seriler[3].ad(), Some("2013"));
        assert_eq!(çözülmüş.seriler[0].veri()[1].değer.sayı(), Some(30.4));
        assert_eq!(çözülmüş.seriler[2].veri()[1].değer.sayı(), Some(86.5));
    }

    #[test]
    fn visual_map_koordinat_dışındaki_dataset_boyutunu_kullanır() {
        let küme = VeriKümesi::yeni(["score", "amount", "product"])
            .satır([89.3.into(), 58_212.into(), "Matcha Latte".into()])
            .satır([10.0.into(), 12_000.into(), "Milk Tea".into()]);
        let seçenekler = GrafikSeçenekleri::yeni()
            .veri_kümesi(küme)
            .görsel_eşleme(
                GörselEşleme::yeni()
                    .boyut("score")
                    .en_az(10.0)
                    .en_çok(90.0)
                    .renkler([0x000000u32, 0xffffffu32]),
            )
            .seri(SütunSerisi::yeni().eşle("product", "amount"));
        let (çözülmüş, hatalar) = seçenekler.veri_kümesini_uygula();
        assert!(hatalar.is_empty());
        let ilk = &çözülmüş.seriler[0].veri()[0];
        assert_eq!(ilk.boyut("score").and_then(VeriDeğeri::sayı), Some(89.3));
        assert_eq!(
            ilk.stil
                .as_ref()
                .and_then(|stil| stil.renk.as_ref())
                .map(Dolgu::temsilî),
            Some(
                GörselEşleme::yeni()
                    .renkler([0x000000u32, 0xffffffu32])
                    .renk_çöz(89.3, [10.0, 90.0])
            )
        );
    }

    #[test]
    fn yalnız_sembol_boyutu_eşleyen_visual_map_dataset_rengini_değiştirmez() {
        let küme = VeriKümesi::yeni(["x", "y", "population"])
            .satır([1.into(), 2.into(), 20_000.into()])
            .satır([3.into(), 4.into(), 1_500_000_000.into()]);
        let seçenekler = GrafikSeçenekleri::yeni()
            .veri_kümesi(küme)
            .görsel_eşleme(
                GörselEşleme::yeni()
                    .boyut("population")
                    .en_az(20_000.0)
                    .en_çok(1_500_000_000.0)
                    .sembol_boyutu(10.0, 70.0),
            )
            .seri(SaçılımSerisi::yeni().eşle("x", "y"));

        let (çözülmüş, hatalar) = seçenekler.veri_kümesini_uygula();

        assert!(hatalar.is_empty());
        assert!(
            çözülmüş.seriler[0]
                .veri()
                .iter()
                .all(|öğe| öğe.stil.is_none())
        );
        assert_eq!(
            çözülmüş.seriler[0].veri()[1]
                .boyut("population")
                .and_then(VeriDeğeri::sayı),
            Some(1_500_000_000.0)
        );
    }

    #[test]
    fn çoklu_visual_map_seriyi_series_index_ile_hedefler() {
        let seçenekler = GrafikSeçenekleri::yeni().görsel_eşlemeler([
            GörselEşleme::yeni().seri_sırası(0).boyut(1usize),
            GörselEşleme::yeni().seri_sırası(1).boyut(0usize),
        ]);

        assert_eq!(
            seçenekler
                .seri_görsel_eşlemesi(0)
                .and_then(|eşleme| eşleme.boyut.as_ref()),
            Some(&crate::model::veri_kumesi::BoyutSeçici::Sıra(1))
        );
        assert_eq!(
            seçenekler
                .seri_görsel_eşlemesi(1)
                .and_then(|eşleme| eşleme.boyut.as_ref()),
            Some(&crate::model::veri_kumesi::BoyutSeçici::Sıra(0))
        );
        assert!(seçenekler.seri_görsel_eşlemesi(2).is_none());
    }

    #[test]
    fn tekil_visual_map_kurucusu_önceki_diziyi_değiştirir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .görsel_eşlemeler([GörselEşleme::yeni().seri_sırası(3)])
            .görsel_eşleme(GörselEşleme::yeni().seri_sırası(1));

        assert!(seçenekler.görsel_eşlemeler.is_empty());
        assert_eq!(seçenekler.tüm_görsel_eşlemeler().count(), 1);
        assert!(seçenekler.seri_görsel_eşlemesi(1).is_some());
        assert!(seçenekler.seri_görsel_eşlemesi(3).is_none());
    }

    #[test]
    fn data_zoom_birden_cok_ekseni_tek_bilesenle_hedefler() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .x_ekseni_ekle(Eksen::kategori().veri(["A", "B"]))
            .x_ekseni_ekle(Eksen::kategori().veri(["A", "B"]))
            .y_ekseni(Eksen::değer())
            .veri_yakınlaştırma(
                VeriYakınlaştırma::sürgü()
                    .x_eksenleri([0, 1, 0])
                    .aralık(30.0, 70.0),
            );

        assert!(seçenekler.doğrula().is_ok());
        let ilk = seçenekler.x_yakınlaştırması(0);
        let ikinci = seçenekler.x_yakınlaştırması(1);
        assert!(ilk.is_some());
        assert!(ikinci.is_some());
        assert!(std::ptr::eq(ilk.unwrap(), ikinci.unwrap()));
        assert_eq!(seçenekler.x_penceresi(1), Some((0.3, 0.7)));
    }

    #[test]
    fn data_zoom_ek_eksen_hedefini_de_dogrular() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .x_ekseni(Eksen::kategori().veri(["A", "B"]))
            .y_ekseni(Eksen::değer())
            .veri_yakınlaştırma(VeriYakınlaştırma::iç().x_eksenleri([0, 2]));

        assert!(matches!(
            seçenekler.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "xAxis",
                sıra: 2
            })
        ));
    }

    #[test]
    fn takvime_bağlı_scatter_eksik_calendar_index_değerini_reddeder() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .seri(crate::model::seri::SaçılımSerisi::yeni().takvim_sırası(2));

        assert!(matches!(
            seçenekler.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "calendar",
                sıra: 2
            })
        ));
    }

    #[test]
    fn takvime_bağlı_pasta_takvimi_ve_tarih_merkezini_doğrular() {
        let eksik_takvim = GrafikSeçenekleri::yeni().seri(
            crate::model::seri::PastaSerisi::yeni()
                .takvim_sırası(2)
                .takvim_merkezi(0.0),
        );
        assert!(matches!(
            eksik_takvim.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "calendar",
                sıra: 2
            })
        ));

        let eksik_merkez = GrafikSeçenekleri::yeni()
            .takvim(crate::model::takvim::TakvimKoordinatı::yıl(2017))
            .seri(crate::model::seri::PastaSerisi::yeni().takvim_sırası(0));
        assert!(matches!(
            eksik_merkez.doğrula(),
            Err(crate::hata::BilesenHatasi::GeçersizSeçenek {
                alan: "series.pie.center",
                ..
            })
        ));
    }

    #[test]
    fn takvime_bağlı_graph_eksik_calendar_index_değerini_reddeder() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .seri(crate::model::seri::GrafoSerisi::yeni().takvim_sırası(2));

        assert!(matches!(
            seçenekler.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "calendar",
                sıra: 2
            })
        ));
    }

    #[test]
    fn takvime_bağlı_custom_eksik_calendar_index_değerini_reddeder() {
        let özel = crate::model::seri::ÖzelSeri::yeni().takvim_sırası(2);
        assert!(!özel.kartezyen_gerekli);
        assert_eq!(özel.takvim_sırası, Some(2));

        let seçenekler = GrafikSeçenekleri::yeni().seri(özel);
        assert!(matches!(
            seçenekler.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "calendar",
                sıra: 2
            })
        ));
    }

    #[test]
    fn eksen_kirilmalari_cakisma_ve_kategori_kullanimini_reddeder() {
        let çakışan = GrafikSeçenekleri::yeni()
            .x_ekseni(Eksen::değer().kırılmalar([
                EksenKırılması::yeni(10.0, 30.0),
                EksenKırılması::yeni(20.0, 40.0),
            ]))
            .y_ekseni(Eksen::değer());
        assert!(matches!(
            çakışan.doğrula(),
            Err(crate::hata::BilesenHatasi::GeçersizSeçenek {
                alan: "axis.breaks",
                ..
            })
        ));

        let kategorik = GrafikSeçenekleri::yeni()
            .x_ekseni(Eksen::kategori().kırılma(EksenKırılması::yeni(0.0, 1.0)))
            .y_ekseni(Eksen::değer());
        assert!(matches!(
            kategorik.doğrula(),
            Err(crate::hata::BilesenHatasi::GeçersizSeçenek {
                alan: "axis.breaks",
                ..
            })
        ));
    }
}
