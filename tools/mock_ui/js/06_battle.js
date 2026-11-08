// ============================================================================
// SCREEN 7: BATTLE SCREEN (`battle/draw.rs` + `stage.rs` + `skill_menu.rs` + `timeline_view.rs`)
// ============================================================================
function renderBattleScreen() {
  const b = BATTLE;
  const tpl = b.tpl;
  const heroes = b.units.filter(u => u.isHero);
  const foes = b.units.filter(u => !u.isHero);
  const selHero = b.units.find(u => u.id === STATE.selectedUnitId && u.isHero) || heroes[0];
  const pinnedFoe = b.units.find(u => u.id === STATE.pinnedFoeId && !u.isHero) || foes[0];

  // Calculate AP spent in selHero's 6-block plan
  const plannedAp = selHero.plan.filter(p => !p.ghost).reduce((s, p) => s + p.ap, 0);
  const availAp = Math.max(0, selHero.ap - plannedAp);

  // Build 6-block spans for Top Timeline Strip (`Rect(2, 2, 636, 24)`)
  let blockCellsHtml = '';
  for (let blk = 0; blk < 6; blk++) {
    const step = selHero.plan.find(p => blk >= p.start && blk < p.start + p.blocks);
    const isImpact = step && (blk === step.start + step.blocks - 1);
    const isPlayHead = STATE.playingRound && STATE.playBlockIdx === blk;
    let bg = '#10131a';
    let border = '#2d333f';
    if (step) {
      if (step.ghost) {
        bg = 'rgba(44, 34, 30, 0.45)';
        border = '#6b7280';
      } else if (isImpact) {
        bg = 'linear-gradient(180deg, #4b221b 0%, #2c1412 100%)';
        border = '#fbbf24';
      } else {
        bg = 'linear-gradient(180deg, #2b221e 0%, #1a1615 100%)';
        border = '#d97706';
      }
    }
    if (isPlayHead) {
      border = '#34d399';
    }

    blockCellsHtml += `
      <div style="flex:1;height:20px;background:${bg};border:1px solid ${border};padding:1px 3px;display:flex;align-items:center;justify-content:space-between;position:relative;opacity:${step && step.ghost ? '0.68' : '1'};">
        <div style="display:flex;align-items:center;gap:2.5px;overflow:hidden;">
          <span class="font-mono" style="font-size:5.5px;color:var(--dim);">B${blk + 1}</span>
          ${step ? `
            <img src="${getSkillIcon(step.skillId)}" class="pixel" style="width:11px;height:11px;">
            <span style="font-size:6px;color:${step.ghost ? 'var(--dim)' : 'var(--text)'};white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
              ${(SKILLS[step.skillId] || { name: step.skillId }).name}
            </span>
          ` : `<span style="font-size:5.5px;color:#4b5563;">—</span>`}
        </div>
        ${step && isImpact ? `<span style="font-size:5.5px;color:var(--gold);" title="Impact Block">★</span>` : ''}
      </div>
    `;
  }

  // 12-pip AP Gauge (`draw_ap_pips` in `battle/draw.rs`)
  let apPipsHtml = '';
  for (let i = 0; i < 12; i++) {
    let col = '#1f2430';
    if (i < availAp) col = '#fbbf24';
    else if (i < selHero.ap) col = '#b45309';
    apPipsHtml += `<div style="width:4px;height:8px;background:${col};border:0.5px solid #374151;"></div>`;
  }

  // Build 12x9 Stage Board (`BOARD = Rect(128, 82, 384, 216)`, cell = 32x24 px)
  let boardCellsHtml = '';
  for (let y = 0; y < 9; y++) {
    for (let x = 0; x < 12; x++) {
      const isWall = tpl.walls.some(([wx, wy]) => wx === x && wy === y);
      const isWater = tpl.water.some(([wx, wy]) => wx === x && wy === y);
      const haz = tpl.hazards.find(h => h.x === x && h.y === y);
      const unitOnTile = b.units.find(u => u.x === x && u.y === y && u.hp > 0);
      const isSelTile = unitOnTile && (unitOnTile.id === selHero.id || unitOnTile.id === pinnedFoe?.id);

      // Highlight range when a skill is active or SkillMenu is open
      const distFromHero = Math.abs(x - selHero.x) + Math.abs(y - selHero.y);
      const activeSk = STATE.activeSkillId ? SKILLS[STATE.activeSkillId] : null;
      const inRange = activeSk && !isWall && distFromHero <= (activeSk.range || 1);

      let tileBg = (x + y) % 2 === 0 ? '#181b24' : '#141720';
      let tileBorder = 'rgba(75,85,99,0.28)';
      if (isWater) {
        tileBg = '#0b1926';
        tileBorder = 'rgba(6,182,212,0.35)';
      } else if (isWall) {
        tileBg = '#252934';
        tileBorder = '#4b5563';
      }
      if (haz) {
        if (haz.type === 'slag') {
          tileBg = 'radial-gradient(circle, rgba(234,88,12,0.42) 0%, #1c1412 100%)';
          tileBorder = '#ea580c';
        } else if (haz.type === 'winch') {
          tileBg = 'radial-gradient(circle, rgba(251,191,36,0.38) 0%, #1c1914 100%)';
          tileBorder = '#fbbf24';
        } else {
          tileBg = 'radial-gradient(circle, rgba(217,119,6,0.28) 0%, #181b24 100%)';
          tileBorder = '#d97706';
        }
      }
      if (inRange) {
        tileBg = activeSk.id === 'move' ? 'rgba(6,182,212,0.24)' : 'rgba(225,29,72,0.25)';
        tileBorder = activeSk.id === 'move' ? '#06b6d4' : '#e11d48';
      }

      boardCellsHtml += `
        <div onclick="clickBattleTile(${x},${y},event)"
             style="position:absolute;left:${x * 32}px;top:${y * 24}px;width:32px;height:24px;background:${tileBg};border:0.5px solid ${tileBorder};cursor:pointer;">
          ${isWall ? `<div style="position:absolute;left:0;right:0;top:-5px;height:9px;background:#313744;border-top:1px solid #6b7280;border-bottom:1px solid #111827;"></div>` : ''}
          ${haz ? `<div style="position:absolute;left:2px;bottom:1px;font-size:5px;color:var(--gold);pointer-events:none;">${haz.type.toUpperCase()}</div>` : ''}
          ${isSelTile ? `<div style="position:absolute;inset:1px;border:1px solid var(--gold);pointer-events:none;"></div>` : ''}
        </div>
      `;
    }
  }

  // Determine current Battle Stage Mode (`pre_battle` | `transition_in` | `active_combat` | `boss_phase2` | `elite_intro` | `elite_aftermath`)
  const stageMode = STATE.battleStageMode || 'active_combat';
  const isPreBattle = stageMode === 'pre_battle';
  const isTransitionIn = stageMode === 'transition_in';
  const encBarks = (typeof COMBAT_BARKS !== 'undefined' && COMBAT_BARKS[b.encId]) ? COMBAT_BARKS[b.encId] : (typeof COMBAT_BARKS !== 'undefined' ? COMBAT_BARKS.enc_0c : null);
  const activeBark = (encBarks && (stageMode === 'pre_battle' || stageMode === 'boss_phase2' || stageMode === 'elite_intro' || stageMode === 'elite_aftermath'))
    ? (encBarks[stageMode] || encBarks.pre_battle)
    : null;

  // Units on the 12x9 Stage (`stage.rs` sprites + overhead HP/Poise bars + Boss Phase 2 / Speaking Aura)
  const stageUnitsHtml = b.units.filter(u => u.hp > 0).map(u => {
    const px = u.x * 32;
    const py = u.y * 24;
    const frames = ASSETS.units[u.unitKey] || ASSETS.units.kaelen;
    const isBarkFocus = activeBark && activeBark.focusUnitId === u.id;
    const spriteUri = (STATE.playingRound || isBarkFocus) ? (frames.atk || frames.idle0) : frames.idle0;
    const flip = !u.isHero ? 'transform:scaleX(-1);' : '';
    const hpPct = Math.round((u.hp / u.maxHp) * 100);

    return `
      <div onclick="clickBattleTile(${u.x},${u.y},event)"
           style="position:absolute;left:${px - 2}px;top:${py - 14}px;width:36px;height:38px;cursor:pointer;z-index:${10 + u.y};display:flex;flex-direction:column;align-items:center;">
        ${isBarkFocus ? `
          <!-- Boss Phase 2 / Scripted Bark Tile Shockwave Ring & Overhead Callout -->
          <div style="position:absolute;left:50%;top:62%;width:44px;height:44px;border-radius:50%;border:1.5px solid ${stageMode === 'boss_phase2' ? '#ef4444' : '#fbbf24'};animation:vnPulseRing 1.15s ease-out infinite;pointer-events:none;"></div>
          <div style="position:absolute;top:-11px;padding:0.5px 4px;border-radius:6px;background:${stageMode === 'boss_phase2' ? '#7f1d1d' : '#fbbf24'};color:${stageMode === 'boss_phase2' ? '#fecaca' : '#090b10'};border:0.5px solid ${stageMode === 'boss_phase2' ? '#ef4444' : '#d97706'};font-size:5px;font-weight:800;white-space:nowrap;z-index:25;box-shadow:0 2px 5px #000;">
            ${stageMode === 'boss_phase2' ? '⚡ PHASE 2' : '💬 SPEAKING'}
          </div>
        ` : ''}
        <!-- Overhead Micro HP & Poise Bar (hidden during pre_battle map confrontation for clean map immersion) -->
        ${!isPreBattle ? `
          <div style="width:24px;height:3px;background:#08090d;border:0.5px solid #000;margin-bottom:1px;">
            <div style="width:${hpPct}%;height:100%;background:${u.isHero ? '#10b981' : '#e11d48'};"></div>
          </div>
          <div style="display:flex;gap:1px;margin-bottom:-2px;">
            ${Array.from({ length: u.maxPoise }).map((_, pi) => `
              <div style="width:3px;height:2px;background:${pi < u.poise ? '#fbbf24' : '#374151'};"></div>
            `).join('')}
          </div>
        ` : `<div style="height:5px;"></div>`}
        <img src="${spriteUri}" class="pixel" style="width:32px;height:32px;${flip}filter:${isBarkFocus ? 'drop-shadow(0 0 6px rgba(251,191,36,0.85))' : 'drop-shadow(0 2px 2px rgba(0,0,0,0.9))'};">
      </div>
    `;
  }).join('');

  // Icon-Only `SkillMenu` (`battle/skill_menu.rs`: ICON = 15, MENU_MIN_W = 170, clamped beside clicked tile)
  let skillMenuHtml = '';
  if (STATE.skillMenu && stageMode === 'active_combat') {
    const sm = STATE.skillMenu;
    const hModel = STATE.roster[selHero.id] || STATE.roster.kaelen;
    const allHeroSkills = [...hModel.basics, ...hModel.actives];
    const byRow = rowName => allHeroSkills.map(id => SKILLS[id]).filter(Boolean).filter(s => s.row === rowName);
    const hovSk = sm.hoveredSkillId ? SKILLS[sm.hoveredSkillId] : SKILLS[hModel.actives[0]];

    const menuLeft = Math.min(445, Math.max(132, 128 + sm.tileX * 32 + 28));
    const menuTop = Math.min(205, Math.max(36, 82 + sm.tileY * 24 - 16));

    skillMenuHtml = ornateCornerBox640(`position:absolute;left:${menuLeft}px;top:${menuTop}px;width:176px;z-index:35;padding:5px;background:rgba(10,11,16,0.97);`, `
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #2d333f;padding-bottom:2px;margin-bottom:3px;">
          <span class="font-title" style="font-size:7.5px;color:var(--gold);">${selHero.name} · Tile (${sm.tileX},${sm.tileY})</span>
          ${closeBtn640("STATE.skillMenu=null;render();", "Close SkillMenu [Esc / Right-Click]")}
        </div>

        <!-- Row 0: Any time (Fill | Move | Rest | Clear) -->
        <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:2.5px;">
          <span style="font-size:6px;color:var(--dim);width:42px;">Any time</span>
          <div style="display:flex;gap:2.5px;flex:1;">
            <button class="rust-btn primary" style="height:14px;font-size:6px;padding:0 4px;" onclick="autoFillHeroPlan('${selHero.id}')">Fill</button>
            ${['move', 'rest', 'disengage'].map(sid => `
              <div class="rust-card" style="width:16px;height:16px;display:flex;align-items:center;justify-content:center;cursor:pointer;"
                   onmouseenter="STATE.skillMenu.hoveredSkillId='${sid}';render();"
                   onclick="queueSkillOnTile('${sid}', ${sm.tileX}, ${sm.tileY})">
                <img src="${getSkillIcon(sid)}" class="pixel" style="width:12px;height:12px;">
              </div>
            `).join('')}
          </div>
        </div>

        <!-- Intent Rows: Assault, Guard, Support, Maneuver (skill_menu.rs) -->
        ${['Assault', 'Guard', 'Support', 'Maneuver'].map(rName => {
          const rowSkills = byRow(rName);
          if (!rowSkills.length) return '';
          return `
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:2.5px;">
              <span style="font-size:6px;color:var(--dim);width:42px;">${rName}</span>
              <div style="display:flex;gap:2.5px;flex:1;">
                ${rowSkills.map(sk => `
                  <div class="rust-card ${sm.hoveredSkillId===sk.id?'selected':''}"
                       style="width:16px;height:16px;display:flex;align-items:center;justify-content:center;cursor:pointer;"
                       onmouseenter="STATE.skillMenu.hoveredSkillId='${sk.id}';render();"
                       onclick="queueSkillOnTile('${sk.id}', ${sm.tileX}, ${sm.tileY})">
                    <img src="${getSkillIcon(sk.id)}" class="pixel" style="width:12px;height:12px;">
                  </div>
                `).join('')}
              </div>
            </div>
          `;
        }).join('')}

        <!-- Hovered Skill Footer (skill_menu.rs single-line detail) -->
        ${hovSk ? `
          <div style="border-top:1px solid #2d333f;padding-top:3px;margin-top:2px;">
            <div style="display:flex;justify-content:space-between;font-size:6.5px;">
              <b style="color:var(--gold);">${hovSk.name}</b>
              <span class="font-mono" style="color:var(--info);">${hovSk.blocks}B · ${hovSk.ap}AP · ${hovSk.shape}</span>
            </div>
            <div style="font-size:5.8px;color:var(--dim);line-height:1.25;margin-top:1px;">${hovSk.desc}</div>
          </div>
        ` : ''}
    `);
  }

  // Scripted Dialogue Dock (rendered during `pre_battle`, `boss_phase2`, `elite_intro`, `elite_aftermath`)
  let scriptedBarkHtml = '';
  if (activeBark) {
    const barkPortrait = getPortraitWithMood(activeBark.portrait, activeBark.mood);
    const isPhase2 = stageMode === 'boss_phase2';
    const badgeBg = isPhase2 ? 'rgba(127,29,29,0.9)' : 'rgba(245,158,11,0.18)';
    const badgeBorder = isPhase2 ? '#ef4444' : '#f59e0b';
    const badgeColor = isPhase2 ? '#fecaca' : '#fbbf24';

    let primaryBtnHtml = '';
    if (stageMode === 'pre_battle') {
      primaryBtnHtml = `
        <button class="rust-btn primary" style="height:22px;padding:0 12px;font-size:7.5px;" onclick="engageCombatFromMap()">
          ⚔ Engage Combat (Slide HUD In) [Space] ▸
        </button>
      `;
    } else if (stageMode === 'elite_intro') {
      primaryBtnHtml = `
        <button class="rust-btn" style="height:20px;padding:0 8px;font-size:6.8px;" onclick="STATE.battleStageMode='elite_aftermath';render();">
          Preview Post-Elite Bark ▸
        </button>
        <button class="rust-btn primary" style="height:22px;padding:0 12px;font-size:7.5px;" onclick="STATE.battleStageMode='active_combat';render();">
          Resume Combat [Space] ▸
        </button>
      `;
    } else {
      primaryBtnHtml = `
        <button class="rust-btn primary" style="height:22px;padding:0 12px;font-size:7.5px;" onclick="STATE.battleStageMode='active_combat';render();">
          Resume Combat [Space] ▸
        </button>
      `;
    }

    scriptedBarkHtml = ornateCornerBox640(`position:absolute;left:54px;right:54px;bottom:6px;height:94px;z-index:38;padding:6px 10px;background:rgba(10,12,18,0.97);box-shadow:0 10px 28px rgba(0,0,0,0.95);`, `
      <div class="hud-slide-up" style="display:flex;gap:10px;align-items:stretch;height:100%;">
        <!-- Speaker Portrait with Mood Expression (faces/<id>/<mood>.png) -->
        <div style="display:flex;flex-direction:column;align-items:center;justify-content:space-between;width:62px;flex-shrink:0;">
          <div class="portrait-box ${isPhase2 ? 'foe-rim' : 'gold-rim'}" style="width:54px;height:54px;position:relative;box-shadow:0 0 10px ${isPhase2 ? 'rgba(239,68,68,0.45)' : 'rgba(245,158,11,0.35)'};">
            <img src="${barkPortrait}" class="pixel" style="width:100%;height:100%;object-fit:cover;">
            <span style="position:absolute;right:1px;bottom:1px;padding:0.5px 3px;background:rgba(8,10,15,0.92);border:0.5px solid ${badgeBorder};font-family:'JetBrainsMono',monospace;font-size:4.8px;color:${badgeColor};text-transform:uppercase;">
              ${activeBark.mood}
            </span>
          </div>
          <span style="font-size:5.2px;color:${badgeColor};font-weight:700;text-transform:uppercase;">Mood: ${activeBark.mood}</span>
        </div>

        <!-- Scripted Dialogue Text + Phase/Elite Mechanic Callout + Action Buttons -->
        <div style="flex:1;display:flex;flex-direction:column;justify-content:space-between;">
          <div>
            <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:0.5px solid #374151;padding-bottom:2px;margin-bottom:3px;">
              <div style="display:flex;align-items:center;gap:6px;">
                <span style="padding:1px 5px;background:${badgeBg};border:0.5px solid ${badgeBorder};border-radius:2px;font-size:5.5px;color:${badgeColor};font-weight:800;letter-spacing:0.4px;">
                  ${activeBark.badge}
                </span>
                <span class="font-title" style="font-size:9px;color:var(--gold);">${activeBark.speaker}</span>
                <span style="font-size:6px;color:var(--dim);">· ${activeBark.speakerTitle}</span>
              </div>
              ${stageMode !== 'pre_battle' ? closeBtn640("STATE.battleStageMode='active_combat';render();", "Resume Combat [Esc / Right-Click]") : ''}
            </div>
            <div style="font-size:7.3px;line-height:1.36;color:var(--text);margin-bottom:3px;">
              ${activeBark.text}
            </div>
          </div>

          <div style="display:flex;justify-content:space-between;align-items:center;border-top:0.5px solid rgba(55,65,81,0.7);padding-top:3px;">
            <div class="font-mono" style="font-size:5.6px;color:#93c5fd;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;max-width:305px;">
              ⚙ ${activeBark.mechanicHint}
            </div>
            <div style="display:flex;gap:4px;align-items:center;flex-shrink:0;">
              ${primaryBtnHtml}
            </div>
          </div>
        </div>
      </div>
    `);
  }

  return `
    <div style="width:640px;height:360px;position:relative;background:#0a0b10;overflow:hidden;">

      ${isPreBattle ? `
        <!-- PRE-BATTLE MAP CONFRONTATION HEADER (Before Combat HUD Slides In) -->
        <div style="position:absolute;left:54px;right:54px;top:4px;height:20px;padding:0 8px;background:rgba(12,15,23,0.92);border:0.5px solid rgba(212,166,74,0.5);border-radius:2px;display:flex;align-items:center;justify-content:space-between;z-index:25;">
          <div style="display:flex;align-items:center;gap:6px;">
            <span style="padding:1px 4px;background:rgba(245,158,11,0.2);border:0.5px solid #f59e0b;font-size:5.5px;color:#fbbf24;font-weight:700;">ENCOUNTER MAP STAGE</span>
            <span class="font-title" style="font-size:8px;color:var(--gold);">${tpl.title}</span>
          </div>
          <span class="font-mono" style="font-size:5.8px;color:var(--dim);">Click [⚔ Engage Combat] below to transition into the Combat UI</span>
        </div>
      ` : `
        <!-- TOP 6-BLOCK TIMELINE STRIP (battle/draw.rs: Rect(2, 2, 636, 24)) -->
        <div class="rust-panel ${isTransitionIn ? 'hud-slide-down' : ''}" style="position:absolute;left:2px;top:2px;width:636px;height:25px;padding:2px 5px;display:flex;align-items:center;gap:6px;z-index:25;">
          <!-- Hero Name + Intent + Class Meter (NAME_W = 104) -->
          <div style="width:104px;flex-shrink:0;display:flex;flex-direction:column;justify-content:center;">
            <div style="display:flex;align-items:center;justify-content:space-between;">
              <span class="font-title" style="font-size:8px;color:var(--gold);">${selHero.name} plan</span>
              <span class="font-mono" style="font-size:5.8px;color:var(--dim);">R${STATE.round}</span>
            </div>
            <div style="display:flex;align-items:center;gap:3px;margin-top:1px;">
              <span style="font-size:5.5px;color:var(--dim);">${selHero.meterName}:</span>
              <div style="flex:1;height:4px;background:#08090d;border:0.5px solid #374151;">
                <div style="width:${selHero.meter}%;height:100%;background:#d97706;"></div>
              </div>
            </div>
          </div>

          <!-- 6 Discrete Block Spans (B1..B6) -->
          <div style="flex:1;display:flex;gap:2px;">
            ${blockCellsHtml}
          </div>

          <!-- Right 12-Pip AP Gauge (AP_W = 86) -->
          <div style="width:88px;flex-shrink:0;display:flex;flex-direction:column;align-items:flex-end;justify-content:center;">
            <div style="font-size:6px;color:var(--gold);font-family:'JetBrainsMono',monospace;">AP ${availAp}/${selHero.ap} (Max 12)</div>
            <div style="display:flex;gap:1.5px;margin-top:2px;">${apPipsHtml}</div>
          </div>
        </div>

        <!-- LEFT PARTY COLUMN (battle/draw.rs: x=2..124, y=30..318) -->
        <div class="${isTransitionIn ? 'hud-slide-down' : ''}" style="position:absolute;left:2px;top:30px;width:122px;height:288px;display:flex;flex-direction:column;gap:4px;overflow-y:auto;opacity:${activeBark ? '0.55' : '1'};transition:opacity 0.2s;">
          ${heroes.map(h => {
            const isSel = h.id === selHero.id;
            const hpPct = Math.round((h.hp / h.maxHp) * 100);
            return `
              <div class="rust-card ${isSel ? 'selected' : ''}"
                   style="padding:4px 5px;cursor:pointer;display:flex;flex-direction:column;gap:3px;"
                   onclick="STATE.selectedUnitId='${h.id}';render();">
                <div style="display:flex;gap:5px;align-items:center;">
                  <div class="portrait-box gold-rim" style="width:28px;height:28px;">
                    <img src="${ASSETS.portraits[h.portrait]}" class="pixel">
                  </div>
                  <div style="flex:1;overflow:hidden;">
                    <div style="display:flex;justify-content:space-between;align-items:baseline;">
                      <span class="font-title" style="font-size:8px;color:var(--gold);">${h.name}</span>
                      <span class="font-mono" style="font-size:6px;color:#60a5fa;">G:${h.guard}</span>
                    </div>
                    <div style="width:100%;height:4px;background:#08090d;border:0.5px solid #374151;margin-top:2px;">
                      <div style="width:${hpPct}%;height:100%;background:#10b981;"></div>
                    </div>
                    <div style="display:flex;justify-content:space-between;align-items:center;margin-top:2px;">
                      <span class="font-mono" style="font-size:5.8px;color:var(--text);">${h.hp}/${h.maxHp} HP</span>
                      <div style="display:flex;gap:1px;" title="Poise Pips">
                        ${Array.from({ length: h.maxPoise }).map((_, pi) => `
                          <div style="width:3.5px;height:3.5px;background:${pi < h.poise ? '#fbbf24' : '#374151'};"></div>
                        `).join('')}
                      </div>
                    </div>
                  </div>
                </div>
                <!-- 6-pip mini plan preview -->
                <div style="display:flex;gap:1.5px;">
                  ${[0,1,2,3,4,5].map(bIdx => {
                    const has = h.plan.find(p => bIdx >= p.start && bIdx < p.start + p.blocks);
                    return `<div style="flex:1;height:3px;background:${has ? (has.ghost ? '#6b7280' : '#d97706') : '#1f2430'};"></div>`;
                  }).join('')}
                </div>
              </div>
            `;
          }).join('')}

          <!-- Active Hero Belt Consumables Quick-Use -->
          <div class="rust-panel" style="padding:4px 5px;margin-top:auto;">
            <div class="font-title" style="font-size:7px;color:var(--gold);margin-bottom:2px;">${selHero.name}'s 5-Pip Belt (0B Free)</div>
            ${(STATE.roster[selHero.id]?.pouch || []).map((item, idx) => `
              <div class="rust-card" style="padding:2.5px 4px;margin-bottom:2px;font-size:6px;cursor:pointer;display:flex;justify-content:space-between;align-items:center;"
                   onclick="useBeltItem('${selHero.id}', ${idx})">
                <span>${item.name}</span>
                <span class="font-mono" style="color:var(--gold);">${item.pips}p ▸</span>
              </div>
            `).join('')}
          </div>
        </div>
      `}

      <!-- CENTER BATTLE STAGE (stage.rs: INTERIOR = Rect(128, 30, 384, 288)) -->
      <div style="position:absolute;left:128px;top:${isPreBattle ? '28px' : '30px'};width:384px;height:${isPreBattle ? '230px' : '288px'};border:1px solid ${stageMode === 'boss_phase2' ? '#ef4444' : '#374151'};background:#08090d;overflow:hidden;box-shadow:${stageMode === 'boss_phase2' ? '0 0 18px rgba(239,68,68,0.4)' : 'none'};">
        <!-- BACKDROP = Rect(128, 30, 384, 52) -->
        <div style="position:relative;width:384px;height:52px;overflow:hidden;border-bottom:1px solid #4b5563;">
          <img src="${ASSETS.bg[tpl.backdrop] || ASSETS.bg.canal}" class="pixel" style="width:384px;height:52px;object-fit:cover;">
          <div style="position:absolute;inset:0;background:linear-gradient(90deg, rgba(10,11,16,0.75) 0%, rgba(10,11,16,0.2) 50%, rgba(10,11,16,0.75) 100%);padding:4px 8px;display:flex;justify-content:space-between;align-items:flex-start;">
            <div>
              <div class="font-title" style="font-size:9px;color:var(--gold);text-shadow:0 1px 2px #000;">${tpl.title}</div>
              <div style="font-size:6px;color:var(--text);text-shadow:0 1px 2px #000;">${tpl.subtitle}</div>
            </div>
            <span class="font-mono" style="font-size:6px;background:rgba(10,11,16,0.85);padding:1px 4px;border:0.5px solid #d97706;color:var(--gold);">
              ${tpl.objective}
            </span>
          </div>
        </div>

        <!-- BOARD = 12x9 tiles (384x216 px) -->
        <div style="position:relative;width:384px;height:216px;">
          ${boardCellsHtml}
          ${stageUnitsHtml}
        </div>

        <!-- FRONT_FACE = 384x12 px stone lip (stage.rs STAGE_LIP = 12) -->
        <div style="width:384px;height:14px;background:linear-gradient(180deg, #262b38 0%, #11141c 100%);border-top:1px solid #4b5563;display:flex;align-items:center;justify-content:space-between;padding:0 8px;font-size:5.8px;color:var(--dim);">
          <span>Click any tile or unit to open the icon-only <b>SkillMenu</b> (skill_menu.rs)</span>
          <span>12×9 Tactical Grid · 6 Blocks / Round</span>
        </div>
      </div>

      ${isTransitionIn ? `
        <!-- ANIMATED MAP → COMBAT ENGAGEMENT BANNER -->
        <div class="combat-banner-flash" style="position:absolute;left:90px;right:90px;top:148px;height:54px;background:linear-gradient(90deg, transparent 0%, rgba(12,15,23,0.96) 18%, rgba(44,18,22,0.98) 50%, rgba(12,15,23,0.96) 82%, transparent 100%);border-top:1.5px solid #fbbf24;border-bottom:1.5px solid #fbbf24;display:flex;flex-direction:column;align-items:center;justify-content:center;z-index:42;pointer-events:none;box-shadow:0 10px 30px rgba(0,0,0,0.9);">
          <div class="font-title" style="font-size:13px;color:#fef3c7;letter-spacing:2px;text-shadow:0 2px 8px #000;">⚔ ENCOUNTER ENGAGED — FORESIGHT ACTIVE ⚔</div>
          <div class="font-mono" style="font-size:6.5px;color:#fbbf24;margin-top:2px;">Round 1 · Enemy Intents Telegraphed on 6-Block Timeline</div>
        </div>
      ` : ''}

      ${skillMenuHtml}

      ${!isPreBattle ? `
        <!-- RIGHT HOSTILE CARDS + PINNED INTEL PANEL (battle/draw.rs: x=516..638, y=30..318) -->
        <div class="${isTransitionIn ? 'hud-slide-down' : ''}" style="position:absolute;left:516px;top:30px;width:122px;height:288px;display:flex;flex-direction:column;gap:4px;opacity:${activeBark ? '0.55' : '1'};transition:opacity 0.2s;">
          <div style="display:flex;flex-direction:column;gap:3px;overflow-y:auto;max-height:168px;">
            ${foes.map(f => {
              const isPinned = pinnedFoe && f.id === pinnedFoe.id;
              const hpPct = Math.max(0, Math.round((f.hp / f.maxHp) * 100));
              return `
                <div class="rust-card ${isPinned ? 'selected' : ''}"
                     style="padding:4px 5px;cursor:pointer;opacity:${f.hp <= 0 ? '0.4' : '1'};"
                     onclick="STATE.pinnedFoeId='${f.id}';render();">
                  <div style="display:flex;gap:5px;align-items:center;">
                    <div class="portrait-box foe-rim" style="width:26px;height:26px;">
                      <img src="${ASSETS.portraits[f.portrait] || ASSETS.portraits.watchman}" class="pixel">
                    </div>
                    <div style="flex:1;overflow:hidden;">
                      <div style="display:flex;justify-content:space-between;align-items:center;">
                        <span class="font-title" style="font-size:7.5px;color:#fda4af;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${f.name}</span>
                        <img src="${ASSETS.ui['intent_' + f.intent] || ASSETS.ui.intent_attack}" class="pixel" style="width:10px;height:10px;" title="Foresight Intent">
                      </div>
                      <div style="width:100%;height:4px;background:#08090d;border:0.5px solid #374151;margin-top:2px;">
                        <div style="width:${hpPct}%;height:100%;background:#e11d48;"></div>
                      </div>
                      <div style="display:flex;justify-content:space-between;align-items:center;margin-top:2px;">
                        <span class="font-mono" style="font-size:5.8px;color:var(--text);">${f.hp}/${f.maxHp} · G:${f.guard}</span>
                        <div style="display:flex;gap:1px;">
                          ${Array.from({ length: f.maxPoise }).map((_, pi) => `
                            <div style="width:3.5px;height:3.5px;background:${pi < f.poise ? '#fbbf24' : '#374151'};"></div>
                          `).join('')}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              `;
            }).join('')}
          </div>

          <!-- Pinned Enemy Intel Card (battle/draw.rs lower-right inspector — Reduced Corner Frame) -->
          ${pinnedFoe ? ornateCornerBox640('margin-top:auto;padding:5px 6px;display:flex;flex-direction:column;gap:3px;', `
              <div style="display:flex;justify-content:space-between;align-items:center;">
                <span class="font-title" style="font-size:7.5px;color:var(--gold);">Intel · ${pinnedFoe.name}</span>
                <img src="${ASSETS.ui.badge_foresight}" class="pixel" style="width:9px;height:9px;">
              </div>
              <div style="font-size:5.8px;color:var(--info);">${pinnedFoe.role}</div>
              <div style="font-size:6px;color:#fda4af;background:rgba(44,20,24,0.8);padding:2px 4px;border:0.5px solid #991b1b;">
                <b>Intent:</b> ${pinnedFoe.intentName}
              </div>
              <div style="font-size:5.8px;color:var(--dim);line-height:1.3;">
                <b style="color:var(--text);">Trait:</b> ${pinnedFoe.passive}
              </div>
          `) : ''}
        </div>
      ` : ''}

      ${!activeBark ? `
        <!-- BOTTOM BAR (battle/draw.rs: Rect(2, 322, 636, 36)) -->
        <div class="rust-panel ${isTransitionIn ? 'hud-slide-up' : ''}" style="position:absolute;left:2px;top:321px;width:636px;height:37px;padding:3px 6px;display:flex;align-items:center;justify-content:space-between;gap:8px;z-index:25;">
          <!-- Combat Log Ticker -->
          <div class="rust-inset" style="flex:1;height:29px;padding:2px 5px;overflow-y:auto;font-size:6.2px;color:var(--dim);display:flex;flex-direction:column;justify-content:center;">
            ${STATE.combatLog.slice(0, 2).map((l, i) => `
              <div style="color:${i===0?'var(--text)':'var(--dim)'};">• ${l}</div>
            `).join('')}
          </div>

          <!-- Round & Timeline Controls -->
          <div style="display:flex;align-items:center;gap:4px;">
            <button class="rust-btn" onclick="clearHeroPlan('${selHero.id}')">Clear [Bksp]</button>
            <button class="rust-btn" onclick="autoFillHeroPlan('${selHero.id}')">Auto-Fill</button>
            <button class="rust-btn ${STATE.modal==='timeline'?'primary':''}" onclick="STATE.modal = STATE.modal==='timeline'?null:'timeline';render();">
              Timeline [T]
            </button>
            <button class="rust-btn primary" style="height:22px;padding:0 10px;font-size:8px;" onclick="executeBattleRound()">
              Confirm Round [Space] ▸
            </button>
          </div>
        </div>
      ` : scriptedBarkHtml}

      <!-- Multi-Track Timeline Overlay (timeline_view.rs) -->
      ${STATE.modal === 'timeline' ? renderTimelineOverlay() : ''}
      <!-- Victory / Settlement Results Modal -->
      ${STATE.modal === 'results' ? renderBattleResultsModal() : ''}
    </div>
  `;
}

