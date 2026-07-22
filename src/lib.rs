//! # çizelge
//!
//! [Apache ECharts](https://echarts.apache.org)'ın
//! [gpui](https://gpui.rs) üzerinde çalışan yerli Rust uyarlaması.
//!
//! Modül düzeni ECharts kaynak ağacını (`src/scale`, `src/coord`, `src/model`,
//! `src/chart`, `src/component`) birebir izler; çekirdek sayısal algoritmalar
//! ("güzel" çentik üretimi, sütun genişlik/kaydırma yerleşimi, yumuşak eğri
//! kontrol noktaları, yığınlama) ilgili ECharts gerçeklemelerinin doğrudan
//! portudur.
//!
//! ```ignore
//! use cizelge::hazir::*;
//!
//! let seçenekler = GrafikSeçenekleri::yeni()
//!     .başlık(Başlık::yeni().metin("Örnek"))
//!     .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
//!     .x_ekseni(Eksen::kategori().veri(["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"]))
//!     .y_ekseni(Eksen::değer())
//!     .seri(ÇizgiSerisi::yeni().veri([150, 230, 224, 218, 135, 147, 260]));
//! ```
#![allow(uncommon_codepoints)]
#![allow(mixed_script_confusables)]
#![allow(confusable_idents)]

pub mod animasyon;
pub mod bilesen;
pub mod calisma_zamani;
pub mod cizim;
pub mod eylem;
pub mod genisletme;
pub mod grafik;
pub mod hata;
pub mod koordinat;
pub mod model;
pub mod olcek;
pub mod renk;
pub mod tema;
pub mod yardimci;
pub mod yerel;
pub mod yerlesim;
pub mod zamanlayici;

