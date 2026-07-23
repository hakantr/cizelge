//! Hiyerarşik veri modeli — ECharts `data/Tree.ts`'in sadeleştirilmiş
//! karşılığı; ağaç haritası (treemap), güneş patlaması (sunburst) ve ağaç
//! (tree) serilerinin ortak veri yapısı.

use crate::model::Uzunluk;
use crate::model::seri::Sembol;
use crate::model::stil::{EtiketYaması, YazıStili, ÇizgiStili, ÖğeStili};
use crate::renk::Renk;

/// Tree yerleşimi (`series-tree.layout`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçYerleşimi {
    /// ECharts `'orthogonal'`.
    #[default]
    Dik,
    /// ECharts `'radial'`.
    Radyal,
}

/// Dik Tree yerleşiminin büyüme yönü (`series-tree.orient`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçYönü {
    /// ECharts `'LR'` / geriye uyumlu `'horizontal'`.
    #[default]
    SoldanSağa,
    /// ECharts `'RL'`.
    SağdanSola,
    /// ECharts `'TB'` / geriye uyumlu `'vertical'`.
    ÜsttenAlta,
    /// ECharts `'BT'`.
    AlttanÜste,
}

/// Tree üst-çocuk bağı geometrisi (`series-tree.edgeShape`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçKenarBiçimi {
    /// İki kontrol noktalı Bezier (`'curve'`).
    #[default]
    Eğri,
    /// Ortak çatallı dik parçalar (`'polyline'`).
    Kırık,
}

/// Tree görünümünde izin verilen gezinme (`series-tree.roam`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçGezinmesi {
    #[default]
    Kapalı,
    Açık,
    Kaydır,
    Ölçekle,
}

impl AğaçGezinmesi {
    pub fn kaydırılabilir(self) -> bool {
        matches!(self, Self::Açık | Self::Kaydır)
    }

    pub fn ölçeklenebilir(self) -> bool {
        matches!(self, Self::Açık | Self::Ölçekle)
    }
}

/// Vurguda ilişkili düğüm kümesi (`emphasis.focus`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçVurguOdağı {
    #[default]
    Yok,
    Ata,
    AltSoy,
    İlişkili,
    Öz,
}

/// Treemap kardeş düğümlerinin yerleşim sırası (`series-treemap.sort`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçHaritasıSırası {
    /// ECharts `true` / `'desc'` öntanımlısı.
    #[default]
    Azalan,
    /// ECharts `'asc'`.
    Artan,
    /// ECharts `false`; veri sırasını korur ve `visibleMin` süzmesini kapatır.
    Veri,
}

/// Yakınlaştırılmış Treemap görünümünün kırpma penceresi (`clipWindow`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçHaritasıKırpmaPenceresi {
    #[default]
    Özgün,
    TamEkran,
}

/// Treemap düğüm tıklamasının resmî davranışı (`nodeClick`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçHaritasıDüğümTıklaması {
    #[default]
    DüğümeYakınlaştır,
    BağlantıyıAç,
    Kapalı,
}

/// Treemap paletinin kardeşler arasında dağıtılma anahtarı.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AğaçHaritasıRenkEşlemesi {
    Değer,
    #[default]
    Sıra,
    Kimlik,
}

/// `visualDimension`: sayısal boyut sırası ya da adlandırılmış boyut.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AğaçHaritasıGörselBoyutu {
    Sıra(usize),
    Ad(String),
}

impl Default for AğaçHaritasıGörselBoyutu {
    fn default() -> Self {
        Self::Sıra(0)
    }
}

/// Treemap'e özgü `itemStyle` uzantıları. Ortak şekil özellikleri `taban`
/// üzerinde, hücreler arası geometri ve türetilmiş kenarlık rengi burada
/// tutulur.
#[derive(Clone, PartialEq, Debug)]
pub struct AğaçHaritasıÖğeStili {
    pub taban: ÖğeStili,
    /// `itemStyle.colorAlpha`: kalıtılmış/dolgu renginin mutlak alfa kanalı.
    pub renk_alfası: Option<f32>,
    /// ECharts'ın tarihsel `itemStyle.colorSaturation` alanı; resmî
    /// `modifyHSL(color, null, null, value)` akışı nedeniyle HSL açıklığına
    /// uygulanır.
    pub renk_doygunluğu: Option<f32>,
    pub boşluk_genişliği: f32,
    pub kenarlık_rengi_doygunluğu: Option<f32>,
    pub(crate) kenarlık_kalınlığı_belirtildi: bool,
    pub(crate) kenarlık_yarıçapı_belirtildi: bool,
    pub(crate) boşluk_genişliği_belirtildi: bool,
}