function clickBattleTile(x, y, ev) {
  if (ev) ev.stopPropagation();
  const b = BATTLE;
  const unitAt = b.units.find(u => u.x === x && u.y === y && u.hp > 0);
  if (unitAt && unitAt.isHero && !STATE.skillMenu) {
    STATE.selectedUnitId = unitAt.id;
  } else if (unitAt && !unitAt.isHero) {
    STATE.pinnedFoeId = unitAt.id;
  }
  const selHero = b.units.find(u => u.id === STATE.selectedUnitId && u.isHero) || b.units[0];
  const hModel = STATE.roster[selHero.id] || STATE.roster.kaelen;
  const defaultSk = (unitAt && !unitAt.isHero) ? hModel.actives[0] : 'move';

  STATE.skillMenu = {
    tileX: x,
    tileY: y,
    hoveredSkillId: defaultSk
  };
  render();
}

function queueSkillOnTile(skillId, tx, ty) {
  const b = BATTLE;
  const selHero = b.units.find(u => u.id === STATE.selectedUnitId && u.isHero);
  const sk = SKILLS[skillId];
  if (!selHero || !sk) return;

  // Remove ghost steps first
  selHero.plan = selHero.plan.filter(p => !p.ghost);
  const usedBlocks = selHero.plan.reduce((acc, p) => Math.max(acc, p.start + p.blocks), 0);
  const usedAp = selHero.plan.reduce((acc, p) => acc + p.ap, 0);

  if (usedBlocks + sk.blocks <= 6 && usedAp + sk.ap <= selHero.ap) {
    const foe = b.units.find(u => u.x === tx && u.y === ty && !u.isHero && u.hp > 0);
    selHero.plan.push({
      skillId,
      start: usedBlocks,
      blocks: sk.blocks,
      ap: sk.ap,
      targetText: foe ? foe.name : `(${tx},${ty})`,
      tx,
      ty,
      ghost: false
    });
    if (skillId === 'move' && !foe) {
      selHero.x = tx;
      selHero.y = ty;
    }
    STATE.combatLog.unshift(`${selHero.name} queued ${sk.name} (B${usedBlocks + 1}-B${usedBlocks + sk.blocks}, ${sk.ap} AP) → ${foe ? foe.name : `Tile (${tx},${ty})`}.`);
  } else {
    STATE.combatLog.unshift(`Not enough Blocks or AP left in Round ${STATE.round} for ${sk.name}.`);
  }
  STATE.skillMenu = null;
  render();
}

