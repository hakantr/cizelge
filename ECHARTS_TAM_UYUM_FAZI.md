# Cizelge — ECharts 6.1 Tam Uyum ve Görsel Galeri Faz Planı

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
| Bağımsız GPUI | `5566476024607a4c6999ab7b91d0218633a9b96c` | `../gpui` | `gpui` ve `gpui_platform` için tek derleme ve güncelleme kaynağı |
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

Sayılar yukarıdaki iki sabit commit görüntüsüne aittir ve envanter
aracı tarafından her yenilemede tekrar hesaplanmalıdır:

- Çekirdek katalog: **377** benzersiz kayıt.
- Resmi keşif sayfasına uygun (`noExplore != true`) çekirdek kayıt: **283**.
- Resmi alan adındaki `BLACK_MAP`, bunlardan `effectScatter-bmap`,
  `lines-airline` ve `scatter-world-population` kimliklerini ayrıca gizler;
  gerçek resmi sayfa bu snapshot'ta **280** çekirdek kayıt gösterir. Üçü de
  zaten Geo/Map kapsam dışı olduğundan kapsam içi sayı değişmez.
- Bunların Geo/Map bağımlı olanları: **22**.
- Resmi sayfada gösterilecek kapsam içi benzersiz örnek: **261**.
- `noExplore` olduğu için resmî keşif sayfasında gizlenen fakat kapsam içi
  ek conformance girdisi olarak kullanılacak örnek: **71**. Yerel uyum
  raporu bu kayıtları kesik çerçeveli “gizli doğrulama” kartları olarak
  ayrıca gösterir.
- Toplam kapsam içi çekirdek örnek kaydı ve yerel rapor kartı: **332**.
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

## 4. Repo durumu ve kanıtın yorumu

Bu plan tarihsel bir devir günlüğü tutmaz. Geçerli durum yalnız çalışma
ağacındaki model/render kaynakları ile yeniden üretilebilen
`uyum/galeri_manifest.json`, `uyum/ozellik_matrisi.json`,
`uyum/senaryolar/`, görsel metrikler ve test çıktılarından okunur. Eski örnek,
test veya dosya sayıları belge içinde “mevcut durum” diye dondurulmaz.

- [FAZ_PLANI.md](FAZ_PLANI.md), eski commit mesajları ya da önceki bir
  checkpointteki tamam işareti bu planın kanıt kapılarının yerine geçmez.
- Kayıt, SVG, piksel ve gerçek GPUI yüzeyleri aynı seçenek/model davranışını
  tüketir; yalnız bir renderer'da görünen destek tam uyum sayılmaz.
- `effectScatter`/`pictorialBar` gibi ortak modelle temsil edilen türler ve
  `matrix` gibi ayrı koordinatlar, resmi option/seri semantiğine göre
  özellik matrisinde ayrı ayrı kanıtlanır; Rust enum biçimi kapsamı daraltmaz.
- Sayısal ilerleme ve galeri adetleri elle yazılmış nottan değil, sabit kaynak
  commitleriyle çalışan üreticiden alınır. Üretilmiş dosya denetimi farklılık
  bulursa belge veya rapordaki sayı geçersiz sayılır.

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

`kaynak_kilidi.toml` ECharts, zrender, echarts-examples ve bağımsız GPUI
commit/sürümünü, snapshot tarihini ve kullanılan sabit font/veri görüntüsünü
tutar. GPUI için başka bir yerel çalışma alanı izlenmez.

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
- Eksen veya kategori taban çizgisi bulunan her kartta çizginin iki ucu,
  sürekliliği, rengi/kalınlığı ve seri dolgularına göre z-sırası ham karede
  ayrı yapısal kapıdır. Sütun ya da hücre dolgusunun taban çizgisini örttüğü
  tek piksellik hata, toplam fark oranı `%1` altında kalsa bile kartı düşürür.

Yoğun küçük yazı içeren ve ham geometri/renk kapıları geçen açık profillerde
renderer'a bağlı glif raster farkı ayrıca sınıflandırılabilir. Bu yol yalnız
manifestte örnek kimliğiyle tanımlanır ve şu değiştirilemez kurallara uyar:

- Referans ve Cizelge görüntüsünün **ikisine birden** aynı Gauss çekirdeği
  (`sigma = 0.8`, bir mantıksal pikselden küçük) uygulanır; pixelmatch
  eşiği `0.1`, `%1` piksel oranı ve `SSIM >= 0.99` değiştirilmez.
- Bu işlem maske değildir. Ham referans, ham gerçek, ham fark, normalize fark,
  ham/normalize metrikleri ve profil açıklaması birlikte saklanır; tümü rapor
  ve manifestte hash'lenir.
- Hücre dolguları, desen/decal katmanları, sınırlar, taban çizgileri ve temsilî
  renk örnekleri yalnız ham kare üzerinde denetlenir. Yapısal kapı geçmeden
  tipografi profili sonucu kanıt sayılamaz.
- Profil yalnız kimliği açıkça kayıtlı `matrix-mbti`,
  `matrix-periodic-table`, `parallel-aqi`, `parallel-nutrients`,
  `doc-example/parallel-all`, `tree-basic`, `tree-legend`,
  `tree-orient-bottom-top`, `tree-orient-right-left`, `tree-polyline`,
  `tree-radial` ve `tree-vertical` kartlarında açıktır; başka karta otomatik
  yayılmaz ve referans yenileme gerekçesi oluşturmaz.

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

Doğrulanmış kapsam — `bar-gradient`:

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

