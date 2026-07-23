//! Ağaç haritası (treemap) — `echarts/src/chart/treemap` karşılığı.
//!
//! Yerleşim ECharts'ın D3 kökenli squarify akışını; seviye kalıtımı,
//! `visibleMin`, `childrenVisibleMin`, `leafDepth`, görsel boyut eşleme,
//! üst etiket ve breadcrumb davranışlarını aynı model üzerinde uygular.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::grafik::pasta::zengin_etiketi_hizalı_yaz;
use crate::koordinat::Dikdörtgen;
use crate::model::Uzunluk;
use crate::model::agac::{
    AğaçDüğümü, AğaçHaritasıDurumu, AğaçHaritasıGörselBoyutu, AğaçHaritasıGörseli,
    AğaçHaritasıKırpmaPenceresi, AğaçHaritasıRenkEşlemesi, AğaçHaritasıSeviyesi,
    AğaçHaritasıSırası, AğaçHaritasıÖğeStili,
};
use crate::model::seri::AğaçHaritasıSerisi;
use crate::model::stil::{
    Etiket, EtiketKonumu, YazıDikeyHizası, YazıYatayHizası, zengin_metin_içeriği,
};
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

/// Yerleşimi ve kalıtılmış görsel durumu hesaplanmış hücre.
#[derive(Clone, Debug)]
pub struct AğaçHücresi {
    pub ad: String,
    pub değer: f64,
    pub değerler: Vec<Option<f64>>,
    pub alan: Dikdörtgen,
    pub renk: Renk,
    pub derinlik: usize,
    pub yaprak: bool,
    /// Ham çocukları `leafDepth` nedeniyle gizlenmiş yakınlaştırılabilir
    /// yaprak kökü (`isLeafRoot`).
    pub inilebilir_yaprak: bool,
    pub veri_sırası: usize,
    pub yol: Vec<String>,
    pub öğe_stili: AğaçHaritasıÖğeStili,
    pub etiket: Etiket,
    pub üst_etiket: Etiket,
    pub vurgu: AğaçHaritasıDurumu,
    pub bağlantı: Option<String>,
    pub hedef: Option<String>,
}

/// Rasterdan bağımsız Treemap geometri kanıtı.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AğaçHaritasıSahneÖzeti {
    pub hücre_sayısı: usize,
    pub yaprak_sayısı: usize,
    pub üst_etiket_sayısı: usize,
    pub etiket_sayısı: usize,
    pub koordinat_sayısı: usize,
    pub fnv1a_64: u64,
}

/// Treemap `rootRect` görünümünü bir yerleşim dikdörtgenine uygular.
///
/// ECharts gezinme sırasında squarify sonucunu yeniden sıralamaz; kök
/// dikdörtgenini kaydırıp ölçekler ve bütün alt dikdörtgenleri aynı affine
/// dönüşümle günceller. Bu yardımcı hem renderer hem etkileşim testleri için
/// tek doğruluk noktasıdır.
pub fn ağaç_haritası_görünümünü_uygula(
    dikdörtgen: Dikdörtgen,
    seri_alanı: Dikdörtgen,
    görünüm: (f32, f32, f32),
    ölçek_sınırı: (f32, f32),
) -> Dikdörtgen {
    let en_küçük = ölçek_sınırı.0.max(0.01);
    let en_büyük = ölçek_sınırı.1.max(en_küçük);
    let ölçek = if görünüm.2.is_finite() {
        görünüm.2.clamp(en_küçük, en_büyük)
    } else {
        1.0_f32.clamp(en_küçük, en_büyük)
    };
    let kayma_x = if görünüm.0.is_finite() {
        görünüm.0
    } else {
        0.0
    };
    let kayma_y = if görünüm.1.is_finite() {
        görünüm.1
    } else {
        0.0
    };
    let merkez = seri_alanı.merkez();
    Dikdörtgen::yeni(
        merkez.0 + (dikdörtgen.x - merkez.0) * ölçek + kayma_x,
        merkez.1 + (dikdörtgen.y - merkez.1) * ölçek + kayma_y,
        dikdörtgen.genişlik * ölçek,
        dikdörtgen.yükseklik * ölçek,
    )
}

fn dikdörtgen_kesişimi(a: Dikdörtgen, b: Dikdörtgen) -> Option<Dikdörtgen> {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let sağ = a.sağ().min(b.sağ());
    let alt = a.alt().min(b.alt());
    (sağ > x && alt > y).then(|| Dikdörtgen::yeni(x, y, sağ - x, alt - y))
}

#[derive(Clone)]
struct ÇözülmüşKatman {
    görsel: AğaçHaritasıGörseli,
    öğe_stili: AğaçHaritasıÖğeStili,
    etiket: Etiket,
    üst_etiket: Etiket,
    vurgu: AğaçHaritasıDurumu,
}

#[derive(Clone, Copy)]
struct AlanlıDüğüm<'a> {
    düğüm: &'a AğaçDüğümü,
    veri_sırası: usize,
    değer: f64,
    alan: f64,
}

#[derive(Default)]
struct Satır<'a> {
    düğümler: Vec<AlanlıDüğüm<'a>>,
    alan: f64,
}

/// ECharts/zrender yerleşimi JavaScript `number` (IEEE-754 double) ile
/// hesaplanır. Derin Treemap'lerde her seviyede `f32`ye inmek görünürMin
/// sınırında düğüm kaybettirebildiği için bütün squarify ara geometrisi f64
/// kalır; yalnız çizim modeline yazılırken `Dikdörtgen`e çevrilir.
#[derive(Clone, Copy, Debug)]
struct KesinDikdörtgen {
    x: f64,
    y: f64,
    genişlik: f64,
    yükseklik: f64,
}

impl From<Dikdörtgen> for KesinDikdörtgen {
    fn from(değer: Dikdörtgen) -> Self {
        Self {
            x: f64::from(değer.x),
            y: f64::from(değer.y),
            genişlik: f64::from(değer.genişlik),
            yükseklik: f64::from(değer.yükseklik),
        }
    }
}

impl KesinDikdörtgen {
    fn sağ(self) -> f64 {
        self.x + self.genişlik
    }

    fn alt(self) -> f64 {
        self.y + self.yükseklik
    }

    fn çizim(self) -> Dikdörtgen {
        Dikdörtgen::yeni(
            self.x as f32,
            self.y as f32,
            self.genişlik as f32,
            self.yükseklik as f32,
        )
    }
}

fn görsel_yama_uygula(
    taban: &AğaçHaritasıGörseli,
    yama: &AğaçHaritasıGörseli,
) -> AğaçHaritasıGörseli {
    let mut sonuç = taban.clone();
    if yama.boyut.is_some() {
        sonuç.boyut.clone_from(&yama.boyut);
    }
    if yama.en_az.is_some() {
        sonuç.en_az = yama.en_az;
    }
    if yama.en_çok.is_some() {
        sonuç.en_çok = yama.en_çok;
    }
    if yama.renkler.is_some() {
        sonuç.renkler.clone_from(&yama.renkler);
    }
    if yama.alfa_aralığı.is_some() {
        sonuç.alfa_aralığı = yama.alfa_aralığı;
    }
    if yama.doygunluk_aralığı.is_some() {
        sonuç.doygunluk_aralığı = yama.doygunluk_aralığı;
    }
    if yama.eşleme.is_some() {
        sonuç.eşleme = yama.eşleme;
    }
    if yama.görünür_en_az.is_some() {
        sonuç.görünür_en_az = yama.görünür_en_az;
    }
    if yama.çocuk_görünür_en_az.is_some() {
        sonuç.çocuk_görünür_en_az = yama.çocuk_görünür_en_az;
    }
    sonuç
}

fn öğe_stili_yama_uygula(
    taban: &AğaçHaritasıÖğeStili,
    yama: &AğaçHaritasıÖğeStili,
) -> AğaçHaritasıÖğeStili {
    let mut sonuç = taban.clone();
    if yama.taban.renk.is_some() {
        sonuç.taban.renk.clone_from(&yama.taban.renk);
    }
    if yama.taban.kenarlık_rengi.is_some() {
        sonuç.taban.kenarlık_rengi = yama.taban.kenarlık_rengi;
    }
    if yama.kenarlık_kalınlığı_belirtildi {
        sonuç.taban.kenarlık_kalınlığı = yama.taban.kenarlık_kalınlığı;
        sonuç.kenarlık_kalınlığı_belirtildi = true;
    }
    if yama.kenarlık_yarıçapı_belirtildi {
        sonuç.taban.kenarlık_yarıçapı = yama.taban.kenarlık_yarıçapı;
        sonuç.kenarlık_yarıçapı_belirtildi = true;
    }
    if yama.taban.opaklık.is_some() {
        sonuç.taban.opaklık = yama.taban.opaklık;
    }
    if yama.renk_alfası.is_some() {
        sonuç.renk_alfası = yama.renk_alfası;
    }
    if yama.renk_doygunluğu.is_some() {
        sonuç.renk_doygunluğu = yama.renk_doygunluğu;
    }
    if yama.taban.gölge_bulanıklığı > 0.0 {
        sonuç.taban.gölge_bulanıklığı = yama.taban.gölge_bulanıklığı;
        sonuç.taban.gölge_rengi = yama.taban.gölge_rengi;
        sonuç.taban.gölge_kayması = yama.taban.gölge_kayması;
    }
    if yama.boşluk_genişliği_belirtildi {
        sonuç.boşluk_genişliği = yama.boşluk_genişliği;
        sonuç.boşluk_genişliği_belirtildi = true;
    }
    if yama.kenarlık_rengi_doygunluğu.is_some() {
        sonuç.kenarlık_rengi_doygunluğu = yama.kenarlık_rengi_doygunluğu;
    }
    sonuç
}

