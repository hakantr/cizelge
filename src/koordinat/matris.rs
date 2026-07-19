//! ECharts 6.1 Matrix koordinat yerleşimi (`coord/matrix/Matrix.ts`).

use std::collections::{BTreeMap, BTreeSet};

use crate::hata::BilesenHatasi;
use crate::koordinat::Dikdörtgen;
use crate::model::Uzunluk;
use crate::model::matris::{
    MatrisAralığı, MatrisBoyutHücresi, MatrisBoyutu, MatrisKonumu, MatrisKoordinatı,
};
use crate::model::stil::{Etiket, ÖğeStili};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrisHücreTürü {
    XBaşlığı,
    YBaşlığı,
    Gövde,
    BirleşikGövde,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatrisHücreYerleşimi {
    pub tür: MatrisHücreTürü,
    pub değer: Option<String>,
    pub kutu: Dikdörtgen,
    pub x_aralığı: [usize; 2],
    pub y_aralığı: [usize; 2],
    pub öğe_stili: ÖğeStili,
    pub etiket: Etiket,
    pub sessiz: bool,
}

#[derive(Clone, Debug)]
struct BoyutDüğümü {
    değer: String,
    başlangıç: usize,
    bitiş: usize,
    derinlik: usize,
    öğe_stili: Option<ÖğeStili>,
    etiket: Option<Etiket>,
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
}

impl MatrisYerleşimi {
    /// `yedek`, x/y seçeneklerinde data/length yoksa seri verisinden çıkarılan
    /// gövde sütun/satır sayısıdır.
    pub fn kur(
        seçenek: &MatrisKoordinatı,
        tuval: (f32, f32),
        yedek: (usize, usize),
    ) -> Result<Self, BilesenHatasi> {
        let dış_kutu = kutuyu_çöz(seçenek, tuval)?;
        let x_taslak = boyut_taslağı(&seçenek.x, yedek.0, "x")?;
        let y_taslak = boyut_taslağı(&seçenek.y, yedek.1, "y")?;

        let x_seviye_boyutları = seviye_boyutları(&seçenek.x, x_taslak.2, dış_kutu.yükseklik);
        let y_seviye_boyutları = seviye_boyutları(&seçenek.y, y_taslak.2, dış_kutu.genişlik);
        let x_başlık = if seçenek.x.göster {
            x_seviye_boyutları.iter().sum()
        } else {
            0.0
        };
        let y_başlık = if seçenek.y.göster {
            y_seviye_boyutları.iter().sum()
        } else {
            0.0
        };
        let gövde_kutusu = Dikdörtgen::yeni(
            dış_kutu.x + y_başlık,
            dış_kutu.y + x_başlık,
            (dış_kutu.genişlik - y_başlık).max(0.0),
            (dış_kutu.yükseklik - x_başlık).max(0.0),
        );
        let x = boyutu_yerleştir(
            x_taslak,
            gövde_kutusu.x,
            gövde_kutusu.genişlik,
            &x_seviye_boyutları,
            dış_kutu.y,
        );
        let y = boyutu_yerleştir(
            y_taslak,
            gövde_kutusu.y,
            gövde_kutusu.yükseklik,
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
        let mut x_aralığı = aralığı_çöz(&self.x, x)?;
        let mut y_aralığı = aralığı_çöz(&self.y, y)?;
        if birleşimleri_genişlet {
            for (birleşik_x, birleşik_y) in &self.birleşimler {
                if aralık_kesişiyor(x_aralığı, *birleşik_x)
                    && aralık_kesişiyor(y_aralığı, *birleşik_y)
                {
                    x_aralığı = aralık_birleştir(x_aralığı, *birleşik_x);
                    y_aralığı = aralık_birleştir(y_aralığı, *birleşik_y);
                }
            }
        }
        aralık_kutusu(&self.x, &self.y, x_aralığı, y_aralığı)
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
            noktadan_boyuta(&self.x, nokta.0, nokta.1, true, self.gövde_kutusu),
            noktadan_boyuta(&self.y, nokta.1, nokta.0, false, self.gövde_kutusu),
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
            self.hücreler.push(MatrisHücreYerleşimi {
                tür: MatrisHücreTürü::XBaşlığı,
                değer: Some(düğüm.değer.clone()),
                kutu,
                x_aralığı: [düğüm.başlangıç, düğüm.bitiş],
                y_aralığı: [0, 0],
                öğe_stili: düğüm
                    .öğe_stili
                    .clone()
                    .unwrap_or_else(|| seçenek.x.öğe_stili.clone()),
                etiket: düğüm
                    .etiket
                    .clone()
                    .unwrap_or_else(|| seçenek.x.etiket.clone()),
                sessiz: true,
            });
        }
        for düğüm in &self.y.düğümler {
            if !seçenek.y.göster {
                continue;
            }
            let Some(kutu) = y_başlık_kutusu(&self.y, düğüm) else {
                continue;
            };
            self.hücreler.push(MatrisHücreYerleşimi {
                tür: MatrisHücreTürü::YBaşlığı,
                değer: Some(düğüm.değer.clone()),
                kutu,
                x_aralığı: [0, 0],
                y_aralığı: [düğüm.başlangıç, düğüm.bitiş],
                öğe_stili: düğüm
                    .öğe_stili
                    .clone()
                    .unwrap_or_else(|| seçenek.y.öğe_stili.clone()),
                etiket: düğüm
                    .etiket
                    .clone()
                    .unwrap_or_else(|| seçenek.y.etiket.clone()),
                sessiz: true,
            });
        }

        let mut özel = BTreeSet::new();
        for hücre in &seçenek.gövde_verisi {
            let x =
                aralığı_çöz(&self.x, &hücre.x).ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "matrix.body.data.coord.x",
                    ayrıntı: format!("{:?} konumu çözülemedi", hücre.x),
                })?;
            let y =
                aralığı_çöz(&self.y, &hücre.y).ok_or_else(|| BilesenHatasi::GeçersizSeçenek {
                    alan: "matrix.body.data.coord.y",
                    ayrıntı: format!("{:?} konumu çözülemedi", hücre.y),
                })?;
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
            self.hücreler.push(MatrisHücreYerleşimi {
                tür: if hücre.hücreleri_birleştir {
                    MatrisHücreTürü::BirleşikGövde
                } else {
                    MatrisHücreTürü::Gövde
                },
                değer: hücre.değer.clone(),
                kutu,
                x_aralığı: x,
                y_aralığı: y,
                öğe_stili: hücre
                    .öğe_stili
                    .clone()
                    .unwrap_or_else(|| seçenek.gövde_stili.clone()),
                etiket: hücre.etiket.clone().unwrap_or_default(),
                sessiz: hücre.sessiz.unwrap_or(false),
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
                    x_aralığı: [x, x],
                    y_aralığı: [y, y],
                    öğe_stili: seçenek.gövde_stili.clone(),
                    etiket: Etiket::default(),
                    sessiz: true,
                });
            }
        }
        Ok(())
    }
}

