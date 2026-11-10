// WEBREL1-G1: the browser smoke test of the web TEST build (tools/fc.sh web test writes it into out/web/; run it with
// tools/fc.sh web smoke). Serves the folder on 127.0.0.1, opens it in headless Chromium (Playwright) and drives the game
// with the page's real keyboard and mouse only. It reads the screens through the test build's page probe
// (`aegisProbe()`, aegis_ui feature `web_probe`: the agent's ui_tree as JSON plus where the canvas draws), so every
// check is on screen ids, widget ids and state, never on pixels; a widget is clicked at the centre of its rectangle.
//   boot   the loading bar shows while part 1 comes and is gone at the main menu, which has no Quit (a page cannot
//          close its tab); both bundle parts are fetched
//   no GL  in a browser without WebGL the renderer fails: the loop stops and the page says why (no frames, no canvas)
//   query  ?screen=runner&port=1 opens the Scene Runner; port= is dropped with a console line
//   (a)    Tools > Combat Arena (goblin_skirmish, the runner's default fight) plays on the battle's own autoplay (A) to
//          the results banner (an outcome, the Leave button); Leave goes back to the runner
//   (b)    New game > Start reaches the first region screen in the world the new game screen had selected
//   (c)    a step on the map, Game menu > Save game > slot 1 (the save lands in localStorage under aegis/), reload the
//          page, Continue restores the slot: the same location and position, nothing unsaved
// Every page must run without a console error or page error. Usage: node tools/web/smoke.js <out/web>. Needs
// Playwright (`require('playwright')`) and its Chromium (or CHROMIUM=<path to the executable>). Exit 0 when all passed.
'use strict';
const fs = require('fs');
const http = require('http');
const path = require('path');
const { chromium } = require('playwright');

const dir = process.argv[2];
if (!dir || !fs.existsSync(path.join(dir, 'emberwake_bg.wasm'))) {
  console.error('usage: node tools/web/smoke.js <folder of a web test build (tools/fc.sh web test)>');
  process.exit(2);
}

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.json': 'application/json' };
const fetched = [];
const server = http.createServer((req, res) => {
  const rel = path.normalize(decodeURIComponent(req.url.split('?')[0])).replace(/^([/\\])+/, '') || 'index.html';
  fs.readFile(path.join(dir, rel), (err, data) => {
    fetched.push([rel, err ? 404 : 200]);
    if (err) { res.writeHead(404); res.end(); return; }
    res.writeHead(200, { 'Content-Type': TYPES[path.extname(rel)] || 'application/octet-stream' });
    res.end(data);
  });
});

