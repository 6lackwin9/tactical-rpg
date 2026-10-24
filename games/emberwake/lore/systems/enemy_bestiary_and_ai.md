# Enemy Bestiary, Tactical AI & Modular Encounter Architecture

> **Category:** Combat Systems, Bestiary & Encounter Design  
> **Status:** Active Canonical Specification  
> **Location:** `games/emberwake/lore/systems/enemy_bestiary_and_ai.md`  
> **Reference Lore:** [combat_and_timeline.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/combat_and_timeline.md), [classes_and_skill_trees.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/classes_and_skill_trees.md), [factions_and_sociology.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/factions_and_sociology.md)

---

## 1. Executive Summary & Design Principles

Tactical combat in *Emberwake: The Low Sleep* requires that adversaries obey the exact same physical and temporal laws as player operatives. Every hostile entity schedules actions across the **6-Tick Simultaneous Timeline**, adheres to standardized **Speed Tiers** (*Snap*, *Standard*, *Heavy*), possesses a quantifiable **Poise Bar**, and telegraphed **Windup windows** that can be interrupted.

```text
                                [ ENEMY FACTION TAXONOMY ]
                                             |
             +-----------------------+-------+---------------+-----------------------+
             |                       |                       |                       |
             v                       v                       v                       v
    [ SOVEREIGN CLERISY ]   [ IRON INQUISITION ]     [ TALLOW GUILD ]      [ LOW SLEEP ABERRATIONS ]
    "Feudal Order & Law"    "Zealotry & Purity"     "Grease, Industry"    "Cosmic Titan Flesh"
    • Heavy Armor/Shields   • High Speed & Bleed    • Hazard Control      • True Armor Piercing
    • Chokepoint Defense    • Phosphorus Flames     • Area Denial Flamm.  • High Strain Generation
    • Telegraph Heavy Swings• Surgical Marrow       • Pack Pounce (Dogs)  • Burrowing / Ambushes
```

### 1.1 The Variant Design Rule: Base $\to$ Elite $\to$ Named Bosses
To maintain mathematical rigor and production efficiency, all hostile encounters are built from a strict hierarchical template:
1. **Base Adversaries:** Foundation archetype (e.g., *Castellan Footman*, *Mortuary Flagellant*). 1 Standard Maneuver, 1 Fast Snap, baseline Poise.
2. **Elite Adversaries:** Enhanced archetype with unique passive traits, expanded Poise bars, multi-tile telegraphs, and armor shredding (e.g., *Inquisitorial Purifier*, *Guild Carver*).
3. **Named Adversaries (Bosses & Mini-Bosses):** Authored character variants of Base and Elite templates possessing an **Innate Boss Trait**, an **Authored Maneuver Variant** (altered speed tier or range), and an **Exclusive Phase Trigger / Capstone**.

### 1.2 Modular Encounter Architecture (Narrative Decoupling)
Because the overarching narrative pacing and campaign mission graphs are still evolving, all encounters in this document are authored as **Modular Encounter Events (`ENC-EVT-XX`)**. Each event is a self-contained combat blueprint specifying:
- Tactical arena layout and environmental hazards.
- Hostile squad roster and named leader variant.
- Scalability parameters (can be deployed as a Main Campaign Story beat, an optional Side Bounty, or a Safehouse Defense breach).
- Dynamic modifiers and narrative harvest payoffs.

---

## 2. Faction Bestiary: Base & Elite Adversaries

### 2.1 The Sovereign Clerisy & City Watch
Clerisy troops enforce the feudal curfew of Cinderreach. They are heavily armored, disciplined, and rely on formation fighting and shield walls.

```text
+-----------------------+--------+-------+-------+-------+------------------------------------------+
| ADVERSARY NAME        | TIER   | HP    | ARMOR | POISE | TACTICAL ROLE & SPEED PROFILE            |
+-----------------------+--------+-------+-------+-------+------------------------------------------+
| Castellan Footman     | Base   | 65    | 3     | 35    | Frontline Anchor / Standard Billhook     |
| Watch Arbalestier     | Base   | 50    | 1     | 25    | Ranged Suppression / Snap Pinning Bolt   |
| Bailiff Enforcer      | Elite  | 110   | 4     | 55    | Heavy Disrupter / Telegraphed Mace Cleave|
+-----------------------+--------+-------+-------+-------+------------------------------------------+
```

#### 1. Castellan Footman (Base Melee)
* **Visual Brief:** Padded gambeson beneath dented breastplate with an unadorned iron kettle helm; carries an iron-rimmed heater shield and a short billhook pole.
* **Movement:** 3 tiles / round (T1–T3 Advance).
* **Maneuvers:**
  * *Billhook Hook (Standard | Impact T4 | Reach 1):* Deals 18 physical damage, +15 Stagger. If the target is moving, pulls them 1 tile closer and halts movement.
  * *Shield Wall Bracing (Snap | Impact T2 | Self):* Gains Active Guard (+40% mitigation, halves incoming stagger) from T2 through T6.
