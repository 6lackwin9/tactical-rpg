# M2C2B: Class skill content for all six classes (final model)

Archived 2026-10-17 (CLN-5), task done: kept for history; current rules live in docs/GAME_BRIEF.md and docs/decisions/. Do not build from this file.

Status: done (header was "proposed")
Depends on: B1, C1, D1 (per asset, see the dependency column); engine gaps X1 and X2 (section "Engine gaps") for four advanced skills
Lane: asset (plus the small logic prerequisite M2C2B-L and the test updates, logic lane)

## Goal

Author the class skill content under the final skill model: every class family has explicit tiers 1-3, advanced families are separate families (base AP 3), basics are the six single-tier skills, the default active loadout is the first 5 class families of the path (max 5 class families + 3 extra, basics free), the unarmed token is used by the Brawler, the Mage has no volley, and the Warrior's default 5 are taunt_shout, riposte, shield_bash, intimidate, sunder. Source: `docs/archive/M2C2_SKILL_DRAFT.md (archived)` v2 plus Revision 3 with the 2026-10-15/16 answers applied (DECISIONS.md rows 'M2c-2b design answers', 'Basics v3', 'Weapon hands', 'Riposte tuning', 'Status registry', 'Tier numbers', 'Slice A foundation'). All numbers are starting values for the balance pass (BAL1); no sims were run.

## Schema contract (logic lane)

No new skill fields here. This spec uses what exists plus D1 (`status_magnitude`, zero-power attacks, `weaken`) and C1 (caster gating). Everything below is expressible with current fields EXCEPT the gaps in "Engine gaps".

Validation reminders for the asset lane (existing rules):

- `level` is the AP cost, 1-5. `tier` 1-3 for class skills. `family` = the T1 id; tier ids are `<id>_2`, `<id>_3`.
- Tier numbers are authoring defaults (T2 +1 AP x1.25, T3 +2 AP x1.5, NO extra block at T3); every tier row carries its own explicit numbers so a family may deviate (several below do, noted).
- Blocks are constant across the tiers of a family.
- Unit skill cap = ceil(level/2): level 1-2 cap 1, 3-4 cap 2, 5-6 cap 3, 7-8 cap 4, 9-10 cap 5. A tier is usable when its AP is within the cap.
- Path unlock level rule used: a family unlocks no earlier than 2 x (T1 AP) - 1 (AP 1 at level 1, AP 2 at 3, AP 3 at 5, AP 4 at 7), so a family never sits unusable in the loadout.

## Rules (logic lane)

1. Default active loadout = the first 5 non-basic families reached by unit level in path order (list order breaks ties at the same level); the rest are owned but inactive (existing M2c-2a1 rule). Every class below has more than 5 families only after the advanced ones are added; the first five rows are the default.
2. Basics stay in each class's `skill_path` exactly as shipped (strike and guard at level 1, plus offhand_jab where present, mage `strike`/`guard`); they never count toward the 5. Do not touch them.
3. Removing `volley` from the mage path and `lunge` from the rogue path is a data change only.

## Assets (asset lane)

Dependency legend: A = independent, can start now; C1 = needs C1-A1 merged (gating edits on hex / arcane_bolt); D1 = needs D1-A; B1 = no asset depends on B1 (tests only); X1/X2 = needs the engine gaps.

Common defaults for every row unless stated: `area` NONE, `offhand` false, `stance` AUTO, `dash_tiles` 0, `status_rounds` 1, `reach` 0/0, `weapon_types` [], `min_proficiency` 0, `basic` false, `learned_only` true for NEW families, existing families keep their shipped flag (shield_bash, hamstring, sunder, aimed_shot, pinning_shot, volley, arcane_bolt, hex, focus stay false; mend, rally, taunt_shout, lunge, bleeding_cut, purify, backflip, sweep_kick stay true). Every row sets `family` (the T1 id) and `tier`. Names: tier 1 "Name", then "Name II", "Name III". Descriptions state the tier's numbers.

### 1. Attack families (class)

Cells read id AP / blocks / power / status. "R" = reach min-max.

