// ============================================================================
// GLOBAL GAME STATE (Playable Campaign State across all 7 emberwake_gui screens)
// ============================================================================
const STATE = {
  screen: 'main_menu',          // 'main_menu' | 'load_slots' | 'runner' | 'story' | 'region' | 'worldmap' | 'battle'
  modal: null,                  // null | 'board' | 'shop' | 'party' | 'bunks' | 'journal' | 'results' | 'timeline'
  newGameModalOpen: false,      // Campaign start prompt for Difficulty & Ironman Chronicle
  settingsOpen: false,          // SettingsScreen
  settingsAskUnapplied: null,   // null | 'close' | 'return_title' (guard when closing Settings with unapplied Display changes)
  settingsTab: 'Display',       // 'Display' first, then 'Sound' | 'Gameplay' | 'Battle' | 'Controls'
  settingsLayout: 'tab_rail',   // 'tab_rail' | 'top_tabs'
  settingsContextMode: 'auto',  // 'auto' (Fullscreen on Main Menu, Popup in-game) | 'force_full' | 'force_popup'
  displayConfirmOpen: false,    // Keep Display Settings? / Revert modal popup
  displayConfirmSec: 15,        // 15s auto-revert countdown
  settings: {
    // Display tab: Resolution at top, merged Display Mode (Windowed / Borderless Fullscreen / Exclusive Fullscreen),
    // staged pendingResIdx & pendingWindowModeIdx with Apply button + Revert popup, and Language in Display tab
    resolutions: ['1280×720', '1366×768', '1600×900', '1920×1080', '2560×1440'],
    resIdx: 0,                  // Applied resolution index
    pendingResIdx: 0,           // Staged resolution index (arrow toggles do not rescale immediately)
    prevResIdx: 0,              // Previous resolution index for Revert popup
    windowModes: ['Fullscreen', 'Borderless Fullscreen', 'Borderless Window', 'Windowed'],
    windowModeIdx: 1,           // Applied display mode (1 = Borderless Fullscreen)
    pendingWindowModeIdx: 1,    // Staged display mode
    prevWindowModeIdx: 1,       // Previous display mode for Revert popup
    browserAutoFit: true,       // Dynamic browser layout auto-resize
    languages: ['English', 'Español', 'Français', 'Deutsch', '日本語'],
    languageIdx: 0,             // Language in Display tab
    textSizes: [
      { label: 'Small (85%)', scale: 0.85 },
      { label: 'Normal (100%)', scale: 1.00 },
      { label: 'Large (115%)', scale: 1.15 },
      { label: 'Extra Large (130%)', scale: 1.30 }
    ],
    textSizeIdx: 1,             // Normal (100%) default
    vsync: true,
    fpsCaps: ['Unlimited', '30 FPS', '60 FPS', '120 FPS', '144 FPS'],
    fpsIdx: 2,                  // 60 FPS default
    reduceMotion: false,
    // Sound tab
    muteAll: false,
    masterVolume: 85,
    musicVolume: 75,
    sfxVolume: 90,
    interfaceVolume: 80,
    ambienceVolume: 80,
    muteInBackground: true,
    // Gameplay tab (mid-campaign adjustable preferences only)
    textSpeeds: ['Slow (30 cps)', 'Normal (60 cps)', 'Fast (120 cps)', 'Instant'],
    textSpeedIdx: 1,
    autoAdvance: false,
    tutorials: true,
    mapNames: 'On hover',       // 'On hover' | 'Always'
    mapZooms: ['0.75×', '1.00×', '1.25×', '1.50×'],
    mapZoomIdx: 1,
    // Campaign Start Options (selected in the New Game prompt, not in Settings)
    difficulties: ['Story (Forgiving)', 'Tactical (Standard)', 'Crucible (Hard)'],
    difficultyIdx: 1,
    ironmanMode: false,
    // Battle tab
    animSpeedIdx: 2,
    animSpeeds: ['0.50×', '0.75×', '1.00×', '1.25×', '1.50×', '2.00×'],
    skipAnim: false,
    playbackSpeeds: ['1× Normal', '0.5× Half-Speed', 'Block-by-Block'],
    playbackSpeedIdx: 0,
    showEnemyTells: true,
    warnMissingActions: true,
    autoFillRest: true,
    autoConfirmRound: true,
    seeThrough: 'Selected unit'
  },
  slotsMode: 'slot_card',
  slotsSaving: false,           // false: Load game, true: Save game
  slotSelected: 0,              // Click 1 selects slot; Click 2 (or action button / Enter) executes Load/Save/Overwrite
  slotAskDelete: null,          // slot index pending delete confirmation
  slotAskOverwrite: null,       // slot index pending overwrite confirmation in Save mode
  slotToast: 'Step 1: Click a chronicle slot to select it (for Lock / Delete). Step 2: Click selected slot again or press [Enter] to Load.',
  slots: [
    {
      id: 0,
      kind: 'auto',
      empty: false,
      locked: false,
      title: 'Autosave · Act I: Chronicles of the Sunk Ward',
      act: 'Act I · Chronicles of the Sunk Ward',
      place: 'The Sluice Undercroft (Hub Sanctuary)',
      lead: 'Lead: Corvo\'s Toll (Third Ward Gate)',
      day: 3,
      tithe: 15,
      scrap: 65,
      tallow: 3,
      when: '14m ago',
      thumb: 'sluice_undercroft',
      party: ['kaelen', 'vesper', 'maren', 'hollis']
    },
    {
      id: 1,
      kind: 'manual',
      empty: false,
      locked: true,
      title: 'Slot 1 · Act I: Chronicles of the Sunk Ward',
      act: 'Act I · Chronicles of the Sunk Ward',
      place: 'Third Ward · Ash-Crawl Crossroads',
      lead: 'Lead: Parley with Gate-Sergeant Corvo',
      day: 2,
      tithe: 16,
      scrap: 45,
      tallow: 4,
      when: '2h ago',
      thumb: 'corvos_gate',
      party: ['kaelen', 'vesper', 'maren', 'hollis']
    },
    {
      id: 2,
      kind: 'manual',
      empty: false,
      locked: false,
      title: 'Slot 2 · Prologue: Seventh Sluice Bridge',
      act: 'Act I · Chronicles of the Sunk Ward',
      place: 'Seventh Sluice Canal Bridge',
      lead: 'Prologue: Break Captain Vane\'s Cordon',
      day: 1,
      tithe: 18,
      scrap: 30,
      tallow: 3,
      when: 'Yesterday',
      thumb: 'story_alley',
      party: ['kaelen', 'vesper', 'maren', 'hollis']
    },
    {
      id: 3,
      kind: 'manual',
      empty: true,
      locked: false,
      title: 'Slot 3 · Empty Chronicle',
      act: '—',
      place: 'No recorded chronicle in this slot',
      lead: 'Available for a new save',
      day: 0,
      tithe: 0,
      scrap: 0,
      tallow: 0,
      when: 'Empty',
      thumb: 'sluice_undercroft',
      party: []
    }
  ],
  menuHover: 0,              // 0: Continue, 1: New game, 2: Load game, 3: Settings, 4: Quit
  runnerWorld: 'sunk_ward',  // 'sunk_ward' | 'ashlands'
  runnerTab: 'encounters',   // 'encounters' | 'locations' | 'stories'
  runnerSel: 0,

  // Campaign resources & clocks (`CAMPAIGN1`)
  day: 3,
  tithingDays: 15,
  scrap: 65,
  tallow: 3,
  alertTier: 1,              // 0: Quiet, 1: Stirred, 2: Hunted, 3: Lockdown
  travelMode: 'Stealth',     // 'Stealth' | 'Fast'

  // StoryStage & Vignette state (`story.rs` + `vignette.rs`)
  storyId: 'prologue_intro',
  storyCardIdx: 0,
  storyMode: 'vn_stage',        // 'vn_stage' | 'map_stage' | 'vignette_reel'
  storyActorStyle: 'full_sprite', // 'full_sprite' | 'bust_cutout' | 'backdrop_only'
  storyMoodOverride: null,      // null (use card.mood) | 'neutral' | 'determined' | 'angry' | 'hurt' | 'surprised' | 'afraid'
  storyReactionOverride: null,  // null (use actor.reactionFx) | 'steel_flash' | 'anger_burst' | 'alert_shock' | 'alchemy_flare' | 'solemn_glow'
  storySkipConfirmOpen: false,
  vignetteAuto: false,          // Auto-advance toggle [A] in minimal Vignette player
  vignetteLogOpen: false,       // Backlog overlay toggle [L] in minimal Vignette player

  // RegionMap state (`region.rs` + `talk.rs`)
  regionId: 'sluice_undercroft', // 'sluice_undercroft' | 'corvos_gate'
  hereOpen: true,
  playerTile: { x: 7, y: 6 },
  targetTile: null,
  talkNpcId: 'aline',        // active NPC in bottom TalkBox
  talkNodeId: 'root',
  rollBanner: null,          // { label, roll, mod, total, dc, passed }
  eventLog: [
    'Entered The Sluice Undercroft.',
    'The Tallow Hearth burns low (3 Tallow left).',
    'Sister Aline has surgical draughts for trade.'
  ],

  // BoardPane (`board.rs`) & ShopPane (`shop.rs`)
  boardSelId: 'q_corvo',
  shopKeeper: 'aline',       // 'aline' | 'bram'
  shopSelIdx: 0,

  // WorldMap (`worldmap.rs`)
  worldNodeId: 'n_sluice',

  // Character / Party (`party.rs`)
  partyHeroId: 'kaelen',
  partyTab: 'Stats',         // 'Stats' | 'Level up' | 'Skills' | 'Gear'

  // Roster (6 Founders — 4 Active, 2 Reserve in The Dry Hollows)
  roster: {
    kaelen: {
      id: 'kaelen', name: 'Kaelen', title: 'Disgraced Surgeon', classId: 'warrior', roleName: 'Vanguard',
      level: 2, xp: 140, active: true, wounded: false,
      hp: 44, maxHp: 44, guard: 12, poise: 5, maxPoise: 5, ap: 6, speed: 11,
      stats: { Might: 15, Agility: 11, Knowledge: 13, Will: 12, Awareness: 10 },
      meterName: 'Bile Phial', meter: 40, maxMeter: 100,
      weapon: 'Surgeon\'s Billhook', armor: 'Waxed Brigandine', trinket: 'Blood-Stained Lancet',
      pouch: [
        { name: 'Coagulant Vial', pips: 2, icon: 'cauterize', desc: 'Heal 15 HP & cure Bleed (0B Free)' },
        { name: 'Bile Flask', pips: 2, icon: 'volatile_flask', desc: 'Throw 3-tile Poison/Burn pool' }
      ],
      basics: ['move', 'rest', 'strike', 'active_guard', 'disengage', 'shield_bash'],
      actives: ['billhook_hook', 'billhook_sunder', 'iron_brace'],
      passives: ['Field Triage', 'Arterial Precision'],
      keystone: 'Crimson Transfusion',
      intentOrder: ['Assault', 'Guard', 'Maneuver', 'Support']
    },
    vesper: {
      id: 'vesper', name: 'Vesper', title: 'Cutthroat Scout', classId: 'rogue', roleName: 'Flanker',
      level: 2, xp: 140, active: true, wounded: false,
      hp: 30, maxHp: 30, guard: 6, poise: 3, maxPoise: 3, ap: 7, speed: 15,
      stats: { Might: 11, Agility: 16, Knowledge: 12, Will: 10, Awareness: 14 },
      meterName: 'Shadow Momentum', meter: 60, maxMeter: 100,
      weapon: 'Twin Gutting Knives', armor: 'Soot-Dyed Leathers', trinket: 'Lockpick Ring',
      pouch: [
        { name: 'Ash Smoke Bomb', pips: 2, icon: 'smoke_bomb', desc: 'Grant Stealth & Blind adjacent foes' },
        { name: 'Throwing Spike', pips: 1, icon: 'bleeding_cut', desc: 'Range 3, 6 dmg + 2 Bleed' }
      ],
      basics: ['move', 'rest', 'strike', 'active_guard', 'disengage', 'flank_cut'],
      actives: ['shadow_step', 'throat_slit', 'ash_cloud_fade'],
      passives: ['Alley Stalker', 'Serrated Edge'],
      keystone: 'Arterial Rupture',
      intentOrder: ['Assault', 'Maneuver', 'Guard', 'Support']
    },
    maren: {
      id: 'maren', name: 'Maren', title: 'Plague Alchemist', classId: 'priest', roleName: 'Support / DoT',
      level: 2, xp: 130, active: true, wounded: false,
      hp: 32, maxHp: 32, guard: 8, poise: 3, maxPoise: 3, ap: 6, speed: 12,
      stats: { Might: 9, Agility: 11, Knowledge: 16, Will: 15, Awareness: 13 },
      meterName: 'Alembic Heat', meter: 50, maxMeter: 100,
      weapon: 'Censer & Bone Saw', armor: 'Alchemist Apron', trinket: 'Sulfur Phial',
      pouch: [
        { name: 'Laudanum Draught', pips: 2, icon: 'field_mend', desc: 'Restore 18 HP & +1 Poise' },
        { name: 'Tallow Salve', pips: 1, icon: 'cauterize', desc: 'Clear Burn/Poison & +6 Guard' }
      ],
      basics: ['move', 'rest', 'strike', 'active_guard', 'disengage', 'field_mend'],
      actives: ['cauterize', 'volatile_flask', 'tallow_leech'],
      passives: ['Fume Immunity', 'Distilled Potency'],
      keystone: 'Scarlet Panacea',
      intentOrder: ['Support', 'Assault', 'Guard', 'Maneuver']
    },
    hollis: {
      id: 'hollis', name: 'Hollis', title: 'Deserter Marksman', classId: 'ranger', roleName: 'Control / Ranged',
      level: 2, xp: 130, active: true, wounded: false,
      hp: 34, maxHp: 34, guard: 8, poise: 4, maxPoise: 4, ap: 6, speed: 13,
      stats: { Might: 12, Agility: 15, Knowledge: 11, Will: 12, Awareness: 16 },
      meterName: 'Deadeye Focus', meter: 75, maxMeter: 100,
      weapon: 'Watch Arbalest', armor: 'Rain-Cloak', trinket: 'Windage Sight',
      pouch: [
        { name: 'Barbed Quiver', pips: 2, icon: 'pinning_bolt', desc: 'Next shot inflicts Immobilize' },
        { name: 'Caltrop Pouch', pips: 2, icon: 'caltrop_scatter', desc: 'Seed 3 tiles with Slow + Bleed' }
      ],
      basics: ['move', 'rest', 'strike', 'active_guard', 'disengage', 'quick_shot'],
      actives: ['pinning_bolt', 'overwatch_line', 'caltrop_scatter'],
      passives: ['High-Ground Sight', 'Armor Piercer'],
      keystone: 'Kill-Zone Overwatch',
      intentOrder: ['Assault', 'Maneuver', 'Support', 'Guard']
    },
    dunstan: {
      id: 'dunstan', name: 'Dunstan', title: 'Foundry Stoker', classId: 'brawler', roleName: 'Poise Breaker',
      level: 2, xp: 110, active: false, wounded: true,
      hp: 48, maxHp: 48, guard: 14, poise: 5, maxPoise: 5, ap: 6, speed: 9,
      stats: { Might: 17, Agility: 9, Knowledge: 10, Will: 14, Awareness: 10 },
      meterName: 'Slag Fury', meter: 30, maxMeter: 100,
      weapon: 'Slag Caestus', armor: 'Foundry Leather', trinket: 'Kiln Chain',
      pouch: [
        { name: 'Slag Grit', pips: 2, icon: 'slag_vent_slam', desc: '+10 Guard & Burn immunity 2 rounds' }
      ],
      basics: ['move', 'rest', 'strike', 'active_guard', 'disengage', 'offhand_jab'],
      actives: ['furnace_haymaker', 'slag_vent_slam', 'cinder_shove'],
      passives: ['Kiln-Hardened Skin', 'Crushing Knuckles'],
      keystone: 'Molten Impact',
      intentOrder: ['Assault', 'Guard', 'Maneuver', 'Support']
    },
    elyse: {
      id: 'elyse', name: 'Elyse', title: 'Hedge Conduit', classId: 'mage', roleName: 'Zone / Arc Caster',
      level: 2, xp: 115, active: false, wounded: false,
      hp: 28, maxHp: 28, guard: 6, poise: 3, maxPoise: 3, ap: 6, speed: 12,
      stats: { Might: 8, Agility: 12, Knowledge: 17, Will: 15, Awareness: 13 },
      meterName: 'Galvanic Charge', meter: 80, maxMeter: 100,
      weapon: 'Copper Wire Staff', armor: 'Insulated Coat', trinket: 'Ley Condenser',
      pouch: [
        { name: 'Copper Capacitor', pips: 2, icon: 'arc_lance', desc: '+2 AP this round' }
      ],
      basics: ['move', 'rest', 'strike', 'active_guard', 'disengage', 'arcane_bolt'],
      actives: ['arc_lance', 'static_tether', 'ward_pulse'],
      passives: ['Wet-Tile Conduction', 'Static Feedback'],
      keystone: 'Storm Sluice',
      intentOrder: ['Assault', 'Support', 'Maneuver', 'Guard']
    }
  },

  // Quests / Notice Board (`board.rs`)
  quests: [
    {
      id: 'q_corvo', section: 'Main', title: 'Corvo\'s Gate (The Third Ward Toll)',
      status: 'Under way', deadlineDays: 4, rewardScrap: 45, rewardTallow: 1,
      where: 'Third Ward Gate / Ash-Crawl Crossroads', nodeId: 'n_corvo',
      summary: 'Gate-Sergeant Corvo has chained the Sluice grate and demanded thirty Scrap or a certified Tallow Guild seal before nightfall.',
      nowStep: 'Travel to Third Ward Gate and parley with Sergeant Corvo (Pay 30 Scrap, Forge Seal [KNOW >= 12], or Break the Gate).',
      thenStep: 'Open the mid-ward supply artery to reach The Fourth Kiln.'
    },
    {
      id: 'q_bonemill', section: 'Main', title: 'The Bone-Mill Raid (M3)',
      status: 'On offer', deadlineDays: 6, rewardScrap: 80, rewardTallow: 2,
      where: 'The Fourth Kiln (Bone-Mill)', nodeId: 'n_kiln',
      summary: 'Foreman Grendel and Guild Master Orlov are rendering debtors in the Fourth Kiln to meet the Red Tithing quota.',
      nowStep: 'Breach the Fourth Kiln floor, survive the rhythmic Slag Vents, and break Foreman Grendel.',
      thenStep: 'Seize the Tallow Guild ledger and unlock the Crucible Watchpost.'
    },
    {
      id: 'q_crucible', section: 'Main', title: 'The Crucible Watchpost (Act I Climax)',
      status: 'On offer', deadlineDays: 18, rewardScrap: 150, rewardTallow: 4,
      where: 'Crucible Watchpost', nodeId: 'n_crucible',
      summary: 'Inquisitor-Prelate Nemeth and Proctor Quell prepare the Great Sluice Purge. Seize the Winch tile or break the Prelate before the Ward drowns.',
      nowStep: 'Storm the Crucible Watchpost and survive Phase 2 Consecration.',
      thenStep: 'Complete Act I of A Scarlet Lullaby.'
    },
    {
      id: 'q_grate', section: 'Jobs', title: 'Clear the Drowned Runoff Grate',
      status: 'On offer', deadlineDays: 3, rewardScrap: 35, rewardTallow: 1,
      where: 'Drowned Runoff Grate', nodeId: 'n_grate',
      summary: 'Feral Render-Beasts and Ash-Crawlers are nesting in the lower runoff grate, choking Sister Aline\'s herb intake.',
      nowStep: 'Eliminate the pack in the sunken canal basin.',
      thenStep: 'Return to Sister Aline for discounted surgical draughts.'
    }
  ],

  // Battle State (`battle/draw.rs`, `stage.rs`, `skill_menu.rs`)
  encId: 'enc_0c',
  battleStageMode: 'active_combat', // 'pre_battle' | 'transition_in' | 'active_combat' | 'boss_phase2' | 'elite_intro' | 'elite_aftermath'
  battleTransitionTimer: null,
  round: 1,
  selectedUnitId: 'kaelen',
  pinnedFoeId: null,
  hoverTile: null,
  skillMenu: null,           // { x, y, tileX, tileY, targetUnitId, hoveredSkillId }
  activeSkillId: null,       // skill awaiting target click
  playingRound: false,
  playBlockIdx: 0,           // 0..6 during playback
  combatLog: [
    'Round 1 begins — Foresight active: enemy intents revealed on timeline.',
    'Click any unit or tile on the 12×9 stage to open the icon-only SkillMenu.'
  ]
};

