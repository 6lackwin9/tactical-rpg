# Items, gear, ossuary growths and progression

Status: **`[LOCKED]` (`2026-11-10` Lore & Visual Hooks Session)** — equipment slots, `POUCH_MAX = 3`, weapon families (`WEAPONROSTER1`), **Ossuary-Relic Growths (refined via `M2` `Slab Retorts`) vs. Hearth Salvage**, **Tactile Soot-Gothic Pouch Consumables**, and all **Named Campaign Relics & Crucible Keepsakes**, matching [CANON.md](../CANON.md) Section 5 (`PR5-35` syncs shipped `items.json` via `aegis content`).

Engine facts (`ITEM1`, `C1-L`, `ITEMUSE1`, `REWARD1`, `WEAPONROSTER1`): items live in `games/emberwake/data/catalog/items.json` on `item_families.json` (`rpgkit_data::ItemDef`); world named items resolve to a family; content is edited only through `aegis content`.

## 1. Equipment slots, Slab implant slots & battle pouch (`[LOCKED]`, updated `2026-11-14` [U3_GEAR_REWORK.md](../../../../../docs/archive/sessions/U3_GEAR_REWORK.md))

| Slot | Engine name | Holds |
| --- | --- | --- |
| `MAIN` | `MAIN_HAND` | Primary weapon (`15` families across `2H Solo`, `2H + Rig`, `1H Heavy`, `1H Light`); sole earner of weapon Mastery XP (`Prof >= 2`) |
| `OFF` | `OFF_HAND` | `7` dedicated `off_only` families (`shield`, `buckler`, `braces`, `ammo`, `cloak`, `orb`, `tome`) or `1H Light` weapon (`dagger`, `knife`, `fist`); drives the Combat UI **Off-Hand skill button** (`Switch Weapon` retired; empty on `2H Solo`, `ammo`/`cloak` pair with `crossbow`/`bow`) |
| `HEAD` | `HEAD` | Heavy kettle helms, medium hoods/cowls, light masks/veils (`heavy`, `medium`, `light` proficiency shared with `BODY`) |
| `BODY` | `BODY` | `heavy`, `medium`, or `light` armor (`6` unique `[3]/[1]` class permutations; replaces `plate`/`leather`/`cloth`) |
| `ACC1`, `ACC2` | `ACCESSORY_1`, `ACCESSORY_2` | Proficiency-free relics, winch ratchets, Crucible `keepsake_<id>` (`ALLOW_DUPLICATE_ACCESSORY = false`; optional `required_stats` hard lock) |
| **Pouch** | `UnitState.pouch` (`POUCH_MAX = 3`) | Tactile Soot-Gothic consumables (`con_heal_potion`/`con_focus_elixir`/`con_fire_bomb` are `ashlands`-only) |
| **Slab Implants** | `4` Act 2+ `body` slots (`PK2`) | `Forearms`, `Thoracic`, `Visceral` (retort-refined ossuary bone-pins) + `Cranial` (class ritual) |

Strict proficiency & burden (`U3_GEAR_REWORK.md` — archived in `docs/archive/sessions/`, `PR5-51a..b`): `Proficiency 0` and unmet `required_stats` **hard-lock equip** (unusable drops feed Hearth salvage); `Prof 1` (`minor`/`sub`) = `0.80x` power/mitigation (no Mastery XP); `Prof 2` (`secondary`/`third`) = `1.00x` + unlocks weapon Mastery XP; `Prof 3` (`main`) = `1.25x` + `1.5x` weapon Mastery XP. Burden lowers **Initiative** (`8 + DEX - burden`) while **Move Rate** (`2..4` tiles/move block) stays unburdened unless Initiative hits floor `1`.

## 2. Weapon & Off-Hand Lineup (`[LOCKED]`, `U3_GEAR_REWORK.md` — archived)