| Family | T1 id | T2 id | T3 id | AP (T1/T2/T3) | Blocks | power_mult (T1/T2/T3) | Area, R, dash | Status: potency / rounds (T1/T2/T3) | weapon_types, min_prof | Dep |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| shield_bash | shield_bash | shield_bash_2 | shield_bash_3 | 2 / 3 / 4 | 2 | 0.80 / 1.00 / 1.20 | NONE, 0-0; stance DEFENSIVE | stagger 70 / 75 / 80, rounds 1 | none | A |
| sunder | sunder | sunder_2 | sunder_3 | 2 / 3 / 4 | 2 | 1.20 / 1.50 / 1.80 | NONE | stagger 50 / 55 / 60, rounds 1 | axe, mace, hammer, p2 | A |
| hamstring | hamstring | hamstring_2 | hamstring_3 | 2 / 3 / 4 | 2 | 0.80 / 1.00 / 1.20 | NONE | root 80 / 85 / 90, rounds 1 / 1 / 2 | none | A |
| aimed_shot | aimed_shot | aimed_shot_2 | aimed_shot_3 | 2 / 3 / 4 | 2 | 1.40 / 1.75 / 2.10 | NONE | none | bow, crossbow, p2 | A |
| pinning_shot | pinning_shot | pinning_shot_2 | pinning_shot_3 | 2 / 3 / 4 | 2 | 0.80 / 1.00 / 1.20 | NONE | root 80 / 85 / 90, rounds 1 / 1 / 2 | bow, crossbow, p1 | A |
| barbed_arrow | barbed_arrow | barbed_arrow_2 | barbed_arrow_3 | 2 / 3 / 4 | 2 | 0.70 / 0.90 / 1.05 | NONE | bleed 60 / 65 / 70, rounds 2 | bow, crossbow, p2 | A |
| volley (ranger only) | volley | volley_2 | volley_3 | 2 / 3 / 4 | 2 | 0.70 / 0.90 / 1.05 | PLUS, R 2-3 | none | bow, crossbow, p1 | A |
| lunge | lunge | lunge_2 | lunge_3 | 2 / 3 / 4 | 2 | 1.20 / 1.50 / 1.80 | NONE, R 1-1, dash 3 / 3 / 4 | none | none | A |
| sweep_kick | sweep_kick | sweep_kick_2 | sweep_kick_3 | 2 / 3 / 4 | 2 | 0.70 / 0.90 / 1.05 | SWEEP | none | none (unarmed) | A |
| body_blow | body_blow | body_blow_2 | body_blow_3 | 2 / 3 / 4 | 2 | 1.00 / 1.25 / 1.50 | NONE | stagger 50 / 55 / 60, rounds 1 | fist, mace, unarmed, p1 | A |
| haymaker | haymaker | haymaker_2 | haymaker_3 | 3 / 4 / 5 | 3 | 2.00 / 2.50 / 3.00 | NONE | stun 50 / 55 / 60, rounds 1 | fist, mace, hammer, unarmed, p2 | A |
| pounce (rogue) | pounce | pounce_2 | pounce_3 | 2 / 3 / 4 | 2 | 0.80 / 1.00 / 1.20 | NONE, R 1-1, dash 3 / 3 / 4 | none | dagger, p1 | A |
| quick_stab | quick_stab | quick_stab_2 | quick_stab_3 | 1 / 2 / 3 | 1 | 0.60 / 0.75 / 0.90 | NONE | none | dagger, p1 | A |
| bleeding_cut | bleeding_cut | bleeding_cut_2 | bleeding_cut_3 | 2 / 3 / 4 | 2 | 0.80 / 1.00 / 1.20 | NONE | bleed 70 / 75 / 80, rounds 2 | none | A |
| smite (priest) | smite | smite_2 | smite_3 | 1 / 2 / 3 | 2 | 0.80 / 1.00 / 1.20 | NONE, R 2-3 | none | mace, staff, p1 | A (mace gating: open question 1 of C1) |
| hex | hex | hex_2 | hex_3 | 2 / 3 / 4 | 2 | 0.60 / 0.75 / 0.90 | NONE, R 2-3 (T1 was 0-0) | stagger 90 / 95 / 95, rounds 1 | mace, staff, wand, p2 | C1 |
| arcane_bolt | arcane_bolt | arcane_bolt_2 | arcane_bolt_3 | 1 / 2 / 3 | 2 | 1.10 / 1.40 / 1.65 | NONE, R 0-0 | none | staff, wand, p1 | C1 |
| arc_lance (mage) | arc_lance | arc_lance_2 | arc_lance_3 | 2 / 3 / 4 | 2 | 0.90 / 1.10 / 1.35 | LINE, R 0-0 (aims at an adjacent tile, like shockwave) | none | staff, wand, p1 | C1 |
| firebomb | firebomb | firebomb_2 | firebomb_3 | 3 / 4 / 5 | 3 | 0.60 / 0.75 / 0.90 | BURST, R 2-2 | none | staff, wand, p1 (T1 was ungated) | C1 |

