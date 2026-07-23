#!/usr/bin/env node

// Kilitli echarts-examples Graph seçeneklerini ve yerel veri varlıklarını
// Rust fixture'larının okuyabileceği belirlenimci JSON belgelerine açar.
// ECharts çekirdeği çalıştırılmaz: kaynak TypeScript doğrudan yürütülür,
// jQuery veri çağrıları ../echarts-examples içindeki aynı varlıklara gider.

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import vm from 'node:vm';
import ts from 'typescript';

const ARAÇ = path.dirname(new URL(import.meta.url).pathname);
const KÖK = path.resolve(ARAÇ, '../..');
const ÖRNEKLER = path.resolve(KÖK, '../echarts-examples/public/examples/ts');
const VERİ = path.resolve(KÖK, '../echarts-examples/public/data/asset/data');
const ÇIKTI = path.resolve(KÖK, 'examples/uyum_veri/graph');
const KİMLİKLER = [
  'graph-force2',
  'graph-grid',
  'graph-simple',
  'graph-force',
  'graph-label-overlap',
  'graph',
  'graph-circular-layout',
  'graph-force-dynamic',
  'graph-life-expectancy',
  'graph-webkit-dep',
  'graph-npm'
];
const İZİNLİ_VERİ = new Set([
  'les-miserables.json',
  'life-expectancy.json',
  'webkit-dep.json',
  'npmdepgraph.min10.json'
]);

function veriOku(url) {
  const ad = path.basename(String(url));
  if (!İZİNLİ_VERİ.has(ad)) {
    throw new Error(`beklenmeyen resmî Graph veri isteği: ${url}`);
  }
  return JSON.parse(fs.readFileSync(path.join(VERİ, ad), 'utf8'));
}

function javascriptOku(id) {
  const adaylar = [path.join(ÖRNEKLER, `${id}.ts`), path.join(ÖRNEKLER, `${id}.js`)];
  const dosya = adaylar.find((aday) => fs.existsSync(aday));
  if (!dosya) throw new Error(`${id}: örnek kaynağı bulunamadı`);
  const kaynak = fs.readFileSync(dosya, 'utf8').replace(/\bexport\s*\{\s*\}\s*;?/g, '');
  const javascript = ts.transpileModule(kaynak, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2020,
      module: ts.ModuleKind.None,
      removeComments: true
    }
  }).outputText
    .replace(/Object\.defineProperty\(exports,\s*["']__esModule["'],\s*\{\s*value:\s*true\s*\}\);?\s*/g, '')
    .replace(/\bexport\s*\{\s*\}\s*;?/g, '');
  return {dosya, javascript};
}

function örnekSeçeneği(id) {
  const {dosya, javascript} = javascriptOku(id);
  const zamanlayıcılar = [];
  const bağlam = {
    console,
    exports: {},
    ROOT_PATH: '',
    $: {
      get(url, işlev) { işlev(veriOku(url)); },
      getJSON(url, işlev) { işlev(veriOku(url)); }
    },
    setInterval(işlev, ms) {
      zamanlayıcılar.push({işlev, ms});
      return zamanlayıcılar.length;
    },
    clearInterval() {},
    myChart: {
      showLoading() {},
      hideLoading() {},
      getWidth() { return 700; },
      getHeight() { return 525; },
      setOption(seçenek) {
        if (!bağlam.option || seçenek?.series?.some((seri) => seri?.type === 'graph')) {
          bağlam.option = seçenek;
        }
      }
    }
  };
  vm.createContext(bağlam);
  const önek = `
    let __uyum_seed = 0x5eed1234;
    Math.random = () => {
      __uyum_seed |= 0; __uyum_seed = __uyum_seed + 0x6D2B79F5 | 0;
      let t = Math.imul(__uyum_seed ^ __uyum_seed >>> 15, 1 | __uyum_seed);
      t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t;
      return ((t ^ t >>> 14) >>> 0) / 4294967296;
    };
  `;
  new vm.Script(`${önek}\n${javascript}\nglobalThis.__uyum_option = option;`, {filename: dosya})
    .runInContext(bağlam, {timeout: 10_000});
  if (id === 'graph-force-dynamic') {
    const zamanlayıcı = zamanlayıcılar[0];
    if (!zamanlayıcı || zamanlayıcı.ms !== 200) {
      throw new Error(`${id}: 200 ms güncelleme zamanlayıcısı bulunamadı`);
    }
    for (let tik = 0; tik < 25; tik += 1) zamanlayıcı.işlev();
  }
  return {dosya, seçenek: bağlam.__uyum_option};
}

fs.mkdirSync(ÇIKTI, {recursive: true});
for (const id of KİMLİKLER) {
  const {dosya, seçenek} = örnekSeçeneği(id);
  const seriler = (Array.isArray(seçenek?.series) ? seçenek.series : [seçenek?.series])
    .filter(Boolean);
  if (!seriler.length || seriler.some((seri) => seri.type !== 'graph'
      || !Array.isArray(seri.data) || !(Array.isArray(seri.links) || Array.isArray(seri.edges)))) {
    throw new Error(`${id}: Graph data ve links/edges bulunamadı`);
  }
  const belge = {
    şema_sürümü: 1,
    kaynak: path.relative(KÖK, dosya).split(path.sep).join('/'),
    dinamik_güncelleme_sayısı: id === 'graph-force-dynamic' ? 25 : 0,
    seçenek: {
      backgroundColor: seçenek.backgroundColor,
      title: seçenek.title,
      tooltip: seçenek.tooltip,
      legend: seçenek.legend,
      grid: seçenek.grid,
      xAxis: seçenek.xAxis,
      yAxis: seçenek.yAxis,
      visualMap: seçenek.visualMap,
      toolbox: seçenek.toolbox,
      dataZoom: seçenek.dataZoom,
      thumbnail: seçenek.thumbnail,
      animation: seçenek.animation,
      series: seriler
    }
  };
  fs.writeFileSync(path.join(ÇIKTI, `${id}.json`), `${JSON.stringify(belge, null, 2)}\n`);
}

process.stdout.write(`${KİMLİKLER.length} Graph seçenek/veri fixture'ı üretildi.\n`);
