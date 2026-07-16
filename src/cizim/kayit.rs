//! Kayıt yüzeyi — [`ÇizimYüzeyi`]nin çizim komutlarını okunabilir metin
//! satırları olarak biriktiren gerçeklemesi. Altın (golden) görsel regresyon
//! testlerinin temelidir: gpui olmadan, tümüyle belirlenimci çalışır.

use crate::cizim::yuzey::{DikeyHiza, SATIR_ORANI, YatayHiza, Yol, YolKomutu, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::stil::ÇizgiTürü;
use crate::renk::{Dolgu, Renk};

/// Kayan noktayı kararlı biçimde yazar (yuvarlama gürültüsünü keser).
fn s(v: f32) -> String {
    let yuvarlanmış = (v * 10.0).round() / 10.0;
    // -0.0 → 0.0 normalizasyonu.
    let yuvarlanmış = if yuvarlanmış == 0.0 { 0.0 } else { yuvarlanmış };
    format!("{yuvarlanmış:.1}")
}

fn renk_yaz(r: Renk) -> String {
    format!(
        "#{:02x}{:02x}{:02x}@{}",
        (r.kırmızı * 255.0).round() as u8,
        (r.yeşil * 255.0).round() as u8,
        (r.mavi * 255.0).round() as u8,
        s(r.alfa)
    )
}

fn dolgu_yaz(d: &Dolgu) -> String {
    match d {
        Dolgu::Düz(r) => renk_yaz(*r),
        Dolgu::DoğrusalGradyan { x, y, x2, y2, duraklar } => {
            let duraklar: Vec<String> = duraklar
                .iter()
                .map(|d| format!("{}:{}", s(d.konum), renk_yaz(d.renk)))
                .collect();
            format!(
                "doğrusal({},{})→({},{})[{}]",
                s(*x),
                s(*y),
                s(*x2),
                s(*y2),
                duraklar.join(" ")
            )
        }
        Dolgu::RadyalGradyan { x, y, yarıçap, duraklar } => {
            let duraklar: Vec<String> = duraklar
                .iter()
                .map(|d| format!("{}:{}", s(d.konum), renk_yaz(d.renk)))
                .collect();
            format!("radyal({},{},{})[{}]", s(*x), s(*y), s(*yarıçap), duraklar.join(" "))
        }
    }
}

fn yol_yaz(yol: &Yol) -> String {
    let parçalar: Vec<String> = yol
        .komutlar
        .iter()
        .map(|k| match *k {
            YolKomutu::Taşı(n) => format!("T({},{})", s(n.0), s(n.1)),
            YolKomutu::Çiz(n) => format!("Ç({},{})", s(n.0), s(n.1)),
            YolKomutu::Kübik { k1, k2, uç } => format!(
                "K({},{} {},{} {},{})",
                s(k1.0),
                s(k1.1),
                s(k2.0),
                s(k2.1),
                s(uç.0),
                s(uç.1)
            ),
            YolKomutu::Yay { yarıçap, büyük_yay, süpürme, uç } => format!(
                "Y({} {}{} {},{})",
                s(yarıçap),
                if büyük_yay { "B" } else { "b" },
                if süpürme { "S" } else { "s" },
                s(uç.0),
                s(uç.1)
            ),
            YolKomutu::Kapat => "Z".to_string(),
        })
        .collect();
    parçalar.join(" ")
}

fn tür_yaz(tür: ÇizgiTürü) -> &'static str {
    match tür {
        ÇizgiTürü::Düz => "düz",
        ÇizgiTürü::Kesikli => "kesikli",
        ÇizgiTürü::Noktalı => "noktalı",
    }
}

/// Komut kaydeden test yüzeyi.
///
/// Metin ölçümü belirlenimcidir: genişlik = karakter sayısı × boyut × 0.6.
pub struct KayıtYüzeyi {
    genişlik: f32,
    yükseklik: f32,
    girinti: usize,
    pub komutlar: Vec<String>,
}

impl KayıtYüzeyi {
    pub fn yeni(genişlik: f32, yükseklik: f32) -> Self {
        KayıtYüzeyi { genişlik, yükseklik, girinti: 0, komutlar: Vec::new() }
    }

    fn kaydet(&mut self, satır: String) {
        let boşluk = "  ".repeat(self.girinti);
        self.komutlar.push(format!("{boşluk}{satır}"));
    }

