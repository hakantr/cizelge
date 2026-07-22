//! ECharts `coord/parallel/Parallel.ts` koordinat yerleşimi.

use std::collections::HashSet;

use crate::koordinat::{Dikdörtgen, ÇalışmaEkseni};
use crate::model::deger::{VeriDeğeri, VeriÖğesi};
use crate::model::eksen::{Eksen, EksenKonumu, EksenTürü};
use crate::model::paralel::{
    ParalelAlanSeçimStili, ParalelEkseni, ParalelKoordinatı, ParalelYerleşim,
};
use crate::model::seri::{ParalelBoyut, ParalelSerisi};
use crate::olcek::{
    AralıkÖlçeği, KategorikÖlçek, KırılmaEşleyici, LogÖlçeği, ZamanÖlçeği, Ölçek
};

/// Eksen alan seçiminden sonra bir veri çizgisinin görsel durumu.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParalelEtkinlik {
    Normal,
    Etkin,
    EtkinDeğil,
}

/// `Parallel.getSlidedAxisExpandWindow` sonucundaki hareket türü.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParalelGenişletmeDavranışı {
    Yok,
    Kaydır,
    Atla,
}

/// Fare konumundan çözülen yeni `axisExpandWindow` ve hareket türü.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ParalelGenişletmeSonucu {
    pub pencere: [f32; 2],
    pub davranış: ParalelGenişletmeDavranışı,
}

#[derive(Clone, Debug)]
struct ParalelGenişletmeBilgisi {
    genişlik: f32,
    dar_genişlik: f32,
    eksen_sayısı: usize,
    pencere_konumu: f32,
    kaydırma_tetik_alanı: [Option<f32>; 3],
}

/// Tek `parallelAxis` bileşeninin çözümlenmiş ölçeği ve ekran konumu.
#[derive(Clone, Debug)]
pub struct ParalelÇalışmaEkseni {
    pub bileşen_sırası: Option<usize>,
    pub kimlik: Option<String>,
    pub boyutlar: Vec<usize>,
    pub konum: f32,
    pub eksen: ÇalışmaEkseni,
    pub etiket_göster: bool,
    pub ad_kullanılabilir_genişlik: f32,
    pub ad_kısaltma_genişliği: Option<f32>,
    pub alan_seçim_stili: ParalelAlanSeçimStili,
    pub gerçek_zamanlı: bool,
    pub etkin_aralıklar: Vec<[f64; 2]>,
}

impl ParalelÇalışmaEkseni {
    pub fn ana_boyut(&self) -> usize {
        self.boyutlar.first().copied().unwrap_or(0)
    }

    /// Ham ECharts satır değerini eksenin sayısal veri uzayına çevirir.
    pub fn değeri_çöz(&self, değer: &VeriDeğeri) -> Option<f64> {
        if self.eksen.ölçek.kategorik_mi() {
            let anahtar = değer_metni(değer)?;
            return self
                .eksen
                .ölçek
                .kategori_sırası(&anahtar)
                .or_else(|| değer.sayı().filter(|değer| değer.is_finite()));
        }
        değer.sayı().filter(|değer| değer.is_finite())
    }

    pub fn etkin_mi(&self, değer: &VeriDeğeri) -> bool {
        if self.etkin_aralıklar.is_empty() {
            return true;
        }
        let Some(değer) = self.değeri_çöz(değer) else {
            return false;
        };
        self.etkin_aralıklar.iter().any(|aralık| {
            let en_az = aralık[0].min(aralık[1]);
            let en_çok = aralık[0].max(aralık[1]);
            değer >= en_az && değer <= en_çok
        })
    }
}

/// Bir `parallel` bileşeninin bütün eksenleriyle çözümlenmiş geometrisi.
#[derive(Clone, Debug)]
pub struct ParalelYerleşimi {
    pub bileşen_sırası: usize,
    pub alan: Dikdörtgen,
    pub yön: ParalelYerleşim,
    pub eksenler: Vec<ParalelÇalışmaEkseni>,
    pub genişletilebilir: bool,
    pub genişletme_penceresi: [f32; 2],
    pub genişletme_tetikleyicisi: crate::model::paralel::ParalelGenişletmeTetikleyicisi,
    pub genişletme_oranı: f32,
    pub genişletme_gecikmesi_ms: u64,
    genişletme: Option<ParalelGenişletmeBilgisi>,
}

