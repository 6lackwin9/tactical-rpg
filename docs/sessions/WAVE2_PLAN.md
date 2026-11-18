# `WAVE2_PLAN` — `[PROVISIONAL]` Wave 2 Master Plan (`PK2-STAGE-PASSIVES`, `WAVE2-BALANCE`, `PK3..PK8`)

- **Status**: **`[PROVISIONAL]`** — Active Wave 2 roadmap (`PR6-1`..`PR6-3`, `PR6-5`, `AI-15` landed; `ACTS23-R6`/`R7` `B1..B7` integrated and accepted by the user 2026-11-17; `PR6-4`, `PR6-6..PR6-13` queued in `READY.md` §1 for review & Build).
- **Baseline**: `origin/develop` (`f132eabc`: `PR5-57..62c` + Wave 2 `PR6-1`, `PR6-2`, `PR6-3`, `PR6-5` & `AI-15` `7ddae3fd`).
- **Locked Foundation**: [CANON.md](../../games/emberwake/docs/lore/CANON.md) (§1–§6 `[LOCKED]`), [ACTS23.md](../specs/ACTS23.md) (Q-A1..Q-A10, Channel §5.1, Techniques §5.2), [PK1_CORRUPTION.md](PK1_CORRUPTION.md), [PK2_GRAFTS.md](PK2_GRAFTS.md), [PRINCIPLES.md](../PRINCIPLES.md).

---

## 1. Wave 2 `.1` Session Index (`<= 300 L / <= 25 KB` per file)

