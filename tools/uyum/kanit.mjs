#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import pixelmatch from 'pixelmatch';
import { PNG } from 'pngjs';
import sharp from 'sharp';
import { ssim } from 'ssim.js';

const ARAÇ = path.dirname(new URL(import.meta.url).pathname);
const KÖK = path.resolve(ARAÇ, '../..');
const COMMIT = '74e9e09a0b5687fdd34319121ac73b3022d1483c';
const TABAN = path.join(KÖK, 'testler/gorsel');
const REFERANS = path.join(TABAN, 'referans', COMMIT, 'default');
const REFERANS_SAHNE = path.join(REFERANS, 'sahneler');
const GERÇEK = path.join(TABAN, 'gerçek', 'default');
const FARK = path.join(TABAN, 'fark', 'default');
const METRİK = path.join(TABAN, 'metrikler');
const SAHNE = path.join(TABAN, 'sahneler');
const RAPOR = path.join(TABAN, 'rapor');
const EŞİK = Object.freeze({ pixelmatch: 0.1, değişenPikselOranı: 0.01, ssim: 0.99 });
const REFERANS_YENİLE = process.argv.includes('--referans-yenile');

const ağaçKarşılaştırması = (özet) => ({
  tipografiSigma: 0.8,
  sahneÖzeti: {
    şema_sürümü: 1,
    koordinat_adımı: 0.001,
    ...özet
  }
});

const treemapKarşılaştırması = () => ({
  tipografiSigma: 0.8,
  // Her hücrenin sınırı ve fill/border katmanı, kilitli ECharts sahnesiyle
  // doğrudan karşılaştırılır; toplam raster oranı ince çizgiyi gizleyemez.
  sahneReferansı: true
});

const sunburstKarşılaştırması = () => ({
  tipografiSigma: 0.8,
  // Her sektörün açı/yarıçap/style/etiket geometrisi kilitli ECharts
  // sahnesiyle alan alan karşılaştırılır.
  sahneReferansı: true
});

const sankeyKarşılaştırması = () => ({
  tipografiSigma: 0.8,
  // Düğüm kutuları ve her bağlantının tam Bézier şeridi, renk uçlarıyla
  // birlikte kilitli ECharts sahnesine karşı denetlenir.
  sahneReferansı: true
});

const chordKarşılaştırması = () => ({
  tipografiSigma: 0.8,
  // Sektör açı/yarıçap/köşe/etiket alanları ile her şeridin iki uç
  // istifi ve etkin renk uçları toplam raster oranından bağımsızdır.
  sahneReferansı: true
});

