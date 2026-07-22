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
const GERÇEK = path.join(TABAN, 'gerçek', 'default');
const FARK = path.join(TABAN, 'fark', 'default');
const METRİK = path.join(TABAN, 'metrikler');
const RAPOR = path.join(TABAN, 'rapor');
const EŞİK = Object.freeze({ pixelmatch: 0.1, değişenPikselOranı: 0.01, ssim: 0.99 });
const REFERANS_YENİLE = process.argv.includes('--referans-yenile');

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

async function referansıYenile(senaryo, kare, referans, sonek) {
  const adaylar = [];
  const dosyaId = dosyaKimliği(senaryo.id);
  for (const geçiş of [1, 2]) {
    const ham = path.join(REFERANS, `.ham-${dosyaId}${sonek}-${geçiş}.png`);
    const aday = path.join(REFERANS, `.aday-${dosyaId}${sonek}-${geçiş}.png`);
    execFileSync('node', [
      path.join(ARAÇ, 'echarts_referans.mjs'),
      '--id', senaryo.id,
      '--output', ham,
      '--frame', String(kare.kare),
      '--state', kare.durum,
      '--width', String(senaryo.genişlik ?? 700),
      '--height', String(senaryo.yükseklik ?? 525)
    ], { cwd: KÖK, stdio: 'inherit' });
    await sharp(ham).resize(600, 450).toFile(aday);
    fs.rmSync(ham, { force: true });
    adaylar.push(aday);
  }
  if (!aynıPiksellerMi(adaylar[0], adaylar[1])) {
    for (const aday of adaylar) fs.rmSync(aday, { force: true });
    throw new Error(`${senaryo.id}${sonek}: ECharts referansı iki ardışık üretimde kararlı değil`);
  }
  fs.renameSync(adaylar[0], referans);
  fs.rmSync(adaylar[1], { force: true });
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

function htmlKaçır(değer) {
  return String(değer).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
}

function göreli(dosya) {
  return path.relative(RAPOR, dosya).split(path.sep).join('/');
}

async function çalıştır() {
  for (const d of [REFERANS, GERÇEK, FARK, METRİK, RAPOR]) dizin(d);
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
      if (REFERANS_YENİLE) {
        await referansıYenile(senaryo, kare, referans, sonek);
      } else if (!fs.existsSync(referans)) {
        throw new Error(
          `kilitli referans eksik: ${path.relative(KÖK, referans)}; `
          + '`--referans-yenile` yalnız açık snapshot yenilemesinde kullanılmalıdır'
        );
      }
      execFileSync('cargo', [
        'run', '--quiet', '--no-default-features', '--features', 'png',
        '--example', 'uyum_fixture', '--',
        '--id', senaryo.id,
        '--output', ham,
        '--frame', String(kare.kare),
        '--state', kare.durum,
        '--width', String(senaryo.genişlik ?? 700),
        '--height', String(senaryo.yükseklik ?? 525)
      ], { cwd: KÖK, stdio: 'inherit' });
      await sharp(ham).resize(600, 450).toFile(gerçek);
      fs.rmSync(ham, { force: true });
      const metrik = await karşılaştır(
        referans,
        gerçek,
        fark,
        normalizeFark,
        senaryo.karşılaştırma
      );
      const yapısal_kontroller = yapısalKontroller(senaryo, referans, gerçek);
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