// ============================================================================
// SKILL CATALOG (`F3.2` 5 Universal Basics + 1 Role Basic + Signature Actives)
// ============================================================================
const SKILLS = {
  move:           { id: 'move', name: 'Move', row: 'Any', blocks: 1, ap: 1, range: 3, shape: 'Path', icon: 'move', desc: 'Step up to 3 tiles across the board.' },
  rest:           { id: 'rest', name: 'Rest', row: 'Any', blocks: 1, ap: 0, range: 0, shape: 'Self', icon: 'rest', desc: 'Recover +2 AP and steady breathing (+4 Guard).' },
  strike:         { id: 'strike', name: 'Strike', row: 'Assault', blocks: 2, ap: 2, range: 1, shape: 'Single', dmg: 10, poiseDmg: 1, icon: 'strike', desc: 'Universal Basic melee strike (10 dmg, 1 Poise).' },
  active_guard:   { id: 'active_guard', name: 'Active Guard', row: 'Guard', blocks: 2, ap: 1, range: 0, shape: 'Self', guard: 10, icon: 'active_guard', desc: 'Universal Basic: gain +10 Guard and +1 Poise.' },
  disengage:      { id: 'disengage', name: 'Disengage', row: 'Maneuver', blocks: 1, ap: 1, range: 2, shape: 'Self', icon: 'disengage', desc: 'Universal Basic: step 2 tiles back ignoring Engagement.' },

  // F3.2 Class Role Basics (1 per class)
  shield_bash:    { id: 'shield_bash', name: 'Shield Bash (Role)', row: 'Guard', blocks: 2, ap: 2, range: 1, shape: 'Single', dmg: 7, poiseDmg: 2, guard: 6, icon: 'shield_bash', desc: 'Warrior Role Basic (IR21 Dual Assault+Guard): 7 dmg, 2 Poise dmg, +6 Guard.' },
  flank_cut:      { id: 'flank_cut', name: 'Flank Cut (Role)', row: 'Maneuver', blocks: 1, ap: 2, range: 1, shape: 'Single', dmg: 9, poiseDmg: 1, status: 'bleed', icon: 'flank_cut', desc: 'Rogue Role Basic: fast 1B cut, +50% flank bonus & Bleed.' },
  field_mend:     { id: 'field_mend', name: 'Field Mend (Role)', row: 'Support', blocks: 2, ap: 2, range: 3, shape: 'Single', heal: 14, icon: 'field_mend', desc: 'Priest Role Basic: restore 14 HP and clear 1 stack of Bleed.' },
  quick_shot:     { id: 'quick_shot', name: 'Quick Shot (Role)', row: 'Assault', blocks: 1, ap: 2, range: 4, shape: 'Single', dmg: 8, poiseDmg: 1, icon: 'quick_shot', desc: 'Ranger Role Basic: snap ranged bolt (1 Block, Range 4).' },
  offhand_jab:    { id: 'offhand_jab', name: 'Offhand Jab (Role)', row: 'Assault', blocks: 1, ap: 1, range: 1, shape: 'Single', dmg: 6, poiseDmg: 2, icon: 'offhand_jab', desc: 'Brawler Role Basic: 1B Poise-chipping punch (2 Poise dmg).' },
  arcane_bolt:    { id: 'arcane_bolt', name: 'Arcane Bolt (Role)', row: 'Assault', blocks: 2, ap: 2, range: 4, shape: 'Single', dmg: 11, poiseDmg: 1, icon: 'arcane_bolt', desc: 'Mage Role Basic: galvanic bolt that arcs on wet tiles.' },

  // Signature Actives
  billhook_hook:  { id: 'billhook_hook', name: 'Billhook Drag', row: 'Maneuver', blocks: 2, ap: 3, range: 2, shape: 'Line', dmg: 12, poiseDmg: 2, pull: 1, icon: 'billhook_hook', desc: 'Pull target 1 tile closer, deal 12 dmg & 2 Poise dmg.' },
  billhook_sunder:{ id: 'billhook_sunder', name: 'Arterial Sunder', row: 'Assault', blocks: 3, ap: 3, range: 1, shape: 'Arc', dmg: 16, poiseDmg: 2, status: 'bleed', icon: 'billhook_sunder', desc: 'Heavy 3-tile Arc cleave: 16 dmg, 2 Poise, inflicts Bleed.' },
  iron_brace:     { id: 'iron_brace', name: 'Surgeon\'s Brace', row: 'Guard', blocks: 2, ap: 2, range: 0, shape: 'Self', guard: 16, heal: 6, icon: 'iron_brace', desc: 'Gain +16 Guard, heal 6 HP, and anchor against Push/Pull.' },
  shadow_step:    { id: 'shadow_step', name: 'Shadow Step', row: 'Maneuver', blocks: 1, ap: 2, range: 4, shape: 'Single', dmg: 11, poiseDmg: 1, icon: 'shadow_step', desc: 'Teleport behind target within 4 tiles and strike for 11 dmg.' },
  throat_slit:    { id: 'throat_slit', name: 'Crimson Lullaby', row: 'Assault', blocks: 2, ap: 3, range: 1, shape: 'Single', dmg: 18, poiseDmg: 2, status: 'bleed', icon: 'throat_slit', desc: 'High-lethality backstab: 18 dmg (+8 if target is Bleeding or Broken).' },
  ash_cloud_fade: { id: 'ash_cloud_fade', name: 'Ash-Cloud Fade', row: 'Guard', blocks: 1, ap: 2, range: 0, shape: 'Ring', guard: 10, icon: 'ash_cloud_fade', desc: 'Drop soot ring (RING shape): gain +10 Guard & Evasion.' },
  cauterize:      { id: 'cauterize', name: 'Brimstone Cauterize', row: 'Support', blocks: 2, ap: 2, range: 3, shape: 'Single', heal: 18, guard: 8, icon: 'cauterize', desc: 'Restore 18 HP, grant +8 Guard, and purge Bleed/Poison.' },
  volatile_flask: { id: 'volatile_flask', name: 'Volatile Tallow Flask', row: 'Assault', blocks: 2, ap: 3, range: 4, shape: 'Cross', dmg: 13, poiseDmg: 1, status: 'burn', icon: 'volatile_flask', desc: 'Hurl alchemical flask (Cross AoE): 13 dmg + Burn hazard.' },
  tallow_leech:   { id: 'tallow_leech', name: 'Sanguine Siphon', row: 'Support', blocks: 2, ap: 2, range: 3, shape: 'Single', dmg: 10, heal: 10, icon: 'tallow_leech', desc: 'Drain 10 HP from foe and transfer it to lowest-HP ally.' },
  pinning_bolt:   { id: 'pinning_bolt', name: 'Pinning Arbalest Bolt', row: 'Assault', blocks: 2, ap: 3, range: 5, shape: 'Single', dmg: 15, poiseDmg: 2, status: 'slow', icon: 'pinning_bolt', desc: 'Long-range heavy bolt: 15 dmg, 2 Poise dmg, inflicts Slow.' },
  overwatch_line: { id: 'overwatch_line', name: 'Sluice Overwatch', row: 'Assault', blocks: 3, ap: 3, range: 5, shape: 'Line', dmg: 14, poiseDmg: 2, icon: 'overwatch_line', desc: 'Pierce a 5-tile Line for 14 dmg and suppress enemy movement.' },
  caltrop_scatter:{ id: 'caltrop_scatter', name: 'Caltrop Scatter', row: 'Maneuver', blocks: 2, ap: 2, range: 3, shape: 'Cone', dmg: 8, status: 'bleed', icon: 'caltrop_scatter', desc: 'Seed a Cone with iron caltrops (8 dmg + Bleed + Slow).' }
};

