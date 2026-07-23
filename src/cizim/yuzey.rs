//! Çizim yüzeyi soyutlaması — zrender'ın `Painter` arayüzünün karşılığı.
//!
//! Tüm seri ve bileşen çizicileri bu trait üzerinden çizer; gpui gerçeklemesi
//! [`crate::cizim::cizici::Çizici`], test/golden gerçeklemesi
//! [`crate::cizim::kayit::KayıtYüzeyi`]dir. İleride SVG/PNG dışa aktarımı da
//! bu trait'in yeni bir gerçeklemesi olarak eklenecektir (Faz 8).

use crate::cizim::donusum::AfinMatris;
use crate::koordinat::Dikdörtgen;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

/// Yazı satır yüksekliğinin yazı boyutuna oranı.
pub const SATIR_ORANI: f32 = 1.4;

/// 1 piksellik çizgileri fiziksel piksel ızgarasına oturtur (yarım piksel
/// hizalama); eksen ve bölme çizgilerinin bulanıklaşmasını önler.
pub fn keskin(v: f32) -> f32 {
    // zrender `subPixelOptimize(position, 1, true)`: önce en yakın yarım
    // piksele yuvarlar, sonuç tam piksele düşerse pozitif yöndeki yarım
    // piksele taşır. Sadece `floor + .5` kullanmak 0.5'i aşan koordinatları
    // bir piksel erken çiziyordu.
    let iki_kat = (v * 2.0).round();
    if ((iki_kat as i64 + 1).rem_euclid(2)) == 0 {
        iki_kat / 2.0
    } else {
        (iki_kat + 1.0) / 2.0
    }
}

/// Canvas `setLineDash` normalleştirmesi: negatif/sonlu olmayan ya da
/// toplamı sıfır diziler düz çizgiye döner; tek uzunluktaki desen,
/// Canvas standardındaki gibi kendisiyle yinelenerek çiftlenir.
pub(crate) fn çizgi_deseni_normalleştir(desen: &[f32]) -> Vec<f32> {
    if desen.is_empty()
        || desen.iter().any(|değer| !değer.is_finite() || *değer < 0.0)
        || desen.iter().all(|değer| *değer == 0.0)
    {
        return Vec::new();
    }
    let mut sonuç = desen.to_vec();
    if sonuç.len() % 2 == 1 {
        sonuç.extend_from_slice(desen);
    }
    sonuç
}

/// Yatay yazı hizası.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum YatayHiza {
    #[default]
    Sol,
    Orta,
    Sağ,
}

/// Dikey yazı hizası.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DikeyHiza {
    Üst,
    #[default]
    Orta,
    Alt,
}

/// Yol komutu — koordinatlar yüzey yerelidir.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum YolKomutu {
    Taşı((f32, f32)),
    Çiz((f32, f32)),
    /// Kübik Bezier: kontrol noktaları `k1`, `k2`, bitiş `uç`.
    Kübik {
        k1: (f32, f32),
        k2: (f32, f32),
        uç: (f32, f32),
    },
    /// Yay: `yarıçap`, `büyük_yay`, `süpürme` (SVG bayrakları) ile `uç`a.
    Yay {
        yarıçap: f32,
        büyük_yay: bool,
        süpürme: bool,
        uç: (f32, f32),
    },
    Kapat,
}

/// Komut listesinden oluşan yol (zrender `PathProxy` karşılığı).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Yol {
    pub komutlar: Vec<YolKomutu>,
}

impl Yol {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn taşı(&mut self, n: (f32, f32)) {
        self.komutlar.push(YolKomutu::Taşı(n));
    }

    pub fn çiz(&mut self, n: (f32, f32)) {
        self.komutlar.push(YolKomutu::Çiz(n));
    }

    pub fn kübik(&mut self, k1: (f32, f32), k2: (f32, f32), uç: (f32, f32)) {
        self.komutlar.push(YolKomutu::Kübik { k1, k2, uç });
    }

    pub fn yay(&mut self, yarıçap: f32, büyük_yay: bool, süpürme: bool, uç: (f32, f32)) {
        self.komutlar.push(YolKomutu::Yay {
            yarıçap,
            büyük_yay,
            süpürme,
            uç,
        });
    }

    pub fn kapat(&mut self) {
        self.komutlar.push(YolKomutu::Kapat);
    }

    pub fn boş_mu(&self) -> bool {
        self.komutlar.len() < 2
    }

    /// Yolun kaba sınır kutusu (kontrol noktaları dahil; gradyan bantlama
    /// için yeterli hassasiyette).
    pub fn sınır_kutusu(&self) -> Option<Dikdörtgen> {
        let mut en_küçük = (f32::INFINITY, f32::INFINITY);
        let mut en_büyük = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut genişlet = |n: (f32, f32)| {
            en_küçük.0 = en_küçük.0.min(n.0);
            en_küçük.1 = en_küçük.1.min(n.1);
            en_büyük.0 = en_büyük.0.max(n.0);
            en_büyük.1 = en_büyük.1.max(n.1);
        };
        for komut in &self.komutlar {
            match *komut {
                YolKomutu::Taşı(n) | YolKomutu::Çiz(n) => genişlet(n),
                YolKomutu::Kübik { k1, k2, uç } => {
                    genişlet(k1);
                    genişlet(k2);
                    genişlet(uç);
                }
                YolKomutu::Yay { uç, .. } => genişlet(uç),
                YolKomutu::Kapat => {}
            }
        }
        if en_küçük.0.is_finite() {
            Some(Dikdörtgen::yeni(
                en_küçük.0,
                en_küçük.1,
                en_büyük.0 - en_küçük.0,
                en_büyük.1 - en_küçük.1,
            ))
        } else {
            None
        }
    }
}