impl Default for AğaçHaritasıÖğeStili {
    fn default() -> Self {
        Self {
            // Bu tür aynı zamanda level/node yaması olduğundan belirtilmemiş
            // `borderColor` burada `None` kalır; ECharts seri öntanımlısı
            // AğaçHaritasıSerisi üzerinde açıkça beyaza kurulur.
            taban: ÖğeStili::default(),
            renk_alfası: None,
            renk_doygunluğu: None,
            boşluk_genişliği: 0.0,
            kenarlık_rengi_doygunluğu: None,
            kenarlık_kalınlığı_belirtildi: false,
            kenarlık_yarıçapı_belirtildi: false,
            boşluk_genişliği_belirtildi: false,
        }
    }
}

impl AğaçHaritasıÖğeStili {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn taban(mut self, stil: ÖğeStili) -> Self {
        self.kenarlık_kalınlığı_belirtildi = stil.kenarlık_kalınlığı != 0.0;
        self.kenarlık_yarıçapı_belirtildi = stil.kenarlık_yarıçapı != [0.0; 4];
        self.taban = stil;
        self
    }

    pub fn renk(mut self, renk: impl Into<crate::renk::Dolgu>) -> Self {
        self.taban = self.taban.renk(renk);
        self
    }

    pub fn kenarlık_rengi(mut self, renk: impl Into<Renk>) -> Self {
        self.taban = self.taban.kenarlık_rengi(renk);
        self
    }

    pub fn kenarlık_kalınlığı(mut self, kalınlık: f32) -> Self {
        self.taban = self.taban.kenarlık_kalınlığı(kalınlık.max(0.0));
        self.kenarlık_kalınlığı_belirtildi = true;
        self
    }

    pub fn kenarlık_yarıçapı(
        mut self,
        yarıçap: impl Into<crate::model::stil::KöşeYarıçapı>,
    ) -> Self {
        self.taban = self.taban.kenarlık_yarıçapı(yarıçap);
        self.kenarlık_yarıçapı_belirtildi = true;
        self
    }

    pub fn opaklık(mut self, opaklık: f32) -> Self {
        self.taban = self.taban.opaklık(opaklık.clamp(0.0, 1.0));
        self
    }

    pub fn renk_alfası(mut self, alfa: f32) -> Self {
        self.renk_alfası = alfa.is_finite().then(|| alfa.clamp(0.0, 1.0));
        self
    }

    pub fn renk_doygunluğu(mut self, doygunluk: f32) -> Self {
        self.renk_doygunluğu = doygunluk.is_finite().then(|| doygunluk.clamp(0.0, 1.0));
        self
    }

    pub fn boşluk_genişliği(mut self, genişlik: f32) -> Self {
        self.boşluk_genişliği = genişlik.max(0.0);
        self.boşluk_genişliği_belirtildi = true;
        self
    }

    pub fn kenarlık_rengi_doygunluğu(mut self, doygunluk: f32) -> Self {
        self.kenarlık_rengi_doygunluğu = Some(doygunluk.clamp(0.0, 1.0));
        self
    }
}

/// Bir seri, seviye ya da düğümde miras alınabilen Treemap görsel kanalları.
#[derive(Clone, PartialEq, Debug)]
pub struct AğaçHaritasıGörseli {
    pub boyut: Option<AğaçHaritasıGörselBoyutu>,
    pub en_az: Option<f64>,
    pub en_çok: Option<f64>,
    /// `None`: üst modelden miras; `Some([])`: ECharts `'none'`.
    pub renkler: Option<Vec<Renk>>,
    /// `None`: miras/kapalı; çift `colorAlpha` aralığıdır.
    pub alfa_aralığı: Option<(f32, f32)>,
    pub doygunluk_aralığı: Option<(f32, f32)>,
    pub eşleme: Option<AğaçHaritasıRenkEşlemesi>,
    pub görünür_en_az: Option<f32>,
    pub çocuk_görünür_en_az: Option<f32>,
}

