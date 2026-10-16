# Balance report (written by the balance agent; read by the brainstorm agent)

## BAL2 baseline 2026-10-17 (worktree `ngp-bal2base`, branch bal/bal2base at main ebe2a3f: BAL2-L run 1 + BAL2-A)

Quick baseline, not the full rerun. Scope note: ROADMAP S2/S4 are not promoted to `balance-ready`; run at the user's request anyway. Method: `tools/fc.sh sim duel|tier|stats` with default args (cap 10, smart_skills on, loadout default, score both), 3 seeds (sim near-deterministic). Cells are `binary/hp-margin` score of the ROW class. Duel is a DIAGNOSTIC with a 30-70 band (docs/decisions/balance.md "BAL2 role answers"); party/role targets arrive with BAL2-L commit 1b (not merged). Levels 1/3/5/7 plus two L5 A/B rows. No 10-seed pass (averages are won-matchup counts, see issue 5).

### Duel averages, hp score (3 seeds; band 30-70, X = outside)

| Class | L1 | L3 | L5 | L7 | L5 smart_skills=off | L5 loadout=newest |
| --- | --- | --- | --- | --- | --- | --- |
| warrior | 47 | 88 X high | 89 X high | 75 X high | 61 | 88 X high |
| ranger | 40 | 64 | 53 | 67 | 40 | 60 |
| priest | 43 | 17 X low | 4 X low | 25 X low | 19 X low | 9 X low |
| brawler | 67 | 51 | 57 | 60 | 63 | 57 |
| rogue | 63 | 60 | 60 | 33 | 60 | 60 |
| mage | 40 | 20 X low | 37 | 40 | 57 | 37 |
| In band | 6/6 | 3/6 | 4/6 | 5/6 | 5/6 | 4/6 |

Binary averages equal the hp score except warrior L3 93, L5 90, L7 77 and priest L3 10, L5 3, L7 23; the picture is the same.

Level 5 matrix (row vs column): warrior beats all five (priest 83, brawler 67); rogue beats ranger, priest, mage and loses to warrior and brawler; ranger beats priest, mage, brawler (67) and loses to warrior and rogue; priest only takes 17 vs warrior and 0 elsewhere; mage beats priest and brawler (83) only; brawler beats priest and rogue, loses to warrior, ranger, mage.

### Comparison with BAL1 (2026-10-16, 10 seeds, different rules)

Not strictly comparable: the range fix (melee stays range 1 at PERC 18), the BAL2-A data pass (growth, kite shield, D1 mitigation, loadouts, trained-fist brawler, goblin_shaman) and the tier multipliers all changed. Directionally:

- Warrior is still dominant from L3 up (BAL1 97/97/56 at L3/L5/L7 vs 88/89/75 now); L1 improved from 80 to 47. It is the only class HIGH at 3 of 4 levels.
- Priest got worse: BAL1 46/32/23/0 vs 43/17/4/25 now; it loses almost every L3/L5 matchup.
- Ranger improved a lot (28/21/22/46 to 40/64/53/67; the range fix removed melee PERC-18 range advantage, plus aimed_shot at L1). Mage fine at L1/L5/L7 (40/37/40), low at L3 (20; BAL1 50). Brawler (now unarmed) and rogue mostly in band; rogue drops to 33 at L7 (BAL1 60).
- In band count: L1 6/6 (BAL1 2/6), L3 3/6 (1/6), L5 4/6 (3/6), L7 5/6 (4/6), but under the wider 30-70 band; at the old 40-60 it is L1 3/6, L3 2/6, L5 3/6, L7 2/6, so no real gain.

### Classes outside 30-70 on the hp score

- Warrior: HIGH at L3 (88), L5 (89), L7 (75); in band only at L1 (47). Still HIGH with loadout=newest (88).
- Priest: LOW at L3 (17), L5 (4), L7 (25), also with smart_skills off (19); fine at L1 (43).
- Mage: LOW at L3 (20) only.
- Near the low edge but in band: rogue L7 33, mage L5 37.

