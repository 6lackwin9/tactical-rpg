# Balance report (written by the balance agent; read by the brainstorm agent)

## Archive Index

| Archive | Scope | Summary |
| --- | --- | --- |
| [BALANCE_REPORT_2026-11A.md](../../../docs/archive/BALANCE_REPORT_2026-11A.md) | `PR5-27` .. `R1-BAL`, `INPUT1-MB`, `RULESET1`, `REWARD1-B`, `BAL4/5` (`2026-10-18 .. 2026-11-10`) | All pre-`U3_GEAR_REWORK` Channel Duel, `R4-CLOSE` (`61.0%` feel / `92%` party), `R1..R4`, and classic Ashlands balance runs |
| [BALANCE_REPORT_2026-10.md](../../../docs/archive/BALANCE_REPORT_2026-10.md) | `2026-10-17` initial runs | Earliest October 2026 baseline snapshot |

## 2026-11-20 ENGINE-27 CRUCIBLE-CHECK1 why both Crucible fights are won 0 % (`FB-389`)

Report only: no data, rule or golden change. Replayed `enc_evt_06` and `enc_evt_06_overlook` round by round (the feel
sim's loop: `auto_draft` balanced, the world's timeline ruleset, 32 seeds each, a scratch test not kept) with the gate,
the holder, every unit's HP and tile, and the outcome printed each round.

**No rule bug.** Every part of the fight works as specified: the holder is picked by `highest_hp` (Kaelen) and walks
to the winch (10, 1) by round 2; the gate opens on round 2; the `arrival` wave lands on round 3 (round 4 in the
Overlook variant) at (6, 4) and (7, 4); `infusion` fires on round 4; Nemeth is never finished; heroes withdraw through
the open gate. The objective is `escape` with `objective_count` 3 and four founders (`party_min` 4), so the three who
are not holding must all get out: **any founder down makes the win impossible** (the fight then ends `fled` or `lost`).

| Party level (authored 1) | Base: won | Base: round-1 founder deaths | Overlook: won | With `objective_count` 2: base / Overlook |
| --- | --- | --- | --- | --- |
| +0 (as the feel sim runs it) | 0/32 | 24 | 0/32 | 0 / 0 |
| +1 | 0/32 | 19 | 0/32 | 0 / 0 |
| +2 | 0/32 | 19 | 0/32 | 0 / 0 |
| +3 | 0/32 (32 fled) | 4 | 24/32 | 30 / 28 |
| +4 | 0/32 (32 fled) | 2 | 27/32 | 32 / 31 |

Two causes, in order:

1. **Party level.** The feel sim fights every world encounter with the founders at their authored level 1; the
   Crucible's foes are level 2-6 (Nemeth 6). Up to level 3 a founder falls in round 1 in most seeds (the line starts
   two tiles from seven foes), which ends the escape before it starts. The other Act 1 bosses are winnable at level 1;
   the Crucible is the one fight tuned for a levelled party (`FB-253`: tuned for human play).
2. **The round-3 wave in the escape lane.** At level 4 few founders fall in round 1, but the base fight still wins
   0/32: the escapers cross to the exits at column 11 through the enemy line, the two wave footmen arrive on round 3
   in that lane, and one escaper falls on the way in every seed. The Overlook variant (wave on round 4) wins 24/32 at
   the same level.

