// ============================================================================
// TOOLS / SCENE RUNNER (`runner.rs` — Decision A1: Ashlands + Dev Launcher)
// ============================================================================
function renderSceneRunner() {
  const sunkEncs = [
    { id: 'enc_0a', name: 'enc_pro_0a · The Alley (Kaelen Solo)', sub: '2 foes · Watch Halberdier + Arbalest' },
    { id: 'enc_0b', name: 'enc_pro_0b · The Rooftop (+Vesper)', sub: '3 foes · Bailiff + 2 Slate Marksmen' },
    { id: 'enc_0c', name: 'enc_pro_0c · The Sluice Bridge (+Maren, +Hollis)', sub: '3 foes · Captain Vane Boss Cordon' },
    { id: 'enc_m2', name: 'enc_m2_corvo · Third Ward Toll Gate', sub: '3 foes · Gate-Sergeant Corvo + Purifier' },
    { id: 'enc_m3', name: 'enc_m3_bonemill · The Fourth Kiln (Bone-Mill)', sub: '3 foes · Foreman Grendel + Slag Vents' },
    { id: 'enc_m5', name: 'enc_m5_crucible · Crucible Watchpost (Climax)', sub: '3 foes · Prelate Nemeth + Proctor Quell' }
  ];
  const ashEncs = [
    { id: 'enc_0a', name: 'ashlands/enc_ash_patrol · Ashmaw Patrol (Legacy Sandbox)', sub: 'Moved out of player New Game flow per Decision A1' },
    { id: 'enc_0b', name: 'ashlands/enc_cinder_gate · Cinder Gate Skirmish (Legacy)', sub: 'Dev Combat & AI Regression Sandbox' }
  ];
  const list = STATE.runnerWorld === 'sunk_ward' ? sunkEncs : ashEncs;

  return `
    <div style="width:640px;height:360px;position:relative;background:#0a0b10;padding:18px;">
      ${ornateBox640(`
        <div style="display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #374151;padding-bottom:6px;">
          <div>
            <div class="font-title" style="font-size:12px;color:var(--gold);">Tools · Scene Runner & Dev Sandbox (runner.rs)</div>
            <div style="font-size:7px;color:var(--dim);">Decision A1: Player-facing "New Game" boots Sunk Ward directly; Legacy Ashlands & direct scene jumps live here.</div>
          </div>
          <div style="display:flex;gap:4px;align-items:center;">
            <button class="rust-btn ${STATE.runnerWorld==='sunk_ward'?'primary':''}" onclick="STATE.runnerWorld='sunk_ward';render();">World: The Sunk Ward</button>
            <button class="rust-btn ${STATE.runnerWorld==='ashlands'?'primary':''}" onclick="STATE.runnerWorld='ashlands';render();">World: The Ashlands (Legacy)</button>
            ${closeBtn640("STATE.screen='main_menu';render();", "Close to Main Menu [Esc / Right-Click]")}
          </div>
        </div>

        <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;flex:1;margin:8px 0;overflow:hidden;">
          <div class="rust-inset" style="padding:6px;overflow-y:auto;">
            <div class="font-title" style="font-size:8.5px;color:var(--gold);margin-bottom:4px;">Encounters (${list.length})</div>
            ${list.map((e, i) => `
              <div class="rust-card ${STATE.runnerSel===i?'selected':''}" style="padding:5px 6px;margin-bottom:4px;cursor:pointer;display:flex;justify-content:space-between;align-items:center;"
                   onclick="STATE.runnerSel=${i};launchEncounter('${e.id}')">
                <div>
                  <div style="font-size:7.5px;color:var(--text);font-weight:bold;">${e.name}</div>
                  <div style="font-size:6.5px;color:var(--dim);">${e.sub}</div>
                </div>
                <span class="rust-btn primary" style="height:14px;font-size:6.5px;">Fight ▸</span>
              </div>
            `).join('')}
          </div>

          <div class="rust-inset" style="padding:6px;overflow-y:auto;">
            <div class="font-title" style="font-size:8.5px;color:var(--gold);margin-bottom:4px;">Locations & Story Sequences</div>
            <div class="rust-card" style="padding:5px 6px;margin-bottom:4px;cursor:pointer;display:flex;justify-content:space-between;align-items:center;"
                 onclick="STATE.screen='region';STATE.regionId='sluice_undercroft';render();">
              <div>
                <div style="font-size:7.5px;color:var(--text);font-weight:bold;">Location: sluice_undercroft</div>
                <div style="font-size:6.5px;color:var(--dim);">Hub Sanctuary · Aline, Bram, Tess & 6 Bunks</div>
              </div>
              <span class="rust-btn" style="height:14px;font-size:6.5px;">Enter ▸</span>
            </div>
            <div class="rust-card" style="padding:5px 6px;margin-bottom:4px;cursor:pointer;display:flex;justify-content:space-between;align-items:center;"
                 onclick="STATE.screen='region';STATE.regionId='corvos_gate';render();">
              <div>
                <div style="font-size:7.5px;color:var(--text);font-weight:bold;">Location: corvos_gate (Ash-Crawl Street)</div>
                <div style="font-size:6.5px;color:var(--dim);">Decision A4 Parley RegionMap · Sergeant Corvo</div>
              </div>
              <span class="rust-btn" style="height:14px;font-size:6.5px;">Enter ▸</span>
            </div>
            <div class="rust-card" style="padding:5px 6px;margin-bottom:4px;cursor:pointer;display:flex;justify-content:space-between;align-items:center;"
                 onclick="STATE.screen='story';STATE.storyId='prologue_intro';STATE.storyCardIdx=0;render();">
              <div>
                <div style="font-size:7.5px;color:var(--text);font-weight:bold;">Sequence: new_game (0a → 0b → 0c → Hearth)</div>
                <div style="font-size:6.5px;color:var(--dim);">Decision A2 Chained Prologue Vignette + Battles</div>
              </div>
              <span class="rust-btn" style="height:14px;font-size:6.5px;">Play ▸</span>
            </div>
          </div>
        </div>

        <div style="font-size:7px;color:var(--dim);display:flex;justify-content:space-between;">
          <span>game.json: <code style="color:var(--gold);">"start_worlds": ["sunk_ward"]</code></span>
          <span>Click any row to launch · [Esc / Right-Click] Close</span>
        </div>
      `, 'width:100%;height:100%;padding:10px;display:flex;flex-direction:column;justify-content:space-between;')}
    </div>
  `;
}

// ============================================================================
// SCREEN 2: HYBRID STORY STAGE (`story.rs` + `vignette.rs` — 1280×720 Native Grid)
// Implements the Hybrid Story Architecture:
//  1. `vn_stage`      : Full-Body Sprite Artwork (ART_PROMPT_TEMPLATES.md) standing on a clearly visible
//                       stone/cobblestone floor plane + Reaction VFX & animations around the sprite +
//                       Portrait Mood Expression (`faces/<id>/<mood>.png`) in the bottom dialogue box.
//  2. `map_stage`     : In-Game Region/Battle Map Stage + On-Tile Unit Reactions + Seamless Transition to Combat UI.
//  3. `vignette_reel` : Dedicated Cutscene / Transition Player (`vignette.rs`) with minimal UI (`Auto`, `Log`, `Skip`).
// ============================================================================
function renderReactionFxOverlay(fxKey, isActive, side) {
  if (!fxKey) return { back: '', front: '' };
  const slashFlip = side === 'right' ? 'scaleX(-1)' : 'scaleX(1)';

  if (fxKey === 'steel_flash') {
    return {
      back: `
        <!-- Centered Torso Aura + Floor Impact Ripple behind sprite -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:1;">
          <div style="position:absolute;left:50%;top:54%;width:190px;height:190px;border-radius:50%;border:2px solid rgba(251,191,36,0.78);box-shadow:0 0 22px rgba(245,158,11,0.35);animation:vnPulseRing 1.4s ease-out infinite;"></div>
          <div style="position:absolute;left:50%;bottom:4px;width:210px;height:36px;border-radius:50%;border:2px solid rgba(56,189,248,0.75);animation:vnFloorRipple 1.4s ease-out infinite;"></div>
        </div>
      `,
      front: `
        <!-- Centered Cold-Steel Slash Arc across weapon/torso + Overhead Callout -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:4;">
          <div style="position:absolute;left:50%;top:50%;width:240px;height:240px;animation:vnSlashGlint 1.8s ease-out infinite;">
            <svg width="240" height="240" viewBox="0 0 240 240" style="overflow:visible;transform:${slashFlip};">
              <path d="M 28 196 Q 118 92 214 34" stroke="#fef3c7" stroke-width="5.5" stroke-linecap="round" fill="none" filter="drop-shadow(0 0 8px #f59e0b)"/>
              <path d="M 48 206 Q 130 112 220 54" stroke="#38bdf8" stroke-width="2.5" stroke-linecap="round" fill="none"/>
              <circle cx="146" cy="84" r="4" fill="#fef3c7" filter="drop-shadow(0 0 6px #fbbf24)"/>
              <path d="M 146 72 L 146 96 M 134 84 L 158 84" stroke="#fef3c7" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </div>
          <div class="vn-emote-badge" style="position:absolute;left:50%;top:-18px;padding:2px 10px;border-radius:10px;background:rgba(15,23,42,0.95);border:1.5px solid #fbbf24;color:#fef3c7;font-size:11px;font-weight:800;white-space:nowrap;box-shadow:0 4px 12px rgba(0,0,0,0.9);">
            ⚔ STEEL DRAWN
          </div>
        </div>
      `
    };
  }

  if (fxKey === 'anger_burst') {
    return {
      back: `
        <!-- Centered Crimson Shockwave Rings behind torso + Floor Tremor Ring -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:1;">
          <div style="position:absolute;left:50%;top:54%;width:205px;height:205px;border-radius:50%;border:2.5px solid rgba(239,68,68,0.85);box-shadow:0 0 26px rgba(220,38,38,0.45);animation:vnPulseRing 1.15s ease-out infinite;"></div>
          <div style="position:absolute;left:50%;top:54%;width:162px;height:162px;border-radius:50%;border:1.5px dashed rgba(251,191,36,0.78);transform:translate(-50%,-50%);"></div>
          <div style="position:absolute;left:50%;bottom:4px;width:215px;height:38px;border-radius:50%;border:2px solid rgba(239,68,68,0.82);animation:vnFloorRipple 1.15s ease-out infinite;"></div>
        </div>
      `,
      front: `
        <!-- Centered Crimson Burst Spokes & Overhead Enraged Badge -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:4;">
          <svg width="240" height="240" viewBox="0 0 240 240" style="position:absolute;left:50%;top:52%;width:240px;height:240px;overflow:visible;animation:vnFloatSparks 1.4s ease-out infinite;">
            <path d="M 46 66 L 26 48 M 194 66 L 214 48 M 36 126 L 14 122 M 204 126 L 226 122" stroke="#ef4444" stroke-width="3" stroke-linecap="round"/>
            <circle cx="52" cy="96" r="3.5" fill="#fbbf24"/>
            <circle cx="188" cy="88" r="3.5" fill="#ef4444"/>
            <circle cx="68" cy="48" r="2.5" fill="#fca5a5"/>
            <circle cx="174" cy="52" r="3" fill="#fbbf24"/>
          </svg>
          <div class="vn-emote-badge" style="position:absolute;left:50%;top:-18px;padding:2px 10px;border-radius:10px;background:rgba(69,10,10,0.96);border:1.5px solid #ef4444;color:#fecaca;font-size:11px;font-weight:800;white-space:nowrap;box-shadow:0 4px 12px rgba(0,0,0,0.9);">
            💢 ENRAGED
          </div>
        </div>
      `
    };
  }

  if (fxKey === 'alert_shock') {
    return {
      back: `
        <!-- Centered Cyan Sonar Ring around upper torso -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:1;">
          <div style="position:absolute;left:50%;top:48%;width:185px;height:185px;border-radius:50%;border:2px solid rgba(56,189,248,0.82);box-shadow:0 0 20px rgba(56,189,248,0.32);animation:vnPulseRing 1.35s ease-out infinite;"></div>
          <div style="position:absolute;left:50%;bottom:4px;width:195px;height:34px;border-radius:50%;border:1.5px solid rgba(56,189,248,0.68);animation:vnFloorRipple 1.35s ease-out infinite;"></div>
        </div>
      `,
      front: `
        <!-- Classic Tactical RPG Comic Reaction Rays & Overhead !! ALERT -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:4;">
          <svg width="180" height="70" viewBox="0 0 180 70" style="position:absolute;left:50%;top:-6px;transform:translateX(-50%);overflow:visible;">
            <path d="M 45 42 L 26 24 M 65 30 L 54 8 M 115 30 L 126 8 M 135 42 L 154 24" stroke="#38bdf8" stroke-width="2.5" stroke-linecap="round"/>
          </svg>
          <div class="vn-emote-badge" style="position:absolute;left:50%;top:-20px;padding:2px 11px;border-radius:12px;background:rgba(12,74,110,0.96);border:1.5px solid #38bdf8;color:#e0f2fe;font-size:11.5px;font-weight:900;white-space:nowrap;box-shadow:0 4px 12px rgba(0,0,0,0.9);">
            !! ALERT
          </div>
        </div>
      `
    };
  }

  if (fxKey === 'alchemy_flare') {
    return {
      back: `
        <!-- Centered Alchemical Vapour Ring + Emerald Floor Halo -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:1;">
          <div style="position:absolute;left:50%;top:54%;width:192px;height:192px;border-radius:50%;border:2px solid rgba(52,211,153,0.78);box-shadow:0 0 24px rgba(16,185,129,0.35);animation:vnPulseRing 1.5s ease-out infinite;"></div>
          <div style="position:absolute;left:50%;bottom:4px;width:205px;height:36px;border-radius:50%;border:2px dashed rgba(52,211,153,0.8);animation:vnFloorRipple 1.5s ease-out infinite;"></div>
        </div>
      `,
      front: `
        <!-- Rising Alchemical Motes + Centered Overhead Badge -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:4;">
          <svg width="220" height="220" viewBox="0 0 220 220" style="position:absolute;left:50%;top:54%;width:220px;height:220px;overflow:visible;animation:vnFloatSparks 1.8s ease-out infinite;">
            <circle cx="52" cy="128" r="4" fill="#34d399"/>
            <circle cx="168" cy="114" r="3.5" fill="#fbbf24"/>
            <circle cx="68" cy="76" r="3" fill="#a7f3d0"/>
            <circle cx="154" cy="68" r="4" fill="#34d399"/>
          </svg>
          <div class="vn-emote-badge" style="position:absolute;left:50%;top:-18px;padding:2px 10px;border-radius:10px;background:rgba(6,78,59,0.96);border:1.5px solid #34d399;color:#a7f3d0;font-size:11px;font-weight:800;white-space:nowrap;box-shadow:0 4px 12px rgba(0,0,0,0.9);">
            ⚗ ALCHEMY READY
          </div>
        </div>
      `
    };
  }

  // Default `solemn_glow`
  return {
    back: isActive ? `
      <div style="position:absolute;inset:0;pointer-events:none;z-index:1;">
        <div style="position:absolute;left:50%;top:54%;width:180px;height:180px;border-radius:50%;border:1.5px solid rgba(245,158,11,0.55);animation:vnPulseRing 2.1s ease-out infinite;"></div>
      </div>
    ` : '',
    front: ''
  };
}

