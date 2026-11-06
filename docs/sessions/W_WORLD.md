# Design session W: World, travel, campaign clock and battle seams (`W1.1-W3.1`, closed 2026-11-07)

Design record for the World block (`W1.1` Region map & interaction, `W2.1` World map & travel, `W3.1` Campaign clock, meters & upkeep) and the World-to-Battle seams (`C18` `WORLDFIGHT1`, `C19`, `C20`, `C21`, `C23`).

## 1. `W1.1` & `W2.1` — Region map, interaction and world travel (`C22`, `C23`)

- **Region Map (`W1`, `explore.rs`)**:
  - Reuses the built region runner: `7` marker kinds (`npc`, `chest`, `lore`, `exit`, `world_map`, `encounter`, `roamer`, `point`), touch or proximity interaction, flag gates (`unless_flags`), `5` services (`SERVICES1` screen seam `C22`), and leader cycling (`explore_leader`).
- **World Map & Travel (`W2`, `travel.rs`)**:
  - Nodes, legs, travel modes (`road_chance`, `leg_days`, `best_path_with`), scripted road fights, and cross-world `connects` transitions (`home_world` preserved across hops).

## 2. `W3.1` — Campaign clock, meters and upkeep (`C20`, `C21`)

- **Reuses `campaign/` (`FB-50`)**:
  - Gauge and countdown meters sharing `bands.rs` with `U1`/`S2` unit meters.
  - Checkpoints and deadline previews (`aegis sim deadlines`), `tick(Cause)` triggered by `Cause` (`Days`, `Leg`, `Battle`, `Enter`, `Effects`, plus act change), daily supply upkeep, rest rules, and defeat handling (`wake`, `retreat`, `run_over`). `W3.2`'s clock parameters are already manifest data (`campaign`, `purse`, `rest`, `defeat`), so `W3.2` is a contract listing only (`FB-50`).

## 3. World-to-Battle seams (`C18` / `WORLDFIGHT1` and `C19`, `FB-50`)

- **Slices (`FB-50`)**:
  - **`W.2-S1` (`WorldRoster::new` + `finish_world`)**: Replace the 16 `WorldRoster { d, world }` struct literals with `WorldRoster::new` and move runner dispatch / `FIGHT_SOURCE` settlement into kit `finish_world` (`0` golden diffs).
  - **`W.2-S2` (`begin_world` + round-trip test, after `READY` row 10)**: Build party and enemy units in kit `begin_world` (returning units with a callback for GUI-only `Prebuilt` art) and add a headless world-fight round-trip test.
  - **`W.2-S3` (`stage` from `campaign.act` + `SERVICES1`)**: Wire `stage` from `campaign.act` after mob growth lands, followed by `SERVICES1` (`C22`).

## 4. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| `begin_world` -> battle -> `finish_world` settlement order (`C18-C21`) | **Fixed (Locked)** | Party first, then rewards, then clock tick | Core `W` <-> battle seam |
| Travel mode `leg_days` & `road_chance` | **Tunable** | World manifest `travel` / `routes` | `W2` travel pacing |
| Campaign meter bands, decay, upkeep & checkpoint days | **Tunable** | World manifest `campaign`, `purse`, `rest`, `defeat` | `W3` pressure rules |

## 5. Acceptance tests (`W1.2-W3.2`, `C18-C21`)

1. **`WORLDFIGHT1` round-trip (`C18`, `C19`)**: Starting a world fight via `begin_world` at `campaign.act > 0` scales enemy levels by stage (`FB-13`), applies party wounds/grants/temperaments via the single `FB-12` builder, and settles party fates, stage-scaled XP, and campaign clock ticks on `finish_world`.
2. **Journey golden stability**: `emberwake_world/tests/journey.rs` passes byte-identical across seeded world walks before new content changes.
