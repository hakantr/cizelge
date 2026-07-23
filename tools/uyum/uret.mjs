#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const ARAÇ_DİZİNİ = path.dirname(new URL(import.meta.url).pathname);
const KÖK = path.resolve(ARAÇ_DİZİNİ, '../..');
const ECHARTS = path.resolve(KÖK, '../echarts');
const ÖRNEKLER = path.resolve(KÖK, '../echarts-examples');
const GPUI = path.resolve(KÖK, '../gpui');
const UYUM = path.join(KÖK, 'uyum');
const GÖRSEL_METRİKLER = path.join(KÖK, 'testler/gorsel/metrikler');

const BEKLENEN = Object.freeze({
  echartsCommit: '74e9e09a0b5687fdd34319121ac73b3022d1483c',
  examplesCommit: '1ff3451941535c51af83eacd504035ef4bfd7d0d',
  gpuiCommit: '5566476024607a4c6999ab7b91d0218633a9b96c',
  echartsVersion: '6.1.0',
  zrenderVersion: '6.1.0',
  çekirdek: 377,
  gl: 59,
  kapsamİçi: 332,
  görünür: 261,
  gizli: 71,
  geo: 45
});

const DURUMLAR = new Set([
  'yok',
  'kısmi',
  'uygulandı_kanıt_bekliyor',
  'tam_kanıtlı',
  'kapsam_dışı_geo_map',
  'kapsam_dışı_gl_3d'
]);

const KATEGORİ_SIRASI = [
  'line', 'bar', 'pie', 'scatter', 'candlestick', 'radar', 'boxplot',
  'heatmap', 'graph', 'lines', 'tree', 'treemap', 'sunburst', 'parallel',
  'sankey', 'funnel', 'gauge', 'pictorialBar', 'themeRiver', 'calendar',
  'matrix', 'chord', 'custom', 'dataset', 'dataZoom', 'graphic', 'rich'
];

const GL_KATEGORİLERİ = new Set([
  'globe', 'bar3D', 'scatter3D', 'surface', 'map3D', 'lines3D', 'line3D',
  'geo3D', 'scatterGL', 'linesGL', 'flowGL', 'graphGL'
]);

