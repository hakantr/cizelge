//! Tema — `echarts/src/visual/tokens.ts` renk belirteçlerinin karşılığı.
//!
//! Açık ve koyu tema aynı belirteç adlarıyla sunulur; etkin kip, boyama
//! başında [`koyu_ayarla`] ile seçilir (gpui arayüzü tek iş parçacıklıdır,
//! kip iş-parçacığı yereli tutulur). Renklere erişim fonksiyonlarladır
//! (`tema::eksen_çizgisi()` gibi) — böylece aynı çizim kodu her iki temada
//! doğru renkleri üretir.

use std::cell::Cell;

use crate::renk::Renk;

/// Öntanımlı seri renk paleti (`tokens.color.theme`) — iki temada ortak.
pub const PALET: [Renk; 9] = [
    Renk::onaltılık(0x5070dd),
    Renk::onaltılık(0xb6d634),
    Renk::onaltılık(0x505372),
    Renk::onaltılık(0xff994d),
    Renk::onaltılık(0x0ca8df),
    Renk::onaltılık(0xffd10a),
    Renk::onaltılık(0xfb628b),
    Renk::onaltılık(0x785db0),
    Renk::onaltılık(0x3fbe95),
];

// Yazı boyutları (`tokens.size`) — tema bağımsız.
pub const YAZI_KÜÇÜK: f32 = 12.0;
pub const YAZI_ORTA: f32 = 14.0;
pub const YAZI_BÜYÜK: f32 = 16.0;
pub const BAŞLIK_BOYUTU: f32 = 18.0;
pub const ALT_BAŞLIK_BOYUTU: f32 = 12.0;

std::thread_local! {
    static KOYU_KIP: Cell<bool> = const { Cell::new(false) };
}

/// Etkin temayı seçer; `grafiği_boya` her karede seçeneklerden çağırır.
pub fn koyu_ayarla(koyu: bool) {
    KOYU_KIP.with(|k| k.set(koyu));
}

/// Etkin kip koyu mu?
pub fn koyu_mu() -> bool {
    KOYU_KIP.with(|k| k.get())
}

/// Tek temanın renk belirteçleri.
pub struct TemaRenkleri {
    pub zemin: Renk,
    pub nötr_00: Renk,
    pub nötr_05: Renk,
    pub nötr_10: Renk,
    pub nötr_15: Renk,
    pub nötr_20: Renk,
    pub nötr_30: Renk,
    pub nötr_40: Renk,
    pub nötr_50: Renk,
    pub nötr_60: Renk,
    pub nötr_70: Renk,
    pub nötr_80: Renk,
    pub nötr_90: Renk,
    pub birincil_metin: Renk,
    pub ikincil_metin: Renk,
    pub üçüncül_metin: Renk,
    pub devre_dışı: Renk,
    pub eksen_çizgisi: Renk,
    pub eksen_çentiği: Renk,
    pub eksen_ara_çentiği: Renk,
    pub eksen_etiketi: Renk,
    pub bölme_çizgisi: Renk,
    pub ara_bölme_çizgisi: Renk,
    pub bölme_alanı: [Renk; 2],
    pub vurgu: Renk,
    pub imleç_gölgesi: Renk,
    pub imleç_çizgisi: Renk,
    pub ipucu_arkaplanı: Renk,
    pub ipucu_kenarlığı: Renk,
    pub ipucu_metni: Renk,
    pub ipucu_gölgesi: Renk,
}