function getSkillIcon(id) {
  const sk = SKILLS[id];
  const key = sk ? sk.icon : id;
  return ASSETS.skills[key] || ASSETS.skills.strike || ASSETS.ui.intent_attack;
}

// ============================================================================
// ENCOUNTER DEFINITIONS (`CAST1_ENCOUNTERS.md` — 0a, 0b, 0c, M2, M3, M5)
// ============================================================================
const ENCOUNTERS = {
  enc_0a: {
    id: 'enc_0a', title: 'Prologue 0a: The Alley', backdrop: 'alley',
    subtitle: 'Solo Kaelen tutorial — Break the Watch Halberdier\'s Poise & survive.',
    objective: 'Defeat the Castellan Watch patrol (2 foes).',
    nextStory: 'prologue_0b',
    walls: [[3,2],[3,3],[8,5],[8,6]],
    water: [[5,7],[6,7],[7,7]],
    hazards: [],
    heroes: [
      { id: 'kaelen', x: 2, y: 4 }
    ],
    foes: [
      { id: 'f_watch1', unitKey: 'watchman', portrait: 'watchman', name: 'Watch Halberdier', role: 'Brute', hp: 28, maxHp: 28, guard: 8, poise: 3, maxPoise: 3, x: 6, y: 4, intent: 'attack', intentName: 'Halberd Thrust (B2-B4 · 9 dmg)', passive: 'Shield Wall: +4 Guard when adjacent to an ally.' },
      { id: 'f_watch2', unitKey: 'crossbow', portrait: 'crossbow', name: 'Watch Arbalest', role: 'Shooter', hp: 20, maxHp: 20, guard: 4, poise: 2, maxPoise: 2, x: 9, y: 3, intent: 'attack', intentName: 'Aimed Bolt (B3-B5 · 8 dmg)', passive: 'Reload: Must spend 1B after firing.' }
    ]
  },
  enc_0b: {
    id: 'enc_0b', title: 'Prologue 0b: The Rooftop', backdrop: 'rooftops',
    subtitle: 'Vesper joins from the chimneys — flank the Arbalests on the slate ridge.',
    objective: 'Route the Rooftop Lookouts with Kaelen & Vesper.',
    nextStory: 'prologue_0c',
    walls: [[4,1],[4,2],[7,6],[7,7]],
    water: [],
    hazards: [{ x: 6, y: 4, type: 'brazier', label: 'Tallow Brazier (Knockback = Burn)' }],
    heroes: [
      { id: 'kaelen', x: 2, y: 5 },
      { id: 'vesper', x: 2, y: 2 }
    ],
    foes: [
      { id: 'f_bailiff', unitKey: 'bailiff', portrait: 'bailiff', name: 'Castellan Bailiff', role: 'Elite Brute', hp: 36, maxHp: 36, guard: 12, poise: 4, maxPoise: 4, x: 6, y: 5, intent: 'area', intentName: 'Mace Sweep (B2-B5 · 12 dmg Arc)', passive: 'Iron Collar: Immune to Pull while Guard > 0.' },
      { id: 'f_xbow1', unitKey: 'crossbow', portrait: 'crossbow', name: 'Slate Marksman', role: 'Shooter', hp: 22, maxHp: 22, guard: 4, poise: 2, maxPoise: 2, x: 9, y: 2, intent: 'attack', intentName: 'Pinning Bolt (B2-B4 · 9 dmg)', passive: 'High Slate: +2 Range from elevation.' },
      { id: 'f_xbow2', unitKey: 'crossbow', portrait: 'crossbow', name: 'Watch Lookout', role: 'Shooter', hp: 22, maxHp: 22, guard: 4, poise: 2, maxPoise: 2, x: 9, y: 6, intent: 'guard', intentName: 'Brace & Signal (B1-B3 · +8 Guard)', passive: 'Flare Horn: Raises Ward Alert if unchallenged.' }
    ]
  },
  enc_0c: {
    id: 'enc_0c', title: 'Prologue 0c: The Sluice Bridge', backdrop: 'canal',
    subtitle: 'Maren & Hollis join — break Captain Vane\'s cordon before the grates lock!',
    objective: 'Break Captain Vane or clear the Sluice Bridge.',
    nextStory: 'hearth_arrival',
    walls: [[5,0],[5,1],[5,7],[5,8]],
    water: [[6,0],[6,1],[6,7],[6,8],[7,0],[7,8]],
    hazards: [{ x: 6, y: 4, type: 'grate', label: 'Sluice Spillway (Wet Tile)' }],
    heroes: [
      { id: 'kaelen', x: 3, y: 4 },
      { id: 'vesper', x: 3, y: 2 },
      { id: 'maren',  x: 1, y: 5 },
      { id: 'hollis', x: 1, y: 3 }
    ],
    foes: [
      { id: 'f_vane', unitKey: 'vane', portrait: 'vane', name: 'Captain Vane', role: 'Boss · Watch Commander', hp: 58, maxHp: 58, guard: 16, poise: 5, maxPoise: 5, x: 8, y: 4, intent: 'area', intentName: 'Executioner\'s Cleave (B2-B5 · 16 dmg)', passive: 'No Quarter: Gains +4 Might when below 50% HP.' },
      { id: 'f_w1', unitKey: 'watchman', portrait: 'watchman', name: 'Bridge Halberdier', role: 'Brute', hp: 28, maxHp: 28, guard: 10, poise: 3, maxPoise: 3, x: 7, y: 3, intent: 'guard', intentName: 'Phalanx Lock (B1-B3 · +10 Guard)', passive: 'Interpose: Absorbs hits aimed at Captain Vane.' },
      { id: 'f_w2', unitKey: 'flayer', portrait: 'flayer', name: 'Tallow Hound-Handler', role: 'Skirmisher', hp: 24, maxHp: 24, guard: 6, poise: 3, maxPoise: 3, x: 7, y: 5, intent: 'attack', intentName: 'Flensing Hook (B1-B3 · 10 dmg + Bleed)', passive: 'Blood Scent: +3 Speed vs Bleeding targets.' }
    ]
  },
  enc_m2: {
    id: 'enc_m2', title: 'M2: Corvo\'s Gate (Third Ward Toll)', backdrop: 'alley',
    subtitle: 'Dilemma Escalation — You refused Sergeant Corvo\'s extortion at the toll gate.',
    objective: 'Defeat Gate-Sergeant Corvo & his Tallow Enforcers.',
    nextStory: null,
    walls: [[6,0],[6,1],[6,2],[6,6],[6,7],[6,8]],
    water: [[4,7],[5,7]],
    hazards: [{ x: 6, y: 4, type: 'gate', label: 'Toll Portcullis Choke' }],
    heroes: [
      { id: 'kaelen', x: 3, y: 4 },
      { id: 'vesper', x: 3, y: 3 },
      { id: 'maren',  x: 2, y: 5 },
      { id: 'hollis', x: 1, y: 3 }
    ],
    foes: [
      { id: 'f_corvo', unitKey: 'bailiff', portrait: 'bailiff', name: 'Gate-Sergeant Corvo', role: 'Mini-Boss · Toll Warden', hp: 52, maxHp: 52, guard: 18, poise: 5, maxPoise: 5, x: 7, y: 4, intent: 'area', intentName: 'Tax-Collector\'s Maul (B2-B5 · 15 dmg)', passive: 'Extortionist\'s Plate: Starts with 18 Guard; drops 45 Scrap.' },
      { id: 'f_flayer1', unitKey: 'flayer', portrait: 'flayer', name: 'Tallow Guild Flayer', role: 'Skirmisher', hp: 28, maxHp: 28, guard: 6, poise: 3, maxPoise: 3, x: 8, y: 3, intent: 'attack', intentName: 'Boning Knife Flurry (B1-B3 · 11 dmg)', passive: 'Render-Blade: Inflicts 2 Bleed on unguarded HP.' },
      { id: 'f_purifier', unitKey: 'purifier', portrait: 'purifier', name: 'Inquisitorial Purifier', role: 'Caster', hp: 26, maxHp: 26, guard: 8, poise: 3, maxPoise: 3, x: 9, y: 5, intent: 'support', intentName: 'Censer Benediction (B2-B4 · +12 Guard)', passive: 'Sanctified Tallow: Allies within 2 tiles resist Burn.' }
    ]
  },
  enc_m3: {
    id: 'enc_m3', title: 'M3: The Fourth Kiln (Bone-Mill Raid)', backdrop: 'foundry',
    subtitle: 'Every even round, Slag Vents erupt for 10 Burn damage & -2 Poise!',
    objective: 'Defeat Foreman Grendel & break the Bone-Mill.',
    nextStory: null,
    walls: [[4,2],[4,6],[8,2],[8,6]],
    water: [],
    hazards: [
      { x: 5, y: 3, type: 'slag', label: 'Slag Vent (Erupts Even Rounds: 10 Burn)' },
      { x: 5, y: 5, type: 'slag', label: 'Slag Vent (Erupts Even Rounds: 10 Burn)' },
      { x: 7, y: 4, type: 'slag', label: 'Slag Vent (Erupts Even Rounds: 10 Burn)' }
    ],
    heroes: [
      { id: 'kaelen', x: 2, y: 3 },
      { id: 'vesper', x: 2, y: 5 },
      { id: 'maren',  x: 1, y: 4 },
      { id: 'hollis', x: 1, y: 2 }
    ],
    foes: [
      { id: 'f_grendel', unitKey: 'grendel', portrait: 'orlov', name: 'Foreman Grendel', role: 'Boss · Kiln Overseer', hp: 72, maxHp: 72, guard: 20, poise: 5, maxPoise: 5, x: 8, y: 4, intent: 'area', intentName: 'Bone-Mill Pulverizer (B3-B6 · 19 dmg + Push)', passive: 'Slag-Tempered: Takes 0 Burn from Slag Vents; Shoves foes onto vents.' },
      { id: 'f_husk', unitKey: 'husk', portrait: 'husk', name: 'Calcified Husk', role: 'Brute', hp: 38, maxHp: 38, guard: 14, poise: 4, maxPoise: 4, x: 6, y: 3, intent: 'attack', intentName: 'Ossified Slam (B2-B4 · 12 dmg)', passive: 'Brittle Shell: Loses all Guard when Poise breaks.' },
      { id: 'f_flag', unitKey: 'flagellant', portrait: 'flagellant', name: 'Penitent Flagellant', role: 'Berserker', hp: 30, maxHp: 30, guard: 0, poise: 3, maxPoise: 3, x: 6, y: 5, intent: 'attack', intentName: 'Rapturous Scourge (B1-B3 · 14 dmg)', passive: 'Martyr\'s Ecstasy: Gains +3 Speed when Bleeding or Burning.' }
    ]
  },
  enc_m5: {
    id: 'enc_m5', title: 'M5: Crucible Watchpost (Act I Climax)', backdrop: 'watchpost',
    subtitle: 'Hold the Sluice Winch tile (6,4) for 2 rounds OR break Inquisitor-Prelate Nemeth!',
    objective: 'Defeat Prelate Nemeth & Proctor Quell or Seize the Winch.',
    nextStory: null,
    walls: [[4,1],[4,7],[8,1],[8,7]],
    water: [[5,0],[6,0],[7,0],[5,8],[6,8],[7,8]],
    hazards: [
      { x: 6, y: 4, type: 'winch', label: 'Great Sluice Winch (Interactive Objective Tile)' },
      { x: 7, y: 3, type: 'consecrate', label: 'Sanctified Pyre (+Burn to Heretics)' },
      { x: 7, y: 5, type: 'consecrate', label: 'Sanctified Pyre (+Burn to Heretics)' }
    ],
    heroes: [
      { id: 'kaelen', x: 3, y: 4 },
      { id: 'vesper', x: 3, y: 2 },
      { id: 'maren',  x: 2, y: 5 },
      { id: 'hollis', x: 1, y: 3 }
    ],
    foes: [
      { id: 'f_nemeth', unitKey: 'nemeth', portrait: 'nemeth', name: 'Prelate Nemeth', role: 'Act I Boss · High Inquisitor', hp: 95, maxHp: 95, guard: 24, poise: 5, maxPoise: 5, x: 9, y: 4, intent: 'area', intentName: 'Scarlet Edict (B2-B6 · 22 dmg Cross + Burn)', passive: 'Phase 2 Consecration: At 50% HP, ignites all Sluice pyres and gains +2 AP.' },
      { id: 'f_quell', unitKey: 'quell', portrait: 'quell', name: 'Proctor Quell', role: 'Elite Sub-Boss', hp: 54, maxHp: 54, guard: 16, poise: 4, maxPoise: 4, x: 7, y: 4, intent: 'guard', intentName: 'Aegis of the Rubric (B1-B4 · +14 Guard)', passive: 'Winch Warden: Blocks Winch interaction while adjacent.' },
      { id: 'f_pur1', unitKey: 'purifier', portrait: 'purifier', name: 'Choir Purifier', role: 'Support Caster', hp: 30, maxHp: 30, guard: 10, poise: 3, maxPoise: 3, x: 8, y: 2, intent: 'support', intentName: 'Brand of the Heretic (B2-B4 · Vulnerable)', passive: 'Choir Link: Restores 1 Poise to Nemeth each round.' }
    ]
  }
};

