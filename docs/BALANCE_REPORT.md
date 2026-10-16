# Balance report (written by the balance agent; read by the brainstorm agent)

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