### A/B rows (L5)

- smart_skills=off: warrior falls 89 to 61 (in band), mage rises 37 to 57, priest 4 to 19, ranger 53 to 40. Consistent with BAL1: AI skill use mostly benefits the warrior (taunt/riposte/intimidate/sunder). The off row printed "2 profiles" in its header (unexplained, issue 7).
- loadout=newest: nearly identical to default (warrior 88, ranger 60, priest 9, rest equal), so default loadouts are not distorting class results.

### Tier (3 seeds, hero vs its own class at the tier; bin score, hp margin in brackets where informative)

| Class | minion 1v1 | standard 1v1 | elite 1v1 | boss 1v1 | any 1v3 |
| --- | --- | --- | --- | --- | --- |
| warrior | 100 | 100 | 0 | 0 | 0 |
| ranger | 100 | 100 | 0 | 0 | 0 (L7 minion 17) |
| priest | 100 | 100 (hp 71-82 at L3-7) | 0 (hp 36-44) | 0 (hp 33-37) | 0 (minion L5 17, L7 50) |
| brawler | 100 | 100 | 0 | 0 | 0 |
| rogue | 100 | 100 | 0 (hp 7-15 at L5-7) | 0 (L1 33) | 0 (minion L3/L5 50) |
| mage | 100 | 100 (L7 50) | 0 (L7 50) | 0 (L7 17) | 0 |

Identical across levels 1/3/5/7 except where noted.

Versus expectations (minion/standard win, elite ~50%, boss favoured): minion and standard are fine. Elite (1.25 HP / 1.1 power) is now 0% for the hero in 1v1 at every level where BAL1 (old multipliers) was 50%; this looks unintended, an equal mirror with +25% HP and +10% power wins a near-deterministic duel every time. Boss (1.5 / 1.1) is 0% as expected (favoured), but the hp margin is mostly 0 too, so heroes do not even bleed it: a solo boss is a wall, not the "10-20% desired". 1v3 is 0% against every tier, including minions, same as BAL1 (Q8 still stands).

### Stats (class spend)

| Class | L1 EHP | L5 EHP | L7 EHP | L5 HP | L5 melee/ranged power | L5 range, crit% |
| --- | --- | --- | --- | --- | --- | --- |
| warrior | 329 | 468 | 542 | 334 | 56/11 | 1, 14 |
| ranger | 177 | 232 | 264 | 200 | 21/67 | 4, 35 |
| priest | 194 | 229 | 255 | 208 | 42/12 | 1, 16 |
| brawler | 264 | 390 | 456 | 320 | 59/59 | 1, 23 |
| rogue | 173 | 225 | 258 | 194 | 60/42 | 1, 36 |
| mage | 156 | 199 | 216 | 184 | 9/65 | 3, 17 |

Warrior L5 EHP 468 matches BAL2-A; it is about 2x the squishy classes, with melee power 56 comparable to rogue/brawler and the control kit (taunt, riposte, shield_bash, intimidate, sunder). Priest has no EHP or damage edge (EHP 229, melee 42, ranged 12, armor 5), and its kit (smite, rally, mend, purify, hex) is support/ally oriented.

### Issues for the brainstorm agent

