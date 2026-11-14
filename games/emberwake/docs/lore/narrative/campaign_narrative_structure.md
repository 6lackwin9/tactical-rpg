# Campaign narrative structure: the three acts

Status: **`[LOCKED]` (`2026-11-11`, synced with [CANON.md](../CANON.md) & [STORY_GUIDE.md](../STORY_GUIDE.md); `CL-01..CL-11`)**. Related:
[choices and dilemmas](narrative_choices_and_dilemmas.md), [agency and fail states](narrative_agency_and_fail_states.md),
[NPCs and questlines](../background/npcs_and_questlines.md), [ACTS23 spec](../../../../docs/specs/ACTS23.md).

Layer tags: `[engine]` technology any game needs; `[kit hook]` a shape any story in the genre could fill; `[data]`
content that fills a kit hook; `[game code]` a mechanic only this story uses.

## 1. The arc in brief

- **Act 1 is 100% Cold Steel**: all `6` Founders start with grounded, un-implanted mortal human visuals (`CANON` 3.3). The player learns human limits (weapon weight, armour, AP, poise breaks) before any eldritch power, so Inoculation in Act 2 arrives as a hard cost the cell accepts after `Inquisitor-Prelate Nemeth` shrugs off Cold Steel at the Crucible.
- **Authored ensemble + resilient inheritance**: Act 1 holds only the `6` named Founders (`4` Prologue + `Dunstan` + `Elyse`); when `M5` or Act 2 claims a Founder, **Survivor Inheritance (`CL-09`)** ensures `Berek`, `Sabina`, `Talia`, and surviving cellmates finish their unfinished Act 2 reckonings rather than stranding story threads.
- **Four Hub Table Debates (`CL-11`)**: `Post-0c`, `Post-Interlude`, `Post-M5`, and `Post-2.4` gather the Hub mentors (`Berek`, `Chirurgeon Sabina`, `Talia Rusk`) and active Founders around the Sluice brazier so philosophical shifts (`Preservationist` vs. `Flesh-Broker`) grow out of lived losses and discoveries.

## 2. The acts at a glance

| Aspect | Act 1 *The Young Vanguard* | Act 2 *The First Rupture* | Act 3 *The Awakening Abyss* |
| --- | --- | --- | --- |
| Region | The Sunk Ward (`00..07`) | The High Bastion (`08..15`) | The Mantle Fissure (`16..22`) |
| Status | **`[LOCKED]`** | **`[LOCKED]`** | **`[LOCKED]`** |
| Length target | ~4-5 h | ~7-9 h | ~5-6 h |
| Battles | `9` main (`0a-0c`, 2-tier `M1-M4`, causal `Int.`, `M5`) + `6` Founder hooks & ward jobs | `6` main (`2.1-2.6`) + `6` dual-rituals + culling wagons & sieges | `4` mandatory (`3.1`, `3.2`, `3.5->3.6`) + `2` optional pre-`3.5` (`3.3`, `3.4`) |
| Rules on | Cold Steel; downed Founders end battles `wounded` (`-20%` HP until rested) | `Channel`, `Strain`, `Calcification`, `4` Slab implants, durability/techniques, cohorts, permadeath/capture, sieges (`Alert >= 30`) | Hard Doom Clock (`min 12d`), Mantle Resonance (`+15` Strain), `3.5->3.6` point-of-no-return |
| Roster | `6` Founders (`Kaelen`, `Vesper`, `Maren`, `Hollis`, `Dunstan`, `Elyse`) | `5` Founders + `18-24` visored/cowled Unsealed cohorts (`6 -> 8` bunks) | Veterans across Calcification stages + `2` Crypt stasis slots |

## 3. Prologue (`0a–0c` -> Hub Debate 1)

Three scripted missions outside the district graph (`enc_pro_0a..0c`), detailed in [onboarding](new_game_onboarding_and_intro.md):

| # | Mission | Who joins & Reactive Beat |
| --- | --- | --- |
| `0a` | The Alley (*Alleyway of Cinders*, Seventh Ward) | `Kaelen`, solo (`Order 44`; clean bell-tremor & Watch whistles, no Day-1 forearm burn) |
| `0b` | The Rooftop (*Meat Market* rooftops) | `Vesper` joins (`talk` smoke callout vs. `3` `Mortuary Flagellants`) |
| `0c` | The Bridge (canal bridge, iron mortuary cart) | `Maren` & `Hollis` freed (`talk` bridge callout vs. `3` Footmen + `1` Crossbowman) |

