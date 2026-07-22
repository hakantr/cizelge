//! Gösterge saati (gauge) serisi — `echarts/src/chart/gauge` karşılığı:
//! renk bantlı yay, çentikler, etiketler, ibre ve değer yazısı.

use crate::cizim::olay::{İsabetBölgesi, İsabetGeometrisi};
use crate::cizim::yuzey::{dilim_yolu, yuvarlak_uçlu_dilim_yolu};
use crate::cizim::{AfinMatris, DikeyHiza, YatayHiza, Yol, yolu_dönüştür, ÇizimYüzeyi};
use crate::grafik::pasta::zengin_etiketi_yaz;
use crate::grafik::sembol_yolu;
use crate::koordinat::Dikdörtgen;
use crate::model::seri::{GöstergeSaatiSerisi, Sembol};
use crate::model::stil::{
    Etiket, EtiketDöndürme, YazıStili, ÇizgiStili, ÇizgiTürü, ÖğeStili
};
use crate::renk::{Dolgu, Renk};
use crate::tema;
use crate::yardimci::bicim::binlik_ayır;
use crate::yardimci::sayi::doğrusal_eşle;

/// ECharts gauge açısı (derece, saat yönünün tersine artan matematik
/// açısı) → ekran radyanı (y aşağı).
fn ekran_açısı(derece: f64) -> f32 {
    (-derece.to_radians()) as f32
}

/// zrender `normalizeArcAngles(angles, !clockwise)` eşdeğeri. Model API'si
/// ECharts derecelerini saklar; saat yönünde yay azalan, ters yönde yay artan
/// derece aralığına normalleştirilir ve tam çember korunur.
fn bitiş_açısını_normalleştir(
    başlangıç: f64, mut bitiş: f64, saat_yönünde: bool
) -> f64 {
    if saat_yönünde {
        while bitiş > başlangıç {
            bitiş -= 360.0;
        }
        while başlangıç - bitiş > 360.0 {
            bitiş += 360.0;
        }
    } else {
        while bitiş < başlangıç {
            bitiş += 360.0;
        }
        while bitiş - başlangıç > 360.0 {
            bitiş -= 360.0;
        }
    }
    bitiş
}

fn gösterge_dilimi_yolu(
    merkez: (f32, f32),
    iç_yarıçap: f32,
    dış_yarıçap: f32,
    açı0: f32,
    açı1: f32,
    yuvarlak_uç: bool,
) -> Yol {
    if yuvarlak_uç {
        yuvarlak_uçlu_dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1)
    } else {
        dilim_yolu(merkez, iç_yarıçap, dış_yarıçap, açı0, açı1)
    }
}

fn gösterge_dilimini_boya(
    çizici: &mut dyn ÇizimYüzeyi,
    yol: &Yol,
    dolgu: &Dolgu,
    kenarlık: Option<(f32, Renk)>,
) {
    if yol.boş_mu() {
        return;
    }
    çizici.yol_doldur(yol, dolgu);
    if let Some((kalınlık, renk)) = kenarlık
        && kalınlık > 0.0
    {
        çizici.yol_çiz(yol, kalınlık, renk, ÇizgiTürü::Düz);
    }
}

/// `createSymbol(path://…, x, y, width, height, keepAspect)` eşdeğeri.
/// Kaynak sembolün kesin sınır kutusunu hedef dikdörtgene taşır.
fn ibre_simgesi_yolu(simge: &Sembol, hedef: Dikdörtgen, oranı_koru: bool) -> Option<Yol> {
    let kaynak = match simge {
        Sembol::SvgYolu(yol) => (**yol).clone(),
        _ => sembol_yolu(simge, (0.0, 0.0), 2.0, false)?,
    };
    let kutu = kaynak.kesin_sınır_kutusu()?;
    if kutu.genişlik <= f32::EPSILON || kutu.yükseklik <= f32::EPSILON {
        return None;
    }
    let mut hedef = hedef;
    if oranı_koru {
        let ölçek = (hedef.genişlik / kutu.genişlik).min(hedef.yükseklik / kutu.yükseklik);
        let genişlik = kutu.genişlik * ölçek;
        let yükseklik = kutu.yükseklik * ölçek;
        hedef.x += (hedef.genişlik - genişlik) / 2.0;
        hedef.y += (hedef.yükseklik - yükseklik) / 2.0;
        hedef.genişlik = genişlik;
        hedef.yükseklik = yükseklik;
    }
    let x_ölçeği = hedef.genişlik / kutu.genişlik;
    let y_ölçeği = hedef.yükseklik / kutu.yükseklik;
    Some(yolu_dönüştür(
        &kaynak,
        AfinMatris::yeni(
            x_ölçeği,
            0.0,
            0.0,
            y_ölçeği,
            hedef.x - kutu.x * x_ölçeği,
            hedef.y - kutu.y * y_ölçeği,
        ),
    ))
}

fn ibre_yolu(
    simge: Option<&Sembol>,
    merkez: (f32, f32),
    yarıçap: f32,
    açı: f32,
    uzunluk: crate::model::Uzunluk,
    genişlik: f32,
    merkez_kayması: (crate::model::Uzunluk, crate::model::Uzunluk),
    oranı_koru: bool,
) -> Option<Yol> {
    let uzunluk = uzunluk.çöz(yarıçap).max(0.0);
    let genişlik = genişlik.max(0.0);
    if uzunluk <= 0.0 || genişlik <= 0.0 {
        return None;
    }
    let kayma = (merkez_kayması.0.çöz(yarıçap), merkez_kayması.1.çöz(yarıçap));
    let yerel = if let Some(simge) = simge {
        ibre_simgesi_yolu(
            simge,
            Dikdörtgen::yeni(
                kayma.0 - genişlik / 2.0,
                kayma.1 - uzunluk,
                genişlik,
                uzunluk,
            ),
            oranı_koru,
        )?
    } else {
        // ECharts PointerPath yerel olarak yukarı (-π/2) bakar; bütün şekil
        // daha sonra değer açısına döndürülür.
        let arka_çarpanı = if genişlik >= uzunluk / 3.0 { 1.0 } else { 2.0 };
        let mut yol = Yol::yeni();
        yol.taşı((kayma.0, kayma.1 + genişlik * arka_çarpanı));
        yol.çiz((kayma.0 - genişlik, kayma.1));
        yol.çiz((kayma.0, kayma.1 - uzunluk));
        yol.çiz((kayma.0 + genişlik, kayma.1));
        yol.kapat();
        yol
    };
    let dönüşüm = AfinMatris::ötele(merkez.0, merkez.1)
        .çarp(AfinMatris::döndür(açı + std::f32::consts::FRAC_PI_2));
    Some(yolu_dönüştür(&yerel, dönüşüm))
}

