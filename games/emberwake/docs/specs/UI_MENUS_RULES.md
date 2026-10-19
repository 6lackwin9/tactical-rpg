# UI-MENUS-RULES: main menu, new game, save/load, party screen, level-up, journal (baseline)

Status: proposed (design-brainstorm 2026-10-17; baseline rules for the open UI parts of S7 and S13 so they can be specced later; nothing approved)
Depends on: M3 (done: PartyState, UnitState, LevelUp, GameState, SaveGame, NewGame), M4-L (quest runtime and save v2, for the journal), S14-RULES (icons, sounds, animation speed; optional, placeholders work without it)
Lane: design (this file); later logic and asset tasks listed in section 4
Touches (when built): `scripts/ui/menus/*` (placeholder folder: `main_menu.gd`, `party_screen.gd`, `journal.gd` planned), new menu scenes under `scenes/`, `scripts/game_state/save_game.gd` (slot header read, autosave slot), `scripts/game_state/new_game.gd` and the world schema (start party, Q-M2), reuse of `scripts/ui/character_equipment_modal.gd`, new suite `menus_test_suite.gd`

## 1. Scope and what it is NOT

In scope (ROADMAP S7 "level-up UI still open", S13 "menus still open", build order step 4):

| Screen | Purpose |
| --- | --- |
| Main menu | continue, new game, load, settings, quit; Tools entry for dev builds |
| New game | pick world and starting party, create the GameState, first save |
| Save / load | slot list with headers, manual save, autosave, overwrite and delete confirms |
| Party screen | roster and active four, per-member stats, level-up spend, gear, skills and loadout, passives |
| Battle results | XP per member, level-ups, loot; entry point to level-up |
| Journal | quests (M4 runtime) by status; later lore (Q-M3) |
| Settings | audio volumes, animation speed and instant mode, text size; stored outside saves |

NOT in scope: exploration, world map and travel screens (S9, S10), dialogue and shop screens (S11), combat HUD redesign, the planner skill-tier picker (ROADMAP open engine item; a combat UI task, suggested separately), Forge and Dojo tools, the art and theme pass (S14 and later), gamepad support, localisation, respec (DECISIONS: confirmed spend is permanent).

## 2. Baseline rules

### 2.1 Architecture

1. Every screen is a `RefCounted` model in `scripts/ui/menus/<screen>_model.gd` plus a thin Control view (same split as `scripts/ui/forge/skill_form_model.gd`). Models are headless-testable; views only render model state and forward input.
2. Models change game data only through the existing rule APIs: `NewGame.create`, `LevelUp.confirm` / `auto_for`, `UnitState.validate`, `PartyState` (members, `active_ids`, stash), `SaveGame.write` / `read` / `exists` / `delete`, `PartyBuilder.build` for previews, `Combatant.family_unlock_level` / `tier_unlocked` for skill state. No gameplay rule is re-implemented in UI (single source, like `DamageCalculator`).
3. Every edit works on a DRAFT (a `duplicate_state()` copy); Confirm validates and commits, Cancel drops the draft. Errors from the rule API are shown verbatim next to the field (same problems-bar pattern as the Forge).
4. `MenuRouter` keeps a screen stack (push, pop, replace); Back (`ui_cancel`) pops; a screen with an uncommitted draft asks before leaving. One router for main-menu and in-game (pause) stacks.
5. Input: mouse plus keyboard (`ui_accept`, `ui_cancel`, arrows, Tab); every action reachable by keyboard; no gamepad in the baseline.
6. The scene runner stays the dev entry; the main menu shows a Tools entry (Forge, Dojo, Test Arena, Scene Runner) only when `OS.is_debug_build()`. Which scene is the project main scene is a planner call at build time.

### 2.2 Main menu and new game

- Continue: loads the slot with the newest `saved_at` (autosave included); hidden when no save exists.
- New game: world list from `WorldCatalog` (playable worlds only; Q-M2 decides how the party is chosen), then `NewGame.create(world_id, unit_ids)`; on errors the screen lists them and stays; on success the state goes to the in-game hub and an autosave is written.
- Quit asks only when the current in-game state differs from the last save (`GameState.equals`).

