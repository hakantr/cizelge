//! İmleyici seçenekleri — ECharts'taki `markLine`, `markPoint`, `markArea`
//! bileşenlerinin karşılığı (`echarts/src/component/marker`).

use crate::model::stil::{
    Biçimleyici, Etiket, YazıDikeyHizası, YazıStili, YazıYatayHizası, ÇizgiStili, ÇizgiTürü,
    ÖğeStili,
};

/// İm değeri: sabit sayı ya da seri verisinden türetilen istatistik
/// (`markLine.data[i].type: 'average' | 'min' | 'max'`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum İmDeğeri {
    Değer(f64),
    Ortalama,
    EnKüçük,
    EnBüyük,
}

/// İm çizgisinin yönü.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum İmYönü {
    /// Değer ekseni üzerinde sabit — yatay çizgi (`yAxis: …`).
    Yatay,
    /// Kategori/x ekseni üzerinde sabit — dikey çizgi (`xAxis: …`).
    Dikey,
}

/// `markLine.label.position`: etiketin çizginin başlangıç, orta veya bitiş
/// noktasına göre yerleşimi. `İç*Üst` / `İç*Alt` seçenekleri metni çizgi
/// doğrultusuna döndürür ve çizginin normalinde konumlandırır.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum İmÇizgisiEtiketKonumu {
    Başlangıç,
    Orta,
    #[default]
    Bitiş,
    İçBaşlangıç,
    İçBaşlangıçÜst,
    İçBaşlangıçAlt,
    İçOrta,
    İçOrtaÜst,
    İçOrtaAlt,
    İçBitiş,
    İçBitişÜst,
    İçBitişAlt,
}

/// Bir `markLine.data` öğesindeki `label` yaması. ECharts'ta öğe etiketi
/// genel `markLine.label` modelinden miras alır; bu nedenle alanlar isteğe
/// bağlıdır ve yalnız açıkça verilen değerler genel etiketi ezer.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct İmÇizgisiEtiketYaması {
    pub göster: Option<bool>,
    pub konum: Option<İmÇizgisiEtiketKonumu>,
    pub biçimleyici: Option<Biçimleyici>,
    pub yazı: Option<YazıStili>,
    /// ECharts `distance: [x, y]`: çizgi doğrultusu ve normalindeki uzaklık.
    pub uzaklık: Option<[f32; 2]>,
    pub yatay_hiza: Option<YazıYatayHizası>,
    pub dikey_hiza: Option<YazıDikeyHizası>,
}

impl İmÇizgisiEtiketYaması {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn göster(mut self, göster: bool) -> Self {
        self.göster = Some(göster);
        self
    }

    pub fn konum(mut self, konum: İmÇizgisiEtiketKonumu) -> Self {
        self.konum = Some(konum);
        self
    }

    pub fn biçimleyici(mut self, biçimleyici: impl Into<Biçimleyici>) -> Self {
        self.biçimleyici = Some(biçimleyici.into());
        self
    }

    pub fn yazı(mut self, yazı: YazıStili) -> Self {
        self.yazı = Some(yazı);
        self
    }

    pub fn uzaklık(mut self, doğrultu: f32, normal: f32) -> Self {
        self.uzaklık = Some([doğrultu, normal]);
        self
    }

    pub fn yatay_hiza(mut self, hiza: YazıYatayHizası) -> Self {
        self.yatay_hiza = Some(hiza);
        self
    }

    pub fn dikey_hiza(mut self, hiza: YazıDikeyHizası) -> Self {
        self.dikey_hiza = Some(hiza);
        self
    }
}

/// Tek bir im çizgisi tanımı (`markLine.data` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct İmÇizgisiTanımı {
    pub ad: Option<String>,
    pub yön: İmYönü,
    pub değer: İmDeğeri,
    pub etiket: Option<İmÇizgisiEtiketYaması>,
}

