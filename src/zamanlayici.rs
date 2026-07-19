//! ECharts scheduler/task boru hattının işbirlikçi Rust karşılığı.
//!
//! `performSeriesTasks`/processor/layout/visual/render sırası tek kayıt
//! defterinde tutulur. Her [`ArtımlıGörev`] küçük aralıklarda ilerler; UI
//! döngüsü her karede sınırlı bütçeyle [`Zamanlayıcı::adım`] çağırabilir.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::hata::BilesenHatasi;

/// Scheduler'ın sabit aşama sırası.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GörevAşaması {
    Önİşleme,
    Veriİşleme,
    Yerleşim,
    Görsel,
    Çizim,
    Sonİşleme,
}

/// Görevler arasında paylaşılan, renderer'dan bağımsız çalışma bağlamı.
#[derive(Clone, Debug, Default)]
pub struct GörevBağlamı {
    pub sayılar: BTreeMap<String, f64>,
    pub metinler: BTreeMap<String, String>,
    pub iz: Vec<String>,
    pub kuşak: u64,
}

/// Processor/layout/visual/render genişletme noktalarının ortak trait'i.
pub trait ArtımlıGörev: Send {
    fn kimlik(&self) -> &str;

    fn aşama(&self) -> GörevAşaması;

    fn bağımlılıklar(&self) -> &[String] {
        &[]
    }

    /// Yeni option/data kuşağı için toplam iş birimi.
    fn hazırla(&mut self, bağlam: &mut GörevBağlamı) -> Result<usize, BilesenHatasi>;

    /// Yarı açık iş aralığını işler. Scheduler aynı aralığı ikinci kez vermez.
    fn çalıştır(
        &mut self,
        aralık: Range<usize>,
        bağlam: &mut GörevBağlamı,
    ) -> Result<(), BilesenHatasi>;

    /// Tek scheduler adımında bu göreve verilebilecek üst sınır.
    fn önerilen_parça(&self) -> usize {
        1_000
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZamanlayıcıDurumu {
    Boş,
    Hazır,
    Çalışıyor,
    Tamamlandı,
    İptalEdildi,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Görevİlerlemesi {
    pub kimlik: String,
    pub aşama: GörevAşaması,
    pub tamamlanan: usize,
    pub toplam: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdımSonucu {
    pub işlenen: usize,
    pub durum: ZamanlayıcıDurumu,
    pub görev: Option<Görevİlerlemesi>,
}

struct GörevKaydı {
    görev: Box<dyn ArtımlıGörev>,
    toplam: usize,
    tamamlanan: usize,
}

/// Kayıtlı görevlerin bağımlılık sırasını çözen progressive scheduler.
pub struct Zamanlayıcı {
    görevler: Vec<GörevKaydı>,
    etkin: usize,
    durum: ZamanlayıcıDurumu,
    bağlam: GörevBağlamı,
    sıralandı: bool,
}

impl Default for Zamanlayıcı {
    fn default() -> Self {
        Self {
            görevler: Vec::new(),
            etkin: 0,
            durum: ZamanlayıcıDurumu::Boş,
            bağlam: GörevBağlamı::default(),
            sıralandı: false,
        }
    }
}

impl Zamanlayıcı {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn kaydet(&mut self, görev: impl ArtımlıGörev + 'static) -> Result<(), BilesenHatasi> {
        let kimlik = görev.kimlik().trim();
        if kimlik.is_empty() {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "scheduler.task.id",
                ayrıntı: "görev kimliği boş olamaz".to_owned(),
            });
        }
        if self
            .görevler
            .iter()
            .any(|kayıt| kayıt.görev.kimlik() == kimlik)
        {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "scheduler.task.id",
                ayrıntı: format!("`{kimlik}` görevi yinelendi"),
            });
        }
        self.görevler.push(GörevKaydı {
            görev: Box::new(görev),
            toplam: 0,
            tamamlanan: 0,
        });
        self.sıralandı = false;
        self.durum = ZamanlayıcıDurumu::Boş;
        Ok(())
    }

