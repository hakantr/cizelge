#!/usr/bin/env node

// Kilitli ECharts 6.1 klonu ve kilitli echarts-examples kaynağından gerçek
// 700x525 referans kare üretir. Harici CDN kullanılmaz.

import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import process from 'node:process';
import { once } from 'node:events';
import puppeteer from 'puppeteer-core';
import ts from 'typescript';

const ARAÇ = path.dirname(new URL(import.meta.url).pathname);
const KÖK = path.resolve(ARAÇ, '../..');
const ECHARTS = path.resolve(KÖK, '../echarts');
const ÖRNEKLER = path.resolve(KÖK, '../echarts-examples');
const CHROME_ADAYLARI = [
  process.env.PUPPETEER_EXECUTABLE_PATH,
  '/usr/bin/google-chrome',
  '/usr/bin/google-chrome-stable',
  '/usr/bin/chromium',
  '/usr/bin/chromium-browser',
  path.join(
    process.env.HOME || '',
    '.cache/puppeteer/chrome-headless-shell/mac_arm-131.0.6778.204/chrome-headless-shell-mac-arm64/chrome-headless-shell'
  ),
  path.join(
    process.env.HOME || '',
    '.cache/puppeteer/chrome/mac_arm-131.0.6778.204/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'
  )
].filter(Boolean);

function argümanlarıOku() {
  const sonuç = { id: '', output: '', frame: 1, state: 'başlangıç' };
  for (let sıra = 2; sıra < process.argv.length; sıra += 1) {
    const argüman = process.argv[sıra];
    const değer = process.argv[sıra + 1];
    if (argüman === '--id') sonuç.id = değer;
    else if (argüman === '--output') sonuç.output = değer;
    else if (argüman === '--frame') sonuç.frame = Number(değer);
    else if (argüman === '--state') sonuç.state = değer;
    else throw new Error(`bilinmeyen argüman: ${argüman}`);
    sıra += 1;
  }
  if (!sonuç.id || !sonuç.output) throw new Error('--id ve --output zorunludur');
  sonuç.frame = Math.max(0, Math.min(1, sonuç.frame));
  return sonuç;
}

function chromeBul() {
  const bulunan = CHROME_ADAYLARI.find((aday) => fs.existsSync(aday));
  if (!bulunan) {
    throw new Error('Puppeteer için yerel Chrome bulunamadı; PUPPETEER_EXECUTABLE_PATH ayarlayın');
  }
  return bulunan;
}