Doğrulanmış kapsam — `candlestick-large`:

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
  `tam_kanıtlı`ya geçti. Gerçek scheduler progressive parçalama, etkileşim,
  erişilebilirlik ve ölçümlü performans Faz 7/8 kapıları tamamlanmadan kart
  nihai `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `candlestick-sh`:

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
  `tam_kanıtlı`ya geçti. Animasyon, etkileşim, erişilebilirlik ve ölçümlü performans
  kapıları ilgili ileri fazlarda kapanmadan kart nihai `tam_kanıtlı`
  sayılmaz.

Doğrulanmış kapsam — `candlestick-brush`:

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
  `tam_kanıtlı`ya geçti. Serbest pointer ile alan çizme/dönüştürme, animasyon,
  erişilebilirlik ve ölçümlü performans kapıları ilgili ileri fazlarda
  kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `candlestick-sh-2015`:

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
  `tam_kanıtlı`ya geçti. Bu statik karede görünür sonucu değiştirmeyen
  `dataZoom.dataBackground`, `dataZoom.textStyle`, `brushSelect` ve
  `tooltip.axisPointer.lineStyle` seçeneklerinin genel API/etkileşim
  kapanışı Faz 5/6 matrisinde `kısmi` kalır; bunlar tamamlanmadan kart nihai
  `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `bar-brush`:

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
  `tam_kanıtlı`ya geçti. Pointer ile serbest alan oluşturma/dönüştürme, tüm brush olay yaşam
  döngüsü, animasyon, erişilebilirlik ve ölçümlü performans kapıları Faz
  5/6/7/8'de kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `bar-polar-label-radial`:

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
  `tam_kanıtlı`ya geçti. Teğetsel polar bar, çoklu stack/grup yerleşimi, `roundCap`,
  `barMinAngle`, polar tooltip/hover davranışının bütün kombinasyonları,
  animasyon, erişilebilirlik ve ölçümlü performans izleyen Faz 3/4/6/7/8
  kartlarıyla kapanmadan bu kart nihai `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `bar-polar-label-tangential`:

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
  `tam_kanıtlı`ya geçti. Birden fazla bağımsız polar stack/grubun bant paylaşımı,
  `roundCap`, `barMinAngle`, tüm negatif/ters/saat yönü kombinasyonları,
  tooltip/hover yaşam döngüsü, animasyon, erişilebilirlik ve ölçümlü
  performans izleyen Faz 3/4/6/7/8 kartları kapanmadan bu kart nihai
  `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `bar-polar-stack`:

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
  `tam_kanıtlı`ya geçti. Birden fazla bağımsız stack/grup, negatif/ters eksen bileşimleri,
  `roundCap`, `barMinAngle`, tooltip/seri odaklı hover, animasyon,
  erişilebilirlik ve ölçümlü performans Faz 3/4/6/7/8 kapılarında
  kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `bar-polar-stack-radial`:

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
  `tam_kanıtlı`ya geçti. Negatif yığınlar, ters/saat yönü kombinasyonları, birden fazla
  stack/grubun polar bant paylaşımı, `roundCap`, `barMinAngle`, tooltip ve
  seri odaklı hover, animasyon, erişilebilirlik ile ölçümlü performans ilgili
  Faz 3/4/6/7/8 kapılarında kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `bar-polar-real-estate`:

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
  `tam_kanıtlı`ya geçti. `roundCap`, `barMinAngle`, negatif/ters polar birleşimleri, tooltip
  pointer/hover yaşam döngüsü, animasyon, erişilebilirlik ve ölçümlü
  performans ilgili Faz 3/4/7/8 kapılarında kapanmadan kart nihai
  `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `polar-roundCap`:

- `e2d9bfc`, `../echarts/src/layout/barPolar.ts` içindeki bağımsız
  stack/grup bant hesabını ve `../echarts/src/util/shape/sausage.ts`
  kaynaklı `roundCap` yolunu çekirdeğe taşır. Aynı polar kategoride bağımsız
  seriler artık `barGap`/`barCategoryGap` ile ayrı bant alır; teğetsel
  yuvarlak uçlar Sausage geometrisini kullanır ve tam tur dikişsizdir.
- `3bfe393`, angle value-axis bağlamını tamamlar. `Eksen`, `splitNumber`ın
  açıkça belirtilip belirtilmediğini korur; belirtilmeyen angle axis,
  `component/polar/install.ts::angleAxisExtraOption` gibi 12 bölme kullanır,
  açık `splitNumber` ve `interval` ise ezilmez. Böylece resmî `max: 2`
  ekseni `0,2` adımlı `0..1,8` etiketlerini üretir.
- `BarView.updateStyle::isZeroOnPolar` davranışı taşındı: sıfır açıklıklı
  teğetsel bar, Sausage yordamı daire üretebilse bile boyanmaz. Seri/veri
  `itemStyle.opacity` değeri dolgu ile kenarlığa birlikte uygulanır.
  Sütun legend simgesi aynı opaklık, `borderColor` ve `borderWidth`
  görselini miras alır.
- `LegendView.getLegendStyle/layoutInner` ile `util/layout.ts::box` kaynakları
  doğrulanarak 1 px vuruşun iki yarım piksellik `Path.getBoundingRect`
  taşması satır yüksekliğine katıldı. Alt legend artık 14 px ham simge
  yerine 15 px görünen kutuyla yerleşir. Bu genel düzeltmenin önceden
  kalibre edilmiş `candlestick-brush` merkezini değiştirmemesi için yalnız
  onun piksel-yüzeyi padding telafisi `15,95`ten `14,95`e çekildi; kilitli
  metriği yeniden tam eski 1.390 piksel / `0,990133` SSIM sonucuna döndü.
- Resmî `public/examples/ts/polar-roundCap.ts` option'ı fixture'da kayıpsızdır:
  `max: 2`, `startAngle: 30`, gizli splitLine, `v..z` radius kategorileri,
  iki bağımsız `[4,3,2,1,0]` serisi, kırmızı/yeşil 1 px kenarlık,
  `opacity: 0.8`, yalnız ikinci seride `roundCap: true` ve iki legend adı.
- Geçici kapı geçtikten sonra resmî referans iki ardışık üretimde piksel
  düzeyinde kararlı bulundu ve yalnız bu yeni kart için bir kez oluşturuldu.
  Referansa yazmayan bağımsız tekrar 1/1; depo çapındaki kilitli regresyon
  180/180 geçti. 600×450 kanıt 1.470 değişen piksel, `%0,5444` fark ve
  `0,991958` SSIM üretir; referans, gerçek, fark ve metrik hashleri galeri
  manifestine bağlıdır.
- Çekirdek 292/292, fixture 43/43 geçti; `cargo check --all-targets`,
  `cargo check --no-default-features` ve `node tools/uyum/uret.mjs --check`
  temizdir. Kart `kısmi`den `uygulandı_kanıt_bekliyor` durumuna, statik
  görsel kapısı `tam_kanıtlı`ya geçti.
- Negatif/ters polar birleşimleri, `barMinAngle`, tooltip pointer/hover yaşam döngüsü,
  animasyon, erişilebilirlik ve ölçümlü performans ilgili Faz 3/4/7/8
  kapılarında kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `polar-endAngle` ve kanıt raporu:

- `../echarts/src/coord/polar/polarCreator.ts`, `AngleAxis.ts`,
  `Polar.ts`, `../echarts/src/component/axis/AngleAxisView.ts` ve
  `RadiusAxisView.ts` kaynakları izlenerek `angleAxis.endAngle` açık kapsamı
  taşındı. `startAngle - endAngle` imzalı ekran açıklığını belirler;
  `clockwise` ile `inverse` etkin yönü kurar ve `endAngle` yoksa aynı yönde
  tam tur üretilir. `boundaryGap: false` kategorik açı ekseninin son kapsamı
  resmî `360 / categoryCount` düzeltmesini uygular.
- Kısmi polar bölme çizgileri ve sıfır iç yarıçaplı angle axis artık tam
  çember yerine imzalı yaydır. Kategori etiketleri bant merkezlerine,
  ayrım ışınları bant sınırlarına düşer; kısmi kapsamın son sınırı ayrıca
  çizilir, tam turdaki çakışan son ışın yinelenmez.
- `GrafikSeçenekleri::kutupsallar`, `kutupsal_ekle`,
  `tüm_kutupsallar` ve `kutupsal_sayısı` tekil API'yi bozmadan ECharts
  `polar: []` dizisini taşır. Line/bar/scatter ve çekirdek `lines` modelleri
  `polarIndex` karşılığı `kutupsal_sırası` kazanmıştır. Doğrulama eksik
  indeksleri reddeder; çalışma zamanı yamaları tekil ve çoklu polar
  bileşenlerini atomik biçimde değiştirir.
- Boyama hattı her polar için ayrı görünürlük maskesi, yığın aralığı ve
  ölçek kurar. Bütün alt eksen katmanları serilerden önce, seri ve tooltip
  isabetleri kendi `polarIndex` grubunda, bütün üst eksen katmanları da
  serilerden sonra çizilir; örtüşen iki polar birbirinin verisini kapsamına
  veya yığınına katmaz.
- Resmî `public/examples/ts/polar-endAngle.ts` fixture'ı iki açık açı
  eksenini (`90→0`, `-90→-180`), `S1..S3`/`T1..T3` kategorilerini ve
  `[1,2,3]` verili iki bar serisinin `polarIndex: 0/1` bağlarını kayıpsız
  taşır. Yeni referans yalnız bu kart için bir kez oluşturuldu; referansa
  yazmayan bağımsız tekrar 1/1 ve tam depo koşusu 181/181 geçti. 600×450
  kanıt 1.122 değişen piksel, `%0,4156` fark ve `0,997493` SSIM üretir.
- Kullanıcının işaretlediği `dataset-encode0` taban çizgisi bar geometrisi
  kaydırılmadan, kategori `axisLine`ı barlardan sonra yeniden çizilerek
  korunur. Görsel kapı x=172 üzerindeki dokuz bar merkezini ayrı yapısal
  örnekler; 9/9 geçmeden toplam piksel/SSIM eşikleri tek başına kartı
  geçiremez. Bu katman değişimiyle tüm eski kartlar referans yenilenmeden
  yeniden doğrulanmıştır.
- Uyum raporu artık kapsam içindeki 332 kaydın tamamını kartlaştırır:
  261 resmî galeri kartı ve 71 işaretli gizli doğrulama kartı. Başlık
  `kilitli görsel kanıt` ile `tüm kapılar tam` sayılarını ayırır;
  uygulanmamış kartlar “Henüz uygulanmadı · Faz N” yazar. Makine-okunur tek
  ilerleme kaynağı `uyum/ozet.json`dır: bu dilim
  sonunda 134/332 (`%40,4`) kilitli statik görsel kanıt ve 0/332 tüm
  kapıları tamamlanmış kart vardır.
- Çekirdek 296/296, fixture 44/44, `cargo check --all-targets`,
  `cargo check --no-default-features` ve üretilmiş-dosya denetimi geçmiştir.
  Kart `uygulandı_kanıt_bekliyor`, statik görsel kapısı `tam_kanıtlı`dır;
  animasyon, etkileşim, erişilebilirlik ve performans kapıları kapanmadan
  nihai `tam_kanıtlı` sayılmaz.

Doğrulanmış kapsam — `bar-histogram`:

- Sabit kaynak `../echarts-examples/public/examples/ts/bar-histogram.ts`
  ile `../echarts-examples/package.json` içindeki `echarts-stat: ^1.2.0`
  bağı birlikte doğrulandı. Örnek başlığı tarihsel olarak “Custom Series”
  olsa da bu snapshot `renderItem` kullanmaz; aynı ham dataset'in 0. ve 1.
  boyutlarına uygulanan iki `ecStat:histogram` dönüşümünü normal scatter ve
  bar serilerine bağlar. Kart bu gerçek kaynak yapısına göre uygulanmış,
  paketin “BSD” beyanı ve yazarı `NOTICE` içinde kayda alınmıştır.
- `HistogramDönüşümü`, echarts-stat 1.2.0'ın `squareRoot`, `scott`,
  `freedmanDiaconis` ve `sturges` eşik yöntemlerini; tick-step/toFixed kutu
  sınırlarını ve `dimensions` seçimini taşır. Dönüşüm ilk sonuçta
  `MeanOfV0V1`, `VCount`, `V0`, `V1`, `DisplayableName`; ikinci sonuçta
  custom-series tüketicileri için `[alt, üst, adet]` üretir. Kök
  `VeriKümesiTanımı::histogram` ve `kaynaktan_histogram` yolları dönüşümü
  sibling dataset'lerde ECharts'ın varsayılan `fromDatasetIndex: 0`
  davranışıyla çalıştırır.
- Resmî 31 satırlık kaynakla ilk histogramın `8..22` aralığındaki
  `[3,11,6,3,5,2,1]`, ikinci histogramın `0..300` aralığındaki
  `[6,7,4,10,3,1]` adetleri ve her iki dönüşüm sonucu kayıpsız API
  testindedir. Dört eşik yöntemi için toplam örnek sayısının korunması da
  ayrı çekirdek testidir.
- Global seri paleti ECharts `PaletteMixin` gibi açık rengi olmayan seri
  adlarını renge eşler. Bu nedenle `origianl scatter` ilk palet rengini,
  aynı `histogram` adlı dikey ve yatay bar serileri birlikte ikinci palet
  rengini alır; açık renkli seri ve candlestick'in palet tüketmeme kuralları
  korunur.
- Fixture resmî üç grid yerleşimini, üç x + üç y eksenini, gizli kategori
  eksen görsellerini, `scale`, `barWidth: '99.3%'`, üst/sağ değer
  etiketlerini, datasetIndex/encode bağlarını ve tooltip bileşenini taşır.
  Yeni resmî referans yalnız bu kart için bir kez üretildi; referansa
  yazmayan tekrar 1/1, depo çapındaki kilitli koşu 182/182 geçti. 600×450
  kanıt 456 değişen piksel, `%0,1689` fark ve `0,998283` SSIM üretir.
- Çekirdek 299/299, fixture 45/45, `cargo check --all-targets`,
  `cargo check --no-default-features` ve üretilmiş-dosya denetimi geçmiştir.
  Rapor gerçekten 332 kart gösterir; makine-okunur güncel ilerleme
  135/332 (`%40,7`) kilitli statik görsel kanıt ve 0/332 tüm kapıları tam
  karttır. `bar-histogram` önizlemesi artık boş değildir; kart
  `uygulandı_kanıt_bekliyor`, statik görsel kapısı `tam_kanıtlı`dır.
- Tooltip hover içeriği, animasyon, klavye/ARIA, koyu profil ve ölçümlü
  performans kapıları kapanmadan kart nihai `tam_kanıtlı` sayılmaz.

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

Gauge doğrulanmış kapsamı:

- Resmî `gauge`, `gauge-simple`, `gauge-speed`, `gauge-progress`,
  `gauge-stage`, `gauge-grade`, `gauge-multi-title`, `gauge-temperature`,
  `gauge-ring`, `gauge-barometer`, `gauge-clock` ve `gauge-car` kaynakları
  ayrı fixture olarak bağlıdır. Davranış kaynağı
  `../echarts/src/chart/gauge/GaugeSeries.ts`, `GaugeView.ts`,
  `PointerPath.ts`, `util/shape/sausage` ve `util/symbol.ts` dosyalarıdır.
- Model, ECharts 6.1 öntanımlılarıyla birlikte `min/max`, `startAngle`,
  `endAngle`, `clockwise`, merkez/yarıçap, çoklu veri ve `colorBy: data`
  paletini taşır. Saat yönü zrender `normalizeArcAngles` kuralıyla
  normalleştirilir; tam çember ve ters yönlü yaylar ayrıca yapısal testlidir.
- `axisLine` için görünürlük, çoklu renk bantları, width, roundCap,
  opacity ve shadow; `splitLine`/`axisTick` için sayı, length, distance,
  width, `auto` renk ve shadow; `axisLabel` için formatter, inherit/auto
  renk, font stili ve sayısal/radial/tangential dönüş desteklenir.
- Pointer; yerleşik `PointerPath` veya tam `path://` SVG, length/width,
  offsetCenter, keepAspect, showAbove, itemStyle, auto renk ve veri-öğesi
  yamasını destekler. Anchor; show/showAbove, size, icon, offsetCenter,
  keepAspect, dolgu/kenarlık/gölge ve doğru katman sırasıyla boyanır.