/// Çizim yüzeyi: temel boyama ilkelleri + bunlardan türeyen ortak
/// yardımcılar. Nesne-güvenlidir (`&mut dyn ÇizimYüzeyi`).
pub trait ÇizimYüzeyi {
    fn genişlik(&self) -> f32;
    fn yükseklik(&self) -> f32;

    /// Yolu dolgu ile boyar.
    fn yol_doldur(&mut self, yol: &Yol, dolgu: &Dolgu);

    /// Yolu verilen kalınlık ve türde çizgiler.
    fn yol_çiz(&mut self, yol: &Yol, kalınlık: f32, renk: Renk, tür: ÇizgiTürü);

    /// Canvas `setLineDash` / SVG `stroke-dasharray` karşılığı. Değerler
    /// çizgi kalınlığından bağımsız piksel uzunluklarıdır; boş/geçersiz dizi
    /// düz çizgiye düşer. Özel seri `renderItem.style.lineDash` bu yüzeyi
    /// kullanır.
    fn yol_çizgi_deseni(
        &mut self,
        yol: &Yol,
        kalınlık: f32,
        renk: Renk,
        desen: &[f32],
        kayma: f32,
    ) {
        let tür = if çizgi_deseni_normalleştir(desen).is_empty() {
            ÇizgiTürü::Düz
        } else {
            ÇizgiTürü::Kesikli
        };
        let _ = kayma;
        self.yol_çiz(yol, kalınlık, renk, tür);
    }

    /// Yolu düz renk yerine desen/gradyan boyasıyla çizgiler. Basit yüzeyler
    /// temsilî renge düşebilir; yerleşik GPUI, PNG ve SVG yüzeyleri doğrusal
    /// gradyan vuruşunu korur.
    fn yol_dolgulu_çiz(&mut self, yol: &Yol, kalınlık: f32, dolgu: &Dolgu, tür: ÇizgiTürü) {
        self.yol_çiz(yol, kalınlık, dolgu.temsilî(), tür);
    }

    /// Keyfi doldurulmuş yolun Canvas/zrender biçimli dış gölgesi. Yüzey
    /// gerçek bulanıklık sunmuyorsa güvenli öntanımlı davranış gölgeyi
    /// atlar; raster ve SVG yüzeyleri bu ilkelin tam karşılığını sağlar.
    fn yol_gölgesi(&mut self, _yol: &Yol, _renk: Renk, _bulanıklık: f32, _kayma: (f32, f32)) {}

    /// Açık bir yol vuruşunun Canvas/zrender gölgesi. Dolgu gölgesinden
    /// ayrıdır: açık polylineler kapanıp bir alana dönüşmemelidir.
    #[allow(clippy::too_many_arguments)]
    fn yol_çizgi_gölgesi(
        &mut self,
        _yol: &Yol,
        _kalınlık: f32,
        _tür: ÇizgiTürü,
        _renk: Renk,
        _bulanıklık: f32,
        _kayma: (f32, f32),
    ) {
    }

    /// Dikdörtgen boyar; `yarıçap` köşe sırası `[sol üst, sağ üst, sağ alt,
    /// sol alt]`, `kenarlık` `(kalınlık, renk)` çiftidir.
    fn dikdörtgen(
        &mut self,
        d: Dikdörtgen,
        dolgu: &Dolgu,
        yarıçap: [f32; 4],
        kenarlık: Option<(f32, Renk)>,
    );

    /// ECharts `LargeSymbolPath.afterBrush` karşılığı: iç içe olmayan
    /// `[x0, y0, x1, y1, ...]` ekran koordinatlarını küçük `fillRect`
    /// sembolleri olarak topluca boyar. Yüzeyler bu çağrıyı hızlandırabilir;
    /// güvenli öntanımlı yol aynı semantiği tek tek dikdörtgenlerle korur.
    fn büyük_saçılım_noktaları(&mut self, konumlar: &[f32], boyut: f32, dolgu: &Dolgu) {
        if boyut <= 0.0 {
            return;
        }
        let yarı = boyut / 2.0;
        for çift in konumlar.chunks_exact(2) {
            let [x, y] = çift else { continue };
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            self.dikdörtgen(
                Dikdörtgen::yeni(*x - yarı, *y - yarı, boyut, boyut),
                dolgu,
                [0.0; 4],
                None,
            );
        }
    }

    /// Gölge boyar (ipucu penceresi vb. için).
    fn gölge(&mut self, d: Dikdörtgen, yarıçap: f32, renk: Renk, bulanıklık: f32);