1. Warrior still HIGH (75-89) at L3-L7 even after the kite shield and D1. EHP 468 at L5 vs 229-390 is the largest gap; AI skill use adds about 28 points at L5. Levers: warrior HP/CONS growth, kite/plate mitigation, riposte or taunt multipliers.
2. Priest LOW at L3-L7 (4-25). Duel may be the wrong test for a support (diagnostic only), but it has no damage identity in 1v1 (melee 42, ranged 12). Decide whether priest is judged by BAL2-L party mode (1b) or gets a self-sustain/offence lever (smite multiplier, mend self-target, growth).
3. Mage L3 20 while L1/L5/L7 are 37-40: the L3 skill set is the suspect (not probed); check what mage owns at L3.
4. Tier: elite 1.25/1.1 gives 0% hero 1v1 (expected ~50%); boss is a flat wall. Levers: tier multipliers (elite nearer 1.1/1.05) or judge tiers in party mode. 1v3 vs any tier is 0% including minions (Q8).
5. Duel results are quantised (0/100 per matchup, deterministic): class averages are won-matchup counts and 3 vs 10 seeds add nothing. Initiative/position randomisation would make the band meaningful.
6. Rogue drops to 33 at L7 (60 at L3/L5); cause not probed (cap, growth vs warrior/brawler?).
7. smart_skills=off header says "2 profiles"; confirm intended.
8. Not run: `levels` and `group` sims, 10-seed passes, `spend`/`preset` variants.

---

## Run 2026-10-16 (BAL1: new roster, AI1 on/off, levels 1/3/5/7, worktree `ngp-bal1` at main e74c356)

Systems tested: S1, S3, S4, S7, S13 plus the reworked S2/S12 content (all six class kits, AI1). Note: ROADMAP lists S2 and S12 as `in-progress`, so only the balance-ready rows were officially in scope; the roster itself cannot be tested without them. Method: `tools/fc.sh sim duel|tier|levels`, 3 seeds to scan and 10 seeds for every duel table. The AI-off column uses a scratch copy of `tools/balance_sim.gd` (agent scratchpad, not in the repo) that sets `smart_skills = false` on every combatant profile after `session.start()`; no repo file was touched. Both sides use the "balanced" preset.

Reliability warning: 3 seeds and 10 seeds give nearly identical tables (fights are almost deterministic), so most matchups are 0% or 100% and a class average is the count of won matchups. More seeds add little power; the noise is initiative/ordering and the 6-round cap (issues 5 and 6).

### Duel averages, 10 seeds (win rate of the ROW class vs the other five; target 40-60%)

| Class | L1 on | L3 on | L5 on | L7 on | L1 off | L3 off | L5 off | L7 off |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| warrior | 80 X | 97 X | 97 X | 56 | 80 X | 93 X | 20 X | 10 X |
| ranger | 28 X | 21 X | 22 X | 46 | 28 X | 11 X | 10 X | 56 |
| priest | 46 | 32 X | 23 X | 0 X | 46 | 34 X | 42 | 20 X |
| brawler | 38 X | 37 X | 60 | 58 | 38 X | 49 | 100 X | 70 X |
| rogue | 68 X | 64 X | 42 | 60 | 68 X | 64 X | 52 | 64 X |
| mage | 40 | 50 | 56 | 80 X | 40 | 50 | 76 X | 80 X |
| In bracket | 2/6 | 1/6 | 3/6 | 4/6 | 2/6 | 2/6 | 2/6 | 1/6 |

(X = outside 40-60%. "on" = default `smart_skills = true`, "off" = pre-AI1 behaviour.) Level 1 is identical on/off: no AI1 skill is unlocked yet.

Level 5 matrix, AI on (row beats column): warrior beats everyone 85-100%; ranger loses to everyone but brawler; priest only beats ranger; brawler beats priest, rogue, mage and loses to warrior and ranger; mage beats ranger, priest, rogue and loses to warrior and brawler.

### Previous run comparison

The 2026-10-15 baseline is for the old roster (shaman/berserker/scout), so classes do not map one to one. Headline: warrior is still dominant (was 100% at L5, now 97% at L3/L5 with AI on) and ranger still weakest (20% then, 22% now). In bracket at L5: 3/6 now versus 2/6 then. No overall improvement; standings reshuffle at every level.

### Tier table (3 seeds, hero vs its own class at the tier; identical across levels except where noted)

