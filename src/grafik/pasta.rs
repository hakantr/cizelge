//! Pasta serisi çizimi — `echarts/src/chart/pie` (yerleşim + görünüm)
//! karşılığı.

use std::collections::{HashMap, HashSet};

use crate::cizim::donusum::AfinMatris;
use crate::cizim::yuzey::yuvarlatılmış_dilim_yolu;
use crate::cizim::{DikeyHiza, YatayHiza, Yol, ÇizimYüzeyi};
use crate::koordinat::Dikdörtgen;
use crate::model::deger::VeriDeğeri;
use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::{EtiketYerleşimParametreleri, GülTürü, PastaSerisi, Seri};
use crate::model::stil::{
    Biçimleyici, DışEtiketHizası, Etiket, EtiketDöndürme, EtiketKonumu, YazıStili,
    zengin_metin_içeriği,
};
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;

/// Yerleşimi hesaplanmış bir pasta dilimi.
#[derive(Clone, Debug)]
pub struct Dilim {
    /// Veri dizisindeki sıra (renk ataması için özgün sıra).
    pub sıra: usize,
    pub ad: String,
    pub değer: f64,
    /// Dataset'ten korunan adlandırılmış ham boyutlar (`{@dimension}`).
    pub boyutlar: Vec<(String, VeriDeğeri)>,
    /// Görünür toplam içindeki pay `0..=1`.
    pub oran: f64,
    /// `getPercentSeats` (en büyük kalan) ile seri hassasiyetinde ayrılmış
    /// yüzde. Etiket/tooltip toplamlarının tam `%100` kalmasını sağlar.
    pub yüzde: f64,
    /// Ekran koordinatında başlangıç/bitiş açısı (radyan).
    pub açı0: f32,
    pub açı1: f32,
    /// Yerleşim kararlarında f32 sınırındaki dikey/yatay yön terslenmesini
    /// önleyen, hesaplandığı hassasiyette orta açı.
    pub orta_açı: f64,
    pub iç_yarıçap: f32,
    pub dış_yarıçap: f32,
    pub dolgu: Dolgu,
    pub renk: Renk,
    pub merkez: (f32, f32),
    pub görünüm_alanı: Dikdörtgen,
    pub etiket_göster: bool,
    pub yüzde_hassasiyeti: u8,
}

impl Dilim {
    /// Nokta dilimin içinde mi (ipucu isabeti için)?
    pub fn içeriyor_mu(&self, nokta: (f32, f32)) -> bool {
        let dx = nokta.0 - self.merkez.0;
        let dy = nokta.1 - self.merkez.1;
        let uzaklık = (dx * dx + dy * dy).sqrt();
        if uzaklık < self.iç_yarıçap || uzaklık > self.dış_yarıçap {
            return false;
        }
        let açı = dy.atan2(dx);
        let tau = std::f32::consts::TAU;
        let (a0, a1) = if self.açı1 >= self.açı0 {
            (self.açı0, self.açı1)
        } else {
            (self.açı1, self.açı0)
        };
        let göreli = (açı - a0).rem_euclid(tau);
        göreli <= (a1 - a0)
    }
}

/// Pasta yerleşimi — `pieLayout.ts` karşılığı. `kapalı` gösterge ile
/// gizlenen dilim adlarıdır; `ilerleme` giriş animasyonunun açı çarpanıdır.
pub fn pasta_yerleşimi(
    seri: &PastaSerisi,
    seçenekler: &GrafikSeçenekleri,
    alan: Dikdörtgen,
    kapalı: &HashSet<String>,
    ilerleme: f32,
) -> Vec<Dilim> {
    pasta_yerleşimi_merkezle(seri, seçenekler, alan, kapalı, ilerleme, None)
}

/// Takvim gibi bir koordinat sisteminin veri noktasından çözdüğü kesin
/// piksel merkezini kullanarak pasta yerleşimini kurar.
pub fn pasta_yerleşimi_merkezle(
    seri: &PastaSerisi,
    seçenekler: &GrafikSeçenekleri,
    alan: Dikdörtgen,
    kapalı: &HashSet<String>,
    ilerleme: f32,
    merkez_yaması: Option<(f32, f32)>,
) -> Vec<Dilim> {
    let alan = pasta_görünüm_alanı(seri, alan);
    let merkez = merkez_yaması.unwrap_or_else(|| {
        (
            alan.x + seri.merkez.0.çöz(alan.genişlik),
            alan.y + seri.merkez.1.çöz(alan.yükseklik),
        )
    });
    // ECharts: yüzde yarıçaplar görünür alanın kısa kenarının yarısına
    // oranlıdır.
    let taban_yarıçap = alan.genişlik.min(alan.yükseklik) / 2.0;
    let iç = seri.yarıçap.0.çöz(taban_yarıçap);
    let dış = seri.yarıçap.1.çöz(taban_yarıçap);

    // `negativeDataFilter`: negatif ve sayısal olmayan değerler veri
    // listesinden çıkar; sıfırlar geçerli kalır ve zero-sum kuralına katılır.
    let görünürler: Vec<(usize, &crate::model::deger::VeriÖğesi, f64)> = seri
        .veri
        .iter()
        .enumerate()
        .filter_map(|(sıra, ö)| {
            let ad = ö.ad.clone().unwrap_or_default();
            let değer = ö.değer.sayı()?;
            (!kapalı.contains(&ad) && değer.is_finite() && değer >= 0.0).then_some((sıra, ö, değer))
        })
        .collect();

    if görünürler.is_empty() {
        return Vec::new();
    }
    let toplam: f64 = görünürler.iter().map(|(_, _, değer)| *değer).sum();
    let yüzdeler = yüzde_koltukları(
        &görünürler
            .iter()
            .map(|(_, _, değer)| *değer)
            .collect::<Vec<_>>(),
        seri.yüzde_hassasiyeti,
    );
    let en_büyük = görünürler
        .iter()
        .map(|(_, _, değer)| *değer)
        .fold(0.0_f64, f64::max);
    let veri_kapsamı = görünürler.iter().fold(
        [f64::INFINITY, f64::NEG_INFINITY],
        |[en_az, en_çok], (_, _, değer)| [en_az.min(*değer), en_çok.max(*değer)],
    );
    let görsel_kapsam = seçenekler
        .görsel_eşleme
        .as_ref()
        .map(|eşleme| eşleme.kapsam_çöz(veri_kapsamı));

    let tau = std::f64::consts::TAU;
    let yön: f64 = if seri.saat_yönünde { 1.0 } else { -1.0 };
    // ECharts `startAngle: 90` üstten başlar; ekran koordinatında -90°'dir.
    let başlangıç = -(seri.başlangıç_açısı as f64).to_radians();
    let bitiş = seri
        .bitiş_açısı
        .map(|derece| -(derece as f64).to_radians())
        .unwrap_or(başlangıç + yön * tau);
    let (başlangıç, bitiş) = açıları_normalleştir(başlangıç, bitiş, seri.saat_yönünde);
    let açı_aralığı = (bitiş - başlangıç).abs();
    let dolgu_açısı = (seri.dolgu_açısı as f64).to_radians().max(0.0);
    let en_küçük_açı = (seri.en_küçük_açı as f64).to_radians().max(0.0);
    let en_küçük_ve_dolgu = en_küçük_açı + dolgu_açısı;

    // Resmi `pieLayout.ts` iki geçişlidir: önce minAngle ile sabitlenen
    // dilimler ayrılır, sonra kalan açı diğer değerlere yeniden dağıtılır.
    let birim = tau
        / if toplam > 0.0 {
            toplam
        } else {
            görünürler.len() as f64
        };
    let mut açılar = Vec::with_capacity(görünürler.len());
    let mut kalan_açı = açı_aralığı;
    let mut büyük_değer_toplamı = 0.0_f64;
    for (_, _, değer) in &görünürler {
        let mut açı = if seri.gül_türü == Some(GülTürü::Alan) {
            açı_aralığı / görünürler.len() as f64
        } else if toplam == 0.0 && seri.sıfır_toplamı_göster {
            birim
        } else {
            *değer * birim
        };
        if açı < en_küçük_ve_dolgu {
            açı = en_küçük_ve_dolgu;
            kalan_açı -= en_küçük_ve_dolgu;
        } else {
            büyük_değer_toplamı += *değer;
        }
        açılar.push(açı);
    }

    if kalan_açı < tau {
        if kalan_açı <= 1e-3 {
            açılar.fill(açı_aralığı / görünürler.len() as f64);
        } else if büyük_değer_toplamı > 0.0 {
            let kalan_birim = kalan_açı / büyük_değer_toplamı;
            for ((_, _, değer), açı) in görünürler.iter().zip(&mut açılar) {
                if (*açı - en_küçük_ve_dolgu).abs() > 1e-12 {
                    *açı = *değer * kalan_birim;
                }
            }
        }
    }

    let mut açı = başlangıç;
    let ilerleme = ilerleme.clamp(0.0, 1.0) as f64;
    let yarım_dolgu = yön * dolgu_açısı / 2.0;

    let mut dilimler = Vec::with_capacity(görünürler.len());
    for (((sıra, öğe, değer), pay), yüzde) in görünürler.into_iter().zip(açılar).zip(yüzdeler)
    {
        let oran = if toplam > 0.0 { değer / toplam } else { 0.0 };
        let geometrik_bitiş = açı + yön * pay;
        let (gerçek_başlangıç, gerçek_bitiş) = if dolgu_açısı > pay {
            let orta = açı + yön * pay / 2.0;
            (orta, orta)
        } else {
            (açı + yarım_dolgu, geometrik_bitiş - yarım_dolgu)
        };

        let dış_dilim = match seri.gül_türü {
            None => dış,
            Some(GülTürü::Yarıçap | GülTürü::Alan) => {
                iç + (dış - iç) * (değer / en_büyük.max(1e-12)) as f32
            }
        };

        // İlk render expansion animasyonu bütün şekilleri ilk startAngle'dan
        // kendi son açılarına taşır; yalnız yay genişliğini çarpmak start
        // konumlarını yanlış bırakırdı.
        let çizim_açısı0 = başlangıç + (gerçek_başlangıç - başlangıç) * ilerleme;
        let çizim_açısı1 = başlangıç + (gerçek_bitiş - başlangıç) * ilerleme;
        let orta_açı = (gerçek_başlangıç + gerçek_bitiş) / 2.0;
        let merkez = if öğe.seçili {
            (
                merkez.0 + orta_açı.cos() as f32 * seri.seçili_uzaklığı,
                merkez.1 + orta_açı.sin() as f32 * seri.seçili_uzaklığı,
            )
        } else {
            merkez
        };

        let palet_sırası = pasta_palet_sırası(seçenekler, seri, sıra, &dilim_adı(öğe, sıra));
        let taban_dolgu = öğe
            .stil
            .as_ref()
            .and_then(|s| s.renk.as_ref())
            .or(seri.öğe_stili.renk.as_ref())
            .cloned()
            .unwrap_or_else(|| Dolgu::Düz(seçenekler.palet_rengi(palet_sırası)));
        let taban_renk = taban_dolgu.temsilî();
        let (dolgu, renk) = seçenekler
            .görsel_eşleme
            .as_ref()
            .zip(görsel_kapsam)
            .map(|(eşleme, kapsam)| {
                let renk = eşleme.renk_çöz_tabanla(değer, kapsam, taban_renk);
                (Dolgu::Düz(renk), renk)
            })
            .unwrap_or((taban_dolgu, taban_renk));

        dilimler.push(Dilim {
            sıra,
            ad: öğe.ad.clone().unwrap_or_else(|| format!("{sıra}")),
            değer,
            boyutlar: öğe.boyutlar.clone(),
            oran,
            yüzde,
            açı0: çizim_açısı0 as f32,
            açı1: çizim_açısı1 as f32,
            orta_açı,
            iç_yarıçap: iç,
            dış_yarıçap: dış_dilim,
            dolgu,
            renk,
            merkez,
            görünüm_alanı: alan,
            etiket_göster: (gerçek_bitiş - gerçek_başlangıç).abs().to_degrees()
                >= seri.en_küçük_etiket_açısı as f64,
            yüzde_hassasiyeti: seri.yüzde_hassasiyeti,
        });
        açı = geometrik_bitiş;
    }
    dilimler
}

