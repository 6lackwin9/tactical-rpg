// ============================================================================
// NAVIGATION & ACTION HELPERS
// ============================================================================
function toggleAtlasDrawer() {
  const el = document.getElementById('atlas-drawer');
  const btn = document.getElementById('atlas-toggle-btn');
  if (!el) return;
  const open = el.style.display !== 'none';
  el.style.display = open ? 'none' : 'block';
  if (btn) btn.innerHTML = (open ? '▸' : '▾') + ' Screen & Art Atlas (ASSETORG3)';
}

function jumpScreen(target) {
  STATE.modal = null;
  STATE.rollBanner = null;
  if (target === 'settings') {
    STATE.settingsOpen = !STATE.settingsOpen;
    render();
    return;
  }
  STATE.settingsOpen = false;
  if (target === 'main_menu') {
    STATE.screen = 'main_menu';
  } else if (target === 'load_slots') {
    STATE.screen = 'load_slots';
    STATE.slotsSaving = false;
    STATE.slotAskDelete = null;
  } else if (target === 'runner') {
    STATE.screen = 'runner';
  } else if (target === 'story') {
    STATE.screen = 'story';
    STATE.storyId = 'prologue_intro';
    STATE.storyCardIdx = 0;
  } else if (target === 'region_hub') {
    STATE.screen = 'region';
    STATE.regionId = 'sluice_undercroft';
    STATE.talkNpcId = 'aline';
    STATE.talkNodeId = 'root';
  } else if (target === 'region_corvo') {
    STATE.screen = 'region';
    STATE.regionId = 'corvos_gate';
    STATE.talkNpcId = 'corvo';
    STATE.talkNodeId = 'root';
  } else if (target === 'board') {
    STATE.screen = 'region';
    STATE.regionId = 'sluice_undercroft';
    STATE.modal = 'board';
  } else if (target === 'shop') {
    STATE.screen = 'region';
    STATE.regionId = 'sluice_undercroft';
    STATE.modal = 'shop';
    STATE.shopKeeper = 'aline';
  } else if (target === 'party') {
    if (STATE.screen === 'main_menu' || STATE.screen === 'story' || STATE.screen === 'load_slots') STATE.screen = 'region';
    STATE.modal = 'party';
  } else if (target === 'worldmap') {
    STATE.screen = 'worldmap';
  } else if (target === 'battle') {
    STATE.screen = 'battle';
  }
  render();
}

function selectEncounter(encId) {
  initBattle(encId);
  STATE.screen = 'battle';
  STATE.modal = null;
  STATE.settingsOpen = false;
  render();
}

function launchEncounter(encId) {
  initBattle(encId);
  STATE.screen = 'battle';
  STATE.modal = null;
  STATE.settingsOpen = false;
  render();
}

function openShop(keeper) {
  STATE.shopKeeper = keeper;
  STATE.shopSelIdx = 0;
  STATE.modal = 'shop';
  render();
}

function runSkillRoll(statName, statVal, dc, okNode, failNode) {
  const mod = Math.floor((statVal - 10) / 2);
  // Deterministic T1.2 seeded roll that passes when stat is high
  const d20 = 11;
  const total = d20 + mod;
  const passed = total >= dc;
  STATE.rollBanner = {
    label: `${statName.toUpperCase()} CHECK`,
    d20,
    mod,
    total,
    dc,
    passed
  };
  STATE.talkNodeId = passed ? okNode : failNode;
  STATE.eventLog.unshift(`Roll [${statName}]: d20(${d20}) + ${mod} = ${total} vs DC ${dc} (${passed ? 'SUCCESS' : 'FAIL'})`);
  render();
}

function resolveCorvoToll() {
  if (STATE.scrap >= 30) {
    STATE.scrap -= 30;
    markQuestDone('q_corvo');
    STATE.eventLog.unshift('Paid 30 Scrap toll to Sergeant Corvo. Third Ward Gate opened.');
    STATE.screen = 'worldmap';
    STATE.worldNodeId = 'n_kiln';
    render();
  }
}