* **AI Heuristics:** Prioritizes moving into adjacent chokepoint tiles with allied footmen. If an ally within 2 tiles is attacked, commits *Shield Wall Bracing* on the next round.

#### 2. Watch Arbalestier (Base Ranged)
* **Visual Brief:** Woolen watch-cloak over boiled leather; carries a hand-cranked stirrup arbalest and a short side-dagger.
* **Movement:** 4 tiles / round.
* **Maneuvers:**
  * *Aimed Bolt (Standard | Windup T1–T2, Impact T4, Recovery T5–T6 | Range 4):* Deals 22 piercing damage (ignores 1 Armor), +10 Stagger. Entity-locked telegraph line.
  * *Buckler Jab (Snap | Impact T2 | Reach 1):* Fast defensive bash dealing 10 blunt damage and pushing the melee attacker 1 tile back.
* **AI Heuristics:** Seeks elevated or rubble tiles. Targets the player operative with the lowest current HP or an operative currently in an active windup to trigger interrupts. Retreats if engaged in melee.

#### 3. Bailiff Enforcer (Elite Heavy)
* **Visual Brief:** Towering city bailiff in full blackened plate, wearing an executioner's hood and swinging a heavy two-handed flanged mace wrapped in chains.
* **Innate Trait — Unyielding Authority:** Immune to knockback and forced movement; cannot be staggered by Snap attacks.
* **Maneuvers:**
  * *Crushing Judgment (Heavy | Windup T1–T4, Impact T5, Recovery T6 | Reach 1 | 3-Tile Frontal Cone):* Telegraphed ground-hazard decal (`[ ⚠ T5 ]`). Deals 45 crushing damage, shatters 3 Armor, and inflicts **Stunned** on all units in the cone.
  * *Iron Shackle Drag (Standard | Impact T4 | Range 3):* Throws weighted chains at an operative. Deals 15 physical damage, pulls target 2 tiles toward the Bailiff, and applies **Rooted** for 1 round.
* **AI Heuristics:** Advances relentlessly toward the player's tank or cluster of operatives. Readily commits to *Crushing Judgment*—inviting player interrupts or evasions.

---

### 2.2 The Iron Inquisition
The enforcement wing of the church. They fight with terrifying fanatical speed, utilizing self-mutilation, chemical phosphorus fire, and stolen marrow organs.

```text
+-----------------------+--------+-------+-------+-------+------------------------------------------+
| ADVERSARY NAME        | TIER   | HP    | ARMOR | POISE | TACTICAL ROLE & SPEED PROFILE            |
+-----------------------+--------+-------+-------+-------+------------------------------------------+
| Mortuary Flagellant   | Base   | 45    | 0     | 20    | Suicide Swarmer / Fast Snap Bleed Flail  |
| Inquisitorial Purifier| Elite  | 80    | 2     | 40    | Area Denial / Phosphorus Spray Hazards   |
| Inquisitor Proctor    | Elite  | 125   | 4     | 60    | Elite Duelist / Marrow Surge & Cleave    |
+-----------------------+--------+-------+-------+-------+------------------------------------------+
```

#### 1. Mortuary Flagellant (Base Fast Skirmisher)
* **Visual Brief:** Emaciated penitent bare-chested under blood-stained mortuary vestments; barbed wire bound tightly around forearms, wielding a three-tailed scourge.
* **Movement:** 5 tiles / round.
* **Innate Trait — Fanatical Agony:** When below 50% HP, gains +1 Movement tile and attacks gain **+20% Critical Hit Chance**.
* **Maneuvers:**
  * *Scourge Lash (Snap | Impact T2 | Reach 2):* Deals 14 slashing damage, +10 Stagger, and inflicts **Bleed** (6 dmg/round for 2 rounds).
  * *Martyr's Rush (Snap | Impact T2 | Reach 1):* Sacrifices 10 HP to leap 3 tiles and strike, dealing 20 physical damage and knocking target off-balance.
* **AI Heuristics:** Disregards personal safety. Rushes low-armor backline targets (Arbalestier, Alchemist, Acolyte), ignoring tank zones of control.

