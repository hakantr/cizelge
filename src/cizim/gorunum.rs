//! Grafik görünümü — ECharts örneğinin (`echarts.init` + `setOption`)
//! gpui karşılığı.
//!
//! Boyama hattının tamamı [`grafiği_boya`] içinde, çizim yüzeyinden bağımsız
//! saf bir işlev olarak durur: gpui penceresi de altın (golden) testlerdeki
//! [`crate::cizim::KayıtYüzeyi`] de aynı hattı çalıştırır. gpui'ye özgü
//! yapıştırma (tuval, fare, animasyon karesi, olay yayını) yalnızca
//! [`GrafikGörünümü`]dedir.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use gpui::{
    Bounds, Context, EventEmitter, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels,
    Render, Window, canvas, div, prelude::*,
};

use crate::bilesen::baslik::başlık_çiz;
use crate::bilesen::eksen_cizimi::{bölme_çizgilerini_çiz, eksenleri_çiz};
use crate::bilesen::gosterge::{gösterge_çiz, GöstergeÖğesi};
use crate::bilesen::ipucu::{ipucu_çiz, İpucuSatırı};
use crate::cizim::cizici::{Çizici, ÖlçümÖnbelleği};
use crate::cizim::olay::{GrafikOlayı, İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::yuzey::{keskin, ÇizimYüzeyi};
use crate::grafik::cizgi::{nokta_listeleri, çizgi_serisi_çiz};
use crate::grafik::gosterge_saati::gösterge_saati_çiz;
use crate::grafik::huni::{huni_yerleşimi, huni_çiz};
use crate::grafik::imleyici::{im_alanlarını_çiz, im_çizgi_ve_noktalarını_çiz};
use crate::grafik::isi::{görsel_eşleme_çiz, ısı_değer_kapsamı, ısı_haritası_çiz};
use crate::grafik::mum::{kutu_çiz, mum_çiz};
use crate::grafik::pasta::{dilim_değer_metni, pasta_yerleşimi, pasta_çiz, Dilim};
use crate::grafik::sacilim::{saçılım_noktaları, saçılım_çiz, SaçılımNoktası};
use crate::grafik::sutun::{sütunları_çiz, SütunGirdisi};
use crate::hata::{BilesenHatasi, BilesenTanisi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B, ÇalışmaEkseni};
use crate::model::bilesen::{GöstergeSimgesi, Tetikleme, İmleçTürü, İpucu};
use crate::model::eksen::{Eksen, EksenKonumu, EksenTürü};
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::Seri;
use crate::model::stil::ÇizgiTürü;
use crate::olcek::{AralıkÖlçeği, KategorikÖlçek, LogÖlçeği, ZamanÖlçeği, Ölçek};
use crate::renk::Dolgu;
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yerlesim::yigin::{yığın_aralıkları, YığınAralığı};

/// Boyamanın etkileşim çıktıları: gösterge kutuları ve veri öğesi isabet
/// bölgeleri (yüzey yerel koordinatlarda).
#[derive(Default)]
pub struct BoyamaÇıktısı {
    pub gösterge_kutuları: Vec<(Dikdörtgen, String)>,
    pub isabetler: Vec<İsabetBölgesi>,
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

/// Kartezyen kurulumun sonucu.
struct KartezyenKurulum {
    kartezyen: Kartezyen2B,
    aralıklar: Vec<Vec<YığınAralığı>>,
    görünürler: Vec<bool>,
}

/// Kartezyen koordinat sistemini kurar: kapsamlar, ölçekler, ızgara alanı.
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
    let eksen_tanımlı = seçenekler.x_ekseni.is_some() || seçenekler.y_ekseni.is_some();
    if !kartezyen_var && !eksen_tanımlı {
        return None;
    }

    let x_seçenek = seçenekler.x_ekseni.clone().unwrap_or_else(Eksen::kategori);
    let y_seçenek = seçenekler.y_ekseni.clone().unwrap_or_else(Eksen::değer);
    let x_kategorik = x_seçenek.tür == EksenTürü::Kategori;
    let y_kategorik = y_seçenek.tür == EksenTürü::Kategori;

    let aralıklar = yığın_aralıkları(&seçenekler.seriler, &görünürler);

    // Değer kapsamları.
    let mut değer_kapsamı = [f64::INFINITY, f64::NEG_INFINITY];
    let mut x_değer_kapsamı = [f64::INFINITY, f64::NEG_INFINITY];
    let kapsa = |kapsam: &mut [f64; 2], v: f64| {
        if v.is_finite() {
            kapsam[0] = kapsam[0].min(v);
            kapsam[1] = kapsam[1].max(v);
        }
    };
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        if !seri.kartezyen_mi() || !görünürler.get(i).copied().unwrap_or(false) {
            continue;
        }
        let sütun_mu = matches!(seri, Seri::Sütun(_));
        // Isı haritası: her iki eksen kategorik; sayısal kapsama girmez.
        if matches!(seri, Seri::Isı(_)) {
            continue;
        }
        // Çok değerli seriler (mum/kutu): dizinin tüm bileşenleri kapsanır.
        if matches!(seri, Seri::Mum(_) | Seri::Kutu(_)) {
            for (j, öğe) in seri.veri().iter().enumerate() {
                if let Some(dizi) = öğe.değer.dizi() {
                    for v in dizi {
                        kapsa(&mut değer_kapsamı, *v);
                    }
                }
                kapsa(&mut x_değer_kapsamı, j as f64);
            }
            continue;
        }
        let Some(seri_aralıkları) = aralıklar.get(i) else { continue };
        for (j, aralık) in seri_aralıkları.iter().enumerate() {
            let Some((taban, tepe)) = aralık else { continue };
            kapsa(&mut değer_kapsamı, *tepe);
            if sütun_mu || taban.abs() > 1e-12 {
                kapsa(&mut değer_kapsamı, *taban);
            }
            let x_değeri = seri
                .veri()
                .get(j)
                .and_then(|ö| ö.değer.x())
                .unwrap_or(j as f64);
            kapsa(&mut x_değer_kapsamı, x_değeri);
        }
    }

    // Kategori listesi: eksen verisi ya da seri verisinden türetilir.
    let kategoriler_derle = |eksen: &Eksen| -> Vec<String> {
        if !eksen.veri.is_empty() {
            return eksen.veri.clone();
        }
        let mut en_uzun = 0usize;
        let mut adlar: Option<Vec<String>> = None;
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if !seri.kartezyen_mi() || !görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            let veri = seri.veri();
            if veri.len() > en_uzun {
                en_uzun = veri.len();
                let tüm_adlar: Vec<String> = veri
                    .iter()
                    .enumerate()
                    .map(|(j, ö)| ö.ad.clone().unwrap_or_else(|| format!("{j}")))
                    .collect();
                adlar = Some(tüm_adlar);
            }
        }
        adlar.unwrap_or_default()
    };

    // Ölçekler (piksel aralığından bağımsız kurulur).
    let x_ölçek = ölçek_kur(
        &x_seçenek,
        if x_kategorik { kategoriler_derle(&x_seçenek) } else { Vec::new() },
        if y_kategorik { değer_kapsamı } else { x_değer_kapsamı },
    );
    let y_ölçek = ölçek_kur(
        &y_seçenek,
        if y_kategorik { kategoriler_derle(&y_seçenek) } else { Vec::new() },
        değer_kapsamı,
    );

    // Izgara alanı.
    let ızgara = &seçenekler.ızgara;
    let mut sol = ızgara.sol.çöz(yüzey.genişlik());
    let mut sağ_boşluk = ızgara.sağ.çöz(yüzey.genişlik());
    let üst = ızgara.üst.çöz(yüzey.yükseklik());
    let mut alt_boşluk = ızgara.alt.çöz(yüzey.yükseklik());

    if ızgara.etiketi_kapsa {
        // Sol eksen etiketlerinin en genişini ölçüp alanı içeri çek.
        let y_boyut = y_seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
        let mut en_geniş = 0.0f32;
        for çentik in y_ölçek.çentikler() {
            let metin = y_ölçek.etiket(çentik.değer);
            en_geniş = en_geniş.max(yüzey.yazı_ölç(&metin, y_boyut).0);
        }
        sol += en_geniş + y_seçenek.etiket.boşluk;
        let x_boyut = x_seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
        alt_boşluk += x_boyut * crate::cizim::yuzey::SATIR_ORANI + x_seçenek.etiket.boşluk;
        sağ_boşluk = sağ_boşluk.max(20.0);
    }

    let alan = Dikdörtgen::yeni(
        sol,
        üst,
        (yüzey.genişlik() - sol - sağ_boşluk).max(1.0),
        (yüzey.yükseklik() - üst - alt_boşluk).max(1.0),
    );

    let x_konum = x_seçenek.konum.unwrap_or(EksenKonumu::Alt);
    let y_konum = y_seçenek.konum.unwrap_or(EksenKonumu::Sol);
    let x_ekseni = ÇalışmaEkseni::yeni(x_seçenek, x_ölçek, [alan.x, alan.sağ()], x_konum);
    // Dikey eksen piksel aralığı alttan yukarı doğrudur.
    let y_ekseni = ÇalışmaEkseni::yeni(y_seçenek, y_ölçek, [alan.alt(), alan.y], y_konum);

    Some(KartezyenKurulum {
        kartezyen: Kartezyen2B { x: x_ekseni, y: y_ekseni, alan },
        aralıklar,
        görünürler,
    })
}

