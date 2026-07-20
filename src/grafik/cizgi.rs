//! Çizgi serisi çizimi — `echarts/src/chart/line/LineView.ts` ile
//! `poly.ts` içindeki yumuşak eğri algoritmasının portu.

use crate::bilesen::eksen_cizimi::{kategori_etiket_adımı, kategori_görünür_sıraları};
use crate::cizim::{Yol, ÇizimYüzeyi};
use crate::grafik::{sembol_stilli_çiz, çizgi_stili_çöz};
use crate::koordinat::Kartezyen2B;
use crate::model::gorsel_esleme::GörselEşleme;
use crate::model::seri::{Basamak, ÇizgiSerisi, Örnekleme};
use crate::model::stil::EtiketKonumu;
use crate::model::veri_kumesi::BoyutSeçici;
use crate::renk::{Dolgu, Renk, RenkDurağı};
use crate::tema;
use crate::yardimci::bicim::ondalık_kırp;
use crate::yerlesim::yigin::YığınAralığı;

/// Boş değerleri `None` olan piksel noktası listesi.
pub type NoktaListesi = Vec<Option<(f32, f32)>>;

/// Serinin `(tepe, taban)` piksel noktalarını üretir; boş değerler `None`.
pub fn nokta_listeleri(
    seri: &ÇizgiSerisi,
    kartezyen: &Kartezyen2B,
    aralıklar: &[YığınAralığı],
) -> (NoktaListesi, NoktaListesi) {
    let yatay_değer_serisi = kartezyen.y.ölçek.kategorik_mi() && !kartezyen.x.ölçek.kategorik_mi();
    let mut tepeler = Vec::with_capacity(seri.veri.len());
    let mut tabanlar = Vec::with_capacity(seri.veri.len());
    for (i, öğe) in seri.veri.iter().enumerate() {
        let x_değeri = öğe.değer.x().unwrap_or(i as f64);
        match aralıklar.get(i).copied().flatten() {
            Some((taban, tepe)) => {
                if yatay_değer_serisi {
                    if !kartezyen.x.veri_penceresinde_mi(tepe)
                        || !kartezyen.y.veri_penceresinde_mi(i as f64)
                    {
                        tepeler.push(None);
                        tabanlar.push(None);
                        continue;
                    }
                    let y = kartezyen.y.veriden_piksele(i as f64);
                    tepeler.push(Some((kartezyen.x.veriden_piksele(tepe), y)));
                    tabanlar.push(Some((kartezyen.x.veriden_piksele(taban), y)));
                } else {
                    if !kartezyen.x.veri_penceresinde_mi(x_değeri)
                        || !kartezyen.y.veri_penceresinde_mi(tepe)
                    {
                        tepeler.push(None);
                        tabanlar.push(None);
                        continue;
                    }
                    let x = kartezyen.x.veriden_piksele(x_değeri);
                    tepeler.push(Some((x, kartezyen.y.veriden_piksele(tepe))));
                    tabanlar.push(Some((x, kartezyen.y.veriden_piksele(taban))));
                }
            }
            None => {
                tepeler.push(None);
                tabanlar.push(None);
            }
        }
    }
    (tepeler, tabanlar)
}

