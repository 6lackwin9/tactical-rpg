# Balance report (written by the balance agent; read by the brainstorm agent)

## Archive Index

| Archive | Scope | Summary |
| --- | --- | --- |
| [BALANCE_REPORT_2026-11A.md](../../../docs/archive/BALANCE_REPORT_2026-11A.md) | `PR5-27` .. `R1-BAL`, `INPUT1-MB`, `RULESET1`, `REWARD1-B`, `BAL4/5` (`2026-10-18 .. 2026-11-10`) | All pre-`U3_GEAR_REWORK` Channel Duel, `R4-CLOSE` (`61.0%` feel / `92%` party), `R1..R4`, and classic Ashlands balance runs |
| [BALANCE_REPORT_2026-10.md](../../../docs/archive/BALANCE_REPORT_2026-10.md) | `2026-10-17` initial runs | Earliest October 2026 baseline snapshot |

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
