# çizelge — ECharts Tam Eşdeğerlik Faz Planı

Hedef: `../echarts` (v6) deposundaki işlevselliğin tamamını, gpui üzerinde
yerli Rust olarak `cizelge`ye taşımak. Fazlar kullanım sıklığına ve teknik
bağımlılığa göre sıralanmıştır; her faz kendi başına yayınlanabilir bir
bütündür.

Ölçek notu: ECharts ~138.000 satır TS + zrender ~50.000 satır. Mevcut taban
(Faz 0) ~7.000 satır. Aşağıdaki tahminler tam zamanlı çalışma haftası
cinsindendir ve ±%50 belirsizlik taşır.

---

## ⚖️ DEĞİŞTİRİLEMEZ KURAL: Apache-2.0 lisans sınırının dışına çıkılmaz

Bu kural fazlar üstüdür, her fazda ve her katkıda **kesinlikle** uyulması
zorunludur; hiçbir iş kalemi, hız veya kolaylık gerekçesiyle bu sınırı
esnetemez:

1. **Projenin tamamı Apache-2.0'dır.** Tüm kaynak kod, örnekler, belgeler ve
   üretilen çıktılar Apache-2.0 altında yayımlanır; başka lisansa geçiş ya da
   çifte lisanslama yapılmaz.
2. **Port kaynağı yalnızca Apache-2.0 ECharts/zrender'dır.** Algoritma ve
   seçenek modeli yalnızca `../echarts` (Apache-2.0) ve onun bağımlılığı
   zrender'dan (BSD-3, uyumlu) uyarlanır; atıf `NOTICE` ve `README` içinde
   korunur.
3. **Bağımlılık sınırı:** Yalnızca Apache-2.0 ile uyumlu, izin verici
   lisanslı crate'lere bağımlılık eklenebilir (Apache-2.0, MIT, BSD, Zlib,
   ISC vb.). **GPL / LGPL / AGPL / SSPL lisanslı veya lisanssız hiçbir kod,
   crate ya da kod parçası projeye giremez.**
4. **Zed deposu özel durumu:** Zed çalışma alanı karma lisanslıdır
   (Apache-2.0 **ve** GPL bölümler içerir). Yalnızca `Cargo.toml`'unda
   açıkça `license = "Apache-2.0"` yazan crate'lere (bugün: `gpui`,
   `gpui_platform`) bağımlılık kurulabilir; Zed'in GPL lisanslı editör
   crate'lerinden kod kopyalanamaz, uyarlanamaz, bağımlılık alınamaz.
5. **Varlıklar da kapsamdadır:** Örnek verileri, GeoJSON/harita verileri
   (Faz 6), yazı tipleri ve görseller Apache-2.0 ile uyumlu lisanslı olmalı
   ve kaynağı `NOTICE`'a işlenmelidir.
6. **Doğrulama otomatiktir:** Faz 1'de CI'a `cargo deny check licenses`
   (izin listesi: Apache-2.0, MIT, BSD-2/3, Zlib, ISC, Unicode) eklenir ve
   her fazın kabul ölçütüne "lisans denetimi yeşil" koşulu dahildir.
7. **Depo yükümlülükleri:** Kök dizinde `LICENSE` (Apache-2.0 tam metni) ve
   Apache ECharts atfını taşıyan `NOTICE` dosyası bulunur; Apache-2.0 §4
   gereği bunlar dağıtımlarda korunur.

---

## Faz 0 — Temel (TAMAMLANDI)

Çizgi/sütun/pasta/saçılım serileri; değer/kategori/zaman/log eksenleri; tek
kartezyen ızgara; başlık, gösterge, ipucu, eksen imleci; v6 teması; giriş
animasyonları; 22 birim testi.

---

## Faz 1 — Çizim çekirdeği ve altyapı sağlamlaştırma (zrender eşdeğeri)