Notes: volley and aimed_shot keep their shipped gating style (T1 volley was ungated; ranger-only now). Mage `volley` is removed from the mage path. Orphan shipped skills `shockwave` and `reckless_swing` stay as they are and are in no path.

### 2. Advanced attack families (separate families, base AP 3)

Each is a new family with explicit tiers; it improves a starter family in one slot (extra effect in the status column). Blocks fixed across tiers.

| Family | Improves | T1 / T2 / T3 id | AP | Blocks | power_mult | Area, R, dash | Extra effect | weapon_types, min_prof | Dep |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| deadeye | aimed_shot | deadeye / deadeye_2 / deadeye_3 | 3 / 4 / 5 | 3 | 2.00 / 2.50 / 3.00 | NONE, 0-0 | stagger 50 / 55 / 60, rounds 1 | bow, crossbow, p3 | A |
| judgement | smite | judgement / judgement_2 / judgement_3 | 3 / 4 / 5 | 3 | 1.20 / 1.50 / 1.80 | NONE, R 2-3 | stagger 40 / 45 / 50, rounds 1 | mace, staff, p2 | A |
| charge_in | lunge | charge_in / charge_in_2 / charge_in_3 | 3 / 4 / 5 | 3 | 1.40 / 1.75 / 2.10 | NONE, R 1-1, dash 4 / 4 / 4 | stagger 60 / 65 / 70, rounds 1 | none | A |
| lacerate | bleeding_cut | lacerate / lacerate_2 / lacerate_3 | 3 / 4 / 5 | 2 | 1.20 / 1.50 / 1.80 | NONE | bleed 90 / 95 / 100, rounds 2 | dagger, p2 | A |
| assassinate | quick_stab | assassinate / assassinate_2 / assassinate_3 | 3 / 4 / 5 | 3 | 2.00 / 2.50 / 3.00 | NONE | none (the extra is power per slot) | dagger, p3 | A |
| cyclone_kick | sweep_kick | cyclone_kick / cyclone_kick_2 / cyclone_kick_3 | 3 / 4 / 5 | 3 | 0.90 / 1.10 / 1.30 | SWEEP | stagger 50 / 55 / 60 on every enemy hit, rounds 1 | none (unarmed) | X1 |
| inferno | firebomb | inferno / inferno_2 / inferno_3 | 4 / 5 / 5 | 3 | 0.80 / 1.00 / 1.20 | BURST, R 2-3 | bleed 50 / 60 / 70, rounds 2, on every enemy hit | staff, wand, p2 | C1 + X1 |
| glacial_burst | hex | glacial_burst / glacial_burst_2 / glacial_burst_3 | 3 / 4 / 5 | 3 | 0.60 / 0.75 / 0.90 | BURST, R 2-3 | root 60 / 70 / 80, rounds 1, on every enemy hit | staff, wand, p2 | C1 + X1 |

### 3. Non-attack families

