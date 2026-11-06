# BASERULES1 values: this game's numbers and picked set

Companion of `docs/specs/BASERULES1.md`, holding what names this game's rows. Measured by a Python replica of the rules over `games/emberwake/data/catalog/` at `db55a63e`: 47 derived units, no build, no
passives or drills. A hit below is rules applied by hand. Numbers go to the balance lane; nothing here tunes.

## 1. Worked hit with shipped rows

`tl_vanguard` (`tl_drill`, level 1, STR 13) with `wpn_broadsword` (power 18, sword level 2, scaling 1.5) hits `tl_footman` (authored armour 25) with `strike`.

| Step | Value |
| --- | --- |
| Attack Power | `roundi(18 + 2 + 13 x 1.5)` = `roundi(39.5)` = 40 |
| Skill, Focus, Stance | 1.0 each |
| Armour | `25 / 75`, keeps 2/3 |
| Hit | 26.67, shown 27, 15 % of the footman's 175 HP |
| Under `guard` (0.3) | 18.67, shown 19 |
| A crit (6 %) | 40 |

## 2. How much each stat moves the outcome

`+4` on one stat (a level 1 unit's spend pool), mean over units, against a reference armour of 20. Effective HP is `HP / (1 - armor share)`.

| Stat | Party damage (max) | Party eHP (max) | Enemy damage | Enemy eHP | Move rate gains | Other |
| --- | --- | --- | --- | --- | --- | --- |
| STR | +4.9 % (15.8) | +6.2 % (7.8) | +7.1 % | +4.8 % | 0 | |
| DEX | +7.1 % (20.5) | 0 | +7.4 % | 0 | 9 of 18; 11 of 29 | Crit, initiative |
| INT | +3.4 % (21.4) | 0 | +1.1 % | 0 | 0 | Resist +10 |
| CONS | 0 | +17.4 % (23.5) | 0 | +15.7 % | 0 | Poise +8 |
| PERC | 0 | 0 | 0 | 0 | 0 | Range +1 for 4 of 18; 3 of 29 |
| KNOW | 0 | 0 | 0 | 0 | 0 | Heals on 6 skills, unlock from 12 |

- **Who reaches a threshold:** PERC 18: the `ranger` unit only. KNOW 12, 16, 20: 7, 5, 2 of 47.
- **Main weapons scale from:** STR 18 units, DEX 15, STR + DEX 8, INT 6. No weapon family may scale from PERC or KNOW, yet the party screen's PERC help says "power with PERC weapons" (`party.rs:297`).
- **Weapon share of attack power:** 40 % on average in the party (25-57 %).
- **Move rate:** party 4 tiles a block: 9, 3: 8, 2: 1; enemies 4: 17, 3: 10, 2: 2. Founders: Kaelen speed 12, rate 2; Maren and Elyse 15-16, rate 3; Vesper, Hollis and Dunstan 18-24, rate 4.
- **Armour:** party mean rating 8.5 (about 15 % less damage); gear holds 62 % of it. Three authored 50s (`captain_vane`, `inquisitor_prelate_nemeth`, `tl_captain`) halve damage. 14 of 47 units set `armor`.
- **Status resist vs potency:** 51 skills, potency 40-90, median 70. INT 6, 9, 12, 17 land 55, 47.5, 40, 27.5 %. The `mage` unit (level 5, 67.5 resist) takes 2.5 % at 70.
- **Crit:** 5-21 % across units.

### Attacks: value per block and AP

95 damaging attacks; 86 set `power_mult`.

| Blocks | Power min, median, max | Rows |
| --- | --- | --- |
| 1 | 0.20, 0.60, 0.95 | 25 |
| 2 | 0.40, 0.90, 2.20 | 45 |
| 3 | 0.40, 1.80, 2.40 | 17 |
| 4 | 1.00, 1.70, 2.20 | 8 |

Power per block: min 0.13, median 0.50, max 1.10; quartiles 0.40-0.60. By riders (status, area, stagger and the like): none 0.57, 1 rider 0.50, 2 0.50, 3 0.47, 4 0.55: **riders carry no price**. Highest: `throat_carver` 1.10, `orlovs_retribution` 1.00. Lowest: `choke_gas` 0.13, `firebomb`, `glacial_burst`, `magnesium_flash_bomb`, `scalding_grease_pot` 0.20.

A first band for balance to start from, not a target: `power_mult` per block on attacks 0.40-0.80 would flag about a quarter of the rows.

### Stale rows

- `feint` (0.7 power, `ends_in_stance` DEFENSIVE), `crushing_blow` (`disrupt` STANCE_BREAK), `shield_bash`, `intimidate` (`stance` DEFENSIVE): inert under timeline.
- `strike` versus `crushing_blow`: both 1.0 power, 2 blocks; AP 1 versus AP 2.
- `move` text: "Keeps your stance." `rest` text: "return to Neutral stance".
- Consumables: `brimstone_draught` `buff_stat` CONS at 0.3 and `con_focus_elixir` STR at 0.5: both boost attack only. Bombs 45 and 40 ignore armour and guard. Gear `shd_tower`, `wpn_glass_wand` have `block_cost` above 1, unread in battle.
- `composure` has no rule reader; only `portrait_cues.json` and a seed name it.

## 3. The set this game picks

Row `timeline`, as shipped, plus what the framework decisions bring:

| Bundle | Holds | State |
| --- | --- | --- |
| `intent_core` | `round_intent`, by-rules five | Shipped |
| `windup` | `enemy_moves`, `ap_cost`, `poise`, `poise_stat` `max_poise` | Shipped |
| `positioning` | `flank`, `reach`, `melee_diagonal` maneuver, `attack_lock` melee | Lock in lane 1 |
| `build` | `deck`, `basics`, `deck_slots` 3, 2, 1 | Shipped |
| `tells` | `plan_badges` | Shipped |
| `hooks` | All 9 | Shipped |
| `field` | INTENT2 K1, K2 | Approved |
| `pouch` | `pouch_stacks`, `item_ap` 0, `thrown_miss_spends` | Approved, K3 |
| `stand` | `guard_model` status | BASERULES1-G |

## 4. Which rows, which worlds

`game.json` would hold `ruleset: timeline`, `test_rulesets: [classic, feel_flat]`. `sunk_ward` names `timeline` already. `ashlands`, `iron_reaches` and `tidewrack` name none, so they read classic. `start_worlds` is still `ashlands`. The Ashlands is a test bed, not a campaign (Q4 in the main spec recommends leaving it on `classic`: the journey and PHASEFOLD goldens rest on it).

## 5. Asks for the balance lane (after BASERULES1-F)

1. Move rate: how many fights turn on the cap; a table that gives DEX a step over the whole range; 3-block Move reach against AP.
2. Same-block dodges by speed under `attack_lock` melee, against a `flat_speed` run.
3. Reference hit bands by level; `power_mult` bands per block and per AP.
4. `drill_scaling` 1.25 against 1.5 for the 7 drilled units.
