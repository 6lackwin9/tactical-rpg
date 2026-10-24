# Classes & Skill Trees: The Six Martial & Heretical Archetypes

> **Category:** Gameplay Systems & Narrative-Combat Parity  
> **Status:** Active Canonical Specification  
> **Location:** `games/emberwake/lore/systems/classes_and_skill_trees.md`  
> **Reference Lore:** [master_specification.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md), [combat_and_timeline.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/combat_and_timeline.md), [magic_and_corruption.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/magic_and_corruption.md), [skill_acquisition_and_loadout_systems.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/skill_acquisition_and_loadout_systems.md)  
> **Decision Reference:** [DEC-LORE-04 (Timeline)](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/worldbuilding_decisions.md#L43-L46), [DEC-LORE-11 (Drop Stances)](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/worldbuilding_decisions.md#L77-L80)

---

## 1. The Archetype Adaptation & Operational Matrix

The six foundational engine classes from the Aegis RPG kit are mapped into visceral, setting-grounded insurgent roles adapted to the dark feudal reality of Cinderreach:

```text
ENGINE CLASS      LORE ARCHETYPE                  ROLE & SPECIALIZATION
[ Warrior ]  ===> The Vanguard (Garrison Deserter)       [ Tank / Posture Breaker / Chokepoint Control ]
[ Rogue ]    ===> The Gutter-Stalker (Curfew Runner)     [ Scout / Assassin / Umbral Phase ]
[ Ranger ]   ===> The Arbalestier (Tollhouse Poacher)     [ Ranged Interrupter / Pin / Volley Denial ]
[ Brawler ]  ===> The Slag-Brute (Bone-Mill Laborer)      [ Brawler / Stagger Engine / Line Breaker ]
[ Priest ]   ===> The Heretic Acolyte (Defrocked Scribe)  [ Field Surgery / Strain Siphon / Flesh Weaver ]
[ Mage ]     ===> The Ichor Alchemist (Hedge Occultist)   [ Volatile Chem / Area Blight / Mutagen Flasks ]
```

### 1.1 The Dual-Branch Progression Paradigm
Every class provides two mechanically and philosophically opposed progression branches:
* **Branch A: Cold Steel (The Iron Discipline):** Pure human biomechanics, leverage, stance discipline, and heavy forged steel. Features high AP efficiency, zero Strain generation (0%), zero permanent Calcification, and mastery over Stagger interrupts and Active Guard.
* **Branch B: The Bitter Inoculation (The Marrow Graft):** Surgical organ harvesting and heretical titan marrow implantation. Features low AP costs, explosive burst/true damage, complete armor penetration, in-combat Strain generation (+15%–35%), elevated Dormant Strain Floors, and acceleration of the permanent Calcification clock.

### 1.2 Timeline Integration & Standardized Speed Tiers
Legacy stances (`Neutral`, `Offensive`, `Defensive`, `Tactical`) have been retired ([DEC-LORE-11](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/worldbuilding_decisions.md#L77-L80)). All maneuvers execute across the **6-Tick Simultaneous Timeline** using standardized speed tiers:
* **Snap (Fast):** Windup: None / T1 | **Impact: Tick 2** | Recovery: Ticks 3–4. Used for quick parries, interrupts, evasions, and blinding throws.
* **Standard (Mid):** Windup: Ticks 1–2 | **Impact: Tick 4** | Recovery: Ticks 5–6. Standard swings, thrusts, aimed arbalest bolts, and field bandages.
* **Heavy (Slow):** Windup: Ticks 1–4 | **Impact: Tick 5 or 6** | Recovery: Tick 6 / Recoil. Crushing two-handed blows, channeled marrow blights, and catastrophic terrain destruction. Highly vulnerable to interrupts during windup.

### 1.3 Active Deck Loadout Constraints
An operative equips an **Active Maneuver Deck** before deploying:
* **2 Baseline Moves:** *Move* (Speed-based tiles/tick) + *Active Guard* (Timeline mitigation & Poise doubling).
* **3 Active Maneuvers:** Chosen freely from unlocked Tier 1–3 active skills (can mix Cold Steel and Inoculated).
* **2 Passive Traits:** Chosen from unlocked Tier 1–2 passives.
* **1 Keystone Perk:** Chosen from unlocked capstone perks.

---

## 2. Class 1: The Vanguard (Warrior)
* **Narrative Identity:** Former garrison infantry, city watchmen, and castellan deserters clad in dented, scavenged iron plate. Masters of narrow corridors, shield walls, and crushing posture breaks.
* **Baseline Profile:** High HP (120), Heavy Armor compatibility (Base 4), High Poise Bar (60).

```text
THE VANGUARD TREE
+-------------------------------------------------------+-------------------------------------------------------+
| BRANCH A: COLD STEEL (THE IRON DISCIPLINE)            | BRANCH B: INOCULATED (THE MARROW GRAFT)               |
+-------------------------------------------------------+-------------------------------------------------------+
| T1 Active: Pommel Strike (2 AP | Snap | Stagger +25)  | T1 Active: Calcified Spike (1 AP | Snap | Bleed 8)    |
| T1 Passive: Braced Poise (+20 Poise, Unyielding)      | T1 Passive: Marrow Chitin (+10% Floor, +2 Armor)      |
| T2 Active: Intercepting Hook (3 AP | Std | Redirect)  | T2 Active: Bile Siphon (2 AP | Std | Corrosive Life)  |
| T2 Passive: Stance Leverage (Parry refunds 1 AP)      | T2 Passive: Spasm Counter (Auto-strike at 50% Strain) |
| T3 Active: Sunder & Pin (4 AP | Heavy | Shred 4, Root)| T3 Active: Ribcage Lock (1 AP | Heavy | 50% Mitigate) |
| Keystone: Unbroken Phalanx (Guard cannot be shattered)| Keystone: Aberrant Core (-1 AP in Aberrant Edge)      |
+-------------------------------------------------------+-------------------------------------------------------+
```

### Branch A: Cold Steel (The Iron Discipline)
* **T1 Active — Pommel Strike:**
  * *Profile:* 2 AP | 0% Strain | **Snap (Impact T2)** | Reach 1 | Single Target.
  * *Mechanics:* Heavy blunt thrust with the pommel or crossguard. Deals 18 physical blunt damage and **+25 Stagger**. If the target is winding up, their Stagger damage received is doubled (+50 Stagger), frequently triggering an immediate interrupt.
* **T1 Passive — Braced Poise:**
  * *Mechanics:* Maximum Poise increased by **+20**. While executing Active Guard, the Vanguard is totally immune to knockback, pull, and forced movement from standard human enemies.
* **T2 Active — Intercepting Hook:**
  * *Profile:* 3 AP | 0% Strain | **Standard (Impact T4)** | Reach 2 | Target: Ally + Attacker.
  * *Mechanics:* Steps up to 1 tile to place oneself between an ally and an incoming melee attack. Redirects the attack onto the Vanguard's armor and uses a billhook hook to pull the attacker 1 tile closer, breaking their recovery phase.
* **T2 Passive — Kinetic Leverage:**
  * *Mechanics:* Successfully mitigating or parrying an attack during Active Guard refunds **1 AP** on the subsequent round (once per round limit).
* **T3 Active — Sunder & Pin:**
  * *Profile:* 4 AP | 0% Strain | **Heavy (Windup T1–T4, Impact T5, Recoil T6)** | Reach 1 | Single Target.
  * *Mechanics:* Massive downward vertical billhook cleave. Deals 45 heavy physical damage, permanently tears off **4 Armor Rating** from the target, and inflicts **Rooted** for 1 round.
* **Keystone — Unbroken Phalanx:**
  * *Mechanics:* While remaining at 0% Strain, incoming critical hits against the Vanguard are downgraded to regular hits, and Active Guard cannot be broken even by unblockable heavy blows.

### Branch B: Inoculated (The Marrow Graft)
* **T1 Active — Calcified Spike:**
  * *Profile:* 1 AP | +15% Strain | **Snap (Impact T2)** | Reach 1 | Single Target.
  * *Mechanics:* Serrated bone spurs punch violently through the vanguard's forearm flesh. Deals 22 piercing damage, **completely ignores target Armor Rating**, and inflicts **Bleed** (8 damage/round for 2 rounds).
* **T1 Passive — Marrow Chitin:**
  * *Mechanics:* Permanently increases Dormant Strain Floor by **+10%**. Grants **+2 Base Armor Rating**, but consumable healing draughts are 20% less effective due to ossified digestive lining.
* **T2 Active — Bile Siphon:**
  * *Profile:* 2 AP | +25% Strain | **Standard (Impact T4)** | Reach 1 | Single Target.
  * *Mechanics:* Grafted glandular sacs exude caustic enzymes along the blade. Deals 35 corrosive damage, shreds 2 Armor, and heals the Vanguard for **50% of the total damage dealt**.
* **T2 Passive — Spasm Counter:**
  * *Mechanics:* Upon crossing into *Flesh-Tension* (50%+ Strain), instantly deliver an automated bone-whip counter-attack against the last unit that attacked the Vanguard (dealing 20 true damage, once per round).
* **T3 Active — Ribcage Lock:**
  * *Profile:* 1 AP | +35% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Self.
  * *Mechanics:* The operative's internal ribcage expands outward through the breastplate into an osseous carapace. Grants **+50% Mitigation** for 1 round. When struck, shatters outward in a 4-tile radial AoE bone-shrapnel burst dealing 25 damage to all surrounding units.
* **Keystone — Aberrant Core:**
  * *Mechanics:* Baseline Dormant Strain Floor permanently locks at 30%. While fighting in *Aberrant Edge* (75%–99% Strain), all active skills cost **-1 AP** (minimum 1 AP), but every completed battle permanently adds **+1 Calcification Point**.

---

## 3. Class 2: The Gutter-Stalker (Rogue)
* **Narrative Identity:** Rooftop couriers, chimney sweeps, and curfew runners from the Ash-Crawl. Specialized in slipping through quarantine grates, blind-spot assassinations, and shadow-phase infiltration.
* **Baseline Profile:** Moderate HP (85), Light Leather/Rags (Base 1 Armor), High Mobility (5 Move), Low Poise Bar (35), High Evasion (20%).

```text
THE GUTTER-STALKER TREE
+-------------------------------------------------------+-------------------------------------------------------+
| BRANCH A: COLD STEEL (CINDER & SINEW)                 | BRANCH B: INOCULATED (THE UMBRAL GRAFT)               |
+-------------------------------------------------------+-------------------------------------------------------+
| T1 Active: Hamstring Sever (2 AP | Snap | -2 Move)    | T1 Active: Void-Flicker (1 AP | Snap | 3-Tile Teleport)|
| T1 Passive: Prowler's Cadence (0 AP in Rubble/Mist)   | T1 Passive: Viscous Shadow (+10% Floor, +15% Evasion) |
| T2 Active: Ash-Cloud & Fade (3 AP | Snap | 3x3 Smoke) | T2 Active: Flensing Filaments (2 AP | Std | Pull 2)   |
| T2 Passive: Anatomical Flaw (Pierce Staggered foes)   | T2 Passive: Phase-Twitch (Auto-evade in Resonance)    |
| T3 Active: Throat-Carver's Art (4 AP | Std | Execute) | T3 Active: Umbral Dissolution (2 AP | Heavy | Immune) |
| Keystone: Ghost of the Undercroft (Stealth reset)     | Keystone: Predatory Singularity (Crit vents Strain)   |
+-------------------------------------------------------+-------------------------------------------------------+
```

### Branch A: Cold Steel (Cinder & Sinew)
* **T1 Active — Hamstring Sever:**
  * *Profile:* 2 AP | 0% Strain | **Snap (Impact T2)** | Reach 1 | Single Target.
  * *Mechanics:* Low slicing dagger strike. Deals 16 physical damage, lowers target Movement by **-2 tiles** for 2 rounds, and disables enemy reaction attacks. Deals +40% bonus damage from behind.
* **T1 Passive — Prowler's Cadence:**
  * *Mechanics:* Moving through low cover, darkness, mist, or rubble costs 0 bonus AP. Decreases the frontal detection cone of enemies by 2 tiles.
* **T2 Active — Ash-Cloud & Fade:**
  * *Profile:* 3 AP | 0% Strain | **Snap (Impact T2)** | Range 2 | 3x3 Ground AoE.
  * *Mechanics:* Smashes a pulverized chimney-soot flask against the stone. Creates a **3x3 smoke zone** that blocks ranged line of sight for 2 rounds; instantly disengages the stalker 2 tiles into stealth.
* **T2 Passive — Anatomical Flaw:**
  * *Mechanics:* Precision attacks directed against Stunned, Staggered, or Rooted targets completely bypass Armor Rating and gain **+25% Critical Hit Chance**.
* **T3 Active — Throat-Carver's Art:**
  * *Profile:* 4 AP | 0% Strain | **Standard (Windup T1–T3, Impact T4, Recovery T5–T6)** | Reach 1 | Single Target.
  * *Mechanics:* Unforgiving execution strike delivered from the rear arc. Deals 55 piercing damage; instantly executes targets below 35% HP and refunds **2 AP**.
* **Keystone — Ghost of the Undercroft:**
  * *Mechanics:* While remaining at 0% Strain, backstab kills immediately re-enter stealth once per encounter and inflict permanent **Fear** (-20% accuracy) on all adjacent standard enemies.

### Branch B: Inoculated (The Umbral Graft)
* **T1 Active — Void-Flicker:**
  * *Profile:* 1 AP | +15% Strain | **Snap (Impact T2)** | Range 3 | Mobility.
  * *Mechanics:* Operative dissolves their physical mass, instantly teleporting up to 3 tiles to an unoccupied square, ignoring all terrain obstacles, elevation, and enemy engagement zones.
* **T1 Passive — Viscous Shadow:**
  * *Mechanics:* Permanently increases Dormant Strain Floor by **+10%**. Silhouette blurs: gains **+15% Evasion**, but light-based and phosphorus attacks deal +25% damage.
* **T2 Active — Flensing Filaments:**
  * *Profile:* 2 AP | +25% Strain | **Standard (Impact T4)** | Range 4 | Single Target.
  * *Mechanics:* Shadowy tendons burst from fingertips. Pulls the target 2 tiles, inflicts corrosive blight (6 dmg/round for 2 rounds), and applies **Exposed** (-3 Armor Rating).
* **T2 Passive — Phase-Twitch:**
  * *Mechanics:* While in *Acute Resonance* (25%–49% Strain), automatically phase-flicker 1 tile away before an incoming melee strike resolves (once per round limit).
* **T3 Active — Umbral Dissolution:**
  * *Profile:* 2 AP | +35% Strain | **Heavy (Windup T1–T4, Impact T5, Recovery T6)** | Self.
  * *Mechanics:* Dissolves physical form for 1 full round, granting total immunity to physical damage. Re-solidifying on the next turn grants +100% true eldritch damage on the next attack, but inflicts a 15% Max HP recoil on the stalker.
* **Keystone — Predatory Singularity:**
  * *Mechanics:* Baseline Dormant Strain Floor permanently locks at 30%. While in *Aberrant Edge*, critical hits vent **20% Strain** and reset Void-Flicker; finishing a battle in this state incurs **+1 Calcification Point**.

---

## 4. Class 3: The Arbalestier (Ranger)
* **Narrative Identity:** Tollhouse clocktower poachers, timberline sharpshooters, and ridge scouts. Masters of long sightlines, heavy winch arbalests, wire trip-snares, and target suppression.
* **Baseline Profile:** Moderate HP (95), Medium Brigandine (Base 2 Armor), Range 5, Moderate Poise Bar (40).

```text
THE ARBALESTIER TREE
+-------------------------------------------------------+-------------------------------------------------------+
| BRANCH A: COLD STEEL (THE IRON SIGHT)                 | BRANCH B: INOCULATED (THE OCULAR GRAFT)               |
+-------------------------------------------------------+-------------------------------------------------------+
| T1 Active: Notched Quarrel (2 AP | Std | Shred 1, +15)| T1 Active: Ocular Lock (1 AP | Snap | +35% Crit Mark) |
| T1 Passive: High-Ground Traverse (+1 Range on Rubble) | T1 Passive: Predatory Pupil (+10% Floor, Low-Light)   |
| T2 Active: Tripwire Snare (3 AP | Snap | Root 1 Round)| T2 Active: Splinter Chitin Bolt (2 AP | Std | AoE Bleed)|
| T2 Passive: Cover Breaker (Halves cover defense)      | T2 Passive: Reflex Twitch (Auto-interrupt at 25%)     |
| T3 Active: Heavy Winch Sunder (4 AP | Heavy | Knock 2)| T3 Active: Arterial Laser Pulse (2 AP | Heavy | Line 6)|
| Keystone: Dead-Eye Discipline (Kill refunds 2 AP)     | Keystone: Apex Stalker Core (100% Piercing in Edge)   |
+-------------------------------------------------------+-------------------------------------------------------+
```

### Branch A: Cold Steel (The Iron Sight)
* **T1 Active — Notched Quarrel:**
  * *Profile:* 2 AP | 0% Strain | **Standard (Windup T1–T2, Impact T4, Recovery T5–T6)** | Range 4 | Single Target.
  * *Mechanics:* Measured release of a heavy steel quarrel. Deals 24 physical piercing damage and **+15 Stagger**. Ignores 1 point of Armor Rating.
* **T1 Passive — High-Ground Traverse:**
  * *Mechanics:* When firing from elevated platforms, roofs, or rubble tiles, weapon range increases by **+1 tile** and incoming ranged attacks against the Arbalestier suffer -15% hit chance.
* **T2 Active — Tripwire Snare:**
  * *Profile:* 3 AP | 0% Strain | **Snap (Impact T2)** | Range 1 (Adjacent Tile) | Ground Hazard.
  * *Mechanics:* Deploys a concealed coiled wire caltrop trap. The first enemy stepping onto the tile suffers 22 physical damage, loses all remaining movement ticks, and is **Rooted** for 1 round.
* **T2 Passive — Cover Breaker:**
  * *Mechanics:* Reduces target cover defense bonuses by **50%**; attacks directed against winding-up enemies gain **+25% Critical Hit Chance** and +15 Stagger.
* **T3 Active — Heavy Winch Sunder:**
  * *Profile:* 4 AP | 0% Strain | **Heavy (Windup T1–T4, Impact T5, Recovery T6)** | Range 5 | Single Target.
  * *Mechanics:* Massive steel bolt driven by an iron lever-winch. Deals 52 physical damage, knocks the target back 2 tiles, shatters **3 Armor Rating**, and immediately interrupts any active windup.
* **Keystone — Dead-Eye Discipline:**
  * *Mechanics:* While remaining at 0% Strain, killing an enemy with a ranged attack refunds **2 AP** and reveals all enemy planned movement paths for the subsequent round.

### Branch B: Inoculated (The Ocular Graft)
* **T1 Active — Ocular Lock:**
  * *Profile:* 1 AP | +15% Strain | **Snap (Impact T2)** | Range 5 | Single Target.
  * *Mechanics:* Grafted night-stalker pupil pulses crimson. Paints the target: target cannot evade or enter stealth, and all allied attacks against them gain **+30% Critical Hit Chance** for 2 rounds.
* **T1 Passive — Predatory Pupil:**
  * *Mechanics:* Baseline Dormant Strain Floor permanently increases by **+10%**. Ignores all darkness, fog, and smoke targeting penalties; can target through low half-cover without damage reduction.
* **T2 Active — Splintering Chitin Bolt:**
  * *Profile:* 2 AP | +25% Strain | **Standard (Impact T4)** | Range 4 | Impact + Adjacent Plus (+).
  * *Mechanics:* Fires an osseous spine bolt. Upon impact, splinters into jagged bone fragments dealing 28 corrosive damage and **Bleed** (6 dmg/round for 2 rounds) to the target and all orthogonally adjacent units.
* **T2 Passive — Reflex Twitch:**
  * *Mechanics:* In *Acute Resonance* (25%–49% Strain), if targeted by an incoming ranged projectile during windup, automatically fire a free Snap interrupt needle at the attacker (dealing 12 damage and +15 Stagger, once per round).
* **T3 Active — Arterial Laser Pulse:**
  * *Profile:* 2 AP | +35% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Range 6 | 1x6 Line AoE.
  * *Mechanics:* Channels raw eldritch energy through the ocular graft. Emits a continuous 6-tile red beam that pierces through all units in a line, ignoring armor and barriers for 48 true damage.
* **Keystone — Apex Stalker Core:**
  * *Mechanics:* Baseline Strain Floor permanently locks at 30%. While fighting in *Aberrant Edge*, all ranged attacks gain **100% Armor Piercing**, but battle completion in this state inflicts **+1 Calcification Point**.

---

## 5. Class 4: The Slag-Brute (Brawler)
* **Narrative Identity:** Escaped bone-mill laborers, kiln stokers, and foundry breakers. Masters of bare-handed brutality, improvised knuckle-guards, body throws, and structural demolition.
* **Baseline Profile:** Very High HP (135), Medium Scavenged Plates (Base 2 Armor), Very High Poise Bar (70).

```text
THE SLAG-BRUTE TREE
+-------------------------------------------------------+-------------------------------------------------------+
| BRANCH A: COLD STEEL (THE FORGE FOUNDRY)              | BRANCH B: INOCULATED (THE MARROW TITAN)               |
+-------------------------------------------------------+-------------------------------------------------------+
| T1 Active: Jaw-Breaker (2 AP | Snap | Stagger +30)    | T1 Active: Tendril Lash (1 AP | Snap | Pull 2)        |
| T1 Passive: Hypertrophy (+20% Max HP, +1 Knockback)   | T1 Passive: Osseous Bulk (+10% Floor, +2 Armor, -1 Mv)|
| T2 Active: Slag Slam & Throw (3 AP | Std | Hurl 2)    | T2 Active: Boiling Slag Bile (2 AP | Std | Melt Armor)|
| T2 Passive: Anvil Posture (Resist Snap interrupts)    | T2 Passive: Flesh Rampage (AoE shockwave at 50%)      |
| T3 Active: Foundry Shatter (4 AP | Heavy | Stun Cone) | T3 Active: Aberrant Colossus (2 AP | Heavy | Cleave)  |
| Keystone: Unbroken Iron (Low HP grants +4 Armor)      | Keystone: Juggernaut Core (Immortality in Edge)       |
+-------------------------------------------------------+-------------------------------------------------------+
```

### Branch A: Cold Steel (The Forge Foundry)
* **T1 Active — Jaw-Breaker:**
  * *Profile:* 2 AP | 0% Strain | **Snap (Impact T2)** | Reach 1 | Single Target.
  * *Mechanics:* Crushing punch delivered with iron-shackled fists. Deals 20 blunt damage and **+30 Stagger**. Pushes target back 1 tile.
* **T1 Passive — Hypertrophy:**
  * *Mechanics:* Increases Maximum HP by **+20%**; all forced knockback distance inflicted on enemies increases by **+1 tile**.
* **T2 Active — Slag Slam & Throw:**
  * *Profile:* 3 AP | 0% Strain | **Standard (Impact T4)** | Reach 1 | Target: Enemy + Landing Tile.
  * *Mechanics:* Grabs an adjacent enemy and hurls them up to 2 tiles into another enemy or solid wall. Both entities suffer 30 blunt damage and **+25 Stagger**; walls suffer structural demolition.
* **T2 Passive — Anvil Posture:**
  * *Mechanics:* Cannot be interrupted or staggered by enemy Snap attacks; incoming melee blunt damage is reduced by **25%**.
* **T3 Active — Foundry Shatter:**
  * *Profile:* 4 AP | 0% Strain | **Heavy (Windup T1–T4, Impact T5, Recovery T6)** | Reach 1 | 3-Tile Frontal Cone.
  * *Mechanics:* Slams both fists into the flagstones. Destroys terrain tiles in a 3-tile frontal cone, converting them into rubble hazard and knocking down all caught units (**Stunned** on Tick 5).
* **Keystone — Unbroken Iron:**
  * *Mechanics:* While remaining at 0% Strain, dropping below 30% HP immediately refunds **2 AP** and grants **+4 Base Armor Rating** for 2 rounds.

### Branch B: Inoculated (The Marrow Titan)
* **T1 Active — Tendril Lash:**
  * *Profile:* 1 AP | +15% Strain | **Snap (Impact T2)** | Range 2 | Single Target.
  * *Mechanics:* Thick muscle tendrils burst from forearms. Deals 18 piercing damage and violently yanks the target into an adjacent tile, locking them in melee engagement.
* **T1 Passive — Osseous Bulk:**
  * *Mechanics:* Baseline Strain Floor increases by **+10%**. Grants **+2 Base Armor Rating**, but movement speed is permanently reduced by **-1 tile**.
* **T2 Active — Boiling Slag Bile:**
  * *Profile:* 2 AP | +25% Strain | **Standard (Impact T4)** | Range 3 | 1x3 Line AoE.
  * *Mechanics:* Vomits caustic, boiling mutagenic bile in a 3-tile line. Ignites the ground and strips **2 Armor Rating** per round for 2 rounds from caught enemies.
* **T2 Passive — Flesh Rampage:**
  * *Mechanics:* In *Flesh-Tension* (50%+ Strain), every landed melee attack triggers an explosive radial bone shockwave hitting all adjacent foes for 15 damage.
* **T3 Active — Aberrant Colossus:**
  * *Profile:* 2 AP | +35% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Self.
  * *Mechanics:* Biological mass violently doubles for 2 rounds. Grants total immunity to crowd control; melee attacks cleave 3 adjacent tiles for double damage, but unit suffers **15% Max HP recoil damage** per round.
* **Keystone — Juggernaut Core:**
  * *Mechanics:* Baseline Strain Floor permanently locks at 30%. While in *Aberrant Edge*, HP cannot drop below 1 (immortality for 1 round), but battle completion in this state inflicts **+1 Calcification Point**.

---

## 6. Class 5: The Heretic Acolyte (Priest)
* **Narrative Identity:** Defrocked church scribes, apostate embalmers, and cemetery recluses. Masters of battlefield trauma surgery, strain siphoning, and mutagenic flesh weaving.
* **Baseline Profile:** Moderate HP (90), Robes & Butcher Apron (Base 1 Armor), Moderate Poise Bar (40).

```text
THE HERETIC ACOLYTE TREE
+-------------------------------------------------------+-------------------------------------------------------+
| BRANCH A: COLD STEEL (THE FIELD SUTURE)               | BRANCH B: INOCULATED (THE MARROW SIPHON)              |
+-------------------------------------------------------+-------------------------------------------------------+
| T1 Active: Brimstone Cauterize (2 AP | Snap | Cleanse)| T1 Active: Strain Siphon (1 AP | Snap | Drain 30%)     |
| T1 Passive: Anatomical Precision (+15% Stability Aura)| T1 Passive: Sympathetic Marrow (+10% Floor, Guard Bond)|
| T2 Active: Tourniquet & Tonic (3 AP | Std | Heal 75)  | T2 Active: Ichor Thread Suture (2 AP | Std | Heal 120)|
| T2 Passive: Triage Efficiency (Low HP refunds 1 AP)   | T2 Passive: Resonant Catharsis (Aura vent at 50%)     |
| T3 Active: Resilience Ward (4 AP | Heavy | Squad Mit) | T3 Active: Aberrant Communion (2 AP | Heavy | Pool HP)|
| Keystone: Pure Hippocratic Oath (+40% Healing at 0%)  | Keystone: Martyr's Singularity (Absorb fatal blows)   |
+-------------------------------------------------------+-------------------------------------------------------+
```

### Branch A: Cold Steel (The Field Suture)
* **T1 Active — Brimstone Cauterize:**
  * *Profile:* 2 AP | 0% Strain | **Snap (Impact T2)** | Range 2 | Ally.
  * *Mechanics:* Presses hot sulfur-dusted steel to a wound. Cleanses Bleed, Poison, or Burn from an ally within 2 tiles and restores **35 HP**.
* **T1 Passive — Anatomical Precision:**
  * *Mechanics:* Allies standing within 2 tiles of the Acolyte gain **+15% Stagger Resistance** and **+10% Debuff Resistance**.
* **T2 Active — Tourniquet & Tonic:**
  * *Profile:* 3 AP | 0% Strain | **Standard (Impact T4)** | Range 2 | Ally.
  * *Mechanics:* Rushes to an injured ally. Heals **75 HP**, resets interrupted recovery frames to Active Guard, and grants **+1 Movement** for 1 round.
* **T2 Passive — Triage Efficiency:**
  * *Mechanics:* Delivering healing skills to an ally below 30% HP immediately refunds **1 AP** (once per round).
* **T3 Active — Resilience Ward:**
  * *Profile:* 4 AP | 0% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Squad-Wide (Range 4).
  * *Mechanics:* Chants pre-cataclysm liturgical verses while swinging a heavy bronze censer. Grants the entire 4-man squad **+30% Physical Mitigation** and **Stagger Immunity** for 1 round.
* **Keystone — Pure Hippocratic Oath:**
  * *Mechanics:* While remaining at 0% Strain, all field sutures and medicinal draughts heal **40% more HP**; the Acolyte cannot be critically struck by standard human enemies.

### Branch B: Inoculated (The Marrow Siphon)
* **T1 Active — Strain Siphon:**
  * *Profile:* 1 AP | +15% Strain | **Snap (Impact T2)** | Range 3 | Ally.
  * *Mechanics:* Extends blackened veins across an ally. Siphons up to **30% Strain** away from the ally into the Acolyte's own gauge, healing the ally for **50 HP**.
* **T1 Passive — Sympathetic Marrow:**
  * *Mechanics:* Baseline Strain Floor increases by **+10%**. When allies within 3 tiles take damage, the Acolyte absorbs +5% Strain but grants the ally **+2 Armor Rating**.
* **T2 Active — Ichor Thread Suture:**
  * *Profile:* 2 AP | +25% Strain | **Standard (Impact T4)** | Range 3 | Ally.
  * *Mechanics:* Launches living black ichor threads to stitch catastrophic wounds. Heals **120 HP**, but increases target ally's Strain by **+10%**.
* **T2 Passive — Resonant Catharsis:**
  * *Mechanics:* When the Acolyte crosses into *Flesh-Tension* (50%+ Strain), all allies within 3 tiles immediately vent **15% Strain**.
* **T3 Active — Aberrant Communion:**
  * *Profile:* 2 AP | +35% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Squad-Wide.
  * *Mechanics:* Psychically links the circulatory systems of the squad for 1 round. All damage taken by any ally is pooled and halved; single-target heals apply to all squad members simultaneously.
* **Keystone — Martyr's Singularity:**
  * *Mechanics:* Baseline Strain Floor permanently locks at 30%. While in *Aberrant Edge*, if an ally would take fatal damage, the Acolyte absorbs the blow entirely; battle completion in this state inflicts **+1 Calcification Point**.

---

## 7. Class 6: The Ichor Alchemist (Mage)
* **Narrative Identity:** Tenement hedge occultists, illegal tallow distillers, and gutter apothecaries. Masters of phosphorus fire, blinding flash-pots, caustic acid sprays, and subterranean miasma.
* **Baseline Profile:** Moderate HP (80), Leather Apron & Bandolier (Base 1 Armor), Low Poise Bar (30).

```text
THE ICHOR ALCHEMIST TREE
+-------------------------------------------------------+-------------------------------------------------------+
| BRANCH A: COLD STEEL (VOLATILE PYROTECHNICS)          | BRANCH B: INOCULATED (THE MUTAGENIC CATALYST)         |
+-------------------------------------------------------+-------------------------------------------------------+
| T1 Active: Phosphorus Pot (2 AP | Std | 3x3 Fire Zone)| T1 Active: Caustic Vitriol (1 AP | Snap | Melt Armor) |
| T1 Passive: Combustion Formula (+25% vs Oiled/Burning)| T1 Passive: Toxic Blood (+10% Floor, Acid Recoil)     |
| T2 Active: Magnesium Flash-Bomb (3 AP | Snap | Blind) | T2 Active: Miasma of Low Sleep (2 AP | Std | Halluc.) |
| T2 Passive: Catalytic Munitions (-2 Armor on Burning) | T2 Passive: Aberrant Catalysis (Acid heals at 50%)    |
| T3 Active: Suffocating Choke-Gas (4 AP | Heavy | LoS) | T3 Active: Liquefaction Transmute (2 AP | Heavy | Bog)|
| Keystone: Master Alchemist (+1 AoE, -1 AP on Flasks)  | Keystone: Primordial Core (Permanent Gear Dissolution)|
+-------------------------------------------------------+-------------------------------------------------------+
```

### Branch A: Cold Steel (Volatile Pyrotechnics)
* **T1 Active — Phosphorus Pot:**
  * *Profile:* 2 AP | 0% Strain | **Standard (Impact T4)** | Range 4 | 3x3 Ground AoE.
  * *Mechanics:* Throws a volatile burning flask. Creates a **3x3 burning ground zone** for 2 rounds, dealing 16 fire damage and applying Burn (6 dmg/round) to any unit entering or ending turn within it.
* **T1 Passive — Combustion Formula:**
  * *Mechanics:* All thrown chemical items and fire skills deal **+25% bonus damage** against enemies standing in tar, oil, or wooden rubble.
* **T2 Active — Magnesium Flash-Bomb:**
  * *Profile:* 3 AP | 0% Strain | **Snap (Impact T2)** | Range 3 | 3x3 Blast AoE.
  * *Mechanics:* Detonates a blinding flash-powder canister. Inflicts **Blind** (cannot target beyond 1 tile) and **+20 Stagger** on all caught enemies.
* **T2 Passive — Catalytic Munitions:**
  * *Mechanics:* Enemies afflicted by Burning or Blind have their Armor Rating permanently reduced by **-2**.
* **T3 Active — Suffocating Choke-Gas:**
  * *Profile:* 4 AP | 0% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Range 4 | 4x4 Ground AoE.
  * *Mechanics:* Deploys a dense cloud of soot and brimstone. Completely blocks ranged line of sight; enemies trapped inside lose **1 AP per round** and suffer 20 choke damage per round.
* **Keystone — Master Alchemist:**
  * *Mechanics:* While remaining at 0% Strain, all thrown flasks and grenade items gain **+1 tile AoE radius** and cost **-1 AP** (minimum 1 AP).

### Branch B: Inoculated (The Mutagenic Catalyst)
* **T1 Active — Caustic Vitriol:**
  * *Profile:* 1 AP | +15% Strain | **Snap (Impact T2)** | Range 4 | Single Target.
  * *Mechanics:* Hurls raw aberrant digestive enzymes. Ignores armor completely, applies **Exposed (-4 Armor)**, and inflicts corrosive decay (8 dmg/round for 2 rounds).
* **T1 Passive — Toxic Blood:**
  * *Mechanics:* Baseline Strain Floor increases by **+10%**. When struck by melee attacks, the attacker suffers **15 corrosive recoil damage** and is splashed with acid.
* **T2 Active — Miasma of the Low Sleep:**
  * *Profile:* 2 AP | +25% Strain | **Standard (Impact T4)** | Range 4 | 3x3 Ground AoE.
  * *Mechanics:* Vents a swirling violet cloud of subterranean spore-gas. Enemies inside suffer eldritch hallucinations (50% chance to attack nearest ally) and gain **+5% Strain per tick**.
* **T2 Passive — Aberrant Catalysis:**
  * *Mechanics:* In *Flesh-Tension* (50%+ Strain), all corrosive damage dealt by the Alchemist heals her for **25% of damage dealt**.
* **T3 Active — Liquefaction Transmutation:**
  * *Profile:* 2 AP | +35% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Range 4 | 3x3 Ground AoE.
  * *Mechanics:* Channels the dreaming titan's tremors, transmuting stone floor into bubbling tar-bog. Ensnared enemies are **Rooted** and suffer continuous liquefaction acid damage (25 dmg/round).
* **Keystone — Primordial Core:**
  * *Mechanics:* Baseline Strain Floor permanently locks at 30%. While in *Aberrant Edge*, corrosive skills permanently dissolve enemy shields and weapons; battle completion in this state inflicts **+1 Calcification Point**.

---

## 8. Named Character Specialization: The Six Founders

While procedural recruits generated in Phase 2 adhere strictly to the standard base class trees, the **Six Authored Founders** possess unique personal backgrounds, physiological mutations, and signature equipment.

Each Founder features:
1. **Innate Founder Trait:** A permanent character-defining passive that reflects their past and physical condition.
2. **Authored Skill Variant:** A unique, modified version of a base class skill that replaces the standard version in their repertoire.
3. **Founder Signature Maneuver:** An exclusive, powerful active capstone maneuver available *only* to that specific character.

```text
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| FOUNDER NAME          | INNATE FOUNDER TRAIT        | AUTHORED SKILL VARIANT        | EXCLUSIVE SIGNATURE CAPSTONE     |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| 1. Kaelen             | Castellan's Penance         | Notched Billhook Sunder       | Seventh Ward Bulwark             |
| 2. Vesper             | Cinder-Child's Reflex       | Orlov's Retribution           | Umbral Flensing                  |
| 3. Bran               | Clocktower Hawkeye          | Poacher's Barbed Quarrel      | Predator Reticle Pulse           |
| 4. Torin "Slag-Jaw"   | Foundry-Wired Spite         | Shackle-Pummel Slam           | Molten Marrow Eruption           |
| 5. Sister Maren       | Anatomist of St. Agnes      | Black-Ichor Transfusion       | Martyr's Shroud                  |
| 6. Elyse              | Volatile Reaction Catalyst  | Pressurized Vitriol Siphon    | Spore-Bloom of the Sleeper       |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
```

---

### 8.1 Kaelen (The Vanguard Founder)
* **Narrative Lore:** Former sergeant of the Seventh Ward Castellan Guard who deserted after refusing to slaughter a pen of "unsealed" youths during a Red Tithing sweep. His left arm is wrapped in blood-soaked linen rags, concealing jagged, emergent calcified white bone spurs that hum whenever the church bells toll.

#### 1. Innate Trait — *Castellan's Penance*
* *Type:* Innate Passive (Character-Locked).
* *Mechanics:* When an adjacent ally is targeted by an incoming melee or ranged attack, Kaelen's *Active Guard* automatically triggers as an un-queued Snap reaction on Tick 2 without spending AP, redirecting **35% of the incoming damage** onto his own armor.

#### 2. Authored Skill Variant — *Notched Billhook Sunder* (Modifies *Sunder & Pin*)
* *Profile:* 4 AP | 0% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Reach 1 | Single Target.
* *Mechanics:* Kaelen's custom-weighted billhook winds up 1 tick faster than standard (striking on Tick 4 instead of Tick 5). In addition to tearing off 4 Armor Rating and Rooting the target, if the victim belongs to the Sovereign Clerisy or Iron Inquisition, they are afflicted with **Disarmed** (-50% damage dealt for 1 round).

#### 3. Exclusive Signature Capstone — *Seventh Ward Bulwark*
* *Profile:* 3 AP | 0% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Self / 3-Tile Linear Barrier.
* *Mechanics:* Kaelen slams his heavy tower shield into the flagstones, bracing it with his body. Delivers 30 blunt damage and **+35 Stagger** to all foes immediately in front of him. Creates an impassable **Bulwark Zone** across 3 tiles for 2 rounds: allies behind the shield gain **+50% Physical Mitigation** and cannot be targeted by ranged attacks; enemies cannot cross the barrier.

---

### 8.2 Vesper (The Gutter-Stalker Founder)
* **Narrative Lore:** Orphan chimney courier from the Ash-Crawl Tenements whose brother died in the tallow vats. Her fingertips dissolve into an ethereal, drifting black smoke that leaves no fingerprints; her shadow stretches unnaturally toward nearby enemies regardless of lantern placement.

#### 1. Innate Trait — *Cinder-Child's Reflex*
* *Type:* Innate Passive (Character-Locked).
* *Mechanics:* Whenever Vesper successfully evades an attack or breaks line of sight behind cover, she immediately gains **+1 Movement tile** for the following round. If Inoculated, entering stealth automatically vents **10% Strain**.

#### 2. Authored Skill Variant — *Orlov's Retribution* (Modifies *Throat-Carver's Art*)
* *Profile:* 3 AP | 0% Strain | **Standard (Windup T1–T2, Impact T3, Recovery T4–T5)** | Reach 1 | Single Target.
* *Mechanics:* Costs **1 less AP** (3 AP instead of 4) and strikes on Tick 3. If the strike successfully kills the target from behind, Vesper immediately dissolves into a puff of chimney soot, teleporting 2 tiles to an unoccupied shadow tile without spending movement ticks and resetting her stealth.

#### 3. Exclusive Signature Capstone — *Umbral Flensing*
* *Profile:* 2 AP | +30% Strain | **Snap (Windup T1, Impact T2, Recovery T3–T4)** | Range 3 | Line 3.
* *Mechanics:* Vesper extends her smoke-dissolving fingers into razor-sharp umbral filaments across a 3-tile line. Completely bypasses physical armor, deals 38 true eldritch damage, violently pulls all caught targets 1 tile inward toward her, and inflicts **Mutilated** (-2 Movement, 10 bleed dmg/round for 2 rounds).

---

### 8.3 Bran (The Arbalestier Founder)
* **Narrative Lore:** Tollhouse poacher who survived in the clocktower rafters by poaching church messenger birds. His left eye socket holds a grafted night-stalker pupil that pulses with an arterial red light, allowing him to perceive the physical strain and intent of living targets.

#### 1. Innate Trait — *Clocktower Hawkeye*
* *Type:* Innate Passive (Character-Locked).
* *Mechanics:* Suffers zero range or accuracy penalties against targets in partial cover. Furthermore, during the planning phase, any enemy within Bran's line of sight has their upcoming action type (Move, Attack, or Guard) revealed to the player via a tactical glyph.

#### 2. Authored Skill Variant — *Poacher's Barbed Quarrel* (Modifies *Notched Quarrel*)
* *Profile:* 2 AP | 0% Strain | **Standard (Impact T4)** | Range 5 | Single Target.
* *Mechanics:* Fires a heavy bolt fitted with reverse wire-barbs. In addition to 24 piercing damage and +15 Stagger, if the target attempts to move on any subsequent tick during the current round, the barbs rip open: the target takes **18 automatic physical damage** and their movement immediately halts on that tile.

#### 3. Exclusive Signature Capstone — *Predator Reticle Pulse*
* *Profile:* 2 AP | +20% Strain | **Standard (Windup T1–T2, Impact T3, Recovery T4–T5)** | Range 6 | Single Target.
* *Mechanics:* Bran focuses his ocular graft, projecting a blazing arterial targeting laser onto an enemy. The target suffers **Exposed (-4 Armor)** and loses all evasion for 2 rounds; **every allied attack** directed at this target automatically gains **+30% Critical Hit Chance** and +15 Stagger.

---

### 8.4 Torin "Slag-Jaw" (The Slag-Brute Founder)
* **Narrative Lore:** Escaped laborer from the Fourth Kiln bone-mills. His jaw was shattered by molten slag splashback and wired together with scrap iron. He uses his unremoved penal iron wrist shackles as crushing weighted cestuses.

#### 1. Innate Trait — *Foundry-Wired Spite*
* *Type:* Innate Passive (Character-Locked).
* *Mechanics:* While Torin's HP is below 50%, his Stagger Bar cannot be filled and he is totally immune to Stun or interrupts. Taking damage builds **Slag Fury** (+6% melee damage per strike received, stacking up to 5 times until the end of the encounter).

#### 2. Authored Skill Variant — *Shackle-Pummel Slam* (Modifies *Jaw-Breaker*)
* *Profile:* 2 AP | 0% Strain | **Snap (Impact T2)** | Reach 1 | Single Target.
* *Mechanics:* Delivers a two-handed hammer smash using his iron wrist cuffs. Deals 24 blunt damage and **+35 Stagger**. If the target is knocked into a wall or solid obstacle, they take **double damage (48)** and their queued action for the round is instantly shattered into a Stagger Block.

#### 3. Exclusive Signature Capstone — *Molten Marrow Eruption*
* *Profile:* 2 AP | +35% Strain | **Heavy (Windup T1–T3, Impact T4, Recovery T5–T6)** | Self / 2-Tile Radius Radial AoE.
* *Mechanics:* Torin forcefully unhinges his wired slag jaw, expelling a concussive blast of superheated black marrow steam. Deals 35 fire/corrosive damage to all surrounding enemies, pushes them 2 tiles back, strips 3 Armor Rating, and leaves burning slag pools on all adjacent tiles for 2 rounds.

---

### 8.5 Sister Maren (The Heretic Acolyte Founder)
* **Narrative Lore:** Defrocked scribe and mortuary embalmer from St. Agnes Chapel. Stole forbidden anatomical scrolls before escaping. Her throat and neck are laced with pulsing, pitch-black veins that leak dark ichor when she channels strain.

#### 1. Innate Trait — *Anatomist of St. Agnes*
* *Type:* Innate Passive (Character-Locked).
* *Mechanics:* When Maren cleanses an ally of a status condition (Bleed, Poison, Burn, or Corrosive Blight), she harvests the pathogen into a **Bile Phial** in her inventory. This phial can be used as a free Snap action to coat an ally's weapon, adding +8 corrosive damage to their next strike.

#### 2. Authored Skill Variant — *Black-Ichor Transfusion* (Modifies *Ichor Thread Suture*)
* *Profile:* 2 AP | +20% Strain | **Standard (Impact T4)** | Range 3 | Ally Target.
* *Mechanics:* Heals the target ally for **130 HP**. Additionally, Maren can balance Strain dynamically: she can either siphon up to 25% Strain away from the target into herself, or inject up to 25% Strain into an Inoculated ally to intentionally launch them into *Aberrant Edge* (75%+ Strain) for maximum damage output.

#### 3. Exclusive Signature Capstone — *Martyr's Shroud*
* *Profile:* 3 AP | +30% Strain | **Standard (Windup T1–T2, Impact T3, Recovery T4–T6)** | Squad-Wide (Range 4).
* *Mechanics:* Maren slices her palm across her bronze censer, filling the air with incense and vaporized blood. For 2 rounds, **no allied squad member can drop below 1 HP**: any lethal blow is absorbed, and 50% of the fatal damage is transferred to Maren as +15% Strain instead of HP damage.

---

### 8.6 Elyse (The Ichor Alchemist Founder)
* **Narrative Lore:** Tenement distiller who ran a clandestine laboratory brewing counterfeit tallow and neural-stimulants. She carries a wicker bandolier filled with volatile glass retorts, acid canisters, and a pressurized brass sprayer.

#### 1. Innate Trait — *Volatile Reaction Catalyst*
* *Type:* Innate Passive (Character-Locked).
* *Mechanics:* Ground hazard zones created by Elyse's flasks persist for **+1 additional round**. Furthermore, if two distinct hazard zones overlap (e.g., Phosphorus Fire + Caustic Acid), they detonate in a **Catastrophic Chemical Reaction**: a 3x3 blast dealing 30 true damage and knocking down all units.

#### 2. Authored Skill Variant — *Pressurized Vitriol Siphon* (Modifies *Caustic Vitriol*)
* *Profile:* 2 AP | +20% Strain | **Snap (Impact T2)** | Range 3 | 3-Tile Frontal Arc.
* *Mechanics:* Instead of a single thrown flask, Elyse pumps her brass sprayer, releasing a continuous arc of caustic acid. Completely bypasses armor, deals 26 corrosive damage to all targets in the 3-tile arc, strips **4 Armor Rating**, and inflicts **Blind** for 1 round.

#### 3. Exclusive Signature Capstone — *Spore-Bloom of the Sleeper*
* *Profile:* 2 AP | +35% Strain | **Standard (Windup T1–T3, Impact T4, Recovery T5–T6)** | Range 4 | 3x3 Ground AoE.
* *Mechanics:* Elyse shatters a warded glass vessel containing live subterranean marrow spores. Enemies inside suffer **Nightmare Frenzy** (compelled to attack the nearest entity, friend or foe) and accumulate +8% Strain per tick; allies standing in the bloom have their Dormant Strain Floor temporarily reduced by 15% as the spores absorb human pain.

---

## 9. Procedural Recruits vs. Authored Founders Matrix

When Phase 2 unlocks after the Crucible climax, the squad begins recruiting procedurally generated youths liberated from Inquisition culling wagons. This table formalizes how procedural recruits compare to the Six Founders:

```text
DIMENSION                AUTHORED FOUNDERS (PHASE 1)                   PROCEDURAL RECRUITS (PHASE 2+)
========================================================================================================
Source & Story           Authored personal dossiers & questlines       Liberated from culling transport wagons
Visual Design            Unique hand-crafted sprites & visible taints  Procedural modular heads, aprons & rags
Innate Passive Trait     Character-defining exclusive trait            Rolled from general recruit trait table
Skill Tree Access        Base Class Trees + Authored Variants          Standard Base Class Trees (Branches A & B)
Signature Capstone       Exclusive character-locked capstone           Standard Base Class Keystones only
Permadeath Impact        Major narrative reaction & eulogy beat        Cell resilience: replaced by next recruit
Calcification Harvest    Yields unique named Apex Graft upon death     Yields standard class marrow organ
Safehouse Bunk           Assigned to permanent named bunks             Assigned to rotating reserve bunks
```

---

## 10. Engine Data Binding (`SkillDef` Representation)

To integrate these specifications into `emberwake_data::SkillDef` within the Rust engine, each skill maps to the expanded data schema:

```json
{
  "id": "vanguard_kaelen_seventh_ward_bulwark",
  "name": "Seventh Ward Bulwark",
  "archetype": "vanguard",
  "character_lock": "kaelen",
  "kind": "DEFEND",
  "branch": "COLD_STEEL",
  "tier": 3,
  "ap_cost": 3,
  "strain_cost": 0,
  "speed_tier": "HEAVY",
  "windup_ticks": [1, 2, 3],
  "impact_tick": 4,
  "recovery_ticks": [5, 6],
  "reach_min": 1,
  "reach_max": 1,
  "area": "LINE_3",
  "power_mult": 1.2,
  "stagger_damage": 35,
  "armor_piercing": false,
  "status_effect": "bulwark_zone",
  "recoil_pct": 0.0,
  "description": "Kaelen slams his tower shield into the flagstones, creating an impassable 3-tile barrier that mitigates 50% damage for allies and blocks ranged line of sight."
}
```
