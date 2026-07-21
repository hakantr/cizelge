# Cizelge — ECharts 6.1 Tam Uyum ve Görsel Galeri Faz Planı

Son güncelleme: 2026-07-20

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

Gerçekleşen dilim — `bar-gradient` (2026-07-20):

- `../echarts-examples/public/examples/ts/bar-gradient.ts` içindeki 20
  kategori/değer, Çince başlık, İngilizce alt başlık, `showBackground`,
  normal ve `emphasis.itemStyle` doğrusal gradyanları kayıpsız fixture'a
  taşındı.
- Ortak çekirdeğe `axisLabel.inside`, eksen `z`, sütun
  `emphasis.itemStyle` mirası ve kategori adı/sayısal değer kabul eden
  `dispatchAction({type: 'dataZoom', startValue, endValue})` eklendi. Yüzde
  ucu aynı taraftaki değer ucunun önüne geçer ve onu temizler; aynı ekseni
  hedefleyen dataZoom bileşenleri atomik güncellenir.
- Kanıt senaryosu üç gerçek durumdan oluşur: başlangıç, canvas pointer ile
  sekizinci sütunun vurgulanması ve aynı sütuna tıklanınca resmi callback'in
  `者..上` aralığına yakınlaştırması. Referans koşucusu hover state'ini ve
  click yükünün iki değer ucunu ekran görüntüsünden önce ayrıca doğrular.
- Kilitli ECharts `74e9e09…` referansına karşı 600×450 sonuçları sırasıyla
  `%0,816 / 0,99689`, `%0,810 / 0,99690` ve `%0,500 / 0,99578`
  (değişen piksel oranı / SSIM) ile kabul eşiğini geçti. Üç referans iki
  ardışık üretimde byte düzeyinde aynı çıktı verdi.
- Kart manifestte `yok`tan `uygulandı_kanıt_bekliyor` durumuna geçti;
  statik görsel kanıtı `tam_kanıtlı`dır. Fazlar üstü animasyon,
  erişilebilirlik ve performans kapıları tamamlanmadan kart yapay biçimde
  `tam_kanıtlı` sayılmaz.

Gerçekleşen dilim — `candlestick-large` (2026-07-20):

- Resmî örnek kaynağı
  `../echarts-examples/public/examples/ts/candlestick-large.ts`; seri
  öntanımları ve large çizim semantiği ise
  `../echarts/src/chart/candlestick/CandlestickSeries.ts`,
  `CandlestickView.ts`, `candlestickLayout.ts`,
  `../echarts/src/chart/bar/BaseBarSeries.ts`, `BarView.ts`,
  `../echarts/src/layout/barGrid.ts` ve
  `../echarts/src/component/dataZoom/SliderZoomView.ts` üzerinden
  doğrulandı. Kaynaklar Faz 0'daki sabit commitlerden okunmuştur.
- Örnekteki 200.000 satır azaltılmadan taşındı. Referans koşucusunun
  Mulberry32 tohumu, JavaScript yuvarlama sırası ve Europe/Istanbul yerel
  zaman akışı korunur; 28 Mart 2011 yaz saati geçişi fixture'da açıkça
  modellenir. İlk iki ve son satırın zaman, OHLC ve hacim değerleri ayrı
  testte kilitlenir; böylece yalnız benzer biçimli sahte bir veri seti kanıt
  sayılmaz.
- Mum dataset eşlemesi `x` ile `[open, close, lowest, highest]` boyutlarını,
  dataset sırasını ve `seriesLayoutBy` yönünü taşır; koordinat dışındaki
  `volume` ve `sign` boyutları visualMap/tooltip için korunur. Mum için
  yükselen/düşen dolgu ile kenarlık renkleri ayrı API yollarıdır.
- `MumSerisi` öntanımları resmî `large: true`, `largeThreshold: 600`,
  `progressive: 3000`, `progressiveThreshold: 10000` değerlerini taşır.
  Large renderer öğe başına 200.000 geometri/isabet bölgesi kurmak yerine
  yükselen ve düşen mumları toplu high-low yollarına böler. `SütunSerisi`
  için `large`, `largeThreshold: 400`, progressive alanları ve resmî large
  kipteki `barMinWidth: 0.5` davranışı eklendi; hacim sütunları tek toplu
  dolgu yoluyla çizilir.
- İki bağlı grid, iki kategori ekseni, `scale: true` değer eksenleri,
  `inside` + `slider` dataZoom (`xAxisIndex: [0, 1]`, `%10..%100`), yalnız x
  yönlü toolbox dataZoom ve gizli parça tabanlı visualMap aynı option
  semantiğiyle kuruldu. `scale: true` bar ekseni geometrik sıfır tabanını
  veri kapsamına katmaz; resmî hacim kapsamı `[3.000.000, 15.000.000]`
  korunur ve grid dışındaki taban kırpılır.
- `SliderZoomView.getShadowDim()` karşılığı olarak candlestick veri gölgesi
  ilk çoklu boyut olan `open` üzerinden üretilir. Büyük veri gölgesi yaklaşık
  bir örnek/piksel adımıyla çizilir; ilk seri seçimi, seçili/seçili-dışı
  gölge parçaları ve bağlı eksen penceresi mevcut dataZoom hattında kalır.
- Large mum ve large sütun yolları, dataset OHLC sırası, ölçekli bar kapsamı,
  candlestick dataZoom gölgesi ve 200.000 satırlık deterministik akış birim
  testleriyle kapatıldı. Bu testler görsel eşiği gevşetmeden gerçek çekirdek
  davranışını korur.
- Kilitli ECharts `74e9e09…` referansı iki ardışık üretimde piksel düzeyinde
  aynı çıktı verdi. 600×450 Cizelge sonucu `%0,608` değişen piksel oranı ve
  `0,99534` SSIM ile `%1 / 0,99` kabul eşiğini geçti; referans, gerçek, fark
  ve metrik dosyaları galeri manifestine hashleriyle bağlandı.
