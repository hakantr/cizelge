//! Isı haritası serisi — `echarts/src/chart/heatmap` karşılığı (kartezyen
//! kip). Veri öğeleri `[x sırası, y sırası, değer]` dizileridir; hücre
//! renkleri görsel eşlemeden çözülür.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B, MatrisYerleşimi};
use crate::model::bilesen::Yön;
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::matris::MatrisAralığı;
use crate::model::seri::IsıHaritasıSerisi;
use crate::model::stil::ÖğeStili;
use crate::model::{DikeyKonum, YatayKonum};
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

fn görsel_eşleme_değer_metni(değer: f64, hassasiyet: usize) -> String {
    if değer.is_finite() {
        format!("{:.*}", hassasiyet.min(20), değer)
    } else {
        değer.to_string()
    }
}

fn denetleyici_durakları(eşleme: &GörselEşleme) -> Vec<crate::renk::RenkDurağı> {
    let sayı = eşleme.denetleyici_durak_sayısı();
    let payda = sayı.saturating_sub(1).max(1) as f32;
    (0..sayı)
        .map(|sıra| {
            let oran = if sayı == 1 {
                0.0
            } else {
                sıra as f32 / payda
            };
            crate::renk::RenkDurağı::yeni(oran, eşleme.denetleyici_rengi(oran))
        })
        .collect()
}

fn denetleyici_zemini(eşleme: &GörselEşleme) -> Renk {
    eşleme
        .denetleyici_aralık_dışı_renk
        .unwrap_or_else(tema::devre_dışı)
}

/// Isı haritası serisinin değer kapsamı (görsel eşleme için).
pub fn ısı_değer_kapsamı(seri: &IsıHaritasıSerisi) -> [f64; 2] {
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    for öğe in &seri.veri {
        if let Some(&değer) = öğe.değer.dizi().and_then(|dizi| dizi.get(2))
            && değer.is_finite()
        {
            kapsam[0] = kapsam[0].min(değer);
            kapsam[1] = kapsam[1].max(değer);
        }
    }
    if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
        [0.0, 1.0]
    } else {
        kapsam
    }
}

/// Isı haritasını çizer; hücreler `eşleme` ile renklendirilir.
#[allow(clippy::too_many_arguments)]
pub fn ısı_haritası_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &IsıHaritasıSerisi,
    genel_sıra: usize,
    kartezyen: &Kartezyen2B,
    eşleme: &GörselEşleme,
    eşleme_kapsamı: [f64; 2],
    ilerleme: f32,
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> Option<usize> {
    let x_bant = kartezyen.x.bant_genişliği();
    let y_bant = kartezyen.y.bant_genişliği();
    let boşluk = seri.hücre_boşluğu.max(0.0);
    let opaklık = ilerleme.clamp(0.0, 1.0);
    let kategori_kapsamında = |x: f64, y: f64| {
        (!kartezyen.x.ölçek.kategorik_mi()
            || (x >= 0.0 && x < kartezyen.x.ölçek.kategori_sayısı() as f64))
            && (!kartezyen.y.ölçek.kategorik_mi()
                || (y >= 0.0 && y < kartezyen.y.ölçek.kategori_sayısı() as f64))
    };

    let hücre = |x_sırası: f64, y_sırası: f64| {
        let merkez_x = kartezyen.x.veriden_piksele(x_sırası);
        let merkez_y = kartezyen.y.veriden_piksele(y_sırası);
        // HeatmapView, komşu hücreler arasında alt-piksel yarığı
        // oluşmaması için layout bandını her yönde 0,25 px genişletir.
        // `cellGap` bu örtünün içinden düşülür.
        Dikdörtgen::yeni(
            merkez_x - x_bant / 2.0 - 0.25 + boşluk / 2.0,
            merkez_y - y_bant / 2.0 - 0.25 + boşluk / 2.0,
            (x_bant + 0.5 - boşluk).max(1.0),
            (y_bant + 0.5 - boşluk).max(1.0),
        )
    };
    let vurgulu = fare.and_then(|fare| {
        seri.veri.iter().enumerate().find_map(|(sıra, öğe)| {
            let dizi = öğe.değer.dizi()?;
            let (&x, &y, &değer) = (dizi.first()?, dizi.get(1)?, dizi.get(2)?);
            (kategori_kapsamında(x, y)
                && değer.is_finite()
                && eşleme.seçili_mi(değer, eşleme_kapsamı)
                && hücre(x, y).içeriyor_mu(fare))
            .then_some(sıra)
        })
    });

    for (i, öğe) in seri.veri.iter().enumerate() {
        let Some(dizi) = öğe.değer.dizi() else {
            continue;
        };
        let (Some(&x_sırası), Some(&y_sırası), Some(&değer)) =
            (dizi.first(), dizi.get(1), dizi.get(2))
        else {
            continue;
        };
        if !değer.is_finite() {
            continue;
        }
        if !kategori_kapsamında(x_sırası, y_sırası) {
            continue;
        }
        if !eşleme.seçili_mi(değer, eşleme_kapsamı) {
            continue;
        }
        // Parçalı eşlemede kapalı dilime düşen hücre çizilmez.
        if eşleme.parçalı_mı() {
            match eşleme.parça_bul_kapsamda(değer, eşleme_kapsamı) {
                Some(parça) if eşleme.parça_açık_mı(parça) => {}
                _ => continue,
            }
        }
        let d = hücre(x_sırası, y_sırası);
        let vurgu = vurgulu == Some(i);
        let normal = &seri.öğe_stili;
        let vurgu_stili = &seri.vurgu_öğe_stili;
        let stil_opaklığı = if vurgu {
            vurgu_stili.opaklık.or(normal.opaklık)
        } else {
            normal.opaklık
        }
        .unwrap_or(1.0);
        let renk = eşleme
            .renk_çöz(değer, eşleme_kapsamı)
            .opaklık(opaklık * stil_opaklığı);
        let kenarlık_rengi = if vurgu {
            vurgu_stili.kenarlık_rengi.or(normal.kenarlık_rengi)
        } else {
            normal.kenarlık_rengi
        };
        let kenarlık_kalınlığı = if vurgu && vurgu_stili.kenarlık_kalınlığı > 0.0 {
            vurgu_stili.kenarlık_kalınlığı
        } else {
            normal.kenarlık_kalınlığı
        };
        let kenarlık = kenarlık_rengi.map(|r| (kenarlık_kalınlığı.max(1.0), r));
        let yarıçap = if vurgu && vurgu_stili.kenarlık_yarıçapı.iter().any(|r| *r > 0.0) {
            vurgu_stili.kenarlık_yarıçapı
        } else {
            normal.kenarlık_yarıçapı
        };
        let gölge_stili: &ÖğeStili = if vurgu
            && (vurgu_stili.gölge_bulanıklığı > 0.0
                || vurgu_stili.gölge_rengi.is_some()
                || vurgu_stili.gölge_kayması != (0.0, 0.0))
        {
            vurgu_stili
        } else {
            normal
        };
        if gölge_stili.gölge_bulanıklığı > 0.0
            && let Some(gölge_rengi) = gölge_stili.gölge_rengi
        {
            let mut yol = Yol::yeni();
            yol.taşı((d.x, d.y));
            yol.çiz((d.sağ(), d.y));
            yol.çiz((d.sağ(), d.alt()));
            yol.çiz((d.x, d.alt()));
            yol.kapat();
            çizici.yol_gölgesi(
                &yol,
                gölge_rengi,
                gölge_stili.gölge_bulanıklığı,
                gölge_stili.gölge_kayması,
            );
        }
        çizici.dikdörtgen(d, &Dolgu::Düz(renk), yarıçap, kenarlık);

        if seri.etiket.göster {
            let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            // HeatmapView'in görsel renge göre otomatik iç etiket karşıtı:
            // açık hücrede legacy `#333`, koyu hücrede `#eee`.
            let parlaklık = 0.299 * renk.kırmızı + 0.587 * renk.yeşil + 0.114 * renk.mavi;
            let yazı_rengi = seri.etiket.yazı.renk.unwrap_or(if parlaklık < 0.55 {
                Renk::onaltılık(0xeeeeee)
            } else {
                Renk::onaltılık(0x333333)
            });
            // HeatmapView, otomatik etiket dolgusunun altına hücrenin
            // görsel rengiyle 2 px `textBorder` koyar. Bu, rakamın
            // kenarındaki alt-piksel örtüşmenin komşu hücre/eksen
            // rengine karışmasını engeller.
            çizici.dönüşümlü_konturlu_yazı(
                &ısı_etiket_metni(seri, öğe, değer),
                (0.0, 0.0),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                boyut,
                yazı_rengi,
                false,
                renk,
                2.0,
                AfinMatris::ötele(d.merkez().0, d.merkez().1 + 0.5),
            );
        }

        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: i,
            seri_adı: seri.ad.clone(),
            ad: öğe.ad.clone(),
            değer: Some(değer),
            geometri: İsabetGeometrisi::Dikdörtgen(d),
        });
    }
    vurgulu
}

