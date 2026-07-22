//! Grafik görünümü — ECharts örneğinin (`echarts.init` + `setOption`)
//! gpui karşılığı.
//!
//! Boyama hattının tamamı [`grafiği_boya`] içinde, çizim yüzeyinden bağımsız
//! saf bir işlev olarak durur: gpui penceresi de altın (golden) testlerdeki
//! [`crate::cizim::KayıtYüzeyi`] de aynı hattı çalıştırır. gpui'ye özgü
//! yapıştırma (tuval, fare, animasyon karesi, olay yayını) yalnızca
//! [`crate::cizim::pencere::GrafikGörünümü`]dedir.

use std::collections::{HashMap, HashSet};

use crate::bilesen::baslik::{başlık_çiz, başlık_çiz_alanda};
use crate::bilesen::eksen_cizimi::{
    bölme_çizgilerini_çiz, eksenleri_çiz, eksenleri_çiz_katman,
    kategori_taban_çizgilerini_üstte_çiz, kırılma_alanlarını_çiz,
};
use crate::bilesen::gosterge::{GöstergeÖğesi, gösterge_çiz};
use crate::bilesen::grafik::{GrafikSahnesi, grafik_sahnesi_hazırla};
use crate::bilesen::ipucu::{ipucu_çiz, İpucuSatırı};
use crate::bilesen::matris_cizimi::matris_çiz;
use crate::bilesen::takvim_cizimi::{takvim_arka_planı_çiz, takvim_üst_katmanı_çiz};
use crate::bilesen::zaman_seridi::{
    ZamanŞeridiEylemi, seçenekli_zaman_şeridi_çiz, zaman_şeridi_çiz,
};
use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::yuzey::{keskin, ÇizimYüzeyi};
use crate::cizim::{AfinMatris, Yol, yolu_dönüştür};
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
    SürekliGörselEşlemeBölgesi, görsel_eşleme_çiz, matris_ısı_haritası_çiz, ısı_değer_kapsamı,
    ısı_haritası_çiz,
};
use crate::grafik::kiris::kiriş_çiz;
use crate::grafik::kutupsal::{
    KutupsalDüzen, kutupsal_ağ_çiz, kutupsal_kur, kutupsal_serileri_çiz,
};
use crate::grafik::mum::{kutu_çiz, mum_çiz};
use crate::grafik::paralel::paralel_çiz;
use crate::grafik::pasta::{
    Dilim, boş_pasta_çiz_merkezle, dilim_değer_metni, pasta_yerleşimi_merkezle, pasta_çiz,
};
use crate::grafik::radar::{
    radar_ağı_çiz, radar_düzeni_serilerle, radar_görsel_kapsamı, radar_ipucu_satırları,
    radar_serisi_çiz, radar_vurgusu,
};
use crate::grafik::sacilim::{
    BüyükSaçılımNoktaları, SaçılımNoktası, büyük_saçılım_noktaları, büyük_saçılım_çiz,
    matris_saçılım_noktaları, saçılım_görsel_kapsamı, saçılım_nokta_boyutlarını_eşle,
    saçılım_noktaları, saçılım_xy, saçılım_çiz_çoklu_eşlemeli, takvim_saçılım_noktaları,
    tek_eksen_saçılım_noktaları,
};
use crate::grafik::sankey::sankey_çiz;
use crate::grafik::sutun::{
    SütunGirdisi, sütun_bant_genişliği, sütun_değeri, sütun_görsel_kapsamı, sütun_taban_değeri,
    sütun_yatay_mı, sütunları_çiz, yerleşim_hesapla,
};
use crate::grafik::takvim_isi::{takvim_değer_kapsamı, takvim_koordinatında_çiz, takvim_çiz};
use crate::grafik::tema_nehri::{
    tema_nehri_katman_adları, tema_nehri_katman_dolgusu, tema_nehri_çiz,
};
use crate::koordinat::{
    Dikdörtgen, Kartezyen2B, TakvimYerleşimi, TekEksenYerleşimi, ÇalışmaEkseni,
};
use crate::model::bilesen::{
    AraçKutusuÖzelliği, FırçaAracıTürü, FırçaBağı, FırçaKoordinatAralığı, FırçaKoordinatı,
    FırçaSeçimAlanı, FırçaStili, FırçaTürü, GöstergeSimgesi, Tetikleme, Yön, İmleçTürü, İpucu,
    İpucuParametresi,
};
use crate::model::eksen::{Eksen, EksenKonumu, EksenTürü};
use crate::model::hatlar::{HatKoordinatSistemi, HatKoordinatı, HatNoktası};
use crate::model::matris::{MatrisAralığı, MatrisKonumu};
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{EksenBağı, GrafoSerisi, SaçılımSerisi, Sembol, Seri, ÖzelBağlam};
use crate::model::stil::ÇizgiTürü;
use crate::model::yakinlastirma::{YakınlaştırmaSüzmeKipi, YakınlaştırmaTürü};
use crate::model::{DikeyKonum, YatayKonum};
use crate::olcek::{
    AralıkÖlçeği, KategorikÖlçek, KırılmaEşleyici, LogÖlçeği, ZamanÖlçeği, Ölçek
};
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yerlesim::yigin::{
    YığınAralığı, yığın_aralıkları, yığın_aralıkları_seçici
};

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

fn fırça_aracı_svg_yolu(tür: FırçaAracıTürü) -> &'static str {
    match tür {
        FırçaAracıTürü::Dikdörtgen => {
            "M7.3,34.7 M0.4,10V-0.2h9.8 M89.6,10V-0.2h-9.8 M0.4,60v10.2h9.8 M89.6,60v10.2h-9.8 M12.3,22.4V10.5h13.1 M33.6,10.5h7.8 M49.1,10.5h7.8 M77.5,22.4V10.5h-13 M12.3,31.1v8.2 M77.7,31.1v8.2 M12.3,47.6v11.9h13.1 M33.6,59.5h7.6 M49.1,59.5 h7.7 M77.5,47.6v11.9h-13"
        }
        FırçaAracıTürü::Çokgen => {
            "M55.2,34.9c1.7,0,3.1,1.4,3.1,3.1s-1.4,3.1-3.1,3.1 s-3.1-1.4-3.1-3.1S53.5,34.9,55.2,34.9z M50.4,51c1.7,0,3.1,1.4,3.1,3.1c0,1.7-1.4,3.1-3.1,3.1c-1.7,0-3.1-1.4-3.1-3.1 C47.3,52.4,48.7,51,50.4,51z M55.6,37.1l1.5-7.8 M60.1,13.5l1.6-8.7l-7.8,4 M59,19l-1,5.3 M24,16.1l6.4,4.9l6.4-3.3 M48.5,11.6 l-5.9,3.1 M19.1,12.8L9.7,5.1l1.1,7.7 M13.4,29.8l1,7.3l6.6,1.6 M11.6,18.4l1,6.1 M32.8,41.9 M26.6,40.4 M27.3,40.2l6.1,1.6 M49.9,52.1l-5.6-7.6l-4.9-1.2"
        }
        FırçaAracıTürü::Yatay => {
            "M15.2,30 M19.7,15.6V1.9H29 M34.8,1.9H40.4 M55.3,15.6V1.9H45.9 M19.7,44.4V58.1H29 M34.8,58.1H40.4 M55.3,44.4 V58.1H45.9 M12.5,20.3l-9.4,9.6l9.6,9.8 M3.1,29.9h16.5 M62.5,20.3l9.4,9.6L62.3,39.7 M71.9,29.9H55.4"
        }
        FırçaAracıTürü::Dikey => {
            "M38.8,7.7 M52.7,12h13.2v9 M65.9,26.6V32 M52.7,46.3h13.2v-9 M24.9,12H11.8v9 M11.8,26.6V32 M24.9,46.3H11.8v-9 M48.2,5.1l-9.3-9l-9.4,9.2 M38.9-3.9V12 M48.2,53.3l-9.3,9l-9.4-9.2 M38.9,62.3V46.4"
        }
        FırçaAracıTürü::Koru => {
            "M4,10.5V1h10.3 M20.7,1h6.1 M33,1h6.1 M55.4,10.5V1H45.2 M4,17.3v6.6 M55.6,17.3v6.6 M4,30.5V40h10.3 M20.7,40 h6.1 M33,40h6.1 M55.4,30.5V40H45.2 M21,18.9h62.9v48.6H21V18.9z"
        }
        FırçaAracıTürü::Temizle => {
            "M22,14.7l30.9,31 M52.9,14.7L22,45.7 M4.7,16.8V4.2h13.1 M26,4.2h7.8 M41.6,4.2h7.8 M70.3,16.8V4.2H57.2 M4.7,25.9v8.6 M70.3,25.9v8.6 M4.7,43.2v12.6h13.1 M26,55.8h7.8 M41.6,55.8h7.8 M70.3,43.2v12.6H57.2"
        }
    }
}

fn fırça_aracı_kaynak_yolu(tür: FırçaAracıTürü) -> Option<Yol> {
    Yol::svg_path_data(fırça_aracı_svg_yolu(tür)).ok()
}

fn fırça_aracı_boyutu(tür: FırçaAracıTürü) -> (f32, f32) {
    let Some(kutu) = fırça_aracı_kaynak_yolu(tür).and_then(|yol| yol.kesin_sınır_kutusu())
    else {
        return (20.0, 20.0);
    };
    let ölçek = 15.0 / kutu.genişlik.max(kutu.yükseklik).max(1.0);
    (kutu.genişlik * ölçek + 5.0, kutu.yükseklik * ölçek + 5.0)
}