fn bant_rengi(seri: &GöstergeSaatiSerisi, oran: f64) -> Renk {
    if seri.renk_bantları.is_empty() {
        return tema::nötr_10();
    }
    let oran = oran.clamp(0.0, 1.0) as f32;
    seri.renk_bantları
        .iter()
        .find(|(son, _)| *son >= oran)
        .or_else(|| seri.renk_bantları.last())
        .map(|(_, renk)| *renk)
        .unwrap_or_else(tema::nötr_10)
}

/// ECharts model zincirindeki `itemStyle` yamalarını yalnız açık alanlar
/// üzerinden birleştirir. `ÖğeStili::default()` boş bir yama sayılır.
fn öğe_stili_yamala(taban: &ÖğeStili, yama: &ÖğeStili) -> ÖğeStili {
    let mut sonuç = taban.clone();
    if yama.renk.is_some() {
        sonuç.renk.clone_from(&yama.renk);
    }
    if yama.kenarlık_rengi.is_some() {
        sonuç.kenarlık_rengi = yama.kenarlık_rengi;
    }
    if yama.kenarlık_kalınlığı != 0.0 {
        sonuç.kenarlık_kalınlığı = yama.kenarlık_kalınlığı;
    }
    if yama.kenarlık_türü != ÇizgiTürü::Düz {
        sonuç.kenarlık_türü = yama.kenarlık_türü;
    }
    if yama.kenarlık_yarıçapı != [0.0; 4] {
        sonuç.kenarlık_yarıçapı = yama.kenarlık_yarıçapı;
    }
    if yama.opaklık.is_some() {
        sonuç.opaklık = yama.opaklık;
    }
    if yama.gölge_bulanıklığı != 0.0 {
        sonuç.gölge_bulanıklığı = yama.gölge_bulanıklığı;
    }
    if yama.gölge_rengi.is_some() {
        sonuç.gölge_rengi = yama.gölge_rengi;
    }
    if yama.gölge_kayması != (0.0, 0.0) {
        sonuç.gölge_kayması = yama.gölge_kayması;
    }
    sonuç
}

fn yolu_stille_boya(
    çizici: &mut dyn ÇizimYüzeyi,
    yol: &Yol,
    stil: &ÖğeStili,
    varsayılan_dolgu: Dolgu,
) {
    let opaklık = stil.opaklık.unwrap_or(1.0).clamp(0.0, 1.0);
    let dolgu = stil
        .renk
        .clone()
        .unwrap_or(varsayılan_dolgu)
        .opaklık(opaklık);
    if stil.gölge_bulanıklığı > 0.0
        && let Some(gölge) = stil.gölge_rengi
    {
        çizici.yol_gölgesi(
            yol,
            gölge.opaklık(opaklık),
            stil.gölge_bulanıklığı,
            stil.gölge_kayması,
        );
    }
    let kenarlık = stil
        .kenarlık_rengi
        .filter(|_| stil.kenarlık_kalınlığı > 0.0)
        .map(|renk| (stil.kenarlık_kalınlığı, renk.opaklık(opaklık)));
    gösterge_dilimini_boya(çizici, yol, &dolgu, kenarlık);
}

fn çizgiyi_stille_boya(
    çizici: &mut dyn ÇizimYüzeyi,
    a: (f32, f32),
    b: (f32, f32),
    stil: &ÇizgiStili,
    kalınlık: f32,
    renk: Renk,
) {
    let mut yol = Yol::yeni();
    yol.taşı(a);
    yol.çiz(b);
    let opak_renk = renk.opaklık(stil.opaklık.clamp(0.0, 1.0));
    if stil.gölge_bulanıklığı > 0.0
        && let Some(gölge) = stil.gölge_rengi
    {
        çizici.yol_çizgi_gölgesi(
            &yol,
            kalınlık,
            stil.tür,
            gölge,
            stil.gölge_bulanıklığı,
            stil.gölge_kayması,
        );
    }
    çizici.yol_çiz(&yol, kalınlık, opak_renk, stil.tür);
}

fn dayanak_yolu(seri: &GöstergeSaatiSerisi, merkez: (f32, f32), yarıçap: f32) -> Option<Yol> {
    let boyut = seri.dayanak_boyutu.max(0.0);
    if boyut <= 0.0 || matches!(seri.dayanak_simgesi, Sembol::Yok) {
        return None;
    }
    let merkez = (
        merkez.0 + seri.dayanak_merkez_kayması.0.çöz(yarıçap),
        merkez.1 + seri.dayanak_merkez_kayması.1.çöz(yarıçap),
    );
    match &seri.dayanak_simgesi {
        Sembol::SvgYolu(_) => ibre_simgesi_yolu(
            &seri.dayanak_simgesi,
            Dikdörtgen::yeni(merkez.0 - boyut / 2.0, merkez.1 - boyut / 2.0, boyut, boyut),
            seri.dayanak_oranı_koru,
        ),
        simge => sembol_yolu(simge, merkez, boyut, false),
    }
}