const SENARYOLAR = [
  { id: 'bar-histogram', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'funnel', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'funnel-align', tür: 'statik', genişlik: 1400, yükseklik: 1050,
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'funnel-customize', tür: 'statik', genişlik: 1000, yükseklik: 750,
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'funnel-mutiple', tür: 'statik', genişlik: 1000, yükseklik: 750,
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  { id: 'radar', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'radar-aqi', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'radar-custom', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'radar2', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'radar-multiple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'parallel-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'parallel-aqi', tür: 'statik',
    karşılaştırma: { tipografiSigma: 0.8 },
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'parallel-nutrients', tür: 'statik',
    karşılaştırma: {
      tipografiSigma: 0.8,
      sahneÖzeti: {
        şema_sürümü: 1,
        çizgi_sayısı: 7637,
        nokta_sayısı: 114555,
        koordinat_sayısı: 229110,
        koordinat_adımı: 0.001,
        fnv1a_64: 'd3f9efb47fd5e2d7'
      }
    },
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'doc-example/parallel-all', tür: 'statik',
    karşılaştırma: { tipografiSigma: 0.8 },
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'tree-basic', tür: 'statik',
    karşılaştırma: ağaçKarşılaştırması({ seri_sayısı: 1, düğüm_sayısı: 52, kenar_sayısı: 51,
      kenar_yolu_sayısı: 51, etiket_sayısı: 52, daraltılmış_düğüm_sayısı: 12,
      koordinat_sayısı: 668, fnv1a_64: 'ac13658335726e2b' }),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'tree-legend', tür: 'statik',
    karşılaştırma: ağaçKarşılaştırması({ seri_sayısı: 2, düğüm_sayısı: 67, kenar_sayısı: 65,
      kenar_yolu_sayısı: 65, etiket_sayısı: 67, daraltılmış_düğüm_sayısı: 2,
      koordinat_sayısı: 855, fnv1a_64: 'c59de5063e316fc4' }),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'tree-orient-bottom-top', tür: 'statik',
    karşılaştırma: ağaçKarşılaştırması({ seri_sayısı: 1, düğüm_sayısı: 111, kenar_sayısı: 110,
      kenar_yolu_sayısı: 110, etiket_sayısı: 111, daraltılmış_düğüm_sayısı: 15,
      koordinat_sayısı: 1435, fnv1a_64: '0df0b4713d1513fa' }),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'tree-orient-right-left', tür: 'statik',
    karşılaştırma: ağaçKarşılaştırması({ seri_sayısı: 1, düğüm_sayısı: 52, kenar_sayısı: 51,
      kenar_yolu_sayısı: 51, etiket_sayısı: 52, daraltılmış_düğüm_sayısı: 12,
      koordinat_sayısı: 668, fnv1a_64: '84162e15313ca35d' }),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'tree-polyline', tür: 'statik',
    karşılaştırma: ağaçKarşılaştırması({ seri_sayısı: 1, düğüm_sayısı: 85, kenar_sayısı: 84,
      kenar_yolu_sayısı: 8, etiket_sayısı: 85, daraltılmış_düğüm_sayısı: 0,
      koordinat_sayısı: 789, fnv1a_64: '37eea6c009dcc5ae' }),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'tree-radial', tür: 'statik',
    karşılaştırma: ağaçKarşılaştırması({ seri_sayısı: 1, düğüm_sayısı: 219, kenar_sayısı: 218,
      kenar_yolu_sayısı: 218, etiket_sayısı: 219, daraltılmış_düğüm_sayısı: 6,
      koordinat_sayısı: 2839, fnv1a_64: 'ad21b3a3594cf7a7' }),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'tree-vertical', tür: 'statik',
    karşılaştırma: ağaçKarşılaştırması({ seri_sayısı: 1, düğüm_sayısı: 111, kenar_sayısı: 110,
      kenar_yolu_sayısı: 110, etiket_sayısı: 111, daraltılmış_düğüm_sayısı: 15,
      koordinat_sayısı: 1435, fnv1a_64: 'cecc82c59e976726' }),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  ...[
    'treemap-sunburst-transition', 'treemap-disk', 'treemap-drill-down',
    'treemap-obama', 'treemap-show-parent', 'treemap-simple', 'treemap-visual'
  ].map((id) => ({
    id,
    tür: 'statik',
    karşılaştırma: treemapKarşılaştırması(),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  })),
  ...[
    { id: 'sunburst-simple' },
    { id: 'sunburst-borderRadius' },
    { id: 'sunburst-label-rotate' },
    { id: 'sunburst-monochrome' },
    { id: 'sunburst-visualMap' },
    { id: 'sunburst-drink', genişlik: 1000, yükseklik: 750 },
    { id: 'sunburst-book', genişlik: 820, yükseklik: 615 }
  ].map((senaryo) => ({
    ...senaryo,
    tür: 'statik',
    karşılaştırma: sunburstKarşılaştırması(),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  })),
  ...[
    'sankey-energy', 'sankey-itemstyle', 'sankey-levels',
    'sankey-nodeAlign-left', 'sankey-nodeAlign-right',
    'sankey-simple', 'sankey-vertical'
  ].map((id) => ({
    id,
    tür: 'statik',
    karşılaştırma: sankeyKarşılaştırması(),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  })),
  ...[
    'chord-simple', 'chord-minAngle', 'chord-lineStyle-color', 'chord-style'
  ].map((id) => ({
    id,
    tür: 'statik',
    karşılaştırma: chordKarşılaştırması(),
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  })),
  { id: 'themeRiver-basic', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'themeRiver-lastfm', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'gauge', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'gauge-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'gauge-speed', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'gauge-progress', tür: 'statik', genişlik: 800, yükseklik: 600,
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  { id: 'gauge-stage', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'gauge-grade', tür: 'statik', genişlik: 800, yükseklik: 600,
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  { id: 'gauge-multi-title', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'gauge-temperature', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'gauge-ring', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'gauge-barometer', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'gauge-clock', tür: 'statik', genişlik: 1000, yükseklik: 750,
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'gauge-car', tür: 'statik', genişlik: 800, yükseklik: 600,
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  { id: 'line-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'line-markline',
    tür: 'statik',
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  { id: 'line-marker', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar1', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'mix-line-bar', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'multiple-x-axis', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'multiple-y-axis', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-smooth', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'area-basic', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'area-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'area-time-axis', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'area-rainfall', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'dynamic-data2',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'ipucu', kare: 1, durum: 'ipucu' },
      { ad: 'son', kare: 1, durum: 'son' }
    ]
  },
  {
    id: 'dynamic-data',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'ipucu', kare: 1, durum: 'ipucu' },
      { ad: 'son', kare: 1, durum: 'son' }
    ]
  },
  { id: 'line-sections', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'area-pieces', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-gradient', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-aqi', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'confidence-band', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-race', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'grid-multiple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'intraday-breaks-1', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'intraday-breaks-2', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-stack', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-style', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-step', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-in-cartesian-coordinate-system', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-y-category', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-log', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-polar', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-polar2', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'line-function', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bump-chart', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'area-stack', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'area-stack-gradient', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-background', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-tick-align', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-data-color', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-stack-borderRadius', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-y-category', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-y-category-stack', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-negative2', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-negative', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-stack', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-waterfall', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-waterfall2', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bar-stack-normalization', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'bar-brush',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'seçim', kare: 1, durum: 'seçim' }
    ]
  },
  {
    id: 'bar-polar-label-radial',
    tür: 'statik',
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'bar-polar-label-tangential',
    tür: 'statik',
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'bar-polar-stack',
    tür: 'statik',
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'bar-polar-stack-radial',
    tür: 'statik',
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'bar-polar-real-estate',
    tür: 'statik',
    genişlik: 800,
    yükseklik: 600,
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'polar-roundCap',
    tür: 'statik',
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'polar-endAngle',
    tür: 'statik',
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  { id: 'bar-label-rotation', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'bar-breaks-simple',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'genişlet', kare: 1, durum: 'genişlet' },
      { ad: 'daralt', kare: 1, durum: 'daralt' }
    ]
  },
  {
    id: 'bar-breaks-brush',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'fırça', kare: 1, durum: 'fırça' },
      { ad: 'sıfırla', kare: 1, durum: 'sıfırla' }
    ]
  },
  {
    id: 'bar-gradient',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'vurgu', kare: 1, durum: 'vurgu' },
      { ad: 'yakınlaştır', kare: 1, durum: 'yakınlaştır' }
    ]
  },
  { id: 'data-transform-sort-bar', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'dataset-simple0', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'dataset-simple1', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'dataset-series-layout-by', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'dataset-encode0', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'dataset-default', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'data-transform-multiple-pie', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'dataset-link',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'son', kare: 1, durum: 'son' }
    ]
  },
  { id: 'data-transform-filter', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'dataset-encode1', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'data-transform-aggregate', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'boxplot-multi', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'boxplot-light-velocity', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'boxplot-light-velocity2', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-anscombe-quartet', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-jitter', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'doc-example/scatter-jitter-avoidOverlap',
    tür: 'statik',
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  { id: 'scatter-punchCard', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-polar-punchCard', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-single-axis', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'bubble-gradient', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-label-align-top', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-label-align-right', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-aqi-color', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-weight', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'scatter-aggregate-bar',
    tür: 'animasyon',
    kareler: [
      { ad: 'scatter', kare: 1, durum: 'başlangıç' },
      { ad: 'bar', kare: 1, durum: 'bar' },
      { ad: 'scatter-dönüş', kare: 1, durum: 'scatter-return' }
    ]
  },
  {
    id: 'scatter-symbol-morph',
    tür: 'animasyon',
    kareler: [
      'round-rect', 'circle', 'heart', 'happy', 'evil', 'hipster',
      'shocked', 'pie', 'users', 'mug', 'plane'
    ].map((ad, sıra) => ({ ad, kare: 1, durum: `shape-${sıra}` }))
  },
  { id: 'scatter-large', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-nebula', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'scatter-nutrients',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'fat-fiber', kare: 1, durum: 'axes-fat-fiber' }
    ]
  },
  {
    id: 'scatter-nutrients-matrix',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'zoom-left', kare: 1, durum: 'zoom-left' }
    ]
  },
  { id: 'scatter-stream-visual', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-painter-choice', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-clustering', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'scatter-clustering-process',
    tür: 'etkileşim',
    kareler: [0, 1, 2, 3, 4, 5].map((sıra) => ({
      ad: `adım-${sıra}`,
      kare: 1,
      durum: `step-${sıra}`
    }))
  },
  { id: 'scatter-exponential-regression', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-linear-regression', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-polynomial-regression', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'scatter-logarithmic-regression', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'candlestick-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'candlestick-sh', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'candlestick-large', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'candlestick-brush', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'candlestick-sh-2015', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'heatmap-cartesian',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'ipucu', kare: 1, durum: 'ipucu' },
      { ad: 'aralık', kare: 1, durum: 'aralık' }
    ]
  },
  { id: 'heatmap-large', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'heatmap-large-piecewise',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'parça', kare: 1, durum: 'parça' }
    ]
  },
  { id: 'matrix-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'matrix-correlation-heatmap', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'matrix-correlation-scatter', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'matrix-covariance', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'matrix-graph', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'matrix-pie', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'matrix-confusion', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'matrix-grid-layout', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'matrix-stock', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'matrix-sparkline', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'matrix-periodic-table', tür: 'statik',
    karşılaştırma: { tipografiSigma: 0.8 },
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  {
    id: 'matrix-mbti', tür: 'statik',
    karşılaştırma: { tipografiSigma: 0.8 },
    kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }]
  },
  { id: 'matrix-mini-bar-data-collection', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'calendar-heatmap', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'calendar-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'calendar-vertical', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'calendar-horizontal', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'calendar-effectscatter',
    tür: 'animasyon',
    kareler: [0, 0.25, 0.5, 0.75, 1].map((kare, sıra) => ({ ad: `kare-${sıra}`, kare, durum: 'başlangıç' }))
  },
  { id: 'calendar-graph', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'calendar-lunar', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'calendar-pie', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'custom-calendar-icon', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'calendar-charts',
    tür: 'animasyon',
    genişlik: 1000,
    yükseklik: 750,
    kareler: [0, 0.25, 0.5, 0.75, 1].map((kare, sıra) => ({ ad: `kare-${sıra}`, kare, durum: 'başlangıç' }))
  },
  { id: 'pie-nest', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-rich-text', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-doughnut', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-roseType-simple', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-roseType', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-legend', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-custom', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-pattern', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-labelLine-adjust', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-padAngle', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-half-donut', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-borderRadius', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  { id: 'pie-alignTo', tür: 'statik', kareler: [{ ad: 'son', kare: 1, durum: 'başlangıç' }] },
  {
    id: 'scatter-effect',
    tür: 'animasyon',
    kareler: [0, 0.25, 0.5, 0.75, 1].map((kare, sıra) => ({ ad: `kare-${sıra}`, kare, durum: 'başlangıç' }))
  },
  {
    id: 'mix-zoom-on-value',
    tür: 'etkileşim',
    kareler: [
      { ad: 'başlangıç', kare: 1, durum: 'başlangıç' },
      { ad: 'son', kare: 1, durum: 'son' }
    ]
  }
];

function dizin(d) {
  fs.mkdirSync(d, { recursive: true });
}

function dosyaKimliği(id) {
  return id.replaceAll('/', '__');
}

function pngOku(dosya) {
  return PNG.sync.read(fs.readFileSync(dosya));
}

function aynıPiksellerMi(aDosyası, bDosyası) {
  const a = pngOku(aDosyası);
  const b = pngOku(bDosyası);
  return a.width === b.width
    && a.height === b.height
    && Buffer.compare(a.data, b.data) === 0;
}

