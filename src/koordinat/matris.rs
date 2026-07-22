//! ECharts 6.1 Matrix koordinat yerleşimi (`coord/matrix/Matrix.ts`).

use std::collections::{BTreeMap, BTreeSet};

use crate::hata::BilesenHatasi;
use crate::koordinat::Dikdörtgen;
use crate::model::Uzunluk;
use crate::model::matris::{
    MatrisAralığı, MatrisBoyutHücresi, MatrisBoyutu, MatrisEtiketiBiçimleyicisi, MatrisKonumu,
    MatrisKoordinatı,
};
use crate::model::stil::{Etiket, ÖğeStili};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrisHücreTürü {
    XBaşlığı,
    YBaşlığı,
    Gövde,
    BirleşikGövde,
    Köşe,
    BirleşikKöşe,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatrisHücreYerleşimi {
    pub tür: MatrisHücreTürü,
    pub değer: Option<String>,
    pub kutu: Dikdörtgen,
    pub x_aralığı: [isize; 2],
    pub y_aralığı: [isize; 2],
    pub öğe_stili: ÖğeStili,
    pub etiket: Etiket,
    pub etiket_bağlamlı_biçimleyici: Option<MatrisEtiketiBiçimleyicisi>,
    pub imleç: Option<String>,
    pub sessiz: bool,
    pub z2: i32,
}

#[derive(Clone, Debug)]
struct BoyutDüğümü {
    değer: String,
    başlangıç: usize,
    bitiş: usize,
    derinlik: usize,
    yaprak: bool,
    öğe_stili: Option<ÖğeStili>,
    etiket: Option<Etiket>,
    etiket_bağlamlı_biçimleyici: Option<MatrisEtiketiBiçimleyicisi>,
    imleç: Option<String>,
    sessiz: Option<bool>,
    z2: Option<i32>,
}

#[derive(Clone, Debug)]
struct BoyutYerleşimi {
    yapraklar: Vec<String>,
    sınırlar: Vec<f32>,
    düğümler: Vec<BoyutDüğümü>,
    aralıklar: BTreeMap<String, [usize; 2]>,
    seviye_sınırları: Vec<f32>,
}

/// Matrix'in çözümlenmiş gövde/başlık geometrisi.
#[derive(Clone, Debug)]
pub struct MatrisYerleşimi {
    pub dış_kutu: Dikdörtgen,
    pub gövde_kutusu: Dikdörtgen,
    x: BoyutYerleşimi,
    y: BoyutYerleşimi,
    pub hücreler: Vec<MatrisHücreYerleşimi>,
    birleşimler: Vec<([usize; 2], [usize; 2])>,
    köşe_birleşimleri: Vec<([usize; 2], [usize; 2])>,
    pub bileşen_sırası: usize,
}

impl MatrisYerleşimi {
    /// `yedek`, x/y seçeneklerinde data/length yoksa seri verisinden çıkarılan
    /// gövde sütun/satır sayısıdır.
    pub fn kur(
        seçenek: &MatrisKoordinatı,
        tuval: (f32, f32),
        yedek: (usize, usize),
    ) -> Result<Self, BilesenHatasi> {
        Self::kur_sıralı(seçenek, tuval, yedek, 0)
    }

    /// Birden çok `matrix` bileşeninde formatter/event bağlamına taşınacak
    /// `matrixIndex` ile yerleşim kurar.
    pub fn kur_sıralı(
        seçenek: &MatrisKoordinatı,
        tuval: (f32, f32),
        yedek: (usize, usize),
        bileşen_sırası: usize,
    ) -> Result<Self, BilesenHatasi> {
        let yedek_x = (0..yedek.0)
            .map(|sıra| sıra.to_string())
            .collect::<Vec<_>>();
        let yedek_y = (0..yedek.1)
            .map(|sıra| sıra.to_string())
            .collect::<Vec<_>>();
        Self::kur_adlarla_sıralı(seçenek, tuval, (&yedek_x, &yedek_y), bileşen_sırası)
    }

    /// `matrix.x/y.data` verilmediğinde bağlı serilerden toplanmış kategori
    /// adlarını kullanır (`collect` aşamasının Rust karşılığı).
    pub fn kur_adlarla_sıralı(
        seçenek: &MatrisKoordinatı,
        tuval: (f32, f32),
        yedek: (&[String], &[String]),
        bileşen_sırası: usize,
    ) -> Result<Self, BilesenHatasi> {
        let dış_kutu = kutuyu_çöz(seçenek, tuval)?;
        let x_taslak = boyut_taslağı(&seçenek.x, yedek.0, "x")?;
        let y_taslak = boyut_taslağı(&seçenek.y, yedek.1, "y")?;

        // ECharts `layOutUnitsOnDimension`: aynı fiziksel boyuttaki ana
        // yapraklar ile karşı boyutun başlık seviyeleri toplam alanı birlikte
        // paylaşır. Açık `size`/`levelSize` değerleri önce ayrılır, kalan bütün
        // belirtilmemiş birimlere eşit dağıtılır.
        let x_seviye_seçenekleri = seviye_boyut_seçenekleri(&seçenek.x, x_taslak.2);
        let y_seviye_seçenekleri = seviye_boyut_seçenekleri(&seçenek.y, y_taslak.2);
        let (x_yaprak_boyutları, y_seviye_boyutları) = boyut_boyutlarını_çöz(
            &x_taslak.3,
            &y_seviye_seçenekleri,
            seçenek.y.göster,
            dış_kutu.genişlik,
        );
        let (y_yaprak_boyutları, x_seviye_boyutları) = boyut_boyutlarını_çöz(
            &y_taslak.3,
            &x_seviye_seçenekleri,
            seçenek.x.göster,
            dış_kutu.yükseklik,
        );
        let x_başlık: f32 = x_seviye_boyutları.iter().sum();
        let y_başlık: f32 = y_seviye_boyutları.iter().sum();
        let gövde_kutusu = Dikdörtgen::yeni(
            dış_kutu.x + y_başlık,
            dış_kutu.y + x_başlık,
            x_yaprak_boyutları.iter().sum(),
            y_yaprak_boyutları.iter().sum(),
        );
        let x = boyutu_yerleştir(
            x_taslak,
            gövde_kutusu.x,
            x_yaprak_boyutları,
            &x_seviye_boyutları,
            dış_kutu.y,
        );
        let y = boyutu_yerleştir(
            y_taslak,
            gövde_kutusu.y,
            y_yaprak_boyutları,
            &y_seviye_boyutları,
            dış_kutu.x,
        );

        let mut yerleşim = Self {
            dış_kutu,
            gövde_kutusu,
            x,
            y,
            hücreler: Vec::new(),
            birleşimler: Vec::new(),
            köşe_birleşimleri: Vec::new(),
            bileşen_sırası,
        };
        yerleşim.hücreleri_üret(seçenek)?;
        Ok(yerleşim)
    }

    pub fn x_sayısı(&self) -> usize {
        self.x.yapraklar.len()
    }

    pub fn y_sayısı(&self) -> usize {
        self.y.yapraklar.len()
    }

    pub fn x_değeri(&self, sıra: usize) -> Option<&str> {
        self.x.yapraklar.get(sıra).map(String::as_str)
    }

    pub fn y_değeri(&self, sıra: usize) -> Option<&str> {
        self.y.yapraklar.get(sıra).map(String::as_str)
    }

    pub fn veriden_yerleşime(
        &self,
        x: &MatrisAralığı,
        y: &MatrisAralığı,
        birleşimleri_genişlet: bool,
    ) -> Option<Dikdörtgen> {
        let x_negatif = aralık_negatif_mi(x)?;
        let y_negatif = aralık_negatif_mi(y)?;
        match (x_negatif, y_negatif) {
            (false, false) => {
                let mut xr = aralığı_çöz(&self.x, x)?;
                let mut yr = aralığı_çöz(&self.y, y)?;
                if birleşimleri_genişlet {
                    birleşik_aralığı_genişlet(&mut xr, &mut yr, &self.birleşimler);
                }
                aralık_kutusu(&self.x, &self.y, xr, yr)
            }
            (false, true) => {
                let xr = aralığı_çöz(&self.x, x)?;
                let yr = negatif_aralığı_çöz(y, seviye_sayısı(&self.x))?;
                x_başlık_aralık_kutusu(&self.x, xr, yr)
            }
            (true, false) => {
                let xr = negatif_aralığı_çöz(x, seviye_sayısı(&self.y))?;
                let yr = aralığı_çöz(&self.y, y)?;
                y_başlık_aralık_kutusu(&self.y, xr, yr)
            }
            (true, true) => {
                let mut xr = negatif_aralığı_çöz(x, seviye_sayısı(&self.y))?;
                let mut yr = negatif_aralığı_çöz(y, seviye_sayısı(&self.x))?;
                if birleşimleri_genişlet {
                    birleşik_aralığı_genişlet(&mut xr, &mut yr, &self.köşe_birleşimleri);
                }
                köşe_aralık_kutusu(&self.x, &self.y, xr, yr)
            }
        }
    }

    pub fn veriden_noktaya(
        &self,
        x: impl Into<MatrisAralığı>,
        y: impl Into<MatrisAralığı>,
    ) -> Option<(f32, f32)> {
        self.veriden_yerleşime(&x.into(), &y.into(), true)
            .map(|kutu| kutu.merkez())
    }

    /// Gövde içinde yaprak sıra çiftini, başlıkta negatif seviye konumunu
    /// döndürür. Dışarıdaki boyut `None` olur.
    pub fn noktadan_veriye(&self, nokta: (f32, f32)) -> [Option<isize>; 2] {
        [
            noktadan_ana_boyuta(&self.x, &self.y, nokta.0),
            noktadan_ana_boyuta(&self.y, &self.x, nokta.1),
        ]
    }

    pub fn içeriyor_mu(&self, nokta: (f32, f32)) -> bool {
        self.dış_kutu.içeriyor_mu(nokta)
    }

    fn hücreleri_üret(&mut self, seçenek: &MatrisKoordinatı) -> Result<(), BilesenHatasi> {
        for düğüm in &self.x.düğümler {
            if !seçenek.x.göster {
                continue;
            }
            let Some(kutu) = x_başlık_kutusu(&self.x, düğüm) else {
                continue;
            };
            let öğe_stili = düğüm
                .öğe_stili
                .clone()
                .unwrap_or_else(|| seçenek.x.öğe_stili.clone());
            let sessiz = düğüm
                .sessiz
                .or(seçenek.x.sessiz)
                .unwrap_or(öğe_stili.renk.is_none());
            self.hücreler.push(MatrisHücreYerleşimi {
                tür: MatrisHücreTürü::XBaşlığı,
                değer: Some(düğüm.değer.clone()),
                kutu,
                x_aralığı: [düğüm.başlangıç as isize, düğüm.bitiş as isize],
                y_aralığı: düğüm_seviye_konumu(düğüm, seviye_sayısı(&self.x)),
                öğe_stili,
                etiket: düğüm
                    .etiket
                    .clone()
                    .unwrap_or_else(|| seçenek.x.etiket.clone()),
                etiket_bağlamlı_biçimleyici: düğüm
                    .etiket_bağlamlı_biçimleyici
                    .clone()
                    .or_else(|| seçenek.x.etiket_bağlamlı_biçimleyici.clone()),
                imleç: düğüm.imleç.clone().or_else(|| seçenek.x.imleç.clone()),
                sessiz,
                z2: düğüm.z2.or(seçenek.x.z2).unwrap_or(50),
            });
        }
        for düğüm in &self.y.düğümler {
            if !seçenek.y.göster {
                continue;
            }
            let Some(kutu) = y_başlık_kutusu(&self.y, düğüm) else {
                continue;
            };
            let öğe_stili = düğüm
                .öğe_stili
                .clone()
                .unwrap_or_else(|| seçenek.y.öğe_stili.clone());
            let sessiz = düğüm
                .sessiz
                .or(seçenek.y.sessiz)
                .unwrap_or(öğe_stili.renk.is_none());
            self.hücreler.push(MatrisHücreYerleşimi {
                tür: MatrisHücreTürü::YBaşlığı,
                değer: Some(düğüm.değer.clone()),
                kutu,
                x_aralığı: düğüm_seviye_konumu(düğüm, seviye_sayısı(&self.y)),
                y_aralığı: [düğüm.başlangıç as isize, düğüm.bitiş as isize],
                öğe_stili,
                etiket: düğüm
                    .etiket
                    .clone()
                    .unwrap_or_else(|| seçenek.y.etiket.clone()),
                etiket_bağlamlı_biçimleyici: düğüm
                    .etiket_bağlamlı_biçimleyici
                    .clone()
                    .or_else(|| seçenek.y.etiket_bağlamlı_biçimleyici.clone()),
                imleç: düğüm.imleç.clone().or_else(|| seçenek.y.imleç.clone()),
                sessiz,
                z2: düğüm.z2.or(seçenek.y.z2).unwrap_or(50),
            });
        }

        let mut özel = BTreeSet::new();
        for hücre in &seçenek.gövde_verisi {
            let x = aralığı_çöz_sınırlı(&self.x, &hücre.x, hücre.koordinatı_sınırla).ok_or_else(
                || BilesenHatasi::GeçersizSeçenek {
                    alan: "matrix.body.data.coord.x",
                    ayrıntı: format!("{:?} konumu çözülemedi", hücre.x),
                },
            )?;
            let y = aralığı_çöz_sınırlı(&self.y, &hücre.y, hücre.koordinatı_sınırla).ok_or_else(
                || BilesenHatasi::GeçersizSeçenek {
                    alan: "matrix.body.data.coord.y",
                    ayrıntı: format!("{:?} konumu çözülemedi", hücre.y),
                },
            )?;
            let Some(kutu) = aralık_kutusu(&self.x, &self.y, x, y) else {
                continue;
            };
            if hücre.hücreleri_birleştir {
                self.birleşimler.push((x, y));
            }
            for xs in x[0]..=x[1] {
                for ys in y[0]..=y[1] {
                    özel.insert((xs, ys));
                }
            }
            let öğe_stili = hücre
                .öğe_stili
                .clone()
                .unwrap_or_else(|| seçenek.gövde_stili.clone());
            let sessiz = hücre
                .sessiz
                .or(seçenek.gövde_sessiz)
                .unwrap_or(öğe_stili.renk.is_none());
            self.hücreler.push(MatrisHücreYerleşimi {
                tür: if hücre.hücreleri_birleştir {
                    MatrisHücreTürü::BirleşikGövde
                } else {
                    MatrisHücreTürü::Gövde
                },
                değer: hücre.değer.clone(),
                kutu,
                x_aralığı: [x[0] as isize, x[1] as isize],
                y_aralığı: [y[0] as isize, y[1] as isize],
                öğe_stili,
                etiket: hücre
                    .etiket
                    .clone()
                    .unwrap_or_else(|| seçenek.gövde_etiketi.clone()),
                etiket_bağlamlı_biçimleyici: hücre
                    .etiket_bağlamlı_biçimleyici
                    .clone()
                    .or_else(|| seçenek.gövde_etiketi_bağlamlı_biçimleyici.clone()),
                imleç: hücre.imleç.clone().or_else(|| seçenek.gövde_imleci.clone()),
                sessiz,
                z2: hücre
                    .z2
                    .or(seçenek.gövde_z2)
                    .unwrap_or(if hücre.öğe_stili.is_some() {
                        100
                    } else {
                        25
                    }),
            });
        }
        for x in 0..self.x_sayısı() {
            for y in 0..self.y_sayısı() {
                if özel.contains(&(x, y)) {
                    continue;
                }
                let Some(kutu) = aralık_kutusu(&self.x, &self.y, [x, x], [y, y]) else {
                    continue;
                };
                self.hücreler.push(MatrisHücreYerleşimi {
                    tür: MatrisHücreTürü::Gövde,
                    değer: None,
                    kutu,
                    x_aralığı: [x as isize, x as isize],
                    y_aralığı: [y as isize, y as isize],
                    öğe_stili: seçenek.gövde_stili.clone(),
                    etiket: seçenek.gövde_etiketi.clone(),
                    etiket_bağlamlı_biçimleyici: seçenek
                        .gövde_etiketi_bağlamlı_biçimleyici
                        .clone(),
                    imleç: seçenek.gövde_imleci.clone(),
                    sessiz: seçenek
                        .gövde_sessiz
                        .unwrap_or(seçenek.gövde_stili.renk.is_none()),
                    z2: seçenek.gövde_z2.unwrap_or(25),
                });
            }
        }
        self.köşe_hücrelerini_üret(seçenek)?;
        Ok(())
    }

    fn köşe_hücrelerini_üret(
        &mut self,
        seçenek: &MatrisKoordinatı,
    ) -> Result<(), BilesenHatasi> {
        if !seçenek.x.göster || !seçenek.y.göster {
            return Ok(());
        }
        let x_seviye = seviye_sayısı(&self.y);
        let y_seviye = seviye_sayısı(&self.x);
        let mut özel = BTreeSet::new();
        for hücre in &seçenek.köşe_verisi {
            let x =
                negatif_aralığı_çöz_sınırlı(&hücre.x, x_seviye, hücre.koordinatı_sınırla)
                    .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                        alan: "matrix.corner.data.coord.x",
                        ayrıntı: format!("{:?} negatif köşe konumu çözülemedi", hücre.x),
                    })?;
            let y =
                negatif_aralığı_çöz_sınırlı(&hücre.y, y_seviye, hücre.koordinatı_sınırla)
                    .ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                        alan: "matrix.corner.data.coord.y",
                        ayrıntı: format!("{:?} negatif köşe konumu çözülemedi", hücre.y),
                    })?;
            let Some(kutu) = köşe_aralık_kutusu(&self.x, &self.y, x, y) else {
                continue;
            };
            if hücre.hücreleri_birleştir {
                self.köşe_birleşimleri.push((x, y));
            }
            for xs in x[0]..=x[1] {
                for ys in y[0]..=y[1] {
                    özel.insert((xs, ys));
                }
            }
            let öğe_stili = hücre
                .öğe_stili
                .clone()
                .unwrap_or_else(|| seçenek.köşe_stili.clone());
            let sessiz = hücre
                .sessiz
                .or(seçenek.köşe_sessiz)
                .unwrap_or(öğe_stili.renk.is_none());
            self.hücreler.push(MatrisHücreYerleşimi {
                tür: if hücre.hücreleri_birleştir {
                    MatrisHücreTürü::BirleşikKöşe
                } else {
                    MatrisHücreTürü::Köşe
                },
                değer: hücre.değer.clone(),
                kutu,
                x_aralığı: seviye_indeksini_konuma(x, x_seviye),
                y_aralığı: seviye_indeksini_konuma(y, y_seviye),
                öğe_stili,
                etiket: hücre
                    .etiket
                    .clone()
                    .unwrap_or_else(|| seçenek.köşe_etiketi.clone()),
                etiket_bağlamlı_biçimleyici: hücre
                    .etiket_bağlamlı_biçimleyici
                    .clone()
                    .or_else(|| seçenek.köşe_etiketi_bağlamlı_biçimleyici.clone()),
                imleç: hücre.imleç.clone().or_else(|| seçenek.köşe_imleci.clone()),
                sessiz,
                z2: hücre
                    .z2
                    .or(seçenek.köşe_z2)
                    .unwrap_or(if hücre.öğe_stili.is_some() {
                        100
                    } else {
                        25
                    }),
            });
        }
        for x in 0..x_seviye {
            for y in 0..y_seviye {
                if özel.contains(&(x, y)) {
                    continue;
                }
                let Some(kutu) = köşe_aralık_kutusu(&self.x, &self.y, [x, x], [y, y]) else {
                    continue;
                };
                self.hücreler.push(MatrisHücreYerleşimi {
                    tür: MatrisHücreTürü::Köşe,
                    değer: None,
                    kutu,
                    x_aralığı: seviye_indeksini_konuma([x, x], x_seviye),
                    y_aralığı: seviye_indeksini_konuma([y, y], y_seviye),
                    öğe_stili: seçenek.köşe_stili.clone(),
                    etiket: seçenek.köşe_etiketi.clone(),
                    etiket_bağlamlı_biçimleyici: seçenek
                        .köşe_etiketi_bağlamlı_biçimleyici
                        .clone(),
                    imleç: seçenek.köşe_imleci.clone(),
                    sessiz: seçenek
                        .köşe_sessiz
                        .unwrap_or(seçenek.köşe_stili.renk.is_none()),
                    z2: seçenek.köşe_z2.unwrap_or(25),
                });
            }
        }
        Ok(())
    }
}

