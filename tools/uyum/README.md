# Uyum envanteri üreticisi

Bu araç, `ECHARTS_TAM_UYUM_FAZI.md` içindeki Faz 0 sözleşmesinin tek
üretim yoludur. Kilitli `../echarts` ve `../echarts-examples` çalışma
ağaçlarını okuyarak galeri manifestini, TypeScript option matrisini,
senaryo iskeletlerini ve salt-okunur kanıt raporunu üretir.

```sh
npm --prefix tools/uyum ci
npm --prefix tools/uyum run uret
npm --prefix tools/uyum run denetle
```

`denetle`, kaynak commitlerini ve üretilmiş dosyaların güncelliğini kontrol
eder; hiçbir dosyayı değiştirmez. Geo/Map sınıflandırması kategoriye ek
olarak örnek kaynağındaki `registerMap`, `geo`/`bmap` koordinatı ve `map`
serisi kullanımlarını tarar. GL kataloğunun tamamı sabit kapsam dışıdır.