impl ParalelYerleşimi {
    #[allow(clippy::too_many_arguments)]
    pub fn kur(
        koordinat: &ParalelKoordinatı,
        bileşen_sırası: usize,
        tüm_eksenler: &[ParalelEkseni],
        bağlı_seriler: &[&ParalelSerisi],
        tuval: (f32, f32),
    ) -> Option<Self> {
        let alan = kutuyu_çöz(koordinat, tuval);
        let mut eksenler = tüm_eksenler
            .iter()
            .enumerate()
            .filter(|(_, eksen)| eksen_bağlı_mı(eksen, koordinat, bileşen_sırası))
            .map(|(sıra, eksen)| (Some(sıra), eksen.clone()))
            .collect::<Vec<_>>();

        // ECharts preprocessor `parallel` bileşenini otomatik üretir. Eski
        // Rust API'sindeki series.boyutlar da aynı yoldan örtük axis listesi
        // olarak korunur.
        if eksenler.is_empty() {
            let boyutlar = eski_boyutları_çöz(bağlı_seriler);
            eksenler = boyutlar
                .into_iter()
                .enumerate()
                .map(|(boyut, seçenek)| {
                    let mut eksen = ParalelEkseni::yeni(boyut).ad(seçenek.ad);
                    if let Some(en_az) = seçenek.en_az {
                        eksen = eksen.en_az(en_az);
                    }
                    if let Some(en_çok) = seçenek.en_çok {
                        eksen = eksen.en_çok(en_çok);
                    }
                    (None, eksen)
                })
                .collect();
        }
        if eksenler.is_empty() {
            return None;
        }

        let konumlar = eksen_konumlarını_çöz(koordinat, alan, eksenler.len());
        let genişletilebilir = konumlar.genisletilebilir;
        let genişletme_penceresi = konumlar.pencere;
        let genişletme = konumlar.genişletme;
        let eksenler = eksenler
            .into_iter()
            .zip(konumlar.eksenler)
            .map(|((bileşen_sırası, model), konum)| {
                let mut seçenek = model.çöz(&koordinat.eksen_varsayılanı);
                // ParallelAxisView AxisBuilder'ı tek başına kurar; kartezyen
                // karşı ekseninin `auto` kararına bağlı değildir.
                seçenek.çizgi.göster.get_or_insert(true);
                seçenek.çentik.göster.get_or_insert(true);
                seçenek.etiket.göster &= konum.etiket_göster;
                let boyut = model.boyutlar.first().copied().unwrap_or(0);
                let (kategoriler, kapsam) = eksen_verisini_topla(&seçenek, boyut, bağlı_seriler);
                let ölçek = ölçek_kur(&seçenek, kategoriler, kapsam);
                let (piksel, eksen_konumu) = match koordinat.yerleşim {
                    ParalelYerleşim::Yatay => ([alan.alt(), alan.y], EksenKonumu::Sağ),
                    ParalelYerleşim::Dikey => ([alan.x, alan.sağ()], EksenKonumu::Alt),
                };
                ParalelÇalışmaEkseni {
                    bileşen_sırası,
                    kimlik: model.kimlik,
                    boyutlar: model.boyutlar,
                    konum: konum.konum,
                    eksen: ÇalışmaEkseni::yeni(seçenek, ölçek, piksel, eksen_konumu),
                    etiket_göster: konum.etiket_göster,
                    ad_kullanılabilir_genişlik: konum.ad_genişliği,
                    ad_kısaltma_genişliği: konum.ad_kısaltma,
                    alan_seçim_stili: model.alan_seçim_stili,
                    gerçek_zamanlı: model.gerçek_zamanlı,
                    etkin_aralıklar: model
                        .etkin_aralıklar
                        .into_iter()
                        .map(|mut aralık| {
                            aralık.sort_by(|a, b| {
                                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                            });
                            aralık
                        })
                        .collect(),
                }
            })
            .collect();

        Some(Self {
            bileşen_sırası,
            alan,
            yön: koordinat.yerleşim,
            eksenler,
            genişletilebilir,
            genişletme_penceresi,
            genişletme_tetikleyicisi: koordinat.eksen_genişletme_tetikleyicisi,
            genişletme_oranı: koordinat.eksen_genişletme_oranı,
            genişletme_gecikmesi_ms: koordinat.eksen_genişletme_gecikmesi_ms,
            genişletme,
        })
    }

    pub fn içeriyor_mu(&self, nokta: (f32, f32)) -> bool {
        self.alan.içeriyor_mu(nokta)
    }