type BoyutTaslağı = (Vec<String>, Vec<BoyutDüğümü>, usize, Vec<Option<Uzunluk>>);

fn boyut_taslağı(
    seçenek: &MatrisBoyutu,
    yedek: &[String],
    ad: &'static str,
) -> Result<BoyutTaslağı, BilesenHatasi> {
    let veri = if seçenek.veri.is_empty() {
        seçenek.uzunluk.map_or_else(
            || {
                yedek
                    .iter()
                    .cloned()
                    .map(MatrisBoyutHücresi::yeni)
                    .collect()
            },
            |uzunluk| {
                (0..uzunluk)
                    .map(|sıra| MatrisBoyutHücresi::yeni(sıra.to_string()))
                    .collect()
            },
        )
    } else {
        seçenek.veri.clone()
    };
    let mut yapraklar = Vec::new();
    let mut düğümler = Vec::new();
    let mut boyutlar = Vec::new();
    let mut görülen = BTreeSet::new();
    let mut en_derin = 0usize;
    for hücre in &veri {
        düğümü_düzleştir(
            hücre,
            0,
            &mut yapraklar,
            &mut düğümler,
            &mut boyutlar,
            &mut görülen,
            &mut en_derin,
            ad,
        )?;
    }
    Ok((yapraklar, düğümler, en_derin.saturating_add(1), boyutlar))
}

