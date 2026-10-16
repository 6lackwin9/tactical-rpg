# Game Brief (agent-facing: read this instead of crawling the codebase)

Godot 4.7 tactical RPG, GDScript, strictly typed, fully headless-testable. Rules engines are `RefCounted` (no scene dependency); scenes only render and call them.

## Layers (who owns what)

| Layer | Owns | Entry points |
| --- | --- | --- |
| Base catalog (lore-free) | skills, items, passives, classes, units, encounters as JSON | `scripts/content/` (`ContentCatalog` store+validation, `ContentSchema` field defs, `ContentFactory` JSON -> engine objects), data in `data/catalog/*.json` |
| World layer (lore) | worlds, regions, locations, factions, characters, named items, disciplines, quests, lore, encounters; inheritance between worlds | `scripts/world/` (`WorldCatalog`, `WorldSchema`, `WorldFactory`), data in `data/worlds/<id>/` |
| Rules/specs | stat rules, classes, proficiency, passives, character stats | `scripts/core/specs/` |
| Combat core | Combatant, damage formula, statuses, inventory/equipment | `scripts/core/combat/`, `scripts/core/inventory/`, `scripts/combat/status/` |
| Planning + resolution | sequence planner, block resolver, AI, session | `scripts/combat/planning/` (`CombatPlan`, `SkillCatalog`), `actions/`, `ai/`, `session/` (`CombatSession`, `CatalogEncounter`, `WorldEncounter`) |
| UI | arena panels, forges, dojo, runner | `scripts/ui/*`, scenes in `scenes/` |
| Tests | headless suites | `scripts/tests/*`, dojo checks in `scenes/dojo/dojo_assertions.gd` |

## Core rules (numbers that tests depend on)

- Stats: STR DEX INT CONS PERC KNOW. Max HP = 60 + CONS*8 + STR*2 (x tier HP). Speed = max(1, 8 + DEX - armor burden). Crit = 5% + 1%/DEX above 10 + 1%/weapon training level above 1, x1.5.
- Damage: `DamageCalculator.compute` is the ONLY formula. Attack Power = (weapon power + training) + scaling stat x proficiency scaling (1.0-2.0, default 1.5), x tier power. Authored unit `power` is absolute. Hit = Attack x Focus x Skill x Stance matchup (1.25/0.75/1) x Crit x (1-Guard) x (1-Armor), armor = rating/(rating+50).
- Combat: 6 blocks per round, simultaneous resolution, AP pool with regen, skill AP = skill level, stances Offensive > Tactical > Defensive > Offensive, +1 AP stance switch, AP debt allowed.
- Classes (exactly six): warrior (tank), ranger, priest (support), brawler, rogue, mage. `StatRules`: budget 62 (6-18/stat), growth 5/level (max 3/stat), unit spend pool = 4 + (level-1) spent exactly (or empty = follow class weights), levels 1-10, skill tiers unlock by level (no skill cap; see "Skill unlock rule (SK2)" below), tiers hero/minion/standard/elite/boss scale HP and power.
- Proficiency: `{gear type: {level 1-3, scaling 1.0-2.0}}` per class; unlisted gear unequippable; training gives +1 power, -2 requirement, crit, burden, block cost (lvl 3), skill gating (`SkillDef.weapon_types/min_proficiency`).
- Passives: catalog templates (stat w/ context, proficiency, resist, ap_regen, skill). Granted only by SOURCES (world character, faction `grants`, discipline `passives`, named item `grants`; later quests/lore). Never by classes/base units.
- Skills: class `skill_path` by level + `extra_skills` + `Combatant.learn_skill`. `Combatant.can_use` gates planner/AI.
- Units: class + level + side (party/enemy/neutral) + tier + spend + gear overrides; `stat_mode: fixed` only for special units (training dummy). A goblin is lore on top of a minion class; worlds set level + explicit tuning deltas (`hp_pct, power, armor, speed, ap`).
- Worlds: base asset decides gameplay; two characters on one base unit behave identically except explicit tuning/items/passives.

## Known gameplay facts that surprise people

