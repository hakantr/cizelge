#!/usr/bin/env node

// Kilitli echarts-examples Chord seçeneklerini Rust fixture'larının
// okuyabileceği belirlenimci JSON belgelerine dönüştürür. İşlevle üretilen
// çoklu seri/başlıklar TypeScript kaynağının kendisi yürütülerek açılır.

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import vm from 'node:vm';
import ts from 'typescript';

const ARAÇ = path.dirname(new URL(import.meta.url).pathname);
const KÖK = path.resolve(ARAÇ, '../..');
const ÖRNEKLER = path.resolve(KÖK, '../echarts-examples/public/examples/ts');
const ÇIKTI = path.resolve(KÖK, 'examples/uyum_veri/chord');
const KİMLİKLER = [
  'chord-simple',
  'chord-minAngle',
  'chord-lineStyle-color',
  'chord-style'
];

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
  const bağlam = { console, exports: {} };
  vm.createContext(bağlam);
  new vm.Script(`${javascript}\nglobalThis.__uyum_option = option;`, { filename: dosya })
    .runInContext(bağlam, { timeout: 5_000 });
  return { dosya, seçenek: bağlam.__uyum_option };
}

fs.mkdirSync(ÇIKTI, { recursive: true });
for (const id of KİMLİKLER) {
  const { dosya, seçenek } = örnekSeçeneği(id);
  const seriler = Array.isArray(seçenek.series) ? seçenek.series : [seçenek.series];
  if (!seriler.length || seriler.some((seri) => seri?.type !== 'chord'
      || !Array.isArray(seri.data) || !Array.isArray(seri.links))) {
    throw new Error(`${id}: Chord data/links bulunamadı`);
  }
  const belge = {
    şema_sürümü: 1,
    kaynak: path.relative(KÖK, dosya).split(path.sep).join('/'),
    seçenek: {
      backgroundColor: seçenek.backgroundColor,
      title: seçenek.title,
      tooltip: seçenek.tooltip,
      legend: seçenek.legend,
      animation: seçenek.animation,
      series: seriler
    }
  };
  fs.writeFileSync(path.join(ÇIKTI, `${id}.json`), `${JSON.stringify(belge, null, 2)}\n`);
}

process.stdout.write(`${KİMLİKLER.length} Chord seçenek/veri fixture'ı üretildi.\n`);