#### 2. Inquisitorial Purifier (Elite Area Denial)
* **Visual Brief:** Heavy warded iron apron, bronze bird-beak gas mask, carrying a pressurized sulfur canister strapped to the back and a long pole-mounted phosphorus nozzle.
* **Movement:** 3 tiles / round.
* **Innate Trait — Heat-Tempered Carapace:** 50% resistance to fire damage and corrosive acid; immune to Blind.
* **Maneuvers:**
  * *Pyre-Nozzle Sweep (Standard | Windup T1–T2, Impact T4, Recovery T5–T6 | Range 3 | 3x3 Ground AoE):* Paints a 3x3 ground-hazard decal (`[ ⚠ T4 ]`). On Tick 4, ignites the tiles in persistent **Phosphorus Fire** for 2 rounds (dealing 18 fire damage/tick and Burn).
  * *Brand of the Heretic (Snap | Impact T2 | Reach 1):* Thrusts a red-hot branding iron into an adjacent foe. Deals 15 fire damage, destroys 2 Armor, and cancels Active Guard.
* **AI Heuristics:** Identifies high-cover zones or clusters of player operatives and coats them in fire to flush units out into open sightlines.

#### 3. Inquisitor Proctor (Elite Duelist & Officer)
* **Visual Brief:** Polished mirror-sheen plate over deep crimson cassock; long executioner’s broadsword held in two hands; a warded tallow vial embedded in a throat-collar.
* **Movement:** 4 tiles / round.
* **Innate Trait — Cold Inquest:** Any operative who begins their turn within 2 tiles of the Proctor suffers -1 AP due to overwhelming dread.
* **Maneuvers:**
  * *Sanctified Decapitation (Heavy | Windup T1–T3, Impact T4, Recovery T5–T6 | Reach 1):* Tremendous two-handed broadsword cleave dealing 50 physical damage, +30 Stagger. Completely bypasses Active Guard mitigation.
  * *Inquisitorial Parry & Riposte (Snap | Impact T2 | Reach 1):* Enters a defensive stance on T2; if struck by a melee attack before T4, completely parries the damage and counters for 25 piercing damage.
* **AI Heuristics:** Challenges the player’s strongest melee hero (Kaelen or Torin). Alternates between Parries and telegraphed Heavy Cleaves, forcing players to read timeline cues.

---

### 2.3 The Tallow Guild
Industrial crime syndicate and corpse-rendering cartel operating the tallow vats and bone-mills. They use industrial butchery tools, scalding grease, and trained beasts.

```text
+-----------------------+--------+-------+-------+-------+------------------------------------------+
| ADVERSARY NAME        | TIER   | HP    | ARMOR | POISE | TACTICAL ROLE & SPEED PROFILE            |
+-----------------------+--------+-------+-------+-------+------------------------------------------+
| Tallow Dredger        | Base   | 55    | 1     | 25    | Skirmisher / Corrosive Grease & Knife    |
| Render-Beast          | Base   | 40    | 0     | 20    | Fast Beast / Snap Pounce & Chase         |
| Guild Carver          | Elite  | 105   | 3     | 50    | Brawler / Flensing Hooks & Body Slam     |
+-----------------------+--------+-------+-------+-------+------------------------------------------+
```

#### 1. Tallow Dredger (Base Skirmisher)
* **Visual Brief:** Leather skinner's apron slick with yellow corpse-grease; crude copper goggles; carries a wide boning knife and ceramic pots of boiling lard.
* **Movement:** 4 tiles / round.
* **Maneuvers:**
  * *Scalding Grease Pot (Standard | Impact T4 | Range 3 | 2x2 Ground AoE):* Throws boiling lard. Creates a slippery, flammable grease puddle for 2 rounds. Units entering lose -2 Movement and become **Flammable** (+50% fire damage taken).
  * *Boning Knife Flurry (Snap | Impact T2 | Reach 1):* Rapid dual slashes dealing 14 physical damage and applying Bleed.
* **AI Heuristics:** Throws grease pots in front of advancing player melee units to impede movement, then flanks from the side.

#### 2. Render-Beast (Base Fast Beast)
* **Visual Brief:** Starved, hairless mastiff-sized beast with ossified bone ridges along its spine and jaws reinforced with iron wire; trained on human flesh.
* **Movement:** 6 tiles / round (Rapid Sprint).
* **Innate Trait — Tenacious Chase:** If its target moves during its windup ticks, the Render-Beast automatically chases up to 2 tiles to maintain melee contact.
* **Maneuvers:**
  * *Throat-Clamp Pounce (Snap | Impact T2 | Reach 1):* Leaps 2 tiles and clamps onto target. Deals 18 physical damage, +20 Stagger, and applies **Pinned** (both unit and beast cannot move until beast takes damage).
* **AI Heuristics:** Operates in pairs. Bypasses frontliners to pin isolated ranged operatives (Bran, Elyse).

