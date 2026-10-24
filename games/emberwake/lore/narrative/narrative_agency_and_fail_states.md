# Narrative Agency, Roster Continuity & Fail States

> **Category:** Narrative Design & Game Systems Architecture  
> **Location:** `games/emberwake/lore/narrative/narrative_agency_and_fail_states.md`  
> **Reference Lore:** [roster_expansion_and_lifecycle.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/roster_expansion_and_lifecycle.md), [npcs_and_questlines.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/npcs_and_questlines.md)

---

## 1. Character Narrative Identity: Authored Lore vs. Self-Insert

A critical question in tactical RPGs with permadeath is: **Are the playable characters authored narrative figures with pre-written stakes, or blank-slate self-inserts?**

*Emberwake* adopts a **"Resilient Ensemble"** narrative architecture:

```text
                                [ THE SEVERED ROOT ]
                                          |
            +-----------------------------+-----------------------------+
            |                                                           |
            v                                                           v
[ THE FOUNDING CELL (AUTHORED) ]                             [ RESCUED RECRUITS (HYBRID POOL) ]
- Kaelen, Vesper, Maren, Bran                                - Liberated from culling wagons & workhouses
- Personal backstories, faction ties,                        - Pre-authored traits, unique background tags,
  and dedicated questlines.                                    and reactive dialogue barks.
- If one dies, the cause survives!                           - Step into leadership roles when veterans fall.
```

### 1.1 The Founding Cell (Kaelen, Vesper, Maren, Bran)
* **Authored World Ties:** The initial four operatives are deeply woven into the world's power struggles:
  - **Kaelen (Vanguard):** Deserter from the Castellan Watch; holds personal guilt over enforcing juvenile curfews; hunting his former commanding officer.
  - **Vesper (Gutter-Stalker):** Born in the Ash-Crawl tenements; knows the hidden courier dead-drops and drainage locks of the city.
  - **Maren (Heretic Acolyte):** Defrocked cathedral scribe; witnessed the horrific rituals of the Hall of Holy Dissection; seeks to expose the truth of the communion draughts.
  - **Bran (Arbalestier):** Tollhouse poacher who shot an inquisitor castellan to protect his family; master of high-elevation sightlines.
* **Dialogue & Interaction:** They have distinct personal voices, reactive barks during combat, and unique dialogue options when interacting with Safehouse NPCs (Sister Aline, Bram Karr, Tessia Morn).