Diving through `01 Drowned Runoff Grate` into `00 Sluice Undercroft` triggers **Hub Debate 1 (*What Keeps Us Alive*)**: **`Berek "Cinder-Lung" Dorn`** (`Hearth`), **`Chirurgeon Sabina`** (`Slab`), and **`Talia Rusk`** (`Basin`) state their competing survival creeds (`4/6` bunks filled, `42d` soft clock).

## 4. Act 1 *"The Young Vanguard"* (`CL-08`, `CL-11`)

- **Cold Steel only**: `0` implants, `Stage 0` clean human portraits/sprites (`CANON` 3.3); downed Founders become `wounded` (`-20%` max HP until rested).
- **2-Tier Sunk Ward Diamond (`CL-08`)**: **Tier 1** opens `M1` and `M2` in either order; completing both unlocks **Tier 2** (`M3` recruiting `Dunstan`, `M4` recruiting `Elyse` in either order) so whichever recruit is rescued first can deploy in the sibling raid and run their side-hook before `Interlude` and `M5`.
- **Soft Doom Clock (`42d`, `CL-04`)**: hitting `0d` triggers a **lesser tithing** (`14d` reset, `+10` Alert, claiming `1` unrescued stake `Ivo` or `Ida`, else `-2` `saved_youths`). `Alert 100` in Act 1 ends on `Extinguished`.

### 4.1 Main missions

| # | Mission | Node | Encounter & Outcome | Joins / loses |
| --- | --- | --- | --- | --- |
| `M1` (*Tier 1*) | The Sweep | `02` Alleyway Junction 7 | `enc_evt_01`, `Capt. Vane` withdraws at `25%` HP (`Pavise`, `Gate Pass`) | - |
| `M2` (*Tier 1*) | The Crossroads Cages | `03` Ash-Crawl Crossroads | `enc_evt_02`, `Proctor Quell` (`35%` HP valve surge, dies; `Scalpel`, `Valve`, `Slab Retorts`; saved captives -> `+1` `saved_youths` + `+1` `Rock Salt`) | - |
| `M3` (*Tier 2*) | The Bone-Mill Raid | `04` Fourth Kiln | `enc_evt_03`, `Overseer Grendel` dies (`Slag-Hammer`, `Slag Plating`) | `Dunstan` joins (bunk 5/6) |
| `M4` (*Tier 2*) | The Tallow Vats | `05` Tallow Vats | `enc_evt_04`, `Guild-Master Orlov` withdraws at `20%` HP (`Flensing Talon`, `Retribution`, `+5` `Warded Tallow`) | `Elyse` joins (bunk 5/6) |
| `Int.` | The Awakening Fissure | `00` Sluice Undercroft | `enc_evt_05`, **causal marrow back-surge** from shutting `M1–M4` (`2x2` `Amalgam`; `Sabina` dose `sunk_ward.sabina_dose` -> **Hub Debate 2**) | - |
| `M5` | The Crucible | `06` Watchpost -> `07` Slag Gate | `enc_evt_06`, `Inquisitor-Prelate Nemeth` (`unkillable`; `Hollis` hook delays R3 wave -> R4) | `1` of `4` deployed captured (`keepsake_<id>` -> **Hub Debate 3**) |

### 4.2 Six Founder Tactical Side-Hooks (`CL-08`)

| Founder | Side-Hook & Node | Tactical & Narrative Reward |
| --- | --- | --- |
| `Kaelen` | *"The Seventh Ward Bellows"* (`02` Watch cache) | Cuts `Berek`'s early Hearth Scrap costs + `1` `Slag Plating` |
| `Vesper` | *"The Sluice Lock-Ciphers"* (`01` flood-locks) | Opens permanent `0`-Alert **Siphon Shortcut** (`1d`, `0` Alert) |
| `Maren` | `M2` + *"Liturgical Cipher Rubbings"* (`03` post-`M2`) | `+1` `saved_youths` & `+1` `Rock Salt` per saved `M2` captive + St. Lyra's vault clue |
| `Hollis` | *"The Crag-Sight Overlook"* (`06` outer chimneys) | **`Hollis's Lever-Winch`** + delays `M5` `gate_reinforcements` from **Round `3` to Round `4`** |
| `Dunstan` | *"Chain-Gang Wedges"* (`04` post-`M3`) | `+1` `Rock Salt` + opens **`Skerrit, Bone-Wright`** barbed bone cestus/quarrel recipes |
| `Elyse` | *"The Tenement Counter-Still"* (`05` post-`M4`) | Checks awakened Spent mother's block -> `+5` `Warded Tallow` + `1` prototype **`Snapped Alum Inhaler`** |

Other Act 1 board work: `Talia`'s *"Night of the Red Pen"* (rescues `12`-yo `Ivo` -> opens **`Widow Varka`**'s Basin `Trade` shop), `Bailiff Corvo`'s Third Ward Gate (`Ida` `16`), `Agnes of Block 12`, and `Brother Rook`.