fn ısı_etiket_metni(
    seri: &IsıHaritasıSerisi,
    öğe: &crate::model::deger::VeriÖğesi,
    değer: f64,
) -> String {
    let ham = binlik_ayır(değer);
    seri.etiket
        .biçimleyici
        .as_ref()
        .map(|biçimleyici| {
            biçimleyici.uygula_bağlamla(
                değer,
                &ham,
                seri.ad.as_deref().unwrap_or_default(),
                öğe.ad.as_deref().unwrap_or_default(),
            )
        })
        .unwrap_or(ham)
}

/// Matrix koordinatına bağlı ısı haritası. Her öğe açık
/// `matris_koordinatları` ile ya da `[x,y,value]` gövde sıra dizisiyle bir
/// hücre/aralığa yerleşir.
#[allow(clippy::too_many_arguments)]
pub fn matris_ısı_haritası_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &IsıHaritasıSerisi,
    genel_sıra: usize,
    matris: &MatrisYerleşimi,
    eşleme: &GörselEşleme,
    eşleme_kapsamı: [f64; 2],
    ilerleme: f32,
    fare: Option<(f32, f32)>,
    isabetler: &mut Vec<İsabetBölgesi>,
) -> Option<usize> {
    let hücre = |sıra: usize, öğe: &crate::model::deger::VeriÖğesi| {
        let dizi = öğe.değer.dizi()?;
        let açık = seri.matris_koordinatları.get(sıra).and_then(Option::as_ref);
        let (x, y) = match açık {
            Some((x, y)) => (x.clone(), y.clone()),
            None => {
                let x = *dizi.first()?;
                let y = *dizi.get(1)?;
                if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
                    return None;
                }
                (
                    MatrisAralığı::from(x.round() as usize),
                    MatrisAralığı::from(y.round() as usize),
                )
            }
        };
        let değer = *dizi.get(2)?;
        let mut kutu = matris.veriden_yerleşime(&x, &y, true)?;
        let boşluk = seri.hücre_boşluğu.max(0.0);
        // ECharts Matrix üzerindeki heatmap için `dataToLayout(...).rect`
        // sonucunu değiştirmeden kullanır. Kartezyen heatmap'e özgü yarım
        // piksel taşması burada uygulanmaz; aksi halde seri hücreleri Matrix
        // bileşeninin taban çizgilerinden çeyrek piksel kayar.
        kutu.x += boşluk / 2.0;
        kutu.y += boşluk / 2.0;
        kutu.genişlik = (kutu.genişlik - boşluk).max(1.0);
        kutu.yükseklik = (kutu.yükseklik - boşluk).max(1.0);
        Some((kutu, değer))
    };
    let vurgulu = fare.and_then(|fare| {
        seri.veri.iter().enumerate().find_map(|(sıra, öğe)| {
            let (kutu, değer) = hücre(sıra, öğe)?;
            (değer.is_finite()
                && eşleme.seçili_mi(değer, eşleme_kapsamı)
                && kutu.içeriyor_mu(fare))
            .then_some(sıra)
        })
    });

    for (sıra, öğe) in seri.veri.iter().enumerate() {
        let Some((kutu, değer)) = hücre(sıra, öğe) else {
            continue;
        };
        if !değer.is_finite() || !eşleme.seçili_mi(değer, eşleme_kapsamı) {
            continue;
        }
        if eşleme.parçalı_mı()
            && !matches!(
                eşleme.parça_bul_kapsamda(değer, eşleme_kapsamı),
                Some(parça) if eşleme.parça_açık_mı(parça)
            )
        {
            continue;
        }
        let vurgu = vurgulu == Some(sıra);
        let normal = &seri.öğe_stili;
        let vurgu_stili = &seri.vurgu_öğe_stili;
        let stil_opaklığı = if vurgu {
            vurgu_stili.opaklık.or(normal.opaklık)
        } else {
            normal.opaklık
        }
        .unwrap_or(1.0);
        let renk = eşleme
            .renk_çöz(değer, eşleme_kapsamı)
            .opaklık(ilerleme.clamp(0.0, 1.0) * stil_opaklığı);
        let kenarlık_rengi = if vurgu {
            vurgu_stili.kenarlık_rengi.or(normal.kenarlık_rengi)
        } else {
            normal.kenarlık_rengi
        };
        let kenarlık_kalınlığı = if vurgu && vurgu_stili.kenarlık_kalınlığı > 0.0 {
            vurgu_stili.kenarlık_kalınlığı
        } else {
            normal.kenarlık_kalınlığı
        };
        çizici.dikdörtgen(
            kutu,
            &Dolgu::Düz(renk),
            if vurgu && vurgu_stili.kenarlık_yarıçapı.iter().any(|r| *r > 0.0) {
                vurgu_stili.kenarlık_yarıçapı
            } else {
                normal.kenarlık_yarıçapı
            },
            kenarlık_rengi.map(|renk| (kenarlık_kalınlığı.max(1.0), renk)),
        );
        if seri.etiket.göster {
            let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            let parlaklık = 0.299 * renk.kırmızı + 0.587 * renk.yeşil + 0.114 * renk.mavi;
            let yazı_rengi = seri.etiket.yazı.renk.unwrap_or(if parlaklık < 0.55 {
                Renk::onaltılık(0xeeeeee)
            } else {
                Renk::onaltılık(0x333333)
            });
            çizici.dönüşümlü_konturlu_yazı(
                &ısı_etiket_metni(seri, öğe, değer),
                (0.0, 0.0),
                YatayHiza::Orta,
                DikeyHiza::Orta,
                boyut,
                yazı_rengi,
                seri.etiket.yazı.kalın,
                renk,
                2.0,
                // Matrix HeatmapView etiketi hücrenin ham merkezine bağlar;
                // Matrix bileşeninin alt-piksel çizgileri seri etiketini
                // ayrıca yarım piksel aşağı taşımaz.
                AfinMatris::ötele(kutu.merkez().0, kutu.merkez().1),
            );
        }
        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: sıra,
            seri_adı: seri.ad.clone(),
            ad: öğe.ad.clone(),
            değer: Some(değer),
            geometri: İsabetGeometrisi::Dikdörtgen(kutu),
        });
    }
    vurgulu
}

/// Sürekli `visualMap` sürüklemesinde vurulan parça.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GörselEşlemeSürgüParçası {
    AltTutamaç,
    ÜstTutamaç,
    Aralık,
}

/// Sürekli, hesaplanabilir `visualMap` bileşeninin isabet geometrisi.
#[derive(Clone, Copy, Debug)]
pub struct SürekliGörselEşlemeBölgesi {
    pub şerit: Dikdörtgen,
    pub seçili_şerit: Dikdörtgen,
    pub alt_tutamaç: Dikdörtgen,
    pub üst_tutamaç: Dikdörtgen,
    pub kapsam: [f64; 2],
    pub seçili_aralık: [f64; 2],
    pub dikey: bool,
}

impl SürekliGörselEşlemeBölgesi {
    pub fn kaydır(self, dx: f32, dy: f32) -> Self {
        let kaydır = |d: Dikdörtgen| Dikdörtgen::yeni(d.x + dx, d.y + dy, d.genişlik, d.yükseklik);
        Self {
            şerit: kaydır(self.şerit),
            seçili_şerit: kaydır(self.seçili_şerit),
            alt_tutamaç: kaydır(self.alt_tutamaç),
            üst_tutamaç: kaydır(self.üst_tutamaç),
            kapsam: self.kapsam,
            seçili_aralık: self.seçili_aralık,
            dikey: self.dikey,
        }
    }

