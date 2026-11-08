# Design session E: Economy, progression and roster (`E1.1-E3.1`, closed 2026-11-07)

Design record for the Economy and Progression block (`E1.1` Levels & XP, `E2.1` Purse, shop, stash & rewards, `E3.1` Roster, wounds, housing & keepsakes). Eliminates the hardcoded `SUPPLIES_MAX = 99` special case in `E2`.

## 1. `E1.1` — Levels, XP, spend pool and skill unlocks (`C02`, `C26`)

- **Reuses `progression.rs`**:
  - XP curve in data (`games/emberwake/data/progression/`), levels `1..10` (`max_level = 10` tunable).
  - Unit stat spend pool `4 + (level - 1)` spent explicitly on `LevelUpModel` or allocated automatically via `U2`'s `growth_shares` (`FB-10`).
  - Skill path unlocks by level (`family_unlock_level + rank_unlock_offset`, `SK2`); early-unlock passives sit in `U4` (`grant_skill`, `U1.1` move-out).
- **Stage-Scaled Enemy XP (`FB-13`, `FB-139`)**:
  - Enemy XP reward weight in `roster()` scales by the stage-resolved enemy level (`base_level + stage * growth_per_stage`, where `stage = campaign.act` in campaign worlds and `0` otherwise, and `growth_per_stage` is a per-unit `int` field on `units.json` defaulting to `0`, applied in `Roster::entry` to enemies only — `tier != "hero"`, excluding party guests).

## 2. `E2.1` — Purse, shop, stash and rewards (`C19`, `C22`)

- **Parameterize `SUPPLIES_MAX` and `GOLD_MAX` (`FB-53`, Fixes `MASTERPLAN1` §1 Special Case)**:
  - Because `Party` and `purse` are global across worlds and `gold_max` is already an encounter check key, store `supplies_max` (`99` default) and `purse_gold_cap` (`9_999_999` default) as game-wide `PurseCaps` on `game.json` (clamped on load so no existing save breaks, and shared by `state.rs` and `check/{world,map,quest}.rs`).
  - `E3` (`roster.rs`: keepsakes, party size, bunks, wounds, settle) is already built; `E3.2` is a contract-listing pass (`FB-53`).
- **Shop Trade, Reputation Bands, and Loot Pools**:
  - All-or-nothing shop settlement (`shop::trade`: integer buy `ceil`, sell `floor`, stock tracked in `GameState.shops`, prices modified by `U4` faction reputation bands).
  - Quest and encounter rewards (`RewardGrant`): `xp`, `items`, `passives`, `skills`, `gold` (cap `5000`, soft `200`), and per-world `loot_pool` picks (`0..loot_picks`, claimed entries removed from other pools).

## 3. `E3.1` — Roster, wounds, housing, departures and keepsakes (`C18`, `C19`, `ROSTER1-L`)

- **Reuses `roster.rs`**:
  - Party members, active party size (`party_max`, default `4`), and housing bunk caps (`RosterCaps`).
  - Post-fight settlement (`settle_party_side` in `C19`): `2` hurt/wound modes (`wounded` stat cut cleared `on_rest`) and `4` loss/departure modes across `5` fates and `7` departure causes.
  - Departing members leave their named `keepsake` item (`keepsake_item_errors`), and recruited characters join at `join_xp = max(base_level_xp, highest_active_xp)` carrying their `U4` `temperament` (`FB-29`).

## 4. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| All-or-nothing shop trade, loot pool claim rules & `settle_party_side` order | **Fixed (Locked)** | Party settled before rewards (`C19`) | Core `E1-E3` loop |
| `max_level` / `base_spend_pool` / `spend_per_level` | **Tunable** | `10` / `4` / `1` | `E1` progression |
| `supplies_max` / `gold_max` | **Tunable** | `99` / `9_999_999` | Removes `SUPPLIES_MAX` hardcode |
| Wound stat penalty & bunk caps | **Tunable** | Manifest `roster` block | `E3` roster rules |

## 5. Acceptance tests (`E1.2-E3.2`)

1. **Data-driven `supplies_max` (`E2.2`)**: Setting a custom `supplies_max` in a world manifest purse block caps supply purchases and loot grants at that number without code changes; default `99` keeps `journey_golden` byte-identical.
2. **Stage-scaled XP & single unit builder (`C26`, `FB-12`, `FB-13`)**: Leveling up a hero and settling a stage-scaled world fight through `progression.rs` and `roster.rs` produces identical unit stats in party preview and battle entry.
