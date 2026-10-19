# SK2: Level-based skill unlock (replaces the AP-cost cap)

Archived 2026-10-17 (CLN-5), task done: kept for history; current rules live in docs/GAME_BRIEF.md and docs/decisions/. Do not build from this file.

Status: done (header was "proposed")
Depends on: M2C2B (merged; supplies the families, paths and advanced rows this task re-gates)
Lane: both (SK2-L logic and schema first, then SK2-A small asset seed)
Touches: `scripts/core/combat/combatant.gd`, `scripts/core/specs/stat_rules.gd`, `scripts/core/specs/passives.gd`, `scripts/combat/planning/skill_def.gd`, `scripts/content/content_schema.gd`, `scripts/content/content_catalog.gd`, `scripts/content/content_factory.gd`, `scripts/combat/ai/ai_profile.gd` (comment only), `scripts/combat/planning/combat_plan.gd` (one message), `scripts/ui/forge/forge_preview.gd`, `scripts/progression/unit_state.gd`, `data/catalog/skills.json`, `data/catalog/classes.json`, `data/catalog/passives.json`, new seed `data/seeds/sk2.gd`, suites listed in "Tests". Overlap note: AI1 (in progress) edits `tactical_ai.gd`, `plan_presets.gd`, `skill_catalog.gd`; SK2 does not edit those three (the gate lives in `Combatant.can_use`, the single choke point), so the two can run in parallel.

## Goal

Replace "skill AP level <= ceil(unit level / 2)" with a plain level check. A class `skill_path` step `{level, skill}` says at which unit level a family is owned; the tiers of that family open at fixed level steps after it; AP is only the per-round budget. Advanced families are gated by a higher minimum path level (validated). Passives and the KNOW stat can lower a family's unlock level by a capped amount so a unit can own it earlier. Trigger: riposte T1 costs AP 3, so the old rule forced it to warrior level 5 (DECISIONS row "M2C2B content", user request 2026-10-16).

HISTORY.md check: nothing deprecated is re-proposed. Passives stay granted only by sources (DECISIONS row "Passives"); classes still grant none. The AP-vs-level tie came from the original unit skill cap ("stand-in for real skill progression", `skill_catalog.gd` header) and from the M2c-2a1 note that a family must not sit unusable in the loadout; the new rule keeps that guarantee by construction (T1 opens at the unlock level).

## Decisions taken (from the options)

1. Cast gate. Options: (A) keep the unit skill cap for tiers, which keeps the riposte problem; (B) a per-row absolute `min_level` field, which cannot be shared (hamstring is at brawler 4 and rogue 5) and cannot move with a passive; (C) tier level = family unlock level + a per-row offset, default 0 / 2 / 4 for tiers 1 / 2 / 3. Chosen: C. It reproduces today's pacing for ordinary families (a family opened at level 3 gets T2 at 5, T3 at 7, as the old cap did for AP 3 and AP 4) and lets one authored offset express the riposte "reverse" tiers.
2. Advanced gate. Options: (A) authored per path only (no protection against a later bad edit); (B) a skill flag plus a validation floor; (C) both. Chosen: C. Flag `advanced` on the family rows, validation floor `StatRules.ADVANCED_MIN_LEVEL` = 5; authors may place at 7 (judgement, assassinate, inferno already are).
3. Passives and KNOW. Options: (A) passive only; (B) KNOW only; (C) both under one combined cap. Chosen: C, with the cap `StatRules.MAX_UNLOCK_REDUCTION` = 2 levels total per family and a floor, so no combination can skip more than two levels of progression.

## Schema contract (schema-first: yes; the asset lane starts after this slice merges)

### Skill row (`skills.json`, `ContentSchema` skills)