#### 3. Guild Carver (Elite Brawler)
* **Visual Brief:** Obese, muscled slaughterhouse foreman wearing a spiked iron belly-plate and wielding a pair of heavy serrated meat hooks.
* **Movement:** 3 tiles / round.
* **Innate Trait — Meat-Hook Pull:** Landing an attack with meat hooks pulls the target 1 tile closer and reduces their evasion to 0%.
* **Maneuvers:**
  * *Flensing Hook Drag (Standard | Impact T4 | Range 3):* Launches hooked chain. Deals 24 piercing damage, drags target into an adjacent tile, and inflicts **Exposed** (-3 Armor).
  * *Butcher's Cleave (Heavy | Windup T1–T3, Impact T4, Recovery T5–T6 | Reach 1):* Crushing downward hack dealing 42 physical damage and +35 Stagger. If target is below 40% HP, deals double damage (84).
* **AI Heuristics:** Drags backline heroes into melee range, then executes them with *Butcher's Cleave*.

---

### 2.4 Subterranean Aberrations (Low Sleep Terrors)
Mutated biological horrors born from the black marrow venting from the sleeping leviathan deep beneath the city foundations.

```text
+-----------------------+--------+-------+-------+-------+------------------------------------------+
| ADVERSARY NAME        | TIER   | HP    | ARMOR | POISE | TACTICAL ROLE & SPEED PROFILE            |
+-----------------------+--------+-------+-------+-------+------------------------------------------+
| Ash-Crawler           | Base   | 35    | 1     | 15    | Burrower / Acid Spit & Ambush            |
| Calcified Husk        | Base   | 75    | 5     | 45    | Tank / Osseous Carapace & Shrapnel       |
| Marrow Horror         | Elite  | 140   | 2     | 65    | Eldritch Terror / High Strain & Mind Fog |
+-----------------------+--------+-------+-------+-------+------------------------------------------+
```

#### 1. Ash-Crawler (Base Burrower)
* **Visual Brief:** Multi-legged chitinous centipede-like monstrosity with a translucent abdomen filled with bubbling violet acid.
* **Movement:** 4 tiles / round (can tunnel through rubble).
* **Maneuvers:**
  * *Burrow Ambush (Snap | Impact T2 | Range 3):* Submerges into stone dust and erupts beneath a target within 3 tiles. Deals 15 true damage and inflicts **Staggered**.
  * *Caustic Vomit (Standard | Impact T4 | Range 3):* Spits mutagenic acid dealing 16 corrosive damage and dissolving 2 Armor Rating.
* **AI Heuristics:** Tunnels behind player lines to strike operatives standing on rubble or in fog.

#### 2. Calcified Husk (Base Tank)
* **Visual Brief:** The petrified corpse of a former miner or unsealed youth whose entire skeleton has ossified into brittle, razor-sharp black bone jutting through decayed skin.
* **Movement:** 2 tiles / round.
* **Innate Trait — Osseous Shrapnel:** When its Poise Bar is broken (suffering an interrupt), its ribcage violently shatters outward, dealing 20 piercing damage to all adjacent units.
* **Maneuvers:**
  * *Bone-Club Slam (Standard | Impact T4 | Reach 1):* Slams its heavy, petrified forearm into the target. Deals 22 blunt damage and +25 Stagger.
* **AI Heuristics:** Mindlessly walks toward the nearest living target. Players must calculate when to break its poise to avoid taking shrapnel recoil.

#### 3. Marrow Horror (Elite Eldritch Aberration)
* **Visual Brief:** Grotesque amorphous mass of black muscle tendrils, fused human skulls, and pulsing violet marrow vents that ripple in sync with the sleeping titan's heartbeat.
* **Movement:** 3 tiles / round.
* **Innate Trait — Resonance Aura:** Any player operative who ends the round within 3 tiles of the Marrow Horror automatically accumulates **+10% Strain**.
* **Maneuvers:**
  * *Marrow Tendril Flail (Standard | Impact T4 | Reach 2 | 3-Tile Sweep):* Sweeps blackened tendrils across a 3-tile arc. Deals 28 true eldritch damage (ignoring armor) and inflicts **Bleed**.
  * *Leviathan's Whispers (Heavy | Windup T1–T3, Impact T4, Recovery T5–T6 | Range 4 | Single Target):* Emits a psychological tremor. Deals no physical damage, but inflicts **+30% Strain** and **Nightmare Hallucination** (50% chance to attack nearest ally next turn).
* **AI Heuristics:** Uses tendril sweeps to push player strain gauges into dangerous brackets, attempting to trigger spontaneous Aberration Crises.

---

## 3. Named Adversaries: Boss & Mini-Boss Specializations

Just as the Six Founders possess unique traits and signature maneuvers over base classes, **Named Adversaries** are authored specializations of the base and elite templates.