type BoyutTaslağı = (Vec<String>, Vec<BoyutDüğümü>, usize, Vec<Option<Uzunluk>>);

fn boyut_taslağı(
    seçenek: &MatrisBoyutu,
    yedek: usize,
    ad: &'static str,
) -> Result<BoyutTaslağı, BilesenHatasi> {
    let veri = if seçenek.veri.is_empty() {
        (0..seçenek.uzunluk.unwrap_or(yedek))
            .map(|sıra| MatrisBoyutHücresi::yeni(sıra.to_string()))
            .collect()
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
        öğe_stili: hücre.öğe_stili.clone(),
        etiket: hücre.etiket.clone(),
    });
    Ok([başlangıç, bitiş])
}

fn boyutu_yerleştir(
    taslak: BoyutTaslağı,
    başlangıç: f32,
    toplam: f32,
    seviye_boyutları: &[f32],
    seviye_başı: f32,
) -> BoyutYerleşimi {
    let (yapraklar, düğümler, _derinlik, boyutlar) = taslak;
    let uzunluklar = yaprak_boyutları(&boyutlar, toplam);
    let mut sınırlar = vec![başlangıç];
    let mut geçerli = başlangıç;
    for uzunluk in uzunluklar {
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

fn yaprak_boyutları(boyutlar: &[Option<Uzunluk>], toplam: f32) -> Vec<f32> {
    if boyutlar.is_empty() {
        return Vec::new();
    }
    let mut sonuç = vec![0.0; boyutlar.len()];
    let mut açık_toplam = 0.0;
    let mut boş = 0usize;
    for (sıra, boyut) in boyutlar.iter().enumerate() {
        if let Some(boyut) = boyut {
            let değer = boyut.çöz(toplam).max(0.0);
            if let Some(yuva) = sonuç.get_mut(sıra) {
                *yuva = değer;
            }
            açık_toplam += değer;
        } else {
            boş = boş.saturating_add(1);
        }
    }
    let kalan = (toplam - açık_toplam).max(0.0);
    let pay = if boş > 0 { kalan / boş as f32 } else { 0.0 };
    for (sıra, boyut) in boyutlar.iter().enumerate() {
        if boyut.is_none()
            && let Some(yuva) = sonuç.get_mut(sıra)
        {
            *yuva = pay;
        }
    }
    let üretilen: f32 = sonuç.iter().sum();
    if üretilen > 0.0 && (üretilen - toplam).abs() > 0.01 {
        let ölçek = toplam / üretilen;
        for boyut in &mut sonuç {
            *boyut *= ölçek;
        }
    }
    sonuç
}

fn seviye_boyutları(seçenek: &MatrisBoyutu, derinlik: usize, bütün: f32) -> Vec<f32> {
    (0..derinlik)
        .map(|sıra| {
            seçenek
                .seviye_boyutları
                .get(sıra)
                .copied()
                .flatten()
                .or(seçenek.seviye_boyutu)
                .unwrap_or(Uzunluk::Piksel(24.0))
                .çöz(bütün)
                .max(0.0)
        })
        .collect()
}

fn kutuyu_çöz(
    seçenek: &MatrisKoordinatı,
    tuval: (f32, f32),
) -> Result<Dikdörtgen, BilesenHatasi> {
    let sol = seçenek.sol.çöz(tuval.0);
    let sağ = seçenek.sağ.çöz(tuval.0);
    let üst = seçenek.üst.çöz(tuval.1);
    let alt = seçenek.alt.çöz(tuval.1);
    let genişlik = tuval.0 - sol - sağ;
    let yükseklik = tuval.1 - üst - alt;
    if ![sol, sağ, üst, alt, genişlik, yükseklik]
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
    Ok(Dikdörtgen::yeni(sol, üst, genişlik, yükseklik))
}

fn aralığı_çöz(boyut: &BoyutYerleşimi, aralık: &MatrisAralığı) -> Option<[usize; 2]> {
    match aralık {
        MatrisAralığı::Tümü => {
            (!boyut.yapraklar.is_empty()).then_some([0, boyut.yapraklar.len() - 1])
        }
        MatrisAralığı::Tek(konum) => konumu_çöz(boyut, konum),
        MatrisAralığı::Aralık(baş, son) => {
            let a = konumu_çöz(boyut, baş)?;
            let b = konumu_çöz(boyut, son)?;
            Some([a[0].min(b[0]), a[1].max(b[1])])
        }
    }
}

fn konumu_çöz(boyut: &BoyutYerleşimi, konum: &MatrisKonumu) -> Option<[usize; 2]> {
    match konum {
        MatrisKonumu::Sıra(sıra) if *sıra >= 0 => {
            let sıra = *sıra as usize;
            (sıra < boyut.yapraklar.len()).then_some([sıra, sıra])
        }
        MatrisKonumu::Değer(değer) => boyut.aralıklar.get(değer).copied(),
        MatrisKonumu::Sıra(_) => None,
    }
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

fn x_başlık_kutusu(boyut: &BoyutYerleşimi, düğüm: &BoyutDüğümü) -> Option<Dikdörtgen> {
    let x0 = boyut.sınırlar.get(düğüm.başlangıç).copied()?;
    let x1 = boyut.sınırlar.get(düğüm.bitiş.saturating_add(1)).copied()?;
    let y0 = boyut.seviye_sınırları.get(düğüm.derinlik).copied()?;
    let y1 = boyut
        .seviye_sınırları
        .get(düğüm.derinlik.saturating_add(1))
        .copied()?;
    Some(Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0))
}

fn y_başlık_kutusu(boyut: &BoyutYerleşimi, düğüm: &BoyutDüğümü) -> Option<Dikdörtgen> {
    let y0 = boyut.sınırlar.get(düğüm.başlangıç).copied()?;
    let y1 = boyut.sınırlar.get(düğüm.bitiş.saturating_add(1)).copied()?;
    let x0 = boyut.seviye_sınırları.get(düğüm.derinlik).copied()?;
    let x1 = boyut
        .seviye_sınırları
        .get(düğüm.derinlik.saturating_add(1))
        .copied()?;
    Some(Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0))
}

fn noktadan_boyuta(
    boyut: &BoyutYerleşimi,
    ana: f32,
    çapraz: f32,
    x_mi: bool,
    gövde: Dikdörtgen,
) -> Option<isize> {
    let gövde_ana = if x_mi {
        ana >= gövde.x && ana <= gövde.sağ()
    } else {
        ana >= gövde.y && ana <= gövde.alt()
    };
    let gövde_çapraz = if x_mi {
        çapraz >= gövde.y && çapraz <= gövde.alt()
    } else {
        çapraz >= gövde.x && çapraz <= gövde.sağ()
    };
    if gövde_ana && gövde_çapraz {
        return boyut
            .sınırlar
            .windows(2)
            .position(|çift| matches!(çift, [a, b] if ana >= *a && ana <= *b))
            .map(|sıra| sıra as isize);
    }
    if gövde_ana {
        return boyut
            .seviye_sınırları
            .windows(2)
            .position(|çift| matches!(çift, [a, b] if çapraz >= *a && çapraz <= *b))
            .map(|seviye| -(seviye as isize) - 1);
    }
    None
}

fn aralık_kesişiyor(a: [usize; 2], b: [usize; 2]) -> bool {
    a[0] <= b[1] && b[0] <= a[1]
}

fn aralık_birleştir(a: [usize; 2], b: [usize; 2]) -> [usize; 2] {
    [a[0].min(b[0]), a[1].max(b[1])]
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::panic)]
mod testler {
    use super::*;
    use crate::model::matris::{MatrisBoyutHücresi, MatrisGövdeHücresi};

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
}