fn durum_yama_uygula(
    taban: &AğaçHaritasıDurumu,
    yama: &AğaçHaritasıDurumu,
) -> AğaçHaritasıDurumu {
    let mut sonuç = taban.clone();
    if let Some(stil) = &yama.öğe_stili {
        sonuç.öğe_stili = Some(match &sonuç.öğe_stili {
            Some(taban) => öğe_stili_yama_uygula(taban, stil),
            None => stil.clone(),
        });
    }
    if yama.etiket.is_some() {
        sonuç.etiket.clone_from(&yama.etiket);
    }
    if yama.üst_etiket.is_some() {
        sonuç.üst_etiket.clone_from(&yama.üst_etiket);
    }
    if yama.odak != Default::default() {
        sonuç.odak = yama.odak;
    }
    sonuç
}

fn seviye_uygula(
    taban: &ÇözülmüşKatman, seviye: &AğaçHaritasıSeviyesi
) -> ÇözülmüşKatman {
    let mut sonuç = taban.clone();
    if let Some(görsel) = &seviye.görsel {
        sonuç.görsel = görsel_yama_uygula(&sonuç.görsel, görsel);
    }
    if let Some(stil) = &seviye.öğe_stili {
        sonuç.öğe_stili = öğe_stili_yama_uygula(&sonuç.öğe_stili, stil);
    }
    if let Some(etiket) = &seviye.etiket {
        sonuç.etiket = etiket.uygula(&sonuç.etiket);
    }
    if let Some(etiket) = &seviye.üst_etiket {
        sonuç.üst_etiket = etiket.uygula(&sonuç.üst_etiket);
    }
    sonuç.vurgu = durum_yama_uygula(&sonuç.vurgu, &seviye.vurgu);
    sonuç
}

fn düğüm_uygula(
    taban: &ÇözülmüşKatman, düğüm: &AğaçDüğümü
) -> ÇözülmüşKatman {
    let mut sonuç = taban.clone();
    if let Some(görsel) = &düğüm.ağaç_haritası_görseli {
        sonuç.görsel = görsel_yama_uygula(&sonuç.görsel, görsel);
    }
    if let Some(stil) = &düğüm.ağaç_haritası_öğe_stili {
        sonuç.öğe_stili = öğe_stili_yama_uygula(&sonuç.öğe_stili, stil);
    }
    if let Some(stil) = &düğüm.öğe_stili {
        sonuç.öğe_stili = öğe_stili_yama_uygula(
            &sonuç.öğe_stili,
            &AğaçHaritasıÖğeStili::yeni().taban(stil.clone()),
        );
    }
    if let Some(etiket) = &düğüm.etiket {
        sonuç.etiket = etiket.uygula(&sonuç.etiket);
    }
    if let Some(etiket) = &düğüm.ağaç_haritası_üst_etiketi {
        sonuç.üst_etiket = etiket.uygula(&sonuç.üst_etiket);
    }
    if let Some(vurgu) = &düğüm.ağaç_haritası_vurgusu {
        sonuç.vurgu = durum_yama_uygula(&sonuç.vurgu, vurgu);
    }
    sonuç
}

fn seri_katmanı(seri: &AğaçHaritasıSerisi) -> ÇözülmüşKatman {
    ÇözülmüşKatman {
        görsel: seri.görsel.clone(),
        öğe_stili: seri.öğe_stili.clone(),
        etiket: seri.etiket.clone(),
        üst_etiket: seri.üst_etiket.clone(),
        vurgu: seri.vurgu.clone(),
    }
}

/// `treemapVisual.buildVisuals/calculateColor`: doğrudan itemStyle rengi,
/// üstten gelen designated renkten önce gelir; ardından tarihsel
/// colorSaturation (zrender HSL açıklık kanalı) ve colorAlpha uygulanır.
fn öğe_görsel_rengi(stil: &AğaçHaritasıÖğeStili, designated: Renk) -> Renk {
    let mut renk = stil
        .taban
        .renk
        .as_ref()
        .map(Dolgu::temsilî)
        .unwrap_or(designated);
    if let Some(doygunluk) = stil.renk_doygunluğu
        && doygunluk != 0.0
    {
        renk = renk.açıklık_ile(doygunluk);
    }
    if let Some(alfa) = stil.renk_alfası
        && alfa != 0.0
    {
        renk = renk.alfa_ile(alfa);
    }
    renk
}

fn derinlik_katmanı(
    seri: &AğaçHaritasıSerisi,
    taban: &ÇözülmüşKatman,
    ağaç_derinliği: usize,
) -> ÇözülmüşKatman {
    seri.seviyeler
        .get(ağaç_derinliği)
        .map_or_else(|| taban.clone(), |seviye| seviye_uygula(taban, seviye))
}

/// Serinin box-layout alanını çözer.
pub fn ağaç_haritası_alanı(seri: &AğaçHaritasıSerisi, tuval: Dikdörtgen) -> Dikdörtgen {
    let sol = seri.sol.çöz(tuval.genişlik);
    let üst = seri.üst.çöz(tuval.yükseklik);
    let genişlik = seri.sağ.map_or_else(
        || seri.genişlik.çöz(tuval.genişlik),
        |sağ| tuval.genişlik - sol - sağ.çöz(tuval.genişlik),
    );
    let yükseklik = seri.alt.map_or_else(
        || seri.yükseklik.çöz(tuval.yükseklik),
        |alt| tuval.yükseklik - üst - alt.çöz(tuval.yükseklik),
    );
    Dikdörtgen::yeni(
        tuval.x + sol,
        tuval.y + üst,
        genişlik.max(0.0),
        yükseklik.max(0.0),
    )
}

fn ön_sıra_indeksleri(düğümler: &[AğaçDüğümü]) -> HashMap<usize, usize> {
    fn gez(
        düğümler: &[AğaçDüğümü], sıra: &mut usize, çıktı: &mut HashMap<usize, usize>
    ) {
        for düğüm in düğümler {
            çıktı.insert(std::ptr::from_ref(düğüm).addr(), *sıra);
            *sıra = sıra.saturating_add(1);
            gez(&düğüm.çocuklar, sıra, çıktı);
        }
    }
    let mut çıktı = HashMap::new();
    let mut sıra = 0;
    gez(düğümler, &mut sıra, &mut çıktı);
    çıktı
}

#[derive(Default)]
struct KimlikSıraları {
    sıralar: HashMap<String, usize>,
    sonraki: usize,
}

impl KimlikSıraları {
    fn sıra(&mut self, düğüm: &AğaçDüğümü) -> usize {
        let kimlik = düğüm.kimlik.as_deref().unwrap_or(&düğüm.ad);
        if let Some(sıra) = self.sıralar.get(kimlik) {
            *sıra
        } else {
            let sıra = self.sonraki;
            self.sıralar.insert(kimlik.to_owned(), sıra);
            self.sonraki = self.sonraki.saturating_add(1);
            sıra
        }
    }
}

fn en_kötü_oran(satır: &Satır<'_>, sabit: f64, kare_oranı: f64) -> f64 {
    let (mut en_az, mut en_çok) = (f64::INFINITY, 0.0_f64);
    for düğüm in &satır.düğümler {
        if düğüm.alan > 0.0 {
            en_az = en_az.min(düğüm.alan);
            en_çok = en_çok.max(düğüm.alan);
        }
    }
    let kare_alan = satır.alan * satır.alan;
    let f = sabit * sabit * kare_oranı;
    if kare_alan <= 0.0 || !en_az.is_finite() {
        f64::INFINITY
    } else {
        ((f * en_çok) / kare_alan).max(kare_alan / (f * en_az))
    }
}

