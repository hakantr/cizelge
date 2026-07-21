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
  'line-marker': 'examples/uyum_fixture.rs#line_marker',
  'grid-multiple': 'examples/uyum_fixture.rs#grid_multiple',
  'intraday-breaks-1': 'examples/uyum_fixture.rs#intraday_breaks_1',
  'intraday-breaks-2': 'examples/uyum_fixture.rs#intraday_breaks_2',
  'mix-zoom-on-value': 'examples/uyum_fixture.rs#mix_zoom_on_value',
  'bar-polar-real-estate': 'examples/uyum_fixture.rs#bar_polar_real_estate',
  'polar-roundCap': 'examples/uyum_fixture.rs#polar_round_cap',
  'polar-endAngle': 'examples/uyum_fixture.rs#polar_end_angle',
  'bar-histogram': 'examples/uyum_fixture.rs#bar_histogram',
  'gauge': 'examples/uyum_fixture.rs#gauge',
  'radar': 'examples/radar.rs',
  'funnel': 'examples/huni.rs',
  'gauge-simple': 'examples/uyum_fixture.rs#gauge_simple',
  'gauge-speed': 'examples/uyum_fixture.rs#gauge_speed',
  'treemap-simple': 'examples/agac_haritasi.rs',
  'sunburst-simple': 'examples/gunes.rs',
  'tree-basic': 'examples/agac.rs',
  'sankey-simple': 'examples/sankey.rs',
  'graph-simple': 'examples/grafo.rs',
  'chord-simple': 'examples/kiris.rs',
  'themeRiver-basic': 'examples/tema_nehri.rs',
  'parallel-simple': 'examples/paralel.rs'
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
      metrik: kanıtDosyası(tam)
    };
    kare.geçti = kare.geçti
      && [referans, gerçek, fark, kare.metrik].every(Boolean)
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
            veri_biçimleri: [],
            koordinat_dalları: [],
            kaynak: {
              dosya: göreli(dosya, ECHARTS),
              satır: konum.line + 1,
              sembol: düğüm.name.text
            },
            testler: [],
            galeri_örnekleri: [],
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
