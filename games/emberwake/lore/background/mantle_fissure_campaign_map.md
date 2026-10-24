# The Mantle Fissure Campaign Graph & Narrative Encounter Architecture

> **Category:** Campaign Level Design, Overworld Graph & Narrative Staging  
> **Status:** Active Canonical Specification  
> **Location:** `games/emberwake/lore/background/mantle_fissure_campaign_map.md`  
> **Reference Specs:** [world_geography_and_hub.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/world_geography_and_hub.md), [sunk_ward_campaign_map.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/sunk_ward_campaign_map.md), [upper_bastion_campaign_map.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/upper_bastion_campaign_map.md), [campaign_narrative_structure.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/campaign_narrative_structure.md), [full_game_systems_roadmap.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/full_game_systems_roadmap.md)  
> **Target Crates:** `emberwake_world` (`explore`, `travel`, `state`, `crisis`, `campaign`), `emberwake_battle` (`resolver`, `session`), `emberwake_gui` (`region`, `hud`)

---

## 1. Executive Summary & Level Architecture

Act III of *Emberwake: The Low Sleep* takes place within **The Mantle Fissure & Titan Catacombs** (The Sub-Basalt Abyss).

Having shattered the Reliquary Crypts in Upper Cinderreach and claimed the *Key of the Mantle Chasm* ([ENC-ACT2-06](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/upper_bastion_campaign_map.md#L173-L188)), the squad unseals the Grand Spire Gate and descends thousands of feet into the subterranean caldera.

Here, far beneath the city's foundations, lies the physical resting place of the primordial cosmic entity: **The Low Sleep**. Pulsing veins of black mutagenic marrow the thickness of aqueducts snake through boiling basalt canyons. Above, the city's cathedral bells resonate down through the stone like thunderclaps. The Inquisition is mobilizing its ultimate rite—the **Red Tithing**—to feed thousands of captive youths to the entity's waking maw to siphon its godly blood.

The overworld campaign is structured as an **8-Node Directed Graph** characterized by brutal environmental friction, the ticking **Red Tithing Doom Clock**, and the threat of catastrophic **Continental Resonance**:

```text
                                [ THE MANTLE FISSURE NODE GRAPH ]
                                                │
                                    [ 23. THE DREAMING CORE ]
                                   (ENC-ACT3-06: The Titan Waking Macro-Boss)
                                                ▲
                                                │ [Route: Umbilical Trench]
                                    [ 21. HIGH SACRIFICIAL ALTAR ]
                                   (ENC-ACT3-05: Arch-Prelate Othemar)
                                                ▲
                        ┌───────────────────────┴───────────────────────┐
                        │ [Route: Crypt of Apostles]                    │ [Route: Pulsing Vein Chasm]
             [ 20. APOSTLES' CRYPT ]                         [ 18. MARROW CONDUITS ]
            (ENC-ACT3-04: The Corrupted Martyr)             (ENC-ACT3-02: Conduit Severance)
                        ▲                                               ▲
                        │ [Route: Chasm Rift]                           │ [Route: Fissure Bridges]
             [ 19. CHORUS REDOUBT ]                          [ 17. WEEPING BASALT CHASM ]
            (ENC-ACT3-03: Heretic Choir)                    (ENC-ACT3-01: Chasm Gatekeepers)
                        ▲                                               ▲
                        │ [Route: Cavern Siphon]                        │ [Route: Basalt Stair]
                        └───────────────────────┬───────────────────────┘
                                                │
                                    [ 16. CHASM INGRESS ]
                                   (Transition from Act 2)
                                                │
                                                v [Reinforced Basalt Vault]
                                    [ THE WEEPING REDOUBT ]
                                   (Forward Expedition FOB - 8 Bunks)
```

---

## 2. Overworld Navigation & Macro-Crises Mechanics

Traversing the Mantle Fissure forces players to navigate apocalyptic world pressures:

### 2.1 The Red Tithing Doom Clock ($D_0 \dots D_{\text{Tithing}}$)
* From the moment Act III begins, a persistent world clock displays days and hours remaining until the Clerisy's continental slaughter.
* Traversing a route costs **1 Time Unit**; resting in the safehouse costs **2 Time Units**.
* If the Doom Clock reaches zero before `Node 21: High Sacrificial Altar` is captured, the titan fully stirs and the campaign ends in **True Campaign Failure (The Great Extinguishing)**.

### 2.2 Continental Resonance & Bell Shockwaves
* As the titan shifts, cathedral bells in the city above ring in acoustic sympathy.
* Moving through unwarded nodes causes all operatives with $\ge 1$ point of Calcification to suffer resonant bone vibrations, passively generating $+10\%$ Strain before combat begins.
* In combat, every 3 rounds triggers a **Continental Shockwave**, forcing all combatants to roll Composure or suffer 10 true damage and -15 Poise.

### 2.3 Atmospheric Strain & Warded Tallow Depletion
* The subterranean atmosphere is saturated with vaporized raw marrow.
* Traveling between nodes consumes **2 Units of Warded Tallow** per leg to keep the expedition's ward-lanterns lit.
* Traveling unsupplied inflicts $+15\%$ baseline Strain on all operatives upon map entry and increases the chance of roamer encounters to $60\%$.

### 2.4 Forward Expedition FOB: The Weeping Redoubt
* Anchored into the petrified marrow arches flanking `Node 16: Chasm Ingress`.
* **Field Quench-Anvil:** Bram Karr uses concentrated basalt heat to forge **Black-Iron penetrators**, granting weapons $+100\%$ Poise damage against titan bone carapaces.
* **Isolation Crypts:** Reinforced iron cages where Stage IV Calcified operatives ("The Shakes") are secured between battles so their psychosis does not spread panic to junior recruits.
* **Living Martyr Authorization:** Allows authorizing a final deployment for a Stage IV operative ([System P2-03](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/full_game_systems_roadmap.md#L121-L129)), granting them aberrant trance before permanent petrification into an authored relic.

---

## 3. The Phase 3 Narrative Campaign: Integrating Placeholder Encounters

The six encounter events for Act III form the climax of Phase 3 (The Awakening Abyss):

```text
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| PHASE 3 MISSION       | PRIMARY MAP NODE            | INTEGRATED ENCOUNTER EVENT    | NARRATIVE PROGRESSION PAYOFF     |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Mission 3.1: Descent  | Node 17: Weeping Chasm      | ENC-ACT3-01 (Gatekeepers)     | Establish subterranean foothold. |
| Mission 3.2: Conduits | Node 18: Marrow Conduits    | ENC-ACT3-02 (Conduit Severance)| Sabotage Clerisy's marrow pumps. |
| Mission 3.3: The Choir| Node 19: Chorus Redoubt     | ENC-ACT3-03 (Brother Malachi) | Settle heretic cult faction war. |
| Mission 3.4: The Martyr| Node 20: Apostles' Crypt   | ENC-ACT3-04 (Corrupted Founder)| Tragic mercy-kill of lost comrade|
| Mission 3.5: High Altar| Node 21: High Sacrif. Altar | ENC-ACT3-05 (Arch-Prelate)    | Slay Othemar; stop Red Tithing.  |
| Mission 3.6: The Sleep| Node 23: The Dreaming Core  | ENC-ACT3-06 (Titan Macro-Boss)| Campaign Climax: Endings A, B, C.|
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
```

---

## 4. Detailed Encounter Breakdowns (`ENC-ACT3-01` to `06`)

### 4.1 Mission 3.1: The Chasm Gatekeepers
* **Primary Node:** `Node 17: Weeping Basalt Chasm`.
* **Narrative Context:** The descent into the fissure is barred by ancient primeval horrors—apex burrowing aberrations awakened by the Inquisition's deep drilling operations.
* **Integrated Encounter:** **`ENC-ACT3-01: The Chasm Gatekeepers`**.
  - *Adversaries:* 2 Bone-Chitin Colossi (Apex Aberrations) + 3 Ash-Crawlers.
  - *Dynamic Hazard:* Unstable basalt shelves; stepping onto fractured rock causes tiles to crumble into the molten abyss after 1 timeline round.
  - *Tactical Focus:* Staggering heavy colossi with Black-Iron weapons and utilizing knockbacks to drop beasts into fissure crevices.

### 4.2 Mission 3.2: The Marrow Conduit Severance
* **Primary Node:** `Node 18: Marrow Conduits`.
* **Narrative Context:** Massive brass distillation siphons are pumping raw, living marrow from the titan's arteries up to the Cathedral’s reservoirs. Severing these conduits deprives church inquisitors of their regenerative serum.
* **Integrated Encounter:** **`ENC-ACT3-02: Marrow Conduit Severance`**.
  - *Adversaries:* High Castellan Vane (Reconstructed Inquisitor) + 2 Inquisitorial Suture-Priests + 2 Purifier Warmachines.
  - *Dynamic Objective:* Four pulsing conduit valves must be sundered while under fire. Severing a conduit floods adjacent tiles with mutagenic marrow, granting temporary $+100\%$ true damage to anyone standing in it at the cost of $+20\%$ Strain per round.
  - *Strategic Payoff:* Deprives the Inquisition of regenerative buffs during subsequent battles and delays the Red Tithing Doom Clock by $+2$ days.

### 4.3 Mission 3.3: The Heretic Choir (Brother Malachi's Sanctum)
* **Primary Node:** `Node 19: The Awakening Chorus Redoubt`.
* **Narrative Context:** Brother Malachi and the fanatic cult of the Sub-Basalt Cults have established an altar over a resonant geyser, singing harmonic chants that hasten the titan’s awakening.
* **Integrated Encounter:** **`ENC-ACT3-03: The Awakening Chorus Ritual`**.
  - *Adversaries:* Brother Malachi (Choral Patriarch) + 4 Chorus Zealots + 2 Calcified Husks.
  - *Dynamic Hazard:* Sonic resonance geysers erupting on Tick 1 and Tick 4, inflicting Deafened and -20 Poise on anyone within 2 tiles.
  - *Faction Dilemma:* If the player has high *Flesh-Broker* alignment, Malachi can be reasoned with to direct the choral chants against the Church's high altar. Otherwise, the cult must be eradicated.

### 4.4 Mission 3.4: The Corrupted Martyr (The Return of the Fallen Founder)
* **Primary Node:** `Node 20: The Silent Crypt of the Apostles`.
* **Narrative Context:** In the deepest mausoleum, the squad encounters a devastating sight: the founding comrade sacrificed at the Crucible Watchpost in Act 1 did not die peacefully. Their corpse was claimed by church anatomists, wired with black-iron scaffolding, and pumped full of raw titan marrow to create an Inquisitorial Saint-Aberration.
* **Integrated Encounter:** **`ENC-ACT3-04: The Corrupted Martyr`**.
  - *Adversaries:* The Martyr-Saint (Corrupted Founder Boss) + 2 Suture-Priests.
  - *Boss Mechanics:* The boss retains the signature maneuvers and class abilities of the sacrificed founder (e.g. Kaelen's *Shield Slam*, Vesper's *Shadow Leap*), twisted with high-tier aberrant mutations.
  - *Emotional Payoff:* Defeating the Martyr frees their soul. Slaying them grants their final, perfected heirloom: **The Founder’s Transcendent Relic**, carrying unique party-wide passive buffs.

### 4.5 Mission 3.5: Arch-Prelate Othemar's Last Sacrament
* **Primary Node:** `Node 21: High Sacrificial Altar`.
* **Narrative Context:** Atop a cyclopean basalt altar suspended directly over the titan’s gaping heart, Arch-Prelate Othemar prepares to slaughter the remaining hundred captive youths to complete the Red Tithing.
* **Integrated Encounter:** **`ENC-ACT3-05: Arch-Prelate Othemar's Last Sacrament`**.
  - *Adversaries:* Arch-Prelate Othemar (mounted in a gargantuan gold-and-iron Reliquary Mech-Chassis) + 2 Castellan Inquisitors + 2 Purifier Executioners.
  - *Dynamic Objective:* Operatives must eliminate the Executioners within 4 rounds to prevent the sacrifice of captive youths while surviving Othemar's sweeping magnesium lasers and holy fire mortars.
  - *Narrative Milestone:* Othemar is shattered and hurled into the chasm. The Red Tithing Doom Clock is halted permanently.

### 4.6 Mission 3.6: The Grand Climax (The Titan Waking Macro-Boss)
* **Primary Node:** `Node 23: The Dreaming Core`.
* **Narrative Context:** The death of Othemar and the severance of the conduits creates an umbilical shockwave. The Low Sleep titan begins to open its colossal, abyssal eyes. The cavern collapses as the squad confronts the cosmic source of the world's curse.
* **Integrated Encounter:** **`ENC-ACT3-06: The Titan Waking Macro-Boss`**.
  - *Multi-Phase Environmental Boss:*
    - *Phase 1 (The Tendril Surge):* Battling titan nerve-tendrils that slam the arena, shattering cover and creating lava rifts.
    - *Phase 2 (The Eye of the Abyss):* The titan's optic iris opens, flooding the battlefield with reality-warping psychic Strain. Operatives must cycle high-Strain Inoculated bursts with Cold Steel Poise shields.
    - *Phase 3 (The Terminal Decision):* The squad reaches the titan's Central Synapse Core, triggering the final campaign choice:
  - *The Three Campaign Endings:*
    1. **Ending A (The Cold Steel Line):** Bram Karr's pure path. Drive cold black-iron spikes into the core to sever the synapse, putting the titan into eternal coma. Humanity remains fragile, mortal, and cold, but free of aberration taint.
    2. **Ending B (The Transhuman Ascendancy):** Sister Aline's transhuman path. Graft the cell's nervous systems directly into the titan's core. The squad merges with the entity, seizing immortal control over Cinderreach and ruling as new demigods.
    3. **Ending C (The Extinguished):** Triggered if all operatives succumb to Stage V Calcification or the core collapses: the titan shatters the mountain, obliterating Cinderreach in a cataclysm of living stone.

---

## 5. Narrative Modularity & Side Quest Decoupling

The Act III encounter architecture supports late-game crisis events and side expeditions:

```text
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENCOUNTER EVENT       | PRIMARY MAIN QUEST ROLE     | MODULAR SIDE QUEST / CRISIS FALLBACK ROLE             |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT3-01 (Gatekpr) | Chasm Ingress Battle        | Subterranean Quarry: Farm rare black basalt ore to    |
|                       |                             | craft endgame Black-Iron armor.                       |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT3-02 (Conduit) | Conduit Severance           | Fuel Deprivation: Cut auxiliary pipelines to halt     |
|                       |                             | church mechanical walker reinforcements.              |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT3-03 (Chorus)  | Heretic Choir Confrontation | Choral Pact: Optional faction alliance allowing cult  |
|                       |                             | zealots to fight alongside the cell.                  |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT3-04 (Martyr)  | The Lost Founder's Rest     | Psychological Closure: Unlocks maximum Composure for  |
|                       |                             | surviving squad members.                              |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT3-05 (Othemar) | High Altar Liberation       | Defuse the Red Tithing Doom Clock.                    |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-ACT3-06 (Titan)   | Macro-Boss Campaign Climax  | Campaign Finale: Resolves narrative into Endings A, B,|
|                       |                             | or C based on player choices and survival.            |
+-----------------------+-----------------------------+-------------------------------------------------------+
```

---

## 6. Engine Data Integration (`world_act3.json`)

Placeholder schema binding for `games/emberwake/crates/emberwake_world`:

```json
{
  "region_id": "sub_basalt_mantle_fissure",
  "locations": [
    {
      "id": "mantle_chasm_ingress",
      "name": "The Chasm Ingress",
      "kind": "descent_checkpoint",
      "routes": ["mantle_weeping_basalt_chasm"]
    },
    {
      "id": "mantle_the_weeping_redoubt",
      "name": "The Weeping Redoubt",
      "kind": "fob_relay",
      "facilities": ["quench_anvil", "isolation_crypts", "ward_lantern_rig"]
    },
    {
      "id": "mantle_weeping_basalt_chasm",
      "name": "Weeping Basalt Chasm",
      "kind": "abyssal_exploration",
      "encounter_ref": "ENC-ACT3-01",
      "hazard_type": "crumbling_basalt_shelves"
    },
    {
      "id": "mantle_marrow_conduits",
      "name": "Pulsing Marrow Conduits",
      "kind": "artery_sabotage",
      "encounter_ref": "ENC-ACT3-02",
      "hazard_type": "mutagenic_marrow_sprays"
    },
    {
      "id": "mantle_chorus_redoubt",
      "name": "The Awakening Chorus Redoubt",
      "kind": "heretic_sanctum",
      "encounter_ref": "ENC-ACT3-03",
      "hazard_type": "acoustic_sonic_geysers"
    },
    {
      "id": "mantle_apostles_crypt",
      "name": "The Silent Crypt of the Apostles",
      "kind": "mausoleum_incursion",
      "encounter_ref": "ENC-ACT3-04",
      "hazard_type": "petrified_bone_barricades"
    },
    {
      "id": "mantle_high_altar",
      "name": "High Sacrificial Altar",
      "kind": "altar_assault",
      "encounter_ref": "ENC-ACT3-05",
      "hazard_type": "magnesium_fire_mortars"
    },
    {
      "id": "mantle_dreaming_core",
      "name": "The Dreaming Core",
      "kind": "macro_boss_climax",
      "encounter_ref": "ENC-ACT3-06",
      "hazard_type": "psychic_strain_waves"
    }
  ]
}
```