    /// Çizimi verilen dikdörtgene kırparak `işlev`i çalıştırır.
    fn kırpılı(&mut self, d: Dikdörtgen, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi));

    /// Çizimi keyfi bir yolun içine kırpar. Yüzey özel bir yol maskesi
    /// sunmuyorsa güvenli öntanımlı davranış yolun sınır kutusudur.
    fn yol_kırpılı(&mut self, yol: &Yol, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi)) {
        if let Some(kutu) = yol.sınır_kutusu() {
            self.kırpılı(kutu, işlev);
        }
    }

    /// Tek satır yazı boyar; `konum`, hizaya göre çapa noktasıdır.
    /// Çizilen `(genişlik, yükseklik)` döner.
    #[allow(clippy::too_many_arguments)]
    fn yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
    ) -> (f32, f32);

    /// Açık CSS yazı tipi ailesiyle metin boyar (`fontFamily`). Aile
    /// seçimini desteklemeyen yüzeyler öntanımlı yazı tipine düşer.
    #[allow(clippy::too_many_arguments)]
    fn aileli_yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
        aile: &str,
    ) -> (f32, f32) {
        let _ = aile;
        self.yazı(metin, konum, yatay, dikey, boyut, renk, kalın)
    }

    /// Yerel metni tam affine dönüşümle boyar. Renderer bunu doğrudan
    /// desteklemiyorsa konum ve ortalama ölçek yine doğru uygulanır.
    #[allow(clippy::too_many_arguments)]
    fn dönüşümlü_yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
        dönüşüm: AfinMatris,
    ) -> (f32, f32) {
        let konum = dönüşüm.noktayı_dönüştür(konum);
        let ölçek = ((dönüşüm.x_ölçeği() + dönüşüm.y_ölçeği()) / 2.0).max(0.0);
        self.yazı(metin, konum, yatay, dikey, boyut * ölçek, renk, kalın)
    }

    /// Dönüştürülmüş glif maskesinin Canvas `textShadow*` gölgesini boyar.
    /// Gölge desteği sunmayan yüzeyler metnin kendisini etkilemeden atlar.
    #[allow(clippy::too_many_arguments)]
    fn dönüşümlü_yazı_gölgesi(
        &mut self,
        _metin: &str,
        _konum: (f32, f32),
        _yatay: YatayHiza,
        _dikey: DikeyHiza,
        _boyut: f32,
        _kalın: bool,
        _renk: Renk,
        _bulanıklık: f32,
        _kayma: (f32, f32),
        _dönüşüm: AfinMatris,
    ) {
    }

    /// Açık yazı tipi ailesini tam affine dönüşümle birlikte korur.
    #[allow(clippy::too_many_arguments)]
    fn dönüşümlü_aileli_yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
        aile: &str,
        dönüşüm: AfinMatris,
    ) -> (f32, f32) {
        let konum = dönüşüm.noktayı_dönüştür(konum);
        let ölçek = ((dönüşüm.x_ölçeği() + dönüşüm.y_ölçeği()) / 2.0).max(0.0);
        self.aileli_yazı(metin, konum, yatay, dikey, boyut * ölçek, renk, kalın, aile)
    }

    /// Yerel metni önce kontur, ardından dolgu ile affine dönüşüm altında
    /// boyar (`textBorderWidth` + `strokeFirst`). Yüzey özel glif vuruşu
    /// sunmuyorsa konturu sekiz yönlü bir piksel örneklemesiyle yaklaştırır.
    #[allow(clippy::too_many_arguments)]
    fn dönüşümlü_konturlu_yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
        kontur_rengi: Renk,
        kontur_kalınlığı: f32,
        dönüşüm: AfinMatris,
    ) -> (f32, f32) {
        let yarıçap = (kontur_kalınlığı / 2.0).max(0.0);
        let köşegen = yarıçap * std::f32::consts::FRAC_1_SQRT_2;
        for (x, y) in [
            (-yarıçap, 0.0),
            (yarıçap, 0.0),
            (0.0, -yarıçap),
            (0.0, yarıçap),
            (-köşegen, -köşegen),
            (köşegen, -köşegen),
            (-köşegen, köşegen),
            (köşegen, köşegen),
        ] {
            self.dönüşümlü_yazı(
                metin,
                konum,
                yatay,
                dikey,
                boyut,
                kontur_rengi,
                kalın,
                AfinMatris::ötele(x, y).çarp(dönüşüm),
            );
        }
        self.dönüşümlü_yazı(metin, konum, yatay, dikey, boyut, renk, kalın, dönüşüm)
    }

    /// Yazının kaplayacağı `(genişlik, yükseklik)` boyutunu ölçer.
    fn yazı_ölç(&self, metin: &str, boyut: f32) -> (f32, f32);

    /// Açık aileyle yazı ölçer; aile çözmeyen yüzeylerde genel ölçüme düşer.
    fn aileli_yazı_ölç(&self, metin: &str, boyut: f32, aile: &str) -> (f32, f32) {
        let _ = aile;
        self.yazı_ölç(metin, boyut)
    }

    /// Yazıyı gerçekten çizilecek ağırlıkla ölçer. Basit yüzeyler normal
    /// metriklere düşebilir; yazı tipi şekillendiren yüzeyler kalın yüzün
    /// kendi ilerleme değerlerini kullanır.
    fn stilli_yazı_ölç(&self, metin: &str, boyut: f32, _kalın: bool) -> (f32, f32) {
        self.yazı_ölç(metin, boyut)
    }

    // ------------------------------------------------------------------
    // Ortak yardımcılar (ilkellerden türetilir; her yüzeyde aynı çalışır).
    // ------------------------------------------------------------------

    /// İki nokta arasına çizgi çeker.
    fn çizgi(
        &mut self, a: (f32, f32), b: (f32, f32), kalınlık: f32, renk: Renk, tür: ÇizgiTürü
    ) {
        let mut yol = Yol::yeni();
        yol.taşı(a);
        yol.çiz(b);
        self.yol_çiz(&yol, kalınlık, renk, tür);
    }

    /// Ardışık noktalardan çoklu çizgi çeker.
    fn çoklu_çizgi(
        &mut self,
        noktalar: &[(f32, f32)],
        kalınlık: f32,
        renk: Renk,
        tür: ÇizgiTürü,
    ) {
        let Some(&ilk) = noktalar.first() else { return };
        if noktalar.len() < 2 {
            return;
        }
        let mut yol = Yol::yeni();
        yol.taşı(ilk);
        for n in noktalar.iter().skip(1) {
            yol.çiz(*n);
        }
        self.yol_çiz(&yol, kalınlık, renk, tür);
    }

    /// Daire boyar; istenirse ayrıca kenarlık halkası çizer. Radyal
    /// gradyanlar eşmerkezli halkalarla yaklaşıklanır.
    fn daire(
        &mut self,
        merkez: (f32, f32),
        yarıçap: f32,
        dolgu: Option<&Dolgu>,
        kenarlık: Option<(f32, Renk)>,
    ) {
        if yarıçap <= 0.0 {
            return;
        }
        if let Some(Dolgu::RadyalGradyan { duraklar, .. }) = dolgu {
            radyal_halkalar(self.olarak(), merkez, 0.0, yarıçap, 0.0, TAM_TUR, duraklar);
        } else if let Some(dolgu) = dolgu {
            let yol = daire_yolu(merkez, yarıçap);
            self.yol_doldur(&yol, dolgu);
        }
        if let Some((kalınlık, renk)) = kenarlık
            && kalınlık > 0.0
        {
            let yol = daire_yolu(merkez, yarıçap);
            self.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
        }
    }

    /// Pasta dilimi (halka parçası) boyar. Açılar radyandır ve ekran
    /// koordinatındadır (0 → sağ, pozitif yön saat yönü). Radyal gradyanlar
    /// eşmerkezli halkalarla yaklaşıklanır (durak konumu iç→dış yarıçapa
    /// eşlenir).
    #[allow(clippy::too_many_arguments)]
    fn dilim(
        &mut self,
        merkez: (f32, f32),
        iç_yarıçap: f32,
        dış_yarıçap: f32,
        açı0: f32,
        açı1: f32,
        dolgu: &Dolgu,
        kenarlık: Option<(f32, Renk)>,
    ) {
        if dış_yarıçap <= 0.0 || (açı1 - açı0).abs() < 1e-5 {
            return;
        }
        if let Dolgu::RadyalGradyan { duraklar, .. } = dolgu {
            radyal_halkalar(
                self.olarak(),
                merkez,
                iç_yarıçap,
                dış_yarıçap,
                açı0,
                açı1,
                duraklar,
            );
        } else {
            let yol = dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1);
            self.yol_doldur(&yol, dolgu);
        }
        if let Some((kalınlık, renk)) = kenarlık
            && kalınlık > 0.0
        {
            let yol = dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1);
            self.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
        }
    }

    /// Teğetsel polar sütun için iki ucu yarım daireli halka parçası
    /// (`series.bar.roundCap`, zrender `Sausage`) boyar.
    #[allow(clippy::too_many_arguments)]
    fn yuvarlak_uçlu_dilim(
        &mut self,
        merkez: (f32, f32),
        iç_yarıçap: f32,
        dış_yarıçap: f32,
        açı0: f32,
        açı1: f32,
        dolgu: &Dolgu,
        kenarlık: Option<(f32, Renk)>,
    ) {
        let yol = yuvarlak_uçlu_dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1);
        if yol.boş_mu() {
            return;
        }
        self.yol_doldur(&yol, dolgu);
        if let Some((kalınlık, renk)) = kenarlık
            && kalınlık > 0.0
        {
            self.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
        }
    }

    /// Köşeleri yuvarlatılmış pasta dilimi boyar. Köşe sırası zrender
    /// `Sector.cornerRadius` ile aynıdır: iç-başlangıç, iç-bitiş,
    /// dış-başlangıç, dış-bitiş. Sıfır yarıçaplar düz [`Self::dilim`]
    /// davranışına düşer.
    #[allow(clippy::too_many_arguments)]
    fn yuvarlatılmış_dilim(
        &mut self,
        merkez: (f32, f32),
        iç_yarıçap: f32,
        dış_yarıçap: f32,
        açı0: f32,
        açı1: f32,
        köşe_yarıçapları: [f32; 4],
        dolgu: &Dolgu,
        kenarlık: Option<(f32, Renk)>,
    ) {
        if köşe_yarıçapları.iter().all(|yarıçap| *yarıçap <= 1e-4) {
            self.dilim(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1, dolgu, kenarlık);
            return;
        }
        let yol = yuvarlatılmış_dilim_yolu(
            merkez,
            iç_yarıçap,
            dış_yarıçap,
            açı0,
            açı1,
            köşe_yarıçapları,
        );
        if yol.boş_mu() {
            return;
        }
        // Radyal gradyanların renk durakları mevcut ortak halka
        // yaklaşımıyla korunur; düz/doğrusal dolgular gerçek sektör yolunu
        // kullanır.
        if let Dolgu::RadyalGradyan { duraklar, .. } = dolgu {
            let mut boya = |yüzey: &mut dyn ÇizimYüzeyi| {
                radyal_halkalar(yüzey, merkez, iç_yarıçap, dış_yarıçap, açı0, açı1, duraklar);
            };
            self.yol_kırpılı(&yol, &mut boya);
        } else {
            self.yol_doldur(&yol, dolgu);
        }
        if let Some((kalınlık, renk)) = kenarlık
            && kalınlık > 0.0
        {
            self.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
        }
    }

    /// `self`i trait nesnesi olarak verir (varsayılan yöntemlerin yardımcı
    /// işlevlere geçirilmesi için).
    fn olarak(&mut self) -> &mut dyn ÇizimYüzeyi;
}

