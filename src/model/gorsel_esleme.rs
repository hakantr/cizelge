//! Görsel eşleme — ECharts `visualMap` bileşeninin çekirdeği: sayısal
//! değerleri renk şeridine eşler. Sürekli (`continuous`) ve parçalı
//! (`piecewise`) kipler aynı modelde tutulur.

use std::borrow::Cow;

use crate::model::bilesen::Yön;
use crate::model::deger::VeriDeğeri;
use crate::model::veri_kumesi::BoyutSeçici;
use crate::model::{DikeyKonum, Uzunluk, YatayKonum};
use crate::renk::Renk;
use crate::yardimci::sayi::yuvarla;

/// Parçalı eşlemenin tek dilimi (`visualMap-piecewise` `pieces` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct EşlemeParçası {
    /// Tek bir kesin değer (`pieces[i].value`). Verildiğinde aralık alanları
    /// dikkate alınmaz.
    pub değer: Option<f64>,
    /// Alt sınır; `None` = -∞.
    pub en_az: Option<f64>,
    /// Alt sınır dâhil mi (`gte`/`min`) yoksa hariç mi (`gt`)?
    pub en_az_dahil: bool,
    /// Üst sınır; `None` = +∞.
    pub en_çok: Option<f64>,
    /// Üst sınır dâhil mi (`lte`/`max`) yoksa hariç mi (`lt`)?
    pub en_çok_dahil: bool,
    pub renk: Renk,
    pub etiket: Option<String>,
}

impl EşlemeParçası {
    pub fn yeni(en_az: Option<f64>, en_çok: Option<f64>, renk: impl Into<Renk>) -> Self {
        EşlemeParçası {
            değer: None,
            en_az,
            // Eski kurucunun sözleşmesi [alt, üst) idi; kaynak uyumluluğu
            // için aynı varsayılanı koruyoruz.
            en_az_dahil: true,
            en_çok,
            en_çok_dahil: false,
            renk: renk.into(),
            etiket: None,
        }
    }

    /// ECharts `gt`/`gte`/`lt`/`lte` birleşimlerinin tam karşılığı.
    pub fn aralık(
        en_az: Option<f64>,
        en_az_dahil: bool,
        en_çok: Option<f64>,
        en_çok_dahil: bool,
        renk: impl Into<Renk>,
    ) -> Self {
        Self {
            değer: None,
            en_az,
            en_az_dahil,
            en_çok,
            en_çok_dahil,
            renk: renk.into(),
            etiket: None,
        }
    }

    /// `pieces[i].value`: yalnız verilen değeri eşler.
    pub fn değer(değer: f64, renk: impl Into<Renk>) -> Self {
        Self {
            değer: Some(değer),
            en_az: None,
            en_az_dahil: true,
            en_çok: None,
            en_çok_dahil: true,
            renk: renk.into(),
            etiket: None,
        }
    }

    pub fn gt(en_az: f64, renk: impl Into<Renk>) -> Self {
        Self::aralık(Some(en_az), false, None, false, renk)
    }

    pub fn gte(en_az: f64, renk: impl Into<Renk>) -> Self {
        Self::aralık(Some(en_az), true, None, false, renk)
    }

    pub fn lt(en_çok: f64, renk: impl Into<Renk>) -> Self {
        Self::aralık(None, true, Some(en_çok), false, renk)
    }

    pub fn lte(en_çok: f64, renk: impl Into<Renk>) -> Self {
        Self::aralık(None, true, Some(en_çok), true, renk)
    }

    pub fn etiket(mut self, etiket: impl Into<String>) -> Self {
        self.etiket = Some(etiket.into());
        self
    }

    pub fn içeriyor_mu(&self, değer: f64) -> bool {
        if let Some(kesin) = self.değer {
            return değer == kesin;
        }
        self.en_az
            .map(|alt| {
                if self.en_az_dahil {
                    değer >= alt
                } else {
                    değer > alt
                }
            })
            .unwrap_or(true)
            && self
                .en_çok
                .map(|üst| {
                    if self.en_çok_dahil {
                        değer <= üst
                    } else {
                        değer < üst
                    }
                })
                .unwrap_or(true)
    }

    /// Görüntülenecek etiket.
    pub fn etiket_metni(&self) -> String {
        if let Some(e) = &self.etiket {
            return e.clone();
        }
        if let Some(değer) = self.değer {
            return format!("{değer}");
        }
        match (self.en_az, self.en_çok) {
            (Some(a), Some(b)) => format!(
                "{}{a} – {b}{}",
                if self.en_az_dahil { "[" } else { "(" },
                if self.en_çok_dahil { "]" } else { ")" }
            ),
            (Some(a), None) => format!("{} {a}", if self.en_az_dahil { "≥" } else { ">" }),
            (None, Some(b)) => format!("{} {b}", if self.en_çok_dahil { "≤" } else { "<" }),
            (None, None) => "tümü".to_string(),
        }
    }
}