| Domain Spec | Release Wave | Covered Systems & Landed / Queued Items | Target Queue |
| --- | --- | --- | --- |
| [PK3_ROSTER_LIFECYCLE.md](PK3_ROSTER_LIFECYCLE.md) | `R5-Finish` | `PR6-2` `PK2-STAGE-PASSIVES` (**done `51dd62b2`**), `PR6-3` Bleed-out & Revive (**done `872df55c`**), `PR6-4` Recruit Generator (`18-24` Act 2 Cohorts, `pool`, `Isolation Crypts`, `Living Martyrs`) & Procedural Keepsakes (`by_class` + `AI-15` `7ddae3fd` assets) | `PR6-4` (`§1` #1) |
| [PK4_PK5_CAMPAIGN_HUB.md](PK4_PK5_CAMPAIGN_HUB.md) | `R6` | `ACTS23-R6` **audited (`B2..B6` accepted by the user 2026-11-17)**: `PK4` (`FX_DOCTRINE`, `signed` lean tracks, strain telemetry, `after_flag`+`offset_days` Checkpoints, `clock_set.keep_longer`, Act 3 Hard Clock + `supply_price_mult_by_act` `2.0x` Tallow); `PK5` (`facilities` `5 × 3` + `sieges.json` w/ `Ruthven` `25%` `"rout"` + `hp_floor: 25`) | `PR6-6`, `PR6-8`, `PR6-9` (`§1`) |
| [PK6_PK7_BOSSES_ENDINGS.md](PK6_PK7_BOSSES_ENDINGS.md) | `R7` | `ACTS23-R7` **audited (`B1` Option B, `B7` accepted by the user 2026-11-17)**: `PK6` (Regional `start_meter`, `when_meter`+`every_rounds`+`ground.rs:408`, `"$captive_unit"`, `Step::CampPrep`, `retry_day_cost`, `echo_fallen`, `1x1` cell + `2x2` oversize art); `PK7` (`END_KINDS::ending`, `endings.json` `A / B / C`, `EpilogueScreen`) | `PR6-7`, `PR6-10`, `PR6-11` (`§1`) |
| [PK8_ACT23_CONTENT.md](PK8_ACT23_CONTENT.md) | `R5-Finish` -> `R7` | `PR6-1` `WAVE2-BALANCE` (**done `29dfc241`**), `PR6-5` Act 2 drops/books (**done `f132eabc`**), `sunk_ward` Act 2 `high_bastion` (`08-15` + deferred `pk2.*`) & Act 3 `mantle_fissure` (`16-22` + `A/B/C`), plus legacy parity (`iron_reaches`, `tidewrack`, `ashlands`) | `PR6-12`, `PR6-13` (`§1`) |

---

## 2. Wave 2 Execution Order & Release Gates

```text
43a6cbd8 (Wave 1 Release) -> f132eabc (PR6-1/2/3/5 + AI-15 DONE)
  ├─► PR6-4  [PK3-RECRUITS-KEEPSAKES]   (pool, Cohorts, crypts, Martyrs fuse, by_class keepsakes)
  ├─► PR6-6  [PK4-CAMPAIGN-HOOKS1]      (FX_DOCTRINE, signed lean, strain telemetry, keep_longer, after_flag)
  │     ├──► PR6-8  [PK5-HUB-FACILITIES1] (5x3 facilities, supply_price_mult_by_act 2.0x, UI)
  │     └──► PR6-9  [PK5-SIEGES1]         (sieges.json carry-over, Ruthven 25% "rout" + hp_floor: 25)
  ├─► PR6-7  [PK6-BATTLE-HOOKS1]        (start_meter, when_meter/every_rounds, "$captive_unit")
  │     └──► PR6-10 [PK6-STORY-PREP1]     (Step::CampPrep, retry_day_cost, echo_fallen)
  └─► PR6-11 [PK7-ENDINGS1]             (END_KINDS::ending, endings.json, EpilogueScreen)
        └──► PR6-12 [PK8-ACT2-CONTENT] -> PR6-13 [PK8-ACT3-CONTENT] (sunk_ward 08-22, 2x2 art, sim parity)
```

- **Strict Parity Invariant (`P-ARCH-02`, `P-CAMP-01`)**: `ashlands`, `iron_reaches`, and `tidewrack` stay `classic` ruleset fixtures (`0` Act 2–3 bleed-out, Strain, durability, or Martyr mechanics leak into classic fights; `journey_golden`, `phasefold_golden`, and `rules_contract_golden` change ONLY when a slice explicitly names a golden update in [READY.md](../READY.md)).
- **Dual Layer Split (`P-ARCH-01`, `fc.sh layers`)**: Every new rule/hook added under `kits/rpg/` and `engine/` uses strictly generic RPG terminology (`unit_tracks.signed`, `FX_DOCTRINE`, `supply_price_mult_by_act`, `bleed_out_rounds`, `revive`, `recruits`, `keepsakes`, `facilities`, `siege`, `"rout"`, `ending`, `"$captive_unit"`) with zero `games/emberwake/lore_words.txt` words.

---

## 3. `[PROVISIONAL]` Decisions Requiring User Playtest / Review Sign-Off

1. **`WAVE2-BALANCE` Ratified (`PR6-1`, `29dfc241`)**: Brawler (`gauntlet 15`, `forearm_braces +1 mit`, `brawler_clinch 0.95x`, `slag_charge 1.60x` -> `brawler -2.6`), `sim.json` `line.enemy_mastery: {"warrior": 1, "rogue": 1}` (`line 88%` sweep / `83%` party), and Mage AoE `0.32/0.30/0.32/0.25` (`mage -0.6`, `boss 32%` sweep / `40%` party, Crucible `0%`); verify `elite 78–80%` stays comfortable during Act 2/3 tuning.
2. **Revive & Bleed-Out (`PR6-3`, `872df55c`)**: Priest `L2` `rally.revive = true` + `smelling_salts` (`1 AP / 1 blk / 1 reach`, `heal: 15`, `15 Scrap` at `sabina_slab`/`varka_trade` behind `pk2.slab_open`), `3`-round Act 2–3 bleed timer, and Chebyshev `1..=2` `(y, x)` nearest-cell revive.
3. **`ACTS23-R6` / `ACTS23-R7` Feasibility Rulings (`B1..B7`)**: Keep `1x1` cell occupancy + oversized `2x2` boss sprite art (`B1` Option B, multi-cell `BIGUNIT1` parked in `READY.md` §3; superseded 2026-11-17: `BIGUNIT1` is built as `PR7-12`); `2–3` connected Siege rooms per Alert Breach (`Ruthven` `25%` `"rout"` + `hp_floor: 25`, damaged facilities `-1` level floor `1`); Act 3 Warded Tallow `2.0x` (`8g -> 16g`).
4. **Endings & Legacy Sample Worlds (`PK7` / `PK8`)**: Lock `iron_reaches`, `tidewrack`, and `ashlands` as classic parity/route fixtures (`[KEEP-PARITY]`) while placing `High Bastion` (`08-15`) and `Mantle Fissure` (`16-22`) inside `sunk_ward` (`ACTS23` Q-A2(a)).