```text
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| NAMED ADVERSARY       | BASE TEMPLATE VARIANT       | INNATE BOSS TRAIT             | SIGNATURE CAPSTONE MANEUVER      |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| 1. Captain Vane       | Clerisy Bailiff Enforcer    | Traitor's Discipline          | Seventh Ward Execution Wall      |
| 2. Proctor Malakar    | Inquisitor Proctor          | Zealous Marrow Graft          | Sacrificial Transfusion Pulse    |
| 3. Guild-Master Orlov | Tallow Guild Carver         | Vat-Master's Caustic Grease   | Scalding Flensing Cataclysm      |
| 4. Overseer Grendel   | Slag-Brute / Foundry Stoker | Molten Slag Tempering         | Bone-Mill Pulverizer             |
| 5. Calcified Amalgam  | Marrow Horror + Husk Amalgam| Resonant Titan Nexus          | Catastrophic Ossification Bloom  |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
```

---

### 3.1 Captain Vane (The Watch Traitor)
* **Lore Tether:** Kaelen’s former commanding officer in the Seventh Ward Watch who betrayed his own men to the Inquisition in exchange for a church commission. Wears reinforced gilded plate and carries an executioner's billhook and heavy pavise shield.
* **Base Template:** Elite Bailiff Enforcer variant.
* **Stats:** HP: 190 | Armor: 5 | Poise: 75 | Move: 3 tiles/round.

#### 1. Innate Boss Trait — *Traitor's Discipline*
* Allied Clerisy soldiers within 3 tiles of Vane gain **+10 Poise** and cannot be broken by Snap attacks.
* If attacked by Kaelen, Vane’s counter-parry triggers automatically on Tick 2.

#### 2. Authored Skill Variant — *Gilded Billhook Sunder* (Modifies *Crushing Judgment*)
* *Profile:* Heavy | Windup T1–T3, Impact T4, Recovery T5–T6 | Reach 1 | 3-Tile Arc.
* *Mechanics:* Strikes 1 tick faster than standard Bailiff attacks (Impact on T4). Deals 38 physical damage, strips 4 Armor Rating, and knocks targets back 2 tiles into rubble.

#### 3. Exclusive Signature Capstone — *Seventh Ward Execution Wall*
* *Trigger:* Unlocked at 50% HP or Round 3.
* *Profile:* Heavy | Windup T1–T4, Impact T5, Recovery T6 | Range 4 | Linear Wall 4.
* *Mechanics:* Vane signals two shield-bearing guards to advance with him, locking shields. Creates a moving barrier that advances 2 tiles, dealing 30 blunt damage, knocking down all player heroes, and pinning them against walls.

---

### 3.2 Proctor Malakar (The Scalpel of St. Agnes)
* **Lore Tether:** Defrocked Sister Maren and oversaw the autopsy of her mentor. Wears ceremonial white linen cassocks soaked in dried black marrow and uses twin sanctified bone-saws.
* **Base Template:** Elite Inquisitor Proctor variant.
* **Stats:** HP: 165 | Armor: 3 | Poise: 65 | Move: 4 tiles/round.

#### 1. Innate Boss Trait — *Zealous Marrow Graft*
* Malakar has surgically installed an illegal titan marrow valve in his neck.
* When he takes damage, he generates Strain (up to 100%). At 50%+ Strain (*Flesh-Tension*), his attacks completely bypass player armor and deal pure corrosive eldritch damage.

#### 2. Authored Skill Variant — *Sanctified Vivisection* (Modifies *Sanctified Decapitation*)
* *Profile:* Standard | Windup T1–T2, Impact T3, Recovery T4–T5 | Reach 1.
* *Mechanics:* Fast dual bone-saw cross. Deals 35 true damage, +25 Stagger, and inflicts **Arterial Rupture** (10 bleed damage per tick whenever the victim moves).

#### 3. Exclusive Signature Capstone — *Sacrificial Transfusion Pulse*
* *Trigger:* When reaching 100% Strain or upon taking fatal damage.
* *Profile:* Snap | Impact T2 | 3-Tile Radius Radial AoE.
* *Mechanics:* Malakar forcibly vents his internal marrow pressure. Explodes in a shockwave of boiling black blood dealing 25 corrosive damage to all heroes and transferring **+25% Strain** into every player operative's gauge. He then knits 50 HP back into his flesh.

---

### 3.3 Guild-Master Orlov (The Ash-Crawl Butcher)
* **Lore Tether:** The ruthless crime lord of Lower Cinderreach who controls the tallow rendering pipelines and murdered Vesper’s family. Fights wearing an iron-plated butcher’s harness, wielding a steam-pressured lard cannon and barbed flensing hooks.
* **Base Template:** Elite Guild Carver variant.
* **Stats:** HP: 210 | Armor: 4 | Poise: 80 | Move: 3 tiles/round.

