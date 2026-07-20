//! `heatmap-large` resmî örneğindeki noisejs `perlin2` yordamının küçük,
//! belirlenimci Rust karşılığı.

const GRADYANLAR: [(f64, f64); 12] = [
    (1.0, 1.0),
    (-1.0, 1.0),
    (1.0, -1.0),
    (-1.0, -1.0),
    (1.0, 0.0),
    (-1.0, 0.0),
    (1.0, 0.0),
    (-1.0, 0.0),
    (0.0, 1.0),
    (0.0, -1.0),
    (0.0, 1.0),
    (0.0, -1.0),
];

#[rustfmt::skip]
const PERMÜTASYON_TABANI: [usize; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225,
    140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148,
    247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32,
    57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122,
    60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54,
    65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169,
    200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3,
    64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85,
    212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
    213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43,
    172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185,
    112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191,
    179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31,
    181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150,
    254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195,
    78, 66, 215, 61, 156, 180,
];

pub struct Perlin2 {
    permütasyon: [usize; 512],
    gradyanlar: [(f64, f64); 512],
}

impl Perlin2 {
    /// noisejs `seed`: `0..1` tohumu 16 bit alana taşır ve permütasyon
    /// tablosunu aynı XOR sırasıyla iki kez doldurur.
    pub fn yeni(mut tohum: f64) -> Self {
        if tohum > 0.0 && tohum < 1.0 {
            tohum *= 65_536.0;
        }
        let mut tohum = tohum.floor() as u32;
        if tohum < 256 {
            tohum |= tohum << 8;
        }
        let mut permütasyon = [0_usize; 512];
        let mut gradyanlar = [(0.0, 0.0); 512];
        for (sıra, taban) in PERMÜTASYON_TABANI.into_iter().enumerate() {
            let maske = if sıra & 1 == 1 {
                (tohum & 255) as usize
            } else {
                ((tohum >> 8) & 255) as usize
            };
            let değer = taban ^ maske;
            permütasyon[sıra] = değer;
            permütasyon[sıra + 256] = değer;
            let gradyan = GRADYANLAR[değer % GRADYANLAR.len()];
            gradyanlar[sıra] = gradyan;
            gradyanlar[sıra + 256] = gradyan;
        }
        Self {
            permütasyon,
            gradyanlar,
        }
    }

    pub fn değer(&self, x: f64, y: f64) -> f64 {
        let hücre_x = (x.floor() as i64 & 255) as usize;
        let hücre_y = (y.floor() as i64 & 255) as usize;
        let yerel_x = x - x.floor();
        let yerel_y = y - y.floor();

        let g00 = self.gradyanlar[hücre_x + self.permütasyon[hücre_y]];
        let g01 = self.gradyanlar[hücre_x + self.permütasyon[hücre_y + 1]];
        let g10 = self.gradyanlar[hücre_x + 1 + self.permütasyon[hücre_y]];
        let g11 = self.gradyanlar[hücre_x + 1 + self.permütasyon[hücre_y + 1]];
        let n00 = g00.0 * yerel_x + g00.1 * yerel_y;
        let n01 = g01.0 * yerel_x + g01.1 * (yerel_y - 1.0);
        let n10 = g10.0 * (yerel_x - 1.0) + g10.1 * yerel_y;
        let n11 = g11.0 * (yerel_x - 1.0) + g11.1 * (yerel_y - 1.0);
        let u = soldur(yerel_x);
        doğrusal(
            doğrusal(n00, n10, u),
            doğrusal(n01, n11, u),
            soldur(yerel_y),
        )
    }
}

fn soldur(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn doğrusal(a: f64, b: f64, t: f64) -> f64 {
    (1.0 - t) * a + t * b
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn noisejs_sabit_tohum_degerlerini_izler() {
        let gürültü = Perlin2::yeni(26_973.0);
        for ((x, y), beklenen) in [
            ((0.0, 0.0), 0.5),
            ((1.0 / 40.0, 1.0 / 20.0), 0.474_108_293_533_851_3),
            ((57.0 / 40.0, 42.0 / 20.0), 0.394_157_609_920_351_4),
            ((5.0, 5.0), 0.5),
        ] {
            assert!((gürültü.değer(x, y) + 0.5 - beklenen).abs() < 1e-14);
        }
    }
}
