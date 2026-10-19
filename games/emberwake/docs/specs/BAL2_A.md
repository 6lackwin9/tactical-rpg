# BAL2-A: growth and gear pass, L1 class skills, party loadouts, goblin shaman, brawler unarmed

Status: spec (design-brainstorm 2026-10-17; user answers in DECISIONS "BAL2 answers" and "BAL2 role answers (user 2026-10-17)"; awaiting approval). Brawler compensation decided (option A, section 6). Section 7 (gear budget review) awaits the user's pick and gates commit A2 only.
Depends on: BAL2-L commit 3 (unit gear `none` and gear-independent tests). A6 also needs BAL2-L commit 5 (trained fists, `UNARMED_VALUE` 15). A2 needs the section 7 answer.
Lane: asset
Touches: new `data/seeds/bal2.gd`; `data/catalog/classes.json`, `data/catalog/units.json`; `data/catalog/items.json` for `shd_kite` and `arm_plate` (section 7 option D1, recommended)

## Goal

Apply the accepted starting values: every damage class grows CONS, the warrior trades the tower shield for a kite shield and grows STR, ranger and brawler get a class skill at level 1, the shipped party units carry explicit role loadouts, the goblin shaman gets its own kit and a correct description, and (after section 6) the brawler fights bare-handed.

## 1. Seed contract

- `data/seeds/bal2.gd`, `extends SceneTree`, same pattern as `data/seeds/sk2.gd`: `ContentCatalog.reload_from_disk()`, every edit through `ContentCatalog.upsert` on a `duplicate(true)` of the current entry, system entries edited inside `ContentCatalog.begin_system_edit()` / `end_system_edit()`, all errors collected and printed, idempotent (re-running changes nothing). Run: `tools/fc.sh run data/seeds/bal2.gd`.
- One function per commit below (`_a1_growth`, `_a2_warrior`, `_a3_paths`, `_a4_loadouts`, `_a5_shaman`, `_a6_brawler`); `_a2_warrior` waits for the section 7 answer; `_a6_brawler` runs once BAL2-L commit 5 is merged.

| Commit | Depends on |
| --- | --- |
| A1 growth, A3 paths, A4 loadouts, A5 shaman | BAL2-L commit 3 |
| A2 warrior gear | BAL2-L commit 3 + section 7 answer |
| A6 brawler unarmed | BAL2-L commit 5 |

The shipped-class warning check (`engine_pieces_test_suite` "warnings for the shipped classes stay empty") passes after every commit only in this order: A2 lands with its section 7 fix in the same commit, and A6 lands after commit 5.

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

Gear budget consequence: hand axe 16 + kite 8 + plate 9 = 33.0, above the band 18.4-27.6 with no special item, so `GearBudget.check_class` warns and `engine_pieces_test_suite` "warnings for the shipped classes stay empty" fails. Section 7 picks the fix; with the recommended option D1, A2 also writes `shd_kite` `mitigation` 8 -> 6 and `arm_plate` `mitigation` 10 -> 8 (items.json, same commit), giving a warrior loadout of 27.0.

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
| warrior (section 7 D1) | 242/284/334/382 | 20 (24 without D1) | 468 (597) | 41 -> 62 melee | 1 | 27.0 (33.0 without D1) |
| ranger | 158/178/200/228 | 8 | 232 (180) | 52 -> 73 ranged | 4 (PERC 19; was 4 from L3) | 24.0 |
| priest | 176/192/208/232 | 5 | 229 (229) | 34 -> 45 melee | 1 | 26.5 |
| rogue | 152/172/194/222 | 8 | 225 (179) | 45 -> 66 melee | 1 | 19.0 |
| mage | 144/160/184/200 | 4 | 199 (146) | 50 -> 71 ranged | 3 (4 at L7) | 27.0 |
| brawler (mace, until A6) | 228/270/320/368 | 11 | 390 (390) | 44 -> 65 melee | 1 | 19.0 |

Acceptance (A1-A3): `validate_all()` is `{}`; `warnings_all()` lists no class (after A2 with its section 7 fix); every HP cell equals the formula from the built unit's CONS and STR; ranger and brawler L1 units own and have active `aimed_shot` / `body_blow`; `StatRules` growth checks pass.

## 4. Party unit loadouts (commit A4)

Each shipped L5 party unit gets an explicit `loadout` = its class's primary role list (`games/emberwake/docs/specs/BAL2_SIM.md` section 6.3). Gear slots stay blank (class gear).

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

## 6. Brawler unarmed (commit A6): decided, option A

Decided (user 2026-10-17, decision 10 and DECISIONS "BAL2 role answers" (5)): the brawler fights unarmed, in melee close combat, engaging several foes at once, compensated by trained fists (option A, base 15; contract in `games/emberwake/docs/specs/BAL2_L.md` Rule 5, commit 5).

Commit A6, data (after BAL2-L commit 5 is merged):

| Field (class `brawler`) | Now | New |
| --- | --- | --- |
| `main_hand` | `wpn_flanged_mace` | "" (bare hands; mace stays in the proficiency, so units may still carry one) |
| `description` | "Bruiser: fists, maces and a thick hide; closes the gap and hits hard." | "Bruiser: bare fists and a thick hide; leaps into a crowd and fights several foes at once." |

Acceptance (A6): brawler hero L1/3/5/7 attack power 44/52/59/66 with `trained_fists` true (BAL2-L Rule 5 check 1); class gear budget 15 + 4 = 19.0, in band; `warnings_all()` lists no class; enemy brawler units (`goblin_brawler`, `goblin_berserker`, `enemy_brawler`, `ambush_berserker`, all blank main hand) now fight bare-handed too.