    pub fn satır_değeri<'a>(
        &self,
        öğe: &'a VeriÖğesi,
        eksen: &ParalelÇalışmaEkseni,
    ) -> Option<&'a VeriDeğeri> {
        satır_değeri(öğe, eksen.ana_boyut())
    }

    pub fn veriden_noktaya(
        &self,
        öğe: &VeriÖğesi,
        eksen: &ParalelÇalışmaEkseni,
    ) -> Option<(f32, f32)> {
        let değer = self.eksen_değeri(öğe, eksen)?;
        let değer_konumu = eksen.eksen.veriden_piksele(değer);
        Some(match self.yön {
            ParalelYerleşim::Yatay => (eksen.konum, değer_konumu),
            ParalelYerleşim::Dikey => (değer_konumu, eksen.konum),
        })
    }

    /// Satırın belirli parallelAxis üzerindeki çözülmüş ölçek değeri.
    pub fn eksen_değeri(
        &self, öğe: &VeriÖğesi, eksen: &ParalelÇalışmaEkseni
    ) -> Option<f64> {
        match &öğe.değer {
            VeriDeğeri::Dizi(dizi) => dizi
                .get(eksen.ana_boyut())
                .copied()
                .filter(|değer| değer.is_finite()),
            VeriDeğeri::Çift(çift) => çift
                .get(eksen.ana_boyut())
                .copied()
                .filter(|değer| değer.is_finite()),
            _ => self
                .satır_değeri(öğe, eksen)
                .and_then(|değer| eksen.değeri_çöz(değer)),
        }
    }

    /// Parallel.eachActiveState: herhangi bir axis seçimi yoksa `normal`,
    /// varsa bütün etkin eksen aralıklarından geçen satır `active` olur.
    pub fn veri_etkinliği(&self, öğe: &VeriÖğesi) -> ParalelEtkinlik {
        if !self
            .eksenler
            .iter()
            .any(|eksen| !eksen.etkin_aralıklar.is_empty())
        {
            return ParalelEtkinlik::Normal;
        }
        if self.eksenler.iter().all(|eksen| {
            eksen.etkin_aralıklar.is_empty()
                || self.eksen_değeri(öğe, eksen).is_some_and(|değer| {
                    eksen.etkin_aralıklar.iter().any(|aralık| {
                        değer >= aralık[0].min(aralık[1]) && değer <= aralık[0].max(aralık[1])
                    })
                })
        }) {
            ParalelEtkinlik::Etkin
        } else {
            ParalelEtkinlik::EtkinDeğil
        }
    }

    /// ECharts `Parallel.getSlidedAxisExpandWindow` portu. Girdi çizelge
    /// tuvalinin yerel piksel koordinatıdır; alan dışı veya merkezdeki
    /// hareketsiz bölgede `Yok` döner.
    pub fn genişletme_penceresini_çöz(&self, nokta: (f32, f32)) -> ParalelGenişletmeSonucu {
        let mut pencere = self.genişletme_penceresi;
        let Some(bilgi) = self.genişletme.as_ref() else {
            return ParalelGenişletmeSonucu {
                pencere,
                davranış: ParalelGenişletmeDavranışı::Yok,
            };
        };
        if !self.içeriyor_mu(nokta) {
            return ParalelGenişletmeSonucu {
                pencere,
                davranış: ParalelGenişletmeDavranışı::Yok,
            };
        }

        let pencere_boyutu = pencere[1] - pencere[0];
        let kapsam = [
            0.0,
            bilgi.genişlik * bilgi.eksen_sayısı.saturating_sub(1) as f32,
        ];
        let düzen_başı = match self.yön {
            ParalelYerleşim::Yatay => self.alan.x,
            ParalelYerleşim::Dikey => self.alan.y,
        };
        let düzen_noktası = match self.yön {
            ParalelYerleşim::Yatay => nokta.0,
            ParalelYerleşim::Dikey => nokta.1,
        };
        let nokta_koordinatı = düzen_noktası - düzen_başı - bilgi.pencere_konumu;
        let mut davranış = ParalelGenişletmeDavranışı::Kaydır;

        if bilgi.dar_genişlik != 0.0 {
            let tetik = bilgi.kaydırma_tetik_alanı;
            let atlama_kullan = tetik[0].is_some();
            let dış = tetik[0].unwrap_or(0.0);
            let iç = tetik[1].unwrap_or(0.0);
            let atlama_merkezi = tetik[2].unwrap_or(0.5);
            let mut fark;
            if atlama_kullan && nokta_koordinatı < pencere_boyutu * dış {
                davranış = ParalelGenişletmeDavranışı::Atla;
                fark = nokta_koordinatı - pencere_boyutu * atlama_merkezi;
            } else if atlama_kullan && nokta_koordinatı > pencere_boyutu * (1.0 - dış) {
                davranış = ParalelGenişletmeDavranışı::Atla;
                fark = nokta_koordinatı - pencere_boyutu * (1.0 - atlama_merkezi);
            } else if nokta_koordinatı < pencere_boyutu * iç {
                fark = nokta_koordinatı - pencere_boyutu * iç;
            } else if nokta_koordinatı > pencere_boyutu * (1.0 - iç) {
                fark = nokta_koordinatı - pencere_boyutu * (1.0 - iç);
            } else {
                fark = 0.0;
            }
            fark *= bilgi.genişlik / bilgi.dar_genişlik;
            if fark == 0.0 {
                davranış = ParalelGenişletmeDavranışı::Yok;
            } else {
                pencere[0] += fark;
                pencere[1] += fark;
                if pencere[0] < kapsam[0] {
                    let düzeltme = kapsam[0] - pencere[0];
                    pencere[0] += düzeltme;
                    pencere[1] += düzeltme;
                }
                if pencere[1] > kapsam[1] {
                    let düzeltme = kapsam[1] - pencere[1];
                    pencere[0] += düzeltme;
                    pencere[1] += düzeltme;
                }
            }
        } else if pencere_boyutu > 0.0 {
            let konum = kapsam[1] * nokta_koordinatı / pencere_boyutu;
            pencere[0] = 0.0f32.max(konum - pencere_boyutu / 2.0);
            pencere[1] = kapsam[1].min(pencere[0] + pencere_boyutu);
            pencere[0] = pencere[1] - pencere_boyutu;
        } else {
            davranış = ParalelGenişletmeDavranışı::Yok;
        }

        ParalelGenişletmeSonucu {
            pencere, davranış
        }
    }
}