| Family | Kind | Ids (T1 / T2 / T3) | AP | Blocks | Numbers per tier (T1 / T2 / T3) | Target, reach | Dep |
| --- | --- | --- | --- | --- | --- | --- | --- |
| taunt_shout | SUPPORT | taunt_shout / taunt_shout_2 / taunt_shout_3 | 1 / 2 / 3 | 1 | grant taunt; status_rounds 2 / 3 / 3 | SELF | A |
| riposte | DEFEND | riposte / riposte_2 / riposte_3 | 1 / 2 / 3 | 2 | mitigation 0.15 / 0.20 / 0.25; grant riposte; rounds 2 / 3 / 3 | SELF | D1 (rows authored in D1-A) |
| intimidate | ATTACK (no damage) | intimidate / intimidate_2 / intimidate_3 | 1 / 2 / 3 | 1 | weaken magnitude 15 / 20 / 25, chance 70 / 75 / 80, rounds 2 / 2 / 3 | R 1-2 / 1-2 / 1-3 | D1 (rows authored in D1-A) |
| shield_wall (advanced) | DEFEND | shield_wall / shield_wall_2 / shield_wall_3 | 3 / 4 / 5 | 3 | mitigation 0.35 / 0.45 / 0.55; grant taunt, status_rounds 1 / 2 / 3; no weapon gating (a shield requirement is not expressible on DEFEND) | SELF | X2 |
| focus | FOCUS | focus / focus_2 / focus_3 | 2 / 3 / 4 | 2 / 2 / 1 (T3 drops to 1 block: user decision 2026-10-16, since T3 adds no boost) | boost 1.50 all tiers (keeps the shipped T1 and the +50% ceiling); boost_rounds 2 / 3 / 3 | SELF | A (replaces deep_focus, already removed) |
| mend | SUPPORT | mend / mend_2 / mend_3 | 2 / 3 / 4 | 2 | heal_stat KNOW; heal_base 8 / 10 / 12; heal_mult 1.5 / 1.8 / 2.1 | ALLY_OR_SELF, R 0-0 (default 0-3) | A |
| purify | SUPPORT | purify / purify_2 / purify_3 | 2 / 3 / 4 | 1 | cleanse ["stun"] / ["stun", "bleed"] / ["all"] (T1 was `all`: see open question 2) | ALLY_OR_SELF, R 0-0 | A |
| rally | SUPPORT | rally / rally_2 / rally_3 | 1 / 2 / 3 | 1 | grant haste; status_rounds 1 / 2 / 2 | ALLY; R 0-0 / 0-0 / 1-4 (+1 reach at T3) | A |
| greater_mend (advanced) | SUPPORT | greater_mend / greater_mend_2 / greater_mend_3 | 3 / 4 / 5 | 3 | heal_stat KNOW; heal_base 10 / 14 / 18; heal_mult 1.8 / 2.0 / 2.2; cleanse ["stun"] / ["all"] / ["all"]; T3 also grant haste, rounds 1 | ALLY_OR_SELF, R 0-0 | A |
| renewal (advanced) | SUPPORT | renewal / renewal_2 / renewal_3 | 3 / 4 / 5 | 2 | heal_stat KNOW; heal_base 6 / 8 / 10; heal_mult 1.0 / 1.2 / 1.4; grant haste, status_rounds 1 / 1 / 2 | ALLY, R 0-0 | A |
| leap_back (ranger) | DISENGAGE | leap_back / leap_back_2 / leap_back_3 | 2 / 3 / 4 | 1 | jump_tiles 2 / 3 / 3 | - | A |
| backflip (rogue) | DISENGAGE | backflip / backflip_2 / backflip_3 | 2 / 3 / 4 | 1 | jump_tiles 3 / 3 / 3 (T1 shipped as 3) | - | A |

Notes: `heal_*` and `cleanse` values come from the Revision 3 baseline (cleanse is never scaled by magnitude; AP rises by the standard rule; heal below the "whole heal x1.25 / x1.5" ceiling on purpose). `purify`'s shipped T1 cleanses `all`; `engine_pieces` tests that assume it must be updated. `all` includes `weaken` (D1).

### 4. Basics (unchanged by this spec)

The six single-tier basics are exactly strike, guard, move, rest, switch_target, offhand_jab (`basic` true). Do not add or edit variants; late scaling comes from the `skill_potency` hook.

### 5. Class `skill_path` (level, skill)

Basics rows stay as shipped and are omitted. Each row names the family (T1 id). The first five rows in order are the default loadout. Phase 1 = no engine gap needed; Phase 2 rows (marked X) are appended after the gaps are built. The class JSON must be edited through `ContentCatalog.upsert`.

