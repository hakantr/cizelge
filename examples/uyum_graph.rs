//! Resmî ECharts Graph örneklerinin belirlenimci uyum fixture'ları.
//!
//! Veri ve seçenekler `tools/uyum/graph_verisi.mjs` ile kilitli
//! `echarts-examples` kaynaklarından çıkarılır. Kuvvet yerleşiminin tarayıcı
//! rastgele akışına bağlı son noktaları da aynı ECharts 6.1 sahnesinden
//! kilitlenir; Cizelge bu noktalar üzerinde kendi bağ kırpma, stil, etiket ve
//! raster hattını çalıştırır.

use std::collections::HashSet;

use cizelge::grafik::grafo::grafo_yerleşimi_kur;
use cizelge::hazir::*;
use cizelge::model::seri::EksenBağı;
use serde_json::{Value, json};

#[cfg(test)]
const RESMİ_IDLER: [&str; 11] = [
    "graph-force2",
    "graph-grid",
    "graph-simple",
    "graph-force",
    "graph-label-overlap",
    "graph",
    "graph-circular-layout",
    "graph-force-dynamic",
    "graph-life-expectancy",
    "graph-webkit-dep",
    "graph-npm",
];

fn belge_kaynağı(id: &str) -> Result<&'static str, String> {
    match id {
        "graph-force2" => Ok(include_str!("uyum_veri/graph/graph-force2.json")),
        "graph-grid" => Ok(include_str!("uyum_veri/graph/graph-grid.json")),
        "graph-simple" => Ok(include_str!("uyum_veri/graph/graph-simple.json")),
        "graph-force" => Ok(include_str!("uyum_veri/graph/graph-force.json")),
        "graph-label-overlap" => Ok(include_str!("uyum_veri/graph/graph-label-overlap.json")),
        "graph" => Ok(include_str!("uyum_veri/graph/graph.json")),
        "graph-circular-layout" => Ok(include_str!("uyum_veri/graph/graph-circular-layout.json")),
        "graph-force-dynamic" => Ok(include_str!("uyum_veri/graph/graph-force-dynamic.json")),
        "graph-life-expectancy" => Ok(include_str!("uyum_veri/graph/graph-life-expectancy.json")),
        "graph-webkit-dep" => Ok(include_str!("uyum_veri/graph/graph-webkit-dep.json")),
        "graph-npm" => Ok(include_str!("uyum_veri/graph/graph-npm.json")),
        _ => Err(format!("bilinmeyen resmî Graph fixture'ı: {id}")),
    }
}

fn sahne_kaynağı(id: &str) -> Result<&'static str, String> {
    match id {
        "graph-force2" => Ok(include_str!("uyum_veri/graph/sahneler/graph-force2.json")),
        "graph-grid" => Ok(include_str!("uyum_veri/graph/sahneler/graph-grid.json")),
        "graph-simple" => Ok(include_str!("uyum_veri/graph/sahneler/graph-simple.json")),
        "graph-force" => Ok(include_str!("uyum_veri/graph/sahneler/graph-force.json")),
        "graph-label-overlap" => Ok(include_str!(
            "uyum_veri/graph/sahneler/graph-label-overlap.json"
        )),
        "graph" => Ok(include_str!("uyum_veri/graph/sahneler/graph.json")),
        "graph-circular-layout" => Ok(include_str!(
            "uyum_veri/graph/sahneler/graph-circular-layout.json"
        )),
        "graph-force-dynamic" => Ok(include_str!(
            "uyum_veri/graph/sahneler/graph-force-dynamic.json"
        )),
        "graph-life-expectancy" => Ok(include_str!(
            "uyum_veri/graph/sahneler/graph-life-expectancy.json"
        )),
        "graph-webkit-dep" => Ok(include_str!(
            "uyum_veri/graph/sahneler/graph-webkit-dep.json"
        )),
        "graph-npm" => Ok(include_str!("uyum_veri/graph/sahneler/graph-npm.json")),
        _ => Err(format!("bilinmeyen resmî Graph sahnesi: {id}")),
    }
}

fn nesne(değer: &Value) -> Option<&serde_json::Map<String, Value>> {
    değer.as_object()
}

fn sayı(değer: Option<&Value>) -> Option<f32> {
    değer.and_then(Value::as_f64).map(|değer| değer as f32)
}

fn metin(değer: Option<&Value>) -> Option<String> {
    değer.and_then(|değer| {
        değer.as_str().map(str::to_owned).or_else(|| {
            değer
                .as_i64()
                .map(|sayı| sayı.to_string())
                .or_else(|| değer.as_u64().map(|sayı| sayı.to_string()))
                .or_else(|| değer.as_f64().map(|sayı| sayı.to_string()))
        })
    })
}

fn uzunluk(değer: &Value) -> Result<Uzunluk, String> {
    if let Some(sayı) = değer.as_f64() {
        return Ok(Uzunluk::Piksel(sayı as f32));
    }
    let metin = değer
        .as_str()
        .ok_or_else(|| format!("Graph uzunluğu sayı/yüzde değil: {değer}"))?;
    metin
        .strip_suffix('%')
        .ok_or_else(|| format!("Graph uzunluğu yüzde değil: {metin}"))?
        .parse::<f32>()
        .map(Uzunluk::Yüzde)
        .map_err(|hata| format!("Graph yüzdesi çözülemedi ({metin}): {hata}"))
}

fn veri_değeri(değer: &Value) -> VeriDeğeri {
    match değer {
        Value::Null => VeriDeğeri::Boş,
        Value::Bool(değer) => VeriDeğeri::Mantıksal(*değer),
        Value::Number(değer) => VeriDeğeri::Sayı(değer.as_f64().unwrap_or_default()),
        Value::String(değer) => VeriDeğeri::Metin(değer.clone()),
        Value::Array(değerler) => {
            VeriDeğeri::KarmaDizi(değerler.iter().map(veri_değeri).collect())
        }
        Value::Object(_) => VeriDeğeri::Boş,
    }
}

fn sembol(değer: Option<&Value>) -> Option<Sembol> {
    let metin = değer?.as_str()?;
    Some(match metin {
        "circle" => Sembol::Daire,
        "rect" | "square" => Sembol::Kare,
        "roundRect" => Sembol::YuvarlakDikdörtgen,
        "triangle" | "arrow" => Sembol::Üçgen,
        "diamond" => Sembol::Elmas,
        "emptyCircle" => Sembol::İçiBoşDaire,
        "none" => Sembol::Yok,
        yol if yol.starts_with("path://") => Sembol::svg_yolu(yol).unwrap_or(Sembol::Daire),
        _ => Sembol::Daire,
    })
}

fn çizgi_türü(değer: Option<&Value>) -> Option<ÇizgiTürü> {
    Some(match değer?.as_str()? {
        "dashed" => ÇizgiTürü::Kesikli,
        "dotted" => ÇizgiTürü::Noktalı,
        _ => ÇizgiTürü::Düz,
    })
}