function markQuestDone(qId) {
  const q = STATE.quests.find(x => x.id === qId);
  if (q && q.status !== 'Closed') {
    q.status = 'Closed';
    STATE.scrap += q.rewardScrap;
    STATE.tallow += q.rewardTallow;
  }
}

function restAtHearth() {
  if (STATE.tallow > 0) {
    STATE.tallow -= 1;
    STATE.day += 1;
    STATE.tithingDays = Math.max(1, STATE.tithingDays - 1);
    Object.values(STATE.roster).forEach(h => {
      h.hp = h.maxHp;
      if (!h.active) h.wounded = false;
    });
    STATE.eventLog.unshift(`Rested at Tallow Hearth (Day ${STATE.day}, -1 Tallow). Reserves healed of Wounds.`);
    render();
  }
}

// Helper 1a (Full Frame for 1280x720 Large Screens): Renders full straddling 9-slice frame from `ui/frames/ornate_full.png`
// (`ornate_full.json`: 56x56 corners + tiled 16x56 / 56x16 iron-and-gold edge bars around all 4 sides)
function ornateBox(innerHtml, styleStr = '', extraClass = '') {
  return `
    <div class="ornate-straddle ${extraClass}" style="${styleStr}">
      <div class="ornate-full-edge-h top" style="background-image:url('${ASSETS.ui.ornate_full_top}');"></div>
      <div class="ornate-full-edge-h bottom" style="background-image:url('${ASSETS.ui.ornate_full_bot}');"></div>
      <div class="ornate-full-edge-v left" style="background-image:url('${ASSETS.ui.ornate_full_left}');"></div>
      <div class="ornate-full-edge-v right" style="background-image:url('${ASSETS.ui.ornate_full_right}');"></div>
      <img src="${ASSETS.ui.ornate_full_tl}" class="ornate-full-corner tl">
      <img src="${ASSETS.ui.ornate_full_tr}" class="ornate-full-corner tr">
      <img src="${ASSETS.ui.ornate_full_bl}" class="ornate-full-corner bl">
      <img src="${ASSETS.ui.ornate_full_br}" class="ornate-full-corner br">
      <div style="position:relative;z-index:5;width:100%;height:100%;display:flex;flex-direction:column;justify-content:space-between;">
        ${innerHtml}
      </div>
    </div>
  `;
}

// Helper 1b (Full Frame for 640x360 Scaled-by-2x Large Modals/Screens: BoardPane, ShopPane, PartyScreen, Timeline, SceneRunner)
function ornateBox640(arg1, arg2 = '', extraClass = '') {
  const isFirstStyle = typeof arg1 === 'string' && !arg1.trim().startsWith('<') && typeof arg2 === 'string' && arg2.trim().startsWith('<');
  const innerHtml = isFirstStyle ? arg2 : arg1;
  const styleStr = isFirstStyle ? arg1 : arg2;
  return `
    <div class="ornate-straddle-640 ${extraClass}" style="${styleStr}">
      <div class="ornate-full-edge-h-640 top" style="background-image:url('${ASSETS.ui.ornate_full_top}');"></div>
      <div class="ornate-full-edge-h-640 bottom" style="background-image:url('${ASSETS.ui.ornate_full_bot}');"></div>
      <div class="ornate-full-edge-v-640 left" style="background-image:url('${ASSETS.ui.ornate_full_left}');"></div>
      <div class="ornate-full-edge-v-640 right" style="background-image:url('${ASSETS.ui.ornate_full_right}');"></div>
      <img src="${ASSETS.ui.ornate_full_tl}" class="ornate-full-corner-640 tl">
      <img src="${ASSETS.ui.ornate_full_tr}" class="ornate-full-corner-640 tr">
      <img src="${ASSETS.ui.ornate_full_bl}" class="ornate-full-corner-640 bl">
      <img src="${ASSETS.ui.ornate_full_br}" class="ornate-full-corner-640 br">
      ${innerHtml}
    </div>
  `;
}

