#!/usr/bin/env node

// Kilitli echarts-examples Sankey seçeneklerini ve veri grafiklerini Rust
// fixture'larının okuyabileceği belirlenimci JSON belgelerine dönüştürür.
// Uzaktan veri isteyen resmî örneklerde $.get çağrısı aynı repo içindeki
// energy.json / product.json varlıklarına yönlendirilir; ağ kullanılmaz.

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import vm from 'node:vm';
import ts from 'typescript';

const ARAÇ = path.dirname(new URL(import.meta.url).pathname);
const KÖK = path.resolve(ARAÇ, '../..');
const ÖRNEKLER = path.resolve(KÖK, '../echarts-examples/public/examples/ts');
const VERİ = path.resolve(KÖK, '../echarts-examples/public/data/asset/data');
const ÇIKTI = path.resolve(KÖK, 'examples/uyum_veri/sankey');
const KİMLİKLER = [
  'sankey-simple',
  'sankey-vertical',
  'sankey-itemstyle',
  'sankey-levels',
  'sankey-energy',
  'sankey-nodeAlign-left',
  'sankey-nodeAlign-right'
];

function veriOku(url) {
  const ad = path.basename(String(url));
  if (!['energy.json', 'product.json'].includes(ad)) {
    throw new Error(`beklenmeyen resmî Sankey veri isteği: ${url}`);
  }
  return JSON.parse(fs.readFileSync(path.join(VERİ, ad), 'utf8'));
}

function örnekSeçeneği(id) {
  const dosya = path.join(ÖRNEKLER, `${id}.ts`);
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
  const bağlam = {
    console,
    exports: {},
    ROOT_PATH: '',
    $: { get(url, işlev) { işlev(veriOku(url)); } },
    myChart: {
      showLoading() {},
      hideLoading() {},
      setOption(seçenek) { bağlam.option = seçenek; }
    }
  };
  vm.createContext(bağlam);
  new vm.Script(`${javascript}\nglobalThis.__uyum_option = option;`, { filename: dosya })
    .runInContext(bağlam, { timeout: 5_000 });
  return { dosya, seçenek: bağlam.__uyum_option };
}

fs.mkdirSync(ÇIKTI, { recursive: true });
for (const id of KİMLİKLER) {
  const { dosya, seçenek } = örnekSeçeneği(id);
  const seri = Array.isArray(seçenek.series) ? seçenek.series[0] : seçenek.series;
  if (!seri || seri.type !== 'sankey' || !Array.isArray(seri.data) || !Array.isArray(seri.links)) {
    throw new Error(`${id}: Sankey data/links bulunamadı`);
  }
  const belge = {
    şema_sürümü: 1,
    kaynak: path.relative(KÖK, dosya).split(path.sep).join('/'),
    seçenek: {
      backgroundColor: seçenek.backgroundColor,
      title: seçenek.title,
      tooltip: seçenek.tooltip,
      animation: seçenek.animation,
      series: seri
    }
  };
  fs.writeFileSync(path.join(ÇIKTI, `${id}.json`), `${JSON.stringify(belge, null, 2)}\n`);
}

process.stdout.write(`${KİMLİKLER.length} Sankey seçenek/veri fixture'ı üretildi.\n`);