fn dilim_adı(öğe: &crate::model::deger::VeriÖğesi, sıra: usize) -> String {
    öğe.ad.clone().unwrap_or_else(|| format!("{sıra}"))
}

/// ECharts pie verileri renk paletini veri adıyla grafik çapında paylaşır.
/// Aynı adlı dilim sonraki pasta serilerinde aynı rengi alır; yeni adlar
/// önceki pasta serilerinin tükettiği sıradan devam eder.
fn pasta_palet_sırası(
    seçenekler: &GrafikSeçenekleri,
    hedef: &PastaSerisi,
    hedef_sıra: usize,
    hedef_ad: &str,
) -> usize {
    let mut ad_sıraları = HashMap::<String, usize>::new();
    let mut sonraki = 0_usize;
    for seri in &seçenekler.seriler {
        let Seri::Pasta(pasta) = seri else {
            continue;
        };
        for (sıra, öğe) in pasta.veri.iter().enumerate() {
            let ad = dilim_adı(öğe, sıra);
            let renk_sırası = *ad_sıraları.entry(ad.clone()).or_insert_with(|| {
                let ayrılan = sonraki;
                sonraki += 1;
                ayrılan
            });
            if std::ptr::eq(pasta, hedef) && sıra == hedef_sıra && ad == hedef_ad {
                return renk_sırası;
            }
        }
        if std::ptr::eq(pasta, hedef) {
            break;
        }
    }
    hedef_sıra
}

/// zrender `normalizeArcAngles` eşdeğeri; açık `endAngle`ın saat yönü
/// semantiğini ve en fazla tek tam tur sınırını korur.
fn açıları_normalleştir(
    mut başlangıç: f64, mut bitiş: f64, saat_yönünde: bool
) -> (f64, f64) {
    let tau = std::f64::consts::TAU;
    let yeni_başlangıç = başlangıç.rem_euclid(tau);
    bitiş += yeni_başlangıç - başlangıç;
    başlangıç = yeni_başlangıç;
    if saat_yönünde {
        if bitiş - başlangıç >= tau {
            bitiş = başlangıç + tau;
        } else if başlangıç > bitiş {
            bitiş = başlangıç + (tau - (başlangıç - bitiş).rem_euclid(tau));
        }
    } else if başlangıç - bitiş >= tau {
        bitiş = başlangıç - tau;
    } else if başlangıç < bitiş {
        bitiş = başlangıç - (tau - (bitiş - başlangıç).rem_euclid(tau));
    }
    (başlangıç, bitiş)
}

/// Geçerli veri kalmadığında ECharts `showEmptyCircle` sektörü.
pub fn boş_pasta_çiz(çizici: &mut dyn ÇizimYüzeyi, seri: &PastaSerisi, alan: Dikdörtgen) {
    boş_pasta_çiz_merkezle(çizici, seri, alan, None);
}

pub fn boş_pasta_çiz_merkezle(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &PastaSerisi,
    alan: Dikdörtgen,
    merkez_yaması: Option<(f32, f32)>,
) {
    if !seri.boş_daire_göster {
        return;
    }
    let alan = pasta_görünüm_alanı(seri, alan);
    let merkez = merkez_yaması.unwrap_or_else(|| {
        (
            alan.x + seri.merkez.0.çöz(alan.genişlik),
            alan.y + seri.merkez.1.çöz(alan.yükseklik),
        )
    });
    let taban_yarıçap = alan.genişlik.min(alan.yükseklik) / 2.0;
    let iç = seri.yarıçap.0.çöz(taban_yarıçap);
    let dış = seri.yarıçap.1.çöz(taban_yarıçap);
    let başlangıç = -(seri.başlangıç_açısı as f64).to_radians();
    let yön = if seri.saat_yönünde { 1.0 } else { -1.0 };
    let bitiş = seri
        .bitiş_açısı
        .map(|derece| -(derece as f64).to_radians())
        .unwrap_or(başlangıç + yön * std::f64::consts::TAU);
    let (açı0, açı1) = açıları_normalleştir(başlangıç, bitiş, seri.saat_yönünde);
    let dolgu = seri
        .boş_daire_stili
        .renk
        .clone()
        .unwrap_or_else(|| Dolgu::Düz(Renk::onaltılık(0xd3d3d3)))
        .opaklık(seri.boş_daire_stili.opaklık.unwrap_or(1.0));
    let kenarlık = seri
        .boş_daire_stili
        .kenarlık_rengi
        .map(|renk| (seri.boş_daire_stili.kenarlık_kalınlığı, renk));
    çizici.dilim(merkez, iç, dış, açı0 as f32, açı1 as f32, &dolgu, kenarlık);
}

/// `BoxLayoutOptionMixin`in pie için görünüm dikdörtgeni.
pub fn pasta_görünüm_alanı(seri: &PastaSerisi, ana: Dikdörtgen) -> Dikdörtgen {
    let sol = seri.sol.çöz(ana.genişlik).max(0.0);
    let sağ = seri.sağ.çöz(ana.genişlik).max(0.0);
    let üst = seri.üst.çöz(ana.yükseklik).max(0.0);
    let alt = seri.alt.çöz(ana.yükseklik).max(0.0);
    let genişlik = seri
        .genişlik
        .map(|değer| değer.çöz(ana.genişlik))
        .unwrap_or(ana.genişlik - sol - sağ)
        .max(0.0);
    let yükseklik = seri
        .yükseklik
        .map(|değer| değer.çöz(ana.yükseklik))
        .unwrap_or(ana.yükseklik - üst - alt)
        .max(0.0);
    Dikdörtgen::yeni(ana.x + sol, ana.y + üst, genişlik, yükseklik)
}