/// `poly.ts` içindeki `drawSegment`in yumuşak dal portu: ardışık noktalara,
/// uç aşımını sınırlayan kontrol noktalı kübik Bezier parçaları ekler.
///
/// `ilk_taşı` doğruysa parça `taşı` ile başlar, değilse `çiz` ile bağlanır.
pub fn yumuşak_parça_ekle(
    yol: &mut Yol,
    noktalar: &[(f32, f32)],
    yumuşaklık: f32,
    ilk_taşı: bool,
) {
    let Some(&ilk) = noktalar.first() else { return };
    if ilk_taşı {
        yol.taşı(ilk);
    } else {
        yol.çiz(ilk);
    }
    if noktalar.len() == 1 || yumuşaklık <= 0.0 {
        for n in noktalar.iter().skip(1) {
            yol.çiz(*n);
        }
        return;
    }

    let yumuşaklık = yumuşaklık as f64;
    // Bir sonraki parçanın başlangıç kontrol noktası.
    let (mut kx0, mut ky0) = (ilk.0 as f64, ilk.1 as f64);
    let mut önceki = ilk;

    for i in 1..noktalar.len() {
        let Some(&şimdiki) = noktalar.get(i) else {
            break;
        };
        let (öx, öy) = (önceki.0 as f64, önceki.1 as f64);
        let (x, y) = (şimdiki.0 as f64, şimdiki.1 as f64);
        let sonraki = noktalar.get(i + 1);

        let (kx1, ky1, sonraki_kx0, sonraki_ky0);
        if let Some(&(sx_f, sy_f)) = sonraki {
            let (sx, sy) = (sx_f as f64, sy_f as f64);
            let vx = sx - öx;
            let vy = sy - öy;

            let dx0 = x - öx;
            let dy0 = y - öy;
            let dx1 = sx - x;
            let dy1 = sy - y;
            let önceki_uzunluk = (dx0 * dx0 + dy0 * dy0).sqrt();
            let sonraki_uzunluk = (dx1 * dx1 + dy1 * dy1).sqrt();

            // Parça uzunluklarının oranı.
            let oran = sonraki_uzunluk / (sonraki_uzunluk + önceki_uzunluk).max(1e-12);

            let mut skx0 = x + vx * yumuşaklık * oran;
            let mut sky0 = y + vy * yumuşaklık * oran;
            // Uç aşımını önleyen yumuşaklık kısıtı: nokta ile sonraki nokta
            // arasında kal.
            skx0 = skx0.min(sx.max(x)).max(sx.min(x));
            sky0 = sky0.min(sy.max(y)).max(sy.min(y));
            // Düzeltilmiş kontrol noktasından cp1'i yeniden hesapla.
            let vx2 = skx0 - x;
            let vy2 = sky0 - y;
            let mut tkx1 = x - vx2 * önceki_uzunluk / sonraki_uzunluk.max(1e-12);
            let mut tky1 = y - vy2 * önceki_uzunluk / sonraki_uzunluk.max(1e-12);
            // Önceki nokta ile arada kal.
            tkx1 = tkx1.min(öx.max(x)).max(öx.min(x));
            tky1 = tky1.min(öy.max(y)).max(öy.min(y));
            // cp1 kırpıldıysa sonraki cp0'ı tekrar ayarla.
            let vx3 = x - tkx1;
            let vy3 = y - tky1;
            sonraki_kx0 = x + vx3 * sonraki_uzunluk / önceki_uzunluk.max(1e-12);
            sonraki_ky0 = y + vy3 * sonraki_uzunluk / önceki_uzunluk.max(1e-12);
            kx1 = tkx1;
            ky1 = tky1;
        } else {
            // Son nokta: kontrol noktaları uca oturur.
            kx1 = x;
            ky1 = y;
            sonraki_kx0 = x;
            sonraki_ky0 = y;
        }

        yol.kübik(
            (kx0 as f32, ky0 as f32),
            (kx1 as f32, ky1 as f32),
            (x as f32, y as f32),
        );
        kx0 = sonraki_kx0;
        ky0 = sonraki_ky0;
        önceki = şimdiki;
    }
}

/// En Büyük Üçgen Üç Kova (LTTB) örneklemesi: `hedef` sayıda nokta seçer.
/// Uçlar her zaman korunur; her kovadan, komşu kovalarla en büyük üçgen
/// alanını kuran nokta seçilir.
pub fn lttb_örnekle(noktalar: &[(f32, f32)], hedef: usize) -> Vec<(f32, f32)> {
    let n = noktalar.len();
    if hedef >= n || hedef < 3 {
        return noktalar.to_vec();
    }
    let (Some(&ilk), Some(&son)) = (noktalar.first(), noktalar.last()) else {
        return noktalar.to_vec();
    };
    let mut sonuç = Vec::with_capacity(hedef);
    sonuç.push(ilk);

    let kova_boyu = (n - 2) as f64 / (hedef - 2) as f64;
    let mut önceki = ilk;
    let mut kova_başı = 1usize;

    for k in 0..hedef - 2 {
        let kova_sonu = ((k as f64 + 1.0) * kova_boyu).floor() as usize + 1;
        let kova_sonu = kova_sonu.min(n - 1).max(kova_başı + 1);

        // Bir sonraki kovanın ortalaması (üçgenin üçüncü köşesi).
        let sonraki_başı = kova_sonu;
        let sonraki_sonu = (((k as f64 + 2.0) * kova_boyu).floor() as usize + 1).min(n - 1);
        let sonraki = noktalar
            .get(sonraki_başı..sonraki_sonu.max(sonraki_başı + 1))
            .unwrap_or(&[]);
        let (mut ox, mut oy) = (son.0 as f64, son.1 as f64);
        if !sonraki.is_empty() {
            ox = sonraki.iter().map(|p| p.0 as f64).sum::<f64>() / sonraki.len() as f64;
            oy = sonraki.iter().map(|p| p.1 as f64).sum::<f64>() / sonraki.len() as f64;
        }

        let mut en_iyi = None;
        let mut en_büyük_alan = -1.0f64;
        for p in noktalar.get(kova_başı..kova_sonu).unwrap_or(&[]) {
            let alan = ((önceki.0 as f64 - ox) * (p.1 as f64 - önceki.1 as f64)
                - (önceki.0 as f64 - p.0 as f64) * (oy - önceki.1 as f64))
                .abs();
            if alan > en_büyük_alan {
                en_büyük_alan = alan;
                en_iyi = Some(*p);
            }
        }
        if let Some(seçilen) = en_iyi {
            sonuç.push(seçilen);
            önceki = seçilen;
        }
        kova_başı = kova_sonu;
    }
    sonuç.push(son);
    sonuç
}