// Live mutable battle units & per-hero 6-block plans
let BATTLE = null;

function initBattle(encId) {
  STATE.encId = encId;
  const sel = document.getElementById('enc-select');
  if (sel) sel.value = encId;

  const tpl = ENCOUNTERS[encId] || ENCOUNTERS.enc_0c;
  const units = [];

  tpl.heroes.forEach((hPos, idx) => {
    const h = STATE.roster[hPos.id];
    // Pre-populate an initial 6-block tactical plan so the strip looks rich immediately
    const defaultPlan = idx === 0
      ? [
          { skillId: 'move', start: 0, blocks: 1, ap: 1, targetText: 'Step (4,4)', ghost: false },
          { skillId: h.actives[0], start: 1, blocks: SKILLS[h.actives[0]].blocks, ap: SKILLS[h.actives[0]].ap, targetText: tpl.foes[0].name, ghost: false },
          { skillId: h.basics[5], start: 1 + SKILLS[h.actives[0]].blocks, blocks: SKILLS[h.basics[5]].blocks, ap: SKILLS[h.basics[5]].ap, targetText: tpl.foes[0].name, ghost: true }
        ]
      : [
          { skillId: h.actives[0], start: 0, blocks: SKILLS[h.actives[0]].blocks, ap: SKILLS[h.actives[0]].ap, targetText: tpl.foes[0].name, ghost: false },
          { skillId: 'active_guard', start: SKILLS[h.actives[0]].blocks, blocks: 2, ap: 1, targetText: 'Self', ghost: true }
        ];

    units.push({
      id: h.id,
      isHero: true,
      unitKey: h.id,
      portrait: h.id,
      name: h.name,
      role: `${h.classId.toUpperCase()} · ${h.roleName}`,
      hp: h.hp,
      maxHp: h.maxHp,
      guard: h.guard,
      poise: h.poise,
      maxPoise: h.maxPoise,
      ap: h.ap,
      maxAp: 12,
      meterName: h.meterName,
      meter: h.meter,
      x: hPos.x,
      y: hPos.y,
      statuses: idx === 0 ? ['focus'] : [],
      plan: defaultPlan
    });
  });

  tpl.foes.forEach((f, idx) => {
    units.push({
      id: f.id,
      isHero: false,
      unitKey: f.unitKey,
      portrait: f.portrait,
      name: f.name,
      role: f.role,
      hp: f.hp,
      maxHp: f.maxHp,
      guard: f.guard,
      poise: f.poise,
      maxPoise: f.maxPoise,
      ap: 6,
      maxAp: 12,
      x: f.x,
      y: f.y,
      intent: f.intent,
      intentName: f.intentName,
      passive: f.passive,
      statuses: idx === 0 ? ['shielded'] : [],
      plan: [
        { skillId: 'move', start: 0, blocks: 1, ap: 1, targetText: 'Advance' },
        { skillId: 'strike', start: 1, blocks: 3, ap: 3, targetText: f.intentName }
      ]
    });
  });

  BATTLE = {
    encId,
    tpl,
    units,
    winchProgress: 0
  };
  STATE.selectedUnitId = units[0].id;
  STATE.pinnedFoeId = tpl.foes[0].id;
  STATE.skillMenu = null;
  STATE.activeSkillId = null;
  STATE.round = 1;
}

initBattle('enc_0c');