- Kart `yok`tan `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 144/332, yani `%43,4`
  oldu. Gerçek scheduler progressive parçalama, etkileşim,
  erişilebilirlik ve ölçümlü performans Faz 7/8 kapıları tamamlanmadan kart
  nihai `tam_kanıtlı` sayılmaz.

Gerçekleşen dilim — `candlestick-sh` (2026-07-20):

- Resmî örnek ve veri akışı
  `../echarts-examples/public/examples/ts/candlestick-sh.ts` dosyasından
  alındı. Marker istatistikleri için
  `../echarts/src/component/marker/markerHelper.ts`,
  `../echarts/src/component/marker/MarkPointView.ts` ve
  `../echarts/src/component/marker/MarkLineView.ts`; zoom penceresi için
  `../echarts/src/component/dataZoom/AxisProxy.ts`; mum genişliği ve eksen
  kapsamı için `../echarts/src/chart/candlestick/candlestickLayout.ts`,
  `CandlestickSeries.ts` ve
  `../echarts/src/chart/helper/axisSnippets.ts`; legend mirası için
  `../echarts/src/component/legend/LegendView.ts` sabit ECharts commitinde
  doğrulandı.
- Kaynaktaki 88 tarihli `[open, close, lowest, highest]` satırı azaltılmadan
  `examples/uyum_veri/candlestick_sh.rs` içine taşındı. `calculateMA`
  yordamının ilk `dayCount` öğeyi boş bırakma ve geçerli noktada bugün ile
  önceki `dayCount - 1` kapanışı toplama sırası korunarak MA5, MA10,
  MA20 ve MA30 serileri üretildi. Kaynak uçları ve ilk MA5 sonucu fixture
  testinde kilitlendi.
- Başlık, beş öğeli legend, `%10/%10/%15` grid, `boundaryGap: false`,
  `axisLine.onZero: false`, `min/max: dataMin/dataMax`, `scale: true`,
  `splitArea`, eksen tetiklemeli çapraz pointer ve `%50..%100` aralıklı
  `inside` + `slider` dataZoom aynı option anlamlarıyla fixture'a bağlandı.
  MA çizgilerinin `smooth: true` ve `lineStyle.opacity: 0.5` değerleri ile
  resmi palet sırası korunur.
- Marker modeli `valueDim` için ad ve sıra seçicilerini taşır. Dataset
  kullanılmadığında da mumun `open`, `close`, `lowest`/`low` ve
  `highest`/`high`; kutunun `min`, `q1`, `median`, `q3`, `max` boyutları
  çözülür. `min`, `max` ve `average` hesabı yalnız dataZoom `filter`
  penceresinde kalan sonlu öğeleri tüketir.
- `markPoint` için 2013/5/31 kategorisindeki koyu 2300 raptiyesi, en yüksek
  `highest`, en düşük `lowest` ve ortalama `close` raptiyeleri taşındı.
  ECharts `markerHelper`, ortalama koordinatında soyut ortalamayı değil
  eksen piksel uzayında ona en yakın gerçek veri öğesini seçer; bu nedenle
  resmi etiket `2242` olur. Eşit uzaklıkta taraf seçimi çekirdek testinde;
  öğeye özgü `itemStyle`, genel formatter ve seri/veri adı bağlamı
  görsel kanıtta doğrulandı.
- `markLine` için `lowest` minimumundan `highest` maksimumuna giden,
  iki ucunda 10 px daire bulunan ve etiketi gizli çift uçlu çizgi ile
  `close` minimum/maksimum yatay çizgileri eklendi. Uçlar ayrı `valueDim`,
  simge ve simge boyutu taşır; tekil ortalama çizgisi hesaplanmış sayıyı,
  istatistik uçlu çizgi ise ECharts gibi en yakın gerçek öğeyi kullanır.
- ECharts 6 `createBandWidthBasedAxisContainShapeHandler` karşılığı,
  `boundaryGap: false` kategori eksenindeki mum/kutu/sütun gövdelerini
  dataZoom penceresinin iki ucunda yarım bantla kapsar. Böylece ilk ve son
  mum kırpılmaz ve kategori koordinatları resmi görüntüyle eşleşir. Mumun
  yükselen rengi ile kenarlığı legend/marker'a geçer, mum serisi sonraki
  MA çizgilerinin palet sırasını tüketmez ve karışık legend satırı ortak
  görünen sınıra hizalanır.
- Birim kapısı 274/274, fixture kapısı 33/33 geçti;
  `cargo check --all-targets`, `cargo check --no-default-features` ve
  `node tools/uyum/uret.mjs --check` temizdir. Tam kanıt koşusu yeni kart
  dahil 170/170 kareyi geçirdi; mevcut kilitli referanslar yenilenmedi.
- Yeni resmi referans iki ardışık üretimde piksel düzeyinde kararlıdır.
  600×450 Cizelge sonucu 1.969 değişen piksel, `%0,7293` fark ve
  `0,991814` SSIM ile `%1 / 0,99` kabul eşiğini geçti. Referans, gerçek,
  fark ve metrik dosyaları ECharts tarzı galeri manifestine SHA-256
  değerleriyle bağlandı.
- Kart `yok`tan `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 145/332, yani
  `%43,7` oldu. Animasyon, etkileşim, erişilebilirlik ve ölçümlü performans
  kapıları ilgili ileri fazlarda kapanmadan kart nihai `tam_kanıtlı`
  sayılmaz.

Gerçekleşen dilim — `candlestick-brush` (2026-07-20):

- Resmî örnek ve veri akışı
  `../echarts-examples/public/examples/ts/candlestick-brush.ts` ile
  `../echarts-examples/public/data/asset/data/stock-DJI.json` dosyalarından
  alındı. Brush model/görsel semantiği
  `../echarts/src/component/brush/BrushModel.ts`, `selector.ts` ve
  `visualEncoding.ts`; mumun görsel kanalları
  `../echarts/src/chart/candlestick/candlestickVisual.ts`,
  `CandlestickSeries.ts` ve `CandlestickView.ts`; hacim renk eşlemesi
  `../echarts/src/component/visualMap/visualEncoding.ts` üzerinden sabit
  kaynak commitlerinde doğrulandı.
- DJI varlığındaki 3.141 `[date, open, close, lowest, highest, volume]`
  satırı azaltılmadan kullanılır. `2004-01-02` ve `2016-06-22` uçları,
  brush aralığının `2016-06-02` / `2016-06-20` ham sıraları ve hacim işaret
  boyutu fixture testinde kilitlidir. MA5/10/20/30 hesapları JavaScript
  örneğindeki ilk `dayCount` değeri boş bırakma, kapanışları aynı sırada
  toplama ve üç basamağa yuvarlama davranışını korur.
- Beş öğeli legend, eksen tetiklemeli çapraz tooltip/pointer bağlantısı,
  yalnız x yönlü dataZoom ve `lineX`/`clear` brush araçları, iki grid, iki
  kategori ve iki ölçekli değer ekseni, bağlı `inside` + `slider` dataZoom
  (`%98..%100`), bir mum, dört yumuşak MA çizgisi ve hacim sütunu aynı
  option anlamlarıyla kuruldu. Gizli parça tabanlı visualMap, hacmin
  adlandırılmış `sign` boyutunu kırmızı/yeşil renge eşler.