fn fırça_aracı_yolu(tür: FırçaAracıTürü, merkez: (f32, f32)) -> Option<Yol> {
    let yol = fırça_aracı_kaynak_yolu(tür)?;
    let kutu = yol.kesin_sınır_kutusu()?;
    let ölçek = 15.0 / kutu.genişlik.max(kutu.yükseklik).max(1.0);
    let kaynak_merkez = (kutu.x + kutu.genişlik / 2.0, kutu.y + kutu.yükseklik / 2.0);
    let dönüşüm = AfinMatris::ötele(merkez.0, merkez.1)
        .çarp(AfinMatris::ölçekle(ölçek, ölçek))
        .çarp(AfinMatris::ötele(-kaynak_merkez.0, -kaynak_merkez.1));
    Some(yolu_dönüştür(&yol, dönüşüm))
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

/// Çalışma zamanında çizilen bir ECharts `brush` örtüsü. Koordinatlar
/// boyama girdisinde yüzey-yerel, gpui görünüm durumunda pencere-mutlaktır.
#[derive(Clone, Debug, PartialEq)]
pub enum FırçaAlanı {
    /// Serbest iki köşeli `rect` alanı.
    Dikdörtgen {
        başlangıç: (f32, f32),
        bitiş: (f32, f32),
    },
    /// Fare iziyle kurulan `polygon` alanı.
    Çokgen { noktalar: Vec<(f32, f32)> },
    /// `lineX`: X boyunca seçilen, ızgaranın tüm yüksekliğini kaplayan alan.
    Yatay {
        başlangıç: f32,
        bitiş: f32,
        üst: f32,
        alt: f32,
    },
    /// `lineY`: Y boyunca seçilen, ızgaranın tüm genişliğini kaplayan alan.
    Dikey {
        başlangıç: f32,
        bitiş: f32,
        sol: f32,
        sağ: f32,
    },
}

impl FırçaAlanı {
    /// Alanı yüzey/pencere kökenleri arasında taşır.
    pub fn kaydır(&self, dx: f32, dy: f32) -> Self {
        match self {
            FırçaAlanı::Dikdörtgen {
                başlangıç, bitiş
            } => FırçaAlanı::Dikdörtgen {
                başlangıç: (başlangıç.0 + dx, başlangıç.1 + dy),
                bitiş: (bitiş.0 + dx, bitiş.1 + dy),
            },
            FırçaAlanı::Çokgen { noktalar } => FırçaAlanı::Çokgen {
                noktalar: noktalar.iter().map(|(x, y)| (x + dx, y + dy)).collect(),
            },
            FırçaAlanı::Yatay {
                başlangıç,
                bitiş,
                üst,
                alt,
            } => FırçaAlanı::Yatay {
                başlangıç: başlangıç + dx,
                bitiş: bitiş + dx,
                üst: üst + dy,
                alt: alt + dy,
            },
            FırçaAlanı::Dikey {
                başlangıç,
                bitiş,
                sol,
                sağ,
            } => FırçaAlanı::Dikey {
                başlangıç: başlangıç + dy,
                bitiş: bitiş + dy,
                sol: sol + dx,
                sağ: sağ + dx,
            },
        }
    }

    /// Veri öğesinin temsilî merkezinin alanın içinde olup olmadığını sınar.
    pub fn içeriyor_mu(&self, nokta: (f32, f32)) -> bool {
        match self {
            FırçaAlanı::Çokgen { noktalar } => çokgen_noktayı_içeriyor(noktalar, nokta),
            _ => self
                .sınırlar()
                .is_some_and(|sınırlar| sınırlar.içeriyor_mu(nokta)),
        }
    }

    /// Sürüklemenin görünür/seçilebilir bir alana dönüştüğünü bildirir.
    pub fn geçerli_mi(&self) -> bool {
        match self {
            FırçaAlanı::Dikdörtgen { .. } => self
                .sınırlar()
                .is_some_and(|d| d.genişlik > 3.0 && d.yükseklik > 3.0),
            FırçaAlanı::Çokgen { noktalar } => {
                noktalar.len() >= 3
                    && self
                        .sınırlar()
                        .is_some_and(|d| d.genişlik > 3.0 && d.yükseklik > 3.0)
            }
            FırçaAlanı::Yatay { .. } => self.sınırlar().is_some_and(|d| d.genişlik > 3.0),
            FırçaAlanı::Dikey { .. } => self.sınırlar().is_some_and(|d| d.yükseklik > 3.0),
        }
    }

    fn sınırlar(&self) -> Option<Dikdörtgen> {
        let (x0, y0, x1, y1) = match self {
            FırçaAlanı::Dikdörtgen {
                başlangıç, bitiş
            } => (başlangıç.0, başlangıç.1, bitiş.0, bitiş.1),
            FırçaAlanı::Çokgen { noktalar } => {
                let ilk = *noktalar.first()?;
                let mut x0 = ilk.0;
                let mut y0 = ilk.1;
                let mut x1 = ilk.0;
                let mut y1 = ilk.1;
                for &(x, y) in noktalar.iter().skip(1) {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
                return Some(Dikdörtgen::yeni(x0, y0, x1 - x0, y1 - y0));
            }
            FırçaAlanı::Yatay {
                başlangıç,
                bitiş,
                üst,
                alt,
            } => (*başlangıç, *üst, *bitiş, *alt),
            FırçaAlanı::Dikey {
                başlangıç,
                bitiş,
                sol,
                sağ,
            } => (*sol, *başlangıç, *sağ, *bitiş),
        };
        Some(Dikdörtgen::yeni(
            x0.min(x1),
            y0.min(y1),
            (x1 - x0).abs(),
            (y1 - y0).abs(),
        ))
    }

    fn yol(&self) -> Option<Yol> {
        let mut yol = Yol::yeni();
        match self {
            FırçaAlanı::Çokgen { noktalar } => {
                let ilk = *noktalar.first()?;
                yol.taşı(ilk);
                for &nokta in noktalar.iter().skip(1) {
                    yol.çiz(nokta);
                }
            }
            _ => {
                let d = self.sınırlar()?;
                yol.taşı((d.x, d.y));
                yol.çiz((d.sağ(), d.y));
                yol.çiz((d.sağ(), d.alt()));
                yol.çiz((d.x, d.alt()));
            }
        }
        yol.kapat();
        Some(yol)
    }
}

fn çokgen_noktayı_içeriyor(noktalar: &[(f32, f32)], nokta: (f32, f32)) -> bool {
    if noktalar.len() < 3 {
        return false;
    }
    let mut içeride = false;
    let mut önceki = *noktalar.last().expect("uzunluk denetlendi");
    for &şimdiki in noktalar {
        // Kenardaki nokta da seçilmiş sayılır.
        let vx = şimdiki.0 - önceki.0;
        let vy = şimdiki.1 - önceki.1;
        let wx = nokta.0 - önceki.0;
        let wy = nokta.1 - önceki.1;
        let çapraz = vx * wy - vy * wx;
        let izdüşüm = wx * vx + wy * vy;
        let uzunluk_kare = vx * vx + vy * vy;
        if çapraz.abs() <= 1e-3 && izdüşüm >= -1e-3 && izdüşüm <= uzunluk_kare + 1e-3 {
            return true;
        }
        if (şimdiki.1 > nokta.1) != (önceki.1 > nokta.1) {
            let kesişim_x =
                (önceki.0 - şimdiki.0) * (nokta.1 - şimdiki.1) / (önceki.1 - şimdiki.1) + şimdiki.0;
            if nokta.0 < kesişim_x {
                içeride = !içeride;
            }
        }
        önceki = şimdiki;
    }
    içeride
}

fn fırça_alanını_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    alan: &FırçaAlanı,
    stil: Option<&FırçaStili>,
) {
    if !alan.geçerli_mi() {
        return;
    }
    let Some(yol) = alan.yol() else { return };
    let varsayılan = FırçaStili::default();
    let stil = stil.unwrap_or(&varsayılan);
    let dolgu = Dolgu::Düz(stil.renk);
    match alan {
        FırçaAlanı::Çokgen { .. } => yüzey.yol_doldur(&yol, &dolgu),
        _ => {
            if let Some(d) = alan.sınırlar() {
                yüzey.dikdörtgen(d, &dolgu, [0.0; 4], None);
            }
        }
    }
    if stil.kenarlık_kalınlığı > 0.0 {
        yüzey.yol_çiz(
            &yol,
            stil.kenarlık_kalınlığı,
            stil.kenarlık_rengi,
            ÇizgiTürü::Düz,
        );
    }
}

#[derive(Default)]
struct HazırFırça {
    alanlar: Vec<FırçaAlanı>,
    /// Seri başına ham veri sırasıyla aynı uzunlukta alfa çarpanları.
    öğe_opaklıkları: Vec<Option<Vec<f32>>>,
    /// Seri başına ham veri sırasıyla aynı uzunlukta sabit renk görselleri.
    öğe_renkleri: Vec<Option<Vec<Option<Dolgu>>>>,
    /// `brushSelected.batch[0].selected[*].dataIndex` karşılığı. Dış dizi
    /// genel seri sırasını, iç diziler ham veri sıralarını korur.
    seçili_ham_sıralar: Vec<Vec<usize>>,
}

struct HazırFırçaAlanı {
    piksel: FırçaAlanı,
    x_ekseni_sırası: Option<usize>,
    y_ekseni_sırası: Option<usize>,
}

fn fırça_koordinatını_çöz(
    koordinat: &FırçaKoordinatı, eksen: &ÇalışmaEkseni
) -> Option<f64> {
    match koordinat {
        FırçaKoordinatı::Sayı(değer) if değer.is_finite() => Some(*değer),
        FırçaKoordinatı::Sayı(_) => None,
        FırçaKoordinatı::Kategori(kategori) => eksen.ölçek.kategori_sırası(kategori),
    }
}

fn fırça_alanını_hazırla(
    alan: &FırçaSeçimAlanı,
    kurulum: &KartezyenKurulum,
) -> Option<HazırFırçaAlanı> {
    let x_sırası = alan.x_ekseni_sırası.or_else(|| {
        alan.y_ekseni_sırası.and_then(|y_sırası| {
            let ızgara = kurulum.y_eksenler.get(y_sırası)?.seçenek.ızgara_sırası;
            kurulum
                .x_eksenler
                .iter()
                .position(|eksen| eksen.seçenek.ızgara_sırası == ızgara)
        })
    });
    let y_sırası = alan.y_ekseni_sırası.or_else(|| {
        x_sırası.and_then(|x_sırası| {
            let ızgara = kurulum.x_eksenler.get(x_sırası)?.seçenek.ızgara_sırası;
            kurulum
                .y_eksenler
                .iter()
                .position(|eksen| eksen.seçenek.ızgara_sırası == ızgara)
        })
    });
    let x_sırası = x_sırası.or((!kurulum.x_eksenler.is_empty()).then_some(0));
    let y_sırası = y_sırası.or((!kurulum.y_eksenler.is_empty()).then_some(0));
    let x = kurulum.x_eksenler.get(x_sırası?)?;
    let y = kurulum.y_eksenler.get(y_sırası?)?;
    if x.seçenek.ızgara_sırası != y.seçenek.ızgara_sırası {
        return None;
    }
    let ızgara = kurulum.ızgara_alanları.get(x.seçenek.ızgara_sırası)?;
    let piksel = match (&alan.tür, &alan.koordinat_aralığı) {
        (FırçaTürü::Yatay, FırçaKoordinatAralığı::Eksen([baş, son])) => {
            FırçaAlanı::Yatay {
                başlangıç: x.veriden_piksele(fırça_koordinatını_çöz(baş, x)?),
                bitiş: x.veriden_piksele(fırça_koordinatını_çöz(son, x)?),
                üst: ızgara.y,
                alt: ızgara.alt(),
            }
        }
        (FırçaTürü::Dikey, FırçaKoordinatAralığı::Eksen([baş, son])) => {
            FırçaAlanı::Dikey {
                başlangıç: y.veriden_piksele(fırça_koordinatını_çöz(baş, y)?),
                bitiş: y.veriden_piksele(fırça_koordinatını_çöz(son, y)?),
                sol: ızgara.x,
                sağ: ızgara.sağ(),
            }
        }
        (
            FırçaTürü::Dikdörtgen,
            FırçaKoordinatAralığı::Dikdörtgen {
                x: [x0, x1],
                y: [y0, y1],
            },
        ) => FırçaAlanı::Dikdörtgen {
            başlangıç: (
                x.veriden_piksele(fırça_koordinatını_çöz(x0, x)?),
                y.veriden_piksele(fırça_koordinatını_çöz(y0, y)?),
            ),
            bitiş: (
                x.veriden_piksele(fırça_koordinatını_çöz(x1, x)?),
                y.veriden_piksele(fırça_koordinatını_çöz(y1, y)?),
            ),
        },
        (FırçaTürü::Çokgen, FırçaKoordinatAralığı::Çokgen(noktalar)) => {
            FırçaAlanı::Çokgen {
                noktalar: noktalar
                    .iter()
                    .map(|[x_değeri, y_değeri]| {
                        Some((
                            x.veriden_piksele(fırça_koordinatını_çöz(x_değeri, x)?),
                            y.veriden_piksele(fırça_koordinatını_çöz(y_değeri, y)?),
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?,
            }
        }
        _ => return None,
    };
    Some(HazırFırçaAlanı {
        piksel,
        x_ekseni_sırası: alan.x_ekseni_sırası.or(Some(x_sırası?)),
        y_ekseni_sırası: alan.y_ekseni_sırası,
    })
}

fn seri_fırça_noktası(
    seri: &Seri,
    veri_sırası: usize,
    kartezyen: &Kartezyen2B,
    sütun_merkez_kayması: f32,
) -> Option<(f32, f32)> {
    let öğe = seri.veri().get(veri_sırası)?;
    let x_kategorik = kartezyen.x.ölçek.kategorik_mi();
    let y_kategorik = kartezyen.y.ölçek.kategorik_mi();
    let (x, y) = match seri {
        Seri::Saçılım(_) => saçılım_xy(&öğe.değer, veri_sırası)?,
        Seri::Mum(_) => {
            let değerler = öğe.değer.dizi()?;
            let kapanış = *değerler.get(1)?;
            if y_kategorik && !x_kategorik {
                (kapanış, veri_sırası as f64)
            } else {
                (veri_sırası as f64, kapanış)
            }
        }
        Seri::Kutu(_) => {
            let değerler = öğe.değer.dizi()?;
            let ortanca = *değerler.get(2)?;
            if y_kategorik && !x_kategorik {
                (ortanca, veri_sırası as f64)
            } else {
                (veri_sırası as f64, ortanca)
            }
        }
        Seri::Sütun(_) => {
            let yatay = sütun_yatay_mı(kartezyen);
            let değer = sütun_değeri(öğe, yatay)?;
            let taban = sütun_taban_değeri(öğe, veri_sırası, yatay);
            if yatay {
                (değer, taban)
            } else {
                (taban, değer)
            }
        }
        _ => {
            let değer = öğe.değer.sayı()?;
            if y_kategorik && !x_kategorik {
                (değer, veri_sırası as f64)
            } else {
                (öğe.değer.x().unwrap_or(veri_sırası as f64), değer)
            }
        }
    };
    let mut nokta = kartezyen.nokta(x, y);
    if matches!(seri, Seri::Sütun(_)) {
        if sütun_yatay_mı(kartezyen) {
            nokta.1 += sütun_merkez_kayması;
        } else {
            nokta.0 += sütun_merkez_kayması;
        }
    }
    Some(nokta)
}

/// Model aşamasındaki ECharts `Cartesian2D.getBaseAxis` seçimi. Çalışma
/// eksenleri kurulmadan önce yığın ve kapsam boyutunu aynı kuralla seçer.
fn sütun_tabanı_y_mi(x: &Eksen, y: &Eksen) -> bool {
    if x.tür == EksenTürü::Kategori {
        false
    } else if y.tür == EksenTürü::Kategori {
        true
    } else if x.tür == EksenTürü::Zaman {
        false
    } else {
        y.tür == EksenTürü::Zaman
    }
}

fn sütun_grup_anahtarı(seri: &Seri, kurulum: &KartezyenKurulum) -> (bool, usize) {
    let bağ = seri.eksen_bağı();
    let yatay = kurulum
        .seri_kartezyeni(seri)
        .is_some_and(|kartezyen| sütun_yatay_mı(&kartezyen));
    if yatay {
        (false, bağ.y)
    } else {
        (true, bağ.x)
    }
}

/// Bar brushSelector veri noktasını kategori merkezinde değil, gerçek sütun
/// dikdörtgeninin merkezinde sınar. Yığınlar aynı kaymayı paylaşır; yan yana
/// yığınlar kategori bandının iki tarafına ayrılır.
fn sütun_fırça_merkez_kaymaları(
    seçenekler: &GrafikSeçenekleri,
    kurulum: &KartezyenKurulum,
) -> HashMap<usize, f32> {
    let mut gruplar: Vec<((bool, usize), Vec<SütunGirdisi>)> = Vec::new();
    for (seri_sırası, seri) in seçenekler.seriler.iter().enumerate() {
        if !kurulum
            .görünürler
            .get(seri_sırası)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        let Seri::Sütun(sütun) = seri else {
            continue;
        };
        let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
            continue;
        };
        let girdi = SütunGirdisi {
            seri: sütun,
            kartezyen,
            genel_sıra: seri_sırası,
            aralıklar: kurulum
                .aralıklar
                .get(seri_sırası)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            renk: seçenekler.seri_rengi(seri_sırası),
            görsel_eşlemeler: Vec::new(),
            öğe_opaklıkları: None,
            öğe_renkleri: None,
        };
        let anahtar = sütun_grup_anahtarı(seri, kurulum);
        match gruplar.iter_mut().find(|(aday, _)| *aday == anahtar) {
            Some((_, grup)) => grup.push(girdi),
            None => gruplar.push((anahtar, vec![girdi])),
        }
    }

    let mut kaymalar = HashMap::new();
    for (_, grup) in gruplar {
        let bant = sütun_bant_genişliği(&grup);
        for (girdi, konum) in grup.iter().zip(yerleşim_hesapla(&grup, bant)) {
            kaymalar.insert(girdi.genel_sıra, konum.kaydırma + konum.genişlik / 2.0);
        }
    }
    kaymalar
}

fn fırçayı_hazırla(
    seçenekler: &GrafikSeçenekleri,
    kurulum: &KartezyenKurulum,
) -> HazırFırça {
    let Some(fırça) = seçenekler.fırça.as_ref() else {
        return HazırFırça::default();
    };
    let alanlar = fırça
        .alanlar
        .iter()
        .filter_map(|alan| fırça_alanını_hazırla(alan, kurulum))
        .collect::<Vec<_>>();
    let sütun_merkez_kaymaları = sütun_fırça_merkez_kaymaları(seçenekler, kurulum);
    let mut doğrudan = seçenekler
        .seriler
        .iter()
        .map(|seri| vec![false; seri.veri().len()])
        .collect::<Vec<_>>();
    let mut denetlendi = vec![false; seçenekler.seriler.len()];
    for alan in &alanlar {
        for (seri_sırası, seri) in seçenekler.seriler.iter().enumerate() {
            if !seri.kartezyen_mi()
                || (!fırça.seri_sıraları.is_empty() && !fırça.seri_sıraları.contains(&seri_sırası))
            {
                continue;
            }
            let bağ = seri.eksen_bağı();
            if alan
                .x_ekseni_sırası
                .is_some_and(|x_sırası| bağ.x != x_sırası)
                || alan
                    .y_ekseni_sırası
                    .is_some_and(|y_sırası| bağ.y != y_sırası)
            {
                continue;
            }
            let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
                continue;
            };
            if let Some(denetlendi) = denetlendi.get_mut(seri_sırası) {
                *denetlendi = true;
            }
            let Some(seçimler) = doğrudan.get_mut(seri_sırası) else {
                continue;
            };
            for (veri_sırası, seçili) in seçimler.iter_mut().enumerate() {
                if seri_fırça_noktası(
                    seri,
                    veri_sırası,
                    &kartezyen,
                    sütun_merkez_kaymaları
                        .get(&seri_sırası)
                        .copied()
                        .unwrap_or(0.0),
                )
                .is_some_and(|nokta| alan.piksel.içeriyor_mu(nokta))
                {
                    *seçili = true;
                }
            }
        }
    }

    let bağlı_mı = |seri_sırası: usize| match &fırça.bağlantı {
        FırçaBağı::Yok => false,
        FırçaBağı::Tümü => true,
        FırçaBağı::Seriler(sıralar) => sıralar.contains(&seri_sırası),
    };
    let mut bağlı_seçimler = Vec::new();
    for (seri_sırası, seçimler) in doğrudan.iter().enumerate() {
        if bağlı_mı(seri_sırası) && denetlendi.get(seri_sırası) == Some(&true) {
            if bağlı_seçimler.len() < seçimler.len() {
                bağlı_seçimler.resize(seçimler.len(), false);
            }
            for (hedef, seçili) in bağlı_seçimler.iter_mut().zip(seçimler) {
                *hedef |= *seçili;
            }
        }
    }
    let son_seçimler = doğrudan
        .into_iter()
        .enumerate()
        .map(|(seri_sırası, seçimler)| {
            if bağlı_mı(seri_sırası) && !alanlar.is_empty() {
                Some(
                    (0..seçimler.len())
                        .map(|sıra| bağlı_seçimler.get(sıra).copied().unwrap_or(false))
                        .collect::<Vec<_>>(),
                )
            } else if denetlendi.get(seri_sırası) == Some(&true) {
                Some(seçimler)
            } else {
                None
            }
        })
        .collect::<Vec<Option<Vec<bool>>>>();
    let seçili_ham_sıralar = son_seçimler
        .iter()
        .map(|seçimler| {
            seçimler
                .as_ref()
                .map(|seçimler| {
                    seçimler
                        .iter()
                        .enumerate()
                        .filter_map(|(sıra, seçili)| seçili.then_some(sıra))
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    let opaklık_görseli_var = fırça.iç_renk_opaklığı.is_some() || fırça.dış_renk_opaklığı.is_some();
    let öğe_opaklıkları = son_seçimler
        .iter()
        .map(|seçimler| {
            let seçimler = seçimler.as_ref()?;
            opaklık_görseli_var.then(|| {
                seçimler
                    .iter()
                    .map(|seçili| {
                        if *seçili {
                            fırça.iç_renk_opaklığı.unwrap_or(1.0)
                        } else {
                            fırça.dış_renk_opaklığı.unwrap_or(1.0)
                        }
                    })
                    .collect()
            })
        })
        .collect();
    // BrushModel, açık bir `outOfBrush` nesnesi yoksa disabled rengini
    // enjekte eder. Yalnız colorAlpha verilmişse nesne zaten vardır ve bu
    // varsayılan renk uygulanmaz.
    let varsayılan_dış_renk = (fırça.dış_renk.is_none() && fırça.dış_renk_opaklığı.is_none())
        .then(|| Dolgu::Düz(Renk::onaltılık(0xcfd2d7)));
    let dış_renk = fırça.dış_renk.clone().or(varsayılan_dış_renk);
    let renk_görseli_var = fırça.iç_renk.is_some() || dış_renk.is_some();
    let öğe_renkleri = son_seçimler
        .iter()
        .map(|seçimler| {
            let seçimler = seçimler.as_ref()?;
            renk_görseli_var.then(|| {
                seçimler
                    .iter()
                    .map(|seçili| {
                        if *seçili {
                            fırça.iç_renk.clone()
                        } else {
                            dış_renk.clone()
                        }
                    })
                    .collect()
            })
        })
        .collect();
    HazırFırça {
        alanlar: alanlar.into_iter().map(|alan| alan.piksel).collect(),
        öğe_opaklıkları,
        öğe_renkleri,
        seçili_ham_sıralar,
    }
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
    /// Tamamlanmış ve sürmekte olan rect/polygon/lineX/lineY alanları.
    /// `fırça`, eski tek-dikdörtgen girdisini korur; yeni etkileşim hattı bu
    /// listeyi kullanır.
    pub fırça_alanları: Vec<FırçaAlanı>,
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
            fırça_alanları: Vec::new(),
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
    /// Programatik brush alanlarının ham veri seçimi. Dış sıra
    /// `seriesIndex`, iç değerler `dataIndex` karşılığıdır.
    pub fırça_seçimleri: Vec<Vec<usize>>,
    /// Kartezyen ızgara alanları; lineX/lineY brush örtülerini sınırlar.
    pub ızgara_alanları: Vec<Dikdörtgen>,
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
    /// `graphic` bileşeninin dönüşümlü isabet sınamasında da kullanılan
    /// gerçek sahnesi.
    pub grafik_sahnesi: Option<GrafikSahnesi>,
}

/// Araç kutusu düğme türleri.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AraçTürü {
    VeriGörünümü,
    /// `toolbox.feature.dataZoom.zoom`.
    VeriYakınlaştır,
    /// `toolbox.feature.dataZoom.back`.
    VeriYakınlaştırmayıGeriAl,
    /// `toolbox.feature.brush` alt araçlarından biri.
    Fırça(FırçaAracıTürü),
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

/// Polar seri isabetini ortak item-tooltip modeline çevirir. Görsel z
/// sırasıyla eklenen bölgeler tersten tarandığı için üstteki bar/saçılım
/// öğesi, ECharts gibi alttaki örtüşen öğenin önüne geçer.
fn kutupsal_ipucu_hazırla(
    seçenekler: &GrafikSeçenekleri,
    düzen: &KutupsalDüzen,
    ipucu: &İpucu,
    fare: Option<(f32, f32)>,
    programatik: Option<(usize, usize)>,
    isabetler: &[İsabetBölgesi],
) -> Option<Bekleyenİpucu> {
    if ipucu.tetikleme != Tetikleme::Öğe || !ipucu.içerik_göster {
        return None;
    }
    let isabet = programatik
        .and_then(|(seri_sırası, veri_sırası)| {
            isabetler.iter().rev().find(|bölge| {
                bölge.seri_sırası == seri_sırası && bölge.veri_sırası == veri_sırası
            })
        })
        .or_else(|| {
            let fare = fare?;
            isabetler
                .iter()
                .rev()
                .find(|bölge| bölge.geometri.içeriyor_mu(fare))
        })?;
    let seri = seçenekler.seriler.get(isabet.seri_sırası)?;
    let öğe = seri.veri().get(isabet.veri_sırası)?;
    let seri_adı = seri.ad().unwrap_or("").to_string();
    let veri_adı = öğe.ad.clone().unwrap_or_else(|| {
        if düzen.açısal_kategorik {
            düzen.açısal_ölçek.etiket(isabet.veri_sırası as f64)
        } else if düzen.radyal_kategorik {
            düzen.radyal_ölçek.etiket(isabet.veri_sırası as f64)
        } else {
            isabet.veri_sırası.to_string()
        }
    });
    let parametre = İpucuParametresi {
        seri_sırası: isabet.seri_sırası,
        seri_adı: seri_adı.clone(),
        veri_sırası: isabet.veri_sırası,
        ad: veri_adı.clone(),
        değer: öğe.değer.clone(),
        boyutlar: öğe.boyutlar.clone(),
    };
    let konum = fare.unwrap_or_else(|| isabet.geometri.merkez());
    if let Some(biçimleyici) = &ipucu.bağlamlı_biçimleyici {
        let metin = biçimleyici.uygula(&[parametre]);
        let metin = metin.replace("<br />", "<br>").replace("<br/>", "<br>");
        let satırlar = metin
            .split("<br>")
            .map(|satır| İpucuSatırı {
                im_rengi: None,
                ad: satır.to_string(),
                değer: String::new(),
            })
            .collect::<Vec<_>>();
        return Some((None, satırlar, konum));
    }

    let değer = isabet.değer?;
    let değer = ipucu
        .değer_biçimleyici
        .as_ref()
        .map(|biçimleyici| biçimleyici.uygula(değer, &binlik_ayır(değer)))
        .unwrap_or_else(|| binlik_ayır(değer));
    Some((
        (!seri_adı.is_empty()).then_some(seri_adı),
        vec![İpucuSatırı {
            im_rengi: Some(seçenekler.seri_rengi(isabet.seri_sırası)),
            ad: veri_adı,
            değer,
        }],
        konum,
    ))
}

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
    palet: &[Renk],
    görsel_eşlemeler: &[&crate::model::gorsel_esleme::GörselEşleme],
    ilerleme: f32,
    zaman_sn: f32,
    ipucu_seçeneği: Option<&İpucu>,
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> Option<Bekleyenİpucu> {
    let eşlemeler = görsel_eşlemeler
        .iter()
        .map(|eşleme| (*eşleme, saçılım_görsel_kapsamı(seri, eşleme)))
        .collect::<Vec<_>>();
    let mut noktalar = takvim_saçılım_noktaları(seri, yerleşim);
    saçılım_nokta_boyutlarını_eşle(seri, &mut noktalar, &eşlemeler);
    let vurgu = match (seri.sessiz, ipucu_seçeneği, fare) {
        (true, _, _) => None,
        (false, Some(ipucu), Some(f)) if ipucu.tetikleme != Tetikleme::Kapalı => noktalar
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
    saçılım_çiz_çoklu_eşlemeli(
        yüzey,
        seri,
        &noktalar,
        seri_rengi,
        ilerleme,
        zaman_sn,
        vurgu,
        &eşlemeler,
        palet,
    );
    if !seri.sessiz {
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
fn matris_saçılım_serisini_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seri: &SaçılımSerisi,
    seri_sırası: usize,
    yerleşim: &crate::koordinat::MatrisYerleşimi,
    seri_rengi: Renk,
    palet: &[Renk],
    görsel_eşlemeler: &[&crate::model::gorsel_esleme::GörselEşleme],
    ilerleme: f32,
    zaman_sn: f32,
    ipucu_seçeneği: Option<&İpucu>,
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> Option<Bekleyenİpucu> {
    let eşlemeler = görsel_eşlemeler
        .iter()
        .map(|eşleme| (*eşleme, saçılım_görsel_kapsamı(seri, eşleme)))
        .collect::<Vec<_>>();
    let mut noktalar = matris_saçılım_noktaları(seri, yerleşim);
    saçılım_nokta_boyutlarını_eşle(seri, &mut noktalar, &eşlemeler);
    let vurgu = match (seri.sessiz, ipucu_seçeneği, fare) {
        (true, _, _) => None,
        (false, Some(ipucu), Some(f)) if ipucu.tetikleme != Tetikleme::Kapalı => noktalar
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
    saçılım_çiz_çoklu_eşlemeli(
        yüzey,
        seri,
        &noktalar,
        seri_rengi,
        ilerleme,
        zaman_sn,
        vurgu,
        &eşlemeler,
        palet,
    );
    if !seri.sessiz {
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

/// `singleAxis` bileşenlerini ve bunlara bağlı scatter/effectScatter
/// serilerini ECharts katman sırasıyla boyar: bölme çizgileri, eksen, seri.
/// Başlıklar daha yüksek `z` değerinde olduğundan çağıran bu katmanı başlık
/// bileşeninden önce geçirir.
fn tek_eksen_yerleşimlerini_kur(
    seçenekler: &GrafikSeçenekleri,
    kapalı: &HashSet<String>,
    tuval: (f32, f32),
) -> Vec<TekEksenYerleşimi> {
    let mut yerleşimler = Vec::with_capacity(seçenekler.tek_eksenler.len());
    for (tek_sırası, tek) in seçenekler.tek_eksenler.iter().enumerate() {
        let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
        let mut en_büyük_kategori = None::<usize>;
        for seri in &seçenekler.seriler {
            match seri {
                Seri::Saçılım(saçılım)
                    if saçılım.tek_eksen_sırası == Some(tek_sırası)
                        && ad_görünür(seri.ad(), kapalı) =>
                {
                    for (veri_sırası, öğe) in saçılım.veri.iter().enumerate() {
                        let Some((değer, _)) = saçılım_xy(&öğe.değer, veri_sırası) else {
                            continue;
                        };
                        if değer.is_finite() {
                            kapsam[0] = kapsam[0].min(değer);
                            kapsam[1] = kapsam[1].max(değer);
                            if tek.eksen.tür == EksenTürü::Kategori
                                && değer >= 0.0
                                && değer.fract().abs() <= 1e-9
                            {
                                en_büyük_kategori =
                                    Some(en_büyük_kategori.unwrap_or_default().max(değer as usize));
                            }
                        }
                    }
                }
                Seri::TemaNehri(nehir)
                    if nehir.tek_eksen_sırası == tek_sırası && ad_görünür(seri.ad(), kapalı) =>
                {
                    for (değer, _, katman) in &nehir.veri {
                        if !değer.is_finite() || kapalı.contains(katman) {
                            continue;
                        }
                        kapsam[0] = kapsam[0].min(*değer);
                        kapsam[1] = kapsam[1].max(*değer);
                        if tek.eksen.tür == EksenTürü::Kategori
                            && *değer >= 0.0
                            && değer.fract().abs() <= 1e-9
                        {
                            en_büyük_kategori =
                                Some(en_büyük_kategori.unwrap_or_default().max(*değer as usize));
                        }
                    }
                }
                _ => {}
            }
        }
        if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
            kapsam = [0.0, 1.0];
        }
        let kategoriler = if tek.eksen.tür == EksenTürü::Kategori && tek.eksen.veri.is_empty() {
            en_büyük_kategori
                .map(|son| (0..=son).map(|sıra| sıra.to_string()).collect())
                .unwrap_or_default()
        } else {
            tek.eksen.veri.clone()
        };
        let ölçek = ölçek_kur(&tek.eksen, kategoriler, kapsam);
        let mut çizim_modeli = tek.clone();
        let bölme_rengi = çizim_modeli
            .eksen
            .bölme_çizgisi
            .renk
            .unwrap_or_else(tema::bölme_çizgisi)
            .opaklık(çizim_modeli.bölme_çizgisi_opaklığı);
        çizim_modeli.eksen.bölme_çizgisi.renk = Some(bölme_rengi);
        yerleşimler.push(TekEksenYerleşimi::kur(&çizim_modeli, tuval, ölçek));
    }
    yerleşimler
}

#[allow(clippy::too_many_arguments)]
fn tek_eksenleri_çiz(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenekler: &GrafikSeçenekleri,
    yerleşimler: &[TekEksenYerleşimi],
    kapalı: &HashSet<String>,
    ilerleme: f32,
    zaman_sn: f32,
    ipucu_seçeneği: Option<&İpucu>,
    fare: Option<(f32, f32)>,
    programatik_ipucu: Option<(usize, usize)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> Option<Bekleyenİpucu> {
    if yerleşimler.is_empty() {
        return None;
    }

    // Bütün bileşen çizgileri seri simgelerinin altında kalır.
    for yerleşim in yerleşimler {
        let eksenler = [&yerleşim.eksen];
        bölme_çizgilerini_çiz(yüzey, yerleşim.alan, &eksenler);
        eksenleri_çiz(yüzey, yerleşim.alan, &eksenler);
    }

    let mut bekleyen = None;
    for (seri_sırası, seri) in seçenekler.seriler.iter().enumerate() {
        let Seri::Saçılım(saçılım) = seri else {
            continue;
        };
        let Some(tek_sırası) = saçılım.tek_eksen_sırası else {
            continue;
        };
        if !ad_görünür(seri.ad(), kapalı) {
            continue;
        }
        let Some(yerleşim) = yerleşimler.get(tek_sırası) else {
            continue;
        };
        let görsel_eşlemeler = seçenekler
            .seri_görsel_eşlemeleri(seri_sırası)
            .map(|eşleme| (eşleme, saçılım_görsel_kapsamı(saçılım, eşleme)))
            .collect::<Vec<_>>();
        let mut noktalar = tek_eksen_saçılım_noktaları(saçılım, yerleşim);
        saçılım_nokta_boyutlarını_eşle(saçılım, &mut noktalar, &görsel_eşlemeler);
        let fare_vurgusu = match (saçılım.sessiz, ipucu_seçeneği, fare) {
            (false, Some(ipucu), Some(f))
                if ipucu.tetikleme != Tetikleme::Kapalı
                    && seçenekler
                        .tek_eksenler
                        .get(tek_sırası)
                        .is_some_and(|tek| tek.ipucu_göster) =>
            {
                noktalar
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
                    .map(|nokta| nokta.sıra)
            }
            _ => None,
        };
        let vurgu = fare_vurgusu.or_else(|| {
            programatik_ipucu
                .filter(|(hedef_seri, _)| *hedef_seri == seri_sırası)
                .map(|(_, veri_sırası)| veri_sırası)
        });
        saçılım_çiz_çoklu_eşlemeli(
            yüzey,
            saçılım,
            &noktalar,
            seçenekler.seri_rengi(seri_sırası),
            ilerleme,
            zaman_sn,
            vurgu,
            &görsel_eşlemeler,
            &seçenekler.palet,
        );
        if !saçılım.sessiz {
            for nokta in &noktalar {
                isabetler.push(İsabetBölgesi {
                    seri_sırası,
                    veri_sırası: nokta.sıra,
                    seri_adı: saçılım.ad.clone(),
                    ad: saçılım.veri.get(nokta.sıra).and_then(|öğe| öğe.ad.clone()),
                    değer: Some(nokta.y_değeri),
                    geometri: İsabetGeometrisi::Daire {
                        merkez: nokta.konum,
                        yarıçap: (nokta.boyut / 2.0 + 3.0).max(8.0),
                    },
                });
            }
        }
        let Some(veri_sırası) = vurgu else {
            continue;
        };
        let Some(nokta) = noktalar.iter().find(|nokta| nokta.sıra == veri_sırası) else {
            continue;
        };
        let konum = fare.unwrap_or(nokta.konum);
        bekleyen = Some((
            saçılım.ad.clone(),
            vec![İpucuSatırı {
                im_rengi: Some(seçenekler.seri_rengi(seri_sırası)),
                ad: binlik_ayır(nokta.x_değeri),
                değer: binlik_ayır(nokta.y_değeri),
            }],
            konum,
        ));
    }
    bekleyen
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
    matris: Option<&crate::koordinat::MatrisYerleşimi>,
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
        matris,
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
                let palet_başlangıcı =
                    crate::grafik::radar::radar_palet_başlangıcı(seçenekler, i);
                let mut adlı_öğe_var = false;
                for (j, öğe) in r.veri.iter().enumerate() {
                    let Some(ad) = öğe.ad.clone() else { continue };
                    adlı_öğe_var = true;
                    if !süzgeç.is_empty() && !süzgeç.contains(&ad) {
                        continue;
                    }
                    let öğe_stili = öğe
                        .stil
                        .as_ref()
                        .or_else(|| {
                            r.veri_ayarları
                                .get(j)
                                .and_then(|ayar| ayar.öğe_stili.as_ref())
                        })
                        .unwrap_or(&r.öğe_stili);
                    öğeler.push(GöstergeÖğesi {
                        kapalı: kapalı.contains(&ad),
                        ad,
                        renk: crate::grafik::radar::radar_öğe_rengi(
                            seçenekler,
                            i,
                            j,
                            palet_başlangıcı + j,
                        ),
                        opaklık: öğe_stili.opaklık.unwrap_or(1.0),
                        simge: GöstergeSimgesi::YuvarlakKöşeliKare,
                        çizgi_kalınlığı: None,
                        çizgi_sembolü: None,
                        kenarlık: None,
                        kapalı_simge_gizli: false,
                    });
                }
                // Veri öğeleri adsızsa LegendVisualProvider seri adına
                // düşer (radar-aqi gibi çok satırlı tek-seri kümeleri).
                if !adlı_öğe_var
                    && let Some(ad) = r.ad.clone()
                    && (süzgeç.is_empty() || süzgeç.contains(&ad))
                {
                    let renk = if r.veri.is_empty() {
                        seçenekler.palet_rengi(palet_başlangıcı)
                    } else {
                        crate::grafik::radar::radar_öğe_rengi(seçenekler, i, 0, palet_başlangıcı)
                    };
                    öğeler.push(GöstergeÖğesi {
                        kapalı: kapalı.contains(&ad),
                        ad,
                        renk,
                        opaklık: r.öğe_stili.opaklık.unwrap_or(1.0),
                        simge: GöstergeSimgesi::YuvarlakKöşeliKare,
                        çizgi_kalınlığı: None,
                        çizgi_sembolü: None,
                        kenarlık: None,
                        kapalı_simge_gizli: r.sembol == Sembol::Yok,
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
                        opaklık: 1.0,
                        simge: GöstergeSimgesi::YuvarlakKöşeliKare,
                        çizgi_kalınlığı: None,
                        çizgi_sembolü: None,
                        kenarlık: None,
                        kapalı_simge_gizli: false,
                    });
                }
            }
            Seri::TemaNehri(nehir) => {
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                for (katman_sırası, ad) in tema_nehri_katman_adları(nehir).into_iter().enumerate()
                {
                    if !süzgeç.is_empty() && !süzgeç.contains(&ad) {
                        continue;
                    }
                    öğeler.push(GöstergeÖğesi {
                        kapalı: kapalı.contains(&ad),
                        ad,
                        renk: tema_nehri_katman_dolgusu(nehir, katman_sırası, &palet).temsilî(),
                        opaklık: nehir.öğe_stili.opaklık.unwrap_or(1.0),
                        simge: GöstergeSimgesi::YuvarlakKöşeliKare,
                        çizgi_kalınlığı: None,
                        çizgi_sembolü: None,
                        kenarlık: nehir
                            .öğe_stili
                            .kenarlık_rengi
                            .filter(|_| nehir.öğe_stili.kenarlık_kalınlığı > 0.0)
                            .map(|renk| (nehir.öğe_stili.kenarlık_kalınlığı, renk)),
                        kapalı_simge_gizli: false,
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
                        .unwrap_or_else(|| {
                            seçenekler.palet_rengi(crate::grafik::pasta::pasta_palet_sırası(
                                seçenekler,
                                p,
                                j,
                                &ad,
                            ))
                        });
                    öğeler.push(GöstergeÖğesi {
                        kapalı: kapalı.contains(&ad),
                        ad,
                        renk,
                        opaklık: 1.0,
                        simge: GöstergeSimgesi::YuvarlakKöşeliKare,
                        çizgi_kalınlığı: None,
                        çizgi_sembolü: None,
                        kenarlık: p
                            .öğe_stili
                            .kenarlık_rengi
                            .filter(|_| p.öğe_stili.kenarlık_kalınlığı > 0.0)
                            .map(|renk| (p.öğe_stili.kenarlık_kalınlığı, renk)),
                        kapalı_simge_gizli: false,
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
                let çizgi_sembolü = match seri {
                    Seri::Çizgi(çizgi) => Some(çizgi.sembol.clone()),
                    _ => None,
                };
                let opaklık = match seri {
                    Seri::Saçılım(saçılım) => saçılım
                        .öğe_stili
                        .opaklık
                        .unwrap_or(if saçılım.efektli { 1.0 } else { 0.8 }),
                    Seri::Sütun(sütun) => sütun.öğe_stili.opaklık.unwrap_or(1.0),
                    _ => 1.0,
                };
                let kenarlık = match seri {
                    Seri::Mum(mum) if mum.kenarlık_kalınlığı > 0.0 => {
                        Some((mum.kenarlık_kalınlığı, mum.yükselen_kenarlık_rengi))
                    }
                    Seri::Sütun(sütun) if sütun.öğe_stili.kenarlık_kalınlığı > 0.0 => {
                        sütun
                            .öğe_stili
                            .kenarlık_rengi
                            .map(|renk| (sütun.öğe_stili.kenarlık_kalınlığı, renk))
                    }
                    _ => None,
                };
                öğeler.push(GöstergeÖğesi {
                    kapalı: kapalı.contains(&ad),
                    ad,
                    renk: seçenekler.seri_rengi(i),
                    opaklık,
                    simge,
                    çizgi_kalınlığı,
                    çizgi_sembolü,
                    kenarlık,
                    kapalı_simge_gizli: false,
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
            if let Some(ea) = en_az {
                kapsam[0] = ea;
            }
            if let Some(eç) = en_çok {
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
            en_az,
            en_çok,
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

#[derive(Default)]
struct MatrisKategoriToplayıcı {
    adlar: Vec<String>,
    en_büyük_sıra: Option<usize>,
}

impl MatrisKategoriToplayıcı {
    fn ad_ekle(&mut self, ad: &str) {
        if !ad.is_empty() && !self.adlar.iter().any(|aday| aday == ad) {
            self.adlar.push(ad.to_owned());
        }
    }

    fn sıra_ekle(&mut self, sıra: f64) {
        if sıra.is_finite() && sıra >= 0.0 {
            let sıra = sıra.round() as usize;
            self.en_büyük_sıra = Some(self.en_büyük_sıra.map_or(sıra, |önceki| önceki.max(sıra)));
        }
    }

    fn aralık_ekle(&mut self, aralık: &MatrisAralığı) {
        let mut konum_ekle = |konum: &MatrisKonumu| match konum {
            MatrisKonumu::Sıra(sıra) if *sıra >= 0 => self.sıra_ekle(*sıra as f64),
            MatrisKonumu::Değer(ad) => self.ad_ekle(ad),
            MatrisKonumu::Sıra(_) => {}
        };
        match aralık {
            MatrisAralığı::Tek(konum) => konum_ekle(konum),
            MatrisAralığı::Aralık(baş, son) => {
                konum_ekle(baş);
                konum_ekle(son);
            }
            MatrisAralığı::Tümü => {}
        }
    }

    fn bitir(mut self) -> Vec<String> {
        if self.adlar.is_empty()
            && let Some(en_büyük) = self.en_büyük_sıra
        {
            self.adlar = (0..=en_büyük).map(|sıra| sıra.to_string()).collect();
        }
        self.adlar
    }
}

fn matris_seri_kategorilerini_topla(
    seçenekler: &GrafikSeçenekleri,
    matris_sırası: usize,
) -> (Vec<String>, Vec<String>) {
    let mut x = MatrisKategoriToplayıcı::default();
    let mut y = MatrisKategoriToplayıcı::default();
    for seri in &seçenekler.seriler {
        match seri {
            Seri::Isı(ısı) if ısı.matris_sırası == Some(matris_sırası) => {
                for (sıra, öğe) in ısı.veri.iter().enumerate() {
                    if let Some(Some((mx, my))) = ısı.matris_koordinatları.get(sıra) {
                        x.aralık_ekle(mx);
                        y.aralık_ekle(my);
                    } else if let Some(dizi) = öğe.değer.dizi() {
                        if let Some(değer) = dizi.first() {
                            x.sıra_ekle(*değer);
                        }
                        if let Some(değer) = dizi.get(1) {
                            y.sıra_ekle(*değer);
                        }
                    }
                }
            }
            Seri::Saçılım(saçılım) if saçılım.matris_sırası == Some(matris_sırası) => {
                for (sıra, öğe) in saçılım.veri.iter().enumerate() {
                    if let Some(Some((mx, my))) = saçılım.matris_koordinatları.get(sıra) {
                        x.aralık_ekle(mx);
                        y.aralık_ekle(my);
                    } else if let Some(dizi) = öğe.değer.dizi() {
                        if let Some(değer) = dizi.first() {
                            x.sıra_ekle(*değer);
                        }
                        if let Some(değer) = dizi.get(1) {
                            y.sıra_ekle(*değer);
                        }
                    }
                }
            }
            Seri::Grafo(grafo) if grafo.matris_sırası == Some(matris_sırası) => {
                for düğüm in &grafo.düğümler {
                    if let Some((mx, my)) = &düğüm.matris_koordinatı {
                        x.aralık_ekle(mx);
                        y.aralık_ekle(my);
                    }
                }
            }
            Seri::Pasta(pasta) if pasta.matris_sırası == Some(matris_sırası) => {
                if let Some((mx, my)) = &pasta.matris_merkezi {
                    x.aralık_ekle(mx);
                    y.aralık_ekle(my);
                }
            }
            Seri::Özel(özel) if özel.matris_sırası == Some(matris_sırası) => {
                for ad in &özel.matris_x_kategorileri {
                    x.ad_ekle(ad);
                }
                for ad in &özel.matris_y_kategorileri {
                    y.ad_ekle(ad);
                }
            }
            Seri::Hatlar(hatlar)
                if hatlar.koordinat_sistemi == HatKoordinatSistemi::Matris
                    && hatlar.matris_sırası == matris_sırası =>
            {
                for hat in &hatlar.veri {
                    for nokta in &hat.koordinatlar {
                        match &nokta.x {
                            HatKoordinatı::Metin(ad) => x.ad_ekle(ad),
                            HatKoordinatı::Sayı(değer) => x.sıra_ekle(*değer),
                            HatKoordinatı::Zaman(değer) => x.sıra_ekle(*değer as f64),
                        }
                        match &nokta.y {
                            HatKoordinatı::Metin(ad) => y.ad_ekle(ad),
                            HatKoordinatı::Sayı(değer) => y.sıra_ekle(*değer),
                            HatKoordinatı::Zaman(değer) => y.sıra_ekle(*değer as f64),
                        }
                    }
                }
            }
            _ => {}
        }
    }
    (x.bitir(), y.bitir())
}

/// Kartezyen koordinat sistemlerini kurar: her eksen için kapsam/ölçek,
/// her ızgara için alan.
#[cfg(test)]
fn kartezyen_kur(
    yüzey: &dyn ÇizimYüzeyi,
    seçenekler: &GrafikSeçenekleri,
    kapalı: &HashSet<String>,
) -> Option<KartezyenKurulum> {
    kartezyen_kur_matrisli(yüzey, seçenekler, kapalı, &[])
}

fn kartezyen_kur_matrisli(
    yüzey: &dyn ÇizimYüzeyi,
    seçenekler: &GrafikSeçenekleri,
    kapalı: &HashSet<String>,
    matris_yerleşimleri: &[Option<crate::koordinat::MatrisYerleşimi>],
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

    let aralıklar =
        yığın_aralıkları_seçici(&seçenekler.seriler, &görünürler, |_, seri, _, öğe| {
            let Seri::Sütun(_) = seri else {
                return öğe.değer.sayı();
            };
            let bağ = seri.eksen_bağı();
            let yatay = x_seçenekler
                .get(bağ.x)
                .zip(y_seçenekler.get(bağ.y))
                .is_some_and(|(x, y)| sütun_tabanı_y_mi(x, y));
            sütun_değeri(öğe, yatay)
        });

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
        if let Seri::Saçılım(saçılım) = seri {
            if saçılım.düz_veri.is_some() {
                for (_, x, y) in saçılım.düz_xy_iter() {
                    if !x_kategorik {
                        kapsa(x_kapsam, x);
                    }
                    if !y_kategorik {
                        kapsa(y_kapsam, y);
                    }
                }
            } else if let Some((x_boyutu, y_boyutu)) = &saçılım.eşleme {
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
            } else {
                for (sıra, öğe) in saçılım.veri.iter().enumerate() {
                    let Some((x, y)) = saçılım_xy(&öğe.değer, sıra) else {
                        continue;
                    };
                    if !x_kategorik {
                        kapsa(x_kapsam, x);
                    }
                    if !y_kategorik {
                        kapsa(y_kapsam, y);
                    }
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
        let sütun_taban_y = sütun_mu && sütun_tabanı_y_mi(x_seçenek, y_seçenek);
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
            // Bar taban ekseni y olduğunda (kategori yanında zaman da)
            // değerler x'e akar. Çizginin tarihsel y-kategori davranışı aynı
            // seçimle korunur.
            let değer_yatay = if sütun_mu {
                sütun_taban_y
            } else {
                y_kategorik && !x_kategorik
            };
            let değer_kapsamı: &mut [f64; 2] = if değer_yatay { x_kapsam } else { y_kapsam };
            kapsa(değer_kapsamı, *tepe);
            // Bar'ın geometrik tabanı her zaman sıfırdır; ancak `scale: true`
            // (`sıfırı_içer: false`) değer ekseninin veri kapsamına sıfırı
            // katmaz. ECharts tabanı ölçek dışında eşleyip grid'de kırpar.
            // Yığılmış serinin sıfırdan farklı tabanı ise gerçek veri
            // kapsamıdır ve her iki kipte de korunmalıdır.
            let değer_ekseni_sıfırı_içer = if değer_yatay {
                x_seçenek.sıfırı_içer
            } else {
                y_seçenek.sıfırı_içer
            };
            if taban.abs() > 1e-12 || (sütun_mu && değer_ekseni_sıfırı_içer) {
                kapsa(değer_kapsamı, *taban);
            }
            if sütun_mu {
                let taban_değeri = seri
                    .veri()
                    .get(j)
                    .map(|öğe| sütun_taban_değeri(öğe, j, sütun_taban_y))
                    .unwrap_or(j as f64);
                if sütun_taban_y {
                    if !y_kategorik {
                        kapsa(y_kapsam, taban_değeri);
                    }
                } else if !x_kategorik {
                    kapsa(x_kapsam, taban_değeri);
                }
            } else if x_kategorik || !y_kategorik {
                let x_değeri = seri
                    .veri()
                    .get(j)
                    .and_then(|ö| ö.değer.x())
                    .unwrap_or(j as f64);
                kapsa(x_kapsam, x_değeri);
            }
        }
    }

    // ECharts 6 `axisBand` sayısal/zaman tabanında pozitif en küçük veri
    // aralığından bant üretir ve `containShape` eşleme kapsamını iki uçta
    // yarım bant genişletir. Böylece ilk/son bar koordinat alanında tam
    // görünür; kategori ekseninin boundaryGap davranışının sayısal eşidir.
    let mut x_sütun_tabanları = vec![Vec::<f64>::new(); x_seçenekler.len()];
    let mut y_sütun_tabanları = vec![Vec::<f64>::new(); y_seçenekler.len()];
    for (seri_sırası, seri) in seçenekler.seriler.iter().enumerate() {
        if !görünürler.get(seri_sırası).copied().unwrap_or(false) || !matches!(seri, Seri::Sütun(_))
        {
            continue;
        }
        let bağ = seri.eksen_bağı();
        let Some((x, y)) = x_seçenekler.get(bağ.x).zip(y_seçenekler.get(bağ.y)) else {
            continue;
        };
        let yatay = sütun_tabanı_y_mi(x, y);
        let hedef = if yatay {
            if y.tür == EksenTürü::Kategori {
                continue;
            }
            y_sütun_tabanları.get_mut(bağ.y)
        } else {
            if x.tür == EksenTürü::Kategori {
                continue;
            }
            x_sütun_tabanları.get_mut(bağ.x)
        };
        let Some(hedef) = hedef else { continue };
        hedef.extend(
            seri.veri()
                .iter()
                .enumerate()
                .map(|(sıra, öğe)| sütun_taban_değeri(öğe, sıra, yatay))
                .filter(|değer| değer.is_finite()),
        );
    }
    let en_küçük_pozitif_aralık = |değerler: &mut Vec<f64>| -> Option<f64> {
        değerler.sort_by(f64::total_cmp);
        değerler.dedup_by(|a, b| (*a - *b).abs() <= f64::EPSILON);
        değerler
            .windows(2)
            .filter_map(|çift| match çift {
                [a, b] => {
                    let fark = b - a;
                    (fark > 0.0 && fark.is_finite()).then_some(fark)
                }
                _ => None,
            })
            .min_by(f64::total_cmp)
    };
    let x_sütun_taban_aralıkları = x_sütun_tabanları
        .iter_mut()
        .map(&en_küçük_pozitif_aralık)
        .collect::<Vec<_>>();
    let y_sütun_taban_aralıkları = y_sütun_tabanları
        .iter_mut()
        .map(en_küçük_pozitif_aralık)
        .collect::<Vec<_>>();

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
            if let Seri::Saçılım(saçılım) = seri {
                if saçılım.düz_veri.is_some() {
                    for (_, x, y) in saçılım.düz_xy_iter() {
                        let x_değerleri = [x];
                        let y_değerleri = [y];
                        if !pencereden_geçer(x_penceresi, &x_değerleri)
                            || !pencereden_geçer(y_penceresi, &y_değerleri)
                        {
                            continue;
                        }
                        if !x_kategorik {
                            kapsa(x_kapsam, x);
                        }
                        if !y_kategorik {
                            kapsa(y_kapsam, y);
                        }
                    }
                    continue;
                }
                for (sıra, öğe) in saçılım.veri.iter().enumerate() {
                    let (x, y) = match &saçılım.eşleme {
                        Some((x_boyutu, y_boyutu)) => (
                            öğe.boyut(x_boyutu).and_then(|değer| değer.sayı()),
                            öğe.boyut(y_boyutu).and_then(|değer| değer.sayı()),
                        ),
                        None => saçılım_xy(&öğe.değer, sıra)
                            .map(|(x, y)| (Some(x), Some(y)))
                            .unwrap_or((None, None)),
                    };
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
            let sütun_taban_y = sütun_mu && sütun_tabanı_y_mi(x_seçenek, y_seçenek);
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
                let taban_değeri = seri
                    .veri()
                    .get(veri_sırası)
                    .map(|öğe| sütun_taban_değeri(öğe, veri_sırası, sütun_taban_y))
                    .unwrap_or(veri_sırası as f64);
                let değer_yatay = if sütun_mu {
                    sütun_taban_y
                } else {
                    y_kategorik && !x_kategorik
                };
                let (x_değerleri, y_değerleri) = if değer_yatay {
                    ([*tepe], [taban_değeri])
                } else {
                    ([if sütun_mu { taban_değeri } else { x_değeri }], [*tepe])
                };
                if !pencereden_geçer(x_penceresi, &x_değerleri)
                    || !pencereden_geçer(y_penceresi, &y_değerleri)
                {
                    continue;
                }
                let değer_kapsamı = if değer_yatay {
                    &mut *x_kapsam
                } else {
                    &mut *y_kapsam
                };
                kapsa(değer_kapsamı, *tepe);
                let değer_ekseni_sıfırı_içer = if değer_yatay {
                    x_seçenek.sıfırı_içer
                } else {
                    y_seçenek.sıfırı_içer
                };
                if taban.abs() > 1e-12 || (sütun_mu && değer_ekseni_sıfırı_içer) {
                    kapsa(değer_kapsamı, *taban);
                }
                if sütun_mu {
                    if sütun_taban_y {
                        if !y_kategorik {
                            kapsa(y_kapsam, taban_değeri);
                        }
                    } else if !x_kategorik {
                        kapsa(x_kapsam, taban_değeri);
                    }
                } else if x_kategorik || !y_kategorik {
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
            let ana_alan = ızgara
                .matris_sırası
                .zip(ızgara.matris_koordinatı.as_ref())
                .and_then(|(sıra, (x, y))| {
                    matris_yerleşimleri
                        .get(sıra)?
                        .as_ref()?
                        .veriden_yerleşime(x, y, true)
                })
                .unwrap_or_else(|| {
                    Dikdörtgen::yeni(0.0, 0.0, yüzey.genişlik(), yüzey.yükseklik())
                });
            let sol_boşluk = ızgara.sol.çöz(ana_alan.genişlik);
            // `containLabel`, yatay eksenin uç etiketini grid'in içine
            // zorlamaz; ECharts açık `right` mesafesini aynen korur ve uç
            // kategori etiketi tuval kenarına kadar uzanabilir.
            let sağ_boşluk = ızgara.sağ.çöz(ana_alan.genişlik);
            let üst_boşluk = ızgara.üst.çöz(ana_alan.yükseklik);
            let alt_boşluk = ızgara.alt.çöz(ana_alan.yükseklik);
            let açık_genişlik = ızgara
                .genişlik
                .map(|uzunluk| uzunluk.çöz(ana_alan.genişlik));
            let açık_yükseklik = ızgara
                .yükseklik
                .map(|uzunluk| uzunluk.çöz(ana_alan.yükseklik));
            let mut genişlik = açık_genişlik
                .unwrap_or_else(|| ana_alan.genişlik - sol_boşluk - sağ_boşluk)
                .max(1.0);
            let mut yükseklik = açık_yükseklik
                .unwrap_or_else(|| ana_alan.yükseklik - üst_boşluk - alt_boşluk)
                .max(1.0);
            let mut sol = if açık_genişlik.is_some() && ızgara.sağ_açık && !ızgara.sol_açık
            {
                ana_alan.x + ana_alan.genişlik - sağ_boşluk - genişlik
            } else {
                ana_alan.x + sol_boşluk
            };
            let üst = if açık_yükseklik.is_some() && ızgara.alt_açık && !ızgara.üst_açık {
                ana_alan.y + ana_alan.yükseklik - alt_boşluk - yükseklik
            } else {
                ana_alan.y + üst_boşluk
            };
            if ızgara.etiketi_kapsa {
                if let Some((yi, y_seçenek)) = y_seçenekler
                    .iter()
                    .enumerate()
                    .find(|(_, e)| e.ızgara_sırası == g && e.etiket.göster && !e.etiket.içeride)
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
                    let sol_etiket_payı = en_geniş + y_seçenek.etiket.boşluk - 0.04;
                    sol += sol_etiket_payı;
                    genişlik = (genişlik - sol_etiket_payı).max(1.0);
                }
                if let Some(x_seçenek) = x_seçenekler
                    .iter()
                    .find(|e| e.ızgara_sırası == g && e.etiket.göster && !e.etiket.içeride)
                {
                    let x_boyut = x_seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
                    // Tek satırlı eksen etiketi için zrender sınır kutusu
                    // font boyudur; genel rich-text satır oranı burada
                    // fazladan dikey boşluk üretmemelidir.
                    yükseklik = (yükseklik - x_boyut - x_seçenek.etiket.boşluk).max(1.0);
                }
            }
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
    // ECharts 6 `createBandWidthBasedAxisContainShapeHandler`, boundaryGap
    // kapalı kategori ekseninde sütun/kutu/mum gövdesinin kırpılmaması için
    // eşleme kapsamını iki uçta yarım veri bandı genişletir.
    let bant_şeklini_kapsar = |x_ekseni_mi: bool, eksen_sırası: usize| {
        seçenekler
            .seriler
            .iter()
            .zip(&görünürler)
            .any(|(seri, görünür)| {
                if !*görünür || !matches!(seri, Seri::Sütun(_) | Seri::Mum(_) | Seri::Kutu(_)) {
                    return false;
                }
                let bağ = seri.eksen_bağı();
                if x_ekseni_mi {
                    bağ.x == eksen_sırası
                } else {
                    bağ.y == eksen_sırası
                }
            })
    };
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
            if pencere.is_none()
                && seçenek.tür != EksenTürü::Kategori
                && let Some(aralık) = x_sütun_taban_aralıkları.get(xi).copied().flatten()
            {
                let ham = eksen.ölçek.kapsam();
                let alt = if seçenek.en_az.is_none() && !seçenek.en_az_veri {
                    ham[0] - aralık / 2.0
                } else {
                    ham[0]
                };
                let üst = if seçenek.en_çok.is_none() && !seçenek.en_çok_veri {
                    ham[1] + aralık / 2.0
                } else {
                    ham[1]
                };
                eksen.eşleme_kapsamı_uygula([alt, üst]);
            }
            if let Some(([p0, p1], oranlar)) = pencere {
                eksen.yakınlaştırma_oranları = Some(oranlar);
                if let Some(yakınlaştırma) = yakınlaştırma {
                    eksen.yakınlaştırma_süzme_kipi = yakınlaştırma.süzme_kipi;
                }
                if seçenek.tür == EksenTürü::Kategori {
                    let kapsama_payı = if !seçenek.bantlı_mı() && bant_şeklini_kapsar(true, xi)
                    {
                        0.5
                    } else {
                        0.0
                    };
                    eksen.değer_penceresi_uygula(
                        p0.round() - kapsama_payı,
                        p1.round() + kapsama_payı,
                    );
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
            if pencere.is_none()
                && seçenek.tür != EksenTürü::Kategori
                && let Some(aralık) = y_sütun_taban_aralıkları.get(yi).copied().flatten()
            {
                let ham = eksen.ölçek.kapsam();
                let alt = if seçenek.en_az.is_none() && !seçenek.en_az_veri {
                    ham[0] - aralık / 2.0
                } else {
                    ham[0]
                };
                let üst = if seçenek.en_çok.is_none() && !seçenek.en_çok_veri {
                    ham[1] + aralık / 2.0
                } else {
                    ham[1]
                };
                eksen.eşleme_kapsamı_uygula([alt, üst]);
            }
            if let Some(([p0, p1], oranlar)) = pencere {
                eksen.yakınlaştırma_oranları = Some(oranlar);
                if let Some(yakınlaştırma) = yakınlaştırma {
                    eksen.yakınlaştırma_süzme_kipi = yakınlaştırma.süzme_kipi;
                }
                if seçenek.tür == EksenTürü::Kategori {
                    let kapsama_payı = if !seçenek.bantlı_mı() && bant_şeklini_kapsar(false, yi)
                    {
                        0.5
                    } else {
                        0.0
                    };
                    eksen.değer_penceresi_uygula(
                        p0.round() - kapsama_payı,
                        p1.round() + kapsama_payı,
                    );
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
            boyutlar: öğe.boyutlar.clone(),
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
        if gösterge.seçim_kipi == crate::model::bilesen::GöstergeSeçimKipi::Tek
            && girdi.kapalı.is_empty()
        {
            let adlar = gösterge_öğeleri(seçenekler, &HashSet::new())
                .into_iter()
                .map(|öğe| öğe.ad)
                .collect::<Vec<_>>();
            let seçilen = adlar
                .iter()
                .find(|ad| gösterge.seçili.get(*ad).copied().unwrap_or(true))
                .or_else(|| adlar.first());
            if let Some(seçilen) = seçilen {
                for ad in &adlar {
                    if ad != seçilen {
                        etkili_kapalı.insert(ad.clone());
                    }
                }
            }
        }
    }
    let kapalı = &etkili_kapalı;
    let ipucu_seçeneği = seçenekler.ipucu.clone().filter(|i| i.göster);
    // `(başlık, satırlar, konum)`; bütün koordinat sistemleri aynı üst
    // katmandaki tooltip penceresine veri bırakır.
    let mut bekleyen_ipucu: Option<Bekleyenİpucu> = None;

    // 1) Arka plan (koyu temada zemin, açıkça verilmemişse de doldurulur).
    let zemin = seçenekler
        .arkaplan
        .clone()
        .or_else(|| seçenekler.koyu.then(|| Dolgu::Düz(crate::tema::zemin())));
    if let Some(dolgu) = zemin {
        let tümü = Dikdörtgen::yeni(0.0, 0.0, yüzey.genişlik(), yüzey.yükseklik());
        yüzey.dikdörtgen(tümü, &dolgu, [0.0; 4], None);
    }

    // Matrix koordinatları seri katmanlarının altında ortak bileşenlerdir.
    // x/y.data boşsa ordinal kategoriler bağlı serilerin veri/encode
    // girdilerinden ECharts'ın toplama aşaması gibi çıkarılır.
    let matrisler = seçenekler.tüm_matrisler().collect::<Vec<_>>();
    let matris_yerleşimleri = matrisler
        .iter()
        .enumerate()
        .map(|(sıra, matris)| {
            let (x, y) = matris_seri_kategorilerini_topla(seçenekler, sıra);
            crate::koordinat::MatrisYerleşimi::kur_adlarla_sıralı(
                matris,
                (yüzey.genişlik(), yüzey.yükseklik()),
                (&x, &y),
                sıra,
            )
            .ok()
        })
        .collect::<Vec<_>>();
    for (matris, yerleşim) in matrisler.iter().zip(&matris_yerleşimleri) {
        if let Some(yerleşim) = yerleşim {
            matris_çiz(yüzey, matris, yerleşim);
        }
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

    // `singleAxis` bileşenleri z=0/seri z=2 katmanındadır. TitleView z=6
    // olduğundan resmî çoklu satır örneğindeki gün başlıkları bunların
    // üzerinde boyanır.
    let tek_eksen_yerleşimleri =
        tek_eksen_yerleşimlerini_kur(seçenekler, kapalı, (yüzey.genişlik(), yüzey.yükseklik()));
    if let Some(ipucu) = tek_eksenleri_çiz(
        yüzey,
        seçenekler,
        &tek_eksen_yerleşimleri,
        kapalı,
        ilerleme,
        zaman_sn,
        ipucu_seçeneği.as_ref(),
        fare,
        girdi.ipucu_öğesi,
        &mut çıktı.isabetler,
    ) {
        bekleyen_ipucu = Some(ipucu);
    }

    // 2) Başlık.
    let başlığı_çiz = |yüzey: &mut dyn ÇizimYüzeyi, başlık: &crate::model::bilesen::Başlık| {
        let matris_alanı = başlık
            .matris_sırası
            .zip(başlık.matris_koordinatı.as_ref())
            .and_then(|(sıra, (x, y))| {
                matris_yerleşimleri
                    .get(sıra)?
                    .as_ref()?
                    .veriden_yerleşime(x, y, true)
            });
        if let Some(alan) = matris_alanı {
            başlık_çiz_alanda(yüzey, başlık, alan);
        } else if başlık.matris_sırası.is_none() {
            başlık_çiz(yüzey, başlık);
        }
    };
    if seçenekler.başlıklar.is_empty() {
        if let Some(başlık) = &seçenekler.başlık {
            başlığı_çiz(yüzey, başlık);
        }
    } else {
        for başlık in &seçenekler.başlıklar {
            başlığı_çiz(yüzey, başlık);
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
            AraçKutusuÖzelliği::Fırça,
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
                AraçKutusuÖzelliği::Fırça if !araçlar.fırça_türleri.is_empty() => {
                    türler.extend(araçlar.fırça_türleri.iter().copied().map(AraçTürü::Fırça));
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
            AraçTürü::Fırça(tür) => fırça_aracı_boyutu(tür).0,
            AraçTürü::SihirliÇizgi => 20.0,
            AraçTürü::SihirliSütun => 19.383_928,
            AraçTürü::SihirliYığın => 20.0,
            AraçTürü::GeriYükle => 19.915_937,
            AraçTürü::SvgKaydet | AraçTürü::PngKaydet => 17.956_896,
        };
        let hit_yüksekliği = |tür: AraçTürü| match tür {
            AraçTürü::VeriGörünümü => 20.0,
            AraçTürü::VeriYakınlaştır | AraçTürü::VeriYakınlaştırmayıGeriAl => 20.0,
            AraçTürü::Fırça(tür) => fırça_aracı_boyutu(tür).1,
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
                AraçTürü::Fırça(tür) => {
                    if let Some(fırça_yolu) = fırça_aracı_yolu(tür, merkez) {
                        yol = fırça_yolu;
                    }
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

    // 4) Kartezyen bölüm (çoklu ızgara/eksen).
    let kurulum = kartezyen_kur_matrisli(yüzey, seçenekler, kapalı, &matris_yerleşimleri);
    let hazır_fırça = kurulum
        .as_ref()
        .map(|kurulum| fırçayı_hazırla(seçenekler, kurulum))
        .unwrap_or_default();
    çıktı
        .fırça_seçimleri
        .clone_from(&hazır_fırça.seçili_ham_sıralar);
    if let Some(kurulum) = &kurulum {
        çıktı.ızgara_alanları.clone_from(&kurulum.ızgara_alanları);
    }
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
            let alt_eksenler = ızgara_eksenleri
                .iter()
                .copied()
                .filter(|eksen| eksen.seçenek.z <= 2)
                .collect::<Vec<_>>();
            bölme_çizgilerini_çiz(yüzey, *alan, &alt_eksenler);

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

            eksenleri_çiz_katman(yüzey, *alan, &ızgara_eksenleri, false);
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
        let mut sütun_grupları: Vec<((bool, usize), Vec<SütunGirdisi>)> = Vec::new();
        for (i, s) in seçenekler.seriler.iter().enumerate() {
            if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            if let Seri::Sütun(sütun) = s {
                let Some(seri_kartezyeni) = kurulum.seri_kartezyeni(s) else {
                    continue;
                };
                let anahtar = sütun_grup_anahtarı(s, kurulum);
                let girdi = SütunGirdisi {
                    seri: sütun,
                    kartezyen: seri_kartezyeni,
                    genel_sıra: i,
                    aralıklar: kurulum.aralıklar.get(i).map(Vec::as_slice).unwrap_or(&[]),
                    renk: seçenekler.seri_rengi(i),
                    görsel_eşlemeler: seçenekler
                        .seri_görsel_eşlemeleri(i)
                        .map(|eşleme| (eşleme, sütun_görsel_kapsamı(sütun, eşleme)))
                        .collect(),
                    öğe_opaklıkları: hazır_fırça
                        .öğe_opaklıkları
                        .get(i)
                        .and_then(Option::as_deref),
                    öğe_renkleri: hazır_fırça.öğe_renkleri.get(i).and_then(Option::as_deref),
                };
                match sütun_grupları.iter_mut().find(|(aday, _)| *aday == anahtar) {
                    Some((_, grup)) => grup.push(girdi),
                    None => sütun_grupları.push((anahtar, vec![girdi])),
                }
            }
        }
        let mut çizilen_sütun_grupları: HashSet<(bool, usize)> = HashSet::new();

        // Saçılım vurgusu (öğe ipucu) için önden isabet araması. Büyük kip,
        // her nokta için ağır model/isabet nesnesi ayırmadan düz piksel
        // tamponunu korur.
        enum HazırSaçılım {
            Normal(Vec<SaçılımNoktası>),
            Büyük(BüyükSaçılımNoktaları),
        }
        // `(seri sırası, vurgulu veri sırası, noktalar)`.
        type SaçılımVurgusu = (usize, Option<usize>, HazırSaçılım);
        let mut saçılım_vurguları: Vec<SaçılımVurgusu> = Vec::new();
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if let Seri::Saçılım(s) = seri {
                if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                    continue;
                }
                let Some(kartezyen) = kurulum.seri_kartezyeni(seri) else {
                    continue;
                };
                if s.büyük_etkin_mi()
                    && let Some(noktalar) = büyük_saçılım_noktaları(s, &kartezyen)
                {
                    // LargeSymbolPath.findDataIndex: en son çizilen öğeden
                    // geriye doğru, en az 4x4 px dikdörtgen isabet sınaması.
                    let vurgu = match (s.sessiz, &ipucu_seçeneği, fare) {
                        (false, Some(ipucu), Some(f)) if ipucu.tetikleme == Tetikleme::Öğe => {
                            let boyut = noktalar.boyut.max(4.0);
                            let yarı = boyut / 2.0;
                            noktalar
                                .konumlar
                                .chunks_exact(2)
                                .enumerate()
                                .rev()
                                .find_map(|(sıra, çift)| {
                                    let [x, y] = çift else { return None };
                                    (x.is_finite()
                                        && y.is_finite()
                                        && f.0 >= *x - yarı
                                        && f.0 <= *x + yarı
                                        && f.1 >= *y - yarı
                                        && f.1 <= *y + yarı)
                                        .then_some(sıra)
                                })
                        }
                        _ => None,
                    };
                    saçılım_vurguları.push((i, vurgu, HazırSaçılım::Büyük(noktalar)));
                    continue;
                }
                let görsel_eşlemeler = seçenekler
                    .seri_görsel_eşlemeleri(i)
                    .map(|eşleme| (eşleme, saçılım_görsel_kapsamı(s, eşleme)))
                    .collect::<Vec<_>>();
                let mut noktalar = saçılım_noktaları(s, &kartezyen);
                saçılım_nokta_boyutlarını_eşle(s, &mut noktalar, &görsel_eşlemeler);
                let vurgu = match (s.sessiz, &ipucu_seçeneği, fare) {
                    (false, Some(ipucu), Some(f)) if ipucu.tetikleme == Tetikleme::Öğe => {
                        noktalar
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
                            .map(|n| n.sıra)
                    }
                    _ => None,
                };
                saçılım_vurguları.push((i, vurgu, HazırSaçılım::Normal(noktalar)));
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
                    hazır_fırça
                        .öğe_opaklıkları
                        .get(i)
                        .and_then(Option::as_deref),
                    ilerleme,
                    ÇizgiKatmanı::Alan,
                    None,
                );
            };
            // LineView alan grubu `clip: true` öntanımıyla her zaman
            // koordinat alanına kırpılır. Bu özellikle eksen sıfırı görünür
            // kapsamın dışındayken alan tabanının tuval dışına taşmasını önler.
            yüzey.kırpılı(kartezyen.alan, &mut alanı_çiz);
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
            // Scatter, ECharts `SymbolDraw` gibi merkez-bazlı süzülür ve
            // kenardaki sembolün taşan kısmı korunur. Diğer serilerde etkin
            // yakınlaştırma penceresi ızgara kırpmasını kullanır.
            let pencereli = (!matches!(seri, Seri::Saçılım(_))
                && (kartezyen.x.pencere.is_some() || kartezyen.y.pencere.is_some()))
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
                                hazır_fırça
                                    .öğe_opaklıkları
                                    .get(i)
                                    .and_then(Option::as_deref),
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
                            let anahtar = sütun_grup_anahtarı(seri, kurulum);
                            if çizilen_sütun_grupları.insert(anahtar)
                                && let Some((_, girdiler)) =
                                    sütun_grupları.iter().find(|(aday, _)| *aday == anahtar)
                            {
                                sütunları_çiz(yüzey, girdiler, ilerleme, fare, isabetler);
                            }
                        }
                        Seri::Saçılım(s) => {
                            let kayıt = saçılım_vurguları.iter().find(|(sıra, ..)| *sıra == i);
                            if let Some((_, vurgu, noktalar)) = kayıt {
                                match noktalar {
                                    HazırSaçılım::Normal(noktalar) => {
                                        let görsel_eşlemeler = seçenekler
                                            .seri_görsel_eşlemeleri(i)
                                            .map(|eşleme| {
                                                (eşleme, saçılım_görsel_kapsamı(s, eşleme))
                                            })
                                            .collect::<Vec<_>>();
                                        saçılım_çiz_çoklu_eşlemeli(
                                            yüzey,
                                            s,
                                            noktalar,
                                            seçenekler.seri_rengi(i),
                                            ilerleme,
                                            zaman_sn,
                                            *vurgu,
                                            &görsel_eşlemeler,
                                            &seçenekler.palet,
                                        );
                                        if !s.sessiz {
                                            for n in noktalar {
                                                isabetler.push(İsabetBölgesi {
                                                    seri_sırası: i,
                                                    veri_sırası: n.sıra,
                                                    seri_adı: s.ad.clone(),
                                                    ad: s
                                                        .veri
                                                        .get(n.sıra)
                                                        .and_then(|ö| ö.ad.clone()),
                                                    değer: Some(n.y_değeri),
                                                    geometri: İsabetGeometrisi::Daire {
                                                        merkez: n.konum,
                                                        yarıçap: (n.boyut / 2.0 + 3.0).max(8.0),
                                                    },
                                                });
                                            }
                                        }
                                    }
                                    HazırSaçılım::Büyük(noktalar) => {
                                        büyük_saçılım_çiz(
                                            yüzey,
                                            s,
                                            noktalar,
                                            seçenekler.seri_rengi(i),
                                            ilerleme,
                                        );
                                        // Bir milyon ayrı bölge yerine yalnız
                                        // ters taramada bulunan etkin öğe
                                        // olay hattına aktarılır.
                                        if !s.sessiz
                                            && let Some(sıra) = *vurgu
                                            && let Some(çift) =
                                                noktalar.konumlar.chunks_exact(2).nth(sıra)
                                            && let [x, y] = çift
                                        {
                                            let boyut = noktalar.boyut.max(4.0);
                                            isabetler.push(İsabetBölgesi {
                                                seri_sırası: i,
                                                veri_sırası: sıra,
                                                seri_adı: s.ad.clone(),
                                                ad: None,
                                                değer: s.xy(sıra).map(|(_, y)| y),
                                                geometri: İsabetGeometrisi::Dikdörtgen(
                                                    Dikdörtgen::yeni(
                                                        *x - boyut / 2.0,
                                                        *y - boyut / 2.0,
                                                        boyut,
                                                        boyut,
                                                    ),
                                                ),
                                            });
                                        }
                                    }
                                }
                                // Öğe ipucu.
                                let ipucu_xy = vurgu.and_then(|sıra| match noktalar {
                                    HazırSaçılım::Normal(noktalar) => noktalar
                                        .iter()
                                        .find(|nokta| nokta.sıra == sıra)
                                        .map(|nokta| (nokta.x_değeri, nokta.y_değeri)),
                                    HazırSaçılım::Büyük(_) => s.xy(sıra),
                                });
                                if let (Some((x, y)), Some(f)) = (ipucu_xy, fare) {
                                    *bekleyen = Some((
                                        seri.ad().map(str::to_string),
                                        vec![İpucuSatırı {
                                            im_rengi: Some(seçenekler.seri_rengi(i)),
                                            ad: format!("({}, {})", binlik_ayır(x), binlik_ayır(y)),
                                            değer: String::new(),
                                        }],
                                        f,
                                    ));
                                }
                            }
                        }
                        // Brush `colorAlpha`, candlestick `drawType` olan
                        // gövde dolgusuna uygulanır; fitil ve kenarlık rengi
                        // ayrı stroke kanalında opak kalır.
                        Seri::Mum(s) => mum_çiz(
                            yüzey,
                            s,
                            i,
                            &kartezyen,
                            ilerleme,
                            hazır_fırça
                                .öğe_opaklıkları
                                .get(i)
                                .and_then(Option::as_deref),
                            isabetler,
                        ),
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
                                    takvim: None,
                                    matris: None,
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

        // Sütun dolgusu kategori ekseninin onZero taban vuruşunu örtmesin.
        // Yalnız sütun taşıyan ızgaralarda yalnız kategori axisLine yeniden
        // boyanır; bar dikdörtgeni ve iç etiket çapasına dokunulmaz.
        for (g, alan) in kurulum.ızgara_alanları.iter().enumerate() {
            let sütun_var = seçenekler.seriler.iter().enumerate().any(|(i, seri)| {
                kurulum.görünürler.get(i).copied().unwrap_or(false)
                    && matches!(seri, Seri::Sütun(_))
                    && kurulum
                        .seri_kartezyeni(seri)
                        .is_some_and(|kartezyen| kartezyen.x.seçenek.ızgara_sırası == g)
            });
            if !sütun_var {
                continue;
            }
            let ızgara_eksenleri = kurulum
                .x_eksenler
                .iter()
                .chain(kurulum.y_eksenler.iter())
                .filter(|eksen| eksen.seçenek.ızgara_sırası == g)
                .collect::<Vec<_>>();
            kategori_taban_çizgilerini_üstte_çiz(yüzey, *alan, &ızgara_eksenleri);
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
                    let anahtar = sütun_grup_anahtarı(seri, kurulum);
                    sütun_grupları
                        .iter()
                        .find(|(aday, _)| *aday == anahtar)
                        .and_then(|(_, girdiler)| {
                            let bant_genişliği = sütun_bant_genişliği(girdiler);
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

        // ECharts kartezyen serileri z=2 katmanındadır. Daha yüksek `axis.z`
        // isteyen eksenlerin bölme/etiket görselleri seri ve imleyicilerden
        // sonra boyanır; örneğin beyaz `axisLabel.inside` metni sütunların
        // üstünde kalır.
        for (g, alan) in kurulum.ızgara_alanları.iter().enumerate() {
            let ızgara_eksenleri = kurulum
                .x_eksenler
                .iter()
                .chain(kurulum.y_eksenler.iter())
                .filter(|eksen| eksen.seçenek.ızgara_sırası == g)
                .collect::<Vec<_>>();
            let üst_eksenler = ızgara_eksenleri
                .iter()
                .copied()
                .filter(|eksen| eksen.seçenek.z > 2)
                .collect::<Vec<_>>();
            if üst_eksenler.is_empty() {
                continue;
            }
            bölme_çizgilerini_çiz(yüzey, *alan, &üst_eksenler);
            eksenleri_çiz_katman(yüzey, *alan, &ızgara_eksenleri, true);
        }

        // `breakArea.zigzagZ` öntanımlı olarak 100'dür; dolgu ve zikzaklar
        // normal z=2 seri katmanının üstünde yeniden boyanarak kırığı geçen
        // sütun/çizgileri görünür biçimde keser.
        for (g, alan) in kurulum.ızgara_alanları.iter().enumerate() {
            let ızgara_eksenleri = kurulum
                .x_eksenler
                .iter()
                .chain(kurulum.y_eksenler.iter())
                .filter(|eksen| eksen.seçenek.ızgara_sırası == g)
                .collect::<Vec<_>>();
            kırılma_alanlarını_çiz(yüzey, *alan, &ızgara_eksenleri, true);
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
                            .or_else(|| {
                                yakınlaştırma
                                    .sağ
                                    .map(|u| yüzey.genişlik() - u.çöz(yüzey.genişlik()) - genişlik)
                            })
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
                            // SliderZoomView'in dikey öntanımlısı hedef
                            // kartezyen alanın üst kenarına hizalanır.
                            .unwrap_or(alan.y)
                            // Dikey grupta döndürülmüş varsayılan tutamaç
                            // yolunun sınır kutusu, filler başlangıcını
                            // layout konumundan 1.764 px aşağı taşır.
                            + 1.764_264_2;
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
                            .or_else(|| {
                                yakınlaştırma
                                    .sağ
                                    .map(|u| yüzey.genişlik() - u.çöz(yüzey.genişlik()) - genişlik)
                            })
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
                    // ECharts `showDataShadow: auto`: hedef eksenlerden
                    // birine bağlı ilk uygun serinin karşı boyutunu sürgü
                    // arkasında gösterir. Dikey SliderZoomView aynı yerel
                    // yolu 90° döndürür; burada eşdeğer koordinatlar doğrudan
                    // tuval uzayında üretilir.
                    if yakınlaştırma.veri_gölgesi
                        && let Some(gölge_serisi) = seçenekler.seriler.iter().find(|seri| {
                            if !seri.kartezyen_mi() {
                                return false;
                            }
                            let bağ = seri.eksen_bağı();
                            let hedefte = if dikey {
                                yakınlaştırma.y_eksenini_hedefler(bağ.y)
                            } else {
                                yakınlaştırma.x_eksenini_hedefler(bağ.x)
                            };
                            hedefte
                                && match seri {
                                    Seri::Saçılım(saçılım) => saçılım.veri_sayısı() > 0,
                                    _ => !seri.veri().is_empty(),
                                }
                        })
                    {
                        let veri_sayısı = match gölge_serisi {
                            Seri::Saçılım(saçılım) => saçılım.veri_sayısı(),
                            _ => gölge_serisi.veri().len(),
                        };
                        let gölge_bağı = gölge_serisi.eksen_bağı();
                        let gölge_x_kategorik = kurulum
                            .x_eksenler
                            .get(gölge_bağı.x)
                            .is_some_and(|eksen| eksen.seçenek.tür == EksenTürü::Kategori);
                        let gölge_y_kategorik = kurulum
                            .y_eksenler
                            .get(gölge_bağı.y)
                            .is_some_and(|eksen| eksen.seçenek.tür == EksenTürü::Kategori);
                        let xy_al = |sıra: usize| match gölge_serisi {
                            Seri::Saçılım(saçılım) if saçılım.düz_veri.is_some() => {
                                saçılım.xy(sıra)
                            }
                            // CandlestickSeries.getShadowDim() açıkça
                            // `open` döndürür. Boxplot bu özel yolu taşımaz;
                            // SliderZoomView'da genel mapDimension sonucu
                            // geçerli olduğunda ayrıca ele alınmalıdır.
                            Seri::Mum(_) => gölge_serisi
                                .veri()
                                .get(sıra)
                                .and_then(|öğe| öğe.değer.dizi())
                                .and_then(|değerler| değerler.first().copied())
                                .map(|değer| {
                                    if gölge_y_kategorik && !gölge_x_kategorik {
                                        (değer, sıra as f64)
                                    } else {
                                        (sıra as f64, değer)
                                    }
                                }),
                            _ => gölge_serisi.veri().get(sıra).and_then(|öğe| {
                                if let Some(x) = öğe.değer.x() {
                                    Some((x, öğe.değer.sayı()?))
                                } else {
                                    // Kategori eksenli line/bar verisi
                                    // tek bir sayıdır; ECharts veri
                                    // deposu örtük kategori boyutunu
                                    // satır sırasından üretir.
                                    Some((sıra as f64, öğe.değer.sayı()?))
                                }
                            }),
                        };
                        let bu_eksen_değeri_al =
                            |sıra: usize| xy_al(sıra).map(|(x, y)| if dikey { y } else { x });
                        let karşı_değer_al =
                            |sıra: usize| xy_al(sıra).map(|(x, y)| if dikey { x } else { y });
                        let kapsam = (0..veri_sayısı)
                            .filter_map(karşı_değer_al)
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
                        let uzunluk = if dikey {
                            şerit.yükseklik
                        } else {
                            şerit.genişlik
                        };
                        let kalınlık = if dikey {
                            şerit.genişlik
                        } else {
                            şerit.yükseklik
                        };
                        let eşle = |değer: f64| {
                            (((değer - alt_kapsam) / (üst_kapsam - alt_kapsam)) as f32 * kalınlık)
                                .clamp(0.0, kalınlık)
                        };
                        // SliderZoomView zaman ekseninde gölge
                        // koordinatını veri sırasından değil ham zaman
                        // değerinin kapsam içindeki konumundan üretir. Bu
                        // ayrım, seans aralarındaki boş satırların önizlemede
                        // doğru genişlikte görünmesi için gereklidir. Kırık
                        // eksen sıkıştırması burada özellikle uygulanmaz:
                        // ECharts `_renderDataShadow` da `getDataExtent` ile
                        // ham zaman kapsamını doğrusal eşler.
                        let zaman_kapsamı = if dikey {
                            yakınlaştırma
                                .y_eksen_sırası
                                .and_then(|sıra| kurulum.y_eksenler.get(sıra))
                        } else {
                            kurulum.x_eksenler.get(yakınlaştırma.x_eksen_sırası)
                        }
                        .filter(|eksen| eksen.seçenek.tür == EksenTürü::Zaman)
                        .and_then(|_| {
                            let kapsam = (0..veri_sayısı)
                                .filter_map(bu_eksen_değeri_al)
                                .filter(|değer| değer.is_finite())
                                .fold(
                                    [f64::INFINITY, f64::NEG_INFINITY],
                                    |[en_az, en_çok], değer| [en_az.min(değer), en_çok.max(değer)],
                                );
                            (kapsam[0].is_finite()
                                && kapsam[1].is_finite()
                                && kapsam[1] > kapsam[0])
                                .then_some(kapsam)
                        });
                        let dönüştür = |eksen: f32, karşı: f32| {
                            if dikey {
                                // SliderZoomView dikey sürgüyü yerel yatay
                                // koordinatlarda çizer ve grubu +PI/2 ile
                                // döndürür. zrender dönüşümünde bu,
                                // `(u, v) -> (sol + v, alt - u)` olur:
                                // veri sırası aşağıdan yukarı, karşı eksen
                                // değeri soldan sağa ilerler.
                                (şerit.x + karşı, şerit.alt() - eksen)
                            } else {
                                (şerit.x + eksen, şerit.alt() - karşı)
                            }
                        };
                        let taban = |eksen: f32| dönüştür(eksen, 0.0);
                        let mut alan_yolu = crate::cizim::Yol::yeni();
                        alan_yolu.taşı(taban(uzunluk));
                        alan_yolu.çiz(taban(0.0));
                        let mut çizgi_yolu = crate::cizim::Yol::yeni();
                        let mut çizgi_başladı = false;
                        let mut son_boş = false;
                        let mut son_eksen = 0.0;
                        // ECharts büyük veri gölgesinde yaklaşık bir örnek /
                        // eksen pikseli bırakır (`Math.round(count / size[0])`).
                        let adım =
                            ((veri_sayısı as f32 / uzunluk.max(1.0)).round() as usize).max(1);
                        for sıra in 0..veri_sayısı {
                            if sıra % adım != 0 {
                                continue;
                            }
                            let oran = zaman_kapsamı
                                .and_then(|[en_az, en_çok]| {
                                    bu_eksen_değeri_al(sıra)
                                        .filter(|değer| değer.is_finite())
                                        .map(|değer| ((değer - en_az) / (en_çok - en_az)) as f32)
                                })
                                .unwrap_or_else(|| {
                                    if veri_sayısı > 1 {
                                        sıra as f32 / (veri_sayısı - 1) as f32
                                    } else {
                                        0.5
                                    }
                                });
                            let eksen = uzunluk * oran;
                            let değer = karşı_değer_al(sıra).filter(|değer| değer.is_finite());
                            if değer.is_none() && !son_boş && sıra > 0 {
                                alan_yolu.çiz(taban(son_eksen));
                                if çizgi_başladı {
                                    çizgi_yolu.çiz(taban(son_eksen));
                                }
                            } else if değer.is_some() && son_boş {
                                alan_yolu.çiz(taban(eksen));
                                if çizgi_başladı {
                                    çizgi_yolu.çiz(taban(eksen));
                                }
                            }
                            if let Some(değer) = değer {
                                let nokta = dönüştür(eksen, eşle(değer));
                                alan_yolu.çiz(nokta);
                                if çizgi_başladı {
                                    çizgi_yolu.çiz(nokta);
                                } else {
                                    çizgi_yolu.taşı(nokta);
                                    çizgi_başladı = true;
                                }
                            }
                            son_boş = değer.is_none();
                            son_eksen = eksen;
                        }
                        alan_yolu.kapat();
                        let parçalar = if dikey {
                            let seçili_üst = şerit.y + şerit.yükseklik * (1.0 - e);
                            let seçili_alt = şerit.y + şerit.yükseklik * (1.0 - b);
                            [
                                (
                                    Dikdörtgen::yeni(
                                        şerit.x,
                                        şerit.y,
                                        şerit.genişlik,
                                        seçili_üst - şerit.y,
                                    ),
                                    false,
                                ),
                                (
                                    Dikdörtgen::yeni(
                                        şerit.x,
                                        seçili_üst,
                                        şerit.genişlik,
                                        seçili_alt - seçili_üst,
                                    ),
                                    true,
                                ),
                                (
                                    Dikdörtgen::yeni(
                                        şerit.x,
                                        seçili_alt,
                                        şerit.genişlik,
                                        şerit.alt() - seçili_alt,
                                    ),
                                    false,
                                ),
                            ]
                        } else {
                            let seçili_sol = şerit.x + şerit.genişlik * b;
                            let seçili_sağ = şerit.x + şerit.genişlik * e;
                            [
                                (
                                    Dikdörtgen::yeni(
                                        şerit.x,
                                        şerit.y,
                                        seçili_sol - şerit.x,
                                        şerit.yükseklik,
                                    ),
                                    false,
                                ),
                                (
                                    Dikdörtgen::yeni(
                                        seçili_sol,
                                        şerit.y,
                                        seçili_sağ - seçili_sol,
                                        şerit.yükseklik,
                                    ),
                                    true,
                                ),
                                (
                                    Dikdörtgen::yeni(
                                        seçili_sağ,
                                        şerit.y,
                                        şerit.sağ() - seçili_sağ,
                                        şerit.yükseklik,
                                    ),
                                    false,
                                ),
                            ]
                        };
                        for (kırpma, seçili) in parçalar {
                            if kırpma.genişlik <= 0.0 || kırpma.yükseklik <= 0.0 {
                                continue;
                            }
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
                    // Tutamaçlar. ECharts yolu önce kare bir sembol kutusuna
                    // oranı korunarak sığdırır, sonra `handleSize` ile kısa
                    // kenara göre ölçekler. Dikey grupta aynı yol saat yönünün
                    // tersine 90 derece döndürülür.
                    let kısa_kenar = if dikey {
                        şerit.genişlik
                    } else {
                        şerit.yükseklik
                    };
                    let tutamaç_boyutu = yakınlaştırma.tutamaç_boyutu.çöz(kısa_kenar).max(0.0);
                    let merkezler = if dikey {
                        [
                            (şerit.merkez().0, pencere.alt() - 1.0),
                            (şerit.merkez().0, pencere.y + 1.0),
                        ]
                    } else {
                        [
                            (pencere.x + 1.0, şerit.merkez().1),
                            (pencere.sağ() - 1.0, şerit.merkez().1),
                        ]
                    };
                    let (sol, sağ) = if let Some(simge) = &yakınlaştırma.tutamaç_simgesi {
                        let mut tutamacı_çiz = |merkez: (f32, f32)| {
                            let yol =
                                crate::grafik::sembol_yolu(simge, merkez, tutamaç_boyutu, true)
                                    .map(|yol| {
                                        if dikey {
                                            let dönüşüm = AfinMatris::ötele(merkez.0, merkez.1)
                                                .çarp(AfinMatris::döndür(
                                                    -std::f32::consts::FRAC_PI_2,
                                                ))
                                                .çarp(AfinMatris::ötele(-merkez.0, -merkez.1));
                                            yolu_dönüştür(&yol, dönüşüm)
                                        } else {
                                            yol
                                        }
                                    });
                            if let Some(yol) = yol {
                                yüzey.yol_doldur(&yol, &Dolgu::Düz(crate::renk::Renk::BEYAZ));
                                yüzey.yol_çiz(
                                    &yol,
                                    1.0,
                                    crate::renk::Renk::onaltılık(0xc0c9e6),
                                    ÇizgiTürü::Düz,
                                );
                                yol.kesin_sınır_kutusu().unwrap_or_else(|| {
                                    Dikdörtgen::yeni(merkez.0, merkez.1, 0.0, 0.0)
                                })
                            } else {
                                Dikdörtgen::yeni(merkez.0, merkez.1, 0.0, 0.0)
                            }
                        };
                        let [sol_merkez, sağ_merkez] = merkezler;
                        (tutamacı_çiz(sol_merkez), tutamacı_çiz(sağ_merkez))
                    } else {
                        // Öntanımlı handleIcon'ın kaynak sınırı 8×40'tır.
                        // zrender yolu oranı korunarak 2×2 sembol kutusuna,
                        // ardından `handleSize` yüksekliğine ölçekler:
                        // toplam uzunluk 20, yuvarlak gövde 12.5×4 ve iki
                        // uçtaki sap 3.75 birimdir.
                        let ölçek = tutamaç_boyutu / 20.0;
                        let toplam = 20.0 * ölçek;
                        let uzun = 12.5 * ölçek;
                        let kısa = 4.0 * ölçek;
                        let (sol, sağ) = if dikey {
                            (
                                Dikdörtgen::yeni(
                                    merkezler[0].0 - uzun / 2.0,
                                    merkezler[0].1 - kısa / 2.0,
                                    uzun,
                                    kısa,
                                ),
                                Dikdörtgen::yeni(
                                    merkezler[1].0 - uzun / 2.0,
                                    merkezler[1].1 - kısa / 2.0,
                                    uzun,
                                    kısa,
                                ),
                            )
                        } else {
                            (
                                Dikdörtgen::yeni(
                                    merkezler[0].0 - kısa / 2.0,
                                    merkezler[0].1 - uzun / 2.0,
                                    kısa,
                                    uzun,
                                ),
                                Dikdörtgen::yeni(
                                    merkezler[1].0 - kısa / 2.0,
                                    merkezler[1].1 - uzun / 2.0,
                                    kısa,
                                    uzun,
                                ),
                            )
                        };
                        for t in [sol, sağ] {
                            yüzey.dikdörtgen(
                                t,
                                &Dolgu::Düz(crate::renk::Renk::BEYAZ),
                                [1.0 * ölçek; 4],
                                Some((1.0, crate::renk::Renk::onaltılık(0xc0c9e6))),
                            );
                        }
                        if dikey {
                            let başlangıç = şerit.merkez().0 - toplam / 2.0;
                            let bitiş = şerit.merkez().0 + toplam / 2.0;
                            for kutu in [sol, sağ] {
                                let merkez_y = kutu.merkez().1;
                                yüzey.çizgi(
                                    (başlangıç, merkez_y),
                                    (kutu.x, merkez_y),
                                    1.0,
                                    crate::renk::Renk::onaltılık(0xc0c9e6),
                                    ÇizgiTürü::Düz,
                                );
                                yüzey.çizgi(
                                    (kutu.sağ(), merkez_y),
                                    (bitiş, merkez_y),
                                    1.0,
                                    crate::renk::Renk::onaltılık(0xc0c9e6),
                                    ÇizgiTürü::Düz,
                                );
                            }
                        } else {
                            let başlangıç = şerit.merkez().1 - toplam / 2.0;
                            let bitiş = şerit.merkez().1 + toplam / 2.0;
                            for kutu in [sol, sağ] {
                                let merkez_x = kutu.merkez().0;
                                yüzey.çizgi(
                                    (merkez_x, başlangıç),
                                    (merkez_x, kutu.y),
                                    1.0,
                                    crate::renk::Renk::onaltılık(0xc0c9e6),
                                    ÇizgiTürü::Düz,
                                );
                                yüzey.çizgi(
                                    (merkez_x, kutu.alt()),
                                    (merkez_x, bitiş),
                                    1.0,
                                    crate::renk::Renk::onaltılık(0xc0c9e6),
                                    ÇizgiTürü::Düz,
                                );
                            }
                        }
                        (sol, sağ)
                    };
                    // Alt/sağ taşıma tutamacı (`brushSelect`).
                    let taşıma = if dikey {
                        Dikdörtgen::yeni(şerit.sağ() - 0.5, pencere.y, 7.0, pencere.yükseklik)
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

    // Radar bileşenleri z=0 katmanında, bütün radar serilerinden önce ve
    // her `radarIndex` için yalnız bir kez çizilir.
    let radar_alanı = Dikdörtgen::yeni(0.0, 0.0, yüzey.genişlik(), yüzey.yükseklik());
    let radar_düzenleri = seçenekler
        .tüm_radarlar()
        .enumerate()
        .map(|(radar_sırası, koordinat)| {
            let bağlı_seriler = seçenekler
                .seriler
                .iter()
                .filter_map(|seri| match seri {
                    Seri::Radar(radar)
                        if radar.radar_sırası == radar_sırası
                            && ad_görünür(seri.ad(), kapalı) =>
                    {
                        Some(radar)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let düzen = radar_düzeni_serilerle(koordinat, radar_alanı, &bağlı_seriler);
            radar_ağı_çiz(yüzey, koordinat, &düzen);
            (koordinat, düzen)
        })
        .collect::<Vec<_>>();

    // 4b) Görsel eşleme bileşenleri (gradyan çubukları). ECharts
    // `visualMap: []` dizisinin her üyesini çizer; ilk üyenin isabet alanı
    // geriye uyumlu tekil etkileşim alanında korunur.
    for (eşleme_sırası, eşleme) in seçenekler.tüm_görsel_eşlemeler().enumerate() {
        let veri_kapsamı = seçenekler
            .seriler
            .iter()
            .enumerate()
            .filter(|(seri_sırası, _)| eşleme.seriye_uygulanır_mı(*seri_sırası))
            .find_map(|(_, s)| match s {
                Seri::Isı(ısı) => Some(ısı_değer_kapsamı(ısı)),
                Seri::Takvim(takvim) => Some(takvim_değer_kapsamı(takvim)),
                Seri::Saçılım(saçılım) => Some(saçılım_görsel_kapsamı(saçılım, eşleme)),
                Seri::Radar(_) => Some(radar_görsel_kapsamı(seçenekler, eşleme)),
                _ => None,
            })
            .unwrap_or([0.0, 1.0]);
        let eşleme_çıktısı = görsel_eşleme_çiz(yüzey, eşleme, eşleme.kapsam_çöz(veri_kapsamı));
        if eşleme_sırası == 0 {
            çıktı.eşleme_kutuları = eşleme_çıktısı.parça_kutuları;
            çıktı.sürekli_eşleme = eşleme_çıktısı.sürekli;
        }
    }

    // 4c) Kutupsal koordinatlar ve `polarIndex` ile bağlı seriler. Her
    // koordinat kendi kapsam/yığın hesabını taşır; tüm alt eksen katmanları
    // serilerden önce, üst eksen katmanları serilerden sonra çizilir.
    let temel_görünürler: Vec<bool> = seçenekler
        .seriler
        .iter()
        .map(|s| ad_görünür(s.ad(), kapalı))
        .collect();
    let mut kutupsal_düzenleri = Vec::new();
    for (kutupsal_sırası, koordinat) in seçenekler.tüm_kutupsallar().enumerate() {
        let görünürler: Vec<bool> = seçenekler
            .seriler
            .iter()
            .enumerate()
            .map(|(seri_sırası, seri)| {
                temel_görünürler[seri_sırası] && seri.kutupsal_sırası() == Some(kutupsal_sırası)
            })
            .collect();
        if !görünürler.iter().any(|görünür| *görünür) {
            continue;
        }
        let aralıklar = yığın_aralıkları(&seçenekler.seriler, &görünürler);
        let düzen = kutupsal_kur(
            koordinat,
            seçenekler,
            &aralıklar,
            &görünürler,
            Dikdörtgen::yeni(0.0, 0.0, yüzey.genişlik(), yüzey.yükseklik()),
        );
        kutupsal_düzenleri.push((koordinat, görünürler, aralıklar, düzen));
    }
    for (koordinat, _, _, düzen) in &kutupsal_düzenleri {
        kutupsal_ağ_çiz(yüzey, koordinat, düzen, false);
    }
    for (_, görünürler, aralıklar, düzen) in &kutupsal_düzenleri {
        let isabet_başlangıcı = çıktı.isabetler.len();
        kutupsal_serileri_çiz(
            yüzey,
            seçenekler,
            düzen,
            aralıklar,
            görünürler,
            kapalı,
            ilerleme,
            zaman_sn,
            &mut çıktı.isabetler,
        );
        if let Some(ipucu) = ipucu_seçeneği.as_ref()
            && let Some(bekleyen) = kutupsal_ipucu_hazırla(
                seçenekler,
                düzen,
                ipucu,
                fare,
                girdi.ipucu_öğesi,
                &çıktı.isabetler[isabet_başlangıcı..],
            )
        {
            bekleyen_ipucu = Some(bekleyen);
        }
    }
    for (koordinat, _, _, düzen) in &kutupsal_düzenleri {
        kutupsal_ağ_çiz(yüzey, koordinat, düzen, true);
    }

    // 4d) Matrix'e doğrudan bağlı heatmap/scatter/graph/custom serileri.
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        if !ad_görünür(seri.ad(), kapalı) {
            continue;
        }
        match seri {
            Seri::Isı(ısı) => {
                let Some(matris_sırası) = ısı.matris_sırası else {
                    continue;
                };
                let Some(Some(yerleşim)) = matris_yerleşimleri.get(matris_sırası) else {
                    continue;
                };
                let eşleme = seçenekler
                    .seri_görsel_eşlemesi(i)
                    .cloned()
                    .unwrap_or_default();
                let kapsam = eşleme.kapsam_çöz(ısı_değer_kapsamı(ısı));
                let vurgulu = matris_ısı_haritası_çiz(
                    yüzey,
                    ısı,
                    i,
                    yerleşim,
                    &eşleme,
                    kapsam,
                    ilerleme,
                    fare,
                    &mut çıktı.isabetler,
                );
                if let (Some(veri_sırası), Some(f), Some(ipucu)) =
                    (vurgulu, fare, ipucu_seçeneği.as_ref())
                    && ipucu.tetikleme == Tetikleme::Öğe
                    && let Some(öğe) = ısı.veri.get(veri_sırası)
                    && let Some(değer) = öğe.değer.dizi().and_then(|dizi| dizi.get(2)).copied()
                {
                    bekleyen_ipucu = Some((
                        ısı.ad.clone(),
                        vec![İpucuSatırı {
                            im_rengi: Some(eşleme.renk_çöz(değer, kapsam)),
                            ad: öğe.ad.clone().unwrap_or_default(),
                            değer: binlik_ayır(değer),
                        }],
                        f,
                    ));
                }
            }
            Seri::Saçılım(saçılım) => {
                let Some(matris_sırası) = saçılım.matris_sırası else {
                    continue;
                };
                let Some(Some(yerleşim)) = matris_yerleşimleri.get(matris_sırası) else {
                    continue;
                };
                let eşlemeler = seçenekler.seri_görsel_eşlemeleri(i).collect::<Vec<_>>();
                if let Some(ipucu) = matris_saçılım_serisini_çiz(
                    yüzey,
                    saçılım,
                    i,
                    yerleşim,
                    seçenekler.seri_rengi(i),
                    &seçenekler.palet,
                    &eşlemeler,
                    ilerleme,
                    zaman_sn,
                    ipucu_seçeneği.as_ref(),
                    fare,
                    &mut çıktı.isabetler,
                ) {
                    bekleyen_ipucu = Some(ipucu);
                }
            }
            Seri::Grafo(grafo) => {
                let Some(matris_sırası) = grafo.matris_sırası else {
                    continue;
                };
                let Some(Some(yerleşim)) = matris_yerleşimleri.get(matris_sırası) else {
                    continue;
                };
                if let Some(ipucu) = grafo_serisini_çiz(
                    yüzey,
                    grafo,
                    i,
                    yerleşim.dış_kutu,
                    seçenekler,
                    ilerleme,
                    girdi.grafo_görünümü,
                    &girdi.grafo_kaymaları,
                    None,
                    Some(yerleşim),
                    ipucu_seçeneği.as_ref(),
                    fare,
                    &mut çıktı.isabetler,
                ) {
                    bekleyen_ipucu = Some(ipucu);
                }
            }
            Seri::Özel(özel) => {
                let Some(matris_sırası) = özel.matris_sırası else {
                    continue;
                };
                let Some(Some(yerleşim)) = matris_yerleşimleri.get(matris_sırası) else {
                    continue;
                };
                if let Some(çizim) = &özel.çizim {
                    let bağlam = ÖzelBağlam {
                        alan: yerleşim.dış_kutu,
                        kartezyen: None,
                        takvim: None,
                        matris: Some(yerleşim),
                        veri: &özel.veri,
                        renk: seçenekler.seri_rengi(i),
                        ilerleme,
                    };
                    çizim(yüzey, &bağlam);
                }
            }
            _ => {}
        }
    }

    // 4e) Calendar ve matrix üzerindeki çekirdek (GL olmayan) lines.
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
                let Some(Some(yerleşim)) = matris_yerleşimleri.get(hatlar.matris_sırası) else {
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
        let koordinat_merkezi = if let Some(matris_sırası) = p.matris_sırası {
            let Some((x, y)) = &p.matris_merkezi else {
                continue;
            };
            let Some(Some(yerleşim)) = matris_yerleşimleri.get(matris_sırası) else {
                continue;
            };
            let Some(merkez) = yerleşim.veriden_noktaya(x.clone(), y.clone()) else {
                continue;
            };
            Some(merkez)
        } else {
            match p.takvim_sırası {
                Some(takvim_sırası) => {
                    let Some(tarih) = p.takvim_merkez_tarihi else {
                        continue;
                    };
                    let Some(Some(yerleşim)) = takvim_yerleşimleri.get(takvim_sırası) else {
                        continue;
                    };
                    let Some(merkez) = yerleşim.veriden_noktaya(tarih) else {
                        continue;
                    };
                    Some(merkez)
                }
                None => None,
            }
        };
        let dilimler: Vec<Dilim> =
            pasta_yerleşimi_merkezle(p, seçenekler, tüm_alan, kapalı, ilerleme, koordinat_merkezi);
        if dilimler.is_empty() {
            boş_pasta_çiz_merkezle(yüzey, p, tüm_alan, koordinat_merkezi);
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
                // Hover emphasis tooltip'ten bağımsızdır; tooltip kapalıyken
                // de ECharts öğe durumu çalışmaya devam eder.
                let vurgu = fare
                    .and_then(|f| dilimler.iter().find(|d| d.içeriyor_mu(f)).map(|d| d.sıra))
                    .or_else(|| {
                        girdi
                            .ipucu_öğesi
                            .filter(|(seri_sırası, _)| *seri_sırası == i)
                            .map(|(_, veri_sırası)| veri_sırası)
                    });
                huni_çiz(
                    yüzey,
                    h,
                    i,
                    &dilimler,
                    vurgu,
                    ilerleme,
                    &mut çıktı.isabetler,
                );
                if let Some(dilim) = vurgu.and_then(|v| dilimler.iter().find(|d| d.sıra == v)) {
                    bekleyen_ipucu = Some((
                        seri.ad().map(str::to_string),
                        vec![İpucuSatırı {
                            im_rengi: Some(dilim.dolgu.temsilî()),
                            ad: dilim.ad.clone(),
                            değer: binlik_ayır(dilim.değer),
                        }],
                        fare.unwrap_or_else(|| dilim.sınır_kutusu().merkez()),
                    ));
                }
            }
            Seri::GöstergeSaati(g) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let veri_paleti = |sıra: usize| seçenekler.palet_rengi(sıra);
                gösterge_saati_çiz(
                    yüzey,
                    g,
                    i,
                    &veri_paleti,
                    tüm_alan,
                    ilerleme,
                    &mut çıktı.isabetler,
                );
            }
            Seri::Radar(r) => {
                if !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                let Some((koordinat, düzen)) = radar_düzenleri.get(r.radar_sırası) else {
                    continue;
                };
                if koordinat.göstergeler.len() < 3 {
                    continue;
                }
                let vurgu = if r.sessiz || koordinat.sessiz {
                    None
                } else {
                    fare.and_then(|fare| radar_vurgusu(r, düzen, kapalı, ilerleme, fare))
                        .or_else(|| {
                            girdi
                                .ipucu_öğesi
                                .filter(|(seri_sırası, _)| *seri_sırası == i)
                                .map(|(_, veri_sırası)| veri_sırası)
                        })
                };
                radar_serisi_çiz(
                    yüzey,
                    r,
                    i,
                    koordinat,
                    düzen,
                    seçenekler,
                    kapalı,
                    ilerleme,
                    vurgu,
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
                // Scatter sembol/etiketleri z2=100'dür; heatmap ile takvim
                // ayırıcı/etiketlerinden sonra katman geçişinde çizilir.
                continue;
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
                let Some(yerleşim) = tek_eksen_yerleşimleri.get(s.tek_eksen_sırası) else {
                    continue;
                };
                let önce = çıktı.isabetler.len();
                let palet = |sıra: usize| seçenekler.palet_rengi(sıra);
                tema_nehri_çiz(
                    yüzey,
                    s,
                    i,
                    yerleşim,
                    &palet,
                    ilerleme,
                    &|ad| !kapalı.contains(ad),
                    fare,
                    girdi
                        .ipucu_öğesi
                        .filter(|(seri_sırası, _)| *seri_sırası == i)
                        .map(|(_, veri_sırası)| veri_sırası),
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
                if g.matris_sırası.is_some()
                    || (g.takvim_sırası.is_some() && g.z > 2)
                    || !ad_görünür(seri.ad(), kapalı)
                {
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
                    None,
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
                if s.matris_sırası.is_some() || !ad_görünür(seri.ad(), kapalı) {
                    continue;
                }
                if let Some(çizim) = &s.çizim {
                    let (alan, takvim) = if let Some(takvim_sırası) = s.takvim_sırası {
                        let Some(Some(yerleşim)) = takvim_yerleşimleri.get(takvim_sırası)
                        else {
                            continue;
                        };
                        (yerleşim.gövde_kutusu, Some(yerleşim))
                    } else {
                        (tüm_alan, None)
                    };
                    let bağlam = ÖzelBağlam {
                        alan,
                        kartezyen: None,
                        takvim,
                        matris: None,
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

    // Aynı tuval katmanındaki calendar scatter sembol ve etiketleri,
    // CalendarView z2=20/30 üst katmanının üzerinde yer alır.
    for (seri_sırası, seri) in seçenekler.seriler.iter().enumerate() {
        let Seri::Saçılım(saçılım) = seri else {
            continue;
        };
        if saçılım.z_seviyesi > 0
            || saçılım.takvim_sırası.is_none()
            || !ad_görünür(seri.ad(), kapalı)
        {
            continue;
        }
        let takvim_sırası = saçılım.takvim_sırası.unwrap_or(0);
        let Some(Some(yerleşim)) = takvim_yerleşimleri.get(takvim_sırası) else {
            continue;
        };
        let görsel_eşlemeler = seçenekler
            .seri_görsel_eşlemeleri(seri_sırası)
            .collect::<Vec<_>>();
        if let Some(ipucu) = takvim_saçılım_serisini_çiz(
            yüzey,
            saçılım,
            seri_sırası,
            yerleşim,
            seçenekler.seri_rengi(seri_sırası),
            &seçenekler.palet,
            &görsel_eşlemeler,
            ilerleme,
            zaman_sn,
            ipucu_seçeneği.as_ref(),
            fare,
            &mut çıktı.isabetler,
        ) {
            bekleyen_ipucu = Some(ipucu);
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
            None,
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
        let görsel_eşlemeler = seçenekler
            .seri_görsel_eşlemeleri(seri_sırası)
            .collect::<Vec<_>>();
        if let Some(ipucu) = takvim_saçılım_serisini_çiz(
            yüzey,
            saçılım,
            seri_sırası,
            yerleşim,
            seçenekler.seri_rengi(seri_sırası),
            &seçenekler.palet,
            &görsel_eşlemeler,
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

    // 5c) Zaman şeridi (timeline) — option modeli varsa ECharts slider
    // yerleşimi; yalnız `GrafikGörünümü::film` kullanılmışsa geriye uyumlu
    // yalın alt şerit.
    if let Some(zaman_şeridi) = &seçenekler.zaman_şeridi {
        çıktı.zaman_düğmeleri =
            seçenekli_zaman_şeridi_çiz(yüzey, zaman_şeridi, girdi.zaman_şeridi);
    } else if let Some((geçerli, toplam, oynuyor)) = girdi.zaman_şeridi {
        çıktı.zaman_düğmeleri = zaman_şeridi_çiz(yüzey, geçerli, toplam, oynuyor);
    }

    // 5d) Fırça seçimi kaplaması.
    if let Some([x0, y0, x1, y1]) = girdi.fırça {
        let alan = FırçaAlanı::Dikdörtgen {
            başlangıç: (x0, y0),
            bitiş: (x1, y1),
        };
        fırça_alanını_çiz(
            yüzey,
            &alan,
            seçenekler.fırça.as_ref().map(|fırça| &fırça.stil),
        );
    }
    for alan in &girdi.fırça_alanları {
        fırça_alanını_çiz(
            yüzey,
            alan,
            seçenekler.fırça.as_ref().map(|fırça| &fırça.stil),
        );
    }
    for alan in &hazır_fırça.alanlar {
        fırça_alanını_çiz(
            yüzey,
            alan,
            seçenekler.fırça.as_ref().map(|fırça| &fırça.stil),
        );
    }

    // 5e) Serbest `graphic` bileşeni. Sahnenin aynısı çıktı içinde
    // korunur; gpui tıklama sınamasında ikinci bir geometri üretmez.
    if let Some(grafik) = &seçenekler.grafik {
        let sahne = grafik_sahnesi_hazırla(grafik, yüzey.genişlik(), yüzey.yükseklik());
        sahne.sahne.çiz(yüzey);
        çıktı.grafik_sahnesi = Some(sahne);
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
    fn sutun_taban_cizgisi_bar_dolgusundan_sonra_yeniden_cizilir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .x_ekseni(Eksen::değer())
            .y_ekseni(Eksen::kategori().veri(["A"]))
            .seri(crate::model::seri::SütunSerisi::yeni().veri([10.0]));
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(600.0, 450.0);

        grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());

        let bar_sırası = yüzey
            .komutlar
            .iter()
            .position(|komut| komut.starts_with("dikdörtgen "))
            .expect("bar dikdörtgeni");
        let önceki_çizgiler = yüzey.komutlar[..bar_sırası]
            .iter()
            .filter(|komut| komut.starts_with("çiz "))
            .collect::<HashSet<_>>();
        assert!(
            yüzey.komutlar[bar_sırası + 1..]
                .iter()
                .any(|komut| önceki_çizgiler.contains(komut)),
            "kategori axisLine barın üstünde ikinci kez bulunmalı:\n{}",
            yüzey.döküm()
        );
    }

    #[test]
    fn polar_item_tooltip_silent_yigini_atlayip_html_satirlarini_cozer() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .kutupsal(
                crate::model::kutupsal::KutupsalKoordinat::yeni()
                    .açısal_eksen(Eksen::kategori().veri(["Beijing"]))
                    .radyal_eksen(Eksen::değer().en_çok(100.0)),
            )
            .ipucu(İpucu::yeni().bağlamlı_biçimleyici(|parametreler| {
                let ad = parametreler
                    .first()
                    .map(|parametre| parametre.ad.as_str())
                    .unwrap_or("");
                format!("{ad}<br>Lowest：50<br>Highest：100")
            }))
            .seri(
                crate::model::seri::SütunSerisi::yeni()
                    .kutupsal(true)
                    .yığın("range")
                    .sessiz(true)
                    .veri([50.0]),
            )
            .seri(
                crate::model::seri::SütunSerisi::yeni()
                    .ad("Range")
                    .kutupsal(true)
                    .yığın("range")
                    .veri([50.0]),
            );
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);
        let çıktı = grafiği_boya(
            &mut yüzey,
            &seçenekler,
            &BoyamaGirdisi {
                ipucu_öğesi: Some((1, 0)),
                ..Default::default()
            },
        );

        assert_eq!(çıktı.isabetler.len(), 1, "silent placeholder atlanmalı");
        assert_eq!(çıktı.isabetler[0].seri_sırası, 1);
        let döküm = yüzey.döküm();
        assert!(döküm.contains("yazı \"Beijing\""));
        assert!(döküm.contains("yazı \"Lowest：50\""));
        assert!(döküm.contains("yazı \"Highest：100\""));
    }

    #[test]
    fn sabit_boyutlu_grid_acik_sag_ve_alt_kenarlara_capanir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .ızgara_ekle(
                crate::model::bilesen::Izgara::yeni()
                    .sağ("7%")
                    .alt("7%")
                    .genişlik("38%")
                    .yükseklik("38%"),
            )
            .x_ekseni(Eksen::değer())
            .y_ekseni(Eksen::değer())
            .seri(crate::model::seri::SaçılımSerisi::yeni().veri([[1.0, 2.0]]));
        let yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);

        let kurulum = kartezyen_kur(&yüzey, &seçenekler, &HashSet::new())
            .expect("kartezyen kurulum üretilmeli");
        let alan = kurulum.ızgara_alanları[0];

        assert!((alan.x - 385.0).abs() < 1e-4, "{alan:?}");
        assert!((alan.y - 288.75).abs() < 1e-4, "{alan:?}");
        assert!((alan.genişlik - 266.0).abs() < 1e-4, "{alan:?}");
        assert!((alan.yükseklik - 199.5).abs() < 1e-4, "{alan:?}");
    }

    #[test]
    fn contain_label_gizli_ve_icerideki_eksen_etiketlerine_alan_ayirmaz() {
        let gizli = GrafikSeçenekleri::yeni()
            .ızgara_ekle(
                crate::model::bilesen::Izgara::yeni()
                    .sol(10)
                    .sağ(10)
                    .üst(10)
                    .alt(10)
                    .etiketi_kapsa(true),
            )
            .x_ekseni(
                Eksen::değer().etiket(crate::model::eksen::EksenEtiketi::yeni().göster(false)),
            )
            .y_ekseni(
                Eksen::değer().etiket(crate::model::eksen::EksenEtiketi::yeni().içeride(true)),
            )
            .seri(crate::model::seri::SaçılımSerisi::yeni().veri([[1.0, 2.0]]));
        let yüzey = crate::cizim::KayıtYüzeyi::yeni(200.0, 100.0);

        let kurulum =
            kartezyen_kur(&yüzey, &gizli, &HashSet::new()).expect("kartezyen kurulum üretilmeli");
        let alan = kurulum.ızgara_alanları[0];

        assert!((alan.x - 10.0).abs() < 1e-4, "{alan:?}");
        assert!((alan.y - 10.0).abs() < 1e-4, "{alan:?}");
        assert!((alan.genişlik - 180.0).abs() < 1e-4, "{alan:?}");
        assert!((alan.yükseklik - 80.0).abs() < 1e-4, "{alan:?}");
    }

    #[test]
    fn olcekli_deger_ekseni_sutun_tabanini_veri_kapsamina_katmaz() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .x_ekseni(Eksen::kategori().veri(["A", "B"]))
            .y_ekseni(Eksen::değer().ölçekli(true).bölme_sayısı(2))
            .seri(crate::model::seri::SütunSerisi::yeni().veri([5_401_538.0, 12_204_426.0]));
        let yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);

        let kurulum = kartezyen_kur(&yüzey, &seçenekler, &HashSet::new())
            .expect("kartezyen kurulum üretilmeli");

        assert_eq!(
            kurulum.y_eksenler[0].ölçek.kapsam(),
            [3_000_000.0, 15_000_000.0]
        );
    }

    #[test]
    fn sessiz_scatter_çizilir_ama_isabet_bölgesi_üretmez() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .x_ekseni(Eksen::değer())
            .y_ekseni(Eksen::değer())
            .seri(
                crate::model::seri::SaçılımSerisi::yeni()
                    .sessiz(true)
                    .veri([[1.0, 2.0]]),
            );
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);

        let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());

        assert!(çıktı.isabetler.is_empty());
        let döküm = yüzey.döküm();
        assert!(döküm.contains("#5070dd@0.8"), "{döküm}");
    }

    #[test]
    fn bar_emphasis_faredeki_sutuna_uygulanir_ve_yuksek_z_inside_etiket_uste_cizilir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .x_ekseni(
                Eksen::kategori()
                    .veri(["A"])
                    .z(10)
                    .çizgi(crate::model::eksen::EksenÇizgisi::yeni().göster(false))
                    .çentik(crate::model::eksen::EksenÇentiği::yeni().göster(false))
                    .etiket(
                        crate::model::eksen::EksenEtiketi::yeni()
                            .içeride(true)
                            .yazı(crate::model::stil::YazıStili::yeni().renk(0xffffff)),
                    ),
            )
            .y_ekseni(Eksen::değer().göster(false))
            .seri(
                crate::model::seri::SütunSerisi::yeni()
                    .öğe_stili(crate::model::stil::ÖğeStili::yeni().renk(0xff0000))
                    .vurgu_öğe_stili(crate::model::stil::ÖğeStili::yeni().renk(0x0000ff))
                    .veri([100]),
            );
        let mut normal = crate::cizim::KayıtYüzeyi::yeni(200.0, 200.0);
        let çıktı = grafiği_boya(&mut normal, &seçenekler, &BoyamaGirdisi::default());
        let İsabetGeometrisi::Dikdörtgen(sütun) = çıktı.isabetler[0].geometri else {
            panic!("sütun dikdörtgen isabeti bekleniyordu");
        };
        let normal_döküm = normal.döküm();
        let sütun_sırası = normal_döküm
            .find("#ff0000@1.0")
            .expect("normal sütun dolgusu");
        let etiket_sırası = normal_döküm.find("yazı \"A\"").expect("x etiketi");
        assert!(etiket_sırası > sütun_sırası, "{normal_döküm}");

        let mut vurgulu = crate::cizim::KayıtYüzeyi::yeni(200.0, 200.0);
        grafiği_boya(
            &mut vurgulu,
            &seçenekler,
            &BoyamaGirdisi {
                fare: Some(sütun.merkez()),
                ..BoyamaGirdisi::default()
            },
        );
        let vurgu_dökümü = vurgulu.döküm();
        assert!(vurgu_dökümü.contains("#0000ff@1.0"), "{vurgu_dökümü}");
        assert!(!vurgu_dökümü.contains("#ff0000@1.0"), "{vurgu_dökümü}");
    }

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
    fn data_min_max_ham_veri_sınırlarını_güzel_kapsama_kilitler() {
        let güzel = ölçek_kur(
            &Eksen::değer().ölçekli(true).bölme_sayısı(5),
            Vec::new(),
            [1007.0, 1925.0],
        );
        assert!(güzel.kapsam()[1] > 1925.0, "{:?}", güzel.kapsam());

        let veri_sınırlı = ölçek_kur(
            &Eksen::değer()
                .ölçekli(true)
                .bölme_sayısı(5)
                .en_az_veri()
                .en_çok_veri()
                .sayısal_kenar_boşluğu("20%", "30%"),
            Vec::new(),
            [1007.0, 1925.0],
        );

        assert_eq!(veri_sınırlı.kapsam(), [1007.0, 1925.0]);
    }

    #[test]
    fn kirik_deger_ekseni_nice_adimini_etkin_acikliktan_hesaplar() {
        let ölçek = ölçek_kur(
            &Eksen::değer().kırılma(
                crate::model::eksen::EksenKırılması::yeni(5_000.0, 100_000.0).boşluk("2%"),
            ),
            Vec::new(),
            [900.0, 107_022.0],
        );
        let Ölçek::Aralık(ölçek) = ölçek else {
            panic!("aralık ölçeği bekleniyordu");
        };

        assert_eq!(ölçek.adım, 2_000.0);
        assert_eq!(ölçek.kapsam, [0.0, 108_000.0]);
    }

    #[test]
    fn değer_ekseni_açık_interval_ile_sabit_adımlı_çentikler_üretir() {
        let ölçek = ölçek_kur(
            &Eksen::değer().en_az(0.0).en_çok(360.0).aralık(60.0),
            Vec::new(),
            [0.0, 359.0],
        );

        assert_eq!(ölçek.kapsam(), [0.0, 360.0]);
        assert_eq!(
            ölçek
                .çentikler()
                .into_iter()
                .map(|çentik| çentik.değer)
                .collect::<Vec<_>>(),
            vec![0.0, 60.0, 120.0, 180.0, 240.0, 300.0, 360.0]
        );
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
    fn dikey_datazoom_golgesi_zrender_donusum_yonunu_izler() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .x_ekseni(Eksen::değer())
            .y_ekseni(Eksen::değer())
            .veri_yakınlaştırma(
                crate::model::yakinlastirma::VeriYakınlaştırma::sürgü()
                    .y_eksen_sırası(0)
                    .sol(10)
                    .üst(20)
                    .genişlik(20)
                    .yükseklik(100),
            )
            .seri(crate::model::seri::SaçılımSerisi::yeni().veri([
                [0.0, 0.0],
                [1.0, 10.0],
                [2.0, 0.0],
            ]));
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(200.0, 200.0);

        grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());

        let döküm = yüzey.döküm();
        assert!(
            döküm.contains("çiz #8292cc@1.0 k=0.5 düz | T(14.3,121.8) Ç(20.5,71.8) Ç(26.8,21.8)"),
            "{döküm}"
        );
    }

    #[test]
    fn candlestick_datazoom_golgesi_acilis_boyutunu_kullanir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .x_ekseni(Eksen::kategori().veri(["A", "B", "C"]))
            .y_ekseni(Eksen::değer().ölçekli(true))
            .veri_yakınlaştırma(
                crate::model::yakinlastirma::VeriYakınlaştırma::sürgü()
                    .sol(20)
                    .alt(10)
                    .genişlik(120)
                    .yükseklik(30),
            )
            .seri(crate::model::seri::MumSerisi::yeni().veri([
                [1.0, 2.0, 0.0, 3.0],
                [10.0, 9.0, 8.0, 11.0],
                [1.0, 2.0, 0.0, 3.0],
            ]));
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(200.0, 200.0);

        grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());

        let döküm = yüzey.döküm();
        assert!(
            döküm
                .contains("çiz #8292cc@1.0 k=0.5 düz | T(22.8,190.9) Ç(82.8,172.1) Ç(142.8,190.9)"),
            "{döküm}"
        );
    }

    #[test]
    fn boundary_gap_kapali_mum_datazoom_eslemesini_yarim_bant_genisletir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .x_ekseni(
                Eksen::kategori()
                    .veri(["A", "B", "C", "D"])
                    .kenar_boşluğu(false),
            )
            .y_ekseni(Eksen::değer())
            .veri_yakınlaştırma(
                crate::model::yakinlastirma::VeriYakınlaştırma::iç().aralık(50.0, 100.0),
            )
            .seri(crate::model::seri::MumSerisi::yeni().veri([
                [1.0, 2.0, 0.0, 3.0],
                [2.0, 3.0, 1.0, 4.0],
                [3.0, 4.0, 2.0, 5.0],
                [4.0, 5.0, 3.0, 6.0],
            ]));
        let yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);

        let kurulum = kartezyen_kur(&yüzey, &seçenekler, &HashSet::new())
            .expect("kartezyen kurulum üretilmeli");
        let eksen = &kurulum.x_eksenler[0];

        assert_eq!(eksen.pencere, Some((1.5, 3.5)));
        assert!(!eksen.veri_penceresinde_mi(1.0));
        assert!(eksen.veri_penceresinde_mi(2.0));
    }

    #[test]
    fn mum_gosterge_simgesi_yukselen_kenarligini_tasir() {
        let seçenekler = GrafikSeçenekleri::yeni().seri(
            crate::model::seri::MumSerisi::yeni()
                .ad("日K")
                .yükselen_renk(0xec0000)
                .yükselen_kenarlık_rengi(0x8a0000)
                .veri([[1.0, 2.0, 0.0, 3.0]]),
        );

        let öğeler = gösterge_öğeleri(&seçenekler, &HashSet::new());

        assert_eq!(öğeler.len(), 1);
        assert_eq!(öğeler[0].renk, Renk::onaltılık(0xec0000));
        assert_eq!(öğeler[0].kenarlık, Some((1.0, Renk::onaltılık(0x8a0000))));
    }

    #[test]
    fn sutun_gosterge_simgesi_item_style_opaklik_ve_kenarligini_tasir() {
        let seçenekler = GrafikSeçenekleri::yeni().seri(
            crate::model::seri::SütunSerisi::yeni()
                .ad("With Round Cap")
                .öğe_stili(
                    crate::model::stil::ÖğeStili::yeni()
                        .opaklık(0.8)
                        .kenarlık_rengi("green")
                        .kenarlık_kalınlığı(1.0),
                )
                .veri([1]),
        );

        let öğeler = gösterge_öğeleri(&seçenekler, &HashSet::new());

        assert_eq!(öğeler.len(), 1);
        assert_eq!(öğeler[0].opaklık, 0.8);
        assert_eq!(öğeler[0].kenarlık, Some((1.0, Renk::from("green"))));
    }

    #[test]
    fn özel_datazoom_tutamacı_yüzde_boyutunu_ve_dikey_dönüşü_korur() {
        let simge = crate::model::seri::Sembol::svg_yolu("path://M0 0H10V20H0Z")
            .expect("özel tutamaç yolu çözülmeli");
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .ızgara(crate::model::bilesen::Izgara::yeni().sağ(70).alt(70))
            .x_ekseni(Eksen::değer())
            .y_ekseni(Eksen::değer())
            .veri_yakınlaştırma(
                crate::model::yakinlastirma::VeriYakınlaştırma::sürgü()
                    .veri_gölgesi(false)
                    .tutamaç_simgesi(simge.clone())
                    .tutamaç_boyutu("80%"),
            )
            .veri_yakınlaştırma(
                crate::model::yakinlastirma::VeriYakınlaştırma::sürgü()
                    .y_eksen_sırası(0)
                    .veri_gölgesi(false)
                    .tutamaç_simgesi(simge)
                    .tutamaç_boyutu("80%"),
            )
            .seri(crate::model::seri::SaçılımSerisi::yeni().veri([[0.0, 0.0]]));
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);

        let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());
        let yatay = çıktı.sürgüler.first().expect("yatay sürgü");
        let dikey = çıktı.sürgüler.get(1).expect("dikey sürgü");

        assert!((yatay.sol_tutamaç.genişlik - 12.0).abs() < 1e-3);
        assert!((yatay.sol_tutamaç.yükseklik - 24.0).abs() < 1e-3);
        assert!((dikey.sol_tutamaç.genişlik - 24.0).abs() < 1e-3);
        assert!((dikey.sol_tutamaç.yükseklik - 12.0).abs() < 1e-3);
        assert!(
            (dikey.şerit.y - 66.764_27).abs() < 1e-3,
            "{:?}",
            dikey.şerit
        );
        assert!((dikey.şerit.yükseklik - 390.0).abs() < 1e-3);
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
                            "{}:{}:{}",
                            parametre.veri_sırası,
                            parametre.değer.sayı().unwrap_or_default(),
                            parametre
                                .boyut("schema")
                                .and_then(|değer| match değer {
                                    crate::model::deger::VeriDeğeri::Metin(metin) => {
                                        Some(metin.as_str())
                                    }
                                    _ => None,
                                })
                                .unwrap_or("")
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
                        crate::model::deger::VeriÖğesi::yeni([86_400_000.0, 20.0]).boyutlar([(
                            "schema".to_owned(),
                            crate::model::deger::VeriDeğeri::Metin("calcium".to_owned()),
                        )]),
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
        assert!(döküm.contains("yazı \"1:20:calcium\""), "{döküm}");
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

    #[test]
    fn saçılım_göstergesi_serinin_öntanımlı_ve_açık_opaklığını_miras_alır() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .gösterge(crate::model::bilesen::Gösterge::yeni())
            .seri(crate::model::seri::SaçılımSerisi::yeni().ad("Saçılım"))
            .seri(
                crate::model::seri::SaçılımSerisi::yeni()
                    .ad("Efekt")
                    .efektli(true),
            )
            .seri(
                crate::model::seri::SaçılımSerisi::yeni()
                    .ad("Açık")
                    .öğe_stili(crate::model::stil::ÖğeStili::yeni().opaklık(0.35)),
            );

        let öğeler = gösterge_öğeleri(&seçenekler, &HashSet::new());
        assert_eq!(
            öğeler
                .iter()
                .map(|öğe| (öğe.ad.as_str(), öğe.opaklık))
                .collect::<Vec<_>>(),
            [("Saçılım", 0.8), ("Efekt", 1.0), ("Açık", 0.35)]
        );
    }

    #[test]
    fn gösterge_top_bottom_anahtar_sözcüğüyle_alt_kenara_yerleşir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .gösterge(
                crate::model::bilesen::Gösterge::yeni()
                    .üst("bottom")
                    .iç_boşluk(15.0),
            )
            .seri(
                crate::model::seri::SütunSerisi::yeni()
                    .ad("Range")
                    .veri([1.0]),
            );
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(800.0, 600.0);

        let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());

        let (kutu, ad) = çıktı
            .gösterge_kutuları
            .first()
            .expect("gösterge öğesi çizilmeli");
        assert_eq!(ad, "Range");
        assert!((kutu.y - 571.0).abs() < 1e-3, "{kutu:?}");
    }

    #[test]
    fn alt_gösterge_item_style_kenarlığının_görünen_yüksekliğini_hesaba_katar() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .gösterge(crate::model::bilesen::Gösterge::yeni().iç_boşluk(15.0))
            .seri(
                crate::model::seri::SütunSerisi::yeni()
                    .ad("With Round Cap")
                    .öğe_stili(
                        crate::model::stil::ÖğeStili::yeni()
                            .kenarlık_rengi("green")
                            .kenarlık_kalınlığı(1.0),
                    )
                    .veri([1.0]),
            );
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);

        let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());

        let (kutu, ad) = çıktı
            .gösterge_kutuları
            .first()
            .expect("gösterge öğesi çizilmeli");
        assert_eq!(ad, "With Round Cap");
        assert_eq!((kutu.y, kutu.yükseklik), (480.0, 15.0));
    }

    #[test]
    fn çokgen_fırça_iç_kenar_ve_dış_noktaları_ayırt_eder() {
        let alan = FırçaAlanı::Çokgen {
            noktalar: vec![(10.0, 10.0), (50.0, 10.0), (30.0, 40.0)],
        };

        assert!(alan.geçerli_mi());
        assert!(alan.içeriyor_mu((30.0, 20.0)));
        assert!(alan.içeriyor_mu((10.0, 10.0)), "kenar seçime dahildir");
        assert!(!alan.içeriyor_mu((8.0, 30.0)));
    }

    #[test]
    fn graphic_sahnesi_ortak_boyama_hattinda_cizilir_ve_dondurulur() {
        use crate::model::grafik_bileseni::{GrafikBağlıMetni, GrafikBileşeni, GrafikÖğesi};

        let seçenekler = GrafikSeçenekleri::yeni().grafik(
            GrafikBileşeni::yeni().öğe(
                GrafikÖğesi::dikdörtgen(Dikdörtgen::yeni(0.0, 0.0, 140.0, 24.0))
                    .kimlik("düğme")
                    .sol(5.0)
                    .üst(5.0)
                    .bağlı_metin(GrafikBağlıMetni::yeni("Collapse Axis Breaks")),
            ),
        );
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);

        let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());

        let sahne = çıktı.grafik_sahnesi.expect("graphic sahnesi");
        assert!(sahne.sahne.isabet((75.0, 17.0)).is_some());
        assert!(yüzey.döküm().contains("Collapse Axis Breaks"));
    }

    #[test]
    fn eksen_fırçaları_ızgara_boyunca_seçer_ve_kaydırılır() {
        let yatay = FırçaAlanı::Yatay {
            başlangıç: 20.0,
            bitiş: 60.0,
            üst: 10.0,
            alt: 90.0,
        };
        let dikey = FırçaAlanı::Dikey {
            başlangıç: 30.0,
            bitiş: 70.0,
            sol: 5.0,
            sağ: 95.0,
        };

        assert!(yatay.içeriyor_mu((40.0, 80.0)));
        assert!(!yatay.içeriyor_mu((70.0, 80.0)));
        assert!(dikey.içeriyor_mu((90.0, 50.0)));
        assert!(!dikey.içeriyor_mu((90.0, 20.0)));
        assert!(yatay.kaydır(100.0, 200.0).içeriyor_mu((140.0, 280.0)));
        assert!(dikey.kaydır(100.0, 200.0).içeriyor_mu((190.0, 250.0)));
    }

    #[test]
    fn programatik_line_x_secimi_bagli_serilerin_ham_siralarina_yansir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .ızgara_ekle(crate::model::bilesen::Izgara::yeni().yükseklik("50%"))
            .ızgara_ekle(
                crate::model::bilesen::Izgara::yeni()
                    .üst("63%")
                    .yükseklik("16%"),
            )
            .x_ekseni_ekle(
                Eksen::kategori()
                    .veri(["A", "B", "C", "D"])
                    .kenar_boşluğu(false),
            )
            .x_ekseni_ekle(
                Eksen::kategori()
                    .veri(["A", "B", "C", "D"])
                    .kenar_boşluğu(false)
                    .ızgara_sırası(1),
            )
            .y_ekseni_ekle(Eksen::değer().ölçekli(true))
            .y_ekseni_ekle(Eksen::değer().ölçekli(true).ızgara_sırası(1))
            .fırça(
                crate::model::bilesen::Fırça::yeni()
                    .x_eksenleri([0, 1])
                    .bağlantı(FırçaBağı::Tümü)
                    .dış_renk_opaklığı(0.1)
                    .alan(FırçaSeçimAlanı::yatay("B", "C").x_ekseni(0)),
            )
            .seri(crate::model::seri::MumSerisi::yeni().veri([
                [10.0, 11.0, 9.0, 12.0],
                [11.0, 12.0, 10.0, 13.0],
                [12.0, 13.0, 11.0, 14.0],
                [13.0, 14.0, 12.0, 15.0],
            ]))
            .seri(crate::model::seri::ÇizgiSerisi::yeni().veri([11.0, 12.0, 13.0, 14.0]))
            .seri(
                crate::model::seri::SütunSerisi::yeni()
                    .eksenler(1, 1)
                    .veri([100.0, 200.0, 300.0, 400.0]),
            );
        let yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);
        let kurulum = kartezyen_kur(&yüzey, &seçenekler, &HashSet::new())
            .expect("kartezyen kurulum üretilmeli");

        let hazır = fırçayı_hazırla(&seçenekler, &kurulum);

        assert_eq!(hazır.alanlar.len(), 1);
        let FırçaAlanı::Yatay {
            başlangıç,
            bitiş,
            üst,
            alt,
        } = hazır.alanlar[0]
        else {
            panic!("lineX yatay piksel alanına dönüşmeli");
        };
        assert!((başlangıç - kurulum.x_eksenler[0].veriden_piksele(1.0)).abs() < 1e-4);
        assert!((bitiş - kurulum.x_eksenler[0].veriden_piksele(2.0)).abs() < 1e-4);
        assert!((üst - kurulum.ızgara_alanları[0].y).abs() < 1e-4);
        assert!((alt - kurulum.ızgara_alanları[0].alt()).abs() < 1e-4);
        assert_eq!(
            hazır.öğe_opaklıkları,
            vec![
                Some(vec![0.1, 1.0, 1.0, 0.1]),
                Some(vec![0.1, 1.0, 1.0, 0.1]),
                Some(vec![0.1, 1.0, 1.0, 0.1]),
            ]
        );
        assert_eq!(hazır.öğe_renkleri, vec![None, None, None]);
        assert_eq!(
            hazır.seçili_ham_sıralar,
            vec![vec![1, 2], vec![1, 2], vec![1, 2]]
        );
    }

    #[test]
    fn varsayilan_dis_firca_rengi_ve_brushselected_ham_siralari_uretilir() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .animasyon(false)
            .x_ekseni(Eksen::kategori().veri(["A", "B", "C", "D"]))
            .y_ekseni(Eksen::değer())
            .fırça(
                crate::model::bilesen::Fırça::default()
                    .alan(FırçaSeçimAlanı::yatay("B", "C").x_ekseni(0)),
            )
            .seri(crate::model::seri::SütunSerisi::yeni().veri([1, 2, 3, 4]));
        let mut yüzey = crate::cizim::KayıtYüzeyi::yeni(700.0, 525.0);
        let çıktı = grafiği_boya(&mut yüzey, &seçenekler, &BoyamaGirdisi::default());

        assert_eq!(çıktı.fırça_seçimleri, vec![vec![1, 2]]);
        let kurulum = kartezyen_kur(&yüzey, &seçenekler, &HashSet::new())
            .expect("kartezyen kurulum üretilmeli");
        let hazır = fırçayı_hazırla(&seçenekler, &kurulum);
        assert_eq!(
            hazır.öğe_renkleri,
            vec![Some(vec![
                Some(Dolgu::Düz(Renk::onaltılık(0xcfd2d7))),
                None,
                None,
                Some(Dolgu::Düz(Renk::onaltılık(0xcfd2d7))),
            ])]
        );
    }
}