function örnekJavaScript(id) {
  const adaylar = [
    path.join(ÖRNEKLER, 'public/examples/ts', `${id}.ts`),
    path.join(ÖRNEKLER, 'public/examples/ts', `${id}.js`)
  ];
  const dosya = adaylar.find((aday) => fs.existsSync(aday));
  if (!dosya) throw new Error(`örnek kaynağı bulunamadı: ${id}`);
  const kaynak = fs
    .readFileSync(dosya, 'utf8')
    .replace(/\bexport\s*\{\s*\}\s*;?/g, '');
  return ts
    .transpileModule(kaynak, {
      compilerOptions: {
        target: ts.ScriptTarget.ES2020,
        module: ts.ModuleKind.None,
        removeComments: true
      }
    })
    .outputText
    .replace(/\bexport\s*\{\s*\}\s*;?/g, '')
    // TypeScript 4.4, yalnız tipte kullanılan normal `import` silinse bile
    // dosyayı modül sayıp CommonJS `exports` işareti bırakır. Örnekler
    // klasik, satır içi tarayıcı betiği olarak çalıştığından bu saf işaretin
    // çalışma zamanı karşılığı yoktur; kaynak davranışını değiştirmeden at.
    .replace(/Object\.defineProperty\(exports,\s*["']__esModule["'],\s*\{\s*value:\s*true\s*\}\);?\s*/g, '');
}

function html(id, kaynak, frame, state) {
  const sonEylem = id === 'mix-zoom-on-value' && state === 'son'
    ? `myChart.dispatchAction({type:'dataZoom', start:70, end:100});`
    : id === 'dataset-link' && state === 'son'
      ? `{
          const x = myChart.convertToPixel({xAxisIndex: 0}, '2014');
          myChart.dispatchAction({type:'updateAxisPointer', x, y:400});
        }`
      : '';
  const zamanlayıcıyıBekle = id === 'dataset-link'
    ? `await new Promise((resolve) => setTimeout(resolve, 0));`
    : '';
  const hedefMs = id === 'scatter-effect' ? frame * 2000 : 0;
  return `<!doctype html><html><head><meta charset="utf-8"><style>
html,body,#viewport{margin:0;width:700px;height:525px;overflow:hidden}
</style><script src="/echarts.js"></script></head><body><div id="viewport"></div><script>
(() => {
  let now = 0;
  let nextId = 1;
  let queue = [];
  const epoch = 1704067200000;
  Date.now = () => epoch + now;
  try { Object.defineProperty(performance, 'now', { value: () => now }); } catch (_) {}
  window.requestAnimationFrame = (callback) => {
    const id = nextId++;
    queue.push({id, callback});
    return id;
  };
  window.cancelAnimationFrame = (id) => { queue = queue.filter((item) => item.id !== id); };
  window.__advance = (target) => {
    const end = Math.max(now, target);
    do {
      now = Math.min(end, now + 1000 / 60);
      const current = queue;
      queue = [];
      for (const item of current) item.callback(now);
    } while (now < end);
  };
})();
let seed = 0x5eed1234;
Math.random = () => {
  seed |= 0; seed = seed + 0x6D2B79F5 | 0;
  let t = Math.imul(seed ^ seed >>> 15, 1 | seed);
  t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t;
  return ((t ^ t >>> 14) >>> 0) / 4294967296;
};
window.ROOT_PATH = location.origin;
window.CDN_PATH = location.origin + '/';
window.app = {};
window.$ = {
  get(url, callback) {
    const görev = fetch(url).then((yanıt) => {
      if (!yanıt.ok) throw new Error('veri isteği: ' + yanıt.status + ' ' + url);
      return yanıt.json();
    }).then(callback);
    window.__pending = görev;
    return görev;
  },
  getScript(url) {
    const görev = new Promise((resolve, reject) => {
      const betik = document.createElement('script');
      betik.src = url;
      betik.onload = resolve;
      betik.onerror = () => reject(new Error('betik isteği başarısız: ' + url));
      document.head.appendChild(betik);
    });
    window.__pending = görev;
    return görev;
  },
  when(...görevler) {
    return {
      done(callback) {
        // jQuery $.when, ajax görevlerinin sonucunu üçlü bir dizi
        // biçiminde ayrı argümanlar olarak iletir. Örnekler res[0] ile
        // veriyi aldığı için en az ilk hücreyi aynı biçimde koru.
        const görev = Promise.all(görevler).then((değerler) =>
          callback(...değerler.map((değer) => [değer]))
        );
        window.__pending = görev;
        return görev;
      }
    };
  }
};
const myChart = echarts.init(document.getElementById('viewport'), null, {
  renderer: 'canvas', width: 700, height: 525, devicePixelRatio: 1
});
window.__chart = myChart;
const originalSetOption = myChart.setOption.bind(myChart);
myChart.setOption = (...args) => { const value = originalSetOption(...args); window.__applied = true; return value; };
echarts.registerPreprocessor((opt) => {
  opt.animation = false;
  const series = Array.isArray(opt.series) ? opt.series : opt.series ? [opt.series] : [];
  for (const item of series) {
    item.animation = false;
    item.progressive = 100000;
  }
  for (const key of ['title', 'legend', 'toolbox']) {
    const values = Array.isArray(opt[key]) ? opt[key] : opt[key] ? [opt[key]] : [];
    for (const item of values) if (item.padding == null) item.padding = 15;
  }
});
${kaynak}
window.__sourceDone = true;
(async () => {
  if (window.__pending) await window.__pending;
  // Veri URL'si veya yerel URL kullanan resmî örneklerde ECharts'a
  // setOption vermeden önce tüm CanvasPattern kaynaklarının gerçekten
  // çözülmesini bekle. Yalnız networkidle beklemek, DOM'a eklenmemiş
  // new Image() nesnelerinin asenkron decode işlemini kapsamıyor ve
  // boş desenin Canvas öntanımlı siyah dolgusu olarak kilitlenmesine yol
  // açıyordu.
  if (typeof option !== 'undefined' && option) {
    const ziyaret = new WeakSet();
    const görüntüler = [];
    const görüntüleriTopla = (değer) => {
      if (!değer || (typeof değer !== 'object' && typeof değer !== 'function')) return;
      if (değer instanceof HTMLImageElement) {
        görüntüler.push(değer);
        return;
      }
      if (ziyaret.has(değer)) return;
      ziyaret.add(değer);
      for (const alt of Object.values(değer)) görüntüleriTopla(alt);
    };
    görüntüleriTopla(option);
    await Promise.all(görüntüler.map(async (görüntü) => {
      if (typeof görüntü.decode === 'function') await görüntü.decode();
      else if (!görüntü.complete) {
        await new Promise((resolve, reject) => {
          görüntü.addEventListener('load', resolve, {once: true});
          görüntü.addEventListener('error', () => reject(new Error('görüntü yüklenemedi')), {once: true});
        });
      }
    }));
  }
  ${zamanlayıcıyıBekle}
  if (!window.__applied && typeof option !== 'undefined' && option) myChart.setOption(option);
  ${sonEylem}
  window.__advance(${hedefMs});
  myChart.getZr().flush();
  window.__ready = true;
})().catch((error) => { window.__referenceError = error.stack || String(error); });
</script></body></html>`;
}

function içerikTürü(dosya) {
  if (dosya.endsWith('.js')) return 'text/javascript; charset=utf-8';
  if (dosya.endsWith('.json')) return 'application/json; charset=utf-8';
  if (dosya.endsWith('.png')) return 'image/png';
  return 'application/octet-stream';
}

async function sunucuBaşlat(sayfa) {
  const sunucu = http.createServer((istek, yanıt) => {
    const url = new URL(istek.url || '/', 'http://localhost');
    let dosya = null;
    if (url.pathname === '/echarts.js') dosya = path.join(ECHARTS, 'dist/echarts.js');
    else if (url.pathname === '/echarts-simple-transform/dist/ecSimpleTransform.min.js') {
      // ECharts'ın kendi pinned test paketindeki resmî UMD derlemesi;
      // harici CDN kullanılmadan örnekle aynı registerTransform yolu çalışır.
      dosya = path.join(ECHARTS, 'test/lib/ecSimpleTransform.js');
    }
    else if (url.pathname.startsWith('/data/')) {
      dosya = path.join(ÖRNEKLER, 'public', url.pathname);
    }
    if (dosya) {
      if (!path.resolve(dosya).startsWith(path.resolve(ÖRNEKLER, 'public'))
          && path.resolve(dosya) !== path.resolve(ECHARTS, 'dist/echarts.js')
          && path.resolve(dosya) !== path.resolve(ECHARTS, 'test/lib/ecSimpleTransform.js')) {
        yanıt.writeHead(403).end();
        return;
      }
      if (!fs.existsSync(dosya)) {
        yanıt.writeHead(404).end();
        return;
      }
      yanıt.writeHead(200, {'content-type': içerikTürü(dosya)});
      fs.createReadStream(dosya).pipe(yanıt);
      return;
    }
    yanıt.writeHead(200, {'content-type': 'text/html; charset=utf-8'});
    yanıt.end(sayfa);
  });
  sunucu.listen(0, '127.0.0.1');
  await once(sunucu, 'listening');
  const adres = sunucu.address();
  if (!adres || typeof adres === 'string') throw new Error('referans sunucusu adres alamadı');
  return { sunucu, url: `http://127.0.0.1:${adres.port}/` };
}

async function çalıştır() {
  const args = argümanlarıOku();
  const kaynak = örnekJavaScript(args.id);
  const { sunucu, url } = await sunucuBaşlat(html(args.id, kaynak, args.frame, args.state));
  const tarayıcı = await puppeteer.launch({
    executablePath: chromeBul(),
    headless: true,
    args: ['--disable-gpu', '--no-sandbox', '--font-render-hinting=none']
  });
  try {
    const sayfa = await tarayıcı.newPage();
    sayfa.on('pageerror', (hata) => process.stderr.write(`ECharts referans sayfası: ${hata.stack || hata}\n`));
    sayfa.on('console', (ileti) => {
      if (ileti.type() === 'error') process.stderr.write(`ECharts console: ${ileti.text()}\n`);
    });
    await sayfa.setViewport({ width: 700, height: 525, deviceScaleFactor: 1 });
    await sayfa.goto(url, { waitUntil: 'networkidle0', timeout: 30000 });
    await sayfa.waitForFunction(() => window.__ready || window.__referenceError, { timeout: 30000 });
    const hata = await sayfa.evaluate(() => window.__referenceError || null);
    if (hata) throw new Error(hata);
    if (process.env.UYUM_DEBUG_LAYOUT) {
      const yerleşim = await sayfa.evaluate(() => {
        const chart = window.__chart;
        const sonuç = [];
        const ölçümBağlamı = document.createElement('canvas').getContext('2d');
        if (ölçümBağlamı) {
          ölçümBağlamı.font = '12px sans-serif';
          sonuç.push({
            bileşen: 'fontMetrics',
            yazılar: ['Email', 'Union Ads', 'Video Ads', 'Direct', 'Search Engine', 'Ads', 'Union', 'Video', 'Search', 'Engine']
              .map((metin) => ({metin, genişlik: ölçümBağlamı.measureText(metin).width}))
          });
        }
        chart.getModel().eachComponent('dataZoom', (model) => {
          const view = chart.getViewOfComponentModel(model);
          const sınır = view?.group?.getBoundingRect?.();
          const görüntüler = view?._displayables;
          const özetle = (öğe) => öğe ? {
            x: öğe.x, y: öğe.y, scaleX: öğe.scaleX, scaleY: öğe.scaleY,
            rotation: öğe.rotation,
            şekil: öğe.shape,
            stil: öğe.style ? {
              fill: öğe.style.fill,
              stroke: öğe.style.stroke,
              lineWidth: öğe.style.lineWidth,
              opacity: öğe.style.opacity
            } : null,
            dönüşüm: öğe.getComputedTransform?.(),
            sınır: (() => {
              const kutu = öğe.getBoundingRect?.();
              return kutu ? { x: kutu.x, y: kutu.y, width: kutu.width, height: kutu.height } : null;
            })(),
            dünyaSınırı: (() => {
              const kutu = öğe.getBoundingRect?.()?.clone?.();
              const dönüşüm = öğe.getComputedTransform?.();
              if (!kutu) return null;
              if (dönüşüm) kutu.applyTransform(dönüşüm);
              return { x: kutu.x, y: kutu.y, width: kutu.width, height: kutu.height };
            })()
          } : null;
          sonuç.push({
            sıra: model.componentIndex,
            tür: model.subType,
            seçenek: model.option,
            konum: view?._location,
            boyut: view?._size,
            uçlar: view?._handleEnds,
            grup: view?.group ? { x: view.group.x, y: view.group.y } : null,
            sürgüGrubu: özetle(görüntüler?.sliderGroup),
            sınır: sınır ? { x: sınır.x, y: sınır.y, width: sınır.width, height: sınır.height } : null,
            filler: özetle(görüntüler?.filler),
            tutamaçlar: görüntüler?.handles?.map(özetle),
            taşıma: özetle(görüntüler?.moveHandle)
          });
        });
        chart.getModel().eachComponent('grid', (model) => {
          const dikdörtgen = model.coordinateSystem?.getRect?.();
          sonuç.push({
            bileşen: 'grid',
            sıra: model.componentIndex,
            dikdörtgen: dikdörtgen ? {
              x: dikdörtgen.x,
              y: dikdörtgen.y,
              width: dikdörtgen.width,
              height: dikdörtgen.height
            } : null
          });
        });
        for (const anaTür of ['xAxis', 'yAxis']) {
          chart.getModel().eachComponent(anaTür, (model) => {
            const adÖğeleri = [];
            const yazıÖğeleri = [];
            const view = chart.getViewOfComponentModel(model);
            view?.group?.traverse?.((öğe) => {
              if (typeof öğe?.style?.text !== 'string' || !öğe.getBoundingRect) return;
              const yerel = öğe.getBoundingRect();
              const dünya = yerel.clone();
              const dönüşüm = öğe.getComputedTransform?.();
              if (dönüşüm) dünya.applyTransform(dönüşüm);
              const özet = {
                metin: öğe.style.text,
                x: öğe.x,
                y: öğe.y,
                dönüş: öğe.rotation,
                hiza: öğe.style.textAlign,
                dikeyHiza: öğe.style.textBaseline,
                yerel: {x: yerel.x, y: yerel.y, width: yerel.width, height: yerel.height},
                dünya: {x: dünya.x, y: dünya.y, width: dünya.width, height: dünya.height}
              };
              yazıÖğeleri.push(özet);
              if (öğe.style.text === model.get('name')) adÖğeleri.push(özet);
            });
            sonuç.push({
              bileşen: anaTür,
              sıra: model.componentIndex,
              ızgaraSırası: model.get('gridIndex'),
              ad: model.get('name'),
              kapsam: model.axis?.getExtent?.(),
              ölçekKapsamı: model.axis?.scale?.getExtent?.(),
              ölçekAralığı: model.axis?.scale?._interval,
              yaklaşıkAralık: model.axis?.scale?._approxInterval,
              enAltZamanBirimi: model.axis?.scale?._minLevelUnit,
              çentikler: model.axis?.scale?.getTicks?.().map((çentik) => ({
                değer: çentik.value,
                kırılma: çentik.break ? {
                  tür: çentik.break.type,
                  başlangıç: çentik.break.parsedBreak?.vmin,
                  bitiş: çentik.break.parsedBreak?.vmax
                } : null,
                zaman: çentik.time
              })),
              adÖğeleri,
              yazıÖğeleri
            });
          });
        }
        chart.getModel().eachComponent('legend', (model) => {
          const view = chart.getViewOfComponentModel(model);
          const özetle = (öğe) => {
            const yerel = öğe?.getBoundingRect?.();
            if (!yerel) return null;
            const dünya = yerel.clone();
            const dönüşüm = öğe.getComputedTransform?.();
            if (dönüşüm) dünya.applyTransform(dönüşüm);
            return {
              tür: öğe.type,
              metin: öğe.style?.text,
              x: öğe.x,
              y: öğe.y,
              yerel: {x: yerel.x, y: yerel.y, width: yerel.width, height: yerel.height},
              dünya: {x: dünya.x, y: dünya.y, width: dünya.width, height: dünya.height}
            };
          };
          const öğeler = [];
          view?.group?.traverse?.((öğe) => {
            const özet = özetle(öğe);
            if (özet) öğeler.push(özet);
          });
          sonuç.push({
            bileşen: 'legend',
            sıra: model.componentIndex,
            gruplar: view?._contentGroup?.children?.().map(özetle).filter(Boolean) || [],
            öğeler
          });
        });
        chart.getModel().eachSeries((model) => {
          const veri = model.getData();
          const koordinat = model.coordinateSystem;
          const view = chart.getViewOfSeriesModel(model);
          const pastaÖğeleri = [];
          if (model.subType === 'pie') {
            view?.group?.traverse?.((öğe) => {
              if (öğe === view.group || !öğe.getBoundingRect) return;
              const kutu = öğe.getBoundingRect();
              const dünya = kutu.clone();
              const dönüşüm = öğe.getComputedTransform?.();
              if (dönüşüm) dünya.applyTransform(dönüşüm);
              pastaÖğeleri.push({
                tür: öğe.type,
                x: öğe.x,
                y: öğe.y,
                dönüş: öğe.rotation,
                şekil: öğe.shape,
                öğeStili: {
                  fill: öğe.style?.fill,
                  stroke: öğe.style?.stroke,
                  lineWidth: öğe.style?.lineWidth,
                  shadowBlur: öğe.style?.shadowBlur,
                  shadowColor: öğe.style?.shadowColor,
                  opacity: öğe.style?.opacity
                },
                metin: öğe.style?.text,
                hiza: öğe.style?.align,
                dikeyHiza: öğe.style?.verticalAlign,
                sınır: {x: kutu.x, y: kutu.y, width: kutu.width, height: kutu.height},
                dünyaSınırı: {x: dünya.x, y: dünya.y, width: dünya.width, height: dünya.height},
                etiket: (() => {
                  const etiket = öğe.getTextContent?.();
                  if (!etiket) return null;
                  const sınır = etiket.getBoundingRect?.();
                  const dünyaSınırı = sınır?.clone?.();
                  const matris = etiket.getComputedTransform?.();
                  if (dünyaSınırı && matris) dünyaSınırı.applyTransform(matris);
                  return {
                    x: etiket.x, y: etiket.y,
                    stil: etiket.style,
                    sınır: sınır ? {x: sınır.x, y: sınır.y, width: sınır.width, height: sınır.height} : null,
                    dünyaSınırı: dünyaSınırı ? {x: dünyaSınırı.x, y: dünyaSınırı.y, width: dünyaSınırı.width, height: dünyaSınırı.height} : null
                  };
                })(),
                etiketÇizgisi: (() => {
                  const çizgi = öğe.getTextGuideLine?.();
                  return çizgi ? {şekil: çizgi.shape, stil: çizgi.style} : null;
                })()
              });
            });
          }
          sonuç.push({
            seri: model.name,
            tür: model.subType,
            örnekÖğeYerleşimi: veri.getItemLayout?.(0),
            tümÖğeYerleşimleri: Array.from(
              {length: veri.count?.() || 0},
              (_, sıra) => veri.getItemLayout?.(sıra)
            ),
            örnekNoktalar: veri.getLayout?.('points')?.slice?.(0, 14),
            sütunYerleşimi: {
              bandWidth: veri.getLayout('bandWidth'),
              offset: veri.getLayout('offset'),
              size: veri.getLayout('size')
            },
            örnekNokta: koordinat?.dataToPoint?.([419, 0]),
            pastaÖğeleri
          });
        });
        chart.getModel().eachComponent('toolbox', (model) => {
          const view = chart.getViewOfComponentModel(model);
          const çocuklar = [];
          view?.group?.traverse?.((öğe) => {
            if (öğe === view.group || !öğe.getBoundingRect) return;
            const kutu = öğe.getBoundingRect();
            çocuklar.push({
              tür: öğe.type,
              x: öğe.x,
              y: öğe.y,
              scaleX: öğe.scaleX,
              scaleY: öğe.scaleY,
              sınır: { x: kutu.x, y: kutu.y, width: kutu.width, height: kutu.height }
            });
          });
          sonuç.push({ araçKutusu: true, seçenek: model.option, grup: { x: view.group.x, y: view.group.y }, çocuklar });
        });
        chart.getModel().eachComponent('legend', (model) => {
          const view = chart.getViewOfComponentModel(model);
          const çocuklar = [];
          view?.group?.traverse?.((öğe) => {
            if (öğe === view.group || !öğe.getBoundingRect) return;
            const kutu = öğe.getBoundingRect();
            çocuklar.push({
              tür: öğe.type,
              x: öğe.x,
              y: öğe.y,
              şekil: öğe.shape,
              stil: öğe.style,
              sınır: { x: kutu.x, y: kutu.y, width: kutu.width, height: kutu.height }
            });
          });
          sonuç.push({ gösterge: true, seçenek: model.option, grup: { x: view.group.x, y: view.group.y }, çocuklar });
        });
        const metinler = chart.getZr().storage.getDisplayList(true)
          .filter((öğe) => öğe.type === 'tspan' || typeof öğe.style?.text === 'string')
          .map((öğe) => {
            const kutu = öğe.getBoundingRect?.();
            const dünya = kutu?.clone?.();
            const dönüşüm = öğe.getComputedTransform?.();
            if (dünya && dönüşüm) dünya.applyTransform(dönüşüm);
            return {
              tür: öğe.type,
              metin: öğe.style?.text,
              font: öğe.style?.font,
              x: öğe.x,
              y: öğe.y,
              hiza: öğe.style?.align,
              dikeyHiza: öğe.style?.verticalAlign,
              sınır: kutu ? { x: kutu.x, y: kutu.y, width: kutu.width, height: kutu.height } : null,
              dünyaSınırı: dünya ? { x: dünya.x, y: dünya.y, width: dünya.width, height: dünya.height } : null
            };
          });
        sonuç.push({ metinler });
        const kesikliYollar = chart.getZr().storage.getDisplayList(true)
          .filter((öğe) => öğe.style?.stroke && öğe.style.stroke !== 'none')
          .map((öğe) => ({
            tür: öğe.type,
            şekil: öğe.shape,
            stil: öğe.style,
            x: öğe.x,
            y: öğe.y,
            dönüşüm: öğe.getComputedTransform?.(),
            ec: öğe.__ecComponentInfo || null
          }));
        sonuç.push({ kesikliYollar });
        return sonuç;
      });
      process.stderr.write(`${JSON.stringify(yerleşim, null, 2)}\n`);
    }
    fs.mkdirSync(path.dirname(path.resolve(args.output)), { recursive: true });
    await sayfa.screenshot({ path: path.resolve(args.output), clip: { x: 0, y: 0, width: 700, height: 525 } });
  } finally {
    await tarayıcı.close();
    sunucu.close();
  }
}

await çalıştır();