const TAM_TUR: f32 = std::f32::consts::TAU * 0.9999;

/// Daire yolu: iki yarım yay.
pub fn daire_yolu(merkez: (f32, f32), yarıçap: f32) -> Yol {
    let (mx, my) = merkez;
    let mut yol = Yol::yeni();
    yol.taşı((mx + yarıçap, my));
    yol.yay(yarıçap, false, true, (mx - yarıçap, my));
    yol.yay(yarıçap, false, true, (mx + yarıçap, my));
    yol.kapat();
    yol
}

/// Dilim (halka parçası) yolu.
pub fn dilim_yolu(
    merkez: (f32, f32),
    iç_yarıçap: f32,
    dış_yarıçap: f32,
    açı0: f32,
    açı1: f32,
) -> Yol {
    // Tam daireye çok yakın dilimler yay uçlarının çakışmaması için kırpılır.
    let tam_tur = std::f32::consts::TAU;
    let açıklık = (açı1 - açı0).clamp(-tam_tur * 0.9999, tam_tur * 0.9999);
    let açı1 = açı0 + açıklık;

    let (mx, my) = merkez;
    let uç = |yarıçap: f32, açı: f32| (mx + yarıçap * açı.cos(), my + yarıçap * açı.sin());
    let büyük = açıklık.abs() > std::f32::consts::PI;
    let süpürme = açıklık > 0.0;

    let mut yol = Yol::yeni();
    if iç_yarıçap > 0.5 {
        yol.taşı(uç(iç_yarıçap, açı0));
        yol.çiz(uç(dış_yarıçap, açı0));
        yol.yay(dış_yarıçap, büyük, süpürme, uç(dış_yarıçap, açı1));
        yol.çiz(uç(iç_yarıçap, açı1));
        yol.yay(iç_yarıçap, büyük, !süpürme, uç(iç_yarıçap, açı0));
        yol.kapat();
    } else {
        yol.taşı(merkez);
        yol.çiz(uç(dış_yarıçap, açı0));
        yol.yay(dış_yarıçap, büyük, süpürme, uç(dış_yarıçap, açı1));
        yol.kapat();
    }
    yol
}