pub use bilesen::erisilebilirlik::{erişilebilirlik_özeti, seri_tür_adı};
pub use bilesen::grafik::{GrafikSahnesi, GrafikÖğeBilgisi, grafik_sahnesi_hazırla};
pub use bilesen::zaman_seridi::ZamanŞeridiEylemi;
pub use calisma_zamani::{
    BileşikSeçenekler, EksenBoyutu, EksenKırılmaDeğişikliği, EksenKırılmaEylemi,
    GrafikÇalışmaZamanı, GöstergeSeçimEylemi, MedyaKuralı, MedyaSorgusu, SeriSeçici, SeçenekAlanı,
    SeçenekAyarlamaKipi, SeçenekYaması, ÇalışmaOlayı, ÇiziciTürü, ÖrnekBaşlatmaSeçenekleri,
};
pub use cizim::gorunum::{
    AraçTürü, BoyamaGirdisi, BoyamaÇıktısı, FırçaAlanı, SürgüBölgesi, SürgüParçası, grafiği_boya,
    İçYakınlaştırmaAlanı,
};
#[cfg(feature = "gpui")]
pub use cizim::pencere::GrafikGörünümü;
#[cfg(feature = "png")]
pub use cizim::piksel::{PikselYüzeyi, png_dışa_aktar};
pub use cizim::{
    AfinMatris, GrafikOlayı, GörselDurum, KayıtYüzeyi, KırpmaYolu, OdakKapsamı, Sahne, SahneDüğümü,
    SahneFarkı, SahneMetni, SahneResmi, SahneStilYaması, SahneStili, SahneÖğesi, Sahneİsabeti,
    SahneŞekli, SihirliSeriTürü, SvgYolHatası, SvgYüzeyi, YerelDönüşüm, Yol, svg_dışa_aktar,
    yolu_dönüştür, ÇizimYüzeyi, İsabetBölgesi, İsabetGeometrisi,
};
pub use eylem::{
    BağlıGrafikler, EylemDeğeri, EylemGüncellemesi, EylemKayıtDefteri, EylemYükü, OlayKayıtDefteri,
    OlaySorgusu, OlayYükü, append_data_eylemini_kaydet, eksen_imleci_eylemini_kaydet,
    eksen_kırılma_eylemlerini_kaydet, fırça_eylemini_kaydet, geri_yükleme_eylemini_kaydet,
    görsel_aralık_eylemini_kaydet, gösterge_eylemlerini_kaydet, veri_yakınlaştırma_eylemini_kaydet,
    öntanımlı_eylemleri_kaydet,
};
pub use genisletme::{
    Genişletme, GenişletmeBağlamı, GenişletmeKayıtDefteri, KoordinatSistemiKayıtDefteri,
    KoordinatSistemiÜreticisi, SeçenekÖnİşleyicisi, YüklemeGirdisi, YüklemeKayıtDefteri,
    YüklemeÇizicisi, ÖnİşlemeKayıtDefteri, ÖzelKoordinatSistemi,
};
pub use hata::{BilesenHatasi, BilesenTanisi};
pub use model::agac::AğaçDüğümü;
pub use model::bilesen::{
    AraçKutusu, AraçKutusuÖzelliği, Başlık, BaşlıkMetinHizası, Fırça, FırçaAracıTürü, FırçaBağı,
    FırçaKoordinatAralığı, FırçaKoordinatı, FırçaSeçimAlanı, FırçaStili, FırçaTürü, Gösterge,
    GöstergeSeçimKipi, Izgara, Tetikleme, Yön, İmleçTürü, İpucu, İpucuBiçimleyicisi, İpucuKonumu,
    İpucuParametresi,
};
pub use model::deger::{VeriDeğeri, VeriÖğesi};
pub use model::eksen::{
    AraÇentik, BölmeAlanı, BölmeÇizgisi, Eksen, EksenAdKonumu, EksenEtiketBağlamı,
    EksenEtiketBiçimleyicisi, EksenEtiketi, EksenKonumu, EksenKırılmaAlanı, EksenKırılmaBilgisi,
    EksenKırılmaBoşluğu, EksenKırılmaUcu, EksenKırılması, EksenSıfırKipi, EksenTürü, EksenÇentiği,
    EksenÇizgisi, SayısalKenarBoşluğu,
};
pub use model::gorsel_esleme::{EşlemeParçası, GörselEşleme};
pub use model::grafik_bileseni::{
    GrafikBağlıMetni, GrafikBileşeni, GrafikMetinKonumu, GrafikYerleşimi, GrafikÖğesi,
    GrafikÖğeİçeriği,
};
pub use model::hatlar::{
    HatEfekti, HatKoordinatSistemi, HatKoordinatı, HatNoktası, HatVerisi, HatlarSerisi,
};
pub use model::imleyici::{
    İmAlanı, İmAlanıDeğeri, İmAlanıTanımı, İmDeğeri, İmNoktası, İmNoktasıTanımı, İmYönü,
    İmleyiciler, İmÇizgisi, İmÇizgisiEtiketKonumu, İmÇizgisiEtiketYaması, İmÇizgisiParçası,
    İmÇizgisiTanımı, İmÇizgisiUcu, İmÇizgisiUçSimgesi,
};
pub use model::kutupsal::KutupsalKoordinat;
pub use model::matris::{
    MatrisAralığı, MatrisBoyutHücresi, MatrisBoyutu, MatrisGövdeHücresi, MatrisKonumu,
    MatrisKoordinatı,
};
pub use model::radar::{
    RadarBölmeAlanı, RadarEksenAdı, RadarGöstergesi, RadarKoordinatı, RadarÇizgileri, RadarŞekli,
};
pub use model::secenekler::GrafikSeçenekleri;
pub use model::seri::{
    AğaçHaritasıSerisi, AğaçSerisi, Basamak, DüzSaçılımVerisi, EtiketYerleşimParametreleri,
    EtiketYerleşimSonucu, EtiketÇizgisi, EtiketÖrtüşmeKaydırması, GrafoDüğümü, GrafoSerisi,
    GrafoYerleşimi, GöstergeMetinYaması, GöstergeSaatiSerisi, GöstergeVeriÖğesi,
    GöstergeİbreYaması, GöstergeİlerlemeYaması, GülTürü, GüneşPatlamasıSerisi, HuniDurumYaması,
    HuniEtiketÇizgisiYaması, HuniHizası, HuniSerisi, HuniSıralaması, HuniVeriYaması, HuniYönü,
    IsıHaritasıSerisi, KirişSerisi, KutuSerisi, MumSerisi, ParalelBoyut, ParalelSerisi,
    PastaSerisi, Piktogram, RadarDurumYaması, RadarSerisi, RadarVeriYaması, SankeyBağı,
    SankeySerisi, SaçılımSerisi, Sembol, SembolBoyutu, Seri, SütunSerisi, TakvimSerisi,
    TemaNehriSerisi, VeriİşlevBağlamı, ÇizgiSerisi, Örnekleme, ÖzelBağlam, ÖzelSeri, ÖzelÇizim,
    ÖğeRengiİşlevi,
};
pub use model::stil::{
    AlanStili, Biçimleyici, DışEtiketHizası, Etiket, EtiketDöndürme, EtiketKonumu, EtiketYaması,
    YazıDikeyHizası, YazıStili, YazıYatayHizası, ÇizgiStili, ÇizgiTürü, ÖğeStili,
};
pub use model::takvim::{
    TakvimAralığı, TakvimEtiketTarafı, TakvimKoordinatı, TakvimYönü, TakvimYılEtiketiKonumu,
};
pub use model::tek_eksen::{TekEksen, TekEksenKonumu, TekEksenYönü};
pub use model::veri_kumesi::{
    BoyutSeçici, BoyutTanımı, BoyutTürü, DönüşümKayıtDefteri, HistogramDönüşümü,
    HistogramEşikYöntemi, Karşılaştırmaİşlemi, KaynakBaşlığı, KaynakSeçenekleri, Kodlama,
    KutuDönüşümü, KutuSınırı, KümelemeDönüşümü, RegresyonDönüşümü, RegresyonFormülKonumu,
    RegresyonYöntemi, SeriYerleşimi, SüzmeDönüşümü, SüzmeKoşulu, SıralamaAnahtarı,
    SıralamaDönüşümü, SıralamaDüzeni, ToplamaBoyutu, ToplamaDönüşümü, ToplamaYöntemi,
    TürlüSayıDizisi, VeriDeposu, VeriDönüşümü, VeriKaynağı, VeriKümesi, VeriKümesiTanımı,
    VeriKümesiZinciri, veri_kümelerini_çöz,
};
pub use model::yakinlastirma::{
    VeriYakınlaştırma, YakınlaştırmaDeğeri, YakınlaştırmaSüzmeKipi, YakınlaştırmaTürü,
};
pub use model::zaman_seridi::{
    ZamanŞeridi, ZamanŞeridiEksenTürü, ZamanŞeridiEtiketKonumu, ZamanŞeridiEtiketi,
    ZamanŞeridiKontrolKonumu, ZamanŞeridiKontrolNoktasıStili, ZamanŞeridiKontrolStili,
    ZamanŞeridiSimgesi,
};
pub use model::{DikeyKonum, Uzunluk, YatayKonum};
pub use renk::{DesenTekrarı, Dolgu, GörüntüDeseni, Renk, RenkDurağı};
pub use yerel::{TÜRKÇE, Yerel, İNGİLİZCE};
pub use zamanlayici::{
    AdımSonucu, ArtımlıGörev, GörevAşaması, GörevBağlamı, Görevİlerlemesi, Zamanlayıcı,
    ZamanlayıcıDurumu,
};