- Brush modeli `rect`, `polygon`, `lineX` ve `lineY` alanlarını; sayısal ya
  da kategori `coordRange` uçlarını; `xAxisIndex`, `yAxisIndex`,
  `seriesIndex`, `transformable`, çoklu seçim, resmî `brushStyle`,
  `inBrush.colorAlpha`, `outOfBrush.colorAlpha` ve `brushLink` için
  `none`/`all`/seri listesi biçimlerini taşır. Bu alanların Rust dışa
  aktarımları ve akıcı kurucuları tek bir ECharts tarzı seçenek ağacında
  kullanılabilir.
- `dispatchAction({type: 'brush', areas})` kayıt defterine gerçek bir action
  olarak eklendi. İç içe action yükü doğrulanarak tipli seçim alanlarına
  çevrilir; eksik alan listesi mevcut seçimi korur, brush bileşeni yoksa
  güvenli no-op olur ve sessiz olmayan güncelleme olay/yeniden çizim üretir.
  Fixture, resmî örnekte `setOption(..., true)` sonrasında gönderilen tarih
  aralığını bu çalışma zamanı yolundan gerçekten yeniden oynatır.
- Ortak renderer programatik alanları veri koordinatından ilgili grid
  pikseline çözer, alan birleşimini hesaplar ve seçili ham veri sıralarını
  `brushLink` kapsamındaki mum, çizgi ve hacim serilerine yayar. İç/dış
  maskeler öğe opaklığına dönüşür. ECharts `drawType` kuralına uygun olarak
  mumda yalnız gövde dolgusu `colorAlpha` ile solar; fitil ve kenarlık opak
  kalır. Sütun visualMap'i adlandırılmış boyutu okur ve dataZoom sonrasında
  eşik altında kalan mum sayısı large yerine normal öğe yolunu seçer.
- Legend için ECharts `itemWidth`, `itemHeight` ve `itemGap` karşılıkları
  akıcı API'ye tamamlandı. Kanıt fixture'ındaki yalnız raster geometriye ait
  kesirli simge ölçüleri, Cizelge SVG çıktısının ECharts Canvas çıktısıyla
  600×450 küçültmede aynı piksel sınırlarına oturmasını sağlar; modelin
  resmî öntanımlıları değiştirilmedi.
- Birim kapısı 279/279, fixture kapısı 34/34 geçti;
  `cargo check --all-targets`, `cargo check --no-default-features` ve
  `node tools/uyum/uret.mjs --check` temizdir. Yeni kart dahil tam görsel
  regresyon 171/171 kareyi geçirdi ve önceden kilitli hiçbir resmî referans
  yenilenmedi.
- Yeni resmî referans iki ardışık üretimde piksel düzeyinde kararlıdır.
  600×450 Cizelge sonucu 1.390 değişen piksel, `%0,5148` fark ve `0,990133`
  SSIM ile `%1 / 0,99` kabul eşiğini geçti. Referans, gerçek, fark ve metrik
  dosyaları ECharts tarzı galeri manifestine SHA-256 değerleriyle bağlandı.
- Kart `yok`tan `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 146/332, yani `%44,0`
  oldu. Serbest pointer ile alan çizme/dönüştürme, animasyon,
  erişilebilirlik ve ölçümlü performans kapıları ilgili ileri fazlarda
  kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

Gerçekleşen dilim — `candlestick-sh-2015` (2026-07-20):

- Resmî kaynak
  `../echarts-examples/public/examples/ts/candlestick-sh-2015.ts`
  dosyasıdır. Mum yerleşimi/görseli için
  `../echarts/src/chart/candlestick/CandlestickSeries.ts`,
  `CandlestickView.ts`, `candlestickLayout.ts` ve `candlestickVisual.ts`;
  özel sürgü tutamacı ile veri gölgesi için
  `../echarts/src/component/dataZoom/SliderZoomModel.ts` ve
  `SliderZoomView.ts` sabit ECharts commitinde karşılaştırıldı.
- Kaynaktaki tek satırlık `rawData` kopyalanıp sadeleştirilmedi; fixture,
  sabitlenmiş TypeScript dosyasındaki diziyi doğrudan ve doğrulamalı olarak
  okur. JavaScript `.reverse()` sırası ile sonda bulunan iki boş satırın
  başa gelmesi ve `+''` işlemiyle OHLC değerlerinin sıfıra dönüşmesi korunur.
  Sonuç 246 satırdır; ilk gerçek `2015/1/5` ve son `2015/12/31` değerleri
  ayrı testte kilitlidir.
- MA5/10/20/30, ilk `dayCount` öğeyi boş bırakır ve kaynaktaki kayan kapanış
  toplamını yuvarlama eklemeden uygular. Beşinci sıradaki `2673,876` sonucu,
  iki sıfır satırın hesaptan yanlışlıkla atılmadığını ayrıca kanıtlar.
- Kaynaktaki ad uyuşmazlığı da değiştirilmedi: legend `日K` isterken mum
  serisinin adı `Day` olduğundan ECharts gibi yalnız MA5/10/20/30 legend
  öğeleri görünür. `inactiveColor`, eksen tetiklemeli ve animasyonsuz çapraz
  pointer, `#8392A5` eksen çizgileri, `scale: true`, kapalı yatay bölme
  çizgileri, `bottom: 80` grid, varsayılan tam aralıklı `slider` + `inside`
  dataZoom ve dört simgesiz/yumuşak 1 px MA çizgisi taşındı. Mumun yükselen
  ve düşen dolgu/kenarlık renkleri ayrı kanallarda korunur.
- `handleIcon` içindeki resmî `path://` SVG verisi ortak yol çözücüsünden
  geçirilerek sürgünün iki ucunda en-boy oranını koruyan gerçek geometriyle
  çizilir. Kaynaktaki tam-seçili başlangıç durumu, veri gölgesi, seçim
  dolgusu ve tutamaçlarıyla birlikte kilitli görsel kanıtta doğrulandı.
- Birim kapısı 279/279, fixture kapısı 35/35, her iki `cargo check` kipi ve
  üretilmiş-dosya denetimi geçti. Mevcut hiçbir resmî referans yenilenmeden
  tam görsel regresyon 172/172 kareyi geçti. Yeni 600×450 sonuç 1.056
  değişen piksel, `%0,3911` fark ve `0,996551` SSIM üretir; dört kanıt
  dosyası hashleriyle galeri manifestine bağlıdır.
