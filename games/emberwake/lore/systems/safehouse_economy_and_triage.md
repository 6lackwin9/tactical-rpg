# Safehouse Economy, Triage Systems & UI Architecture

> **Category:** Macro Gameplay Systems, Economy & UI/UX Specifications  
> **Status:** Active Canonical Specification  
> **Location:** `games/emberwake/lore/systems/safehouse_economy_and_triage.md`  
> **Target Crates:** `emberwake_gui` (`hub`, `party`, `shop`, `forge`, `hud`), `emberwake_world` (`state`, `save`), `aegis_ui` (`window`, `ui`, `canvas`)  
> **Reference Lore:** [master_specification.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md), [world_geography_and_hub.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/world_geography_and_hub.md), [screen_architecture_and_separation_plan.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/screen_architecture_and_separation_plan.md)

---

## 1. Architectural Philosophy: The Macro Gameplay Loop

While combat in *Emberwake: The Low Sleep* provides visceral, high-stakes tactical chess across the 6-tick timeline, the game is sustained by an equally unforgiving **Safehouse Macro-Economy**. 

Between dangerous expeditions into the quarantined districts of Cinderreach, the insurgent cell retreats to their subterranean haven: **The Sluice Undercroft**.

```text
                                [ THE CLOSED MACRO EXPEDITION LOOP ]
                                                 │
                   ┌─────────────────────────────┴─────────────────────────────┐
                   ▼                                                           ▼
       [ EXPEDITION INFILTRATION ]                                 [ THE SLUICE UNDERCROFT HAVEN ]
       • District Map node navigation                              • 4 Functional Operational Wings
       • Lantern cone & curfew evasion                             • Resting & trauma triage (Warded Tallow)
       • Tactical 6-Tick Combat Encounters                         • Active Maneuver Deck loadout assembly
       • Extraction with harvested spoils                          • Surgical Marrow Grafts & Cold Steel Forging
                   │                                                           ▲
                   └─────────────────[ RETURN TO SAFEHOUSE ]───────────────────┘
```

### 1.1 How Each System Contributes to the Gameplay Loop
1. **The Infiltration Phase (Resource Inflow):** Sunk Ward expeditions reward four critical survival resources: *Scrap Iron*, *Warded Tallow Vials*, *Rock Salt*, and *Harvested Aberrant Organs*.
2. **The Haven Phase (Resource Sinks & Tough Trade-offs):**
   - **The Apothecary's Slab (Sister Aline Vance):** Players spend harvested organs to surgically graft Inoculated skills onto operatives, permanently raising their Dormant Strain Floor. Rock Salt is expended to flush calcified marrow and halt permanent biological decay.
   - **The Tinker's Hearth (Bram Karr):** Players spend Scrap Iron to reinforce Cold Steel armor, craft Stagger-enhancing weapon modifications, and drill operatives in martial discipline (0% Strain).
   - **The Dry Hollows (Rest & Bunk Management):** Operatives recover HP and reset combat Strain to their dormant floor. Every night consumes Warded Tallow; running out of fuel extinguishes the protective flame, subjecting sleeping operatives to psychic nightmares.
   - **The Water-Lock Basin (Logistics & Stealth):** Stash management, party member rotation (4-man strike cell chosen from 6–8 bunks), and spending ciphers/bribes to depress the rising **Inquisition Alert Meter**.
3. **The Macro Doom Clock:** Mass cathedral sacrifice (**The Red Tithing**) ticks forward on an overarching calendar. Delaying expeditions or resting excessively brings the execution date closer, while reckless high-profile raids spike Inquisition Alert toward a catastrophic safehouse siege.

---

## 2. Repurposing Existing Engine UI Elements

A cornerstone of our architectural strategy is **maximizing the reuse of the existing Aegis UI framework and Emberwake GUI code**. We repurpose tested engine components rather than building redundant widgets:

```text
+------------------------------------+--------------------------------+-------------------------------------------------------+
| EXISTING ENGINE UI COMPONENT       | SOURCE FILE IN CODEBASE        | REPURPOSED EMBERWAKE SAFEHOUSE FUNCTION               |
+------------------------------------+--------------------------------+-------------------------------------------------------+
| 1. `aegis_ui::window::Desk` & `Win`| `aegis_ui/src/window.rs`,      | **The Safehouse Multi-Wing Workspace Desk:**          |
|    Multi-window workspace layouts  | `emberwake_gui/src/dojo.rs`    | Smooth switching between Slab, Forge, Bunks, & Stash. |
+------------------------------------+--------------------------------+-------------------------------------------------------+
| 2. `Character` Popup & Tabs        | `emberwake_gui/src/party.rs`   | **Active Maneuver Deck & Hero Dossier:**              |
|    Stats, Level-Up, Skills, Gear   |                                | Replaces generic skill lists with 6-card deck slots.  |
+------------------------------------+--------------------------------+-------------------------------------------------------+
| 3. `shop::Trade` 3-Column Widget   | `emberwake_gui/src/shop.rs`    | **Surgical Slab & Forge Transaction Interfaces:**     |
|    Offers, Trade Line, Party Stash |                                | Column 1: Organs/Blueprints; 2: Cost; 3: Stash.       |
+------------------------------------+--------------------------------+-------------------------------------------------------+
| 4. `hud::Hud` Resource Bars        | `emberwake_gui/src/hud.rs`     | **Safehouse Status & Economy Ticker:**                |
|    Gold, Supplies, Place Labels    |                                | Tallow Fuel, Rock Salt, Scrap Iron, Alert Meter.      |
+------------------------------------+--------------------------------+-------------------------------------------------------+
| 5. `hub::Hub` Menu Doors           | `emberwake_gui/src/hub.rs`     | **Undercroft Navigation Doorways:**                   |
|    Party, Journal, Save, Settings  |                                | Traversal between the four wings of the safehouse.    |
+------------------------------------+--------------------------------+-------------------------------------------------------+
| 6. `aegis_ui::timeline::Timeline`  | `aegis_ui/src/timeline.rs`     | **Dojo Drill Timeline Scrubbing:**                    |
|    Ruler, track rows, playhead     |                                | Testing deck maneuver timing before deploying.        |
+------------------------------------+--------------------------------+-------------------------------------------------------+
```

---

## 3. The Four Safehouse Wings: System Breakdown & UI Designs

### 3.1 Wing 1: The Apothecary's Slab (Sister Aline Vance)
* **Narrative Concept:** A dimly lit brick cistern converted into an illegal trauma surgery and dissection theatre. Sister Aline Vance works amidst bone-saws, sulfur pans, and warded tallow candles.
* **Core Systems:**
  1. **Anatomical Graft Installation (Phase 2+):** Consumes harvested aberrant organs (e.g. *Stalker Optic Nerve*, *Titan Marrow Core*) to permanently install Inoculated Branch B skills into an operative.
  2. **Dormant Strain Floor Ratchet:** Each installed graft increases the operative’s permanent Dormant Strain Floor (e.g. $+10\%$ or $+30\%$).
  3. **Trauma Triage & Salt Flushes:** Spends **Rock Salt** to flush crystallized marrow from an operative's blood, stabilizing Stage IV heroes and preventing them from tipping into terminal Stage V Calcification.

#### UI Implementation & Repurposed Components:
* **Repurposed:** Reuses the 3-column layout from `emberwake_gui::shop::Trade` (`SHOP_W = 600, SHOP_H = 300`):
  * *Column 1 (Available Surgeries):* Harvested organs in stash eligible for grafting.
  * *Column 2 (The Surgical Commitment):* Details the operative selected, the anatomical slot, the biological price (+Strain Floor, +Calcification Risk), and the Salt cost.
  * *Column 3 (The Anatomical Wireframe):* Displays the patient's anatomical silhouette.

```text
+---------------------------------------------------------------------------------------------------+
| THE APOTHECARY'S SLAB (SISTER ALINE VANCE)                                           [ESC: CLOSE] |
+-----------------------------------+-------------------------------+-------------------------------+
| HARVESTED REAGENTS (STASH)        | SURGICAL COMMITMENT           | ANATOMICAL SILHOUETTE (KAELEN)|
+-----------------------------------+-------------------------------+-------------------------------+
| [x1] Stalker Optic Nerve (Range)  | Target: Kaelen (The Vanguard) |          [ HEAD ]             |
| [x2] Titan Marrow Core (Bulk)     | Graft: Marrow Carapace        |             │                 |
| [x1] Caustic Bile Sac (Acid)      | Cost:                         |      [x] [ TORSO ] [ ]        |
|                                   |  • 1x Titan Marrow Core       |    L.Arm    │    R.Arm        |
| FLUSH REMEDIES                    |  • 15 Rock Salt               | (Spurred)   │                 |
| [x4] Sanctified Rock Salt         | Consequences:                 |          [ LEGS ]             |
| [x2] Brimstone Tallow Draught     |  • Strain Floor: 0% -> +15%   |                               |
|                                   |  • Adds: "Ribcage Lock" skill | Dormant Strain Floor: [ 15% ] |
|                                   |  • Pain Trauma: -10 HP        | Calcification Stage:  [ II  ] |
|                                   |                               | Salt Flush Status:    [STABLE]|
|                                   | [ EXECUTE SURGERY ]           | [ ADMINISTER SALT FLUSH ]     |
+-----------------------------------+-------------------------------+-------------------------------+
```

