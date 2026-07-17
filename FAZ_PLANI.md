# çizelge — ECharts Tam Eşdeğerlik Faz Planı

Hedef: `../echarts` (v6) deposundaki işlevselliği, aşağıdaki kapsam sınırları
içinde, gpui üzerinde yerli Rust olarak `cizelge`ye taşımak. Fazlar kullanım
sıklığına ve teknik bağımlılığa göre sıralanmıştır; her faz kendi başına
yayınlanabilir bir bütündür. **Faz geçişlerinde onay beklenmez**: bir fazın
kabul ölçütleri sağlandığında bir sonraki faza doğrudan geçilir.

## Kapsam dışı (kesin)

Aşağıdakiler bu projenin parçası DEĞİLDİR ve hiçbir faza eklenmez:

- **Coğrafi katman**: `geo` koordinat sistemi, `map` serisi, GeoJSON/harita
  kaydı, projeksiyonlar — ayrı bir çalışmanın konusudur.
- **3B görünümler ve GL serileri**: 3B destekli hiçbir grafik yok;
  `scatterGL`, `linesGL`, `flowGL`, `graphGL` ve `echarts-gl` ekosisteminin
  tamamı kapsam dışıdır.

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
3. **Bağımlılık sınırı (onaylı liste, 2026-07-17):** Bağımlılıklarda
   yalnızca şu lisanslar kullanılabilir: **MIT, BSD-2-Clause, BSD-3-Clause,
   ISC, CC0, Apache-2.0 ve MPL-2.0**. **GPL / LGPL / AGPL / SSPL lisanslı
   veya lisanssız hiçbir kod, crate ya da kod parçası projeye giremez.**

   **Kendimiz geliştiririz ilkesi (bağlayıcı):** Gereksinim duyduğumuz bir
   yetenek yalnızca onaylı liste dışındaki (Apache-2.0 sınırını aşan) bir
   lisansla sunuluyorsa, o bağımlılık ALINMAZ — **eşdeğeri bu projede,
   Apache-2.0 altında kendimiz geliştirilir.** (Bugüne kadarki uygulama da
   budur: takvim/tarih dönüşümleri `chrono` yerine `yardimci::takvim`
   olarak, çentik/yerleşim/eğri matematiği harici crate yerine yerli port
   olarak yazılmıştır.)

   **MPL-2.0 özel koşulu:** MPL lisanslı kodda **değişiklik yapılmaz**;
   yapılması gerekirse o değişiklikler MPL-2.0 gereği **açık kaynak olarak
   yayımlanır** ve bu durum `NOTICE` dosyasındaki MPL bölümüne işlenir.
   MPL'li bağımlılıklar ve durumları `NOTICE`'ta liste olarak tutulur.
4. **Zed deposu özel durumu (kesinleştirildi, 2026-07-17):** Zed çalışma
   alanı karma lisanslıdır (Apache-2.0 **ve** GPL bölümler içerir).
   **Yalnızca `Cargo.toml`'unda açıkça `license = "Apache-2.0"` beyan eden
   crate'ler kullanılır** (bugün: `gpui`, `gpui_platform`); **GPL-3
   sınırına dokunulmaz**: GPL lisanslı hiçbir Zed crate'inden kod
   kopyalanamaz, uyarlanamaz, doğrudan bağımlılık alınamaz, yama yapılamaz.
   `gpui`nin kendi içinden geçişli olarak gelen GPL crate'leri (zlog,
   ztracing) üst-akımın tutarsızlığıdır; bizim tarafımızda kullanılmaz,
   yalnızca bulgu olarak izlenir (aşağıda).
5. **Varlıklar da kapsamdadır:** Örnek verileri, yazı tipleri ve görseller Apache-2.0 ile uyumlu lisanslı olmalı
   ve kaynağı `NOTICE`'a işlenmelidir.