- Speed varies a lot by class (ranger ~35, warrior ~16 at level 5); never assume the warrior is faster.
- The planner resolves "friend vacates tile" fastest-first.
- Dojo warrior (Lv2, tank kit) hits the dummy for 47; dojo tests now derive this from the stats and weapon.

## Where to look for X

- Add/edit content data: `ContentCatalog.upsert` (never hand-edit JSON). Fields: `content_schema.gd` (API; tables in `SkillFields`, `ItemFields`, `ClassUnitFields`, `scripts/content/*_fields.gd`). Validation: dispatch in `content_catalog.gd` (`validate_entry`, `warnings_for`); rules per kind in `scripts/content/validators/` (`ClassUnitValidator.class_errors`/`unit_errors`, `ItemValidator.item_errors`, `SkillValidator.entry_errors`). Attack power has one source: `DamageCalculator.attack_power`. Skill hand/grade: `SkillDef.is_off_hand()`, `SkillDef.grade`.
- Change damage: `damage_calculator.gd` only. Planner estimate and dojo already share it.
- Change a class: `data/catalog/classes.json` via upsert; rules in `stat_rules.gd`.
- World data/inheritance: `world_catalog.gd`; character spawn: `world_factory.gd`.
- Test expectations for a system: its suite in `scripts/tests/` (class rules: `class_rules_test_suite.gd`).

See `docs/ROADMAP.md` for what is next and `docs/DECISIONS.md` (index) plus `docs/decisions/<system>.md` for why things are the way they are.

## Combat extensions (M1, S12-A)

- Support skills (`SkillDef.Kind.SUPPORT`, target modes SELF/ALLY/ALLY_OR_SELF): heal = `heal_base + round(KNOW|INT x heal_mult)` (`SupportRules`), optional `grant_status`. Planner mode `PlanStep.Mode.ALLY`.
- Gap-close: `dash_tiles` on attack skills; the dash is the move for the round, first in the sequence.
- Stance: Rest -> Tactical (free entry); a stance change costs `CombatTypes.STANCE_SWITCH_COST` = 1 AP (single constant).
- Statuses (eight in `StatusRegistry`): weaken lowers outgoing skill power by x(1 - magnitude/100) via `SkillDef.power_for` and the riposte counter (cap 25, magnitude from `status_magnitude`, stored per unit); a zero-power ATTACK needs a status and deals no damage. haste (+1 tile per move block), bleed (6% max HP at round end, ignores armor/guard), taunt (forces TacticalAI target, not the player planner); all start next round for `status_rounds`. Resistable keys: stagger, stun, root, bleed.
- AI: kites to max range, no idle turns, SUPPORT heals the most wounded ally (<=70%, max 2/round), `reckless` profile never flees. `learned_only` skills reach a unit only via extra_skills / learn_skill.
- Riposte (M2a): DEFEND skill `riposte` (warrior path family; AP, mitigation and rounds per rank are skill data in `skills.json`, not code constants) grants status `riposte` immediately for the rank's `status_rounds` (`StatusEffects.grant_now`); `RiposteRules.counter` (called by Attack/AreaAttack commands) counters adjacent melee hits it survives for 50% power via `DamageCalculator.compute`, once per attacker per round; `Combatant.set_stance` removes it. Results go in `BattleCommand.reactions`.
- Gear budget (M2d): `GearBudget` (`scripts/core/specs/gear_budget.gd`) gives each equipment item one weighted value (power 1.0, mitigation 1.5, stat bonus 2.0/pt, burden -2.0, extra block cost -3.0) and sums a class's default loadout (pouch ignored) against TARGET_BUDGET 23 +-20%. Soft only: `ContentCatalog.warnings_for(kind, entry)` / `warnings_all()` (separate from `validate_all()`, which stays `{}`). Items with `special` + `special_note` may push a loadout over the band; special items are listed separately in the Forge class preview.

## Skill families and loadouts (M2c-2a1)

