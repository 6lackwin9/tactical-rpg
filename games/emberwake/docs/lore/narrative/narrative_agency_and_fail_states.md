# Narrative agency, roster continuity and fail states

Status: **`[LOCKED]` (`2026-11-11`, synced with [CANON.md](../CANON.md) & [STORY_GUIDE.md](../STORY_GUIDE.md); `CL-01..CL-11`)**. Related:
[NPCs and questlines](../background/npcs_and_questlines.md), [ACTS23 spec](../../../../docs/specs/ACTS23.md).
Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Authored Founders, Visored Cohorts & Survivor Inheritance (`CL-07..11`)

The cast is a resilient ensemble: the `6` Founders have personal arcs across Acts 1-3, visored/cowled Unsealed cohorts (`18-24`) replenish lost bunks from Act 2 onward, and **Survivor Inheritance (`CL-09`)** guarantees that losing a Founder at `M5` or in Act 2 reshapes *how* their personal reckoning plays out rather than deleting it.

### 1.1 The Six Founders (`CANON` 3.1)

| Founder | World Tie | Act 1 Hook (`CL-08`) | Act 2 Reckoning & Survivor Inheritance (`CL-02`, `CL-09`) |
| --- | --- | --- | --- |
| `Kaelen` (`Vanguard`, `24`) | Seventh Ward Watch deserter who refused `Vane`'s Order 44 | `M1` (`Vane` withdraws) + *"Seventh Ward Bellows"* (`Slag Plating` + Hearth discount) | Hunts `Capt. Vane` & Watch debt ledgers *(inherited by `Berek` + cell)* |
| `Vesper` (`Gutter-Stalker`, `21`) | Ash-Crawl curfew runner whose brother died on `Orlov`'s tallow tithe | `M4` (`Orlov` withdraws) + *"Sluice Lock-Ciphers"* (`0`-Alert Siphon Shortcut) | Raids `Orlov`'s Counting-House *(inherited by `Talia` + cell)* |
| `Maren` (`Heretic Acolyte`, `22`) | Defrocked St. Lyra's copyist whose mentor `Quell` vivisected | `M2` (`Quell` slain, saved captives -> `+1` `Rock Salt`) + *"Liturgical Cipher Rubbings"* | Unseals St. Lyra's Forbidden Vault *(inherited by `Sabina` + cell)* |
| `Hollis` (`Arbalestier`, `23`) | Crag poacher whose bolt blinded `Ruthven` while `Wren` (`14`) was taken | *"The Crag-Sight Overlook"* (`Lever-Winch` + delays `M5` R3 wave -> R4) | Frees `Wren` from Bastion convoy *(inherited by `Talia` + cell)* & faces `Ruthven` at `2.6` |
| `Dunstan "Slag-Jaw"` (`Slag-Brute`, `22`) | Fourth Kiln bone-mill convict whose Spent father came down `Grendel`'s chute | Joins at `M3` (`Grendel` slain) + *"Chain-Gang Wedges"* (`Skerrit` bone-wright recipes) | Kills `Foundry-Master Jessop` & seizes `3.2` Conduit plans w/ `Berek` *(inherited by `Berek` + cell)* |
| `Elyse` (`Ichor Alchemist`, `19`) | Vat assayer whose alum counter-tonic woke her Spent mother's tenement block | Joins at `M4` + *"Tenement Counter-Still"* (`+5` `Warded Tallow` + `1` prototype `Snapped Alum Inhaler`) | Sabotages Cathedral sedative cisterns *(inherited by `Sabina` + cell)* |

- **Hub Debates & Battle `talk` Cues (`CL-11`)**: Founders speak in the **4 Hub Table Debates** (`Post-0c`, `Post-Interlude`, `Post-M5`, `Post-2.4`) alongside `Chirurgeon Sabina`, `Berek "Cinder-Lung" Dorn`, and `Talia Rusk`, and trigger `1-2` line mid-battle `talk` (`TalkCue`) phase overlays when deployed against their personal antagonists.

### 1.2 Act 2-3 Unsealed Recruit Cohorts (`18-24`, `CL-07`)

