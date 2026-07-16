//! Grafik görünümü — ECharts örneğinin (`echarts.init` + `setOption`)
//! gpui karşılığı. `Render` uygulayan bu görünüm; yerleşimi hesaplar,
//! bileşenleri ve serileri boyar, fare etkileşimini (gösterge tıklaması,
//! eksen imleci, ipucu) yönetir.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use gpui::{
    App, Bounds, Context, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Render, Window,
    canvas, div, prelude::*,
};

use crate::bilesen::baslik::başlık_çiz;
use crate::bilesen::eksen_cizimi::{bölme_çizgilerini_çiz, eksenleri_çiz};
use crate::bilesen::gosterge::{gösterge_çiz, GöstergeÖğesi};
use crate::bilesen::ipucu::{ipucu_çiz, İpucuSatırı};
use crate::cizim::cizici::Çizici;
use crate::grafik::cizgi::çizgi_serisi_çiz;
use crate::grafik::pasta::{dilim_değer_metni, pasta_yerleşimi, pasta_çiz, Dilim};
use crate::grafik::sacilim::{saçılım_noktaları, saçılım_çiz, SaçılımNoktası};
use crate::grafik::sutun::{sütunları_çiz, SütunGirdisi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B, ÇalışmaEkseni};
use crate::model::bilesen::{GöstergeSimgesi, Tetikleme, İmleçTürü, İpucu};
use crate::model::eksen::{Eksen, EksenKonumu, EksenTürü};
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::Seri;
use crate::olcek::{
    AralıkÖlçeği, KategorikÖlçek, LogÖlçeği, ZamanÖlçeği, Ölçek,
};
use crate::model::stil::ÇizgiTürü;
use crate::renk::Dolgu;
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yerlesim::yigin::{yığın_aralıkları, YığınAralığı};

/// Boyama sırasında tuvale aktarılan anlık durum.
struct BoyamaDurumu {
    seçenekler: Arc<GrafikSeçenekleri>,
    ilerleme: f32,
    /// Pencere-mutlak fare konumu.
    fare: Option<(f32, f32)>,
    kapalı: HashSet<String>,
    /// Gösterge öğelerinin pencere-mutlak isabet kutuları (tıklama için).
    gösterge_kutuları: Rc<RefCell<Vec<(Bounds<Pixels>, String)>>>,
}

/// ECharts grafik örneğinin gpui görünümü.
pub struct GrafikGörünümü {
    seçenekler: Arc<GrafikSeçenekleri>,
    başlangıç: Instant,
    fare: Option<(f32, f32)>,
    kapalı: HashSet<String>,
    gösterge_kutuları: Rc<RefCell<Vec<(Bounds<Pixels>, String)>>>,
}