    pub fn parça_bul(&self, nokta: (f32, f32)) -> Option<GörselEşlemeSürgüParçası> {
        if self.alt_tutamaç.içeriyor_mu(nokta) {
            Some(GörselEşlemeSürgüParçası::AltTutamaç)
        } else if self.üst_tutamaç.içeriyor_mu(nokta) {
            Some(GörselEşlemeSürgüParçası::ÜstTutamaç)
        } else if self.seçili_şerit.içeriyor_mu(nokta) {
            Some(GörselEşlemeSürgüParçası::Aralık)
        } else {
            None
        }
    }

    /// Fare noktasını değer artış yönündeki tek boyutlu sürükleme eksenine
    /// çevirir. Dikey bileşende ekran y'si aşağı arttığından işaret tersidir.
    pub fn sürükleme_ekseni(&self, nokta: (f32, f32)) -> f32 {
        if self.dikey { -nokta.1 } else { nokta.0 }
    }

    /// Bir sürükleme parçasını yatay piksel farkıyla taşıyıp yeni değer
    /// aralığını üretir. Tutamaçlar birbirini geçmez; aralık dolgusu kendi
    /// genişliğini koruyarak etkin kapsamın uçlarında durur.
    pub fn sürüklenmiş_aralık(
        &self,
        parça: GörselEşlemeSürgüParçası,
        piksel_farkı: f32,
    ) -> [f64; 2] {
        let açıklık = (self.kapsam[1] - self.kapsam[0]).max(f64::EPSILON);
        let şerit_uzunluğu = if self.dikey {
            self.şerit.yükseklik
        } else {
            self.şerit.genişlik
        };
        let fark = f64::from(piksel_farkı / şerit_uzunluğu.max(1.0)) * açıklık;
        let [ilk_alt, ilk_üst] = self.seçili_aralık;
        match parça {
            GörselEşlemeSürgüParçası::AltTutamaç => {
                [(ilk_alt + fark).clamp(self.kapsam[0], ilk_üst), ilk_üst]
            }
            GörselEşlemeSürgüParçası::ÜstTutamaç => {
                [ilk_alt, (ilk_üst + fark).clamp(ilk_alt, self.kapsam[1])]
            }
            GörselEşlemeSürgüParçası::Aralık => {
                let genişlik = ilk_üst - ilk_alt;
                let alt = (ilk_alt + fark).clamp(
                    self.kapsam[0],
                    (self.kapsam[1] - genişlik).max(self.kapsam[0]),
                );
                [alt, alt + genişlik]
            }
        }
    }
}

#[derive(Default)]
pub struct GörselEşlemeÇıktısı {
    pub parça_kutuları: Vec<(Dikdörtgen, usize)>,
    pub sürekli: Option<SürekliGörselEşlemeBölgesi>,
}