| Field | Type | Allowed | Default | Meaning |
| --- | --- | --- | --- | --- |
| `advanced` | bool | true/false | false | Family-level flag; every row of a family must carry the same value (validation error otherwise). Only meaningful for class-path families (not basics) |
| `unlock_offset` | int | -1 or 0-8 | -1 | Levels added to the family's unlock level for THIS row. -1 = derive: `StatRules.TIER_UNLOCK_STEP * (tier - 1)` with `TIER_UNLOCK_STEP` = 2 (tiers 1 / 2 / 3 -> 0 / 2 / 4) |

Validation (`ContentCatalog._skill_errors`): `unlock_offset` outside -1..8 is an error; tier 1 with an explicit offset other than 0 is an error (T1 opens at the unlock level); `advanced` true on a basic or system skill is an error.

### Class path validation (`_class_errors`, replaces the "too advanced for unit level" check)

- Remove the check `skill.level <= StatRules.skill_cap_for_level(step level)`. A skill's AP level no longer constrains the step level.
- New: a step whose family is `advanced` must have `level >= StatRules.ADVANCED_MIN_LEVEL` (error text names skill, level, minimum).
- New: a family may appear at most once in one class path (error on duplicates; shipped data already complies).
- New: the step must name a tier-1 row (`tier` == 1); tier rows are never listed. Existing rule kept: ascending order, at least one level-1 skill, no system skills.

### Constants (`stat_rules.gd`)

| Constant | Value |
| --- | --- |
| `TIER_UNLOCK_STEP` | 2 |
| `ADVANCED_MIN_LEVEL` | 5 |
| `MAX_UNLOCK_REDUCTION` | 2 (passive + KNOW combined) |
| `ADVANCED_UNLOCK_FLOOR` | 3 (effective unlock of an advanced family never below this) |
| `KNOW_UNLOCK_BASE` | 12 |
| `KNOW_UNLOCK_STEP` | 4 |

New pure helpers in `StatRules`: `default_unlock(ap: int) -> int` = `maxi(1, 2 * ap - 1)` (the old rule, now only the fallback for off-path skills); `know_unlock_reduction(know: int) -> int` = `clampi((know - KNOW_UNLOCK_BASE) / KNOW_UNLOCK_STEP, 0, MAX_UNLOCK_REDUCTION)` (integer division; KNOW 16 -> 1, 20 -> 2, below 16 -> 0).

`StatRules.skill_cap_for_level`: no remaining caller after this task except `UnitState.skill_cap()`; delete both when unreferenced (builder greps; keep with a "deprecated" comment only if a caller remains).

### Passive effect `unlock` (`passives.gd`, TYPES gains `unlock`)

`{"type": "unlock", "family": "<skill id>" | "group": "spell" | "heal", "levels": 1-2}`. Exactly one of `family` / `group` (same validation as `skill_potency`: a known family id, group in `Passives.GROUPS`). `levels` int 1-2 (error otherwise). Not allowed on items (item effects stay restricted to their current types). Applied by `Passives.apply_effects` into `Combatant.unlock_bonus_family: Dictionary` and `unlock_bonus_group: Dictionary` (max wins per key, never summed across passives, so stacking passives cannot beat the cap). Describe text: "opens <family/group> N levels earlier". The effect lowers only families already on the unit's class path; it never grants ownership (the `skill` effect does that).

### Unit field `skill_cap` (units) and `AiProfile.skill_cap`

Meaning changes to "explicit AP ceiling override, 0 = none". `ContentFactory.ai_profile`: a derived (class-based) unit with `skill_cap` 0 gets profile cap 5 (no ceiling) instead of `skill_cap_for_level`; an explicit value > 0 is honoured as before. Hand-built profiles (`AiProfile.create(role)` default 1) are unchanged so existing hand-built tests keep their behaviour. `ContentSchema` help text updated. `prefer_level` is unchanged (AI preference by AP level among unlocked, affordable tiers).

## Rules (logic lane)

Definitions. `U(f)` = base unlock level of family f for the unit = the lowest `level` among the class-path steps whose skill is in family f. For a family not on the class path (extras, `learn_skill`, granted) `U(f)` = `default_unlock(T1 AP)`. Unit level L = `stats.level`.

