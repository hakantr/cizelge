# Cizelge — ECharts 6.1 Tam Uyum ve Görsel Galeri Faz Planı

Son güncelleme: 2026-07-17

Bu belge, `cizelge` için **normatif tam uyum planıdır**. [FAZ_PLANI.md](FAZ_PLANI.md)
mevcut uygulama tarihini ve daha önce tamamlanan özellik dilimlerini kaydeder;
ancak bir işin orada `✅` olması, bu belgedeki ECharts uyum kapıları geçilmeden
"tam eşdeğer" sayıldığı anlamına gelmez.

Hedef, aşağıda kesin olarak kapsam dışı bırakılan Geo/Map, 3B ve GL alanları
hariç olmak üzere Apache ECharts'ın iki boyutlu çizelgelerini, koordinat
sistemlerini, bileşenlerini, veri işleme hattını, etkileşimlerini,
animasyonlarını ve çıktı yeteneklerini Türkçe Rust API'siyle bu repoya
aktarmaktır. Ayrıca resmi ECharts örnek sayfası tarzında bir galeri kurulacak
ve kapsamdaki her resmi örneğin desteği görsel ve davranışsal kanıtla
doğrulanacaktır.

## 1. Sabit referanslar ve kaynak önceliği

Uyum çalışması "son sürüm" gibi hareketli bir hedefe göre değil, commit'i
sabitlenmiş bir kaynak görüntüsüne göre yürütülür.