#[allow(clippy::too_many_arguments)]
fn düğümü_düzleştir(
    hücre: &MatrisBoyutHücresi,
    derinlik: usize,
    yapraklar: &mut Vec<String>,
    düğümler: &mut Vec<BoyutDüğümü>,
    boyutlar: &mut Vec<Option<Uzunluk>>,
    görülen: &mut BTreeSet<String>,
    en_derin: &mut usize,
    boyut_adı: &'static str,
) -> Result<[usize; 2], BilesenHatasi> {
    if hücre.değer.is_empty() || !görülen.insert(hücre.değer.clone()) {
        return Err(BilesenHatasi::GeçersizSeçenek {
            alan: "matrix.dimension.data.value",
            ayrıntı: format!("matrix.{boyut_adı} değerleri boş/yinelenen olamaz"),
        });
    }
    *en_derin = (*en_derin).max(derinlik);
    let başlangıç = yapraklar.len();
    if hücre.çocuklar.is_empty() {
        yapraklar.push(hücre.değer.clone());
        boyutlar.push(hücre.boyut);
    } else {
        for çocuk in &hücre.çocuklar {
            let _ = düğümü_düzleştir(
                çocuk,
                derinlik.saturating_add(1),
                yapraklar,
                düğümler,
                boyutlar,
                görülen,
                en_derin,
                boyut_adı,
            )?;
        }
    }
    let bitiş = yapraklar.len().saturating_sub(1);
    düğümler.push(BoyutDüğümü {
        değer: hücre.değer.clone(),
        başlangıç,
        bitiş,
        derinlik,
        yaprak: hücre.çocuklar.is_empty(),
        öğe_stili: hücre.öğe_stili.clone(),
        etiket: hücre.etiket.clone(),
        etiket_bağlamlı_biçimleyici: hücre.etiket_bağlamlı_biçimleyici.clone(),
        imleç: hücre.imleç.clone(),
        sessiz: hücre.sessiz,
        z2: hücre.z2,
    });
    Ok([başlangıç, bitiş])
}

