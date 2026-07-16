//! Görsel eşleme — ECharts `visualMap` bileşeninin çekirdeği: sayısal
//! değerleri renk şeridine eşler. Sürekli (continuous) kip; parçalı
//! (piecewise) kip Faz 3'te eklenecektir.

use crate::renk::Renk;

/// Sürekli görsel eşleme (`visualMap: { type: 'continuous' }`).
#[derive(Clone, PartialEq, Debug)]
pub struct GörselEşleme {
    /// Eşleme alt sınırı; `None` ise veri en küçüğü.
    pub en_az: Option<f64>,
    /// Eşleme üst sınırı; `None` ise veri en büyüğü.
    pub en_çok: Option<f64>,
    /// Renk şeridi, düşükten yükseğe (`inRange.color`).
    pub renkler: Vec<Renk>,
    /// Bileşen (gradyan çubuğu) çizilsin mi?
    pub göster: bool,
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
            göster: true,
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

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
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

    /// Değeri renk şeridinde çok duraklı doğrusal ara değerlemeyle çözer.
    pub fn renk_çöz(&self, değer: f64, kapsam: [f64; 2]) -> Renk {
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
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
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
}
