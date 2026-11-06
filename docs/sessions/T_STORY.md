# Design session T: Story, quests, flags, dialogue rolls and vignettes (`T1.1-T2.1`, closed 2026-11-07)

Design record for the Story block (`T1.1` Dialogue, quests, flags & stat rolls, `T2.1` Sequences, vignettes & story fights) and seams `C22`, `C24`, `C25`.

## 1. `T1.1` — Dialogue, quests, flags and deterministic stat rolls (`C22`, `C25`)

- **Reuses `dialogue.rs`, `quest.rs`, and `board.rs`**:
  - Flag store (`GameState.flags`) with `6` comparison operators gating region markers, travel routes, shop stock, dialogue choices, and vignette cards (`C25`).
  - Dialogue nodes with `5` condition kinds and `9` effect kinds (kept as direct calls per `SILOS1` T7), `U4` faction reputation bands, and service replies opening shop, recruit, or mission board screens (`SERVICES1`, `C22`).
  - Lore-free quest templates bound by world quests with flag-conditioned `variants` (`VARIANT_FIELDS`) and all-or-nothing completion (`QuestRunner::complete`).
- **Dialogue Stat Check: Threshold or Seeded Roll (`U1.1` -> `T1.1`, `C25`)**:
  - A dialogue `stat` condition reads any `U1` primary stat or standing derived rating and supports two modes:
    1. **Flat threshold (`mode: "threshold"`, default)**: Succeeds when `unit_stat >= threshold` (preserving existing dialogue checks).
    2. **Seeded roll (`mode: "roll"`, `U1.1` decision)**: Rolls `1..roll_die` (`d20` default) on the save's deterministic `GameRolls` stream plus the unit's stat bonus against `dc` (`roll + stat_value >= dc`), recording the outcome flag so the check cannot be save-scummed and remains 100% deterministic in `journey_golden`.

## 2. `T2.1` — Sequences, vignettes and story fights (`C24`)

- **Reuses `story.rs` and `aegis_ui::vignette`**:
  - `4` card styles, `5` sequence step kinds, `5` camera shake presets (`SHAKE_PRESETS`), and portrait expression cues (`PORTRAITFX1`).
  - Story fight step (`C24`, `FIGHT_SOURCE`): launches a battle from a running sequence and handles `won`, `retry`, `abort`, or `continue` outcomes cleanly.

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| Flag operators, quest variant resolution & seeded `GameRolls` dialogue roll | **Fixed (Locked)** | `threshold` and `roll` modes on `U1` stats | Core `T1` contract |
| `dialogue_roll_die` | **Tunable** | `20` (`6..100`) | Die size for `"roll"` mode dialogue checks |
| Sequence card styles & shake presets | **Tunable** | `story.json` / `vignettes.json` | `T2` presentation data |

## 4. Acceptance tests (`T1.2-T2.2`)

1. **Seeded dialogue stat roll (`T1.2`)**: A dialogue choice with a `"roll"` stat condition draws from `GameRolls`, succeeds/fails deterministically by `stat_value` + seed, and cannot be re-rolled by reloading the same state.
2. **Cross-world quest & sequence stability**: `story_checks.rs` and `journey_golden` pass unchanged on shipped data.