---

### 3.2 Wing 2: The Tinker's Hearth (Bram Karr)
* **Narrative Concept:** A dry storm-drain chamber fitted with a charcoal forge, anvils, and racks of scavenged garrison weapons. Bram Karr pounds dented iron breastplates and tunes heavy winch arbalests.
* **Core Systems:**
  1. **Cold Steel Equipment Crafting & Forging:** Spends **Scrap Iron** to forge Tier 1–3 masterwork weapons (billhooks, bodkin daggers, heavy lever-arbalests, iron knuckles).
  2. **Armor Reinforcement:** Upgrades armor pieces to increase **Base Armor Rating** and expand the operative's **Max Poise Bar** ($+10$ to $+25$ Poise).
  3. **Stagger Tuning (Maneuver Modification):** Adjusts weapon weight balances to increase maneuver Stagger damage at the cost of slight Speed Tier shifts.
  4. **Martial Drills:** Conducts training drills to unlock Cold Steel Branch A passives without biological risk.

#### UI Implementation & Repurposed Components:
* **Repurposed:** Reuses `emberwake_gui::forge::Forge` and `emberwake_gui::shop::Trade`:
  * *Left Panel:* Crafting and upgrade recipes grouped by weapon families (Swords, Poleaxes, Daggers, Arbalests, Armor).
  * *Center Panel:* Selected equipment inspection showing Attack Power, Armor Rating, Poise bonus, and Speed Tier interactions.
  * *Right Panel:* Scrap Iron inventory and party stash.

---

### 3.3 Wing 3: The Dry Hollows (Rest & Bunk Management)
* **Narrative Concept:** An elevated, vaulted brick sewer gallery lined with 6 heavy timber bunk frames, each draped in rough wool blankets. A solitary brass brazier in the center burns yellowish-green warded tallow to repel subterranean mists.
* **Core Systems:**
  1. **Bunk Capacity Management:** Starts with **6 Bunks** (housing the Six Founders in Phase 1). Expandable to **8 Bunks** in Phase 2 via Bram's forge upgrade (*"Expanded Timber Frames"*).
  2. **Resting & Recovery:** Resting a squad in the safehouse restores HP to 100% and decays in-combat Strain back down to each hero's Dormant Strain Floor.
  3. **Warded Tallow Fuel Upkeep:** Every safehouse rest consumes **1 Vial of Warded Tallow**.
     - *If Tallow > 0:* Heroes rest peacefully; minor stress neuroses heal.
     - *If Tallow = 0 (The Extinguished Flame):* The yellow-green flame dies. Subterranean mists seep into the bunks; resting operatives awaken with **"The Leviathan's Whispers"** (suffering $-2$ starting AP and gaining severe mental afflictions like *Paranoid Twitching* or *Claustrophobia*).
  4. **The Isolation Crypts (Stage IV Quarantine):** Operatives reaching Stage IV Calcification (*The Shakes*) hum with resonant vibration. Leaving them in communal bunks panics other recruits (imposing $-15\%$ max composure). The player must allocate them to sealed brick quarantine crypts.

#### UI Implementation & Repurposed Components:
* **Repurposed:** Reuses the grid layout from `emberwake_gui::party::Character` (member cycling cards) rendered across a dedicated 6-cell bunk grid:

```text
+---------------------------------------------------------------------------------------------------+
| THE DRY HOLLOWS (BUNKS & RECOVERY)                                                    [REST: 1 VI] |
+-----------------------------------+-------------------------------+-------------------------------+
| BUNK 1: KAELEN (VANGUARD)         | BUNK 2: VESPER (STALKER)      | BUNK 3: BRAN (ARBALESTIER)    |
| • Status: Rested (HP 120/120)     | • Status: Rested (HP 85/85)   | • Status: Light Wounds (70/95)|
| • Strain: 15% (Dormant Floor)     | • Strain: 0% (Pure Steel)     | • Strain: 10% (Floor)         |
| • Calcification: Stage II (Chitin)| • Calcification: Stage 0      | • Calcification: Stage I      |
| • State: Communal Bunk            | • State: Communal Bunk        | • State: Communal Bunk        |
+-----------------------------------+-------------------------------+-------------------------------+
| BUNK 4: TORIN (SLAG-BRUTE)        | BUNK 5: MAREN (ACOLYTE)       | BUNK 6: ELYSE (ALCHEMIST)     |
| • Status: Critical (HP 32/135)    | • Status: Rested (HP 90/90)   | • Status: Rested (HP 80/80)   |
| • Strain: 30% (Floor)             | • Strain: 20% (Floor)         | • Strain: 0% (Pure Steel)     |
| • Calcification: Stage IV (Shakes)| • Calcification: Stage I      | • Calcification: Stage 0      |
| • State: [! QUARANTINE CRYPT !]   | • State: Communal Bunk        | • State: Communal Bunk        |
+-----------------------------------+-------------------------------+-------------------------------+
| WARDED TALLOW FUEL: [████████░░] 8 Days Remaining   | INQUISITION ALERT: [████░░░░░░] 38% (Guarded) |
+-----------------------------------------------------+---------------------------------------------+
```

---

### 3.4 Wing 4: The Water-Lock Basin (Logistics & Strike Cells)
* **Narrative Concept:** A submerged iron sluice gate and storm-drain spillway where dead-drop bone ciphers are fished out from the upper city flues. Serves as the launch point for all missions.
* **Core Systems:**
  1. **Strike Cell Assembly (The 4-Man Squad):** The player selects exactly 4 operatives from their roster of 6–8 heroes to form the active deployment cell.
  2. **Active Maneuver Deck Assembly:** For each of the 4 deployed heroes, the player equips their **Active Deck (3 Actives + 2 Passives + 1 Keystone)**.
  3. **Inquisition Alert Meter Management:** Displays the citywide alert level (0%–100%). High alert increases checkpoint density and road encounters. Players can spend intercepted Clerisy ciphers or gold bribes through the drainage grates to lower alert levels.

---

## 4. Brand New UI Elements to Build

To support this rich economy and tactical deck customization, four specialized UI widgets must be built into `emberwake_gui`:

### 4.1 The Active Maneuver Deck Builder Widget
Integrated directly into `emberwake_gui::party::Character` as an upgraded replacement for the legacy `Skills` tab:
* **Visual Card Layout:** Displays 6 tactile card slots:
  - Slots 1–2 (Fixed Baseline): *Move* and *Active Guard*.
  - Slots 3–5 (Class Actives): 3 active maneuver cards equipped from the hero's unlocked pool. Cards are color-coded: **Steel-Gray** for Cold Steel (0% Strain) and **Bruised-Violet** for Inoculated (+Strain).
  - Slots 6–7 (Passive Traits): 2 passive trait cards.
  - Slot 8 (Keystone Perk): 1 hexagonal capstone card.
* **Card Anatomy:** Each card clearly displays:
  - **Speed Tier Badge:** `[SNAP (T2)]`, `[STD (T4)]`, `[HEAVY (T5)]`.
  - **Resource Cost:** AP cost diamond and Strain percentage flame.
  - **Stagger Power:** Direct poise impact badge (`+25 Stagger`).
  - **Armor Interaction:** Shield icon (`Normal`) vs. Broken Shield (`Piercing`).

```text
+---------------------------------------------------------------------------------------------------+
| ACTIVE MANEUVER DECK (KAELEN)                                                        [AUTO-FILL]  |
+-------------------+-------------------+-------------------+-------------------+-------------------+
| FIXED BASELINE    | ACTIVE SLOT 1     | ACTIVE SLOT 2     | ACTIVE SLOT 3     | KEYSTONE PERK     |
| [ ACTIVE GUARD ]  | [ POMMEL STRIKE ] | [ S門DER & PIN ]  | [ BILE SIPHON ]   | [UNBROKEN PHALANX]|
| • Speed: Snap T2  | • Speed: Snap T2  | • Speed: Heavy T4 | • Speed: Std T4   | • Discipline:     |
| • Cost: 2 AP | 0% | • Cost: 2 AP | 0% | • Cost: 4 AP | 0% | • Cost: 2 AP|+25% |   Cold Steel      |
| • 50% Mitigation  | • Stagger: +25    | • Stagger: +35    | • Corrosive Life  | • Active Guard    |
| • Double Poise    | • Interrupts Wind | • Shreds 4 Armor  | • Bypasses Armor  |   cannot shatter  |
| [Branch: Steel]   | [Branch: Steel]   | [Branch: Steel]   | [Branch: Inoc.]   | [Branch: Steel]   |
+-------------------+-------------------+-------------------+-------------------+-------------------+
| PASSIVE 1: [ BRACED POISE ] (+20 Poise, Unyielding)       | PASSIVE 2: [ MARROW CHITIN ] (+2 Armor, Floor +10%)|
+-----------------------------------------------------------+---------------------------------------+
```

