//! Tek eksenli koordinat yerleşimi — ECharts `coord/single/Single.ts`
//! karşılığı.

use crate::koordinat::{Dikdörtgen, ÇalışmaEkseni};
use crate::model::tek_eksen::{TekEksen, TekEksenYönü};
use crate::olcek::Ölçek;

/// Çözülmüş `singleAxis` kutusu ile piksel ölçeği.
#[derive(Clone, Debug)]
pub struct TekEksenYerleşimi {
    pub alan: Dikdörtgen,
    pub eksen: ÇalışmaEkseni,
    pub yön: TekEksenYönü,
}

impl TekEksenYerleşimi {
    /// ECharts `getLayoutRect` + `Single._adjustAxis` eşdeğeri.
    pub fn kur(model: &TekEksen, tuval: (f32, f32), ölçek: Ölçek) -> Self {
        let (tuval_genişliği, tuval_yüksekliği) = tuval;
        let sol = model.sol.map(|u| u.çöz(tuval_genişliği));
        let sağ = model.sağ.map(|u| u.çöz(tuval_genişliği));
        let üst = model.üst.map(|u| u.çöz(tuval_yüksekliği));
        let alt = model.alt.map(|u| u.çöz(tuval_yüksekliği));
        let genişlik = model
            .genişlik
            .map(|u| u.çöz(tuval_genişliği))
            .unwrap_or_else(|| {
                (tuval_genişliği - sol.unwrap_or(0.0) - sağ.unwrap_or(0.0)).max(0.0)
            });
        let yükseklik = model
            .yükseklik
            .map(|u| u.çöz(tuval_yüksekliği))
            .unwrap_or_else(|| {
                (tuval_yüksekliği - üst.unwrap_or(0.0) - alt.unwrap_or(0.0)).max(0.0)
            });
        let x = sol.unwrap_or_else(|| {
            sağ.map(|sağ| tuval_genişliği - sağ - genişlik)
                .unwrap_or((tuval_genişliği - genişlik) / 2.0)
        });
        let y = üst.unwrap_or_else(|| {
            alt.map(|alt| tuval_yüksekliği - alt - yükseklik)
                .unwrap_or((tuval_yüksekliği - yükseklik) / 2.0)
        });
        let alan = Dikdörtgen::yeni(x, y, genişlik, yükseklik);
        let piksel = match model.yön {
            TekEksenYönü::Yatay => [alan.x, alan.sağ()],
            // `Single._updateAxisTransform`: dikey eksende veri başlangıcı
            // kutunun altına, veri sonu üstüne taşınır.
            TekEksenYönü::Dikey => [alan.alt(), alan.y],
        };
        let eksen = ÇalışmaEkseni::yeni(
            model.eksen.clone(),
            ölçek,
            piksel,
            model.konum.eksen_konumu(),
        );
        Self {
            alan,
            eksen,
            yön: model.yön,
        }
    }

    /// Birincil tek eksen değerini koordinat kutusunun orta çizgisine taşır
    /// (`Single.dataToPoint`).
    pub fn veriden_noktaya(&self, değer: f64) -> (f32, f32) {
        match self.yön {
            TekEksenYönü::Yatay => (
                self.eksen.veriden_piksele(değer),
                self.alan.y + self.alan.yükseklik / 2.0,
            ),
            TekEksenYönü::Dikey => (
                self.alan.x + self.alan.genişlik / 2.0,
                self.eksen.veriden_piksele(değer),
            ),
        }
    }

    pub fn içeriyor_mu(&self, nokta: (f32, f32)) -> bool {
        self.alan.içeriyor_mu(nokta)
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::model::Uzunluk;
    use crate::model::tek_eksen::TekEksenKonumu;
    use crate::olcek::KategorikÖlçek;

    #[test]
    fn yuzdeli_kutuyu_resmi_ornek_gibi_cozer() {
        let model = TekEksen::kategori()
            .sol(150.0)
            .kenar_boşluğu(false)
            .üst(Uzunluk::Yüzde(5.0))
            .yükseklik(Uzunluk::Yüzde(100.0 / 7.0 - 10.0));
        let yerleşim = TekEksenYerleşimi::kur(
            &model,
            (700.0, 525.0),
            Ölçek::Kategorik(KategorikÖlçek::yeni(vec!["a".into(), "b".into()])),
        );
        assert!((yerleşim.alan.x - 150.0).abs() < 1e-5);
        assert!((yerleşim.alan.sağ() - 665.0).abs() < 1e-5);
        assert!((yerleşim.alan.y - 26.25).abs() < 1e-5);
        assert!((yerleşim.alan.yükseklik - 22.5).abs() < 1e-5);
        assert_eq!(yerleşim.veriden_noktaya(0.0), (150.0, 37.5));
        assert_eq!(yerleşim.veriden_noktaya(1.0), (665.0, 37.5));
    }

    #[test]
    fn dikey_eksen_deger_baslangicini_alta_koyar() {
        let model = TekEksen::kategori()
            .konum(TekEksenKonumu::Sol)
            .genişlik(20.0);
        let yerleşim = TekEksenYerleşimi::kur(
            &model,
            (200.0, 100.0),
            Ölçek::Kategorik(KategorikÖlçek::yeni(vec!["a".into(), "b".into()])),
        );
        assert!(yerleşim.veriden_noktaya(0.0).1 > yerleşim.veriden_noktaya(1.0).1);
    }
}
