# BAL2 proposal: answers to the BAL1 open issues (options, trade-offs, recommendation)

Status: proposed (design-brainstorm 2026-10-17; nothing here is approved)
Input: `games/emberwake/docs/BALANCE_REPORT.md` run 2026-10-16 (BAL1), plus three read-only scratch probes on a sandbox fork (no repo code or data touched; scripts in the brainstorm agent's scratchpad).
Target: every class 40-60% duel win rate at L1/3/5/7.
Probe state: data as of main e74c356 (BAL1). Main moved to c6b5e6c during the pass (SKILL1 merge: basics removed from class paths, no number changes). A scratch sim meant to confirm root cause 1 (an L7 priest with spend kept out of PERC) did not compile against the stale class cache after the merge, so root cause 1 is based on reading the code. Confirming it is the first acceptance check of BAL2-L.

## 1. Probe data (what the numbers are built on)

Derived stats per class (hero, empty spend = class weights, class default loadout). EHP = HP / (1 - armor reduction).

| Class | HP L1/L3/L5/L7 | Armor (red.) | EHP L5 | Melee/ranged power L1 -> L7 | Speed L5 | Growth |
| --- | --- | --- | --- | --- | --- | --- |
| warrior | 248/308/370/430 | 31 (38%) | 597 | 39 -> 50 melee | 13 | CONS3 STR1 PERC1 |
| brawler | 228/270/320/368 | 11 (18%) | 390 | 44 -> 65 melee | 23 | STR2 CONS2 DEX1 |
| priest | 176/192/208/232 | 5 (9%) | 229 | 34 -> 45 melee | 16 | KNOW2 CONS1 INT1 PERC1 |
| ranger | 148/154/158/164 | 7 (12%) | 180 | 52 -> 73 ranged | 35 | DEX2 PERC2 STR1 |
| rogue | 144/148/154/158 | 8 (14%) | 179 | 47 -> 78 melee, crit 16% -> 37% | 41 | DEX3 PERC1 STR1 |
| mage | 136/136/136/136 | 4 (7%) | 146 | 51 -> 83 ranged | 17 | INT3 KNOW1 PERC1 |

Active class families (default loadout = first five path families by level, then path order):

| Class | L1 | L5 and L7 (identical) | Owned but never active |
| --- | --- | --- | --- |
| warrior | taunt_shout | taunt_shout, shield_bash, riposte, intimidate, sunder | shield_wall |
| ranger | none | aimed_shot, pinning_shot, leap_back, barbed_arrow, volley | deadeye (2.0x), focus |
| priest | smite, rally | smite, rally, mend, purify, hex (L3 had greater_mend instead of hex) | greater_mend, renewal, judgement |
| brawler | none | lunge, body_blow, sweep_kick, hamstring, haymaker | charge_in, cyclone_kick |
| rogue | quick_stab | quick_stab, pounce, bleeding_cut, backflip, hamstring | lacerate, assassinate |
| mage | arcane_bolt | arcane_bolt, arc_lance, hex, focus, firebomb | glacial_burst, inferno |

Scratch A/B at L5, warrior vs the other five, 3 seeds, sides swapped (binary score / continuous score 0.5 + (HP fraction left - foe's) / 2 / average rounds):

| Variant | ranger | priest | brawler | rogue | mage |
| --- | --- | --- | --- | --- | --- |
| default, cap 6 | 100 / 78 / 4.0 | 92 / 65 / 6.0 | 100 / 66 / 4.7 | 100 / 75 / 2.0 | 100 / 88 / 3.7 |
| riposte removed from loadout, cap 6 | 100 / 78 / 4.0 | 100 / 76 / 6.0 | 100 / 63 / 5.3 | 100 / 81 / 3.0 | 100 / 88 / 3.7 |
| default, cap 12 | 100 / 78 / 4.0 | 83 / 64 / 12.0 | 100 / 66 / 4.7 | 100 / 75 / 2.0 | 100 / 88 / 3.7 |

## 2. Root causes found

1. **Engine bug: PERC >= 18 turns melee units into ranged units.** `CharacterStats.get_effective_range` adds +1 range at PERC >= 18 for every weapon. Every consumer then treats `attack_range > 1` as "ranged": `AttackCommand` / `AreaAttackCommand` call `DamageCalculator.compute` with `is_ranged = true` (a melee weapon does not "fit", so no weapon power and DEX scaling), `SkillDef.reach_for` returns band 2..3 (the unit can no longer strike an adjacent foe with a plain attack), `PlanContext.profile_of` defaults it to SNIPER, and the AI kites it and never ripostes against it. Hit units: priest from L7 (power 45 -> 12), rogue from L7 (78 -> 60, plus kiting), warrior from L9 (53 -> 11). This explains issue 7 (priest 0% at L7), the L5-beats-L7 priest mirror and the L7-beats-L9 warrior mirror (issue 9), and part of the rogue/warrior L7 numbers. Nothing in DECISIONS or the brief documents the PERC range rule.
2. **Durability spread is 4x.** L5 EHP: warrior 597, brawler 390, priest 229, ranger 180, rogue 179, mage 146. Warrior armor 31 comes from `shd_tower` (special, mitigation 16) + `arm_plate` (10) + STR/4; class gear value 40 vs `GearBudget.TARGET_BUDGET` 23 (15 excused as special).
3. **HP growth is CONS growth only.** Max HP = 60 + CONS x 8 + STR x 2 and three classes grow CONS 0: mage +0 HP per level, ranger and rogue +2-3, while damage grows +5 per level (mage INT3, rogue DEX3 with crit +3.5%/level). Result: the warrior's EHP lead wins L1-L5, the damage dealers outscale it by L7 (warrior 56%, mage 80%).
4. **Riposte is not the warrior lever.** Removing riposte from the L5 warrior changes nothing (100% vs all five either way; continuous 74 -> 77). The AI1 swing (warrior 20% off -> 97% on) must come from the other AI1 casts (taunt, intimidate weaken 25%, shield_bash/sunder stagger use), not the finisher.
5. **Kits stop growing at L5.** The (level, order) default loadout keeps the earliest five families, so every class has the same active kit at L5 and L7, and the ranger's 2.0x deadeye and the priest's greater_mend / judgement are never used. The ranger also fills a slot with `volley` (area, needs 2 foes: dead in a duel).
6. **No class family at L1** for ranger and brawler (first path family at L3), while mage, rogue, priest and warrior get one at L1. Matches ranger 28% and brawler 38% at L1.
7. **The sim is near-deterministic, not cap-limited.** A 12-round cap moves only the priest matchup; most fights end in 2-5 rounds. Binary scoring hides large margins (warrior vs brawler: 100% binary, 66 continuous).

## 3. Issues: options and recommendation

### Issue 6 (first): sim reliability

- A. Continuous score (0.5 + HP-fraction margin / 2, KO = 0 or 1) next to the binary one. Cheap (`tools/balance_sim.gd` only). Shows margins; does not add variance.
- B. A plus variance inputs: spend variants (class weights, CONS-heavy, primary-heavy: DECISIONS "Level-up spend" already asks sims to try variants), start distance 3 and 5, both of the balanced and aggressive presets. Medium (sim only); 4-8x more fights per matchup, so keep 3 seeds.
- C. B plus arguments: `smart_skills=on|off`, `cap=N` (default 10), `loadout=default|newest` (issue 1), and a `stats` mode that prints the table of section 1 (HP, armor, EHP, power, speed, active families) per class and level.
- Risk: runtime (BAL1 duel tables took minutes; B multiplies it); mitigate with 3 seeds and a `quick` flag. HISTORY: nothing touches it.
- **Recommend C** with cap 10 as the default. The bracket is judged on the continuous score; the binary score stays printed for comparison.

### Issue 7 and 9: priest L7 collapse and level-mirror anomalies

- A. Fix the range rule: PERC >= 18 grants +1 range only to ranged weapons (`base_range > 1`). One line in `character_stats.gd` plus a test; melee units stay melee. Changes the ranger only if its bow already counts (it does, unchanged) and the mage (staff/wand range 3 -> 4 at PERC 18, unchanged).
- B. Keep the bonus for melee as "reach" but decouple: ranged power and band 2..3 only for ranged weapons (`Equipment.is_ranged_weapon()`), melee band 1..range. Touches `attack_command`, `area_attack_command`, `riposte` call sites, `skill_def.reach_for`, `plan_context`, `tactical_ai` (about 8 call sites). Gives spears/polearm-like reach for free, but adds a new mechanic nobody designed.
- C. Remove the PERC range bonus entirely. Ranger/mage lose their L3/L7 range growth.
- **Recommend A.** Re-measure the mage mirror (L5 vs L7) after it: with A the mage keeps range 4, so the remaining anomaly is the flat mage HP (issue 5).

### Issue 1: class standings (warrior high, ranger and priest low, mage high at L7)

- A. Data pass on growth and gear (asset lane, `ContentCatalog.upsert` on `classes.json`; `StatRules` validation unchanged: total 62, growth 5, max 3/stat). Starting values:

| Class | Growth now -> proposed | Other data change | L5 effect (estimate) |
| --- | --- | --- | --- |
| warrior | CONS3 STR1 PERC1 -> CONS2 STR2 PERC1 | off-hand `shd_tower` -> `shd_kite` (mitigation 16 -> 8); tower stays as a reward item | HP 370 -> 338, armor 31 -> 23 (38% -> 32%), EHP 597 -> 497 |
| ranger | DEX2 PERC2 STR1 -> DEX2 PERC1 STR1 CONS1 | path: `aimed_shot` L3 -> L1 | HP 158 -> 190, EHP 216 |
| priest | unchanged | path order so greater_mend stays active (see issue 1 B) | kit fix only |
| brawler | unchanged | path: `body_blow` L3 -> L1 | L1 class skill |
| rogue | DEX3 PERC1 STR1 -> DEX2 PERC1 STR1 CONS1 | none | HP 154 -> 186, speed 41 -> 37, crit 30% -> 26% |
| mage | INT3 KNOW1 PERC1 -> INT2 CONS1 KNOW1 PERC1 | none | HP 136 -> 168 at L5, 184 at L7; ranged power L7 83 -> about 72 |

- B. Default loadout rule "newest five": keep the level-1 family, fill the other four with the most recently unlocked families (ties: path order); area families only if the class has fewer than five single-target ones. Logic change in the default-loadout derivation (`Combatant` / `ContentFactory` loadout default). This re-opens the FORGE1 "(level, order)" default (DECISIONS row "Strict hand typing and no class skill loadout"); new reason: BAL1 shows identical L5/L7 kits for all six classes and the ranger/priest best families never active. Alternative without re-opening it: author explicit unit `loadout` lists in data for party/enemy units and let the sim pass `loadout=newest`.
- C. Warrior-only nerf (HP/armor) and accept that standings drift by level. Cheapest, but BAL1 shows inversions by level, so one-level fixes will not hold.
- D. Treat the tank differently (measure the warrior in 2v2 party sims instead of duels). Changes the target the user set; listed only as a question.
- Risk: A moves every level at once, so it must be measured with the fixed sim (issue 6) and the range fix (issue 7). HISTORY: the unequal stat totals row is respected (totals stay 62/5).
- **Recommend A + B-alternative** (explicit data loadouts plus sim `loadout=newest`), and decide on B (rule change) after the BAL2 rerun shows whether data loadouts are enough.

### Issue 2: elite tier identical to hero

- A. Elite hp 1.25, power 1.1 (`StatRules.TIERS`, code constant, logic lane). Target: hero wins 25-40% 1v1 on the continuous score.
- B. Elite hp 1.4, power 1.1: closer to boss (1.7/1.2), clearer "rare elite" threat (DECISIONS exploration encounters).
- C. Accept elite == hero (an elite is a hero-strength enemy).
- **Recommend A**: elites appear on the step counter (DECISIONS), so they should be beatable solo more often than a boss (10-20%).

### Issue 3: riposte finisher cost

- A. Keep as is. Data: no measurable effect at L5 (section 1 A/B).
- B. Condition it (foe melee, adjacent, unit HP above `defend_below`): already the AI1 condition.
- C. Lower the counter or raise AP: no evidence it is needed.
- **Recommend A**; re-check with the new sim at L3 (riposte unlocks at L3) and against melee-heavy groups once the warrior tuning lands.

### Issue 4: goblin shaman and blank unit slots

- A. Explicit data only: set the shaman's slots in `units.json` (via upsert). No code; but a blank slot still means "class default", so a unit cannot express "no off-hand".
- B. Sentinel value `none` for unit gear slots (`main_hand`, `off_hand`, `head`, `body`, `accessory_1`, `accessory_2`; pouch: `["none"]` = empty): `ContentSchema` allows it, `ContentCatalog._unit_errors` accepts it, `ContentFactory._equip` skips the class fallback for that slot. Blank keeps meaning "inherit". Small logic change plus a test.
- C. Unit flag `inherit_class_gear` (bool, default true): all-or-nothing.
- **Recommend B** (per-slot control, blank stays backward compatible), then data: goblin_shaman main `wpn_cleric_mace`, off `none`, body `arm_robe`, pouch `none`, description "Smites and mends its allies." (the current "Opens with Focus, then fights from range" is stale).
- Side finding: the brawler class loadout carries `wpn_flanged_mace` while DECISIONS says "Brawler stays unarmed (no knuckles item for now)". Question for the user (keep the mace or go unarmed: `GearBudget.UNARMED_VALUE` 14 vs the mace's value).

### Issue 5: mage burst and flat HP

- A. Growth change of issue 1 A (CONS1): +8 HP per level, slower INT scaling.
- B. Lower spell tiers (`arcane_bolt` T2/T3 power) or raise their `unlock_offset`.
- C. Initiative tie-break for mirrors (first block order). Engine change for one symptom; no.
- **Recommend A**, re-measure; B only if the mage stays above 60% at L7 after A.

### Issue 8: boss and 1v3 unwinnable solo

- In a near-deterministic mirror any enemy with HP x power above the hero's loses 0%; 10-20% (DECISIONS "Solo difficulty") is only measurable with the variance inputs of issue 6.
- A. Boss hp 1.5, power 1.1 (from 1.7 / 1.2), measured on the continuous score with variants.
- B. Keep the boss and add boss mechanics later (S12 combos/terrain).
- C. 1v3: measure the hero against three minions TWO levels lower as the "should win" check (DECISIONS: losses fine when under-levelled, improves with level); keep same-level 1v3 as an expected loss. Add a `group` sim mode.
- **Recommend A + C.** Q8 in ROADMAP is answered by DECISIONS "Solo difficulty"; mark it so.

## 4. Milestones (coarse, sequenced)

| Id | Lane | Goal | Touches | Depends on |
| --- | --- | --- | --- | --- |
| BAL2-L | logic (one branch, commit order fixed) | (1) sim reliability: continuous score, cap arg (default 10), `smart_skills`, `loadout=default/newest`, spend/distance/preset variants, `stats` and `group` modes; (2) PERC range fix (issue 7 A) with a test; (3) gear sentinel `none` (issue 4 B); (4) `StatRules.TIERS` elite and boss values the user picks | `tools/balance_sim.gd`, `character_stats.gd`, `content_schema.gd`, `content_catalog.gd` (unit part), `content_factory.gd` (`_equip`), `stat_rules.gd` (TIERS), new suite `bal2_rules_test_suite.gd` | none; conflicts with SKILL1-L on `content_schema/catalog/factory`, so run after SKILL1-L or keep (3) as its last commit |
| BAL2-A | asset (one branch, a seed in `data/seeds/bal2.gd`) | Data pass of issue 1 A (growth, warrior off-hand, ranger/brawler L1 path steps, explicit unit loadouts if picked), goblin_shaman gear and description | `data/catalog/classes.json`, `units.json` via `ContentCatalog.upsert`; rule-derived tests only if a test pins a changed number | BAL2-L commit 3 (sentinel) |
| BAL2 | balance | Rerun duel (L1/3/5/7, continuous score, variants), tier, levels, group; A/B smart_skills; second small data round if a class is outside 40-60 | `games/emberwake/docs/BALANCE_REPORT.md` | BAL2-L, BAL2-A |

Order: BAL2-L first (the sim fix and the range bug change every number), then BAL2-A, then BAL2. S2 and S4 stay out of `balance-ready` until BAL2 passes (BAL1 issue 9).

## 5. Out of scope

New skills or items, terrain/combos, AI behaviour changes beyond what the range fix implies, the Forge, XP curve values (not tested by BAL1).

## 6. Decisions for the user

1. Sim reliability: issue 6 option C, judged on the continuous score, default cap 10? (recommended: yes)
2. PERC range: +1 only for ranged weapons (7 A), reach for melee (7 B) or remove (7 C)? (recommended: A)
3. Growth/gear data pass of issue 1 A as starting values (every damage class grows CONS 1; warrior CONS2 STR2 with a kite shield)? (recommended: yes)
4. Late-game kits: explicit data loadouts + sim `loadout=newest` now, rule change "newest five" only if needed (recommended), or change the default rule now?
5. Ranger `aimed_shot` and brawler `body_blow` at L1? (recommended: yes)
6. Elite tier: 1.25/1.1 (recommended), 1.4/1.1, or keep 1.0/1.0?
7. Boss tier 1.5/1.1 measured with variants, and 1v3 checked against minions two levels lower? (recommended: yes)
8. Riposte finisher: keep unchanged? (recommended: yes)
9. Unit gear: per-slot sentinel `none` (recommended), explicit data only, or an all-or-nothing flag; goblin_shaman kit as listed?
10. Brawler loadout: keep `wpn_flanged_mace` or make it unarmed as DECISIONS says?
11. Tank metric: keep the warrior in the duel bracket 40-60 (recommended) or measure it in party sims?