1. Reduction `R(f)` (class-path families only, else 0): `min(MAX_UNLOCK_REDUCTION, max(passive R, KNOW R))`, where passive R = max of `unlock_bonus_family[f]` and the `unlock_bonus_group[g]` of every group f belongs to, and KNOW R = `know_unlock_reduction(KNOW)` applied ONLY if f is `advanced`. (max, not sum: one source at a time, so the cap of 2 is also the real ceiling; this is the conservative reading, see open question 2.)
2. Effective unlock `E(f)` = `max(floor, U(f) - R(f))` with floor 1 for ordinary families and `ADVANCED_UNLOCK_FLOOR` for advanced ones; a family with U(f) = 1 is never changed. Never below the floor even when U(f) is already under it (then E = U).
3. Tier availability: a tier row s of family f is unlocked when `L >= min(StatRules.MAX_LEVEL, E(f) + offset(s))`, `offset(s)` = `unlock_offset` or the derived `TIER_UNLOCK_STEP * (tier - 1)`. Reductions move all tiers of the family uniformly; they do not touch offsets.
4. Ownership: a path family is owned (T1 in `known_skills`) when `L >= E(f)`. Higher tiers are owned together with T1 but castable only per rule 3. Implementation: `Combatant` stores `path_levels: Array[Dictionary]` (copy of the class path) and `unit_level`; `Combatant.refresh_path_skills()` recomputes `known_skills` (path part, extras and granted preserved). It is called at the end of `ContentFactory.unit`, from `Passives.apply_effects` when an `unlock` effect is applied, and from `sync_from_stats_and_gear` (KNOW can change with gear). It does nothing while `loadout_locked` (a battle freezes ownership; mid-battle buffs and gear swaps cannot change it). KNOW used = `stats` base-plus-spend plus gear bonus, excluding temporary combat buffs.
5. The single cast gate: `Combatant.tier_unlocked(skill) -> bool` (rule 3; true for basics, system skills, and units with empty `known_skills`, i.e. hand-built unrestricted units). `Combatant.can_use` calls it after `knows_skill`. `SkillCatalog.available/grouped/single_attacks/area_attacks`, the planner offers and the AI already go through `can_use`, so none of them needs another change. The `skill_cap` parameter on those functions stays as the AP-ceiling override (callers pass the profile cap, 5 for derived units).
6. `CombatPlan._apply` keeps the AP-ceiling message for the explicit override; add a second message "`<skill>` unlocks at unit level N" when `tier_unlocked` is false. The planner's "too expensive / not enough AP" behaviour is the ordinary AP budget only; nothing else ties AP to level.
7. Off-path skills (rule definition above) keep working: a learned or extra skill is owned immediately (as today) but its tiers use `U` = `default_unlock(AP)` plus offsets; so an event-granted advanced skill is not castable at level 1.
8. Default loadout (`ContentFactory.loadout_of`): unchanged rule, "first 5 non-basic path families owned at L in PATH ORDER", now using ownership from rule 4. Because advanced families are listed after the starters, an earlier-owned advanced family never displaces a default; it is owned and slottable. Slots stay 5 class + 3 extra; the player (or a unit `loadout` list) chooses. A unit `loadout` entry naming a family not yet owned at L is an existing validation concern; the builder keeps the current message and must not count passive-lowered unlocks in catalog validation (validation uses unreduced path levels, so an authored loadout that relies on a passive is rejected until a world-level check exists, see open question 3).
9. Forge preview (`forge_preview.gd` line `s.level <= profile.skill_cap`): list skills with `can_use`-style checks (owned and `tier_unlocked`) and show the first castable tier and unlock level; header "Skills" without a cap number.
10. `UnitState.skill_cap()`: remove, or return `StatRules.MAX_LEVEL`-free data: callers (progression UI) use `Combatant.family_unlock_level(family)` (new, returns `E(f)`) instead. `Combatant.family_unlock_level` is the public read for UI tooltips.