- Kart `yok`tan `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 147/332, yani `%44,3`
  oldu. Bu statik karede görünür sonucu değiştirmeyen
  `dataZoom.dataBackground`, `dataZoom.textStyle`, `brushSelect` ve
  `tooltip.axisPointer.lineStyle` seçeneklerinin genel API/etkileşim
  kapanışı Faz 5/6 matrisinde `kısmi` kalır; bunlar tamamlanmadan kart nihai
  `tam_kanıtlı` sayılmaz.

Gerçekleşen dilim — `bar-brush` (2026-07-20):

- Resmî örnek ve olay akışı
  `../echarts-examples/public/examples/ts/bar-brush.ts` dosyasından taşındı.
  Brush seçimi ve varsayılan görsel kanalları
  `../echarts/src/component/brush/BrushModel.ts`, `selector.ts`,
  `visualEncoding.ts`, `install.ts` ve `preprocessor.ts`; yan yana/stack
  sütun yerleşimi ise `../echarts/src/chart/bar/` ile
  `../echarts/src/layout/barGrid.ts` kaynaklarından sabit committe
  doğrulandı.
- Örneğin on kategorisi ve dört serisi, resmî koşucudaki deterministik
  Mulberry32 akışıyla üretildi. `bar`/`bar2` serileri `one`, `bar3`/`bar4`
  serileri `two` stack'inde; legend'in `%10` sol konumu, boş tooltip,
  `X Axis` adlı kategori ekseni, `bottom: 100` grid, 10 px vurgu gölgesi ve
  toolbox içindeki `magicType(stack)`, `dataView`, ardından
  `rect/polygon/lineX/lineY/keep/clear` sırası korunur.
- Programatik `dispatchAction({type: 'brush', areas})` sonucuna seri bazlı
  ham veri sıraları eklendi ve `BoyamaÇıktısı.fırça_seçimleri` üzerinden
  tipli olarak dışarı açıldı. `Class2..Class5` lineX alanında ECharts'ın
  gerçek sütun gövdesi isabeti korunur: `one` stack'indeki ilk iki seri
  `[3, 4, 5]`, sağdaki `two` stack'indeki son iki seri `[2, 3, 4]` seçer.
  Böylece seçim kategori merkezinden kestirilmez; side-by-side stack
  dikdörtgeninin gerçek merkezi ve ham veri sırası kullanılır.
- `Fırça` API'sine `inBrush.color` ve `outOfBrush.color` karşılıkları
  eklendi. Açık bir dış görsel verilmediğinde ECharts tema token'ının
  devre dışı rengi `#cfd2d7` uygulanır; yalnız alpha kanalı verilmişse renk
  zorla değiştirilmez. Öğe bazlı brush görseli gereken sütun serisi large
  toplu yoldan güvenle normal öğe yoluna düşer; seçili ve seçili-dışı
  renkler her veri öğesine ayrı uygulanır.
- Resmî `brushSelected` callback'i seçilen ham sıraları birleştirip
  `SELECTED DATA INDICES` başlığını `bottom: 0`, `right: 10%`,
  `width: 100`, 12 px beyaz yazı ve `#333` zeminle kurar. Ortak başlık
  modeli `right`, `bottom` ve `width` option yollarını taşır; sağ/alt
  yerleşim, kalın yazının gerçek ölçüsüyle hesaplanır. Fixture koşucusu
  hem gönderilen brush action'ını hem de oluşan dört seri satırlı başlık
  metnini ekran görüntüsünden önce doğrular.
- Çekirdek birim kapısı 281/281, fixture kapısı 37/37 geçti;
  `cargo check --all-targets`, `cargo check --no-default-features` ve
  `node tools/uyum/uret.mjs --check` temizdir. Yeni iki kare dahil tam
  görsel regresyon 174/174 geçti ve önceden kilitli hiçbir resmî referans
  yenilenmedi. Ortak kalın başlık ölçümü yalnız iki mevcut
  `boxplot-light-velocity*` gerçek/fark çıktısını yeniden üretti; sabit
  referansları değişmeden SSIM değerleri iyileşti.
- Başlangıç karesi 741 değişen piksel, `%0,2744` fark ve `0,999128` SSIM;
  lineX seçim karesi 1.316 değişen piksel, `%0,4874` fark ve `0,998631`
  SSIM üretti. İki resmî referans yalnız bu yeni kart için bir kez üretildi,
  ardından referans, gerçek, fark ve metrik dosyaları SHA-256 değerleriyle
  galeri manifestine kilitlendi.
- Kart `yok`tan `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 148/332, yani `%44,6`
  oldu. Pointer ile serbest alan oluşturma/dönüştürme, tüm brush olay yaşam
  döngüsü, animasyon, erişilebilirlik ve ölçümlü performans kapıları Faz
  5/6/7/8'de kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

Gerçekleşen dilim — `bar-polar-label-radial` (2026-07-20):

- Resmî örnek
  `../echarts-examples/public/examples/ts/bar-polar-label-radial.ts`
  dosyasından kayıpsız fixture'a taşındı. Polar oluşturma ve yarıçap
  çözümü `../echarts/src/coord/polar/PolarModel.ts` ile `polarCreator.ts`;
  bar sektör yerleşimi `../echarts/src/layout/barPolar.ts`; sektör metni
  konum/dönüşü `../echarts/src/chart/bar/BarView.ts` ile
  `../echarts/src/label/sectorLabel.ts`; eksen katmanları ise
  `../echarts/src/component/axis/AngleAxisView.ts` ve
  `RadiusAxisView.ts` üzerinden sabit ECharts commitinde doğrulandı.
- `polar.radius: [30, '80%']` tek dış yarıçap modeline indirgenmedi.
  `KutupsalKoordinat::yarıçap_aralığı(30, "80%")` iç ve dış uçları ayrı
  taşır; 700×525 görünümde 30 px iç, 210 px dış yarıçap üretir. Radyal
  değer ölçeği bu fiziksel aralığa eşlenir; bölme halkaları, radiusAxis
  çizgisi, angleAxis ışınları, çentik ve isabet geometrileri iç halkayı
  doğru başlangıç kabul eder.
- `radiusAxis.max: 4`, `angleAxis.type: 'category'`, `a/b/c/d` verisi ve
  `startAngle: 75` aynı option anlamlarıyla kuruldu. İlk kategori merkezi
  ekran uzayında −30°'ye düşer. Polar barın açık `barWidth` olmadığı
  durumdaki resmî `%20` kategori boşluğu uygulanarak dört 90° bandın her
  birinde 72° sektör oluşturulur; açık `barWidth`, `barMinWidth`,
  `barMaxWidth` ve `barCategoryGap` değerleri de açı biriminde çözülür.
- Polar bar etiketleri ortak renderer'a eklendi. `start`, `insideStart`,
  `middle`, `end` ve `insideEnd` option yolları sırasıyla
  `EtiketKonumu::Başlangıç`, `İçBaşlangıç`, `Merkez`, `Bitiş` ve
  `İçBitiş` olarak tiplenir. Resmî `{b}: {c}` formatter'ı kategori adı ve
  ham değeri kullanır; varsayılan iç yazı rengi sektör dolgusunun
  parlaklığından gelir. Açık `rotate` yokken zrender'ın sektör açısına göre
  teğetsel döndürme ve `middle` için okunabilir yarım tur çevirme kuralı
  ekran koordinatı işaretiyle korunur.
- Polar çizim seri öncesi/sonrası iki geçişe ayrıldı: varsayılan `z=0`
  eksenin splitLine, axisLine, axisTick ve axisLabel öğeleri `z=2` serinin
  altında; yalnız `axis.z > 2` isteyen eksenin bütün öğeleri üstündedir.
  Veri öğesine özgü dolgu, opaklık ve kenarlık da sektör çizimine taşınır.
  Bu ortak düzeltme mevcut `line-polar`, `line-polar2` ve
  `scatter-polar-punchCard` referanslarını değiştirmeden SSIM değerlerini
  izleyen `axis.z` düzeltmesinden sonra sırasıyla `0,997712`, `0,997679`
  ve `0,997008` düzeyinde tutar.
- Fixture, resmî başlığı, boş tooltip'i, dört değeri
  `[2, 1.2, 2.4, 3.6]`, eksen kapsamını, yarıçap uçlarını ve formatter
  bağlamını ayrı testte kilitler. Çekirdek birim kapısı 282/282, fixture
  kapısı 38/38 geçti; `cargo check --all-targets`,
  `cargo check --no-default-features` ve
  `node tools/uyum/uret.mjs --check` temizdir. Önceden kilitli hiçbir resmî
  referans yenilenmeden tam görsel regresyon 175/175 kareyi geçti.
- İlk 600×450 kilitli kanıt 942 değişen piksel, `%0,3489` fark ve
  `0,997158` SSIM üretti. İzleyen teğetsel polar bar diliminde AngleAxis'in
  iki uçlu radius için iç halkası da tamamlanınca aynı kilitli referans
  yenilenmeden sonuç 782 piksel, `%0,2896` fark ve `0,997534` SSIM'e;
  polar eksenlerin resmî `z=0` katmanına alınmasıyla da 759 piksel,
  `%0,2811` fark ve `0,997563` SSIM'e iyileşti. Resmî referans iki ardışık
  üretimde piksel düzeyinde aynı çıktı verdikten sonra yalnız bu kart ilk
  açılırken bir kez oluşturuldu; referans, gerçek, fark ve metrik dosyaları
  hashleriyle galeri manifestine bağlıdır.
- Kart `yok`tan `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 149/332, yani `%44,9`
  oldu. Teğetsel polar bar, çoklu stack/grup yerleşimi, `roundCap`,
  `barMinAngle`, polar tooltip/hover davranışının bütün kombinasyonları,
  animasyon, erişilebilirlik ve ölçümlü performans izleyen Faz 3/4/6/7/8
  kartlarıyla kapanmadan bu kart nihai `tam_kanıtlı` sayılmaz.