/// Pasta serisini çizer; `vurgulu` ipucuyla öne çıkarılan dilimin sırasıdır.
pub fn pasta_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &PastaSerisi,
    dilimler: &[Dilim],
    vurgulu: Option<usize>,
    genel_etiket_kutuları: &mut Vec<Dikdörtgen>,
) {
    // 1) Dilimler.
    for (i, dilim) in dilimler.iter().enumerate() {
        // ECharts `emphasis.scaleSize` davranışı: vurgulu dilim büyür.
        let dış = if vurgulu == Some(i) {
            dilim.dış_yarıçap + 6.0
        } else {
            dilim.dış_yarıçap
        };
        let opaklık = seri.öğe_stili.opaklık.unwrap_or(1.0);
        let kenarlık = seri.öğe_stili.kenarlık_rengi.map(|r| {
            (
                seri.öğe_stili.kenarlık_kalınlığı.max(1.0),
                r.opaklık(opaklık),
            )
        });
        if seri.öğe_stili.gölge_bulanıklığı > 0.0
            && let Some(gölge_rengi) = seri.öğe_stili.gölge_rengi
        {
            let yol = yuvarlatılmış_dilim_yolu(
                dilim.merkez,
                dilim.iç_yarıçap,
                dış,
                dilim.açı0,
                dilim.açı1,
                seri.öğe_stili.kenarlık_yarıçapı,
            );
            çizici.yol_gölgesi(
                &yol,
                gölge_rengi,
                seri.öğe_stili.gölge_bulanıklığı,
                seri.öğe_stili.gölge_kayması,
            );
        }
        çizici.yuvarlatılmış_dilim(
            dilim.merkez,
            dilim.iç_yarıçap,
            dış,
            dilim.açı0,
            dilim.açı1,
            seri.öğe_stili.kenarlık_yarıçapı,
            &dilim.dolgu.opaklık(opaklık),
            kenarlık,
        );
    }

    // 2) Etiketler ve etiket çizgileri.
    if !seri.etiket.göster {
        return;
    }
    let boyut = seri.etiket.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
    // `pie/labelLayout.ts`: roseType dilimlerinin gerçek dış yarıçapları
    // farklı olsa da kırık çizginin dirseği seri `r + length` elipsine
    // taşınır. Böylece bütün dış etiketler aynı sanal çemberi paylaşır.
    let seri_dış_yarıçapı = dilimler
        .iter()
        .map(|dilim| dilim.dış_yarıçap)
        .fold(0.0_f32, f32::max);
    let mut dış_etiketler = Vec::new();
    for (dilim_sırası, dilim) in dilimler.iter().enumerate() {
        if !dilim.etiket_göster {
            continue;
        }
        let orta_açı = dilim.orta_açı;
        let orta_kosinüs = orta_açı.cos() as f32;
        let orta_sinüs = orta_açı.sin() as f32;
        let ham_metin = pasta_etiket_zengin_metni(seri, dilim);
        let metin = zengin_metin_içeriği(ham_metin.clone());
        match seri.etiket.konum {
            EtiketKonumu::Merkez => {
                let renk = seri.etiket.yazı.renk.unwrap_or(tema::birincil_metin());
                zengin_etiketi_yaz(
                    çizici,
                    &ham_metin,
                    &seri.etiket,
                    dilim.merkez,
                    YatayHiza::Orta,
                    renk,
                    etiket_dönüş_açısı(seri.etiket.döndürme, orta_açı as f32, false),
                );
            }
            EtiketKonumu::İç => {
                // `pie/labelLayout.ts`, iç etiketi dilim yarıçaplarının
                // ortasından normal yönünde 3 px daha dışarı taşır.
                let yarıçap = (dilim.iç_yarıçap + dilim.dış_yarıçap) / 2.0 + 3.0;
                let konum = (
                    dilim.merkez.0 + yarıçap * orta_kosinüs,
                    dilim.merkez.1 + yarıçap * orta_sinüs,
                );
                let opaklık = seri.öğe_stili.opaklık.unwrap_or(1.0);
                let (renk, kontur) = match seri.etiket.yazı.renk {
                    Some(renk) => (renk, None),
                    None => {
                        let (metin, kontur) = dilim.dolgu.zrender_iç_etiket_stili(tema::koyu_mu());
                        (
                            metin.opaklık(opaklık),
                            kontur.map(|kontur| kontur.opaklık(opaklık)),
                        )
                    }
                };
                zengin_etiketi_konturlu_yaz(
                    çizici,
                    &ham_metin,
                    &seri.etiket,
                    konum,
                    YatayHiza::Orta,
                    renk,
                    kontur,
                    etiket_dönüş_açısı(seri.etiket.döndürme, orta_açı as f32, true),
                );
            }
            _ => {
                // Dış etiket: dilimden çıkan kırık çizgi + metin.
                let sağda = orta_kosinüs >= 0.0;
                let u1 = seri.etiket_çizgisi.uzunluk1;
                let u2 = seri.etiket_çizgisi.uzunluk2;
                let b0 = (
                    dilim.merkez.0 + dilim.dış_yarıçap * orta_kosinüs,
                    dilim.merkez.1 + dilim.dış_yarıçap * orta_sinüs,
                );
                let b1 = (
                    dilim.merkez.0 + (seri_dış_yarıçapı + u1) * orta_kosinüs,
                    dilim.merkez.1 + (seri_dış_yarıçapı + u1) * orta_sinüs,
                );
                let b2 = (b1.0 + if sağda { u2 } else { -u2 }, b1.1);
                let görünüm = dilim.görünüm_alanı;
                let kenar = kenar_uzaklığını_çöz(seri, görünüm);
                let (konum, hiza) = match seri.etiket.dış_hiza {
                    DışEtiketHizası::Kenar if sağda => {
                        ((görünüm.sağ() - kenar, b2.1), YatayHiza::Sağ)
                    }
                    DışEtiketHizası::Kenar => ((görünüm.x + kenar, b2.1), YatayHiza::Sol),
                    _ if sağda => ((b2.0 + seri.etiket.çizgi_uzaklığı, b2.1), YatayHiza::Sol),
                    _ => ((b2.0 - seri.etiket.çizgi_uzaklığı, b2.1), YatayHiza::Sağ),
                };
                let metin_ölçüsü = zengin_metin_ölç(çizici, &ham_metin, &seri.etiket);
                let yarım_boşluk = seri.etiket.en_küçük_boşluk / 2.0;
                dış_etiketler.push(DışEtiketYerleşimi {
                    dilim_sırası,
                    tam_ham_metin: ham_metin.clone(),
                    ham_metin,
                    tam_metin: metin.clone(),
                    metin,
                    konum,
                    hiza,
                    noktalar: [b0, b1, b2],
                    orta_açı,
                    metin_genişliği: metin_ölçüsü.0,
                    metin_yüksekliği: metin_ölçüsü.1,
                    dikdörtgen_y: konum.1 - metin_ölçüsü.1 / 2.0 - yarım_boşluk,
                    dikdörtgen_yüksekliği: metin_ölçüsü.1 + seri.etiket.en_küçük_boşluk,
                });
            }
        }
    }
    dış_etiketleri_yerleştir_ve_çiz(
        çizici,
        seri,
        dilimler,
        &mut dış_etiketler,
        seri_dış_yarıçapı,
        boyut,
        genel_etiket_kutuları,
    );
}

#[cfg(test)]
fn pasta_etiket_metni(seri: &PastaSerisi, dilim: &Dilim) -> String {
    zengin_metin_içeriği(pasta_etiket_zengin_metni(seri, dilim))
}

fn pasta_etiket_zengin_metni(seri: &PastaSerisi, dilim: &Dilim) -> String {
    let Some(biçimleyici) = &seri.etiket.biçimleyici else {
        return dilim.ad.clone();
    };
    let Biçimleyici::Şablon(şablon) = biçimleyici else {
        return biçimleyici.uygula(dilim.değer, &dilim.ad);
    };

    let mut şablon = şablon.clone();
    let yüzde = yüzde_metni(dilim.yüzde, dilim.yüzde_hassasiyeti);
    şablon = şablon.replace("{d}", &yüzde);
    şablon = dataset_yer_tutucularını_çöz(&şablon, &dilim.boyutlar);
    Biçimleyici::Şablon(şablon).uygula_bağlamla_zengin(
        dilim.değer,
        &crate::yardimci::bicim::ondalık_kırp(dilim.değer),
        seri.ad.as_deref().unwrap_or_default(),
        &dilim.ad,
    )
}

#[derive(Clone)]
struct ZenginKoşu {
    metin: String,
    yazı: YazıStili,
}

struct ZenginSatır {
    koşular: Vec<ZenginKoşu>,
    genişlik: f32,
    yükseklik: f32,
}

struct ZenginMetinYerleşimi {
    satırlar: Vec<ZenginSatır>,
    genişlik: f32,
    yükseklik: f32,
}

fn yazı_stilini_birleştir(taban: &YazıStili, yama: &YazıStili) -> YazıStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk = yama.renk;
    }
    if yama.boyut.is_some() {
        sonuç.boyut = yama.boyut;
    }
    if yama.satır_yüksekliği.is_some() {
        sonuç.satır_yüksekliği = yama.satır_yüksekliği;
    }
    if yama.kalınlık_belirtildi {
        sonuç.kalın = yama.kalın;
        sonuç.kalınlık_belirtildi = true;
    }
    if yama.aile.is_some() {
        sonuç.aile.clone_from(&yama.aile);
    }
    sonuç
}

fn zengin_satırı_çöz(satır: &str, etiket: &Etiket) -> Vec<ZenginKoşu> {
    let mut taban = etiket.yazı.clone();
    if taban.boyut.is_none() {
        taban.boyut = Some(tema::YAZI_KÜÇÜK);
    }
    let mut koşular = Vec::new();
    let mut kalan = satır;
    while let Some(açılış) = kalan.find('{') {
        if açılış > 0
            && let Some(önek) = kalan.get(..açılış)
        {
            koşular.push(ZenginKoşu {
                metin: önek.to_owned(),
                yazı: taban.clone(),
            });
        }
        let Some(açılış_sonrası) = kalan.get(açılış + 1..) else {
            break;
        };
        let Some(kapanış_göreli) = açılış_sonrası.find('}') else {
            koşular.push(ZenginKoşu {
                metin: kalan.get(açılış..).unwrap_or_default().to_owned(),
                yazı: taban.clone(),
            });
            kalan = "";
            break;
        };
        let kapanış = açılış + 1 + kapanış_göreli;
        let belirteç = kalan.get(açılış + 1..kapanış).unwrap_or_default();
        let Some((ad, içerik)) = belirteç.split_once('|') else {
            koşular.push(ZenginKoşu {
                metin: kalan.get(açılış..=kapanış).unwrap_or_default().to_owned(),
                yazı: taban.clone(),
            });
            kalan = kalan.get(kapanış + 1..).unwrap_or_default();
            continue;
        };
        let yazı = etiket
            .zengin
            .get(ad)
            .map(|yama| yazı_stilini_birleştir(&taban, yama))
            .unwrap_or_else(|| taban.clone());
        koşular.push(ZenginKoşu {
            metin: içerik.to_owned(),
            yazı,
        });
        kalan = kalan.get(kapanış + 1..).unwrap_or_default();
    }
    if !kalan.is_empty() || koşular.is_empty() {
        koşular.push(ZenginKoşu {
            metin: kalan.to_owned(),
            yazı: taban,
        });
    }
    koşular
}

fn zengin_metin_yerleşimi(
    çizici: &dyn ÇizimYüzeyi,
    ham_metin: &str,
    etiket: &Etiket,
) -> ZenginMetinYerleşimi {
    let mut satırlar = Vec::new();
    let mut toplam_yükseklik = 0.0_f32;
    let mut en_büyük_genişlik = 0.0_f32;
    for satır in ham_metin.split('\n') {
        let koşular = zengin_satırı_çöz(satır, etiket);
        let genişlik = koşular
            .iter()
            .map(|koşu| {
                çizici
                    .yazı_ölç(&koşu.metin, koşu.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK))
                    .0
            })
            .sum();
        let doğal_yükseklik = koşular
            .iter()
            .map(|koşu| {
                koşu
                    .yazı
                    .satır_yüksekliği
                    .unwrap_or(koşu.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK))
            })
            .fold(0.0_f32, f32::max);
        let yükseklik = etiket
            .yazı
            .satır_yüksekliği
            .unwrap_or(doğal_yükseklik.max(tema::YAZI_KÜÇÜK));
        en_büyük_genişlik = en_büyük_genişlik.max(genişlik);
        toplam_yükseklik += yükseklik;
        satırlar.push(ZenginSatır {
            koşular,
            genişlik,
            yükseklik,
        });
    }
    ZenginMetinYerleşimi {
        satırlar,
        genişlik: en_büyük_genişlik,
        yükseklik: toplam_yükseklik,
    }
}