/// Görsel eşleme bileşenini seçeneklerdeki kutu yerleşimine göre çizer:
/// sürekli kipte gradyan çubuğu, parçalı kipte tıklanabilir dilim listesi.
/// Parçalı kipte her dilimin isabet kutusu döndürülür.
pub fn görsel_eşleme_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    eşleme: &GörselEşleme,
    kapsam: [f64; 2],
) -> GörselEşlemeÇıktısı {
    if !eşleme.göster {
        return GörselEşlemeÇıktısı::default();
    }
    if eşleme.parçalı_mı() {
        // PiecewiseModel varsayılanları: 20×14 simge, 10 px itemGap ve
        // ECharts 6 orta boy bileşen iç boşluğu (15 px). Dikey/inverse=false
        // düzeninde yüksek değer üstte görünür.
        const KUTU_GENİŞLİĞİ: f32 = 20.0;
        const KUTU_YÜKSEKLİĞİ: f32 = 14.0;
        const ÖĞE_BOŞLUĞU: f32 = 10.0;
        const METİN_BOŞLUĞU: f32 = 10.0;
        const İÇ_BOŞLUK: f32 = 15.0;
        let boyut = tema::YAZI_KÜÇÜK;
        let parçalar = eşleme.parçaları_çöz(kapsam);
        let mut kutular = Vec::new();
        let n = parçalar.len();
        if eşleme.yön == Yön::Yatay {
            // PiecewiseView yatay akışında parça sırası düşükten yükseğe,
            // soldan sağadır. Her öğenin genişliği kendi etiketiyle ölçülür;
            // böylece `left: 'center'` ECharts grubuyla aynı merkezi bulur.
            let etiketler = parçalar
                .iter()
                .map(|parça| {
                    let metin = parça.etiket_metni();
                    let genişlik = çizici.yazı_ölç(&metin, boyut).0;
                    (metin, genişlik)
                })
                .collect::<Vec<_>>();
            let öğe_genişlikleri = etiketler
                .iter()
                .map(|(_, genişlik)| KUTU_GENİŞLİĞİ + METİN_BOŞLUĞU + genişlik)
                .collect::<Vec<_>>();
            let içerik_genişliği =
                öğe_genişlikleri.iter().sum::<f32>() + n.saturating_sub(1) as f32 * ÖĞE_BOŞLUĞU;
            let içerik_yüksekliği = KUTU_YÜKSEKLİĞİ;
            let üst = match eşleme.dikey_konum {
                Some(DikeyKonum::Üst) => İÇ_BOŞLUK,
                Some(DikeyKonum::Orta) => (çizici.yükseklik() - içerik_yüksekliği) / 2.0,
                Some(DikeyKonum::Alt) => çizici.yükseklik() - İÇ_BOŞLUK - içerik_yüksekliği,
                Some(DikeyKonum::Değer(uzunluk)) => uzunluk.çöz(çizici.yükseklik()) + İÇ_BOŞLUK,
                None => eşleme.üst.map_or_else(
                    || {
                        çizici.yükseklik()
                            - eşleme.alt.çöz(çizici.yükseklik())
                            - İÇ_BOŞLUK
                            - içerik_yüksekliği
                    },
                    |üst| üst.çöz(çizici.yükseklik()) + İÇ_BOŞLUK,
                ),
            };
            let içerik_solu = if let Some(sağ) = eşleme.sağ {
                çizici.genişlik() - sağ.çöz(çizici.genişlik()) - İÇ_BOŞLUK - içerik_genişliği
            } else {
                match eşleme.sol {
                    YatayKonum::Sol => İÇ_BOŞLUK,
                    YatayKonum::Orta => (çizici.genişlik() - içerik_genişliği) / 2.0,
                    YatayKonum::Sağ => çizici.genişlik() - İÇ_BOŞLUK - içerik_genişliği,
                    YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()) + İÇ_BOŞLUK,
                }
            };
            let sağa_yaslı = eşleme.sağ.is_some() || eşleme.sol == YatayKonum::Sağ;
            let mut x = içerik_solu;
            for (i, ((parça, (etiket, etiket_genişliği)), öğe_genişliği)) in parçalar
                .iter()
                .zip(&etiketler)
                .zip(&öğe_genişlikleri)
                .enumerate()
            {
                let açık = eşleme.parça_açık_mı(i);
                let renk = if açık {
                    parça.renk
                } else {
                    tema::devre_dışı()
                };
                let kutu_x = if sağa_yaslı {
                    x + etiket_genişliği + METİN_BOŞLUĞU
                } else {
                    x
                };
                let kutu = Dikdörtgen::yeni(kutu_x, üst, KUTU_GENİŞLİĞİ, KUTU_YÜKSEKLİĞİ);
                çizici.dikdörtgen(kutu, &Dolgu::Düz(renk), [3.5; 4], None);
                let (etiket_x, hiza) = if sağa_yaslı {
                    (kutu_x - METİN_BOŞLUĞU, YatayHiza::Sağ)
                } else {
                    (kutu_x + KUTU_GENİŞLİĞİ + METİN_BOŞLUĞU, YatayHiza::Sol)
                };
                çizici.yazı(
                    etiket,
                    (etiket_x, üst + KUTU_YÜKSEKLİĞİ / 2.0),
                    hiza,
                    DikeyHiza::Orta,
                    boyut,
                    if açık {
                        tema::ikincil_metin()
                    } else {
                        tema::ikincil_metin().opaklık(0.5)
                    },
                    false,
                );
                kutular.push((Dikdörtgen::yeni(x, üst, *öğe_genişliği, KUTU_YÜKSEKLİĞİ), i));
                x += *öğe_genişliği + ÖĞE_BOŞLUĞU;
            }
            return GörselEşlemeÇıktısı {
                parça_kutuları: kutular,
                sürekli: None,
            };
        }
        let içerik_yüksekliği =
            n as f32 * KUTU_YÜKSEKLİĞİ + n.saturating_sub(1) as f32 * ÖĞE_BOŞLUĞU;
        let üst = match eşleme.dikey_konum {
            Some(DikeyKonum::Üst) => İÇ_BOŞLUK,
            Some(DikeyKonum::Orta) => (çizici.yükseklik() - içerik_yüksekliği) / 2.0,
            Some(DikeyKonum::Alt) => çizici.yükseklik() - İÇ_BOŞLUK - içerik_yüksekliği,
            Some(DikeyKonum::Değer(uzunluk)) => uzunluk.çöz(çizici.yükseklik()) + İÇ_BOŞLUK,
            None => eşleme.üst.map_or_else(
                || {
                    çizici.yükseklik()
                        - eşleme.alt.çöz(çizici.yükseklik())
                        - İÇ_BOŞLUK
                        - içerik_yüksekliği
                },
                |üst| üst.çöz(çizici.yükseklik()) + İÇ_BOŞLUK,
            ),
        };
        let sağa_yaslı = eşleme.sağ.is_some() || eşleme.sol == YatayKonum::Sağ;
        let en_geniş_etiket = parçalar
            .iter()
            .map(|parça| çizici.yazı_ölç(&parça.etiket_metni(), boyut).0)
            .fold(0.0_f32, f32::max);
        let içerik_genişliği = KUTU_GENİŞLİĞİ + METİN_BOŞLUĞU + en_geniş_etiket;
        let içerik_solu = if let Some(sağ) = eşleme.sağ {
            çizici.genişlik() - sağ.çöz(çizici.genişlik()) - İÇ_BOŞLUK - içerik_genişliği
        } else {
            match eşleme.sol {
                YatayKonum::Sol => İÇ_BOŞLUK,
                YatayKonum::Orta => (çizici.genişlik() - içerik_genişliği) / 2.0,
                YatayKonum::Sağ => çizici.genişlik() - İÇ_BOŞLUK - içerik_genişliği,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()) + İÇ_BOŞLUK,
            }
        };
        let kutu_x = if sağa_yaslı {
            içerik_solu + en_geniş_etiket + METİN_BOŞLUĞU
        } else {
            içerik_solu
        };
        for (satır, (i, parça)) in parçalar.iter().enumerate().rev().enumerate() {
            let y = üst + satır as f32 * (KUTU_YÜKSEKLİĞİ + ÖĞE_BOŞLUĞU);
            let açık = eşleme.parça_açık_mı(i);
            let renk = if açık {
                parça.renk
            } else {
                tema::devre_dışı()
            };
            let kutu = Dikdörtgen::yeni(kutu_x, y, KUTU_GENİŞLİĞİ, KUTU_YÜKSEKLİĞİ);
            çizici.dikdörtgen(kutu, &Dolgu::Düz(renk), [3.5; 4], None);
            let yazı_rengi = if açık {
                tema::ikincil_metin()
            } else {
                tema::ikincil_metin().opaklık(0.5)
            };
            let etiket = parça.etiket_metni();
            let (etiket_x, yatay_hiza) = if sağa_yaslı {
                (kutu_x - METİN_BOŞLUĞU, YatayHiza::Sağ)
            } else {
                (kutu_x + KUTU_GENİŞLİĞİ + METİN_BOŞLUĞU, YatayHiza::Sol)
            };
            çizici.yazı(
                &etiket,
                (etiket_x, y + KUTU_YÜKSEKLİĞİ / 2.0),
                yatay_hiza,
                DikeyHiza::Orta,
                boyut,
                yazı_rengi,
                false,
            );
            kutular.push((
                Dikdörtgen::yeni(içerik_solu, y, içerik_genişliği, KUTU_YÜKSEKLİĞİ),
                i,
            ));
        }
        return GörselEşlemeÇıktısı {
            parça_kutuları: kutular,
            sürekli: None,
        };
    }
    if eşleme.renkler.is_empty() {
        return GörselEşlemeÇıktısı::default();
    }
    if eşleme.yön == Yön::Dikey && eşleme.hesaplanabilir {
        // ContinuousView dikey `calculable` geometrisi. `symbolSize`
        // denetlenen görselse çubuk, alt ve üst uçtaki sembol çaplarını
        // izleyen bir yamuktur; aksi halde klasik 20×140 dikdörtgendir.
        const ETİKET_KENAR_BOŞLUĞU: f32 = 14.0;
        const ETİKET_YARI_YÜKSEKLİĞİ: f32 = 6.0;
        const İÇ_BOŞLUK: f32 = 15.0;
        let boyut = tema::YAZI_KÜÇÜK;
        let şerit_genişliği = eşleme.öğe_genişliği.unwrap_or(20.0).max(0.1);
        let şerit_yüksekliği = eşleme.öğe_yüksekliği.unwrap_or(140.0).max(0.1);
        let (yüksek_metin, düşük_metin) = eşleme
            .metin
            .as_ref()
            .map(|(yüksek, düşük)| {
                (
                    (!yüksek.is_empty()).then_some(yüksek.as_str()),
                    (!düşük.is_empty()).then_some(düşük.as_str()),
                )
            })
            .unwrap_or((None, None));
        let seçili = eşleme.seçili_kapsam(kapsam);
        let düşük = görsel_eşleme_değer_metni(seçili[0], eşleme.hassasiyet);
        let yüksek = görsel_eşleme_değer_metni(seçili[1], eşleme.hassasiyet);
        let düşük_genişliği = çizici.yazı_ölç(&düşük, boyut).0;
        let yüksek_genişliği = çizici.yazı_ölç(&yüksek, boyut).0;

        // `align: auto`, bileşen sağ yarıya yerleştirildiğinde uç
        // etiketlerini çubuğun soluna alır. Box konumu, padding ve handle
        // taşmaları dâhil dış sınır kutusuna uygulanır.
        let sayısal_sol = match eşleme.sol {
            YatayKonum::Değer(uzunluk) => Some(uzunluk.çöz(çizici.genişlik())),
            _ => None,
        };
        let sağa_yaslı = eşleme.sağ.is_some()
            || eşleme.sol == YatayKonum::Sağ
            || sayısal_sol.is_some_and(|sol| sol > çizici.genişlik() / 2.0);

        // ControllerModel, değişken hedef symbolSize kanalını itemWidth'e
        // normalleştirir ve okunaklı bir yamuğa ulaşmak için küçük ucu büyük
        // ucun üçte birine tamamlar.
        let değişken_sembol = eşleme
            .sembol_boyutu
            .is_some_and(|[düşük, yüksek]| (düşük - yüksek).abs() > f32::EPSILON);
        let denetleyici_boyutu = |oran: f32| {
            if değişken_sembol {
                şerit_genişliği / 3.0 + şerit_genişliği * (2.0 / 3.0) * oran
            } else {
                şerit_genişliği
            }
        };
        let alt_uç_boyutu = denetleyici_boyutu(0.0);
        let üst_uç_boyutu = denetleyici_boyutu(1.0);
        let yerel_başlangıç = |uç_boyutu: f32| {
            if sağa_yaslı {
                0.0
            } else {
                şerit_genişliği - uç_boyutu
            }
        };
        let tutamaç_taban_genişliği = şerit_genişliği * (26.0 / 20.0);
        let tutamaç_taban_yüksekliği = şerit_genişliği * (7.793_104 / 20.0);
        let tutamaç_boyutu = |uç_boyutu: f32| {
            let ölçek = uç_boyutu / şerit_genişliği;
            (
                tutamaç_taban_genişliği * ölçek,
                tutamaç_taban_yüksekliği * ölçek,
            )
        };

        // ECharts önce tam kapsamla bir "sketch" kurup bileşen kutusunu
        // hesaplar; seçili aralık değiştiğinde grup konumu sıçramaz.
        let (alt_tutamaç_genişliği, alt_tutamaç_yüksekliği) = tutamaç_boyutu(alt_uç_boyutu);
        let (üst_tutamaç_genişliği, üst_tutamaç_yüksekliği) = tutamaç_boyutu(üst_uç_boyutu);
        let alt_tutamaç_merkezi_x = yerel_başlangıç(alt_uç_boyutu) + alt_uç_boyutu / 2.0;
        let üst_tutamaç_merkezi_x = yerel_başlangıç(üst_uç_boyutu) + üst_uç_boyutu / 2.0;
        let etiket_x = if sağa_yaslı {
            -ETİKET_KENAR_BOŞLUĞU
        } else {
            şerit_genişliği + ETİKET_KENAR_BOŞLUĞU
        };
        let mut içerik_solu = (alt_tutamaç_merkezi_x - alt_tutamaç_genişliği / 2.0)
            .min(üst_tutamaç_merkezi_x - üst_tutamaç_genişliği / 2.0)
            .min(0.0);
        let mut içerik_sağı = (alt_tutamaç_merkezi_x + alt_tutamaç_genişliği / 2.0)
            .max(üst_tutamaç_merkezi_x + üst_tutamaç_genişliği / 2.0)
            .max(şerit_genişliği);
        if sağa_yaslı {
            içerik_solu = içerik_solu.min(etiket_x - düşük_genişliği.max(yüksek_genişliği));
        } else {
            içerik_sağı = içerik_sağı.max(etiket_x + düşük_genişliği.max(yüksek_genişliği));
        }
        for metin in yüksek_metin.into_iter().chain(düşük_metin) {
            let metin_genişliği = çizici.yazı_ölç(metin, boyut).0;
            içerik_solu = içerik_solu.min(şerit_genişliği / 2.0 - metin_genişliği / 2.0);
            içerik_sağı = içerik_sağı.max(şerit_genişliği / 2.0 + metin_genişliği / 2.0);
        }
        let dış_genişlik = içerik_sağı - içerik_solu + 2.0 * İÇ_BOŞLUK;
        let dış_x = if let Some(sağ) = eşleme.sağ {
            çizici.genişlik() - sağ.çöz(çizici.genişlik()) - dış_genişlik
        } else {
            match eşleme.sol {
                YatayKonum::Sol => 0.0,
                YatayKonum::Orta => (çizici.genişlik() - dış_genişlik) / 2.0,
                YatayKonum::Sağ => çizici.genişlik() - dış_genişlik,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()),
            }
        };
        let şerit_x = dış_x + İÇ_BOŞLUK - içerik_solu;

        let mut içerik_üstü = (-ETİKET_YARI_YÜKSEKLİĞİ).min(-üst_tutamaç_yüksekliği / 2.0);
        let mut içerik_altı = (şerit_yüksekliği + ETİKET_YARI_YÜKSEKLİĞİ)
            .max(şerit_yüksekliği + alt_tutamaç_yüksekliği / 2.0);
        if yüksek_metin.is_some() {
            içerik_üstü = içerik_üstü.min(-eşleme.metin_boşluğu - boyut);
        }
        if düşük_metin.is_some() {
            içerik_altı = içerik_altı.max(şerit_yüksekliği + eşleme.metin_boşluğu + boyut);
        }
        let dış_yükseklik = içerik_altı - içerik_üstü + 2.0 * İÇ_BOŞLUK;
        let dış_y = match eşleme.dikey_konum {
            Some(DikeyKonum::Üst) => 0.0,
            Some(DikeyKonum::Orta) => (çizici.yükseklik() - dış_yükseklik) / 2.0,
            Some(DikeyKonum::Alt) => çizici.yükseklik() - dış_yükseklik,
            Some(DikeyKonum::Değer(uzunluk)) => uzunluk.çöz(çizici.yükseklik()),
            None => eşleme.üst.map_or_else(
                || çizici.yükseklik() - eşleme.alt.çöz(çizici.yükseklik()) - dış_yükseklik,
                |üst| üst.çöz(çizici.yükseklik()),
            ),
        };
        let şerit_y = dış_y + İÇ_BOŞLUK - içerik_üstü;
        let duraklar = denetleyici_durakları(eşleme);
        let şerit = Dikdörtgen::yeni(şerit_x, şerit_y, şerit_genişliği, şerit_yüksekliği);
        let yamuk = |alt_y: f32, alt_boyut: f32, üst_y: f32, üst_boyut: f32| {
            let mut yol = Yol::yeni();
            yol.taşı((şerit.x + yerel_başlangıç(üst_boyut), üst_y));
            yol.çiz((şerit.x + yerel_başlangıç(üst_boyut) + üst_boyut, üst_y));
            yol.çiz((şerit.x + yerel_başlangıç(alt_boyut) + alt_boyut, alt_y));
            yol.çiz((şerit.x + yerel_başlangıç(alt_boyut), alt_y));
            yol.kapat();
            yol
        };
        let dış_yamuk = yamuk(şerit.alt(), alt_uç_boyutu, şerit.y, üst_uç_boyutu);
        çizici.yol_doldur(&dış_yamuk, &Dolgu::Düz(denetleyici_zemini(eşleme)));

        let açıklık = (kapsam[1] - kapsam[0]).max(f64::EPSILON);
        let oran = |değer: f64| ((değer - kapsam[0]) / açıklık).clamp(0.0, 1.0) as f32;
        let alt_oran = oran(seçili[0]);
        let üst_oran = oran(seçili[1]);
        let alt_boyut = denetleyici_boyutu(alt_oran);
        let üst_boyut = denetleyici_boyutu(üst_oran);
        let seçili_kutu = Dikdörtgen::yeni(
            şerit.x,
            şerit.alt() - üst_oran * şerit.yükseklik,
            şerit.genişlik,
            ((üst_oran - alt_oran) * şerit.yükseklik).max(0.1),
        );
        let seçili_yamuk = yamuk(seçili_kutu.alt(), alt_boyut, seçili_kutu.y, üst_boyut);
        çizici.yol_doldur(
            &seçili_yamuk,
            &crate::renk::Dolgu::doğrusal(0.0, 1.0, 0.0, 0.0, duraklar),
        );

        let alt_merkez_y = şerit.alt() - alt_oran * şerit.yükseklik;
        let üst_merkez_y = şerit.alt() - üst_oran * şerit.yükseklik;
        let (alt_tutamaç_genişliği, alt_tutamaç_yüksekliği) = tutamaç_boyutu(alt_boyut);
        let (üst_tutamaç_genişliği, üst_tutamaç_yüksekliği) = tutamaç_boyutu(üst_boyut);
        let alt_tutamaç_merkezi_x = şerit.x + yerel_başlangıç(alt_boyut) + alt_boyut / 2.0;
        let üst_tutamaç_merkezi_x = şerit.x + yerel_başlangıç(üst_boyut) + üst_boyut / 2.0;
        let alt_tutamaç = Dikdörtgen::yeni(
            alt_tutamaç_merkezi_x - alt_tutamaç_genişliği / 2.0,
            alt_merkez_y - alt_tutamaç_yüksekliği / 2.0,
            alt_tutamaç_genişliği,
            alt_tutamaç_yüksekliği,
        );
        let üst_tutamaç = Dikdörtgen::yeni(
            üst_tutamaç_merkezi_x - üst_tutamaç_genişliği / 2.0,
            üst_merkez_y - üst_tutamaç_yüksekliği / 2.0,
            üst_tutamaç_genişliği,
            üst_tutamaç_yüksekliği,
        );
        for (oran, tutamaç) in [(alt_oran, alt_tutamaç), (üst_oran, üst_tutamaç)] {
            çizici.dikdörtgen(
                tutamaç,
                &Dolgu::Düz(eşleme.denetleyici_rengi(oran)),
                [3.5; 4],
                Some((2.0, tema::nötr_00())),
            );
        }
        let (etiket_x, etiket_hizası) = if sağa_yaslı {
            (şerit.x - ETİKET_KENAR_BOŞLUĞU, YatayHiza::Sağ)
        } else {
            (
                şerit.x + şerit_genişliği + ETİKET_KENAR_BOŞLUĞU,
                YatayHiza::Sol,
            )
        };
        çizici.yazı(
            &düşük,
            (etiket_x, alt_merkez_y),
            etiket_hizası,
            DikeyHiza::Orta,
            boyut,
            tema::ikincil_metin(),
            false,
        );
        çizici.yazı(
            &yüksek,
            (etiket_x, üst_merkez_y),
            etiket_hizası,
            DikeyHiza::Orta,
            boyut,
            tema::ikincil_metin(),
            false,
        );
        if let Some(metin) = yüksek_metin {
            çizici.yazı(
                metin,
                (
                    şerit.x + şerit_genişliği / 2.0,
                    şerit.y - eşleme.metin_boşluğu,
                ),
                YatayHiza::Orta,
                DikeyHiza::Alt,
                boyut,
                tema::ikincil_metin(),
                false,
            );
        }
        if let Some(metin) = düşük_metin {
            çizici.yazı(
                metin,
                (
                    şerit.x + şerit_genişliği / 2.0,
                    şerit.alt() + eşleme.metin_boşluğu,
                ),
                YatayHiza::Orta,
                DikeyHiza::Üst,
                boyut,
                tema::ikincil_metin(),
                false,
            );
        }
        return GörselEşlemeÇıktısı {
            parça_kutuları: Vec::new(),
            sürekli: Some(SürekliGörselEşlemeBölgesi {
                şerit,
                seçili_şerit: seçili_kutu,
                alt_tutamaç,
                üst_tutamaç,
                kapsam,
                seçili_aralık: seçili,
                dikey: true,
            }),
        };
    }
    if eşleme.yön == Yön::Yatay && eşleme.hesaplanabilir {
        // ContinuousView yatay `calculable` öntanımlıları: itemSize
        // 20×140, handleSize %120 ve bileşen padding'i 15 px. ECharts
        // çubuğu içeride 90° döndürür; burada aynı dünya geometrisini
        // doğrudan yatay koordinatlarda kuruyoruz.
        const İÇ_BOŞLUK: f32 = 15.0;
        let şerit_genişliği = eşleme.öğe_yüksekliği.unwrap_or(140.0).max(0.1);
        let şerit_yüksekliği = eşleme.öğe_genişliği.unwrap_or(20.0).max(0.1);
        let tutamaç_genişliği = şerit_yüksekliği * (7.793_104 / 20.0);
        let tutamaç_yüksekliği = şerit_yüksekliği * (26.0 / 20.0);
        let boyut = tema::YAZI_KÜÇÜK;
        let seçili = eşleme.seçili_kapsam(kapsam);
        let düşük = görsel_eşleme_değer_metni(seçili[0], eşleme.hassasiyet);
        let yüksek = görsel_eşleme_değer_metni(seçili[1], eşleme.hassasiyet);
        let düşük_genişliği = çizici.yazı_ölç(&düşük, boyut).0;
        let yüksek_genişliği = çizici.yazı_ölç(&yüksek, boyut).0;
        let yarım_tutamaç = tutamaç_genişliği / 2.0;
        let içerik_solu = (-düşük_genişliği / 2.0).min(-yarım_tutamaç);
        let içerik_sağı =
            (şerit_genişliği + yüksek_genişliği / 2.0).max(şerit_genişliği + yarım_tutamaç);
        let dış_solu = içerik_solu - İÇ_BOŞLUK;
        let dış_sağı = içerik_sağı + İÇ_BOŞLUK;
        let dış_genişlik = dış_sağı - dış_solu;
        let şerit_x = if let Some(sağ) = eşleme.sağ {
            çizici.genişlik() - sağ.çöz(çizici.genişlik()) - dış_genişlik - dış_solu
        } else {
            match eşleme.sol {
                YatayKonum::Sol => -dış_solu,
                YatayKonum::Orta => (çizici.genişlik() - dış_genişlik) / 2.0 - dış_solu,
                YatayKonum::Sağ => çizici.genişlik() - dış_genişlik - dış_solu,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()) - dış_solu,
            }
        };
        // `align: auto`, üst yarıdaki yatay denetimde etiketleri çubuğun
        // altına; orta/alt yarıda ise üstüne koyar. Box konumu etiket,
        // tutamaç ve 15 px padding'in oluşturduğu bütün dış kutuya uygulanır.
        const ETİKET_BOŞLUĞU: f32 = 14.0;
        const ETİKET_YARI_YÜKSEKLİĞİ: f32 = 6.0;
        let tutamaç_taşması = (tutamaç_yüksekliği - şerit_yüksekliği) / 2.0;
        let dış_yükseklik = 2.0 * İÇ_BOŞLUK
            + şerit_yüksekliği
            + tutamaç_taşması
            + ETİKET_BOŞLUĞU
            + ETİKET_YARI_YÜKSEKLİĞİ;
        let dış_y = match eşleme.dikey_konum {
            Some(DikeyKonum::Üst) => 0.0,
            Some(DikeyKonum::Orta) => (çizici.yükseklik() - dış_yükseklik) / 2.0,
            Some(DikeyKonum::Alt) => çizici.yükseklik() - dış_yükseklik,
            Some(DikeyKonum::Değer(uzunluk)) => uzunluk.çöz(çizici.yükseklik()),
            None => eşleme.üst.map_or_else(
                || çizici.yükseklik() - eşleme.alt.çöz(çizici.yükseklik()) - dış_yükseklik,
                |üst| üst.çöz(çizici.yükseklik()),
            ),
        };
        let etiket_altta = dış_y + dış_yükseklik / 2.0 < çizici.yükseklik() / 2.0;
        let şerit_y = dış_y
            + İÇ_BOŞLUK
            + if etiket_altta {
                tutamaç_taşması
            } else {
                ETİKET_YARI_YÜKSEKLİĞİ + ETİKET_BOŞLUĞU
            };
        let duraklar = denetleyici_durakları(eşleme);
        let şerit = Dikdörtgen::yeni(şerit_x, şerit_y, şerit_genişliği, şerit_yüksekliği);
        çizici.dikdörtgen(
            şerit,
            &Dolgu::Düz(denetleyici_zemini(eşleme)),
            [3.0; 4],
            None,
        );
        let açıklık = (kapsam[1] - kapsam[0]).max(f64::EPSILON);
        let oran = |değer: f64| ((değer - kapsam[0]) / açıklık).clamp(0.0, 1.0) as f32;
        let alt_oran = oran(seçili[0]);
        let üst_oran = oran(seçili[1]);
        let seçili_kutu = Dikdörtgen::yeni(
            şerit.x + alt_oran * şerit.genişlik,
            şerit.y,
            ((üst_oran - alt_oran) * şerit.genişlik).max(0.1),
            şerit.yükseklik,
        );
        çizici.kırpılı(seçili_kutu, &mut |yüzey| {
            yüzey.dikdörtgen(
                şerit,
                &crate::renk::Dolgu::doğrusal(0.0, 0.0, 1.0, 0.0, duraklar.clone()),
                [3.0; 4],
                None,
            );
        });

        let tutamaç_y = şerit_y - (tutamaç_yüksekliği - şerit_yüksekliği) / 2.0;
        let alt_tutamaç = Dikdörtgen::yeni(
            şerit.x + alt_oran * şerit.genişlik - yarım_tutamaç,
            tutamaç_y,
            tutamaç_genişliği,
            tutamaç_yüksekliği,
        );
        let üst_tutamaç = Dikdörtgen::yeni(
            şerit.x + üst_oran * şerit.genişlik - yarım_tutamaç,
            tutamaç_y,
            tutamaç_genişliği,
            tutamaç_yüksekliği,
        );
        for (oran, tutamaç) in [(alt_oran, alt_tutamaç), (üst_oran, üst_tutamaç)] {
            çizici.dikdörtgen(
                tutamaç,
                &Dolgu::Düz(eşleme.denetleyici_rengi(oran)),
                [3.5; 4],
                Some((2.0, tema::nötr_00())),
            );
        }
        let etiket_y = if etiket_altta {
            şerit_y + şerit_yüksekliği + ETİKET_BOŞLUĞU
        } else {
            şerit_y - ETİKET_BOŞLUĞU
        };
        çizici.yazı(
            &düşük,
            (şerit.x + alt_oran * şerit.genişlik, etiket_y),
            YatayHiza::Orta,
            DikeyHiza::Orta,
            boyut,
            tema::ikincil_metin(),
            false,
        );
        çizici.yazı(
            &yüksek,
            (şerit.x + üst_oran * şerit.genişlik, etiket_y),
            YatayHiza::Orta,
            DikeyHiza::Orta,
            boyut,
            tema::ikincil_metin(),
            false,
        );
        return GörselEşlemeÇıktısı {
            parça_kutuları: Vec::new(),
            sürekli: Some(SürekliGörselEşlemeBölgesi {
                şerit,
                seçili_şerit: seçili_kutu,
                alt_tutamaç,
                üst_tutamaç,
                kapsam,
                seçili_aralık: seçili,
                dikey: false,
            }),
        };
    }
    if eşleme.yön == Yön::Yatay {
        const İÇ_BOŞLUK: f32 = 15.0;
        let şerit_genişliği = eşleme.öğe_yüksekliği.unwrap_or(140.0).max(0.1);
        let şerit_yüksekliği = eşleme.öğe_genişliği.unwrap_or(20.0).max(0.1);
        let boyut = tema::YAZI_KÜÇÜK;
        let (yüksek, düşük) = eşleme
            .metin
            .as_ref()
            .map(|(yüksek, düşük)| {
                (
                    (!yüksek.is_empty()).then_some(yüksek.as_str()),
                    (!düşük.is_empty()).then_some(düşük.as_str()),
                )
            })
            .unwrap_or((None, None));
        let düşük_genişliği = düşük.map_or(0.0, |metin| çizici.yazı_ölç(metin, boyut).0);
        let yüksek_genişliği = yüksek.map_or(0.0, |metin| çizici.yazı_ölç(metin, boyut).0);
        let düşük_bölümü = düşük.map_or(0.0, |_| düşük_genişliği + eşleme.metin_boşluğu);
        let yüksek_bölümü = yüksek.map_or(0.0, |_| eşleme.metin_boşluğu + yüksek_genişliği);
        let içerik_genişliği = düşük_bölümü + şerit_genişliği + yüksek_bölümü;
        let dış_genişlik = içerik_genişliği + 2.0 * İÇ_BOŞLUK;
        let dış_x = if let Some(sağ) = eşleme.sağ {
            çizici.genişlik() - sağ.çöz(çizici.genişlik()) - dış_genişlik
        } else {
            match eşleme.sol {
                YatayKonum::Sol => 0.0,
                YatayKonum::Orta => (çizici.genişlik() - dış_genişlik) / 2.0,
                YatayKonum::Sağ => çizici.genişlik() - dış_genişlik,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()),
            }
        };
        let dış_yükseklik = şerit_yüksekliği + 2.0 * İÇ_BOŞLUK;
        let dış_y = match eşleme.dikey_konum {
            Some(DikeyKonum::Üst) => 0.0,
            Some(DikeyKonum::Orta) => (çizici.yükseklik() - dış_yükseklik) / 2.0,
            Some(DikeyKonum::Alt) => çizici.yükseklik() - dış_yükseklik,
            Some(DikeyKonum::Değer(uzunluk)) => uzunluk.çöz(çizici.yükseklik()),
            None => eşleme.üst.map_or_else(
                || çizici.yükseklik() - eşleme.alt.çöz(çizici.yükseklik()) - dış_yükseklik,
                |üst| üst.çöz(çizici.yükseklik()),
            ),
        };
        let y = dış_y + İÇ_BOŞLUK;
        let şerit_x = dış_x + İÇ_BOŞLUK + düşük_bölümü;
        let şerit = Dikdörtgen::yeni(şerit_x, y, şerit_genişliği, şerit_yüksekliği);
        çizici.dikdörtgen(
            şerit,
            &Dolgu::Düz(denetleyici_zemini(eşleme)),
            [0.0; 4],
            None,
        );
        çizici.dikdörtgen(
            şerit,
            &crate::renk::Dolgu::doğrusal(0.0, 0.0, 1.0, 0.0, denetleyici_durakları(eşleme)),
            [0.0; 4],
            None,
        );
        if let Some(düşük) = düşük {
            çizici.yazı(
                düşük,
                (şerit_x - eşleme.metin_boşluğu, y + şerit_yüksekliği / 2.0),
                YatayHiza::Sağ,
                DikeyHiza::Orta,
                boyut,
                tema::ikincil_metin(),
                false,
            );
        }
        if let Some(yüksek) = yüksek {
            çizici.yazı(
                yüksek,
                (
                    şerit_x + şerit_genişliği + eşleme.metin_boşluğu,
                    y + şerit_yüksekliği / 2.0,
                ),
                YatayHiza::Sol,
                DikeyHiza::Orta,
                boyut,
                tema::ikincil_metin(),
                false,
            );
        }
        return GörselEşlemeÇıktısı::default();
    }
    const İÇ_BOŞLUK: f32 = 15.0;
    let genişlik = eşleme.öğe_genişliği.unwrap_or(20.0).max(0.1);
    let yükseklik = eşleme.öğe_yüksekliği.unwrap_or(140.0).max(0.1);
    let boyut = tema::YAZI_KÜÇÜK;
    let (yüksek, düşük) = eşleme
        .metin
        .as_ref()
        .map(|(yüksek, düşük)| {
            (
                (!yüksek.is_empty()).then_some(yüksek.as_str()),
                (!düşük.is_empty()).then_some(düşük.as_str()),
            )
        })
        .unwrap_or((None, None));
    let yüksek_yüksekliği = yüksek.map_or(0.0, |_| boyut);
    let düşük_yüksekliği = düşük.map_or(0.0, |_| boyut);
    let üst_bölümü = yüksek.map_or(0.0, |_| yüksek_yüksekliği + eşleme.metin_boşluğu);
    let alt_bölümü = düşük.map_or(0.0, |_| eşleme.metin_boşluğu + düşük_yüksekliği);
    let metin_genişliği = yüksek
        .into_iter()
        .chain(düşük)
        .map(|metin| çizici.yazı_ölç(metin, boyut).0)
        .fold(0.0_f32, f32::max);
    let içerik_genişliği = genişlik.max(metin_genişliği);
    let dış_genişlik = içerik_genişliği + 2.0 * İÇ_BOŞLUK;
    let dış_x = if let Some(sağ) = eşleme.sağ {
        çizici.genişlik() - sağ.çöz(çizici.genişlik()) - dış_genişlik
    } else {
        match eşleme.sol {
            YatayKonum::Sol => 0.0,
            YatayKonum::Orta => (çizici.genişlik() - dış_genişlik) / 2.0,
            YatayKonum::Sağ => çizici.genişlik() - dış_genişlik,
            YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()),
        }
    };
    let içerik_yüksekliği = üst_bölümü + yükseklik + alt_bölümü;
    let dış_yükseklik = içerik_yüksekliği + 2.0 * İÇ_BOŞLUK;
    let dış_y = match eşleme.dikey_konum {
        Some(DikeyKonum::Üst) => 0.0,
        Some(DikeyKonum::Orta) => (çizici.yükseklik() - dış_yükseklik) / 2.0,
        Some(DikeyKonum::Alt) => çizici.yükseklik() - dış_yükseklik,
        Some(DikeyKonum::Değer(uzunluk)) => uzunluk.çöz(çizici.yükseklik()),
        None => eşleme.üst.map_or_else(
            || çizici.yükseklik() - eşleme.alt.çöz(çizici.yükseklik()) - dış_yükseklik,
            |üst| üst.çöz(çizici.yükseklik()),
        ),
    };
    let x = dış_x + İÇ_BOŞLUK + (içerik_genişliği - genişlik) / 2.0;
    let y = dış_y + İÇ_BOŞLUK + üst_bölümü;
    let şerit = Dikdörtgen::yeni(x, y, genişlik, yükseklik);
    çizici.dikdörtgen(
        şerit,
        &Dolgu::Düz(denetleyici_zemini(eşleme)),
        [0.0; 4],
        None,
    );
    çizici.dikdörtgen(
        şerit,
        &crate::renk::Dolgu::doğrusal(0.0, 1.0, 0.0, 0.0, denetleyici_durakları(eşleme)),
        [0.0; 4],
        None,
    );
    if let Some(yüksek) = yüksek {
        çizici.yazı(
            yüksek,
            (dış_x + dış_genişlik / 2.0, y - eşleme.metin_boşluğu),
            YatayHiza::Orta,
            DikeyHiza::Alt,
            boyut,
            tema::ikincil_metin(),
            false,
        );
    }
    if let Some(düşük) = düşük {
        çizici.yazı(
            düşük,
            (
                dış_x + dış_genişlik / 2.0,
                y + yükseklik + eşleme.metin_boşluğu,
            ),
            YatayHiza::Orta,
            DikeyHiza::Üst,
            boyut,
            tema::ikincil_metin(),
            false,
        );
    }
    GörselEşlemeÇıktısı::default()
}

