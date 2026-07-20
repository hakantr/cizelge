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
use crate::bilesen::gosterge::{GöstergeÖğesi, gösterge_çiz};
use crate::bilesen::ipucu::{ipucu_çiz, İpucuSatırı};
use crate::bilesen::matris_cizimi::matris_çiz;
use crate::bilesen::takvim_cizimi::{takvim_arka_planı_çiz, takvim_üst_katmanı_çiz};
use crate::bilesen::zaman_seridi::{ZamanŞeridiEylemi, zaman_şeridi_çiz};
use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::yuzey::{keskin, ÇizimYüzeyi};
use crate::grafik::agac::ağaç_çiz;
use crate::grafik::agac_haritasi::{ağaç_haritası_çiz, hücre_değer_metni};
use crate::grafik::cizgi::{nokta_listeleri, ÇizgiKatmanı, çizgi_serisi_çiz};
use crate::grafik::gosterge_saati::gösterge_saati_çiz;
use crate::grafik::grafo::grafo_çiz;
use crate::grafik::gunes::güneş_patlaması_çiz;
use crate::grafik::hatlar::hatlar_çiz;
use crate::grafik::huni::{huni_yerleşimi, huni_çiz};
use crate::grafik::imleyici::{im_alanlarını_çiz, im_çizgi_ve_noktalarını_çiz};
use crate::grafik::isi::{
    SürekliGörselEşlemeBölgesi, görsel_eşleme_çiz, ısı_değer_kapsamı, ısı_haritası_çiz,
};
use crate::grafik::kiris::kiriş_çiz;
use crate::grafik::kutupsal::{kutupsal_ağ_çiz, kutupsal_kur, kutupsal_serileri_çiz};
use crate::grafik::mum::{kutu_çiz, mum_çiz};
use crate::grafik::paralel::paralel_çiz;
use crate::grafik::pasta::{
    Dilim, boş_pasta_çiz, dilim_değer_metni, pasta_yerleşimi, pasta_çiz
};
use crate::grafik::radar::{
    radar_ağı_çiz, radar_düzeni, radar_ipucu_satırları, radar_serisi_çiz
};
use crate::grafik::sacilim::{
    SaçılımNoktası, saçılım_noktaları, saçılım_çiz, takvim_saçılım_noktaları,
};
use crate::grafik::sankey::sankey_çiz;
use crate::grafik::sutun::{SütunGirdisi, sütunları_çiz, yerleşim_hesapla};
use crate::grafik::takvim_isi::{takvim_değer_kapsamı, takvim_koordinatında_çiz, takvim_çiz};
use crate::grafik::tema_nehri::tema_nehri_çiz;
use crate::koordinat::{Dikdörtgen, Kartezyen2B, TakvimYerleşimi, ÇalışmaEkseni};
use crate::model::bilesen::{
    AraçKutusuÖzelliği, GöstergeSimgesi, Tetikleme, Yön, İmleçTürü, İpucu, İpucuParametresi,
};
use crate::model::eksen::{Eksen, EksenKonumu, EksenTürü};
use crate::model::hatlar::{HatKoordinatSistemi, HatKoordinatı, HatNoktası};
use crate::model::matris::{MatrisAralığı, MatrisKonumu};
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{EksenBağı, GrafoSerisi, SaçılımSerisi, Seri, ÖzelBağlam};
use crate::model::stil::ÇizgiTürü;
use crate::model::yakinlastirma::{YakınlaştırmaSüzmeKipi, YakınlaştırmaTürü};
use crate::model::{DikeyKonum, YatayKonum};
use crate::olcek::{
    AralıkÖlçeği, KategorikÖlçek, KırılmaEşleyici, LogÖlçeği, ZamanÖlçeği, Ölçek
};
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yerlesim::yigin::{YığınAralığı, yığın_aralıkları};

/// Toolbox'ın 0..60 civarındaki resmi SVG yol koordinatını 15 px simge
/// kutusuna, en-boy oranını koruyarak ortalar.
fn araç_noktası(merkez: (f32, f32), sınır: [f32; 4], nokta: (f32, f32)) -> (f32, f32) {
    let ölçek = 15.0 / (sınır[2] - sınır[0]).max(sınır[3] - sınır[1]).max(1.0);
    let kaynak_merkez = ((sınır[0] + sınır[2]) / 2.0, (sınır[1] + sınır[3]) / 2.0);
    (
        merkez.0 + (nokta.0 - kaynak_merkez.0) * ölçek,
        merkez.1 + (nokta.1 - kaynak_merkez.1) * ölçek,
    )
}

/// `labelLayout.moveOverlap: 'shiftY'` için zrender'ın tek yönlü dikey
/// itme adımı. Her tuple `(seri, eksen bağı, merkez y, etiket yüksekliği)`dir.
fn çizgi_uç_etiketlerini_dikey_kaydır(
    adaylar: &[(usize, EksenBağı, f32, f32)],
    seri_sayısı: usize,
) -> Vec<Option<f32>> {
    let mut sonuç = vec![None; seri_sayısı];
    let mut işlenen_bağlar = Vec::new();
    for (_, bağ, ..) in adaylar {
        if işlenen_bağlar.contains(bağ) {
            continue;
        }
        işlenen_bağlar.push(*bağ);
        let mut grup = adaylar
            .iter()
            .filter(|(_, aday_bağ, ..)| aday_bağ == bağ)
            .copied()
            .collect::<Vec<_>>();
        grup.sort_by(|a, b| {
            a.2.partial_cmp(&b.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        let mut önceki_alt: Option<f32> = None;
        for (sıra, _, ham_y, yükseklik) in grup {
            let y = önceki_alt
                .map(|alt| ham_y.max(alt + yükseklik / 2.0))
                .unwrap_or(ham_y);
            if let Some(yer) = sonuç.get_mut(sıra) {
                *yer = Some(y);
            }
            önceki_alt = Some(y + yükseklik / 2.0);
        }
    }
    sonuç
}

/// Boyamanın anlık girdileri (görünüm durumundan türetilir).
#[derive(Clone, Debug)]
pub struct BoyamaGirdisi {
    /// Giriş animasyonunun yumuşatılmış ilerlemesi `0..=1`.
    pub ilerleme: f32,
    /// Sürekli animasyonlar için geçen süre (saniye).
    pub zaman_sn: f32,
    /// Yüzey yerel fare konumu.
    pub fare: Option<(f32, f32)>,
    /// `showTip` benzeri programatik öğe ipucunun `(seri, veri)` sırası.
    /// Fare vurgusu oluşturmadan aynı tooltip içeriğini açar.
    pub ipucu_öğesi: Option<(usize, usize)>,
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
            ipucu_öğesi: None,
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
    pub dikey: bool,
}

impl SürgüBölgesi {
    /// Sürgünün veri yüzdesi artan yönündeki işaretçi koordinatı. Dikey
    /// sürgüde ECharts aralığı aşağıdan yukarıya arttığı için ekran Y'sinin
    /// işareti çevrilir.
    pub fn eksen_konumu(&self, nokta: (f32, f32)) -> f32 {
        if self.dikey { -nokta.1 } else { nokta.0 }
    }

    pub fn eksen_uzunluğu(&self) -> f32 {
        if self.dikey {
            self.şerit.yükseklik
        } else {
            self.şerit.genişlik
        }
    }
}

/// İç (tekerlek/sürükleme) yakınlaştırmanın etkin olduğu ızgara alanı.
#[derive(Clone, Debug)]
pub struct İçYakınlaştırmaAlanı {
    pub yakınlaştırma_sırası: usize,
    pub alan: Dikdörtgen,
    pub dikey: bool,
}

impl İçYakınlaştırmaAlanı {
    pub fn eksen_konumu(&self, nokta: (f32, f32)) -> f32 {
        if self.dikey { -nokta.1 } else { nokta.0 }
    }

    pub fn eksen_uzunluğu(&self) -> f32 {
        if self.dikey {
            self.alan.yükseklik
        } else {
            self.alan.genişlik
        }
    }

    /// İşaretçinin veri yüzdesi artan yöndeki, `0..=1` odak oranı.
    pub fn odak_oranı(&self, nokta: (f32, f32)) -> f32 {
        if self.dikey {
            ((self.alan.alt() - nokta.1) / self.alan.yükseklik.max(1.0)).clamp(0.0, 1.0)
        } else {
            ((nokta.0 - self.alan.x) / self.alan.genişlik.max(1.0)).clamp(0.0, 1.0)
        }
    }
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
    /// Sürekli, hesaplanabilir görsel eşlemenin tutamaç/şerit bölgesi.
    pub sürekli_eşleme: Option<SürekliGörselEşlemeBölgesi>,
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
    VeriGörünümü,
    /// `toolbox.feature.dataZoom.zoom`.
    VeriYakınlaştır,
    /// `toolbox.feature.dataZoom.back`.
    VeriYakınlaştırmayıGeriAl,
    SihirliÇizgi,
    SihirliSütun,
    SihirliYığın,
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

type Bekleyenİpucu = (Option<String>, Vec<İpucuSatırı>, (f32, f32));

/// Takvim koordinatındaki scatter/effectScatter serisini tek katmanda
/// boyar. `zlevel > 0` serileri takvim üst katmanından sonra yeniden aynı
/// yordamla çizilebildiği için isabet ve tooltip davranışı katmandan kopmaz.
#[allow(clippy::too_many_arguments)]
fn takvim_saçılım_serisini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &SaçılımSerisi,
    seri_sırası: usize,
    yerleşim: &TakvimYerleşimi,
    seri_rengi: Renk,
    ilerleme: f32,
    zaman_sn: f32,
    ipucu_seçeneği: Option<&İpucu>,
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> Option<Bekleyenİpucu> {
    let noktalar = takvim_saçılım_noktaları(seri, yerleşim);
    let vurgu = match (ipucu_seçeneği, fare) {
        (Some(ipucu), Some(f)) if ipucu.tetikleme != Tetikleme::Kapalı => noktalar
            .iter()
            .filter(|nokta| {
                let dx = nokta.konum.0 - f.0;
                let dy = nokta.konum.1 - f.1;
                let yarıçap = (nokta.boyut / 2.0 + 3.0).max(8.0);
                dx * dx + dy * dy <= yarıçap * yarıçap
            })
            .min_by(|a, b| {
                let da = (a.konum.0 - f.0).powi(2) + (a.konum.1 - f.1).powi(2);
                let db = (b.konum.0 - f.0).powi(2) + (b.konum.1 - f.1).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|nokta| nokta.sıra),
        _ => None,
    };
    saçılım_çiz(
        yüzey, seri, &noktalar, seri_rengi, ilerleme, zaman_sn, vurgu,
    );
    for nokta in &noktalar {
        isabetler.push(İsabetBölgesi {
            seri_sırası,
            veri_sırası: nokta.sıra,
            seri_adı: seri.ad.clone(),
            ad: seri.veri.get(nokta.sıra).and_then(|öğe| öğe.ad.clone()),
            değer: Some(nokta.y_değeri),
            geometri: İsabetGeometrisi::Daire {
                merkez: nokta.konum,
                yarıçap: (nokta.boyut / 2.0 + 3.0).max(8.0),
            },
        });
    }
    let (Some(veri_sırası), Some(f)) = (vurgu, fare) else {
        return None;
    };
    let nokta = noktalar.iter().find(|nokta| nokta.sıra == veri_sırası)?;
    Some((
        seri.ad.clone(),
        vec![İpucuSatırı {
            im_rengi: Some(seri_rengi),
            ad: binlik_ayır(nokta.x_değeri),
            değer: binlik_ayır(nokta.y_değeri),
        }],
        f,
    ))
}

#[allow(clippy::too_many_arguments)]
fn grafo_serisini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &GrafoSerisi,
    seri_sırası: usize,
    tuval: Dikdörtgen,
    seçenekler: &GrafikSeçenekleri,
    ilerleme: f32,
    görünüm: (f32, f32, f32),
    kaymalar: &[(usize, f32, f32)],
    takvim: Option<&TakvimYerleşimi>,
    ipucu_seçeneği: Option<&İpucu>,
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> Option<Bekleyenİpucu> {
    let önce = isabetler.len();
    let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
    grafo_çiz(
        yüzey,
        seri,
        seri_sırası,
        tuval,
        &palet,
        ilerleme,
        görünüm,
        kaymalar,
        takvim,
        isabetler,
    );
    let (Some(ipucu), Some(f)) = (ipucu_seçeneği, fare) else {
        return None;
    };
    if ipucu.tetikleme == Tetikleme::Kapalı {
        return None;
    }
    let b = isabetler
        .iter()
        .skip(önce)
        .rev()
        .find(|b| b.geometri.içeriyor_mu(f))?;
    Some((
        seri.ad.clone(),
        vec![İpucuSatırı {
            im_rengi: None,
            ad: b.ad.clone().unwrap_or_default(),
            değer: b.değer.map(binlik_ayır).unwrap_or_default(),
        }],
        f,
    ))
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
                        çizgi_kalınlığı: None,
                        kenarlık: None,
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
                        çizgi_kalınlığı: None,
                        kenarlık: None,
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
                        simge: GöstergeSimgesi::YuvarlakKöşeliKare,
                        çizgi_kalınlığı: None,
                        kenarlık: p
                            .öğe_stili
                            .kenarlık_rengi
                            .filter(|_| p.öğe_stili.kenarlık_kalınlığı > 0.0)
                            .map(|renk| (p.öğe_stili.kenarlık_kalınlığı, renk)),
                    });
                }
            }
            _ => {
                let Some(ad) = seri.ad().map(str::to_string) else {
                    continue;
                };
                if !süzgeç.is_empty() && !süzgeç.contains(&ad) {
                    continue;
                }
                let simge = match seri {
                    Seri::Çizgi(_) => GöstergeSimgesi::Çizgi,
                    Seri::Saçılım(_) => GöstergeSimgesi::Daire,
                    _ => GöstergeSimgesi::YuvarlakKöşeliKare,
                };
                let çizgi_kalınlığı = match seri {
                    Seri::Çizgi(çizgi) => Some(çizgi.çizgi_stili.kalınlık),
                    _ => None,
                };
                öğeler.push(GöstergeÖğesi {
                    kapalı: kapalı.contains(&ad),
                    ad,
                    renk: seçenekler.seri_rengi(i),
                    simge,
                    çizgi_kalınlığı,
                    kenarlık: None,
                });
            }
        }
    }
    // `legend.data` yalnız bir süzgeç değildir; açıkça verilen sıra resmi
    // gösterge sırasıdır. Seri ekleme sırası farklı olsa da bu düzen korunur.
    if !süzgeç.is_empty() {
        öğeler.sort_by_key(|öğe| {
            süzgeç
                .iter()
                .position(|ad| ad == &öğe.ad)
                .unwrap_or(usize::MAX)
        });
    }
    // Legend veri sağlayıcıları aynı adı birden çok seri üzerinden sunsa da
    // ECharts `LegendModel` tek bir öğe üretir. Özellikle aynı ürünleri
    // gösteren çoklu pasta serilerinde ilk sağlayıcının simge/stili korunur.
    let mut görülen = HashSet::new();
    öğeler.retain(|öğe| görülen.insert(öğe.ad.clone()));
    öğeler
}