- Progress; overlap, clip, roundCap, width, itemStyle, auto renk ve çoklu
  veride ECharts `z2` değer sırasını uygular. Örtüşmeyen halka düzeninde her
  veri öğesi kendi şerit kanalına yerleşir. Pointer/progress/itemStyle
  kalıtımı seri ve veri-öğesi katmanlarında yalnız açık alanları yamalar.
- `title` ve `detail` görünürlük, formatter, offsetCenter, renk mirası,
  background/border mirası, width/height/padding/lineHeight, çok satır ve
  rich koşularını destekler. Açık width hizalama kutusudur; yalnız
  `overflow: truncate` karşılığı açıkça seçilirse üç noktayla kısaltılır.
  `detail.precision`, final değeri değiştirmez ve yalnız valueAnimation ara
  karelerine uygulanır; seri ve veri-öğesi düzeyinde kalıtılır.
- Dinamik stage, multi-title, temperature, ring ve barometer örneklerinde
  resmî 2000 ms callback tam bir kez; clock örneğinde sabit Date ile 1000 ms
  callback tam bir kez çalıştırılır. Mulberry32 akışı ECharts'ın kaynak
  tüketim sırasını korur. Her yeni referans iki bağımsız üretimde bit düzeyi
  aynı olduğu doğrulandıktan sonra yalnız bir kez kilitlenir.
- `gauge-car`, yedi gauge serisini, siyah arka planı, toolbox'ı, birbirinin
  üstündeki kadranları, ters yönlü sıcaklık yayını, yakıt/sıcaklık SVG
  anchor'larını, özel ibreleri, Çince çok satırlı title ve rich detail
  panellerini aynı option bileşiminde doğrular. 800×600 kaynak çekimi resmî
  600×450 kanıt boyutuna küçültülerek ince yazı raster farkını dengeler;
  geometri ve metin içeriği ayrıca yapısal kapılardadır.

600×450 kilitli statik kanıt metrikleri:

| Örnek | Değişen piksel oranı | SSIM |
|---|---:|---:|
| `gauge` | %0,1478 | 0,998441 |
| `gauge-simple` | %0,1489 | 0,998431 |
| `gauge-speed` | %0,2204 | 0,996458 |
| `gauge-progress` | %0,1352 | 0,998724 |
| `gauge-stage` | %0,2219 | 0,998123 |
| `gauge-grade` | %0,1170 | 0,999286 |
| `gauge-multi-title` | %0,1756 | 0,998460 |
| `gauge-temperature` | %0,2181 | 0,996992 |
| `gauge-ring` | %0,1544 | 0,999158 |
| `gauge-barometer` | %0,4022 | 0,998194 |
| `gauge-clock` | %0,1852 | 0,999049 |
| `gauge-car` | %0,8907 | 0,994046 |

Tüm on iki Gauge kartının statik görsel kapısı `tam_kanıtlı`dır. Rapor her
zaman 332 gerçek kartı korur ve bu dilimle kilitli statik görsel kanıt sayısı
147'dir. Canlı valueAnimation zaman çizelgesi, hover/blur/select,
erişilebilirlik ve ölçümlü performans kapıları Faz 7/8 tamamlanana kadar ayrı
`kısmi` durumlar olarak kalır; statik kanıt bunları tamamlanmış saymaz.

Funnel doğrulanmış kapsamı:

- Resmî `funnel`, `funnel-align`, `funnel-customize` ve kaynak kimliğindeki
  yazımı korunmuş `funnel-mutiple` örneklerinin her biri
  `../echarts-examples/public/examples/ts/<id>.ts` kaynağından ayrı fixture'a
  bağlanmıştır. Model ve davranış için
  `../echarts/src/chart/funnel/FunnelSeries.ts`, `funnelLayout.ts` ve
  `FunnelView.ts` normatiftir.
- `HuniSerisi`, ECharts 6.1 öntanımlı `left: 80`, `top: 60`, `right: 80`,
  `bottom: 65`, `minSize: 0%`, `maxSize: 100%`, `sort: descending`,
  `orient: vertical`, `gap: 0`, `funnelAlign: center`, dış etiket,
  20 piksellik labelLine ve beyaz bir piksellik kenarı taşır. Piksel/yüzde
  kutu ölçüleri, açık veya veri kapsamından türetilen `min/max`, dikey/yatay
  yön, sol/orta/sağ ve üst/orta/alt hiza, artan/azalan/sırasız akış,
  öğe-başına width/height, `z`, `silent`, dataset index/layout ve
  `encode(name, value)` Rust API'sinde ayrıdır.
- Yerleşim, resmî sıralanmış ham indeksleri ve legend süzmesini korur;
  değerleri sıkıştırmalı doğrusal eşlemeyle min/max size aralığına taşır.
  Artan akışta başlangıç alt/sağ kenara alınır, dilim boyu ve gap işareti
  ters çevrilir. Son dilimin bulunmayan devam değeri ECharts gibi sıfır
  sayıldığı için `minSize: 0%` huniyi gerçek bir uca kapatır. Negatif kutu
  konumları kırpılmadan, genişlik/yükseklik ise güvenli biçimde çözülür.
- Normal ve veri-öğesi `itemStyle`, `label`, `labelLine` ile
  `emphasis`/`blur`/`select` yamaları model miras zincirindedir; statik
  boyama emphasis ve select yamalarını uygular. İç, merkez,
  iç-sol/iç-sağ, dış, sol/sağ, üst/alt ve dört dış köşe etiketi; rich text,
  formatter `{a}/{b}/{c}/{d}`, dönüş, kayma, dolguya göre otomatik iç yazı
  kontrastı, çizgi rengi ve opaklığı desteklenir. Seçili öğenin açıkça
  değiştirilmemiş kenarı etkin temanın `primary` rengine döner; hover
  vurgusu tooltip'in açık olmasına bağlı değildir.
- Hover ve tıklama, yamuğun sınır kutusunu değil gerçek dört köşeli çokgeni
  sınar. Böylece dar uçların çevresindeki boş pikseller olay üretmez;
  `silent` seri isabet kaydı üretmez. İlk oluşturma animasyonu ECharts
  `FunnelPiece` gibi geometriyi daraltmak yerine bütün polygon, kenar,
  label ve labelLine opaklığını birlikte geçirir.
- `funnel-align`, dört ayrı görünüm kutusunda artan/azalan ve sol/sağ
  hizalamayı; `funnel-mutiple`, dört kutuda dış/sol etiket ve piramit
  yönünü; `funnel-customize`, üst üste Expected/Actual serilerini,
  `maxSize: 80%`, seri opaklıklarını, iki piksellik kenarı ve normal/vurgu
  formatter'larını; `funnel` ise öntanımlı tek seri, legend, tooltip ve
  toolbox bileşimini kanıtlar. Dataset name/value aktarımı ve öğe yaması
  indekslerinin veri çözümünden sonra korunması ayrıca çekirdek testidir.
- Her yeni referans iki bağımsız ECharts üretiminde bit düzeyinde aynı
  çıktı verdiği doğrulandıktan sonra yalnız bir kez kilitlenmiştir.
  `funnel` resmî 700×525 çalışma alanında; ince uzun kenarlarda raster
  örnekleme gürültüsünü azaltmak için `funnel-customize` ve
  `funnel-mutiple` 1000×750, `funnel-align` 1400×1050 kaynak alanında
  üretilip resmî 600×450 kanıt boyutuna indirilir. Bütün ölçüler her iki
  renderer'da aynı kaynak alanına göre yeniden çözülür; çözünürlük seçimi
  geometri feragati veya maske değildir.

600×450 kilitli Funnel statik kanıt metrikleri:

| Örnek | Değişen piksel oranı | SSIM |
|---|---:|---:|
| `funnel` | %0,4459 | 0,991139 |
| `funnel-align` | %0,2804 | 0,996296 |
| `funnel-customize` | %0,5026 | 0,992059 |
| `funnel-mutiple` | %0,5093 | 0,991896 |

