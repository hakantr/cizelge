//! Grafik görünümü — ECharts örneğinin (`echarts.init` + `setOption`)
//! gpui karşılığı.
//!
//! Boyama hattının tamamı [`grafiği_boya`] içinde, çizim yüzeyinden bağımsız
//! saf bir işlev olarak durur: gpui penceresi de altın (golden) testlerdeki
//! [`crate::cizim::KayıtYüzeyi`] de aynı hattı çalıştırır. gpui'ye özgü
//! yapıştırma (tuval, fare, animasyon karesi, olay yayını) yalnızca
//! [`crate::cizim::pencere::GrafikGörünümü`]dedir.

use std::collections::HashSet;

use crate::bilesen::baslik::başlık_çiz;
use crate::bilesen::eksen_cizimi::{bölme_çizgilerini_çiz, eksenleri_çiz};
use crate::bilesen::gosterge::{gösterge_çiz, GöstergeÖğesi};
use crate::bilesen::zaman_seridi::{zaman_şeridi_çiz, ZamanŞeridiEylemi};
use crate::bilesen::ipucu::{ipucu_çiz, İpucuSatırı};
use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::yuzey::{keskin, ÇizimYüzeyi};
use crate::grafik::cizgi::{nokta_listeleri, çizgi_serisi_çiz};
use crate::grafik::agac::ağaç_çiz;
use crate::grafik::agac_haritasi::{ağaç_haritası_çiz, hücre_değer_metni};
use crate::grafik::gosterge_saati::gösterge_saati_çiz;
use crate::grafik::grafo::grafo_çiz;
use crate::grafik::gunes::güneş_patlaması_çiz;
use crate::grafik::huni::{huni_yerleşimi, huni_çiz};
use crate::grafik::imleyici::{im_alanlarını_çiz, im_çizgi_ve_noktalarını_çiz};
use crate::grafik::isi::{görsel_eşleme_çiz, ısı_değer_kapsamı, ısı_haritası_çiz};
use crate::grafik::kiris::kiriş_çiz;
use crate::grafik::paralel::paralel_çiz;
use crate::grafik::kutupsal::{kutupsal_ağ_çiz, kutupsal_kur, kutupsal_serileri_çiz};
use crate::grafik::mum::{kutu_çiz, mum_çiz};
use crate::grafik::pasta::{dilim_değer_metni, pasta_yerleşimi, pasta_çiz, Dilim};
use crate::grafik::radar::{radar_ağı_çiz, radar_düzeni, radar_ipucu_satırları, radar_serisi_çiz};
use crate::grafik::sacilim::{saçılım_noktaları, saçılım_çiz, SaçılımNoktası};
use crate::grafik::sankey::sankey_çiz;
use crate::grafik::sutun::{sütunları_çiz, SütunGirdisi};
use crate::grafik::takvim_isi::{takvim_değer_kapsamı, takvim_çiz};
use crate::grafik::tema_nehri::tema_nehri_çiz;
use crate::koordinat::{Dikdörtgen, Kartezyen2B, ÇalışmaEkseni};
use crate::model::bilesen::{GöstergeSimgesi, Tetikleme, İmleçTürü, İpucu};
use crate::model::eksen::{Eksen, EksenKonumu, EksenTürü};
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{Seri, ÖzelBağlam};
use crate::model::stil::ÇizgiTürü;
use crate::model::yakinlastirma::YakınlaştırmaTürü;
use crate::olcek::{AralıkÖlçeği, KategorikÖlçek, LogÖlçeği, ZamanÖlçeği, Ölçek};
use crate::renk::Dolgu;
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yerlesim::yigin::{yığın_aralıkları, YığınAralığı};

/// Boyamanın anlık girdileri (görünüm durumundan türetilir).
#[derive(Clone, Debug)]
pub struct BoyamaGirdisi {
    /// Giriş animasyonunun yumuşatılmış ilerlemesi `0..=1`.
    pub ilerleme: f32,
    /// Sürekli animasyonlar için geçen süre (saniye).
    pub zaman_sn: f32,
    /// Yüzey yerel fare konumu.
    pub fare: Option<(f32, f32)>,
    /// Gösterge ile kapatılmış adlar.
    pub kapalı: HashSet<String>,
    /// Kaydırmalı göstergenin geçerli sayfası.
    pub gösterge_sayfası: usize,
    /// Etkin fırça seçimi, yüzey yerel `[x0, y0, x1, y1]`.
    pub fırça: Option<[f32; 4]>,
    /// Zaman şeridi durumu: `(geçerli kare, kare sayısı, oynuyor)`.
    pub zaman_şeridi: Option<(usize, usize, bool)>,
    /// Hiyerarşik gezinme yolu (ağaç haritası inme / güneş patlaması odak):
    /// kökten itibaren ad zinciri.
    pub hiyerarşi_yolu: Vec<String>,
    /// Grafo gezinmesi (roam): `(kayma_x, kayma_y, ölçek)`.
    pub grafo_görünümü: (f32, f32, f32),
    /// Grafo düğümü sürükleme kaymaları: `(düğüm sırası, dx, dy)`.
    pub grafo_kaymaları: Vec<(usize, f32, f32)>,
}

impl Default for BoyamaGirdisi {
    fn default() -> Self {
        BoyamaGirdisi {
            ilerleme: 1.0,
            zaman_sn: 0.0,
            fare: None,
            kapalı: HashSet::new(),
            gösterge_sayfası: 0,
            fırça: None,
            zaman_şeridi: None,
            hiyerarşi_yolu: Vec::new(),
            grafo_görünümü: (0.0, 0.0, 1.0),
            grafo_kaymaları: Vec::new(),
        }
    }
}

/// Sürgünün sürüklenebilir parçaları.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SürgüParçası {
    SolTutamaç,
    SağTutamaç,
    Pencere,
}

/// Çizilen bir yakınlaştırma sürgüsünün etkileşim bölgeleri.
#[derive(Clone, Debug)]
pub struct SürgüBölgesi {
    /// `veri_yakınlaştırmaları` içindeki sıra.
    pub yakınlaştırma_sırası: usize,
    pub şerit: Dikdörtgen,
    pub pencere: Dikdörtgen,
    pub sol_tutamaç: Dikdörtgen,
    pub sağ_tutamaç: Dikdörtgen,
}

/// İç (tekerlek/sürükleme) yakınlaştırmanın etkin olduğu ızgara alanı.
#[derive(Clone, Debug)]
pub struct İçYakınlaştırmaAlanı {
    pub yakınlaştırma_sırası: usize,
    pub alan: Dikdörtgen,
}

/// Boyamanın etkileşim çıktıları: gösterge kutuları ve veri öğesi isabet
/// bölgeleri (yüzey yerel koordinatlarda).
#[derive(Default)]
pub struct BoyamaÇıktısı {
    pub gösterge_kutuları: Vec<(Dikdörtgen, String)>,
    pub isabetler: Vec<İsabetBölgesi>,
    pub sürgüler: Vec<SürgüBölgesi>,
    pub iç_yakınlaştırmalar: Vec<İçYakınlaştırmaAlanı>,
    /// Parçalı görsel eşleme dilimlerinin isabet kutuları.
    pub eşleme_kutuları: Vec<(Dikdörtgen, usize)>,
    /// Kaydırmalı gösterge okları: `(kutu, yön)`.
    pub gösterge_okları: Vec<(Dikdörtgen, i32)>,
    /// Araç kutusu düğmeleri.
    pub araç_düğmeleri: Vec<(Dikdörtgen, AraçTürü)>,
    /// Zaman şeridi düğmeleri (oynat/durdur + kare noktaları).
    pub zaman_düğmeleri: Vec<(Dikdörtgen, ZamanŞeridiEylemi)>,
    /// Hiyerarşi kırıntıları (breadcrumb / geri): `(kutu, yeni yol uzunluğu)`.
    pub kırıntılar: Vec<(Dikdörtgen, usize)>,
}

/// Araç kutusu düğme türleri.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AraçTürü {
    GeriYükle,
    /// Grafiği SVG dosyası olarak kaydet (`saveAsImage`).
    SvgKaydet,
    /// Grafiği PNG dosyası olarak kaydet (`saveAsImage`, `type: 'png'`).
    PngKaydet,
}

/// Ad görünür mü (gösterge ile kapatılmamış mı)?
fn ad_görünür(ad: Option<&str>, kapalı: &HashSet<String>) -> bool {
    ad.map(|a| !kapalı.contains(a)).unwrap_or(true)
}

