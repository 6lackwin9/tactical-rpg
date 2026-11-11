# Whole-Codebase Consistency Audit — 2026-11-10 (`@0de2c60`)

- **Date**: `2026-11-10`
- **Branch / Commit**: `origin/develop` `@0de2c60` (initial pass `@0f4d5a4`, reconciled with `PR5-7` `@c1ca8cc`, `FB-191` `@1995313`, and `PR5-18` `@0de2c60`)
- **Scope**: Full 22-crate workspace (`engine/`, `kits/rpg/`, `games/emberwake/`), `kits/rpg/schema.json`, `games/emberwake/data/`, `tools/`, active specs (`docs/sessions/*.md`, `docs/specs/*.md`, `CAST1_*.md`), READMEs, and `games/emberwake/docs/lore/**`

## 1. Pillar 1 — Specs, Docs, Schema & Tooling vs. Build

### 1.1 Active `R5` (`PK1` / `PK2`) & Tooling Bugs (Queued in `PR5-26`, `PR5-27`)

| ID | Area | Finding | Resolution |
| --- | --- | --- | --- |
| `B1` | `rpgkit_battle/src/command.rs:1062-1084` | `Command::into_rest` leaves `self.skill` and `self.marrow` populated; converted Rest commands retain stale skill/channeled state and consume `uses_per_battle`. | Clear `skill = None` and `marrow = false` (`PR5-26`). |
| `B2` | `rpgkit_world/src/roster.rs:223-237`, `rpgkit_rules/src/unit.rs:1495` | `stash_forbidden` (`forbid_hands: 2`) ignores empty `m.equipment` slots that inherit class/unit default `2h` weapons (`wpn_iron_billhook`, `stirrup_arbalest`), and writes `""` instead of `"none"`, so `build_from` re-equips the forbidden class default. | Check resolved effective loadout and write `"none"` (`NO_ITEM`) when stashing (`PR5-26`). |
| `B3` | `rpgkit_content/src/check/unit.rs:544`, `rpgkit_world/src/state.rs:813` | `marrow_errors` checks untruncated `auto_actives_all`, while `UnitState::deck_errors` checks `self.deck.active` (`[]` on auto decks) instead of effective `actives_of`. | Validate against truncated `auto_actives` / `actives_of` in both (`PR5-26`). |
| `B4` | `emberwake_gui/src/party.rs:791-965` | `DeckTerms::base`, `unordered`, and `deck_json` drop `deck.marrow`, silently wiping channeled active selections on any party/Dojo Deck edit. | Preserve, prune, and serialize the channeled list (`PR5-26`, then move to kit in `PR5-31`). |
| `B5` | `rpgkit_battle/src/command.rs:554`, `dojo.rs:861` | `Command::pierces` (`HitForecast` `"Armour pierced"`) and `DojoSession` omit `attacker.pierce_melee`. | Include `pierce_melee && !is_ranged` in both (`PR5-26`). |
| `B6` | `rpgkit_battle/src/dojo.rs:1390`, `party.rs:1361` | Dojo and Party Gear screen allow equipping `forbid_hands: 2` two-handed weapons. | Enforce `Unit::forbids` in Dojo and Party Gear (`PR5-26`). |
| `B7` | `rpgkit_rules/src/damage.rs:163` | `weapon_for(u, false)` reads raw `main_hand()` instead of `active_weapon()` on stowed 2H+1H pairs. | Route `off == false` through `u.equipment.active_weapon()` (`PR5-26`). |
| `B8` | `emberwake_gui/src/battle/marks.rs:444`, `tips.rs:437`, `hub.rs` | Hazard overlays omit `HazardMeter` (`+N Strain`); strip Rest tip ignores `channel_rest` (`Feed`); `BattleOutcome` drops `Settled.tracks` and `stashed`. | Wire `HazardMeter`, Channel Rest/mark tips, and settle lines (`PR5-26`). |
| `B9` | `rpgkit_content/src/source.rs:519`, `check/unit_track.rs` | `hp_cost_pct` and `uses_per_battle` missing from `RANK_KEYS`; `unit_tracks.name` allows empty string. | Add both keys to `RANK_KEYS` and validate non-empty `name` (`PR5-26`). |
| `B10` | `rpgkit_host/src/lib.rs` | `round_plan`, `state_of`, `outcome_of`, and `skills_of` omit `PK1`/`PK2` fields (`channel`, `marrow`, `marks`, `skill_uses`, `forbid_hands`, `pierce_melee`, `unmark`, `uses_per_battle`). | Expose in `rpgkit_host` JSON replies (`PR5-26`). |
| `B11` | `meter.rs`, `ai.rs`, `rpgkit_sim/src/{args,feel}.rs` (`FB-191`, `FB-193`) | `meter::round_end` decays Strain `100 -> 90` before `round_end` crisis triggers (`0/120` crises); `ai::channel_row` ignores `channel.only` skills; AI never Vents; `sim tune=`/`counts=on` lack Channel/Strain support; `sim party seeds=40` resets to `10`. | Run `round_end` triggers before decay, score `channel.only` + AI Vent, extend `tune=`/`counts=on`, honour `seeds_set` (`PR5-26`), and update `inoculated_hero` / `pk1_channel_duel` while removing `channel_duel` from `dojo_presets.json` (`PR5-27`). |
| `B12` | `tools/fc.sh`, `tools/tiles/board.py:263` | `rust-toolchain.toml` pins `stable-x86_64-pc-windows-gnullvm`, breaking `fc.sh tooltests` (`ground_compose` subprocess) on Linux when `RUSTUP_TOOLCHAIN` is unset. | Default `RUSTUP_TOOLCHAIN=stable` on non-Windows when unset (`PR5-26`). |