The options below are kept as the decision record.

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

### 6.3 Outcome

A chosen with base 15 and the fist level >= 2 gate (exact mace parity, budget in band through one shared constant, STR + DEX hybrid kept). D is re-measured after BAL2 with the `aoe_swarm` limit test (brawler primary role = melee AoE): if the brawler misses the AoE bands, D is the data follow-up.

## 7. Gear budget review (awaiting user choice; gates commit A2)

User 2026-10-17 (DECISIONS "BAL2 role answers" (6)): find which `GearBudget` component inflates the warrior; prefer a data-only fix, else a formula re-weighting that favours no role; no role-based budgets.

### 7.1 Breakdown per class default loadout

Weights (DECISIONS "Gear budget constants (M2d)"): power x1.0, mitigation x1.5, stat bonus x2.0 per point, burden x-2.0, extra block cost x-3.0; item effects (orb, tome) as listed. Band 18.4-27.6 (target 23 +-20%). Measured with `GearBudget.item_value` on main c3520b4.

| Loadout | Items | Power | Mitig. x1.5 | Stats x2 | Burden x-2 | Block x-3 | Effects | Total |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| warrior now | hand axe, tower (special), plate | 16 | 39.0 (16 + 10) | 0 | -12 (3 + 3) | -3 | 0 | 40.0 (excused) |
| warrior BAL2 | hand axe, kite, plate | 16 | 27.0 (8 + 10) | 0 | -10 (2 + 3) | 0 | 0 | 33.0 X |
| warrior BAL2 + D1 | hand axe, kite 6, plate 8 | 16 | 21.0 (6 + 8) | 0 | -10 | 0 | 0 | 27.0 |
| ranger | shortbow (2H), leather | 20 | 6.0 (4) | 0 | -2 | 0 | 0 | 24.0 |
| priest | cleric mace, tome, vestment | 13 | 4.5 (3) | 6 (3 KNOW) | 0 | 0 | 3 (tome) | 26.5 |
| brawler now | flanged mace, leather | 17 | 6.0 (4) | 0 | -4 (1 + 1) | 0 | 0 | 19.0 |
| brawler BAL2 (A6) | bare hands 15, leather | 15 | 6.0 (4) | 0 | -2 | 0 | 0 | 19.0 |
| rogue | stiletto, leather | 15 | 6.0 (4) | 0 | -2 | 0 | 0 | 19.0 |
| mage | wand, orb, mage robe | 15 | 3.0 (2) | 4 (2 INT) | 0 | 0 | 5 (orb) | 27.0 |

Finding: mitigation is the inflating component. The warrior's 27.0 mitigation value is 82% of its 33.0 total; every other class has 3.0-6.0. The cause is item count, not the weight: the warrior wears two mitigation items (shield + heavy body) holding 18 mitigation; everyone else wears one light body (2-4). The 1.5 weight is roughly fair: EHP = HP x (1 + rating / 50), so one armor point is +2% effective HP at every level, while one power point is +1/P damage (+2.3% at P 44, +1.5% at P 65); armor is worth about 1.0 power early and about 1.3 power late.

### 7.2 Options

- D1 (data only, recommended): `shd_kite` mitigation 8 -> 6 and `arm_plate` mitigation 10 -> 8 (burden, block cost, requirements unchanged). Item values kite 8.0 -> 5.0, plate 9.0 -> 6.0; warrior 27.0, every shipped class in band (19.0-27.0), formula untouched. Warrior L5 armor 20 (reduction 29%), EHP 468: still the highest (brawler 390, ranger 232). Risk: both items are in the starter stash and in test fixtures (equip/slot checks only; the tower-vs-kite test still holds, 16 > 6); run the dojo suite.
- D2 (data only): warrior shortsword + leather instead of hand axe + plate: 14 + 12.0 - 6 = 26.0; armor L5 about 18, EHP about 450, power -1. Loses the plate identity of the tank; items unchanged.
- F (formula, role-neutral): `W_MITIGATION` 1.5 -> 1.0 (armor point = power point, the EHP parity above), `TARGET_BUDGET` 23 -> 21.5 (the re-weighted mean is 21.8), `TOLERANCE` 0.20 -> 0.25 (band 16.1-26.9). Before -> after: warrior 33.0 -> 24.0, ranger 24.0 -> 22.0, priest 26.5 -> 25.0, brawler 19.0 -> 17.0, rogue 19.0 -> 17.0, mage 27.0 -> 26.0: all in band, no item change. Cost: three constants in `gear_budget.gd`, the M2B range test (`shd_tower` value 15 -> 7, outside its [14, 18] range), the DECISIONS M2d row. Risk: a wider band weakens the guard, and every armor item's value drops for the Forge previews.
- Not recommended: a documented special flag on `arm_plate` (my earlier default): it keeps 33.0 by excusing it instead of fixing it.

Recommendation: D1. It is the smallest data change that brings every shipped class into band with the current formula, keeps the warrior the clear EHP leader, and follows the user's order of preference (data before formula). F is the fallback if the user wants the accepted armor 24 kept.

## 8. Out of scope

New items (and C of section 6 only if it had been picked), skill number changes (except D after BAL2), enemy unit loadouts, world data, Forge, AI, gear budget formula (unless the user picks F).

## 9. Open questions

1. Gear budget review (section 7): D1 (recommended), D2 or F?
2. Should priest and brawler party units at L5 be added? Answered 2026-10-17: no.
