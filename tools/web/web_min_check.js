// WEBREL1-E2: the engine's web start in headless Chromium. Serves a folder holding the bound example (web_min.js and
// web_min_bg.wasm from `wasm-bindgen --target web`; tools/web/web_min_check.sh builds it), writes the page (a canvas with
// id "aegis") and checks, at a device pixel ratio of 1 and of 2: the canvas is 1280 x 720 CSS px drawn at the ratio; a
// frame is presented (the canvas pixels are not one colour and show the screen's first background); a key and a left
// click reach the screen (the title the example writes, and the background colour, change); a right click goes back
// and its context menu is prevented; audio is off before the first gesture and on after it; the shell draws at the ratio.
// Usage: node tools/web/web_min_check.js <served folder>. Needs Playwright (`require('playwright')`) and its Chromium
// (or CHROMIUM=<path to the executable>). Exit 0 when every check passed.
'use strict';
const fs = require('fs');
const http = require('http');
const path = require('path');
const { chromium } = require('playwright');

const dir = process.argv[2];
if (!dir || !fs.existsSync(path.join(dir, 'web_min_bg.wasm'))) {
  console.error('usage: node web_min_check.js <folder with web_min.js and web_min_bg.wasm>');
  process.exit(2);
}
fs.writeFileSync(path.join(dir, 'index.html'), `<!doctype html>
<html><head><meta charset="utf-8"><title>web_min</title>
<style>body { margin: 0; background: #111; } canvas { display: block; }</style></head>
<body><canvas id="aegis"></canvas>
<script type="module">import init from './web_min.js'; init();</script>
</body></html>
`);

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const server = http.createServer((req, res) => {
  const file = path.join(dir, path.normalize(decodeURIComponent(req.url.split('?')[0])).replace(/^([/\\])+/, '') || 'index.html');
  fs.readFile(file, (err, data) => {
    if (err) { res.writeHead(404); res.end(); return; }
    res.writeHead(200, { 'Content-Type': TYPES[path.extname(file)] || 'application/octet-stream' });
    res.end(data);
  });
});

