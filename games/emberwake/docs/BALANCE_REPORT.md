# Balance report (written by the balance agent; read by the brainstorm agent)

## Archive Index

| Archive | Scope | Summary |
| --- | --- | --- |
| [BALANCE_REPORT_2026-11A.md](../../../docs/archive/BALANCE_REPORT_2026-11A.md) | `PR5-27` .. `R1-BAL`, `INPUT1-MB`, `RULESET1`, `REWARD1-B`, `BAL4/5` (`2026-10-18 .. 2026-11-10`) | All pre-`U3_GEAR_REWORK` Channel Duel, `R4-CLOSE` (`61.0%` feel / `92%` party), `R1..R4`, and classic Ashlands balance runs |
| [BALANCE_REPORT_2026-10.md](../../../docs/archive/BALANCE_REPORT_2026-10.md) | `2026-10-17` initial runs | Earliest October 2026 baseline snapshot |

## Current Baselines & Open Issues (`U3_GEAR_REWORK`, `PR5-51b` .. `PR5-57`)

- **`PR5-51b` (`30415b91`, `FB-268`) `sim party` / `sim feel` sweep**: `sim party` boss clear jumped to **`80%`** (target band **`30-50%`**), driven primarily by `mage` `orb` `power_stat_override` (`INT` substitution lifting boss damage `214 -> 396`), while `warrior` sustained tank pressure and output dip; `sim feel` `Crucible` (`world:sunk_ward/enc_evt_06`) remains **`0/10`** auto-sim wins and stays tuned for human play per `FB-253`.
- **Action queued in `PR5-57` (`GEAR-REWORK1-FOLLOWUP`)**: add `counts=on` `RULE_COUNTERS` (`ammo_empower`, `reload`, `knife_throw`, `orb_override`), audit `orb` scaling, and retune `sim party` boss clear back into **`30-50%`** + check `warrior` tank/power balance without weakening human `Crucible` (`FB-253`).
- **Pre-rework `PK1` & `R4-CLOSE` baselines (reference in [BALANCE_REPORT_2026-11A.md](../../../docs/archive/BALANCE_REPORT_2026-11A.md))**: `PR5-27` (`2026-11-10`) Channel Duel `37.5%` won with `Vent -50 Strain / 10% HP` (`channel_power = 1.25`, `channel_ai_edge = 1.2`); `R4-CLOSE` (`2026-11-08`) `sim feel` `61.0%` overall, `sim party` `100/92/100/92/92%` (post-boss HP `0.31`), campaign `arms` `3/6`, `gate` `5/6`, `goblin_skirmish` `17.5%` (20/40 capped, held in `POST-R4-CLEANUP`).