async function referansıYenile(senaryo, kare, referans, referansSahne, sonek) {
  const adaylar = [];
  const sahneAdayları = [];
  const dosyaId = dosyaKimliği(senaryo.id);
  for (const geçiş of [1, 2]) {
    const ham = path.join(REFERANS, `.ham-${dosyaId}${sonek}-${geçiş}.png`);
    const aday = path.join(REFERANS, `.aday-${dosyaId}${sonek}-${geçiş}.png`);
    const argümanlar = [
      path.join(ARAÇ, 'echarts_referans.mjs'),
      '--id', senaryo.id,
      '--output', ham,
      '--frame', String(kare.kare),
      '--state', kare.durum,
      '--width', String(senaryo.genişlik ?? 700),
      '--height', String(senaryo.yükseklik ?? 525)
    ];
    if (referansSahne) {
      const sahneAdayı = `${referansSahne}.aday-${geçiş}`;
      argümanlar.push('--scene-output', sahneAdayı);
      sahneAdayları.push(sahneAdayı);
    }
    execFileSync('node', argümanlar, { cwd: KÖK, stdio: 'inherit' });
    await sharp(ham).resize(600, 450).toFile(aday);
    fs.rmSync(ham, { force: true });
    adaylar.push(aday);
  }
  if (!aynıPiksellerMi(adaylar[0], adaylar[1])) {
    for (const aday of adaylar) fs.rmSync(aday, { force: true });
    for (const aday of sahneAdayları) fs.rmSync(aday, { force: true });
    throw new Error(`${senaryo.id}${sonek}: ECharts referansı iki ardışık üretimde kararlı değil`);
  }
  fs.renameSync(adaylar[0], referans);
  fs.rmSync(adaylar[1], { force: true });
  if (referansSahne) {
    if (Buffer.compare(fs.readFileSync(sahneAdayları[0]), fs.readFileSync(sahneAdayları[1])) !== 0) {
      for (const aday of sahneAdayları) fs.rmSync(aday, { force: true });
      throw new Error(`${senaryo.id}${sonek}: ECharts Treemap sahnesi kararlı değil`);
    }
    dizin(path.dirname(referansSahne));
    fs.renameSync(sahneAdayları[0], referansSahne);
    fs.rmSync(sahneAdayları[1], { force: true });
  }
}

function görüntüMetrikleri(referans, gerçek, farkDosyası) {
  if (referans.width !== gerçek.width || referans.height !== gerçek.height) {
    return {
      geçti: false,
      hata: `boyut farklı: ${referans.width}x${referans.height} / ${gerçek.width}x${gerçek.height}`,
      değişen_piksel: referans.width * referans.height,
      değişen_piksel_oranı: 1,
      ssim: 0
    };
  }
  const fark = new PNG({ width: referans.width, height: referans.height });
  const değişen = pixelmatch(referans.data, gerçek.data, fark.data, referans.width, referans.height, {
    threshold: EŞİK.pixelmatch,
    includeAA: false
  });
  dizin(path.dirname(farkDosyası));
  fs.writeFileSync(farkDosyası, PNG.sync.write(fark));
  const oran = değişen / (referans.width * referans.height);
  const benzerlik = ssim(
    { data: referans.data, width: referans.width, height: referans.height },
    { data: gerçek.data, width: gerçek.width, height: gerçek.height }
  ).mssim;
  return {
    geçti: oran <= EŞİK.değişenPikselOranı && benzerlik >= EŞİK.ssim,
    değişen_piksel: değişen,
    değişen_piksel_oranı: oran,
    ssim: benzerlik
  };
}

async function tipografiyiNormalizeEt(görüntü, sigma) {
  const veri = await sharp(görüntü.data, {
    raw: { width: görüntü.width, height: görüntü.height, channels: 4 }
  })
    .blur(sigma)
    .raw()
    .toBuffer();
  return { data: veri, width: görüntü.width, height: görüntü.height };
}

async function karşılaştır(
  referansDosyası,
  gerçekDosyası,
  farkDosyası,
  normalizeFarkDosyası,
  karşılaştırma = null
) {
  const referans = pngOku(referansDosyası);
  const gerçek = pngOku(gerçekDosyası);
  const ham = görüntüMetrikleri(referans, gerçek, farkDosyası);
  const sigma = karşılaştırma?.tipografiSigma;
  if (!(Number.isFinite(sigma) && sigma >= 0.3) || ham.hata) {
    return ham;
  }
  const [normalizeReferans, normalizeGerçek] = await Promise.all([
    tipografiyiNormalizeEt(referans, sigma),
    tipografiyiNormalizeEt(gerçek, sigma)
  ]);
  const normalize = görüntüMetrikleri(
    normalizeReferans,
    normalizeGerçek,
    normalizeFarkDosyası
  );
  return {
    ...normalize,
    ham: {
      geçti: ham.geçti,
      değişen_piksel: ham.değişen_piksel,
      değişen_piksel_oranı: ham.değişen_piksel_oranı,
      ssim: ham.ssim
    },
    tipografi_normalizasyonu: {
      gaussian_sigma: sigma,
      açıklama: 'İki görüntüye de aynı Gauss çekirdeği uygulanır; maske ve eşik değişikliği yoktur.'
    }
  };
}