// ============================================================================
// DIALOGUE TREES (`talk.rs` — Sister Aline, Bram, Tess, Dry Hollows, Corvo)
// ============================================================================
const DIALOGUES = {
  aline: {
    name: 'Sister Aline',
    band: 'Order of the Ashen Leech · Apothecary',
    portrait: 'aline',
    nodes: {
      root: {
        text: '"Keep your boots out of the runoff trough, Kaelen. The Castellan\'s Purifiers dumped quicklime in the Third Ward sluice at dawn. Half the debtors coming down the steps have blistered lungs—and my last crate of coagulant is locked behind Sergeant Corvo\'s toll gate."',
        replies: [
          { label: '1. [Trade] Browse Apothecary Draughts & Surgical Supplies.', action: () => openShop('aline') },
          { label: '2. [Roll: KNOW >= 12] Inspect her quicklime burns & synthesize a neutralizing wash.', action: () => runSkillRoll('Knowledge', 13, 12, 'aline_roll_ok', 'aline_roll_fail') },
          { label: '3. "Tell me about the Red Tithing countdown."', next: 'tithing' },
          { label: '4. Step back from the operating table.', action: () => { STATE.talkNpcId = null; render(); } }
        ]
      },
      tithing: {
        text: '"Eighteen days until Prelate Nemeth opens the Great Sluice and flushes the Undercroft into the river. Every raid you run burns clock—and if the Hearth goes cold for lack of Tallow, wounds won\'t knit between sorties."',
        replies: [
          { label: '1. [Trade] Open Apothecary Shop.', action: () => openShop('aline') },
          { label: '2. Back to previous topics.', next: 'root' }
        ]
      },
      aline_roll_ok: {
        text: '"Look at that hand—steady as the old Ward College days. Using sulfur-ash to precipitates the lime out of the tallow... Take this Coagulant Vial and 15 Scrap from the infirmary coffer, Surgeon."',
        replies: [
          { label: '1. Pocket the 15 Scrap & open Apothecary Shop.', action: () => { STATE.scrap += 15; openShop('aline'); } },
          { label: '2. Return to the Undercroft.', next: 'root' }
        ]
      },
      aline_roll_fail: {
        text: '"Careful! The mixture boils over the brass basin. Leave the alchemy to Maren until your hands stop shaking, Kaelen."',
        replies: [
          { label: '1. Return.', next: 'root' }
        ]
      }
    }
  },
  bram: {
    name: 'Bram the Stoker',
    band: 'Slag-Heap Anvil · Quartermaster & Smith',
    portrait: 'bram',
    nodes: {
      root: {
        text: '"Brought more bent Castellan iron down the steps? Good. Their halberds are soft-core trash, but once I quench \'em in heavy tallow they\'ll split a Bailiff\'s gorget in two. Watch your belt pouches too—five pips is all a rigging harness holds before it drags you under."',
        replies: [
          { label: '1. [Forge & Trade] Reforge Weapons, Armor & Belt Pouches.', action: () => openShop('bram') },
          { label: '2. [Squad & Gear] Open Character & 5-Pip Belt Loadouts [P].', action: () => { STATE.modal = 'party'; render(); } },
          { label: '3. "What\'s waiting for us at the Fourth Kiln?"', next: 'kiln_intel' }
        ]
      },
      kiln_intel: {
        text: '"Foreman Grendel wired the floor grates to the main blast furnace. Every second round, those Slag Vents blow white-hot. Use Kaelen\'s Billhook Drag or Dunstan\'s Cinder Shove to haul their own Purifiers onto the grates before they vent!"',
        replies: [
          { label: '1. [Forge & Trade] Open Slag-Heap Anvil.', action: () => openShop('bram') },
          { label: '2. Back.', next: 'root' }
        ]
      }
    }
  },
  tess: {
    name: 'Night-Runner Tess',
    band: 'Sunk Ward Informant · Notice Board Keeper',
    portrait: 'tess',
    nodes: {
      root: {
        text: '"Three runners went up the chimney-ladders at dusk; only Vesper came back dry. I\'ve pinned the active Ward contracts to the sluice pillar. Watch the red ink—when a contract\'s deadline runs out, the Watch seals that block for good."',
        replies: [
          { label: '1. [Notice Board] Inspect Sunk Ward Contracts & Deadlines.', action: () => { STATE.modal = 'board'; render(); } },
          { label: '2. [District Map] Ascend the Sluice Steps to the Sunk Ward Map.', action: () => { STATE.screen = 'worldmap'; render(); } },
          { label: '3. "How high is Watch Alert right now?"', next: 'alert_info' }
        ]
      },
      alert_info: {
        text: '"Alert Tier 1—Stirred. Travel in Stealth mode on the District Map if you want to avoid street ambushes, though it takes an extra day compared to Fast marching."',
        replies: [
          { label: '1. [Notice Board] View Contracts.', action: () => { STATE.modal = 'board'; render(); } },
          { label: '2. Back.', next: 'root' }
        ]
      }
    }
  },
  bunks: {
    name: 'The Dry Hollows (6 Bunks)',
    band: 'Sanctuary Quarters · Rest & Active 4-of-6 Squad',
    portrait: 'dunstan',
    nodes: {
      root: {
        text: 'Six stone alcoves carved above the high-water mark, warmed by the central Tallow Hearth. Four Founders march in the active vanguard; two rest here to mend their Wounded status between sorties.',
        replies: [
          { label: '1. [Rest at Hearth] Spend 1 Tallow & Advance 1 Day (Full HP + Clear Wounded on Reserves).', action: () => restAtHearth() },
          { label: '2. [Manage Roster] Swap Active 4-of-6 Founders & Inspect Stats [P].', action: () => { STATE.modal = 'party'; render(); } },
          { label: '3. Step away from the bunks.', next: 'root' }
        ]
      }
    }
  },
  corvo: {
    name: 'Gate-Sergeant Corvo',
    band: 'Castellan Watch · Third Ward Toll-Keeper',
    portrait: 'bailiff',
    nodes: {
      root: {
        text: '"Halt at the chain! By order of Proctor Quell and the Tallow Guild, the Third Ward Sluice is under quarantine tithe. Thirty Scrap on the drumhead, or show a signed Guild Seal—else my arbalests pin your coats to the brickwork!"',
        replies: [
          { label: '1. [Pay Toll: -30 Scrap] Toss thirty Scrap onto the iron drumhead (Bypass fight, Alert +0).', action: () => resolveCorvoToll() },
          { label: '2. [Roll: KNOW >= 12] Present a forged Tallow Guild wax seal (Kaelen KNOW +3).', action: () => runSkillRoll('Knowledge', 13, 12, 'corvo_seal_ok', 'corvo_seal_fail') },
          { label: '3. [Draw Steel] "Take your toll in cold iron, Corvo." (Launch M2: Corvo\'s Gate Battle).', action: () => launchEncounter('enc_m2') }
        ]
      },
      corvo_seal_ok: {
        text: 'Corvo squints at the tallow-stamped parchment in the rain, then spits into the canal. "Guild Master Orlov\'s mark, right enough. Raise the grate, boys! Don\'t linger in the Ash-Crawl." (+25 XP, Gate Unlocked without bloodshed!)',
        replies: [
          { label: '1. Proceed to the Sunk Ward District Map (Fourth Kiln unlocked).', action: () => { markQuestDone('q_corvo'); STATE.screen = 'worldmap'; render(); } }
        ]
      },
      corvo_seal_fail: {
        text: '"Wait a breath—this wax is still warm from a surgeon\'s lamp! Crossbows, load! Gut the counterfeiters!"',
        replies: [
          { label: '1. [Ambush!] Enter Battle at Corvo\'s Gate (M2).', action: () => launchEncounter('enc_m2') }
        ]
      }
    }
  }
};