function renderStoryActorSlot(actor, side) {
  if (!actor) return `<div style="width:310px;height:345px;pointer-events:none;"></div>`;
  const style = STATE.storyActorStyle || 'full_sprite';
  if (style === 'backdrop_only') return `<div style="width:310px;height:345px;pointer-events:none;"></div>`;

  const isActive = !!actor.active;
  const activeFx = isActive && STATE.storyReactionOverride ? STATE.storyReactionOverride : actor.reactionFx;
  const animType = isActive && STATE.storyReactionOverride
    ? (STATE.storyReactionOverride === 'steel_flash' ? 'lunge' : STATE.storyReactionOverride === 'anger_burst' ? 'shake' : 'hop')
    : (actor.anim || 'breathe');

  let animClass = 'vn-anim-breathe';
  if (isActive) {
    if (animType === 'lunge') animClass = side === 'left' ? 'vn-anim-lunge-right' : 'vn-anim-lunge-left';
    else if (animType === 'shake') animClass = 'vn-anim-shake';
    else if (animType === 'hop') animClass = 'vn-anim-hop';
  }

  const filterStr = isActive
    ? 'brightness(1.06) contrast(1.06) drop-shadow(0 0 14px rgba(245,158,11,0.45)) drop-shadow(0 10px 18px rgba(0,0,0,0.92))'
    : 'brightness(0.58) saturate(0.75) contrast(0.96) drop-shadow(0 8px 14px rgba(0,0,0,0.88))';
  const borderCol = isActive ? '#f59e0b' : '#4b5563';

  if (style === 'bust_cutout') {
    const portraitSrc = getPortraitWithMood(actor.portrait, isActive ? (STATE.storyMoodOverride || 'determined') : 'neutral');
    return `
      <div class="${animClass}" style="display:flex;flex-direction:column;align-items:center;transition:all 0.22s ease;">
        <div style="width:220px;height:220px;border-radius:10px;overflow:hidden;border:2px solid ${borderCol};background:radial-gradient(circle at 50% 35%, rgba(245,158,11,0.22), rgba(10,12,18,0.95));filter:${filterStr};position:relative;">
          <img src="${portraitSrc}" class="pixel" style="width:100%;height:100%;object-fit:cover;">
        </div>
        <div style="margin-top:8px;padding:4px 14px;background:rgba(10,12,18,0.92);border:1px solid ${borderCol};border-radius:4px;text-align:center;">
          <div class="font-title" style="font-size:13px;color:${isActive ? 'var(--gold)' : '#9ca3af'};">${actor.name}</div>
          <div style="font-size:10px;color:var(--dim);">${actor.title || ''}</div>
        </div>
      </div>
    `;
  }

  // Default: `full_sprite` — Full-Body Pixel Art Character Sprite (`ASSETS.fullBody[unitKey]`)
  // standing directly on the visible stone floor with grounded contact shadow & aligned reaction VFX around the sprite
  const fullBodySrc = (ASSETS.fullBody && ASSETS.fullBody[actor.unitKey])
    ? ASSETS.fullBody[actor.unitKey]
    : ((ASSETS.units[actor.unitKey] || ASSETS.units.kaelen).idle0);
  const nativeFacesLeft = (actor.unitKey === 'vane' || actor.unitKey === 'watchman' || actor.unitKey === 'bailiff');
  const shouldFlip = (side === 'right' && !nativeFacesLeft) || (side === 'left' && nativeFacesLeft);
  const flipX = shouldFlip ? 'scaleX(-1)' : 'scaleX(1)';
  const fxLayers = renderReactionFxOverlay(activeFx, isActive, side);

  return `
    <div style="display:flex;flex-direction:column;align-items:center;position:relative;width:310px;height:345px;justify-content:flex-end;">
      <div class="${animClass}" style="position:relative;width:288px;height:288px;display:flex;align-items:flex-end;justify-content:center;">
        <!-- Ground Contact Shadow + Tallow Floor Ring directly on the visible stone floor -->
        <div style="position:absolute;left:50%;bottom:2px;transform:translateX(-50%);width:195px;height:34px;border-radius:50%;background:${isActive ? 'radial-gradient(ellipse, rgba(0,0,0,0.92) 0%, rgba(245,158,11,0.34) 58%, transparent 82%)' : 'radial-gradient(ellipse, rgba(0,0,0,0.88) 0%, rgba(0,0,0,0.45) 60%, transparent 82%)'};border-bottom:${isActive ? '2px solid rgba(251,191,36,0.78)' : '1px solid rgba(148,163,184,0.28)'};z-index:0;"></div>

        ${fxLayers.back}

        <!-- Full-Body Pixel Art Sprite Artwork (340×340 native, feet anchored at bottom) -->
        <img src="${fullBodySrc}" class="pixel" style="width:288px;height:288px;object-fit:contain;object-position:center bottom;transform:${flipX};filter:${filterStr};position:relative;z-index:2;">

        ${fxLayers.front}
      </div>

      <!-- Compact Grounded Nameplate below boots -->
      <div style="margin-top:4px;padding:3px 14px;background:rgba(10,12,18,0.92);border:1px solid ${borderCol};border-radius:4px;text-align:center;box-shadow:0 4px 12px rgba(0,0,0,0.85);z-index:5;">
        <span class="font-title" style="font-size:12.5px;color:${isActive ? 'var(--gold)' : '#9ca3af'};">${actor.name}</span>
        <span style="font-size:10px;color:var(--dim);margin-left:6px;">· ${actor.emotion || actor.title || ''}</span>
      </div>
    </div>
  `;
}