/// Gösterge öğelerini serilerden derler: kartezyen seriler ad, pasta
/// serileri dilim adlarıyla listelenir (ECharts davranışı).
fn gösterge_öğeleri(
    seçenekler: &GrafikSeçenekleri,
    kapalı: &HashSet<String>,
) -> Vec<GöstergeÖğesi> {
    let süzgeç = seçenekler
        .gösterge
        .as_ref()
        .map(|g| g.veri.clone())
        .unwrap_or_default();
    let mut öğeler = Vec::new();
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        match seri {
            Seri::Radar(r) => {
                for (j, öğe) in r.veri.iter().enumerate() {
                    let Some(ad) = öğe.ad.clone() else { continue };
                    if !süzgeç.is_empty() && !süzgeç.contains(&ad) {
                        continue;
                    }
                    öğeler.push(GöstergeÖğesi {
                        kapalı: kapalı.contains(&ad),
                        ad,
                        renk: seçenekler.palet_rengi(j),
                        simge: GöstergeSimgesi::Çizgi,
                    });
                }
            }
            Seri::Huni(h) => {
                for (j, öğe) in h.veri.iter().enumerate() {
                    let Some(ad) = öğe.ad.clone() else { continue };
                    if !süzgeç.is_empty() && !süzgeç.contains(&ad) {
                        continue;
                    }
                    öğeler.push(GöstergeÖğesi {
                        kapalı: kapalı.contains(&ad),
                        ad,
                        renk: seçenekler.palet_rengi(j),
                        simge: GöstergeSimgesi::YuvarlakKöşeliKare,
                    });
                }
            }
            Seri::Pasta(p) => {
                for (j, öğe) in p.veri.iter().enumerate() {
                    let Some(ad) = öğe.ad.clone() else { continue };
                    if !süzgeç.is_empty() && !süzgeç.contains(&ad) {
                        continue;
                    }
                    let renk = öğe
                        .stil
                        .as_ref()
                        .and_then(|s| s.renk.as_ref())
                        .map(|d| d.temsilî())
                        .unwrap_or_else(|| seçenekler.palet_rengi(j));
                    öğeler.push(GöstergeÖğesi {
                        kapalı: kapalı.contains(&ad),
                        ad,
                        renk,
                        simge: GöstergeSimgesi::Daire,
                    });
                }
            }
            _ => {
                let Some(ad) = seri.ad().map(str::to_string) else { continue };
                if !süzgeç.is_empty() && !süzgeç.contains(&ad) {
                    continue;
                }
                let simge = match seri {
                    Seri::Çizgi(_) => GöstergeSimgesi::Çizgi,
                    Seri::Saçılım(_) => GöstergeSimgesi::Daire,
                    _ => GöstergeSimgesi::YuvarlakKöşeliKare,
                };
                öğeler.push(GöstergeÖğesi {
                    kapalı: kapalı.contains(&ad),
                    ad,
                    renk: seçenekler.seri_rengi(i),
                    simge,
                });
            }
        }
    }
    öğeler
}

/// Eksen seçeneğinden ölçek kurar.
fn ölçek_kur(seçenek: &Eksen, kategoriler: Vec<String>, kapsam: [f64; 2]) -> Ölçek {
    match seçenek.tür {
        EksenTürü::Kategori => Ölçek::Kategorik(KategorikÖlçek::yeni(kategoriler)),
        EksenTürü::Değer => Ölçek::Aralık(AralıkÖlçeği::kur(
            kapsam,
            seçenek.en_az,
            seçenek.en_çok,
            seçenek.sıfırı_içer,
            seçenek.bölme_sayısı,
            seçenek.en_küçük_adım,
            seçenek.en_büyük_adım,
        )),
        EksenTürü::Zaman => {
            let mut kapsam = kapsam;
            if let Some(ea) = seçenek.en_az {
                kapsam[0] = ea;
            }
            if let Some(eç) = seçenek.en_çok {
                kapsam[1] = eç;
            }
            Ölçek::Zaman(ZamanÖlçeği::kur(kapsam, seçenek.bölme_sayısı))
        }
        EksenTürü::Log => Ölçek::Log(LogÖlçeği::kur(
            kapsam,
            seçenek.log_tabanı,
            seçenek.en_az,
            seçenek.en_çok,
            seçenek.bölme_sayısı,
        )),
    }
}

/// Kartezyen kurulumun sonucu: tüm ızgaralar ve eksenler.
struct KartezyenKurulum {
    ızgara_alanları: Vec<Dikdörtgen>,
    x_eksenler: Vec<ÇalışmaEkseni>,
    y_eksenler: Vec<ÇalışmaEkseni>,
    aralıklar: Vec<Vec<YığınAralığı>>,
    görünürler: Vec<bool>,
}

impl KartezyenKurulum {
    /// Serinin bağlı olduğu eksen çiftinden koordinat sistemi kurar.
    fn seri_kartezyeni(&self, seri: &Seri) -> Option<Kartezyen2B> {
        let bağ = seri.eksen_bağı();
        let x = self.x_eksenler.get(bağ.x)?;
        let y = self.y_eksenler.get(bağ.y)?;
        let alan = self
            .ızgara_alanları
            .get(x.seçenek.ızgara_sırası)
            .copied()?;
        Some(Kartezyen2B { x: x.clone(), y: y.clone(), alan })
    }

    /// Izgaranın birincil (ilk) x/y eksen çifti.
    fn birincil_kartezyen(&self, ızgara: usize) -> Option<Kartezyen2B> {
        let x = self
            .x_eksenler
            .iter()
            .find(|e| e.seçenek.ızgara_sırası == ızgara)?;
        let y = self
            .y_eksenler
            .iter()
            .find(|e| e.seçenek.ızgara_sırası == ızgara)?;
        let alan = self.ızgara_alanları.get(ızgara).copied()?;
        Some(Kartezyen2B { x: x.clone(), y: y.clone(), alan })
    }

    /// Farenin üzerinde olduğu ızgara.
    fn faredeki_ızgara(&self, fare: (f32, f32)) -> Option<usize> {
        self.ızgara_alanları
            .iter()
            .position(|alan| alan.içeriyor_mu(fare))
    }
}