/// Seri `parallelId` verdiğinde kimlik, aksi halde `parallelIndex` bağı.
pub fn seri_bağlı_mı(
    seri: &ParalelSerisi,
    koordinat: &ParalelKoordinatı,
    bileşen_sırası: usize,
) -> bool {
    seri.paralel_kimliği
        .as_ref()
        .map(|kimlik| koordinat.kimlik.as_ref() == Some(kimlik))
        .unwrap_or(seri.paralel_sırası == bileşen_sırası)
}

fn eksen_bağlı_mı(
    eksen: &ParalelEkseni,
    koordinat: &ParalelKoordinatı,
    bileşen_sırası: usize,
) -> bool {
    eksen
        .paralel_kimliği
        .as_ref()
        .map(|kimlik| koordinat.kimlik.as_ref() == Some(kimlik))
        .unwrap_or(eksen.paralel_sırası == bileşen_sırası)
}

fn eski_boyutları_çöz(seriler: &[&ParalelSerisi]) -> Vec<ParalelBoyut> {
    seriler
        .iter()
        .max_by_key(|seri| seri.boyutlar.len())
        .map(|seri| seri.boyutlar.clone())
        .unwrap_or_default()
}

fn satır_değeri(öğe: &VeriÖğesi, boyut: usize) -> Option<&VeriDeğeri> {
    match &öğe.değer {
        VeriDeğeri::KarmaDizi(dizi) => dizi.get(boyut),
        VeriDeğeri::Dizi(dizi) => {
            // Sayısal dizi ödünç bir VeriDeğeri üretmediği için aşağıdaki
            // özel yol çağıran dönüşümde ele alınır.
            let _ = dizi.get(boyut)?;
            None
        }
        VeriDeğeri::Çift(_) if boyut < 2 => None,
        _ if boyut == 0 => Some(&öğe.değer),
        _ => öğe.boyutlar.get(boyut).map(|(_, değer)| değer),
    }
}

/// Sayısal `Dizi` ve `Çift` satırları için ödünç değer üretmeden doğrudan
/// eksen değerini çözer.
fn satır_sayısı(öğe: &VeriÖğesi, boyut: usize) -> Option<f64> {
    match &öğe.değer {
        VeriDeğeri::Dizi(dizi) => dizi.get(boyut).copied(),
        VeriDeğeri::Çift(çift) => çift.get(boyut).copied(),
        _ => satır_değeri(öğe, boyut).and_then(VeriDeğeri::sayı),
    }
}

fn satır_metni(öğe: &VeriÖğesi, boyut: usize) -> Option<String> {
    match &öğe.değer {
        VeriDeğeri::Dizi(dizi) => dizi.get(boyut).map(|değer| değer.to_string()),
        VeriDeğeri::Çift(çift) => çift.get(boyut).map(|değer| değer.to_string()),
        _ => satır_değeri(öğe, boyut).and_then(değer_metni),
    }
}

fn değer_metni(değer: &VeriDeğeri) -> Option<String> {
    match değer {
        VeriDeğeri::Boş | VeriDeğeri::Dizi(_) | VeriDeğeri::Çift(_) | VeriDeğeri::KarmaDizi(_) => {
            None
        }
        VeriDeğeri::Sayı(değer) => Some(crate::yardimci::bicim::ondalık_kırp(*değer)),
        VeriDeğeri::Metin(metin) => Some(metin.clone()),
        VeriDeğeri::Mantıksal(değer) => Some(değer.to_string()),
        VeriDeğeri::Zaman(ms) => Some(ms.to_string()),
    }
}