    /// Bağımlılık grafiğini çözer ve bütün görevleri yeni kuşak için sıfırlar.
    pub fn hazırla(&mut self) -> Result<(), BilesenHatasi> {
        if !self.sıralandı {
            self.görevleri_sırala()?;
        }
        self.bağlam.kuşak = self.bağlam.kuşak.saturating_add(1);
        self.etkin = 0;
        for kayıt in &mut self.görevler {
            kayıt.tamamlanan = 0;
            kayıt.toplam = kayıt.görev.hazırla(&mut self.bağlam)?;
        }
        self.durum = if self.görevler.is_empty() {
            ZamanlayıcıDurumu::Tamamlandı
        } else {
            ZamanlayıcıDurumu::Hazır
        };
        Ok(())
    }

    /// En çok `bütçe` iş birimi ilerler. Sıfır bütçe modelde değişiklik yapmaz.
    pub fn adım(&mut self, mut bütçe: usize) -> Result<AdımSonucu, BilesenHatasi> {
        if matches!(
            self.durum,
            ZamanlayıcıDurumu::Boş | ZamanlayıcıDurumu::İptalEdildi
        ) {
            return Err(BilesenHatasi::GeçersizSeçenek {
                alan: "scheduler.state",
                ayrıntı: "adım öncesinde scheduler hazırlanmalı".to_owned(),
            });
        }
        if self.durum == ZamanlayıcıDurumu::Tamamlandı || bütçe == 0 {
            return Ok(AdımSonucu {
                işlenen: 0,
                durum: self.durum,
                görev: self.etkin_ilerleme(),
            });
        }

        self.durum = ZamanlayıcıDurumu::Çalışıyor;
        let başlangıç_bütçesi = bütçe;
        while bütçe > 0 {
            let Some(kayıt) = self.görevler.get_mut(self.etkin) else {
                self.durum = ZamanlayıcıDurumu::Tamamlandı;
                break;
            };
            if kayıt.tamamlanan >= kayıt.toplam {
                self.etkin = self.etkin.saturating_add(1);
                continue;
            }
            let kalan = kayıt.toplam.saturating_sub(kayıt.tamamlanan);
            let parça = kayıt.görev.önerilen_parça().max(1).min(kalan).min(bütçe);
            let başlangıç = kayıt.tamamlanan;
            let bitiş = başlangıç.saturating_add(parça);
            kayıt.görev.çalıştır(başlangıç..bitiş, &mut self.bağlam)?;
            kayıt.tamamlanan = bitiş;
            bütçe = bütçe.saturating_sub(parça);
        }
        if self
            .görevler
            .get(self.etkin)
            .map(|kayıt| kayıt.tamamlanan >= kayıt.toplam)
            .unwrap_or(false)
        {
            while self
                .görevler
                .get(self.etkin)
                .map(|kayıt| kayıt.tamamlanan >= kayıt.toplam)
                .unwrap_or(false)
            {
                self.etkin = self.etkin.saturating_add(1);
            }
        }
        if self.etkin >= self.görevler.len() {
            self.durum = ZamanlayıcıDurumu::Tamamlandı;
        }
        Ok(AdımSonucu {
            işlenen: başlangıç_bütçesi.saturating_sub(bütçe),
            durum: self.durum,
            görev: self.etkin_ilerleme(),
        })
    }

    pub fn iptal(&mut self) {
        if self.durum != ZamanlayıcıDurumu::Tamamlandı {
            self.durum = ZamanlayıcıDurumu::İptalEdildi;
        }
    }

    /// Etkin çalışmayı iptal edip aynı kayıtlarla yeni kuşak başlatır.
    pub fn yeniden_başlat(&mut self) -> Result<(), BilesenHatasi> {
        self.iptal();
        self.hazırla()
    }

    pub fn durum(&self) -> ZamanlayıcıDurumu {
        self.durum
    }

    pub fn bağlam(&self) -> &GörevBağlamı {
        &self.bağlam
    }

    pub fn bağlam_mut(&mut self) -> &mut GörevBağlamı {
        &mut self.bağlam
    }

    pub fn ilerleme(&self) -> Vec<Görevİlerlemesi> {
        self.görevler
            .iter()
            .map(|kayıt| Görevİlerlemesi {
                kimlik: kayıt.görev.kimlik().to_owned(),
                aşama: kayıt.görev.aşama(),
                tamamlanan: kayıt.tamamlanan,
                toplam: kayıt.toplam,
            })
            .collect()
    }