fn zengin_metin_ölç(çizici: &dyn ÇizimYüzeyi, ham_metin: &str, etiket: &Etiket) -> (f32, f32) {
    let yerleşim = zengin_metin_yerleşimi(çizici, ham_metin, etiket);
    (yerleşim.genişlik, yerleşim.yükseklik)
}

pub(crate) fn zengin_etiketi_yaz(
    çizici: &mut dyn ÇizimYüzeyi,
    ham_metin: &str,
    etiket: &Etiket,
    konum: (f32, f32),
    hiza: YatayHiza,
    varsayılan_renk: Renk,
    dönüş: f32,
) {
    zengin_etiketi_konturlu_yaz(
        çizici,
        ham_metin,
        etiket,
        konum,
        hiza,
        varsayılan_renk,
        None,
        dönüş,
    );
}

#[allow(clippy::too_many_arguments)]
fn zengin_etiketi_konturlu_yaz(
    çizici: &mut dyn ÇizimYüzeyi,
    ham_metin: &str,
    etiket: &Etiket,
    konum: (f32, f32),
    hiza: YatayHiza,
    varsayılan_renk: Renk,
    varsayılan_kontur: Option<Renk>,
    dönüş: f32,
) {
    let yerleşim = zengin_metin_yerleşimi(çizici, ham_metin, etiket);
    let mut satır_y = -yerleşim.yükseklik / 2.0;
    let konum = (konum.0 + etiket.kayma.0, konum.1 + etiket.kayma.1);
    let dönüşüm = AfinMatris::ötele(konum.0, konum.1).çarp(AfinMatris::döndür(dönüş));
    for satır in yerleşim.satırlar {
        let mut koşu_x = match hiza {
            YatayHiza::Sol => 0.0,
            YatayHiza::Orta => -satır.genişlik / 2.0,
            YatayHiza::Sağ => -satır.genişlik,
        };
        let koşu_y = satır_y + satır.yükseklik / 2.0;
        for koşu in satır.koşular {
            let boyut = koşu.yazı.boyut.unwrap_or(tema::YAZI_KÜÇÜK);
            let genişlik = çizici.yazı_ölç(&koşu.metin, boyut).0;
            let renk = koşu.yazı.renk.unwrap_or(varsayılan_renk);
            let kontur = koşu
                .yazı
                .renk
                .is_none()
                .then_some(varsayılan_kontur)
                .flatten();
            if let Some(kontur) = kontur {
                çizici.dönüşümlü_konturlu_yazı(
                    &koşu.metin,
                    (koşu_x, koşu_y),
                    YatayHiza::Sol,
                    DikeyHiza::Orta,
                    boyut,
                    renk,
                    koşu.yazı.kalın,
                    kontur,
                    2.0,
                    dönüşüm,
                );
            } else if dönüş.abs() <= 1e-6 {
                çizici.yazı(
                    &koşu.metin,
                    (konum.0 + koşu_x, konum.1 + koşu_y),
                    YatayHiza::Sol,
                    DikeyHiza::Orta,
                    boyut,
                    renk,
                    koşu.yazı.kalın,
                );
            } else {
                çizici.dönüşümlü_yazı(
                    &koşu.metin,
                    (koşu_x, koşu_y),
                    YatayHiza::Sol,
                    DikeyHiza::Orta,
                    boyut,
                    renk,
                    koşu.yazı.kalın,
                    dönüşüm,
                );
            }
            koşu_x += genişlik;
        }
        satır_y += satır.yükseklik;
    }
}

fn yüzde_metni(yüzde: f64, hassasiyet: u8) -> String {
    let çarpan = 10_f64.powi(hassasiyet as i32);
    crate::yardimci::bicim::ondalık_kırp((yüzde * çarpan).round() / çarpan)
}

/// ECharts `util/number.getPercentSeats`: Hamilton/en büyük kalan yöntemi.
/// Her öğe önce aşağı yuvarlanır, eksik yüzde puanları kalan sırasıyla
/// dağıtılır; böylece görüntülenen değerler seçilen hassasiyette tam 100 eder.
fn yüzde_koltukları(değerler: &[f64], hassasiyet: u8) -> Vec<f64> {
    let toplam: f64 = değerler
        .iter()
        .copied()
        .filter(|değer| değer.is_finite())
        .sum();
    if toplam == 0.0 {
        return vec![0.0; değerler.len()];
    }
    let basamak = 10_u64.saturating_pow(u32::from(hassasiyet));
    let hedef = basamak.saturating_mul(100);
    let oylar = değerler
        .iter()
        .map(|değer| {
            if değer.is_finite() {
                *değer / toplam * hedef as f64
            } else {
                0.0
            }
        })
        .collect::<Vec<_>>();
    let mut koltuklar = oylar
        .iter()
        .map(|oy| oy.floor().max(0.0) as u64)
        .collect::<Vec<_>>();
    let mut kalanlar = oylar
        .iter()
        .zip(&koltuklar)
        .map(|(oy, koltuk)| oy - *koltuk as f64)
        .collect::<Vec<_>>();
    let mut toplam_koltuk: u64 = koltuklar.iter().sum();
    while toplam_koltuk < hedef {
        let Some((sıra, _)) = kalanlar
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        else {
            break;
        };
        let Some(koltuk) = koltuklar.get_mut(sıra) else {
            break;
        };
        *koltuk = koltuk.saturating_add(1);
        if let Some(kalan) = kalanlar.get_mut(sıra) {
            // Resmî gerçekleme aynı öğenin ancak bütün pozitif kalanlar
            // tüketildikten sonra yeniden seçilebilmesi için sıfırlar.
            *kalan = 0.0;
        }
        toplam_koltuk += 1;
    }
    koltuklar
        .into_iter()
        .map(|koltuk| koltuk as f64 / basamak.max(1) as f64)
        .collect()
}

fn dataset_yer_tutucularını_çöz(şablon: &str, boyutlar: &[(String, VeriDeğeri)]) -> String {
    let mut sonuç = şablon.to_owned();
    let mut tarama = 0usize;
    while let Some(göreli) = sonuç.get(tarama..).and_then(|kalan| kalan.find("{@")) {
        let başlangıç = tarama + göreli;
        let Some(kapanış_göreli) = sonuç.get(başlangıç + 2..).and_then(|kalan| kalan.find('}'))
        else {
            break;
        };
        let kapanış = başlangıç + 2 + kapanış_göreli;
        let seçici = sonuç.get(başlangıç + 2..kapanış).unwrap_or_default().trim();
        let değer = seçici
            .strip_prefix('[')
            .and_then(|seçici| seçici.strip_suffix(']'))
            .and_then(|seçici| seçici.trim().parse::<usize>().ok())
            .and_then(|sıra| boyutlar.get(sıra).map(|(_, değer)| değer))
            .or_else(|| {
                boyutlar
                    .iter()
                    .find(|(ad, _)| ad == seçici)
                    .map(|(_, değer)| değer)
            })
            .map(veri_değeri_metni)
            .unwrap_or_default();
        sonuç.replace_range(başlangıç..=kapanış, &değer);
        tarama = başlangıç + değer.len();
    }
    sonuç
}

fn veri_değeri_metni(değer: &VeriDeğeri) -> String {
    match değer {
        VeriDeğeri::Boş => "-".to_owned(),
        VeriDeğeri::Sayı(sayı) => crate::yardimci::bicim::ondalık_kırp(*sayı),
        VeriDeğeri::Çift([x, y]) => format!(
            "{},{}",
            crate::yardimci::bicim::ondalık_kırp(*x),
            crate::yardimci::bicim::ondalık_kırp(*y)
        ),
        VeriDeğeri::Dizi(dizi) => dizi
            .iter()
            .map(|sayı| crate::yardimci::bicim::ondalık_kırp(*sayı))
            .collect::<Vec<_>>()
            .join(","),
        VeriDeğeri::Metin(metin) => metin.clone(),
        VeriDeğeri::Mantıksal(değer) => değer.to_string(),
        VeriDeğeri::Zaman(ms) => ms.to_string(),
    }
}

#[derive(Clone, Debug)]
struct DışEtiketYerleşimi {
    dilim_sırası: usize,
    tam_ham_metin: String,
    ham_metin: String,
    tam_metin: String,
    metin: String,
    konum: (f32, f32),
    hiza: YatayHiza,
    noktalar: [(f32, f32); 3],
    orta_açı: f64,
    metin_genişliği: f32,
    metin_yüksekliği: f32,
    dikdörtgen_y: f32,
    dikdörtgen_yüksekliği: f32,
}

fn kenar_uzaklığını_çöz(seri: &PastaSerisi, görünüm: Dikdörtgen) -> f32 {
    if seri.etiket.dış_hiza == DışEtiketHizası::Kenar && seri.etiket.kenar_boşluğu > 0.0 {
        seri.etiket.kenar_boşluğu
    } else {
        seri.etiket.kenar_uzaklığı.çöz(görünüm.genişlik)
    }
}

fn etiket_dönüş_açısı(döndürme: EtiketDöndürme, orta_açı: f32, iç: bool) -> f32 {
    match döndürme {
        EtiketDöndürme::Yok => 0.0,
        EtiketDöndürme::Derece(derece) => derece.to_radians(),
        EtiketDöndürme::Radyal => {
            if orta_açı.cos() < 0.0 {
                -orta_açı + std::f32::consts::PI
            } else {
                -orta_açı
            }
        }
        EtiketDöndürme::Teğetsel | EtiketDöndürme::TeğetselÇevirmesiz if iç => {
            let mut açı = orta_açı
                .cos()
                .atan2(orta_açı.sin())
                .rem_euclid(std::f32::consts::TAU);
            if orta_açı.sin() > 0.0 && döndürme == EtiketDöndürme::Teğetsel {
                açı += std::f32::consts::PI;
            }
            açı - std::f32::consts::PI
        }
        EtiketDöndürme::Teğetsel | EtiketDöndürme::TeğetselÇevirmesiz => 0.0,
    }
}

