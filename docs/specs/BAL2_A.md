# BAL2-A: growth and gear pass, L1 class skills, party loadouts, goblin shaman, brawler unarmed

Status: spec (design-brainstorm 2026-10-17; user answers in DECISIONS "BAL2 answers (user 2026-10-17)"; awaiting approval). Section 6 (brawler compensation) is awaiting the user's choice; sections 2-5 can be built without it.
Depends on: BAL2-L commit 3 (unit gear `none` and gear-independent tests). Commit A6 also needs the brawler compensation the user picks in section 6 (and its logic commit if it has one).
Lane: asset
Touches: new `data/seeds/bal2.gd`; `data/catalog/classes.json`, `data/catalog/units.json`; `data/catalog/items.json` only for `arm_plate` (Q-A1 option c)

## Goal

Apply the accepted starting values: every damage class grows CONS, the warrior trades the tower shield for a kite shield and grows STR, ranger and brawler get a class skill at level 1, the shipped party units carry explicit role loadouts, the goblin shaman gets its own kit and a correct description, and (after section 6) the brawler fights bare-handed.

## 1. Seed contract

- `data/seeds/bal2.gd`, `extends SceneTree`, same pattern as `data/seeds/sk2.gd`: `ContentCatalog.reload_from_disk()`, every edit through `ContentCatalog.upsert` on a `duplicate(true)` of the current entry, system entries edited inside `ContentCatalog.begin_system_edit()` / `end_system_edit()`, all errors collected and printed, idempotent (re-running changes nothing). Run: `tools/fc.sh run data/seeds/bal2.gd`.
- One function per commit below (`_a1_growth`, `_a2_warrior`, `_a3_paths`, `_a4_loadouts`, `_a5_shaman`, `_a6_brawler`); `_a6_brawler` stays disabled (not called) until the user picks section 6.
- No test edits in this lane. If a test fails because it pins a number this pass changes, stop and report it to the planner (logic-lane fix).

## 2. Classes (commits A1-A3)

### A1 growth

`growth` replaces the whole dict; key order as written (it is the tie order of `StatRules.auto_spend`). Each total is 5, no stat above 3 (`StatRules` validation unchanged). Priest and brawler unchanged.

| Class | Growth now | Growth new |
| --- | --- | --- |
| warrior | CONS 3, STR 1, PERC 1 | CONS 2, STR 2, PERC 1 |
| ranger | DEX 2, PERC 2, STR 1 | DEX 2, PERC 1, STR 1, CONS 1 |
| rogue | DEX 3, PERC 1, STR 1 | DEX 2, PERC 1, STR 1, CONS 1 |
| mage | INT 3, KNOW 1, PERC 1 | INT 2, CONS 1, KNOW 1, PERC 1 |

### A2 warrior gear

| Field | Now | New |
| --- | --- | --- |
| `off_hand` | `shd_tower` | `shd_kite` (mitigation 8, burden 2, block cost 1, STR 12 required; warrior STR 13) |
| `description` | "Tank: huge constitution, tower shield and plate; a wall that holds the line and hits lightly." | "Tank: huge constitution, kite shield and plate; a wall that holds the line and hits lightly." |

`shd_tower` itself is unchanged (stays a special reward item; the gear budget test that compares tower and kite keeps passing).

Gear budget consequence: hand axe 16 + kite 8 + plate 9 = 33.0, above the band 18.4-27.6 with no special item, so `GearBudget.check_class` warns and `engine_pieces_test_suite` "warnings for the shipped classes stay empty" fails. Q-A1 decides the fix; the default below keeps the accepted armor numbers:

- Q-A1 default (option c): `arm_plate` gets `special: true` and `special_note`: "Heavy plate: the best body armor, but burden 3 slows the wearer; the tank warrior trades offense and mobility for it." Data only; the warrior loadout (33.0, 9.0 special) is then excused as before.

### A3 class paths

The whole `skill_path` list is written in this order (the default loadout follows path order).

| Class | New `skill_path` (level, family) | Change |
| --- | --- | --- |
| ranger | 1 aimed_shot, 3 pinning_shot, 3 leap_back, 5 barbed_arrow, 5 volley, 5 deadeye, 5 focus | aimed_shot 3 -> 1 |
| brawler | 1 body_blow, 3 lunge, 3 sweep_kick, 4 hamstring, 5 haymaker, 5 charge_in, 5 cyclone_kick | body_blow 3 -> 1, moved first |

With the SK2 offsets (0/2/4) aimed_shot and body_blow tiers 1/2/3 become castable at levels 1/3/5 (were 3/5/7). This also clears the "missing level-1 path skill" warning for both classes.

