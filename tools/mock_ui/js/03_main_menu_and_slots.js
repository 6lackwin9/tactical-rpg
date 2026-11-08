// ============================================================================
// SCREEN 1a: MAIN MENU (1280x720 Native Grid — menus.rs Grid::Native)
// ============================================================================
function renderMainMenu() {
  // 1. Tools removed from player-facing MainMenu; 5. No subtitle on hover, centered under Title Card
  const items = [
    { label: 'Continue' },
    { label: 'New game' },
    { label: 'Load game' },
    { label: 'Settings' },
    { label: 'Quit' }
  ];

  // 1b: Fixed 4-slot party array (each slot gets an identical 25% width block)
  const activeHeroes = Object.values(STATE.roster).filter(h => h.active).slice(0, 4);
  const fixedPartySlots = [0, 1, 2, 3].map(i => activeHeroes[i] || null);

  // 1a: Hover Peek ONLY appears on Continue (idx === 0), not on Load game (idx === 2)
  const showPeek = (STATE.menuHover === 0 && !STATE.newGameModalOpen);

  // Atmospheric ash specks & gentle rising embers from the portal torches/lanterns
  const ashSpecks = [
    { x: 96,  d: '9.5s', del: '-1.2s', s: 3.5 },
    { x: 210, d: '11.2s', del: '-4.5s', s: 3.0 },
    { x: 320, d: '8.8s', del: '-2.8s', s: 4.0 },
    { x: 440, d: '12.0s', del: '-6.1s', s: 3.0 },
    { x: 560, d: '9.4s', del: '-3.3s', s: 3.5 },
    { x: 680, d: '10.6s', del: '-7.4s', s: 3.0 },
    { x: 790, d: '8.9s', del: '-0.9s', s: 4.0 },
    { x: 910, d: '11.5s', del: '-5.2s', s: 3.0 },
    { x: 1040, d: '9.2s', del: '-2.1s', s: 3.5 },
    { x: 1160, d: '10.8s', del: '-6.8s', s: 3.0 }
  ];
  const emberSparks = [
    { x: 369, y: 528, d: '5.6s', del: '-0.8s', s: 3.5, c: '#fbbf24' },
    { x: 409, y: 575, d: '6.2s', del: '-2.1s', s: 3.5, c: '#f59e0b' },
    { x: 526, y: 539, d: '5.1s', del: '-1.4s', s: 3.5, c: '#f97316' },
    { x: 745, y: 580, d: '5.8s', del: '-3.5s', s: 3.5, c: '#fbbf24' },
    { x: 922, y: 580, d: '5.4s', del: '-4.0s', s: 3.5, c: '#f97316' },
    { x: 1097, y: 580, d: '6.0s', del: '-2.7s', s: 3.5, c: '#fbbf24' }
  ];

  // Light Category 1: Indoor Stained-Glass & Sanctuary Window Light (.fx-window-glow — subtle breathing crimson-amber bloom)
  // Includes newly lit Upper Bell-Tower Lancet Windows (205, 92) and Upper Gable Arch Window (422, 158)
  const indoorWindows = [
    // Upper Bell-Tower Lancet Windows: center=(205, 92), bbox=(154,56)..(243,111)
    { x: 205, y: 92,  w: 94,  h: 66,  del: '-0.9s', grad: 'radial-gradient(ellipse at center, rgba(251,146,60,0.30) 0%, rgba(220,38,38,0.16) 48%, rgba(0,0,0,0) 74%)' },
    // Upper Gable Arch Window (above Rose Window): center=(422, 158), bbox=(404,130)..(442,182)
    { x: 422, y: 158, w: 56,  h: 66,  del: '-2.5s', grad: 'radial-gradient(ellipse at center, rgba(251,146,60,0.32) 0%, rgba(220,38,38,0.18) 48%, rgba(0,0,0,0) 74%)' },
    // Central Rose Window: center=(423, 286), bbox=(381,228)..(466,339)
    { x: 423, y: 286, w: 108, h: 118, del: '0s',    grad: 'radial-gradient(ellipse at center, rgba(251,146,60,0.34) 0%, rgba(220,38,38,0.18) 46%, rgba(0,0,0,0) 74%)' },
    // Gallery Lancet Row (6 small arches below Rose Window): center=(415, 380), bbox=(371,358)..(475,399)
    { x: 415, y: 380, w: 116, h: 48,  del: '-1.8s', grad: 'radial-gradient(ellipse at center, rgba(251,146,60,0.30) 0%, rgba(220,38,38,0.15) 50%, rgba(0,0,0,0) 76%)' },
    // Left Tall Stained-Glass Lancet Window: center=(282, 543), bbox=(264,480)..(298,597)
    { x: 282, y: 543, w: 52,  h: 126, del: '-3.1s', grad: 'radial-gradient(ellipse at center, rgba(251,146,60,0.32) 0%, rgba(185,28,28,0.18) 48%, rgba(0,0,0,0) 76%)' },
    // Right Tall Stained-Glass Lancet Window: center=(535, 554), bbox=(510,504)..(556,616)
    { x: 535, y: 554, w: 54,  h: 122, del: '-4.4s', grad: 'radial-gradient(ellipse at center, rgba(251,146,60,0.32) 0%, rgba(185,28,28,0.18) 48%, rgba(0,0,0,0) 76%)' },
    // Open Cathedral Sanctuary Doorway: center=(409, 564), bbox=(385,516)..(439,598)
    { x: 409, y: 564, w: 72,  h: 98,  del: '-2.2s', grad: 'radial-gradient(ellipse at center, rgba(254,215,170,0.36) 0%, rgba(234,88,12,0.18) 48%, rgba(0,0,0,0) 76%)' }
  ];

  // Light Category 2: Outdoor Wrought-Iron Lanterns (.fx-lantern-glow — toned-down golden point-light flicker; ground brazier removed)
  const outdoorLanterns = [
    // Left Portal Wall-Bracket Iron Lantern: center=(369, 528)
    { x: 369, y: 528, w: 50,  h: 50, del: '-0.4s' },
    // Right Portal Wall-Bracket Iron Lantern: center=(526, 539)
    { x: 526, y: 539, w: 50,  h: 50, del: '-1.5s' },
    // Bridge Post Lantern 1 (Left Pier): center=(745, 580)
    { x: 745, y: 580, w: 52,  h: 52, del: '-2.1s' },
    // Bridge Post Lantern 2 (Middle Pier): center=(922, 580)
    { x: 922, y: 580, w: 52,  h: 52, del: '-0.8s' },
    // Bridge Post Lantern 3 (Right Pier): center=(1097, 580)
    { x: 1097, y: 580, w: 52, h: 52, del: '-1.7s' }
  ];

  // 1b: Simpler Hover Peek layout + Fixed 25% width per party member block (Kaelen, Vesper, Maren, Hollis identical width)
  const peekInner = `
    <!-- Row 1: Act I Name ("Act I · Chronicles of the Sunk Ward") + Day Pill -->
    <div style="display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid rgba(217,119,6,0.38);padding-bottom:6px;">
      <span class="font-title" style="font-size:15.5px;color:var(--gold);letter-spacing:0.3px;">Act I · Chronicles of the Sunk Ward</span>
      <span class="font-mono" style="font-size:11.5px;color:var(--gold);background:rgba(15,17,24,0.9);border:1px solid #4b5563;padding:1px 8px;border-radius:2px;">Day ${STATE.day}</span>
    </div>

    <!-- Row 2: Simple Region & Mission Line -->
    <div style="display:flex;align-items:center;justify-content:space-between;font-size:13px;padding:4px 0;">
      <span style="color:var(--text);font-weight:bold;">The Sluice Undercroft</span>
      <span style="color:var(--dim);font-size:12px;">M1 · Corvo's Toll</span>
    </div>

    <!-- Row 3: 4 Fixed-Size 25% Party Member Blocks (exact equal width for Kaelen, Vesper, Maren, Hollis) -->
    <div style="display:grid;grid-template-columns:repeat(4, minmax(0, 1fr));gap:6px;width:100%;padding-top:6px;border-top:1px solid rgba(55,65,81,0.5);">
      ${fixedPartySlots.map(h => h ? `
        <div style="width:100%;min-width:0;box-sizing:border-box;display:flex;flex-direction:column;align-items:center;gap:4px;background:rgba(10,11,16,0.82);border:1px solid #374151;padding:6px 4px;border-radius:2px;">
          <div class="portrait-box gold-rim" style="width:40px;height:40px;flex-shrink:0;">
            <img src="${ASSETS.portraits[h.id]}">
          </div>
          <span style="width:100%;text-align:center;font-size:11.5px;color:var(--text);font-weight:bold;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${h.name.split(' ')[0]}</span>
        </div>
      ` : `
        <div style="width:100%;min-width:0;box-sizing:border-box;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:4px;background:rgba(10,11,16,0.45);border:1px dashed #2d333f;padding:6px 4px;border-radius:2px;">
          <div style="width:40px;height:40px;border:1px solid #1f2937;background:#07080c;"></div>
          <span style="width:100%;text-align:center;font-size:11px;color:var(--dim);">Empty</span>
        </div>
      `).join('')}
    </div>
  `;

  const diffDescriptions = [
    'Forgiving enemy damage & morale checks; focus on the story of the Sunk Ward.',
    'Intended tactical balance — full AP economy, enemy Foresight tells & Tithing pressure.',
    'Relentless Inquisition squads, stricter Tithing deadlines, and lethal critical strikes.'
  ];
  const curDiffIdx = STATE.settings.difficultyIdx;

  return `
    <div style="width:1280px;height:720px;position:relative;overflow:hidden;">
      <!-- Full-screen 1280x720 v2 background (all windows lit, ground brazier removed, lower canal mist added) -->
      <img src="${ASSETS.bg.title}" class="pixel" style="position:absolute;inset:0;width:1280px;height:720px;object-fit:cover;z-index:1;">

      <!-- Subtle Edge Vignette -->
      <div style="position:absolute;inset:0;background:radial-gradient(circle at 38% 58%, rgba(10,11,16,0) 44%, rgba(10,11,16,0.52) 100%);pointer-events:none;z-index:2;"></div>

      <!-- 1. Indoor Stained-Glass & Sanctuary Window Light (.fx-window-glow — toned down) -->
      ${indoorWindows.map(w => `
        <div class="fx-window-glow" style="left:${Math.round(w.x - w.w/2)}px;top:${Math.round(w.y - w.h/2)}px;width:${w.w}px;height:${w.h}px;animation-delay:${w.del};background:${w.grad};"></div>
      `).join('')}

      <!-- 2. Outdoor Wrought-Iron Lantern Glow (.fx-lantern-glow — toned down, no ground brazier) -->
      ${outdoorLanterns.map(l => `
        <div class="fx-lantern-glow" style="left:${Math.round(l.x - l.w/2)}px;top:${Math.round(l.y - l.h/2)}px;width:${l.w}px;height:${l.h}px;animation-delay:${l.del};background:radial-gradient(ellipse at center, rgba(254,243,199,0.58) 0%, rgba(245,158,11,0.28) 42%, rgba(234,88,12,0.08) 65%, rgba(0,0,0,0) 78%);"></div>
      `).join('')}

      <!-- 3. Low-Lying Sunk Ward Canal Mist & Haze Band over Unlit Lower Structures (y=540..720) -->
      <div class="fx-mist-band" style="top:545px;height:175px;background:radial-gradient(ellipse 65% 70% at 32% 68%, rgba(148,163,184,0.22) 0%, rgba(100,116,139,0.12) 52%, rgba(15,23,42,0) 85%);"></div>
      <div class="fx-mist-band" style="top:585px;height:135px;animation-delay:-7s;animation-duration:19s;background:radial-gradient(ellipse 70% 65% at 68% 72%, rgba(148,163,184,0.20) 0%, rgba(71,85,105,0.12) 55%, rgba(15,23,42,0) 88%);"></div>

      <!-- 60fps Canvas Particle Layer (Gentle Rising Cinders + Drifting Sunk Ward Ash) -->
      <canvas id="title-fx-canvas" width="1280" height="720" style="position:absolute;inset:0;width:1280px;height:720px;pointer-events:none;z-index:4;"></canvas>

      <!-- CSS Keyframe Ash & Cinder Motes -->
      <div class="fx-layer">
        ${ashSpecks.map(a => `
          <div class="ash-mote" style="left:${a.x}px;top:-16px;width:${a.s}px;height:${a.s}px;animation-duration:${a.d};animation-delay:${a.del};"></div>
        `).join('')}
        ${emberSparks.map(e => `
          <div class="ember-mote" style="left:${e.x}px;top:${e.y}px;width:${e.s}px;height:${e.s}px;background:${e.c};box-shadow:0 0 6px 1px ${e.c};animation-duration:${e.d};animation-delay:${e.del};"></div>
        `).join('')}
      </div>

      <!-- Title Card (ui/logo_title.png: "EMBERWAKE / A SCARLET LULLABY" at x=776, y=38, w=470, h=124; center_x = 1011) -->
      <img src="${ASSETS.ui.logo_title}" style="position:absolute;left:776px;top:38px;width:470px;height:124px;image-rendering:auto;filter:drop-shadow(0 6px 14px rgba(0,0,0,0.95));z-index:8;">

      <!-- 1a & 1b: Simpler Hover Peek Card ONLY on Continue (idx === 0) using Reduced Corner-Only Frame (ornateCornerBox) -->
      <div id="hover-peek-wrap" style="display:${showPeek ? 'block' : 'none'};">
        ${ornateCornerBox(
          peekInner,
          'position:absolute;left:456px;top:268px;width:392px;height:166px;padding:12px 16px;z-index:9;'
        )}
      </div>

      <!-- Menu Column (Centered horizontally under Title Card center_x = 1011: left=881, w=260; in-place hover so click is never lost) -->
      <div id="main-menu-col" style="position:absolute;left:881px;top:268px;width:260px;display:flex;flex-direction:column;gap:8px;z-index:9;">
        ${items.map((it, idx) => {
          const isHot = STATE.menuHover === idx;
          return `
            <div class="menu-item-720 ${isHot ? 'hot' : ''}"
                 data-menu-idx="${idx}"
                 onmouseenter="updateMenuHover(${idx})"
                 onclick="mainMenuClick(${idx})">
              <span class="menu-diamond" style="font-size:13px;color:${isHot ? 'var(--gold)' : 'transparent'};transition:color 0.12s;">◆</span>
              <span>${it.label}</span>
              <span class="menu-diamond" style="font-size:13px;color:${isHot ? 'var(--gold)' : 'transparent'};transition:color 0.12s;">◆</span>
            </div>
          `;
        }).join('')}
      </div>

      <!-- Version stamp -->
      <div class="font-body" style="position:absolute;right:20px;bottom:14px;font-size:13px;color:var(--dim);text-shadow:0 2px 4px #000;z-index:8;">
        Version 0.4.0 · Aegis Engine
      </div>

      <!-- New Game Campaign Setup Modal (Reduced Corner-Only Frame + Fixed Card Heights & Generous Button Gap so text never touches buttons or shifts) -->
      ${STATE.newGameModalOpen ? `
        <div class="modal-backdrop" style="z-index:30;">
          ${ornateCornerBox(`
            <div style="display:flex;align-items:flex-start;justify-content:space-between;gap:16px;border-bottom:1px solid #374151;padding-bottom:10px;flex-shrink:0;">
              <div>
                <div class="font-title" style="font-size:21px;color:var(--gold);">New Chronicle · Act I: Chronicles of the Sunk Ward</div>
                <div style="font-size:12.5px;color:var(--dim);margin-top:2px;">Select your campaign rules before entering the Sunk Ward. These settings are locked for the duration of the run.</div>
              </div>
              ${closeBtn720("STATE.newGameModalOpen=false;render();", "Close [Esc / Right-Click]")}
            </div>

            <div style="display:flex;flex-direction:column;gap:14px;margin:14px 0;flex:1;justify-content:center;">
              <!-- Campaign Difficulty Selection (Fixed 86px height + 24px gap between text and stepper) -->
              <div class="rust-card" style="height:86px;padding:12px 18px;display:flex;align-items:center;justify-content:space-between;gap:24px;flex-shrink:0;">
                <div style="flex:1;min-width:0;padding-right:12px;">
                  <div class="font-title" style="font-size:16px;color:var(--text);">Campaign Difficulty</div>
                  <div style="font-size:12px;color:var(--gold);margin-top:4px;height:34px;line-height:17px;overflow:hidden;">${diffDescriptions[curDiffIdx]}</div>
                </div>
                <div style="flex-shrink:0;">
                  ${renderStepper('Difficulty', STATE.settings.difficulties[curDiffIdx], "stepSetting('difficulties','difficultyIdx',-1)", "stepSetting('difficulties','difficultyIdx',1)")}
                </div>
              </div>

              <!-- Ironman Chronicle Selection (Fixed 86px height + 24px gap + 12px right padding so text never touches the button) -->
              <div class="rust-card" style="height:86px;padding:12px 18px;display:flex;align-items:center;justify-content:space-between;gap:24px;flex-shrink:0;">
                <div style="flex:1;min-width:0;padding-right:12px;">
                  <div class="font-title" style="font-size:16px;color:var(--text);">Ironman Chronicle Mode</div>
                  <div style="font-size:12px;color:var(--dim);margin-top:4px;height:34px;line-height:17px;overflow:hidden;">
                    ${STATE.settings.ironmanMode
                      ? 'Single Autosave slot only — manual saves disabled; every choice and fallen comrade is permanent.'
                      : 'Standard Chronicle — 1 Autosave slot + 3 Manual Chronicle slots available at any time.'}
                  </div>
                </div>
                <div style="flex-shrink:0;display:flex;align-items:center;">
                  <button class="rust-btn-720 ${STATE.settings.ironmanMode ? 'primary' : ''}" style="width:164px;height:32px;flex-shrink:0;" onclick="STATE.settings.ironmanMode=!STATE.settings.ironmanMode;render();">
                    ${STATE.settings.ironmanMode ? 'Ironman: On' : 'Ironman: Off'}
                  </button>
                </div>
              </div>
            </div>

            <div style="display:flex;align-items:center;justify-content:space-between;border-top:1px solid #374151;padding-top:12px;flex-shrink:0;">
              <span class="font-mono" style="font-size:11.5px;color:var(--dim);">Starting Region: The Sluice · Prologue: The Alley · [Esc / Right-Click] Close</span>
              <div style="display:flex;gap:10px;">
                <button class="rust-btn-720 primary" onclick="startNewCampaign()">Begin Chronicle [Enter]</button>
              </div>
            </div>
          `, 'width:710px;height:356px;padding:20px 26px;')}
        </div>
      ` : ''}
    </div>
  `;
}