fn boyutu_yerleştir(
    taslak: BoyutTaslağı,
    başlangıç: f32,
    yaprak_boyutları: Vec<f32>,
    seviye_boyutları: &[f32],
    seviye_başı: f32,
) -> BoyutYerleşimi {
    let (yapraklar, düğümler, _derinlik, _boyutlar) = taslak;
    let mut sınırlar = vec![başlangıç];
    let mut geçerli = başlangıç;
    for uzunluk in yaprak_boyutları {
        geçerli += uzunluk;
        sınırlar.push(geçerli);
    }
    let mut aralıklar = BTreeMap::new();
    for düğüm in &düğümler {
        aralıklar.insert(düğüm.değer.clone(), [düğüm.başlangıç, düğüm.bitiş]);
    }
    let mut seviye_sınırları = vec![seviye_başı];
    let mut seviye = seviye_başı;
    for boyut in seviye_boyutları {
        seviye += *boyut;
        seviye_sınırları.push(seviye);
    }
    BoyutYerleşimi {
        yapraklar,
        sınırlar,
        düğümler,
        aralıklar,
        seviye_sınırları,
    }
}

fn seviye_boyut_seçenekleri(seçenek: &MatrisBoyutu, derinlik: usize) -> Vec<Option<Uzunluk>> {
    (0..derinlik)
        .map(|sıra| {
            seçenek
                .seviye_boyutları
                .get(sıra)
                .copied()
                .flatten()
                .or(seçenek.seviye_boyutu)
        })
        .collect()
}

