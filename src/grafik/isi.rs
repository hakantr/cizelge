//! Isı haritası serisi — `echarts/src/chart/heatmap` karşılığı (kartezyen
//! kip). Veri öğeleri `[x sırası, y sırası, değer]` dizileridir; hücre
//! renkleri görsel eşlemeden çözülür.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::{Dikdörtgen, Kartezyen2B};
use crate::model::YatayKonum;
use crate::model::bilesen::Yön;
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::seri::IsıHaritasıSerisi;
use crate::model::stil::ÖğeStili;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

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
            (değer.is_finite()
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
        if !eşleme.seçili_mi(değer, eşleme_kapsamı) {
            continue;
        }
        // Parçalı eşlemede kapalı dilime düşen hücre çizilmez.
        if eşleme.parçalı_mı() {
            match eşleme.parça_bul(değer) {
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
                &binlik_ayır(değer),
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

    /// Bir sürükleme parçasını yatay piksel farkıyla taşıyıp yeni değer
    /// aralığını üretir. Tutamaçlar birbirini geçmez; aralık dolgusu kendi
    /// genişliğini koruyarak etkin kapsamın uçlarında durur.
    pub fn sürüklenmiş_aralık(
        &self,
        parça: GörselEşlemeSürgüParçası,
        piksel_farkı: f32,
    ) -> [f64; 2] {
        let açıklık = (self.kapsam[1] - self.kapsam[0]).max(f64::EPSILON);
        let fark = f64::from(piksel_farkı / self.şerit.genişlik.max(1.0)) * açıklık;
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
        let mut kutular = Vec::new();
        let n = eşleme.parçalar.len();
        let içerik_yüksekliği =
            n as f32 * KUTU_YÜKSEKLİĞİ + n.saturating_sub(1) as f32 * ÖĞE_BOŞLUĞU;
        let üst = eşleme
            .üst
            .map(|üst| üst.çöz(çizici.yükseklik()) + İÇ_BOŞLUK)
            .unwrap_or_else(|| {
                çizici.yükseklik()
                    - eşleme.alt.çöz(çizici.yükseklik())
                    - İÇ_BOŞLUK
                    - içerik_yüksekliği
            });
        let sağa_yaslı = eşleme.sağ.is_some() || eşleme.sol == YatayKonum::Sağ;
        let en_geniş_etiket = eşleme
            .parçalar
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
                YatayKonum::Sağ => çizici.genişlik() - 10.0 - İÇ_BOŞLUK - içerik_genişliği,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()) + İÇ_BOŞLUK,
            }
        };
        let kutu_x = if sağa_yaslı {
            içerik_solu + en_geniş_etiket + METİN_BOŞLUĞU
        } else {
            içerik_solu
        };
        for (satır, (i, parça)) in eşleme.parçalar.iter().enumerate().rev().enumerate() {
            let y = üst + satır as f32 * (KUTU_YÜKSEKLİĞİ + ÖĞE_BOŞLUĞU);
            let açık = eşleme.parça_açık_mı(i);
            let renk = if açık {
                parça.renk
            } else {
                tema::devre_dışı()
            };
            let kutu = Dikdörtgen::yeni(kutu_x, y, KUTU_GENİŞLİĞİ, KUTU_YÜKSEKLİĞİ);
            çizici.dikdörtgen(kutu, &Dolgu::Düz(renk), [3.0; 4], None);
            let yazı_rengi = if açık {
                tema::ikincil_metin()
            } else {
                tema::devre_dışı()
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
    if eşleme.yön == Yön::Yatay && eşleme.hesaplanabilir {
        // ContinuousView yatay `calculable` öntanımlıları: itemSize
        // 20×140, handleSize %120 ve bileşen padding'i 15 px. ECharts
        // çubuğu içeride 90° döndürür; burada aynı dünya geometrisini
        // doğrudan yatay koordinatlarda kuruyoruz.
        const ŞERİT_GENİŞLİĞİ: f32 = 140.0;
        const ŞERİT_YÜKSEKLİĞİ: f32 = 20.0;
        const TUTAMAÇ_GENİŞLİĞİ: f32 = 7.793_104;
        const TUTAMAÇ_YÜKSEKLİĞİ: f32 = 26.0;
        const İÇ_BOŞLUK: f32 = 15.0;
        let boyut = tema::YAZI_KÜÇÜK;
        let seçili = eşleme.seçili_kapsam(kapsam);
        let düşük = binlik_ayır(seçili[0]);
        let yüksek = binlik_ayır(seçili[1]);
        let düşük_genişliği = çizici.yazı_ölç(&düşük, boyut).0;
        let yüksek_genişliği = çizici.yazı_ölç(&yüksek, boyut).0;
        let yarım_tutamaç = TUTAMAÇ_GENİŞLİĞİ / 2.0;
        let içerik_solu = (-düşük_genişliği / 2.0).min(-yarım_tutamaç);
        let içerik_sağı =
            (ŞERİT_GENİŞLİĞİ + yüksek_genişliği / 2.0).max(ŞERİT_GENİŞLİĞİ + yarım_tutamaç);
        let dış_solu = içerik_solu - İÇ_BOŞLUK;
        let dış_sağı = içerik_sağı + İÇ_BOŞLUK;
        let dış_genişlik = dış_sağı - dış_solu;
        let şerit_x = if let Some(sağ) = eşleme.sağ {
            çizici.genişlik() - sağ.çöz(çizici.genişlik()) - dış_genişlik - dış_solu
        } else {
            match eşleme.sol {
                YatayKonum::Sol => 10.0 - dış_solu,
                YatayKonum::Orta => (çizici.genişlik() - dış_genişlik) / 2.0 - dış_solu,
                YatayKonum::Sağ => çizici.genişlik() - 10.0 - dış_genişlik - dış_solu,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()) - dış_solu,
            }
        };
        // Grup sınırının altı `bottom` değerine oturur. Tutamaç, çubuğun
        // iki yanında üçer piksel taşar; üzerine 15 px padding eklenir.
        let grup_altı = çizici.yükseklik() - eşleme.alt.çöz(çizici.yükseklik());
        let grup_y = grup_altı - (TUTAMAÇ_YÜKSEKLİĞİ - ŞERİT_YÜKSEKLİĞİ) / 2.0 - İÇ_BOŞLUK;
        let şerit_y = grup_y - ŞERİT_YÜKSEKLİĞİ;
        let durak_sayısı = eşleme.renkler.len().saturating_sub(1).max(1) as f32;
        let duraklar: Vec<crate::renk::RenkDurağı> = eşleme
            .renkler
            .iter()
            .enumerate()
            .map(|(sıra, renk)| crate::renk::RenkDurağı::yeni(sıra as f32 / durak_sayısı, *renk))
            .collect();
        let şerit = Dikdörtgen::yeni(şerit_x, şerit_y, ŞERİT_GENİŞLİĞİ, ŞERİT_YÜKSEKLİĞİ);
        çizici.dikdörtgen(şerit, &Dolgu::Düz(tema::devre_dışı()), [3.0; 4], None);
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

        let tutamaç_y = şerit_y - (TUTAMAÇ_YÜKSEKLİĞİ - ŞERİT_YÜKSEKLİĞİ) / 2.0;
        let alt_tutamaç = Dikdörtgen::yeni(
            şerit.x + alt_oran * şerit.genişlik - yarım_tutamaç,
            tutamaç_y,
            TUTAMAÇ_GENİŞLİĞİ,
            TUTAMAÇ_YÜKSEKLİĞİ,
        );
        let üst_tutamaç = Dikdörtgen::yeni(
            şerit.x + üst_oran * şerit.genişlik - yarım_tutamaç,
            tutamaç_y,
            TUTAMAÇ_GENİŞLİĞİ,
            TUTAMAÇ_YÜKSEKLİĞİ,
        );
        for (değer, tutamaç) in [(seçili[0], alt_tutamaç), (seçili[1], üst_tutamaç)] {
            çizici.dikdörtgen(
                tutamaç,
                &Dolgu::Düz(eşleme.renk_çöz(değer, kapsam)),
                [3.5; 4],
                Some((2.0, tema::nötr_00())),
            );
        }
        let etiket_y = şerit_y - 14.0;
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
            }),
        };
    }
    if eşleme.yön == Yön::Yatay {
        const ŞERİT_GENİŞLİĞİ: f32 = 140.0;
        const ŞERİT_YÜKSEKLİĞİ: f32 = 20.0;
        const METİN_BOŞLUĞU: f32 = 10.0;
        let boyut = tema::YAZI_KÜÇÜK;
        let yüksek = eşleme
            .metin
            .as_ref()
            .map(|(yüksek, _)| yüksek.clone())
            .unwrap_or_else(|| binlik_ayır(kapsam[1]));
        let düşük = eşleme
            .metin
            .as_ref()
            .map(|(_, düşük)| düşük.clone())
            .unwrap_or_else(|| binlik_ayır(kapsam[0]));
        let düşük_genişliği = çizici.yazı_ölç(&düşük, boyut).0;
        let yüksek_genişliği = çizici.yazı_ölç(&yüksek, boyut).0;
        let toplam_genişlik =
            düşük_genişliği + METİN_BOŞLUĞU + ŞERİT_GENİŞLİĞİ + METİN_BOŞLUĞU + yüksek_genişliği;
        let grup_x = if let Some(sağ) = eşleme.sağ {
            çizici.genişlik() - sağ.çöz(çizici.genişlik()) - toplam_genişlik
        } else {
            match eşleme.sol {
                YatayKonum::Sol => 10.0,
                YatayKonum::Orta => (çizici.genişlik() - toplam_genişlik) / 2.0,
                YatayKonum::Sağ => çizici.genişlik() - 10.0 - toplam_genişlik,
                YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()),
            }
        };
        let y = eşleme
            .üst
            .map(|üst| üst.çöz(çizici.yükseklik()))
            .unwrap_or_else(|| {
                çizici.yükseklik() - eşleme.alt.çöz(çizici.yükseklik()) - ŞERİT_YÜKSEKLİĞİ
            });
        let şerit_x = grup_x + düşük_genişliği + METİN_BOŞLUĞU;
        let durak_sayısı = eşleme.renkler.len().saturating_sub(1).max(1) as f32;
        let duraklar = eşleme
            .renkler
            .iter()
            .enumerate()
            .map(|(sıra, renk)| crate::renk::RenkDurağı::yeni(sıra as f32 / durak_sayısı, *renk))
            .collect();
        çizici.dikdörtgen(
            Dikdörtgen::yeni(şerit_x, y, ŞERİT_GENİŞLİĞİ, ŞERİT_YÜKSEKLİĞİ),
            &crate::renk::Dolgu::doğrusal(0.0, 0.0, 1.0, 0.0, duraklar),
            [0.0; 4],
            None,
        );
        çizici.yazı(
            &düşük,
            (şerit_x - METİN_BOŞLUĞU, y + ŞERİT_YÜKSEKLİĞİ / 2.0),
            YatayHiza::Sağ,
            DikeyHiza::Orta,
            boyut,
            tema::ikincil_metin(),
            false,
        );
        çizici.yazı(
            &yüksek,
            (
                şerit_x + ŞERİT_GENİŞLİĞİ + METİN_BOŞLUĞU,
                y + ŞERİT_YÜKSEKLİĞİ / 2.0,
            ),
            YatayHiza::Sol,
            DikeyHiza::Orta,
            boyut,
            tema::ikincil_metin(),
            false,
        );
        return GörselEşlemeÇıktısı::default();
    }
    const GENİŞLİK: f32 = 14.0;
    const YÜKSEKLİK: f32 = 130.0;
    let x = if let Some(sağ) = eşleme.sağ {
        çizici.genişlik() - sağ.çöz(çizici.genişlik()) - GENİŞLİK
    } else {
        match eşleme.sol {
            YatayKonum::Sol => 10.0,
            YatayKonum::Orta => (çizici.genişlik() - GENİŞLİK) / 2.0,
            YatayKonum::Sağ => çizici.genişlik() - 10.0 - GENİŞLİK,
            YatayKonum::Değer(uzunluk) => uzunluk.çöz(çizici.genişlik()),
        }
    };
    let y = eşleme
        .üst
        .map(|üst| üst.çöz(çizici.yükseklik()))
        .unwrap_or_else(|| çizici.yükseklik() - eşleme.alt.çöz(çizici.yükseklik()) - YÜKSEKLİK);

    // Şerit: renk duraklarını dikey gradyan bantları olarak çiz
    // (üst = en yüksek değer).
    let bölme_sayısı = eşleme.renkler.len().saturating_sub(1).max(1);
    let bant_yüksekliği = YÜKSEKLİK / bölme_sayısı as f32;
    for i in 0..bölme_sayısı {
        let üst_renk = eşleme
            .renkler
            .get(eşleme.renkler.len().saturating_sub(1).saturating_sub(i))
            .copied()
            .unwrap_or(Renk::SİYAH);
        let alt_renk = eşleme
            .renkler
            .get(eşleme.renkler.len().saturating_sub(2).saturating_sub(i))
            .copied()
            .unwrap_or(üst_renk);
        let bant = Dikdörtgen::yeni(x, y + i as f32 * bant_yüksekliği, GENİŞLİK, bant_yüksekliği);
        çizici.dikdörtgen(
            bant,
            &crate::renk::Dolgu::doğrusal(
                0.0,
                0.0,
                0.0,
                1.0,
                vec![
                    crate::renk::RenkDurağı::yeni(0.0, üst_renk),
                    crate::renk::RenkDurağı::yeni(1.0, alt_renk),
                ],
            ),
            [0.0; 4],
            None,
        );
    }

    // Uç etiketleri.
    let boyut = tema::YAZI_KÜÇÜK;
    let yüksek = eşleme
        .metin
        .as_ref()
        .map(|(yüksek, _)| yüksek.clone())
        .unwrap_or_else(|| binlik_ayır(kapsam[1]));
    let düşük = eşleme
        .metin
        .as_ref()
        .map(|(_, düşük)| düşük.clone())
        .unwrap_or_else(|| binlik_ayır(kapsam[0]));
    çizici.yazı(
        &yüksek,
        (x + GENİŞLİK / 2.0, y - 4.0),
        YatayHiza::Orta,
        DikeyHiza::Alt,
        boyut,
        tema::ikincil_metin(),
        false,
    );
    çizici.yazı(
        &düşük,
        (x + GENİŞLİK / 2.0, y + YÜKSEKLİK + 4.0),
        YatayHiza::Orta,
        DikeyHiza::Üst,
        boyut,
        tema::ikincil_metin(),
        false,
    );
    GörselEşlemeÇıktısı::default()
}

#[cfg(test)]
mod sürekli_bölge_testleri {
    use super::*;

    fn bölge() -> SürekliGörselEşlemeBölgesi {
        SürekliGörselEşlemeBölgesi {
            şerit: Dikdörtgen::yeni(10.0, 10.0, 100.0, 20.0),
            seçili_şerit: Dikdörtgen::yeni(30.0, 10.0, 40.0, 20.0),
            alt_tutamaç: Dikdörtgen::yeni(26.0, 7.0, 8.0, 26.0),
            üst_tutamaç: Dikdörtgen::yeni(66.0, 7.0, 8.0, 26.0),
            kapsam: [0.0, 10.0],
            seçili_aralık: [2.0, 6.0],
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
}