/// Kova ortalaması örneklemesi.
pub fn ortalama_örnekle(noktalar: &[(f32, f32)], hedef: usize) -> Vec<(f32, f32)> {
    let n = noktalar.len();
    if hedef >= n || hedef == 0 {
        return noktalar.to_vec();
    }
    let kova_boyu = n as f64 / hedef as f64;
    (0..hedef)
        .filter_map(|k| {
            let başı = (k as f64 * kova_boyu).floor() as usize;
            let sonu = (((k + 1) as f64 * kova_boyu).floor() as usize)
                .min(n)
                .max(başı + 1);
            let kova = noktalar.get(başı..sonu)?;
            if kova.is_empty() {
                return None;
            }
            let x = kova.iter().map(|p| p.0).sum::<f32>() / kova.len() as f32;
            let y = kova.iter().map(|p| p.1).sum::<f32>() / kova.len() as f32;
            Some((x, y))
        })
        .collect()
}

/// Basamaklı çizgi için ara noktaları üretir.
fn basamaklı_noktalar(noktalar: &[(f32, f32)], basamak: Basamak) -> Vec<(f32, f32)> {
    let mut sonuç = Vec::with_capacity(noktalar.len().saturating_mul(2));
    if let Some(&ilk) = noktalar.first() {
        sonuç.push(ilk);
    }
    for (&önceki, &n) in noktalar.iter().zip(noktalar.iter().skip(1)) {
        match basamak {
            Basamak::Baş => sonuç.push((önceki.0, n.1)),
            Basamak::Son => sonuç.push((n.0, önceki.1)),
            Basamak::Orta => {
                let orta_x = (önceki.0 + n.0) / 2.0;
                sonuç.push((orta_x, önceki.1));
                sonuç.push((orta_x, n.1));
            }
        }
        sonuç.push(n);
    }
    sonuç
}