impl Default for AğaçHaritasıGörseli {
    fn default() -> Self {
        Self {
            boyut: None,
            en_az: None,
            en_çok: None,
            renkler: None,
            alfa_aralığı: None,
            doygunluk_aralığı: None,
            eşleme: None,
            görünür_en_az: None,
            çocuk_görünür_en_az: None,
        }
    }
}

impl AğaçHaritasıGörseli {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// ECharts `TreemapSeries.defaultOption` görsel başlangıcı.
    pub fn seri_varsayılanı() -> Self {
        Self {
            boyut: Some(AğaçHaritasıGörselBoyutu::Sıra(0)),
            eşleme: Some(AğaçHaritasıRenkEşlemesi::Sıra),
            görünür_en_az: Some(10.0),
            ..Self::default()
        }
    }

    pub fn boyut(mut self, sıra: usize) -> Self {
        self.boyut = Some(AğaçHaritasıGörselBoyutu::Sıra(sıra));
        self
    }

    pub fn boyut_adı(mut self, ad: impl Into<String>) -> Self {
        self.boyut = Some(AğaçHaritasıGörselBoyutu::Ad(ad.into()));
        self
    }

    pub fn aralık(mut self, en_az: f64, en_çok: f64) -> Self {
        self.en_az = en_az.is_finite().then_some(en_az);
        self.en_çok = en_çok.is_finite().then_some(en_çok);
        self
    }

    pub fn renkler(mut self, renkler: impl IntoIterator<Item = impl Into<Renk>>) -> Self {
        self.renkler = Some(renkler.into_iter().map(Into::into).collect());
        self
    }

    pub fn renk_yok(mut self) -> Self {
        self.renkler = Some(Vec::new());
        self
    }

    pub fn alfa_aralığı(mut self, en_az: f32, en_çok: f32) -> Self {
        self.alfa_aralığı = Some((en_az.clamp(0.0, 1.0), en_çok.clamp(0.0, 1.0)));
        self
    }

    pub fn doygunluk_aralığı(mut self, en_az: f32, en_çok: f32) -> Self {
        self.doygunluk_aralığı = Some((en_az.clamp(0.0, 1.0), en_çok.clamp(0.0, 1.0)));
        self
    }

    pub fn eşleme(mut self, eşleme: AğaçHaritasıRenkEşlemesi) -> Self {
        self.eşleme = Some(eşleme);
        self
    }

    pub fn görünür_en_az(mut self, alan: f32) -> Self {
        self.görünür_en_az = Some(alan.max(0.0));
        self
    }

    pub fn görünür_eşiği_kapalı(mut self) -> Self {
        // `None` bir seviye/düğüm yamasında üst modelden kalıtım demektir;
        // eşiği gerçekten kapatan resmî değer `visibleMin: 0`dır.
        self.görünür_en_az = Some(0.0);
        self
    }

    pub fn çocuk_görünür_en_az(mut self, alan: f32) -> Self {
        self.çocuk_görünür_en_az = Some(alan.max(0.0));
        self
    }
}

/// `emphasis` / `blur` / `select` Treemap yaması.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AğaçHaritasıDurumu {
    pub öğe_stili: Option<AğaçHaritasıÖğeStili>,
    pub etiket: Option<EtiketYaması>,
    pub üst_etiket: Option<EtiketYaması>,
    pub odak: AğaçVurguOdağı,
}

impl AğaçHaritasıDurumu {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn öğe_stili(mut self, stil: AğaçHaritasıÖğeStili) -> Self {
        self.öğe_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: EtiketYaması) -> Self {
        self.etiket = Some(etiket);
        self
    }

    pub fn üst_etiket(mut self, etiket: EtiketYaması) -> Self {
        self.üst_etiket = Some(etiket);
        self
    }

    pub fn odak(mut self, odak: AğaçVurguOdağı) -> Self {
        self.odak = odak;
        self
    }
}

/// `series-treemap.levels[i]` miras katmanı.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AğaçHaritasıSeviyesi {
    pub görsel: Option<AğaçHaritasıGörseli>,
    pub öğe_stili: Option<AğaçHaritasıÖğeStili>,
    pub etiket: Option<EtiketYaması>,
    pub üst_etiket: Option<EtiketYaması>,
    pub vurgu: AğaçHaritasıDurumu,
    pub bulanık: AğaçHaritasıDurumu,
    pub seçili: AğaçHaritasıDurumu,
}