function clearHeroPlan(heroId) {
  const u = BATTLE.units.find(x => x.id === heroId);
  if (u) u.plan = [];
  STATE.skillMenu = null;
  render();
}

function autoFillHeroPlan(heroId) {
  const u = BATTLE.units.find(x => x.id === heroId);
  const h = STATE.roster[heroId];
  if (!u || !h) return;
  u.plan = [
    { skillId: h.actives[0], start: 0, blocks: SKILLS[h.actives[0]].blocks, ap: SKILLS[h.actives[0]].ap, targetText: 'Nearest Foe', ghost: false },
    { skillId: h.basics[5], start: SKILLS[h.actives[0]].blocks, blocks: SKILLS[h.basics[5]].blocks, ap: SKILLS[h.basics[5]].ap, targetText: 'Nearest Foe', ghost: false },
    { skillId: 'active_guard', start: Math.min(4, SKILLS[h.actives[0]].blocks + SKILLS[h.basics[5]].blocks), blocks: 2, ap: 1, targetText: 'Self', ghost: true }
  ];
  STATE.skillMenu = null;
  STATE.combatLog.unshift(`Auto-filled ${u.name}'s 6-block timeline using ${h.intentOrder[0]} priority.`);
  render();
}

function useBeltItem(heroId, itemIdx) {
  const h = STATE.roster[heroId];
  const u = BATTLE.units.find(x => x.id === heroId);
  const item = h?.pouch[itemIdx];
  if (!u || !item) return;
  u.hp = Math.min(u.maxHp, u.hp + 15);
  u.guard += 6;
  STATE.combatLog.unshift(`${u.name} used ${item.name} (0B Free Belt Action): +15 HP, +6 Guard!`);
  render();
}