/// Sürekli görsel eşleme (`visualMap: { type: 'continuous' }`).
#[derive(Clone, PartialEq, Debug)]
pub struct GörselEşleme {
    /// Eşleme alt sınırı; `None` ise veri en küçüğü.
    pub en_az: Option<f64>,
    /// Eşleme üst sınırı; `None` ise veri en büyüğü.
    pub en_çok: Option<f64>,
    /// Renk şeridi, düşükten yükseğe (`inRange.color`).
    pub renkler: Vec<Renk>,
    /// Parçaların/kapsamın dışında kalan değer rengi (`outOfRange.color`).
    /// ECharts, yalnız `inRange`/`pieces` verildiğinde bunu saydam ve
    /// opaklığı sıfır olarak tamamlar.
    pub aralık_dışı_renk: Renk,
    /// HSL açıklık şeridi (`inRange.colorLightness`). Bu kanal renk
    /// kanalından farklı olarak öğenin/paletin mevcut rengini değiştirir.
    pub renk_açıklığı: Option<[f32; 2]>,
    /// Tam renk kanalı etkin mi? ECharts, açık bir `inRange` nesnesinde
    /// yalnız `symbolSize`/`opacity` gibi kısmi kanallar verildiğinde
    /// öntanımlı renk şeridini hedef veriye eklemez.
    pub renk_kanalı: bool,
    /// Renk kanalının kurucu üzerinden açıkça seçilip seçilmediği. Bu bilgi,
    /// kurucu çağrı sırasından bağımsız olarak `renkler(...).sembol_boyutu(...)`
    /// ile birleşik kanal tanımlamayı mümkün kılar.
    pub renk_kanalı_açıkça_ayarlandı: bool,
    /// Sembol çapı şeridi (`inRange.symbolSize`).
    pub sembol_boyutu: Option<[f32; 2]>,
    /// Seçili kapsam dışındaki sembol çapı şeridi
    /// (`outOfRange.symbolSize`). `None`, mevcut seri boyutunu korur.
    pub aralık_dışı_sembol_boyutu: Option<[f32; 2]>,
    /// `outOfRange.color` açıkça verilmiş mi? Yalnız sembol boyutu kullanan
    /// eşlemelerde seçimsiz öğenin taban rengini yanlışlıkla saydamlaştırmayı
    /// önler.
    pub aralık_dışı_renk_kanalı: bool,
    /// Opaklık şeridi (`inRange.opacity`). Tek değer iki uca aynı değer
    /// yazılarak sabit opaklık olarak temsil edilir.
    pub opaklık: Option<[f32; 2]>,
    /// Denetleyici çubuğuna özgü `controller.inRange.color`.
    pub denetleyici_renkleri: Option<Vec<Renk>>,
    /// Denetleyici çubuğuna özgü `controller.inRange.opacity`.
    pub denetleyici_opaklığı: Option<[f32; 2]>,
    /// Denetleyici çubuğunun `controller.outOfRange.color` zemini.
    pub denetleyici_aralık_dışı_renk: Option<Renk>,
    /// `visualMap.dimension`: seri koordinat değerinden farklı bir dataset
    /// boyutu da renk kanalını sürebilir.
    pub boyut: Option<BoyutSeçici>,
    /// `seriesIndex`; boş liste tüm serileri hedefler.
    pub seri_sıraları: Vec<usize>,
    /// Bileşen yönü (`orient`).
    pub yön: Yön,
    /// Denetleyici öğesinin kısa kenarı (`itemWidth`). `None`, sürekli
    /// visualMap öntanımlısı olan 20 pikseli kullanır.
    pub öğe_genişliği: Option<f32>,
    /// Denetleyici öğesinin uzun kenarı (`itemHeight`). `None`, sürekli
    /// visualMap öntanımlısı olan 140 pikseli kullanır.
    pub öğe_yüksekliği: Option<f32>,
    /// Uç metni ile denetleyici arasındaki boşluk (`textGap`).
    pub metin_boşluğu: f32,
    pub sol: YatayKonum,
    /// Sağ kenardan uzaklık (`right`). Verildiğinde `left` yerleşiminin
    /// önüne geçer.
    pub sağ: Option<Uzunluk>,
    /// Üst kenardan uzaklık (`top`). Verildiğinde `bottom` yerleşiminin
    /// önüne geçer.
    pub üst: Option<Uzunluk>,
    /// `top: 'top' | 'center' | 'bottom'` anahtar sözcüklerinden biri.
    /// Sayısal/yüzde `top` değerleri geriye dönük uyum için `üst`te tutulur.
    pub dikey_konum: Option<DikeyKonum>,
    pub alt: Uzunluk,
    /// `(yüksek, düşük)` uç metinleri (`text`). `None`, sürekli ve
    /// hesaplanamaz denetleyicide uç etiketlerini gizler.
    pub metin: Option<(String, String)>,
    /// Bileşen (gradyan çubuğu) çizilsin mi?
    pub göster: bool,
    /// Sürekli eşlemede iki tutamaçlı değer aralığı denetimi
    /// (`visualMap.calculable`).
    pub hesaplanabilir: bool,
    /// Sürükleme/seçim sırasında hedef görseller anlık güncellensin mi
    /// (`visualMap.realtime`). Parçalı ve gizli eşlemelerde de seçenek
    /// değeri kayıpsız korunur.
    pub gerçek_zamanlı: bool,
    /// Sürekli eşlemenin seçili veri aralığı (`visualMap.range`). `None`,
    /// bütün etkin kapsamın seçili olduğu otomatik durumdur.
    pub seçili_aralık: Option<[f64; 2]>,
    /// Parçalı kip (`visualMap: piecewise`): boş değilse renkler
    /// parçalardan çözülür ve bileşen tıklanabilir bir listedir.
    pub parçalar: Vec<EşlemeParçası>,
    /// `visualMap-piecewise.categories`: veri boyutundaki metin değerlerini
    /// sıra korumalı kategorilere eşler. ECharts'ta `categories`, `pieces`
    /// ve `splitNumber` kipleri birbirini dışlar.
    pub kategoriler: Vec<String>,
    /// `visualMap-piecewise.splitNumber`: eşit aralıklı otomatik parça
    /// sayısı. `Some` olması parçalı kipi etkinleştirir.
    pub bölme_sayısı: Option<usize>,
    /// Parça sınırı etiketlerinin en az ondalık basamak sayısı
    /// (`visualMap.precision`). Bölme adımı gerekirse ECharts gibi en çok
    /// beş basamağa kadar otomatik artırılır.
    pub hassasiyet: usize,
    /// `minOpen`: `min` altındaki değerler için açık uçlu ek parça.
    pub en_az_açık: bool,
    /// `maxOpen`: `max` üstündeki değerler için açık uçlu ek parça.
    pub en_çok_açık: bool,
    /// Parça seçim durumu (kapalı parçaların verisi çizilmez); boşsa hepsi
    /// açıktır. Çalışma zamanında bileşen tıklamalarıyla değişir.
    pub kapalı_parçalar: Vec<usize>,
}