// Helper 2a (Reduced Frame with Only Corners for 1280x720 Small Popups): Renders `ui/frames/ornate.png`
// (`ornate.json`: 50x50 spear-tip corners + thin 1px crimson line between them — Hover Peek, New Chronicle Modal, Confirm Popups)
function ornateCornerBox(innerHtml, styleStr = '', extraClass = '') {
  return `
    <div class="ornate-corners-only ${extraClass}" style="${styleStr}">
      <div class="ornate-edge-h top" style="background-image:url('${ASSETS.ui.ornate_top}');"></div>
      <div class="ornate-edge-h bottom" style="background-image:url('${ASSETS.ui.ornate_bot}');"></div>
      <div class="ornate-edge-v left" style="background-image:url('${ASSETS.ui.ornate_left}');"></div>
      <div class="ornate-edge-v right" style="background-image:url('${ASSETS.ui.ornate_right}');"></div>
      <img src="${ASSETS.ui.ornate_tl}" class="ornate-corner-sm tl">
      <img src="${ASSETS.ui.ornate_tr}" class="ornate-corner-sm tr">
      <img src="${ASSETS.ui.ornate_bl}" class="ornate-corner-sm bl">
      <img src="${ASSETS.ui.ornate_br}" class="ornate-corner-sm br">
      <div style="position:relative;z-index:5;width:100%;height:100%;display:flex;flex-direction:column;justify-content:space-between;">
        ${innerHtml}
      </div>
    </div>
  `;
}

// Helper 2b (Reduced Frame with Only Corners for 640x360 Small Popups & Sub-Cards)
function ornateCornerBox640(arg1, arg2 = '', extraClass = '') {
  const isFirstStyle = typeof arg1 === 'string' && !arg1.trim().startsWith('<') && typeof arg2 === 'string' && arg2.trim().startsWith('<');
  const innerHtml = isFirstStyle ? arg2 : arg1;
  const styleStr = isFirstStyle ? arg1 : arg2;
  return `
    <div class="ornate-corners-only-640 ${extraClass}" style="${styleStr}">
      <img src="${ASSETS.ui.ornate_tl}" class="ornate-corner-sm-640 tl">
      <img src="${ASSETS.ui.ornate_tr}" class="ornate-corner-sm-640 tr">
      <img src="${ASSETS.ui.ornate_bl}" class="ornate-corner-sm-640 bl">
      <img src="${ASSETS.ui.ornate_br}" class="ornate-corner-sm-640 br">
      ${innerHtml}
    </div>
  `;
}

// Helper 3: Crisp [X] Close Icon Buttons (`Glyph::Close` in `aegis_ui/src/ui.rs:902`)
function closeBtn720(onclickStr = 'uiBack()', titleStr = 'Close [Esc / Right-Click]') {
  return `
    <button class="close-x-btn-720" onclick="${onclickStr}" title="${titleStr}" aria-label="Close">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" style="pointer-events:none;">
        <path d="M2.5 2.5L11.5 11.5M11.5 2.5L2.5 11.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
      </svg>
    </button>
  `;
}

function closeBtn640(onclickStr = 'uiBack()', titleStr = 'Close [Esc / Right-Click]') {
  return `
    <button class="close-x-btn-640" onclick="${onclickStr}" title="${titleStr}" aria-label="Close">
      <svg width="9" height="9" viewBox="0 0 10 10" fill="none" style="pointer-events:none;">
        <path d="M2 2L8 8M8 2L2 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
    </button>
  `;
}