### 1.2 `R1`–`R4` Spec, Schema & README Drift (Queued in `PR5-28` + Applied to Docs)

- **Queued in `PR5-28` (`R14-SPEC-CLEANUP`)**: Retire dead `spell_reach` across `rpgkit_rules`/`rpgkit_battle`/`rpgkit_content`/`schema.json` (migrating `wpn_orb` to `range_band`); read reach from `item_families.json` `range_band.max` instead of `REACH_WEAPON_REACH = 2`; remove stale `block_cost` help and `CAMPAIGN1-DOC` notes in `schema.json` and document `reach` on `engine_rules` plus `PHASEFX1` `side`/`swap` on `encounters.phases`.
- **Reconciled in Design Docs**: Updated `A2_RESOLUTION.md`, `S1_STATUS.md`, `S2_POISE.md`, `U1_STATS.md`, `U2_CLASS.md`, `U4_PASSIVES.md` (`guard_cap = 0.90` kept per `R2-CLOSE`, heal `finish_min: 0` without `KNOW_Potency`, consecutive-round `stagger_repeat_mult`, `stagger_windup_mult = 1.0` in `timeline`, embedded `tiers.json` `{id, mult, skill_cap}` with aliases removed in `PR4-2`), plus `README.md`, `kits/rpg/README.md`, `engine/README.md`, and `GAME_BRIEF.md`.

## 2. Pillar 2 — Three-Layer Split & Rule-Engine Purity

### 2.1 Downward Leaks (`engine/` & `kits/rpg/` Naming Game/Lore Concepts — `PR5-29`, `PR5-30`)

1. **`engine/`**: `aegis_core/src/assets.rs:757` and `aegis_ui/src/portrait_fx.rs:33` hardcode `["veins", "eye_glow", "ash", "smoke"]`; `aegis_ui`/`aegis_host` carry RPG nouns (`battle`, `round`, `block`, `encounter`, `ruleset`).
2. **`kits/rpg/`**: `PK1`/`PK2` introduced Emberwake lore identifiers (`marrow`, `marrow_actives`, `marrow_mark`, `cold_steel_mark` `"Cold Steel"`, `[Spasm]` in `log.rs:248`, `"doctrine lean"` in `schema.json:587`); `rpgkit_content/src/check/{item,unit,skill}.rs` hardcode `polearm, scourge, censer, bone_saw, sprayer` and closed 16-family arrays instead of reading `item_families.json`; `rpgkit_battle/src/dojo.rs` hardcodes 19 catalog unit/item IDs; `grid.rs` hardcodes `TERRAIN_IDS`.
3. **`tools/fc.sh` `layers_check`**: `sed` replacement bug (`\&` instead of `\\&` at `fc.sh:217`), case-sensitive `G3`, `engine/` omitted from `G3`, `tests/` omitted while containing Emberwake names (`kaelen`, `sunk_ward`), `G4` missing in `check_project.ps1`, and `lore_words.txt` missing `marrow`, `Cold Steel`, `strain`, `calcification`, `graft`.