### 2.3 Save and load

- Slot header without a full load: new `SaveGame.peek(slot)` returning `{exists, ok, saved_at, version, world_id, location_id, party: [{id, base_unit, level}]}` from the envelope; a corrupt file shows as "unreadable" with Delete only.
- Load shows `SaveGame.read` warnings (migrations) before entering the game; errors keep the player in the menu.
- Saving is allowed only out of combat (GameState holds no battle snapshot). Overwrite and delete need a confirm. Writes keep the previous file on failure (existing tmp-and-rename rule).
- Slot count and autosave are Q-M1. Settings are NOT in a slot: `user://settings.cfg` (`ConfigFile`), keys `audio/master`, `audio/music`, `audio/sfx`, `audio/ui` (0..1), `anim/speed` (multiplier), `anim/instant` (bool), `ui/text_scale`; defaults when missing; never migrated with saves.

### 2.4 Party screen

- Roster: every member, active ones marked; toggling active edits `active_ids` with at most `PartyState.PARTY_MAX` (4); empty `active_ids` displays as the first four (current rule) and the first edit writes them explicitly.
- Stats tab: six stats with sources (class base, growth, spend), derived values from the built Combatant of the draft (max HP, speed, tiles per move block, crit, attack power through `DamageCalculator.attack_power`, armor rating). Level and XP bar from `XpCurve`.
- Gear tab: reuses `CharacterEquipmentModal` against `UnitState.equipment` and `PartyState.stash`; validation via `UnitState.validate` (proficiency, hands, requirements); the gear budget stays a Forge-only warning.
- Skills tab: owned families grouped by class path and extra, each with grade, tiers and the level each tier unlocks (`tier_unlocked`), locked tiers greyed with the unlock level. Loadout editor: 5 class slots plus 3 extra (`StatRules.MAX_CLASS_SLOTS` / `MAX_EXTRA_SLOTS`), basics shown separately and never slotted; "Use default" clears the unit loadout (empty = default rule). Validation via `UnitState.validate` (`_loadout_errors`). No AP cap or skill cap shown anywhere (removed in SK2).
- Passives tab: list with the source that granted each (quest, item, faction, discipline) when known; read only.

### 2.5 Level-up

- Trigger: `LevelUp.award_encounter` results (`levels_gained`, `unspent`) feed the battle results screen; Q-M4 decides whether it prompts.
- Allocation: the player distributes exactly the unspent points (1 per level after the base 4, DECISIONS "Level-up spend", HISTORY row 14) with plus and minus per stat on a draft; the UI enforces nothing itself beyond non-negative steps; `LevelUp.confirm(state, draft)` is the validator, and the preview shows before and after derived values (2.4).
- Auto button: `LevelUp.auto_for` on the draft (class growth weights); the player can still adjust before Confirm.
- Deferring is allowed: unspent points stay on the state and the battle builder auto-spends them on a copy (M3 rule), so the party screen shows an "unspent points" badge until confirmed.
- New skills: after the level change the screen lists families or tiers that became castable (diff of `tier_unlocked` before and after) and offers the loadout editor if a slot is free.
- Confirmed points are permanent (no respec, DECISIONS M3).

### 2.6 Battle results

- Shows per active member the XP gained, new level, and the loot or rewards granted (M4 `RewardGrant` output when present); benched members get none (M3 rule). Downed members still get XP (M3 rule).
- Continue returns to the caller (exploration later; the test arena or encounter runner now) after the autosave of Q-M1.

### 2.7 Journal

- Quests from the M4-L runtime, grouped by status (active, ready to turn in, completed, failed; `blocked` and `unavailable` shown only once known to the player), using resolved variants at read time (M4 rule: fields re-read on every read). Entry text: name, giver and turn-in metadata, objectives with progress, generic rewards and the loot pool with picks left.
- Read only: the journal never changes quest state.
- Lore codex and bestiary are Q-M3.

## 3. Open design questions (only choices that block a spec)

Q-M1 Save slots and autosave.