// Toplam piksel oranı, ince fakat anlamlı bir eksen çizgisinin tamamen
// örtülmesini tek başına yakalayamaz. Kritik geometri kontrolleri, kilitli
// kartın bilinen semantik örnek noktalarını ayrı bir geçiş kapısı yapar.
function yapısalKontroller(senaryo, referansDosyası, gerçekDosyası) {
  const referansGörüntü = pngOku(referansDosyası);
  const görüntü = pngOku(gerçekDosyası);
  const pikselOku = (kaynak, x, y) => {
    const başlangıç = (y * kaynak.width + x) * 4;
    return Array.from(kaynak.data.subarray(başlangıç, başlangıç + 3));
  };
  const piksel = (x, y) => pikselOku(görüntü, x, y);
  const renkKatmanıÖrneği = (x, y, tolerans = 3) => {
    const referans = pikselOku(referansGörüntü, x, y);
    const gerçek = piksel(x, y);
    const doygunluk = Math.max(...referans) - Math.min(...referans);
    const enBüyükFark = Math.max(...referans.map((kanal, sıra) => Math.abs(kanal - gerçek[sıra])));
    return {
      x,
      y,
      referans,
      gerçek,
      en_büyük_kanal_farkı: enBüyükFark,
      geçti: doygunluk >= 20 && enBüyükFark <= tolerans
    };
  };
  const nötrKoyuÖrnek = (x, y) => {
    const rgb = piksel(x, y);
    const sapma = Math.max(...rgb) - Math.min(...rgb);
    const parlaklık = rgb.reduce((toplam, kanal) => toplam + kanal, 0) / rgb.length;
    return { x, y, rgb, geçti: sapma <= 18 && parlaklık <= 190 };
  };
  const tabanÇizgisi = (ad, açıklama, y, xler) => {
    const örnekler = xler.map((x) => nötrKoyuÖrnek(x, y));
    return { ad, geçti: örnekler.every((örnek) => örnek.geçti), açıklama, örnekler };
  };
  // Bir piksellik eksenler toplam fark oranında kolayca kaybolur. Referans
  // ve gerçek görüntüde beklenen konumun ±2 px çevresindeki en yüksek yerel
  // kontrastı ayrı ölçerek eksen katmanını zorunlu kapı yapar. Arka plan,
  // çizginin normalinde ±5 px örneklenir; bu hem açık hem koyu temada çalışır.
  const eksenÇizgisiÖrneği = (x, y, dikey, nötrHedef = null) => {
    const tek = (kaynak) => {
      const adaylar = [];
      for (let kayma = -2; kayma <= 2; kayma += 1) {
        const px = x + (dikey ? kayma : 0);
        const py = y + (dikey ? 0 : kayma);
        const rgb = pikselOku(kaynak, px, py);
        const önce = pikselOku(kaynak, px + (dikey ? -5 : 0), py + (dikey ? 0 : -5));
        const sonra = pikselOku(kaynak, px + (dikey ? 5 : 0), py + (dikey ? 0 : 5));
        const arkaplan = önce.map((kanal, sıra) => (kanal + sonra[sıra]) / 2);
        const kontrast = Math.max(...rgb.map((kanal, sıra) => Math.abs(kanal - arkaplan[sıra])));
        const parlaklık = rgb.reduce((toplam, kanal) => toplam + kanal, 0) / rgb.length;
        const nötrlükSapması = Math.max(...rgb) - Math.min(...rgb);
        const nötrPuan = Number.isFinite(nötrHedef)
          ? Math.abs(parlaklık - nötrHedef) + nötrlükSapması
          : null;
        adaylar.push({
          kayma,
          x: px,
          y: py,
          rgb,
          arkaplan,
          kontrast,
          parlaklık,
          nötrlük_sapması: nötrlükSapması,
          ...(nötrPuan === null ? {} : { nötr_puan: nötrPuan })
        });
      }
      return adaylar.sort((a, b) => (
        Number.isFinite(nötrHedef)
          ? a.nötr_puan - b.nötr_puan || b.kontrast - a.kontrast
          : b.kontrast - a.kontrast
      ))[0];
    };
    const referans = tek(referansGörüntü);
    const gerçek = tek(görüntü);
    return {
      x,
      y,
      yön: dikey ? 'dikey' : 'yatay',
      referans,
      gerçek,
      geçti: referans.kontrast >= 18
        && gerçek.kontrast >= 18
        && Math.abs(referans.kayma - gerçek.kayma) <= 2
    };
  };
  const paralelEksenKapısı = (
    ad,
    açıklama,
    konumlar,
    örnekKonumlar,
    dikey,
    nötrHedef = null
  ) => {
    const örnekler = konumlar.flatMap((konum) => örnekKonumlar.map((örnekKonum) => (
      dikey
        ? eksenÇizgisiÖrneği(konum, örnekKonum, true, nötrHedef)
        : eksenÇizgisiÖrneği(örnekKonum, konum, false, nötrHedef)
    )));
    return { ad, geçti: örnekler.every((örnek) => örnek.geçti), açıklama, örnekler };
  };

  if (senaryo.id === 'dataset-encode0') {
    const x = 172;
    const örnekler = [72, 106, 141, 176, 211, 246, 281, 316, 351]
      .map((y) => nötrKoyuÖrnek(x, y));
    return [{
      ad: 'kategori_taban_çizgisi',
      geçti: örnekler.every((örnek) => örnek.geçti),
      açıklama: 'Y ekseni taban vuruşu dokuz barın başlangıcında kesintisiz görünmeli',
      örnekler
    }];
  }

  if (senaryo.id === 'calendar-simple') {
    const aySınırıÖrnekleri = [
      [154, 77],
      [222, 85],
      [291, 94],
      [360, 120],
      [428, 137],
      [514, 145],
      [582, 128]
    ].map(([x, y]) => nötrKoyuÖrnek(x, y));
    return [
      tabanÇizgisi(
        'takvim_üst_sınırı',
        'Takvim üst sınırı ısı hücrelerinin altında kalmadan tüm genişlikte görünmeli',
        51,
        [69, 100, 200, 300, 400, 500, 598]
      ),
      tabanÇizgisi(
        'takvim_alt_sınırı',
        'Takvim alt sınırı ısı hücrelerinin altında kalmadan tüm genişlikte görünmeli',
        171,
        [69, 100, 200, 300, 400, 500, 598]
      ),
      {
        ad: 'takvim_ay_ayırıcıları',
        geçti: aySınırıÖrnekleri.every((örnek) => örnek.geçti),
        açıklama: 'Ay ayırıcılarının merdiven vuruşları seri hücrelerinin üstünde görünmeli',
        örnekler: aySınırıÖrnekleri
      }
    ];
  }

  if (senaryo.id === 'parallel-simple') {
    return [paralelEksenKapısı(
      'parallel_axis_taban_cizgileri',
      'Dört parallelAxis çizgisi seri polylinelerinin üstünde ve tüm boyda görünmeli',
      [69, 223, 377, 531],
      [90, 225, 360],
      true
    )];
  }

  if (senaryo.id === 'parallel-aqi') {
    return [paralelEksenKapısı(
      'parallel_axis_taban_cizgileri',
      'Koyu zemindeki sekiz parallelAxis çizgisi yoğun AQI çizgilerinin üstünde kalmalı',
      [30, 96, 162, 228, 294, 360, 426, 492],
      [80, 180, 290],
      true
    )];
  }

  if (senaryo.id === 'doc-example/parallel-all') {
    return [paralelEksenKapısı(
      'parallel_axis_taban_cizgileri',
      'Gizli resmî örneğin sekiz parallelAxis çizgisi seri katmanı tarafından örtülmemeli',
      [30, 100, 171, 241, 311, 381, 452, 522],
      [120, 240, 370],
      true
    )];
  }

  if (senaryo.id === 'parallel-nutrients') {
    return [paralelEksenKapısı(
      'parallel_axis_yatay_taban_cizgileri',
      'On beş dikey-layout parallelAxis tabanı 7.637 yumuşak çizginin üstünde kesintisiz görünmeli',
      [17, 44, 72, 99, 126, 153, 181, 208, 235, 262, 290, 317, 344, 371, 399],
      [260, 410, 570],
      false,
      153
    )];
  }

  if (senaryo.id === 'themeRiver-basic') {
    const beklenenRenkler = [
      [80, 112, 221],
      [182, 214, 52],
      [80, 83, 114],
      [255, 153, 77],
      [12, 168, 223],
      [255, 209, 10]
    ];
    const katmanÖrnekleri = [93, 158, 201, 244, 292, 340].map((y, sıra) => {
      const x = 85;
      const rgb = piksel(x, y);
      const beklenen = beklenenRenkler[sıra];
      return {
        x,
        y,
        rgb,
        beklenen,
        geçti: rgb.every((kanal, kanalSırası) => Math.abs(kanal - beklenen[kanalSırası]) <= 2)
      };
    });
    return [
      tabanÇizgisi(
        'single_axis_taban_çizgisi',
        'Tema nehri altındaki singleAxis taban çizgisi tüm genişlikte görünmeli',
        407,
        [40, 100, 200, 300, 400, 500, 560]
      ),
      {
        ad: 'alti_katmanli_siluet',
        geçti: katmanÖrnekleri.every((örnek) => örnek.geçti),
        açıklama: 'Resmî ilk güçlü kesitte altı katman doğru sıra ve renkte bulunmalı',
        örnekler: katmanÖrnekleri
      }
    ];
  }

  if (senaryo.id === 'themeRiver-lastfm') {
    return [tabanÇizgisi(
      'single_axis_taban_çizgisi',
      'dataMax singleAxis taban çizgisi tüm genişlikte görünmeli',
      428,
      [60, 100, 200, 300, 400, 500, 560]
    )];
  }

  if (senaryo.id === 'matrix-mbti') {
    const örnekler = [
      [142, 92], [210, 136], [232, 181], [300, 226], [322, 271],
      [390, 315], [412, 360], [480, 427], [435, 405], [187, 383]
    ].map(([x, y]) => renkKatmanıÖrneği(x, y));
    return [{
      ad: 'mbti_16x16_hücre_katmanları',
      geçti: örnekler.every((örnek) => örnek.geçti),
      açıklama: 'Dört grup boyunca heatmap tabanı ve iki aria/decal boya katmanı ham karede korunmalı',
      örnekler
    }];
  }

  if (senaryo.id === 'matrix-periodic-table') {
    const örnekler = [
      [4, 115], [4, 146], [73, 180], [112, 214], [448, 302],
      [459, 115], [500, 146], [73, 345], [110, 377], [550, 400]
    ].map(([x, y]) => renkKatmanıÖrneği(x, y, 2));
    return [{
      ad: 'periyodik_tablo_hücre_geometrisi',
      geçti: örnekler.every((örnek) => örnek.geçti),
      açıklama: 's/p/d/f bloklarının ham hücre dolguları resmî koordinatlarda aynı kalmalı',
      örnekler
    }];
  }

  return [];
}