fn boyut_boyutlarını_çöz(
    yapraklar: &[Option<Uzunluk>],
    seviyeler: &[Option<Uzunluk>],
    seviyeler_görünür: bool,
    toplam: f32,
) -> (Vec<f32>, Vec<f32>) {
    let mut yaprak_boyutları = vec![f32::NAN; yapraklar.len()];
    let mut seviye_boyutları = vec![f32::NAN; seviyeler.len()];
    let mut kalan = toplam.max(0.0);
    let mut kalan_birim = yapraklar.len() + seviyeler.len();

    // Resmî sıra: önce karşı boyutun seviyeleri, sonra ana boyutun yaprakları.
    for (sıra, seçenek) in seviyeler.iter().enumerate() {
        let açık = if seviyeler_görünür {
            seçenek.map(|boyut| boyut.çöz(toplam).max(0.0))
        } else {
            Some(0.0)
        };
        if let Some(açık) = açık {
            let açık = açık.min(kalan);
            seviye_boyutları[sıra] = açık;
            kalan -= açık;
            kalan_birim = kalan_birim.saturating_sub(1);
        }
    }
    for (sıra, seçenek) in yapraklar.iter().enumerate() {
        if let Some(açık) = seçenek.map(|boyut| boyut.çöz(toplam).max(0.0)) {
            let açık = açık.min(kalan);
            yaprak_boyutları[sıra] = açık;
            kalan -= açık;
            kalan_birim = kalan_birim.saturating_sub(1);
        }
    }
    let pay = if kalan_birim > 0 {
        kalan / kalan_birim as f32
    } else {
        0.0
    };
    for boyut in seviye_boyutları.iter_mut().chain(&mut yaprak_boyutları) {
        if boyut.is_nan() {
            *boyut = pay;
        }
    }
    (yaprak_boyutları, seviye_boyutları)
}

fn kutuyu_çöz(
    seçenek: &MatrisKoordinatı,
    tuval: (f32, f32),
) -> Result<Dikdörtgen, BilesenHatasi> {
    let sol = seçenek.sol.çöz(tuval.0);
    let sağ = seçenek.sağ.çöz(tuval.0);
    let üst = seçenek.üst.çöz(tuval.1);
    let alt = seçenek.alt.çöz(tuval.1);
    let kalan_genişlik = tuval.0 - sol - sağ;
    let kalan_yükseklik = tuval.1 - üst - alt;
    let genişlik = seçenek
        .genişlik
        .map(|değer| değer.çöz(tuval.0))
        .unwrap_or(kalan_genişlik);
    let yükseklik = seçenek
        .yükseklik
        .map(|değer| değer.çöz(tuval.1))
        .unwrap_or(kalan_yükseklik);
    let x = if seçenek.yatay_ortala {
        (tuval.0 - genişlik) / 2.0
    } else {
        sol
    };
    let y = if seçenek.dikey_ortala {
        (tuval.1 - yükseklik) / 2.0
    } else {
        üst
    };
    if ![x, y, sağ, alt, genişlik, yükseklik]
        .into_iter()
        .all(f32::is_finite)
        || genişlik <= 0.0
        || yükseklik <= 0.0
    {
        return Err(BilesenHatasi::GeçersizSeçenek {
            alan: "matrix.layout",
            ayrıntı: "matrix kutusu pozitif ve sonlu olmalı".to_owned(),
        });
    }
    Ok(Dikdörtgen::yeni(x, y, genişlik, yükseklik))
}

fn aralığı_çöz(boyut: &BoyutYerleşimi, aralık: &MatrisAralığı) -> Option<[usize; 2]> {
    aralığı_çöz_sınırlı(boyut, aralık, false)
}

fn aralığı_çöz_sınırlı(
    boyut: &BoyutYerleşimi,
    aralık: &MatrisAralığı,
    sınırla: bool,
) -> Option<[usize; 2]> {
    match aralık {
        MatrisAralığı::Tümü => {
            (!boyut.yapraklar.is_empty()).then_some([0, boyut.yapraklar.len() - 1])
        }
        MatrisAralığı::Tek(konum) => konumu_çöz_sınırlı(boyut, konum, sınırla),
        MatrisAralığı::Aralık(baş, son) => {
            let a = konumu_çöz_sınırlı(boyut, baş, sınırla)?;
            let b = konumu_çöz_sınırlı(boyut, son, sınırla)?;
            Some([a[0].min(b[0]), a[1].max(b[1])])
        }
    }
}