### Acceptance checks (rule-derived; read AP, levels and counts from the catalog, never hard-code)

| Check | Derivation |
| --- | --- |
| Riposte opens at 3 | warrior level 3 unit: `owned_families()` contains `riposte`, `can_use(riposte)` true; level 2: false |
| Riposte tiers are siblings | with the asset seed, riposte_2 and riposte_3 are castable at the same level as T1 (offset 0) |
| T1 AP no longer gates | for every shipped class path step, a unit at that exact level `can_use` the T1 row regardless of its AP (including AP 3 at level 3) |
| Tier steps | for a family with U = u and derived offsets: tier 2 castable iff L >= u + 2, tier 3 iff L >= min(10, u + 4); checked at L = u + 1, u + 2, u + 4 |
| Old pacing kept | taunt_shout (U 1) at warrior level 5: tiers 1-3 castable, same as the old cap 3 allowed |
| Advanced floor | validation rejects an advanced family at path level 4 and accepts 5; rejects mixed `advanced` values within a family |
| Passive unlock | a passive `unlock` family greater_mend levels 2 on a priest of level 3: owned; level 2: not owned; levels 2 passive on an ordinary family at U 3: E = 1; on an advanced family at U 5: E = 3 (floor) |
| Cap | two passives (levels 2 and 2) on one family still give R 2; passive 2 plus KNOW 2 still R 2 |
| KNOW | priest with KNOW 16 owns greater_mend at level 4 and not at 3; KNOW 20 at level 3; KNOW 15 gains nothing; KNOW never lowers a non-advanced family |
| No grant | an `unlock` passive naming a family not on the class path changes nothing |
| Freeze | after `CombatSession.start`, changing KNOW or applying an unlock passive does not alter `known_skills` |
| Default loadout | the default 5 for every class at every level 1-10 equals the first 5 owned path families in path order; reductions never change the default when 5 are already owned |
| AP ceiling | a unit entry with `skill_cap` 2 still refuses AP 3 tiers; a derived unit with 0 has profile cap 5 |
| Planner | a level-3 warrior's plan offers riposte T1 (AP 3) as affordable when AP allows and as "not enough AP" otherwise, never as "isn't unlocked" |
| Tests encoding the old rule are replaced, not deleted | see Tests |

## Assets (asset lane, SK2-A; needs the SK2-L schema slice)

Seed script `data/seeds/sk2.gd` (idempotent, through `ContentCatalog.upsert`, in the style of `m2c2b_*.gd`).

### Skill rows

| Rows | Change |
| --- | --- |
| shield_wall, shield_wall_2, shield_wall_3 | `advanced` true |
| deadeye (3 rows) | `advanced` true |
| greater_mend, renewal (3 rows each) | `advanced` true |
| judgement (3 rows) | `advanced` true |
| charge_in (3 rows) | `advanced` true |
| cyclone_kick (3 rows) | `advanced` true |
| lacerate, assassinate (3 rows each) | `advanced` true |
| glacial_burst, inferno (3 rows each) | `advanced` true |
| riposte_2, riposte_3 | `unlock_offset` 0 (their AP 2 / 1 and mitigation 0.05 / 0.00 make them cheaper-but-riskier sidegrades, not upgrades, per DECISIONS "Riposte tuning"); riposte T1 stays derived 0 |
| every other row | no change (offset stays -1, advanced false) |

### Class paths (`classes.json`)

Only one row moves (every other shipped row is valid under the new rules and keeps its level; no rebalance is attempted here, that is BAL1):

| Class | Family | Old level | New level | Why |
| --- | --- | --- | --- | --- |
| warrior | riposte | 5 | 3 | The reason for this task; the 2 x AP - 1 tie is gone. Warrior default 5 (taunt_shout, shield_bash, intimidate, riposte, sunder) is unchanged as a set; riposte moves ahead of intimidate (4) in LEVEL, so move the row to keep the list ascending: order becomes taunt_shout 1, shield_bash 3, riposte 3, intimidate 4, sunder 5, shield_wall 5 |