fn dış_etiketleri_yerleştir_ve_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &PastaSerisi,
    dilimler: &[Dilim],
    etiketler: &mut [DışEtiketYerleşimi],
    seri_yarıçapı: f32,
    boyut: f32,
    genel_etiket_kutuları: &mut Vec<Dikdörtgen>,
) {
    if etiketler.is_empty() {
        return;
    }
    let merkez = dilimler.first().map(|d| d.merkez).unwrap_or_default();
    let görünüm = dilimler
        .first()
        .map(|d| d.görünüm_alanı)
        .unwrap_or_else(|| Dikdörtgen::yeni(0.0, 0.0, çizici.genişlik(), çizici.yükseklik()));

    // `pie/labelLayout.ts::constrainTextWidth`: varsayılan
    // `overflow: 'truncate'` görünüm kutusuna sığmayan dış etiketleri üç
    // noktayla kısaltır. Hedef genişlik hizalama stratejisine göre değişir.
    etiket_metinlerini_sınırla(çizici, seri, etiketler, merkez, görünüm, boyut, false);

    // `alignTo: 'labelLine'`: aynı taraftaki bütün metin çapalarını en
    // uzaktaki label-line ucuna taşır; dirsek aynı miktarda ötelenir.
    if seri.etiket.dış_hiza == DışEtiketHizası::EtiketÇizgisi {
        for sağda in [false, true] {
            let uç = etiketler
                .iter()
                .filter(|e| (e.konum.0 >= merkez.0) == sağda)
                .map(|e| e.konum.0)
                .reduce(if sağda { f32::max } else { f32::min });
            if let Some(uç) = uç {
                for etiket in etiketler
                    .iter_mut()
                    .filter(|e| (e.konum.0 >= merkez.0) == sağda)
                {
                    let fark = etiket.konum.0 - uç;
                    etiket.noktalar[1].0 += fark;
                    etiket.konum.0 = uç;
                }
            }
        }
    }

    let döndürülmüş = !matches!(
        seri.etiket.döndürme,
        EtiketDöndürme::Yok | EtiketDöndürme::Derece(0.0)
    );
    if seri.etiket_çakışmasını_önle && !döndürülmüş {
        for sağda in [false, true] {
            let mut sıralar: Vec<usize> = etiketler
                .iter()
                .enumerate()
                .filter_map(|(sıra, e)| ((e.konum.0 >= merkez.0) == sağda).then_some(sıra))
                .collect();
            let değişti =
                etiketleri_dikey_kaydır(etiketler, &mut sıralar, görünüm.y, görünüm.alt());
            if değişti && seri.etiket.dış_hiza == DışEtiketHizası::Yok {
                etiket_xlerini_elipse_uyarla(
                    etiketler,
                    &sıralar,
                    merkez,
                    seri_yarıçapı,
                    if sağda { 1.0 } else { -1.0 },
                    seri.etiket_çizgisi.uzunluk1,
                    seri.etiket_çizgisi.uzunluk2,
                );
                // Elips üzerinde x değişince kenara kalan metin genişliği de
                // değişir; resmî kod `forceRecalculate` ile tam metinden
                // yeniden hesaplar.
                etiket_metinlerini_sınırla(çizici, seri, etiketler, merkez, görünüm, boyut, true);
            }
        }
    }

    for etiket in etiketler {
        let Some(dilim) = dilimler.get(etiket.dilim_sırası) else {
            continue;
        };
        let sağda = etiket.konum.0 >= merkez.0;
        let eski_fark = etiket.noktalar[1].0 - etiket.noktalar[2].0;
        match seri.etiket.dış_hiza {
            DışEtiketHizası::Kenar if sağda => {
                etiket.noktalar[2].0 = görünüm.sağ()
                    - kenar_uzaklığını_çöz(seri, görünüm)
                    - etiket.metin_genişliği
                    - seri.etiket.çizgi_uzaklığı;
            }
            DışEtiketHizası::Kenar => {
                etiket.noktalar[2].0 = görünüm.x
                    + kenar_uzaklığını_çöz(seri, görünüm)
                    + etiket.metin_genişliği
                    + seri.etiket.çizgi_uzaklığı;
            }
            _ if sağda => {
                etiket.noktalar[2].0 = etiket.konum.0 - seri.etiket.çizgi_uzaklığı;
                etiket.noktalar[1].0 = etiket.noktalar[2].0 + eski_fark;
            }
            _ => {
                etiket.noktalar[2].0 = etiket.konum.0 + seri.etiket.çizgi_uzaklığı;
                etiket.noktalar[1].0 = etiket.noktalar[2].0 + eski_fark;
            }
        }
        etiket.noktalar[1].1 = etiket.konum.1;
        etiket.noktalar[2].1 = etiket.konum.1;
        label_line_açılarını_sınırla(
            &mut etiket.noktalar,
            (etiket.orta_açı.cos() as f32, etiket.orta_açı.sin() as f32),
            seri.etiket_çizgisi.en_küçük_dönüş_açısı,
            seri.etiket_çizgisi.en_büyük_yüzey_açısı,
        );
        let etiket_kutusu = |etiket: &DışEtiketYerleşimi| {
            let x = match etiket.hiza {
                YatayHiza::Sol => etiket.konum.0,
                YatayHiza::Orta => etiket.konum.0 - etiket.metin_genişliği / 2.0,
                YatayHiza::Sağ => etiket.konum.0 - etiket.metin_genişliği,
            };
            Dikdörtgen::yeni(
                x,
                etiket.konum.1 - etiket.metin_yüksekliği / 2.0,
                etiket.metin_genişliği,
                etiket.metin_yüksekliği,
            )
        };
        if let Some(işlev) = &seri.etiket_yerleşimi {
            let parametreler = EtiketYerleşimParametreleri {
                veri_sırası: etiket.dilim_sırası,
                veri_adı: dilim.ad.clone(),
                değer: dilim.değer,
                etiket_kutusu: etiket_kutusu(etiket),
                etiket_çizgisi_noktaları: seri.etiket_çizgisi.göster.then_some(etiket.noktalar),
            };
            let sonuç = işlev.uygula(&parametreler);
            if let Some(x) = sonuç.x {
                etiket.konum.0 = x;
            }
            if let Some(y) = sonuç.y {
                etiket.konum.1 = y;
            }
            if let Some(noktalar) = sonuç.etiket_çizgisi_noktaları {
                etiket.noktalar = noktalar;
            }
        }
        // `PieSeries.defaultOption.labelLayout.hideOverlap`: ECharts bütün
        // seri görünümlerinin etiketlerini LabelManager'da birlikte ele alır.
        // Pasta serileri sırayla boyanırken daha önce kabul edilen global
        // metin kutusuyla çakışan etiket ve kılavuz çizgisi gizlenir.
        let etiket_kutusu = etiket_kutusu(etiket);
        if genel_etiket_kutuları
            .iter()
            .any(|kutu| etiket_kutuları_örtüşüyor(*kutu, etiket_kutusu))
        {
            continue;
        }
        genel_etiket_kutuları.push(etiket_kutusu);
        if seri.etiket_çizgisi.göster {
            let mut yol = Yol::yeni();
            yol.taşı(etiket.noktalar[0]);
            let yumuşaklık = seri.etiket_çizgisi.yumuşaklık;
            if yumuşaklık > 0.0 {
                let p0 = etiket.noktalar[0];
                let p1 = etiket.noktalar[1];
                let p2 = etiket.noktalar[2];
                let u1 = (p1.0 - p0.0).hypot(p1.1 - p0.1);
                let u2 = (p2.0 - p1.0).hypot(p2.1 - p1.1);
                if u1 > 1e-6 && u2 > 1e-6 {
                    let taşı = u1.min(u2) * yumuşaklık;
                    let a = (
                        p1.0 + (p0.0 - p1.0) * taşı / u1,
                        p1.1 + (p0.1 - p1.1) * taşı / u1,
                    );
                    let b = (
                        p1.0 + (p2.0 - p1.0) * taşı / u2,
                        p1.1 + (p2.1 - p1.1) * taşı / u2,
                    );
                    let orta = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
                    yol.kübik(a, a, orta);
                    yol.kübik(b, b, p2);
                } else {
                    yol.çiz(p1);
                    yol.çiz(p2);
                }
            } else {
                yol.çiz(etiket.noktalar[1]);
                yol.çiz(etiket.noktalar[2]);
            }
            let çizgi_stili = &seri.etiket_çizgisi.stil;
            let renk = çizgi_stili
                .renk
                .unwrap_or(dilim.renk)
                .opaklık(çizgi_stili.opaklık);
            çizici.yol_çiz(&yol, çizgi_stili.kalınlık, renk, çizgi_stili.tür);
        }
        let renk = seri.etiket.yazı.renk.unwrap_or(tema::birincil_metin());
        let çizilecek_metin = if etiket.metin == etiket.tam_metin {
            &etiket.ham_metin
        } else {
            &etiket.metin
        };
        zengin_etiketi_yaz(
            çizici,
            çizilecek_metin,
            &seri.etiket,
            etiket.konum,
            etiket.hiza,
            renk,
            etiket_dönüş_açısı(seri.etiket.döndürme, etiket.orta_açı as f32, false),
        );
    }
}

fn etiket_kutuları_örtüşüyor(a: Dikdörtgen, b: Dikdörtgen) -> bool {
    const DOKUNMA_EŞİĞİ: f32 = 0.05;
    a.x < b.sağ() - DOKUNMA_EŞİĞİ
        && b.x < a.sağ() - DOKUNMA_EŞİĞİ
        && a.y < b.alt() - DOKUNMA_EŞİĞİ
        && b.y < a.alt() - DOKUNMA_EŞİĞİ
}