function renderStoryMapBackdrop(st, card) {
  const enc = ENCOUNTERS[st.encId] || ENCOUNTERS.enc_0a;
  const horizonStripImg = st.mapBackdrop === 'sluice_hub'
    ? ASSETS.bg.hearth
    : (ASSETS.bg[st.mapBackdrop] || ASSETS.bg.alley);
  const ambientBgImg = ASSETS.bg[st.bg] || ASSETS.bg.vn_alley_stage || ASSETS.bg.intro_1;

  const cols = 12;
  const rows = 9;
  const tileW = 76;
  const tileH = 40;
  const boardW = cols * tileW; // 912
  const boardH = rows * tileH; // 360
  const boardLeft = Math.round((1280 - boardW) / 2); // 184
  const boardTop = 122;

  let tilesHtml = '';
  for (let y = 0; y < rows; y++) {
    for (let x = 0; x < cols; x++) {
      const isWall = st.mapBackdrop !== 'sluice_hub' && enc.walls.some(w => w[0] === x && w[1] === y);
      const isWater = st.mapBackdrop !== 'sluice_hub' && enc.water.some(w => w[0] === x && w[1] === y);
      const haz = st.mapBackdrop !== 'sluice_hub' ? enc.hazards.find(h => h.x === x && h.y === y) : null;
      const unitOnTile = (card.mapUnits || []).find(u => u.x === x && u.y === y);

      let bg = ((x + y) % 2 === 0) ? 'rgba(22, 26, 36, 0.82)' : 'rgba(18, 21, 30, 0.82)';
      let bdr = 'rgba(148, 163, 184, 0.16)';
      if (isWall) { bg = 'rgba(12, 14, 20, 0.94)'; bdr = 'rgba(75, 85, 99, 0.55)'; }
      else if (isWater) { bg = 'rgba(12, 74, 110, 0.55)'; bdr = 'rgba(56, 189, 248, 0.38)'; }
      else if (haz) { bg = 'rgba(180, 83, 9, 0.42)'; bdr = 'rgba(251, 191, 36, 0.7)'; }

      if (unitOnTile && unitOnTile.speaking) {
        bg = 'rgba(245, 158, 11, 0.32)';
        bdr = '#fbbf24';
      }

      tilesHtml += `
        <div style="position:absolute;left:${x * tileW}px;top:${y * tileH}px;width:${tileW}px;height:${tileH}px;background:${bg};border:1px solid ${bdr};box-sizing:border-box;">
          ${isWall ? `<div style="position:absolute;inset:3px;background:#1f2430;border-top:2px solid #374151;"></div>` : ''}
          ${haz ? `<div style="position:absolute;inset:0;display:flex;align-items:center;justify-content:center;font-size:10px;color:#fbbf24;font-weight:bold;text-shadow:0 1px 3px #000;">🔥 ${haz.type.toUpperCase()}</div>` : ''}
        </div>
      `;
    }
  }

  const unitsHtml = (card.mapUnits || []).map(u => {
    const left = u.x * tileW + Math.round(tileW / 2);
    const top = u.y * tileH + Math.round(tileH / 2);
    const isSpeaking = !!u.speaking;
    const ringCol = isSpeaking ? '#fbbf24' : (u.isHero ? '#38bdf8' : '#ef4444');
    const uFrames = ASSETS.units[u.unitKey] || ASSETS.units.kaelen;
    const unitImg = isSpeaking ? (uFrames.atk || uFrames.idle0) : uFrames.idle0;
    const flip = u.isHero ? 'scaleX(1)' : 'scaleX(-1)';
    return `
      <div style="position:absolute;left:${left - 40}px;top:${top - 56}px;width:80px;height:80px;pointer-events:none;z-index:${10 + u.y};display:flex;flex-direction:column;align-items:center;">
        ${u.emote ? `
          <div style="margin-bottom:2px;padding:2px 8px;border-radius:10px;background:${isSpeaking ? '#fbbf24' : 'rgba(15,18,26,0.92)'};color:${isSpeaking ? '#090b10' : '#f3f4f6'};border:1px solid ${ringCol};font-size:11px;font-weight:800;box-shadow:0 3px 8px rgba(0,0,0,0.85);white-space:nowrap;">
            ${u.emote} ${isSpeaking ? u.name : ''}
          </div>
        ` : ''}
        <div style="position:relative;width:64px;height:64px;display:flex;align-items:center;justify-content:center;">
          <!-- Tactical tile ring -->
          <div style="position:absolute;bottom:4px;width:54px;height:20px;border-radius:50%;border:2px solid ${ringCol};background:${isSpeaking ? 'rgba(251,191,36,0.32)' : 'rgba(0,0,0,0.55)'};box-shadow:${isSpeaking ? '0 0 14px rgba(251,191,36,0.8)' : 'none'};"></div>
          <img src="${unitImg}" class="pixel" style="width:60px;height:60px;object-fit:contain;transform:${flip} ${isSpeaking ? 'scale(1.14) translateY(-4px)' : 'scale(1)'};filter:${isSpeaking ? 'drop-shadow(0 0 10px rgba(251,191,36,0.75))' : 'brightness(0.88)'};position:relative;z-index:2;">
        </div>
      </div>
    `;
  }).join('');

  return `
    <div style="position:absolute;inset:0;background:#080a0f;overflow:hidden;">
      <img src="${ambientBgImg}" class="pixel" style="position:absolute;inset:0;width:1280px;height:720px;object-fit:cover;filter:brightness(0.32) blur(2px);">
      <div style="position:absolute;inset:0;background:radial-gradient(circle at 50% 42%, transparent 35%, rgba(6,8,13,0.82) 92%), linear-gradient(180deg, rgba(6,8,13,0.82) 0%, transparent 16%, transparent 66%, rgba(6,8,13,0.95) 100%);"></div>

      <!-- Top In-Engine Stage Direction / Camera Focus Strip -->
      <div style="position:absolute;left:72px;right:72px;top:14px;height:36px;padding:0 16px;background:rgba(12,15,23,0.9);border:1px solid rgba(212,166,74,0.45);border-radius:4px;display:flex;align-items:center;justify-content:space-between;z-index:15;">
        <div style="display:flex;align-items:center;gap:10px;">
          <span style="padding:2px 8px;background:rgba(245,158,11,0.18);border:1px solid #f59e0b;border-radius:3px;font-size:10px;color:#fbbf24;font-weight:700;letter-spacing:0.5px;">IN-ENGINE MAP STAGE</span>
          <span class="font-title" style="font-size:13px;color:var(--gold);">${st.title}</span>
          <span style="font-size:12px;color:var(--dim);">· ${st.subtitle}</span>
        </div>
        <div class="font-mono" style="font-size:11px;color:#93c5fd;">
          🎬 ${card.stageAction}
        </div>
      </div>

      <!-- Horizon Backdrop Banner above the 12×9 Tactical Grid -->
      <div style="position:absolute;left:${boardLeft}px;top:58px;width:${boardW}px;height:62px;border:1px solid rgba(212,166,74,0.38);border-bottom:none;overflow:hidden;">
        <img src="${horizonStripImg}" class="pixel" style="width:100%;height:100%;object-fit:cover;filter:brightness(0.85);">
        <div style="position:absolute;inset:0;background:linear-gradient(180deg, rgba(8,10,15,0.25), rgba(8,10,15,0.75));"></div>
      </div>

      <!-- 12×9 Live Tactical / Region Map Stage -->
      <div style="position:absolute;left:${boardLeft}px;top:${boardTop}px;width:${boardW}px;height:${boardH}px;border:2px solid rgba(212,166,74,0.38);box-shadow:0 16px 40px rgba(0,0,0,0.85);background:rgba(10,12,18,0.65);">
        ${tilesHtml}
        ${unitsHtml}
      </div>
    </div>
  `;
}

// Minimal Cutscene / Transition Player (`aegis_ui::vignette::VignetteScreen`)
// Used purely for act openings, transitions, and CG cutscenes with minimal UI: Auto [A], Log [L], and Skip [✕]
function renderStoryVignetteReel(st, card, isLastCard) {
  const bgKey = st.vignetteBg || st.bg || 'intro_1';
  const bgSrc = ASSETS.bg[bgKey] || ASSETS.bg.intro_1;
  const cardIdx = Math.min(STATE.storyCardIdx, st.cards.length - 1);

  const logOverlayHtml = STATE.vignetteLogOpen ? `
    <div style="position:absolute;inset:0;background:rgba(5,6,10,0.86);z-index:60;display:flex;align-items:center;justify-content:center;" onclick="STATE.vignetteLogOpen=false;render();">
      <div onclick="event.stopPropagation();" style="width:760px;max-height:480px;background:rgba(12,15,22,0.97);border:1px solid rgba(212,166,74,0.55);border-radius:6px;padding:18px 24px;display:flex;flex-direction:column;gap:12px;box-shadow:0 20px 50px rgba(0,0,0,0.95);">
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #2d333f;padding-bottom:8px;">
          <div>
            <span class="font-title" style="font-size:17px;color:var(--gold);">Cutscene Backlog · ${st.title}</span>
            <span class="font-mono" style="font-size:11px;color:var(--dim);margin-left:10px;">(${st.cards.length} Cards)</span>
          </div>
          ${closeBtn720("STATE.vignetteLogOpen=false;render();", "Close Log [L / Esc / Right-Click]")}
        </div>
        <div style="overflow-y:auto;display:flex;flex-direction:column;gap:10px;padding-right:6px;">
          ${st.cards.map((c, idx) => `
            <div onclick="STATE.storyCardIdx=${idx};STATE.vignetteLogOpen=false;render();"
                 style="padding:10px 14px;border-radius:4px;background:${idx === cardIdx ? 'rgba(245,158,11,0.14)' : 'rgba(18,22,31,0.8)'};border:1px solid ${idx === cardIdx ? '#f59e0b' : '#2d333f'};cursor:pointer;">
              <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:4px;">
                <span class="font-title" style="font-size:13.5px;color:var(--gold);">${idx + 1}. ${c.speaker} <span style="font-size:11px;color:var(--dim);font-family:'Inter',sans-serif;">(${c.emotion || c.mood || 'Narration'})</span></span>
                <span class="font-mono" style="font-size:10px;color:#9ca3af;">${idx === cardIdx ? '● Current Card' : 'Jump to Card ▸'}</span>
              </div>
              <div style="font-size:13px;color:var(--text);line-height:1.45;">${c.text}</div>
            </div>
          `).join('')}
        </div>
      </div>
    </div>
  ` : '';

  return `
    <div style="position:absolute;inset:0;background:#05060a;overflow:hidden;"
         onclick="${!isLastCard ? 'STATE.storyCardIdx++;render();' : 'STORIES[STATE.storyId].onFinish();'}">
      <!-- Full-bleed Letterboxed Cutscene Canvas (y: 52..612) -->
      <div style="position:absolute;left:0;right:0;top:52px;bottom:108px;overflow:hidden;border-top:1px solid rgba(212,166,74,0.35);border-bottom:1px solid rgba(212,166,74,0.35);">
        <img src="${bgSrc}" class="pixel" style="width:100%;height:100%;object-fit:cover;filter:${card.bgFilter || 'brightness(0.92)'};">
        <div style="position:absolute;inset:0;background:radial-gradient(circle at 50% 45%, transparent 45%, rgba(5,6,10,0.78) 96%), linear-gradient(180deg, rgba(5,6,10,0.45) 0%, transparent 22%, transparent 68%, rgba(5,6,10,0.92) 100%);"></div>

        <!-- Centered Act / Chapter Title Overlay on Title Cards -->
        <div style="position:absolute;left:140px;right:140px;top:28px;text-align:center;pointer-events:none;">
          <div class="font-mono" style="font-size:11px;color:#f59e0b;letter-spacing:3px;text-transform:uppercase;margin-bottom:4px;">${st.actBanner || 'CHRONICLES OF THE SUNK WARD'}</div>
          <div class="font-title" style="font-size:28px;color:#fef3c7;letter-spacing:1.2px;text-shadow:0 4px 18px rgba(0,0,0,0.95);">${st.title}</div>
          <div style="width:180px;height:1px;background:linear-gradient(90deg, transparent, #d4a64a, transparent);margin:8px auto 4px;"></div>
          <div style="font-size:12px;color:#d1d5db;font-style:italic;">${st.subtitle}</div>
        </div>
      </div>

      <!-- Minimal Top-Right Cutscene Controls Bar: Auto [A] · Log [L] · Skip [✕] -->
      <div style="position:absolute;left:32px;right:32px;top:10px;height:32px;display:flex;align-items:center;justify-content:space-between;z-index:20;" onclick="event.stopPropagation();">
        <div class="font-mono" style="font-size:11px;color:rgba(156,163,175,0.75);display:flex;align-items:center;gap:8px;">
          <span>VIGNETTE CUTSCENE PLAYER</span>
          <span>·</span>
          <span>Card ${cardIdx + 1} / ${st.cards.length}</span>
          ${STATE.vignetteAuto ? `<span style="color:#34d399;font-weight:700;">● AUTO-PLAYING (3.5s)</span>` : ''}
        </div>
        <div style="display:flex;align-items:center;gap:8px;">
          <button class="rust-btn ${STATE.vignetteAuto ? 'primary' : ''}" style="height:26px;padding:0 12px;font-size:11.5px;background:rgba(15,18,26,0.78);"
                  onclick="STATE.vignetteAuto=!STATE.vignetteAuto;render();" title="Toggle Auto-Advance Cutscene">
            ${STATE.vignetteAuto ? '⏸ Auto: ON [A]' : '▶ Auto [A]'}
          </button>
          <button class="rust-btn ${STATE.vignetteLogOpen ? 'primary' : ''}" style="height:26px;padding:0 12px;font-size:11.5px;background:rgba(15,18,26,0.78);"
                  onclick="STATE.vignetteLogOpen=!STATE.vignetteLogOpen;render();" title="Open Cutscene Transcript Log">
            ☰ Log [L]
          </button>
          <button class="rust-btn" style="height:26px;padding:0 12px;font-size:11.5px;background:rgba(15,18,26,0.78);"
                  onclick="STATE.storySkipConfirmOpen=true;render();" title="Skip Cutscene [Esc / Right-Click]">
            Skip ⏭
          </button>
          ${closeBtn720("STATE.storySkipConfirmOpen=true;render();", "Skip Cutscene [Esc / Right-Click]")}
        </div>
      </div>

      <!-- Minimal Bottom Cinematic Subtitle Bar (Click anywhere or press Space to advance) -->
      <div style="position:absolute;left:140px;right:140px;bottom:14px;height:82px;display:flex; flex-direction:column;align-items:center;justify-content:center;text-align:center;padding:8px 28px;z-index:15;" onclick="event.stopPropagation();">
        <div style="font-size:12px;color:var(--gold);font-family:'Cinzel',serif;letter-spacing:0.8px;margin-bottom:4px;">
          ${card.speaker} <span style="color:var(--dim);font-family:'Inter',sans-serif;font-size:11px;">· ${card.stageAction}</span>
        </div>
        <div class="font-body" style="font-size:15.5px;line-height:1.45;color:#f3f4f6;max-width:880px;text-shadow:0 2px 6px #000;">
          ${card.text}
        </div>
        <div style="margin-top:6px;display:flex;align-items:center;gap:12px;font-family:'JetBrainsMono',monospace;font-size:10.5px;color:#9ca3af;">
          ${cardIdx > 0 ? `<span style="cursor:pointer;color:#d1d5db;" onclick="STATE.storyCardIdx--;render();">◀ Prev</span>` : ''}
          <span>Click or [Space] to ${!isLastCard ? 'Continue ▸' : st.nextLabel + ' ▸'}</span>
        </div>
      </div>

      ${logOverlayHtml}
    </div>
  `;
}