fn konumu_çöz_sınırlı(
    boyut: &BoyutYerleşimi,
    konum: &MatrisKonumu,
    sınırla: bool,
) -> Option<[usize; 2]> {
    match konum {
        MatrisKonumu::Sıra(sıra) if *sıra >= 0 => {
            let mut sıra = usize::try_from(*sıra).ok()?;
            if sınırla {
                sıra = sıra.min(boyut.yapraklar.len().checked_sub(1)?);
            }
            (sıra < boyut.yapraklar.len()).then_some([sıra, sıra])
        }
        MatrisKonumu::Değer(değer) => boyut.aralıklar.get(değer).copied(),
        MatrisKonumu::Sıra(_) => None,
    }
}

fn aralık_negatif_mi(aralık: &MatrisAralığı) -> Option<bool> {
    let konum = |konum: &MatrisKonumu| match konum {
        MatrisKonumu::Sıra(sıra) => Some(*sıra < 0),
        MatrisKonumu::Değer(_) => Some(false),
    };
    match aralık {
        MatrisAralığı::Tümü => Some(false),
        MatrisAralığı::Tek(değer) => konum(değer),
        MatrisAralığı::Aralık(baş, son) => {
            let baş = konum(baş)?;
            let son = konum(son)?;
            (baş == son).then_some(baş)
        }
    }
}

fn negatif_aralığı_çöz(aralık: &MatrisAralığı, seviye: usize) -> Option<[usize; 2]> {
    negatif_aralığı_çöz_sınırlı(aralık, seviye, false)
}

fn negatif_aralığı_çöz_sınırlı(
    aralık: &MatrisAralığı,
    seviye: usize,
    sınırla: bool,
) -> Option<[usize; 2]> {
    let çöz = |konum: &MatrisKonumu| {
        let MatrisKonumu::Sıra(sıra) = konum else {
            return None;
        };
        if seviye == 0 || *sıra >= 0 {
            return None;
        }
        let alt = -(seviye as isize);
        let konum = if sınırla {
            (*sıra).clamp(alt, -1)
        } else {
            *sıra
        };
        (konum >= alt).then_some((seviye as isize + konum) as usize)
    };
    match aralık {
        MatrisAralığı::Tümü => (seviye > 0).then_some([0, seviye - 1]),
        MatrisAralığı::Tek(konum) => çöz(konum).map(|sıra| [sıra, sıra]),
        MatrisAralığı::Aralık(baş, son) => {
            let baş = çöz(baş)?;
            let son = çöz(son)?;
            Some([baş.min(son), baş.max(son)])
        }
    }
}

fn seviye_sayısı(boyut: &BoyutYerleşimi) -> usize {
    boyut.seviye_sınırları.len().saturating_sub(1)
}

fn seviye_indeksini_konuma(aralık: [usize; 2], seviye: usize) -> [isize; 2] {
    [
        aralık[0] as isize - seviye as isize,
        aralık[1] as isize - seviye as isize,
    ]
}

fn aralık_kutusu(
    x: &BoyutYerleşimi,
    y: &BoyutYerleşimi,
    xr: [usize; 2],
    yr: [usize; 2],
) -> Option<Dikdörtgen> {
    let x0 = x.sınırlar.get(xr[0]).copied()?;
    let x1 = x.sınırlar.get(xr[1].saturating_add(1)).copied()?;
    let y0 = y.sınırlar.get(yr[0]).copied()?;
    let y1 = y.sınırlar.get(yr[1].saturating_add(1)).copied()?;
    Some(Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0))
}

fn x_başlık_aralık_kutusu(
    x: &BoyutYerleşimi,
    xr: [usize; 2],
    seviye: [usize; 2],
) -> Option<Dikdörtgen> {
    let x0 = x.sınırlar.get(xr[0]).copied()?;
    let x1 = x.sınırlar.get(xr[1].saturating_add(1)).copied()?;
    let y0 = x.seviye_sınırları.get(seviye[0]).copied()?;
    let y1 = x
        .seviye_sınırları
        .get(seviye[1].saturating_add(1))
        .copied()?;
    Some(Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0))
}

fn y_başlık_aralık_kutusu(
    y: &BoyutYerleşimi,
    seviye: [usize; 2],
    yr: [usize; 2],
) -> Option<Dikdörtgen> {
    let x0 = y.seviye_sınırları.get(seviye[0]).copied()?;
    let x1 = y
        .seviye_sınırları
        .get(seviye[1].saturating_add(1))
        .copied()?;
    let y0 = y.sınırlar.get(yr[0]).copied()?;
    let y1 = y.sınırlar.get(yr[1].saturating_add(1)).copied()?;
    Some(Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0))
}

fn köşe_aralık_kutusu(
    x: &BoyutYerleşimi,
    y: &BoyutYerleşimi,
    xr: [usize; 2],
    yr: [usize; 2],
) -> Option<Dikdörtgen> {
    let x0 = y.seviye_sınırları.get(xr[0]).copied()?;
    let x1 = y.seviye_sınırları.get(xr[1].saturating_add(1)).copied()?;
    let y0 = x.seviye_sınırları.get(yr[0]).copied()?;
    let y1 = x.seviye_sınırları.get(yr[1].saturating_add(1)).copied()?;
    Some(Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0))
}

fn x_başlık_kutusu(boyut: &BoyutYerleşimi, düğüm: &BoyutDüğümü) -> Option<Dikdörtgen> {
    let x0 = boyut.sınırlar.get(düğüm.başlangıç).copied()?;
    let x1 = boyut.sınırlar.get(düğüm.bitiş.saturating_add(1)).copied()?;
    let y0 = boyut.seviye_sınırları.get(düğüm.derinlik).copied()?;
    let y1_sırası = if düğüm.yaprak {
        seviye_sayısı(boyut)
    } else {
        düğüm.derinlik.saturating_add(1)
    };
    let y1 = boyut.seviye_sınırları.get(y1_sırası).copied()?;
    Some(Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0))
}