function executeBattleRound() {
  STATE.skillMenu = null;
  const b = BATTLE;
  const foes = b.units.filter(u => !u.isHero && u.hp > 0);
  const heroes = b.units.filter(u => u.isHero && u.hp > 0);

  // Apply hero attacks to foes
  heroes.forEach(h => {
    h.plan.forEach(step => {
      const sk = SKILLS[step.skillId];
      if (!sk) return;
      if (sk.dmg && foes.length > 0) {
        const target = foes[0];
        const dmg = sk.dmg;
        const absorbed = Math.min(target.guard, Math.floor(dmg * 0.5));
        target.guard -= absorbed;
        target.hp = Math.max(0, target.hp - (dmg - absorbed));
        target.poise = Math.max(0, target.poise - (sk.poiseDmg || 1));
        if (target.poise === 0) {
          target.guard = 0;
          STATE.combatLog.unshift(`POISE BROKEN! ${target.name} is Dazed (Guard shattered, actions interrupted)!`);
        }
      }
      if (sk.guard) h.guard += sk.guard;
      if (sk.heal) h.hp = Math.min(h.maxHp, h.hp + sk.heal);
    });
  });

  // Even-round Slag Vent hazard in M3
  if (b.encId === 'enc_m3' && STATE.round % 2 === 1) {
    STATE.combatLog.unshift('SLAG VENTS ERUPT! White-hot foundry steam sears the grates for 10 Burn damage!');
    if (foes[0]) foes[0].hp = Math.max(0, foes[0].hp - 10);
  }

  // Check if all foes defeated or boss broken
  const aliveFoes = b.units.filter(u => !u.isHero && u.hp > 0);
  if (aliveFoes.length === 0 || STATE.round >= 2) {
    aliveFoes.forEach(f => { f.hp = 0; });
    STATE.modal = 'results';
  } else {
    STATE.round += 1;
    if (!b.phase2Triggered) {
      b.phase2Triggered = true;
      if (aliveFoes[0]) {
        aliveFoes[0].hp = Math.min(aliveFoes[0].hp, Math.ceil(aliveFoes[0].maxHp * 0.48));
        aliveFoes[0].guard += 10;
      }
      STATE.battleStageMode = 'boss_phase2';
      STATE.combatLog.unshift(`⚡ PHASE 2 TRANSFORMATION! ${aliveFoes[0] ? aliveFoes[0].name : 'Boss'} crosses 50% HP threshold!`);
    } else {
      STATE.combatLog.unshift(`Round ${STATE.round} begins — +6 AP replenished across squad.`);
    }
  }
  render();
}