function renderStoryStage() {
  const st = STORIES[STATE.storyId] || STORIES.prologue_intro;
  const cardIdx = Math.min(STATE.storyCardIdx, st.cards.length - 1);
  const card = st.cards[cardIdx];
  const isLastCard = cardIdx >= st.cards.length - 1;
  const mode = STATE.storyMode || 'vn_stage';
  const activeMood = STATE.storyMoodOverride || card.mood || 'neutral';
  const portraitSrc = getPortraitWithMood(card.portrait, activeMood);
  const availMoods = ['neutral', 'determined', 'angry', 'hurt', 'surprised', 'afraid'];

  // Skip confirmation popup (confirm_skip in aegis_ui/src/vignette.rs:171-208)
  const skipConfirmHtml = STATE.storySkipConfirmOpen ? `
    <div style="position:absolute;inset:0;background:rgba(6,8,13,0.78);z-index:80;display:flex;align-items:center;justify-content:center;">
      ${ornateCornerBox(`
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #2d333f;padding-bottom:8px;">
          <div class="font-title" style="font-size:18px;color:var(--gold);">Skip This Story Scene?</div>
          ${closeBtn720("STATE.storySkipConfirmOpen=false;render();", "Cancel [Esc / Right-Click]")}
        </div>
        <div style="font-size:13.5px;color:var(--text);line-height:1.5;margin:12px 0;">
          Skip the remaining cards in <b>${st.title}</b> and proceed directly to:<br>
          <span style="color:#fbbf24;font-weight:700;">${st.nextLabel}</span>
        </div>
        <div style="display:flex;justify-content:flex-end;gap:10px;border-top:1px solid #2d333f;padding-top:10px;">
          <button class="rust-btn" style="height:32px;padding:0 16px;font-size:12.5px;" onclick="STATE.storySkipConfirmOpen=false;render();">Cancel [Esc]</button>
          <button class="rust-btn primary" style="height:32px;padding:0 18px;font-size:12.5px;" onclick="STATE.storySkipConfirmOpen=false;STORIES[STATE.storyId].onFinish();">Skip Scene [Enter] ▸</button>
        </div>
      `, 'width:460px;min-height:185px;padding:18px 22px;background:rgba(14,17,24,0.98);box-shadow:0 20px 50px rgba(0,0,0,0.92);')}
    </div>
  ` : '';

  if (mode === 'vignette_reel') {
    return `
      <div style="width:1280px;height:720px;position:relative;overflow:hidden;background:#05060a;">
        ${renderStoryVignetteReel(st, card, isLastCard)}
        ${skipConfirmHtml}
      </div>
    `;
  }

  return `
    <div style="width:1280px;height:720px;position:relative;overflow:hidden;background:#080a0f;">
      ${mode === 'map_stage' ? renderStoryMapBackdrop(st, card) : `
        <!-- APPROACH 1: VN ILLUSTRATED STAGE WITH CLEARLY VISIBLE BACKDROP + STONE FLOOR + FULL-BODY SPRITES -->
        <img src="${ASSETS.bg[st.bg] || ASSETS.bg.vn_alley_stage || ASSETS.bg.intro_1}" class="pixel"
             style="position:absolute;left:0;top:0;width:1280px;height:560px;object-fit:cover;object-position:center bottom;filter:${card.bgFilter || 'none'};transition:filter 0.3s ease;">
        <!-- Subtle top vignette only — keeps the lower stone/cobblestone floor (y: 330..540) bright and unobstructed! -->
        <div style="position:absolute;left:0;right:0;top:0;height:560px;background:linear-gradient(180deg, rgba(8,10,15,0.55) 0%, transparent 24%, transparent 82%, rgba(8,10,15,0.65) 100%);pointer-events:none;"></div>

        <!-- Top Location & Stage Floor Indicator Strip -->
        <div style="position:absolute;left:72px;right:72px;top:16px;height:36px;padding:0 16px;background:rgba(12,15,23,0.84);border:1px solid rgba(212,166,74,0.42);border-radius:4px;display:flex;align-items:center;justify-content:space-between;z-index:15;">
          <div style="display:flex;align-items:center;gap:10px;">
            <span style="padding:2px 8px;background:rgba(212,166,74,0.18);border:1px solid var(--gold);border-radius:3px;font-size:10px;color:var(--gold);font-weight:700;letter-spacing:0.5px;">VN STAGE · VISIBLE FLOOR & FULL-BODY SPRITES</span>
            <span class="font-title" style="font-size:14px;color:var(--gold);">${st.title}</span>
            <span style="font-size:12px;color:var(--dim);">· ${st.subtitle}</span>
          </div>
          <div class="font-mono" style="font-size:11px;color:#fbbf24;display:flex;align-items:center;gap:10px;">
            <span>🎨 ${card.bgVariantLabel || 'Stage Artwork'}</span>
            ${card.shake ? `<span style="padding:1px 7px;background:rgba(220,38,38,0.22);border:1px solid #ef4444;border-radius:3px;color:#fca5a5;">📳 ${card.shake}</span>` : ''}
          </div>
        </div>

        <!-- Left & Right Full-Body Character Sprites standing directly on the stone/cobblestone floor (y: 192px above bottom) -->
        <div style="position:absolute;left:115px;right:115px;bottom:192px;height:345px;display:flex;align-items:flex-end;justify-content:space-between;pointer-events:none;z-index:10;">
          ${renderStoryActorSlot(card.leftActor, 'left')}
          ${renderStoryActorSlot(card.rightActor, 'right')}
        </div>
      `}

      <!-- Shared Bottom Ornate Dialogue & Portrait Mood Dock (story.rs / talk.rs — Reduced Corner Frame) -->
      ${ornateCornerBox(`
        <div style="display:flex;gap:18px;align-items:stretch;height:100%;">
          <!-- Left Speaker Portrait — Alters Facial Expression for Mood (faces/<id>/<mood>.png) -->
          <div style="display:flex;flex-direction:column;align-items:center;justify-content:space-between;width:136px;flex-shrink:0;">
            <div class="portrait-box gold-rim" style="width:112px;height:112px;box-shadow:0 0 18px rgba(245,158,11,0.35);position:relative;">
              <img src="${portraitSrc}" class="pixel" style="width:100%;height:100%;object-fit:cover;">
              <div style="position:absolute;right:4px;bottom:4px;padding:1px 6px;background:rgba(10,12,18,0.92);border:1px solid #f59e0b;border-radius:3px;font-family:'JetBrainsMono',monospace;font-size:9px;color:#fbbf24;text-transform:uppercase;">
                ${activeMood}
              </div>
            </div>
            <div style="padding:2px 8px;background:rgba(245,158,11,0.14);border:1px solid rgba(245,158,11,0.45);border-radius:3px;font-size:10px;color:#fbbf24;font-weight:700;white-space:nowrap;">
              Mood: ${card.emotion || activeMood}
            </div>
          </div>

          <!-- Right Dialogue Body, Portrait Mood Switcher & Navigation Controls -->
          <div style="flex:1;display:flex;flex-direction:column;justify-content:space-between;">
            <div>
              <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid rgba(75,85,99,0.45);padding-bottom:5px;margin-bottom:6px;">
                <div style="display:flex;align-items:baseline;gap:10px;">
                  <span class="font-title" style="font-size:18.5px;color:var(--gold);letter-spacing:0.4px;">${card.speaker}</span>
                  <span style="font-size:11.5px;color:var(--dim);">${card.speakerTitle || ''}</span>
                </div>
                <div style="display:flex;align-items:center;gap:8px;">
                  <!-- Interactive Portrait Mood Pills so user can test altering portrait mood live -->
                  <span class="font-mono" style="font-size:10px;color:#9ca3af;">Portrait Mood:</span>
                  ${availMoods.map(mKey => `
                    <button class="rust-btn ${activeMood === mKey ? 'primary' : ''}"
                            style="height:20px;padding:0 6px;font-size:9.5px;text-transform:capitalize;"
                            onclick="STATE.storyMoodOverride='${mKey}';render();"
                            title="Switch ${card.speaker}'s portrait expression to ${mKey} (faces/${card.portrait}/${mKey}.png)">
                      ${mKey}
                    </button>
                  `).join('')}
                  ${closeBtn720("STATE.storySkipConfirmOpen=true;render();", "Skip Scene [Esc / Right-Click]")}
                </div>
              </div>
              <div class="font-body" style="font-size:15.5px;line-height:1.48;color:var(--text);padding-right:12px;">
                ${card.text}
              </div>
            </div>

            <div style="display:flex;align-items:center;justify-content:space-between;border-top:1px solid rgba(55,65,81,0.6);padding-top:7px;margin-top:6px;">
              <div style="display:flex;align-items:center;gap:8px;">
                <span class="font-mono" style="font-size:11px;color:var(--dim);">Beat ${cardIdx + 1}/${st.cards.length}</span>
                <div style="display:flex;gap:4px;">
                  ${st.cards.map((_, idx) => `
                    <span onclick="STATE.storyCardIdx=${idx};STATE.storyMoodOverride=null;STATE.storyReactionOverride=null;render();" style="width:${idx === cardIdx ? '22px' : '8px'};height:6px;border-radius:3px;background:${idx === cardIdx ? '#f59e0b' : '#374151'};cursor:pointer;transition:all 0.2s;"></span>
                  `).join('')}
                </div>
                <span class="font-mono" style="font-size:10.5px;color:#93c5fd;margin-left:4px;">· ${card.stageAction}</span>
              </div>

              <div style="display:flex;align-items:center;gap:8px;">
                ${cardIdx > 0 ? `
                  <button class="rust-btn" style="height:30px;padding:0 12px;font-size:12px;" onclick="STATE.storyCardIdx--;STATE.storyMoodOverride=null;STATE.storyReactionOverride=null;render();">◀ Prev</button>
                ` : ''}
                <button class="rust-btn" style="height:30px;padding:0 12px;font-size:12px;" onclick="STATE.storyMode = 'vignette_reel';render();" title="Open Minimal Vignette Cutscene Player (Auto / Log / Skip)">🎬 Cutscene Player</button>
                ${!isLastCard ? `
                  <button class="rust-btn primary" style="height:32px;padding:0 18px;font-size:13px;" onclick="STATE.storyCardIdx++;STATE.storyMoodOverride=null;STATE.storyReactionOverride=null;render();">Continue [Space] ▸</button>
                ` : `
                  <button class="rust-btn primary" style="height:32px;padding:0 18px;font-size:13px;" onclick="STORIES[STATE.storyId].onFinish()">⚔ ${st.nextLabel} ▸</button>
                `}
              </div>
            </div>
          </div>
        </div>
      `, 'position:absolute;left:72px;right:72px;bottom:16px;height:172px;padding:14px 20px;background:rgba(12,15,22,0.96);box-shadow:0 16px 42px rgba(0,0,0,0.94);z-index:20;')}

      ${skipConfirmHtml}
    </div>
  `;
}