fn y_başlık_kutusu(boyut: &BoyutYerleşimi, düğüm: &BoyutDüğümü) -> Option<Dikdörtgen> {
    let y0 = boyut.sınırlar.get(düğüm.başlangıç).copied()?;
    let y1 = boyut.sınırlar.get(düğüm.bitiş.saturating_add(1)).copied()?;
    let x0 = boyut.seviye_sınırları.get(düğüm.derinlik).copied()?;
    let x1_sırası = if düğüm.yaprak {
        seviye_sayısı(boyut)
    } else {
        düğüm.derinlik.saturating_add(1)
    };
    let x1 = boyut.seviye_sınırları.get(x1_sırası).copied()?;
    Some(Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0))
}

fn noktadan_ana_boyuta(
    gövde_boyutu: &BoyutYerleşimi,
    başlık_boyutu: &BoyutYerleşimi,
    piksel: f32,
) -> Option<isize> {
    if let Some(sıra) = gövde_boyutu
        .sınırlar
        .windows(2)
        .position(|çift| matches!(çift, [a, b] if piksel >= *a && piksel <= *b))
    {
        return Some(sıra as isize);
    }
    başlık_boyutu
        .seviye_sınırları
        .windows(2)
        .position(|çift| matches!(çift, [a, b] if piksel >= *a && piksel <= *b))
        .map(|seviye| seviye as isize - seviye_sayısı(başlık_boyutu) as isize)
}

fn düğüm_seviye_konumu(düğüm: &BoyutDüğümü, seviye: usize) -> [isize; 2] {
    let başlangıç = düğüm.derinlik as isize - seviye as isize;
    let bitiş = if düğüm.yaprak { -1 } else { başlangıç };
    [başlangıç, bitiş]
}

fn aralık_kesişiyor(a: [usize; 2], b: [usize; 2]) -> bool {
    a[0] <= b[1] && b[0] <= a[1]
}

fn aralık_birleştir(a: [usize; 2], b: [usize; 2]) -> [usize; 2] {
    [a[0].min(b[0]), a[1].max(b[1])]
}