fn eksen_verisini_topla(
    seçenek: &Eksen,
    boyut: usize,
    seriler: &[&ParalelSerisi],
) -> (Vec<String>, [f64; 2]) {
    let mut kategoriler = seçenek.veri.clone();
    let mut görülen = kategoriler.iter().cloned().collect::<HashSet<_>>();
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    for öğe in seriler.iter().flat_map(|seri| seri.veri.iter()) {
        if seçenek.tür == EksenTürü::Kategori {
            if let Some(metin) = satır_metni(öğe, boyut)
                && görülen.insert(metin.clone())
            {
                kategoriler.push(metin);
            }
        } else if let Some(değer) = satır_sayısı(öğe, boyut).filter(|değer| değer.is_finite())
        {
            kapsam[0] = kapsam[0].min(değer);
            kapsam[1] = kapsam[1].max(değer);
        }
    }
    if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
        kapsam = [0.0, 1.0];
    }
    (kategoriler, kapsam)
}

fn kutuyu_çöz(model: &ParalelKoordinatı, tuval: (f32, f32)) -> Dikdörtgen {
    let (tuval_genişliği, tuval_yüksekliği) = tuval;
    let sol = model.sol.map(|u| u.çöz(tuval_genişliği));
    let sağ = model.sağ.map(|u| u.çöz(tuval_genişliği));
    let üst = model.üst.map(|u| u.çöz(tuval_yüksekliği));
    let alt = model.alt.map(|u| u.çöz(tuval_yüksekliği));
    let genişlik = model
        .genişlik
        .map(|u| u.çöz(tuval_genişliği))
        .unwrap_or_else(|| (tuval_genişliği - sol.unwrap_or(0.0) - sağ.unwrap_or(0.0)).max(0.0));
    let yükseklik = model
        .yükseklik
        .map(|u| u.çöz(tuval_yüksekliği))
        .unwrap_or_else(|| (tuval_yüksekliği - üst.unwrap_or(0.0) - alt.unwrap_or(0.0)).max(0.0));
    let x = sol.unwrap_or_else(|| {
        sağ.map(|sağ| tuval_genişliği - sağ - genişlik)
            .unwrap_or((tuval_genişliği - genişlik) / 2.0)
    });
    let y = üst.unwrap_or_else(|| {
        alt.map(|alt| tuval_yüksekliği - alt - yükseklik)
            .unwrap_or((tuval_yüksekliği - yükseklik) / 2.0)
    });
    Dikdörtgen::yeni(x, y, genişlik, yükseklik)
}

#[derive(Clone, Copy, Debug)]
struct EksenKonumBilgisi {
    konum: f32,
    ad_genişliği: f32,
    etiket_göster: bool,
    ad_kısaltma: Option<f32>,
}

struct EksenKonumları {
    eksenler: Vec<EksenKonumBilgisi>,
    genisletilebilir: bool,
    pencere: [f32; 2],
    genişletme: Option<ParalelGenişletmeBilgisi>,
}