// ============================================================================
// MULTI-TRACK TIMELINE OVERLAY (`timeline_view.rs` — [T] Key)
// ============================================================================
function renderTimelineOverlay() {
  const b = BATTLE;
  return `
    <div style="position:absolute;inset:0;background:rgba(0,0,0,0.72);z-index:40;display:flex;align-items:center;justify-content:center;">
      ${ornateBox640('width:540px;height:280px;padding:8px 10px;display:flex;flex-direction:column;justify-content:space-between;background:var(--panel);', `
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #2d333f;padding-bottom:4px;">
          <span class="font-title" style="font-size:10px;color:var(--gold);">Multi-Unit 6-Block Timeline Inspector (timeline_view.rs)</span>
          <div style="display:flex;align-items:center;gap:6px;">
            <span class="font-mono" style="font-size:6.5px;color:var(--dim);">Round ${STATE.round} · Simultaneous Block Resolution</span>
            ${closeBtn640("STATE.modal=null;render();", "Close Timeline [T / Esc / Right-Click]")}
          </div>
        </div>

        <div style="flex:1;overflow-y:auto;margin:6px 0;display:flex;flex-direction:column;gap:4px;">
          ${b.units.filter(u => u.hp > 0).map(u => `
            <div style="display:grid;grid-template-columns:110px repeat(6, 1fr);gap:3px;align-items:center;">
              <div style="display:flex;align-items:center;gap:4px;">
                <div class="portrait-box ${u.isHero?'gold-rim':'foe-rim'}" style="width:20px;height:20px;">
                  <img src="${ASSETS.portraits[u.portrait]}" class="pixel">
                </div>
                <span style="font-size:6.8px;color:${u.isHero?'var(--gold)':'#fda4af'};font-weight:bold;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${u.name}</span>
              </div>
              ${[0,1,2,3,4,5].map(blk => {
                const st = u.plan.find(p => blk >= p.start && blk < p.start + p.blocks);
                return `
                  <div class="rust-card" style="height:20px;padding:2px 3px;display:flex;align-items:center;gap:2px;font-size:5.8px;border-color:${st ? (u.isHero ? '#d97706' : '#991b1b') : '#2d333f'};">
                    <span class="font-mono" style="color:var(--dim);">B${blk+1}</span>
                    ${st ? `<span style="color:var(--text);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">${(SKILLS[st.skillId]||{name:st.targetText}).name}</span>` : ''}
                  </div>
                `;
              }).join('')}
            </div>
          `).join('')}
        </div>

        <div style="display:flex;justify-content:space-between;align-items:center;">
          <span style="font-size:6.5px;color:var(--dim);">Compare hero wind-up/impact blocks against telegraphed enemy Foresight spans</span>
          <span class="font-mono" style="font-size:6.2px;color:var(--dim);">Right-Click, [T], or [Esc] to Close</span>
        </div>
      `)}
    </div>
  `;
}

