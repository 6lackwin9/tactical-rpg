// ============================================================================
// MASTER RENDER & KEYBOARD SHORTCUTS (`P`, `J`, `I`, `T`, `Esc`, `Space`)
// ============================================================================
function render() {
  const root = document.getElementById('stage-root');
  if (!root) return;

  let html = '';
  if (STATE.screen === 'main_menu') {
    html = renderMainMenu();
  } else if (STATE.screen === 'load_slots') {
    html = renderLoadSlots();
  } else if (STATE.screen === 'runner') {
    html = `<div class="legacy-640-stage">${renderSceneRunner()}</div>`;
  } else if (STATE.screen === 'story') {
    html = `<div class="legacy-640-stage">${renderStoryStage()}</div>`;
  } else if (STATE.screen === 'region') {
    html = `<div class="legacy-640-stage">${renderRegionScreen()}</div>`;
  } else if (STATE.screen === 'worldmap') {
    html = `<div class="legacy-640-stage">${renderWorldMap()}</div>`;
  } else if (STATE.screen === 'battle') {
    html = `<div class="legacy-640-stage">${renderBattleScreen()}</div>`;
  }

  if (STATE.settingsOpen) {
    html += renderSettingsModal();
  }
  root.innerHTML = html;

  if (STATE.screen === 'main_menu') {
    startTitleFxCanvas();
  }

  // Update active state on top Design Session Inspector buttons
  document.querySelectorAll('#design-bar .db-btn').forEach(btn => {
    const j = btn.getAttribute('data-jump');
    let active = false;
    if (j === 'settings' && STATE.settingsOpen) active = true;
    else if (!STATE.settingsOpen) {
      if (j === 'main_menu' && STATE.screen === 'main_menu') active = true;
      else if (j === 'load_slots' && STATE.screen === 'load_slots') active = true;
      else if (j === 'runner' && STATE.screen === 'runner') active = true;
      else if (j === 'story' && STATE.screen === 'story') active = true;
      else if (j === 'region_hub' && STATE.screen === 'region' && STATE.regionId === 'sluice_undercroft' && !STATE.modal) active = true;
      else if (j === 'region_corvo' && STATE.screen === 'region' && STATE.regionId === 'corvos_gate' && !STATE.modal) active = true;
      else if (j === 'board' && STATE.modal === 'board') active = true;
      else if (j === 'shop' && STATE.modal === 'shop') active = true;
      else if (j === 'party' && STATE.modal === 'party') active = true;
      else if (j === 'worldmap' && STATE.screen === 'worldmap' && !STATE.modal) active = true;
      else if (j === 'battle' && STATE.screen === 'battle') active = true;
    }
    btn.classList.toggle('active', active);
  });

  // Populate screen-specific comparison toggles in the top bar (keeping the 1280x720 game pane 100% pure player UI)
  const togglesEl = document.getElementById('screen-toggles');
  if (togglesEl) {
    if (STATE.settingsOpen) {
      const isMainMenuCtx = STATE.settingsContextMode === 'force_full' || (STATE.settingsContextMode === 'auto' && STATE.screen === 'main_menu');
      togglesEl.innerHTML = `
        <span style="color:#fbbf24;font-weight:700;font-size:10px;">Settings Toggles:</span>
        <button class="db-btn ${isMainMenuCtx ? 'active' : ''}" onclick="STATE.settingsContextMode='force_full';render();" title="Preview Settings as a Full-Screen Main Menu Screen">View: Main Menu Fullscreen</button>
        <button class="db-btn ${!isMainMenuCtx ? 'active' : ''}" onclick="STATE.settingsContextMode='force_popup';render();" title="Preview Settings as an In-Game Popup Modal">View: In-Game Popup</button>
        <button class="db-btn ${STATE.settingsLayout === 'tab_rail' ? 'active' : ''}" onclick="STATE.settingsLayout='tab_rail';render();">Tabs: ★ Vertical Rail</button>
        <button class="db-btn ${STATE.settingsLayout === 'top_tabs' ? 'active' : ''}" onclick="STATE.settingsLayout='top_tabs';render();">Tabs: Top Horizontal</button>
      `;
    } else if (STATE.screen === 'load_slots') {
      togglesEl.innerHTML = `
        <span style="color:#fbbf24;font-weight:700;font-size:10px;">Slots Toggles:</span>
        <button class="db-btn ${!STATE.slotsSaving ? 'active' : ''}" onclick="STATE.slotsSaving=false;STATE.slotToast=null;render();">Mode: Load</button>
        <button class="db-btn ${STATE.slotsSaving ? 'active' : ''}" onclick="STATE.slotsSaving=true;STATE.slotToast=null;render();">Mode: Save</button>
        <button class="db-btn ${STATE.slotsMode === 'slot_card' ? 'active' : ''}" onclick="STATE.slotsMode='slot_card';render();">Style: ★ Slot Cards</button>
        <button class="db-btn ${STATE.slotsMode === 'flat_rows' ? 'active' : ''}" onclick="STATE.slotsMode='flat_rows';render();">Style: Flat Rows</button>
      `;
    } else if (STATE.screen === 'main_menu') {
      togglesEl.innerHTML = `
        <span style="color:#fbbf24;font-weight:700;font-size:10px;">MainMenu Toggles:</span>
        <button class="db-btn ${!STATE.newGameModalOpen ? 'active' : ''}" onclick="STATE.newGameModalOpen=false;render();" title="Show the Main Menu Title Screen">View: Title Screen</button>
        <button class="db-btn ${STATE.newGameModalOpen ? 'active' : ''}" onclick="STATE.newGameModalOpen=true;render();" title="Show the New Game Campaign Setup Modal (Difficulty & Ironman)">View: New Game Setup</button>
      `;
    } else {
      togglesEl.innerHTML = '';
    }
  }

  // Update architecture context footer note
  const noteEl = document.getElementById('arch-note');
  if (noteEl) {
    const notes = {
      main_menu: '<b>MainMenu (1280×720 Native Grid):</b> Cathedral title art with lower-ward mist & toned-down lanterns · <b>Reduced Corner Frame</b> on Hover Peek & New Chronicle Setup · Top-right <code>✕</code> & Right-Click to close.',
      load_slots: '<b>Load / Save Chronicles (1280×720 Native Grid):</b> <b>Full Frame</b> (<code>ornate_full.png</code> tiled iron-and-gold bars) on main screen + <b>Reduced Corner Frame</b> on Delete/Overwrite popups · Top-right <code>✕</code> & Right-Click to close.',
      runner: '<b>SceneRunner (Dev F10):</b> <b>Full Frame</b> (<code>ornate_full.png</code>) + top-right <code>✕</code> & Right-Click to close.',
      story: '<b>StoryStage (Decision A2):</b> Full-screen <code>backgrounds/story/*.png</code> vignettes + <b>Reduced Corner Frame</b> caption card + top-right <code>✕</code> & Right-Click to skip.',
      region: '<b>RegionMap + TalkBox (Decision A3/A4):</b> Top <code>hud_bar</code>, walkable tile map, and <b>Reduced Corner Frame</b> <code>TalkBox</code> (Right-Click closes active dialogue or returns to Main Menu).',
      worldmap: '<b>WorldMap (Decision A4):</b> Sunk Ward pins & route lines + <b>Reduced Corner Frame</b> District Card with top-right <code>✕</code> & Right-Click back to Sluice Hub.',
      battle: '<b>BattleScreen:</b> <b>Full Frame</b> on Multi-Unit Timeline overlay + <b>Reduced Corner Frame</b> on <code>SkillMenu</code>, Intel card & Victory modal · Top-right <code>✕</code> & Right-Click to close.'
    };
    if (STATE.settingsOpen) {
      noteEl.innerHTML = '<b>Settings (1280×720 Native Grid):</b> <b>Full Frame</b> (<code>ornate_full.png</code> tiled iron-and-gold bars) on Settings + <b>Reduced Corner Frame</b> on 15s Revert popup · Top-right <code>✕</code> & Right-Click to close.';
    } else {
      noteEl.innerHTML = notes[STATE.screen] || notes.main_menu;
    }
  }
}

