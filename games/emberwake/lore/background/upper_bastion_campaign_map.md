# The High Bastion Campaign Graph & Narrative Encounter Architecture

> **Category:** Campaign Level Design, Overworld Graph & Narrative Staging  
> **Status:** Active Canonical Specification  
> **Location:** `games/emberwake/lore/background/upper_bastion_campaign_map.md`  
> **Reference Specs:** [world_geography_and_hub.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/world_geography_and_hub.md), [sunk_ward_campaign_map.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/sunk_ward_campaign_map.md), [enemy_bestiary_and_ai.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/enemy_bestiary_and_ai.md), [campaign_narrative_structure.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/campaign_narrative_structure.md), [full_game_systems_roadmap.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/full_game_systems_roadmap.md)  
> **Target Crates:** `emberwake_world` (`explore`, `travel`, `state`, `recruitment`), `emberwake_gui` (`region`, `hud`)

---

## 1. Executive Summary & Level Architecture

Act II of *Emberwake: The Low Sleep* takes place within **The High Bastion & St. Lyra's Cathedral District** (Upper Cinderreach). 

Following the harrowing sacrifice at the Crucible Watchpost ([ENC-EVT-06](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/sunk_ward_campaign_map.md#L143-L155)), the five surviving founders breach the Slag Gate and emerge into the wealthy, monumental heart of the Sovereign Clerisy's domain. Here, majestic gothic spires, polished basalt boulevards, and glowing magnesium lanterns conceal the regime's grim butchery: industrial embalming laboratories, public execution gallows, and iron culling stockades.

The overworld campaign is structured as an **8-Node Directed Graph** that expands tactical navigation with **verticality** (Street-Level Cobbles vs. Rooftop Gargoyle Rafters), timed **Curfew Searchlight Sweeps**, and moving **Culling Transport Convoys**:

```text
                                [ THE HIGH BASTION NODE GRAPH ]
                                               │
                                    [ 15. GRAND SPIRE GATE ]
                                   (Overlook to Mantle Fissure)
                                               ▲
                                               │ [Route: Sacred Chancel]
                                    [ 14. RELIQUARY CRYPTS ]
                                   (ENC-ACT2-06: Reliquary Heist Climax)
                                               ▲
                        ┌──────────────────────┴──────────────────────┐
                        │ [Route: Spire Buttress]                     │ [Route: Catacomb Flue]
             [ 13. GRAND BELLTOWER ]                       [ 11. EMBALMING CLOISTERS ]
            (ENC-ACT2-05: Chime Sabotage)                 (ENC-ACT2-04: Aline's Graft Lab)
                        ▲                                             ▲
                        │ [Route: Gargoyle Rafters]                   │ [Route: Scriptorium Duct]
             [ 12. CULLING STOCKADES ]                     [ 10. TALLOW RESERVOIR ]
            (ENC-ACT2-02: Convoy Jailbreak)               (ENC-ACT2-03: Fuel Silo Sabotage)
                        ▲                                             ▲
                        │ [Route: Gallows Avenue]                     │ [Route: Viaduct Aqueduct]
                        └──────────────────────┬──────────────────────┘
                                               │
                                    [ 09. GILDED BOULEVARD ]
                                   (ENC-ACT2-01: Checkpoint Breach)
                                               ▲
                                               │ [Breached Slag Portcullis]
                                    [ 08. SLAG GATE BREACH ]
                                   (Ingress from Act 1)
                                               │
                                               v [Concealed Siphon Runoff]
                                    [ THE SCRIBE'S CRYPT ]
                                   (Forward Safehouse Relay - 8 Bunks)
```

---

## 2. Overworld Navigation & Exploration Mechanics

Traversing the High Bastion introduces advanced stealth and risk/reward dynamics:

### 2.1 Vertical Traversal: Street-Level vs. Rooftop Rafters
Between nodes, players can choose between two travel routes:
1. **Street-Level Cobbles:**
   - *Travel Time:* 1 Time Unit.
   - *Inquisition Alert Risk:* High ($+10\%$ Alert per leg). Checkpoints are guarded by Castellan Sergeants and Heavy Shield-Bearers. High chance of encountering Inquisitorial Purifiers.
2. **Rooftop Rafters & Flying Buttresses:**
   - *Travel Time:* 2 Time Units (navigating narrow masonry and iron gutters).
   - *Inquisition Alert Risk:* Low ($+2\%$ Alert). Avoids ground checkpoints, but triggers **Structural Collapse** hazards (chance for operatives to suffer minor Stagger/fall damage) and exposes the party to Cathedral Crossbow Snipers.

### 2.2 Magnesium Searchlight Curfew & Beacon Sweeps
* Upper Cinderreach is illuminated by tower-mounted **Magnesium Arc-Lamps**.
* The District Map displays sweeping searchlight cones that sweep back and forth across boulevards every 2 Time Units.
* Entering a swept node while the beam is active immediately forces an ambush battle with $+25\%$ enemy accuracy and increases citywide alert by $+15\%$.

### 2.3 Moving Culling Convoys & Procedural Recruitment
* **Convoy Tracking:** Armored culling wagons move along `Node 09` $\to$ `Node 12` $\to$ `Node 11` every 3 Time Units, transporting captive youths to the dissection slabs.
* **Interception:** Engaging a convoy node triggers **`ENC-ACT2-02`**. Successfully liberating the cart unlocks **1–2 Procedural Recruits** (ages 15–24, "The Unsealed") to occupy empty bunks in the safehouse.

### 2.4 Forward Safehouse Relay: The Scribe's Crypt
* Located in the subterranean foundations beneath St. Lyra's scriptorium, accessed via `Node 08`.
* **Roster Expansion:** Expands safehouse capacity from 6 to **8 Bunks**, providing space for rescued procedural recruits.
* **Sister Aline's Surgical Rig:** Unlocks full **Apex Grafting** (Cranial, Thoracic, Forearm, and Visceral organ installation) and sulfur-flushes to control Calcification floors.

---

## 3. The Phase 2 Narrative Campaign: Integrating Placeholder Encounters

The six encounter events for Act II form the tactical core of Phase 2 (The Transhuman Rupture):

```text
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| PHASE 2 MISSION       | PRIMARY MAP NODE            | INTEGRATED ENCOUNTER EVENT    | NARRATIVE PROGRESSION PAYOFF     |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Mission 2.1: Ingress  | Node 09: Gilded Boulevard   | ENC-ACT2-01 (Vaelen's Guard)  | Break into Upper Cinderreach.    |
| Mission 2.2: Jailbreak| Node 12: Culling Stockades  | ENC-ACT2-02 (Culling Wagon)   | Unlock Procedural Recruits pool. |
| Mission 2.3: Sabotage | Node 10: Tallow Reservoir   | ENC-ACT2-03 (Silo Sabotage)   | Seize High-Grade Tallow & fuel.  |
| Mission 2.4: Graft Lab| Node 11: Embalming Cloister | ENC-ACT2-04 (Dr. Melchior)    | Aline secures Apex Grafting.     |
| Mission 2.5: The Chimes| Node 13: Grand Belltower   | ENC-ACT2-05 (Belltower Sentinel)| Silence acoustic resonance beacon|
| Mission 2.6: Climax   | Node 14: Reliquary Crypts   | ENC-ACT2-06 (Saint Sarcophagus)| Secure Mantle Key to breach Act 3|
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
```

---

## 4. Detailed Encounter Breakdowns (`ENC-ACT2-01` to `06`)

### 4.1 Mission 2.1: The Gilded Boulevard Breach
* **Primary Node:** `Node 09: Gilded Boulevard`.
* **Narrative Context:** Emerging from the smoking ruins of the Slag Gate, the five surviving founders must breach the outer curfew barrier manned by Inquisitor-General Vaelen’s elite iron vanguard.
* **Integrated Encounter:** **`ENC-ACT2-01: Gilded Avenue Checkpoint Breach`**.
  - *Adversaries:* Castellan Captain + 2 Inquisitorial Purifiers + 2 Greatshield Castellans.
  - *Tactical Focus:* Piercing high-armor Greatshields using Stagger wall-slams or initial Inoculated piercing strikes; dodging magnesium flash-bombs that inflict Blindness.
  - *Fallen Founder's Keepsake:* The squad equips the heirloom left behind by the hero sacrificed in Act 1, providing a crucial morale aura.

### 4.2 Mission 2.2: The Culling Wagon Jailbreak
* **Primary Node:** `Node 12: Inquisitorial Penitentiary & Culling Stockades`.
* **Narrative Context:** Word reaches the Scribe’s Crypt that a fresh levy of young factory apprentices has been locked into iron transit pens awaiting transfer to the cathedral dissection slabs.
* **Integrated Encounter:** **`ENC-ACT2-02: Culling Wagon Interception`**.
  - *Adversaries:* Proctor Jailer + 2 Bailiff Enforcers + 2 War Hounds (Render-Beast variants).
  - *Dynamic Objective:* The transport cart is set to depart on Round 4. The squad must breach the pen gate or kill the draft beasts before the wagon exits the map.
  - *Roster Payoff:* Liberates **Procedural Recruits** (`ProceduralRecruit` schema), replenishing lost squad slots.

### 4.3 Mission 2.3: The Tallow Reservoir Sabotage
* **Primary Node:** `Node 10: Merchant Guild Tallow Reservoir`.
* **Narrative Context:** Upper Cinderreach is powered and illuminated by three gargantuan tallow silos holding purified human fat. Sabotaging the pumps will plunge the district into blackout, crippling church searchlights.
* **Integrated Encounter:** **`ENC-ACT2-03: Tallow Silo Sabotage`**.
  - *Adversaries:* Guild Master's Provost + 3 Tallow Dredgers + 2 Crossbow Watchmen.
  - *Dynamic Hazard:* Silo release valves can be smashed to flood lanes with scalding grease, setting enemies ablaze or creating slick terrain.
  - *Strategic Payoff:* Lowers citywide Inquisition Alert by $-30\%$ and rewards 20 units of High-Grade Warded Tallow.

### 4.4 Mission 2.4: The Embalming Cloister Heist (Sister Aline's Quest)
* **Primary Node:** `Node 11: St. Lyra's Embalming Cloisters & Dispensary`.
* **Narrative Context:** Sister Aline Vance leads the squad into her former order’s private surgical laboratory to retrieve her confiscated surgical notes and the preserved gland of an Apex Aberration.
* **Integrated Encounter:** **`ENC-ACT2-04: The Embalming Cloister Infiltration`**.
  - *Adversaries:* Chief Anatomist Melchior + 2 Calcified Husks + 2 Inquisitorial Suture-Priests.
  - *Tactical Focus:* Suture-Priests actively regenerate husks; the squad must interrupt their surgical channeling on Tick 2.
  - *Progression Payoff:* Unlocks **Apex Grafting System (P2-02)** at Sister Aline’s surgical rig in the Scribe's Crypt. Discovers documents revealing the exact date of the **Red Tithing**.

### 4.5 Mission 2.5: Silencing the Grand Chimes
* **Primary Node:** `Node 13: Grand Belltower of the Slagged Saint`.
* **Narrative Context:** The massive bronze bells of the Grand Belltower chime every hour, transmitting acoustic vibrations that agitate the sleeping titan and send resonance waves through calcified bones.
* **Integrated Encounter:** **`ENC-ACT2-05: The Grand Belltower Sabotage`**.
  - *Adversaries:* The Bell-Warden Colossus (Heavy Mech-Automaton) + 2 Watch Snipers.
  - *Dynamic Hazard:* Bell Chimes fire every 2 timeline rounds, forcing all units to pass a Composure test or lose 15 Poise.
  - *Strategic Payoff:* Disables the **Acoustic Resonance Penalty** across Upper Cinderreach, preventing passive Strain gains during exploration.

### 4.6 Mission 2.6: The Reliquary Heist Climax (Act II Milestone)
* **Primary Node:** `Node 14: Reliquary Crypts of the First Apostles` $\to$ `Node 15: Grand Spire Gate`.
* **Narrative Context:** Deep beneath the cathedral floor lies the tomb of the First Extinguishers. Here rests the **Key of the Mantle Chasm**—the ancient iron seal granting access to the subterranean volcanic vents beneath the city.
* **Integrated Encounter:** **`ENC-ACT2-06: The Reliquary Heist Climax`**.
  - *Adversaries:* The Reliquary Guardian (Gilded Saint Sarcophagus animated by Apex Marrow) + 2 Inquisitor Castellans + 2 Choral Acolytes.
  - *Tactical Focus:* The Guardian is immune to physical Cold Steel until its four reliquary marrow-vials are smashed. High Strain surfing is required to bypass its defensive wards.
  - *Act II Conclusion:* Defeating the Guardian yields the *Key of the Mantle Chasm*. The squad unseals `Node 15: Grand Spire Gate` and looks down into the glowing, smoking caldera of the Sub-Basalt Abyss.
  - *The Phase Transition:* Unlocks **Phase 3 (The Awakening Abyss)**: The Red Tithing Doom Clock ignites, continental tremors shake the earth, and the descent into the Titan Catacombs begins.

---

## 5. Narrative Modularity & Side Quest Decoupling

The Act II encounter architecture is fully modular, allowing encounters to serve as repeatable or dynamic world events:

```text
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENCOUNTER EVENT       | PRIMARY MAIN QUEST ROLE     | MODULAR SIDE QUEST / CRISIS FALLBACK ROLE             |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT2-01 (Checkpt) | Boulevard Ingress           | Dynamic Roadblock: Re-spawns at random avenue nodes   |
|                       |                             | if Inquisition Alert exceeds 60%.                     |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT2-02 (Culling) | Roster Recruitment Unlock   | Repeatable Rescue: Procedural convoy appears every    |
|                       |                             | 5 travel ticks to rescue new procedural youths.       |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT2-03 (Tallow)  | Reservoir Sabotage          | Fuel Bounty: Sabotage side-silos to keep safehouse    |
|                       |                             | tallow stockpiles above critical threshold.           |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT2-04 (Cloister)| Aline's Grafting Quest      | Apex Trophy Hunt: Raid side dispensaries for rare     |
|                       |                             | anatomical monster harvest organs.                    |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT2-05 (Chimes)  | Belltower Infiltration      | Alert Suppression: Sabotage auxiliary searchlight     |
|                       |                             | mirrors to clear fog-of-war on map.                   |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT2-06 (Relic)   | Mantle Key Milestone        | Permanent Phase 2 Milestone: Triggers transition to   |
|                       |                             | Phase 3 and ignites the Red Tithing Doom Clock.       |
+-----------------------+-----------------------------+-------------------------------------------------------+
```

---

## 6. Engine Data Integration (`world_act2.json`)

Placeholder schema binding for `games/emberwake/crates/emberwake_world`:

```json
{
  "region_id": "upper_cinderreach_bastion",
  "locations": [
    {
      "id": "bastion_slag_gate_breach",
      "name": "Slag Gate Breach",
      "kind": "entry_checkpoint",
      "routes": ["bastion_gilded_boulevard"]
    },
    {
      "id": "bastion_the_scribes_crypt",
      "name": "The Scribe's Crypt",
      "kind": "hub_relay",
      "facilities": ["field_surgical_rig", "courier_blind", "dormitory_8_bunks"]
    },
    {
      "id": "bastion_gilded_boulevard",
      "name": "Gilded Boulevard",
      "kind": "street_exploration",
      "encounter_ref": "ENC-ACT2-01",
      "hazard_type": "magnesium_spotlights"
    },
    {
      "id": "bastion_tallow_reservoir",
      "name": "Merchant Guild Tallow Reservoir",
      "kind": "industrial_sabotage",
      "encounter_ref": "ENC-ACT2-03",
      "hazard_type": "scalding_fat_vats"
    },
    {
      "id": "bastion_embalming_cloisters",
      "name": "St. Lyra's Embalming Cloisters",
      "kind": "hospital_incursion",
      "encounter_ref": "ENC-ACT2-04",
      "hazard_type": "blessed_phosphor_traps"
    },
    {
      "id": "bastion_culling_stockades",
      "name": "Inquisitorial Penitentiary",
      "kind": "rescue_operation",
      "encounter_ref": "ENC-ACT2-02",
      "hazard_type": "iron_gallows_chokepoints"
    },
    {
      "id": "bastion_grand_belltower",
      "name": "Grand Belltower of the Slagged Saint",
      "kind": "spire_infiltration",
      "encounter_ref": "ENC-ACT2-05",
      "hazard_type": "acoustic_resonance_chimes"
    },
    {
      "id": "bastion_reliquary_crypts",
      "name": "Reliquary Crypts of the First Apostles",
      "kind": "dungeon_climax",
      "encounter_ref": "ENC-ACT2-06",
      "hazard_type": "saintly_wards"
    }
  ]
}
```
