//! Saçılım serisi çizimi — `echarts/src/chart/scatter` karşılığı.

use crate::cizim::Çizici;
use crate::grafik::sembol_çiz;
use crate::koordinat::Kartezyen2B;
use crate::model::seri::SaçılımSerisi;
use crate::renk::Renk;

/// Yerleşimi hesaplanmış bir saçılım noktası.
#[derive(Clone, Copy, Debug)]
pub struct SaçılımNoktası {
    pub sıra: usize,
    pub konum: (f32, f32),
    /// Sembol çapı.
    pub boyut: f32,
    pub x_değeri: f64,
    pub y_değeri: f64,
}

/// Serinin piksel noktalarını üretir. Veri `[x, y]` çifti değilse `x`
/// olarak veri sırası kullanılır.
pub fn saçılım_noktaları(
    seri: &SaçılımSerisi,
    kartezyen: &Kartezyen2B,
) -> Vec<SaçılımNoktası> {
    let mut sonuç = Vec::with_capacity(seri.veri.len());
    for (i, öğe) in seri.veri.iter().enumerate() {
        if öğe.değer.boş_mu() {
            continue;
        }
        let Some(y) = öğe.değer.sayı() else { continue };
        let x = öğe.değer.x().unwrap_or(i as f64);
        sonuç.push(SaçılımNoktası {
            sıra: i,
            konum: kartezyen.nokta(x, y),
            boyut: seri.sembol_boyutu.çöz(öğe),
            x_değeri: x,
            y_değeri: y,
        });
    }
    sonuç
}

/// Saçılım serisini çizer; `vurgulu` ipucuyla öne çıkarılan noktadır.
pub fn saçılım_çiz(
    çizici: &mut Çizici,
    seri: &SaçılımSerisi,
    noktalar: &[SaçılımNoktası],
    seri_rengi: Renk,
    ilerleme: f32,
    vurgulu: Option<usize>,
) {
    // ECharts saçılım öğeleri öntanımlı 0.8 opaklıkla çizilir.
    let opaklık = seri.öğe_stili.opaklık.unwrap_or(0.8);
    let renk = seri
        .öğe_stili
        .renk
        .as_ref()
        .map(|d| d.temsilî())
        .unwrap_or(seri_rengi);
    for nokta in noktalar {
        let vurgulu_mu = vurgulu == Some(nokta.sıra);
        let boyut = nokta.boyut * ilerleme.clamp(0.0, 1.0)
            * if vurgulu_mu { 1.15 } else { 1.0 };
        let renk = if vurgulu_mu {
            renk.opaklık(1.0)
        } else {
            renk.opaklık(opaklık)
        };
        sembol_çiz(çizici, seri.sembol, nokta.konum, boyut, renk);
    }
}