// ============================================================================
// STORY STAGE SEQUENCES (`story.rs` + `vignette.rs` — Decision A2 Chained Prologue)
// Supports both Approach 1 (VN Artwork + Dynamic Actor Stage) and Approach 2 (In-Game Map Stage + Bottom Dialogue)
// ============================================================================
const STORIES = {
  prologue_intro: {
    title: 'Prologue 0a · A Scarlet Lullaby',
    actBanner: 'ACT I · THE SUNK WARD',
    subtitle: 'Seventh Sluice Tenements · Rain & Tallow-Smoke',
    bg: 'vn_alley_stage',
    vignetteBg: 'intro_1',
    mapBackdrop: 'alley',
    encId: 'enc_0a',
    cards: [
      {
        vignetteStyle: 'Title',
        speaker: 'Kaelen',
        speakerTitle: 'Disgraced Surgeon · Seventh Sluice Dispensary',
        portrait: 'kaelen',
        mood: 'neutral',
        emotion: 'Somber Resolve',
        bgFilter: 'brightness(0.96) contrast(1.04)',
        bgVariantLabel: 'Stage A · Wet Cobblestone Alley & Distant Foundry Glow',
        shake: null,
        stageAction: 'Camera frames the rain-slicked alley cobblestones outside Kaelen\'s tenement surgery.',
        leftActor: { unitKey: 'kaelen', portrait: 'kaelen', name: 'Kaelen', title: 'Disgraced Surgeon', active: true, emotion: 'Grim Resolve', reactionFx: 'solemn_glow', anim: 'breathe' },
        rightActor: null,
        mapUnits: [
          { id: 'kaelen', unitKey: 'kaelen', name: 'Kaelen', isHero: true, x: 2, y: 4, speaking: true, emote: '...' },
          { id: 'w1', unitKey: 'watchman', name: 'Watch Halberdier', isHero: false, x: 7, y: 4, speaking: false, emote: null },
          { id: 'w2', unitKey: 'crossbow', name: 'Watch Arbalest', isHero: false, x: 9, y: 3, speaking: false, emote: null }
        ],
        text: 'The rain in the Sunk Ward never washes anything clean; it only carries the tallow-smoke down from the high foundries onto the flooded cobblestones where the debtors sleep.'
      },
      {
        vignetteStyle: 'Image',
        speaker: 'Watch Halberdier',
        speakerTitle: 'Castellan Watch · Seventh Sluice Patrol',
        portrait: 'watchman',
        mood: 'angry',
        emotion: 'Hostile Writ',
        bgFilter: 'brightness(1.02) contrast(1.08) sepia(0.08)',
        bgVariantLabel: 'Stage B · Lantern Flare on the Alley Stones (Shake: Light)',
        shake: 'Light Shake (FX1)',
        stageAction: 'Iron boots splinter the surgery door; a Castellan Watchman plants his halberd on the wet flagstones.',
        leftActor: { unitKey: 'kaelen', portrait: 'kaelen', name: 'Kaelen', title: 'Disgraced Surgeon', active: false, emotion: 'Alert', reactionFx: 'alert_shock', anim: 'breathe' },
        rightActor: { unitKey: 'watchman', portrait: 'watchman', name: 'Watch Halberdier', title: 'Castellan Enforcer', active: true, emotion: 'Barking Orders', reactionFx: 'anger_burst', anim: 'shake' },
        mapUnits: [
          { id: 'kaelen', unitKey: 'kaelen', name: 'Kaelen', isHero: true, x: 2, y: 4, speaking: false, emote: '!' },
          { id: 'w1', unitKey: 'watchman', name: 'Watch Halberdier', isHero: false, x: 6, y: 4, speaking: true, emote: '⚔' },
          { id: 'w2', unitKey: 'crossbow', name: 'Watch Arbalest', isHero: false, x: 9, y: 3, speaking: false, emote: '🎯' }
        ],
        text: '"Surgeon Kaelen! By writ of Captain Vane, your dispensary is forfeit for refusing to sign the Red Tithing certificates. Lay down your instruments and present your wrists for the iron!"'
      },
      {
        vignetteStyle: 'Image',
        speaker: 'Kaelen',
        speakerTitle: 'Disgraced Surgeon · Vanguard Founder',
        portrait: 'kaelen',
        mood: 'determined',
        emotion: 'Steel Drawn',
        bgFilter: 'brightness(1.06) contrast(1.12)',
        bgVariantLabel: 'Stage C · Cold Steel Glint & Tallow Lamp Rim',
        shake: 'Medium Pulse (FX1)',
        stageAction: 'Kaelen lifts his Surgeon\'s Billhook and lunges forward across the wet stone floor.',
        leftActor: { unitKey: 'kaelen', portrait: 'kaelen', name: 'Kaelen', title: 'Disgraced Surgeon', active: true, emotion: 'Steel Drawn', lunge: true, reactionFx: 'steel_flash', anim: 'lunge' },
        rightActor: { unitKey: 'watchman', portrait: 'watchman', name: 'Watch Halberdier', title: 'Castellan Enforcer', active: false, emotion: 'Braced', reactionFx: 'alert_shock', anim: 'breathe' },
        mapUnits: [
          { id: 'kaelen', unitKey: 'kaelen', name: 'Kaelen', isHero: true, x: 3, y: 4, speaking: true, emote: '⚔' },
          { id: 'w1', unitKey: 'watchman', name: 'Watch Halberdier', isHero: false, x: 6, y: 4, speaking: false, emote: '🛡' },
          { id: 'w2', unitKey: 'crossbow', name: 'Watch Arbalest', isHero: false, x: 9, y: 3, speaking: false, emote: null }
        ],
        text: '"You brought shackles to an operating room, halberdier. Come closer—let us see how Castellan mail holds against a bone-saw and a billhook."'
      }
    ],
    nextLabel: 'Transition to Combat: Prologue 0a — The Alley',
    onFinish: () => launchEncounterWithTransition('enc_0a')
  },
  prologue_0b: {
    title: 'Prologue 0b · The Chimney Slate',
    actBanner: 'PROLOGUE 0B · SLATE RIDGE',
    subtitle: 'Above the Chained Tenements · Brazier Light',
    bg: 'vn_alley_stage',
    vignetteBg: 'intro_2',
    mapBackdrop: 'rooftops',
    encId: 'enc_0b',
    cards: [
      {
        vignetteStyle: 'Image',
        speaker: 'Vesper',
        speakerTitle: 'Cutthroat Scout · Flanker Founder',
        portrait: 'vesper',
        mood: 'determined',
        emotion: 'Urgent Whisper',
        bgFilter: 'brightness(0.98) contrast(1.06)',
        bgVariantLabel: 'Stage A · Slick Slate Ledge & Brazier Glare',
        shake: null,
        stageAction: 'Vesper drops lightly onto the wet stone ledge and signals down to Kaelen.',
        leftActor: { unitKey: 'vesper', portrait: 'vesper', name: 'Vesper', title: 'Cutthroat Scout', active: true, emotion: 'Smirking', reactionFx: 'alert_shock', anim: 'hop' },
        rightActor: { unitKey: 'kaelen', portrait: 'kaelen', name: 'Kaelen', title: 'Disgraced Surgeon', active: false, emotion: 'Alert', reactionFx: 'solemn_glow', anim: 'breathe' },
        mapUnits: [
          { id: 'vesper', unitKey: 'vesper', name: 'Vesper', isHero: true, x: 2, y: 2, speaking: true, emote: '💬' },
          { id: 'kaelen', unitKey: 'kaelen', name: 'Kaelen', isHero: true, x: 2, y: 5, speaking: false, emote: null },
          { id: 'bailiff', unitKey: 'bailiff', name: 'Castellan Bailiff', isHero: false, x: 6, y: 5, speaking: false, emote: '!' },
          { id: 'xbow1', unitKey: 'crossbow', name: 'Slate Marksman', isHero: false, x: 9, y: 2, speaking: false, emote: null }
        ],
        text: '"Down here, Doc! The Watch has the lower street chained off, and a Castellan Bailiff is mustering crossbowmen along the slate ridge. Give me a boost and we\'ll flank them before they lock the windlass."'
      },
      {
        vignetteStyle: 'Image',
        speaker: 'Castellan Bailiff',
        speakerTitle: 'Elite Brute · Ridge Warden',
        portrait: 'bailiff',
        mood: 'angry',
        emotion: 'Enraged Roar',
        bgFilter: 'brightness(1.05) contrast(1.12) saturate(1.12)',
        bgVariantLabel: 'Stage B · Tallow Brazier Ignited on the Ridge',
        shake: 'Light Shake (FX1)',
        stageAction: 'The Castellan Bailiff slams his spiked club against his shield beside the burning Tallow Brazier.',
        leftActor: { unitKey: 'vesper', portrait: 'vesper', name: 'Vesper', title: 'Cutthroat Scout', active: false, emotion: 'Poised', reactionFx: 'steel_flash', anim: 'breathe' },
        rightActor: { unitKey: 'bailiff', portrait: 'bailiff', name: 'Castellan Bailiff', title: 'Ridge Warden', active: true, emotion: 'Roaring', lunge: true, reactionFx: 'anger_burst', anim: 'lunge' },
        mapUnits: [
          { id: 'vesper', unitKey: 'vesper', name: 'Vesper', isHero: true, x: 3, y: 2, speaking: false, emote: '🗡' },
          { id: 'kaelen', unitKey: 'kaelen', name: 'Kaelen', isHero: true, x: 3, y: 5, speaking: false, emote: '⚔' },
          { id: 'bailiff', unitKey: 'bailiff', name: 'Castellan Bailiff', isHero: false, x: 6, y: 5, speaking: true, emote: '💢' },
          { id: 'xbow1', unitKey: 'crossbow', name: 'Slate Marksman', isHero: false, x: 9, y: 2, speaking: false, emote: '🎯' }
        ],
        text: '"Runners on the wet slate! Marksmen, pin their cloaks to the chimney bricks—kick the brazier over if they try to cross the ridge!"'
      }
    ],
    nextLabel: 'Transition to Combat: Prologue 0b — The Rooftop',
    onFinish: () => launchEncounterWithTransition('enc_0b')
  },
  prologue_0c: {
    title: 'Prologue 0c · The Sluice Bridge',
    actBanner: 'PROLOGUE 0C · SEVENTH SLUICE BRIDGE',
    subtitle: 'Spillway Cordon · Full 4-Founder Vanguard',
    bg: 'vn_bridge_stage',
    vignetteBg: 'meat_market',
    mapBackdrop: 'canal',
    encId: 'enc_0c',
    cards: [
      {
        vignetteStyle: 'Image',
        speaker: 'Maren',
        speakerTitle: 'Plague Alchemist · Support / DoT Founder',
        portrait: 'maren',
        mood: 'determined',
        emotion: 'Alchemical Focus',
        bgFilter: 'brightness(0.98) contrast(1.06)',
        bgVariantLabel: 'Stage A · Flooded Bridge Flagstones & Alchemical Vapour',
        shake: null,
        stageAction: 'Maren and Hollis emerge onto the wide stone spillway bridge as green alchemical vapour curls from Maren\'s censer.',
        leftActor: { unitKey: 'maren', portrait: 'maren', name: 'Maren', title: 'Plague Alchemist', active: true, emotion: 'Flask Ready', reactionFx: 'alchemy_flare', anim: 'hop' },
        rightActor: { unitKey: 'vane', portrait: 'vane', name: 'Captain Vane', title: 'Watch Commander', active: false, emotion: 'Cold Glare', reactionFx: 'solemn_glow', anim: 'breathe' },
        mapUnits: [
          { id: 'kaelen', unitKey: 'kaelen', name: 'Kaelen', isHero: true, x: 3, y: 4, speaking: false, emote: null },
          { id: 'vesper', unitKey: 'vesper', name: 'Vesper', isHero: true, x: 3, y: 2, speaking: false, emote: null },
          { id: 'maren', unitKey: 'maren', name: 'Maren', isHero: true, x: 1, y: 5, speaking: true, emote: '⚗' },
          { id: 'hollis', unitKey: 'hollis', name: 'Hollis', isHero: true, x: 1, y: 3, speaking: false, emote: '🏹' },
          { id: 'vane', unitKey: 'vane', name: 'Captain Vane', isHero: false, x: 8, y: 4, speaking: false, emote: '!' }
        ],
        text: '"Kaelen! Hollis and I blew the lock on the Undercroft grate, but Captain Vane is standing right on the spillway bridge. Break his line now—before the tide rises!"'
      },
      {
        vignetteStyle: 'Image',
        speaker: 'Captain Vane',
        speakerTitle: 'Boss · Castellan Watch Commander',
        portrait: 'vane',
        mood: 'angry',
        emotion: 'Ruthless Command',
        bgFilter: 'brightness(1.06) contrast(1.14)',
        bgVariantLabel: 'Stage B · Vane\'s Halberd Planted on the Bridge Stones (Shake: Heavy)',
        shake: 'Heavy Shake (FX1)',
        stageAction: 'Captain Vane slams his gilded halberd into the wet bridge flagstones at (8,4); crimson sparks erupt around his boots.',
        leftActor: { unitKey: 'kaelen', portrait: 'kaelen', name: 'Kaelen', title: 'Disgraced Surgeon', active: false, emotion: 'Defiant', reactionFx: 'steel_flash', anim: 'breathe' },
        rightActor: { unitKey: 'vane', portrait: 'vane', name: 'Captain Vane', title: 'Watch Commander', active: true, emotion: 'Executioner\'s Stance', lunge: true, reactionFx: 'anger_burst', anim: 'lunge' },
        mapUnits: [
          { id: 'kaelen', unitKey: 'kaelen', name: 'Kaelen', isHero: true, x: 3, y: 4, speaking: false, emote: '⚔' },
          { id: 'vesper', unitKey: 'vesper', name: 'Vesper', isHero: true, x: 3, y: 2, speaking: false, emote: null },
          { id: 'maren', unitKey: 'maren', name: 'Maren', isHero: true, x: 1, y: 5, speaking: false, emote: null },
          { id: 'hollis', unitKey: 'hollis', name: 'Hollis', isHero: true, x: 1, y: 3, speaking: false, emote: null },
          { id: 'vane', unitKey: 'vane', name: 'Captain Vane', isHero: false, x: 8, y: 4, speaking: true, emote: '💢' },
          { id: 'w1', unitKey: 'watchman', name: 'Bridge Halberdier', isHero: false, x: 7, y: 3, speaking: false, emote: '🛡' }
        ],
        text: '"A disgraced sawbones, a gutter cutthroat, and two deserters. Hold the spillway, Watchmen! The river takes them tonight!"'
      }
    ],
    nextLabel: 'Transition to Combat: Prologue 0c — Canal Bridge',
    onFinish: () => launchEncounterWithTransition('enc_0c')
  },
  hearth_arrival: {
    title: 'Sanctuary · The Sluice Undercroft',
    actBanner: 'SANCTUARY · THE TALLOW HEARTH',
    subtitle: 'Below the Seventh Sluice · 18 Days to the Red Tithing',
    bg: 'vn_hearth_stage',
    vignetteBg: 'hearth',
    mapBackdrop: 'sluice_hub',
    encId: 'enc_0c',
    cards: [
      {
        vignetteStyle: 'Image',
        speaker: 'Sister Aline',
        speakerTitle: 'Order of the Ashen Leech · Sanctuary Apothecary',
        portrait: 'aline',
        mood: 'determined',
        emotion: 'Solemn Welcome',
        bgFilter: 'brightness(1.02) contrast(1.05)',
        bgVariantLabel: 'Stage A · Warm Tallow Hearth & Dry Sanctuary Flagstones',
        shake: null,
        stageAction: 'The iron grate bars shut behind the squad; Sister Aline steps across the dry sanctuary flagstones beside the Tallow Hearth.',
        leftActor: { unitKey: 'kaelen', portrait: 'kaelen', name: 'Kaelen', title: 'Disgraced Surgeon', active: false, emotion: 'Weary', reactionFx: 'solemn_glow', anim: 'breathe' },
        rightActor: { unitKey: 'aline', portrait: 'aline', name: 'Sister Aline', title: 'Sanctuary Apothecary', active: true, emotion: 'Solemn Welcome', reactionFx: 'solemn_glow', anim: 'hop' },
        mapUnits: [
          { id: 'kaelen', unitKey: 'kaelen', name: 'Kaelen', isHero: true, x: 6, y: 5, speaking: false, emote: null },
          { id: 'aline', unitKey: 'aline', name: 'Sister Aline', isHero: true, x: 4, y: 4, speaking: true, emote: '🔥' },
          { id: 'bram', unitKey: 'bram', name: 'Bram the Stoker', isHero: true, x: 10, y: 4, speaking: false, emote: '⚒' },
          { id: 'tess', unitKey: 'tess', name: 'Night-Runner Tess', isHero: true, x: 8, y: 3, speaking: false, emote: '📜' }
        ],
        text: '"Bar the iron grate and stir the Tallow Hearth. You\'re below the Castellan\'s sightlines now, Kaelen—but Prelate Nemeth has decreed the Red Tithing in eighteen days. If we don\'t break the Fourth Kiln and seize the Crucible Winch, the entire Sunk Ward drowns."'
      }
    ],
    nextLabel: 'Enter Hub: The Sluice Undercroft (RegionMap)',
    onFinish: () => {
      STATE.screen = 'region';
      STATE.regionId = 'sluice_undercroft';
      STATE.talkNpcId = 'aline';
      STATE.talkNodeId = 'root';
      render();
    }
  }
};

