# Design session F: Fight flow (`F1.1-F4.1`, closed 2026-11-07)

Design record for the Fight Flow block (`F1.1` Round & blocks, `F2.1` AP & rest, `F3.1` Intent & planning, `F4.1` Encounter rules). Unblocks `ATTACKLOCK1-O`.

## 1. `F1.1` — Round, blocks, initiative and attack lock (`C11`, `ATTACKLOCK1-O`)

- **6-Block Timeline (`total_blocks = 6`)**:
  - Actions resolve simultaneously block by block (`1..6`).
  - Within-block resolution order is `priority * 1000 + initiative`, where `initiative` reads `U1`'s `Initiative` rating (`DEX` minus gear/carry burden per `FB-07`), and `priority` is `Defend = 1`, `Target = 2`, all others `0` (`command.rs:276`).
  - Wind-up runs from `start_block` to `impact_block = start_block + blocks - 1`; under `enemy_moves`, enemies pay no AP and book `recovery_blocks` after impact.
- **Switch on `attack_lock = melee` (`ATTACKLOCK1-O`, `FB-58`)**:
  - Unblocks adding the `power > 0` check in `command.rs` `lock_tile` (so zero-power skills `intimidate` and `tripwire_snare` do not lock) and switching `attack_lock` to `melee` in the timeline ruleset in its own named commit (`F1.2-S1`, no code dependency on `A3.2`):
    - Single-target attacks with `power > 0` in melee band (`reach_min == 1` and `reach_max <= 2`) lock to the target's tile when wind-up starts (`command.rs` `lock_tile`, after any opener chase/dash) and miss with `[Dodged]` if the target moves off that tile by its own movement before impact.
    - `Field` and directional shaped skills lock to their ground tiles at wind-up start (`A3.1`, `FB-57`).

## 2. `F2.1` — AP, rest and overspend debt (`C10`, `C11`, `FB-58`)

- **AP Pool (`U1` / `S2` Pool facade)**:
  - Default `unit_ap = 12`, `start_ap = 10`, `ap_regen = 4` per round; Rest under `rest_ap = round_end` restores `min(rest_cap = 2, floor(blocks * rest_per_block = 0.5))` AP at the next round start.
- **Data-Driven Overspend Debt Statuses (`C10`, `FB-58`)**:
  - Planning into negative AP is allowed up to the debt ceiling; ending a round plan at `>= debt_stagger` (`3` AP debt) or `>= debt_stun` (`5` AP debt) inserts the ruleset-configured status ID (`debt_stagger_status = "stagger"`, `debt_stun_status = "stun"`) directly into `st.active` with consistent `rounds_left` (`status.rs:436-439`, so it applies on the upcoming round rather than landing a round late via `queue`), promoting `DEBT_STAGGER` and `DEBT_STUN` from `LOCKED` to ruleset numbers (`F2.2-S1`).

## 3. `F3.1` — Intent and planning (`INTENT2-K1`)

- **5 Visible Round Intents + Hidden `Channel`**:
  - `Assault`, `Stand`, `Support`, `Maneuver`, and `Field` (plus `Channel` when a unit owns a `U4` `grant_pool` meter).
  - One-flow ghost fill (`INPUT1-U`), `field_fill_once = on`, row-aware dual-intent skill picks (`A4.1`, `FB-19`), and `Stand`'s 1-tile reposition booked as a standard `Move` command (`FB-22`).

## 4. `F4.1` — Encounter rules (`C17`, `FB-58`)

- Reuses the built encounter engine (`encounter.rs`, `script.rs`): board setup, win/loss/escape objectives, boss phases (9 effect kinds: `line`, `wave`, `status`, `ai`, `enrage`, `deck`, `shake`, `kill`, `talk`), scripted rounds (`1..3` rounds), stay-behind (`HoldGate`), and timeout/flee.
- Encounter phase `status` effects route through `A4.1`'s unified `EffectSpec` with default `resistable = Some(false)` (`grant_now`, no resist roll unless explicitly set on the phase row, `FB-58`).

## 5. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| 6-block timeline, simultaneous resolution, wind-up/impact/recovery | **Fixed (Locked)** | `priority * 1000 + initiative` | Core `F1` resolver loop |
| `attack_lock` | **Tunable** | `melee` (`none` \| `melee` \| `all`) | `ATTACKLOCK1-O` switch-on |
| `unit_ap` / `start_ap` / `ap_regen` / `rest_per_block` / `rest_cap` | **Tunable** | `12` / `10` / `4` / `0.5` / `2` | `F2` AP economy |
| `debt_stagger` / `debt_stun` & status IDs | **Tunable** | `3` (`stagger`) / `5` (`stun`) | `F2` overspend debt |

## 6. Acceptance tests (`F1.2-F4.2`)

1. **`F2` data-driven debt statuses**: Replacing hardcoded `"stagger"`/`"stun"` debt literals with ruleset status IDs keeps `rules_golden` and `timeline_golden` byte-identical, and allows a custom ruleset to map AP debt to any `S1` status row.
2. **`ATTACKLOCK1-O` switch-on**: Enabling `attack_lock = melee` in its own named commit (with `HISTORY.md` row and golden re-record) locks melee (`1..2`) single-target swings at wind-up start while leaving ranged attacks tracking and shaped skills tile-locked.