/// `None` boşluklarına göre bitişik parçalara ayırır; `boşları_bağla`
/// doğruysa boşluklar atlanarak tek parça üretilir.
fn parçalara_ayır(
    noktalar: &[Option<(f32, f32)>], boşları_bağla: bool
) -> Vec<Vec<(f32, f32)>> {
    if boşları_bağla {
        let dolu: Vec<(f32, f32)> = noktalar.iter().flatten().copied().collect();
        return if dolu.is_empty() {
            Vec::new()
        } else {
            vec![dolu]
        };
    }
    let mut parçalar = Vec::new();
    let mut geçerli = Vec::new();
    for n in noktalar {
        match n {
            Some(nokta) => geçerli.push(*nokta),
            None => {
                if !geçerli.is_empty() {
                    parçalar.push(std::mem::take(&mut geçerli));
                }
            }
        }
    }
    if !geçerli.is_empty() {
        parçalar.push(geçerli);
    }
    parçalar
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GörselKoordinat {
    X,
    Y,
}

/// LineView yalnız x/y koordinat boyutuna bağlanan visualMap girdisini
/// polyline/polygon gradyanına dönüştürür. Boyut verilmezse ECharts'ın son
/// koordinat boyutu olan y kullanılır.
fn görsel_koordinat(seri: &ÇizgiSerisi, eşleme: &GörselEşleme) -> Option<GörselKoordinat> {
    match eşleme.boyut.as_ref() {
        None | Some(BoyutSeçici::Sıra(1)) => Some(GörselKoordinat::Y),
        Some(BoyutSeçici::Sıra(0)) => Some(GörselKoordinat::X),
        Some(BoyutSeçici::Ad(ad)) => {
            if ad == "x" || seri.eşleme.as_ref().is_some_and(|(x, _)| x == ad) {
                Some(GörselKoordinat::X)
            } else if ad == "y" || seri.eşleme.as_ref().is_some_and(|(_, y)| y == ad) {
                Some(GörselKoordinat::Y)
            } else {
                None
            }
        }
        Some(BoyutSeçici::Sıra(_)) => None,
    }
}

fn görsel_değer(seri: &ÇizgiSerisi, eşleme: &GörselEşleme, sıra: usize) -> Option<f64> {
    let öğe = seri.veri.get(sıra)?;
    match eşleme.boyut.as_ref() {
        None | Some(BoyutSeçici::Sıra(1)) => öğe.değer.sayı(),
        Some(BoyutSeçici::Sıra(0)) => öğe.değer.x().or(Some(sıra as f64)),
        Some(BoyutSeçici::Sıra(boyut)) => {
            öğe.boyutlar.get(*boyut).and_then(|(_, değer)| değer.sayı())
        }
        Some(BoyutSeçici::Ad(ad)) if ad == "x" => öğe.değer.x().or(Some(sıra as f64)),
        Some(BoyutSeçici::Ad(ad)) if ad == "y" => öğe.değer.sayı(),
        Some(BoyutSeçici::Ad(ad)) => öğe.boyut(ad).and_then(|değer| değer.sayı()),
    }
}

fn görsel_kapsam(seri: &ÇizgiSerisi, eşleme: &GörselEşleme) -> [f64; 2] {
    let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
    for sıra in 0..seri.veri.len() {
        if let Some(değer) = görsel_değer(seri, eşleme, sıra)
            && değer.is_finite()
        {
            kapsam[0] = kapsam[0].min(değer);
            kapsam[1] = kapsam[1].max(değer);
        }
    }
    if !kapsam[0].is_finite() || !kapsam[1].is_finite() {
        kapsam = [0.0, 1.0];
    }
    eşleme.kapsam_çöz(kapsam)
}

/// `LineView.getIsIgnoreFunc`: kategori ekseni dar olduğunda varsayılan
/// `showAllSymbol: 'auto'`, yalnız görünür eksen etiketlerine denk gelen
/// sembolleri bırakır. `None`, bütün sembollerin çizileceği anlamına gelir.
fn görünür_sembol_sıraları(
    çizici: &dyn ÇizimYüzeyi,
    seri: &ÇizgiSerisi,
    kartezyen: &Kartezyen2B,
) -> Option<Vec<usize>> {
    if seri.tüm_sembolleri_göster == Some(true) {
        return None;
    }
    let kategori_ekseni = if kartezyen.x.ölçek.kategorik_mi() {
        &kartezyen.x
    } else if kartezyen.y.ölçek.kategorik_mi() {
        &kartezyen.y
    } else {
        return None;
    };

    // ECharts, otomatik kipte kategori başına kullanılabilir alan sembolün
    // 1,5 katını karşılıyorsa bütün sembolleri korur.
    if seri.tüm_sembolleri_göster.is_none()
        && seri.sembol_boyutu * 1.5 <= kategori_ekseni.bant_genişliği()
    {
        return None;
    }

    let adım = kategori_etiket_adımı(çizici, kategori_ekseni);
    kategori_görünür_sıraları(kategori_ekseni, adım)
}

/// ECharts `getVisualGradient` portu. Renk durakları veri değerinden eksen
/// pikseline taşınır, sonra yolun sınır kutusuna göre yerelleştirilir. Bu
/// sayede raster, SVG ve GPUI yüzeyleri aynı gradyanı paylaşır.
fn görsel_gradyan(
    seri: &ÇizgiSerisi,
    eşleme: &GörselEşleme,
    kartezyen: &Kartezyen2B,
    yol: &Yol,
    opaklık: f32,
) -> Option<Dolgu> {
    let koordinat = görsel_koordinat(seri, eşleme)?;
    let kutu = yol.sınır_kutusu()?;
    let (eksen, piksel0, piksel1) = match koordinat {
        GörselKoordinat::X => (&kartezyen.x, kutu.x, kutu.sağ()),
        GörselKoordinat::Y => (&kartezyen.y, kutu.y, kutu.alt()),
    };
    let açıklık = piksel1 - piksel0;
    if açıklık.abs() < 1e-3 {
        return None;
    }
    let kapsam = görsel_kapsam(seri, eşleme);
    let mut kırılmalar = vec![piksel0, piksel1];
    if eşleme.parçalı_mı() {
        for parça in eşleme.parçaları_çöz(kapsam) {
            if let Some(değer) = parça.değer {
                kırılmalar.push(eksen.veriden_piksele(değer));
            }
            if let Some(değer) = parça.en_az {
                kırılmalar.push(eksen.veriden_piksele(değer));
            }
            if let Some(değer) = parça.en_çok {
                kırılmalar.push(eksen.veriden_piksele(değer));
            }
        }
    } else if eşleme.renkler.len() > 1 {
        let payda = (eşleme.renkler.len() - 1) as f64;
        for sıra in 0..eşleme.renkler.len() {
            let değer = kapsam[0] + (kapsam[1] - kapsam[0]) * sıra as f64 / payda;
            kırılmalar.push(eksen.veriden_piksele(değer));
        }
    }
    kırılmalar.retain(|piksel| {
        piksel.is_finite() && *piksel >= piksel0 - 1e-3 && *piksel <= piksel1 + 1e-3
    });
    kırılmalar.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    kırılmalar.dedup_by(|a, b| (*a - *b).abs() < 1e-3);

    let renk = |piksel: f32| {
        eşleme
            .renk_çöz(eksen.pikselden_veriye(piksel), kapsam)
            .opaklık(opaklık)
    };
    let mut duraklar = Vec::new();
    if eşleme.parçalı_mı() {
        for pencere in kırılmalar.windows(2) {
            let [baş, son] = pencere else { continue };
            let orta = (*baş + *son) / 2.0;
            let parça_rengi = renk(orta);
            duraklar.push(RenkDurağı::yeni((*baş - piksel0) / açıklık, parça_rengi));
            duraklar.push(RenkDurağı::yeni((*son - piksel0) / açıklık, parça_rengi));
        }
    } else {
        duraklar.extend(
            kırılmalar
                .into_iter()
                .map(|piksel| RenkDurağı::yeni((piksel - piksel0) / açıklık, renk(piksel))),
        );
    }
    if duraklar.is_empty() {
        return None;
    }
    Some(match koordinat {
        GörselKoordinat::X => Dolgu::doğrusal(0.0, 0.0, 1.0, 0.0, duraklar),
        GörselKoordinat::Y => Dolgu::doğrusal(0.0, 0.0, 0.0, 1.0, duraklar),
    })
}

/// ECharts çizgi görünümünün z2 katmanları: alan poligonu `0`, çizgi ve
/// semboller daha üst katmandadır. Yığınlı alanlarda bütün dolguların bütün
/// çizgilerden önce boyanması ortak sınırların kapanmaması için zorunludur.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ÇizgiKatmanı {
    Alan,
    ÇizgiVeSembol,
}