### 1.2 The Replacement Recruits (The Unsealed Pool)
* If an authored founder dies or succumbs to Stage V Calcification, the game does **NOT** replace them with generic nameless mutes.
* Replacement recruits are drawn from an authored pool of **Unsealed youths (ages 15–24)**, each generated with:
  - A unique personal name and visual silhouette.
  - An **Origin Tag** (e.g., *Escaped Kiln Stoker*, *Defrocked Acolyte*, *Gutter Orphan*, *Smuggler's Apprentice*).
  - A **Temperament Trait** (e.g., *Grim Fatalist*, *Cold Pragmatist*, *Vengeful Zealot*) that dictates their in-combat dialogue and banter at the safehouse fire.
* Any recruit can be promoted to squad leader and carry forward the rebellion's legacy.

---

## 2. Roster Reserves: Fixed vs. Infinite Expansion

To prevent unwinnable campaign soft-locks while preserving scarcity, recruitment is structured into **Authored Story Unlocks** and **Repeatable Insurgency Operations**:

```text
                         [ RECRUITMENT REPOSITORY ]
                                     |
         +---------------------------+---------------------------+
         |                                                       |
         v                                                       v
[ UNIQUE QUEST RECRUITS ]                               [ REPEATABLE CULLING RAIDS ]
- Story-driven named characters                         - Procedural "Culling Wagon" intercepts
- Liberated during major milestone quests                - Always available as an emergency valve
- High starting skills / unique perks                   - Ensures the bench never permanently empties
```

### 2.1 The Anti-Softlock Guarantee
* **The player can NEVER be permanently locked out of reserves.**
* If the player's bench is completely depleted due to catastrophic battle losses or Calcification purges, the safehouse courier (**Tessia Morn**) will always offer a repeatable emergency contract:
  - **"Intercept Seventh Ward Culling Wagon"** or **"Storm Canal Stockade"**.
  - Completing this mission liberates 1–2 new Unsealed recruits to replenish the cell.

### 2.2 Why Players Cannot Abuse / Farm Infinite Recruits
The game prevents recruit hoarding through three systemic governors:
1. **Safehouse Bunk Cap:** The Dry Hollows holds a maximum of **6 bunks** (expandable to **8** via Bram's forge projects). Once bunks are full, no further recruits can be brought into the safehouse.
2. **Upkeep Tax:** Every resting operative consumes **1 unit of Warded Tallow and Salt** per travel leg. Maintaining a full bench of 8 heroes requires a steady supply of fuel; running dry inflicts Strain and panic on resting units.
3. **Inquisition Alert Meter (City "Heat"):** Launching frequent, loud street raids to rescue extra recruits spikes the district's Alert Meter, causing the Inquisition to deploy elite Castellan Wardens and lock down travel routes.

---

## 3. Fail States & Game Over Conditions

### 3.1 The Golden Rule: No Single Character Death Triggers Game Over
* **Losing any specific hero (even Kaelen, Vesper, Maren, or Bran) NEVER triggers a Game Over.**
* If a single character's death caused a hard fail state, players would treat them as fragile VIPs, savescumming every death and refusing to use high-risk Inoculation abilities or heretical marrow grafts.
* In *Emberwake*, **the cause is greater than any individual**. When an operative dies or calcifies, their equipment drops into the safehouse stash, their death is recorded on the Memorial Stone in the Dry Hollows, and surviving cell members gain temporary Resolve or grief traits.

---

### 3.2 When DOES Game Over Occur?

A true Campaign Defeat occurs under only three specific systemic conditions:

```text
                                [ TRUE GAME OVER CONDITIONS ]
                                              |
      +---------------------------------------+---------------------------------------+
      |                                       |                                       |
      v                                       v                                       v
[ 1. TOTAL CELL ANNIHILATION ]          [ 2. SAFEHOUSE BREACH / SIEGE ]         [ 3. THE RED TITHING DOOM CLOCK ]
Active squad wipes in battle,           Inquisition Alert Meter hits 100%,      Player fails to disrupt cathedral
AND zero reserve operatives             and the Sluice Undercroft safehouse      sacraments before the global bell
remain in the safehouse bunks.          defense encounter is lost.              chime countdown triggers the titan.
```

#### Condition 1: Total Cell Annihilation (Zero Living Operatives)
* If all 4 active operatives in a combat encounter are defeated, AND the safehouse reserve bunks are completely empty ($0$ living units in the roster).
* **The Comeback Mechanic:** If the active squad wipes in battle, but the player has **even 1 recruit resting back at the safehouse**, the campaign continues! The surviving reserve must conduct a desperate low-tier reconnaissance or rescue mission to rebuild the cell, creating immense emergent drama.

#### Condition 2: Safehouse Breach (The Fall of the Sluice)
* If the player acts recklessly in the overworld (failing stealth checks, leaving living witnesses, raiding too frequently), the **Inquisition Alert Meter** peaks at 100%.
* An elite Inquisitorial strike force discovers the drowned drainage grate and launches an all-out assault on **The Sluice Undercroft**.
* The player must defend the hub in a multi-stage siege battle (fighting alongside Sister Aline and Bram Karr). If the safehouse central basin falls, the insurrection is crushed (**Game Over**).

#### Condition 3: The Red Tithing Doom Clock (The Global Countdown)
* The overarching campaign objective is to prevent the **Red Tithing**—a catastrophic ritual where the Sovereign Clerisy plans to sacrifice thousands of youth to pacify the slumbering titan.
* The campaign advances along an authored world clock. If the player wastes excessive days resting without striking cathedral supply nodes, the Red Tithing bells toll, awakening the Low Sleep and dragging the continent into the black tar-sea (**Campaign Loss**).