/// Açık tema (`tokens.color`).
pub const AÇIK: TemaRenkleri = TemaRenkleri {
    zemin: Renk::onaltılık(0xffffff),
    nötr_00: Renk::onaltılık(0xffffff),
    nötr_05: Renk::onaltılık(0xf4f7fd),
    nötr_10: Renk::onaltılık(0xe8ebf0),
    nötr_15: Renk::onaltılık(0xdbdee4),
    nötr_20: Renk::onaltılık(0xcfd2d7),
    nötr_30: Renk::onaltılık(0xb7b9be),
    nötr_40: Renk::onaltılık(0x9ea0a5),
    nötr_50: Renk::onaltılık(0x86878c),
    nötr_60: Renk::onaltılık(0x6d6e73),
    nötr_70: Renk::onaltılık(0x54555a),
    nötr_80: Renk::onaltılık(0x3c3c41),
    nötr_90: Renk::onaltılık(0x232328),
    birincil_metin: Renk::onaltılık(0x3c3c41),
    ikincil_metin: Renk::onaltılık(0x54555a),
    üçüncül_metin: Renk::onaltılık(0x6d6e73),
    devre_dışı: Renk::onaltılık(0xcfd2d7),
    eksen_çizgisi: Renk::onaltılık(0x54555a),
    eksen_çentiği: Renk::onaltılık(0x54555a),
    eksen_ara_çentiği: Renk::onaltılık(0x6d6e73),
    eksen_etiketi: Renk::onaltılık(0x54555a),
    bölme_çizgisi: Renk::onaltılık(0xdbdee4),
    ara_bölme_çizgisi: Renk::onaltılık(0xf4f7fd),
    bölme_alanı: [
        // ECharts 6.1 `tokens.color.backgroundTint`, ardından tam saydam
        // `backgroundTransparent`. Sıra önemlidir: ilk kategori bandı
        // renkli başlar.
        // tiny-skia kaynak-üstü harmanlaması yarım değeri yukarı yuvarlar;
        // 233 girdisi Canvas çıktısı olan birleşik 244'e karşılık gelir.
        Renk::kyma(233.0 / 255.0, 237.0 / 255.0, 245.0 / 255.0, 0.5),
        Renk::kyma(1.0, 1.0, 1.0, 0.0),
    ],
    vurgu: Renk::kyma(1.0, 231.0 / 255.0, 130.0 / 255.0, 0.8),
    imleç_gölgesi: Renk::kyma(150.0 / 255.0, 150.0 / 255.0, 150.0 / 255.0, 0.2),
    imleç_çizgisi: Renk::onaltılık(0x9ea0a5),
    ipucu_arkaplanı: Renk::onaltılık(0xffffff),
    ipucu_kenarlığı: Renk::onaltılık(0xb7b9be),
    ipucu_metni: Renk::onaltılık(0x6d6e73),
    ipucu_gölgesi: Renk::kyma(0.0, 0.0, 0.0, 0.2),
};

/// Koyu tema (`tokens.darkColor` yaklaşımı: nötr skala tersine döner).
pub const KOYU: TemaRenkleri = TemaRenkleri {
    zemin: Renk::onaltılık(0x141418),
    nötr_00: Renk::onaltılık(0x141418),
    nötr_05: Renk::onaltılık(0x17171b),
    nötr_10: Renk::onaltılık(0x232328),
    nötr_15: Renk::onaltılık(0x303034),
    nötr_20: Renk::onaltılık(0x3c3c41),
    nötr_30: Renk::onaltılık(0x54555a),
    nötr_40: Renk::onaltılık(0x6d6e73),
    nötr_50: Renk::onaltılık(0x86878c),
    nötr_60: Renk::onaltılık(0x9ea0a5),
    nötr_70: Renk::onaltılık(0xb7b9be),
    nötr_80: Renk::onaltılık(0xcfd2d7),
    nötr_90: Renk::onaltılık(0xe8ebf0),
    birincil_metin: Renk::onaltılık(0xe8ebf0),
    ikincil_metin: Renk::onaltılık(0xb7b9be),
    üçüncül_metin: Renk::onaltılık(0x9ea0a5),
    devre_dışı: Renk::onaltılık(0x54555a),
    eksen_çizgisi: Renk::onaltılık(0xb7b9be),
    eksen_çentiği: Renk::onaltılık(0xb7b9be),
    eksen_ara_çentiği: Renk::onaltılık(0x9ea0a5),
    eksen_etiketi: Renk::onaltılık(0xb7b9be),
    bölme_çizgisi: Renk::onaltılık(0x303034),
    ara_bölme_çizgisi: Renk::onaltılık(0x1b1b20),
    bölme_alanı: [
        Renk::kyma(1.0, 1.0, 1.0, 0.02),
        Renk::kyma(1.0, 1.0, 1.0, 0.05),
    ],
    vurgu: Renk::kyma(1.0, 231.0 / 255.0, 130.0 / 255.0, 0.8),
    imleç_gölgesi: Renk::kyma(1.0, 1.0, 1.0, 0.08),
    imleç_çizgisi: Renk::onaltılık(0x6d6e73),
    ipucu_arkaplanı: Renk::onaltılık(0x232328),
    ipucu_kenarlığı: Renk::onaltılık(0x3c3c41),
    ipucu_metni: Renk::onaltılık(0xe8ebf0),
    ipucu_gölgesi: Renk::kyma(0.0, 0.0, 0.0, 0.5),
};

/// Etkin temanın belirteçleri.
pub fn renkler() -> &'static TemaRenkleri {
    if koyu_mu() { &KOYU } else { &AÇIK }
}

/// Palet içinden sıra numarasıyla renk seçer (dolanarak).
pub fn palet_rengi(sıra: usize) -> Renk {
    PALET
        .get(sıra % PALET.len())
        .copied()
        .unwrap_or_else(|| renkler().nötr_50)
}

// ---------------------------------------------------------------------
// Belirteç erişimcileri — çizim kodunun tek başvuru noktası.
// ---------------------------------------------------------------------

pub fn zemin() -> Renk {
    renkler().zemin
}