// ============================================================================
// IN-COMBAT SCRIPTED DIALOGUE & MAP→COMBAT TRANSITION BARKS
// Covers:
//  1. `pre_battle`      : Pre-Battle Map Confrontation (Units on tactical map + bottom dialogue box before Combat HUD slides in)
//  2. `boss_phase2`     : Mid-Combat Boss Phase 2 Transformation (50% HP threshold bark + aura ring + passive activation)
//  3. `elite_intro`     : Before Encountering / Engaging the Elite Unit
//  4. `elite_aftermath` : After Breaking / Defeating the Elite Unit
// ============================================================================
const COMBAT_BARKS = {
  enc_0a: {
    pre_battle: {
      badge: 'PRE-COMBAT MAP CONFRONTATION',
      speaker: 'Watch Halberdier',
      speakerTitle: 'Castellan Watch · Seventh Sluice Patrol',
      portrait: 'watchman',
      mood: 'angry',
      focusUnitId: 'f_watch1',
      mechanicHint: 'Map → Combat Transition: Units are already in position on the 12×9 Alley grid. Click [Engage Combat] to slide in the Foresight Timeline & Action HUD.',
      text: '"Crossbowman, cover the rain-trough! If the sawbones raises that billhook, pin him to his own surgery door!"'
    },
    boss_phase2: {
      badge: 'IN-COMBAT SCRIPTED EVENT · LAST STAND',
      speaker: 'Watch Halberdier',
      speakerTitle: 'Poise Shattered · Desperate Thrust',
      portrait: 'watchman',
      mood: 'hurt',
      focusUnitId: 'f_watch1',
      mechanicHint: 'Phase Trigger (≤ 50% HP): Halberdier shifts to Desperate Thrust (+3 Damage, -4 Guard).',
      text: '"Gah—he hooked the rim of my gorget! Arbalest, reload faster—don\'t let him close the distance!"'
    },
    elite_intro: {
      badge: 'ELITE ENCOUNTER · BEFORE ENGAGING',
      speaker: 'Kaelen',
      speakerTitle: 'Disgraced Surgeon · Tactical Assessment',
      portrait: 'kaelen',
      mood: 'determined',
      focusUnitId: 'f_watch1',
      mechanicHint: 'Elite Trait Preview: Shield Wall grants +4 Guard while adjacent to an ally. Use Billhook Drag to separate them.',
      text: '"His halberdier mail is thick at the shoulder, but his stance is top-heavy on the wet cobbles. Two Poise strikes will break his guard wide open."'
    },
    elite_aftermath: {
      badge: 'ELITE ENCOUNTER · AFTER BREAKING ELITE',
      speaker: 'Kaelen',
      speakerTitle: 'Disgraced Surgeon · Alley Cleared',
      portrait: 'kaelen',
      mood: 'neutral',
      focusUnitId: 'kaelen',
      mechanicHint: 'Patrol Broken: Path to the Seventh Sluice chimney ladder is clear.',
      text: '"Stay down in the runoff trough. Your brass writ won\'t staunch a femoral bleed—and I have no bandages to spare for Captain Vane\'s hounds."'
    }
  },
  enc_0b: {
    pre_battle: {
      badge: 'PRE-COMBAT MAP CONFRONTATION',
      speaker: 'Castellan Bailiff',
      speakerTitle: 'Elite Brute · Ridge Warden',
      portrait: 'bailiff',
      mood: 'angry',
      focusUnitId: 'f_bailiff',
      mechanicHint: 'Map → Combat Transition: Camera locks onto the 12×9 Rooftop grid before sliding in the Combat HUD.',
      text: '"Hold the chimney ridge! Whoever brings me the surgeon\'s billhook gets a double ration of tallow and silver!"'
    },
    boss_phase2: {
      badge: 'BOSS / ELITE PHASE TRANSFORMATION (50% HP)',
      speaker: 'Castellan Bailiff',
      speakerTitle: 'Iron Collar Unlatched · Brazier Frenzy',
      portrait: 'bailiff',
      mood: 'hurt',
      focusUnitId: 'f_bailiff',
      mechanicHint: 'Phase 2 Trigger: Bailiff ignites his mace in the Tallow Brazier at (6,4) — Mace Sweep now inflicts +2 Burn!',
      text: '"Bloody gutter-rats! If cold iron won\'t break you, let\'s see how you like burning tallow across the slate!"'
    },
    elite_intro: {
      badge: 'ELITE ENCOUNTER · BEFORE ENGAGING BAILIFF',
      speaker: 'Vesper',
      speakerTitle: 'Cutthroat Scout · Flanker Founder',
      portrait: 'vesper',
      mood: 'determined',
      focusUnitId: 'f_bailiff',
      mechanicHint: 'Elite Trait [Iron Collar]: Immune to Pull while Guard > 0. Strip his Guard with Kaelen or kick him into the Brazier!',
      text: '"Careful with the big ugly one in the iron gorget, Doc! You can\'t hook-drag him while his Guard holds—let me slip behind him with Shadow Step first."'
    },
    elite_aftermath: {
      badge: 'ELITE ENCOUNTER · AFTER DEFEATING BAILIFF',
      speaker: 'Vesper',
      speakerTitle: 'Cutthroat Scout · Ridge Secured',
      portrait: 'vesper',
      mood: 'happy',
      focusUnitId: 'vesper',
      mechanicHint: 'Elite Defeated: Remaining Marksmen lose morale (-1 Poise) and drop the Sluice Bridge key.',
      text: '"Down goes the Bailiff—and look, his belt ring has the brass windlass key for the Sluice Bridge! Move it, Kaelen!"'
    }
  },
  enc_0c: {
    pre_battle: {
      badge: 'PRE-COMBAT MAP CONFRONTATION',
      speaker: 'Captain Vane',
      speakerTitle: 'Boss · Castellan Watch Commander',
      portrait: 'vane',
      mood: 'angry',
      focusUnitId: 'f_vane',
      mechanicHint: 'Map → Combat Transition: Confrontation on the live 12×9 Canal Bridge map; clicking [Engage Combat] slides in the Top Timeline & Bottom Action HUD.',
      text: '"Lock shields across the spillway grate! Neither the surgeon nor his alchemist crosses into the Undercroft alive!"'
    },
    boss_phase2: {
      badge: '⚡ BOSS PHASE 2 TRANSFORMATION · 50% HP THRESHOLD',
      speaker: 'Captain Vane',
      speakerTitle: 'Phase 2: "No Quarter" · Crimson Overdrive',
      portrait: 'vane',
      mood: 'hurt',
      focusUnitId: 'f_vane',
      mechanicHint: 'Boss Phase 2 Activated: Captain Vane gains +4 Might, refreshes +12 Guard, and queues Executioner\'s Cleave (3-tile Arc, 18 dmg)!',
      text: '"You dare scar Castellan plate?! Sound the red horn! No prisoners—I will carve the Red Tithing out of your hides myself!"'
    },
    elite_intro: {
      badge: 'ELITE ENCOUNTER · BEFORE ENGAGING PHALANX GUARD',
      speaker: 'Maren',
      speakerTitle: 'Plague Alchemist · Support / DoT Founder',
      portrait: 'maren',
      mood: 'determined',
      focusUnitId: 'f_w1',
      mechanicHint: 'Elite Escort [Interpose]: Bridge Halberdier absorbs direct strikes aimed at Captain Vane until displaced or broken.',
      text: '"Kaelen, don\'t waste a heavy swing on Vane while his Bridge Halberdier is braced beside him! My Volatile Tallow Flask will scorch both of them through the shield wall."'
    },
    elite_aftermath: {
      badge: 'ELITE ENCOUNTER · AFTER BREAKING ESCORT',
      speaker: 'Kaelen',
      speakerTitle: 'Disgraced Surgeon · Vanguard Founder',
      portrait: 'kaelen',
      mood: 'determined',
      focusUnitId: 'f_vane',
      mechanicHint: 'Escort Broken: Captain Vane\'s flank is exposed (+50% Flank Bonus active).',
      text: '"Your shield-bearer is down, Vane, and the Undercroft grate is open behind you. Yield the bridge or go into the canal!"'
    }
  },
  enc_m2: {
    pre_battle: {
      badge: 'PRE-COMBAT MAP CONFRONTATION · PARLEY ESCALATION',
      speaker: 'Gate-Sergeant Corvo',
      speakerTitle: 'Mini-Boss · Third Ward Toll Warden',
      portrait: 'bailiff',
      mood: 'angry',
      focusUnitId: 'f_corvo',
      mechanicHint: 'RegionMap → Combat UI Transition: Escalated directly from Corvo\'s Gate street parley into tactical combat!',
      text: '"No thirty Scrap? Then my Flayers take thirty ounces of flesh! Drop the portcullis chain and gut them in the mud!"'
    },
    boss_phase2: {
      badge: '⚡ MINI-BOSS PHASE 2 TRANSFORMATION · 50% HP',
      speaker: 'Gate-Sergeant Corvo',
      speakerTitle: 'Phase 2: "Extortionist\'s Fury" · Toll Bell Rung',
      portrait: 'bailiff',
      mood: 'hurt',
      focusUnitId: 'f_corvo',
      mechanicHint: 'Phase 2 Trigger: Corvo drinks a Purifier Tallow Draught (+10 Guard, Unstoppable for 1 Round).',
      text: '"Purifier! Throw the sanctified tallow on my maul! Nobody crosses the Third Ward gate without paying the butcher\'s bill!"'
    },
    elite_intro: {
      badge: 'ELITE ENCOUNTER · BEFORE ENGAGING PURIFIER',
      speaker: 'Kaelen',
      speakerTitle: 'Disgraced Surgeon · Vanguard Founder',
      portrait: 'kaelen',
      mood: 'determined',
      focusUnitId: 'f_purifier',
      mechanicHint: 'Elite Caster [Inquisitorial Purifier]: Grants +12 Guard & Burn immunity within 2 tiles. Focus-fire or silence early!',
      text: '"Watch the Inquisitorial Purifier behind the toll chain—as long as he swings that brass censer, Corvo\'s armor won\'t catch Maren\'s alchemical fire."'
    },
    elite_aftermath: {
      badge: 'ELITE ENCOUNTER · AFTER DEFEATING CORVO',
      speaker: 'Vesper',
      speakerTitle: 'Cutthroat Scout · Toll Coffer Seized',
      portrait: 'vesper',
      mood: 'happy',
      focusUnitId: 'vesper',
      mechanicHint: 'Elite Bounty Claimed: +45 Scrap recovered from Corvo\'s iron drumhead.',
      text: '"Forty-five Scrap in the toll drumhead AND Sister Aline\'s confiscated coagulant crate! Pleasure doing business with you, Sergeant Corvo."'
    }
  },
  enc_m3: {
    pre_battle: {
      badge: 'PRE-COMBAT MAP CONFRONTATION · FOURTH KILN',
      speaker: 'Foreman Grendel',
      speakerTitle: 'Boss · Bone-Mill Overseer',
      portrait: 'orlov',
      mood: 'angry',
      focusUnitId: 'f_grendel',
      mechanicHint: 'Map → Combat Transition: Slag Vents at (5,3), (5,5), and (7,4) prime for even-round eruption.',
      text: '"Intruders on the rendering floor! Open the main furnace dampers—let the Slag Vents roast them in their boots!"'
    },
    boss_phase2: {
      badge: '⚡ BOSS PHASE 2 TRANSFORMATION · OVERPRESSURE',
      speaker: 'Foreman Grendel',
      speakerTitle: 'Phase 2: "Kiln Overpressure" · All Vents Active',
      portrait: 'orlov',
      mood: 'angry',
      focusUnitId: 'f_grendel',
      mechanicHint: 'Boss Phase 2 Activated: Slag Vents now erupt EVERY round for 10 Burn damage & -2 Poise!',
      text: '"Chain the safety valves shut! If the Fourth Kiln goes down, the whole foundry goes to ash with it!"'
    },
    elite_intro: {
      badge: 'ELITE ENCOUNTER · BEFORE ENGAGING CALCIFIED HUSK',
      speaker: 'Maren',
      speakerTitle: 'Plague Alchemist · Support / DoT Founder',
      portrait: 'maren',
      mood: 'surprised',
      focusUnitId: 'f_husk',
      mechanicHint: 'Elite Brute [Calcified Husk]: 14 Guard, but Brittle Shell shatters all Guard immediately when Poise reaches 0.',
      text: '"Look at the bone-ash crust on that Husk—blade cuts will bounce off its shell, but a single Poise break will shatter its entire armor layer!"'
    },
    elite_aftermath: {
      badge: 'ELITE ENCOUNTER · AFTER SHATTERING HUSK',
      speaker: 'Kaelen',
      speakerTitle: 'Disgraced Surgeon · Vanguard Founder',
      portrait: 'kaelen',
      mood: 'determined',
      focusUnitId: 'kaelen',
      mechanicHint: 'Husk Shattered: Clear line of sight to hook Foreman Grendel onto the central Slag Vent at (7,4).',
      text: '"The Husk\'s shell is cracked. Now drag Grendel onto his own furnace grate before the next vent blows!"'
    }
  },
  enc_m5: {
    pre_battle: {
      badge: 'PRE-COMBAT MAP CONFRONTATION · CRUCIBLE CLIMAX',
      speaker: 'Prelate Nemeth',
      speakerTitle: 'Act I Boss · High Inquisitor of the Rubric',
      portrait: 'nemeth',
      mood: 'angry',
      focusUnitId: 'f_nemeth',
      mechanicHint: 'Map → Combat Transition: Great Sluice Winch objective active at tile (6,4).',
      text: '"So the disgraced surgeon climbs out of the sewer at last. Proctor Quell, hold the Great Sluice Winch—in three tolls of the bell, the Undercroft drowns!"'
    },
    boss_phase2: {
      badge: '🔥 ACT I BOSS PHASE 2 TRANSFORMATION · SCARLET CONSECRATION',
      speaker: 'Prelate Nemeth',
      speakerTitle: 'Phase 2: "Scarlet Consecration" · Pyres Ignited',
      portrait: 'nemeth',
      mood: 'angry',
      focusUnitId: 'f_nemeth',
      mechanicHint: 'Boss Phase 2 Activated (50% HP): Nemeth ignites all Sanctified Pyres, gains +2 AP per round, and unlocks Scarlet Edict (Cross AoE)!',
      text: '"By the Scarlet Rubric, I consecrate this sluice in tallow and flame! Burn the heretics from the winch-wheel!"'
    },
    elite_intro: {
      badge: 'ELITE ENCOUNTER · BEFORE ENGAGING PROCTOR QUELL',
      speaker: 'Kaelen',
      speakerTitle: 'Disgraced Surgeon · Vanguard Founder',
      portrait: 'kaelen',
      mood: 'determined',
      focusUnitId: 'f_quell',
      mechanicHint: 'Elite Sub-Boss [Proctor Quell]: Blocks interaction with the Sluice Winch at (6,4) while adjacent.',
      text: '"Proctor Quell has his tower shield planted right across the Sluice Winch gear. Displace him from tile (7,4) or break his Poise so Vesper can turn the wheel!"'
    },
    elite_aftermath: {
      badge: 'ELITE ENCOUNTER · AFTER BREAKING PROCTOR QUELL',
      speaker: 'Vesper',
      speakerTitle: 'Cutthroat Scout · Winch Unblocked',
      portrait: 'vesper',
      mood: 'determined',
      focusUnitId: 'vesper',
      mechanicHint: 'Proctor Quell Downed: The Great Sluice Winch at (6,4) is now unlocked for interaction!',
      text: '"Quell\'s down! The winch ratchet is clear, Kaelen—cover me for two rounds while I jam the floodgate gears!"'
    }
  }
};

