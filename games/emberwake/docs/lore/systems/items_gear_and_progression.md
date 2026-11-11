# Items, gear, ossuary growths and progression

Status: **`[LOCKED]` (`2026-11-10` Lore & Visual Hooks Session)** — equipment slots, `POUCH_MAX = 3`, weapon families (`WEAPONROSTER1`), **Ossuary-Relic Growths (refined via `M2` `Slab Retorts`) vs. Hearth Salvage**, **Tactile Soot-Gothic Pouch Consumables**, and all **Named Campaign Relics & Crucible Keepsakes**, matching [CANON.md](../CANON.md) Section 5 (`PR5-35` syncs shipped `items.json` via `aegis content`).

Engine facts (`ITEM1`, `C1-L`, `ITEMUSE1`, `REWARD1`, `WEAPONROSTER1`): items live in `games/emberwake/data/catalog/items.json` on `item_families.json` (`rpgkit_data::ItemDef`); world named items resolve to a family; content is edited only through `aegis content`.

## 1. Equipment slots, Slab implant slots & battle pouch (`[LOCKED]`)

| Slot | Engine name | Holds |
| --- | --- | --- |
| `MAIN` | `MAIN_HAND` | Primary weapon (`1H` or `2H`); sets active weapon family for `[Signature]` skills |
| `OFF` | `OFF_HAND` | Shield, off-hand weapon, orb or tome (`empty` when `MAIN` is `2H`) |
| `HEAD` | `HEAD` | Kettle helm, cowl, hood, iron mask |
| `BODY` | `BODY` | Plate, brigandine/leather, or mortuary/vat cloth |
| `ACC1`, `ACC2` | `ACCESSORY_1`, `ACCESSORY_2` | Relics, winch ratchets, Crucible `keepsake_<id>` (`ALLOW_DUPLICATE_ACCESSORY = false`) |
| **Pouch** | `UnitState.pouch` (`POUCH_MAX = 3`) | Tactile Soot-Gothic consumables (`con_heal_potion`/`con_focus_elixir`/`con_fire_bomb` are `ashlands`-only) |
| **Slab Implants** | `4` Act 2+ `body` slots (`PK2`) | `Forearms`, `Thoracic`, `Visceral` (retort-refined ossuary bone-pins) + `Cranial` (class ritual) |

Soft wield & burden (`rpgkit_rules::gear`): untrained weapon families stay equippable at `0.75x` physical weapon potency and `+1` burden; class `proficiency` (`3/2/2`) waives the penalty and unlocks `[Signature]` skills. Burden lowers **Initiative** (`8 + DEX - burden`) while **Move Rate** (`2..4` tiles/move block) stays unburdened.

## 2. Weapon families & Founder signatures (`[LOCKED]`, `WEAPONROSTER1`)

| Soot-Gothic Family | Engine Family | Hands | Reach | Speed Tier | Scales | Primary Class & Founder Signature (`CAST1_GEAR` 2) |
| --- | --- | --- | --- | --- | --- | --- |
| Billhooks, poleaxes | `polearm` | `2H` | `1-2` | Standard / Heavy | `STR` | Vanguard (`Kaelen`: `notched_billhook`) |
| Broadswords, greatswords | `sword`, `greatsword` | `1H`/`2H` | `1` | Standard / Heavy | `STR` | Vanguard, Gutter-Stalker, Arbalestier |
| Bodkins, flensing knives | `dagger` | `1H` | `1` | Snap | `DEX` | Gutter-Stalker (`Vesper`: `curfew_bodkin`) |
| Heavy lever-arbalests | `crossbow` | `2H` | `2-5` | Standard / Heavy | `DEX`/`PERC` | Arbalestier (`Hollis`: `winch_arbalest`) |
| Riveted shackles, cestuses | `fist` | `1H`/pair | `1` | Snap | `STR`/`CONS` | Slag-Brute (`Dunstan`: `penal_shackles`) |
| Foundry sledges, piston-mauls | `hammer` | `2H` | `1` | Heavy | `STR` | Slag-Brute, Vanguard |
| Censers, bone-saws, scourges | `censer`, `bone_saw`, `scourge` | `1H` | `1` (`scourge` `1-2`) | Snap / Standard | `KNOW`/`INT`/`STR` | Heretic Acolyte (`Maren`: `bronze_censer` + `copied_psalter`) |
| Pressurized vitriol sprayers | `sprayer` | `2H` | `2-4` | Standard | `INT`/`PERC` | Ichor Alchemist (`Elyse`: `brass_vitriol_sprayer`) |

**Founder Signature Armour (`CAST1_GEAR` 2):** `castellan_breastplate` (`Kaelen`), `soot_leathers` (`Vesper`), `embalmers_habit` (`Maren`), `poachers_brigandine` (`Hollis`), `penal_foundry_apron` (`Dunstan`), `tanners_rubbered_coat` (`Elyse`).

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
| **Grendel's Pneumatic Slag-Hammer** (`grendels_slag_hammer`) & **Slag Plating** (`slag_plating`) | `MAIN` `2H hammer` / `BODY` `plate` | `M3` (`ENC-EVT-03`) on `Grendel`'s death | Heavy `line_2` shockwave maul & kiln-quenched plate |
| **Orlov's Flensing Talon** (`orlovs_flensing_talon`) & **Orlov's Retribution** | `MAIN` `dagger` + `rogue` skill | `M4` (`ENC-EVT-04`) when `Orlov` withdraws (`PR5-34`) | Snap; executes bleeding targets below `35%` HP in Act 2 |
| **The Dreamer's Crystalline Vertebra** (`dreamers_vertebra`) | `ACC` | `Interlude` (`ENC-EVT-05`) `Calcified Amalgam` | Locks Strain floor `20`; `+30%` Inoculated damage (Act 2+) |
| **Hollis's Lever-Winch** / **Deserter's Kettle Helm** / **Sabina's Prosector Crook** | `ACC` / `HEAD` `plate` / `MAIN` `staff` | `Hollis` 7th Ward cache / `Kaelen` Act 2 `Vane` ritual / `2.4` `Lazar` raid | `-1` block Heavy Winch Sunder; `Active Guard` ally aura; `+35%` heal & free cleanse |
| **6 Crucible Keepsakes** (`keepsake_<id>`) | `ACC` (`value 0`) | `M5` Crucible winch founder (`CL-03`) | **Kaelen's Gouged Sigil**, **Vesper's Runner Whistle**, **Maren's Copyist Quill**, **Hollis's Focusing Monocle**, **Dunstan's Shackle Link**, **Elyse's Cracked Retort** (`~50%` founder trait -> upgraded to **Transcendent Relic** at `3.4`) |
| **Key of the Mantle Chasm** & **Nemeth's Warded Cassock** | `quest` / `BODY` `cloth` | `2.6` Reliquary tomb & `3.4` Silent Ossuary (`node 20`) | Opens Act 3 (`2.6`); high-Poise burn/miasma-warded vestments (`3.4`, missed if `node 20` skipped) |
