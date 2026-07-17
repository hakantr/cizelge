//! gpui penceresi — ECharts örneğinin (`echarts.init` + `setOption`) gpui
//! yapıştırması: tuval, fare/tekerlek olayları, animasyon kareleri, olay
//! yayını. Boyama hattının kendisi [`crate::cizim::gorunum::grafiği_boya`]
//! içindedir ve bu modül olmadan da (ör. WASM/SVG hedeflerinde) çalışır.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use gpui::{
    Bounds, Context, EventEmitter, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Render, ScrollWheelEvent, Window, canvas, div, prelude::*,
};

use crate::bilesen::zaman_seridi::ZamanŞeridiEylemi;
use crate::cizim::cizici::{Çizici, ÖlçümÖnbelleği};
use crate::cizim::gorunum::{
    grafiği_boya, AraçTürü, BoyamaGirdisi, SürgüBölgesi, SürgüParçası,
    İçYakınlaştırmaAlanı,
};
use crate::cizim::olay::{GrafikOlayı, İsabetBölgesi};
use crate::hata::{BilesenHatasi, BilesenTanisi};
use crate::koordinat::Dikdörtgen;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::Seri;

/// Gösterge öğelerinin pencere-mutlak isabet kutuları (tıklama için).
type GöstergeKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, String)>>>;

/// Parçalı eşleme dilimlerinin pencere-mutlak kutuları.
type EşlemeKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, usize)>>>;

/// Gösterge kaydırma oklarının pencere-mutlak kutuları.
type OkKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, i32)>>>;

/// Araç kutusu düğmelerinin pencere-mutlak kutuları.
type AraçKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, AraçTürü)>>>;

/// Zaman şeridi düğmelerinin pencere-mutlak kutuları.
type FilmDüğmeleri = Rc<RefCell<Vec<(Bounds<Pixels>, ZamanŞeridiEylemi)>>>;

/// Hiyerarşi kırıntılarının pencere-mutlak kutuları: `(kutu, yeni yol uzunluğu)`.
type KırıntıKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, usize)>>>;

/// Zaman şeridi (timeline) durumu: kare listesi + oynatma.
struct Film {
    kareler: Vec<Arc<GrafikSeçenekleri>>,
    geçerli: usize,
    oynuyor: bool,
    aralık_ms: f32,
    son_geçiş: Instant,
}

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
    /// Pencere-mutlak sürgü etkileşim bölgeleri.
    sürgü_bölgeleri: Rc<RefCell<Vec<SürgüBölgesi>>>,
    /// Pencere-mutlak parçalı eşleme dilim kutuları.
    eşleme_kutuları: EşlemeKutuları,
    /// Pencere-mutlak gösterge kaydırma okları.
    gösterge_okları: OkKutuları,
    /// Pencere-mutlak araç kutusu düğmeleri.
    araç_düğmeleri: AraçKutuları,
    /// Pencere-mutlak iç yakınlaştırma alanları.
    iç_yakınlaştırma_alanları: Rc<RefCell<Vec<İçYakınlaştırmaAlanı>>>,
    /// Etkin sürükleme (kaydırma ya da sürgü).
    sürükleme: Option<Sürükleme>,
    /// Kaydırmalı göstergenin sayfası.
    gösterge_sayfası: usize,
    /// Etkin fırça seçimi: (başlangıç, şimdiki) pencere-mutlak.
    fırça_seçimi: Option<((f32, f32), (f32, f32))>,
    /// İlk seçenekler (araç kutusu "geri yükle" için).
    ilk_seçenekler: Arc<GrafikSeçenekleri>,
    ölçüm_önbelleği: ÖlçümÖnbelleği,
    /// Zaman şeridi (timeline) durumu.
    film: Option<Film>,
    /// Son çizimdeki tuval boyutu (SVG kaydetme için).
    son_boyut: Rc<Cell<(f32, f32)>>,
    /// Pencere-mutlak zaman şeridi düğmeleri.
    film_düğmeleri: FilmDüğmeleri,
    /// Hiyerarşik gezinme yolu (ağaç haritası inme / güneş odak).
    hiyerarşi_yolu: Vec<String>,
    /// Pencere-mutlak kırıntı kutuları.
    kırıntı_kutuları: KırıntıKutuları,
    /// Grafo gezinmesi: `(kayma_x, kayma_y, ölçek)`.
    grafo_görünümü: (f32, f32, f32),
    /// Grafo düğümü sürükleme kaymaları.
    grafo_kaymaları: std::collections::HashMap<usize, (f32, f32)>,
}