    /// Tüm komutların satır satır dökümü.
    pub fn döküm(&self) -> String {
        self.komutlar.join("\n")
    }
}

impl ÇizimYüzeyi for KayıtYüzeyi {
    fn genişlik(&self) -> f32 {
        self.genişlik
    }

    fn yükseklik(&self) -> f32 {
        self.yükseklik
    }

    fn yol_doldur(&mut self, yol: &Yol, dolgu: &Dolgu) {
        if yol.boş_mu() {
            return;
        }
        let satır = format!("doldur {} | {}", dolgu_yaz(dolgu), yol_yaz(yol));
        self.kaydet(satır);
    }

    fn yol_çiz(&mut self, yol: &Yol, kalınlık: f32, renk: Renk, tür: ÇizgiTürü) {
        if yol.boş_mu() || kalınlık <= 0.0 || renk.alfa <= 0.0 {
            return;
        }
        let satır = format!(
            "çiz {} k={} {} | {}",
            renk_yaz(renk),
            s(kalınlık),
            tür_yaz(tür),
            yol_yaz(yol)
        );
        self.kaydet(satır);
    }

    fn dikdörtgen(
        &mut self,
        d: Dikdörtgen,
        dolgu: &Dolgu,
        yarıçap: [f32; 4],
        kenarlık: Option<(f32, Renk)>,
    ) {
        if d.genişlik <= 0.0 || d.yükseklik <= 0.0 {
            return;
        }
        let mut satır = format!(
            "dikdörtgen ({},{} {}x{}) {} r=[{},{},{},{}]",
            s(d.x),
            s(d.y),
            s(d.genişlik),
            s(d.yükseklik),
            dolgu_yaz(dolgu),
            s(yarıçap[0]),
            s(yarıçap[1]),
            s(yarıçap[2]),
            s(yarıçap[3])
        );
        if let Some((kalınlık, renk)) = kenarlık {
            satır.push_str(&format!(" kenarlık={}:{}", s(kalınlık), renk_yaz(renk)));
        }
        self.kaydet(satır);
    }

    fn gölge(&mut self, d: Dikdörtgen, yarıçap: f32, renk: Renk, bulanıklık: f32) {
        let satır = format!(
            "gölge ({},{} {}x{}) r={} {} b={}",
            s(d.x),
            s(d.y),
            s(d.genişlik),
            s(d.yükseklik),
            s(yarıçap),
            renk_yaz(renk),
            s(bulanıklık)
        );
        self.kaydet(satır);
    }

    fn kırpılı(&mut self, d: Dikdörtgen, işlev: &mut dyn FnMut(&mut dyn ÇizimYüzeyi)) {
        self.kaydet(format!(
            "kırp ({},{} {}x{}) {{",
            s(d.x),
            s(d.y),
            s(d.genişlik),
            s(d.yükseklik)
        ));
        self.girinti += 1;
        işlev(self);
        self.girinti -= 1;
        self.kaydet("}".to_string());
    }

    fn yazı(
        &mut self,
        metin: &str,
        konum: (f32, f32),
        yatay: YatayHiza,
        dikey: DikeyHiza,
        boyut: f32,
        renk: Renk,
        kalın: bool,
    ) -> (f32, f32) {
        if metin.is_empty() {
            return (0.0, 0.0);
        }
        let (genişlik, yükseklik) = self.yazı_ölç(metin, boyut);
        let yatay_ad = match yatay {
            YatayHiza::Sol => "sol",
            YatayHiza::Orta => "orta",
            YatayHiza::Sağ => "sağ",
        };
        let dikey_ad = match dikey {
            DikeyHiza::Üst => "üst",
            DikeyHiza::Orta => "orta",
            DikeyHiza::Alt => "alt",
        };
        let satır = format!(
            "yazı \"{metin}\" ({},{}) {yatay_ad}/{dikey_ad} b={} {}{}",
            s(konum.0),
            s(konum.1),
            s(boyut),
            renk_yaz(renk),
            if kalın { " kalın" } else { "" }
        );
        self.kaydet(satır);
        (genişlik, yükseklik)
    }

    fn yazı_ölç(&self, metin: &str, boyut: f32) -> (f32, f32) {
        (
            metin.chars().count() as f32 * boyut * 0.6,
            boyut * SATIR_ORANI,
        )
    }

    fn olarak(&mut self) -> &mut dyn ÇizimYüzeyi {
        self
    }
}