impl GrafikGörünümü {
    pub fn yeni(seçenekler: GrafikSeçenekleri) -> Self {
        GrafikGörünümü {
            seçenekler: Arc::new(seçenekler),
            başlangıç: Instant::now(),
            fare: None,
            kapalı: HashSet::new(),
            gösterge_kutuları: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Seçenekleri değiştirir ve giriş animasyonunu yeniden başlatır
    /// (ECharts `setOption` karşılığı).
    pub fn seçenekleri_değiştir(
        &mut self,
        seçenekler: GrafikSeçenekleri,
        cx: &mut Context<Self>,
    ) {
        self.seçenekler = Arc::new(seçenekler);
        self.başlangıç = Instant::now();
        cx.notify();
    }

    pub fn seçenekler(&self) -> &GrafikSeçenekleri {
        &self.seçenekler
    }
}

impl Render for GrafikGörünümü {
    fn render(&mut self, pencere: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ham_ilerleme = if self.seçenekler.animasyon && self.seçenekler.animasyon_süresi > 0.0
        {
            (self.başlangıç.elapsed().as_secs_f32() * 1000.0 / self.seçenekler.animasyon_süresi)
                .min(1.0)
        } else {
            1.0
        };
        if ham_ilerleme < 1.0 {
            pencere.request_animation_frame();
        }

        let durum = BoyamaDurumu {
            seçenekler: self.seçenekler.clone(),
            ilerleme: self.seçenekler.animasyon_eğrisi.uygula(ham_ilerleme),
            fare: self.fare,
            kapalı: self.kapalı.clone(),
            gösterge_kutuları: self.gösterge_kutuları.clone(),
        };

        div()
            .id("cizelge")
            .size_full()
            .child(
                canvas(
                    |_, _, _| {},
                    move |sınırlar, _, pencere, uygulama| {
                        boya(sınırlar, &durum, pencere, uygulama);
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
                    let vurulan = bu
                        .gösterge_kutuları
                        .borrow()
                        .iter()
                        .find(|(kutu, _)| kutu.contains(&olay.position))
                        .map(|(_, ad)| ad.clone());
                    if let Some(ad) = vurulan {
                        // Gösterge tıklaması: seriyi/dilimi aç-kapat.
                        if !bu.kapalı.remove(&ad) {
                            bu.kapalı.insert(ad);
                        }
                        cx.notify();
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
    çizici: &Çizici,
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
        if !seri.kartezyen_mi() || !görünürler[i] {
            continue;
        }
        let sütun_mu = matches!(seri, Seri::Sütun(_));
        for (j, aralık) in aralıklar[i].iter().enumerate() {
            let Some((taban, tepe)) = aralık else { continue };
            kapsa(&mut değer_kapsamı, *tepe);
            if sütun_mu || taban.abs() > 1e-12 {
                kapsa(&mut değer_kapsamı, *taban);
            }
            let x_değeri = seri.veri()[j].değer.x().unwrap_or(j as f64);
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
            if !seri.kartezyen_mi() || !görünürler[i] {
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
    let mut sol = ızgara.sol.çöz(çizici.genişlik);
    let mut sağ_boşluk = ızgara.sağ.çöz(çizici.genişlik);
    let üst = ızgara.üst.çöz(çizici.yükseklik);
    let mut alt_boşluk = ızgara.alt.çöz(çizici.yükseklik);

    if ızgara.etiketi_kapsa {
        // Sol eksen etiketlerinin en genişini ölçüp alanı içeri çek.
        let y_boyut = y_seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
        let mut en_geniş = 0.0f32;
        for çentik in y_ölçek.çentikler() {
            let metin = y_ölçek.etiket(çentik.değer);
            en_geniş = en_geniş.max(çizici.yazı_ölç(&metin, y_boyut).0);
        }
        sol += en_geniş + y_seçenek.etiket.boşluk;
        let x_boyut = x_seçenek.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
        alt_boşluk += x_boyut * crate::cizim::cizici::SATIR_ORANI + x_seçenek.etiket.boşluk;
        sağ_boşluk = sağ_boşluk.max(20.0);
    }

    let alan = Dikdörtgen::yeni(
        sol,
        üst,
        (çizici.genişlik - sol - sağ_boşluk).max(1.0),
        (çizici.yükseklik - üst - alt_boşluk).max(1.0),
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
        if !seri.kartezyen_mi() || !kurulum.görünürler[i] {
            continue;
        }
        let Some(öğe) = seri.veri().get(sıra) else { continue };
        let Some(değer) = öğe.değer.sayı() else { continue };
        let metin = match &ipucu.değer_biçimleyici {
            Some(b) => b.uygula(değer, &binlik_ayır(değer)),
            None => binlik_ayır(değer),
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

/// Tüm grafiği boyar.
fn boya(
    sınırlar: Bounds<Pixels>,
    durum: &BoyamaDurumu,
    pencere: &mut Window,
    uygulama: &mut App,
) {
    let mut çizici = Çizici::yeni(pencere, uygulama, sınırlar);
    let seçenekler = &*durum.seçenekler;

    // 1) Arka plan.
    if let Some(renk) = seçenekler.arkaplan {
        let tümü = Dikdörtgen::yeni(0.0, 0.0, çizici.genişlik, çizici.yükseklik);
        çizici.dikdörtgen(tümü, &Dolgu::Düz(renk), [0.0; 4], None);
    }

    // 2) Başlık.
    if let Some(başlık) = &seçenekler.başlık {
        başlık_çiz(&mut çizici, başlık);
    }

    // 3) Gösterge.
    let öğeler = gösterge_öğeleri(seçenekler, &durum.kapalı);
    let kutular = match &seçenekler.gösterge {
        Some(g) => gösterge_çiz(&mut çizici, g, &öğeler),
        None => Vec::new(),
    };
    {
        let mut kayıt = durum.gösterge_kutuları.borrow_mut();
        kayıt.clear();
        for (kutu, ad) in kutular {
            kayıt.push((çizici.sınırlar(kutu), ad));
        }
    }

    // Fare, grafik yerel koordinatta.
    let fare = durum
        .fare
        .map(|(x, y)| (x - çizici.köken.0, y - çizici.köken.1));

    let ipucu_seçeneği = seçenekler.ipucu.clone().filter(|i| i.göster);

    // 4) Kartezyen bölüm.
    let kurulum = kartezyen_kur(&çizici, seçenekler, &durum.kapalı);
    let mut bekleyen_ipucu: Option<(Option<String>, Vec<İpucuSatırı>, (f32, f32))> = None;

    if let Some(kurulum) = &kurulum {
        let kartezyen = &kurulum.kartezyen;

        bölme_çizgilerini_çiz(&mut çizici, kartezyen);

        // Eksen imleci içeriği (gölge serilerin altına, çizgi üstüne çizilir).
        let eksen_ipucu = match (&ipucu_seçeneği, fare) {
            (Some(ipucu), Some(f)) if ipucu.tetikleme == Tetikleme::Eksen => {
                eksen_ipucu_derle(seçenekler, kurulum, f, ipucu)
            }
            _ => None,
        };

        if let (Some(ipucu), Some(eksen_ip)) = (&ipucu_seçeneği, &eksen_ipucu) {
            if ipucu.imleç == İmleçTürü::Gölge {
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
                çizici.dikdörtgen(d, &Dolgu::Düz(tema::İMLEÇ_GÖLGESİ), [0.0; 4], None);
            }
        }

        eksenleri_çiz(&mut çizici, kartezyen);

        // Seriler: sütunlar toplu (yerleşim paylaşımı), diğerleri sırayla.
        let sütun_girdileri: Vec<SütunGirdisi> = seçenekler
            .seriler
            .iter()
            .enumerate()
            .filter(|(i, s)| matches!(s, Seri::Sütun(_)) && kurulum.görünürler[*i])
            .map(|(i, s)| {
                let Seri::Sütun(sütun) = s else { unreachable!() };
                SütunGirdisi {
                    seri: sütun,
                    genel_sıra: i,
                    aralıklar: &kurulum.aralıklar[i],
                    renk: seçenekler.seri_rengi(i),
                }
            })
            .collect();
        let mut sütunlar_çizildi = false;

        // Saçılım vurgusu (öğe ipucu) için önden isabet araması.
        let mut saçılım_vurguları: Vec<(usize, Option<usize>, Vec<SaçılımNoktası>)> = Vec::new();
        for (i, seri) in seçenekler.seriler.iter().enumerate() {
            if let Seri::Saçılım(s) = seri {
                if !kurulum.görünürler[i] {
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
            if !kurulum.görünürler[i] {
                continue;
            }
            match seri {
                Seri::Çizgi(s) => çizgi_serisi_çiz(
                    &mut çizici,
                    s,
                    kartezyen,
                    &kurulum.aralıklar[i],
                    seçenekler.seri_rengi(i),
                    durum.ilerleme,
                ),
                Seri::Sütun(_) => {
                    if !sütunlar_çizildi {
                        sütunları_çiz(&mut çizici, &sütun_girdileri, kartezyen, durum.ilerleme);
                        sütunlar_çizildi = true;
                    }
                }
                Seri::Saçılım(s) => {
                    let kayıt = saçılım_vurguları.iter().find(|(sıra, ..)| *sıra == i);
                    if let Some((_, vurgu, noktalar)) = kayıt {
                        saçılım_çiz(
                            &mut çizici,
                            s,
                            noktalar,
                            seçenekler.seri_rengi(i),
                            durum.ilerleme,
                            *vurgu,
                        );
                        // Öğe ipucu.
                        if let (Some(sıra), Some(f)) = (vurgu, fare) {
                            if let Some(nokta) = noktalar.iter().find(|n| n.sıra == *sıra) {
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
                }
                Seri::Pasta(_) => {}
            }
        }

        // Eksen imleci çizgisi + eksen ipucu penceresi.
        if let Some(eksen_ip) = eksen_ipucu {
            if let Some(ipucu) = &ipucu_seçeneği {
                if ipucu.imleç == İmleçTürü::Çizgi || ipucu.imleç == İmleçTürü::Çapraz {
                    let bant_x = kartezyen.x.ölçek.kategorik_mi();
                    let bant_ekseni = if bant_x { &kartezyen.x } else { &kartezyen.y };
                    let merkez = bant_ekseni.veriden_piksele(eksen_ip.kategori_sırası as f64);
                    let alan = kartezyen.alan;
                    if bant_x {
                        çizici.çizgi(
                            (merkez, alan.y),
                            (merkez, alan.alt()),
                            1.0,
                            tema::İMLEÇ_ÇİZGİSİ,
                            ÇizgiTürü::Düz,
                        );
                    } else {
                        çizici.çizgi(
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
    }

    // 5) Pasta serileri.
    let tüm_alan = Dikdörtgen::yeni(0.0, 0.0, çizici.genişlik, çizici.yükseklik);
    for seri in &seçenekler.seriler {
        let Seri::Pasta(p) = seri else { continue };
        if !ad_görünür(seri.ad(), &durum.kapalı) {
            continue;
        }
        let dilimler: Vec<Dilim> =
            pasta_yerleşimi(p, seçenekler, tüm_alan, &durum.kapalı, durum.ilerleme);

        // Öğe ipucu: fare hangi dilimde?
        let vurgu = match (&ipucu_seçeneği, fare) {
            (Some(ipucu), Some(f)) if ipucu.tetikleme != Tetikleme::Kapalı => dilimler
                .iter()
                .position(|d| d.içeriyor_mu(f)),
            _ => None,
        };

        pasta_çiz(&mut çizici, p, &dilimler, vurgu);

        if let (Some(sıra), Some(f)) = (vurgu, fare) {
            let dilim = &dilimler[sıra];
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

    // 6) İpucu penceresi (her şeyin üstüne).
    if let (Some(ipucu), Some((başlık, satırlar, konum))) = (&ipucu_seçeneği, bekleyen_ipucu) {
        ipucu_çiz(&mut çizici, ipucu, konum, başlık.as_deref(), &satırlar);
    }
}
