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
// ============================================================================
// LIVE2D / VTUBER-STYLE 2D RIG ENGINE (60fps Deformer Mesh + Separated Weapon Joint)
// Models each full-body sprite like a 2D VTuber model:
//  - Part Separation: `body` layer (head, torso, legs, coat) vs `weapon` layer (polearm/blade + weapon hand)
//  - Warp Deformers : Horizontal strip mesh with pinned boots (`y = 336`), knee flex (`y = 278`),
//                     pelvis crouch drop (`y = 215`), spine lean (`y = 95..215`), chest breathing expansion,
//                     head counter-tilt (`y < 95` keeps eyes level when crouching!), and coat pendulum lag.
//  - Rigid Joint    : `weapon` layer rotates cleanly around the animated shoulder/grip joint `(px', py')`
//                     so polearms and blades never warp when knees bend or when drawing steel!
//  - 10-Part All-Limbs Skeletal Segmentation (Offscreen Canvas Parts with Circular Joint Caps):
//      * `head`       : Head & neck (pivots at `neck`, counter-tilts to keep eyes locked forward)
//      * `torso`      : Chest, cuirass, pelvis, & belt tassets (pivots at `pelvis` + chest breathing)
//      * `armL_upper` : Back/Off-hand Upper Arm (`shoulderL -> elbowL`)
//      * `armL_lower` : Back/Off-hand Forearm & Bracer/Hand (`elbowL -> handL`)
//      * `armR_upper` : Front/Weapon Upper Arm (`shoulderR -> elbowR`)
//      * `armR_lower` : Front/Weapon Forearm & Hand (`elbowR -> handR`)
//      * `legL_thigh` : Back/Left Thigh (`hipL -> kneeL`, rigid bone length L1_L)
//      * `legL_shin`  : Back/Left Shin/Calf (`kneeL -> ankleL`, rigid bone length L2_L)
//      * `legL_boot`  : Back/Left Boot (`ankleL -> toeL`, stays planted flat on the floor)
//      * `legR_thigh` : Front/Right Thigh (`hipR -> kneeR`, rigid bone length L1_R)
//      * `legR_shin`  : Front/Right Shin/Calf (`kneeR -> ankleR`, rigid bone length L2_R)
//      * `legR_boot`  : Front/Right Boot (`ankleR -> toeR`, stays planted flat on the floor)
//      * `weapon`     : Rigid polearm/blade attached to `handR` pivot
//  - True 2-Bone Analytic Inverse Kinematics (Law of Cosines) on Both Legs:
//      When crouching in `2. Sneaking` or lunging in `3. Weapons Drawn`, the pelvis drops while the
//      feet step into stance; both knees (`kneeL`, `kneeR`) genuinely bend forward via 2-bone IK
//      preserving 100% of thigh and shin bone lengths (zero vertical squishing!).
// ============================================================================
const VTUBER_ANATOMY_CONFIG = {
  kaelen: {
    head: [148, 50], neck: [148, 95], chest: [148, 146], pelvis: [146, 206],
    shoulderL: [104, 114], elbowL: [96, 154], handL: [104, 194],
    shoulderR: [174, 114], elbowR: [192, 150], handR: [220, 156],
    hipL: [123, 222], kneeL: [113, 268], ankleL: [108, 316], toeL: [104, 330],
    hipR: [167, 222], kneeR: [167, 268], ankleR: [168, 316], toeR: [178, 330],
    torsoL: 113, torsoR: 175, hipY: 216, legSplitX: 145
  },
  kaelen_side: {
    head: [148, 50], neck: [148, 95], chest: [148, 146], pelvis: [146, 206],
    shoulderL: [104, 114], elbowL: [96, 154], handL: [104, 194],
    shoulderR: [174, 114], elbowR: [192, 150], handR: [220, 156],
    hipL: [123, 222], kneeL: [113, 268], ankleL: [108, 316], toeL: [104, 330],
    hipR: [167, 222], kneeR: [167, 268], ankleR: [168, 316], toeR: [178, 330],
    torsoL: 113, torsoR: 175, hipY: 216, legSplitX: 145
  },
  kaelen_front: {
    head: [154, 50], neck: [154, 95], chest: [154, 146], pelvis: [154, 206],
    shoulderL: [110, 114], elbowL: [98, 152], handL: [96, 192],
    shoulderR: [192, 116], elbowR: [206, 150], handR: [228, 156],
    hipL: [132, 220], kneeL: [125, 268], ankleL: [120, 316], toeL: [112, 330],
    hipR: [176, 220], kneeR: [181, 268], ankleR: [185, 316], toeR: [192, 330],
    torsoL: 117, torsoR: 188, hipY: 214, legSplitX: 154
  },
  kaelen_back: {
    head: [184, 50], neck: [184, 95], chest: [184, 146], pelvis: [184, 206],
    shoulderL: [142, 116], elbowL: [126, 150], handL: [110, 156],
    shoulderR: [224, 116], elbowR: [238, 152], handR: [242, 192],
    hipL: [162, 220], kneeL: [158, 268], ankleL: [152, 316], toeL: [146, 330],
    hipR: [204, 220], kneeR: [212, 268], ankleR: [216, 316], toeR: [224, 330],
    torsoL: 148, torsoR: 220, hipY: 214, legSplitX: 184
  },
  vane: {
    head: [190, 52], neck: [190, 96], chest: [196, 148], pelvis: [196, 208],
    shoulderL: [148, 114], elbowL: [134, 148], handL: [132, 162],
    shoulderR: [244, 116], elbowR: [256, 158], handR: [262, 198],
    hipL: [162, 220], kneeL: [154, 270], ankleL: [146, 316], toeL: [140, 330],
    hipR: [222, 220], kneeR: [228, 270], ankleR: [234, 316], toeR: [244, 330],
    torsoL: 152, torsoR: 240, hipY: 214, legSplitX: 192
  },
  vesper: {
    head: [158, 52], neck: [156, 96], chest: [156, 146], pelvis: [156, 204],
    shoulderL: [122, 114], elbowL: [112, 152], handL: [108, 188],
    shoulderR: [184, 116], elbowR: [194, 152], handR: [190, 165],
    hipL: [138, 218], kneeL: [124, 268], ankleL: [116, 316], toeL: [112, 330],
    hipR: [178, 218], kneeR: [182, 268], ankleR: [186, 316], toeR: [202, 330],
    torsoL: 126, torsoR: 182, hipY: 212, legSplitX: 160
  },
  maren: {
    head: [158, 52], neck: [156, 96], chest: [156, 146], pelvis: [156, 208],
    shoulderL: [118, 114], elbowL: [108, 152], handL: [106, 190],
    shoulderR: [188, 116], elbowR: [196, 150], handR: [192, 155],
    hipL: [134, 220], kneeL: [130, 270], ankleL: [126, 316], toeL: [124, 330],
    hipR: [180, 220], kneeR: [184, 270], ankleR: [188, 316], toeR: [204, 330],
    torsoL: 124, torsoR: 186, hipY: 214, legSplitX: 158
  },
  aline: {
    head: [168, 52], neck: [166, 96], chest: [166, 146], pelvis: [166, 208],
    shoulderL: [134, 114], elbowL: [128, 152], handL: [128, 190],
    shoulderR: [192, 116], elbowR: [198, 152], handR: [192, 160],
    hipL: [146, 220], kneeL: [144, 270], ankleL: [142, 316], toeL: [140, 330],
    hipR: [186, 220], kneeR: [188, 270], ankleR: [190, 316], toeR: [204, 330],
    torsoL: 138, torsoR: 190, hipY: 214, legSplitX: 166
  },
  watchman: {
    head: [196, 54], neck: [196, 98], chest: [200, 148], pelvis: [200, 206],
    shoulderL: [158, 116], elbowL: [142, 150], handL: [124, 155],
    shoulderR: [238, 118], elbowR: [248, 156], handR: [248, 196],
    hipL: [176, 218], kneeL: [174, 268], ankleL: [170, 316], toeL: [154, 330],
    hipR: [222, 218], kneeR: [228, 268], ankleR: [232, 316], toeR: [236, 330],
    torsoL: 162, torsoR: 234, hipY: 212, legSplitX: 198
  }
};

function segmentCharacterIntoLimbs(bodyImg, cfg) {
  const W = 340, H = 340;
  if (typeof document === 'undefined' || !document.createElement) return null;
  const srcCv = document.createElement('canvas');
  srcCv.width = W; srcCv.height = H;
  const srcCtx = srcCv.getContext('2d');
  if (!srcCtx) return null;
  srcCtx.drawImage(bodyImg, 0, 0, W, H);
  let srcData;
  try {
    srcData = srcCtx.getImageData(0, 0, W, H);
  } catch (e) {
    return null;
  }
  const px = srcData.data;

  const partNames = [
    'head', 'torso',
    'armL_upper', 'armL_lower',
    'armR_upper', 'armR_lower',
    'legL_thigh', 'legL_shin', 'legL_boot',
    'legR_thigh', 'legR_shin', 'legR_boot'
  ];
  const partCanvases = {};
  const partImgData = {};
  for (const name of partNames) {
    const cv = document.createElement('canvas');
    cv.width = W; cv.height = H;
    const c = cv.getContext('2d');
    const id = c.createImageData(W, H);
    partCanvases[name] = { cv, ctx: c };
    partImgData[name] = id;
  }

  function copyPx(partName, idx) {
    const dst = partImgData[partName].data;
    dst[idx]     = px[idx];
    dst[idx + 1] = px[idx + 1];
    dst[idx + 2] = px[idx + 2];
    dst[idx + 3] = px[idx + 3];
  }

  function distSq(x, y, pt) {
    const dx = x - pt[0];
    const dy = y - pt[1];
    return dx * dx + dy * dy;
  }

  const rNeckSq = 16 * 16;
  const rShoulderSq = 24 * 24;
  const rElbowSq = 16 * 16;
  const rHipSq = 19 * 19;
  const rKneeSq = 18 * 18;
  const rAnkleSq = 15 * 15;

  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      const idx = (y * W + x) * 4;
      if (px[idx + 3] === 0) continue;

      // 1. Head vs Upper Body
      if (y <= cfg.neck[1] && x >= cfg.torsoL - 4 && x <= cfg.torsoR + 4) {
        copyPx('head', idx);
        if (distSq(x, y, cfg.neck) <= rNeckSq) copyPx('torso', idx);
        continue;
      }
      if (distSq(x, y, cfg.neck) <= rNeckSq && y <= cfg.neck[1] + 14) {
        copyPx('head', idx);
      }

      // 2. Upper Body: Left Arm, Right Arm, and Torso (with rounded shoulder and elbow caps)
      if (y <= cfg.hipY) {
        const armL_xMax = y >= 170 ? cfg.torsoL + 4 : cfg.torsoL;
        const armR_xMin = y >= 172 ? cfg.torsoR + 12 : cfg.torsoR;
        const inL = (x <= armL_xMax && y >= cfg.shoulderL[1] - 12) || (distSq(x, y, cfg.shoulderL) <= rShoulderSq && x <= cfg.torsoL + 6);
        const inR = (x >= armR_xMin && y >= cfg.shoulderR[1] - 14) || (distSq(x, y, cfg.shoulderR) <= rShoulderSq && x >= cfg.torsoR - 6);

        if (inL && y <= 207) {
          if (y <= cfg.elbowL[1]) {
            if ((y >= cfg.shoulderL[1] + 6 || distSq(x, y, cfg.shoulderL) <= rShoulderSq) && (y <= cfg.elbowL[1] - 6 || distSq(x, y, cfg.elbowL) <= rElbowSq)) {
              copyPx('armL_upper', idx);
            }
            if (distSq(x, y, cfg.elbowL) <= rElbowSq) copyPx('armL_lower', idx);
          } else {
            if (y >= cfg.elbowL[1] + 6 || distSq(x, y, cfg.elbowL) <= rElbowSq) {
              copyPx('armL_lower', idx);
            }
            if (distSq(x, y, cfg.elbowL) <= rElbowSq) copyPx('armL_upper', idx);
          }
          if (y <= 145 && (x >= cfg.torsoL - 8 || distSq(x, y, cfg.shoulderL) <= 324)) {
            copyPx('torso', idx);
          }
          continue;
        }

        if (inR && y <= 178) {
          if (y <= cfg.elbowR[1]) {
            if ((y >= cfg.shoulderR[1] + 6 || distSq(x, y, cfg.shoulderR) <= rShoulderSq) && (y <= cfg.elbowR[1] - 6 || distSq(x, y, cfg.elbowR) <= rElbowSq)) {
              copyPx('armR_upper', idx);
            }
            if (distSq(x, y, cfg.elbowR) <= rElbowSq) copyPx('armR_lower', idx);
          } else {
            if (y >= cfg.elbowR[1] + 6 || distSq(x, y, cfg.elbowR) <= rElbowSq) {
              copyPx('armR_lower', idx);
            }
            if (distSq(x, y, cfg.elbowR) <= rElbowSq) copyPx('armR_upper', idx);
          }
          if (y <= 145 && (x <= cfg.torsoR + 10 || distSq(x, y, cfg.shoulderR) <= 324)) {
            copyPx('torso', idx);
          }
          continue;
        }

        if (!(x <= cfg.torsoL + 1 && y >= 190)) {
          copyPx('torso', idx);
        }
        if (distSq(x, y, cfg.hipL) <= rHipSq && x >= cfg.torsoL - 5) {
          copyPx('legL_thigh', idx);
        }
        if (distSq(x, y, cfg.hipR) <= rHipSq) {
          copyPx('legR_thigh', idx);
        }
        continue;
      }

      // 3. Lower Body: Left Leg (`x < legSplitX`) and Right Leg (`x >= legSplitX`)
      //    Each leg is segmented into Thigh (`hip -> knee`), Shin (`knee -> ankle`), and Boot (`ankle -> toe`)
      //    with rounded circular caps at Hip, Knee, and Ankle so rotating never exposes corners or gaps!
      if (x < cfg.legSplitX) {
        if (y <= cfg.kneeL[1]) {
          if ((y >= cfg.hipL[1] + 6 || distSq(x, y, cfg.hipL) <= rHipSq) && (y <= cfg.kneeL[1] - 8 || distSq(x, y, cfg.kneeL) <= rKneeSq)) {
            copyPx('legL_thigh', idx);
          }
          if (y <= cfg.hipL[1] + 2 && x >= cfg.torsoL - 1) copyPx('torso', idx);
          if (distSq(x, y, cfg.kneeL) <= rKneeSq) copyPx('legL_shin', idx);
        } else if (y <= cfg.ankleL[1]) {
          if ((y >= cfg.kneeL[1] + 8 || distSq(x, y, cfg.kneeL) <= rKneeSq) && (y <= cfg.ankleL[1] - 6 || distSq(x, y, cfg.ankleL) <= rAnkleSq)) {
            copyPx('legL_shin', idx);
          }
          if (distSq(x, y, cfg.kneeL) <= rKneeSq) copyPx('legL_thigh', idx);
          if (distSq(x, y, cfg.ankleL) <= rAnkleSq) copyPx('legL_boot', idx);
        } else {
          copyPx('legL_boot', idx);
          if (distSq(x, y, cfg.ankleL) <= rAnkleSq) copyPx('legL_shin', idx);
        }
      } else {
        if (y <= cfg.kneeR[1]) {
          if ((y >= cfg.hipR[1] + 6 || distSq(x, y, cfg.hipR) <= rHipSq) && (y <= cfg.kneeR[1] - 8 || distSq(x, y, cfg.kneeR) <= rKneeSq)) {
            copyPx('legR_thigh', idx);
          }
          if (y <= cfg.hipR[1] + 2) copyPx('torso', idx);
          if (distSq(x, y, cfg.kneeR) <= rKneeSq) copyPx('legR_shin', idx);
        } else if (y <= cfg.ankleR[1]) {
          if ((y >= cfg.kneeR[1] + 8 || distSq(x, y, cfg.kneeR) <= rKneeSq) && (y <= cfg.ankleR[1] - 6 || distSq(x, y, cfg.ankleR) <= rAnkleSq)) {
            copyPx('legR_shin', idx);
          }
          if (distSq(x, y, cfg.kneeR) <= rKneeSq) copyPx('legR_thigh', idx);
          if (distSq(x, y, cfg.ankleR) <= rAnkleSq) copyPx('legR_boot', idx);
        } else {
          copyPx('legR_boot', idx);
          if (distSq(x, y, cfg.ankleR) <= rAnkleSq) copyPx('legR_shin', idx);
        }
      }
    }
  }

  const out = {};
  for (const name of partNames) {
    partCanvases[name].ctx.putImageData(partImgData[name], 0, 0);
    out[name] = partCanvases[name].cv;
  }
  return out;
}