impl AğaçHaritasıSeviyesi {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn görsel(mut self, görsel: AğaçHaritasıGörseli) -> Self {
        self.görsel = Some(görsel);
        self
    }

    pub fn öğe_stili(mut self, stil: AğaçHaritasıÖğeStili) -> Self {
        self.öğe_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: EtiketYaması) -> Self {
        self.etiket = Some(etiket);
        self
    }

    pub fn üst_etiket(mut self, etiket: EtiketYaması) -> Self {
        self.üst_etiket = Some(etiket);
        self
    }

    pub fn vurgu(mut self, durum: AğaçHaritasıDurumu) -> Self {
        self.vurgu = durum;
        self
    }

    pub fn bulanık(mut self, durum: AğaçHaritasıDurumu) -> Self {
        self.bulanık = durum;
        self
    }

    pub fn seçili(mut self, durum: AğaçHaritasıDurumu) -> Self {
        self.seçili = durum;
        self
    }
}

/// Treemap kırıntı çubuğu (`breadcrumb`). Konumlar ECharts box-layout
/// kurallarındaki gibi sol/sağ ve üst/alt çiftlerinden çözülür.
#[derive(Clone, PartialEq, Debug)]
pub struct AğaçHaritasıKırıntısı {
    pub göster: bool,
    pub sol: Option<Uzunluk>,
    pub sağ: Option<Uzunluk>,
    pub üst: Option<Uzunluk>,
    pub alt: Option<Uzunluk>,
    pub yükseklik: f32,
    pub boş_öğe_genişliği: f32,
    pub öğe_stili: ÖğeStili,
    pub yazı: YazıStili,
    pub vurgu_devre_dışı: bool,
    pub vurgu_öğe_stili: Option<ÖğeStili>,
}

impl Default for AğaçHaritasıKırıntısı {
    fn default() -> Self {
        Self {
            göster: true,
            sol: Some(Uzunluk::Yüzde(50.0)),
            sağ: None,
            üst: None,
            alt: Some(Uzunluk::Piksel(16.0)),
            yükseklik: 22.0,
            boş_öğe_genişliği: 25.0,
            // ECharts v6 tokenları: `backgroundShade` / `secondary`.
            öğe_stili: ÖğeStili::yeni().renk(Renk::onaltılık(0xe8ebf0)),
            yazı: YazıStili::yeni().renk(Renk::onaltılık(0x54555a)),
            vurgu_devre_dışı: false,
            vurgu_öğe_stili: Some(ÖğeStili::yeni().renk(Renk::BEYAZ)),
        }
    }
}

impl AğaçHaritasıKırıntısı {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = göster;
        self
    }

    pub fn sol(mut self, sol: impl Into<Uzunluk>) -> Self {
        self.sol = Some(sol.into());
        self.sağ = None;
        self
    }

    pub fn sağ(mut self, sağ: impl Into<Uzunluk>) -> Self {
        self.sağ = Some(sağ.into());
        self.sol = None;
        self
    }

    pub fn üst(mut self, üst: impl Into<Uzunluk>) -> Self {
        self.üst = Some(üst.into());
        self.alt = None;
        self
    }

    pub fn alt(mut self, alt: impl Into<Uzunluk>) -> Self {
        self.alt = Some(alt.into());
        self.üst = None;
        self
    }

    pub fn yükseklik(mut self, yükseklik: f32) -> Self {
        self.yükseklik = yükseklik.max(0.0);
        self
    }

    pub fn boş_öğe_genişliği(mut self, genişlik: f32) -> Self {
        self.boş_öğe_genişliği = genişlik.max(0.0);
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = stil;
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = yazı;
        self
    }

    pub fn vurgu_devre_dışı(mut self, devre_dışı: bool) -> Self {
        self.vurgu_devre_dışı = devre_dışı;
        self
    }

    pub fn vurgu_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.vurgu_öğe_stili = Some(stil);
        self
    }
}