/// Etkin sürükleme durumu.
#[derive(Clone, Copy, Debug)]
enum Sürükleme {
    /// Grafo düğümünü taşıma.
    GrafoDüğüm { veri_sırası: usize, son: (f32, f32) },
    /// Grafo görünümünü kaydırma (roam).
    GrafoKaydırma { son: (f32, f32) },
    /// Izgara içinde yatay kaydırma (pan).
    Kaydırma {
        yakınlaştırma_sırası: usize,
        başlangıç_x: f32,
        pencere: (f32, f32),
        alan_genişliği: f32,
    },
    /// Sürgü parçası sürükleme.
    Sürgü {
        yakınlaştırma_sırası: usize,
        parça: SürgüParçası,
        başlangıç_x: f32,
        pencere: (f32, f32),
        şerit_genişliği: f32,
    },
}

impl EventEmitter<GrafikOlayı> for GrafikGörünümü {}
impl EventEmitter<BilesenTanisi> for GrafikGörünümü {}

impl GrafikGörünümü {
    pub fn yeni(seçenekler: GrafikSeçenekleri) -> Self {
        let seçenekler = Arc::new(seçenekler);
        GrafikGörünümü {
            ilk_seçenekler: seçenekler.clone(),
            seçenekler,
            başlangıç: Instant::now(),
            eski_seçenekler: None,
            geçiş_başlangıcı: None,
            fare: None,
            kapalı: HashSet::new(),
            gösterge_kutuları: Rc::new(RefCell::new(Vec::new())),
            isabetler: Rc::new(RefCell::new(Vec::new())),
            bekleyen_tanılar: Rc::new(RefCell::new(Vec::new())),
            sürgü_bölgeleri: Rc::new(RefCell::new(Vec::new())),
            eşleme_kutuları: Rc::new(RefCell::new(Vec::new())),
            gösterge_okları: Rc::new(RefCell::new(Vec::new())),
            araç_düğmeleri: Rc::new(RefCell::new(Vec::new())),
            iç_yakınlaştırma_alanları: Rc::new(RefCell::new(Vec::new())),
            sürükleme: None,
            gösterge_sayfası: 0,
            fırça_seçimi: None,
            ölçüm_önbelleği: Rc::new(RefCell::new(std::collections::HashMap::new())),
            film: None,
            film_düğmeleri: Rc::new(RefCell::new(Vec::new())),
            son_boyut: Rc::new(Cell::new((800.0, 600.0))),
            hiyerarşi_yolu: Vec::new(),
            kırıntı_kutuları: Rc::new(RefCell::new(Vec::new())),
            grafo_görünümü: (0.0, 0.0, 1.0),
            grafo_kaymaları: std::collections::HashMap::new(),
        }
    }

    /// Zaman şeridiyle (timeline) kurulum: kare başına tam seçenekler.
    /// Geçersiz kareler atlanır; kalan kareler `aralık_ms` aralıkla
    /// kendiliğinden oynatılır. Kare geçişleri veri geçiş animasyonuyla
    /// yumuşatılır ve [`GrafikOlayı::ZamanKaresiDeğişti`] yayımlanır.
    pub fn film(kareler: Vec<GrafikSeçenekleri>, aralık_ms: f32) -> Self {
        let geçerli_kareler: Vec<Arc<GrafikSeçenekleri>> = kareler
            .into_iter()
            .filter(|k| k.doğrula().is_ok())
            .map(Arc::new)
            .collect();
        let ilk = geçerli_kareler
            .first()
            .map(|k| (**k).clone())
            .unwrap_or_default();
        let mut görünüm = Self::yeni(ilk);
        if !geçerli_kareler.is_empty() {
            görünüm.film = Some(Film {
                kareler: geçerli_kareler,
                geçerli: 0,
                oynuyor: true,
                aralık_ms: aralık_ms.max(100.0),
                son_geçiş: Instant::now(),
            });
        }
        görünüm
    }

    /// Zaman şeridinde verilen kareye geçer (geçiş animasyonlu).
    pub fn kare_seç(&mut self, sıra: usize, cx: &mut Context<Self>) {
        let Some(film) = &mut self.film else { return };
        if film.kareler.is_empty() {
            return;
        }
        let sıra = sıra % film.kareler.len();
        if sıra == film.geçerli {
            return;
        }
        film.geçerli = sıra;
        film.son_geçiş = Instant::now();
        let Some(kare) = film.kareler.get(sıra).cloned() else { return };
        if self.seçenekler.animasyon {
            self.eski_seçenekler = Some(self.seçenekler.clone());
            self.geçiş_başlangıcı = Some(Instant::now());
        }
        self.seçenekler = kare.clone();
        self.ilk_seçenekler = kare;
        cx.emit(GrafikOlayı::ZamanKaresiDeğişti { sıra });
        cx.notify();
    }

