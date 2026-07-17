# çizelge

[Apache ECharts](https://echarts.apache.org)'ın, [Zed](https://zed.dev) editörünün
arayüz çatısı **gpui** üzerinde çalışan, tümüyle Türkçe API'li yerli Rust uyarlaması.

Web/Canvas yerine doğrudan GPU'da çizer: ECharts'ın bildirime dayalı `option`
modeli, "güzel" çentik algoritması, sütun yerleşimi, yumuşak eğri kontrol
noktaları ve yığınlama davranışı ilgili ECharts kaynaklarının birebir portudur.

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

> Depo, `../zed` içindeki gpui'ye yol bağımlılığıyla bağlıdır.

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

## Bugünkü kapsam

- **Seriler:** çizgi (yumuşak/basamaklı, alan, yığın, boş değer), sütun
  (gruplu/yığılmış/yatay, köşe yarıçapı, değer etiketi), pasta (halka, gül,
  dış/iç/merkez etiket, etiket çizgisi), saçılım (işlevsel sembol boyutu).
- **Eksenler:** değer, kategori, zaman, log; `min/max`, `splitNumber`,
  `boundaryGap`, ters çevirme, biçimleyiciler, otomatik etiket seyreltme,
  `containLabel`.
- **Bileşenler:** başlık, tıklanabilir gösterge (seri/dilim açma-kapama),
  ızgara, eksen imleçli (çizgi/gölge) ipucu penceresi.
- **Görsellik:** ECharts v6 tema paleti, gradyan dolgular, kesikli/noktalı
  çizgiler, `cubicOut` giriş animasyonları.

## Yol haritası (tam eşdeğerlik için)

Radar, ısı haritası, mum (candlestick), ağaç/halka (treemap/sunburst), grafo,
sankey, gösterge saati (gauge); `dataZoom`, `visualMap`, fırça, araç kutusu;
çoklu ızgara/eksen, kutupsal koordinat; `dataset`/dönüşümler.

**Kapsam dışı (kesin):** coğrafi katman (`geo`/`map`) ayrı bir çalışmadır;
3B görünümler ve GL serileri (`scatterGL`, `linesGL`, `flowGL`, `graphGL`)
bu projeye dahil değildir.

Ayrıntılı, fazlara bölünmüş plan için: **[FAZ_PLANI.md](FAZ_PLANI.md)**.

## Güvence kuralları

- **Panik yasağı:** çalışma zamanı kodunda `panic!`/`unwrap`/`expect`/
  doğrulanmamış `[]` vb. yasaktır ve clippy `deny` lintleriyle derlemede
  engellenir; hatalar `BilesenHatasi` olarak döner, boyama sırasındaki
  kurtarılabilir sorunlar `BilesenTanisi` olay kanalından yayımlanır,
  `seçenekleri_değiştir` doğrulama + işlem geri alma yapar.
- **Lisans sınırı:** proje hiçbir koşulda Apache-2.0 dışına çıkmaz
  (ayrıntılar FAZ_PLANI.md'de).

## Lisans ve atıf

Apache-2.0 (bkz. [LICENSE](LICENSE) ve [NOTICE](NOTICE)). Algoritmalar ve
seçenek modeli, Apache Software Foundation'ın Apache-2.0 lisanslı
[Apache ECharts](https://github.com/apache/echarts) projesinden
uyarlanmıştır.

**Değiştirilemez kural:** Proje hiçbir koşulda Apache-2.0 lisans sınırının
dışına çıkmaz; GPL/LGPL/AGPL lisanslı kod veya bağımlılık kabul edilmez.
Ayrıntı: [FAZ_PLANI.md](FAZ_PLANI.md) içindeki lisans kuralı bölümü.

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