const VTUBER_IMAGE_CACHE = {};
function getVtuberRigImages(rigKey) {
  if (!ASSETS.vtuberRig) return null;
  const lookupKey = (rigKey === 'bailiff' && !ASSETS.vtuberRig.bailiff) ? 'watchman' : rigKey;
  const data = ASSETS.vtuberRig[lookupKey] || ASSETS.vtuberRig.kaelen;
  if (!data) return null;
  if (!VTUBER_IMAGE_CACHE[rigKey]) {
    const bodyImg = new Image();
    const weapImg = new Image();
    const cfg = VTUBER_ANATOMY_CONFIG[lookupKey] || VTUBER_ANATOMY_CONFIG.kaelen;
    const entry = {
      bodyImg,
      weapImg,
      cfg,
      parts: null,
      pivot: data.pivot || cfg.handR,
      nativeLeft: !!data.nativeLeft
    };
    const onDecode = () => {
      if (bodyImg.complete && !entry.parts) {
        entry.parts = segmentCharacterIntoLimbs(bodyImg, cfg);
      }
      if (typeof startVtuberRigLoop === 'function') startVtuberRigLoop();
    };
    bodyImg.onload = onDecode;
    weapImg.onload = onDecode;
    bodyImg.src = data.body;
    weapImg.src = data.weapon;
    if (bodyImg.complete) {
      entry.parts = segmentCharacterIntoLimbs(bodyImg, cfg);
    }
    VTUBER_IMAGE_CACHE[rigKey] = entry;
  } else if (VTUBER_IMAGE_CACHE[rigKey].bodyImg.complete && !VTUBER_IMAGE_CACHE[rigKey].parts) {
    VTUBER_IMAGE_CACHE[rigKey].parts = segmentCharacterIntoLimbs(
      VTUBER_IMAGE_CACHE[rigKey].bodyImg,
      VTUBER_IMAGE_CACHE[rigKey].cfg
    );
  }
  return VTUBER_IMAGE_CACHE[rigKey];
}

// ============================================================================
// APPROACH B: EXPLODED 13-PART SPINE2D CUTOUT ATLAS RIG (`atlas_b`)
// Extracts 13 unoccluded, rounded-joint parts from `ASSETS.partsAtlas.kaelen` (495×276):
// `cloak`, `torso`, `head`, `armL_upper`, `armL_lower`, `armR_upper`, `armR_lower`,
// `legL_thigh`, `legL_shin`, `legL_boot`, `legR_thigh`, `legR_shin`, `legR_boot`, `weapon`
// ============================================================================
const ATLAS_ANATOMY_CONFIG = {
  kaelen: {
    head: [160, 52], neck: [160, 90], chest: [159, 136], pelvis: [158, 178],
    shoulderL: [120, 106], elbowL: [110, 148], handL: [121, 198],
    shoulderR: [182, 106], elbowR: [192, 148], handR: [202, 198],
    hipL: [142, 184], kneeL: [134, 244], ankleL: [126, 294], toeL: [120, 324],
    mirrorHipL: [142, 184], mirrorKneeL: [134, 244], mirrorAnkleL: [132, 294],
    hipR: [168, 184], kneeR: [176, 244], ankleR: [178, 294], toeR: [196, 324],
    weaponPivot: [202, 198]
  }
};

const ATLAS_POSE_TARGETS = {
  relaxed: {
    pelvisDx: 4, pelvisDy: 0, torsoAngle: 0, headAngle: 0,
    armL_upperAngle: 2, armL_lowerAngle: -6,
    armR_upperAngle: -8, armR_lowerAngle: -66,
    weaponWorldAngle: 9, weapSlide: 44,
    footLDx: 0, footRDx: 6,
    kneeBendL: -1, kneeBendR: 2,
    bootLAngle: 0, bootRAngle: 0,
    mirrorLegL: 0
  },
  sneaking: {
    pelvisDx: 8, pelvisDy: 14, torsoAngle: 14, headAngle: -12,
    armL_upperAngle: 10, armL_lowerAngle: -32,
    armR_upperAngle: -2, armR_lowerAngle: -34,
    weaponWorldAngle: 90, weapSlide: 72,
    footLDx: -26, footRDx: 14,
    kneeBendL: 2, kneeBendR: 11,
    bootLAngle: 12, bootRAngle: 0,
    mirrorLegL: 0
  },
  weapons_drawn: {
    pelvisDx: 8, pelvisDy: 16, torsoAngle: 8, headAngle: -7,
    armL_upperAngle: 8, armL_lowerAngle: -42,
    armR_upperAngle: -4, armR_lowerAngle: -102,
    weaponWorldAngle: 57, weapSlide: 78,
    footLDx: -24, footRDx: 34,
    kneeBendL: -12, kneeBendR: 12,
    bootLAngle: 0, bootRAngle: 0,
    mirrorLegL: 1
  }
};

function extractAtlasPartsForCharacter(atlasImg) {
  if (typeof document === 'undefined' || !document.createElement) return null;
  const W_A = 495, H_A = 276;
  const srcCv = document.createElement('canvas');
  srcCv.width = W_A; srcCv.height = H_A;
  const srcCtx = srcCv.getContext('2d');
  if (!srcCtx) return null;
  srcCtx.drawImage(atlasImg, 0, 0, W_A, H_A);

  // Split torso box (2..109, 90..270) into upper shoulder `cloak` vs front `torso` (breastplate + belt + quilted tunic skirt)
  const tx0 = 2, ty0 = 90, tw = 107, th = 180;
  let torsoData;
  try {
    torsoData = srcCtx.getImageData(tx0, ty0, tw, th);
  } catch (e) {
    return null;
  }
  const cloakCv = document.createElement('canvas'); cloakCv.width = tw; cloakCv.height = th;
  const torsoCv = document.createElement('canvas'); torsoCv.width = tw; torsoCv.height = th;
  const cloakCtx = cloakCv.getContext('2d');
  const torsoCtx = torsoCv.getContext('2d');
  const cloakId = cloakCtx.createImageData(tw, th);
  const torsoId = torsoCtx.createImageData(tw, th);
  const sPx = torsoData.data;
  const cPx = cloakId.data;
  const tPx = torsoId.data;

  for (let y = 0; y < th; y++) {
    for (let x = 0; x < tw; x++) {
      const idx = (y * tw + x) * 4;
      if (sPx[idx + 3] === 0) continue;
      const r = sPx[idx], g = sPx[idx + 1], b = sPx[idx + 2];
      const isCloak = (y > 119) || (x < 24 && y > 19) || (g > r + 6 && g > b + 12 && y > 25);
      // Keep only the upper shoulder/back mantle (y <= 110) so the leg silhouette matches the 3-pose sheet cleanly
      if (isCloak && y <= 110) {
        cPx[idx] = r; cPx[idx + 1] = g; cPx[idx + 2] = b; cPx[idx + 3] = sPx[idx + 3];
      }
      if (!(isCloak && y > 115)) {
        tPx[idx] = r; tPx[idx + 1] = g; tPx[idx + 2] = b; tPx[idx + 3] = sPx[idx + 3];
      }
    }
  }
  cloakCtx.putImageData(cloakId, 0, 0);
  torsoCtx.putImageData(torsoId, 0, 0);

  function makePlacedPart(srcSource, sx, sy, sw, sh, dstX, dstY, scale, flipX = false) {
    const cv = document.createElement('canvas');
    cv.width = 340; cv.height = 340;
    const c = cv.getContext('2d');
    c.imageSmoothingEnabled = true;
    const dw = Math.round(sw * scale);
    const dh = Math.round(sh * scale);
    if (flipX) {
      c.save();
      c.translate(dstX + dw, dstY);
      c.scale(-1, 1);
      c.drawImage(srcSource, sx, sy, sw, sh, 0, 0, dw, dh);
      c.restore();
    } else {
      c.drawImage(srcSource, sx, sy, sw, sh, dstX, dstY, dw, dh);
    }
    return cv;
  }

  return {
    head:              makePlacedPart(srcCv,   27,   5,  70,  73, 124,  26, 0.94),
    cloak:             makePlacedPart(cloakCv,  0,   0,  tw,  th,  98,  82, 0.95),
    torso:             makePlacedPart(torsoCv,  0,   0,  tw,  th,  98,  82, 0.95),
    armL_upper:        makePlacedPart(srcCv,  130,  11,  47,  71,  95,  92, 0.90),
    armL_lower:        makePlacedPart(srcCv,  134, 152,  45,  87,  95, 136, 0.88),
    armR_upper:        makePlacedPart(srcCv,  130,  11,  47,  71, 165,  92, 0.90, true),
    armR_lower:        makePlacedPart(srcCv,  201, 152,  42,  88, 178, 136, 0.88),
    legL_thigh:        makePlacedPart(srcCv,  278,  10,  48, 106, 120, 168, 0.86),
    legL_shin:         makePlacedPart(srcCv,  270, 128,  45,  78, 111, 226, 0.86),
    legL_boot:         makePlacedPart(srcCv,  273, 215,  31,  61, 113, 276, 0.88),
    // Mirrored left-leg parts (100% horizontal mirror of legR across x=155 for wide spread combat stance in weapons_drawn)
    legL_mirror_thigh: makePlacedPart(srcCv,  346,  10,  49, 103, 120, 168, 0.86, true),
    legL_mirror_shin:  makePlacedPart(srcCv,  359, 123,  36,  81, 118, 226, 0.86, true),
    legL_mirror_boot:  makePlacedPart(srcCv,  362, 216,  48,  54, 103, 278, 0.88, true),
    legR_thigh:        makePlacedPart(srcCv,  346,  10,  49, 103, 148, 168, 0.86),
    legR_shin:         makePlacedPart(srcCv,  359, 123,  36,  81, 161, 226, 0.86),
    legR_boot:         makePlacedPart(srcCv,  362, 216,  48,  54, 165, 278, 0.88),
    weapon:            makePlacedPart(srcCv,  435,   3,  38, 273, 180,  20, 1.04)
  };
}

const ATLAS_RIG_CACHE = {};
function getAtlasRigEntry(rigKey) {
  const baseKey = rigKey.startsWith('kaelen') ? 'kaelen' : rigKey;
  if (!ASSETS.partsAtlas || !ASSETS.partsAtlas[baseKey]) return null;
  if (!ATLAS_RIG_CACHE[baseKey]) {
    const atlasImg = new Image();
    const cfg = ATLAS_ANATOMY_CONFIG[baseKey] || ATLAS_ANATOMY_CONFIG.kaelen;
    const entry = { atlasImg, cfg, parts: null };
    atlasImg.onload = () => {
      if (!entry.parts) entry.parts = extractAtlasPartsForCharacter(atlasImg);
      if (typeof startVtuberRigLoop === 'function') startVtuberRigLoop();
    };
    atlasImg.src = ASSETS.partsAtlas[baseKey];
    if (atlasImg.complete) {
      entry.parts = extractAtlasPartsForCharacter(atlasImg);
    }
    ATLAS_RIG_CACHE[baseKey] = entry;
  } else if (ATLAS_RIG_CACHE[baseKey].atlasImg.complete && !ATLAS_RIG_CACHE[baseKey].parts) {
    ATLAS_RIG_CACHE[baseKey].parts = extractAtlasPartsForCharacter(ATLAS_RIG_CACHE[baseKey].atlasImg);
  }
  return ATLAS_RIG_CACHE[baseKey];
}

// ============================================================================
// APPROACH C: 3-POSE KEYFRAME SHEET + LIVE IDLE RIG (`pose_c`)
// Uses dedicated hand-drawn keyframe sprites (`relaxed`, `sneaking`, `weapons_drawn`)
// split into upper/lower body deformers with live breathing, knee suspension,
// steel flash, and snappy squash-and-stretch transitions when changing stances.
// ============================================================================
const POSE_SHEET_CACHE = {};
function getPoseSheetEntry(rigKey) {
  const baseKey = rigKey.startsWith('kaelen') ? 'kaelen' : rigKey;
  if (!ASSETS.poseSheets || !ASSETS.poseSheets[baseKey]) return null;
  if (!POSE_SHEET_CACHE[baseKey]) {
    const srcMap = ASSETS.poseSheets[baseKey];
    const entry = { images: {}, layers: {} };
    ['relaxed', 'sneaking', 'weapons_drawn'].forEach(poseName => {
      const img = new Image();
      img.onload = () => {
        entry.layers[poseName] = splitPoseSpriteUpperLower(img, poseName);
        if (typeof startVtuberRigLoop === 'function') startVtuberRigLoop();
      };
      img.src = srcMap[poseName];
      if (img.complete) {
        entry.layers[poseName] = splitPoseSpriteUpperLower(img, poseName);
      }
      entry.images[poseName] = img;
    });
    POSE_SHEET_CACHE[baseKey] = entry;
  } else {
    const entry = POSE_SHEET_CACHE[baseKey];
    ['relaxed', 'sneaking', 'weapons_drawn'].forEach(poseName => {
      if (entry.images[poseName] && entry.images[poseName].complete && !entry.layers[poseName]) {
        entry.layers[poseName] = splitPoseSpriteUpperLower(entry.images[poseName], poseName);
      }
    });
  }
  return POSE_SHEET_CACHE[baseKey];
}

function splitPoseSpriteUpperLower(img, poseName) {
  if (typeof document === 'undefined' || !document.createElement) return null;
  const W = 340, H = 340;
  // Per-pose anatomical deformer anchors & non-overlapping head bounding boxes on the 340x340 pose sprites
  const meta = poseName === 'relaxed'
    ? { hipX: 162, hipY: 196, neckX: 160, neckY: 92,  headBox: [115, 24, 195, 96],  kneeY: 262 }
    : (poseName === 'sneaking'
      ? { hipX: 168, hipY: 214, neckX: 186, neckY: 134, headBox: [135, 68, 225, 138], kneeY: 266 }
      : { hipX: 172, hipY: 214, neckX: 178, neckY: 128, headBox: [135, 66, 212, 132], kneeY: 266 });

  const { hipX, hipY, neckX, neckY, headBox, kneeY } = meta;
  const [hx0, hy0, hx1, hy1] = headBox;

  const upCv = document.createElement('canvas'); upCv.width = W; upCv.height = H;
  const loCv = document.createElement('canvas'); loCv.width = W; loCv.height = H;
  const legLCv = document.createElement('canvas'); legLCv.width = W; legLCv.height = H;
  const legRCv = document.createElement('canvas'); legRCv.width = W; legRCv.height = H;
  const torsoCv = document.createElement('canvas'); torsoCv.width = W; torsoCv.height = H;
  const headCv = document.createElement('canvas'); headCv.width = W; headCv.height = H;

  const upCtx = upCv.getContext('2d');
  const loCtx = loCv.getContext('2d');
  const legLCtx = legLCv.getContext('2d');
  const legRCtx = legRCv.getContext('2d');
  const torsoCtx = torsoCv.getContext('2d');
  const headCtx = headCv.getContext('2d');
  if (!upCtx || !loCtx || !legLCtx || !legRCtx || !torsoCtx || !headCtx) return null;

  // 1. Full upper body (with 8px waist overlap across hipY)
  upCtx.drawImage(img, 0, 0, W, hipY + 8, 0, 0, W, hipY + 8);
  // 2. Lower body (with 6px waist overlap above hipY for seamless scanline strip-mesh warping)
  loCtx.drawImage(img, 0, hipY - 6, W, H - (hipY - 6), 0, hipY - 6, W, H - (hipY - 6));

  // 3. Torso + Arms + Weapon (head region above neckY - 3 cleared so independent head nod/stabilizer never ghosts)
  torsoCtx.drawImage(img, 0, 0, W, hipY + 8, 0, 0, W, hipY + 8);
  torsoCtx.clearRect(hx0, hy0, hx1 - hx0, (neckY - 3) - hy0);

  // Keep any weapon shaft/tip pixels that dip below hipY on `torsoOnly` (and clear from `lower`) so the polearm stays 100% rigid!
  if (poseName === 'relaxed') {
    torsoCtx.drawImage(img, 194, hipY, W - 194, 314 - hipY, 194, hipY, W - 194, 314 - hipY);
    loCtx.clearRect(194, hipY - 6, W - 194, 314 - (hipY - 6));
  } else if (poseName === 'sneaking') {
    torsoCtx.drawImage(img, 214, hipY, W - 214, 216 - hipY, 214, hipY, W - 214, 216 - hipY);
    loCtx.clearRect(214, hipY - 6, W - 214, 216 - (hipY - 6));
  } else if (poseName === 'weapons_drawn') {
    torsoCtx.drawImage(img, 0, hipY, 113, 234 - hipY, 0, hipY, 113, 234 - hipY);
    loCtx.clearRect(0, hipY - 6, 113, 234 - (hipY - 6));
  }

  // Split `loCv` into Left Leg (`legLCv`) and Right Leg (`legRCv`) along the zero-collision air gap between the legs
  for (let y = hipY - 6; y < H; y++) {
    const sx = poseName === 'relaxed'
      ? Math.round(154 - Math.max(0, y - 218) * 0.08)
      : (poseName === 'sneaking'
        ? Math.round(170 - Math.max(0, y - 230) * 0.20)
        : 167);
    legLCtx.drawImage(loCv, 0, y, sx, 1, 0, y, sx, 1);
    legRCtx.drawImage(loCv, sx, y, W - sx, 1, sx, y, W - sx, 1);
  }

  // 4. Independent Head + Straw-Blond Hair (with 5px collar overlap down to neckY + 5)
  headCtx.drawImage(img, hx0, hy0, hx1 - hx0, (neckY + 5) - hy0, hx0, hy0, hx1 - hx0, (neckY + 5) - hy0);

  return {
    upper: upCv,
    lower: loCv,
    legL: legLCv,
    legR: legRCv,
    torsoOnly: torsoCv,
    headOnly: headCv,
    hipX, hipY, neckX, neckY, kneeY, headBox
  };
}