### 4.2 The Anatomical Wireframe & Graft Inspection Widget
A dedicated biological diagram widget added to Sister Aline's slab screen:
* Renders a human silhouette divided into 4 graft regions: **Cranial/Ocular**, **Thoracic/Ribs**, **Forearms/Spurs**, and **Visceral/Organs**.
* Empty slots appear as faint dashed outlines; grafted slots pulse with violet vein textures.
* Clicking an installed graft reveals its harvested organ source, the Strain Floor penalty it imparts, and an option for surgical excision (which inflicts severe HP trauma).

### 4.3 The Warded Tallow Brazier & Doom Clock HUD
An ambient status header widget added to `emberwake_gui::hud`:
* **The Warded Tallow Brazier:** An illustrated brass censer in the HUD corner showing a flickering yellow-green flame. A radial liquid meter displays days of tallow remaining. If reserves hit 0, the flame snuffs out, turning the UI border into a cold, ashen slate gray.
* **The Red Tithing Doom Clock:** A gothic astronomical clock displaying the days remaining before the mass cathedral culling. Every mission and safehouse rest advances the hand.
* **The Inquisition Alert Meter:** A tiered bar showing 0% to 100%:
  - *0%–30% (Guarded):* Standard curfew patrols.
  - *31%–70% (Suspicious):* Armored checkposts at district gates; Arbalestiers posted on rooftops.
  - *71%–99% (Lockdown):* Street hunter packs with Render-Beasts; active lantern detection sweeps.
  - *100% (Breached):* **Safehouse Siege Incursion!** Hostile squads breach the Sluice Undercroft storm-gates.

---

## 5. Safehouse Resource Economy & Conversion Matrix

The macro-economy relies on four tightly balanced resources:

```text
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| RESOURCE NAME         | PRIMARY INFLOW SOURCE       | PRIMARY CONSUMPTION SINK      | TACTICAL / STRATEGIC PURPOSE     |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| 1. Scrap Iron         | Defeated Clerisy/Inquisitors| Bram's Tinker Forge           | Weapon crafting, armor plating,  |
|                       | and foundry demolition nodes|                               | and Poise Bar reinforcement.     |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| 2. Warded Tallow      | Tallow Guild shipments,     | Dry Hollows safehouse upkeep  | Prevents nightmare neuroses and  |
|                       | Inquisitor vials, vats      | (1 vial per rest)             | maintains safehouse wards.       |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| 3. Rock Salt          | Mortuary carts, cathedral   | Sister Aline's Apothecary Slab| Halts Calcification advancement  |
|                       | salt cellars, bone-mills    |                               | and stabilizes Stage IV heroes.  |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| 4. Aberrant Marrow    | Slain Aberrations, elite    | Sister Aline's Apothecary Slab| Unlocks devastating Inoculated   |
|    Organs / Grafts    | Inquisitor officer autopsies| (Surgical graft installations)| Branch B skills on operatives.   |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
```

---

## 6. Engine State Data Binding

These systems interface directly with `emberwake_world::state`:

```rust
// In emberwake_world::state:

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafehouseState {
    /// Remaining warded tallow fuel in days/rests.
    pub warded_tallow_vials: i32,
    /// Rock salt inventory for calcification flushes.
    pub rock_salt_units: i32,
    /// Scrap iron for Bram's forge.
    pub scrap_iron: i32,
    /// Current bunk capacity (6 in Phase 1, 8 in Phase 2).
    pub bunk_capacity: usize,
    /// Citywide Inquisition Alert meter (0 to 100).
    pub inquisition_alert: i32,
    /// Red Tithing Doom Clock counter (days remaining).
    pub doom_clock_days: i32,
    /// List of heroes placed in quarantine crypts.
    pub quarantined_heroes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveDeck {
    pub active_maneuvers: [String; 3],
    pub passive_traits: [String; 2],
    pub keystone_perk: Option<String>,
}
```