// The example's four backgrounds, one per press (examples/web_min.rs `Board::draw`).
const BG = [[32, 48, 96], [96, 32, 48], [32, 96, 48], [96, 96, 32]];
let failures = 0;
function check(ok, what) {
  console.log(`${ok ? 'PASS' : 'FAIL'} ${what}`);
  if (!ok) failures++;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// The canvas as presented on screen (a screenshot of the element), decoded in the page: the number of distinct colours
// and the colour near its bottom-left corner (the background: no text or mark is drawn there).
async function picture(page) {
  const png = (await page.locator('#aegis').screenshot()).toString('base64');
  return page.evaluate(async (b64) => {
    const img = new Image();
    img.src = 'data:image/png;base64,' + b64;
    await img.decode();
    const c = document.createElement('canvas');
    c.width = img.width; c.height = img.height;
    const g = c.getContext('2d');
    g.drawImage(img, 0, 0);
    const px = g.getImageData(0, 0, c.width, c.height).data;
    const seen = new Set();
    for (let i = 0; i < px.length; i += 4 * 7) seen.add((px[i] << 16) | (px[i + 1] << 8) | px[i + 2]);
    const at = ((c.height - 10) * c.width + 10) * 4;
    return { w: c.width, h: c.height, colours: seen.size, bg: [px[at], px[at + 1], px[at + 2]] };
  }, png);
}

const near = (a, b) => a.every((v, i) => Math.abs(v - b[i]) <= 3);

async function waitFor(fn, ms) {
  const until = Date.now() + ms;
  for (;;) {
    const v = await fn();
    if (v || Date.now() > until) return v;
    await sleep(100);
  }
}

async function run(base, dpr) {
  console.log(`-- device pixel ratio ${dpr}`);
  // Both the flag and the context's scale: with only the context's, Chromium's emulation reports a
  // devicePixelContentBoxSize in CSS px (which winit sizes the canvas by); with only the flag, a devicePixelRatio of 1.
  const args = [`--force-device-scale-factor=${dpr}`];
  const browser = await chromium.launch(process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM, args } : { args });
  const context = await browser.newContext({ viewport: { width: 1400, height: 800 }, deviceScaleFactor: dpr });
  const page = await context.newPage();
  const errors = [];
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  page.on('pageerror', (e) => errors.push(String(e)));
  await page.goto(base + '/index.html');
  const title = () => page.title();
  const field = async (k) => ((await title()).match(new RegExp(`${k}=(\\S+)`)) || [])[1];

  const drawn = await waitFor(async () => Number(await field('frames')) > 0, 20000);
  check(drawn, `the screen draws frames (title "${await title()}")`);
  const size = await page.evaluate(() => { const c = document.getElementById('aegis'); return [c.width, c.height, c.clientWidth, c.clientHeight]; });
  check((await field('canvas')) === `${1280 * dpr}x${720 * dpr}` && Number(await field('dpr')) === dpr, `the shell draws ${1280 * dpr} x ${720 * dpr} px at ratio ${dpr} (title "${await title()}")`);
  check(size[0] === 1280 * dpr && size[1] === 720 * dpr && size[2] === 1280 && size[3] === 720, `canvas 1280 x 720 CSS px at ${1280 * dpr} x ${720 * dpr} px (got ${size.join(' ')})`);
  const first = await waitFor(async () => { const p = await picture(page); return p.colours > 2 && near(p.bg, BG[0]) ? p : null; }, 20000);
  check(!!first, `a frame is presented: not one colour, first background ${BG[0]} (${JSON.stringify(first || await picture(page))})`);
  check((await field('audio')) === 'off', 'audio is off before the first gesture');

  // Key: the canvas has the focus from the start (shell::start_web focuses it).
  await page.keyboard.press('a');
  const keyed = await waitFor(async () => (await field('key')) === 'a', 5000);
  check(keyed, `key "a" reaches the screen (title "${await title()}")`);
  check((await field('audio')) === 'on', 'audio is on after the first gesture (a key)');
  const afterKey = await waitFor(async () => { const p = await picture(page); return near(p.bg, BG[1]) ? p : null; }, 5000);
  check(!!afterKey, `the key shows in the pixels: background ${BG[1]}`);

  // Left click at CSS (300, 200): the screen's old units are half the CSS px at any ratio (640 x 360 over 1280 x 720).
  await page.mouse.click(300, 200);
  const clicked = await waitFor(async () => (await field('click')) === '150,100', 5000);
  check(clicked, `the click reaches the screen at 150,100 (title "${await title()}")`);
  const afterClick = await waitFor(async () => { const p = await picture(page); return near(p.bg, BG[2]) ? p : null; }, 5000);
  check(!!afterClick, `the click shows in the pixels: background ${BG[2]}`);

  // Right click: back, and no browser context menu (the page sees the event already prevented).
  await page.evaluate(() => { window.__menu = []; document.addEventListener('contextmenu', (e) => window.__menu.push(e.defaultPrevented)); });
  await page.mouse.click(400, 300, { button: 'right' });
  const backed = await waitFor(async () => (await field('back')) === '1', 5000);
  check(backed, `the right click goes back (title "${await title()}")`);
  const menu = await page.evaluate(() => window.__menu);
  check(menu.length > 0 && menu.every((p) => p), `the context menu is prevented (${JSON.stringify(menu)})`);

  check(errors.length === 0, `no console errors (${JSON.stringify(errors)})`);
  await browser.close();
}

(async () => {
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const base = `http://127.0.0.1:${server.address().port}`;
  try {
    for (const dpr of [1, 2]) await run(base, dpr);
  } finally {
    server.close();
  }
  console.log(failures === 0 ? 'web_min: all checks passed' : `web_min: ${failures} check(s) failed`);
  process.exit(failures === 0 ? 0 : 1);
})().catch((e) => { console.error(e); process.exit(1); });