fn satırı_yerleştir<'a>(
    satır: &Satır<'a>,
    sabit: f64,
    kalan: &mut KesinDikdörtgen,
    yarım_boşluk: f64,
    son: bool,
) -> Vec<(AlanlıDüğüm<'a>, KesinDikdörtgen)> {
    // ECharts `position`: `rowFixedLength === rect.width`.
    let yatay = sabit == kalan.genişlik;
    let diğer_sınır = if yatay {
        kalan.yükseklik
    } else {
        kalan.genişlik
    };
    let mut diğer = if sabit > 0.0 {
        satır.alan / sabit
    } else {
        0.0
    };
    if son || diğer > diğer_sınır {
        diğer = diğer_sınır;
    }
    let mut son_konum = if yatay { kalan.x } else { kalan.y };
    let mut çıktı = Vec::with_capacity(satır.düğümler.len());
    for (sıra, düğüm) in satır.düğümler.iter().copied().enumerate() {
        let adım = if diğer > 0.0 {
            düğüm.alan / diğer
        } else {
            0.0
        };
        let kalan_uzunluk = if yatay {
            kalan.sağ() - son_konum
        } else {
            kalan.alt() - son_konum
        };
        let uzunluk = if sıra + 1 == satır.düğümler.len() || kalan_uzunluk < adım {
            kalan_uzunluk
        } else {
            adım
        };
        let (x, y, genişlik, yükseklik) = if yatay {
            (
                son_konum + yarım_boşluk.min((uzunluk - 2.0 * yarım_boşluk).max(0.0) / 2.0),
                kalan.y + yarım_boşluk.min((diğer - 2.0 * yarım_boşluk).max(0.0) / 2.0),
                (uzunluk - 2.0 * yarım_boşluk).max(0.0),
                (diğer - 2.0 * yarım_boşluk).max(0.0),
            )
        } else {
            (
                kalan.x + yarım_boşluk.min((diğer - 2.0 * yarım_boşluk).max(0.0) / 2.0),
                son_konum + yarım_boşluk.min((uzunluk - 2.0 * yarım_boşluk).max(0.0) / 2.0),
                (diğer - 2.0 * yarım_boşluk).max(0.0),
                (uzunluk - 2.0 * yarım_boşluk).max(0.0),
            )
        };
        çıktı.push((
            düğüm,
            KesinDikdörtgen {
                x,
                y,
                genişlik,
                yükseklik,
            },
        ));
        son_konum += uzunluk;
    }
    if yatay {
        kalan.y += diğer;
        kalan.yükseklik = (kalan.yükseklik - diğer).max(0.0);
    } else {
        kalan.x += diğer;
        kalan.genişlik = (kalan.genişlik - diğer).max(0.0);
    }
    çıktı
}

fn kareselleştir<'a>(
    düğümler: Vec<AlanlıDüğüm<'a>>,
    alan: KesinDikdörtgen,
    boşluk: f64,
    kare_oranı: f64,
) -> Vec<(AlanlıDüğüm<'a>, KesinDikdörtgen)> {
    let mut kalan = alan;
    let mut çıktı = Vec::with_capacity(düğümler.len());
    let mut satır = Satır::default();
    let mut en_iyi = f64::INFINITY;
    let mut sıra = 0usize;
    while sıra < düğümler.len() {
        let düğüm = düğümler[sıra];
        satır.düğümler.push(düğüm);
        satır.alan += düğüm.alan;
        let sabit = kalan.genişlik.min(kalan.yükseklik);
        let puan = en_kötü_oran(&satır, sabit, kare_oranı);
        if puan <= en_iyi {
            sıra += 1;
            en_iyi = puan;
        } else {
            satır.alan -= satır.düğümler.pop().map_or(0.0, |öğe| öğe.alan);
            çıktı.extend(satırı_yerleştir(
                &satır,
                sabit,
                &mut kalan,
                boşluk / 2.0,
                false,
            ));
            satır = Satır::default();
            en_iyi = f64::INFINITY;
        }
    }
    if !satır.düğümler.is_empty() {
        let sabit = kalan.genişlik.min(kalan.yükseklik);
        çıktı.extend(satırı_yerleştir(
            &satır,
            sabit,
            &mut kalan,
            boşluk / 2.0,
            true,
        ));
    }
    çıktı
}

fn renk_ara_değeri(renkler: &[Renk], t: f32) -> Option<Renk> {
    if renkler.is_empty() {
        return None;
    }
    if renkler.len() == 1 {
        return renkler.first().copied();
    }
    let konum = t.clamp(0.0, 1.0) * (renkler.len() - 1) as f32;
    let alt = konum.floor() as usize;
    let üst = (alt + 1).min(renkler.len() - 1);
    Some(renkler[alt].karıştır(renkler[üst], konum - alt as f32))
}

fn doğrusal_oran(değer: f64, en_az: f64, en_çok: f64) -> f32 {
    if !değer.is_finite() || (en_çok - en_az).abs() <= f64::EPSILON {
        0.5
    } else {
        ((değer - en_az) / (en_çok - en_az)).clamp(0.0, 1.0) as f32
    }
}

#[allow(clippy::too_many_arguments)]
fn çocuk_renkleri(
    düğümler: &[AlanlıDüğüm<'_>],
    ham_kardeşler: &[AğaçDüğümü],
    görsel: &AğaçHaritasıGörseli,
    üst_renk: Option<Renk>,
    palet: &dyn Fn(usize) -> Renk,
    kimlikler: &mut KimlikSıraları,
) -> Vec<Renk> {
    let boyut = görsel
        .boyut
        .as_ref()
        .cloned()
        .unwrap_or(AğaçHaritasıGörselBoyutu::Sıra(0));
    let değerler = düğümler
        .iter()
        .map(|öğe| öğe.düğüm.görsel_değer(&boyut).unwrap_or(öğe.değer))
        .collect::<Vec<_>>();
    // ECharts `statistic`, visual extent'i `visibleMin` elemesinden önceki
    // ham kardeşlerden çıkarır. Aksi halde yakınlaştırma/filtreleme renkleri
    // değiştirir ve özellikle disk örneğindeki en küçük değerler yanlış
    // açıklık aralığına taşınır.
    let kapsam_değerleri = ham_kardeşler
        .iter()
        .filter(|düğüm| düğüm.etkin_değer().is_finite() && düğüm.etkin_değer() >= 0.0)
        .filter_map(|düğüm| düğüm.görsel_değer(&boyut))
        .collect::<Vec<_>>();
    let mut en_az = kapsam_değerleri
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let mut en_çok = kapsam_değerleri
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    if !en_az.is_finite() || !en_çok.is_finite() {
        en_az = değerler.iter().copied().fold(f64::INFINITY, f64::min);
        en_çok = değerler.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    }
    if let Some(sınır) = görsel.en_az
        && sınır < en_az
    {
        en_az = sınır;
    }
    if let Some(sınır) = görsel.en_çok
        && sınır > en_çok
    {
        en_çok = sınır;
    }
    let eşleme = görsel.eşleme.unwrap_or(AğaçHaritasıRenkEşlemesi::Sıra);
    let renk_aralığı = görsel
        .renkler
        .as_deref()
        .filter(|renkler| !renkler.is_empty());
    // ECharts `setDefault`, global paleti yalnız level[0]'a renk aralığı
    // olarak ekler. Modelde bu örtük kaldığı için üst rengi olmayan sanal
    // kökte aynı eşleme palet üzerinden yapılır; alt seviyelerde aralık
    // yoksa designated parent rengi değişmeden miras alınır.
    let örtük_kök_paleti = görsel.renkler.is_none() && üst_renk.is_none();
    let renk_eşlemesi_etkin = renk_aralığı.is_some() || örtük_kök_paleti;
    düğümler
        .iter()
        .zip(değerler)
        .enumerate()
        .map(|(sıra, (öğe, değer))| {
            let kategori = match eşleme {
                AğaçHaritasıRenkEşlemesi::Sıra => sıra,
                AğaçHaritasıRenkEşlemesi::Kimlik if renk_eşlemesi_etkin => {
                    kimlikler.sıra(öğe.düğüm)
                }
                AğaçHaritasıRenkEşlemesi::Kimlik => sıra,
                AğaçHaritasıRenkEşlemesi::Değer => sıra,
            };
            let t = doğrusal_oran(değer, en_az, en_çok);
            let mut renk = renk_aralığı
                .and_then(|renkler| match eşleme {
                    AğaçHaritasıRenkEşlemesi::Değer => renk_ara_değeri(renkler, t),
                    _ => renkler.get(kategori % renkler.len()).copied(),
                })
                .or_else(|| örtük_kök_paleti.then(|| palet(kategori)))
                .or(üst_renk)
                .unwrap_or_else(|| palet(kategori));
            // `getRangeVisual`: color > colorAlpha > colorSaturation.
            if !renk_eşlemesi_etkin && let Some((baş, son)) = görsel.alfa_aralığı {
                renk = renk.alfa_ile(baş + (son - baş) * t);
            } else if !renk_eşlemesi_etkin && let Some((baş, son)) = görsel.doygunluk_aralığı
            {
                // ECharts treemap'in tarihsel `colorSaturation` adı,
                // `modifyHSL(color, null, null, value)` ile gerçekte HSL
                // açıklık kanalını değiştirir; uyumluluk için bu davranış
                // kasıtlı olarak korunur.
                renk = renk.açıklık_ile(baş + (son - baş) * t);
            }
            öğe.düğüm.renk.unwrap_or(renk)
        })
        .collect()
}

fn iç_alan(
    alan: KesinDikdörtgen, katman: &ÇözülmüşKatman, üst_etiket: bool
) -> KesinDikdörtgen {
    let kenar = f64::from(katman.öğe_stili.taban.kenarlık_kalınlığı.max(0.0));
    let yarım_boşluk = f64::from(katman.öğe_stili.boşluk_genişliği) / 2.0;
    let üst_etiket_yüksekliği = if üst_etiket && katman.üst_etiket.göster {
        f64::from(katman.üst_etiket.yazı.yükseklik.unwrap_or(20.0).max(0.0))
    } else {
        0.0
    };
    let yatay_kayma = kenar - yarım_boşluk;
    let üst_kayma = kenar.max(üst_etiket_yüksekliği) - yarım_boşluk;
    KesinDikdörtgen {
        x: alan.x + yatay_kayma,
        y: alan.y + üst_kayma,
        genişlik: (alan.genişlik - 2.0 * yatay_kayma).max(0.0),
        yükseklik: (alan.yükseklik - yatay_kayma - üst_kayma).max(0.0),
    }
}

fn etkin_kökler<'a>(
    kökler: &'a [AğaçDüğümü],
    yol: &[String],
) -> (&'a [AğaçDüğümü], usize, Option<&'a AğaçDüğümü>) {
    let mut etkin = kökler;
    let mut derinlik = 0usize;
    let mut kök = None;
    for ad in yol {
        let Some(düğüm) = etkin.iter().find(|düğüm| &düğüm.ad == ad) else {
            break;
        };
        if düğüm.çocuklar.is_empty() {
            break;
        }
        kök = Some(düğüm);
        etkin = &düğüm.çocuklar;
        derinlik = derinlik.saturating_add(1);
    }
    (etkin, derinlik, kök)
}