## 5. Act 2 *"The First Rupture"* (`CL-02`, `CL-05..11`)

- **Post-`M5` Wake (`Hub Debate 3`) & Cohort Arrival (`CL-07`)**: The `5` surviving Founders mourn over the empty bunk (`keepsake_<id>` in stash) before `Talia` brings in `1` class-matched `18-24` visored/cowled Unsealed runaway (`6/6` bunks -> `8` via `Berek`'s *"Expanded Timber Frames"*).
- **Inoculation vs. Cold Steel**: `Sabina` refines ossuary growths in `M2`'s `Slab Retorts` (`4` `body` implant slots + `Channel`/`Strain`/`Calcification`), while `Berek` unlocks Cold Steel Mastery `1-3` techniques and Act 2 weapon durability.
- **Main Spine (`2.1..2.6`)**:
  - `2.1` Gilded Boulevard (`09`), `2.2` Culling Stockades (`12`, repeatable `18-24` recruits + `14`-yo `Lucan`), `2.3` Tallow Reservoir (`10`, `+20` Tallow / `-30` Alert).
  - **`2.4` Embalming Cloisters (`11`)**: `Chief Anatomist Lazar` slain (`Sabina's Prosector Crook`, Tier III Apex implants & bone-scraping); `Nemeth` delivers `M5` captive behind leaded glass (`no rescue`); **Lullaby House** ledger triggers **Hub Debate 4** and relabels the clock *the Great Tithing* (`max(days, 18)`).
  - **`2.5` Grand Belltower (`13`, `CL-10`)**: Stage IV **Penitent Bell-Warden** wired into St. Lyra's bell-yoke (silences curfew-bell `+10` Strain resonance).
  - **`2.6` Reliquary Crypts (`14 -> 15`, `CL-10`)**: **`Inquisitor-General Ruthven` ("the Blind Eye")** (`25%` HP retreat in any earlier Breach siege) + **First Apostle Saint-Warden** -> First Extinguishers' confession + **Key of the Mantle Chasm**.
- **Six Dual-Rituals + Survivor Inheritance (`CL-02`, `CL-09`)**: Every Founder's personal quest (`Vane`, `Orlov`, St. Lyra's vault, `Wren`, `Jessop` + `3.2` Conduit blueprints, sedative cisterns) is inherited by `Berek`/`Talia`/`Sabina` + surviving cellmates if `roster.departed.<id>` is true.

## 6. Act 3 *"The Awakening Abyss"* (`CL-04..11`)

- **Hard Doom Clock (`min 12d`, `CL-04`) & Mantle Resonance (`+15` Strain, `CL-05`)**: Hitting `0d` before `3.5` ends on `Extinguished`.
- **Main & Pre-`3.5` Optional Nodes**:
  - `3.1` **Conduit Survey Outpost** (`17`, `CL-11`): Overrun Clerisy gauge-station introducing **`Brother Malachi`**.
  - **`3.2` Marrow Conduits (`18`, `CL-10`)**: **`Warden-Inquisitor Ulric`** slain; severs both the steeped-ichor conduit (`21 -> 11`) and raw-condensate bleed-line (`+2d` clock).
  - `3.3` **Chorus Spire** (`19`, optional pre-`3.5`): **`Cantor Ysolde`** slain or mediated via `Malachi` on Flesh-Broker runs (halves Mantle Resonance).
  - **`3.4` Silent Ossuary (`20`, optional pre-`3.5`)**: Corrupted `M5` captive snaps `hostile_all` (`PHASEFX1`), kills `Nemeth`, and faces the squad as a `2x2` aberration (`Nemeth's Warded Cassock` + `Transcendent Relic`).
  - **`3.5` High Altar (`21`) -> `3.6` Dreaming Core (`22`, `CL-11`)**: Warned **point-of-no-return**; `Arch-Prelate Othemar` falls, halting the Tithing triggers the waking caldera collapse, and a **one-time camp prep/rest screen on the cracked Altar lip** leads directly into the `2x2` `Dreaming Core` (`Endings A / B / C`: `Black-Iron Severance`, `The Living Conduit`, `Stone Martyrdom`).

## 7. Scope targets (`[LOCKED]`)

| Item | Target |
| --- | --- |
| Playtime | 16-20 h |
| Battles | 32-40 |
| Bunks | `6` (`8` through `Berek`'s Act 2 Hearth project + `2` Crypt stasis slots) |
| Replay | Different `M5` captive & Survivor Inheritance arcs, `2`-tier Act 1 order, `Preservationist` vs. `Flesh-Broker` Endings `A/B/C` |
