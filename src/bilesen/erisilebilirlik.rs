//! Erişilebilirlik — `echarts/src/visual/aria.ts` karşılığı: seçeneklerden
//! ekran okuyucular için okunabilir bir özet metni üretir (ECharts'ta
//! `aria.enabled` ile `aria-label` niteliğine yazılan varsayılan cümleler).

use crate::model::secenekler::GrafikSeçenekleri;
use crate::model::seri::Seri;
use crate::yardimci::bicim::ondalık_kırp;

/// Özet cümlelerinde seri başına en çok kaç veri noktası sayılır
/// (`aria.data.maxCount` öntanımlısı).
const EN_ÇOK_VERİ: usize = 10;

/// Serinin okunur tür adı (etkin yerele göre).
pub fn seri_tür_adı(seri: &Seri) -> &'static str {
    let tr = crate::yerel::etkin_yerel().kod == "tr";
    match seri {
        Seri::Çizgi(_) => {
            if tr {
                "çizgi"
            } else {
                "line"
            }
        }
        Seri::Sütun(_) => {
            if tr {
                "sütun"
            } else {
                "bar"
            }
        }
        Seri::Pasta(_) => {
            if tr {
                "pasta"
            } else {
                "pie"
            }
        }
        Seri::Saçılım(_) => {
            if tr {
                "saçılım"
            } else {
                "scatter"
            }
        }
        Seri::Mum(_) => {
            if tr {
                "şamdan (mum)"
            } else {
                "candlestick"
            }
        }
        Seri::Kutu(_) => {
            if tr {
                "kutu"
            } else {
                "boxplot"
            }
        }
        Seri::Isı(_) => {
            if tr {
                "ısı haritası"
            } else {
                "heatmap"
            }
        }
        Seri::Huni(_) => {
            if tr {
                "huni"
            } else {
                "funnel"
            }
        }
        Seri::GöstergeSaati(_) => {
            if tr {
                "gösterge saati"
            } else {
                "gauge"
            }
        }
        Seri::Radar(_) => "radar",
        Seri::Özel(_) => {
            if tr {
                "özel"
            } else {
                "custom"
            }
        }
        Seri::AğaçHaritası(_) => {
            if tr {
                "ağaç haritası"
            } else {
                "treemap"
            }
        }
        Seri::GüneşPatlaması(_) => {
            if tr {
                "güneş patlaması"
            } else {
                "sunburst"
            }
        }
        Seri::Ağaç(_) => {
            if tr {
                "ağaç"
            } else {
                "tree"
            }
        }
        Seri::Sankey(_) => "sankey",
        Seri::Grafo(_) => {
            if tr {
                "ilişki (grafo)"
            } else {
                "graph"
            }
        }
        Seri::Kiriş(_) => {
            if tr {
                "kiriş"
            } else {
                "chord"
            }
        }
        Seri::Paralel(_) => {
            if tr {
                "paralel koordinat"
            } else {
                "parallel"
            }
        }
        Seri::Takvim(_) => {
            if tr {
                "takvim ısı haritası"
            } else {
                "calendar heatmap"
            }
        }
        Seri::TemaNehri(_) => {
            if tr {
                "tema nehri"
            } else {
                "theme river"
            }
        }
        Seri::Hatlar(_) => {
            if tr {
                "bağlantı çizgileri"
            } else {
                "lines"
            }
        }
    }
}

/// Kategori etiketini bul: öğenin adı, yoksa x ekseni kategorisi, o da
/// yoksa sıra numarası.
fn nokta_adı(seçenekler: &GrafikSeçenekleri, seri: &Seri, sıra: usize) -> String {
    if let Some(ad) = seri.veri().get(sıra).and_then(|ö| ö.ad.clone()) {
        return ad;
    }
    if seri.kartezyen_mi() {
        let eksenler = seçenekler.etkin_x_eksenleri();
        let bağ = seri.eksen_bağı();
        if let Some(etiket) = eksenler.get(bağ.x).and_then(|e| e.veri.get(sıra)) {
            return etiket.clone();
        }
    }
    format!("{}", sıra.saturating_add(1))
}

