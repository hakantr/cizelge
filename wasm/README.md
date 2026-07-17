# cizelge-wasm

`cizelge` çekirdeğinin (gpui'siz, `default-features = false`) tarayıcı
köprüsü: seçeneklerden **SVG** üretir, `wasm-bindgen` ile JavaScript'e
açılır. Çizim hattı, masaüstündeki altın testlerle birebir aynıdır
(`grafiği_boya` → `SvgYüzeyi`).

## Derleme

```bash
rustup target add wasm32-unknown-unknown
cd wasm
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir pkg \
    target/wasm32-unknown-unknown/release/cizelge_wasm.wasm
# (eşdeğeri: wasm-pack build --target web)
```

> Not: `wasm-bindgen` CLI sürümü, `Cargo.toml`'daki `wasm-bindgen`
> bağımlılığıyla birebir aynı olmalıdır (şu an `=0.2.120`).

## Demo

```bash
cd wasm
python3 -m http.server 8080
# tarayıcı: http://localhost:8080/www/index.html
```

Sayfada değer ekleme/karıştırma, kategori düzenleme, tür seçimi ve koyu
tema anahtarı vardır; her girişte SVG yeniden üretilir ve altta
`erişilebilirlik_özeti` metni gösterilir.

## API

| İşlev | Açıklama |
|---|---|
| `svg_cizgi(başlık, kategoriler, değerler, koyu, g, y)` | Çizgi grafiği SVG'si |
| `svg_sutun(...)` | Sütun grafiği SVG'si |
| `svg_pasta(başlık, adlar, değerler, koyu, g, y)` | Halka pasta SVG'si |
| `ozet_cizgi(başlık, kategoriler, değerler)` | Erişilebilirlik (aria) özeti |

Dışa açılan adlar `#[no_mangle]` ABI sınırı ASCII gerektirdiği için
aksansızdır (dosya adı kuralıyla aynı); Rust içi adlar Türkçedir.

## Lisans

Apache-2.0 (kök depoyla aynı). `wasm-bindgen` MIT/Apache-2.0 ikili
lisanslıdır — onaylı listenin içindedir.
