//! gpui penceresi — ECharts örneğinin (`echarts.init` + `setOption`) gpui
//! yapıştırması: tuval, fare/tekerlek olayları, animasyon kareleri, olay
//! yayını. Boyama hattının kendisi [`crate::cizim::gorunum::grafiği_boya`]
//! içindedir ve bu modül olmadan da (ör. WASM/SVG hedeflerinde) çalışır.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use gpui::{
    Bounds, Context, CursorStyle, EventEmitter, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Render, ScrollWheelEvent, Window, canvas, div, prelude::*,
};

use crate::bilesen::grafik::GrafikSahnesi;
use crate::bilesen::zaman_seridi::ZamanŞeridiEylemi;
use crate::calisma_zamani::{
    GrafikÇalışmaZamanı, SeçenekAyarlamaKipi, SeçenekYaması, ÖrnekBaşlatmaSeçenekleri,
};
use crate::cizim::cizici::{Çizici, ÖlçümÖnbelleği};
use crate::cizim::gorunum::{
    AraçTürü, BoyamaGirdisi, FırçaAlanı, SürgüBölgesi, SürgüParçası, grafiği_boya, gösterge_adları,
    İçYakınlaştırmaAlanı,
};
use crate::cizim::olay::{GrafikOlayı, MatrisHücreBölgesi, İsabetBölgesi};
use crate::grafik::isi::{GörselEşlemeSürgüParçası, SürekliGörselEşlemeBölgesi};
use crate::hata::{BilesenHatasi, BilesenTanisi};
use crate::koordinat::Dikdörtgen;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::Seri;

fn model_kapalı_adları(seçenekler: &GrafikSeçenekleri) -> HashSet<String> {
    seçenekler
        .gösterge
        .as_ref()
        .into_iter()
        .flat_map(|gösterge| gösterge.seçili.iter())
        .filter_map(|(ad, seçili)| (!*seçili).then_some(ad.clone()))
        .collect()
}

/// Gösterge öğelerinin pencere-mutlak isabet kutuları (tıklama için).
type GöstergeKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, String)>>>;

/// Parçalı eşleme dilimlerinin pencere-mutlak kutuları.
type EşlemeKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, usize)>>>;

/// Sürekli görsel eşleme bölgesinin pencere-mutlak kopyası.
type SürekliEşlemeAlanı = Rc<RefCell<Option<SürekliGörselEşlemeBölgesi>>>;

/// Gösterge kaydırma oklarının pencere-mutlak kutuları.
type OkKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, i32)>>>;

/// Araç kutusu düğmelerinin pencere-mutlak kutuları.
type AraçKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, AraçTürü)>>>;

/// Zaman şeridi düğmelerinin pencere-mutlak kutuları.
type FilmDüğmeleri = Rc<RefCell<Vec<(Bounds<Pixels>, ZamanŞeridiEylemi)>>>;

/// Hiyerarşi kırıntılarının pencere-mutlak kutuları: `(kutu, yeni yol uzunluğu)`.
type KırıntıKutuları = Rc<RefCell<Vec<(Bounds<Pixels>, usize)>>>;

/// Son boyamadaki `graphic` sahnesi ve pencere-mutlak tuval kökeni.
type GrafikSahneKaydı = Rc<RefCell<Option<(GrafikSahnesi, (f32, f32))>>>;

/// Matrix bileşen hücrelerinin pencere-mutlak etkileşim kayıtları.
type MatrisHücreKayıtları = Rc<RefCell<Vec<MatrisHücreBölgesi>>>;