/// zrender `Sausage.buildPath` eşdeğeri: orta yarıçap boyunca ilerleyen
/// halka şeridinin başlangıç ve bitişine yarım daire kapak ekler. Yol
/// bilerek kapatılmaz; Canvas/SVG dolgu açık yolu kapatırken stroke, tam tur
/// durumunda görünmeyen bir radyal dikiş üretmez.
pub fn yuvarlak_uçlu_dilim_yolu(
    merkez: (f32, f32),
    iç_yarıçap: f32,
    dış_yarıçap: f32,
    açı0: f32,
    açı1: f32,
) -> Yol {
    const EPS: f32 = 1e-5;
    let iç = iç_yarıçap.min(dış_yarıçap).max(0.0);
    let dış = iç_yarıçap.max(dış_yarıçap).max(0.0);
    if dış - iç <= EPS || !açı0.is_finite() || !açı1.is_finite() {
        return Yol::yeni();
    }

    let saat_yönünde = açı1 >= açı0;
    let yön = if saat_yönünde { 1.0 } else { -1.0 };
    let tam_tur = std::f32::consts::TAU;
    let özgün_açıklık = (açı1 - açı0).abs();
    let tam_tur_mu = özgün_açıklık >= tam_tur - EPS;
    let başlangıç = if tam_tur_mu {
        açı1 - yön * tam_tur
    } else {
        açı0
    };
    let yarım_kalınlık = (dış - iç) / 2.0;
    let uç = |yarıçap: f32, açı: f32| {
        (
            merkez.0 + yarıçap * açı.cos(),
            merkez.1 + yarıçap * açı.sin(),
        )
    };
    let dış_baş = uç(dış, başlangıç);
    let iç_baş = uç(iç, başlangıç);
    let iç_son = uç(iç, açı1);

    let yay = |yol: &mut Yol, yarıçap: f32, baş: f32, son: f32, süpürme: bool| {
        if yarıçap <= EPS || (son - baş).abs() <= EPS {
            return;
        }
        let yay_açıklığı = (son - baş).abs();
        if yay_açıklığı >= tam_tur - EPS {
            let orta_açı = baş
                + if süpürme {
                    std::f32::consts::PI
                } else {
                    -std::f32::consts::PI
                };
            yol.yay(yarıçap, false, süpürme, uç(yarıçap, orta_açı));
            yol.yay(yarıçap, false, süpürme, uç(yarıçap, son));
        } else {
            yol.yay(
                yarıçap,
                yay_açıklığı > std::f32::consts::PI,
                süpürme,
                uç(yarıçap, son),
            );
        }
    };

    let mut yol = Yol::yeni();
    if tam_tur_mu {
        yol.taşı(dış_baş);
    } else {
        yol.taşı(iç_baş);
        yol.yay(yarım_kalınlık, false, saat_yönünde, dış_baş);
    }
    yay(&mut yol, dış, başlangıç, açı1, saat_yönünde);
    yol.yay(yarım_kalınlık, false, saat_yönünde, iç_son);
    if iç > EPS {
        yay(&mut yol, iç, açı1, başlangıç, !saat_yönünde);
    }
    yol
}