/// `initChildren` içindeki `visibleMin` süzmesini yerleşim üretmeden uygular.
/// ECharts `isLeafRoot` bayrağını ham çocuk varlığına göre değil, bu süzmeden
/// sonra en az bir `viewChild` kaldığında koyar.
fn görünür_çocuk_var_mı(
    seri: &AğaçHaritasıSerisi,
    çocuklar: &[AğaçDüğümü],
    toplam_alan: f64,
    katman: &ÇözülmüşKatman,
) -> bool {
    let mut değerler = çocuklar
        .iter()
        .map(AğaçDüğümü::etkin_değer)
        .filter(|değer| değer.is_finite() && *değer > 0.0)
        .collect::<Vec<_>>();
    if değerler.is_empty() {
        return false;
    }
    if seri.sıralama == AğaçHaritasıSırası::Veri {
        return true;
    }
    let Some(eşik) = katman.görsel.görünür_en_az else {
        return true;
    };
    değerler.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let mut toplam = değerler.iter().sum::<f64>();
    for değer in değerler {
        if değer / toplam * toplam_alan >= f64::from(eşik) {
            return true;
        }
        toplam -= değer;
        if toplam <= 0.0 {
            return false;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn düğümleri_yerleştir(
    seri: &AğaçHaritasıSerisi,
    düğümler: &[AğaçDüğümü],
    alan: KesinDikdörtgen,
    sanal_veya_düğüm_katmanı: &ÇözülmüşKatman,
    üst_renk: Option<Renk>,
    gerçek_derinlik: usize,
    görünüm_derinliği: usize,
    yol: &mut Vec<String>,
    indeksler: &HashMap<usize, usize>,
    kimlikler: &mut KimlikSıraları,
    palet: &dyn Fn(usize) -> Renk,
    çıktı: &mut Vec<AğaçHücresi>,
) {
    let içerik = iç_alan(alan, sanal_veya_düğüm_katmanı, gerçek_derinlik > 0);
    let toplam_alan = içerik.genişlik * içerik.yükseklik;
    if toplam_alan <= 0.0 {
        return;
    }

    let mut görünür = düğümler
        .iter()
        .filter_map(|düğüm| {
            let değer = düğüm.etkin_değer();
            (değer.is_finite() && değer > 0.0).then(|| AlanlıDüğüm {
                düğüm,
                veri_sırası: indeksler
                    .get(&std::ptr::from_ref(düğüm).addr())
                    .copied()
                    .unwrap_or_default(),
                değer,
                alan: 0.0,
            })
        })
        .collect::<Vec<_>>();
    match seri.sıralama {
        AğaçHaritasıSırası::Azalan => görünür.sort_by(|a, b| {
            b.değer
                .partial_cmp(&a.değer)
                .unwrap_or(Ordering::Equal)
                .then_with(|| b.veri_sırası.cmp(&a.veri_sırası))
        }),
        AğaçHaritasıSırası::Artan => görünür.sort_by(|a, b| {
            a.değer
                .partial_cmp(&b.değer)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.veri_sırası.cmp(&b.veri_sırası))
        }),
        AğaçHaritasıSırası::Veri => {}
    }
    let mut toplam: f64 = görünür.iter().map(|öğe| öğe.değer).sum();
    if toplam <= 0.0 {
        return;
    }
    if seri.sıralama != AğaçHaritasıSırası::Veri
        && let Some(eşik) = sanal_veya_düğüm_katmanı.görsel.görünür_en_az
    {
        loop {
            let küçük_sıra = match seri.sıralama {
                AğaçHaritasıSırası::Azalan => görünür.len().checked_sub(1),
                AğaçHaritasıSırası::Artan => (!görünür.is_empty()).then_some(0),
                AğaçHaritasıSırası::Veri => None,
            };
            let Some(küçük_sıra) = küçük_sıra else {
                break;
            };
            let küçük = görünür[küçük_sıra].değer;
            if küçük / toplam * toplam_alan >= f64::from(eşik) {
                break;
            }
            görünür.remove(küçük_sıra);
            toplam -= küçük;
            if toplam <= 0.0 {
                return;
            }
        }
    }
    for öğe in &mut görünür {
        öğe.alan = öğe.değer / toplam * toplam_alan;
    }
    let renkler = çocuk_renkleri(
        &görünür,
        düğümler,
        &sanal_veya_düğüm_katmanı.görsel,
        üst_renk,
        palet,
        kimlikler,
    );
    let kutular = kareselleştir(
        görünür.clone(),
        içerik,
        f64::from(sanal_veya_düğüm_katmanı.öğe_stili.boşluk_genişliği),
        f64::from(seri.kare_oranı),
    );

    for ((öğe, kutu), designated_renk) in kutular.into_iter().zip(renkler) {
        let ağaç_derinliği = gerçek_derinlik.saturating_add(1);
        let seviye = derinlik_katmanı(seri, &seri_katmanı(seri), ağaç_derinliği);
        let katman = düğüm_uygula(&seviye, öğe.düğüm);
        let renk = öğe_görsel_rengi(&katman.öğe_stili, designated_renk);
        let düğüm_içi = iç_alan(kutu, &katman, !öğe.düğüm.çocuklar.is_empty());
        let yaprak_derinliği = seri
            .yaprak_derinliği
            .unwrap_or_else(|| seri.en_çok_derinlik.saturating_add(1));
        let derinlikte_yaprak = yaprak_derinliği != usize::MAX
            && yaprak_derinliği <= görünüm_derinliği.saturating_add(1);
        let eşikten_gizli = katman
            .görsel
            .çocuk_görünür_en_az
            .is_some_and(|eşik| düğüm_içi.genişlik * düğüm_içi.yükseklik < f64::from(eşik));
        let inilebilir_yaprak = derinlikte_yaprak
            && görünür_çocuk_var_mı(
                seri,
                &öğe.düğüm.çocuklar,
                düğüm_içi.genişlik * düğüm_içi.yükseklik,
                &katman,
            );
        let yaprak = öğe.düğüm.çocuklar.is_empty() || derinlikte_yaprak || eşikten_gizli;
        yol.push(öğe.düğüm.ad.clone());
        let hücre_sırası = çıktı.len();
        çıktı.push(AğaçHücresi {
            ad: öğe.düğüm.ad.clone(),
            değer: öğe.değer,
            değerler: öğe.düğüm.değerler.clone(),
            alan: kutu.çizim(),
            renk,
            derinlik: gerçek_derinlik,
            yaprak,
            inilebilir_yaprak,
            veri_sırası: öğe.veri_sırası,
            yol: yol.clone(),
            öğe_stili: katman.öğe_stili.clone(),
            etiket: katman.etiket.clone(),
            üst_etiket: katman.üst_etiket.clone(),
            vurgu: katman.vurgu.clone(),
            bağlantı: öğe.düğüm.bağlantı.clone(),
            hedef: öğe.düğüm.hedef.clone(),
        });
        if !yaprak {
            let çocuk_başlangıcı = çıktı.len();
            düğümleri_yerleştir(
                seri,
                &öğe.düğüm.çocuklar,
                kutu,
                &katman,
                Some(renk),
                gerçek_derinlik.saturating_add(1),
                görünüm_derinliği.saturating_add(1),
                yol,
                indeksler,
                kimlikler,
                palet,
                çıktı,
            );
            // `visibleMin` bütün çocukları elediğinde ECharts'ın
            // `viewChildren` dizisi boş kalır ve düğüm içerik dikdörtgeni
            // olan bir yaprak gibi boyanır. Ham veride çocuk bulunması tek
            // başına parent görünümü vermemelidir.
            if çıktı.len() == çocuk_başlangıcı {
                çıktı[hücre_sırası].yaprak = true;
            }
        }
        yol.pop();
    }
}

/// Resmî squarify ve kalıtım kurallarıyla hücre listesini üretir.
pub fn ağaç_haritası_hücreleri(
    seri: &AğaçHaritasıSerisi,
    tuval: Dikdörtgen,
    kök_yolu: &[String],
    palet: &dyn Fn(usize) -> Renk,
) -> Vec<AğaçHücresi> {
    let alan = ağaç_haritası_alanı(seri, tuval);
    let (kökler, inilen, etkin_kök) = etkin_kökler(&seri.kökler, kök_yolu);
    let indeksler = ön_sıra_indeksleri(&seri.kökler);
    let mut kimlikler = KimlikSıraları::default();
    let taban = seri_katmanı(seri);
    let sanal = derinlik_katmanı(seri, &taban, inilen);
    let üst_renk = etkin_kök.and_then(|düğüm| düğüm.renk);
    let mut çıktı = Vec::new();
    let mut yol = kök_yolu.iter().take(inilen).cloned().collect();
    düğümleri_yerleştir(
        seri,
        kökler,
        KesinDikdörtgen::from(alan),
        &sanal,
        üst_renk,
        inilen,
        0,
        &mut yol,
        &indeksler,
        &mut kimlikler,
        palet,
        &mut çıktı,
    );
    çıktı
}

fn etiket_metni(seri: &AğaçHaritasıSerisi, hücre: &AğaçHücresi, etiket: &Etiket) -> String {
    let ham = hücre.ad.clone();
    let metin = etiket
        .biçimleyici
        .as_ref()
        .map_or(ham.clone(), |biçimleyici| {
            if etiket.zengin.is_empty() {
                biçimleyici.uygula_bağlamla(
                    hücre.değer,
                    &binlik_ayır(hücre.değer),
                    seri.ad.as_deref().unwrap_or(""),
                    &ham,
                )
            } else {
                biçimleyici.uygula_bağlamla_zengin(
                    hücre.değer,
                    &binlik_ayır(hücre.değer),
                    seri.ad.as_deref().unwrap_or(""),
                    &ham,
                )
            }
        });
    if hücre.inilebilir_yaprak && !seri.inme_simgesi.is_empty() {
        format!("{} {metin}", seri.inme_simgesi)
    } else {
        metin
    }
}

fn metni_sığdır(çizici: &dyn ÇizimYüzeyi, metin: &str, boyut: f32, genişlik: f32) -> String {
    if genişlik <= 0.0 {
        return String::new();
    }
    let mut metin_genişliği = çizici.yazı_ölç(metin, boyut).0;
    let kutu = (genişlik - 1.0).max(0.0);
    if metin_genişliği <= kutu {
        return metin.to_owned();
    }

    // TreemapView her bağlı metne `truncateMinChar = 2` koyar. zrender
    // ellipsis kararından önce iki ASCII karakterlik alanı ayırır; dar
    // hücrelerde bu nedenle `...` tamamen düşer ve `Versions` yerine `V`
    // gibi tek harflik etiketler kalabilir.
    let ascii = çizici.yazı_ölç("a", boyut).0;
    let mut üç_nokta_alanı = kutu;
    for _ in 0..2 {
        if üç_nokta_alanı >= ascii {
            üç_nokta_alanı -= ascii;
        }
    }
    let üç_nokta = if çizici.yazı_ölç("...", boyut).0 > üç_nokta_alanı {
        ""
    } else {
        "..."
    };
    let içerik = kutu - çizici.yazı_ölç(üç_nokta, boyut).0;
    let mut karakterler = metin.chars().collect::<Vec<_>>();
    for geçiş in 0..=2 {
        if metin_genişliği <= içerik || geçiş >= 2 {
            return karakterler.into_iter().chain(üç_nokta.chars()).collect();
        }
        let yeni_uzunluk = if geçiş == 0 {
            let mut ölçü = 0.0_f32;
            let mut uzunluk = 0_usize;
            while ölçü < içerik {
                let Some(karakter) = karakterler.get(uzunluk) else {
                    break;
                };
                ölçü += çizici.yazı_ölç(&karakter.to_string(), boyut).0;
                uzunluk += 1;
            }
            uzunluk
        } else if metin_genişliği > 0.0 {
            ((karakterler.len() as f32 * içerik / metin_genişliği).floor() as usize)
                .min(karakterler.len())
        } else {
            0
        };
        karakterler.truncate(yeni_uzunluk);
        metin_genişliği = çizici
            .yazı_ölç(&karakterler.iter().collect::<String>(), boyut)
            .0;
    }
    unreachable!("zrender kısaltma döngüsü üçüncü geçişte dönmelidir")
}

fn etiketi_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &AğaçHaritasıSerisi,
    hücre: &AğaçHücresi,
    etiket: &Etiket,
    üst_etiket: bool,
) {
    if !etiket.göster {
        return;
    }
    let kenar = hücre.öğe_stili.taban.kenarlık_kalınlığı.max(0.0);
    let etiket_alanı = if üst_etiket {
        let yükseklik = hücre
            .üst_etiket
            .yazı
            .yükseklik
            .unwrap_or(20.0)
            .max(kenar)
            .min(hücre.alan.yükseklik);
        Dikdörtgen::yeni(
            hücre.alan.x + kenar,
            hücre.alan.y,
            (hücre.alan.genişlik - 2.0 * kenar).max(0.0),
            yükseklik,
        )
    } else {
        Dikdörtgen::yeni(
            hücre.alan.x + kenar,
            hücre.alan.y + kenar,
            (hücre.alan.genişlik - 2.0 * kenar).max(0.0),
            (hücre.alan.yükseklik - 2.0 * kenar).max(0.0),
        )
    };
    let boyut = etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let iç_boşluk = etiket.yazı.iç_boşluk.unwrap_or([0.0; 4]);
    let kullanılabilir = (etiket_alanı.genişlik - iç_boşluk[1] - iç_boşluk[3]).max(0.0);
    let ham = etiket_metni(seri, hücre, etiket);
    let satır_yüksekliği = etiket.yazı.satır_yüksekliği.unwrap_or(boyut * 1.4);
    let kullanılabilir_yükseklik = (etiket_alanı.yükseklik - iç_boşluk[0] - iç_boşluk[2]).max(0.0);
    let en_çok_satır = (kullanılabilir_yükseklik / satır_yüksekliği).floor() as usize;
    if en_çok_satır == 0 {
        return;
    }
    let (varsayılan_yatay, varsayılan_dikey, çapa) = if üst_etiket {
        (
            YatayHiza::Sol,
            DikeyHiza::Orta,
            (etiket_alanı.x, etiket_alanı.merkez().1),
        )
    } else {
        match etiket.konum {
            EtiketKonumu::İçSolÜst | EtiketKonumu::SolÜst => (
                YatayHiza::Sol,
                DikeyHiza::Üst,
                (etiket_alanı.x, etiket_alanı.y),
            ),
            EtiketKonumu::İçSağÜst | EtiketKonumu::SağÜst => (
                YatayHiza::Sağ,
                DikeyHiza::Üst,
                (etiket_alanı.sağ(), etiket_alanı.y),
            ),
            EtiketKonumu::İçSol | EtiketKonumu::Sol => (
                YatayHiza::Sol,
                DikeyHiza::Orta,
                (etiket_alanı.x, etiket_alanı.merkez().1),
            ),
            EtiketKonumu::İçSağ | EtiketKonumu::Sağ => (
                YatayHiza::Sağ,
                DikeyHiza::Orta,
                (etiket_alanı.sağ(), etiket_alanı.merkez().1),
            ),
            _ => (YatayHiza::Orta, DikeyHiza::Orta, etiket_alanı.merkez()),
        }
    };
    let yatay = match etiket.yatay_hiza {
        Some(YazıYatayHizası::Sol) => YatayHiza::Sol,
        Some(YazıYatayHizası::Orta) => YatayHiza::Orta,
        Some(YazıYatayHizası::Sağ) => YatayHiza::Sağ,
        None => varsayılan_yatay,
    };
    let dikey = match etiket.dikey_hiza {
        Some(YazıDikeyHizası::Üst) => DikeyHiza::Üst,
        Some(YazıDikeyHizası::Orta) => DikeyHiza::Orta,
        Some(YazıDikeyHizası::Alt) => DikeyHiza::Alt,
        None => varsayılan_dikey,
    };

    let renk = etiket.yazı.renk.unwrap_or_else(|| {
        if üst_etiket {
            Renk::BEYAZ
        } else {
            hücre.renk.zrender_iç_etiket_stili(false).0
        }
    });
    if !etiket.zengin.is_empty() {
        // TreemapView bağlı zrender metninin boyutlarını her güncellemede
        // hücrenin içerik dikdörtgenine sabitler. Ortak rich-text motoruna
        // aynı genişliği verince koşu padding'i, arka planı ve truncate
        // davranışı Pasta/Radar/Huni ile birebir aynı kod yolunu izler.
        let mut zengin_etiket = etiket.clone();
        zengin_etiket.yazı.genişlik = Some(Uzunluk::Piksel(kullanılabilir));
        zengin_etiket.yazı.yükseklik = Some(kullanılabilir_yükseklik);
        let mut kırpılmış = |yüzey: &mut dyn ÇizimYüzeyi| {
            zengin_etiketi_hizalı_yaz(yüzey, &ham, &zengin_etiket, çapa, yatay, dikey, renk, 0.0);
        };
        çizici.kırpılı(etiket_alanı, &mut kırpılmış);
        return;
    }

    let satırlar = zengin_metin_içeriği(ham)
        .lines()
        .map(|satır| {
            if etiket.yazı.taşmayı_kısalt {
                metni_sığdır(çizici, satır, boyut, kullanılabilir)
            } else {
                satır.to_owned()
            }
        })
        .filter(|satır| !satır.is_empty())
        .take(en_çok_satır)
        .collect::<Vec<_>>();
    if satırlar.is_empty() {
        return;
    }
    let toplam_yükseklik = satır_yüksekliği * satırlar.len() as f32;
    let başlangıç_y = match dikey {
        DikeyHiza::Üst => çapa.1 + iç_boşluk[0],
        DikeyHiza::Orta => çapa.1 - toplam_yükseklik / 2.0,
        DikeyHiza::Alt => çapa.1 - toplam_yükseklik,
    };
    let mut kırpılmış = |yüzey: &mut dyn ÇizimYüzeyi| {
        for (sıra, satır) in satırlar.iter().enumerate() {
            let y = başlangıç_y + sıra as f32 * satır_yüksekliği;
            let x = match yatay {
                YatayHiza::Sol => çapa.0 + iç_boşluk[3],
                YatayHiza::Orta => çapa.0,
                YatayHiza::Sağ => çapa.0 - iç_boşluk[1],
            };
            yüzey.yazı(
                satır,
                (x + etiket.kayma.0, y + etiket.kayma.1),
                yatay,
                DikeyHiza::Üst,
                boyut,
                renk,
                etiket.yazı.kalın,
            );
        }
    };
    çizici.kırpılı(etiket_alanı, &mut kırpılmış);
}

fn kırıntı_yolu(kutu: Dikdörtgen, ilk: bool, son: bool) -> Yol {
    let ok = (kutu.yükseklik / 2.0).min(10.0);
    let girinti = if ilk { 0.0 } else { ok };
    let çıkıntı = if son { 0.0 } else { ok };
    let mut yol = Yol::yeni();
    yol.taşı((kutu.x, kutu.y));
    yol.çiz((kutu.sağ() - çıkıntı, kutu.y));
    if !son {
        yol.çiz((kutu.sağ(), kutu.merkez().1));
        yol.çiz((kutu.sağ() - çıkıntı, kutu.alt()));
    } else {
        yol.çiz((kutu.sağ(), kutu.alt()));
    }
    yol.çiz((kutu.x, kutu.alt()));
    if !ilk {
        yol.çiz((kutu.x + girinti, kutu.merkez().1));
    }
    yol.kapat();
    yol
}

fn kırıntıları_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &AğaçHaritasıSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    kök_yolu: &[String],
    kırıntılar: &mut Vec<(Dikdörtgen, usize, usize)>,
) {
    if !seri.kırıntı.göster {
        return;
    }
    let boyut = seri.kırıntı.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    let mut parçalar = if kök_yolu.is_empty() {
        if let Some(ad) = seri.ad.as_ref().filter(|ad| !ad.is_empty()) {
            vec![ad.clone()]
        } else {
            // ECharts'ın adsız sanal kökünde başlangıç breadcrumb hedefi,
            // azalan yerleşimin ilk (en büyük) dalındaki son alt düğümdür.
            // `treemap-simple` resmî kartındaki nodeB › nodeBa › nodeBa1
            // zinciri bu tarihsel davranıştan gelir.
            // Zincirin başındaki adsız sanal seri kökü de 25 px'lik boş
            // breadcrumb öğesi olarak çizilir.
            let mut yol = vec![String::new()];
            let mut düğümler = seri.kökler.as_slice();
            while let Some(düğüm) = düğümler.iter().max_by(|a, b| {
                a.etkin_değer()
                    .partial_cmp(&b.etkin_değer())
                    .unwrap_or(Ordering::Equal)
            }) {
                yol.push(düğüm.ad.clone());
                düğümler = &düğüm.çocuklar;
            }
            yol
        }
    } else {
        let mut yol = vec![seri.ad.clone().unwrap_or_default()];
        yol.extend(kök_yolu.iter().cloned());
        yol
    };
    if parçalar.is_empty() {
        parçalar.push(String::new());
    }
    const METİN_YATAY_DOLGUSU: f32 = 8.0;
    const ÖĞE_BOŞLUĞU: f32 = 8.0;
    let genişlikler = parçalar
        .iter()
        .map(|parça| {
            if parça.is_empty() {
                seri.kırıntı.boş_öğe_genişliği
            } else {
                (çizici.yazı_ölç(parça, boyut).0 + 2.0 * METİN_YATAY_DOLGUSU)
                    .max(seri.kırıntı.boş_öğe_genişliği)
            }
        })
        .collect::<Vec<_>>();
    // ECharts `Breadcrumb._prepare`, son öğeden sonra da ITEM_GAP'i toplam
    // box-layout genişliğine katar. Grup şeklinin sağında kalan bu pay,
    // `left: center` çözümünde görünür zinciri yarım boşluk sola taşır.
    let toplam: f32 = genişlikler
        .iter()
        .map(|genişlik| genişlik + ÖĞE_BOŞLUĞU)
        .sum();
    let mut x = seri.kırıntı.sol.map_or_else(
        || {
            seri.kırıntı.sağ.map_or(tuval.x, |sağ| {
                tuval.sağ() - sağ.çöz(tuval.genişlik) - toplam
            })
        },
        |sol| {
            let çözülen = sol.çöz(tuval.genişlik);
            if matches!(sol, crate::model::Uzunluk::Yüzde(_)) {
                tuval.x + çözülen - toplam / 2.0
            } else {
                tuval.x + çözülen
            }
        },
    );
    let y = seri.kırıntı.üst.map_or_else(
        || {
            seri.kırıntı.alt.map_or(tuval.y, |alt| {
                tuval.alt() - alt.çöz(tuval.yükseklik) - seri.kırıntı.yükseklik
            })
        },
        |üst| tuval.y + üst.çöz(tuval.yükseklik),
    );
    for (sıra, (parça, genişlik)) in parçalar.iter().zip(genişlikler).enumerate() {
        let kutu = Dikdörtgen::yeni(x, y, genişlik, seri.kırıntı.yükseklik);
        let yol = kırıntı_yolu(kutu, sıra == 0, sıra + 1 == parçalar.len());
        let dolgu = seri
            .kırıntı
            .öğe_stili
            .renk
            .clone()
            .unwrap_or_else(|| Dolgu::Düz(tema::nötr_05()));
        çizici.yol_doldur(&yol, &dolgu);
        if let Some(renk) = seri.kırıntı.öğe_stili.kenarlık_rengi
            && seri.kırıntı.öğe_stili.kenarlık_kalınlığı > 0.0
        {
            çizici.yol_çiz(
                &yol,
                seri.kırıntı.öğe_stili.kenarlık_kalınlığı,
                renk,
                seri.kırıntı.öğe_stili.kenarlık_türü,
            );
        }
        if !parça.is_empty() {
            çizici.yazı(
                parça,
                kutu.merkez(),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                boyut,
                seri.kırıntı.yazı.renk.unwrap_or_else(tema::ikincil_metin),
                seri.kırıntı.yazı.kalın,
            );
        }
        kırıntılar.push((kutu, genel_sıra, sıra));
        x += genişlik + ÖĞE_BOŞLUĞU;
    }
}

