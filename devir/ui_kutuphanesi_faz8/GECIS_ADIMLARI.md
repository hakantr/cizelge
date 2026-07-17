# Kavis'ten Cizelge'ye Gecis Adimlari

Adimlar siralidir. Bir adimin cikis kapisi gecilmeden sonraki adim urun
entegrasyonu sayilmaz.

## 1. Devir kaydini kilitle

1. `ENVANTER.json` semasini CI'da JSON olarak dogrula.
2. Kaynak commit ve SHA-256 degerlerini denetim izi olarak koru.
3. `kaynak/kavis-2862f3b/` arsivini byte-esit ve MIT `LICENSE` bildirimiyle
   koru; Cargo workspace'e veya urun paketine kendiliginden dahil etme.
4. Arsivden urun koduna alinacak her parca icin MIT kokenini, degisiklikleri
   ve `NOTICE` kaydini koru. Yalniz Apache-2.0 olarak yazilacak kodu arsivden
   kopyalama.
5. Uygulama kararlarinda `ECHARTS_TAM_UYUM_FAZI.md`, `FAZ_PLANI.md`,
   `../echarts` ve zrender kaynaklarini esas al.

Cikis: sekiz kaynak ve lisans byte-esit; arsiv derleme disi; lisans ve NOTICE
kaydi eksiksiz.

## 2. Gereksinimleri Cizelge envanterine bagla

1. `KABUL_SOZLESMESI.md` maddelerine kararlı gereksinim kimlikleri ver.
2. Her kimligi ECharts uyum envanterindeki API, gorsel, davranis ve dayaniklilik
   satirlarina bagla.
3. Mevcut Cizelge yetenegini `var`, `eksik`, `farkli` diye olc; Kavis facade
   adlarini hedef API olarak kullanma.

Cikis: sahipsiz devir maddesi ve kanitsiz "tamam" satiri yok.

## 3. Veri ve guvenli cikti portlarini kur

1. Uygulama tarafinin hassas veri politikasini enjekte edecegi dar bir
   `GuvenliDegerSunumu` benzeri port tanimla; Kavis crate'ine bagimlilik ekleme.
2. Tembel erisilebilir tablo satiri ve akisli CSV cikti sozlesmelerini ekle.
3. Formula enjeksiyonu, sonlu olmayan sayi, yinelenen kimlik, azami satir,
   azami metin ve azami bayt negatif testlerini yaz.

Cikis: veri ve gizlilik testleri paniksiz geciyor.

## 4. Action, odak ve AccessKit katmanini tamamla

1. Pointer ve klavye girdisini ECharts action/event hattina bagla.
2. Grafik kokunu ve veri ogelerini AccessKit agacina ekle; odak aktarim
   kurallarini uygula.
3. `grafik_accesskit_k2` platform testini yeni Cizelge API'siyle sifirdan yaz.
4. Lejant, dataZoom, firca ve timeline sonrasinda odak/kimlik korunmasini test
   et.

Cikis: Faz 7 platform ve davranis kapilari geciyor.

## 5. Buyuk veri ve yasam dongusu kanitlarini tasarla

1. `grafik_lttb_1m` profilini Cizelge benchmark aracina ekle. Kavis MIT
   uygulamasi yeniden kullanilirsa kokenini koru; hangi uygulama secilirse
   secilsin sampling semantigini ECharts 6.1 kaynagiyla dogrula.
2. Sicak dongu tahsisi, tepe bellek, sure, ilk/son/tepe ve kararli kimlik
   kontrollerini raporla.
3. Tum kapsam ici seri ailelerini entity/abonelik kur-yik soak senaryosuna ekle.
4. Baslangic ve bitis kaynak sayaclarinin esitligini zorunlu kil.

Cikis: Faz 8 ve Faz 9 performans/dayaniklilik kapilari geciyor.

## 6. Galeri ve uyum kapanisini yap

1. Devir kabul maddelerini 261 orneklik galeri/uyum matrisine bagla.
2. API, gorsel, davranis ve dayaniklilik eksenlerinin dordunu de kapat.
3. Geo/map ve tum GL/3B yeteneklerinin kapsam disi kaldigini otomatik denetle.
4. Yeniden uretilebilir surum etiketi, SBOM, lisans raporu ve degisiklik
   notunu uret.

Cikis: `ECHARTS_TAM_UYUM_FAZI.md` Faz 9 surum kapisi geciyor.

## 7. Kavis'e ince adapter ile baglan

Bu adim Cizelge Faz 9 tamamlanmadan uygulanmaz.

1. Kavis ve Cizelge'nin gpui/Zed revizyonlarini uyumlu kil.
2. Yol bagimliligi yerine temiz tuketicide yeniden uretilebilen sabit bir git
   revizyonu veya yayinlanmis surum kullan.
3. Kavis uygulama katmaninda guvenli deger portunu Cizelge'ye bagla; UI crate'i
   ag, dosya sistemi veya veritabani yan etkisi baslatmasin.
4. Yalniz gereken `GrafikGorunumu` adapter'ini ekle. Kaldirilan 15 facade'i
   geri getirme.
5. Kavis temiz-tuketici, WASM, AccessKit, SBOM/lisans ve tam `dogrula.sh`
   kapilarini calistir.

Cikis: Cizelge tek grafik motoru, Kavis yalniz urune ozgu ince adapter sahibi.

## Geri donus

Entegrasyon kapilarindan biri gecmezse Kavis, yerel grafik motorunu geri
getirmez. Grafik yuzeyi kapali kalir; sorun Cizelge'de duzeltilip yeni sabit
surumle tekrar denenir. Harita/karo ve FontPicker bu surecten etkilenmez.