/// Etkileşim katmanının `selectedMode: single` uygulayabilmesi için legend
/// veri sağlayıcılarından türetilmiş, kararlı ve yinelenmeyen ad listesi.
#[cfg(feature = "gpui")]
pub(crate) fn gösterge_adları(seçenekler: &GrafikSeçenekleri) -> Vec<String> {
    let mut görülen = HashSet::new();
    gösterge_öğeleri(seçenekler, &HashSet::new())
        .into_iter()
        .map(|öğe| öğe.ad)
        .filter(|ad| görülen.insert(ad.clone()))
        .collect()
}

/// Eksen seçeneğinden ölçek kurar.
fn ölçek_kur(seçenek: &Eksen, kategoriler: Vec<String>, kapsam: [f64; 2]) -> Ölçek {
    let mut kapsam = kapsam;
    if seçenek.tür != EksenTürü::Kategori
        && let Some([alt, üst]) = seçenek.sayısal_kenar_boşluğu
        && kapsam[0].is_finite()
        && kapsam[1].is_finite()
    {
        let fark = (kapsam[1] - kapsam[0]).abs();
        let açıklık = if fark > 0.0 { fark } else { kapsam[0].abs() };
        if seçenek.en_az.is_none() {
            kapsam[0] -= alt.çöz(açıklık);
        }
        if seçenek.en_çok.is_none() {
            kapsam[1] += üst.çöz(açıklık);
        }
    }
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
        let alan = self.ızgara_alanları.get(x.seçenek.ızgara_sırası).copied()?;
        Some(Kartezyen2B {
            x: x.clone(),
            y: y.clone(),
            alan,
        })
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
        Some(Kartezyen2B {
            x: x.clone(),
            y: y.clone(),
            alan,
        })
    }

    /// Farenin üzerinde olduğu ızgara.
    fn faredeki_ızgara(&self, fare: (f32, f32)) -> Option<usize> {
        self.ızgara_alanları
            .iter()
            .position(|alan| alan.içeriyor_mu(fare))
    }
}

fn hat_eksen_değeri(değer: &HatKoordinatı, eksen: &ÇalışmaEkseni) -> Option<f64> {
    if eksen.ölçek.kategorik_mi() {
        değer
            .metin()
            .and_then(|ad| eksen.ölçek.kategori_sırası(ad))
            .or_else(|| değer.sayı())
    } else {
        değer.sayı()
    }
}

fn kartezyen_hat_noktası(nokta: &HatNoktası, kartezyen: &Kartezyen2B) -> Option<(f32, f32)> {
    let x = hat_eksen_değeri(&nokta.x, &kartezyen.x)?;
    let y = hat_eksen_değeri(&nokta.y, &kartezyen.y)?;
    Some(kartezyen.nokta(x, y))
}