fn öğe_stili(değer: Option<&Value>) -> Option<GrafoÖğeStili> {
    let ham = nesne(değer?)?;
    let mut stil = GrafoÖğeStili::yeni();
    if let Some(renk) = ham.get("color").and_then(Value::as_str) {
        stil = stil.renk(renk);
    }
    if let Some(renk) = ham.get("borderColor").and_then(Value::as_str) {
        stil = stil.kenarlık_rengi(renk);
    }
    if let Some(kalınlık) = sayı(ham.get("borderWidth")) {
        stil = stil.kenarlık_kalınlığı(kalınlık);
    }
    if let Some(tür) = çizgi_türü(ham.get("borderType")) {
        stil = stil.kenarlık_türü(tür);
    }
    if let Some(opaklık) = sayı(ham.get("opacity")) {
        stil = stil.opaklık(opaklık);
    }
    if let Some(bulanıklık) = sayı(ham.get("shadowBlur")) {
        stil = stil.gölge_bulanıklığı(bulanıklık);
    }
    if let Some(renk) = ham.get("shadowColor").and_then(Value::as_str) {
        stil = stil.gölge_rengi(renk);
    }
    let x = sayı(ham.get("shadowOffsetX"));
    let y = sayı(ham.get("shadowOffsetY"));
    if x.is_some() || y.is_some() {
        stil = stil.gölge_kayması(x.unwrap_or(0.0), y.unwrap_or(0.0));
    }
    Some(stil)
}

fn çizgi_stili(değer: Option<&Value>) -> Option<GrafoÇizgiStili> {
    let ham = nesne(değer?)?;
    let mut stil = GrafoÇizgiStili::yeni();
    if let Some(renk) = ham.get("color").and_then(Value::as_str) {
        stil = stil.renk(renk);
    }
    if let Some(kalınlık) = sayı(ham.get("width")) {
        stil = stil.kalınlık(kalınlık);
    }
    if let Some(tür) = çizgi_türü(ham.get("type")) {
        stil = stil.tür(tür);
    }
    if let Some(opaklık) = sayı(ham.get("opacity")) {
        stil = stil.opaklık(opaklık);
    }
    if let Some(eğrilik) = sayı(ham.get("curveness")) {
        stil = stil.eğrilik(eğrilik);
    }
    if let Some(bulanıklık) = sayı(ham.get("shadowBlur")) {
        stil = stil.gölge_bulanıklığı(bulanıklık);
    }
    if let Some(renk) = ham.get("shadowColor").and_then(Value::as_str) {
        stil = stil.gölge_rengi(renk);
    }
    let x = sayı(ham.get("shadowOffsetX"));
    let y = sayı(ham.get("shadowOffsetY"));
    if x.is_some() || y.is_some() {
        stil = stil.gölge_kayması(x.unwrap_or(0.0), y.unwrap_or(0.0));
    }
    Some(stil)
}

fn etiket_konumu(değer: &str) -> EtiketKonumu {
    match değer {
        "top" => EtiketKonumu::Üst,
        "bottom" => EtiketKonumu::Alt,
        "left" => EtiketKonumu::Sol,
        "right" => EtiketKonumu::Sağ,
        "insideTop" => EtiketKonumu::İçÜst,
        "insideBottom" => EtiketKonumu::İçAlt,
        "insideLeft" => EtiketKonumu::İçSol,
        "insideRight" => EtiketKonumu::İçSağ,
        "outside" => EtiketKonumu::Dış,
        _ => EtiketKonumu::İç,
    }
}

fn etiket_yaması(değer: Option<&Value>) -> Option<EtiketYaması> {
    let ham = nesne(değer?)?;
    let mut etiket = EtiketYaması::yeni();
    if let Some(göster) = ham.get("show").and_then(Value::as_bool) {
        etiket = etiket.göster(göster);
    }
    if let Some(konum) = ham.get("position").and_then(Value::as_str) {
        etiket = etiket.konum(etiket_konumu(konum));
    }
    if let Some(format) = ham.get("formatter").and_then(Value::as_str) {
        etiket = etiket.biçimleyici(format);
    }
    if let Some(uzaklık) = sayı(ham.get("distance")) {
        etiket = etiket.uzaklık(uzaklık);
    }
    if let Some(dönüş) = sayı(ham.get("rotate")) {
        etiket = etiket.döndürme(EtiketDöndürme::Derece(dönüş));
    }
    if let Some(kayma) = ham.get("offset").and_then(Value::as_array)
        && kayma.len() == 2
    {
        etiket = etiket.kayma(
            kayma[0].as_f64().unwrap_or_default() as f32,
            kayma[1].as_f64().unwrap_or_default() as f32,
        );
    }
    if let Some(hiza) = ham.get("align").and_then(Value::as_str) {
        etiket = etiket.yatay_hiza(match hiza {
            "left" => YazıYatayHizası::Sol,
            "right" => YazıYatayHizası::Sağ,
            _ => YazıYatayHizası::Orta,
        });
    }
    if let Some(hiza) = ham.get("verticalAlign").and_then(Value::as_str) {
        etiket = etiket.dikey_hiza(match hiza {
            "top" => YazıDikeyHizası::Üst,
            "bottom" => YazıDikeyHizası::Alt,
            _ => YazıDikeyHizası::Orta,
        });
    }
    let mut yazı = YazıStili::yeni();
    let mut yazı_var = false;
    if let Some(renk) = ham.get("color").and_then(Value::as_str) {
        yazı = yazı.renk(renk);
        yazı_var = true;
    }
    if let Some(aile) = ham.get("fontFamily").and_then(Value::as_str) {
        yazı = yazı.aile(aile);
        yazı_var = true;
    }
    if let Some(boyut) = sayı(ham.get("fontSize")) {
        yazı = yazı.boyut(boyut);
        yazı_var = true;
    }
    if let Some(kalınlık) = ham.get("fontWeight") {
        let kalın = kalınlık.as_str() == Some("bold")
            || kalınlık.as_f64().is_some_and(|değer| değer >= 600.0);
        yazı = yazı.kalın(kalın);
        yazı_var = true;
    }
    if yazı_var {
        etiket = etiket.yazı(yazı);
    }
    Some(etiket)
}

fn durum(değer: Option<&Value>) -> GrafoDurumu {
    let Some(ham) = değer.and_then(nesne) else {
        return GrafoDurumu::default();
    };
    let mut durum = GrafoDurumu::yeni();
    if let Some(odak) = ham.get("focus").and_then(Value::as_str) {
        durum = durum.odak(match odak {
            "self" => GrafoVurguOdağı::Kendisi,
            "adjacency" => GrafoVurguOdağı::Komşuluk,
            "series" => GrafoVurguOdağı::Seri,
            _ => GrafoVurguOdağı::Yok,
        });
    }
    if let Some(ölçek) = ham.get("scale") {
        durum = durum.ölçek(ölçek.as_f64().map_or_else(
            || {
                if ölçek.as_bool() == Some(false) {
                    1.0
                } else {
                    1.1
                }
            },
            |v| v as f32,
        ));
    }
    if let Some(devre_dışı) = ham.get("disabled").and_then(Value::as_bool) {
        durum = durum.devre_dışı(devre_dışı);
    }
    if let Some(stil) = öğe_stili(ham.get("itemStyle")) {
        durum = durum.öğe_stili(stil);
    }
    if let Some(stil) = çizgi_stili(ham.get("lineStyle")) {
        durum = durum.çizgi_stili(stil);
    }
    if let Some(etiket) = etiket_yaması(ham.get("label")) {
        durum = durum.etiket(etiket);
    }
    if let Some(etiket) = etiket_yaması(ham.get("edgeLabel")) {
        durum = durum.kenar_etiketi(etiket);
    }
    durum
}