## 3. Expected derived numbers (rule-derived checks for the asset-builder)

Hero, empty spend, class gear, after A1-A3 and BAL2-L commit 2 (measured by a sandbox probe on main c3520b4 with these values; HP = 60 + 8 CONS + 2 STR; EHP = HP / (1 - armor / (armor + 50))).

| Class | HP L1/L3/L5/L7 | Armor L5 | EHP L5 (now) | Power L1 -> L7 | Range L5 | Gear budget |
| --- | --- | --- | --- | --- | --- | --- |
| warrior | 242/284/334/382 | 24 | 494 (597) | 41 -> 62 melee | 1 | 33.0 (special plate) |
| ranger | 158/178/200/228 | 8 | 232 (180) | 52 -> 73 ranged | 4 (PERC 19; was 4 from L3) | 24.0 |
| priest | 176/192/208/232 | 5 | 229 (229) | 34 -> 45 melee | 1 | 26.5 |
| rogue | 152/172/194/222 | 8 | 225 (179) | 45 -> 66 melee | 1 | 19.0 |
| mage | 144/160/184/200 | 4 | 199 (146) | 50 -> 71 ranged | 3 (4 at L7) | 27.0 |
| brawler (mace, until A6) | 228/270/320/368 | 11 | 390 (390) | 44 -> 65 melee | 1 | 19.0 |

Acceptance (A1-A3): `validate_all()` is `{}`; `warnings_all()` lists no class (with Q-A1 resolved); every HP cell equals the formula from the built unit's CONS and STR; ranger and brawler L1 units own and have active `aimed_shot` / `body_blow`; `StatRules` growth checks pass.

## 4. Party unit loadouts (commit A4)

Each shipped L5 party unit gets an explicit `loadout` = its class's primary role list (`docs/specs/BAL2_SIM.md` section 6.3). Gear slots stay blank (class gear).

| Unit | Role | `loadout` | Differs from the default by |
| --- | --- | --- | --- |
| `warrior` | Tank | taunt_shout, shield_bash, riposte, intimidate, shield_wall | shield_wall instead of sunder |
| `ranger` | Ranged | aimed_shot, pinning_shot, leap_back, barbed_arrow, deadeye | deadeye instead of volley |
| `rogue` | Striker | quick_stab, pounce, lacerate, backflip, hamstring | lacerate instead of bleeding_cut |
| `mage` | AoE | arcane_bolt, arc_lance, firebomb, glacial_burst, focus | glacial_burst instead of hex |

Enemy units (levels 3-4) own at most five path families, so their default already equals an explicit list; they stay blank. Dojo units stay blank.

Acceptance (A4): each built unit's `active_class_families` equals its list; `_skill_loadout_errors` and `requires` checks report nothing.

## 5. Goblin shaman (commit A5)

`goblin_shaman` (priest, level 3, minion), via upsert:

| Field | Now | New |
| --- | --- | --- |
| `main_hand` | "" (class: wpn_cleric_mace) | `wpn_cleric_mace` |
| `off_hand` | "" (class: wpn_tome) | `none` |
| `body` | "" (class: arm_cleric_vestment) | `arm_robe` (cloth; priest cloth level 3) |
| `pouch` | [] (class: con_heal_potion) | `["none"]` |
| `description` | "Opens with Focus, then fights from range." | "Smites and mends its allies." |

`head`, `accessory_1`, `accessory_2` stay blank (the priest class has none). Acceptance (A5): the built unit has the cleric mace, no off-hand, `arm_robe`, an empty battle pouch, and the new description.

## 6. Brawler unarmed (commit A6): awaiting user choice

Decided (user 2026-10-17, decision 10): the brawler fights unarmed, in melee close combat, engaging several foes at once. Commit A6, data: brawler `main_hand` `wpn_flanged_mace` -> "" and `description` "Bruiser: bare fists and a thick hide; leaps into a crowd and fights several foes at once." (mace stays in the proficiency, so units may still carry one). Open: how to compensate the lost weapon power.

### 6.1 The gap (probe, main c3520b4, hero, empty spend)

| | L1 | L3 | L5 | L7 |
| --- | --- | --- | --- | --- |
| Mace power (17 + mace training 3 + 1.5 x STR) | 44 | 52 | 59 | 65 |
| Bare hands (STR + DEX) | 26 | 34 | 41 | 48 |
| Brawler duel row, mace (hp score, cap 10, 3 seeds) | 47 | 40 | 60 | 60 |
| Brawler duel row, bare hands | 33 | 41 | 40 | 70 |