| Class | Level | Family (T1 id) | Phase | Dep | Default? |
| --- | --- | --- | --- | --- | --- |
| warrior | 1 | taunt_shout | 1 | A | yes |
| warrior | 3 | riposte | 1 | D1 | yes |
| warrior | 3 | shield_bash | 1 | A | yes |
| warrior | 4 | intimidate | 1 | D1 | yes |
| warrior | 5 | sunder | 1 | A | yes |
| warrior | 5 | shield_wall | 2 | X2 | no (swap for taunt_shout) |
| ranger | 3 | aimed_shot | 1 | A | yes |
| ranger | 3 | pinning_shot | 1 | A | yes |
| ranger | 3 | leap_back | 1 | A | yes |
| ranger | 5 | barbed_arrow | 1 | A | yes |
| ranger | 5 | volley | 1 | A | yes |
| ranger | 5 | deadeye | 1 | A | no (swap for aimed_shot) |
| ranger | 5 | focus | 1 | A | no (swap for volley) |
| priest | 1 | smite | 1 | A | yes |
| priest | 1 | rally | 1 | A | yes |
| priest | 3 | mend | 1 | A | yes |
| priest | 3 | purify | 1 | A | yes |
| priest | 5 | hex | 1 | C1 | yes |
| priest | 5 | greater_mend | 1 | A | no (replaces mend + purify) |
| priest | 5 | renewal | 1 | A | no (replaces mend + rally) |
| priest | 7 | judgement | 1 | A | no (replaces smite) |
| brawler | 3 | lunge | 1 | A | yes |
| brawler | 3 | body_blow | 1 | A | yes |
| brawler | 3 | sweep_kick | 1 | A | yes |
| brawler | 4 | hamstring | 1 | A | yes |
| brawler | 5 | haymaker | 1 | A | yes |
| brawler | 5 | charge_in | 1 | A | no (replaces lunge) |
| brawler | 5 | cyclone_kick | 2 | X1 | no (replaces sweep_kick) |
| rogue | 1 | quick_stab | 1 | A | yes |
| rogue | 3 | pounce | 1 | A | yes |
| rogue | 3 | bleeding_cut | 1 | A | yes |
| rogue | 4 | backflip | 1 | A | yes |
| rogue | 5 | hamstring | 1 | A | yes |
| rogue | 5 | lacerate | 1 | A | no (replaces bleeding_cut) |
| rogue | 7 | assassinate | 1 | A | no (replaces quick_stab) |
| mage | 1 | arcane_bolt | 1 | C1 | yes |
| mage | 3 | arc_lance | 1 | C1 | yes |
| mage | 3 | hex | 1 | C1 | yes |
| mage | 4 | focus | 1 | A | yes |
| mage | 5 | firebomb | 1 | C1 | yes |
| mage | 5 | glacial_burst | 2 | C1 + X1 | no (replaces hex) |
| mage | 7 | inferno | 2 | C1 + X1 | no (replaces firebomb) |

Removed from shipped paths: mage `volley`; mage `hex` moves from 4 to 3 and `focus` from 3 to 4; rogue `lunge` (replaced by pounce) and `hamstring` 4 to 5; brawler `hamstring` stays at 4; ranger `aimed_shot` 4 to 3 and `focus` stays 5; priest `rally` 2 to 1 and `focus` removed; warrior `taunt_shout` 2 to 1.

Default loadout per class (derived from the table): warrior taunt_shout, riposte, shield_bash, intimidate, sunder; ranger aimed_shot, pinning_shot, leap_back, barbed_arrow, volley; priest smite, rally, mend, purify, hex; brawler lunge, body_blow, sweep_kick, hamstring, haymaker; rogue quick_stab, pounce, bleeding_cut, backflip, hamstring; mage arcane_bolt, arc_lance, hex, focus, firebomb.

### 6. Unit migration

Kits come from the class default by unit level (cap in brackets). Verify each unit's class in `units.json` before editing (classes below are from the shipped roster notes); no unit stores a skill list, so no per-unit data edit is needed unless noted.

| Unit | Class, level (cap) | Class families after migration |
| --- | --- | --- |
| warrior (hero) | warrior, 5 (3) | taunt_shout, riposte, shield_bash, intimidate, sunder |
| ranger (hero) | ranger, 5 (3) | aimed_shot, pinning_shot, leap_back, barbed_arrow, volley |
| dojo_warrior | warrior, 2 (1) | taunt_shout (riposte and shield_bash need level 3; same as before for riposte) |
| dojo_ranger | ranger, 1 (1) | none (aimed_shot now unlocks at 3; it had 4 before) |
| goblin_brawler, goblin_berserker, enemy_brawler | brawler, 3 (2) | lunge, body_blow, sweep_kick (new: sweep_kick) |
| ambush_berserker | brawler, 4 (2) | lunge, body_blow, sweep_kick, hamstring |
| goblin_skirmisher, fleeing_scout | rogue, 3 (2) | quick_stab, pounce, bleeding_cut (lunge dropped) |
| goblin_archer | ranger, 3 (2) | aimed_shot, pinning_shot, leap_back (loses volley until level 5) |
| goblin_shaman | priest, 3 (2) | smite, rally, mend, purify |
| training_dummy | per its class, fixed | no change expected; verify |