function treemapSahneKontrolleri(senaryo, sahneDosyası, referansSahneDosyası) {
  if (!senaryo.karşılaştırma?.sahneReferansı) return [];
  if (!sahneDosyası || !fs.existsSync(sahneDosyası)) {
    return [{
      ad: 'treemap_hücre_sınırları_ve_katmanları',
      geçti: false,
      açıklama: 'Her Treemap hücresinin sınırı, fill/border katmanı ve leaf durumu doğrulanmalı',
      hata: 'Cizelge sahne kanıtı eksik'
    }];
  }
  if (!referansSahneDosyası || !fs.existsSync(referansSahneDosyası)) {
    return [{
      ad: 'treemap_hücre_sınırları_ve_katmanları',
      geçti: false,
      açıklama: 'Her Treemap hücresinin sınırı, fill/border katmanı ve leaf durumu doğrulanmalı',
      hata: 'kilitli ECharts sahne kanıtı eksik'
    }];
  }
  const gerçek = JSON.parse(fs.readFileSync(sahneDosyası, 'utf8'));
  const beklenen = JSON.parse(fs.readFileSync(referansSahneDosyası, 'utf8'));
  const uyuşmazlıklar = [];
  const ekle = (yol, beklenenDeğer, gerçekDeğer) => {
    if (uyuşmazlıklar.length < 40) {
      uyuşmazlıklar.push({ yol, beklenen: beklenenDeğer, gerçek: gerçekDeğer });
    }
  };
  for (const alan of ['şema_sürümü', 'tür', 'koordinat_adımı']) {
    if (gerçek[alan] !== beklenen[alan]) ekle(alan, beklenen[alan], gerçek[alan]);
  }
  const beklenenSeriler = beklenen.seriler || [];
  const gerçekSeriler = gerçek.seriler || [];
  if (gerçekSeriler.length !== beklenenSeriler.length) {
    ekle('seriler.length', beklenenSeriler.length, gerçekSeriler.length);
  }
  const koordinatlar = new Set([
    'alan.x', 'alan.y', 'alan.genişlik', 'alan.yükseklik',
    'x', 'y', 'genişlik', 'yükseklik', 'kenarlık_kalınlığı',
    'boşluk_genişliği', 'üst_yükseklik'
  ]);
  const eşitMi = (alan, a, b) => {
    if (koordinatlar.has(alan) && typeof a === 'number' && typeof b === 'number') {
      return Math.abs(a - b) <= 0.0011;
    }
    return JSON.stringify(a) === JSON.stringify(b);
  };
  for (let seriSırası = 0; seriSırası < Math.min(beklenenSeriler.length, gerçekSeriler.length); seriSırası += 1) {
    const bSeri = beklenenSeriler[seriSırası];
    const gSeri = gerçekSeriler[seriSırası];
    if (bSeri.seri_sırası !== gSeri.seri_sırası) {
      ekle(`seriler[${seriSırası}].seri_sırası`, bSeri.seri_sırası, gSeri.seri_sırası);
    }
    for (const alan of ['x', 'y', 'genişlik', 'yükseklik']) {
      if (!eşitMi(`alan.${alan}`, bSeri.alan?.[alan], gSeri.alan?.[alan])) {
        ekle(`seriler[${seriSırası}].alan.${alan}`, bSeri.alan?.[alan], gSeri.alan?.[alan]);
      }
    }
    const bDüğümler = bSeri.düğümler || [];
    const gDüğümler = gSeri.düğümler || [];
    if (bDüğümler.length !== gDüğümler.length) {
      ekle(`seriler[${seriSırası}].düğümler.length`, bDüğümler.length, gDüğümler.length);
    }
    for (let düğümSırası = 0; düğümSırası < Math.min(bDüğümler.length, gDüğümler.length); düğümSırası += 1) {
      const bDüğüm = bDüğümler[düğümSırası];
      const gDüğüm = gDüğümler[düğümSırası];
      for (const alan of [
        'veri_sırası', 'ad', 'derinlik', 'x', 'y', 'genişlik', 'yükseklik',
        'renk', 'kenarlık_rengi', 'kenarlık_kalınlığı', 'boşluk_genişliği',
        'üst_yükseklik', 'yaprak', 'inilebilir_yaprak'
      ]) {
        // Parent `style.fill`, ECharts veri görselinde tutulsa da renderer'da
        // boyanmaz; parent yalnız border/background katmanıdır. Fill rengi
        // ancak içerik dikdörtgeni çizilen, pozitif alanlı leaf/leafRoot için
        // kanıttır. Sıfır alanlı yaprağı iki renderer da çizmez.
        if (alan === 'renk' && (
          (!bDüğüm.yaprak && !gDüğüm.yaprak)
          || bDüğüm.genişlik <= 0 || bDüğüm.yükseklik <= 0
          || gDüğüm.genişlik <= 0 || gDüğüm.yükseklik <= 0
        )) continue;
        if (!eşitMi(alan, bDüğüm[alan], gDüğüm[alan])) {
          ekle(
            `seriler[${seriSırası}].düğümler[${düğümSırası}](${bDüğüm.ad}).${alan}`,
            bDüğüm[alan],
            gDüğüm[alan]
          );
        }
      }
    }
  }
  const beklenenDüğüm = beklenenSeriler.reduce((toplam, seri) => toplam + (seri.düğümler?.length || 0), 0);
  const gerçekDüğüm = gerçekSeriler.reduce((toplam, seri) => toplam + (seri.düğümler?.length || 0), 0);
  return [{
    ad: 'treemap_hücre_sınırları_ve_katmanları',
    geçti: uyuşmazlıklar.length === 0,
    açıklama: 'Toplam raster oranından bağımsız olarak her hücrenin x/y/width/height sınırı, dolgu, kenarlık, gap, upperLabel ve leafRoot katmanı kilitli ECharts sahnesiyle eşleşmeli',
    beklenen_düğüm: beklenenDüğüm,
    gerçek_düğüm: gerçekDüğüm,
    karşılaştırılan_alan: Math.min(beklenenDüğüm, gerçekDüğüm) * 15,
    uyuşmazlıklar
  }];
}

function sunburstSahneKontrolleri(senaryo, sahneDosyası, referansSahneDosyası) {
  if (!senaryo.karşılaştırma?.sahneReferansı) return [];
  const açıklama = 'Her Sunburst sektörünün veri sırası, açı/yarıçap sınırı, dolgu/kenarlık/köşe katmanı ve bağlı etiket geometrisi kilitli ECharts sahnesiyle eşleşmeli';
  if (!sahneDosyası || !fs.existsSync(sahneDosyası)) {
    return [{
      ad: 'sunburst_sektör_geometrisi_ve_katmanları', geçti: false, açıklama,
      hata: 'Cizelge sahne kanıtı eksik'
    }];
  }
  if (!referansSahneDosyası || !fs.existsSync(referansSahneDosyası)) {
    return [{
      ad: 'sunburst_sektör_geometrisi_ve_katmanları', geçti: false, açıklama,
      hata: 'kilitli ECharts sahne kanıtı eksik'
    }];
  }
  const gerçek = JSON.parse(fs.readFileSync(sahneDosyası, 'utf8'));
  const beklenen = JSON.parse(fs.readFileSync(referansSahneDosyası, 'utf8'));
  const uyuşmazlıklar = [];
  const ekle = (yol, beklenenDeğer, gerçekDeğer) => {
    if (uyuşmazlıklar.length < 60) {
      uyuşmazlıklar.push({ yol, beklenen: beklenenDeğer, gerçek: gerçekDeğer });
    }
  };
  for (const alan of ['şema_sürümü', 'tür', 'koordinat_adımı']) {
    if (gerçek[alan] !== beklenen[alan]) ekle(alan, beklenen[alan], gerçek[alan]);
  }
  const koordinatlar = new Set([
    'cx', 'cy', 'r0', 'r', 'başlangıç_açısı', 'bitiş_açısı',
    'kenarlık_kalınlığı', 'etiket_x', 'etiket_y', 'etiket_dönüşü',
    'köşe_yarıçapları'
  ]);
  const eşitMi = (alan, a, b) => {
    if (alan === 'köşe_yarıçapları' && Array.isArray(a) && Array.isArray(b)) {
      return a.length === b.length && a.every((değer, sıra) => Math.abs(değer - b[sıra]) <= 0.0011);
    }
    if (koordinatlar.has(alan) && typeof a === 'number' && typeof b === 'number') {
      return Math.abs(a - b) <= 0.0011;
    }
    return JSON.stringify(a) === JSON.stringify(b);
  };
  const bSeriler = beklenen.seriler || [];
  const gSeriler = gerçek.seriler || [];
  if (bSeriler.length !== gSeriler.length) ekle('seriler.length', bSeriler.length, gSeriler.length);
  for (let seriSırası = 0; seriSırası < Math.min(bSeriler.length, gSeriler.length); seriSırası += 1) {
    const bSeri = bSeriler[seriSırası];
    const gSeri = gSeriler[seriSırası];
    if (bSeri.seri_sırası !== gSeri.seri_sırası) {
      ekle(`seriler[${seriSırası}].seri_sırası`, bSeri.seri_sırası, gSeri.seri_sırası);
    }
    const bDilimler = bSeri.dilimler || [];
    const gDilimler = gSeri.dilimler || [];
    if (bDilimler.length !== gDilimler.length) {
      ekle(`seriler[${seriSırası}].dilimler.length`, bDilimler.length, gDilimler.length);
    }
    for (let sıra = 0; sıra < Math.min(bDilimler.length, gDilimler.length); sıra += 1) {
      const bDilim = bDilimler[sıra];
      const gDilim = gDilimler[sıra];
      for (const alan of [
        'veri_sırası', 'ad', 'derinlik', 'değer', 'cx', 'cy', 'r0', 'r',
        'başlangıç_açısı', 'bitiş_açısı', 'saat_yönünde', 'renk',
        'kenarlık_rengi', 'kenarlık_kalınlığı', 'köşe_yarıçapları',
        'etiket_göster', 'etiket_metni', 'etiket_x', 'etiket_y', 'etiket_dönüşü'
      ]) {
        // Görünmeyen etiketin metin/konum/dönüş alanları çizim çıktısının
        // parçası değildir. ECharts bu iç durumu korurken Çizelge sahnesi
        // kasıtlı olarak boşaltabilir; görünürlük bitini yine sıkı kıyasla.
        if (!bDilim.etiket_göster && !gDilim.etiket_göster
            && ['etiket_metni', 'etiket_x', 'etiket_y', 'etiket_dönüşü'].includes(alan)) {
          continue;
        }
        if (!eşitMi(alan, bDilim[alan], gDilim[alan])) {
          ekle(
            `seriler[${seriSırası}].dilimler[${sıra}](${bDilim.ad}).${alan}`,
            bDilim[alan], gDilim[alan]
          );
        }
      }
    }
  }
  const beklenenDilim = bSeriler.reduce((toplam, seri) => toplam + (seri.dilimler?.length || 0), 0);
  const gerçekDilim = gSeriler.reduce((toplam, seri) => toplam + (seri.dilimler?.length || 0), 0);
  return [{
    ad: 'sunburst_sektör_geometrisi_ve_katmanları',
    geçti: uyuşmazlıklar.length === 0,
    açıklama,
    beklenen_dilim: beklenenDilim,
    gerçek_dilim: gerçekDilim,
    karşılaştırılan_alan: Math.min(beklenenDilim, gerçekDilim) * 20,
    uyuşmazlıklar
  }];
}