fn gpui_imleci(ad: &str) -> CursorStyle {
    match ad.trim().to_ascii_lowercase().as_str() {
        "pointer" => CursorStyle::PointingHand,
        "text" => CursorStyle::IBeam,
        "vertical-text" => CursorStyle::IBeamCursorForVerticalLayout,
        "crosshair" => CursorStyle::Crosshair,
        "grab" | "open-hand" => CursorStyle::OpenHand,
        "grabbing" | "closed-hand" | "move" => CursorStyle::ClosedHand,
        "not-allowed" | "no-drop" => CursorStyle::OperationNotAllowed,
        "alias" => CursorStyle::DragLink,
        "copy" => CursorStyle::DragCopy,
        "context-menu" => CursorStyle::ContextualMenu,
        "col-resize" => CursorStyle::ResizeColumn,
        "row-resize" => CursorStyle::ResizeRow,
        "ew-resize" => CursorStyle::ResizeLeftRight,
        "ns-resize" => CursorStyle::ResizeUpDown,
        "nesw-resize" => CursorStyle::ResizeUpRightDownLeft,
        "nwse-resize" => CursorStyle::ResizeUpLeftDownRight,
        "n-resize" => CursorStyle::ResizeUp,
        "e-resize" => CursorStyle::ResizeRight,
        "s-resize" => CursorStyle::ResizeDown,
        "w-resize" => CursorStyle::ResizeLeft,
        _ => CursorStyle::Arrow,
    }
}

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
    /// Matrix bileşeninin tooltip/triggerEvent/cursor hedefleri.
    matris_hücreleri: MatrisHücreKayıtları,
    /// Boyama sırasında biriken, bir sonraki karede olay olarak yayımlanacak
    /// tanılar.
    bekleyen_tanılar: Rc<RefCell<Vec<BilesenTanisi>>>,
    /// Pencere-mutlak sürgü etkileşim bölgeleri.
    sürgü_bölgeleri: Rc<RefCell<Vec<SürgüBölgesi>>>,
    /// Pencere-mutlak parçalı eşleme dilim kutuları.
    eşleme_kutuları: EşlemeKutuları,
    /// Pencere-mutlak sürekli eşleme tutamaç/şerit bölgesi.
    sürekli_eşleme_alanı: SürekliEşlemeAlanı,
    /// Pencere-mutlak gösterge kaydırma okları.
    gösterge_okları: OkKutuları,
    /// Pencere-mutlak araç kutusu düğmeleri.
    araç_düğmeleri: AraçKutuları,
    /// Pencere-mutlak iç yakınlaştırma alanları.
    iç_yakınlaştırma_alanları: Rc<RefCell<Vec<İçYakınlaştırmaAlanı>>>,
    /// Pencere-mutlak Kartezyen ızgaralar (lineX/lineY brush sınırı).
    ızgara_alanları: Rc<RefCell<Vec<Dikdörtgen>>>,
    /// Etkin sürükleme (kaydırma ya da sürgü).
    sürükleme: Option<Sürükleme>,
    /// Kaydırmalı göstergenin sayfası.
    gösterge_sayfası: usize,
    /// Sürmekte olan fırça alanı, pencere-mutlak.
    fırça_seçimi: Option<FırçaAlanı>,
    /// Tamamlanmış, temizlenene kadar kalan fırça alanları.
    fırça_alanları: Vec<FırçaAlanı>,
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
    /// Dönüşümlü `graphic` isabet sınamasında kullanılan sahnenin kendisi.
    grafik_sahnesi: GrafikSahneKaydı,
    /// Grafo gezinmesi: `(kayma_x, kayma_y, ölçek)`.
    grafo_görünümü: (f32, f32, f32),
    /// Grafo düğümü sürükleme kaymaları.
    grafo_kaymaları: std::collections::HashMap<usize, (f32, f32)>,
}

/// Etkin sürükleme durumu.
#[derive(Clone, Copy, Debug)]
enum Sürükleme {
    /// Grafo düğümünü taşıma.
    GrafoDüğüm {
        veri_sırası: usize, son: (f32, f32)
    },
    /// Grafo görünümünü kaydırma (roam).
    GrafoKaydırma { son: (f32, f32) },
    /// Izgara içinde yatay kaydırma (pan).
    Kaydırma {
        yakınlaştırma_sırası: usize,
        başlangıç_ekseni: f32,
        pencere: (f32, f32),
        alan_uzunluğu: f32,
        dikey: bool,
    },
    /// Sürgü parçası sürükleme.
    Sürgü {
        yakınlaştırma_sırası: usize,
        parça: SürgüParçası,
        başlangıç_ekseni: f32,
        pencere: (f32, f32),
        şerit_uzunluğu: f32,
        dikey: bool,
    },
    /// Sürekli `visualMap` tutamacı ya da seçili aralık sürükleme.
    GörselEşleme {
        parça: GörselEşlemeSürgüParçası,
        başlangıç_ekseni: f32,
        bölge: SürekliGörselEşlemeBölgesi,
    },
}

impl EventEmitter<GrafikOlayı> for GrafikGörünümü {}
impl EventEmitter<BilesenTanisi> for GrafikGörünümü {}