Behaviour changes to flag for the planner: the AI does not yet use DISENGAGE, SWEEP or cleanse-only skills (AI1), so leap_back, sweep_kick and purify on minions are inert until AI1; minions mostly play tier 1.

### 7. Tests needing rule-derived updates (logic lane)

Read counts, ids, AP and numbers from the catalog or the class rules; do not hard-code.

| Suite | What changes |
| --- | --- |
| `class_rules_test_suite.gd` | path unlock levels; 5-family default; priest has no shield (C1); warrior path |
| `skill_family_test_suite.gd` | family/tier/AP and planner grouping for every family above; family-less ids; basics are exactly six |
| `content_test_suite.gd` | skill count; validation of new rows; `arcane_bolt`/`hex` gating no longer names orb or tome |
| `engine_pieces_test_suite.gd` | `purify` T1 now stun-only; `backflip` / `sweep_kick` are tier 1 of families |
| `combat_extensions_test_suite.gd` | mend, rally, lunge, taunt values per tier; taunt rounds |
| `riposte_test_suite.gd` | T1 AP is 1 (D1); tiers 2 and 3 (mitigation 0.20 / 0.25, rounds 3) |
| `gear_budget_test_suite.gd` | mage and priest loadouts with the C1 off-hand items |
| `arena_test_suite.gd`, `runner_test_suite.gd` | skills are loadout-driven; kits per the migration table |
| `world_test_suite.gd` | minion kits (table above); dojo checks that assume the old warrior or ranger kit |
| `foundation_test_suite.gd` | unchanged unless a path edit breaks the basics checks |

### 8. Asset phases (for the asset lane)

1. Phase A (now, independent): rows with Dep A in sections 1 to 3 and the matching path rows for warrior (without riposte/intimidate), ranger, brawler (without cyclone_kick), rogue, priest (without hex).
2. After C1-A1: hex, arcane_bolt, arc_lance, firebomb families, the priest `hex` path row, the whole mage path (phase 1).
3. After D1-A: the warrior riposte and intimidate path rows (their skill rows belong to D1-A).
4. After M2C2B-L (X1, X2): cyclone_kick, inferno, glacial_burst, shield_wall and their phase 2 path rows.
5. Last: unit/test updates, then BAL1.

## Engine gaps (logic lane, task `M2C2B-L`)

| Id | Gap | Needed by | Spec |
| --- | --- | --- | --- |
| X1 | Area attacks (`AreaAttackCommand`) apply no status today; only single-target `AttackCommand` calls `StatusEffects.try_inflict` | cyclone_kick (SWEEP), inferno, glacial_burst (BURST) | Each enemy hit by an area attack rolls the skill's `status_effect` / `status_potency` / `status_rounds` independently with the same resistance rules and log line; allies hit by friendly fire roll as well (consistent with friendly fire damage); acceptance: seeded rng, one roll per hit target, resisted/applied log per target |
| X2 | A DEFEND skill cannot grant `taunt` (DEFEND grants are limited to `riposte`, registry grant `defend`), and `weapon_types` / `min_proficiency` are ATTACK-only so a "needs a shield" gate on shield_wall is not expressible | shield_wall | Allow a DEFEND skill whose `grant_status` is a `support`-grant buff (taunt): it is queued for next round per the registry while the mitigation applies this round; validation allows `riposte` or a support-grant buff; shield gating is dropped (not built) |

Phase 1 content does not depend on either gap.

## Out of scope

- Balance numbers beyond these starting values, BAL1, AI behaviour (AI1), new engine pieces other than X1/X2, art, the `shockwave` / `reckless_swing` rework, a mage PLUS-pattern family (parked), progression/save (M3), any per-world skill data.
- The orb/tome item stats (C1) and the weaken/intimidate/riposte rows (D1).

## Open questions

1. Priest damage skills (smite, hex, judgement) allow a `mace` main hand (the priest default) in addition to staff/wand. Confirm; the alternative is giving the priest a one-handed wand and gating strictly on staff/wand (see C1 open question 1).
2. `purify` T1 shrinks from "all debuffs" to stun-only (Revision 3: T1 one type, T2 two, T3 all). Accept the nerf of the shipped skill, or keep T1 = all and scale something else (reach, or a heal-free second target)?
3. Focus tiers keep boost 1.5 and scale only the duration (2 / 3 / 3 rounds) to respect the +50% ceiling; the draft's "+1 step per tier" would exceed it. Accept?