6. **Doğrulama otomatiktir:** `deny.toml`, madde 3'teki onaylı listeyi
   uygular (`cargo deny check licenses`); her fazın kabul ölçütüne "lisans
   denetimi yeşil" koşulu dahildir. Onaylı listenin dışında kalan ama Rust
   ekosisteminde fiilen kaçınılmaz izin verici lisanslar (Unicode-3.0, Zlib,
   Apache-2.0 LLVM istisnası, bzip2, Unlicense-ikili) `deny.toml`de
   gerekçeli olarak işaretlenmiştir ve kullanıcı vetosuna açıktır.

   **Bilinen lisans bulguları (2026-07-17 taraması):**
   - ❗ Zed'in `zlog`, `ztracing`, `ztracing_macro` crate'leri
     **GPL-3.0-or-later** ve Apache-2.0 beyanlı `gpui`, `sum_tree` üzerinden
     bunlara bağımlı (üst-akım tutarsızlığı). Politika gereği izinli
     DEĞİLDİR; çözüm seçenekleri: üst-akıma bildirim, Zed'de yama/`[patch]`
     ile GPL'siz eşdeğer, ya da kullanıcı kararı. Çözülene dek lisans
     denetimi bu üç crate için kırmızıdır.
   - ❗ `gpui_shared_string`, `gpui_util` lisans alanı bildirmiyor —
     üst-akıma bildirilecek.
   - ✅ MPL-2.0 bağımlılıkları (değiştirilmeden kullanılır, NOTICE'ta
     listeli): `option-ext`, `dwrote` (yalnız Windows hedefi), `cbindgen`
     (derleme aracı; mevcut derleme grafiğinde etkin değil).
7. **Depo yükümlülükleri:** Kök dizinde `LICENSE` (Apache-2.0 tam metni) ve
   Apache ECharts atfını taşıyan `NOTICE` dosyası bulunur; Apache-2.0 §4
   gereği bunlar dağıtımlarda korunur.

---

## 🚫 DEĞİŞTİRİLEMEZ KURAL: Çalışma zamanında panik yasaktır

Kütüphane ve örnek kodunda (tüm `src/` ve `examples/`) aşağıdaki yapılar
**kesinlikle** kullanılamaz; `Cargo.toml [lints.clippy]` bunları derleme
düzeyinde `deny` eder:

| Yasak | Yerine |
|---|---|
| `panic!`, `todo!`, `unimplemented!`, `unreachable!` | `Result<T, BilesenHatasi>` dönen yollar; imkânsız dallarda güvenli varsayılan |
| `unwrap`, `unwrap_unchecked`, `expect` | `Option::ok_or_else`, `unwrap_or`/`unwrap_or_else`, `let-else` |
| Çalışma zamanı `assert!`/`assert_eq!`/`assert_ne!`/`debug_assert!` | Tipli doğrulama (`doğrula() -> Result`) + işlem geri alma |
| Doğrulanmamış `[]` dizinleme/dilimleme | `get`/`get_mut`, `first`/`last`, desen eşleme (`if let [a, b] = …`) |
| Panikleyebilecek kontrolsüz aritmetik | `checked_*` ya da gerekçeli `saturating_*` |
| `RefCell::borrow` / `borrow_mut` | `try_borrow` / `try_borrow_mut` + kilitli durumda tanı |
| Metinden panikli dönüşüm | `TryFrom`, `Renk::çöz`, `Option` dönen çözümleyiciler |