impl İmÇizgisiTanımı {
    pub fn yeni(yön: İmYönü, değer: İmDeğeri) -> Self {
        Self {
            ad: None,
            yön,
            değer,
            etiket: None,
        }
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn etiket(mut self, etiket: İmÇizgisiEtiketYaması) -> Self {
        self.etiket = Some(etiket);
        self
    }
}

/// Eksenler üzerinde bir markLine ucunun konumu. İstatistik türü, değerin
/// kendisini ve bulunduğu veri sırasını birlikte çözer.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum İmÇizgisiUcu {
    İstatistik(İmDeğeri),
    Koordinat(f64, f64),
}

/// İki uçlu markLine parçasının uç simgesi (`data[i][j].symbol`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum İmÇizgisiUçSimgesi {
    Yok,
    Daire,
    Ok,
}

/// İki uçlu markLine tanımı (`markLine.data: [[başlangıç, bitiş]]`).
#[derive(Clone, PartialEq, Debug)]
pub struct İmÇizgisiParçası {
    pub ad: Option<String>,
    pub başlangıç: İmÇizgisiUcu,
    pub bitiş: İmÇizgisiUcu,
    pub başlangıç_simgesi: İmÇizgisiUçSimgesi,
    pub bitiş_simgesi: İmÇizgisiUçSimgesi,
    pub etiket: Option<İmÇizgisiEtiketYaması>,
}

impl İmÇizgisiParçası {
    pub fn yeni(başlangıç: İmÇizgisiUcu, bitiş: İmÇizgisiUcu) -> Self {
        Self {
            ad: None,
            başlangıç,
            bitiş,
            başlangıç_simgesi: İmÇizgisiUçSimgesi::Daire,
            bitiş_simgesi: İmÇizgisiUçSimgesi::Ok,
            etiket: None,
        }
    }

    pub fn koordinatlar(başlangıç: (f64, f64), bitiş: (f64, f64)) -> Self {
        Self::yeni(
            İmÇizgisiUcu::Koordinat(başlangıç.0, başlangıç.1),
            İmÇizgisiUcu::Koordinat(bitiş.0, bitiş.1),
        )
    }

    pub fn ad(mut self, ad: impl Into<String>) -> Self {
        self.ad = Some(ad.into());
        self
    }

    pub fn uç_simgeleri(
        mut self,
        başlangıç: İmÇizgisiUçSimgesi,
        bitiş: İmÇizgisiUçSimgesi,
    ) -> Self {
        self.başlangıç_simgesi = başlangıç;
        self.bitiş_simgesi = bitiş;
        self
    }

    pub fn etiket(mut self, etiket: İmÇizgisiEtiketYaması) -> Self {
        self.etiket = Some(etiket);
        self
    }
}

/// İm çizgisi (`markLine`): seriye bağlı yatay/dikey başvuru çizgileri.
#[derive(Clone, PartialEq, Debug)]
pub struct İmÇizgisi {
    pub veri: Vec<İmÇizgisiTanımı>,
    pub parçalar: Vec<İmÇizgisiParçası>,
    /// Tek uçlu `data` çizgilerinin başlangıç/bitiş simgeleri (`symbol`).
    pub başlangıç_simgesi: İmÇizgisiUçSimgesi,
    pub bitiş_simgesi: İmÇizgisiUçSimgesi,
    /// Öntanımlı: seri renginde kesikli.
    pub stil: ÇizgiStili,
    pub etiket: Etiket,
    /// `markLine.label.position`; ECharts öntanımlısı `end`.
    pub etiket_konumu: İmÇizgisiEtiketKonumu,
    /// `markLine.label.distance`; sayı biçimi iki eksende aynı değere açılır.
    pub etiket_uzaklığı: [f32; 2],
}

impl Default for İmÇizgisi {
    fn default() -> Self {
        İmÇizgisi {
            veri: Vec::new(),
            parçalar: Vec::new(),
            başlangıç_simgesi: İmÇizgisiUçSimgesi::Daire,
            bitiş_simgesi: İmÇizgisiUçSimgesi::Ok,
            stil: ÇizgiStili {
                kalınlık: 1.0,
                tür: ÇizgiTürü::Kesikli,
                ..Default::default()
            },
            etiket: Etiket {
                göster: true,
                ..Default::default()
            },
            etiket_konumu: İmÇizgisiEtiketKonumu::Bitiş,
            etiket_uzaklığı: [5.0, 5.0],
        }
    }
}