/// Kartezyen koordinat sistemlerini kurar: her eksen için kapsam/ölçek,
/// her ızgara için alan.
fn kartezyen_kur(
    yüzey: &dyn ÇizimYüzeyi,
    seçenekler: &GrafikSeçenekleri,
    kapalı: &HashSet<String>,
) -> Option<KartezyenKurulum> {
    let görünürler: Vec<bool> = seçenekler
        .seriler
        .iter()
        .map(|s| ad_görünür(s.ad(), kapalı))
        .collect();

    let kartezyen_var = seçenekler
        .seriler
        .iter()
        .zip(&görünürler)
        .any(|(s, g)| s.kartezyen_mi() && *g);
    let eksen_tanımlı = seçenekler.x_ekseni.is_some()
        || seçenekler.y_ekseni.is_some()
        || !seçenekler.x_eksenleri.is_empty()
        || !seçenekler.y_eksenleri.is_empty();
    if !kartezyen_var && !eksen_tanımlı {
        return None;
    }

    let x_seçenekler = seçenekler.etkin_x_eksenleri();
    let y_seçenekler = seçenekler.etkin_y_eksenleri();
    let ızgara_seçenekleri = seçenekler.etkin_ızgaralar();
    let ızgara_sayısı = ızgara_seçenekleri.len();

    let aralıklar = yığın_aralıkları(&seçenekler.seriler, &görünürler);

    let kapsa = |kapsam: &mut [f64; 2], v: f64| {
        if v.is_finite() {
            kapsam[0] = kapsam[0].min(v);
            kapsam[1] = kapsam[1].max(v);
        }
    };

    // Her eksenin sayısal kapsamı: serinin değerleri kategorik olmayan
    // eksenine, sıra/çift-x değerleri diğerine akar.
    let mut x_kapsamlar = vec![[f64::INFINITY, f64::NEG_INFINITY]; x_seçenekler.len()];
    let mut y_kapsamlar = vec![[f64::INFINITY, f64::NEG_INFINITY]; y_seçenekler.len()];

    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        if !seri.kartezyen_mi() || !görünürler.get(i).copied().unwrap_or(false) {
            continue;
        }
        let bağ = seri.eksen_bağı();
        let (Some(x_seçenek), Some(y_seçenek)) =
            (x_seçenekler.get(bağ.x), y_seçenekler.get(bağ.y))
        else {
            continue;
        };
        let x_kategorik = x_seçenek.tür == EksenTürü::Kategori;
        let y_kategorik = y_seçenek.tür == EksenTürü::Kategori;
        let (Some(x_kapsam), Some(y_kapsam)) =
            (x_kapsamlar.get_mut(bağ.x), y_kapsamlar.get_mut(bağ.y))
        else {
            continue;
        };

        // Isı haritası: iki eksen de kategorik; sayısal kapsam gerekmez.
        if matches!(seri, Seri::Isı(_)) {
            continue;
        }
        // Çok değerli seriler (mum/kutu): dizinin tüm bileşenleri değer
        // eksenine, sıra bant eksenine.
        if matches!(seri, Seri::Mum(_) | Seri::Kutu(_)) {
            for (j, öğe) in seri.veri().iter().enumerate() {
                if let Some(dizi) = öğe.değer.dizi() {
                    for v in dizi {
                        kapsa(y_kapsam, *v);
                    }
                }
                kapsa(x_kapsam, j as f64);
            }
            continue;
        }

        let sütun_mu = matches!(seri, Seri::Sütun(_));
        let Some(seri_aralıkları) = aralıklar.get(i) else { continue };
        for (j, aralık) in seri_aralıkları.iter().enumerate() {
            let Some((taban, tepe)) = aralık else { continue };
            // Yatay yerleşim (y kategorik, x değer): değerler x'e akar.
            let değer_kapsamı: &mut [f64; 2] = if y_kategorik && !x_kategorik {
                x_kapsam
            } else {
                y_kapsam
            };
            kapsa(değer_kapsamı, *tepe);
            if sütun_mu || taban.abs() > 1e-12 {
                kapsa(değer_kapsamı, *taban);
            }
            if x_kategorik || !y_kategorik {
                let x_değeri = seri
                    .veri()
                    .get(j)
                    .and_then(|ö| ö.değer.x())
                    .unwrap_or(j as f64);
                kapsa(x_kapsam, x_değeri);
            }
        }
    }

    // Kategorik eksen verisi: eksen verisi ya da bağlı serilerden türetilir.
    let kategoriler_derle = |eksen: &Eksen, x_mi: bool, eksen_sırası: usize| -> Vec<String> {
        if !eksen.veri.is_empty() {
            return eksen.veri.clone();
        }
        let mut en_uzun = 0usize;
        let mut adlar: Option<Vec<String>> = None;
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if !seri.kartezyen_mi() || !görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            let bağ = seri.eksen_bağı();
            let bağlı = if x_mi { bağ.x } else { bağ.y };
            if bağlı != eksen_sırası {
                continue;
            }
            let veri = seri.veri();
            if veri.len() > en_uzun {
                en_uzun = veri.len();
                adlar = Some(
                    veri.iter()
                        .enumerate()
                        .map(|(j, ö)| ö.ad.clone().unwrap_or_else(|| format!("{j}")))
                        .collect(),
                );
            }
        }
        adlar.unwrap_or_default()
    };

    // Izgara alanları (etiket kapsama, o ızgaranın ilk y/x eksenine göre).
    let ızgara_alanları: Vec<Dikdörtgen> = ızgara_seçenekleri
        .iter()
        .enumerate()
        .map(|(g, ızgara)| {
            let mut sol = ızgara.sol.çöz(yüzey.genişlik());
            let mut sağ_boşluk = ızgara.sağ.çöz(yüzey.genişlik());
            let üst = ızgara.üst.çöz(yüzey.yükseklik());
            let mut alt_boşluk = ızgara.alt.çöz(yüzey.yükseklik());
            if ızgara.etiketi_kapsa {
                if let Some((yi, y_seçenek)) = y_seçenekler
                    .iter()
                    .enumerate()
                    .find(|(_, e)| e.ızgara_sırası == g)
                {
                    let y_boyut = y_seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                    let kapsam = y_kapsamlar.get(yi).copied().unwrap_or([0.0, 1.0]);
                    let ölçek = ölçek_kur(
                        y_seçenek,
                        if y_seçenek.tür == EksenTürü::Kategori {
                            kategoriler_derle(y_seçenek, false, yi)
                        } else {
                            Vec::new()
                        },
                        kapsam,
                    );
                    let mut en_geniş = 0.0f32;
                    for çentik in ölçek.çentikler() {
                        en_geniş =
                            en_geniş.max(yüzey.yazı_ölç(&ölçek.etiket(çentik.değer), y_boyut).0);
                    }
                    sol += en_geniş + y_seçenek.etiket.boşluk;
                }
                if let Some(x_seçenek) =
                    x_seçenekler.iter().find(|e| e.ızgara_sırası == g)
                {
                    let x_boyut = x_seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                    alt_boşluk +=
                        x_boyut * crate::cizim::yuzey::SATIR_ORANI + x_seçenek.etiket.boşluk;
                }
                sağ_boşluk = sağ_boşluk.max(20.0);
            }
            Dikdörtgen::yeni(
                sol,
                üst,
                (yüzey.genişlik() - sol - sağ_boşluk).max(1.0),
                (yüzey.yükseklik() - üst - alt_boşluk).max(1.0),
            )
        })
        .collect();

    // Çalışma eksenleri: piksel aralıkları kendi ızgarasından; konum, aynı
    // ızgaradaki sırasına göre (x: Alt→Üst, y: Sol→Sağ).
    let mut ızgara_x_sayaç = vec![0usize; ızgara_sayısı];
    let x_eksenler: Vec<ÇalışmaEkseni> = x_seçenekler
        .iter()
        .enumerate()
        .map(|(xi, seçenek)| {
            let g = seçenek.ızgara_sırası.min(ızgara_sayısı.saturating_sub(1));
            let alan = ızgara_alanları.get(g).copied().unwrap_or_default();
            let mut kapsam = x_kapsamlar.get(xi).copied().unwrap_or([0.0, 1.0]);
            // Veri yakınlaştırma: sayısal eksenlerde kapsam pencereye
            // daraltılır; kategorik eksenlerde pencere kurulumdan sonra
            // sıra uzayında uygulanır.
            let pencere = seçenekler
                .x_penceresi(xi)
                .filter(|(b, e)| *b > 0.001 || *e < 0.999);
            let mut seçenek = seçenek.clone();
            if let Some((b, e)) = pencere
                && seçenek.tür != EksenTürü::Kategori {
                    let açıklık = kapsam[1] - kapsam[0];
                    if açıklık.is_finite() && açıklık > 0.0 {
                        let yeni = [
                            kapsam[0] + açıklık * b as f64,
                            kapsam[0] + açıklık * e as f64,
                        ];
                        seçenek.en_az = Some(yeni[0]);
                        seçenek.en_çok = Some(yeni[1]);
                        kapsam = yeni;
                    }
                }
            let ölçek = ölçek_kur(
                &seçenek,
                if seçenek.tür == EksenTürü::Kategori {
                    kategoriler_derle(&seçenek, true, xi)
                } else {
                    Vec::new()
                },
                kapsam,
            );
            let sıra_no = ızgara_x_sayaç.get_mut(g).map(|s| {
                let şimdiki = *s;
                *s += 1;
                şimdiki
            });
            let konum = seçenek.konum.unwrap_or(if sıra_no == Some(0) {
                EksenKonumu::Alt
            } else {
                EksenKonumu::Üst
            });
            let mut eksen =
                ÇalışmaEkseni::yeni(seçenek.clone(), ölçek, [alan.x, alan.sağ()], konum);
            if seçenek.tür == EksenTürü::Kategori
                && let Some((b, e)) = pencere {
                    eksen.pencere_uygula(b, e);
                }
            eksen
        })
        .collect();
    let mut ızgara_y_sayaç = vec![0usize; ızgara_sayısı];
    let y_eksenler: Vec<ÇalışmaEkseni> = y_seçenekler
        .iter()
        .enumerate()
        .map(|(yi, seçenek)| {
            let g = seçenek.ızgara_sırası.min(ızgara_sayısı.saturating_sub(1));
            let alan = ızgara_alanları.get(g).copied().unwrap_or_default();
            let kapsam = y_kapsamlar.get(yi).copied().unwrap_or([0.0, 1.0]);
            let ölçek = ölçek_kur(
                seçenek,
                if seçenek.tür == EksenTürü::Kategori {
                    kategoriler_derle(seçenek, false, yi)
                } else {
                    Vec::new()
                },
                kapsam,
            );
            let sıra_no = ızgara_y_sayaç.get_mut(g).map(|s| {
                let şimdiki = *s;
                *s += 1;
                şimdiki
            });
            let konum = seçenek.konum.unwrap_or(if sıra_no == Some(0) {
                EksenKonumu::Sol
            } else {
                EksenKonumu::Sağ
            });
            // Dikey eksen piksel aralığı alttan yukarı doğrudur.
            ÇalışmaEkseni::yeni(seçenek.clone(), ölçek, [alan.alt(), alan.y], konum)
        })
        .collect();

    // Çentik hizalama (`alignTicks`): aynı ızgaradaki İLK değer y-ekseni
    // referanstır; `çentik_hizala` işaretli diğer değer eksenleri onun
    // bölme sayısına uyacak biçimde yeniden kurulur — bölme çizgileri
    // üst üste düşer.
    let mut y_eksenler = y_eksenler;
    for g in 0..ızgara_sayısı {
        let referans_bölme = y_eksenler
            .iter()
            .find(|e| {
                e.seçenek.ızgara_sırası.min(ızgara_sayısı.saturating_sub(1)) == g
                    && matches!(&e.ölçek, Ölçek::Aralık(_))
            })
            .and_then(|e| match &e.ölçek {
                Ölçek::Aralık(ö) => Some(ö.çentikler().len().saturating_sub(1)),
                _ => None,
            });
        let Some(bölme) = referans_bölme.filter(|b| *b >= 1) else { continue };
        let mut ilk_görüldü = false;
        for (yi, eksen) in y_eksenler.iter_mut().enumerate() {
            let eksen_g =
                eksen.seçenek.ızgara_sırası.min(ızgara_sayısı.saturating_sub(1));
            if eksen_g != g || !matches!(&eksen.ölçek, Ölçek::Aralık(_)) {
                continue;
            }
            if !ilk_görüldü {
                // Referansın kendisi olduğu gibi kalır.
                ilk_görüldü = true;
                continue;
            }
            if !eksen.seçenek.çentik_hizala {
                continue;
            }
            let kapsam = y_kapsamlar.get(yi).copied().unwrap_or([0.0, 1.0]);
            eksen.ölçek = Ölçek::Aralık(AralıkÖlçeği::hizalı_kur(
                kapsam,
                eksen.seçenek.en_az,
                eksen.seçenek.en_çok,
                eksen.seçenek.sıfırı_içer,
                bölme,
            ));
        }
    }

    Some(KartezyenKurulum {
        ızgara_alanları,
        x_eksenler,
        y_eksenler,
        aralıklar,
        görünürler,
    })
}