impl GrafikGörünümü {
    pub fn yeni(seçenekler: GrafikSeçenekleri) -> Self {
        let kapalı = model_kapalı_adları(&seçenekler);
        let seçenekler = Arc::new(seçenekler);
        GrafikGörünümü {
            ilk_seçenekler: seçenekler.clone(),
            seçenekler,
            başlangıç: Instant::now(),
            eski_seçenekler: None,
            geçiş_başlangıcı: None,
            fare: None,
            kapalı,
            gösterge_kutuları: Rc::new(RefCell::new(Vec::new())),
            isabetler: Rc::new(RefCell::new(Vec::new())),
            matris_hücreleri: Rc::new(RefCell::new(Vec::new())),
            bekleyen_tanılar: Rc::new(RefCell::new(Vec::new())),
            sürgü_bölgeleri: Rc::new(RefCell::new(Vec::new())),
            eşleme_kutuları: Rc::new(RefCell::new(Vec::new())),
            sürekli_eşleme_alanı: Rc::new(RefCell::new(None)),
            gösterge_okları: Rc::new(RefCell::new(Vec::new())),
            araç_düğmeleri: Rc::new(RefCell::new(Vec::new())),
            iç_yakınlaştırma_alanları: Rc::new(RefCell::new(Vec::new())),
            ızgara_alanları: Rc::new(RefCell::new(Vec::new())),
            sürükleme: None,
            gösterge_sayfası: 0,
            fırça_seçimi: None,
            fırça_alanları: Vec::new(),
            ölçüm_önbelleği: Rc::new(RefCell::new(std::collections::HashMap::new())),
            film: None,
            film_düğmeleri: Rc::new(RefCell::new(Vec::new())),
            son_boyut: Rc::new(Cell::new((800.0, 600.0))),
            hiyerarşi_yolu: Vec::new(),
            kırıntı_kutuları: Rc::new(RefCell::new(Vec::new())),
            grafik_sahnesi: Rc::new(RefCell::new(None)),
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
        let Some(kare) = film.kareler.get(sıra).cloned() else {
            return;
        };
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
            let sonuç =
                crate::cizim::piksel::png_dışa_aktar(&self.seçenekler, genişlik, yükseklik, 2.0)
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
                    if toplam == 0 {
                        0
                    } else {
                        (geçerli + 1) % toplam
                    }
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
        let Some(kaynak) = seçenekler.veri_yakınlaştırmaları.get(sıra).cloned() else {
            return;
        };
        let bağlı_sıralar = seçenekler
            .veri_yakınlaştırmaları
            .iter()
            .enumerate()
            .filter_map(|(hedef_sırası, yakınlaştırma)| {
                kaynak
                    .aynı_eksenleri_hedefler(yakınlaştırma)
                    .then_some(hedef_sırası)
            })
            .collect::<Vec<_>>();
        let değişti = bağlı_sıralar.iter().any(|hedef_sırası| {
            seçenekler
                .veri_yakınlaştırmaları
                .get(*hedef_sırası)
                .is_some_and(|yakınlaştırma| {
                    (yakınlaştırma.başlangıç - başlangıç).abs() >= 0.01
                        || (yakınlaştırma.bitiş - bitiş).abs() >= 0.01
                })
        });
        if !değişti {
            return;
        }
        for hedef_sırası in bağlı_sıralar {
            if let Some(yakınlaştırma) = seçenekler.veri_yakınlaştırmaları.get_mut(hedef_sırası)
            {
                yakınlaştırma.başlangıç = başlangıç;
                yakınlaştırma.bitiş = bitiş;
            }
        }
        cx.emit(GrafikOlayı::YakınlaştırmaDeğişti {
            sıra,
            başlangıç,
            bitiş,
        });
        cx.notify();
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
        let giriş_bitti =
            self.başlangıç.elapsed().as_secs_f32() * 1000.0 >= self.seçenekler.animasyon_süresi;
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
        self.kapalı = model_kapalı_adları(&self.seçenekler);
        self.gösterge_sayfası = 0;
        self.fırça_seçimi = None;
        self.fırça_alanları.clear();
        self.gezinmeyi_sıfırla();
        cx.notify();
        Ok(())
    }

    /// Alan varlığını koruyan ECharts uyumlu `setOption` yolu. Normal merge,
    /// `notMerge`, `replaceMerge`, `lazyUpdate` ve `silent` çözümü ortak
    /// başsız çalışma zamanı tarafından yapılır; gpui görünümü yalnız ortaya
    /// çıkan doğrulanmış option'ı animasyonlu olarak devralır.
    pub fn seçenek_yaması_uygula(
        &mut self,
        yama: impl Into<SeçenekYaması>,
        kip: SeçenekAyarlamaKipi,
        cx: &mut Context<Self>,
    ) -> Result<(), BilesenHatasi> {
        let (genişlik, yükseklik) = self.son_boyut.get();
        let başlatma = ÖrnekBaşlatmaSeçenekleri {
            genişlik: genişlik.max(1.0),
            yükseklik: yükseklik.max(1.0),
            yerel: self.seçenekler.yerel,
            ..ÖrnekBaşlatmaSeçenekleri::default()
        };
        let mut çalışma = GrafikÇalışmaZamanı::yeni(başlatma, (*self.seçenekler).clone())?;
        çalışma.seçenekleri_ayarla(yama, kip)?;
        let seçenekler = çalışma.seçenekleri_al()?;
        self.seçenekleri_değiştir(seçenekler, cx)
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
        let etkin_imleç = fare
            .and_then(|fare| {
                self.matris_hücreleri
                    .try_borrow()
                    .ok()
                    .and_then(|bölgeler| {
                        bölgeler
                            .iter()
                            .rev()
                            .find(|bölge| bölge.geometri.içeriyor_mu(fare))
                            .and_then(|bölge| bölge.imleç.as_deref().map(gpui_imleci))
                    })
            })
            .unwrap_or(CursorStyle::Arrow);
        let kapalı = self.kapalı.clone();
        let gösterge_sayfası = self.gösterge_sayfası;
        let mut fırça_alanları = self.fırça_alanları.clone();
        if let Some(alan) = self.fırça_seçimi.clone() {
            fırça_alanları.push(alan);
        }
        let gösterge_kutuları = self.gösterge_kutuları.clone();
        let isabetler = self.isabetler.clone();
        let matris_hücreleri = self.matris_hücreleri.clone();
        let tanılar = self.bekleyen_tanılar.clone();
        let sürgüler = self.sürgü_bölgeleri.clone();
        let iç_alanlar = self.iç_yakınlaştırma_alanları.clone();
        let ızgara_alanları = self.ızgara_alanları.clone();
        let eşleme_kutuları = self.eşleme_kutuları.clone();
        let sürekli_eşleme_alanı = self.sürekli_eşleme_alanı.clone();
        let gösterge_okları = self.gösterge_okları.clone();
        let araç_düğmeleri = self.araç_düğmeleri.clone();
        let film_düğmeleri = self.film_düğmeleri.clone();
        let kırıntı_kutuları = self.kırıntı_kutuları.clone();
        let grafik_sahnesi = self.grafik_sahnesi.clone();
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
                        let mut çizici = Çizici::yeni(pencere, uygulama, sınırlar, Some(önbellek));
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
                            ipucu_öğesi: None,
                            kapalı: kapalı.clone(),
                            gösterge_sayfası,
                            fırça: None,
                            fırça_alanları: fırça_alanları
                                .iter()
                                .map(|alan| alan.kaydır(-köken.0, -köken.1))
                                .collect(),
                            zaman_şeridi,
                            hiyerarşi_yolu: hiyerarşi_yolu.clone(),
                            grafo_görünümü,
                            grafo_kaymaları: grafo_kaymaları.clone(),
                        };
                        let mut çıktı = grafiği_boya(&mut çizici, &etkin_seçenekler, &girdi);
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
                        match matris_hücreleri.try_borrow_mut() {
                            Ok(mut bölgeler) => {
                                bölgeler.clear();
                                bölgeler.extend(
                                    çıktı
                                        .matris_hücreleri
                                        .iter()
                                        .map(|bölge| bölge.kaydır(köken.0, köken.1)),
                                );
                            }
                            Err(_) => tanı_bildir("matris_hücreleri"),
                        }
                        let kaydırılmış = |d: Dikdörtgen| {
                            Dikdörtgen::yeni(d.x + köken.0, d.y + köken.1, d.genişlik, d.yükseklik)
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
                                        dikey: s.dikey,
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
                                        dikey: a.dikey,
                                    });
                                }
                            }
                            Err(_) => tanı_bildir("iç_yakınlaştırma_alanları"),
                        }
                        match ızgara_alanları.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                kayıt.clear();
                                kayıt
                                    .extend(çıktı.ızgara_alanları.iter().copied().map(kaydırılmış));
                            }
                            Err(_) => tanı_bildir("ızgara_alanları"),
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
                        match sürekli_eşleme_alanı.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                *kayıt = çıktı
                                    .sürekli_eşleme
                                    .map(|bölge| bölge.kaydır(köken.0, köken.1));
                            }
                            Err(_) => tanı_bildir("sürekli_eşleme_alanı"),
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
                        match grafik_sahnesi.try_borrow_mut() {
                            Ok(mut kayıt) => {
                                *kayıt = çıktı.grafik_sahnesi.take().map(|sahne| (sahne, köken));
                            }
                            Err(_) => tanı_bildir("graphic_sahnesi"),
                        }
                    },
                )
                .size_full()
                .cursor(etkin_imleç),
            )
            .on_mouse_move(cx.listener(|bu, olay: &MouseMoveEvent, _, cx| {
                let yeni = (f32::from(olay.position.x), f32::from(olay.position.y));
                // Fırça seçimi sürüyor.
                if olay.pressed_button == Some(MouseButton::Left)
                    && let Some(alan) = bu.fırça_seçimi.as_mut()
                {
                    match alan {
                        FırçaAlanı::Dikdörtgen { bitiş, .. } => *bitiş = yeni,
                        FırçaAlanı::Çokgen { noktalar } => {
                            let ekle = noktalar.last().is_none_or(|son| {
                                (yeni.0 - son.0).powi(2) + (yeni.1 - son.1).powi(2) >= 4.0
                            });
                            if ekle {
                                noktalar.push(yeni);
                            }
                        }
                        FırçaAlanı::Yatay { bitiş, .. } => *bitiş = yeni.0,
                        FırçaAlanı::Dikey { bitiş, .. } => *bitiş = yeni.1,
                    }
                    cx.notify();
                    return;
                }
                // Etkin sürükleme: kaydırma ya da sürgü.
                if olay.pressed_button == Some(MouseButton::Left) {
                    match bu.sürükleme {
                        Some(Sürükleme::GrafoDüğüm { veri_sırası, son }) => {
                            let fark = (yeni.0 - son.0, yeni.1 - son.1);
                            let kayıt = bu.grafo_kaymaları.entry(veri_sırası).or_insert((0.0, 0.0));
                            kayıt.0 += fark.0;
                            kayıt.1 += fark.1;
                            bu.sürükleme = Some(Sürükleme::GrafoDüğüm {
                                veri_sırası,
                                son: yeni,
                            });
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
                            başlangıç_ekseni,
                            pencere,
                            alan_uzunluğu,
                            dikey,
                        }) => {
                            let yeni_ekseni = if dikey { -yeni.1 } else { yeni.0 };
                            let oran_farkı =
                                (yeni_ekseni - başlangıç_ekseni) / alan_uzunluğu.max(1.0);
                            let genişlik = pencere.1 - pencere.0;
                            // İçerik fareyle sürüklenir: pencere ters yönde kayar.
                            let kayma = -oran_farkı * genişlik * 100.0;
                            let b =
                                (pencere.0 * 100.0 + kayma).clamp(0.0, 100.0 - genişlik * 100.0);
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
                            başlangıç_ekseni,
                            pencere,
                            şerit_uzunluğu,
                            dikey,
                        }) => {
                            let yeni_ekseni = if dikey { -yeni.1 } else { yeni.0 };
                            let oran_farkı =
                                (yeni_ekseni - başlangıç_ekseni) / şerit_uzunluğu.max(1.0) * 100.0;
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
                                    let b = (b0 + oran_farkı).clamp(0.0, 100.0 - genişlik);
                                    (b, b + genişlik)
                                }
                            };
                            bu.pencereyi_güncelle(yakınlaştırma_sırası, b, e, cx);
                            return;
                        }
                        Some(Sürükleme::GörselEşleme {
                            parça,
                            başlangıç_ekseni,
                            bölge,
                        }) => {
                            let yeni_ekseni = bölge.sürükleme_ekseni(yeni);
                            let [alt, üst] =
                                bölge.sürüklenmiş_aralık(parça, yeni_ekseni - başlangıç_ekseni);
                            if let Some(eşleme) =
                                Arc::make_mut(&mut bu.seçenekler).görsel_eşleme.as_mut()
                            {
                                eşleme.seçili_aralık = Some([alt, üst]);
                                cx.emit(GrafikOlayı::GörselAralıkDeğişti {
                                    sıra: 0,
                                    alt,
                                    üst,
                                });
                                cx.notify();
                            }
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
                    Ok(alanlar) => alanlar.iter().find(|a| a.alan.içeriyor_mu(konum)).cloned(),
                    Err(_) => None,
                };
                let Some(kayıt) = alan_kaydı else {
                    // Grafo gezinmesi (roam): tekerlek görünümü ölçekler.
                    if bu
                        .seçenekler
                        .seriler
                        .iter()
                        .any(|s| matches!(s, Seri::Grafo(_)))
                    {
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
                        bu.grafo_görünümü =
                            (kayma_x * gerçek_çarpan, kayma_y * gerçek_çarpan, yeni_ölçek);
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
                let imleç_oranı = kayıt.odak_oranı(konum);
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
                    if let Some(alan) = bu.fırça_seçimi.take() {
                        if alan.geçerli_mi() {
                            let çoklu = bu
                                .seçenekler
                                .fırça
                                .as_ref()
                                .map(|fırça| fırça.çoklu)
                                .unwrap_or(false);
                            if !çoklu {
                                bu.fırça_alanları.clear();
                            }
                            bu.fırça_alanları.push(alan);
                            let mut öğeler: Vec<(usize, usize)> = match bu.isabetler.try_borrow() {
                                Ok(bölgeler) => bölgeler
                                    .iter()
                                    .filter(|bölge| {
                                        let merkez = bölge.geometri.merkez();
                                        bu.fırça_alanları
                                            .iter()
                                            .any(|alan| alan.içeriyor_mu(merkez))
                                    })
                                    .map(|bölge| (bölge.seri_sırası, bölge.veri_sırası))
                                    .collect(),
                                Err(_) => Vec::new(),
                            };
                            öğeler.sort_unstable();
                            öğeler.dedup();
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
                                BilesenHatasi::KilitliDurum {
                                    bileşen: "gösterge_kutuları",
                                },
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
                        let adlar = gösterge_adları(&bu.seçenekler);
                        let Some(gösterge) = Arc::make_mut(&mut bu.seçenekler).gösterge.as_mut()
                        else {
                            return;
                        };
                        if gösterge.seçim_kipi
                            == crate::model::bilesen::GöstergeSeçimKipi::Kapalı
                        {
                            return;
                        }
                        gösterge.seçimi_değiştir(&ad, &adlar);
                        bu.kapalı = adlar
                            .iter()
                            .filter_map(|ad| (!gösterge.seçili_mi(ad)).then_some(ad.clone()))
                            .collect();
                        let görünür = gösterge.seçili_mi(&ad);
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
                            bu.fırça_alanları.clear();
                            bu.gezinmeyi_sıfırla();
                            cx.emit(GrafikOlayı::GeriYüklendi);
                            cx.notify();
                            return;
                        }
                        Some(AraçTürü::VeriGörünümü) => {
                            cx.emit(GrafikOlayı::VeriGörünümüİstendi);
                            return;
                        }
                        Some(AraçTürü::VeriYakınlaştır) => {
                            let seçenekler = Arc::make_mut(&mut bu.seçenekler);
                            let etkin = seçenekler
                                .fırça
                                .as_ref()
                                .map(|fırça| fırça.etkin)
                                .unwrap_or(false);
                            seçenekler.fırça = (!etkin).then(crate::model::bilesen::Fırça::yeni);
                            bu.fırça_seçimi = None;
                            bu.fırça_alanları.clear();
                            cx.notify();
                            return;
                        }
                        Some(AraçTürü::VeriYakınlaştırmayıGeriAl) => {
                            let seçenekler = Arc::make_mut(&mut bu.seçenekler);
                            seçenekler.fırça = None;
                            seçenekler.veri_yakınlaştırmaları.clear();
                            bu.fırça_seçimi = None;
                            bu.fırça_alanları.clear();
                            cx.notify();
                            return;
                        }
                        Some(AraçTürü::Fırça(tür)) => {
                            use crate::model::bilesen::{FırçaAracıTürü, FırçaTürü};

                            let seçenekler = Arc::make_mut(&mut bu.seçenekler);
                            match tür {
                                FırçaAracıTürü::Temizle => {
                                    bu.fırça_seçimi = None;
                                    bu.fırça_alanları.clear();
                                    cx.emit(GrafikOlayı::FırçaSeçildi {
                                        öğeler: Vec::new()
                                    });
                                }
                                FırçaAracıTürü::Koru => {
                                    let mut fırça = seçenekler.fırça.clone().unwrap_or_default();
                                    fırça.çoklu = !fırça.çoklu;
                                    seçenekler.fırça = Some(fırça);
                                }
                                tür => {
                                    let fırça_türü = match tür {
                                        FırçaAracıTürü::Dikdörtgen => {
                                            FırçaTürü::Dikdörtgen
                                        }
                                        FırçaAracıTürü::Çokgen => FırçaTürü::Çokgen,
                                        FırçaAracıTürü::Yatay => FırçaTürü::Yatay,
                                        FırçaAracıTürü::Dikey => FırçaTürü::Dikey,
                                        FırçaAracıTürü::Koru | FırçaAracıTürü::Temizle => {
                                            return;
                                        }
                                    };
                                    let etkin = seçenekler.fırça.as_ref().is_some_and(|fırça| {
                                        fırça.etkin && fırça.tür == fırça_türü
                                    });
                                    let mut fırça = seçenekler.fırça.clone().unwrap_or_default();
                                    fırça.etkin = !etkin;
                                    fırça.tür = fırça_türü;
                                    seçenekler.fırça = Some(fırça);
                                    bu.fırça_seçimi = None;
                                }
                            }
                            cx.notify();
                            return;
                        }
                        Some(AraçTürü::SihirliÇizgi) => {
                            cx.emit(GrafikOlayı::SihirliTürİstendi {
                                tür: crate::cizim::olay::SihirliSeriTürü::Çizgi,
                            });
                            return;
                        }
                        Some(AraçTürü::SihirliSütun) => {
                            cx.emit(GrafikOlayı::SihirliTürİstendi {
                                tür: crate::cizim::olay::SihirliSeriTürü::Sütun,
                            });
                            return;
                        }
                        Some(AraçTürü::SihirliYığın) => {
                            cx.emit(GrafikOlayı::SihirliTürİstendi {
                                tür: crate::cizim::olay::SihirliSeriTürü::Yığın,
                            });
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
                    // 0e) Serbest graphic öğeleri: sahnenin affine isabet
                    // sınaması, çizimde kullanılan dönüşümün aynısını izler.
                    let grafik_vuruşu = match bu.grafik_sahnesi.try_borrow() {
                        Ok(kayıt) => kayıt.as_ref().and_then(|(hazır, köken)| {
                            let yerel = (konum.0 - köken.0, konum.1 - köken.1);
                            hazır
                                .sahne
                                .isabet(yerel)
                                .and_then(|isabet| hazır.öğe_bilgileri.get(&isabet.kimlik).cloned())
                        }),
                        Err(_) => None,
                    };
                    if let Some(bilgi) = grafik_vuruşu {
                        cx.emit(GrafikOlayı::GrafikÖğesiTıklandı {
                            kimlik: bilgi.kimlik,
                            ad: bilgi.ad,
                        });
                        return;
                    }
                    // 0c) Fırça etkinse seçim başlat.
                    if let Some(fırça) = bu.seçenekler.fırça.as_ref().filter(|fırça| fırça.etkin)
                    {
                        use crate::model::bilesen::FırçaTürü;

                        let ızgara = || {
                            bu.ızgara_alanları.try_borrow().ok().and_then(|alanlar| {
                                alanlar.iter().find(|alan| alan.içeriyor_mu(konum)).copied()
                            })
                        };
                        bu.fırça_seçimi = match fırça.tür {
                            FırçaTürü::Dikdörtgen => Some(FırçaAlanı::Dikdörtgen {
                                başlangıç: konum,
                                bitiş: konum,
                            }),
                            FırçaTürü::Çokgen => Some(FırçaAlanı::Çokgen {
                                noktalar: vec![konum],
                            }),
                            FırçaTürü::Yatay => ızgara().map(|alan| FırçaAlanı::Yatay {
                                başlangıç: konum.0,
                                bitiş: konum.0,
                                üst: alan.y,
                                alt: alan.alt(),
                            }),
                            FırçaTürü::Dikey => ızgara().map(|alan| FırçaAlanı::Dikey {
                                başlangıç: konum.1,
                                bitiş: konum.1,
                                sol: alan.x,
                                sağ: alan.sağ(),
                            }),
                        };
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
                    if let Some(parça_sırası) = eşleme_vuruşu {
                        let seçenekler = Arc::make_mut(&mut bu.seçenekler);
                        if let Some(eşleme) = seçenekler.görsel_eşleme.as_mut() {
                            if let Some(k) = eşleme
                                .kapalı_parçalar
                                .iter()
                                .position(|k| *k == parça_sırası)
                            {
                                eşleme.kapalı_parçalar.remove(k);
                            } else {
                                eşleme.kapalı_parçalar.push(parça_sırası);
                            }
                            let seçili = (0..eşleme.parça_sayısı())
                                .map(|sıra| (sıra, eşleme.parça_açık_mı(sıra)))
                                .collect::<BTreeMap<_, _>>();
                            cx.emit(GrafikOlayı::GörselParçalarDeğişti { sıra: 0, seçili });
                            cx.notify();
                        }
                        return;
                    }
                    // 1b) Sürekli görsel eşleme tutamacı/seçili şerit.
                    let sürekli_vuruş = match bu.sürekli_eşleme_alanı.try_borrow() {
                        Ok(alan) => alan
                            .as_ref()
                            .and_then(|bölge| bölge.parça_bul(konum).map(|parça| (*bölge, parça))),
                        Err(_) => None,
                    };
                    if let Some((bölge, parça)) = sürekli_vuruş {
                        bu.sürükleme = Some(Sürükleme::GörselEşleme {
                            parça,
                            başlangıç_ekseni: bölge.sürükleme_ekseni(konum),
                            bölge,
                        });
                        return;
                    }
                    // 1c) Veri yakınlaştırma sürgüsü parçası.
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
                            parça.map(|p| (s.yakınlaştırma_sırası, p, s.eksen_uzunluğu(), s.dikey))
                        }),
                        Err(_) => None,
                    };
                    if let Some((sıra, parça, şerit_uzunluğu, dikey)) = sürgü_vuruşu {
                        if let Some(y) = bu.seçenekler.veri_yakınlaştırmaları.get(sıra) {
                            bu.sürükleme = Some(Sürükleme::Sürgü {
                                yakınlaştırma_sırası: sıra,
                                parça,
                                başlangıç_ekseni: if dikey { -konum.1 } else { konum.0 },
                                pencere: y.oranlar(),
                                şerit_uzunluğu,
                                dikey,
                            });
                        }
                        return;
                    }
                    // 1d) İç yakınlaştırma alanında kaydırma başlat.
                    let iç_vuruş = match bu.iç_yakınlaştırma_alanları.try_borrow() {
                        Ok(alanlar) => alanlar.iter().find(|a| a.alan.içeriyor_mu(konum)).cloned(),
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
                            başlangıç_ekseni: kayıt.eksen_konumu(konum),
                            pencere: y.oranlar(),
                            alan_uzunluğu: kayıt.eksen_uzunluğu(),
                            dikey: kayıt.dikey,
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
                                BilesenHatasi::KilitliDurum {
                                    bileşen: "isabet_bölgeleri",
                                },
                            ));
                            return;
                        }
                    };
                    let matris_bölgesi = if bölge.is_none() {
                        bu.matris_hücreleri.try_borrow().ok().and_then(|bölgeler| {
                            bölgeler
                                .iter()
                                .rev()
                                .find(|bölge| {
                                    bölge.olay_tetikle && bölge.geometri.içeriyor_mu(nokta)
                                })
                                .cloned()
                        })
                    } else {
                        None
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
                    } else if let Some(b) = matris_bölgesi {
                        cx.emit(GrafikOlayı::MatrisHücresiTıklandı {
                            bileşen_sırası: b.bileşen_sırası,
                            hedef_türü: b.hedef_türü,
                            ad: b.ad,
                            değer: b.değer,
                            koordinat: b.koordinat,
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