/// Ağaç haritasını çizer ve isabet bölgelerini toplar.
#[allow(clippy::too_many_arguments)]
pub fn ağaç_haritası_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &AğaçHaritasıSerisi,
    genel_sıra: usize,
    tuval: Dikdörtgen,
    yerleşim_referansı: Dikdörtgen,
    palet: &dyn Fn(usize) -> Renk,
    ilerleme: f32,
    kök_yolu: &[String],
    görünüm: (f32, f32, f32),
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
    kırıntılar: &mut Vec<(Dikdörtgen, usize, usize)>,
) {
    let alan = ağaç_haritası_alanı(seri, yerleşim_referansı);
    let kırpma = match seri.kırpma_penceresi {
        AğaçHaritasıKırpmaPenceresi::Özgün => alan,
        AğaçHaritasıKırpmaPenceresi::TamEkran => tuval,
    };
    let ölçek_sınırı = (seri.en_küçük_ölçek, seri.en_büyük_ölçek);
    let görünüm_alanı = ağaç_haritası_görünümünü_uygula(alan, alan, görünüm, ölçek_sınırı);
    let (_, inilen, etkin_kök) = etkin_kökler(&seri.kökler, kök_yolu);
    let taban = seri_katmanı(seri);
    let mut kök_katmanı = derinlik_katmanı(seri, &taban, inilen);
    if let Some(kök) = etkin_kök {
        kök_katmanı = düğüm_uygula(&kök_katmanı, kök);
    }
    let kök_designated_rengi = etkin_kök
        .and_then(|kök| kök.renk)
        .unwrap_or_else(|| palet(0));
    let kök_görsel_rengi = öğe_görsel_rengi(&kök_katmanı.öğe_stili, kök_designated_rengi);
    let kök_kenarlık_rengi = kök_katmanı
        .öğe_stili
        .kenarlık_rengi_doygunluğu
        .map(|açıklık| kök_görsel_rengi.açıklık_ile(açıklık))
        .or(kök_katmanı.öğe_stili.taban.kenarlık_rengi)
        .unwrap_or(Renk::BEYAZ)
        .opaklık(kök_katmanı.öğe_stili.taban.opaklık.unwrap_or(1.0));
    // ECharts sanal kökü de normal bir Treemap background öğesi olarak
    // boyar. Kök çizilmezse level[0] borderColor/gapWidth alanları tuvalin
    // beyazına düşer (Gradient Mapping ve Obama örneklerindeki siyah çerçeve
    // bunun doğrudan kanıtıdır).
    let mut hücreler = ağaç_haritası_hücreleri(seri, yerleşim_referansı, kök_yolu, palet);
    for hücre in &mut hücreler {
        hücre.alan =
            ağaç_haritası_görünümünü_uygula(hücre.alan, alan, görünüm, ölçek_sınırı);
    }
    let vurgulu = fare
        .filter(|fare| kırpma.içeriyor_mu(*fare))
        .and_then(|fare| {
            hücreler
                .iter()
                .rev()
                .find(|hücre| hücre.alan.içeriyor_mu(fare))
                .map(|hücre| hücre.veri_sırası)
        });
    let opaklık = ilerleme.clamp(0.0, 1.0);
    let mut gövde = |çizici: &mut dyn ÇizimYüzeyi| {
        çizici.dikdörtgen(
            görünüm_alanı,
            &Dolgu::Düz(kök_kenarlık_rengi),
            kök_katmanı.öğe_stili.taban.kenarlık_yarıçapı,
            None,
        );
        for hücre in &hücreler {
            let mut stil = hücre.öğe_stili.clone();
            if vurgulu == Some(hücre.veri_sırası)
                && let Some(vurgu_stili) = &hücre.vurgu.öğe_stili
            {
                stil = öğe_stili_yama_uygula(&stil, vurgu_stili);
            }
            let mut dolgu = stil.taban.renk.clone().unwrap_or(Dolgu::Düz(hücre.renk));
            if let Dolgu::Düz(renk) = &mut dolgu {
                if let Some(doygunluk) = stil.renk_doygunluğu
                    && doygunluk != 0.0
                {
                    *renk = renk.açıklık_ile(doygunluk);
                }
                if let Some(alfa) = stil.renk_alfası
                    && alfa != 0.0
                {
                    *renk = renk.alfa_ile(alfa);
                }
            }
            dolgu = dolgu.opaklık(stil.taban.opaklık.unwrap_or(1.0) * opaklık);
            let renk = dolgu.temsilî();
            if stil.taban.gölge_bulanıklığı > 0.0
                && let Some(gölge) = stil.taban.gölge_rengi
            {
                çizici.gölge(
                    hücre.alan,
                    stil.taban.kenarlık_yarıçapı[0],
                    gölge,
                    stil.taban.gölge_bulanıklığı,
                );
            }
            let kenarlık_rengi = stil
                .kenarlık_rengi_doygunluğu
                .map(|doygunluk| renk.açıklık_ile(doygunluk))
                .or(stil.taban.kenarlık_rengi)
                .unwrap_or(Renk::BEYAZ)
                .opaklık(stil.taban.opaklık.unwrap_or(1.0) * opaklık);
            // ECharts Treemap iki ayrı dikdörtgen kullanır: bütün düğümlerde
            // `borderColor` ile boyanan arka plan; yalnız yapraklarda bunun
            // `borderWidth` kadar içine yerleşen görsel renkli içerik. Böylece
            // gap ve üst etiket şeritleri, kenarlık çizgisinin merkezlenmesine
            // bağlı kalmadan doğru katmanda görünür.
            çizici.dikdörtgen(
                hücre.alan,
                &Dolgu::Düz(kenarlık_rengi),
                stil.taban.kenarlık_yarıçapı,
                None,
            );
            if hücre.yaprak {
                let kenar = stil.taban.kenarlık_kalınlığı.max(0.0);
                let içerik = Dikdörtgen::yeni(
                    hücre.alan.x + kenar,
                    hücre.alan.y + kenar,
                    (hücre.alan.genişlik - 2.0 * kenar).max(0.0),
                    (hücre.alan.yükseklik - 2.0 * kenar).max(0.0),
                );
                çizici.dikdörtgen(içerik, &dolgu, stil.taban.kenarlık_yarıçapı, None);
            }
            if !seri.sessiz
                && let Some(isabet_alanı) = dikdörtgen_kesişimi(hücre.alan, kırpma)
            {
                isabetler.push(İsabetBölgesi {
                    seri_sırası: genel_sıra,
                    veri_sırası: hücre.veri_sırası,
                    seri_adı: seri.ad.clone(),
                    ad: Some(hücre.ad.clone()),
                    değer: Some(hücre.değer),
                    geometri: İsabetGeometrisi::Dikdörtgen(isabet_alanı),
                });
            }
        }
        // Metinler çocuk dolgularının altında kalmamalıdır.
        for hücre in &hücreler {
            let vurguda = vurgulu == Some(hücre.veri_sırası);
            if !hücre.yaprak && hücre.üst_etiket.göster {
                let etiket = if vurguda {
                    hücre.vurgu.üst_etiket.as_ref().map_or_else(
                        || hücre.üst_etiket.clone(),
                        |yama| yama.uygula(&hücre.üst_etiket),
                    )
                } else {
                    hücre.üst_etiket.clone()
                };
                etiketi_çiz(çizici, seri, hücre, &etiket, true);
            } else if hücre.yaprak {
                let etiket = if vurguda {
                    hücre
                        .vurgu
                        .etiket
                        .as_ref()
                        .map_or_else(|| hücre.etiket.clone(), |yama| yama.uygula(&hücre.etiket))
                } else {
                    hücre.etiket.clone()
                };
                etiketi_çiz(çizici, seri, hücre, &etiket, false);
            }
        }
    };
    çizici.kırpılı(kırpma, &mut gövde);
    kırıntıları_çiz(çizici, seri, genel_sıra, tuval, kök_yolu, kırıntılar);
}