### 2.2 Upward Leaks (Game-Unique Rule Engines in `games/emberwake/` — `PR5-31`..`PR5-33`)

1. **`emberwake_gui/src/party.rs` (`~2,200` lines, `PR5-31`)**: Implements standalone deck validation/auto-pinning (`DeckTerms`), classic 5+3 slot/rank validation, 2H/1H off-hand auto-stow, pouch stack editing, and active squad rules outside `rpgkit_world` (already causing `B4` and `B6`), plus stale Godot-era AP help (`party.rs:307-337`, `forge.rs:141`).
2. **GUI English String-Scraping (`PR5-32`)**: `intent_text.rs:122-243` parses kit refusal strings via `strip_prefix`/`split(" (")`; `fx.rs:606,868` and `battle/flow.rs:533` scrape `[Dodged]`, `CRIT`, `Guard Broken!`, `[Break]`; `tips.rs:388` hardcodes `"Channel (Marrow)"` and re-implements `Field`/`Assault` bonus math.
3. **Hardcoded Game VFX/Status/Stage & Sim Tables (`PR5-33`)**: `fx.rs` (`STATUS_LOOKS`, 25 skill VFX, 16 of 21 weapon families — missing the 5 new Emberwake weapon families), `tokens.rs` (`SKILL_CLIPS`), `stage.rs` (`TERRAIN` table), and `emberwake_sim/src/lib.rs` (`"ashlands"`, `tl_*`, 36-skill scenarios) hardcode data in Rust instead of JSON + kit readers.

## 3. Pillar 3 — Lore/Worldbuilding vs. Shipped Data & User Rulings (`A1`–`A6`)

### 3.1 `sunk_ward` Data-Wiring Bugs (Queued in `PR5-34` `SUNKWARD-FIX1`)

- **Typos & Missing Rows in `sunk_ward/characters.json`**: `kaelen` (`castellan_penance` -> `castellans_penance`), `maren` (`copyist_st_lyra` -> `copyist_of_st_lyras`, restoring `bile_phial` for `bile_coat`), `elyse` (`volatile_reaction` -> `volatile_reaction_catalyst`); add missing `vesper`, `hollis`, `dunstan`, `inquisitor_prelate_nemeth` (`marrow_infused`), `guild_master_orlov` (`vat_masters_grease`), `calcified_amalgam` (`amalgam_carapace` + `{"hp_pct": 60}`), `overseer_grendel`, `bailiff_enforcer`, and `calcified_husk`.
- **World & Catalog Gaps**: Set `sunk_ward/world.json` `"start_act": 1` (fixing `roster.rules` act `1..3`) and update its Vanguard blurb to 2H polearms; set `doom_clock` `start: 42, max: 42, visible: true` and `alert` `visible: true` in `sunk_ward/meters.json`; author `orlovs_flensing_talon`, `tithe_captive`, and a Level 1 Cold Steel censer `ATTACK` skill for `Heretic Acolyte` / `founder_maren`.

### 3.2 User Rulings Summary (`A1`–`A6`)

- **`A1` (`PK1` Dojo vs. Battle Fixture)**: Keep `DojoSession` lightweight; drop `channel_duel` from `dojo_presets.json` and test Channel/Strain in `pk1_channel_duel` (`PR5-27`).
- **`A2` & `A3` (Full Layer Purity)**: Execute both downward (`PR5-29`, `PR5-30`) and upward (`PR5-31`..`PR5-33`) layer cleanups so zero game-unique rule engines remain in `games/emberwake/`.
- **`A4` (`R1`–`R4` Spec & Schema Cleanup)**: Reconcile `R1`–`R4` session docs now and run `PR5-28` (`spell_reach` retirement, data-driven `reach`, `schema.json` cleanup).
- **`A5` & `A6` (`sunk_ward` Data + Canon/Lore Alignment + Orlov Retreat)**: Queue `PR5-34`; align `CANON.md` and `docs/lore/` with locked mechanics (`35%` Active Guard, 5 Round Intents + converting Channel toggle, Strain `50` post-crisis, Rock Salt `-10` floor, 2H billhook Vanguard/Kaelen, single `worlds/sunk_ward/` manifest); and give Guild-Master Orlov a low-HP `orlov_withdraws` retreat phase in M4 (`CAST1_BOSSES.md`, `CAST1_ENCOUNTERS.md`, `CANON.md`, `who_is_who.md`) so he survives into Act 2 for Vesper's loyalty quest.