/// Hiyerarşideki tek düğüm.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AğaçDüğümü {
    /// Kararlı veri kimliği (`data[i].id`); diff sırasında add/update/remove
    /// eşlemesinin ad değişikliğinden bağımsız kalmasını sağlar.
    pub kimlik: Option<String>,
    pub ad: String,
    /// Yaprak değeri; dallarda `None` ise çocuk toplamı kullanılır.
    pub değer: Option<f64>,
    /// Treemap/Sunburst çok boyutlu `value` dizisi. İlk eleman alan
    /// değeridir; `None` JSON `null` değerini korur.
    pub değerler: Vec<Option<f64>>,
    pub çocuklar: Vec<AğaçDüğümü>,
    /// Açık renk; verilmezse üst düzey paletten, alt düzeyler üstten türetilir.
    pub renk: Option<Renk>,
    /// Düğümün ilk Tree görünümünde kapalı olup olmadığı. `None`, serinin
    /// `ilk_ağaç_derinliği` kararını kullanır.
    pub daraltılmış: Option<bool>,
    pub kategori: Option<usize>,
    pub bağlantı: Option<String>,
    pub hedef: Option<String>,
    pub sembol: Option<Sembol>,
    pub sembol_boyutu: Option<f32>,
    pub sembol_döndürme: Option<f32>,
    pub sembol_kayması: Option<(Uzunluk, Uzunluk)>,
    pub sembol_oranını_koru: Option<bool>,
    pub öğe_stili: Option<ÖğeStili>,
    /// Tree'de bu düğümü üstüne bağlayan kenarın stili.
    pub çizgi_stili: Option<ÇizgiStili>,
    pub etiket: Option<EtiketYaması>,
    pub vurgu_öğe_stili: Option<ÖğeStili>,
    pub vurgu_çizgi_stili: Option<ÇizgiStili>,
    pub vurgu_etiketi: Option<EtiketYaması>,
    pub bulanık_öğe_stili: Option<ÖğeStili>,
    pub bulanık_çizgi_stili: Option<ÇizgiStili>,
    pub bulanık_etiketi: Option<EtiketYaması>,
    pub seçili_öğe_stili: Option<ÖğeStili>,
    pub seçili_çizgi_stili: Option<ÇizgiStili>,
    pub seçili_etiketi: Option<EtiketYaması>,
    /// Treemap düğümüne özgü miras katmanı.
    pub ağaç_haritası_görseli: Option<AğaçHaritasıGörseli>,
    pub ağaç_haritası_öğe_stili: Option<AğaçHaritasıÖğeStili>,
    pub ağaç_haritası_üst_etiketi: Option<EtiketYaması>,
    pub ağaç_haritası_vurgusu: Option<AğaçHaritasıDurumu>,
    pub ağaç_haritası_bulanıklığı: Option<AğaçHaritasıDurumu>,
    pub ağaç_haritası_seçilisi: Option<AğaçHaritasıDurumu>,
    /// ECharts `cursor`; GPUI katmanı desteklenen CSS adlarını yerel imlece
    /// dönüştürür, bilinmeyen değerler normal imlece düşer.
    pub imleç: Option<String>,
}

impl AğaçDüğümü {
    /// Yaprak düğüm.
    pub fn yaprak(ad: impl Into<String>, değer: f64) -> Self {
        AğaçDüğümü {
            ad: ad.into(),
            değer: Some(değer),
            ..Default::default()
        }
    }

    /// Dal düğümü (değeri çocuk toplamından türeyen).
    pub fn dal(ad: impl Into<String>, çocuklar: Vec<AğaçDüğümü>) -> Self {
        AğaçDüğümü {
            ad: ad.into(),
            çocuklar,
            ..Default::default()
        }
    }

    pub fn renk(mut self, renk: impl Into<Renk>) -> Self {
        self.renk = Some(renk.into());
        self
    }

    pub fn kimlik(mut self, kimlik: impl Into<String>) -> Self {
        self.kimlik = Some(kimlik.into());
        self
    }

    pub fn daraltılmış(mut self, daraltılmış: bool) -> Self {
        self.daraltılmış = Some(daraltılmış);
        self
    }

    pub fn kategori(mut self, kategori: usize) -> Self {
        self.kategori = Some(kategori);
        self
    }

    pub fn bağlantı(mut self, bağlantı: impl Into<String>) -> Self {
        self.bağlantı = Some(bağlantı.into());
        self
    }

    pub fn hedef(mut self, hedef: impl Into<String>) -> Self {
        self.hedef = Some(hedef.into());
        self
    }

    pub fn sembol(mut self, sembol: Sembol) -> Self {
        self.sembol = Some(sembol);
        self
    }