Tamamlayıcı mekanizmalar (Faz 1'de kuruldu, tüm fazlarda zorunlu):

- **`BilesenHatasi`** — bileşenlerden dönen tipli hata; `Display` ile güvenli
  hata görünümü (`src/hata.rs`).
- **`BilesenTanisi` olay kanalı** — boyama hattı kurtarılabilir sorunda
  durmaz: sorunlu öğeyi atlar, tanıyı biriktirir; `GrafikGörünümü` bunları
  `EventEmitter<BilesenTanisi>` üzerinden yayımlar.
- **İşlem geri alma (transaction rollback)** — `seçenekleri_değiştir`, yeni
  seçenekleri `doğrula()` ile denetler; hata varsa mevcut durum korunur,
  `Err` döner ve tanı yayımlanır.
- **Tipli yetenek/iptal sonuçları** — durum değiştiren API'ler `Result`
  döner; sessiz başarısızlık yoktur.
- **Test-zamanı istisnası** — `#[cfg(test)]` modülleri ve `testler/`
  hedefleri test raporlaması için `assert`/`panic` kullanabilir (yerel
  `#[allow]` ile); bu istisna çalışma zamanı koduna taşınamaz.
- **Aşamalı sıkılaştırma** — tamsayı aritmetiği denetimi
  (`clippy::arithmetic_side_effects`) bugün gürültü oranı nedeniyle kapalı;
  taşma riskli çıkarmalar `saturating_sub` ile yazılır ve Faz 6'da lint
  tam açılır.

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

**Durum (2026-07-17):**
- ✅ 1.1 `ÇizimYüzeyi` trait'i (`cizim/yuzey.rs`): gpui gerçeklemesi `Çizici`,
  test gerçeklemesi `KayıtYüzeyi`; boyama hattı `grafiği_boya` saf işleve
  ayrıldı.
- ✅ 1.2 Gradyan tamlığı: eksene hizalı çok duraklı doğrusal gradyanlar
  kırpma bantlarıyla birebir; `Dolgu::RadyalGradyan` eşmerkezli halkalarla.
- ✅ 1.4 Kırpma (`kırpılı`, iç içe kullanılabilir).
- ✅ 1.5 Piksel netliği: `keskin` (yarım piksel) hizalama — eksen, çentik,
  bölme çizgileri, eksen imleci.
- ✅ 1.6 Olay sistemi: `GrafikOlayı::{ÖğeTıklandı, GöstergeDeğişti}`;
  sütun/dilim/sembol/saçılım isabet bölgeleri; `examples/olaylar.rs`.
- ✅ 1.7 `seçenekleri_değiştir`: `doğrula()` + işlem geri alma + veri geçiş
  animasyonu (`ara_değerle`, `animasyon_süresi_güncelleme`).
- ✅ 1.8 Metin ölçüm önbelleği (kareler arası, `try_borrow` korumalı).
- ✅ 1.9 Altın test altyapısı: `testler/altin.rs` + 6 altın dosyası
  (`ALTIN_GUNCELLE=1` ile yenilenir).
- ✅ Panik yasağı süpürmesi: tüm `src/` clippy `deny` setiyle temiz;
  `BilesenHatasi` + `BilesenTanisi` kanalı çalışıyor.
- ⏳ 1.3 Döndürülmüş metin: gpui metin sistemi döndürme desteklemiyor.
  Karar: kısa vadede yatay geri düşüş; kalıcı çözüm gpui'ye upstream katkı
  (metin koşusuna dönüşüm matrisi) ya da glif konturlarını yol olarak
  doldurma. Faz 2'deki etiket döndürme kalemi bu karara bağlı.
- ⏳ CI hattı yok (depo yerel); `cargo deny` yapılandırması `deny.toml`de
  hazır, CI kurulunca bağlanacak.

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

**Durum (2026-07-17):**
- ✅ 2.4 İmleyiciler: markLine (sabit/ortalama/min/maks, yatay-dikey),
  markPoint (raptiye), markArea — üç kartezyen seriye bağlı, altın testli.
- ✅ 2.2 (kısmi) Ara çentikler (`minorTick`), ara bölme çizgileri
  (`minorSplitLine`), bölme alanı (`splitArea`) — aralık ölçeğinde eşit,
  log ölçeğinde geometrik bölme.
- ✅ 2.5 (kısmi) Mum (candlestick) ve kutu (boxplot) serileri —
  VeriDeğeri::Dizi çok değerli öğelerle; efektli saçılım (effectScatter,
  `SaçılımSerisi::efektli(true)` + sürekli dalga animasyonu); kartezyen
  ısı haritası + GörselEşleme (visualMap sürekli kip çekirdeği).
- ✅ 2.1 Çoklu ızgara / çoklu eksen: `ızgara_ekle`, `x_ekseni_ekle`,
  `y_ekseni_ekle`, eksenlerde `ızgara_sırası`, serilerde `eksenler(x, y)`
  bağı; ızgara başına bölme/eksen çizimi, konum öntanımıları (2. x → Üst,
  2. y → Sağ), eksen-bağlı kapsam/kategori çözümü, ızgara-altında eksen
  ipucu ve doğrulamada bağ denetimi.
- ✅ 2.3 (kısmi) Çapraz imleç: fareden geçen kesikli çizgiler + eksen
  kenarlarında değer kutuları. `link` (ızgaralar arası eşleme) açık.
- ✅ 2.5 pictorialBar (Piktogram) ve `custom` seri (ÖzelSeri — eklenti
  noktası); 2.6 çizgi `sampling` (LTTB + ortalama).
- ⏳ Açık: axisPointer `link`, etiket döndürme (gpui kısıtı) +
  `alignTicks` + zaman ölçeği tam kademe.

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
4. **Toolbox:** PNG dışa aktarım (Faz 7.4 ile ortak çekirdek), veri
   görünümü, dataZoom/restore düğmeleri.
5. **Brush** (dikdörtgen/çokgen seçim) + seçim olayları.
6. **Timeline** bileşeni (seçenek dizisi üzerinde oynatma).
7. İpucunun zengin içeriği: çok sütunlu düzen, kullanıcı biçimleyicisine
   yapılandırılmış satır API'si (ECharts HTML formatter'ının tip-güvenli
   karşılığı).

