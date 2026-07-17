//! Faz 8 grafik adapterının gerçek GPUI/AccessKit protokol kanıtı.
//!
//! Bu test native ekran okuyucu değildir. GPUI'nin platform activation,
//! `Window::draw`, kısmi AccessKit ağacı ve `ActionRequest` geri dönüş yolunu
//! aynı oturumda çalıştırır.

use std::sync::Arc;

use gpui::{AppContext, Role, WindowOptions, accesskit};
use grafik_bilesenleri::{CizgiGrafik, GrafikNoktasi, Nokta};
use ortak_bilesenler::{OrtakBilesenAyarlari, baslat};
use ortak_bilesenler_cekirdek::{
    GuvenliDegerTemsili, HassasMetin, SunumYapilandirmasi, VeriHassasiyeti,
};
use ortak_tema::VarsayilanTema;
use ui_test_destegi::ErisilebilirTestUygulamasi;

fn hassas_etiket(metin: &str) -> GuvenliDegerTemsili {
    GuvenliDegerTemsili::olustur(
        HassasMetin::yeni(metin),
        SunumYapilandirmasi::yeni()
            .veri_hassasiyeti(VeriHassasiyeti::Kisisel)
            .derle()
            .expect("grafik erişilebilirlik profili geçerli olmalı"),
    )
}

#[test]
fn grafik_rol_deger_focus_ve_actionlari_ayni_durumu_gunceller() {
    let uygulama = ErisilebilirTestUygulamasi::yeni();
    let pencere = uygulama.guncelle(|cx| {
        baslat(
            OrtakBilesenAyarlari {
                tema_saglayici: Some(Arc::new(VarsayilanTema::koyu())),
                ..OrtakBilesenAyarlari::default()
            },
            cx,
        )
        .expect("kanıt teması geçerli olmalı");

        cx.open_window(WindowOptions::default(), |_window, cx| {
            cx.new(|cx| {
                CizgiGrafik::yeni(
                    "Gelir grafiği",
                    vec![
                        GrafikNoktasi::yeni(
                            "bir",
                            Nokta::yeni(1.0, 10.0),
                            hassas_etiket("Ayşe Yılmaz"),
                        ),
                        GrafikNoktasi::yeni(
                            "iki",
                            Nokta::yeni(2.0, 20.0),
                            hassas_etiket("Fatma Kaya"),
                        ),
                    ],
                    cx,
                )
                .expect("grafik fixture'ı geçerli olmalı")
            })
        })
        .expect("grafik kanıt penceresi açılmalı")
    });
    let pencere_tutamaci = pencere.into();

    uygulama.erisilebilirligi_etkinlestir();
    uygulama.pencereyi_ciz(pencere_tutamaci);
    let ilk = uygulama.agac();
    let (grafik_kimligi, grafik) = ilk
        .bul(Role::Group, "Gelir grafiği")
        .expect("grafik gerçek AccessKit ağacında bulunmalı");
    let ilk_deger = grafik.value().expect("grafik erişilebilir değer taşımalı");
    assert!(ilk_deger.contains("Ay** Yı****"));
    assert!(!ilk_deger.contains("Ayşe Yılmaz"));
    assert!(grafik.supports_action(accesskit::Action::Focus));
    assert!(grafik.supports_action(accesskit::Action::Increment));
    assert!(grafik.supports_action(accesskit::Action::Decrement));
    assert!(grafik.supports_action(accesskit::Action::Click));

    uygulama.erisilebilirlik_eylemi(grafik_kimligi, accesskit::Action::Focus);
    uygulama.pencereyi_ciz(pencere_tutamaci);
    assert_eq!(uygulama.agac().odak(), Some(grafik_kimligi));

    uygulama.erisilebilirlik_eylemi(grafik_kimligi, accesskit::Action::Increment);
    uygulama.pencereyi_ciz(pencere_tutamaci);
    let sonraki = uygulama.agac();
    let (_, grafik) = sonraki
        .bul(Role::Group, "Gelir grafiği")
        .expect("grafik node kimliği frame'ler arasında yaşamalı");
    let deger = grafik.value().expect("güncel değer taşınmalı");
    assert!(deger.contains("Fa*** Ka**"));
    assert!(!deger.contains("Fatma Kaya"));

    uygulama.erisilebilirlik_eylemi(grafik_kimligi, accesskit::Action::Click);
    uygulama.pencereyi_ciz(pencere_tutamaci);
    let secili = uygulama.agac();
    let (_, grafik) = secili
        .bul(Role::Group, "Gelir grafiği")
        .expect("grafik seçimden sonra yaşamalı");
    assert!(
        grafik
            .value()
            .expect("seçim özeti olmalı")
            .contains("1 seçili")
    );
}
