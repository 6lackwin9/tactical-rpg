# The Sunk Ward: campaign graph and Act 1 missions

Status: canon (`2026-11-11`, synced with [CANON.md](../CANON.md) & [STORY_GUIDE.md](../STORY_GUIDE.md); Act 1 *"The Young Vanguard"*, `CL-01..CL-11`). World id
`sunk_ward`, the first Low Sleep world in `games/emberwake`. Related: [world geography](world_geography_and_hub.md),
[NPCs and questlines](npcs_and_questlines.md), [CAST1 encounters](../../specs/CAST1_ENCOUNTERS.md),
[onboarding and prologue](../narrative/new_game_onboarding_and_intro.md),
[campaign structure](../narrative/campaign_narrative_structure.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`,
`[game code]`.

## 1. Overview

Act 1 of *Emberwake: A Scarlet Lullaby* takes place in **the Sunk Ward**, the lowest industrial quarter of Cinderreach.
The district map is an 8-node **2-tier diamond** of street crossings, drain conduits, industrial kilns/vats, and guarded church posts
`[kit hook]` world map (`worldmap.rs` + `travel.rs`), `[data]` `locations.json` and `routes.json`. Travel is
non-linear within each tier: **Tier 1** opens `02 Alleyway Junction 7` (`M1`) and `03 Ash-Crawl Crossroads` (`M2`) in either order;
completing both opens **Tier 2**—`04 Fourth Kiln` (`M3`, recruiting `Dunstan`) and `05 The Tallow Vats` (`M4`, recruiting `Elyse`)
in either order—so whichever recruit is freed first can deploy in the sibling raid and run their personal side-hook before shutting down
all four surface valves forces the **causal `Interlude` marrow back-surge** (`00`) and **`M5` Crucible** (`06 -> 07`).

```text
                              [ THE SUNK WARD NODE GRAPH ]
                                           |
                                [ 07. THE SLAG GATE ]
                               (exit to Upper Cinderreach)
                                           ^
                                           | [route: High Gorge]
                                [ 06. CRUCIBLE WATCHPOST ]
                             (M5, ENC-EVT-06: Nemeth, the winch)
                                           ^
                    +----------------------+----------------------+
                    | [route: Flue Climb]                         | [route: Convoy Track]
         [ 04. FOURTH KILN ]                           [ 05. THE TALLOW VATS ]
      (M3, Tier 2: Grendel; Dunstan)                (M4, Tier 2: Orlov; Elyse)
                    ^                                             ^
                    | [route: Pipe Duct]                          | [route: Slum Alleys]
         [ 02. ALLEYWAY JUNCTION 7 ]                   [ 03. ASH-CRAWL CROSSROADS ]
       (M1, Tier 1: Captain Vane)                    (M2, Tier 1: Proctor Quell)
                    ^                                             ^
                    | [route: Water Siphon]                       | [route: Drain Flue]
                    +----------------------+----------------------+
                                           |
                              [ 01. DROWNED RUNOFF GRATE ]
                                 (egress / checkpoint)
                                           ^
                                           | [submerged sluice gate]
                              [ 00. SLUICE UNDERCROFT ]
                     (the hub; causal interlude ENC-EVT-05: the Awakening Fissure)
```

The prologue maps (`0a–0c`) are scripted and sit outside this graph; Vesper's Act 1 side-hook (*"The Sluice Lock-Ciphers"*) opens a permanent `0`-Alert **Siphon Shortcut** (`1d`, `0` Alert) bypassing lit surface legs.

## 2. Travel and the Inquisition Alert

### 2.1 Route travel

- **Cost:** a route leg costs 1 day on the travel clock `[kit hook]` (`travel.rs`), and 1
  Warded Tallow per day `[kit hook]` resource purse (`CANON` 4.2).
- **Road encounters:** each route has a road-encounter chance (`15%` standard `encounter_pct`), raised to `150%` under Lockdown (`Alert 71-99`); at `0` Warded Tallow, every battle starts at `-2` AP.
- **Stealth or forced march** (chosen per leg) `[kit hook]`, `[data]`:
  - stealth (default): under fog and chimney soot; lower encounter risk; `+1` day;
  - forced march: no extra day, `+5%` Alert per leg and a guaranteed roamer encounter while Alert is `50%` or more.

### 2.2 Inquisition Alert and searchlight sweeps

The Inquisition Alert (`0-100%`) is the regime's search intensity (`CANON` 4.2):

- **Searchlight sweeps:** watchtower cones sweep exposed routes on a timer. Entering a lit node starts an ambush battle and adds `+10%` Alert.
- **Bands:**
  - `0-30%` Guarded: ordinary watch patrols; routes mostly open;
  - `31-70%` Suspicious: checkpoint gates locked; Castellan Footmen hold bridge nodes; Scrap/Rock Salt tolls to pass quietly;
  - `71-99%` Lockdown: `Inquisitor-General Ruthven`'s `Render-Beast` packs roam main roads (`150%` road fights); searchlights sweep every 2 days;
  - `100%` Breach: ends the run in Act 1 on the **Extinguished** failure screen (`CANON` 4.2).
- **Soft Doom Clock (`42d` -> Lesser Tithing, `CL-04`)**: hitting `0d` in Act 1 does *not* end the run; a lesser tithing sweeps the ward (`14d` reset, `+10` Alert, claiming `1` unrescued personal stake—`Talia`'s courier `Ivo` (`12`) or `Bailiff Corvo`'s sister `Ida` (`16`)—or `-2` `saved_youths` if both were already secured).

## 3. Act 1 missions (`CL-08`, `CL-11`)

Missions are coordinated from **Talia Rusk's cipher board** at the Water-Lock Basin alongside **Berek "Cinder-Lung" Dorn** (`Hearth`) and **Chirurgeon Sabina** (`Slab`). Squad: up to `4` deployed. Founder at `0` HP: downed -> `wounded` (`-20%` max HP until rested; no death or capture in Act 1 except `M5` Crucible). Act 1 is **100% Cold Steel**—all `6` Founders have grounded, un-implanted mortal human visuals (`CANON` 3.3).

| Stage | Mission | Node | Encounter | Joins / loses | Key Narrative & Tactical Payoff |
| --- | --- | --- | --- | --- | --- |
| `0a` | The Alley | prologue map | `enc_pro_0a` | `kaelen` (solo) | Clean curfew bell-tremor (no Day-1 forearm burn); `Pommel Strike` interrupt |
| `0b` | The Rooftop | prologue map | `enc_pro_0b` | `vesper` | `talk` smoke cue; `Ash-Cloud Fade`, `Intercepting Hook` |
| `0c` | The Bridge -> Hearth | prologue map -> `00` | `enc_pro_0c` | `maren`, `hollis` | Frees mortuary cart -> **Hub Debate 1** (`Sabina` vs. `Berek` vs. `Talia`) |
| `M1` (*Tier 1*) | The Sweep | `02` Alleyway Junction 7 | `enc_evt_01`, `Capt. Vane` | - | `Vane` withdraws at `25%` HP (`CL-01`) -> `Vane's Gilded Pavise`, `Gate Pass Cipher` |
| `M2` (*Tier 1*) | The Crossroads Cages | `03` Ash-Crawl Crossroads | `enc_evt_02`, `Proctor Quell` | - | `Quell` `35%` HP valve surge & dies; each saved `tithe_captive` -> `+1` `saved_youths` + `+1` `Rock Salt`; `Quell's Marrow Scalpel`, `Slab Retorts` |
| `M3` (*Tier 2*) | The Bone-Mill Raid | `04` Fourth Kiln | `enc_evt_03`, `Overseer Grendel` | `dunstan` (bunk 5/6) | `Grendel` dies; `Grendel's Slag-Hammer`, `Slag Plating`, opens `Dunstan` hook |
| `M4` (*Tier 2*) | The Tallow Vats | `05` Tallow Vats | `enc_evt_04`, `Guild-Master Orlov` | `elyse` (bunk 5/6) | `Orlov` withdraws at `20%` HP (`CL-01`) -> `Orlov's Flensing Talon`, `Orlov's Retribution`, `+5` `Warded Tallow`, opens `Elyse` hook |
| `Int.` | The Awakening Fissure | `00` Sluice Undercroft | `enc_evt_05`, `Calcified Amalgam` (`2x2`) | - | **Causal marrow back-surge** from shutting `M1–M4`; `Sabina` marrow dose (`sunk_ward.sabina_dose`) -> **Hub Debate 2**; `Dreamer's Vertebra`, `Dreamer's Marrow Geode` |
| `M5` | The Crucible | `06` Watchpost -> `07` Slag Gate | `enc_evt_06`, `Inquisitor-Prelate Nemeth` | Loses `1` deployed founder (`capture`) | `Hollis` overlook delays R3 wave -> R4; `1` of `4` holds winch -> `keepsake_<id>` -> **Hub Debate 3** wake |

### 3.1 Prologue `0a–0c` & Hub Debate 1

Full staging: [onboarding and intro](../narrative/new_game_onboarding_and_intro.md).

- **`0a` The Alley** (Alleyway of Cinders, Seventh Ward): `Kaelen`, who knocked `Capt. Vane`'s iron cudgel aside rather than break an Unsealed girl's arm in a Tithing sweep, fights `2` `footman_patrol` alone after `Vane` barks `"Order 44"` and withdraws. Kaelen hides only a faded Lullaby wrist-number under bloodied linen (`CL-08`).
- **`0b` The Rooftop** (Meat Market rooftops): `Vesper` drops in with a mid-battle `talk` cue against `3` `mortuary_flagellant` pursuers.
- **`0c` The Bridge** (canal bridge): `Kaelen` and `Vesper` smash an iron mortuary cart, freeing `Maren` and `Hollis` against `3` `footman_patrol` and `1` `watch_crossbowman`.
- **Sluice Hearth (`Hub Debate 1 — What Keeps Us Alive`)**: Diving through `01 Drowned Runoff Grate` into `00 Sluice Undercroft`, the four Founders meet `Berek`, `Chirurgeon Sabina`, and `Talia Rusk` (`4/6` bunks filled, `42d` soft clock). Over the brazier, `Berek` warns what Fourth-Kiln valves pump beneath the city, `Sabina` warns Cold Steel alone cannot kill an elevated Prelate, and `Talia` lays down the Tier 1 cipher board.

### 3.2 Tier 1: `M1` The Sweep & `M2` The Crossroads Cages (Either Order)

- **`M1` The Sweep (`02 Alleyway Junction 7`)**: `Capt. Vane` (`38`, selling captives for a Bastion commission before he turns Spent at `40`) seals the Seventh Ward grates. `Kaelen` confronts him (`vane_taunt` `talk`); `Vane` calls `execution_wall` at `50%` HP and withdraws at `25%` HP (`vane_withdraws`, `CL-01`) into Upper Cinderreach for Act 2, leaving `vanes_gilded_pavise` and `city_gate_pass`.
- **`M2` The Crossroads Cages (`03 Ash-Crawl Crossroads`)**: `Proctor Quell` (`34`, gaunt ex-junior to `Sabina`) runs an open-air titration pen where `Mortuary Flagellants` cull `tithe_captive_a` (Round 3) and `tithe_captive_b` (Round 5). `Maren` names the silver vivisection saw `Quell` used on her Scriptorium mentor; at `35%` HP (`or_round: 4`), `Quell` opens his brass neck-valve (`regen` + `enrage` + `transfusion_pulse`, `P-COMBAT-03`) and dies on the cobbles (`quells_marrow_scalpel`, `inquisitor_marrow_valve`, `slab_retorts`). Each surviving `18-24` `tithe_captive` adds `+1` `saved_youths` and slips the cell **`+1` `Rock Salt`** plus cage-seal rubbings (`CL-08`).

### 3.3 Tier 2: `M3` The Bone-Mill Raid & `M4` The Tallow Vats (Either Order)

- **`M3` The Bone-Mill Raid (`04 Fourth Kiln`)**: `Overseer Grendel` (`41`) grinds uncremated Spent corpses into bone-mortar for Inquisition watchposts. `Dunstan "Slag-Jaw"` (`22`, jaw wired shut after `Grendel`'s slag-ladle blow) fights as guest (`mill_oath` `talk`), joins into bunk `5` or `6`, and smashes `Grendel` (`grendels_slag_hammer`, `slag_plating`). If `M4` was cleared first, `Elyse` can deploy here.
- **`M4` The Tallow Vats (`05 Tallow Vats`)**: `Guild-Master Orlov` (`46`) leads `Render-Beasts` and `Tallow Dredgers` to burn out `Elyse` (`19`), whose counter-tonic woke her Spent mother's tenement crew. `Vesper` calls out `Orlov` for the tallow tithe that killed her brother (`orlov_taunt` `talk`); `Orlov` unleashes `scalding_cataclysm` at `40%` HP and withdraws at `20%` HP (`orlov_withdraws`, `CL-01`) into Upper Cinderreach, dropping `orlovs_flensing_talon` (`orlovs_retribution` + `+5` `Warded Tallow` on turn-in). `Elyse` joins into bunk `5` or `6`; if `M3` was cleared first, `Dunstan` can deploy here.

### 3.4 Causal Interlude: `ENC-EVT-05` The Awakening Fissure & Hub Debate 2

- **Node:** `00 Sluice Undercroft` (deep sub-basin drain-vault).
- **Causal Blowback (`CL-08`)**: Wrecking `M1`–`M4`'s Watch grates, titration cages, bone-mill chutes, and rendering vats backs raw abyssal condensate down the choked pipes until pressure ruptures the Sluice Undercroft floor—driving the `2x2` **`Calcified Amalgam`**, `3` `Ash-Crawlers`, and `2` `Calcified Husks` straight into the refuge.
- **Sabina's Dose & Hub Debate 2 (*The Price of Our Raids*)**: Pre-fight, `Chirurgeon Sabina` offers **1 deployed Founder** an emergency unrefined marrow dose (`sunk_ward.sabina_dose`), marking them for Act 2 Stage I `Vein-Blackening` + a free Tier I implant head-start. Post-fight, `Berek`, `Sabina`, and `Talia` clash over the ruptured stone: the cell's own victories are starving and backing up the city's marrow arteries (`dreamers_vertebra`, `titan_marrow_core`).

### 3.5 `M5` The Crucible & Hub Debate 3

- **Node:** `06 Crucible Watchpost -> 07 Slag Gate`.
- **Encounter (`enc_evt_06`, `party_min: 4`)**: `Inquisitor-Prelate Nemeth` (`unkillable` marrow-infused commander—`"Cold steel cannot finish me"`) + `2` `Inquisitorial Purifiers` + `2` `Castellan Footmen` + `2` `Watch Crossbowmen`. If `Hollis` completed *"The Crag-Sight Overlook"* (`sunk_ward.hollis_overlook`), the dropped chimney counterweight delays `gate_reinforcements` from Round `3` to Round `4` (`CL-08`).
- **The Winch & Hub Debate 3 (*The Empty Bunk*)**: One of the `4` deployed Founders holds winch `[10, 1]` (`stay_behind` -> `then: "capture"`), paying their bespoke `keepsake_<id>` (`CANON` 3.1, 5.2) as `Nemeth` takes them alive for `Lazar`. Beyond the Slag Gate, the `5` surviving Founders gather over the empty bunk (`Hub Debate 3`) before `Talia` brings in `1` class-matched `18-24` visored/cowled runaway (`CL-07`) and Act 2 switches on `Channel`, `Strain`, `Calcification`, `Slab` implants, and `Hearth` durability/techniques.

### 3.6 Six Founder Tactical Side-Hooks & Ward Board Jobs (`CL-08`)

Every Founder has an actionable Act 1 side-hook alongside ward board dilemmas (`PR5-38`):

1. **`Kaelen` — *"The Seventh Ward Bellows"* (`02 Alleyway Junction 7`)**: Raids his old Watch armory cache -> cuts `Berek`'s early Hearth Scrap costs & awards `1` `Slag Plating`.
2. **`Vesper` — *"The Sluice Lock-Ciphers"* (`01 Drowned Runoff Grate`)**: Maps forgotten flood-locks -> opens a permanent `0`-Alert **Siphon Shortcut** (`1d`, `0` Alert).
3. **`Maren` — *"Liturgical Cipher Rubbings"* (`03 Ash-Crawl Crossroads`, post-`M2`)**: Decodes cage-seal rubbings from rescued `M2` `tithe_captive`s (`+1` `saved_youths` + `+1` `Rock Salt` each) -> marks St. Lyra's hidden reliquary flue for Act 2.
4. **`Hollis` — *"The Crag-Sight Overlook"* (`06 Crucible Watchpost` outer approach)**: Secures chimney sightlines (`sunk_ward.hollis_overlook`), recovers **`Hollis's Lever-Winch`**, and drops the outer counterweight so `M5`'s `gate_reinforcements` wave arrives on **Round `4` instead of Round `3`**.
5. **`Dunstan` — *"Chain-Gang Wedges"* (`04 Fourth Kiln`, post-`M3`)**: Pulls black-iron splitting wedges from the ruined mill -> `+1` `Rock Salt` + opens **`Skerrit, Bone-Wright`**'s barbed bone cestus & quarrel recipes.
6. **`Elyse` — *"The Tenement Counter-Still"* (`05 Tallow Vats`, post-`M4`)**: Checks her awakened Spent mother's tenement block & distills tallow -> `+5` `Warded Tallow` + `1` early prototype **`Snapped Alum Inhaler`**.
7. **Neutral Stakes**: **`Talia`'s *"Night of the Red Pen"*** (`03` Toll-Pen -> rescues `12`-yo `Ivo`, opening **`Widow Varka`**'s Basin `Trade` shop), **`Bailiff Corvo`'s Third Ward Gate** (`enc_job_corvo_gate`: bribe Scrap/Salt to keep `Corvo` warm & protect `16`-yo `Ida`, or force loud `sunk_ward.corvo_forced`), **`Agnes of Block 12`**, and **`Brother Rook`**.

## 4. Act 2-3 Continuity of Act 1 Threads (`[LOCKED]`)

| Act 1 Encounter | Act 1 Outcome | Act 2 / Act 3 Payoff (`Survivor Inheritance` per `CL-09`) |
| --- | --- | --- |
| `enc_evt_01` (`Capt. Vane`) | Withdraws at `25%` HP | `Kaelen`'s Act 2 duel & debt-ledger raid (inherited by `Berek` + cell if `Kaelen` departed at `M5`) |
| `enc_evt_02` (`Proctor Quell`) | Slain at `M2` | `Slab Retorts` enable `Sabina`'s Act 2 ossuary-growth refinement; `Maren` uses cipher rubbings at St. Lyra's |
| `enc_evt_03` (`Overseer Grendel`) | Slain at `M3` | `Dunstan` + `Berek` hunt `Foundry-Master Jessop` in Act 2 & seize `3.2` Conduit blueprints (`Berek` leads if `Dunstan` departed) |
| `enc_evt_04` (`Guild-Master Orlov`) | Withdraws at `20%` HP | `Vesper`'s Act 2 Guild Counting-House raid (inherited by `Talia` + cell if `Vesper` departed) |
| `enc_evt_05` (`Calcified Amalgam`) | Slain in Sluice back-surge | `sunk_ward.sabina_dose` recipient starts Act 2 at Stage I + Tier I implant (`Q-94`: stronger `3.4` boss if captured at `M5`) |
| `enc_evt_06` (`Prelate Nemeth`) | `1` Founder captured; `Nemeth` escapes | Handover witnessed at `2.4` (`no rescue`); tragic `2`-phase confrontation at optional `3.4` Silent Ossuary (`node 20`) |

## 5. Data sketch `[data]`

Illustrative field names following `sunk_ward/{locations,routes}.json`:

```json
{
  "locations": [
    { "id": "sluice_undercroft", "name": "The Sluice Undercroft", "kind": "hub", "region": "sunk_ward" },
    { "id": "drowned_runoff_grate", "name": "Drowned Runoff Grate", "kind": "gate", "region": "sunk_ward" },
    { "id": "alleyway_junction_7", "name": "Alleyway Junction 7", "kind": "street", "encounters": ["enc_evt_01"] },
    { "id": "ash_crawl_crossroads", "name": "Ash-Crawl Crossroads", "kind": "street", "encounters": ["enc_evt_02"] },
    { "id": "fourth_kiln", "name": "The Fourth Kiln", "kind": "industrial", "encounters": ["enc_evt_03"] },
    { "id": "tallow_vats", "name": "The Tallow Vats", "kind": "industrial", "encounters": ["enc_evt_04"] },
    { "id": "crucible_watchpost", "name": "The Crucible Watchpost", "kind": "fortress", "encounters": ["enc_evt_06"] },
    { "id": "slag_gate", "name": "The Slag Gate", "kind": "gate", "region": "sunk_ward" }
  ],
  "routes": [
    { "id": "water_siphon", "from": "drowned_runoff_grate", "to": "alleyway_junction_7", "two_way": true, "time": 1, "encounter_pct": 15 }
  ]
}
```

## 6. Resolved design notes

- **M2 freed captives (`CL-08`)**: Each surviving `tithe_captive` (`18-24`) grants `+1` `saved_youths` and `+1` `Rock Salt` + cipher rubbings.
- **Orlov's vendetta (`CL-01`, `PR5-34`)**: `Orlov` withdraws at `20%` HP (`orlov_withdraws`), dropping `orlovs_flensing_talon` + `orlovs_retribution` (`+5` `Warded Tallow` on turn-in) so he survives for Act 2.
- **Board job nodes (`CL-08`)**: Assigned in Section 3.6 above (`01` Siphon Shortcut, `02` Bellows, `03` Red Pen & Cipher Rubbings, `04` Wedges, `05` Counter-Still, `06` Crag-Sight Overlook, `third_ward_gate` Corvo).
- **Wounded & Keepsakes (`PR5-34`, `PR5-35`)**: Act 1 `wounded` (`-20%` max HP until rested) and all `6` `keepsake_<id>` items are authored (`CANON` 5.2).
