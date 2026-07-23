# çizelge

Bu proje, [Apache ECharts](https://echarts.apache.org) 6.1.0'ın bildirime dayalı
`option` modelini, çizelge motorunu ve görsel davranışlarını Rust'a ve GPUI'ye
taşıyan, tümüyle Türkçe API'li bir porttur. Bağımsız olarak ortaya çıkarılmış
yeni bir grafik motoru değildir. Normatif kaynak,
[Apache ECharts deposundaki `74e9e09a` commit'idir](https://github.com/apache/echarts/commit/74e9e09a0b5687fdd34319121ac73b3022d1483c);
davranış, API, algoritma, varsayılan değer ve görsel uyum kararlarında Apache
ECharts esas alınır. Resmî galeri envanteri ve örnek kaynakları için normatif
yardımcı kaynak,
[`echarts-examples` deposundaki `1ff3451` commit'idir](https://github.com/apache/echarts-examples/commit/1ff3451941535c51af83eacd504035ef4bfd7d0d).
Doğrulama araçları bu kaynakların sırasıyla `../echarts` ve
`../echarts-examples` altındaki yerel klonlarını kullanır.

Web/Canvas yerine doğrudan GPU'da çizer. ECharts'ın "güzel" çentik algoritması,
sütun yerleşimi, yumuşak eğri kontrol noktaları ve yığınlama davranışı dahil
uyum kapsamındaki yetenekler ilgili ECharts kaynaklarından port edilir.

```rust
use cizelge::hazir::*;

let seçenekler = GrafikSeçenekleri::yeni()
    .başlık(Başlık::yeni().metin("Haftalık Sıcaklık"))
    .ipucu(İpucu::yeni().tetikleme(Tetikleme::Eksen))
    .x_ekseni(Eksen::kategori().veri(["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"]))
    .y_ekseni(Eksen::değer().etiket_biçimleyici("{value} °C"))
    .seri(ÇizgiSerisi::yeni().ad("En Yüksek").veri([11.0, 13.0, 15.0, 13.0, 12.0, 16.0, 21.0]).yumuşat(true));

// gpui penceresinde:
// cx.new(|_| GrafikGörünümü::yeni(seçenekler))
```

## Örnekler

```bash
cargo run --example galeri    # TÜM çizelgeler: ağaç menü + canlı veri düzenleme
cargo run --example cizgi     # yumuşak çizgi + alan + eksen ipucu
cargo run --example sutun     # gruplu + yığılmış sütun, gölge imleç
cargo run --example pasta     # halka pasta, dış etiketler, gösterge
cargo run --example sacilim   # kabarcık saçılımı, işlevsel sembol boyutu
cargo run --example pano      # tek pencerede dört grafik
```

> Depo, bağımsız `../gpui` çalışma alanındaki `gpui` ve `gpui_platform`
> crate'lerine yol bağımlılığıyla bağlıdır. Doğrulanan GPUI kaynağı
> `5566476024607a4c6999ab7b91d0218633a9b96c` commit'idir; GPUI çizelge
> davranışları için normatif kaynak değil, masaüstü çalışma zamanı ve çizim
> yüzeyidir.

## Modül eşlemesi (ECharts → çizelge)

| ECharts | çizelge | İçerik |
|---|---|---|
| `src/util/number.ts` | `yardimci::sayi` | `nice`, `linearMap`, `getPrecision`, `quantityExponent` portları |
| `src/util/format.ts` | `yardimci::bicim` | `addCommas`, değer biçimleme |
| `src/scale/Interval.ts` + `helper.ts` | `olcek::aralik` | "Güzel" çentik üretimi (`intervalScaleNiceTicks`) |
| `src/scale/Ordinal.ts` | `olcek::kategorik` | Kategori ölçeği |
| `src/scale/Log.ts` | `olcek::log` | Log uzayında tam sayı adımlı çentikler |
| `src/scale/Time.ts` | `olcek::zaman` | Birim seçimli, takvim hizalı zaman çentikleri |
| `src/coord/Axis.ts`, `cartesian/` | `koordinat` | Çalışma ekseni, bant yerleşimi, `Kartezyen2B` |
| `src/layout/barGrid.ts` | `yerlesim::sutun` | `calcBarWidthAndOffset` portu |
| `data/helper/dataStackHelper.ts` | `yerlesim::yigin` | İşarete göre ayrık yığın birikimi |
| `src/chart/line/poly.ts` | `grafik::cizgi` | Uç aşımı kısıtlı yumuşak Bezier (`drawSegment`) |
| `src/chart/{bar,pie,scatter}` | `grafik::{sutun,pasta,sacilim}` | Seri görünümleri |
| `src/component/{title,legend,axis,tooltip}` | `bilesen` | Başlık, gösterge, eksen çizimi, ipucu |
| `src/visual/tokens.ts` | `tema` | v6 renk paleti ve eksen belirteçleri |
| zrender `Painter` | `cizim::cizici` | gpui `Window` üstünde yol/metin/dilim çizimi |
| `echarts.init` + `setOption` | `cizim::gorunum::GrafikGörünümü` | `Render` görünümü, animasyon, fare etkileşimi |

## Uyum durumu ve kapsam

Destek durumu README içinde tarihsel bir özellik listesi olarak elle
tutulmaz. Normatif ve yeniden üretilebilir durum
[`uyum/galeri_manifest.json`](uyum/galeri_manifest.json),
[`uyum/ozellik_matrisi.json`](uyum/ozellik_matrisi.json),
[`uyum/ozet.json`](uyum/ozet.json), senaryo dosyaları ve görsel metriklerden
okunur. Bunların tümü şu komutla kilitli ECharts/ECharts Examples kaynaklarına
karşı yeniden üretilir:

```sh
node tools/uyum/uret.mjs
node tools/uyum/uret.mjs --check
```

Bir kartın yalnız çizilmesi tam destek sayılmaz; API, statik/animasyonlu
render, davranış ve dayanıklılık kapıları ayrı durum taşır. Kesin kapsam dışı
alanlar `geo`/`map`, bütün 3B görünümler ve `scatterGL`, `linesGL`, `flowGL`,
`graphGL` dahil tüm GL serileridir. Bunların dışındaki resmi iki boyutlu
ECharts kapsamı ve ayrıntılı kabul kapıları
[`ECHARTS_TAM_UYUM_FAZI.md`](ECHARTS_TAM_UYUM_FAZI.md) içinde tanımlıdır.

Ayrıntılı ECharts uyum planı için
**[ECHARTS_TAM_UYUM_FAZI.md](ECHARTS_TAM_UYUM_FAZI.md)** dosyasına bakın.

## Güvence kuralları

- **Panik yasağı:** çalışma zamanı kodunda `panic!`/`unwrap`/`expect`/
  doğrulanmamış `[]` vb. yasaktır ve clippy `deny` lintleriyle derlemede
  engellenir; hatalar `BilesenHatasi` olarak döner, boyama sırasındaki
  kurtarılabilir sorunlar `BilesenTanisi` olay kanalından yayımlanır,
  `seçenekleri_değiştir` doğrulama + işlem geri alma yapar.
- **Lisans sınırı:** proje hiçbir koşulda Apache-2.0 dışına çıkmaz
  (ayrıntılar ECHARTS_TAM_UYUM_FAZI.md'de).

## Atıf ve teşekkür

Grafik motorunun özgün tasarımı, `option`/API fikirleri, algoritmaları,
varsayılan davranışları, görsel dili ve resmî örnek senaryoları
[Apache ECharts projesine](https://github.com/apache/echarts) aittir. Bu
depodaki Rust kodu; söz konusu çalışmayı GPUI tabanlı bir çalışma zamanına
uyarlamak, eşdeğerliğini sınamak ve belgelemek amacıyla geliştirilir.

Apache Software Foundation'a; Apache ECharts PMC üyelerine, commit sahiplerine
ve katkıcılarına; ayrıca resmî galeri kaynaklarını geliştiren
`echarts-examples` katkıcılarına içtenlikle teşekkür ederiz. Çizelge'nin
ulaşabildiği işlevsellik ve doğruluk, onların açık kaynak olarak paylaştığı
çalışma sayesinde mümkündür.

## Lisans

Bu repo Apache-2.0 lisanslıdır (bkz. [LICENSE](LICENSE) ve [NOTICE](NOTICE)).
Normatif Apache ECharts kaynağı Apache-2.0, ECharts'ın kullandığı zrender ise
BSD-3-Clause lisanslıdır; kaynak telif ve lisans bildirimleri [NOTICE](NOTICE)
içinde korunur.

**Değiştirilemez kural:** Proje hiçbir koşulda Apache-2.0 lisans sınırının
dışına çıkmaz; GPL/LGPL/AGPL lisanslı kod veya bağımlılık kabul edilmez.
Ayrıntı: [ECHARTS_TAM_UYUM_FAZI.md](ECHARTS_TAM_UYUM_FAZI.md) içindeki lisans
kuralı bölümü.

## PNG / SVG dışa aktarım

`svg_dışa_aktar(&seçenekler, g, y)` SVG metni, `png_dışa_aktar(&seçenekler,
g, y, ölçek)` PNG baytları üretir (`png` özelliği; tiny-skia + ab_glyph +
fontdb). Araç kutusundaki `⤓ SVG` / `⤓ PNG` düğmeleri aynı hattı kullanır:
`AraçKutusu::yeni().svg_kaydet(true).png_kaydet(true)`.

## WASM

Çekirdek (model + ölçekler + boyama + SVG dışa aktarım) gpui olmadan da
derlenir: `cargo check --no-default-features` ya da
`--target wasm32-unknown-unknown`. Tarayıcı köprüsü ve canlı demo için
[`wasm/README.md`](wasm/README.md) dosyasına bakın.
