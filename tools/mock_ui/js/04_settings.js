// ============================================================================
// SCREEN 1c: SETTINGS MENU (Staged Display Mode/Resolution + Apply & Revert Popup + Decluttered In-Game Popup)
// ============================================================================
let DISPLAY_CONFIRM_TIMER = null;

function stepSetting(listKey, idxKey, delta) {
  const s = STATE.settings;
  const arr = s[listKey];
  if (!arr || !arr.length) return;
  s[idxKey] = (s[idxKey] + delta + arr.length) % arr.length;
  render();
}

// 3c: Arrow toggles on Resolution & Display Mode stage pending values WITHOUT rescaling on each arrow click
function stepPendingDisplay(listKey, pendingIdxKey, delta) {
  const s = STATE.settings;
  const arr = s[listKey];
  if (!arr || !arr.length) return;
  s[pendingIdxKey] = (s[pendingIdxKey] + delta + arr.length) % arr.length;
  render();
}

function discardPendingDisplay() {
  const s = STATE.settings;
  s.pendingResIdx = s.resIdx;
  s.pendingWindowModeIdx = s.windowModeIdx;
  render();
}

function applyDisplaySettings() {
  const s = STATE.settings;
  s.prevResIdx = s.resIdx;
  s.prevWindowModeIdx = s.windowModeIdx;
  s.resIdx = s.pendingResIdx;
  s.windowModeIdx = s.pendingWindowModeIdx;
  STATE.displayConfirmOpen = true;
  STATE.displayConfirmSec = 15;
  fitViewportScale();
  render();

  if (DISPLAY_CONFIRM_TIMER) clearTimeout(DISPLAY_CONFIRM_TIMER);
  function tickConfirm() {
    if (!STATE.displayConfirmOpen) return;
    STATE.displayConfirmSec -= 1;
    if (STATE.displayConfirmSec <= 0) {
      revertDisplaySettings();
    } else {
      render();
      DISPLAY_CONFIRM_TIMER = setTimeout(tickConfirm, 1000);
    }
  }
  DISPLAY_CONFIRM_TIMER = setTimeout(tickConfirm, 1000);
}

function confirmDisplaySettings() {
  if (DISPLAY_CONFIRM_TIMER) clearTimeout(DISPLAY_CONFIRM_TIMER);
  DISPLAY_CONFIRM_TIMER = null;
  STATE.displayConfirmOpen = false;
  render();
}

function revertDisplaySettings() {
  if (DISPLAY_CONFIRM_TIMER) clearTimeout(DISPLAY_CONFIRM_TIMER);
  DISPLAY_CONFIRM_TIMER = null;
  const s = STATE.settings;
  s.resIdx = s.prevResIdx;
  s.windowModeIdx = s.prevWindowModeIdx;
  s.pendingResIdx = s.resIdx;
  s.pendingWindowModeIdx = s.windowModeIdx;
  STATE.displayConfirmOpen = false;
  fitViewportScale();
  render();
}

function hasUnappliedDisplayChanges() {
  const s = STATE.settings;
  return (s.pendingResIdx !== s.resIdx) || (s.pendingWindowModeIdx !== s.windowModeIdx);
}

// Guard closing Settings (via Right-Click, Esc, [X] button, or Return to Title) when Display Mode / Resolution changes are staged but unapplied.
// All other settings (Audio, Language, Text Size, FPS, VSync, Gameplay, Battle) apply instantly on toggle.
function requestCloseSettings(afterCloseAction = 'close') {
  if (hasUnappliedDisplayChanges()) {
    STATE.settingsAskUnapplied = afterCloseAction;
    render();
    return;
  }
  finishCloseSettings(afterCloseAction);
}

function finishCloseSettings(afterCloseAction = 'close') {
  STATE.settingsAskUnapplied = null;
  STATE.settingsOpen = false;
  STATE.settingsContextMode = 'auto';
  if (afterCloseAction === 'return_title') {
    STATE.screen = 'main_menu';
  }
  render();
}