**Amaç:** Sonraki tüm fazların üzerine oturacağı çizim/etkileşim/test
altyapısını tamamlamak. En kritik faz; buradaki eksikler ileride her seriye
ayrı ayrı yansır.

**Kaynak karşılığı:** zrender `graphic/`, `core/`, echarts `src/util/graphic.ts`,
`src/core/echarts.ts` (olay bölümü).

**İş kalemleri:**
1. **Çizici soyutlaması:** `Çizici`yi bir trait arkasına al
   (`ÇizimYüzeyi`); gpui gerçeklemesi + test için komut kaydeden sahte
   yüzey. Bu, görsel regresyon testinin temelidir (aşağıda).
2. **Gradyan tamlığı:** çok duraklı doğrusal gradyan (2-duraklı gpui
   gradyanına parçalayarak), radyal gradyan yaklaşımı (halka dilimleme).
   gpui'nin doğal kısıtları burada kapatılır.
3. **Döndürülmüş metin:** eksen etiketi `rotate` için strateji
   (glif-yolu üretimi ya da gpui'ye katkı). Riskli kalem — ön araştırma ilk
   hafta yapılmalı.
4. **Kırpma altyapısı:** iç içe kırpma yığını, ızgara-dışı taşma denetimi
   (bugün yalnız animasyon kırpması var).
5. **Piksel netliği:** 1 px çizgilerde 0.5 hizalama (eksen/bölme çizgileri).
6. **Olay sistemi:** ECharts `on('click' | 'mouseover' | 'legendselectchanged' …)`
   karşılığı tip-güvenli geri çağrılar; seri/öğe düzeyinde isabet testi
   çerçevesi (her seri türü kendi isabet geometrisini kaydeder).
7. **`seçenekleri_değiştir` birleştirme:** ECharts `setOption(merge)`
   davranışı + **durum geçiş animasyonu** (eski→yeni veri ara değerleme;
   çizgi/sütun/pasta için).
8. **Metin ölçüm önbelleği** ve boyama profilleme kancaları.
9. **Görsel regresyon testleri:** sahte yüzeyle üretilen komut listelerinin
   altın (golden) anlık görüntüleri; her seri türü için en az 3 senaryo.

**Kabul ölçütü:** Mevcut 5 örnek piksel-eş görünümünü korur; olay API'siyle
tıklanan dilimi raporlayan yeni örnek; golden test altyapısı CI'da koşuyor.

**Tahmin:** 3–4 hafta.

---

## Faz 2 — Kartezyen tamlık

**Amaç:** Kartezyen dünyada ECharts ile bire bir seçenek uyumu.

**Kaynak karşılığı:** `coord/cartesian/`, `component/axis/`,
`component/axisPointer/`, `component/marker/`, `chart/{candlestick,boxplot,
effectScatter,pictorialBar,heatmap,custom}`, `scale/Time.ts` (tam kademe),
`scale/minorTicks.ts`.

**İş kalemleri:**
1. **Çoklu ızgara / çoklu eksen:** `grid: []`, `xAxisIndex`/`yAxisIndex`
   karşılıkları (`ızgaralar`, `eksen_sırası`), eksen `offset`, sağ/üst ikinci
   değer ekseni.
2. **Eksen tamlığı:** `minorTick`/`minorSplitLine`, `splitArea`, etiket
   döndürme (Faz 1.3'e bağlı), `nameLocation/nameGap/nameRotate`,
   `alignTicks` (çift eksen hizalama — `axisAlignTicks.ts` portu),
   zaman ölçeğinin tam kademeli biçimleyicisi (`Time.ts` birebir).
3. **axisPointer tamamı:** `cross` tipi, eksen etiketli imleç, bağlantılı
   (`link`) imleçler, tetikleme gecikmeleri.
4. **İmleyiciler:** `markPoint`, `markLine` (min/max/average dahil),
   `markArea`.
5. **Yeni seriler:** mum (candlestick), kutu (boxplot), efektli saçılım
   (effectScatter — Faz 1 animasyon altyapısıyla), resimli sütun
   (pictorialBar), kartezyen ısı haritası (heatmap), **özel seri**
   (`custom` — renderItem karşılığı olarak kapalı çizim geri çağrısı).
6. Çizgi serisine `sampling` (lttb) ve `areaStyle.origin`.

**Kabul ölçütü:** ECharts belgelerindeki temel kartezyen örneklerin
(çift eksen, mum grafiği, markLine'lı sütun) Türkçe eşdeğerleri örnek
galerisinde; golden testleri yeşil.

**Tahmin:** 4–5 hafta. (Bağımlılık: Faz 1.)

---

## Faz 3 — Etkileşim bileşenleri

**Amaç:** ECharts'ı "keşfedilebilir" yapan bileşenler.

**Kaynak karşılığı:** `component/dataZoom/`, `component/visualMap/`,
`component/legend/legendScroll`, `component/toolbox/`, `component/brush/`,
`component/timeline/`, `visual/` (eşleme boru hattı).

**İş kalemleri:**
1. **dataZoom:** önce `inside` (tekerlek + sürükleme; eksen pencereleme
   altyapısı `coord` katmanına iner), sonra `slider` bileşeni
   (mini önizleme çizimiyle), `select` en son.
2. **Görsel eşleme boru hattı:** ECharts `visual/` katmanının karşılığı —
   seri verisinden renk/boyut/opaklık eşlemesi; **visualMap**
   (continuous + piecewise) bileşenleri.
3. **Gösterge genişletmeleri:** kaydırmalı gösterge, `selectedMode`,
   zengin biçimleyici.
4. **Toolbox:** PNG dışa aktarım (Faz 8.4 ile ortak çekirdek), veri
   görünümü, dataZoom/restore düğmeleri.
5. **Brush** (dikdörtgen/çokgen seçim) + seçim olayları.
6. **Timeline** bileşeni (seçenek dizisi üzerinde oynatma).
7. İpucunun zengin içeriği: çok sütunlu düzen, kullanıcı biçimleyicisine
   yapılandırılmış satır API'si (ECharts HTML formatter'ının tip-güvenli
   karşılığı).

**Kabul ölçütü:** 100k noktalı çizgi grafikte akıcı inside-zoom; visualMap'li
ısı haritası örneği; fırça seçimi olay yayını.

**Tahmin:** 5–6 hafta. (Bağımlılık: Faz 1; heatmap için Faz 2.)

---

## Faz 4 — Yeni koordinat sistemleri

**Amaç:** Kartezyen dışındaki düzlemler.

**Kaynak karşılığı:** `coord/polar/`, `coord/radar/`, `coord/single/`,
`coord/calendar/`, `chart/{radar,gauge,funnel,themeRiver}`.

**İş kalemleri:**
1. **Kutupsal (polar):** açısal + radyal eksen, kutupsal sütun/çizgi/saçılım,
   kutupsal ısı haritası.
2. **Radar koordinatı + radar serisi** (gösterge entegrasyonuyla).
3. **Gösterge saati (gauge)** — koordinatsız ama yay/işaretçi çizimi
   kutupsal altyapıyı paylaşır.
4. **Huni (funnel)** — koordinatsız yerleşim.
5. **SingleAxis + themeRiver.**
6. **Takvim koordinatı** + takvim ısı haritası.

**Kabul ölçütü:** ECharts galerisindeki klasik radar ("Bütçe Dağılımı"),
gauge ve kutupsal sütun örneklerinin eşdeğerleri.

**Tahmin:** 4–5 hafta. (Bağımlılık: Faz 1; visualMap'li örnekler için Faz 3.)

---

## Faz 5 — Hiyerarşik ve ilişkisel seriler

**Amaç:** Veri yapısı ağırlıklı, yerleşim algoritması yoğun seriler.

**Kaynak karşılığı:** `chart/{treemap,sunburst,tree,graph,sankey,chord,
parallel}`, `coord/parallel/`, `data/Tree.ts`, `data/Graph.ts`.

**İş kalemleri:**
1. **Ağaç/hiyerarşi veri modeli** (`Ağaç`, `Çizge` tipleri — `data/Tree.ts`,
   `data/Graph.ts` portu).
2. **treemap:** squarify yerleşimi, seviye stilleri, içeri gezinme
   (breadcrumb) — etkileşim yoğun.
3. **sunburst** (dilim hiyerarşisi + tıklamayla odaklanma) ve **tree**
   (orthogonal/radial yerleşim).
4. **graph:** none/circular/force yerleşimleri (kuvvet simülasyonu ayrı
   modül), sürükleme, roam (kaydır/yakınlaştır — Faz 3 zoom altyapısı).
5. **sankey** (katmanlı yerleşim + Bezier bağlar) ve **chord**.
6. **Paralel koordinat** + parallel serisi + eksen fırçalama.

**Kabul ölçütü:** Disk kullanımı treemap'i, organizasyon şeması tree'si,
kuvvet yönlendirmeli graph ve sankey örnekleri akıcı çalışır.

**Tahmin:** 6–8 hafta. (Bağımlılık: Faz 1, roam için Faz 3.)

---

## Faz 6 — Coğrafi görselleştirme

**Amaç:** geo/map ekosistemi.

**Kaynak karşılığı:** `coord/geo/`, `component/geo/`, `chart/{map,lines}`,
`chart/effectScatter` (geo üstünde).

**İş kalemleri:**
1. GeoJSON çözümleyici (+ TopoJSON değerlendirmesi), harita kaydı API'si
   (`harita_kaydet("türkiye", geojson)`).
2. **Geo koordinat sistemi:** projeksiyonlar (varsayılan + Mercator +
   özel projeksiyon kancası), roam, bölge seçimi/vurgusu.
3. **map serisi** (visualMap ile koroplet), geo üstünde scatter/effectScatter/
   **lines** (uçuş rotası efektleri).
4. Büyük çokgen setlerinde tessellation önbelleği.

**Kabul ölçütü:** Türkiye il haritasında koroplet + il tıklama olayı; uçuş
rotaları örneği.

**Tahmin:** 5–6 hafta. (Bağımlılık: Faz 3 — visualMap, roam.)

---

## Faz 7 — Veri katmanı ve ölçeklenebilirlik

**Amaç:** ECharts'ın veri boru hattının tam karşılığı; büyük veri.

**Kaynak karşılığı:** `component/dataset/`, `component/transform/`,
`data/` (DataStore, SeriesData, dimensions), `core/Scheduler.ts`,
`core/task.ts`, `processor/`.

**İş kalemleri:**
1. **dataset + dimensions + encode:** serilerden bağımsız veri kaynağı,
   sütun/satır düzeni, `encode` eşlemesi.
2. **transform:** filter/sort yerleşik; kullanıcı dönüşümü için trait.
3. **DataStore karşılığı:** tiplenmiş sütunlu depolama (`Vec<f64>` sütunlar),
   kopyasız dilimleme — bugünkü `Vec<VeriÖğesi>`nin yerini büyük veride alır.
4. **Aşamalı (progressive) işleme:** Scheduler/task boru hattının sade
   karşılığı; kare başına parça çizim, 1M+ noktada etkileşimi koruma.
5. Örnekleme (lttb/min-max) ve `large` kipi optimizasyonları.
6. Çok iş parçacıklı yerleşim (force, sankey, treemap yerleşimlerini
   arka plan iş parçacığına alma — gpui executor entegrasyonu).

**Kabul ölçütü:** 1M noktalı saçılım 60 fps kaydırma; dataset+encode ile
tek veri kaynağından 3 farklı grafik örneği.

**Tahmin:** 4–6 hafta. (Bağımlılık: Faz 1; dataZoom ile birlikte anlamlı.)

---

## Faz 8 — Tema, erişilebilirlik, çıktı ve yayın

**Amaç:** Ürünleşme.

**Kaynak karşılığı:** `theme/`, `i18n/`, `component/aria/`, `ssr/`,
`extension-src/`.

**İş kalemleri:**
1. **Tema sistemi:** tema kaydı, koyu tema (v6 `darkColor` belirteçleri),
   çalışma anında tema değişimi.
2. **Yerelleştirme:** metin kataloğu (TR yerleşik, EN paketi), sayı/tarih
   biçimlerinin yerele bağlanması.
3. **Erişilebilirlik:** gpui erişilebilirlik API'siyle `aria` karşılığı
   (grafik özeti, seri/öğe etiketleri).
4. **Dışa aktarım:** PNG (ekran dışı gpui render), SVG üretici
   (`ÇizimYüzeyi`nin ikinci gerçeklemesi — Faz 1.1 sayesinde ucuz), başsız
   (SSR benzeri) görüntü üretimi.
5. **Eklenti mimarisi:** seri/bileşen kaydı için genel API (`extension`
   karşılığı) — üçüncü taraf seri türleri.
6. **Yayın:** API dokümantasyonu (docs.rs), örnek galerisi, benchmark
   paketi, crates.io yayını (gpui bağımlılığının sürümlenmesi çözülmeli —
   bkz. Riskler).

**Kabul ölçütü:** Koyu temalı galeri; PNG/SVG çıktı testleri; crates.io'da
0.x yayını.

**Tahmin:** 3–4 hafta.

---

## Fazlar arası sürekli işler

- Her faz kendi golden/birim testlerini ekler (Faz 1.9 altyapısı).
- Her yeni seçenek README'deki eşleme tablosuna işlenir.
- Her faz sonunda `../echarts` ile **seçenek uyum denetimi**: ilgili
  `defaultOption` dosyaları taranıp desteklenmeyen alanlar listelenir.
- Türkçe adlandırma sözlüğü (`SOZLUK.md`) güncel tutulur: ECharts terimi →
  Türkçe karşılık (yeni katkıcılar için).

## Riskler ve açık sorular

1. **gpui kısıtları:** radyal/konik gradyan yok, metin döndürme yok,
   gradyanlar 2 duraklı. Çözüm yolları Faz 1'de; gerekirse gpui'ye yama
   (upstream katkı) planlanmalı.
2. **gpui sürümlenmesi:** yol bağımlılığı crates.io yayınını engeller;
   gpui'nin yayımlanan sürümüne geçiş ya da git bağımlılığıyla yayın
   stratejisi Faz 8'de netleşmeli.
3. **Metin ölçüm farkları:** font sistemine bağlı ölçümler golden testleri
   kırılgan yapabilir → testlerde sabit ölçümlü sahte metin sistemi.
4. **Performans:** lyon tessellation'ın kare başına maliyeti büyük veride
   sınırlayıcı olabilir → Faz 7'de tessellation önbelleği/instancing.
5. **Kapsam kayması:** `custom` seri ve `graphic` bileşeni ECharts'ta neredeyse
   ayrı birer çizim API'sidir; Faz 2.5'te bilinçli olarak daraltılmış API ile
   başlanmalı.

## Kaba toplam

| Faz | Süre (hafta) | Kümülatif |
|---|---|---|
| 1 — Çizim çekirdeği | 3–4 | 4 |
| 2 — Kartezyen tamlık | 4–5 | 9 |
| 3 — Etkileşim | 5–6 | 15 |
| 4 — Koordinat sistemleri | 4–5 | 20 |
| 5 — Hiyerarşik/ilişkisel | 6–8 | 28 |
| 6 — Coğrafi | 5–6 | 34 |
| 7 — Veri katmanı | 4–6 | 40 |
| 8 — Ürünleşme | 3–4 | ~44 |

Tam zamanlı ~10–11 ay; fazlar 3+4 ve 5+6 kısmen paralelleştirilebilir.