const GEO_DESENLERİ = [
  ['registerMap', /echarts\.registerMap\s*\(/i],
  ['geo_koordinatı', /coordinateSystem\s*:\s*['"](?:geo|bmap)['"]/i],
  ['map_serisi', /type\s*:\s*['"]map['"]/i],
  ['geo_bileşeni', /\bgeo\s*:\s*[{[]/i],
  ['bmap_bileşeni', /\bbmap\s*:\s*[{[]/i]
];

const BİLEŞENLER = [
  'title', 'legend', 'tooltip', 'axisPointer', 'grid', 'polar', 'radar',
  'singleAxis', 'calendar', 'matrix', 'parallel', 'dataset', 'dataZoom',
  'visualMap', 'toolbox', 'brush', 'graphic', 'timeline', 'markPoint',
  'markLine', 'markArea', 'aria'
];

const RUST_SERİLERİ = new Set([
  'line', 'bar', 'pie', 'scatter', 'effectScatter', 'candlestick', 'boxplot',
  'heatmap', 'pictorialBar', 'radar', 'tree', 'treemap', 'sunburst', 'graph',
  'chord', 'sankey', 'parallel', 'funnel', 'gauge', 'themeRiver', 'custom'
]);

// Otomatik TypeScript envanteri yalnız option adını bulabilir; aile portu
// tamamlandıkça gerçek Rust yüzeyi ile onu kanıtlayan test/örnek bağlantısı
// burada açıkça kilitlenir. `uygulandı_kanıt_bekliyor`, API ve statik
// eşdeğerliğin mevcut olduğunu fakat ortak davranış/erişilebilirlik/
// performans kapılarının henüz tümüyle kapanmadığını ifade eder.
const CALENDAR_OPTION_KANITI = Object.freeze({
  mainType: {
    api: 'src/model/secenekler.rs (Seçenekler::takvim, Seçenekler::takvimler)',
    testler: ['model::secenekler::testler::takvime_bağlı_scatter_eksik_calendar_index_değerini_reddeder'],
    örnekler: ['calendar-simple', 'calendar-vertical', 'calendar-charts'],
    veri_biçimleri: ['TakvimKoordinatı', 'Vec<TakvimKoordinatı>'],
    dallar: ['tek-takvim', 'çoklu-takvim']
  },
  cellSize: {
    api: 'src/model/takvim.rs (TakvimKoordinatı::hücre_boyutu)',
    testler: [
      'koordinat::takvim::testler::resmi_2016_isı_haritası_kutusu_ve_hücreleri_çözülür',
      'koordinat::takvim::testler::dikey_otomatik_hücre_alt_kenara_kadar_sığar'
    ],
    örnekler: ['calendar-heatmap', 'calendar-vertical', 'calendar-horizontal'],
    veri_biçimleri: ['number', 'auto', '[number|auto, number|auto]'],
    dallar: ['horizontal', 'vertical']
  },
  orient: {
    api: 'src/model/takvim.rs (TakvimKoordinatı::yön)',
    testler: ['koordinat::takvim::testler::yatay_ve_dikey_tarih_dönüşümü_tersinir'],
    örnekler: ['calendar-horizontal', 'calendar-vertical'],
    veri_biçimleri: ['TakvimYönü::Yatay', 'TakvimYönü::Dikey'],
    dallar: ['horizontal', 'vertical']
  },
  splitLine: {
    api: 'src/model/takvim.rs (TakvimKoordinatı::ayırıcı_göster, TakvimKoordinatı::ayırıcı)',
    testler: [
      'bilesen::takvim_cizimi::testler::split_line_show_false_ay_ayırıcılarını_tamamen_gizler',
      'calendar-simple::takvim_ay_ayırıcıları'
    ],
    örnekler: ['calendar-simple', 'calendar-heatmap', 'calendar-vertical'],
    veri_biçimleri: ['show:boolean', 'ÇizgiStili'],
    dallar: ['horizontal', 'vertical', 'seri-üstü-katman']
  },
  itemStyle: {
    api: 'src/model/takvim.rs (TakvimKoordinatı::öğe_stili)',
    testler: ['koordinat::takvim::testler::resmi_2016_isı_haritası_kutusu_ve_hücreleri_çözülür'],
    örnekler: ['calendar-simple', 'calendar-heatmap', 'calendar-lunar'],
    veri_biçimleri: ['ÖğeStili'],
    dallar: ['arka-plan', 'contentRect-iç-pay']
  },
  range: {
    api: 'src/model/takvim.rs (TakvimAralığı, TakvimKoordinatı::aralık)',
    testler: [
      'model::takvim::testler::iki_uçlu_ters_aralık_resmi_calendar_gibi_kronolojik_sıralanır',
      'koordinat::takvim::testler::aralık_dışı_tarih_yoktur'
    ],
    örnekler: ['calendar-simple', 'calendar-pie', 'calendar-charts'],
    veri_biçimleri: ['Unix ms başlangıç/bitiş', 'tam yıl'],
    dallar: ['yıl', 'ay', 'gün-aralığı', 'ters-uç-normalizasyonu']
  },
  dayLabel: {
    api: 'src/model/takvim.rs (gün_etiketi, ilk_gün, gün_etiketi_tarafı, gün_etiketi_kenar_boşluğu, gün_adları)',
    testler: ['bilesen::takvim_cizimi::testler::yıl_takvimi_ay_sınırları_ve_tek_harfli_günleri_çizer'],
    örnekler: ['calendar-simple', 'calendar-horizontal', 'calendar-vertical'],
    veri_biçimleri: ['Etiket', 'nameMap:String[]', 'margin:number|percent'],
    dallar: ['start', 'end', 'firstDay', 'yerel']
  },
  monthLabel: {
    api: 'src/model/takvim.rs (ay_etiketi, ay_adları, ay_etiketi_bağlamlı_biçimleyici)',
    testler: [
      'bilesen::takvim_cizimi::testler::ay_etiketi_echarts_yer_tutucularını_çözer',
      'bilesen::takvim_cizimi::testler::ay_ve_yıl_etiketi_bağlamlı_geri_çağrıları_tüm_resmi_alanları_taşır'
    ],
    örnekler: ['calendar-simple', 'calendar-horizontal', 'calendar-vertical'],
    veri_biçimleri: ['Etiket', 'nameMap:String[]', 'formatter:string|callback'],
    dallar: ['start', 'end', 'left', 'center', 'yerel']
  },
  yearLabel: {
    api: 'src/model/takvim.rs (yıl_etiketi, yıl_etiketi_konumu, yıl_etiketi_bağlamlı_biçimleyici)',
    testler: [
      'bilesen::takvim_cizimi::testler::yıl_etiketi_echarts_start_end_ve_name_map_yer_tutucularını_çözer',
      'bilesen::takvim_cizimi::testler::ay_ve_yıl_etiketi_bağlamlı_geri_çağrıları_tüm_resmi_alanları_taşır'
    ],
    örnekler: ['calendar-simple', 'calendar-horizontal', 'calendar-vertical'],
    veri_biçimleri: ['Etiket', 'formatter:string|callback'],
    dallar: ['auto', 'top', 'bottom', 'left', 'right']
  }
});

const MATRIX_OPTION_KANITI = Object.freeze({
  mainType: {
    api: 'src/model/secenekler.rs (Seçenekler::matris, Seçenekler::matris_ekle, Seçenekler::tüm_matrisler)',
    testler: ['model::secenekler::testler::matrixe_bagli_baslik_ve_izgara_index_ile_koordinati_birlikte_dogrular'],
    örnekler: ['matrix-simple', 'matrix-grid-layout', 'matrix-correlation-heatmap'],
    veri_biçimleri: ['MatrisKoordinatı', 'Vec<MatrisKoordinatı>'],
    dallar: ['tek-matrix', 'çoklu-matrix', 'matrixIndex']
  },
  x: {
    api: 'src/model/matris.rs (MatrisKoordinatı::x, MatrisBoyutu)',
    testler: ['koordinat::matris::testler::flat_matrix_data_point_layout_roundtrip'],
    örnekler: ['matrix-simple', 'matrix-grid-layout', 'matrix-periodic-table'],
    veri_biçimleri: ['MatrisBoyutu'],
    dallar: ['başlık', 'hiyerarşik-başlık', 'gizli-başlık']
  },
  y: {
    api: 'src/model/matris.rs (MatrisKoordinatı::y, MatrisBoyutu)',
    testler: ['koordinat::matris::testler::flat_matrix_data_point_layout_roundtrip'],
    örnekler: ['matrix-simple', 'matrix-mbti', 'matrix-covariance'],
    veri_biçimleri: ['MatrisBoyutu'],
    dallar: ['başlık', 'hiyerarşik-başlık', 'gizli-başlık']
  },
  body: {
    api: 'src/model/matris.rs (MatrisKoordinatı::gövde_hücresi/gövde_stili/gövde_etiketi/gövde_sessiz/gövde_z2)',
    testler: ['koordinat::matris::testler::hierarchy_size_and_merged_cells'],
    örnekler: ['matrix-simple', 'matrix-confusion', 'matrix-periodic-table'],
    veri_biçimleri: ['MatrisGövdeHücresi[]', 'üst-model hücre stili'],
    dallar: ['varsayılan-hücre', 'özel-hücre', 'birleşik-hücre']
  },
  corner: {
    api: 'src/model/matris.rs (MatrisKoordinatı::köşe_hücresi/köşe_stili/köşe_etiketi/köşe_sessiz/köşe_z2)',
    testler: ['koordinat::matris::testler::corner_negative_locator_and_point_roundtrip'],
    örnekler: ['matrix-grid-layout', 'matrix-mbti', 'matrix-periodic-table'],
    veri_biçimleri: ['MatrisGövdeHücresi[]', 'negatif MatrixXYLocator'],
    dallar: ['varsayılan-köşe', 'özel-köşe', 'birleşik-köşe']
  },
  backgroundStyle: {
    api: 'src/model/matris.rs (MatrisKoordinatı::arkaplan_stili)',
    testler: ['bilesen::matris_cizimi::testler::tooltip_trigger_event_ve_cursor_ayri_rect_etiket_hedefleri_uretir'],
    örnekler: ['matrix-simple', 'matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['ÖğeStili'],
    dallar: ['dolgu', 'dış-kenarlık', 'yuvarlak-köşe']
  },
  borderZ2: {
    api: 'src/model/matris.rs (MatrisKoordinatı::kenarlık_z2)',
    testler: ['bilesen::matris_cizimi::testler::tooltip_trigger_event_ve_cursor_ayri_rect_etiket_hedefleri_uretir'],
    örnekler: ['matrix-simple', 'matrix-confusion'],
    veri_biçimleri: ['i32'],
    dallar: ['hücre-altı', 'dış-sınır', 'özel-hücre-üstü']
  },
  tooltip: {
    api: 'src/model/matris.rs (MatrisKoordinatı::ipucu, MatrisKoordinatı::ipucu_bağlamlı_biçimleyici)',
    testler: [
      'cizim::gorunum::yakınlaştırma_yönü_testleri::matrix_yerel_tooltip_formatter_bilesen_baglamiyla_cizilir',
      'bilesen::matris_cizimi::testler::tooltip_trigger_event_ve_cursor_ayri_rect_etiket_hedefleri_uretir'
    ],
    örnekler: ['matrix-simple', 'matrix-mbti'],
    veri_biçimleri: ['İpucu', 'formatter(MatrisİpucuBağlamı)'],
    dallar: ['rect-hedefi', 'etiket-hedefi', 'yerel-formatter']
  },
  triggerEvent: {
    api: 'src/model/matris.rs (MatrisKoordinatı::tetikleme_olayı); src/cizim/olay.rs (GrafikOlayı::MatrisHücresiTıklandı)',
    testler: [
      'bilesen::matris_cizimi::testler::tooltip_trigger_event_ve_cursor_ayri_rect_etiket_hedefleri_uretir',
      'cizim::gorunum::yakınlaştırma_yönü_testleri::matrix_yerel_tooltip_formatter_bilesen_baglamiyla_cizilir'
    ],
    örnekler: ['matrix-simple', 'matrix-grid-layout'],
    veri_biçimleri: ['bool', 'MatrisHedefTürü', '[x,y] MatrixXYLocator'],
    dallar: ['x', 'y', 'body', 'corner', 'seri-z-sırası']
  },
  data: {
    api: 'src/model/matris.rs (MatrisBoyutu::veri; MatrisKoordinatı::gövde_hücresi/köşe_hücresi)',
    testler: [
      'koordinat::matris::testler::flat_matrix_data_point_layout_roundtrip',
      'koordinat::matris::testler::corner_negative_locator_and_point_roundtrip'
    ],
    örnekler: ['matrix-simple', 'matrix-periodic-table', 'matrix-mini-bar-data-collection'],
    veri_biçimleri: ['string[]', 'MatrisBoyutHücresi[]', 'MatrisGövdeHücresi[]'],
    dallar: ['dimension-data', 'body-data', 'corner-data', 'seri-kategori-toplama']
  },
  value: {
    api: 'src/model/matris.rs (MatrisBoyutHücresi::yeni; MatrisGövdeHücresi::değer)',
    testler: [
      'koordinat::matris::testler::length_fallback_and_invalid_duplicate',
      'koordinat::matris::testler::corner_negative_locator_and_point_roundtrip'
    ],
    örnekler: ['matrix-confusion', 'matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['String', 'null/None'],
    dallar: ['dimension-value', 'body-value', 'corner-value']
  },
  coord: {
    api: 'src/model/matris.rs (MatrisKonumu, MatrisAralığı, MatrisGövdeHücresi::yeni)',
    testler: [
      'koordinat::matris::testler::flat_matrix_data_point_layout_roundtrip',
      'koordinat::matris::testler::corner_negative_locator_and_point_roundtrip'
    ],
    örnekler: ['matrix-periodic-table', 'matrix-mbti', 'matrix-mini-bar-data-collection'],
    veri_biçimleri: ['index', 'ordinal value', '[start,end]', 'all', 'negatif locator'],
    dallar: ['tek-hücre', 'satır/sütun', 'dikdörtgen-aralık', 'başlık', 'köşe']
  },
  coordClamp: {
    api: 'src/model/matris.rs (MatrisGövdeHücresi::koordinatı_sınırla)',
    testler: ['koordinat::matris::testler::explicit_size_center_and_coord_clamp'],
    örnekler: ['matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['bool'],
    dallar: ['body', 'corner', 'geçersiz-uç', 'tüm-boyut']
  },
  mergeCells: {
    api: 'src/model/matris.rs (MatrisGövdeHücresi::birleştir)',
    testler: ['koordinat::matris::testler::hierarchy_size_and_merged_cells'],
    örnekler: ['matrix-grid-layout', 'matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['bool', 'MatrisAralığı'],
    dallar: ['body', 'corner', 'dataToLayout-genişletme']
  },
  show: {
    api: 'src/model/matris.rs (MatrisBoyutu::göster)',
    testler: ['koordinat::matris::testler::yapraklar_ve_karsi_baslik_seviyesi_ayni_fiziksel_boyutu_paylasir'],
    örnekler: ['matrix-correlation-scatter', 'matrix-covariance', 'matrix-stock'],
    veri_biçimleri: ['bool'],
    dallar: ['x-gizli', 'y-gizli', 'köşe-bastırma', 'gövdeyi-genişletme']
  },
  length: {
    api: 'src/model/matris.rs (MatrisBoyutu::uzunluk)',
    testler: ['koordinat::matris::testler::length_fallback_and_invalid_duplicate'],
    örnekler: ['matrix-correlation-heatmap', 'matrix-covariance'],
    veri_biçimleri: ['usize'],
    dallar: ['data-yok', 'data-öncelikli', 'seri-yedeği']
  },
  children: {
    api: 'src/model/matris.rs (MatrisBoyutHücresi::çocuk, MatrisBoyutHücresi::çocuklar)',
    testler: [
      'koordinat::matris::testler::hierarchy_size_and_merged_cells',
      'koordinat::matris::testler::shallow_leaf_spans_remaining_header_levels'
    ],
    örnekler: ['matrix-grid-layout', 'matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['MatrisBoyutHücresi[]'],
    dallar: ['yaprak', 'dal', 'dengesiz-derinlik', 'ordinal-aralık']
  },
  size: {
    api: 'src/model/matris.rs (MatrisBoyutHücresi::boyut)',
    testler: ['koordinat::matris::testler::hierarchy_size_and_merged_cells'],
    örnekler: ['matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['number', 'percent', 'auto/None'],
    dallar: ['x-yaprak-genişliği', 'y-yaprak-yüksekliği', 'kalanı-eşit-paylaştırma']
  },
  levels: {
    api: 'src/model/matris.rs (MatrisBoyutu::seviye_boyutları)',
    testler: ['koordinat::matris::testler::shallow_leaf_spans_remaining_header_levels'],
    örnekler: ['matrix-grid-layout', 'matrix-periodic-table'],
    veri_biçimleri: ['Vec<Option<Uzunluk>>'],
    dallar: ['x-seviyeleri', 'y-seviyeleri', 'null-seviye', 'açık-levelSize']
  },
  levelSize: {
    api: 'src/model/matris.rs (MatrisBoyutu::seviye_boyutu, MatrisBoyutu::seviye_boyutları)',
    testler: ['koordinat::matris::testler::yapraklar_ve_karsi_baslik_seviyesi_ayni_fiziksel_boyutu_paylasir'],
    örnekler: ['matrix-grid-layout', 'matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['number', 'percent', 'auto/None'],
    dallar: ['varsayılan-seviye', 'seviye-yaması', 'karşı-boyut-paylaşımı']
  },
  dividerLineStyle: {
    api: 'src/model/matris.rs (MatrisBoyutu::ayırıcı)',
    testler: ['koordinat::matris::testler::yapraklar_ve_karsi_baslik_seviyesi_ayni_fiziksel_boyutu_paylasir'],
    örnekler: ['matrix-simple', 'matrix-grid-layout', 'matrix-mbti'],
    veri_biçimleri: ['ÇizgiStili'],
    dallar: ['x-ayırıcı', 'y-ayırıcı', 'borderZ2-altı']
  },
  itemStyle: {
    api: 'src/model/matris.rs (MatrisBoyutHücresi/MatrisBoyutu/MatrisGövdeHücresi öğe_stili; gövde/köşe_stili)',
    testler: [
      'koordinat::matris::testler::cursor_mirası_ve_rect_silent_dolgudan_resmi_kuralla_cozulur',
      'bilesen::matris_cizimi::testler::label_silent_ve_item_style_opacity_shadow_border_type_uygulanir'
    ],
    örnekler: ['matrix-confusion', 'matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['ÖğeStili'],
    dallar: ['üst-model', 'hücre-yaması', 'dolgu', 'kenarlık', 'desen']
  },
  label: {
    api: 'src/model/matris.rs (MatrisBoyutHücresi/MatrisBoyutu/MatrisGövdeHücresi etiket; gövde/köşe_etiketi)',
    testler: [
      'bilesen::matris_cizimi::testler::tooltip_trigger_event_ve_cursor_ayri_rect_etiket_hedefleri_uretir',
      'bilesen::matris_cizimi::testler::label_silent_ve_item_style_opacity_shadow_border_type_uygulanir'
    ],
    örnekler: ['matrix-confusion', 'matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['Etiket', 'YazıStili', 'padding', 'overflow-break/truncate'],
    dallar: ['üst-model', 'hücre-yaması', 'sarım', 'kırpma', 'çok-satır']
  },
  formatter: {
    api: 'src/model/matris.rs (MatrisEtiketiBiçimleyicisi; etiket_bağlamlı_biçimleyici)',
    testler: ['cizim::gorunum::yakınlaştırma_yönü_testleri::matrix_yerel_tooltip_formatter_bilesen_baglamiyla_cizilir'],
    örnekler: ['matrix-confusion', 'matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['template string', 'callback(MatrisEtiketiBağlamı)'],
    dallar: ['x', 'y', 'body', 'corner', 'name/value/coord']
  },
  cursor: {
    api: 'src/model/matris.rs (imleç/gövde_imleci/köşe_imleci); src/cizim/pencere.rs (gpui_imleci)',
    testler: [
      'koordinat::matris::testler::cursor_mirası_ve_rect_silent_dolgudan_resmi_kuralla_cozulur',
      'bilesen::matris_cizimi::testler::tooltip_trigger_event_ve_cursor_ayri_rect_etiket_hedefleri_uretir'
    ],
    örnekler: ['matrix-simple', 'matrix-mbti'],
    veri_biçimleri: ['CSS cursor string'],
    dallar: ['üst-model-mirası', 'hücre-yaması', 'gpui CursorStyle']
  },
  silent: {
    api: 'src/model/matris.rs (sessiz/gövde_sessiz/köşe_sessiz); src/koordinat/matris.rs (dolguya bağlı varsayılan)',
    testler: ['koordinat::matris::testler::cursor_mirası_ve_rect_silent_dolgudan_resmi_kuralla_cozulur'],
    örnekler: ['matrix-simple', 'matrix-periodic-table', 'matrix-mbti'],
    veri_biçimleri: ['bool', 'null/otomatik'],
    dallar: ['dolgulu-rect', 'dolgusuz-rect', 'üst-model', 'hücre-yaması', 'etiket-hedefi']
  },
  z2: {
    api: 'src/model/matris.rs (z2/gövde_z2/köşe_z2)',
    testler: ['bilesen::matris_cizimi::testler::tooltip_trigger_event_ve_cursor_ayri_rect_etiket_hedefleri_uretir'],
    örnekler: ['matrix-confusion', 'matrix-periodic-table'],
    veri_biçimleri: ['i32'],
    dallar: ['dimension-default-50', 'body-default-25', 'special-default-100', 'borderZ2']
  },
  type: {
    api: 'src/model/matris.rs (MatrisBoyutu; ECharts iç invariantı: category)',
    testler: ['koordinat::matris::testler::flat_matrix_data_point_layout_roundtrip'],
    örnekler: ['matrix-simple'],
    veri_biçimleri: ["sabit 'category'"],
    dallar: ['ordinal-meta', 'index', 'string-value']
  }
});

const PARALLEL_OPTION_KANITI = Object.freeze({
  mainType: {
    api: 'src/model/secenekler.rs (GrafikSeçenekleri::paralel, paralel_ekle, tüm_paraleller)',
    testler: ['model::secenekler::testler::parallel_serisi_ortuk_koordinati_kabul_eder_ve_acik_baglari_dogrular'],
    örnekler: ['parallel-simple', 'parallel-aqi', 'parallel-nutrients', 'doc-example/parallel-all'],
    veri_biçimleri: ['ParalelKoordinatı', 'Vec<ParalelKoordinatı>'],
    dallar: ['örtük-koordinat', 'tek-parallel', 'çoklu-parallel', 'parallelId/index']
  },
  layout: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::yerleşim, ParalelYerleşim)',
    testler: ['koordinat::paralel::testler::dikey_yerlesimde_eksenleri_y_boyutuna_dizer'],
    örnekler: ['parallel-simple', 'parallel-nutrients'],
    veri_biçimleri: ['ParalelYerleşim::Yatay', 'ParalelYerleşim::Dikey'],
    dallar: ['horizontal', 'vertical']
  },
  axisExpandable: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::eksen_genişletilebilir)',
    testler: ['koordinat::paralel::testler::genisletme_penceresi_disindaki_eksenleri_daraltir'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['bool'],
    dallar: ['kapalı', 'geniş-eksen', 'dar-eksen', 'etiket-gizleme']
  },
  axisExpandCenter: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::eksen_genişletme_merkezi)',
    testler: ['koordinat::paralel::testler::genisletme_penceresi_disindaki_eksenleri_daraltir'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['Option<f32>'],
    dallar: ['otomatik-merkez', 'açık-merkez', 'sınır-normalizasyonu']
  },
  axisExpandCount: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::eksen_genişletme_sayısı)',
    testler: ['koordinat::paralel::testler::genisletme_penceresi_disindaki_eksenleri_daraltir'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['usize'],
    dallar: ['sıfır', 'kısmi-pencere', 'tüm-eksenler']
  },
  axisExpandWidth: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::eksen_genişletme_genişliği)',
    testler: ['koordinat::paralel::testler::genisletme_penceresi_disindaki_eksenleri_daraltir'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['f32 piksel'],
    dallar: ['50px-öntanım', 'özel-genişlik', 'daralan-aralık']
  },
  axisExpandTriggerOn: {
    api: 'src/model/paralel.rs (ParalelGenişletmeTetikleyicisi); src/cizim/pencere.rs (click/mousemove GPUI yolu)',
    testler: ['koordinat::paralel::testler::genisletme_faresi_resmi_merkez_slide_ve_jump_kurallarini_uygular'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['Tıklama', 'FareHareketi'],
    dallar: ['click', 'mousemove', '5px-tıklama-eşiği']
  },
  axisExpandRate: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::eksen_genişletme_oranı); src/cizim/pencere.rs (fixRate)',
    testler: ['koordinat::paralel::testler::genisletme_faresi_resmi_merkez_slide_ve_jump_kurallarini_uygular'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['f32 milisaniye oranı'],
    dallar: ['slide', 'hız-sınırlama']
  },
  axisExpandDebounce: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::eksen_genişletme_gecikmesi_ms); src/cizim/pencere.rs (GPUI zamanlayıcı/iptal belirteci)',
    testler: ['koordinat::paralel::testler::genisletme_faresi_resmi_merkez_slide_ve_jump_kurallarini_uygular'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['u64 ms'],
    dallar: ['jump-gecikmesi', 'son-girdi-kazanır']
  },
  axisExpandSlideTriggerArea: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::eksen_genişletme_kaydırma_tetik_alanı)',
    testler: ['koordinat::paralel::testler::genisletme_faresi_resmi_merkez_slide_ve_jump_kurallarini_uygular'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['[Option<f32>; 3]'],
    dallar: ['merkez-yok', 'kenar-slide', 'dış-jump']
  },
  axisExpandWindow: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::eksen_genişletme_penceresi); src/eylem.rs (parallelAxisExpand)',
    testler: [
      'koordinat::paralel::testler::genisletme_faresi_resmi_merkez_slide_ve_jump_kurallarini_uygular',
      'eylem::testler::parallel_actionlari_axis_araliklarini_ve_genisletme_penceresini_gunceller'
    ],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['[f32; 2]', 'parallelAxisExpand action'],
    dallar: ['başlangıç-penceresi', 'slide', 'jump', 'kapsama-sınırlama']
  },
  parallelAxisDefault: {
    api: 'src/model/paralel.rs (ParalelKoordinatı::eksen_varsayılanı, ParalelEkseni::çöz)',
    testler: ['cizim::gorunum::yakınlaştırma_yönü_testleri::parallel_seri_axis_default_ortuk_koordinata_miras_kalir'],
    örnekler: ['parallel-aqi', 'parallel-nutrients', 'doc-example/parallel-all'],
    veri_biçimleri: ['Eksen', 'alan-bazlı ParalelEkseni yaması'],
    dallar: ['value', 'category', 'time', 'log', 'ad/stil mirası', 'z=10']
  }
});

const PARALLEL_SERIES_OPTION_KANITI = Object.freeze({
  type: {
    api: 'src/model/seri.rs (Seri::Paralel, ParalelSerisi)',
    testler: ['grafik::paralel::testler::resmi_seri_varsayilanlarini_korur'],
    örnekler: ['parallel-simple', 'parallel-aqi', 'parallel-nutrients'],
    veri_biçimleri: ["sabit 'parallel'"],
    dallar: ['kayıtlı-seri', 'örtük-parallel']
  },
  coordinateSystem: {
    api: 'src/model/seri.rs (Seri::Paralel; sabit parallel koordinatı)',
    testler: ['model::secenekler::testler::parallel_serisi_ortuk_koordinati_kabul_eder_ve_acik_baglari_dogrular'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ["sabit 'parallel'"],
    dallar: ['örtük', 'açık-bileşen']
  },
  data: {
    api: 'src/model/seri.rs (ParalelSerisi::veri, karma_veri); src/model/deger.rs (KarmaDizi)',
    testler: ['cizim::gorunum::yakınlaştırma_yönü_testleri::parallel_resmi_karma_satirlari_eksenleri_ve_ust_axis_katmanini_uretir'],
    örnekler: ['parallel-simple', 'parallel-aqi', 'parallel-nutrients'],
    veri_biçimleri: ['number[]', '(number|string|boolean|null|time)[]', 'VeriÖğesi'],
    dallar: ['sayısal', 'kategori', 'boş-boyut', '7.637-satır']
  },
  value: {
    api: 'src/model/deger.rs (VeriDeğeri::Dizi, KarmaDizi); src/model/deger.rs (VeriÖğesi)',
    testler: ['grafik::paralel::testler::kategori_bos_deger_smooth_ve_coklu_cizgi_hitini_destekler'],
    örnekler: ['parallel-simple', 'parallel-aqi'],
    veri_biçimleri: ['ParallelSeriesDataValue', 'data item value'],
    dallar: ['nesne-öğesi', 'yalın-dizi', 'öğe-stili/etiketi']
  },
  lineStyle: {
    api: 'src/model/seri.rs (çizgi_stili/vurgu_çizgi_stili/bulanık_çizgi_stili/seçili_çizgi_stili)',
    testler: ['grafik::paralel::testler::resmi_seri_varsayilanlarini_korur'],
    örnekler: ['parallel-simple', 'parallel-aqi', 'parallel-nutrients'],
    veri_biçimleri: ['ÇizgiStili', 'öğe ÖğeStili'],
    dallar: ['normal', 'emphasis', 'blur', 'select', 'öğe-rengi']
  },
  label: {
    api: 'src/model/seri.rs (etiket ve durum EtiketYaması alanları); src/grafik/paralel.rs (paralel_etiketini_çiz)',
    testler: ['grafik::paralel::testler::sessiz_seri_hit_uretmez_ama_programatik_durum_etiketini_cizer'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['Etiket', 'EtiketYaması', 'formatter'],
    dallar: ['normal', 'emphasis', 'select', 'öğe-yaması']
  },
  activeOpacity: {
    api: 'src/model/seri.rs (ParalelSerisi::aktif_opaklık)',
    testler: ['cizim::gorunum::yakınlaştırma_yönü_testleri::parallel_coklu_koordinat_secim_opakligi_ve_expand_hedefini_korur'],
    örnekler: ['parallel-nutrients'],
    veri_biçimleri: ['f32 0..1'],
    dallar: ['normal', 'active']
  },
  inactiveOpacity: {
    api: 'src/model/seri.rs (ParalelSerisi::etkin_değil_opaklık)',
    testler: ['cizim::gorunum::yakınlaştırma_yönü_testleri::parallel_coklu_koordinat_secim_opakligi_ve_expand_hedefini_korur'],
    örnekler: ['parallel-nutrients'],
    veri_biçimleri: ['f32 0..1'],
    dallar: ['normal', 'inactive', 'sıfır-opaklık']
  },
  smooth: {
    api: 'src/model/seri.rs (ParalelSerisi::yumuşak/yumuşaklık); src/grafik/paralel.rs (Bezier yolu)',
    testler: ['grafik::paralel::testler::kategori_bos_deger_smooth_ve_coklu_cizgi_hitini_destekler'],
    örnekler: ['parallel-nutrients'],
    veri_biçimleri: ['bool', 'f32 0..1'],
    dallar: ['polyline', 'Bezier', 'boş-boyut']
  },
  realtime: {
    api: 'src/model/seri.rs (ParalelSerisi::gerçek_zamanlı); src/cizim/pencere.rs (axisAreaSelect güncellemesi)',
    testler: ['eylem::testler::parallel_actionlari_axis_araliklarini_ve_genisletme_penceresini_gunceller'],
    örnekler: ['parallel-nutrients'],
    veri_biçimleri: ['bool'],
    dallar: ['sürüklerken', 'sürükleme-sonunda']
  },
  tooltip: {
    api: 'src/model/seri.rs (ParalelSerisi::ipucu); src/grafik/paralel.rs (paralel_ipucu_değerleri)',
    testler: ['cizim::gorunum::yakınlaştırma_yönü_testleri::parallel_resmi_karma_satirlari_eksenleri_ve_ust_axis_katmanini_uretir'],
    örnekler: ['parallel-aqi', 'parallel-nutrients'],
    veri_biçimleri: ['İpucu', 'boyut adı/değer satırları'],
    dallar: ['global', 'seri-yaması', 'silent']
  },
  parallelAxisDefault: {
    api: 'src/model/seri.rs (ParalelSerisi::eksen_varsayılanı)',
    testler: ['cizim::gorunum::yakınlaştırma_yönü_testleri::parallel_seri_axis_default_ortuk_koordinata_miras_kalir'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['Eksen'],
    dallar: ['örtük-parallel', 'bileşen-varsayılanı']
  },
  parallelId: {
    api: 'src/model/seri.rs (ParalelSerisi::paralel_kimliği)',
    testler: ['cizim::gorunum::yakınlaştırma_yönü_testleri::parallel_coklu_koordinat_secim_opakligi_ve_expand_hedefini_korur'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['String'],
    dallar: ['kimlik-bağı', 'eksik-kimlik-doğrulaması']
  },
  parallelIndex: {
    api: 'src/model/seri.rs (ParalelSerisi::paralel_sırası)',
    testler: ['model::secenekler::testler::parallel_serisi_ortuk_koordinati_kabul_eder_ve_acik_baglari_dogrular'],
    örnekler: ['parallel-simple'],
    veri_biçimleri: ['usize'],
    dallar: ['sıra-bağı', 'çoklu-parallel']
  }
});

const TREE_ÖRNEKLERİ = Object.freeze([
  'tree-basic', 'tree-legend', 'tree-orient-bottom-top',
  'tree-orient-right-left', 'tree-polyline', 'tree-radial', 'tree-vertical'
]);

const TREE_SERIES_OPTION_KANITI = Object.freeze({
  type: {
    api: 'src/model/seri.rs (Seri::Ağaç, AğaçSerisi)',
    testler: ['tree_fixture_testleri::yedi_resmi_tree_fixture_seçeneklerini_ve_verisini_korur'],
    örnekler: TREE_ÖRNEKLERİ,
    veri_biçimleri: ["sabit 'tree'"],
    dallar: ['kayıtlı-seri', 'boyama', 'isabet/tooltip']
  },
  data: {
    api: 'src/model/seri.rs (AğaçSerisi::kökler); src/model/agac.rs (AğaçDüğümü)',
    testler: [
      'tree_fixture_testleri::yedi_resmi_tree_fixture_seçeneklerini_ve_verisini_korur',
      'tree_fixture_testleri::yedi_resmi_tree_sahnesinin_geometrik_ozeti_kilitlidir'
    ],
    örnekler: TREE_ÖRNEKLERİ,
    veri_biçimleri: ['TreeSeriesNodeItemOption[]', 'AğaçDüğümü[]'],
    dallar: ['sanal-kök', 'preorder-dataIndex', 'değer', 'ad/id', 'çok-kök-girdisi']
  },
  children: {
    api: 'src/model/agac.rs (AğaçDüğümü::çocuklar, dal)',
    testler: ['model::agac::testler::tree_dugumu_resmi_veri_sembol_ve_durum_alanlarini_korur'],
    örnekler: TREE_ÖRNEKLERİ,
    veri_biçimleri: ['Vec<AğaçDüğümü>'],
    dallar: ['yaprak', 'dal', 'özyinelemeli-alt-soy']
  },
  collapsed: {
    api: 'src/model/agac.rs (AğaçDüğümü::daraltılmış); src/model/seri.rs (düğüm_daraltmasını_değiştir)',
    testler: [
      'eylem::testler::tree_expand_and_collapse_action_modeli_ve_olay_yukunu_gunceller',
      'grafik::agac::testler::ilk_derinlik_ve_açık_daraltılmış_yaması_görünür_alt_soyu_belirler'
    ],
    örnekler: ['tree-basic', 'tree-orient-right-left', 'tree-radial'],
    veri_biçimleri: ['Option<bool>', 'treeExpandAndCollapse action'],
    dallar: ['açık-yama', 'kapalı-yama', 'ilk-derinlik-mirası', 'dal-tıklaması']
  },
  layout: {
    api: 'src/model/agac.rs (AğaçYerleşimi); src/grafik/agac.rs (D3/Reingold–Tilford yerleşimi)',
    testler: [
      'grafik::agac::testler::radyal_kök_merkezde_ve_alt_soy_yarıçapta_yerleşir',
      'tree_fixture_testleri::yedi_resmi_tree_sahnesinin_geometrik_ozeti_kilitlidir'
    ],
    örnekler: TREE_ÖRNEKLERİ,
    veri_biçimleri: ['AğaçYerleşimi::Dik', 'AğaçYerleşimi::Radyal'],
    dallar: ['orthogonal', 'radial', 'çoklu-dal-ayırma']
  },
  orient: {
    api: 'src/model/agac.rs (AğaçYönü); src/grafik/agac.rs (dört dik yön dönüşümü)',
    testler: ['grafik::agac::testler::dört_dik_yön_aynı_düğüm_sayısını_korur'],
    örnekler: ['tree-basic', 'tree-orient-bottom-top', 'tree-orient-right-left', 'tree-vertical'],
    veri_biçimleri: ['LR/horizontal', 'RL', 'TB/vertical', 'BT'],
    dallar: ['soldan-sağa', 'sağdan-sola', 'üstten-alta', 'alttan-üste']
  },
  edgeShape: {
    api: 'src/model/agac.rs (AğaçKenarBiçimi); src/grafik/agac.rs (eğri_kenar_yolu, kırık_kenar_yolu)',
    testler: [
      'grafik::agac::testler::kırık_kenar_tek_çocukta_doğrudan_çizgidir',
      'tree_fixture_testleri::yedi_resmi_tree_sahnesinin_geometrik_ozeti_kilitlidir'
    ],
    örnekler: ['tree-basic', 'tree-polyline', 'tree-radial'],
    veri_biçimleri: ['curve', 'polyline'],
    dallar: ['Bezier', 'tek-çocuk-doğrudan', 'çok-çocuk-ortak-çatal', 'radyal-güvenli-eğri']
  },
  edgeForkPosition: {
    api: 'src/model/seri.rs (AğaçSerisi::kenar_çatal_yüzdesi, kenar_çatal_oranı)',
    testler: ['tree_fixture_testleri::yedi_resmi_tree_fixture_seçeneklerini_ve_verisini_korur'],
    örnekler: ['tree-polyline'],
    veri_biçimleri: ['yüzde', '0..1 oran'],
    dallar: ['yatay-çatal', 'dikey-çatal', 'sınır-kısıtlama']
  },
  curveness: {
    api: 'src/model/seri.rs (AğaçSerisi::kenar_eğriliği); src/grafik/agac.rs (eğri_kenar_yolu)',
    testler: ['tree_fixture_testleri::yedi_resmi_tree_sahnesinin_geometrik_ozeti_kilitlidir'],
    örnekler: ['tree-basic', 'tree-radial'],
    veri_biçimleri: ['f32 0..1'],
    dallar: ['LR/RL-kontrol-noktası', 'TB/BT-kontrol-noktası', 'radyal-kontrol-noktası']
  },
  expandAndCollapse: {
    api: 'src/model/seri.rs (AğaçSerisi::genişlet_ve_daralt, düğüm_daraltmasını_değiştir); src/eylem.rs (treeExpandAndCollapse)',
    testler: ['eylem::testler::tree_expand_and_collapse_action_modeli_ve_olay_yukunu_gunceller'],
    örnekler: ['tree-basic', 'tree-radial'],
    veri_biçimleri: ['bool', 'action payload dataIndex/series seçicisi'],
    dallar: ['etkin', 'kapalı', 'yaprak-değişmez', 'model+olay-yükü']
  },
  initialTreeDepth: {
    api: 'src/model/seri.rs (AğaçSerisi::ilk_ağaç_derinliği); src/grafik/agac.rs (düğüm_açık_mı)',
    testler: ['grafik::agac::testler::ilk_derinlik_ve_açık_daraltılmış_yaması_görünür_alt_soyu_belirler'],
    örnekler: ['tree-basic', 'tree-polyline', 'tree-radial'],
    veri_biçimleri: ['isize', 'negatif=tümü-açık'],
    dallar: ['öntanım-2', 'açık-derinlik', 'negatif', 'düğüm-yaması-önceliği']
  },
  leaves: {
    api: 'src/model/seri.rs (AğaçSerisi::yaprak_etiketi); src/grafik/agac.rs (görünür uç state mirası)',
    testler: ['tree_fixture_testleri::yedi_resmi_tree_fixture_seçeneklerini_ve_verisini_korur'],
    örnekler: ['tree-basic', 'tree-legend', 'tree-polyline', 'tree-vertical'],
    veri_biçimleri: ['EtiketYaması', 'TreeSeriesLeavesOption state mirası'],
    dallar: ['gerçek-yaprak', 'daraltılmış-dal-görünür-uç', 'normal/emphasis/blur/select']
  },
  itemStyle: {
    api: 'src/model/seri.rs ve src/model/agac.rs (normal/emphasis/blur/select öğe stilleri)',
    testler: [
      'model::agac::testler::tree_dugumu_resmi_veri_sembol_ve_durum_alanlarini_korur',
      'tree_fixture_testleri::yedi_resmi_tree_sahnesinin_geometrik_ozeti_kilitlidir'
    ],
    örnekler: TREE_ÖRNEKLERİ,
    veri_biçimleri: ['ÖğeStili', 'düğüm/seri durum yaması'],
    dallar: ['renk', 'kenarlık', 'opaklık', 'gölge', 'normal/emphasis/blur/select']
  },
  lineStyle: {
    api: 'src/model/seri.rs ve src/model/agac.rs (normal/emphasis/blur/select çizgi stilleri)',
    testler: [
      'model::agac::testler::tree_dugumu_resmi_veri_sembol_ve_durum_alanlarini_korur',
      'tree_fixture_testleri::yedi_resmi_tree_sahnesinin_geometrik_ozeti_kilitlidir'
    ],
    örnekler: TREE_ÖRNEKLERİ,
    veri_biçimleri: ['ÇizgiStili', 'curveness'],
    dallar: ['seri-varsayılanı', 'çocuk-kenar-yaması', 'normal/emphasis/blur/select']
  },
  label: {
    api: 'src/model/seri.rs ve src/model/agac.rs (etiket/durum etiketleri); src/grafik/agac.rs (etiket_geometrisi)',
    testler: [
      'model::agac::testler::tree_dugumu_resmi_veri_sembol_ve_durum_alanlarini_korur',
      'tree_fixture_testleri::yedi_resmi_tree_sahnesinin_geometrik_ozeti_kilitlidir'
    ],
    örnekler: TREE_ÖRNEKLERİ,
    veri_biçimleri: ['Etiket', 'EtiketYaması'],
    dallar: ['dört-dik-yön', 'radyal-otomatik-döndürme', 'açık/yaprak', 'durumlar']
  },
  link: {
    api: 'src/model/agac.rs (AğaçDüğümü::bağlantı)',
    testler: ['model::agac::testler::tree_dugumu_resmi_veri_sembol_ve_durum_alanlarini_korur'],
    örnekler: ['tree-basic'],
    veri_biçimleri: ['Option<String>'],
    dallar: ['yok', 'URL metadata']
  },
  target: {
    api: 'src/model/agac.rs (AğaçDüğümü::hedef)',
    testler: ['model::agac::testler::tree_dugumu_resmi_veri_sembol_ve_durum_alanlarini_korur'],
    örnekler: ['tree-basic'],
    veri_biçimleri: ['Option<String>'],
    dallar: ['yok', 'bağlantı-hedefi metadata']
  }
});

const TREEMAP_ÖRNEKLERİ = Object.freeze([
  'treemap-sunburst-transition', 'treemap-disk', 'treemap-drill-down',
  'treemap-obama', 'treemap-show-parent', 'treemap-simple', 'treemap-visual'
]);
const TREEMAP_FIXTURE_TESTİ =
  'treemap_fixture_testleri::yedi_resmi_treemap_fixture_seceneklerini_ve_verisini_korur';
const TREEMAP_SAHNE_TESTİ =
  'treemap_fixture_testleri::yedi_resmi_treemap_sahnesi_tum_gorunur_hucreleri_korur';

const TREEMAP_SERIES_OPTION_KANITI = Object.freeze({
  type: {
    api: 'src/model/seri.rs (Seri::AğaçHaritası, AğaçHaritasıSerisi)',
    testler: [TREEMAP_FIXTURE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ["sabit 'treemap'"], dallar: ['kayıtlı-seri', 'boyama', 'isabet/tooltip']
  },
  data: {
    api: 'src/model/seri.rs (AğaçHaritasıSerisi::kökler); src/model/agac.rs (AğaçDüğümü)',
    testler: [TREEMAP_FIXTURE_TESTİ, TREEMAP_SAHNE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ['number', 'number[]', 'TreemapSeriesNodeItemOption[]'],
    dallar: ['sanal-kök', 'preorder-dataIndex', 'çok-boyutlu-değer', 'iç-düğüm-toplamı']
  },
  children: {
    api: 'src/model/agac.rs (AğaçDüğümü::çocuklar, dal)',
    testler: [TREEMAP_FIXTURE_TESTİ, TREEMAP_SAHNE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ['Vec<AğaçDüğümü>'], dallar: ['yaprak', 'dal', 'özyinelemeli-alt-soy']
  },
  id: {
    api: 'src/model/agac.rs (AğaçDüğümü::kimlik); src/model/seri.rs (AğaçHaritasıSerisi::kimlik)',
    testler: ['eylem::testler::treemap_dort_view_actioni_kok_hedef_ve_root_rect_durumunu_korur'],
    örnekler: ['treemap-sunburst-transition'], veri_biçimleri: ['String'],
    dallar: ['seri-kimliği', 'düğüm-kimliği', 'targetNodeId', 'ad-yedeği']
  },
  name: {
    api: 'src/model/agac.rs (AğaçDüğümü::ad); src/model/seri.rs (AğaçHaritasıSerisi::ad)',
    testler: [TREEMAP_FIXTURE_TESTİ, TREEMAP_SAHNE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ['String'], dallar: ['etiket', 'tooltip', 'breadcrumb', 'seri-seçici']
  },
  value: {
    api: 'src/model/agac.rs (değer, değerler, etkin_değer, görsel_değer)',
    testler: [TREEMAP_FIXTURE_TESTİ, TREEMAP_SAHNE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ['number', 'number[]', 'null boyut'],
    dallar: ['alan-boyutu', 'görsel-boyut', 'iç-düğüm-toplamı', 'NaN-süzme']
  },
  sort: {
    api: 'src/model/agac.rs (AğaçHaritasıSırası); src/grafik/agac_haritasi.rs (squarify sırası)',
    testler: ['grafik::agac_haritasi::testler::squarify_alan_toplamini_korur_ve_sabit_siralidir', TREEMAP_SAHNE_TESTİ],
    örnekler: TREEMAP_ÖRNEKLERİ, veri_biçimleri: ['true/desc', 'asc', 'false/veri'],
    dallar: ['azalan', 'artan', 'ham-sıra', 'eşit-dataIndex']
  },
  clipWindow: {
    api: 'src/model/agac.rs (AğaçHaritasıKırpmaPenceresi); src/grafik/agac_haritasi.rs (kırpılı çizim/isabet)',
    testler: ['grafik::agac_haritasi::testler::root_rect_donusumu_scale_limit_ve_clip_isabetini_birlikte_korur'],
    örnekler: TREEMAP_ÖRNEKLERİ, veri_biçimleri: ['origin', 'fullscreen'],
    dallar: ['seri-kutusu', 'tam-tuval', 'roam-isabeti']
  },
  squareRatio: {
    api: 'src/model/seri.rs (kare_oranı); src/grafik/agac_haritasi.rs (en_kötü_oran/kareselleştir)',
    testler: ['grafik::agac_haritasi::testler::squarify_alan_toplamini_korur_ve_sabit_siralidir', TREEMAP_SAHNE_TESTİ],
    örnekler: TREEMAP_ÖRNEKLERİ, veri_biçimleri: ['f32'],
    dallar: ['altın-oran-varsayılanı', 'özel-oran', 'f64-ara-geometri']
  },
  leafDepth: {
    api: 'src/model/seri.rs (yaprak_derinliği); src/grafik/agac_haritasi.rs (view-root göreli derinlik/isLeafRoot)',
    testler: ['grafik::agac_haritasi::testler::leaf_depth_view_root_degistiginde_yeniden_sifirdan_sayilir', TREEMAP_SAHNE_TESTİ],
    örnekler: ['treemap-drill-down'], veri_biçimleri: ['Option<usize>'],
    dallar: ['sınırsız', 'view-root göreli', 'visibleMin sonrası leafRoot']
  },
  drillDownIcon: {
    api: 'src/model/seri.rs (inme_simgesi); src/grafik/agac_haritasi.rs (etiket_metni)',
    testler: [TREEMAP_SAHNE_TESTİ], örnekler: ['treemap-drill-down'], veri_biçimleri: ['String'],
    dallar: ['varsayılan ▶', 'özel metin', 'boş simge', 'yalnız isLeafRoot']
  },
  zoomToNodeRatio: {
    api: 'src/model/seri.rs (düğüme_yakınlaştırma_oranı); src/cizim/pencere.rs (nodeClick hedef görünümü)',
    testler: ['eylem::testler::treemap_dort_view_actioni_kok_hedef_ve_root_rect_durumunu_korur'],
    örnekler: ['treemap-simple', 'treemap-drill-down'], veri_biçimleri: ['f32'],
    dallar: ['tıklama', 'treemapZoomToNode', 'scaleLimit-kıstırma']
  },
  nodeClick: {
    api: 'src/model/agac.rs (AğaçHaritasıDüğümTıklaması); src/cizim/pencere.rs (zoom/link/kapalı)',
    testler: ['eylem::testler::treemap_dort_view_actioni_kok_hedef_ve_root_rect_durumunu_korur'],
    örnekler: ['treemap-sunburst-transition', 'treemap-drill-down'],
    veri_biçimleri: ['zoomToNode', 'link', 'false'],
    dallar: ['leafRoot-rootToNode', 'zoomToNode', 'host-güvenli bağlantı olayı', 'kapalı']
  },
  breadcrumb: {
    api: 'src/model/agac.rs (AğaçHaritasıKırıntısı); src/grafik/agac_haritasi.rs (kırıntıları_çiz)',
    testler: [TREEMAP_FIXTURE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ['box layout', 'ÖğeStili', 'YazıStili'],
    dallar: ['show', 'left/right/top/bottom', 'emptyItemWidth', 'vurgu', 'kök-yukarı-tıklama']
  },
  levels: {
    api: 'src/model/agac.rs (AğaçHaritasıSeviyesi); src/grafik/agac_haritasi.rs (derinlik_katmanı)',
    testler: ['grafik::agac_haritasi::testler::visible_min_leaf_depth_ve_seviye_gorseli_uygulanir', TREEMAP_SAHNE_TESTİ],
    örnekler: ['treemap-disk', 'treemap-drill-down', 'treemap-obama', 'treemap-visual'],
    veri_biçimleri: ['Vec<AğaçHaritasıSeviyesi>'],
    dallar: ['seri→seviye→düğüm mirası', 'normal/emphasis/blur/select modeli']
  },
  color: {
    api: 'src/model/agac.rs (AğaçHaritasıGörseli::renkler/renk_yok); src/grafik/agac_haritasi.rs (çocuk_renkleri)',
    testler: ['grafik::agac_haritasi::testler::visible_min_leaf_depth_ve_seviye_gorseli_uygulanir', TREEMAP_SAHNE_TESTİ],
    örnekler: ['treemap-obama', 'treemap-visual'], veri_biçimleri: ['ColorString[]', 'none'],
    dallar: ['global-palet', 'seviye-aralığı', 'parent-designated', 'none']
  },
  colorAlpha: {
    api: 'src/model/agac.rs (AğaçHaritasıGörseli::alfa_aralığı, AğaçHaritasıÖğeStili::renk_alfası)',
    testler: ['grafik::agac_haritasi::testler::item_style_renk_alpha_ve_doygunluk_designated_rengi_degistirir'],
    örnekler: ['treemap-visual'], veri_biçimleri: ['[number, number]', 'number', 'none'],
    dallar: ['doğrusal-aralık', 'itemStyle-mutlak-alfa', 'miras']
  },
  colorSaturation: {
    api: 'src/model/agac.rs (AğaçHaritasıGörseli::doygunluk_aralığı, AğaçHaritasıÖğeStili::renk_doygunluğu)',
    testler: ['grafik::agac_haritasi::testler::item_style_renk_alpha_ve_doygunluk_designated_rengi_degistirir', TREEMAP_SAHNE_TESTİ],
    örnekler: ['treemap-disk', 'treemap-drill-down'], veri_biçimleri: ['[number, number]', 'number', 'none'],
    dallar: ['doğrusal-aralık', 'zrender-HSL-açıklığı', 'itemStyle', 'miras']
  },
  colorMappingBy: {
    api: 'src/model/agac.rs (AğaçHaritasıRenkEşlemesi); src/grafik/agac_haritasi.rs (KimlikSıraları)',
    testler: [TREEMAP_SAHNE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ['value', 'index', 'id'], dallar: ['doğrusal-değer', 'kardeş-sırası', 'kararlı-kimlik']
  },
  visualDimension: {
    api: 'src/model/agac.rs (AğaçHaritasıGörselBoyutu, AğaçDüğümü::görsel_değer)',
    testler: [TREEMAP_FIXTURE_TESTİ, TREEMAP_SAHNE_TESTİ], örnekler: ['treemap-visual'],
    veri_biçimleri: ['number', 'string/value'], dallar: ['sıra', 'ad', 'çok-boyutlu-value']
  },
  visualMin: {
    api: 'src/model/agac.rs (AğaçHaritasıGörseli::en_az); src/grafik/agac_haritasi.rs (çocuk_renkleri kapsamı)',
    testler: [TREEMAP_SAHNE_TESTİ], örnekler: ['treemap-visual'], veri_biçimleri: ['f64'],
    dallar: ['ham-kardeş-kapsamı', 'açık-alt-sınır']
  },
  visualMax: {
    api: 'src/model/agac.rs (AğaçHaritasıGörseli::en_çok); src/grafik/agac_haritasi.rs (çocuk_renkleri kapsamı)',
    testler: [TREEMAP_SAHNE_TESTİ], örnekler: ['treemap-visual'], veri_biçimleri: ['f64'],
    dallar: ['ham-kardeş-kapsamı', 'açık-üst-sınır']
  },
  visibleMin: {
    api: 'src/model/agac.rs (görünür_en_az/görünür_eşiği_kapalı); src/grafik/agac_haritasi.rs (filterByThreshold portu)',
    testler: ['grafik::agac_haritasi::testler::visible_min_kapatma_kalitimi_sifir_degerle_ezer', TREEMAP_SAHNE_TESTİ],
    örnekler: ['treemap-disk', 'treemap-drill-down'], veri_biçimleri: ['f32 piksel²'],
    dallar: ['asc/desc', 'sort=false kapalı', 'sıfırla açık-kapatma', 'leafRoot sonrası']
  },
  childrenVisibleMin: {
    api: 'src/model/agac.rs (çocuk_görünür_en_az); src/grafik/agac_haritasi.rs (alt-soy gizleme)',
    testler: ['grafik::agac_haritasi::testler::visible_min_leaf_depth_ve_seviye_gorseli_uygulanir', TREEMAP_SAHNE_TESTİ],
    örnekler: TREEMAP_ÖRNEKLERİ, veri_biçimleri: ['f32 piksel²'],
    dallar: ['düğüm-iç-alanı', 'leafDepth önceliği', 'görünür-yaprak']
  },
  itemStyle: {
    api: 'src/model/agac.rs (AğaçHaritasıÖğeStili); src/grafik/agac_haritasi.rs (background/content iki katman)',
    testler: ['grafik::agac_haritasi::testler::item_style_renk_alpha_ve_doygunluk_designated_rengi_degistirir', TREEMAP_SAHNE_TESTİ],
    örnekler: TREEMAP_ÖRNEKLERİ, veri_biçimleri: ['ÖğeStili', 'AğaçHaritasıÖğeStili'],
    dallar: ['dolgu/gradyan/desen', 'border/gap/radius', 'opacity/shadow', 'seviye/düğüm/durum-yaması']
  },
  borderRadius: {
    api: 'src/model/agac.rs (AğaçHaritasıÖğeStili::kenarlık_yarıçapı)',
    testler: [TREEMAP_SAHNE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ['number', '[number;4]'], dallar: ['background', 'leaf-content', 'durum-yaması']
  },
  borderColorSaturation: {
    api: 'src/model/agac.rs (kenarlık_rengi_doygunluğu); src/grafik/agac_haritasi.rs (açıklık_ile)',
    testler: [TREEMAP_SAHNE_TESTİ], örnekler: ['treemap-disk', 'treemap-drill-down'],
    veri_biçimleri: ['f32'], dallar: ['dolgu-türetimi', 'borderColor önceliği', 'seviye/düğüm']
  },
  gapWidth: {
    api: 'src/model/agac.rs (boşluk_genişliği); src/grafik/agac_haritasi.rs (yarım-gap squarify)',
    testler: [TREEMAP_SAHNE_TESTİ], örnekler: ['treemap-disk', 'treemap-drill-down'],
    veri_biçimleri: ['f32'], dallar: ['yarım-boşluk', 'parent içerik kutusu', 'seviye/düğüm']
  },
  label: {
    api: 'src/model/seri.rs ve src/model/agac.rs (etiket/durum yamaları); src/grafik/agac_haritasi.rs (etiketi_çiz)',
    testler: [TREEMAP_FIXTURE_TESTİ, TREEMAP_SAHNE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ['Etiket', 'EtiketYaması', 'rich formatter'],
    dallar: ['inside konumları', 'padding/overflow', 'rich text', 'drillDownIcon', 'emphasis']
  },
  upperLabel: {
    api: 'src/model/seri.rs ve src/model/agac.rs (üst_etiket); src/grafik/agac_haritasi.rs (üst şerit)',
    testler: [TREEMAP_FIXTURE_TESTİ, TREEMAP_SAHNE_TESTİ],
    örnekler: ['treemap-disk', 'treemap-obama', 'treemap-show-parent', 'treemap-visual'],
    veri_biçimleri: ['Etiket', 'EtiketYaması'], dallar: ['parent-only', 'height', 'middle', 'truncate', 'emphasis']
  },
  formatter: {
    api: 'src/model/stil.rs (Biçimleyici); src/grafik/agac_haritasi.rs (etiket_metni)',
    testler: [TREEMAP_FIXTURE_TESTİ], örnekler: ['treemap-obama', 'treemap-visual'],
    veri_biçimleri: ['şablon', 'tipli Rust işlevi', 'rich şablon'],
    dallar: ['{a}/{b}/{c}', 'normal', 'upperLabel', 'rich']
  },
  textStyle: {
    api: 'src/model/agac.rs (AğaçHaritasıKırıntısı::yazı); src/grafik/agac_haritasi.rs (kırıntıları_çiz)',
    testler: [TREEMAP_FIXTURE_TESTİ], örnekler: TREEMAP_ÖRNEKLERİ,
    veri_biçimleri: ['YazıStili'], dallar: ['breadcrumb normal', 'renk', 'boyut', 'kalınlık']
  }
});

const SUNBURST_ÖRNEKLERİ = Object.freeze([
  'sunburst-simple', 'sunburst-borderRadius', 'sunburst-label-rotate',
  'sunburst-monochrome', 'sunburst-visualMap', 'sunburst-drink', 'sunburst-book'
]);
const SUNBURST_FIXTURE_TESTİ =
  'sunburst_fixture_testleri::yedi_resmi_sunburst_fixture_seceneklerini_ve_verisini_korur';
const SUNBURST_SAHNE_TESTİ =
  'sunburst_fixture_testleri::yedi_resmi_sunburst_sahnesi_tum_dilimleri_korur';
const SUNBURST_ACTION_TESTİ =
  'eylem::testler::sunburst_root_ve_eski_vurgu_actionlari_resmi_yuku_korur';

const SUNBURST_SERIES_OPTION_KANITI = Object.freeze({
  type: {
    api: 'src/model/seri.rs (Seri::GüneşPatlaması, GüneşPatlamasıSerisi)',
    testler: [SUNBURST_FIXTURE_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ["sabit 'sunburst'"], dallar: ['kayıtlı-seri', 'boyama', 'isabet/tooltip']
  },
  data: {
    api: 'src/model/seri.rs (GüneşPatlamasıSerisi::kökler); src/model/agac.rs (AğaçDüğümü)',
    testler: [SUNBURST_FIXTURE_TESTİ, SUNBURST_SAHNE_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['number', 'number[]', 'SunburstSeriesNodeItemOption[]'],
    dallar: ['sanal-kök', 'preorder-dataIndex', 'iç-düğüm-toplamı', 'negatif-sıfırlama']
  },
  children: {
    api: 'src/model/agac.rs (AğaçDüğümü::çocuklar, dal)',
    testler: [SUNBURST_FIXTURE_TESTİ, SUNBURST_SAHNE_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['Vec<AğaçDüğümü>'], dallar: ['yaprak', 'dal', 'özyinelemeli-alt-soy']
  },
  radius: {
    api: 'src/model/seri.rs (GüneşPatlamasıSerisi::halka); src/grafik/gunes.rs (güneş_patlaması_dilimleri)',
    testler: [SUNBURST_FIXTURE_TESTİ, SUNBURST_SAHNE_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['number', 'percent', '[inner, outer]'], dallar: ['disk', 'halka', 'seviye-override']
  },
  r0: {
    api: 'src/model/agac.rs (GüneşPatlamasıSeviyesi::yarıçap)',
    testler: [SUNBURST_SAHNE_TESTİ], örnekler: ['sunburst-drink', 'sunburst-book'],
    veri_biçimleri: ['number', 'percent'], dallar: ['otomatik-katman', 'mutlak-seviye-içi']
  },
  r: {
    api: 'src/model/agac.rs (GüneşPatlamasıSeviyesi::yarıçap)',
    testler: [SUNBURST_SAHNE_TESTİ], örnekler: ['sunburst-drink', 'sunburst-book'],
    veri_biçimleri: ['number', 'percent'], dallar: ['otomatik-katman', 'mutlak-seviye-dışı']
  },
  clockwise: {
    api: 'src/model/seri.rs (GüneşPatlamasıSerisi::saat_yönünde)',
    testler: ['grafik::gunes::testler::resmi_birim_aciyi_ve_on_sirali_indeksleri_korur', SUNBURST_SAHNE_TESTİ],
    örnekler: SUNBURST_ÖRNEKLERİ, veri_biçimleri: ['bool'], dallar: ['saat-yönü', 'ters-yön']
  },
  startAngle: {
    api: 'src/model/seri.rs (GüneşPatlamasıSerisi::başlangıç_açısı)',
    testler: ['grafik::gunes::testler::resmi_birim_aciyi_ve_on_sirali_indeksleri_korur', SUNBURST_SAHNE_TESTİ],
    örnekler: SUNBURST_ÖRNEKLERİ, veri_biçimleri: ['degree'], dallar: ['90-varsayılanı', 'özel-açı']
  },
  minAngle: {
    api: 'src/model/seri.rs (GüneşPatlamasıSerisi::en_küçük_açı); src/grafik/gunes.rs (minAngle dağıtımı)',
    testler: ['grafik::gunes::testler::resmi_birim_aciyi_ve_on_sirali_indeksleri_korur'],
    örnekler: SUNBURST_ÖRNEKLERİ, veri_biçimleri: ['degree'], dallar: ['sıfır', 'pozitif-alt-sınır']
  },
  stillShowZeroSum: {
    api: 'src/model/seri.rs (GüneşPatlamasıSerisi::sıfır_toplamı_göster)',
    testler: ['grafik::gunes::testler::resmi_birim_aciyi_ve_on_sirali_indeksleri_korur'],
    örnekler: SUNBURST_ÖRNEKLERİ, veri_biçimleri: ['bool'], dallar: ['eşit-açı', 'boş-açı']
  },
  renderLabelForZeroData: {
    api: 'src/model/seri.rs (sıfır_veri_etiketini_göster); src/grafik/gunes.rs (sıfır sektör/etiket ayrımı)',
    testler: [SUNBURST_SAHNE_TESTİ], örnekler: ['sunburst-book'], veri_biçimleri: ['bool'],
    dallar: ['sıfır-sektör-korunur', 'etiket-gizli', 'etiket-açık']
  },
  sort: {
    api: 'src/model/agac.rs (GüneşPatlamasıSırası); src/model/seri.rs (GüneşPatlamasıSıralamaİşlevi)',
    testler: [SUNBURST_FIXTURE_TESTİ, SUNBURST_SAHNE_TESTİ],
    örnekler: ['sunburst-simple', 'sunburst-monochrome', 'sunburst-label-rotate', 'sunburst-drink', 'sunburst-book'],
    veri_biçimleri: ['desc', 'asc', 'null/undefined', 'tipli Rust callback'],
    dallar: ['değer-azalan', 'değer-artan', 'ham-sıra', 'depth/dataIndex callback']
  },
  nodeClick: {
    api: 'src/model/agac.rs (GüneşPatlamasıDüğümTıklaması); src/cizim/pencere.rs (root/link/false)',
    testler: [SUNBURST_ACTION_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['rootToNode', 'link', 'false'],
    dallar: ['düğümü-kök-yap', 'merkez-rollup', 'host-güvenli-bağlantı', 'kapalı']
  },
  link: {
    api: 'src/model/agac.rs (AğaçDüğümü::bağlantı)', testler: [SUNBURST_ACTION_TESTİ],
    örnekler: SUNBURST_ÖRNEKLERİ, veri_biçimleri: ['Option<String>'], dallar: ['yok', 'URL metadata']
  },
  target: {
    api: 'src/model/agac.rs (AğaçDüğümü::hedef)', testler: [SUNBURST_ACTION_TESTİ],
    örnekler: SUNBURST_ÖRNEKLERİ, veri_biçimleri: ['Option<String>'], dallar: ['_blank-varsayılanı', 'özel-hedef']
  },
  cursor: {
    api: 'src/model/agac.rs (AğaçDüğümü::imleç); src/cizim/pencere.rs (CSS→GPUI imleç eşlemesi)',
    testler: [SUNBURST_FIXTURE_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['String'], dallar: ['pointer', 'default', 'bilinmeyen-güvenli-düşüş']
  },
  collapsed: {
    api: 'src/model/agac.rs (AğaçDüğümü::daraltılmış; resmî ortak Tree metadata alanı)',
    testler: [SUNBURST_FIXTURE_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['Option<bool>'], dallar: ['metadata-koruma', 'Sunburst yerleşiminde resmî olarak etkisiz']
  },
  itemStyle: {
    api: 'src/model/agac.rs (GüneşPatlamasıÖğeStili); src/grafik/gunes.rs (seri→level→node→state kalıtımı)',
    testler: [SUNBURST_FIXTURE_TESTİ, SUNBURST_SAHNE_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['Dolgu', 'Renk', 'number/percent cornerRadius'],
    dallar: ['fill/border', 'opacity', 'shadow', 'solid/dashed/dotted', 'normal/emphasis/blur/select']
  },
  borderRadius: {
    api: 'src/model/agac.rs (GüneşPatlamasıKöşeYarıçapı); src/cizim/yuzey.rs (yuvarlatılmış_dilim_yolu)',
    testler: ['grafik::gunes::testler::mutlak_level_yaricapi_ve_yuzde_kose_yaricapi_korunur', SUNBURST_SAHNE_TESTİ],
    örnekler: ['sunburst-borderRadius'], veri_biçimleri: ['number', '[inner,outer]', '[4]', 'percent'],
    dallar: ['dört-sector-köşesi', 'halka-kalınlığına-yüzde', 'dar-açı-kıstırma']
  },
  label: {
    api: 'src/model/stil.rs (Etiket/EtiketYaması); src/grafik/gunes.rs (etiket_geometrisi)',
    testler: [SUNBURST_FIXTURE_TESTİ, SUNBURST_SAHNE_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['Etiket', 'EtiketYaması', 'treePathInfo callback'],
    dallar: ['inside/outside', 'radial/tangential/degree', 'align/distance', 'minAngle', 'durumlar']
  },
  rotate: {
    api: 'src/model/stil.rs (EtiketDöndürme); src/grafik/gunes.rs (etiket_geometrisi)',
    testler: [SUNBURST_SAHNE_TESTİ], örnekler: ['sunburst-simple', 'sunburst-label-rotate', 'sunburst-drink', 'sunburst-book'],
    veri_biçimleri: ['radial', 'tangential', 'number'], dallar: ['otomatik-flip', 'dış-etiket', 'derece']
  },
  position: {
    api: 'src/model/stil.rs (EtiketKonumu); src/grafik/gunes.rs (etiket_geometrisi)',
    testler: [SUNBURST_SAHNE_TESTİ], örnekler: ['sunburst-drink', 'sunburst-book'],
    veri_biçimleri: ['inside', 'outside'], dallar: ['halka-ortası', 'dış-yarıçap+distance']
  },
  levels: {
    api: 'src/model/agac.rs (GüneşPatlamasıSeviyesi); src/grafik/gunes.rs (seviye_uygula)',
    testler: [SUNBURST_FIXTURE_TESTİ, SUNBURST_SAHNE_TESTİ],
    örnekler: ['sunburst-label-rotate', 'sunburst-drink', 'sunburst-book'],
    veri_biçimleri: ['Vec<GüneşPatlamasıSeviyesi>'],
    dallar: ['radius', 'itemStyle', 'label', 'emphasis/blur/select', 'seri→level→node kalıtımı']
  },
  silent: {
    api: 'src/model/seri.rs (GüneşPatlamasıSerisi::sessiz); src/grafik/gunes.rs (isabet kaydı)',
    testler: [SUNBURST_FIXTURE_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['bool'], dallar: ['boya-korunur', 'isabet-kapalı']
  },
  animationType: {
    api: 'src/model/agac.rs (GüneşPatlamasıAnimasyonTürü); resmî 6.1 çalışma zamanı bu bildirimi okumaz',
    testler: ['grafik::gunes::testler::animation_type_iki_resmi_degeri_modelde_korur'],
    örnekler: SUNBURST_ÖRNEKLERİ, veri_biçimleri: ['expansion', 'scale'],
    dallar: ['iki-değer-modelde-korunur', 'resmî-runtime-etkisiz']
  },
  highlight: {
    api: 'src/eylem.rs (sunburstHighlight/sunburstUnhighlight); src/grafik/gunes.rs (odak/blur durumları)',
    testler: [SUNBURST_ACTION_TESTİ], örnekler: SUNBURST_ÖRNEKLERİ,
    veri_biçimleri: ['dispatchAction payload'], dallar: ['highlight→highlight', 'unhighlight→downplay', 'ancestor/descendant/relative']
  }
});

const SANKEY_ÖRNEKLERİ = Object.freeze([
  'sankey-energy', 'sankey-itemstyle', 'sankey-levels',
  'sankey-nodeAlign-left', 'sankey-nodeAlign-right', 'sankey-simple', 'sankey-vertical'
]);
const SANKEY_FIXTURE_TESTİ =
  'sankey_fixture_testleri::yedi_resmi_sankey_fixture_tum_dugum_bag_ve_seceneklerini_korur';
const SANKEY_SAHNE_TESTİ =
  'sankey_fixture_testleri::yedi_resmi_sankey_sahnesi_tum_geometri_ve_etiket_tabanlarini_korur';
const SANKEY_ACTION_TESTİ =
  'eylem::testler::sankey_drag_ve_roam_actionlari_modeli_ve_resmi_olay_yukunu_korur';
const SANKEY_YERLEŞİM_TESTİ =
  'grafik::sankey::testler::resmi_dag_deger_katman_ve_bag_kalinligini_korur';

const SANKEY_SERIES_OPTION_KANITI = Object.freeze({
  type: {
    api: 'src/model/seri.rs (Seri::Sankey); src/model/sankey.rs (SankeySerisi); src/grafik/sankey.rs (sankey_çiz)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ["sabit 'sankey'"], dallar: ['kayıtlı-seri', 'boyama', 'isabet/tooltip']
  },
  data: {
    api: 'src/model/sankey.rs (SankeyDüğümü, SankeySerisi::düğümler); src/grafik/sankey.rs (grafiği_kur)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ, SANKEY_YERLEŞİM_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['SankeyNodeItemOption[]'], dallar: ['ad/id', 'açık/değerden-türetilen value', 'ham dataIndex']
  },
  nodes: {
    api: 'src/model/sankey.rs (SankeySerisi::düğümler; data/nodes eşdeğeri)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['SankeyNodeItemOption[]'], dallar: ['data-alias', 'nodes-alias']
  },
  links: {
    api: 'src/model/sankey.rs (SankeyBağı, SankeySerisi::bağlar/ayrıntılı_bağlar); src/grafik/sankey.rs (şerit yerleşimi)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ, SANKEY_YERLEŞİM_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['SankeyEdgeItemOption[]'], dallar: ['source/target/value', 'çoklu-bağ', 'DAG-doğrulama']
  },
  edges: {
    api: 'src/model/sankey.rs (SankeySerisi::bağlar; links/edges eşdeğeri)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['SankeyEdgeItemOption[]'], dallar: ['links-alias', 'edges-alias']
  },
  color: {
    api: 'src/model/sankey.rs (SankeySerisi::renkler); src/grafik/sankey.rs (renk_eşle)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['ColorString[]'], dallar: ['seri-paleti', 'global-palet', 'değer-eşleme']
  },
  coordinateSystem: {
    api: 'src/model/sankey.rs (view/takvim/matris bağları); src/cizim/gorunum.rs (yerleşim_referansı)',
    testler: ['model::secenekler::testler::sankey_view_calendar_ve_matrix_kutu_koordinatlarini_dogrular', SANKEY_SAHNE_TESTİ],
    örnekler: SANKEY_ÖRNEKLERİ, veri_biçimleri: ["'view'", 'calendarIndex+coord', 'matrixIndex+coord'],
    dallar: ['view', 'calendar-box', 'matrix-box']
  },
  orient: {
    api: 'src/model/sankey.rs (SankeyYönü); src/grafik/sankey.rs (dikey/yatay eksen değişimi)',
    testler: ['grafik::sankey::testler::dikey_yon_ve_sag_hiza_eksenleri_degistirir', SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ],
    örnekler: ['sankey-simple', 'sankey-vertical'], veri_biçimleri: ['horizontal', 'vertical'],
    dallar: ['düğüm ekseni', 'bağ kontrol noktaları', 'etiket konumu']
  },
  nodeWidth: {
    api: 'src/model/sankey.rs (düğüm_genişliği); src/grafik/sankey.rs (düğüm_enini_ata)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['number'], dallar: ['yatay-genişlik', 'dikey-yükseklik', 'sıfır']
  },
  nodeGap: {
    api: 'src/model/sankey.rs (düğüm_boşluğu); src/grafik/sankey.rs (derinlik_grupları/çakışma_çöz)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['number'], dallar: ['derinlik-içi-boşluk', 'negatif-olmayan-doğrulama']
  },
  draggable: {
    api: 'src/model/sankey.rs (seri/düğüm sürüklenebilir); src/cizim/pencere.rs (SankeyDüğüm sürükleme); src/eylem.rs (dragNode)',
    testler: [SANKEY_ACTION_TESTİ, 'cizim::pencere::testler::sankey_ekran_yerel_donusumu_model_ve_gecici_gorunumu_tersine_cevirir'],
    örnekler: SANKEY_ÖRNEKLERİ, veri_biçimleri: ['bool', 'dragNode localX/localY'],
    dallar: ['seri-varsayılanı', 'düğüm-override', 'pan/zoom ters-dönüşümü']
  },
  layoutIterations: {
    api: 'src/model/sankey.rs (yerleşim_yinelemesi); src/grafik/sankey.rs (gevşet/çakışma yinelemesi)',
    testler: [SANKEY_YERLEŞİM_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['nonnegative integer'], dallar: ['0', 'varsayılan-32', 'ileri/geri-gevşetme']
  },
  sort: {
    api: 'src/model/sankey.rs (SankeySırası); src/grafik/sankey.rs (düğüm sırası)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['desc', 'null/veri'], dallar: ['değer-azalan', 'ham-dataIndex', 'eşit-kararlılık']
  },
  nodeAlign: {
    api: 'src/model/sankey.rs (SankeyDüğümHizası); src/grafik/sankey.rs (derinlik ata)',
    testler: ['grafik::sankey::testler::dikey_yon_ve_sag_hiza_eksenleri_degistirir', SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ],
    örnekler: ['sankey-nodeAlign-left', 'sankey-nodeAlign-right', 'sankey-simple'],
    veri_biçimleri: ['justify', 'left', 'right'], dallar: ['iki-yana', 'sol', 'sağ', 'sink-hizası']
  },
  levels: {
    api: 'src/model/sankey.rs (SankeySeviyesi); src/grafik/sankey.rs (seri→seviye→öğe stil kalıtımı)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: ['sankey-levels'],
    veri_biçimleri: ['SankeyLevelOption[]'], dallar: ['depth', 'itemStyle', 'lineStyle', 'label', 'edgeLabel']
  },
  itemStyle: {
    api: 'src/model/sankey.rs (SankeyÖğeStili); src/grafik/sankey.rs (öğe_stili_yama_uygula)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: ['sankey-itemstyle', 'sankey-levels'],
    veri_biçimleri: ['color/border/radius/opacity/shadow'],
    dallar: ['seri', 'seviye', 'düğüm', 'emphasis/blur/select', 'negatif-alan-normalizasyonu']
  },
  lineStyle: {
    api: 'src/model/sankey.rs (SankeyÇizgiStili, SankeyKenarBoyası); src/grafik/sankey.rs (bağ_dolgusu/bağ_yolu)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: ['sankey-energy', 'sankey-levels', 'sankey-simple'],
    veri_biçimleri: ['color/source/target/gradient', 'opacity', 'curveness', 'width/type/shadow'],
    dallar: ['seri', 'seviye', 'bağ', 'durum', 'yatay/dikey-gradyan']
  },
  curveness: {
    api: 'src/model/sankey.rs (SankeyÇizgiStili::eğrilik); src/grafik/sankey.rs (Bézier kontrol noktaları)',
    testler: [SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['number'], dallar: ['0', '0.5-varsayılanı', 'yatay', 'dikey']
  },
  label: {
    api: 'src/model/stil.rs (Etiket/EtiketYaması); src/grafik/sankey.rs (düğüm_etiket_geometrisi/etiketi_çiz)',
    testler: ['grafik::sankey::testler::etiket_tabani_negatif_alani_ve_yarim_kenarligi_hesaba_katar', SANKEY_SAHNE_TESTİ],
    örnekler: ['sankey-itemstyle', 'sankey-levels', 'sankey-vertical'],
    veri_biçimleri: ['Etiket', 'EtiketYaması', 'formatter/rich'],
    dallar: ['seri→seviye→düğüm', 'dört-dış/inside konumu', 'kenarlık-dahil-taban', 'font/renk/durum']
  },
  edgeLabel: {
    api: 'src/model/sankey.rs (kenar_etiketi); src/grafik/sankey.rs (bağ orta-nokta etiketi)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['Etiket', 'EtiketYaması'], dallar: ['seri', 'seviye', 'bağ', 'durum', 'inside']
  },
  emphasis: {
    api: 'src/model/sankey.rs (SankeyDurumu/SankeyVurguOdağı); src/grafik/sankey.rs (odak_kümeleri)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['self', 'series', 'adjacency', 'trajectory'],
    dallar: ['düğüm', 'bağ', 'item/line/label/edgeLabel', 'blur-komşu-dışı']
  },
  focusNodeAdjacency: {
    api: 'src/model/sankey.rs (SankeyKomşulukOdağı; seri/düğüm/bağ); src/grafik/sankey.rs (odak_kümeleri)',
    testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['false', 'inEdges', 'outEdges', 'allEdges'],
    dallar: ['seri', 'düğüm', 'bağ', 'eski-option→emphasis eşlemesi']
  },
  id: {
    api: 'src/model/sankey.rs (SankeySerisi::kimlik, SankeyDüğümü::kimlik); src/eylem.rs (seriesId seçici)',
    testler: [SANKEY_ACTION_TESTİ, SANKEY_FIXTURE_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['String'], dallar: ['seri-kimliği', 'düğüm-kimliği', 'ad-yedeği']
  },
  depth: {
    api: 'src/model/sankey.rs (SankeyDüğümü::derinlik, SankeySeviyesi::derinlik); src/grafik/sankey.rs (derinlik ata)',
    testler: [SANKEY_YERLEŞİM_TESTİ, SANKEY_SAHNE_TESTİ], örnekler: ['sankey-levels'],
    veri_biçimleri: ['nonnegative integer'], dallar: ['DAG-türetilen', 'açık-düğüm-depth', 'level-eşleme']
  },
  localX: {
    api: 'src/model/sankey.rs (SankeyDüğümü::yerel_x/düğüm_konumunu_ayarla); src/eylem.rs (dragNode)',
    testler: [SANKEY_ACTION_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['number'], dallar: ['layout-yedeği', 'dragNode', 'pan/zoom ters-dönüşümü']
  },
  localY: {
    api: 'src/model/sankey.rs (SankeyDüğümü::yerel_y/düğüm_konumunu_ayarla); src/eylem.rs (dragNode)',
    testler: [SANKEY_ACTION_TESTİ], örnekler: SANKEY_ÖRNEKLERİ,
    veri_biçimleri: ['number'], dallar: ['layout-yedeği', 'dragNode', 'pan/zoom ters-dönüşümü']
  }
});

const CHORD_ÖRNEKLERİ = Object.freeze([
  'chord-simple', 'chord-minAngle', 'chord-lineStyle-color', 'chord-style'
]);
const CHORD_FIXTURE_TESTİ =
  'kiriş_fixture_testleri::dört_resmi_chord_fixture_tum_dugum_bag_ve_seceneklerini_korur';
const CHORD_SAHNE_TESTİ =
  'kiriş_fixture_testleri::dört_resmi_chord_sahnesi_tum_sektor_serit_ve_etiket_tabanlarini_korur';
const CHORD_YERLEŞİM_TESTİ =
  'grafik::kiris::testler::resmi_baslangic_yon_ve_bag_birikimi_korunur';
const CHORD_İSABET_TESTİ =
  'grafik::kiris::testler::serit_boyali_alani_olay_ve_tooltip_isabeti_uretir';
const CHORD_SEÇENEK_TESTİ =
  'grafik::kiris::testler::acik_curveness_kenar_etiketi_ve_disabled_vurgu_dallari_calisir';
const CHORD_GÖSTERGE_TESTİ =
  'cizim::gorunum::yakınlaştırma_yönü_testleri::kiris_gosterge_filtresi_dugumu_ve_bagli_seritleri_yerlesimden_cikarir';
const CHORD_TOOLTIP_TESTİ =
  'cizim::gorunum::yakınlaştırma_yönü_testleri::kiris_seri_tooltipi_kok_tooltip_olmadan_seridi_gosterir';

const CHORD_SERIES_OPTION_KANITI = Object.freeze({
  type: {
    api: 'src/model/seri.rs (Seri::Kiriş/From<KirişSerisi>); src/model/kiris.rs (KirişSerisi); src/grafik/kiris.rs (kiriş_çiz)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ["sabit 'chord'"], dallar: ['kayıtlı-seri', 'boyama', 'isabet/tooltip']
  },
  coordinateSystem: {
    api: "src/model/kiris.rs (KirişSerisi yalnız coordinateSystem='none' değişmezini kabul eder); src/grafik/kiris.rs (kiriş_alanı)",
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ["'none'"], dallar: ['none', 'box-layout görünüm alanı']
  },
  data: {
    api: 'src/model/kiris.rs (KirişDüğümü, KirişSerisi::düğümler); src/grafik/kiris.rs (düğümleri_ve_bağları_kur)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ, CHORD_YERLEŞİM_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['KirişDüğümü[]', 'string shorthand', 'number/number[] value'], dallar: ['id/name', 'açık/türetilen-value', 'ham dataIndex']
  },
  nodes: {
    api: 'src/model/kiris.rs (KirişSerisi::düğümler; data/nodes eşdeğeri)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['KirişDüğümü[]'], dallar: ['data-alias', 'nodes-alias']
  },
  links: {
    api: 'src/model/kiris.rs (KirişBağı, bağlar/ayrıntılı_bağlar); src/grafik/kiris.rs (iki-uç açı istifi)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ, CHORD_YERLEŞİM_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['KirişBağı[]', '(source,target,value)[]'], dallar: ['source/target/value', 'self-edge', 'çoklu-bağ']
  },
  edges: {
    api: 'src/model/kiris.rs (KirişSerisi::bağlar; links/edges eşdeğeri)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['KirişBağı[]'], dallar: ['links-alias', 'edges-alias']
  },
  id: {
    api: 'src/model/kiris.rs (KirişSerisi::kimlik, KirişDüğümü::kimlik); src/model/seri.rs (Seri::kimlik)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['String'], dallar: ['seri-id', 'düğüm-id', 'name-yedeği', 'link endpoint']
  },
  name: {
    api: 'src/model/kiris.rs (KirişDüğümü::ad, KirişSerisi::ad); src/grafik/kiris.rs (etiket/tooltip)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['String'], dallar: ['düğüm-adı', 'seri-adı', 'legend provider']
  },
  value: {
    api: 'src/model/kiris.rs (KirişDüğümü::değer, KirişBağı::değer); src/grafik/kiris.rs (uç toplamı/max açık değer)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ, CHORD_YERLEŞİM_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['number', 'number[]', 'edge number'], dallar: ['iki-uç-toplamı', 'açık-node-max', 'allZero=1']
  },
  clockwise: {
    api: 'src/model/kiris.rs (KirişSerisi::saat_yönünde); src/grafik/kiris.rs (açıları_normalleştir/arc sweep)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ, CHORD_YERLEŞİM_TESTİ], örnekler: ['chord-simple', 'chord-style'],
    veri_biçimleri: ['bool'], dallar: ['saat-yönü', 'ters-yön', 'şerit-yayı']
  },
  startAngle: {
    api: 'src/model/kiris.rs (başlangıç_açısı); src/grafik/kiris.rs (derece→ekran-radyanı)',
    testler: [CHORD_SAHNE_TESTİ, CHORD_YERLEŞİM_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['degree'], dallar: ['90-varsayılanı', 'özel-açı']
  },
  endAngle: {
    api: 'src/model/kiris.rs (bitiş_açısı/otomatik_bitiş_açısı); src/grafik/kiris.rs (kilitli ECharts chordLayout tam-tur davranışı)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['number', 'auto'], dallar: ['API-değeri-korunur', 'resmî-runtime-tam-tur']
  },
  padAngle: {
    api: 'src/model/kiris.rs (dolgu_açısı); src/grafik/kiris.rs (çizilen düğüm sayısına göre boşluk azaltımı)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: ['chord-style'],
    veri_biçimleri: ['nonnegative degree'], dallar: ['3-varsayılanı', 'minAngle-önceliği', 'yetersiz-açı-kıstırma']
  },
  minAngle: {
    api: 'src/model/kiris.rs (en_küçük_açı); src/grafik/kiris.rs (açık/fazla yeniden dağıtımı)',
    testler: ['grafik::kiris::testler::min_angle_bagsiz_dugumleri_de_cizer', CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: ['chord-minAngle'],
    veri_biçimleri: ['nonnegative degree'], dallar: ['bağsız-düğüm', 'deficit/surplus', 'padAngle-önceliği']
  },
  itemStyle: {
    api: 'src/model/kiris.rs (KirişÖğeStili); src/grafik/kiris.rs (seri→düğüm→durum kalıtımı)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: ['chord-style'],
    veri_biçimleri: ['color/border/radius/opacity/shadow'], dallar: ['seri', 'düğüm', 'emphasis/blur/select', 'fill/border/shadow']
  },
  borderRadius: {
    api: 'src/model/kiris.rs (KirişKöşeYarıçapı); src/cizim/yuzey.rs (yuvarlatılmış_dilim_yolu)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: ['chord-style'],
    veri_biçimleri: ['number', '[inner,outer]', '[4]', 'percent'], dallar: ['dört-sektör-köşesi', 'halka-kalınlığına-yüzde', 'dar-açı-kıstırma']
  },
  lineStyle: {
    api: 'src/model/kiris.rs (KirişÇizgiStili/KirişKenarBoyası); src/grafik/kiris.rs (şerit_yolu/bağ_dolgusu)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: ['chord-simple', 'chord-lineStyle-color', 'chord-style'],
    veri_biçimleri: ['source/target/gradient/color', 'opacity/width/type/curveness/shadow'], dallar: ['seri', 'bağ', 'durum', 'iki-uç-gradyan']
  },
  curveness: {
    api: 'src/model/kiris.rs (KirişÇizgiStili::eğrilik); src/grafik/kiris.rs (şerit_yolu kübik kontrol oranı)',
    testler: [CHORD_SAHNE_TESTİ, CHORD_SEÇENEK_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['number'], dallar: ['0', '0.7-varsayılanı', 'kaynak/hedef-kontrol-çifti']
  },
  label: {
    api: 'src/model/stil.rs (Etiket/EtiketYaması); src/model/kiris.rs (düğüm/durum label); src/grafik/kiris.rs (etiket_geometrisi)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['Etiket', 'EtiketYaması', 'formatter/rich/font'], dallar: ['seri→düğüm', 'kenar-label alias', 'emphasis/blur/select', 'renk-kalıtımı']
  },
  position: {
    api: 'src/model/stil.rs (EtiketKonumu); src/grafik/kiris.rs (etiket_geometrisi)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: ['chord-simple', 'chord-style'],
    veri_biçimleri: ['outside', 'inside/SeriesLabelPosition'], dallar: ['r+distance', 'halka-ortası', 'otomatik-hiza']
  },
  silent: {
    api: 'src/model/stil.rs (Etiket::sessiz/EtiketYaması::sessiz); src/model/kiris.rs (KirişSerisi::sessiz)',
    testler: [CHORD_FIXTURE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['bool'], dallar: ['etiket-olay-geçişi', 'seri-isabet-kapalı']
  },
  edgeLabel: {
    api: 'src/model/kiris.rs (kenar_etiketi/KirişBağı::kenar_etiketi); src/grafik/kiris.rs (şerit etiketi)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ, CHORD_SEÇENEK_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['Etiket', 'EtiketYaması'], dallar: ['seri', 'bağ', 'durum', 'formatter']
  },
  emphasis: {
    api: 'src/model/kiris.rs (KirişDurumu/KirişVurguOdağı); src/grafik/kiris.rs (node/edge focus kümeleri)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ, CHORD_SEÇENEK_TESTİ], örnekler: ['chord-style'],
    veri_biçimleri: ['none/self/adjacency/series', 'scale', 'style/label yamaları'], dallar: ['düğüm', 'bağ', 'self', 'adjacency', 'disabled']
  },
  blur: {
    api: 'src/model/kiris.rs (KirişDurumu::bulanık); src/grafik/kiris.rs (odak-dışı katman)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['itemStyle/lineStyle/label/edgeLabel'], dallar: ['düğüm', 'bağ', 'otomatik-0.1-opaklık']
  },
  select: {
    api: 'src/model/kiris.rs (KirişDurumu::seçili, KirişDüğümü::başlangıçta_seçili); src/grafik/kiris.rs (seçili katman)',
    testler: [CHORD_FIXTURE_TESTİ, CHORD_SAHNE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['itemStyle/lineStyle/label/edgeLabel'], dallar: ['düğüm', 'bağ', 'başlangıç-seçimi']
  },
  legendHoverLink: {
    api: 'src/model/kiris.rs (KirişSerisi::gösterge_vurgusu); src/cizim/gorunum.rs (Chord LegendVisualProvider düğüm öğeleri)',
    testler: [CHORD_FIXTURE_TESTİ], örnekler: CHORD_ÖRNEKLERİ,
    veri_biçimleri: ['bool'], dallar: ['node-legend-provider', 'hover-link açık/kapalı']
  }
});

// Yalnız farklı zaman yüzdelerinden gerçekten örneklenen senaryolar tam
// animasyon kanıtı sayılır. Çok sayıda kararlı setOption uç durumu (örneğin
// scatter-symbol-morph şekilleri) kare sayısı yüksek olsa da ara geçişi
// kanıtlamaz.
const ARA_KARE_ANIMASYON_KANITI = new Set([
  'scatter-effect',
  'calendar-effectscatter',
  'calendar-charts'
]);

const YEREL_FIXTURE = Object.freeze({
  'line-simple': 'examples/uyum_fixture.rs#line_simple',
  'line-smooth': 'examples/uyum_fixture.rs#line_smooth',
  'area-basic': 'examples/uyum_fixture.rs#area_basic',
  'area-simple': 'examples/uyum_fixture.rs#area_simple',
  'area-time-axis': 'examples/uyum_fixture.rs#area_time_axis',
  'area-rainfall': 'examples/uyum_fixture.rs#area_rainfall',
  'dynamic-data2': 'examples/uyum_fixture.rs#dynamic_data2',
  'dynamic-data': 'examples/uyum_fixture.rs#dynamic_data',
  'line-sections': 'examples/uyum_fixture.rs#line_sections',
  'line-markline': 'examples/uyum_fixture.rs#line_markline',
  'area-pieces': 'examples/uyum_fixture.rs#area_pieces',
  'line-gradient': 'examples/uyum_fixture.rs#line_gradient',
  'line-aqi': 'examples/uyum_fixture.rs#line_aqi',
  'confidence-band': 'examples/uyum_fixture.rs#confidence_band',
  'line-race': 'examples/uyum_fixture.rs#line_race',
  'line-stack': 'examples/uyum_fixture.rs#line_stack',
  'line-style': 'examples/uyum_fixture.rs#line_style',
  'line-step': 'examples/uyum_fixture.rs#line_step',
  'line-in-cartesian-coordinate-system': 'examples/uyum_fixture.rs#line_in_cartesian_coordinate_system',
  'line-y-category': 'examples/uyum_fixture.rs#line_y_category',
  'line-log': 'examples/uyum_fixture.rs#line_log',
  'line-polar': 'examples/uyum_fixture.rs#line_polar',
  'line-polar2': 'examples/uyum_fixture.rs#line_polar2',
  'line-function': 'examples/uyum_fixture.rs#line_function',
  'bump-chart': 'examples/uyum_fixture.rs#bump_chart',
  'area-stack': 'examples/uyum_fixture.rs#area_stack',
  'area-stack-gradient': 'examples/uyum_fixture.rs#area_stack_gradient',
  'bar-simple': 'examples/uyum_fixture.rs#bar_simple',
  'bar1': 'examples/uyum_fixture.rs#bar1',
  'mix-line-bar': 'examples/uyum_fixture.rs#mix_line_bar',
  'multiple-x-axis': 'examples/uyum_fixture.rs#multiple_x_axis',
  'multiple-y-axis': 'examples/uyum_fixture.rs#multiple_y_axis',
  'bar-background': 'examples/uyum_fixture.rs#bar_background',
  'bar-tick-align': 'examples/uyum_fixture.rs#bar_tick_align',
  'bar-data-color': 'examples/uyum_fixture.rs#bar_data_color',
  'bar-stack-borderRadius': 'examples/uyum_fixture.rs#bar_stack_border_radius',
  'bar-y-category': 'examples/uyum_fixture.rs#bar_y_category',
  'bar-y-category-stack': 'examples/uyum_fixture.rs#bar_y_category_stack',
  'bar-negative2': 'examples/uyum_fixture.rs#bar_negative2',
  'bar-negative': 'examples/uyum_fixture.rs#bar_negative',
  'bar-stack': 'examples/uyum_fixture.rs#bar_stack',
  'bar-waterfall': 'examples/uyum_fixture.rs#bar_waterfall',
  'bar-waterfall2': 'examples/uyum_fixture.rs#bar_waterfall2',
  'bar-stack-normalization': 'examples/uyum_fixture.rs#bar_stack_normalization',
  'bar-brush': 'examples/uyum_fixture.rs#bar_brush',
  'bar-polar-label-radial': 'examples/uyum_fixture.rs#bar_polar_label_radial',
  'bar-polar-label-tangential': 'examples/uyum_fixture.rs#bar_polar_label_tangential',
  'bar-polar-stack': 'examples/uyum_fixture.rs#bar_polar_stack',
  'bar-polar-stack-radial': 'examples/uyum_fixture.rs#bar_polar_stack_radial',
  'bar-label-rotation': 'examples/uyum_fixture.rs#bar_label_rotation',
  'bar-breaks-simple': 'examples/uyum_fixture.rs#bar_breaks_simple',
  'bar-breaks-brush': 'examples/uyum_fixture.rs#bar_breaks_brush',
  'bar-gradient': 'examples/uyum_fixture.rs#bar_gradient',
  'data-transform-sort-bar': 'examples/uyum_fixture.rs#data_transform_sort_bar',
  'dataset-simple0': 'examples/uyum_fixture.rs#dataset_simple0',
  'dataset-simple1': 'examples/uyum_fixture.rs#dataset_simple1',
  'dataset-series-layout-by': 'examples/uyum_fixture.rs#dataset_series_layout_by',
  'dataset-encode0': 'examples/uyum_fixture.rs#dataset_encode0',
  'dataset-default': 'examples/uyum_fixture.rs#dataset_default',
  'data-transform-multiple-pie': 'examples/uyum_fixture.rs#data_transform_multiple_pie',
  'dataset-link': 'examples/uyum_fixture.rs#dataset_link',
  'data-transform-filter': 'examples/uyum_fixture.rs#data_transform_filter',
  'dataset-encode1': 'examples/uyum_fixture.rs#dataset_encode1',
  'data-transform-aggregate': 'examples/uyum_fixture.rs#data_transform_aggregate',
  'boxplot-multi': 'examples/uyum_fixture.rs#boxplot_multi',
  'boxplot-light-velocity': 'examples/uyum_fixture.rs#boxplot_light_velocity',
  'boxplot-light-velocity2': 'examples/uyum_fixture.rs#boxplot_light_velocity',
  'pie-nest': 'examples/uyum_fixture.rs#pie_nest',
  'pie-rich-text': 'examples/uyum_fixture.rs#pie_rich_text',
  'pie-simple': 'examples/uyum_fixture.rs#pie_simple',
  'pie-doughnut': 'examples/uyum_fixture.rs#pie_doughnut',
  'pie-roseType-simple': 'examples/uyum_fixture.rs#pie_rose_type_simple',
  'pie-roseType': 'examples/uyum_fixture.rs#pie_rose_type',
  'pie-legend': 'examples/uyum_fixture.rs#pie_legend',
  'pie-custom': 'examples/uyum_fixture.rs#pie_custom',
  'pie-pattern': 'examples/uyum_fixture.rs#pie_pattern',
  'pie-labelLine-adjust': 'examples/uyum_fixture.rs#pie_label_line_adjust',
  'pie-padAngle': 'examples/uyum_fixture.rs#pie_pad_angle',
  'pie-half-donut': 'examples/uyum_fixture.rs#pie_half_donut',
  'pie-borderRadius': 'examples/uyum_fixture.rs#pie_border_radius',
  'pie-alignTo': 'examples/uyum_fixture.rs#pie_align_to',
  'scatter-simple': 'examples/uyum_fixture.rs#scatter_simple',
  'scatter-anscombe-quartet': 'examples/uyum_fixture.rs#scatter_anscombe_quartet',
  'scatter-jitter': 'examples/uyum_fixture.rs#scatter_jitter',
  'doc-example/scatter-jitter-avoidOverlap': 'examples/uyum_fixture.rs#scatter_jitter_avoid_overlap',
  'scatter-punchCard': 'examples/uyum_fixture.rs#scatter_punch_card',
  'scatter-polar-punchCard': 'examples/uyum_fixture.rs#scatter_polar_punch_card',
  'scatter-single-axis': 'examples/uyum_fixture.rs#scatter_single_axis',
  'bubble-gradient': 'examples/uyum_fixture.rs#bubble_gradient',
  'scatter-label-align-top': 'examples/uyum_fixture.rs#scatter_label_align_top',
  'scatter-label-align-right': 'examples/uyum_fixture.rs#scatter_label_align_right',
  'scatter-aqi-color': 'examples/uyum_fixture.rs#scatter_aqi_color',
  'scatter-weight': 'examples/uyum_fixture.rs#scatter_weight',
  'scatter-aggregate-bar': 'examples/uyum_fixture.rs#scatter_aggregate_bar',
  'scatter-symbol-morph': 'examples/uyum_fixture.rs#scatter_symbol_morph',
  'scatter-large': 'examples/uyum_fixture.rs#scatter_large',
  'scatter-nebula': 'examples/uyum_fixture.rs#scatter_nebula',
  'scatter-nutrients': 'examples/uyum_fixture.rs#scatter_nutrients',
  'scatter-nutrients-matrix': 'examples/uyum_fixture.rs#scatter_nutrients_matrix',
  'scatter-stream-visual': 'examples/uyum_fixture.rs#scatter_stream_visual',
  'scatter-painter-choice': 'examples/uyum_fixture.rs#scatter_painter_choice',
  'scatter-clustering': 'examples/uyum_fixture.rs#scatter_clustering',
  'scatter-clustering-process': 'examples/uyum_fixture.rs#scatter_clustering_process',
  'scatter-exponential-regression': 'examples/uyum_fixture.rs#scatter_exponential_regression',
  'scatter-linear-regression': 'examples/uyum_fixture.rs#scatter_linear_regression',
  'scatter-polynomial-regression': 'examples/uyum_fixture.rs#scatter_polynomial_regression',
  'scatter-logarithmic-regression': 'examples/uyum_fixture.rs#scatter_logarithmic_regression',
  'scatter-effect': 'examples/uyum_fixture.rs#scatter_effect',
  'candlestick-simple': 'examples/uyum_fixture.rs#candlestick_simple',
  'candlestick-sh': 'examples/uyum_fixture.rs#candlestick_sh',
  'candlestick-large': 'examples/uyum_fixture.rs#candlestick_large',
  'candlestick-brush': 'examples/uyum_fixture.rs#candlestick_brush',
  'candlestick-sh-2015': 'examples/uyum_fixture.rs#candlestick_sh_2015',
  'heatmap-cartesian': 'examples/uyum_fixture.rs#heatmap_cartesian',
  'heatmap-large': 'examples/uyum_fixture.rs#heatmap_large',
  'heatmap-large-piecewise': 'examples/uyum_fixture.rs#heatmap_large_piecewise',
  'calendar-heatmap': 'examples/uyum_fixture.rs#calendar_heatmap',
  'calendar-simple': 'examples/uyum_fixture.rs#calendar_simple',
  'calendar-vertical': 'examples/uyum_fixture.rs#calendar_vertical',
  'calendar-horizontal': 'examples/uyum_fixture.rs#calendar_horizontal',
  'calendar-effectscatter': 'examples/uyum_fixture.rs#calendar_effectscatter',
  'calendar-graph': 'examples/uyum_fixture.rs#calendar_graph',
  'calendar-lunar': 'examples/uyum_fixture.rs#calendar_lunar',
  'calendar-pie': 'examples/uyum_fixture.rs#calendar_pie',
  'custom-calendar-icon': 'examples/uyum_fixture.rs#custom_calendar_icon',
  'calendar-charts': 'examples/uyum_fixture.rs#calendar_charts',
  'matrix-simple': 'examples/uyum_fixture.rs#matrix_simple',
  'matrix-correlation-heatmap': 'examples/uyum_fixture.rs#matrix_correlation_heatmap',
  'matrix-correlation-scatter': 'examples/uyum_fixture.rs#matrix_correlation_scatter',
  'matrix-covariance': 'examples/uyum_fixture.rs#matrix_covariance',
  'matrix-graph': 'examples/uyum_fixture.rs#matrix_graph',
  'matrix-pie': 'examples/uyum_fixture.rs#matrix_pie',
  'matrix-confusion': 'examples/uyum_fixture.rs#matrix_confusion',
  'matrix-grid-layout': 'examples/uyum_fixture.rs#matrix_grid_layout',
  'matrix-stock': 'examples/uyum_fixture.rs#matrix_stock',
  'matrix-sparkline': 'examples/uyum_fixture.rs#matrix_sparkline',
  'matrix-periodic-table': 'examples/uyum_fixture.rs#matrix_periodic_table',
  'matrix-mbti': 'examples/uyum_fixture.rs#matrix_mbti',
  'matrix-mini-bar-data-collection': 'examples/uyum_fixture.rs#matrix_mini_bar_data_collection',
  'line-marker': 'examples/uyum_fixture.rs#line_marker',
  'grid-multiple': 'examples/uyum_fixture.rs#grid_multiple',
  'intraday-breaks-1': 'examples/uyum_fixture.rs#intraday_breaks_1',
  'intraday-breaks-2': 'examples/uyum_fixture.rs#intraday_breaks_2',
  'mix-zoom-on-value': 'examples/uyum_fixture.rs#mix_zoom_on_value',
  'bar-polar-real-estate': 'examples/uyum_fixture.rs#bar_polar_real_estate',
  'polar-roundCap': 'examples/uyum_fixture.rs#polar_round_cap',
  'polar-endAngle': 'examples/uyum_fixture.rs#polar_end_angle',
  'bar-histogram': 'examples/uyum_fixture.rs#bar_histogram',
  'funnel': 'examples/uyum_fixture.rs#funnel',
  'funnel-align': 'examples/uyum_fixture.rs#funnel_align',
  'funnel-customize': 'examples/uyum_fixture.rs#funnel_customize',
  'funnel-mutiple': 'examples/uyum_fixture.rs#funnel_mutiple',
  'radar': 'examples/uyum_fixture.rs#radar',
  'radar-aqi': 'examples/uyum_fixture.rs#radar_aqi',
  'radar-custom': 'examples/uyum_fixture.rs#radar_custom',
  'radar2': 'examples/uyum_fixture.rs#radar2',
  'radar-multiple': 'examples/uyum_fixture.rs#radar_multiple',
  'parallel-simple': 'examples/uyum_fixture.rs#parallel_simple',
  'parallel-aqi': 'examples/uyum_fixture.rs#parallel_aqi',
  'parallel-nutrients': 'examples/uyum_fixture.rs#parallel_nutrients',
  'doc-example/parallel-all': 'examples/uyum_fixture.rs#parallel_all',
  'themeRiver-basic': 'examples/uyum_fixture.rs#theme_river_basic',
  'themeRiver-lastfm': 'examples/uyum_fixture.rs#theme_river_lastfm',
  'gauge': 'examples/uyum_fixture.rs#gauge',
  'gauge-simple': 'examples/uyum_fixture.rs#gauge_simple',
  'gauge-speed': 'examples/uyum_fixture.rs#gauge_speed',
  'gauge-progress': 'examples/uyum_fixture.rs#gauge_progress',
  'gauge-stage': 'examples/uyum_fixture.rs#gauge_stage',
  'gauge-grade': 'examples/uyum_fixture.rs#gauge_grade',
  'gauge-multi-title': 'examples/uyum_fixture.rs#gauge_multi_title',
  'gauge-temperature': 'examples/uyum_fixture.rs#gauge_temperature',
  'gauge-ring': 'examples/uyum_fixture.rs#gauge_ring',
  'gauge-barometer': 'examples/uyum_fixture.rs#gauge_barometer',
  'gauge-clock': 'examples/uyum_fixture.rs#gauge_clock',
  'gauge-car': 'examples/uyum_fixture.rs#gauge_car',
  'treemap-sunburst-transition': 'examples/uyum_fixture.rs#treemap_sunburst_transition',
  'treemap-disk': 'examples/uyum_fixture.rs#treemap_disk',
  'treemap-drill-down': 'examples/uyum_fixture.rs#treemap_drill_down',
  'treemap-obama': 'examples/uyum_fixture.rs#treemap_obama',
  'treemap-show-parent': 'examples/uyum_fixture.rs#treemap_show_parent',
  'treemap-simple': 'examples/uyum_fixture.rs#treemap_simple',
  'treemap-visual': 'examples/uyum_fixture.rs#treemap_visual',
  'sunburst-simple': 'examples/uyum_fixture.rs#sunburst_simple',
  'sunburst-borderRadius': 'examples/uyum_fixture.rs#sunburst_border_radius',
  'sunburst-label-rotate': 'examples/uyum_fixture.rs#sunburst_label_rotate',
  'sunburst-monochrome': 'examples/uyum_fixture.rs#sunburst_monochrome',
  'sunburst-visualMap': 'examples/uyum_fixture.rs#sunburst_visual_map',
  'sunburst-drink': 'examples/uyum_fixture.rs#sunburst_drink',
  'sunburst-book': 'examples/uyum_fixture.rs#sunburst_book',
  'sankey-energy': 'examples/uyum_fixture.rs#sankey_resmi',
  'sankey-itemstyle': 'examples/uyum_fixture.rs#sankey_resmi',
  'sankey-levels': 'examples/uyum_fixture.rs#sankey_resmi',
  'sankey-nodeAlign-left': 'examples/uyum_fixture.rs#sankey_resmi',
  'sankey-nodeAlign-right': 'examples/uyum_fixture.rs#sankey_resmi',
  'sankey-simple': 'examples/uyum_fixture.rs#sankey_resmi',
  'sankey-vertical': 'examples/uyum_fixture.rs#sankey_resmi',
  'chord-simple': 'examples/uyum_fixture.rs#kiriş_resmi',
  'chord-minAngle': 'examples/uyum_fixture.rs#kiriş_resmi',
  'chord-lineStyle-color': 'examples/uyum_fixture.rs#kiriş_resmi',
  'chord-style': 'examples/uyum_fixture.rs#kiriş_resmi',
  'tree-basic': 'examples/uyum_fixture.rs#tree_basic',
  'tree-legend': 'examples/uyum_fixture.rs#tree_legend',
  'tree-orient-bottom-top': 'examples/uyum_fixture.rs#tree_orient_bottom_top',
  'tree-orient-right-left': 'examples/uyum_fixture.rs#tree_orient_right_left',
  'tree-polyline': 'examples/uyum_fixture.rs#tree_polyline',
  'tree-radial': 'examples/uyum_fixture.rs#tree_radial',
  'tree-vertical': 'examples/uyum_fixture.rs#tree_vertical',
  'graph-simple': 'examples/grafo.rs'
});

function hata(mesaj) {
  throw new Error(`[uyum] ${mesaj}`);
}

function oku(dosya) {
  return fs.readFileSync(dosya, 'utf8');
}

function sha256(veri) {
  return crypto.createHash('sha256').update(veri).digest('hex');
}

function göreli(dosya, taban = KÖK) {
  return path.relative(taban, dosya).split(path.sep).join('/');
}

function gitCommit(dizin) {
  return execFileSync('git', ['-C', dizin, 'rev-parse', 'HEAD'], {
    encoding: 'utf8'
  }).trim();
}

function jsonDizisiOku(dosya) {
  const metin = oku(dosya);
  const başlangıç = metin.indexOf('[');
  const bitiş = metin.lastIndexOf(']');
  if (başlangıç < 0 || bitiş < başlangıç) {
    hata(`${dosya} içinde JSON dizisi bulunamadı`);
  }
  return JSON.parse(metin.slice(başlangıç, bitiş + 1));
}

function metadataOku(kaynak) {
  const eşleşme = kaynak.match(/^\s*\/\*([\s\S]*?)\*\//);
  if (!eşleşme) return {};
  const sonuç = {};
  for (const satır of eşleşme[1].split(/\r?\n/)) {
    const çift = satır.match(/^\s*([A-Za-z][\w-]*)\s*:\s*(.*?)\s*$/);
    if (!çift) continue;
    const [, anahtar, ham] = çift;
    if (/^-?\d+(?:\.\d+)?$/.test(ham)) sonuç[anahtar] = Number(ham);
    else if (ham === 'true' || ham === 'false') sonuç[anahtar] = ham === 'true';
    else sonuç[anahtar] = ham;
  }
  return sonuç;
}

function örnekYolu(kayıt, gl = false) {
  const uzantı = kayıt.ts ? '.ts' : '.js';
  return path.join(
    ÖRNEKLER,
    'public/examples/ts',
    gl ? 'gl' : '',
    `${kayıt.id}${uzantı}`
  );
}

function geoNedenleri(kayıt, kaynak) {
  const nedenler = [];
  if (kayıt.category.some((kategori) => kategori === 'map' || kategori === 'geo')) {
    nedenler.push('metadata_kategorisi');
  }
  for (const [ad, desen] of GEO_DESENLERİ) {
    if (desen.test(kaynak)) nedenler.push(ad);
  }
  return nedenler;
}

function benzersizEşleşmeler(kaynak, desen) {
  return [...new Set([...kaynak.matchAll(desen)].map((eşleşme) => eşleşme[1]))].sort();
}

function kaynakYetenekleri(kaynak) {
  const seriler = benzersizEşleşmeler(kaynak, /\btype\s*:\s*['"]([A-Za-z][\w]*)['"]/g)
    .filter((ad) => !['value', 'category', 'time', 'log', 'inside', 'slider'].includes(ad));
  const koordinatlar = benzersizEşleşmeler(
    kaynak,
    /\bcoordinateSystem\s*:\s*['"]([A-Za-z][\w]*)['"]/g
  );
  const bileşenler = BİLEŞENLER.filter((ad) => new RegExp(`\\b${ad}\\s*:`).test(kaynak));
  const actions = benzersizEşleşmeler(kaynak, /dispatchAction\s*\(\s*{[\s\S]{0,300}?\btype\s*:\s*['"]([\w]+)['"]/g);
  const events = benzersizEşleşmeler(kaynak, /\.on\s*\(\s*['"]([\w]+)['"]/g);
  const dışVarlıklar = benzersizEşleşmeler(
    kaynak,
    /(?:ROOT_PATH|CDN_PATH)\s*\+\s*['"]([^'"]+)['"]/g
  );
  return { seriler, koordinatlar, bileşenler, actions, events, dışVarlıklar };
}

function sahipFaz(kategoriler, yetenekler) {
  const tüm = new Set([...kategoriler, ...yetenekler.seriler, ...yetenekler.bileşenler]);
  if ([...tüm].some((x) => ['custom', 'graphic', 'rich'].includes(x))) return 6;
  if ([...tüm].some((x) => ['graph', 'tree', 'treemap', 'sunburst', 'sankey', 'chord'].includes(x))) return 5;
  if ([...tüm].some((x) => ['radar', 'parallel', 'funnel', 'gauge', 'themeRiver', 'calendar', 'matrix', 'lines'].includes(x))) return 4;
  if ([...tüm].some((x) => ['line', 'bar', 'pie', 'scatter', 'effectScatter', 'candlestick', 'boxplot', 'heatmap', 'pictorialBar'].includes(x))) return 3;
  if ([...tüm].some((x) => ['dataset', 'transform'].includes(x))) return 2;
  return 7;
}

function kanıt(durum = 'yok') {
  return {
    api: durum,
    statik_görsel: durum,
    animasyon: durum,
    etkileşim: durum,
    erişilebilirlik: durum,
    performans: durum
  };
}

function kanıtDosyası(dosya) {
  if (!dosya || !fs.existsSync(dosya) || !fs.statSync(dosya).isFile()) return null;
  return {
    yol: göreli(dosya),
    sha256: sha256(fs.readFileSync(dosya))
  };
}

/**
 * Görsel koşucunun ürettiği metrikleri resmi örnek kimliğine bağlar.
 * Yalnız eşiği geçmiş ve bütün dosyaları mevcut kareler kanıt sayılır.
 */
function görselKanıtlarıOku() {
  const sonuç = new Map();
  if (!fs.existsSync(GÖRSEL_METRİKLER)) return sonuç;
  for (const dosya of fs.readdirSync(GÖRSEL_METRİKLER).sort()) {
    if (!dosya.endsWith('.json')) continue;
    const tam = path.join(GÖRSEL_METRİKLER, dosya);
    let metrik;
    try {
      metrik = JSON.parse(oku(tam));
    } catch (neden) {
      hata(`görsel metrik ayrıştırılamadı: ${göreli(tam)} (${neden})`);
    }
    if (typeof metrik.id !== 'string' || typeof metrik.kare !== 'string') {
      hata(`görsel metrik kimliği/kare adı eksik: ${göreli(tam)}`);
    }
    const referans = kanıtDosyası(path.join(KÖK, metrik.dosyalar?.referans ?? ''));
    const gerçek = kanıtDosyası(path.join(KÖK, metrik.dosyalar?.gerçek ?? ''));
    const fark = kanıtDosyası(path.join(KÖK, metrik.dosyalar?.fark ?? ''));
    const tipografiProfili = metrik.tipografi_normalizasyonu ?? null;
    const normalizeFark = tipografiProfili
      ? kanıtDosyası(path.join(KÖK, metrik.dosyalar?.normalize_fark ?? ''))
      : null;
    const sahne = metrik.dosyalar?.sahne
      ? kanıtDosyası(path.join(KÖK, metrik.dosyalar.sahne))
      : null;
    const kare = {
      ad: metrik.kare,
      senaryo: metrik.senaryo,
      geçti: metrik.geçti === true,
      değişen_piksel: metrik.değişen_piksel,
      değişen_piksel_oranı: metrik.değişen_piksel_oranı,
      ssim: metrik.ssim,
      referans,
      gerçek,
      fark,
      ...(metrik.ham ? { ham: metrik.ham } : {}),
      ...(tipografiProfili
        ? { tipografi_normalizasyonu: tipografiProfili, normalize_fark: normalizeFark }
        : {}),
      ...(sahne ? { sahne } : {}),
      metrik: kanıtDosyası(tam)
    };
    kare.geçti = kare.geçti
      && [referans, gerçek, fark, kare.metrik].every(Boolean)
      && (!metrik.dosyalar?.sahne || Boolean(sahne))
      && (!tipografiProfili || (
        Number.isFinite(tipografiProfili.gaussian_sigma)
        && tipografiProfili.gaussian_sigma > 0
        && Boolean(normalizeFark)
        && metrik.ham
        && Number.isFinite(metrik.ham.değişen_piksel_oranı)
        && Number.isFinite(metrik.ham.ssim)
      ))
      && Number.isFinite(kare.değişen_piksel_oranı)
      && kare.değişen_piksel_oranı <= 0.01
      && Number.isFinite(kare.ssim)
      && kare.ssim >= 0.99;
    const kareler = sonuç.get(metrik.id) ?? [];
    kareler.push(kare);
    sonuç.set(metrik.id, kareler);
  }
  for (const kareler of sonuç.values()) kareler.sort((a, b) => a.ad.localeCompare(b.ad));
  return sonuç;
}

function manifestKaydı(kayıt, gl = false, görselKanıtlar = new Map()) {
  const dosya = örnekYolu(kayıt, gl);
  if (!fs.existsSync(dosya)) hata(`örnek kaynağı bulunamadı: ${dosya}`);
  const kaynak = oku(dosya);
  const metadata = metadataOku(kaynak);
  const yetenekler = kaynakYetenekleri(kaynak);
  const geo = gl ? [] : geoNedenleri(kayıt, kaynak);
  const kapsamDışıDurumu = gl
    ? 'kapsam_dışı_gl_3d'
    : geo.length > 0
      ? 'kapsam_dışı_geo_map'
      : null;
  const görünür = !kayıt.noExplore;
  const fixture = YEREL_FIXTURE[kayıt.id] ?? null;
  const faz = gl || geo.length > 0 ? null : sahipFaz(kayıt.category, yetenekler);
  const kareler = görselKanıtlar.get(kayıt.id) ?? [];
  const görselTam = kareler.length > 0 && kareler.every((kare) => kare.geçti);
  // Tek bir statik görsel, davranış/erişilebilirlik/performans kapılarını
  // kanıtlamaz. Bu yüzden yeşil kareli örnek "tam" yapılmaz; yalnız
  // uygulamanın ve kilitli görsel kanıtın varlığı kayda geçirilir.
  const durum = kapsamDışıDurumu
    ?? (fixture ? (görselTam ? 'uygulandı_kanıt_bekliyor' : 'kısmi') : 'yok');
  const kayıtKanıtı = kanıt(fixture ? 'kısmi' : 'yok');
  if (görselTam) {
    kayıtKanıtı['statik_görsel'] = 'tam_kanıtlı';
    const senaryoTürleri = new Set(kareler.map((kare) => kare.senaryo));
    if (senaryoTürleri.has('animasyon') && ARA_KARE_ANIMASYON_KANITI.has(kayıt.id)) {
      kayıtKanıtı.animasyon = 'tam_kanıtlı';
    }
    // Durum kareleri davranışın görsel bölümünü kanıtlar; olay yükü ve
    // option/state anlık görüntüsü doğrulanana dek etkileşim kanıtı tam değil.
    if (senaryoTürleri.has('etkileşim')) {
      kayıtKanıtı.etkileşim = 'uygulandı_kanıt_bekliyor';
    }
  }
  if (durum.startsWith('kapsam_dışı_')) {
    for (const anahtar of Object.keys(kayıtKanıtı)) kayıtKanıtı[anahtar] = durum;
  }
  return {
    id: kayıt.id,
    başlık: { en: kayıt.title, tr: kayıt.title },
    başlık_çeviri_durumu: 'bekliyor',
    kategoriler: kayıt.category,
    etiketler: kayıt.tags ?? [],
    difficulty: kayıt.difficulty ?? 0,
    since: kayıt.since ?? null,
    theme: kayıt.theme ?? metadata.theme ?? 'default',
    noExplore: Boolean(kayıt.noExplore),
    resmi_sayfada_görünür: görünür && !gl,
    shotWidth: metadata.shotWidth ?? 700,
    shotDelay: metadata.shotDelay ?? 0,
    videoStart: metadata.videoStart ?? null,
    videoEnd: metadata.videoEnd ?? null,
    kaynak: göreli(dosya, ÖRNEKLER),
    kaynak_sha256: sha256(kaynak),
    cizelge_fixture: fixture,
    seriler: yetenekler.seriler,
    koordinatlar: yetenekler.koordinatlar,
    bileşenler: yetenekler.bileşenler,
    actions: yetenekler.actions,
    events: yetenekler.events,
    dış_varlıklar: yetenekler.dışVarlıklar,
    kapsam_durumu: durum,
    kapsam_gerekçeleri: gl
      ? ['echarts_gl_kataloğu']
      : geo.length > 0
        ? geo
        : ['çekirdek_2b_geo_dışı'],
    sahip_faz: faz,
    bağımlı_fazlar: faz == null ? [] : [...Array(Math.max(0, faz)).keys()],
    kanıt: kayıtKanıtı,
    çıktılar: { kareler }
  };
}

function dosyalarıYürü(dizin, kabul) {
  const sonuç = [];
  for (const girdi of fs.readdirSync(dizin, { withFileTypes: true })) {
    const tam = path.join(dizin, girdi.name);
    if (girdi.isDirectory()) sonuç.push(...dosyalarıYürü(tam, kabul));
    else if (kabul(tam)) sonuç.push(tam);
  }
  return sonuç.sort();
}

function kaynakKökü(dosya, arayüz) {
  const göreliYol = göreli(dosya, ECHARTS);
  let eş = göreliYol.match(/^src\/chart\/([^/]+)\//);
  if (eş) return `series.${eş[1]}`;
  eş = göreliYol.match(/^src\/component\/([^/]+)\//);
  if (eş) return eş[1];
  eş = göreliYol.match(/^src\/coord\/([^/]+)\//);
  if (eş) return eş[1];
  const temiz = arayüz
    .replace(/SeriesOption.*$/, '')
    .replace(/Option.*$/, '')
    .replace(/Model.*$/, '')
    .replace(/([a-z0-9])([A-Z])/g, '$1.$2')
    .toLowerCase();
  return temiz || 'core';
}

function nesneVarsayılanları(düğüm, kaynakDosya, önek, sonuç) {
  if (!ts.isObjectLiteralExpression(düğüm)) return;
  for (const özellik of düğüm.properties) {
    if (!ts.isPropertyAssignment(özellik) && !ts.isShorthandPropertyAssignment(özellik)) continue;
    const ad = özellik.name?.getText(kaynakDosya).replace(/^['"]|['"]$/g, '');
    if (!ad) continue;
    const yol = önek ? `${önek}.${ad}` : ad;
    if (ts.isPropertyAssignment(özellik)) {
      sonuç.set(yol, özellik.initializer.getText(kaynakDosya));
      nesneVarsayılanları(özellik.initializer, kaynakDosya, yol, sonuç);
    }
  }
}

function varsayılanlarıTopla(kaynakDosya) {
  const sonuç = new Map();
  function gez(düğüm) {
    if (
      ts.isPropertyDeclaration(düğüm)
      && düğüm.name?.getText(kaynakDosya) === 'defaultOption'
      && düğüm.initializer
    ) {
      const başlatıcı = düğüm.initializer;
      if (ts.isObjectLiteralExpression(başlatıcı)) nesneVarsayılanları(başlatıcı, kaynakDosya, '', sonuç);
      else if (ts.isCallExpression(başlatıcı)) {
        for (const argüman of başlatıcı.arguments) {
          nesneVarsayılanları(argüman, kaynakDosya, '', sonuç);
        }
      }
    }
    ts.forEachChild(düğüm, gez);
  }
  gez(kaynakDosya);
  return sonuç;
}

function rustKarşılığı(kök, özellik) {
  if (kök.toLowerCase() === 'parallel' && PARALLEL_OPTION_KANITI[özellik]) {
    const kanıt = PARALLEL_OPTION_KANITI[özellik];
    return {
      api: kanıt.api,
      durum: 'uygulandı_kanıt_bekliyor',
      testler: kanıt.testler,
      galeri_örnekleri: kanıt.örnekler,
      veri_biçimleri: kanıt.veri_biçimleri,
      koordinat_dalları: kanıt.dallar
    };
  }
  if (kök.toLowerCase() === 'series.parallel' && PARALLEL_SERIES_OPTION_KANITI[özellik]) {
    const kanıt = PARALLEL_SERIES_OPTION_KANITI[özellik];
    return {
      api: kanıt.api,
      durum: 'uygulandı_kanıt_bekliyor',
      testler: kanıt.testler,
      galeri_örnekleri: kanıt.örnekler,
      veri_biçimleri: kanıt.veri_biçimleri,
      koordinat_dalları: kanıt.dallar
    };
  }
  if (kök.toLowerCase() === 'series.tree' && TREE_SERIES_OPTION_KANITI[özellik]) {
    const kanıt = TREE_SERIES_OPTION_KANITI[özellik];
    return {
      api: kanıt.api,
      durum: 'uygulandı_kanıt_bekliyor',
      testler: kanıt.testler,
      galeri_örnekleri: kanıt.örnekler,
      veri_biçimleri: kanıt.veri_biçimleri,
      koordinat_dalları: kanıt.dallar
    };
  }
  if (kök.toLowerCase() === 'series.treemap' && TREEMAP_SERIES_OPTION_KANITI[özellik]) {
    const kanıt = TREEMAP_SERIES_OPTION_KANITI[özellik];
    return {
      api: kanıt.api,
      durum: 'uygulandı_kanıt_bekliyor',
      testler: kanıt.testler,
      galeri_örnekleri: kanıt.örnekler,
      veri_biçimleri: kanıt.veri_biçimleri,
      koordinat_dalları: kanıt.dallar
    };
  }
  if (kök.toLowerCase() === 'series.sunburst' && SUNBURST_SERIES_OPTION_KANITI[özellik]) {
    const kanıt = SUNBURST_SERIES_OPTION_KANITI[özellik];
    return {
      api: kanıt.api,
      durum: 'uygulandı_kanıt_bekliyor',
      testler: kanıt.testler,
      galeri_örnekleri: kanıt.örnekler,
      veri_biçimleri: kanıt.veri_biçimleri,
      koordinat_dalları: kanıt.dallar
    };
  }
  if (kök.toLowerCase() === 'series.sankey' && SANKEY_SERIES_OPTION_KANITI[özellik]) {
    const kanıt = SANKEY_SERIES_OPTION_KANITI[özellik];
    return {
      api: kanıt.api,
      durum: 'uygulandı_kanıt_bekliyor',
      testler: kanıt.testler,
      galeri_örnekleri: kanıt.örnekler,
      veri_biçimleri: kanıt.veri_biçimleri,
      koordinat_dalları: kanıt.dallar
    };
  }
  if (kök.toLowerCase() === 'series.chord' && CHORD_SERIES_OPTION_KANITI[özellik]) {
    const kanıt = CHORD_SERIES_OPTION_KANITI[özellik];
    return {
      api: kanıt.api,
      durum: 'uygulandı_kanıt_bekliyor',
      testler: kanıt.testler,
      galeri_örnekleri: kanıt.örnekler,
      veri_biçimleri: kanıt.veri_biçimleri,
      koordinat_dalları: kanıt.dallar
    };
  }
  if (kök.toLowerCase() === 'echarts' && özellik === 'parallel') {
    return {
      api: 'src/model/secenekler.rs (GrafikSeçenekleri::paralel, paralel_ekle)',
      durum: 'uygulandı_kanıt_bekliyor',
      testler: ['calisma_zamani::testler::parallel_ve_parallel_axis_set_option_kokleri_bagimsiz_birlesir'],
      galeri_örnekleri: ['parallel-simple', 'parallel-aqi', 'parallel-nutrients', 'doc-example/parallel-all'],
      veri_biçimleri: ['ParalelKoordinatı', 'Vec<ParalelKoordinatı>'],
      koordinat_dalları: ['tekil', 'dizi', 'setOption-merge', 'replaceMerge']
    };
  }
  if (kök.toLowerCase() === 'echarts' && özellik === 'parallelAxis') {
    return {
      api: 'src/model/secenekler.rs (GrafikSeçenekleri::paralel_ekseni, paralel_eksenleri)',
      durum: 'uygulandı_kanıt_bekliyor',
      testler: ['calisma_zamani::testler::parallel_ve_parallel_axis_set_option_kokleri_bagimsiz_birlesir'],
      galeri_örnekleri: ['parallel-simple', 'parallel-aqi', 'parallel-nutrients'],
      veri_biçimleri: ['ParalelEkseni', 'Vec<ParalelEkseni>'],
      koordinat_dalları: ['value', 'category', 'time', 'log', 'çoklu-dim']
    };
  }
  if (kök.toLowerCase() === 'registered' && özellik === 'parallel') {
    return {
      api: 'src/model/seri.rs (Seri::Paralel, From<ParalelSerisi>)',
      durum: 'uygulandı_kanıt_bekliyor',
      testler: ['grafik::paralel::testler::resmi_seri_varsayilanlarini_korur'],
      galeri_örnekleri: ['parallel-simple', 'parallel-aqi', 'parallel-nutrients'],
      veri_biçimleri: ['ParalelSerisi'],
      koordinat_dalları: ['kayıt', 'boyama', 'olay/isabet']
    };
  }
  if (kök.toLowerCase() === 'registered' && özellik === 'tree') {
    return {
      api: 'src/model/seri.rs (Seri::Ağaç, From<AğaçSerisi>)',
      durum: 'uygulandı_kanıt_bekliyor',
      testler: ['tree_fixture_testleri::yedi_resmi_tree_fixture_seçeneklerini_ve_verisini_korur'],
      galeri_örnekleri: TREE_ÖRNEKLERİ,
      veri_biçimleri: ['AğaçSerisi'],
      koordinat_dalları: ['kayıt', 'boyama', 'olay/isabet']
    };
  }
  if (kök.toLowerCase() === 'registered' && özellik === 'treemap') {
    return {
      api: 'src/model/seri.rs (Seri::AğaçHaritası, From<AğaçHaritasıSerisi>)',
      durum: 'uygulandı_kanıt_bekliyor',
      testler: [TREEMAP_FIXTURE_TESTİ, TREEMAP_SAHNE_TESTİ],
      galeri_örnekleri: TREEMAP_ÖRNEKLERİ,
      veri_biçimleri: ['AğaçHaritasıSerisi'],
      koordinat_dalları: ['none', 'calendar-box', 'matrix-box', 'boyama', 'olay/isabet']
    };
  }
  if (kök.toLowerCase() === 'registered' && özellik === 'sunburst') {
    return {
      api: 'src/model/seri.rs (Seri::GüneşPatlaması, From<GüneşPatlamasıSerisi>)',
      durum: 'uygulandı_kanıt_bekliyor',
      testler: [SUNBURST_FIXTURE_TESTİ, SUNBURST_SAHNE_TESTİ, SUNBURST_ACTION_TESTİ],
      galeri_örnekleri: SUNBURST_ÖRNEKLERİ,
      veri_biçimleri: ['GüneşPatlamasıSerisi'],
      koordinat_dalları: ['none', 'calendar-box', 'matrix-box', 'boyama', 'olay/isabet']
    };
  }
  if (kök.toLowerCase() === 'registered' && özellik === 'sankey') {
    return {
      api: 'src/model/seri.rs (Seri::Sankey, From<SankeySerisi>); src/grafik/sankey.rs (yerleşim/boyama)',
      durum: 'uygulandı_kanıt_bekliyor',
      testler: [SANKEY_FIXTURE_TESTİ, SANKEY_SAHNE_TESTİ, SANKEY_ACTION_TESTİ],
      galeri_örnekleri: SANKEY_ÖRNEKLERİ,
      veri_biçimleri: ['SankeySerisi'],
      koordinat_dalları: ['view', 'calendar-box', 'matrix-box', 'boyama', 'isabet/tooltip', 'drag/roam']
    };
  }
  if (kök.toLowerCase() === 'registered' && özellik === 'chord') {
    return {
      api: 'src/model/seri.rs (Seri::Kiriş, From<KirişSerisi>); src/model/kiris.rs (model); src/grafik/kiris.rs (yerleşim/boyama)',
      durum: 'uygulandı_kanıt_bekliyor',
      testler: [
        CHORD_FIXTURE_TESTİ,
        CHORD_SAHNE_TESTİ,
        CHORD_YERLEŞİM_TESTİ,
        CHORD_İSABET_TESTİ,
        CHORD_SEÇENEK_TESTİ,
        CHORD_GÖSTERGE_TESTİ,
        CHORD_TOOLTIP_TESTİ
      ],
      galeri_örnekleri: CHORD_ÖRNEKLERİ,
      veri_biçimleri: ['KirişSerisi'],
      koordinat_dalları: ['none', 'box-layout', 'boyama', 'isabet/tooltip', 'durum katmanları']
    };
  }
  if (kök.toLowerCase() === 'calendar' && CALENDAR_OPTION_KANITI[özellik]) {
    const kanıt = CALENDAR_OPTION_KANITI[özellik];
    return {
      api: kanıt.api,
      durum: 'uygulandı_kanıt_bekliyor',
      testler: kanıt.testler,
      galeri_örnekleri: kanıt.örnekler,
      veri_biçimleri: kanıt.veri_biçimleri,
      koordinat_dalları: kanıt.dallar
    };
  }
  if (kök.toLowerCase() === 'matrix' && MATRIX_OPTION_KANITI[özellik]) {
    const kanıt = MATRIX_OPTION_KANITI[özellik];
    return {
      api: kanıt.api,
      durum: 'uygulandı_kanıt_bekliyor',
      testler: kanıt.testler,
      galeri_örnekleri: kanıt.örnekler,
      veri_biçimleri: kanıt.veri_biçimleri,
      koordinat_dalları: kanıt.dallar
    };
  }
  if (kök.toLowerCase() === 'visualmap' && özellik === 'categories') {
    return { api: 'src/model/gorsel_esleme.rs (GörselEşleme::kategoriler)', durum: 'kısmi' };
  }
  if (kök.toLowerCase() === 'visualmap' && özellik === 'dimension') {
    return { api: 'src/model/gorsel_esleme.rs (GörselEşleme::boyut)', durum: 'kısmi' };
  }
  if (kök.toLowerCase() === 'visualmap' && özellik === 'realtime') {
    return { api: 'src/model/gorsel_esleme.rs (GörselEşleme::gerçek_zamanlı)', durum: 'kısmi' };
  }
  if (kök === 'animation' && özellik === 'animationThreshold') {
    return { api: 'src/model/seri.rs (SaçılımSerisi::animasyon_eşiği)', durum: 'kısmi' };
  }
  if (kök.toLowerCase() === 'datazoom' && özellik === 'handleIcon') {
    return { api: 'src/model/yakinlastirma.rs (VeriYakınlaştırma::tutamaç_simgesi)', durum: 'kısmi' };
  }
  if (kök.toLowerCase() === 'datazoom' && özellik === 'handleSize') {
    return { api: 'src/model/yakinlastirma.rs (VeriYakınlaştırma::tutamaç_boyutu)', durum: 'kısmi' };
  }
  if (kök.toLowerCase() === 'datazoom' && özellik === 'showDataShadow') {
    return { api: 'src/model/yakinlastirma.rs (VeriYakınlaştırma::veri_gölgesi)', durum: 'kısmi' };
  }
  if (kök.toLowerCase() === 'datazoom' && özellik === 'realtime') {
    return { api: 'src/model/yakinlastirma.rs (VeriYakınlaştırma::gerçek_zamanlı)', durum: 'kısmi' };
  }
  if (kök.toLowerCase() === 'datazoom' && özellik === 'right') {
    return { api: 'src/model/yakinlastirma.rs (VeriYakınlaştırma::sağ)', durum: 'kısmi' };
  }
  if (kök === 'series.large' && özellik === 'large') {
    return { api: 'src/model/seri.rs (SaçılımSerisi::büyük)', durum: 'kısmi' };
  }
  if (kök === 'series.large' && özellik === 'largeThreshold') {
    return { api: 'src/model/seri.rs (SaçılımSerisi::büyük_eşiği)', durum: 'kısmi' };
  }
  if (kök === 'core' && özellik === 'progressive') {
    return { api: 'src/model/seri.rs (SaçılımSerisi::aşamalı)', durum: 'kısmi' };
  }
  if (kök === 'core' && özellik === 'progressiveThreshold') {
    return { api: 'src/model/seri.rs (SaçılımSerisi::aşamalı_eşiği)', durum: 'kısmi' };
  }
  const seri = kök.match(/^series\.([^.]+)/)?.[1];
  if (seri && RUST_SERİLERİ.has(seri)) {
    return { api: `src/model/seri.rs (${seri}.${özellik})`, durum: 'kısmi' };
  }
  const mevcutBileşenler = new Set([
    'title', 'legend', 'tooltip', 'grid', 'toolbox', 'brush', 'timeline',
    'datazoom', 'visualmap', 'axisPointer'.toLowerCase(), 'calendar', 'radar',
    'parallel'
  ]);
  if (mevcutBileşenler.has(kök.toLowerCase())) {
    return { api: `src/model/bilesen.rs (${kök}.${özellik})`, durum: 'kısmi' };
  }
  return { api: null, durum: 'yok' };
}

function özellikMatrisi() {
  const dosyalar = dosyalarıYürü(path.join(ECHARTS, 'src'), (dosya) => dosya.endsWith('.ts'));
  const satırlar = [];
  for (const dosya of dosyalar) {
    const metin = oku(dosya);
    const kaynakDosya = ts.createSourceFile(
      dosya,
      metin,
      ts.ScriptTarget.ES2020,
      true,
      ts.ScriptKind.TS
    );
    const varsayılanlar = varsayılanlarıTopla(kaynakDosya);
    function gez(düğüm) {
      const arayüzMü = ts.isInterfaceDeclaration(düğüm);
      const tipMi = ts.isTypeAliasDeclaration(düğüm) && ts.isTypeLiteralNode(düğüm.type);
      if ((arayüzMü || tipMi) && /Option/.test(düğüm.name.text)) {
        const üyeler = arayüzMü ? düğüm.members : düğüm.type.members;
        const kök = kaynakKökü(dosya, düğüm.name.text);
        for (const üye of üyeler) {
          if (!ts.isPropertySignature(üye) || !üye.name) continue;
          const özellik = üye.name.getText(kaynakDosya).replace(/^['"]|['"]$/g, '');
          const konum = kaynakDosya.getLineAndCharacterOfPosition(üye.getStart(kaynakDosya));
          const eşleme = rustKarşılığı(kök, özellik);
          const kısaVarsayılan = varsayılanlar.get(özellik) ?? null;
          const satır = {
            id: sha256(`${göreli(dosya, ECHARTS)}:${düğüm.name.text}:${özellik}`).slice(0, 20),
            tür: arayüzMü ? 'interface_property' : 'type_property',
            sahip: kök,
            option_yolu: `${kök}.${özellik}`,
            ts_arayüzü: düğüm.name.text,
            ts_türü: üye.type?.getText(kaynakDosya) ?? 'unknown',
            zorunlu: !üye.questionToken,
            echarts_varsayılanı: kısaVarsayılan,
            rust_api: eşleme.api,
            veri_biçimleri: eşleme.veri_biçimleri ?? [],
            koordinat_dalları: eşleme.koordinat_dalları ?? [],
            kaynak: {
              dosya: göreli(dosya, ECHARTS),
              satır: konum.line + 1,
              sembol: düğüm.name.text
            },
            testler: eşleme.testler ?? [],
            galeri_örnekleri: eşleme.galeri_örnekleri ?? [],
            durum: eşleme.durum
          };
          if (!DURUMLAR.has(satır.durum)) hata(`geçersiz matris durumu: ${satır.durum}`);
          satırlar.push(satır);
        }
      }
      ts.forEachChild(düğüm, gez);
    }
    gez(kaynakDosya);
  }
  satırlar.sort((a, b) => a.option_yolu.localeCompare(b.option_yolu) || a.id.localeCompare(b.id));
  return satırlar;
}

function tomlKaçır(metin) {
  return String(metin).replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

function senaryoMetni(kayıt) {
  return `# Bu dosya tools/uyum/uret.mjs tarafından üretilir; elle düzenlemeyin.\n`+
    `id = "${tomlKaçır(kayıt.id)}"\n`+
    `kapsam_durumu = "${kayıt.kapsam_durumu}"\n`+
    `sahip_faz = ${kayıt.sahip_faz ?? 0}\n`+
    `resmi_kaynak = "${tomlKaçır(kayıt.kaynak)}"\n`+
    `fixture = "${tomlKaçır(kayıt.cizelge_fixture ?? '')}"\n`+
    `viewport_genişlik = ${kayıt.shotWidth}\n`+
    `viewport_yükseklik = ${Math.round(kayıt.shotWidth * 0.75)}\n`+
    `shot_delay_ms = ${kayıt.shotDelay}\n`+
    `profiller = ["açık", "koyu"]\n\n`+
    `[kanıt]\n`+
    `api = "${kayıt.kanıt.api}"\n`+
    `statik_görsel = "${kayıt.kanıt['statik_görsel']}"\n`+
    `animasyon = "${kayıt.kanıt.animasyon}"\n`+
    `etkileşim = "${kayıt.kanıt.etkileşim}"\n`+
    `erişilebilirlik = "${kayıt.kanıt['erişilebilirlik']}"\n`+
    `performans = "${kayıt.kanıt.performans}"\n`;
}

function htmlKaçır(değer) {
  return String(değer)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

function raporHtml(manifest, özet) {
  const kartlar = manifest
    .filter((kayıt) => !kayıt.kapsam_durumu.startsWith('kapsam_dışı_'))
    .map((kayıt) => {
      const önizleme = kayıt.çıktılar.kareler.find((kare) => kare.geçti)?.gerçek?.yol;
      const boşAçıklaması = kayıt.kapsam_durumu === 'yok'
        ? `Henüz uygulanmadı · Faz ${kayıt.sahip_faz ?? '—'}`
        : kayıt.cizelge_fixture
          ? 'Fixture var · görsel kapı bekliyor'
          : 'Görsel kanıt henüz yok';
      const önizlemeHtml = önizleme
        ? `<img class="önizleme" loading="lazy" src="../../${htmlKaçır(önizleme)}" alt="${htmlKaçır(kayıt.başlık.en)} Cizelge çıktısı">`
        : `<div class="önizleme boş">${htmlKaçır(boşAçıklaması)}</div>`;
      const gizliRozeti = kayıt.resmi_sayfada_görünür
        ? ''
        : '\n      <p class="kaynak-rozeti">Resmî galeride gizli · doğrulama senaryosu</p>';
      return `<article class="kart ${kayıt.kapsam_durumu}${kayıt.resmi_sayfada_görünür ? '' : ' gizli-konformans'}">
      ${önizlemeHtml}
      <h2>${htmlKaçır(kayıt.başlık.en)}</h2>
      <code>${htmlKaçır(kayıt.id)}</code>
      <p>${htmlKaçır(kayıt.kategoriler.join(' · '))}</p>${gizliRozeti}
      <p class="rozet">API ${htmlKaçır(kayıt.kanıt.api)} · görsel ${htmlKaçır(kayıt.kanıt['statik_görsel'])}</p>
    </article>`;
    }).join('\n');
  return `<!doctype html>
<html lang="tr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Cizelge ECharts 6.1 uyum raporu</title>
<style>
:root{color-scheme:light dark;font:14px system-ui;background:#f5f7fa;color:#1f2937}body{margin:0}.üst{position:sticky;top:0;background:#fff;padding:18px 24px;border-bottom:1px solid #d8dee9;z-index:2}.üst h1{margin:0 0 8px}.özet{display:flex;gap:16px;flex-wrap:wrap}.özet strong{font-size:18px}.ızgara{display:grid;grid-template-columns:repeat(auto-fill,minmax(240px,1fr));gap:16px;padding:24px}.kart{background:#fff;border:1px solid #d8dee9;border-top:5px solid #dc2626;border-radius:8px;padding:12px;min-width:0}.kart.gizli-konformans{border-style:dashed}.kart h2{font-size:16px;margin:10px 0 4px}.kart code{font-size:12px}.önizleme{display:block;width:100%;height:135px;object-fit:contain;border-radius:5px;background:#fff}.önizleme.boş{display:grid;place-items:center;background:repeating-linear-gradient(135deg,#edf0f5,#edf0f5 10px,#e3e7ee 10px,#e3e7ee 20px);color:#6b7280}.kaynak-rozeti{font-size:12px;color:#475569}.rozet{font-size:12px;color:#b91c1c}
@media(prefers-color-scheme:dark){:root{background:#111827;color:#e5e7eb}.üst,.kart{background:#1f2937;border-color:#374151}.önizleme.boş{background:#111827}}
</style></head><body><header class="üst"><h1>Cizelge · ECharts 6.1 uyum kanıtı</h1><div class="özet"><span><strong>${özet.statik_görsel_kanıtlı}</strong> / ${özet.kapsam_içi_toplam} kilitli görsel kanıt</span><span>${özet.tam_kanıtlı} / ${özet.kapsam_içi_toplam} tüm kapılar tam</span><span>${özet.görünür_kapsam_içi} resmî galeri kartı</span><span>${özet.gizli_conformance} gizli doğrulama kartı</span><span>${özet.geo_map_kapsam_dışı} Geo/Map dışı</span><span>${özet.gl_3d_kapsam_dışı} GL/3B dışı</span></div></header><main class="ızgara">${kartlar}</main></body></html>`;
}

function kilitToml(echartsPaket, commitler) {
  return `# Bu dosya tools/uyum/uret.mjs tarafından üretilir.\n`+
    `snapshot_tarihi = "2026-07-17"\n`+
    `manifest_sürümü = 1\n\n`+
    `[echarts]\n`+
    `sürüm = "${echartsPaket.version}"\n`+
    `commit = "${commitler.echarts}"\n`+
    `yol = "../echarts"\n`+
    `lisans = "Apache-2.0"\n\n`+
    `[zrender]\n`+
    `sürüm = "${echartsPaket.dependencies.zrender}"\n`+
    `kaynak = "echarts/package.json"\n`+
    `lisans = "BSD-3-Clause"\n\n`+
    `[echarts_examples]\n`+
    `dal = "gh-pages"\n`+
    `commit = "${commitler.examples}"\n`+
    `yol = "../echarts-examples"\n`+
    `lisans = "Apache-2.0"\n\n`+
    `[gpui]\n`+
    `commit = "${commitler.gpui}"\n`+
    `yol = "../gpui"\n`+
    `paketler = ["gpui", "gpui_platform"]\n`+
    `lisans_belgesi = "../gpui/NOTICE"\n\n`+
    `[render_profili]\n`+
    `varsayılan_viewport = [700, 525]\n`+
    `varsayılan_çıktı = [600, 450]\n`+
    `device_pixel_ratio = 1\n`+
    `locale = "en"\n`+
    `saat_dilimi = "UTC"\n`;
}

function üret(hedef) {
  const echartsPaket = JSON.parse(oku(path.join(ECHARTS, 'package.json')));
  const commitler = {
    echarts: gitCommit(ECHARTS),
    examples: gitCommit(ÖRNEKLER),
    gpui: gitCommit(GPUI)
  };
  if (commitler.echarts !== BEKLENEN.echartsCommit) hata(`ECharts commit farklı: ${commitler.echarts}`);
  if (commitler.examples !== BEKLENEN.examplesCommit) hata(`örnekler commit farklı: ${commitler.examples}`);
  if (commitler.gpui !== BEKLENEN.gpuiCommit) hata(`GPUI commit farklı: ${commitler.gpui}`);
  if (echartsPaket.version !== BEKLENEN.echartsVersion) hata(`ECharts sürümü farklı: ${echartsPaket.version}`);
  if (echartsPaket.dependencies.zrender !== BEKLENEN.zrenderVersion) hata(`zrender sürümü farklı: ${echartsPaket.dependencies.zrender}`);

  const çekirdek = jsonDizisiOku(path.join(ÖRNEKLER, 'src/data/chart-list-data.js'));
  const gl = jsonDizisiOku(path.join(ÖRNEKLER, 'src/data/chart-list-data-gl.js'));
  const görselKanıtlar = görselKanıtlarıOku();
  const manifest = [
    ...çekirdek.map((kayıt) => manifestKaydı(kayıt, false, görselKanıtlar)),
    ...gl.map((kayıt) => manifestKaydı(kayıt, true, görselKanıtlar))
  ];
  const çekirdekManifest = manifest.filter((kayıt) => kayıt.kapsam_durumu !== 'kapsam_dışı_gl_3d');
  const kapsamİçi = çekirdekManifest.filter((kayıt) => kayıt.kapsam_durumu !== 'kapsam_dışı_geo_map');
  const görünür = kapsamİçi.filter((kayıt) => kayıt.resmi_sayfada_görünür);
  const gizli = kapsamİçi.filter((kayıt) => kayıt.noExplore);
  const geo = çekirdekManifest.filter((kayıt) => kayıt.kapsam_durumu === 'kapsam_dışı_geo_map');
  const sayılar = {
    çekirdek: çekirdek.length,
    gl: gl.length,
    kapsamİçi: kapsamİçi.length,
    görünür: görünür.length,
    gizli: gizli.length,
    geo: geo.length
  };
  for (const [ad, beklenen] of Object.entries({
    çekirdek: BEKLENEN.çekirdek,
    gl: BEKLENEN.gl,
    kapsamİçi: BEKLENEN.kapsamİçi,
    görünür: BEKLENEN.görünür,
    gizli: BEKLENEN.gizli,
    geo: BEKLENEN.geo
  })) {
    if (sayılar[ad] !== beklenen) hata(`${ad}: ${sayılar[ad]}, beklenen ${beklenen}`);
  }
  const matris = özellikMatrisi();
  const özet = {
    şema_sürümü: 1,
    üretici: 'tools/uyum/uret.mjs',
    echarts_commit: commitler.echarts,
    examples_commit: commitler.examples,
    çekirdek_toplam: çekirdek.length,
    gl_toplam: gl.length,
    kapsam_içi_toplam: kapsamİçi.length,
    görünür_kapsam_içi: görünür.length,
    gizli_conformance: gizli.length,
    geo_map_kapsam_dışı: geo.length,
    gl_3d_kapsam_dışı: gl.length,
    option_satırı: matris.length,
    statik_görsel_kanıtlı: kapsamİçi.filter(
      (kayıt) => kayıt.kanıt['statik_görsel'] === 'tam_kanıtlı'
    ).length,
    tam_kanıtlı: kapsamİçi.filter((kayıt) => kayıt.kapsam_durumu === 'tam_kanıtlı').length,
    kategori_sırası: KATEGORİ_SIRASI,
    kategori_sayıları: Object.fromEntries(KATEGORİ_SIRASI.map((kategori) => [
      kategori,
      görünür.filter((kayıt) => kayıt.kategoriler.includes(kategori)).length
    ]))
  };

  fs.mkdirSync(path.join(hedef, 'senaryolar'), { recursive: true });
  fs.mkdirSync(path.join(hedef, 'rapor'), { recursive: true });
  fs.writeFileSync(path.join(hedef, 'kaynak_kilidi.toml'), kilitToml(echartsPaket, commitler));
  fs.writeFileSync(path.join(hedef, 'galeri_manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
  fs.writeFileSync(path.join(hedef, 'ozellik_matrisi.json'), `${JSON.stringify(matris, null, 2)}\n`);
  fs.writeFileSync(path.join(hedef, 'ozet.json'), `${JSON.stringify(özet, null, 2)}\n`);
  fs.writeFileSync(path.join(hedef, 'rapor/index.html'), raporHtml(manifest, özet));
  fs.writeFileSync(path.join(hedef, 'dis_varliklar.toml'), '# Uyumlu dış varlıklar kanıt üretildikçe buraya eklenir.\nsürüm = 1\nvarlıklar = []\n');
  for (const kayıt of kapsamİçi) {
    const güvenliAd = kayıt.id.replaceAll('/', '__');
    fs.writeFileSync(path.join(hedef, 'senaryolar', `${güvenliAd}.toml`), senaryoMetni(kayıt));
  }
  return özet;
}

function dosyaHaritası(dizin) {
  if (!fs.existsSync(dizin)) return new Map();
  return new Map(dosyalarıYürü(dizin, () => true).map((dosya) => [göreli(dosya, dizin), sha256(fs.readFileSync(dosya))]));
}

function denetle() {
  const geçici = fs.mkdtempSync(path.join(os.tmpdir(), 'cizelge-uyum-'));
  try {
    üret(geçici);
    const beklenen = dosyaHaritası(geçici);
    const gerçek = dosyaHaritası(UYUM);
    const farklar = [];
    for (const [dosya, hash] of beklenen) {
      if (gerçek.get(dosya) !== hash) farklar.push(dosya);
    }
    for (const dosya of gerçek.keys()) {
      if (!beklenen.has(dosya)) farklar.push(`${dosya} (fazla)`);
    }
    if (farklar.length > 0) hata(`üretilmiş uyum dosyaları güncel değil:\n- ${farklar.join('\n- ')}`);
    process.stdout.write('Uyum kaynak kilidi ve üretilmiş dosyalar güncel.\n');
  } finally {
    fs.rmSync(geçici, { recursive: true, force: true });
  }
}

if (process.argv.includes('--check')) denetle();
else {
  fs.rmSync(UYUM, { recursive: true, force: true });
  const özet = üret(UYUM);
  process.stdout.write(`${JSON.stringify(özet, null, 2)}\n`);
}