fn matris_hat_aralığı(değer: &HatKoordinatı) -> Option<MatrisAralığı> {
    match değer {
        HatKoordinatı::Metin(ad) => Some(MatrisAralığı::Tek(MatrisKonumu::Değer(ad.clone()))),
        HatKoordinatı::Sayı(sıra) if sıra.is_finite() && sıra.fract() == 0.0 => {
            Some(MatrisAralığı::Tek(MatrisKonumu::Sıra(*sıra as isize)))
        }
        HatKoordinatı::Zaman(sıra) => {
            Some(MatrisAralığı::Tek(MatrisKonumu::Sıra(*sıra as isize)))
        }
        HatKoordinatı::Sayı(_) => None,
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
        let (Some(x_seçenek), Some(y_seçenek)) = (x_seçenekler.get(bağ.x), y_seçenekler.get(bağ.y))
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
        if let Seri::Hatlar(hatlar) = seri {
            for nokta in hatlar.veri.iter().flat_map(|veri| &veri.koordinatlar) {
                if !x_kategorik && let Some(değer) = nokta.x.sayı() {
                    kapsa(x_kapsam, değer);
                }
                if !y_kategorik && let Some(değer) = nokta.y.sayı() {
                    kapsa(y_kapsam, değer);
                }
            }
            continue;
        }
        // Scatter `encode.x/y`, veri öğesinin birincil (y) değerinden
        // bağımsız iki dataset boyutudur. Kapsamı ham sıra uzayından değil
        // bu iki boyuttan kurmak, çoklu grid/değer eksenlerini doğru ölçekler.
        if let Seri::Saçılım(saçılım) = seri
            && let Some((x_boyutu, y_boyutu)) = &saçılım.eşleme
        {
            for öğe in &saçılım.veri {
                if !x_kategorik
                    && let Some(değer) = öğe.boyut(x_boyutu).and_then(|değer| değer.sayı())
                {
                    kapsa(x_kapsam, değer);
                }
                if !y_kategorik
                    && let Some(değer) = öğe.boyut(y_boyutu).and_then(|değer| değer.sayı())
                {
                    kapsa(y_kapsam, değer);
                }
            }
            continue;
        }
        // Çok değerli seriler (mum/kutu): dizinin tüm bileşenleri değer
        // eksenine, sıra bant eksenine.
        if matches!(seri, Seri::Mum(_) | Seri::Kutu(_)) {
            for (j, öğe) in seri.veri().iter().enumerate() {
                if let Some(dizi) = öğe.değer.dizi() {
                    if y_kategorik && !x_kategorik {
                        for v in dizi {
                            kapsa(x_kapsam, *v);
                        }
                    } else {
                        for v in dizi {
                            kapsa(y_kapsam, *v);
                        }
                    }
                }
                if y_kategorik && !x_kategorik {
                    kapsa(y_kapsam, j as f64);
                } else {
                    kapsa(x_kapsam, j as f64);
                }
            }
            continue;
        }

        let sütun_mu = matches!(seri, Seri::Sütun(_));
        // Bir XY öğesinin karşı boyutu NaN olsa da sonlu x değeri eksen
        // kapsamına katılır. ECharts bunu özellikle çizgiyi kesen
        // `[timestamp, NaN]` satırlarında korur; son zaman çentiği ve eksen
        // kırılması bu x ucuna kadar uzanır.
        if !x_kategorik {
            for öğe in seri.veri() {
                if let Some(x) = öğe.değer.x() {
                    kapsa(x_kapsam, x);
                }
            }
        }
        let Some(seri_aralıkları) = aralıklar.get(i) else {
            continue;
        };
        for (j, aralık) in seri_aralıkları.iter().enumerate() {
            let Some((taban, tepe)) = aralık else {
                continue;
            };
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
            if let Seri::Hatlar(hatlar) = seri {
                let mut kategoriler = Vec::new();
                for koordinat in hatlar.veri.iter().flat_map(|veri| &veri.koordinatlar) {
                    let değer = if x_mi { &koordinat.x } else { &koordinat.y };
                    let ad = değer
                        .metin()
                        .map(str::to_owned)
                        .or_else(|| değer.sayı().map(|sayı| sayı.to_string()));
                    if let Some(ad) = ad
                        && !kategoriler.contains(&ad)
                    {
                        kategoriler.push(ad);
                    }
                }
                if kategoriler.len() > en_uzun {
                    en_uzun = kategoriler.len();
                    adlar = Some(kategoriler);
                }
                continue;
            }
            if let Seri::Saçılım(saçılım) = seri
                && let Some((x_boyutu, y_boyutu)) = &saçılım.eşleme
            {
                let boyut = if x_mi { x_boyutu } else { y_boyutu };
                let mut kategoriler = Vec::new();
                for öğe in &saçılım.veri {
                    let ad = match öğe.boyut(boyut) {
                        Some(crate::model::deger::VeriDeğeri::Metin(ad)) => Some(ad.clone()),
                        Some(crate::model::deger::VeriDeğeri::Sayı(değer)) => {
                            Some(crate::yardimci::bicim::ondalık_kırp(*değer))
                        }
                        Some(crate::model::deger::VeriDeğeri::Zaman(değer)) => {
                            Some(değer.to_string())
                        }
                        Some(crate::model::deger::VeriDeğeri::Mantıksal(değer)) => {
                            Some(değer.to_string())
                        }
                        _ => None,
                    };
                    if let Some(ad) = ad
                        && !kategoriler.contains(&ad)
                    {
                        kategoriler.push(ad);
                    }
                }
                if kategoriler.len() > en_uzun {
                    en_uzun = kategoriler.len();
                    adlar = Some(kategoriler);
                }
                continue;
            }
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

    // ECharts `AxisProxy` önce ham veri kapsamından dataZoom penceresini
    // çözer, ardından `filter`/`weakFilter` ile kalan satırlardan diğer
    // eksenin kapsamını yeniden kurar. Ham kapsamı ayrı tutmak zorunludur:
    // aksi halde her render'da yüzde penceresi yeniden küçülür.
    let ham_x_kapsamlar = x_kapsamlar.clone();
    let ham_y_kapsamlar = y_kapsamlar.clone();
    type SüzmePenceresi = ([f64; 2], (f32, f32), YakınlaştırmaSüzmeKipi);
    let x_pencereleri: Vec<Option<SüzmePenceresi>> = x_seçenekler
        .iter()
        .enumerate()
        .map(|(sıra, eksen)| {
            let yakınlaştırma = seçenekler.x_yakınlaştırması(sıra)?;
            let kategoriler = if eksen.tür == EksenTürü::Kategori {
                kategoriler_derle(eksen, true, sıra)
            } else {
                Vec::new()
            };
            let ham = if eksen.tür == EksenTürü::Kategori {
                [0.0, kategoriler.len().saturating_sub(1) as f64]
            } else {
                let veri = ham_x_kapsamlar.get(sıra).copied().unwrap_or([0.0, 1.0]);
                [
                    eksen.en_az.unwrap_or(veri[0]),
                    eksen.en_çok.unwrap_or(veri[1]),
                ]
            };
            yakınlaştırma
                .pencere_çöz(&kategoriler, ham)
                .map(|(mut pencere, oranlar)| {
                    if eksen.tür == EksenTürü::Kategori {
                        pencere = [pencere[0].round(), pencere[1].round()];
                    }
                    (pencere, oranlar, yakınlaştırma.süzme_kipi)
                })
        })
        .collect();
    let y_pencereleri: Vec<Option<SüzmePenceresi>> = y_seçenekler
        .iter()
        .enumerate()
        .map(|(sıra, eksen)| {
            let yakınlaştırma = seçenekler.y_yakınlaştırması(sıra)?;
            let kategoriler = if eksen.tür == EksenTürü::Kategori {
                kategoriler_derle(eksen, false, sıra)
            } else {
                Vec::new()
            };
            let ham = if eksen.tür == EksenTürü::Kategori {
                [0.0, kategoriler.len().saturating_sub(1) as f64]
            } else {
                let veri = ham_y_kapsamlar.get(sıra).copied().unwrap_or([0.0, 1.0]);
                [
                    eksen.en_az.unwrap_or(veri[0]),
                    eksen.en_çok.unwrap_or(veri[1]),
                ]
            };
            yakınlaştırma
                .pencere_çöz(&kategoriler, ham)
                .map(|(mut pencere, oranlar)| {
                    if eksen.tür == EksenTürü::Kategori {
                        pencere = [pencere[0].round(), pencere[1].round()];
                    }
                    (pencere, oranlar, yakınlaştırma.süzme_kipi)
                })
        })
        .collect();

    let pencereden_geçer = |pencere: Option<SüzmePenceresi>, değerler: &[f64]| -> bool {
        let Some(([baş, son], _, kip)) = pencere else {
            return true;
        };
        match kip {
            YakınlaştırmaSüzmeKipi::Yok | YakınlaştırmaSüzmeKipi::Boşalt => true,
            YakınlaştırmaSüzmeKipi::Süz => değerler
                .iter()
                .all(|değer| değer.is_finite() && *değer >= baş && *değer <= son),
            YakınlaştırmaSüzmeKipi::ZayıfSüz => {
                let mut değer_var = false;
                let mut solda = false;
                let mut sağda = false;
                for değer in değerler.iter().copied().filter(|değer| değer.is_finite()) {
                    değer_var = true;
                    if değer >= baş && değer <= son {
                        return true;
                    }
                    solda |= değer < baş;
                    sağda |= değer > son;
                }
                // `weakFilter`, bütün boyutlar aynı tarafta kaldığında
                // süzer; pencerenin iki yanına yayılan öğeyi korur.
                değer_var && solda && sağda
            }
        }
    };

    if x_pencereleri.iter().flatten().any(|(_, _, kip)| {
        matches!(
            kip,
            YakınlaştırmaSüzmeKipi::Süz | YakınlaştırmaSüzmeKipi::ZayıfSüz
        )
    }) || y_pencereleri.iter().flatten().any(|(_, _, kip)| {
        matches!(
            kip,
            YakınlaştırmaSüzmeKipi::Süz | YakınlaştırmaSüzmeKipi::ZayıfSüz
        )
    }) {
        x_kapsamlar.fill([f64::INFINITY, f64::NEG_INFINITY]);
        y_kapsamlar.fill([f64::INFINITY, f64::NEG_INFINITY]);

        for (seri_sırası, seri) in seçenekler.seriler.iter().enumerate() {
            if !seri.kartezyen_mi() || !görünürler.get(seri_sırası).copied().unwrap_or(false) {
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
            let x_penceresi = x_pencereleri.get(bağ.x).copied().flatten();
            let y_penceresi = y_pencereleri.get(bağ.y).copied().flatten();

            if matches!(seri, Seri::Isı(_)) {
                continue;
            }
            if let Seri::Hatlar(hatlar) = seri {
                for nokta in hatlar.veri.iter().flat_map(|veri| &veri.koordinatlar) {
                    let x = nokta.x.sayı();
                    let y = nokta.y.sayı();
                    let x_değerleri = [x.unwrap_or(f64::NAN)];
                    let y_değerleri = [y.unwrap_or(f64::NAN)];
                    if !pencereden_geçer(x_penceresi, &x_değerleri)
                        || !pencereden_geçer(y_penceresi, &y_değerleri)
                    {
                        continue;
                    }
                    if !x_kategorik && let Some(x) = x {
                        kapsa(x_kapsam, x);
                    }
                    if !y_kategorik && let Some(y) = y {
                        kapsa(y_kapsam, y);
                    }
                }
                continue;
            }
            if let Seri::Saçılım(saçılım) = seri
                && let Some((x_boyutu, y_boyutu)) = &saçılım.eşleme
            {
                for öğe in &saçılım.veri {
                    let x = öğe.boyut(x_boyutu).and_then(|değer| değer.sayı());
                    let y = öğe.boyut(y_boyutu).and_then(|değer| değer.sayı());
                    let x_değerleri = [x.unwrap_or(f64::NAN)];
                    let y_değerleri = [y.unwrap_or(f64::NAN)];
                    if !pencereden_geçer(x_penceresi, &x_değerleri)
                        || !pencereden_geçer(y_penceresi, &y_değerleri)
                    {
                        continue;
                    }
                    if !x_kategorik && let Some(x) = x {
                        kapsa(x_kapsam, x);
                    }
                    if !y_kategorik && let Some(y) = y {
                        kapsa(y_kapsam, y);
                    }
                }
                continue;
            }
            if matches!(seri, Seri::Mum(_) | Seri::Kutu(_)) {
                for (veri_sırası, öğe) in seri.veri().iter().enumerate() {
                    let Some(dizi) = öğe.değer.dizi() else {
                        continue;
                    };
                    let sıra_değeri = [veri_sırası as f64];
                    let (x_değerleri, y_değerleri): (&[f64], &[f64]) =
                        if y_kategorik && !x_kategorik {
                            (dizi, &sıra_değeri)
                        } else {
                            (&sıra_değeri, dizi)
                        };
                    if !pencereden_geçer(x_penceresi, x_değerleri)
                        || !pencereden_geçer(y_penceresi, y_değerleri)
                    {
                        continue;
                    }
                    if y_kategorik && !x_kategorik {
                        for değer in x_değerleri {
                            kapsa(x_kapsam, *değer);
                        }
                        kapsa(y_kapsam, veri_sırası as f64);
                    } else {
                        kapsa(x_kapsam, veri_sırası as f64);
                        for değer in y_değerleri {
                            kapsa(y_kapsam, *değer);
                        }
                    }
                }
                continue;
            }

            let sütun_mu = matches!(seri, Seri::Sütun(_));
            let Some(seri_aralıkları) = aralıklar.get(seri_sırası) else {
                continue;
            };
            for (veri_sırası, aralık) in seri_aralıkları.iter().enumerate() {
                let Some((taban, tepe)) = aralık else {
                    continue;
                };
                let x_değeri = seri
                    .veri()
                    .get(veri_sırası)
                    .and_then(|öğe| öğe.değer.x())
                    .unwrap_or(veri_sırası as f64);
                let (x_değerleri, y_değerleri) = if y_kategorik && !x_kategorik {
                    ([*tepe], [veri_sırası as f64])
                } else {
                    ([x_değeri], [*tepe])
                };
                if !pencereden_geçer(x_penceresi, &x_değerleri)
                    || !pencereden_geçer(y_penceresi, &y_değerleri)
                {
                    continue;
                }
                let değer_kapsamı = if y_kategorik && !x_kategorik {
                    &mut *x_kapsam
                } else {
                    &mut *y_kapsam
                };
                kapsa(değer_kapsamı, *tepe);
                if sütun_mu || taban.abs() > 1e-12 {
                    kapsa(değer_kapsamı, *taban);
                }
                if x_kategorik || !y_kategorik {
                    kapsa(x_kapsam, x_değeri);
                }
            }
        }
    }

    // Izgara alanları (etiket kapsama, o ızgaranın ilk y/x eksenine göre).
    let mut ızgara_alanları: Vec<Dikdörtgen> = ızgara_seçenekleri
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
                        let ham = ölçek.etiket(çentik.değer);
                        let metin = y_seçenek
                            .etiket
                            .biçimleyici
                            .as_ref()
                            .map(|biçimleyici| biçimleyici.uygula(çentik.değer, &ham))
                            .unwrap_or(ham);
                        en_geniş = en_geniş.max(yüzey.yazı_ölç(&metin, y_boyut).0);
                    }
                    // Sabit Arial dosyasında ab_glyph ile Chromium Canvas
                    // ölçümü arasındaki kalan ortalama fark yaklaşık 0,04
                    // pikseldir. Eski 3/8 px dengesi boşluk çevresi kerning'i
                    // düzeltilmeden önce gerekliydi ve yatay çizgileri 0,34
                    // px genişletiyordu.
                    sol += en_geniş + y_seçenek.etiket.boşluk - 0.04;
                }
                if let Some(x_seçenek) = x_seçenekler.iter().find(|e| e.ızgara_sırası == g) {
                    let x_boyut = x_seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                    // Tek satırlı eksen etiketi için zrender sınır kutusu
                    // font boyudur; genel rich-text satır oranı burada
                    // fazladan dikey boşluk üretmemelidir.
                    alt_boşluk += x_boyut + x_seçenek.etiket.boşluk;
                }
                sağ_boşluk = sağ_boşluk.max(20.0);
            }
            let genişlik = ızgara
                .genişlik
                .map(|uzunluk| uzunluk.çöz(yüzey.genişlik()))
                .unwrap_or_else(|| yüzey.genişlik() - sol - sağ_boşluk)
                .max(1.0);
            let yükseklik = ızgara
                .yükseklik
                .map(|uzunluk| uzunluk.çöz(yüzey.yükseklik()))
                .unwrap_or_else(|| yüzey.yükseklik() - üst - alt_boşluk)
                .max(1.0);
            Dikdörtgen::yeni(sol, üst, genişlik, yükseklik)
        })
        .collect();

    // ECharts 6 `grid.outerBoundsMode: auto`, uçta duran eksen adlarının
    // tuval dışına taşması halinde yalnız taşan kenarı daraltır. Bu ikinci
    // yerleşim turu özellikle sağ-alt çoklu gridlerdeki uzun x ekseni
    // adlarının kırpılmasını önler. Hesap AxisBuilder'ın varsayılan
    // `nameGap: 15` sınır kutusunu kullanır; gerçek boya alt-piksel metin
    // sınırı nedeniyle daha yakın görünen çapa kullanmaya devam eder.
    for (g, alan) in ızgara_alanları.iter_mut().enumerate() {
        let sağ_taşma = x_seçenekler
            .iter()
            .filter(|eksen| eksen.ızgara_sırası == g)
            .filter(|eksen| eksen.ad_konumu == crate::model::eksen::EksenAdKonumu::Bitiş)
            .filter_map(|eksen| eksen.ad.as_deref())
            .map(|ad| {
                let ad_genişliği = yüzey.yazı_ölç(ad, tema::YAZI_KÜÇÜK).0;
                (alan.sağ()
                    + x_seçenekler
                        .iter()
                        .find(|eksen| eksen.ızgara_sırası == g && eksen.ad.as_deref() == Some(ad))
                        .map(|eksen| eksen.ad_boşluğu)
                        .unwrap_or(15.0)
                    // `createOrUpdateAxesView` uzun (tuvalin yarısından
                    // yüksek) gridlerde eksen adı için ikinci margin
                    // seviyesini seçer. Sağ fiziksel piksel payı bu durumda
                    // 3, küçük çoklu gridlerde 1 pikseldir.
                    + if alan.yükseklik > yüzey.yükseklik() * 0.5 {
                        3.0
                    } else {
                        1.0
                    }
                    + ad_genişliği
                    - yüzey.genişlik())
                .max(0.0)
            })
            .fold(0.0_f32, f32::max);
        if sağ_taşma > 0.0 {
            // GridModel.outerBoundsClampWidth öntanımlı `%25`: ilk alanın
            // en az dörtte biri korunur.
            alan.genişlik = (alan.genişlik - sağ_taşma).max(alan.genişlik * 0.25);
        }
    }

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
            let kategoriler = if seçenek.tür == EksenTürü::Kategori {
                kategoriler_derle(seçenek, true, xi)
            } else {
                Vec::new()
            };
            // Veri yakınlaştırma: sayısal eksenlerde kapsam pencereye
            // daraltılır; kategorik eksenlerde pencere kurulumdan sonra
            // sıra uzayında uygulanır.
            let yakınlaştırma = seçenekler.x_yakınlaştırması(xi).filter(|y| y.etkin_mi());
            let pencere = x_pencereleri
                .get(xi)
                .copied()
                .flatten()
                .map(|(pencere, oranlar, _)| (pencere, oranlar));
            let mut seçenek = seçenek.clone();
            if let Some(([p0, p1], _)) = pencere
                && seçenek.tür != EksenTürü::Kategori
            {
                if let Some(yakınlaştırma) = yakınlaştırma {
                    // AxisProxy yalnız gerçekten daraltılan ucu sabitler.
                    // %0/%100 sınırı, ölçeğin o ucu güzel bir çentiğe
                    // genişletmesine izin verir; startValue/endValue ise
                    // yüzdeden bağımsız olarak sabittir.
                    if yakınlaştırma.başlangıç_değeri.is_some() || yakınlaştırma.başlangıç > 0.001
                    {
                        seçenek.en_az = Some(p0);
                    }
                    if yakınlaştırma.bitiş_değeri.is_some() || yakınlaştırma.bitiş < 99.999
                    {
                        seçenek.en_çok = Some(p1);
                    }
                }
                kapsam = [p0, p1];
            }
            let ölçek = ölçek_kur(&seçenek, kategoriler, kapsam);
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
            if let Some(([p0, p1], oranlar)) = pencere {
                eksen.yakınlaştırma_oranları = Some(oranlar);
                if let Some(yakınlaştırma) = yakınlaştırma {
                    eksen.yakınlaştırma_süzme_kipi = yakınlaştırma.süzme_kipi;
                }
                if seçenek.tür == EksenTürü::Kategori {
                    eksen.değer_penceresi_uygula(p0.round(), p1.round());
                } else {
                    let ölçek_kapsamı = eksen.ölçek.kapsam();
                    eksen.değer_penceresi_uygula(ölçek_kapsamı[0], ölçek_kapsamı[1]);
                }
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
            let mut kapsam = y_kapsamlar.get(yi).copied().unwrap_or([0.0, 1.0]);
            let kategoriler = if seçenek.tür == EksenTürü::Kategori {
                kategoriler_derle(seçenek, false, yi)
            } else {
                Vec::new()
            };
            let yakınlaştırma = seçenekler.y_yakınlaştırması(yi).filter(|y| y.etkin_mi());
            let pencere = y_pencereleri
                .get(yi)
                .copied()
                .flatten()
                .map(|(pencere, oranlar, _)| (pencere, oranlar));
            let mut seçenek = seçenek.clone();
            if let Some(([p0, p1], _)) = pencere
                && seçenek.tür != EksenTürü::Kategori
            {
                if let Some(yakınlaştırma) = yakınlaştırma {
                    if yakınlaştırma.başlangıç_değeri.is_some() || yakınlaştırma.başlangıç > 0.001
                    {
                        seçenek.en_az = Some(p0);
                    }
                    if yakınlaştırma.bitiş_değeri.is_some() || yakınlaştırma.bitiş < 99.999
                    {
                        seçenek.en_çok = Some(p1);
                    }
                }
                kapsam = [p0, p1];
            }
            let ölçek = ölçek_kur(&seçenek, kategoriler, kapsam);
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
            let mut eksen =
                ÇalışmaEkseni::yeni(seçenek.clone(), ölçek, [alan.alt(), alan.y], konum);
            if let Some(([p0, p1], oranlar)) = pencere {
                eksen.yakınlaştırma_oranları = Some(oranlar);
                if let Some(yakınlaştırma) = yakınlaştırma {
                    eksen.yakınlaştırma_süzme_kipi = yakınlaştırma.süzme_kipi;
                }
                if seçenek.tür == EksenTürü::Kategori {
                    eksen.değer_penceresi_uygula(p0.round(), p1.round());
                } else {
                    let ölçek_kapsamı = eksen.ölçek.kapsam();
                    eksen.değer_penceresi_uygula(ölçek_kapsamı[0], ölçek_kapsamı[1]);
                }
            }
            eksen
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
        let Some(bölme) = referans_bölme.filter(|b| *b >= 1) else {
            continue;
        };
        let mut ilk_görüldü = false;
        for (yi, eksen) in y_eksenler.iter_mut().enumerate() {
            let eksen_g = eksen
                .seçenek
                .ızgara_sırası
                .min(ızgara_sayısı.saturating_sub(1));
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
    /// İmleç ekseni x mi (dikey imleç) yoksa y mi (yatay imleç)?
    bant_x: bool,
    kategorik: bool,
    eksen_değeri: f64,
    başlık: String,
    satırlar: Vec<İpucuSatırı>,
    parametreler: Vec<İpucuParametresi>,
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
            if satır.im_rengi.is_none() && satır.değer.is_empty() {
                return satır;
            }
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
                crate::model::stil::Biçimleyici::İşlev(f) => f(f64::NAN, &satır.değer),
            };
            İpucuSatırı {
                im_rengi: satır.im_rengi,
                ad: metin,
                değer: String::new(),
            }
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
    // ECharts eksen tetiklemesinde kategorik x/y önceliklidir. Böyle bir
    // eksen yoksa zaman/sayısal x eksenindeki fare değerine en yakın veri
    // noktası seçilir.
    let (bant_ekseni, bant_x, eksen_sırası, kategorik) = kurulum
        .x_eksenler
        .iter()
        .enumerate()
        .find(|(_, e)| e.seçenek.ızgara_sırası == ızgara && e.ölçek.kategorik_mi())
        .map(|(i, e)| (e, true, i, true))
        .or_else(|| {
            kurulum
                .y_eksenler
                .iter()
                .enumerate()
                .find(|(_, e)| e.seçenek.ızgara_sırası == ızgara && e.ölçek.kategorik_mi())
                .map(|(i, e)| (e, false, i, true))
        })
        .or_else(|| {
            kurulum
                .x_eksenler
                .iter()
                .enumerate()
                .find(|(_, e)| e.seçenek.ızgara_sırası == ızgara)
                .map(|(i, e)| (e, true, i, false))
        })?;
    let fare_konumu = if bant_x { fare.0 } else { fare.1 };
    let fare_değeri = bant_ekseni.pikselden_veriye(fare_konumu);
    let (sıra, eksen_değeri) = if kategorik {
        let sıra = fare_değeri.max(0.0) as usize;
        (sıra, sıra as f64)
    } else {
        seçenekler
            .seriler
            .iter()
            .enumerate()
            .filter(|(i, seri)| {
                seri.kartezyen_mi()
                    && kurulum.görünürler.get(*i).copied().unwrap_or(false)
                    && seri.eksen_bağı().x == eksen_sırası
            })
            .flat_map(|(_, seri)| seri.veri().iter().enumerate())
            .filter_map(|(sıra, öğe)| öğe.değer.x().map(|x| (sıra, x)))
            .min_by(|(_, a), (_, b)| (a - fare_değeri).abs().total_cmp(&(b - fare_değeri).abs()))?
    };
    let varsayılan_başlık = bant_ekseni.ölçek.etiket(eksen_değeri);

    // Aynı ızgaradaki paralel kategori eksenleri ortak sıra üzerinde
    // tetiklenir. ECharts her kartezyen çifti için kendi eksen başlığını
    // gösterir; seri sırası grup sırasını belirler.
    let mut gruplar: Vec<(usize, String, Vec<İpucuSatırı>)> = Vec::new();
    let mut parametreler = Vec::new();
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        if !seri.kartezyen_mi() || !kurulum.görünürler.get(i).copied().unwrap_or(false) {
            continue;
        }
        let bağ = seri.eksen_bağı();
        let seri_ekseni = if bant_x {
            kurulum.x_eksenler.get(bağ.x)
        } else {
            kurulum.y_eksenler.get(bağ.y)
        };
        let Some(seri_ekseni) = seri_ekseni else {
            continue;
        };
        let seri_eksen_sırası = if bant_x { bağ.x } else { bağ.y };
        let aynı_tetik = seri_eksen_sırası == eksen_sırası
            || (kategorik
                && seri_ekseni.seçenek.ızgara_sırası == ızgara
                && seri_ekseni.ölçek.kategorik_mi());
        if !aynı_tetik {
            continue;
        }
        let veri_sırası = if kategorik {
            Some(sıra)
        } else {
            seri.veri()
                .iter()
                .enumerate()
                .filter_map(|(sıra, öğe)| öğe.değer.x().map(|x| (sıra, x)))
                .min_by(|(_, a), (_, b)| {
                    (a - eksen_değeri)
                        .abs()
                        .total_cmp(&(b - eksen_değeri).abs())
                })
                .map(|(sıra, _)| sıra)
        };
        let Some(veri_sırası) = veri_sırası else {
            continue;
        };
        let Some(öğe) = seri.veri().get(veri_sırası) else {
            continue;
        };
        let metin = if let Some(dizi) = öğe.değer.dizi() {
            // Mum: A/K/D/Y — Kutu: beş sayının özeti.
            dizi.iter()
                .map(|v| binlik_ayır(*v))
                .collect::<Vec<_>>()
                .join(" / ")
        } else {
            let Some(değer) = öğe.değer.sayı() else {
                continue;
            };
            match &ipucu.değer_biçimleyici {
                Some(b) => b.uygula(değer, &binlik_ayır(değer)),
                None => binlik_ayır(değer),
            }
        };
        let grup_başlığı = if kategorik {
            seri_ekseni.ölçek.etiket(sıra as f64)
        } else {
            varsayılan_başlık.clone()
        };
        let satır = İpucuSatırı {
            im_rengi: Some(seçenekler.seri_rengi(i)),
            ad: seri.ad().unwrap_or("-").to_string(),
            değer: metin,
        };
        match gruplar
            .iter_mut()
            .find(|(grup_sırası, _, _)| *grup_sırası == seri_eksen_sırası)
        {
            Some((_, _, satırlar)) => satırlar.push(satır),
            None => gruplar.push((seri_eksen_sırası, grup_başlığı.clone(), vec![satır])),
        }
        parametreler.push(İpucuParametresi {
            seri_sırası: i,
            seri_adı: seri.ad().unwrap_or("").to_string(),
            veri_sırası,
            ad: öğe.ad.clone().unwrap_or(grup_başlığı),
            değer: öğe.değer.clone(),
        });
    }
    if gruplar.is_empty() {
        return None;
    }
    let mut gruplar = gruplar.into_iter();
    let (_, başlık, mut satırlar) = gruplar.next()?;
    for (_, grup_başlığı, grup_satırları) in gruplar {
        satırlar.push(İpucuSatırı {
            im_rengi: None,
            ad: grup_başlığı,
            değer: String::new(),
        });
        satırlar.extend(grup_satırları);
    }
    Some(Eksenİpucu {
        ızgara,
        bant_x,
        kategorik,
        eksen_değeri,
        başlık,
        satırlar,
        parametreler,
    })
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
    let seçenekler = if seçenekler.veri_kümesi.is_some() || !seçenekler.veri_kümeleri.is_empty()
    {
        let (yeni, _hatalar) = seçenekler.veri_kümesini_uygula();
        türetilmiş = yeni;
        &türetilmiş
    } else {
        seçenekler
    };
    let ilerleme = girdi.ilerleme;
    let zaman_sn = girdi.zaman_sn;
    let fare = girdi.fare;
    // Başsız/Piksel/SVG koşucuları da `legend.selected` başlangıç durumunu
    // görmelidir; gpui'nin etkileşim kümesi bunun üzerine eklenir.
    let mut etkili_kapalı = girdi.kapalı.clone();
    if let Some(gösterge) = &seçenekler.gösterge {
        etkili_kapalı.extend(
            gösterge
                .seçili
                .iter()
                .filter_map(|(ad, seçili)| (!*seçili).then_some(ad.clone())),
        );
    }
    let kapalı = &etkili_kapalı;

    // 1) Arka plan (koyu temada zemin, açıkça verilmemişse de doldurulur).
    let zemin = seçenekler
        .arkaplan
        .clone()
        .or_else(|| seçenekler.koyu.then(|| Dolgu::Düz(crate::tema::zemin())));
    if let Some(dolgu) = zemin {
        let tümü = Dikdörtgen::yeni(0.0, 0.0, yüzey.genişlik(), yüzey.yükseklik());
        yüzey.dikdörtgen(tümü, &dolgu, [0.0; 4], None);
    }

    // Matrix koordinatı seri katmanlarının altında ortak bir bileşendir.
    // Veri sayısı bilinmiyorsa dimension.length/data kendi gövdesini belirler.
    let matris_yerleşimi = seçenekler.matris.as_ref().and_then(|matris| {
        crate::koordinat::MatrisYerleşimi::kur(
            matris,
            (yüzey.genişlik(), yüzey.yükseklik()),
            (0, 0),
        )
        .ok()
    });
    if let (Some(matris), Some(yerleşim)) = (&seçenekler.matris, &matris_yerleşimi) {
        matris_çiz(yüzey, matris, yerleşim);
    }
    let takvim_yerleşimleri: Vec<Option<TakvimYerleşimi>> = seçenekler
        .takvimler
        .iter()
        .map(|takvim| TakvimYerleşimi::kur(takvim, (yüzey.genişlik(), yüzey.yükseklik())).ok())
        .collect();
    for (takvim, yerleşim) in seçenekler.takvimler.iter().zip(&takvim_yerleşimleri) {
        if let Some(yerleşim) = yerleşim {
            takvim_arka_planı_çiz(yüzey, takvim, yerleşim);
        }
    }

    // 2) Başlık.
    if seçenekler.başlıklar.is_empty() {
        if let Some(başlık) = &seçenekler.başlık {
            başlık_çiz(yüzey, başlık);
        }
    } else {
        for başlık in &seçenekler.başlıklar {
            başlık_çiz(yüzey, başlık);
        }
    }

    // 3) Gösterge verisi burada çözülür; legend z=4 olduğundan asıl çizim
    // seri/dataZoom katmanlarından sonra yapılır.
    let gösterge_öğeleri = gösterge_öğeleri(seçenekler, kapalı);

    // 3b) Araç kutusu: ECharts'ın 15 px, çıplak vektör ikonları.
    if let Some(araçlar) = &seçenekler.araç_kutusu
        && araçlar.göster
    {
        let mut özellikler = araçlar.özellik_sırası.clone();
        let varsayılan_sıra = [
            AraçKutusuÖzelliği::VeriGörünümü,
            AraçKutusuÖzelliği::VeriYakınlaştırma,
            AraçKutusuÖzelliği::SihirliÇizgi,
            AraçKutusuÖzelliği::SihirliSütun,
            AraçKutusuÖzelliği::SihirliYığın,
            AraçKutusuÖzelliği::GeriYükle,
            AraçKutusuÖzelliği::SvgKaydet,
            AraçKutusuÖzelliği::PngKaydet,
        ];
        for özellik in varsayılan_sıra {
            if !özellikler.contains(&özellik) {
                özellikler.push(özellik);
            }
        }
        let mut türler = Vec::new();
        for özellik in özellikler {
            match özellik {
                AraçKutusuÖzelliği::VeriGörünümü if araçlar.veri_görünümü => {
                    türler.push(AraçTürü::VeriGörünümü);
                }
                AraçKutusuÖzelliği::VeriYakınlaştırma if araçlar.veri_yakınlaştırma => {
                    türler.push(AraçTürü::VeriYakınlaştır);
                    türler.push(AraçTürü::VeriYakınlaştırmayıGeriAl);
                }
                AraçKutusuÖzelliği::SihirliÇizgi if araçlar.sihirli_çizgi => {
                    türler.push(AraçTürü::SihirliÇizgi);
                }
                AraçKutusuÖzelliği::SihirliSütun if araçlar.sihirli_sütun => {
                    türler.push(AraçTürü::SihirliSütun);
                }
                AraçKutusuÖzelliği::SihirliYığın if araçlar.sihirli_yığın => {
                    türler.push(AraçTürü::SihirliYığın);
                }
                AraçKutusuÖzelliği::GeriYükle if araçlar.geri_yükle => {
                    türler.push(AraçTürü::GeriYükle);
                }
                AraçKutusuÖzelliği::SvgKaydet if araçlar.svg_kaydet => {
                    türler.push(AraçTürü::SvgKaydet);
                }
                AraçKutusuÖzelliği::PngKaydet if araçlar.png_kaydet => {
                    türler.push(AraçTürü::PngKaydet);
                }
                _ => {}
            }
        }
        // zrender kutu yerleşimi simgelerin (5 px hit eşiğiyle genişlemiş)
        // sınır kutularını ve 10 px itemGap'i kullanır.
        let hit_genişliği = |tür: AraçTürü| match tür {
            AraçTürü::VeriGörünümü => 15.580_358,
            AraçTürü::VeriYakınlaştır => 20.0,
            AraçTürü::VeriYakınlaştırmayıGeriAl => 16.904_762,
            AraçTürü::SihirliÇizgi => 20.0,
            AraçTürü::SihirliSütun => 19.383_928,
            AraçTürü::SihirliYığın => 20.0,
            AraçTürü::GeriYükle => 19.915_937,
            AraçTürü::SvgKaydet | AraçTürü::PngKaydet => 17.956_896,
        };
        let hit_yüksekliği = |tür: AraçTürü| match tür {
            AraçTürü::VeriGörünümü => 20.0,
            AraçTürü::VeriYakınlaştır | AraçTürü::VeriYakınlaştırmayıGeriAl => 20.0,
            AraçTürü::SihirliÇizgi => 19.912_452,
            AraçTürü::SihirliSütun => 20.0,
            AraçTürü::SihirliYığın => 18.853_82,
            AraçTürü::GeriYükle => 20.0,
            AraçTürü::SvgKaydet | AraçTürü::PngKaydet => 20.0,
        };
        let mut yerel_merkezler = Vec::with_capacity(türler.len());
        let mut yerel = 0.0;
        for (sıra, tür) in türler.iter().copied().enumerate() {
            if sıra > 0 {
                let önceki = türler.get(sıra - 1).copied().unwrap_or(tür);
                let önceki_boyut = if araçlar.yön == Yön::Yatay {
                    hit_genişliği(önceki)
                } else {
                    hit_yüksekliği(önceki)
                };
                let boyut = if araçlar.yön == Yön::Yatay {
                    hit_genişliği(tür)
                } else {
                    hit_yüksekliği(tür)
                };
                yerel += önceki_boyut / 2.0 + 10.0 + boyut / 2.0;
            }
            yerel_merkezler.push(yerel);
        }
        let ilk_tür = türler.first().copied();
        let son_tür = türler.last().copied();
        let ilk_merkez = yerel_merkezler.first().copied().unwrap_or(0.0);
        let son_merkez = yerel_merkezler.last().copied().unwrap_or(0.0);
        let yatay_en_az = if araçlar.yön == Yön::Yatay {
            ilk_tür
                .map(|tür| ilk_merkez - hit_genişliği(tür) / 2.0)
                .unwrap_or(0.0)
        } else {
            türler
                .iter()
                .copied()
                .map(|tür| -hit_genişliği(tür) / 2.0)
                .fold(0.0f32, f32::min)
        };
        let yatay_en_çok = if araçlar.yön == Yön::Yatay {
            son_tür
                .map(|tür| son_merkez + hit_genişliği(tür) / 2.0)
                .unwrap_or(0.0)
        } else {
            türler
                .iter()
                .copied()
                .map(|tür| hit_genişliği(tür) / 2.0)
                .fold(0.0f32, f32::max)
        };
        let dikey_en_az = if araçlar.yön == Yön::Dikey {
            ilk_tür
                .map(|tür| ilk_merkez - hit_yüksekliği(tür) / 2.0)
                .unwrap_or(0.0)
        } else {
            -10.0
        };
        let dikey_en_çok = if araçlar.yön == Yön::Dikey {
            son_tür
                .map(|tür| son_merkez + hit_yüksekliği(tür) / 2.0)
                .unwrap_or(0.0)
        } else {
            10.0
        };
        let grup_x = if let Some(sağ) = araçlar.sağ {
            yüzey.genişlik() - 15.0 - sağ.çöz(yüzey.genişlik()) - yatay_en_çok
        } else {
            match araçlar.sol {
                YatayKonum::Sol => 15.0 - yatay_en_az,
                YatayKonum::Orta => yüzey.genişlik() / 2.0 - (yatay_en_az + yatay_en_çok) / 2.0,
                YatayKonum::Sağ => yüzey.genişlik() - 15.0 - yatay_en_çok,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(yüzey.genişlik()) - yatay_en_az,
            }
        };
        let grup_y = match araçlar.üst {
            DikeyKonum::Üst => 15.0 - dikey_en_az,
            DikeyKonum::Orta => yüzey.yükseklik() / 2.0 - (dikey_en_az + dikey_en_çok) / 2.0,
            DikeyKonum::Alt => yüzey.yükseklik() - 15.0 - dikey_en_çok,
            DikeyKonum::Değer(uzunluk) => uzunluk.çöz(yüzey.yükseklik()) - dikey_en_az,
        };
        let renk = crate::renk::Renk::onaltılık(0x6578ba);
        for (sıra, tür) in türler.into_iter().enumerate() {
            let yerel = yerel_merkezler.get(sıra).copied().unwrap_or(0.0);
            let merkez = if araçlar.yön == Yön::Yatay {
                (grup_x + yerel, grup_y)
            } else {
                (grup_x, grup_y + yerel)
            };
            let hit = hit_genişliği(tür);
            let hit_y = hit_yüksekliği(tür);
            let kutu = Dikdörtgen::yeni(merkez.0 - hit / 2.0, merkez.1 - hit_y / 2.0, hit, hit_y);
            let mut yol = crate::cizim::Yol::yeni();
            match tür {
                AraçTürü::VeriGörünümü => {
                    let n = |x, y| araç_noktası(merkez, [11.5, 2.0, 51.0, 58.0], (x, y));
                    yol.taşı(n(17.5, 17.3));
                    yol.çiz(n(33.0, 17.3));
                    yol.taşı(n(17.5, 17.3));
                    yol.çiz(n(33.0, 17.3));
                    yol.taşı(n(45.4, 29.5));
                    yol.çiz(n(17.4, 29.5));
                    yol.taşı(n(11.5, 2.0));
                    yol.çiz(n(11.5, 58.0));
                    yol.çiz(n(51.0, 58.0));
                    yol.çiz(n(51.0, 14.8));
                    yol.çiz(n(38.4, 2.0));
                    yol.kapat();
                    yol.taşı(n(38.4, 2.2));
                    yol.çiz(n(38.4, 14.9));
                    yol.çiz(n(51.0, 14.9));
                    yol.taşı(n(45.4, 41.7));
                    yol.çiz(n(17.4, 41.7));
                }
                AraçTürü::VeriYakınlaştır => {
                    // ECharts DataZoomFeature.defaultOption.icon.zoom.
                    let n = |x, y| araç_noktası(merkez, [0.0, 0.0, 58.0, 58.0], (x, y));
                    yol.taşı(n(0.0, 13.5));
                    yol.çiz(n(26.9, 13.5));
                    yol.taşı(n(13.5, 26.9));
                    yol.çiz(n(13.5, 0.0));
                    yol.taşı(n(32.1, 13.5));
                    yol.çiz(n(58.0, 13.5));
                    yol.çiz(n(58.0, 58.0));
                    yol.çiz(n(13.5, 58.0));
                    yol.çiz(n(13.5, 32.1));
                }
                AraçTürü::VeriYakınlaştırmayıGeriAl => {
                    // ECharts DataZoomFeature.defaultOption.icon.back.
                    let n = |x, y| araç_noktası(merkez, [9.9, 1.4, 54.9, 58.1], (x, y));
                    yol.taşı(n(22.0, 1.4));
                    yol.çiz(n(9.9, 13.5));
                    yol.çiz(n(22.2, 25.8));
                    yol.taşı(n(10.3, 13.5));
                    yol.çiz(n(54.9, 13.5));
                    yol.çiz(n(54.9, 58.1));
                    yol.çiz(n(10.3, 58.1));
                    yol.çiz(n(10.3, 32.1));
                }
                AraçTürü::SihirliÇizgi => {
                    let n = |x, y| araç_noktası(merkez, [4.1, 6.9, 55.5, 58.0], (x, y));
                    yol.taşı(n(4.1, 28.9));
                    yol.çiz(n(11.2, 28.9));
                    yol.çiz(n(20.5, 6.9));
                    yol.çiz(n(27.9, 44.9));
                    yol.çiz(n(37.6, 25.2));
                    yol.çiz(n(40.6, 38.0));
                    yol.çiz(n(55.5, 38.0));
                    yol.taşı(n(4.1, 58.0));
                    yol.çiz(n(55.5, 58.0));
                }
                AraçTürü::SihirliSütun => {
                    let n = |x, y| araç_noktası(merkez, [3.1, 2.0, 56.8, 58.0], (x, y));
                    for (x0, y0, x1, y1) in [
                        (6.7, 22.9, 16.7, 48.0),
                        (24.9, 13.0, 34.9, 48.0),
                        (43.2, 2.0, 53.2, 48.0),
                    ] {
                        yol.taşı(n(x0, y0));
                        yol.çiz(n(x1, y0));
                        yol.çiz(n(x1, y1));
                        yol.çiz(n(x0, y1));
                        yol.kapat();
                    }
                    yol.taşı(n(3.1, 58.0));
                    yol.çiz(n(56.8, 58.0));
                }
                AraçTürü::SihirliYığın => {
                    let n = |x, y| araç_noktası(merkez, [-0.2, 2.2, 60.0, 57.8], (x, y));
                    for noktalar in [
                        vec![
                            (8.2, 38.4),
                            (-0.2, 42.5),
                            (30.4, 57.8),
                            (60.0, 42.5),
                            (51.9, 38.4),
                            (30.4, 49.4),
                        ],
                        vec![
                            (51.9, 30.0),
                            (43.8, 34.2),
                            (30.4, 41.1),
                            (16.5, 34.2),
                            (8.2, 30.0),
                            (-0.2, 34.2),
                            (8.2, 38.4),
                            (30.4, 49.4),
                            (51.9, 38.4),
                            (60.0, 34.2),
                        ],
                        vec![
                            (51.9, 21.7),
                            (43.8, 25.9),
                            (35.7, 30.0),
                            (30.4, 32.8),
                            (24.9, 30.0),
                            (16.5, 25.9),
                            (8.2, 21.7),
                            (-0.2, 25.9),
                            (8.2, 30.0),
                            (16.5, 34.2),
                            (30.4, 41.1),
                            (43.8, 34.2),
                            (51.9, 30.0),
                            (60.0, 25.9),
                        ],
                        vec![
                            (30.4, 2.2),
                            (-0.2, 17.5),
                            (8.2, 21.6),
                            (16.5, 25.8),
                            (24.9, 30.0),
                            (30.4, 32.7),
                            (35.7, 30.0),
                            (43.8, 25.8),
                            (51.9, 21.6),
                            (60.0, 17.5),
                        ],
                    ] {
                        if let Some(ilk) = noktalar.first().copied() {
                            yol.taşı(n(ilk.0, ilk.1));
                            for nokta in noktalar.iter().skip(1).copied() {
                                yol.çiz(n(nokta.0, nokta.1));
                            }
                            yol.kapat();
                        }
                    }
                }
                AraçTürü::GeriYükle => {
                    let n = |x, y| araç_noktası(merkez, [1.6, 0.6, 57.9, 58.0], (x, y));
                    yol.taşı(n(47.0, 18.9));
                    yol.çiz(n(56.8, 18.9));
                    yol.çiz(n(56.8, 8.7));
                    yol.taşı(n(56.3, 20.1));
                    yol.kübik(n(52.1, 9.0), n(40.5, 0.6), n(26.8, 2.1));
                    yol.kübik(n(12.6, 3.7), n(1.6, 16.2), n(2.1, 30.6));
                    yol.taşı(n(13.0, 41.1));
                    yol.çiz(n(3.1, 41.1));
                    yol.çiz(n(3.1, 51.3));
                    yol.taşı(n(3.7, 39.9));
                    yol.kübik(n(7.9, 51.0), n(19.5, 59.4), n(33.2, 57.9));
                    yol.kübik(n(47.4, 56.3), n(58.4, 43.8), n(57.9, 29.4));
                }
                AraçTürü::SvgKaydet | AraçTürü::PngKaydet => {
                    let n = |x, y| araç_noktası(merkez, [4.6, 0.0, 54.7, 58.0], (x, y));
                    yol.taşı(n(4.7, 22.9));
                    yol.çiz(n(29.3, 45.5));
                    yol.çiz(n(54.7, 23.4));
                    yol.taşı(n(4.6, 43.6));
                    yol.çiz(n(4.6, 58.0));
                    yol.çiz(n(53.8, 58.0));
                    yol.çiz(n(53.8, 43.6));
                    yol.taşı(n(29.2, 45.1));
                    yol.çiz(n(29.2, 0.0));
                }
            }
            yüzey.yol_çiz(&yol, 1.0, renk, ÇizgiTürü::Düz);
            çıktı.araç_düğmeleri.push((kutu, tür));
        }
    }

    let ipucu_seçeneği = seçenekler.ipucu.clone().filter(|i| i.göster);

    // 4) Kartezyen bölüm (çoklu ızgara/eksen).
    let kurulum = kartezyen_kur(yüzey, seçenekler, kapalı);
    // `(başlık, satırlar, konum)`.
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
                && ipucu.imleç == İmleçTürü::Gölge
                && eksen_ip.ızgara == g
                && eksen_ip.kategorik
            {
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
                    let merkez = bant_ekseni.veriden_piksele(eksen_ip.eksen_değeri);
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
            let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
                continue;
            };
            if let Some(imleyiciler) = seri.imleyiciler()
                && imleyiciler.alan.is_some()
            {
                im_alanlarını_çiz(
                    yüzey,
                    imleyiciler,
                    seri,
                    &kartezyen,
                    seçenekler.seri_rengi(i),
                );
            }
        }

        // Sütunlar değer eksenine göre değil ortak kategori (taban) eksenine
        // göre gruplanır. ECharts böylece iki ayrı yAxis'e bağlı sütunları da
        // aynı kategoride yan yana yerleştirir.
        let sütun_grup_anahtarı = |seri: &Seri| {
            let bağ = seri.eksen_bağı();
            let y_kategorik = kurulum
                .y_eksenler
                .get(bağ.y)
                .map(|eksen| eksen.ölçek.kategorik_mi())
                .unwrap_or(false);
            if y_kategorik {
                (false, bağ.y)
            } else {
                (true, bağ.x)
            }
        };
        let mut sütun_grupları: Vec<((bool, usize), Vec<SütunGirdisi>)> = Vec::new();
        for (i, s) in seçenekler.seriler.iter().enumerate() {
            if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            if let Seri::Sütun(sütun) = s {
                let Some(seri_kartezyeni) = kurulum.seri_kartezyeni(s) else {
                    continue;
                };
                let anahtar = sütun_grup_anahtarı(s);
                let girdi = SütunGirdisi {
                    seri: sütun,
                    kartezyen: seri_kartezyeni,
                    genel_sıra: i,
                    aralıklar: kurulum.aralıklar.get(i).map(Vec::as_slice).unwrap_or(&[]),
                    renk: seçenekler.seri_rengi(i),
                };
                match sütun_grupları.iter_mut().find(|(aday, _)| *aday == anahtar) {
                    Some((_, grup)) => grup.push(girdi),
                    None => sütun_grupları.push((anahtar, vec![girdi])),
                }
            }
        }
        let mut çizilen_sütun_grupları: HashSet<(bool, usize)> = HashSet::new();

        // Saçılım vurgusu (öğe ipucu) için önden isabet araması.
        // `(seri sırası, vurgulu veri sırası, noktalar)`.
        type SaçılımVurgusu = (usize, Option<usize>, Vec<SaçılımNoktası>);
        let mut saçılım_vurguları: Vec<SaçılımVurgusu> = Vec::new();
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if let Seri::Saçılım(s) = seri {
                if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                    continue;
                }
                let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
                    continue;
                };
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

        // LineView alan poligonlarını z2=0, polylineleri z2=10 ile boyar.
        // Bütün alanları önce geçirmek, sonraki yığın dolgularının daha önce
        // çizilmiş sınır çizgilerini örtmesini engeller.
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            let Seri::Çizgi(çizgi) = seri else { continue };
            if çizgi.alan_stili.is_none() || !kurulum.görünürler.get(i).copied().unwrap_or(false)
            {
                continue;
            }
            let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
                continue;
            };
            let aralıklar = kurulum.aralıklar.get(i).map(Vec::as_slice).unwrap_or(&[]);
            let mut alanı_çiz = |yüzey: &mut dyn ÇizimYüzeyi| {
                çizgi_serisi_çiz(
                    yüzey,
                    çizgi,
                    &kartezyen,
                    aralıklar,
                    seçenekler.seri_rengi(i),
                    seçenekler.seri_görsel_eşlemesi(i),
                    ilerleme,
                    ÇizgiKatmanı::Alan,
                    None,
                );
            };
            if kartezyen.x.pencere.is_some() || kartezyen.y.pencere.is_some() {
                yüzey.kırpılı(kartezyen.alan, &mut alanı_çiz);
            } else {
                alanı_çiz(yüzey);
            }
        }

        // ECharts `labelLayout.moveOverlap: 'shiftY'`: aynı eksen çiftine
        // bağlı çizgilerin uç etiketlerini ham y konumuna göre sıralar ve
        // her sınır kutusunu bir öncekinin hemen altına iter. Etiketler seri
        // döngüsünde ayrı ayrı boyansa da yerleşim ortak hesaplanmalıdır.
        let mut uç_etiketi_adayları = Vec::new();
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            let Seri::Çizgi(çizgi) = seri else { continue };
            if !çizgi.uç_etiketi.göster
                || !çizgi.etiket_örtüşmesini_dikey_kaydır
                || !kurulum.görünürler.get(i).copied().unwrap_or(false)
            {
                continue;
            }
            let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
                continue;
            };
            let aralıklar = kurulum.aralıklar.get(i).map(Vec::as_slice).unwrap_or(&[]);
            let (tepeler, _) = nokta_listeleri(çizgi, &kartezyen, aralıklar);
            let son = tepeler.iter().enumerate().rev().find_map(|(sıra, nokta)| {
                let nokta = (*nokta)?;
                let öğe = çizgi.veri.get(sıra)?;
                let x_değeri = öğe.değer.x().unwrap_or(sıra as f64);
                let y_değeri = öğe.değer.sayı()?;
                (kartezyen.x.pencerede_mi(x_değeri) && kartezyen.y.pencerede_mi(y_değeri))
                    .then_some(nokta.1)
            });
            if let Some(y) = son {
                let yükseklik = çizgi.uç_etiketi.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                uç_etiketi_adayları.push((i, çizgi.eksen_bağı, y, yükseklik));
            }
        }
        let uç_etiketi_yerleşimleri =
            çizgi_uç_etiketlerini_dikey_kaydır(&uç_etiketi_adayları, seçenekler.seriler.len());

        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
                continue;
            };
            // Yakınlaştırma penceresi etkinse seri ızgaraya kırpılır
            // (ECharts `clip: true`).
            let pencereli = kartezyen.x.pencere.is_some()
                || kartezyen.y.pencere.is_some()
                || matches!(seri, Seri::Hatlar(hatlar) if hatlar.kırp);
            let mut yerel_isabetler: Vec<İsabetBölgesi> = Vec::new();
            let mut yerel_ipucu: Option<Bekleyenİpucu> = None;
            let mut seri_çiz =
                |yüzey: &mut dyn ÇizimYüzeyi,
                 isabetler: &mut Vec<İsabetBölgesi>,
                 bekleyen: &mut Option<Bekleyenİpucu>| {
                    match seri {
                        Seri::Çizgi(s) => {
                            let seri_aralıkları =
                                kurulum.aralıklar.get(i).map(Vec::as_slice).unwrap_or(&[]);
                            çizgi_serisi_çiz(
                                yüzey,
                                s,
                                &kartezyen,
                                seri_aralıkları,
                                seçenekler.seri_rengi(i),
                                seçenekler.seri_görsel_eşlemesi(i),
                                ilerleme,
                                ÇizgiKatmanı::ÇizgiVeSembol,
                                uç_etiketi_yerleşimleri.get(i).copied().flatten(),
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
                            let anahtar = sütun_grup_anahtarı(seri);
                            if çizilen_sütun_grupları.insert(anahtar)
                                && let Some((_, girdiler)) =
                                    sütun_grupları.iter().find(|(aday, _)| *aday == anahtar)
                            {
                                sütunları_çiz(yüzey, girdiler, ilerleme, isabetler);
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
                                    && let Some(nokta) = noktalar.iter().find(|n| n.sıra == *sıra)
                                {
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
                        Seri::Mum(s) => mum_çiz(yüzey, s, i, &kartezyen, ilerleme, isabetler),
                        Seri::Kutu(s) => {
                            // ECharts kutu serilerini aynı kategorik taban
                            // ekseni üzerinde yan yana yerleştirir. Değer
                            // ekseni farklı olsa bile ortak taban ekseni aynı
                            // yerleşim istatistiğini paylaşır.
                            let yatay = kartezyen.y.ölçek.kategorik_mi()
                                && !kartezyen.x.ölçek.kategorik_mi();
                            let bağ = seri.eksen_bağı();
                            let grup: Vec<usize> = seçenekler
                                .seriler
                                .iter()
                                .enumerate()
                                .filter_map(|(sıra, aday)| {
                                    if !kurulum.görünürler.get(sıra).copied().unwrap_or(false)
                                        || !matches!(aday, Seri::Kutu(_))
                                    {
                                        return None;
                                    }
                                    let aday_kartezyen = kurulum.seri_kartezyeni(aday)?;
                                    let aday_yatay = aday_kartezyen.y.ölçek.kategorik_mi()
                                        && !aday_kartezyen.x.ölçek.kategorik_mi();
                                    let aday_bağ = aday.eksen_bağı();
                                    let aynı_taban = if yatay {
                                        aday_yatay && aday_bağ.y == bağ.y
                                    } else {
                                        !aday_yatay && aday_bağ.x == bağ.x
                                    };
                                    aynı_taban.then_some(sıra)
                                })
                                .collect();
                            let grup_sırası = grup.iter().position(|&sıra| sıra == i).unwrap_or(0);
                            kutu_çiz(
                                yüzey,
                                s,
                                i,
                                &kartezyen,
                                grup_sırası,
                                grup.len(),
                                seçenekler.seri_rengi(i),
                                isabetler,
                            )
                        }
                        Seri::Isı(s) => {
                            let eşleme = seçenekler.görsel_eşleme.clone().unwrap_or_default();
                            let kapsam = eşleme.kapsam_çöz(ısı_değer_kapsamı(s));
                            let vurgulu = ısı_haritası_çiz(
                                yüzey, s, i, &kartezyen, &eşleme, kapsam, ilerleme, fare, isabetler,
                            );
                            let programatik = girdi
                                .ipucu_öğesi
                                .filter(|(seri_sırası, _)| *seri_sırası == i)
                                .map(|(_, veri_sırası)| veri_sırası);
                            if let (Some(veri_sırası), Some(ipucu)) =
                                (vurgulu.or(programatik), ipucu_seçeneği.as_ref())
                                && ipucu.tetikleme == Tetikleme::Öğe
                                && let Some(öğe) = s.veri.get(veri_sırası)
                                && let Some(dizi) = öğe.değer.dizi()
                                && let (Some(&x), Some(&değer)) = (dizi.first(), dizi.get(2))
                            {
                                let renk = eşleme.renk_çöz(değer, kapsam);
                                let hücre = isabetler
                                    .iter()
                                    .rev()
                                    .find(|b| b.seri_sırası == i && b.veri_sırası == veri_sırası)
                                    .and_then(|b| match &b.geometri {
                                        İsabetGeometrisi::Dikdörtgen(d) => Some(*d),
                                        _ => None,
                                    });
                                let konum = match (ipucu.konum, hücre, fare) {
                                    (crate::model::bilesen::İpucuKonumu::Üst, Some(d), _) => {
                                        // Heatmap hücresi kaynak rect'in kenar
                                        // saçaklanması için her yönde 0,25 px
                                        // büyütülür; tooltip ise asıl bandın
                                        // tam üst sınırına bağlanır.
                                        (d.merkez().0, d.y + 0.25)
                                    }
                                    (_, _, Some(f)) => f,
                                    (_, Some(d), None) => d.merkez(),
                                    (_, None, None) => kartezyen.nokta(x, dizi[1]),
                                };
                                *bekleyen = Some((
                                    s.ad.clone(),
                                    vec![İpucuSatırı {
                                        im_rengi: Some(renk),
                                        ad: kartezyen.x.ölçek.etiket(x),
                                        değer: binlik_ayır(değer),
                                    }],
                                    konum,
                                ));
                            }
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
                        Seri::Hatlar(s) => {
                            hatlar_çiz(
                                yüzey,
                                s,
                                i,
                                &|nokta| kartezyen_hat_noktası(nokta, &kartezyen),
                                seçenekler.seri_rengi(i),
                                ilerleme,
                                zaman_sn,
                                isabetler,
                            );
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
            let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
                continue;
            };
            if let Some(imleyiciler) = seri.imleyiciler()
                && (imleyiciler.çizgi.is_some() || imleyiciler.nokta.is_some())
            {
                let kategori_kaydırması = if matches!(seri, Seri::Sütun(_)) {
                    let anahtar = sütun_grup_anahtarı(seri);
                    sütun_grupları
                        .iter()
                        .find(|(aday, _)| *aday == anahtar)
                        .and_then(|(_, girdiler)| {
                            let bant_genişliği = if kartezyen.x.ölçek.kategorik_mi() {
                                kartezyen.x.bant_genişliği()
                            } else {
                                kartezyen.y.bant_genişliği()
                            };
                            let konumlar = yerleşim_hesapla(girdiler, bant_genişliği);
                            girdiler
                                .iter()
                                .zip(konumlar)
                                .find(|(girdi, _)| girdi.genel_sıra == i)
                                .map(|(_, konum)| konum.kaydırma + konum.genişlik / 2.0)
                        })
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                im_çizgi_ve_noktalarını_çiz(
                    yüzey,
                    imleyiciler,
                    seri,
                    &kartezyen,
                    seçenekler.seri_rengi(i),
                    kategori_kaydırması,
                );
            }
        }

        // Çapraz imleç: fareden geçen kesikli yatay+dikey çizgiler ve
        // eksen kenarlarında değer etiketleri (`axisPointer: cross`).
        if let (Some(ipucu), Some(f)) = (&ipucu_seçeneği, fare)
            && ipucu.imleç == İmleçTürü::Çapraz
            && let Some(g) = kurulum.faredeki_ızgara(f)
            && let Some(kartezyen) = kurulum.birincil_kartezyen(g)
        {
            let alan = kartezyen.alan;
            let (fx, fy) = (keskin(f.0), keskin(f.1));
            yüzey.çizgi(
                (fx, alan.alt()),
                (fx, alan.y),
                1.0,
                tema::nötr_30(),
                ÇizgiTürü::Kesikli,
            );
            yüzey.çizgi(
                (alan.x, fy),
                (alan.sağ(), fy),
                1.0,
                tema::imleç_çizgisi(),
                ÇizgiTürü::Kesikli,
            );
            let arkaplan = ipucu.imleç_etiketi_arkaplanı.unwrap_or(tema::nötr_70());
            let mut kenar_etiketi =
                |metin: &str, koordinat: f32, konum: EksenKonumu, yatay: bool| {
                    let boyut = tema::YAZI_KÜÇÜK;
                    let (gş, y) = yüzey.yazı_ölç(metin, boyut);
                    let genişlik = gş + 14.0;
                    let yükseklik = y + 10.0;
                    let kutu = if yatay {
                        let üst = match konum {
                            EksenKonumu::Üst => alan.y - yükseklik - 3.0,
                            _ => alan.alt() + 3.0,
                        };
                        Dikdörtgen::yeni(koordinat - genişlik / 2.0, üst, genişlik, yükseklik)
                    } else {
                        let sol = match konum {
                            EksenKonumu::Sağ => alan.sağ() + 3.0,
                            _ => alan.x - genişlik - 3.0,
                        };
                        Dikdörtgen::yeni(sol, koordinat - yükseklik / 2.0, genişlik, yükseklik)
                    };
                    yüzey.dikdörtgen(kutu, &Dolgu::Düz(arkaplan), [2.0; 4], None);
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
            for eksen in kurulum
                .x_eksenler
                .iter()
                .filter(|eksen| eksen.seçenek.ızgara_sırası == g)
            {
                let metin = eksen.ölçek.etiket(eksen.pikselden_veriye(f.0));
                kenar_etiketi(&metin, f.0, eksen.konum, true);
            }
            for eksen in kurulum
                .y_eksenler
                .iter()
                .filter(|eksen| eksen.seçenek.ızgara_sırası == g)
            {
                let metin = eksen.ölçek.etiket(eksen.pikselden_veriye(f.1));
                kenar_etiketi(&metin, fy, eksen.konum, false);
            }
        }

        // Veri yakınlaştırma: iç alan kayıtları + sürgü çizimi.
        for (z, yakınlaştırma) in seçenekler.veri_yakınlaştırmaları.iter().enumerate() {
            if !yakınlaştırma.göster {
                continue;
            }
            let dikey = yakınlaştırma.dikey_mi();
            let hedef_ızgara = if let Some(y_sırası) = yakınlaştırma.y_eksen_sırası {
                kurulum
                    .y_eksenler
                    .get(y_sırası)
                    .map(|eksen| eksen.seçenek.ızgara_sırası)
            } else {
                kurulum
                    .x_eksenler
                    .get(yakınlaştırma.x_eksen_sırası)
                    .map(|eksen| eksen.seçenek.ızgara_sırası)
            };
            let Some(hedef_ızgara) = hedef_ızgara else {
                continue;
            };
            let Some(alan) = kurulum.ızgara_alanları.get(hedef_ızgara).copied() else {
                continue;
            };
            match yakınlaştırma.tür {
                YakınlaştırmaTürü::İç => {
                    çıktı.iç_yakınlaştırmalar.push(İçYakınlaştırmaAlanı {
                        yakınlaştırma_sırası: z,
                        alan,
                        dikey,
                    });
                }
                YakınlaştırmaTürü::Sürgü => {
                    let (b, e) = if let Some(y_sırası) = yakınlaştırma.y_eksen_sırası {
                        kurulum
                            .y_eksenler
                            .get(y_sırası)
                            .and_then(|eksen| eksen.yakınlaştırma_oranları)
                            .unwrap_or_else(|| yakınlaştırma.oranlar())
                    } else {
                        kurulum
                            .x_eksenler
                            .get(yakınlaştırma.x_eksen_sırası)
                            .and_then(|eksen| eksen.yakınlaştırma_oranları)
                            .unwrap_or_else(|| yakınlaştırma.oranlar())
                    };
                    let şerit = if dikey {
                        let genişlik = yakınlaştırma
                            .genişlik
                            .map(|u| u.çöz(yüzey.genişlik()))
                            .unwrap_or(30.0);
                        let yükseklik = yakınlaştırma
                            .yükseklik
                            .map(|u| u.çöz(yüzey.yükseklik()))
                            .unwrap_or(alan.yükseklik);
                        let x = yakınlaştırma
                            .sol
                            .map(|u| u.çöz(yüzey.genişlik()))
                            .unwrap_or(yüzey.genişlik() - genişlik - 15.0)
                            + 0.5;
                        let y = yakınlaştırma
                            .üst
                            .map(|u| u.çöz(yüzey.yükseklik()))
                            .or_else(|| {
                                yakınlaştırma.alt.map(|u| {
                                    yüzey.yükseklik() - u.çöz(yüzey.yükseklik()) - yükseklik
                                })
                            })
                            .unwrap_or((yüzey.yükseklik() - yükseklik) / 2.0 + 10.0);
                        Dikdörtgen::yeni(x, y, genişlik, yükseklik)
                    } else {
                        let genişlik = yakınlaştırma
                            .genişlik
                            .map(|u| u.çöz(yüzey.genişlik()))
                            .unwrap_or(alan.genişlik);
                        let yükseklik = yakınlaştırma
                            .yükseklik
                            .map(|u| u.çöz(yüzey.yükseklik()))
                            .unwrap_or(30.0);
                        let x = yakınlaştırma
                            .sol
                            .map(|u| u.çöz(yüzey.genişlik()))
                            .unwrap_or(alan.x)
                            + 2.8;
                        let y = yakınlaştırma
                            .üst
                            .map(|u| u.çöz(yüzey.yükseklik()) + 6.5)
                            .or_else(|| {
                                yakınlaştırma.alt.map(|u| {
                                    yüzey.yükseklik() - u.çöz(yüzey.yükseklik()) - yükseklik + 6.5
                                })
                            })
                            .unwrap_or(yüzey.yükseklik() - 45.5);
                        Dikdörtgen::yeni(x, y, genişlik, yükseklik)
                    };
                    let kenarlık = crate::renk::Renk::onaltılık(0xe0e4f2);
                    let vurgu = crate::renk::Renk::onaltılık(0x8292cc);
                    // Şerit zemini.
                    yüzey.dikdörtgen(
                        şerit,
                        &Dolgu::Düz(crate::renk::Renk::SAYDAM),
                        [0.0; 4],
                        None,
                    );
                    // ECharts `showDataShadow: auto`: ilk uygun serinin tüm
                    // verisini sürgünün arkasında düşük opaklıklı alan olarak
                    // gösterir.
                    if yakınlaştırma.veri_gölgesi
                        && !dikey
                        && let Some(veri) = seçenekler
                            .seriler
                            .iter()
                            .map(Seri::veri)
                            .find(|veri| !veri.is_empty())
                    {
                        let kapsam = veri
                            .iter()
                            .filter_map(|öğe| öğe.değer.sayı())
                            .filter(|değer| değer.is_finite())
                            .fold(
                                [f64::INFINITY, f64::NEG_INFINITY],
                                |[en_az, en_çok], değer| [en_az.min(değer), en_çok.max(değer)],
                            );
                        // SliderZoomView `_renderDataShadow`, ham karşı-eksen
                        // kapsamını veri açıklığının %30'u kadar iki yönde
                        // genişletir. Sıfır merkezli varsayım, uzun rastgele
                        // yürüyüşlerin negatif yarısını kırpıyordu.
                        let açıklık = (kapsam[1] - kapsam[0]).max(f64::EPSILON);
                        let alt_kapsam = kapsam[0] - açıklık * 0.3;
                        let üst_kapsam = kapsam[1] + açıklık * 0.3;
                        let eşle = |değer: f64| {
                            ((değer - alt_kapsam) / (üst_kapsam - alt_kapsam)) as f32
                                * şerit.yükseklik
                        };
                        // SliderZoomView zaman ekseninde yatay gölge
                        // koordinatını veri sırasından değil ham zaman
                        // değerinin kapsam içindeki konumundan üretir. Bu
                        // ayrım, seans aralarındaki boş satırların önizlemede
                        // doğru genişlikte görünmesi için gereklidir. Kırık
                        // eksen sıkıştırması burada özellikle uygulanmaz:
                        // ECharts `_renderDataShadow` da `getDataExtent` ile
                        // ham zaman kapsamını doğrusal eşler.
                        let zaman_x_kapsamı = kurulum
                            .x_eksenler
                            .get(yakınlaştırma.x_eksen_sırası)
                            .filter(|eksen| eksen.seçenek.tür == EksenTürü::Zaman)
                            .and_then(|_| {
                                let kapsam = veri
                                    .iter()
                                    .filter_map(|öğe| öğe.değer.x())
                                    .filter(|değer| değer.is_finite())
                                    .fold(
                                        [f64::INFINITY, f64::NEG_INFINITY],
                                        |[en_az, en_çok], değer| {
                                            [en_az.min(değer), en_çok.max(değer)]
                                        },
                                    );
                                (kapsam[0].is_finite()
                                    && kapsam[1].is_finite()
                                    && kapsam[1] > kapsam[0])
                                    .then_some(kapsam)
                            });
                        let mut alan_yolu = crate::cizim::Yol::yeni();
                        alan_yolu.taşı((şerit.sağ(), şerit.alt()));
                        alan_yolu.çiz((şerit.x, şerit.alt()));
                        let mut çizgi_yolu = crate::cizim::Yol::yeni();
                        let mut çizgi_başladı = false;
                        let mut son_boş = false;
                        let mut son_x = şerit.x;
                        // ECharts büyük veri gölgesinde yaklaşık bir örnek /
                        // yatay piksel bırakır (`Math.round(count / width)`).
                        let adım =
                            ((veri.len() as f32 / şerit.genişlik.max(1.0)).round() as usize).max(1);
                        for (sıra, öğe) in veri.iter().enumerate() {
                            if sıra % adım != 0 {
                                continue;
                            }
                            let oran =
                                zaman_x_kapsamı
                                    .and_then(|[en_az, en_çok]| {
                                        öğe.değer.x().filter(|değer| değer.is_finite()).map(
                                            |değer| ((değer - en_az) / (en_çok - en_az)) as f32,
                                        )
                                    })
                                    .unwrap_or_else(|| {
                                        if veri.len() > 1 {
                                            sıra as f32 / (veri.len() - 1) as f32
                                        } else {
                                            0.5
                                        }
                                    });
                            let x = şerit.x + şerit.genişlik * oran;
                            let değer = öğe.değer.sayı().filter(|değer| değer.is_finite());
                            if değer.is_none() && !son_boş && sıra > 0 {
                                alan_yolu.çiz((son_x, şerit.alt()));
                                if çizgi_başladı {
                                    çizgi_yolu.çiz((son_x, şerit.alt()));
                                }
                            } else if değer.is_some() && son_boş {
                                alan_yolu.çiz((x, şerit.alt()));
                                if çizgi_başladı {
                                    çizgi_yolu.çiz((x, şerit.alt()));
                                }
                            }
                            if let Some(değer) = değer {
                                let nokta = (x, şerit.alt() - eşle(değer));
                                alan_yolu.çiz(nokta);
                                if çizgi_başladı {
                                    çizgi_yolu.çiz(nokta);
                                } else {
                                    çizgi_yolu.taşı(nokta);
                                    çizgi_başladı = true;
                                }
                            }
                            son_boş = değer.is_none();
                            son_x = x;
                        }
                        alan_yolu.kapat();
                        let seçili_sol = şerit.x + şerit.genişlik * b;
                        let seçili_sağ = şerit.x + şerit.genişlik * e;
                        let parçalar = [
                            (şerit.x, seçili_sol, false),
                            (seçili_sol, seçili_sağ, true),
                            (seçili_sağ, şerit.sağ(), false),
                        ];
                        for (parça_sol, parça_sağ, seçili) in parçalar {
                            if parça_sağ <= parça_sol {
                                continue;
                            }
                            let kırpma = Dikdörtgen::yeni(
                                parça_sol,
                                şerit.y,
                                parça_sağ - parça_sol,
                                şerit.yükseklik,
                            );
                            let alan_rengi = crate::renk::Renk::onaltılık(0xc0c9e6)
                                // Canvas kaynak-üstte birleşimi en yakın
                                // kanala yuvarlar; tiny-skia yolu aşağı
                                // kırptığı için aynı son rengi bu opaklıklar
                                // verir.
                                .opaklık(if seçili { 0.29 } else { 0.19 });
                            let çizgi_rengi = crate::renk::Renk::onaltılık(if seçili {
                                0x8292cc
                            } else {
                                0xa1aed9
                            });
                            yüzey.kırpılı(kırpma, &mut |çizici| {
                                çizici.yol_doldur(&alan_yolu, &Dolgu::Düz(alan_rengi));
                                çizici.yol_çiz(&çizgi_yolu, 0.5, çizgi_rengi, ÇizgiTürü::Düz);
                            });
                        }
                    }
                    // Seçili pencere.
                    let pencere = if dikey {
                        let p_y = şerit.y + şerit.yükseklik * (1.0 - e);
                        let p_yükseklik = (şerit.yükseklik * (e - b)).max(4.0);
                        Dikdörtgen::yeni(şerit.x, p_y, şerit.genişlik, p_yükseklik)
                    } else {
                        let p_x = şerit.x + şerit.genişlik * b;
                        let p_g = (şerit.genişlik * (e - b)).max(4.0);
                        Dikdörtgen::yeni(p_x, şerit.y, p_g, şerit.yükseklik)
                    };
                    yüzey.dikdörtgen(
                        pencere,
                        &Dolgu::Düz(crate::renk::Renk::kyma(
                            135.0 / 255.0,
                            175.0 / 255.0,
                            1.0,
                            0.2,
                        )),
                        [0.0; 4],
                        None,
                    );
                    // Çerçeve gölge ve filler katmanlarının üstündedir.
                    if dikey {
                        yüzey.dikdörtgen(
                            şerit,
                            &Dolgu::Düz(crate::renk::Renk::SAYDAM),
                            [0.0; 4],
                            Some((1.0, kenarlık)),
                        );
                    } else {
                        // SubPixelOptimize edilmiş Canvas çerçevesi yarım
                        // örtüyü iki komşu piksele dağıtır.
                        let üst = şerit.y + 0.5;
                        let alt = şerit.alt() - 0.5;
                        yüzey.çizgi(
                            (şerit.x, üst),
                            (şerit.sağ(), üst),
                            1.0,
                            kenarlık,
                            ÇizgiTürü::Düz,
                        );
                        yüzey.çizgi(
                            (şerit.x, alt),
                            (şerit.sağ(), alt),
                            1.0,
                            kenarlık,
                            ÇizgiTürü::Düz,
                        );
                        yüzey.çizgi(
                            (şerit.x + 0.5, şerit.y),
                            (şerit.x + 0.5, şerit.alt()),
                            1.0,
                            kenarlık,
                            ÇizgiTürü::Düz,
                        );
                        yüzey.çizgi(
                            (şerit.sağ() - 0.5, şerit.y),
                            (şerit.sağ() - 0.5, şerit.alt()),
                            1.0,
                            kenarlık,
                            ÇizgiTürü::Düz,
                        );
                    }
                    // Tutamaçlar.
                    let (sol, sağ) = if dikey {
                        // ECharts'ın öntanımlı handleIcon yolu 40 birimlik
                        // eksende 7.5 / 25.06 / 7.44 oranında sap–gövde–sap
                        // parçalarından oluşur. 30 px handleSize karşılıkları:
                        // 5.625 / 18.795 / 5.58 px.
                        (
                            Dikdörtgen::yeni(şerit.x + 5.625, pencere.alt() - 4.0, 18.795, 6.0),
                            Dikdörtgen::yeni(şerit.x + 5.625, pencere.y - 2.0, 18.795, 6.0),
                        )
                    } else {
                        // Varsayılan handle yolunda uçları pencereye doğru
                        // birer piksel kaydıran ECharts hizalama hilesi.
                        (
                            Dikdörtgen::yeni(pencere.x - 2.0, şerit.y + 5.625, 6.0, 18.795),
                            Dikdörtgen::yeni(pencere.sağ() - 4.0, şerit.y + 5.625, 6.0, 18.795),
                        )
                    };
                    for t in [sol, sağ] {
                        yüzey.dikdörtgen(
                            t,
                            &Dolgu::Düz(crate::renk::Renk::BEYAZ),
                            [1.5; 4],
                            Some((1.0, crate::renk::Renk::onaltılık(0xc0c9e6))),
                        );
                    }
                    if dikey {
                        for merkez_y in [sol.merkez().1, sağ.merkez().1] {
                            yüzey.çizgi(
                                (şerit.x, merkez_y),
                                (şerit.x + 5.625, merkez_y),
                                1.0,
                                crate::renk::Renk::onaltılık(0xc0c9e6),
                                ÇizgiTürü::Düz,
                            );
                            yüzey.çizgi(
                                (şerit.x + 24.42, merkez_y),
                                (şerit.sağ(), merkez_y),
                                1.0,
                                crate::renk::Renk::onaltılık(0xc0c9e6),
                                ÇizgiTürü::Düz,
                            );
                        }
                    } else {
                        for merkez_x in [sol.merkez().0, sağ.merkez().0] {
                            yüzey.çizgi(
                                (merkez_x, şerit.y),
                                (merkez_x, şerit.y + 5.625),
                                1.0,
                                crate::renk::Renk::onaltılık(0xc0c9e6),
                                ÇizgiTürü::Düz,
                            );
                            yüzey.çizgi(
                                (merkez_x, şerit.y + 24.42),
                                (merkez_x, şerit.alt()),
                                1.0,
                                crate::renk::Renk::onaltılık(0xc0c9e6),
                                ÇizgiTürü::Düz,
                            );
                        }
                    }
                    // Alt/sağ taşıma tutamacı (`brushSelect`).
                    let taşıma = if dikey {
                        Dikdörtgen::yeni(şerit.sağ(), pencere.y, 7.0, pencere.yükseklik)
                    } else {
                        Dikdörtgen::yeni(pencere.x, şerit.y - 6.5, pencere.genişlik, 7.0)
                    };
                    let taşıma_yarıçapı = if dikey {
                        [0.0, 2.0, 2.0, 0.0]
                    } else {
                        [2.0, 2.0, 0.0, 0.0]
                    };
                    yüzey.dikdörtgen(
                        taşıma,
                        &Dolgu::Düz(vurgu.opaklık(0.5)),
                        taşıma_yarıçapı,
                        None,
                    );
                    if !dikey {
                        let orta = taşıma.merkez();
                        for dx in [-2.0, 0.0, 2.0] {
                            yüzey.dikdörtgen(
                                Dikdörtgen::yeni(orta.0 + dx - 0.5, orta.1 - 1.5, 1.0, 3.0),
                                &Dolgu::Düz(crate::renk::Renk::BEYAZ),
                                [0.5; 4],
                                None,
                            );
                        }
                    } else {
                        let orta = taşıma.merkez();
                        for dy in [-2.0, 0.0, 2.0] {
                            yüzey.dikdörtgen(
                                Dikdörtgen::yeni(orta.0 - 1.5, orta.1 + dy - 0.5, 3.0, 1.0),
                                &Dolgu::Düz(crate::renk::Renk::BEYAZ),
                                [0.5; 4],
                                None,
                            );
                        }
                    }
                    çıktı.sürgüler.push(SürgüBölgesi {
                        yakınlaştırma_sırası: z,
                        şerit,
                        pencere,
                        sol_tutamaç: sol,
                        sağ_tutamaç: sağ,
                        dikey,
                    });
                }
            }
        }

        // Eksen imleci çizgisi + eksen ipucu penceresi. `bağlantılı`
        // (axisPointer.link) açıkken çizgi, aynı kategori sırasında TÜM
        // ızgaralarda çizilir.
        if let Some(eksen_ip) = eksen_ipucu
            && let Some(ipucu) = &ipucu_seçeneği
        {
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
                                && e.ölçek.kategorik_mi() == eksen_ip.kategorik
                        })
                    } else {
                        kurulum.y_eksenler.iter().find(|e| {
                            e.seçenek.ızgara_sırası == ızgara
                                && e.ölçek.kategorik_mi() == eksen_ip.kategorik
                        })
                    };
                    let Some(bant_ekseni) = bant_ekseni else {
                        continue;
                    };
                    // Yakınlaştırma penceresi dışındaysa çizme.
                    if !bant_ekseni.pencerede_mi(eksen_ip.eksen_değeri) {
                        continue;
                    }
                    let merkez = keskin(bant_ekseni.veriden_piksele(eksen_ip.eksen_değeri));
                    if eksen_ip.bant_x {
                        yüzey.çizgi(
                            (merkez, alan.alt()),
                            (merkez, alan.y),
                            1.0,
                            tema::nötr_30(),
                            ÇizgiTürü::Kesikli,
                        );
                    } else {
                        yüzey.çizgi(
                            (alan.x, merkez),
                            (alan.sağ(), merkez),
                            1.0,
                            tema::nötr_30(),
                            ÇizgiTürü::Kesikli,
                        );
                    }
                }

                // `showSymbol: false` çizgilerinde ECharts, eksen imlecinin
                // yakaladığı noktayı geçici vurgu sembolüyle görünür kılar.
                if !eksen_ip.kategorik {
                    for parametre in &eksen_ip.parametreler {
                        let Some(seri) = seçenekler.seriler.get(parametre.seri_sırası) else {
                            continue;
                        };
                        let Seri::Çizgi(çizgi) = seri else {
                            continue;
                        };
                        if çizgi.sembol_göster {
                            continue;
                        }
                        let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
                            continue;
                        };
                        let Some(x) = parametre.değer.x() else {
                            continue;
                        };
                        let Some(y) = parametre.değer.sayı() else {
                            continue;
                        };
                        let merkez = kartezyen.nokta(x, y);
                        yüzey.daire(
                            merkez,
                            4.0,
                            Some(&Dolgu::Düz(crate::renk::Renk::BEYAZ)),
                            Some((2.0, seçenekler.seri_rengi(parametre.seri_sırası))),
                        );
                    }
                }
            }
            if ipucu.içerik_göster
                && let Some(f) = fare
            {
                let (başlık, satırlar) = if let Some(biçimleyici) = &ipucu.bağlamlı_biçimleyici
                {
                    (
                        None,
                        vec![İpucuSatırı {
                            im_rengi: None,
                            ad: biçimleyici.uygula(&eksen_ip.parametreler),
                            değer: String::new(),
                        }],
                    )
                } else {
                    (Some(eksen_ip.başlık), eksen_ip.satırlar)
                };
                bekleyen_ipucu = Some((başlık, satırlar, f));
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
                Seri::Takvim(takvim) => Some(takvim_değer_kapsamı(takvim)),
                _ => None,
            })
            .unwrap_or([0.0, 1.0]);
        let eşleme_çıktısı = görsel_eşleme_çiz(yüzey, eşleme, eşleme.kapsam_çöz(veri_kapsamı));
        çıktı.eşleme_kutuları = eşleme_çıktısı.parça_kutuları;
        çıktı.sürekli_eşleme = eşleme_çıktısı.sürekli;
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
                zaman_sn,
                &mut çıktı.isabetler,
            );
        }
    }

    // 4d) Calendar ve matrix üzerindeki çekirdek (GL olmayan) lines.
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        let Seri::Hatlar(hatlar) = seri else {
            continue;
        };
        if !ad_görünür(seri.ad(), kapalı) {
            continue;
        }
        let mut yerel_isabetler = Vec::new();
        match hatlar.koordinat_sistemi {
            HatKoordinatSistemi::Takvim => {
                let Some(Some(yerleşim)) = takvim_yerleşimleri.get(hatlar.takvim_sırası) else {
                    continue;
                };
                let çiz = |yüzey: &mut dyn ÇizimYüzeyi, isabetler: &mut Vec<İsabetBölgesi>| {
                    hatlar_çiz(
                        yüzey,
                        hatlar,
                        i,
                        &|nokta| yerleşim.veriden_noktaya(nokta.x.sayı()?),
                        seçenekler.seri_rengi(i),
                        ilerleme,
                        zaman_sn,
                        isabetler,
                    );
                };
                if hatlar.kırp {
                    yüzey.kırpılı(yerleşim.gövde_kutusu, &mut |kırpılı| {
                        çiz(kırpılı, &mut yerel_isabetler);
                    });
                } else {
                    çiz(yüzey, &mut yerel_isabetler);
                }
            }
            HatKoordinatSistemi::Matris => {
                // Şimdiki kök model tek matrix taşır; sıfır dışındaki açık
                // indeks sessizce başka matrix'e düşürülmez.
                if hatlar.matris_sırası != 0 {
                    continue;
                }
                let Some(yerleşim) = &matris_yerleşimi else {
                    continue;
                };
                let çiz = |yüzey: &mut dyn ÇizimYüzeyi, isabetler: &mut Vec<İsabetBölgesi>| {
                    hatlar_çiz(
                        yüzey,
                        hatlar,
                        i,
                        &|nokta| {
                            yerleşim.veriden_noktaya(
                                matris_hat_aralığı(&nokta.x)?,
                                matris_hat_aralığı(&nokta.y)?,
                            )
                        },
                        seçenekler.seri_rengi(i),
                        ilerleme,
                        zaman_sn,
                        isabetler,
                    );
                };
                if hatlar.kırp {
                    yüzey.kırpılı(yerleşim.dış_kutu, &mut |kırpılı| {
                        çiz(kırpılı, &mut yerel_isabetler);
                    });
                } else {
                    çiz(yüzey, &mut yerel_isabetler);
                }
            }
            HatKoordinatSistemi::Kartezyen2B | HatKoordinatSistemi::Kutupsal => continue,
        }
        çıktı.isabetler.append(&mut yerel_isabetler);
    }

    // 5) Pasta serileri.
    let tüm_alan = Dikdörtgen::yeni(0.0, 0.0, yüzey.genişlik(), yüzey.yükseklik());
    let mut pasta_etiket_kutuları = Vec::new();
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        let Seri::Pasta(p) = seri else { continue };
        if !ad_görünür(seri.ad(), kapalı) {
            continue;
        }
        let dilimler: Vec<Dilim> = pasta_yerleşimi(p, seçenekler, tüm_alan, kapalı, ilerleme);
        if dilimler.is_empty() {
            boş_pasta_çiz(yüzey, p, tüm_alan);
        }

        // Öğe ipucu: fare hangi dilimde?
        let vurgu = match (&ipucu_seçeneği, fare) {
            (Some(ipucu), Some(f)) if ipucu.tetikleme != Tetikleme::Kapalı => {
                dilimler.iter().position(|d| d.içeriyor_mu(f))
            }
            _ => None,
        };

        pasta_çiz(yüzey, p, &dilimler, vurgu, &mut pasta_etiket_kutuları);

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
                    (Some(ipucu), Some(f)) if ipucu.tetikleme != Tetikleme::Kapalı => dilimler
                        .iter()
                        .position(|d| d.sınır_kutusu().içeriyor_mu(f)),
                    _ => None,
                };
                huni_çiz(yüzey, h, i, &dilimler, vurgu, &mut çıktı.isabetler);
                if let (Some(dilim), Some(f)) = (vurgu.and_then(|v| dilimler.get(v)), fare) {
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
                let Some(koordinat) = &seçenekler.radar else {
                    continue;
                };
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
                    && ipucu.tetikleme != Tetikleme::Kapalı
                {
                    let vurgu = çıktı
                        .isabetler
                        .iter()
                        .rev()
                        .find(|b| b.seri_sırası == i && b.geometri.içeriyor_mu(f))
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
            Seri::Saçılım(s) if s.takvim_sırası.is_some() => {
                // Pozitif zlevel, CalendarView'ın z=2 üst katmanını da aşar;
                // bu seriler aşağıdaki ikinci geçişte çizilir.
                if s.z_seviyesi > 0 || !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let takvim_sırası = s.takvim_sırası.unwrap_or(0);
                let Some(Some(yerleşim)) = takvim_yerleşimleri.get(takvim_sırası) else {
                    continue;
                };
                if let Some(ipucu) = takvim_saçılım_serisini_çiz(
                    yüzey,
                    s,
                    i,
                    yerleşim,
                    seçenekler.seri_rengi(i),
                    ilerleme,
                    zaman_sn,
                    ipucu_seçeneği.as_ref(),
                    fare,
                    &mut çıktı.isabetler,
                ) {
                    bekleyen_ipucu = Some(ipucu);
                }
            }
            Seri::Takvim(s) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let eşleme = seçenekler
                    .seri_görsel_eşlemesi(i)
                    .cloned()
                    .unwrap_or_default();
                let kapsam = eşleme.kapsam_çöz(takvim_değer_kapsamı(s));
                if let (Some(takvim), Some(Some(yerleşim))) = (
                    seçenekler.takvimler.get(s.takvim_sırası),
                    takvim_yerleşimleri.get(s.takvim_sırası),
                ) {
                    takvim_koordinatında_çiz(
                        yüzey,
                        s,
                        i,
                        takvim,
                        yerleşim,
                        &eşleme,
                        kapsam,
                        ilerleme,
                        &mut çıktı.isabetler,
                    );
                } else {
                    // Eski `TakvimSerisi::yeni(yıl)` bileşensiz kullanımını
                    // kaynak uyumluluğu için koru.
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
                }
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
                    bekleyen_ipucu = Some((seri.ad().map(str::to_string), vec![satır], f));
                }
            }
            Seri::TemaNehri(s) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                tema_nehri_çiz(
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
                    bekleyen_ipucu = Some((seri.ad().map(str::to_string), vec![satır], f));
                }
            }
            Seri::Kiriş(s) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let önce = çıktı.isabetler.len();
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                kiriş_çiz(
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
                    bekleyen_ipucu = Some((seri.ad().map(str::to_string), vec![satır], f));
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
                // Takvim bileşeninden daha yüksek z değerleri aşağıdaki üst
                // katman geçişinde çizilir.
                if (g.takvim_sırası.is_some() && g.z > 2) || !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let takvim = g
                    .takvim_sırası
                    .and_then(|sıra| takvim_yerleşimleri.get(sıra))
                    .and_then(Option::as_ref);
                if g.takvim_sırası.is_some() && takvim.is_none() {
                    continue;
                }
                if let Some(ipucu) = grafo_serisini_çiz(
                    yüzey,
                    g,
                    i,
                    tüm_alan,
                    seçenekler,
                    ilerleme,
                    girdi.grafo_görünümü,
                    &girdi.grafo_kaymaları,
                    takvim,
                    ipucu_seçeneği.as_ref(),
                    fare,
                    &mut çıktı.isabetler,
                ) {
                    bekleyen_ipucu = Some(ipucu);
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
                    bekleyen_ipucu = Some((seri.ad().map(str::to_string), vec![satır], f));
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
                        değer: b.değer.map(binlik_ayır).unwrap_or_default(),
                    };
                    bekleyen_ipucu = Some((seri.ad().map(str::to_string), vec![satır], f));
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

    // CalendarView ayırıcı ve metinleri z2=20/30 ile seri şekillerinin
    // üstünde tutar. Gün zemini ise serilerden önce çizilmişti.
    for (takvim, yerleşim) in seçenekler.takvimler.iter().zip(&takvim_yerleşimleri) {
        if let Some(yerleşim) = yerleşim {
            takvim_üst_katmanı_çiz(yüzey, takvim, yerleşim);
        }
    }

    // CalendarView z=2'den yüksek takvim graph serileri ayırıcıların
    // üstündedir (`calendar-graph` resmî örneğinde z=20).
    for (seri_sırası, seri) in seçenekler.seriler.iter().enumerate() {
        let Seri::Grafo(grafo) = seri else {
            continue;
        };
        if grafo.z <= 2 || !ad_görünür(seri.ad(), kapalı) {
            continue;
        }
        let Some(takvim_sırası) = grafo.takvim_sırası else {
            continue;
        };
        let Some(Some(yerleşim)) = takvim_yerleşimleri.get(takvim_sırası) else {
            continue;
        };
        if let Some(ipucu) = grafo_serisini_çiz(
            yüzey,
            grafo,
            seri_sırası,
            tüm_alan,
            seçenekler,
            ilerleme,
            girdi.grafo_görünümü,
            &girdi.grafo_kaymaları,
            Some(yerleşim),
            ipucu_seçeneği.as_ref(),
            fare,
            &mut çıktı.isabetler,
        ) {
            bekleyen_ipucu = Some(ipucu);
        }
    }

    // Ayrı zlevel tuvali kullanan takvim scatter/effectScatter serileri,
    // CalendarView'ın ayırıcı ve etiketlerinden sonra boyanır. Resmî
    // `calendar-effectscatter` örneğindeki zlevel=1 bunun görünür kanıtıdır.
    for (seri_sırası, seri) in seçenekler.seriler.iter().enumerate() {
        let Seri::Saçılım(saçılım) = seri else {
            continue;
        };
        if saçılım.z_seviyesi <= 0
            || saçılım.takvim_sırası.is_none()
            || !ad_görünür(seri.ad(), kapalı)
        {
            continue;
        }
        let takvim_sırası = saçılım.takvim_sırası.unwrap_or(0);
        let Some(Some(yerleşim)) = takvim_yerleşimleri.get(takvim_sırası) else {
            continue;
        };
        if let Some(ipucu) = takvim_saçılım_serisini_çiz(
            yüzey,
            saçılım,
            seri_sırası,
            yerleşim,
            seçenekler.seri_rengi(seri_sırası),
            ilerleme,
            zaman_sn,
            ipucu_seçeneği.as_ref(),
            fare,
            &mut çıktı.isabetler,
        ) {
            bekleyen_ipucu = Some(ipucu);
        }
    }

    // 5b) Legend (z=4): dataZoom şeridi üstünde kalmalıdır.
    if let Some(g) = &seçenekler.gösterge {
        let gösterge_çıktısı = gösterge_çiz(yüzey, g, &gösterge_öğeleri, girdi.gösterge_sayfası);
        çıktı.gösterge_kutuları = gösterge_çıktısı.kutular;
        çıktı.gösterge_okları = gösterge_çıktısı.oklar;
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

#[cfg(test)]
mod yakınlaştırma_yönü_testleri {
    use super::*;

    #[test]
    fn sayısal_boundary_gap_veri_açıklığını_yüzdeyle_genişletir() {
        let ölçek = ölçek_kur(
            &Eksen::değer().sayısal_kenar_boşluğu(0.0, "30%"),
            Vec::new(),
            [200.0, 750.0],
        );
        assert_eq!(ölçek.kapsam(), [0.0, 1000.0]);

        let sabit_üst = ölçek_kur(
            &Eksen::değer()
                .sayısal_kenar_boşluğu(0.0, "30%")
                .en_çok(800.0),
            Vec::new(),
            [200.0, 750.0],
        );
        assert_eq!(sabit_üst.kapsam(), [0.0, 800.0]);
    }

    #[test]
    fn iç_yakınlaştırma_odağı_yatay_ve_dikey_veri_yönünü_izler() {
        let alan = Dikdörtgen::yeni(10.0, 20.0, 100.0, 200.0);
        let yatay = İçYakınlaştırmaAlanı {
            yakınlaştırma_sırası: 0,
            alan,
            dikey: false,
        };
        assert!((yatay.odak_oranı((35.0, 70.0)) - 0.25).abs() < 1e-6);
        assert_eq!(yatay.eksen_uzunluğu(), 100.0);

        let dikey = İçYakınlaştırmaAlanı {
            yakınlaştırma_sırası: 1,
            alan,
            dikey: true,
        };
        assert!((dikey.odak_oranı((35.0, 70.0)) - 0.75).abs() < 1e-6);
        assert_eq!(dikey.eksen_uzunluğu(), 200.0);
        assert!(dikey.eksen_konumu((0.0, 60.0)) > dikey.eksen_konumu((0.0, 70.0)));
    }

    #[test]
    fn dikey_sürgü_ekseni_ekranda_yukarı_doğru_artar() {
        let şerit = Dikdörtgen::yeni(10.0, 20.0, 30.0, 200.0);
        let sürgü = SürgüBölgesi {
            yakınlaştırma_sırası: 0,
            şerit,
            pencere: şerit,
            sol_tutamaç: şerit,
            sağ_tutamaç: şerit,
            dikey: true,
        };
        assert_eq!(sürgü.eksen_uzunluğu(), 200.0);
        assert!(sürgü.eksen_konumu((20.0, 40.0)) > sürgü.eksen_konumu((20.0, 80.0)));
    }

    #[test]
    fn zaman_ekseni_ipucu_en_yakin_noktayi_baglamli_bicimlendirir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .ipucu(
                İpucu::yeni()
                    .tetikleme(Tetikleme::Eksen)
                    .imleç_animasyonu(false)
                    .bağlamlı_biçimleyici(|parametreler| {
                        let Some(parametre) = parametreler.first() else {
                            return String::new();
                        };
                        format!(
                            "{}:{}",
                            parametre.veri_sırası,
                            parametre.değer.sayı().unwrap_or_default()
                        )
                    }),
            )
            .x_ekseni(Eksen::zaman())
            .y_ekseni(Eksen::değer())
            .seri(
                crate::model::seri::ÇizgiSerisi::yeni()
                    .sembol_göster(false)
                    .veri([
                        crate::model::deger::VeriÖğesi::yeni([0.0, 10.0]),
                        crate::model::deger::VeriÖğesi::yeni([86_400_000.0, 20.0]),
                    ]),
            );
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);

        grafiği_boya(
            &mut yüzey,
            &seçenekler,
            &BoyamaGirdisi {
                fare: Some((629.0, 250.0)),
                ..BoyamaGirdisi::default()
            },
        );

        let döküm = yüzey.döküm();
        assert!(döküm.contains("yazı \"1:20\""), "{döküm}");
        assert!(
            döküm.contains("Y(4.0 bS 626.0,65.0)"),
            "showSymbol=false imleç vurgu sembolü eksik: {döküm}"
        );
    }

    #[test]
    fn çizgi_uç_etiketleri_aynı_eksende_örtüşmeden_aşağı_itilir() {
        let birinci = EksenBağı::default();
        let ikinci = EksenBağı { x: 1, y: 1 };
        let sonuç = çizgi_uç_etiketlerini_dikey_kaydır(
            &[
                (0, birinci, 10.0, 12.0),
                (1, birinci, 15.0, 12.0),
                (2, birinci, 40.0, 12.0),
                (3, ikinci, 11.0, 12.0),
            ],
            4,
        );
        assert_eq!(sonuç, vec![Some(10.0), Some(22.0), Some(40.0), Some(11.0)]);
    }

    #[test]
    fn dikey_araç_kutusu_resmi_sınır_kutusu_aralıklarıyla_ortalanır() {
        let seçenekler = GrafikSeçenekleri::yeni().araç_kutusu(
            crate::model::bilesen::AraçKutusu::yeni()
                .yön(Yön::Dikey)
                .sol("right")
                .üst("center")
                .veri_görünümü(true)
                .sihirli_tür(true, true)
                .sihirli_yığın(true)
                .geri_yükle(true)
                .png_kaydet(true),
        );
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);
        let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());
        let türler: Vec<AraçTürü> = çıktı.araç_düğmeleri.iter().map(|(_, tür)| *tür).collect();
        assert_eq!(
            türler,
            vec![
                AraçTürü::VeriGörünümü,
                AraçTürü::SihirliÇizgi,
                AraçTürü::SihirliSütun,
                AraçTürü::SihirliYığın,
                AraçTürü::GeriYükle,
                AraçTürü::PngKaydet,
            ]
        );
        let merkezler: Vec<(f32, f32)> = çıktı
            .araç_düğmeleri
            .iter()
            .map(|(kutu, _)| (kutu.x + kutu.genişlik / 2.0, kutu.y + kutu.yükseklik / 2.0))
            .collect();
        let beklenen_y = [
            188.116_87, 218.073_09, 248.029_31, 277.456_2, 306.883_15, 336.883_15,
        ];
        for ((x, y), beklenen_y) in merkezler.iter().zip(beklenen_y) {
            assert!((*x - 675.0).abs() < 1e-3);
            assert!((*y - beklenen_y).abs() < 1e-3);
        }
    }

    #[test]
    fn araç_kutusu_sağ_uzaklığını_iç_boşluktan_sonra_uygular() {
        let seçenekler = GrafikSeçenekleri::yeni().araç_kutusu(
            crate::model::bilesen::AraçKutusu::yeni()
                .sağ(10)
                .veri_yakınlaştırma(true)
                .geri_yükle(true)
                .png_kaydet(true),
        );
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);
        let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());

        assert_eq!(çıktı.araç_düğmeleri.len(), 4);
        let ilk = çıktı.araç_düğmeleri.first().map(|(kutu, _)| kutu);
        let son = çıktı.araç_düğmeleri.last().map(|(kutu, _)| kutu);
        assert!(ilk.is_some_and(|kutu| {
            (kutu.x + kutu.genişlik / 2.0 - 580.222_4).abs() < 1e-3
                && (kutu.y + kutu.yükseklik / 2.0 - 25.0).abs() < 1e-3
        }));
        // 15 px bileşen iç boşluğu + açık 10 px `right` uzaklığı.
        assert!(son.is_some_and(|kutu| (kutu.sağ() - 675.0).abs() < 1e-3));
    }

    #[test]
    fn çoklu_pasta_göstergesi_yinelenen_dilim_adlarını_tekilleştirir() {
        let pasta = || {
            crate::model::seri::PastaSerisi::yeni().veri([
                crate::model::deger::VeriÖğesi::adlı("Kek", 10),
                crate::model::deger::VeriÖğesi::adlı("Tahıl", 20),
            ])
        };
        let seçenekler = GrafikSeçenekleri::yeni()
            .gösterge(crate::model::bilesen::Gösterge::yeni())
            .seri(pasta())
            .seri(pasta());

        let öğeler = gösterge_öğeleri(&seçenekler, &HashSet::new());
        assert_eq!(
            öğeler.iter().map(|öğe| öğe.ad.as_str()).collect::<Vec<_>>(),
            ["Kek", "Tahıl"]
        );
    }
}
