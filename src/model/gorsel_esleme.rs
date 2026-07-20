//! Görsel eşleme — ECharts `visualMap` bileşeninin çekirdeği: sayısal
//! değerleri renk şeridine eşler. Sürekli (`continuous`) ve parçalı
//! (`piecewise`) kipler aynı modelde tutulur.

use crate::model::bilesen::Yön;
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
    /// `visualMap.dimension`: seri koordinat değerinden farklı bir dataset
    /// boyutu da renk kanalını sürebilir.
    pub boyut: Option<BoyutSeçici>,
    /// `seriesIndex`; boş liste tüm serileri hedefler.
    pub seri_sıraları: Vec<usize>,
    /// Bileşen yönü (`orient`).
    pub yön: Yön,
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
    /// `(yüksek, düşük)` uç metinleri (`text`). Boşsa sayısal kapsam yazılır.
    pub metin: Option<(String, String)>,
    /// Bileşen (gradyan çubuğu) çizilsin mi?
    pub göster: bool,
    /// Sürekli eşlemede iki tutamaçlı değer aralığı denetimi
    /// (`visualMap.calculable`).
    pub hesaplanabilir: bool,
    /// Sürekli eşlemenin seçili veri aralığı (`visualMap.range`). `None`,
    /// bütün etkin kapsamın seçili olduğu otomatik durumdur.
    pub seçili_aralık: Option<[f64; 2]>,
    /// Parçalı kip (`visualMap: piecewise`): boş değilse renkler
    /// parçalardan çözülür ve bileşen tıklanabilir bir listedir.
    pub parçalar: Vec<EşlemeParçası>,
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
            boyut: None,
            seri_sıraları: Vec::new(),
            yön: Yön::Dikey,
            sol: YatayKonum::Değer(Uzunluk::Piksel(10.0)),
            sağ: None,
            üst: None,
            dikey_konum: None,
            alt: Uzunluk::Piksel(10.0),
            metin: None,
            göster: true,
            hesaplanabilir: false,
            seçili_aralık: None,
            parçalar: Vec::new(),
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
        self
    }

    pub fn aralık_dışı_renk(mut self, renk: impl Into<Renk>) -> Self {
        self.aralık_dışı_renk = renk.into();
        self
    }

    /// `inRange.colorLightness: [düşük, yüksek]`.
    pub fn renk_açıklığı(mut self, düşük: f32, yüksek: f32) -> Self {
        self.renk_açıklığı = Some([düşük.clamp(0.0, 1.0), yüksek.clamp(0.0, 1.0)]);
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
        self.bölme_sayısı = None;
        self
    }

    /// Eşit aralıklı otomatik parçalı kip (`splitNumber`).
    pub fn bölme_sayısı(mut self, sayı: usize) -> Self {
        self.bölme_sayısı = Some(sayı.max(1));
        self.parçalar.clear();
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
        !self.parçalar.is_empty() || self.bölme_sayısı.is_some()
    }

    /// Parça açık mı (bileşende kapatılmamış mı)?
    pub fn parça_açık_mı(&self, sıra: usize) -> bool {
        !self.kapalı_parçalar.contains(&sıra)
    }

    /// Çözülmüş parça sayısı; otomatik açık uç parçalarını da kapsar.
    pub fn parça_sayısı(&self) -> usize {
        if !self.parçalar.is_empty() {
            self.parçalar.len()
        } else {
            self.bölme_sayısı
                .unwrap_or(0)
                .saturating_add(usize::from(self.en_az_açık))
                .saturating_add(usize::from(self.en_çok_açık))
        }
    }

    /// Değerin düştüğü parça sırası.
    pub fn parça_bul(&self, değer: f64) -> Option<usize> {
        self.parçalar.iter().position(|p| p.içeriyor_mu(değer))
    }

    /// Değerin düştüğü parça sırası. Otomatik `splitNumber` parçalarını da
    /// etkin min/max kapsamıyla çözer.
    pub fn parça_bul_kapsamda(&self, değer: f64, kapsam: [f64; 2]) -> Option<usize> {
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

    /// Değeri renge çözer: parçalı kipte ilgili dilimin rengi, sürekli
    /// kipte şeritte çok duraklı doğrusal ara değerleme.
    pub fn renk_çöz(&self, değer: f64, kapsam: [f64; 2]) -> Renk {
        if self.parçalı_mı() {
            let Some(sıra) = self
                .parça_bul_kapsamda(değer, kapsam)
                .filter(|sıra| self.parça_açık_mı(*sıra))
            else {
                return self.aralık_dışı_renk;
            };
            if let Some(parça) = self.parçalar.get(sıra) {
                return parça.renk;
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
            return self.şerit_rengi(oran);
        }
        let oran = if kapsam[1] > kapsam[0] {
            ((değer - kapsam[0]) / (kapsam[1] - kapsam[0])).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        self.şerit_rengi(oran)
    }

    fn şerit_rengi(&self, oran: f32) -> Renk {
        let (Some(ilk), Some(son)) = (self.renkler.first(), self.renkler.last()) else {
            return Renk::SİYAH;
        };
        if self.renkler.len() == 1 {
            return *ilk;
        }
        let oran = oran.clamp(0.0, 1.0);
        if oran >= 1.0 {
            return *son;
        }
        let bölme_sayısı = self.renkler.len() - 1;
        let konum = oran * bölme_sayısı as f32;
        let alt = (konum.floor() as usize).min(bölme_sayısı.saturating_sub(1));
        let t = konum - alt as f32;
        match (self.renkler.get(alt), self.renkler.get(alt + 1)) {
            (Some(a), Some(b)) => a.karıştır(*b, t),
            _ => *ilk,
        }
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

    /// Görsel eşlemeyi mevcut öğe rengine uygular. `colorLightness`,
    /// ECharts'ta bağımsız bir kısmi renk kanalıdır ve seri `itemStyle.color`
    /// değerinin ton/doygunluğunu korur.
    pub fn renk_çöz_tabanla(&self, değer: f64, kapsam: [f64; 2], taban: Renk) -> Renk {
        let Some([düşük, yüksek]) = self.renk_açıklığı else {
            return self.renk_çöz(değer, kapsam);
        };
        let oran = if kapsam[1] > kapsam[0] {
            ((değer - kapsam[0]) / (kapsam[1] - kapsam[0])).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        taban.açıklık_ile(düşük + (yüksek - düşük) * oran)
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