// ============================================================================
// SCREEN 3: REGION MAP (`region.rs` + `talk.rs` Dialogue Dock + Modal Overlays)
// ============================================================================
function renderRegionScreen() {
  const reg = REGIONS[STATE.regionId] || REGIONS.sluice_undercroft;
  const dlg = STATE.talkNpcId ? DIALOGUES[STATE.talkNpcId] : null;
  const dlgNode = dlg ? (dlg.nodes[STATE.talkNodeId] || dlg.nodes.root) : null;

  // Build 16x10 walkable tile grid (each tile 26x20 px centered in the 640x340 area)
  const tileW = 28;
  const tileH = 20;
  const gridLeft = 148;
  const gridTop = 26;

  let tilesHtml = '';
  for (let y = 0; y < reg.rows; y++) {
    for (let x = 0; x < reg.cols; x++) {
      const isWater = (reg.id === 'sluice_undercroft')
        ? (y === 0 || y === 9 || (x >= 6 && x <= 9 && (y <= 1 || y >= 8)))
        : (y === 8 || y === 9) && (x >= 4 && x <= 11);
      const isWall = (y === 0 || x === 0 || x === 15);
      const isHearth = (reg.id === 'sluice_undercroft' && x >= 6 && x <= 8 && y >= 5 && y <= 6);
      const isPlayer = (STATE.playerTile.x === x && STATE.playerTile.y === y);

      let bg = '#151821';
      let border = 'rgba(75,85,99,0.22)';
      if (isWater) {
        bg = '#0c1926';
        border = 'rgba(6,182,212,0.25)';
      } else if (isWall) {
        bg = '#0d0f14';
      } else if (isHearth) {
        bg = 'radial-gradient(circle, rgba(217,119,6,0.32) 0%, rgba(20,23,31,0.95) 100%)';
        border = 'rgba(217,119,6,0.4)';
      } else if ((x + y) % 2 === 0) {
        bg = '#181c26';
      }

      tilesHtml += `
        <div onclick="clickRegionTile(${x},${y})"
             style="position:absolute;left:${gridLeft + x * tileW}px;top:${gridTop + y * tileH}px;width:${tileW}px;height:${tileH}px;background:${bg};border:0.5px solid ${border};cursor:pointer;">
          ${isPlayer ? `<div style="position:absolute;inset:1px;border:1px solid var(--gold);background:rgba(251,191,36,0.12);"></div>` : ''}
        </div>
      `;
    }
  }

  // Hearth centerpiece in Sluice Undercroft
  const hearthHtml = reg.id === 'sluice_undercroft' ? `
    <div style="position:absolute;left:${gridLeft + 7 * tileW - 6}px;top:${gridTop + 5 * tileH - 4}px;width:40px;height:28px;pointer-events:none;display:flex;flex-direction:column;align-items:center;">
      <div style="width:18px;height:18px;border-radius:50%;background:radial-gradient(circle, #fbbf24 0%, #d97706 55%, rgba(153,27,27,0) 100%);box-shadow:0 0 14px #f59e0b;"></div>
      <span style="font-size:6px;color:var(--gold);background:rgba(10,11,16,0.85);padding:0 3px;border:0.5px solid #b45309;">Tallow Hearth</span>
    </div>
  ` : '';

  // Entities on the map
  const entitiesHtml = reg.entities.map(ent => {
    const px = gridLeft + ent.x * tileW;
    const py = gridTop + ent.y * tileH;
    const isSel = STATE.talkNpcId === ent.id;
    const spriteUri = ent.unitKey && ASSETS.units[ent.unitKey] ? ASSETS.units[ent.unitKey].idle0 : null;
    const iconUri = ent.icon ? ASSETS.ui[ent.icon] : null;
    const badgeUri = ent.badge ? ASSETS.ui[ent.badge] : null;

    return `
      <div onclick="interactRegionEntity('${ent.id}')"
           style="position:absolute;left:${px - 4}px;top:${py - 16}px;width:36px;height:36px;cursor:pointer;display:flex;flex-direction:column;align-items:center;z-index:10;">
        ${isSel ? `<div style="position:absolute;bottom:2px;width:26px;height:10px;border-radius:50%;border:1px solid var(--gold);background:rgba(251,191,36,0.22);"></div>` : ''}
        ${spriteUri ? `<img src="${spriteUri}" class="pixel" style="width:32px;height:32px;filter:drop-shadow(0 2px 2px #000);">` : ''}
        ${iconUri ? `<img src="${iconUri}" class="pixel" style="width:16px;height:16px;margin-top:10px;">` : ''}
        ${badgeUri ? `<img src="${badgeUri}" class="pixel" style="position:absolute;right:2px;top:0;width:10px;height:10px;">` : ''}
        <div style="font-size:6px;color:${isSel ? 'var(--gold)' : 'var(--text)'};background:rgba(10,11,16,0.88);padding:0 3px;border:0.5px solid ${isSel ? '#d97706' : '#4b5563'};white-space:nowrap;margin-top:-3px;">
          ${ent.name}
        </div>
      </div>
    `;
  }).join('');

  // Player Kaelen sprite
  const kPx = gridLeft + STATE.playerTile.x * tileW;
  const kPy = gridTop + STATE.playerTile.y * tileH;
  const playerHtml = `
    <div style="position:absolute;left:${kPx - 2}px;top:${kPy - 16}px;width:32px;height:34px;pointer-events:none;z-index:12;display:flex;flex-direction:column;align-items:center;">
      <img src="${ASSETS.units.kaelen.idle0}" class="pixel" style="width:32px;height:32px;filter:drop-shadow(0 2px 3px #000);">
      <span style="font-size:5.5px;color:#34d399;background:rgba(10,11,16,0.9);padding:0 2px;margin-top:-4px;">Kaelen (You)</span>
    </div>
  `;

  return `
    <div style="width:640px;height:360px;position:relative;background:#0a0b10;overflow:hidden;">
      ${renderHudBar(reg.title)}

      <!-- Walkable Region Tile Canvas (region.rs) -->
      <div style="position:absolute;left:0;top:20px;width:640px;height:340px;">
        ${tilesHtml}
        ${hearthHtml}
        ${entitiesHtml}
        ${playerHtml}
      </div>

      <!-- Top-Left Foldable "Here (N)" Legend (region.rs) -->
      <div class="rust-panel" style="position:absolute;left:6px;top:24px;width:136px;z-index:20;padding:4px 6px;">
        <div style="display:flex;align-items:center;justify-content:space-between;cursor:pointer;" onclick="STATE.hereOpen=!STATE.hereOpen;render();">
          <span class="font-title" style="font-size:8.5px;color:var(--gold);">Here (${reg.entities.length})</span>
          <span style="font-size:7px;color:var(--dim);">${STATE.hereOpen ? '▾' : '▸'}</span>
        </div>
        ${STATE.hereOpen ? `
          <div style="margin-top:4px;display:flex;flex-direction:column;gap:3px;">
            ${reg.entities.map(ent => `
              <div class="rust-card ${STATE.talkNpcId===ent.id?'selected':''}"
                   style="padding:3px 5px;cursor:pointer;display:flex;align-items:center;justify-content:space-between;"
                   onclick="interactRegionEntity('${ent.id}')">
                <div>
                  <div style="font-size:7px;color:var(--text);font-weight:bold;">${ent.name}</div>
                  <div style="font-size:6px;color:var(--dim);">${ent.role}</div>
                </div>
                <span style="font-size:7px;color:var(--gold);">▸</span>
              </div>
            `).join('')}
          </div>
        ` : ''}
      </div>

      <!-- Top-Right Event Ticker (region.rs log) -->
      <div class="rust-panel" style="position:absolute;right:6px;top:24px;width:168px;z-index:20;padding:4px 6px;">
        <div class="font-mono" style="font-size:6.5px;color:var(--gold);margin-bottom:2px;">WARD LOG</div>
        ${STATE.eventLog.slice(0, 3).map(line => `
          <div style="font-size:6px;color:var(--dim);line-height:1.3;border-top:0.5px solid #1f2937;padding-top:1px;">• ${line}</div>
        `).join('')}
      </div>

      <!-- Bottom-Center TalkBox (talk.rs: TALK_W = 400, TALK_LEFT = 76, PORTRAIT = 64 — Reduced Corner Frame) -->
      ${dlg && dlgNode ? ornateCornerBox640('position:absolute;left:120px;bottom:6px;width:410px;min-height:106px;z-index:25;padding:6px;display:flex;gap:8px;background:var(--talk-box);', `
          <!-- Left Column (76px): 64x64 Portrait + Speaker Name + Band -->
          <div style="width:76px;flex-shrink:0;display:flex;flex-direction:column;align-items:center;">
            <div class="portrait-box gold-rim" style="width:60px;height:60px;">
              <img src="${ASSETS.portraits[dlg.portrait] || ASSETS.portraits.aline}" class="pixel">
            </div>
            <div class="font-title" style="font-size:7.5px;color:var(--gold);margin-top:3px;text-align:center;line-height:1.1;">${dlg.name}</div>
            <div style="font-size:5.5px;color:var(--dim);text-align:center;line-height:1.15;margin-top:1px;">${dlg.band}</div>
          </div>

          <!-- Right Column: Roll Result Banner (if any) + Dialogue Text + 1px Divider + Numbered Reply Cards -->
          <div style="flex:1;display:flex;flex-direction:column;justify-content:space-between;">
            ${STATE.rollBanner ? `
              <div style="background:${STATE.rollBanner.passed ? 'rgba(16,185,129,0.18)' : 'rgba(225,29,72,0.2)'};border:1px solid ${STATE.rollBanner.passed ? '#10b981' : '#e11d48'};padding:2px 5px;margin-bottom:3px;display:flex;justify-content:space-between;align-items:center;font-family:'JetBrainsMono',monospace;font-size:6.5px;">
                <span style="color:var(--gold);">${STATE.rollBanner.label}: d20(${STATE.rollBanner.d20}) + ${STATE.rollBanner.mod} = ${STATE.rollBanner.total} vs DC ${STATE.rollBanner.dc}</span>
                <span style="color:${STATE.rollBanner.passed ? '#34d399' : '#fda4af'};font-weight:bold;">${STATE.rollBanner.passed ? 'PASSED' : 'FAILED'}</span>
              </div>
            ` : ''}
            <div style="font-size:7px;line-height:1.38;color:var(--text);max-height:42px;overflow-y:auto;padding-right:2px;">
              ${dlgNode.text}
            </div>
            <div style="border-top:1px solid #374151;margin:4px 0 3px 0;"></div>
            <div style="display:flex;flex-direction:column;gap:2.5px;">
              ${dlgNode.replies.map((rep, rIdx) => `
                <div class="rust-card" style="padding:2.5px 6px;font-size:6.8px;color:var(--text);cursor:pointer;display:flex;align-items:center;justify-content:space-between;"
                     onclick="clickTalkReply(${rIdx})">
                  <span>${rep.label}</span>
                  <span style="color:var(--gold);font-size:6px;">↵</span>
                </div>
              `).join('')}
            </div>
          </div>
      `) : ''}

      <!-- Modal Overlays (BoardPane, ShopPane, Character) -->
      ${STATE.modal === 'board' ? renderBoardModal() : ''}
      ${STATE.modal === 'shop' ? renderShopModal() : ''}
      ${STATE.modal === 'party' ? renderPartyModal() : ''}
    </div>
  `;
}

