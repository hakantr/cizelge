#!/usr/bin/env node

// Kilitli echarts-examples Sunburst veri ağaçlarını, Rust fixture'larının
// doğrudan okuyabileceği belirlenimci JSON dosyalarına dönüştürür. Kitap
// örneğindeki çalışma-zamanı veri zenginleştirmesi de resmî betik çalıştırılarak
// yakalanır; böylece büyük veri ağaçları elle kopyalanmaz.

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import vm from 'node:vm';
import ts from 'typescript';

const ARAÇ = path.dirname(new URL(import.meta.url).pathname);
const KÖK = path.resolve(ARAÇ, '../..');
const ÖRNEKLER = path.resolve(KÖK, '../echarts-examples/public/examples/ts');
const ÇIKTI = path.resolve(KÖK, 'examples/uyum_veri/sunburst');
const KİMLİKLER = [
  'sunburst-simple',
  'sunburst-borderRadius',
  'sunburst-label-rotate',
  'sunburst-monochrome',
  'sunburst-visualMap',
  'sunburst-drink',
  'sunburst-book'
];

function örnekSeçeneği(id) {
  const adaylar = [path.join(ÖRNEKLER, `${id}.ts`), path.join(ÖRNEKLER, `${id}.js`)];
  const dosya = adaylar.find((aday) => fs.existsSync(aday));
  if (!dosya) throw new Error(`resmî örnek bulunamadı: ${id}`);
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
  const seri = Array.isArray(seçenek.series) ? seçenek.series[0] : seçenek.series;
  if (!seri || seri.type !== 'sunburst' || !Array.isArray(seri.data)) {
    throw new Error(`${id}: Sunburst data bulunamadı`);
  }
  const belge = {
    şema_sürümü: 1,
    kaynak: path.relative(KÖK, dosya).split(path.sep).join('/'),
    veri: seri.data
  };
  fs.writeFileSync(
    path.join(ÇIKTI, `${id}.json`),
    `${JSON.stringify(belge, null, 2)}\n`
  );
}

process.stdout.write(`${KİMLİKLER.length} Sunburst veri fixture'ı üretildi.\n`);