Smallest fixes (data; they wait for the user with the other boss calls): (a) state the party level the Crucible is
tuned for and let the feel sim fight it there (at level 4 the Overlook is 75 %, the base 0 %); (b) the base wave on
round 4 as in the Overlook (the base then plays like the Overlook); (c) `objective_count` 2 (94 % at level 4, but
CANON's "one holds, three get out" changes). Open question: no sim models the founders' real level at M5 (there is
no Act 1 XP campaign); below level 4 the fight stays unwinnable for the auto driver whatever the wave does.

## 2026-11-19 ENGINE-26 BALANCE-WHATIF1 what-if runs (`FB-381`)

Report only: no data or golden change; the user picks the data change. Sunk Ward, timeline rules with the ENRAGE2
enrage, the authored parties, 32 seeds per fight, `aegis sim feel pace=on` with the new what-if keys `foe_hp=`,
`foes=` and `foe_level=` (the foes as changed for the run only). Minutes are the EN7b estimate (10 s planning per hero
per round plus 5 s, 0.42 s playback per block, 0.9 s between rounds).

**1. Random fights** (the 10 escape fights together, 320 fights; target median 2-3 rounds with won % at least 85):

| Change | Won % | Escaped % | Won median rounds | Minutes |
| --- | --- | --- | --- | --- |
| As authored | 100 | 0 | 1 | 0.8 |
| Foe HP +25 % | 100 | 0 | 1 | 0.8 |
| Foe HP +50 % | 100 | 0 | 1 | 0.8 |
| Foe HP +100 % | 100 | 0 | 1 | 0.8 |
| One more foe | 99.4 | 0.6 | 2 | 1.6 |
| One more foe, HP +25 % | 100 | 0 | 2 | 1.6 |
| One more foe, HP +50 % | 100 | 0 | 2 | 1.6 |
| One more foe, HP +100 % | 98.4 | 1.6 | 2 | 1.6 |
| Two more foes | 98.1 | 1.9 | 3 | 2.4 |

HP alone does not move the median (foe HP x4 first reaches 2 rounds, x10 reaches 4 with 22 % escaping). The smallest
change that reaches the target is **one more foe** per random fight (median 2); two more reach median 3.

**2. Elite minutes** (four heroes and 6 blocks in every elite fight):

| Fight | Won % | Won median rounds | Minutes now |
| --- | --- | --- | --- |
| Corvo's Gate | 93.8 | 5 | 4.0 |
| Proctor Wardens | 100 | 3 | 2.4 |
| Kiln Haulers | 96.9 | 3 | 2.4 |
| Guild Bruisers | 96.9 | 4 | 3.2 |

At the elite round target of 5, today's estimate reads 4.0 minutes. Five rounds read 7 minutes at **18.9 s** of planning
per hero per round and 8 minutes at **21.9 s** (today's estimate: 10 s). Whether either fits real play is for the
user's playtest.

**3. Bosses** at the authored level and with every foe one level up (`foe_level=1`), beside the approved boss band of
20-40 % won:

| Fight | Won % (authored) | Median rounds won | Won % (+1 level) | Median rounds won |
| --- | --- | --- | --- | --- |
| The Sweep | 53.1 | 10 | 50.0 | 10 |
| The Crossroads Cages | 84.4 | 7 | 81.3 | 8 |
| The Bone-Mill Raid | 28.1 | 7 | 25.0 | 6.5 |
| The Tallow Vats | 56.3 | 9 | 40.6 | 8 |
| The Awakening Fissure | 56.3 | 10 | 46.9 | 10 |
| The Crucible | 0 | - | 0 | - |
| The Crucible (Overlook) | 0 | - | 0 | - |

Only the Bone-Mill Raid sits in the band as authored; +1 level brings the Tallow Vats to its top edge (40.6 %). The
Sweep, the Cages and the Fissure stay above it at +1 level, and both Crucible fights are never won at either level
(lost 28 of 32 as authored).

## 2026-11-19 PR7-55 ENRAGE2 + PR7-56 PHASEBREAK1 boss run (`FB-377`)

- Run: `aegis sim feel enc=<all 24 Sunk Ward encounters> seeds=32 pace=on`, rule off (`tune=enrage_from_round:0`) and
  on (the shipped timeline numbers: from round 6, minion and standard 10 %, elite 15 %, boss 20 %, 4 steps), both with
  the phase break and the new caps (mob 6, elite 8, boss 12, random Sunk Ward fights 5 with the foes escaping).
- Enrage, off to on (won %): The Sweep 56 to 53, The Cages 84 to 84, Bone-Mill 28 to 28, Vats 62 to 56, The Awakening
  Fissure 66 to 56 (the largest drop, inside the 10-point band), Crucible 0 to 0 (tuned for human play,
  `FB-253`); every fight won in 5 rounds or fewer is unchanged; Vats and the Fissure median 0.5 rounds shorter. The
  steps stay as specified (no tuning needed inside the band).
- Pace by tier, rule on (EN6, EN7, EN7b; minutes = won median rounds x (10 s per hero + 5 s + 6 blocks x 0.42 s +
  0.9 s)): mob (prologue 3) won 95.8 %, capped 4.2 %, median 4, 1.8 min; random (10) won 100 %, capped 0 %, median
  1, 0.8 min; elite (corvo gate and 3 elite roamers) won 96.9 %, capped 2.3 %, median 4, 3.2 min; boss (7) won
  39.7 %, capped 5.4 %, median 9, 7.3 min. Caps: mob and elite under the 5 % target; boss median inside 8-11.
- Misses, not tuned (EN8 raises lethality, which would make them shorter still; listed for Design): random fights
  end in 1 round (PACE1 mob band 2-4); elites take about 3 minutes against the 7-8 target (at 5 rounds the EN7b
  estimate is about 4 minutes, so the elite minutes target and the round target disagree); boss win rate is low at the
  authored levels (Bone-Mill 28 %).
- Stun and stagger (blocks lost per fight; SV5): mob 0 and 1.7, random 0 and 0, elite 0.8 and 0.8, boss 0.2 and 1.5.

## 2026-11-19 PR7-42 pacing run (`FB-348`)

- Feel (10 seeds, authored party), unchanged by PR7-42: Rooftop 100%, Corvo gate 90%, The Sweep 30%, The Cages 80%,
  Bone-Mill 20%, Vats 50%, The Crucible 0%.
- `sim bands` already out on develop: duel ranger 63, priest 32, brawler 64; party line clear 75 (band 80-95).
  Tracked for `PR7-20` (BALANCE-TUNE1).

## Current Baselines & Open Issues (`U3_GEAR_REWORK`, `PR5-51b` .. `PR5-57`)

- **`PR5-51b` (`30415b91`, `FB-268`) `sim party` / `sim feel` sweep**: `sim party` boss clear jumped to **`80%`** (target band **`30-50%`**), driven primarily by `mage` `orb` `power_stat_override` (`INT` substitution lifting boss damage `214 -> 396`), while `warrior` sustained tank pressure and output dip; `sim feel` `Crucible` (`world:sunk_ward/enc_evt_06`) remains **`0/10`** auto-sim wins and stays tuned for human play per `FB-253`.
- **Action queued in `PR5-57` (`GEAR-REWORK1-FOLLOWUP`)**: add `counts=on` `RULE_COUNTERS` (`ammo_empower`, `reload`, `knife_throw`, `orb_override`), audit `orb` scaling, and retune `sim party` boss clear back into **`30-50%`** + check `warrior` tank/power balance without weakening human `Crucible` (`FB-253`).
- **Pre-rework `PK1` & `R4-CLOSE` baselines (reference in [BALANCE_REPORT_2026-11A.md](../../../docs/archive/BALANCE_REPORT_2026-11A.md))**: `PR5-27` (`2026-11-10`) Channel Duel `37.5%` won with `Vent -50 Strain / 10% HP` (`channel_power = 1.25`, `channel_ai_edge = 1.2`); `R4-CLOSE` (`2026-11-08`) `sim feel` `61.0%` overall, `sim party` `100/92/100/92/92%` (post-boss HP `0.31`), campaign `arms` `3/6`, `gate` `5/6`, `goblin_skirmish` `17.5%` (20/40 capped, held in `POST-R4-CLEANUP`).