pub fn nötr_00() -> Renk {
    renkler().nötr_00
}

pub fn nötr_05() -> Renk {
    renkler().nötr_05
}

pub fn nötr_10() -> Renk {
    renkler().nötr_10
}

pub fn nötr_15() -> Renk {
    renkler().nötr_15
}

pub fn nötr_20() -> Renk {
    renkler().nötr_20
}

pub fn nötr_30() -> Renk {
    renkler().nötr_30
}

pub fn nötr_40() -> Renk {
    renkler().nötr_40
}

pub fn nötr_50() -> Renk {
    renkler().nötr_50
}

pub fn nötr_60() -> Renk {
    renkler().nötr_60
}

pub fn nötr_70() -> Renk {
    renkler().nötr_70
}

pub fn nötr_80() -> Renk {
    renkler().nötr_80
}

pub fn nötr_90() -> Renk {
    renkler().nötr_90
}

/// Timeline ve benzeri vurgu bileşenlerinin ECharts `accent*` belirteçleri.
/// Koyu karşılıklar `tokens.ts` içindeki doygunluk/aydınlık dönüşümünün
/// sabitlenmiş sonuçlarıdır.
pub fn aksan_10() -> Renk {
    if koyu_mu() {
        Renk::onaltılık(0x4e5777)
    } else {
        Renk::onaltılık(0xe0e4f2)
    }
}

pub fn aksan_20() -> Renk {
    if koyu_mu() {
        Renk::onaltılık(0x5e6b93)
    } else {
        Renk::onaltılık(0xc0c9e6)
    }
}

pub fn aksan_30() -> Renk {
    if koyu_mu() {
        Renk::onaltılık(0x7782a6)
    } else {
        Renk::onaltılık(0xa1aed9)
    }
}

pub fn aksan_40() -> Renk {
    if koyu_mu() {
        Renk::onaltılık(0x919ab8)
    } else {
        Renk::onaltılık(0x8292cc)
    }
}

pub fn aksan_50() -> Renk {
    if koyu_mu() {
        Renk::onaltılık(0xafb5c9)
    } else {
        Renk::onaltılık(0x6578ba)
    }
}

pub fn aksan_60() -> Renk {
    if koyu_mu() {
        Renk::onaltılık(0xd0d3dc)
    } else {
        Renk::onaltılık(0x536298)
    }
}

pub fn aksan_70() -> Renk {
    if koyu_mu() {
        Renk::onaltılık(0xeeeff3)
    } else {
        Renk::onaltılık(0x404c76)
    }
}

pub fn birincil_metin() -> Renk {
    renkler().birincil_metin
}

pub fn ikincil_metin() -> Renk {
    renkler().ikincil_metin
}

pub fn üçüncül_metin() -> Renk {
    renkler().üçüncül_metin
}

pub fn devre_dışı() -> Renk {
    renkler().devre_dışı
}

pub fn eksen_çizgisi() -> Renk {
    renkler().eksen_çizgisi
}

pub fn eksen_çentiği() -> Renk {
    renkler().eksen_çentiği
}

pub fn eksen_ara_çentiği() -> Renk {
    renkler().eksen_ara_çentiği
}

pub fn eksen_etiketi() -> Renk {
    renkler().eksen_etiketi
}

pub fn bölme_çizgisi() -> Renk {
    renkler().bölme_çizgisi
}

pub fn ara_bölme_çizgisi() -> Renk {
    renkler().ara_bölme_çizgisi
}

pub fn bölme_alanı_renkleri() -> [Renk; 2] {
    renkler().bölme_alanı
}

pub fn vurgu() -> Renk {
    renkler().vurgu
}

pub fn imleç_gölgesi() -> Renk {
    renkler().imleç_gölgesi
}

pub fn imleç_çizgisi() -> Renk {
    renkler().imleç_çizgisi
}

pub fn ipucu_arkaplanı() -> Renk {
    renkler().ipucu_arkaplanı
}

pub fn ipucu_kenarlığı() -> Renk {
    renkler().ipucu_kenarlığı
}

pub fn ipucu_metni() -> Renk {
    renkler().ipucu_metni
}

pub fn ipucu_gölgesi() -> Renk {
    renkler().ipucu_gölgesi
}

/// Yazı boyutu erişimcileri (tema bağımsız; tekdüzelik için fonksiyon).
pub fn yazı_küçük() -> f32 {
    YAZI_KÜÇÜK
}

pub fn yazı_orta() -> f32 {
    YAZI_ORTA
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
    fn kip_geçişi() {
        koyu_ayarla(false);
        let açık_metin = birincil_metin();
        koyu_ayarla(true);
        let koyu_metin = birincil_metin();
        koyu_ayarla(false);
        assert_ne!(açık_metin, koyu_metin);
        assert_eq!(birincil_metin(), açık_metin);
    }
}