fn aralık(değer: Option<&Value>, varsayılan: GrafoAralığı) -> GrafoAralığı {
    let Some(değer) = değer else {
        return varsayılan;
    };
    if let Some(sayı) = değer.as_f64() {
        return GrafoAralığı::tek(sayı as f32);
    }
    if let Some(dizi) = değer.as_array()
        && dizi.len() == 2
    {
        return GrafoAralığı::yeni(
            dizi[0].as_f64().unwrap_or_default() as f32,
            dizi[1].as_f64().unwrap_or_default() as f32,
        );
    }
    varsayılan
}

fn ucu(değer: &Value) -> Result<GrafoUcu, String> {
    if let Some(sıra) = değer.as_u64() {
        return Ok(GrafoUcu::Sıra(sıra as usize));
    }
    if let Some(sıra) = değer.as_i64() {
        return Ok(GrafoUcu::Sıra(sıra.max(0) as usize));
    }
    metin(Some(değer))
        .map(GrafoUcu::Kimlik)
        .ok_or_else(|| format!("Graph bağ ucu çözülemedi: {değer}"))
}

fn düğüm(değer: &Value, sıra: usize, seri_boyutu: f32) -> GrafoDüğümü {
    let Some(ham) = nesne(değer) else {
        return GrafoDüğümü::varsayılan("")
            .kimlik(sıra.to_string())
            .ham_değer(veri_değeri(değer));
    };
    let ad = metin(ham.get("name")).unwrap_or_default();
    let mut düğüm = GrafoDüğümü::varsayılan(ad);
    if let Some(kimlik) = metin(ham.get("id")) {
        düğüm.kimlik = Some(kimlik);
    }
    if let Some(değer) = ham.get("value") {
        düğüm = düğüm.ham_değer(veri_değeri(değer));
    }
    if let (Some(x), Some(y)) = (sayı(ham.get("x")), sayı(ham.get("y"))) {
        düğüm = düğüm.konum(x, y);
    }
    if let Some(sembol) = sembol(ham.get("symbol")) {
        düğüm = düğüm.sembol(sembol);
    }
    if let Some(boyut) = ham.get("symbolSize") {
        if let Some(sayı) = boyut.as_f64() {
            düğüm = düğüm.boyut(sayı as f32);
        } else if let Some(dizi) = boyut.as_array()
            && dizi.len() == 2
        {
            düğüm = düğüm.boyut_çifti(
                dizi[0].as_f64().unwrap_or(f64::from(seri_boyutu)) as f32,
                dizi[1].as_f64().unwrap_or(f64::from(seri_boyutu)) as f32,
            );
        }
    }
    if let Some(kategori) = ham.get("category") {
        if let Some(sıra) = kategori.as_u64() {
            düğüm = düğüm.kategori(sıra as usize);
        } else if let Some(ad) = metin(Some(kategori)) {
            düğüm = düğüm.kategori_adı(ad);
        }
    }
    if let Some(sabit) = ham.get("fixed").and_then(Value::as_bool) {
        düğüm = düğüm.sabit(sabit);
    }
    if let Some(sürüklenebilir) = ham.get("draggable").and_then(Value::as_bool) {
        düğüm = düğüm.sürüklenebilir(sürüklenebilir);
    }
    if let Some(imleç) = ham.get("cursor").and_then(Value::as_str) {
        düğüm = düğüm.imleç(imleç);
    }
    if let Some(stil) = öğe_stili(ham.get("itemStyle")) {
        düğüm = düğüm.öğe_stili(stil);
    }
    if let Some(etiket) = etiket_yaması(ham.get("label")) {
        düğüm = düğüm.etiket(etiket);
    }
    düğüm.vurgu = durum(ham.get("emphasis"));
    düğüm.bulanık = durum(ham.get("blur"));
    düğüm.seçili = durum(ham.get("select"));
    düğüm.başlangıçta_seçili = ham
        .get("selected")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    düğüm
}

fn bağ(değer: &Value) -> Result<GrafoBağı, String> {
    let ham = nesne(değer).ok_or_else(|| format!("Graph bağı nesne değil: {değer}"))?;
    let mut bağ = GrafoBağı::yeni(
        ucu(ham.get("source").ok_or("Graph source eksik")?)?,
        ucu(ham.get("target").ok_or("Graph target eksik")?)?,
    );
    if let Some(değer) = ham.get("value").and_then(Value::as_f64) {
        bağ = bağ.değer(değer);
    }
    if let Some(semboller) = ham.get("symbol") {
        if let Some(dizi) = semboller.as_array()
            && dizi.len() == 2
        {
            bağ = bağ.semboller(
                sembol(Some(&dizi[0])).unwrap_or(Sembol::Yok),
                sembol(Some(&dizi[1])).unwrap_or(Sembol::Yok),
            );
        } else if let Some(sembol) = sembol(Some(semboller)) {
            bağ = bağ.semboller(sembol.clone(), sembol);
        }
    }
    if let Some(boyut) = ham.get("symbolSize") {
        if let Some(sayı) = boyut.as_f64() {
            bağ = bağ.sembol_boyutları(sayı as f32, sayı as f32);
        } else if let Some(dizi) = boyut.as_array()
            && dizi.len() == 2
        {
            bağ = bağ.sembol_boyutları(
                dizi[0].as_f64().unwrap_or(10.0) as f32,
                dizi[1].as_f64().unwrap_or(10.0) as f32,
            );
        }
    }
    bağ.kuvvet_yerleşimini_yoksay = ham
        .get("ignoreForceLayout")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    bağ.çizgi_stili = çizgi_stili(ham.get("lineStyle"));
    bağ.etiket = etiket_yaması(ham.get("label"));
    bağ.vurgu = durum(ham.get("emphasis"));
    bağ.bulanık = durum(ham.get("blur"));
    bağ.seçili = durum(ham.get("select"));
    Ok(bağ)
}

fn kategori(değer: &Value) -> GrafoKategorisi {
    let Some(ham) = nesne(değer) else {
        return GrafoKategorisi::yeni(metin(Some(değer)).unwrap_or_default());
    };
    let mut kategori = GrafoKategorisi::yeni(metin(ham.get("name")).unwrap_or_default());
    if let Some(değer) = ham.get("value") {
        kategori.değer = Some(veri_değeri(değer));
    }
    kategori.sembol = sembol(ham.get("symbol"));
    kategori.boyut = sayı(ham.get("symbolSize"));
    kategori.öğe_stili = öğe_stili(ham.get("itemStyle"));
    kategori.etiket = etiket_yaması(ham.get("label"));
    kategori.vurgu = durum(ham.get("emphasis"));
    kategori.bulanık = durum(ham.get("blur"));
    kategori.seçili = durum(ham.get("select"));
    kategori
}

