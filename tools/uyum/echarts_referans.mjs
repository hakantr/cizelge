#!/usr/bin/env node

// Kilitli ECharts 6.1 klonu ve kilitli echarts-examples kaynağından gerçek
// Varsayılan 700x525 veya örnek metadata'sından gelen 4:3 viewport'ta
// referans kare üretir. Harici CDN kullanılmaz.

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
  const sonuç = { id: '', output: '', frame: 1, state: 'başlangıç', width: 700, height: 525 };
  for (let sıra = 2; sıra < process.argv.length; sıra += 1) {
    const argüman = process.argv[sıra];
    const değer = process.argv[sıra + 1];
    if (argüman === '--id') sonuç.id = değer;
    else if (argüman === '--output') sonuç.output = değer;
    else if (argüman === '--frame') sonuç.frame = Number(değer);
    else if (argüman === '--state') sonuç.state = değer;
    else if (argüman === '--width') sonuç.width = Number(değer);
    else if (argüman === '--height') sonuç.height = Number(değer);
    else throw new Error(`bilinmeyen argüman: ${argüman}`);
    sıra += 1;
  }
  if (!sonuç.id || !sonuç.output) throw new Error('--id ve --output zorunludur');
  sonuç.frame = Math.max(0, Math.min(1, sonuç.frame));
  sonuç.width = Math.max(1, Math.round(sonuç.width));
  sonuç.height = Math.max(1, Math.round(sonuç.height));
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