**Kabul ölçütü:** 100k noktalı çizgi grafikte akıcı inside-zoom; visualMap'li
ısı haritası örneği; fırça seçimi olay yayını.

**Tahmin:** 5–6 hafta. (Bağımlılık: Faz 1; heatmap için Faz 2.)

**Durum (2026-07-17):**
- ✅ 3.1 dataZoom: `VeriYakınlaştırma` (İç + Sürgü), eksen pencereleme
  (kategorikte sıra-uzayı, sayısalda kapsam daraltma), pencereli seride
  ızgara kırpması, tekerlekle imleç-odaklı yakınlaştırma, sürükleyerek
  kaydırma, tutamaçlı/sürüklenebilir alt sürgü, `YakınlaştırmaDeğişti`
  olayı; örnek: examples/yakinlastirma.rs.
- ✅ 3.2 (kısmi) GörselEşleme sürekli kip + gradyan çubuğu (Faz 2'de).
- ✅ 3.2 visualMap parçalı (piecewise): tıklanabilir dilim listesi,
  kapalı dilim verisi çizilmez; 3.3 kaydırmalı gösterge (‹ n/m ›);
  3.4 araç kutusu "geri yükle" (+GeriYüklendi olayı); 3.5 fırça:
  dikdörtgen seçim kaplaması + FırçaSeçildi olayı.
- ⏳ Açık: timeline, zengin ipucu içeriği, toolbox PNG kaydetme.

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

**Durum (2026-07-17):**
- ✅ 4.3 Gösterge saati (gauge): renk bantlı yay, çentik/etiketler, dönen
  ibre, değer yazısı, banda göre renk çözümü, animasyonlu değer.
- ✅ 4.4 Huni (funnel): sıralama (azalan/artan/yok), dilim boşluğu,
  min/maks genişlik, iç/dış etiket + kılavuz çizgisi, gösterge (legend)
  entegrasyonu, öğe ipucu.
- ✅ 4.2 Radar: RadarKoordinatı (göstergeler, çokgen/daire ağ, dönüşümlü
  bölme alanları, kollar + gösterge adları) ve RadarSerisi (öğe başına
  çokgen, alan dolgusu, semboller, gösterge/legend ve öğe ipucu
  entegrasyonu, merkezden büyüme animasyonu).
- ✅ 4.1 Kutupsal koordinat: `KutupsalKoordinat` (açısal kategori/değer +
  radyal değer ekseni), halkalar/ışınlar/etiketlerle ağ çizimi; kutupsal
  sütun (yığın destekli dilimler), çizgi ve saçılım — serilerde
  `.kutupsal(true)`; öğe ipucu ve isabetler.
- ⏳ Açık: singleAxis/themeRiver (4.5), takvim koordinatı (4.6).

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

**Durum (2026-07-17):**
- ✅ 5.1 Hiyerarşik veri modeli: AğaçDüğümü (yaprak/dal, etkin değer).
- ✅ 5.2 (kısmi) Ağaç haritası: kareselleştirilmiş (squarify) yerleşim,
  iç içe derinlik (dal başlıkları + üst renkten türeyen tonlar), sığan
  hücrelere etiket, öğe ipucu; içeri gezinme (breadcrumb) açık.
- ✅ 5.3 (kısmi) Güneş patlaması: hiyerarşi iç içe halkalarda açısal
  paylara bölünür, üst renkten türeyen tonlar, öğe ipucu; tıklamayla
  odaklanma açık.
- ✅ 5.3 Ağaç (tree): düzenli yatay yerleşim, kübik bağlantılar, düğüm
  sembolleri + konuma göre etiketler, öğe ipucu.
- ✅ 5.5 (kısmi) Sankey: kaynaklardan en-uzun-yol katmanlaması (döngü
  bağları atlanır), değerle orantılı düğüm yükseklikleri ve kübik bağ
  şeritleri, düğüm etiketleri + öğe ipucu.
- ✅ 5.4 (kısmi) Grafo (graph): belirlenimci kuvvet yerleşimi (dairesel
  başlangıç + sabit yinelemeli itme/çekme/yerçekimi — altın test
  uyumlu) ve dairesel yerleşim; kategori renkli düğümler, eşik üstü ad
  etiketleri, öğe ipucu. Roam (sürükle/yakınlaş) açık.
- ✅ 5.5 Kiriş (chord): düğümler çemberde toplam değerle orantılı yay
  dilimleri, akışlar merkez kontrollü kübik şeritler, dış etiketler,
  yay isabetleriyle öğe ipucu.
- ✅ 5.6 Paralel koordinat: boyut başına aralık ölçeği + çentikli dikey
  eksenler, öğe başına yarı saydam çoklu çizgi, geçiş animasyonuyla
  uyumlu Dizi verisi.
- ⏳ Açık: treemap breadcrumb/gezinme, grafo roam + sürüklenebilir
  düğümler, sunburst tıklamayla odaklanma.

---

## Faz 6 — Veri katmanı ve ölçeklenebilirlik

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

## Faz 7 — Tema, erişilebilirlik, çıktı ve yayın

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

**Durum (2026-07-17):**
- ✅ 7.4 (kısmi) SVG dışa aktarım: `SvgYüzeyi` (ÇizimYüzeyi'nin üçüncü
  gerçeklemesi) + `svg_dışa_aktar` — yollar/dikdörtgenler/yazılar,
  kırpma `<clipPath>`, çok duraklı doğrusal ve radyal gradyanlar SVG'de
  doğal `<defs>` tanımlarıyla; belirlenimci çıktı altın testli.
  PNG (başsız rasterleştirme) açık.
- ⏳ Açık: tema sistemi/koyu tema (7.1), i18n (7.2), erişilebilirlik
  (7.3), eklenti kaydı API'si (7.5 — ÖzelSeri temelini attı), yayın
  (7.6; gpui yol bağımlılığı çözülmeli).

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
   stratejisi Faz 7'de netleşmeli.
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
| 6 — Veri katmanı | 4–6 | 34 |
| 7 — Ürünleşme | 3–4 | ~38 |

Tam zamanlı ~9 ay; fazlar 3+4 ve 5+6 kısmen paralelleştirilebilir.