impl Default for GörselEşleme {
    fn default() -> Self {
        GörselEşleme {
            en_az: None,
            en_çok: None,
            // ECharts 6 globalDefault.gradientColor: ilk tema renginin
            // HSL açıklığı 0.9 olan tonu → ilk tema rengi.
            renkler: vec![
                Renk::onaltılık(0x5070dd).açıklık_ile(0.9),
                Renk::onaltılık(0x5070dd),
            ],
            aralık_dışı_renk: Renk::SAYDAM,
            renk_açıklığı: None,
            renk_kanalı: true,
            renk_kanalı_açıkça_ayarlandı: false,
            sembol_boyutu: None,
            aralık_dışı_sembol_boyutu: None,
            aralık_dışı_renk_kanalı: false,
            opaklık: None,
            denetleyici_renkleri: None,
            denetleyici_opaklığı: None,
            denetleyici_aralık_dışı_renk: None,
            boyut: None,
            seri_sıraları: Vec::new(),
            yön: Yön::Dikey,
            öğe_genişliği: None,
            öğe_yüksekliği: None,
            metin_boşluğu: 10.0,
            sol: YatayKonum::Değer(Uzunluk::Piksel(10.0)),
            sağ: None,
            üst: None,
            dikey_konum: None,
            alt: Uzunluk::Piksel(10.0),
            metin: None,
            göster: true,
            hesaplanabilir: false,
            gerçek_zamanlı: true,
            seçili_aralık: None,
            parçalar: Vec::new(),
            kategoriler: Vec::new(),
            bölme_sayısı: None,
            hassasiyet: 0,
            en_az_açık: false,
            en_çok_açık: false,
            kapalı_parçalar: Vec::new(),
        }
    }
}

impl GörselEşleme {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn en_az(mut self, değer: f64) -> Self {
        self.en_az = Some(değer);
        self
    }

    pub fn en_çok(mut self, değer: f64) -> Self {
        self.en_çok = Some(değer);
        self
    }

    pub fn renkler<R: Into<Renk>>(mut self, renkler: impl IntoIterator<Item = R>) -> Self {
        self.renkler = renkler.into_iter().map(Into::into).collect();
        self.renk_kanalı = true;
        self.renk_kanalı_açıkça_ayarlandı = true;
        self
    }

    pub fn aralık_dışı_renk(mut self, renk: impl Into<Renk>) -> Self {
        self.aralık_dışı_renk = renk.into();
        self.aralık_dışı_renk_kanalı = true;
        self
    }

    /// `inRange.colorLightness: [düşük, yüksek]`.
    pub fn renk_açıklığı(mut self, düşük: f32, yüksek: f32) -> Self {
        self.renk_açıklığı = Some([düşük.clamp(0.0, 1.0), yüksek.clamp(0.0, 1.0)]);
        if !self.renk_kanalı_açıkça_ayarlandı {
            self.renk_kanalı = false;
        }
        self
    }

    /// `inRange.symbolSize: [düşük, yüksek]`. Açık bir renk şeridi ayrıca
    /// isteniyorsa aynı zincirde [`Self::renkler`] de çağrılabilir.
    pub fn sembol_boyutu(mut self, düşük: f32, yüksek: f32) -> Self {
        self.sembol_boyutu = Some([düşük.max(0.0), yüksek.max(0.0)]);
        if !self.renk_kanalı_açıkça_ayarlandı {
            self.renk_kanalı = false;
        }
        self
    }

    /// `outOfRange.symbolSize: [düşük, yüksek]`.
    pub fn aralık_dışı_sembol_boyutu(mut self, düşük: f32, yüksek: f32) -> Self {
        self.aralık_dışı_sembol_boyutu = Some([düşük.max(0.0), yüksek.max(0.0)]);
        self
    }

    /// Tam `inRange.color` kanalını açıkça açar ya da kapatır. Çoğu kullanım
    /// için [`Self::renkler`] ve kısmi kanal kurucuları bunu kendiliğinden
    /// yönetir.
    pub fn renk_kanalı(mut self, açık: bool) -> Self {
        self.renk_kanalı = açık;
        self.renk_kanalı_açıkça_ayarlandı = true;
        self
    }

    /// `inRange.opacity: değer`.
    pub fn opaklık(mut self, opaklık: f32) -> Self {
        let opaklık = opaklık.clamp(0.0, 1.0);
        self.opaklık = Some([opaklık, opaklık]);
        if !self.renk_kanalı_açıkça_ayarlandı {
            self.renk_kanalı = false;
        }
        self
    }

    /// `inRange.opacity: [düşük, yüksek]`.
    pub fn opaklık_aralığı(mut self, düşük: f32, yüksek: f32) -> Self {
        self.opaklık = Some([düşük.clamp(0.0, 1.0), yüksek.clamp(0.0, 1.0)]);
        if !self.renk_kanalı_açıkça_ayarlandı {
            self.renk_kanalı = false;
        }
        self
    }

    pub fn denetleyici_renkleri<R: Into<Renk>>(
        mut self,
        renkler: impl IntoIterator<Item = R>,
    ) -> Self {
        self.denetleyici_renkleri = Some(renkler.into_iter().map(Into::into).collect());
        self
    }

    pub fn denetleyici_opaklık_aralığı(mut self, düşük: f32, yüksek: f32) -> Self {
        self.denetleyici_opaklığı = Some([düşük.clamp(0.0, 1.0), yüksek.clamp(0.0, 1.0)]);
        self
    }

    pub fn denetleyici_aralık_dışı_renk(mut self, renk: impl Into<Renk>) -> Self {
        self.denetleyici_aralık_dışı_renk = Some(renk.into());
        self
    }

    pub fn boyut(mut self, boyut: impl Into<BoyutSeçici>) -> Self {
        self.boyut = Some(boyut.into());
        self
    }

    pub fn seri_sırası(mut self, sıra: usize) -> Self {
        self.seri_sıraları = vec![sıra];
        self
    }

    pub fn seri_sıraları(mut self, sıralar: impl IntoIterator<Item = usize>) -> Self {
        self.seri_sıraları = sıralar.into_iter().collect();
        self
    }

    pub fn seriye_uygulanır_mı(&self, sıra: usize) -> bool {
        self.seri_sıraları.is_empty() || self.seri_sıraları.contains(&sıra)
    }

    pub fn yön(mut self, yön: Yön) -> Self {
        self.yön = yön;
        self
    }

    pub fn öğe_genişliği(mut self, genişlik: f32) -> Self {
        self.öğe_genişliği = genişlik.is_finite().then_some(genişlik.max(0.0));
        self
    }

    pub fn öğe_yüksekliği(mut self, yükseklik: f32) -> Self {
        self.öğe_yüksekliği = yükseklik.is_finite().then_some(yükseklik.max(0.0));
        self
    }