- Skills carry `family` + `tier`; the planner offers each affordable tier (AP = tier's level, offered once the tier is unlocked by level, see "Skill unlock rule (SK2)") grouped by family ("Mend I / II / III", `PlanOption.group_by_family`). Basics are exactly strike, guard, move, rest, switch_target and offhand_jab: single tier, always active, no variants (their late-game scaling will come from gear affixes and passives through the `skill_potency` hook).
- Active loadout: max 5 class families + 3 extra (`StatRules.MAX_CLASS_SLOTS/MAX_EXTRA_SLOTS`); `Combatant.set_loadout` only outside battle (locked by `CombatSession.start`). Empty loadout = unrestricted (hand-built units).

## Engine pieces (M2c-2a2)

- Cleanse: SUPPORT field `cleanse_statuses` (debuff ids or `all`) removes active AND pending debuffs via `SupportRules.apply` (never buffs; combinable with heal and `grant_status`). Planner options list what is removed and warn "Nothing to cleanse". Example `purify`.
- Disengage: `SkillDef.Kind.DISENGAGE` + `jump_tiles`; `CombatPlan._apply_disengage` picks the landing tile (farthest from the nearest/chosen foe, then shortest jump, nearest centre, lowest x/y; a GOTO step names the tile), `DisengageCommand` jumps (over units, not walls). It is the unit's move for the round (first, refused when rooted), AP = skill level. Example `backflip`.
- `Area.SWEEP`: the L (1,0) (1,1) (0,1) rotated into the quadrant of a diagonal aim tile; friendly fire as other areas. Example `sweep_kick`.
- Unarmed: `DamageCalculator` already uses STR+DEX with no weapon; `GearBudget.UNARMED_VALUE` (14) counts for an empty main hand.

## Foundation (slice A)

- `StatusRegistry` is the single table of statuses (kind, resistable, default rounds, stacking rule, cap); `StatusEffects.debuffs()/buffs()/resistable()` derive from it; add a status by following the checklist in its header.
- `unarmed` is a valid skill `weapon_types` token (no main-hand weapon; proficiency uses the class's `fist` level).
- Items have a derived hand: `main_1h`, `main_2h` or `off` (`Equipment.hand()`).
- `Combatant.potency_for(family)` / `SkillDef.power_for(user)` scale skill output; the `skill_potency` passive effect feeds it. Default behaviour unchanged.

## Item effects (C1-L)

- Items have an `effects` list (passive grammar restricted to item types: `skill_potency` by family or group `spell`/`heal`, `status_potency` 1..25, `spell_reach` 1..2, `resist`). They apply only while equipped and are rebuilt on every `sync_from_stats_and_gear` (`Combatant.rebuild_gear_effects`), kept in separate `gear_*` fields. Spell = ATTACK skill gated on staff or wand; heal = healing SUPPORT skill. Gear spell reach is clamped to 1. Orb and tome are off-hand only; a 2H main weapon refuses them.

- Add a test suite: new `scripts/tests/<name>_test_suite.gd` + one `results.append` line in `scripts/tests/suite_registry.gd` (called by `scenes/test_arena.gd`); shared helpers in `TestHelpers` (report / report_ok / failed, unit, sandbox + fork / cleanup / fork_worlds, skill fixtures). Sandboxes: always `TestHelpers.sandbox(tag)`, never a fixed `user://` path. Never add tests to a shared runner file.

- Area attacks roll the skill's status once per unit hit (allies included). A DEFEND skill may grant riposte (immediate) or a support buff (taunt, haste: queued next round).

## Progression and saves (M3)

- `PartyState` / `UnitState` (scripts/progression/): level derives from `total_xp` (`XpCurve`, data/progression/xp_curve.json); `LevelUp.award_encounter` gives full XP to every active member; `PartyBuilder.build` turns a state into a Combatant (same `ContentFactory.unit` path as catalog units). `PARTY_MAX` = 4 active, roster unbounded.
- `GameState` + `SaveGame` (scripts/game_state/): single slot file `user://saves/slot_<n>.json`, envelope `SaveGame.VERSION` = 1, migrations in `save_migrations.gd`; `NewGame.create` builds a starting state.

## AI skill use (AI1)

- `TacticalAI` (AI1 section at the end of `tactical_ai.gd`) casts disengage, area attacks, cleanse-only skills, taunt, ally buffs, debuff attacks and riposte when `AiProfile.smart_skills` is true (default); false = old behaviour. Area attacks need foes hit >= `area_min_foes` (2) and no ally hit. Zero-power attacks are only cast as debuffs. Tests: `ai_skills_test_suite.gd`.

## Skill unlock rule (SK2)

- A class path step `{level, skill}` owns a family. Tier t is castable at `unlock + unlock_offset` (offset default 0/2/4 for tiers 1/2/3). AP is only the per-round budget (no skill cap). Families of `grade` advanced or higher need path level >= 5; KNOW lowers an advanced family by `(KNOW-12)/4` and the passive `unlock` effect lowers any path family; the larger of the two applies, max 2 levels, advanced floor 3. Constants in `StatRules` (TIER_UNLOCK_STEP, ADVANCED_MIN_LEVEL, MAX_UNLOCK_REDUCTION, ADVANCED_UNLOCK_FLOOR, KNOW_UNLOCK_BASE/STEP, MAX_UNLOCK_OFFSET). Unlock levels freeze while the loadout is locked (battle). Read: `Combatant.family_unlock_level`, `tier_unlocked`.

## Ranked skills and asset origin (SKILL1)

- `skills.json` version 2 stores one entry per family (base + up to 2 rank modifiers); in memory rows stay flat (`head`, `head_2`, `head_3`). Edit skills with `ContentCatalog.upsert_skill(entry)` (ranked) or `upsert`/`upsert_many` (flat rows); read `skill_entry(head)` / `skill_entries()`. Ranks: `scripts/content/skill_ranks.gd`; kind sections: `skill_kinds.gd`; description tokens: `skill_text.gd`.
- `grade` (novice/advanced/expert) and `requires` (lower-grade families a unit must own) replace `advanced` / `evolves_from`. `weapon_hand` (main/off; `both` off while `SkillDef.BOTH_ENABLED` is false) replaces `offhand`.
- Basics (strike, guard, move, rest, switch_target, offhand_jab) are owned by every unit from level 1 and are rejected in class paths.
- `origin` system/user on every kind: system assets lock key fields (seeds edit them inside `begin_system_edit`/`end_system_edit`) and cannot be deleted; `promote`/`demote` switch origin. Safe delete: `AssetScan` + `tools/safe_delete.gd`.
- Saving never drops data: a kind whose file loaded with errors is save-blocked (`ContentCatalog.save_blocked(kind)`); `validate_all` reports unfoldable skill families under `skills:fold`. `warnings_all` checks skills once per family, keyed `skills:<head id>`. `ContentSchema.fields()` always returns copies.
- Forge skill logic: `scripts/ui/forge/skill_form_model.gd`, `skill_filter.gd`, `skill_ranks_panel.gd`.

## Balance rules and sim v2 (BAL2-L)

- Attack range +1 at PERC >= `CharacterStats.RANGE_PERC_THRESHOLD` (18) only for ranged weapons (base range > 1); melee units stay at range 1.
- Unit gear slot `none` (`ContentSchema.NO_ITEM`, pouch `["none"]`) = no item; a blank slot = class default. Units only (classes and world characters reject it).
- Tiers: elite 1.25 HP / 1.1 power, boss 1.5 / 1.1 (`StatRules.TIERS`).
- Trained fists: empty main hand + fist proficiency >= `DamageCalculator.TRAINED_FIST_MIN_LEVEL` (2): weapon power `UNARMED_BASE_POWER` (15) + fist training, fist crit bonus; `GearBudget.UNARMED_VALUE` reads the same constant. Inputs only, the formula is unchanged.
- Sim: `tools/fc.sh sim <duel|tier|levels|group|stats> [level] [seeds] [key=value]` (`tools/balance_sim.gd` + `tools/sim/`); cells `binary/hp-margin`; duel = diagnostic band 30-70 (party and role targets: `docs/decisions/balance.md`). Modes party / role / sweep (BAL2_SIM 4-8): role tests start enemies 4 tiles away (`distance=` overrides); off-target values marked X; `METRIC FAIL` lines (`log_damage`, `log_area`, `log_heal`, `log_riposte`, `other_*`) mean the collector found HP it cannot explain; party fight ~3 s, full role run ~28 min. Tests: `bal2_rules_test_suite.gd`.