Dört Funnel kartının statik görsel kapısı `tam_kanıtlı`dır; böylece rapordaki
kilitli statik kanıt sayısı 151 olur. Çekirdek 311/311 ve uyum fixture
50/50 testleri; `cargo check --all-targets`, no-default derleme ve üretilmiş
dosya denetimi geçmiştir. Kilitli depo görsel koşusu, `dataset-encode0`
taban çizgisi yapısal kontrolü dâhil 198/198 kareyi yeniden doğrulamıştır.
Eski golden'lar yenilenmemiştir; `cargo test --all-targets` önceden kayıtlı
34 `testler/altin.rs` uyuşmazlığında kalır. Blur/focus eylemleri,
erişilebilirlik, koyu profil ve ölçümlü performans kapıları kapanana kadar
kartların genel durumu `uygulandı_kanıt_bekliyor` kalır; statik kanıt bu
kapıları tamamlanmış saymaz.

Radar doğrulanmış kapsamı:

- Resmî `radar`, `radar-aqi`, `radar-custom`, `radar2` ve
  `radar-multiple` kaynaklarının tamamı
  `../echarts-examples/public/examples/ts/<id>.ts` dosyasından ayrı Rust
  fixture'ına bağlanmıştır. Model ve davranış için
  `../echarts/src/coord/radar/RadarModel.ts`, `Radar.ts`,
  `../echarts/src/component/radar/RadarView.ts` ile
  `../echarts/src/chart/radar/RadarSeries.ts`, `RadarView.ts` ve
  `radarLayout.ts` normatiftir.
- `RadarKoordinatı`, ECharts 6.1 öntanımlı merkez, iç/dış yarıçap,
  `startAngle: 90`, `clockwise: false`, beş bölme, polygon şekli ve
  `scale: false` davranışını taşır. Çoklu radar dizisi ve `radarIndex` bağı;
  gösterge başına ad, açık/otomatik min-max ve renk; circle/polygon;
  `axisName` görünürlük/formatter/gap/yazı stili; `axisLine`, `splitLine`,
  `splitArea` görünürlük, renk döngüsü, çizgi stili, opaklık ve gölge;
  `silent` ile `z` ayrı model alanlarıdır.
- Gösterge ölçekleri, `axisAlignTicks.scaleCalcAlign` akışındaki güzel adım
  büyütme sırasını izler. Yalnız pozitif `max` için örtük sıfır min,
  yalnız negatif `min` için örtük sıfır max, açık iki uç, veri kapsamından
  otomatik uçlar ve her kolda aynı `splitNumber` halka yapısal testlidir.
  Açı dönüşümü ECharts'ın yukarı yönlü koordinatını ekran Y eksenine çevirir;
  `clockwise: false` gösterge sırasını saat yönünün tersinde ilerletir.
- `RadarSerisi`; çoklu veri öğesi, `radarIndex`, `colorBy: data` palet
  ilerlemesi, sembol/tür/boyut, çizgi/alan/öğe/etiket stili, veri-öğesi
  yamaları, `emphasis`/`blur`/`select`, `z` ve `silent` seçeneklerini taşır.
  Düz ve radyal gradyan alanlar, kesikli çizgi, rect/circle/path semboller,
  formatter ve rich etiketler ortak boyama hattında çözülür. Hover gerçek
  radar polygonunu sınar; sessiz seri veya koordinat isabet kaydı üretmez.
- Bir radar ağı, kendisine bağlı bütün serilerden kapsam çıkarılarak yalnız
  bir kez ve serilerin altında çizilir. Split alanları gerçek halka olarak
  boyanır; dairesel gölge maskesi dolu disk değil zrender `Ring` deliğini
  koruyan annulus'tur. Katman sırası polyline, polygon ve sembol düzenini;
  legend rengi seri/veri `itemStyle`ı ve `visualMap` sonucunu izler.
- `radar-aqi`, `selectedMode: single` ilk seçimini ve `symbol: none`
  serilerde kapalı legend simgesinin boyanmamasını; `radar2`, 28 serilik
  sürekli visualMap'i ve scroll legend'in kısmi son öğeyi sonraki sayfada
  tekrar başlatan kırpma/sayfalama davranışını; `radar-custom`, iki radar,
  otomatik kapsam, renkli halka gölgesi, veri-öğesi areaStyle, rect sembol,
  kesikli çizgi, değer etiketi ve radyal gradyanı; `radar-multiple`, üç
  bağımsız merkez/yarıçap ve beş legend sağlayıcısını aynı option
  bileşiminde doğrular.
- Beş yeni referansın her biri, 700×525 resmî çalışma alanında iki bağımsız
  üretimin bit düzeyinde aynı olduğu doğrulandıktan sonra yalnız bir kez
  kilitlenmiştir. Her iki renderer'ın ham çıktısı aynı
  `sharp.resize(600, 450)` adımından geçer; maske, eşik gevşetmesi veya
  geometri feragati kullanılmaz.

600×450 kilitli Radar statik kanıt metrikleri:

| Örnek | Değişen piksel oranı | SSIM |
|---|---:|---:|
| `radar` | %0,2593 | 0,998295 |
| `radar-aqi` | %0,4056 | 0,997823 |
| `radar-custom` | %0,4863 | 0,996951 |
| `radar2` | %0,4804 | 0,997508 |
| `radar-multiple` | %0,2956 | 0,997976 |

Beş Radar kartının statik görsel kapısı `tam_kanıtlı`dır. Eski golden'lar
yenilenmez; animasyon, programatik hover/blur/select, erişilebilirlik, koyu
profil ve ölçümlü performans kapıları Faz 7/8/9 tamamlanana kadar kartların
genel durumunu `uygulandı_kanıt_bekliyor` olarak tutar.

Parallel doğrulanmış kapsamı:

- Resmî `parallel-simple`, `parallel-aqi`, `parallel-nutrients` ve keşif
  sayfasında `noExplore` ile gizlenen `doc-example/parallel-all` kaynakları
  ayrı Rust fixture'larına bağlıdır. Seri davranışı için
  `../echarts/src/chart/parallel/ParallelSeries.ts`, `ParallelView.ts` ve
  `parallelVisual.ts`; bileşen/eksen görünümü için
  `../echarts/src/component/parallel/ParallelView.ts`; koordinat sistemi için
  `../echarts/src/coord/parallel/ParallelModel.ts`, `Parallel.ts`,
  `ParallelAxis.ts`, `AxisModel.ts`, `parallelCreator.ts` ve
  `parallelPreprocessor.ts` normatiftir.
- `ParalelKoordinatı`; piksel/yüzde `left/right/top/bottom/width/height`,
  yatay/dikey yerleşim, `parallelAxisDefault`, çoklu `parallelIndex`,
  genişletilebilir eksen penceresi, merkez/slide/jump davranışı,
  genişletme oranı/gecikmesi ve `click`/`mousemove` tetikleyicisini taşır.
  Alanı açıkça verilmeyen seri için resmî ön işlemci gibi örtük koordinat ve
  eksenler üretilir; aynı seçeneklerde birden fazla bağımsız Parallel kutusu
  seri bağıyla ayrılır.
- `ParalelEkseni`; bir veya birden fazla veri boyutu, value/category/time/log
  ölçeği, kategori sırası, min/max/scale/inverse, ad/etiket/çentik/bölme
  stilleri, ad kısaltması, `realtime`, z katmanı ve çok aralıklı
  `areaSelectStyle` seçimini uygular. `parallelAxisDefault`, yalnız eksende
  açıkça verilmemiş alanları tamamlar; eksen çizgileri seri polyline'larının
  üstünde boyanır.
- `VeriDeğeri::KarmaDizi`, aynı satırdaki sayı, kategori, zaman, mantıksal ve
  boş hücreleri kayıpsız taşır. Dataset/encode boyutları ile doğrudan karma
  satırlar aynı koordinat yolunda çözülür; boş değer yalnız ilgili eksen
  parçasını keser. Seri `smooth`, `lineStyle`, item style, normal/emphasis/
  select/inactive durumları, label, silent, clip animasyonu, progressive
  eşiği, legend ve sayısal/kategorik visualMap kanallarını uygular.
- Smooth çizgi, zrender `graphic/helper/smoothBezier` ve `Polyline.buildPath`
  kontrol noktalarının doğrudan portudur; Kartezyen Line eğrisiyle
  karıştırılmaz. Vuruş butt cap, miter join ve limit 10 ile çizilir. Gerçek
  polyline geometrisi isabet/tooltip hedefidir; tooltip bütün eksen adlarını
  ve karma değerleri korur.
- Eksen seçim sürüklemesi `axisAreaSelect` eylemini, son durum
  `axisAreaSelected` olayını üretir; bütün eksen aralıklarının kesişimi
  active/inactive veri durumunu belirler. Genişletme etkileşimi
  `parallelAxisExpand` ile tipli çalışma zamanı olayına çevrilir. `setOption`
  birleştirme/değiştirme yolu `parallel` ve `parallelAxis` köklerini diğer
  bileşenlerle aynı kimlik/index kurallarıyla işler.
- `parallel-nutrients`, `../echarts-examples` içindeki 7.637×17 resmî JSON'u,
  ilk görülme sırasındaki 25 grubu, 15 ekseni ve resmî HSL paletini eksiksiz
  okur. Pinned ECharts sahnesindeki 7.637 Polyline'ın 229.110 koordinatı,
  çizgi sırası, 25 RGB rengi ve width/opacity/smooth değerleri 0,001 mantıksal
  piksel çözünürlüklü kanonik FNV-1a özetiyle birebir eşleşir:
  `d3f9efb47fd5e2d7`.
- Toplam fark oranının ince eksenleri saklamaması için her Parallel kartında
  her eksen üç ayrı noktada ham kareden denetlenir. Nutrients'in yoğun renkli
  eğrileri en yüksek kontrastla eksen sanılmaz; açık `#aaa` axisLine için
  nötr renk hedefi kullanılır. Böylece simple 12/12, AQI 24/24, gizli all
  24/24 ve nutrients 45/45 eksen örneği geçmeden kanıt başarılı sayılamaz.

600×450 kilitli Parallel statik kanıt metrikleri:

| Örnek | Değişen piksel oranı | SSIM | Ham oran / SSIM | Yapısal kapı | Durum |
|---|---:|---:|---:|---:|---|
| `parallel-simple` | %0,3093 | 0,995626 | aynı | 12/12 | geçti |
| `parallel-aqi` | %0,7581 | 0,991426 | %1,4215 / 0,986990 | 24/24 | geçti |
| `doc-example/parallel-all` | %0,2985 | 0,993425 | %0,6767 / 0,989702 | 24/24 | geçti |
| `parallel-nutrients` | %5,0770 | 0,948540 | %5,3037 / 0,946689 | 45/45 + sahne özeti | kanıt bekliyor |

Üç Parallel kartının statik görsel kapısı `tam_kanıtlı`dır ve depo özeti
`174/332` kilitli statik kanıta yükselmiştir. `parallel-nutrients` veri,
geometri, renk, stil ve eksen kapılarında geçmesine rağmen Chrome Canvas ile
tiny-skia'nın 0,5 px, %5 opak, yoğun kübik eğrileri rasterleştirme farkı genel
`%1 / 0,99` kapısını aşar. Eşik, maske, renk veya opaklık telafisi uygulanmaz;
ham referans/gerçek/fark ile normalize metrikler raporda tutulur ve kart
`kısmi` kalır. Kullanıcının son görsel incelemesi ya da renderer düzeltmesi
olmadan bu kart `tam_kanıtlı` sayılmaz. Koyu profil, klavye/ARIA, bütün
programatik durum geçişleri ve ölçümlü progressive performans kapıları da
Faz 7/8/9'da kapanacaktır.

ThemeRiver doğrulanmış kapsamı:

- Resmî `themeRiver-basic` ve `themeRiver-lastfm` kaynakları
  `../echarts-examples/public/examples/ts/<id>.ts` dosyalarından kayıpsız
  Rust fixture'larına bağlanmıştır. Model ve yerleşim için
  `../echarts/src/chart/themeRiver/ThemeRiverSeries.ts`,
  `themeRiverLayout.ts`, `ThemeRiverView.ts` ile
  `../echarts/src/coord/single/Single.ts` normatiftir.
- `TemaNehriSerisi`; `singleAxisIndex`, iki uçlu yüzde/piksel
  `boundaryGap`, katman renk paleti, normal ve emphasis `itemStyle`, normal
  ve emphasis label, `label.margin` ve `silent` yüzeylerini taşır.
  `GrafikSeçenekleri::doğrula`, bulunmayan `singleAxisIndex` bağını sessizce
  yok saymaz ve tipli `EksikVeri { bileşen: "singleAxis" }` hatası verir.
- Resmî `fixData` akışı gibi katman adları ilk görülme sırasıyla gruplanır;
  bütün tek-eksen değerlerinin birleşimi çıkarılır ve bir katmanda eksik
  kalan değerler sıfırla tamamlanır. Adı `undefined` olan Last.fm satırları
  veri modeline alınmaz; resmî kaynakta 24 ham satırdan yalnız ad taşıyan
  20 katman ve bunların 400 noktası çizilir.
- Yerleşim gerçek `singleAxis` kutusunu ve veri ölçeğini paylaşır. Her
  kesitte katman toplamları çıkarılır, en büyük toplamdan ECharts silhouette
  tabanı `(max - sum) / 2` hesaplanır ve dik yöndeki `boundaryGap`
  düşüldükten sonra tek bir `ky` ile ölçeklenir. Her iki kenar zrender
  `ECPolygon` ile aynı `smooth: 0.4` kübik kontrol noktalarıyla çizilir;
  dolgular arasında boşluk veya örtüşme bırakılmaz.
- Katman paleti `itemStyle.color` → `series.color` → global palet
  önceliğini izler. Opaklık, kenarlık ve gölge; hover/programatik emphasis;
  gerçek bant çokgeniyle isabet; animasyon kırpması; legend katman adları,
  renkleri ve seçili-gizli yeniden yerleşimi uygulanır. Varsayılan 11 px
  katman etiketi ilk noktanın dört piksel soluna, koyu `#333` dolgu ve iki
  piksellik zemin konturuyla bütün bantların üstünde çizilir.
- `themeRiver-basic` fixture'ı altı katman × 21 tarihi, legend ve zaman
  singleAxis'ini taşır. Resmî referans tarayıcısının `Europe/Istanbul`
  saat dilimindeki 8–9 Kasım 2015 yaz saati geçişi ilk aralığı 25 saat
  yaptığından fixture aynı mutlak aralığı korur; iki günlük ECharts zaman
  çentikleri bu kapsam için açık bölme sayısıyla eşlenir. `themeRiver-lastfm`
  `max: dataMax`, etiketsiz 20 katman ve sayısal singleAxis yolunu doğrular.
- İki referans da 700×525 çalışma alanında iki bağımsız resmî üretimin bit
  düzeyinde aynı olduğu görüldükten sonra kilitlenmiş, her iki renderer aynı
  `sharp.resize(600, 450)` adımından geçirilmiştir. Maske veya eşik
  gevşetmesi yoktur. Genel pixelmatch/SSIM kapısına ek olarak her iki kartta
  singleAxis taban çizgisi genişlik boyunca ayrı örneklenir;
  `themeRiver-basic` güçlü bir kesitte altı katmanın sıra ve rengini altı
  bağımsız piksel örneğiyle ayrıca kilitler.

600×450 kilitli ThemeRiver statik kanıt metrikleri:

| Örnek | Değişen piksel oranı | SSIM | Yapısal kapı |
|---|---:|---:|---:|
| `themeRiver-basic` | %0,1819 | 0,998507 | 2/2 |
| `themeRiver-lastfm` | %0,0485 | 0,999297 | 1/1 |

İki ThemeRiver kartının statik görsel kapısı `tam_kanıtlı`dır. Galeri 332
gerçek kartı ve kanıtsız kartlarda açık yer tutucuyu korurken 158 kartta
sıfır baytlı veya eksik olmayan gerçek önizleme gösterir. Çekirdek ve uyum
fixture testleri, `cargo check --all-targets`, no-default PNG derlemesi ve
üretilmiş dosya denetimi geçmiştir. Kilitli depo koşusu,
`dataset-encode0` kategori taban çizgisi ile ThemeRiver taban/siluet
kontrolleri dâhil 205/205 kareyi doğrular. Eski golden'lar yenilenmez;
animasyonun ara kareleri, erişilebilirlik, koyu profil ve ölçümlü performans
kapıları Faz 7/8/9 tamamlanana kadar kartların genel durumunu
`uygulandı_kanıt_bekliyor` olarak tutar.

Calendar doğrulanmış kapsamı:

- Resmî `calendar-simple`, `calendar-heatmap`, `calendar-vertical`,
  `calendar-horizontal`, `calendar-graph`, `calendar-lunar`, `calendar-pie`,
  `custom-calendar-icon` ve `calendar-charts` kategori kartları ile çapraz
  `calendar-effectscatter` örneği ayrı Rust fixture'larına bağlıdır. Option,
  koordinat ve görünüm davranışı için
  `../echarts/src/coord/calendar/CalendarModel.ts`, `Calendar.ts` ve
  `../echarts/src/component/calendar/CalendarView.ts`; bağlı seri dalları
  için ilgili Heatmap, Scatter, EffectScatter, Graph, Lines, Pie ve Custom
  kaynakları normatiftir.
- `TakvimKoordinatı`; ECharts 6.1 öntanımlı `left: 80`, `top: 60`,
  `cellSize: 20`, yatay yön, Pazar başlangıcı, görünür gün/ay/yıl
  etiketleri, görünür ay ayırıcısı ve bir piksellik gün hücresi kenarını
  taşır. Piksel/yüzde sol-sağ-üst-alt ve width/height, tek veya iki boyutlu
  sayısal/otomatik hücre boyutu, yatay/dikey yön, birden fazla takvim ve
  bütün bağlı serilerde `calendarIndex` doğrulanır.
- `TakvimAralığı`, tam yıl ve kapsayıcı iki Unix-ms ucu kabul eder; ay ve
  keyfî gün aralığı aynı uçlarla kayıpsız ifade edilir. Resmî
  `_initRangeOption` gibi ters verilen iki uç kronolojik sıraya çevrilir;
  sonlu olmayan değerler ve aralık dışındaki veri koordinat dönüşümünde
  tipli hata/yok sonucu üretir. `dataToPoint`, `pointToData`, yatay/dikey
  hafta hesabı ve ilk gün kayması tersinir testlerle kilitlidir.
- `dayLabel` için show, `firstDay`, start/end position, hücre boyutuna göre
  çözülen sayı/yüzde margin, açık `nameMap` dizisi ve etkin yerel;
  `monthLabel` için show, start/end, left/center hizası, margin, `nameMap`
  ve formatter; `yearLabel` için show, otomatik/top/bottom/left/right,
  margin ve formatter uygulanır. Ay şablonları `{nameMap}`, `{yyyy}`,
  `{yy}`, `{MM}`, `{M}`; yıl şablonları `{nameMap}`, `{start}`, `{end}`
  alanlarını çözer. İşlev biçimindeki formatter'lar da aynı resmî bağlamın
  bütün alanlarını klonlanabilir tipli Rust parametreleriyle alır.
- `splitLine.show` ayırıcıları stil kalınlığını sıfıra düşürmeye gerek
  bırakmadan bütünüyle kapatır; lineStyle renk, opaklık, kalınlık ve türünü
  korur. Calendar itemStyle hücre dolgu/kenarlığını belirler; heatmap
  `contentRect`, resmî `dataToLayout` gibi bu kenarlığın yarısı kadar içeri
  alınır. Katman sırası hücre zemini → seri → ayırıcı/yıl/ay/gün etiketidir;
  bu nedenle seri hücreleri ay sınırlarını örtemez.
- Heatmap, scatter/effectScatter, graph, custom, Geo dışı lines ve takvim
  merkezli pie bağları aynı `TakvimYerleşimi` dönüşümünü paylaşır. Çoklu
  takvim, çoklu seri, görsel eşleme, tooltip/isabet, legend, custom ikon,
  ay takvimi, ay evreleri ve birleşik pie/scatter/graph çizimleri resmî
  örnek bileşimlerinde ayrı ayrı çalışır. `calendar-effectscatter` ile
  `calendar-charts` beşer ara kare üzerinden gerçek animasyon senaryosudur.
