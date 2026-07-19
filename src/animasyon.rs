//! Animasyon yumuşatma işlevleri — `zrender/src/animation/easing.ts`
//! içindeki ilgili eğrilerin portu.

/// Yumuşatma eğrisi.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Yumuşatma {
    Doğrusal,
    /// ECharts serilerinin öntanımlı giriş animasyonu (`cubicOut`).
    #[default]
    KübikÇıkış,
    KübikGirişÇıkış,
    ElastikÇıkış,
}

impl Yumuşatma {
    /// `t ∈ [0, 1]` ilerlemesine eğriyi uygular.
    pub fn uygula(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Yumuşatma::Doğrusal => t,
            Yumuşatma::KübikÇıkış => {
                let k = t - 1.0;
                k * k * k + 1.0
            }
            Yumuşatma::KübikGirişÇıkış => {
                let k = t * 2.0;
                if k < 1.0 {
                    0.5 * k * k * k
                } else {
                    let k = k - 2.0;
                    0.5 * (k * k * k + 2.0)
                }
            }
            Yumuşatma::ElastikÇıkış => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.4;
                    let s = p / 4.0;
                    (2.0f32).powf(-10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin()
                        + 1.0
                }
            }
        }
    }
}

/// ECharts'ın öntanımlı giriş animasyonu süresi (ms), `animationDuration`.
pub const ÖNTANIMLI_SÜRE_MS: f32 = 1000.0;

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
    fn uç_değerler() {
        for y in [
            Yumuşatma::Doğrusal,
            Yumuşatma::KübikÇıkış,
            Yumuşatma::KübikGirişÇıkış,
            Yumuşatma::ElastikÇıkış,
        ] {
            assert!((y.uygula(0.0) - 0.0).abs() < 1e-6);
            assert!((y.uygula(1.0) - 1.0).abs() < 1e-6);
        }
    }
}