| Tier | 1v1 hero wins | 1v3 hero wins | Expected |
| --- | --- | --- | --- |
| minion | 100% (priest L7 67%) | 0% (ranger L5 50%, L7 17%) | win 1v1: ok |
| standard | 100% (mage L3 67%, L7 50%; priest L7 33%) | 0% | win 1v1: mostly ok |
| elite | 50% every class and level | 0% | ~50%: ok but see issue 2 |
| boss | 0% (priest L7 100%) | 0% | boss favoured: ok; solo 10-20% desired, got 0% |

1v3 against the same tier is 0% almost everywhere, even three minions (0.4 HP, 0.6 power each) against one hero (Q8 still stands).

### Levels (lower level N vs level N+2 mirror; a healthy value is well under 50%), AI on / AI off

| Class | L1 v L3 | L3 v L5 | L5 v L7 | L7 v L9 |
| --- | --- | --- | --- | --- |
| warrior | 0 / 0 | 0 / 100 | 75 / 50 | 100 / 50 |
| ranger | 0 / 0 | 33 / 33 | 0 / 0 | 0 / 0 |
| priest | 0 / 0 | 17 / 17 | 100 / 100 | 50 / 92 |
| brawler | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| rogue | 0 / 0 | 33 / 33 | 33 / 33 | 0 / 0 |
| mage | 0 / 0 | 33 / 33 | 100 / 100 | 0 / 0 |

Unintended: the L5 priest and mage beat the L7 mirror 100%, the L7 warrior beats the L9 warrior 100% (AI on). Probes (scratch script, seed 11): L5 mage beats L7 mage in round 1 with both at 136 HP (mage HP is flat between L5 and L7), so initiative/alpha strike decides, not level; L7 warrior beats L9 warrior at the 6-round cap on HP fraction (389/430 vs 319/498), a timeout decided by the sim's HP-fraction rule.

### A/B `smart_skills` (AI1)

- Level 1: identical (nothing unlocked).
- Level 3: small (warrior 93 -> 97, ranger 11 -> 21, brawler 49 -> 37).
- Level 5: large. Warrior 20% (off) -> 97% (on); brawler 100 -> 60; mage 76 -> 56; ranger 10 -> 22; priest 42 -> 23. With AI1 off the warrior stalls (50% draws vs ranger/rogue, 0% vs priest, brawler, mage) and brawler/mage dominate.
- Level 7: warrior 10 -> 56, priest 20 -> 0, ranger 56 -> 46, brawler 70 -> 58, mage 80 both.
- Net: AI1 moves brawler and mage toward the bracket and lifts the warrior from far below to far above it (L3-L5). The warrior is the only riposte owner (class path L3), so the warrior swing is riposte finisher plus taunt/intimidate/guard use; not isolated per skill (would need a per-feature switch).

### Issues for the brainstorm agent