The gap is a flat 17-18 power at every level. The L7 bare-hands 70 is one matchup flipping in a near-deterministic sim (why BAL2 judges on the new sim). Gear budget: bare hands count `GearBudget.UNARMED_VALUE` 14, so the unarmed brawler loadout is 14 + leather 4 = 18.0, below the band floor 18.4: A6 alone makes the shipped-class warning check fail. Any option must also fix that.

### 6.2 Options

- A. Trained fists (flat base). When the main hand is empty and the class's `fist` proficiency level is >= 2, bare-hand attacks get weapon power `UNARMED_BASE_POWER` 15 + fist training (level x `Proficiency.LEVEL_POWER`, brawler +3) and the fist crit bonus (+2%); unit power stays STR + DEX. Brawler power 44/52/59/66 (mace 44/52/59/65). `GearBudget.UNARMED_VALUE` becomes `float(DamageCalculator.UNARMED_BASE_POWER)` (15): loadout 19.0, in band. Off-hand bare-hand hits and classes without fist level 2 (rogue has 1) are unchanged. Effect: the brawler keeps its BAL1 strength without a weapon, and DEX growth matters (hybrid, DECISIONS "Brawler"). Cost: `damage_calculator.gd` (unarmed branch of `attack_power`, `crit_chance`), `gear_budget.gd` (one constant), `engine_pieces_test_suite` unarmed checks made rule-derived, 2-3 checks in `bal2_rules_test_suite`. Risk: an input change to the S1 formula (allowed: DECISIONS "Combat extensions" permits stat-input changes; the balance lane must rerun S1). HISTORY: nothing touches it.
- B. Fist scaling. Bare hands = (STR + DEX) x fist scaling (1.5) + fist training. Brawler 42/54/65/75: grows faster than any weapon (+6 at L5, +10 at L7 over the mace) because two stats grow. Needs the same `UNARMED_VALUE` raise for the budget. Cost as A. Risk: a late-game brawler outlier like the BAL1 L7 mage (80%).
- C. Hand wraps item. A new main-hand fist item `wpn_hand_wraps` (base 15, STR scaling) that skills treat as bare hands. Brawler 42/50/57/63, budget 19.0. Cost: asset item plus a gate change so `unarmed` skills accept it. Risk: it is a weapon, not bare hands (contradicts decision 10 and DECISIONS "Brawler unarmed"); DEX growth stops counting for damage; and the shipped `wpn_knuckles` is currently refused in the main hand ("Spiked Knuckles can't be equipped in Main Hand": it still carries `off_hand_category` `knuckles`, pending the FORGE1-ITEMS migration), so the fist item family needs fixing first.
- D. Crowd identity only. Raise the multi-target skills: `sweep_kick` power 0.7/0.9/1.05 -> 0.85/1.05/1.25 and `cyclone_kick` 0.9/1.1/1.3 -> 1.05/1.3/1.5 (data), optionally later a skill-granted "brawl" buff (+10% power per extra adjacent foe, max +30%; new engine effect; not a class passive, since passives never come from classes). Effect: the brawler wins swarm fights; nothing changes in single-target fights (the AI casts sweeps only on 2+ foes), so the -18 power gap stays. Does not fix the budget.

### 6.3 Recommendation

A, with base 15 and the fist level >= 2 gate: it closes the gap exactly at every level, puts the budget in band through one shared constant, and keeps the STR + DEX hybrid. Re-measure D after BAL2 with the `aoe_swarm` limit test (brawler's primary role is melee AoE): if the brawler misses the AoE KPIs, D is the data follow-up. If A is picked, the planner adds it to BAL2-L (or a small BAL2-L2 task) and A6 waits for it.

## 7. Out of scope

New items (except Q-A1 option c's flag on `arm_plate`, and C only if picked), skill number changes (except D if picked), enemy unit loadouts, world data, Forge, AI.

## 8. Open questions

1. Q-A1 warrior gear budget (33.0 vs band 27.6): (a) accept a soft warning and let the logic lane relax the "shipped classes stay warning-free" check (weakens the M2d guard), (b) shortsword + leather instead of axe + plate (26.0 in band; armor L5 about 18, EHP about 450, power -1), (c) `arm_plate` becomes a documented special (recommended; keeps the accepted armor 24 / EHP 494; plate is in no other class loadout).
2. Brawler compensation: A (recommended), B, C or D (6.2); for A, base 15 (exact mace parity) or 14 (today's `UNARMED_VALUE`, -1 power, budget 18.0 still below band)?
3. Should priest and brawler party units at L5 be added (priest and brawler have no shipped party unit; the M3 builder-equivalence test covers only the units that exist)? Not needed by the sim (it builds its own units).