#[derive(Clone, Copy)]
struct KöşeTeğetleri {
    merkez: (f32, f32),
    dış_uç: (f32, f32),
    ana_uç: (f32, f32),
}

fn kesişim(a0: (f32, f32), a1: (f32, f32), b0: (f32, f32), b1: (f32, f32)) -> Option<(f32, f32)> {
    let d10 = (a1.0 - a0.0, a1.1 - a0.1);
    let d32 = (b1.0 - b0.0, b1.1 - b0.1);
    let mut t = d32.1 * d10.0 - d32.0 * d10.1;
    if t * t < 1e-4 {
        return None;
    }
    t = (d32.0 * (a0.1 - b0.1) - d32.1 * (a0.0 - b0.0)) / t;
    Some((a0.0 + t * d10.0, a0.1 + t * d10.1))
}

/// zrender `computeCornerTangents` eşdeğeri. Bütün koordinatlar sektör
/// merkezine göre yereldir.
fn köşe_teğetleri(
    p0: (f32, f32),
    p1: (f32, f32),
    ana_yarıçap: f32,
    köşe_yarıçapı: f32,
    saat_yönünde: bool,
) -> KöşeTeğetleri {
    let d01 = (p0.0 - p1.0, p0.1 - p1.1);
    let uzunluk = (d01.0 * d01.0 + d01.1 * d01.1).sqrt().max(1e-12);
    let lo = (if saat_yönünde {
        köşe_yarıçapı
    } else {
        -köşe_yarıçapı
    }) / uzunluk;
    let kayma = (lo * d01.1, -lo * d01.0);
    let a = (p0.0 + kayma.0, p0.1 + kayma.1);
    let b = (p1.0 + kayma.0, p1.1 + kayma.1);
    let orta = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let d = (b.0 - a.0, b.1 - a.1);
    let d2 = (d.0 * d.0 + d.1 * d.1).max(1e-12);
    let r = ana_yarıçap - köşe_yarıçapı;
    let s = a.0 * b.1 - b.0 * a.1;
    let kök = (r * r * d2 - s * s).max(0.0).sqrt();
    let işaretli_kök = if d.1 < 0.0 { -kök } else { kök };
    let aday0 = (
        (s * d.1 - d.0 * işaretli_kök) / d2,
        (-s * d.0 - d.1 * işaretli_kök) / d2,
    );
    let aday1 = (
        (s * d.1 + d.0 * işaretli_kök) / d2,
        (-s * d.0 + d.1 * işaretli_kök) / d2,
    );
    let uzaklık2 = |n: (f32, f32)| {
        let dx = n.0 - orta.0;
        let dy = n.1 - orta.1;
        dx * dx + dy * dy
    };
    let merkez = if uzaklık2(aday0) > uzaklık2(aday1) {
        aday1
    } else {
        aday0
    };
    let oran = if r.abs() > 1e-12 {
        ana_yarıçap / r - 1.0
    } else {
        0.0
    };
    KöşeTeğetleri {
        merkez,
        dış_uç: (-kayma.0, -kayma.1),
        ana_uç: (merkez.0 * oran, merkez.1 * oran),
    }
}

fn merkezli_yay(
    yol: &mut Yol,
    merkez: (f32, f32),
    yarıçap: f32,
    açı0: f32,
    açı1: f32,
    saat_yönünde: bool,
) {
    if yarıçap <= 1e-6 {
        return;
    }
    let tau = std::f32::consts::TAU;
    let açıklık = if saat_yönünde {
        (açı1 - açı0).rem_euclid(tau)
    } else {
        (açı0 - açı1).rem_euclid(tau)
    };
    let uç = (
        merkez.0 + yarıçap * açı1.cos(),
        merkez.1 + yarıçap * açı1.sin(),
    );
    yol.yay(yarıçap, açıklık > std::f32::consts::PI, saat_yönünde, uç);
}

