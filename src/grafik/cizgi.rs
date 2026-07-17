//! Çizgi serisi çizimi — `echarts/src/chart/line/LineView.ts` ile
//! `poly.ts` içindeki yumuşak eğri algoritmasının portu.

use crate::cizim::{Yol, ÇizimYüzeyi};
use crate::grafik::{sembol_çiz, çizgi_stili_çöz};
use crate::koordinat::Kartezyen2B;
use crate::model::seri::{Basamak, ÇizgiSerisi, Örnekleme};
use crate::model::stil::EtiketKonumu;
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yerlesim::yigin::YığınAralığı;

/// Boş değerleri `None` olan piksel noktası listesi.
pub type NoktaListesi = Vec<Option<(f32, f32)>>;

/// Serinin `(tepe, taban)` piksel noktalarını üretir; boş değerler `None`.
pub fn nokta_listeleri(
    seri: &ÇizgiSerisi,
    kartezyen: &Kartezyen2B,
    aralıklar: &[YığınAralığı],
) -> (NoktaListesi, NoktaListesi) {
    let mut tepeler = Vec::with_capacity(seri.veri.len());
    let mut tabanlar = Vec::with_capacity(seri.veri.len());
    for (i, öğe) in seri.veri.iter().enumerate() {
        let x_değeri = öğe.değer.x().unwrap_or(i as f64);
        match aralıklar.get(i).copied().flatten() {
            Some((taban, tepe)) => {
                let x = kartezyen.x.veriden_piksele(x_değeri);
                tepeler.push(Some((x, kartezyen.y.veriden_piksele(tepe))));
                tabanlar.push(Some((x, kartezyen.y.veriden_piksele(taban))));
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
        let Some(&şimdiki) = noktalar.get(i) else { break };
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
            let sonu = (((k + 1) as f64 * kova_boyu).floor() as usize).min(n).max(başı + 1);
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
    noktalar: &[Option<(f32, f32)>],
    boşları_bağla: bool,
) -> Vec<Vec<(f32, f32)>> {
    if boşları_bağla {
        let dolu: Vec<(f32, f32)> = noktalar.iter().flatten().copied().collect();
        return if dolu.is_empty() { Vec::new() } else { vec![dolu] };
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

/// Çizgi serisini çizer: alan dolgusu, çizgi, semboller ve etiketler.
#[allow(clippy::too_many_arguments)]
pub fn çizgi_serisi_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &ÇizgiSerisi,
    kartezyen: &Kartezyen2B,
    aralıklar: &[YığınAralığı],
    seri_rengi: Renk,
    ilerleme: f32,
) {
    let (tepeler, tabanlar) = nokta_listeleri(seri, kartezyen, aralıklar);
    let alan = kartezyen.alan;

    let mut gövde = |ç: &mut dyn ÇizimYüzeyi| {
        let mut tepeler_parçalı = parçalara_ayır(&tepeler, seri.boşları_bağla);
        let mut tabanlar_parçalı = parçalara_ayır(&tabanlar, seri.boşları_bağla);
        // Büyük veri örneklemesi: hedef, ızgara genişliği kadar noktadır.
        // Açıkça seçilmemişse, piksel başına birden çok nokta düşen büyük
        // serilerde LTTB kendiliğinden devreye girer (ECharts'ın aşamalı/
        // progressive büyük-veri yolunun tek karelik karşılığı).
        let hedef = (alan.genişlik.max(2.0) as usize).max(2);
        let örnekleme = seri.örnekleme.or_else(|| {
            let en_uzun = tepeler_parçalı.iter().map(Vec::len).max().unwrap_or(0);
            (en_uzun > hedef.saturating_mul(2)).then_some(Örnekleme::Lttb)
        });
        if let Some(örnekleme) = örnekleme {
            let örnekle = |parça: &Vec<(f32, f32)>| match örnekleme {
                Örnekleme::Lttb => lttb_örnekle(parça, hedef),
                Örnekleme::Ortalama => ortalama_örnekle(parça, hedef),
            };
            tepeler_parçalı = tepeler_parçalı.iter().map(örnekle).collect();
            tabanlar_parçalı = tabanlar_parçalı.iter().map(örnekle).collect();
        }

        // 1) Alan dolgusu (çizginin altına).
        if let Some(alan_stili) = &seri.alan_stili {
            let dolgu = alan_stili
                .renk
                .clone()
                .unwrap_or(Dolgu::Düz(seri_rengi))
                .opaklık(alan_stili.opaklık);
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
                let yumuşaklık = if seri.basamak.is_some() { 0.0 } else { seri.yumuşaklık };
                yumuşak_parça_ekle(&mut yol, &üst, yumuşaklık, true);
                let mut alt: Vec<(f32, f32)> = alt_kaynak;
                alt.reverse();
                yumuşak_parça_ekle(&mut yol, &alt, yumuşaklık, false);
                yol.kapat();
                ç.yol_doldur(&yol, &dolgu);
            }
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
            let yumuşaklık = if seri.basamak.is_some() { 0.0 } else { seri.yumuşaklık };
            yumuşak_parça_ekle(&mut yol, &noktalar, yumuşaklık, true);
            ç.yol_çiz(&yol, kalınlık, çizgi_rengi, tür);
        }

        // 3) Semboller.
        if seri.sembol_göster && seri.sembol != crate::model::seri::Sembol::Yok {
            for nokta in tepeler.iter().flatten() {
                sembol_çiz(ç, seri.sembol, *nokta, seri.sembol_boyutu, seri_rengi);
            }
        }

        // 4) Değer etiketleri.
        if seri.etiket.göster {
            let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            let renk = seri.etiket.yazı.renk.unwrap_or(tema::birincil_metin());
            for (i, nokta) in tepeler.iter().enumerate() {
                let Some((x, y)) = nokta else { continue };
                let Some(öğe) = seri.veri.get(i) else { continue };
                let Some(değer) = öğe.değer.sayı() else { continue };
                let metin = match &seri.etiket.biçimleyici {
                    Some(b) => b.uygula(değer, &binlik_ayır(değer)),
                    None => binlik_ayır(değer),
                };
                let (hiza, kaydırma) = match seri.etiket.konum {
                    EtiketKonumu::Alt => (crate::cizim::DikeyHiza::Üst, 6.0),
                    _ => (crate::cizim::DikeyHiza::Alt, -6.0),
                };
                ç.yazı(
                    &metin,
                    (*x, *y + kaydırma),
                    crate::cizim::YatayHiza::Orta,
                    hiza,
                    boyut,
                    renk,
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
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod testler {
    use super::*;

    #[test]
    fn lttb_uçları_korur() {
        let noktalar: Vec<(f32, f32)> =
            (0..100).map(|i| (i as f32, ((i * 7) % 13) as f32)).collect();
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