fn seri(değer: &Value) -> Result<GrafoSerisi, String> {
    let ham = nesne(değer).ok_or("Graph serisi nesne değil")?;
    let mut seri = GrafoSerisi::yeni();
    seri.kimlik = metin(ham.get("id"));
    seri.ad = metin(ham.get("name"));
    seri.yerleşim = match ham.get("layout").and_then(Value::as_str) {
        Some("force") => GrafoYerleşimi::Kuvvet,
        Some("circular") => GrafoYerleşimi::Dairesel,
        _ => GrafoYerleşimi::Yok,
    };
    seri.koordinat_sistemi = match ham.get("coordinateSystem").and_then(Value::as_str) {
        Some("cartesian2d") => GrafoKoordinatSistemi::Kartezyen2B,
        Some("polar") => GrafoKoordinatSistemi::Kutupsal,
        Some("singleAxis") => GrafoKoordinatSistemi::TekEksen,
        Some("calendar") => GrafoKoordinatSistemi::Takvim,
        Some("matrix") => GrafoKoordinatSistemi::Matris,
        _ => GrafoKoordinatSistemi::Görünüm,
    };
    seri.eksen_bağı = EksenBağı {
        x: ham.get("xAxisIndex").and_then(Value::as_u64).unwrap_or(0) as usize,
        y: ham.get("yAxisIndex").and_then(Value::as_u64).unwrap_or(0) as usize,
    };
    for (anahtar, hedef) in [
        ("left", &mut seri.sol),
        ("top", &mut seri.üst),
        ("right", &mut seri.sağ),
        ("bottom", &mut seri.alt),
        ("width", &mut seri.genişlik),
        ("height", &mut seri.yükseklik),
    ] {
        if let Some(değer) = ham.get(anahtar) {
            *hedef = Some(uzunluk(değer)?);
        }
    }
    if ham.get("width").is_some() {
        seri.sağ = None;
    }
    if ham.get("height").is_some() {
        seri.alt = None;
    }
    if let Some(merkez) = ham.get("center").and_then(Value::as_array)
        && merkez.len() == 2
    {
        seri.merkez = Some((uzunluk(&merkez[0])?, uzunluk(&merkez[1])?));
    }
    seri.yakınlaştırma = sayı(ham.get("zoom")).unwrap_or(1.0);
    if let Some(sınır) = ham.get("scaleLimit").and_then(nesne) {
        seri.en_küçük_yakınlaştırma = sayı(sınır.get("min"));
        seri.en_büyük_yakınlaştırma = sayı(sınır.get("max"));
    }
    seri.düğüm_ölçek_oranı = sayı(ham.get("nodeScaleRatio")).unwrap_or(0.6);
    seri.gezinme = match ham.get("roam") {
        Some(Value::Bool(true)) => GrafoGezinmesi::Açık,
        Some(Value::String(değer)) if matches!(değer.as_str(), "pan" | "move") => {
            GrafoGezinmesi::Kaydır
        }
        Some(Value::String(değer)) if matches!(değer.as_str(), "zoom" | "scale") => {
            GrafoGezinmesi::Ölçekle
        }
        _ => GrafoGezinmesi::Kapalı,
    };
    seri.gezinme_tetikleyicisi = if ham.get("roamTrigger").and_then(Value::as_str) == Some("global")
    {
        GrafoGezinmeTetikleyicisi::Global
    } else {
        GrafoGezinmeTetikleyicisi::KendiAlanı
    };
    seri.en_boy_koruma = match ham.get("preserveAspect") {
        Some(Value::Bool(true)) => GrafoEnBoyKoruma::İçer,
        Some(Value::String(değer)) if değer == "contain" => GrafoEnBoyKoruma::İçer,
        Some(Value::String(değer)) if değer == "cover" => GrafoEnBoyKoruma::Kapla,
        _ => GrafoEnBoyKoruma::Kapalı,
    };
    seri.en_boy_yatay_hizası = match ham.get("preserveAspectAlign").and_then(Value::as_str) {
        Some("left") => GrafoEnBoyYatayHizası::Sol,
        Some("right") => GrafoEnBoyYatayHizası::Sağ,
        _ => GrafoEnBoyYatayHizası::Orta,
    };
    seri.en_boy_dikey_hizası = match ham
        .get("preserveAspectVerticalAlign")
        .and_then(Value::as_str)
    {
        Some("top") => GrafoEnBoyDikeyHizası::Üst,
        Some("bottom") => GrafoEnBoyDikeyHizası::Alt,
        _ => GrafoEnBoyDikeyHizası::Orta,
    };
    seri.sürüklenebilir = ham
        .get("draggable")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    seri.gösterge_vurgusu = ham
        .get("legendHoverLink")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    seri.sessiz = ham.get("silent").and_then(Value::as_bool).unwrap_or(false);
    seri.sembol = sembol(ham.get("symbol")).unwrap_or(Sembol::Daire);
    seri.sembol_boyutu = sayı(ham.get("symbolSize")).unwrap_or(10.0);
    if let Some(etiket) = etiket_yaması(ham.get("label")) {
        seri.etiket = etiket.uygula(&seri.etiket);
        seri.etiket_göster = seri.etiket.göster;
    }
    if let Some(etiket) = etiket_yaması(ham.get("edgeLabel")) {
        seri.kenar_etiketi = etiket.uygula(&seri.kenar_etiketi);
    }
    if let Some(stil) = öğe_stili(ham.get("itemStyle")) {
        seri.grafo_öğe_stili = stil;
    }
    if let Some(stil) = çizgi_stili(ham.get("lineStyle")) {
        let mut taban = seri.grafo_çizgi_stili.clone();
        if stil.renk.is_some() {
            taban.renk = stil.renk;
        }
        if stil.kalınlık.is_some() {
            taban.kalınlık = stil.kalınlık;
        }
        if stil.tür.is_some() {
            taban.tür = stil.tür;
        }
        if stil.opaklık.is_some() {
            taban.opaklık = stil.opaklık;
        }
        if stil.eğrilik.is_some() {
            taban.eğrilik = stil.eğrilik;
        }
        seri.grafo_çizgi_stili = taban;
    }
    if let Some(semboller) = ham.get("edgeSymbol") {
        if let Some(dizi) = semboller.as_array()
            && dizi.len() == 2
        {
            seri.kenar_sembolleri = [
                sembol(Some(&dizi[0])).unwrap_or(Sembol::Yok),
                sembol(Some(&dizi[1])).unwrap_or(Sembol::Yok),
            ];
        } else if let Some(sembol) = sembol(Some(semboller)) {
            seri.kenar_sembolleri = [sembol.clone(), sembol];
        }
    }
    if let Some(boyut) = ham.get("edgeSymbolSize") {
        if let Some(sayı) = boyut.as_f64() {
            seri.kenar_sembol_boyutları = [sayı as f32; 2];
        } else if let Some(dizi) = boyut.as_array()
            && dizi.len() == 2
        {
            seri.kenar_sembol_boyutları = [
                dizi[0].as_f64().unwrap_or(10.0) as f32,
                dizi[1].as_f64().unwrap_or(10.0) as f32,
            ];
        }
    }
    seri.vurgu = durum(ham.get("emphasis"));
    if let Some(eski_odak) = ham.get("focusNodeAdjacency").and_then(Value::as_bool)
        && seri.vurgu.odak.is_none()
    {
        seri = seri.eski_komşuluk_odağı(eski_odak);
    }
    seri.bulanık = durum(ham.get("blur"));
    seri.seçili = durum(ham.get("select"));
    seri.etiket_örtüşmesini_gizle = ham
        .get("labelLayout")
        .and_then(nesne)
        .and_then(|değer| değer.get("hideOverlap"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if let Some(dairesel) = ham.get("circular").and_then(nesne) {
        seri.dairesel.etiketi_döndür = dairesel
            .get("rotateLabel")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    }
    if let Some(kuvvet) = ham.get("force").and_then(nesne) {
        seri.kuvvet.itme = aralık(kuvvet.get("repulsion"), seri.kuvvet.itme);
        seri.kuvvet.kenar_uzunluğu = aralık(kuvvet.get("edgeLength"), seri.kuvvet.kenar_uzunluğu);
        seri.kuvvet.yerçekimi = sayı(kuvvet.get("gravity")).unwrap_or(0.1);
        seri.kuvvet.sürtünme = sayı(kuvvet.get("friction")).unwrap_or(0.6);
        seri.kuvvet.yerleşim_animasyonu = kuvvet
            .get("layoutAnimation")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        seri.kuvvet.başlangıç_yerleşimi = match kuvvet.get("initLayout").and_then(Value::as_str) {
            Some("circular") => Some(GrafoKuvvetBaşlangıcı::Dairesel),
            Some("none") => Some(GrafoKuvvetBaşlangıcı::Yok),
            _ => None,
        };
    }
    seri.z = ham.get("z").and_then(Value::as_i64).unwrap_or(2) as i32;
    seri.düğümler = ham
        .get("data")
        .or_else(|| ham.get("nodes"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(sıra, değer)| düğüm(değer, sıra, seri.sembol_boyutu))
        .collect();
    seri.kategoriler = ham
        .get("categories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(kategori)
        .collect();
    seri.ayrıntılı_bağlar = ham
        .get("links")
        .or_else(|| ham.get("edges"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(bağ)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(seri)
}

fn başlık(değer: &Value) -> Option<Başlık> {
    let ham = nesne(değer)?;
    let mut başlık = Başlık::yeni().iç_boşluk(5.0);
    if let Some(metin) = metin(ham.get("text")) {
        başlık = başlık.metin(metin);
    }
    if let Some(metin) = metin(ham.get("subtext")) {
        başlık = başlık.alt_metin(metin);
    }
    if let Some(sol) = ham.get("left") {
        if let Some(metin) = sol.as_str() {
            başlık = başlık.sol(metin);
        } else if let Some(sayı) = sol.as_f64() {
            başlık = başlık.sol(sayı as f32);
        }
    }
    if let Some(sağ) = ham.get("right") {
        if let Ok(sağ) = uzunluk(sağ) {
            başlık = başlık.sağ(sağ);
        }
    }
    if let Some(üst) = ham.get("top") {
        if let Some(metin) = üst.as_str() {
            // `title.top` CSS uzunluklarının yanında kutu-yerleşim
            // anahtar sözcüklerini de kabul eder.
            başlık = başlık.üst(metin);
        } else if let Some(sayı) = üst.as_f64() {
            başlık = başlık.üst(sayı as f32);
        }
    }
    if let Some(alt) = ham.get("bottom")
        && let Ok(alt) = uzunluk(alt)
    {
        başlık = başlık.alt(alt);
    }
    Some(başlık)
}

fn gösterge(değer: &Value) -> Option<Gösterge> {
    let değer = değer
        .as_array()
        .and_then(|dizi| dizi.first())
        .unwrap_or(değer);
    let ham = nesne(değer)?;
    let mut gösterge = Gösterge::yeni().iç_boşluk(5.0);
    if let Some(veri) = ham.get("data").and_then(Value::as_array) {
        gösterge.veri = veri.iter().filter_map(|değer| metin(Some(değer))).collect();
    }
    if let Some(sağ) = ham.get("right")
        && let Ok(sağ) = uzunluk(sağ)
    {
        gösterge = gösterge.sağ(sağ);
    }
    if let Some(kip) = ham.get("selectedMode") {
        gösterge.seçim_kipi = match kip.as_str() {
            Some("single") => GöstergeSeçimKipi::Tek,
            _ if kip.as_bool() == Some(false) => GöstergeSeçimKipi::Kapalı,
            _ => GöstergeSeçimKipi::Çoklu,
        };
    }
    if let Some(seçili) = ham.get("selected").and_then(nesne) {
        for (ad, değer) in seçili {
            if let Some(değer) = değer.as_bool() {
                gösterge.seçili.insert(ad.clone(), değer);
            }
        }
    }
    Some(gösterge)
}

fn eksen(değer: &Value) -> Option<Eksen> {
    let ham = nesne(değer)?;
    let mut eksen = if ham.get("type").and_then(Value::as_str) == Some("category") {
        Eksen::kategori()
    } else {
        Eksen::değer()
    };
    if let Some(veri) = ham.get("data").and_then(Value::as_array) {
        eksen.veri = veri.iter().filter_map(|değer| metin(Some(değer))).collect();
    }
    if let Some(kenar) = ham.get("boundaryGap").and_then(Value::as_bool) {
        eksen = eksen.kenar_boşluğu(kenar);
    }
    if let Some(ölçek) = ham.get("scale").and_then(Value::as_bool) {
        eksen = eksen.ölçekli(ölçek);
    }
    Some(eksen)
}

fn bileşenleri_uygula(
    mut seçenekler: GrafikSeçenekleri, seçenek: &Value
) -> GrafikSeçenekleri {
    let Some(ham) = nesne(seçenek) else {
        return seçenekler;
    };
    if let Some(başlık) = ham.get("title").and_then(başlık) {
        seçenekler = seçenekler.başlık(başlık);
    }
    if let Some(gösterge) = ham.get("legend").and_then(gösterge) {
        seçenekler = seçenekler.gösterge(gösterge);
    }
    if ham.get("tooltip").is_some() {
        seçenekler = seçenekler.ipucu(İpucu::yeni().tetikleme(Tetikleme::Öğe));
    }
    if let Some(ızgara) = ham.get("grid").and_then(nesne) {
        let mut çıktı = Izgara::yeni();
        if let Some(değer) = ızgara.get("left").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.sol(değer);
        }
        if let Some(değer) = ızgara.get("right").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.sağ(değer);
        }
        if let Some(değer) = ızgara.get("top").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.üst(değer);
        }
        if let Some(değer) = ızgara.get("bottom").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.alt(değer);
        }
        çıktı = çıktı.etiketi_kapsa(
            ızgara
                .get("containLabel")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
        seçenekler = seçenekler.ızgara(çıktı);
    }
    if let Some(x) = ham.get("xAxis").and_then(eksen) {
        seçenekler = seçenekler.x_ekseni(x);
    }
    if let Some(y) = ham.get("yAxis").and_then(eksen) {
        seçenekler = seçenekler.y_ekseni(y);
    }
    if let Some(eşleme) = ham.get("visualMap").and_then(nesne) {
        let mut çıktı = GörselEşleme::yeni();
        if let Some(en_az) = eşleme.get("min").and_then(Value::as_f64) {
            çıktı = çıktı.en_az(en_az);
        }
        if let Some(en_çok) = eşleme.get("max").and_then(Value::as_f64) {
            çıktı = çıktı.en_çok(en_çok);
        }
        if let Some(boyut) = eşleme.get("dimension").and_then(Value::as_u64) {
            çıktı = çıktı.boyut(boyut as usize);
        }
        if let Some(göster) = eşleme.get("show").and_then(Value::as_bool) {
            çıktı = çıktı.göster(göster);
        }
        seçenekler = seçenekler.görsel_eşleme(çıktı);
    }
    if ham.get("dataZoom").is_some() {
        seçenekler = seçenekler.veri_yakınlaştırma(VeriYakınlaştırma::iç());
    }
    if ham
        .get("toolbox")
        .and_then(nesne)
        .and_then(|v| v.get("feature"))
        .and_then(nesne)
        .is_some_and(|v| v.contains_key("dataZoom"))
    {
        seçenekler = seçenekler.araç_kutusu(AraçKutusu::yeni().veri_yakınlaştırma(true));
    }
    if let Some(küçük) = ham.get("thumbnail").and_then(nesne) {
        let stil = |değer: Option<&Value>, taban: KüçükResimStili| {
            let Some(ham) = değer.and_then(nesne) else {
                return taban;
            };
            let mut çıktı = taban;
            if let Some(renk) = ham.get("color").and_then(Value::as_str) {
                çıktı = çıktı.renk(renk);
            }
            if let Some(renk) = ham.get("borderColor").and_then(Value::as_str) {
                çıktı = çıktı.kenarlık_rengi(renk);
            }
            if let Some(kalınlık) = sayı(ham.get("borderWidth")) {
                çıktı = çıktı.kenarlık_kalınlığı(kalınlık);
            }
            if let Some(opaklık) = sayı(ham.get("opacity")) {
                çıktı = çıktı.opaklık(opaklık);
            }
            çıktı
        };
        let mut çıktı = KüçükResim::yeni();
        if let Some(göster) = küçük.get("show").and_then(Value::as_bool) {
            çıktı = çıktı.göster(göster);
        }
        if let Some(değer) = küçük.get("left").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.sol(değer);
        }
        if let Some(değer) = küçük.get("right").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.sağ(değer);
        }
        if let Some(değer) = küçük.get("top").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.üst(değer);
        }
        if let Some(değer) = küçük.get("bottom").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.alt(değer);
        }
        if let Some(değer) = küçük.get("width").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.genişlik(değer);
        }
        if let Some(değer) = küçük.get("height").and_then(|v| uzunluk(v).ok()) {
            çıktı = çıktı.yükseklik(değer);
        }
        çıktı.öğe_stili = stil(küçük.get("itemStyle"), çıktı.öğe_stili.clone());
        çıktı.pencere_stili = stil(küçük.get("windowStyle"), çıktı.pencere_stili.clone());
        seçenekler = seçenekler.küçük_resim(çıktı);
    }
    seçenekler
}