| Class & Founder | `main (3)` / `secondary (2)` / `third (2)` / `minor (1)` Weapons | Off-Hands (`Main [3] / Sub [1]`) | Armor (`Main [3] / Sub [1]`) & Founder Signature Gear |
| --- | --- | --- | --- |
| **Vanguard** (`Kaelen`) | `polearm` *(1H Phalanx Spear/Short-Bill, `1-2`)* / `greatsword` (`2H`) / `sword` (`1H`) / `hammer` (`2H`) | `shield (3)` / `buckler (1)` | `heavy (3)` / `medium (1)` — `notched_billhook` (`1H`) + shield, `castellan_breastplate` (`heavy`) |
| **Slag-Brute** (`Dunstan`) | `gauntlet` *(1H Piston/Shackles)* / `fist` (`1H Light`) / `hammer` (`2H`) / `greatsword` (`2H`) | `braces (3)` / `shield (1)` *(+ `fist`)* | `heavy (3)` / `light (1)` — `penal_shackles` (`gauntlet`), `penal_foundry_apron` (`heavy`) |
| **Arbalestier** (`Hollis`) | `crossbow` (`2H+Rig`, `2-5`) / `bow` (`2H+Rig`, `2-4`) / `knife` (`1H Light`) / `sword` (`1H`) | `ammo (3)` / `cloak (1)` *(+ `knife`)* | `medium (3)` / `heavy (1)` — `winch_arbalest`, `poachers_brigandine` (`medium`) |
| **Gutter-Stalker** (`Vesper`) | `dagger` (`1H Light`) / `knife` (`1H Light`, `2-3` w/ `ammo`) / `bow` (`2H+Rig`) / `fist` (`1H Light`) | `cloak (3)` / `braces (1)` *(+ `1H Light`)* | `medium (3)` / `light (1)` — `curfew_bodkin`, `soot_leathers` (`medium`) |
| **Heretic Acolyte** (`Maren`) | `censer` (`1H`) / `bone_saw` (`1H`) / `scourge` (`1H`, `1-2`) / `staff` (`2H`) | `tome (3)` / `orb (1)` *(`INT` hybrid)* | `light (3)` / `heavy (1)` — `bronze_censer` + `copied_psalter` (`tome`), `embalmers_habit` (`light`) |
| **Ichor Alchemist** (`Elyse`) | `sprayer` *(1H Nozzle, `2-4`)* / `staff` (`2H`) / `bone_saw` (`1H`) / `scourge` (`1H`) | `orb (3)` / `ammo (1)` | `light (3)` / `medium (1)` — `brass_vitriol_sprayer` (`1H`), `tanners_rubbered_coat` (`light`) |

## 3. Tactile Soot-Gothic Pouch Consumables (`POUCH_MAX = 3`, `[LOCKED]`, `PR5-35`)

Generic high-fantasy vials (`con_heal_potion`, `con_focus_elixir`, `con_fire_bomb`) stay `ashlands`-only; `sunk_ward` uses tactile stitch packs, tallow salves, censers, inhalers, and wafers:

| Locked Display Name & ID | Speed | Effect (`sunk_ward` `timeline`) | Availability |
| --- | --- | --- | --- |
| **Tallow-Wax Stitch Pack** (`sulfur_poultice`) | Snap | Heals `60` HP; seals `bleed` & `poison` | Prologue `0c` onward |
| **Brimstone Tallow Salve** (`brimstone_draught`) | Snap | Grants `empower` (`+30%` outgoing skill potency, `2` rounds) | Act 1 onward (`PK3`) |
| **Pitch Censer-Pot** (`pitch_fire_bomb`) | Standard | Thrown (`range 4`, `3x3`): `pitch_fire` burn (`2` rounds) | Act 1 onward |
| **Magnesium Flare-Tube** (`magnesium_flash_pot`) | Snap | Thrown (`range 3`, `3x3`): `20` Poise break | Act 1 onward |
| **Snapped Alum Inhaler** | Snap | Vents `-25` Strain (replaces salt-sachet overlap w/ hub `Rock Salt`) | Act 2 onward |
| **Lead-Sealed Communion Wafer** | Snap | Vents `-40` Strain, then `-2` AP next round | Act 2 onward (`Widow Varka` / Clerisy loot) |
| **Raw Marrow Splinter** | Snap | Surges `+30` Strain on purpose to enter `Aberrant Edge` (`75-99`) | Act 2 onward |
| **Quenched Cinder Salt** (`con_quenched_cinder_salt`) | Snap | Rubbed into calcifying wounds to apply `unmark` & halt Stage V Terminal Drift | Act 2 onward (`PK2`/`PK3`) |

## 4. Ossuary-Relic Growths (`Slab Retorts`) vs. Hearth & Bone-Wright Salvage (`[LOCKED]`, `PR5-35`)

`Chirurgeon Sabina` never stitches wet butcher offal into operatives: **Aberrant Organs** are **half-petrified ossuary growths** refined through `M2`'s confiscated **`Slab Retorts`** into surgical bone-pins and marrow infusions, cleanly separated from mechanical/structural **Hearth & Bone-Wright Salvage**:

| Category | Locked Item Name & Catalog ID | Source | Used At & Purpose |
| --- | --- | --- | --- |
| **Ossuary Growth (`Forearms`)** | **Black-Sinew Braid** | `Calcified Husks`, `Ash-Crawlers` | `Slab Retorts` -> `Forearms` bone-pin (`Tier I/II`) |
| **Ossuary Growth (`Thoracic`)** | **Ossified Bellows** | `Calcified Husks`, `Marrow Horrors` | `Slab Retorts` -> `Thoracic` marrow-rib cage (`Tier I/II`) |
| **Ossuary Growth (`Visceral`)** | **Vitriol Bezoar** (`caustic_gall_sac`) | `Tallow Dredgers`, `Ash-Crawlers` | `Slab Retorts` & `Elyse` -> `Visceral` acid calculus |
| **Ossuary Growth (`Sensory`)** | **Amber Tapetum** (`stalker_optic_nerve`) | `Render-Beasts` (`M4` / Lockdown) | `Slab Retorts` -> `Ocular` / night-sight pin (`Hollis` Act 2 hook) |
| **Ossuary Growth (`Tier III Apex`)** | **Dreamer's Marrow Geode** (`titan_marrow_core`) | `Calcified Amalgam` (`Int.`), `Living Martyrs`, `Horrors` | `Slab Retorts` (post-`2.4`) -> `Tier III Apex` implants (+1 Calcification) |
| **Hardware Salvage** | **Brass Marrow Valve** (`inquisitor_marrow_valve`) | `Proctor Quell` (`M2`), Inquisitor Proctors | `Hearth` / `Slab` pressure-siphon hardware |
| **Structural Salvage** | **Calcified Bone Shard** (`calcified_bone_shard`) | `Calcified Husks`, `M3` bone-mill | `Berek` (`Hearth`) & `Skerrit, Bone-Wright` quarrels/cestuses |
| **Structural Salvage** | **Render-Beast Tendon** (`render_beast_tendon`) | `Render-Beasts` | `Berek` (`Hearth`) heavy arbalest winches (`+1` range / Poise) |
| **Mineral / Purse** | **Rock Salt** (`material`) / **Scrap** & **Warded Tallow** (purse) | Salt cellars, carts, breakdown | `Rock Salt` = Slab `-10` floor flushes & `Corvo` bribes; `Tallow` = `1/day` |

## 5. Unique Campaign Relics & Crucible Keepsakes (`1` Copy per Run, `[LOCKED]`)

| Relic / Key Item & ID | Slot / Type | Campaign Source & Story Origin | Mechanical / Campaign Role |
| --- | --- | --- | --- |
| **Vane's Gilded Castellan Pavise** (`vanes_gilded_pavise`) | `OFF` `shield` | `M1` (`ENC-EVT-01`) when `Vane` withdraws at `25%` HP | High Poise guard; reflects melee damage/Poise on `Active Guard` |
| **Gate Pass Cipher** (`city_gate_pass`) & **Slab Retorts** (`slab_retorts`) | `quest` | `M1` `key_loot` & `M2` `objective_reward` | Opens ward/Corvo routes (`M1`); unlocks `Sabina`'s Slab refinement (`M2`) |
| **Quell's Marrow Scalpel** (`quells_marrow_scalpel`) | `MAIN` `bone_saw` | `M2` (`ENC-EVT-02`) on `Quell`'s death | Snap; bleeds target & vents `10` Strain on hit (Act 2+) |
| **Grendel's Pneumatic Slag-Hammer** (`grendels_slag_hammer`) & **Slag Plating** (`slag_plating`) | `MAIN` `2H hammer` / `BODY` `heavy` | `M3` (`ENC-EVT-03`) on `Grendel`'s death | Heavy `line_2` shockwave maul & kiln-quenched plate |
| **Orlov's Flensing Talon** (`orlovs_flensing_talon`) & **Orlov's Retribution** | `MAIN` `knife` (`1H Light`) + `rogue` skill | `M4` (`ENC-EVT-04`) when `Orlov` withdraws (`PR5-34`) | Snap; executes bleeding targets below `35%` HP in Act 2 |
| **The Dreamer's Crystalline Vertebra** (`dreamers_vertebra`) | `ACC` | `Interlude` (`ENC-EVT-05`) `Calcified Amalgam` | Locks Strain floor `20`; `+30%` Inoculated damage (Act 2+) |
| **Hollis's Lever-Winch** / **Deserter's Kettle Helm** / **Sabina's Prosector Crook** | `ACC` / `HEAD` `heavy` / `MAIN` `staff` | `Hollis` 7th Ward cache / `Kaelen` Act 2 `Vane` ritual / `2.4` `Lazar` raid | `-1` block Heavy Winch Sunder; `Active Guard` ally aura; `+35%` heal & free cleanse |
| **6 Crucible Keepsakes** (`keepsake_<id>`) | `ACC` (`value 0`) | `M5` Crucible winch founder (`CL-03`) | **Kaelen's Gouged Sigil**, **Vesper's Runner Whistle**, **Maren's Copyist Quill**, **Hollis's Focusing Monocle**, **Dunstan's Shackle Link**, **Elyse's Cracked Retort** (`~50%` founder trait -> upgraded to **Transcendent Relic** at `3.4`) |
| **Key of the Mantle Chasm** & **Nemeth's Warded Cassock** | `quest` / `BODY` `light` | `2.6` Reliquary tomb & `3.4` Silent Ossuary (`node 20`) | Opens Act 3 (`2.6`); high-Poise burn/miasma-warded vestments (`3.4`, missed if `node 20` skipped) |