let failures = 0;
function check(ok, what) {
  console.log(`${ok ? 'PASS' : 'FAIL'} ${what}`);
  if (!ok) failures++;
  return ok;
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// One page of the game, with its probe and real input.
class Game {
  constructor(page) {
    this.page = page;
    this.errors = [];
    this.warnings = [];
    page.on('console', (m) => {
      if (m.type() === 'error') this.errors.push(m.text());
      if (m.type() === 'warning' && !m.text().includes('GL Driver')) this.warnings.push(m.text());
    });
    page.on('pageerror', (e) => this.errors.push(String(e)));
  }

  // The probe's snapshot: asked once (the next frame takes a new one), then read again a moment later.
  async probe() {
    const ask = () => this.page.evaluate(async () => {
      try { return (await import('./emberwake.js')).aegisProbe(); } catch (e) { return ''; }
    });
    await ask();
    await sleep(60);
    const text = await ask();
    return text ? JSON.parse(text) : null;
  }

  // The first snapshot `fn` accepts within `ms`, else null.
  async until(fn, ms) {
    const end = Date.now() + ms;
    for (;;) {
      const p = await this.probe();
      if (p && fn(p)) return p;
      if (Date.now() > end) return null;
      await sleep(150);
    }
  }

  async screen(id, ms = 20000) {
    return this.until((p) => p.screen === id, ms);
  }

  widget(p, id) {
    return p && p.widgets.find((w) => w.id === id);
  }

  // A left click at the centre of widget `id` (its rectangle in the screen's units, through the view onto the canvas).
  async click(id) {
    const p = await this.until((s) => this.widget(s, id), 10000);
    if (!check(!!p, `widget ${id} is on screen`)) return false;
    const w = this.widget(p, id);
    const box = await this.page.locator('#aegis').boundingBox();
    const [ox, oy, s] = p.view;
    const x = box.x + (ox + (w.rect[0] + w.rect[2] / 2) * s) / p.dpr;
    const y = box.y + (oy + (w.rect[1] + w.rect[3] / 2) * s) / p.dpr;
    await this.page.mouse.move(x, y);
    await sleep(50);
    await this.page.mouse.click(x, y);
    return true;
  }

  noErrors(what) {
    check(this.errors.length === 0, `${what}: no console or page errors (${JSON.stringify(this.errors.slice(0, 3))})`);
  }
}

// A page with the loading bar watched from the first moment (whether it was added, its last text).
async function open(context, base, query = '') {
  const page = await context.newPage();
  await page.addInitScript(() => {
    window.__loading = { seen: false, text: '' };
    new MutationObserver(() => {
      const el = document.getElementById('aegis-loading');
      if (el) { window.__loading.seen = true; window.__loading.text = el.textContent; }
    }).observe(document, { childList: true, subtree: true, characterData: true });
  });
  const game = new Game(page);
  await page.goto(`${base}/index.html${query}`);
  return game;
}

async function boot(context, base) {
  console.log('-- boot');
  const g = await open(context, base);
  const t0 = Date.now();
  const menu = await g.screen('main_menu', 60000);
  check(!!menu, `the test build boots to the main menu (${Date.now() - t0} ms)`);
  check(!!g.widget(menu, 'menu.tools'), 'the test build has the Tools entry (dev tools on)');
  check(!!menu && !g.widget(menu, 'menu.quit'), `the web main menu has no Quit (${menu && menu.widgets.filter((w) => w.id.startsWith('menu.')).map((w) => w.id).join(' ')})`);
  const loading = await g.page.evaluate(() => [window.__loading, !!document.getElementById('aegis-loading')]);
  check(loading[0].seen && !loading[1], `the loading bar showed (last "${loading[0].text}") and is gone at the menu`);
  const parts = JSON.parse(fs.readFileSync(path.join(dir, 'bundle.json'), 'utf8')).parts.map((p) => p.file);
  await sleep(500);
  for (const p of parts) check(fetched.some(([f, code]) => f === p && code === 200), `bundle part ${p} was fetched`);
  check(!g.warnings.some((w) => w.includes('did not load')), 'every later part mounted');
  g.noErrors('boot');
  await g.page.close();
}

async function query(context, base) {
  console.log('-- URL query');
  const g = await open(context, base, '?screen=runner&port=1');
  check(!!(await g.screen('scene_runner', 60000)), '?screen=runner opens the Scene Runner');
  check(g.warnings.some((w) => w.includes('port=')), 'port= is dropped with a console line');
  g.noErrors('query');
  await g.page.close();
}

async function arena(context, base) {
  console.log('-- (a) Tools > Combat Arena, goblin_skirmish to the results');
  const g = await open(context, base);
  await g.screen('main_menu', 60000);
  await g.click('menu.tools');
  const runner = await g.screen('scene_runner');
  check(!!runner, 'Tools opens the Scene Runner');
  const picked = g.widget(runner, 'runner.arena.encounter');
  check(!!picked && /goblin skirmish/i.test(picked.text), `the arena's fight is Goblin Skirmish ("${picked && picked.text}")`);
  await g.click('runner.arena');
  const fight = await g.until((p) => p.screen === 'test_arena' && p.state && p.state.playing === false, 30000);
  check(!!fight, 'Launch opens the Combat Arena');
  await g.page.keyboard.press('a');
  check(!!(await g.until((p) => p.state && p.state.auto === true, 5000)), 'A turns on the autoplay');
  const end = await g.until((p) => p.screen === 'test_arena' && p.state && p.state.outcome, 240000);
  check(!!end, `the fight plays to an outcome (${end && end.state.outcome}: "${end && end.state.message}")`);
  check(!!(end && g.widget(end, 'banner.leave')), 'the results banner shows (Leave)');
  if (end) {
    await g.click('banner.leave');
    check(!!(await g.screen('scene_runner')), 'Leave goes back to the Scene Runner');
  }
  g.noErrors('(a)');
  await g.page.close();
}

async function newGameSaveContinue(context, base) {
  console.log('-- (b) New game to the first region screen');
  const g = await open(context, base);
  await g.screen('main_menu', 60000);
  await g.click('menu.new_game');
  const ng = await g.screen('new_game');
  const world = ng && ng.state && ng.state.selected;
  check(!!world, `the new game screen selects a world (${world})`);
  await g.click('new_game.start');
  const region = await g.until((p) => p.screen === 'region_map' && p.game && p.game.location, 30000);
  check(!!region && region.game.world === world, `Start reaches the region map in ${world} (${region && region.game.location})`);
  if (!region) return;

  console.log('-- (c) save, reload the page, Continue');
  const start = region.game.position;
  let moved = null;
  for (const k of ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown']) {
    await g.page.keyboard.press(k);
    moved = await g.until((p) => p.state && !p.state.walking && String(p.game.position) !== String(start), 3000);
    if (moved) break;
  }
  check(!!moved, `a key walks the party (${start} -> ${moved && moved.game.position})`);
  const before = (moved || region).game;
  await g.click('region.menu');
  await g.screen('hub');
  await g.click('hub.save');
  await g.screen('save_slots');
  await g.click('slot.1');
  const saved = await g.until((p) => p.game && p.game.dirty === false && /^Slot 1: /.test((g.widget(p, 'slot.1') || {}).text || '') && !/Empty/.test(g.widget(p, 'slot.1').text), 10000);
  check(!!saved, `slot 1 holds the game (${saved && g.widget(saved, 'slot.1').text})`);
  const keys = await g.page.evaluate(() => Object.keys(localStorage));
  check(keys.some((k) => k.startsWith('aegis/') && k.includes('/saves/')), `the save is in localStorage (${keys.filter((k) => k.includes('/saves/')).join(', ')})`);
  g.noErrors('(b), (c) before the reload');

  g.errors.length = 0;
  await g.page.reload();
  const menu = await g.screen('main_menu', 60000);
  check(!!(menu && g.widget(menu, 'menu.continue')), 'after the reload the main menu offers Continue');
  await g.click('menu.continue');
  const back = await g.until((p) => p.screen === 'region_map' && p.game && p.game.location, 30000);
  check(!!back && back.game.location === before.location && String(back.game.position) === String(before.position) && back.game.dirty === false,
    `Continue restores the slot: ${back && back.game.location} at ${back && back.game.position} (saved ${before.location} at ${before.position})`);
  g.noErrors('(c) after the reload');
  await g.page.close();
}

// A browser with no WebGL (`--disable-3d-apis`): the renderer cannot start, so the shell stops and shows the reason
// in place of the canvas; the page probe stops answering new frames.
async function noGl(base) {
  console.log('-- no WebGL: the renderer fails on the page');
  const args = ['--disable-3d-apis'];
  const browser = await chromium.launch(process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM, args } : { args });
  try {
    const page = await (await browser.newContext({ viewport: { width: 1400, height: 800 } })).newPage();
    await page.goto(`${base}/index.html`);
    const shown = await page.waitForSelector('#aegis-failure', { timeout: 60000 }).then((el) => el.textContent()).catch(() => null);
    check(!!shown && /renderer/.test(shown), `the page says the renderer did not start ("${shown && shown.slice(0, 90)}")`);
    const hidden = await page.evaluate(() => getComputedStyle(document.getElementById('aegis')).display === 'none');
    check(hidden, 'the canvas is hidden');
    const g = new Game(page);
    const a = await g.probe();
    await sleep(500);
    const b = await g.probe();
    check(!a || !b || a.frame === b.frame, `no frames are drawn after the failure (frame ${a && a.frame} -> ${b && b.frame})`);
  } finally {
    await browser.close();
  }
}

(async () => {
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const base = `http://127.0.0.1:${server.address().port}`;
  const browser = await chromium.launch(process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {});
  try {
    // A fresh context (its own localStorage) per part, so no part sees another's saves or settings.
    for (const part of [boot, query, arena, newGameSaveContinue]) {
      const context = await browser.newContext({ viewport: { width: 1400, height: 800 } });
      await part(context, base);
      await context.close();
    }
    await noGl(base);
  } finally {
    await browser.close();
    server.close();
  }
  console.log(failures === 0 ? 'web smoke: all checks passed' : `web smoke: ${failures} check(s) failed`);
  process.exit(failures === 0 ? 0 : 1);
})().catch((e) => { console.error(e); process.exit(1); });