/// Eksen tetiklemeli ipucunun hazırlanmış içeriği.
struct Eksenİpucu {
    kategori_sırası: usize,
    başlık: String,
    satırlar: Vec<İpucuSatırı>,
}

fn eksen_ipucu_derle(
    seçenekler: &GrafikSeçenekleri,
    kurulum: &KartezyenKurulum,
    fare: (f32, f32),
    ipucu: &İpucu,
) -> Option<Eksenİpucu> {
    let kartezyen = &kurulum.kartezyen;
    if !kartezyen.alan.içeriyor_mu(fare) {
        return None;
    }
    // Bant ekseni: kategorik olan (öncelik x).
    let (bant_ekseni, yatay_bant) = if kartezyen.x.ölçek.kategorik_mi() {
        (&kartezyen.x, true)
    } else if kartezyen.y.ölçek.kategorik_mi() {
        (&kartezyen.y, false)
    } else {
        return None;
    };
    let fare_konumu = if yatay_bant { fare.0 } else { fare.1 };
    let sıra = bant_ekseni.pikselden_veriye(fare_konumu) as usize;
    let başlık = bant_ekseni.ölçek.etiket(sıra as f64);

    let mut satırlar = Vec::new();
    for (i, seri) in seçenekler.seriler.iter().enumerate() {
        if !seri.kartezyen_mi() || !kurulum.görünürler.get(i).copied().unwrap_or(false) {
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
    Some(Eksenİpucu { kategori_sırası: sıra, başlık, satırlar })
}

/// Tüm grafiği verilen yüzeye boyar; etkileşim bölgelerini döndürür.
///
/// `ilerleme` giriş animasyonunun yumuşatılmış oranı, `fare` yüzey yerel
/// fare konumu, `kapalı` gösterge ile kapatılmış adlardır.
pub fn grafiği_boya(
    yüzey: &mut dyn ÇizimYüzeyi,
    seçenekler: &GrafikSeçenekleri,
    ilerleme: f32,
    zaman_sn: f32,
    fare: Option<(f32, f32)>,
    kapalı: &HashSet<String>,
) -> BoyamaÇıktısı {
    let mut çıktı = BoyamaÇıktısı::default();

    // 1) Arka plan.
    if let Some(renk) = seçenekler.arkaplan {
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
        çıktı.gösterge_kutuları = gösterge_çiz(yüzey, g, &öğeler);
    }

    let ipucu_seçeneği = seçenekler.ipucu.clone().filter(|i| i.göster);

    // 4) Kartezyen bölüm.
    let kurulum = kartezyen_kur(yüzey, seçenekler, kapalı);
    /// `(başlık, satırlar, konum)`.
    type Bekleyenİpucu = (Option<String>, Vec<İpucuSatırı>, (f32, f32));
    let mut bekleyen_ipucu: Option<Bekleyenİpucu> = None;

    if let Some(kurulum) = &kurulum {
        let kartezyen = &kurulum.kartezyen;

        bölme_çizgilerini_çiz(yüzey, kartezyen);

        // Eksen imleci içeriği (gölge serilerin altına, çizgi üstüne çizilir).
        let eksen_ipucu = match (&ipucu_seçeneği, fare) {
            (Some(ipucu), Some(f)) if ipucu.tetikleme == Tetikleme::Eksen => {
                eksen_ipucu_derle(seçenekler, kurulum, f, ipucu)
            }
            _ => None,
        };

        if let (Some(ipucu), Some(eksen_ip)) = (&ipucu_seçeneği, &eksen_ipucu)
            && ipucu.imleç == İmleçTürü::Gölge {
                let bant_x = kartezyen.x.ölçek.kategorik_mi();
                let bant_ekseni = if bant_x { &kartezyen.x } else { &kartezyen.y };
                let merkez = bant_ekseni.veriden_piksele(eksen_ip.kategori_sırası as f64);
                let bant = bant_ekseni.bant_genişliği();
                let alan = kartezyen.alan;
                let d = if bant_x {
                    Dikdörtgen::yeni(merkez - bant / 2.0, alan.y, bant, alan.yükseklik)
                } else {
                    Dikdörtgen::yeni(alan.x, merkez - bant / 2.0, alan.genişlik, bant)
                };
                yüzey.dikdörtgen(d, &Dolgu::Düz(tema::İMLEÇ_GÖLGESİ), [0.0; 4], None);
            }

        eksenleri_çiz(yüzey, kartezyen);

        // İm alanları serilerin altına boyanır.
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            if let Some(imleyiciler) = seri.imleyiciler()
                && imleyiciler.alan.is_some() {
                    im_alanlarını_çiz(
                        yüzey,
                        imleyiciler,
                        seri,
                        kartezyen,
                        seçenekler.seri_rengi(i),
                    );
                }
        }

        // Seriler: sütunlar toplu (yerleşim paylaşımı), diğerleri sırayla.
        let sütun_girdileri: Vec<SütunGirdisi> = seçenekler
            .seriler
            .iter()
            .enumerate()
            .filter(|(i, _)| kurulum.görünürler.get(*i).copied().unwrap_or(false))
            .filter_map(|(i, s)| match s {
                Seri::Sütun(sütun) => Some(SütunGirdisi {
                    seri: sütun,
                    genel_sıra: i,
                    aralıklar: kurulum
                        .aralıklar
                        .get(i)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]),
                    renk: seçenekler.seri_rengi(i),
                }),
                _ => None,
            })
            .collect();
        let mut sütunlar_çizildi = false;

        // Saçılım vurgusu (öğe ipucu) için önden isabet araması.
        // `(seri sırası, vurgulu veri sırası, noktalar)`.
        type SaçılımVurgusu = (usize, Option<usize>, Vec<SaçılımNoktası>);
        let mut saçılım_vurguları: Vec<SaçılımVurgusu> = Vec::new();
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if let Seri::Saçılım(s) = seri {
                if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                    continue;
                }
                let noktalar = saçılım_noktaları(s, kartezyen);
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
                        kartezyen,
                        seri_aralıkları,
                        seçenekler.seri_rengi(i),
                        ilerleme,
                    );
                    // Sembol noktaları tıklanabilir bölgelerdir.
                    let (tepeler, _) = nokta_listeleri(s, kartezyen, seri_aralıkları);
                    for (j, nokta) in tepeler.iter().enumerate() {
                        let Some(nokta) = nokta else { continue };
                        let Some(öğe) = s.veri.get(j) else { continue };
                        çıktı.isabetler.push(İsabetBölgesi {
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
                    if !sütunlar_çizildi {
                        sütunları_çiz(
                            yüzey,
                            &sütun_girdileri,
                            kartezyen,
                            ilerleme,
                            &mut çıktı.isabetler,
                        );
                        sütunlar_çizildi = true;
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
                            çıktı.isabetler.push(İsabetBölgesi {
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
                                bekleyen_ipucu = Some((
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
                    kartezyen,
                    ilerleme,
                    &mut çıktı.isabetler,
                ),
                Seri::Kutu(s) => kutu_çiz(
                    yüzey,
                    s,
                    i,
                    kartezyen,
                    seçenekler.seri_rengi(i),
                    &mut çıktı.isabetler,
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
                        kartezyen,
                        &eşleme,
                        kapsam,
                        ilerleme,
                        &mut çıktı.isabetler,
                    );
                }
                Seri::Pasta(_) | Seri::Huni(_) | Seri::GöstergeSaati(_) => {}
            }
        }

        // İm çizgileri ve raptiyeler serilerin üstüne boyanır.
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if !kurulum.görünürler.get(i).copied().unwrap_or(false) {
                continue;
            }
            if let Some(imleyiciler) = seri.imleyiciler()
                && (imleyiciler.çizgi.is_some() || imleyiciler.nokta.is_some()) {
                    im_çizgi_ve_noktalarını_çiz(
                        yüzey,
                        imleyiciler,
                        seri,
                        kartezyen,
                        seçenekler.seri_rengi(i),
                    );
                }
        }

        // Eksen imleci çizgisi + eksen ipucu penceresi.
        if let Some(eksen_ip) = eksen_ipucu
            && let Some(ipucu) = &ipucu_seçeneği {
                if ipucu.imleç == İmleçTürü::Çizgi || ipucu.imleç == İmleçTürü::Çapraz {
                    let bant_x = kartezyen.x.ölçek.kategorik_mi();
                    let bant_ekseni = if bant_x { &kartezyen.x } else { &kartezyen.y };
                    let merkez =
                        keskin(bant_ekseni.veriden_piksele(eksen_ip.kategori_sırası as f64));
                    let alan = kartezyen.alan;
                    if bant_x {
                        yüzey.çizgi(
                            (merkez, alan.y),
                            (merkez, alan.alt()),
                            1.0,
                            tema::İMLEÇ_ÇİZGİSİ,
                            ÇizgiTürü::Düz,
                        );
                    } else {
                        yüzey.çizgi(
                            (alan.x, merkez),
                            (alan.sağ(), merkez),
                            1.0,
                            tema::İMLEÇ_ÇİZGİSİ,
                            ÇizgiTürü::Düz,
                        );
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
        görsel_eşleme_çiz(yüzey, eşleme, eşleme.kapsam_çöz(veri_kapsamı));
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

    // 5b) Huni ve gösterge saati serileri.
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
            _ => {}
        }
    }

    // 6) İpucu penceresi (her şeyin üstüne).
    if let (Some(ipucu), Some((başlık, satırlar, konum))) = (&ipucu_seçeneği, bekleyen_ipucu) {
        ipucu_çiz(yüzey, ipucu, konum, başlık.as_deref(), &satırlar);
    }

    çıktı
}

/// Gösterge öğelerinin pencere-mutlak isabet kutuları (tıklama için).
type GöstergeKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, String)>>>;

/// ECharts grafik örneğinin gpui görünümü.
pub struct GrafikGörünümü {
    seçenekler: Arc<GrafikSeçenekleri>,
    /// Giriş animasyonunun başlangıcı.
    başlangıç: Instant,
    /// Veri geçiş animasyonu: eski seçenekler + geçiş başlangıcı.
    eski_seçenekler: Option<Arc<GrafikSeçenekleri>>,
    geçiş_başlangıcı: Option<Instant>,
    /// Pencere-mutlak fare konumu.
    fare: Option<(f32, f32)>,
    kapalı: HashSet<String>,
    gösterge_kutuları: GöstergeKutuları,
    /// Pencere-mutlak isabet bölgeleri (tıklama olayları için).
    isabetler: Rc<RefCell<Vec<İsabetBölgesi>>>,
    /// Boyama sırasında biriken, bir sonraki karede olay olarak yayımlanacak
    /// tanılar.
    bekleyen_tanılar: Rc<RefCell<Vec<BilesenTanisi>>>,
    ölçüm_önbelleği: ÖlçümÖnbelleği,
}

impl EventEmitter<GrafikOlayı> for GrafikGörünümü {}
impl EventEmitter<BilesenTanisi> for GrafikGörünümü {}

impl GrafikGörünümü {
    pub fn yeni(seçenekler: GrafikSeçenekleri) -> Self {
        GrafikGörünümü {
            seçenekler: Arc::new(seçenekler),
            başlangıç: Instant::now(),
            eski_seçenekler: None,
            geçiş_başlangıcı: None,
            fare: None,
            kapalı: HashSet::new(),
            gösterge_kutuları: Rc::new(RefCell::new(Vec::new())),
            isabetler: Rc::new(RefCell::new(Vec::new())),
            bekleyen_tanılar: Rc::new(RefCell::new(Vec::new())),
            ölçüm_önbelleği: Rc::new(RefCell::new(std::collections::HashMap::new())),
        }
    }

    /// Seçenekleri değiştirir (ECharts `setOption` karşılığı). Yeni
    /// seçenekler önce doğrulanır: geçersizse **işlem geri alınır** (mevcut
    /// seçenekler korunur), hata tanı olayı olarak yayımlanır ve `Err`
    /// döner. Geçerliyse, grafik zaten çizilmişse eski veriden yeniye akan
    /// bir geçiş animasyonu oynatılır; ilk kurulumdaysa giriş animasyonu
    /// yeniden başlar.
    pub fn seçenekleri_değiştir(
        &mut self,
        seçenekler: GrafikSeçenekleri,
        cx: &mut Context<Self>,
    ) -> Result<(), BilesenHatasi> {
        if let Err(hata) = seçenekler.doğrula() {
            cx.emit(BilesenTanisi::yeni("seçenekleri_değiştir", hata.clone()));
            return Err(hata);
        }
        let giriş_bitti = self.başlangıç.elapsed().as_secs_f32() * 1000.0
            >= self.seçenekler.animasyon_süresi;
        if giriş_bitti && seçenekler.animasyon {
            self.eski_seçenekler = Some(self.seçenekler.clone());
            self.geçiş_başlangıcı = Some(Instant::now());
        } else {
            self.başlangıç = Instant::now();
            self.eski_seçenekler = None;
            self.geçiş_başlangıcı = None;
        }
        self.seçenekler = Arc::new(seçenekler);
        cx.notify();
        Ok(())
    }

    pub fn seçenekler(&self) -> &GrafikSeçenekleri {
        &self.seçenekler
    }

    /// Etkin boyama seçenekleri ve giriş ilerlemesi; geçiş animasyonu
    /// sırasında ara değerli seçenekler üretir.
    fn boyama_durumu(&mut self) -> (Arc<GrafikSeçenekleri>, f32, bool) {
        if let (Some(eski), Some(t0)) = (&self.eski_seçenekler, self.geçiş_başlangıcı) {
            let süre = self.seçenekler.animasyon_süresi_güncelleme.max(1.0);
            let t = t0.elapsed().as_secs_f32() * 1000.0 / süre;
            if t >= 1.0 {
                self.eski_seçenekler = None;
                self.geçiş_başlangıcı = None;
                return (self.seçenekler.clone(), 1.0, false);
            }
            let eğri = self.seçenekler.animasyon_eğrisi.uygula(t);
            let ara = GrafikSeçenekleri::ara_değerle(eski, &self.seçenekler, eğri);
            return (Arc::new(ara), 1.0, true);
        }

        let ham = if self.seçenekler.animasyon && self.seçenekler.animasyon_süresi > 0.0 {
            (self.başlangıç.elapsed().as_secs_f32() * 1000.0 / self.seçenekler.animasyon_süresi)
                .min(1.0)
        } else {
            1.0
        };
        let sürüyor = ham < 1.0;
        (
            self.seçenekler.clone(),
            self.seçenekler.animasyon_eğrisi.uygula(ham),
            sürüyor,
        )
    }
}

impl Render for GrafikGörünümü {
    fn render(&mut self, pencere: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Önceki karede biriken tanıları olay olarak yayımla.
        let bekleyenler = self
            .bekleyen_tanılar
            .try_borrow_mut()
            .map(|mut tanılar| std::mem::take(&mut *tanılar))
            .unwrap_or_default();
        for tanı in bekleyenler {
            cx.emit(tanı);
        }

        let (etkin_seçenekler, ilerleme, sürüyor) = self.boyama_durumu();
        // Dalga efektli seriler sürekli kare ister.
        let sürekli = etkin_seçenekler
            .seriler
            .iter()
            .any(|s| matches!(s, Seri::Saçılım(sa) if sa.efektli));
        if sürüyor || sürekli {
            pencere.request_animation_frame();
        }
        let zaman_sn = self.başlangıç.elapsed().as_secs_f32();

        let fare = self.fare;
        let kapalı = self.kapalı.clone();
        let gösterge_kutuları = self.gösterge_kutuları.clone();
        let isabetler = self.isabetler.clone();
        let tanılar = self.bekleyen_tanılar.clone();
        let önbellek = self.ölçüm_önbelleği.clone();

        div()
            .id("cizelge")
            .size_full()
            .child(
                canvas(
                    |_, _, _| {},
                    move |sınırlar, _, pencere, uygulama| {
                        let mut çizici =
                            Çizici::yeni(pencere, uygulama, sınırlar, Some(önbellek));
                        let köken = çizici.köken;
                        let yerel_fare = fare.map(|(x, y)| (x - köken.0, y - köken.1));
                        let çıktı = grafiği_boya(
                            &mut çizici,
                            &etkin_seçenekler,
                            ilerleme,
                            zaman_sn,
                            yerel_fare,
                            &kapalı,
                        );
                        let tanı_bildir = |bileşen: &'static str| {
                            if let Ok(mut kayıt) = tanılar.try_borrow_mut() {
                                kayıt.push(BilesenTanisi::yeni(
                                    bileşen,
                                    BilesenHatasi::KilitliDurum { bileşen },
                                ));
                            }
                        };
                        // Çıktıları pencere-mutlak koordinata çevirip sakla.
                        match gösterge_kutuları.try_borrow_mut() {
                            Ok(mut kutular) => {
                                kutular.clear();
                                for (kutu, ad) in çıktı.gösterge_kutuları {
                                    kutular.push((çizici.sınırlar(kutu), ad));
                                }
                            }
                            Err(_) => tanı_bildir("gösterge_kutuları"),
                        }
                        match isabetler.try_borrow_mut() {
                            Ok(mut bölgeler) => {
                                bölgeler.clear();
                                for bölge in çıktı.isabetler {
                                    bölgeler.push(İsabetBölgesi {
                                        geometri: bölge.geometri.kaydır(köken.0, köken.1),
                                        ..bölge
                                    });
                                }
                            }
                            Err(_) => tanı_bildir("isabet_bölgeleri"),
                        }
                    },
                )
                .size_full(),
            )
            .on_mouse_move(cx.listener(|bu, olay: &MouseMoveEvent, _, cx| {
                let yeni = (f32::from(olay.position.x), f32::from(olay.position.y));
                if bu.fare != Some(yeni) {
                    bu.fare = Some(yeni);
                    cx.notify();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|bu, olay: &MouseDownEvent, _, cx| {
                    // 1) Gösterge tıklaması: seriyi/dilimi aç-kapat.
                    let kutular = match bu.gösterge_kutuları.try_borrow() {
                        Ok(kutular) => kutular,
                        Err(_) => {
                            cx.emit(BilesenTanisi::yeni(
                                "gösterge_tıklaması",
                                BilesenHatasi::KilitliDurum { bileşen: "gösterge_kutuları" },
                            ));
                            return;
                        }
                    };
                    let vurulan = kutular
                        .iter()
                        .find(|(kutu, _)| kutu.contains(&olay.position))
                        .map(|(_, ad)| ad.clone());
                    drop(kutular);
                    if let Some(ad) = vurulan {
                        let görünür = bu.kapalı.remove(&ad);
                        if !görünür {
                            bu.kapalı.insert(ad.clone());
                        }
                        cx.emit(GrafikOlayı::GöstergeDeğişti { ad, görünür });
                        cx.notify();
                        return;
                    }
                    // 2) Veri öğesi tıklaması: en üstte çizilen bölge kazanır.
                    let nokta = (f32::from(olay.position.x), f32::from(olay.position.y));
                    let bölge = match bu.isabetler.try_borrow() {
                        Ok(bölgeler) => bölgeler
                            .iter()
                            .rev()
                            .find(|b| b.geometri.içeriyor_mu(nokta))
                            .cloned(),
                        Err(_) => {
                            cx.emit(BilesenTanisi::yeni(
                                "öğe_tıklaması",
                                BilesenHatasi::KilitliDurum { bileşen: "isabet_bölgeleri" },
                            ));
                            return;
                        }
                    };
                    if let Some(b) = bölge {
                        cx.emit(GrafikOlayı::ÖğeTıklandı {
                            seri_sırası: b.seri_sırası,
                            veri_sırası: b.veri_sırası,
                            seri_adı: b.seri_adı,
                            ad: b.ad,
                            değer: b.değer,
                        });
                    }
                }),
            )
            .on_hover(cx.listener(|bu, üzerinde: &bool, _, cx| {
                if !üzerinde && bu.fare.is_some() {
                    bu.fare = None;
                    cx.notify();
                }
            }))
    }
}