Gerçekleşen dilim — `bar-polar-label-tangential` (2026-07-20):

- Resmî örnek
  `../echarts-examples/public/examples/ts/bar-polar-label-tangential.ts`
  dosyasından kayıpsız fixture'a taşındı. Kategorik taban eksenini seçen
  kutupsal bar yerleşimi `../echarts/src/layout/barPolar.ts`; barın sektör
  görünümü ve radial/tangential konum eşlemesi
  `../echarts/src/chart/bar/BarView.ts`; sektör metninin
  `startAngle`/`insideStartAngle`/`middle`/`endAngle`/`insideEndAngle`
  geometrisi ve otomatik dönüşü `../echarts/src/label/sectorLabel.ts`
  üzerinden sabit ECharts commitinde doğrulandı. AngleAxis'in iki yarıçaplı
  çizgisi de resmî `AngleAxisView` Ring davranışıyla karşılaştırıldı.
- Polar tek değerli bar kapsamı artık eksen rollerine göre çözülür.
  `radiusAxis` kategorik, `angleAxis` değer olduğunda yığın aralığının
  taban/tepe değerleri açısal kapsama; veri sırası radyal kategori kapsamına
  gider. Böylece `[2, 1.2, 2.4, 3.6]` verisi yarıçap uzunluğu gibi değil,
  sırasıyla 180°, 108°, 216° ve 324° saat yönlü sektör süpürmesi olarak
  çizilir.
- `polar.radius: [30, '80%']` 700×525 görünümde yine 30..210 px aralığı
  üretir. Dört kategorinin her biri 45 px radyal banttır;
  `barCategoryGap` verilmediğinde resmî `%20` boşluk uygulanıp 36 px sektör
  kalınlığı elde edilir. Açık `barWidth`, `barMinWidth`, `barMaxWidth` ve
  `barCategoryGap` yüzdeleri banda göre, sayısal değerleri radiusAxis
  tabanında piksel olarak çözülür.
- Teğetsel sektör animasyonu bitiş açısını başlangıçtan hedefe büyütür;
  normal ve veri öğesine özgü dolgu, opaklık, kenarlık ile halka isabet
  geometrisi aynı gerçek `r0/r/startAngle/endAngle` şekline bağlanır.
  `outside` pozitif/negatif süpürmeye göre doğru uç kenarını seçer;
  `start`, `insideStart`, `middle`, `end`, `insideEnd` konumları sektörün
  açısal kenarlarına, varsayılan iç/dış yazı rengine ve zrender'ın okunabilir
  teğetsel dönüşüne karşılık gelir. `{b}: {c}` formatter'ı bu yönelimde
  kategori adını `radiusAxis` ölçeğinden alır.
- `angleAxis.max: 4`, varsayılan on iki açısal bölme, `startAngle: 75`,
  kategorik `radiusAxis` içindeki `a/b/c/d`, boş tooltip, `middle` etiketi
  ve resmî kaynağın kapatmadığı animation varsayılanı fixture testinde
  ayrı ayrı kilitlendi. AngleAxis axisLine dış çembere ek olarak 30 px iç
  çemberi de çizer; bu düzeltme önceki radyal polar kanıtın SSIM değerini
  eski referansa dokunmadan yükseltti.
- Çekirdek birim kapısı 283/283, fixture kapısı 39/39 geçti;
  `cargo check --all-targets`, `cargo check --no-default-features` ve
  `node tools/uyum/uret.mjs --check` temizdir. Yeni kart kilitli referansla
  bağımsız 1/1 tekrar koşusunu, tüm depo ise önceden kilitli hiçbir resmî
  referans yenilenmeden 176/176 görsel regresyonu geçti.
- Yeni 600×450 kanıt ilk geçişte 1.174 değişen piksel, `%0,4348` fark ve
  `0,997553` SSIM üretti. Polar eksenlerin seriyle gerçek `z` sırasına
  alınmasıyla kilitli referansa dokunmadan güncel sonuç 1.169 piksel,
  `%0,4330` fark ve `0,997557` SSIM'dir. Resmî referans iki ardışık üretimde
  piksel düzeyinde aynı çıktı verdikten sonra yalnız bu yeni kart için bir
  kez oluşturuldu; referans, gerçek, fark ve metrik dosyaları hashleriyle
  galeri manifestine bağlandı.
