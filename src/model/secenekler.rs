//! Grafik seçenekleri — ECharts'taki kök `option` nesnesinin karşılığı.

use crate::animasyon::{Yumuşatma, ÖNTANIMLI_SÜRE_MS};
use crate::model::bilesen::{AraçKutusu, Başlık, Fırça, Gösterge, Izgara, İpucu};
use crate::model::eksen::Eksen;
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::kutupsal::KutupsalKoordinat;
use crate::model::radar::RadarKoordinatı;
use crate::model::yakinlastirma::VeriYakınlaştırma;
use crate::model::seri::Seri;
use crate::renk::Renk;
use crate::tema;

/// Kök grafik seçenekleri (`EChartsOption`).
#[derive(Clone, Debug)]
pub struct GrafikSeçenekleri {
    pub başlık: Option<Başlık>,
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
    pub ipucu: Option<İpucu>,
    /// Görsel eşleme bileşeni (`visualMap`); ısı haritası hücre renkleri
    /// buradan çözülür.
    pub görsel_eşleme: Option<GörselEşleme>,
    /// Radar koordinat sistemi (`radar`).
    pub radar: Option<RadarKoordinatı>,
    /// Kutupsal koordinat sistemi (`polar` + `angleAxis` + `radiusAxis`).
    pub kutupsal: Option<KutupsalKoordinat>,
    /// Veri yakınlaştırmaları (`dataZoom`).
    pub veri_yakınlaştırmaları: Vec<VeriYakınlaştırma>,
    /// Araç kutusu (`toolbox`).
    pub araç_kutusu: Option<AraçKutusu>,
    /// Fırça (`brush`): dikdörtgen seçim.
    pub fırça: Option<Fırça>,
    /// Seri renk paleti (`color`).
    pub palet: Vec<Renk>,
    pub arkaplan: Option<Renk>,
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
            gösterge: None,
            ızgara: Izgara::default(),
            ızgaralar: Vec::new(),
            x_ekseni: None,
            y_ekseni: None,
            x_eksenleri: Vec::new(),
            y_eksenleri: Vec::new(),
            seriler: Vec::new(),
            ipucu: None,
            görsel_eşleme: None,
            radar: None,
            kutupsal: None,
            veri_yakınlaştırmaları: Vec::new(),
            araç_kutusu: None,
            fırça: None,
            palet: tema::PALET.to_vec(),
            arkaplan: None,
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
        self
    }

    pub fn seriler<S: Into<Seri>>(mut self, seriler: impl IntoIterator<Item = S>) -> Self {
        self.seriler.extend(seriler.into_iter().map(Into::into));
        self
    }

    pub fn ipucu(mut self, ipucu: İpucu) -> Self {
        self.ipucu = Some(ipucu);
        self
    }

    pub fn görsel_eşleme(mut self, eşleme: GörselEşleme) -> Self {
        self.görsel_eşleme = Some(eşleme);
        self
    }

    pub fn radar(mut self, koordinat: RadarKoordinatı) -> Self {
        self.radar = Some(koordinat);
        self
    }

    pub fn kutupsal(mut self, koordinat: KutupsalKoordinat) -> Self {
        self.kutupsal = Some(koordinat);
        self
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
        self.veri_yakınlaştırmaları
            .iter()
            .find(|y| y.x_eksen_sırası == eksen_sırası)
            .map(|y| y.oranlar())
    }

    pub fn palet<R: Into<Renk>>(mut self, renkler: impl IntoIterator<Item = R>) -> Self {
        self.palet = renkler.into_iter().map(Into::into).collect();
        self
    }

    pub fn arkaplan(mut self, renk: impl Into<Renk>) -> Self {
        self.arkaplan = Some(renk.into());
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
        self.seriler
            .get(sıra)
            .and_then(|s| s.açık_renk())
            .map(|d| d.temsilî())
            .unwrap_or_else(|| self.palet_rengi(sıra))
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
        if !self.animasyon_süresi_güncelleme.is_finite()
            || self.animasyon_süresi_güncelleme < 0.0
        {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "animasyon_süresi_güncelleme",
                ayrıntı: format!(
                    "{} geçerli bir süre değil",
                    self.animasyon_süresi_güncelleme
                ),
            });
        }
        for eksen in [&self.x_ekseni, &self.y_ekseni].into_iter().flatten() {
            if let (Some(en_az), Some(en_çok)) = (eksen.en_az, eksen.en_çok)
                && en_az >= en_çok {
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
        }
        for seri in &self.seriler {
            if let Seri::Pasta(p) = seri
                && p.başlangıç_açısı.is_nan() {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "pasta.başlangıç_açısı",
                        ayrıntı: "başlangıç açısı sayı olmalı".to_string(),
                    });
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
            VeriÖğesi { değer, ..y.clone() }
        };

        for (sıra, seri) in sonuç.seriler.iter_mut().enumerate() {
            let Some(eski_seri) = eski.seriler.get(sıra) else { continue };
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
                Seri::AğaçHaritası(_) | Seri::GüneşPatlaması(_) | Seri::Ağaç(_) => {}
            }
        }
        sonuç
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod testler {
    use super::*;
    use crate::model::seri::ÇizgiSerisi;

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
}