function clickRegionTile(x, y) {
  STATE.playerTile = { x, y };
  const reg = REGIONS[STATE.regionId];
  const hit = reg.entities.find(e => e.x === x && e.y === y);
  if (hit) {
    interactRegionEntity(hit.id);
  } else {
    render();
  }
}

function interactRegionEntity(entId) {
  const reg = REGIONS[STATE.regionId];
  const ent = reg.entities.find(e => e.id === entId);
  if (!ent) return;
  STATE.playerTile = { x: Math.max(1, ent.x - 1), y: ent.y };
  STATE.rollBanner = null;

  if (ent.kind === 'exit') {
    STATE.screen = 'worldmap';
    render();
  } else if (ent.kind === 'encounter') {
    launchEncounter('enc_m2');
  } else if (ent.kind === 'chest') {
    STATE.scrap += 20;
    STATE.eventLog.unshift('Looted Submerged Smuggler Crate (+20 Scrap).');
    render();
  } else {
    STATE.talkNpcId = ent.id;
    STATE.talkNodeId = 'root';
    render();
  }
}

function clickTalkReply(idx) {
  const dlg = DIALOGUES[STATE.talkNpcId];
  if (!dlg) return;
  const node = dlg.nodes[STATE.talkNodeId] || dlg.nodes.root;
  const rep = node.replies[idx];
  if (!rep) return;
  if (rep.action) {
    rep.action();
  } else if (rep.next) {
    STATE.rollBanner = null;
    STATE.talkNodeId = rep.next;
    render();
  }
}

// ============================================================================
// SCREEN 4A: NOTICE BOARD MODAL (`board.rs` — `BoardPane` with `#e11d48` Deadlines)
// ============================================================================
function renderBoardModal() {
  const selQuest = STATE.quests.find(q => q.id === STATE.boardSelId) || STATE.quests[0];

  return `
    <div style="position:absolute;inset:0;background:rgba(0,0,0,0.65);z-index:40;display:flex;align-items:center;justify-content:center;">
      ${ornateBox640('width:520px;height:286px;padding:8px 10px;display:flex;flex-direction:column;justify-content:space-between;background:var(--panel);', `
        <!-- Header: Title left, Keeper name + X button right (board.rs) -->
        <div>
          <div style="display:flex;justify-content:space-between;align-items:center;">
            <span class="font-title" style="font-size:11px;color:var(--gold);">Sluice Pillar Notice Board</span>
            <div style="display:flex;align-items:center;gap:6px;">
              <span style="font-size:7px;color:var(--dim);">Keeper: Night-Runner Tess</span>
              ${closeBtn640("STATE.modal=null;render();", "Close [Esc / Right-Click]")}
            </div>
          </div>
          <div style="font-size:6.8px;color:var(--dim);margin-top:1px;">
            "Pinned contracts across the Sunk Ward. Watch the red deadline counters—expired jobs seal off."
          </div>
        </div>

        <!-- Two-Column Body (LIST_W = 240 scaled) -->
        <div style="display:grid;grid-template-columns:205px 1fr;gap:8px;flex:1;margin:6px 0;overflow:hidden;">
          <!-- Left List -->
          <div class="rust-inset" style="padding:5px;overflow-y:auto;">
            ${['Main', 'Jobs'].map(sec => `
              <div class="font-title" style="font-size:7.5px;color:var(--gold);margin:2px 0 3px 2px;">${sec}</div>
              ${STATE.quests.filter(q => q.section === sec).map(q => `
                <div class="rust-card ${q.id === selQuest.id ? 'selected' : ''}"
                     style="padding:4px 5px;margin-bottom:3px;cursor:pointer;display:flex;align-items:center;justify-content:space-between;"
                     onclick="STATE.boardSelId='${q.id}';render();">
                  <div style="display:flex;align-items:center;gap:4px;overflow:hidden;">
                    <img src="${ASSETS.ui.badge_quest}" class="pixel" style="width:9px;height:9px;">
                    <span style="font-size:7px;color:var(--text);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${q.title}</span>
                  </div>
                  <span class="font-mono" style="font-size:6.5px;color:#e11d48;flex-shrink:0;">${q.deadlineDays}d</span>
                </div>
              `).join('')}
            `).join('')}
          </div>

          <!-- Right Detail Pane (board.rs detail layout) -->
          <div class="rust-inset" style="padding:7px 9px;display:flex;flex-direction:column;justify-content:space-between;">
            <div>
              <div style="display:flex;justify-content:space-between;align-items:center;">
                <span class="font-title" style="font-size:9.5px;color:var(--gold);">${selQuest.title}</span>
                <span class="font-mono" style="font-size:6.5px;padding:1px 4px;border:1px solid #d97706;color:var(--gold);">${selQuest.status}</span>
              </div>
              <div style="font-size:7px;color:var(--text);line-height:1.4;margin-top:5px;">
                ${selQuest.summary}
              </div>
              <div style="border-top:1px solid #2d333f;margin:6px 0;padding-top:5px;display:flex;flex-direction:column;gap:3px;font-size:6.8px;">
                <div><b style="color:var(--gold);">Now:</b> <span style="color:var(--text);">${selQuest.nowStep}</span></div>
                <div><b style="color:var(--dim);">Then:</b> <span style="color:var(--dim);">${selQuest.thenStep}</span></div>
              </div>
            </div>

            <div style="border-top:1px solid #2d333f;padding-top:5px;font-size:6.8px;display:flex;flex-direction:column;gap:2px;">
              <div><span style="color:var(--dim);">Where:</span> <b style="color:var(--text);">${selQuest.where}</b></div>
              <div><span style="color:var(--dim);">Rewards:</span> <b style="color:var(--gold);">${selQuest.rewardScrap} Scrap, +${selQuest.rewardTallow} Tallow</b></div>
              <div><span style="color:var(--dim);">Deadline:</span> <b style="color:#e11d48;">${selQuest.deadlineDays} days left before Red Tithing</b></div>
            </div>
          </div>
        </div>

        <!-- Bottom Bar: Hint left, Accept & Go right (board.rs) -->
        <div style="display:flex;justify-content:space-between;align-items:center;">
          <span class="font-mono" style="font-size:6.2px;color:var(--dim);">Right-Click or [Esc] to Close</span>
          <div style="display:flex;gap:5px;">
            <button class="rust-btn" onclick="acceptBoardQuest('${selQuest.id}', false)">Accept Contract</button>
            <button class="rust-btn primary" onclick="acceptBoardQuest('${selQuest.id}', true)">Accept & Plot on District Map ▸</button>
          </div>
        </div>
      `)}
    </div>
  `;
}

function acceptBoardQuest(qId, goMap) {
  const q = STATE.quests.find(x => x.id === qId);
  if (q && q.status === 'On offer') {
    q.status = 'Under way';
    STATE.eventLog.unshift(`Accepted contract: ${q.title}`);
  }
  if (goMap && q) {
    STATE.modal = null;
    STATE.screen = 'worldmap';
    STATE.worldNodeId = q.nodeId;
  }
  render();
}

// ============================================================================
// SCREEN 4B: SHOP MODAL (`shop.rs` — Sister Aline Apothecary & Bram Forge)
// ============================================================================
const SHOP_STOCK = {
  aline: {
    title: 'Sister Aline\'s Apothecary & Triage',
    sub: 'Surgical draughts, coagulants, and refined tallow for the Hearth.',
    items: [
      { name: 'Coagulant Vial (2-Pip Belt)', cost: 15, kind: 'Consumable', desc: 'Restores 15 HP & cures Bleed during battle (0B Free action).' },
      { name: 'Refined Hearth Tallow (+1 Supply)', cost: 20, kind: 'Campaign Supply', desc: 'Adds +1 Tallow for resting at The Dry Hollows and curing Wounds.' },
      { name: 'Laudanum Tincture (2-Pip Belt)', cost: 18, kind: 'Consumable', desc: 'Restores 18 HP and grants +1 Poise pip immediately.' },
      { name: 'Brimstone Salve (1-Pip Belt)', cost: 12, kind: 'Consumable', desc: 'Purges Burn & Poison and grants +6 Guard.' }
    ]
  },
  bram: {
    title: 'Bram\'s Slag-Heap Anvil',
    sub: 'Tempered Sunk Ward steel, reinforced brigandines, and belt rigs.',
    items: [
      { name: 'Tallow-Quenched Billhook (+2 Might)', cost: 35, kind: 'Weapon · Kaelen', desc: 'Upgrades Kaelen\'s Billhook Drag damage by +3 and Poise break by +1.' },
      { name: 'Serrated Gutting Pair (+2 Agility)', cost: 30, kind: 'Weapon · Vesper', desc: 'Adds +1 Bleed stack on Flank Cut and Crimson Lullaby.' },
      { name: 'Kiln-Plate Gorget (+6 Guard)', cost: 25, kind: 'Armor · Vanguard', desc: 'Increases starting battle Guard by +6.' },
      { name: 'Smuggler\'s Rigging (+1 Belt Pip)', cost: 28, kind: 'Trinket', desc: 'Reinforced leather loops for carrying heavy flasks.' }
    ]
  }
};

