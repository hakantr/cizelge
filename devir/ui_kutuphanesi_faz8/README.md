# Kavis UI Kutuphanesi Faz 8 Devir Paketi

Bu dizin, Kavis `ui_kutuphanesi` icinde gecici olarak gelistirilmis grafik
yuzeylerinin Cizelge sahipligine devrini kaydeder. Devir 17 Temmuz 2026
tarihinde, Kavis `2862f3b` commit'i uzerinden hazirlanmistir.

Hedef kapsam `ECHARTS_TAM_UYUM_FAZI.md` icindeki ECharts 6.1 uyumudur. Kavis'te
bulunan cizgi, alan, cubuk, pasta, halka, dagilim, baloncuk, birlesik, isi
haritasi, agac haritasi, sankey, gosterge, mini cizgi, KPI ve pano yuzeylerinin
yerel sahipligi kaldirilmistir. Bundan sonra bu yeteneklerin motoru, model
sozlesmesi, etkileşimi ve galerisi Cizelge'de gelistirilecektir.

## Lisans ve kaynak siniri

Kavis kaynaklari MIT lisanslidir. Cizelge'nin degistirilemez kurali ise
algoritma ve secenek modeli portlarinin yalniz Apache-2.0 lisansli
ECharts/zrender kaynaklarini davranis ve tam uyum otoritesi kabul eder. MIT ve
Apache-2.0 izin verici lisanslardir ve ayni dagitimda birlikte bulunabilir;
ancak kopyalanan Kavis dosyalari kendiliginden Apache-2.0 olmaz. Bu dosyalarin
MIT telif ve izin bildirimi korunmak zorundadir.

Kavis grafik kaynaklarinin byte-esit goruntusu
`kaynak/kavis-2862f3b/` altina, kendi `LICENSE` dosyasiyla kopyalanmistir.
Arsiv Cargo workspace uyesi veya Cizelge runtime kaynagi degildir. Bir kaynak
parcasi urun kodunda yeniden kullanilacaksa MIT kokeni ve bildirimi korunur,
lisans/NOTICE envanterine girer ve davranis yine ECharts 6.1 kaynagiyla
dogrulanir. Yalniz Apache-2.0 olarak sahiplenilmek istenen uygulama ise Kavis
kodundan kopyalanmadan yeniden yazilmalidir.

## Paket icerigi

- `ENVANTER.json`: kaldirilan yuzeyler, eski dosya ozetleri ve Cizelge faz
  eslemeleri.
- `kaynak/kavis-2862f3b/`: devredilen MIT Rust kaynaklarinin ve lisansinin
  byte-esit arsivi.
- `KABUL_SOZLESMESI.md`: devirle kaybedilmemesi gereken guvenlik,
  erisilebilirlik, performans ve etkileşim gereksinimleri.
- `GECIS_ADIMLARI.md`: Cizelge uygulamasindan Kavis'teki ince adapter'a kadar
  izlenecek sirali gecis.

## Kapsam disi ve Kavis'te kalanlar

- `Harita`, karo kaynagi/onbellegi ve katmanlar Cizelge plani §2.3 geregi
  devredilmedi; `geo`/`map` kesin kapsam disidir.
- `FontPicker` grafik motoru degildir ve Kavis'te kalir.
- `diyagram_bilesenleri` genel amacli dugum/kenar duzenleme yuzeyidir;
  ECharts `graph`, `tree`, `treemap` veya `sankey` seri motoru sayilmaz.
- Typst rapor grafik veri modeli bir rapor cikti sozlesmesidir; canli GPUI
  grafik motoru olmadigi icin bu devrin parcasi degildir.

Bu devir paketi Cizelge uyumlulugu tamamlandi anlamina gelmez. Tamamlanma
karari yalniz `ECHARTS_TAM_UYUM_FAZI.md` Faz 9 kapilarindan sonra verilebilir.