/// zrender `Sector.buildPath` köşe geometrisinin renderer-bağımsız yolu.
pub fn yuvarlatılmış_dilim_yolu(
    merkez: (f32, f32),
    iç_yarıçap: f32,
    dış_yarıçap: f32,
    açı0: f32,
    açı1: f32,
    köşe_yarıçapları: [f32; 4],
) -> Yol {
    const EPS: f32 = 1e-4;
    let mut dış = dış_yarıçap.max(0.0);
    let mut iç = iç_yarıçap.max(0.0);
    if dış <= EPS && iç <= EPS {
        return Yol::yeni();
    }
    if dış <= EPS {
        dış = iç;
        iç = 0.0;
    }
    if iç > dış {
        std::mem::swap(&mut iç, &mut dış);
    }
    if !açı0.is_finite() || !açı1.is_finite() {
        return Yol::yeni();
    }
    let saat_yönünde = açı1 >= açı0;
    let mut açıklık = (açı1 - açı0).abs();
    let tau = std::f32::consts::TAU;
    let kalan = açıklık % tau;
    if açıklık > tau && kalan > EPS {
        açıklık = kalan;
    }
    if açıklık > tau - EPS {
        return dilim_yolu(merkez, iç, dış, açı0, açı1);
    }

    let uç = |yarıçap: f32, açı: f32| (yarıçap * açı.cos(), yarıçap * açı.sin());
    let dış_baş = uç(dış, açı0);
    let dış_son = uç(dış, açı1);
    let iç_baş = uç(iç, açı0);
    let iç_son = uç(iç, açı1);
    if açıklık <= EPS {
        let mut yol = Yol::yeni();
        yol.taşı((merkez.0 + dış_baş.0, merkez.1 + dış_baş.1));
        yol.kapat();
        return yol;
    }

    let yarım_kalınlık = (dış - iç).abs() / 2.0;
    let mut iç_baş_r = köşe_yarıçapları[0].max(0.0).min(yarım_kalınlık);
    let mut iç_son_r = köşe_yarıçapları[1].max(0.0).min(yarım_kalınlık);
    let mut dış_baş_r = köşe_yarıçapları[2].max(0.0).min(yarım_kalınlık);
    let mut dış_son_r = köşe_yarıçapları[3].max(0.0).min(yarım_kalınlık);
    let dış_en_çok = dış_baş_r.max(dış_son_r);
    let iç_en_çok = iç_baş_r.max(iç_son_r);
    let mut sınırlı_dış = dış_en_çok;
    let mut sınırlı_iç = iç_en_çok;
    if (dış_en_çok > EPS || iç_en_çok > EPS)
        && açıklık < std::f32::consts::PI
        && let Some(kesişim) = kesişim(dış_baş, iç_baş, dış_son, iç_son)
    {
        let v0 = (dış_baş.0 - kesişim.0, dış_baş.1 - kesişim.1);
        let v1 = (dış_son.0 - kesişim.0, dış_son.1 - kesişim.1);
        let payda =
            ((v0.0 * v0.0 + v0.1 * v0.1).sqrt() * (v1.0 * v1.0 + v1.1 * v1.1).sqrt()).max(1e-12);
        let kosinüs = ((v0.0 * v1.0 + v0.1 * v1.1) / payda).clamp(-1.0, 1.0);
        let a = 1.0 / (kosinüs.acos() / 2.0).sin().max(1e-12);
        let b = (kesişim.0 * kesişim.0 + kesişim.1 * kesişim.1).sqrt();
        sınırlı_dış = sınırlı_dış.min((dış - b) / (a + 1.0));
        sınırlı_iç = sınırlı_iç.min((iç - b) / (a - 1.0).max(1e-12));
    }

    let ekle = |n: (f32, f32)| (merkez.0 + n.0, merkez.1 + n.1);
    let mut yol = Yol::yeni();
    if sınırlı_dış > EPS {
        dış_baş_r = dış_baş_r.min(sınırlı_dış);
        dış_son_r = dış_son_r.min(sınırlı_dış);
        let t0 = köşe_teğetleri(iç_baş, dış_baş, dış, dış_baş_r, saat_yönünde);
        let t1 = köşe_teğetleri(dış_son, iç_son, dış, dış_son_r, saat_yönünde);
        yol.taşı(ekle((t0.merkez.0 + t0.dış_uç.0, t0.merkez.1 + t0.dış_uç.1)));
        if sınırlı_dış < dış_en_çok && (dış_baş_r - dış_son_r).abs() <= EPS {
            merkezli_yay(
                &mut yol,
                ekle(t0.merkez),
                sınırlı_dış,
                t0.dış_uç.1.atan2(t0.dış_uç.0),
                t1.dış_uç.1.atan2(t1.dış_uç.0),
                saat_yönünde,
            );
        } else {
            if dış_baş_r > 0.0 {
                merkezli_yay(
                    &mut yol,
                    ekle(t0.merkez),
                    dış_baş_r,
                    t0.dış_uç.1.atan2(t0.dış_uç.0),
                    t0.ana_uç.1.atan2(t0.ana_uç.0),
                    saat_yönünde,
                );
            }
            merkezli_yay(
                &mut yol,
                merkez,
                dış,
                (t0.merkez.1 + t0.ana_uç.1).atan2(t0.merkez.0 + t0.ana_uç.0),
                (t1.merkez.1 + t1.ana_uç.1).atan2(t1.merkez.0 + t1.ana_uç.0),
                saat_yönünde,
            );
            if dış_son_r > 0.0 {
                merkezli_yay(
                    &mut yol,
                    ekle(t1.merkez),
                    dış_son_r,
                    t1.ana_uç.1.atan2(t1.ana_uç.0),
                    t1.dış_uç.1.atan2(t1.dış_uç.0),
                    saat_yönünde,
                );
            }
        }
    } else {
        yol.taşı(ekle(dış_baş));
        merkezli_yay(&mut yol, merkez, dış, açı0, açı1, saat_yönünde);
    }

    if iç <= EPS {
        yol.çiz(merkez);
    } else if sınırlı_iç > EPS {
        iç_baş_r = iç_baş_r.min(sınırlı_iç);
        iç_son_r = iç_son_r.min(sınırlı_iç);
        let t0 = köşe_teğetleri(iç_son, dış_son, iç, -iç_son_r, saat_yönünde);
        let t1 = köşe_teğetleri(dış_baş, iç_baş, iç, -iç_baş_r, saat_yönünde);
        yol.çiz(ekle((t0.merkez.0 + t0.dış_uç.0, t0.merkez.1 + t0.dış_uç.1)));
        if sınırlı_iç < iç_en_çok && (iç_baş_r - iç_son_r).abs() <= EPS {
            merkezli_yay(
                &mut yol,
                ekle(t0.merkez),
                sınırlı_iç,
                t0.dış_uç.1.atan2(t0.dış_uç.0),
                t1.dış_uç.1.atan2(t1.dış_uç.0),
                saat_yönünde,
            );
        } else {
            if iç_son_r > 0.0 {
                merkezli_yay(
                    &mut yol,
                    ekle(t0.merkez),
                    iç_son_r,
                    t0.dış_uç.1.atan2(t0.dış_uç.0),
                    t0.ana_uç.1.atan2(t0.ana_uç.0),
                    saat_yönünde,
                );
            }
            merkezli_yay(
                &mut yol,
                merkez,
                iç,
                (t0.merkez.1 + t0.ana_uç.1).atan2(t0.merkez.0 + t0.ana_uç.0),
                (t1.merkez.1 + t1.ana_uç.1).atan2(t1.merkez.0 + t1.ana_uç.0),
                !saat_yönünde,
            );
            if iç_baş_r > 0.0 {
                merkezli_yay(
                    &mut yol,
                    ekle(t1.merkez),
                    iç_baş_r,
                    t1.ana_uç.1.atan2(t1.ana_uç.0),
                    t1.dış_uç.1.atan2(t1.dış_uç.0),
                    saat_yönünde,
                );
            }
        }
    } else {
        yol.çiz(ekle(iç_son));
        merkezli_yay(&mut yol, merkez, iç, açı1, açı0, !saat_yönünde);
    }
    yol.kapat();
    yol
}

