#!/usr/bin/env node

// Resmî Circle Packing örneğinin d3-hierarchy@2.0.0 yerleşimini kilitli
// 700x525 kanıt tuvali için üretir. Rust fixture'ı bu çıktıyı okuyarak aynı
// üçüncü taraf algoritmanın sonucunu çalışma zamanında Node'a ihtiyaç
// duymadan çizer.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { pack, stratify } from 'd3-hierarchy';

const ARAÇ = path.dirname(fileURLToPath(import.meta.url));
const KÖK = path.resolve(ARAÇ, '../..');
const kaynak = path.resolve(
  KÖK,
  '../echarts-examples/public/data/asset/data/option-view.json'
);
const hedef = path.resolve(KÖK, 'examples/uyum_veri/circle-packing-layout.json');
const ham = JSON.parse(fs.readFileSync(kaynak, 'utf8'));
const seri = [];
let enDerin = 0;

function dönüştür(düğüm, tabanYolu, derinlik) {
  if (düğüm == null || enDerin > 5) return;
  enDerin = Math.max(derinlik, enDerin);
  seri.push({
    id: tabanYolu,
    value: düğüm.$count,
    depth: derinlik,
    index: seri.length
  });
  for (const anahtar in düğüm) {
    if (Object.prototype.hasOwnProperty.call(düğüm, anahtar) && !/^\$/.test(anahtar)) {
      dönüştür(düğüm[anahtar], `${tabanYolu}.${anahtar}`, derinlik + 1);
    }
  }
}

dönüştür(ham, 'option', 0);
const kök = stratify()
  .parentId((düğüm) => düğüm.id.substring(0, düğüm.id.lastIndexOf('.')))(seri)
  .sum((düğüm) => düğüm.value || 0)
  .sort((a, b) => b.value - a.value);
pack().size([698, 523]).padding(3)(kök);

function yaprakAdı(yol) {
  return yol
    .slice(yol.lastIndexOf('.') + 1)
    .split(/(?=[A-Z][^A-Z])/g)
    .join('\n');
}

const yerleşim = new Map(kök.descendants().map((düğüm) => [düğüm.id, düğüm]));
const çıktı = {
  source: '../echarts-examples/public/data/asset/data/option-view.json',
  algorithm: 'd3-hierarchy@2.0.0 pack size=[698,523] padding=3',
  viewport: [700, 525],
  maxDepth: enDerin,
  // renderItem çağrı sırası dataset/source sırasıdır; d3.descendants()
  // yerleşim dolaşım sırasını buraya sızdırmamak z2 eşitliğindeki kararlı
  // boya sırası ve dataIndex kimliği için önemlidir.
  nodes: seri.map((veri) => {
    const düğüm = yerleşim.get(veri.id);
    return ({
    id: veri.id,
    value: veri.value || 0,
    depth: veri.depth,
    index: veri.index,
    x: düğüm.x,
    y: düğüm.y,
    r: düğüm.r,
    leaf: !düğüm.children || düğüm.children.length === 0,
    label: !düğüm.children || düğüm.children.length === 0 ? yaprakAdı(düğüm.id) : ''
  });})
};

fs.mkdirSync(path.dirname(hedef), { recursive: true });
fs.writeFileSync(hedef, `${JSON.stringify(çıktı, null, 2)}\n`);
process.stdout.write(`${hedef}: ${çıktı.nodes.length} düğüm, derinlik ${enDerin}\n`);