#[cfg(test)]
mod sürekli_bölge_testleri {
    use super::*;
    use crate::cizim::KayıtYüzeyi;

    fn bölge() -> SürekliGörselEşlemeBölgesi {
        SürekliGörselEşlemeBölgesi {
            şerit: Dikdörtgen::yeni(10.0, 10.0, 100.0, 20.0),
            seçili_şerit: Dikdörtgen::yeni(30.0, 10.0, 40.0, 20.0),
            alt_tutamaç: Dikdörtgen::yeni(26.0, 7.0, 8.0, 26.0),
            üst_tutamaç: Dikdörtgen::yeni(66.0, 7.0, 8.0, 26.0),
            kapsam: [0.0, 10.0],
            seçili_aralık: [2.0, 6.0],
            dikey: false,
        }
    }

    #[test]
    fn tutamaçlar_seçili_şeritten_önce_isabet_alır() {
        let bölge = bölge();

        assert_eq!(
            bölge.parça_bul((30.0, 15.0)),
            Some(GörselEşlemeSürgüParçası::AltTutamaç)
        );
        assert_eq!(
            bölge.parça_bul((50.0, 15.0)),
            Some(GörselEşlemeSürgüParçası::Aralık)
        );
        assert_eq!(bölge.parça_bul((15.0, 15.0)), None);
    }

