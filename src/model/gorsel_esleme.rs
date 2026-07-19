//! Görsel eşleme — ECharts `visualMap` bileşeninin çekirdeği: sayısal
//! değerleri renk şeridine eşler. Sürekli (continuous) kip; parçalı
//! (piecewise) kip Faz 3'te eklenecektir.

use crate::model::bilesen::Yön;
use crate::model::veri_kumesi::BoyutSeçici;
use crate::model::{Uzunluk, YatayKonum};
use crate::renk::Renk;

/// Parçalı eşlemenin tek dilimi (`visualMap-piecewise` `pieces` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct EşlemeParçası {
    /// Alt sınır (dahil); `None` = -∞.
    pub en_az: Option<f64>,
    /// Üst sınır (hariç); `None` = +∞.
    pub en_çok: Option<f64>,
    pub renk: Renk,
    pub etiket: Option<String>,
}

impl EşlemeParçası {
    pub fn yeni(en_az: Option<f64>, en_çok: Option<f64>, renk: impl Into<Renk>) -> Self {
        EşlemeParçası {
            en_az,
            en_çok,
            renk: renk.into(),
            etiket: None,
        }
    }

    pub fn etiket(mut self, etiket: impl Into<String>) -> Self {
        self.etiket = Some(etiket.into());
        self
    }

    pub fn içeriyor_mu(&self, değer: f64) -> bool {
        self.en_az.map(|a| değer >= a).unwrap_or(true)
            && self.en_çok.map(|b| değer < b).unwrap_or(true)
    }

    /// Görüntülenecek etiket.
    pub fn etiket_metni(&self) -> String {
        if let Some(e) = &self.etiket {
            return e.clone();
        }
        match (self.en_az, self.en_çok) {
            (Some(a), Some(b)) => format!("{a} – {b}"),
            (Some(a), None) => format!("≥ {a}"),
            (None, Some(b)) => format!("< {b}"),
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
    /// HSL açıklık şeridi (`inRange.colorLightness`). Bu kanal renk
    /// kanalından farklı olarak öğenin/paletin mevcut rengini değiştirir.
    pub renk_açıklığı: Option<[f32; 2]>,
    /// `visualMap.dimension`: seri koordinat değerinden farklı bir dataset
    /// boyutu da renk kanalını sürebilir.
    pub boyut: Option<BoyutSeçici>,
    /// Bileşen yönü (`orient`).
    pub yön: Yön,
    pub sol: YatayKonum,
    pub alt: Uzunluk,
    /// `(yüksek, düşük)` uç metinleri (`text`). Boşsa sayısal kapsam yazılır.
    pub metin: Option<(String, String)>,
    /// Bileşen (gradyan çubuğu) çizilsin mi?
    pub göster: bool,
    /// Parçalı kip (`visualMap: piecewise`): boş değilse renkler
    /// parçalardan çözülür ve bileşen tıklanabilir bir listedir.
    pub parçalar: Vec<EşlemeParçası>,
    /// Parça seçim durumu (kapalı parçaların verisi çizilmez); boşsa hepsi
    /// açıktır. Çalışma zamanında bileşen tıklamalarıyla değişir.
    pub kapalı_parçalar: Vec<usize>,
}

impl Default for GörselEşleme {
    fn default() -> Self {
        GörselEşleme {
            en_az: None,
            en_çok: None,
            // ECharts visualMap öntanımlı şeridi (düşük → yüksek).
            renkler: vec![
                Renk::onaltılık(0xf6efa6),
                Renk::onaltılık(0xd88273),
                Renk::onaltılık(0xbf444c),
            ],
            renk_açıklığı: None,
            boyut: None,
            yön: Yön::Dikey,
            sol: YatayKonum::Değer(Uzunluk::Piksel(10.0)),
            alt: Uzunluk::Piksel(10.0),
            metin: None,
            göster: true,
            parçalar: Vec::new(),
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

    /// `inRange.colorLightness: [düşük, yüksek]`.
    pub fn renk_açıklığı(mut self, düşük: f32, yüksek: f32) -> Self {
        self.renk_açıklığı = Some([düşük.clamp(0.0, 1.0), yüksek.clamp(0.0, 1.0)]);
        self
    }

    pub fn boyut(mut self, boyut: impl Into<BoyutSeçici>) -> Self {
        self.boyut = Some(boyut.into());
        self
    }

    pub fn yön(mut self, yön: Yön) -> Self {
        self.yön = yön;
        self
    }

    pub fn sol(mut self, sol: impl Into<YatayKonum>) -> Self {
        self.sol = sol.into();
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = alt.into();
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

    /// Parçalı kip: dilim listesi.
    pub fn parçalar(mut self, parçalar: impl IntoIterator<Item = EşlemeParçası>) -> Self {
        self.parçalar = parçalar.into_iter().collect();
        self
    }

    /// Parçalı kip mi?
    pub fn parçalı_mı(&self) -> bool {
        !self.parçalar.is_empty()
    }

    /// Parça açık mı (bileşende kapatılmamış mı)?
    pub fn parça_açık_mı(&self, sıra: usize) -> bool {
        !self.kapalı_parçalar.contains(&sıra)
    }

    /// Değerin düştüğü parça sırası.
    pub fn parça_bul(&self, değer: f64) -> Option<usize> {
        self.parçalar.iter().position(|p| p.içeriyor_mu(değer))
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

    /// Değeri renge çözer: parçalı kipte ilgili dilimin rengi, sürekli
    /// kipte şeritte çok duraklı doğrusal ara değerleme.
    pub fn renk_çöz(&self, değer: f64, kapsam: [f64; 2]) -> Renk {
        if self.parçalı_mı() {
            return self
                .parça_bul(değer)
                .and_then(|i| self.parçalar.get(i))
                .map(|p| p.renk)
                .unwrap_or(crate::tema::nötr_20());
        }
        let (Some(ilk), Some(son)) = (self.renkler.first(), self.renkler.last()) else {
            return Renk::SİYAH;
        };
        if self.renkler.len() == 1 || kapsam[1] <= kapsam[0] {
            return *ilk;
        }
        let oran = ((değer - kapsam[0]) / (kapsam[1] - kapsam[0])).clamp(0.0, 1.0) as f32;
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
}