#### 1. Innate Boss Trait — *Vat-Master's Caustic Grease*
* The arena floor around Orlov is slick with boiling corpse-fat.
* Any operative moving within 2 tiles of Orlov must pass a Poise check or slip, losing 2 movement ticks and becoming Flammable.

#### 2. Authored Skill Variant — *Pressurized Lard Spray* (Modifies *Flensing Hook Drag*)
* *Profile:* Standard | Windup T1–T2, Impact T4, Recovery T5–T6 | Range 4 | 3-Tile Frontal Cone.
* *Mechanics:* Coats a 3-tile cone in scalding yellow grease. Deals 22 fire damage, inflicts **Blind** for 1 round, and leaves the floor burning for 2 rounds.

#### 3. Exclusive Signature Capstone — *Scalding Flensing Cataclysm*
* *Trigger:* When Vesper attacks him or at 40% HP.
* *Profile:* Heavy | Windup T1–T3, Impact T4, Recovery T5–T6 | Reach 2 | Multi-Target.
* *Mechanics:* Orlov spins with both flensing chains extended, dragging all units within Reach 2 directly into his boiling vat harness. Deals 45 physical damage, shreds 4 Armor, and inflicts Bleed and Burn simultaneously.

---

### 3.4 Overseer Grendel (The Bone-Mill Slaver)
* **Lore Tether:** The brutal overseer of the Fourth Kiln who enslaved Torin "Slag-Jaw" and branded his face with molten iron. Fights with a massive iron-spiked coal-shovel and an industrial pneumatic slag-hammer.
* **Base Template:** Elite Slag-Brute / Foundry Stoker variant.
* **Stats:** HP: 230 | Armor: 4 | Poise: 85 | Move: 3 tiles/round.

#### 1. Innate Boss Trait — *Molten Slag Tempering*
* Grendel's weapon glows red-hot. His physical attacks ignite combustible terrain and melt 1 point of player Armor Rating on every successful hit.
* Cannot be staggered while standing on burning or rubble tiles.

#### 2. Authored Skill Variant — *Pneumatic Hammer Strike* (Modifies *Butcher's Cleave*)
* *Profile:* Heavy | Windup T1–T3, Impact T4, Recovery T5–T6 | Reach 1 | Single Target + Shockwave.
* *Mechanics:* Drives a steam-powered piston hammer into the flagstones. Deals 48 blunt damage, +40 Stagger to primary target, and sends a radial ground shockwave knocking down all adjacent units.

#### 3. Exclusive Signature Capstone — *Bone-Mill Pulverizer*
* *Trigger:* Enters at 50% HP.
* *Profile:* Heavy | Windup T1–T4, Impact T5, Recovery T6 | Range 3 | 3x3 Demolition Zone.
* *Mechanics:* Smashes the arena's architectural pillars. Permanently destroys the target 3x3 grid area, dropping heavy masonry that deals 50 crushing damage to caught units and leaves the area impassable rubble.

---

### 3.5 The Calcified Amalgam (The Dreamer's Whelp)
* **Lore Tether:** A hideous subterranean chimera discovered in the deepest storm-drains beneath Cinderreach. Composed of dozens of calcified corpses fused together by pulsing black marrow tendrils, radiating the psychic frequencies of the slumbering titan.
* **Base Template:** Elite Marrow Horror + Calcified Husk fusion.
* **Stats:** HP: 260 | Armor: 6 | Poise: 90 | Move: 2 tiles/round.

#### 1. Innate Boss Trait — *Resonant Titan Nexus*
* Every 2 rounds, when the city cathedral bells toll, the Amalgam pulses.
* All player operatives who possess $\ge 1$ point of Calcification or $\ge 30\%$ Strain immediately suffer 15 true damage and +10% Strain.

#### 2. Authored Skill Variant — *Ossified Spine Volley* (Modifies *Marrow Tendril Flail*)
* *Profile:* Standard | Windup T1–T2, Impact T4, Recovery T5–T6 | Range 5 | 360-Degree Radial Burst.
* *Mechanics:* Fires a storm of calcified bone needles in all directions. Deals 24 piercing damage (ignoring armor) to all units within a 4-tile radius and inflicts Bleed.

#### 3. Exclusive Signature Capstone — *Catastrophic Ossification Bloom*
* *Trigger:* Upon Poise Break or at 30% HP.
* *Profile:* Heavy | Windup T1–T4, Impact T5, Recovery T6 | Full Arena Battlefield.
* *Mechanics:* The Amalgam anchors itself to the bedrock. On Tick 5, massive black bone spires punch through the floor across the arena. Any unit not standing on safe designated warded tiles takes 40 physical damage and has their legs **Calcified** (locked in stone for 1 round).