/// Çizgi serisini çizer: alan dolgusu, çizgi, semboller ve etiketler.
#[allow(clippy::too_many_arguments)]
pub fn çizgi_serisi_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &ÇizgiSerisi,
    kartezyen: &Kartezyen2B,
    aralıklar: &[YığınAralığı],
    seri_rengi: Renk,
    görsel_eşleme: Option<&GörselEşleme>,
    öğe_opaklıkları: Option<&[f32]>,
    ilerleme: f32,
    katman: ÇizgiKatmanı,
    uç_etiketi_y: Option<f32>,
) {
    let (tepeler, tabanlar) = nokta_listeleri(seri, kartezyen, aralıklar);
    let alan = kartezyen.alan;
    let görsel_değer_kapsamı = görsel_eşleme.map(|eşleme| görsel_kapsam(seri, eşleme));
    let görünür_semboller = görünür_sembol_sıraları(çizici, seri, kartezyen);
    let sembol_görünür_mü = |sıra: usize| {
        görünür_semboller
            .as_ref()
            .is_none_or(|görünürler| görünürler.binary_search(&sıra).is_ok())
    };

    let mut gövde = |ç: &mut dyn ÇizimYüzeyi| {
        let mut tepeler_parçalı = parçalara_ayır(&tepeler, seri.boşları_bağla);
        let mut tabanlar_parçalı = parçalara_ayır(&tabanlar, seri.boşları_bağla);
        // Büyük veri örneklemesi: hedef, ızgara genişliği kadar noktadır.
        // ECharts `sampling` belirtilmediğinde veriyi kendiliğinden LTTB'ye
        // indirmez; progressive çizim yalnız işi parçalara ayırır ve bütün
        // noktaları korur. Bu nedenle örnekleme yalnız açık seçenekle çalışır.
        let hedef = (alan.genişlik.max(2.0) as usize).max(2);
        let örnekleme = seri.örnekleme;
        if let Some(örnekleme) = örnekleme {
            let örnekle = |parça: &Vec<(f32, f32)>| match örnekleme {
                Örnekleme::Lttb => lttb_örnekle(parça, hedef),
                Örnekleme::Ortalama => ortalama_örnekle(parça, hedef),
            };
            tepeler_parçalı = tepeler_parçalı.iter().map(örnekle).collect();
            tabanlar_parçalı = tabanlar_parçalı.iter().map(örnekle).collect();
        }

        // 1) Alan dolgusu (çizginin altına).
        if katman == ÇizgiKatmanı::Alan
            && let Some(alan_stili) = &seri.alan_stili
        {
            for (tepe_parça, taban_parça) in tepeler_parçalı.iter().zip(&tabanlar_parçalı) {
                if tepe_parça.len() < 2 {
                    continue;
                }
                let mut yol = Yol::yeni();
                let üst: Vec<(f32, f32)> = match seri.basamak {
                    Some(b) => basamaklı_noktalar(tepe_parça, b),
                    None => tepe_parça.clone(),
                };
                let alt_kaynak: Vec<(f32, f32)> = match seri.basamak {
                    Some(b) => basamaklı_noktalar(taban_parça, b),
                    None => taban_parça.clone(),
                };
                let yumuşaklık = if seri.basamak.is_some() {
                    0.0
                } else {
                    seri.yumuşaklık
                };
                yumuşak_parça_ekle(&mut yol, &üst, yumuşaklık, true);
                let mut alt: Vec<(f32, f32)> = alt_kaynak;
                alt.reverse();
                yumuşak_parça_ekle(&mut yol, &alt, yumuşaklık, false);
                yol.kapat();
                let dolgu = alan_stili
                    .renk
                    .clone()
                    .map(|dolgu| dolgu.opaklık(alan_stili.opaklık))
                    .or_else(|| {
                        görsel_eşleme.and_then(|eşleme| {
                            görsel_gradyan(seri, eşleme, kartezyen, &yol, alan_stili.opaklık)
                        })
                    })
                    .unwrap_or_else(|| Dolgu::Düz(seri_rengi).opaklık(alan_stili.opaklık));
                ç.yol_doldur(&yol, &dolgu);
            }
        }

        if katman == ÇizgiKatmanı::Alan {
            return;
        }

        // 2) Çizgi.
        let (çizgi_rengi, kalınlık, tür) = çizgi_stili_çöz(&seri.çizgi_stili, seri_rengi);
        for parça in &tepeler_parçalı {
            if parça.len() < 2 {
                continue;
            }
            let noktalar: Vec<(f32, f32)> = match seri.basamak {
                Some(b) => basamaklı_noktalar(parça, b),
                None => parça.clone(),
            };
            let mut yol = Yol::yeni();
            let yumuşaklık = if seri.basamak.is_some() {
                0.0
            } else {
                seri.yumuşaklık
            };
            yumuşak_parça_ekle(&mut yol, &noktalar, yumuşaklık, true);
            if let Some(gölge_rengi) = seri.çizgi_stili.gölge_rengi
                && (seri.çizgi_stili.gölge_bulanıklığı > 0.0
                    || seri.çizgi_stili.gölge_kayması != (0.0, 0.0))
            {
                ç.yol_çizgi_gölgesi(
                    &yol,
                    kalınlık,
                    tür,
                    // tiny-skia'nın vuruş A8 örtüsü Chromium/Skia Canvas
                    // maskesinden çok az daha yoğundur.
                    gölge_rengi.opaklık(0.95),
                    seri.çizgi_stili.gölge_bulanıklığı,
                    seri.çizgi_stili.gölge_kayması,
                );
            }
            if seri.çizgi_stili.renk.is_none()
                && let Some(dolgu) = görsel_eşleme.and_then(|eşleme| {
                    görsel_gradyan(seri, eşleme, kartezyen, &yol, seri.çizgi_stili.opaklık)
                })
            {
                ç.yol_dolgulu_çiz(&yol, kalınlık, &dolgu, tür);
            } else {
                ç.yol_çiz(&yol, kalınlık, çizgi_rengi, tür);
            }
        }

        // 3) Semboller.
        if seri.sembol_göster && seri.sembol != crate::model::seri::Sembol::Yok {
            for (i, nokta) in tepeler.iter().enumerate() {
                let Some(nokta) = nokta else { continue };
                if !sembol_görünür_mü(i) {
                    continue;
                }
                let veri_stili = seri.veri.get(i).and_then(|öğe| öğe.stil.as_ref());
                let açık_dolgu = veri_stili
                    .and_then(|stil| stil.renk.as_ref())
                    .or(seri.öğe_stili.renk.as_ref());
                let görsel_renk = görsel_eşleme.and_then(|eşleme| {
                    görsel_değer(seri, eşleme, i)
                        .zip(görsel_değer_kapsamı)
                        .map(|(değer, kapsam)| eşleme.renk_çöz(değer, kapsam))
                });
                let görsel_dolgu = görsel_renk.map(Dolgu::Düz);
                let dolgu = açık_dolgu.or(görsel_dolgu.as_ref());
                let kenarlık_rengi = veri_stili
                    .and_then(|stil| stil.kenarlık_rengi)
                    .or(seri.öğe_stili.kenarlık_rengi);
                let kenarlık_kalınlığı = veri_stili
                    .filter(|stil| stil.kenarlık_kalınlığı > 0.0)
                    .map(|stil| stil.kenarlık_kalınlığı)
                    .unwrap_or(seri.öğe_stili.kenarlık_kalınlığı);
                let kenarlık = kenarlık_rengi
                    .filter(|_| kenarlık_kalınlığı > 0.0)
                    .map(|renk| (kenarlık_kalınlığı, renk));
                let opaklık = veri_stili
                    .and_then(|stil| stil.opaklık)
                    .or(seri.öğe_stili.opaklık)
                    .unwrap_or(1.0)
                    * öğe_opaklıkları
                        .and_then(|opaklıklar| opaklıklar.get(i))
                        .copied()
                        .unwrap_or(1.0);
                sembol_stilli_çiz(
                    ç,
                    &seri.sembol,
                    *nokta,
                    seri.sembol_boyutu,
                    görsel_renk.unwrap_or(seri_rengi),
                    dolgu,
                    kenarlık,
                    opaklık,
                    false,
                );
            }
        }

        // 4) Değer etiketleri.
        // ECharts LineView etiketleri SymbolDraw öğesine bağlar. Seri
        // `showSymbol: false` ya da `symbol: 'none'` ile sembol grubunu
        // kaldırdığında açık `label.show` da tek başına etiket üretmez.
        if seri.etiket.göster
            && seri.sembol_göster
            && seri.sembol != crate::model::seri::Sembol::Yok
        {
            let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            let renk = seri.etiket.yazı.renk.unwrap_or(tema::birincil_metin());
            for (i, nokta) in tepeler.iter().enumerate() {
                let Some((x, y)) = nokta else { continue };
                if !sembol_görünür_mü(i) {
                    continue;
                }
                let Some(öğe) = seri.veri.get(i) else {
                    continue;
                };
                let Some(değer) = öğe.değer.sayı() else {
                    continue;
                };
                let metin = match &seri.etiket.biçimleyici {
                    Some(b) => b.uygula(değer, &ondalık_kırp(değer)),
                    None => ondalık_kırp(değer),
                };
                // AttachedText konumu sembolün sınır kutusundan başlar ve
                // `label.distance` kadar daha dışarı gider.
                let sembol_yarıçapı = seri.sembol_boyutu / 2.0;
                let uzaklık = seri.etiket.uzaklık + sembol_yarıçapı;
                let (hiza, kaydırma) = match seri.etiket.konum {
                    EtiketKonumu::Alt => (crate::cizim::DikeyHiza::Üst, uzaklık),
                    _ => (crate::cizim::DikeyHiza::Alt, -uzaklık),
                };
                ç.yazı(
                    &metin,
                    (
                        *x + seri.etiket.kayma.0,
                        *y + kaydırma + seri.etiket.kayma.1,
                    ),
                    seri.etiket
                        .yatay_hiza
                        .map(|hiza| match hiza {
                            crate::model::stil::YazıYatayHizası::Sol => {
                                crate::cizim::YatayHiza::Sol
                            }
                            crate::model::stil::YazıYatayHizası::Orta => {
                                crate::cizim::YatayHiza::Orta
                            }
                            crate::model::stil::YazıYatayHizası::Sağ => {
                                crate::cizim::YatayHiza::Sağ
                            }
                        })
                        .unwrap_or(crate::cizim::YatayHiza::Orta),
                    seri.etiket
                        .dikey_hiza
                        .map(|hiza| match hiza {
                            crate::model::stil::YazıDikeyHizası::Üst => {
                                crate::cizim::DikeyHiza::Üst
                            }
                            crate::model::stil::YazıDikeyHizası::Orta => {
                                crate::cizim::DikeyHiza::Orta
                            }
                            crate::model::stil::YazıDikeyHizası::Alt => {
                                crate::cizim::DikeyHiza::Alt
                            }
                        })
                        .unwrap_or(hiza),
                    boyut,
                    renk,
                    false,
                );
            }
        }

        // 5) `endLabel`: son görünür veri noktasına bağlı, çizginin akış
        // yönünün dışında duran etiket. LineView bunu sembol etiketinden
        // bağımsız bir `Text` öğesi olarak üretir; showSymbol kapalıyken de
        // görünür kalır.
        if seri.uç_etiketi.göster {
            let son = tepeler.iter().enumerate().rev().find_map(|(sıra, nokta)| {
                let nokta = (*nokta)?;
                let öğe = seri.veri.get(sıra)?;
                let x_değeri = öğe.değer.x().unwrap_or(sıra as f64);
                let değer = öğe.değer.sayı()?;
                (kartezyen.x.pencerede_mi(x_değeri) && kartezyen.y.pencerede_mi(değer))
                    .then_some((sıra, nokta, değer))
            });
            if let Some((sıra, nokta, değer)) = son {
                let öğe = seri.veri.get(sıra);
                let ham = ondalık_kırp(değer);
                let metin = seri
                    .uç_etiketi
                    .biçimleyici
                    .as_ref()
                    .map(|biçimleyici| {
                        biçimleyici.uygula_bağlamla(
                            değer,
                            &ham,
                            seri.ad.as_deref().unwrap_or_default(),
                            öğe.and_then(|öğe| öğe.ad.as_deref()).unwrap_or_default(),
                        )
                    })
                    .unwrap_or(ham);
                ç.yazı(
                    &metin,
                    (
                        nokta.0 + seri.uç_etiketi.uzaklık,
                        uç_etiketi_y.unwrap_or(nokta.1),
                    ),
                    crate::cizim::YatayHiza::Sol,
                    crate::cizim::DikeyHiza::Orta,
                    seri.uç_etiketi.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK),
                    seri.uç_etiketi.yazı.renk.unwrap_or(tema::birincil_metin()),
                    false,
                );
            }
        }
    };

    if ilerleme >= 0.999 {
        gövde(çizici);
    } else {
        // Giriş animasyonu: ECharts'taki gibi soldan sağa açılan kırpma.
        let kırpma = crate::koordinat::Dikdörtgen::yeni(
            alan.x,
            0.0,
            alan.genişlik * ilerleme.clamp(0.0, 1.0),
            çizici.yükseklik(),
        );
        çizici.kırpılı(kırpma, &mut gövde);
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod testler {
    use super::*;

    #[test]
    fn lttb_uçları_korur() {
        let noktalar: Vec<(f32, f32)> = (0..100)
            .map(|i| (i as f32, ((i * 7) % 13) as f32))
            .collect();
        let seçilen = lttb_örnekle(&noktalar, 10);
        assert_eq!(seçilen.len(), 10);
        assert_eq!(seçilen[0], noktalar[0]);
        assert_eq!(*seçilen.last().unwrap(), *noktalar.last().unwrap());
    }

    #[test]
    fn lttb_küçük_veriye_dokunmaz() {
        let noktalar = vec![(0.0, 1.0), (1.0, 2.0)];
        assert_eq!(lttb_örnekle(&noktalar, 10), noktalar);
    }

    #[test]
    fn ortalama_örnekleme_boyutu() {
        let noktalar: Vec<(f32, f32)> = (0..50).map(|i| (i as f32, i as f32)).collect();
        assert_eq!(ortalama_örnekle(&noktalar, 5).len(), 5);
    }
}