fn etiket_metinlerini_sınırla(
    çizici: &dyn ÇizimYüzeyi,
    seri: &PastaSerisi,
    etiketler: &mut [DışEtiketYerleşimi],
    merkez: (f32, f32),
    görünüm: Dikdörtgen,
    boyut: f32,
    yalnız_yok: bool,
) {
    let sol_uç = etiketler
        .iter()
        .filter(|etiket| etiket.konum.0 < merkez.0)
        .map(|etiket| etiket.konum.0)
        .reduce(f32::min)
        .unwrap_or(merkez.0);
    let sağ_uç = etiketler
        .iter()
        .filter(|etiket| etiket.konum.0 >= merkez.0)
        .map(|etiket| etiket.konum.0)
        .reduce(f32::max)
        .unwrap_or(merkez.0);
    let taşma = seri.etiket.taşma_payını.unwrap_or_else(|| {
        if görünüm.genişlik.min(görünüm.yükseklik) > 200.0 {
            10.0
        } else {
            2.0
        }
    });
    let kenar = kenar_uzaklığını_çöz(seri, görünüm);
    for etiket in etiketler {
        if yalnız_yok && seri.etiket.dış_hiza != DışEtiketHizası::Yok {
            continue;
        }
        let solda = etiket.konum.0 < merkez.0;
        let hedef = match seri.etiket.dış_hiza {
            DışEtiketHizası::Kenar if solda => {
                etiket.noktalar[2].0 - seri.etiket.çizgi_uzaklığı - görünüm.x - kenar
            }
            DışEtiketHizası::Kenar => {
                görünüm.sağ() - kenar - etiket.noktalar[2].0 - seri.etiket.çizgi_uzaklığı
            }
            DışEtiketHizası::EtiketÇizgisi if solda => sol_uç - görünüm.x - taşma,
            DışEtiketHizası::EtiketÇizgisi => görünüm.sağ() - sağ_uç - taşma,
            DışEtiketHizası::Yok if solda => etiket.konum.0 - görünüm.x - taşma,
            DışEtiketHizası::Yok => görünüm.sağ() - etiket.konum.0 - taşma,
        }
        .max(0.0);
        let tam_ölçü = zengin_metin_ölç(çizici, &etiket.tam_ham_metin, &seri.etiket);
        if tam_ölçü.0 <= hedef {
            etiket.metin.clone_from(&etiket.tam_metin);
            etiket.ham_metin.clone_from(&etiket.tam_ham_metin);
            etiket.metin_genişliği = tam_ölçü.0;
            etiket.metin_yüksekliği = tam_ölçü.1;
        } else {
            etiket.metin = etiket
                .tam_metin
                .split('\n')
                .map(|satır| metni_üç_noktayla_sığdır(çizici, satır, boyut, hedef))
                .collect::<Vec<_>>()
                .join("\n");
            etiket.ham_metin.clone_from(&etiket.metin);
            let ölçü = zengin_metin_ölç(çizici, &etiket.ham_metin, &seri.etiket);
            etiket.metin_genişliği = ölçü.0;
            etiket.metin_yüksekliği = ölçü.1;
        }
        let yarım_boşluk = seri.etiket.en_küçük_boşluk / 2.0;
        etiket.dikdörtgen_y = etiket.konum.1 - etiket.metin_yüksekliği / 2.0 - yarım_boşluk;
        etiket.dikdörtgen_yüksekliği = etiket.metin_yüksekliği + seri.etiket.en_küçük_boşluk;
    }
}

fn metni_üç_noktayla_sığdır(
    çizici: &dyn ÇizimYüzeyi,
    metin: &str,
    boyut: f32,
    hedef: f32,
) -> String {
    // zrender `prepareTruncateOptions` çizim kutusundan bir piksel güvenlik
    // payı düşürür. İlk önek tahmini tek tek karakter genişlikleriyle,
    // sonraki düzeltme ise ölçülen satır oranıyla yapılır.
    let kutu = (hedef - 1.0).max(0.0);
    if kutu <= 0.0 {
        return String::new();
    }
    let mut satır_genişliği = çizici.yazı_ölç(metin, boyut).0;
    if satır_genişliği <= kutu {
        return metin.to_string();
    }
    const ÜÇ_NOKTA: &str = "...";
    let üç_nokta_genişliği = çizici.yazı_ölç(ÜÇ_NOKTA, boyut).0;
    let (üç_nokta, içerik) = if üç_nokta_genişliği > kutu {
        ("", kutu)
    } else {
        (ÜÇ_NOKTA, kutu - üç_nokta_genişliği)
    };
    let mut karakterler: Vec<char> = metin.chars().collect();
    for geçiş in 0..2 {
        if satır_genişliği <= içerik {
            return karakterler.into_iter().chain(üç_nokta.chars()).collect();
        }
        let yeni_uzunluk = if geçiş == 0 {
            let mut genişlik = 0.0_f32;
            let mut uzunluk = 0_usize;
            while genişlik < içerik {
                let Some(karakter) = karakterler.get(uzunluk) else {
                    break;
                };
                let karakter = karakter.to_string();
                genişlik += çizici.yazı_ölç(&karakter, boyut).0;
                uzunluk += 1;
            }
            uzunluk
        } else if satır_genişliği > 0.0 {
            ((karakterler.len() as f32 * içerik / satır_genişliği).floor() as usize)
                .min(karakterler.len())
        } else {
            0
        };
        karakterler.truncate(yeni_uzunluk);
        let aday = karakterler.iter().collect::<String>();
        satır_genişliği = çizici.yazı_ölç(&aday, boyut).0;
    }
    karakterler.into_iter().chain(üç_nokta.chars()).collect()
}

