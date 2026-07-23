//! Grafik seçenekleri — ECharts'taki kök `option` nesnesinin karşılığı.

use std::collections::BTreeMap;

use crate::animasyon::{Yumuşatma, ÖNTANIMLI_SÜRE_MS};
use crate::model::bilesen::{AraçKutusu, Başlık, Fırça, Gösterge, Izgara, İpucu};
use crate::model::eksen::{Eksen, EksenTürü};
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::grafik_bileseni::GrafikBileşeni;
use crate::model::kutupsal::KutupsalKoordinat;
use crate::model::matris::MatrisKoordinatı;
use crate::model::paralel::{ParalelEkseni, ParalelKoordinatı};
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
    /// Çoklu radar koordinat listesi (`radar: []`). Boşsa geriye uyumlu
    /// tekil `radar` alanı kullanılır.
    pub radarlar: Vec<RadarKoordinatı>,
    /// Kutupsal koordinat sistemi (`polar` + `angleAxis` + `radiusAxis`).
    pub kutupsal: Option<KutupsalKoordinat>,
    /// Çoklu kutupsal koordinat listesi (`polar: []`). Boşsa geriye
    /// uyumlu tekil `kutupsal` alanı kullanılır.
    pub kutupsallar: Vec<KutupsalKoordinat>,
    /// ECharts 6.1 `matrix` koordinat sistemi.
    pub matris: Option<MatrisKoordinatı>,
    /// Çoklu `matrix: []` koordinat bileşenleri. Boşsa geriye uyumlu
    /// `matris` alanı kullanılır.
    pub matrisler: Vec<MatrisKoordinatı>,
    /// ECharts `parallel` koordinat bileşeni.
    pub paralel: Option<ParalelKoordinatı>,
    /// Çoklu `parallel: []` koordinat bileşenleri.
    pub paraleller: Vec<ParalelKoordinatı>,
    /// `parallelAxis: []` bileşenleri; her eksen `parallelIndex`/`parallelId`
    /// ile bir koordinata bağlanır.
    pub paralel_eksenleri: Vec<ParalelEkseni>,
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
            radarlar: Vec::new(),
            kutupsal: None,
            kutupsallar: Vec::new(),
            matris: None,
            matrisler: Vec::new(),
            paralel: None,
            paraleller: Vec::new(),
            paralel_eksenleri: Vec::new(),
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
        let seri = seri.into();
        self.seri_kimlikleri.push(seri.kimlik().map(str::to_owned));
        self.seriler.push(seri);
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
            let seri = seri.into();
            self.seri_kimlikleri.push(seri.kimlik().map(str::to_owned));
            self.seriler.push(seri);
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
        self.radarlar.clear();
        self
    }

    /// `radar: []` listesini doğrudan ayarlar.
    pub fn radarlar(mut self, koordinatlar: impl IntoIterator<Item = RadarKoordinatı>) -> Self {
        self.radar = None;
        self.radarlar = koordinatlar.into_iter().collect();
        self
    }

    /// Var olan tekil/serili radar listesine bir koordinat ekler.
    pub fn radar_ekle(mut self, koordinat: RadarKoordinatı) -> Self {
        if self.radarlar.is_empty()
            && let Some(tekil) = self.radar.take()
        {
            self.radarlar.push(tekil);
        }
        self.radarlar.push(koordinat);
        self
    }

    /// Tekil ve çoğul radar seçeneklerini ECharts bileşen sırasıyla dolaşır.
    pub fn tüm_radarlar(&self) -> impl DoubleEndedIterator<Item = &RadarKoordinatı> {
        self.radar.iter().chain(self.radarlar.iter())
    }

    pub fn kutupsal(mut self, koordinat: KutupsalKoordinat) -> Self {
        self.kutupsal = Some(koordinat);
        self.kutupsallar.clear();
        self
    }

    /// `polar: []` listesini doğrudan ayarlar.
    pub fn kutupsallar(
        mut self,
        koordinatlar: impl IntoIterator<Item = KutupsalKoordinat>,
    ) -> Self {
        self.kutupsal = None;
        self.kutupsallar = koordinatlar.into_iter().collect();
        self
    }

    /// Var olan tekil/serili kutupsal listeye bir koordinat ekler.
    pub fn kutupsal_ekle(mut self, koordinat: KutupsalKoordinat) -> Self {
        if self.kutupsallar.is_empty()
            && let Some(tekil) = self.kutupsal.take()
        {
            self.kutupsallar.push(tekil);
        }
        self.kutupsallar.push(koordinat);
        self
    }

    /// Etkin `polar` bileşenlerini ECharts dizi sırasıyla dolaşır.
    pub fn tüm_kutupsallar(&self) -> impl Iterator<Item = &KutupsalKoordinat> {
        self.kutupsal
            .iter()
            .filter(|_| self.kutupsallar.is_empty())
            .chain(self.kutupsallar.iter())
    }

    pub fn kutupsal_sayısı(&self) -> usize {
        self.tüm_kutupsallar().count()
    }

    pub fn matris(mut self, koordinat: MatrisKoordinatı) -> Self {
        self.matris = Some(koordinat);
        self.matrisler.clear();
        self
    }

    pub fn matris_ekle(mut self, koordinat: MatrisKoordinatı) -> Self {
        if self.matrisler.is_empty()
            && let Some(tekil) = self.matris.take()
        {
            self.matrisler.push(tekil);
        }
        self.matrisler.push(koordinat);
        self
    }

    pub fn tüm_matrisler(&self) -> impl Iterator<Item = &MatrisKoordinatı> {
        self.matris
            .iter()
            .filter(|_| self.matrisler.is_empty())
            .chain(self.matrisler.iter())
    }

    pub fn matris_sayısı(&self) -> usize {
        self.tüm_matrisler().count()
    }

    pub fn paralel(mut self, koordinat: ParalelKoordinatı) -> Self {
        self.paralel = Some(koordinat);
        self.paraleller.clear();
        self
    }

    pub fn paralel_ekle(mut self, koordinat: ParalelKoordinatı) -> Self {
        if self.paraleller.is_empty()
            && let Some(tekil) = self.paralel.take()
        {
            self.paraleller.push(tekil);
        }
        self.paraleller.push(koordinat);
        self
    }

    pub fn tüm_paraleller(&self) -> impl Iterator<Item = &ParalelKoordinatı> {
        self.paralel
            .iter()
            .filter(|_| self.paraleller.is_empty())
            .chain(self.paraleller.iter())
    }

    pub fn paralel_sayısı(&self) -> usize {
        self.tüm_paraleller().count()
    }

    pub fn paralel_ekseni(mut self, eksen: ParalelEkseni) -> Self {
        self.paralel_eksenleri.push(eksen);
        self
    }

    pub fn paralel_eksenleri(mut self, eksenler: impl IntoIterator<Item = ParalelEkseni>) -> Self {
        self.paralel_eksenleri = eksenler.into_iter().collect();
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
            // Candlestick encode.y dört boyut taşır. Tek değerli serilerin
            // ortak `(x, y)` yoluna indirgenirse open/close/lowest/highest
            // sırası kaybolur; resmî `createSeriesData` gibi bu seri için
            // çok boyutlu değeri doğrudan kur.
            if let Seri::Mum(mum) = &mut *seri {
                if !mum.veri.is_empty() {
                    continue;
                }
                let Some(taban_küme) = kümeler.get(mum.veri_kümesi_sırası) else {
                    hatalar.push(crate::hata::BilesenHatasi::EksikVeri {
                        bileşen: "dataset",
                        sıra: mum.veri_kümesi_sırası,
                    });
                    continue;
                };
                let dönüştürülmüş;
                let küme = if mum.seri_yerleşimi == SeriYerleşimi::Sütun {
                    taban_küme
                } else {
                    dönüştürülmüş = taban_küme.seri_yerleşimiyle(mum.seri_yerleşimi);
                    &dönüştürülmüş
                };
                let (x_boyutu, y_boyutları) = match mum.eşleme.clone() {
                    Some(eşleme) => eşleme,
                    None => {
                        let Some(x) = küme.boyutlar.first().cloned() else {
                            hatalar.push(crate::hata::BilesenHatasi::GeçersizSeçenek {
                                alan: "dataset.encode",
                                ayrıntı: format!(
                                    "dataset[{}] içinde mum x boyutu yok",
                                    mum.veri_kümesi_sırası
                                ),
                            });
                            continue;
                        };
                        let Some(y) = küme.boyutlar.get(1..5) else {
                            hatalar.push(crate::hata::BilesenHatasi::GeçersizSeçenek {
                                alan: "dataset.encode",
                                ayrıntı: format!(
                                    "dataset[{}] içinde mum için dört y boyutu yok",
                                    mum.veri_kümesi_sırası
                                ),
                            });
                            continue;
                        };
                        (x, [y[0].clone(), y[1].clone(), y[2].clone(), y[3].clone()])
                    }
                };
                let adlar = match küme.metinler(&x_boyutu) {
                    Ok(adlar) => adlar,
                    Err(hata) => {
                        hatalar.push(hata);
                        continue;
                    }
                };
                let mut değer_sütunları = Vec::with_capacity(4);
                let mut geçerli = true;
                for boyut in &y_boyutları {
                    match küme.sayılar(boyut) {
                        Ok(değerler) => değer_sütunları.push(değerler),
                        Err(hata) => {
                            hatalar.push(hata);
                            geçerli = false;
                            break;
                        }
                    }
                }
                if !geçerli {
                    continue;
                }
                mum.veri = adlar
                    .into_iter()
                    .enumerate()
                    .map(|(satır_sırası, ad)| {
                        let değer = [
                            değer_sütunları[0][satır_sırası],
                            değer_sütunları[1][satır_sırası],
                            değer_sütunları[2][satır_sırası],
                            değer_sütunları[3][satır_sırası],
                        ];
                        let boyutlar = küme
                            .boyutlar
                            .iter()
                            .enumerate()
                            .map(|(boyut_sırası, boyut_adı)| {
                                (
                                    boyut_adı.clone(),
                                    küme
                                        .satırlar
                                        .get(satır_sırası)
                                        .and_then(|satır| satır.get(boyut_sırası))
                                        .cloned()
                                        .unwrap_or(crate::model::deger::VeriDeğeri::Boş),
                                )
                            })
                            .collect::<Vec<_>>();
                        crate::model::deger::VeriÖğesi::adlı(ad, değer).boyutlar(boyutlar)
                    })
                    .collect();
                continue;
            }
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
                Seri::Huni(s) => (
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
            let dönüştürülmüş;
            let küme = if yerleşim == SeriYerleşimi::Sütun {
                taban_küme
            } else {
                dönüştürülmüş = taban_küme.seri_yerleşimiyle(yerleşim);
                &dönüştürülmüş
            };
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
                Seri::Huni(s) => {
                    s.veri = veri;
                    s.öğe_yamaları.resize(s.veri.len(), None);
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
        // Candlestick'in iki yönlü itemStyle'ı tek bir `Dolgu` alanında
        // tutulmaz; ECharts palette görevi yine de açık `color`ı seri
        // rengi sayar. Legend/marker rengi yükselen renk olur ve bu seri
        // sonraki çizgi serilerinin palet sırasını tüketmez.
        if let Some(Seri::Mum(mum)) = self.seriler.get(sıra) {
            return mum.yükselen_renk;
        }
        if let Some(renk) = self.seriler.get(sıra).and_then(|seri| seri.açık_renk()) {
            return renk.temsilî();
        }
        // PaletteTask, açık bir `itemStyle.color` taşıyan seriyi paletten
        // boyamaz ve bu seri palette bir sıra da tüketmez. Global
        // PaletteMixin ayrıca seri adını renge eşler; aynı adlı iki seri
        // (ör. histogramın dikey/yatay görünümleri) aynı palet rengini alır.
        let mut sonraki = 0usize;
        let mut ad_sıraları = BTreeMap::<&str, usize>::new();
        for (aday_sırası, seri) in self.seriler.iter().enumerate().take(sıra.saturating_add(1)) {
            if seri.açık_renk().is_some() || matches!(seri, Seri::Mum(_)) {
                continue;
            }
            let palet_sırası = if let Some(ad) = seri.ad().filter(|ad| !ad.is_empty()) {
                if let Some(kayıtlı) = ad_sıraları.get(ad) {
                    *kayıtlı
                } else {
                    let yeni = sonraki;
                    ad_sıraları.insert(ad, yeni);
                    sonraki = sonraki.saturating_add(1);
                    yeni
                }
            } else {
                let yeni = sonraki;
                sonraki = sonraki.saturating_add(1);
                yeni
            };
            if aday_sırası == sıra {
                return self.palet_rengi(palet_sırası);
            }
        }
        self.palet_rengi(0)
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
        for (sıra, kutupsal) in self.tüm_kutupsallar().enumerate() {
            for (alan, açı) in [
                ("angleAxis.startAngle", Some(kutupsal.başlangıç_açısı)),
                ("angleAxis.endAngle", kutupsal.bitiş_açısı),
            ] {
                if açı.is_some_and(|değer| !değer.is_finite()) {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan,
                        ayrıntı: format!("polar[{sıra}] açısı sonlu olmalı"),
                    });
                }
            }
        }
        let paralel_koordinatları = self.tüm_paraleller().collect::<Vec<_>>();
        let örtük_paralel = paralel_koordinatları.is_empty()
            && self
                .seriler
                .iter()
                .any(|seri| matches!(seri, Seri::Paralel(_)));
        let paralel_adedi = if örtük_paralel {
            1
        } else {
            paralel_koordinatları.len()
        };
        let mut paralel_kimlikleri = std::collections::HashSet::new();
        for (sıra, koordinat) in paralel_koordinatları.iter().enumerate() {
            if let Some(kimlik) = koordinat.kimlik.as_deref()
                && !paralel_kimlikleri.insert(kimlik)
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "parallel.id",
                    ayrıntı: format!("{sıra}. bileşenin `{kimlik}` kimliği yinelendi"),
                });
            }
            for (alan, değer) in [
                (
                    "parallel.axisExpandCenter",
                    koordinat.eksen_genişletme_merkezi,
                ),
                (
                    "parallel.axisExpandWidth",
                    Some(koordinat.eksen_genişletme_genişliği),
                ),
                (
                    "parallel.axisExpandRate",
                    Some(koordinat.eksen_genişletme_oranı),
                ),
            ] {
                if değer.is_some_and(|değer| !değer.is_finite() || değer < 0.0) {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan,
                        ayrıntı: format!(
                            "parallel[{sıra}] değeri sonlu ve negatif olmayan olmalı"
                        ),
                    });
                }
            }
            if let Some([başlangıç, bitiş]) = koordinat.eksen_genişletme_penceresi
                && (!başlangıç.is_finite() || !bitiş.is_finite() || başlangıç > bitiş)
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "parallel.axisExpandWindow",
                    ayrıntı: format!(
                        "parallel[{sıra}] penceresi sonlu ve başlangıç <= bitiş olmalı"
                    ),
                });
            }
        }

        let paralel_sırasını_çöz = |sıra: usize, kimlik: Option<&str>| -> Option<usize> {
            if let Some(kimlik) = kimlik {
                paralel_koordinatları
                    .iter()
                    .position(|koordinat| koordinat.kimlik.as_deref() == Some(kimlik))
            } else {
                (sıra < paralel_adedi).then_some(sıra)
            }
        };
        let örtük_varsayılan = crate::model::paralel::ParalelKoordinatı::default();
        let mut çözülmüş_paralel_eksenler = Vec::with_capacity(self.paralel_eksenleri.len());
        let mut görülen_paralel_boyutlar = std::collections::HashSet::new();
        for (eksen_sırası, eksen) in self.paralel_eksenleri.iter().enumerate() {
            let Some(paralel_sırası) =
                paralel_sırasını_çöz(eksen.paralel_sırası, eksen.paralel_kimliği.as_deref())
            else {
                if let Some(kimlik) = &eksen.paralel_kimliği {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "parallelAxis.parallelId",
                        ayrıntı: format!("{eksen_sırası}. eksenin `{kimlik}` koordinatı yok"),
                    });
                }
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "parallel",
                    sıra: eksen.paralel_sırası,
                });
            };
            if eksen.boyutlar.is_empty() {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "parallelAxis.dim",
                    ayrıntı: format!("{eksen_sırası}. eksen en az bir boyut taşımalı"),
                });
            }
            for boyut in &eksen.boyutlar {
                if !görülen_paralel_boyutlar.insert((paralel_sırası, *boyut)) {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "parallelAxis.dim",
                        ayrıntı: format!(
                            "parallel[{paralel_sırası}] içinde {boyut}. boyut birden çok eksene bağlı"
                        ),
                    });
                }
            }
            let stil = &eksen.alan_seçim_stili;
            if !stil.genişlik.is_finite()
                || stil.genişlik < 0.0
                || !stil.kenarlık_kalınlığı.is_finite()
                || stil.kenarlık_kalınlığı < 0.0
                || !stil.opaklık.is_finite()
                || !(0.0..=1.0).contains(&stil.opaklık)
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "parallelAxis.areaSelectStyle",
                    ayrıntı: format!("{eksen_sırası}. eksenin seçim stili geçerli değil"),
                });
            }
            for (aralık_sırası, aralık) in eksen.etkin_aralıklar.iter().enumerate() {
                if !aralık[0].is_finite() || !aralık[1].is_finite() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "parallelAxis.activeIntervals",
                        ayrıntı: format!(
                            "{eksen_sırası}. eksenin {aralık_sırası}. aralığı sonlu olmalı"
                        ),
                    });
                }
            }
            let koordinat = paralel_koordinatları
                .get(paralel_sırası)
                .copied()
                .unwrap_or(&örtük_varsayılan);
            çözülmüş_paralel_eksenler.push(eksen.çöz(&koordinat.eksen_varsayılanı));
        }
        for (seri_sırası, seri) in self.seriler.iter().enumerate() {
            let Seri::Paralel(paralel) = seri else {
                continue;
            };
            if paralel_sırasını_çöz(paralel.paralel_sırası, paralel.paralel_kimliği.as_deref())
                .is_none()
            {
                if let Some(kimlik) = &paralel.paralel_kimliği {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.parallel.parallelId",
                        ayrıntı: format!("{seri_sırası}. serinin `{kimlik}` koordinatı yok"),
                    });
                }
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "parallel",
                    sıra: paralel.paralel_sırası,
                });
            }
            if !(0.0..=1.0).contains(&paralel.aktif_opaklık)
                || !(0.0..=1.0).contains(&paralel.etkin_değil_opaklık)
                || !(0.0..=1.0).contains(&paralel.yumuşaklık)
            {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "series.parallel.opacity/smooth",
                    ayrıntı: format!("{seri_sırası}. parallel seri oranları 0..=1 olmalı"),
                });
            }
        }
        for eksen in x_eksenler
            .iter()
            .chain(y_eksenler.iter())
            .chain(self.tek_eksenler.iter().map(|tek| &tek.eksen))
            .chain(
                self.tüm_kutupsallar()
                    .flat_map(|kutupsal| [&kutupsal.açısal_eksen, &kutupsal.radyal_eksen]),
            )
            .chain(
                paralel_koordinatları
                    .iter()
                    .map(|paralel| &paralel.eksen_varsayılanı),
            )
            .chain(self.seriler.iter().filter_map(|seri| match seri {
                Seri::Paralel(paralel) => paralel.eksen_varsayılanı.as_ref(),
                _ => None,
            }))
            .chain(çözülmüş_paralel_eksenler.iter())
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
        let başlıklar = self
            .başlık
            .iter()
            .filter(|_| self.başlıklar.is_empty())
            .chain(self.başlıklar.iter());
        for başlık in başlıklar {
            if let Some(matris_sırası) = başlık.matris_sırası {
                if self.tüm_matrisler().nth(matris_sırası).is_none() {
                    return Err(BilesenHatasi::EksikVeri {
                        bileşen: "matrix",
                        sıra: matris_sırası,
                    });
                }
                if başlık.matris_koordinatı.is_none() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "title.coord",
                        ayrıntı: "Matrix başlığı bir hücre/aralık koordinatı taşımalı".to_owned(),
                    });
                }
            } else if başlık.matris_koordinatı.is_some() {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "title.coordinateSystem",
                    ayrıntı: "Matrix başlık koordinatı bir matrixIndex ile bağlanmalı".to_owned(),
                });
            }
        }
        for ızgara in self.etkin_ızgaralar() {
            if let Some(matris_sırası) = ızgara.matris_sırası {
                if self.tüm_matrisler().nth(matris_sırası).is_none() {
                    return Err(BilesenHatasi::EksikVeri {
                        bileşen: "matrix",
                        sıra: matris_sırası,
                    });
                }
                if ızgara.matris_koordinatı.is_none() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "grid.coord",
                        ayrıntı: "Matrix ızgarası bir hücre/aralık koordinatı taşımalı"
                            .to_owned(),
                    });
                }
            } else if ızgara.matris_koordinatı.is_some() {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "grid.coordinateSystem",
                    ayrıntı: "Matrix ızgara koordinatı bir matrixIndex ile bağlanmalı".to_owned(),
                });
            }
        }
        for seri in &self.seriler {
            let matris_sırası = match seri {
                Seri::Isı(ısı) => ısı.matris_sırası,
                Seri::Saçılım(saçılım) => saçılım.matris_sırası,
                Seri::Pasta(pasta) => pasta.matris_sırası,
                Seri::Grafo(grafo) => grafo.matris_sırası,
                Seri::Özel(özel) => özel.matris_sırası,
                Seri::AğaçHaritası(ağaç_haritası) => ağaç_haritası.matris_sırası,
                Seri::GüneşPatlaması(güneş) => güneş.matris_sırası,
                _ => None,
            };
            if let Some(matris_sırası) = matris_sırası
                && self.tüm_matrisler().nth(matris_sırası).is_none()
            {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "matrix",
                    sıra: matris_sırası,
                });
            }
            if let Seri::Radar(radar) = seri
                && self.tüm_radarlar().nth(radar.radar_sırası).is_none()
            {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "radar",
                    sıra: radar.radar_sırası,
                });
            }
            if let Some(kutupsal_sırası) = seri.kutupsal_sırası()
                && self.tüm_kutupsallar().nth(kutupsal_sırası).is_none()
            {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "polar",
                    sıra: kutupsal_sırası,
                });
            }
            if let Seri::Saçılım(saçılım) = seri
                && let Some(tek_eksen_sırası) = saçılım.tek_eksen_sırası
                && self.tek_eksenler.get(tek_eksen_sırası).is_none()
            {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "singleAxis",
                    sıra: tek_eksen_sırası,
                });
            }
            if let Seri::TemaNehri(nehir) = seri
                && self.tek_eksenler.get(nehir.tek_eksen_sırası).is_none()
            {
                return Err(BilesenHatasi::EksikVeri {
                    bileşen: "singleAxis",
                    sıra: nehir.tek_eksen_sırası,
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
                if p.matris_sırası.is_some() && p.matris_merkezi.is_none() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.pie.center",
                        ayrıntı: "Matrix pastası bir hücre/aralık merkezi taşımalı".to_owned(),
                    });
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
            if let Seri::AğaçHaritası(ağaç_haritası) = seri {
                if let Some(takvim_sırası) = ağaç_haritası.takvim_sırası {
                    if self.takvimler.get(takvim_sırası).is_none() {
                        return Err(BilesenHatasi::EksikVeri {
                            bileşen: "calendar",
                            sıra: takvim_sırası,
                        });
                    }
                    if !ağaç_haritası.takvim_koordinatı.is_some_and(f64::is_finite) {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.treemap.coord",
                            ayrıntı: "takvim Treemap'i sonlu bir tarih koordinatı taşımalı"
                                .to_owned(),
                        });
                    }
                } else if ağaç_haritası.takvim_koordinatı.is_some() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.treemap.coordinateSystem",
                        ayrıntı: "takvim koordinatı bir calendarIndex ile bağlanmalı".to_owned(),
                    });
                }
                if ağaç_haritası.matris_sırası.is_some()
                    && ağaç_haritası.matris_koordinatı.is_none()
                {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.treemap.coord",
                        ayrıntı: "Matrix Treemap'i bir hücre/aralık koordinatı taşımalı"
                            .to_owned(),
                    });
                }
                if ağaç_haritası.matris_sırası.is_none()
                    && ağaç_haritası.matris_koordinatı.is_some()
                {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.treemap.coordinateSystem",
                        ayrıntı: "Matrix koordinatı bir matrixIndex ile bağlanmalı".to_owned(),
                    });
                }
            }
            if let Seri::GüneşPatlaması(güneş) = seri {
                if let Some(takvim_sırası) = güneş.takvim_sırası {
                    if self.takvimler.get(takvim_sırası).is_none() {
                        return Err(BilesenHatasi::EksikVeri {
                            bileşen: "calendar",
                            sıra: takvim_sırası,
                        });
                    }
                    if !güneş.takvim_koordinatı.is_some_and(f64::is_finite) {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.sunburst.coord",
                            ayrıntı: "takvim Sunburst'ü sonlu bir tarih koordinatı taşımalı"
                                .to_owned(),
                        });
                    }
                } else if güneş.takvim_koordinatı.is_some() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sunburst.coordinateSystem",
                        ayrıntı: "takvim koordinatı bir calendarIndex ile bağlanmalı".to_owned(),
                    });
                }
                if güneş.matris_sırası.is_some() && güneş.matris_koordinatı.is_none() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sunburst.coord",
                        ayrıntı: "Matrix Sunburst'ü bir hücre/aralık koordinatı taşımalı"
                            .to_owned(),
                    });
                }
                if güneş.matris_sırası.is_none() && güneş.matris_koordinatı.is_some() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sunburst.coordinateSystem",
                        ayrıntı: "Matrix koordinatı bir matrixIndex ile bağlanmalı".to_owned(),
                    });
                }
                if !güneş.başlangıç_açısı.is_finite()
                    || !güneş.en_küçük_açı.is_finite()
                    || güneş.en_küçük_açı < 0.0
                {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sunburst.startAngle/minAngle",
                        ayrıntı: "açılar sonlu, minAngle negatif olmayan sayı olmalı".to_owned(),
                    });
                }
                let uzunluk_geçerli = |uzunluk: crate::model::Uzunluk, negatif_olabilir: bool| {
                    let değer = match uzunluk {
                        crate::model::Uzunluk::Piksel(değer)
                        | crate::model::Uzunluk::Yüzde(değer) => değer,
                    };
                    değer.is_finite() && (negatif_olabilir || değer >= 0.0)
                };
                if !uzunluk_geçerli(güneş.merkez.0, true)
                    || !uzunluk_geçerli(güneş.merkez.1, true)
                    || !uzunluk_geçerli(güneş.yarıçap.0, false)
                    || !uzunluk_geçerli(güneş.yarıçap.1, false)
                {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sunburst.center/radius",
                        ayrıntı: "merkez sonlu, yarıçaplar sonlu ve negatif olmayan olmalı"
                            .to_owned(),
                    });
                }
                for (sıra, seviye) in güneş.seviyeler.iter().enumerate() {
                    if let Some((iç, dış)) = seviye.yarıçap
                        && (!uzunluk_geçerli(iç, false) || !uzunluk_geçerli(dış, false))
                    {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.sunburst.levels.radius",
                            ayrıntı: format!("{sıra}. level yarıçapları geçersiz"),
                        });
                    }
                }
            }
            if let Seri::Sankey(sankey) = seri {
                if let Some(takvim_sırası) = sankey.takvim_sırası {
                    if self.takvimler.get(takvim_sırası).is_none() {
                        return Err(BilesenHatasi::EksikVeri {
                            bileşen: "calendar",
                            sıra: takvim_sırası,
                        });
                    }
                    if !sankey.takvim_koordinatı.is_some_and(f64::is_finite) {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.sankey.coord",
                            ayrıntı: "takvim Sankey'i sonlu bir tarih koordinatı taşımalı"
                                .to_owned(),
                        });
                    }
                } else if sankey.takvim_koordinatı.is_some() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sankey.coordinateSystem",
                        ayrıntı: "takvim koordinatı bir calendarIndex ile bağlanmalı".to_owned(),
                    });
                }
                if let Some(matris_sırası) = sankey.matris_sırası
                    && self.tüm_matrisler().nth(matris_sırası).is_none()
                {
                    return Err(BilesenHatasi::EksikVeri {
                        bileşen: "matrix",
                        sıra: matris_sırası,
                    });
                }
                if sankey.matris_sırası.is_some() && sankey.matris_koordinatı.is_none() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sankey.coord",
                        ayrıntı: "Matrix Sankey'i bir hücre/aralık koordinatı taşımalı"
                            .to_owned(),
                    });
                }
                if sankey.matris_sırası.is_none() && sankey.matris_koordinatı.is_some() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sankey.coordinateSystem",
                        ayrıntı: "Matrix koordinatı bir matrixIndex ile bağlanmalı".to_owned(),
                    });
                }
                if !sankey.düğüm_genişliği.is_finite()
                    || sankey.düğüm_genişliği < 0.0
                    || !sankey.düğüm_boşluğu.is_finite()
                    || sankey.düğüm_boşluğu < 0.0
                {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sankey.nodeWidth/nodeGap",
                        ayrıntı: "düğüm genişliği ve boşluğu sonlu, negatif olmayan sayı olmalı"
                            .to_owned(),
                    });
                }
                if !sankey.yakınlaştırma.is_finite()
                    || sankey.yakınlaştırma <= 0.0
                    || !sankey.en_küçük_ölçek.is_finite()
                    || !sankey.en_büyük_ölçek.is_finite()
                    || sankey.en_küçük_ölçek <= 0.0
                    || sankey.en_büyük_ölçek < sankey.en_küçük_ölçek
                {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sankey.zoom/scaleLimit",
                        ayrıntı:
                            "yakınlaştırma ve ölçek sınırları pozitif, sonlu ve sıralı olmalı"
                                .to_owned(),
                    });
                }

                let mut adlar = std::collections::HashSet::new();
                for (sıra, düğüm) in sankey.düğümler.iter().enumerate() {
                    if düğüm.ad.is_empty() || !adlar.insert(düğüm.ad.as_str()) {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.sankey.data.name",
                            ayrıntı: format!("{sıra}. düğüm adı boş ya da yinelenmiş"),
                        });
                    }
                    if düğüm
                        .değer
                        .is_some_and(|değer| !değer.is_finite() || değer < 0.0)
                    {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.sankey.data.value",
                            ayrıntı: format!("{sıra}. düğüm değeri geçersiz"),
                        });
                    }
                }
                for (sıra, bağ) in sankey.bağlar.iter().enumerate() {
                    if !bağ.değer.is_finite() || bağ.değer < 0.0 {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.sankey.links.value",
                            ayrıntı: format!(
                                "{sıra}. bağ değeri sonlu ve negatif olmayan olmalı"
                            ),
                        });
                    }
                    if !adlar.contains(bağ.kaynak.as_str()) || !adlar.contains(bağ.hedef.as_str())
                    {
                        return Err(BilesenHatasi::GeçersizSeçenek {
                            alan: "series.sankey.links.source/target",
                            ayrıntı: format!("{sıra}. bağ bilinmeyen düğüme başvuruyor"),
                        });
                    }
                }
                let palet = |sıra: usize| self.palet_rengi(sıra);
                if let Err(hata) = crate::grafik::sankey::sankey_yerleşimi(
                    sankey,
                    crate::koordinat::Dikdörtgen::yeni(0.0, 0.0, 1000.0, 600.0),
                    &palet,
                ) {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "series.sankey.data/links",
                        ayrıntı: hata.to_string(),
                    });
                }
            }
            if let Seri::Hatlar(hatlar) = seri {
                match hatlar.koordinat_sistemi {
                    crate::model::hatlar::HatKoordinatSistemi::Kutupsal
                        if self.tüm_kutupsallar().nth(hatlar.kutupsal_sırası).is_none() =>
                    {
                        return Err(BilesenHatasi::EksikVeri {
                            bileşen: "polar",
                            sıra: hatlar.kutupsal_sırası,
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
                        if self.tüm_matrisler().nth(hatlar.matris_sırası).is_none() =>
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

        fn değeri_karıştır(eski: &VeriDeğeri, yeni: &VeriDeğeri, t: f64) -> VeriDeğeri {
            match (eski, yeni) {
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
                (VeriDeğeri::KarmaDizi(a), VeriDeğeri::KarmaDizi(b)) if a.len() == b.len() => {
                    VeriDeğeri::KarmaDizi(
                        a.iter()
                            .zip(b.iter())
                            .map(|(eski, yeni)| değeri_karıştır(eski, yeni, t))
                            .collect(),
                    )
                }
                _ => yeni.clone(),
            }
        }

        let öğe_karıştır = |e: &VeriÖğesi, y: &VeriÖğesi| -> VeriÖğesi {
            let değer = değeri_karıştır(&e.değer, &y.değer, t);
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
    use crate::model::seri::{HuniSerisi, MumSerisi, SaçılımSerisi, SütunSerisi, ÇizgiSerisi};
    use crate::model::stil::ÖğeStili;
    use crate::renk::{Dolgu, Renk};

    #[test]
    fn coklu_polar_tekil_apiyi_yukseltir_ve_polar_indexi_dogrular() {
        let ilk = KutupsalKoordinat::yeni().başlangıç_açısı(90.0);
        let ikinci = KutupsalKoordinat::yeni().başlangıç_açısı(-90.0);
        let seçenekler = GrafikSeçenekleri::yeni()
            .kutupsal(ilk)
            .kutupsal_ekle(ikinci)
            .seri(SütunSerisi::yeni().kutupsal_sırası(1).veri([1.0]));

        assert!(seçenekler.kutupsal.is_none());
        assert_eq!(seçenekler.kutupsal_sayısı(), 2);
        assert_eq!(
            seçenekler
                .tüm_kutupsallar()
                .map(|kutupsal| kutupsal.başlangıç_açısı)
                .collect::<Vec<_>>(),
            [90.0, -90.0]
        );
        assert!(seçenekler.doğrula().is_ok());

        let eksik = GrafikSeçenekleri::yeni()
            .kutupsal(KutupsalKoordinat::yeni())
            .seri(SütunSerisi::yeni().kutupsal_sırası(1).veri([1.0]));
        assert!(matches!(
            eksik.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "polar",
                sıra: 1
            })
        ));
    }

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
    fn mum_yukselen_rengi_marker_ve_sonraki_seri_paletini_belirler() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .seri(
                MumSerisi::yeni()
                    .yükselen_renk(0xec0000)
                    .veri([[10.0, 20.0, 5.0, 25.0]]),
            )
            .seri(ÇizgiSerisi::yeni().veri([20.0]));

        assert_eq!(seçenekler.seri_rengi(0), Renk::onaltılık(0xec0000));
        assert_eq!(seçenekler.seri_rengi(1), seçenekler.palet_rengi(0));
    }

    #[test]
    fn ayni_adli_seriler_global_palet_rengini_paylasir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .seri(SaçılımSerisi::yeni().ad("ham").veri([[1.0, 2.0]]))
            .seri(SütunSerisi::yeni().ad("histogram").veri([2]))
            .seri(SütunSerisi::yeni().ad("histogram").veri([3]));

        assert_eq!(seçenekler.seri_rengi(0), seçenekler.palet_rengi(0));
        assert_eq!(seçenekler.seri_rengi(1), seçenekler.palet_rengi(1));
        assert_eq!(seçenekler.seri_rengi(2), seçenekler.palet_rengi(1));
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
    fn dataset_mum_encode_dort_y_boyutunu_resmi_sirada_tasir() {
        let küme = VeriKümesi::yeni([
            "time", "open", "highest", "lowest", "close", "volume", "sign",
        ])
        .satır([
            "2011-01-01\n00:01:00".into(),
            10.0.into(),
            15.0.into(),
            8.0.into(),
            12.0.into(),
            42_000.0.into(),
            1.0.into(),
        ]);
        let seçenekler = GrafikSeçenekleri::yeni()
            .veri_kümesi(küme)
            .seri(MumSerisi::yeni().eşle("time", ["open", "close", "lowest", "highest"]));

        let (çözülmüş, hatalar) = seçenekler.veri_kümesini_uygula();

        assert!(hatalar.is_empty());
        let öğe = &çözülmüş.seriler[0].veri()[0];
        assert_eq!(öğe.ad.as_deref(), Some("2011-01-01\n00:01:00"));
        assert_eq!(öğe.değer.dizi(), Some([10.0, 12.0, 8.0, 15.0].as_slice()));
        assert_eq!(
            öğe.boyut("volume").and_then(VeriDeğeri::sayı),
            Some(42_000.0)
        );
        assert_eq!(öğe.boyut("sign").and_then(VeriDeğeri::sayı), Some(1.0));
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
    fn huni_dataset_encode_ad_deger_ve_oge_yama_sirasini_korur() {
        let küme = VeriKümesi::yeni(["stage", "amount"])
            .satır(["Visit".into(), 100.into()])
            .satır(["Order".into(), 40.into()]);
        let seçenekler = GrafikSeçenekleri::yeni()
            .veri_kümesi(küme)
            .seri(HuniSerisi::yeni().eşle("stage", "amount"));

        let (çözülmüş, hatalar) = seçenekler.veri_kümesini_uygula();

        assert!(hatalar.is_empty());
        let Seri::Huni(huni) = &çözülmüş.seriler[0] else {
            unreachable!();
        };
        assert_eq!(huni.veri[0].ad.as_deref(), Some("Visit"));
        assert_eq!(huni.veri[1].değer.sayı(), Some(40.0));
        assert_eq!(huni.öğe_yamaları.len(), 2);
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
    fn tema_nehri_single_axis_index_bagini_dogrular() {
        let eksik = GrafikSeçenekleri::yeni().seri(
            crate::model::seri::TemaNehriSerisi::yeni()
                .tek_eksen_sırası(1)
                .veri([(0.0, 1.0, "A")]),
        );
        assert!(matches!(
            eksik.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "singleAxis",
                sıra: 1
            })
        ));

        let geçerli = GrafikSeçenekleri::yeni()
            .tek_eksen(TekEksen::yeni())
            .seri(crate::model::seri::TemaNehriSerisi::yeni().veri([(0.0, 1.0, "A")]));
        assert!(geçerli.doğrula().is_ok());
    }

    #[test]
    fn matrixe_bagli_baslik_ve_izgara_index_ile_koordinati_birlikte_dogrular() {
        let eksik_matris =
            GrafikSeçenekleri::yeni().başlık(Başlık::yeni().matris_hücresi(1, 0usize, 0usize));
        assert!(matches!(
            eksik_matris.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "matrix",
                sıra: 1
            })
        ));

        let mut başlık = Başlık::yeni().matris_hücresi(0, 0usize, 0usize);
        başlık.matris_koordinatı = None;
        let eksik_koordinat = GrafikSeçenekleri::yeni()
            .matris(MatrisKoordinatı::yeni())
            .başlık(başlık);
        assert!(matches!(
            eksik_koordinat.doğrula(),
            Err(crate::hata::BilesenHatasi::GeçersizSeçenek {
                alan: "title.coord",
                ..
            })
        ));

        let geçerli = GrafikSeçenekleri::yeni()
            .matris(MatrisKoordinatı::yeni())
            .ızgara(Izgara::yeni().matris_hücresi(0, 0usize, 0usize));
        assert!(geçerli.doğrula().is_ok());
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
    fn treemap_calendar_ve_matrix_kutu_koordinatlarini_dogrular() {
        let eksik_takvim = GrafikSeçenekleri::yeni()
            .seri(crate::model::seri::AğaçHaritasıSerisi::yeni().takvim_hücresi(0.0));
        assert!(matches!(
            eksik_takvim.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "calendar",
                sıra: 0
            })
        ));

        let eksik_koordinat = GrafikSeçenekleri::yeni()
            .matris(MatrisKoordinatı::yeni())
            .seri(crate::model::seri::AğaçHaritasıSerisi::yeni().matris_sırası(0));
        assert!(matches!(
            eksik_koordinat.doğrula(),
            Err(crate::hata::BilesenHatasi::GeçersizSeçenek {
                alan: "series.treemap.coord",
                ..
            })
        ));

        let geçerli = GrafikSeçenekleri::yeni()
            .matris(MatrisKoordinatı::yeni())
            .seri(crate::model::seri::AğaçHaritasıSerisi::yeni().matris_hücresi(0usize, 0usize));
        assert!(geçerli.doğrula().is_ok());
    }

    #[test]
    fn sankey_view_calendar_ve_matrix_kutu_koordinatlarini_dogrular() {
        let seri = || {
            crate::model::sankey::SankeySerisi::yeni()
                .düğümler(["A", "B"])
                .bağlar([("A", "B", 1.0)])
        };
        let eksik_takvim = GrafikSeçenekleri::yeni().seri(seri().takvim_hücresi(0.0));
        assert!(matches!(
            eksik_takvim.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "calendar",
                sıra: 0
            })
        ));

        let eksik_koordinat = GrafikSeçenekleri::yeni()
            .matris(MatrisKoordinatı::yeni())
            .seri(seri().matris_sırası(0));
        assert!(matches!(
            eksik_koordinat.doğrula(),
            Err(crate::hata::BilesenHatasi::GeçersizSeçenek {
                alan: "series.sankey.coord",
                ..
            })
        ));

        let geçerli_matris = GrafikSeçenekleri::yeni()
            .matris(MatrisKoordinatı::yeni())
            .seri(seri().matris_hücresi(0usize, 0usize));
        assert!(geçerli_matris.doğrula().is_ok());

        let geçerli_takvim = GrafikSeçenekleri::yeni()
            .takvim(crate::model::takvim::TakvimKoordinatı::yıl(1970))
            .seri(seri().takvim_hücresi(0.0));
        assert!(geçerli_takvim.doğrula().is_ok());
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

    #[test]
    fn parallel_serisi_ortuk_koordinati_kabul_eder_ve_acik_baglari_dogrular() {
        let örtük = GrafikSeçenekleri::yeni().seri(
            crate::model::seri::ParalelSerisi::yeni()
                .boyutlar(["A", "B"])
                .veri([vec![1.0, 2.0]]),
        );
        assert!(örtük.doğrula().is_ok());

        let geçerli = GrafikSeçenekleri::yeni()
            .paralel(crate::model::paralel::ParalelKoordinatı::yeni().kimlik("ikinci"))
            .paralel_ekseni(crate::model::paralel::ParalelEkseni::yeni(0).paralel_kimliği("ikinci"))
            .paralel_ekseni(crate::model::paralel::ParalelEkseni::yeni(1).paralel_kimliği("ikinci"))
            .seri(
                crate::model::seri::ParalelSerisi::yeni()
                    .paralel_kimliği("ikinci")
                    .karma_veri([vec![VeriDeğeri::from(1), VeriDeğeri::from("A")]]),
            );
        assert!(geçerli.doğrula().is_ok());

        let eksik = GrafikSeçenekleri::yeni()
            .paralel(crate::model::paralel::ParalelKoordinatı::yeni())
            .seri(
                crate::model::seri::ParalelSerisi::yeni()
                    .paralel_sırası(1)
                    .veri([vec![1.0, 2.0]]),
            );
        assert!(matches!(
            eksik.doğrula(),
            Err(crate::hata::BilesenHatasi::EksikVeri {
                bileşen: "parallel",
                sıra: 1
            })
        ));
    }

    #[test]
    fn parallel_axis_dim_tekrarini_ve_gecersiz_expand_penceresini_reddeder() {
        let yinelenen = GrafikSeçenekleri::yeni()
            .paralel(crate::model::paralel::ParalelKoordinatı::yeni())
            .paralel_ekseni(crate::model::paralel::ParalelEkseni::yeni(0))
            .paralel_ekseni(crate::model::paralel::ParalelEkseni::yeni(0));
        assert!(matches!(
            yinelenen.doğrula(),
            Err(crate::hata::BilesenHatasi::GeçersizSeçenek {
                alan: "parallelAxis.dim",
                ..
            })
        ));

        let mut koordinat = crate::model::paralel::ParalelKoordinatı::yeni();
        koordinat.eksen_genişletme_penceresi = Some([20.0, 10.0]);
        let pencere = GrafikSeçenekleri::yeni().paralel(koordinat);
        assert!(matches!(
            pencere.doğrula(),
            Err(crate::hata::BilesenHatasi::GeçersizSeçenek {
                alan: "parallel.axisExpandWindow",
                ..
            })
        ));
    }
}