- Kart `yok`tan `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 150/332, yani `%45,2`
  oldu. Birden fazla bağımsız polar stack/grubun bant paylaşımı,
  `roundCap`, `barMinAngle`, tüm negatif/ters/saat yönü kombinasyonları,
  tooltip/hover yaşam döngüsü, animasyon, erişilebilirlik ve ölçümlü
  performans izleyen Faz 3/4/6/7/8 kartları kapanmadan bu kart nihai
  `tam_kanıtlı` sayılmaz.

Gerçekleşen dilim — `bar-polar-stack` (2026-07-20):

- Resmî örnek `../echarts-examples/public/examples/ts/bar-polar-stack.ts`
  dosyasından kayıpsız fixture'a taşındı. Kutupsal sütun yerleşimi
  `../echarts/src/layout/barPolar.ts`; ortak yığın boyutları ve hesaplama
  sırası `../echarts/src/data/helper/dataStackHelper.ts` ile
  `../echarts/src/processor/dataStack.ts`; eksen katmanları
  `../echarts/src/component/axis/AngleAxisView.ts` ve `RadiusAxisView.ts`;
  gösterge ölçümü/yerleşimi ise `../echarts/src/component/legend/LegendModel.ts`
  ile `LegendView.ts` üzerinden sabit ECharts commitinde doğrulandı.
- Boş `polar`, `angleAxis` ve resmî `radiusAxis` option'ı tipli modele aynı
  anlamla aktarıldı. `radiusAxis.type: 'category'`,
  `['Mon', 'Tue', 'Wed', 'Thu']` ve `z: 10` korunur; eksen çizgi, çentik ve
  etiketleri yığın sektörlerinin üstünde çizilir. ECharts'ın varsayılan
  `%50/%50` merkezi ile `%80` dış yarıçapı 700×525 fixture görünümünde
  `(350, 262,5)` merkez ve 210 px yarıçap üretir.
- Dört radius kategorisinin her biri 52,5 px banttır. Açık `barWidth` veya
  `barCategoryGap` verilmediği için resmî `%20` kategori boşluğu uygulanır;
  her yığın 42 px kalınlıkla ve sırasıyla 5,25..47,25,
  57,75..99,75, 110,25..152,25 ve 162,75..204,75 px yarıçap aralıklarında
  çizilir. Bu hesap önceki teğetsel polar sütunun açık
  `barWidth`/`barMinWidth`/`barMaxWidth`/`barCategoryGap` yollarıyla aynı
  ortak yerleşimden gelir.
- Üç `bar` serisinin `coordinateSystem: 'polar'`, adları `A/B/C`, verileri
  `[1, 2, 3, 4]`, `[2, 4, 6, 8]`, `[1, 2, 3, 4]` ve ortak `stack: 'a'`
  değeri fixture testinde kilitlendi. İşaret-korumalı yığın motoru açısal
  aralıkları A için `0→[1,2,3,4]`, B için `[1,2,3,4]→[3,6,9,12]`, C için
  `[3,6,9,12]→[4,8,12,16]` olarak üretir; sektörlerin birbirinin üstüne
  binmesi yerine aynı radyal bantta uç uca eklenmesi böylece doğrulanır.
- Ortak yığının açısal veri kapsamı `0..16` olarak toplanır. Boş angleAxis'in
  varsayılan on iki bölmeli güzel ölçeği 1 birim aralık seçer ve `0..15`
  etiket/ışınlarını üretir; 16'nın 0 ile aynı kutupsal ışına düşen yinelenen
  ucu bastırılır. Saat yönü ve başlangıç açısı ECharts varsayılanlarını
  koruduğu için bütün sektör başlangıç/bitiş açıları resmî rasterle aynı
  konumdadır.
- Açık legend `A`, `B`, `C` sırasını ve seri palet renklerini korur. Resmî
  karşılaştırma koşucusunun title/legend/toolbox için uyguladığı 15 px
  normalize padding fixture'a açıkça taşındı; gösterge 700×525 option
  görünümünün alt merkezinde aynı simge, metin aralığı ve katmanda çizilir.
  Kaynaktaki üç `emphasis.focus: 'series'` bildiriminin statik rastere
  etkisi yoktur; seri odaklı hover/blur yaşam döngüsü ortak etkileşim
  sistemiyle Faz 7'de kapanacağı için kanıt matrisi etkileşimi bilinçli
  olarak `kısmi` tutar.
- Çekirdek birim kapısı 283/283, fixture kapısı 40/40 geçti;
  `cargo check --all-targets`, `cargo check --no-default-features` ve
  `node tools/uyum/uret.mjs --check` temizdir. Yeni kart kilitli referansla
  bağımsız 1/1 tekrar koşusunu, tüm depo ise önceden kilitli hiçbir resmî
  referans yenilenmeden 177/177 görsel regresyonu geçti.
- Yeni 600×450 kilitli kanıt 769 değişen piksel, `%0,2848` fark ve
  `0,997743` SSIM üretir. Resmî referans geçici üretimlerde kararlı olduğu
  doğrulandıktan sonra yalnız bu yeni kart için bir kez oluşturuldu;
  izleyen bağımsız koşu referansa yazmadan geçti. Referans, gerçek, fark ve
  metrik dosyaları SHA-256 değerleriyle galeri manifestine bağlandı.
- Kart `yok`tan `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 151/332, yani `%45,5`
  oldu. Birden fazla bağımsız stack/grup, negatif/ters eksen bileşimleri,
  `roundCap`, `barMinAngle`, tooltip/seri odaklı hover, animasyon,
  erişilebilirlik ve ölçümlü performans Faz 3/4/6/7/8 kapılarında
  kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

Gerçekleşen dilim — `bar-polar-stack-radial` (2026-07-20):

- Resmî örnek
  `../echarts-examples/public/examples/ts/bar-polar-stack-radial.ts`
  dosyasından kayıpsız fixture'a taşındı. Kutupsal bar bant/stack yerleşimi
  `../echarts/src/layout/barPolar.ts`; veri yığını
  `../echarts/src/data/helper/dataStackHelper.ts` ile
  `../echarts/src/processor/dataStack.ts`; radyal eşleme ve polar
  varsayılanları `../echarts/src/coord/polar/RadiusAxis.ts` ile
  `PolarModel.ts`; görünüm katmanları
  `../echarts/src/component/axis/AngleAxisView.ts` ve `RadiusAxisView.ts`;
  seri katmanı da `../echarts/src/chart/bar/BaseBarSeries.ts` üzerinden
  sabit ECharts commitinde doğrulandı.