    #[test]
    fn sürükleme_tutamaçları_ve_aralığı_kapsamda_tutar() {
        let bölge = bölge();

        assert_eq!(
            bölge.sürüklenmiş_aralık(GörselEşlemeSürgüParçası::AltTutamaç, 80.0),
            [6.0, 6.0]
        );
        assert_eq!(
            bölge.sürüklenmiş_aralık(GörselEşlemeSürgüParçası::ÜstTutamaç, 80.0),
            [2.0, 10.0]
        );
        assert_eq!(
            bölge.sürüklenmiş_aralık(GörselEşlemeSürgüParçası::Aralık, 80.0),
            [6.0, 10.0]
        );
        assert_eq!(
            bölge.sürüklenmiş_aralık(GörselEşlemeSürgüParçası::Aralık, -80.0),
            [0.0, 4.0]
        );
    }

    #[test]
    fn dikey_hesaplanabilir_geometri_resmi_yerlesimi_izler() {
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let eşleme = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(1.0)
            .hesaplanabilir(true)
            .sol(0.0_f32)
            .alt(0.0_f32);

        let çıktı = görsel_eşleme_çiz(&mut yüzey, &eşleme, [0.0, 1.0]);
        let bölge = çıktı.sürekli.expect("dikey isabet bölgesi");
        assert!(bölge.dikey);
        assert_eq!(bölge.şerit, Dikdörtgen::yeni(18.0, 364.0, 20.0, 140.0));
        assert!((bölge.alt_tutamaç.x - 15.0).abs() < 1e-4);
        assert!((bölge.alt_tutamaç.y - 500.103_45).abs() < 1e-4);
        assert!((bölge.üst_tutamaç.y - 360.103_45).abs() < 1e-4);
        assert_eq!(bölge.sürükleme_ekseni((20.0, 400.0)), -400.0);
    }

