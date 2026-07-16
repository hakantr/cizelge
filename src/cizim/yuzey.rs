//! Çizim yüzeyi soyutlaması — zrender'ın `Painter` arayüzünün karşılığı.
//!
//! Tüm seri ve bileşen çizicileri bu trait üzerinden çizer; gpui gerçeklemesi
//! [`crate::cizim::cizici::Çizici`], test/golden gerçeklemesi
//! [`crate::cizim::kayit::KayıtYüzeyi`]dir. İleride SVG/PNG dışa aktarımı da
//! bu trait'in yeni bir gerçeklemesi olarak eklenecektir (Faz 8).

use crate::koordinat::Dikdörtgen;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

/// Yazı satır yüksekliğinin yazı boyutuna oranı.
pub const SATIR_ORANI: f32 = 1.4;

/// 1 piksellik çizgileri fiziksel piksel ızgarasına oturtur (yarım piksel
/// hizalama); eksen ve bölme çizgilerinin bulanıklaşmasını önler.
pub fn keskin(v: f32) -> f32 {
    v.floor() + 0.5
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
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Debug, Default)]
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
        self.komutlar.push(YolKomutu::Yay { yarıçap, büyük_yay, süpürme, uç });
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

    /// Dikdörtgen boyar; `yarıçap` köşe sırası `[sol üst, sağ üst, sağ alt,
    /// sol alt]`, `kenarlık` `(kalınlık, renk)` çiftidir.
    fn dikdörtgen(
        &mut self,
        d: Dikdörtgen,
        dolgu: &Dolgu,
        yarıçap: [f32; 4],
        kenarlık: Option<(f32, Renk)>,
    );

    /// Gölge boyar (ipucu penceresi vb. için).
    fn gölge(&mut self, d: Dikdörtgen, yarıçap: f32, renk: Renk, bulanıklık: f32);

    /// Çizimi verilen dikdörtgene kırparak `işlev`i çalıştırır.
    fn kırpılı(&mut self, d: Dikdörtgen, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi));

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

    /// Yazının kaplayacağı `(genişlik, yükseklik)` boyutunu ölçer.
    fn yazı_ölç(&self, metin: &str, boyut: f32) -> (f32, f32);

    // ------------------------------------------------------------------
    // Ortak yardımcılar (ilkellerden türetilir; her yüzeyde aynı çalışır).
    // ------------------------------------------------------------------

    /// İki nokta arasına çizgi çeker.
    fn çizgi(
        &mut self,
        a: (f32, f32),
        b: (f32, f32),
        kalınlık: f32,
        renk: Renk,
        tür: ÇizgiTürü,
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
        if let Some((kalınlık, renk)) = kenarlık {
            if kalınlık > 0.0 {
                let yol = daire_yolu(merkez, yarıçap);
                self.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
            }
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
        if let Some((kalınlık, renk)) = kenarlık {
            if kalınlık > 0.0 {
                let yol = dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1);
                self.yol_çiz(&yol, kalınlık, renk, ÇizgiTürü::Düz);
            }
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
    let (Some(ilk), Some(son)) = (duraklar.first(), duraklar.last()) else { return };
    if duraklar.len() == 1 {
        let yol = dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1);
        yüzey.yol_doldur(&yol, &Dolgu::Düz(ilk.renk));
        return;
    }
    let yarıçap_çöz =
        |konum: f32| iç_yarıçap + (dış_yarıçap - iç_yarıçap) * konum.clamp(0.0, 1.0);

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