---

## 4. Modular Encounter Architecture (Placeholder Events)

To decouple encounter design from the unfinished campaign story, the following **six placeholder encounter events (`ENC-EVT-01` to `ENC-EVT-06`)** define self-contained tactical scenarios. As the campaign narrative crystallizes, these modules can be slotted into main story missions, crisis incursions, or optional side quests, scaling dynamically based on player phase.

```text
+--------------+----------------------------------+---------------------+-----------------------------------+
| EVENT ID     | WORKING TITLE                    | KEY ADVERSARY       | NARRATIVE ROLE FLEXIBILITY        |
+--------------+----------------------------------+---------------------+-----------------------------------+
| ENC-EVT-01   | Seventh Ward Alleyway Ambush     | Captain Vane        | Main Intro / Side Bounty / Crisis |
| ENC-EVT-02   | The Mortuary Cart Hijack         | Proctor Malakar     | Main Heist / Prison Wagon Rescue  |
| ENC-EVT-03   | The Bone-Mill Crucible Raid      | Overseer Grendel    | Slag-Brute Rescue / Foundry Raid  |
| ENC-EVT-04   | The Tallow Vat Incursion         | Guild-Master Orlov  | Syndicate Confrontation / Sabotage|
| ENC-EVT-05   | The Awakening Fissure            | Calcified Amalgam   | Deep Undercroft Breach / Horror   |
| ENC-EVT-06   | The Crucible Watchpost Climax    | High Inquisitor Bast| Phase 1 Climax / Scripted Sacrifice|
+--------------+----------------------------------+---------------------+-----------------------------------+
```

---

### 4.1 ENC-EVT-01: The Seventh Ward Alleyway Ambush
* **Concept:** A tight, claustrophobic street confrontation in rain-slick cobblestone alleys flanked by tall timber tenement walls.
* **Tactical Layout:** 12x10 grid with narrow 2-tile choke alleys, low wooden crates (half-cover), and high rain-gutters (sniping platforms).
* **Adversary Roster:**
  * Leader: **Captain Vane** (Bailiff Enforcer variant).
  * Minions: 2x Castellan Footmen, 1x Watch Arbalestier (perched on elevation).
* **Dynamic Modifiers:**
  * *Rain-Slick Cobblestones:* Moving more than 2 tiles in a single tick requires a DEX check or unit slips and loses 1 tick.
* **Scalability Levers:**
  * *Early Phase 1 (Tutorial/Prologue):* Vane starts at 60% HP without his signature capstone; footmen have no shields.
  * *Late Phase 1 / Side Bounty:* Vane has full HP, 2 additional Arbalestiers, and reinforcements arrive on Round 3.
* **Harvest & Narrative Payoff:** Vane's *Castellan Billhook* (Cold Steel T2 weapon), 25 Scrap Iron, City Gate Pass cipher.

---

### 4.2 ENC-EVT-02: The Mortuary Cart Hijack
* **Concept:** An assault on an Inquisition prisoner-transport wagon carrying captured "unsealed" youths to the dissection slabs.
* **Tactical Layout:** 14x10 grid. Center features a heavy iron cage wagon (impassable block line) flanked by burning street lanterns.
* **Adversary Roster:**
  * Leader: **Proctor Malakar** (Inquisitor Proctor variant).
  * Minions: 1x Inquisitorial Purifier, 2x Mortuary Flagellants.
* **Dynamic Objective:**
  * *The Culling Timer:* Flagellants will execute 1 captive youth every 2 rounds. Player must rush or suppress flagellants to secure secondary recruits.
* **Scalability Levers:**
  * *Main Story Mission:* Rescues Sister Maren and Bran from the cage.
  * *Phase 2 Procedural Rescue:* Rescues 2 procedural young-blood recruits with random class proficiencies.
* **Harvest & Narrative Payoff:** Malakar’s *Surgical Bone-Saws*, 2 vials of *Warded Tallow*, 1 *Inquisitor Marrow Gland*.

---

### 4.3 ENC-EVT-03: The Bone-Mill Crucible Raid
* **Concept:** An industrial assault inside a roaring foundry kiln filled with grinding gears, bone pulverizers, and rivers of molten slag.
* **Tactical Layout:** 14x12 grid. Features conveyor belts that push units 1 tile per tick toward crushing rollers; open slag pools acting as lethal hazard zones.
* **Adversary Roster:**
  * Leader: **Overseer Grendel** (Slag-Brute variant).
  * Minions: 2x Tallow Dredgers, 1x Bailiff Enforcer.