Path order consequence: the default loadout is by path order, so the warrior default stays taunt_shout, shield_bash, riposte, intimidate, sunder (M2C2B text listed riposte second; the order inside the default is cosmetic). Advanced rows already at 5 or 7 satisfy `ADVANCED_MIN_LEVEL`: shield_wall 5, deadeye 5, greater_mend 5, renewal 5, charge_in 5, cyclone_kick 5, lacerate 5, glacial_burst 5, judgement 7, assassinate 7, inferno 7.

Unit effects (verify against `units.json`): dojo_warrior (warrior 2) unchanged (riposte needs 3); warriors at 3-4 (if any) gain riposte T1 and shield_bash; heroes at 5 gain nothing new except riposte now castable at AP 3 from level 3 (it was castable from 5 already).

### Passive seed (small, so the effect is testable and usable by sources)

Add to `passives.json` (templates only; no source grants them yet, a world/discipline decides):

| Id | Name | Effect | Description |
| --- | --- | --- | --- |
| field_medic | Field Medic | `unlock` group `heal`, levels 1 | Opens healing families one level earlier. |
| arcane_study | Arcane Study | `unlock` group `spell`, levels 1 | Opens spell families one level earlier. |
| riposte_drill | Riposte Drill | `unlock` family `riposte`, levels 2 | Opens Riposte two levels earlier (level 1 for a warrior). |

No world data is changed.

## Tests (logic lane; rule-derived, no hard-coded numbers)

| Suite | Change |
| --- | --- |
| `class_rules_test_suite.gd` | line ~170 "paths respect the level cap" is replaced by "advanced steps >= ADVANCED_MIN_LEVEL, one step per family, tier-1 rows only"; "path unlocks by level" kept (still true); add warrior riposte at 3 |
| `content_test_suite.gd` | new validation cases (advanced mismatch, advanced below 5, bad offset, duplicate family, tier-2 row in a path); the fixture `skill_cap: 2` keeps meaning an AP ceiling; add `unlock` passive validation |
| `skill_family_test_suite.gd` | the check "tiers offered = those with AP level <= cap" (line ~148) is rewritten: offered tiers = tiers with `tier_unlocked` true and AP within the explicit ceiling; fixture units set levels |
| `riposte_test_suite.gd` | warrior built at level 3 uses riposte; tiers 2 and 3 castable with T1 |
| new `skill_unlock_test_suite.gd` | the acceptance table above (one `results.append` line in `suite_registry.gd`) |
| `foundation_test_suite.gd`, `world_test_suite.gd`, `runner_test_suite.gd` | run and fix only expectations that assumed riposte at 5 (derive from the class path) |

## Out of scope

Rebalancing any other unlock level, XP curve, tier numbers (BAL1); granting the new passives from worlds, disciplines or items; the skill-choice / loadout UI; adding the AI use of any skill (AI1); changing slots (5 + 3); per-world unlock tables; `learned_only` rules; a KNOW growth change.

## Open questions

1. KNOW thresholds (base 12, step 4, max 2) mean the priest (KNOW 15 at start, growing) reaches the -1 band within a few levels, the mage (12) needs about +4, the warrior (10) needs +6 spent. Intended (healers and casters get advanced skills slightly earlier), or should the reduction be class-relative so every class needs the same investment?
2. Combine passive and KNOW as max (chosen, conservative) or sum under the cap of 2? Sum lets a KNOW-heavy unit with a passive reach the cap sooner.
3. Should `ContentCatalog` loadout validation accept a family that is owned only through a passive/KNOW reduction (needs a world-level check that sees granted passives), or stay on unreduced path levels as specified?
4. Tier step 2 / 2 reproduces the old pacing; keep, or make T3 land earlier (for example 0 / 2 / 3) so late-game tier 3 is more reachable below level 9-10 for advanced families (advanced at 5 gets T3 at 9; at 7 it is clamped to 10)?