// `labelLayoutHelper.shiftLayoutOnXY`nin indis temelli birebir portudur.
// `sıralar`, bu fonksiyonda aynı `etiketler` diliminden enumerate edilerek
// üretildiğinden bütün indisler geçerlidir.
#[allow(clippy::indexing_slicing)]
fn etiketleri_dikey_kaydır(
    etiketler: &mut [DışEtiketYerleşimi],
    sıralar: &mut [usize],
    en_az: f32,
    en_çok: f32,
) -> bool {
    if sıralar.len() < 2 {
        return false;
    }
    sıralar.sort_by(|a, b| {
        etiketler[*a]
            .dikdörtgen_y
            .partial_cmp(&etiketler[*b].dikdörtgen_y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut değişti = false;
    let mut son_konum = 0.0_f32;
    for sıra in sıralar.iter().copied() {
        let fark = etiketler[sıra].dikdörtgen_y - son_konum;
        if fark < 0.0 {
            etiketler[sıra].dikdörtgen_y -= fark;
            etiketler[sıra].konum.1 -= fark;
            değişti = true;
        }
        son_konum = etiketler[sıra].dikdörtgen_y + etiketler[sıra].dikdörtgen_yüksekliği;
    }

    let boşlukları_sıkıştır = |etiketler: &mut [DışEtiketYerleşimi],
                               sıralar: &[usize],
                               fark: f32,
                               en_çok_oran: f32|
     -> bool {
        let boşluklar = sıralar
            .windows(2)
            .map(|çift| {
                let önceki = &etiketler[çift[0]];
                let sonraki = &etiketler[çift[1]];
                (sonraki.dikdörtgen_y - önceki.dikdörtgen_y - önceki.dikdörtgen_yüksekliği).max(0.0)
            })
            .collect::<Vec<_>>();
        let toplam: f32 = boşluklar.iter().sum();
        if toplam <= 0.0 {
            return false;
        }
        let oran = (fark.abs() / toplam).min(en_çok_oran);
        if fark > 0.0 {
            for (sıra, boşluk) in boşluklar.iter().enumerate() {
                etiket_aralığını_kaydır(etiketler, sıralar, 0, sıra + 1, boşluk * oran);
            }
        } else {
            for sıra in (1..sıralar.len()).rev() {
                etiket_aralığını_kaydır(
                    etiketler,
                    sıralar,
                    sıra,
                    sıralar.len(),
                    -boşluklar[sıra - 1] * oran,
                );
            }
        }
        oran > 0.0
    };

    let sınır_boşlukları = |etiketler: &[DışEtiketYerleşimi], sıralar: &[usize]| {
        let ilk = &etiketler[sıralar[0]];
        let son = &etiketler[*sıralar.last().unwrap_or(&sıralar[0])];
        (
            ilk.dikdörtgen_y - en_az,
            en_çok - son.dikdörtgen_y - son.dikdörtgen_yüksekliği,
        )
    };

    let (mut üst_boşluk, mut alt_boşluk) = sınır_boşlukları(etiketler, sıralar);
    if üst_boşluk < 0.0 {
        değişti |= boşlukları_sıkıştır(etiketler, sıralar, -üst_boşluk, 0.8);
    }
    if alt_boşluk < 0.0 {
        değişti |= boşlukları_sıkıştır(etiketler, sıralar, alt_boşluk, 0.8);
    }
    (üst_boşluk, alt_boşluk) = sınır_boşlukları(etiketler, sıralar);

    if üst_boşluk < 0.0 {
        let öbüründen = alt_boşluk.min(-üst_boşluk).max(0.0);
        if öbüründen > 0.0 {
            etiket_aralığını_kaydır(etiketler, sıralar, 0, sıralar.len(), öbüründen);
            değişti = true;
            let kalan = öbüründen + üst_boşluk;
            if kalan < 0.0 {
                değişti |= boşlukları_sıkıştır(etiketler, sıralar, -kalan, 1.0);
            }
        } else {
            değişti |= boşlukları_sıkıştır(etiketler, sıralar, -üst_boşluk, 1.0);
        }
    }
    (üst_boşluk, alt_boşluk) = sınır_boşlukları(etiketler, sıralar);
    if alt_boşluk < 0.0 {
        let öbüründen = üst_boşluk.min(-alt_boşluk).max(0.0);
        if öbüründen > 0.0 {
            etiket_aralığını_kaydır(etiketler, sıralar, 0, sıralar.len(), -öbüründen);
            değişti = true;
            let kalan = öbüründen + alt_boşluk;
            if kalan < 0.0 {
                değişti |= boşlukları_sıkıştır(etiketler, sıralar, kalan, 1.0);
            }
        } else {
            değişti |= boşlukları_sıkıştır(etiketler, sıralar, alt_boşluk, 1.0);
        }
    }

    (üst_boşluk, alt_boşluk) = sınır_boşlukları(etiketler, sıralar);
    if üst_boşluk < 0.0 {
        etiketleri_sınırda_sıkıştır(etiketler, sıralar, -üst_boşluk);
        değişti = true;
    }
    if alt_boşluk < 0.0 {
        etiketleri_sınırda_sıkıştır(etiketler, sıralar, alt_boşluk);
        değişti = true;
    }
    değişti
}

fn etiket_aralığını_kaydır(
    etiketler: &mut [DışEtiketYerleşimi],
    sıralar: &[usize],
    başlangıç: usize,
    bitiş: usize,
    fark: f32,
) {
    if fark == 0.0 {
        return;
    }
    for konum in başlangıç..bitiş.min(sıralar.len()) {
        if let Some(etiket) = sıralar.get(konum).and_then(|sıra| etiketler.get_mut(*sıra)) {
            etiket.dikdörtgen_y += fark;
            etiket.konum.1 += fark;
        }
    }
}

fn etiketleri_sınırda_sıkıştır(
    etiketler: &mut [DışEtiketYerleşimi],
    sıralar: &[usize],
    mut fark: f32,
) {
    if sıralar.len() < 2 || fark == 0.0 {
        return;
    }
    let yön = if fark < 0.0 { -1.0 } else { 1.0 };
    fark = fark.abs();
    let her_biri = (fark / (sıralar.len() - 1) as f32).ceil();
    for sıra in 0..sıralar.len() - 1 {
        if yön > 0.0 {
            etiket_aralığını_kaydır(etiketler, sıralar, 0, sıra + 1, her_biri);
        } else {
            etiket_aralığını_kaydır(
                etiketler,
                sıralar,
                sıralar.len() - sıra - 1,
                sıralar.len(),
                -her_biri,
            );
        }
        fark -= her_biri;
        if fark <= 0.0 {
            break;
        }
    }
}

// Sıra indisleri yalnız yukarıdaki yerleşim listesinden türetilir; doğrudan
// erişim resmî `recalculateX` portunun okunabilirliğini korur.
#[allow(clippy::indexing_slicing)]
fn etiket_xlerini_elipse_uyarla(
    etiketler: &mut [DışEtiketYerleşimi],
    sıralar: &[usize],
    merkez: (f32, f32),
    yarıçap: f32,
    yön: f32,
    uzunluk1: f32,
    uzunluk2: f32,
) {
    for altta in [false, true] {
        let yarı: Vec<usize> = sıralar
            .iter()
            .copied()
            .filter(|sıra| (etiketler[*sıra].konum.1 > merkez.1) == altta)
            .collect();
        let Some(aşırı) = yarı.iter().copied().max_by(|a, b| {
            (etiketler[*a].konum.1 - merkez.1)
                .abs()
                .partial_cmp(&(etiketler[*b].konum.1 - merkez.1).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        }) else {
            continue;
        };
        let dy = (etiketler[aşırı].konum.1 - merkez.1).abs();
        let dx = etiketler[aşırı].konum.0 - merkez.0 - uzunluk2 * yön;
        let ra = yarıçap + uzunluk1;
        let rb = if dx.abs() < ra {
            (dy * dy / (1.0 - dx * dx / (ra * ra)).max(1e-12)).sqrt()
        } else {
            ra
        }
        .max(1e-6);
        for sıra in yarı {
            let dy = (etiketler[sıra].konum.1 - merkez.1).abs();
            let dx = ((1.0 - dy * dy / (rb * rb)).abs() * ra * ra).sqrt();
            etiketler[sıra].konum.0 = merkez.0 + (dx + uzunluk2) * yön;
        }
    }
}

// Üç noktalı sabit label-line geometrisi; 0/1/2 indisleri tipin değişmezidir.
#[allow(clippy::indexing_slicing)]
fn label_line_açılarını_sınırla(
    noktalar: &mut [(f32, f32); 3],
    yüzey_normali: (f32, f32),
    en_küçük_dönüş: f32,
    en_büyük_yüzey: f32,
) {
    let mut p = noktalar.map(|(x, y)| (f64::from(x), f64::from(y)));
    if en_küçük_dönüş > 0.0 && en_küçük_dönüş <= 180.0 {
        let a = (p[0].0 - p[1].0, p[0].1 - p[1].1);
        let b = (p[2].0 - p[1].0, p[2].1 - p[1].1);
        let (ua, ub) = (a.0.hypot(a.1), b.0.hypot(b.1));
        if ua >= 1e-3 && ub >= 1e-3 {
            let a = (a.0 / ua, a.1 / ua);
            let b = (b.0 / ub, b.1 / ub);
            let en_küçük = f64::from(en_küçük_dönüş).to_radians();
            if en_küçük.cos() < a.0 * b.0 + a.1 * b.1 {
                let (mut izdüşüm, uzaklık) = doğruya_izdüşür(p[1], p[2], p[0]);
                let ölçek = uzaklık / (std::f64::consts::PI - en_küçük).tan();
                izdüşüm.0 += b.0 * ölçek;
                izdüşüm.1 += b.1 * ölçek;
                p[1] = doğru_parçasına_sınırla(izdüşüm, p[1], p[2]);
            }
        }
    }
    if en_büyük_yüzey > 0.0 && en_büyük_yüzey <= 180.0 {
        let a = (p[1].0 - p[0].0, p[1].1 - p[0].1);
        let b = (p[2].0 - p[1].0, p[2].1 - p[1].1);
        let (ua, ub) = (a.0.hypot(a.1), b.0.hypot(b.1));
        if ua >= 1e-3 && ub >= 1e-3 {
            let a = (a.0 / ua, a.1 / ua);
            let b = (b.0 / ub, b.1 / ub);
            let normal = (f64::from(yüzey_normali.0), f64::from(yüzey_normali.1));
            let en_büyük = f64::from(en_büyük_yüzey).to_radians();
            if a.0 * normal.0 + a.1 * normal.1 < en_büyük.cos() {
                let (mut izdüşüm, uzaklık) = doğruya_izdüşür(p[1], p[2], p[0]);
                let açı2 = (b.0 * normal.0 + b.1 * normal.1).clamp(-1.0, 1.0).acos();
                let yeni_açı = std::f64::consts::FRAC_PI_2 + açı2 - en_büyük;
                if yeni_açı >= std::f64::consts::FRAC_PI_2 {
                    izdüşüm = p[2];
                } else {
                    let ölçek = uzaklık / (std::f64::consts::FRAC_PI_2 - yeni_açı).tan();
                    izdüşüm.0 += b.0 * ölçek;
                    izdüşüm.1 += b.1 * ölçek;
                    izdüşüm = doğru_parçasına_sınırla(izdüşüm, p[1], p[2]);
                }
                p[1] = izdüşüm;
            }
        }
    }
    for (hedef, kaynak) in noktalar.iter_mut().zip(p) {
        *hedef = (kaynak.0 as f32, kaynak.1 as f32);
    }
}

fn doğruya_izdüşür(a: (f64, f64), b: (f64, f64), nokta: (f64, f64)) -> ((f64, f64), f64) {
    let yön = (b.0 - a.0, b.1 - a.1);
    let uzunluk = yön.0.hypot(yön.1);
    if uzunluk <= f64::EPSILON {
        return (a, (nokta.0 - a.0).hypot(nokta.1 - a.1));
    }
    let birim = (yön.0 / uzunluk, yön.1 / uzunluk);
    let izdüşen = (nokta.0 - a.0) * birim.0 + (nokta.1 - a.1) * birim.1;
    let sonuç = (a.0 + izdüşen * birim.0, a.1 + izdüşen * birim.1);
    (sonuç, (sonuç.0 - nokta.0).hypot(sonuç.1 - nokta.1))
}

fn doğru_parçasına_sınırla(
    aday: (f64, f64),
    başlangıç: (f64, f64),
    bitiş: (f64, f64),
) -> (f64, f64) {
    let t = if (bitiş.0 - başlangıç.0).abs() > f64::EPSILON {
        (aday.0 - başlangıç.0) / (bitiş.0 - başlangıç.0)
    } else if (bitiş.1 - başlangıç.1).abs() > f64::EPSILON {
        (aday.1 - başlangıç.1) / (bitiş.1 - başlangıç.1)
    } else {
        return başlangıç;
    };
    if !t.is_finite() || t < 0.0 {
        başlangıç
    } else if t > 1.0 {
        bitiş
    } else {
        aday
    }
}

/// İpucu satır metni: `değer (%oran)`.
pub fn dilim_değer_metni(dilim: &Dilim) -> String {
    format!(
        "{} (%{:.*})",
        binlik_ayır(dilim.değer),
        dilim.yüzde_hassasiyeti as usize,
        dilim.yüzde
    )
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
    fn yüzdeler_resmi_en_büyük_kalan_yöntemiyle_tam_yüze_dağılır() {
        let sonuç = yüzde_koltukları(&[56.5, 51.1, 40.1, 25.2], 2);
        assert_eq!(sonuç, [32.68, 29.55, 23.19, 14.58]);
        assert!((sonuç.iter().sum::<f64>() - 100.0).abs() < 1e-12);
    }

    #[test]
    fn pasta_etiketi_dataset_boyutu_ve_yüzde_yer_tutucularını_çözer() {
        let seri = PastaSerisi::yeni()
            .ad("Sales")
            .etiket(
                crate::model::stil::Etiket::yeni()
                    .göster(true)
                    .konum(EtiketKonumu::Dış)
                    .biçimleyici("{a} / {b}: {@2012} = {@[1]} ({d}%)"),
            )
            .veri([
                crate::model::deger::VeriÖğesi::adlı("Milk Tea", 56.5).boyutlar([
                    ("product".to_owned(), VeriDeğeri::from("Milk Tea")),
                    ("2012".to_owned(), VeriDeğeri::from(56.5)),
                ]),
                crate::model::deger::VeriÖğesi::adlı("Other", 43.5).boyutlar([
                    ("product".to_owned(), VeriDeğeri::from("Other")),
                    ("2012".to_owned(), VeriDeğeri::from(43.5)),
                ]),
            ]);
        let seçenekler = GrafikSeçenekleri::yeni().seri(seri.clone());
        let dilimler = pasta_yerleşimi(
            &seri,
            &seçenekler,
            Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0),
            &HashSet::new(),
            1.0,
        );

        assert_eq!(
            pasta_etiket_metni(&seri, &dilimler[0]),
            "Sales / Milk Tea: 56.5 = 56.5 (56.5%)"
        );
    }
    use crate::cizim::KayıtYüzeyi;
    use crate::model::deger::VeriÖğesi;
    use crate::model::seri::EtiketÇizgisi;

    fn yerleşim(seri: &PastaSerisi) -> Vec<Dilim> {
        pasta_yerleşimi(
            seri,
            &GrafikSeçenekleri::default(),
            Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0),
            &HashSet::new(),
            1.0,
        )
    }

    #[test]
    fn resmi_varsayılan_yarıçap_ve_etiket_çizgisi() {
        let seri = PastaSerisi::default();
        assert_eq!(seri.yarıçap.1, crate::model::Uzunluk::Yüzde(50.0));
        assert_eq!(EtiketÇizgisi::default().uzunluk2, 30.0);
    }

    #[test]
    fn sıfır_toplam_eşit_dilim_ve_kapatma_kuralı() {
        let seri = PastaSerisi::yeni().veri([("A", 0.0), ("B", 0.0), ("C", 0.0)]);
        let dilimler = yerleşim(&seri);
        assert_eq!(dilimler.len(), 3);
        for dilim in &dilimler {
            assert!(((dilim.açı1 - dilim.açı0).abs() - std::f32::consts::TAU / 3.0).abs() < 1e-5);
            assert_eq!(dilim.oran, 0.0);
        }

        let kapalı = yerleşim(&seri.sıfır_toplamı_göster(false));
        assert!(
            kapalı
                .iter()
                .all(|dilim| (dilim.açı1 - dilim.açı0).abs() < 1e-6)
        );
    }

    #[test]
    fn min_pad_ve_end_angle_resmi_iki_geçişli_dağılımı_izler() {
        let seri = PastaSerisi::yeni()
            .veri([("küçük", 1.0), ("büyük", 99.0)])
            .bitiş_açısı(-90.0)
            .en_küçük_açı(30.0)
            .dolgu_açısı(10.0);
        let dilimler = yerleşim(&seri);
        assert_eq!(dilimler.len(), 2);
        // Açık endAngle toplam aralığı yarım turdur. İlk geometrik pay 40°,
        // görünür yay her iki yandan 5° pad düşülünce 30° kalır.
        assert!(((dilimler[0].açı1 - dilimler[0].açı0).abs().to_degrees() - 30.0).abs() < 1e-3);
        let görünür_toplam: f32 = dilimler
            .iter()
            .map(|dilim| (dilim.açı1 - dilim.açı0).abs())
            .sum();
        assert!((görünür_toplam.to_degrees() - 160.0).abs() < 1e-3);
    }

    #[test]
    fn seçili_dilim_kayar_ve_dar_dilim_etiketi_gizlenir() {
        let seri = PastaSerisi::yeni()
            .veri([
                VeriÖğesi::adlı("A", 1.0).seçili(true),
                VeriÖğesi::adlı("B", 99.0),
            ])
            .seçili_uzaklığı(20.0)
            .en_küçük_etiket_açısı(5.0);
        let dilimler = yerleşim(&seri);
        assert!(!dilimler[0].etiket_göster);
        assert!(dilimler[1].etiket_göster);
        assert!(
            (dilimler[0].merkez.0 - 200.0).abs() > 0.1
                || (dilimler[0].merkez.1 - 150.0).abs() > 0.1
        );
    }

    #[test]
    fn geçerli_veri_yokken_boş_daire_çizilir() {
        let seri = PastaSerisi::yeni();
        let mut yüzey = KayıtYüzeyi::yeni(400.0, 300.0);
        boş_pasta_çiz(&mut yüzey, &seri, Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0));
        assert!(yüzey.döküm().contains("#d3d3d3"));
    }

    #[test]
    fn öğe_opaklığı_dilim_dolgusu_ve_kenarlığını_birlikte_soldurur() {
        let seri = PastaSerisi::yeni()
            .etiket(crate::model::stil::Etiket::yeni().göster(false))
            .öğe_stili(
                crate::model::stil::ÖğeStili::yeni()
                    .renk("#ff0000")
                    .kenarlık_rengi("#235894")
                    .kenarlık_kalınlığı(3.0)
                    .opaklık(0.7),
            )
            .veri([("A", 1.0)]);
        let dilimler = yerleşim(&seri);
        let mut yüzey = KayıtYüzeyi::yeni(400.0, 300.0);
        pasta_çiz(&mut yüzey, &seri, &dilimler, None, &mut Vec::new());
        let döküm = yüzey.döküm();
        assert!(döküm.contains("#ff0000@0.7"));
        assert!(döküm.contains("#235894@0.7"));
    }

    #[test]
    fn iç_pasta_etiketi_dilim_parlaklığına_göre_renk_ve_kontur_seçer() {
        let seri = PastaSerisi::yeni()
            .yarıçap(30.0)
            .etiket(
                crate::model::stil::Etiket::yeni()
                    .göster(true)
                    .konum(EtiketKonumu::İç),
            )
            .veri([("A", 1.0)]);
        let dilimler = yerleşim(&seri);
        let mut yüzey = KayıtYüzeyi::yeni(400.0, 300.0);

        pasta_çiz(&mut yüzey, &seri, &dilimler, None, &mut Vec::new());

        let döküm = yüzey.döküm();
        assert!(döküm.contains("#eeeeee"), "{döküm}");
        assert!(döküm.contains("#5070dd"), "{döküm}");
        // Tek tam dilimin orta açısı aşağı bakar: (30 / 2) + 3 = 18 px.
        assert!(döküm.contains(" 200.0 168.0]"), "{döküm}");
    }

    #[test]
    fn seri_görünüm_kutusu_merkez_ve_yarıçapı_kendi_alanında_çözer() {
        let seri = PastaSerisi::yeni()
            .sol("33.3333%")
            .sağ("33.3333%")
            .yarıçap("25%")
            .veri([("A", 1.0)]);
        let dilimler = yerleşim(&seri);
        let dilim = &dilimler[0];
        assert!((dilim.görünüm_alanı.x - 133.3332).abs() < 1e-3);
        assert!((dilim.görünüm_alanı.genişlik - 133.3336).abs() < 1e-3);
        assert!((dilim.merkez.0 - 200.0).abs() < 1e-3);
        assert!((dilim.dış_yarıçap - 16.6667).abs() < 1e-3);
    }

    #[test]
    fn koordinat_merkezi_pasta_görünüm_merkezinin_yerini_alır() {
        let seri = PastaSerisi::yeni().yarıçap(30.0).veri([("A", 1.0)]);
        let seçenekler = GrafikSeçenekleri::yeni().seri(seri.clone());
        let dilimler = pasta_yerleşimi_merkezle(
            &seri,
            &seçenekler,
            Dikdörtgen::yeni(0.0, 0.0, 400.0, 300.0),
            &HashSet::new(),
            1.0,
            Some((123.0, 234.0)),
        );

        assert_eq!(dilimler[0].merkez, (123.0, 234.0));
        assert_eq!(dilimler[0].dış_yarıçap, 30.0);
    }

    #[test]
    fn pasta_paleti_aynı_veri_adını_seriler_arasında_korur() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .seri(PastaSerisi::yeni().veri([("A", 1.0), ("B", 2.0)]))
            .seri(PastaSerisi::yeni().veri([("C", 3.0), ("A", 4.0)]));
        let Seri::Pasta(ilk) = &seçenekler.seriler[0] else {
            panic!("ilk seri pasta olmalı");
        };
        let Seri::Pasta(ikinci) = &seçenekler.seriler[1] else {
            panic!("ikinci seri pasta olmalı");
        };
        assert_eq!(pasta_palet_sırası(&seçenekler, ilk, 0, "A"), 0);
        assert_eq!(pasta_palet_sırası(&seçenekler, ikinci, 0, "C"), 2);
        assert_eq!(pasta_palet_sırası(&seçenekler, ikinci, 1, "A"), 0);
    }

    #[test]
    fn zengin_etiket_satırları_ayrı_stillerle_ölçülür_ve_çizilir() {
        let etiket = crate::model::stil::Etiket::yeni()
            .yazı(crate::model::stil::YazıStili::yeni().satır_yüksekliği(15.0))
            .zengin_stil(
                "time",
                crate::model::stil::YazıStili::yeni()
                    .boyut(10.0)
                    .renk("#999"),
            );
        let mut yüzey = KayıtYüzeyi::yeni(400.0, 300.0);
        let ölçü = zengin_metin_ölç(&yüzey, "{name|AB}\n{time|12 h}", &etiket);
        assert!((ölçü.0 - 24.0).abs() < 1e-6);
        assert!((ölçü.1 - 30.0).abs() < 1e-6);
        zengin_etiketi_yaz(
            &mut yüzey,
            "{name|AB}\n{time|12 h}",
            &etiket,
            (100.0, 100.0),
            YatayHiza::Sol,
            Renk::SİYAH,
            0.0,
        );
        let döküm = yüzey.döküm();
        assert!(döküm.contains("yazı \"AB\""));
        assert!(döküm.contains("yazı \"12 h\""));
        assert!(döküm.contains("#999999"));
    }

    #[test]
    fn label_line_yüzey_açısı_resmi_izdüşüm_geometrisini_izler() {
        let mut noktalar = [
            (297.7829, 117.04414),
            (284.7286, 87.48251),
            (225.0, 87.48251),
        ];
        let normal = (-0.8702852, 0.49254808);
        label_line_açılarını_sınırla(&mut noktalar, normal, 90.0, 80.0);
        assert!((noktalar[1].0 - 273.40705).abs() < 1e-3);
        assert!((noktalar[1].1 - 87.48251).abs() < 1e-3);
    }
}