* **Dynamic Hazard:**
  * *Slag Vent Burst:* On Ticks 3 and 6, steam vents vent boiling steam across column 4 and column 8 (deals 20 fire damage to anyone standing there).
* **Scalability Levers:**
  * *Main Story Mission:* Rescues Torin "Slag-Jaw" from the penal chain gang.
  * *Side Raid:* Disables the city’s cement mortar production, reducing Inquisition alert meter by 25%.
* **Harvest & Narrative Payoff:** Grendel’s *Pneumatic Hammer*, 50 Scrap Iron, *Slag-Tempered Plating* (Armor upgrade).

---

### 4.4 ENC-EVT-04: The Tallow Vat Incursion
* **Concept:** Infiltrating the subterranean tallow storage vats beneath the tanneries. Dim lighting, caustic fumes, and narrow wooden cat-walks over boiling fat.
* **Tactical Layout:** 16x10 grid with narrow 1-tile walkways over vat pits; falling into a vat deals 40 acid/fire damage.
* **Adversary Roster:**
  * Leader: **Guild-Master Orlov** (Guild Carver variant).
  * Minions: 2x Render-Beasts, 2x Tallow Dredgers.
* **Dynamic Modifiers:**
  * *Grease Miasma:* Units lose 1 AP while standing in low-lying vapor unless wearing leather masks or wet rags.
* **Scalability Levers:**
  * *Vesper’s Personal Vendetta Quest:* Grants Vesper unique dialogue and unlocks her *Orlov's Retribution* skill.
  * *Economic Sabotage Side Mission:* Hijacks the Guild's tallow shipment, granting 5 daily rations of safehouse warded fuel.
* **Harvest & Narrative Payoff:** Orlov’s *Flensing Chain Hooks*, *Pressurized Lard Sprayer*, 100 Gold, 5x *Warded Tallow Vials*.

---

### 4.5 ENC-EVT-05: The Awakening Fissure
* **Concept:** An emergency subterranean crisis deep beneath the Sluice Undercroft. The stone floor has cracked open, revealing a throbbing leviathan vein.
* **Tactical Layout:** 12x12 cavern grid. Uneven terrain, jagged black bone pillars (full cover), and purple mutagenic mist pools.
* **Adversary Roster:**
  * Leader: **The Calcified Amalgam** (Marrow Horror variant).
  * Minions: 3x Ash-Crawlers, 2x Calcified Husks.
* **Dynamic Crisis:**
  * *Aberrant Proximity:* Every round, player operatives accumulate +5% Strain passively just by standing in the chamber.
* **Scalability Levers:**
  * *Phase 1 Endgame Teaser:* Sister Aline provides the emergency syringe, allowing 1 hero to experience Inoculation temporarily.
  * *Phase 2 Macro Crisis:* Triggers as a Safehouse Defense incursion if Inquisition Alert hits 80%.
* **Harvest & Narrative Payoff:** 1x *Apex Leviathan Marrow Core* (required to unlock high-tier Inoculation grafts at Sister Aline’s slab).

---

### 4.6 ENC-EVT-06: The Crucible Watchpost Climax
* **Concept:** The fortified watchpost separating the Sunk Ward from the High Cathedral district. Heavy iron portcullises, burning braziers, and siege ballistas.
* **Tactical Layout:** 18x12 grid divided by an iron portcullis. Winch mechanisms on east and west flanks required to open the gate.
* **Adversary Roster:**
  * Leader: **High Inquisitor Bastion** (Elite Proctor variant).
  * Minions: 2x Inquisitorial Purifiers, 2x Castellan Footmen, 2x Watch Arbalestiers.
* **The Dramatic Climax:**
  * *The Scripted Tactical Sacrifice:* The portcullis winch is jammed by iron bolts. One operative must remain on the winch tile to hold the lever against continuous enemy fire, allowing the remaining 3 squad members to escape the collapsing ward.
* **Scalability Levers:**
  * Serves as the canonical transitional climax of Phase 1, unlocking Phase 2, the Apothecary Slab, and the Bitter Inoculation.
* **Harvest & Narrative Payoff:** Unlocks the High Ward map routes, leaves a permanent martyr memorial in the Dry Hollows, and yields the fallen founder's gear and personal heirloom.

---

## 5. Summary & Technical Integration Checklist

```text
[ emberwake_data ]
  └── models/enemy.rs        ---> Implement EnemyDef: hp, armor, poise, speed_tier, maneuver_deck.
  └── catalogs/enemies.json  ---> Catalog 12 base/elite adversaries + 5 named bosses.

[ emberwake_battle ]
  └── ai.rs                  ---> Implement timeline intent heuristics (Windup selection, Guard triggers).
  └── encounter.rs           ---> Bind modular ENC-EVT-01 through 06 into encounter generators.
```