// Pose targets for all 4 limbs + spine + head + 2-handed polearm grip:
//  - `1. Relaxed`       : Upright stance, one-handed vertical billhook at side, back arm resting at side.
//  - `2. Sneaking`      : Deep stealth crouch with 2-bone IK knee bend; BOTH hands grip the billhook
//                         horizontally (`weaponAngle = 80°` + `10°` native = `90°` parallel to the ground!).
//  - `3. Weapons Drawn` : Wide martial combat stance with 2-bone IK knee bend; BOTH hands grip the
//                         billhook diagonally (`weaponAngle = 48°` + `10°` native = `58°` guard across chest!).
const VTUBER_POSE_TARGETS = {
  relaxed: {
    label: '1. Relaxed',
    crouchDrop: 1.0,
    pelvisShiftX: 0,
    spineAngle: -1.0,
    headAngle: 1.0,
    backStepX: 0,
    frontStepX: 0,
    shoulderLAngle: 2.0,
    elbowLAngle: -6.0,
    shoulderRAngle: 2.0,
    elbowRAngle: -4.0,
    weaponAngle: -2.0,
    weapSlide: 0.0,
    lungeOffset: 0.0,
    flinchOffset: 0.0,
    breathAmp: 2.0,
    breathFreq: 1.85
  },
  sneaking: {
    label: '2. Sneaking',
    crouchDrop: 22.0,       // Deep stealth crouch — 2-bone IK bends BOTH knees forward without squishing!
    pelvisShiftX: 8.0,
    spineAngle: 15.0,
    headAngle: -14.0,
    backStepX: -14.0,
    frontStepX: 12.0,
    shoulderLAngle: -6.0,   // Back arm brings bone-spiked bracer hand onto the rear horizontal shaft
    elbowLAngle: -22.0,
    shoulderRAngle: 24.0,   // Front arm lowers hand to waist level to hold the horizontal shaft
    elbowRAngle: 16.0,
    weaponAngle: 80.0,      // 80° + 10° rest shaft angle = 90.0° (100% horizontal / parallel to ground!)
    weapSlide: 18.0,        // Slides shaft back through both gripping hands
    lungeOffset: 0.0,
    flinchOffset: 0.0,
    breathAmp: 1.3,
    breathFreq: 1.35
  },
  weapons_drawn: {
    label: '3. Weapons Drawn',
    crouchDrop: 15.0,       // Martial combat crouch — both knees bend via 2-bone IK!
    pelvisShiftX: 8.0,
    spineAngle: 10.0,
    headAngle: -9.0,
    backStepX: -15.0,
    frontStepX: 15.0,
    shoulderLAngle: -10.0,  // Back hand grips the lower shaft across the waist
    elbowLAngle: -26.0,
    shoulderRAngle: -12.0,  // Front hand raises the upper shaft across the chest
    elbowRAngle: -28.0,
    weaponAngle: 48.0,      // 48° + 10° rest shaft angle = 58.0° two-handed diagonal combat guard!
    weapSlide: 26.0,        // Slides shaft diagonally through both gripping hands
    lungeOffset: 0.0,
    flinchOffset: 0.0,
    breathAmp: 2.6,
    breathFreq: 2.85
  }
};

const VTUBER_SPRING_KEYS = [
  'crouchDrop', 'pelvisShiftX', 'spineAngle', 'headAngle',
  'backStepX', 'frontStepX',
  'shoulderLAngle', 'elbowLAngle',
  'shoulderRAngle', 'elbowRAngle',
  'weaponAngle', 'weapSlide',
  'lungeOffset', 'flinchOffset'
];

const VTUBER_SLOT_SPRINGS = {};
function getVtuberSlotSpring(slotId, targetPose) {
  const tgt = VTUBER_POSE_TARGETS[targetPose] || VTUBER_POSE_TARGETS.relaxed;
  if (!VTUBER_SLOT_SPRINGS[slotId]) {
    const init = {
      pose: targetPose,
      prevPose: targetPose,
      poseTransitionT: 1.0,
      glintProgress: targetPose === 'weapons_drawn' ? 0.15 : 1.5
    };
    for (const k of VTUBER_SPRING_KEYS) {
      init[k] = tgt[k] ?? 0;
      init['v_' + k] = 0;
    }
    VTUBER_SLOT_SPRINGS[slotId] = init;
  }
  const s = VTUBER_SLOT_SPRINGS[slotId];
  if (s.pose !== targetPose) {
    s.prevPose = s.pose;
    s.poseTransitionT = 0.0;
    // Inject crisp physical limb & deformer impulses when transitioning between stances
    if (targetPose === 'weapons_drawn') {
      s.v_weaponAngle += 220;
      s.v_shoulderLAngle -= 90;
      s.v_crouchDrop += 95;
      s.v_lungeOffset += 165;
      s.glintProgress = 0.0;
    } else if (targetPose === 'sneaking') {
      s.v_crouchDrop += 110;
      s.v_weaponAngle += 180;
      s.v_spineAngle += 70;
      s.v_lungeOffset += 95;
      s.glintProgress = 0.0;
    } else {
      s.v_crouchDrop -= 75;
    }
    s.pose = targetPose;
  }
  return s;
}

// Interactive Limit-Test Impulse Trigger (`'lunge'` | `'flinch'`) for testing Deformer Rig physical extremes
function triggerStoryRigAction(actionType) {
  Object.values(VTUBER_SLOT_SPRINGS).forEach(s => {
    if (!s) return;
    if (actionType === 'lunge') {
      s.v_lungeOffset = (s.v_lungeOffset || 0) + 360;
      s.v_crouchDrop  = (s.v_crouchDrop  || 0) + 140;
      s.v_spineAngle  = (s.v_spineAngle  || 0) + 120;
      s.poseTransitionT = 0.05;
      s.glintProgress = 0.0;
    } else if (actionType === 'flinch') {
      s.v_flinchOffset = (s.v_flinchOffset || 0) + 380;
      s.v_crouchDrop   = (s.v_crouchDrop   || 0) + 95;
      s.v_spineAngle   = (s.v_spineAngle   || 0) - 140;
      s.poseTransitionT = 0.05;
    }
  });
}

function stepVtuberSpring(s, tgt, dt) {
  const k = 150;  // Spring stiffness
  const d = 14.2; // Damping coefficient (allows crisp spring overshoot on lunges/flinches)
  for (const key of VTUBER_SPRING_KEYS) {
    const vKey = 'v_' + key;
    const targetVal = tgt[key] ?? 0;
    const force = (targetVal - (s[key] ?? 0)) * k - (s[vKey] ?? 0) * d;
    s[vKey] = (s[vKey] ?? 0) + force * dt;
    s[key]  = (s[key]  ?? 0) + s[vKey] * dt;
  }
  if (s.poseTransitionT < 1.0) {
    s.poseTransitionT = Math.min(1.0, s.poseTransitionT + dt * 4.8);
  }
  if (s.glintProgress < 1.4) {
    s.glintProgress += dt * 1.35;
  } else if (s.pose === 'weapons_drawn' && Math.random() < dt * 0.45) {
    s.glintProgress = 0.0;
  }
}

// Analytic 2-Bone Inverse Kinematics Solver (Law of Cosines)
// Given animated `hip = [hx, hy]`, planted `ankle = [ax, ay]`, rigid bone lengths `L1` (thigh)
// and `L2` (shin), and `bendSign` (+1 to bend knee right, -1 to bend knee left), returns
// the exact `knee = [kx, ky]` so neither thigh nor shin EVER squishes or changes length!
function solveTwoBoneIK(hip, ankle, L1, L2, bendSign) {
  const dx = ankle[0] - hip[0];
  const dy = ankle[1] - hip[1];
  const distRaw = Math.hypot(dx, dy);
  const maxReach = (L1 + L2) * 0.996;
  const minReach = Math.abs(L1 - L2) + 2.0;
  const D = Math.max(minReach, Math.min(maxReach, distRaw));

  const cosAlpha = Math.max(-1, Math.min(1, (L1 * L1 + D * D - L2 * L2) / (2 * L1 * D)));
  const alpha = Math.acos(cosAlpha);
  const baseAngle = Math.atan2(dy, dx);
  const thighAngle = baseAngle - bendSign * alpha;
  const kx = hip[0] + Math.cos(thighAngle) * L1;
  const ky = hip[1] + Math.sin(thighAngle) * L1;
  return [kx, ky];
}

function rotatePtAround(pt, origin, rad) {
  const dx = pt[0] - origin[0];
  const dy = pt[1] - origin[1];
  const c = Math.cos(rad);
  const s = Math.sin(rad);
  return [
    origin[0] + dx * c - dy * s,
    origin[1] + dx * s + dy * c
  ];
}

function drawArticulatedPart(ctx, partCv, restPivot, animPivot, deltaRad, scaleX = 1.0, scaleY = 1.0) {
  if (!partCv) return;
  ctx.save();
  ctx.translate(animPivot[0], animPivot[1]);
  ctx.rotate(deltaRad);
  if (scaleX !== 1.0 || scaleY !== 1.0) {
    ctx.scale(scaleX, scaleY);
  }
  ctx.drawImage(partCv, -restPivot[0], -restPivot[1]);
  ctx.restore();
}

function drawBonePartAtoB(ctx, partCv, restA, restB, animA, animB) {
  if (!partCv) return;
  const restAngle = Math.atan2(restB[1] - restA[1], restB[0] - restA[0]);
  const animAngle = Math.atan2(animB[1] - animA[1], animB[0] - animA[0]);
  const deltaRad = animAngle - restAngle;
  drawArticulatedPart(ctx, partCv, restA, animA, deltaRad);
}

function computeAtlasKneeWithBend(hip, ankle, kneeBendPx) {
  const dx = ankle[0] - hip[0];
  const dy = ankle[1] - hip[1];
  const dist = Math.max(1.0, Math.hypot(dx, dy));
  const mx = hip[0] + dx * 0.54;
  const my = hip[1] + dy * 0.54;
  // Normal pointing to the RIGHT of the downward hip->ankle vector: (dy/dist, -dx/dist)
  // Positive kneeBendPx bends knee to the right; negative kneeBendPx bends knee outward to the left!
  const nx = dy / dist;
  const ny = -dx / dist;
  return [mx + nx * kneeBendPx, my + ny * kneeBendPx];
}