function renderShopModal() {
  const shop = SHOP_STOCK[STATE.shopKeeper] || SHOP_STOCK.aline;
  const selItem = shop.items[STATE.shopSelIdx] || shop.items[0];

  return `
    <div style="position:absolute;inset:0;background:rgba(0,0,0,0.65);z-index:40;display:flex;align-items:center;justify-content:center;">
      ${ornateBox640('width:500px;height:268px;padding:8px 10px;display:flex;flex-direction:column;justify-content:space-between;background:var(--panel);', `
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #2d333f;padding-bottom:4px;">
          <div>
            <div class="font-title" style="font-size:10.5px;color:var(--gold);">${shop.title}</div>
            <div style="font-size:6.8px;color:var(--dim);">${shop.sub}</div>
          </div>
          <div style="display:flex;gap:4px;align-items:center;">
            <button class="rust-btn ${STATE.shopKeeper==='aline'?'primary':''}" onclick="openShop('aline')">Aline (Apothecary)</button>
            <button class="rust-btn ${STATE.shopKeeper==='bram'?'primary':''}" onclick="openShop('bram')">Bram (Forge)</button>
            ${closeBtn640("STATE.modal=null;render();", "Close [Esc / Right-Click]")}
          </div>
        </div>

        <div style="display:grid;grid-template-columns:220px 1fr;gap:8px;flex:1;margin:6px 0;overflow:hidden;">
          <div class="rust-inset" style="padding:5px;overflow-y:auto;">
            ${shop.items.map((it, idx) => `
              <div class="rust-card ${STATE.shopSelIdx===idx?'selected':''}"
                   style="padding:4px 6px;margin-bottom:3px;cursor:pointer;display:flex;justify-content:space-between;align-items:center;"
                   onclick="STATE.shopSelIdx=${idx};render();">
                <span style="font-size:7px;color:var(--text);">${it.name}</span>
                <span class="font-mono" style="font-size:7px;color:var(--gold);">${it.cost}s</span>
              </div>
            `).join('')}
          </div>

          <div class="rust-inset" style="padding:8px;display:flex;flex-direction:column;justify-content:space-between;">
            <div>
              <div class="font-title" style="font-size:9px;color:var(--gold);">${selItem.name}</div>
              <div class="font-mono" style="font-size:6.5px;color:var(--info);margin-top:2px;">${selItem.kind}</div>
              <div style="font-size:7px;color:var(--text);line-height:1.4;margin-top:6px;">${selItem.desc}</div>
            </div>
            <div style="border-top:1px solid #2d333f;padding-top:5px;display:flex;justify-content:space-between;align-items:center;">
              <span class="font-mono" style="font-size:7.5px;color:var(--gold);">Cost: ${selItem.cost} Scrap (Have: ${STATE.scrap})</span>
              <button class="rust-btn primary" ${STATE.scrap < selItem.cost ? 'disabled' : ''} onclick="buyShopItem(${STATE.shopSelIdx})">Buy Item</button>
            </div>
          </div>
        </div>

        <div style="display:flex;justify-content:space-between;align-items:center;">
          <span class="font-mono" style="font-size:6.2px;color:var(--dim);">Right-Click or [Esc] to Close</span>
          <span style="font-size:6.5px;color:var(--dim);">Decision A3: Reuses existing <code style="color:var(--gold);">ShopPane</code> overlay from <code style="color:var(--gold);">shop.rs</code></span>
        </div>
      `)}
    </div>
  `;
}

function buyShopItem(idx) {
  const shop = SHOP_STOCK[STATE.shopKeeper] || SHOP_STOCK.aline;
  const it = shop.items[idx];
  if (!it || STATE.scrap < it.cost) return;
  STATE.scrap -= it.cost;
  if (it.name.includes('Tallow')) STATE.tallow += 1;
  STATE.eventLog.unshift(`Purchased ${it.name} (-${it.cost} Scrap).`);
  render();
}

// ============================================================================
// SCREEN 6: CHARACTER / PARTY MODAL (`party.rs` — 48x48 Portrait, 4 Tabs, 5-Pip Belt)
// ============================================================================
function renderPartyModal() {
  const h = STATE.roster[STATE.partyHeroId] || STATE.roster.kaelen;
  const activeCount = Object.values(STATE.roster).filter(x => x.active).length;
  const usedPips = h.pouch.reduce((acc, p) => acc + p.pips, 0);

  let tabContent = '';
  if (STATE.partyTab === 'Stats') {
    tabContent = `
      <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;height:100%;">
        <div class="rust-inset" style="padding:6px;display:flex;flex-direction:column;gap:3px;">
          <div class="font-title" style="font-size:8px;color:var(--gold);">Primary Attributes</div>
          ${Object.entries(h.stats).map(([k, v]) => `
            <div style="display:flex;justify-content:space-between;font-size:7px;border-bottom:0.5px solid #1f2937;padding:1.5px 0;">
              <span style="color:var(--dim);">${k}</span>
              <span class="font-mono" style="color:var(--text);">${v} (${v >= 10 ? '+' : ''}${Math.floor((v - 10) / 2)})</span>
            </div>
          `).join('')}
        </div>
        <div class="rust-inset" style="padding:6px;display:flex;flex-direction:column;justify-content:space-between;">
          <div>
            <div class="font-title" style="font-size:8px;color:var(--gold);">Combat Vital & Status</div>
            <div style="font-size:7px;margin-top:3px;display:flex;flex-direction:column;gap:2px;">
              <div>HP: <b class="font-mono" style="color:#34d399;">${h.hp} / ${h.maxHp}</b> · Guard: <b class="font-mono" style="color:#60a5fa;">${h.guard}</b></div>
              <div>Poise Threshold: <b class="font-mono" style="color:var(--gold);">${h.poise} / ${h.maxPoise} pips</b></div>
              <div>Speed: <b class="font-mono">${h.speed}</b> · Round AP: <b class="font-mono">+${h.ap} AP</b></div>
              <div>Class Meter: <b style="color:var(--gold);">${h.meterName}</b> (${h.meter}/${h.maxMeter})</div>
            </div>
          </div>
          ${h.wounded ? `
            <div style="background:rgba(225,29,72,0.2);border:1px solid #e11d48;padding:4px 6px;display:flex;justify-content:space-between;align-items:center;">
              <span style="font-size:6.5px;color:#fda4af;"><b>WOUNDED:</b> -25% Max HP & -1 Poise</span>
              <button class="rust-btn primary" style="height:13px;font-size:6px;" onclick="h_cureWound('${h.id}')">Cure (1 Tallow)</button>
            </div>
          ` : `<div style="font-size:6.5px;color:#34d399;">Status: Combat Ready (No Wounds)</div>`}
        </div>
      </div>
    `;
  } else if (STATE.partyTab === 'Skills') {
    tabContent = `
      <div style="display:grid;grid-template-columns:1.25fr 1fr;gap:8px;height:100%;">
        <div class="rust-inset" style="padding:6px;overflow-y:auto;">
          <div class="font-title" style="font-size:8px;color:var(--gold);margin-bottom:3px;">Active Deck (3 Actives / 2 Passives / 1 Keystone)</div>
          <div style="display:flex;flex-direction:column;gap:3px;">
            ${h.actives.map(skId => {
              const sk = SKILLS[skId] || SKILLS.strike;
              return `
                <div class="rust-card" style="padding:3px 5px;display:flex;align-items:center;gap:5px;">
                  <img src="${getSkillIcon(skId)}" class="pixel" style="width:14px;height:14px;">
                  <div style="flex:1;">
                    <div style="font-size:7px;color:var(--text);font-weight:bold;">${sk.name} <span class="font-mono" style="color:var(--gold);">(${sk.blocks}B · ${sk.ap}AP)</span></div>
                    <div style="font-size:6px;color:var(--dim);">${sk.desc}</div>
                  </div>
                </div>
              `;
            }).join('')}
            <div style="font-size:6.5px;color:var(--dim);margin-top:3px;">
              <b>Passives (2):</b> ${h.passives.join(' · ')}<br>
              <b style="color:var(--gold);">Keystone (1):</b> ${h.keystone}
            </div>
          </div>
        </div>
        <div class="rust-inset" style="padding:6px;display:flex;flex-direction:column;justify-content:space-between;">
          <div>
            <div class="font-title" style="font-size:8px;color:var(--gold);">F3.2 Basics (5 Universal + 1 Role Basic)</div>
            <div style="display:flex;flex-wrap:wrap;gap:3px;margin-top:4px;">
              ${h.basics.map((bId, idx) => {
                const sk = SKILLS[bId] || SKILLS.strike;
                return `
                  <div class="rust-card ${idx===5?'selected':''}" style="padding:2px 4px;display:flex;align-items:center;gap:3px;font-size:6.3px;" title="${sk.desc}">
                    <img src="${getSkillIcon(bId)}" class="pixel" style="width:11px;height:11px;">
                    <span>${sk.name}</span>
                  </div>
                `;
              }).join('')}
            </div>
          </div>
          <div style="border-top:1px solid #2d333f;padding-top:4px;">
            <div class="font-title" style="font-size:7.5px;color:var(--gold);">Auto-Fill Intent Priority</div>
            <div class="font-mono" style="font-size:6.5px;color:var(--text);margin-top:2px;">${h.intentOrder.join(' → ')}</div>
          </div>
        </div>
      </div>
    `;
  } else if (STATE.partyTab === 'Gear') {
    tabContent = `
      <div style="display:grid;grid-template-columns:1fr 1.1fr 1fr;gap:6px;height:100%;">
        <div class="rust-inset" style="padding:6px;">
          <div class="font-title" style="font-size:8px;color:var(--gold);margin-bottom:4px;">Equipped Gear</div>
          <div style="display:flex;flex-direction:column;gap:4px;font-size:6.8px;">
            <div class="rust-card" style="padding:4px;"><span style="color:var(--dim);">Weapon:</span><br><b>${h.weapon}</b></div>
            <div class="rust-card" style="padding:4px;"><span style="color:var(--dim);">Armor:</span><br><b>${h.armor}</b></div>
            <div class="rust-card" style="padding:4px;"><span style="color:var(--dim);">Trinket:</span><br><b>${h.trinket}</b></div>
          </div>
        </div>
        <div class="rust-inset" style="padding:6px;display:flex;flex-direction:column;justify-content:space-between;">
          <div>
            <div style="display:flex;justify-content:space-between;align-items:center;">
              <span class="font-title" style="font-size:8px;color:var(--gold);">5-Pip Belt Pouch</span>
              <span class="font-mono" style="font-size:6.5px;color:var(--gold);">${usedPips}/5 Pips</span>
            </div>
            <!-- 5 visual belt pips -->
            <div style="display:flex;gap:3px;margin:4px 0 6px 0;">
              ${[0,1,2,3,4].map(i => `
                <div style="flex:1;height:7px;background:${i < usedPips ? '#d97706' : '#1f2937'};border:1px solid ${i < usedPips ? '#fbbf24' : '#374151'};"></div>
              `).join('')}
            </div>
            ${h.pouch.map(item => `
              <div class="rust-card" style="padding:3px 5px;margin-bottom:3px;font-size:6.5px;">
                <div style="display:flex;justify-content:space-between;">
                  <b style="color:var(--text);">${item.name}</b>
                  <span class="font-mono" style="color:var(--gold);">${item.pips} pip</span>
                </div>
                <div style="color:var(--dim);font-size:6px;">${item.desc}</div>
              </div>
            `).join('')}
          </div>
          <div style="font-size:6px;color:var(--dim);">Belt consumables trigger as 0B Free actions in battle.</div>
        </div>
        <div class="rust-inset" style="padding:6px;">
          <div class="font-title" style="font-size:8px;color:var(--gold);margin-bottom:4px;">Shared Sluice Stash</div>
          <div class="rust-card" style="padding:4px;font-size:6.5px;margin-bottom:3px;">
            <b>Watchman\'s Buckler</b><br><span style="color:var(--dim);">+4 Guard · Offhand</span>
          </div>
          <div class="rust-card" style="padding:4px;font-size:6.5px;margin-bottom:3px;">
            <b>Smuggler\'s Fire-Oil (2 Pip)</b><br><span style="color:var(--dim);">+4 Burn on next weapon skill</span>
          </div>
        </div>
      </div>
    `;
  } else {
    tabContent = `
      <div class="rust-inset" style="padding:8px;height:100%;display:flex;flex-direction:column;justify-content:space-between;">
        <div>
          <div class="font-title" style="font-size:8.5px;color:var(--gold);">Class Mastery & Level Progression (${h.name} · Lv.${h.level})</div>
          <div style="font-size:7px;color:var(--dim);margin-top:2px;">XP: ${h.xp} / 200 to Level 3 · Unlock Tier II Signature Active or Passive slot.</div>
          <div style="display:grid;grid-template-columns:1fr 1fr;gap:6px;margin-top:6px;">
            <div class="rust-card selected" style="padding:5px;font-size:6.8px;">
              <b style="color:var(--gold);">Tier I Mastery (Unlocked)</b><br>
              <span>${h.passives[0]} + ${h.actives[0]}</span>
            </div>
            <div class="rust-card" style="padding:5px;font-size:6.8px;">
              <b style="color:var(--text);">Tier II Specialization (Lv.3)</b><br>
              <span>Unlocks Keystone: ${h.keystone}</span>
            </div>
          </div>
        </div>
        <div style="font-size:6.5px;color:var(--dim);">XP is awarded from completed combat encounters and skill-check parleys.</div>
      </div>
    `;
  }

  return `
    <div style="position:absolute;inset:0;background:rgba(0,0,0,0.68);z-index:40;display:flex;align-items:center;justify-content:center;">
      ${ornateBox640('width:540px;height:296px;padding:8px 10px;display:flex;flex-direction:column;justify-content:space-between;background:var(--panel);', `
        <!-- Top Row: 6 Founder Switcher Tabs (Q/E) + Active 4/6 Counter + X Button -->
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #2d333f;padding-bottom:5px;">
          <div style="display:flex;gap:3px;">
            ${Object.values(STATE.roster).map(m => `
              <button class="rust-btn ${m.id===h.id?'primary':''}" style="height:17px;padding:0 6px;" onclick="STATE.partyHeroId='${m.id}';render();">
                <span>${m.name}</span>
                ${m.active ? `<span style="color:#34d399;font-size:6px;">●</span>` : `<span style="color:#6b7280;font-size:6px;">○</span>`}
                ${m.wounded ? `<span style="color:#e11d48;font-size:6px;">✚</span>` : ''}
              </button>
            `).join('')}
          </div>
          <div style="display:flex;align-items:center;gap:6px;">
            <span class="font-mono" style="font-size:7px;color:var(--gold);">Active Vanguard: ${activeCount} / 4</span>
            ${closeBtn640("STATE.modal=null;render();", "Close [Esc / Right-Click]")}
          </div>
        </div>

        <!-- Hero Header (party.rs: 48x48 Portrait + Name/Role + Active Toggle + Sub-Tabs) -->
        <div style="display:flex;align-items:center;justify-content:space-between;margin:5px 0;">
          <div style="display:flex;align-items:center;gap:8px;">
            <div class="portrait-box gold-rim" style="width:44px;height:44px;">
              <img src="${ASSETS.portraits[h.id]}" class="pixel">
            </div>
            <img src="${ASSETS.units[h.id].idle0}" class="pixel" style="width:36px;height:36px;">
            <div>
              <div class="font-title" style="font-size:10px;color:var(--gold);">${h.name} — ${h.title}</div>
              <div style="font-size:7px;color:var(--dim);">Lv.${h.level} ${h.classId.toUpperCase()} (${h.roleName})</div>
            </div>
          </div>

          <div style="display:flex;flex-direction:column;align-items:flex-end;gap:4px;">
            <button class="rust-btn ${h.active?'primary':''}" onclick="toggleHeroActive('${h.id}')">
              ${h.active ? '✓ In Active 4-Squad (Click to Bench)' : '+ Assign to Active Squad'}
            </button>
            <div style="display:flex;gap:3px;">
              ${['Stats', 'Skills', 'Gear', 'Level up'].map(t => `
                <button class="rust-btn ${STATE.partyTab===t?'primary':''}" style="height:14px;font-size:6.5px;" onclick="STATE.partyTab='${t}';render();">${t}</button>
              `).join('')}
            </div>
          </div>
        </div>

        <!-- Tab Body -->
        <div style="flex:1;overflow:hidden;margin-bottom:5px;">
          ${tabContent}
        </div>

        <!-- Footer -->
        <div style="display:flex;justify-content:space-between;align-items:center;">
          <span class="font-mono" style="font-size:6.2px;color:var(--dim);">Right-Click or [Esc] to Close</span>
          <span style="font-size:6.5px;color:var(--dim);">Switch Founders with tabs above · Bench Wounded heroes in The Dry Hollows</span>
        </div>
      `)}
    </div>
  `;
}

