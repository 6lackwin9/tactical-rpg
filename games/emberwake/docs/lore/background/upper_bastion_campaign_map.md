# The High Bastion: campaign graph and Act 2 missions

Status: canon (`2026-11-11`, synced with [CANON.md](../CANON.md) & [STORY_GUIDE.md](../STORY_GUIDE.md); Act 2 *"The First Rupture"*, `CL-02..CL-11`). Related:
[world geography](world_geography_and_hub.md), [Sunk Ward map](sunk_ward_campaign_map.md),
[NPCs and questlines](npcs_and_questlines.md), [ACTS23 spec](../../../../docs/specs/ACTS23.md),
[campaign structure](../narrative/campaign_narrative_structure.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`,
`[game code]`.

## 1. Overview

Act 2 takes place in **the High Bastion and St. Lyra's Cathedral district** (Upper Cinderreach). After the Crucible
([Sunk Ward map](sunk_ward_campaign_map.md) section 3.5), the `5` surviving Founders gather over the empty bunk (**Hub Debate 3**)
before `Talia Rusk` shelters `1` class-matched `18-24` visored/cowled runaway (`CL-07`) and the cell pushes through the Slag Gate
into the Clerisy's wealthy heart: soot-blackened gothic spires, polished basalt boulevards, and magnesium lamps over embalming
cloisters, gallows, and iron culling stockades.

The district graph has 8 nodes `[data]` with street vs. rooftop legs, timed magnesium searchlights, and moving culling convoys.

```text
                             [ THE HIGH BASTION NODE GRAPH ]
                                           |
                                [ 15. GRAND SPIRE GATE ]
                               (overlook to the Mantle Fissure)
                                           ^
                                           | [route: Sacred Chancel]
                                [ 14. RELIQUARY CRYPTS ]
                  (2.6, ENC-ACT2-06: Ruthven + First Apostle Saint-Warden)
                                           ^
                    +----------------------+----------------------+
                    | [route: Spire Buttress]                     | [route: Catacomb Flue]
         [ 13. GRAND BELLTOWER ]                       [ 11. EMBALMING CLOISTERS ]
      (2.5, ENC-ACT2-05: Penitent Bell-Warden)      (2.4, ENC-ACT2-04: Chief Anatomist Lazar)
                    ^                                             ^
                    | [route: Gargoyle Rafters]                   | [route: Scriptorium Duct]
         [ 12. CULLING STOCKADES ]                     [ 10. TALLOW RESERVOIR ]
        (2.2, ENC-ACT2-02: culling wagon)              (2.3, ENC-ACT2-03: silo sabotage)
                    ^                                             ^
                    | [route: Gallows Avenue]                     | [route: Viaduct Aqueduct]
                    +----------------------+----------------------+
                                           |
                                [ 09. GILDED BOULEVARD ]
                       (2.1, ENC-ACT2-01: Bastion curfew cordon)
                                           ^
                                           | [breached Slag portcullis]
                                [ 08. SLAG GATE BREACH ]
                                   (entry from Act 1)
                                           |
                                           v [concealed siphon runoff]
                              [ 00. SLUICE UNDERCROFT ]
                     (single campaign hub; bunks 6 -> 8 via Berek's Hearth)
```

## 2. Travel, Convoys & Hub Continuity

### 2.1 Street or rooftop

Each leg offers two routes (`ACTS23` `PK1`, `[data]`):

1. **Street cobbles:** `1` day; `+10%` Alert per leg. Checkpoints held by Castellan Footmen, Shield-Bearers, and Inquisitorial Purifiers.
2. **Rooftop rafters and flying buttresses:** `2` days; `+2%` Alert. Avoids street checkpoints, risking Watch Crossbowmen.

Both cost `1` Warded Tallow per day (`CANON` 4.2); Act 2 raises the **Alert floor to `30`** (`Guarded` is gone above the Slag Gate).

### 2.2 Magnesium searchlights & Curfew Bells (`CL-05`)

- Upper Cinderreach is lit by tower-mounted magnesium arc-lamps sweeping exposed boulevards every `2` days (`+15%` Alert on lit entry).
- **Curfew Bell Resonance (`CL-05`)**: Until `2.5 Grand Belltower` falls, St. Lyra's curfew bells deal **`+10` starting Strain** to every Stage I+ operative in Upper Cinderreach battles.

### 2.3 Culling convoys, recruits & `25+` non-combat rescues (`CL-07`)

- **Act 2 Entry Recruit (`CL-07`)**: `Talia` shelters `1` class-matched `18-24` visored/cowled runaway into the `M5` bunk (`6/6` filled; expandable to `8` via `Berek`'s *"Expanded Timber Frames"* Hearth project).
- **Convoys (`2.2` Culling Stockades)**: Intercepting a culling wagon (`enc_act2_02`) yields `1-2` visored/cowled `18-24` Unsealed recruits (`RECRUIT1`) for open bunks; freed Unsealed under `18`—including **`Lucan`** (`14`, `Berek`'s former Fourth Kiln apprentice traded to the Tithing by `Foundry-Master Jessop`) and **`Wren`** (`14`, `Hollis`'s sister)—are sheltered by `Talia` as non-combat Basin wards (`+saved_youths`). Rescued `25+` Spent specialists (`Ferrand` `40`, `Herbalist Clemence` `36`) stay bunk-free Basin guests (`CL-07`).

### 2.4 Single Hub (`Sluice Undercroft`) & Siege Stakes

- The cell operates out of the **Sluice Undercroft** across all three acts (`ACTS23` `Q-90 a`; `The Scribe's Crypt` is a narrative forward blind, not a second hub).
- **`Alert 100` Hub Breach (`CL-06`, `CL-10`)**: Triggers a Sluice Hub Siege where **`Chirurgeon Sabina`** and **`Berek`** stand on `protect` tiles (`ACTS23` `PK1`) while **`Inquisitor-General Ruthven` ("the Blind Eye")** leads the breach—withdrawing at **`25%` HP (`ruthven_withdraws`)** so he always survives to command the `2.6` Reliquary Gate climax. Losing a siege costs `10` Warded Tallow, `50` Scrap, `+2d` facility lock, and resets Alert to `60` (never a Game Over).

## 3. Act 2 Main Missions, 6 Dual-Rituals & Survivor Inheritance (`CL-02`, `CL-09`, `CL-10`, `CL-11`)

| # | Mission | Node | Encounter | Payoff |
| --- | --- | --- | --- | --- |
| `2.1` | Ingress | `09` Gilded Boulevard | `enc_act2_01`, Bastion curfew cordon | Foothold in Upper Cinderreach (`Ruthven` inspects & withdraws before clash) |
| `2.2` | Jailbreak | `12` Culling Stockades | `enc_act2_02`, culling wagon (repeatable) | `18-24` Cohort recruits; rescues `14`-yo `Lucan` (`Berek` stake) |
| `2.3` | Sabotage | `10` Tallow Reservoir | `enc_act2_03`, silo sabotage | `+20` `Warded Tallow`, `-30%` Alert |
| `2.4` | The Cloisters | `11` Embalming Cloisters | `enc_act2_04`, `Chief Anatomist Lazar` (`Sabina`'s quest) | `Lazar` slain -> `Sabina's Prosector Crook`, Tier III Apex implants & bone-scraping; `Nemeth` captive handover (`no rescue`); **Hub Debate 4** (`Great Tithing` `max(days, 18)`) |
| `2.5` | The Chimes | `13` Grand Belltower | `enc_act2_05`, **Penitent Bell-Warden** (Stage IV) | Silences curfew-bell `+10` Strain resonance (`CL-05`, `CL-10`) |
| `2.6` | The Reliquary | `14` Reliquary Crypts -> `15` Grand Spire Gate | `enc_act2_06`, **Inquisitor-General Ruthven + First Apostle Saint-Warden** | First Extinguishers' tomb truth; **Key of the Mantle Chasm**; opens Act 3 |

### 3.1 Six Dual-Path Personal Quests & Survivor Inheritance (`CL-02`, `CL-09`)

Every Founder has an Act 2 personal reckoning offering **Cold Steel vs. Inoculated** completion (`CL-02`). **Survivor Inheritance (`CL-09`)**: if that Founder was captured on the `M5` winch or fell in Act 2 (`roster.departed.<id>`), the mission **never vanishes**—their closest Hub mentor and surviving cellmates inherit the raid, paying cell-wide Hearth/Slab relics and Doctrine shifts instead of that Founder's personal keystone:

1. **`Kaelen` (or `Berek` + cell)**: Hunts **`Capt. Vane`** in the Seventh Ward Watch garrison (`Vane` dies; burns or leverages his Bastion debt ledgers -> `Deserter's Kettle Helm` / cell-wide Watch plate).
2. **`Vesper` (or `Talia` + cell)**: Corners **`Guild-Master Orlov`** in his Guild Counting-House (`Orlov` dies; seizes `+10` `Warded Tallow` & tallow ledgers).
3. **`Maren` (or `Sabina` + cell)**: Unseals St. Lyra's Scriptorium Forbidden Vault using `M2`'s cipher rubbings.
4. **`Hollis` (or `Talia` + cell)**: Intercepts `Ruthven`'s decoy convoy to rescue `Hollis`'s `14`-yo sister **`Wren`** into Basin shelter.
5. **`Dunstan` + `Berek` (or `Berek` + cell)**: Storms the Fourth Kiln inner foundry to kill **`Foundry-Master Jessop`**, flood the bone-mortar moulds with molten iron, and seize `Jessop`/`Berek`'s **`3.2` Marrow Conduit** valve blueprints.
6. **`Elyse` (or `Sabina` + cell)**: Sabotages the Cathedral sedative cisterns feeding the Lullaby Houses.

## 4. Main Mission Encounters

### 4.1 Mission `2.1`: Gilded Boulevard Breach (`enc_act2_01`)

- **Story:** Emerging from the smoking Slag Gate, the squad glimpses `Inquisitor-General Ruthven` ("the Blind Eye", `Hollis`'s quarrel-dent scarred across his left iron mask-plate) setting his hounds onto `Talia`'s couriers before leaving his Castellan cordon to hold the avenue.
- **Play:** First test of `Channel` armour-piercing strikes vs. Cold Steel stagger/techniques against heavy Bastion shields.

### 4.2 Mission `2.2`: Culling Stockades (`enc_act2_02`)

- **Story:** Fresh levies of `18-24` Unsealed apprentices and `14`-yo `Lucan` wait in iron transit pens bound for `Lazar`'s Cloisters.
- **Objective:** Breach the pen gate before Round 4 departure -> `RECRUIT1` cohort recruits + `Lucan` brought safely to `Berek`.

### 4.3 Mission `2.3`: Tallow Reservoir (`enc_act2_03`)

- **Story:** Wrecking Upper Cinderreach's three Guild tallow silos blacks out magnesium arc-lamps (`-30%` Alert, `+20` `Warded Tallow`).

### 4.4 Mission `2.4`: Embalming Cloisters & Hub Debate 4 (`enc_act2_04`)

- **Story (`Sabina`'s Quest)**: `Sabina` directs the cell into her former mentor **`Chief Anatomist Lazar`**'s crimson-and-pearl surgical cloister (`Lazar` + `2` `Calcified Husks` + `2` Suture-Priests). Through sealed leaded glass, the squad witnesses `Nemeth` hand the `M5` captive over to `Lazar`'s inner ward—too late to rescue (`Q-33`).
- **Hub Debate 4 (*The Ledger of the Lullaby Houses*)**: `Lazar` dies (`Sabina's Prosector Crook`, Tier III Apex implants & Slab bone-scraping unlocked), and his ledger reveals the whole engine: **Lullaby Houses** harvest Unsealed blood, `3.2` Conduits pump **steeped sacrificial ichor** *up* from `21` High Altar to `Lazar`, and the cell's own raids starved the titan's sleep—forcing `Othemar` to call a **second Great Tithing** (`max(days, 18)`, `CL-04`). Back at the Sluice, `Sabina`, `Berek`, `Talia`, and the surviving Founders debate how Cinderreach can survive without child blood (`CL-11`).

### 4.5 Mission `2.5`: Grand Belltower (`enc_act2_05`, `CL-10`)

- **Story & Ossuary Boss (`CL-10` — no clockwork construct)**: St. Lyra's curfew bells are rung by a Stage IV **`Penitent Bell-Warden`**—a calcifying living martyr chained and bone-pinned directly into the great bronze bell-yoke, flanked by `2` Watch Crossbowmen. Every `2` rounds the bell-toll strips Poise and spikes Strain; slaying the Bell-Warden permanently silences Upper Cinderreach's `+10` Strain resonance (`CL-05`).

### 4.6 Mission `2.6`: Reliquary Crypts Climax (`enc_act2_06`, `CL-10`)

- **Story & Climax Boss (`CL-10`)**: Beneath the cathedral nave lies the tomb of the **First Extinguishers** (revered publicly as the *First Apostles*), holding their 200-year confession of the first Great Tithing and the **Key of the Mantle Chasm**. **`Inquisitor-General Ruthven`** (`4`-screw blind iron mask, collarbone tremor-grafts) makes his stand to the death here alongside an awakened **First Apostle Saint-Warden** (`Lazar`-embalmed reliquary corpse-knight, replacing generic sarcophagus constructs) and `2` Inquisitorial Purifiers.
- **Transition:** Claiming the Mantle Key opens `15 Grand Spire Gate` into Act 3 (*The Awakening Abyss*): the Doom Clock turns **hard** (`min 12d`, `CL-04`).

## 5. Data sketch `[data]`

```json
{
  "locations": [
    { "id": "slag_gate_breach", "name": "Slag Gate Breach", "kind": "gate", "region": "high_bastion" },
    { "id": "gilded_boulevard", "name": "Gilded Boulevard", "kind": "street", "encounters": ["enc_act2_01"] },
    { "id": "tallow_reservoir", "name": "Tallow Reservoir", "kind": "industrial", "encounters": ["enc_act2_03"] },
    { "id": "embalming_cloisters", "name": "St. Lyra's Embalming Cloisters", "kind": "cloister", "encounters": ["enc_act2_04"] },
    { "id": "culling_stockades", "name": "Culling Stockades", "kind": "prison", "encounters": ["enc_act2_02"] },
    { "id": "grand_belltower", "name": "Grand Belltower of the Slagged Saint", "kind": "spire", "encounters": ["enc_act2_05"] },
    { "id": "reliquary_crypts", "name": "Reliquary Crypts of the First Apostles", "kind": "crypt", "encounters": ["enc_act2_06"] },
    { "id": "grand_spire_gate", "name": "Grand Spire Gate", "kind": "gate", "region": "high_bastion" }
  ]
}
```

## 6. Resolved design notes

- **Act 2 entry recruit & non-combat rescues (`CL-07`)**: `1` class-matched `18-24` visored/cowled runaway joins at Act 2 start; under-`18` (`Lucan`, `Wren`) and `25+` (`Ferrand`, `Clemence`) rescues stay non-combat Basin wards/guests.
- **Doom Clock at `2.4` (`CL-04`)**: Relabelled *the Great Tithing* and set to `max(days, 18)`.
- **`2.5` & `2.6` Bosses (`CL-10`)**: `Penitent Bell-Warden` (`2.5`) and `Inquisitor-General Ruthven + First Apostle Saint-Warden` (`2.6`) replace generic clockwork/animated-sarcophagus RPG tropes; `Ruthven` withdraws at `25%` HP in any earlier Hub Breach.