// ============================================================================
// VICTORY / SETTLEMENT RESULTS MODAL (`battle/draw.rs` Results)
// ============================================================================
function renderBattleResultsModal() {
  const tpl = BATTLE.tpl;
  return `
    <div style="position:absolute;inset:0;background:rgba(0,0,0,0.75);z-index:45;display:flex;align-items:center;justify-content:center;">
      ${ornateCornerBox640('width:420px;padding:10px 12px;display:flex;flex-direction:column;gap:8px;background:var(--panel);', `
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #991b1b;padding-bottom:4px;">
          <span class="font-title" style="font-size:12px;color:var(--gold);">Victory · ${tpl.title}</span>
          <div style="display:flex;align-items:center;gap:6px;">
            <span class="font-mono" style="font-size:7px;color:#34d399;">Cleared in ${STATE.round} Rounds</span>
            ${closeBtn640("STATE.modal=null;STATE.screen='region';STATE.regionId='sluice_undercroft';render();", "Close [Esc / Right-Click]")}
          </div>
        </div>

        <div class="rust-inset" style="padding:6px 8px;font-size:7px;display:flex;flex-direction:column;gap:3px;">
          <div><span style="color:var(--dim);">Spoils Recovered:</span> <b style="color:var(--gold);">+35 Scrap · +1 Refined Tallow</b></div>
          <div><span style="color:var(--dim);">Squad Experience:</span> <b style="color:var(--info);">+45 XP to Active Vanguard</b></div>
          <div><span style="color:var(--dim);">Wound Check:</span> <b style="color:#34d399;">No Founders downed (0 Wounds inflicted)</b></div>
        </div>

        <div style="display:flex;justify-content:flex-end;gap:6px;">
          ${tpl.nextStory ? `
            <button class="rust-btn primary" style="height:20px;padding:0 10px;" onclick="advanceFromBattle('${tpl.nextStory}')">
              Continue Prologue Sequence (A2) ▸
            </button>
          ` : `
            <button class="rust-btn" onclick="STATE.modal=null;STATE.screen='worldmap';render();">Return to District Map</button>
            <button class="rust-btn primary" onclick="STATE.modal=null;STATE.screen='region';STATE.regionId='sluice_undercroft';render();">Return to Sluice Undercroft Hub ▸</button>
          `}
        </div>
      `)}
    </div>
  `;
}

function advanceFromBattle(nextStoryId) {
  STATE.scrap += 35;
  STATE.tallow += 1;
  STATE.modal = null;
  STATE.screen = 'story';
  STATE.storyId = nextStoryId;
  STATE.storyCardIdx = 0;
  render();
}