/// Eksen tetiklemeli ipucunun hazırlanmış içeriği.
struct Eksenİpucu {
    ızgara: usize,
    kategori_sırası: usize,
    /// Bant ekseni x mi (dikey imleç) yoksa y mi (yatay imleç)?
    bant_x: bool,
    başlık: String,
    satırlar: Vec<İpucuSatırı>,
}

/// `tooltip.formatter` uygulaması: `{a}` seri adı, `{b}` öğe/kategori adı,
/// `{c}` değer. Eksen tetiklemesinde satır adı seri, başlık kategoridir;
/// öğe tetiklemesinde başlık seri, satır adı öğedir.
fn ipucu_satırlarını_biçimle(
    ipucu: &İpucu,
    başlık: Option<&str>,
    satırlar: Vec<İpucuSatırı>,
) -> Vec<İpucuSatırı> {
    let Some(biçimleyici) = &ipucu.biçimleyici else {
        return satırlar;
    };
    satırlar
        .into_iter()
        .map(|satır| {
            let (seri_adı, öğe_adı) = if ipucu.tetikleme == Tetikleme::Eksen {
                (satır.ad.as_str(), başlık.unwrap_or(""))
            } else {
                (başlık.unwrap_or(""), satır.ad.as_str())
            };
            let metin = match biçimleyici {
                crate::model::stil::Biçimleyici::Şablon(ş) => ş
                    .replace("{a}", seri_adı)
                    .replace("{b}", öğe_adı)
                    .replace("{c}", &satır.değer),
                crate::model::stil::Biçimleyici::İşlev(f) => {
                    f(f64::NAN, &satır.değer)
                }
            };
            İpucuSatırı { im_rengi: satır.im_rengi, ad: metin, değer: String::new() }
        })
        .collect()
}

fn eksen_ipucu_derle(
    seçenekler: &GrafikSeçenekleri,
    kurulum: &KartezyenKurulum,
    fare: (f32, f32),
    ipucu: &İpucu,
) -> Option<Eksenİpucu> {
    let ızgara = kurulum.faredeki_ızgara(fare)?;
    // Bant ekseni: o ızgaradaki ilk kategorik x (öncelik) ya da y ekseni.
    let (bant_ekseni, bant_x, eksen_sırası) = kurulum
        .x_eksenler
        .iter()
        .enumerate()
        .find(|(_, e)| e.seçenek.ızgara_sırası == ızgara && e.ölçek.kategorik_mi())
        .map(|(i, e)| (e, true, i))
        .or_else(|| {
            kurulum
                .y_eksenler
                .iter()
                .enumerate()
                .find(|(_, e)| e.seçenek.ızgara_sırası == ızgara && e.ölçek.kategorik_mi())
                .map(|(i, e)| (e, false, i))
        })?;
    let fare_konumu = if bant_x { fare.0 } else { fare.1 };
    let sıra = bant_ekseni.pikselden_veriye(fare_konumu) as usize;
    let başlık = bant_ekseni.ölçek.etiket(sıra as f64);

    let mut satırlar = Vec::new();
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        if !seri.kartezyen_mi() || !kurulum.görünürler.get(i).copied().unwrap_or(false) {
            continue;
        }
        // Yalnız bu bant eksenine bağlı seriler.
        let bağ = seri.eksen_bağı();
        if (bant_x && bağ.x != eksen_sırası) || (!bant_x && bağ.y != eksen_sırası) {
            continue;
        }
        let Some(öğe) = seri.veri().get(sıra) else { continue };
        let metin = if let Some(dizi) = öğe.değer.dizi() {
            // Mum: A/K/D/Y — Kutu: beş sayının özeti.
            dizi.iter()
                .map(|v| binlik_ayır(*v))
                .collect::<Vec<_>>()
                .join(" / ")
        } else {
            let Some(değer) = öğe.değer.sayı() else { continue };
            match &ipucu.değer_biçimleyici {
                Some(b) => b.uygula(değer, &binlik_ayır(değer)),
                None => binlik_ayır(değer),
            }
        };
        satırlar.push(İpucuSatırı {
            im_rengi: Some(seçenekler.seri_rengi(i)),
            ad: seri.ad().unwrap_or("-").to_string(),
            değer: metin,
        });
    }
    if satırlar.is_empty() {
        return None;
    }
    Some(Eksenİpucu { ızgara, kategori_sırası: sıra, bant_x, başlık, satırlar })
}

