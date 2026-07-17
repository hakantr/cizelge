# Devir Kabul Sozlesmesi

Bu belge eski Kavis uygulamasinin API'sini dondurmaz. Cizelge'nin ECharts 6.1
uyum modeline ek olarak korunmasi gereken urun kalite sonuclarini tanimlar.
Davranis otoritesi `../echarts` ve onun zrender alt moduludur. Devredilen
Kavis MIT kaynaklari gecis/regresyon referansidir; tek basina ECharts uyumu
kaniti degildir.

## 1. Veri ve gizlilik

- Tum seri degerleri islenmeden once sonlu sayi, boyut, kimlik ve metin
  butcelerinden gecmelidir. Gecersiz veri panige veya sessiz veri kaybina
  donusmemelidir.
- Erisilebilir tablo, ipucu, hata ve tani ciktilari ham hassas deger tasimaz.
  Guvenli gosterim politikasi uygulama katmanindan enjekte edilebilmelidir.
- Tablo/CSV disa aktariminda `=`, `+`, `-` veya `@` ile baslayan hucreler
  elektronik tablo formulu olarak calisamaz. CSV uretimi akislidir; tum veri
  icin ikinci bir satir kopyasi zorunlu tutulamaz.
- Kararli veri/seri kimligi, ornekleme, filtreleme, zoom ve animasyon boyunca
  korunur.

## 2. Erisilebilirlik ve klavye

- Grafik kok yuzeyi odaklanabilir ve anlamli ad/ozet sunar.
- Seri ve veri ogeleri platform AccessKit agacinda rol, ad, secim/etkinlik
  durumu ve uygulanabilir eylemleriyle gorunur.
- Ok tuslari onceki/sonraki veri ogesine gider; Home/End sinirlara gider;
  Enter/Space secimi degistirir; Escape gecici secim, firca veya ipucunu
  kapatir.
- Zoom, lejant, secim ve veri degisimleri odagi kaybetmez. Gizlenen seri veya
  silinen veri ogesi icin odak deterministik bir komsuya ya da grafik kokune
  tasinir.
- MacOS ve Linux'ta gercek platform penceresi uzerinden en az bir AccessKit
  agac/eylem kaniti uretilir; yalniz kaynak tarama kabul edilmez.
- ARIA ozetleri veri buyudukce sinirli kalir; butun veri setini tek metne
  kopyalamaz.

Bu maddeler `ECHARTS_TAM_UYUM_FAZI.md` Faz 7 ve Faz 9 kapilarina ek kabul
kosuludur.

## 3. Etkilesim ve olaylar

- Isaretci hit-test'i gorunur/orneklenmis ogelerle kararli kimlikleri
  eslestirir; esit uzaklikta sonuc deterministiktir.
- Tiklama, klavye secimi, lejant, dataZoom, firca ve zaman seridi degisimi tek
  bir action/event hatti kullanir. Boyama sirasinda urun yan etkisi baslatmaz.
- Islem geri alma gereken option degisimlerinde ya tam yeni durum gorulur ya
  da onceki durum korunur; yarim guncelleme gorunmez.
- Entity/event abonelikleri yuzey yok edilirken serbest kalir.

## 4. Buyuk veri ve ornekleme

- `grafik_lttb_1m`: 1.000.000 sonlu noktadan 1.200 temsilci uretme senaryosu
  Cizelge Faz 8 performans paketine eklenir. Ilk/son nokta, belirgin tepe ve
  kararli kaynak kimlikleri korunur.
- Cizgi/alan icin sampling, scatter/heatmap icin large/progressive davranisi
  ECharts 6.1 semantigiyle uyumlu olmalidir. Kavis LTTB kodu yeniden
  kullanilirsa MIT lisans bildirimi korunur; ECharts semantik ve gorsel kanit
  kapilari yine zorunludur.
- Performans sonucu sure, tepe bellek ve sicak-dongu tahsis sayisini kaydeder;
  butce donanim profiliyle birlikte surumlenir.
- Buyuk veride erisilebilir tablo ve CSV tembel/akiskan kalir.

## 5. API ve entegrasyon karari

- Kavis'teki tur basina facade'lar (`CizgiGrafik`, `CubukGrafik` vb.) Cizelge
  API'sine tasinmaz. Cizelge'nin `GrafikSecenekleri` + seri tipleri tek kaynak
  olur.
- Cizelge hata/tani kanali panik uretmeden butce ve veri reddini bildirir.
- Kavis'e geri baglanti ancak kararlı, yeniden uretilebilir bir Cizelge surumu
  ve ince bir GPUI adapter'i uzerinden yapilir. Eski facade adlari uyumluluk
  alias'i olarak geri getirilmez.

## 6. Kabul matrisi

| Sonuc | Cizelge plani | Zorunlu kanit |
|---|---|---|
| Option ve veri dogrulama | Faz 2 | negatif testler + panik yasagi |
| Temel/ozel/hiyerarsik seriler | Faz 3-6 | API + gorsel + davranissal matris |
| Hit-test ve action/event hatti | Faz 1, 7 | pointer/klavye senaryolari |
| AccessKit K2 | Faz 7 | platform agaci ve eylem kaniti |
| Buyuk veri/ornekleme | Faz 8 | `grafik_lttb_1m` ve bellek/tahsis sonucu |
| Yasam dongusu | Faz 9 | tekrarli kur-yik soak sonucu |
| Kavis adapter'i | Faz 9 sonrasi | sabit surum + temiz tuketici derlemesi |
