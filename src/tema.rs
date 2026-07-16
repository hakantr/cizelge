//! Öntanımlı tema — `echarts/src/visual/tokens.ts` içindeki renk
//! belirteçlerinin birebir karşılığı.

use crate::renk::Renk;

/// Öntanımlı seri renk paleti (`tokens.color.theme`).
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

// Nötr tonlar (`tokens.color.neutralXX`).
pub const NÖTR_00: Renk = Renk::onaltılık(0xffffff);
pub const NÖTR_05: Renk = Renk::onaltılık(0xf4f7fd);
pub const NÖTR_10: Renk = Renk::onaltılık(0xe8ebf0);
pub const NÖTR_15: Renk = Renk::onaltılık(0xdbdee4);
pub const NÖTR_20: Renk = Renk::onaltılık(0xcfd2d7);
pub const NÖTR_30: Renk = Renk::onaltılık(0xb7b9be);
pub const NÖTR_40: Renk = Renk::onaltılık(0x9ea0a5);
pub const NÖTR_50: Renk = Renk::onaltılık(0x86878c);
pub const NÖTR_60: Renk = Renk::onaltılık(0x6d6e73);
pub const NÖTR_70: Renk = Renk::onaltılık(0x54555a);
pub const NÖTR_80: Renk = Renk::onaltılık(0x3c3c41);
pub const NÖTR_90: Renk = Renk::onaltılık(0x232328);

// Anlamsal metin renkleri.
pub const BİRİNCİL_METİN: Renk = NÖTR_80;
pub const İKİNCİL_METİN: Renk = NÖTR_70;
pub const ÜÇÜNCÜL_METİN: Renk = NÖTR_60;
pub const DEVRE_DIŞI: Renk = NÖTR_20;

// Eksen belirteçleri (`tokens.color.axis*`).
pub const EKSEN_ÇİZGİSİ: Renk = NÖTR_70;
pub const EKSEN_ÇENTİĞİ: Renk = NÖTR_70;
/// Ara çentik rengi (`axisTickMinor`).
pub const EKSEN_ARA_ÇENTİĞİ: Renk = NÖTR_60;
pub const EKSEN_ETİKETİ: Renk = NÖTR_70;
pub const BÖLME_ÇİZGİSİ: Renk = NÖTR_15;
/// Ara bölme çizgisi rengi (`axisMinorSplitLine`).
pub const ARA_BÖLME_ÇİZGİSİ: Renk = NÖTR_05;
/// Bölme alanı (`splitArea`) dönüşümlü bant renkleri.
pub const BÖLME_ALANI_RENKLERİ: [Renk; 2] = [
    Renk::kyma(250.0 / 255.0, 250.0 / 255.0, 250.0 / 255.0, 0.2),
    Renk::kyma(210.0 / 255.0, 219.0 / 255.0, 238.0 / 255.0, 0.2),
];

/// Vurgu rengi (`tokens.color.highlight`).
pub const VURGU: Renk = Renk::kyma(1.0, 231.0 / 255.0, 130.0 / 255.0, 0.8);

/// Eksen imleci gölge dolgusu (ECharts `axisPointer shadowStyle`).
pub const İMLEÇ_GÖLGESİ: Renk = Renk::kyma(150.0 / 255.0, 150.0 / 255.0, 150.0 / 255.0, 0.2);

/// Eksen imleci çizgisi.
pub const İMLEÇ_ÇİZGİSİ: Renk = NÖTR_40;

// İpucu penceresi.
pub const İPUCU_ARKAPLANI: Renk = NÖTR_00;
pub const İPUCU_KENARLIĞI: Renk = NÖTR_15;
pub const İPUCU_METNİ: Renk = NÖTR_80;
pub const İPUCU_GÖLGESİ: Renk = Renk::kyma(0.0, 0.0, 0.0, 0.2);

// Yazı boyutları (`tokens.size`).
pub const YAZI_KÜÇÜK: f32 = 12.0;
pub const YAZI_ORTA: f32 = 14.0;
pub const YAZI_BÜYÜK: f32 = 16.0;
pub const BAŞLIK_BOYUTU: f32 = 18.0;
pub const ALT_BAŞLIK_BOYUTU: f32 = 12.0;

/// Palet içinden sıra numarasıyla renk seçer (dolanarak).
pub fn palet_rengi(sıra: usize) -> Renk {
    PALET.get(sıra % PALET.len()).copied().unwrap_or(NÖTR_50)
}