function sankeySahneKontrolleri(senaryo, sahneDosyası, referansSahneDosyası) {
  if (!senaryo.karşılaştırma?.sahneReferansı) return [];
  const açıklama = 'Her Sankey düğüm kutusu ile bağlantı şeridinin veri kimliği, değer/derinlik, Bézier kontrol noktaları, kalınlığı, etkin RGBA dolgusu; ayrıca etiket tabanı, dönüşü ve fontu kilitli ECharts sahnesiyle eşleşmeli';
  if (!sahneDosyası || !fs.existsSync(sahneDosyası)) {
    return [{
      ad: 'sankey_düğüm_ve_bağ_geometrisi', geçti: false, açıklama,
      hata: 'Cizelge sahne kanıtı eksik'
    }];
  }
  if (!referansSahneDosyası || !fs.existsSync(referansSahneDosyası)) {
    return [{
      ad: 'sankey_düğüm_ve_bağ_geometrisi', geçti: false, açıklama,
      hata: 'kilitli ECharts sahne kanıtı eksik'
    }];
  }
  const gerçek = JSON.parse(fs.readFileSync(sahneDosyası, 'utf8'));
  const beklenen = JSON.parse(fs.readFileSync(referansSahneDosyası, 'utf8'));
  const uyuşmazlıklar = [];
  const ekle = (yol, beklenenDeğer, gerçekDeğer) => {
    if (uyuşmazlıklar.length < 80) {
      uyuşmazlıklar.push({ yol, beklenen: beklenenDeğer, gerçek: gerçekDeğer });
    }
  };
  const sayısalAlanlar = new Set([
    'x', 'y', 'genişlik', 'yükseklik', 'değer', 'kenarlık_kalınlığı',
    'x1', 'y1', 'x2', 'y2', 'cpx1', 'cpy1', 'cpx2', 'cpy2', 'kalınlık',
    'etiket_x', 'etiket_y', 'etiket_dönüşü'
  ]);
  const eşitMi = (alan, a, b) => {
    if (sayısalAlanlar.has(alan) && typeof a === 'number' && typeof b === 'number') {
      return Math.abs(a - b) <= 0.0011;
    }
    return JSON.stringify(a) === JSON.stringify(b);
  };
  for (const alan of ['şema_sürümü', 'tür', 'koordinat_adımı']) {
    if (!eşitMi(alan, beklenen[alan], gerçek[alan])) ekle(alan, beklenen[alan], gerçek[alan]);
  }
  const bSeriler = beklenen.seriler || [];
  const gSeriler = gerçek.seriler || [];
  if (bSeriler.length !== gSeriler.length) ekle('seriler.length', bSeriler.length, gSeriler.length);
  for (let seriSırası = 0; seriSırası < Math.min(bSeriler.length, gSeriler.length); seriSırası += 1) {
    const bSeri = bSeriler[seriSırası];
    const gSeri = gSeriler[seriSırası];
    if (bSeri.seri_sırası !== gSeri.seri_sırası) {
      ekle(`seriler[${seriSırası}].seri_sırası`, bSeri.seri_sırası, gSeri.seri_sırası);
    }
    for (const alan of ['x', 'y', 'genişlik', 'yükseklik']) {
      if (!eşitMi(alan, bSeri.alan?.[alan], gSeri.alan?.[alan])) {
        ekle(`seriler[${seriSırası}].alan.${alan}`, bSeri.alan?.[alan], gSeri.alan?.[alan]);
      }
    }
    const bDüğümler = bSeri.düğümler || [];
    const gDüğümler = gSeri.düğümler || [];
    if (bDüğümler.length !== gDüğümler.length) {
      ekle(`seriler[${seriSırası}].düğümler.length`, bDüğümler.length, gDüğümler.length);
    }
    for (let sıra = 0; sıra < Math.min(bDüğümler.length, gDüğümler.length); sıra += 1) {
      const bDüğüm = bDüğümler[sıra];
      const gDüğüm = gDüğümler[sıra];
      for (const alan of [
        'veri_sırası', 'ad', 'değer', 'derinlik', 'x', 'y', 'genişlik', 'yükseklik',
        'renk', 'kenarlık_rengi', 'kenarlık_kalınlığı', 'etiket_göster', 'etiket_metni',
        'etiket_x', 'etiket_y', 'etiket_dönüşü', 'etiket_fontu'
      ]) {
        if (!eşitMi(alan, bDüğüm[alan], gDüğüm[alan])) {
          ekle(`seriler[${seriSırası}].düğümler[${sıra}](${bDüğüm.ad}).${alan}`,
            bDüğüm[alan], gDüğüm[alan]);
        }
      }
    }
    const bBağlar = bSeri.bağlar || [];
    const gBağlar = gSeri.bağlar || [];
    if (bBağlar.length !== gBağlar.length) {
      ekle(`seriler[${seriSırası}].bağlar.length`, bBağlar.length, gBağlar.length);
    }
    for (let sıra = 0; sıra < Math.min(bBağlar.length, gBağlar.length); sıra += 1) {
      const bBağ = bBağlar[sıra];
      const gBağ = gBağlar[sıra];
      for (const alan of [
        'veri_sırası', 'kaynak', 'hedef', 'değer', 'yön',
        'x1', 'y1', 'x2', 'y2', 'cpx1', 'cpy1', 'cpx2', 'cpy2', 'kalınlık',
        'renkler', 'etiket_göster', 'etiket_metni'
      ]) {
        if (!eşitMi(alan, bBağ[alan], gBağ[alan])) {
          ekle(`seriler[${seriSırası}].bağlar[${sıra}](${bBağ.kaynak}->${bBağ.hedef}).${alan}`,
            bBağ[alan], gBağ[alan]);
        }
      }
    }
  }
  const bDüğüm = bSeriler.reduce((toplam, seri) => toplam + (seri.düğümler?.length || 0), 0);
  const gDüğüm = gSeriler.reduce((toplam, seri) => toplam + (seri.düğümler?.length || 0), 0);
  const bBağ = bSeriler.reduce((toplam, seri) => toplam + (seri.bağlar?.length || 0), 0);
  const gBağ = gSeriler.reduce((toplam, seri) => toplam + (seri.bağlar?.length || 0), 0);
  return [{
    ad: 'sankey_düğüm_ve_bağ_geometrisi',
    geçti: uyuşmazlıklar.length === 0,
    açıklama,
    beklenen_düğüm: bDüğüm,
    gerçek_düğüm: gDüğüm,
    beklenen_bağ: bBağ,
    gerçek_bağ: gBağ,
    karşılaştırılan_alan: Math.min(bDüğüm, gDüğüm) * 17 + Math.min(bBağ, gBağ) * 17,
    uyuşmazlıklar
  }];
}