function discardAndCloseSettings() {
  const action = STATE.settingsAskUnapplied || 'close';
  const s = STATE.settings;
  s.pendingResIdx = s.resIdx;
  s.pendingWindowModeIdx = s.windowModeIdx;
  finishCloseSettings(action);
}

function applyUnappliedAndKeepInSettings() {
  STATE.settingsAskUnapplied = null;
  applyDisplaySettings();
}

function restoreDefaultSettings() {
  const s = STATE.settings;
  s.resIdx = 0;
  s.pendingResIdx = 0;
  s.prevResIdx = 0;
  s.windowModeIdx = 1;
  s.pendingWindowModeIdx = 1;
  s.prevWindowModeIdx = 1;
  s.browserAutoFit = true;
  s.languageIdx = 0;
  s.textSizeIdx = 1;
  s.vsync = true;
  s.fpsIdx = 2;
  s.reduceMotion = false;
  s.muteAll = false;
  s.masterVolume = 85;
  s.musicVolume = 75;
  s.sfxVolume = 90;
  s.interfaceVolume = 80;
  s.ambienceVolume = 80;
  s.muteInBackground = true;
  s.textSpeedIdx = 1;
  s.autoAdvance = false;
  s.tutorials = true;
  s.mapNames = 'On hover';
  s.mapZoomIdx = 1;
  s.animSpeedIdx = 2;
  s.skipAnim = false;
  s.playbackSpeedIdx = 0;
  s.showEnemyTells = true;
  s.warnMissingActions = true;
  s.autoFillRest = true;
  s.autoConfirmRound = true;
  s.seeThrough = 'Selected unit';
  fitViewportScale();
  render();
}

function renderStepper(label, valueText, onPrev, onNext, disabled = false) {
  return `
    <div style="display:flex;align-items:center;gap:6px;flex-shrink:0;opacity:${disabled ? '0.45' : '1'};pointer-events:${disabled ? 'none' : 'auto'};">
      <button class="rust-btn-720" style="width:28px;height:28px;padding:0;font-size:12px;" onclick="${onPrev}">◀</button>
      <div class="font-mono" style="width:176px;height:28px;background:#07080c;border:1px solid #4b5563;display:flex;align-items:center;justify-content:center;padding:0 8px;font-size:12.5px;color:var(--gold);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
        ${valueText}
      </div>
      <button class="rust-btn-720" style="width:28px;height:28px;padding:0;font-size:12px;" onclick="${onNext}">▶</button>
    </div>
  `;
}