- Genel pixelmatch/SSIM kapısına ek olarak `calendar-simple` üst ve alt
  dış sınırı yedişer nokta boyunca, ay ayırıcı merdivenleri ise yedi bağımsız
  semantik pikselde denetlenir. Böylece ince sınırın tamamen seri altında
  kalması toplam piksel oranı küçük olsa bile kanıtı düşürür. Referanslar
  yenilenmemiş; mevcut ECharts commit kilidiyle Cizelge çıktıları yeniden
  üretilmiştir.

600×450 kilitli Calendar kanıt metrikleri:

| Örnek | Kare | En yüksek değişen piksel oranı | En düşük SSIM | Yapısal kapı |
|---|---:|---:|---:|---:|
| `calendar-simple` | 1 | %0,0511 | 0,999505 | 3/3 |
| `calendar-heatmap` | 1 | %0,1359 | 0,999317 | — |
| `calendar-vertical` | 1 | %0,1893 | 0,998345 | — |
| `calendar-horizontal` | 1 | %0,2196 | 0,998160 | — |
| `calendar-effectscatter` | 5 | %0,5781 | 0,997125 | — |
| `calendar-graph` | 1 | %0,1133 | 0,998996 | — |
| `calendar-lunar` | 1 | %0,7578 | 0,991538 | — |
| `calendar-pie` | 1 | %0,6074 | 0,996919 | — |
| `custom-calendar-icon` | 1 | %0,1126 | 0,998767 | — |
| `calendar-charts` | 5 | %0,0204 | 0,999630 | — |

On Calendar senaryosunun 18/18 karesi geçer; dokuz Calendar kategori
kartının ve çapraz effectScatter kartının statik görsel kapısı
`tam_kanıtlı`dır. Option matrisindeki dokuz `calendar.*` satırı gerçek
`src/model/takvim.rs` API'lerine, testlere, veri biçimlerine, koordinat
dallarına ve galeri örneklerine bağlanmıştır. Çekirdek 319/319 ve uyum
fixture 54/54 testi, `cargo check --all-targets` ve üretilmiş-dosya
denetimi geçer. Depo çapındaki kilitli koşu 205/205 kareyi; kategori taban
çizgisi, ThemeRiver ve Calendar kontrollerinden oluşan 7/7 yapısal kapıyı
birlikte doğrular. Rapor 332 galeri kartını ve 205 görsel kanıt karesini
eksiksiz üretir. Varsayılan özellikli çekirdek testlerin 322/322'si geçer;
eski golden dosyaları yenilenmediği için ayrı `testler/altin.rs` kapısında
14/48 geçiş ve güncel davranışla uyuşmayan 34 kayıtlı snapshot kalır. Ortak
etiket stilinin kalan çapraz seçenekleri,
programatik hover/blur/select olay yükleri,
erişilebilirlik, koyu profil ve ölçümlü performans kapıları Faz 7/8/9'da
kapanana kadar genel kart ve option durumu
`uygulandı_kanıt_bekliyor` olarak kalır; statik kanıt bunları tamamlanmış
saymaz.

Matrix doğrulanmış kapsamı:

- Resmî `matrix-simple`, `matrix-correlation-heatmap`,
  `matrix-correlation-scatter`, `matrix-covariance`, `matrix-graph`,
  `matrix-pie`, `matrix-confusion`, `matrix-grid-layout`, `matrix-stock`,
  `matrix-sparkline`, `matrix-periodic-table`, `matrix-mbti` ve keşif
  sayfasında `noExplore` ile gizlenen `matrix-mini-bar-data-collection`
  kaynakları ayrı Rust fixture'larına bağlıdır. Görünür kategori sayısı 12,
  gizli conformance girdisi 1'dir. Yalnız Geo gömen
  `matrix-mini-bar-geo`, değiştirilemez kapsam kuralıyla dışarıdadır.
- Option ve koordinat davranışı için
  `../echarts/src/coord/matrix/MatrixModel.ts`, `Matrix.ts`, `MatrixDim.ts`,
  `MatrixBodyCorner.ts` ve `matrixHelper.ts`; görünüm/etkileşim için
  `../echarts/src/component/matrix/MatrixView.ts` normatiftir. Bağlı Heatmap,
  Scatter, Graph, Pie, Line/Bar, Candlestick, Custom ve çekirdek Lines
  kaynakları kendi seri dalları için ayrıca izlenir.
- `MatrisKoordinatı`, body/corner ile x/y başlıklarını; piksel/yüzde/dinamik
  kutu yerleşimini; `data` veya `length` ile ordinal boyutu; hiyerarşik
  `children`; yaprak `size`; varsayılan ve seviye başına `levelSize`;
  `show`; `dividerLineStyle`; arka plan/dış sınır ve `borderZ2` katmanını
  taşır. Yapraklarla karşı başlık seviyeleri resmî
  `layOutUnitsOnDimension` sırasıyla aynı fiziksel alanı paylaşır; dengesiz
  ağaçtaki sığ yaprak kalan başlık seviyelerini kaplar.
- Gövde ve köşe hücreleri index, ordinal değer, kapsayıcı aralık ve tüm-boyut
  koordinatlarını kabul eder. Negatif `MatrixXYLocator` başlık/köşe alanını,
  `coordClamp` geçersiz ucu sınıra kıstırmayı, `mergeCells` hem boyamayı hem
  `dataToLayout/dataToPoint` sonucunu genişletmeyi sağlar. Hücre, üst modelden
  `itemStyle`, `label`, `formatter`, `cursor`, `silent` ve `z2` miras alır;
  açık hücre alanı yalnız ilgili değeri geçersiz kılar.
- Hücre etiketi resmî `name/value/coord/componentIndex` bağlamlı şablon veya
  callback formatter'ı, çok satır/sarım, padding, clipping, renk/aile/kalınlık
  ve hücre içine bağlı dönüşümlü metin fazını uygular. Body/corner özel
  kenarlıkları dış sınırın altında ya da açık `z2` ile üstünde kalabilir;
  `itemStyle` opaklığı dolgu/kenarlığı birlikte etkiler; gölge ile
  düz/kesikli/noktalı yuvarlatılmış kenarlık aynı hücre geometrisini izler.
  Desen/decal ve iki katmanlı MBTI hücreleri ham piksel kontrollerindedir.
- `matrix.tooltip`, global seri tooltip'inden bağımsızdır ve resmî
  `matrixIndex/name/xyLocator` bağlamını verir. `cursor` hücre → body/corner
  veya x/y üst modeli sırasıyla miras alınır ve CSS cursor değerleri GPUI
  `CursorStyle` karşılığına çevrilir. `silent` belirtilmezse dolgulu rect
  etkileşimli, yalnız kenarlığı olan rect etkileşimsizdir; etiket tooltip ya
  da `triggerEvent` için ayrı ölçülmüş hedef olarak kalır. `triggerEvent`,
  x/y/body/corner türünü ve koordinatı taşıyan
  `GrafikOlayı::MatrisHücresiTıklandı` üretir; üstteki seri isabeti önce gelir.
- Bağlı seri kanıtları yalnız tek bir ısı haritasıyla sınırlı değildir:
  korelasyon/covariance tabloları Heatmap ve Scatter'ı, MBTI ile periyodik
  tablo Custom/Heatmap/decal bileşimini, graph/pie/grid-layout kendi gömülü
  koordinatlarını, stock Candlestick/Bar/Line'ı, sparkline Line'ı ve gizli
  veri-toplama örneği seri verisinden otomatik x/y kategori çıkarımını aynı
  `MatrisYerleşimi` üzerinde doğrular.
- Özellik matrisindeki 30 `matrix.*` satırının tamamı gerçek Rust API'sine,
  birim testine, veri biçimine, koordinat dalına ve resmî örnek kimliğine
  bağlanmıştır. `mainType`/çoklu `matrixIndex`, body/corner/data/value,
  coord/clamp/merge, children/size/levels/levelSize, show/type,
  itemStyle/label/formatter, cursor/silent/z2, background/border, tooltip ve
  triggerEvent satırlarının hiçbiri `yok` veya genel bir sahte eşlemeye
  düşmez.

600×450 kilitli Matrix statik kanıt metrikleri:

| Örnek | Değişen piksel oranı | SSIM | Ham oran / SSIM | Yapısal kapı |
|---|---:|---:|---:|---:|
| `matrix-confusion` | %0,9089 | 0,991307 | aynı | — |
| `matrix-correlation-heatmap` | %0,3696 | 0,992069 | aynı | — |
| `matrix-correlation-scatter` | %0,9804 | 0,993816 | aynı | — |
| `matrix-covariance` | %0,0581 | 0,999076 | aynı | — |
| `matrix-graph` | %0,5148 | 0,997802 | aynı | — |
| `matrix-grid-layout` | %0,7985 | 0,992557 | aynı | — |
| `matrix-mbti` | %0,7607 | 0,991013 | %2,9007 / 0,985845 | 10/10 |
| `matrix-mini-bar-data-collection` | %0,9963 | 0,992739 | aynı | — |
| `matrix-periodic-table` | %0,9370 | 0,993354 | %2,1911 / 0,987444 | 10/10 |
| `matrix-pie` | %0,3070 | 0,997986 | aynı | — |
| `matrix-simple` | %0,4537 | 0,992199 | aynı | — |
| `matrix-sparkline` | %0,8974 | 0,990813 | aynı | — |
| `matrix-stock` | %0,4193 | 0,993747 | aynı | — |

13 Matrix senaryosunun 13/13 karesi genel eşiği geçer. MBTI ve periyodik
tablo için tabloda birincil değer, iki görüntüye aynı `sigma=0.8` profili
uygulandıktan sonraki sonuçtur; ham metrik ve ham fark gizlenmez. Her iki
kartta dört renk ailesini/hücre katmanlarını temsil eden onar ham örnek de
kanal toleransını geçer. Böylece 332 kart korunurken statik görsel kanıtlı
kart sayısı 171 olur. Varsayılan özellikli çekirdek testlerin 344/344'ü,
uyum fixture testlerinin 55/55'i, `cargo check --all-targets`, no-default
çekirdek ve no-default PNG derlemeleri geçer. Matrix'e özel koşu 13/13,
ortak eksen/yazı regresyonlarını da içeren depo çapındaki koşu 218/218
karedir. Koyu kip, klavye/ARIA, bütün
programatik hover/select/blur yükleri, canlı animasyon ve ölçümlü performans
kapıları kapanana kadar kartların ve option satırlarının genel durumu
`uygulandı_kanıt_bekliyor` kalır; 13 statik görüntü nihai tamlık iddiası
değildir.

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