impl İmÇizgisi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Yatay çizgi ekler (`{ yAxis: değer }` / `{ type: 'average' }`).
    pub fn yatay(mut self, değer: İmDeğeri) -> Self {
        self.veri.push(İmÇizgisiTanımı::yeni(İmYönü::Yatay, değer));
        self
    }

    /// Dikey çizgi ekler (`{ xAxis: değer }`).
    pub fn dikey(mut self, değer: İmDeğeri) -> Self {
        self.veri.push(İmÇizgisiTanımı::yeni(İmYönü::Dikey, değer));
        self
    }

    /// Adlandırılmış tanım ekler.
    pub fn tanım(mut self, tanım: İmÇizgisiTanımı) -> Self {
        self.veri.push(tanım);
        self
    }

    /// İki uçlu açık bir `markLine.data` öğesi ekler.
    pub fn parça(mut self, parça: İmÇizgisiParçası) -> Self {
        self.parçalar.push(parça);
        self
    }

    /// Serideki iki istatistik noktasını birleştirir; örneğin resmi
    /// `[{type:'min'}, {type:'max'}]` markLine biçimi.
    pub fn istatistik_parçası(mut self, başlangıç: İmDeğeri, bitiş: İmDeğeri) -> Self {
        self.parçalar.push(İmÇizgisiParçası::yeni(
            İmÇizgisiUcu::İstatistik(başlangıç),
            İmÇizgisiUcu::İstatistik(bitiş),
        ));
        self
    }

    /// Açık iki veri koordinatı arasında markLine çizer.
    pub fn koordinat_parçası(mut self, başlangıç: (f64, f64), bitiş: (f64, f64)) -> Self {
        self.parçalar
            .push(İmÇizgisiParçası::koordinatlar(başlangıç, bitiş));
        self
    }

    /// Son eklenen iki uçlu parçanın uç simgelerini ayarlar.
    pub fn parça_simgeleri(
        mut self,
        başlangıç: İmÇizgisiUçSimgesi,
        bitiş: İmÇizgisiUçSimgesi,
    ) -> Self {
        if let Some(parça) = self.parçalar.last_mut() {
            parça.başlangıç_simgesi = başlangıç;
            parça.bitiş_simgesi = bitiş;
        }
        self
    }

    pub fn stil(mut self, stil: ÇizgiStili) -> Self {
        self.stil = stil;
        self
    }

    pub fn uç_simgeleri(
        mut self,
        başlangıç: İmÇizgisiUçSimgesi,
        bitiş: İmÇizgisiUçSimgesi,
    ) -> Self {
        self.başlangıç_simgesi = başlangıç;
        self.bitiş_simgesi = bitiş;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket_uzaklığı = [etiket.uzaklık; 2];
        self.etiket = etiket;
        self
    }

    pub fn etiket_konumu(mut self, konum: İmÇizgisiEtiketKonumu) -> Self {
        self.etiket_konumu = konum;
        self
    }

    pub fn etiket_uzaklığı(mut self, doğrultu: f32, normal: f32) -> Self {
        self.etiket_uzaklığı = [doğrultu, normal];
        self
    }
}

/// Tek bir im noktası tanımı (`markPoint.data` öğesi).
#[derive(Clone, PartialEq, Debug)]
pub struct İmNoktasıTanımı {
    pub ad: Option<String>,
    /// İstatistik konumu (en büyük/en küçük değerli nokta) ya da
    /// `koordinat` ile doğrudan `(x, y)`.
    pub değer: Option<İmDeğeri>,
    pub koordinat: Option<(f64, f64)>,
}

/// İm noktası (`markPoint`): raptiye biçimli değer vurguları.
#[derive(Clone, PartialEq, Debug)]
pub struct İmNoktası {
    pub veri: Vec<İmNoktasıTanımı>,
    /// Raptiye çapı (`symbolSize`, ECharts öntanımlısı 50).
    pub boyut: f32,
    pub etiket: Etiket,
}