    #[test]
    fn yatay_parçalı_eşleme_düşükten_yükseğe_ve_ortalanmış_akar() {
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let eşleme = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(10_000.0)
            .bölme_sayısı(5)
            .yön(Yön::Yatay)
            .sol("center")
            .üst(65);

        let çıktı = görsel_eşleme_çiz(&mut yüzey, &eşleme, [0.0, 10_000.0]);
        assert_eq!(çıktı.parça_kutuları.len(), 5);
        assert_eq!(çıktı.parça_kutuları[0].1, 0);
        assert_eq!(çıktı.parça_kutuları[4].1, 4);
        assert_eq!(çıktı.parça_kutuları[0].0.y, 80.0);
        assert!(çıktı.parça_kutuları[0].0.x < çıktı.parça_kutuları[4].0.x);
        let kayıt = yüzey.döküm();
        assert!(kayıt.find("0 - 2000").unwrap() < kayıt.find("8000 - 10000").unwrap());
    }

    #[test]
    fn dikey_hesaplanabilir_sağda_etiketleri_sola_alıp_ortalanır() {
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let eşleme = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(1000.0)
            .hesaplanabilir(true)
            .sol("670")
            .üst("center");

        let çıktı = görsel_eşleme_çiz(&mut yüzey, &eşleme, [0.0, 1000.0]);
        let bölge = çıktı.sürekli.expect("dikey isabet bölgesi");
        assert_eq!(bölge.şerit.y, 192.5);
        assert!(bölge.şerit.x > 700.0);
        let kayıt = yüzey.döküm();
        assert!(kayıt.contains("yazı \"1000\""));
        assert!(!kayıt.contains("1,000"));
    }