/// `Parallel._makeLayoutInfo`, `layoutAxisWithoutExpand` ve
/// `layoutAxisWithExpand`in aynı sınır/yuvarlama kurallarıyla portu.
fn eksen_konumlarını_çöz(
    model: &ParalelKoordinatı,
    alan: Dikdörtgen,
    eksen_sayısı: usize,
) -> EksenKonumları {
    let (başlangıç, uzunluk) = match model.yerleşim {
        ParalelYerleşim::Yatay => (alan.x, alan.genişlik),
        ParalelYerleşim::Dikey => (alan.y, alan.yükseklik),
    };
    if eksen_sayısı == 1 {
        return EksenKonumları {
            eksenler: vec![EksenKonumBilgisi {
                konum: başlangıç + uzunluk / 2.0,
                ad_genişliği: uzunluk,
                etiket_göster: true,
                ad_kısaltma: None,
            }],
            genisletilebilir: false,
            pencere: [0.0, 0.0],
            genişletme: None,
        };
    }

    let genişlik = model.eksen_genişletme_genişliği.clamp(0.0, uzunluk);
    let sayı = model.eksen_genişletme_sayısı.min(eksen_sayısı);
    let genişletilebilir = model.eksen_genişletilebilir
        && eksen_sayısı > 3
        && eksen_sayısı > sayı
        && sayı > 1
        && genişlik > 0.0
        && uzunluk > 0.0;
    if !genişletilebilir {
        let adım = uzunluk / (eksen_sayısı - 1) as f32;
        return EksenKonumları {
            eksenler: (0..eksen_sayısı)
                .map(|sıra| EksenKonumBilgisi {
                    konum: başlangıç + adım * sıra as f32,
                    ad_genişliği: adım,
                    etiket_göster: true,
                    ad_kısaltma: None,
                })
                .collect(),
            genisletilebilir: false,
            pencere: [0.0, 0.0],
            genişletme: None,
        };
    }

    let mut pencere = model.eksen_genişletme_penceresi.unwrap_or_else(|| {
        let pencere_boyutu = (genişlik * (sayı - 1) as f32).clamp(0.0, uzunluk);
        let merkez = model
            .eksen_genişletme_merkezi
            .unwrap_or((eksen_sayısı / 2) as f32);
        let başlangıç = genişlik * merkez - pencere_boyutu / 2.0;
        [başlangıç, başlangıç + pencere_boyutu]
    });
    let pencere_boyutu = (pencere[1] - pencere[0]).clamp(0.0, uzunluk);
    pencere[1] = pencere[0] + pencere_boyutu;
    let mut dar_genişlik = (uzunluk - pencere_boyutu) / (eksen_sayısı - sayı) as f32;
    if dar_genişlik < 3.0 {
        dar_genişlik = 0.0;
    }
    let onda_bir = |değer: f32| (değer * 10.0).round() / 10.0;
    let iç_baş = (onda_bir(pencere[0] / genişlik).floor() as isize + 1).max(0);
    let iç_son = (onda_bir(pencere[1] / genişlik).ceil() as isize - 1).max(-1);
    let pencere_konumu = dar_genişlik / genişlik * pencere[0];

    let eksenler = (0..eksen_sayısı)
        .map(|sıra| {
            let sıra_i = sıra as isize;
            let (konum, ad_genişliği, etiket_göster, ad_kısaltma) = if sıra_i < iç_baş {
                (
                    sıra as f32 * dar_genişlik,
                    dar_genişlik,
                    false,
                    Some(dar_genişlik),
                )
            } else if sıra_i <= iç_son {
                (
                    pencere_konumu + sıra as f32 * genişlik - pencere[0],
                    genişlik,
                    true,
                    None,
                )
            } else {
                (
                    uzunluk - (eksen_sayısı - 1 - sıra) as f32 * dar_genişlik,
                    dar_genişlik,
                    false,
                    Some(dar_genişlik),
                )
            };
            EksenKonumBilgisi {
                konum: başlangıç + konum,
                ad_genişliği,
                etiket_göster,
                ad_kısaltma,
            }
        })
        .collect();
    EksenKonumları {
        eksenler,
        genisletilebilir: true,
        pencere,
        genişletme: Some(ParalelGenişletmeBilgisi {
            genişlik,
            dar_genişlik,
            eksen_sayısı,
            pencere_konumu,
            kaydırma_tetik_alanı: model.eksen_genişletme_kaydırma_tetik_alanı,
        }),
    }
}