- Founders cannot die in Act 1 (`0` HP -> `wounded`, `-20%` max HP until rested).
- At Act 2 start, `Talia` shelters `1` class-matched `18-24` runaway into the `M5` bunk (`CL-07`); repeatable `2.2 Culling Stockades` wagon raids (`enc_act2_02`) offer `1-2` recruits from the `6` **Act 2 Recruit Cohorts** (*Turncoat Watch Levy*, *Ash-Crawl Flue-Runner*, *Apostate Charnel-Novice*, *Outwall Bolt-Man*, *Kiln Chain-Breaker*, *Vat-Still Saboteur*), all wearing **visors, cowls, scarves, or respirator-goggles** (`CANON` 3.2).
- Rescued children under `18` (`Ivo` `12`, `Lucan` `14`, `Wren` `14`, `Ida` `16`) and Spent specialists `25+` (`Ferrand` `40`, `Herbalist Clemence` `36`) stay bunk-free non-combat Basin wards/guests (`CL-07`).

## 2. Reserves, Bunks & Upkeep

- **No soft lock**: `Talia`'s cipher board always offers `2.2` culling-wagon raids in Acts 2-3 while at least `1` operative lives.
- **Bunk cap**: `6` in the Dry Hollows (`8` after `Berek`'s *"Expanded Timber Frames"* Hearth project, plus `2` **Isolation Crypt** stasis slots for Stage IV operatives).
- **Cell upkeep**: `1` `Warded Tallow` per day of travel or rest (`0` = `-2` starting AP in battle; never scales with bench count).

## 3. Lifecycle, Sieges & Fail States (`[LOCKED]`)

### 3.1 Operative Downed Rules (`ACTS23` `PK3`)

- **Act 1**: Downed -> `wounded` (`-20%` max HP until rested); `M5` Crucible winch is the sole scripted capture (`sunk_ward.crucible_stayed`).
- **Acts 2-3**: On a **won** battle (`held field`), downed operatives survive `wounded` unless claimed by Stage V **Terminal Drift** (`lost`, `Sabina` recovers `1` core from implanted allies); on a **fled/lost** battle, downed operatives face seeded `captured` / `dead` (`ACTS23` `PK3`), returning gear to stash and inscribing their name on the Dry Hollows Memorial Stone.

### 3.2 The `Extinguished` Failure Screen

`Extinguished` is a campaign failure screen (`aegis_ui`), never a narrative ending:

| Condition | Trigger |
| --- | --- |
| **Act 1 Breach** | `Alert` hits `100` in Act 1 (`CANON` 4.2) |
| **Total Cell Wipe** | `0` living operatives remain across deployed squad and hub bunks |
| **Act 3 Hard Doom Clock** | *The Great Tithing* reaches `0d` before `3.5 High Altar` (`node 21`) falls (`3.6` dream-retries also tick Act 3 days, `CL-11`) |

### 3.3 Sluice Hub Breach Sieges (`Acts 2-3`, `CL-06`, `CL-10`)

- Hitting `Alert 100` in Acts 2-3 triggers a **Sluice Undercroft Siege** (`Chirurgeon Sabina` and `Berek` stand on `protect` tiles; `Inquisitor-General Ruthven` leads Act 2 Breaches and withdraws at `25%` HP so he always commands `2.6`).
- Losing a siege never ends the run: the cell holds the single Sluice hub (`Q-90 a`), paying `10` `Warded Tallow`, `50` `Scrap`, `+2d` facility lock, and resetting `Alert` to `60`.

### 3.4 Lesser Tithing vs. Great Tithing (`CL-04`)

- **Acts 1-2 Soft Clock (`42d` -> `max(days, 18)` at `2.4`)**: Hitting `0d` triggers a **lesser tithing** (`14d` reset, `+10` Alert, claiming `1` unrescued named stake—`Ivo` `12` or `Ida` `16`—else `-2` `saved_youths`).
- **Act 3 Hard Clock (`min 12d`)**: Stopped permanently when `Arch-Prelate Othemar` falls at `3.5 High Altar`, triggering the waking caldera collapse into `3.6 Dreaming Core` (`CL-11`).