/// Sık kullanılan tiplerin topluca içe aktarımı (ECharts'taki `echarts` ana
/// girişinin karşılığı).
pub mod hazir {
    pub use crate::bilesen::erisilebilirlik::{erişilebilirlik_özeti, seri_tür_adı};
    pub use crate::bilesen::grafik::{GrafikSahnesi, GrafikÖğeBilgisi, grafik_sahnesi_hazırla};
    pub use crate::bilesen::zaman_seridi::ZamanŞeridiEylemi;
    pub use crate::calisma_zamani::{
        BileşikSeçenekler, EksenBoyutu, EksenKırılmaDeğişikliği, EksenKırılmaEylemi,
        GrafikÇalışmaZamanı, GöstergeSeçimEylemi, MedyaKuralı, MedyaSorgusu, SeriSeçici,
        SeçenekAlanı, SeçenekAyarlamaKipi, SeçenekYaması, ÇalışmaOlayı, ÇiziciTürü,
        ÖrnekBaşlatmaSeçenekleri,
    };
    pub use crate::cizim::gorunum::{
        BoyamaGirdisi, BoyamaÇıktısı, FırçaAlanı, grafiği_boya
    };
    #[cfg(feature = "gpui")]
    pub use crate::cizim::pencere::GrafikGörünümü;
    #[cfg(feature = "png")]
    pub use crate::cizim::piksel::{PikselYüzeyi, png_dışa_aktar};
    pub use crate::cizim::{
        AfinMatris, GrafikOlayı, GörselDurum, KayıtYüzeyi, KırpmaYolu, OdakKapsamı, Sahne,
        SahneDüğümü, SahneFarkı, SahneMetni, SahneResmi, SahneStilYaması, SahneStili, SahneÖğesi,
        Sahneİsabeti, SahneŞekli, SvgYolHatası, SvgYüzeyi, YerelDönüşüm, Yol, svg_dışa_aktar,
        yolu_dönüştür, ÇizimYüzeyi,
    };
    pub use crate::eylem::{
        BağlıGrafikler, EylemDeğeri, EylemGüncellemesi, EylemKayıtDefteri, EylemYükü,
        OlayKayıtDefteri, OlaySorgusu, OlayYükü, append_data_eylemini_kaydet,
        eksen_imleci_eylemini_kaydet, eksen_kırılma_eylemlerini_kaydet, fırça_eylemini_kaydet,
        geri_yükleme_eylemini_kaydet, görsel_aralık_eylemini_kaydet, gösterge_eylemlerini_kaydet,
        veri_yakınlaştırma_eylemini_kaydet, öntanımlı_eylemleri_kaydet,
    };
    pub use crate::genisletme::{
        Genişletme, GenişletmeBağlamı, GenişletmeKayıtDefteri, KoordinatSistemiKayıtDefteri,
        KoordinatSistemiÜreticisi, SeçenekÖnİşleyicisi, YüklemeGirdisi, YüklemeKayıtDefteri,
        YüklemeÇizicisi, ÖnİşlemeKayıtDefteri, ÖzelKoordinatSistemi,
    };
    pub use crate::hata::{BilesenHatasi, BilesenTanisi};
    pub use crate::model::agac::AğaçDüğümü;
    pub use crate::model::bilesen::{
        AraçKutusu, AraçKutusuÖzelliği, Başlık, BaşlıkMetinHizası, Fırça, FırçaAracıTürü,
        FırçaBağı, FırçaKoordinatAralığı, FırçaKoordinatı, FırçaSeçimAlanı, FırçaStili, FırçaTürü,
        Gösterge, GöstergeSeçimKipi, Izgara, Tetikleme, Yön, İmleçTürü, İpucu, İpucuBiçimleyicisi,
        İpucuKonumu, İpucuParametresi,
    };
    pub use crate::model::deger::{VeriDeğeri, VeriÖğesi};
    pub use crate::model::eksen::{
        AraÇentik, BölmeAlanı, BölmeÇizgisi, Eksen, EksenAdKonumu, EksenEtiketBağlamı,
        EksenEtiketBiçimleyicisi, EksenEtiketi, EksenKonumu, EksenKırılmaAlanı,
        EksenKırılmaBilgisi, EksenKırılmaBoşluğu, EksenKırılmaUcu, EksenKırılması, EksenSıfırKipi,
        EksenTürü, EksenÇentiği, EksenÇizgisi, SayısalKenarBoşluğu,
    };
    pub use crate::model::gorsel_esleme::{EşlemeParçası, GörselEşleme};
    pub use crate::model::grafik_bileseni::{
        GrafikBağlıMetni, GrafikBileşeni, GrafikMetinKonumu, GrafikYerleşimi, GrafikÖğesi,
        GrafikÖğeİçeriği,
    };
    pub use crate::model::hatlar::{
        HatEfekti, HatKoordinatSistemi, HatKoordinatı, HatNoktası, HatVerisi, HatlarSerisi,
    };
    pub use crate::model::imleyici::{
        İmAlanı, İmAlanıDeğeri, İmAlanıTanımı, İmDeğeri, İmNoktası, İmNoktasıTanımı, İmYönü,
        İmleyiciler, İmÇizgisi, İmÇizgisiEtiketKonumu, İmÇizgisiEtiketYaması, İmÇizgisiParçası,
        İmÇizgisiTanımı, İmÇizgisiUcu, İmÇizgisiUçSimgesi,
    };
    pub use crate::model::kutupsal::KutupsalKoordinat;
    pub use crate::model::matris::{
        MatrisAralığı, MatrisBoyutHücresi, MatrisBoyutu, MatrisGövdeHücresi, MatrisKonumu,
        MatrisKoordinatı,
    };
    pub use crate::model::radar::{
        RadarBölmeAlanı, RadarEksenAdı, RadarGöstergesi, RadarKoordinatı, RadarÇizgileri,
        RadarŞekli,
    };
    pub use crate::model::secenekler::GrafikSeçenekleri;
    pub use crate::model::seri::{
        AğaçHaritasıSerisi, AğaçSerisi, Basamak, EtiketYerleşimParametreleri, EtiketYerleşimSonucu,
        EtiketÇizgisi, EtiketÖrtüşmeKaydırması, GrafoDüğümü, GrafoSerisi, GrafoYerleşimi,
        GöstergeMetinYaması, GöstergeSaatiSerisi, GöstergeVeriÖğesi, GöstergeİbreYaması,
        GöstergeİlerlemeYaması, GülTürü, GüneşPatlamasıSerisi, HuniDurumYaması,
        HuniEtiketÇizgisiYaması, HuniHizası, HuniSerisi, HuniSıralaması, HuniVeriYaması, HuniYönü,
        IsıHaritasıSerisi, KirişSerisi, KutuSerisi, MumSerisi, ParalelBoyut, ParalelSerisi,
        PastaSerisi, Piktogram, RadarDurumYaması, RadarSerisi, RadarVeriYaması, SankeyBağı,
        SankeySerisi, SaçılımSerisi, Sembol, SembolBoyutu, Seri, SütunSerisi, TakvimSerisi,
        TemaNehriSerisi, VeriİşlevBağlamı, ÇizgiSerisi, Örnekleme, ÖzelBağlam, ÖzelSeri, ÖzelÇizim,
        ÖğeRengiİşlevi,
    };
    pub use crate::model::stil::{
        AlanStili, Biçimleyici, DışEtiketHizası, Etiket, EtiketDöndürme, EtiketKonumu,
        EtiketYaması, YazıDikeyHizası, YazıStili, YazıYatayHizası, ÇizgiStili, ÇizgiTürü, ÖğeStili,
    };
    pub use crate::model::takvim::{
        TakvimAralığı, TakvimEtiketTarafı, TakvimKoordinatı, TakvimYönü, TakvimYılEtiketiKonumu,
    };
    pub use crate::model::tek_eksen::{TekEksen, TekEksenKonumu, TekEksenYönü};
    pub use crate::model::veri_kumesi::{
        BoyutSeçici, BoyutTanımı, BoyutTürü, DönüşümKayıtDefteri, HistogramDönüşümü,
        HistogramEşikYöntemi, Karşılaştırmaİşlemi, KaynakBaşlığı, KaynakSeçenekleri, Kodlama,
        KutuDönüşümü, KutuSınırı, KümelemeDönüşümü, RegresyonDönüşümü, RegresyonFormülKonumu,
        RegresyonYöntemi, SeriYerleşimi, SüzmeDönüşümü, SüzmeKoşulu, SıralamaAnahtarı,
        SıralamaDönüşümü, SıralamaDüzeni, ToplamaBoyutu, ToplamaDönüşümü, ToplamaYöntemi,
        TürlüSayıDizisi, VeriDeposu, VeriDönüşümü, VeriKaynağı, VeriKümesi, VeriKümesiTanımı,
        VeriKümesiZinciri, veri_kümelerini_çöz,
    };
    pub use crate::model::yakinlastirma::{
        VeriYakınlaştırma, YakınlaştırmaDeğeri, YakınlaştırmaSüzmeKipi, YakınlaştırmaTürü,
    };
    pub use crate::model::zaman_seridi::{
        ZamanŞeridi, ZamanŞeridiEksenTürü, ZamanŞeridiEtiketKonumu, ZamanŞeridiEtiketi,
        ZamanŞeridiKontrolKonumu, ZamanŞeridiKontrolNoktasıStili, ZamanŞeridiKontrolStili,
        ZamanŞeridiSimgesi,
    };
    pub use crate::model::{DikeyKonum, Uzunluk, YatayKonum};
    pub use crate::renk::{DesenTekrarı, Dolgu, GörüntüDeseni, Renk, RenkDurağı};
    pub use crate::yerel::{TÜRKÇE, Yerel, İNGİLİZCE};
    pub use crate::zamanlayici::{
        AdımSonucu, ArtımlıGörev, GörevAşaması, GörevBağlamı, Görevİlerlemesi, Zamanlayıcı,
        ZamanlayıcıDurumu,
    };
}