// Update menu hover state IN-PLACE without calling render() so onclick on "New game" is never swallowed by DOM replacement
function updateMenuHover(idx) {
  STATE.menuHover = idx;
  const peekWrap = document.getElementById('hover-peek-wrap');
  if (peekWrap) {
    peekWrap.style.display = (idx === 0 && !STATE.newGameModalOpen) ? 'block' : 'none';
  }
  const menuCol = document.getElementById('main-menu-col');
  if (menuCol) {
    menuCol.querySelectorAll('.menu-item-720').forEach(el => {
      const itemIdx = Number(el.getAttribute('data-menu-idx'));
      const hot = (itemIdx === idx);
      el.classList.toggle('hot', hot);
      el.querySelectorAll('.menu-diamond').forEach(d => {
        d.style.color = hot ? 'var(--gold)' : 'transparent';
      });
    });
  }
}

// 60fps Atmospheric Canvas FX on Title Screen (Gentle Lantern Cinders + Falling Sunk Ward Ash)
let TITLE_FX_RAF = null;
const TITLE_PARTICLES = {
  init: false,
  embers: [],
  ash: []
};

function initTitleParticles() {
  if (TITLE_PARTICLES.init) return;
  TITLE_PARTICLES.init = true;
  const lanternOrigins = [
    { x: 369, y: 528 },
    { x: 409, y: 564 },
    { x: 526, y: 539 },
    { x: 745, y: 580 },
    { x: 922, y: 580 },
    { x: 1097, y: 580 }
  ];
  for (let i = 0; i < 20; i++) {
    const orig = lanternOrigins[i % lanternOrigins.length];
    TITLE_PARTICLES.embers.push({
      ox: orig.x,
      oy: orig.y,
      x: orig.x + ((i * 7) % 18) - 9,
      y: orig.y - ((i * 19) % 140),
      vx: 0.12 + (i % 5) * 0.06,
      vy: -(0.38 + (i % 6) * 0.12),
      r: 1.2 + (i % 3) * 0.45,
      phase: i * 0.7,
      color: i % 3 === 0 ? '#fbbf24' : (i % 3 === 1 ? '#f97316' : '#ef4444')
    });
  }
  for (let i = 0; i < 28; i++) {
    TITLE_PARTICLES.ash.push({
      x: (i * 47) % 1280,
      y: (i * 31) % 720,
      vx: -0.22 - (i % 4) * 0.08,
      vy: 0.38 + (i % 5) * 0.12,
      r: 1.3 + (i % 3) * 0.55,
      phase: i * 1.1
    });
  }
}