fn ölçek_kur(seçenek: &Eksen, kategoriler: Vec<String>, kapsam: [f64; 2]) -> Ölçek {
    let mut kapsam = kapsam;
    let veri_en_azı = kapsam[0];
    let veri_en_çoğu = kapsam[1];
    let en_az = seçenek
        .en_az
        .or_else(|| seçenek.en_az_veri.then_some(veri_en_azı));
    let en_çok = seçenek
        .en_çok
        .or_else(|| seçenek.en_çok_veri.then_some(veri_en_çoğu));
    if seçenek.tür != EksenTürü::Kategori
        && let Some([alt, üst]) = seçenek.sayısal_kenar_boşluğu
        && kapsam[0].is_finite()
        && kapsam[1].is_finite()
    {
        let fark = (kapsam[1] - kapsam[0]).abs();
        let açıklık = if fark > 0.0 { fark } else { kapsam[0].abs() };
        if en_az.is_none() {
            kapsam[0] -= alt.çöz(açıklık);
        }
        if en_çok.is_none() {
            kapsam[1] += üst.çöz(açıklık);
        }
    }
    match seçenek.tür {
        EksenTürü::Kategori => Ölçek::Kategorik(KategorikÖlçek::yeni(kategoriler)),
        EksenTürü::Değer => {
            let mut kırılma_kapsamı = kapsam;
            if seçenek.sıfırı_içer {
                kırılma_kapsamı[0] = kırılma_kapsamı[0].min(0.0);
                kırılma_kapsamı[1] = kırılma_kapsamı[1].max(0.0);
            }
            if let Some(en_az) = en_az {
                kırılma_kapsamı[0] = en_az;
            }
            if let Some(en_çok) = en_çok {
                kırılma_kapsamı[1] = en_çok;
            }
            let etkin_açıklık = KırılmaEşleyici::kur(&seçenek.kırılmalar, kırılma_kapsamı)
                .map(|eşleyici| eşleyici.etkin_açıklık());
            let mut ölçek = if let Some(etkin_açıklık) = etkin_açıklık {
                AralıkÖlçeği::kur_etkin_açıklıkla(
                    kapsam,
                    en_az,
                    en_çok,
                    seçenek.sıfırı_içer,
                    seçenek.bölme_sayısı,
                    seçenek.en_küçük_adım,
                    seçenek.en_büyük_adım,
                    etkin_açıklık,
                )
            } else {
                AralıkÖlçeği::kur(
                    kapsam,
                    en_az,
                    en_çok,
                    seçenek.sıfırı_içer,
                    seçenek.bölme_sayısı,
                    seçenek.en_küçük_adım,
                    seçenek.en_büyük_adım,
                )
            };
            if let Some(aralık) = seçenek.aralık {
                ölçek.açık_aralık_uygula(aralık);
            }
            Ölçek::Aralık(ölçek)
        }
        EksenTürü::Zaman => {
            let mut kapsam = kapsam;
            if let Some(en_az) = en_az {
                kapsam[0] = en_az;
            }
            if let Some(en_çok) = en_çok {
                kapsam[1] = en_çok;
            }
            let etkin_açıklık = KırılmaEşleyici::kur(&seçenek.kırılmalar, kapsam)
                .map(|eşleyici| eşleyici.etkin_açıklık())
                .unwrap_or_else(|| (kapsam[1] - kapsam[0]).abs());
            Ölçek::Zaman(ZamanÖlçeği::kur_etkin_açıklıkla(
                kapsam,
                seçenek.bölme_sayısı,
                etkin_açıklık,
            ))
        }
        EksenTürü::Log => Ölçek::Log(LogÖlçeği::kur(
            kapsam,
            seçenek.log_tabanı,
            en_az,
            en_çok,
            seçenek.bölme_sayısı,
        )),
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::model::Uzunluk;

    fn seri() -> ParalelSerisi {
        ParalelSerisi::yeni()
            .boyutlar(["Fiyat", "Ağırlık", "Puan", "Sınıf"])
            .karma_veri([
                vec![
                    VeriDeğeri::from(12.99),
                    VeriDeğeri::from(100),
                    VeriDeğeri::from(82),
                    VeriDeğeri::from("Good"),
                ],
                vec![
                    VeriDeğeri::from(9.99),
                    VeriDeğeri::from(80),
                    VeriDeğeri::from(77),
                    VeriDeğeri::from("OK"),
                ],
                vec![
                    VeriDeğeri::from(20),
                    VeriDeğeri::from(120),
                    VeriDeğeri::from(60),
                    VeriDeğeri::from("Excellent"),
                ],
            ])
    }

    #[test]
    fn resmi_basic_parallel_kutusunu_ve_kategori_olcegini_cozer() {
        let seri = seri();
        let axes = vec![
            ParalelEkseni::yeni(0).ad("Price"),
            ParalelEkseni::yeni(1).ad("Net Weight"),
            ParalelEkseni::yeni(2).ad("Amount"),
            ParalelEkseni::yeni(3)
                .ad("Score")
                .kategori()
                .veri(["Excellent", "Good", "OK", "Bad"]),
        ];
        let yerleşim = ParalelYerleşimi::kur(
            &ParalelKoordinatı::yeni(),
            0,
            &axes,
            &[&seri],
            (600.0, 450.0),
        )
        .unwrap();
        assert_eq!(yerleşim.alan, Dikdörtgen::yeni(80.0, 60.0, 440.0, 330.0));
        assert_eq!(yerleşim.eksenler.len(), 4);
        assert!(
            yerleşim
                .eksenler
                .iter()
                .all(|eksen| eksen.eksen.seçenek.z == 10)
        );
        assert!((yerleşim.eksenler[1].konum - (80.0 + 440.0 / 3.0)).abs() < 1e-4);
        let kategori = &yerleşim.eksenler[3];
        assert_eq!(kategori.eksen.ölçek.kategori_sayısı(), 4);
        let excellent = yerleşim.veriden_noktaya(&seri.veri[2], kategori).unwrap();
        assert!(excellent.1 > yerleşim.alan.y + yerleşim.alan.yükseklik * 0.8);
    }

    #[test]
    fn dikey_yerlesimde_eksenleri_y_boyutuna_dizer() {
        let seri = seri();
        let yerleşim = ParalelYerleşimi::kur(
            &ParalelKoordinatı::yeni()
                .yerleşim(ParalelYerleşim::Dikey)
                .sol(Uzunluk::Piksel(20.0))
                .sağ(Uzunluk::Piksel(20.0)),
            0,
            &[],
            &[&seri],
            (600.0, 450.0),
        )
        .unwrap();
        assert_eq!(yerleşim.eksenler.first().unwrap().konum, yerleşim.alan.y);
        assert_eq!(yerleşim.eksenler.last().unwrap().konum, yerleşim.alan.alt());
        let ilk = yerleşim
            .veriden_noktaya(&seri.veri[0], &yerleşim.eksenler[0])
            .unwrap();
        assert_eq!(ilk.1, yerleşim.alan.y);
    }

    #[test]
    fn genisletme_penceresi_disindaki_eksenleri_daraltir() {
        let seri = ParalelSerisi::yeni()
            .boyutlar((0..8).map(|i| format!("B{i}")))
            .veri([VeriÖğesi::from(vec![1.0; 8])]);
        let yerleşim = ParalelYerleşimi::kur(
            &ParalelKoordinatı::yeni()
                .eksen_genişletilebilir(true)
                .eksen_genişletme_sayısı(3)
                .eksen_genişletme_genişliği(50.0)
                .eksen_genişletme_penceresi([100.0, 200.0]),
            0,
            &[],
            &[&seri],
            (700.0, 500.0),
        )
        .unwrap();
        assert!(yerleşim.genişletilebilir);
        assert_eq!(yerleşim.genişletme_penceresi, [100.0, 200.0]);
        let aralıklar = yerleşim
            .eksenler
            .windows(2)
            .map(|çift| çift[1].konum - çift[0].konum)
            .collect::<Vec<_>>();
        assert!(aralıklar.iter().any(|aralık| (*aralık - 50.0).abs() < 1e-4));
        assert!(yerleşim.eksenler.iter().any(|eksen| !eksen.etiket_göster));
    }

    #[test]
    fn genisletme_faresi_resmi_merkez_slide_ve_jump_kurallarini_uygular() {
        let seri = ParalelSerisi::yeni()
            .boyutlar((0..8).map(|i| format!("B{i}")))
            .veri([VeriÖğesi::from(vec![1.0; 8])]);
        let yerleşim = ParalelYerleşimi::kur(
            &ParalelKoordinatı::yeni()
                .eksen_genişletilebilir(true)
                .eksen_genişletme_sayısı(3)
                .eksen_genişletme_genişliği(50.0)
                .eksen_genişletme_penceresi([100.0, 200.0]),
            0,
            &[],
            &[&seri],
            (700.0, 500.0),
        )
        .unwrap();

        // Geniş pencerenin merkezindeki güvenli alan hareket üretmez.
        let merkez = yerleşim.genişletme_penceresini_çöz((306.0, 250.0));
        assert_eq!(merkez.davranış, ParalelGenişletmeDavranışı::Yok);
        assert_eq!(merkez.pencere, [100.0, 200.0]);

        // Sağ dış tetik alanı jump uygular ve pencereyi sanal eksen
        // kapsamının sağ ucunda boyutunu bozmadan sınırlar.
        let sağ = yerleşim.genişletme_penceresini_çöz((yerleşim.alan.sağ(), 250.0));
        assert_eq!(sağ.davranış, ParalelGenişletmeDavranışı::Atla);
        assert!((sağ.pencere[0] - 250.0).abs() < 1e-3, "{:?}", sağ.pencere);
        assert!((sağ.pencere[1] - 350.0).abs() < 1e-3, "{:?}", sağ.pencere);

        let dış = yerleşim.genişletme_penceresini_çöz((0.0, 0.0));
        assert_eq!(dış.davranış, ParalelGenişletmeDavranışı::Yok);
    }

    #[test]
    fn alan_secimi_butun_eksenlerde_kesisimdir() {
        let seri = seri();
        let axes = vec![
            ParalelEkseni::yeni(0).etkin_aralık(10.0, 15.0),
            ParalelEkseni::yeni(1).etkin_aralık(90.0, 110.0),
            ParalelEkseni::yeni(2),
            ParalelEkseni::yeni(3)
                .kategori()
                .veri(["Excellent", "Good", "OK"]),
        ];
        let yerleşim = ParalelYerleşimi::kur(
            &ParalelKoordinatı::yeni(),
            0,
            &axes,
            &[&seri],
            (600.0, 450.0),
        )
        .unwrap();
        assert_eq!(
            yerleşim.veri_etkinliği(&seri.veri[0]),
            ParalelEtkinlik::Etkin
        );
        assert_eq!(
            yerleşim.veri_etkinliği(&seri.veri[1]),
            ParalelEtkinlik::EtkinDeğil
        );
    }
}