function renderSettingsModal() {
  // 3a: Display at top of vertical tab rail
  const tabs = [
    { id: 'Display',  sub: 'Resolution, Mode & Language' },
    { id: 'Sound',    sub: 'Volume & Audio Mix' },
    { id: 'Gameplay', sub: 'Dialogue & Map Pacing' },
    { id: 'Battle',   sub: 'Timeline & Speed' },
    { id: 'Controls', sub: 'Keybindings' }
  ];
  const s = STATE.settings;
  const isRail = STATE.settingsLayout === 'tab_rail';

  // Fullscreen Settings when opened on Main Menu vs Compact Decluttered Popup Modal when opened In-Game
  const isMainMenuScreen = STATE.settingsContextMode === 'force_full'
    ? true
    : (STATE.settingsContextMode === 'force_popup' ? false : (STATE.screen === 'main_menu'));
  const isPopup = !isMainMenuScreen;
  const rowPad = isPopup ? '6px 12px' : '9px 16px';
  const rowGap = isPopup ? '7px' : '10px';
  const rowHeight = isPopup ? '44px' : '54px';

  // Helper to render a setting row: locked height so changing values never shifts sibling elements
  function settingRow(title, subtitle, controlHtml) {
    return `
      <div class="rust-card" style="height:${rowHeight};padding:${rowPad};display:flex;align-items:center;justify-content:space-between;gap:14px;flex-shrink:0;">
        <div style="flex:1;min-width:0;padding-right:8px;">
          <div style="font-size:14px;color:var(--text);font-weight:bold;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${title}</div>
          ${(!isPopup && subtitle) ? `<div style="font-size:11.5px;color:var(--dim);margin-top:2px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${subtitle}</div>` : ''}
        </div>
        <div style="flex-shrink:0;">
          ${controlHtml}
        </div>
      </div>
    `;
  }

  const appliedRes = s.resolutions[s.resIdx];
  const appliedMode = s.windowModes[s.windowModeIdx];
  const pendingRes = s.resolutions[s.pendingResIdx];
  const pendingMode = s.windowModes[s.pendingWindowModeIdx];
  const displayDirty = (s.pendingResIdx !== s.resIdx) || (s.pendingWindowModeIdx !== s.windowModeIdx);
  const currentFps = s.fpsCaps[s.fpsIdx];
  const currentTextSize = s.textSizes[s.textSizeIdx];
  const currentLang = s.languages[s.languageIdx];

  // Concise 2-line descriptions for all 4 Display Modes (locked inside a fixed 32px-tall box so Popup/Fullscreen panes never resize)
  const modeDescriptions = {
    'Fullscreen': 'Dedicated GPU fullscreen — locks monitor output to chosen Resolution for lowest latency.',
    'Borderless Fullscreen': 'Fills entire monitor without borders for instant Alt+Tab; scales Resolution to fit.',
    'Borderless Window': 'Fixed-size window at chosen Resolution with no OS title bar or borders, centered on desktop.',
    'Windowed': 'Standard movable desktop window with OS title bar and borders at chosen Resolution.'
  };
  const pendingModeDesc = modeDescriptions[pendingMode] || '';

  let tabBody = '';
  if (STATE.settingsTab === 'Display') {
    const videoCardHeight = isPopup ? '116px' : '128px';
    tabBody = `
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:${rowGap};height:24px;flex-shrink:0;">
        <span class="font-title" style="font-size:17px;color:var(--gold);">Display & Language</span>
        <span class="font-mono" style="font-size:11.5px;color:#93c5fd;background:rgba(30,58,138,0.35);border:1px solid #3b82f6;padding:2px 8px;">
          Active: ${appliedMode} · ${appliedRes}
        </span>
      </div>
      <div style="display:flex;flex-direction:column;gap:${rowGap};">
        <!-- Top Grouped Video Block: Fixed height (${videoCardHeight}) so switching Display Modes NEVER changes pane or card height -->
        <div class="rust-card" style="height:${videoCardHeight};padding:${rowPad};border-color:${displayDirty ? 'var(--gold)' : '#374151'};display:flex;align-items:stretch;justify-content:space-between;gap:14px;flex-shrink:0;">
          <div style="flex:1;min-width:0;display:flex;flex-direction:column;justify-content:space-between;">
            <!-- 1. Resolution Selection at Top -->
            <div style="height:${isPopup ? '36px' : '42px'};display:flex;align-items:center;justify-content:space-between;gap:12px;">
              <div style="flex:1;min-width:0;padding-right:8px;">
                <div style="font-size:14px;color:var(--text);font-weight:bold;">Resolution</div>
                ${!isPopup ? `<div style="font-size:11.5px;color:var(--dim);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">16:9 widescreen render size (staged until Apply is clicked)</div>` : ''}
              </div>
              ${renderStepper('Resolution', pendingRes, "stepPendingDisplay('resolutions','pendingResIdx',-1)", "stepPendingDisplay('resolutions','pendingResIdx',1)")}
            </div>

            <!-- 2. Merged Display Mode (Fullscreen / Borderless Fullscreen / Borderless Window / Windowed) with Fixed-Height 32px Description Box -->
            <div style="height:${isPopup ? '56px' : '60px'};display:flex;align-items:center;justify-content:space-between;gap:12px;border-top:1px solid #2d333f;padding-top:6px;">
              <div style="flex:1;min-width:0;padding-right:8px;">
                <div style="font-size:14px;color:var(--text);font-weight:bold;line-height:1.15;">Display Mode</div>
                <div style="font-size:11.5px;line-height:15px;height:30px;min-height:30px;max-height:30px;overflow:hidden;margin-top:2px;color:${isPopup ? 'var(--dim)' : 'var(--gold)'};">
                  ${pendingModeDesc}
                </div>
              </div>
              ${renderStepper('Display Mode', pendingMode, "stepPendingDisplay('windowModes','pendingWindowModeIdx',-1)", "stepPendingDisplay('windowModes','pendingWindowModeIdx',1)")}
            </div>
          </div>

          <!-- Apply Button Next to Resolution & Display Mode -->
          <div style="display:flex;flex-direction:column;justify-content:center;align-items:stretch;gap:6px;padding-left:12px;border-left:1px solid #374151;width:118px;flex-shrink:0;">
            <button class="rust-btn-720 ${displayDirty ? 'primary' : ''}" style="height:36px;font-weight:bold;" onclick="applyDisplaySettings()">
              Apply
            </button>
            <div style="height:24px;display:flex;align-items:center;justify-content:center;">
              ${displayDirty ? `
                <button class="rust-btn-720" style="width:100%;height:24px;font-size:11px;" onclick="discardPendingDisplay()">
                  Reset
                </button>
              ` : `
                <span class="font-mono" style="font-size:10.5px;color:var(--dim);text-align:center;">In Sync</span>
              `}
            </div>
          </div>
        </div>

        <!-- 3. Language Selection in Display Tab -->
        ${settingRow(
          'Language',
          'Interface, story captions, and dialogue locale',
          renderStepper('Language', currentLang, "stepSetting('languages','languageIdx',-1)", "stepSetting('languages','languageIdx',1)")
        )}

        <!-- 4. Text Size Selection -->
        ${settingRow(
          'Text Size',
          'Scale dialogue, tooltips, and tactical readouts (85%–130%)',
          renderStepper('Text Size', currentTextSize.label, "stepSetting('textSizes','textSizeIdx',-1)", "stepSetting('textSizes','textSizeIdx',1)")
        )}

        <!-- 5. Frame Rate Cap, VSync, and Reduce Motion -->
        ${settingRow(
          'Frame Rate Cap',
          'Maximum rendering frame rate',
          renderStepper('FPS Cap', currentFps, "stepSetting('fpsCaps','fpsIdx',-1)", "stepSetting('fpsCaps','fpsIdx',1)")
        )}

        <div style="display:grid;grid-template-columns:1fr 1fr;gap:${rowGap};">
          ${settingRow(
            'Vertical Sync (VSync)',
            'Prevent frame tearing',
            `<button class="rust-btn-720 ${s.vsync ? 'primary' : ''}" style="width:88px;" onclick="STATE.settings.vsync=!STATE.settings.vsync;render();">${s.vsync ? 'On' : 'Off'}</button>`
          )}
          ${settingRow(
            'Reduce Motion',
            'Dampen camera shake & particle drift',
            `<button class="rust-btn-720 ${s.reduceMotion ? 'primary' : ''}" style="width:88px;" onclick="STATE.settings.reduceMotion=!STATE.settings.reduceMotion;render();">${s.reduceMotion ? 'On' : 'Off'}</button>`
          )}
        </div>
      </div>
    `;
  } else if (STATE.settingsTab === 'Sound') {
    const sliders = [
      { key: 'masterVolume',    label: 'Master Volume',    desc: 'Overall game audio output' },
      { key: 'musicVolume',     label: 'Music Volume',     desc: 'Orchestral & hurdy-gurdy ward score' },
      { key: 'sfxVolume',       label: 'Effects Volume',   desc: 'Combat steel clashes, billhooks & chains' },
      { key: 'interfaceVolume', label: 'Interface Volume', desc: 'Menu chimes, card clicks & roll ticks' },
      { key: 'ambienceVolume',  label: 'Ward Ambience',    desc: 'Canal rain, distant bells & furnace rumble' }
    ];
    tabBody = `
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:${rowGap};height:24px;flex-shrink:0;">
        <span class="font-title" style="font-size:17px;color:var(--gold);">Audio & Soundscape</span>
        <div style="display:flex;gap:8px;">
          <button class="rust-btn-720 ${s.muteAll ? 'primary' : ''}" style="width:118px;" onclick="STATE.settings.muteAll=!STATE.settings.muteAll;render();">
            Mute All: ${s.muteAll ? 'On' : 'Off'}
          </button>
          <button class="rust-btn-720 ${s.muteInBackground ? 'primary' : ''}" style="width:190px;" onclick="STATE.settings.muteInBackground=!STATE.settings.muteInBackground;render();">
            Mute in Background: ${s.muteInBackground ? 'On' : 'Off'}
          </button>
        </div>
      </div>
      <div style="display:flex;flex-direction:column;gap:${rowGap};opacity:${s.muteAll ? '0.45' : '1'};">
        ${sliders.map(sl => settingRow(
          sl.label,
          sl.desc,
          `<div style="display:flex;align-items:center;gap:8px;">
            <button class="rust-btn-720" style="width:28px;height:28px;padding:0;" onclick="STATE.settings.${sl.key}=Math.max(0,STATE.settings.${sl.key}-5);render();">◀</button>
            <div style="width:160px;height:12px;background:#07080c;border:1px solid #374151;position:relative;cursor:pointer;"
                 onclick="STATE.settings.${sl.key}=(STATE.settings.${sl.key}>=100?0:STATE.settings.${sl.key}+20);render();">
              <div style="width:${s[sl.key]}%;height:100%;background:linear-gradient(90deg, #991b1b, #d97706);"></div>
            </div>
            <button class="rust-btn-720" style="width:28px;height:28px;padding:0;" onclick="STATE.settings.${sl.key}=Math.min(100,STATE.settings.${sl.key}+5);render();">▶</button>
            <span class="font-mono" style="width:42px;text-align:right;font-size:12.5px;color:var(--gold);">${s[sl.key]}%</span>
          </div>`
        )).join('')}
      </div>
    `;
  } else if (STATE.settingsTab === 'Gameplay') {
    const curSpeed = s.textSpeeds[s.textSpeedIdx];
    const curZoom = s.mapZooms[s.mapZoomIdx];

    tabBody = `
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:${rowGap};height:24px;flex-shrink:0;">
        <span class="font-title" style="font-size:17px;color:var(--gold);">Gameplay & Exploration</span>
      </div>
      <div style="display:flex;flex-direction:column;gap:${rowGap};">
        ${settingRow(
          'Text Speed',
          'How fast story captions and dialogue lines reveal',
          renderStepper('Text Speed', curSpeed, "stepSetting('textSpeeds','textSpeedIdx',-1)", "stepSetting('textSpeeds','textSpeedIdx',1)")
        )}
        ${settingRow(
          'Auto-Advance Story Dialogue',
          'Automatically advance story cards after voice/text finishes',
          `<button class="rust-btn-720 ${s.autoAdvance ? 'primary' : ''}" style="width:104px;" onclick="STATE.settings.autoAdvance=!STATE.settings.autoAdvance;render();">${s.autoAdvance ? 'On' : 'Off'}</button>`
        )}
        ${settingRow(
          'Map Labels & Names',
          'Show NPC and exit labels on street and hub maps',
          `<button class="rust-btn-720 primary" style="width:116px;" onclick="STATE.settings.mapNames=(STATE.settings.mapNames==='On hover'?'Always':'On hover');render();">${s.mapNames}</button>`
        )}
        ${settingRow(
          'Map Camera Zoom',
          'Default camera zoom on street and hub maps',
          renderStepper('Map Zoom', curZoom, "stepSetting('mapZooms','mapZoomIdx',-1)", "stepSetting('mapZooms','mapZoomIdx',1)")
        )}
        ${settingRow(
          'Tutorial & Tactical Hints',
          'Show contextual prompts when encountering new mechanics',
          `<button class="rust-btn-720 ${s.tutorials ? 'primary' : ''}" style="width:104px;" onclick="STATE.settings.tutorials=!STATE.settings.tutorials;render();">${s.tutorials ? 'On' : 'Off'}</button>`
        )}
      </div>
    `;
  } else if (STATE.settingsTab === 'Battle') {
    const curAnim = s.animSpeeds[s.animSpeedIdx];
    const curPlay = s.playbackSpeeds[s.playbackSpeedIdx];
    tabBody = `
      <div class="font-title" style="font-size:17px;color:var(--gold);margin-bottom:${rowGap};height:24px;flex-shrink:0;">Battle & Timeline Planner</div>
      <div style="display:flex;flex-direction:column;gap:${rowGap};">
        ${settingRow(
          'Playback Speed',
          'How timeline rounds play out (1×, 0.5×, or Block-by-Block)',
          renderStepper('Playback', curPlay, "stepSetting('playbackSpeeds','playbackSpeedIdx',-1)", "stepSetting('playbackSpeeds','playbackSpeedIdx',1)")
        )}
        ${settingRow(
          'Animation Speed',
          'Unit movement stride and attack cadence',
          `<div style="display:flex;align-items:center;gap:8px;">
            ${renderStepper('Anim Speed', curAnim, "stepSetting('animSpeeds','animSpeedIdx',-1)", "stepSetting('animSpeeds','animSpeedIdx',1)", s.skipAnim)}
            <button class="rust-btn-720 ${s.skipAnim ? 'primary' : ''}" style="width:92px;" onclick="STATE.settings.skipAnim=!STATE.settings.skipAnim;render();">Skip: ${s.skipAnim ? 'On' : 'Off'}</button>
          </div>`
        )}
        <div style="display:grid;grid-template-columns:1fr 1fr;gap:${rowGap};">
          ${settingRow(
            'Auto-Fill Remaining Blocks',
            'Fill free round blocks with follow-ups',
            `<button class="rust-btn-720 ${s.autoFillRest ? 'primary' : ''}" style="width:84px;" onclick="STATE.settings.autoFillRest=!STATE.settings.autoFillRest;render();">${s.autoFillRest ? 'On' : 'Off'}</button>`
          )}
          ${settingRow(
            'Auto-Confirm Next Round',
            'Advance to next round planning automatically',
            `<button class="rust-btn-720 ${s.autoConfirmRound ? 'primary' : ''}" style="width:84px;" onclick="STATE.settings.autoConfirmRound=!STATE.settings.autoConfirmRound;render();">${s.autoConfirmRound ? 'On' : 'Off'}</button>`
          )}
          ${settingRow(
            'Show Enemy Tells',
            'Display enemy intent arcs and targets',
            `<button class="rust-btn-720 ${s.showEnemyTells ? 'primary' : ''}" style="width:84px;" onclick="STATE.settings.showEnemyTells=!STATE.settings.showEnemyTells;render();">${s.showEnemyTells ? 'On' : 'Off'}</button>`
          )}
          ${settingRow(
            'Warn Missing Actions',
            'Prompt before confirming with unspent AP',
            `<button class="rust-btn-720 ${s.warnMissingActions ? 'primary' : ''}" style="width:84px;" onclick="STATE.settings.warnMissingActions=!STATE.settings.warnMissingActions;render();">${s.warnMissingActions ? 'On' : 'Off'}</button>`
          )}
        </div>
      </div>
    `;
  } else if (STATE.settingsTab === 'Controls') {
    const groups = [
      {
        head: 'Menus & Navigation',
        rows: [
          ['Confirm / Activate', 'Enter, Space, L-Click'],
          ['Back / Close Panel', 'Escape, R-Click'],
          ['Cycle Focus / Step Value', 'Tab, Left / Right']
        ]
      },
      {
        head: 'Region Exploration & Hub',
        rows: [
          ['Move Party / Run', 'WASD / Arrows (Shift)'],
          ['Interact with Marker', 'E / Click Marker'],
          ['Party, Board, Stash, Log', 'P, J, I, L']
        ]
      },
      {
        head: 'Timeline Tactical Battle',
        rows: [
          ['Confirm Round / Next Block', 'Enter / Space, N'],
          ['Select Hero 1–4 / Skills', '1–4 / Shift+1–9'],
          ['Undo Step / Inspect Unit', 'Z, R-Click'],
          ['Timeline Inspector / Speed', 'T / P']
        ]
      }
    ];
    tabBody = `
      <div class="font-title" style="font-size:17px;color:var(--gold);margin-bottom:${rowGap};height:24px;flex-shrink:0;">Controls & Keybindings</div>
      <div style="display:flex;flex-direction:column;gap:${rowGap};">
        ${groups.map(g => `
          <div class="rust-card" style="padding:7px 12px;">
            <div class="font-title" style="font-size:13.5px;color:var(--gold);margin-bottom:4px;">${g.head}</div>
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:5px;">
              ${g.rows.map(([act, key]) => `
                <div style="display:flex;align-items:center;justify-content:space-between;background:#07080c;border:1px solid #2d333f;padding:3px 8px;">
                  <span style="font-size:12px;color:var(--text);">${act}</span>
                  <span class="font-mono" style="font-size:11px;color:var(--gold);">${key}</span>
                </div>
              `).join('')}
            </div>
          </div>
        `).join('')}
      </div>
    `;
  }

  const modalInner = `
    <!-- Clean Player Header Bar -->
    <div style="display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #374151;padding-bottom:7px;margin-bottom:8px;flex-shrink:0;">
      <span class="font-title" style="font-size:21px;color:var(--gold);">Settings</span>
      ${closeBtn720("requestCloseSettings('close')", "Close [Esc / Right-Click]")}
    </div>

    ${isRail ? `
      <!-- Vertical Left Rail with Display at Top + Right Inset Content Pane -->
      <div style="display:flex;gap:14px;flex:1;overflow:hidden;">
        <div style="width:${isPopup ? '184px' : '216px'};display:flex;flex-direction:column;gap:7px;flex-shrink:0;">
          ${tabs.map(t => {
            const active = STATE.settingsTab === t.id;
            return `
              <div class="rust-card ${active ? 'selected' : ''}"
                   style="height:${isPopup ? '44px' : '54px'};padding:${isPopup ? '8px 12px' : '10px 14px'};cursor:pointer;display:flex;align-items:center;justify-content:space-between;flex-shrink:0;"
                   onclick="STATE.settingsTab='${t.id}';render();">
                <div>
                  <div class="font-title" style="font-size:15.5px;color:${active ? 'var(--gold)' : 'var(--text)'};">${t.id}</div>
                  ${!isPopup ? `<div style="font-size:11px;color:var(--dim);">${t.sub}</div>` : ''}
                </div>
                <span style="font-size:14px;color:${active ? 'var(--gold)' : 'var(--dim)'};">▸</span>
              </div>
            `;
          }).join('')}
        </div>
        <div class="rust-inset" style="flex:1;padding:${isPopup ? '10px 14px' : '14px 18px'};overflow:hidden;">
          ${tabBody}
        </div>
      </div>
    ` : `
      <!-- Horizontal Top Tabs + Content Area -->
      <div style="display:flex;flex-direction:column;flex:1;overflow:hidden;">
        <div style="display:flex;gap:8px;margin-bottom:8px;flex-shrink:0;">
          ${tabs.map(t => {
            const active = STATE.settingsTab === t.id;
            return `
              <button class="rust-btn-720 ${active ? 'primary' : ''}" style="flex:1;height:32px;font-size:13.5px;" onclick="STATE.settingsTab='${t.id}';render();">
                ${t.id}
              </button>
            `;
          }).join('')}
        </div>
        <div class="rust-inset" style="flex:1;padding:${isPopup ? '10px 14px' : '14px 18px'};overflow:hidden;">
          ${tabBody}
        </div>
      </div>
    `}

    <!-- Clean Player Footer Bar (all settings except Display Mode / Resolution apply instantly) -->
    <div style="display:flex;align-items:center;justify-content:space-between;border-top:1px solid #374151;padding-top:7px;margin-top:8px;font-size:12px;color:var(--dim);flex-shrink:0;">
      <span class="font-mono">All settings apply instantly (except Resolution & Display Mode, which use Apply) · [Esc / Right-Click] Close</span>
      <div style="display:flex;gap:8px;">
        <button class="rust-btn-720" onclick="restoreDefaultSettings()">Restore Defaults</button>
        ${!isMainMenuScreen ? `
          <button class="rust-btn-720" onclick="requestCloseSettings('return_title')">Return to Title</button>
        ` : ''}
      </div>
    </div>
  `;

  // Confirmation Modal Popup when Display Apply is clicked ("Keep Display Settings?" with 15s Revert Countdown — Small Popup -> Reduced Corner-Only Frame)
  const displayConfirmOverlay = STATE.displayConfirmOpen ? `
    <div class="modal-backdrop" style="z-index:45;">
      ${ornateCornerBox(`
        <div style="display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #374151;padding-bottom:6px;">
          <span class="font-title" style="font-size:19px;color:var(--gold);">Keep Display Settings?</span>
          ${closeBtn720("revertDisplaySettings()", "Revert & Close [Esc / Right-Click]")}
        </div>
        <div style="font-size:13.5px;color:var(--text);line-height:1.5;margin:10px 0;">
          Display switched to <b style="color:var(--gold);">${appliedMode} · ${appliedRes}</b>.<br>
          Automatically reverting to previous display settings in <b class="font-mono" style="color:#fca5a5;">${STATE.displayConfirmSec}s</b>.
        </div>
        <div style="display:flex;justify-content:flex-end;gap:10px;">
          <button class="rust-btn-720" onclick="revertDisplaySettings()">Revert</button>
          <button class="rust-btn-720 primary" onclick="confirmDisplaySettings()">Keep Changes [Enter]</button>
        </div>
      `, 'width:520px;height:210px;padding:18px 22px;')}
    </div>
  ` : '';

  // Confirmation Modal Popup when user tries to close Settings (via Right-Click, Esc, [X], or Return to Title) with unapplied Display Mode / Resolution changes
  const unappliedDisplayOverlay = STATE.settingsAskUnapplied !== null ? `
    <div class="modal-backdrop" style="z-index:45;">
      ${ornateCornerBox(`
        <div style="display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #374151;padding-bottom:6px;">
          <span class="font-title" style="font-size:19px;color:var(--gold);">Unapplied Display Changes?</span>
          ${closeBtn720("STATE.settingsAskUnapplied=null;render();", "Stay in Settings [Esc / Right-Click]")}
        </div>
        <div style="font-size:13.5px;color:var(--text);line-height:1.5;margin:10px 0;">
          You selected <b style="color:var(--gold);">${pendingMode} · ${pendingRes}</b> without clicking <b>Apply</b>.<br>
          <span style="font-size:12px;color:var(--dim);">All other settings apply automatically. Apply or discard your staged display changes before closing?</span>
        </div>
        <div style="display:flex;justify-content:flex-end;gap:10px;">
          <button class="rust-btn-720" onclick="STATE.settingsAskUnapplied=null;render();">Stay in Settings</button>
          <button class="rust-btn-720" onclick="discardAndCloseSettings()">Discard & Close</button>
          <button class="rust-btn-720 primary" onclick="applyUnappliedAndKeepInSettings()">Apply Display [Enter]</button>
        </div>
      `, 'width:560px;height:218px;padding:18px 22px;')}
    </div>
  ` : '';

  if (isMainMenuScreen) {
    return `
      <div style="position:absolute;inset:0;width:1280px;height:720px;z-index:25;overflow:hidden;">
        <img src="${ASSETS.bg.title}" class="pixel" style="position:absolute;inset:0;width:1280px;height:720px;object-fit:cover;filter:brightness(0.32) blur(1px);">
        <div style="position:absolute;inset:0;background:radial-gradient(circle at 50% 50%, rgba(10,11,16,0.45) 0%, rgba(10,11,16,0.88) 100%);"></div>
        ${ornateBox(modalInner, 'position:absolute;left:64px;top:36px;width:1152px;height:648px;padding:18px 24px;z-index:5;')}
        ${displayConfirmOverlay}
        ${unappliedDisplayOverlay}
      </div>
    `;
  }

  return `
    <div class="modal-backdrop" style="z-index:25;">
      ${ornateBox(modalInner, 'width:940px;height:520px;padding:14px 20px;')}
      ${displayConfirmOverlay}
      ${unappliedDisplayOverlay}
    </div>
  `;
}