    #[test]
    fn dikey_sembol_boyutu_denetcisi_item_size_ve_ust_metni_uygular() {
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let eşleme = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(250.0)
            .boyut(2usize)
            .sembol_boyutu(10.0, 70.0)
            .aralık_dışı_sembol_boyutu(10.0, 70.0)
            .hesaplanabilir(true)
            .sol("right")
            .üst("10%")
            .öğe_genişliği(30.0)
            .öğe_yüksekliği(120.0)
            .metin("圆形大小：PM2.5", "")
            .metin_boşluğu(30.0);

        let çıktı = görsel_eşleme_çiz(&mut yüzey, &eşleme, [0.0, 250.0]);
        let bölge = çıktı.sürekli.expect("dikey isabet bölgesi");

        assert_eq!(bölge.şerit.y, 109.5);
        assert_eq!(bölge.şerit.genişlik, 30.0);
        assert_eq!(bölge.şerit.yükseklik, 120.0);
        assert!(
            bölge.şerit.x > 630.0 && bölge.şerit.x < 640.0,
            "şerit x={}",
            bölge.şerit.x
        );
        assert!(bölge.üst_tutamaç.genişlik > bölge.alt_tutamaç.genişlik * 2.9);
        assert!(yüzey.döküm().contains("圆形大小：PM2.5"));
    }

    #[test]
    fn yatay_hesaplanabilir_üstte_etiketleri_çubuğun_altına_alır() {
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let eşleme = GörselEşleme::yeni()
            .en_az(0.0)
            .en_çok(1000.0)
            .hesaplanabilir(true)
            .yön(Yön::Yatay)
            .sol("center")
            .üst("top");

        let çıktı = görsel_eşleme_çiz(&mut yüzey, &eşleme, [0.0, 1000.0]);
        let bölge = çıktı.sürekli.expect("yatay isabet bölgesi");
        assert!(!bölge.dikey);
        assert_eq!(bölge.şerit.y, 18.0);
        assert!(bölge.şerit.x > 270.0 && bölge.şerit.x < 280.0);
        let kayıt = yüzey.döküm();
        let düşük = kayıt.find("yazı \"0\"").expect("alt uç etiketi");
        let yüksek = kayıt.find("yazı \"1000\"").expect("üst uç etiketi");
        assert!(düşük < yüksek);
        assert!(kayıt[düşük..].contains("(274.7,52.0)"));
    }
}