/// Gösterge saatini çizer.
pub fn gösterge_saati_çiz(
    çizici: &mut dyn ÇizimYüzeyi,
    seri: &GöstergeSaatiSerisi,
    genel_sıra: usize,
    palet: &dyn Fn(usize) -> Renk,
    tuval: Dikdörtgen,
    ilerleme: f32,
    isabetler: &mut Vec<İsabetBölgesi>,
) {
    let merkez = (
        tuval.x + seri.merkez.0.çöz(tuval.genişlik),
        tuval.y + seri.merkez.1.çöz(tuval.yükseklik),
    );
    let taban_yarıçap = tuval.genişlik.min(tuval.yükseklik) / 2.0;
    let yarıçap = seri.yarıçap.çöz(taban_yarıçap);
    if yarıçap <= 0.0 {
        return;
    }
    let şerit = seri.şerit_kalınlığı.max(0.0);
    let kapsam = [seri.en_az, seri.en_çok.max(seri.en_az + 1e-9)];

    let değer_oranı = |değer: f64, kırp: bool| {
        if kırp {
            doğrusal_eşle(değer, kapsam, [0.0, 1.0], true)
        } else {
            (değer - kapsam[0]) / (kapsam[1] - kapsam[0])
        }
    };
    let başlangıç_açısı = seri.başlangıç_açısı as f64;
    let bitiş_açısı = bitiş_açısını_normalleştir(
        başlangıç_açısı,
        seri.bitiş_açısı as f64,
        seri.saat_yönünde,
    );
    let orandan_açı = |oran: f64| başlangıç_açısı + (bitiş_açısı - başlangıç_açısı) * oran;

    // 1) Renk bantlı yay (axisLine). Boş renk dizisi, ECharts 6.1'in
    // `tokens.color.neutral10` öntanımlısıdır; modelde somutlaştırılmaz ki
    // koyu tema aynı option nesnesiyle doğru belirteci çözebilsin.
    if seri.şeridi_göster && şerit > 0.0 {
        if seri.renk_bantları.is_empty() {
            let yol = gösterge_dilimi_yolu(
                merkez,
                yarıçap - şerit,
                yarıçap,
                ekran_açısı(orandan_açı(0.0)),
                ekran_açısı(orandan_açı(1.0)),
                seri.şerit_yuvarlak_uç,
            );
            if seri.şerit_stili.gölge_bulanıklığı > 0.0
                && let Some(gölge) = seri.şerit_stili.gölge_rengi
            {
                çizici.yol_gölgesi(
                    &yol,
                    gölge,
                    seri.şerit_stili.gölge_bulanıklığı,
                    seri.şerit_stili.gölge_kayması,
                );
            }
            gösterge_dilimini_boya(
                çizici,
                &yol,
                &Dolgu::Düz(tema::nötr_10().opaklık(seri.şerit_stili.opaklık)),
                None,
            );
        } else {
            let mut önceki_oran = 0.0f64;
            for (bant_sonu, renk) in &seri.renk_bantları {
                let son = (*bant_sonu as f64).clamp(önceki_oran, 1.0);
                if son > önceki_oran {
                    let yol = gösterge_dilimi_yolu(
                        merkez,
                        yarıçap - şerit,
                        yarıçap,
                        ekran_açısı(orandan_açı(önceki_oran)),
                        ekran_açısı(orandan_açı(son)),
                        seri.şerit_yuvarlak_uç,
                    );
                    if seri.şerit_stili.gölge_bulanıklığı > 0.0
                        && let Some(gölge) = seri.şerit_stili.gölge_rengi
                    {
                        çizici.yol_gölgesi(
                            &yol,
                            gölge,
                            seri.şerit_stili.gölge_bulanıklığı,
                            seri.şerit_stili.gölge_kayması,
                        );
                    }
                    gösterge_dilimini_boya(
                        çizici,
                        &yol,
                        &Dolgu::Düz(renk.opaklık(seri.şerit_stili.opaklık)),
                        None,
                    );
                }
                önceki_oran = son;
            }
            if önceki_oran < 1.0 {
                let yol = gösterge_dilimi_yolu(
                    merkez,
                    yarıçap - şerit,
                    yarıçap,
                    ekran_açısı(orandan_açı(önceki_oran)),
                    ekran_açısı(orandan_açı(1.0)),
                    seri.şerit_yuvarlak_uç,
                );
                gösterge_dilimini_boya(
                    çizici,
                    &yol,
                    &Dolgu::Düz(tema::nötr_10().opaklık(seri.şerit_stili.opaklık)),
                    None,
                );
            }
        }
    }

    // 2) Ana bölme çizgileri, etiketler ve her bölmedeki ara çentikler.
    // Uzaklıklar ECharts GaugeView ile aynı biçimde yay kalınlığının iç
    // kenarından ölçülür.
    let bölme = seri.bölme_sayısı.max(1);
    for i in 0..=bölme {
        let oran = i as f64 / bölme as f64;
        let açı = ekran_açısı(orandan_açı(oran));
        let (kos, sin) = (açı.cos(), açı.sin());
        let ana_dış = yarıçap - şerit - seri.çentik_mesafesi;
        let ana_iç = ana_dış - seri.çentik_uzunluğu.max(0.0);
        if seri.çentikleri_göster && seri.çentik_kalınlığı > 0.0 {
            let renk = if seri.çentik_rengi_otomatik {
                bant_rengi(seri, oran)
            } else {
                seri.çentik_rengi.unwrap_or_else(tema::eksen_çentiği)
            };
            çizgiyi_stille_boya(
                çizici,
                (merkez.0 + ana_dış * kos, merkez.1 + ana_dış * sin),
                (merkez.0 + ana_iç * kos, merkez.1 + ana_iç * sin),
                &seri.çentik_stili,
                seri.çentik_kalınlığı,
                renk,
            );
        }
        if seri.etiketleri_göster {
            let değer = doğrusal_eşle(oran, [0.0, 1.0], kapsam, true);
            let ham = binlik_ayır(değer);
            let metin = seri
                .etiket_biçimleyici
                .as_ref()
                .map_or_else(|| ham.clone(), |biçim| biçim.uygula(değer, &ham));
            let konum_yarıçapı = yarıçap
                - seri.çentik_uzunluğu.max(0.0)
                - seri.çentik_mesafesi
                - seri.etiket_mesafesi;
            let yatay = if kos < -0.4 {
                YatayHiza::Sol
            } else if kos > 0.4 {
                YatayHiza::Sağ
            } else {
                YatayHiza::Orta
            };
            let dikey = if sin < -0.8 {
                DikeyHiza::Üst
            } else if sin > 0.8 {
                DikeyHiza::Alt
            } else {
                DikeyHiza::Orta
            };
            let boyut = seri.etiket_stili.boyut.unwrap_or(seri.etiket_boyutu);
            // Chromium Canvas'ın büyük Arial glif kutusu ab_glyph'in em
            // tabanından birkaç piksel yukarıdadır. Küçük öntanımlıları
            // değiştirmeden büyük gauge kadranlarını aynı optik tabana al.
            let büyük_yazı_yukarı = ((boyut - 20.0).max(0.0) / 12.0).min(2.5);
            let konum = (
                merkez.0 + konum_yarıçapı * kos,
                merkez.1 + konum_yarıçapı * sin - büyük_yazı_yukarı,
            );
            let renk = seri
                .etiket_stili
                .renk
                .or(seri.etiket_rengi)
                .unwrap_or_else(|| {
                    if seri.etiket_rengi_miras {
                        bant_rengi(seri, oran)
                    } else {
                        tema::eksen_etiketi()
                    }
                });
            let opaklık = seri.etiket_stili.opaklık.unwrap_or(1.0);
            let dönüş = match seri.etiket_döndürme {
                EtiketDöndürme::Yok => 0.0,
                // zrender `Text.rotation` açısını kendi y-ekseni yönüne
                // göre uygular; ortak yüzey matrisinin ekran-yönlü açısı
                // bunun ters işaretidir.
                EtiketDöndürme::Derece(derece) => -derece.to_radians(),
                EtiketDöndürme::Radyal => {
                    let mut dönüş = -açı + std::f32::consts::TAU;
                    if dönüş > std::f32::consts::FRAC_PI_2 {
                        dönüş += std::f32::consts::PI;
                    }
                    -dönüş
                }
                EtiketDöndürme::Teğetsel | EtiketDöndürme::TeğetselÇevirmesiz => {
                    açı + std::f32::consts::FRAC_PI_2
                }
            };
            if dönüş.abs() <= f32::EPSILON {
                çizici.yazı(
                    &metin,
                    konum,
                    yatay,
                    dikey,
                    boyut,
                    renk.opaklık(opaklık),
                    seri.etiket_stili.kalın,
                );
            } else {
                çizici.dönüşümlü_yazı(
                    &metin,
                    (0.0, 0.0),
                    YatayHiza::Orta,
                    DikeyHiza::Orta,
                    boyut,
                    renk.opaklık(opaklık),
                    seri.etiket_stili.kalın,
                    AfinMatris::ötele(konum.0, konum.1).çarp(AfinMatris::döndür(dönüş)),
                );
            }
        }
        if seri.ara_çentikleri_göster && i != bölme && seri.ara_çentik_kalınlığı > 0.0 {
            let ara_sayısı = seri.ara_çentik_sayısı.max(1);
            let ara_uzunluğu = seri.ara_çentik_uzunluğu.çöz(yarıçap).max(0.0);
            let ara_dış = yarıçap - şerit - seri.ara_çentik_mesafesi;
            let ara_iç = ara_dış - ara_uzunluğu;
            for j in 0..=ara_sayısı {
                let ara_oranı = (i as f64 + j as f64 / ara_sayısı as f64) / bölme as f64;
                let ara_açısı = ekran_açısı(orandan_açı(ara_oranı));
                let (ara_kos, ara_sin) = (ara_açısı.cos(), ara_açısı.sin());
                let renk = if seri.ara_çentik_rengi_otomatik {
                    bant_rengi(seri, ara_oranı)
                } else {
                    seri.ara_çentik_rengi
                        .unwrap_or_else(tema::eksen_ara_çentiği)
                };
                çizgiyi_stille_boya(
                    çizici,
                    (merkez.0 + ara_dış * ara_kos, merkez.1 + ara_dış * ara_sin),
                    (merkez.0 + ara_iç * ara_kos, merkez.1 + ara_iç * ara_sin),
                    &seri.ara_çentik_stili,
                    seri.ara_çentik_kalınlığı,
                    renk,
                );
            }
        }
    }

    // 3) Data-item görsel stilleri. Gauge `colorBy: data` öntanımlısı
    // nedeniyle açık itemStyle yoksa her öğe paletin kendi sırasını alır.
    let veri_stilleri = seri
        .veri
        .iter()
        .enumerate()
        .map(|(sıra, öğe)| {
            let mut stil = seri.öğe_stili.clone();
            if let Some(yama) = &öğe.stil {
                stil = öğe_stili_yamala(&stil, yama);
            }
            if stil.renk.is_none() {
                stil.renk = Some(Dolgu::Düz(palet(sıra)));
            }
            stil
        })
        .collect::<Vec<_>>();

    let dayanağı_boya = |çizici: &mut dyn ÇizimYüzeyi| {
        if !seri.dayanağı_göster {
            return;
        }
        let Some(yol) = dayanak_yolu(seri, merkez, yarıçap) else {
            return;
        };
        let mut stil = seri.dayanak_stili.clone();
        if stil.kenarlık_kalınlığı > 0.0 && stil.kenarlık_rengi.is_none() {
            stil.kenarlık_rengi = Some(palet(0));
        }
        yolu_stille_boya(çizici, &yol, &stil, Dolgu::Düz(tema::nötr_00()));
    };

    let şekilleri_boya = |çizici: &mut dyn ÇizimYüzeyi| {
        let mut sıralar = (0..seri.veri.len()).collect::<Vec<_>>();
        if seri.ilerlemeyi_göster && seri.ilerleme_örtüşmesi {
            // zrender `z2 = linearMap(value, [min,max], [100,0])`:
            // uzun yay önce, kısa yay en üstte boyanır.
            sıralar.sort_by(|a, b| {
                let av = seri.veri[*a].değer.sayı().unwrap_or(kapsam[0]);
                let bv = seri.veri[*b].değer.sayı().unwrap_or(kapsam[0]);
                bv.total_cmp(&av)
            });
        }
        for sıra in sıralar {
            let Some(öğe) = seri.veri.get(sıra) else {
                continue;
            };
            let Some(değer) = öğe.değer.sayı() else {
                continue;
            };
            let ayar = seri.veri_ayarları.get(sıra);
            let animasyonlu = kapsam[0] + (değer - kapsam[0]) * ilerleme.clamp(0.0, 1.0) as f64;
            let oran = değer_oranı(animasyonlu, true);
            let açı = ekran_açısı(orandan_açı(oran));
            let görsel = &veri_stilleri[sıra];
            let görsel_rengi = görsel
                .renk
                .as_ref()
                .map(Dolgu::temsilî)
                .unwrap_or_else(|| palet(sıra));

            let ibre_yaması = ayar.map(|ayar| &ayar.ibre);
            let ibre_göster = ibre_yaması
                .and_then(|yama| yama.göster)
                .unwrap_or(seri.ibreyi_göster);
            if ibre_göster {
                let simge = ibre_yaması
                    .and_then(|yama| yama.simge.as_ref())
                    .or(seri.ibre_simgesi.as_ref());
                let uzunluk = ibre_yaması
                    .and_then(|yama| yama.uzunluk)
                    .unwrap_or(seri.ibre_uzunluğu);
                let genişlik = ibre_yaması
                    .and_then(|yama| yama.genişlik)
                    .unwrap_or(seri.ibre_genişliği);
                let kayma = ibre_yaması
                    .and_then(|yama| yama.merkez_kayması)
                    .unwrap_or(seri.ibre_merkez_kayması);
                let oranı_koru = ibre_yaması
                    .and_then(|yama| yama.oranı_koru)
                    .unwrap_or(seri.ibre_oranı_koru);
                if let Some(ibre) = ibre_yolu(
                    simge,
                    merkez,
                    yarıçap,
                    açı,
                    uzunluk,
                    genişlik,
                    kayma,
                    oranı_koru,
                ) {
                    let mut stil = öğe_stili_yamala(görsel, &seri.ibre_stili);
                    if let Some(yama) = ibre_yaması.and_then(|yama| yama.stil.as_ref()) {
                        stil = öğe_stili_yamala(&stil, yama);
                    }
                    let otomatik = ibre_yaması
                        .and_then(|yama| yama.renk_otomatik)
                        .unwrap_or(seri.ibre_rengi_otomatik);
                    if otomatik {
                        stil.renk = Some(Dolgu::Düz(bant_rengi(seri, oran)));
                    } else if let Some(renk) = seri.ibre_rengi {
                        stil.renk = Some(Dolgu::Düz(renk));
                    }
                    yolu_stille_boya(çizici, &ibre, &stil, Dolgu::Düz(görsel_rengi));
                }
            }

            let ilerleme_yaması = ayar.map(|ayar| &ayar.ilerleme);
            let ilerleme_göster = ilerleme_yaması
                .and_then(|yama| yama.göster)
                .unwrap_or(seri.ilerlemeyi_göster);
            if ilerleme_göster {
                let progress_oranı = değer_oranı(animasyonlu, seri.ilerleme_kırp);
                if progress_oranı.abs() > f64::EPSILON {
                    let (iç, dış) = if seri.ilerleme_örtüşmesi {
                        ((yarıçap - seri.ilerleme_kalınlığı).max(0.0), yarıçap)
                    } else {
                        let kalınlık = if seri.veri.is_empty() {
                            0.0
                        } else {
                            şerit / seri.veri.len() as f32
                        };
                        (
                            (yarıçap - (sıra + 1) as f32 * kalınlık).max(0.0),
                            (yarıçap - sıra as f32 * kalınlık).max(0.0),
                        )
                    };
                    let progress = gösterge_dilimi_yolu(
                        merkez,
                        iç,
                        dış,
                        ekran_açısı(orandan_açı(0.0)),
                        ekran_açısı(orandan_açı(progress_oranı)),
                        seri.ilerleme_yuvarlak_uç,
                    );
                    let mut stil = öğe_stili_yamala(görsel, &seri.ilerleme_stili);
                    if let Some(yama) = ilerleme_yaması.and_then(|yama| yama.stil.as_ref()) {
                        stil = öğe_stili_yamala(&stil, yama);
                    }
                    let otomatik = ilerleme_yaması
                        .and_then(|yama| yama.renk_otomatik)
                        .unwrap_or(seri.ilerleme_rengi_otomatik);
                    if otomatik {
                        stil.renk = Some(Dolgu::Düz(bant_rengi(seri, progress_oranı)));
                    } else if let Some(renk) = seri.ilerleme_rengi {
                        stil.renk = Some(Dolgu::Düz(renk));
                    }
                    yolu_stille_boya(çizici, &progress, &stil, Dolgu::Düz(görsel_rengi));
                }
            }
        }
    };

    if !seri.dayanak_üstte {
        dayanağı_boya(çizici);
    }
    if !seri.ibre_üstte {
        şekilleri_boya(çizici);
    }

    // 4) Her data öğesinin kendi title/detail yaması seri varsayılanlarının
    // üzerine uygulanır. `inherit`, bant ya da data palette rengini çözer.
    for (sıra, öğe) in seri.veri.iter().enumerate() {
        let Some(değer) = öğe.değer.sayı() else {
            continue;
        };
        let ayar = seri.veri_ayarları.get(sıra);
        let animasyonlu = kapsam[0] + (değer - kapsam[0]) * ilerleme.clamp(0.0, 1.0) as f64;
        let oran = değer_oranı(animasyonlu, true);
        let görsel_rengi = veri_stilleri[sıra]
            .renk
            .as_ref()
            .map(Dolgu::temsilî)
            .unwrap_or_else(|| palet(sıra));
        let miras_rengi = if seri.ilerlemeyi_göster {
            görsel_rengi
        } else {
            bant_rengi(seri, oran)
        };

        let başlık_yaması = ayar.map(|ayar| &ayar.başlık);
        let başlığı_göster = başlık_yaması
            .and_then(|yama| yama.göster)
            .unwrap_or(seri.adı_göster);
        if başlığı_göster && let Some(ad) = &öğe.ad {
            let kayma = başlık_yaması
                .and_then(|yama| yama.merkez_kayması)
                .unwrap_or(seri.ad_merkez_kayması);
            let mut stil = YazıStili {
                renk: seri.ad_rengi,
                boyut: Some(seri.ad_boyutu),
                ..YazıStili::default()
            }
            .yama_uygula(&seri.ad_stili);
            if let Some(yama) = başlık_yaması {
                stil = stil.yama_uygula(&yama.stil);
            }
            let renk_miras = başlık_yaması
                .and_then(|yama| yama.renk_miras)
                .unwrap_or(seri.ad_rengi_miras);
            let renk = stil.renk.unwrap_or_else(|| {
                if renk_miras {
                    miras_rengi
                } else {
                    tema::ikincil_metin()
                }
            });
            let metin = başlık_yaması
                .and_then(|yama| yama.biçimleyici.as_ref())
                .or(seri.ad_biçimleyici.as_ref())
                .map_or_else(|| ad.clone(), |b| b.uygula(değer, ad));
            let konum = (
                merkez.0 + kayma.0.çöz(yarıçap),
                merkez.1 + kayma.1.çöz(yarıçap),
            );
            if metin.contains('\n') {
                zengin_etiketi_yaz(
                    çizici,
                    &metin,
                    &Etiket {
                        göster: true,
                        yazı: stil,
                        ..Etiket::default()
                    },
                    konum,
                    YatayHiza::Orta,
                    renk,
                    0.0,
                );
            } else {
                çizici.yazı(
                    &metin,
                    konum,
                    YatayHiza::Orta,
                    DikeyHiza::Orta,
                    stil.boyut.unwrap_or(seri.ad_boyutu),
                    renk.opaklık(stil.opaklık.unwrap_or(1.0)),
                    stil.kalın,
                );
            }
        }

        let ayrıntı_yaması = ayar.map(|ayar| &ayar.ayrıntı);
        let ayrıntıyı_göster = ayrıntı_yaması
            .and_then(|yama| yama.göster)
            .unwrap_or(seri.değeri_göster);
        if !ayrıntıyı_göster {
            continue;
        }
        let değer_animasyonu = ayrıntı_yaması
            .and_then(|yama| yama.değer_animasyonu)
            .unwrap_or(seri.değer_animasyonu);
        let görüntülenen_değer = if değer_animasyonu {
            animasyonlu
        } else {
            değer
        };
        let duyarlılık = ayrıntı_yaması
            .and_then(|yama| yama.duyarlılık)
            .or(seri.değer_duyarlılığı);
        let animasyon_ara_karesi = değer_animasyonu && ilerleme < 1.0 - f32::EPSILON;
        let ham = if animasyon_ara_karesi {
            duyarlılık.map_or_else(
                || crate::yardimci::bicim::ondalık_kırp(görüntülenen_değer),
                |basamak| format!("{görüntülenen_değer:.basamak$}"),
            )
        } else {
            crate::yardimci::bicim::ondalık_kırp(görüntülenen_değer)
        };
        let biçimleyici = ayrıntı_yaması
            .and_then(|yama| yama.biçimleyici.as_ref())
            .or(seri.değer_biçimleyici.as_ref());
        let metin = biçimleyici.map_or_else(
            || ham.clone(),
            |b| {
                b.uygula_bağlamla_zengin(
                    görüntülenen_değer,
                    &ham,
                    seri.ad.as_deref().unwrap_or_default(),
                    öğe.ad.as_deref().unwrap_or_default(),
                )
            },
        );
        let taban = YazıStili {
            renk: seri.değer_rengi,
            boyut: Some(seri.değer_boyutu),
            satır_yüksekliği: Some(30.0),
            kalın: seri.değer_kalın,
            kalınlık_belirtildi: true,
            kenarlık_kalınlığı: Some(0.0),
            iç_boşluk: Some([5.0, 10.0, 5.0, 10.0]),
            genişlik: Some(crate::model::Uzunluk::Piksel(100.0)),
            ..YazıStili::default()
        };
        let mut yazı = taban.yama_uygula(&seri.değer_stili);
        if let Some(yama) = ayrıntı_yaması {
            yazı = yazı.yama_uygula(&yama.stil);
        }
        let renk_miras = ayrıntı_yaması
            .and_then(|yama| yama.renk_miras)
            .unwrap_or(seri.değer_rengi_miras);
        let arkaplan_miras = ayrıntı_yaması
            .and_then(|yama| yama.arkaplan_miras)
            .unwrap_or(seri.değer_arkaplanı_miras);
        let kenarlık_miras = ayrıntı_yaması
            .and_then(|yama| yama.kenarlık_miras)
            .unwrap_or(seri.değer_kenarlığı_miras);
        if renk_miras && yazı.renk.is_none() {
            yazı.renk = Some(miras_rengi);
        }
        if arkaplan_miras && yazı.arkaplan.is_none() {
            yazı.arkaplan = Some(Dolgu::Düz(miras_rengi));
        }
        if kenarlık_miras && yazı.kenarlık_rengi.is_none() {
            yazı.kenarlık_rengi = Some(miras_rengi);
        }
        // zrender'da açık `detail.height`, arka planın içerik yüksekliğini
        // doğrudan sınırlar; kalıtılan varsayılan `lineHeight: 30` kutuyu
        // yeniden büyütmez. Ortak rich-text yerleşimindeki satır ölçüsünü
        // aynı açık yüksekliğe sabitle.
        if let Some(yükseklik) = yazı.yükseklik {
            yazı.satır_yüksekliği = Some(yükseklik);
        }
        if let Some(genişlik) = yazı.genişlik {
            yazı.genişlik = Some(crate::model::Uzunluk::Piksel(genişlik.çöz(yarıçap)));
        }
        let mut zengin = seri.değer_zengin.clone();
        if let Some(yama) = ayrıntı_yaması {
            zengin.extend(yama.zengin.clone());
        }
        let etiket = Etiket {
            göster: true,
            yazı,
            zengin,
            ..Etiket::default()
        };
        let kayma = ayrıntı_yaması
            .and_then(|yama| yama.merkez_kayması)
            .unwrap_or(seri.değer_merkez_kayması);
        let konum = (
            merkez.0 + kayma.0.çöz(yarıçap),
            merkez.1 + kayma.1.çöz(yarıçap),
        );
        let varsayılan_renk = if renk_miras {
            miras_rengi
        } else {
            seri.değer_rengi.unwrap_or_else(tema::birincil_metin)
        };
        let kutu_görünür = etiket.yazı.arkaplan.is_some()
            || etiket.yazı.kenarlık_rengi.is_some()
                && etiket.yazı.kenarlık_kalınlığı.unwrap_or(1.0) > 0.0;
        if kutu_görünür {
            zengin_etiketi_yaz(
                çizici,
                "",
                &Etiket {
                    zengin: Default::default(),
                    ..etiket.clone()
                },
                konum,
                YatayHiza::Orta,
                varsayılan_renk,
                0.0,
            );
        }
        let mut içerik_etiketi = etiket;
        içerik_etiketi.yazı.arkaplan = None;
        içerik_etiketi.yazı.kenarlık_rengi = None;
        içerik_etiketi.yazı.kenarlık_kalınlığı = None;
        içerik_etiketi.yazı.kenarlık_yarıçapları = None;
        // Arka plan ayrı katmanda boyansa da width/height/padding metin
        // yerleşiminin parçasıdır. Bunları kaldırmak özellikle sola hizalı
        // rich detail koşularını doğal metin genişliğine göre kaydırır.
        // Canvas/zrender, font boyutu lineHeight'tan büyük olduğunda glif
        // kutusunu satır kutusunun optik merkezine doğru sınırlı miktarda
        // yukarı taşır. Ortak rich-text taban çizgisi doğal font kutusunu
        // kullandığından bu farkı gauge detail içeriğinde tamamla.
        let boyut = içerik_etiketi.yazı.boyut.unwrap_or(seri.değer_boyutu);
        let satır = içerik_etiketi.yazı.satır_yüksekliği.unwrap_or(boyut);
        let optik_yukarı = ((boyut - satır).max(0.0) * 0.2).min(5.0);
        zengin_etiketi_yaz(
            çizici,
            &metin,
            &içerik_etiketi,
            (konum.0, konum.1 - optik_yukarı),
            YatayHiza::Orta,
            varsayılan_renk,
            0.0,
        );
    }

    if seri.ibre_üstte {
        şekilleri_boya(çizici);
    }
    if seri.dayanak_üstte {
        dayanağı_boya(çizici);
    }

    for (sıra, öğe) in seri.veri.iter().enumerate() {
        let Some(değer) = öğe.değer.sayı() else {
            continue;
        };
        isabetler.push(İsabetBölgesi {
            seri_sırası: genel_sıra,
            veri_sırası: sıra,
            seri_adı: seri.ad.clone(),
            ad: öğe.ad.clone(),
            değer: Some(değer),
            geometri: İsabetGeometrisi::Daire { merkez, yarıçap },
        });
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::cizim::KayıtYüzeyi;
    use crate::model::stil::{Biçimleyici, ÖğeStili};
    use std::sync::Arc;

    #[test]
    fn echarts_6_1_ontanimli_geometri_ana_ara_centik_ve_metni_korur() {
        tema::koyu_ayarla(false);
        let seri = GöstergeSaatiSerisi::yeni()
            .ad("Pressure")
            .değer(50.0, "SCORE")
            .değer_biçimleyici("{value}");
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        let mut isabetler = Vec::new();

        gösterge_saati_çiz(
            &mut yüzey,
            &seri,
            0,
            &|_| Renk::onaltılık(0x5070dd),
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            1.0,
            &mut isabetler,
        );

        let döküm = yüzey.döküm();
        assert!(döküm.starts_with("doldur #e8ebf0@1.0 | T(217.9,394.6)"));
        assert_eq!(
            yüzey
                .komutlar
                .iter()
                .filter(|komut| komut.starts_with("çiz "))
                .count(),
            71,
            "11 ana bölme + 10 × 6 ara çentik çizilmeli"
        );
        assert!(döküm.contains("yazı \"50\" (350.0,100.6) orta/üst b=12.0"));
        assert!(döküm.contains("yazı \"SCORE\" (350.0,301.9) orta/orta b=16.0"));
        assert!(döküm.contains("yazı \"50\" (350.0,341.3) orta/orta b=30.0"));
        assert!(
            döküm.contains(
                "doldur #5070dd@1.0 | T(350.0,274.5) Ç(344.0,262.5) Ç(350.0,144.4) Ç(356.0,262.5) Z"
            ),
            "{döküm}"
        );
        assert!(matches!(
            isabetler.first().map(|isabet| isabet.geometri.clone()),
            Some(İsabetGeometrisi::Daire {
                merkez: (350.0, 262.5),
                yarıçap: 196.875
            })
        ));

        tema::koyu_ayarla(true);
        let mut koyu_yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
        gösterge_saati_çiz(
            &mut koyu_yüzey,
            &seri,
            0,
            &|_| Renk::onaltılık(0x5070dd),
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            1.0,
            &mut Vec::new(),
        );
        assert!(koyu_yüzey.döküm().starts_with("doldur #232328@1.0"));
        tema::koyu_ayarla(false);
    }

    #[test]
    fn progress_yayi_ve_detail_value_animation_ayni_gecis_degerini_izler() {
        tema::koyu_ayarla(false);
        let seri = GöstergeSaatiSerisi::yeni()
            .değer(50.0, "SCORE")
            .ilerleme(true, 10.0)
            .değer_animasyonu(true);
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);

        gösterge_saati_çiz(
            &mut yüzey,
            &seri,
            0,
            &|_| Renk::onaltılık(0x5070dd),
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            0.5,
            &mut Vec::new(),
        );

        let palet_dolguları = yüzey
            .komutlar
            .iter()
            .filter(|komut| komut.starts_with("doldur #5070dd@1.0"))
            .collect::<Vec<_>>();
        assert_eq!(palet_dolguları.len(), 2, "ibre ve progress yayı çizilmeli");
        let ibre = palet_dolguları.first().expect("ibre dolgusu");
        let progress = palet_dolguları.last().expect("progress dolgusu");
        assert!(ibre.contains(" Ç(") && !ibre.contains(" Y("));
        assert!(progress.contains(" Y("), "progress bir yay yolu olmalı");
        assert!(
            yüzey
                .komutlar
                .iter()
                .any(|komut| komut.contains("yazı \"25\" (350.0,341.3)")),
            "detail.valueAnimation yarı karede 25 göstermeli: {}",
            yüzey.döküm()
        );
    }

    #[test]
    fn detail_precision_yalniz_animasyonun_ara_karesini_yuvarlar() {
        let seri = GöstergeSaatiSerisi::yeni()
            .veri([84.18])
            .değer_animasyonu(true)
            .değer_duyarlılığı(1);
        let çiz = |ilerleme| {
            let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);
            gösterge_saati_çiz(
                &mut yüzey,
                &seri,
                0,
                &|_| Renk::onaltılık(0x5070dd),
                Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
                ilerleme,
                &mut Vec::new(),
            );
            yüzey.döküm()
        };

        assert!(çiz(0.5).contains("yazı \"42.1\""));
        assert!(çiz(1.0).contains("yazı \"84.18\""));
    }

    #[test]
    fn saat_yonu_zrender_yay_normalizasyonunu_korur() {
        assert_eq!(bitiş_açısını_normalleştir(90.0, -270.0, true), -270.0);
        assert_eq!(bitiş_açısını_normalleştir(10.0, 80.0, true), -280.0);
        assert_eq!(bitiş_açısını_normalleştir(230.0, 310.0, false), 310.0);
        assert_eq!(bitiş_açısını_normalleştir(80.0, 10.0, false), 370.0);
    }

    #[test]
    fn speed_gauge_round_cap_svg_ibre_golge_ve_rich_detaili_yapisal_kilitler() {
        tema::koyu_ayarla(false);
        let simge = Sembol::svg_yolu(
            "path://M5,0 C7,0 8,2 8,4 L10,100 C10,105 0,105 0,100 L2,4 C2,2 3,0 5,0 Z",
        )
        .expect("özel ibre yolu");
        let seri = GöstergeSaatiSerisi::yeni()
            .veri([100.0])
            .aralık(0.0, 240.0)
            .açılar(180.0, 0.0)
            .bölme_sayısı(12)
            .öğe_stili(
                ÖğeStili::yeni()
                    .renk("#58D9F9")
                    .gölge_rengi("rgba(0,138,255,0.45)")
                    .gölge_bulanıklığı(10.0)
                    .gölge_kayması(2.0, 2.0),
            )
            .şerit(true, 18.0)
            .şerit_yuvarlak_uç(true)
            .ilerleme(true, 18.0)
            .ilerleme_yuvarlak_uç(true)
            .ibre(true, "75%", 16.0)
            .ibre_simgesi(simge)
            .ibre_merkez_kayması(0.0, "5%")
            .ara_çentikler(true, 2, 6.0, 10.0, 2.0)
            .ara_çentik_rengi("#999")
            .ana_çentikler(true, 12.0, 10.0, 3.0)
            .ana_çentik_rengi("#999")
            .eksen_etiketleri(true, 30.0, 20.0)
            .eksen_etiket_rengi("#999")
            .ad_göster(false)
            .değer_merkez_kayması(0.0, "35%")
            .değer_animasyonu(true)
            .değer_biçimleyici(Biçimleyici::İşlev(Arc::new(|değer, _| {
                format!("{{value|{değer:.0}}}{{unit|km/h}}")
            })))
            .değer_stili(
                YazıStili::yeni()
                    .arkaplan("#fff")
                    .kenarlık_rengi("#999")
                    .kenarlık_kalınlığı(2.0)
                    .genişlik("60%")
                    .satır_yüksekliği(40.0)
                    .yükseklik(40.0)
                    .kenarlık_yarıçapı(8.0),
            )
            .değer_zengin_stil(
                "value",
                YazıStili::yeni().boyut(50.0).kalın(true).renk("#777"),
            )
            .değer_zengin_stil(
                "unit",
                YazıStili::yeni()
                    .boyut(20.0)
                    .renk("#999")
                    .iç_boşluk([0.0, 0.0, -20.0, 10.0]),
            );
        let mut yüzey = KayıtYüzeyi::yeni(700.0, 525.0);

        gösterge_saati_çiz(
            &mut yüzey,
            &seri,
            0,
            &|_| Renk::onaltılık(0x58d9f9),
            Dikdörtgen::yeni(0.0, 0.0, 700.0, 525.0),
            1.0,
            &mut Vec::new(),
        );

        let döküm = yüzey.döküm();
        assert!(
            döküm
                .lines()
                .next()
                .is_some_and(|satır| satır.contains(" Y(")),
            "axisLine.roundCap yay kapaklarını üretmeli"
        );
        assert_eq!(
            yüzey
                .komutlar
                .iter()
                .filter(|komut| komut.starts_with("çiz "))
                .count(),
            49,
            "13 splitLine + 12 × 3 axisTick çizilmeli"
        );
        assert_eq!(
            yüzey
                .komutlar
                .iter()
                .filter(|komut| komut.starts_with("yol-gölgesi #008aff@0.5 b=10.0 k=(2.0,2.0)"))
                .count(),
            2,
            "pointer ve progress aynı itemStyle gölgesini miras almalı: {döküm}"
        );
        let mavi = yüzey
            .komutlar
            .iter()
            .filter(|komut| komut.starts_with("doldur #58d9f9@1.0"))
            .collect::<Vec<_>>();
        assert_eq!(mavi.len(), 2, "özel pointer ve progress çizilmeli");
        assert!(mavi[0].contains(" K("), "SVG pointer kübik yolunu korumalı");
        assert!(mavi[1].contains(" Y("), "progress roundCap yay yolu olmalı");
        assert!(
            döküm.contains("138.1x50.0"),
            "detail.width radiusun %60'ıdır"
        );
        assert!(döküm.contains("yazı \"100\""));
        assert!(döküm.contains("yazı \"km/h\""));
    }
}