// Unified Back / Close handler (`MouseButton::Right => s.back()` in `aegis_ui/src/shell.rs:1572` + `Key::Esc` + `[X]` button)
function uiBack() {
  if (STATE.displayConfirmOpen) {
    revertDisplaySettings();
  } else if (STATE.settingsAskUnapplied !== null) {
    STATE.settingsAskUnapplied = null;
    render();
  } else if (STATE.newGameModalOpen) {
    STATE.newGameModalOpen = false;
    render();
  } else if (STATE.slotAskDelete !== null) {
    STATE.slotAskDelete = null;
    render();
  } else if (STATE.slotAskOverwrite !== null) {
    STATE.slotAskOverwrite = null;
    render();
  } else if (STATE.storySkipConfirmOpen) {
    STATE.storySkipConfirmOpen = false;
    render();
  } else if (STATE.settingsOpen) {
    requestCloseSettings('close');
  } else if (STATE.skillMenu) {
    STATE.skillMenu = null;
    render();
  } else if (STATE.modal) {
    STATE.modal = null;
    render();
  } else if (STATE.screen === 'story') {
    STATE.storySkipConfirmOpen = true;
    render();
  } else if (STATE.screen === 'worldmap') {
    STATE.screen = 'region';
    STATE.regionId = 'sluice_undercroft';
    render();
  } else if (STATE.screen === 'load_slots' || STATE.screen === 'runner') {
    STATE.screen = 'main_menu';
    render();
  } else if (STATE.screen !== 'main_menu') {
    STATE.screen = 'main_menu';
    render();
  }
}


// ============================================================================
// TOP HUD BAR (`hud.rs` — Shared by RegionMap & WorldMap)
// ============================================================================
function renderHudBar(placeTitle) {
  const alertNames = ['0: Quiet', '1: Stirred', '2: Hunted', '3: Lockdown'];
  return `
    <div class="hud-bar">
      <div class="hud-place">
        <span>${placeTitle}</span>
        <span class="font-mono" style="font-size:7px;color:var(--dim);">· Day ${STATE.day}</span>
      </div>
      <div class="hud-right">
        <div class="hud-chip" title="Scrap (Currency)">
          <img src="${ASSETS.ui.gold}" class="pixel" style="width:10px;height:10px;">
          <span class="font-mono">${STATE.scrap} Scrap</span>
        </div>
        <div class="hud-chip" title="Tallow (Hearth & Rest Supplies)">
          <img src="${ASSETS.ui.supplies}" class="pixel" style="width:10px;height:10px;">
          <span class="font-mono">${STATE.tallow} Tallow</span>
        </div>
        <div class="hud-chip" title="Castellan Watch Alert Tier">
          <span style="color:#f59e0b;">●</span>
          <span class="font-mono">Alert ${alertNames[STATE.alertTier]}</span>
        </div>
        <div class="hud-chip deadline" title="Macro Campaign Clock: The Red Tithing">
          <span class="font-mono">Tithing: ${STATE.tithingDays}d</span>
        </div>
        <button class="hud-icon-btn ${STATE.modal==='party'?'active':''}" title="Party & Gear [P]" onclick="STATE.modal = STATE.modal==='party'?null:'party';render();">
          <img src="${ASSETS.ui.party}" class="pixel">
        </button>
        <button class="hud-icon-btn ${STATE.modal==='board'?'active':''}" title="Notice Board / Quests [J]" onclick="STATE.modal = STATE.modal==='board'?null:'board';render();">
          <img src="${ASSETS.ui.journal}" class="pixel">
        </button>
        <button class="hud-icon-btn ${STATE.modal==='shop'?'active':''}" title="Quartermaster / Stash [I]" onclick="openShop('bram')">
          <img src="${ASSETS.ui.stash}" class="pixel">
        </button>
        <button class="hud-icon-btn" title="District World Map [M]" onclick="STATE.screen='worldmap';STATE.modal=null;render();">
          <img src="${ASSETS.ui.pin_town}" class="pixel">
        </button>
        <button class="hud-icon-btn" title="Main Menu [Esc / Right-Click]" onclick="STATE.screen='main_menu';STATE.modal=null;render();">
          <img src="${ASSETS.ui.menu}" class="pixel">
        </button>
      </div>
    </div>
  `;
}