function html(id, kaynak, frame, state, width, height) {
  const ecStatBetigi = kaynak.includes('ecStat')
    ? '<script src="/ecStat.min.js"></script>'
    : '';
  const erkenRastgele = id === 'bar-breaks-brush' || id === 'bar-breaks-simple'
    // Kırık zikzağı, global rastgele akışta kendinden önce kaç tüketim
    // yapıldığından etkilenmemeli. 0.5 resmî algoritmanın geçerli ve sabit
    // bir girdisidir; hem modül yükleme hem görünüm kurma evresini kilitler.
    ? '<script>Math.random = () => 0.5;</script>'
    : '';
  const kümelemeSırası = id === 'scatter-clustering-process' && state.startsWith('step-')
    ? Number(state.slice('step-'.length))
    : null;
  const aggregateTikleri = id === 'scatter-aggregate-bar'
    ? state === 'bar' ? 1 : state === 'scatter-return' ? 2 : 0
    : null;
  const symbolMorphTikleri = id === 'scatter-symbol-morph' && state.startsWith('shape-')
    ? Number(state.slice('shape-'.length))
    : null;
  const sonEylem = Number.isInteger(kümelemeSırası)
    ? `myChart.dispatchAction({type:'timelineChange', currentIndex:${kümelemeSırası}});`
    : Number.isInteger(symbolMorphTikleri) && symbolMorphTikleri > 0
      ? `{
          const zamanlayıcı = window.__capturedIntervals[0];
          if (!zamanlayıcı || zamanlayıcı.ms !== 700) {
            throw new Error('scatter-symbol-morph 700 ms zamanlayıcısı yakalanamadı');
          }
          for (let tik = 0; tik < ${symbolMorphTikleri}; tik += 1) zamanlayıcı.callback();
        }`
    : Number.isInteger(aggregateTikleri) && aggregateTikleri > 0
      ? `{
          const zamanlayıcı = window.__capturedIntervals[0];
          if (!zamanlayıcı || zamanlayıcı.ms !== 2000) {
            throw new Error('scatter-aggregate-bar 2000 ms zamanlayıcısı yakalanamadı');
          }
          for (let tik = 0; tik < ${aggregateTikleri}; tik += 1) zamanlayıcı.callback();
        }`
    : id === 'mix-zoom-on-value' && state === 'son'
    ? `myChart.dispatchAction({type:'dataZoom', start:70, end:100});`
    : id === 'bar-brush' && state === 'seçim'
      ? `{
          myChart.dispatchAction({
            type:'brush',
            areas:[{
              brushType:'lineX',
              coordRange:['Class2', 'Class5'],
              xAxisIndex:0
            }]
          });
          const metin = myChart.getOption().title?.[0]?.text;
          const beklenen = 'SELECTED DATA INDICES: \\n'
            + '[Series 0] 3, 4, 5\\n'
            + '[Series 1] 3, 4, 5\\n'
            + '[Series 2] 2, 3, 4\\n'
            + '[Series 3] 2, 3, 4';
          if (metin !== beklenen) {
            throw new Error('bar-brush brushSelected başlığı üretilmedi: ' + metin);
          }
        }`
    : id === 'scatter-nutrients-matrix' && state === 'zoom-left'
      ? `myChart.dispatchAction({type:'dataZoom', dataZoomIndex:0, start:20, end:80});`
    : id === 'bar-breaks-simple' && state === 'genişlet'
      ? `{
          myChart.dispatchAction({
            type:'expandAxisBreak',
            yAxisIndex:0,
            breaks:[{start:5000, end:100000}]
          });
          const expanded = (myChart.getOption().yAxis[0].breaks || [])
            .find((item) => item.start === 5000 && item.end === 100000);
          if (!expanded || expanded.isExpanded !== true) {
            throw new Error('bar-breaks-simple expandAxisBreak durumu uygulanmadı');
          }
          const graphic = myChart.getOption().graphic || [];
          const elements = graphic.flatMap((item) => item.elements || []);
          const button = elements.find((item) => item.name === 'collapseAxisBreakBtn');
          if (!button || button.ignore === true) {
            throw new Error('bar-breaks-simple genişletme düğmeyi görünür yapmadı');
          }
        }`
    : id === 'bar-breaks-simple' && state === 'daralt'
      ? `{
          myChart.dispatchAction({
            type:'expandAxisBreak',
            yAxisIndex:0,
            breaks:[{start:5000, end:100000}]
          });
          const zr = myChart.getZr();
          zr.flush();
          const target = zr.storage.getDisplayList()
            .find((item) => item.name === 'collapseAxisBreakBtn');
          if (!target) {
            throw new Error('bar-breaks-simple gerçek graphic düğmesi bulunamadı');
          }
          zr.trigger('click', {target, offsetX:75, offsetY:17});
          const breaks = myChart.getOption().yAxis[0].breaks || [];
          if (breaks.some((item) => item.isExpanded === true)) {
            throw new Error('bar-breaks-simple düğme tıklaması kırıkları daraltmadı');
          }
          const graphic = myChart.getOption().graphic || [];
          const elements = graphic.flatMap((item) => item.elements || []);
          const button = elements.find((item) => item.name === 'collapseAxisBreakBtn');
          if (!button || button.ignore !== true) {
            throw new Error('bar-breaks-simple daraltma düğmeyi ignore yapmadı');
          }
        }`
    : id === 'bar-breaks-brush' && state === 'fırça'
      ? `{
          const x = myChart.getWidth() / 2;
          const y0 = myChart.convertToPixel({yAxisIndex:0}, 2000);
          const y1 = myChart.convertToPixel({yAxisIndex:0}, 3000);
          const zr = myChart.getZr();
          zr.trigger('mousedown', {offsetX:x, offsetY:y0});
          zr.trigger('mousemove', {offsetX:x, offsetY:y1});
          const mouseup = new MouseEvent('mouseup', {clientX:x, clientY:y1, bubbles:true});
          Object.defineProperties(mouseup, {
            offsetX: {value:x},
            offsetY: {value:y1}
          });
          document.dispatchEvent(mouseup);
          // Kaynağın tam açıklıktan %2 boşluğa indirdiği sıfır gecikmeli
          // ikinci setOption çağrısını tamamla.
          await new Promise((resolve) => setTimeout(resolve, 0));
          const breaks = myChart.getOption().yAxis[0].breaks || [];
          if (!breaks.some((item) => item.start === 2000 && item.end === 3000 && item.gap === '2%')) {
            throw new Error('bar-breaks-brush fırça etkileşimi ikinci kırığı üretmedi');
          }
          myChart.dispatchAction({
            type:'expandAxisBreak',
            yAxisIndex:0,
            breaks:[{start:5000, end:100000}]
          });
          const expanded = (myChart.getOption().yAxis[0].breaks || [])
            .find((item) => item.start === 5000 && item.end === 100000);
          if (!expanded || expanded.isExpanded !== true) {
            throw new Error('bar-breaks-brush eski kırığı genişletme eylemi uygulanmadı');
          }
          // Mousemove tooltip'i yeni kırığı örter. Kanıt karesi, gerçek
          // fırçanın eklediği 2000–3000 kırığını tek başına ve bütünüyle
          // karşılaştırır; eski kırık yerleşik action ile genişletilmiştir.
          myChart.dispatchAction({type:'hideTip'});
          zr.trigger('globalout', {});
        }`
    : id === 'bar-breaks-brush' && state === 'sıfırla'
      ? `{
          myChart.dispatchAction({
            type:'expandAxisBreak',
            yAxisIndex:0,
            breaks:[{start:5000, end:100000}]
          });
          const item = (myChart.getOption().yAxis[0].breaks || [])[0];
          if (!item || item.isExpanded !== true) {
            throw new Error('bar-breaks-brush expandAxisBreak sıfırlama durumunu üretmedi');
          }
        }`
    : id === 'scatter-nutrients' && state === 'axes-fat-fiber'
      ? `{
          app.config.xAxis = 'fat';
          app.config.yAxis = 'fiber';
          app.config.onChange();
        }`
    : id === 'dataset-link' && state === 'son'
      ? `{
          const x = myChart.convertToPixel({xAxisIndex: 0}, '2014');
          myChart.dispatchAction({type:'updateAxisPointer', x, y:400});
        }`
      : id === 'dynamic-data2' && state === 'ipucu'
        ? `{
            const x = myChart.convertToPixel({xAxisIndex: 0}, data[700].value[0]);
            myChart.dispatchAction({type:'updateAxisPointer', x, y:250});
          }`
        : id === 'dynamic-data' && state === 'ipucu'
          ? `{
              const x = myChart.convertToPixel({xAxisIndex: 0}, categories[6]);
              myChart.dispatchAction({type:'updateAxisPointer', x, y:250});
            }`
        : id === 'heatmap-cartesian' && state === 'ipucu'
          ? `myChart.dispatchAction({type:'showTip', seriesIndex:0, dataIndex:${3 * 24 + 13}});`
        : id === 'heatmap-cartesian' && state === 'aralık'
          ? `myChart.dispatchAction({type:'selectDataRange', visualMapIndex:0, selected:[3, 7]});`
        : id === 'heatmap-large-piecewise' && state === 'parça'
          ? `myChart.dispatchAction({
              type:'selectDataRange',
              visualMapIndex:0,
              selected:{0:true,1:true,2:true,3:false,4:true,5:true,6:true,7:true}
            });`
        : id === 'dynamic-data2' && state === 'son'
        ? `{
            const zamanlayıcı = window.__capturedIntervals[0];
            if (!zamanlayıcı || zamanlayıcı.ms !== 1000) {
              throw new Error('dynamic-data2 1000 ms zamanlayıcısı yakalanamadı');
            }
            // Resmî callback'i yirmi kez çalıştır: her çağrı beş noktayı
            // kaydırır; son kare tam olarak 20 saniyelik canlı durumdur.
            for (let tik = 0; tik < 20; tik += 1) zamanlayıcı.callback();
          }`
        : id === 'dynamic-data' && state === 'son'
          ? `{
              const zamanlayıcı = window.__capturedIntervals[0];
              if (!zamanlayıcı || zamanlayıcı.ms !== 2100) {
                throw new Error('dynamic-data 2100 ms zamanlayıcısı yakalanamadı');
              }
              // On tik, başlangıç penceresindeki bütün örnekleri kaynak
              // callback'in ürettiği canlı değerlerle değiştirir.
              for (let tik = 0; tik < 10; tik += 1) {
                window.__setNow((tik + 1) * 2100);
                zamanlayıcı.callback();
              }
            }`
      : '';
  const zamanlayıcıyıBekle = id === 'dataset-link'
    || (id === 'bar-brush' && state === 'seçim')
    || id === 'bar-breaks-brush'
    || id === 'bar-breaks-simple'
    || (id === 'dynamic-data2' && state === 'ipucu')
    ? `await new Promise((resolve) => setTimeout(resolve, 0));`
    : '';
  const zamanŞeridiAnimasyonunuTamamla = id === 'scatter-clustering-process'
    && Number.isInteger(kümelemeSırası);
  const hedefMs = zamanŞeridiAnimasyonunuTamamla
    // Resmî örnek checkpointStyle.animationDuration=1500 kullanıyor.
    // timelineChange veriyi hemen güncellese de kontrol noktasını bir
    // sonraki karelerde taşır; kanıt seçilen adımın tamamlanmış
    // görsel durumunu karşılaştırır.
    ? 1500
    : id === 'scatter-effect'
      || id === 'calendar-effectscatter'
      || id === 'calendar-charts'
      ? frame * 2000
      : 0;
  return `<!doctype html><html><head><meta charset="utf-8"><style>
html,body,#viewport{margin:0;width:${width}px;height:${height}px;overflow:hidden}
</style>${erkenRastgele}<script src="/echarts.js"></script>${ecStatBetigi}</head><body><div id="viewport"></div><script>
(() => {
  let now = 0;
  let nextId = 1;
  let queue = [];
  const epoch = 1704067200000;
  const NativeDate = window.Date;
  function FixedDate(...args) {
    if (!new.target) return new NativeDate(epoch + now).toString();
    return args.length === 0
      ? new NativeDate(epoch + now)
      : new NativeDate(...args);
  }
  FixedDate.prototype = NativeDate.prototype;
  Object.setPrototypeOf(FixedDate, NativeDate);
  FixedDate.now = () => epoch + now;
  FixedDate.parse = NativeDate.parse;
  FixedDate.UTC = NativeDate.UTC;
  window.Date = FixedDate;
  try { Object.defineProperty(performance, 'now', { value: () => now }); } catch (_) {}
  window.requestAnimationFrame = (callback) => {
    const id = nextId++;
    queue.push({id, callback});
    return id;
  };
  window.cancelAnimationFrame = (id) => { queue = queue.filter((item) => item.id !== id); };
  // Kaynak zamanlayıcısının Date saatini, renderer kare kuyruğunu araya
  // sokmadan ilerletir. Böylece örneğin Math.random akışı yalnız kaynak
  // callback'leri tarafından tüketilir ve iki koşucu arasında birebirdir.
  window.__setNow = (target) => { now = Math.max(now, target); };
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
Math.random = ${id === 'bar-breaks-brush' || id === 'bar-breaks-simple' ? '() => 0.5' : `() => {
  seed |= 0; seed = seed + 0x6D2B79F5 | 0;
  let t = Math.imul(seed ^ seed >>> 15, 1 | seed);
  t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t;
  return ((t ^ t >>> 14) >>> 0) / 4294967296;
}`};
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
  // Resmî örnekler veri varlığını hem jQuery get hem de getJSON ile
  // çağırır; ikisi de JSON yanıtı ve aynı jQuery-benzeri promise zincirini
  // kullanır.
  getJSON(url, callback) {
    return this.get(url, callback);
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
  renderer: 'canvas', width: ${width}, height: ${height}, devicePixelRatio: 1
});
window.__chart = myChart;
const originalSetOption = myChart.setOption.bind(myChart);
myChart.setOption = (...args) => {
  // dynamic-data her callback'te setOption'dan sonra yeniden rastgele
  // değer üretir. Görselleştiricinin dahili tüketimini yalnız bu kaynağın
  // veri akışından ayır; diğer kilitli referansların tarihsel davranışını
  // değiştirme.
  const sourceRandomKorunacak = ${id === 'dynamic-data'};
  const sourceSeed = seed;
  const value = originalSetOption(...args);
  if (sourceRandomKorunacak) seed = sourceSeed;
  window.__applied = true;
  return value;
};
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
${id === 'dynamic-data2' || id === 'dynamic-data' || id === 'scatter-aggregate-bar' || id === 'scatter-symbol-morph' ? `
// Yalnız örnek kaynağının kurduğu zamanlayıcıyı yakala; ECharts çekirdeği
// ve renderer başlatılırken kullanılan olası iç zamanlayıcılar bu kapsama
// girmez. Callback değişmeden saklanıp son durum eyleminde yeniden oynatılır.
window.__capturedIntervals = [];
window.setInterval = (callback, ms) => {
  window.__capturedIntervals.push({callback, ms});
  return window.__capturedIntervals.length;
};
window.clearInterval = (timerId) => {
  const sıra = Number(timerId) - 1;
  if (sıra >= 0 && sıra < window.__capturedIntervals.length) {
    window.__capturedIntervals[sıra] = null;
  }
};
` : ''}
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
  if (${zamanŞeridiAnimasyonunuTamamla}) myChart.getZr().animation.update();
  window.__advance(${hedefMs});
  // ECharts betiği yüklenirken native requestAnimationFrame'i kendi içine
  // bağlar. Sanal saati ilerlettikten sonra zrender animasyon yöneticisini
  // bir kez elle güncelleyerek hedef andaki tamamlanmış durumu deterministik
  // biçimde boya; gerçek zamanlı bir beklemeye bağlı kalma.
  if (${zamanŞeridiAnimasyonunuTamamla}) myChart.getZr().animation.update();
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
    else if (url.pathname === '/ecStat.min.js') {
      // Pinned ECharts klonunun test varlığı, resmî echarts-stat UMD
      // dağıtımını taşır; kümeleme/regresyon referansları CDN'e çıkmaz.
      dosya = path.join(ECHARTS, 'test/lib/ecStat.min.js');
    }
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
          && path.resolve(dosya) !== path.resolve(ECHARTS, 'test/lib/ecStat.min.js')
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
  const { sunucu, url } = await sunucuBaşlat(
    html(args.id, kaynak, args.frame, args.state, args.width, args.height)
  );
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
    await sayfa.setViewport({ width: args.width, height: args.height, deviceScaleFactor: 1 });
    await sayfa.emulateTimezone('Europe/Istanbul');
    await sayfa.goto(url, { waitUntil: 'networkidle0', timeout: 30000 });
    await sayfa.waitForFunction(() => window.__ready || window.__referenceError, { timeout: 30000 });
    const hata = await sayfa.evaluate(() => window.__referenceError || null);
    if (hata) throw new Error(hata);
    if (args.id === 'bar-gradient' && (args.state === 'vurgu' || args.state === 'yakınlaştır')) {
      // Gerçek canvas işaretçi yolunu kullan: vurgu karesi zrender hit-test
      // üzerinden emphasis durumuna, yakınlaştırma karesi ise resmî kaynakta
      // kayıtlı chart.on('click') callback'ine ulaşır.
      const nokta = await sayfa.evaluate(() => {
        const chart = window.__chart;
        const seri = chart.getModel().getSeriesByIndex(0);
        const öğe = seri?.getData?.()?.getItemGraphicEl?.(8);
        const kutu = öğe?.getBoundingRect?.()?.clone?.();
        const dönüşüm = öğe?.getComputedTransform?.();
        if (!kutu) throw new Error('bar-gradient dataIndex=8 grafik öğesi bulunamadı');
        if (dönüşüm) kutu.applyTransform(dönüşüm);
        return { x: kutu.x + kutu.width / 2, y: kutu.y + kutu.height / 2 };
      });
      if (args.state === 'vurgu') await sayfa.mouse.move(nokta.x, nokta.y);
      else await sayfa.mouse.click(nokta.x, nokta.y);
      const etkileşim = await sayfa.evaluate((durum) => {
        const chart = window.__chart;
        window.__advance(350);
        chart.getZr().animation.update();
        chart.getZr().flush();
        const öğe = chart.getModel().getSeriesByIndex(0).getData().getItemGraphicEl(8);
        const seçenek = chart.getOption();
        return {
          vurgulu: öğe?.currentStates?.includes?.('emphasis') === true,
          başlangıç: seçenek.dataZoom?.[0]?.startValue,
          bitiş: seçenek.dataZoom?.[0]?.endValue
        };
      }, args.state);
      if (args.state === 'vurgu' && !etkileşim.vurgulu) {
        throw new Error('bar-gradient gerçek pointer hareketi emphasis durumunu üretmedi');
      }
      if (args.state === 'yakınlaştır'
          && !['者', 5].includes(etkileşim.başlangıç)) {
        throw new Error(`bar-gradient click startValue üretmedi: ${etkileşim.başlangıç}`);
      }
      if (args.state === 'yakınlaştır'
          && !['上', 11].includes(etkileşim.bitiş)) {
        throw new Error(`bar-gradient click endValue üretmedi: ${etkileşim.bitiş}`);
      }
    }
    if (process.env.UYUM_DEBUG_LAYOUT) {
      const yerleşim = await sayfa.evaluate((yalnızDataZoom) => {
        const chart = window.__chart;
        const sonuç = [];
        const tooltipAdayları = [...document.querySelectorAll('div')]
          .filter((öğe) => (öğe.textContent?.includes('Dynamic Bar') || öğe.textContent?.includes('Punch Card'))
            && getComputedStyle(öğe).position === 'absolute');
        if (tooltipAdayları.length) {
          const kök = tooltipAdayları.sort((a, b) => a.getBoundingClientRect().width - b.getBoundingClientRect().width).at(-1);
          const özetle = (öğe) => {
            const kutu = öğe.getBoundingClientRect();
            const stil = getComputedStyle(öğe);
            return {
              etiket: öğe.tagName,
              metin: öğe.childElementCount ? null : öğe.textContent,
              kutu: {x: kutu.x, y: kutu.y, width: kutu.width, height: kutu.height},
              stil: {
                marginTop: stil.marginTop,
                padding: stil.padding,
                fontSize: stil.fontSize,
                lineHeight: stil.lineHeight
              },
              çocuklar: [...öğe.children].map(özetle)
            };
          };
          sonuç.push({bileşen: 'tooltipDom', ağaç: özetle(kök)});
        }
        const ölçümBağlamı = document.createElement('canvas').getContext('2d');
        if (ölçümBağlamı) {
          ölçümBağlamı.font = '12px sans-serif';
          sonuç.push({
            bileşen: 'fontMetrics',
            yazılar: ['Email', 'Union Ads', 'Video Ads', 'Direct', 'Search Engine', 'Ads', 'Union', 'Video', 'Search', 'Engine']
              .map((metin) => ({metin, genişlik: ölçümBağlamı.measureText(metin).width}))
          });
        }
        const kırılmaŞekilleri = chart.getZr().storage.getDisplayList()
          .filter((öğe) => Array.isArray(öğe?.shape?.points) && öğe.z >= 100)
          .map((öğe) => ({
            tür: öğe.type,
            z: öğe.z,
            noktalar: öğe.shape.points,
            stil: {
              fill: öğe.style?.fill,
              stroke: öğe.style?.stroke,
              lineWidth: öğe.style?.lineWidth,
              lineDash: öğe.style?.lineDash,
              opacity: öğe.style?.opacity
            }
          }));
        if (kırılmaŞekilleri.length) {
          sonuç.push({bileşen: 'axisBreakShapes', öğeler: kırılmaŞekilleri});
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
          const gölgeÖzetle = (grup) => grup?.children?.().map((öğe) => {
            const noktalar = öğe.shape?.points || [];
            return {
              tür: öğe.type,
              noktaSayısı: noktalar.length,
              ilkNoktalar: noktalar.slice(0, 12),
              sonNoktalar: noktalar.slice(-3),
              stil: {
                fill: öğe.style?.fill,
                stroke: öğe.style?.stroke,
                lineWidth: öğe.style?.lineWidth,
                opacity: öğe.style?.opacity
              },
              dönüşüm: öğe.getComputedTransform?.()
            };
          }) || [];
          sonuç.push({
            sıra: model.componentIndex,
            tür: model.subType,
            seçenek: model.option,
            konum: view?._location,
            boyut: view?._size,
            uçlar: view?._handleEnds,
            grup: view?.group ? { x: view.group.x, y: view.group.y } : null,
            sürgüGrubu: özetle(görüntüler?.sliderGroup),
            sürgüÖğeleri: görüntüler?.sliderGroup?.children?.().map(özetle),
            sınır: sınır ? { x: sınır.x, y: sınır.y, width: sınır.width, height: sınır.height } : null,
            filler: özetle(görüntüler?.filler),
            veriGölgeleri: görüntüler?.dataShadowSegs?.map(gölgeÖzetle),
            tutamaçlar: görüntüler?.handles?.map(özetle),
            taşıma: özetle(görüntüler?.moveHandle)
          });
        });
        if (yalnızDataZoom) return sonuç;
        chart.getModel().eachComponent('visualMap', (model) => {
          const view = chart.getViewOfComponentModel(model);
          const öğeler = [];
          view?.group?.traverse?.((öğe) => {
            const yerel = öğe?.getBoundingRect?.();
            if (!yerel) return;
            const dünya = yerel.clone();
            const dönüşüm = öğe.getComputedTransform?.();
            if (dönüşüm) dünya.applyTransform(dönüşüm);
            öğeler.push({
              tür: öğe.type,
              metin: öğe.style?.text,
              şekil: öğe.shape,
              stil: öğe.style ? {
                fill: öğe.style.fill,
                stroke: öğe.style.stroke,
                lineWidth: öğe.style.lineWidth,
                opacity: öğe.style.opacity
              } : null,
              x: öğe.x,
              y: öğe.y,
              rotation: öğe.rotation,
              yerel: {x: yerel.x, y: yerel.y, width: yerel.width, height: yerel.height},
              dünya: {x: dünya.x, y: dünya.y, width: dünya.width, height: dünya.height},
              dönüşüm
            });
          });
          const sınır = view?.group?.getBoundingRect?.();
          sonuç.push({
            bileşen: 'visualMap',
            sıra: model.componentIndex,
            seçenek: model.option,
            itemSize: model.itemSize,
            seçili: model.getSelected?.(),
            uçlar: view?._handleEnds,
            grup: view?.group ? {x: view.group.x, y: view.group.y} : null,
            sınır: sınır ? {x: sınır.x, y: sınır.y, width: sınır.width, height: sınır.height} : null,
            öğeler
          });
        });
        chart.getModel().eachSeries((model) => {
          if (model.subType !== 'heatmap') return;
          const view = chart.getViewOfSeriesModel(model);
          const öğeler = [];
          view?.group?.traverse?.((öğe) => {
            const yerel = öğe?.getBoundingRect?.();
            if (!yerel) return;
            const dünya = yerel.clone();
            const dönüşüm = öğe.getComputedTransform?.();
            if (dönüşüm) dünya.applyTransform(dönüşüm);
            const bağlıMetin = öğe.getTextContent?.();
            const bağlıYerel = bağlıMetin?.getBoundingRect?.();
            const bağlıDünya = bağlıYerel?.clone?.();
            const bağlıDönüşüm = bağlıMetin?.getComputedTransform?.();
            if (bağlıDünya && bağlıDönüşüm) bağlıDünya.applyTransform(bağlıDönüşüm);
            öğeler.push({
              tür: öğe.type,
              şekil: öğe.shape,
              metin: öğe.style?.text,
              stil: öğe.style ? {
                fill: öğe.style.fill,
                stroke: öğe.style.stroke,
                lineWidth: öğe.style.lineWidth,
                shadowBlur: öğe.style.shadowBlur,
                shadowColor: öğe.style.shadowColor
              } : null,
              etiket: bağlıMetin ? {
                metin: bağlıMetin.style?.text,
                yapılandırma: öğe.textConfig,
                stil: {
                  fill: bağlıMetin.style?.fill,
                  stroke: bağlıMetin.style?.stroke,
                  opacity: bağlıMetin.style?.opacity,
                  font: bağlıMetin.style?.font,
                  align: bağlıMetin.style?.align,
                  verticalAlign: bağlıMetin.style?.verticalAlign
                },
                dünya: bağlıDünya ? {
                  x: bağlıDünya.x,
                  y: bağlıDünya.y,
                  width: bağlıDünya.width,
                  height: bağlıDünya.height
                } : null
              } : null,
              dünya: {x: dünya.x, y: dünya.y, width: dünya.width, height: dünya.height}
            });
          });
          sonuç.push({
            bileşen: 'series',
            tür: model.subType,
            sıra: model.seriesIndex,
            öğeler
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
              dolgu: öğe.style?.fill,
              x: öğe.x,
              y: öğe.y,
              hiza: öğe.style?.align,
              dikeyHiza: öğe.style?.verticalAlign,
              dönüşüm: öğe.getComputedTransform?.(),
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
      }, Boolean(process.env.UYUM_DEBUG_DATAZOOM));
      process.stderr.write(`${JSON.stringify(yerleşim, null, 2)}\n`);
    }
    fs.mkdirSync(path.dirname(path.resolve(args.output)), { recursive: true });
    await sayfa.screenshot({
      path: path.resolve(args.output),
      clip: { x: 0, y: 0, width: args.width, height: args.height }
    });
  } finally {
    await tarayıcı.close();
    sunucu.close();
  }
}

await çalıştır();