    /// Grafiği çalışma dizinine SVG dosyası olarak kaydeder
    /// (`saveAsImage`). Başarıda [`GrafikOlayı::SvgKaydedildi`] yayımlanır;
    /// yazma hatası tanı olayına dönüşür (panik yok).
    pub fn svg_kaydet(&mut self, cx: &mut Context<Self>) {
        let (genişlik, yükseklik) = self.son_boyut.get();
        let svg = crate::cizim::svg_dışa_aktar(&self.seçenekler, genişlik, yükseklik);
        let damga = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or_default();
        let yol = format!("cizelge-{damga}.svg");
        match std::fs::write(&yol, svg) {
            Ok(()) => cx.emit(GrafikOlayı::SvgKaydedildi { yol }),
            Err(hata) => cx.emit(BilesenTanisi::yeni(
                "svg_kaydet",
                BilesenHatasi::GeçersizSeçenek {
                    alan: "araç_kutusu.svg_kaydet",
                    ayrıntı: format!("`{yol}` yazılamadı: {hata}"),
                },
            )),
        }
    }

    /// Grafiği çalışma dizinine PNG dosyası olarak kaydeder
    /// (`saveAsImage`, `type: 'png'`; 2× piksel oranı). `png` özelliği
    /// kapalıysa tanı yayımlanır.
    pub fn png_kaydet(&mut self, cx: &mut Context<Self>) {
        #[cfg(feature = "png")]
        {
            let (genişlik, yükseklik) = self.son_boyut.get();
            let damga = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|s| s.as_secs())
                .unwrap_or_default();
            let yol = format!("cizelge-{damga}.png");
            let sonuç = crate::cizim::piksel::png_dışa_aktar(
                &self.seçenekler,
                genişlik,
                yükseklik,
                2.0,
            )
            .and_then(|baytlar| {
                std::fs::write(&yol, baytlar).map_err(|hata| {
                    BilesenHatasi::GeçersizSeçenek {
                        alan: "araç_kutusu.png_kaydet",
                        ayrıntı: format!("`{yol}` yazılamadı: {hata}"),
                    }
                })
            });
            match sonuç {
                Ok(()) => cx.emit(GrafikOlayı::PngKaydedildi { yol }),
                Err(hata) => cx.emit(BilesenTanisi::yeni("png_kaydet", hata)),
            }
        }
        #[cfg(not(feature = "png"))]
        cx.emit(BilesenTanisi::yeni(
            "png_kaydet",
            BilesenHatasi::GeçersizSeçenek {
                alan: "araç_kutusu.png_kaydet",
                ayrıntı: "`png` özelliği derlemede kapalı".to_string(),
            },
        ));
    }

    /// Oynatmayı açar/kapatır.
    pub fn oynat_durdur(&mut self, cx: &mut Context<Self>) {
        if let Some(film) = &mut self.film {
            film.oynuyor = !film.oynuyor;
            film.son_geçiş = Instant::now();
            cx.notify();
        }
    }

    /// Oynatma sırasında kare ilerletir; şerit durumunu döndürür.
    fn film_ilerlet(&mut self, cx: &mut Context<Self>) -> Option<(usize, usize, bool)> {
        let (ilerlet, durum) = match &self.film {
            Some(film) => {
                let bekleme_bitti = film.oynuyor
                    && film.son_geçiş.elapsed().as_secs_f32() * 1000.0 >= film.aralık_ms;
                (
                    bekleme_bitti,
                    Some((film.geçerli, film.kareler.len(), film.oynuyor)),
                )
            }
            None => (false, None),
        };
        if ilerlet {
            let sonraki = durum
                .map(|(geçerli, toplam, _)| {
                    if toplam == 0 { 0 } else { (geçerli + 1) % toplam }
                })
                .unwrap_or(0);
            self.kare_seç(sonraki, cx);
            return self
                .film
                .as_ref()
                .map(|f| (f.geçerli, f.kareler.len(), f.oynuyor));
        }
        durum
    }

    /// Yakınlaştırma penceresini günceller, olay yayımlar.
    fn pencereyi_güncelle(
        &mut self,
        sıra: usize,
        başlangıç: f32,
        bitiş: f32,
        cx: &mut Context<Self>,
    ) {
        let başlangıç = başlangıç.clamp(0.0, 100.0);
        let bitiş = bitiş.clamp(başlangıç + 1.0, 100.0);
        let seçenekler = Arc::make_mut(&mut self.seçenekler);
        if let Some(y) = seçenekler.veri_yakınlaştırmaları.get_mut(sıra) {
            if (y.başlangıç - başlangıç).abs() < 0.01 && (y.bitiş - bitiş).abs() < 0.01 {
                return;
            }
            y.başlangıç = başlangıç;
            y.bitiş = bitiş;
            cx.emit(GrafikOlayı::YakınlaştırmaDeğişti { sıra, başlangıç, bitiş });
            cx.notify();
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
        self.ilk_seçenekler = self.seçenekler.clone();
        self.gösterge_sayfası = 0;
        self.fırça_seçimi = None;
        self.gezinmeyi_sıfırla();
        cx.notify();
        Ok(())
    }

    /// Gezinme durumunu (hiyerarşi yolu, grafo görünümü) sıfırlar.
    fn gezinmeyi_sıfırla(&mut self) {
        self.hiyerarşi_yolu.clear();
        self.grafo_görünümü = (0.0, 0.0, 1.0);
        self.grafo_kaymaları.clear();
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

        // Zaman şeridi: oynatma sırasında kare ilerlet (geçiş animasyonunu
        // tetikler) ve şerit durumunu boyamaya taşı.
        let zaman_şeridi = self.film_ilerlet(cx);

        let (etkin_seçenekler, ilerleme, sürüyor) = self.boyama_durumu();
        // Dalga efektli seriler ve oynayan zaman şeridi sürekli kare ister.
        let sürekli = etkin_seçenekler
            .seriler
            .iter()
            .any(|s| matches!(s, Seri::Saçılım(sa) if sa.efektli))
            || zaman_şeridi.map(|(_, _, oynuyor)| oynuyor).unwrap_or(false);
        if sürüyor || sürekli {
            pencere.request_animation_frame();
        }
        let zaman_sn = self.başlangıç.elapsed().as_secs_f32();

        let fare = self.fare;
        let kapalı = self.kapalı.clone();
        let gösterge_sayfası = self.gösterge_sayfası;
        let fırça = self.fırça_seçimi.map(|(b, ş)| [b.0, b.1, ş.0, ş.1]);
        let gösterge_kutuları = self.gösterge_kutuları.clone();
        let isabetler = self.isabetler.clone();
        let tanılar = self.bekleyen_tanılar.clone();
        let sürgüler = self.sürgü_bölgeleri.clone();
        let iç_alanlar = self.iç_yakınlaştırma_alanları.clone();
        let eşleme_kutuları = self.eşleme_kutuları.clone();
        let gösterge_okları = self.gösterge_okları.clone();
        let araç_düğmeleri = self.araç_düğmeleri.clone();
        let film_düğmeleri = self.film_düğmeleri.clone();
        let kırıntı_kutuları = self.kırıntı_kutuları.clone();
        let son_boyut = self.son_boyut.clone();
        let hiyerarşi_yolu = self.hiyerarşi_yolu.clone();
        let grafo_görünümü = self.grafo_görünümü;
        let grafo_kaymaları: Vec<(usize, f32, f32)> = self
            .grafo_kaymaları
            .iter()
            .map(|(sıra, (dx, dy))| (*sıra, *dx, *dy))
            .collect();
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
                        son_boyut.set((
                            f32::from(sınırlar.size.width),
                            f32::from(sınırlar.size.height),
                        ));
                        let yerel_fare = fare.map(|(x, y)| (x - köken.0, y - köken.1));
                        let girdi = BoyamaGirdisi {
                            ilerleme,
                            zaman_sn,
                            fare: yerel_fare,
                            kapalı: kapalı.clone(),
                            gösterge_sayfası,
                            fırça: fırça.map(|[x0, y0, x1, y1]| {
                                [x0 - köken.0, y0 - köken.1, x1 - köken.0, y1 - köken.1]
                            }),
                            zaman_şeridi,
                            hiyerarşi_yolu: hiyerarşi_yolu.clone(),
                            grafo_görünümü,
                            grafo_kaymaları: grafo_kaymaları.clone(),
                        };
                        let çıktı = grafiği_boya(&mut çizici, &etkin_seçenekler, &girdi);
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
                        let kaydırılmış = |d: Dikdörtgen| {
                            Dikdörtgen::yeni(
                                d.x + köken.0,
                                d.y + köken.1,
                                d.genişlik,
                                d.yükseklik,
                            )
                        };
                        match sürgüler.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                kayıt.clear();
                                for s in çıktı.sürgüler {
                                    kayıt.push(SürgüBölgesi {
                                        yakınlaştırma_sırası: s.yakınlaştırma_sırası,
                                        şerit: kaydırılmış(s.şerit),
                                        pencere: kaydırılmış(s.pencere),
                                        sol_tutamaç: kaydırılmış(s.sol_tutamaç),
                                        sağ_tutamaç: kaydırılmış(s.sağ_tutamaç),
                                    });
                                }
                            }
                            Err(_) => tanı_bildir("sürgü_bölgeleri"),
                        }
                        match iç_alanlar.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                kayıt.clear();
                                for a in çıktı.iç_yakınlaştırmalar {
                                    kayıt.push(İçYakınlaştırmaAlanı {
                                        yakınlaştırma_sırası: a.yakınlaştırma_sırası,
                                        alan: kaydırılmış(a.alan),
                                    });
                                }
                            }
                            Err(_) => tanı_bildir("iç_yakınlaştırma_alanları"),
                        }
                        match eşleme_kutuları.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                kayıt.clear();
                                for (kutu, sıra) in çıktı.eşleme_kutuları {
                                    kayıt.push((çizici.sınırlar(kutu), sıra));
                                }
                            }
                            Err(_) => tanı_bildir("eşleme_kutuları"),
                        }
                        match gösterge_okları.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                kayıt.clear();
                                for (kutu, yön) in çıktı.gösterge_okları {
                                    kayıt.push((çizici.sınırlar(kutu), yön));
                                }
                            }
                            Err(_) => tanı_bildir("gösterge_okları"),
                        }
                        match araç_düğmeleri.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                kayıt.clear();
                                for (kutu, tür) in çıktı.araç_düğmeleri {
                                    kayıt.push((çizici.sınırlar(kutu), tür));
                                }
                            }
                            Err(_) => tanı_bildir("araç_düğmeleri"),
                        }
                        match film_düğmeleri.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                kayıt.clear();
                                for (kutu, eylem) in çıktı.zaman_düğmeleri {
                                    kayıt.push((çizici.sınırlar(kutu), eylem));
                                }
                            }
                            Err(_) => tanı_bildir("zaman_düğmeleri"),
                        }
                        match kırıntı_kutuları.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                kayıt.clear();
                                for (kutu, uzunluk) in çıktı.kırıntılar {
                                    kayıt.push((çizici.sınırlar(kutu), uzunluk));
                                }
                            }
                            Err(_) => tanı_bildir("kırıntı_kutuları"),
                        }
                    },
                )
                .size_full(),
            )
            .on_mouse_move(cx.listener(|bu, olay: &MouseMoveEvent, _, cx| {
                let yeni = (f32::from(olay.position.x), f32::from(olay.position.y));
                // Fırça seçimi sürüyor.
                if olay.pressed_button == Some(MouseButton::Left)
                    && let Some((_, şimdiki)) = bu.fırça_seçimi.as_mut() {
                        *şimdiki = yeni;
                        cx.notify();
                        return;
                    }
                // Etkin sürükleme: kaydırma ya da sürgü.
                if olay.pressed_button == Some(MouseButton::Left) {
                    match bu.sürükleme {
                        Some(Sürükleme::GrafoDüğüm { veri_sırası, son }) => {
                            let fark = (yeni.0 - son.0, yeni.1 - son.1);
                            let kayıt = bu
                                .grafo_kaymaları
                                .entry(veri_sırası)
                                .or_insert((0.0, 0.0));
                            kayıt.0 += fark.0;
                            kayıt.1 += fark.1;
                            bu.sürükleme =
                                Some(Sürükleme::GrafoDüğüm { veri_sırası, son: yeni });
                            cx.notify();
                            return;
                        }
                        Some(Sürükleme::GrafoKaydırma { son }) => {
                            bu.grafo_görünümü.0 += yeni.0 - son.0;
                            bu.grafo_görünümü.1 += yeni.1 - son.1;
                            bu.sürükleme = Some(Sürükleme::GrafoKaydırma { son: yeni });
                            cx.notify();
                            return;
                        }
                        Some(Sürükleme::Kaydırma {
                            yakınlaştırma_sırası,
                            başlangıç_x,
                            pencere,
                            alan_genişliği,
                        }) => {
                            let oran_farkı =
                                (yeni.0 - başlangıç_x) / alan_genişliği.max(1.0);
                            let genişlik = pencere.1 - pencere.0;
                            // İçerik fareyle sürüklenir: pencere ters yönde kayar.
                            let kayma = -oran_farkı * genişlik * 100.0;
                            let b = (pencere.0 * 100.0 + kayma)
                                .clamp(0.0, 100.0 - genişlik * 100.0);
                            bu.pencereyi_güncelle(
                                yakınlaştırma_sırası,
                                b,
                                b + genişlik * 100.0,
                                cx,
                            );
                            return;
                        }
                        Some(Sürükleme::Sürgü {
                            yakınlaştırma_sırası,
                            parça,
                            başlangıç_x,
                            pencere,
                            şerit_genişliği,
                        }) => {
                            let oran_farkı =
                                (yeni.0 - başlangıç_x) / şerit_genişliği.max(1.0) * 100.0;
                            let (b0, e0) = (pencere.0 * 100.0, pencere.1 * 100.0);
                            let (b, e) = match parça {
                                SürgüParçası::SolTutamaç => {
                                    ((b0 + oran_farkı).min(e0 - 1.0), e0)
                                }
                                SürgüParçası::SağTutamaç => {
                                    (b0, (e0 + oran_farkı).max(b0 + 1.0))
                                }
                                SürgüParçası::Pencere => {
                                    let genişlik = e0 - b0;
                                    let b =
                                        (b0 + oran_farkı).clamp(0.0, 100.0 - genişlik);
                                    (b, b + genişlik)
                                }
                            };
                            bu.pencereyi_güncelle(yakınlaştırma_sırası, b, e, cx);
                            return;
                        }
                        None => {}
                    }
                } else if bu.sürükleme.is_some() {
                    bu.sürükleme = None;
                }
                if bu.fare != Some(yeni) {
                    bu.fare = Some(yeni);
                    cx.notify();
                }
            }))
            .on_scroll_wheel(cx.listener(|bu, olay: &ScrollWheelEvent, _, cx| {
                // Fare hangi iç yakınlaştırma alanındaysa pencereyi ölçekle.
                let konum = (f32::from(olay.position.x), f32::from(olay.position.y));
                let alan_kaydı = match bu.iç_yakınlaştırma_alanları.try_borrow() {
                    Ok(alanlar) => alanlar
                        .iter()
                        .find(|a| a.alan.içeriyor_mu(konum))
                        .cloned(),
                    Err(_) => None,
                };
                let Some(kayıt) = alan_kaydı else {
                    // Grafo gezinmesi (roam): tekerlek görünümü ölçekler.
                    if bu.seçenekler.seriler.iter().any(|s| matches!(s, Seri::Grafo(_))) {
                        let yön = match olay.delta {
                            gpui::ScrollDelta::Pixels(p) => f32::from(p.y),
                            gpui::ScrollDelta::Lines(l) => l.y * 20.0,
                        };
                        if yön.abs() < 0.01 {
                            return;
                        }
                        let çarpan = if yön > 0.0 { 1.0 / 0.85 } else { 0.85 };
                        let (kayma_x, kayma_y, ölçek) = bu.grafo_görünümü;
                        let yeni_ölçek = (ölçek * çarpan).clamp(0.2, 8.0);
                        let gerçek_çarpan = yeni_ölçek / ölçek.max(1e-6);
                        // Merkez odaklı: kaymalar ölçekle birlikte büyür.
                        bu.grafo_görünümü = (
                            kayma_x * gerçek_çarpan,
                            kayma_y * gerçek_çarpan,
                            yeni_ölçek,
                        );
                        cx.notify();
                    }
                    return;
                };
                let pencere = bu
                    .seçenekler
                    .veri_yakınlaştırmaları
                    .get(kayıt.yakınlaştırma_sırası)
                    .map(|y| y.oranlar());
                let Some((b, e)) = pencere else { return };
                let yön = match olay.delta {
                    gpui::ScrollDelta::Pixels(p) => f32::from(p.y),
                    gpui::ScrollDelta::Lines(l) => l.y * 20.0,
                };
                if yön.abs() < 0.01 {
                    return;
                }
                // Yukarı tekerlek = yakınlaş.
                let çarpan = if yön > 0.0 { 0.85 } else { 1.0 / 0.85 };
                let imleç_oranı =
                    ((konum.0 - kayıt.alan.x) / kayıt.alan.genişlik.max(1.0)).clamp(0.0, 1.0);
                let odak = b + (e - b) * imleç_oranı;
                let yeni_b = (odak - (odak - b) * çarpan).max(0.0);
                let yeni_e = (odak + (e - odak) * çarpan).min(1.0);
                if yeni_e - yeni_b >= 0.01 {
                    bu.pencereyi_güncelle(
                        kayıt.yakınlaştırma_sırası,
                        yeni_b * 100.0,
                        yeni_e * 100.0,
                        cx,
                    );
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|bu, _: &MouseUpEvent, _, cx| {
                    bu.sürükleme = None;
                    if let Some((başlangıç, şimdiki)) = bu.fırça_seçimi.take() {
                        let x0 = başlangıç.0.min(şimdiki.0);
                        let x1 = başlangıç.0.max(şimdiki.0);
                        let y0 = başlangıç.1.min(şimdiki.1);
                        let y1 = başlangıç.1.max(şimdiki.1);
                        if x1 - x0 > 3.0 && y1 - y0 > 3.0 {
                            let öğeler: Vec<(usize, usize)> =
                                match bu.isabetler.try_borrow() {
                                    Ok(bölgeler) => bölgeler
                                        .iter()
                                        .filter(|b| {
                                            let (mx, my) = b.geometri.merkez();
                                            mx >= x0 && mx <= x1 && my >= y0 && my <= y1
                                        })
                                        .map(|b| (b.seri_sırası, b.veri_sırası))
                                        .collect(),
                                    Err(_) => Vec::new(),
                                };
                            cx.emit(GrafikOlayı::FırçaSeçildi { öğeler });
                        }
                        cx.notify();
                    }
                }),
            )
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
                    let konum = (f32::from(olay.position.x), f32::from(olay.position.y));
                    // 0) Gösterge kaydırma okları.
                    let ok_vuruşu = match bu.gösterge_okları.try_borrow() {
                        Ok(oklar) => oklar
                            .iter()
                            .find(|(kutu, _)| kutu.contains(&olay.position))
                            .map(|(_, yön)| *yön),
                        Err(_) => None,
                    };
                    if let Some(yön) = ok_vuruşu {
                        if yön < 0 {
                            bu.gösterge_sayfası = bu.gösterge_sayfası.saturating_sub(1);
                        } else {
                            bu.gösterge_sayfası = bu.gösterge_sayfası.saturating_add(1);
                        }
                        cx.notify();
                        return;
                    }
                    // 0b) Araç kutusu düğmeleri.
                    let araç_vuruşu = match bu.araç_düğmeleri.try_borrow() {
                        Ok(düğmeler) => düğmeler
                            .iter()
                            .find(|(kutu, _)| kutu.contains(&olay.position))
                            .map(|(_, tür)| *tür),
                        Err(_) => None,
                    };
                    match araç_vuruşu {
                        Some(AraçTürü::GeriYükle) => {
                            bu.seçenekler = bu.ilk_seçenekler.clone();
                            bu.kapalı.clear();
                            bu.gösterge_sayfası = 0;
                            bu.fırça_seçimi = None;
                            bu.gezinmeyi_sıfırla();
                            cx.emit(GrafikOlayı::GeriYüklendi);
                            cx.notify();
                            return;
                        }
                        Some(AraçTürü::SvgKaydet) => {
                            bu.svg_kaydet(cx);
                            return;
                        }
                        Some(AraçTürü::PngKaydet) => {
                            bu.png_kaydet(cx);
                            return;
                        }
                        None => {}
                    }
                    // 0c) Zaman şeridi düğmeleri.
                    let film_vuruşu = match bu.film_düğmeleri.try_borrow() {
                        Ok(düğmeler) => düğmeler
                            .iter()
                            .find(|(kutu, _)| kutu.contains(&olay.position))
                            .map(|(_, eylem)| *eylem),
                        Err(_) => None,
                    };
                    match film_vuruşu {
                        Some(ZamanŞeridiEylemi::Kare(sıra)) => {
                            bu.kare_seç(sıra, cx);
                            return;
                        }
                        Some(ZamanŞeridiEylemi::OynatDurdur) => {
                            bu.oynat_durdur(cx);
                            return;
                        }
                        None => {}
                    }
                    // 0d) Hiyerarşi kırıntıları (breadcrumb / güneş geri).
                    let kırıntı_vuruşu = match bu.kırıntı_kutuları.try_borrow() {
                        Ok(kutular) => kutular
                            .iter()
                            .find(|(kutu, _)| kutu.contains(&olay.position))
                            .map(|(_, uzunluk)| *uzunluk),
                        Err(_) => None,
                    };
                    if let Some(uzunluk) = kırıntı_vuruşu {
                        bu.hiyerarşi_yolu.truncate(uzunluk);
                        cx.notify();
                        return;
                    }
                    // 0c) Fırça etkinse seçim başlat.
                    if bu.seçenekler.fırça.map(|f| f.etkin).unwrap_or(false) {
                        bu.fırça_seçimi = Some((konum, konum));
                        cx.notify();
                        return;
                    }
                    // 1a) Parçalı görsel eşleme dilimi aç/kapat.
                    let eşleme_vuruşu = match bu.eşleme_kutuları.try_borrow() {
                        Ok(kutular) => kutular
                            .iter()
                            .find(|(kutu, _)| kutu.contains(&olay.position))
                            .map(|(_, sıra)| *sıra),
                        Err(_) => None,
                    };
                    if let Some(sıra) = eşleme_vuruşu {
                        let seçenekler = Arc::make_mut(&mut bu.seçenekler);
                        if let Some(eşleme) = seçenekler.görsel_eşleme.as_mut() {
                            if let Some(k) =
                                eşleme.kapalı_parçalar.iter().position(|k| *k == sıra)
                            {
                                eşleme.kapalı_parçalar.remove(k);
                            } else {
                                eşleme.kapalı_parçalar.push(sıra);
                            }
                            cx.notify();
                        }
                        return;
                    }
                    // 1b) Sürgü parçası sürüklemesi.
                    let sürgü_vuruşu = match bu.sürgü_bölgeleri.try_borrow() {
                        Ok(bölgeler) => bölgeler.iter().find_map(|s| {
                            let parça = if s.sol_tutamaç.içeriyor_mu(konum) {
                                Some(SürgüParçası::SolTutamaç)
                            } else if s.sağ_tutamaç.içeriyor_mu(konum) {
                                Some(SürgüParçası::SağTutamaç)
                            } else if s.pencere.içeriyor_mu(konum) {
                                Some(SürgüParçası::Pencere)
                            } else {
                                None
                            };
                            parça.map(|p| (s.yakınlaştırma_sırası, p, s.şerit.genişlik))
                        }),
                        Err(_) => None,
                    };
                    if let Some((sıra, parça, şerit_genişliği)) = sürgü_vuruşu {
                        if let Some(y) = bu.seçenekler.veri_yakınlaştırmaları.get(sıra) {
                            bu.sürükleme = Some(Sürükleme::Sürgü {
                                yakınlaştırma_sırası: sıra,
                                parça,
                                başlangıç_x: konum.0,
                                pencere: y.oranlar(),
                                şerit_genişliği,
                            });
                        }
                        return;
                    }
                    // 1c) İç yakınlaştırma alanında kaydırma başlat.
                    let iç_vuruş = match bu.iç_yakınlaştırma_alanları.try_borrow() {
                        Ok(alanlar) => alanlar
                            .iter()
                            .find(|a| a.alan.içeriyor_mu(konum))
                            .cloned(),
                        Err(_) => None,
                    };
                    if let Some(kayıt) = iç_vuruş
                        && let Some(y) = bu
                            .seçenekler
                            .veri_yakınlaştırmaları
                            .get(kayıt.yakınlaştırma_sırası)
                        {
                            bu.sürükleme = Some(Sürükleme::Kaydırma {
                                yakınlaştırma_sırası: kayıt.yakınlaştırma_sırası,
                                başlangıç_x: konum.0,
                                pencere: y.oranlar(),
                                alan_genişliği: kayıt.alan.genişlik,
                            });
                        }
                    // 2) Veri öğesi tıklaması: en üstte çizilen bölge kazanır.
                    let nokta = konum;
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
                        match bu.seçenekler.seriler.get(b.seri_sırası) {
                            // Ağaç haritası / güneş patlaması: dala in (odakla).
                            Some(Seri::AğaçHaritası(_) | Seri::GüneşPatlaması(_)) => {
                                if let Some(ad) = &b.ad {
                                    bu.hiyerarşi_yolu.push(ad.clone());
                                    cx.notify();
                                }
                            }
                            // Grafo düğümü: sürüklemeyi başlat.
                            Some(Seri::Grafo(_)) => {
                                bu.sürükleme = Some(Sürükleme::GrafoDüğüm {
                                    veri_sırası: b.veri_sırası,
                                    son: nokta,
                                });
                            }
                            _ => {}
                        }
                        cx.emit(GrafikOlayı::ÖğeTıklandı {
                            seri_sırası: b.seri_sırası,
                            veri_sırası: b.veri_sırası,
                            seri_adı: b.seri_adı,
                            ad: b.ad,
                            değer: b.değer,
                        });
                    } else if bu
                        .seçenekler
                        .seriler
                        .iter()
                        .any(|s| matches!(s, Seri::Grafo(_)))
                        && !bu.seçenekler.seriler.iter().any(Seri::kartezyen_mi)
                    {
                        // Grafo boş alanı: görünümü kaydırma (roam).
                        bu.sürükleme = Some(Sürükleme::GrafoKaydırma { son: nokta });
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