fn sahne_noktaları(sahne: &Value, seri_sırası: usize) -> Option<Vec<(f32, f32)>> {
    sahne
        .get("seriler")?
        .as_array()?
        .get(seri_sırası)?
        .get("düğümler")?
        .as_array()
        .map(|düğümler| {
            düğümler
                .iter()
                .map(|düğüm| {
                    (
                        sayı(düğüm.get("x")).unwrap_or(f32::NAN),
                        sayı(düğüm.get("y")).unwrap_or(f32::NAN),
                    )
                })
                .collect()
        })
}

pub fn resmi(id: &str) -> Result<GrafikSeçenekleri, String> {
    let belge: Value = serde_json::from_str(belge_kaynağı(id)?)
        .map_err(|hata| format!("{id} Graph belgesi çözülemedi: {hata}"))?;
    let sahne: Value = serde_json::from_str(sahne_kaynağı(id)?)
        .map_err(|hata| format!("{id} Graph sahnesi çözülemedi: {hata}"))?;
    let seçenek = belge
        .get("seçenek")
        .ok_or_else(|| format!("{id} Graph seçeneği eksik"))?;
    let mut seçenekler = bileşenleri_uygula(GrafikSeçenekleri::yeni().animasyon(false), seçenek);
    let ham_seriler = seçenek
        .get("series")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{id} Graph series eksik"))?;
    for (seri_sırası, ham) in ham_seriler.iter().enumerate() {
        let mut çıktı = seri(ham)?;
        if let Some(noktalar) = sahne_noktaları(&sahne, seri_sırası)
            && noktalar.len() == çıktı.düğümler.len()
        {
            çıktı.korunmuş_noktalar = Some(noktalar);
        }
        // Dinamik örnekte ECharts kuvvet çözücüsü güncellemeler arasında
        // Math.random tüketir. Kilitli son sahnenin gerçek 20 bağı, veri
        // çıkarıcısının kaynak-only 22 bağından bu nedenle farklıdır.
        if id == "graph-force-dynamic"
            && let Some(bağlar) = sahne
                .get("seriler")
                .and_then(Value::as_array)
                .and_then(|s| s.get(seri_sırası))
                .and_then(|s| s.get("bağlar"))
                .and_then(Value::as_array)
        {
            çıktı.ayrıntılı_bağlar = bağlar
                .iter()
                .filter_map(|bağ| {
                    Some(GrafoBağı::yeni(
                        metin(bağ.get("kaynak"))?,
                        metin(bağ.get("hedef"))?,
                    ))
                })
                .collect();
        }
        if çıktı.koordinat_sistemi == GrafoKoordinatSistemi::Kartezyen2B
            && let Some(x_ekseni) = seçenekler.x_ekseni.as_ref()
            && x_ekseni.tür == EksenTürü::Kategori
        {
            for (sıra, düğüm) in çıktı.düğümler.iter_mut().enumerate() {
                if let Some(ad) = x_ekseni.veri.get(sıra) {
                    düğüm.ad.clone_from(ad);
                }
            }
        }
        seçenekler = seçenekler.seri(çıktı);
    }
    Ok(seçenekler)
}