/// Tüm grafiği verilen yüzeye boyar; etkileşim bölgelerini döndürür.
///
/// `ilerleme` giriş animasyonunun yumuşatılmış oranı, `fare` yüzey yerel
/// fare konumu, `kapalı` gösterge ile kapatılmış adlardır.
pub fn grafiği_boya(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenekler: &GrafikSeçenekleri,
    girdi: &BoyamaGirdisi,
) -> BoyamaÇıktısı {
    let mut çıktı = BoyamaÇıktısı::default();
    // Veri kümesi eşlemeleri: seriler tablodan türetilir.
    // Etkin tema kipi ve yerel: tüm `tema::*` / `yerel::*` erişimcileri bu
    // seçime göre çözülür.
    crate::tema::koyu_ayarla(seçenekler.koyu);
    crate::yerel::yerel_ayarla(seçenekler.yerel);

    let türetilmiş;
    let seçenekler = if seçenekler.veri_kümesi.is_some() {
        let (yeni, _hatalar) = seçenekler.veri_kümesini_uygula();
        türetilmiş = yeni;
        &türetilmiş
    } else {
        seçenekler
    };
    let ilerleme = girdi.ilerleme;
    let zaman_sn = girdi.zaman_sn;
    let fare = girdi.fare;
    let kapalı = &girdi.kapalı;

    // 1) Arka plan (koyu temada zemin, açıkça verilmemişse de doldurulur).
    let zemin = seçenekler
        .arkaplan
        .or_else(|| seçenekler.koyu.then(crate::tema::zemin));
    if let Some(renk) = zemin {
        let tümü = Dikdörtgen::yeni(0.0, 0.0, yüzey.genişlik(), yüzey.yükseklik());
        yüzey.dikdörtgen(tümü, &Dolgu::Düz(renk), [0.0; 4], None);
    }

    // 2) Başlık.
    if let Some(başlık) = &seçenekler.başlık {
        başlık_çiz(yüzey, başlık);
    }

    // 3) Gösterge.
    let öğeler = gösterge_öğeleri(seçenekler, kapalı);
    if let Some(g) = &seçenekler.gösterge {
        let gösterge_çıktısı = gösterge_çiz(yüzey, g, &öğeler, girdi.gösterge_sayfası);
        çıktı.gösterge_kutuları = gösterge_çıktısı.kutular;
        çıktı.gösterge_okları = gösterge_çıktısı.oklar;
    }

    // 3b) Araç kutusu (sağ üst): sağdan sola dizilen düğmeler.
    if let Some(araçlar) = &seçenekler.araç_kutusu
        && araçlar.göster {
            let boyut = tema::YAZI_KÜÇÜK;
            let mut sağ_kenar = yüzey.genişlik() - 10.0;
            let düğme_çiz = |yüzey: &mut dyn ÇizimYüzeyi,
                                 metin: &str,
                                 tür: AraçTürü,
                                 sağ_kenar: &mut f32| {
                let (gş, _) = yüzey.yazı_ölç(metin, boyut);
                let kutu = Dikdörtgen::yeni(*sağ_kenar - gş - 16.0, 6.0, gş + 16.0, 22.0);
                *sağ_kenar = kutu.x - 6.0;
                yüzey.dikdörtgen(
                    kutu,
                    &Dolgu::Düz(tema::nötr_05()),
                    [4.0; 4],
                    Some((1.0, tema::nötr_15())),
                );
                yüzey.yazı(
                    metin,
                    kutu.merkez(),
                    crate::cizim::YatayHiza::Orta,
                    crate::cizim::DikeyHiza::Orta,
                    boyut,
                    tema::ikincil_metin(),
                    false,
                );
                (kutu, tür)
            };
            if araçlar.geri_yükle {
                let düğme = düğme_çiz(yüzey, "↺ Sıfırla", AraçTürü::GeriYükle, &mut sağ_kenar);
                çıktı.araç_düğmeleri.push(düğme);
            }
            if araçlar.svg_kaydet {
                let düğme = düğme_çiz(yüzey, "⤓ SVG", AraçTürü::SvgKaydet, &mut sağ_kenar);
                çıktı.araç_düğmeleri.push(düğme);
            }
            if araçlar.png_kaydet {
                let düğme = düğme_çiz(yüzey, "⤓ PNG", AraçTürü::PngKaydet, &mut sağ_kenar);
                çıktı.araç_düğmeleri.push(düğme);
            }
        }

    let ipucu_seçeneği = seçenekler.ipucu.clone().filter(|i| i.göster);

    // 4) Kartezyen bölüm (çoklu ızgara/eksen).
    let kurulum = kartezyen_kur(yüzey, seçenekler, kapalı);
    /// `(başlık, satırlar, konum)`.
    type Bekleyenİpucu = (Option<String>, Vec<İpucuSatırı>, (f32, f32));
    let mut bekleyen_ipucu: Option<Bekleyenİpucu> = None;

    if let Some(kurulum) = &kurulum {
        // Eksen imleci içeriği (gölge serilerin altına, çizgi üstüne çizilir).
        let eksen_ipucu = match (&ipucu_seçeneği, fare) {
            (Some(ipucu), Some(f)) if ipucu.tetikleme == Tetikleme::Eksen => {
                eksen_ipucu_derle(seçenekler, kurulum, f, ipucu)
            }
            _ => None,
        };

        // Izgara başına: bölme çizgileri + imleç gölgesi + eksenler.
        for (g, alan) in kurulum.ızgara_alanları.iter().enumerate() {
            let ızgara_eksenleri: Vec<&ÇalışmaEkseni> = kurulum
                .x_eksenler
                .iter()
                .chain(kurulum.y_eksenler.iter())
                .filter(|e| e.seçenek.ızgara_sırası == g)
                .collect();
            if ızgara_eksenleri.is_empty() {
                continue;
            }
            bölme_çizgilerini_çiz(yüzey, *alan, &ızgara_eksenleri);

            if let (Some(ipucu), Some(eksen_ip)) = (&ipucu_seçeneği, &eksen_ipucu)
                && ipucu.imleç == İmleçTürü::Gölge && eksen_ip.ızgara == g {
                    let bant_ekseni = if eksen_ip.bant_x {
                        kurulum
                            .x_eksenler
                            .iter()
                            .find(|e| e.seçenek.ızgara_sırası == g && e.ölçek.kategorik_mi())
                    } else {
                        kurulum
                            .y_eksenler
                            .iter()
                            .find(|e| e.seçenek.ızgara_sırası == g && e.ölçek.kategorik_mi())
                    };
                    if let Some(bant_ekseni) = bant_ekseni {
                        let merkez =
                            bant_ekseni.veriden_piksele(eksen_ip.kategori_sırası as f64);
                        let bant = bant_ekseni.bant_genişliği();
                        let d = if eksen_ip.bant_x {
                            Dikdörtgen::yeni(merkez - bant / 2.0, alan.y, bant, alan.yükseklik)
                        } else {
                            Dikdörtgen::yeni(alan.x, merkez - bant / 2.0, alan.genişlik, bant)
                        };
                        yüzey.dikdörtgen(d, &Dolgu::Düz(tema::imleç_gölgesi()), [0.0; 4], None);
                    }
                }

            eksenleri_çiz(yüzey, *alan, &ızgara_eksenleri);
        }

        // İm alanları serilerin altına boyanır.
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else { continue };
            if let Some(imleyiciler) = seri.imleyiciler()
                && imleyiciler.alan.is_some() {
                    im_alanlarını_çiz(
                        yüzey,
                        imleyiciler,
                        seri,
                        &kartezyen,
                        seçenekler.seri_rengi(i),
                    );
                }
        }

        // Sütunlar eksen çifti başına gruplanır (yerleşim paylaşımı).
        let mut sütun_grupları: Vec<((usize, usize), Vec<SütunGirdisi>)> = Vec::new();
        for (i, s) in seçenekler.seriler.iter().enumerate() {
            if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            if let Seri::Sütun(sütun) = s {
                let bağ = s.eksen_bağı();
                let girdi = SütunGirdisi {
                    seri: sütun,
                    genel_sıra: i,
                    aralıklar: kurulum
                        .aralıklar
                        .get(i)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]),
                    renk: seçenekler.seri_rengi(i),
                };
                match sütun_grupları
                    .iter_mut()
                    .find(|(anahtar, _)| *anahtar == (bağ.x, bağ.y))
                {
                    Some((_, grup)) => grup.push(girdi),
                    None => sütun_grupları.push(((bağ.x, bağ.y), vec![girdi])),
                }
            }
        }
        let mut çizilen_sütun_grupları: HashSet<(usize, usize)> = HashSet::new();

        // Saçılım vurgusu (öğe ipucu) için önden isabet araması.
        // `(seri sırası, vurgulu veri sırası, noktalar)`.
        type SaçılımVurgusu = (usize, Option<usize>, Vec<SaçılımNoktası>);
        let mut saçılım_vurguları: Vec<SaçılımVurgusu> = Vec::new();
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if let Seri::Saçılım(s) = seri {
                if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                    continue;
                }
                let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else { continue };
                let noktalar = saçılım_noktaları(s, &kartezyen);
                let vurgu = match (&ipucu_seçeneği, fare) {
                    (Some(ipucu), Some(f)) if ipucu.tetikleme == Tetikleme::Öğe => noktalar
                        .iter()
                        .filter(|n| {
                            let dx = n.konum.0 - f.0;
                            let dy = n.konum.1 - f.1;
                            let yarıçap = (n.boyut / 2.0 + 3.0).max(8.0);
                            dx * dx + dy * dy <= yarıçap * yarıçap
                        })
                        .min_by(|a, b| {
                            let da = (a.konum.0 - f.0).powi(2) + (a.konum.1 - f.1).powi(2);
                            let db = (b.konum.0 - f.0).powi(2) + (b.konum.1 - f.1).powi(2);
                            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|n| n.sıra),
                    _ => None,
                };
                saçılım_vurguları.push((i, vurgu, noktalar));
            }
        }

        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else { continue };
            // Yakınlaştırma penceresi etkinse seri ızgaraya kırpılır
            // (ECharts `clip: true`).
            let pencereli = kartezyen.x.pencere.is_some() || kartezyen.y.pencere.is_some();
            let mut yerel_isabetler: Vec<İsabetBölgesi> = Vec::new();
            let mut yerel_ipucu: Option<Bekleyenİpucu> = None;
            let mut seri_çiz = |yüzey: &mut dyn ÇizimYüzeyi,
                                isabetler: &mut Vec<İsabetBölgesi>,
                                bekleyen: &mut Option<Bekleyenİpucu>| {
            match seri {
                Seri::Çizgi(s) => {
                    let seri_aralıkları = kurulum
                        .aralıklar
                        .get(i)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]);
                    çizgi_serisi_çiz(
                        yüzey,
                        s,
                        &kartezyen,
                        seri_aralıkları,
                        seçenekler.seri_rengi(i),
                        ilerleme,
                    );
                    // Sembol noktaları tıklanabilir bölgelerdir.
                    let (tepeler, _) = nokta_listeleri(s, &kartezyen, seri_aralıkları);
                    for (j, nokta) in tepeler.iter().enumerate() {
                        let Some(nokta) = nokta else { continue };
                        let Some(öğe) = s.veri.get(j) else { continue };
                        isabetler.push(İsabetBölgesi {
                            seri_sırası: i,
                            veri_sırası: j,
                            seri_adı: s.ad.clone(),
                            ad: öğe.ad.clone(),
                            değer: öğe.değer.sayı(),
                            geometri: İsabetGeometrisi::Daire {
                                merkez: *nokta,
                                yarıçap: (s.sembol_boyutu / 2.0 + 3.0).max(8.0),
                            },
                        });
                    }
                }
                Seri::Sütun(_) => {
                    let bağ = seri.eksen_bağı();
                    if çizilen_sütun_grupları.insert((bağ.x, bağ.y))
                        && let Some((_, girdiler)) = sütun_grupları
                            .iter()
                            .find(|(anahtar, _)| *anahtar == (bağ.x, bağ.y))
                        {
                            sütunları_çiz(
                                yüzey,
                                girdiler,
                                &kartezyen,
                                ilerleme,
                                isabetler,
                            );
                        }
                }
                Seri::Saçılım(s) => {
                    let kayıt = saçılım_vurguları.iter().find(|(sıra, ..)| *sıra == i);
                    if let Some((_, vurgu, noktalar)) = kayıt {
                        saçılım_çiz(
                            yüzey,
                            s,
                            noktalar,
                            seçenekler.seri_rengi(i),
                            ilerleme,
                            zaman_sn,
                            *vurgu,
                        );
                        for n in noktalar {
                            isabetler.push(İsabetBölgesi {
                                seri_sırası: i,
                                veri_sırası: n.sıra,
                                seri_adı: s.ad.clone(),
                                ad: s.veri.get(n.sıra).and_then(|ö| ö.ad.clone()),
                                değer: Some(n.y_değeri),
                                geometri: İsabetGeometrisi::Daire {
                                    merkez: n.konum,
                                    yarıçap: (n.boyut / 2.0 + 3.0).max(8.0),
                                },
                            });
                        }
                        // Öğe ipucu.
                        if let (Some(sıra), Some(f)) = (vurgu, fare)
                            && let Some(nokta) = noktalar.iter().find(|n| n.sıra == *sıra) {
                                *bekleyen = Some((
                                    seri.ad().map(str::to_string),
                                    vec![İpucuSatırı {
                                        im_rengi: Some(seçenekler.seri_rengi(i)),
                                        ad: format!(
                                            "({}, {})",
                                            binlik_ayır(nokta.x_değeri),
                                            binlik_ayır(nokta.y_değeri)
                                        ),
                                        değer: String::new(),
                                    }],
                                    f,
                                ));
                            }
                    }
                }
                Seri::Mum(s) => mum_çiz(
                    yüzey,
                    s,
                    i,
                    &kartezyen,
                    ilerleme,
                    isabetler,
                ),
                Seri::Kutu(s) => kutu_çiz(
                    yüzey,
                    s,
                    i,
                    &kartezyen,
                    seçenekler.seri_rengi(i),
                    isabetler,
                ),
                Seri::Isı(s) => {
                    let eşleme = seçenekler
                        .görsel_eşleme
                        .clone()
                        .unwrap_or_default();
                    let kapsam = eşleme.kapsam_çöz(ısı_değer_kapsamı(s));
                    ısı_haritası_çiz(
                        yüzey,
                        s,
                        i,
                        &kartezyen,
                        &eşleme,
                        kapsam,
                        ilerleme,
                        isabetler,
                    );
                }
                Seri::Özel(s) => {
                    if let Some(çizim) = &s.çizim {
                        let bağlam = ÖzelBağlam {
                            alan: kartezyen.alan,
                            kartezyen: Some(&kartezyen),
                            veri: &s.veri,
                            renk: seçenekler.seri_rengi(i),
                            ilerleme,
                        };
                        çizim(yüzey, &bağlam);
                    }
                }
                Seri::Pasta(_)
                | Seri::Huni(_)
                | Seri::GöstergeSaati(_)
                | Seri::Radar(_)
                | Seri::AğaçHaritası(_)
                | Seri::GüneşPatlaması(_)
                | Seri::Ağaç(_)
                | Seri::Sankey(_)
                | Seri::Grafo(_)
                | Seri::Kiriş(_)
                | Seri::Paralel(_)
                | Seri::Takvim(_)
                | Seri::TemaNehri(_) => {}
            }
            };
            if pencereli {
                let alan_kırp = kartezyen.alan;
                yüzey.kırpılı(alan_kırp, &mut |y| {
                    seri_çiz(y, &mut yerel_isabetler, &mut yerel_ipucu);
                });
            } else {
                seri_çiz(yüzey, &mut yerel_isabetler, &mut yerel_ipucu);
            }
            çıktı.isabetler.append(&mut yerel_isabetler);
            if yerel_ipucu.is_some() {
                bekleyen_ipucu = yerel_ipucu;
            }
        }

        // İm çizgileri ve raptiyeler serilerin üstüne boyanır.
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else { continue };
            if let Some(imleyiciler) = seri.imleyiciler()
                && (imleyiciler.çizgi.is_some() || imleyiciler.nokta.is_some()) {
                    im_çizgi_ve_noktalarını_çiz(
                        yüzey,
                        imleyiciler,
                        seri,
                        &kartezyen,
                        seçenekler.seri_rengi(i),
                    );
                }
        }

        // Çapraz imleç: fareden geçen kesikli yatay+dikey çizgiler ve
        // eksen kenarlarında değer etiketleri (`axisPointer: cross`).
        if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
            && ipucu.imleç == İmleçTürü::Çapraz
                && let Some(g) = kurulum.faredeki_ızgara(f)
                    && let Some(kartezyen) = kurulum.birincil_kartezyen(g) {
                        let alan = kartezyen.alan;
                        let (fx, fy) = (keskin(f.0), keskin(f.1));
                        yüzey.çizgi(
                            (fx, alan.y),
                            (fx, alan.alt()),
                            1.0,
                            tema::imleç_çizgisi(),
                            ÇizgiTürü::Kesikli,
                        );
                        yüzey.çizgi(
                            (alan.x, fy),
                            (alan.sağ(), fy),
                            1.0,
                            tema::imleç_çizgisi(),
                            ÇizgiTürü::Kesikli,
                        );
                        let mut kenar_etiketi =
                            |metin: &str, konum: (f32, f32), yatay_orta: bool| {
                                let boyut = tema::YAZI_KÜÇÜK;
                                let (gş, y) = yüzey.yazı_ölç(metin, boyut);
                                let kutu = if yatay_orta {
                                    Dikdörtgen::yeni(
                                        konum.0 - gş / 2.0 - 5.0,
                                        konum.1,
                                        gş + 10.0,
                                        y + 4.0,
                                    )
                                } else {
                                    Dikdörtgen::yeni(
                                        konum.0 - gş - 10.0,
                                        konum.1 - y / 2.0 - 2.0,
                                        gş + 10.0,
                                        y + 4.0,
                                    )
                                };
                                yüzey.dikdörtgen(kutu, &Dolgu::Düz(tema::nötr_70()), [2.0; 4], None);
                                yüzey.yazı(
                                    metin,
                                    kutu.merkez(),
                                    crate::cizim::YatayHiza::Orta,
                                    crate::cizim::DikeyHiza::Orta,
                                    boyut,
                                    crate::renk::Renk::BEYAZ,
                                    false,
                                );
                            };
                        let x_metin = kartezyen
                            .x
                            .ölçek
                            .etiket(kartezyen.x.pikselden_veriye(f.0));
                        let y_metin = kartezyen
                            .y
                            .ölçek
                            .etiket(kartezyen.y.pikselden_veriye(f.1));
                        kenar_etiketi(&x_metin, (fx, alan.alt() + 4.0), true);
                        kenar_etiketi(&y_metin, (alan.x - 4.0, fy), false);
                    }

        // Veri yakınlaştırma: iç alan kayıtları + sürgü çizimi.
        for (z, yakınlaştırma) in seçenekler.veri_yakınlaştırmaları.iter().enumerate() {
            let Some(x_ekseni) = kurulum.x_eksenler.get(yakınlaştırma.x_eksen_sırası)
            else {
                continue;
            };
            let Some(alan) = kurulum
                .ızgara_alanları
                .get(x_ekseni.seçenek.ızgara_sırası)
                .copied()
            else {
                continue;
            };
            match yakınlaştırma.tür {
                YakınlaştırmaTürü::İç => {
                    çıktı.iç_yakınlaştırmalar.push(İçYakınlaştırmaAlanı {
                        yakınlaştırma_sırası: z,
                        alan,
                    });
                }
                YakınlaştırmaTürü::Sürgü => {
                    let (b, e) = yakınlaştırma.oranlar();
                    let şerit = Dikdörtgen::yeni(
                        alan.x,
                        yüzey.yükseklik() - 36.0,
                        alan.genişlik,
                        22.0,
                    );
                    // Şerit zemini.
                    yüzey.dikdörtgen(
                        şerit,
                        &Dolgu::Düz(tema::nötr_05()),
                        [3.0; 4],
                        Some((1.0, tema::nötr_15())),
                    );
                    // Seçili pencere.
                    let p_x = şerit.x + şerit.genişlik * b;
                    let p_g = (şerit.genişlik * (e - b)).max(4.0);
                    let pencere = Dikdörtgen::yeni(p_x, şerit.y, p_g, şerit.yükseklik);
                    yüzey.dikdörtgen(
                        pencere,
                        &Dolgu::Düz(tema::nötr_40().opaklık(0.35)),
                        [2.0; 4],
                        None,
                    );
                    // Tutamaçlar.
                    let tutamaç = |x: f32| {
                        Dikdörtgen::yeni(x - 4.0, şerit.y - 2.0, 8.0, şerit.yükseklik + 4.0)
                    };
                    let sol = tutamaç(p_x);
                    let sağ = tutamaç(p_x + p_g);
                    for t in [sol, sağ] {
                        yüzey.dikdörtgen(
                            t,
                            &Dolgu::Düz(tema::nötr_50()),
                            [3.0; 4],
                            Some((1.0, tema::nötr_00())),
                        );
                    }
                    çıktı.sürgüler.push(SürgüBölgesi {
                        yakınlaştırma_sırası: z,
                        şerit,
                        pencere,
                        sol_tutamaç: sol,
                        sağ_tutamaç: sağ,
                    });
                }
            }
        }

        // Eksen imleci çizgisi + eksen ipucu penceresi. `bağlantılı`
        // (axisPointer.link) açıkken çizgi, aynı kategori sırasında TÜM
        // ızgaralarda çizilir.
        if let Some(eksen_ip) = eksen_ipucu
            && let Some(ipucu) = &ipucu_seçeneği {
                if ipucu.imleç == İmleçTürü::Çizgi {
                    let hedef_ızgaralar: Vec<usize> = if ipucu.bağlantılı {
                        (0..kurulum.ızgara_alanları.len()).collect()
                    } else {
                        vec![eksen_ip.ızgara]
                    };
                    for ızgara in hedef_ızgaralar {
                        let alan = kurulum
                            .ızgara_alanları
                            .get(ızgara)
                            .copied()
                            .unwrap_or_default();
                        let bant_ekseni = if eksen_ip.bant_x {
                            kurulum.x_eksenler.iter().find(|e| {
                                e.seçenek.ızgara_sırası == ızgara
                                    && e.ölçek.kategorik_mi()
                            })
                        } else {
                            kurulum.y_eksenler.iter().find(|e| {
                                e.seçenek.ızgara_sırası == ızgara
                                    && e.ölçek.kategorik_mi()
                            })
                        };
                        let Some(bant_ekseni) = bant_ekseni else { continue };
                        // Yakınlaştırma penceresi dışındaysa çizme.
                        if !bant_ekseni.pencerede_mi(eksen_ip.kategori_sırası as f64) {
                            continue;
                        }
                        let merkez = keskin(
                            bant_ekseni.veriden_piksele(eksen_ip.kategori_sırası as f64),
                        );
                        if eksen_ip.bant_x {
                            yüzey.çizgi(
                                (merkez, alan.y),
                                (merkez, alan.alt()),
                                1.0,
                                tema::imleç_çizgisi(),
                                ÇizgiTürü::Düz,
                            );
                        } else {
                            yüzey.çizgi(
                                (alan.x, merkez),
                                (alan.sağ(), merkez),
                                1.0,
                                tema::imleç_çizgisi(),
                                ÇizgiTürü::Düz,
                            );
                        }
                    }
                }
                if let Some(f) = fare {
                    bekleyen_ipucu = Some((Some(eksen_ip.başlık), eksen_ip.satırlar, f));
                }
            }
    }

    // 4b) Görsel eşleme bileşeni (gradyan çubuğu).
    if let Some(eşleme) = &seçenekler.görsel_eşleme {
        let veri_kapsamı = seçenekler
            .seriler
            .iter()
            .find_map(|s| match s {
                Seri::Isı(ısı) => Some(ısı_değer_kapsamı(ısı)),
                _ => None,
            })
            .unwrap_or([0.0, 1.0]);
        çıktı.eşleme_kutuları =
            görsel_eşleme_çiz(yüzey, eşleme, eşleme.kapsam_çöz(veri_kapsamı));
    }

    // 4c) Kutupsal koordinat ve kutupsal seriler.
    if let Some(koordinat) = &seçenekler.kutupsal {
        let görünürler: Vec<bool> = seçenekler
            .seriler
            .iter()
            .map(|s| ad_görünür(s.ad(), kapalı))
            .collect();
        let kutupsal_var = seçenekler
            .seriler
            .iter()
            .zip(&görünürler)
            .any(|(s, g)| s.kutupsal_mı() && *g);
        if kutupsal_var {
            let aralıklar = yığın_aralıkları(&seçenekler.seriler, &görünürler);
            let düzen = kutupsal_kur(
                koordinat,
                seçenekler,
                &aralıklar,
                &görünürler,
                Dikdörtgen::yeni(0.0, 0.0, yüzey.genişlik(), yüzey.yükseklik()),
            );
            kutupsal_ağ_çiz(yüzey, &düzen);
            kutupsal_serileri_çiz(
                yüzey,
                seçenekler,
                &düzen,
                &aralıklar,
                &görünürler,
                kapalı,
                ilerleme,
                &mut çıktı.isabetler,
            );
        }
    }

    // 5) Pasta serileri.
    let tüm_alan = Dikdörtgen::yeni(0.0, 0.0, yüzey.genişlik(), yüzey.yükseklik());
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        let Seri::Pasta(p) = seri else { continue };
        if !ad_görünür(seri.ad(), kapalı) {
            continue;
        }
        let dilimler: Vec<Dilim> = pasta_yerleşimi(p, seçenekler, tüm_alan, kapalı, ilerleme);

        // Öğe ipucu: fare hangi dilimde?
        let vurgu = match (&ipucu_seçeneği, fare) {
            (Some(ipucu), Some(f)) if ipucu.tetikleme != Tetikleme::Kapalı => {
                dilimler.iter().position(|d| d.içeriyor_mu(f))
            }
            _ => None,
        };

        pasta_çiz(yüzey, p, &dilimler, vurgu);

        for dilim in &dilimler {
            çıktı.isabetler.push(İsabetBölgesi {
                seri_sırası: i,
                veri_sırası: dilim.sıra,
                seri_adı: p.ad.clone(),
                ad: Some(dilim.ad.clone()),
                değer: Some(dilim.değer),
                geometri: İsabetGeometrisi::Halka {
                    merkez: dilim.merkez,
                    iç_yarıçap: dilim.iç_yarıçap,
                    dış_yarıçap: dilim.dış_yarıçap,
                    açı0: dilim.açı0,
                    açı1: dilim.açı1,
                },
            });
        }

        if let (Some(dilim), Some(f)) = (vurgu.and_then(|sıra| dilimler.get(sıra)), fare) {
            bekleyen_ipucu = Some((
                seri.ad().map(str::to_string),
                vec![İpucuSatırı {
                    im_rengi: Some(dilim.renk),
                    ad: dilim.ad.clone(),
                    değer: dilim_değer_metni(dilim),
                }],
                f,
            ));
        }
    }

    // 5b) Huni, gösterge saati ve radar serileri.
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        match seri {
            Seri::Huni(h) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let dilimler = huni_yerleşimi(h, seçenekler, tüm_alan, kapalı, ilerleme);
                let vurgu = match (&ipucu_seçeneği, fare) {
                    (Some(ipucu), Some(f)) if ipucu.tetikleme != Tetikleme::Kapalı => {
                        dilimler.iter().position(|d| d.sınır_kutusu().içeriyor_mu(f))
                    }
                    _ => None,
                };
                huni_çiz(yüzey, h, i, &dilimler, vurgu, &mut çıktı.isabetler);
                if let (Some(dilim), Some(f)) =
                    (vurgu.and_then(|v| dilimler.get(v)), fare)
                {
                    bekleyen_ipucu = Some((
                        seri.ad().map(str::to_string),
                        vec![İpucuSatırı {
                            im_rengi: Some(dilim.renk),
                            ad: dilim.ad.clone(),
                            değer: binlik_ayır(dilim.değer),
                        }],
                        f,
                    ));
                }
            }
            Seri::GöstergeSaati(g) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                gösterge_saati_çiz(yüzey, g, i, tüm_alan, ilerleme, &mut çıktı.isabetler);
            }
            Seri::Radar(r) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let Some(koordinat) = &seçenekler.radar else { continue };
                if koordinat.göstergeler.len() < 3 {
                    continue;
                }
                let düzen = radar_düzeni(koordinat, tüm_alan);
                radar_ağı_çiz(yüzey, koordinat, &düzen);
                radar_serisi_çiz(
                    yüzey,
                    r,
                    i,
                    koordinat,
                    &düzen,
                    seçenekler,
                    kapalı,
                    ilerleme,
                    &mut çıktı.isabetler,
                );
                // Öğe ipucu: köşe sembolü isabeti.
                if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
                    && ipucu.tetikleme != Tetikleme::Kapalı {
                        let vurgu = çıktı
                            .isabetler
                            .iter()
                            .rev()
                            .find(|b| {
                                b.seri_sırası == i && b.geometri.içeriyor_mu(f)
                            })
                            .map(|b| (b.veri_sırası, b.ad.clone()));
                        if let Some((veri_sırası, ad)) = vurgu {
                            let satırlar: Vec<İpucuSatırı> =
                                radar_ipucu_satırları(r, koordinat, veri_sırası)
                                    .into_iter()
                                    .map(|(gösterge_adı, değer)| İpucuSatırı {
                                        im_rengi: None,
                                        ad: gösterge_adı,
                                        değer,
                                    })
                                    .collect();
                            if !satırlar.is_empty() {
                                bekleyen_ipucu = Some((ad, satırlar, f));
                            }
                        }
                    }
            }
            Seri::Takvim(s) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let eşleme = seçenekler.görsel_eşleme.clone().unwrap_or_default();
                let kapsam = eşleme.kapsam_çöz(takvim_değer_kapsamı(s));
                takvim_çiz(
                    yüzey,
                    s,
                    i,
                    tüm_alan,
                    &eşleme,
                    kapsam,
                    ilerleme,
                    &mut çıktı.isabetler,
                );
                if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
                    && ipucu.tetikleme != Tetikleme::Kapalı
                        && let Some(b) = çıktı
                            .isabetler
                            .iter()
                            .skip(önce)
                            .rev()
                            .find(|b| b.geometri.içeriyor_mu(f))
                        {
                            let satır = İpucuSatırı {
                                im_rengi: None,
                                ad: b.ad.clone().unwrap_or_default(),
                                değer: b.değer.map(binlik_ayır).unwrap_or_default(),
                            };
                            bekleyen_ipucu =
                                Some((seri.ad().map(str::to_string), vec![satır], f));
                        }
            }
            Seri::TemaNehri(s) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                tema_nehri_çiz(yüzey, s, i, tüm_alan, &palet, ilerleme, &mut çıktı.isabetler);
                if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
                    && ipucu.tetikleme != Tetikleme::Kapalı
                        && let Some(b) = çıktı
                            .isabetler
                            .iter()
                            .skip(önce)
                            .rev()
                            .find(|b| b.geometri.içeriyor_mu(f))
                        {
                            let satır = İpucuSatırı {
                                im_rengi: None,
                                ad: b.ad.clone().unwrap_or_default(),
                                değer: b.değer.map(binlik_ayır).unwrap_or_default(),
                            };
                            bekleyen_ipucu =
                                Some((seri.ad().map(str::to_string), vec![satır], f));
                        }
            }
            Seri::Kiriş(s) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                kiriş_çiz(yüzey, s, i, tüm_alan, &palet, ilerleme, &mut çıktı.isabetler);
                if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
                    && ipucu.tetikleme != Tetikleme::Kapalı
                        && let Some(b) = çıktı
                            .isabetler
                            .iter()
                            .skip(önce)
                            .rev()
                            .find(|b| b.geometri.içeriyor_mu(f))
                        {
                            let satır = İpucuSatırı {
                                im_rengi: None,
                                ad: b.ad.clone().unwrap_or_default(),
                                değer: b.değer.map(binlik_ayır).unwrap_or_default(),
                            };
                            bekleyen_ipucu =
                                Some((seri.ad().map(str::to_string), vec![satır], f));
                        }
            }
            Seri::Paralel(s) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                paralel_çiz(
                    yüzey,
                    s,
                    i,
                    tüm_alan,
                    seçenekler.seri_rengi(i),
                    ilerleme,
                    &mut çıktı.isabetler,
                );
            }
            Seri::Grafo(g) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                grafo_çiz(
                    yüzey,
                    g,
                    i,
                    tüm_alan,
                    &palet,
                    ilerleme,
                    girdi.grafo_görünümü,
                    &girdi.grafo_kaymaları,
                    &mut çıktı.isabetler,
                );
                if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
                    && ipucu.tetikleme != Tetikleme::Kapalı
                        && let Some(b) = çıktı
                            .isabetler
                            .iter()
                            .skip(önce)
                            .rev()
                            .find(|b| b.geometri.içeriyor_mu(f))
                        {
                            let satır = İpucuSatırı {
                                im_rengi: None,
                                ad: b.ad.clone().unwrap_or_default(),
                                değer: b.değer.map(binlik_ayır).unwrap_or_default(),
                            };
                            bekleyen_ipucu =
                                Some((seri.ad().map(str::to_string), vec![satır], f));
                        }
            }
            Seri::Sankey(s) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                sankey_çiz(
                    yüzey,
                    s,
                    i,
                    tüm_alan,
                    &palet,
                    ilerleme,
                    &mut çıktı.isabetler,
                );
                if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
                    && ipucu.tetikleme != Tetikleme::Kapalı
                        && let Some(b) = çıktı
                            .isabetler
                            .iter()
                            .skip(önce)
                            .rev()
                            .find(|b| b.geometri.içeriyor_mu(f))
                        {
                            let satır = İpucuSatırı {
                                im_rengi: None,
                                ad: b.ad.clone().unwrap_or_default(),
                                değer: b.değer.map(binlik_ayır).unwrap_or_default(),
                            };
                            bekleyen_ipucu =
                                Some((seri.ad().map(str::to_string), vec![satır], f));
                        }
            }
            Seri::Ağaç(a) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                ağaç_çiz(
                    yüzey,
                    a,
                    i,
                    tüm_alan,
                    seçenekler.seri_rengi(i),
                    ilerleme,
                    &mut çıktı.isabetler,
                );
                if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
                    && ipucu.tetikleme != Tetikleme::Kapalı
                        && let Some(b) = çıktı
                            .isabetler
                            .iter()
                            .skip(önce)
                            .rev()
                            .find(|b| b.geometri.içeriyor_mu(f))
                        {
                            let satır = İpucuSatırı {
                                im_rengi: Some(seçenekler.seri_rengi(i)),
                                ad: b.ad.clone().unwrap_or_default(),
                                değer: b
                                    .değer
                                    .map(binlik_ayır)
                                    .unwrap_or_default(),
                            };
                            bekleyen_ipucu =
                                Some((seri.ad().map(str::to_string), vec![satır], f));
                        }
            }
            Seri::AğaçHaritası(a) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                ağaç_haritası_çiz(
                    yüzey,
                    a,
                    i,
                    tüm_alan,
                    &palet,
                    ilerleme,
                    &girdi.hiyerarşi_yolu,
                    &mut çıktı.isabetler,
                    &mut çıktı.kırıntılar,
                );
                // Öğe ipucu.
                if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
                    && ipucu.tetikleme != Tetikleme::Kapalı
                        && let Some(b) = çıktı
                            .isabetler
                            .iter()
                            .skip(önce)
                            .rev()
                            .find(|b| b.geometri.içeriyor_mu(f))
                        {
                            bekleyen_ipucu = Some((
                                b.ad.clone(),
                                vec![İpucuSatırı {
                                    im_rengi: None,
                                    ad: "Değer".to_string(),
                                    değer: hücre_değer_metni(b.değer.unwrap_or(0.0)),
                                }],
                                f,
                            ));
                        }
            }
            Seri::GüneşPatlaması(g) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                güneş_patlaması_çiz(
                    yüzey,
                    g,
                    i,
                    tüm_alan,
                    &palet,
                    ilerleme,
                    &girdi.hiyerarşi_yolu,
                    &mut çıktı.isabetler,
                    &mut çıktı.kırıntılar,
                );
                if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
                    && ipucu.tetikleme != Tetikleme::Kapalı
                        && let Some(b) = çıktı
                            .isabetler
                            .iter()
                            .skip(önce)
                            .rev()
                            .find(|b| b.geometri.içeriyor_mu(f))
                        {
                            bekleyen_ipucu = Some((
                                b.ad.clone(),
                                vec![İpucuSatırı {
                                    im_rengi: None,
                                    ad: "Değer".to_string(),
                                    değer: hücre_değer_metni(b.değer.unwrap_or(0.0)),
                                }],
                                f,
                            ));
                        }
            }
            Seri::Özel(s) if !s.kartezyen_gerekli => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                if let Some(çizim) = &s.çizim {
                    let bağlam = ÖzelBağlam {
                        alan: tüm_alan,
                        kartezyen: None,
                        veri: &s.veri,
                        renk: seçenekler.seri_rengi(i),
                        ilerleme,
                    };
                    çizim(yüzey, &bağlam);
                }
            }
            _ => {}
        }
    }

    // 5c) Zaman şeridi (timeline) — kare noktaları + oynat/durdur.
    if let Some((geçerli, toplam, oynuyor)) = girdi.zaman_şeridi {
        çıktı.zaman_düğmeleri = zaman_şeridi_çiz(yüzey, geçerli, toplam, oynuyor);
    }

    // 5d) Fırça seçimi kaplaması.
    if let Some([x0, y0, x1, y1]) = girdi.fırça {
        let d = Dikdörtgen::yeni(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
        if d.genişlik > 1.0 && d.yükseklik > 1.0 {
            yüzey.dikdörtgen(
                d,
                &Dolgu::Düz(tema::nötr_40().opaklık(0.15)),
                [0.0; 4],
                None,
            );
            let mut çerçeve = crate::cizim::Yol::yeni();
            çerçeve.taşı((d.x, d.y));
            çerçeve.çiz((d.sağ(), d.y));
            çerçeve.çiz((d.sağ(), d.alt()));
            çerçeve.çiz((d.x, d.alt()));
            çerçeve.kapat();
            yüzey.yol_çiz(&çerçeve, 1.0, tema::nötr_50(), ÇizgiTürü::Kesikli);
        }
    }

    // 6) İpucu penceresi (her şeyin üstüne). `formatter` verilmişse
    // satırlar şablonla yeniden yazılır.
    if let (Some(ipucu), Some((başlık, satırlar, konum))) = (&ipucu_seçeneği, bekleyen_ipucu) {
        let satırlar = ipucu_satırlarını_biçimle(ipucu, başlık.as_deref(), satırlar);
        ipucu_çiz(yüzey, ipucu, konum, başlık.as_deref(), &satırlar);
    }

    çıktı
}


#[cfg(feature = "gpui")]
pub use crate::cizim::pencere::GrafikGörünümü;