function toggleHeroActive(id) {
  const h = STATE.roster[id];
  if (!h) return;
  const activeCount = Object.values(STATE.roster).filter(x => x.active).length;
  if (h.active && activeCount > 1) {
    h.active = false;
  } else if (!h.active && activeCount < 4) {
    h.active = true;
  }
  render();
}

function h_cureWound(id) {
  if (STATE.tallow > 0) {
    STATE.tallow -= 1;
    STATE.roster[id].wounded = false;
    STATE.roster[id].hp = STATE.roster[id].maxHp;
    render();
  }
}

// ============================================================================
// SCREEN 5: WORLD MAP (`worldmap.rs` — 500x316 Left Map + Right Side Panel)
// ============================================================================
function renderWorldMap() {
  const selNode = WORLD_NODES.find(n => n.id === STATE.worldNodeId) || WORLD_NODES[2];
  const travelCost = STATE.travelMode === 'Stealth' ? selNode.days + 1 : selNode.days;

  // SVG route lines between nodes
  let linesSvg = '';
  WORLD_NODES.forEach(n => {
    n.edges.forEach(targetId => {
      const t = WORLD_NODES.find(x => x.id === targetId);
      if (t && n.id < t.id) {
        const isHot = (n.id === selNode.id || t.id === selNode.id);
        linesSvg += `<line x1="${n.x}" y1="${n.y}" x2="${t.x}" y2="${t.y}" stroke="${isHot ? '#fbbf24' : '#6b7280'}" stroke-width="${isHot ? '2' : '1.2'}" stroke-dasharray="3,2" />`;
      }
    });
  });

  return `
    <div style="width:640px;height:360px;position:relative;background:#0a0b10;overflow:hidden;">
      ${renderHudBar('The Sunk Ward · District Map')}

      <!-- Left Map Viewport (MAP = Rect(0, 22, 500, 316)) -->
      <div style="position:absolute;left:4px;top:24px;width:496px;height:328px;border:1px solid #374151;overflow:hidden;">
        <img src="${ASSETS.bg.sunk_ward}" class="pixel" style="position:absolute;inset:0;width:100%;height:100%;object-fit:cover;filter:brightness(0.82) contrast(1.08);">
        <div style="position:absolute;inset:0;background:radial-gradient(circle at 50% 50%, rgba(10,11,16,0.05) 40%, rgba(10,11,16,0.75) 100%);"></div>

        <svg style="position:absolute;inset:0;width:100%;height:100%;pointer-events:none;">
          ${linesSvg}
        </svg>

        ${WORLD_NODES.map(n => {
          const isSel = n.id === selNode.id;
          const isHere = n.id === 'n_sluice';
          return `
            <div onclick="STATE.worldNodeId='${n.id}';render();"
                 style="position:absolute;left:${n.x - 14}px;top:${n.y - 14}px;width:28px;height:28px;cursor:pointer;display:flex;flex-direction:column;align-items:center;z-index:10;">
              <div style="width:20px;height:20px;border-radius:50%;background:${isSel ? 'rgba(217,119,6,0.35)' : 'rgba(10,11,16,0.85)'};border:1.5px solid ${isSel ? '#fbbf24' : '#9ca3af'};display:flex;align-items:center;justify-content:center;box-shadow:0 2px 6px #000;">
                <img src="${ASSETS.ui[n.pin] || ASSETS.ui.pin_town}" class="pixel" style="width:12px;height:12px;">
              </div>
              ${n.quest ? `<img src="${ASSETS.ui.badge_quest}" class="pixel" style="position:absolute;right:0;top:-2px;width:10px;height:10px;">` : ''}
              ${isHere ? `<img src="${ASSETS.ui.pin_here}" class="pixel" style="position:absolute;left:-2px;top:-2px;width:10px;height:10px;">` : ''}
              <div style="font-size:6px;color:${isSel ? 'var(--gold)' : 'var(--text)'};background:rgba(10,11,16,0.92);padding:0.5px 3px;border:0.5px solid ${isSel ? '#d97706' : '#374151'};white-space:nowrap;margin-top:1px;">
                ${n.name}
              </div>
            </div>
          `;
        }).join('')}

        <!-- Map Legend Overlay -->
        <div class="rust-panel" style="position:absolute;left:6px;bottom:6px;padding:3px 6px;font-size:6px;color:var(--dim);display:flex;gap:8px;">
          <span><b style="color:var(--gold);">Decision A4 Hybrid Arrival:</b></span>
          <span>Parley Nodes → Street RegionMap</span>
          <span>·</span>
          <span>Raid Nodes → Direct BattleScreen</span>
        </div>
      </div>

      <!-- Right Side Panel (worldmap.rs: x=504..636 — Reduced Corner Frame + X Close Button) -->
      ${ornateCornerBox640('position:absolute;left:504px;top:24px;width:132px;height:328px;padding:7px;display:flex;flex-direction:column;justify-content:space-between;', `
        <div>
          <div style="display:flex;justify-content:space-between;align-items:flex-start;gap:4px;">
            <div class="font-title" style="font-size:9px;color:var(--gold);line-height:1.2;">${selNode.name}</div>
            ${closeBtn640("STATE.screen='region';STATE.regionId='sluice_undercroft';render();", "Back to Sluice Hub [Esc / Right-Click]")}
          </div>
          <div class="font-mono" style="font-size:6px;color:var(--info);margin-top:2px;">${selNode.mode}</div>
          <div style="border-top:1px solid #2d333f;margin:5px 0;padding-top:5px;font-size:6.8px;color:var(--text);line-height:1.38;">
            ${selNode.desc}
          </div>
          <div class="rust-inset" style="padding:4px 5px;font-size:6.3px;color:var(--dim);margin-top:4px;">
            <div>Travel Mode: <b style="color:var(--gold);">${STATE.travelMode}</b></div>
            <div>Transit Time: <b style="color:var(--text);">${selNode.days === 0 ? 'Here (0d)' : `${travelCost} day(s)`}</b></div>
            <div style="color:#fda4af;margin-top:2px;">Arrive Day ${STATE.day + (selNode.days ? travelCost : 0)} (${ Math.max(1, STATE.tithingDays - (selNode.days ? travelCost : 0)) }d left)</div>
          </div>
        </div>

        <div style="display:flex;flex-direction:column;gap:4px;">
          <button class="rust-btn primary" style="width:100%;height:18px;" onclick="WORLD_NODES.find(n=>n.id==='${selNode.id}').onEnter()">
            ${selNode.days === 0 ? 'Enter Sanctuary Hub ▸' : 'Travel & Enter Node ▸'}
          </button>
          <button class="rust-btn" style="width:100%;" onclick="STATE.travelMode = STATE.travelMode==='Stealth'?'Fast':'Stealth';render();">
            Mode: ${STATE.travelMode} (${STATE.travelMode==='Stealth'?'Low Alert':'Fast -1d'})
          </button>
          <button class="rust-btn" style="width:100%;" onclick="restAtHearth()">
            Camp & Tend Wounds (-1 Tallow)
          </button>
          <div class="font-mono" style="font-size:5.8px;color:var(--dim);text-align:center;margin-top:1px;">
            Right-Click or [Esc] → Hub
          </div>
        </div>
      `)}

      ${STATE.modal === 'party' ? renderPartyModal() : ''}
      ${STATE.modal === 'board' ? renderBoardModal() : ''}
      ${STATE.modal === 'shop' ? renderShopModal() : ''}
    </div>
  `;
}