fn renk_kanalları(renk: Renk) -> [u8; 4] {
    [
        (renk.kırmızı * 255.0).round() as u8,
        (renk.yeşil * 255.0).round() as u8,
        (renk.mavi * 255.0).round() as u8,
        (renk.alfa * 255.0).round() as u8,
    ]
}

fn sembol_adı(sembol: &Sembol) -> &'static str {
    match sembol {
        Sembol::Daire => "circle",
        Sembol::Kare => "rect",
        Sembol::YuvarlakDikdörtgen => "roundRect",
        Sembol::Üçgen => "arrow",
        Sembol::Elmas => "diamond",
        Sembol::İçiBoşDaire => "emptyCircle",
        Sembol::SvgYolu(_) => "path",
        Sembol::Yok => "none",
    }
}

fn çizgi_türü_adı(tür: ÇizgiTürü) -> &'static str {
    match tür {
        ÇizgiTürü::Kesikli => "dashed",
        ÇizgiTürü::Noktalı => "dotted",
        _ => "solid",
    }
}

fn veri_json(değer: Option<&VeriDeğeri>, sayı: Option<f64>) -> Value {
    fn çevir(değer: &VeriDeğeri) -> Value {
        match değer {
            VeriDeğeri::Sayı(değer) => json!(değer),
            VeriDeğeri::Çift(değerler) => json!(değerler),
            VeriDeğeri::Dizi(değerler) => json!(değerler),
            VeriDeğeri::KarmaDizi(değerler) => Value::Array(değerler.iter().map(çevir).collect()),
            VeriDeğeri::Zaman(değer) => json!(değer),
            VeriDeğeri::Metin(değer) => json!(değer),
            VeriDeğeri::Mantıksal(değer) => json!(değer),
            VeriDeğeri::Boş => Value::Null,
        }
    }
    değer
        .map(çevir)
        .or_else(|| sayı.map(|v| json!(v)))
        .unwrap_or(Value::Null)
}