/// Radyal gradyanı, durakları iç→dış yarıçapa eşleyerek düz renkli
/// eşmerkezli halkalarla yaklaşıklar.
fn radyal_halkalar(
    yüzey: &mut dyn ÇizimYüzeyi,
    merkez: (f32, f32),
    iç_yarıçap: f32,
    dış_yarıçap: f32,
    açı0: f32,
    açı1: f32,
    duraklar: &[crate::renk::RenkDurağı],
) {
    let (Some(ilk), Some(son)) = (duraklar.first(), duraklar.last()) else {
        return;
    };
    if duraklar.len() == 1 {
        let yol = dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1);
        yüzey.yol_doldur(&yol, &Dolgu::Düz(ilk.renk));
        return;
    }
    let yarıçap_çöz = |konum: f32| iç_yarıçap + (dış_yarıçap - iç_yarıçap) * konum.clamp(0.0, 1.0);

    // Uçlardaki düz bölgeler dahil ardışık durak çiftleri.
    let mut çiftler: Vec<(f32, f32, Renk, Renk)> = Vec::new();
    if ilk.konum > 0.0 {
        çiftler.push((0.0, ilk.konum, ilk.renk, ilk.renk));
    }
    for pencere in duraklar.windows(2) {
        if let [a, b] = pencere {
            çiftler.push((a.konum, b.konum, a.renk, b.renk));
        }
    }
    if son.konum < 1.0 {
        çiftler.push((son.konum, 1.0, son.renk, son.renk));
    }

    for (k0, k1, r0, r1) in çiftler {
        let y0 = yarıçap_çöz(k0);
        let y1 = yarıçap_çöz(k1);
        if y1 - y0 < 0.01 {
            continue;
        }
        // Halka kalınlığına göre alt bölme sayısı (4 px başına bir halka).
        let bölme = (((y1 - y0) / 4.0).ceil() as usize).clamp(1, 24);
        for i in 0..bölme {
            let t0 = i as f32 / bölme as f32;
            let t1 = (i + 1) as f32 / bölme as f32;
            let renk = r0.karıştır(r1, (t0 + t1) / 2.0);
            let yol = dilim_yolu(
                merkez,
                y0 + (y1 - y0) * t0,
                // Halkalar arasında dikiş görünmemesi için minik bindirme.
                (y0 + (y1 - y0) * t1) + 0.3,
                açı0,
                açı1,
            );
            yüzey.yol_doldur(&yol, &Dolgu::Düz(renk));
        }
    }
}