#### Tree aile kapısı

Normatif kaynak yüzeyi `../echarts/src/chart/tree/TreeSeries.ts`,
`TreeView.ts`, `treeLayout.ts`, `layoutHelper.ts`, `treeAction.ts`,
`treeVisual.ts` ve `../echarts/src/data/Tree.ts` dosyalarıdır. Galeri
seçenekleri ve verileri `../echarts-examples/public/examples/ts/tree-*.ts`
ile `public/data/asset/data/flare.json` kaynaklarından okunur. Bu kaynakların
yerel commit kilitleri Faz 0 manifestine dahildir; fixture içine yaklaşık ya
da elle sadeleştirilmiş veri kopyası konulmaz.

Uygulama kapısı şu yüzeyi birlikte kapsar:

- `TreeSeriesNodeItemOption` için `id`, `name`, `value`, `children`,
  `collapsed`, `link`, `target`, kategori, sembol/boyut/döndürme/kayma,
  `symbolKeepAspect`, item/line/label ve normal/emphasis/blur/select durumları;
- `TreeSeriesOption` için kutu yerleşimi, `orthogonal`/`radial`, dört `orient`
  yönü ve geriye uyumlu adları, `curve`/`polyline`, `edgeForkPosition`,
  `lineStyle.curveness`, `expandAndCollapse`, `initialTreeDepth`, `leaves`,
  `roam`, `nodeScaleRatio`, `center`, `zoom`, `silent`, `z`, tooltip ve
  seri/düğüm durum mirası;
- ECharts'ın sanal kök ve preorder `dataIndex` sözleşmesi, D3/Reingold–Tilford
  yerleşimi, radyal etiket yönü, daraltılmış dalın görünür yaprak gibi
  davranması, tek/çok çocuk polyline çatalı ve düğüm boyutunu roam sırasında
  koruyan ölçek telafisi;
- `treeExpandAndCollapse` action'ı, dal tıklaması, yaprağın değişmemesi,
  seri index/id/name seçicileri, tooltip'te kökten düğüme ad yolu ve GPUI
  üstünde ayrı `move`/`scale`/`true` roam izinleri. Tıklama, kaydırma ve
  yakınlaştırma model/görünüm durumunu günceller ve ayrı olay yükü üretir.

Yedi resmî kart (`tree-basic`, `tree-legend`, `tree-orient-bottom-top`,
`tree-orient-right-left`, `tree-polyline`, `tree-radial`, `tree-vertical`)
700×525 kaynak viewport'unda ve 600×450 karşılaştırma boyutunda ayrı fixture
olarak bulunur. Her kartta raster karşılaştırmasına ek olarak görünür düğüm,
ebeveyn/kenar, kenar yolu, etiket çapası/dönüşü, daraltma durumu, sembol
kayması/dönüşü ve 0,001 piksel nicemli koordinatlar FNV-1a sahne özetiyle
kilitlenir. Bu yapısal kapı yedi kartın yedisinde geçmeden tipografi profili
değerlendirilemez.

Tree için geçerli güncel kanıt durumu şudur: yedi sahne özeti de tam eşleşir;
ham ve aynı `sigma=0.8` profilli rasterlar ise sabit `%1`/`0.99` kapılarını
henüz geçmez. En iyi profilli değişen piksel oranı/SSIM değerleri sırasıyla
`tree-basic` `%0,980 / 0,98840`, `tree-legend` `%3,204 / 0,96278`,
`bottom-top` `%4,203 / 0,96321`, `right-left` `%2,087 / 0,98456`,
`polyline` `%2,234 / 0,98277`, `radial` `%8,590 / 0,93656` ve `vertical`
`%1,929 / 0,97864` düzeyindedir. Bu nedenle kartlar yapısal olarak doğru
olsa da statik görsel kapıda `kısmi` kalır; eşik, sigma veya maske büyütülerek
yeşile çevrilemez. Aile kapatılmadan önce glif rasteri sabit eşiklere
yaklaştırılmalı; aç/kapat ve roam için başlangıç/son görüntüleriyle olay ve
option/state günlükleri de kanıt paketine eklenmelidir.

#### Treemap aile kapısı

Normatif model, yerleşim, görsel ve etkileşim kaynakları
`../echarts/src/chart/treemap/TreemapSeries.ts`, `treemapLayout.ts`,
`treemapVisual.ts`, `TreemapView.ts`, `Breadcrumb.ts`, `treemapAction.ts` ile
`../echarts/src/chart/helper/treeHelper.ts` dosyalarıdır. Resmî galeri kanıtı
`../echarts-examples/public/examples/ts/treemap-{simple,disk,drill-down,obama,
show-parent,visual}.ts` ve `treemap-sunburst-transition.ts` kaynaklarını;
`disk.tree.json`, `obama_budget_proposal_2012.json`,
`ec-option-doc-statistics-201604.json` ve `echarts-package-size.json` sabit
verilerini doğrudan kullanır.

Uygulanan Treemap çekirdeği şu yüzeyi birlikte taşır:

- Sanal kök, kararlı preorder `dataIndex`, açık `id`/ad yedeği, sayı veya
  çok boyutlu değer, iç düğüm toplamı ve sonlu olmayan düğüm süzmesi; `sort`
  için azalan/artan/veri sırası ve ECharts eşit-değer indeks kararı;
- JavaScript `number` hassasiyetini koruyan `f64` squarify; altın oran ya da
  açık `squareRatio`; yarım `gapWidth`, `borderWidth`, üst etiket yüksekliği,
  `visibleMin`, `childrenVisibleMin` ve ham kardeş kapsamından hesaplanan
  görsel extent. `leafDepth`, her view-root değişiminde yeniden sıfırdan
  sayılır; `isLeafRoot`, ham çocuk varlığıyla değil `visibleMin` sonrasında
  kalan `viewChildren` ile belirlenir;
- Seri → level → düğüm mirasında color/colorAlpha/colorSaturation,
  visualMin/visualMax, sayı veya ad `visualDimension`, value/index/id
  `colorMappingBy`, doğrudan itemStyle colorAlpha/colorSaturation,
  borderColorSaturation, radius, opacity, shadow ve düz/gradyan/desen dolgu.
  Parent arka plan/kenarlık ile leaf içerik dolgusu iki ayrı katmandır;
  parent fill toplam raster içinde varmış gibi kabul edilmez;
- Normal/level/node label ve upperLabel; formatter, padding, taşma/kısaltma,
  rich-text koşuları, üst şerit ve yalnız gerçek leaf-root üzerinde
  `drillDownIcon`. Breadcrumb box yerleşimi, boş öğe genişliği, normal/vurgu
  stili ve hedef seri/kırıntı indeksini taşıyan geri çıkış isabeti;
- `roam` move/scale/true/false, `scaleLimit`, `clipWindow` origin/fullscreen,
  affine `rootRect`, düğüme merkezli `zoomToNodeRatio`; seri başına bağımsız
  view-root ve görünüm durumu. `nodeClick` zoomToNode/link/false, leaf-root
  inişi, breadcrumb çıkışı ve host tarafından güvenli biçimde ele alınan
  bağlantı isteği ayrı tipli olaylardır;
- ECharts action adlarıyla `treemapRootToNode`, `treemapZoomToNode`,
  `treemapRender` ve `treemapMove`; `seriesIndex`/`seriesId`/`seriesName`,
  `dataIndex`/`targetNodeId`, `rootRect`, batch ve `silent` davranışı.
  `setOption`, `replaceMerge`, restore ve clear yolları seri durumunu aynı
  kimlik/index yaşam döngüsüyle temizler veya korur;
- ECharts 6.1 `coordinateSystemUsage: 'box'` dalı: Treemap kutusu tüm tuval,
  Calendar gün hücresinin `contentRect`i veya Matrix hücre/aralık
  dikdörtgeni içinde aynı left/right/top/bottom/width/height kurallarıyla
  çözülür. Eksik calendar/matrix index ya da `coord` sessiz düşüş değil tipli
  doğrulama hatasıdır.

Yedi resmî fixture için iki bağımsız kanıt katmanı vardır. Referans yenileme,
aynı ECharts kaynağını iki ayrı Chromium çalıştırmasında üretip bit düzeyinde
kararlı olduğunu görmeden kilit yazmaz. Rasterdan ayrıca
`tools/uyum/echarts_referans.mjs --scene-output`, çalışan resmî ECharts
modelinden her görünür hücrenin dataIndex/ad/göreli derinlik,
`x/y/width/height`, fill/stroke, border/gap/upperHeight ve leaf/isLeafRoot
alanlarını 0,001 mantıksal piksel çözünürlükte çıkarır. Cizelge aynı şemayı
fixture'dan üretir; `tools/uyum/kanit.mjs` bu alanları hücre hücre karşılaştırır.
Sıfır alanlı ve iki renderer'ın da boyamadığı leaf dolgusu ile ECharts
renderer'ının zaten boyamadığı parent fill karşılaştırmaya sokulmaz; parent
border/background ve bütün geometrisi yine zorunludur. Sonuç; disk 265,
drill-down 173, Obama 245, show-parent 132, simple 6, transition 731 ve visual
245 olmak üzere **1.797/1.797 görünür hücrede sıfır yapısal uyuşmazlıktır**.
Toplam fark oranı bu kapıyı geçersiz kılamaz veya ince sınır farkını saklayamaz.

600×450 kilitli Treemap raster durumu:

| Örnek | Değişen piksel oranı | SSIM | Hücre kapısı | Statik durum |
|---|---:|---:|---:|---|
| `treemap-disk` | %3,4356 | 0,917960 | 265/265 | kanıt bekliyor |
| `treemap-drill-down` | %3,8448 | 0,902181 | 173/173 | kanıt bekliyor |
| `treemap-obama` | %3,8770 | 0,915761 | 245/245 | kanıt bekliyor |
| `treemap-show-parent` | %4,4381 | 0,910958 | 132/132 | kanıt bekliyor |
| `treemap-simple` | %0,3626 | 0,990192 | 6/6 | geçti |
| `treemap-sunburst-transition` | %3,9948 | 0,824104 | 731/731 | kanıt bekliyor |
| `treemap-visual` | %4,0233 | 0,901938 | 245/245 | kanıt bekliyor |

`treemap-simple` sabit `%1 / 0,99` raster kapısını geçer. Diğer altı kartın
hücre geometrisi ve boya katmanları doğrudan resmî sahneyle eşleşse de yoğun
küçük metinde Chrome Canvas ile tiny-skia glif rasteri farkı genel kapıyı
aşar; eşik, maske veya asimetrik bulanıklaştırma uygulanmaz. Bu nedenle yedi
karttan yalnız biri `statik_görsel: tam_kanıtlı`, diğerleri `kısmi` kalır.
`cursor`, ARIA/decal, bütün programatik emphasis/blur/select odak zincirleri,
universal transition ara kareleri ve ölçümlü/iptal edilebilir büyük veri işi
Faz 7/8 kapılarında tamamlanmadan Treemap ailesi genel olarak
`tam_kanıtlı` sayılmaz.

#### Sunburst aile kapısı

Sunburst aktarımının normatif çekirdeği
`../echarts/src/chart/sunburst/SunburstSeries.ts`, `sunburstLayout.ts`,
`sunburstVisual.ts`, `SunburstPiece.ts`, `SunburstView.ts`,
`sunburstAction.ts` ve `install.ts` dosyalarıdır. Galeri davranışı ve veri
şekli için `../echarts-examples/public/examples/ts` altındaki
`sunburst-simple`, `sunburst-borderRadius`, `sunburst-label-rotate`,
`sunburst-monochrome`, `sunburst-visualMap`, `sunburst-drink` ve
`sunburst-book` kaynakları kullanılır. Book kaynağının uzantısı resmî repoda
`.js`, diğerleri `.ts`dir. `tools/uyum/sunburst_verisi.mjs` bu dosyaları
TypeScript ile yerel olarak dönüştürür, kaynağın kendi veri kurma kodunu
çalıştırır ve yedi belirlenimci JSON ağacı üretir; elle kısaltılmış veya
temsili veri kullanılmaz.

Uygulanan Sunburst yüzeyi şunları birlikte taşır:

- Sanal kök, kararlı preorder `dataIndex`, sayı/çok boyutlu değer, iç düğüm
  toplamı, sıfır/sonlu olmayan değer kuralları ve seri/veri `colorBy` palet
  paylaşımı. Sıralama `desc`, `asc`, ham veri sırası veya tipli Rust callback
  olabilir; callback derinlik, değer ve veri sırasını görür;
- `radius`, seviye başına `r0/r`, `center`, `startAngle`, `clockwise`,
  `minAngle`, `stillShowZeroSum` ve `renderLabelForZeroData`. Sıfır değerli
  sektör sahne ve veri indeksinden atılmaz; yalnız ilgili seçenek kapalıysa
  etiketi görünmez. Yüzde ve mutlak yarıçaplar aynı kısa-tuval boyutuna göre
  çözülür;
- Seri → level → düğüm → normal/emphasis/blur/select kalıtımı; düz,
  gradyan ve desen dolgu, border türü/kalınlığı/rengi, dört sektör köşesi,
  opacity, dolgu ve kenarlık gölgesi. Saydam dolguda Canvas gölgesi dolgu
  maskesine yanlışlıkla yayılmaz; varsa yalnız stroke maskesi gölgelenir;
- İç/dış etiket, `radial`, `tangential` ve sayısal dönüş, ters tarafta
  otomatik 180° çevirme, distance/offset/align/verticalAlign/padding,
  çok satır, rich text ve `treePathInfo` taşıyan formatter callback'i.
  Zrender'ın dolgu parlaklığına bağlı iç metin rengi, otomatik 2 px iç/dış
  kontur, açık `textBorder*`, `textShadow*`, font ailesi ve macOS CJK geri
  düşümü raster yolunda uygulanır;
- `nodeClick: rootToNode | link | false`, merkez roll-up halkası, güvenli
  link/target isteği, `cursor` ve `silent`; vurgu odağında self/ancestor/
  descendant/relative. `sunburstRootToNode`, `sunburstHighlight` ve
  `sunburstUnhighlight` action adları query/batch/silent yükleriyle tipli
  çalışma zamanı olaylarına bağlıdır;
- `coordinateSystemUsage: 'box'` için none, Calendar içerik kutusu ve Matrix
  hücre/aralık kutusu. Eksik bileşen veya geçersiz koordinat tipli doğrulama
  hatasıdır. Seri kaydı, setOption birleşimi, replaceMerge/restore/clear ve
  seri kimliğine bağlı görünüm kökü aynı yaşam döngüsüne dahildir;
- `animationType: 'expansion' | 'scale'` iki resmî değer olarak modelde
  korunur. Kilitli ECharts 6.1 kaynağı bu alanı bildirip varsayılanı
  `expansion` yapsa da Sunburst çalışma zamanı alanı okumaz; Çizelge de
  kaynağın sahip olmadığı sahte bir davranış dalı kanıtlamaz. Genel
  add/update/remove ve universal transition ara kareleri Faz 8 kapısındadır.

Yapısal kanıt rasterdan bağımsızdır. Referans üreticisi çalışan ECharts
modelinden her sektör için veri sırası, ad, derinlik, değer, merkez,
`r0/r`, başlangıç/bitiş açısı, yön, dolgu, kenarlık, köşe yarıçapları ve
bağlı etiket görünürlüğü/metni/konumu/dönüşünü 0,001 mantıksal piksel
çözünürlükte çıkarır. Çizelge aynı şemayı kendi yerleşiminden üretir;
`tools/uyum/kanit.mjs` alanları sektör sektör karşılaştırır. İki tarafta da
görünmeyen etikete ait boyanmayan metin/konum ayrıntıları karşılaştırılmaz,
fakat görünürlük biti ve sektörün kendisi zorunlu kalır. Sonuç; Book 68,
Border Radius 13, Drink 110, Label Rotate 16, Monochrome 46, Simple 13 ve
VisualMap 20 olmak üzere **286/286 sektör ve 5.720 karşılaştırılan alanda
sıfır uyuşmazlıktır**.

600×450 kilitli Sunburst raster durumu şöyledir. Kapı metrikleri iki
görüntüye de aynı Gauss çekirdeği (`σ=0,8`) uygulandıktan sonra değişmeyen
`pixelmatch=0,1`, değişen piksel `≤%1` ve `SSIM≥0,99` eşikleriyle hesaplanır;
ham metrikler ve ham fark görüntüsü ayrıca saklanır.

| Örnek | Kapı farkı | Kapı SSIM | Ham fark | Ham SSIM | Sektör kapısı | Statik durum |
|---|---:|---:|---:|---:|---:|---|
| `sunburst-book` | %0,1663 | 0,994007 | %1,2856 | 0,991627 | 68/68 | geçti |
| `sunburst-borderRadius` | %0,0000 | 0,999734 | %0,0189 | 0,999606 | 13/13 | geçti |
| `sunburst-drink` | %0,0026 | 0,998112 | %0,5381 | 0,997092 | 110/110 | geçti |
| `sunburst-label-rotate` | %0,0133 | 0,998152 | %0,4826 | 0,997097 | 16/16 | geçti |
| `sunburst-monochrome` | %0,0000 | 0,999857 | %0,0119 | 0,999801 | 46/46 | geçti |
| `sunburst-simple` | %0,0037 | 0,998763 | %0,3226 | 0,997708 | 13/13 | geçti |
| `sunburst-visualMap` | %0,0067 | 0,998456 | %0,4804 | 0,997150 | 20/20 | geçti |

Referans yenileme her örneği iki bağımsız Chromium çalıştırmasında üretir
ve bit düzeyinde aynı değilse kilit yazmaz. Sunburst referansında örnek
seçeneği animation'ın kanıt için kapatılması dışında değiştirilmez; örneğin
kaynakta yazılmayan `title.padding`, ECharts'ın resmî 5 px varsayılanında
kalır. Yedi kartın fixture ve sahne testleri, çekirdek/action testleri ve
görsel raporu sırasıyla `examples/uyum_fixture.rs`,
`src/grafik/gunes.rs`, `src/eylem.rs` ve
`testler/gorsel/rapor/index.html` üzerinden denetlenir. Bu geçitle statik
görsel kanıt sayacı **182/332 (%54,8)** olur; tüm kapılar sayacı, Faz 7/8
etkileşim/animasyon ve çapraz yüzey kapıları tamamlanana kadar ayrı kalır.

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
2. 71 kapsam içi `noExplore` kaydının işaretli gizli doğrulama kartını
   conformance testine veya aynı yeteneği kanıtlayan açık bir matrise bağlama.
3. Kategori/arama/filtre/dark/decal/diff UX'ini tamamlayıp klavye ve ekran
   okuyucuyla test etme.
4. Tüm referans ve gerçek küçük resimleri kilitli kaynaklarla yeniden üretme;
   başarısız farkları tek tek kapatma.
5. API sözlüğü, Türkçe adlar, taşıma rehberi, destek matrisi ve resmi örnek
   kaynak atıfları.
6. Linux/macOS/Windows, gpui/Piksel/SVG ve gerekli DPI profillerinde tam CI.
7. Lisans/NOTICE, benchmark, panik yasağı ve yayın paketini doğrulama.

Nihai kabul:

- Galeri sayaçları `332 / 332 kilitli görsel kanıt` ve `332 / 332 tüm
  kapılar tam` gösterir; 261 resmî görünür ve 71 gizli doğrulama kartı ayrı
  sayılır.
- 332 kapsam içi resmi çekirdek kaydın tamamı kart ve conformance kanıtına
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