window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    uiBack();
  } else if (e.key === 'Enter') {
    if (STATE.displayConfirmOpen) { confirmDisplaySettings(); }
    else if (STATE.settingsAskUnapplied !== null) { applyUnappliedAndKeepInSettings(); }
    else if (STATE.newGameModalOpen) { startNewCampaign(); }
    else if (STATE.slotAskDelete !== null) { confirmDeleteSlot(); }
    else if (STATE.slotAskOverwrite !== null) { writeSaveToSlot(STATE.slotAskOverwrite); }
    else if (STATE.screen === 'load_slots' && !STATE.settingsOpen) { executeSlot(STATE.slotSelected); }
  } else if (e.key === 'F10') {
    e.preventDefault();
    STATE.settingsOpen = false;
    STATE.screen = STATE.screen === 'runner' ? 'main_menu' : 'runner';
    render();
  } else if ((e.key === 'l' || e.key === 'L') && STATE.screen === 'load_slots' && STATE.slotAskDelete === null && STATE.slotAskOverwrite === null) {
    toggleSlotLock(STATE.slotSelected);
  } else if (e.key === 'Delete' && STATE.screen === 'load_slots' && STATE.slotAskDelete === null && STATE.slotAskOverwrite === null) {
    promptDeleteSlot(STATE.slotSelected);
  } else if (e.key === 'p' || e.key === 'P') {
    if (STATE.screen !== 'main_menu' && STATE.screen !== 'load_slots') { STATE.modal = STATE.modal === 'party' ? null : 'party'; render(); }
  } else if (e.key === 'j' || e.key === 'J') {
    if (STATE.screen !== 'main_menu' && STATE.screen !== 'load_slots') { STATE.modal = STATE.modal === 'board' ? null : 'board'; render(); }
  } else if (e.key === 't' || e.key === 'T') {
    if (STATE.screen === 'battle') { STATE.modal = STATE.modal === 'timeline' ? null : 'timeline'; render(); }
  }
});

// Right-Click (`contextmenu`) anywhere in the 1280x720 game viewport triggers `uiBack()` (`MouseButton::Right => s.back()` in `shell.rs`)
document.getElementById('viewport-wrap')?.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  uiBack();
});

function fitViewportScale() {
  const wrap = document.getElementById('viewport-wrap');
  const root = document.getElementById('stage-root');
  if (!wrap || !root) return;
  if (STATE.settings && STATE.settings.browserAutoFit === false) {
    wrap.style.width = '1280px';
    wrap.style.height = '720px';
    root.style.transform = 'scale(1)';
    return;
  }
  const barH = (document.getElementById('design-bar')?.offsetHeight || 34) + (document.getElementById('arch-footer')?.offsetHeight || 26) + 16;
  const availW = Math.max(320, (window.innerWidth || 1280) - 20);
  const availH = Math.max(200, (window.innerHeight || 800) - barH);
  const scale = Math.min(1.5, availW / 1280, availH / 720);
  wrap.style.width = Math.round(1280 * scale) + 'px';
  wrap.style.height = Math.round(720 * scale) + 'px';
  root.style.transform = `scale(${scale})`;
}

window.addEventListener('resize', fitViewportScale);
fitViewportScale();
render();