/// Hücre değer metni (ipucu için).
pub fn hücre_değer_metni(değer: f64) -> String {
    binlik_ayır(değer)
}

/// Hücre geometrisini kararlı FNV-1a özetiyle kilitler.
pub fn ağaç_haritası_sahne_özeti(
    seri: &AğaçHaritasıSerisi,
    tuval: Dikdörtgen,
    kök_yolu: &[String],
    palet: &dyn Fn(usize) -> Renk,
) -> AğaçHaritasıSahneÖzeti {
    fn bayt(özet: &mut u64, değer: u8) {
        *özet ^= u64::from(değer);
        *özet = özet.wrapping_mul(0x0000_0100_0000_01b3);
    }
    fn sayı(özet: &mut u64, değer: u64) {
        for bayt_değeri in değer.to_le_bytes() {
            bayt(özet, bayt_değeri);
        }
    }
    let hücreler = ağaç_haritası_hücreleri(seri, tuval, kök_yolu, palet);
    let mut özet = 0xcbf2_9ce4_8422_2325_u64;
    sayı(&mut özet, hücreler.len() as u64);
    for hücre in &hücreler {
        sayı(&mut özet, hücre.veri_sırası as u64);
        sayı(&mut özet, hücre.derinlik as u64);
        for koordinat in [
            hücre.alan.x,
            hücre.alan.y,
            hücre.alan.genişlik,
            hücre.alan.yükseklik,
        ] {
            sayı(&mut özet, ((koordinat * 1_000.0).round() as i64) as u64);
        }
        for kanal in [
            hücre.renk.kırmızı,
            hücre.renk.yeşil,
            hücre.renk.mavi,
            hücre.renk.alfa,
        ] {
            bayt(&mut özet, (kanal * 255.0).round() as u8);
        }
        bayt(&mut özet, u8::from(hücre.yaprak));
        bayt(&mut özet, u8::from(hücre.inilebilir_yaprak));
    }
    AğaçHaritasıSahneÖzeti {
        hücre_sayısı: hücreler.len(),
        yaprak_sayısı: hücreler.iter().filter(|hücre| hücre.yaprak).count(),
        üst_etiket_sayısı: hücreler
            .iter()
            .filter(|hücre| !hücre.yaprak && hücre.üst_etiket.göster)
            .count(),
        etiket_sayısı: hücreler
            .iter()
            .filter(|hücre| {
                (hücre.yaprak && hücre.etiket.göster) || (!hücre.yaprak && hücre.üst_etiket.göster)
            })
            .count(),
        koordinat_sayısı: hücreler.len() * 4,
        fnv1a_64: özet,
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::model::agac::{
        AğaçHaritasıGörseli, AğaçHaritasıRenkEşlemesi, AğaçHaritasıSeviyesi
    };

    fn palet(sıra: usize) -> Renk {
        [Renk::onaltılık(0x5070dd), Renk::onaltılık(0xb6d634)][sıra % 2]
    }

    #[test]
    fn squarify_alan_toplamini_korur_ve_sabit_siralidir() {
        let seri = AğaçHaritasıSerisi::yeni()
            .kökler([AğaçDüğümü::yaprak("A", 6.0), AğaçDüğümü::yaprak("B", 4.0)]);
        let hücreler = ağaç_haritası_hücreleri(
            &seri,
            Dikdörtgen::yeni(0.0, 0.0, 200.0, 150.0),
            &[],
            &palet,
        );
        assert_eq!(hücreler.len(), 2);
        assert_eq!(hücreler[0].ad, "A");
        assert_eq!(hücreler[1].ad, "B");
        let görünür = ağaç_haritası_alanı(&seri, Dikdörtgen::yeni(0.0, 0.0, 200.0, 150.0));
        let toplam: f32 = hücreler
            .iter()
            .map(|hücre| hücre.alan.genişlik * hücre.alan.yükseklik)
            .sum();
        assert!((toplam - görünür.genişlik * görünür.yükseklik).abs() < 1.0);
    }

    #[test]
    fn visible_min_leaf_depth_ve_seviye_gorseli_uygulanir() {
        let seri = AğaçHaritasıSerisi::yeni()
            .yaprak_derinliği(1)
            .görsel(AğaçHaritasıGörseli::seri_varsayılanı().görünür_en_az(100.0))
            .seviyeler([AğaçHaritasıSeviyesi::yeni().görsel(
                AğaçHaritasıGörseli::yeni()
                    .renkler(["#ff0000", "#00ff00"])
                    .eşleme(AğaçHaritasıRenkEşlemesi::Sıra),
            )])
            .kökler([
                AğaçDüğümü::dal("A", vec![AğaçDüğümü::yaprak("Aa", 9.0)]),
                AğaçDüğümü::yaprak("tiny", 0.0001),
            ]);
        let hücreler = ağaç_haritası_hücreleri(
            &seri,
            Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0),
            &[],
            &palet,
        );
        assert_eq!(hücreler.len(), 1);
        assert_eq!(hücreler[0].ad, "A");
        assert!(hücreler[0].yaprak);
        assert_eq!(hücreler[0].renk, Renk::from("#ff0000"));
    }

    #[test]
    fn leaf_depth_view_root_degistiginde_yeniden_sifirdan_sayilir() {
        let seri = AğaçHaritasıSerisi::yeni()
            .yaprak_derinliği(1)
            .kökler([AğaçDüğümü::dal(
                "A",
                vec![AğaçDüğümü::dal(
                    "B",
                    vec![AğaçDüğümü::yaprak("C", 4.0)],
                )],
            )]);
        let tuval = Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0);
        let ilk = ağaç_haritası_hücreleri(&seri, tuval, &[], &palet);
        assert_eq!(ilk.len(), 1);
        assert_eq!(ilk[0].ad, "A");
        assert!(ilk[0].inilebilir_yaprak);

        let inilmiş = ağaç_haritası_hücreleri(&seri, tuval, &["A".to_owned()], &palet);
        assert_eq!(inilmiş.len(), 1);
        assert_eq!(inilmiş[0].ad, "B");
        assert!(inilmiş[0].inilebilir_yaprak);
        assert_eq!(inilmiş[0].yol, vec!["A".to_owned(), "B".to_owned()]);
    }

    #[test]
    fn item_style_renk_alpha_ve_doygunluk_designated_rengi_degistirir() {
        let seri = AğaçHaritasıSerisi::yeni().kökler([AğaçDüğümü::yaprak("A", 1.0)
            .ağaç_haritası_öğe_stili(
                AğaçHaritasıÖğeStili::yeni()
                    .renk("#ff0000")
                    .renk_doygunluğu(0.25)
                    .renk_alfası(0.4),
            )]);
        let hücreler = ağaç_haritası_hücreleri(
            &seri,
            Dikdörtgen::yeni(0.0, 0.0, 300.0, 200.0),
            &[],
            &palet,
        );
        assert_eq!(hücreler.len(), 1);
        assert_eq!(hücreler[0].renk, Renk::kyma(128.0 / 255.0, 0.0, 0.0, 0.4));
    }

    #[test]
    fn calendar_ve_matrix_kutu_referansi_seri_box_olculerini_yerellestirir() {
        let seri = AğaçHaritasıSerisi::yeni().sol(10).sağ(20).üst(5).alt(15);
        let sağlayıcı = Dikdörtgen::yeni(100.0, 50.0, 240.0, 160.0);
        assert_eq!(
            ağaç_haritası_alanı(&seri, sağlayıcı),
            Dikdörtgen::yeni(110.0, 55.0, 210.0, 140.0)
        );
    }

    #[test]
    fn root_rect_donusumu_scale_limit_ve_clip_isabetini_birlikte_korur() {
        let seri = AğaçHaritasıSerisi::yeni()
            .ölçek_sınırı(0.5, 2.0)
            .kökler([AğaçDüğümü::yaprak("A", 2.0), AğaçDüğümü::yaprak("B", 1.0)]);
        let tuval = Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0);
        let alan = ağaç_haritası_alanı(&seri, tuval);
        let dönüşmüş = ağaç_haritası_görünümünü_uygula(
            alan,
            alan,
            (10.0, -5.0, 9.0),
            (seri.en_küçük_ölçek, seri.en_büyük_ölçek),
        );
        assert!((dönüşmüş.genişlik - alan.genişlik * 2.0).abs() < 1e-4);
        assert!((dönüşmüş.merkez().0 - (alan.merkez().0 + 10.0)).abs() < 1e-4);
        assert!((dönüşmüş.merkez().1 - (alan.merkez().1 - 5.0)).abs() < 1e-4);

        let mut yüzey = KayıtYüzeyi::yeni(400.0, 300.0);
        let mut isabetler = Vec::new();
        let mut kırıntılar = Vec::new();
        ağaç_haritası_çiz(
            &mut yüzey,
            &seri,
            0,
            tuval,
            tuval,
            &palet,
            1.0,
            &[],
            (200.0, 0.0, 2.0),
            None,
            &mut isabetler,
            &mut kırıntılar,
        );
        assert!(!isabetler.is_empty());
        for isabet in isabetler {
            let İsabetGeometrisi::Dikdörtgen(kutu) = isabet.geometri else {
                panic!("Treemap isabeti dikdörtgen olmalı");
            };
            assert!(kutu.x >= alan.x && kutu.y >= alan.y);
            assert!(kutu.sağ() <= alan.sağ() && kutu.alt() <= alan.alt());
        }
    }

    #[test]
    fn visible_min_kapatma_kalitimi_sifir_degerle_ezer() {
        let görsel = AğaçHaritasıGörseli::yeni().görünür_eşiği_kapalı();
        assert_eq!(görsel.görünür_en_az, Some(0.0));
    }
}