// ============================================================================
// RENDERER FOR APPROACH B: EXPLODED 13-PART SPINE2D ATLAS (`drawAtlasRigOnCanvas`)
// ============================================================================
function drawAtlasRigOnCanvas(ctx, atlasEntry, s, tgt, tSec, side, isActive, showDebug) {
  const W = 340, H = 340;
  ctx.clearRect(0, 0, W, H);
  ctx.imageSmoothingEnabled = false;

  const cfg = atlasEntry.cfg;
  const parts = atlasEntry.parts;
  if (!parts) return;

  const shouldFlip = side === 'right';
  const phaseOffset = side === 'left' ? 0.0 : 1.7;
  const breath = Math.sin(tSec * tgt.breathFreq + phaseOffset);
  const talkBob = isActive ? Math.sin(tSec * 6.5) * 0.9 : 0;

  // Interpolate smoothly using spring progress toward the active ATLAS_POSE_TARGETS
  const pTgt = ATLAS_POSE_TARGETS[s.pose] || ATLAS_POSE_TARGETS.relaxed;
  const pPrev = ATLAS_POSE_TARGETS[s.prevPose] || pTgt;
  const blend = Math.min(1.0, Math.max(0.0, 1.0 - Math.pow(1.0 - (s.poseTransitionT ?? 1.0), 3)));
  function lerpP(key) {
    return (pPrev[key] ?? 0) + ((pTgt[key] ?? 0) - (pPrev[key] ?? 0)) * blend;
  }

  const isWalking = (typeof STATE !== 'undefined' && STATE.storyWalkMode && STATE.storyWalkMode !== 'off') && (side === 'left' || STATE.storyWalkMode === 'scroll_bg');
  const walkFreq = s.pose === 'sneaking' ? 4.4 : 5.5;
  const phiL = tSec * walkFreq + phaseOffset;
  const phiR = phiL + Math.PI;
  const walkBobDy = isWalking ? Math.sin(phiL * 2 - 0.5) * 3.0 : 0;
  const walkArmSwing = isWalking && s.pose === 'relaxed' ? Math.sin(phiR) * 11.0 : (isWalking ? Math.sin(phiR) * 3.5 : 0);

  const pelvisDx = lerpP('pelvisDx') + (isWalking ? Math.cos(phiL * 2) * 1.5 : 0);
  const pelvisDy = lerpP('pelvisDy') + Math.max(0, (s.crouchDrop - tgt.crouchDrop) * 0.25) + walkBobDy;
  const torsoDeg = lerpP('torsoAngle') + breath * 1.1 + (isWalking ? Math.cos(phiL * 2) * 1.4 : 0);
  const headDeg  = lerpP('headAngle') + talkBob * 1.3 - (isWalking ? Math.cos(phiL * 2) * 1.1 : 0);
  const torsoRad = (torsoDeg * Math.PI) / 180;
  const headRad  = ((torsoDeg + headDeg) * Math.PI) / 180;

  const animPelvis    = [cfg.pelvis[0] + pelvisDx, cfg.pelvis[1] + pelvisDy];
  const animChest     = rotatePtAround([cfg.chest[0] + pelvisDx, cfg.chest[1] + pelvisDy - breath * tgt.breathAmp * 0.6], animPelvis, torsoRad);
  const animNeck      = rotatePtAround([cfg.neck[0]  + pelvisDx, cfg.neck[1]  + pelvisDy - breath * tgt.breathAmp], animPelvis, torsoRad);
  const animHead      = rotatePtAround([animNeck[0] + (cfg.head[0] - cfg.neck[0]), animNeck[1] + (cfg.head[1] - cfg.neck[1])], animNeck, headRad);
  const animShoulderL = rotatePtAround([cfg.shoulderL[0] + pelvisDx, cfg.shoulderL[1] + pelvisDy - breath * tgt.breathAmp * 0.8], animPelvis, torsoRad);
  const animShoulderR = rotatePtAround([cfg.shoulderR[0] + pelvisDx, cfg.shoulderR[1] + pelvisDy - breath * tgt.breathAmp * 0.8], animPelvis, torsoRad);

  const aLU_deg = torsoDeg + lerpP('armL_upperAngle') + breath * 1.0 + walkArmSwing;
  const aLL_deg = aLU_deg  + lerpP('armL_lowerAngle');
  const aRU_deg = torsoDeg + lerpP('armR_upperAngle') - breath * 1.0 - walkArmSwing * 0.65;
  const aRL_deg = aRU_deg  + lerpP('armR_lowerAngle');
  const aLU_rad = (aLU_deg * Math.PI) / 180;
  const aLL_rad = (aLL_deg * Math.PI) / 180;
  const aRU_rad = (aRU_deg * Math.PI) / 180;
  const aRL_rad = (aRL_deg * Math.PI) / 180;

  const elL_b = rotatePtAround([cfg.elbowL[0] + pelvisDx, cfg.elbowL[1] + pelvisDy - breath * tgt.breathAmp * 0.8], animPelvis, torsoRad);
  const animElbowL = rotatePtAround(elL_b, animShoulderL, ((lerpP('armL_upperAngle') + walkArmSwing) * Math.PI) / 180);
  const animHandL  = rotatePtAround([animElbowL[0] + (cfg.handL[0] - cfg.elbowL[0]), animElbowL[1] + (cfg.handL[1] - cfg.elbowL[1])], animElbowL, aLL_rad);

  const elR_b = rotatePtAround([cfg.elbowR[0] + pelvisDx, cfg.elbowR[1] + pelvisDy - breath * tgt.breathAmp * 0.8], animPelvis, torsoRad);
  const animElbowR = rotatePtAround(elR_b, animShoulderR, ((lerpP('armR_upperAngle') - walkArmSwing * 0.65) * Math.PI) / 180);
  const animHandR  = rotatePtAround([animElbowR[0] + (cfg.handR[0] - cfg.elbowR[0]), animElbowR[1] + (cfg.handR[1] - cfg.elbowR[1])], animElbowR, aRL_rad);

  // Lock weapon grip pivot directly to the animated right hand (`animHandR`) so the shaft never separates from the glove
  const wp = animHandR;

  const useMirroredLeftLeg = !isWalking && (lerpP('mirrorLegL') >= 0.5);
  const restHipL   = useMirroredLeftLeg ? cfg.mirrorHipL   : cfg.hipL;
  const restKneeL  = useMirroredLeftLeg ? cfg.mirrorKneeL  : cfg.kneeL;
  const restAnkleL = useMirroredLeftLeg ? cfg.mirrorAnkleL : cfg.ankleL;

  const strideAmp = s.pose === 'sneaking' ? 14.0 : 17.5;
  const liftAmp   = s.pose === 'sneaking' ? 7.5  : 9.5;
  const walkDxL   = isWalking ? Math.sin(phiL) * strideAmp : 0;
  const walkDxR   = isWalking ? Math.sin(phiR) * strideAmp : 0;
  const walkLiftL = isWalking ? -Math.max(0, Math.cos(phiL)) * liftAmp : 0;
  const walkLiftR = isWalking ? -Math.max(0, Math.cos(phiR)) * liftAmp : 0;

  const animHipL = rotatePtAround([restHipL[0] + pelvisDx, restHipL[1] + pelvisDy], animPelvis, torsoRad * 0.35);
  const animHipR = rotatePtAround([cfg.hipR[0] + pelvisDx, cfg.hipR[1] + pelvisDy], animPelvis, torsoRad * 0.35);
  const animAnkleL = [restAnkleL[0] + lerpP('footLDx') * (isWalking ? 0.35 : 1.0) + walkDxL, restAnkleL[1] + walkLiftL];
  const animAnkleR = [cfg.ankleR[0] + lerpP('footRDx') * (isWalking ? 0.35 : 1.0) + walkDxR, cfg.ankleR[1] + walkLiftR];
  const animToeL   = [cfg.toeL[0]   + lerpP('footLDx') * (isWalking ? 0.35 : 1.0) + walkDxL, cfg.toeL[1]   + walkLiftL];
  const animToeR   = [cfg.toeR[0]   + lerpP('footRDx') * (isWalking ? 0.35 : 1.0) + walkDxR, cfg.toeR[1]   + walkLiftR];

  // Signed perpendicular knee bend (with dynamic forward knee bend during walking leg lift!)
  const walkKneeBendL = isWalking ? (4.0 + Math.max(0, Math.cos(phiL)) * 12.0) : lerpP('kneeBendL');
  const walkKneeBendR = isWalking ? (5.0 + Math.max(0, Math.cos(phiR)) * 12.0) : lerpP('kneeBendR');
  const animKneeL = computeAtlasKneeWithBend(animHipL, animAnkleL, walkKneeBendL);
  const animKneeR = computeAtlasKneeWithBend(animHipR, animAnkleR, walkKneeBendR);

  const weapDeg = lerpP('weaponWorldAngle') + breath * 0.9 - walkArmSwing * 0.35;
  const weapRad = (weapDeg * Math.PI) / 180;
  const weapSlide = lerpP('weapSlide');
  const animPx = wp[0] - Math.sin(weapRad) * weapSlide;
  const animPy = wp[1] + Math.cos(weapRad) * weapSlide;

  ctx.save();
  if (shouldFlip) {
    ctx.translate(W, 0);
    ctx.scale(-1, 1);
  }

  // Painter's order for 13-part Spine2D Atlas:
  // 1. Back Shoulder Mantle (`cloak`)
  // 2. Far Upper Arm (`armR_upper`)
  // 3. Left & Right Legs (`thigh -> shin -> boot`, using mirrored left leg in wide combat stance)
  // 4. Unoccluded Torso & Tunic Skirt (`torso`)
  // 5. Head (`head`)
  // 6. Near Upper Arm (`armL_upper`) covering left breastplate armhole
  // 7. Standalone Billhook Weapon (`weapon`)
  // 8. Far Forearm/Hand (`armR_lower`) & Near Forearm/Hand (`armL_lower`) clasping shaft
  drawArticulatedPart(ctx, parts.cloak, cfg.pelvis, animPelvis, torsoRad * 0.55);
  drawArticulatedPart(ctx, parts.armR_upper, cfg.shoulderR, animShoulderR, aRU_rad);

  const thighLPart = useMirroredLeftLeg ? parts.legL_mirror_thigh : parts.legL_thigh;
  const shinLPart  = useMirroredLeftLeg ? parts.legL_mirror_shin  : parts.legL_shin;
  const bootLPart  = useMirroredLeftLeg ? parts.legL_mirror_boot  : parts.legL_boot;

  const bootLPitch = isWalking ? (Math.sin(phiL) * 10 - Math.max(0, Math.cos(phiL)) * 12) : lerpP('bootLAngle');
  const bootRPitch = isWalking ? (Math.sin(phiR) * 10 - Math.max(0, Math.cos(phiR)) * 12) : lerpP('bootRAngle');

  drawBonePartAtoB(ctx, thighLPart, restHipL,  restKneeL,  animHipL,  animKneeL);
  drawBonePartAtoB(ctx, shinLPart,  restKneeL, restAnkleL, animKneeL, animAnkleL);
  drawArticulatedPart(ctx, bootLPart, restAnkleL, animAnkleL, (bootLPitch * Math.PI) / 180);

  drawBonePartAtoB(ctx, parts.legR_thigh, cfg.hipR,  cfg.kneeR,  animHipR,  animKneeR);
  drawBonePartAtoB(ctx, parts.legR_shin,  cfg.kneeR, cfg.ankleR, animKneeR, animAnkleR);
  drawArticulatedPart(ctx, parts.legR_boot, cfg.ankleR, animAnkleR, (bootRPitch * Math.PI) / 180);

  const chestScaleX = 1.0 + Math.max(0, breath) * 0.018;
  const chestScaleY = 1.0 + Math.max(0, breath) * 0.012;
  drawArticulatedPart(ctx, parts.torso, cfg.pelvis, animPelvis, torsoRad, chestScaleX, chestScaleY);
  drawArticulatedPart(ctx, parts.head,  cfg.neck,   animNeck,   headRad);

  drawArticulatedPart(ctx, parts.armL_upper, cfg.shoulderL, animShoulderL, aLU_rad);
  drawArticulatedPart(ctx, parts.weapon,     cfg.weaponPivot, [animPx, animPy], weapRad);
  drawArticulatedPart(ctx, parts.armR_lower, cfg.elbowR, animElbowR, aRL_rad);
  drawArticulatedPart(ctx, parts.armL_lower, cfg.elbowL, animElbowL, aLL_rad);

  // Specular blade glint
  if (s.pose === 'weapons_drawn' && s.glintProgress >= 0 && s.glintProgress <= 1.0) {
    ctx.save();
    ctx.translate(animPx, animPy);
    ctx.rotate(weapRad);
    const g = s.glintProgress;
    const gx = 4;
    const gy = -20 - g * 120;
    ctx.strokeStyle = 'rgba(254, 243, 199, 0.92)';
    ctx.lineWidth = 3.0;
    ctx.shadowColor = '#f59e0b';
    ctx.shadowBlur = 10;
    ctx.beginPath();
    ctx.moveTo(gx - 8, gy + 18);
    ctx.lineTo(gx + 8, gy - 18);
    ctx.stroke();
    ctx.restore();
  }

  if (showDebug) {
    drawSkeletonDebugOverlay(ctx, {
      animPelvis, animChest, animNeck, animHead,
      animHipL, animKneeL, animAnkleL, animToeL,
      animHipR, animKneeR, animAnkleR, animToeR,
      animShoulderL, animElbowL, animHandL,
      animShoulderR, animElbowR, animHandR,
      animPx, animPy, rad: weapRad
    });
  }

  ctx.restore();
}