    pub fn metin_boşluğu(mut self, boşluk: f32) -> Self {
        self.metin_boşluğu = if boşluk.is_finite() {
            boşluk.max(0.0)
        } else {
            10.0
        };
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

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = alt.into();
        self.üst = None;
        self.dikey_konum = None;
        self
    }

    pub fn üst(mut self, üst: impl Into<DikeyKonum>) -> Self {
        match üst.into() {
            DikeyKonum::Değer(uzunluk) => {
                self.üst = Some(uzunluk);
                self.dikey_konum = None;
            }
            konum => {
                self.üst = None;
                self.dikey_konum = Some(konum);
            }
        }
        self
    }

    pub fn metin(mut self, yüksek: impl Into<String>, düşük: impl Into<String>) -> Self {
        self.metin = Some((yüksek.into(), düşük.into()));
        self
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn hesaplanabilir(mut self, hesaplanabilir: bool) -> Self {
        self.hesaplanabilir = hesaplanabilir;
        self
    }

    pub fn gerçek_zamanlı(mut self, gerçek_zamanlı: bool) -> Self {
        self.gerçek_zamanlı = gerçek_zamanlı;
        self
    }

    pub fn seçili_aralık(mut self, en_az: f64, en_çok: f64) -> Self {
        self.seçili_aralık = Some([en_az.min(en_çok), en_az.max(en_çok)]);
        self
    }

    pub fn seçili_aralığı_temizle(mut self) -> Self {
        self.seçili_aralık = None;
        self
    }

    /// Parçalı kip: dilim listesi.
    pub fn parçalar(mut self, parçalar: impl IntoIterator<Item = EşlemeParçası>) -> Self {
        self.parçalar = parçalar.into_iter().collect();
        self.kategoriler.clear();
        self.bölme_sayısı = None;
        self
    }

    /// Sıra korumalı kategorik parçalı kip (`categories`). `inRange.color`
    /// dizisinin aynı sırasındaki renk kategoriye doğrudan uygulanır.
    pub fn kategoriler<S>(mut self, kategoriler: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        self.kategoriler = kategoriler.into_iter().map(Into::into).collect();
        self.parçalar.clear();
        self.bölme_sayısı = None;
        self
    }

    /// Eşit aralıklı otomatik parçalı kip (`splitNumber`).
    pub fn bölme_sayısı(mut self, sayı: usize) -> Self {
        self.bölme_sayısı = Some(sayı.max(1));
        self.parçalar.clear();
        self.kategoriler.clear();
        self
    }

    pub fn hassasiyet(mut self, basamak: usize) -> Self {
        self.hassasiyet = basamak.min(20);
        self
    }

    pub fn en_az_açık(mut self, açık: bool) -> Self {
        self.en_az_açık = açık;
        self
    }

    pub fn en_çok_açık(mut self, açık: bool) -> Self {
        self.en_çok_açık = açık;
        self
    }

    /// `visualMap-piecewise.selected[index]` karşılığı.
    pub fn parça_seçimi(mut self, sıra: usize, seçili: bool) -> Self {
        if seçili {
            self.kapalı_parçalar.retain(|parça| *parça != sıra);
        } else if !self.kapalı_parçalar.contains(&sıra) {
            self.kapalı_parçalar.push(sıra);
        }
        self
    }

    /// Parçalı kip mi?
    pub fn parçalı_mı(&self) -> bool {
        !self.parçalar.is_empty() || !self.kategoriler.is_empty() || self.bölme_sayısı.is_some()
    }

    /// `categories` kullanan kategorik parçalı kip mi?
    pub fn kategorik_mi(&self) -> bool {
        !self.kategoriler.is_empty()
    }

    /// Parça açık mı (bileşende kapatılmamış mı)?
    pub fn parça_açık_mı(&self, sıra: usize) -> bool {
        !self.kapalı_parçalar.contains(&sıra)
    }

    /// Çözülmüş parça sayısı; otomatik açık uç parçalarını da kapsar.
    pub fn parça_sayısı(&self) -> usize {
        if !self.parçalar.is_empty() {
            self.parçalar.len()
        } else if !self.kategoriler.is_empty() {
            self.kategoriler.len()
        } else {
            self.bölme_sayısı
                .unwrap_or(0)
                .saturating_add(usize::from(self.en_az_açık))
                .saturating_add(usize::from(self.en_çok_açık))
        }
    }

    /// Değerin düştüğü parça sırası.
    pub fn parça_bul(&self, değer: f64) -> Option<usize> {
        if self.kategorik_mi() {
            return self.kategori_bul(&VeriDeğeri::Sayı(değer));
        }
        self.parçalar.iter().position(|p| p.içeriyor_mu(değer))
    }

    /// Ham veri değerinin `categories` dizisindeki sırası. JavaScript nesne
    /// anahtarları gibi metin, sayı, zaman ve mantıksal değerler metinsel
    /// anahtarla karşılaştırılır; bileşik/boş değerler kategori değildir.
    pub fn kategori_bul(&self, değer: &VeriDeğeri) -> Option<usize> {
        let anahtar = match değer {
            VeriDeğeri::Metin(metin) => Cow::Borrowed(metin.as_str()),
            VeriDeğeri::Sayı(sayı) => Cow::Owned(sayı.to_string()),
            VeriDeğeri::Zaman(ms) => Cow::Owned(ms.to_string()),
            VeriDeğeri::Mantıksal(mantıksal) => Cow::Owned(mantıksal.to_string()),
            VeriDeğeri::Boş | VeriDeğeri::Çift(_) | VeriDeğeri::Dizi(_) => return None,
        };
        self.kategoriler
            .iter()
            .position(|kategori| kategori == anahtar.as_ref())
    }

    /// Değerin düştüğü parça sırası. Otomatik `splitNumber` parçalarını da
    /// etkin min/max kapsamıyla çözer.
    pub fn parça_bul_kapsamda(&self, değer: f64, kapsam: [f64; 2]) -> Option<usize> {
        if self.kategorik_mi() {
            return self.kategori_bul(&VeriDeğeri::Sayı(değer));
        }
        if !self.parçalar.is_empty() {
            return self.parça_bul(değer);
        }
        let (kapsam, sayı, adım, _, alt_farkı, _) = self.otomatik_bölme(kapsam)?;
        if değer < kapsam[0] {
            return self.en_az_açık.then_some(0);
        }
        if değer > kapsam[1] {
            return self.en_çok_açık.then_some(alt_farkı.saturating_add(sayı));
        }
        for sıra in 0..sayı {
            let alt = kapsam[0] + sıra as f64 * adım;
            let üst = if sıra + 1 == sayı {
                kapsam[1]
            } else {
                alt + adım
            };
            let alt_uyuyor = if sıra == 0 {
                değer >= alt
            } else {
                değer > alt
            };
            if alt_uyuyor && değer <= üst {
                return Some(alt_farkı + sıra);
            }
        }
        None
    }

    /// ECharts `PiecewiseModel` tarafından üretilen düşükten yükseğe parça
    /// listesini döndürür. Açık/kapalı ortak sınırlar, otomatik hassasiyet,
    /// etiketler ve parça sıra renkleri resmî modelle aynıdır.
    pub fn parçaları_çöz(&self, kapsam: [f64; 2]) -> Vec<EşlemeParçası> {
        if !self.parçalar.is_empty() {
            return self.parçalar.clone();
        }
        if self.kategorik_mi() {
            return self
                .kategoriler
                .iter()
                .enumerate()
                .map(|(sıra, kategori)| {
                    let renk = self
                        .renkler
                        .get(sıra)
                        .copied()
                        .unwrap_or(self.aralık_dışı_renk);
                    EşlemeParçası::değer(sıra as f64, renk).etiket(kategori)
                })
                .collect();
        }
        let Some((kapsam, sayı, adım, hassasiyet, _, toplam)) = self.otomatik_bölme(kapsam)
        else {
            return Vec::new();
        };
        let mut sonuç = Vec::with_capacity(toplam);
        if self.en_az_açık {
            sonuç.push(
                EşlemeParçası::aralık(None, false, Some(kapsam[0]), false, Renk::SİYAH)
                    .etiket(format!("< {:.*}", hassasiyet, kapsam[0])),
            );
        }
        for sıra in 0..sayı {
            let alt = kapsam[0] + sıra as f64 * adım;
            let üst = if sıra + 1 == sayı {
                kapsam[1]
            } else {
                alt + adım
            };
            sonuç.push(
                EşlemeParçası::aralık(Some(alt), sıra == 0, Some(üst), true, Renk::SİYAH)
                    .etiket(format!("{:.*} - {:.*}", hassasiyet, alt, hassasiyet, üst)),
            );
        }
        if self.en_çok_açık {
            sonuç.push(
                EşlemeParçası::aralık(Some(kapsam[1]), false, None, false, Renk::SİYAH)
                    .etiket(format!("> {:.*}", hassasiyet, kapsam[1])),
            );
        }
        let payda = sonuç.len().saturating_sub(1);
        for (sıra, parça) in sonuç.iter_mut().enumerate() {
            let oran = if payda == 0 {
                0.5
            } else {
                sıra as f32 / payda as f32
            };
            parça.renk = self.şerit_rengi(oran);
        }
        sonuç
    }

    /// Etkin eşleme kapsamı: seçenek sınırları veri kapsamıyla birleşir.
    pub fn kapsam_çöz(&self, veri_kapsamı: [f64; 2]) -> [f64; 2] {
        let en_az = self.en_az.unwrap_or(veri_kapsamı[0]);
        let en_çok = self.en_çok.unwrap_or(veri_kapsamı[1]);
        if en_çok > en_az {
            [en_az, en_çok]
        } else {
            [en_az, en_az + 1.0]
        }
    }

    /// `range` değerini etkin min/max kapsamına sıralayıp kıstırır.
    pub fn seçili_kapsam(&self, kapsam: [f64; 2]) -> [f64; 2] {
        let [mut alt, mut üst] = self.seçili_aralık.unwrap_or(kapsam);
        if alt > üst {
            std::mem::swap(&mut alt, &mut üst);
        }
        [
            alt.clamp(kapsam[0], kapsam[1]),
            üst.clamp(kapsam[0], kapsam[1]),
        ]
    }

    pub fn seçili_mi(&self, değer: f64, kapsam: [f64; 2]) -> bool {
        let [alt, üst] = self.seçili_kapsam(kapsam);
        // ECharts `unboundedRange: true` öntanımlısı: tutamaç kapsamın bir
        // ucundaysa o yöndeki min/max dışı değerler de seçili kalır.
        (alt <= kapsam[0] || değer >= alt) && (üst >= kapsam[1] || değer <= üst)
    }

    fn doğrusal_oran(&self, değer: f64, kapsam: [f64; 2]) -> f32 {
        if kapsam[1] > kapsam[0] {
            ((değer - kapsam[0]) / (kapsam[1] - kapsam[0])).clamp(0.0, 1.0) as f32
        } else {
            0.0
        }
    }

    /// `symbolSize` kanalını seri tarafından hesaplanan taban çapa uygular.
    /// Seçili aralık dışında açık `outOfRange.symbolSize` yoksa taban çap
    /// korunur.
    pub fn sembol_boyutu_çöz(&self, değer: f64, kapsam: [f64; 2], taban: f32) -> f32 {
        let aralık = if self.seçili_mi(değer, kapsam) {
            self.sembol_boyutu
        } else {
            self.aralık_dışı_sembol_boyutu
        };
        let Some([düşük, yüksek]) = aralık else {
            return taban;
        };
        düşük + (yüksek - düşük) * self.doğrusal_oran(değer, kapsam)
    }

    /// Değeri renge çözer: parçalı kipte ilgili dilimin rengi, sürekli
    /// kipte şeritte çok duraklı doğrusal ara değerleme.
    pub fn renk_çöz(&self, değer: f64, kapsam: [f64; 2]) -> Renk {
        if self.kategorik_mi() {
            return self.kategori_rengi_uygula(&VeriDeğeri::Sayı(değer), Renk::SAYDAM);
        }
        if self.parçalı_mı() {
            let Some(sıra) = self
                .parça_bul_kapsamda(değer, kapsam)
                .filter(|sıra| self.parça_açık_mı(*sıra))
            else {
                return self.aralık_dışı_renk;
            };
            if let Some(parça) = self.parçalar.get(sıra) {
                return self.opaklığı_uygula(parça.renk, değer, kapsam);
            }
            let toplam = self
                .bölme_sayısı
                .unwrap_or(1)
                .saturating_add(usize::from(self.en_az_açık))
                .saturating_add(usize::from(self.en_çok_açık));
            let oran = if toplam <= 1 {
                0.5
            } else {
                sıra as f32 / (toplam - 1) as f32
            };
            return self.opaklığı_uygula(self.şerit_rengi(oran), değer, kapsam);
        }
        let oran = self.doğrusal_oran(değer, kapsam);
        self.opaklığı_uygula(self.şerit_rengi(oran), değer, kapsam)
    }

    fn şerit_rengi(&self, oran: f32) -> Renk {
        renk_dizisi_karıştır(&self.renkler, oran)
    }

    /// Controller görselini veri üzerindeki `inRange` kanallarından bağımsız
    /// çözer. Açık controller kanalı yoksa ECharts gibi veri kanalına düşer.
    pub fn denetleyici_rengi(&self, oran: f32) -> Renk {
        let renkler = self
            .denetleyici_renkleri
            .as_deref()
            .unwrap_or(&self.renkler);
        let mut renk = renk_dizisi_karıştır(renkler, oran);
        if let Some([düşük, yüksek]) = self.renk_açıklığı {
            renk = renk.açıklık_ile(düşük + (yüksek - düşük) * oran.clamp(0.0, 1.0));
        }
        let opaklık = self.denetleyici_opaklığı.or(self.opaklık);
        match opaklık {
            Some([düşük, yüksek]) => {
                renk.opaklık(düşük + (yüksek - düşük) * oran.clamp(0.0, 1.0))
            }
            None => renk,
        }
    }

    /// Controller renk/opaklık gradyanını temsil etmek için gereken durak
    /// sayısı (tek renk + değişken opaklıkta iki uç gerekir).
    pub fn denetleyici_durak_sayısı(&self) -> usize {
        let renk_durakları = self
            .denetleyici_renkleri
            .as_ref()
            .map(Vec::len)
            .unwrap_or(self.renkler.len());
        let opaklık_durakları = if self.denetleyici_opaklığı.or(self.opaklık).is_some() {
            2
        } else {
            1
        };
        let açıklık_durakları = if self.renk_açıklığı.is_some() {
            2
        } else {
            1
        };
        renk_durakları
            .max(opaklık_durakları)
            .max(açıklık_durakları)
            .max(1)
    }

    fn otomatik_bölme(
        &self,
        veri_kapsamı: [f64; 2],
    ) -> Option<([f64; 2], usize, f64, usize, usize, usize)> {
        let sayı = self.bölme_sayısı?;
        let kapsam = self.kapsam_çöz(veri_kapsamı);
        let ham_adım = (kapsam[1] - kapsam[0]) / sayı as f64;
        let mut hassasiyet = self.hassasiyet.min(20);
        while yuvarla(ham_adım, hassasiyet) != ham_adım && hassasiyet < 5 {
            hassasiyet += 1;
        }
        let adım = yuvarla(ham_adım, hassasiyet);
        let alt_farkı = usize::from(self.en_az_açık);
        let toplam = sayı
            .saturating_add(alt_farkı)
            .saturating_add(usize::from(self.en_çok_açık));
        Some((kapsam, sayı, adım, hassasiyet, alt_farkı, toplam))
    }

    /// Görsel kanalları mevcut öğe rengine ECharts sırasıyla uygular. Tam
    /// `color` kanalı taban rengi değiştirir; `colorLightness` ve `opacity`
    /// ise sonuç üzerindeki kısmi kanallardır.
    pub fn rengi_uygula(&self, değer: f64, kapsam: [f64; 2], taban: Renk) -> Renk {
        if self.kategorik_mi() {
            return self.kategori_rengi_uygula(&VeriDeğeri::Sayı(değer), taban);
        }
        let etkin = self.seçili_mi(değer, kapsam)
            && (!self.parçalı_mı()
                || self
                    .parça_bul_kapsamda(değer, kapsam)
                    .is_some_and(|sıra| self.parça_açık_mı(sıra)));
        if !etkin {
            return if self.aralık_dışı_renk_kanalı {
                self.aralık_dışı_renk
            } else if self.renk_kanalı {
                self.aralık_dışı_renk
            } else {
                taban
            };
        }
        let oran = self.doğrusal_oran(değer, kapsam);
        let mut renk = if self.renk_kanalı {
            self.renk_çöz(değer, kapsam)
        } else {
            taban
        };
        if let Some([düşük, yüksek]) = self.renk_açıklığı {
            renk = renk.açıklık_ile(düşük + (yüksek - düşük) * oran);
        }
        self.opaklığı_uygula(renk, değer, kapsam)
    }

    /// Kategorik parçalı eşlemenin renk kanalını ham veri değerine uygular.
    /// `categories[i]`, `inRange.color[i]` ile doğrudan eşlenir; bilinmeyen,
    /// kapalı veya karşılık rengi bulunmayan kategori `outOfRange` kanalına
    /// düşer.
    pub fn kategori_rengi_uygula(&self, değer: &VeriDeğeri, taban: Renk) -> Renk {
        let sıra = self
            .kategori_bul(değer)
            .filter(|sıra| self.parça_açık_mı(*sıra));
        let Some(renk) = sıra.and_then(|sıra| self.renkler.get(sıra)).copied() else {
            return if self.aralık_dışı_renk_kanalı || self.renk_kanalı {
                self.aralık_dışı_renk
            } else {
                taban
            };
        };
        if self.renk_kanalı { renk } else { taban }
    }

    /// Geriye uyumlu ad; yeni kodda [`Self::rengi_uygula`] tercih edilir.
    pub fn renk_çöz_tabanla(&self, değer: f64, kapsam: [f64; 2], taban: Renk) -> Renk {
        self.rengi_uygula(değer, kapsam, taban)
    }

    fn opaklığı_uygula(&self, renk: Renk, değer: f64, kapsam: [f64; 2]) -> Renk {
        let Some([düşük, yüksek]) = self.opaklık else {
            return renk;
        };
        let oran = self.doğrusal_oran(değer, kapsam);
        renk.opaklık(düşük + (yüksek - düşük) * oran)
    }
}

fn renk_dizisi_karıştır(renkler: &[Renk], oran: f32) -> Renk {
    let (Some(ilk), Some(son)) = (renkler.first(), renkler.last()) else {
        return Renk::SİYAH;
    };
    if renkler.len() == 1 {
        return *ilk;
    }
    let oran = oran.clamp(0.0, 1.0);
    if oran >= 1.0 {
        return *son;
    }
    let bölme_sayısı = renkler.len() - 1;
    let konum = oran * bölme_sayısı as f32;
    let alt = (konum.floor() as usize).min(bölme_sayısı.saturating_sub(1));
    let t = konum - alt as f32;
    match (renkler.get(alt), renkler.get(alt + 1)) {
        (Some(a), Some(b)) => a.karıştır(*b, t),
        _ => *ilk,
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

    #[test]
    fn uç_renkler() {
        let e = GörselEşleme::yeni().renkler([0x000000u32, 0xffffffu32]);
        let kapsam = [0.0, 10.0];
        assert_eq!(e.renk_çöz(0.0, kapsam), Renk::onaltılık(0x000000));
        assert_eq!(e.renk_çöz(10.0, kapsam), Renk::onaltılık(0xffffff));
    }

    #[test]
    fn orta_karışım() {
        let e = GörselEşleme::yeni().renkler([0x000000u32, 0xffffffu32]);
        let orta = e.renk_çöz(5.0, [0.0, 10.0]);
        assert!((orta.kırmızı - 0.5).abs() < 1e-4);
    }

    #[test]
    fn açıklık_kanalı_taban_rengin_tonunu_korur() {
        let e = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(10.0)
            .renk_açıklığı(0.0, 1.0);
        let taban = Renk::onaltılık(0xc23531);
        assert_eq!(e.renk_çöz_tabanla(0.0, [0.0, 10.0], taban), Renk::SİYAH);
        assert_eq!(e.renk_çöz_tabanla(10.0, [0.0, 10.0], taban), Renk::BEYAZ);
        let orta = e.renk_çöz_tabanla(5.0, [0.0, 10.0], taban);
        assert!(orta.kırmızı > orta.yeşil && orta.yeşil > orta.mavi);
    }

    #[test]
    fn sembol_boyutu_kanalı_renk_şeridini_kendiliğinden_eklemez() {
        let eşleme = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(100.0)
            .sembol_boyutu(10.0, 70.0);
        let taban = Renk::onaltılık(0xdd4444);

        assert!(!eşleme.renk_kanalı);
        assert_eq!(eşleme.rengi_uygula(50.0, [0.0, 100.0], taban), taban);
        assert!((eşleme.sembol_boyutu_çöz(50.0, [0.0, 100.0], 4.0) - 40.0).abs() < 1e-6);
        assert_eq!(eşleme.sembol_boyutu_çöz(200.0, [0.0, 100.0], 4.0), 70.0);
    }

    #[test]
    fn açık_renk_ve_sembol_boyutu_kanalları_birlikte_kalır() {
        let eşleme = GörselEşleme::yeni()
            .renkler([0x000000u32, 0xffffffu32])
            .sembol_boyutu(10.0, 70.0);

        assert!(eşleme.renk_kanalı);
        let orta = eşleme.rengi_uygula(5.0, [0.0, 10.0], Renk::onaltılık(0xdd4444));
        assert!((orta.kırmızı - 0.5).abs() < 1e-4);
        assert!((eşleme.sembol_boyutu_çöz(5.0, [0.0, 10.0], 4.0) - 40.0).abs() < 1e-6);
    }

    #[test]
    fn opaklık_kanalı_sabit_ve_aralıklı_değerleri_renge_uygular() {
        let sabit = GörselEşleme::yeni()
            .renkler([0x006eddu32])
            .opaklık(0.3)
            .renk_çöz(150.0, [0.0, 300.0]);
        assert!((sabit.alfa - 0.3).abs() < 1e-6);

        let yüksek = GörselEşleme::yeni()
            .renkler([0x006eddu32])
            .opaklık_aralığı(0.2, 0.8)
            .renk_çöz(300.0, [0.0, 300.0]);
        assert!((yüksek.alfa - 0.8).abs() < 1e-6);
    }

    #[test]
    fn denetleyici_kanalları_veri_görselinden_bağımsız_çözülür() {
        let eşleme = GörselEşleme::yeni()
            .renkler(["grey"])
            .opaklık_aralığı(0.0, 0.3)
            .denetleyici_renkleri(["red", "blue"])
            .denetleyici_opaklık_aralığı(0.3, 0.6)
            .denetleyici_aralık_dışı_renk("#ccc");

        let veri_sonu = eşleme.renk_çöz(1000.0, [0.0, 1000.0]);
        let denetleyici_başı = eşleme.denetleyici_rengi(0.0);
        let denetleyici_sonu = eşleme.denetleyici_rengi(1.0);
        assert_eq!(veri_sonu, Renk::from("grey").opaklık(0.3));
        assert_eq!(denetleyici_başı, Renk::from("red").opaklık(0.3));
        assert_eq!(denetleyici_sonu, Renk::from("blue").opaklık(0.6));
        assert_eq!(eşleme.denetleyici_durak_sayısı(), 2);
        assert_eq!(
            eşleme.denetleyici_aralık_dışı_renk,
            Some(Renk::from("#ccc"))
        );
    }

    #[test]
    fn parçalı_eşleme_kapalı_sınırları_ve_kesin_değeri_ayırt_eder() {
        let kırmızı = Renk::onaltılık(0xff0000);
        let yeşil = Renk::onaltılık(0x00ff00);
        let eşleme = GörselEşleme::yeni().parçalar([
            EşlemeParçası::aralık(Some(1.0), false, Some(3.0), false, kırmızı),
            EşlemeParçası::değer(3.0, yeşil),
        ]);

        assert_eq!(eşleme.renk_çöz(1.0, [0.0, 4.0]), Renk::SAYDAM);
        assert_eq!(eşleme.renk_çöz(2.0, [0.0, 4.0]), kırmızı);
        assert_eq!(eşleme.renk_çöz(3.0, [0.0, 4.0]), yeşil);
        assert_eq!(eşleme.parçalar[0].etiket_metni(), "(1 – 3)");
        assert_eq!(eşleme.parçalar[1].etiket_metni(), "3");
    }

    #[test]
    fn kategorik_piecewise_metni_sira_rengine_esler_ve_secimi_uygular() {
        let taban = Renk::onaltılık(0x112233);
        let eşleme = GörselEşleme::yeni()
            .kategoriler(["süt", "baharat", "yağ"])
            .renkler([0xdf5a5au32, 0xdf775a, 0xdf945a])
            .aralık_dışı_renk("#ccc")
            .parça_seçimi(1, false);

        assert!(eşleme.parçalı_mı());
        assert!(eşleme.kategorik_mi());
        assert_eq!(eşleme.parça_sayısı(), 3);
        assert_eq!(
            eşleme.kategori_rengi_uygula(&VeriDeğeri::Metin("süt".to_owned()), taban),
            Renk::onaltılık(0xdf5a5a)
        );
        assert_eq!(
            eşleme.kategori_rengi_uygula(&VeriDeğeri::Metin("baharat".to_owned()), taban),
            Renk::onaltılık(0xcccccc)
        );
        assert_eq!(
            eşleme.kategori_rengi_uygula(&VeriDeğeri::Metin("bilinmeyen".to_owned()), taban),
            Renk::onaltılık(0xcccccc)
        );
        let parçalar = eşleme.parçaları_çöz([0.0, 1.0]);
        assert_eq!(parçalar[0].etiket_metni(), "süt");
        assert_eq!(parçalar[2].renk, Renk::onaltılık(0xdf945a));
    }

    #[test]
    fn kategorik_kurucu_diger_piecewise_kiplerini_dislar() {
        let kategorik = GörselEşleme::yeni().bölme_sayısı(5).kategoriler(["A", "B"]);
        assert_eq!(kategorik.bölme_sayısı, None);
        assert!(kategorik.parçalar.is_empty());

        let sayısal = kategorik.parçalar([EşlemeParçası::değer(1.0, "red")]);
        assert!(sayısal.kategoriler.is_empty());
        assert_eq!(sayısal.parça_sayısı(), 1);
    }

    #[test]
    fn split_number_resmi_araliklari_etiketleri_ve_renk_sirasini_uretir() {
        let eşleme = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(1.0)
            .bölme_sayısı(8)
            .renkler([
                0x313695u32,
                0x4575b4,
                0x74add1,
                0xabd9e9,
                0xe0f3f8,
                0xffffbf,
                0xfee090,
                0xfdae61,
                0xf46d43,
                0xd73027,
                0xa50026,
            ]);
        let parçalar = eşleme.parçaları_çöz([0.0, 1.0]);

        assert_eq!(parçalar.len(), 8);
        assert_eq!(parçalar[0].etiket_metni(), "0.000 - 0.125");
        assert_eq!(parçalar[7].etiket_metni(), "0.875 - 1.000");
        assert!(parçalar[0].en_az_dahil && parçalar[0].en_çok_dahil);
        assert!(!parçalar[1].en_az_dahil && parçalar[1].en_çok_dahil);
        assert_eq!(eşleme.parça_bul_kapsamda(0.125, [0.0, 1.0]), Some(0));
        assert_eq!(eşleme.parça_bul_kapsamda(0.125_001, [0.0, 1.0]), Some(1));
        assert_eq!(eşleme.parça_bul_kapsamda(1.0, [0.0, 1.0]), Some(7));
        assert_eq!(eşleme.parça_bul_kapsamda(1.01, [0.0, 1.0]), None);

        let rgb8 = |renk: Renk| {
            [
                (renk.kırmızı * 255.0).round() as u8,
                (renk.yeşil * 255.0).round() as u8,
                (renk.mavi * 255.0).round() as u8,
            ]
        };
        assert_eq!(rgb8(parçalar[0].renk), [49, 54, 149]);
        assert_eq!(rgb8(parçalar[1].renk), [89, 141, 192]);
        assert_eq!(rgb8(parçalar[7].renk), [165, 0, 38]);
    }

    #[test]
    fn split_number_acik_uclari_ayri_parca_olarak_cozer() {
        let eşleme = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(10.0)
            .bölme_sayısı(2)
            .en_az_açık(true)
            .en_çok_açık(true);
        let parçalar = eşleme.parçaları_çöz([0.0, 10.0]);

        assert_eq!(parçalar.len(), 4);
        assert_eq!(parçalar[0].etiket_metni(), "< 0");
        assert_eq!(parçalar[3].etiket_metni(), "> 10");
        assert_eq!(eşleme.parça_bul_kapsamda(-1.0, [0.0, 10.0]), Some(0));
        assert_eq!(eşleme.parça_bul_kapsamda(0.0, [0.0, 10.0]), Some(1));
        assert_eq!(eşleme.parça_bul_kapsamda(11.0, [0.0, 10.0]), Some(3));
    }

    #[test]
    fn seri_index_süzgeci_boşken_tüm_serilere_doluiken_seçilenlere_uygulanır() {
        let tümü = GörselEşleme::yeni();
        assert!(tümü.seriye_uygulanır_mı(0));
        assert!(tümü.seriye_uygulanır_mı(42));

        let seçili = GörselEşleme::yeni().seri_sıraları([1, 3]);
        assert!(!seçili.seriye_uygulanır_mı(0));
        assert!(seçili.seriye_uygulanır_mı(1));
        assert!(seçili.seriye_uygulanır_mı(3));
    }

    #[test]
    fn sürekli_seçili_aralık_sıralanır_ve_kapsama_kıstırılır() {
        let eşleme = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(10.0)
            .hesaplanabilir(true)
            .seçili_aralık(12.0, -2.0);

        assert_eq!(eşleme.seçili_kapsam([0.0, 10.0]), [0.0, 10.0]);
        assert!(eşleme.seçili_mi(5.0, [0.0, 10.0]));
        assert!(
            !GörselEşleme::yeni()
                .seçili_aralık(2.0, 4.0)
                .seçili_mi(5.0, [0.0, 10.0])
        );
    }
}