function chordSahneKontrolleri(senaryo, sahneDosyası, referansSahneDosyası) {
  if (!senaryo.karşılaştırma?.sahneReferansı) return [];
  const açıklama = 'Her Chord sektörünün açı/yarıçap/köşe/etiket geometrisi ile her şeridin kaynak-hedef açı istifi, dört uç noktası, çizgi katmanı ve etkin RGBA renk uçları kilitli ECharts sahnesiyle eşleşmeli';
  if (!sahneDosyası || !fs.existsSync(sahneDosyası)) {
    return [{ ad: 'chord_sektör_ve_şerit_geometrisi', geçti: false, açıklama,
      hata: 'Cizelge sahne kanıtı eksik' }];
  }
  if (!referansSahneDosyası || !fs.existsSync(referansSahneDosyası)) {
    return [{ ad: 'chord_sektör_ve_şerit_geometrisi', geçti: false, açıklama,
      hata: 'kilitli ECharts sahne kanıtı eksik' }];
  }
  const gerçek = JSON.parse(fs.readFileSync(sahneDosyası, 'utf8'));
  const beklenen = JSON.parse(fs.readFileSync(referansSahneDosyası, 'utf8'));
  const uyuşmazlıklar = [];
  const ekle = (yol, beklenenDeğer, gerçekDeğer) => {
    if (uyuşmazlıklar.length < 100) {
      uyuşmazlıklar.push({ yol, beklenen: beklenenDeğer, gerçek: gerçekDeğer });
    }
  };
  const eşitMi = (a, b) => {
    if (typeof a === 'number' && typeof b === 'number') return Math.abs(a - b) <= 0.0011;
    if (Array.isArray(a) && Array.isArray(b)) {
      return a.length === b.length && a.every((değer, sıra) => eşitMi(değer, b[sıra]));
    }
    return JSON.stringify(a) === JSON.stringify(b);
  };
  for (const alan of ['şema_sürümü', 'tür', 'koordinat_adımı']) {
    if (!eşitMi(beklenen[alan], gerçek[alan])) ekle(alan, beklenen[alan], gerçek[alan]);
  }
  const bSeriler = beklenen.seriler || [];
  const gSeriler = gerçek.seriler || [];
  if (bSeriler.length !== gSeriler.length) ekle('seriler.length', bSeriler.length, gSeriler.length);
  const düğümAlanları = [
    'veri_sırası', 'kimlik', 'ad', 'değer', 'cx', 'cy', 'r0', 'r',
    'başlangıç_açısı', 'bitiş_açısı', 'saat_yönünde', 'renk',
    'kenarlık_rengi', 'kenarlık_kalınlığı', 'köşe_yarıçapları',
    'etiket_göster', 'etiket_metni', 'etiket_x', 'etiket_y', 'etiket_dönüşü',
    'etiket_yatay_hizası', 'etiket_dikey_hizası', 'etiket_fontu', 'etiket_rengi'
  ];
  const bağAlanları = [
    'veri_sırası', 'kaynak', 'hedef', 'değer',
    'kaynak_başlangıç_açısı', 'kaynak_bitiş_açısı',
    'hedef_başlangıç_açısı', 'hedef_bitiş_açısı',
    'kaynak1', 'kaynak2', 'hedef1', 'hedef2', 'cx', 'cy', 'r',
    'saat_yönünde', 'renkler', 'kenarlık_kalınlığı',
    'etiket_göster', 'etiket_metni'
  ];
  for (let seriSırası = 0; seriSırası < Math.min(bSeriler.length, gSeriler.length); seriSırası += 1) {
    const bSeri = bSeriler[seriSırası];
    const gSeri = gSeriler[seriSırası];
    if (bSeri.seri_sırası !== gSeri.seri_sırası) {
      ekle(`seriler[${seriSırası}].seri_sırası`, bSeri.seri_sırası, gSeri.seri_sırası);
    }
    for (const alan of ['x', 'y', 'genişlik', 'yükseklik']) {
      if (!eşitMi(bSeri.alan?.[alan], gSeri.alan?.[alan])) {
        ekle(`seriler[${seriSırası}].alan.${alan}`, bSeri.alan?.[alan], gSeri.alan?.[alan]);
      }
    }
    const bDüğümler = bSeri.düğümler || [];
    const gDüğümler = gSeri.düğümler || [];
    if (bDüğümler.length !== gDüğümler.length) {
      ekle(`seriler[${seriSırası}].düğümler.length`, bDüğümler.length, gDüğümler.length);
    }
    for (let sıra = 0; sıra < Math.min(bDüğümler.length, gDüğümler.length); sıra += 1) {
      for (const alan of düğümAlanları) {
        if (!eşitMi(bDüğümler[sıra][alan], gDüğümler[sıra][alan])) {
          ekle(`seriler[${seriSırası}].düğümler[${sıra}](${bDüğümler[sıra].ad}).${alan}`,
            bDüğümler[sıra][alan], gDüğümler[sıra][alan]);
        }
      }
    }
    const bBağlar = bSeri.bağlar || [];
    const gBağlar = gSeri.bağlar || [];
    if (bBağlar.length !== gBağlar.length) {
      ekle(`seriler[${seriSırası}].bağlar.length`, bBağlar.length, gBağlar.length);
    }
    for (let sıra = 0; sıra < Math.min(bBağlar.length, gBağlar.length); sıra += 1) {
      for (const alan of bağAlanları) {
        if (!eşitMi(bBağlar[sıra][alan], gBağlar[sıra][alan])) {
          ekle(`seriler[${seriSırası}].bağlar[${sıra}](${bBağlar[sıra].kaynak}->${bBağlar[sıra].hedef}).${alan}`,
            bBağlar[sıra][alan], gBağlar[sıra][alan]);
        }
      }
    }
  }
  const bDüğüm = bSeriler.reduce((toplam, seri) => toplam + (seri.düğümler?.length || 0), 0);
  const gDüğüm = gSeriler.reduce((toplam, seri) => toplam + (seri.düğümler?.length || 0), 0);
  const bBağ = bSeriler.reduce((toplam, seri) => toplam + (seri.bağlar?.length || 0), 0);
  const gBağ = gSeriler.reduce((toplam, seri) => toplam + (seri.bağlar?.length || 0), 0);
  return [{
    ad: 'chord_sektör_ve_şerit_geometrisi',
    geçti: uyuşmazlıklar.length === 0,
    açıklama,
    beklenen_düğüm: bDüğüm,
    gerçek_düğüm: gDüğüm,
    beklenen_bağ: bBağ,
    gerçek_bağ: gBağ,
    karşılaştırılan_alan: Math.min(bDüğüm, gDüğüm) * düğümAlanları.length
      + Math.min(bBağ, gBağ) * bağAlanları.length,
    uyuşmazlıklar
  }];
}

function sahneÖzetiKontrolleri(senaryo, sahneDosyası, referansSahneDosyası) {
  if (senaryo.karşılaştırma?.sahneReferansı) {
    if (senaryo.id.startsWith('sunburst-')) {
      return sunburstSahneKontrolleri(senaryo, sahneDosyası, referansSahneDosyası);
    }
    if (senaryo.id.startsWith('sankey-')) {
      return sankeySahneKontrolleri(senaryo, sahneDosyası, referansSahneDosyası);
    }
    if (senaryo.id.startsWith('chord-')) {
      return chordSahneKontrolleri(senaryo, sahneDosyası, referansSahneDosyası);
    }
    return treemapSahneKontrolleri(senaryo, sahneDosyası, referansSahneDosyası);
  }
  const beklenen = senaryo.karşılaştırma?.sahneÖzeti;
  if (!beklenen) return [];
  if (!sahneDosyası || !fs.existsSync(sahneDosyası)) {
    return [{
      ad: `${senaryo.id.startsWith('tree-') ? 'tree' : 'parallel'}_sahne_özeti`,
      geçti: false,
      açıklama: 'Renderer bağımsız veri/geometri/renk sahne özeti üretilmelidir',
      beklenen,
      hata: 'sahne özeti dosyası eksik'
    }];
  }
  const gerçek = JSON.parse(fs.readFileSync(sahneDosyası, 'utf8'));
  const alanlar = Object.keys(beklenen).map((alan) => ({
    alan,
    beklenen: beklenen[alan],
    gerçek: gerçek[alan],
    geçti: gerçek[alan] === beklenen[alan]
  }));
  const tree = senaryo.id.startsWith('tree-');
  return [{
    ad: `${tree ? 'tree' : 'parallel'}_sahne_özeti`,
    geçti: alanlar.every((alan) => alan.geçti),
    açıklama: tree
      ? 'Görünür Tree düğümleri, kenar yolları, etiket çapaları ve daraltma durumu kesin eşleşmeli'
      : '7.637 Polyline; 229.110 koordinat, RGB, width, opacity ve smooth ile eşleşmeli',
    alanlar
  }];
}

function htmlKaçır(değer) {
  return String(değer).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
}

function göreli(dosya) {
  return path.relative(RAPOR, dosya).split(path.sep).join('/');
}