// ============================================================================
// RENDERER FOR APPROACH C: 3-POSE KEYFRAME SHEET + MULTI-ZONE DEFORMER RIG (`drawPoseSheetRigOnCanvas`)
// ============================================================================
function drawPoseSheetRigOnCanvas(ctx, poseEntry, s, tgt, tSec, side, isActive, showDebug) {
  const W = 340, H = 340;
  ctx.clearRect(0, 0, W, H);
  ctx.imageSmoothingEnabled = false;

  const activePose = s.pose || 'relaxed';
  const layer = poseEntry.layers[activePose] || poseEntry.layers.relaxed;
  const rawImg = poseEntry.images[activePose] || poseEntry.images.relaxed;
  if (!rawImg || !rawImg.complete) return;

  const shouldFlip = side === 'right';
  const phaseOffset = side === 'left' ? 0.0 : 1.7;
  const boost = (typeof STATE !== 'undefined' && STATE.storyDeformerBoost) ? STATE.storyDeformerBoost : 1.0;
  const isWalking = (typeof STATE !== 'undefined' && STATE.storyWalkMode && STATE.storyWalkMode !== 'off') && (side === 'left' || STATE.storyWalkMode === 'scroll_bg');
  const walkScale = Math.min(1.65, 0.82 + boost * 0.22);

  // Walking / Sneak-Walking gait phase (Left Leg `phiL`, Right Leg `phiR` 180° opposite)
  const walkFreq = activePose === 'sneaking' ? 4.4 : 5.5;
  const phiL = tSec * walkFreq + phaseOffset;
  const phiR = phiL + Math.PI;
  const walkBobDy   = isWalking ? Math.sin(phiL * 2 - 0.45) * 3.1 * walkScale : 0;
  const walkSurgeDx = isWalking ? Math.cos(phiL * 2) * 1.6 * walkScale : 0;

  // Multi-harmonic physiological waveforms (breathing + 2.5D weight shift + speaking cadence + hair breeze)
  const breath   = Math.sin(tSec * tgt.breathFreq + phaseOffset);
  const swayWave = Math.sin(tSec * (tgt.breathFreq * 0.55) + phaseOffset + 0.8);
  const hairWave = Math.sin(tSec * 3.4 + phaseOffset + 1.4);
  const talkBob  = isActive ? Math.sin(tSec * 6.8) * 1.15 : 0;

  // Live physical spring impulses from stance transitions, Combat Lunge (`⚔️ Lunge`), and Hit Flinch (`💥 Flinch`)
  const crouchSpringDelta = (s.crouchDrop ?? tgt.crouchDrop) - tgt.crouchDrop;
  const lungePx  = (s.lungeOffset  ?? 0) * (0.28 * Math.min(1.8, boost * 0.75));
  const flinchPx = (s.flinchOffset ?? 0) * (0.28 * Math.min(1.8, boost * 0.75));

  // Snappy anticipation squash-and-stretch impulse when switching between keyframe poses!
  const transT = s.poseTransitionT ?? 1.0;
  const squashWave = transT < 1.0 ? Math.sin(transT * Math.PI) : 0.0;
  const squashScale = Math.min(2.2, 0.85 + boost * 0.35);
  const squashX = 1.0 + squashWave * 0.075 * squashScale;
  const squashY = 1.0 - squashWave * 0.068 * squashScale;
  const hopY    = -squashWave * 5.0 * squashScale;

  ctx.save();
  if (shouldFlip) {
    ctx.translate(W, 0);
    ctx.scale(-1, 1);
  }

  // Apply foot-anchored squash & stretch during pose transitions
  ctx.translate(170, 325 + hopY);
  ctx.scale(squashX, squashY);
  ctx.translate(-170, -325);

  // 0. Cross-pose directional motion smear ghost (`prevPose`) during the first 50% of a stance change
  if (transT < 0.52 && s.prevPose && s.prevPose !== activePose) {
    const prevImg = poseEntry.images[s.prevPose];
    if (prevImg && prevImg.complete) {
      ctx.save();
      ctx.globalAlpha = (1.0 - transT / 0.52) * 0.36;
      const smearShiftX = activePose === 'relaxed' ? -10 : 12;
      ctx.transform(1, 0, smearShiftX * 0.012, 1, -smearShiftX * (1.0 - transT), 0);
      ctx.drawImage(prevImg, 0, 0);
      ctx.restore();
    }
  }

  if (layer) {
    const hipX  = layer.hipX  || 166;
    const hipY  = layer.hipY  || 210;
    const neckX = layer.neckX || 174;
    const neckY = layer.neckY || 120;
    const footY = 325;

    // Waist displacement: breathing suspension + 2.5D weight shift + walking gait bounce + lunge/flinch impulse
    const breathCrouchDy = (activePose === 'relaxed' ? -breath * 1.6 : (0.9 - breath) * 2.2) * boost;
    const waistDy = Math.max(-14, Math.min(22,
      breathCrouchDy + walkBobDy + crouchSpringDelta * 0.32 * Math.min(2.0, boost * 0.85) + Math.abs(lungePx) * 0.18 + Math.abs(flinchPx) * 0.12
    ));
    const waistDx = (swayWave * (activePose === 'relaxed' ? 1.8 : 2.8) * boost) + walkSurgeDx + lungePx - flinchPx * 0.85;

    // Mid-leg knee bulge & wide-stance knee spread driven by waist crouch suspension
    const kneeFlexSign = activePose === 'weapons_drawn' ? 0.35 : 1.0;
    const kneeBulgeX = (waistDy * 0.62 + Math.max(0, -breath) * (activePose === 'sneaking' ? 3.2 : 1.8) * boost) * kneeFlexSign + lungePx * 0.35;
    const kneeSpreadScale = activePose === 'weapons_drawn'
      ? (waistDy * 0.0055 + Math.max(0, -breath) * 0.016 * boost + Math.abs(lungePx) * 0.0025)
      : (waistDy * 0.0022);

    // ========================================================================
    // ZONE 1: LOWER BODY 2px SCANLINE STRIP-MESH WARP (Stationary Knee Flex OR 180° Alternating Walking Stride!)
    // ========================================================================
    const stripStep = 2;
    const yStart = hipY - 4;
    const totalLegH = Math.max(1, footY - hipY);

    const strideAmp     = isWalking ? (activePose === 'sneaking' ? 12.5 : (activePose === 'weapons_drawn' ? 11.0 : 14.5)) * walkScale : 0;
    const liftHeight    = isWalking ? (activePose === 'sneaking' ? 7.5  : 9.5) * walkScale : 0;
    const stepKneeBulge = isWalking ? (activePose === 'sneaking' ? 8.5  : 7.5) * walkScale : 0;

    if (isWalking && layer.legL && layer.legR) {
      // Render Left Leg (`legL`) and Right Leg (`legR`) with 180°-out-of-phase stride & foot-lift trajectories
      const legsToDraw = [
        { cv: layer.legL, phi: phiL },
        { cv: layer.legR, phi: phiR }
      ];
      for (const leg of legsToDraw) {
        const strideSwing = Math.sin(leg.phi);              // -1 (back push-off) .. +1 (forward step)
        const liftWave    = Math.max(0, Math.cos(leg.phi)); // >0 during forward swing in air, 0 during grounded stance
        for (let sy = yStart; sy < H; sy += stripStep) {
          const sh = Math.min(stripStep, H - sy);
          const u = Math.max(0.0, Math.min(1.0, (footY - sy) / totalLegH));
          const kneeArch = Math.sin(u * Math.PI);
          // Smoothly lock the top 16% pelvis/tunic band (`u > 0.86`) to `(waistDx, waistDy)` so the waist never splits!
          const splitBlend = Math.max(0.0, Math.min(1.0, (0.88 - u) / 0.16));

          const rowDx = waistDx * Math.pow(u, 1.15)
            + (strideSwing * strideAmp * Math.pow(1.0 - u, 1.05) + (kneeBulgeX + liftWave * stepKneeBulge) * kneeArch) * splitBlend;
          const rowDy = waistDy * u - (liftWave * liftHeight * Math.pow(1.0 - u, 1.25)) * splitBlend;

          ctx.drawImage(leg.cv, 0, sy, W, sh, rowDx, sy + rowDy, W, sh + 0.65);
        }
      }
    } else {
      for (let sy = yStart; sy < H; sy += stripStep) {
        const sh = Math.min(stripStep, H - sy);
        // Normalized height `u`: 0.0 at planted boot sole (`footY = 325`), 1.0 at waist (`hipY`)
        const u = Math.max(0.0, Math.min(1.0, (footY - sy) / totalLegH));
        const kneeArch = Math.sin(u * Math.PI); // Peaks at 1.0 right at knee height (`u = 0.5`)!

        // Horizontal displacement: 0px at planted boots -> knee bulge at mid-leg -> `waistDx` at pelvis
        const rowDx = waistDx * Math.pow(u, 1.15) + kneeBulgeX * kneeArch;
        // Vertical compression: 0px at planted boots -> `waistDy` at pelvis (compressing thigh/shin around bending knee)
        const rowDy = waistDy * u;
        // Horizontal straddle spread around `hipX` (widens outward `/` `\` at the knees in `weapons_drawn`!)
        const rowScaleX = 1.0 + kneeSpreadScale * kneeArch;

        const destX = hipX * (1.0 - rowScaleX) + rowDx;
        const destY = sy + rowDy;
        const destH = sh + 0.65;
        ctx.drawImage(layer.lower, 0, sy, W, sh, destX, destY, W * rowScaleX, destH);
      }
    }

    // ========================================================================
    // ZONE 2: TORSO + ARMS + 2H BILLHOOK DEFORMER (Waist Pivot + 2.5D Twist/Shear + Ribcage Breath)
    // ========================================================================
    const posePitchBias = activePose === 'sneaking' ? 1.35 : (activePose === 'weapons_drawn' ? 1.15 : 0.85);
    const walkPitchDeg = isWalking ? (Math.cos(phiL * 2) * 1.6 + (activePose === 'sneaking' ? 2.2 : 1.2)) * walkScale : 0;
    const swayDeg = (breath * 1.15 * posePitchBias + swayWave * 0.65) * boost + walkPitchDeg + talkBob * 0.75 + lungePx * 0.45 - flinchPx * 0.65;
    const swayRad = (swayDeg * Math.PI) / 180;
    const breathLiftDy = (-breath * tgt.breathAmp * 0.75 * boost) + talkBob * 0.45;
    const chestScaleX = 1.0 + Math.max(0, breath) * 0.018 * boost + Math.abs(lungePx) * 0.002;
    const chestScaleY = 1.0 + Math.max(0, breath) * 0.014 * boost;
    // 2.5D horizontal parallax shear (`shearX`) tilts shoulders/polearm forward on inhale & lunge, and counter-swings with walking stride
    const walkCounterShear = isWalking ? -Math.sin(phiR) * 0.016 * walkScale : 0;
    const torsoShearX = (swayWave * 0.014 * boost) + walkCounterShear + (lungePx * 0.0045) - (flinchPx * 0.0055);

    const waistPivotX = hipX + waistDx;
    const waistPivotY = hipY + waistDy + breathLiftDy;

    // Helper mapping any rest upper-body point `(px, py)` through the active Torso affine deformer
    function mapTorsoPoint(px, py) {
      const dx0 = (px - hipX) * chestScaleX;
      const dy0 = (py - hipY) * chestScaleY;
      // Apply horizontal shear (`x' = x + shearX * y`) then rotate by `swayRad`
      const sx = dx0 + torsoShearX * dy0;
      const sy = dy0;
      const c = Math.cos(swayRad), sn = Math.sin(swayRad);
      return [
        waistPivotX + sx * c - sy * sn,
        waistPivotY + sx * sn + sy * c
      ];
    }

    ctx.save();
    ctx.translate(waistPivotX, waistPivotY);
    ctx.rotate(swayRad);
    ctx.transform(chestScaleX, 0, torsoShearX, chestScaleY, 0, 0);
    ctx.drawImage(layer.torsoOnly || layer.upper, -hipX, -hipY);
    ctx.restore();

    // ========================================================================
    // ZONE 3: INDEPENDENT NECK-PIVOTED HEAD STABILIZER + SPEAKING NOD + HAIR BREEZE
    // ========================================================================
    if (layer.headOnly) {
      const [animNeckX, animNeckY] = mapTorsoPoint(neckX, neckY);
      // Gaze counter-stabilization keeps Kaelen's eyes locked on his target (`-swayRad * 0.68`) while adding expressive speaking nod!
      const headCounterRad = -swayRad * 0.68 + ((talkBob * 1.65 + Math.sin(tSec * 2.1 + phaseOffset) * 0.75 * boost - flinchPx * 0.35) * Math.PI) / 180;
      const hairShearX = (hairWave * 0.018 + (isWalking ? -0.022 : 0)) * boost;

      ctx.save();
      ctx.translate(animNeckX, animNeckY);
      ctx.rotate(swayRad + headCounterRad);
      ctx.transform(1.0, 0, hairShearX, 1.0, 0, 0);
      ctx.drawImage(layer.headOnly, -neckX, -neckY);
      ctx.restore();
    }

    // ========================================================================
    // LIVE DEFORMING 2D STRIP-MESH & ZONE CAGE OVERLAY (`🦴 Rig Mesh`)
    // ========================================================================
    if (showDebug) {
      ctx.save();
      // 1. Lower-body deforming scanline lattice (`#38bdf8` cyan) showing pinned/stepping boots + flexing knee curve
      ctx.strokeStyle = 'rgba(56, 189, 248, 0.72)';
      ctx.lineWidth = 1.15;
      const leftRail = [], rightRail = [], midRail = [];
      for (let sy = hipY; sy <= footY; sy += 13) {
        const u = Math.max(0.0, Math.min(1.0, (footY - sy) / totalLegH));
        const kneeArch = Math.sin(u * Math.PI);
        const rowDx = waistDx * Math.pow(u, 1.15) + kneeBulgeX * kneeArch;
        const rowDy = waistDy * u;
        const rowScaleX = 1.0 + kneeSpreadScale * kneeArch;
        const lx = hipX + (96 - hipX) * rowScaleX + rowDx;
        const rx = hipX + (246 - hipX) * rowScaleX + rowDx;
        const mx = hipX + rowDx;
        const wy = sy + rowDy;
        leftRail.push([lx, wy]);
        rightRail.push([rx, wy]);
        midRail.push([mx, wy]);
        ctx.beginPath();
        ctx.moveTo(lx, wy);
        ctx.lineTo(rx, wy);
        ctx.stroke();
      }
      [leftRail, rightRail, midRail].forEach((rail, rIdx) => {
        ctx.strokeStyle = rIdx === 2 ? '#fbbf24' : 'rgba(56, 189, 248, 0.88)';
        ctx.lineWidth = rIdx === 2 ? 2.0 : 1.4;
        ctx.beginPath();
        rail.forEach((pt, i) => { if (i === 0) ctx.moveTo(pt[0], pt[1]); else ctx.lineTo(pt[0], pt[1]); });
        ctx.stroke();
      });

      // 2. Torso 2.5D Shear & Breathing Warp Cage (`#34d399` emerald)
      const tCorners = [
        mapTorsoPoint(82, neckY - 6),
        mapTorsoPoint(268, neckY - 6),
        mapTorsoPoint(268, hipY),
        mapTorsoPoint(82, hipY)
      ];
      ctx.strokeStyle = 'rgba(52, 211, 153, 0.85)';
      ctx.lineWidth = 1.5;
      ctx.setLineDash([4, 3]);
      ctx.beginPath();
      tCorners.forEach((pt, i) => { if (i === 0) ctx.moveTo(pt[0], pt[1]); else ctx.lineTo(pt[0], pt[1]); });
      ctx.closePath();
      ctx.stroke();

      // 3. Head Stabilizer Box (`#c084fc` purple)
      const [hx0, hy0, hx1, hy1] = layer.headBox || [130, 50, 215, 130];
      const [animNeckX, animNeckY] = mapTorsoPoint(neckX, neckY);
      ctx.strokeStyle = 'rgba(192, 132, 252, 0.9)';
      ctx.strokeRect(hx0 + (animNeckX - neckX), hy0 + (animNeckY - neckY), hx1 - hx0, hy1 - hy0);
      ctx.setLineDash([]);

      // 4. Anatomical Deformer Pivots (Planted/Stepping Ankles, Flexing Knees, Waist Hinge, Chest, Neck Stabilizer, Weapon Shaft)
      const uKnee = 0.5;
      const kDx = waistDx * Math.pow(uKnee, 1.15) + kneeBulgeX;
      const kDy = waistDy * uKnee;
      const kSpread = 1.0 + kneeSpreadScale;
      const restPts = activePose === 'relaxed'
        ? { kl: 142, kr: 188, al: [134, 318], ar: [196, 318], w0: [224, 162], w1: [248, 28] }
        : (activePose === 'sneaking'
          ? { kl: 144, kr: 204, al: [122, 318], ar: [202, 318], w0: [82, 190], w1: [288, 190] }
          : { kl: 138, kr: 216, al: [122, 318], ar: [228, 318], w0: [104, 216], w1: [276, 108] });

      const liftL = isWalking ? Math.max(0, Math.cos(phiL)) : 0;
      const liftR = isWalking ? Math.max(0, Math.cos(phiR)) : 0;
      const strideL = isWalking ? Math.sin(phiL) * strideAmp : 0;
      const strideR = isWalking ? Math.sin(phiR) * strideAmp : 0;

      const animKL = [hipX + (restPts.kl - hipX) * kSpread + kDx + strideL * 0.5 + liftL * stepKneeBulge, (hipY + footY) * 0.5 + kDy - liftL * liftHeight * 0.45];
      const animKR = [hipX + (restPts.kr - hipX) * kSpread + kDx + strideR * 0.5 + liftR * stepKneeBulge, (hipY + footY) * 0.5 + kDy - liftR * liftHeight * 0.45];
      const animAL = [restPts.al[0] + strideL, restPts.al[1] - liftL * liftHeight];
      const animAR = [restPts.ar[0] + strideR, restPts.ar[1] - liftR * liftHeight];
      const animChest = mapTorsoPoint(hipX + 4, (hipY + neckY) * 0.52);
      const animW0 = mapTorsoPoint(restPts.w0[0], restPts.w0[1]);
      const animW1 = mapTorsoPoint(restPts.w1[0], restPts.w1[1]);

      ctx.strokeStyle = '#38bdf8'; ctx.lineWidth = 2.3;
      ctx.beginPath(); ctx.moveTo(waistPivotX, waistPivotY); ctx.lineTo(animKL[0], animKL[1]); ctx.lineTo(animAL[0], animAL[1]); ctx.stroke();
      ctx.strokeStyle = '#34d399';
      ctx.beginPath(); ctx.moveTo(waistPivotX, waistPivotY); ctx.lineTo(animKR[0], animKR[1]); ctx.lineTo(animAR[0], animAR[1]); ctx.stroke();
      ctx.strokeStyle = '#fbbf24';
      ctx.beginPath(); ctx.moveTo(waistPivotX, waistPivotY); ctx.lineTo(animChest[0], animChest[1]); ctx.lineTo(animNeckX, animNeckY); ctx.stroke();
      ctx.strokeStyle = '#fb7185';
      ctx.beginPath(); ctx.moveTo(animW0[0], animW0[1]); ctx.lineTo(animW1[0], animW1[1]); ctx.stroke();

      [[waistPivotX, waistPivotY], animChest, [animNeckX, animNeckY], animKL, animKR, animAL, animAR, animW0, animW1].forEach(pt => {
        ctx.fillStyle = '#090d16'; ctx.strokeStyle = '#fbbf24'; ctx.lineWidth = 2;
        ctx.beginPath(); ctx.arc(pt[0], pt[1], 4.5, 0, Math.PI * 2); ctx.fill(); ctx.stroke();
      });
      ctx.restore();
    }
  } else {
    ctx.drawImage(rawImg, 0, 0);
  }

  // Motion smear / steel flash arc during pose transitions or in `weapons_drawn`
  if ((activePose === 'weapons_drawn' || activePose === 'sneaking') && s.glintProgress >= 0 && s.glintProgress <= 1.0) {
    const g = s.glintProgress;
    ctx.save();
    ctx.strokeStyle = 'rgba(254, 243, 199, 0.92)';
    ctx.lineWidth = 3.0;
    ctx.shadowColor = '#f59e0b';
    ctx.shadowBlur = 10;
    ctx.beginPath();
    if (activePose === 'sneaking') {
      const gx = 120 + g * 165;
      const gy = 188;
      ctx.moveTo(gx - 16, gy - 4);
      ctx.lineTo(gx + 16, gy + 4);
    } else {
      const gx = 128 + g * 145;
      const gy = 198 - g * 92;
      ctx.moveTo(gx - 12, gy + 12);
      ctx.lineTo(gx + 12, gy - 12);
    }
    ctx.stroke();
    ctx.restore();
  }

  ctx.restore();
}