/// Seçeneklerden erişilebilirlik özeti üretir (ekran okuyucu metni).
///
/// Etkin yerel `seçenekler.yerel`den alınır; cümle kalıpları Türkçe ve
/// İngilizce olarak sunulur (`i18n/langTR.aria` / `langEN.aria` karşılığı).
pub fn erişilebilirlik_özeti(seçenekler: &GrafikSeçenekleri) -> String {
    crate::yerel::yerel_ayarla(seçenekler.yerel);
    let tr = seçenekler.yerel.kod == "tr";
    let mut cümleler: Vec<String> = Vec::new();

    // 1) Başlık.
    let başlık = seçenekler
        .başlıklar
        .first()
        .or(seçenekler.başlık.as_ref())
        .and_then(|b| b.metin.as_deref())
        .map(str::trim)
        .filter(|m| !m.is_empty());
    match başlık {
        Some(m) if tr => cümleler.push(format!("Bu, “{m}” başlıklı bir grafiktir.")),
        Some(m) => cümleler.push(format!("This is a chart about “{m}”.")),
        None if tr => cümleler.push("Bu bir grafiktir.".to_string()),
        None => cümleler.push("This is a chart.".to_string()),
    }

    // 2) Seri sayısı.
    let sayı = seçenekler.seriler.len();
    if sayı == 0 {
        cümleler.push(if tr {
            "Grafikte seri yok.".to_string()
        } else {
            "The chart has no series.".to_string()
        });
        return cümleler.join(" ");
    }
    if sayı > 1 {
        cümleler.push(if tr {
            format!("{sayı} seri içerir.")
        } else {
            format!("It consists of {sayı} series.")
        });
    }

    // 3) Seri başına tür, ad ve veriler.
    for (s, seri) in seçenekler.seriler.iter().enumerate() {
        let tür = seri_tür_adı(seri);
        let ad = seri.ad().map(str::trim).filter(|a| !a.is_empty());
        let mut cümle = match (tr, ad) {
            (true, Some(ad)) => {
                format!(
                    "{}. seri, “{ad}” adlı {tür} türünde bir seridir",
                    s.saturating_add(1)
                )
            }
            (true, None) => format!("{}. seri {tür} türündedir", s.saturating_add(1)),
            (false, Some(ad)) => format!(
                "The series {} is of type {tür} and is named “{ad}”",
                s.saturating_add(1)
            ),
            (false, None) => {
                format!("The series {} is of type {tür}", s.saturating_add(1))
            }
        };
        let veri = seri.veri();
        if let Seri::Hatlar(hatlar) = seri {
            if !hatlar.veri.is_empty() {
                cümle.push_str(if tr {
                    " ve şu bağlantıları içerir: "
                } else {
                    " with the connections: "
                });
                let parçalar: Vec<String> = hatlar
                    .veri
                    .iter()
                    .take(EN_ÇOK_VERİ)
                    .enumerate()
                    .map(|(sıra, hat)| {
                        hat.ad
                            .clone()
                            .or_else(|| match (&hat.kaynak_adı, &hat.hedef_adı) {
                                (Some(kaynak), Some(hedef)) => Some(format!("{kaynak} > {hedef}")),
                                _ => None,
                            })
                            .unwrap_or_else(|| sıra.saturating_add(1).to_string())
                    })
                    .collect();
                cümle.push_str(&parçalar.join(", "));
            }
            cümle.push('.');
            cümleler.push(cümle);
            continue;
        }
        if !veri.is_empty() {
            cümle.push_str(if tr {
                " ve şu verileri içerir: "
            } else {
                " with the data: "
            });
            let parçalar: Vec<String> = veri
                .iter()
                .take(EN_ÇOK_VERİ)
                .enumerate()
                .map(|(i, öğe)| {
                    let ad = nokta_adı(seçenekler, seri, i);
                    match öğe.değer.sayı() {
                        Some(d) => format!("{ad}: {}", ondalık_kırp(d)),
                        None => ad,
                    }
                })
                .collect();
            cümle.push_str(&parçalar.join(", "));
            if veri.len() > EN_ÇOK_VERİ {
                let kuyruk = if tr {
                    format!(" (ilk {EN_ÇOK_VERİ} nokta; toplam {})", veri.len())
                } else {
                    format!(" (first {EN_ÇOK_VERİ} of {})", veri.len())
                };
                cümle.push_str(&kuyruk);
            }
        }
        cümle.push('.');
        cümleler.push(cümle);
    }
    cümleler.join(" ")
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
    use crate::model::bilesen::Başlık;
    use crate::model::eksen::Eksen;
    use crate::model::seri::{SütunSerisi, ÇizgiSerisi};

    #[test]
    fn özet_türkçe() {
        let seçenekler = GrafikSeçenekleri::yeni()
            .başlık(Başlık::yeni().metin("Haftalık Satış"))
            .x_ekseni(Eksen::kategori().veri(["Pzt", "Sal"]))
            .seri(SütunSerisi::yeni().ad("Satış").veri([120.0, 80.0]))
            .seri(ÇizgiSerisi::yeni().veri([10.0, 20.0]));
        let özet = erişilebilirlik_özeti(&seçenekler);
        assert!(özet.contains("“Haftalık Satış” başlıklı"), "{özet}");
        assert!(özet.contains("2 seri içerir."), "{özet}");
        assert!(özet.contains("“Satış” adlı sütun türünde"), "{özet}");
        assert!(özet.contains("Pzt: 120"), "{özet}");
        assert!(özet.contains("2. seri çizgi türündedir"), "{özet}");
    }

    #[test]
    fn özet_ingilizce_ve_boş() {
        let seçenekler = GrafikSeçenekleri::yeni().yerel(&crate::yerel::İNGİLİZCE);
        let özet = erişilebilirlik_özeti(&seçenekler);
        assert_eq!(özet, "This is a chart. The chart has no series.");
        crate::yerel::yerel_ayarla(&crate::yerel::TÜRKÇE);
    }

    #[test]
    fn uzun_veri_kırpılır() {
        let veri: Vec<f64> = (0..25).map(|i| i as f64).collect();
        let seçenekler = GrafikSeçenekleri::yeni().seri(ÇizgiSerisi::yeni().ad("Uzun").veri(veri));
        let özet = erişilebilirlik_özeti(&seçenekler);
        assert!(özet.contains("ilk 10 nokta; toplam 25"), "{özet}");
    }
}