    fn etkin_ilerleme(&self) -> Option<Görevİlerlemesi> {
        let kayıt = self.görevler.get(self.etkin)?;
        Some(Görevİlerlemesi {
            kimlik: kayıt.görev.kimlik().to_owned(),
            aşama: kayıt.görev.aşama(),
            tamamlanan: kayıt.tamamlanan,
            toplam: kayıt.toplam,
        })
    }

    fn görevleri_sırala(&mut self) -> Result<(), BilesenHatasi> {
        let kimlikler: BTreeSet<String> = self
            .görevler
            .iter()
            .map(|kayıt| kayıt.görev.kimlik().to_owned())
            .collect();
        for kayıt in &self.görevler {
            for bağımlılık in kayıt.görev.bağımlılıklar() {
                if !kimlikler.contains(bağımlılık) {
                    return Err(BilesenHatasi::GeçersizSeçenek {
                        alan: "scheduler.task.dependencies",
                        ayrıntı: format!(
                            "`{}` görevinin `{bağımlılık}` bağımlılığı kayıtlı değil",
                            kayıt.görev.kimlik()
                        ),
                    });
                }
            }
        }

        let mut kalan: Vec<usize> = (0..self.görevler.len()).collect();
        let mut tamamlanan = BTreeSet::new();
        let mut sıra = Vec::with_capacity(kalan.len());
        while !kalan.is_empty() {
            let en_erken_aşama = kalan
                .iter()
                .filter_map(|sıra| self.görevler.get(*sıra))
                .map(|kayıt| kayıt.görev.aşama())
                .min();
            let seçim = kalan.iter().position(|sıra| {
                let Some(kayıt) = self.görevler.get(*sıra) else {
                    return false;
                };
                Some(kayıt.görev.aşama()) == en_erken_aşama
                    && kayıt
                        .görev
                        .bağımlılıklar()
                        .iter()
                        .all(|bağımlılık| tamamlanan.contains(bağımlılık))
            });
            let Some(seçim) = seçim else {
                return Err(BilesenHatasi::GeçersizSeçenek {
                    alan: "scheduler.task.dependencies",
                    ayrıntı: "döngüsel bağımlılık veya daha geç aşamaya bağımlılık var"
                        .to_owned(),
                });
            };
            let sıra_no = kalan.remove(seçim);
            if let Some(kayıt) = self.görevler.get(sıra_no) {
                tamamlanan.insert(kayıt.görev.kimlik().to_owned());
            }
            sıra.push(sıra_no);
        }

        let mut eski: Vec<Option<GörevKaydı>> = std::mem::take(&mut self.görevler)
            .into_iter()
            .map(Some)
            .collect();
        for sıra_no in sıra {
            if let Some(kayıt) = eski.get_mut(sıra_no).and_then(Option::take) {
                self.görevler.push(kayıt);
            }
        }
        self.sıralandı = true;
        Ok(())
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

    struct KayıtGörevi {
        kimlik: String,
        aşama: GörevAşaması,
        bağımlılıklar: Vec<String>,
        toplam: usize,
        parça: usize,
    }

    impl KayıtGörevi {
        fn yeni(kimlik: &str, aşama: GörevAşaması, toplam: usize) -> Self {
            Self {
                kimlik: kimlik.to_owned(),
                aşama,
                bağımlılıklar: Vec::new(),
                toplam,
                parça: usize::MAX,
            }
        }

        fn bağımlı(mut self, kimlik: &str) -> Self {
            self.bağımlılıklar.push(kimlik.to_owned());
            self
        }

        fn parça(mut self, parça: usize) -> Self {
            self.parça = parça;
            self
        }
    }

    impl ArtımlıGörev for KayıtGörevi {
        fn kimlik(&self) -> &str {
            &self.kimlik
        }

        fn aşama(&self) -> GörevAşaması {
            self.aşama
        }

        fn bağımlılıklar(&self) -> &[String] {
            &self.bağımlılıklar
        }

        fn hazırla(&mut self, bağlam: &mut GörevBağlamı) -> Result<usize, BilesenHatasi> {
            bağlam.iz.push(format!("{}:hazır", self.kimlik));
            Ok(self.toplam)
        }

        fn çalıştır(
            &mut self,
            aralık: Range<usize>,
            bağlam: &mut GörevBağlamı,
        ) -> Result<(), BilesenHatasi> {
            bağlam
                .iz
                .push(format!("{}:{}..{}", self.kimlik, aralık.start, aralık.end));
            Ok(())
        }

        fn önerilen_parça(&self) -> usize {
            self.parça
        }
    }

    #[test]
    fn asama_ve_bagimlilik_sirasi() {
        let mut zamanlayıcı = Zamanlayıcı::yeni();
        zamanlayıcı
            .kaydet(KayıtGörevi::yeni("çiz", GörevAşaması::Çizim, 1).bağımlı("renk"))
            .unwrap();
        zamanlayıcı
            .kaydet(KayıtGörevi::yeni("yer", GörevAşaması::Yerleşim, 1).bağımlı("veri"))
            .unwrap();
        zamanlayıcı
            .kaydet(KayıtGörevi::yeni("veri", GörevAşaması::Veriİşleme, 1))
            .unwrap();
        zamanlayıcı
            .kaydet(KayıtGörevi::yeni("renk", GörevAşaması::Görsel, 1).bağımlı("yer"))
            .unwrap();
        zamanlayıcı.hazırla().unwrap();
        while zamanlayıcı.durum() != ZamanlayıcıDurumu::Tamamlandı {
            zamanlayıcı.adım(1).unwrap();
        }
        assert_eq!(
            zamanlayıcı.bağlam().iz,
            vec![
                "veri:hazır",
                "yer:hazır",
                "renk:hazır",
                "çiz:hazır",
                "veri:0..1",
                "yer:0..1",
                "renk:0..1",
                "çiz:0..1",
            ]
        );
    }

    #[test]
    fn butce_ve_gorev_parcasi_asılmaz() {
        let mut zamanlayıcı = Zamanlayıcı::yeni();
        zamanlayıcı
            .kaydet(KayıtGörevi::yeni("büyük", GörevAşaması::Çizim, 10).parça(3))
            .unwrap();
        zamanlayıcı.hazırla().unwrap();
        let ilk = zamanlayıcı.adım(8).unwrap();
        assert_eq!(ilk.işlenen, 8);
        assert_eq!(zamanlayıcı.ilerleme()[0].tamamlanan, 8);
        assert_eq!(
            zamanlayıcı.bağlam().iz,
            vec!["büyük:hazır", "büyük:0..3", "büyük:3..6", "büyük:6..8"]
        );
        assert_eq!(zamanlayıcı.adım(8).unwrap().işlenen, 2);
        assert_eq!(zamanlayıcı.durum(), ZamanlayıcıDurumu::Tamamlandı);
    }

    #[test]
    fn iptal_ve_yeniden_baslat_yeni_kusak_uretir() {
        let mut zamanlayıcı = Zamanlayıcı::yeni();
        zamanlayıcı
            .kaydet(KayıtGörevi::yeni("iş", GörevAşaması::Veriİşleme, 10))
            .unwrap();
        zamanlayıcı.hazırla().unwrap();
        zamanlayıcı.adım(2).unwrap();
        let ilk_kuşak = zamanlayıcı.bağlam().kuşak;
        zamanlayıcı.iptal();
        assert!(zamanlayıcı.adım(1).is_err());
        zamanlayıcı.yeniden_başlat().unwrap();
        assert_eq!(zamanlayıcı.bağlam().kuşak, ilk_kuşak + 1);
        assert_eq!(zamanlayıcı.ilerleme()[0].tamamlanan, 0);
    }

    #[test]
    fn bilinmeyen_dongusel_ve_gec_asama_bagimliligi_reddedilir() {
        let mut bilinmeyen = Zamanlayıcı::yeni();
        bilinmeyen
            .kaydet(KayıtGörevi::yeni("a", GörevAşaması::Çizim, 1).bağımlı("yok"))
            .unwrap();
        assert!(bilinmeyen.hazırla().is_err());

        let mut geç = Zamanlayıcı::yeni();
        geç.kaydet(KayıtGörevi::yeni("erken", GörevAşaması::Yerleşim, 1).bağımlı("geç"))
            .unwrap();
        geç.kaydet(KayıtGörevi::yeni("geç", GörevAşaması::Görsel, 1))
            .unwrap();
        assert!(geç.hazırla().is_err());
    }
}