fn birleşik_aralığı_genişlet(
    x: &mut [usize; 2],
    y: &mut [usize; 2],
    birleşimler: &[([usize; 2], [usize; 2])],
) {
    // Bir birleşimin genişlemesi başka bir birleşimle yeni kesişim
    // oluşturabilir; sabit noktaya kadar resmi `expandRangeByCellMerge`
    // davranışını yineleriz.
    loop {
        let önceki = (*x, *y);
        for (birleşik_x, birleşik_y) in birleşimler {
            if aralık_kesişiyor(*x, *birleşik_x) && aralık_kesişiyor(*y, *birleşik_y) {
                *x = aralık_birleştir(*x, *birleşik_x);
                *y = aralık_birleştir(*y, *birleşik_y);
            }
        }
        if önceki == (*x, *y) {
            break;
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::panic)]
mod testler {
    use super::*;
    use crate::model::matris::{MatrisBoyutHücresi, MatrisGövdeHücresi};
    use crate::model::stil::ÖğeStili;

    #[test]
    fn flat_matrix_data_point_layout_roundtrip() {
        let seçenek = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni().veri(["A", "B", "C"]))
            .y(MatrisBoyutu::yeni().veri(["X", "Y"]));
        let yer = MatrisYerleşimi::kur(&seçenek, (500.0, 400.0), (0, 0)).unwrap();
        assert_eq!(yer.x_sayısı(), 3);
        assert_eq!(yer.y_sayısı(), 2);
        let kutu = yer
            .veriden_yerleşime(&"B".into(), &"Y".into(), true)
            .unwrap();
        let nokta = kutu.merkez();
        assert_eq!(yer.noktadan_veriye(nokta), [Some(1), Some(1)]);
        assert!(yer.içeriyor_mu(nokta));
    }

    #[test]
    fn hierarchy_size_and_merged_cells() {
        let seçenek = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni().veri([
                MatrisBoyutHücresi::yeni("Grup").çocuklar([
                    MatrisBoyutHücresi::yeni("A").boyut("25%"),
                    MatrisBoyutHücresi::yeni("B"),
                ]),
                MatrisBoyutHücresi::yeni("C"),
            ]))
            .y(MatrisBoyutu::yeni().veri(["X", "Y"]))
            .gövde_hücresi(
                MatrisGövdeHücresi::yeni(MatrisAralığı::Aralık("A".into(), "B".into()), "X")
                    .değer("AB")
                    .birleştir(true),
            );
        let yer = MatrisYerleşimi::kur(&seçenek, (600.0, 400.0), (0, 0)).unwrap();
        let a = yer
            .veriden_yerleşime(&"A".into(), &"X".into(), true)
            .unwrap();
        let ab = yer
            .veriden_yerleşime(
                &MatrisAralığı::Aralık("A".into(), "B".into()),
                &"X".into(),
                false,
            )
            .unwrap();
        assert_eq!(a, ab, "birleşik hücre tek koordinattan genişler");
        assert!(
            yer.hücreler
                .iter()
                .any(|h| h.tür == MatrisHücreTürü::BirleşikGövde)
        );
    }

    #[test]
    fn length_fallback_and_invalid_duplicate() {
        let seçenek = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni().uzunluk(4))
            .y(MatrisBoyutu::yeni());
        let yer = MatrisYerleşimi::kur(&seçenek, (400.0, 300.0), (0, 3)).unwrap();
        assert_eq!((yer.x_sayısı(), yer.y_sayısı()), (4, 3));

        let kötü = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni().veri(["A", "A"]))
            .y(MatrisBoyutu::yeni().veri(["Y"]));
        assert!(MatrisYerleşimi::kur(&kötü, (400.0, 300.0), (0, 0)).is_err());
    }

    #[test]
    fn corner_negative_locator_and_point_roundtrip() {
        let seçenek = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni().veri([MatrisBoyutHücresi::yeni("X")
                .çocuklar([MatrisBoyutHücresi::yeni("A"), MatrisBoyutHücresi::yeni("B")])]))
            .y(MatrisBoyutu::yeni().veri([MatrisBoyutHücresi::yeni("Y")
                .çocuklar([MatrisBoyutHücresi::yeni("U"), MatrisBoyutHücresi::yeni("V")])]))
            .köşe_hücresi(MatrisGövdeHücresi::yeni(-1isize, -1isize).değer("Köşe"));
        let yer = MatrisYerleşimi::kur_sıralı(&seçenek, (500.0, 400.0), (0, 0), 3)
            .expect("matrix kurulmalı");
        let köşe = yer
            .veriden_yerleşime(&(-1isize).into(), &(-1isize).into(), true)
            .expect("negatif köşe çözümlenmeli");
        assert_eq!(yer.noktadan_veriye(köşe.merkez()), [Some(-1), Some(-1)]);
        assert!(yer.hücreler.iter().any(|hücre| {
            hücre.tür == MatrisHücreTürü::Köşe && hücre.değer.as_deref() == Some("Köşe")
        }));
        assert_eq!(yer.bileşen_sırası, 3);
    }

    #[test]
    fn shallow_leaf_spans_remaining_header_levels() {
        let seçenek = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni().veri([
                MatrisBoyutHücresi::yeni("Kısa"),
                MatrisBoyutHücresi::yeni("Grup")
                    .çocuklar([MatrisBoyutHücresi::yeni("Alt")
                        .çocuklar([MatrisBoyutHücresi::yeni("Derin")])]),
            ]))
            .y(MatrisBoyutu::yeni().veri(["Satır"]));
        let yer = MatrisYerleşimi::kur(&seçenek, (600.0, 400.0), (0, 0)).unwrap();
        let kısa = yer
            .hücreler
            .iter()
            .find(|hücre| hücre.değer.as_deref() == Some("Kısa"))
            .unwrap();
        assert_eq!(kısa.y_aralığı, [-3, -1]);
        assert_eq!(kısa.kutu.alt(), yer.gövde_kutusu.y);
    }

    #[test]
    fn yapraklar_ve_karsi_baslik_seviyesi_ayni_fiziksel_boyutu_paylasir() {
        let seçenek = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni().veri(["A", "B", "C"]))
            .y(MatrisBoyutu::yeni().veri(["R1", "R2"]));
        let yer = MatrisYerleşimi::kur(&seçenek, (500.0, 300.0), (0, 0)).unwrap();

        // Dış kutu 400×240'tır. Y başlığının tek seviyesi ile üç X yaprağı
        // yatayda dörder 100 px; X başlığıyla iki Y yaprağı dikeyde üçer
        // 80 px birim olarak resmi `layOutUnitsOnDimension` sırasını izler.
        assert_eq!(yer.dış_kutu, Dikdörtgen::yeni(50.0, 30.0, 400.0, 240.0));
        assert_eq!(
            yer.gövde_kutusu,
            Dikdörtgen::yeni(150.0, 110.0, 300.0, 160.0)
        );
    }

    #[test]
    fn explicit_size_center_and_coord_clamp() {
        let seçenek = MatrisKoordinatı::yeni()
            .genişlik(300)
            .yükseklik(200)
            .yatay_ortala()
            .dikey_ortala()
            .x(MatrisBoyutu::yeni().veri(["A", "B"]))
            .y(MatrisBoyutu::yeni().veri(["Y"]))
            .gövde_hücresi(
                MatrisGövdeHücresi::yeni(99usize, 0usize)
                    .koordinatı_sınırla(true)
                    .değer("B"),
            );
        let yer = MatrisYerleşimi::kur(&seçenek, (600.0, 400.0), (0, 0)).unwrap();
        assert_eq!(yer.dış_kutu, Dikdörtgen::yeni(150.0, 100.0, 300.0, 200.0));
        assert!(yer.hücreler.iter().any(|hücre| {
            hücre.değer.as_deref() == Some("B") && hücre.x_aralığı == [1, 1]
        }));
    }

    #[test]
    fn cursor_mirası_ve_rect_silent_dolgudan_resmi_kuralla_cozulur() {
        let seçenek = MatrisKoordinatı::yeni()
            .x(MatrisBoyutu::yeni()
                .imleç("pointer")
                .öğe_stili(ÖğeStili::yeni().renk(0x112233))
                .veri([
                    MatrisBoyutHücresi::yeni("A"),
                    MatrisBoyutHücresi::yeni("B")
                        .imleç("crosshair")
                        .sessiz(true),
                ]))
            .y(MatrisBoyutu::yeni().veri(["Y"]))
            .gövde_hücresi(
                MatrisGövdeHücresi::yeni("A", "Y")
                    .değer("dolgulu")
                    .öğe_stili(ÖğeStili::yeni().renk(0xaabbcc))
                    .imleç("copy"),
            )
            .gövde_hücresi(MatrisGövdeHücresi::yeni("B", "Y").değer("dolgusuz"));
        let yer = MatrisYerleşimi::kur(&seçenek, (400.0, 300.0), (0, 0)).unwrap();

        let a = yer
            .hücreler
            .iter()
            .find(|hücre| {
                hücre.tür == MatrisHücreTürü::XBaşlığı && hücre.değer.as_deref() == Some("A")
            })
            .unwrap();
        assert_eq!(a.imleç.as_deref(), Some("pointer"));
        assert!(!a.sessiz, "dolgulu rect varsayılan olarak etkileşimli");

        let b = yer
            .hücreler
            .iter()
            .find(|hücre| {
                hücre.tür == MatrisHücreTürü::XBaşlığı && hücre.değer.as_deref() == Some("B")
            })
            .unwrap();
        assert_eq!(b.imleç.as_deref(), Some("crosshair"));
        assert!(b.sessiz, "hücre silent üst modeli geçersiz kılar");

        let dolgulu = yer
            .hücreler
            .iter()
            .find(|hücre| hücre.değer.as_deref() == Some("dolgulu"))
            .unwrap();
        assert_eq!(dolgulu.imleç.as_deref(), Some("copy"));
        assert!(!dolgulu.sessiz);

        let dolgusuz = yer
            .hücreler
            .iter()
            .find(|hücre| hücre.değer.as_deref() == Some("dolgusuz"))
            .unwrap();
        assert!(
            dolgusuz.sessiz,
            "yalnız kenarlığı olan rect, metni olsa da varsayılan silent kalmalı"
        );
    }
}