    pub fn sembol_boyutu(mut self, boyut: f32) -> Self {
        self.sembol_boyutu = Some(boyut.max(0.0));
        self
    }

    pub fn sembol_döndürme(mut self, derece: f32) -> Self {
        self.sembol_döndürme = derece.is_finite().then_some(derece);
        self
    }

    pub fn sembol_kayması(mut self, x: impl Into<Uzunluk>, y: impl Into<Uzunluk>) -> Self {
        self.sembol_kayması = Some((x.into(), y.into()));
        self
    }

    pub fn sembol_oranını_koru(mut self, koru: bool) -> Self {
        self.sembol_oranını_koru = Some(koru);
        self
    }

    pub fn öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.öğe_stili = Some(stil);
        self
    }

    pub fn çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.çizgi_stili = Some(stil);
        self
    }

    pub fn etiket(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.etiket = Some(etiket.into());
        self
    }

    pub fn vurgu_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.vurgu_öğe_stili = Some(stil);
        self
    }

    pub fn vurgu_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.vurgu_çizgi_stili = Some(stil);
        self
    }

    pub fn vurgu_etiketi(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.vurgu_etiketi = Some(etiket.into());
        self
    }

    pub fn bulanık_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.bulanık_öğe_stili = Some(stil);
        self
    }

    pub fn bulanık_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.bulanık_çizgi_stili = Some(stil);
        self
    }

    pub fn bulanık_etiketi(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.bulanık_etiketi = Some(etiket.into());
        self
    }

    pub fn seçili_öğe_stili(mut self, stil: ÖğeStili) -> Self {
        self.seçili_öğe_stili = Some(stil);
        self
    }

    pub fn seçili_çizgi_stili(mut self, stil: ÇizgiStili) -> Self {
        self.seçili_çizgi_stili = Some(stil);
        self
    }

    pub fn seçili_etiketi(mut self, etiket: impl Into<EtiketYaması>) -> Self {
        self.seçili_etiketi = Some(etiket.into());
        self
    }

    pub fn değerli(mut self, değer: f64) -> Self {
        self.değer = Some(değer);
        if self.değerler.is_empty() {
            self.değerler.push(Some(değer));
        } else {
            self.değerler[0] = Some(değer);
        }
        self
    }

    /// Çok boyutlu `value` dizisini kurar. İlk eleman hücre alanını belirler.
    pub fn çoklu_değerler(mut self, değerler: impl IntoIterator<Item = Option<f64>>) -> Self {
        self.değerler = değerler.into_iter().collect();
        self.değer = self.değerler.first().copied().flatten();
        self
    }

    pub fn ağaç_haritası_görseli(mut self, görsel: AğaçHaritasıGörseli) -> Self {
        self.ağaç_haritası_görseli = Some(görsel);
        self
    }

    pub fn ağaç_haritası_öğe_stili(mut self, stil: AğaçHaritasıÖğeStili) -> Self {
        self.ağaç_haritası_öğe_stili = Some(stil);
        self
    }

    pub fn ağaç_haritası_üst_etiketi(mut self, etiket: EtiketYaması) -> Self {
        self.ağaç_haritası_üst_etiketi = Some(etiket);
        self
    }

    pub fn ağaç_haritası_vurgusu(mut self, durum: AğaçHaritasıDurumu) -> Self {
        self.ağaç_haritası_vurgusu = Some(durum);
        self
    }

    pub fn ağaç_haritası_bulanıklığı(mut self, durum: AğaçHaritasıDurumu) -> Self {
        self.ağaç_haritası_bulanıklığı = Some(durum);
        self
    }

    pub fn ağaç_haritası_seçilisi(mut self, durum: AğaçHaritasıDurumu) -> Self {
        self.ağaç_haritası_seçilisi = Some(durum);
        self
    }

    pub fn imleç(mut self, imleç: impl Into<String>) -> Self {
        self.imleç = Some(imleç.into());
        self
    }

    /// Etkin değer: verilmişse kendisi, yoksa çocukların toplamı.
    pub fn etkin_değer(&self) -> f64 {
        match self
            .değer
            .or_else(|| self.değerler.first().copied().flatten())
        {
            Some(d) if d.is_finite() => d,
            _ => self.çocuklar.iter().map(|ç| ç.etkin_değer()).sum(),
        }
    }

    /// Treemap görsel eşlemesinin istediği boyutu döndürür. Adlandırılmış
    /// boyutlar için ECharts'ın ortak `value` eş adı desteklenir.
    pub fn görsel_değer(&self, boyut: &AğaçHaritasıGörselBoyutu) -> Option<f64> {
        match boyut {
            AğaçHaritasıGörselBoyutu::Sıra(sıra) => self
                .değerler
                .get(*sıra)
                .copied()
                .flatten()
                .or_else(|| (*sıra == 0).then(|| self.etkin_değer())),
            AğaçHaritasıGörselBoyutu::Ad(ad) if ad == "value" => Some(self.etkin_değer()),
            AğaçHaritasıGörselBoyutu::Ad(_) => None,
        }
        .filter(|değer| değer.is_finite())
    }

    pub fn yaprak_mı(&self) -> bool {
        self.çocuklar.is_empty()
    }
}