function startTitleFxCanvas() {
  if ( typeof window === 'undefined' || !window.requestAnimationFrame ) return;
  initTitleParticles();
  if (TITLE_FX_RAF && window.cancelAnimationFrame) window.cancelAnimationFrame(TITLE_FX_RAF);

  let tick = 0;
  function step() {
    const cv = document.getElementById('title-fx-canvas');
    if (!cv || STATE.screen !== 'main_menu') {
      TITLE_FX_RAF = null;
      return;
    }
    const ctx = cv.getContext && cv.getContext('2d');
    if (!ctx) return;
    tick += 0.035;
    ctx.clearRect(0, 0, 1280, 720);

    // Draw drifting ash flakes
    for (const a of TITLE_PARTICLES.ash) {
      a.x += a.vx + Math.sin(tick + a.phase) * 0.18;
      a.y += a.vy;
      if (a.y > 730) { a.y = -10; a.x = (a.x + 300) % 1280; }
      if (a.x < -10) { a.x = 1290; }
      const alpha = 0.40 + 0.22 * Math.sin(tick * 1.5 + a.phase);
      ctx.fillStyle = `rgba(229, 224, 216, ${alpha.toFixed(2)})`;
      ctx.beginPath();
      ctx.arc(a.x, a.y, a.r, 0, Math.PI * 2);
      ctx.fill();
    }

    // Draw subtle rising cinder sparks from the iron lanterns & sanctuary portal
    for (const e of TITLE_PARTICLES.embers) {
      e.x += e.vx + Math.cos(tick * 1.3 + e.phase) * 0.16;
      e.y += e.vy;
      if (e.y < e.oy - 180 || e.x > 1260) {
        e.x = e.ox + ((e.phase * 17) % 14) - 7;
        e.y = e.oy;
      }
      const dist = Math.max(0, e.oy - e.y);
      const fade = Math.max(0, Math.min(1, 1 - dist / 180));
      const pulse = 0.55 + 0.30 * Math.sin(tick * 2.6 + e.phase);
      ctx.save();
      ctx.globalAlpha = fade * pulse;
      ctx.shadowColor = e.color;
      ctx.shadowBlur = 5;
      ctx.fillStyle = e.color;
      ctx.beginPath();
      ctx.arc(e.x, e.y, e.r, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();
    }

    TITLE_FX_RAF = window.requestAnimationFrame(step);
  }
  TITLE_FX_RAF = window.requestAnimationFrame(step);
}

function startNewCampaign() {
  STATE.newGameModalOpen = false;
  STATE.screen = 'story';
  STATE.storyId = 'prologue_intro';
  STATE.storyCardIdx = 0;
  render();
}

function mainMenuClick(idx) {
  if (idx === 0) {
    // Continue -> Sluice Undercroft Hub
    STATE.screen = 'region';
    STATE.regionId = 'sluice_undercroft';
    render();
  } else if (idx === 1) {
    // New game -> Open New Chronicle Campaign Setup Modal (asks Difficulty & Ironman Mode)
    STATE.newGameModalOpen = true;
    render();
  } else if (idx === 2) {
    // Load game -> Load / Save Slots Screen
    STATE.screen = 'load_slots';
    STATE.slotsSaving = false;
    STATE.slotAskDelete = null;
    STATE.slotAskOverwrite = null;
    STATE.slotToast = 'Step 1: Click a chronicle slot to select it (for Lock / Delete). Step 2: Click selected slot again or press [Enter] to Load.';
    render();
  } else if (idx === 3) {
    // Settings -> Fullscreen Settings on Main Menu
    STATE.settingsOpen = true;
    render();
  }
}

// ============================================================================
// SCREEN 1b: LOAD / SAVE SLOTS (1280x720 Native Grid — 2a: No Scrollbar for 4 Fixed Slots; 2b: Card Click Loads/Saves)
// ============================================================================
function toggleSlotLock(idx, ev) {
  if (ev) ev.stopPropagation();
  const s = STATE.slots[idx];
  if (!s || s.empty || s.kind === 'auto') return;
  s.locked = !s.locked;
  STATE.slotToast = s.locked
    ? `Locked Slot ${idx} (protected from overwrite and delete).`
    : `Unlocked Slot ${idx}.`;
  render();
}

function promptDeleteSlot(idx, ev) {
  if (ev) ev.stopPropagation();
  const s = STATE.slots[idx];
  if (!s || s.empty) return;
  if (s.locked) {
    STATE.slotToast = 'Locked: unlock this chronicle first to delete it.';
    render();
    return;
  }
  STATE.slotAskDelete = idx;
  render();
}

function confirmDeleteSlot() {
  const idx = STATE.slotAskDelete;
  if (idx !== null && STATE.slots[idx] && !STATE.slots[idx].locked) {
    const isAuto = STATE.slots[idx].kind === 'auto';
    STATE.slots[idx] = {
      id: STATE.slots[idx].id,
      kind: isAuto ? 'auto' : 'manual',
      empty: true,
      title: isAuto ? 'Autosave · Empty' : `Slot ${idx} · Empty Chronicle`,
      act: '—',
      place: 'No recorded chronicle in this slot',
      lead: isAuto ? 'The game writes the autosave' : 'Click in Save Mode to record chronicle',
      day: 0,
      tithe: 0,
      scrap: 0,
      tallow: 0,
      when: 'Empty',
      locked: false,
      thumb: 'sluice_undercroft',
      party: []
    };
    STATE.slotToast = `Deleted chronicle in ${isAuto ? 'Autosave' : 'Slot ' + idx}.`;
  }
  STATE.slotAskDelete = null;
  render();
}

function writeSaveToSlot(idx) {
  const s = STATE.slots[idx];
  if (!s || s.kind === 'auto' || s.locked) return;
  STATE.slots[idx] = {
    id: s.id,
    kind: 'manual',
    empty: false,
    title: `Slot ${idx} · Act I: Chronicles of the Sunk Ward`,
    act: 'Act I · Chronicles of the Sunk Ward',
    place: STATE.regionId === 'corvos_gate' ? 'Third Ward · Ash-Crawl Street' : 'The Sluice Undercroft (Hub Sanctuary)',
    lead: 'M1 · Corvo\'s Toll (Third Ward Gate)',
    day: STATE.day,
    tithe: STATE.tithingDays,
    scrap: STATE.scrap,
    tallow: STATE.tallow,
    when: 'Just now',
    locked: false,
    thumb: STATE.regionId === 'corvos_gate' ? 'corvos_gate' : 'sluice_undercroft',
    party: ['kaelen', 'vesper', 'maren', 'hollis']
  };
  STATE.slotAskOverwrite = null;
  STATE.slotToast = `Saved chronicle to Slot ${idx} (Just now).`;
  render();
}

// 2. Two-Click Save / Load Process:
//    - Click 1 on an unselected slot selects it (enabling Lock [L], Trash [Del], and the primary Load/Save/Overwrite action).
//    - Click 2 on the already-selected slot (or clicking the primary action button / pressing Enter) executes Load / Save / Overwrite.
function selectSlot(idx) {
  const s = STATE.slots[idx];
  if (!s) return;
  STATE.slotSelected = idx;
  if (STATE.slotsSaving) {
    if (s.kind === 'auto') {
      STATE.slotToast = 'Autosave selected (read-only in Save Mode — select Manual Slot 1–3 to save).';
    } else if (s.locked) {
      STATE.slotToast = `Selected Slot ${idx} (Locked — click Lock [L] to unlock before overwriting).`;
    } else if (s.empty) {
      STATE.slotToast = `Selected empty Slot ${idx} — click again or press [Save Chronicle] to record here.`;
    } else {
      STATE.slotToast = `Selected Slot ${idx} — click again or press [Overwrite Chronicle] (or use Lock / Delete).`;
    }
  } else {
    if (s.empty) {
      STATE.slotToast = `Selected empty Slot ${idx} — choose an occupied slot to load.`;
    } else {
      STATE.slotToast = `Selected ${s.kind === 'auto' ? 'Autosave' : 'Slot ' + idx} — click again or press [Load Chronicle] (or use Lock / Delete).`;
    }
  }
  render();
}

function executeSlot(idx, ev) {
  if (ev) ev.stopPropagation();
  const s = STATE.slots[idx];
  if (!s) return;
  STATE.slotSelected = idx;

  if (STATE.slotsSaving) {
    if (s.kind === 'auto') {
      STATE.slotToast = 'The game writes the autosave automatically — select Manual Slot 1–3 to save.';
      render();
      return;
    }
    if (s.locked) {
      STATE.slotToast = 'Locked: unlock this slot first to save over it.';
      render();
      return;
    }
    if (!s.empty) {
      STATE.slotAskOverwrite = idx;
      render();
      return;
    }
    writeSaveToSlot(idx);
  } else {
    if (s.empty) {
      STATE.slotToast = 'Empty slot — select an occupied chronicle to load.';
      render();
      return;
    }
    if (s.thumb === 'story_alley') {
      STATE.screen = 'story';
      STATE.storyId = 'prologue_intro';
      STATE.storyCardIdx = 0;
    } else {
      STATE.screen = 'region';
      STATE.regionId = s.thumb === 'corvos_gate' ? 'corvos_gate' : 'sluice_undercroft';
      STATE.day = s.day;
      STATE.tithingDays = s.tithe;
      STATE.scrap = s.scrap;
      STATE.tallow = s.tallow;
    }
    render();
  }
}

// 2-Click Card Handler: Click 1 = Select Slot; Click 2 on already-selected slot = Execute Load/Save/Overwrite
function activateSlot(idx) {
  if (STATE.slotSelected !== idx) {
    selectSlot(idx);
  } else {
    executeSlot(idx);
  }
}

function renderLoadSlots() {
  const isRich = STATE.slotsMode === 'slot_card';
  const delSlot = STATE.slotAskDelete !== null ? STATE.slots[STATE.slotAskDelete] : null;
  const ovrSlot = STATE.slotAskOverwrite !== null ? STATE.slots[STATE.slotAskOverwrite] : null;
  const selSlot = STATE.slots[STATE.slotSelected] || STATE.slots[0];
  const selCanAct = STATE.slotsSaving
    ? (selSlot && selSlot.kind !== 'auto' && !selSlot.locked)
    : (selSlot && !selSlot.empty);
  const primaryActLabel = STATE.slotsSaving
    ? (selSlot && !selSlot.empty ? 'Overwrite Chronicle [Enter]' : 'Save Chronicle [Enter]')
    : 'Load Chronicle [Enter]';

  const innerContent = `
    <!-- Clean Player-Facing Header Bar -->
    <div style="display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #374151;padding-bottom:8px;margin-bottom:10px;flex-shrink:0;">
      <div style="display:flex;align-items:baseline;gap:12px;">
        <span class="font-title" style="font-size:22px;color:var(--gold);">${STATE.slotsSaving ? 'Save Chronicle' : 'Load Chronicle'}</span>
        <span style="font-size:12px;color:var(--dim);">1st Click: Select Slot (for Lock / Delete) · 2nd Click or Action Button: ${STATE.slotsSaving ? 'Save / Overwrite' : 'Load'}</span>
      </div>
      ${closeBtn720("STATE.screen='main_menu';render();", "Close [Esc / Right-Click]")}
    </div>

    <!-- Fixed 4-Slot Grid (1 Autosave + 3 Manual Slots) with overflow:hidden (NO scrollbar) -->
    <div style="flex:1;overflow:hidden;display:grid;grid-template-rows:repeat(4, minmax(0, 1fr));gap:10px;">
      ${STATE.slots.map((s, idx) => {
        const sel = STATE.slotSelected === idx;
        const canExecute = STATE.slotsSaving ? (s.kind !== 'auto' && !s.locked) : !s.empty;
        const cardActText = STATE.slotsSaving
          ? (s.kind === 'auto' ? 'Autosave Only' : (s.locked ? 'Locked' : (s.empty ? 'Save Here' : 'Overwrite')))
          : (s.empty ? 'Empty' : 'Load');

        if (!isRich) {
          return `
            <div class="rust-card ${sel ? 'selected' : ''}"
                 style="padding:10px 18px;display:flex;align-items:center;justify-content:space-between;cursor:pointer;opacity:${(!sel && !canExecute) ? '0.68' : '1'};"
                 onclick="activateSlot(${idx})">
              <div>
                <div style="font-size:15.5px;color:var(--text);font-weight:bold;">
                  ${s.title} ${s.locked ? '<span style="color:var(--gold);font-size:12px;">[LOCKED]</span>' : ''}
                </div>
                <div style="font-size:13px;color:var(--dim);margin-top:3px;">
                  ${s.empty ? 'Empty slot' : `${s.place} · Day ${s.day} · ${s.when}`}
                </div>
              </div>
              <div style="display:flex;gap:8px;align-items:center;">
                ${sel ? `
                  <button class="rust-btn-720 primary" ${!canExecute ? 'disabled' : ''} onclick="executeSlot(${idx}, event)">
                    ${cardActText} [Enter]
                  </button>
                ` : `
                  <span class="font-mono" style="font-size:11.5px;color:var(--dim);margin-right:6px;">Click to Select</span>
                `}
                ${(!s.empty && s.kind !== 'auto') ? `
                  <button class="rust-btn-720" style="width:30px;height:28px;padding:0;opacity:${sel ? '1' : '0.45'};" title="${s.locked ? 'Unlock Slot [L]' : 'Lock Slot [L]'}" onclick="STATE.slotSelected=${idx};toggleSlotLock(${idx}, event)">
                    <img src="${s.locked ? ASSETS.ui.lock : ASSETS.ui.unlock}" class="pixel" style="width:18px;height:18px;">
                  </button>
                ` : ''}
                ${!s.empty ? `
                  <button class="rust-btn-720" style="width:30px;height:28px;padding:0;opacity:${(sel && !s.locked) ? '1' : '0.35'};" title="${s.locked ? 'Unlock before deleting' : 'Delete Slot [Del]'}" onclick="STATE.slotSelected=${idx};promptDeleteSlot(${idx}, event)">
                    <img src="${ASSETS.ui.delete}" class="pixel" style="width:18px;height:18px;">
                  </button>
                ` : ''}
              </div>
            </div>
          `;
        }

        const thumbSrc = s.thumb === 'corvos_gate' ? ASSETS.bg.alley : (s.thumb === 'story_alley' ? ASSETS.bg.intro_1 : ASSETS.bg.hearth);
        const fixedSlotParty = [0, 1, 2, 3].map(i => (!s.empty && s.party[i]) ? STATE.roster[s.party[i]] : null);

        return `
          <div class="rust-card ${sel ? 'selected' : ''}"
               style="padding:10px 14px;display:flex;align-items:stretch;gap:16px;cursor:pointer;opacity:${(!sel && !canExecute) ? '0.72' : '1'};position:relative;overflow:hidden;"
               onclick="activateSlot(${idx})">
            <!-- Left 16:9 Location Thumbnail (184x98) -->
            <div style="width:184px;height:100%;border:1px solid ${sel ? 'var(--gold)' : '#374151'};background:#07080c;flex-shrink:0;position:relative;overflow:hidden;display:flex;align-items:center;justify-content:center;">
              ${s.empty ? `
                <span class="font-mono" style="font-size:12px;color:var(--dim);">— Empty Slot —</span>
              ` : `
                <img src="${thumbSrc}" class="pixel" style="width:100%;height:100%;object-fit:cover;">
                <div style="position:absolute;inset:0;background:linear-gradient(180deg, rgba(0,0,0,0) 48%, rgba(10,11,16,0.88) 100%);"></div>
                <span class="font-mono" style="position:absolute;left:6px;bottom:4px;font-size:11px;color:#fef3c7;">Day ${s.day} · Tithe ${s.tithe}d</span>
              `}
            </div>

            <!-- Middle Metadata Column (Title, Place/Mission, 4 Equal-Width Party Blocks) -->
            <div style="flex:1;display:flex;flex-direction:column;justify-content:space-between;min-width:0;">
              <div style="display:flex;align-items:center;gap:10px;">
                <span class="font-title" style="font-size:17px;color:${sel ? 'var(--gold)' : (s.empty ? 'var(--dim)' : 'var(--text)')};">${s.title}</span>
                ${s.kind === 'auto' ? `<span class="font-mono" style="font-size:11px;background:#1e3a8a;color:#93c5fd;padding:1px 6px;border:1px solid #3b82f6;">AUTO</span>` : ''}
                ${s.locked ? `<span class="font-mono" style="font-size:11px;background:rgba(217,119,6,0.25);color:var(--gold);padding:1px 6px;border:1px solid var(--gold);">LOCKED</span>` : ''}
                ${sel ? `<span class="font-mono" style="font-size:10.5px;background:rgba(180,83,9,0.28);color:#fde68a;padding:1px 6px;border:1px solid #d97706;">SELECTED</span>` : ''}
              </div>

              <div style="font-size:13px;color:var(--dim);display:flex;align-items:center;gap:10px;">
                <span style="color:var(--text);font-weight:bold;">${s.place}</span>
                ${!s.empty ? `<span>· <b style="color:#93c5fd;">${s.lead}</b></span>` : ''}
              </div>

              <!-- Bottom row: 4 Fixed Equal-Width Party Member Pills -->
              <div style="display:flex;align-items:center;gap:8px;">
                ${!s.empty ? `
                  <div style="display:grid;grid-template-columns:repeat(4, 108px);gap:6px;">
                    ${fixedSlotParty.map(h => h ? `
                      <div style="width:108px;box-sizing:border-box;display:flex;align-items:center;gap:6px;background:rgba(10,11,16,0.8);border:1px solid #374151;padding:2px 5px;border-radius:2px;">
                        <div class="portrait-box gold-rim" style="width:32px;height:32px;flex-shrink:0;">
                          <img src="${ASSETS.portraits[h.id]}">
                        </div>
                        <span style="font-size:11.5px;color:var(--text);font-weight:bold;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${h.name.split(' ')[0]}</span>
                      </div>
                    ` : '').join('')}
                  </div>
                  <span class="font-mono" style="font-size:12px;color:var(--gold);margin-left:6px;">${s.scrap} Scrap · ${s.tallow} Tallow</span>
                ` : `<span style="font-size:12.5px;color:var(--dim);">${STATE.slotsSaving ? 'Select slot, then click Save to record chronicle here.' : 'Empty chronicle slot.'}</span>`}
              </div>
            </div>

            <!-- Right Controls Column: Lock & Delete (active when selected) + Step 2 Action / Timestamp -->
            <div style="display:flex;flex-direction:column;justify-content:space-between;align-items:flex-end;flex-shrink:0;min-width:146px;">
              <div style="display:flex;align-items:center;gap:6px;">
                <span class="font-mono" style="font-size:11.5px;color:var(--dim);margin-right:4px;">${s.when}</span>
                ${(!s.empty && s.kind !== 'auto') ? `
                  <button class="rust-btn-720" style="width:30px;height:28px;padding:0;opacity:${sel ? '1' : '0.45'};border-color:${sel ? '#6b7280' : '#374151'};" title="${s.locked ? 'Unlock Slot [L]' : 'Lock Slot [L]'}" onclick="STATE.slotSelected=${idx};toggleSlotLock(${idx}, event)">
                    <img src="${s.locked ? ASSETS.ui.lock : ASSETS.ui.unlock}" class="pixel" style="width:18px;height:18px;">
                  </button>
                ` : ''}
                ${!s.empty ? `
                  <button class="rust-btn-720" style="width:30px;height:28px;padding:0;opacity:${(sel && !s.locked) ? '1' : '0.35'};border-color:${(sel && !s.locked) ? '#6b7280' : '#374151'};" title="${s.locked ? 'Unlock before deleting' : 'Delete Slot [Del]'}" onclick="STATE.slotSelected=${idx};promptDeleteSlot(${idx}, event)">
                    <img src="${ASSETS.ui.delete}" class="pixel" style="width:18px;height:18px;">
                  </button>
                ` : ''}
              </div>

              <div style="display:flex;align-items:center;gap:6px;">
                ${sel ? `
                  <button class="rust-btn-720 primary" style="height:28px;padding:0 12px;font-size:12.5px;" ${!canExecute ? 'disabled' : ''} onclick="executeSlot(${idx}, event)">
                    ${cardActText} ▸
                  </button>
                ` : `
                  <span class="font-mono" style="font-size:11px;color:var(--dim);">Click to Select</span>
                `}
              </div>
            </div>
          </div>
        `;
      }).join('')}
    </div>

    <!-- Bottom Action Bar (2-Click Best Practice: Shows Status + Explicit Action Buttons for Selected Slot) -->
    <div style="display:flex;align-items:center;justify-content:space-between;border-top:1px solid #374151;padding-top:8px;margin-top:8px;font-size:12px;color:var(--dim);flex-shrink:0;">
      <span>${STATE.slotToast}</span>
      <div style="display:flex;align-items:center;gap:8px;">
        ${(selSlot && !selSlot.empty && selSlot.kind !== 'auto') ? `
          <button class="rust-btn-720" onclick="toggleSlotLock(${STATE.slotSelected})">
            ${selSlot.locked ? 'Unlock [L]' : 'Lock [L]'}
          </button>
        ` : ''}
        ${(selSlot && !selSlot.empty) ? `
          <button class="rust-btn-720" ${selSlot.locked ? 'disabled' : ''} onclick="promptDeleteSlot(${STATE.slotSelected})">
            Delete [Del]
          </button>
        ` : ''}
        <button class="rust-btn-720 primary" ${!selCanAct ? 'disabled' : ''} onclick="executeSlot(${STATE.slotSelected})">
          ${primaryActLabel}
        </button>
      </div>
    </div>
  `;

  return `
    <div style="width:1280px;height:720px;position:relative;overflow:hidden;">
      <img src="${ASSETS.bg.title}" class="pixel" style="position:absolute;inset:0;width:1280px;height:720px;object-fit:cover;filter:brightness(0.35) blur(1px);">
      <div style="position:absolute;inset:0;background:radial-gradient(circle at 50% 50%, rgba(10,11,16,0.45) 0%, rgba(10,11,16,0.88) 100%);"></div>

      ${ornateBox(innerContent, 'position:absolute;left:64px;top:36px;width:1152px;height:648px;padding:18px 24px;z-index:5;')}

      <!-- Confirmation Dialog when Delete is clicked (Small Popup -> Reduced Corner-Only Frame) -->
      ${delSlot ? `
        <div class="modal-backdrop" style="z-index:30;">
          ${ornateCornerBox(`
            <div style="display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #374151;padding-bottom:6px;">
              <span class="font-title" style="font-size:19px;color:var(--gold);">Delete Chronicle?</span>
              ${closeBtn720("STATE.slotAskDelete=null;render();", "Cancel [Esc / Right-Click]")}
            </div>
            <div style="font-size:14px;color:var(--text);line-height:1.45;margin:12px 0;">
              Permanently erase <b>${delSlot.title}</b> (${delSlot.place}, Day ${delSlot.day})? This action cannot be undone.
            </div>
            <div style="display:flex;justify-content:flex-end;gap:10px;">
              <button class="rust-btn-720" onclick="STATE.slotAskDelete=null;render();">Cancel</button>
              <button class="rust-btn-720 primary" onclick="confirmDeleteSlot()">Delete Chronicle [Enter]</button>
            </div>
          `, 'width:540px;height:210px;padding:18px 22px;')}
        </div>
      ` : ''}

      <!-- Confirmation Dialog when Overwriting an occupied Manual Slot in Save Mode (Small Popup -> Reduced Corner-Only Frame) -->
      ${ovrSlot ? `
        <div class="modal-backdrop" style="z-index:30;">
          ${ornateCornerBox(`
            <div style="display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #374151;padding-bottom:6px;">
              <span class="font-title" style="font-size:19px;color:var(--gold);">Overwrite Chronicle?</span>
              ${closeBtn720("STATE.slotAskOverwrite=null;render();", "Cancel [Esc / Right-Click]")}
            </div>
            <div style="font-size:14px;color:var(--text);line-height:1.45;margin:12px 0;">
              Replace <b>${ovrSlot.title}</b> (${ovrSlot.place}, Day ${ovrSlot.day}) with your current progress?
            </div>
            <div style="display:flex;justify-content:flex-end;gap:10px;">
              <button class="rust-btn-720" onclick="STATE.slotAskOverwrite=null;render();">Cancel</button>
              <button class="rust-btn-720 primary" onclick="writeSaveToSlot(${STATE.slotAskOverwrite})">Overwrite Chronicle [Enter]</button>
            </div>
          `, 'width:540px;height:210px;padding:18px 22px;')}
        </div>
      ` : ''}
    </div>
  `;
}