| Kaynak | Bu planın tabanı | Yerel yol | Kullanım amacı |
|---|---|---|---|
| Apache ECharts | v6.1.0, `74e9e09a0b5687fdd34319121ac73b3022d1483c` | `../echarts` | Seri/bileşen listesi, seçenek tipleri, `defaultOption`, yerleşim, olay ve animasyon davranışı |
| Apache ECharts Examples | `gh-pages`, `1ff3451941535c51af83eacd504035ef4bfd7d0d` | `../echarts-examples` | Galeri kategorileri, örnek metadatası/kaynağı, küçük resimler ve görsel test yaklaşımı |
| Resmi galeri | [echarts.apache.org/examples/en/index.html](https://echarts.apache.org/examples/en/index.html) | — | Kullanıcıya sunulacak liste ve gezinme deneyiminin referansı |

ECharts tarafında başlıca kaynaklar:

- `../echarts/src/export/charts.ts`: resmi yerleşik seri kurulumları.
- `../echarts/src/export/components.ts`: resmi bileşen kurulumları.
- `../echarts/src/chart/**/**Series.ts`: seçenek arayüzleri ve varsayılanlar.
- `../echarts/src/coord/**`, `src/component/**`, `src/data/**`,
  `src/model/**`, `src/core/**`, `src/animation/**` ve `src/label/**`:
  davranışın asıl kaynağı.
- `../echarts/package.json`: ECharts ve zrender sürüm kilidi.

Örnekler tarafında başlıca kaynaklar:

- `../echarts-examples/src/common/config.js`: kategori sırası, GL kategorileri
  ve resmi sitedeki `BLACK_MAP` filtresi.
- `../echarts-examples/src/explore/Explore.vue`: sol kategori menüsü,
  kart ızgarası, `noExplore` filtresi ve koyu kip davranışı.
- `../echarts-examples/src/data/chart-list-data.js`: otomatik üretilen çekirdek
  örnek kataloğu.
- `../echarts-examples/src/data/chart-list-data-gl.js`: otomatik üretilen ve
  bu proje için bütünüyle kapsam dışı olan GL kataloğu.
- `../echarts-examples/public/examples/ts/`: çevrilecek gerçek örnek kaynakları.
- `../echarts-examples/tool/build-example.js`: 700×525 çalışma alanı,
  600×450 küçük resim, `shotWidth`, `shotDelay`, `videoStart` ve `videoEnd`
  kuralları.
- `../echarts-examples/common/compareImage.js`: `pixelmatch` tabanlı piksel
  farkı hesabı.
- `../echarts-examples/e2e/main.js`: Puppeteer ile başsız render, ekran
  görüntüsü ve fark raporu yaklaşımı.
- `../echarts-examples/README.md`: örnek metadatası, yerel ECharts ile çalışma
  ve görsel üretim sözleşmesi.

`chart-list-data*.js` dosyaları resmi repoda otomatik üretilir; elle
düzenlenmez. Cizelge envanteri bu dosyaları ve örneklerin ilk yorum bloğundaki
metadatayı okuyarak yeniden üretilebilir olmalıdır.

## 2. Kesin kapsam

### 2.1 Kapsam içi seriler

ECharts `src/export/charts.ts` içindeki aşağıdaki seriler tüm seçenekleri,
durumları, etkileşimleri, animasyonları ve destekledikleri **Geo dışı**
koordinat dallarıyla kapsam içidir:

- `line`, `bar`, `pie`, `scatter`, `effectScatter`, `candlestick`,
  `boxplot`, `heatmap` ve `pictorialBar`;
- `radar`, `tree`, `treemap`, `sunburst`, `graph`, `chord`, `sankey`,
  `parallel`, `funnel`, `gauge` ve `themeRiver`;
- `lines` ve `custom`;
- aynı option içinde birden fazla seri türünün birlikte kullanıldığı karma
  çizelgeler.

Önemli ayrım:

- Çekirdek `scatter` ve `effectScatter` kapsam içidir; `scatterGL` kapsam
  dışıdır.
- Çekirdek `graph` kapsam içidir; `graphGL` kapsam dışıdır.
- Çekirdek `lines`, `cartesian2d`, `polar`, `calendar` ve `matrix` gibi
  Geo dışı kullanım dallarında kapsam içidir; `linesGL` ve `geo` üzerinde
  çalışan `lines` örnekleri kapsam dışıdır.
- `heatmap`, `custom`, `scatter`, `effectScatter`, `graph` ve `lines`
  serilerinin yalnız `geo`/harita bağımlı dalları kapsam dışıdır. Seri bütünü
  kapsam dışına atılmaz.

### 2.2 Kapsam içi koordinatlar ve bileşenler

- Koordinatlar: `cartesian2d/grid`, `polar`, `radar`, `singleAxis`,
  `calendar`, `matrix` ve `parallel`.
- Bileşenler: title, plain/scroll legend, tooltip, axis/axisPointer, grid,
  polar, radar, singleAxis, calendar, matrix, parallel, dataset, transform,
  dataZoom inside/slider, visualMap continuous/piecewise, toolbox, brush,
  graphic, timeline, thumbnail, markPoint, markLine, markArea ve aria/decal.
- Genel yetenekler: tema ve koyu kip, rich text, etiket yerleşimi, responsive
  `media`, dataset/dimensions/encode/transform, durumlar (`emphasis`, `blur`,
  `select`), olaylar/actions, `setOption` birleştirme kuralları, animasyon ve
  universal transition, progressive/large kipleri, dışa aktarım, erişilebilirlik
  ve yerelleştirme.
- Resmi örnek bir üçüncü taraf istatistik/yerleşim yordamı kullanıyorsa,
  örneğin görsel ve davranışsal sonucu da kapsam içidir. Uyumlu lisanslı bir
  Rust bağımlılığı kullanılabilir; uygun bağımlılık yoksa eşdeğer algoritma
  Apache-2.0 altında bu repoda geliştirilir.

### 2.3 Kapsam dışı — değiştirilemez

1. **Geo/Map:** `geo` koordinat sistemi, `map` serisi, `registerMap`,
   GeoJSON/SVG harita kaydı, projeksiyonlar, coğrafi roam, BMap ve diğer harita
   servisleri.
2. **Tüm 3B çizelgeler:** `globe`, `bar3D`, `scatter3D`, `surface`, `map3D`,
   `lines3D`, `line3D`, `geo3D` ve başka herhangi bir 3B görünüm.
3. **Tüm GL serileri:** `scatterGL`, `linesGL`, `flowGL`, `graphGL` ve
   `echarts-gl` paketinin tamamı.

Bu alanlar için model, boş kabuk, sahte destek rozeti veya sessiz geri düşüş
eklenmez. Envanterde açıkça `kapsam_dışı_geo_map` ya da
`kapsam_dışı_gl_3d` olarak görünürler ve başarı paydasına girmezler.

### 2.4 Galeri görüntüsü için kapsam sayıları

Sayılar yukarıdaki iki commit'in 2026-07-17 görüntüsüne aittir ve envanter
aracı tarafından her yenilemede tekrar hesaplanmalıdır:

- Çekirdek katalog: **377** benzersiz kayıt.
- Resmi keşif sayfasına uygun (`noExplore != true`) çekirdek kayıt: **283**.
- Resmi alan adındaki `BLACK_MAP`, bunlardan `effectScatter-bmap`,
  `lines-airline` ve `scatter-world-population` kimliklerini ayrıca gizler;
  gerçek resmi sayfa bu snapshot'ta **280** çekirdek kayıt gösterir. Üçü de
  zaten Geo/Map kapsam dışı olduğundan kapsam içi sayı değişmez.
- Bunların Geo/Map bağımlı olanları: **22**.
- Resmi sayfada gösterilecek kapsam içi benzersiz örnek: **261**.
- `noExplore` olduğu için kart listesinde gösterilmeyen fakat kapsam içi ve
  ek conformance girdisi olarak kullanılacak örnek: **71**.
- Toplam kapsam içi çekirdek örnek kaydı: **332**.
- GL kataloğu: **59** kayıt; tamamı kapsam dışı. Resmi alan adına özgü
  `BLACK_MAP` filtresi bunların dokuzunu daha gizlese de kapsam kararı değişmez.

Yalnız kategori adına bakmak yeterli değildir. Örneğin `custom-wind` ve
`scatter-world-population` Geo kullandığı hâlde birincil kategorileri `map`
değildir. Kaynak kod bağımlılığı da taranmalıdır.

Bu görüntüde `noExplore` filtresini geçen kümeden Geo/Map nedeniyle çıkarılan
22 çekirdek örnek kimliği (üçü ayrıca yukarıdaki `BLACK_MAP` içindedir):

`custom-hexbin`, `custom-wind`, `effectScatter-bmap`, `geo-beef-cuts`,
`geo-choropleth-scatter`, `geo-graph`, `geo-organ`, `geo-seatmap-flight`,
`geo-svg-custom-effect`, `geo-svg-lines`, `geo-svg-map`,
`geo-svg-scatter-simple`, `geo-svg-traffic`, `lines-airline`, `lines-ny`,
`map-bar-morph`, `map-HK`, `map-iceland-pie`, `map-usa`,
`map-usa-projection`, `matrix-mini-bar-geo` ve `scatter-world-population`.

## 3. "Tam uyum" tanımı

Bir seri adının Rust enumunda bulunması veya tek bir örneğin çizilmesi tam
uyum değildir. Bir yetenek ancak aşağıdaki dört kanıtın tamamı varsa `tam`
sayılır:

1. **API kanıtı:** İlgili ECharts option yolları, varsayılanları, veri
   biçimleri ve geçersiz girdi davranışı özellik matrisinde eşlenmiştir.
2. **Render kanıtı:** Statik ve gerekiyorsa animasyonlu görsel karşılaştırma
   kabul eşiğini geçmiştir.
3. **Davranış kanıtı:** Olay/action, hover/select/blur, güncelleme, resize ve
   ilgili kullanıcı etkileşimleri otomatik senaryoda doğrulanmıştır.
4. **Dayanıklılık kanıtı:** Birim/golden testleri, panik yasağı, lisans
   denetimi ve ilgili performans bütçesi yeşildir.

Durum değerleri yalnız şunlardır:

- `yok`
- `kısmi`
- `uygulandı_kanıt_bekliyor`
- `tam_kanıtlı`
- `kapsam_dışı_geo_map`
- `kapsam_dışı_gl_3d`

`kısmi` hiçbir toplamda tamamlanmış sayılmaz. "Gpui bunu sunmuyor",
"örnekte kullanılmıyor" veya "yaklaşık benziyor" kapanış gerekçesi değildir;
kapsam içi yetenek için eşdeğer yerel çözüm geliştirilir.

## 4. Mevcut reponun başlangıç noktası

2026-07-17 itibarıyla repo güçlü bir başlangıç sunmaktadır:

- `src/model/seri.rs` ve `src/grafik/` altında kapsam içi seri ailelerinin
  çoğu için bir model/render dilimi vardır; belirgin eksik çekirdek seri
  `lines`tır. `effectScatter` ve `pictorialBar` bugün ayrı seri yerine mevcut
  modellerde özellik olarak temsil edilmektedir.
- Kartezyen, polar, radar, singleAxis benzeri akış, takvim ve paralel
  yerleşimlerin başlangıçları vardır; ECharts 6 `matrix` koordinatı yoktur.
- [examples/galeri.rs](examples/galeri.rs) 27 elle yazılmış galeri girdisi
  sunar; repoda 28 örnek dosyası vardır.
- `testler/altin.rs` içinde 48 test ve `testler/altin/` altında 38 komut
  golden dosyası vardır.
- Kayıt, SVG ve piksel yüzeyleri; PNG/SVG dışa aktarım; gpui görünümü,
  temel olaylar ve çeşitli etkileşimler hazırdır.

Bu varlıklar korunacak ve yeni altyapıya taşınacaktır. Bununla birlikte,
27 yerel galeri girdisi ile 261 resmi kapsam içi galeri örneği arasındaki fark
ve ECharts seçenek yüzeyinin derinliği nedeniyle mevcut işaretlerin tamamı
Faz 0'da yeniden denetlenir. Eski "faz tamam" işaretleri yeni matrise otomatik
olarak aktarılmaz.

## 5. Üretilecek uyum envanteri

Faz 0 sonunda aşağıdaki makine-okunur kaynaklar bulunmalıdır:

```text
uyum/
  kaynak_kilidi.toml
  galeri_manifest.json
  ozellik_matrisi.json
  dis_varliklar.toml
  senaryolar/
    <ornek-id>.toml
  sapmalar/
    <ornek-id>.md
testler/gorsel/
  referans/<echarts-commit>/<profil>/<ornek-id>.png
  beklenen/<profil>/<ornek-id>.png
  metrikler/<ornek-id>.json
```

`kaynak_kilidi.toml` ECharts, zrender ve echarts-examples commit/sürümünü,
snapshot tarihini ve kullanılan sabit font/veri görüntüsünü tutar.

Her `galeri_manifest.json` kaydı en az şunları içerir:

- resmi `id`, İngilizce/Türkçe başlık, kategoriler, `difficulty`, `since`,
  `theme`, `shotWidth`, `shotDelay`, `videoStart`, `videoEnd` ve kaynak hash'i;
- resmi örnek yolu ve Cizelge karşılığının Rust fixture/üretici yolu;
- kullandığı seri, koordinat, bileşen, option yolları, actions/events,
  dış veri/varlık ve üçüncü taraf yordamlar;
- kapsam durumu ve makine-okunur gerekçe;
- sahibi olan faz ve bağımlı fazlar;
- API, statik görsel, animasyon, etkileşim, erişilebilirlik ve performans
  kanıtlarının ayrı durumları;
- referans, gerçek, fark ve metrik dosyalarının hash'leri.

`ozellik_matrisi.json`, TypeScript AST ve `defaultOption` kaynaklarından
üretilir. En az şu eksenlerde satır açar:

- seri/bileşen/koordinat adı;
- tam option yolu ve türü;
- ECharts varsayılanı;
- Türkçe Rust API karşılığı;
- desteklenen veri biçimleri ve koordinat dalları;
- kaynak dosya/satır sembolü;
- test ve galeri örnek kimlikleri;
- yukarıdaki kesin durum değeri.

Galeri örnekleri tüm option yüzeyini kapsamaz. Bu nedenle 261 kartın yeşil
olması zorunlu ama tek başına yeterli değildir; özellik matrisinde kapsam içi
hiçbir `yok`, `kısmi` veya `uygulandı_kanıt_bekliyor` satırı kalmamalıdır.
Özellikle
resmi görünür galeride Geo dışı `lines` örneği bulunmadığından, bu seri için
ECharts kaynağına karşı `cartesian2d`, `polar`, `calendar` ve `matrix`
conformance fixture'ları ayrıca yazılacaktır.

## 6. ECharts tarzı Cizelge galerisi

Mevcut tek-grafik ağaç görünümü, resmi `Explore.vue` deneyimini temel alan
manifest güdümlü bir galeriye dönüştürülecektir.

### 6.1 Kategori sırası ve snapshot sayıları

Aşağıdaki sayılar 261 benzersiz örneğin kategori üyelikleridir; bir örnek
birden fazla kategoriye girebildiği için sütun toplamı 261 olmak zorunda
değildir.

| Kategori | Kapsam içi görünür üyelik | Kategori | Kapsam içi görünür üyelik |
|---|---:|---|---:|
| Line | 40 | Bar | 45 |
| Pie | 18 | Scatter/EffectScatter | 29 |
| Candlestick | 10 | Radar | 5 |
| Boxplot | 4 | Heatmap | 5 |
| Graph | 12 | Lines | 0 resmi kart + yerel conformance |
| Tree | 7 | Treemap | 7 |
| Sunburst | 7 | Parallel | 4 |
| Sankey | 7 | Funnel | 4 |
| Gauge | 12 | PictorialBar | 8 |
| ThemeRiver | 2 | Calendar | 9 |
| Matrix | 12 | Chord | 4 |
| Custom | 17 | Dataset | 9 |
| DataZoom | 5 | Graphic | 5 |
| Rich Text | 3 | — | — |

`transform`, `visualMap`, `animation` ve resmi metadatadaki yazım hatalı
`animtion` değerleri ana menü başlığı değil, aranabilir özellik etiketi olarak
korunur. Map/GEO ve GL/3B kategorileri desteklenen kategori menüsünde yer
almaz; ayrı "Kapsam dışı" bilgi görünümünde gerekçeleriyle listelenir.

### 6.2 Galeri kullanıcı deneyimi

- Sol tarafta resmi kategori sırasını izleyen sabit/kaydırılabilir gezinme.
- Sağda pencere genişliğine göre 2–6 sütuna uyarlanan, tembel yüklenen küçük
  resim kartları.
- Kartta başlık, resmi kimlik, kategori, `since`, zorluk ve ayrı API/görsel/
  etkileşim kanıt rozetleri.
- Kimlik, başlık, seri, bileşen ve option yolu üzerinde arama; kategori,
  zorluk, sürüm ve kanıt durumuna göre filtreleme.
- Açık/koyu tema anahtarı; uygun seriler için decal/desen önizlemesi.
- Karta tıklanınca çalışan Cizelge görünümü, Türkçe Rust seçenek kodu,
  resmi ECharts kaynak bağlantısı, kullanılan veri/varlıklar ve kanıt paneli.
- Kanıt panelinde ECharts referansı, Cizelge çıktısı, fark görüntüsü,
  üst-üste bindirme ve sürgülü karşılaştırma.
- Statik örneklerde yeniden çiz/resize; dinamik örneklerde resmi örnekteki
  denetimler, başlat/durdur ve belirlenimci zaman çizgisi.
- Başarısız veya kısmi kart yeşil görünemez. Eksik kanıt doğrudan kartta
  görünür.

Galeri iki çıktı üretir:

1. `cargo run --example galeri` ile gerçek gpui etkileşimli galeri.
2. CI artefaktı olarak aynı manifestten üretilen salt-okunur HTML kanıt
   raporu. Bu rapor kod çalıştırmaz; sonuçları, görüntüleri ve farkları gösterir.

Elle tutulan `KATEGORİLER` sabiti kaldırılır. Yeni örnek eklemek manifest ve
fixture eklemekle mümkün olmalı; galeri listesi, test shard'ları ve rapor aynı
kaynaktan üretilmelidir.

## 7. Görsel ve davranışsal doğrulama sözleşmesi

### 7.1 Referans üretimi

- Referans, ağdaki güncel site yerine kilitli `../echarts` build'i ve
  `../echarts-examples/public/examples/ts/` kaynağıyla yerelde üretilir.
- ECharts tarafında örneğin kendi verisi, teması, gecikmesi ve viewport'u
  korunur. Resmi varsayılan 700×525 viewport ve 600×450 çıktı kullanılır;
  `shotWidth`/`shotDelay` varsa metadata üstündür.
- Fontlar, locale, device pixel ratio, saat dilimi, `Date.now`,
  `Math.random`, veri tohumu ve ağ cevapları sabitlenir.
- Tüm veri ve görseller yerel, hash'li ve lisans envanterli olmalıdır.
  Doğrulama sırasında genel internet erişimi gerekmez.
- `videoStart`/`videoEnd` taşıyan örneklerde başlangıç, %25, %50, %75 ve
  bitiş kareleri alınır; yalnız tek küçük resimle animasyon kanıtlanmış sayılmaz.

### 7.2 Cizelge çıktıları

Her kapsam içi örnek için en az:

- `PikselYüzeyi` ile belirlenimci PNG;
- `SvgYüzeyi` ile parse edilebilir SVG ve yapısal komut golden'ı;
- gerçek gpui penceresinden aynı viewport'ta ekran görüntüsü;
- açık ve koyu kip render smoke testi üretilir.

Decal/desen, renderer farkı, yüksek DPI veya özel tema kullanan örnekler için
manifest ek profiller açar. Gerçek gpui görüntüsü PR'da değişen shard için,
gecelik ve sürüm kapısında 261 örneğin tamamı için çalışır.

### 7.3 Görsel karşılaştırma ve eşikler

Resmi `compareImage.js` yaklaşımı başlangıç alınır:

- Piksel renk toleransı: `pixelmatch threshold = 0.1`.
- Değişen piksel oranı: en fazla `%1`.
- Tamamlayıcı yapısal benzerlik: `SSIM >= 0.99`.
- Çizim komutlarında veri geometrisi, eksen kapsamı, seri/öğe sayısı, metin
  içeriği ve kırpma sınırları ayrıca karşılaştırılır; kritik geometrik sapma
  bir mantıksal pikseli aşamaz.

Referansın iki ardışık üretimi önce kendi içinde kararlılık kontrolünden
geçmelidir. Sistem fontu, rastlantı veya zaman yüzünden kararsız olan örnek
maskelenmeden önce belirlenimci hâle getirilir. Zorunlu dinamik maske varsa
manifestte koordinatı, nedeni, sahibi ve sona erme şartı bulunur. Son tam uyum
sürümünde açık görsel sapma feragati kalamaz.

Referanslar yalnız ECharts commit'i değiştiğinde açık bir "referans yenile"
işlemiyle güncellenir. Cizelge çıktısını referansın üzerine otomatik kopyalayan
bir `bless` yolu bulunmaz.

### 7.4 Etkileşim doğrulaması

Manifestte uygun olan her örnek için belirlenimci eylem dizileri yazılır:

- hover → `emphasis`/tooltip/axisPointer;
- tıkla → select/unselect, legend toggle ve yayılan olay;
- tekerlek/sürükle → dataZoom, roam ve slider;
- brush rect/polygon/lineX/lineY ve seçim temizleme;
- graph düğüm sürükleme, tree aç/kapat, treemap drill-down, sunburst root;
- timeline oynat/durdur/kare seç;
- toolbox restore, dataView, magicType, dataZoom, brush ve saveAsImage;
- `setOption`, `notMerge`, `replaceMerge`, `lazyUpdate`, resize, clear ve
  dispose yaşam döngüsü;
- `dispatchAction` ve karşılık gelen olay yükü.

Her senaryo başlangıç/son görüntüsünü, olay günlüğünü ve option/state
anlık görüntüsünü doğrular. Yalnız ekran görüntüsü, olay semantiğinin kanıtı
değildir.

### 7.5 Görsel olmayan zorunlu kapılar

- ARIA açıklaması, decal, klavye odağı ve renk kontrastı.
- Büyük veri örneklerinde ilk anlamlı çizim, p95 kare süresi, bellek zirvesi
  ve etkileşim gecikmesi.
- PNG/SVG çıktı boyutu, alpha, tema ve yüksek DPI doğrulaması.
- Çalışma zamanı panik yasağı ve hatalı option için tipli tanı.
- `cargo deny check licenses`, `NOTICE` ve dış varlık lisans envanteri.

## 8. Faz planı

Her faz sonunda yalnız o fazın birim testleri değil, önceki fazların tüm
galeri ve matris kanıtları da yeniden çalışır. Bir fazın kabul maddeleri
sağlandığında sonraki faza geçilir; "kısmi" iş bir sonraki faza tamamlanmış
bağımlılık olarak devredilemez.

### Faz 0 — Kaynak kilidi, kapsam envanteri ve kanıt altyapısı

Amaç: Hareketli ve yoruma açık "tam uyum" hedefini ölçülebilir hâle getirmek.

İş kalemleri:

1. `uyum/kaynak_kilidi.toml` ve kaynak commit doğrulama komutu.
2. `chart-list-data.js`, örnek metadata blokları ve kaynak dosyalarından
   377 çekirdek + 59 GL kaydını okuyan envanter üreticisi.
3. Kategoriye ek olarak option/kaynak bağımlılığını inceleyen Geo/Map ve GL
   kapsam sınıflandırıcısı; 261 görünür kapsam içi hedefin sabitlenmesi.
4. ECharts TypeScript AST, exported option tipleri ve `defaultOption`
   nesnelerinden özellik matrisi üretimi.
5. Mevcut Rust API/model/render/test yüzeyini matrise bağlayan başlangıç
   denetimi; eski `✅` kayıtlarının `tam_kanıtlı` ölçütüyle yeniden atanması.
6. Referans ECharts renderer'ı, Cizelge fixture runner'ı, görüntü farkı,
   metrik JSON'u ve HTML rapor iskeleti.
7. Manifest güdümlü yeni galeri kabuğu; başlangıçta mevcut 27 fixture'ı
   göstermesi, eksikleri kırmızı listelemesi.

Kabul:

- 377 çekirdek ve 59 GL kaydın hiçbiri sınıflandırmasız değildir.
- Snapshot sayıları 332 kapsam içi toplam, 261 görünür, 71 gizli conformance,
  45 Geo/Map kapsam dışı ve 59 GL/3B kapsam dışı olarak tekrar üretilebilir.
- Kapsam içi her option yolu matris satırına sahiptir.
- En az bir statik, bir animasyonlu ve bir etkileşimli örnek uçtan uca
  referans/gerçek/fark raporu üretir.

### Faz 1 — Çizim çekirdeği, sahne grafiği, metin ve durumlar

Amaç: Her seri tarafından tekrar kullanılacak zrender eşdeğeri temel.

İş kalemleri:

1. Kimlikli ve hiyerarşik sahne grafiği: group, path, image, text ve temel
   şekiller; z/zlevel/z2 sıralaması, görünürlük, silent/cursor/draggable.
2. Yerel/dünya dönüşümleri: translate, rotate, scale, origin, affine matris,
   iç içe clip path ve contain/isabet testi.
3. Yol tamlığı: line, polyline, polygon, rect, roundRect, circle, ring,
   sector, arc, Bezier, compoundPath ve SVG path data.
4. Stil tamlığı: stroke/fill, alpha, dash/cap/join/miter, shadow, lineer ve
   radyal gradyan, pattern/image ve decal.
5. Metin motoru: sabit font çözümü, ölçüm önbelleği, dönüş, hizalama,
   overflow/truncate/breakAll/break, padding, background/border, rich text
   parçaları ve satır yüksekliği.
6. Ortak label/labelLine yerleşimi, çakışma gizleme/taşıma ve
   `labelLayout` geri çağrısı.
7. Normal/emphasis/blur/select durum mirası, focus/blurScope ve
   stateAnimation.
8. Diff anahtarları, enter/update/leave, keyframe ve temel morph altyapısı;
   pointer yakalama ve olay yayılımı.
9. Kayıt/Piksel/SVG/gpui yüzeylerinin aynı sahne ağacını tüketmesi.

Kabul:

- Primitive, transform, clip, rich text, pattern/decal ve dört durum için
  resmi zrender davranışına karşı görsel fixture'lar yeşildir.
- Döndürülmüş metin artık yatay geri düşüş yapmaz.
- Aynı sahne dört yüzeyde semantik olarak aynı komut/geometriyi üretir.
- Sonraki fazlarda seri başına özel hover/animasyon altyapısı yazılması
  gerekmez.

### Faz 2 — Option, veri, zamanlayıcı ve çalışma zamanı sözleşmesi

Amaç: ECharts'ın `setOption` ve veri boru hattını seri türünden bağımsız
olarak tamamlamak.

İş kalemleri:

1. Component/series kimliği, `id`/`name` eşleme, normal merge,
   `notMerge`, `replaceMerge`, `lazyUpdate`, `silent`, clear/dispose/resize
   ve `getOption` davranışı.
2. `baseOption`, timeline `options` ve responsive `media` sorguları.
3. Source/DataStore: dizi nesneleri, satır/sütun tabloları, typed array,
   dimensions, sourceHeader, seriesLayoutBy, encode ve eksik değerler.
4. Dataset zincirleri, built-in filter/sort transform, çok sonuçlu transform
   ve uyumlu kullanıcı transform trait'i.
5. Scheduler/task aşamaları: data processor, layout, visual, render;
   incremental/progressive yürütme ve iptal.
6. Renk/görsel paleti, visual encoding, ortak stack ve stackStrategy.
7. Action/event kayıt defteri; query ile olay süzme, `dispatchAction`,
   batch payload ve connected chart grupları.
8. `convertToPixel`, `convertFromPixel`, `containPixel`, `appendData`,
   loading ve yaşam döngüsü API'lerinin Rust karşılığı.
9. `init`/instance seçenekleri: locale/theme, boyut, devicePixelRatio,
   coarse pointer, dirty rect, başsız/SSR ve renderer seçiminin Rust'taki
   açık karşılıkları.
10. ECharts `use/register*` genişletme noktalarının Rust trait karşılıkları:
   preprocessor, processor, layout, visual, transform, action, loading ve
   Geo dışı coordinate system. Derleme zamanı kaydı seçilse bile aynı yaşam
   döngüsü ve bağımlılık sırası test edilir.
11. Tanı modeli: bilinmeyen/yanlış option ve desteklenmeyen Geo dalı sessizce
   yok sayılmaz.

Kabul:

- Option merge ve yaşam döngüsü için ECharts ile aynı sonuçları veren tablo
  güdümlü testler vardır.
- Dataset/encode/transform örneklerinin seri bağımsız kısmı hazırdır.
- Progressive görevler UI iş parçacığını bloklamadan iptal/yeniden başlatılır.
- Matrisin runtime/data satırlarında `kısmi` kalmaz.

### Faz 3 — Kartezyen, temel seriler ve pasta tamlığı

Amaç: En yoğun resmi galeri gruplarını tüm seçenekleriyle tamamlamak.

Seriler: line, bar, pie, scatter, effectScatter, candlestick, boxplot,
kartezyen heatmap ve pictorialBar.

İş kalemleri:

1. Grid ve çoklu eksen; value/category/time/log, min/max callback,
   boundaryGap, interval, minor tick, split area, inverse, offset, alignTicks,
   axis break, name/label rotation ve tam zaman biçimleme.
2. Line: smooth/monotone/step, connectNulls, endLabel, area origin,
   symbols, sampling, stack, mark bileşenleri ve polar dal için ortak model.
3. Bar: yatay/dikey/polar, group/stack, realtimeSort, background,
   minHeight/minAngle, barGap/categoryGap ve roundCap.
4. Pie: rose, donut, pad/min angle, zero/empty davranışı, seçili dilim,
   tüm label/labelLine hizalama ve çakışma kuralları.
5. Scatter/effectScatter: tüm semboller, işlevsel boyut/renk, jitter,
   ripple seçenekleri ve Geo dışı koordinatlar.
6. Candlestick/boxplot: veri dönüşümü, item state, yatay/dikey yerleşim ve
   outlier birlikteliği.
7. Heatmap: cartesian hücre, border radius, label ve continuous/piecewise
   visual mapping. Yalnız Geo dalında anlamlı olan `blurSize`, `pointSize`,
   `minOpacity` ve `maxOpacity` option yolları kapsam dışı işaretlenir.
8. PictorialBar: repeat, clip, bounding data, margin, symbol position ve
   animasyon.
9. Bu serilerin tooltip, legend, markers, select/blur/emphasis, dataset,
   dataZoom, animation ve export bütünleşmesi.

Kabul:

- Manifestte bu serilere ait, başka ileri faz özelliği beklemeyen tüm resmi
  örnekler dört kanıtla yeşildir.
- Line 40, Bar 45, Pie 18, Scatter 29, Candlestick 10, Boxplot 4,
  Heatmap 5 ve PictorialBar 8 kategori üyeliği raporda izlenebilir; çapraz
  faz bağımlı kartlar açıkça sonraki faz sahibini gösterir.
- İlgili option matrisi satırlarında kapsam içi eksik kalmaz.

### Faz 4 — Geo dışı koordinatlar ve koordinatsız seriler

Amaç: ECharts 6'nın kalan iki boyutlu koordinatlarını ve gösterge serilerini
tamamlamak.

İş kalemleri:

1. Polar: angle/radius axis'in tüm seçenekleri, start/end angle,
   clockwise, clamp, category/value/time/log davranışı; line/bar/scatter.
2. Radar: shape, indicator, split area/line, name yerleşimi, birden fazla
   radar ve tüm state/label seçenekleri.
3. SingleAxis + ThemeRiver: boundary, label, dataZoom ve katman yerleşimi.
   Scatter, effectScatter, graph ve custom'ın singleAxis bağları da burada
   tamamlanır.
4. Calendar: yatay/dikey, range, cellSize, day/month/year label ve birden
   fazla takvim; heatmap/scatter/effectScatter/custom/lines bağları.
5. Matrix: body/corner/row/column, hücre birleştirme, kategori/değer
   boyutları, mini bar/scatter/heatmap/custom/lines; yalnız `geo` gömülü
   `matrix-mini-bar-geo` hariç.
6. Parallel: axis türleri, expand, areaSelect/brush, active/inactive style,
   çizim ve events.
7. Gauge: axis bands, progress, anchor, pointer, detail/title, overlap,
   roundCap ve değer animasyonu.
8. Funnel: min/max size, gap, align, sort, label/labelLine ve states.
9. Çekirdek `lines`: cartesian2d, polar, calendar ve matrix üzerinde symbol,
   curve, polyline, label, effect/trail ve clip. Geo
   öntanımı kullanılmaz; kullanıcı kapsam içi koordinatı açıkça seçer.

Kabul:

- Radar 5, Gauge 12, Funnel 4, ThemeRiver 2, Calendar 9, Matrix 12 ve
  Parallel 4 kategori üyeliği uygun çapraz bağımlılıklarla yeşildir.
- Resmi görünür galeride kapsam içi `lines` kartı olmadığı için her Geo dışı
  koordinat dalında en az bir ECharts referanslı conformance örneği vardır.
- Hiçbir karma koordinat örneği Geo desteğini dolaylı olarak etkinleştirmez.

### Faz 5 — Hiyerarşik ve ilişkisel seriler

Amaç: Yerleşim ve etkileşim ağırlıklı seri ailelerini tamamlama.

İş kalemleri:

1. Ortak Tree/Graph veri modeli, id/name diff, kategori, edge/node state,
   label ve tooltip.
2. Tree: orthogonal/radial, edgeShape/fork, collapse/expand, initialDepth,
   leaves ve roam.
3. Treemap: squarify, visibleMin/childrenVisibleMin, levels, breadcrumb,
   leafDepth, zoom/root ve üst etiketler.
4. Sunburst: levels, sort, stillShowZeroSum, minAngle, radius, label rotate,
   rootToNode ve highlight/downplay.
5. Graph: none/circular/force, cartesian/polar/singleAxis/calendar/matrix
   bağları, force parametreleri, edge symbols/labels/curveness, node drag ve roam.
   Geo dalı hariçtir.
6. Sankey: node align/depth/gap/width, layout iterations, edge focus,
   orient ve döngü/hata tanıları.
7. Chord: ECharts 6 minAngle/padAngle, ribbon/arc state, sort, label ve
   seçme davranışı.
8. Büyük graph/tree veri kümeleri için artımlı yerleşim, iptal ve kararlı
   belirlenimci test kipi.

Kabul:

- Graph 12, Tree 7, Treemap 7, Sunburst 7, Sankey 7 ve Chord 4 görünür
  kategori üyeliği yeşildir.
- Kuvvet yerleşimi, düğüm sürükleme, roam, drill-down/root ve breadcrumb
  davranışları görüntü + olay günlüğüyle doğrulanır.
- `geo-graph` kapsam dışı kalırken çekirdek Graph'ın diğer dalları tamdır.

### Faz 6 — Custom, Graphic ve Rich Text tamlığı

Amaç: ECharts'ın genel amaçlı iki boyutlu çizim genişletme yüzeyini taşımak.

İş kalemleri:

1. Custom `renderItem` bağlamı: `coord`, `size`, `value`, `ordinalRawValue`,
   `visual`, `barLayout`, currentSeriesIndices, view width/height ve action
   bağlamı.
2. Dönüş tipleri: group, path/pathData, image, text, rect, circle, ring,
   sector, arc, polygon, polyline, line, Bezier ve compoundPath.
3. Custom enter/update/leave, `transition`, `morph`, `during`, clip,
   emphasis/select/blur.
4. Custom'ın none/cartesian/polar/singleAxis/calendar/matrix koordinatları;
   Geo dalı kapsam dışı.
5. Graphic component: id tabanlı merge/replace/remove, nested group,
   layout konumları, transform, draggable, olaylar, keyframe ve textContent.
6. Rich text'in label, tooltip, legend, axis, title ve graphic içindeki tam
   kullanımı; inline image/pattern ve miras.
7. Resmi örneklerdeki histogram, clustering, regression, contour veya başka
   yardımcı algoritmaların lisanslı yerel karşılıkları.

Kabul:

- Custom 17, Graphic 5 ve Rich Text 3 görünür kategori üyeliği yeşildir.
- `custom-wind`, `custom-hexbin` gibi Geo bağımlı örnekler kapsam dışı
  kalır; kalan custom örnekler tam çalışır.
- Kullanıcı özel çizimi gpui, PNG ve SVG yüzeylerinde aynı semantiği taşır.

### Faz 7 — Bileşenler, etkileşim, erişilebilirlik ve çıktı

Amaç: Çizelgeleri ECharts düzeyinde kullanılabilir yapan çevre yeteneklerini
tamamlamak.

İş kalemleri:

1. Title ve plain/scroll legend: selector, pagination, formatter, icon,
   selectedMode, inactive state ve events.
2. Tooltip: axis/item/none, HTML yerine güvenli zengin içerik modeli,
   formatter/valueFormatter, order, confine, enterable eşdeğeri ve gecikmeler.
3. AxisPointer: line/shadow/cross, snap, handle, link mapper, label ve action.
4. DataZoom inside/slider/select: tüm eksen bağları, filterMode, span/range,
   realtime, throttle, data shadow ve events.
5. VisualMap continuous/piecewise: color/symbol/size/opacity boyutları,
   calculable, pieces/categories, hoverLink ve in/outOfRange.
6. Brush: rect/polygon/lineX/lineY, toolbox, brushLink, seriesIndex,
   transformable, throttle ve seçim görseli.
7. Toolbox: saveAsImage, restore, dataView, dataZoom, magicType, brush ve
   güvenli kullanıcı feature trait'i.
8. Timeline, thumbnail, markers, responsive media ve tüm resize akışı.
9. ARIA açıklamaları, decal, klavye erişimi, locale paketleri, koyu/özel
   tema kaydı ve çalışma anında tema değiştirme.
10. PNG/SVG/data URL, bağlı grafik dışa aktarımı, yüksek DPI ve şeffaf arka
    plan.

Kabul:

- Dataset 9, DataZoom 5 ve bileşen etiketi taşıyan tüm kapsam içi kartlar
  yeşildir.
- Galeride açık/koyu tema ve uygun örneklerde decal karşılaştırması vardır.
- Her action için karşılık olay yükü, query filtresi ve sessiz kip testlidir.
- Erişilebilirlik ve çıktı kapıları tüm 261 kart için smoke, ilgili kartlar
  için ayrıntılı doğrulama geçirir.

### Faz 8 — Animasyon, universal transition ve büyük veri

Amaç: Statik benzerliğin ötesinde ECharts'ın akışkanlık ve ölçeklenebilirlik
davranışını tamamlamak.

İş kalemleri:

1. Data diff ile add/update/remove; seri ve öğe gecikme/easing callback'leri.
2. Universal transition: `groupId`, `childGroupId`, divideShape, seriesKey ve
   seri türleri arası morph.
3. Realtime sort/bar race, keyframe graphic/custom animasyonları ve state
   transition.
4. EffectScatter ripple, Lines effect/trail ve Gauge value animation.
5. Gerçek Scheduler progressive parçalama; scatter/bar/candlestick/heatmap/
   lines/custom/parallel için large veya incremental yollar.
6. Typed sütunlu DataStore, appendData/streaming, görünür pencere kırpması,
   uzamsal isabet indeksi, geometri/tessellation cache ve dirty-region.
7. Force/Sankey/Treemap gibi pahalı yerleşimlerin iptal edilebilir arka plan
   işi; sonuçların UI iş parçacığına güvenli aktarımı.
8. Resmi metadata `videoStart`/`videoEnd` örnekleri için kare ve kısa video
   kanıtı.

Kabul:

- Animasyonlu resmi örneklerin başlangıç/ara/bitiş kareleri ve olayları
  referansla uyumludur.
- Otomatik LTTB, ECharts progressive/large desteğinin yerine sayılmaz;
  ilgili option yolları gerçekten uygulanmış ve ölçülmüştür.
- Faz 0'da belirlenen büyük veri bütçeleri sabit CI donanımında aşılmaz ve
  yakınlaştırma/hover sırasında UI uzun görev üretmez.

### Faz 9 — Galeri kapanışı ve sürüm kapısı

Amaç: Bütün parçaları tek, denetlenebilir uyum teslimine dönüştürmek.

İş kalemleri:

1. 261 resmi kapsam içi görünür örneğin Rust fixture'ını ve kartını kapatma.
2. 71 kapsam içi `noExplore` kaydını görünür karta dönüştürmeden conformance
   testine veya aynı yeteneği kanıtlayan açık bir matrise bağlama.
3. Kategori/arama/filtre/dark/decal/diff UX'ini tamamlayıp klavye ve ekran
   okuyucuyla test etme.
4. Tüm referans ve gerçek küçük resimleri kilitli kaynaklarla yeniden üretme;
   başarısız farkları tek tek kapatma.
5. API sözlüğü, Türkçe adlar, taşıma rehberi, destek matrisi ve resmi örnek
   kaynak atıfları.
6. Linux/macOS/Windows, gpui/Piksel/SVG ve gerekli DPI profillerinde tam CI.
7. Lisans/NOTICE, benchmark, panik yasağı ve yayın paketini doğrulama.

Nihai kabul:

- Galeri sayacı `261 / 261 tam_kanıtlı` gösterir.
- 332 kapsam içi resmi çekirdek kaydın tamamı kart veya conformance kanıtına
  bağlıdır.
- Özellik matrisinde kapsam içi `yok`, `kısmi` veya
  `uygulandı_kanıt_bekliyor` satırı yoktur.
- 45 Geo/Map ve 59 GL/3B kayıt yalnız sabit kapsam dışı durumundadır; kapsam
  dışı sayılar başarı oranını şişirmez.
- Açık görsel sapma feragati, kararsız test, lisans bulgusu veya çalışma
  zamanı panik yolu yoktur.

## 9. CI ve değişiklik yönetimi

### Her pull request

- Kaynak kilidi ve manifest şema denetimi.
- Rust format/clippy/test, option matrisi farkı ve panik yasağı.
- Değişen option yollarının etkilediği örnek shard'ları için referanssız
  yeniden render; mevcut kilitli referanslara görsel fark.
- Değişen etkileşim senaryoları ve PNG/SVG/gpui smoke testleri.
- Lisans ve varlık hash denetimi.

### Gecelik

- 261 görünür örneğin açık/koyu render'ı, gerçek gpui ekran görüntüsü ve
  sharded görsel karşılaştırması.
- Animasyon/etkileşim senaryolarının tamamı.
- 71 gizli conformance kaydı ve performans profilleri.
- Fark triptikleri ile tek HTML rapor artefaktı.

### ECharts snapshot yükseltmesi

1. `../echarts` ve `../echarts-examples` yeni commit'leri açıkça seçilir.
2. Envanter `eklendi/silindi/metadata_değişti/option_değişti` raporu üretir.
3. Yeni Geo/GL kayıtlar otomatik kapsam dışı önerilebilir ama insan
   incelemesi olmadan kalıcı sınıflandırılmaz.
4. Yeni kapsam içi kayıt ve option yolları kırmızı olarak matrise girer;
   mevcut `261/261` sayısı eski snapshot etiketiyle korunur.
5. Referans görüntüleri yalnız kaynak yükseltme PR'ında yenilenir.

## 10. Fazlar üstü değiştirilemez kalite kuralları

- Repo ve yeni kod Apache-2.0 lisans sınırında kalır. Örnek kodu, veri,
  görsel, font ve üçüncü taraf algoritmalar `NOTICE`/varlık envanterine girer.
- `src/` ve `examples/` çalışma zamanı kodunda panik üreten yapı kullanılmaz;
  hatalar tipli ve gözlemlenebilir olur.
- Türkçe genel API korunur; manifestte ECharts İngilizce option adıyla
  birebir izlenebilir eşleme bulunur.
- Geo/Map, 3B veya GL desteği başka bir fazın kolaylaştırıcısı olarak dahi
  eklenmez.
- Görsel test eşiği bir özelliği yeşile çevirmek için gevşetilmez. Eşik
  değişikliği ayrı, gerekçeli ve tüm baseline üzerinde ölçülmüş PR ister.
- Yeni seri/bileşen yalnız uygulama koduyla kabul edilmez; manifest, Rust
  fixture, API testi, görsel kanıt, etkileşim senaryosu ve dokümantasyon aynı
  değişiklikte eklenir.

Bu planın tamamlanması, "ECharts'a benzeyen çok sayıda çizelge" değil;
tanımlı kapsam içinde kaynak, option, davranış ve görsel kanıtla denetlenebilen
bir ECharts 6.1 uyarlaması teslim etmek anlamına gelir.