/// Ad zincirini (kök yolu) izleyerek etkin kök listesini bulur — ağaç
/// haritası inme (drill-down) ve güneş patlaması odak gezinmesi için.
/// Ad bulunamazsa ya da bulunan düğüm yapraksa iniş orada durur.
/// Dönen ikinci değer: gerçekten inilen adım sayısıdır.
pub fn yolu_çöz<'a>(
    kökler: &'a [AğaçDüğümü], yol: &[String]
) -> (&'a [AğaçDüğümü], usize) {
    let mut etkin = kökler;
    let mut derinlik = 0usize;
    for ad in yol {
        let Some(düğüm) = etkin.iter().find(|d| &d.ad == ad) else {
            break;
        };
        if düğüm.çocuklar.is_empty() {
            break;
        }
        etkin = &düğüm.çocuklar;
        derinlik = derinlik.saturating_add(1);
    }
    (etkin, derinlik)
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::model::stil::EtiketYaması;

    #[test]
    fn tree_dugumu_resmi_veri_sembol_ve_durum_alanlarini_korur() {
        let düğüm = AğaçDüğümü::dal("root", vec![AğaçDüğümü::yaprak("leaf", 3.0)])
            .kimlik("node-0")
            .daraltılmış(true)
            .kategori(2)
            .bağlantı("https://example.invalid")
            .hedef("_blank")
            .sembol(Sembol::Kare)
            .sembol_boyutu(18.0)
            .sembol_döndürme(30.0)
            .sembol_kayması("25%", -2)
            .sembol_oranını_koru(true)
            .öğe_stili(ÖğeStili::yeni().renk("#123456"))
            .çizgi_stili(ÇizgiStili::yeni().kalınlık(3.0))
            .etiket(EtiketYaması::yeni().göster(true))
            .vurgu_etiketi(EtiketYaması::yeni().göster(false))
            .bulanık_öğe_stili(ÖğeStili::yeni().opaklık(0.2))
            .seçili_çizgi_stili(ÇizgiStili::yeni().kalınlık(5.0));

        assert_eq!(düğüm.kimlik.as_deref(), Some("node-0"));
        assert_eq!(düğüm.daraltılmış, Some(true));
        assert_eq!(düğüm.kategori, Some(2));
        assert_eq!(düğüm.bağlantı.as_deref(), Some("https://example.invalid"));
        assert_eq!(düğüm.hedef.as_deref(), Some("_blank"));
        assert_eq!(düğüm.sembol, Some(Sembol::Kare));
        assert_eq!(düğüm.sembol_boyutu, Some(18.0));
        assert_eq!(düğüm.sembol_döndürme, Some(30.0));
        assert_eq!(
            düğüm.sembol_kayması,
            Some((Uzunluk::Yüzde(25.0), Uzunluk::Piksel(-2.0)))
        );
        assert_eq!(düğüm.sembol_oranını_koru, Some(true));
        assert_eq!(düğüm.çocuklar.len(), 1);
        assert_eq!(düğüm.etkin_değer(), 3.0);
        assert!(düğüm.öğe_stili.is_some());
        assert!(düğüm.çizgi_stili.is_some());
        assert!(düğüm.etiket.is_some());
        assert!(düğüm.vurgu_etiketi.is_some());
        assert!(düğüm.bulanık_öğe_stili.is_some());
        assert!(düğüm.seçili_çizgi_stili.is_some());
    }
}