- `angleAxis.type: 'category'` ve `Mon..Sun` sırasındaki yedi kategori,
  boş `radiusAxis`/`polar`, resmî kaynağın açık bıraktığı animasyon ile
  üç polar barın ad, veri ve ortak `stack: 'a'` alanları tipli fixture'a
  aktarıldı. Varsayılan `%50/%50` merkez ve `%80` yarıçap 700×525 option
  görünümünde yine `(350, 262,5)` merkez ile 210 px dış yarıçap üretir.
- Yedi kategori bandı `360/7 = 51,4286°`dir. Açık `barWidth` ve
  `barCategoryGap` olmadığı için ECharts'ın `%20` boşluğu uygulanır;
  sektör açıklığı `41,1429°`, komşu sektörler arasındaki toplam boşluk
  `10,2857°` olur. Varsayılan `startAngle: 90`, `clockwise: true` ve
  `boundaryGap: true` ile kategori merkezleri ekran uzayında sırasıyla
  `−64,2857°`, `−12,8571°`, `38,5714°`, `90°`, `141,4286°`,
  `192,8571°` ve `244,2857°` konumlarına düşer.
- A verisi `[1,2,3,4,3,5,1]` için radyal taban/tepe `0→A`; B verisi
  `[2,4,6,1,3,2,1]` için `A→[3,6,9,5,6,7,2]`; C verisi
  `[1,2,3,4,1,2,5]` için `[3,6,9,5,6,7,2]→[4,8,12,9,7,9,7]` olarak
  hesaplanır. Böylece her günün A/B/C dilimleri aynı açısal bantta merkezden
  dışarı uç uca eklenir ve ortak radyal kapsam `0..12` olur.
- Boş radiusAxis'in `splitNumber: 5` güzel ölçeği 2 birim aralık seçerek
  `0,2,4,6,8,10,12` halkalarını üretir; her değer birimi 17,5 px yarıçapa
  karşılık gelir. Polar eksenlerin varsayılan `z=0` öğeleri bar serisinin
  resmî `z=2` katmanından önce boyanır; bu nedenle merkezdeki `0` etiketi
  ilk sektörün altında kalır. `axis.z > 2` verilirse splitLine, axisLine,
  axisTick ve axisLabel birlikte seri üstü geçişe alınır. Karışık `z=0/10`
  birim testi bu iki komut kümesini ayrı ayrı kilitler.
- Legend `A/B/C` sırasını ve resmî seri paletini korur; karşılaştırma
  koşucusunun ortak title/legend/toolbox normalizasyonu olan 15 px padding
  fixture'da açıktır. Üç serideki `emphasis.focus: 'series'` statik rasteri
  değiştirmez; seri odaklı hover/blur yaşam döngüsü Faz 7'nin ortak state
  sistemiyle tamamlanacağı için etkileşim kanıtı `kısmi` kalır.
- Çekirdek birim kapısı 284/284, fixture kapısı 41/41 geçti;
  `cargo check --all-targets`, `cargo check --no-default-features` ve
  `node tools/uyum/uret.mjs --check` temizdir. Yeni kart kilitli referansla
  bağımsız 1/1 tekrar koşusunu, depo ise önceden kilitli hiçbir resmî
  referans yenilenmeden 178/178 tam görsel regresyonu geçti.
- Yeni 600×450 kilitli kanıt 783 değişen piksel, `%0,2900` fark ve
  `0,997681` SSIM üretir. Resmî referans iki ardışık üretimde piksel
  düzeyinde aynı olduğu doğrulandıktan sonra yalnız bu kart için bir kez
  oluşturuldu ve izleyen koşuda referansa yazılmadan tekrar geçti;
  referans, gerçek, fark ve metrik hashleri galeri manifestine bağlıdır.
- Kart `yok`tan `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 152/332, yani `%45,8`
  oldu. Negatif yığınlar, ters/saat yönü kombinasyonları, birden fazla
  stack/grubun polar bant paylaşımı, `roundCap`, `barMinAngle`, tooltip ve
  seri odaklı hover, animasyon, erişilebilirlik ile ölçümlü performans ilgili
  Faz 3/4/6/7/8 kapılarında kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

Gerçekleşen dilim — `bar-polar-real-estate` (2026-07-20):

- Resmî örnek
  `../echarts-examples/public/examples/ts/bar-polar-real-estate.ts`
  dosyasından kayıpsız fixture'a taşındı. İki polar stack'in bant ve ofset
  hesabı `../echarts/src/layout/barPolar.ts`; yığın boyutları
  `../echarts/src/data/helper/dataStackHelper.ts` ile
  `../echarts/src/processor/dataStack.ts`; bar `z`, `coordinateSystem` ve
  genişlik varsayılanları `../echarts/src/chart/bar/BaseBarSeries.ts`;
  callback parametreleri `../echarts/src/model/mixin/dataFormat.ts`; item
  tooltip akışı `../echarts/src/component/tooltip/TooltipView.ts`; legend
  kutusu da `../echarts/src/component/legend/LegendModel.ts`,
  `LegendView.ts` ve `../echarts/src/util/layout.ts` üzerinden sabit ECharts
  commitinde doğrulandı.
- Başlıktaki `How expensive is it to rent an apartment in China?`,
  `Data from https://www.numbeo.com` alt metni, `grid.top: 100`, boş
  `radiusAxis`/`polar`, kategorik `angleAxis` ve resmî sıradaki 19 şehir
  (`北京`den `烟台`a) tipli modele aktarıldı. 19 adet
  `[lowest, highest, average]` üçlüsünün ondalıkları korunur; fixture testi
  hem ham minimum dizisini hem `highest-lowest`, `average-50` ve sabit 100
  dönüşümlerinin tamamını resmî veriyle karşılaştırır.
- Dört polar bar resmî option sırasını korur. İlk sessiz ve saydam seri
  `Min Max` yığınını `0→lowest` aralığına taşır; görünen `Range` serisi
  `lowest→highest` aralığını mavi çizer. Üçüncü sessiz/saydam ve `z: 10`
  seri `Average` yığınını `0→average-50` aralığına taşır; görünen
  `Average`, 100 birimlik verisiyle `average-50→average+50` aralığını yeşil
  çizer. Böylece veri yerleşimi için kullanılan görünmez sektörler legend
  ya da pointer hedefi olmadan gerçek ECharts stack semantiğini sürdürür.