impl Default for İmNoktası {
    fn default() -> Self {
        İmNoktası {
            veri: Vec::new(),
            boyut: 50.0,
            etiket: Etiket {
                göster: true,
                yazı: YazıStili {
                    kalın: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

impl İmNoktası {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// En büyük değerli noktayı imler (`{ type: 'max' }`).
    pub fn en_büyük(mut self) -> Self {
        self.veri.push(İmNoktasıTanımı {
            ad: Some("En Büyük".to_string()),
            değer: Some(İmDeğeri::EnBüyük),
            koordinat: None,
        });
        self
    }

    /// En küçük değerli noktayı imler (`{ type: 'min' }`).
    pub fn en_küçük(mut self) -> Self {
        self.veri.push(İmNoktasıTanımı {
            ad: Some("En Küçük".to_string()),
            değer: Some(İmDeğeri::EnKüçük),
            koordinat: None,
        });
        self
    }

    /// Doğrudan `(x, y)` koordinatına im koyar (`{ coord: [x, y] }`).
    pub fn koordinat(mut self, x: f64, y: f64) -> Self {
        self.veri.push(İmNoktasıTanımı {
            ad: None,
            değer: None,
            koordinat: Some((x, y)),
        });
        self
    }

    /// Noktayı açık eksen koordinatında çizerken etikette ayrı bir değer
    /// gösterir (`{name, value, xAxis, yAxis}` markPoint biçimi).
    pub fn adlı_koordinat_değeri(
        mut self,
        ad: impl Into<String>,
        x: f64,
        y: f64,
        değer: f64,
    ) -> Self {
        self.veri.push(İmNoktasıTanımı {
            ad: Some(ad.into()),
            değer: Some(İmDeğeri::Değer(değer)),
            koordinat: Some((x, y)),
        });
        self
    }

    pub fn boyut(mut self, boyut: f32) -> Self {
        self.boyut = boyut;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }
}

/// Tek bir im alanı tanımı (`markArea.data` öğesi): eksen değerleriyle
/// sınırlanan dikdörtgen. `None` uç, ızgara kenarına uzanır.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct İmAlanıTanımı {
    pub x0: Option<f64>,
    pub x1: Option<f64>,
    pub y0: Option<f64>,
    pub y1: Option<f64>,
}

/// İm alanı (`markArea`): vurgulanan bölgeler.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct İmAlanı {
    pub veri: Vec<(Option<String>, İmAlanıTanımı)>,
    /// Öntanımlı: seri renginin %15 opaklısı.
    pub stil: ÖğeStili,
    pub etiket: Etiket,
}

impl İmAlanı {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// X aralığını vurgular (tüm yükseklik boyunca).
    pub fn x_aralığı(mut self, ad: impl Into<String>, x0: f64, x1: f64) -> Self {
        self.veri.push((
            Some(ad.into()),
            İmAlanıTanımı {
                x0: Some(x0),
                x1: Some(x1),
                ..Default::default()
            },
        ));
        self
    }

    /// Y aralığını vurgular (tüm genişlik boyunca).
    pub fn y_aralığı(mut self, ad: impl Into<String>, y0: f64, y1: f64) -> Self {
        self.veri.push((
            Some(ad.into()),
            İmAlanıTanımı {
                y0: Some(y0),
                y1: Some(y1),
                ..Default::default()
            },
        ));
        self
    }

    pub fn tanım(mut self, ad: Option<String>, tanım: İmAlanıTanımı) -> Self {
        self.veri.push((ad, tanım));
        self
    }

    pub fn stil(mut self, stil: ÖğeStili) -> Self {
        self.stil = stil;
        self
    }

    pub fn etiket(mut self, etiket: Etiket) -> Self {
        self.etiket = etiket;
        self
    }
}

/// Bir serinin imleyici üçlüsü.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct İmleyiciler {
    pub çizgi: Option<İmÇizgisi>,
    pub nokta: Option<İmNoktası>,
    pub alan: Option<İmAlanı>,
}

impl İmleyiciler {
    pub fn boş_mu(&self) -> bool {
        self.çizgi.is_none() && self.nokta.is_none() && self.alan.is_none()
    }
}
