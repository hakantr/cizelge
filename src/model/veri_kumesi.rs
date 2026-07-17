//! Veri kümesi — ECharts `dataset` + `encode` + `transform`
//! bileşenlerinin karşılığı: seriler, ortak bir tablodan boyut adlarıyla
//! beslenir; süzme/sıralama dönüşümleri tablo üzerinde zincirlenir.

use crate::hata::BilesenHatasi;
use crate::model::deger::VeriDeğeri;

/// Sütunlu veri tablosu (`dataset.source`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct VeriKümesi {
    /// Sütun (boyut) adları.
    pub boyutlar: Vec<String>,
    /// Satırlar; her satır boyut sayısı kadar hücre içerir.
    pub satırlar: Vec<Vec<VeriDeğeri>>,
}

impl VeriKümesi {
    pub fn yeni<S: Into<String>>(boyutlar: impl IntoIterator<Item = S>) -> Self {
        VeriKümesi {
            boyutlar: boyutlar.into_iter().map(Into::into).collect(),
            satırlar: Vec::new(),
        }
    }

    /// Satır ekler; hücre sayısı boyut sayısından azsa `Boş` ile tamamlanır.
    pub fn satır(mut self, hücreler: impl IntoIterator<Item = VeriDeğeri>) -> Self {
        let mut satır: Vec<VeriDeğeri> = hücreler.into_iter().collect();
        satır.resize(self.boyutlar.len(), VeriDeğeri::Boş);
        self.satırlar.push(satır);
        self
    }

    /// `(metin, sayı...)` biçimindeki kayıtlardan hızlı kurulum: ilk sütun
    /// metin (kategori), kalanlar sayıdır.
    pub fn kayıtlar<S: Into<String>>(
        mut self,
        kayıtlar: impl IntoIterator<Item = (S, Vec<f64>)>,
    ) -> Self {
        for (ad, sayılar) in kayıtlar {
            let mut satır: Vec<VeriDeğeri> = vec![VeriDeğeri::Metin(ad.into())];
            satır.extend(sayılar.into_iter().map(VeriDeğeri::Sayı));
            satır.resize(self.boyutlar.len(), VeriDeğeri::Boş);
            self.satırlar.push(satır);
        }
        self
    }

    /// Boyut adının sütun sırası.
    pub fn boyut_sırası(&self, ad: &str) -> Option<usize> {
        self.boyutlar.iter().position(|b| b == ad)
    }

    /// Bir boyutun hücresi.
    pub fn hücre(&self, satır: usize, boyut: &str) -> Option<&VeriDeğeri> {
        let sütun = self.boyut_sırası(boyut)?;
        self.satırlar.get(satır)?.get(sütun)
    }

    /// Boyutu sayı listesi olarak döker (sayı olmayanlar `NaN`).
    pub fn sayılar(&self, boyut: &str) -> Result<Vec<f64>, BilesenHatasi> {
        let sütun = self
            .boyut_sırası(boyut)
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "veri_kümesi.boyut",
                ayrıntı: format!("`{boyut}` boyutu yok"),
            })?;
        Ok(self
            .satırlar
            .iter()
            .map(|satır| {
                satır
                    .get(sütun)
                    .and_then(|h| h.sayı())
                    .unwrap_or(f64::NAN)
            })
            .collect())
    }

    /// Boyutu metin listesi olarak döker.
    pub fn metinler(&self, boyut: &str) -> Result<Vec<String>, BilesenHatasi> {
        let sütun = self
            .boyut_sırası(boyut)
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "veri_kümesi.boyut",
                ayrıntı: format!("`{boyut}` boyutu yok"),
            })?;
        Ok(self
            .satırlar
            .iter()
            .map(|satır| match satır.get(sütun) {
                Some(VeriDeğeri::Metin(m)) => m.clone(),
                Some(VeriDeğeri::Sayı(s)) => crate::yardimci::bicim::ondalık_kırp(*s),
                _ => String::new(),
            })
            .collect())
    }

    // ------------------------------------------------------------------
    // Dönüşümler (`transform` karşılığı) — zincirlenebilir, kaynak tabloyu
    // değiştirmez.
    // ------------------------------------------------------------------

    /// Süzme dönüşümü (`transform: filter`).
    pub fn süz(&self, koşul: impl Fn(&[VeriDeğeri]) -> bool) -> VeriKümesi {
        VeriKümesi {
            boyutlar: self.boyutlar.clone(),
            satırlar: self
                .satırlar
                .iter()
                .filter(|satır| koşul(satır))
                .cloned()
                .collect(),
        }
    }

    /// Sıralama dönüşümü (`transform: sort`): verilen boyutun sayısal
    /// değerine göre.
    pub fn sırala(&self, boyut: &str, artan: bool) -> Result<VeriKümesi, BilesenHatasi> {
        let sütun = self
            .boyut_sırası(boyut)
            .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                alan: "veri_kümesi.sırala",
                ayrıntı: format!("`{boyut}` boyutu yok"),
            })?;
        let mut satırlar = self.satırlar.clone();
        satırlar.sort_by(|a, b| {
            let av = a.get(sütun).and_then(|h| h.sayı()).unwrap_or(f64::NAN);
            let bv = b.get(sütun).and_then(|h| h.sayı()).unwrap_or(f64::NAN);
            let sıra = av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal);
            if artan { sıra } else { sıra.reverse() }
        });
        Ok(VeriKümesi { boyutlar: self.boyutlar.clone(), satırlar })
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod testler {
    use super::*;

    fn örnek() -> VeriKümesi {
        VeriKümesi::yeni(["ürün", "satış", "kâr"]).kayıtlar([
            ("Elma", vec![120.0, 30.0]),
            ("Armut", vec![80.0, 22.0]),
            ("Kiraz", vec![160.0, 41.0]),
        ])
    }

    #[test]
    fn boyut_erişimi() {
        let küme = örnek();
        assert_eq!(küme.sayılar("satış").unwrap(), vec![120.0, 80.0, 160.0]);
        assert_eq!(
            küme.metinler("ürün").unwrap(),
            vec!["Elma", "Armut", "Kiraz"]
        );
        assert!(küme.sayılar("yok").is_err());
    }

    #[test]
    fn dönüşümler() {
        let küme = örnek();
        let sıralı = küme.sırala("satış", false).unwrap();
        assert_eq!(sıralı.metinler("ürün").unwrap(), vec!["Kiraz", "Elma", "Armut"]);
        let süzülü = küme.süz(|satır| {
            satır.get(1).and_then(|h| h.sayı()).unwrap_or(0.0) > 100.0
        });
        assert_eq!(süzülü.satırlar.len(), 2);
    }
}