- On dokuz kategorili bant `360/19 = 18,9474°`dir. İki bağımsız stack,
  varsayılan `barCategoryGap: '20%'` ve son serideki
  `barGap: '-100%'` ile `barPolar.ts` formülünde aynı `15,1579°` sektör
  açıklığını ve aynı `−7,5789°` ofseti alır; dolayısıyla yeşil ortalama
  şeridi mavi aralık sektörünün açısal merkezine tam bindirilir. Bu negatif
  yüzdelik değer `SütunSerisi::sütun_boşluğu`nda kayıpsız saklanır.
- 800×600 resmî `shotWidth` görünümünde polar merkez `(400, 300)`, dış
  yarıçap 240 px'dir. Ham kapsam `0..10000`, boş radiusAxis'in güzel ölçeği
  de 2.000 birimlik `0..10000` halkalarını üretir. Böylece 100 birimlik
  ortalama bandı 2,4 px radyal kalınlığa; örneğin Pekin aralığı
  `5000..10000`, ortalama şeridi `6735,71..6835,71` yarıçap değerlerine
  karşılık gelir. Varsayılan `startAngle: 90`, saat yönü ve kategori
  `boundaryGap` davranışı 19 sektör/etiketin resmî raster konumunu korur.
- `SütunSerisi` artık ECharts'ın bar öntanımlı `z: 2` değerini ve açık
  `silent` seçeneğini taşır. Polar seriler `(z, seriesIndex)` sırasıyla
  boyanır; `z: 10` ortalama yığını mavi aralığın üstündedir. Sessiz iki
  taban seri çizilir fakat isabet bölgesi üretmez. Programatik item seçimi
  de sessiz tabanı atlayıp görünür serinin ham `dataIndex` değerini
  `İpucuParametresi`ne geçirir.
- Resmî formatter aynı veri sırasını kullanarak
  `北京<br>Lowest：5000<br>Highest：10000<br>Average：6785.71` sonucunu
  üretir. Bağlamlı formatter'ın `<br>`, `<br/>` ve `<br />` ayrımları
  güvenli yerel tooltip satırlarına çevrilir; birim testinde başlık ile üç
  değer satırının ayrı çizildiği ve sessiz placeholder'ın hover hedefi
  olmadığı kilitlidir. Statik kart bu tooltip'i açık göstermediği için tam
  pointer/hover görsel yaşam döngüsü Faz 7 etkileşim kapısında kalır.
- `legend.top: 'bottom'` yalnız bu fixture için yaklaşık bir `bottom`
  uzaklığına çevrilmedi. Genel `Gösterge::üst` API'si sayısal/yüzdelik
  değerlerin yanında `top`, `middle/center` ve `bottom` anahtarlarını tipli
  `DikeyKonum` olarak korur. `top: 'bottom'` ve normalize 15 px padding,
  `layout.getLayoutRect` gibi 600 px yükseklikte 14 px legend içeriğini
  `y=571`e yerleştirir; `Range/Average` sırası ve renkleri ayrıca fixture
  ve yerleşim testleriyle sabittir.
- Çekirdek birim kapısı 287/287, fixture kapısı 42/42 geçti;
  `cargo check --all-targets`, `cargo check --no-default-features` ve
  `node tools/uyum/uret.mjs --check` temizdir. Yeni kart kilitli referansla
  bağımsız 1/1 tekrar koşusunu, depo ise önceden kilitli hiçbir resmî
  referans yenilenmeden 179/179 tam görsel regresyonu geçti.
- 800×600 kaynaklardan ortak 600×450 boyuta indirilen yeni kilitli kanıt
  1.239 değişen piksel, `%0,4589` fark ve `0,997249` SSIM üretir. Resmî
  referans iki ardışık üretimde piksel düzeyinde kararlı olduğu
  doğrulandıktan sonra yalnız bu kart için bir kez oluşturuldu; izleyen
  bağımsız ve tam koşular referansa yazmadan geçti. Referans, gerçek, fark
  ve metrik hashleri galeri manifestine bağlıdır.
- Kart `kısmi`den `uygulandı_kanıt_bekliyor` durumuna, statik görsel kapısı
  `tam_kanıtlı`ya geçti. Operasyonel kart ilerlemesi 153/332, yani `%46,1`
  oldu. `roundCap`, `barMinAngle`, negatif/ters polar birleşimleri, tooltip
  pointer/hover yaşam döngüsü, animasyon, erişilebilirlik ve ölçümlü
  performans ilgili Faz 3/4/7/8 kapılarında kapanmadan kart nihai
  `tam_kanıtlı` sayılmaz.

Çalışma kontrol noktası — `polar-roundCap` (tamamlanmadı, 2026-07-20):

- Operasyonel ilerleme 153/332 (`%46,1`) olarak kalır; bu kart için kilitli
  referans oluşturulmadı ve kart tamamlanmış sayılmadı.
- `e2d9bfc` commit'i, `../echarts/src/layout/barPolar.ts` içindeki bağımsız
  stack/grup bant hesabını ve `../echarts/src/util/shape/sausage.ts`
  kaynaklı `roundCap` yolunu çekirdeğe taşır. Sıfır değer iki yarım yaylı
  daireye, tam tur dikişsiz halkaya dönüşür; 289/289 kütüphane testi
  geçmiştir.
- Resmî `public/examples/ts/polar-roundCap.ts` option'ı fixture'a aktarıldı:
  `max: 2`, `startAngle: 30`, gizli splitLine, `v..z` radius kategorileri,
  `[4,3,2,1,0]` verili iki bağımsız seri, kırmızı/yeşil 1 px kenarlık,
  `opacity: 0.8`, yalnız ikinci seride `roundCap: true` ve iki legend adı.
  Fixture sözleşme testi geçer; üretici bu kartı `kısmi` olarak kaydeder.
- Geçici 700×525 renderlarda halka geometrisi, bant ofsetleri, sıfır-değer
  kapağı ve legend yerleşimi resmî kareyle görsel olarak hizalıdır. Kalan
  bilinen sapma angle value-axis tikleridir: resmî kare `0,2` aralıklı
  `0..1,8`, Cizelge ise `0,5` aralıklı `0..1,5` etiketleri çizer.
- Devamda önce `../echarts/src/component/polar/install.ts` içindeki angle-axis
  `splitNumber: 12` varsayılanı ile `../echarts/src/coord/axisNiceTicks.ts`
  aralık hesabı Cizelge'nin polar değer eksenine uygulanacak. Ardından geçici
  PNG'ler 600×450'ye indirilip `%1` pixelmatch / `0,99` SSIM kapısı geçilecek;
  ancak bundan sonra kart kanıt koşucusuna eklenecek ve yeni referans yalnız
  bir kez oluşturulacaktır. Son adımlar kilitli tekrar, tam görsel regresyon,
  tüm derleme/test kapıları, gerçekleşen dilim kaydı ve ilerlemeyi
  154/332 (`%46,4`) yapmaktır.

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