async function çalıştır() {
  for (const d of [REFERANS, REFERANS_SAHNE, GERÇEK, FARK, METRİK, SAHNE, RAPOR]) dizin(d);
  const sonuçlar = [];
  const idSırası = process.argv.indexOf('--id');
  const önekSırası = process.argv.indexOf('--id-prefix');
  const seçilenId = idSırası >= 0 ? process.argv[idSırası + 1] : null;
  const seçilenÖnek = önekSırası >= 0 ? process.argv[önekSırası + 1] : null;
  if (seçilenId && seçilenÖnek) {
    throw new Error('`--id` ve `--id-prefix` aynı anda kullanılamaz');
  }
  if ((idSırası >= 0 && !seçilenId) || (önekSırası >= 0 && !seçilenÖnek)) {
    throw new Error('kanıt seçme seçeneğinin değeri eksik');
  }
  const senaryolar = seçilenId
    ? SENARYOLAR.filter((senaryo) => senaryo.id === seçilenId)
    : seçilenÖnek
      ? SENARYOLAR.filter((senaryo) => senaryo.id.startsWith(seçilenÖnek))
      : SENARYOLAR;
  if (seçilenId && senaryolar.length === 0) {
    throw new Error(`bilinmeyen kanıt senaryosu: ${seçilenId}`);
  }
  if (seçilenÖnek && senaryolar.length === 0) {
    throw new Error(`kanıt senaryosu öneki eşleşmedi: ${seçilenÖnek}`);
  }
  for (const senaryo of senaryolar) {
    for (const kare of senaryo.kareler) {
      const sonek = senaryo.kareler.length === 1 ? '' : `-${kare.ad}`;
      const dosyaId = dosyaKimliği(senaryo.id);
      const referans = path.join(REFERANS, `${dosyaId}${sonek}.png`);
      const gerçek = path.join(GERÇEK, `${dosyaId}${sonek}.png`);
      const ham = path.join(GERÇEK, `.ham-${dosyaId}${sonek}.png`);
      const fark = path.join(FARK, `${dosyaId}${sonek}.png`);
      const normalizeFark = path.join(FARK, `${dosyaId}${sonek}-tipografi.png`);
      const sahne = senaryo.karşılaştırma?.sahneÖzeti
        || senaryo.karşılaştırma?.sahneReferansı
        ? path.join(SAHNE, `${dosyaId}${sonek}.json`)
        : null;
      const referansSahne = senaryo.karşılaştırma?.sahneReferansı
        ? path.join(REFERANS_SAHNE, `${dosyaId}${sonek}.json`)
        : null;
      if (REFERANS_YENİLE) {
        await referansıYenile(senaryo, kare, referans, referansSahne, sonek);
      } else if (!fs.existsSync(referans)) {
        throw new Error(
          `kilitli referans eksik: ${path.relative(KÖK, referans)}; `
          + '`--referans-yenile` yalnız açık snapshot yenilemesinde kullanılmalıdır'
        );
      }
      const fixtureArgümanları = [
        'run', '--quiet', '--no-default-features', '--features', 'png',
        '--example', 'uyum_fixture', '--',
        '--id', senaryo.id,
        '--output', ham,
        '--frame', String(kare.kare),
        '--state', kare.durum,
        '--width', String(senaryo.genişlik ?? 700),
        '--height', String(senaryo.yükseklik ?? 525)
      ];
      if (sahne) fixtureArgümanları.push('--scene-output', sahne);
      execFileSync('cargo', fixtureArgümanları, { cwd: KÖK, stdio: 'inherit' });
      await sharp(ham).resize(600, 450).toFile(gerçek);
      fs.rmSync(ham, { force: true });
      const metrik = await karşılaştır(
        referans,
        gerçek,
        fark,
        normalizeFark,
        senaryo.karşılaştırma
      );
      const yapısal_kontroller = [
        ...yapısalKontroller(senaryo, referans, gerçek),
        ...sahneÖzetiKontrolleri(senaryo, sahne, referansSahne)
      ];
      metrik.geçti = metrik.geçti
        && yapısal_kontroller.every((kontrol) => kontrol.geçti);
      sonuçlar.push({
        id: senaryo.id,
        senaryo: senaryo.tür,
        kare: kare.ad,
        eşikler: EŞİK,
        ...metrik,
        ...(yapısal_kontroller.length > 0 ? { yapısal_kontroller } : {}),
        dosyalar: {
          referans: path.relative(KÖK, referans),
          gerçek: path.relative(KÖK, gerçek),
          fark: path.relative(KÖK, fark),
          ...(metrik.tipografi_normalizasyonu
            ? { normalize_fark: path.relative(KÖK, normalizeFark) }
            : {}),
          ...(sahne ? { sahne: path.relative(KÖK, sahne) } : {}),
          ...(referansSahne
            ? { referans_sahne: path.relative(KÖK, referansSahne) }
            : {})
        }
      });
    }
  }
  for (const sonuç of sonuçlar) {
    fs.writeFileSync(
      path.join(METRİK, `${dosyaKimliği(sonuç.id)}-${sonuç.kare}.json`),
      `${JSON.stringify(sonuç, null, 2)}\n`
    );
  }
  const satırlar = sonuçlar.map((sonuç) => {
    const yapısalKontroller = sonuç.yapısal_kontroller ?? [];
    const yapısal = yapısalKontroller.length
      ? ` · yapısal ${yapısalKontroller.filter((kontrol) => kontrol.geçti).length}/${yapısalKontroller.length}`
      : '';
    const ham = sonuç.ham
      ? ` · ham fark %${(sonuç.ham.değişen_piksel_oranı * 100).toFixed(3)} · ham SSIM ${sonuç.ham.ssim.toFixed(5)}`
      : '';
    const profil = sonuç.tipografi_normalizasyonu
      ? ` · aynı Gauss çekirdeği σ=${sonuç.tipografi_normalizasyonu.gaussian_sigma}`
      : '';
    const normalizeFark = sonuç.dosyalar.normalize_fark
      ? `<figure><img src="${göreli(path.join(KÖK, sonuç.dosyalar.normalize_fark))}"><figcaption>Normalize fark</figcaption></figure>`
      : '';
    return `<article class="${sonuç.geçti ? 'geçti' : 'kaldı'}"><h2>${htmlKaçır(sonuç.id)} · ${htmlKaçır(sonuç.kare)}</h2><p>${sonuç.geçti ? 'GEÇTİ' : 'KALDI'} · kapı farkı %${(sonuç.değişen_piksel_oranı * 100).toFixed(3)} · kapı SSIM ${sonuç.ssim.toFixed(5)}${ham}${profil}${yapısal}</p><div><figure><img src="${göreli(path.join(KÖK, sonuç.dosyalar.referans))}"><figcaption>ECharts</figcaption></figure><figure><img src="${göreli(path.join(KÖK, sonuç.dosyalar.gerçek))}"><figcaption>Cizelge</figcaption></figure><figure><img src="${göreli(path.join(KÖK, sonuç.dosyalar.fark))}"><figcaption>Ham fark</figcaption></figure>${normalizeFark}</div></article>`;
  }).join('\n');
  fs.writeFileSync(path.join(RAPOR, 'index.html'), `<!doctype html><html lang="tr"><head><meta charset="utf-8"><title>Uyum görsel kanıtı</title><style>body{font:14px system-ui;margin:24px;background:#f5f7fa;color:#1f2937}article{background:white;border:1px solid #ddd;border-left:6px solid #dc2626;border-radius:8px;margin:18px 0;padding:16px}.geçti{border-left-color:#16a34a}article>div{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px}figure{margin:0}img{width:100%;border:1px solid #ddd}figcaption{text-align:center;padding:5px}</style></head><body><h1>Cizelge görsel kanıt raporu</h1><p>pixelmatch 0.1 · değişen piksel ≤ %1 · SSIM ≥ 0.99. Yoğun tipografi profili varsa değişmeyen eşikler iki görüntüye de aynı Gauss çekirdeği uygulandıktan sonra değerlendirilir; ham metrik ve ham fark daima gösterilir.</p>${satırlar}</body></html>`);
  const geçen = sonuçlar.filter((sonuç) => sonuç.geçti).length;
  process.stdout.write(`${geçen}/${sonuçlar.length} kare eşikleri geçti. Rapor: ${path.relative(KÖK, path.join(RAPOR, 'index.html'))}\n`);
  if (process.argv.includes('--enforce') && geçen !== sonuçlar.length) process.exitCode = 1;
}

await çalıştır();