1. Outside the bracket at most levels: warrior (80-97% at L1-L5, 56% at L7), ranger (21-28% at L1-L5), priest (23-32% at L3/L5, 0% at L7), mage 80% at L7, rogue 64-68% at L1/L3. Standings invert between levels, so one flat per-class change will not stabilise it. Levers: warrior HP/armor (gear budget) and riposte, ranger durability and kiting AI, priest damage/survival, mage HP growth.
2. Elite tier is numerically identical to hero (`StatRules.TIERS`: elite hp 1.0, power 1.0), so every "elite 1v1" is a mirror coin flip (exactly 50%). Options: elite 1.15-1.3 HP/power, or accept hero == elite.
3. Riposte finisher cost (flagged by AI1): the finisher is added after the first strike and the stance ends the round (a later strike would cancel it), so a riposte owner gives up further strikes that round. Net effect in the A/B is strongly positive for the warrior (L5 20% -> 97%), so the cost is outweighed; it is the main warrior lever. Options: condition the finisher (foe melee and adjacent and HP below a threshold), lower the counter (50% attack power) or raise the AP cost, or keep it and rebalance warrior gear/HP.
4. Goblin priest tome inheritance: there is no unit named goblin priest; the match is `goblin_shaman` in `data/catalog/units.json` (class `priest`, level 3, minion, empty main_hand/off_hand/body/pouch). `ContentFactory._equip` falls back to the class loadout for every empty slot, so it carries `wpn_cleric_mace`, `wpn_tome` (off-hand, +1 KNOW, INT scaling), `arm_cleric_vestment` and a `con_heal_potion`. Its description ("Opens with Focus, then fights from range") is stale: the priest path is melee-capable with strike/guard/smite/rally/mend/purify and has no focus skill. Options: an explicit "no item" sentinel so a unit can opt out of the class loadout, or set the shaman's slots explicitly in data. The same fallback applies to every class-based enemy that leaves slots blank.
5. Mage burst: HP 136 at both L5 and L7 and it kills a 232-HP priest in round 1; initiative decides mage mirrors regardless of level. Levers: mage HP growth, spell potency or AP costs, first-block ordering.
6. Sim validity: fights cap at 6 rounds and ties are decided by HP fraction with a 2% margin; warrior and priest mirrors always time out. Results are binary per matchup, so class averages jump in steps. Suggest tooling changes for whoever owns `tools/balance_sim.gd`: a higher cap, a continuous score, and a `smart_skills` argument (I used a scratch copy for the A/B).
7. Priest at 0% at L7 (loses every matchup in 1-2 rounds, mostly to the mage burst and high-power melee) while the L5 priest beats the L7 priest 100%: the L7 priest is not stronger than the L5 one in practice; needs a probe of what the AI casts at L7.
8. Boss 1v1 is 0% at every level (target 10-20% solo) and 1v3 is 0%: lone-hero content is unwinnable, party play required (Q8).
9. BAL1 in the ROADMAP promotes S1/S2/S4/S7 to `balance-ready` afterwards; with these results S2 (classes) and S4 (AI) should not be promoted yet.

> NOTE 2026-10-15: the roster was reworked (M2c-1: warrior tank, ranger, priest, brawler, rogue, mage; M1 combat extensions; AI fixes). The tables below describe the OLD roster (shaman/berserker/scout) and are STALE; rerun `tools/fc.sh sim ...` once the balance-relevant systems are frozen.

Target: every class between **40% and 60%** average duel win rate at each tested level (10 seeds near the bracket). Only systems marked `balance-ready` in `docs/ROADMAP.md` are tested. Append new runs on top; never delete old runs (history).

## Run 2026-10-15 (baseline, level 5, 3 seeds, systems S1-S4 before any tuning)

| Class | Duel avg | In 40-60%? | Notes |
| --- | --- | --- | --- |
| warrior | 100% | no | armor 25 (plate + kite shield), HP 314 |
| berserker | 80% | no | Reckless Swing 1.7x |
| brawler | 53% | yes | |
| scout | 47% | yes | fastest, Quick Stab 0.6x |
| ranger | 20% | no | HP 158, armor 7, AI kites and wastes turns |
| shaman | 0% | no | armor 2, no body armor item, Hex 0.6x / Arcane Bolt 1.1x; cannot beat a minion of itself |

Tier 1v1 (hero vs own class): minion 100%, standard 100%, elite 50%, boss 0%. 1v3 minions: hero loses (except shaman stalemates).

Issues for the brainstorm agent: (1) warrior dominance from gear budget; (2) shaman damage + armor + AI turn waste; (3) ranger durability/AI; (4) are 1v3 and boss outcomes intended?

> NOTE 2026-10-16: since the baseline run below, the game gained new class kits (M2C2B), the AI1 skill use (riposte finisher, taunt, disengage, areas), statuses and gear effects. BAL1 must A/B `AiProfile.smart_skills` and rerun duel, tier and level tables; XP curve values are starting values.