- A: one manual slot (slot 1) only, as M3 built. Simplest; a bad save cannot be undone.
- B: three manual slots (1-3) plus one autosave slot written after every battle and on new game; Continue picks the newest. Needs a reserved autosave slot id (SaveGame refuses slots below 1 today, so autosave gets its own file name or slot number) and `peek`.
- C: unlimited named slots. More list UI, no gameplay gain for one small campaign.
- Recommendation: B (the archived P-S13 plan was "A growing to B").

Q-M2 How the starting party is chosen.

- A: fixed per world: a new world field `start_party` (party-side unit ids, 1..4), validated by the world catalog; New Game only picks the world.
- B: the player picks 1..4 from all party-side units (`side: party`).
- C: the player picks one hero; the world adds fixed companions.
- Recommendation: A for the first campaign (DECISIONS "Playable scope": one small campaign first; story fixes the party); B can be a later "custom start" option on top.

Q-M3 Journal scope of the first slice.

- A: quests only.
- B: quests plus a lore codex (world `lore` entries unlocked by a flag field on the lore entry). Needs a world schema field owned by the S8/S9-S11 design work.
- C: B plus a bestiary of encounters seen.
- Recommendation: A, with B once the world lore unlock field is decided by the S8-S11 specs.

Q-M4 When level-up allocation happens.

- A: only from the party screen, any time out of combat.
- B: a forced prompt on the battle results screen.
- C: a skippable prompt on the results screen plus the party screen any time.
- Recommendation: C (players see level-ups at once without being blocked; deferral already works through the auto-spend copy).

## 4. Milestone shape (coarse; each needs user approval)

| Task | Lane | Content | Depends on |
| --- | --- | --- | --- |
| UI1-L | logic, schema first | `MenuRouter`, main menu, new game (Q-M2 field and validation first), save and load with `peek` and the Q-M1 slot rules, settings file, suite `menus_test_suite.gd` (sandboxed save dir via `SaveGame.set_base_dir` and `TestHelpers.sandbox`) | M4-L merged (save v2 envelope, world schema) |
| UI1-A | asset | `start_party` on the playable worlds (Q-M2 A) | UI1-L commit 1 |
| UI2-L | logic | party screen models (roster, stats, gear reuse, skills and loadout, passives), level-up draft and preview, battle results | UI1-L |
| UI3-L | logic | journal over the M4 runtime | UI1-L, M4-L commit 2 |

S14-L1 icons and S14-L3 sounds plug in later through keys; until then views use `UnitArt` and `ItemArt` directly as today.

Rule-derived acceptance checks for the suites:

1. New game with the world's `start_party` yields a GameState that passes `validate`, and its autosave reads back equal (`GameState.equals`).
2. `peek` on a corrupt file returns `ok` false without throwing; on a valid file its world and party match `read`.
3. A level-up draft the model accepts is exactly one `LevelUp.confirm` would accept (same error list for an overspent draft); Cancel leaves the state equal to before.
4. Toggling a fifth active member is refused; the first toggle on empty `active_ids` writes the first four explicitly.
5. A loadout edit with 6 class families is refused with the `UnitState.validate` message; basics never appear as slottable.
6. The new-skills list after a level-up equals the diff of `tier_unlocked` between the old and new level.
7. Settings round-trip through `user://settings.cfg` and are untouched by save, load and delete of any slot.

## 5. HISTORY rows that touch the menus

| HISTORY row | Effect |
| --- | --- |
| 14, free level-up allocation deprecated | spend pool exactly; the UI distributes unspent points only, `LevelUp.confirm` validates |
| 28, AP-cost unlock and skill cap deprecated | no AP or skill cap display; tiers show unlock levels |
| 29, loadout on the class deprecated | the loadout editor edits the unit (`UnitState.loadout`) |
| 31, basic variants deprecated | basics are listed once, never slotted, no tiers |
| 27, `.context/` design notes deprecated (and row 1, the 10-block layer they described) | the early notes' combat UI ideas are not used; only the S9-S11 non-combat parts were kept (ROADMAP inputs) |
