# Roster Architecture: Lifecycle, Expansion, and Safehouse Capacity

> **Category:** Game Systems Design & Roster Management  
> **Location:** `games/emberwake/lore/systems/roster_expansion_and_lifecycle.md`  
> **Reference Lore:** [cosmology_and_setting.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/cosmology_and_setting.md), [risk_reward_and_corruption_balance.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/risk_reward_and_corruption_balance.md)

---

## 1. Roster Philosophy: An Asymmetric Insurgent Cell

Unlike high-fantasy tactical games where the player hires infinite mercenaries from tavern boards, *Emberwake* enforces a tense, thematic lifecycle for party management:
- The resistance operates out of a cramped, submerged storm-drain (**The Sluice Undercroft**).
- Humanity is physically divided by age: adults (ages 25+) are docile and addicted to church draughts, leaving only the youth (ages 15–24, **The Unsealed**) physically capable of resistance.
- Roster management is an active game loop of **liberation, physical transformation, triage, and eventual loss**.

---

## 2. The Starting Roster

When launching a new campaign, the player starts with the core **Severed Root Infiltration Cell** (4 named operatives matching the engine's `PARTY_MAX = 4` limit):

```text
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| HERO NAME             | CLASS ARCHETYPE             | NARRATIVE BACKGROUND               | INITIAL TAINT & LOADOUT  |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 1. Kaelen             | The Vanguard (Warrior)      | Garrison infantry deserter.        | Scavenged plate, billhook|
|                       |                             | Fled after refusing culling orders.| Jagged arm bone spurs.   |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 2. Vesper             | The Gutter-Stalker (Rogue)  | Curfew courier and rooftop scout.  | Leather jerkin, bodkins. |
|                       |                             | Operates via blind-spot paths.     | Shadow-dissolving fingers|
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 3. Maren              | The Heretic Acolyte (Priest)| Defrocked church embalmer/scribe.  | Butcher apron, bone-saw. |
|                       |                             | Stole forbidden dissection codices.| Pulsing violet throat.   |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 4. Bran               | The Arbalestier (Ranger)    | Clocktower tollhouse poacher.      | Horn pauldron, arbalest. |
|                       |                             | Slew an Inquisitor castellan.      | Grafted night-stalker eye|
+-----------------------+-----------------------------+------------------------------------+--------------------------+
```

### Squad Deployment vs. Safehouse Roster
* **Active Mission Squad:** Exactly **3 or 4 operatives** take the field per tactical encounter (the "Rule of Three" cell structure prevents triggering citywide alarm bells).
* **Safehouse Reserve:** Un-deployed operatives rest in the **Dry Hollows** to heal wounds, clear combat fatigue, or vent residual mental Strain.

---

## 3. Roster Expansion (How New Recruits Join)

Players cannot simply buy recruits from commercial menus. Roster additions are earned through active insurgent operations:

```text
                                [ ROSTER RECRUITMENT PATHS ]
                                              |
      +---------------------------------------+---------------------------------------+
      |                                       |                                       |
      v                                       v                                       v
[ CULLING WAGON JAILBREAKS ]        [ GARRISON & GUILD DEFECTORS ]          [ BLACK-MARKET EXTRACTIONS ]
Intercept Inquisitor prison convoys  Rescue rogue scribes, deserters,        Bribe corrupt wardens with salt
before the Red Tithing sacrifices.   or escaped bone-mill laborers.          and tallow to release prisoners.
```

### 3.1 Recruitment Mechanisms
1. **Culling Wagon & Holding Pen Interceptions:**
   - The Inquisition regularly rounds up adolescents in street sweeps, holding them in stockades (such as the Seventh Ward Toll-Pen) before transport to the cathedral crypts.
   - Successful jailbreak missions reward the player with **1 liberated adolescent recruit** from a random class archetype.
2. **Defector Sympathizers & Sewer Runaways:**
   - Disillusioned garrison watchmen (Vanguards), runaway chimney sweeps (Gutter-Stalkers), or escaped bone-mill bruisers (Slag-Brutes) arrive at the **Water-Lock** dead-drop following coded bone-cipher contacts.
3. **Black-Market Ransom (Pip's Runner Network):**
   - Unlocked via Tessia Morn's quest. Allows the player to spend scarce **Salt Rations** and **Warded Tallow** ledgers to smuggle marked youths out of municipal workhouses.

---

## 4. Roster Contraction (How Operatives Are Lost)

Operatives leave the roster through four distinct systemic pathways:

```text
                                [ ROSTER CONTRACTION PATHS ]
                                              |
      +--------------------+------------------+------------------+--------------------+
      |                    |                                     |                    |
      v                    v                                     v                    v
[ TERMINAL DRIFT ]   [ LETHAL RUPTURE ]                    [ RETIREMENT TRIAGE ] [ SAFE-DEPOSIT LOSS ]
Stage V Calcification Aberration Crisis at 100%             Voluntary retirement  Safehouse outpost raid
ends human ego.       Strain inflicts mortal hemorrhage.    to the deep crypts.   forces hard evacuations.
```

### 4.1 Contraction Mechanisms
1. **The Permadeath Clock (Stage V Calcification):**
   - When an operative reaches Calcification Level V, their biological integrity collapses into **Terminal Drift**.
   - The hero is permanently removed from the roster. Depending on their corruption source:
     - They wander into the deep subterranean catacombs to spend their remaining days as a petrified elder.
     - OR their graft violently overtakes their corpse, transforming them into an aberrant boss encounter that the safehouse must neutralize.
2. **Lethal Battlefield Rupture:**
   - If an operative reaches 100% Strain during an encounter and rolls a catastrophic **Biological Collapse** while at low HP, they suffer unpreventable internal hemorrhage and bleed out before extraction.
3. **Honorable Safehouse Retirement:**
   - When an operative accumulates high Calcification (e.g. Stage IV "The Shakes") and becomes too volatile for field missions, the player can choose to honorably retire them to the Undercroft's support staff, freeing up a bunk while granting a permanent passive bonus (e.g., +5% forge speed at Bram's hearth).
4. **Surgical Triage Sacrifice:**
   - Sister Aline Vance can reverse 1 stage of Calcification at the Apothecary's Slab, but this procedure permanently destroys **10% of the character's Max HP**. Repeated surgeries can leave a hero with too little health to survive frontline combat.

---

## 5. Roster Constraints & Limitations

To maintain tension and prevent hoarding armies of high-tier units, the system enforces four hard architectural constraints:

### 5.1 Safehouse Bunk Capacity (The Dry Hollows)
* The Sluice Undercroft is a hidden sewer hideout, not a fortress.
* **Bunk Limit:** The safehouse starts with **6 bunks** (4 active squad + 2 reserves).
* **Expansion:** Upgradable to a hard ceiling of **8 bunks** through engineering upgrades at Bram's Tinker Hearth (requiring salvaged timber and charcoal flues).
* If all bunks are full, the player must dismiss or retire an existing operative before liberating a new recruit.

### 5.2 The Biological Age Ceiling (The 15–24 Rule)
* Every recruit in the game **must be between ages 15 and 24 ("The Unsealed")**.
* Adults aged 25+ (**The Spent Generation**) cannot be recruited into the combat roster. Their vascular systems are calcified, their reflexes are dulled by tallow fumes, and they harbor compulsive religious obedience to the Clerisy. Older NPCs only function as safehouse specialists (Bram Karr, Sister Aline Vance).

### 5.3 The Upkeep Tax (Warded Tallow & Salt)
* Every operative resting in the safehouse consumes **1 unit of Warded Tallow and Salt per campaign travel leg**.
* If the safehouse tallow supply reaches zero:
  - Ambient subterranean mist seeps into the Dry Hollows bunks.
  - All idle reserve operatives suffer **+10% baseline Strain** and risk developing paranoia or nightmares.

### 5.4 Psychological Quarantine (The Stage IV Isolation Rule)
* Operatives at **Stage IV Calcification ("The Shakes")** or with baseline Strain floors $\ge 50\%$ emit erratic aberrant psychic resonance.
* They cannot sleep in the communal bunks without inflicting Composure penalties on adjacent recruits; they must be placed in an isolated **Quarantine Kiln Cell** in the Dry Hollows, taking up a dedicated bunk slot until triaged.