function drawSkeletonDebugOverlay(ctx, j) {
  ctx.save();
  function drawOrientedLimbBox(pA, pB, halfW, strokeCol) {
    const ang = Math.atan2(pB[1] - pA[1], pB[0] - pA[0]);
    const len = Math.hypot(pB[0] - pA[0], pB[1] - pA[1]);
    ctx.save();
    ctx.translate(pA[0], pA[1]);
    ctx.rotate(ang);
    ctx.strokeStyle = strokeCol;
    ctx.lineWidth = 1.2;
    ctx.setLineDash([3, 2]);
    ctx.strokeRect(0, -halfW, len, halfW * 2);
    ctx.restore();
  }
  drawOrientedLimbBox(j.animHipL,      j.animKneeL,  14, 'rgba(56, 189, 248, 0.55)');
  drawOrientedLimbBox(j.animKneeL,     j.animAnkleL, 12, 'rgba(56, 189, 248, 0.55)');
  drawOrientedLimbBox(j.animHipR,      j.animKneeR,  14, 'rgba(52, 211, 153, 0.55)');
  drawOrientedLimbBox(j.animKneeR,     j.animAnkleR, 12, 'rgba(52, 211, 153, 0.55)');
  drawOrientedLimbBox(j.animShoulderL, j.animElbowL, 12, 'rgba(168, 85, 247, 0.55)');
  drawOrientedLimbBox(j.animElbowL,    j.animHandL,  11, 'rgba(168, 85, 247, 0.55)');
  drawOrientedLimbBox(j.animShoulderR, j.animElbowR, 12, 'rgba(244, 63, 94, 0.55)');
  drawOrientedLimbBox(j.animElbowR,    j.animHandR,  11, 'rgba(244, 63, 94, 0.55)');
  drawOrientedLimbBox(j.animPelvis,    j.animNeck,   26, 'rgba(251, 191, 36, 0.45)');

  function drawChain(pts, color, width = 2.4) {
    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    pts.forEach((pt, i) => {
      if (i === 0) ctx.moveTo(pt[0], pt[1]);
      else ctx.lineTo(pt[0], pt[1]);
    });
    ctx.stroke();
  }
  drawChain([j.animPelvis, j.animHipL, j.animKneeL, j.animAnkleL, j.animToeL], '#38bdf8', 2.4);
  drawChain([j.animPelvis, j.animHipR, j.animKneeR, j.animAnkleR, j.animToeR], '#34d399', 2.4);
  drawChain([j.animPelvis, j.animChest, j.animNeck, j.animHead], '#fbbf24', 2.6);
  drawChain([j.animChest, j.animShoulderL, j.animElbowL, j.animHandL], '#a855f7', 2.3);
  drawChain([j.animChest, j.animShoulderR, j.animElbowR, j.animHandR], '#f43f5e', 2.3);

  const weapTipX = j.animPx + Math.sin(j.rad) * 92;
  const weapTipY = j.animPy - Math.cos(j.rad) * 92;
  drawChain([[j.animPx, j.animPy], [weapTipX, weapTipY]], '#fb7185', 2.2);

  const allJoints = [
    { pt: j.animPelvis,    col: '#fbbf24', r: 5.0 },
    { pt: j.animChest,     col: '#fbbf24', r: 4.5 },
    { pt: j.animNeck,      col: '#fbbf24', r: 4.5 },
    { pt: j.animHead,      col: '#fbbf24', r: 4.5 },
    { pt: j.animHipL,      col: '#38bdf8', r: 4.5 },
    { pt: j.animKneeL,     col: '#38bdf8', r: 5.2 },
    { pt: j.animAnkleL,    col: '#38bdf8', r: 4.2 },
    { pt: j.animToeL,      col: '#38bdf8', r: 3.2 },
    { pt: j.animHipR,      col: '#34d399', r: 4.5 },
    { pt: j.animKneeR,     col: '#34d399', r: 5.2 },
    { pt: j.animAnkleR,    col: '#34d399', r: 4.2 },
    { pt: j.animToeR,      col: '#34d399', r: 3.2 },
    { pt: j.animShoulderL, col: '#a855f7', r: 4.5 },
    { pt: j.animElbowL,    col: '#a855f7', r: 4.8 },
    { pt: j.animHandL,     col: '#a855f7', r: 4.0 },
    { pt: j.animShoulderR, col: '#f43f5e', r: 4.5 },
    { pt: j.animElbowR,    col: '#f43f5e', r: 4.8 },
    { pt: j.animHandR,     col: '#f43f5e', r: 5.5 }
  ];
  allJoints.forEach(item => {
    ctx.fillStyle = '#090d16';
    ctx.strokeStyle = item.col;
    ctx.lineWidth = 2.0;
    ctx.beginPath();
    ctx.arc(item.pt[0], item.pt[1], item.r, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
  });
  ctx.restore();
}

function drawVtuberRigOnCanvas(canvas, s, tgt, tSec) {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  const rigKey = canvas.getAttribute('data-rig-key') || 'kaelen';
  const rigEngine = canvas.getAttribute('data-rig-engine') || STATE.storyRigEngine || 'atlas_b';
  const side = canvas.getAttribute('data-side') || 'left';
  const isActive = canvas.getAttribute('data-active') === '1';
  const showDebug = canvas.getAttribute('data-debug') === '1';

  // Dispatch to Approach B (Exploded 13-Part Spine2D Atlas Rig) or Approach C (3-Pose Sheet + Rig) for side-view Kaelen
  if (rigKey === 'kaelen' || rigKey === 'kaelen_side') {
    if (rigEngine === 'atlas_b') {
      const atlasEntry = getAtlasRigEntry('kaelen');
      if (atlasEntry && atlasEntry.parts) {
        drawAtlasRigOnCanvas(ctx, atlasEntry, s, tgt, tSec, side, isActive, showDebug);
        return;
      }
    } else if (rigEngine === 'pose_c') {
      const poseEntry = getPoseSheetEntry('kaelen');
      if (poseEntry && poseEntry.images.relaxed && poseEntry.images.relaxed.complete) {
        drawPoseSheetRigOnCanvas(ctx, poseEntry, s, tgt, tSec, side, isActive, showDebug);
        return;
      }
    }
  }

  const rigImgs = getVtuberRigImages(rigKey);
  if (!rigImgs || !rigImgs.bodyImg.complete) return;

  const W = 340, H = 340;
  ctx.clearRect(0, 0, W, H);
  ctx.imageSmoothingEnabled = false;

  const isFrontOrBack = rigKey.endsWith('_front') || rigKey.endsWith('_back');
  const shouldFlip = !isFrontOrBack && ((side === 'right' && !rigImgs.nativeLeft) || (side === 'left' && rigImgs.nativeLeft));
  const dirSign = rigImgs.nativeLeft ? -1 : 1; // +1 if sprite faces right, -1 if sprite faces left

  const phaseOffset = side === 'left' ? 0.0 : 1.7;
  const breath = Math.sin(tSec * tgt.breathFreq + phaseOffset);
  const talkBob = isActive ? Math.sin(tSec * 6.5) * 0.9 : 0;

  const isWalking = (typeof STATE !== 'undefined' && STATE.storyWalkMode && STATE.storyWalkMode !== 'off') && (side === 'left' || STATE.storyWalkMode === 'scroll_bg');
  const walkFreq = s.pose === 'sneaking' ? 4.4 : 5.5;
  const phiL = tSec * walkFreq + phaseOffset;
  const phiR = phiL + Math.PI;
  const walkBobDy = isWalking ? Math.sin(phiL * 2 - 0.45) * 3.0 : 0;

  const cfg = rigImgs.cfg;
  const parts = rigImgs.parts;

  // 1. Compute Animated Root (`animPelvis`) and Spine / Torso / Head FK Hierarchy
  const pelvisDx = dirSign * (s.pelvisShiftX + (isWalking ? Math.cos(phiL * 2) * 1.5 : 0));
  const pelvisDy = Math.max(0, s.crouchDrop) + walkBobDy;
  const animPelvis = [cfg.pelvis[0] + pelvisDx, cfg.pelvis[1] + pelvisDy];

  const spineRad = (dirSign * (s.spineAngle + breath * 1.1 + (isWalking ? Math.cos(phiL * 2) * 1.4 : 0)) * Math.PI) / 180;
  const animChest = rotatePtAround([cfg.chest[0] + pelvisDx, cfg.chest[1] + pelvisDy - breath * tgt.breathAmp * 0.6], animPelvis, spineRad);
  const animNeck  = rotatePtAround([cfg.neck[0]  + pelvisDx, cfg.neck[1]  + pelvisDy - breath * tgt.breathAmp], animPelvis, spineRad);

  const headRad = spineRad + (dirSign * (s.headAngle + talkBob * 1.4 - (isWalking ? Math.cos(phiL * 2) * 1.1 : 0)) * Math.PI) / 180;
  const headDeltaFromRest = [cfg.head[0] - cfg.neck[0], cfg.head[1] - cfg.neck[1]];
  const animHead = rotatePtAround([animNeck[0] + headDeltaFromRest[0], animNeck[1] + headDeltaFromRest[1]], animNeck, headRad);

  // 2. Compute Animated Left & Right Legs using True 2-Bone Analytic Inverse Kinematics (IK)!
  const animHipL = rotatePtAround([cfg.hipL[0] + pelvisDx, cfg.hipL[1] + pelvisDy], animPelvis, spineRad * 0.25);
  const animHipR = rotatePtAround([cfg.hipR[0] + pelvisDx, cfg.hipR[1] + pelvisDy], animPelvis, spineRad * 0.25);

  const strideAmp = s.pose === 'sneaking' ? 12.5 : 15.5;
  const liftAmp   = s.pose === 'sneaking' ? 7.5  : 9.5;
  const walkDxL   = isWalking ? dirSign * Math.sin(phiL) * strideAmp : 0;
  const walkDxR   = isWalking ? dirSign * Math.sin(phiR) * strideAmp : 0;
  const walkLiftL = isWalking ? -Math.max(0, Math.cos(phiL)) * liftAmp : 0;
  const walkLiftR = isWalking ? -Math.max(0, Math.cos(phiR)) * liftAmp : 0;

  const stepL = (dirSign > 0 ? s.backStepX : -s.frontStepX) * (isWalking ? 0.35 : 1.0) + walkDxL;
  const stepR = (dirSign > 0 ? s.frontStepX : -s.backStepX) * (isWalking ? 0.35 : 1.0) + walkDxR;
  const animAnkleL = [cfg.ankleL[0] + stepL, cfg.ankleL[1] + walkLiftL];
  const animAnkleR = [cfg.ankleR[0] + stepR, cfg.ankleR[1] + walkLiftR];
  const animToeL   = [cfg.toeL[0]   + stepL, cfg.toeL[1]   + walkLiftL];
  const animToeR   = [cfg.toeR[0]   + stepR, cfg.toeR[1]   + walkLiftR];

  const L1_L = Math.hypot(cfg.kneeL[0] - cfg.hipL[0], cfg.kneeL[1] - cfg.hipL[1]);
  const L2_L = Math.hypot(cfg.ankleL[0] - cfg.kneeL[0], cfg.ankleL[1] - cfg.kneeL[1]);
  const L1_R = Math.hypot(cfg.kneeR[0] - cfg.hipR[0], cfg.kneeR[1] - cfg.hipR[1]);
  const L2_R = Math.hypot(cfg.ankleR[0] - cfg.kneeR[0], cfg.ankleR[1] - cfg.kneeR[1]);

  const bendDirL = rigKey.endsWith('_front') ? -0.65 : (rigKey.endsWith('_back') ? -0.65 : dirSign);
  const bendDirR = rigKey.endsWith('_front') ?  0.85 : (rigKey.endsWith('_back') ?  0.85 : dirSign);

  const animKneeL = solveTwoBoneIK(animHipL, animAnkleL, L1_L, L2_L, bendDirL);
  const animKneeR = solveTwoBoneIK(animHipR, animAnkleR, L1_R, L2_R, bendDirR);

  // 3. Compute Animated Left & Right Arms (`Shoulder -> Elbow -> Hand`) via Forward Kinematics (FK)
  const animShoulderL = rotatePtAround([cfg.shoulderL[0] + pelvisDx, cfg.shoulderL[1] + pelvisDy - breath * tgt.breathAmp * 0.8], animPelvis, spineRad);
  const animShoulderR = rotatePtAround([cfg.shoulderR[0] + pelvisDx, cfg.shoulderR[1] + pelvisDy - breath * tgt.breathAmp * 0.8], animPelvis, spineRad);

  const walkArmSwing = isWalking && s.pose === 'relaxed' ? Math.sin(phiR) * 10.0 : (isWalking ? Math.sin(phiR) * 3.0 : 0);
  const lIsWeaponArm = rigImgs.nativeLeft || rigKey === 'kaelen_back';
  const shL_deg = (lIsWeaponArm ? -s.shoulderRAngle : dirSign * s.shoulderLAngle) + walkArmSwing;
  const elL_deg = lIsWeaponArm ? -s.elbowRAngle    : dirSign * s.elbowLAngle;
  const shR_deg = (lIsWeaponArm ? -s.shoulderLAngle : dirSign * s.shoulderRAngle) - walkArmSwing * 0.65;
  const elR_deg = lIsWeaponArm ? -s.elbowLAngle    : dirSign * s.elbowRAngle;

  const shL_rad = spineRad * 0.55 + ((shL_deg + breath * 1.2) * Math.PI) / 180;
  const elL_rad = shL_rad + (elL_deg * Math.PI) / 180;
  const shR_rad = spineRad * 0.55 + ((shR_deg - breath * 1.2) * Math.PI) / 180;
  const elR_rad = shR_rad + (elR_deg * Math.PI) / 180;

  const uArmL_vec = [cfg.elbowL[0] - cfg.shoulderL[0], cfg.elbowL[1] - cfg.shoulderL[1]];
  const fArmL_vec = [cfg.handL[0]  - cfg.elbowL[0],    cfg.handL[1]  - cfg.elbowL[1]];
  const animElbowL = rotatePtAround([animShoulderL[0] + uArmL_vec[0], animShoulderL[1] + uArmL_vec[1]], animShoulderL, shL_rad);
  const animHandL  = rotatePtAround([animElbowL[0]    + fArmL_vec[0], animElbowL[1]    + fArmL_vec[1]], animElbowL,    elL_rad);

  const uArmR_vec = [cfg.elbowR[0] - cfg.shoulderR[0], cfg.elbowR[1] - cfg.shoulderR[1]];
  const fArmR_vec = [cfg.handR[0]  - cfg.elbowR[0],    cfg.handR[1]  - cfg.elbowR[1]];
  const animElbowR = rotatePtAround([animShoulderR[0] + uArmR_vec[0], animShoulderR[1] + uArmR_vec[1]], animShoulderR, shR_rad);
  const animHandR  = rotatePtAround([animElbowR[0]    + fArmR_vec[0], animElbowR[1]    + fArmR_vec[1]], animElbowR,    elR_rad);

  // 4. Compute Weapon Pivot & Two-Handed Shaft Alignment (`animPx, animPy, rad`)
  const [px, py] = rigImgs.pivot;
  const weapElbowRest = lIsWeaponArm ? cfg.elbowL : cfg.elbowR;
  const weapElbowAnim = lIsWeaponArm ? animElbowL : animElbowR;
  const weapArmRad    = lIsWeaponArm ? elL_rad    : elR_rad;
  let [animPx, animPy] = rotatePtAround(
    [weapElbowAnim[0] + (px - weapElbowRest[0]), weapElbowAnim[1] + (py - weapElbowRest[1])],
    weapElbowAnim,
    weapArmRad
  );

  const weapSign = rigKey === 'kaelen_back' ? -1 : dirSign;
  const totalWeapDeg = weapSign * (s.weaponAngle + breath * 1.1 + (isActive ? Math.sin(tSec * 5.2) * 1.0 : 0));
  const rad = (totalWeapDeg * Math.PI) / 180;
  const shaftAng = rad + (weapSign * 10.0 * Math.PI) / 180;
  const slideAmt = s.weapSlide || 0;
  animPx -= Math.sin(shaftAng) * slideAmt;
  animPy += Math.cos(shaftAng) * slideAmt;

  ctx.save();
  if (shouldFlip) {
    ctx.translate(W, 0);
    ctx.scale(-1, 1);
  }

  // 5. Render All 10 Articulated Limb Parts + Weapon in Painter's Depth Order:
  //    Legs -> Upper Arms -> Torso -> Head -> Weapon Shaft -> Both Forearms/Hands (so both hands visibly grip the polearm!)
  if (parts) {
    drawBonePartAtoB(ctx, parts.legL_thigh, cfg.hipL,   cfg.kneeL,  animHipL,   animKneeL);
    drawBonePartAtoB(ctx, parts.legL_shin,  cfg.kneeL,  cfg.ankleL, animKneeL,  animAnkleL);
    drawArticulatedPart(ctx, parts.legL_boot, cfg.ankleL, animAnkleL, 0);

    drawBonePartAtoB(ctx, parts.legR_thigh, cfg.hipR,   cfg.kneeR,  animHipR,   animKneeR);
    drawBonePartAtoB(ctx, parts.legR_shin,  cfg.kneeR,  cfg.ankleR, animKneeR,  animAnkleR);
    drawArticulatedPart(ctx, parts.legR_boot, cfg.ankleR, animAnkleR, 0);

    if (!lIsWeaponArm) {
      drawBonePartAtoB(ctx, parts.armR_upper, cfg.shoulderR, cfg.elbowR, animShoulderR, animElbowR);
    } else {
      drawBonePartAtoB(ctx, parts.armL_upper, cfg.shoulderL, cfg.elbowL, animShoulderL, animElbowL);
    }

    const chestScaleX = 1.0 + Math.max(0, breath) * 0.018;
    const chestScaleY = 1.0 + Math.max(0, breath) * 0.012;
    drawArticulatedPart(ctx, parts.torso, cfg.pelvis, animPelvis, spineRad, chestScaleX, chestScaleY);
    drawArticulatedPart(ctx, parts.head, cfg.neck, animNeck, headRad);

    if (!lIsWeaponArm) {
      drawBonePartAtoB(ctx, parts.armL_upper, cfg.shoulderL, cfg.elbowL, animShoulderL, animElbowL);
    } else {
      drawBonePartAtoB(ctx, parts.armR_upper, cfg.shoulderR, cfg.elbowR, animShoulderR, animElbowR);
    }
  } else {
    ctx.drawImage(rigImgs.bodyImg, 0, 0);
  }

  // Draw Rigid `weapon` Layer (between upper arms/torso and gripping forearms/hands!)
  ctx.save();
  ctx.translate(animPx, animPy);
  ctx.rotate(rad);
  if (rigImgs.weapImg && rigImgs.weapImg.complete) {
    ctx.drawImage(rigImgs.weapImg, -px, -py);
  }
  ctx.restore();

  // Draw both Forearms/Hands OVER the weapon shaft so both hands visibly clasp the polearm in `sneaking` and `weapons_drawn`!
  if (parts) {
    drawBonePartAtoB(ctx, parts.armR_lower, cfg.elbowR, cfg.handR, animElbowR, animHandR);
    drawBonePartAtoB(ctx, parts.armL_lower, cfg.elbowL, cfg.handL, animElbowL, animHandL);
  }

  // 6. Blade-aligned specular cold-steel glint in `weapons_drawn`
  ctx.save();
  ctx.translate(animPx, animPy);
  ctx.rotate(rad);
  if (s.pose === 'weapons_drawn' && s.glintProgress >= 0 && s.glintProgress <= 1.0) {
    const g = s.glintProgress;
    const x0 = weapSign * 6;
    const y0 = 18;
    const x1 = weapSign * 36;
    const y1 = -112;
    const gx = x0 + (x1 - x0) * g;
    const gy = y0 + (y1 - y0) * g;

    ctx.save();
    ctx.strokeStyle = 'rgba(254, 243, 199, 0.92)';
    ctx.lineWidth = 3.2;
    ctx.shadowColor = '#f59e0b';
    ctx.shadowBlur = 10;
    ctx.beginPath();
    ctx.moveTo(gx - weapSign * 8, gy + 22);
    ctx.lineTo(gx + weapSign * 8, gy - 22);
    ctx.stroke();

    ctx.strokeStyle = '#ffffff';
    ctx.lineWidth = 1.8;
    ctx.beginPath();
    ctx.moveTo(gx - 11, gy);
    ctx.lineTo(gx + 11, gy);
    ctx.moveTo(gx, gy - 11);
    ctx.lineTo(gx, gy + 11);
    ctx.stroke();
    ctx.restore();
  }
  ctx.restore();

  // 7. Full-Body All-Limbs 16-Joint / 14-Bone Skeleton & Limb Box Debug Overlay (`🦴 Rig Mesh`)
  if (showDebug) {
    ctx.save();

    function drawOrientedLimbBox(pA, pB, halfW, strokeCol) {
      const ang = Math.atan2(pB[1] - pA[1], pB[0] - pA[0]);
      const len = Math.hypot(pB[0] - pA[0], pB[1] - pA[1]);
      ctx.save();
      ctx.translate(pA[0], pA[1]);
      ctx.rotate(ang);
      ctx.strokeStyle = strokeCol;
      ctx.lineWidth = 1.2;
      ctx.setLineDash([3, 2]);
      ctx.strokeRect(0, -halfW, len, halfW * 2);
      ctx.restore();
    }

    // Limb bounding boxes showing rigid bone preservation (zero squishing!)
    drawOrientedLimbBox(animHipL,      animKneeL,  14, 'rgba(56, 189, 248, 0.55)');
    drawOrientedLimbBox(animKneeL,     animAnkleL, 12, 'rgba(56, 189, 248, 0.55)');
    drawOrientedLimbBox(animHipR,      animKneeR,  14, 'rgba(52, 211, 153, 0.55)');
    drawOrientedLimbBox(animKneeR,     animAnkleR, 12, 'rgba(52, 211, 153, 0.55)');
    drawOrientedLimbBox(animShoulderL, animElbowL, 12, 'rgba(168, 85, 247, 0.55)');
    drawOrientedLimbBox(animElbowL,    animHandL,  11, 'rgba(168, 85, 247, 0.55)');
    drawOrientedLimbBox(animShoulderR, animElbowR, 12, 'rgba(244, 63, 94, 0.55)');
    drawOrientedLimbBox(animElbowR,    animHandR,  11, 'rgba(244, 63, 94, 0.55)');
    drawOrientedLimbBox(animPelvis,    animNeck,   26, 'rgba(251, 191, 36, 0.45)');

    function drawChain(pts, color, width = 2.4) {
      ctx.strokeStyle = color;
      ctx.lineWidth = width;
      ctx.beginPath();
      pts.forEach((pt, i) => {
        if (i === 0) ctx.moveTo(pt[0], pt[1]);
        else ctx.lineTo(pt[0], pt[1]);
      });
      ctx.stroke();
    }

    // 1) Left / Back Leg 2-Bone IK Chain (Cyan `#38bdf8`)
    drawChain([animPelvis, animHipL, animKneeL, animAnkleL, animToeL], '#38bdf8', 2.4);
    // 2) Right / Front Leg 2-Bone IK Chain (Emerald `#34d399`)
    drawChain([animPelvis, animHipR, animKneeR, animAnkleR, animToeR], '#34d399', 2.4);
    // 3) Spine & Head Chain (Gold `#fbbf24`)
    drawChain([animPelvis, animChest, animNeck, animHead], '#fbbf24', 2.6);
    // 4) Left Arm FK Chain (Violet `#a855f7`)
    drawChain([animChest, animShoulderL, animElbowL, animHandL], '#a855f7', 2.3);
    // 5) Right Arm FK Chain (Crimson `#f43f5e`)
    drawChain([animChest, animShoulderR, animElbowR, animHandR], '#f43f5e', 2.3);

    // 6) Weapon Vector from Grip Pivot
    const weapTipX = animPx + Math.sin(rad) * 92;
    const weapTipY = animPy - Math.cos(rad) * 92;
    drawChain([[animPx, animPy], [weapTipX, weapTipY]], '#fb7185', 2.2);

    // Draw all 16 anatomical joints
    const allJoints = [
      { pt: animPelvis,    col: '#fbbf24', r: 5.0 },
      { pt: animChest,     col: '#fbbf24', r: 4.5 },
      { pt: animNeck,      col: '#fbbf24', r: 4.5 },
      { pt: animHead,      col: '#fbbf24', r: 4.5 },
      { pt: animHipL,      col: '#38bdf8', r: 4.5 },
      { pt: animKneeL,     col: '#38bdf8', r: 5.2 },
      { pt: animAnkleL,    col: '#38bdf8', r: 4.2 },
      { pt: animToeL,      col: '#38bdf8', r: 3.2 },
      { pt: animHipR,      col: '#34d399', r: 4.5 },
      { pt: animKneeR,     col: '#34d399', r: 5.2 },
      { pt: animAnkleR,    col: '#34d399', r: 4.2 },
      { pt: animToeR,      col: '#34d399', r: 3.2 },
      { pt: animShoulderL, col: '#a855f7', r: 4.5 },
      { pt: animElbowL,    col: '#a855f7', r: 4.8 },
      { pt: animHandL,     col: '#a855f7', r: 4.0 },
      { pt: animShoulderR, col: '#f43f5e', r: 4.5 },
      { pt: animElbowR,    col: '#f43f5e', r: 4.8 },
      { pt: animHandR,     col: '#f43f5e', r: 5.5 }
    ];
    allJoints.forEach(j => {
      ctx.fillStyle = '#090d16';
      ctx.strokeStyle = j.col;
      ctx.lineWidth = 2.0;
      ctx.beginPath();
      ctx.arc(j.pt[0], j.pt[1], j.r, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
    });

    ctx.restore();
  }

  ctx.restore();
}

let _vtuberRafId = null;
let _vtuberLastTs = 0;
let _storyBgScrollPx = 0;
let _storyCameraPanProgress = 0;

function resetStoryCameraPan() {
  _storyCameraPanProgress = 0;
  _storyBgScrollPx = 0;
}

function startVtuberRigLoop() {
  if (_vtuberRafId) cancelAnimationFrame(_vtuberRafId);
  _vtuberLastTs = performance.now();

  function tick(now) {
    if (STATE.screen !== 'story' || (STATE.storyMode || 'vn_stage') !== 'vn_stage') {
      _vtuberRafId = null;
      return;
    }
    const dt = Math.min(0.05, Math.max(0.001, (now - _vtuberLastTs) / 1000));
    _vtuberLastTs = now;
    const tSec = now / 1000;

    // 1. Update 60fps Stage Walk Motion (`scroll_bg` Treadmill vs `pan_camera` Follow Hero)
    const walkMode = STATE.storyWalkMode || 'off';
    const isSneak = STATE.storyRigPose === 'sneaking';
    const bgStripEl   = document.getElementById('vn-bg-strip');
    const fgParallaxEl = document.getElementById('vn-fg-parallax');
    const leftWrapEl  = document.getElementById('vn-left-actor-wrap');
    const rightWrapEl = document.getElementById('vn-right-actor-wrap');
    const telemEl     = document.getElementById('vn-walk-telemetry');

    if (walkMode === 'scroll_bg') {
      // Mode 1: Hero stays in place; seamless 3-tile background & 1.48x foreground mist/cobblestones scroll past
      const speedPxPerSec = isSneak ? 64 : 98;
      _storyBgScrollPx = (_storyBgScrollPx + speedPxPerSec * dt) % 2560;
      const fgScrollPx = (_storyBgScrollPx * 1.48) % 2560;
      if (bgStripEl)    bgStripEl.style.transform    = `translate3d(${-_storyBgScrollPx.toFixed(1)}px, 0, 0)`;
      if (fgParallaxEl) fgParallaxEl.style.transform = `translate3d(${-fgScrollPx.toFixed(1)}px, 0, 0)`;
      if (leftWrapEl)   leftWrapEl.style.transform   = 'translate3d(0px, 0, 0)';
      if (rightWrapEl)  rightWrapEl.style.transform  = 'translate3d(0px, 0, 0)';
      if (telemEl)      telemEl.textContent          = `🔄 TREADMILL SCROLL · BG: -${Math.round(_storyBgScrollPx)}px · FG: -${Math.round(fgScrollPx)}px (${speedPxPerSec}px/s)`;
    } else if (walkMode === 'pan_camera') {
      // Mode 2: Wide 2160px stage; Kaelen walks across the stage while the camera pans smoothly to follow him
      const panRate = isSneak ? 0.085 : 0.125;
      _storyCameraPanProgress = (_storyCameraPanProgress + panRate * dt) % 1.0;
      // Smooth ease-in-out camera follow curve across the 720px wide-stage pan range
      const p = _storyCameraPanProgress;
      const camEase = p < 0.5 ? 2 * p * p : 1 - Math.pow(-2 * p + 2, 2) / 2;
      const camPanX = -camEase * 680;
      const fgPanX  = camPanX * 1.42;
      // Hero physically advances across the screen (-90px -> +190px) while camera pans (-680px)
      const heroScreenX = -90 + p * 280;
      // Right-side NPC is anchored in world space ahead in the alley and pans into the right slot (+240px -> 0px)
      const npcScreenX  = (1.0 - camEase) * 240;

      if (bgStripEl)    bgStripEl.style.transform    = `translate3d(${camPanX.toFixed(1)}px, 0, 0)`;
      if (fgParallaxEl) fgParallaxEl.style.transform = `translate3d(${fgPanX.toFixed(1)}px, 0, 0)`;
      if (leftWrapEl)   leftWrapEl.style.transform   = `translate3d(${heroScreenX.toFixed(1)}px, 0, 0)`;
      if (rightWrapEl)  rightWrapEl.style.transform  = `translate3d(${npcScreenX.toFixed(1)}px, 0, 0)`;
      if (telemEl)      telemEl.textContent          = `🎥 CAMERA FOLLOW · CamPan: ${Math.round(camPanX)}px · HeroX: ${heroScreenX >= 0 ? '+' : ''}${Math.round(heroScreenX)}px (${Math.round(p * 100)}%)`;
    } else {
      if (bgStripEl)    bgStripEl.style.transform    = 'translate3d(0px, 0, 0)';
      if (fgParallaxEl) fgParallaxEl.style.transform = 'translate3d(0px, 0, 0)';
      if (leftWrapEl)   leftWrapEl.style.transform   = 'translate3d(0px, 0, 0)';
      if (rightWrapEl)  rightWrapEl.style.transform  = 'translate3d(0px, 0, 0)';
    }

    // 2. Step springs & render 60fps rigged canvases
    const canvases = document.querySelectorAll('canvas.vn-vtuber-canvas');
    canvases.forEach(cv => {
      const slotId = cv.getAttribute('data-slot-id') || 'left';
      const targetPose = cv.getAttribute('data-pose') || 'relaxed';
      const tgt = VTUBER_POSE_TARGETS[targetPose] || VTUBER_POSE_TARGETS.relaxed;
      const spring = getVtuberSlotSpring(slotId, targetPose);
      stepVtuberSpring(spring, tgt, dt);
      drawVtuberRigOnCanvas(cv, spring, tgt, tSec);
    });

    _vtuberRafId = requestAnimationFrame(tick);
  }
  _vtuberRafId = requestAnimationFrame(tick);
}

function renderReactionFxOverlay(fxKey, isActive, side, activePose) {
  if (!fxKey && activePose !== 'weapons_drawn' && activePose !== 'sneaking') return { back: '', front: '' };

  if (fxKey === 'steel_flash' || activePose === 'weapons_drawn') {
    return {
      back: `
        <!-- Centered Torso Aura + Floor Impact Ripple behind rigged sprite -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:1;">
          <div style="position:absolute;left:50%;top:56%;width:190px;height:190px;border-radius:50%;border:2px solid rgba(251,191,36,0.75);box-shadow:0 0 22px rgba(245,158,11,0.32);animation:vnPulseRing 1.5s ease-out infinite;"></div>
          <div style="position:absolute;left:50%;bottom:4px;width:210px;height:36px;border-radius:50%;border:2px solid rgba(56,189,248,0.72);animation:vnFloorRipple 1.5s ease-out infinite;"></div>
        </div>
      `,
      front: `
        <div style="position:absolute;inset:0;pointer-events:none;z-index:4;">
          <div class="vn-emote-badge" style="position:absolute;left:50%;top:-18px;padding:2px 10px;border-radius:10px;background:rgba(15,23,42,0.95);border:1.5px solid #fbbf24;color:#fef3c7;font-size:11px;font-weight:800;white-space:nowrap;box-shadow:0 4px 12px rgba(0,0,0,0.9);">
            ⚔ WEAPONS DRAWN
          </div>
        </div>
      `
    };
  }

  if (activePose === 'sneaking' && fxKey !== 'anger_burst' && fxKey !== 'alchemy_flare') {
    return {
      back: `
        <!-- Stealth Shadow Ripple on the cobblestones -->
        <div style="position:absolute;inset:0;pointer-events:none;z-index:1;">
          <div style="position:absolute;left:50%;bottom:4px;width:205px;height:34px;border-radius:50%;border:1.5px dashed rgba(56,189,248,0.72);animation:vnFloorRipple 1.8s ease-out infinite;"></div>
        </div>
      `,
      front: `
        <div style="position:absolute;inset:0;pointer-events:none;z-index:4;">
          <div class="vn-emote-badge" style="position:absolute;left:50%;top:-14px;padding:2px 10px;border-radius:10px;background:rgba(12,74,110,0.94);border:1.5px solid #38bdf8;color:#e0f2fe;font-size:10.5px;font-weight:800;white-space:nowrap;box-shadow:0 4px 12px rgba(0,0,0,0.9);">
            👣 SNEAKING (LOW GUARD)
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

function resolveActorPose(actor) {
  if (STATE.storyRigPose && STATE.storyRigPose !== 'auto') {
    return STATE.storyRigPose;
  }
  if (STATE.storyReactionOverride === 'steel_flash') {
    return 'weapons_drawn';
  }
  if (actor.pose) return actor.pose;
  if (actor.reactionFx === 'steel_flash' || actor.lunge) return 'weapons_drawn';
  return 'relaxed';
}

function resolveActorRigKey(unitKey) {
  if (unitKey === 'kaelen') {
    const v = STATE.storyTurnaroundView || 'side';
    if (v === 'front') return 'kaelen_front';
    if (v === 'back') return 'kaelen_back';
    if (v === 'side_sheet') return 'kaelen_side';
    return 'kaelen';
  }
  return unitKey;
}

function renderStoryActorSlot(actor, side) {
  if (!actor) return `<div style="width:310px;height:345px;pointer-events:none;"></div>`;
  const style = STATE.storyActorStyle || 'full_sprite';
  if (style === 'backdrop_only') return `<div style="width:310px;height:345px;pointer-events:none;"></div>`;

  const isActive = !!actor.active;
  const activePose = resolveActorPose(actor);
  const activeFx = isActive && STATE.storyReactionOverride ? STATE.storyReactionOverride : actor.reactionFx;
  const rigKey = resolveActorRigKey(actor.unitKey);

  const filterStr = isActive
    ? 'brightness(1.06) contrast(1.06) drop-shadow(0 0 14px rgba(245,158,11,0.45)) drop-shadow(0 10px 18px rgba(0,0,0,0.92))'
    : 'brightness(0.62) saturate(0.78) contrast(0.96) drop-shadow(0 8px 14px rgba(0,0,0,0.88))';
  const borderCol = isActive ? '#f59e0b' : '#4b5563';

  if (style === 'bust_cutout') {
    const portraitSrc = getPortraitWithMood(actor.portrait, isActive ? (STATE.storyMoodOverride || 'determined') : 'neutral');
    return `
      <div style="display:flex;flex-direction:column;align-items:center;transition:all 0.22s ease;">
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

  const fxLayers = renderReactionFxOverlay(activeFx, isActive, side, activePose);
  const poseMeta = VTUBER_POSE_TARGETS[activePose] || VTUBER_POSE_TARGETS.relaxed;
  const poseBadgeCol = activePose === 'weapons_drawn' ? '#fbbf24' : (activePose === 'sneaking' ? '#38bdf8' : '#9ca3af');

  return `
    <div style="display:flex;flex-direction:column;align-items:center;position:relative;width:310px;height:345px;justify-content:flex-end;">
      <div style="position:relative;width:288px;height:288px;display:flex;align-items:flex-end;justify-content:center;">
        <!-- Ground Contact Shadow + Tallow Floor Ring directly on the visible stone floor -->
        <div style="position:absolute;left:50%;bottom:2px;transform:translateX(-50%);width:195px;height:34px;border-radius:50%;background:${isActive ? 'radial-gradient(ellipse, rgba(0,0,0,0.92) 0%, rgba(245,158,11,0.34) 58%, transparent 82%)' : 'radial-gradient(ellipse, rgba(0,0,0,0.88) 0%, rgba(0,0,0,0.45) 60%, transparent 82%)'};border-bottom:${isActive ? '2px solid rgba(251,191,36,0.78)' : '1px solid rgba(148,163,184,0.28)'};z-index:0;"></div>

        ${fxLayers.back}

        <!-- 60fps Live2D / VTuber-Style Deformer & Joint Rig Canvas (340×340 internal resolution) -->
        <canvas class="vn-vtuber-canvas pixel"
                width="340" height="340"
                data-slot-id="${side}_${actor.unitKey}_${STATE.storyRigEngine || 'atlas_b'}"
                data-rig-key="${rigKey}"
                data-rig-engine="${STATE.storyRigEngine || 'atlas_b'}"
                data-side="${side}"
                data-pose="${activePose}"
                data-active="${isActive ? '1' : '0'}"
                data-debug="${STATE.storyShowRigDebug ? '1' : '0'}"
                style="width:288px;height:288px;filter:${filterStr};position:relative;z-index:2;"></canvas>

        ${fxLayers.front}
      </div>

      <!-- Compact Grounded Nameplate + Active VTuber Rig Pose Tag below boots -->
      <div style="margin-top:4px;padding:3px 12px;background:rgba(10,12,18,0.92);border:1px solid ${borderCol};border-radius:4px;text-align:center;box-shadow:0 4px 12px rgba(0,0,0,0.85);z-index:5;display:flex;align-items:center;gap:6px;">
        <span class="font-title" style="font-size:12.5px;color:${isActive ? 'var(--gold)' : '#9ca3af'};">${actor.name}</span>
        <span style="font-size:10px;color:var(--dim);">· ${actor.emotion || actor.title || ''}</span>
        <span class="font-mono" style="padding:1px 5px;border-radius:3px;background:rgba(17,24,39,0.95);border:1px solid ${poseBadgeCol};color:${poseBadgeCol};font-size:8.5px;">
          ${poseMeta.label}
        </span>
      </div>
    </div>
  `;
}

// Side-by-Side Rig Engine Comparison Showcase (`compare_bc` — renders Approach C [3-Pose Keyframe Sheet + Multi-Zone Deformer], Approach B [Exploded 13-Part Spine2D Atlas], and Approach A [Single-Sprite 10-Part Auto-Slice] simultaneously at 60fps!)
function renderRigEngineComparisonShowcase() {
  const activePose = (STATE.storyRigPose && STATE.storyRigPose !== 'auto') ? STATE.storyRigPose : 'weapons_drawn';
  const engines = [
    {
      engine: 'pose_c',
      badge: '🎬 APPROACH C · 3-POSE SHEET + DEFORMER (PRIMARY)',
      title: '1. Hand-Drawn Keyframe Poses + Multi-Zone Deformer',
      sub: 'Strip-mesh knee flex + 2.5D torso twist + head stabilizer',
      col: '#fbbf24'
    },
    {
      engine: 'atlas_b',
      badge: '🧩 APPROACH B · EXPLODED 13-PART ATLAS',
      title: '2. Spine2D Cutout Atlas + 2-Bone IK/FK',
      sub: 'Unoccluded chest + separate cloak + mirrored wide leg',
      col: '#34d399'
    },
    {
      engine: 'slice_a',
      badge: '✂️ APPROACH A · SINGLE-SPRITE AUTO-SLICE',
      title: '3. Neutral Sprite Sliced into 10 Limbs',
      sub: '1 image per view + circular joint caps + 2-Bone IK',
      col: '#38bdf8'
    }
  ];
  return `
    <div style="position:absolute;left:54px;right:54px;bottom:192px;height:348px;display:flex;align-items:flex-end;justify-content:space-around;z-index:12;">
      ${engines.map((e, idx) => `
        <div onclick="STATE.storyRigEngine='${e.engine}';render();"
             title="Click to select ${e.title} for the full StoryStage"
             style="display:flex;flex-direction:column;align-items:center;position:relative;width:340px;height:344px;justify-content:flex-end;cursor:pointer;">
          <div style="position:relative;width:274px;height:274px;display:flex;align-items:flex-end;justify-content:center;">
            <div style="position:absolute;left:50%;bottom:2px;transform:translateX(-50%);width:190px;height:32px;border-radius:50%;background:radial-gradient(ellipse, rgba(0,0,0,0.92) 0%, rgba(245,158,11,0.34) 58%, transparent 82%);border-bottom:2px solid ${e.col};z-index:0;"></div>
            <canvas class="vn-vtuber-canvas pixel"
                    width="340" height="340"
                    data-slot-id="compare_eng_${idx}_${e.engine}"
                    data-rig-key="kaelen"
                    data-rig-engine="${e.engine}"
                    data-side="left"
                    data-pose="${activePose}"
                    data-active="1"
                    data-debug="${STATE.storyShowRigDebug ? '1' : '0'}"
                    style="width:274px;height:274px;filter:brightness(1.06) contrast(1.06) drop-shadow(0 0 12px rgba(0,0,0,0.65)) drop-shadow(0 10px 18px rgba(0,0,0,0.92));position:relative;z-index:2;"></canvas>
          </div>
          <div style="margin-top:4px;padding:4px 12px;background:rgba(10,12,18,0.95);border:1.5px solid ${e.col};border-radius:5px;text-align:center;box-shadow:0 4px 14px rgba(0,0,0,0.92);">
            <div class="font-mono" style="font-size:8.5px;color:${e.col};font-weight:800;letter-spacing:0.4px;">${e.badge}</div>
            <div class="font-title" style="font-size:11.5px;color:#fef3c7;margin-top:1px;">${e.title}</div>
            <div class="font-mono" style="font-size:8.5px;color:#9ca3af;">${e.sub}</div>
          </div>
        </div>
      `).join('')}
    </div>
  `;
}

// 3-View Turnaround Showcase Mode (`sheet_3view` — renders Front [Down], Side [Profile], and Back [Up] simultaneously with the 60fps VTuber rig!)
function renderThreeViewTurnaroundShowcase() {
  const activePose = (STATE.storyRigPose && STATE.storyRigPose !== 'auto') ? STATE.storyRigPose : 'weapons_drawn';
  const views = [
    { key: 'kaelen_front', title: '1. FRONT VIEW (Down / South)', sub: 'DIR4: idle_down / walk_down / attack_down' },
    { key: 'kaelen_side',  title: '2. SIDE VIEW (Profile / East-West)', sub: 'VN Stage Default & DIR4 Side Profile (Mirrored)' },
    { key: 'kaelen_back',  title: '3. BACK VIEW (Up / North)', sub: 'DIR4: idle_up / walk_up / attack_up' }
  ];
  return `
    <div style="position:absolute;left:72px;right:72px;bottom:192px;height:345px;display:flex;align-items:flex-end;justify-content:space-around;z-index:12;pointer-events:none;">
      ${views.map((v, idx) => `
        <div style="display:flex;flex-direction:column;align-items:center;position:relative;width:300px;height:340px;justify-content:flex-end;">
          <div style="position:relative;width:268px;height:268px;display:flex;align-items:flex-end;justify-content:center;">
            <div style="position:absolute;left:50%;bottom:2px;transform:translateX(-50%);width:185px;height:32px;border-radius:50%;background:radial-gradient(ellipse, rgba(0,0,0,0.92) 0%, rgba(245,158,11,0.34) 58%, transparent 82%);border-bottom:2px solid rgba(251,191,36,0.78);z-index:0;"></div>
            <canvas class="vn-vtuber-canvas pixel"
                    width="340" height="340"
                    data-slot-id="turnaround_${idx}"
                    data-rig-key="${v.key}"
                    data-rig-engine="slice_a"
                    data-side="left"
                    data-pose="${activePose}"
                    data-active="1"
                    data-debug="${STATE.storyShowRigDebug ? '1' : '0'}"
                    style="width:268px;height:268px;filter:brightness(1.06) contrast(1.06) drop-shadow(0 0 12px rgba(245,158,11,0.38)) drop-shadow(0 10px 18px rgba(0,0,0,0.92));position:relative;z-index:2;"></canvas>
          </div>
          <div style="margin-top:4px;padding:4px 12px;background:rgba(10,12,18,0.94);border:1px solid #f59e0b;border-radius:4px;text-align:center;box-shadow:0 4px 12px rgba(0,0,0,0.9);">
            <div class="font-title" style="font-size:11.5px;color:var(--gold);">${v.title}</div>
            <div class="font-mono" style="font-size:9px;color:#93c5fd;">${v.sub}</div>
          </div>
        </div>
      `).join('')}
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
        <!-- APPROACH 1: VN ILLUSTRATED STAGE WITH MULTI-LAYER PARALLAX SCROLL / WIDE CAMERA PAN + STONE FLOOR + FULL-BODY SPRITES -->
        <div style="position:absolute;left:0;top:0;width:1280px;height:560px;overflow:hidden;">
          <div id="vn-bg-strip" style="position:absolute;left:0;top:0;width:3840px;height:560px;display:flex;will-change:transform;filter:${card.bgFilter || 'none'};">
            ${STATE.storyWalkMode === 'pan_camera' ? `
              <!-- Wide 1960×560 Panoramic Stage for Camera-Follow Mode (pans smoothly up to -680px as Kaelen walks across the stage) -->
              <img src="${ASSETS.bg[st.bg] || ASSETS.bg.vn_alley_stage || ASSETS.bg.intro_1}" class="pixel"
                   style="width:1960px;height:560px;object-fit:cover;object-position:center bottom;flex-shrink:0;">
            ` : `
              <!-- Seamless 3-Tile [Normal | Mirrored | Normal] Strip (Repeats every 2560px with zero seam at tile borders!) -->
              <img src="${ASSETS.bg[st.bg] || ASSETS.bg.vn_alley_stage || ASSETS.bg.intro_1}" class="pixel"
                   style="width:1280px;height:560px;object-fit:cover;object-position:center bottom;flex-shrink:0;">
              <img src="${ASSETS.bg[st.bg] || ASSETS.bg.vn_alley_stage || ASSETS.bg.intro_1}" class="pixel"
                   style="width:1280px;height:560px;object-fit:cover;object-position:center bottom;flex-shrink:0;transform:scaleX(-1);">
              <img src="${ASSETS.bg[st.bg] || ASSETS.bg.vn_alley_stage || ASSETS.bg.intro_1}" class="pixel"
                   style="width:1280px;height:560px;object-fit:cover;object-position:center bottom;flex-shrink:0;">
            `}
          </div>

          <!-- 2.5D Foreground Parallax Layer (1.45× Speed): Passing Tallow Lantern Light Shafts, Low Sunk-Ward Mist & Cobblestone Floor Markers -->
          <div id="vn-fg-parallax" style="position:absolute;left:0;top:0;width:3840px;height:560px;pointer-events:none;z-index:11;will-change:transform;display:${STATE.storyWalkMode && STATE.storyWalkMode !== 'off' ? 'block' : 'none'};">
            ${[0, 1, 2, 3, 4, 5, 6, 7].map(i => `
              <!-- Passing Overhead Tallow Street-Lantern Warm Light Cone -->
              <div style="position:absolute;left:${i * 480 + 140}px;top:0;width:180px;height:540px;background:linear-gradient(180deg, rgba(245,158,11,0.16) 0%, rgba(245,158,11,0.05) 65%, transparent 100%);transform:skewX(-12deg);"></div>
              <!-- Drifting Low Sunk-Ward Mist Wisp -->
              <div style="position:absolute;left:${i * 480 + 40}px;bottom:34px;width:310px;height:54px;border-radius:50%;background:radial-gradient(ellipse, rgba(148,163,184,0.18) 0%, rgba(56,189,248,0.06) 55%, transparent 80%);filter:blur(4px);"></div>
              <!-- Foreground Cobblestone Curb Parallax Seam -->
              <div style="position:absolute;left:${i * 480 + 210}px;bottom:8px;width:96px;height:4px;border-radius:2px;background:rgba(212,166,74,0.22);box-shadow:0 1px 4px rgba(0,0,0,0.85);"></div>
            `).join('')}
          </div>
        </div>

        <!-- Subtle top vignette only — keeps the lower stone/cobblestone floor (y: 330..540) bright and unobstructed! -->
        <div style="position:absolute;left:0;right:0;top:0;height:560px;background:linear-gradient(180deg, rgba(8,10,15,0.55) 0%, transparent 24%, transparent 82%, rgba(8,10,15,0.65) 100%);pointer-events:none;"></div>

        <!-- Top Location & Stage Floor Indicator Strip -->
        <div style="position:absolute;left:72px;right:72px;top:16px;height:36px;padding:0 16px;background:rgba(12,15,23,0.84);border:1px solid rgba(212,166,74,0.42);border-radius:4px;display:flex;align-items:center;justify-content:space-between;z-index:15;">
          <div style="display:flex;align-items:center;gap:10px;">
            <span style="padding:2px 8px;background:rgba(212,166,74,0.18);border:1px solid var(--gold);border-radius:3px;font-size:10px;color:var(--gold);font-weight:700;letter-spacing:0.5px;">VN STAGE · 60FPS VTUBER 2D RIG & VISIBLE FLOOR</span>
            <span class="font-title" style="font-size:14px;color:var(--gold);">${st.title}</span>
            <span style="font-size:12px;color:var(--dim);">· ${st.subtitle}</span>
          </div>
          <div class="font-mono" style="font-size:11px;color:#fbbf24;display:flex;align-items:center;gap:10px;">
            ${STATE.storyWalkMode && STATE.storyWalkMode !== 'off' ? `
              <span id="vn-walk-telemetry" style="padding:2px 8px;background:rgba(12,74,110,0.88);border:1px solid #38bdf8;border-radius:3px;color:#e0f2fe;font-size:10px;font-weight:700;">
                ${STATE.storyWalkMode === 'scroll_bg' ? '🔄 TREADMILL SCROLL ACTIVE' : '🎥 CAMERA FOLLOW ACTIVE'}
              </span>
              ${STATE.storyWalkMode === 'pan_camera' ? `
                <button class="rust-btn" style="height:20px;padding:0 7px;font-size:9.5px;" onclick="resetStoryCameraPan();" title="Restart Camera Pan & Hero Walk from Left Edge">↺ Reset Pan</button>
              ` : ''}
              <span>·</span>
            ` : ''}
            <span>⚙️ Engine: <b>${STATE.storyRigEngine === 'atlas_b' ? 'B: 13-Part Atlas' : STATE.storyRigEngine === 'pose_c' ? 'C: 3-Pose Sheet' : STATE.storyRigEngine === 'slice_a' ? 'A: Auto-Slice' : 'C vs B vs A Compare'}</b></span>
            <span>·</span>
            <span>🦴 Pose: <b>${STATE.storyRigPose === 'auto' ? 'Beat Default' : STATE.storyRigPose.toUpperCase()}</b></span>
            ${card.shake ? `<span style="padding:1px 7px;background:rgba(220,38,38,0.22);border:1px solid #ef4444;border-radius:3px;color:#fca5a5;">📳 ${card.shake}</span>` : ''}
          </div>
        </div>

        ${STATE.storyTurnaroundView === 'sheet_3view'
          ? renderThreeViewTurnaroundShowcase()
          : STATE.storyRigEngine === 'compare_bc'
            ? renderRigEngineComparisonShowcase()
            : `
              <!-- Left & Right Full-Body Character Sprites standing/walking directly on the stone/cobblestone floor (y: 192px above bottom) -->
              <div style="position:absolute;left:115px;right:115px;bottom:192px;height:345px;display:flex;align-items:flex-end;justify-content:space-between;pointer-events:none;z-index:10;">
                <div id="vn-left-actor-wrap" style="will-change:transform;">
                  ${renderStoryActorSlot(card.leftActor, 'left')}
                </div>
                <div id="vn-right-actor-wrap" style="will-change:transform;">
                  ${renderStoryActorSlot(card.rightActor, 'right')}
                </div>
              </div>
            `}
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