// ============================================================================
// REGION MAPS (`region.rs` — Sluice Undercroft Hub & Corvo's Gate Street Map)
// ============================================================================
const REGIONS = {
  sluice_undercroft: {
    id: 'sluice_undercroft',
    title: 'The Sluice Undercroft',
    subtitle: 'Flooded Gothic Sanctuary · Tallow Hearth Lit',
    cols: 16,
    rows: 10,
    entities: [
      { id: 'aline', kind: 'npc', name: 'Sister Aline', role: 'Apothecary & Triage', x: 4, y: 4, unitKey: 'aline', badge: 'service_shop' },
      { id: 'bram',  kind: 'npc', name: 'Bram the Stoker', role: 'Slag-Heap Anvil', x: 11, y: 4, unitKey: 'bram', badge: 'service_shop' },
      { id: 'tess',  kind: 'npc', name: 'Night-Runner Tess', role: 'Notice Board & Intel', x: 8, y: 3, unitKey: 'tess', badge: 'badge_quest' },
      { id: 'bunks', kind: 'bunks', name: 'The Dry Hollows', role: '6 Bunks · Rest & Squad', x: 4, y: 7, unitKey: 'dunstan', badge: 'service_recruit' },
      { id: 'exit',  kind: 'exit', name: 'Sluice Steps (World Map)', role: 'Ascend to District Map', x: 13, y: 7, icon: 'marker_exit' }
    ]
  },
  corvos_gate: {
    id: 'corvos_gate',
    title: 'Ash-Crawl Crossroads · Corvo\'s Gate',
    subtitle: 'Decision A4: Dilemma/Parley Street Map before Combat Escalation',
    cols: 16,
    rows: 10,
    entities: [
      { id: 'corvo', kind: 'npc', name: 'Gate-Sergeant Corvo', role: 'Parley / Toll Dilemma', x: 9, y: 4, unitKey: 'bailiff', badge: 'badge_quest' },
      { id: 'ambush_enc', kind: 'encounter', name: 'Storm the Toll Gate', role: 'Combat: M2 Corvo\'s Gate', x: 11, y: 4, unitKey: 'flayer', badge: 'intent_attack' },
      { id: 'smuggler_chest', kind: 'chest', name: 'Submerged Smuggler Crate', role: 'Loot: +20 Scrap', x: 5, y: 7, icon: 'marker_chest' },
      { id: 'exit', kind: 'exit', name: 'Return to District Map', role: 'World Map', x: 2, y: 5, icon: 'marker_exit' }
    ]
  }
};

// ============================================================================
// WORLD MAP NODES (`worldmap.rs` — Sunk Ward District Map)
// ============================================================================
const WORLD_NODES = [
  {
    id: 'n_sluice', name: '00 · The Sluice Undercroft', kind: 'town', pin: 'pin_town',
    x: 115, y: 225, days: 0, mode: 'Hub',
    desc: 'Flooded vaulted sanctuary beneath the Seventh Sluice. Houses the Tallow Hearth, Sister Aline\'s Apothecary, Bram\'s Forge, Tess\'s Notice Board, and The Dry Hollows (6 Bunks).',
    edges: ['n_grate', 'n_corvo'],
    onEnter: () => { STATE.screen = 'region'; STATE.regionId = 'sluice_undercroft'; STATE.talkNpcId = 'aline'; STATE.talkNodeId = 'root'; render(); }
  },
  {
    id: 'n_grate', name: '01 · Drowned Runoff Grate', kind: 'ruin', pin: 'pin_ruin',
    x: 195, y: 265, days: 1, mode: 'Raid (Direct Battle)',
    desc: 'Sunken drainage basin choked with feral Render-Beasts and Ash-Crawlers. Pure combat raid node (launches BattleScreen directly per Decision A4).',
    edges: ['n_sluice', 'n_corvo'],
    quest: true,
    onEnter: () => launchEncounter('enc_0c')
  },
  {
    id: 'n_corvo', name: '02 · Third Ward Gate (Corvo\'s Toll)', kind: 'gate', pin: 'pin_gate',
    x: 245, y: 170, days: 1, mode: 'Parley RegionMap + Battle',
    desc: 'Fortified toll portcullis held by Gate-Sergeant Corvo. Entering opens the Ash-Crawl street RegionMap for parley, stat-check forgery, or combat (Decision A4).',
    edges: ['n_sluice', 'n_grate', 'n_cross', 'n_kiln'],
    quest: true,
    onEnter: () => { STATE.screen = 'region'; STATE.regionId = 'corvos_gate'; STATE.playerTile = { x: 4, y: 5 }; STATE.talkNpcId = 'corvo'; STATE.talkNodeId = 'root'; render(); }
  },
  {
    id: 'n_cross', name: '03 · Ash-Crawl Crossroads', kind: 'camp', pin: 'pin_camp',
    x: 310, y: 115, days: 1, mode: 'Parley / Camp Node',
    desc: 'Soot-choked tenement crossroads where smugglers and Tallow Guild lookouts trade rations and watch the Bone-Mill chimneys.',
    edges: ['n_corvo', 'n_kiln', 'n_crucible'],
    onEnter: () => { STATE.screen = 'region'; STATE.regionId = 'corvos_gate'; STATE.talkNpcId = 'corvo'; STATE.talkNodeId = 'root'; render(); }
  },
  {
    id: 'n_kiln', name: '04 · The Fourth Kiln (Bone-Mill)', kind: 'fortress', pin: 'pin_fortress',
    x: 375, y: 195, days: 2, mode: 'Raid (M3 Slag Vents)',
    desc: 'Industrial rendering furnace overseen by Foreman Grendel. Rhythmic Slag Vents erupt every even round across the foundry floor.',
    edges: ['n_corvo', 'n_cross', 'n_crucible'],
    quest: true,
    onEnter: () => launchEncounter('enc_m3')
  },
  {
    id: 'n_crucible', name: '06 · Crucible Watchpost (Climax)', kind: 'fortress', pin: 'pin_fortress',
    x: 435, y: 95, days: 2, mode: 'Act I Boss Climax (M5)',
    desc: 'High Inquisitor-Prelate Nemeth\'s bastion above the Great Sluice Winch. Seize the Winch tile (6,4) or break Prelate Nemeth in Phase 2 Consecration.',
    edges: ['n_cross', 'n_kiln'],
    quest: true,
    onEnter: () => launchEncounter('enc_m5')
  }
];