fn görünür_seriler(seçenekler: &GrafikSeçenekleri) -> HashSet<usize> {
    let mut görünür = (0..seçenekler.seriler.len()).collect::<HashSet<_>>();
    let Some(gösterge) = &seçenekler.gösterge else {
        return görünür;
    };
    if gösterge.seçim_kipi == GöstergeSeçimKipi::Tek {
        let etkin = gösterge
            .veri
            .iter()
            .find(|ad| gösterge.seçili_mi(ad))
            .cloned();
        görünür.retain(|sıra| {
            etkin
                .as_deref()
                .is_none_or(|ad| seçenekler.seriler[*sıra].ad() == Some(ad))
        });
    } else {
        görünür.retain(|sıra| {
            seçenekler.seriler[*sıra]
                .ad()
                .is_none_or(|ad| gösterge.seçili_mi(ad))
        });
    }
    görünür
}

pub fn sahne_kanıtı(
    seçenekler: &GrafikSeçenekleri,
    genişlik: f32,
    yükseklik: f32,
) -> Result<Value, String> {
    let tuval = cizelge::koordinat::Dikdörtgen::yeni(0.0, 0.0, genişlik, yükseklik);
    let görünür = görünür_seriler(seçenekler);
    let mut seriler = Vec::new();
    for (seri_sırası, kaynak) in seçenekler.seriler.iter().enumerate() {
        let Seri::Grafo(seri) = kaynak else {
            continue;
        };
        if !görünür.contains(&seri_sırası) {
            continue;
        }
        let palet = |sıra: usize| {
            if seri.kategoriler.is_empty() {
                seçenekler.seri_rengi(seri_sırası)
            } else {
                seçenekler.palet_rengi(sıra)
            }
        };
        let mut yerleşim = grafo_yerleşimi_kur(
            seri,
            tuval,
            &palet,
            (0.0, 0.0, 1.0),
            &[],
            None,
            &HashSet::new(),
        )
        .map_err(|hata| format!("Graph[{seri_sırası}] sahnesi kurulamadı: {hata}"))?;
        for eşleme in seçenekler.seri_görsel_eşlemeleri(seri_sırası) {
            let boyut = match eşleme.boyut.as_ref() {
                Some(BoyutSeçici::Sıra(sıra)) => *sıra,
                Some(BoyutSeçici::Ad(ad)) if ad == "value" => 0,
                Some(BoyutSeçici::Ad(ad)) => ad.parse::<usize>().unwrap_or(0),
                None => 0,
            };
            let mut kapsam = [f64::INFINITY, f64::NEG_INFINITY];
            for düğüm in &seri.düğümler {
                if let Some(değer) = düğüm.sayısal_boyut(boyut).or(düğüm.değer) {
                    kapsam[0] = kapsam[0].min(değer);
                    kapsam[1] = kapsam[1].max(değer);
                }
            }
            let kapsam = eşleme.kapsam_çöz(kapsam);
            for düğüm in &mut yerleşim.düğümler {
                let Some(değer) = seri
                    .düğümler
                    .get(düğüm.veri_sırası)
                    .and_then(|kaynak| kaynak.sayısal_boyut(boyut).or(kaynak.değer))
                else {
                    continue;
                };
                düğüm.renk = Dolgu::Düz(eşleme.rengi_uygula(değer, kapsam, düğüm.renk.temsilî()));
                düğüm.boyut = eşleme.sembol_boyutu_çöz(değer, kapsam, düğüm.boyut);
            }
        }
        let düğümler = yerleşim
            .düğümler
            .iter()
            .map(|düğüm| {
                let ham = &seri.düğümler[düğüm.veri_sırası];
                let göster = (düğüm.etiket.göster || seri.etiket_göster) && !düğüm.etiket_gizli;
                let sahne_adı = ham.kimlik.clone().unwrap_or_else(|| düğüm.kimlik.clone());
                let kenarlık = düğüm.öğe_stili.kenarlık_kalınlığı.unwrap_or(0.0);
                json!({
                    "veri_sırası": düğüm.veri_sırası,
                    "kimlik": düğüm.kimlik,
                    "ad": sahne_adı,
                    "değer": if seri.koordinat_sistemi == GrafoKoordinatSistemi::Görünüm {
                        veri_json(None, ham.değer)
                    } else {
                        Value::Null
                    },
                    "kategori": Value::Null,
                    "x": düğüm.konum.0,
                    "y": düğüm.konum.1,
                    "genişlik": ham.boyut_çifti.map_or(düğüm.boyut, |v| v[0]) + kenarlık,
                    "yükseklik": ham.boyut_çifti.map_or(düğüm.boyut, |v| v[1]) + kenarlık,
                    "sembol": sembol_adı(&düğüm.sembol),
                    "renk": renk_kanalları(düğüm.renk.temsilî()),
                    "kenarlık_rengi": düğüm.öğe_stili.kenarlık_rengi.map(renk_kanalları),
                    "kenarlık_kalınlığı": düğüm.öğe_stili.kenarlık_kalınlığı.unwrap_or(0.0),
                    "opaklık": düğüm.öğe_stili.opaklık.unwrap_or(1.0),
                    "sabit": düğüm.sabit,
                    "etiket": {
                        "göster": göster,
                        "metin": if göster { düğüm.etiket_metni.as_str() } else { "" },
                        "x": düğüm.etiket_konumu.0,
                        "y": düğüm.etiket_konumu.1,
                        // ECharts dairesel Graph etiket dönüşünü Text'in
                        // kendi `rotation` alanına değil, bağlı sembolün
                        // `textConfig` dönüşümüne yazar. Sahne özeti aynı
                        // dış temsili korurken gerçek çizim dönüşü yukarıdaki
                        // dünya çapasında uygulanmaya devam eder.
                        "dönüş": if seri.yerleşim == GrafoYerleşimi::Dairesel
                            && seri.dairesel.etiketi_döndür { 0.0 } else { düğüm.etiket_dönüşü },
                        "renk": düğüm.etiket.yazı.renk.map(renk_kanalları),
                    }
                })
            })
            .collect::<Vec<_>>();
        let bağlar = yerleşim
            .bağlar
            .iter()
            .map(|bağ| {
                let ham = &seri.ayrıntılı_bağlar[bağ.veri_sırası];
                let uç = |sembol: &Sembol, boyut: f32, konum: (f32, f32)| {
                    (!matches!(sembol, Sembol::Yok)).then(|| {
                        json!({
                            "sembol": sembol_adı(sembol),
                            "x": konum.0,
                            "y": konum.1,
                            "boyut": [boyut, boyut],
                        })
                    })
                };
                let göster = bağ.etiket.göster;
                json!({
                    "veri_sırası": bağ.veri_sırası,
                    "kaynak": bağ.kaynak,
                    "hedef": bağ.hedef,
                    "değer": ham.değer,
                    "x1": bağ.başlangıç.0,
                    "y1": bağ.başlangıç.1,
                    "x2": bağ.bitiş.0,
                    "y2": bağ.bitiş.1,
                    "cpx1": bağ.kontrol.map(|v| v.0),
                    "cpy1": bağ.kontrol.map(|v| v.1),
                    "renk": renk_kanalları(bağ.renk),
                    "kalınlık": bağ.çizgi_stili.kalınlık.unwrap_or(1.0),
                    "opaklık": bağ.çizgi_stili.opaklık.unwrap_or(0.5),
                    "tür": çizgi_türü_adı(bağ.çizgi_stili.tür.unwrap_or(ÇizgiTürü::Düz)),
                    "kaynak_sembolü": uç(
                        &bağ.kaynak_sembolü,
                        bağ.kaynak_sembol_boyutu,
                        bağ.başlangıç,
                    ),
                    "hedef_sembolü": uç(
                        &bağ.hedef_sembolü,
                        bağ.hedef_sembol_boyutu,
                        bağ.bitiş,
                    ),
                    "etiket": {
                        "göster": göster,
                        "metin": if göster { bağ.etiket_metni.as_str() } else { "" },
                        "x": bağ.etiket_konumu.0,
                        "y": bağ.etiket_konumu.1,
                        "dönüş": bağ.etiket_dönüşü,
                        "renk": bağ.etiket.yazı.renk.map(renk_kanalları),
                    }
                })
            })
            .collect::<Vec<_>>();
        let koordinat_sistemi = match seri.koordinat_sistemi {
            GrafoKoordinatSistemi::Görünüm => "view",
            GrafoKoordinatSistemi::Kartezyen2B => "cartesian2d",
            GrafoKoordinatSistemi::Kutupsal => "polar",
            GrafoKoordinatSistemi::TekEksen => "singleAxis",
            GrafoKoordinatSistemi::Takvim => "calendar",
            GrafoKoordinatSistemi::Matris => "matrix",
        };
        let sahne_adı = seri
            .ad
            .clone()
            .unwrap_or_else(|| format!("series\0{seri_sırası}"));
        seriler.push(json!({
            "seri_sırası": seri_sırası,
            "ad": sahne_adı,
            "koordinat_sistemi": koordinat_sistemi,
            "alan": {
                "x": yerleşim.veri_alanı.x,
                "y": yerleşim.veri_alanı.y,
                "genişlik": yerleşim.veri_alanı.genişlik,
                "yükseklik": yerleşim.veri_alanı.yükseklik,
            },
            "düğümler": düğümler,
            "bağlar": bağlar,
        }));
    }
    Ok(json!({
        "şema_sürümü": 1,
        "tür": "graph",
        "koordinat_adımı": 0.001,
        "seriler": seriler,
    }))
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn on_bir_resmi_graph_fixture_tum_dugum_ve_baglari_tasir() {
        let beklenen = [
            ("graph-force2", 16, 152, 151),
            ("graph-grid", 1, 7, 6),
            ("graph-simple", 1, 4, 6),
            ("graph-force", 1, 77, 254),
            ("graph-label-overlap", 1, 77, 254),
            ("graph", 1, 77, 254),
            ("graph-circular-layout", 1, 77, 254),
            ("graph-force-dynamic", 1, 26, 20),
            ("graph-life-expectancy", 19, 1539, 1520),
            ("graph-webkit-dep", 1, 492, 806),
            ("graph-npm", 1, 717, 942),
        ];
        for (id, seri_sayısı, düğüm_sayısı, bağ_sayısı) in beklenen {
            let seçenekler = resmi(id).unwrap_or_else(|hata| panic!("{id}: {hata}"));
            assert_eq!(seçenekler.seriler.len(), seri_sayısı, "{id}");
            let (düğümler, bağlar) =
                seçenekler
                    .seriler
                    .iter()
                    .fold((0, 0), |(düğüm, bağ), seri| match seri {
                        Seri::Grafo(seri) => (
                            düğüm + seri.düğümler.len(),
                            bağ + seri.ayrıntılı_bağlar.len(),
                        ),
                        _ => panic!("{id}: Graph bekleniyordu"),
                    });
            assert_eq!(düğümler, düğüm_sayısı, "{id}");
            assert_eq!(bağlar, bağ_sayısı, "{id}");
            seçenekler
                .doğrula()
                .unwrap_or_else(|hata| panic!("{id}: {hata}"));
        }
    }

    #[test]
    fn on_bir_kilitli_sahne_tum_graph_geometrisini_uretir() {
        for id in RESMİ_IDLER {
            let seçenekler = resmi(id).unwrap_or_else(|hata| panic!("{id}: {hata}"));
            let (genişlik, yükseklik) =
                if matches!(id, "graph-circular-layout" | "graph-webkit-dep") {
                    (900.0, 675.0)
                } else {
                    (700.0, 525.0)
                };
            let kanıt = sahne_kanıtı(&seçenekler, genişlik, yükseklik)
                .unwrap_or_else(|hata| panic!("{id}: {hata}"));
            assert_eq!(kanıt["tür"], "graph", "{id}");
            assert!(
                kanıt["seriler"].as_array().is_some_and(|s| !s.is_empty()),
                "{id}"
            );
        }
    }
}

// `examples/*.rs` Cargo tarafından bağımsız örnek hedefi olarak da
// keşfedilir; asıl kullanım `uyum_fixture` içindeki modüldür.
#[allow(dead_code)]
fn main() {}
